# Task STEP5C14-ALGORITHM-SEMANTICS: static algorithm observations

Canonical language: English; [Japanese pointer](../ja/STEP5C14-ALGORITHM-SEMANTICS.md).
Status: partial; bounded static increment; tier: full. Primary owner: mizar-core; consumers: mizar-test;
checker owns authenticated binding/type intake. Dependencies: existing parser,
resolver symbol replay, BindingEnv, term inference, Core algorithm seeds and CFG.
Owner plans: [Core](../../mizar-core/en/00.crate_plan.md),
[checker](../../mizar-checker/en/00.crate_plan.md),
[test](../../mizar-test/en/00.crate_plan.md).

## Purpose and authority
Activate only the existing algorithm break-outside-loop and ghost-isolation
type-elaboration failures using actual source -> checker -> Core -> CFG data.
Authority: spec [20](../../../spec/en/20.algorithm_and_verification.md)
§§20.1.2–3, 20.2.6 and 20.13.5; tests/miz/fail/algorithms/
`fail_type_elaboration_algorithm_break_outside_loop_001.miz` and
`fail_type_elaboration_algorithm_ghost_isolation_001.miz`, with existing sidecars.
Both observations remain type_elaboration / elaboration / fail / type_error;
their existing stable detail keys and empty public codes are unchanged.

## Scope and ownership
Admit one builtin-object definition parameter, one algorithm with the matching
parameter and object return type, initialized single var/ghost-var bindings,
variable references, return and break. Authenticate complete source/environment/
typed correspondence, owner, binder identity, ranges, order and normal recovery.
Use existing BindingEnv/term inference, a minimal sealed checker result, existing
Core algorithm seeds and CFG IllegalBreak; add runtime initializer/return ghost
dependency rejection. The runner maps real diagnostics only and emits no VCs.
API/invariants: [checker](../../mizar-checker/en/type_checker.md),
[lowering](../../mizar-core/en/elaborator.md),
[flow](../../mizar-core/en/control_flow.md).
Runner/admission: [harness](../../mizar-test/en/harness.md).
Graph boundaries: [source families](../../mizar-core/en/source_family_decomposition.md).

Source drift: bounded source algorithm checking/lowering and ghost rejection
are absent; existing parser topology, binding lookup and Core/CFG tables suffice.
No semantic blocker remains. Raw SymbolEnv and unresolved syntax stay outside
Core; no parallel AST, accepted algorithm fact or fabricated type evidence.
Exclude const/assignment, nested flow, calls, contracts, state/VC substitution,
snapshots/claim, computation, termination proof/promotion, MVM and Step6.
This is partial Core42/43/48/53 diagnostic coverage, not their completion or a
CFG snapshot baseline. Task274 and all later C14 source/VC slices remain deferred.

## Artifacts and exit
Change existing checker/Core/CFG/runner modules and relevant Rust tests only;
activate the two sidecar tags without changing their semantic expectations.
Maintain paired owning module docs and public-item inventories where needed,
this EN contract plus JA pointer, owner plan links and Chapter20 audit's partial
static-source coverage. No spec or existing .miz edit is required.
Require real-source outcomes, safe/renamed/reference-mutation controls, source/
typed/env/owner/binding corruption rejection, exact stage admission, deterministic
Core/CFG/source mapping and zero obligation/VC production.
Run specification/documentation, test-sufficiency, implementation, volume/scope
and source/documentation consistency reviews; resolve findings and repeat.
Run narrow checker/Core/runner tests, cargo fmt --check,
cargo clippy --all-targets --all-features -- -D warnings, and cargo test.
Exit with both authentic static observations, preserved failure boundaries,
unchanged later-slice deferrals, and a task-only commit.
