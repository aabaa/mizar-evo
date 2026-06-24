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
| 2. Spec: `vc_ir.md` | complete | `ac778b008be75ea21eda4d2e69c7713a88b0d4ea` | Spec/doc review: medium seed-accounting、generated-goal、status-name、expansion-index findings を修正し、final re-review は ledger-status bookkeeping のみ残り、それも修正済み。Test sufficiency review: medium task-8 seed-bijection wording を修正し、final re-review は no blocking/high/medium findings。Full implementation review: medium status-name、proof-hint、algorithm-subkind、ledger-status findings を修正。Source/doc consistency review: medium todo seed-accounting drift を修正し、final re-review は no blocking/high/medium findings。 | `git diff --check` passed; 明示 path staging 後の `git diff --cached --check` passed。 | Spec-only。英語/日本語 `vc_ir.md` を追加し、task 2 と task 8 の todo wording を seed accounting / explicit concrete cardinality に同期する。Rust source、`.miz` fixture、expectation、`doc/spec`、traceability metadata は変更しない。proof-verification runner、ATP/kernel/proof/cache consumer、source-derived payload の external gaps は deferred のまま。 |
| 3. Implement `vc_ir` data shapes | complete | `c32d767368ef9d16fdcf92620c2b2afecb13fc9d` | Spec/doc review: medium `ModuleId`、expanded-index/rendering、incomplete-anchor、quantified-binder findings を修正し、final re-review は no blocking/high/medium findings。Test sufficiency review: medium rendering、seed-accounting、status/context coverage findings を修正し、final re-review は no blocking/high/medium findings。Full implementation review: medium seed-mapping、nested-reference、anchor-completeness、quantified-binder findings を修正し、final re-review は no blocking/high/medium findings。Source/doc consistency review: medium `PolicyOpen` no-VC mismatch を修正し、low module-link finding も修正; final re-review は no blocking/high/medium findings。 | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; 明示 path staging 後の `git diff --cached --check` passed。 | Rust source task。`src/vc_ir.rs` を追加し、`pub mod vc_ir;` だけを expose し、spec-backed module 用に lint guard を更新し、validation と deterministic debug rendering tests を追加する。seed intake、generator logic、status transition、discharge、dependency slice、ATP translation、proof/cache reuse、kernel acceptance、`.miz` fixture、expectation、`doc/spec`、traceability metadata は deferred/out of scope のまま。 |
| 4. Obligation-seed intake | complete | `ba20db550cf92979bdb8809e9f64fbe5cd193c1b` | Spec/doc review: medium missing source-map documentation finding を修正し、final re-review は no blocking/high/medium findings。Test sufficiency review: medium origin-preservation coverage finding を修正し、final re-review は no blocking/high/medium findings。Full implementation review: follow-up 後 no blocking/high/medium findings。Source/doc consistency review: follow-up 後 no blocking/high/medium findings。 | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; 明示 path staging 後の `git diff --cached --check` passed。 | Rust source task。`ObligationSeedHandoff` 上の pre-`VcId` `SeedIntakeTable` を追加し、handoff order と origin を保持し、duplicate `(canonical_key, origin)` row と missing source-map entry を拒否し、skipped/deferred/error/missing-goal row を visible no-VC mapping として表す。concrete VC generation、generator normalization、final `VcId` assignment、discharge、dependency slice、ATP translation、proof/cache reuse、kernel acceptance、`.miz` fixture、expectation、`doc/spec`、traceability metadata は deferred/out of scope のまま。 |
| 5. Spec: `generator.md` | complete | `e324beab799f972dcf78e897b163aebd9414725e` | Spec/doc review: high generated-core ownership、medium Pick non-emptiness、medium module-table findings を修正し、verification/staging 後の final re-review は no blocking/high/medium findings。Test sufficiency review: medium theorem-status、sethood/non-emptiness、call/return coverage findings を修正し、final re-review は no blocking/high/medium findings。Full implementation review: medium module-table finding を修正し、final re-review は no blocking/high/medium findings。Source/doc consistency review: medium module-table finding を修正し、final re-review は no blocking/high/medium findings。 | `git diff --check` passed; 明示 path staging 後の `git diff --cached --check` passed。 | Spec-only。英語/日本語 `generator.md` を追加し、local-context construction、theorem/definition generation、generated core obligation、explicit registration-style correctness payload handling、Pick non-emptiness を含む algorithm VC families、controlled unfolding、task-8 normalization/classification handoff を記録する。Rust source、`.miz` fixture、expectation、`doc/spec`、traceability metadata、proof-verification runner activation、利用不能な dedicated registration/redefinition/reduction payload は deferred/out of scope のまま。 |
| 6. Theorem, definition, generated core, and registration-style correctness VCs | complete | `b5634eb878b39558b981bcbba972e8b36c3203c9` | Spec/doc review: high registration-style boundary と medium theorem-status gap findings を修正し、staged-verification 記録更新後の final re-review は no blocking/high/medium findings。Test sufficiency review: medium definition-family、theorem-status、registration-negative、no-candidate、determinism findings を修正し、final re-review は no blocking/high/medium findings。Full implementation review: high stale/partial intake findings と medium unfold、context-sort、schema、terminal-goal、diagnostic-wording findings を修正し、final re-review は no findings。Source/doc consistency review: high/medium marker、schema、sort-key、unfold、lint-message、GEN-G005 wording findings を修正し、final re-review は no findings。 | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; 明示 path staging 後の `git diff --cached --check` passed。 | Rust source task。task 5 hash を backfill し、`src/generator.rs` を追加し、`pub mod generator` を expose し、task-6 seed family 向けの pre-normalized `CoreGenerationCandidateSet` を実装し、handoff に対する full seed-intake table equality を強制し、registration-style、theorem-status、terminal-proof、unfold behavior の explicit `CoreProvenance` marker を保持し、local context を canonicalize し、lint guard を拡張する。後続 algorithm VC、final `VcId` assignment、status transition、discharge、dependency slice、ATP/kernel/proof/cache/corpus integration、未提供の dedicated registration/redefinition/reduction payload field は external/deferred に保つ。 |
| 7. Algorithm VCs | complete | `a15a2ee3e21974727fab2f8406b2e161b3f3c2f7` | Spec/doc review: high seed-intake conflict と medium broad-scope wording findings を修正し、final re-review は no blocking/high/medium findings。Test sufficiency review: medium AlgorithmAssertion、partial/ghost、unavailable-family、metadata、determinism findings を修正し、final re-review は no blocking/high/medium findings。Full implementation review: medium flow/algorithm mismatch と site-membership findings を修正し、final re-review は no blocking/high/medium findings。Source/doc consistency review: medium eligible-intake、site-validation、planned-test drift findings を修正し、final re-review は no findings。 | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; 明示 path staging 後の `git diff --cached --check` passed。 | Rust source task。task 6 hash を backfill する。requires、ensures、assertions、supported loop-invariant entry/preservation/break/continue site 向けの goal-bearing flow-derived algorithm candidate を追加する。eligible deferred `FlowDerived` `AlgorithmContract` row は seed status を保持したまま candidate-eligible になるよう seed intake を更新する。flow id、algorithm id、site table membership、goal、placement metadata を検証する。missing site/data、term-only termination、partial termination、ghost erasure、unavailable algorithm family、incomplete loop metadata は visible no-candidate/deferred record として記録する。`ControlFlowIr` fixture の `SymbolId` 構築だけのため test-only `mizar-resolve` dev-dependency を追加する。 |
| 8. Normalization, classification, and `VcId` assignment | complete | `6b4a7ef661886d6339f8ac24e21ad68e9f7ac302` | Spec/doc review: medium stable-kind-order と task-gap-classification findings を修正し、final re-review は no blocking/high/medium findings。Test sufficiency review: medium full-rank coverage と status-boundary findings を修正し、final re-review は no blocking/high/medium findings。Full implementation review: no findings; test 修正後の final re-review も no findings。Source/doc consistency review: no findings; test 修正後の final re-review も no findings。 | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; 明示 path staging 後の `git diff --cached --check` passed。 | Rust source task。`VcNormalizationInput` と candidate-set から final `VcSet` への normalization を追加し、documented `VcKind` classification rank、candidate sort key、handoff id の順で snapshot-local dense `VcId` を割り当て、sorted final no-VC/one-VC seed accounting を構築し、deferred flow seed status と既存 VC status を保持し、normalization provenance だけを追加し、duplicate candidate sort key と duplicate seed ownership を拒否する。expanded mapping は `VcSet` validation 経由の validation-only に保つ。Status transition、discharge、dependency slice、ATP translation、kernel/proof/cache/corpus integration、`.miz` fixture、expectation、`doc/spec`、traceability metadata は deferred/out of scope のまま。 |
| 9. Status and policy model | complete | `30c8e303c2c88d70a0dd69295ec001280471519a` | Spec/doc review: medium todo discharge-scope finding を修正し、final re-review は no blocking/high/medium findings。Test sufficiency review: medium multi-VC default/override、policy-action provenance、invalid generated-marker findings を修正し、final re-review は no blocking/high/medium findings。Full implementation review: no findings; test 修正後の final re-review も no findings。Source/doc consistency review: no findings; test 修正後の final re-review も no findings。 | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; 明示 path staging 後の `git diff --cached --check` passed。 | Rust source task。immutable deterministic status-policy projection 用に `VcStatusPlan`、`VcStatusAction`、`VcStatusOverride`、`VcSet::try_with_status_plan` を追加する。preserve、`NeedsAtp`、`PolicyOpen`、`AssumedByPolicy` を support し、VC order、context、premise、proof hint、anchor、generated formula、seed accounting、ATP-bound obligation を保持し、実際の status change だけに `StatusPolicy` provenance を追加し、duplicate/unsorted/missing override を拒否し、invalid assumption marker は `VcSet` validation 経由で fail closed する。Discharge evidence、dependency slice、ATP translation、kernel/proof/cache/corpus integration、`.miz` fixture、expectation、`doc/spec`、new generator payload family は deferred/out of scope のまま。 |
| 10. Spec: `discharge.md` | ready to commit | pending self-hash; commit 後に `git log` で確認 | Spec/doc review: medium discharged-evidence unavailable-trace wording を修正し、final re-review は no blocking/high/medium findings。Test sufficiency review: medium positive discharge-class coverage finding を修正し、final re-review は no blocking/high/medium findings。Full implementation review: no findings。Source/doc consistency review: no findings; planned-test 修正後の final re-review も no findings。 | `git diff --check` passed; 明示 path staging 後の `git diff --cached --check` passed。 | Spec-only task。英語/日本語 `discharge.md` を追加し、deterministic pre-ATP discharge scope、fail-closed supported rule classes、limit model shape、evidence/explanation requirements、status interaction、no-erase ATP boundary、task-11/task-12 planned tests、`spec_gap` / `source_drift` / `test_gap` / `external_dependency_gap` / `deferred` classifications を記録する。Rust source、`.miz` fixture、expectation、`doc/spec`、traceability metadata、ATP/kernel/proof/cache/corpus integration、dependency slice、active runner support は変更しない。 |
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

## Task 3 Handoff

Recommended reasoning: `xhigh`。

Prompt:

```text
Continue mizar-vc autonomous crate development from the completed task 3
vc_ir data-shape commit. Before starting task 4, verify a clean worktree,
confirm the task 3 commit exists in git log, and re-read
doc/design/mizar-vc/en/vc_ir.md, 00.crate_plan.md, task_ledger.md, todo.md,
and crates/mizar-vc/src/vc_ir.rs. Implement task 4 only: consume
mizar_core::control_flow::ObligationSeedHandoff into a deterministic
seed-accounting table over the existing vc_ir data shapes. Preserve explicit
core/control-flow origins, reject duplicate handoff rows or duplicate canonical
seed ownership deterministically, represent skipped/deferred/error seeds with
visible no-VC mappings and reasons, and do not generate concrete VCs beyond the
data already present in eligible active seed rows. Do not implement generator
normalization, VcId assignment beyond deterministic table construction,
discharge, dependency slices, ATP translation, proof/cache reuse, kernel
acceptance, or active .miz proof_verification fixtures. Add Rust tests for
handoff order, duplicate rejection, no-VC reasons, active one-VC seed-accounting
rows where goals exist, and stable debug rendering. Run cargo fmt --check,
cargo test -p mizar-vc, cargo clippy -p mizar-vc --all-targets -- -D warnings,
git diff --check, and git diff --cached --check after explicit path staging.
Use review-only agents for the required AGENTS.md review phases.
```

Rationale: task 4 は `mizar-core` から `mizar-vc` への最初の handoff boundary
である。seed accounting は proof-obligation completeness boundary なので
`xhigh` を保つ。typo-only の documentation cleanup だけなら lower reasoning でもよい。

## Task 4 Handoff

Recommended reasoning: `xhigh`。

Prompt:

```text
Continue mizar-vc autonomous crate development from the completed task 4
seed-intake commit. Before starting task 5, verify a clean worktree, confirm
the task 4 commit exists in git log, and re-read
doc/design/mizar-vc/en/vc_ir.md, 00.crate_plan.md, task_ledger.md, todo.md,
and crates/mizar-vc/src/vc_ir.rs. Implement task 5 only: write the
English/Japanese generator.md module spec with no Rust source changes. Cover
local-context construction, theorem/definition VC generation, explicit
registration/redefinition/reduction correctness seeds when available,
algorithm VCs over ControlFlowIr, controlled definition unfolding, and
normalization/classification handoff to later tasks. Preserve the rule that
unavailable explicit registration payloads are external/deferred rather than
fabricated. Run git diff --check and git diff --cached --check after explicit
path staging. Use review-only agents for the required AGENTS.md review phases.
```

Rationale: task 5 は implementation task 6-8 の前に generation contract を定義する。
proof obligation、registration-style correctness boundary、algorithm-control-flow
VC category をまたぐため `xhigh` を保つ。typo-only の documentation cleanup だけなら
lower reasoning でもよい。

## Task 5 Handoff

Recommended reasoning: `xhigh`。

Prompt:

```text
Continue mizar-vc autonomous crate development from the completed task 5
generator.md spec commit. Before starting task 6, verify a clean worktree,
confirm the task 5 commit exists in git log, and re-read
doc/design/mizar-vc/en/generator.md, vc_ir.md, 00.crate_plan.md,
task_ledger.md, todo.md, and crates/mizar-vc/src/vc_ir.rs. Implement task 6
only: theorem, definition, generated core, and registration-style correctness
VC generation over explicit mizar-core payloads. Generate theorem/proof-step
terminal goals, definition correctness candidates, generated non-emptiness,
generated sethood, and Fraenkel membership axiom candidates, and preserve
registration/redefinition/reduction correctness only when explicit core/checker
payloads exist. Missing dedicated registration-style payloads must stay
DeferredExternal or visible no-VC records; do not fabricate them from
registration activation or source syntax. Do not implement algorithm VCs,
normalization/final VcId assignment, status transitions, discharge, dependency
slices, ATP translation, proof/cache reuse, kernel acceptance, or active .miz
proof_verification fixtures. Add focused Rust tests for local contexts,
symbolic citations, theorem status dependency preservation, generated core
obligations, definition correctness families, registration-style payload
presence/absence, proof hints, and local unfold requests. Run cargo fmt --check,
cargo test -p mizar-vc, cargo clippy -p mizar-vc --all-targets -- -D warnings,
git diff --check, and git diff --cached --check after explicit path staging.
Use review-only agents for the required AGENTS.md review phases.
```

Rationale: task 6 は最初の generator implementation slice であり、final `VcId`
assignment にはまだ進まないが proof obligation completeness に触れる。seed accounting、
registration-style correctness、generated core obligation boundary を保つため `xhigh` を
維持する。typo-only の documentation cleanup だけなら lower reasoning でもよい。

## Task 6 Handoff

Recommended reasoning: `xhigh`。

Prompt:

```text
Continue mizar-vc autonomous crate development from the completed task 6
core-generation-candidate commit. Before starting task 7, verify a clean
worktree, confirm the task 6 commit exists in git log, and re-read
doc/design/mizar-vc/en/generator.md, vc_ir.md, 00.crate_plan.md,
task_ledger.md, todo.md, crates/mizar-vc/src/generator.rs, and
crates/mizar-vc/src/vc_ir.rs. Implement task 7 only: algorithm VC generation
for explicit goal-bearing flow-derived mizar-core ControlFlowIr /
obligation-seed payloads. Generate candidates for requires, ensures,
assertions, and supported loop-invariant entry/preservation/break/continue
sites when `ControlFlowObligationSite`, `ControlFlowOutput`, and a goal
formula are present. Keep unavailable call, branch, match, range-loop,
collection-loop, term-only termination, partial-termination, Pick, and
ghost-erasure payload families as external_dependency_gap/deferred visible
no-candidate rows. Keep the task-6 CoreGenerationCandidateSet pre-normalized
and do not implement normalization/final VcId assignment, status transitions,
discharge, dependency slices, ATP translation, proof/cache reuse, kernel
acceptance, corpus runner activation, or missing external ControlFlowIr
payloads. Do not fabricate control-flow facts from labels or source text. Add
focused Rust tests for goal-bearing preconditions, postconditions, assertions,
loop invariant entry/preservation/break/continue classification, missing flow
site/data no-candidate records, term-only termination, ghost/Pick deferred
records, deterministic sorting, and handoff/intake mismatch rejection. Run
cargo fmt --check,
cargo test -p mizar-vc, cargo clippy -p mizar-vc --all-targets -- -D warnings,
git diff --check, and git diff --cached --check after explicit path staging.
Use review-only agents for the required AGENTS.md review phases.
```

Rationale: task 7 は generation の algorithm 側を追加し、task-6 の proof-obligation
accounting boundary に依存する。control-flow VC generation は範囲が広く proof gap を
静かに作り得るため `xhigh` を保つ。documentation-only typo fix だけなら lower
reasoning でもよい。

## Task 7 Handoff

Recommended reasoning: `xhigh`。

Prompt:

```text
Continue mizar-vc autonomous crate development from the completed task 7
algorithm-candidate commit. Before starting task 8, verify a clean worktree,
confirm the task 7 commit exists in git log, and re-read
doc/design/mizar-vc/en/vc_ir.md, generator.md, 00.crate_plan.md,
task_ledger.md, todo.md, crates/mizar-vc/src/vc_ir.rs, and
crates/mizar-vc/src/generator.rs. Implement task 8 only: normalize task-6/7
generation candidates, classify them into final canonical order, assign dense
within-snapshot `VcId`s, build final `VcSet`/seed accounting, and reject
duplicate candidate keys or incomplete seed ownership deterministically. Preserve
the task-7 boundary: do not add new algorithm payload families, discharge,
status transitions beyond the task-8 classification contract, dependency
slices, ATP translation, proof/cache reuse, kernel acceptance, or corpus runner
activation. Add Rust tests for deterministic `VcId` assignment across repeated
runs, complete no-VC/one-VC/expanded seed mapping, deferred flow-derived status
accounting, duplicate rejection, generated formula table references, stable
debug rendering, and incomplete-anchor preservation. Run cargo fmt --check,
cargo test -p mizar-vc, cargo clippy -p mizar-vc --all-targets -- -D warnings,
git diff --check, and git diff --cached --check after explicit path staging.
Use review-only agents for the required AGENTS.md review phases.
```

Rationale: task 8 は pre-normalized candidate を final `VcIr` に変換し、初めて `VcId`
を割り当てる task である。seed accounting と proof-obligation completeness のため
`xhigh` が適切である。typo-only documentation cleanup だけなら lower reasoning でもよい。

## Task 8 Handoff

Recommended reasoning: `xhigh`。

Prompt:

```text
Continue mizar-vc autonomous crate development from the completed task 8
normalization/VcId-assignment commit. Before starting task 9, verify a clean
worktree, confirm the task 8 commit exists in git log, and re-read
doc/design/mizar-vc/en/vc_ir.md, generator.md, 00.crate_plan.md,
task_ledger.md, todo.md, crates/mizar-vc/src/vc_ir.rs, and
crates/mizar-vc/src/generator.rs. Implement task 9 only: the VC status and
policy model over the final normalized `VcSet`. Add deterministic status
transition APIs or helpers that reflect verifier policy into `Open`,
`NeedsAtp`, policy-open, and policy-assumed states without dropping local
contexts, premises, proof hints, anchors, seed accounting, or ATP-bound
obligations. Preserve task-8 normalization output and do not implement
discharge evidence, dependency slices, ATP translation, proof/cache reuse,
kernel acceptance, corpus runner activation, or new algorithm payload
families. Add focused Rust tests for status transitions, policy statuses that
retain context and proof hints, ATP-bound obligations that remain concrete, and
deterministic debug/rendering inputs. Run cargo fmt --check,
cargo test -p mizar-vc, cargo clippy -p mizar-vc --all-targets -- -D warnings,
git diff --check, and git diff --cached --check after explicit path staging.
Use review-only agents for the required AGENTS.md review phases.
```

Rationale: task 9 は normalized VC data と後続 pre-ATP discharge の境界である。
status の誤りは obligation を隠したり ATP-bound goal を弱めたりし得るため `xhigh`
を保つ。documentation-only typo fix だけなら lower reasoning でもよい。

## Task 9 Handoff

Recommended reasoning: `xhigh`。

Prompt:

```text
Continue mizar-vc autonomous crate development from the completed task 9
status-policy commit. Before starting task 10, verify a clean worktree,
confirm the task 9 commit exists in git log, and re-read
doc/design/mizar-vc/en/vc_ir.md, generator.md, 00.crate_plan.md,
task_ledger.md, todo.md, doc/design/architecture/en/07.vc_generation.md,
crates/mizar-vc/src/vc_ir.rs, and crates/mizar-vc/src/generator.rs.
Implement task 10 only: write the English/Japanese `discharge.md` module spec
for deterministic pre-ATP discharge. Specify supported deterministic discharge
classes, computation limits, explanations, evidence references, status
interaction with `NeedsAtp` and policy statuses, and the no-erase ATP boundary.
Classify external gaps for ATP/kernel/proof/cache/corpus consumers and missing
source-derived fixtures. Do not write Rust source, create evidence, compute
dependency slices, translate ATP problems, change `.miz` fixtures,
expectations, `doc/spec`, or traceability metadata, or activate corpus runner
support. Run git diff --check and git diff --cached --check after explicit path
staging. Use review-only agents for the required AGENTS.md review phases.
```

Rationale: task 10 は implementation の前に phase-12 contract を定義する。
ATP boundary や evidence contract が曖昧だと discharge は obligation を隠し得るため
`xhigh` を保つ。typo-only documentation cleanup だけなら lower reasoning でもよい。

## Task 10 Handoff

Recommended reasoning: `xhigh`。

Prompt:

```text
Continue mizar-vc autonomous crate development from the completed task 10
discharge.md spec commit. Before starting task 11, verify a clean worktree,
confirm the task 10 commit exists in git log, and re-read
doc/design/mizar-vc/en/discharge.md, vc_ir.md, generator.md,
00.crate_plan.md, task_ledger.md, todo.md, crates/mizar-vc/src/vc_ir.rs, and
crates/mizar-vc/src/generator.rs. Implement task 11 only: deterministic
pre-ATP discharge engine source and focused tests for the supported task-10
classes whose explicit inputs are already represented in `VcIr`. Preserve VC
order, contexts, proof hints, anchors, generated formulas, seed accounting, and
the no-erase `NeedsAtp` boundary. Add stable handling for limit-exceeded and
unsupported-rule cases. Use only replayable explicit facts/premise refs/policy
inputs; missing traces must fail closed. Do not implement dependency slices,
ATP translation, kernel/proof/cache/corpus integration, active `.miz` fixtures,
expectations, `doc/spec`, traceability metadata, or broad evidence
serialization beyond the minimal untrusted evidence refs required to represent
`Discharged` status. Run cargo fmt --check, cargo test -p mizar-vc,
cargo clippy -p mizar-vc --all-targets -- -D warnings, git diff --check, and
git diff --cached --check after explicit path staging. Use review-only agents
for the required AGENTS.md review phases.
```

Rationale: task 11 は最初の phase-12 source implementation である。deterministic
discharge は ATP-bound obligation を黙って消したり unavailable trace を信頼したりしては
ならないため `xhigh` を保つ。documentation-only typo fix だけなら lower reasoning でもよい。
