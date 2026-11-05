# Task CHECKER269-SET-BINDINGS: authenticated descendant Set names
Canonical language: English; [Japanese pointer](../ja/CHECKER269-SET-BINDINGS.md).
Status: complete. Tier: full. This is a bounded Task 269 increment, not its exit.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md).
Consumer: [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: completed Task 286 receipt and descendant binding transport.

## Authority and boundary
EN04 §§4.4.1, 4.6; EN15 §15.2.2 establish local names, definition order and
lexical visibility. EN04 §4.7.3 conflicts with EN04 §4.4.1 and EN15 §15.11.2
on equality axioms; all FOL encoding, facts and expansion remain deferred.
Existing broad proof-local declaration-gap `.miz` and its sidecar stay unchanged.
Gap: `source_drift`: the authenticated descendant profile has two Equating
rows but the descendant binding environment retains neither name.
The [proof-local owner](../../mizar-checker/en/source_proof_local_declaration.md)
owns the receipt, public transaction and replay boundaries.

## Frozen scope
Add `build_set_bindings` on the existing descendant binding producer; reuse its
handoff, Given-only row table, Typed/Resolved slots and existing errors.
Consume the Task 286 receipt plus sealed resolver proof-occurrence scope.
Associate all declaration and RHS reference roles with original rows; preserve
resolver identities, lexical scopes and declaration clocks in the new path.
Retain the sealed scope privately for independent installation/replay validation.
The old `build` and its two-binding environment remain unchanged.
The new path has four bindings and three contexts; its Given-only table still
has one row. The descendant context owns only the two new abbreviations.
Types are missing, captures empty; emit no facts, obligations or proof results.
Ship the Equating original-row rule and its ledger row together; SetStatement
remains Task 272. Keep unsupported names/shapes and unrelated donors refused.
No specification, existing fixture/expectation, trace-status or coverage-credit edits.
Task 269 stays open for remaining introductions and closure/capture consumers.

## Verification and exit
Existing owner tests plus runner tests cover canonical and genuine layout sources,
source-order lookup, scope exclusion, foreign authorities, coherent corruption,
atomic Typed installation and Resolved replay, and unchanged semantic tables.
Type/term donors keep their existing two-binding restriction.
Run fmt, Clippy, checker owner/type/term regressions, runner proof-local tests,
type/formula corpus and documentation/ledger checks, with a short executable case.
Unchanged evidence/cache heavy matrices defer to periodic/integration cadence.
Full regression follows protocol cadence; this bounded increment alone is no trigger.
Require independent specification, test-sufficiency, implementation, volume/scope,
and consistency reviews under the full-tier protocol.
Exit: the real private Task-10 runner consumes the new producer and replays it;
Equating is done in the ledger; old descendant output and protected artifacts persist.
