# Task Ledger: mizar-kernel

> Canonical language: English. Japanese companion:
> [../ja/task_ledger.md](../ja/task_ledger.md).

This ledger is the restart point for autonomous `mizar-kernel` crate work.
Before starting any task, check `git status`, `git log`, this table, and
[todo.md](./todo.md). A task is complete only when its commit exists in
history, final review outcomes are known, verification results are known, and
deferred reasons are recorded. A commit cannot contain its own final hash, so
self-hashes are verified from `git log` before the next task starts and
backfilled by a later committed bookkeeping point or the closeout task.

| Task | Status | Commit | Reviews | Verification | Deferred / notes |
|---|---|---|---|---|---|
| 0. Crate plan | ready to commit | pending self-hash | Spec/doc review: low pending-status finding fixed; final re-review no findings. Test sufficiency review: medium `--all-features` and conditional cross-crate verification findings fixed; final re-review no findings. Full implementation review: high sequencing and medium cluster-gate/status findings fixed; final re-review no findings. Source/doc consistency review: medium internal-04 rejection-reason and low JA companion-link findings fixed; final re-review no findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Docs-only. Creates paired crate plan and ledger, classifies initial `spec_gap`, `test_gap`, `design_drift`, `source_drift`, `external_dependency_gap`, `deferred`, and `repo_metadata_conflict` state, records kernel prohibitions and trusted-baseline lint policy, strict linear task sequencing, internal-04 rejection reason coverage, and cluster trace external-readiness gates, and does not create crate source. |
| 1. Crate scaffold and trusted-baseline lint policy | not started | pending | pending | pending | Requires task 0 commit in `git log`. |
| 2. Spec: `clause.md` | not started | pending | pending | pending | Requires task 1 commit. Spec-only. |
| 3. Implement clause representation | not started | pending | pending | pending | Requires task 2 commit. Rust source task. |
| 4. Spec: `certificate_parser.md` | not started | pending | pending | pending | Requires task 3 commit. Semantic dependency: task 2 clause spec. Spec-only. |
| 5. Implement certificate parsing and structural validation | not started | pending | pending | pending | Requires task 4 commit. Rust source task. |
| 6. Spec: `rejection.md` | not started | pending | pending | pending | Requires task 5 commit. Semantic dependency: task 1 trusted baseline. Spec-only. |
| 7. Implement rejection records | not started | pending | pending | pending | Requires task 6 commit. Semantic dependency: task 5 parser and task 6 rejection spec. Rust source task. |
| 8. Spec: `resolution_trace.md` | not started | pending | pending | pending | Requires task 7 commit. Semantic dependency: task 4 certificate spec. Spec-only. |
| 9. Implement resolution trace checker | not started | pending | pending | pending | Requires task 8 commit. Semantic dependency: task 7 rejection records. Rust source task. |
| 10. Spec: `substitution_checker.md` | not started | pending | pending | pending | Requires task 9 commit. Semantic dependency: task 4 certificate spec. Spec-only. |
| 11. Implement substitution checking | not started | pending | pending | pending | Requires task 10 commit. Semantic dependency: task 7 rejection records. Rust source task. |
| 12. Implement alpha-conversion and free-variable checks | not started | pending | pending | pending | Requires task 11 commit. Rust source task. |
| 13. Spec: `checker.md` | not started | pending | pending | pending | Requires task 12 commit. Semantic dependencies: task 6 rejection spec, task 8 resolution spec, and task 10 substitution spec. Spec-only. |
| 14. Implement imported-fact checking | not started | pending | pending | pending | Requires task 13 commit. Rust source task. |
| 15. Implement cluster trace replay | not started | pending | pending | pending | Requires task 14 commit. Semantic dependency: task 13 checker spec plus external `mizar-checker` cluster trace payload readiness review or deferred record. Rust source task. |
| 16. Kernel check service and deterministic batch ordering | not started | pending | pending | pending | Requires task 15 commit. Semantic dependencies: task 9 resolution checker, task 12 substitution checker, task 14 imported-fact checking, and task 15 cluster replay. Rust source task. |
| 17. Soundness fail-test corpus | not started | pending | pending | pending | Requires task 16 commit. Test/audit task; source-derived corpus runner gaps may remain `external_dependency_gap`. |
| 18. Determinism and replay-cost suite | not started | pending | pending | pending | Requires task 17 commit. Semantic dependency: task 16 checker service. Test task. |
| 19. Public-enum forward-compatibility policy | not started | pending | pending | pending | Requires task 18 commit. Semantic dependency: task 16 public API surface. Test/docs task. |
| 20. Source/spec correspondence and prohibition audit | not started | pending | pending | pending | Requires task 19 commit. Audit task. |
| 21. Bilingual documentation sync audit | not started | pending | pending | pending | Requires task 20 commit. Docs audit task. |
| 22. Module-boundary refactor gate | not started | pending | pending | pending | Requires task 21 commit. Audit or move-only task. |
| Closeout. Crate exit report and quality review | not started | pending | pending | pending | Requires task 22 commit, all hard gates passing, and read-only quality review score >= 90/100. |

## Task 0 Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-kernel autonomous crate development from the completed task 0
crate-plan commit. Before starting task 1, verify a clean worktree, confirm the
task 0 commit exists in git log, and re-read
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/task_ledger.md, doc/design/mizar-kernel/en/todo.md,
doc/design/internal/en/07.crate_module_layout.md,
doc/design/architecture/en/08.reasoning_boundary.md,
doc/design/architecture/en/15.kernel_certificate_format.md, and
doc/design/architecture/en/19.failure_semantics.md. Implement task 1 only: add
the mizar-kernel workspace member, minimal crate manifest, crate-root trust
statement, and trusted-baseline lint-policy guard. Keep production dependencies
limited to mizar-session and mizar-core, forbid unsafe code, and do not expose
semantic modules until paired module specs exist. Run cargo fmt --check,
cargo test -p mizar-kernel, cargo clippy -p mizar-kernel --all-targets
--all-features -- -D warnings, git diff --check, and git diff --cached --check
after explicit path staging. Use review-only agents for the required AGENTS.md
review phases.
```

Rationale: task 1 creates the trusted crate boundary and dependency guard that
all later kernel work relies on. Keep `xhigh` because dependency discipline,
trusted lint policy, and no-search/no-ATP boundaries are soundness-critical.
Lower reasoning is appropriate only for typo-only documentation cleanup; raise
only if repository metadata or contradictory specifications block the scaffold.
