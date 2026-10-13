# Task ALGDEF-VC59-IMPORTED-PROMOTION-FACTS: imported promotion axioms
Canonical language: English; [Japanese pointer](../ja/ALGDEF-VC59-IMPORTED-PROMOTION-FACTS.md).
Status: blocked-reserved. Tier: full. Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md); consumers: [proof plan](../../mizar-proof/en/00.crate_plan.md), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: ALGDEF-VC57-PROMOTION-FACTS; ALGDEF-IMPORT-AUTHORITY reviewed boundary and implemented authenticated transferable applicable E1/E2/E4/group acceptance for an exported algorithm.
Readiness: dependency-blocked on transferable verified-status authority and authenticated imported inputs.
Blocker: genuine transferable applicable E1/E2/E4/group evidence and its import validators are not yet available. ALGDEF-IMPORT-AUTHORITY owns inventory, validator/producer design and exact spec_gap release conditions if needed; adoption remains blocked until genuine outputs exist. Do not invent authentication rules.
Authority: specification §20.13.2 promotion axioms, §16.5.1 citation, §12 imports, §23 artifacts.
Gap: `external_dependency_gap`.

## Scope

When the authority exists, expose the promotion axioms of an imported
promoted algorithm, including its defining equations when it is definitional,
as premises with provenance, exactly as VC 57 does for module-local ones.

## Forbidden

Treating a returned or public database, artifact field, or summary as proof
authority without the missing authentication rules; re-proving another
module's obligations as a substitute; specification, corpus, or expectation
changes made to match behavior.

## Tests

When unblocked: an importer citing an imported definitional algorithm; an
importer whose dependency lacks authenticated acceptance, contributing no
premise; corruption tests for forged, stale, or mismatched acceptance.

## Coverage, reviews, and exit

Coverage audit: the chapter 20 row records this reserved ownership. Require
independent specification, test-sufficiency, implementation, volume/scope,
and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: imported promotion axioms are available exactly after authenticated
transferable acceptance and never before.
