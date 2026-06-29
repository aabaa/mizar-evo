# mizar-proof Task Ledger

> 正本は英語です。英語版:
> [../en/task_ledger.md](../en/task_ledger.md)。

この ledger は autonomous crate-development task ごとに 1 commit を記録する。
task commit は自分自身の hash を含められないため、self-hash は ledger を編集する
次の番号付き task、または final closeout task でだけ backfill する。番号のない
追加 bookkeeping commit は許可しない。

| Task | Status | Commit | Review result | Verification | Summary |
|---|---|---|---|---|---|
| 0. Crate plan | complete | pending self-hash | 初回 review は ledger/TODO status drift、広すぎる ATP gap 分類、`discharged_builtin` artifact-witness gap の欠落を指摘し、docs を同期した。focused re-review は no findings。Test sufficiency review も docs-only verification で no findings。 | `git diff --check` は passed。task-0 path の明示 staging 後に `git diff --cached --check` は passed。 | paired crate plan と task ledger を追加し、kickoff gap を分類する。source は scaffold しない。 |

## Current Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-proof autonomous crate development with task 1 after the task-0
commit exists. First verify a clean worktree and confirm the task-0 commit in
HEAD history. Then scaffold `crates/mizar-proof` as a workspace member with
dependencies on `mizar-session`, `mizar-kernel`, `mizar-vc`, `mizar-atp`, and
`mizar-artifact`; add a lint-policy guard mirroring the repository pattern.
Do not implement policy evaluator behavior, winner selection, status
projection, witness staging, cache lookup, artifact commit, ATP execution, SAT
solving, proof search, premise selection, substitution invention, or any
placeholder downstream integration in task 1.
```

Rationale: task 1 は狭い scaffold/lint step だが、proof trust boundary 上にあるため
one-task-one-commit discipline を維持する。typo-only documentation follow-up だけなら
lower reasoning でよい。repository metadata や workspace dependency policy conflict が
scaffold を block する場合だけ上げる。
