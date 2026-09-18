# Task STEP5C12-UNBOUNDED-TEMPLATES: direct template type checking

Canonical language: English; [Japanese pointer](../ja/STEP5C12-UNBOUNDED-TEMPLATES.md).
Owning plans: [checker](../../mizar-checker/en/00.crate_plan.md#task-index),
[resolve](../../mizar-resolve/en/00.crate_plan.md#task-index), [test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

- Status: complete for the five mapped outcomes; full tier; checker owns typing, resolver owns formal identities.
- Dependencies: completed bounded 5A.2/5C.5; no accepted registration or Core execution prerequisite.
- Authority: [spec18](../../../spec/en/18.templates.md) §§18.2.1–3,18.2.6–7,18.7,18.8.2,18.10.1–3;
  [spec17](../../../spec/en/17.clusters_and_registrations.md) §17.3.4 builtin/schema inhabitation.
- Scope: pass_type_elaboration_template_type_param_functor_001,
  pass_type_elaboration_template_pred_param_001 and fail_type_elaboration_template_arity_mismatch_001, plus
  pass_type_elaboration_template_extends_bound_001 and
  fail_type_elaboration_template_bound_violation_001.
  Each retains type_elaboration/type_check and its existing outcome/key.
- Classification: source/formal-use resolution and symbolic/concrete typing gaps.

## Scope and boundaries

Authenticate resolved source, environment, declarations, order, recovery and all referenced owners.
Resolve type/predicate/value formals to actual declaration identities; no spelling-only bindings,
fabricated SymbolId(T/P), or treating abstract T as builtin set.
Check the identity functor's parameter type, bracket locus, result type and body under the same
abstract T identity before concrete substitution. Validate coherence ownership without proof credit.
The bounded continuation authenticates one earlier bare structure with a required set field and its constructor witness.
Resolve T to the schema structure object for T.carrier; x:T remains its symbolic element type, never the structure type.
Check the real selector/result type; only authenticated builtin actuals lacking the structure bound yield bound_violation after both calls are inspected.
Resolve both theorem and proof-local applications; derive actual ordered bracket arguments and
report arity failure from the checker only when the explicit bracket count mismatches the declaration.
Omitted bracket inference (§18.2.7) remains unsupported, never a zero-actual arity error.
For supported set actuals, perform genuine concrete type substitution and check both equality terms
and actual term bindings using existing checker type/term machinery and builtin inhabitation.
Check pred(T) signatures and each quantified P(x) call under its real formal and binder identities;
check connective/quantifier typing, never use tautology as a substitute for typing.
Predicate formals remain restricted to theorem/algorithm owners; ordinary defining bodies reject them.
Use existing declaration IDs privately for symbolic equality and existing concrete normalized types.
No new AST, formal-type IR, accepted facts, type-parameter sethood, proof execution or Task277B credit.
Preserve historical Task277A/277B neutral transports and fixed profiles.
Affected existing sources: resolver names, checker type_checker, type-elaboration harness/admission.
Planned seams: resolve_template_formal(&SurfaceResolvedArena, reference) -> actual ResolvedNodeId;
check_source_unbounded_template_types(&SurfaceResolvedArena, symbols) -> Result<(), String>.
The checker may match authenticated resolved shapes using the exact SurfaceNodeKind/SurfaceTokenKind
import in type_checker.rs; the existing import lint retains all other module and dependency restrictions.
Harness owns exact IDs/paths/phase/keys/tags and observations only; unsupported inputs earn no arity credit.
Add only the two structure-bound active_type_elaboration tags; preserve the prior three profiles, repaired source/expectation intent, trace and map.
Owner APIs and tests: [names](../../mizar-resolve/en/names.md#unbounded-template-formal-resolution),
[type checker](../../mizar-checker/en/type_checker.md#unbounded-template-type-checking),
[harness](../../mizar-test/en/harness.md#unbounded-template-type-admission).
Audit impact: Chapter18 gains bare structure-bound declaration typing and builtin actual-bound rejection; general instantiation/views remain deferred.

## Exit

Independent specs/docs, test-sufficiency, implementation, volume and source/docs reviews.
Source-driven positive/arity cases, consistent renaming, wrong formal/body/locus/type/signature,
missing/duplicate/forward/foreign/recovered declarations, each application occurrence and phase gates.
Require all five source-derived outcomes, bound/member/body/call corruption controls and Task277B zero-credit boundaries.
Run narrow tests/corpus then fmt, warnings-denied full Clippy and cargo test; commit the bounded result.
General bounded instantiation, inherited/attributed bounds and views remain outside these five outcomes.
