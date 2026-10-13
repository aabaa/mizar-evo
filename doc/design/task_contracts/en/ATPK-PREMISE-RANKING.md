# Task ATPK-PREMISE-RANKING: deterministic premise ranking
Canonical language: English; [Japanese pointer](../ja/ATPK-PREMISE-RANKING.md).
Status: planned. Tier: full. Owner: [atp plan](../../mizar-atp/en/00.crate_plan.md); consumers: [resolve plan](../../mizar-resolve/en/00.crate_plan.md) (unresolved citations), [artifact plan](../../mizar-artifact/en/00.crate_plan.md) (`missing_facts.json`), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: an index of the facts visible at a goal (module summaries and imports); [VC-owned pretranslation assembly](../../mizar-atp/en/translator.md#planned-premise-selection-boundary).
Readiness: Ranking over authenticated inputs can start; complete integration is blocked on visible-fact indexes, source obligations and suggestion artifacts.
Authority: specification §22.4.4, §21.4.1 (`max_axioms` pruning), §23.5.2 (`missing_facts.json`, `ranking` field); [architecture 09](../../architecture/en/09.atp_interface_protocol.md).
Gap: `test_gap` and `source_drift`; no verifier-side ranking exists.

## Scope

Implement the `symbol-overlap-v1` ranking: relevance grows with the goal
symbols a fact shares, weighting rare symbols more, with a total deterministic
tie-break. Use it to prune assembled facts under `max_axioms`, to rank library
facts for lemma suggestions of failed or open obligations (with the optional
retry), and to rank labels for unresolved citations by spelling similarity and goal relevance. Record the configuration
name with every ranked result.

## Forbidden

Accepting a source proof using an uncited fact; learned ranking (a later
configuration); nondeterministic scores; changing proof acceptance. A
configured diagnostic retry may add suggested candidates, but the source
citation is required before acceptance.

## Tests and exit

Fixed goals and fact sets with expected orders, tie-breaks, pruning at the
budget, suggestion lists with and without retry, retry success that cannot
accept an unchanged source proof, and unresolved-citation
candidates ranked by both similarity and relevance. Include failed pruning with E0321; failed retry as hints versus successful retry as a suggestion, with the resolving candidate first in the artifact and matching IDE fix. Corrupt visible-fact, source-provenance and configuration identities must fail closed. Require independent specification, test-sufficiency,
implementation, volume/scope, and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: every ranking use in §22.4.4 is served by the recorded configuration,
and no accepted source proof gains an uncited fact.
