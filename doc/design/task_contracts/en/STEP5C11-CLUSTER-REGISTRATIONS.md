# Task STEP5C11-CLUSTER-REGISTRATIONS: cluster source bridge

Canonical language: English; [Japanese pointer](../ja/STEP5C11-CLUSTER-REGISTRATIONS.md).
Owning plans: [checker](../../mizar-checker/en/00.crate_plan.md#task-index),
[resolve](../../mizar-resolve/en/00.crate_plan.md#task-index), [Core](../../mizar-core/en/00.crate_plan.md#task-index), [VC](../../mizar-vc/en/00.crate_plan.md#task-index),
[test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

- Status: partial; full tier; checker owns semantics; resolve owns bindings.
- Dependencies: 5A.2, 5B.2, 5C.3 and 5C.10 bounded work complete.
- Authority: [Chapter 17](../../../spec/en/17.clusters_and_registrations.md)
  §§17.1,17.3–7,17.8.3,17.9–10; Chapters [3](../../../spec/en/03.type_system.md),
  [6](../../../spec/en/06.attributes.md), [10](../../../spec/en/10.functors.md).
- Cases: the seven 5C.11 rows in the [activation map](../../../../tests/coverage/step5_activation_map.tsv).
  Parser rejection and four pending-intake rows are active; next is only functorial false coherence at proof_verification/verification.
- Classification: source/Core correctness-checking drift and executable test gap.

## Scope and boundaries

Authenticate complete SurfaceResolvedArena/TypedArena/source/module correspondence,
resolver registration/symbol/declaration provenance, source order and recovery.
Resolve definition/registration parameters to existing declaration identities.
Check builtin set parameter/result types, attribute subject signatures and possibly negated equality
bodies, identity functor signatures/bodies, actual application arguments, registration
kind/operands and correctness-clause ownership. Unsupported forms fail closed.
Derive reduction sizes and free-variable multiplicities from resolved term structure,
never token counts or case-specific constants; retain the existing strict order.
The shared check_source_registration_intake returns sealed SourceRegistrationCheck with existing
RegistrationDatabase, TermFormulaInferenceOutput and BindingEnv. Consume genuine checked term/
formula/type references and construct each pending request from its §17.8.3 schema:
existential domain/attribute; conditional domain/ordered antecedent/consequent;
functorial parameter AND full result guards with the typed application; reduction
parameter guards and typed equality operands. InitialObligationGoal is a deterministic
owner-bound request key over these checked operands, never a label-only placeholder
or a new string-expression language. Generated binders are registration-owned;
no fabricated source bindings, symbols, facts or proof acceptance are permitted.
Use from_symbol_env_with_validation with no activation inputs. Require validated
pending rows, matching pending RegistrationCorrectness obligations and empty activated,
rejected and diagnostic tables. Pending records contribute no inference/effects.
Harness adds real advanced-semantics execution/CLI admission, reusing existing report
objects; it extracts neutral inputs and consumes checker outputs, never decides semantics.
Authenticate exact ids/paths/stage/phase/outcome/keys/tags, reject cross-stage fallback,
and add only the false-coherence sidecar's active_proof_verification tag.
Preserve existing profiles, .miz/expectation intent, trace/map/ratchet and Task277B.
Core consumes only that check, with no raw SymbolEnv or repeated name/type checking, and lowers the full guarded functorial goal and actual definitions/proof through existing IR; VC failure requires authenticated Core/VC links and actual negated reflexive equality under established guards, never NeedsAtp alone.
Task274 accepted effects remain blocked. Owners: [Core](../../mizar-core/en/elaborator.md), [VC](../../mizar-vc/en/discharge.md), [registration](../../mizar-checker/en/registration_resolution.md),
[names](../../mizar-resolve/en/names.md), [harness](../../mizar-test/en/harness.md).
Audit impact: Chapter17 pending validation and source correctness failure, without acceptance or closure credit.

## Exit

Review specs/docs, tests, implementation, volume and consistency; test all four outputs,
renaming, source/owner/scope/type/guard/operand corruption, reduction order and exact
admission, false coherence, positive polarity and Core/VC-link corruption; preserve pending non-inference and the parser oracle.
Run focused tests/corpora, fmt, warnings-denied Clippy and cargo test; commit this increment.
Keep the parent task open until all seven unchanged-stage outcomes execute.
