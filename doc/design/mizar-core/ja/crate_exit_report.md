# Crate Exit Report: mizar-core

> 正本言語: English。英語正本:
> [../en/crate_exit_report.md](../en/crate_exit_report.md)。

## Result

Status: complete。
Quality score: 94/100。
Score caps applied: none。

## Scope

Milestone scope:

- `mizar-core` crate を task 24 と closeout task まで構築する。
- 明示的な `mizar_checker::resolved_typed_ast::ResolvedTypedAst` 由来 payload
  から backend-neutral な `CoreIr` へ lower する phase 9 elaboration を所有する。
- core algorithm shell から `ControlFlowIr` へ準備する phase 10 control-flow
  preparation を所有する。
- binder normalization、alpha-equivalence、capture-avoiding substitution、
  soft type erasure の明示 record、definition boundary、proof skeleton、
  algorithm shell、structured core/flow diagnostic、obligation seed handoff を
  所有する。
- 未完成の upstream/downstream seam は placeholder behavior を仮造せず、
  external dependency gap または deferred work として分類する。

Included:

- `doc/design/mizar-core/{en,ja}/` 配下の英日 crate plan、module spec、audit、
  closeout report。
- `crates/mizar-core/src/` 配下の Rust source。
- `crates/mizar-core/` 配下の crate-local unit test と integration test。
- `tests/coverage/spec_trace.toml` の deferred source-derived snapshot
  traceability row。

Excluded:

- `doc/spec` への直接編集。
- 既存 `.miz` fixture または expectation sidecar の rebaseline。
- 欠けている semantic payload の source-to-checker / source-to-core extraction。
- VC generation、proof acceptance、kernel checking、artifact schema emission、
  proof/cache reuse、public diagnostic-code allocation。
- 具体的な downstream `VcId`、`ObligationAnchor`、VC fingerprint、artifact
  identity assignment。
- 利用不能な `mizar-vc`、`mizar-kernel`、`mizar-proof`、diagnostics registry、
  artifact、source-extraction seam の placeholder module。

## Task Commits

| Task | Commit | Subject |
|---|---|---|
| 0 | `57fb027` | `docs(core-task-0): add autonomous crate plan` |
| 1 | `c3dccec` | `feat(core-task-1): scaffold mizar-core crate` |
| 2 | `40d9c3b` | `docs(core-task-2): specify core ir data shapes` |
| 3 | `b58be7c` | `feat(core-task-3): implement core ir data shapes` |
| 4 | `6e7c458` | `docs(core-task-4): specify binder normalization` |
| 5 | `300f814` | `feat(core-task-5): implement binder substitution` |
| 6 | `841ba6e` | `feat(core-task-6): add alpha normalization utilities` |
| 7 | `0290541` | `docs(core-task-7): specify elaborator lowering` |
| 8 | `a0d6d3f` | `feat(core-task-8): prepare elaboration context` |
| 9 | `860736a` | `feat(core-task-9): lower type facts` |
| 10 | `14e76d6` | `feat(core-task-10): lower terms and formulas` |
| 11 | `9523a0e` | `feat(core-task-11): lower definitions` |
| 12 | `e4afecb` | `feat(core-task-12): lower proof skeletons` |
| 13 | `23d420b` | `feat(core-task-13): lower algorithm shells` |
| 14 | `93a66e3` | `docs(core-task-14): specify control-flow ir` |
| 15 | `73a5786` | `feat(core-task-15): build control-flow ir` |
| 16 | `07d802f` | `feat(core-task-16): attach control-flow contracts` |
| 17 | `004c837` | `feat(core-task-17): add flow diagnostics` |
| 18 | `64ce704` | `feat(core-task-18): add obligation seed handoff` |
| 19 | `45a8762` | `docs(core-task-19): defer corpus snapshot seams` |
| 20 | `6bfe55b` | `test(core-task-20): add determinism suite` |
| 21 | `184c0f1` | `test(core-task-21): guard public enum policy` |
| 22 | `6b62bc8` | `test(core-task-22): audit source spec correspondence` |
| 23 | `260e41e` | `docs(core-task-23): audit bilingual documentation sync` |
| 24 | `6c7268d` | `docs(core-task-24): audit module boundaries` |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | 英日 module spec と closeout review は unresolved blocking/high/medium specification inconsistency がないことを確認した。 |
| Test contract | passed | Rust unit test、integration test、lint guard、determinism test、deferred traceability が milestone-owned behavior を cover する。 |
| Traceability | passed | `source_spec_audit.md`、`bilingual_sync_audit.md`、`module_boundary_audit.md`、本 report、task ledger、`tests/coverage/spec_trace.toml` が implemented/tested/deferred surface を記録する。 |
| Design/source sync | passed | Task 22-24 audit と closeout source/documentation review は source/spec、bilingual、module-boundary drift なしを記録する。 |
| Boundary discipline | passed | Core は explicit-payload lowering、binder normalization、CFG preparation、diagnostic、obligation seed のみを所有する。downstream proof/VC/kernel/artifact seam は deferred。 |
| Verification | passed | Closeout broad command と diff check は commit 前に通過済み。 |
| Residual risk | passed with deferred items | 残る利用不能 seam は下で external dependency gap または deferred work として分類する。 |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 18/20 |
| Traceability | 14/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 9/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 5/5 |
| Total | 94/100 |

## Review Results

| Review | Result |
|---|---|
| Implementation specification / documentation review | corrected closeout spec が completion を hard gate 条件付きにし、ledger commit-hash backfill を要求し、crate-exit template に一致した後、blocking/high/medium finding なし。 |
| Test sufficiency review | blocking/high/medium finding なし。docs-only closeout scope には broad workspace verification と diff check で十分であり、Rust source、`.miz`、expectation、`doc/spec`、traceability metadata の変更は不要と確認。 |
| Full implementation review | Japanese Task 3 ledger verification drift と bilingual audit の stale closeout wording を修正後、blocking/high/medium finding なし。 |
| Source/documentation consistency review | blocking/high/medium finding なし。task hash、bilingual closeout sync、deferred seam classification、docs-only scope が一致していることを確認。 |
| Read-only crate quality review | hard gate failure、blocking/high/medium finding、score cap なし。Valid score は 94/100 で 90 以上。 |

Quality-review residual risk: source-to-checker extraction、active
`type_elaboration` / `proof_verification` snapshot、downstream
VC/proof/kernel/artifact consumer、具体的な VC identity / anchor、public
diagnostic code-space、より豊かな source-derived payload、optional private
helper extraction は crate-owned blocker ではなく external または deferred のまま。

## Deferred Items

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| CORE-AUDIT-G001 | Source-to-checker extraction が full source-derived `ResolvedTypedAst` payload と production source-to-core fixture をまだ blocking している。 | Checker extraction / mizar-test integration follow-up。 | `mizar-core` が raw syntax を再走査しなくても checker-ready AST-wide payload extraction が存在する。 |
| CORE-AUDIT-G002 | `CoreIr` と `ControlFlowIr` 用の active source-derived `type_elaboration` / `proof_verification` snapshot runner がまだ存在しない。 | `mizar-test` staged runner follow-up。 | real checker-derived `CoreIr` / `ControlFlowIr` baseline を比較できる stage runner が存在する。 |
| CORE-AUDIT-G003 | Artifact schema emission、proof acceptance、VC generation、kernel checking は downstream または cross-crate work。 | `mizar-artifact`、`mizar-proof`、`mizar-vc`、`mizar-kernel` phase。 | Downstream crate が core/control-flow handoff 用の accepted schema と consumer を定義する。 |
| CORE-AUDIT-G004 | 具体的な `VcId`、`ObligationAnchor`、VC fingerprint、proof/cache reuse anchor、downstream artifact identity は `mizar-core` の所有ではない。 | `mizar-vc` incremental verification / artifact phase。 | Downstream identity と anchor contract が存在する。 |
| CORE-AUDIT-G005 | Source-derived call/result substitution、pattern、snapshot、claim、より豊かな algorithm payload seam には checker-owned explicit payload が必要。 | Checker payload extraction と phase-10/phase-11 integration。 | それらの source form 用の explicit checker payload が存在する。 |
| CORE-AUDIT-G006 | Public diagnostic code-space はこの crate で割り当てない。 | Diagnostics registry owner。 | 共有 public diagnostic registry と allocation policy が存在する。 |
| CORE-BOUNDARY-G001 | `src/elaborator.rs` は大きく、future private helper/test extraction が有益になる可能性がある。 | Future move-only core maintenance task。 | reviewability bottleneck が発生し、public API と behavior を保つ専用 split task を実行できる。 |
| CORE-BOUNDARY-G002 | `src/control_flow.rs` は大きく、private builder/diagnostic/handoff helper extraction が有益になる可能性がある。 | Future move-only core maintenance task。 | reviewability bottleneck が発生し、public API と behavior を保つ専用 split task を実行できる。 |
| CORE-BOUNDARY-G003 | `src/binder_normalization.rs` は大きく、private helper extraction が有益になる可能性がある。 | Future move-only core maintenance task。 | reviewability bottleneck が発生し、public API と behavior を保つ専用 split task を実行できる。 |

## Human Review Surface

- `doc/design/mizar-core/en/` 配下の英語正本。
- `doc/design/mizar-core/ja/` 配下の日本語 companion。
- `crates/mizar-core/src/` 配下の core source。
- `crates/mizar-core/tests/` と module-local Rust test。
- `tests/coverage/spec_trace.toml` の deferred corpus traceability row。
- Upstream checker inputs:
  `doc/design/mizar-checker/en/crate_exit_report.md`,
  `doc/design/mizar-checker/en/source_spec_audit.md`,
  `doc/design/mizar-checker/en/resolved_typed_ast.md`。

## Test Expectation Summary

既存 `.miz` fixture や expectation sidecar は implementation behavior に合わせるために
変更していない。Milestone-owned behavior は Rust unit test、integration test、
lint-policy guard、determinism test、または explicit deferred traceability row で
cover される。Source-derived semantic corpus coverage は上記の external extraction と
staged-runner gap により blocked のまま。

## Verification Commands

| Command | Result |
|---|---|
| `cargo fmt --check` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test` | passed |
| `git diff --check` | passed |
| `git diff --cached --check` | 明示 closeout path staging 後に passed |

## Next-Task Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue from the completed mizar-core autonomous crate milestone after the
closeout commit. Start the next crate or integration phase from a clean
worktree. Use doc/design/mizar-core/en/crate_exit_report.md,
source_spec_audit.md, module_boundary_audit.md, and the CORE-AUDIT /
CORE-BOUNDARY deferred rows as inputs. Do not fabricate source-to-checker or
source-to-core payloads, active type_elaboration/proof_verification snapshots,
artifact schemas, proof acceptance, VC generation, kernel checking, concrete
VcIds, ObligationAnchors, proof/cache reuse anchors, public diagnostic codes, or
source-derived call/pattern/snapshot/claim payloads. Select the owning
crate/task for the missing seam and follow AGENTS.md with one task per commit.
```

Rationale: next work crosses crate boundaries into upstream extraction,
downstream VC/kernel/proof/artifact consumers, diagnostics, or move-only source
maintenance. Narrow docs-only synchronization or mechanical guard maintenance
なら reasoning を下げてもよい。semantic behavior、binder/proof boundary、VC
identity、artifact schema、proof/kernel integration では `xhigh` を維持または上げる。
