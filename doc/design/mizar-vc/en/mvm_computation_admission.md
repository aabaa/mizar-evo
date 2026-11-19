# MVM R6 — Complete Computation-Closure Admission

> Canonical English; [Japanese companion](../ja/mvm_computation_admission.md).
> R6 design task `MVM-R6-ADMISSION-DESIGN`; no accepted computation
> or runnable implementation is established by this document.

## Normative boundary and qualifying condition

[§20.9.2](../../../spec/en/20.algorithm_and_verification.md#2092-by-computation-proof-step)
requires the result of a computation used to discharge an obligation to
be independent of nondeterministic choices. The closure is **every
algorithm actually evaluated directly or transitively**, not merely the
named root, locally visible source or a one-level caller list. For
each **value-returning** `g` containing a non-ghost Pick or unordered
**set** `for ... in`, admit either an accepted obligation:

```text
forall x1,...,xn,r1,r2.
  is_A1(x1) & ... & P(x1,...,xn)
  & Q(x1,...,xn,r1) & Q(x1,...,xn,r2) implies r1 = r2
```

or the specification's exact exemption for an `ensures` of form
`result = t` or `result = true iff phi`, where the right side
contains no occurrence of `result`. The formula above is a readable
schematic copy of §20.9.2, **not** a new grammar or encoding; VC
owns capture-avoiding typed instantiation.

Non-trigger value-returning bodies (no non-ghost Pick and no unordered
set iteration) require neither this VC nor an exemption. A procedure
with no value result is not required to prove result uniqueness.
Ghost-only Pick does not trigger it; ghost Pick alongside unordered
set iteration **does** trigger through the set, and a sequence loop
alongside non-ghost Pick **does** trigger through Pick. Set iteration
cannot be made ordered by choosing a canonical runtime enumeration.
E1 function-graph single-valuedness is a **separate** gate: passing
R6 or observing a VM result must not establish E1, E2, E4, requires,
`Adm`/`InputOK` or replay.

## Producer and acceptance DAG

| R6 slice and owner | Authenticated input and output | Missing or invalid input |
|---|---|---|
| **R6-A — local/imported closure** (Resolver/Artifact inputs; VC owner) | Resolve each original local direct/transitive evaluated call, including internal private helpers; import only **validated** public computational exports and their complete executable/dependency body closure. Preserve selected algorithm identity, signatures, source/versions, call provenance and complete request/closure association. | Unresolved/ambiguous call, missing body, missing helper, private cross-package target, stale/altered export or incomplete transitive closure -> unavailable/Open; no local source fallback or invented callee |
| **R6-B — classification/exemptions** (Checker/Core facts; VC owner) | Distinguish value/procedure, non-ghost versus ghost Pick, unordered set versus ordered sequence, typed `ensures` and whether `result` occurs in exact resolved right side. Only the two spec-enumerated forms exempt; retain each reason with actual callee/contract identity. | Missing type, original body/loop-classification, contract, or untrusted exemption flag -> no exemption or admission; no spelling-only search |
| **R6-C — VC generation** (VC 58 local; full admission successor) | For each triggering, non-exempt callee, form the exact parameter-type-guard, precondition `P`, and twice-instantiated complete `Q` uniqueness goal; retain original goal/context, dependency/version and call identity. Local VC58 may yield **Open** goals without full imported closure and without acceptance. | Wrong guard or result binder, omitted `requires`/`ensures`, substituted origin, missing/synthetic proof -> remain Open; generation never means proved |
| **R6-D — Proof admission** (Proof owner) | Validate completeness of the entire request-linked closure and genuinely accepted per-callee proof or authenticated exact exemption before **any computation-based discharge**. Proof owns E0330 when a real applicable nondeterminism obligation fails; publish no fact from mere VC/VM report. | Rejected/Open/stale/forged proof or missing closure prevents admission. Distinguish actual nondeterminism rejection from unavailable producer and backend/timeout failure; do not invent an E0330 on a merely absent input |

Ordering: original source call and typed Core/CFG classification +
R3 validated local/imported computational export -> R6-A/B ->
R6-C per-callee accepted-proof requests -> R6-D admission ->
R7 certified computation import. R4/R5 execution can provide an
internal normal-return report, but it cannot cause R6-D to pass.
Accepted admission is a gate **for discharging a mathematical
obligation using a computation**; R6 does not synthesize an E5
normal return or require a future R7 computation record to build
the R3 executable (no circular dependency).

## Test-first design, not activated cases

| Scenario | Expected admission rule | Negative or corruption control |
|---|---|---|
| Direct ordinary value `idalgo` | Non-trigger shape if the actual checked body/classification confirms no Pick or unordered set. Existing [source](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) has only an [Open VC expectation](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.expect.toml): not a runtime or accepted computation positive. | Alter body/call/provenance, mark Open as proved, omit verified body or forge result type |
| Set-loop `exists_greater` from §20.9.2 | Trigger is present; `result = true iff phi` exemption only when genuine resolved `phi` contains no `result`, not a proof inferred from an example | Tamper with `ensures`, inject `result` dependency, replace set/sequence classifier, forge exemption |
| Non-ghost Pick with relational `ensures` | Generate and accept actual uniqueness goal before discharge; source-run value or E1 alone is insufficient | Open/failed proof, false claim that observed two runs agree, wrong guarding `P` or result substitution |
| Closure with nested helpers and imported public callee | All direct/transitive eligible value callees are accounted for; private helpers inside a validated export still inspected | Remove nested edge, substitute other package/body, cross-package private call, unavailable artifact, reordered/stale dependency graph |
| Non-trigger controls | Ordered sequence without Pick, ghost-only Pick, void procedure do not acquire a value-result uniqueness VC solely from those features | Sequence with non-ghost Pick or set with ghost Pick still triggers; missing classifications fail closed |
| Accepted gate versus runtime/replay | Correct source/closure/proof records allow R6-D decision *only*, not `computed` or `kernel_verified` | Raw E5 report, R7 replay evidence, hash/status flag, weak cache record or stale goal does not bypass R6 |

## Open Questions and implementation release

- **Q1:** establish an exact typed call/body/dependency closure source
  and a validated imported export (R2/R3), including source order,
  complete transitive helper identities and default-public visibility;
  local VC58 cannot manufacture imported closure.
- **Q2:** confirm the minimal Proof-owned receipt/binding of one
  accepted uniqueness VC versus one syntax exemption to the **same**
  source request/contract/closure. Define fail-closed revalidation on
  source, contract, policy, dependency and context change without
  inventing a new public acceptance model.
- **Q3:** specify E0330 mapping and strict provenance only after a
  supported real failing uniqueness obligation; absent producer/Open
  status is not automatically evidence of nondeterminism.

**First possible implementation freeze:** R6-A local source call-edge
inventory and R6-B classification where exact Resolver/Checker/Core
producers exist, using the existing VC 58 contract. R6-C full
export-fed generation awaits R3 actual validated export; R6-D
acceptance awaits genuine Proof evidence. No R6-A–D implementation
is authorized merely by this design, and no unchanged `.miz`
expectation/trace or coverage is promoted.
