# Task ALGDEF-VC57-PROMOTION-FACTS: module-local promotion axioms as premises
Canonical language: English; [Japanese pointer](../ja/ALGDEF-VC57-PROMOTION-FACTS.md).
Status: planned. Tier: full. Owners: [VC plan](../../mizar-vc/en/00.crate_plan.md) (premise exposure) and [proof plan](../../mizar-proof/en/00.crate_plan.md) (module-local acceptance); consumers: [resolve plan](../../mizar-resolve/en/00.crate_plan.md), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: ALGDEF-CORE54-DEFINING-EQUATIONS; VC 52 termination obligations; ALGDEF-RESOLVE33-ALGORITHM-REFERENCES.
Precedent: the bounded [source existential-registration proof](../../mizar-proof/en/status.md#source-existential-registration-proof) freshly proves obligations inside one authenticated source orchestration, checks kernel evidence itself, and activates its result module-locally. Imported (transferable) acceptance is ALGDEF-VC59-IMPORTED-PROMOTION-FACTS.
Authority: specification §20.7.2 and §20.13.2 promotion axioms, §16.5.1 citation of promoted algorithms, §10.12 definitional facts; [architecture 07](../../architecture/en/07.vc_generation.md#definition-unfolding-is-controlled).
Gap: `test_gap` and `source_drift`; no producer accepts termination or exposes promotion axioms.

## Scope

Within one authenticated source orchestration, freshly prove the termination
obligations of an algorithm declared in the same module, following the
precedent, and on genuine acceptance expose its totality, contract, and (for a
definitional algorithm) defining-equation axioms as premises with provenance,
under the definition expansion policy, when a later VC cites the algorithm
name or the policy makes them available. Item order is preserved and no fact
is retroactive.

## Forbidden

Availability before acceptance; deriving acceptance from obligation presence,
order, or local checking; returning the accepted state as a transferable
receipt; imported algorithms (VC 59); kernel or trust-policy changes; fact
publication outside the artifact path; specification, corpus, or expectation
changes made to match behavior.

## Tests

Test-first real sources: `FactorialZero` and `FactorialStep` from §20.7.2
proved `by factorial`; a citation whose termination proof fails, contributing
no premise; a non-definitional algorithm contributing only totality and
contract. Corruption tests cover forged or stale acceptance, reordered items,
and mismatched provenance.

## Coverage, reviews, and exit

Coverage audit: the chapter 16 and 20 rows record this ownership of the
§20.13.2 premises. Require independent specification, test-sufficiency,
implementation, volume/scope, and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: module-local promotion axioms are available exactly after genuine
in-orchestration acceptance and never before.
