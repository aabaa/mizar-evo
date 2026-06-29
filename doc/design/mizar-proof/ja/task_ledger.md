# mizar-proof Task Ledger

> 正本は英語です。英語版:
> [../en/task_ledger.md](../en/task_ledger.md)。

この ledger は autonomous crate-development task ごとに 1 commit を記録する。
task commit は自分自身の hash を含められないため、self-hash は ledger を編集する
次の番号付き task、または final closeout task でだけ backfill する。番号のない
追加 bookkeeping commit は許可しない。

| Task | Status | Commit | Review result | Verification | Summary |
|---|---|---|---|---|---|
| 0. Crate plan | complete | `a1f6d1dba4ecc46aa2d434c8da8ae0279a2c23ec` | 初回 review は ledger/TODO status drift、広すぎる ATP gap 分類、`discharged_builtin` artifact-witness gap の欠落を指摘し、docs を同期した。focused re-review は no findings。Test sufficiency review も docs-only verification で no findings。 | `git diff --check` は passed。task-0 path の明示 staging 後に `git diff --cached --check` は passed。 | paired crate plan と task ledger を追加し、kickoff gap を分類する。source は scaffold しない。 |
| 1. Crate scaffold and lint-policy guard | complete | `bc7d33efd8b7427fa026807e729642219f62f809` | 初回 review は stale ledger/handoff finalization wording のみを指摘した。scaffold、dependency boundary、lockfile、crate root、lint guard には finding なし。更新後の focused re-review は scaffold / handoff finding なし。Test sufficiency review は no findings。 | `cargo test -p mizar-proof` passed; `cargo clippy -p mizar-proof --all-targets -- -D warnings` passed; `cargo fmt --check` passed; `git diff --check` passed; task-1 path の明示 staging 後に `git diff --cached --check` passed。 | workspace crate、scaffold-only crate root、dependency manifest、lint-policy guard を追加する。 |
| 2. Spec: `policy.md` | complete | `193598cd5db448b17c5cddb721a5e22010b241b6` | 初回 review は built-in discharge scheduling wording、`AssumedByPolicy`、artifact witness gap、fingerprint TODO、ledger finalization の問題を指摘した。修正後、focused spec、test-sufficiency、full、source-doc review は no findings。 | `git diff --check` passed。task-2 path の明示 staging 後に `git diff --cached --check` passed。 | verifier policy setting、candidate class、external evidence、open obligation、policy assumption、fingerprint surface、scheduling、early stop、diagnostics、trust boundary を覆う paired policy spec を追加する。 |
| 3. Policy evaluator | complete | pending self-hash | 初回 review は synthesized accepted-kernel input risk、evidence-kind test coverage 不足、negative scheduling test gap、kernel/policy rejection layering drift、task-4 external-admission overreach を指摘した。修正後、focused spec、test-sufficiency、full、source-doc review は no findings。 | `cargo test -p mizar-proof` passed; `cargo clippy -p mizar-proof --all-targets -- -D warnings` passed; `cargo fmt --check` passed; `git diff --check` passed; task-3 path の明示 staging 後に `git diff --cached --check` passed。 | policy evaluator、normalized kernel-origin wrapper、candidate class、schedulability check、kernel rejection separation、deterministic policy fingerprint を実装し、policy module 用に lint guard を更新する。 |

## Current Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-proof autonomous crate development with task 4 after the task-3
commit exists. First verify a clean worktree and confirm the task-3 commit in
HEAD history. Then implement externally attested evidence handling on top of
the task-3 base classifier: admission and labeling by policy profile,
recordable development evidence, never winning under
`require_kernel_certificates`, never producing trusted `used_axioms`, and
stable policy rejection diagnostics. Do not implement winner selection,
artifact status projection, witness staging, cache lookup, artifact commit, ATP
execution, SAT solving, proof search, premise selection, substitution
invention, or placeholder downstream integration.
```

Rationale: task 4 は trust boundary を保ちながら external-evidence admission matrix を
完成させる。external または policy-tainted evidence が trusted acceptance を満たす誤りを
避けるため `xhigh` を維持する。wording-only cleanup だけなら lower reasoning でよい。
upstream API が新しい明示 gap classification を必要とする場合だけ上げる。
