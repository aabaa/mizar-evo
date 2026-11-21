# Task P23-ATP-VARIABLE-GOALS: variable-bearing local goals
Canonical language: English; [Japanese pointer](../ja/P23-ATP-VARIABLE-GOALS.md).
Status: frozen prerequisite for P23; tier: full (production accepted-input boundary).
Owner: [ATP plan](../../mizar-atp/en/00.crate_plan.md); consumer: P23 ATP-G-029 seeds, both retaining `q(x)`.
Dependency/readiness: owner decision 2026-11-22 option 1 authorizes this bounded prerequisite; P23 resumes after the landed-unit audit.
Authority: specification [§21.7.2](../../../spec/en/21.source_code_annotation_and_atp.md#2172-problem-translation-pipeline-translator), [§21.7.1](../../../spec/en/21.source_code_annotation_and_atp.md#2171-minimum-kernel-architecture-and-de-bruijn-criterion), [§21.7.5](../../../spec/en/21.source_code_annotation_and_atp.md#2175-kernel-evidence-format).
Existing tests: encoder scope/profile refusals, translator `instance_finder_*`, `local_candidate_*` and both upstream `task_29_*` refusals.
Gap: source_gap; concrete encoders reject free goal variables, and finder correspondence rejects them, including the factory's search-disabled validation call.

## Scope and design
Explicitly universally close only free conjecture variables in deterministic symbol order
for concrete backend text, using existing unsorted generated-binder validation and names.
SMT-LIB negates the entire closure. First-order profile, duplicate/shadowed/sorted binder,
declaration/source, collision and non-goal free-variable checks remain fail closed.
Use a crate-private free-variable collector shared by both encoders and finder.
Keep the original problem, local handoff formulas, variable IDs, provenance and context unchanged.
Finder authenticates goal-variable correspondence and treats those variables as rigid
equality-search terms charged against the term limit, outside the closed substitution domain.
The factory reuses that correspondence validation with search disabled; no new API is needed.
General quantified-goal finder support and premise instantiation on local variables stay deferred.
Paired owner boundaries: [TPTP scope](../../mizar-atp/en/tptp_encoder.md#name-mangling), [SMT scope](../../mizar-atp/en/smtlib_encoder.md#name-mangling), [finder](../../mizar-atp/en/instance_finder.md#api-and-boundary), [candidate factory](../../mizar-atp/en/backend.md#candidate-evidence-boundary).
Affected: these owners, owner-plan backlink, problem free-variable traversal, both concrete encoders, finder and existing encoder/translator test modules.
Forbidden: language/spec changes, Kernel/schema/acceptance changes, source extraction expansion,
proof status/publication/cache, corpus/trace/expectation activation, owner-gated scope and P23 completion.
Audit/deferral: no spec-coverage-audit credit changes; original P23 acceptance scope remains pending.

## Test matrix and exit
Six cases maximum; further sufficiency cases must cite authority.
1. Both encoders: repeated/multiple/nested free goal variables close once, lexically bound variables stay bound; deterministic text and metadata; closed goals unchanged.
2. Negative controls: free premise variables, non-first-order closure, invalid binder source, duplicate/shadowed/sorted binders still refuse. Retarget existing free-variable refusal inputs from conjecture to premise; preserve their oracle and unrelated checks.
3. Genuine variable-goal fixture: factory emits canonical fresh bytes through both formats; matching finder validates with absent limits and bounded search yields only independently Kernel-accepted evidence when available.
4. Both original rejection shapes: consistency polarity and unverifiable local-source candidates with `q(x)` parse and independently reject against the unchanged original context.
5. Variable correspondence mismatch, stale target/producer/encoded binding and unsupported goal shapes refuse before emission; zero/insufficient limits remain literal and rigid terms consume the term budget.
6. Ground behavior, original upstream task-29 refusals and existing scoped-theorem finder/substitution-domain behavior remain unchanged.
Establish the genuine variable-goal fixture and pass one setup case before expanding its tests.
Reviews: independent specification/test intent, sufficiency, precision/trust, removable volume/scope, source/docs consistency and scoped quality evaluation.
Verification: fmt; all-target/all-feature Clippy `-D warnings`; checker/test `lint_policy` including links/fragments; ATP owner tests; proof-verification executable check; full `cargo test` on final pushed tree (dispatch push/cadence trigger).
Exit: freeze committed first; matrix/checks pass; original trust boundaries preserved; implementation committed and pushed for T2 audit.
