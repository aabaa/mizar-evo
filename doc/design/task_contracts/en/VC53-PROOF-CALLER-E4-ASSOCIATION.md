# Task VC53-PROOF-CALLER-E4-ASSOCIATION: arena-local caller E4 association
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E4-ASSOCIATION.md).
Status: frozen bounded association. Tier: full; new Proof public type/API fails the pattern criterion.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [checked caller composition](VC53-CALLER-E4-COMPOSITION.md), [child association](VC53-PROOF-UNARY-CHILD-ASSOCIATION.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1](../../../spec/en/20.algorithm_and_verification.md), [§16.7.3](../../../spec/en/16.theorems_and_proofs.md#1673-skeleton-encoding).
Owner invariants: [Proof association](../../mizar-proof/en/status.md#arena-local-unary-child-request-association), [caller descriptor/composition](../../mizar-vc/en/generator.md#bounded-unary-caller-e4-descriptor), [Kernel child rule](../../mizar-kernel/en/checker.md#unary-child-existential-instantiation).
Gap: source_drift/test_gap; authenticated caller descriptor and same-Core checked composition exist, but Proof lacks whole caller request retention/recheck. No language-intent change is required.

## Frozen design and boundaries
Add opaque `SourceUnaryCallerE4Association` in Proof status.rs. Private fields retain only the whole existing `SourceUnaryChildCallRequest`, caller `(LocalContext, [VcGeneratedFormula; 2])` descriptor and same-call borrowed `AlgorithmReturnsExistentialCheck`. Immutable `request`, `descriptor` and `result` getters expose these existing values.
`from_source(source, typed, symbols, core, request)` obtains genuine child evidence from the unchanged VC checked-child producer, then invokes the unchanged checked-caller composition on the same inputs and evidence. Preserve upstream errors; clone the request and retain the returned caller descriptor/result atomically. Child evidence/descriptor are temporary, never retained.
`matches_source` takes the same five inputs. FIRST regenerate the unchanged authenticated caller descriptor, preserving errors before mismatch. Return true only for whole request/descriptor equality and pointer-identical Core to the retained result. Equal cloned arenas return false after authentication. Private construction and immutable whole data preserve original caller/CFG/Return/Apply/callee/actual/formal/both result-slot joins; no detached constructor, partial comparison, hash-only receipt or repeated Kernel check. Currentness remains caller-owned.
Accept only the existing genuine two-algorithm unary object profile; optional terminating intent grants no promotion. Keep caller and child identities distinct: this holder has no canonical identity or child-identity reuse.
Excluded: canonical caller identity, policy consumption/native adoption, E1/E2/whole acceptance, generic evidence/status/publication, VcSet/seeds/scheduling, complete footprints, cache admission and owner-gated scope. Preserve raw unknown/incomplete/cache-miss boundaries. No language intent, public diagnostics, lower-stage production, spec/.miz/expectation/trace/audit-status or AGENTS/protocol edits.
Files: Proof status.rs, existing runner algorithm tests, paired Proof status/plan documents and this contract. spec_coverage_audit coverage/owner/deferral remain unchanged.

## Test matrix and exit
Cap: five families; reuse the established genuine unary fixture and pass one successful caller association/recheck before expanding.
1. Genuine pair: compare whole retained request and independently generated caller descriptor/composition result, exact caller relation/flow/Return/input/fresh binder, child actual witness, guard/schema and original joins. Repeat recheck; all inputs and retained products remain immutable.
2. Existing rename/trivia/equal-spelling and both terminating variants, repeat/fresh associations and changed-source success. Freshly authenticated changed sources and equal cloned Core cannot match an old association; fresh cloned-Core association succeeds.
3. Reuse whole request/Core drift matrices, including valid wrong caller/Apply/Return/formal/actual/result IDs. Calibrate exact lower authentication errors for constructor and recheck; original association recovers after refusals.
4. Reuse calibrated foreign/stale/forged source, typed and environment plus combined drift; exact upstream errors precede association mismatch. No fabricated positive evidence/result/AST/environment.
5. Existing direct-return/descriptor/Kernel/child Proof identity controls remain valid, with distinct child target and unchanged raw unknown/incomplete/cache-miss behavior; no new policy, identity, adoption, status or acceptance path.
Independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews required.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, algorithm/source-algorithm regressions and short proof-verification E2E. Changed association/authentication matrices run immediately; unchanged heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean groups defer to orchestrator integration. Full cargo test only on cadence; lane dispatch has no full/push trigger.
Exit: immutable authenticated caller association and fail-closed same-arena recheck; no caller policy/adoption or whole acceptance.
