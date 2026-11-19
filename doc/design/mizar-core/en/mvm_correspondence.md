# MVM R2 — Verified Body and Call Correspondence

> Canonical English. [Japanese companion](../ja/mvm_correspondence.md).
> Owned by [MVM-R2-CORRESPONDENCE-DESIGN](../../task_contracts/en/MVM-R2-CORRESPONDENCE-DESIGN.md).
> Design proposal only; not a checked executable, approved runtime format or
> extension to the existing proof/acceptance boundary.

## Specification boundary

[§20.1.1](../../../spec/en/20.algorithm_and_verification.md#2011-algorithm-definition-block)
checks call arguments *contextually*: an ordinary parameter accepts a
`term_expression`; formula, label and `thesis` are allowed only for a
parameter of type `expr of boolean`. The [MVM](../../../spec/en/20.algorithm_and_verification.md#2091-the-mizar-virtual-machine-mvm)
evaluates ground calls with call-by-value, local mutable state and validated
execution artifacts. [E1–E5](../../../spec/en/20.algorithm_and_verification.md#20131-contract-axioms)
separate function-graph admission, partial correctness, normal-return
guarantees and *actual* finite execution. They are not one proof bit.

**Q1 — proposed scope distinction, not an approved dependency change.**
An ordinary typed ground call has no reflected syntax payload by virtue of
being ground. Reflection-specific `expr` formation/UID/Binder/`C` conditions
apply where that type of input is actually used; they must not be fabricated
for ordinary values. However [AST-LOWER](../../task_contracts/en/AST-LOWER.md)
and [ALG-MVM-EVALUATION](../../task_contracts/en/ALG-MVM-EVALUATION.md)
currently name checked reflection as an execution dependency. Until a
specification/contract-owner review explicitly accepts a separate ordinary
Core handoff or revises that scope, the implementation must **not** treat this
interpretation as authorization to bypass either contract. Non-ground
arguments, `expr`, captured Binder/context `C`, template calls and
cross-package imports remain out of the pilot.

## Required authentic binding (conceptual; no new public struct)

| Origin | Exact producer-controlled relation | Consumer |
|---|---|---|
| Resolver / Checker | One resolved algorithm symbol and owner, source/module/snapshot identity, ordered formal parameter types, context-sensitive call resolution, typed concrete actuals and effective visibility | Core (cannot recover identities from text) |
| Core | Same owner and immutable `CoreIr` algorithm/body/statement order, original contracts, source mapping, complete normal/failed exits and corresponding `ControlFlowIr` for the selected profile | VC / later R3 executable producer |
| VC / Kernel / Proof | Exact source-derived obligations, complete dependency set, original context/polarity, policy and matching accepted check results; no Open/Assumed status upgraded by Core | R3 eligible verified-body consumer; Proof remains authority |
| Artifact / Build (R3) | Only after accepted correspondence, bind exact executable content, body/signature/contracts, source/dependency versions, private-helper closure and validated loader inputs | MVM (R4) |

**Identity rule.** Every join must use owner-authenticated identities and
complete content/version provenance; source hashes alone, node offsets,
symbol spellings, accepted status flags or runtime-output bytes are not
sufficient proof. Changed source, typed actuals, contracts, CFG, dependency
versions, policy or selected evidence require revalidation at their owning
boundary. An exporter may not supply a caller's unproven evidence.

**Q2 — evidence gates.** For a value-returning algorithm, E1 requires
result typing, single-valued execution for complete input and inhabitation
on every input to which the return type is asserted; E2 requires actually
verified partial correctness with preconditions and exact return context.
A `terminating` declaration additionally needs E4 accepted normal-return
evidence for promotion; merely reaching `return`, an E2 VC, or a Kernel
check of one local VC does not establish E4. A *partial* body may later run
for a supported concrete input without a global E4 claim, provided the
applicable verification and runtime artifact conditions hold. Only MVM-R4
may produce E5 after a genuine finite normal return; R2 supplies no E5
report, contract fact, status or proof witness.

## Pilot and evidence gaps

The [existing `idalgo(a)` fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz)
has one ordinary `object` parameter, an `ensures result = a` clause and a
direct `return a`; the [sidecar](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.expect.toml)
requires only an **Open source-derived VC**. Existing
[`check_source_algorithm_types`](../../../../crates/mizar-checker/src/type_checker.rs),
[`lower_source_algorithms`](../../../../crates/mizar-core/src/elaborator.rs)
and [VC generation](../../../../crates/mizar-vc/src/generator.rs)
support a bounded original body; they supply neither a verified executable
nor an authenticated ground caller. `idalgo(0)` remains a *proposed*
source-call/type oracle, not an established positive test.

| Test obligation (new, not activated) | Positive requirement | Mandatory negative controls |
|---|---|---|
| Declared body ↔ typed call | Same owned source, exact symbol/signature/argument order/types, original contracts and Core/CFG content | Wrong overload/actual type, changed symbol/module, stale snapshot, body/statement reordered or altered |
| Proof and obligation closure | All applicable E1/E2 and (for terminating promotion) E4 evidence genuinely accepted under the selected policy | Open/Assumed/tainted evidence, missing VC, mismatched goal/context/polarity, substituted receipt, forged source/hash |
| Executable handoff to R3 | Complete authenticated body+call provenance and precise rejected/unavailable result if incomplete | Missing dependency, ghost flow into runtime, private external call, inferred executable from Core-only metadata |

## Dependency handoff and open decisions

R2 design → **Core-owned implementation freeze** only after Q1 is accepted
and the actual source-call/type oracle and the appropriate E1/E2/E4 proof
producer are identified. Core may preserve an immutable association but
must not mint a verified status. The existing [Core 42–53](./todo.md)
and [normal-return contract](../../task_contracts/en/ALGDEF-VC53-NORMAL-RETURN.md)
retain missing semantic/CFG/proof families. R3 (Artifact) freezes the
serialized executable and validating loader only after a real verified
source/body association exists; [code-extraction RuntimeIr](../../mizar-doc/en/todo.md)
remains separately owned. No choice of bytecode, runtime crate, or second
logical expression model is made here.

**Open:** Q1 contract scope / human-authority sign-off; Q2 exact accepted
whole-algorithm evidence producer and policy footprint; R3 executable format
and the same-package compilation/loader ordering. For the existing
`terminating idalgo` test, E4 is a mandatory outstanding gate for promotion,
not a reason to fabricate a result. Keep MVM-R2 *implementation blocked*.
