# Task STEP5C14-ALGORITHM-SEMANTICS: bounded algorithm observations

Canonical language: English; [Japanese pointer](../ja/STEP5C14-ALGORITHM-SEMANTICS.md).
Status: partial; static/state and bounded void-claim VCs active; contradictory-assertion failure active; computation request active; flat snapshot capture active; tier: full. Primary owner: mizar-core; consumers: mizar-vc, mizar-test;
checker owns authenticated binding/type intake. Dependencies: existing parser,
resolver replay, BindingEnv, term/formula inference, Core/CFG and VC seed accounting.
Owner plans: [Core](../../mizar-core/en/00.crate_plan.md),
[checker](../../mizar-checker/en/00.crate_plan.md),
[test](../../mizar-test/en/00.crate_plan.md).

## Purpose and authority
Activate the existing algorithm break-outside-loop and ghost-isolation
type-elaboration failures through source -> checker -> Core -> CFG; additionally activate the existing `pass_proof_verification_algorithm_ensures_return_001.miz` and `pass_proof_verification_algorithm_var_const_assert_001.miz` and `pass_proof_verification_claim_block_theorem_001.miz` at proof_verification/vc_generation.
The existing computation row retains source-owned literal/request transport. The unchanged `pass_proof_verification_algorithm_ghost_snapshot_001.miz` is specified at pass/vc_generation through [flat capture](../../mizar-core/en/control_flow.md#flat-snapshot-flow) and its ordinary Open return VC.
Authority: [2](../../../spec/en/02.lexical_structure.md) §2.7, [3](../../../spec/en/03.type_system.md) §3.4, [16](../../../spec/en/16.theorems_and_proofs.md) §§16.1–2; [14](../../../spec/en/14.formulas.md) §§14.3.5,14.5.2; spec [20](../../../spec/en/20.algorithm_and_verification.md)
§§20.1.1–4, 20.2.6, 20.4.1–3, 20.6.1–2, 20.9.2 and 20.13.1/3/5; tests/miz/{pass,fail}/algorithms/
`fail_type_elaboration_algorithm_break_outside_loop_001.miz` and
`fail_type_elaboration_algorithm_ghost_isolation_001.miz`, with existing sidecars.
Both observations remain type_elaboration / elaboration / fail / type_error;
their existing stable detail keys and empty public codes are unchanged.
The existing `fail_proof_verification_algorithm_assert_unprovable_001.miz` retains verification/fail, proof_failure and algorithms.assert.unprovable; no source or semantic expectation changes.

## Scope and ownership
Admit one builtin-object definition parameter, one algorithm with the matching
parameter and object return type, initialized single var/const/ghost-var bindings,
variable references, return and break. Authenticate complete source/environment/
typed correspondence, owner, binder identity, ranges, order and normal recovery.
Use existing BindingEnv/term inference, a minimal sealed checker result, existing
Core algorithm seeds and CFG IllegalBreak; add runtime initializer/return ghost
dependency rejection; the static route emits no VCs. The flat VC profile admits typed local assignment, equality or single prefix-negated equality asserts, optional equality ensures and a final return. Result is contract-only; pre-state values receive write identities, and later assertion assumptions retain their pending obligation dependencies. Const/parameter writes and ghost-to-runtime assignment fail statically.
The separate void-claim profile admits only an earlier parameterless/resultless algorithm with bare return and its later single unmodified set-equality theorem. Authenticate the actual algorithm symbol, empty execution interface, wrapper, quantified proposition and proof-local binding/goal; retain the target through existing Core dependencies and seed references, never an ordinary-theorem fallback.
For the negative assertion, preserve the real Not(Eq) graph and complete open VC; only whole-result replay with the same immutable object parameter and its consistent guard may yield failure. No pending assertion, unknown goal or terminating annotation is failure evidence.
API/invariants: [checker](../../mizar-checker/en/type_checker.md),
[lowering](../../mizar-core/en/elaborator.md),
[flow](../../mizar-core/en/control_flow.md).
VC ownership: [plan](../../mizar-vc/en/00.crate_plan.md), [generator](../../mizar-vc/en/generator.md), [failure observer](../../mizar-vc/en/discharge.md), [IR](../../mizar-vc/en/vc_ir.md), [VC families](../../mizar-vc/en/source_vc_decomposition.md); runner: [harness](../../mizar-test/en/harness.md).
Graph boundaries: [source families](../../mizar-core/en/source_family_decomposition.md).

No semantic blocker remains. Raw SymbolEnv and unresolved syntax stay outside
Core; no parallel AST, accepted algorithm fact or fabricated type evidence.
Exclude field assignment, nested flow, calls, other contracts, loop havoc,
nested snapshots and snapshot claims, broader claims, computation execution/acceptance, termination proof/promotion, MVM and Step6. The [bounded request producer](../../mizar-vc/en/generator.md#bounded-computation-request-generation) retains the actual zero equality and explicit steps digits as an Open theorem-proof VC; other literal/options profiles remain unsupported.
This is partial Core42/43/46/47/48/51/52/53 source/diagnostic coverage, not completion or a
CFG snapshot baseline; flat captures use actual visible declaration identities and capture-point context without adding facts; bounded postconditions/assertions are partial VC43 and void claims partial VC54, and computation requests partial VC32. Task274 and later C14 slices remain deferred.

## Artifacts and exit
Change existing checker/Core/CFG/VC/runner modules and relevant Rust tests;
activate only these sidecars without changing semantic expectations; return/state cases retain generated/zero-VC baselines; the void claim adds its full generated VcSet baseline and narrow snapshot trace. The contradictory-assertion row adds complete raw-negative and positive-equality control baselines; the computation row retains its request-preserving baseline. Flat snapshot adds its complete return VcIr baseline/trace and exact Core/CFG capture controls, including shadowing, declaration/write order and ghost isolation.
Maintain paired owning module docs and public-item inventories where needed,
this EN contract plus JA pointer, owner plan links and Chapter20 audit's partial
static-source coverage, symbolic computation requests and Chapter14's bounded negated-reflexive failure coverage. Preserve the repaired claim fixture and all other source/semantic expectations; no execution, theorem acceptance or Task274 credit follows.
Require real-source outcomes, safe/renamed/reference-mutation controls, source/
typed/env/owner/binding corruption rejection, exact stage admission, deterministic
Core/CFG/source mapping, static zero-VC behavior, actual return/state formulas, pre-assert contexts and pending dependency links, full deterministic VcSet snapshots and exact seed accounting. Two independent Core parameter values test copy, old-state and self-assignment behavior without broadening source admission; no-contract controls retain honest accounting.
Run specification/documentation, test-sufficiency, implementation, volume/scope
and source/documentation consistency reviews; resolve findings and repeat.
Run narrow checker/Core/runner tests, cargo fmt --check,
cargo clippy --all-targets --all-features -- -D warnings, and cargo test.
Exit with authentic static observations and open return/assertion/claim/computation VCs plus bounded assertion failure observations, preserved boundaries,
unchanged later-slice deferrals, and a task-only commit.
