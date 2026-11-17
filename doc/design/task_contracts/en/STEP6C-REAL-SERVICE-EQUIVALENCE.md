# Task STEP6C-REAL-SERVICE-EQUIVALENCE: equivalence over real phase services
Canonical language: English; [Japanese pointer](../ja/STEP6C-REAL-SERVICE-EQUIVALENCE.md).
Status: in progress. Tier: full. Owner: [build plan](../../mizar-build/en/00.crate_plan.md); consumers: [driver plan](../../mizar-driver/en/00.crate_plan.md), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: at least one real phase service after SourceLoad registered as available (Frontend is); later services extend the same gate as they land.
Readiness: NoKey and bounded reflexivity comparisons exist. Automatic Disk SourceLoad and [one-leaf imported Frontend caching](./DRIVER-DEPENDENCY-AWARE-FRONTEND-CACHING.md) provide the producers for the bounded cache comparison below; broader cache, unregistered services and Proof reuse/dependency-binding remain deferred.
Authority: [architecture 22](../../architecture/en/22.incremental_verification_contract.md) (IV-002, IV-003); [incremental/parallel equivalence](../../mizar-build/en/incremental_parallel_equivalence.md).
Gap: BUILD-G-017 `external_dependency_gap`. Build task 24 and driver task 16 prove equivalence only for the implemented seams, not for real phase outputs.

## Scope
Run the clean/incremental and sequential/parallel equivalence gate over real
workspaces through every phase service registered as available, comparing
published outputs, diagnostics, and cache decisions. Record coverage for
checked phase/policy combinations; retain named cache-producer and missing
service gaps until their real producers exist.
The first slice compares edit/rebuild with clean runs of the real NoKey services,
including actual published payloads, diagnostics and uncached outcomes.
Full completion still requires broader cache-hit/miss coverage. Reuse replay/capture-order/worker
comparisons in [real source regressions](../../../../crates/mizar-driver/tests/source_load/main.rs).

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
Case cap: two edit families, each cold/warm/edit/edited-warm with both worker/order
profiles; reuse currentness/refusal tests. Affect source_load/native_disk regressions
only unless a bounded implementation defect is exposed. No spec, trace/audit, coverage
promotion, broader cache, parsing-avoidance, Proof reuse or full STEP6C credit follows.
## Forbidden
Synthetic phase outputs; comparing only event streams; weakening the existing
seam-level gate; claiming equivalence for unregistered services.
## Tests and exit
Real multi-module workspaces with an edit between runs, several worker counts,
and cache hits and misses; stale-snapshot rejection. Require independent
specification, test-sufficiency, implementation, volume/scope, and
consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, both lint_policy suites,
Driver/Build and Frontend/Cache owner tests; full `cargo test` only at protocol cadence.
Exit: the gate holds for every registered real service with genuine cache
behavior, and BUILD-G-017 retains only remaining service/cache-producer gaps.
