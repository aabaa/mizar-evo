# Task DOC-269G-INTERMEDIATE-COMPACT: Given Intermediate Completion Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-269G-INTERMEDIATE-COMPACT.md](../ja/DOC-269G-INTERMEDIATE-COMPACT.md).

This derived documentation-maintenance contract freezes one coherent legacy
family before deletion. It cannot introduce or reinterpret language behavior,
test intent, API, diagnostics, traceability, or coverage credit.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-269G-INTERMEDIATE-COMPACT` |
| Status | Redirect migration, schema-v1 ledger expansion, independent reviews, full verification, and final quality review complete with all nine gates passing at 100/100; exact staging and commit remain. |
| Purpose | Centralize completion-only H3 records for the contiguous GUPT → GU → GCP → GC dependency chain while preserving every frozen H2 product owner. |
| Owners | Repository migration policy, the four paired historical contracts, [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index), and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Consumers | 36 checker-first EN/JA design documents, `mizar-test` consumer documents, four Task Indexes, and the post-migration schema-v1 ledger/lint |
| Dependencies | GUPT `c5292451`; GU `998dc104`; GCP `59eb7de6`; GC `8181ae8f`; manifest consumer `0ec5fce2`; prior compaction `34b42908` |
| Readiness | Documentation prerequisite commit `cb03a208`; fresh clean selection inventory `origin/main...HEAD=0/2` with protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`; no blocking authority gap. |

## Authority And Classification

Authority is the user's checker-first consolidation decision,
[`AGENTS.md`](../../../../AGENTS.md), the
[migration policy](../../autonomous_crate_development.md#migration-policy),
the four completed task records, and their current frozen H2/H3 design
owners. No source behavior is normative for this task.

| Class | Decision |
|---|---|
| `design_drift` | 142 completion-only H3 sections repeat overlapping historical status, measurements, exclusions, and review evidence across 36 paths. Their bytes are unique, so exact preimages and fact equivalence must be frozen before removal. |
| `test_gap` | None. Schema v1 accepts one complete-section redirect per `(path, task)`; the current generic 15-test consumer needs no Rust or test-count change. |
| `spec_gap` | None for structural migration; no semantic issue is selected. |
| `source_drift` | None; production source is protected. |
| `source_undocumented_behavior` | None introduced or inferred. |
| `test_expectation_drift` | None; specification, `.miz`, fixtures, sidecars, expectations, trace, and metadata are protected. |
| `boundary_violation` | Avoided by moving completion-only H3 evidence to historical contracts while retaining all frozen H2 module, audit, runner, trace, sequencing, and deferral owners. |
| `repo_metadata_conflict` | At implementation selection, `origin/main...HEAD` was `0/2` after the prior compaction and this task's prerequisite commits. The reflog later recorded an external `update by push` aligning both refs at `cb03a208` (`0/0`). Both observations are report-only; no repair or agent push is authorized. |

## Frozen Preimage Inventory

The language-neutral
[`DOC-269G-INTERMEDIATE-COMPACT.sources.tsv`](../DOC-269G-INTERMEDIATE-COMPACT.sources.tsv)
contains exactly 142 byte-sorted data rows plus two comments and final LF. Each
data row records task, language, component, exact path, ATX level, exact
heading text without the prefix, section SHA-256, and physical lines. The raw
heading is reconstructed as three `#` bytes, one space, and the heading text;
the section ends immediately before the next visible ATX heading at H3 or
higher. Explicit replay against clean HEAD must match every row before
migration.

The task-local preimage data-row SHA-256, excluding the two comments, is
`038f79e147a1d3f04d20edc1ca1493974f151ef6aa757d29070216b41ce5bd2c`.
The 144-line physical TSV SHA-256 is
`a6e539beb0d04137fdb0d90d011553eff86d9a655e86d63769ad0642a2d1eb55`.
These are review evidence, not the future manifest expanded-inventory hash and
not yet lint-enforced.

| Task | Sections | Physical lines | Distinct paths | Historical owner |
|---|---:|---:|---:|---|
| 269GUPT | 34 | 305 | 34 | [contract](./269GUPT.md#completion-evidence) |
| 269GU | 36 | 303 | 36 | [contract](./269GU.md#completion-evidence) |
| 269GCP | 36 | 313 | 36 | [contract](./269GCP.md#completion-evidence) |
| 269GC | 36 | 437 | 36 | [contract](./269GC.md#completion-evidence) |
| **Total** | **142** | **1,358** | **36 union paths** |  |

The exact EN/JA-symmetric path/task matrix is:

| Component | Relative file | Selected tasks per language |
|---|---|---|
| mizar-checker | `00.crate_plan.md` | GUPT, GU, GCP, GC |
| mizar-checker | `bilingual_sync_audit.md` | GUPT, GU, GCP, GC |
| mizar-checker | `binding_env.md` | GUPT, GU, GCP, GC |
| mizar-checker | `module_boundary_audit.md` | GUPT, GU, GCP, GC |
| mizar-checker | `payload_family_decomposition.md` | GUPT, GU, GCP, GC |
| mizar-checker | `resolved_typed_ast.md` | GUPT, GU, GCP, GC |
| mizar-checker | `semantic_spec_audit.md` | GUPT, GU, GCP, GC |
| mizar-checker | `source_proof_local_declaration.md` | GUPT, GU, GCP, GC |
| mizar-checker | `source_spec_audit.md` | GUPT, GU, GCP, GC |
| mizar-checker | `source_statement.md` | GUPT, GU, GCP, GC |
| mizar-checker | `source_term.md` | GU, GCP, GC |
| mizar-checker | `source_type.md` | GUPT, GU, GCP, GC |
| mizar-checker | `typed_ast.md` | GUPT, GU, GCP, GC |
| mizar-test | `00.crate_plan.md` | GUPT, GU, GCP, GC |
| mizar-test | `bilingual_sync_audit.md` | GUPT, GU, GCP, GC |
| mizar-test | `harness.md` | GUPT, GU, GCP, GC |
| mizar-test | `module_boundary_audit.md` | GUPT, GU, GCP, GC |
| mizar-test | `traceability.md` | GUPT, GU, GCP, GC |

All selected headings occur only in the frozen rows. GCP's EN
`Task 269GCP Condition-profile Decomposition` and JA
`Task 269GCP condition-profile decomposition` H3 sections are excluded. They
are durable payload owners, and omitting them also preserves schema v1's one
redirect per `(path, task)` identity. The selected GCP payload completion H3
uses the corresponding retained, language-exact decomposition H3 as its actual
preceding same-level anchor; migration must not invent a GCP H2.

## Documentation-Prerequisite Scope

The prerequisite changes exactly 15 paths: this EN/JA pair; four paired
historical contracts; the language-neutral source TSV; and the checker/test
EN/JA crate plans. Each plan receives five language-local Task Index rows—four
historical contracts plus this batch—for 20 rows total.

It does not change any of the 36 migration sources, the existing
`legacy_compactions.tsv`, Rust, Cargo, specification, `.miz`, fixture, sidecar,
expectation, trace, metadata, root audit, count/hash/status, or executable
behavior. `doc/design/spec_coverage_audit.md` remains unchanged because no
coverage, design mapping, follow-up owner, or deferral status changes.

## Frozen Migration And Ownership Boundary

After the prerequisite commit and fresh replay, implementation replaces each
listed complete H3 section with one language-local redirect to its task
contract's `#completion-evidence`. It changes only the 36 source paths, this
EN/JA status/evidence pair, and `legacy_compactions.tsv`: 39 paths total. The
ledger adds one batch, four tasks, 142 redirects, and 20 index records with a
new independently computed manifest inventory hash. The source TSV and
historical contracts remain immutable.

The historical contracts own consolidated completion measurements and review
evidence only. Module documents retain durable public/private API,
fingerprints, validation, binding/lookup, payload, Typed/Resolved, runner,
audit, trace, bilingual, sequencing, and deferral contracts in their frozen
H2 sections. The migration neither deletes nor rewrites those H2 sections.

The given-scope statement is preservation-only: a binding covers its own
`such that`, the remainder of the innermost proof/reasoning block, and
unshadowed descendants, but not parent, sibling, or post-exit sites. No goal,
guard, fact, equality, condition meaning, proof, discharge, acceptance,
initial obligation, capture/export, IR, VC, ATP, or Task-270 behavior may be
invented. GUPT, GU, GCP, and GC remain distinct; GCT/GCU ownership is
unchanged.

## Tests, Reviews, And Exit

Prerequisite review must independently verify policy/scope, all 142 preimages
and preserved facts, durable-owner exclusions, EN/JA equivalence and links.
All reviews must end **NO FINDINGS**. Verification includes explicit TSV
replay, path/task/count/hash checks, GCP exclusion, recursive task-contract
pair/link/fragment lint, full lint policy, checker/runner libraries, metadata,
checker lint, formatting, Cargo metadata, warnings-denied workspace Clippy,
full tests, all five CLIs and protected hashes, protected-scope inspection,
`git diff --check`, exact 15-path staging, and all nine gates with uncapped
score `>=90/100`.

After one docs-only prerequisite commit, fresh inventory must reselect this
same batch before implementation. The implementation receives separate test-
sufficiency, equivalence/boundary, source/document/EN-JA, and final-quality
reviews, exact 39-path staging, and one separate commit. The agent does not
push. Parent reasoning remains `xhigh`; bounded mechanically frozen reviews
may use `high`.

## Documentation-Prerequisite Evidence

Independent policy/equivalence, test-sufficiency, and implementation-boundary
reviews ended **NO FINDINGS** after correcting the frozen module-size mapping,
the language-exact GCP exclusion/anchor, and Japanese GU fragments. Parent and
independent replays matched all 142 TSV rows, all 36 paths, all section hashes
and physical-line counts, the 71/71 EN/JA split, the four task counts, and the
two retained GCP decomposition sections. The four Task Indexes contain exactly
20 new language-local records.

The focused recursive task-contract lint and the complete 15-test lint policy
passed. `mizar-checker` library tests passed 530/530, `mizar-test` library tests
passed 600/600, runner metadata tests passed 137/137, and checker lint passed
15/15. `cargo fmt --all --check`, Cargo metadata, warnings-denied workspace
Clippy, and the complete workspace test suite passed. All five target CLIs
exited successfully and preserved these output hashes: plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.
The trace hash remains
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.
Protected specification, tests, traceability, production, Cargo, root audit,
migration sources, and manifest paths are unchanged. The final read-only
quality review reported **NO FINDINGS**, all nine hard gates **PASS**, and an
uncapped score of **100/100**. Only the cached 15-path audit remains before the
prerequisite commit.

## Implementation Evidence

Fresh post-prerequisite replay matched all 142 frozen preimages before editing.
The mechanical migration now changes exactly the 36 declared source documents,
this EN/JA status/evidence pair, and `legacy_compactions.tsv`: 39 paths. It
removes 1,216 completion-section lines, adds 142 standard language-local
redirect lines, and leaves the two durable GCP decomposition H3 owners as the
only matching historical headings. No frozen H2 product owner was changed.

The ledger adds exactly one batch, four task records, 142 redirects over 36
distinct source paths, and 20 index records. Its declared expanded-inventory
SHA-256 is
`d934963a0043aa5a6b7c4b04bbc86ee27875484c6a2d58cff040fcb493c8b3b3`;
the complete physical ledger SHA-256 is
`f18988333588664aab1e9bb1c92382100f2b240ce04fb59229c09cea19a83283`.
The unchanged generic schema-v1 lint consumer accepts the complete migration.

Specification, `.miz`, fixture, sidecar, expectation, trace TOML/status/
backlinks, coverage credit, active outcomes, production, Cargo, public API,
diagnostics, root coverage audit, source inventory TSV, and historical
contracts are unchanged. The paired traceability design documents change only
by redirecting their selected completion evidence. The protected trace hash
remains
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.

Independent test-sufficiency and equivalence/boundary reviews reported **NO
FINDINGS**. Source/document/EN-JA review reported two low wording findings;
after the origin-distance and traceability-scope wording was corrected, its
finding-specific re-review reported **NO FINDINGS**. The complete 15-test
`mizar-test` lint policy, checker 530-test and runner 600-test libraries,
runner metadata 137/137, and checker lint 15/15 pass. `cargo fmt --all
--check`, Cargo metadata, warnings-denied workspace Clippy, full `cargo test`,
all five target CLIs with the prerequisite hashes above, protected count/hash
checks, and `git diff --check` pass. The final read-only quality review's one low
`repo_metadata_conflict` wording finding was corrected; finding-specific
re-review reported **NO FINDINGS**, all nine hard gates **PASS**, no score cap,
and **100/100**. Only exact staging remains before commit.
