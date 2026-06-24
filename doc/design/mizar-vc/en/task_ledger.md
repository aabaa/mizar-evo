# Task Ledger: mizar-vc

> Canonical language: English. Japanese companion:
> [../ja/task_ledger.md](../ja/task_ledger.md).

This ledger is the restart point for autonomous `mizar-vc` crate work. Before
starting any task, check `git status`, `git log`, this table, and
[todo.md](./todo.md). A task is complete only when its commit exists in
history, final review outcomes are known, verification results are known, and
deferred reasons are recorded. A commit cannot contain its own final hash, so
self-hashes are verified from `git log` before the next task starts and
backfilled by a later committed bookkeeping point or the closeout task.

| Task | Status | Commit | Reviews | Verification | Deferred / notes |
|---|---|---|---|---|---|
| 0. Crate plan | complete | `9697036b0f012cfc578a015dc5a0d6f37bf85143` | Spec/doc review: medium registration-correctness and derived-doc authority findings fixed; final re-review no blocking/high/medium findings. Test sufficiency review: no findings. Full implementation review: low future-link and stale task-scope findings fixed; final re-review no blocking/high/medium findings. Source/doc consistency review: medium task-15 and conditional-verification findings fixed; final re-review no blocking/high/medium findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Docs-only. Classifies initial `spec_gap`, `test_gap`, `design_drift`, `source_drift`, `external_dependency_gap`, and `deferred` rows in `00.crate_plan.md`; synchronizes todo wording for current runner/verification gaps and registration-style correctness seed scope; no crate source is created. |
| 1. Crate scaffold and lint-policy guard | complete | `adfff1cbc3ebce9db13e73d4d29bfd9b1ac1971d` | Spec/doc review: no blocking/high/medium/low findings. Test sufficiency review: low private-scope guard finding fixed; final re-review no findings. Full implementation review: no findings after guard strengthening. Source/doc consistency review: no blocking/high/medium/low findings. | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Scaffold-only. Adds workspace member, lockfile entry, minimal crate manifest, documentation-only `src/lib.rs`, and lint guard. No semantic VC APIs, module source files, `.miz` fixtures, expectations, `doc/spec`, or module specs changed. |
| 2. Spec: `vc_ir.md` | complete | `ac778b008be75ea21eda4d2e69c7713a88b0d4ea` | Spec/doc review: medium seed-accounting, generated-goal, status-name, and expansion-index findings fixed; final re-review only ledger-status bookkeeping remained, then fixed. Test sufficiency review: medium task-8 seed-bijection wording fixed; final re-review no blocking/high/medium findings. Full implementation review: medium status-name, proof-hint, algorithm-subkind, and ledger-status findings fixed. Source/doc consistency review: medium todo seed-accounting drift fixed; final re-review no blocking/high/medium findings. | `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Spec-only. Adds English/Japanese `vc_ir.md`, syncs task 2 and task 8 todo wording to seed accounting / explicit concrete cardinality, and changes no Rust source, `.miz` fixtures, expectations, `doc/spec`, or traceability metadata. External gaps for proof-verification runner, ATP/kernel/proof/cache consumers, and source-derived payloads remain deferred. |
| 3. Implement `vc_ir` data shapes | complete | `c32d767368ef9d16fdcf92620c2b2afecb13fc9d` | Spec/doc review: medium `ModuleId`, expanded-index/rendering, incomplete-anchor, and quantified-binder findings fixed; final re-review no blocking/high/medium findings. Test sufficiency review: medium rendering, seed-accounting, and status/context coverage findings fixed; final re-review no blocking/high/medium findings. Full implementation review: medium seed-mapping, nested-reference, anchor-completeness, and quantified-binder findings fixed; final re-review no blocking/high/medium findings. Source/doc consistency review: medium `PolicyOpen` no-VC mismatch fixed and low module-link finding fixed; final re-review no blocking/high/medium findings. | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Rust source task. Adds `src/vc_ir.rs`, exposes only `pub mod vc_ir;`, updates lint guard for the spec-backed module, adds validation and deterministic debug rendering tests, and keeps seed intake, generator logic, status transitions, discharge, dependency slices, ATP translation, proof/cache reuse, kernel acceptance, `.miz` fixtures, expectations, `doc/spec`, and traceability metadata deferred/out of scope. |
| 4. Obligation-seed intake | complete | `ba20db550cf92979bdb8809e9f64fbe5cd193c1b` | Spec/doc review: medium missing source-map documentation finding fixed; final re-review no blocking/high/medium findings. Test sufficiency review: medium origin-preservation coverage finding fixed; final re-review no blocking/high/medium findings. Full implementation review: no blocking/high/medium findings after follow-up. Source/doc consistency review: no blocking/high/medium findings after follow-up. | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Rust source task. Adds a pre-`VcId` `SeedIntakeTable` over `ObligationSeedHandoff`, preserving handoff order and origins, rejecting duplicate `(canonical_key, origin)` rows and missing source-map entries, representing skipped/deferred/error/missing-goal rows as visible no-VC mappings, and keeping concrete VC generation, generator normalization, final `VcId` assignment, discharge, dependency slices, ATP translation, proof/cache reuse, kernel acceptance, `.miz` fixtures, expectations, `doc/spec`, and traceability metadata deferred/out of scope. |
| 5. Spec: `generator.md` | complete | `e324beab799f972dcf78e897b163aebd9414725e` | Spec/doc review: high generated-core ownership, medium Pick non-emptiness, and medium module-table findings fixed; final re-review no blocking/high/medium findings after verification/staging. Test sufficiency review: medium theorem-status, sethood/non-emptiness, and call/return coverage findings fixed; final re-review no blocking/high/medium findings. Full implementation review: medium module-table finding fixed; final re-review no blocking/high/medium findings. Source/doc consistency review: medium module-table finding fixed; final re-review no blocking/high/medium findings. | `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Spec-only. Adds English/Japanese `generator.md`, records local-context construction, theorem/definition generation, generated core obligations, explicit registration-style correctness payload handling, algorithm VC families including Pick non-emptiness, controlled unfolding, and task-8 normalization/classification handoff. Leaves Rust source, `.miz` fixtures, expectations, `doc/spec`, traceability metadata, proof-verification runner activation, and unavailable dedicated registration/redefinition/reduction payloads deferred/out of scope. |
| 6. Theorem, definition, generated core, and registration-style correctness VCs | complete | `b5634eb878b39558b981bcbba972e8b36c3203c9` | Spec/doc review: high registration-style boundary and medium theorem-status gap findings fixed; final re-review no blocking/high/medium findings after staged-verification record update. Test sufficiency review: medium definition-family, theorem-status, registration-negative, no-candidate, and determinism findings fixed; final re-review no blocking/high/medium findings. Full implementation review: high stale/partial intake findings and medium unfold, context-sort, schema, terminal-goal, and diagnostic-wording findings fixed; final re-review no findings. Source/doc consistency review: high/medium marker, schema, sort-key, unfold, lint-message, and GEN-G005 wording findings fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Rust source task. Backfills task 5 hash and adds `src/generator.rs`, exposes `pub mod generator`, implements pre-normalized `CoreGenerationCandidateSet` for task-6 seed families, enforces full seed-intake table equality against the handoff, preserves explicit `CoreProvenance` markers for registration-style, theorem-status, terminal-proof, and unfold behavior, canonicalizes local context, and extends lint guards. Later algorithm VCs, final `VcId` assignment, status transitions, discharge, dependency slices, ATP/kernel/proof/cache/corpus integration, and missing dedicated registration/redefinition/reduction payload fields stay external/deferred. |
| 7. Algorithm VCs | complete | `a15a2ee3e21974727fab2f8406b2e161b3f3c2f7` | Spec/doc review: high seed-intake conflict and medium broad-scope wording findings fixed; final re-review no blocking/high/medium findings. Test sufficiency review: medium AlgorithmAssertion, partial/ghost, unavailable-family, metadata, and determinism findings fixed; final re-review no blocking/high/medium findings. Full implementation review: medium flow/algorithm mismatch and site-membership findings fixed; final re-review no blocking/high/medium findings. Source/doc consistency review: medium eligible-intake, site-validation, and planned-test drift findings fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Rust source task. Backfills task 6 hash; adds goal-bearing flow-derived algorithm candidates for requires, ensures, assertions, and supported loop-invariant entry/preservation/break/continue sites; updates seed intake so eligible deferred `FlowDerived` `AlgorithmContract` rows remain status-preserved but candidate-eligible; validates flow id, algorithm id, site table membership, goal, and placement metadata; records missing site/data, term-only termination, partial termination, ghost erasure, unavailable algorithm families, and incomplete loop metadata as visible no-candidate/deferred records. Adds test-only `mizar-resolve` dev-dependency for `ControlFlowIr` fixture `SymbolId` construction only. |
| 8. Normalization, classification, and `VcId` assignment | complete | `6b4a7ef661886d6339f8ac24e21ad68e9f7ac302` | Spec/doc review: medium stable-kind-order and task-gap-classification findings fixed; final re-review no blocking/high/medium findings. Test sufficiency review: medium full-rank coverage and status-boundary findings fixed; final re-review no blocking/high/medium findings. Full implementation review: no findings; final re-review after test fixes no findings. Source/doc consistency review: no findings; final re-review after test fixes no findings. | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Rust source task. Adds `VcNormalizationInput` and candidate-set normalization into final `VcSet`, assigns dense snapshot-local `VcId`s by documented `VcKind` classification rank, candidate sort key, and handoff id, builds sorted final no-VC/one-VC seed accounting, preserves deferred flow seed status and existing VC status, appends normalization provenance only, rejects duplicate candidate sort keys and duplicate seed ownership, and keeps expanded mappings validation-only through `VcSet` validation. Status transitions, discharge, dependency slices, ATP translation, kernel/proof/cache/corpus integration, `.miz` fixtures, expectations, `doc/spec`, and traceability metadata remain deferred/out of scope. |
| 9. Status and policy model | complete | `30c8e303c2c88d70a0dd69295ec001280471519a` | Spec/doc review: medium todo discharge-scope finding fixed; final re-review no blocking/high/medium findings. Test sufficiency review: medium multi-VC default/override, policy-action provenance, and invalid generated-marker findings fixed; final re-review no blocking/high/medium findings. Full implementation review: no findings; final re-review after test fixes no findings. Source/doc consistency review: no findings; final re-review after test fixes no findings. | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Rust source task. Adds `VcStatusPlan`, `VcStatusAction`, `VcStatusOverride`, and `VcSet::try_with_status_plan` for immutable deterministic status-policy projection. Supports preserve, `NeedsAtp`, `PolicyOpen`, and `AssumedByPolicy`, preserves VC order, contexts, premises, proof hints, anchors, generated formulas, seed accounting, and ATP-bound obligations, appends `StatusPolicy` provenance only on actual status changes, rejects duplicate/unsorted/missing overrides, and fails closed through `VcSet` validation for invalid assumption markers. Discharge evidence, dependency slices, ATP translation, kernel/proof/cache/corpus integration, `.miz` fixtures, expectations, `doc/spec`, and new generator payload families remain deferred/out of scope. |
| 10. Spec: `discharge.md` | complete | `18c86f9b03318c28e39311162ae3e89adc0e2d2a` | Spec/doc review: medium discharged-evidence unavailable-trace wording fixed; final re-review no blocking/high/medium findings. Test sufficiency review: medium positive discharge-class coverage finding fixed; final re-review no blocking/high/medium findings. Full implementation review: no findings. Source/doc consistency review: no findings; final re-review after planned-test fix no findings. | `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Spec-only task. Adds English/Japanese `discharge.md` with deterministic pre-ATP discharge scope, fail-closed supported rule classes, limit model shape, evidence/explanation requirements, status interaction, no-erase ATP boundary, planned task-11/task-12 tests, and `spec_gap` / `source_drift` / `test_gap` / `external_dependency_gap` / `deferred` classifications. No Rust source, `.miz` fixtures, expectations, `doc/spec`, traceability metadata, ATP/kernel/proof/cache/corpus integration, dependency slices, or active runner support. |
| 11. Deterministic discharge engine | ready to commit | pending self-hash; verify from `git log` after commit | Spec/doc review: medium task-slice/gap-classification finding fixed; final re-review no blocking/high/medium findings. Test sufficiency review: medium reflexivity/ref-normalization, local fact family, trace family, definitional negative gating, and marker-only fail-closed findings fixed; final re-review no findings. Full implementation review: high marker-only trace/unfold/computation erasure findings fixed; final re-review no blocking/high/medium findings. Source/doc consistency review: medium default-limit, evidence-boundary, planned-test, and status-bookkeeping findings fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; `git diff --cached --check` pending explicit staging. | Rust source task. Adds `src/discharge.rs`, exposes `pub mod discharge`, updates lint-policy coverage, and implements deterministic pre-ATP discharge for explicit task-10 classes represented in `VcIr`. Discharges only goal-linked tautology/reflexivity/ref-normalization, local contradiction, explicit type/sethood/non-emptiness/checker/local facts, explicit trace refs with goal-linked support, policy-gated definitional reductions with goal-linked support, and bounded computation with explicit goal-linked result support. Records default computation policy `task-11-computation-step-limit` with `max_steps = 64`, uses minimal stable `DischargeEvidenceRef`, preserves order/context/proof hints/anchors/generated formulas/seed accounting, and returns stable `NeedsAtp` explanations for unsupported, marker-only, or limit-exceeded cases. Detailed evidence serialization, dependency slices, ATP/kernel/proof/cache/corpus integration, `.miz` fixtures, expectations, `doc/spec`, traceability metadata, and active runner support remain deferred/out of scope. |
| 12. Discharge evidence and explanations | not started | pending | pending | pending | Rust source task. |
| 13. Spec: `dependency_slice.md` | not started | pending | pending | pending | Spec-only task. |
| 14. Dependency-slice computation | not started | pending | pending | pending | Rust source task. |
| 15. Corpus runner record for `proof_verification` | not started | pending | pending | pending | Deferred-record task unless runner/extraction seams exist by then. |
| 16. Determinism suite | not started | pending | pending | pending | Test task plus source fixes only when spec-backed. |
| 17. Public-enum forward-compatibility policy | not started | pending | pending | pending | Test/docs task. |
| 18. Source/spec correspondence audit | not started | pending | pending | pending | Audit task. |
| 19. Bilingual documentation sync audit | not started | pending | pending | pending | Audit/docs task. |
| 20. Obligation anchors and cross-edit reuse identity | not started | pending | pending | pending | Rust source task over architecture-22 identity. |
| 21. Architecture-22 follow-up audit | not started | pending | pending | pending | Audit task. |
| 22. Module-boundary refactor gate | not started | pending | pending | pending | Audit task; source moves only if required. |
| Closeout. Crate exit report and quality review | not started | pending | pending | pending | Complete only after hard gates pass and read-only quality score is >= 90. |

## Task 0 Handoff

Recommended reasoning: `xhigh`.

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

Rationale: task 1 changes workspace and Rust crate scaffolding, so xhigh keeps
the manifest, lint policy, and one-task-one-commit constraints in view. Lower
reasoning is acceptable only for a purely mechanical ledger typo fix; keep
`xhigh` if dependencies, lint policy, or workspace membership are touched.

## Task 1 Handoff

Recommended reasoning: `xhigh`.

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

Rationale: task 2 defines the central VC IR and anchor contract before any
semantic API appears. Keep `xhigh` for the proof/identity boundary; lower
reasoning is acceptable only for typo-only documentation cleanup.

## Task 2 Handoff

Recommended reasoning: `xhigh`.

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

Rationale: task 3 is the first semantic Rust surface in `mizar-vc`, so `xhigh`
keeps the proof-boundary, identity, and no-downstream-ownership constraints in
view. Lower reasoning is acceptable only for documentation-only typo fixes.

## Task 3 Handoff

Recommended reasoning: `xhigh`.

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

Rationale: task 4 is the first handoff boundary from `mizar-core` into
`mizar-vc`. Keep `xhigh` because seed accounting is a proof-obligation
completeness boundary; lower reasoning is acceptable only for typo-only
documentation cleanup.

## Task 4 Handoff

Recommended reasoning: `xhigh`.

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

Rationale: task 5 defines the generation contract before implementation tasks
6-8. Keep `xhigh` because it spans proof obligations, registration-style
correctness boundaries, and algorithm-control-flow VC categories; lower
reasoning is acceptable only for typo-only documentation cleanup.

## Task 5 Handoff

Recommended reasoning: `xhigh`.

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

Rationale: task 6 is the first generator implementation slice and touches proof
obligation completeness without yet assigning final `VcId`s. Keep `xhigh` for
the seed accounting, registration-style correctness, and generated core
obligation boundaries; lower reasoning is acceptable only for typo-only
documentation cleanup.

## Task 6 Handoff

Recommended reasoning: `xhigh`.

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

Rationale: task 7 adds the algorithm side of generation and depends on the
task-6 proof-obligation accounting boundary. Keep `xhigh` because control-flow
VC generation is broad and can silently create proof gaps; lower reasoning is
acceptable only for a documentation-only typo fix.

## Task 7 Handoff

Recommended reasoning: `xhigh`.

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

Rationale: task 8 turns pre-normalized candidates into final `VcIr` and is the
first task to assign `VcId`s, so `xhigh` is appropriate for seed accounting and
proof-obligation completeness. Lower reasoning is acceptable only for typo-only
documentation cleanup.

## Task 8 Handoff

Recommended reasoning: `xhigh`.

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

Rationale: task 9 is the boundary between normalized VC data and later
pre-ATP discharge. Keep `xhigh` because status mistakes can silently hide
obligations or weaken ATP-bound goals; lower reasoning is acceptable only for
documentation-only typo fixes.

## Task 9 Handoff

Recommended reasoning: `xhigh`.

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

Rationale: task 10 defines the phase-12 contract before implementation. Keep
`xhigh` because discharge can hide obligations if the ATP boundary or evidence
contract is underspecified; lower reasoning is acceptable only for typo-only
documentation cleanup.

## Task 10 Handoff

Recommended reasoning: `xhigh`.

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

Rationale: task 11 is the first phase-12 source implementation. Keep `xhigh`
because deterministic discharge must never silently erase ATP-bound obligations
or trust unavailable traces; lower reasoning is acceptable only for
documentation-only typo fixes.
