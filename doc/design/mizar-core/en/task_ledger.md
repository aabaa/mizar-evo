# Task Ledger: mizar-core

> Canonical language: English. Japanese companion:
> [../ja/task_ledger.md](../ja/task_ledger.md).

This ledger is the restart point for autonomous `mizar-core` crate work. Before
starting any task, check `git status`, `git log`, and this table. A task is
complete only when it has a commit hash, final review outcomes, verification
results, and deferred reasons.

| Task | Status | Commit | Reviews | Verification | Deferred / notes |
|---|---|---|---|---|---|
| 0. Crate plan | ready to commit | recorded after commit in task handoff; self-hash cannot be embedded in the commit that creates this row | Spec/doc review: no blocking/high/medium findings. Test sufficiency review: medium staged-check gap fixed, re-review no findings. Full implementation review: no blocking/high/medium findings. Source/doc consistency review: no blocking/high/medium findings. | `git diff --check` passed before staging; `git diff --cached --check` passed after explicit path staging. | Docs-only. Deferred external gaps classified in `00.crate_plan.md`; no task-local deferred work. |
| 1. Crate scaffold and lint-policy guard | ready to commit | recorded after commit in task handoff; self-hash cannot be embedded in the commit that updates this row | Spec/doc review: medium lint-boundary findings fixed, re-review no findings. Test sufficiency review: medium alias/API-surface findings fixed, re-review no findings. Full implementation review: no blocking/high/medium findings. Source/doc consistency review: no blocking/high/medium findings. | `cargo fmt --check`; `cargo test -p mizar-core`; `cargo clippy -p mizar-core --all-targets -- -D warnings`; `git diff --check` before staging; `git diff --cached --check` after explicit path staging. | Scaffold-only. Adds workspace member, lockfile entry, minimal crate, and lint guard. No deferred task-local work. |
| 2. Spec: `core_ir.md` | ready to commit | recorded after commit in task handoff; self-hash cannot be embedded in the commit that updates this row | Spec/doc review: medium seed/statement/diagnostic/choice/comprehension/source-map findings fixed, re-review no findings. Test sufficiency review: no blocking/high/medium findings. Full implementation review: medium findings fixed via spec/doc re-review. Source/doc consistency review: no blocking/high/medium findings. | `git diff --check` before staging; `git diff --cached --check` after explicit path staging. | Spec-only. Rust tests deferred to task 3; source-derived `.miz` core fixtures deferred until checker payload extraction and mizar-test stage support exist. |
| 3. Implement `core_ir` data shapes | ready to commit | recorded after commit in task handoff; self-hash cannot be embedded in the commit that updates this row | Spec/doc review: medium stable-choice/proof-status/source-validation findings fixed, low owner-diagnostic/doc-status findings fixed, final re-review no blocking/high/medium findings. Test sufficiency review: medium invalid-reference/seed/debug/error coverage findings fixed; medium diagnostic/provenance tie-break coverage and low source-map/owner/error mismatch gaps fixed, final re-review no findings. Full implementation review: medium algorithm-owner/generated-origin/provenance findings fixed, low crate-doc/source-sort findings fixed. Source/doc consistency review: medium proof-node/provenance/label/status doc findings fixed, final re-review no findings. | `cargo fmt --check`; `cargo test -p mizar-core`; `cargo clippy -p mizar-core --all-targets -- -D warnings`; stage checks pending. | Implements `core_ir` data shapes, validation, deterministic debug rendering, and lint guard expansion. Source-derived `.miz` core fixtures remain deferred until checker payload extraction and mizar-test stage support exist; no task-local external dependency is fabricated. |
| 4. Spec: `binder_normalization.md` | not started | pending | pending | pending |  |
| 5. Binder representation and substitution | not started | pending | pending | pending |  |
| 6. Alpha-equivalence and normalization utilities | not started | pending | pending | pending |  |
| 7. Spec: `elaborator.md` | not started | pending | pending | pending |  |
| 8. Core context preparation | not started | pending | pending | pending |  |
| 9. Type and fact lowering | not started | pending | pending | pending |  |
| 10. Term and formula lowering | not started | pending | pending | pending |  |
| 11. Definition lowering | not started | pending | pending | pending |  |
| 12. Proof-skeleton lowering | not started | pending | pending | pending |  |
| 13. Algorithm-shell lowering | not started | pending | pending | pending |  |
| 14. Spec: `control_flow.md` | not started | pending | pending | pending |  |
| 15. `ControlFlowIr` construction | not started | pending | pending | pending |  |
| 16. Contracts, ghost effects, termination measures | not started | pending | pending | pending |  |
| 17. Flow diagnostics | not started | pending | pending | pending |  |
| 18. Obligation-seed handoff contract | not started | pending | pending | pending |  |
| 19. Snapshot dumps and corpus contributions | not started | pending | pending | pending |  |
| 20. Determinism suite | not started | pending | pending | pending |  |
| 21. Public-enum forward-compatibility policy | not started | pending | pending | pending |  |
| 22. Source/spec correspondence audit | not started | pending | pending | pending |  |
| 23. Bilingual documentation sync audit | not started | pending | pending | pending |  |
| 24. Module-boundary refactor gate | not started | pending | pending | pending |  |
| Closeout. Crate exit report and quality review | not started | pending | pending | pending | Requires all tasks complete and quality review score >= 90. |
