# Task PROOF-SOURCE-CANONICAL-COMPONENTS: bind genuine source content
Canonical language: English; [Japanese pointer](../ja/PROOF-SOURCE-CANONICAL-COMPONENTS.md).
Status: implemented. Tier: full. Owner: [Proof plan](../../mizar-proof/en/00.crate_plan.md), task 32; consumers: future source anchor and proof-reuse assembly.
Authority: spec §§6.2/17.1–17.3; architecture07 context preservation and architecture18/22 canonical content, identity split and fail-closed reuse; [sole API/order/trust/test owner](../../mizar-proof/en/status.md#source-canonical-components).
Dependencies/readiness: genuine source checker/Core/VC generation, Resolver36 arena owner content, Core55 selected formula content and VC65/66 canonical components exist.
Gap: source_drift/test_gap; the content producers expose caller-supplied opaque associations, with no production same-call source assembly.
Scope: one fresh Proof facade returns existing canonical VC/goal/context values from the same source/typed/env inputs through unchanged lower-stage producers. No caller Core, graph, payload map, identity, check result or accepted flag is admitted.
Write scope: additive status.rs function only, append-only runner tests reusing genuine setup, paired owner/plan/TODO and contracts. All existing source facade bodies and lower-stage APIs/bytes remain unchanged.
Tests: three genuine families for independent exact leaf components and existing producer equality, neutral/current-key/participating edits and goal/context separation, foreign/malformed/unsupported inputs, absent selection, upstream error order and immutable API inputs.
Forbidden: new type/field/adapter/file/dependency/hash profile, shared one-use setup or encoder, protected spec/.miz/expectation/trace/ledger changes, proof-policy/acceptance/status/cache/publication changes or fabricated positive data.
Deferrals: caller source/snapshot currentness; normalized full anchor, composed dependency footprint, complete Proof identity, accepted evidence, trusted metadata tokens and witness/runtime integration remain separate.
Reviews: independent SPEC/TEST/CODE/VOLUME/CONSISTENCY and task-bounded full gates; narrow then affected Proof/Resolve/Checker/Core/VC/Cache/MizarTest checks; parent cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: selected bounded content comes from genuine same-call producers without conferring accepted status, complete identity, reuse eligibility or publication authority.
