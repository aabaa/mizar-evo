# MVM R1 — Authentic First-Slice Readiness Inventory

> Documentation-only inventory for MVM-READINESS-MAP;
> Japanese [pointer](../ja/mvm_readiness.md). This inventory does **not** certify
> an executable MVM, an accepted algorithm, or completed Chapter-20 runtime coverage.

## Authority and selection

- Language authority: [§20.1](../../../spec/en/20.algorithm_and_verification.md#201-algorithm-syntax),
  [§20.9](../../../spec/en/20.algorithm_and_verification.md#209-execution-model-and-computability),
  [§20.13.1 E1–E5](../../../spec/en/20.algorithm_and_verification.md#20131-contract-axioms),
  and [§20.13.7 T1–T8](../../../spec/en/20.algorithm_and_verification.md#20137-certified-computation-import-t1t8).
  Existing AST-LOWER,
  AST-CONTRACT,
  AST-ARTIFACT,
  ALG-MVM-EVALUATION,
  AST-COMPUTE and
  ALGDEF-COMPUTATION-ADMISSION
  remain the implementation owners.
- **Source-backed starting profile:** the existing
  [`idalgo(a)` source](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz)
  declares an `object`-result `terminating algorithm` with `ensures result = a`
  and `return a`. Its [expectation](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.expect.toml)
  is an active **`vc_generation`** case for a real, *open* VcSet; it
  explicitly claims **no proof, termination or algorithm acceptance**.
  The existing [state variant](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz)
  is a later control, not part of the minimum executor input.
- **Pilot execution hypothesis (not an accepted test):** a same-module
  application of the verified `idalgo` body to a concrete ordinary argument
  and a normal return of that argument. The literal application
  `idalgo(0)` is only a proposed syntax/typing probe: its source-call
  resolution, numeral-to-`object` admissibility, fully ground value and
  typed execution-input representation have **not** been demonstrated by
  the existing fixture. Do not mark an end-to-end pass expected until the
  specification-derived oracle and genuine caller inputs are reviewed.
  The [separate `0 = 0 by computation` case](../../../../tests/miz/pass/algorithms/pass_proof_verification_computation_justification_001.miz)
  has an [expectation](../../../../tests/miz/pass/algorithms/pass_proof_verification_computation_justification_001.expect.toml)
  limited to an **Open symbolic computation request**, not an executed
  algorithm. It cannot substitute for this pilot.
- Plain ordinary values and reflected `expr` / Binder / reconstruction
  context `C` are different inputs. A proposed ground-only first slice
  does **not** override the current `AST-TYPE` / `AST-LOWER` dependency;
  a reviewed owner and specification decision is required before any
  narrower non-reflected runtime handoff can bypass reflection-specific work.

## Source-to-runtime edge inventory

Legend: **implemented** means only the stated bounded producer exists;
**ready-for-contract** authorizes a scoped *design or freeze*, not executable
output; **dependency-blocked** means the named producer/trust input is absent;
**open-question** means authority must resolve a material implementation
choice. No row asserts that current runtime execution is ready.

| Edge | Classification and exact evidence | Producer → consumer; missing input and negative control |
|---|---|---|
| Original algorithm declaration | **Implemented (bounded).** Source algorithm syntax payload binds a real declaration; [test source](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz). | Frontend/Resolver → Checker. General call application, original body/current source identity and cross-module visibility are not established by declaration bytes. Reject detached/mismatched AST, source or symbol identity. |
| Type-checked algorithm input | **Implemented (bounded).** [`check_source_algorithm_types`](../../../../crates/mizar-checker/src/type_checker.rs) returns an authenticated `SourceAlgorithmCheck` for supported `object`-result profiles. | Checker → Core. Only proven supported profile, not arbitrary value/call computability. Reject recovered AST, wrong typed-arena ownership and missing resolved argument/type evidence. |
| Source-derived body and contracts | **Implemented (bounded).** [`lower_source_algorithms`](../../../../crates/mizar-core/src/elaborator.rs) constructs `CoreIr` algorithm/statement rows; Core semantic payload records source-derived program content. [`ControlFlowIr`](../../../../crates/mizar-core/src/control_flow.rs) exists as a representation. | Checker/Core → VC and eventual executable producer. No complete, authenticated *runtime executable/body* correspondence for an actual ground call. Reject reordered/changed statements, ghosts, source ranges or signature/contract mismatches. |
| Generated contract VC | **Implemented (bounded).** [`generate_source_algorithm_postconditions`](../../../../crates/mizar-vc/src/generator.rs) generates actual formula/seed context; the [ensures/return sidecar](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.expect.toml) explicitly limits credit to an Open VC. | Core → VC. Open VC is not a checked body or proof acceptance. Reject absent caller prerequisites and false promotion from Open to verified. |
| Actual proof evidence / normal return | **Implemented for bounded proof checks, not algorithm acceptance.** Return reflexivity connects a genuine return goal to existing Kernel/Proof checks; state transitivity is another bounded profile. | VC/Kernel/Proof → verified-body owner. Neither a return-step result nor equality `f(x)=r` alone proves E1/E2/E4 or a complete normal-return guarantee. Reject forged witness, wrong goal/context/polarity, Open/tainted status and skipped call obligations. |
| Verified definition and typed invocation | **Dependency-blocked.** [Core algorithm tasks](../../mizar-core/en/todo.md) 42–53, AST-TYPE, AST-LOWER, AST-CONTRACT and VC 53 retain applicable authentic producer and accepted-evidence gates. | Core/Checker/VC/Proof → Artifact/MVM. First freeze must identify definition-side accepted body data for R3 separately from typed invocation data for R4, with their own authorities. A partial algorithm does not require global E4 merely for concrete execution, but its required verified body and applicable contracts must not be invented. |
| Executable serialization, publication and loader | **Dependency-blocked; design ready after owner mapping.** AST-ARTIFACT explicitly defers executable serialization/build-loader protocol. Existing [`VerifiedArtifact` schema](../../../../crates/mizar-artifact/src/verified_artifact.rs) supplies stable proof/metadata records, **not** an authenticated executable implementation. | Core/Proof → Artifact → Build/Driver loader → VC MVM. Missing exact executable content/verified-body binding, visibility, transitive dependencies and validating loader. Reject swapped bytes, stale versions, nonpublic imports, missing/corrupt artifacts and dependency-source fallback. |
| MVM evaluator and results | **Dependency-blocked.** ALG-MVM-EVALUATION explicitly remains blocked; [computation request generator](../../mizar-vc/en/generator.md#bounded-computation-request-generation) emits an Open hint, not an evaluator. | Validated Artifact + concrete input → VC-owned MVM → real E5 normal-return report. Missing actual call-by-value runtime and exact report/limits. Reject wrong body/input/context/`C`, noncomputable call, `fail`, ghost flow and steps/timeout/nest exhaustion without inventing a non-return theorem. |
| Full computation-closure admission | **Dependency-blocked.** ALGDEF-COMPUTATION-ADMISSION, VC 58. | Artifact call/dependency closure + VC → Proof. Check every applicable direct/transitive value callee and a genuine accepted uniqueness obligation or specified syntactic exemption before **proof discharge**. Missing/forged/partial closure, rejected or Open proof refuses admission. |
| `by computation` fact import/replay | **Dependency-blocked.** AST-COMPUTE, AST-ARTIFACT later provenance, and Build computation policy. | MVM normal result + checked contract/admission → Proof → Artifact/Cache. Independent `requires`/`Adm`/`InputOK`, finite T1–T8 whole-candidate conversion, computed versus kernel-replayed identity and strict release are not discharged by a raw return. Reject residual internal syntax/`C`, replay downgrades, weak-cache hits and fabricated candidates. |
| Executable-stage tests | **Ready-for-contract only for spec-derived test design.** AST-CORPUS permits stage-specific test-first work after an owner freeze. Existing VCs/snapshots do not prove runtime execution. | Test → every responsible producer. No activation or changed expectation until an authentic producer is exercised. Keep test-stage-specific pass/fail and corruption controls distinct. |

The [extraction `RuntimeIr` task](../../mizar-doc/en/todo.md)
is a separate code-extraction strand; it does not authorize reuse as an MVM
executable format or creation of a second unsynchronized source semantics.
The missing `mizar-doc/en/runtime_ir.md` design remains on that strand.

## Dependency DAG and ordered owner freezes

```text
R1 source/owner inventory
   |
   +--> R2 definition lane:
   |       genuine declaration + checked Core body/CFG + applicable
   |       accepted VC/Proof obligations (E1/E2, E4 when required)
   |       -> R3 reusable verified executable + validating loader
   |           (no concrete caller needed for publication)
   |
   +--> R2 invocation lane:
   |       resolved original typed call + complete input/context/C
   |       -> join R3 validated loader at R4
   |       -> R4 finite normal return, or failure/incomplete
   |       -> R5 supported state/control/failure extensions
   |
   +--> R6 local VC58 preparation: authentic Resolver call edges,
   |       Core typed Pick/set classification and actual contracts
   |       (may proceed before R4 execution)
   |       + R3 validated imported callee/closure inputs
   |       -> R6 Proof acceptance, bound to full evaluated closure
   |       (required before any computation-based discharge)
   |
   +--> R8 staged test design; activation follows actual producer

R4 normal report + R6 accepted closure + independent requires,
Adm/InputOK, verified contracts and formation/policy inputs
   -> R7 whole-candidate T1-T8 conversion + replay/strict policy
   -> later Artifact computation-record publication / Cache reuse
```

Runtime export in R3 must **not** depend on the post-R7 computation record:
doing so would make R3 → R4 → R7 → R3 circular. A minimal executor may
return an internal report without accepting a `by computation` proof;
R6 must precede computation **discharge**, not fabricate runtime inputs.
MVM remains owned by the existing VC contract; this inventory proposes no
new crate, public status, Kernel rule or duplicate `RuntimeIr`.

| Roadmap slice | One owner for next freeze | Trigger and explicit blocker |
|---|---|---|
| R2 | Core (`mizar-core`); coordinate Checker/VC/Proof through their existing contracts | Separate a verified **definition/body** for R3 from typed **invocation** for R4; the definition export has no ground-call prerequisite. If ordinary ground evaluation needs distinct authority from reflected AST-TYPE/LOWER, first resolve that by specification review; do not treat source-only Core content as executable. |
| R3 | Artifact (`mizar-artifact`), Build/Driver as consumers | Freeze reusable executable representation/loader after the R2 accepted definition and required imports, **not** after a concrete call. Do not fabricate export bytes to satisfy MVM. |
| R4–R5 | VC (`mizar-vc`) under ALG-MVM-EVALUATION | Freeze executor implementation only after R2 checked content + R3 validating export and an authentic supported ground-call test; later source features follow their actual CFG producers. |
| R6 | VC for closure generation, Proof for acceptance; existing joint ALGDEF task | Local call-closure/VC58 preparation may proceed with real source facts **before** R4; imported closure requires R3 validated exports, and final Proof admission requires the complete executed/dependency closure and genuine accepted uniqueness or exemptions. No Open credit. |
| R7 | Proof for conversion/acceptance, Artifact for subsequent provenance | Requires actual R4 normal return and R6 accepted prerequisites plus existing policy/formation contracts; no early certified-result publication. |
| R8 | Test (`mizar-test`) | Freeze a spec-derived **test-first** corpus family separately, with independent pass/fail oracles. Parser-only additions can precede runtime producers; proof/runtime activation cannot. |

## Test design (not implemented/activated)

1. Pilot source: retain the unchanged `idalgo(a)` VC baseline; propose
   `idalgo(0)` only as a separate provisional ground-call typing/parse
   probe. An eventual positive MVM oracle requires a *genuine accepted*
   verified body, validated runtime artifact, a typed concrete input and
   actual normal return. Reject mismatched source/function/argument types.
2. R2: correct matched original body, call, signature and contract versus
   changed/reordered body, stale typed node, wrong module/snapshot,
   unresolved call, incomplete/failed VC, or substituted proof receipt.
3. R3: valid same-version public executable with exact transitive private
   helper closure versus changed bytes/hash, forged proof identity,
   private cross-package call, absent imported artifact and dependency
   source fallback.
4. R4–R5: supported call-by-value/return; local state and branch control
   when their source/CFG producers are ready; `fail`, noncomputable
   operation, explicit/implicit partial failure, missing transitive callee,
   gas/time/depth limit, ghost-to-runtime leak; no fabricated return evidence.
5. R6–R7: no Pick/unordered-set trigger control; each triggering closure
   callee with genuine uniqueness/exemption versus Open/forged/missing
   premise; independent `requires` and complete `ensures` with
   T1–T8 residual inspection; `computed`, requested replay, strict
   release, stale policy and weak-cache-hit rejection.
6. Maintain separate corpus stages: parser, type/formation, Core/CFG,
   VC/proof, executor normal-return and certified computation. Never
   rebaseline the two existing `active_proof_verification` sidecars to
   claim MVM runtime acceptance.

## Open questions

- **Q1 (owner decision, blocking R2):** Is the current reflected
  `AST-TYPE`/`AST-LOWER` acceptance required for an ordinary ground-value
  algorithm, or can Core/Checker specify a source-derived ordinary-only
  runtime handoff without changing §20? The current ALG-MVM contract lists
  AST-LOWER as a dependency. No exemption is approved here.
- **Q2 (R2/R3):** Which existing verified-body, policy and obligation
  evidence is *actually sufficient* to bind the executable to the
  verified algorithm for a **partial** concrete call? The checked return
  VC and source program bytes are not alone such a binding. Preserve
  E1/E2/E4 distinctions.
- **Q3 (R3):** Should the private MVM encoding share a representation with
  the separately planned extraction `RuntimeIr`, or use a distinct
  authenticated executable format? No implementation choice or new crate is
  authorized by this inventory.
