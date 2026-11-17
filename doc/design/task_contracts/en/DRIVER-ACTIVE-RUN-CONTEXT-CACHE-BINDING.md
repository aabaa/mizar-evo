# Task DRIVER-ACTIVE-RUN-CONTEXT-CACHE-BINDING: native Disk active-run binding
Canonical language: English; [Japanese pointer](../ja/DRIVER-ACTIVE-RUN-CONTEXT-CACHE-BINDING.md).
Status: frozen scope. Tier: full (new runtime/cache binding boundary).
Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md#task-index). Consumer: [STEP6C](STEP6C-REAL-SERVICE-EQUIVALENCE.md), without completion credit.
Prerequisites: owner approved all [proposal](../../mizar-driver/en/active_run_context_cache_binding.md#owner-q-decisions) Q1–Q6 as A on 2026-11-17; this contract freezes their bounded implementation and Q2-A prerequisite.

## Authority and producer prerequisite

Authority: spec [§23.4](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility); architecture 22 [equivalence](../../architecture/en/22.incremental_verification_contract.md#clean-build-equivalence), [fallback](../../architecture/en/22.incremental_verification_contract.md#cache-miss-fallback), [freshness](../../architecture/en/22.incremental_verification_contract.md#snapshot-scoped-results), [footprints](../../architecture/en/22.incremental_verification_contract.md#dependency-slice-completeness).
Q2-A source: Linux `/proc/self/exe` bytes identify the executed artifact containing the workspace's Cargo rlib component closure. Require OS mapping evidence that participating implementation addresses reside in that opened image. Driver acquires outside pure queries; unsupported linkage/platform or acquisition is unavailable. OS libraries and release/pin authentication are excluded; SourceLoad selects no external backend.
Encoding: BLAKE3 derive-key domain `mizar-driver/running-native-image/v1` over complete artifact bytes; lowercase digest in versioned Cache compatibility. Built-in construction/selection and registered/actual descriptors bind SourceLoad implementation/schemas; descriptors alone confer no provenance.
Cache retains [exact canonical key comparison](../../mizar-cache/en/cache_key.md#stable-hashing) and [fail-closed validation](../../mizar-cache/en/cache_key.md#fail-closed-rules); include running identity in toolchain compatibility and require exact produced-by correspondence. Different artifacts gain no compatibility; unknown is unavailable. Release/pin authentication remains separate.

## Binding and scope

Driver `assemble_registry(StageConfig)` seals a private immutable association from the policy it installs and actual selected built-ins; submission bootstrap reconstructs the association with protected capture, produced Build plan/index and exact SourceLoad task. Do not expose transferable context or accept caller-created provenance.
Use actual selected package verifier/build settings, typed lock and existing [config/lock comparisons](../../mizar-driver/en/frontend_adapter.md#native-config-content-association); preserve existing precedence and per-package Store routing. `StageConfig.verifier_policy` remains the explicit Proof-owned invocation policy supplied to configured policy consumers and cache validation; do not invent a config-to-policy mapping. Conflicting cache-option policy withholds SourceLoad optimization.
SourceLoad consumes no execution verifier policy, backend or artifact dependency; its conservative policy key remains separate from config identity. The association covers only its exhaustive zero-parent footprint and existing owner output/key/storage/checker schemas. Unknown later footprints are not empty.
Keep [key purity](../../mizar-driver/en/registry.md#cache-key-purity-contract), [capture](../../mizar-driver/en/request.md#native-source-capture), existing live lane/snapshot/cancellation/publisher/physical-handle checks and scoped retention. Reconstruct per submission; no atomic Store/currentness promise. Missing association withholds this profile's optimization, preserving ordinary outcomes and invalid-input/producer-error precedence.
Direct SourceLoadInputs projection/encoding/lookup APIs retain limited claims; active Driver cache preparation requires bootstrap provenance. Existing retained Frontend behavior grants no cold producer or new phase authority.
Writes: Driver registry/bootstrap/source-cache code, existing Driver/Cache owner test modules, this contract pair and Driver plan rows; paired Driver registry/driver/frontend-adapter/proposal boundary documents only as needed. Build/Proof/Cache/IR authority stays with its owner; no new public context API.
Forbidden: spec, .miz, expectations, trace/audit/coverage-credit, AGENTS/protocol/global TODO edits; new policy/precedence/compatibility equivalence; release attestation; cold Frontend/import/proof/witness/reuse activation; full STEP6C completion. New semantic choices stop for owner decision.

## Test matrix and exit

Six cases (reuse existing native_disk/misc/explicit/native_toolchain and Cache/IR matrices):
1. Genuine bootstrap/capture/selected built-ins, actual cold Store insertion and fresh warm SourceLoad hit equal independent clean bytes/diagnostics/outcomes; vary workers/order and reconstruct submission association.
2. Independently viable source/module/package/snapshot/plan/config/lock/policy mismatches and cross-run swaps withhold reuse; unchanged ordinary execution preserves effective policy.
3. Genuine running-image determinism, encoding and exact compatibility; execute a genuinely different supported artifact against an otherwise viable record and require a miss. Unknown/spoofed service/schema/provenance or unsupported linkage refuse; labels/hashes cannot replace provenance or later dependency closure.
4. Cancellation/supersession/publisher replacement/collection and retained readers preserve existing live guards and release acquired owners on success/refusal/error; no stale publication authority.
5. Disabled cache, unsupported origin/profile, missing producer and real lookup/decode/Store failures preserve miss/error precedence and viable ordinary execution; invalid inputs are not repaired.
6. Frontend cold expectations/imports/proof/reuse and STEP6C stay gated; context alone grants no acceptance/publication/completion. Legacy direct API and registered NoKey controls remain.
Independent specification/test-intent review before freeze, then sufficiency, precision, volume/scope and consistency/final reviews. Extra cases require an authority clause or .miz test.
Freeze: both lint_policy suites, links/fragments, EN/JA parity, diff check. Implementation: fmt, all-target/all-feature Clippy with denied warnings, both lint_policy suites, Driver owner tests, Cache tests and short proof-verification E2E; affected lifecycle/identity matrices run, unrelated heavy groups defer under cadence.
Full cargo test requires a cadence trigger. Exit: freeze then implementation, two local commits, one changed task; no push. Documentation cap: 200 added lines; contract cap: 60 lines/660 words.
