# Task TEST-METAMORPHIC-BRIDGE-GENERALIZATION: generalize source bridge extraction
Canonical language: English; [Japanese pointer](../ja/TEST-METAMORPHIC-BRIDGE-GENERALIZATION.md).
Status: in progress. Tier: full. Owner: [test plan](../../mizar-test/en/00.crate_plan.md); consumers: resolver/checker/Core corpus routes.
Dependencies: the metamorphic guard and authenticated producer handoffs for each affected bridge.
Readiness: inventory each bridge before implementation; missing producer support stays explicitly blocked.
Authority: specification §§2.2/2.9, 4.6, 15.10; unchanged active `.miz` cases and expectations.
Gap: `source_drift` in test-runner production bridges: exact source/offset/name gates refuse meaning-preserving variants before the semantic producer is reached.

## Scope and boundaries

Own the named exceptions in `tests/coverage/metamorphic_exceptions.tsv`:
proof-label confinement dispatch; attribute, structure, mode, predicate,
functor and property-definition extraction; predicate chains, nested
quantifiers, grouped/connective formulas and bound uses.
Replace fixture-byte gates with source-authenticated structural extraction,
using existing producer authority; preserve malformed/foreign-input refusal.
Coordinate functor lower-profile changes with the existing exact-profile
retirement task rather than widening a lower handoff in the runner.
Do not alter specification, original corpus/expectations, admission, trace
status, semantic credit or acceptance policy. Remove an exception only when
its exact variant passes; a missing producer is not permission to fabricate it.

## Verification and exit

Require independent specification, test-sufficiency, implementation,
volume/scope and source/documentation consistency reviews. Run each affected
original corpus and metamorphic variant, malformed/foreign negative controls,
`cargo fmt --check`, Clippy with all targets/features and warnings denied,
and verification cadence including affected long obligations.
Exit: every owned exception is removed through genuine producer execution;
remaining lower prerequisites retain a named owner and explicit blocking reason.
