# task ledger: mizar-core

> 正本は英語です。英語版:
> [../en/task_ledger.md](../en/task_ledger.md)。

この ledger は `mizar-core` 自律 crate 作業の再開点である。task を始める前に
`git status`、`git log`、本表を確認する。task は commit hash、最終 review 結果、
verification 結果、deferred 理由が揃って初めて完了とする。

| task | status | commit | reviews | verification | deferred / notes |
|---|---|---|---|---|---|
| 0. Crate plan | ready to commit | commit 後に task handoff で記録する。この行を作る commit 自身に self-hash は埋め込めない | Spec/doc review: blocking/high/medium finding なし。Test sufficiency review: staged-check gap の medium を修正し、再 review で finding なし。Full implementation review: blocking/high/medium finding なし。Source/doc consistency review: blocking/high/medium finding なし。 | stage 前の `git diff --check` 通過。明示 path stage 後の `git diff --cached --check` 通過。 | docs-only。external gap は `00.crate_plan.md` で分類済み。task-local deferred なし。 |
| 1. Crate scaffold and lint-policy guard | not started | pending | pending | pending | task 0 commit hash が先に必要。 |
| 2. Spec: `core_ir.md` | not started | pending | pending | pending |  |
| 3. Implement `core_ir` data shapes | not started | pending | pending | pending |  |
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
| Closeout. Crate exit report and quality review | not started | pending | pending | pending | すべての task 完了と quality review score >= 90 が必要。 |
