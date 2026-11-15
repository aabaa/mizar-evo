# Task REFACTOR-R2-DRIVER-FACADE: driver facade and composition root
Canonical language: English; [Japanese pointer](../ja/REFACTOR-R2-DRIVER-FACADE.md).

## Contract

- Status: frozen; full tier. Primary owner: mizar-driver; consumer: mizar-lsp.
- Purpose: centralize existing stage wiring and submission choices without changing execution semantics.
Owners: [Driver plan](../../mizar-driver/en/00.crate_plan.md).
- Authority: [build lifecycle](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility), [driver responsibility](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md#compiler-driver), [snapshot rules](../../architecture/en/22.incremental_verification_contract.md#snapshot-scoped-results).
- Local boundaries: [registry](../../mizar-driver/en/registry.md#ownership-boundary), [submission](../../mizar-driver/en/driver.md#submit-control-flow), [diagnostics](../../mizar-driver/en/driver.md#diagnostics-boundary), [native cache](../../mizar-driver/en/driver.md#explicit-native-disk-cached-sourceload-scheduling).
- Readiness: R1 is landed; settled OpenBuffer provenance/staging decisions remain binding. No semantic decision is opened.
- Evidence: owner-adopted design audit 2026-11-15, section 7 R2 and findings H3/H1: registry recipes exist only in fixtures; submission names encode feature combinations and private boolean tuples.
- Classification: bounded design/source drift and facade test gap; language/specification coverage and deferral ownership are unchanged.

## Scope and API decisions

- Add `StageConfig` with explicit stage cutoff, artifact roots, existing VC schemas, discharge/ATP/verifier settings, and witness/artifact paths. Provide frontend-only and bounded source-theorem presets matching the existing LSP recipe: snapshot-native FOF, supported equality/native properties, first-order quantifiers, scoped guard predicates, TPTP; default owner policies, snapshot-generation-v1/snapshot-vc-v1, proof-witnesses/snapshot, main.mizir.json, artifacts.
- Add one production `assemble_registry` function, registering only the existing contiguous SourceLoad-through-cutoff adapters; reject unsupported cutoffs and existing invalid transport/path settings. Preserve builder validation, descriptors and phase ownership.
- Add an inline consumer-facing `facade` re-export module for driver/request/registry entry types and owner configuration types needed by these APIs; add no dependency edges or new semantic representations.
- Replace feature-combination submission methods with `SubmitOptions`: import discovery, explicit Disk/native diagnostic mode, named borrowed native-cache settings, and retained Frontend oracle. Keep only draft `submit`, native `submit_captured`, and watch `submit_watch_change` lifecycle entries, each taking the same options type.
- Accept only legacy option combinations: draft has default or discovery; watch has default; captured without cache has default, discovery, Disk diagnostics, native diagnostics, or discovery+native diagnostics. Cached captures have no discovery/diagnostics and choose read-only, configured read-only, persistent source, or persistent prefix; an oracle requires read-only cache. Reject other combinations before allocation/capture or lane mutation with a structured options error.
- Preserve allocation/capture, captured one-shot validation, watch supersession, overlay precedence, cache eligibility/persistence, retained oracle validation, currentness, and structured errors. Defaults reproduce ordinary submission.
- Return optional diagnostic index on `BuildSubmission`; aggregate original same-call phase batches with the selected existing guard before/after, or return None. Keep LSP wrapper return contracts unchanged during migration.
- Migrate workspace callers mechanically, preserving every previous option selection and assertion. Replace the LSP coherent-snapshot fixture's full registry recipe with the production root and explicit old settings.
- Allowed artifacts: driver lib/registry/driver/CLI and their tests; LSP submission call sites and coherent-snapshot fixture; other workspace submission callers only if found; paired driver API/registry/plan inventories and this contract.
- Forbidden: AGENTS/protocol/specification edits; owner-gated scope; R1, Task 9 or later R work; frontend adapter splitting/dead-code removal; Session/IR retention/projection changes; scheduling split, cache-key model, artifact semantics, kernel/ATP execution, dependency prohibitions or protected lower owners.
- General service completion, new accepted inputs, proof/policy choices, live LSP activation and codec restructuring remain deferred to their existing owners.

## Tests and exit

- Reuse unchanged assertions in driver submission/watch/import/diagnostic/native-cache/retained-oracle suites and LSP integration suites after call migration.
- New matrix (cap six cases): frontend preset exposes exactly SourceLoad/Frontend; source-theorem preset matches the existing full recipe including descriptors; explicit configuration matches an independent manual recipe; invalid cutoff/transport/path settings reject; invalid submission combinations reject before mutation while all legacy combinations retain existing assertions; genuine coherent OpenBuffer build through the root preserves publication/currentness/retention assertions.
- Existing test owners: `crates/mizar-driver/tests/registry.rs`, `tests/source_load/{main,current,import,native_disk}.rs`, `tests/watch.rs`, `crates/mizar-lsp/src/server.rs`.
- Reviews: independent specification/test intent, sufficiency, implementation precision, volume/scope, source/document consistency, and final hard-gate/quality assessment.
- Freeze checks: both lint_policy suites, including links/fragments/pairing/volume. Implementation: fmt, workspace all-target/all-feature Clippy with warnings denied, both lint_policy suites, full driver/LSP owner tests and explicit genuine coherent-snapshot E2E.
- Full cargo test follows protocol cadence; unaffected heavy proof/evidence/resolver/cache-key matrices defer to periodic/integration verification. Exit requires focused gates, no unresolved review findings, and local implementation commit; no push.
