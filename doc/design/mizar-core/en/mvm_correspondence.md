# MVM R2 — Verified Body and Call Correspondence

> Canonical English; [Japanese companion](../ja/mvm_correspondence.md).
> Documentation-only design proposal. This does not enable execution,
> proof acceptance, runtime serialization, or coverage.

## Existing specification and proposed ordinary-input boundary

[§20.1.1](../../../spec/en/20.algorithm_and_verification.md#2011-algorithm-definition-block)
uses expected parameter types: ordinary parameters take terms; formulas,
labels and `thesis` are restricted to `expr of boolean` parameters.
[§20.9](../../../spec/en/20.algorithm_and_verification.md#209-execution-model-and-computability)
requires call-by-value, local state, a computable verified body and validating
runtime artifacts for ground execution. [E1–E5](../../../spec/en/20.algorithm_and_verification.md#20131-contract-axioms)
separates acceptance from actual evaluation.

**Q1 proposal, not authorization:** an ordinary ground call needs authenticated
ordinary terms and values, not invented `expr`/Binder/UID/context-`C` values.
The existing AST-TYPE/AST-LOWER and ALG-MVM-EVALUATION contracts nevertheless
list reflected formation/checked lowering as prerequisites. A reviewed owner
decision must explicitly authorize any ordinary-only subpath before code.
Reflected, non-ground, template and cross-package calls stay out of the pilot.

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

**Q2 proof gates:** E1 needs result typing, single-valued execution and
inhabitation for all inputs on which the return type is asserted. E2 needs
genuinely accepted partial correctness with preconditions and exact
continuation context. A `terminating` promotion additionally requires
E4 accepted normal return, not a local `return` or an Open VC. A partial
algorithm can later execute on a supported concrete input without a global
E4 claim, but still requires the applicable verified-body/runtime conditions.
R2 emits no E5: only later actual MVM normal execution may do so.

## Pilot, rejection and successors

Existing [`idalgo(a)`](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz)
uses ordinary `object` input and direct `return a`; its
[expectation](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.expect.toml)
is an **Open VC**, not accepted algorithm evidence. The existing
[Checker](../../../../crates/mizar-checker/src/type_checker.rs),
[Core](../../../../crates/mizar-core/src/elaborator.rs) and
[VC](../../../../crates/mizar-vc/src/generator.rs) producers support bounded
source-derived pieces only. `idalgo(0)` remains a proposed
ground-call parse/type oracle, not a verified call or execution test.

Future tests must independently check matched body/signature/typed call/CFG
versus wrong type, swapped parameter/order, mutated body/contract, stale
module/snapshot, missing/deferred VC, forged evidence, wrong goal/context/
polarity, skipped E4 and ghost leakage. Preserve the existing test/sidecar.

**R2 implementation remains blocked:** Core owner first reviews Q1 and
locates genuine checked definition/accepted E1/E2 and applicable E4
producers; R3 can export a verified body **without a prior ground call**.
A separately authenticated typed invocation is required for R4.
Existing Core 42–53 and VC53 dependencies remain. R3 then freezes
serialization, correspondence and loader/visibility.
Code-extraction RuntimeIr has separate ownership and is not an implicit
MVM format. Open R3 choices: executable encoding and same-package
build/publication order. No new crate, runtime API, cache key or Kernel rule.
