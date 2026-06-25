# Task Ledger: mizar-kernel

> 正本は英語です。英語版:
> [../en/task_ledger.md](../en/task_ledger.md)。

この ledger は `mizar-kernel` crate 自律作業の再開地点である。task を開始する
前に `git status`、`git log`、この表、[todo.md](./todo.md) を確認する。
task は commit が履歴に存在し、final review outcome、verification result、
deferred reason が分かるまで完了ではない。commit は自身の最終 hash を含め
られないため、self-hash は次 task 開始前に `git log` で確認し、後続の
bookkeeping commit または closeout task で反映する。

| Task | Status | Commit | Reviews | Verification | Deferred / notes |
|---|---|---|---|---|---|
| 0. Crate plan | ready to commit | pending self-hash | Spec/doc review: low pending-status finding fixed; final re-review no findings. Test sufficiency review: medium `--all-features` and conditional cross-crate verification findings fixed; final re-review no findings. Full implementation review: high sequencing and medium cluster-gate/status findings fixed; final re-review no findings. Source/doc consistency review: medium internal-04 rejection-reason and low JA companion-link findings fixed; final re-review no findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Docs-only。paired crate plan と ledger を作成し、初期 `spec_gap`, `test_gap`, `design_drift`, `source_drift`, `external_dependency_gap`, `deferred`, `repo_metadata_conflict` 状態を分類し、kernel 禁止事項、trusted-baseline lint policy、strict linear task sequencing、internal-04 rejection reason coverage、cluster trace external-readiness gate を記録する。crate source は作らない。 |
| 1. Crate scaffold and trusted-baseline lint policy | not started | pending | pending | pending | task 0 commit が `git log` に存在することが必要。 |
| 2. Spec: `clause.md` | not started | pending | pending | pending | task 1 commit が必要。Spec-only。 |
| 3. Implement clause representation | not started | pending | pending | pending | task 2 commit が必要。Rust source task。 |
| 4. Spec: `certificate_parser.md` | not started | pending | pending | pending | task 3 commit が必要。Semantic dependency: task 2 clause spec。Spec-only。 |
| 5. Implement certificate parsing and structural validation | not started | pending | pending | pending | task 4 commit が必要。Rust source task。 |
| 6. Spec: `rejection.md` | not started | pending | pending | pending | task 5 commit が必要。Semantic dependency: task 1 trusted baseline。Spec-only。 |
| 7. Implement rejection records | not started | pending | pending | pending | task 6 commit が必要。Semantic dependency: task 5 parser と task 6 rejection spec。Rust source task。 |
| 8. Spec: `resolution_trace.md` | not started | pending | pending | pending | task 7 commit が必要。Semantic dependency: task 4 certificate spec。Spec-only。 |
| 9. Implement resolution trace checker | not started | pending | pending | pending | task 8 commit が必要。Semantic dependency: task 7 rejection records。Rust source task。 |
| 10. Spec: `substitution_checker.md` | not started | pending | pending | pending | task 9 commit が必要。Semantic dependency: task 4 certificate spec。Spec-only。 |
| 11. Implement substitution checking | not started | pending | pending | pending | task 10 commit が必要。Semantic dependency: task 7 rejection records。Rust source task。 |
| 12. Implement alpha-conversion and free-variable checks | not started | pending | pending | pending | task 11 commit が必要。Rust source task。 |
| 13. Spec: `checker.md` | not started | pending | pending | pending | task 12 commit が必要。Semantic dependencies: task 6 rejection spec、task 8 resolution spec、task 10 substitution spec。Spec-only。 |
| 14. Implement imported-fact checking | not started | pending | pending | pending | task 13 commit が必要。Rust source task。 |
| 15. Implement cluster trace replay | not started | pending | pending | pending | task 14 commit が必要。Semantic dependency: task 13 checker spec と external `mizar-checker` cluster trace payload readiness review または deferred record。Rust source task。 |
| 16. Kernel check service and deterministic batch ordering | not started | pending | pending | pending | task 15 commit が必要。Semantic dependencies: task 9 resolution checker、task 12 substitution checker、task 14 imported-fact checking、task 15 cluster replay。Rust source task。 |
| 17. Soundness fail-test corpus | not started | pending | pending | pending | task 16 commit が必要。Test/audit task。source-derived corpus runner gap は `external_dependency_gap` として残り得る。 |
| 18. Determinism and replay-cost suite | not started | pending | pending | pending | task 17 commit が必要。Semantic dependency: task 16 checker service。Test task。 |
| 19. Public-enum forward-compatibility policy | not started | pending | pending | pending | task 18 commit が必要。Semantic dependency: task 16 public API surface。Test/docs task。 |
| 20. Source/spec correspondence and prohibition audit | not started | pending | pending | pending | task 19 commit が必要。Audit task。 |
| 21. Bilingual documentation sync audit | not started | pending | pending | pending | task 20 commit が必要。Docs audit task。 |
| 22. Module-boundary refactor gate | not started | pending | pending | pending | task 21 commit が必要。Audit または move-only task。 |
| Closeout. Crate exit report and quality review | not started | pending | pending | pending | task 22 commit、全 hard gate pass、read-only quality review score >= 90/100 が必要。 |

## Task 0 handoff

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

Rationale: task 1 は、後続の kernel 作業が依存する trusted crate boundary と
dependency guard を作る。dependency discipline、trusted lint policy、
no-search/no-ATP boundary は soundness-critical なので `xhigh` を維持する。
typo-only documentation cleanup だけなら低い reasoning でもよい。repository
metadata や矛盾仕様が scaffold を block する場合だけ上げる。
