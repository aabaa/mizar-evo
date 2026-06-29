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
| 2. Spec: `policy.md` | complete | pending self-hash | 初回 review は built-in discharge scheduling wording、`AssumedByPolicy`、artifact witness gap、fingerprint TODO、ledger finalization の問題を指摘した。修正後、focused spec、test-sufficiency、full、source-doc review は no findings。 | `git diff --check` passed。task-2 path の明示 staging 後に `git diff --cached --check` passed。 | verifier policy setting、candidate class、external evidence、open obligation、policy assumption、fingerprint surface、scheduling、early stop、diagnostics、trust boundary を覆う paired policy spec を追加する。 |

## Current Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-proof autonomous crate development with task 3 after the task-2
commit exists. First verify a clean worktree and confirm the task-2 commit in
HEAD history. Then implement `ProofPolicyEvaluator` in `src/policy.rs` from
the paired `policy.md` specs: verifier policy settings, candidate
classification, `can_schedule_kernel_check`, and the `PolicyFingerprint`
projection. Add focused classification and fingerprint tests. Do not implement
winner selection, artifact status projection, witness staging, cache lookup,
artifact commit, ATP execution, SAT solving, proof search, premise selection,
substitution invention, externally attested winner behavior beyond the
task-3 base classifier, or placeholder downstream integration.
```

Rationale: task 3 は proof-policy crate の最初の Rust behavior task であり、再利用される
policy data model を定義する。trusted kernel acceptance と policy evidence / cache
metadata をぼかす誤りを避けるため `xhigh` を維持する。comment-only cleanup だけなら
lower reasoning でよい。既存 upstream API では spec を表せない場合だけ上げ、追加 gap
record を作る。
