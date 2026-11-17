# Task STEP6C-REAL-SERVICE-EQUIVALENCE: equivalence over real phase services
Canonical language: English; [Japanese pointer](../ja/STEP6C-REAL-SERVICE-EQUIVALENCE.md).
Status: in progress. Tier: full. Owner: [build plan](../../mizar-build/en/00.crate_plan.md); consumers: [driver plan](../../mizar-driver/en/00.crate_plan.md), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: at least one real phase service after SourceLoad registered as available (Frontend is); later services extend the same gate as they land.
Readiness: NoKey and bounded reflexivity comparisons exist. Automatic Disk SourceLoad, [one-leaf](./DRIVER-DEPENDENCY-AWARE-FRONTEND-CACHING.md) and [cross-package intermediate caching](./DRIVER-CROSS-PACKAGE-INTERMEDIATE-FRONTEND-CACHING.md) supply the bounded producers; broader cache, unregistered services and Proof reuse/dependency-binding remain deferred.
Authority: [architecture 22](../../architecture/en/22.incremental_verification_contract.md) (IV-002, IV-003); [incremental/parallel equivalence](../../mizar-build/en/incremental_parallel_equivalence.md).
Gap: BUILD-G-017 `external_dependency_gap`. Build task 24 and driver task 16 prove equivalence only for the implemented seams, not for real phase outputs.

## Scope
Run the clean/incremental and sequential/parallel equivalence gate over real
workspaces through every phase service registered as available, comparing
published outputs, diagnostics, and cache decisions. Record coverage for
checked phase/policy combinations; retain named cache-producer and missing
service gaps until their real producers exist.
The first slice compares real NoKey edit/rebuild payloads, diagnostics and uncached outcomes.
Broader hit/miss coverage remains required. Reuse replay/capture-order/worker comparisons in
[real source regressions](../../../../crates/mizar-driver/tests/source_load/main.rs).

### Bounded reflexivity edit/clean profiles
Preserve the NoKey twelve-service profile blocked at missing VcDischarge and exact
thirteen-service composition completing real discharge and all tasks. Each compares
theorem/binder renaming and whitespace edits against independent clean workspaces,
worker1/canonical versus worker4/reverse: genuine artifact DATA, validated module/
manifest payloads, FILE witnesses, diagnostics and uncached outcomes. Normalize only
roots and snapshot-local identities; require changed source identity and independent
stores/captures. Preserve stale/currentness and seam gates. The thirteen-service
bridge authenticates concrete types/descriptors and singleton discharge/kernel tasks
without ATP/backend tasks. Nominal admission and spoofed type, descriptor mismatch,
surplus service, duplicate discharge/kernel refusals retain their existing authority.

### Bounded automatic cache-backed edit/clean comparison
Freeze one test-only unit using the existing one-leaf importer profile and genuine
automatic PersistentPrefix writes, never seeded or synthesized records. Two independent
baseline-to-edit families: importer source bytes, and leaf lexical contribution consumed
by the importer. Each starts with an empty Store, observes cold execution, then warm
SourceLoad/Frontend hits without TaskStarted; the edit misses affected keys and executes,
while unchanged SourceLoad/leaf work may hit. Recapture the edited state for warm hits.
Compare every edited/warm result to an uncached run of the same bytes in a separate
workspace, capture registry and empty Store, with distinct SourceIds. Exercise worker1/
canonical and worker4/reverse controls and reversed capture/layout order. Compare genuine
SourceLoad/Frontend canonical payloads, complete keys, line/loading maps, side tables,
parent commitments, diagnostics and session/phase outcomes; normalize only roots and
snapshot-local identities. Check the leaf lexical change reaches importer tokenization.
Pattern amendment: follow full-tier Build28 (`5fa8374ea`) with I=app/main, M=app/leaf, A=core/base.
For A-lexical, M-export and I-source edits, append fresh edited-state warm recaptures
after immediate edits; compare against independent clean outputs with worker1/canonical
and worker4/reverse controls. Require app SourceLoad/Frontend hits without TaskStarted;
core/base executes normally under current Store routing. Case cap: the original two
families plus these three, each with both controls; reuse currentness/refusal tests.
Write scope: native_disk regressions only; no production, spec, trace/audit, coverage
promotion, parsing-avoidance, Proof reuse or full STEP6C credit. Broader cache is deferred.
## Forbidden
Synthetic outputs, event-only comparisons, weakening seam gates or claiming unregistered-service equivalence.
## Tests and exit
Real multi-module edit/clean workspaces, worker controls, hits/misses and stale rejection.
Use protocol pattern gates for the amendment: combined implementation/sufficiency and
volume/scope reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, both lint_policy suites,
Driver/Build and Frontend/Cache owner tests; full `cargo test` only at protocol cadence.
Exit: genuine-cache equivalence for every registered service; BUILD-G-017 retains remaining service/cache-producer gaps.
