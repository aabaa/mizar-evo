# Task Ledger: mizar-vc

> 正本は英語です。英語版:
> [../en/task_ledger.md](../en/task_ledger.md)。

この ledger は自律 `mizar-vc` crate work の再開地点である。task を開始する前に
`git status`、`git log`、この表、[todo.md](./todo.md) を確認する。task は、
commit が履歴に存在し、最終 review outcome、verification result、deferred
理由が判明して初めて完了である。commit は自分自身の最終 hash を同じ commit
内に含められないため、自己 hash は次 task 開始前に `git log` で確認し、後続の
記録ポイントまたは closeout task で backfill する。

| Task | 状態 | Commit | Reviews | Verification | Deferred / notes |
|---|---|---|---|---|---|
| 0. Crate plan | complete | `9697036b0f012cfc578a015dc5a0d6f37bf85143` | Spec/doc review: medium registration-correctness と derived-doc authority findings を修正し、final re-review は no blocking/high/medium findings。Test sufficiency review: no findings。Full implementation review: low future-link と stale task-scope findings を修正し、final re-review は no blocking/high/medium findings。Source/doc consistency review: medium task-15 と conditional-verification findings を修正し、final re-review は no blocking/high/medium findings。 | `git diff --check` は明示 staging 前に passed; `git diff --cached --check` は明示 path staging 後に passed。 | Docs-only。初期 `spec_gap`、`test_gap`、`design_drift`、`source_drift`、`external_dependency_gap`、`deferred` rows を `00.crate_plan.md` に分類し、現在の runner / verification gap と registration-style correctness seed scope に合わせて todo wording を同期する。crate source は作らない。 |
| 1. Crate scaffold and lint-policy guard | complete | `adfff1cbc3ebce9db13e73d4d29bfd9b1ac1971d` | Spec/doc review: no blocking/high/medium/low findings。Test sufficiency review: low private-scope guard finding を修正し、final re-review は no findings。Full implementation review: guard 強化後 no findings。Source/doc consistency review: no blocking/high/medium/low findings。 | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; 明示 path staging 後の `git diff --cached --check` passed。 | Scaffold-only。workspace member、lockfile entry、minimal crate manifest、documentation-only `src/lib.rs`、lint guard を追加する。semantic VC API、module source file、`.miz` fixture、expectation、`doc/spec`、module spec は変更しない。 |
| 2. Spec: `vc_ir.md` | ready to commit | pending self-hash; commit 後に `git log` で確認 | Spec/doc review: medium seed-accounting、generated-goal、status-name、expansion-index findings を修正し、final re-review は ledger-status bookkeeping のみ残り、それも修正済み。Test sufficiency review: medium task-8 seed-bijection wording を修正し、final re-review は no blocking/high/medium findings。Full implementation review: medium status-name、proof-hint、algorithm-subkind、ledger-status findings を修正。Source/doc consistency review: medium todo seed-accounting drift を修正し、final re-review は no blocking/high/medium findings。 | `git diff --check` passed; 明示 path staging 後の `git diff --cached --check` passed。 | Spec-only。英語/日本語 `vc_ir.md` を追加し、task 2 と task 8 の todo wording を seed accounting / explicit concrete cardinality に同期する。Rust source、`.miz` fixture、expectation、`doc/spec`、traceability metadata は変更しない。proof-verification runner、ATP/kernel/proof/cache consumer、source-derived payload の external gaps は deferred のまま。 |
| 3. Implement `vc_ir` data shapes | not started | pending | pending | pending | Rust source task。 |
| 4. Obligation-seed intake | not started | pending | pending | pending | Rust source task。 |
| 5. Spec: `generator.md` | not started | pending | pending | pending | Spec-only task。explicit payload が存在する場合の registration-style correctness seed scope を含む。 |
| 6. Theorem, definition, and registration-style correctness VCs | not started | pending | pending | pending | Rust source task。利用不能な explicit registration payload は external/deferred に保つ。 |
| 7. Algorithm VCs | not started | pending | pending | pending | Rust source task。 |
| 8. Normalization, classification, and `VcId` assignment | not started | pending | pending | pending | Rust source task。 |
| 9. Status and policy model | not started | pending | pending | pending | Rust source task。 |
| 10. Spec: `discharge.md` | not started | pending | pending | pending | Spec-only task。 |
| 11. Deterministic discharge engine | not started | pending | pending | pending | Rust source task。 |
| 12. Discharge evidence and explanations | not started | pending | pending | pending | Rust source task。 |
| 13. Spec: `dependency_slice.md` | not started | pending | pending | pending | Spec-only task。 |
| 14. Dependency-slice computation | not started | pending | pending | pending | Rust source task。 |
| 15. Corpus runner record for `proof_verification` | not started | pending | pending | pending | その時点で runner/extraction seam が存在しなければ deferred-record task。 |
| 16. Determinism suite | not started | pending | pending | pending | Test task。source fix は spec-backed の場合だけ。 |
| 17. Public-enum forward-compatibility policy | not started | pending | pending | pending | Test/docs task。 |
| 18. Source/spec correspondence audit | not started | pending | pending | pending | Audit task。 |
| 19. Bilingual documentation sync audit | not started | pending | pending | pending | Audit/docs task。 |
| 20. Obligation anchors and cross-edit reuse identity | not started | pending | pending | pending | architecture-22 identity の Rust source task。 |
| 21. Architecture-22 follow-up audit | not started | pending | pending | pending | Audit task。 |
| 22. Module-boundary refactor gate | not started | pending | pending | pending | Audit task。source move は必要な場合のみ。 |
| Closeout. Crate exit report and quality review | not started | pending | pending | pending | hard gates が通り read-only quality score >= 90 の場合だけ完了。 |

## Task 0 Handoff

Recommended reasoning: `xhigh`。

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

Rationale: task 1 は workspace と Rust crate scaffold を変更するため、
manifest、lint policy、one-task-one-commit constraints を保つには `xhigh` が
適している。純粋に機械的な ledger typo 修正だけなら lower reasoning でもよい。
dependencies、lint policy、workspace membership に触れるなら `xhigh` を保つ。

## Task 1 Handoff

Recommended reasoning: `xhigh`。

Prompt:

```text
Continue mizar-vc autonomous crate development from the completed task 1
scaffold commit. Before starting task 2, verify a clean worktree, confirm the
task 1 commit exists in git log, and re-read
doc/design/mizar-vc/en/00.crate_plan.md, task_ledger.md, and todo.md. Implement
task 2 only: write the English/Japanese vc_ir.md module spec with no Rust source
changes. Cover VcId, VcKind, LocalContext, symbolic PremiseRefs, goal formula,
ProofHint, VC status including NeedsAtp and policy statuses, the seed-intake
mapping rule, and the architecture-22 ObligationAnchor contract. Preserve the
task-0 design_drift classification around active seed intake versus skipped or
expanded obligations. Run git diff --check and git diff --cached --check after
explicit path staging, and use review-only agents for the required AGENTS.md
review phases.
```

Rationale: task 2 は semantic API が現れる前に central VC IR と anchor contract
を定義する。proof / identity boundary なので `xhigh` を保つ。typo-only の
documentation cleanup だけなら lower reasoning でもよい。

## Task 2 Handoff

Recommended reasoning: `xhigh`。

Prompt:

```text
Continue mizar-vc autonomous crate development from the completed task 2
vc_ir.md spec commit. Before starting task 3, verify a clean worktree, confirm
the task 2 commit exists in git log, and re-read
doc/design/mizar-vc/en/vc_ir.md, 00.crate_plan.md, task_ledger.md, and todo.md.
Implement task 3 only: add src/vc_ir.rs and expose the vc_ir module from
src/lib.rs according to vc_ir.md. Implement data shapes, validation, and
deterministic debug rendering only; do not implement seed intake, generator
logic, discharge, dependency slices, ATP translation, cache/proof reuse, or
kernel acceptance. Add Rust tests for construction, symbolic premise refs,
status/context preservation, generated formula table behavior, incomplete
anchor markers, and rendering stability. Run cargo fmt --check,
cargo test -p mizar-vc, cargo clippy -p mizar-vc --all-targets -- -D warnings,
git diff --check, and git diff --cached --check after explicit path staging.
Use review-only agents for the required AGENTS.md review phases.
```

Rationale: task 3 は `mizar-vc` 最初の semantic Rust surface であるため、
proof-boundary、identity、downstream ownership 禁止を保つには `xhigh` が適している。
documentation-only typo fix だけなら lower reasoning でもよい。
