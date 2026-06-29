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
| 1. Crate scaffold and lint-policy guard | complete | pending self-hash | 初回 review は stale ledger/handoff finalization wording のみを指摘した。scaffold、dependency boundary、lockfile、crate root、lint guard には finding なし。更新後の focused re-review は scaffold / handoff finding なし。Test sufficiency review は no findings。 | `cargo test -p mizar-proof` passed; `cargo clippy -p mizar-proof --all-targets -- -D warnings` passed; `cargo fmt --check` passed; `git diff --check` passed; task-1 path の明示 staging 後に `git diff --cached --check` passed。 | workspace crate、scaffold-only crate root、dependency manifest、lint-policy guard を追加する。 |

## Current Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-proof autonomous crate development with task 2 after the task-1
commit exists. First verify a clean worktree and confirm the task-1 commit in
HEAD history. Then write the paired English/Japanese `policy.md` module spec
only: verifier policy settings, `CandidatePolicyClass`, externally attested
admission rules, `require_kernel_certificates`, build-mode open-obligation
allowances, policy fingerprint surface, kernel-check scheduling policy, and the
rule that policy outcomes are distinct from trusted proof status. Do not add
Rust policy evaluator behavior, winner selection, status projection, witness
staging, cache lookup, artifact commit, ATP execution, SAT solving, proof
search, premise selection, substitution invention, or placeholder downstream
integration in task 2.
```

Rationale: task 2 は spec-only だが、以降すべての実装 task が使う trust と policy の
語彙を固定する。policy surface は proof reuse、external evidence、open obligation、
kernel certificate requirement に影響するため `xhigh` を維持する。typo-only
documentation sync だけなら lower reasoning でよい。architecture/internal spec が互いに
矛盾する場合だけ上げる。
