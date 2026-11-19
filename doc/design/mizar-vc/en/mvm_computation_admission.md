# MVM R6 — Complete Computation-Closure Admission

> Canonical English; [Japanese companion](../ja/mvm_computation_admission.md).
> R6 design task `MVM-R6-ADMISSION-DESIGN`; no accepted computation
> or runnable implementation is established by this document.

## Normative boundary and qualifying condition

[§20.9.2](../../../spec/en/20.algorithm_and_verification.md#2092-by-computation-proof-step)
requires nondeterministic-choice independence for computation-based
discharge. The closure is **every algorithm evaluated directly or
transitively**, not only the root. For each **value-returning** `g`
with non-ghost Pick or unordered **set** `for ... in`, admit either:

```text
forall x1,...,xn,r1,r2.
  is_A1(x1) & ... & P(x1,...,xn)
  & Q(x1,...,xn,r1) & Q(x1,...,xn,r2) implies r1 = r2
```

or the exact `ensures` exemption `result = t` or
`result = true iff phi`, with no `result` in the right side.
The schematic formula copies §20.9.2; it adds no grammar or encoding.
VC owns capture-avoiding typed instantiation.

A non-trigger value-returning body or void procedure needs no
result-uniqueness VC. Ghost-only Pick does not trigger; unordered set
iteration with ghost Pick **does**, as does sequence iteration with
non-ghost Pick. Canonical runtime enumeration cannot make set order
specified. E1 single-valuedness is **separate**: R6 or VM execution
establishes neither E1/E2/E4, requires, Adm/InputOK nor replay.

## Producer and acceptance DAG

| R6 slice and owner | Authenticated input and output | Missing or invalid input |
|---|---|---|
| **R6-A — local/imported closure** (Resolver/Artifact inputs; VC owner) | Resolve each original local direct/transitive evaluated call, including internal private helpers; import only **validated** public computational exports and their complete executable/dependency body closure. Preserve selected algorithm identity, signatures, source/versions, call provenance and complete request/closure association. | Unresolved/ambiguous call, missing body, missing helper, private cross-package target, stale/altered export or incomplete transitive closure -> unavailable/Open; no local source fallback or invented callee |
| **R6-B — classification/exemptions** (Checker/Core facts; VC owner) | Distinguish value/procedure, non-ghost versus ghost Pick, unordered set versus ordered sequence, typed `ensures` and whether `result` occurs in exact resolved right side. Only the two spec-enumerated forms exempt; retain each reason with actual callee/contract identity. | Missing type, original body/loop-classification, contract, or untrusted exemption flag -> no exemption or admission; no spelling-only search |
| **R6-C — VC generation** (VC 58 local; full admission successor) | R6-C requires R3 exports. For each triggering, non-exempt callee, form the exact parameter-type-guard, precondition `P`, and twice-instantiated complete `Q` uniqueness goal; retain original goal/context, dependency/version and call identity. | Wrong guard or result binder, omitted `requires`/`ensures`, substituted origin, missing/synthetic proof -> remain Open; generation never means proved |
| **R6-D — Proof admission** (Proof owner) | Validate completeness of the entire request-linked closure and genuinely accepted per-callee proof or authenticated exact exemption before **any computation-based discharge**. Proof owns E0330 when a real applicable nondeterminism obligation fails; publish no fact from mere VC/VM report. | Rejected/Open/stale/forged proof or missing closure prevents admission. When a *genuine generated* uniqueness obligation cannot be discharged in an actual computation proof, apply spec E0330; mere absence of a source/export producer is an unavailable input, not fabricated E0330 evidence |

Order: authentic call/CFG -> local R6-A/B preparation
(in parallel with R3/R4 where genuine producers permit); R3 validated
exports -> R6-C goal generation. Final R6-D checks the source closure
against the actual executed-call/dependency record or an independently
authenticated complete execution-closure relation, then accepts only
genuine proofs/exemptions. A static candidate graph or VM return alone
does not attest the complete evaluated closure. R6 gates **proof
discharge**, not R4 execution, emits no E5, and never makes R3 export
depend on later R7 records.

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

- **Q1:** authenticate source calls and imported exports (R2/R3),
  including transitive helpers and visibility; local VC58 cannot
  manufacture imported closure.
- **Q2:** bind each accepted VC or syntactic exemption to the **same**
  request, contract, closure, policy and version; validate on change.
  Proof remains the sole acceptance owner.
- **Q3:** a genuine uniqueness VC that cannot be discharged on a supported
  computation request yields E0330; an unformed producer/closure has no such VC.
  Never infer E0330 from an arbitrary status flag.

**First possible implementation freeze:** R6-A local source-edge
inventory and R6-B classification once authentic producers exist,
under VC58. R6-C requires R3 exports; R6-D requires genuine Proof
acceptance. This design authorizes no R6 implementation or change
to existing `.miz` expectations, traces or coverage.
