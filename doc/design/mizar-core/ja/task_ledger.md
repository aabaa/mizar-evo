# task ledger: mizar-core

> 正本は英語です。英語版:
> [../en/task_ledger.md](../en/task_ledger.md)。

この ledger は `mizar-core` 自律 crate 作業の再開点である。task を始める前に
`git status`、`git log`、本表を確認する。task は commit hash、最終 review 結果、
verification 結果、deferred 理由が揃って初めて完了とする。

| task | status | commit | reviews | verification | deferred / notes |
|---|---|---|---|---|---|
| 0. Crate plan | ready to commit | commit 後に task handoff で記録する。この行を作る commit 自身に self-hash は埋め込めない | Spec/doc review: blocking/high/medium finding なし。Test sufficiency review: staged-check gap の medium を修正し、再 review で finding なし。Full implementation review: blocking/high/medium finding なし。Source/doc consistency review: blocking/high/medium finding なし。 | stage 前の `git diff --check` 通過。明示 path stage 後の `git diff --cached --check` 通過。 | docs-only。external gap は `00.crate_plan.md` で分類済み。task-local deferred なし。 |
| 1. Crate scaffold and lint-policy guard | ready to commit | commit 後に task handoff で記録する。この行を更新する commit 自身に self-hash は埋め込めない | Spec/doc review: lint boundary の medium を修正し、再 review で finding なし。Test sufficiency review: alias/API surface の medium を修正し、再 review で finding なし。Full implementation review: blocking/high/medium finding なし。Source/doc consistency review: blocking/high/medium finding なし。 | `cargo fmt --check`; `cargo test -p mizar-core`; `cargo clippy -p mizar-core --all-targets -- -D warnings`; stage 前の `git diff --check`; 明示 path stage 後の `git diff --cached --check`。 | scaffold-only。workspace member、lockfile entry、最小 crate、lint guard を追加。task-local deferred なし。 |
| 2. Spec: `core_ir.md` | ready to commit | commit 後に task handoff で記録する。この行を更新する commit 自身に self-hash は埋め込めない | Spec/doc review: seed/statement/diagnostic/choice/comprehension/source-map の medium を修正し、再 review で finding なし。Test sufficiency review: blocking/high/medium finding なし。Full implementation review: medium を spec/doc 再 review で解消。Source/doc consistency review: blocking/high/medium finding なし。 | stage 前の `git diff --check`; 明示 path stage 後の `git diff --cached --check`。 | spec-only。Rust test は task 3 に deferred。source-derived `.miz` core fixture は checker payload extraction と mizar-test stage support まで deferred。 |
| 3. Implement `core_ir` data shapes | ready to commit | commit 後に task handoff で記録する。この行を更新する commit 自身に self-hash は埋め込めない | Spec/doc review: stable-choice/proof-status/source validation の medium を修正。owner diagnostic / task status doc の low も修正。最終再 review で blocking/high/medium finding なし。Test sufficiency review: invalid reference / seed / debug / error coverage の medium を修正。diagnostic/provenance tie-break coverage の medium と source-map / owner / error mismatch coverage の low を修正し、最終再 review で finding なし。Full implementation review: algorithm owner / generated origin / provenance の medium を修正し、crate doc / source sort の low も修正。Source/doc consistency review: proof-node/provenance/label/status doc の medium を修正し、最終再 review で finding なし。 | `cargo fmt --check`; `cargo test -p mizar-core`; `cargo clippy -p mizar-core --all-targets -- -D warnings`; stage check は未実施。 | `core_ir` data shape、validation、deterministic debug rendering、lint guard 拡張を実装。source-derived `.miz` core fixture は checker payload extraction と mizar-test stage support まで deferred。task-local external dependency の仮実装なし。 |
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
