# Task DOC-269GT-COMPACT: Proof-Given Type Completion Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-269GT-COMPACT.md](../ja/DOC-269GT-COMPACT.md).

This derived documentation-maintenance contract freezes one completed
checker-first task family before deletion. It cannot introduce or reinterpret
language behavior, test intent, API, diagnostics, traceability, or coverage
credit.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-269GT-COMPACT` |
| Status | Exact migration, independent reviews, required verification, and all nine hard gates complete at uncapped 100/100. Exact staging and the separate migration commit remain. |
| Purpose | Centralize Task-269GT implementation-completion evidence while retaining every prerequisite, verification, H2 product, runner, trace, TODO, and semantic owner. |
| Owners | Repository migration policy, [historical 269GT contract](./269GT.md#completion-evidence), [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index), and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Consumers | 38 checker-first EN/JA design documents, four Task Indexes, and the post-migration schema-v1 ledger/lint |
| Dependencies | This batch's prerequisite `133128bc`; Task-269GT prerequisite `35bc97b9`; implementation `1fc6cc01`; generic manifest consumer `0ec5fce2`; prior compaction `f3dd80bc` |
| Readiness | Post-prerequisite clean HEAD `133128bc`, `origin/main...HEAD=0/2`, and protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`; all 38 preimages replayed before editing and no blocking authority gap exists. |

## Authority And Classification

Authority is the user's checker-first consolidation decision,
[`AGENTS.md`](../../../../AGENTS.md), the
[migration policy](../../autonomous_crate_development.md#migration-policy),
the completed Task-269GT records, and their frozen H2/H3 design owners. Source
behavior is not normative for this task.

| Class | Decision |
|---|---|
| `design_drift` | 38 completion-only H3 sections repeat overlapping implementation status, measurements, exclusions, trace state, and review evidence across 38 paths. Their exact preimages and fact owners must be frozen before removal. |
| `test_gap` | None. Schema v1 accepts one complete-section redirect per unique `(path, task)`, and the generic 15-test consumer already covers this shape. |
| `spec_gap` | None for this structural migration; no semantic issue is selected. |
| `source_drift` | None; production source is protected. |
| `source_undocumented_behavior` | None introduced or inferred. |
| `test_expectation_drift` | None; specification, `.miz`, fixtures, sidecars, expectations, trace TOML, and metadata are protected. |
| `boundary_violation` | Avoided by moving historical completion measurements to the paired 269GT contract while retaining every H2 and the eight plan-local prerequisite/verification H3s. |
| `repo_metadata_conflict` | None. The two local commits ahead of `origin/main` are the expected preceding compaction and this batch's prerequisite; this state is report-only and no repair or push is authorized. |

## Frozen Preimage Inventory

The language-neutral
[`DOC-269GT-COMPACT.sources.tsv`](../DOC-269GT-COMPACT.sources.tsv)
contains exactly 38 byte-sorted data rows plus two comments and final LF. Each
row records task, language, component, exact path, ATX level, exact heading
text without the prefix, complete-section SHA-256, and physical lines. The
raw heading is reconstructed as three `#` bytes, one space, and the heading
text; the section ends immediately before the next visible H3-or-higher ATX
heading. Replay against clean `f3dd80bc` must match every row before migration.

The data-row SHA-256, excluding the two comments, is
`62cec2bf5412bd1ea89791b4df75dd1709d182d4f1aad699af8ffe988725482a`.
The 40-line physical TSV SHA-256 is
`1dfde440f4ad4a2b7f203dee472640ccb0ea7cba2e6d937b2eeedaeac6809d86`.
These values are review evidence, not the future manifest expanded-inventory
hash.

| Component | Relative file | Selected section per language |
|---|---|---|
| mizar-checker | `00.crate_plan.md` | Task-269GT implementation status |
| mizar-checker | `bilingual_sync_audit.md` | implementation synchronization |
| mizar-checker | `binding_env.md` | implemented overlay |
| mizar-checker | `module_boundary_audit.md` | implemented boundary |
| mizar-checker | `payload_family_decomposition.md` | implemented payload delta |
| mizar-checker | `resolved_typed_ast.md` | implemented final owner |
| mizar-checker | `semantic_spec_audit.md` | implementation semantic audit |
| mizar-checker | `source_proof_local_declaration.md` | implemented consumer status |
| mizar-checker | `source_spec_audit.md` | implemented source/API delta |
| mizar-checker | `source_statement.md` | implemented statement boundary |
| mizar-checker | `source_type.md` | implementation verification status |
| mizar-checker | `todo.md` | implementation handoff |
| mizar-checker | `typed_ast.md` | implemented typed owner |
| mizar-test | `00.crate_plan.md` | Task-269GT implementation status |
| mizar-test | `bilingual_sync_audit.md` | implementation synchronization |
| mizar-test | `harness.md` | implemented dormant harness |
| mizar-test | `module_boundary_audit.md` | implemented runner boundary |
| mizar-test | `todo.md` | implementation handoff |
| mizar-test | `traceability.md` | implementation trace status |
| **Total** | **19 paired relative files / 38 physical paths** | **19 EN + 19 JA; 205 physical lines** |

All selected `(path, task)` identities are unique. The exact plan-local H3s
that remain are EN `Task-269GT frozen source-type prerequisite`, EN
`Task-269GT documentation-prerequisite verification`, EN/JA
`Task-269GT frozen dormant type consumer`, JA
`Task-269GT frozen source-type prerequisite`, and the two JA
`Task-269GT documentation prerequisite verification` spellings. Those eight
sections and every H2 are outside the TSV.

## Documentation-Prerequisite Scope

The prerequisite changes exactly nine paths: this EN/JA pair, the paired
historical 269GT contract, the language-neutral source TSV, and the checker/
test EN/JA crate plans. Each plan receives two language-local Task Index
records—269GT and this batch—for eight records total.

It does not change any of the 38 migration sources' selected sections, the
existing `legacy_compactions.tsv`, Rust, Cargo, specification, `.miz`, fixture,
sidecar, expectation, trace TOML, metadata, root audit, count/hash/status, or
executable behavior. `doc/design/spec_coverage_audit.md` remains unchanged
because coverage, design mapping, follow-up ownership, trace status, and
semantic deferrals do not change.

## Frozen Migration And Ownership Boundary

After the prerequisite commit and fresh replay, implementation replaces each
listed complete H3 section with one language-local redirect to the historical
contract's `#completion-evidence`. It changes only the 38 source paths, this
EN/JA status/evidence pair, and `legacy_compactions.tsv`: 41 paths total. The
ledger adds one batch, one task, 38 redirects over 38 sources, and eight index
records with a new independently computed expanded-inventory hash. The source
TSV and historical contract remain immutable.

The historical contract owns completion measurements and review evidence.
Module, runner, audit, trace, bilingual, TODO, and plan documents retain their
durable H2 contracts. The four plans also retain all eight frozen-prerequisite
and documentation-verification H3s. No H2 or unlisted H3 may be deleted or
rewritten.

The migration preserves only Task-269GT's frozen source-provenance result. It
cannot add direct Given-binding or generic type behavior, condition/fact,
existential/Skolem meaning, assumption/guard, goal/initial obligation,
use/capture/export, proof/discharge/acceptance, Core/CFG/VC/ATP, active
dispatch, coverage credit, or later-task behavior.

## Documentation-Prerequisite Evidence

Independent contract, test-sufficiency, and equivalence reviews ended **NO
FINDINGS** after correcting the historical prerequisite commit to
`35bc97b92ce075226105e8fcd4c1e43c8621995c`. Parent replay found exactly 38
selected sections, 38 unique paths, 205 physical lines, 19 EN and 19 JA paths,
26 checker and 12 runner paths, eight excluded plan-local H3s, and eight new
Task Index records. The data-row and physical TSV hashes matched the frozen
values above; the protected trace hash remained
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.

Recursive task-contract/link/fragment lint, all 15 full runner lint-policy
tests, 530 checker library tests, 600 runner library tests, 137 metadata tests,
all 15 checker lint-policy tests, `cargo fmt --all --check`, Cargo metadata,
warnings-denied all-target/all-feature Clippy, the full workspace test suite,
and `git diff --check` passed. The five CLI stdout hashes remained:

| Route | SHA-256 |
|---|---|
| plan | `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718` |
| parse-only | `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56` |
| declaration-symbol | `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74` |
| type-elaboration | `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f` |
| proof-verification | `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450` |

The prerequisite remains exactly nine documentation paths. Production,
specification, tests, fixtures, expectations, traceability, ledger, and the
root coverage audit remain unchanged. The final read-only quality review ended
**NO FINDINGS**, passed all nine hard gates, applied no score cap, and assigned
**100/100**. The commit identity is recorded after exact staging and commit.

## Implementation Evidence

The documentation prerequisite is commit
`133128bc4d4909b8be8c3c2b5f8206fe8b94649b`. Fresh post-commit inventory was
clean at `origin/main...HEAD=0/2`, and the protected stash was unchanged. Before
editing, parent and independent deterministic replay matched all 38 frozen
preimages, their headings, hashes, physical line counts, language/component
partition, and neighboring anchors.

The mechanical migration changes exactly the 38 declared source documents,
this EN/JA status/evidence pair, and `legacy_compactions.tsv`: 41 paths. The 38
complete sections occupied 205 physical lines, including their 38 separator
blank lines. Replacement removes 167 completion-content lines, adds exactly 38
standard language-local redirect lines, preserves those separators, and
therefore reduces the mapped intervals by 129 lines. All 38 forbidden headings
are absent, all 38 redirects are unique, every H2 and all eight excluded
plan-local H3s remain, and no unlisted task section changed.

The 450-line ledger adds exactly one batch, one task record, 38 redirects over
38 distinct source paths, and eight index records. Its expanded-inventory
SHA-256 is
`319638d715de101065fe65fd16a15f7bacbc07dc52db12dd8479cbcd492ad5e2`;
its complete physical SHA-256 is
`8c896ee2812b36435113bfb55cd1f65885d5d329d967401fa9251ad4c935ca37`.
The source inventory remains physically
`1dfde440f4ad4a2b7f203dee472640ccb0ea7cba2e6d937b2eeedaeac6809d86`.

Specification, `.miz`, fixture, sidecar, expectation, trace TOML/status/
backlinks, coverage credit, active outcomes, production, Cargo, public API,
diagnostics, root coverage audit, source inventory, and historical 269GT
contract are unchanged. The paired traceability design documents change only
by redirecting their selected completion evidence. The protected trace hash
remains
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.

Independent test-sufficiency and equivalence/boundary reviews ended **NO
FINDINGS**. Source/document/EN-JA review found one medium stale repository-
distance sentence; after synchronizing the expected two-commit distance, its
finding-specific re-review ended **NO FINDINGS**.

Focused and full runner lint policy passed 1/1 and 15/15, checker and runner
libraries passed 530/530 and 600/600, runner metadata passed 137/137, and
checker lint passed 15/15. `cargo fmt --all --check`, Cargo metadata,
warnings-denied all-target/all-feature Clippy, the full workspace test suite,
all five CLIs with the prerequisite hashes above, protected count/hash replay,
and `git diff --check` passed. The final read-only quality review ended **NO
FINDINGS**, passed all nine hard gates, applied no score cap, and assigned
**100/100** with no residual risk inside scope. Exact staging and commit
identity remain to be recorded.

## Tests, Reviews, And Exit

Prerequisite review independently verifies the 38 preimages, fact ownership,
retained H2/H3 exclusions, EN/JA equivalence, plan indexes, and local links.
All reviews must end **NO FINDINGS**. Verification includes TSV replay and
count/hash checks, recursive task-contract pair/link/fragment lint, the full
lint policy, checker/runner libraries, metadata, checker lint, formatting,
Cargo metadata, warnings-denied workspace Clippy, full tests, all five CLIs
and protected hashes, protected-scope inspection, `git diff --check`, exact
nine-path staging, and all nine gates with uncapped score `>=90/100`.

After one docs-only prerequisite commit, fresh inventory reselects the same
batch before implementation. Implementation receives separate test-
sufficiency, equivalence/boundary, source/document/EN-JA, and final-quality
reviews, exact 41-path staging, and one separate commit. The agent does not
push. Parent reasoning remains `xhigh`; bounded reviews may use `high`.
