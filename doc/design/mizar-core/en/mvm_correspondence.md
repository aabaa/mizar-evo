# MVM R2 — Verified Body and Call Correspondence

> Canonical English; [Japanese companion](../ja/mvm_correspondence.md).
> Documentation-only design proposal. This does not enable execution,
> proof acceptance, runtime serialization, or coverage.

## Q1: ordinary and reflected dependency boundary

[§20.1.1](../../../spec/en/20.algorithm_and_verification.md#2011-algorithm-definition-block)
uses expected parameter types: ordinary parameters take terms; formulas,
labels and `thesis` are restricted to `expr of boolean` parameters.
[§20.9](../../../spec/en/20.algorithm_and_verification.md#209-execution-model-and-computability)
requires call-by-value, local state, a computable verified body and validating
runtime artifacts for ground execution. [E1–E5](../../../spec/en/20.algorithm_and_verification.md#20131-contract-axioms)
separates acceptance from actual evaluation.

**Decision: retain the existing dependency gates.** A term argument alone
does not establish an ordinary-value path: an expected `expr of T` can
reflect that term by R1 or embed an existing syntax value by R2
([§20.5.3](../../../spec/en/20.algorithm_and_verification.md#2053-reflected-syntax)).
Ordinary parameters also do not exclude reflected operations in the body or
transitive callees. Genuine ordinary declaration/type identities and source
provenance remain necessary; reflection-specific UID/environment/context-`C`
inputs must neither be fabricated nor omitted where applicable.

**Owner decision 2026-11-19 23:28 JST:** narrowing the dependencies for an
ordinary-only, non-reflected pilot was not approved. The current
AST-TYPE/AST-LOWER and ALG-MVM-EVALUATION gates remain in force.

## Authentic binding: Core records, not a new runtime type

| Owner | Exact association | Consumer |
|---|---|---|
| Resolver / Checker (definition) | Original declaration identity, source/module/snapshot, visibility and signature | Core and definition-side R3 |
| Core + VC / Kernel / Proof (definition) | Same immutable body, contracts, CFG/exits and actual accepted obligations under complete policy and provenance | R3 reusable algorithm export; Core never mints Proof status |
| Resolver / Checker (invocation) | Independently resolved, typed concrete call and ordered actuals with original source/context and target identity | R4, **not a prerequisite for R3 export** |
| Artifact / Build (R3) | Validated executable bound to the verified definition, signature, contracts and dependencies | R4 joins loader handle with typed invocation |

Every join is fail-closed on missing, duplicate, reordered, stale,
recovered or mismatched owner/input. A source hash, name, Core node or
unverified status flag alone never authenticates the whole relation. Core
must not manufacture a proof receipt or reparse original source.

## Q2: definition evidence and invocation inputs

E1 needs result typing, single-valued execution and
inhabitation for all inputs on which the return type is asserted. E2 needs
genuinely accepted partial correctness with preconditions and exact
continuation context. A `terminating` promotion additionally requires
E4 accepted normal return, not a local `return` or an Open VC. A partial
algorithm can later execute on a supported concrete input without a global
E4 claim, but still requires the applicable verified-body/runtime conditions.
R2 emits no E5: only later actual MVM normal execution may do so.

| Definition-side obligation | Genuine current producer and its limit | Missing acceptance for R3 |
|---|---|---|
| E1 | Checker [`check_source_algorithm_types`](../../../../crates/mizar-checker/src/type_checker.rs) and Core [`lower_source_algorithms`](../../../../crates/mizar-core/src/elaborator.rs) authenticate the bounded profile. Proof associates checked original typing/single-valuedness schemas and separately adopts original inhabitation under native policy. | Schema association or single-obligation adoption is not complete E1 or definition acceptance. |
| E2 | VC [`generate_source_algorithm_postconditions`](../../../../crates/mizar-vc/src/generator.rs) produces actual return goals/context. Proof [`check_source_algorithm_return_reflexivity`](../../../../crates/mizar-proof/src/status.rs) checks one selected return VC through Kernel/policy. | A checked return step does not establish complete body correctness or acceptance of every applicable obligation under the same body/contracts/CFG and policy. Its [projection test](../../../../crates/mizar-test/src/runner/proof_verification/term_proof_tests/source_algorithm_return.rs) retains an Open, incomplete dependency slice and an Artifact publication gap. |
| Applicable E4 | Proof can adopt the original bounded E4 existential under native policy; the [pilot snapshot](../../../../tests/snapshots/vc/pass_proof_verification_algorithm_ensures_return_001.vc_ir.snap) still has a Deferred termination seed without a concrete VC. | Local E4 adoption leaves that seed and independent E1/E2 duties unchanged. Broader normal return still covers failure, child calls and iteration ranges. |

AST-CONTRACT retains reflected E1/E2/E4 guards and formation dependencies.
Neither the bounded producers nor their planning contracts are complete
accepted definition evidence. R3 still needs the missing accepted obligations
and their authentic association to the immutable definition. Independently,
R4 needs resolved original call/target identity, signature-checked ordered
actuals, complete supported values and source/context provenance; those
invocation inputs cannot repair missing definition evidence or gate R3 export.

## Bounded accepted-definition boundary

The [existing VC profile](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design)
is exact: one immutable `object` parameter, `object` result, equality ensures,
sole return of that parameter and authenticated `terminating` intent.
Calls, loops, Pick, recursion, groups, reflected/opaque types, recovered nodes
and ghost flow remain outside this profile; no ordinary-only dependency bypass follows.

The bounded set contains five distinct original obligations: E1 result typing,
single-valuedness, all-typed-input inhabitation, the E2 substituted postcondition
and independent E4 normal-return existence. Inhabitation retains typed-input
guards without assuming return or restricting it to `requires`; E4 retains
the operational return relation. E2 must cover the whole supported body with
its exact preconditions/continuation context; equality ensures or a selected
return proof is insufficient without that complete-body association.
E3 supplies no missing E1 premise; E5, symbol introduction and promotion are excluded.

**Binding and atomicity.** Regenerate/authenticate source, Checker result,
symbols, original declaration/module/signature, immutable Core body/contracts,
CFG/exits and all five original goals/contexts under their owning schemas.
The E2 return identity retains its own generation/VC schemas; it is not
retargeted to a four-role direct-return identity. Keep each role's entire raw
slice and original identity, associated to the same definition and snapshot.
Canonical hashes omit snapshot identity, so separately require exact snapshot
agreement and the identical immutable Core arena. Caller-owned captured-source,
module/dependency and snapshot currentness must be established at the boundary;
a supplied snapshot ID or unchanged name/hash alone is insufficient.

Use genuine same-call lower-owner results and re-evaluate each actual rule,
checker/evidence schema, origin and polarity under the current effective policy.
Typing/single-valuedness schema scalars alone are not policy-accepted evidence.
The existing [inhabitation](../../mizar-proof/en/status.md#native-e1-inhabitation-adoption),
[E4](../../mizar-proof/en/status.md#native-original-e4-adoption) and
[E2](../../mizar-proof/en/status.md#source-algorithm-return-reflexivity) paths
retain their limits; no detached token, earlier policy fingerprint or fabricated
generic receipt can substitute. Expose the bounded adopted set only after all
five roles and bindings succeed under that policy; any refusal/error exposes
no partial set, symbol, promotion or export. Raw Open/Deferred graphs are unchanged.

| Bounded refusal | Required result |
|---|---|
| Missing/duplicate/swapped role, wrong original goal/context/owner/schema, Open-only or tainted evidence | Refuse whole set; raw Open requests require independent genuine adoption, not status relabeling. |
| E1 omitted, E2 selected-step success without whole-body coverage, or E4 replaced by declaration/equality/continuation/stopping evidence | Refuse; no role repairs another. |
| Changed source/check/symbols/Core/CFG/contracts, foreign snapshot or stale dependencies; policy disabled or incompatible | Refuse whole set after authentic upstream checks; preserve producer errors. |
| Unsupported profile or unresolved required provenance/policy input | Refuse; no synthetic guards, complete slices or eligibility. |

**R3 input, not export.** A future consumer receives the matched definition,
all five original identities and current-policy adoption decisions, entire raw
slices, snapshot/provenance bindings and explicit unresolved dependencies.
Unknown/incomplete slices remain reuse-ineligible. Independent bounded adoption
does not erase Open4 or the goal-free Deferred termination seed. G10 remains
parked/unregistered here and gates P13; this freeze neither discharges it nor
claims complete definition/R3 readiness while required residuals remain.
R3-Q2/R5-Q2 remain G/RESERVED/unassigned.

## Pilot, rejection and successors

Existing [`idalgo(a)`](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz)
uses ordinary `object` input and direct `return a`; its
[expectation](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.expect.toml)
is an **Open VC**, not accepted algorithm evidence. `idalgo(0)` remains a proposed
ground-call parse/type oracle, not a verified call or execution test.

Future tests use genuine matched definition evidence as the positive control
for R3, and separately a typed call joined to that definition for R4:

| Boundary | Required rejection control |
|---|---|
| Definition binding | Changed body/contracts/signature/CFG/exits, reordered statements or foreign owner. |
| Evidence acceptance | Missing/deferred/Open/tainted obligation, forged receipt or wrong goal/context/polarity/policy. |
| E1 and E4 | Missing typing/single-valuedness/inhabitation; skipped normal-return obligation for a terminating definition, including partial child/range or failure paths. |
| Currentness | Changed source/module/snapshot or dependency/evidence version; an unchanged name/hash alone cannot repair the join. |
| Invocation join | Unresolved/wrong target, wrong type, actuals missing/duplicated/reordered relative to the authenticated typed call, or stale call context; reject at R4 without demanding a caller at R3. |
| Runtime eligibility | Unsupported reflected input/body/dependency, recovered nodes or ghost flow; never synthesize omitted context/`C` or an E5 result. |

Preserve the existing test/sidecar; these are future stage-specific controls,
not activated runtime tests.

**R2 implementation remains blocked:** the retained gates are unmet, and
complete accepted E1/E2 and
applicable E4 producers are still missing. R3 can export a verified body
**without a prior ground call** once its definition evidence exists.
A separately authenticated typed invocation is required for R4.
Existing Core 42–53 and VC53 dependencies remain. R3 then freezes
serialization, correspondence and loader/visibility.
Code-extraction RuntimeIr has separate ownership and is not an implicit
MVM format. Open R3 choices: executable encoding and same-package
build/publication order. No new crate, runtime API, cache key or Kernel rule.
