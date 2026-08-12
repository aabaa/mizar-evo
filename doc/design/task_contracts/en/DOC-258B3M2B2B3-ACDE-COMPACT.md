# Task DOC-258B3M2B2B3-ACDE-COMPACT: Set-Witness Completion Compaction

> Canonical language: English. Japanese companion:
> [../ja/DOC-258B3M2B2B3-ACDE-COMPACT.md](../ja/DOC-258B3M2B2B3-ACDE-COMPACT.md).

This documentation-maintenance contract freezes one checker-only family of
completed Task-255 set-witness logs before exact whole-section migration. It
cannot introduce or reinterpret language behavior, test intent, API,
diagnostics, traceability, or coverage credit.

## Identity And Status

| Field | Frozen value |
|---|---|
| Task | `DOC-258B3M2B2B3-ACDE-COMPACT` |
| Status | Complete. The migration is registered in the schema-2 ledger; task-local completion evidence below preserves the committed migration and clean replay. |
| Purpose | Centralize the completion-only payload-family evidence for Tasks 258B3M2B2B3A, C, D, and E while retaining every frozen/durable owner and the asymmetric B3B record. |
| Owners | Migration policy; historical [A](./258B3M2B2B3A.md#completion-evidence), [C](./258B3M2B2B3C.md#completion-evidence), [D](./258B3M2B2B3D.md#completion-evidence), and [E](./258B3M2B2B3E.md#completion-evidence) contracts; [checker](../../mizar-checker/en/00.crate_plan.md#task-index) and [runner](../../mizar-test/en/00.crate_plan.md#task-index) indexes |
| Consumers | Paired checker payload-family documents, four Task Indexes, and the post-migration generic schema-v1 ledger/lint |
| Historical sequence | A `f4ff4596` -> `a147bad8`; C `ea48ffc4` -> `7988a509`; D `43af562c` -> `08a7d1e3`; E `8075000b` -> `e4479691` |
| Documentation prerequisite | Committed as `497e60b2ad5ec338cf28d1846663364aeb45f6b6`. |
| Readiness | Clean selection HEAD `95b4ce9801bc0b5ec85dbdba30d40ec26d44d3d7`, `origin/main...HEAD=0/6`, protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`; exact selection is dependency-ready. |

The four tasks are adjacent completed siblings in the same Task-255 set-term
witness family. This grouping preserves separate task identities and facts;
it does not assert a new semantic dependency between enumeration, choice,
`qua`, and comprehension.

## Authority And Classification

Authority is the user's checker-first consolidation direction,
[`AGENTS.md`](../../../../AGENTS.md), the
[migration policy](../../autonomous_crate_development.md#migration-policy),
the retained canonical/test references linked by each task owner, and the
completed reviewed history. Source behavior is not normative for this task.

| Task | Exact canonical/executable authority | Existing test owner |
|---|---|---|
| A | Chapters 13 §§13.4.1/13.9, 4 §4.4.3, 15 §§15.4.4/15.11.5, and 16 §§16.2/16.3.3/16.7.3; [`pass_parser_simple_statements_001.miz`](../../../../tests/miz/pass/parser/pass_parser_simple_statements_001.miz); [`fail_type_elaboration_set_enumeration_formula_gap_001.miz`](../../../../tests/miz/fail/types/fail_type_elaboration_set_enumeration_formula_gap_001.miz), expectation, and trace | Exact checker four and runner five names/matrices in the [checker](../../mizar-checker/en/00.crate_plan.md#task-258b3m2b2b3a-frozen-contract) and [runner](../../mizar-test/en/00.crate_plan.md#checker-task-258b3m2b2b3a-runner-frozen-contract) frozen contracts |
| C | Chapters 13 §13.5, 4 §4.4.3, 15 §15.4.4, and 16 §16.3.3; [`pass_parser_primary_terms_001.miz`](../../../../tests/miz/pass/parser/pass_parser_primary_terms_001.miz); choice member of [`fail_type_elaboration_local_set_choice_qua_term_gap_001.miz`](../../../../tests/miz/fail/types/fail_type_elaboration_local_set_choice_qua_term_gap_001.miz), expectation, and trace | Exact checker four and runner five names/matrices in the [checker](../../mizar-checker/en/00.crate_plan.md#task-258b3m2b2b3c-frozen-choice-witness-contract) and [runner](../../mizar-test/en/00.crate_plan.md#checker-task-258b3m2b2b3c-runner-frozen-contract) frozen contracts |
| D | Chapters 13 §13.6, 4 §4.4.3, 15 §15.4.4, and 16 §16.3.3; [`pass_parser_qua_terms_001.miz`](../../../../tests/miz/pass/parser/pass_parser_qua_terms_001.miz); `equals 4 qua set;` member of the same fail fixture, expectation, and trace | Exact checker four and runner five names/matrices in the [checker](../../mizar-checker/en/00.crate_plan.md#task-258b3m2b2b3d-frozen-qua-witness-contract) and [runner](../../mizar-test/en/00.crate_plan.md#checker-task-258b3m2b2b3d-runner-frozen-contract) frozen contracts |
| E | Chapters 13 §§13.4/13.4.2, 4 §4.4.3, 15 §15.4.4, and 16 §16.3.3; omitted-condition case in [`pass_parser_set_comprehensions_001.miz`](../../../../tests/miz/pass/parser/pass_parser_set_comprehensions_001.miz); `{3 where candidate255 is set}` member of the same fail fixture, expectation, and trace | Exact checker four and runner five names/matrices in the [checker](../../mizar-checker/en/00.crate_plan.md#task-258b3m2b2b3e-frozen-independent-comprehension-witness-contract) and [runner](../../mizar-test/en/00.crate_plan.md#checker-task-258b3m2b2b3e-frozen-runner-contract) frozen contracts |

| Class | Decision |
|---|---|
| `design_drift` | Eight paired completion-log H3s repeat the same time-local evidence shape in one EN/JA payload-family owner; paired historical contracts become their completion-evidence owners. |
| `spec_gap` / `test_gap` | None for this structural migration. Generic schema-v1 lint covers the exact whole-section shape. |
| `source_drift` / `source_undocumented_behavior` | None introduced or inferred; Rust and Cargo are protected. |
| `test_expectation_drift` | None; specifications and every test-intent artifact are protected. |
| `boundary_violation` | Avoided by retaining all frozen H3/H2 owners, every unlisted section, all component API/invariant/runner/audit owners, and B3B's non-paired completion record. |
| `repo_metadata_conflict` | Historical remote-ref movement remains report-only and human-owned. Current `0/6` distance is measured, not repaired; no fetch/reset/push is authorized. |

## Frozen Preimage And Anchors

[`DOC-258B3M2B2B3-ACDE-COMPACT.sources.tsv`](../DOC-258B3M2B2B3-ACDE-COMPACT.sources.tsv)
contains eight byte-sorted data rows plus two comments and final LF. Data-row
SHA-256 is
`9046a2fa4a71e210ecf2e4d3fb1f115e426b070e7a5b434eb81dfc9fa4598fcc`;
the complete ten-line TSV SHA-256 is
`cad05407f570a7305bf31168a78de2a5dd577577b0abd6f7267fe07628010b5e`.

The selection is eight unique `(path, task)` H3s over two physical paths,
157 physical lines: EN/JA `4/79` and `4/78`; checker/runner `8/157` and
`0/0`. No selected section contains a nested heading, table, or fence, and no
contract, index, or ledger identity collides.

| Task | Retained preceding / following same-or-higher heading |
|---|---|
| A EN | `### Task 258B3M2B2B3A Frozen Upper-Family Edge` / `### Task 258B3M2B2B3B Frozen Zero-Edge Family Boundary` |
| A JA | `### Task 258B3M2B2B3A frozen upper-family edge` / `### Task 258B3M2B2B3B frozen zero-edge family boundary` |
| C EN | `## Task 258B3M2B2B3C Choice-Witness Family` / `### Task 258B3M2B2B3D Frozen Qua-Witness Edge` |
| C JA | `## Task 258B3M2B2B3C choice-witness family` / `### Task 258B3M2B2B3D frozen qua-witness edge` |
| D EN | `### Task 258B3M2B2B3D Frozen Qua-Witness Edge` / `### Task 258B3M2B2B3E Frozen Comprehension-Witness Edge` |
| D JA | `### Task 258B3M2B2B3D frozen qua-witness edge` / `### Task 258B3M2B2B3E frozen condition-free-comprehension witness edge` |
| E EN | `### Task 258B3M2B2B3E Frozen Comprehension-Witness Edge` / `## Task 258B4 Composite-Root Decomposition` |
| E JA | `### Task 258B3M2B2B3E frozen condition-free-comprehension witness edge` / `### Task 258B4 composite-root decomposition` |

B3B is expressly excluded: its EN implementation result is H3 while the JA
completion owner is H2. Schema version 1 cannot represent that asymmetry, and
neither side may be changed by this task.

## Frozen Protected Baseline

Selection-HEAD replay fixes these task-independent protected inventories. The
expected prerequisite and migration delta is zero for every row.

| Surface | Paths | Path SHA-256 | Content SHA-256 |
|---|---:|---|---|
| specification | 64 | `d900ba9e43ab094925f36493f830e0c6a2964be2911d5d229014a58842067a25` | `b30dd5519191a4407399826faf91cc58853d7944df7630734b5ab05de48c9f7b` |
| `.miz` | 343 | `d94980e167b4b8ac196f91e7694ff044080c6fb4d04c135b3cd5e65b9a019f22` | `54e6ea1254a0bd963c39026d788711507f353e9c6df3b4f9fd268b2e9f240afb` |
| expectation | 435 | `22a5ee257a294e3f2ed4b24bf9ca243d037bbd798f009d0d1e3a176dad8b4fea` | `b5f0ed1a8d73bbfb78af5cad87a8d426e97269f7f70163750893a9fe1f39d2ea` |
| checker production | 30 | `a41370d7150a587369cea5f7a67b60417dd1372592f55c0d65bec369eb39fdc6` | `05fd5e0eaed4361b824693941e9056a552c476f050915ea5052a85c8c7174dfd` |
| runner production | 90 | `05245a54160dfce17336b476b07885eb6d5afe138c4780a6a6a7b47043e7248c` | `210f294aebfe22c12324ef9919ac68147f8025f0da8de166403dada87bac5eae` |
| Cargo | 21 | `d93f2816b760d8ba45430d6d8570e598864aa7b20b19a001f45171d36fd3a030` | `146e9b4024d2c344b2eca9c6f5ca6d6a80a3de427e382953a2280bc63cb3ecca` |

The protected trace SHA-256 is
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.
Expected CLI stdout hashes are plan `700f4bf5`, parse-only `a8a7aa63`,
declaration-symbol `71e83ba0`, type-elaboration `4b2c7bd5`, and
proof-verification `ccf3d2d4`; final evidence records the reproduced full
hashes once.

## Scope, Impact, And Exit

The prerequisite changes exactly 15 paths: this EN/JA batch pair, four paired
historical contracts, the language-neutral source TSV, and four crate plans.
Each plan receives rows for A, C, D, E, and the batch: 20 index records.
Selected preimages and `legacy_compactions.tsv` remain unchanged.

After a separate prerequisite commit and fresh replay, migration may replace
only the eight complete H3s with language-local redirects to the corresponding
historical contract's `#completion-evidence`. It changes exactly the two
payload files, this EN/JA batch pair, and the ledger: five paths. The mapped
157 physical lines become 16 redirect-plus-separator lines, a reduction of
141. Expected ledger impact is one batch, four tasks, eight redirects over two
distinct source paths, 20 index records, and one expanded-inventory hash. The
source TSV, historical contracts, and four Task Indexes become immutable.

Specification, `.miz`, fixture, sidecar, expectation, trace status/count/
backlinks, coverage credit, root coverage audit, source, Cargo, public API,
diagnostics, and executable behavior remain unchanged. Goal/guard composition,
proof/discharge/acceptance, facts, Core/CFG/VC/ATP state, binding/capture,
sethood/type semantics, active dispatch, B4/B5 behavior, and new coverage
credit are forbidden. `doc/design/spec_coverage_audit.md` remains unchanged
because its owned mapping and status do not change.

Prerequisite and migration reviews must independently reproduce chronology,
all eight preimages and anchors, every task-specific fact, retained-owner
links, EN/JA equivalence, exact scopes, index/ledger arithmetic, and no-impact
claims, ending **NO FINDINGS**. Verification includes recursive contract/link/
fragment and generic-ledger lint, full lint policies, checker/runner/metadata
tests, formatting, Cargo metadata, warnings-denied Clippy, workspace tests,
all five CLIs, protected counts/hashes, `git diff --check`, exact staging, all
nine hard gates, and an uncapped score `>=90/100`. No push or stash mutation is
authorized.

## Documentation-Prerequisite Evidence

Pre-edit selection review ended **NO FINDINGS** and rejected the alternate
Task-258B3 whole-result family because its Typed, Resolved, runner-plan, and
boundary H3s contain durable owner-local facts. Contract review found one
medium `design_drift`: exact canonical/executable authority and existing test
owners were indirect. After adding the task-specific authority/test map and
the four historical-contract rows, finding-specific re-review ended
**NO FINDINGS**. Equivalence/ownership/EN-JA review found two medium
`design_drift` omissions in A and D completion evidence. After restoring the
coverage/fresh chronology and the complete `qua` composition, test split,
deferrals, corrections, operational residuals, and pending/closure chronology,
re-review ended **NO FINDINGS**. Independent test-sufficiency review ended
**NO FINDINGS** and confirmed that generic schema-v1 lint is sufficient and a
task-specific Rust/schema change is forbidden.

All eight preimages replayed at 157 lines with exact headings, hashes, and
anchors; both TSV hashes and the `4/79`, `4/78`, `8/157`, `0/0` partitions
matched. Recursive task-contract/link/fragment lint passed `1/1`; full
runner/checker lint policies passed `15/15` and `15/15`; checker/runner
libraries passed `530/530` and `600/600`; runner metadata passed `137/137`.
`cargo fmt --all --check`, Cargo metadata, warnings-denied all-target/
all-feature Clippy, full workspace tests, and `git diff --check` passed.

All five CLIs exited zero with the unchanged 23 warnings and these stdout
SHA-256 values:

| CLI | SHA-256 |
|---|---|
| plan | `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718` |
| parse-only | `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56` |
| declaration-symbol | `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74` |
| type-elaboration | `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f` |
| proof-verification | `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450` |

Protected counts, path hashes, content baselines, trace hash, selected
preimages, legacy ledger, and root coverage audit reproduced with zero task
delta. The worktree is the exact 15-path prerequisite.

Final read-only quality review ended **NO FINDINGS**. All nine hard gates pass,
no score cap applies, and the valid score is `100/100`
(`20/20/15/15/10/10/5/5`). It independently reproduced both lint policies at
`15/15`, `git diff --check`, exact scope, 20 indexes, TSV/preimage/anchor
replay, protected no-ops, and repository metadata. Residual risk is limited to
the separately committed migration and retained historical semantic/coverage
deferrals. Exact task-only staging/cached review, commit, and post-commit replay
remain.

## Migration Evidence

The prerequisite is commit
`497e60b2ad5ec338cf28d1846663364aeb45f6b6`. Its post-commit inventory was
clean at `origin/main...HEAD=0/7`, protected `stash@{0}` was unchanged, and all
eight frozen preimages replayed before editing.

The mechanical migration changes exactly the paired payload-family documents,
this EN/JA batch pair, and `legacy_compactions.tsv`: five paths. It replaces
only the eight declared complete H3s with language-local redirects. Their 157
physical lines become 16 redirect-plus-separator lines, a reduction of 141.
Every frozen/H2 owner, B3B record, adjacent anchor, and unlisted section
remains.

The generic ledger now has 592 physical lines and adds exactly one batch, four
tasks, eight redirects over two distinct source paths, and 20 index records.
Its expanded-inventory SHA-256 is
`89f03fdf9d967a1c5d72bbf4830acf1d8af7fa4af94d8da62e386f2c1bb857a9`;
its complete physical SHA-256 is
`d261a5c87f7f8adeb18cdfe0c9d49cc5d260f446120b7c09c48ca69d24cfddbb`.
The immutable source TSV remains
`cad05407f570a7305bf31168a78de2a5dd577577b0abd6f7267fe07628010b5e`.

Focused generic-ledger/link/fragment lint and `git diff --check` pass.
Specification, `.miz`, fixture, sidecar, expectation, trace status/count/
backlinks, coverage credit, source, Cargo, public API, diagnostics, root
coverage audit, historical contracts, source TSV, and four Task Indexes are
unchanged.

Independent test-sufficiency review ended **NO FINDINGS** and confirmed that
the generic schema-v1 lint is sufficient for the exact whole-section shape;
task-specific Rust or schema changes would be unwarranted. Equivalence and
boundary review found one medium `design_drift`: the identity table still
said that the prerequisite was pending after its commit. After synchronizing
the EN/JA cells to `497e60b2`, finding-specific re-review ended
**NO FINDINGS**. It replayed all eight preimages, hashes, line counts, and
anchors and confirmed that the frozen owners, asymmetric B3B material, and
unlisted sections remain. Independent source/documentation and EN/JA review
also ended **NO FINDINGS** after reproducing the ledger order, arithmetic,
both hashes, source-TSV hashes, language-local redirects, and protected
no-impact claims.

Full migration verification passed: recursive runner and checker lint policies
`15/15` and `15/15`; checker and runner libraries `530/530` and `600/600`;
runner metadata `137/137`; `cargo fmt --all --check`; offline Cargo metadata;
warnings-denied all-target/all-feature Clippy; and the complete offline
workspace test suite. All five CLIs exited zero with `23/0` unchanged
warnings/errors and exactly reproduced the prerequisite stdout hashes:

| CLI | SHA-256 |
|---|---|
| plan | `700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718` |
| parse-only | `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56` |
| declaration-symbol | `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74` |
| type-elaboration | `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f` |
| proof-verification | `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450` |

The frozen protected counts and path/content hashes reproduce with zero delta;
the trace remains `55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.
The ledger remains 592 lines at physical hash `d261a5c8...ddbb` and expanded
hash `89f03fdf...857a`; the immutable ten-line source TSV remains
`cad05407...0b5e`. Exact protected-surface diffs against the prerequisite are
empty, the worktree is exactly the declared five paths, and `git diff --check`
passes. At that verification checkpoint, final quality and the subsequent
staging/commit gates remained.

Independent final read-only quality review ended **NO FINDINGS**. All nine
hard gates PASS, no score cap applies, and the valid score is `100/100`
(`20/20/15/15/10/10/5/5`). The reviewer independently reproduced the exact
eight-section migration, ledger arithmetic and hashes, protected boundaries,
focused and both full lint policies, checker/runner/metadata counts, format,
Cargo metadata, and whitespace. Its residual non-rerun set was full Clippy,
the workspace suite, and five CLIs, all of which the parent had already passed
against the unchanged diff with the exact results above. Only exact staging/
cached review, commit, and post-commit replay remain.

## Handoff

After both task-only commits and clean post-commit inventory, select exactly
one dependency-ready checker-owned whole-section duplication family. Keep the
parent at `xhigh`, use `high` for independent semantic/equivalence reviews,
and use `medium` only for deterministic inventory.
