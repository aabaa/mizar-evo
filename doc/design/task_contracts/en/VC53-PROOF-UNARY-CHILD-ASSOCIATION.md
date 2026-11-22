# Task VC53-PROOF-UNARY-CHILD-ASSOCIATION: arena-local unary child association
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-UNARY-CHILD-ASSOCIATION.md).
Status: frozen bounded association. Tier: full; new Proof consumer and association boundary.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependency: [authenticated existential producer](VC53-UNARY-CHILD-EXISTENTIAL.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1](../../../spec/en/20.algorithm_and_verification.md), [§16.7.3](../../../spec/en/16.theorems_and_proofs.md#1673-skeleton-encoding).
Owner APIs/invariants: [Proof status](../../mizar-proof/en/status.md#arena-local-unary-child-request-association), [VC descriptor](../../mizar-vc/en/generator.md#bounded-unary-child-request-descriptor), [Kernel result](../../mizar-kernel/en/checker.md#original-algorithmreturns-existential).
Gap: source_drift/test_gap; Kernel checks caller/Apply/Return/result-slot joins but its opaque result retains only callee relation, witness, guard and borrowed Core. No language-intent gap is demonstrated.

## Frozen scope and design
Add `SourceUnaryChildCallAssociation` in Proof status.rs, with private whole existing `SourceUnaryChildCallRequest`, existing `(LocalContext, [VcGeneratedFormula; 2], CoreTermId)` descriptor and same-call borrowed `AlgorithmReturnsExistentialCheck`. This opaque holder is required to prevent reconstruction of an association from detached callee-only result data; it introduces no parallel payload representation.
`from_source(source, typed, symbols, core, request)` calls the unchanged VC checked-result producer once, preserves upstream errors, and retains the exact request clone and returned descriptor/result atomically. Immutable `request`, `descriptor` and `result` getters expose only the existing values.
`matches_source` takes the same five inputs. FIRST regenerate the existential descriptor with the unchanged authenticating VC generator, preserving errors. Return true only for whole request/descriptor equality and pointer-identical Core to the retained result. Immutable original result plus complete authenticated equality retains caller, Apply, Return, formal/actual, both result slots, source and CFG joins. A structurally equal cloned Core returns false after authentication. Mismatch returns false; failed authentication remains an error. Source/environment/filesystem currentness remains caller-owned.
Accept only the existing genuine two-algorithm unary object profile; optional terminating intent grants nothing. No detached result/descriptor constructor, mutation, hash-only association or partial projection comparison.
Excluded: current-policy consumption, native adoption, canonical ProofObligationIdentity/reuse, caller E4 acceptance, E1 discharge, generic substitution, broader profiles, evidence/status/promotion/publication/coverage credit and owner-gated scope. No language intent, public diagnostic, AGENTS/protocol/spec/.miz/expectation/trace or audit-status changes.
Files: Proof status.rs, existing runner algorithm tests, paired Proof status/plan documents and this contract. Lower-stage production remains unchanged. Existing direct-return and child producer boundaries remain intact.

## Test matrix and exit
Cap: five families, reuse the genuine existing unary fixture; establish one successful association/recheck before refusal expansion.
1. Genuine pair: compare whole retained request and independently generated descriptor; inspect same-Core Kernel target/witness/guard/schema and all caller/Apply/Return/result-slot joins. Repeat recheck; inputs and retained products remain immutable.
2. Existing rename/trivia, equal-spelling distinct-scope and optional-terminating fixtures preserve genuine association and determinism; fresh authenticated changed source/Core/request cannot match an old association.
3. Whole request and Core drift refuse through lower authentication, including valid wrong caller/Apply/Return/formal/actual/result IDs; preserve old association usability. Same-content cloned Core authenticates but returns false.
4. Foreign/stale source, typed or environment preserve exact lower errors before association mismatch; calibrate genuine lower outcomes and one precedence case. No fabricated positive check/result/AST/environment fixture.
5. Existing direct-return/descriptor/Kernel controls remain valid; no new VcSet, policy, identity, adoption, status or acceptance path.
Independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews required.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, focused association regressions, algorithm/source-return groups and short proof-verification E2E. Association obligations run immediately; unchanged policy/evidence/cache/acceptance/edit-versus-clean heavy matrices defer to orchestrator integration. Full cargo test only on cadence; lane dispatch has no full/push trigger.
Exit: complete immutable authenticated request/descriptor association and fail-closed same-arena recheck, with refusal matrix; no caller normal-return acceptance.
