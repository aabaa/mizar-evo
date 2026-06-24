# Task Ledger: mizar-vc

> Canonical language: English. Japanese companion:
> [../ja/task_ledger.md](../ja/task_ledger.md).

This ledger is the restart point for autonomous `mizar-vc` crate work. Before
starting any task, check `git status`, `git log`, this table, and
[todo.md](./todo.md). A task is complete only when its commit exists in
history, final review outcomes are known, verification results are known, and
deferred reasons are recorded. A commit cannot contain its own final hash, so
self-hashes are verified from `git log` before the next task starts and
backfilled by a later committed bookkeeping point or the closeout task.

| Task | Status | Commit | Reviews | Verification | Deferred / notes |
|---|---|---|---|---|---|
| 0. Crate plan | ready to commit | pending self-hash; verify from `git log` after commit | Spec/doc review: medium registration-correctness and derived-doc authority findings fixed; final re-review no blocking/high/medium findings. Test sufficiency review: no findings. Full implementation review: low future-link and stale task-scope findings fixed; final re-review no blocking/high/medium findings. Source/doc consistency review: medium task-15 and conditional-verification findings fixed; final re-review no blocking/high/medium findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Docs-only. Classifies initial `spec_gap`, `test_gap`, `design_drift`, `source_drift`, `external_dependency_gap`, and `deferred` rows in `00.crate_plan.md`; synchronizes todo wording for current runner/verification gaps and registration-style correctness seed scope; no crate source is created. |
| 1. Crate scaffold and lint-policy guard | not started | pending | pending | pending | Must not start until task 0 commit exists in `git log`. |
| 2. Spec: `vc_ir.md` | not started | pending | pending | pending | Spec-only task after scaffold. |
| 3. Implement `vc_ir` data shapes | not started | pending | pending | pending | Rust source task. |
| 4. Obligation-seed intake | not started | pending | pending | pending | Rust source task. |
| 5. Spec: `generator.md` | not started | pending | pending | pending | Spec-only task; includes registration-style correctness seed scope when explicit payloads exist. |
| 6. Theorem, definition, and registration-style correctness VCs | not started | pending | pending | pending | Rust source task; unavailable explicit registration payloads stay external/deferred. |
| 7. Algorithm VCs | not started | pending | pending | pending | Rust source task. |
| 8. Normalization, classification, and `VcId` assignment | not started | pending | pending | pending | Rust source task. |
| 9. Status and policy model | not started | pending | pending | pending | Rust source task. |
| 10. Spec: `discharge.md` | not started | pending | pending | pending | Spec-only task. |
| 11. Deterministic discharge engine | not started | pending | pending | pending | Rust source task. |
| 12. Discharge evidence and explanations | not started | pending | pending | pending | Rust source task. |
| 13. Spec: `dependency_slice.md` | not started | pending | pending | pending | Spec-only task. |
| 14. Dependency-slice computation | not started | pending | pending | pending | Rust source task. |
| 15. Corpus runner record for `proof_verification` | not started | pending | pending | pending | Deferred-record task unless runner/extraction seams exist by then. |
| 16. Determinism suite | not started | pending | pending | pending | Test task plus source fixes only when spec-backed. |
| 17. Public-enum forward-compatibility policy | not started | pending | pending | pending | Test/docs task. |
| 18. Source/spec correspondence audit | not started | pending | pending | pending | Audit task. |
| 19. Bilingual documentation sync audit | not started | pending | pending | pending | Audit/docs task. |
| 20. Obligation anchors and cross-edit reuse identity | not started | pending | pending | pending | Rust source task over architecture-22 identity. |
| 21. Architecture-22 follow-up audit | not started | pending | pending | pending | Audit task. |
| 22. Module-boundary refactor gate | not started | pending | pending | pending | Audit task; source moves only if required. |
| Closeout. Crate exit report and quality review | not started | pending | pending | pending | Complete only after hard gates pass and read-only quality score is >= 90. |

## Task 0 Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-vc autonomous crate development from the completed task 0
crate-plan commit. Before starting task 1, verify a clean worktree, confirm the
task 0 commit exists in git log, and re-read
doc/design/mizar-vc/en/00.crate_plan.md, task_ledger.md, and todo.md. Implement
task 1 only: add the mizar-vc workspace member, crate manifest, minimal
src/lib.rs, and lint-policy guard. Keep the scope scaffold-only; do not add
semantic VC APIs until vc_ir.md exists. Run cargo fmt --check,
cargo test -p mizar-vc, cargo clippy -p mizar-vc --all-targets -- -D warnings,
git diff --check, and git diff --cached --check after explicit path staging.
Use review-only agents for the required AGENTS.md review phases.
```

Rationale: task 1 changes workspace and Rust crate scaffolding, so xhigh keeps
the manifest, lint policy, and one-task-one-commit constraints in view. Lower
reasoning is acceptable only for a purely mechanical ledger typo fix; keep
`xhigh` if dependencies, lint policy, or workspace membership are touched.
