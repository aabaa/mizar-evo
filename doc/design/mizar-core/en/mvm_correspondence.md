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
| Resolver / Checker | Original source/module/snapshot; resolved algorithm ID, visibility, formal types, ordered typed concrete actuals and source call | Core |
| Core | Same algorithm ID/item, immutable CoreIr body, ordered statements, contracts, CFG/exits, source mapping and dependency identities | VC; future R3 |
| VC / Kernel / Proof | Applicable generated obligations and exact accepted results, original goal/context/polarity, dependencies and policy | R3 eligibility, never Core proof authority |
| Artifact / Build (R3) | Validated executable content and loader bound to the accepted body/signature/contracts/dependencies | MVM (R4) |

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
identifies a genuine typed call and complete accepted E1/E2/E4 producers;
existing Core 42–53 and VC53 dependencies remain. R3 then freezes
executable serialization, verified-body linkage and loader/visibility.
Code-extraction RuntimeIr has separate ownership and is not an implicit
MVM format. Open R3 choices: executable encoding and same-package
build/publication order. No new crate, runtime API, cache key or Kernel rule.
