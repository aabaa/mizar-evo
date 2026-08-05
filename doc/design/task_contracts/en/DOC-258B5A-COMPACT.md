# Task DOC-258B5A-COMPACT: Task-258B5A Review-Evidence Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-258B5A-COMPACT.md](../ja/DOC-258B5A-COMPACT.md).

This documentation-maintenance contract freezes one completed checker-first
review family before exact whole-section migration. It cannot change language
behavior, test intent, API, diagnostics, traceability, or coverage.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B5A-COMPACT` |
| Status | Documentation prerequisite committed; exact migration, independent reviews, full verification, and final quality are complete. Exact staging and commit remain. |
| Purpose | Centralize repeated Task-258B5A documentation-review, verification, authority, boundary, and bilingual evidence while preserving every durable and later owner. |
| Owners | Migration policy, historical [258B5A](./258B5A.md#completion-evidence), [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index), and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Consumers | Fourteen EN/JA checker/runner source paths, four Task Indexes, and the post-migration generic schema-v1 ledger/lint |
| Historical sequence | `50ab1ebc` -> `59021f76` -> `4a79116c` -> `141dc44a` -> `46dd9db5` -> `f27d2c91` |
| Documentation prerequisite | `153dd93b3304be6c5bea0a8861fa5940abf1913c` |
| Readiness | Clean selection HEAD `f77f68f9b0bd48c681396afb4125cba343a294a8`, `origin/main...HEAD=0/4`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`; exact selection is dependency-ready. |

## Authority And Classification

Authority is the user-approved checker-first compaction program,
[`AGENTS.md`](../../../../AGENTS.md), the
[migration policy](../../autonomous_crate_development.md#migration-policy),
canonical authority and tests linked by the retained Task-258B5A owners, and
the completed reviewed history. Source behavior is not normative here.

| Class | Decision |
|---|---|
| `design_drift` | Fourteen H3s repeat one historical review checkpoint across fourteen paths; the paired historical contract becomes their owner. |
| `spec_gap` / `test_gap` | None for this structural migration. Historical B5B/B5C `test_gap` ownership remains unchanged. |
| `source_drift` / `source_undocumented_behavior` | None introduced. The historical next-task-owned B5A `source_drift` is preserved as time-local evidence and was closed by `4a79116c`. |
| `test_expectation_drift` | None; protected test-intent artifacts are unchanged. |
| `boundary_violation` | Avoided by retaining every H2, implementation section, owner-local section, all eight source-local final-quality H3s, both root coverage-review H3s, and every unlisted section. |
| `repo_metadata_conflict` | Historical remote-ref movement remains report-only and human-owned. Current `0/4` distance is measured, not repaired; no fetch/reset/push is authorized. |

## Frozen Preimage And Anchors

[`DOC-258B5A-COMPACT.sources.tsv`](../DOC-258B5A-COMPACT.sources.tsv) contains
14 byte-sorted data rows plus two comments and final LF. Data-row SHA-256 is
`00f8311a0620475f366919bf24820b17b79b41b180c2cff2a57abf131482ac3f`;
the complete 16-line TSV SHA-256 is
`ffd6e9161804d82baaf89c2a843db5e19a9e48c34faa24ecd4a4513d02ac51bc`.

The selection is 14 unique `(path, task)` H3s over 14 paths, 133 physical
lines: EN/JA `7/68` and `7/65`, checker/runner `8/84` and `6/49`. No selected
section contains a nested heading, table, or fence, and no ledger identity
collides.

| Source | Retained preceding / following same-or-higher heading |
|---|---|
| checker EN plan | `### Tests, deferrals, audit impact, and exit` / `### Task 258B5A Documentation Final Quality` |
| checker EN bilingual | `## Task 258B5A Frozen-Contract Synchronization` / `### Task 258B5A Final-Quality Synchronization` |
| checker EN boundary | `## Task 258B5A Frozen Consumer Boundary` / `## Task 258B5A Implemented Consumer Boundary` |
| checker EN authority | `## Task 258B5A Frozen Authority Audit` / `## Task 258B5A Implementation Authority Result` |
| runner EN plan | `## Checker Task 258B5A Frozen Runner Contract` / `### Checker Task 258B5A Documentation Final Quality` |
| runner EN bilingual | `## Checker Task 258B5A Frozen-Contract Synchronization` / `### Checker Task 258B5A Final-Quality Synchronization` |
| runner EN boundary | `## Checker Task 258B5A Frozen Runner Boundary` / `## Checker Task 258B5A Implemented Runner Boundary` |
| checker JA plan | `### Tests、deferrals、audit impact、exit` / `### Task 258B5A documentation final quality` |
| checker JA bilingual | `## Task 258B5A frozen-contract synchronization` / `### Task 258B5A final-quality synchronization` |
| checker JA boundary | `## Task 258B5A frozen consumer boundary` / `## Task 258B5A implemented consumer boundary` |
| checker JA authority | `## Task 258B5A frozen authority audit` / `## Task 258B5A implementation authority result` |
| runner JA plan | `## Checker Task 258B5A frozen runner contract` / `### Checker Task 258B5A documentation final quality` |
| runner JA bilingual | `## Checker Task 258B5A frozen-contract synchronization` / `### Checker Task 258B5A final-quality synchronization` |
| runner JA boundary | `## Checker Task 258B5A frozen runner boundary` / `## Checker Task 258B5A implemented runner boundary` |

The prerequisite changes exactly nine paths: this EN/JA pair, historical
EN/JA pair, source TSV, and four plans. Each plan receives `258B5A` and the
batch Task Index row, eight rows total. It changes no selected preimage,
ledger, specification, `.miz`, fixture, sidecar, expectation, trace, coverage
audit, Rust/Cargo, public API, diagnostics, count/hash/status, or behavior.

After its separate prerequisite commit and fresh replay, migration may replace
only the 14 complete H3s with language-local redirects to
`258B5A.md#completion-evidence`. It changes exactly the 14 sources, this pair,
and `legacy_compactions.tsv`: 17 paths. Ledger impact is one batch, one task,
14 redirects over 14 distinct paths, eight index records, and one expanded-
inventory hash. The source TSV and historical contracts become immutable.

Every H2, implementation section, TODO/trace owner, component API/invariant/
boundary owner, and unlisted section remains. In particular, the eight final-
quality H3s inside the migration sources and both root coverage-audit H3s at
`#task-258b5a-documentation-review-evidence` and
`#task-258b5a-documentation-final-quality` are protected. The latter makes
nine repository-wide final-quality H3s. The root coverage audit stays
unchanged because design mapping, trace status, coverage credit, and semantic
ownership do not change.

## Reviews, Verification, And Exit

Prerequisite and migration each require independent specification/contract,
test-sufficiency, equivalence/boundary, source/document/EN-JA, and final-quality
reviews as applicable, ending **NO FINDINGS**. Verification includes exact
preimage/hash/count/anchor replay, recursive contract/link/fragment and generic
ledger lint, full lint policies, checker/runner/metadata tests, formatting,
Cargo metadata, warnings-denied Clippy, workspace tests, all five CLIs,
protected hashes, `git diff --check`, exact staging, all nine hard gates, and
an uncapped score `>=90/100`.

The migration must preserve the `59021f76` checkpoint chronology, later
`4a79116c` implementation, B5A/B5B/B5C split, eight classifications, all
review/verification facts, and every protected owner. No push or stash
mutation is authorized.

### Documentation-Prerequisite Evidence

Finding-specific contract review, independent test-sufficiency review, and
independent equivalence/ownership/EN-JA review ended **NO FINDINGS**. They
replayed all 14 preimages and anchors, corrected and confirmed the eight
source-local versus nine repository-wide final-quality count, and verified
chronology, ownership, classifications, the audit no-impact decision, exact
nine-path scope, eight indexes, both TSV hashes, and paired links.

Focused recursive contract/link/fragment lint and full checker/runner lint
policies passed `1/1`, `15/15`, and `15/15`. Checker/runner libraries passed
`530/530` and `600/600`; runner metadata passed `137/137`. `cargo fmt --all
--check`, Cargo metadata, warnings-denied all-target/all-feature Clippy, full
workspace tests, and `git diff --check` passed. All five CLIs exited zero with
23 unchanged warnings and these stdout hashes:

| CLI | SHA-256 |
|---|---|
| plan | `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718` |
| parse-only | `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56` |
| declaration-symbol | `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74` |
| type-elaboration | `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f` |
| proof-verification | `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450` |

Protected inventory measurements are:

| Surface | Paths | Path SHA-256 | Content SHA-256 |
|---|---:|---|---|
| spec | 64 | `d900ba9e43ab094925f36493f830e0c6a2964be2911d5d229014a58842067a25` | `b30dd5519191a4407399826faf91cc58853d7944df7630734b5ab05de48c9f7b` |
| `.miz` | 343 | `d94980e167b4b8ac196f91e7694ff044080c6fb4d04c135b3cd5e65b9a019f22` | `54e6ea1254a0bd963c39026d788711507f353e9c6df3b4f9fd268b2e9f240afb` |
| expectation | 435 | `22a5ee257a294e3f2ed4b24bf9ca243d037bbd798f009d0d1e3a176dad8b4fea` | `b5f0ed1a8d73bbfb78af5cad87a8d426e97269f7f70163750893a9fe1f39d2ea` |
| checker production | 30 | `a41370d7150a587369cea5f7a67b60417dd1372592f55c0d65bec369eb39fdc6` | `05fd5e0eaed4361b824693941e9056a552c476f050915ea5052a85c8c7174dfd` |
| runner production | 90 | `05245a54160dfce17336b476b07885eb6d5afe138c4780a6a6a7b47043e7248c` | `210f294aebfe22c12324ef9919ac68147f8025f0da8de166403dada87bac5eae` |
| Cargo | 21 | `d93f2816b760d8ba45430d6d8570e598864aa7b20b19a001f45171d36fd3a030` | `146e9b4024d2c344b2eca9c6f5ca6d6a80a3de427e382953a2280bc63cb3ecca` |

The protected trace manifest remains
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.
Final read-only quality review ended **NO FINDINGS**. All nine hard gates pass,
no score cap applies, and the valid score is `100/100`
(`20/20/15/15/10/10/5/5`). At that prerequisite checkpoint, only exact
task-only staging and its dedicated commit remained; migration was still a
separate future change.

## Migration Evidence

The prerequisite is commit `153dd93b3304be6c5bea0a8861fa5940abf1913c`.
Its post-commit inventory was clean at `origin/main...HEAD=0/5`, the protected
stash was unchanged, and all 14 frozen preimages replayed before editing.

The mechanical migration changes exactly the 14 declared sources, this EN/JA
pair, and `legacy_compactions.tsv`: 17 paths. It replaces only the 14 complete
H3 sections with language-local redirects. Their 133 physical lines become 28
redirect-plus-separator lines, a reduction of 105 lines. Every H2,
implementation section, all eight source-local final-quality H3s, both root
coverage-audit H3s, and every unlisted owner remain.

The ledger now has 559 physical lines and adds exactly one batch, one task, 14
redirects over 14 distinct source paths, and eight index records. Its expanded-
inventory SHA-256 is
`7484411f88cb4009b4ad6ea0cd9bd0e1d99e1e92fe4e0bf2bc9c578369510e34`;
its complete physical SHA-256 is
`55ecba46e9847d2bfcea17c6f7df64ca4f6248d689654c820ffccb3a3b396dae`.
The immutable source TSV remains
`ffd6e9161804d82baaf89c2a843db5e19a9e48c34faa24ecd4a4513d02ac51bc`.

Focused generic-ledger/link/fragment lint and `git diff --check` pass.
Specification, `.miz`, fixture, sidecar, expectation, trace status/backlinks,
coverage credit, source, Cargo, public API, diagnostics, root coverage audit,
historical contracts, source TSV, and the four Task Indexes are unchanged.

Independent test-sufficiency, equivalence/boundary, and source/document/EN-JA
consistency reviews ended **NO FINDINGS**. They replayed the committed
preimages, every live fact owner and retained section, exact redirects/anchors,
ledger ordering/arithmetic/hashes, chronology, classifications, protected
scope, bilingual parity, and audit no-impact. Generic schema-v1 lint is
sufficient; a batch-specific Rust or semantic test is neither required nor
allowed by the data-driven policy.

Focused/full runner lint policy passed `1/1` and `15/15`; checker lint passed
`15/15`; checker/runner libraries passed `530/530` and `600/600`; runner
metadata passed `137/137`. Formatting, Cargo metadata, warnings-denied Clippy,
the full workspace suite, protected count/hash replay, and `git diff --check`
passed. All five CLIs exited zero with 23 unchanged warnings and the same
prerequisite hashes recorded above.
Final read-only quality review ended **NO FINDINGS**. All nine hard gates pass,
no score cap applies, and the valid score is `100/100`
(`20/20/15/15/10/10/5/5`), with no residual risk inside migration scope.

## Handoff

After both task-only commits and clean post-commit inventory, select exactly
one dependency-ready checker-owned duplication family from fresh read-only
inventory. Keep the parent at `xhigh`, use `high` for bounded independent
reviews, and `medium` only for deterministic non-semantic inventory.
