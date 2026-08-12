# Task DOC-258AB-COMPACT: Source-Statement Completion Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-258AB-COMPACT.md](../ja/DOC-258AB-COMPACT.md).

This derived documentation-maintenance contract freezes one coherent family
of completed checker-first tasks before deletion. It cannot introduce or
reinterpret language behavior, test intent, API, diagnostics, traceability,
or coverage credit.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258AB-COMPACT` |
| Status | Complete. The migration is registered in the schema-2 ledger; task-local completion evidence below preserves the committed migration and clean replay. |
| Purpose | Centralize completion-only evidence for Tasks 258A, 258B1, and 258B2 while retaining every frozen contract, owner-local invariant, semantic deferral, and verification owner. |
| Owners | Repository migration policy, historical [258A](./258A.md#completion-evidence), [258B1](./258B1.md#completion-evidence), and [258B2](./258B2.md#completion-evidence) contracts, plus the [checker](../../mizar-checker/en/00.crate_plan.md#task-index) and [runner](../../mizar-test/en/00.crate_plan.md#task-index) Task Indexes |
| Consumers | 18 checker/runner EN/JA design paths, four Task Indexes, and the post-migration generic schema-v1 ledger/lint |
| Historical commits | 258A prerequisite/implementation `e0b4bb59`/`1e81db7a`; 258B1 `ddcac673`/`e87b4a48`; 258B2 `3dd38526`/`4d9ed4f5` |
| Documentation prerequisite | `d767941aad8f0339af76500c3801823675f2b139` |
| Readiness | Clean post-prerequisite HEAD `d767941a`, `origin/main...HEAD=0/1`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`; all 38 preimages replayed before editing and no blocking authority gap exists. |

Task order is historical selection order: 258A, then the pair-only 258B1
slice, then the base-only 258B2 sibling. It is not a semantic dependency
chain. B2 depends on the shared 258A/base owner and has no B1 reference edge.

## Authority And Classification

Authority is the user's checker-first consolidation decision,
[`AGENTS.md`](../../../../AGENTS.md), the
[migration policy](../../autonomous_crate_development.md#migration-policy),
canonical spec chapters named in the retained task plans, and the completed
derived records. Source behavior is not normative for this task.

| Class | Decision |
|---|---|
| `design_drift` | 38 completion-only H3 sections repeat implementation status, measurements, exclusions, and review evidence across 18 paths. The paired historical contracts become their single owner. |
| `test_gap` | None. Schema v1 supports exact whole-section redirects, and the generic lint consumer covers this shape. |
| `spec_gap` | None for this structural migration; no language-semantic issue is selected. |
| `source_drift` | None; Rust and Cargo are protected. |
| `source_undocumented_behavior` | None introduced or inferred. |
| `test_expectation_drift` | None; specs, `.miz`, fixtures, sidecars, expectations, trace TOML, and metadata are protected. |
| `boundary_violation` | Avoided by preserving every H2, every unlisted H3, and especially all twelve EN/JA Task-258B2 frozen H3 owner sections. |
| `repo_metadata_conflict` | Report-only: after the VS Code restart, `origin/main` changed from the previously observed three-commit distance to equal clean HEAD `a1bf34e8` without an authorized push in this workflow. No repair, fetch, reset, or push is attempted; exact task files remain safely identifiable. |

## Frozen Preimage Inventory

[`DOC-258AB-COMPACT.sources.tsv`](../DOC-258AB-COMPACT.sources.tsv) contains
exactly 38 byte-sorted data rows plus two comments and final LF. Each row
records task, language, component, exact path, ATX level, exact heading,
complete-section SHA-256, and physical lines. A section begins at its H3 and
ends immediately before the next visible H3-or-higher ATX heading.

The data-row SHA-256 is
`f51fbef7c54f41065409b53eb8a5485b0d6ff6f67c42eb63cd4755909ad5c87d`;
the complete 40-line TSV SHA-256 is
`65fda187d1f5e0e5202269918c78cf3a74f7eda451d0d70fab9c7d9f3a2db119`.
The selection totals are 38 unique `(path, task)` sections over 18 physical
paths / nine paired relative files, 502 physical lines, EN/JA `19/19`, and
checker/runner `28/10`. Per task: 258A `10/155`, 258B1 `16/229`, and 258B2
`12/118` sections/lines. No selected section contains a nested ATX heading,
table, or fence, and no selected heading is already forbidden by the ledger.

| Component | Relative files |
|---|---|
| mizar-checker | `00.crate_plan.md`, `binding_env.md`, `payload_family_decomposition.md`, `resolved_typed_ast.md`, `source_spec_audit.md`, `source_statement.md`, `typed_ast.md` |
| mizar-test | `00.crate_plan.md`, `harness.md` |

The Task-258B2 checker-plan and source-statement implementation results are
H2 sections and remain. The six paired B2 frozen H3 owners in binding,
payload, Typed, Resolved, runner plan, and harness remain. Every other H2,
frozen section, unselected H3, adjacent owner-local fact, and later task is
also outside the TSV.

## Documentation-Prerequisite Scope

The prerequisite changes exactly 13 paths: this EN/JA batch pair, three
paired historical contracts, the language-neutral TSV, and four checker/test
EN/JA plans. Each plan receives four Task Index rows, 16 records total.

It does not edit any selected preimage, the legacy ledger, production,
Cargo, specification, `.miz`, fixture, sidecar, expectation, trace TOML,
metadata, root coverage audit, executable count/hash/status, or behavior.
`doc/design/spec_coverage_audit.md` remains unchanged because spec coverage,
design mapping, trace status, coverage credit, and semantic ownership do not
change.

## Frozen Migration And Ownership Boundary

After a dedicated prerequisite commit and fresh replay, implementation may
replace only the 38 inventory-listed complete H3 sections with language-local
redirects to the corresponding historical contract's `#completion-evidence`.
It may change exactly the 18 sources, this EN/JA batch pair, and
`legacy_compactions.tsv`: 21 paths. The ledger impact is one batch, three task
records, 38 redirects over 18 distinct source paths, 16 index records, and one
newly computed expanded-inventory hash. The TSV and historical contracts become
immutable during migration.

Historical contracts own completion measurements and review evidence.
Component documents retain module APIs, ownership, validation, invariants,
runner boundaries, and frozen plans. Migration cannot add assumptions as
accepted facts, goal/guard composition, proof/discharge/acceptance, theorem
publication, diagnostics, Core/CFG/VC/ATP state, active dispatch, or coverage
credit. Tasks 258B3/B4/B5 and 269–272 retain their stated semantics and
follow-up ownership.

## Documentation-Prerequisite Evidence

The pre-edit specification review found two medium design-drift risks:
chronological order could be mistaken for a semantic dependency chain, and
the six paired B2 frozen H3 owners needed explicit links. The drafted
contracts resolved both. Contract review then found one medium schema-v1
wording mismatch that incorrectly implied source records; after changing it
to 38 redirects over 18 distinct source paths, finding-specific re-review
ended **NO FINDINGS**. Independent test-sufficiency and equivalence/EN-JA/
ownership reviews also ended **NO FINDINGS**.

Parent replay matched all 38 preimages and 502 physical lines, both TSV
hashes, the `10/155`, `16/229`, and `12/118` task partitions, EN/JA `19/19`,
checker/runner `28/10`, 18 distinct paths, 13-path prerequisite scope, and 16
index records. Recursive task-contract/link/fragment lint and all 15 runner
lint-policy tests passed. Checker and runner libraries passed `530/530` and
`600/600`; runner metadata passed `137/137`; checker lint passed `15/15`.
`cargo fmt --all --check`, Cargo metadata, warnings-denied all-target/
all-feature Clippy, the full workspace test suite, and `git diff --check`
passed. The protected trace SHA-256 remains
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.

The five CLI stdout hashes remain:

| Route | SHA-256 |
|---|---|
| plan | `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718` |
| parse-only | `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56` |
| declaration-symbol | `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74` |
| type-elaboration | `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f` |
| proof-verification | `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450` |

All 13 changed paths are documentation-only. Production, Cargo,
specification, test artifacts, trace status/count/backlinks, coverage audit,
legacy ledger, and executable behavior remain unchanged. The existing plan
warnings are baseline warnings and did not alter stdout hashes.

The final independent read-only quality review ended **NO FINDINGS**, passed
all nine hard gates, applied no score cap, and assigned **100/100**. The only
residual observation is the report-only remote-ref metadata conflict; it does
not obstruct exact staging.

## Implementation Evidence

Fresh post-prerequisite inventory replayed all 38 frozen preimages, headings,
hashes, physical-line counts, language/component partitions, and neighboring
anchors before editing. The migration replaces only those 38 complete H3
sections with 38 language-local redirects to the corresponding historical
contract. The selected intervals occupied 502 physical lines, including 38
separator blanks. Replacement preserves those blanks, removes 464 completion-
content lines, adds 38 redirect lines, and reduces the mapped intervals by 426
lines. Every H2, all unlisted H3s, and all six paired B2 frozen H3 owners remain.

The ledger has 508 physical lines and adds exactly one batch, three task
records, 38 redirects over 18 distinct source paths, and 16 index records. Its
expanded-inventory SHA-256 is
`c472137844a8f41c6e3ad7ab96b8a8de559df962979b148c2dc706b1de6acbd8`;
its complete physical SHA-256 is
`4d6dd6103ee721e72b2c008247eeb84fcd30a7023e38cedbe8b73571ed621dd0`.
The immutable 40-line source TSV remains
`65fda187d1f5e0e5202269918c78cf3a74f7eda451d0d70fab9c7d9f3a2db119`.

Specification, `.miz`, fixtures, sidecars, expectations, trace TOML/status/
backlinks, coverage credit, source, Cargo, public API, diagnostics, root
coverage audit, historical contracts, and the four prerequisite Task Indexes
are unchanged. The protected trace hash remains
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.

Independent test-sufficiency, equivalence/boundary, and source/document/EN-JA
consistency reviews each ended **NO FINDINGS**. Focused and full runner lint
policy passed `1/1` and `15/15`; checker and runner libraries passed `530/530`
and `600/600`; runner metadata passed `137/137`; checker lint passed `15/15`.
`cargo fmt --all --check`, Cargo metadata, warnings-denied all-target/
all-feature Clippy, the full workspace test suite, all five CLIs with the
prerequisite hashes above, protected count/hash replay, and `git diff --check`
passed.

The final independent read-only quality review ended **NO FINDINGS**, passed
all nine hard gates, applied no score cap, and assigned **100/100** with no
residual risk inside migration scope. The historical remote-ref movement
remains a report-only, human-owned observation.

## Tests, Reviews, And Exit

Prerequisite review must independently reproduce preimages, hashes, counts,
fact ownership, sequencing/dependency wording, all retained H2/H3 exclusions,
EN/JA equivalence, indexes, and links. Test-sufficiency and source/document
consistency reviews must end **NO FINDINGS**. Verification includes preimage
replay, recursive task-contract/link/fragment lint, full lint policy,
checker/runner libraries, metadata, checker lint, formatting, Cargo metadata,
warnings-denied all-target/all-feature Clippy, full workspace tests, all five
CLIs and protected hashes, `git diff --check`, exact 13-path staging, and all
nine hard gates with uncapped score `>=90/100`.

After the prerequisite commit, fresh inventory must select the same batch.
Migration then receives separate test-sufficiency, equivalence/boundary,
source/document/EN-JA, and final-quality reviews, exact 21-path staging, and a
separate commit. The agent does not push or modify the protected stash.
