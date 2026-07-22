# Crate Exit Report: mizar-core

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete.
Quality score: 94/100.
Score caps applied: none.

## Scope

Milestone scope:

- Build the `mizar-core` crate through task 24 and the closeout task.
- Own phase 9 elaboration from explicit
  `mizar_checker::resolved_typed_ast::ResolvedTypedAst`-derived payloads into
  backend-neutral `CoreIr`.
- Own phase 10 control-flow preparation from core algorithm shells into
  `ControlFlowIr`.
- Own binder normalization, alpha-equivalence, capture-avoiding substitution,
  explicit soft-type erasure records, definition boundaries, proof skeletons,
  algorithm shells, structured core/flow diagnostics, and obligation seed
  handoff.
- Keep unfinished upstream/downstream seams classified as external dependency
  gaps or deferred work instead of fabricating placeholder behavior.

Included:

- English/Japanese mizar-core crate plan, module specifications, audits, and
  closeout report under `doc/design/mizar-core/{en,ja}/`.
- Rust source under `crates/mizar-core/src/`.
- Crate-local unit tests and integration tests under `crates/mizar-core/`.
- Deferred source-derived snapshot traceability rows in
  `tests/coverage/spec_trace.toml`.

Excluded:

- Direct edits to `doc/spec`.
- Rebaselining existing `.miz` fixtures or expectation sidecars.
- Source-to-checker or source-to-core extraction for missing semantic payloads.
- VC generation, proof acceptance, kernel checking, artifact schema emission,
  proof/cache reuse, and public diagnostic-code allocation.
- Concrete downstream `VcId`, `ObligationAnchor`, VC fingerprint, or artifact
  identity assignment.
- Placeholder modules for unavailable `mizar-vc`, `mizar-kernel`,
  `mizar-proof`, diagnostics registry, artifact, or source-extraction seams.

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
| Specification consistency | passed | English/Japanese module specs and closeout reviews found no unresolved blocking/high/medium specification inconsistency. |
| Test contract | passed | Rust unit tests, integration tests, lint guards, determinism tests, and deferred traceability cover milestone-owned behavior. |
| Traceability | passed | `source_spec_audit.md`, `bilingual_sync_audit.md`, `module_boundary_audit.md`, this report, task ledgers, and `tests/coverage/spec_trace.toml` record implemented, tested, and deferred surfaces. |
| Design/source sync | passed | Task 22-24 audits and the closeout source/documentation review report no source/spec, bilingual, or module-boundary drift. |
| Boundary discipline | passed | Core owns explicit-payload lowering, binder normalization, CFG preparation, diagnostics, and obligation seeds only; downstream proof/VC/kernel/artifact seams remain deferred. |
| Verification | passed | Closeout broad commands and diff checks passed before commit. |
| Residual risk | passed with deferred items | Remaining unavailable seams are classified below as external dependency gaps or deferred work. |

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
| Implementation specification / documentation review | No blocking/high/medium findings after the corrected closeout specification made completion conditional on hard gates, required ledger commit-hash backfill, and matched the crate-exit template. |
| Test sufficiency review | No blocking/high/medium findings. The review confirmed that broad workspace verification plus diff checks are sufficient for the docs-only closeout scope and that no Rust source, `.miz`, expectation, `doc/spec`, or traceability metadata changes are required. |
| Full implementation review | No blocking/high/medium findings after Japanese Task 3 ledger verification drift and stale closeout wording in the bilingual audit were fixed. |
| Source/documentation consistency review | No blocking/high/medium findings. The review confirmed task hashes, bilingual closeout sync, deferred seam classifications, and docs-only scope. |
| Read-only crate quality review | No hard gate failures, blocking/high/medium findings, or score caps. Valid score 94/100, which is >= 90. |

Quality-review residual risks: source-to-checker extraction, active
`type_elaboration` / `proof_verification` snapshots, downstream
VC/proof/kernel/artifact consumers, concrete VC identities and anchors, public
diagnostic code-space, richer source-derived payloads, and optional private
helper extraction remain external or deferred rather than crate-owned blockers.

## Deferred Items

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|
| CORE-AUDIT-G001 | Source-to-checker extraction still blocks full source-derived `ResolvedTypedAst` payloads and production source-to-core fixtures. | Checker extraction / mizar-test integration follow-up. | Checker-ready AST-wide payload extraction exists without raw syntax rescanning in `mizar-core`. |
| CORE-AUDIT-G002 | Core Task 31 now provides the one exact Task-180 source-derived `type_elaboration` `CoreIr` snapshot; all non-Task-180 `CoreIr`, all `ControlFlowIr`, and `proof_verification` snapshot runners remain unavailable. | Core Task 32 decomposition after checker Task 247. | Bounded descendant tasks provide prepared stage consumers for the remaining real checker-derived baselines. |
| CORE-AUDIT-G003 | Artifact schema emission, proof acceptance, VC generation, and kernel checking are downstream or cross-crate work. | `mizar-artifact`, `mizar-proof`, `mizar-vc`, and `mizar-kernel` phases. | Downstream crates define accepted schemas and consumers for core/control-flow handoff. |
| CORE-AUDIT-G004 | Concrete `VcId`, `ObligationAnchor`, VC fingerprints, proof/cache reuse anchors, and downstream artifact identities are not owned by `mizar-core`. | `mizar-vc` incremental verification / artifact phases. | Downstream identity and anchor contracts exist. |
| CORE-AUDIT-G005 | Source-derived call/result substitution, pattern, snapshot, claim, and richer algorithm payload seams require checker-owned explicit payloads. | Checker payload extraction plus phase-10/phase-11 integration. | Explicit checker payloads exist for those source forms. |
| CORE-AUDIT-G006 | Public diagnostic code-space is not allocated by this crate. | Diagnostics registry owner. | Shared public diagnostic registry and allocation policy exist. |
| CORE-BOUNDARY-G001 | `src/elaborator.rs` is large and may benefit from future private helper/test extraction. | Future move-only core maintenance task. | Reviewability bottleneck emerges and a dedicated split task can preserve public API and behavior. |
| CORE-BOUNDARY-G002 | `src/control_flow.rs` is large and may benefit from private builder/diagnostic/handoff helper extraction. | Future move-only core maintenance task. | Reviewability bottleneck emerges and a dedicated split task can preserve public API and behavior. |
| CORE-BOUNDARY-G003 | `src/binder_normalization.rs` is large and may benefit from private helper extraction. | Future move-only core maintenance task. | Reviewability bottleneck emerges and a dedicated split task can preserve public API and behavior. |

## Human Review Surface

- Canonical English docs under `doc/design/mizar-core/en/`.
- Japanese companions under `doc/design/mizar-core/ja/`.
- Core source under `crates/mizar-core/src/`.
- Core tests under `crates/mizar-core/tests/` and module-local Rust tests.
- Deferred corpus traceability rows in `tests/coverage/spec_trace.toml`.
- Upstream checker inputs:
  `doc/design/mizar-checker/en/crate_exit_report.md`,
  `doc/design/mizar-checker/en/source_spec_audit.md`, and
  `doc/design/mizar-checker/en/resolved_typed_ast.md`.

## Test Expectation Summary

No existing `.miz` fixtures or expectation sidecars were changed to match
implementation behavior. Milestone-owned behavior is covered by Rust unit
tests, integration tests, lint-policy guards, determinism tests, or explicit
deferred traceability rows. Source-derived semantic corpus coverage remains
blocked by the external extraction and staged-runner gaps listed above.

## Verification Commands

| Command | Result |
|---|---|
| `cargo fmt --check` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test` | passed |
| `git diff --check` | passed |
| `git diff --cached --check` | passed after explicit closeout path staging |

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
maintenance. Lower reasoning is acceptable only for narrow docs-only
synchronization or mechanical guard maintenance; keep or raise to `xhigh` for
semantic behavior, binder/proof boundaries, VC identity, artifact schemas, or
proof/kernel integration.

## Core Task 31 Post-Closeout Addendum

The historical closeout and score above remain valid for the original crate
milestone. Later Step-5 authority re-opened one bounded integration task. Core
Task 31 now consumes only the exact checker Task-180 singleton bundle through
`lower_exact_task180_handoff`, produces one public structurally valid theorem,
one `False`, one `PendingAutomaticProof`, one direct terminal node, and one
Active undischarged `TheoremProof` seed, and verifies the full deterministic
debug baseline in the active type-elaboration runner. The adapter remains
syntax-free and transactional; it adds no acceptance, discharge, CFG, VC,
kernel, artifact, or broader-family behavior. CORE-AUDIT-G002 therefore remains
open only for every non-Task-180 family and later stages owned by Core Task 32.
