# Task STEP5C11-CLUSTER-REGISTRATIONS: cluster source bridge

Canonical language: English; [Japanese pointer](../ja/STEP5C11-CLUSTER-REGISTRATIONS.md).
Owning plans: [checker](../../mizar-checker/en/00.crate_plan.md#task-index),
[resolve](../../mizar-resolve/en/00.crate_plan.md#task-index),
[test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

- Status: partial; full tier; checker owns semantics; resolve owns bindings.
- Dependencies: 5A.2, 5B.2, 5C.3 and 5C.10 bounded work complete.
- Authority: [Chapter 17](../../../spec/en/17.clusters_and_registrations.md)
  §§17.1,17.3–7,17.8.3,17.9–10; Chapters [3](../../../spec/en/03.type_system.md),
  [6](../../../spec/en/06.attributes.md), [10](../../../spec/en/10.functors.md).
- Cases: the seven 5C.11 rows in the [activation map](../../../../tests/coverage/step5_activation_map.tsv).
  Restricted-adjective parse rejection is active; current increment adds only the
  four positive rows at unchanged advanced_semantics/cluster_resolution endpoints.
- Classification: source drift / executable test gap in pending registration intake.
  The two proof failures await real source/Core correctness checking; the reduction
  negative also awaits reconciliation with §17.6.4's strict size decrease.

## Scope and boundaries

Authenticate complete SurfaceResolvedArena/TypedArena/source/module correspondence,
resolver registration/symbol/declaration provenance, source order and recovery.
Resolve definition/registration parameters to existing declaration identities.
Check builtin set parameter/result types, attribute subject signatures and equality
bodies, identity functor signatures/bodies, actual application arguments, registration
kind/operands and correctness-clause ownership. Unsupported forms fail closed.
Derive reduction sizes and free-variable multiplicities from resolved term structure,
never token counts or case-specific constants; retain the existing strict order.
Add check_source_registration_intake in registration_resolution, returning existing
RegistrationDatabase and TermFormulaInferenceOutput. Consume genuine checked term/
formula/type references and construct each pending request from its §17.8.3 schema:
existential domain/attribute; conditional domain/ordered antecedent/consequent;
functorial parameter AND full result guards with the typed application; reduction
parameter guards and typed equality operands. InitialObligationGoal is a deterministic
owner-bound request key over these checked operands, never a label-only placeholder
or a new string-expression language. Generated binders are registration-owned;
no fabricated source bindings, symbols, facts or fully lowered FOL are claimed.
Use from_symbol_env_with_validation with no activation inputs. Require validated
pending rows, matching pending RegistrationCorrectness obligations and empty activated,
rejected and diagnostic tables. Pending records contribute no inference/effects.
Harness adds real advanced-semantics execution/CLI admission, reusing existing report
objects; it extracts neutral inputs and consumes checker outputs, never decides semantics.
Authenticate exact ids/paths/stage/phase/outcome/keys/tags, reject cross-stage fallback,
and add only each positive sidecar's active_advanced_semantics tag.
Preserve existing profiles, .miz/expectation intent, trace/map/ratchet and Task277B.
Task274 remains blocked for actual accepted effects; Core lowering and proof discharge
are deferred. Owner details: [registration](../../mizar-checker/en/registration_resolution.md),
[names](../../mizar-resolve/en/names.md), [harness](../../mizar-test/en/harness.md).
Audit impact: Chapter17 pending validation only, without acceptance or closure credit.

## Exit

Review specs/docs, tests, implementation, volume and consistency; test all four outputs,
renaming, source/owner/scope/type/guard/operand corruption, reduction order and exact
admission, including no inference from pending rows. Preserve the active parser oracle.
Run focused tests/corpora, fmt, warnings-denied Clippy and cargo test; commit this increment.
Keep the parent task open until all seven unchanged-stage outcomes execute.
