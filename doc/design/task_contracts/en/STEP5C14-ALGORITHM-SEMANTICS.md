# Task STEP5C14-ALGORITHM-SEMANTICS: bounded algorithm observations

Canonical language: English; [Japanese pointer](../ja/STEP5C14-ALGORITHM-SEMANTICS.md).
Status: partial; static diagnostics and flat-state contract VCs; tier: full. Primary owner: mizar-core; consumers: mizar-vc, mizar-test;
checker owns authenticated binding/type intake. Dependencies: existing parser,
resolver replay, BindingEnv, term/formula inference, Core/CFG and VC seed accounting.
Owner plans: [Core](../../mizar-core/en/00.crate_plan.md),
[checker](../../mizar-checker/en/00.crate_plan.md),
[test](../../mizar-test/en/00.crate_plan.md).

## Purpose and authority
Activate the existing algorithm break-outside-loop and ghost-isolation
type-elaboration failures through source -> checker -> Core -> CFG; additionally activate the existing `pass_proof_verification_algorithm_ensures_return_001.miz` and `pass_proof_verification_algorithm_var_const_assert_001.miz` at proof_verification/vc_generation.
Authority: spec [20](../../../spec/en/20.algorithm_and_verification.md)
§§20.1.2–4, 20.2.6, 20.4.1–3 and 20.13.1/3/5; tests/miz/{pass,fail}/algorithms/
`fail_type_elaboration_algorithm_break_outside_loop_001.miz` and
`fail_type_elaboration_algorithm_ghost_isolation_001.miz`, with existing sidecars.
Both observations remain type_elaboration / elaboration / fail / type_error;
their existing stable detail keys and empty public codes are unchanged.

## Scope and ownership
Admit one builtin-object definition parameter, one algorithm with the matching
parameter and object return type, initialized single var/const/ghost-var bindings,
variable references, return and break. Authenticate complete source/environment/
typed correspondence, owner, binder identity, ranges, order and normal recovery.
Use existing BindingEnv/term inference, a minimal sealed checker result, existing
Core algorithm seeds and CFG IllegalBreak; add runtime initializer/return ghost
dependency rejection; the static route emits no VCs. The flat VC profile admits typed local assignment, equality asserts, optional equality ensures and a final return. Result is contract-only; pre-state values receive write identities, and later assertion assumptions retain their pending obligation dependencies. Const/parameter writes and ghost-to-runtime assignment fail statically.
API/invariants: [checker](../../mizar-checker/en/type_checker.md),
[lowering](../../mizar-core/en/elaborator.md),
[flow](../../mizar-core/en/control_flow.md).
VC ownership: [plan](../../mizar-vc/en/00.crate_plan.md), [generator](../../mizar-vc/en/generator.md), [IR](../../mizar-vc/en/vc_ir.md), [VC families](../../mizar-vc/en/source_vc_decomposition.md); runner: [harness](../../mizar-test/en/harness.md).
Graph boundaries: [source families](../../mizar-core/en/source_family_decomposition.md).

No semantic blocker remains. Raw SymbolEnv and unresolved syntax stay outside
Core; no parallel AST, accepted algorithm fact or fabricated type evidence.
Exclude field assignment, nested flow, calls, other contracts, loop havoc,
snapshots/claim, computation, termination proof/promotion, MVM and Step6.
This is partial Core42/43/46/48/52/53 source/diagnostic coverage, not completion or a
CFG snapshot baseline; bounded postconditions and assertions are partial VC43. Task274 and later C14 slices remain deferred.

## Artifacts and exit
Change existing checker/Core/CFG/VC/runner modules and relevant Rust tests;
activate only these sidecars without changing semantic expectations; both VC cases own complete generated/zero-VC baselines and narrow snapshot traces.
Maintain paired owning module docs and public-item inventories where needed,
this EN contract plus JA pointer, owner plan links and Chapter20 audit's partial
static-source coverage. The claim fixture gains only its matching void algorithm declaration (§§20.1.1,20.2.6,20.6.2); claim semantics and activation remain deferred.
Require real-source outcomes, safe/renamed/reference-mutation controls, source/
typed/env/owner/binding corruption rejection, exact stage admission, deterministic
Core/CFG/source mapping, static zero-VC behavior, actual return/state formulas, pre-assert contexts and pending dependency links, full deterministic VcSet snapshots and exact seed accounting. Two independent Core parameter values test copy, old-state and self-assignment behavior without broadening source admission; no-contract controls retain honest accounting.
Run specification/documentation, test-sufficiency, implementation, volume/scope
and source/documentation consistency reviews; resolve findings and repeat.
Run narrow checker/Core/runner tests, cargo fmt --check,
cargo clippy --all-targets --all-features -- -D warnings, and cargo test.
Exit with authentic static observations and open return/assertion VCs, preserved boundaries,
unchanged later-slice deferrals, and a task-only commit.
