# MVM R4 — Minimal Authenticated Executor

> Canonical English; [Japanese companion](../ja/mvm_executor.md).
> R4 design task `MVM-R4-EXECUTOR-DESIGN`.
> Design proposal only. The executor and its proof/runtime interfaces
> are **not implemented** by this document.

## Contract and initial scope

[Spec §20.9](../../../spec/en/20.algorithm_and_verification.md#209-execution-model-and-computability)
defines verification-time call-by-value evaluation, only local mutable
state, ground/direct or explicit calls, and runtime failure/exhaustion.
[§20.13.1 E5](../../../spec/en/20.algorithm_and_verification.md#20131-contract-axioms)
allows an internal `returns_f(c,v)` fact **only from a real finite normal
MVM return** bound to the verified body and complete input. The current
`ALG-MVM-EVALUATION`
owner is `mizar-vc`, not Core, Kernel or Artifact.

First *hypothetical* supported case: the existing
[`idalgo(a)`](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz)
with one authenticated concrete ordinary input and `return a`. Its
[sidecar](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.expect.toml)
records an **Open VC only**; `idalgo(0)` is an unverified call/type probe.
The existing [computation hint](../../../../tests/miz/pass/algorithms/pass_proof_verification_computation_justification_001.miz)
is another Open request, **not** an evaluated algorithm. This is not a
positive runtime fixture or an approved ordinary-only bypass of
AST-TYPE/AST-LOWER.

## Proposed entry, output and owner boundary (no new public API)

| Stage | Required authenticated input / output | Refusal and authority |
|---|---|---|
| R2 Checker/Core/VC/Proof | One source-bound resolved call, typed complete actuals and original checked body, immutable Core/CFG contract and applicable accepted evidence | Missing source call, Open/deferred body, wrong typed actual or version cannot be promoted by the VM |
| R3 Artifact / Build / Driver | Read-only validated executable handle, matching entry/signature/body, verified correspondence, full transitive executable/dependency/VM identities | Unknown schema, altered bytes, incompatible version, missing/stale import, or invalid visibility gives **no executable input**; no dependency-source fallback |
| R4 VC executor | Validated handle plus authenticated concrete call and effective limits -> actual finite return or explicit incomplete/failure | Entry matches one supported instruction/body profile; no reparse, speculative evaluation, ghost input, mutable global state or out-of-scope fallback |
| R6 VC / Proof admission | Complete evaluated-call closure and accepted uniqueness/exemption where required | Must precede **computation-based discharge**; not a substitute for R4 runtime inputs |
| R7 Proof / Artifact / Cache | R4 normal report plus independently justified preconditions/Adm/InputOK and certified conversion/replay -> proof status and later computation record | No proof acceptance, contract fact, cached `computed`/replay upgrade, or Kernel witness is issued by R4 |

A successful R4 attempt retains original target, complete logical values
and context (including `C` when applicable), actual result, executed-call
and transitive artifact identities, compatible VM/rule versions and
source/currentness provenance. **Report shape and schema remain R3/R4
freeze decisions**; a tuple/hash/request-only marker is not proof.

## Execution semantics, failure and budgets

For the minimal identity-like `return` path, evaluate arguments before
entry (call-by-value), bind their already-typed values to the correct
formal parameters, then evaluate the **actual validated** return operand.
The result is a normal return only if all required operations complete
and the recorded execution matches the original entry/body.
Only local non-ghost state has runtime effects. `requires` and
computability are **mandatory preconditions**, not facts proven by the
callee's own `ensures`. The executor does not check or discharge them
as a mathematical proof.

Separate outcomes: (a) normal return with authenticated value; (b)
runtime failure, including reachable `fail`/unexecutable operation;
(c) resource limit/incomplete execution; (d) invalid/unavailable
artifact or input. Only (a) can supply an internal E5 candidate;
(b)–(d) supply **no** return, public contract proof or theorem of
non-return. Runtime diagnostic mapping belongs to the specified stage,
not a new R4 error-code allocation. R5 owns branches, loops, recursive
calls, Pick, nontrivial local state, failure and complex resource cases.

The spec's `steps`, `timeout`, `nest` each default to `0` (unlimited);
a positive bound applies to the actual execution. Before R4 code,
freeze what one VM step charges, nested-call accounting, timeout
measurement and cancellation/limit precedence, and VM-versioned
behavior. The current [pre-ATP discharge limit model](./discharge.md#limit-model)
has a separate policy key/default and does **not** define VM gas.
Wall-clock-dependent timeouts may change whether a run finishes;
they cannot serve as accepted or reproducible proof evidence.
Exact successful computations must be reproducible for fixed
authenticated input, compatible VM version and execution closure.

## Readiness and test design

| Case | Future executable oracle | Required negative control |
|---|---|---|
| Ground direct return | Real verified `idalgo` body, genuine typed concrete call, validated R3 handle, exact returned value | No real caller, wrong argument/type, changed body, forged receipt, or incomplete E1/E2/E4 remains blocked |
| Artifact and version | Same checked entry/version/dependency closure loads; stale/invalid inputs refused | Swapped executable bytes, wrong VM/contract/manifest, missing/private imported target, local source fallback |
| Result classification | Only actual finite normal return emits report; same call repeats its exact result | `fail`, missing operation, missing requires evidence, unsupported instruction, expired budget and stale snapshot yield no accepted result |
| Execution isolation | No ghost-to-runtime flow, global mutation or implicit source evaluation | Ghost value leak, alias/state mismatch, same-name different symbol, reused weaker-policy record rejected |

**Open R4 choices:** (1) a minimal instruction/return value
representation consistent with R3's as-yet-unfrozen format; (2) the
authenticated typed ground call and ordinary/reflected dependency
decision from R2; (3) gas/timeout/nest counting and report/framing;
(4) whether same-package verification requires R3's staged verified
load path. Never decide these by fabricating runnable artifacts.

**Implementation release:** resolve R1/R2/R3 independent reviews,
genuine accepted whole-body evidence and real loader prerequisites;
freeze the smallest executor contract under the existing
`ALG-MVM-EVALUATION` owner, with test-first positive and rejection
oracles. Until then R4 is design-only; R5–R7 are separate.
