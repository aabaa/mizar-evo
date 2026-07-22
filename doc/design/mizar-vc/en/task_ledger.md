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
| 11. Deterministic discharge engine | complete | `d4643a7f1078ec330640e63021942bc245d9a609` | Spec/doc review: medium task-slice/gap-classification finding fixed; final re-review no blocking/high/medium findings. Test sufficiency review: medium reflexivity/ref-normalization, local fact family, trace family, definitional negative gating, and marker-only fail-closed findings fixed; final re-review no findings. Full implementation review: high marker-only trace/unfold/computation erasure findings fixed; final re-review no blocking/high/medium findings. Source/doc consistency review: medium default-limit, evidence-boundary, planned-test, and status-bookkeeping findings fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Rust source task. Adds `src/discharge.rs`, exposes `pub mod discharge`, updates lint-policy coverage, and implements deterministic pre-ATP discharge for explicit task-10 classes represented in `VcIr`. Discharges only goal-linked tautology/reflexivity/ref-normalization, local contradiction, explicit type/sethood/non-emptiness/checker/local facts, explicit trace refs with goal-linked support, policy-gated definitional reductions with goal-linked support, and bounded computation with explicit goal-linked result support. Records default computation policy `task-11-computation-step-limit` with `max_steps = 64`, uses minimal stable `DischargeEvidenceRef`, preserves order/context/proof hints/anchors/generated formulas/seed accounting, and returns stable `NeedsAtp` explanations for unsupported, marker-only, or limit-exceeded cases. Detailed evidence serialization, dependency slices, ATP/kernel/proof/cache/corpus integration, `.miz` fixtures, expectations, `doc/spec`, traceability metadata, and active runner support remain deferred/out of scope. |
| 12. Discharge evidence and explanations | complete | `57c4e247ca13cdcf05e92d9854e41f60fa5e0f49` | Spec/doc review: high pre-existing `Discharged` ambiguity and medium artifact/kernel/round-trip/computation-step wording findings fixed; final re-review no findings. Test sufficiency review: medium multi-discharged-output, replay-input, non-evidence-status, and clone coverage findings fixed; final re-review no findings. Full implementation review: no findings. Source/doc consistency review: no findings. | `cargo fmt --check` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Rust source task. Adds structured in-memory `DischargeEvidenceRecord`s, `DischargeEvidenceSource`, input/replay/computation evidence data, `DischargeOutput::evidence_records`, and deterministic `DischargeOutput::debug_text`. Newly discharged VCs get `NewlyProduced` evidence records with preserved input refs, policy inputs, unfold requests, computation hints, and limit tuple data; pre-existing `Discharged` input statuses get `PreExistingStatus` records that copy status evidence and explicitly avoid reconstructing missing replay data. Policy, assumed, skipped, deferred, error, unsupported, marker-only, and limit-exceeded outputs retain explanations but no discharged evidence. Keeps `VcStatus::Discharged` as minimal `DischargeEvidenceRef`; artifact serialization, dependency slices, kernel/proof/cache validation, ATP integration, `.miz` fixtures, expectations, `doc/spec`, and traceability metadata remain deferred/out of scope. |
| 13. Spec: `dependency_slice.md` | complete | `6238217eedc55e76ec277ab14bd1d78a3b57c6a6` | Spec/doc review: medium `VcId` fingerprint-boundary and task-14 bookkeeping findings fixed; final re-review no findings. Test sufficiency review: medium core-formula/definition/unfold and `VcId`/unknown-marker fingerprint test-boundary findings fixed; final re-review no findings. Full implementation review: no findings. Source/doc consistency review: no findings. | `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Spec-only task. Adds English/Japanese dependency-slice contract for conservative per-VC dependency classification, stable cross-edit slice fingerprints that exclude snapshot-local `VcId`, unknown coverage handling, Task-14 planned coverage, and downstream cache/reuse boundaries. No Rust source, `.miz` fixtures, expectations, `doc/spec`, traceability metadata, ATP/kernel/proof/cache/corpus integration, or dependency-slice implementation. |
| 14. Dependency-slice computation | complete | `26e5fea26769e1bf7ccb47e99d814709f035801f` | Spec/doc review: medium completeness/cache-miss surface and mismatched discharge-output test-scope findings fixed; final re-review no findings. Test sufficiency review: medium status-boundary, discharge-input, broad assertion, and anchor-hash fingerprint coverage findings fixed; final re-review no findings. Full implementation review: high snapshot-local discharge hash fingerprint and binder/context recursion findings plus medium provenance finding fixed; final re-review no findings. Source/doc consistency review: medium evidence-hash boundary wording fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo test -p mizar-vc dependency_slice` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Rust source task. Adds `src/dependency_slice.rs`, exposes `pub mod dependency_slice`, updates lint-policy coverage, and implements deterministic dependency-slice computation against `dependency_slice.md`. Slices preserve VC order and source status, keep `VcId` as owner/order metadata, collect local context/generated/core/definition/import/trace/policy/anchor/discharge/seed dependencies, expose `Complete` vs `IncompleteUncacheable` and `requires_cache_miss`, reject mismatched `DischargeOutput`/`VcSet`, include provenance and stable anchor hash bytes, normalize snapshot-local discharge evidence hashes out of reusable fingerprints, and fail closed on binder/context recursion with unknown coverage. `.miz` fixtures, expectations, `doc/spec`, traceability metadata, ATP/kernel/proof/cache/corpus integration, artifact serialization, and future proof-evidence validation gates remain deferred/out of scope. |
| 15. Corpus runner record for `proof_verification` | complete | `beee07a8009245e2bc0096d98df3968ea1212ac3` | Spec/doc review: initial medium Task-7 audit-scope finding fixed; final re-review no findings. Test sufficiency review: no findings. Full implementation review: no findings. Source/doc consistency review: no findings. | `git diff --check` passed; `cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml` passed; `cargo test -p mizar-test` passed; `git diff --cached --check` passed after explicit path staging. | Deferred-record task. Records concrete external_dependency_gap reasons instead of fake active fixtures because mizar-test has no active proof_verification runner/tag gate, source-to-core/source-to-VC extraction seams are unavailable for `.miz` corpus inputs, downstream ATP/proof/kernel/artifact consumers are not wired to mizar-vc outputs, and task-7 algorithm VC audit payload families are not all exposed by upstream ControlFlowIr/obligation handoff data. Updates traceability metadata for the mizar-vc proof_verification corpus obligation and the task-7 algorithm VC review-audit obligation. No `.miz` fixtures, expectations, runner code, Rust source, `doc/spec`, or active proof_verification coverage are added. |
| 16. Determinism suite | complete | `8b183e538fa4007e82b0c2b2af058ebe566fca22` | Spec/doc review: low Task-15 hash consistency finding fixed; final re-review no findings. Test sufficiency review: no findings. Full implementation review: medium status-projection-to-discharge/slice boundary finding fixed; final re-review no blocking/high/medium findings. Source/doc consistency review: low Task-15 hash consistency finding fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo test -p mizar-vc --test determinism_suite` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Test task. Adds `crates/mizar-vc/tests/determinism_suite.rs`, a public integration determinism suite that builds identical seed/handoff inputs through seed intake, candidate generation, normalization, status projection, deterministic discharge, and dependency-slice computation. The suite compares VC sets/debug text, dense `VcId`s, kind order, status projection, discharge output/debug text/evidence, slice output/debug text, slice ids, and slice fingerprints. Source behavior is unchanged; no `.miz` fixtures, expectations, `doc/spec`, runner support, ATP/kernel/proof/cache integration, or external dependency wiring are added. |
| 17. Public-enum forward-compatibility policy | complete | `f65ff56d9a3a555586cf21189780aaaa1017359d` | Spec/doc review: no findings. Test sufficiency review: no findings. Full implementation review: no findings. Source/doc consistency review: no findings. | `cargo fmt --check` passed; `cargo test -p mizar-vc vc_public_enums_are_forward_compatible_and_documented` passed; `cargo test -p mizar-vc --test lint_policy` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Test/docs task. Classifies every current public enum in `vc_ir`, `generator`, `discharge`, and `dependency_slice` as a downstream forward-compatible `#[non_exhaustive]` surface in the owning EN/JA module specs, records that no exhaustive public enum exceptions are owned by mizar-vc, and extends `tests/lint_policy.rs` to guard missing attributes, new nested/re-export enum surfaces, missing policy sections, and exact source/spec enum-list drift. Source behavior is unchanged; no `.miz` fixtures, expectations, `doc/spec`, runner support, ATP/kernel/proof/cache integration, or external dependency wiring are added. |
| 18. Source/spec correspondence audit | complete | `373e943b43e2c17b5a1cad282160e71c4c51de89` | Spec/doc review: no findings. Test sufficiency review: no findings. Full implementation review: no findings. Source/doc consistency review: no findings. | `cargo fmt --check` passed; `cargo test -p mizar-vc vc_source_spec_audit_covers_public_modules_and_deferred_gaps` passed; `cargo test -p mizar-vc --test lint_policy` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Audit/docs task. Adds paired EN/JA `source_spec_audit.md` reports that trace public modules, public API families, promised behavior, source files, and test evidence for `vc_ir`, `generator`, `discharge`, and `dependency_slice`; records no new source/spec drift or repo metadata conflict; and preserves known `external_dependency_gap` / `deferred` follow-ups for proof-verification runner/extraction, downstream ATP/kernel/proof/cache consumers, incomplete upstream payload families, Task 20 reuse identity, Task 21 architecture audit, Task 22 module-boundary gate, and closeout. Adds a lint-policy smoke guard for audit coverage markers. Source behavior is unchanged; no `.miz` fixtures, expectations, `doc/spec`, runner support, ATP/kernel/proof/cache integration, or external dependency wiring are added. |
| 19. Bilingual documentation sync audit | complete | `f36852c74d5f1d0724514f7ecda0b1a539ab6561` | Spec/doc review: medium semantic-parity scope finding fixed; final re-review no findings. Test sufficiency review: no findings. Full implementation review: medium self-inventory finding fixed; final re-review no findings. Source/doc consistency review: no findings. | `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Audit/docs task. Adds paired EN/JA `bilingual_sync_audit.md` reports that compare every current English canonical design document with its Japanese companion for file pairing, companion links, substantive semantic parity, task status, gap/deferred classifications, audit inventories, planned tests, and public enum policy tables. Backfills the Task 18 commit hash and marks Task 19 complete in paired todos. Source behavior is unchanged; no Rust source, `.miz` fixtures, expectations, `doc/spec`, traceability metadata, runner support, ATP/kernel/proof/cache integration, or external dependency wiring are added. |
| 20. Obligation anchors and cross-edit reuse identity | complete | `2f3eb323be8080bf231e1b69dfc9e9e729bb45f9` | Spec/doc review: high proof-evidence, raw-id fingerprint, `VcId`-derived discharge-hash findings plus medium generated-formula/bookkeeping findings fixed; final re-review no findings. Test sufficiency review: medium independent input and unresolved-payload coverage findings fixed; final re-review no findings. Full implementation review: high raw row-id and dense owner-id reuse findings fixed; final re-review no findings. Source/doc consistency review: medium quantified-binder finding fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo test -p mizar-vc cross_edit_reuse_key_survives_vc_id_shift_only_with_required_inputs` passed; generator focused tests passed; unresolved-payload focused tests passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets -- -D warnings` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Rust source/docs task. Wires source-shape `ObligationAnchor` hashes, fail-closed canonical goal/context hash markers, `CanonicalVcFingerprint`, `LocalContextFingerprint`, cross-edit-stable deterministic discharge evidence hashes, reusable dependency-slice seed fingerprint payloads that exclude diagnostic row ids, and a proof-reuse candidate key that requires a complete anchor, current matching slice, policy fingerprint, and newly produced replayable deterministic discharge evidence. Adds insertion-before-obligation reuse coverage plus generated-formula-id-shift, stale-slice, policy-change, local-context-change, generated-goal-change, unresolved-payload, pre-existing-evidence, and incomplete-anchor checks. Proof-witness hashes, ATP/kernel/proof/cache validation, artifact consumers, source-derived runner support, and upstream missing payload families remain `external_dependency_gap` / `deferred`. |
| 21. Architecture-22 follow-up audit | complete | `a8243c3498249fe75d3619fbbe4f5a2dc94b86a2` | Spec/doc review: high audit-artifact mismatch plus medium remaining-gap, bookkeeping, and stale pair-inventory findings fixed; final re-review no findings. Test sufficiency review: no findings. Full implementation review: medium stale bilingual/ledger finding fixed; final re-review no findings. Source/doc consistency review: low public-surface bucket finding fixed; final re-review no findings. | `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Docs-only audit task. Adds paired `architecture_22_audit.md`, updates source/spec and bilingual sync audit reports after Task 20, backfills the Task 20 hash, and marks Task 21 complete in paired todos. Rust source, `.miz` fixtures, expectations, `doc/spec`, traceability metadata, runner support, ATP/kernel/proof/cache validation, artifact consumers, and source-derived payload integration are unchanged. Remaining architecture-22 gaps are explicitly classified as `external_dependency_gap` / `deferred`. |
| 22. Module-boundary refactor gate | complete | `76f286f9a3d1e6d6f096b84be7b5f38873e48d42` | Spec/doc review: medium stale source/spec, bilingual audit, and task-state findings fixed; final re-review no findings. Test sufficiency review: no findings. Full implementation review: medium ledger-status finding fixed; final re-review no findings. Source/doc consistency review: no findings. | `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Docs-only audit task. Adds paired `module_boundary_audit.md`, backfills the Task 21 hash, updates source/spec and bilingual sync audit reports, and marks Task 22 complete in paired todos. No Rust source, `.miz` fixtures, expectations, `doc/spec`, traceability metadata, runner support, downstream ATP/kernel/proof/cache integration, public APIs, diagnostics, deterministic renderings, or artifact-facing schemas are changed. No required move-only split was found; optional private helper/test splits inside large modules are deferred maintenance tasks if pursued. |
| Closeout. Crate exit report and quality review | complete | `0996ad28c57298bd68024eb1f9a6638ef7e37108` | Spec/doc review: no findings. Test sufficiency review: no findings. Full implementation review: no findings. Source/doc consistency review: no findings. Read-only crate quality review: hard gates pass, score 94/100. | `cargo fmt --check` passed; `cargo clippy --all-targets --all-features -- -D warnings` passed; `cargo test` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit path staging. | Docs-only closeout task. Adds paired `crate_exit_report.md`, updates plan/todo/audit ledgers, backfills the Task 22 hash, records all task commits, records broad verification, classifies remaining `external_dependency_gap` / `deferred` items, and hands off to the next owner for proof-verification runner/extraction or downstream ATP/kernel/proof/cache work. Rust source, `.miz` fixtures, expectations, `doc/spec`, traceability metadata, runner support, ATP/kernel/proof/cache consumers, public APIs, diagnostics, deterministic renderings, and artifact-facing schemas are unchanged. |
| 24. Spec: kernel evidence handoff | complete | `c33c583d107c8211c22efcbb89d88144f32d163c` | Spec/doc review: initial high premise-selection wording and medium imported-context finding fixed; final re-review no blocking/high/medium findings. Test sufficiency review: no findings. Full implementation review: initial medium envelope-shape and stale next-handoff findings fixed; final re-review no blocking/high/medium findings. Source/doc consistency review: no findings. | `git diff --check` passed; `cargo test -p mizar-vc --test lint_policy --offline` passed; `git diff --cached --check` passed after explicit path staging. | Docs-only post-closeout correction task. Adds paired `kernel_evidence_handoff.md`, updates plan/todo/audit/exit-report classifications after `mizar-kernel` tasks 23-29, and records the task-25 handoff. Rust source, `.miz` fixtures, expectations, `doc/spec`, traceability metadata, runner support, SAT solving, kernel calls, ATP backends, backend proof methods, resolution traces, legacy certificate acceptance, and fabricated formula/substitution/provenance payloads are unchanged. |
| 25. Kernel evidence handoff builder | complete | `0ed1bc23e2bc7f66d2f4f53a8e289721d47105b9` | Spec/doc review: initial high ledger/task-state inconsistency plus medium discharge-record and fingerprint-algorithm findings fixed; final re-review no blocking/high/medium findings. Test sufficiency review: initial medium generated-premise/provenance and missing-payload coverage findings, follow-up medium proof-hint/discharge identity, generated/imported fail-closed, and context-canonicalization findings, plus low ordering/API-name guard findings fixed; final re-review no findings. Full implementation review: initial high imported-fingerprint mismatch, medium proof-hint target-binding, context canonicalization, substitution side-condition, status-evidence target, and premise-order findings, plus low role/diagnostic findings fixed; final re-review no findings. Source/doc consistency review: initial medium task-state and stale-handoff finding fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo test -p mizar-vc kernel_evidence_handoff --offline` passed; `cargo test -p mizar-vc --test lint_policy --offline` passed; `cargo test -p mizar-vc --offline` passed; `cargo clippy -p mizar-vc --all-targets --all-features -- -D warnings` passed; `cargo test -p mizar-core --offline` passed; `cargo test -p mizar-kernel --offline` passed; `cargo test -p mizar-artifact --offline` passed; `cargo test -p mizar-checker --offline` passed; `git diff --check` passed before explicit staging. | Rust source/docs task. Adds `src/kernel_evidence_handoff.rs`, exposes the module, registers lint-policy guards, and implements immutable producer-side handoff packages for explicit formula/substitution/provenance/target-binding payloads. No SAT solving, kernel calls, ATP backend calls, backend proof methods, resolution traces, legacy certificate acceptance, caller-supplied instantiated formulas, or fabricated payloads are added. Downstream ATP/proof/cache/artifact consumers remain `external_dependency_gap` / `deferred`; task 26 owns reuse-hash integration. |
| 26. Dependency-slice and proof-reuse identity update | complete | `9c86900451068553a8e96938c420872b047c1d62` | Spec/doc review: initial medium task-ledger and stale-gap findings fixed; final re-review no blocking/high/medium findings. Test sufficiency review: initial high selected-VC target-binding coverage and medium lint-boundary findings fixed; final re-review no blocking/high/medium findings. Full implementation review: no blocking/high/medium findings. Source/doc consistency review: initial high premature completion state plus medium stale handoff and bilingual-inventory findings fixed; final re-review no blocking/high/medium findings. | `cargo fmt --check` passed; `cargo test -p mizar-vc dependency_slice --offline` passed; `cargo test -p mizar-vc --test determinism_suite --offline` passed; `cargo test -p mizar-vc --test lint_policy --offline` passed; `cargo test -p mizar-vc --offline` passed; `cargo clippy -p mizar-vc --all-targets --all-features -- -D warnings` passed; `cargo test -p mizar-core --offline` passed; `cargo test -p mizar-kernel --offline` passed; `cargo test -p mizar-artifact --offline` passed; `cargo test -p mizar-checker --offline` passed. | Rust source/docs task. Adds kernel evidence handoff identity as a dependency-slice class and proof-reuse key input, introduces `KernelEvidenceDependencyInput` and `try_compute_dependency_slices_with_kernel_evidence`, adds selected-VC target-fingerprint validation through `VcKernelEvidenceHandoff::targets_vc`, and makes legacy `proof_reuse_key_for` fail closed without current kernel handoff identity. Duplicate, unknown-VC, selected-VC-mismatched, stale-slice, missing-handoff, incomplete-anchor, and non-newly-produced evidence paths fail closed. No kernel calls, SAT solving, ATP backend calls, proof/cache/artifact consumers, resolution traces, backend proof methods, legacy certificate acceptance, or trusted instantiated-formula payloads are added. Downstream ATP/proof/cache/artifact consumers remain `external_dependency_gap` / `deferred`. |
| 27. Explicit goal polarity in the kernel evidence handoff | complete | `2d167bde40ccf7788b6de49cc9e324e7e7879987` | Spec/doc review: initial medium mizar-kernel soundness-argument finding fixed; final re-review no findings. Test sufficiency review: initial medium fail-closed ordering and current-`VcKind` coverage findings fixed; final re-review no findings. Full implementation review: initial medium missing ledger row and stale mizar-kernel TODO disposition findings fixed; final re-review no findings. Source/doc consistency review: no findings; `doc/design/spec_coverage_audit.md` unchanged because coverage ownership, traceability status, and deferred coverage classification did not change. | `cargo fmt --check` passed; `cargo test -p mizar-vc kernel_evidence_handoff` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets --all-features -- -D warnings` passed; `cargo test -p mizar-atp` passed; `cargo clippy --all-targets --all-features -- -D warnings` passed; `cargo test` passed. | Rust source/docs task. Adds explicit `KernelEvidenceHandoffInput::goal_polarity`, validates every current proof-obligation `VcKind` and nested kind family against `AssertFalseForRefutation` before package assembly, records the validated polarity in `final_goal.polarity`, and rejects consistency polarity with `GoalPolarityMismatch`. Direct mizar-atp test call sites now pass refutation polarity explicitly. Paired EN/JA docs record producer-side F1 closure; trusted checker-side B4 acceptance binding is implemented by mizar-kernel task 30. No SAT solving, kernel calls, ATP backend semantic changes, proof rows, placeholder runners, expectation changes, checker/core semantic changes, or fabricated formula/substitution/provenance payloads are added. `doc/design/spec_coverage_audit.md` remains unchanged because coverage ownership, traceability status, and deferred coverage classification did not change. |
| 28. Context-identity payload for non-imported source bindings | complete | `ab23833f70f3e8a0733621453e283246c1b5b7d1` | Spec/doc review: initial high handoff-identity-boundary and medium row-level membership findings fixed before implementation; follow-up low kernel task-31 hash wording, medium task-ledger/handoff, and low stale soundness wording findings fixed; final re-review no findings. Test sufficiency review: initial medium isolated `context_identity_hash()` reuse participation and low imported-theorem exclusion findings fixed; final re-review no findings. Full implementation review: initial medium task-ledger/handoff and generated-formula-id shift documentation findings plus low consistency-API finding fixed; follow-up low planned-test wording finding fixed; final re-review no findings. Source/doc consistency review: initial medium public-constant inventory and low bilingual architecture-audit sync findings fixed; final re-review no findings. `doc/design/spec_coverage_audit.md` unchanged because coverage ownership, traceability status, owner crates, and deferred coverage classification did not change. | `cargo fmt --check` passed; `git diff --check` passed; focused `cargo test -p mizar-vc kernel_evidence_handoff`, `cargo test -p mizar-vc dependency_slice`, `cargo test -p mizar-vc context_identity`, `cargo test -p mizar-vc context_identity_hash_participates_independently_in_slice_and_reuse_key`, and `cargo test -p mizar-vc --test determinism_suite kernel_evidence_reuse_key_requires_handoff_and_tracks_handoff_hashes` passed; `cargo test -p mizar-vc` passed; `cargo clippy -p mizar-vc --all-targets --all-features -- -D warnings` passed; `cargo clippy --all-targets --all-features -- -D warnings` passed; `cargo test` passed. | Rust source/docs task. Adds stable `KernelContextIdentityPayload` rows for local-hypothesis, cited-premise, and generated-VC-fact formula evidence bindings, binds them to the target VC and canonical evidence hash, exposes `context_identity_hash()`, and includes that hash in dependency-slice and proof-reuse identity. Imported axiom/theorem entries stay out of context identity. No kernel calls, SAT solving, ATP backend semantic changes, checker/core semantic changes, source-derived bridge fabrication, proof rows, placeholder runners, `.miz` fixtures, expectation changes, or `doc/spec` edits are added. Kernel-side trusted membership verification is implemented by `mizar-kernel` task 31, and `doc/design/spec_coverage_audit.md` remains unchanged because coverage ownership, traceability status, owner crates, and deferred coverage classification did not change. |

| 29. Imported-statement projection producer side | complete | `83ff33edda6c308018d0d499259631c9160708d3` | Spec/doc review: initial medium imported-statement algorithm/dependency-slice findings, follow-up medium ledger-row and low public-constant/plan/stale-wording findings fixed; final re-review no findings. Test sufficiency review: medium unsupported projection formula algorithm, direct canonical/debug projection visibility, mapped-fingerprint slice coverage, and import-unknown proof-reuse specificity findings plus low projection-payload assertion finding fixed; final re-review no findings. Full implementation review: medium same-requirement conflicting projection finding fixed; final re-review no findings. Source/doc consistency review: initial high missing task row, medium acceptance/public-constant findings, and low dependency-slice coverage finding fixed; final re-review no findings. `doc/design/spec_coverage_audit.md` unchanged because coverage ownership, traceability metadata, owner crates, and deferred coverage classification did not change. | Focused `cargo test -p mizar-vc kernel_evidence_handoff::tests::imported_premise_requires_formula_context`, `cargo test -p mizar-vc dependency_slice::tests::imported_statement_projection_participates_in_slice_and_reuse_key`, `cargo test -p mizar-vc kernel_evidence_handoff`, `cargo test -p mizar-vc dependency_slice`, and `cargo test -p mizar-atp imported` passed. Crate/broad verification passed: `cargo fmt --check`; `cargo test -p mizar-vc`; `cargo test -p mizar-atp`; `cargo clippy -p mizar-vc --all-targets --all-features -- -D warnings`; `cargo clippy -p mizar-atp --all-targets --all-features -- -D warnings`; `cargo test -p mizar-kernel`; `cargo test -p mizar-test`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test`; `git diff --check`. | Rust source/docs task. Adds `IMPORTED_STATEMENT_FINGERPRINT_ALGORITHM_ID` and `KernelImportedStatementProjection`, validates imported-statement fingerprints separately from kernel formula-tree fingerprints, rejects unsupported algorithms, stale/mismatched/empty/noncanonical/conflicting projections, and records projection data in canonical evidence/debug/hash input. Dependency slices include imported-statement projection data and prove statement/formula changes and their canonical projection payload affect slice fingerprints while conservative imported coverage still blocks proof-reuse keys. Affected mizar-atp tests now use imported-statement algorithm ids and shared projections for duplicate source tuples. Kernel-side trusted F6 validation/pass fixtures are implemented by `mizar-kernel` task 33. No SAT solving, kernel calls, checker/core semantic changes, source-derived bridge fabrication, `.miz` fixture activation, expectation rebaseline, `doc/spec` edit, or `doc/design/spec_coverage_audit.md` edit is added. |

| 30. Source-derived VC integration contract and exhaustive task decomposition | complete | `d0e2f3a791647c38b9222e64846ac17fcfc4490d` | Spec/doc review: exact Task-180 mapping, exhaustive ownership, both-style functor correctness, concrete VC-37/39 trace-decoration targets, bounded VC-53 admission-authority `spec_gap`, and explicit-handoff-only `GeneratedSethood` findings fixed; final re-review no findings. Test sufficiency review: per-family positive/zero/negative requirements, exact Task-31 admission/trace contract, expanded VC-33 branches, nonempty blocked VC-40 integration, VC-53 no-fabrication boundary, and generated-sethood near misses fixed; final re-review no findings. Full implementation-scope review: invented simplification-order/partial-termination/ghost-erasure VCs, overlapping definition owners, missing non-template `qua`/direct-template owners, trace naming, and task-mixing risks fixed; final re-review no findings. Source/doc consistency review: ledger/bilingual/top-level/Core-current-state drift, VC-53 rollup classification, admission-vs-transport wording, Japanese traceability inversion, and historical deferred/no-candidate wording fixed; final re-review no findings. `spec_coverage_audit.md` changes are required for follow-up ownership/classification only and add no coverage credit. | Focused VC/Core/checker/test lint-policy, mizar-test metadata, and mizar-vc deferred-family documentation-contract tests passed. `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and final workspace `cargo test` passed; two transient parallel mizar-test temp-directory failures passed individually and in the 272-test lib rerun, and the separate documentation-contract failure was fixed before the final workspace pass. Exact preservation oracles: CLI hashes `0915fed1465c86f4b4d0420a35703fe93aed0cbb23b7304abff927195b4f5758` / `57d0fba9be95644890b80bfa4ec2cd992e47bb8ad4b67c130f5194ea73aa0273` / `08b00a9f6fe70d94fe2c1b2bdebbdb5603bcee39bf3ceb460abe53f403bba7b5` / `1dadbeabb219f5853c713ad53aa1cc7cd720a0e80abd7f882e9e0a5ea7802625`; active 96/4/188; plan 403/368; type 236/224; pass/fail 219/184; warnings/errors 23/0; raw/normalized 272-test-list hashes `5e41e4dbfcc303322c246a612de61926a628957a168589b45864d0a5070bb07e` / `c0c2b80f8b4e6c84cd25d77573fda722c4d1846fed168cd4a478781cdb42775e`; production 17 paths/19,803 lines with path/content hashes `b36d96fed3207b415c95de27be11ade57654c6573a2f0637aa2d0a3d56aca01d` / `5f9e716169964a861b71576957c05e2dc2538b5e0ff9d1025ef51a4bea6aa306`; `git diff --check` and explicit staged `git diff --cached --check` passed. | Docs/traceability-only. Adds the paired `source_vc_decomposition.md`, freezes exact VC Task-31/`MT10-VC-T180` authority, assigns bounded VC or zero-VC integration Tasks 32-55 and `MT10-VC-PV/VC<n>` consumers, and changes exactly two existing trace `deferred_reason` strings. Adds no Rust, `doc/spec`, `.miz`, expectation, snapshot, runner, trace requirement/status/test, case, coverage credit, or behavior. VC 40 remains unavailable until VC 37/39 plus Core 40/A1; VC 53 remains unavailable behind its bounded canonical-authority `spec_gap`; missing S1 roles remain unavailable. |

| 31. Exact Task-180 source-to-VC integration | complete at task commit | pending self-hash; verify and backfill from the next bookkeeping commit | Spec/doc review: initial exact-provenance, output-postcondition, current-state, and unsupported downstream-gate findings fixed; final re-review no findings. Test sufficiency review: initial provenance, corruption-matrix, runner-failure, trace-isolation, residual-table, and outcome near-miss findings fixed; final re-review no findings. Full implementation review: initial provenance-grammar, output-postvalidation, and corruption-matrix findings fixed; final re-review no findings. Source/doc consistency review: initial medium missing-ledger, global-priority, and prospective-trace `design_drift` findings fixed; final re-review no findings. `doc/design/spec_coverage_audit.md` is updated because one exact source/spec/test/trace owner becomes covered; the broad proof-verification row and every other deferred family retain no credit. | Focused exact Task-180 adapter, Task-31 runner, repository metadata, and snapshot-path tests passed. `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and workspace `cargo test` passed. Plan/parse/declaration/type/proof CLI hashes are `572873f4f678d446b5b383c3a466bd657df218590b088f1d32b10a98c87ce6ae` / `57d0fba9be95644890b80bfa4ec2cd992e47bb8ad4b67c130f5194ea73aa0273` / `08b00a9f6fe70d94fe2c1b2bdebbdb5603bcee39bf3ceb460abe53f403bba7b5` / `1dadbeabb219f5853c713ad53aa1cc7cd720a0e80abd7f882e9e0a5ea7802625` / `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`; plan 404/369, active 96/4/188/1, proof coverage 4/1, pass/fail 220/184, warnings/errors 23/0. Raw/normalized 276-test-list hashes are `967495e78e1068f592e64834ea3ffb9eac9c25692ea5cbd4f11006a679c66590` / `1be4ae09188b27a40814adc6597de4806dabb13bcac019b294154e1455072adf`; production is 18 paths/20,085 lines with path/content hashes `63e4e770b0d10872415548410d417071c1901f3ffa5aea964a81d2dbbc572ed0` / `a7745e222032a5b6dfeda5ec7a90888c569270134d316166914c959a1684c14c`; final diff checks are required before commit. | Rust/source-corpus/docs task. Adds one public borrowed exact adapter over private `generator/task180.rs`, the distinct proof-verification source/sidecar and complete `VcSet` baseline, one exact runner/tag/guard, and one narrow covered trace row. It accepts only the marker-free structural Task-180 CoreIr/empty CFG/singleton ExistingCore handoff and emits exactly one dense Open `TerminalProofGoal` with an incomplete anchor missing only `CanonicalGoalHash`. No broader theorem/VC family, discharge, `NeedsAtp`, ATP/kernel/proof acceptance, fact publication, existing type-sidecar reclassification, expectation rebaseline, or Steps 6/7 promotion is included. |

Task 31 commit preparation completed both required diff gates: `git diff
--check` and the explicit staged `git diff --cached --check` passed.

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

## Current STEP 5 Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue autonomous STEP 5 development after the completed VC Task 31 commit.
First verify that commit from `git log`, require a clean worktree, and fresh-
inventory the top-level todo plus the checker/Core/VC decomposition documents.
No mizar-vc Task 32-55 is dependency-ready: VC 32 first requires Core 33-35/37,
and Core 33 first requires checker Task 248. The top-level TODO also authorizes
independent parser Tasks 47-48 and resolver Task 31, so inventory all of them
without asserting a unique global priority. If continuing the Core-33-to-VC-32
chain, select checker Task 248 as its next prerequisite and freeze its exact
source-item/declaration-site/local-scope/ordinal/reserve/default/`BindingEnv`
payload contract, existing resolver inputs, `mizar-test` Task-10 consumer,
forbidden type/RHS/proof/global-resolution scope, tests, coverage impact, and
exit criteria before editing. Do not start Core 33 or VC 32 early, invent a
payload, promote Steps 6/7, or repair any `repo_metadata_conflict`.
```

Rationale: the next safe work crosses the source/checker ownership boundary and
unblocks several later Core and VC families. Keep `xhigh`; lower reasoning is
appropriate only for a separately authorized documentation-only typo fix.
