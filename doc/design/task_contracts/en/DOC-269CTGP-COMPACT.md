# Task DOC-269CTGP-COMPACT: Proof-Local Lower Completion Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-269CTGP-COMPACT.md](../ja/DOC-269CTGP-COMPACT.md).

This derived documentation-maintenance contract freezes one completed
checker-first sequence before exact whole-section migration. It cannot change
language behavior, test intent, API, diagnostics, traceability, or coverage.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-269CTGP-COMPACT` |
| Status | Documentation prerequisite ready for its dedicated commit; independent reviews, verification, and final quality gate are complete. |
| Purpose | Centralize completion evidence for 269CT and 269GP while retaining every prerequisite, durable owner, later authority, and semantic deferral. |
| Owners | Migration policy, historical [269CT](./269CT.md#completion-evidence) and [269GP](./269GP.md#completion-evidence) records, [checker plan](../../mizar-checker/en/00.crate_plan.md#task-index), and [runner plan](../../mizar-test/en/00.crate_plan.md#task-index) |
| Consumers | Eight EN/JA checker/runner design paths, four Task Indexes, and the post-migration generic schema-v1 ledger/lint |
| Historical commits | 269CT prerequisite/implementation `b1c91b1b`/`c6036197`; 269GP `97a75fd9`/`adea7f0e` |
| Readiness | Clean selection HEAD `5a83db6f82aa789e31b00601e66d57fe4cda2601`, `origin/main...HEAD=0/2`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`; all preimages replay and no blocking authority gap. |

Post-269CT inventory selected 269GP next, establishing a coherent historical
sequence, not a semantic dependency. GP is runner-private syntax-only and does
not consume CT's checker/type composite. Its completion-time scope blocker is
historical; later 269GS resolved it without changing these completion records.

## Authority And Classification

Authority is the user-approved checker-first compaction program,
[`AGENTS.md`](../../../../AGENTS.md), the
[migration policy](../../autonomous_crate_development.md#migration-policy),
canonical chapters cited by retained plans, and completed reviewed records.

| Class | Decision |
|---|---|
| `design_drift` | Twelve completion H3s repeat measurements, exclusions, and reviews across eight paths; paired historical contracts become their owner. |
| `spec_gap` | None for structural migration. The historical GP scope conflict was later resolved by 269GS and is neither reinstated nor reinterpreted. |
| `test_gap` | None; the generic schema-v1 lint covers this exact shape. |
| `source_drift` / `source_undocumented_behavior` | None; source is protected and not normative. |
| `test_expectation_drift` | None; spec/test/trace/expectation artifacts are protected. |
| `boundary_violation` | Avoided by preserving every H2, eight prerequisite H3s, later 269GS, and all unselected owners. The historical 269CT review classification remains owned by its task contract. |
| `repo_metadata_conflict` | Historical remote-ref movement remains report-only and human-owned. Current two-commit distance is measured, not repaired; no fetch/reset/push is authorized. |

## Frozen Preimage And Scope

[`DOC-269CTGP-COMPACT.sources.tsv`](../DOC-269CTGP-COMPACT.sources.tsv)
contains 12 byte-sorted data rows plus two comments and final LF. Data-row
SHA-256 is
`3d3423a76a5dbdef0208733ce8a24332d9b39ee46ec15dde11fc89855d526c90`;
the complete 14-line TSV is
`6d32ed76afb190c3669b48359ded7a7d2fdd54018b01e729d37a195b4dd8b0f9`.

The selection is 12 unique `(path, task)` H3s over eight paths / four paired
relative files, 299 physical lines, EN/JA `6/6`, checker/runner `8/4`, and
269CT/269GP `6/184` and `6/115`. No selected section contains a nested heading,
table, or fence, and no heading collides with the ledger.

The prerequisite changes exactly 11 paths: this EN/JA pair, two historical
EN/JA pairs, the TSV, and four plans. Each plan receives 269CT, 269GP, and
batch Task Index rows—12 total. It changes no selected preimage, ledger,
production, Cargo, specification, test, fixture, sidecar, expectation, trace,
metadata, coverage audit, count/hash/status, or behavior.

After a dedicated prerequisite commit and fresh replay, migration may replace
only the 12 complete H3s with language-local redirects. It changes exactly
eight sources, this EN/JA pair, and `legacy_compactions.tsv`: 11 paths. Ledger
impact is one batch, two task records, 12 redirects over eight distinct source
paths, 12 index records, and one expanded-inventory hash. TSV and historical
contracts become immutable. The six CT and two GP prerequisite H3s, every H2,
all unlisted sections, 269GS, and later owners remain.

`doc/design/spec_coverage_audit.md` remains unchanged because specification
coverage, design mapping, trace state, coverage credit, and current semantic
ownership do not change. Migration cannot add Given scope, binding/type,
condition/fact, proof/discharge/acceptance, goal/obligation, IR/VC/ATP,
diagnostics, dispatch, or active coverage.

## Reviews, Verification, And Exit

Prerequisite reviews independently reproduce preimages, historical facts,
sequence wording, GP/269GS history, retained owners, indexes, EN/JA equivalence,
and links. Verification includes replay and hashes, recursive contract/link/
fragment and legacy-ledger lint, full lint policies, checker/runner libraries,
metadata, formatting, Cargo metadata, warnings-denied Clippy, workspace tests,
all five CLIs, protected hashes, `git diff --check`, exact 11-path staging, and
all nine gates with uncapped score `>=90/100`.

After the prerequisite commit, fresh inventory must reselect the same batch.
Migration receives separate test, equivalence/boundary, source/docs/EN-JA, and
final-quality reviews, exact 11-path staging, and a separate commit. No push or
stash mutation is authorized.

### Documentation-Prerequisite Evidence

Independent contract, test-sufficiency, and equivalence/bilingual reviews ended
with **NO FINDINGS**. They reproduced the 12-section / eight-path / 299-line
preimage, both TSV hashes, the exact 11-path scope, all 12 Task Index rows, the
historical commit facts, CT-to-GP chronological wording, the historical GP
blocker and later 269GS resolution, retained owners, exclusions, and EN/JA
equivalence.

Focused recursive contract/link/fragment lint passed `1/1`; full checker and
runner lint policies passed `15/15` each. Checker library tests passed `530/530`,
runner library tests `600/600`, and runner metadata tests `137/137`. `cargo fmt
--all --check`, Cargo metadata, warnings-denied all-target/all-feature Clippy,
the full workspace test suite, protected-surface checks, trace hash replay, and
`git diff --check` passed. All five CLIs exited successfully with 23 unchanged
warnings and these stdout hashes:

| CLI | SHA-256 |
|---|---|
| plan | `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718` |
| parse-only | `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56` |
| declaration-symbol | `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74` |
| type-elaboration | `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f` |
| proof-verification | `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450` |

The protected trace manifest remains
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.
Final finding-specific re-review reports **NO FINDINGS**; all nine hard gates
pass without a score cap at a valid `100/100`.
