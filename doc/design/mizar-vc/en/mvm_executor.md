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

## R5-Q1 accounting and report dossier

**For owner consideration; no answer or implementation freeze.** This dossier
prepares R4 choice (3) below, including the pilot; R5-F extends the eventual
rule. Recommendations do not release R4 or settle R3 representation/loading,
R5 witness providers, R6 admission or R7 conversion/replay.

### Fixed constraints and decision surface

[§20.9.2–4](../../../spec/en/20.algorithm_and_verification.md#2092-by-computation-proof-step)
fixes `0` as unlimited, timeout as wall-clock milliseconds and nest as maximum
function-call stack depth. Options apply to every call in a computation
reference; §20.9.4 fixes gas per `by computation` invocation. Child calls must
not obtain fresh budgets. Actual argument evaluation and transitive calls are
execution work even when performed before the selected entry is entered.
Validation, proof checking, conversion and inspection retain their own stages.

The remaining choices below concern derived VM accounting and internal report
design. A choice requiring different language intent must return to the owner
as a specification question rather than become a pilot convention.

### Options, consequences and recommendations

| Question | Options and consequences | Recommendation to the owner |
|---|---|---|
| Step unit | Charge each executable instruction: easy to count, but tied to R3 encoding/lowering and may hide expensive primitive work. Charge versioned logical execution operations, with explicit costs for composite primitives: less sensitive to encoding, but needs an audited cost table and interruptible implementation. | Prefer logical operations with a small explicit schedule agreed with R3; bind it to VM semantics. Enumerate argument, entry, return and primitive work before the pilot; do not infer gas from AST nodes, source lines or pre-ATP fuel. |
| Nested sharing | One invocation ledger shared by frames makes cumulative steps and the deadline explicit. Passing remaining fuel/deadline through each call can preserve the same bounds, but requires exact debit/return handling on every exit, including failure. Resetting a child budget is outside the fixed constraints. | Prefer one invocation ledger; freeze where each charge is reserved, and ensure completed work is neither refunded nor double charged. Retain one gas bound across explicit/goal calls in the same reference; specify their order and repeated-call treatment. |
| Nest | Count active logical algorithm frames, including the root as depth 1; this matches stack depth directly. Counting physical host frames makes limits depend on implementation; eliminating logical frames on tail calls changes which runs fit. A root-at-zero convention needs an explicit relation to the specified depth. | Prefer root depth 1, a check before pushing a callee, and logical frames retained despite host optimizations. Argument calls contribute while active, and completed sibling calls do not accumulate depth. Freeze exact entry/pop and maximum-depth reporting. |
| Timeout span | A monotonic elapsed wall-clock timer over VM argument/body/callee execution preserves milliseconds despite wall-clock adjustments. Timing the entire load/prove/import attempt additionally includes non-execution work and obscures the exhausted stage. CPU time or step-derived time is not the specified timeout. | Prefer one VM deadline, never renewed by a child. Keep validation/import resource accounting separate. Freeze start/end points, safe-point placement and maximum primitive interruption latency; polling only between unbounded primitives cannot enforce a positive bound. |
| Boundaries and interruption | Checking before work prevents an over-budget operation; checking only afterward may execute beyond the bound. A fixed tie order gives repeatable internal reasons; retaining all observed stop causes preserves detail but complicates framing. Wall-time observation remains scheduling-dependent. | Prefer pre-charge/pre-push checks and deadline/cancellation checks at bounded safe points, including before publishing success. Owner must choose equality at the deadline, simultaneous stop precedence and cancellation framing; no diagnostic code or precedence is selected here. |
| Report framing | A minimal internal normal-return record plus a separate attempt outcome separates E5 input from incomplete/failure data. A full instruction trace aids investigation/replay but couples R4 to an unfrozen format and can grow without bound. Serializing either now would prematurely settle an R3/R7 schema. | Prefer the minimal internal record and separate attempt outcome; defer trace/persistence until their owners require it. Report concepts below are a checklist, not fields, types or a wire format. |

An instruction-count pilot remains an alternative only if its accepted unit
and expensive primitives are explicitly bounded and versioned; it must not
silently become the later logical-operation schedule. Any accounting change
that can change completion at a finite limit needs compatible VM semantics
and consumer invalidation rules coordinated with R3/R7.

### Report checklist and freeze handoff

The existing successful-report provenance above remains mandatory. For the
owner's framing choice, additionally consider effective limits, accounting-rule
identity, consumed steps, peak logical depth, timeout scope and the completed
call's place in the invocation. Elapsed time is operational information, not
reproducible proof evidence. A digest, counter or successful child does not
replace the authenticated complete normal return of a requested call.

For failed attempts preserve stage/operation, the applicable resource/limit
and call path without carrying a successful-result marker. Existing
[E0331/E0333 obligations](../../../spec/en/22.error_handling_and_diagnostics.md#2248-reflection-and-computation-diagnostics)
still apply: runtime failure reports its message, listed values and call chain;
resource incompleteness names the actual stage, operation, resource and limit.
Cancellation framing stays open. If an earlier call returned before a later
call or import failed, retain that genuine internal evidence separately;
the failed operation supplies no new E5 fact or unchecked public candidate.

Before R4's pilot, the owner answer must make the charge schedule, root/callee
depth, shared limits, clock interval, check points, tie/cancellation policy and
report framing unambiguous, in coordination with R3's instruction/value and
VM-version choices. The implementation contract must then bind genuine
R2/R3 inputs to the following controls within the existing readiness matrix:

- Exact required steps versus one less, omitted/zero limits, argument and
  nested work, and multiple calls without budget reset or duplicate charging.
- Root and nested depth at/over the bound, siblings and optimized calls;
  R5 supplies the real recursive/complex-flow extensions once supported.
- Timeout before/during execution, expensive operations, cancellation and
  simultaneous limits using controlled clock observations; repeat successful
  values while allowing elapsed time and timeout completion to vary.
- Failure/incomplete versus normal return, stale VM/accounting identities,
  forged metrics and mixed successful/failed calls; subsequent import failure
  preserves prior internal evidence without admitting a public result.

These are future oracle requirements, not new fixtures or coverage credit.
The existing Open requests cannot establish any boundary count or timeout
behavior, and a synthetic receipt cannot discharge the pilot prerequisites.

## Readiness and test design

| Case | Future executable oracle | Required negative control |
|---|---|---|
| Ground direct return | Real verified `idalgo` body, genuine typed concrete call, validated R3 handle, exact returned value | No real caller, wrong argument/type, changed body, forged receipt, or incomplete E1/E2/E4 remains blocked |
| Artifact and version | Same checked entry/version/dependency closure loads; stale/invalid inputs refused | Swapped executable bytes, wrong VM/contract/manifest, missing/private imported target, local source fallback |
| Result classification | Only actual finite normal return emits report; same call repeats its exact result | `fail`, missing operation, missing requires evidence, unsupported instruction, expired budget and stale snapshot yield no accepted result |
| Execution isolation | No ghost-to-runtime flow, global mutation or implicit source evaluation | Ghost value leak, alias/state mismatch, same-name different symbol, reused weaker-policy record rejected |

**R4 choice dispositions (owner decision 2026-11-20 11:00 JST):**
(1) and (4) reference the existing R3 questions; they are not new R4 questions.
- **(1) [R3-Q1](../../mizar-artifact/en/mvm_runtime_export.md#tests-and-decisions-for-the-next-implementation-freeze)
  — Artifact; undecided; blocks R3 implementation freeze and R4 format choice:**
  minimal instruction/return representation consistent with the R3 format.
- **(2) [R1-Q1](./mvm_readiness.md#open-questions) — DECIDED
  2026-11-19 23:28 JST:** retain AST-TYPE/AST-LOWER gates; ordinary-only
  narrowing is not approved. Authentic typed ground invocation remains required.
- **(3) R5-Q1 — VC-owned R4 decision; undecided; blocks R4's first execution:**
  step-charge/timeout/nest accounting and report/framing. Freeze before even
  the pilot runs; R5 retains a reference and its own extensions only.
- **(4) [R3-Q2](../../mizar-artifact/en/mvm_runtime_export.md#tests-and-decisions-for-the-next-implementation-freeze)
  — RESERVED, owner/specification judgment at R3 freeze; unassigned and undecided;
  blocks R3 export and R4 same-module pilot:** staged verified load ordering.
Never decide these by fabricating runnable artifacts.

**Implementation release:** requires R1/R2/R3 independent reviews,
genuine accepted whole-body evidence and real loader prerequisites.
AST-LOWER complete inputs and typed computability facts remain mandatory.
Freeze the smallest executor contract under the existing
`ALG-MVM-EVALUATION` owner, with test-first positive and rejection
oracles. Until then R4 is design-only; R5–R7 are separate.
