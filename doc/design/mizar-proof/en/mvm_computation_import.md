# MVM R7 — Certified Computation Import and Replay

> Canonical English; [Japanese companion](../ja/mvm_computation_import.md).
> Design task `MVM-R7-CERTIFIED-IMPORT-DESIGN` under Proof.
> No runnable evaluator or computation-accepted witness exists here.

## Authority and stage boundary

[§20.9.2](../../../spec/en/20.algorithm_and_verification.md#2092-by-computation-proof-step)
sets `computed` versus replayed `kernel_verified` trust and limits;
[§20.13.7](../../../spec/en/20.algorithm_and_verification.md#20137-certified-computation-import-t1t8)
defines an ordered finite *partial* conversion T1–T8 and whole-candidate
inspection. The existing `AST-COMPUTE` Proof contract owns admission,
conversion and status; Artifact owns later publication, Cache owns reuse,
Build owns package configuration and R4/R5 own actual MVM execution.

| R7 slice / owner | Mandatory input and effect | Refusal / stage distinction |
|---|---|---|
| **A. Evidence gate (Proof)** | Actual R4/R5 finite normal E5 report, genuine typed complete inputs, original context and reconstruction `C` where present, validated R3 executable/transitive identities, applicable verified E1/E2/E4/contract facts and R6 accepted closure. Independently justify each call's `requires`, `Adm`, `InputOK` and compatibility **without using its own consequences**. | Open/stale/forged contract, policy, source, body, VM version or R6 exemption; failed/incomplete runtime; circular prerequisite: no public fact. |
| **B. Candidate translation (Proof)** | Fix finite whole `ensures` instances and ordinary value equalities; retain guards and original source correspondence. Apply T1–T8 one-way with typed terminal simultaneous UID substitutions, verified resolved meanings and certified Binder/structure facts. | No invented term from a formula or boolean truth condition, spelling-based UID, repeated substitution, unrestricted unfolding/re-reflection or unsupported AST conversion. |
| **C. Whole residual inspection (Proof)** | Inspect formula, bound-variable types, input/result, annotations and all nested collections; publish only an entirely terminal mathematical candidate. Keep internal `returns_f` on later import failure. | Reject entire candidate for residual AST value/type/ref, UID, `C`, `val`, `val_at`, `valid_at` or expr operation. No stripped subformula of `ensures` or partial success. |
| **D. Trust/replay (Proof/Kernel)** | No replay: validated MVM E5 plus accepted contract/conversion may produce `computed` under permitted policy. Replay true: every direct/transitive call has validated defining-equation basis and minimum-kernel SAT evidence, leading to `kernel_verified` only on actual success. | Partial/non-definitional call lacking basis, failed replay, missing per-call evidence or strict `require_kernel_certificates` reject computed; no fallback from replay to MVM trust or kernel relabel of computed dependencies. |
| **E. Publication/reuse (Proof → Artifact/Cache)** | Bind accepted result, complete inputs/context/`C`, executable bytes/versions, entire transitive artifact closure, compatible VM/rule versions, policy and effective replay mode; then stage validated computation provenance/manifest. | Hash/flag alone gives no proof; stale dependency, changed executable, weaker mode/policy cache hit or uncommitted manifest cannot be promoted. R3 executable export must precede this downstream stage. |

## Directed conversion requirements

T1 retains ordinary resolved func/pred/connective identity, typed children
and term-versus-truth slots; T2 admits only genuine **pure R1** correspondence;
T3 preserves opaque leaves and capture-avoiding substitution, never decomposes.
T4 converts Binder with fresh output binder, all-input formation/range
proofs and correct empty-domain behavior. T5 translates the update value
*before* UID assignment; T6 expands authenticated finite `KΓ` once with
equivalent `Adm` proof and guarded consequences. T7 computes exact size,
children and structural equality, not semantic equivalence. T8 separates
internal return/AST evidence from **whole** public candidates.
Boolean `val`/`val_at` comparison rules use certified two-valuedness;
they cannot manufacture boolean terms for function/Binder argument slots.

Execution order is fixed: genuine input/candidates and finite records;
certification; guarded formula traversal with T5/T6 update first and
T6/T7 once; terminal T2/T3 or T1/T4/UID meaning translation; certified
Binder construction; then whole residual inspection. Do not cycle output
through reflection or re-enter the same converter. Resource exhaustion
reports the actual incomplete stage, not a false theorem or normal-return
denial. Ordinary `func` definitions are never opened here.

## Trust controls and next implementation freeze

Manifest `computation_replay` defaults false; explicit per-reference
`replay: true/false` overrides it for **every call**. Replay true demands
the equation-and-kernel basis for the *entire* executed closure. With
`require_kernel_certificates=true`, any transitive unreplayed
`computed` premise is insufficient even after a separate kernel check.
Exact executed content invalidates computation reuse on change despite
possible SemVer compatibility; effective replay mode is in the cache key.

Proposed independent tests: ordinary ground `idalgo` with only genuine
accepted prerequisites (its existing [source](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz)
still has an Open VC), explicit request versus goal call, circular
`requires`, exact whole `ensures`, mismatched UID/context/`C`,
pure-R1 versus mixed-R2, opaque substitution, T4 Binder capture/empty
domain, T5 pre-update, T6 missing KΓ/Adm proof, T7 wrong DAG size,
T8 residual inside quantified types/sequences, boolean term/truth,
transitive replay refusal, strict computed-taint, mode-dependent cache
and stale executable bytes. No existing oracle is relabeled runnable.

**Open (all undecided):**

- **R7-Q1 — Proof; blocks R7-A:** which genuine accepted E5/contract/R6
  and formation inputs are supplied by R4/R5, R6, AST-CONTRACT and formation producers?
- **R7-Q2 — Proof; blocks R7-B and R7-C:** which directed converter/Binder
  proof producer and bounded diagnostic-stage mapping implement
  [§22.4.8](../../../spec/en/22.error_handling_and_diagnostics.md#2248-reflection-and-computation-diagnostics)?
- **R7-Q3 — Proof (record content); blocks R7-D and R7-E:** which
  runtime/executable/Proof computation-record schema, replay/equation
  witnesses and policy identities are supplied to Artifact (manifest)
  and Cache (reuse)? Relate the record to
  [R3-Q1/Q3](../../mizar-artifact/en/mvm_runtime_export.md#tests-and-decisions-for-the-next-implementation-freeze)
  and the [manifest transport design](../../mizar-artifact/en/manifest.md#producer-transport-design).

For imported execution artifacts, R7-Q1/Q3 also refer to the undecided
[R3-N2 trust-basis question](../../mizar-artifact/en/mvm_runtime_export.md#tests-and-decisions-for-the-next-implementation-freeze),
the [Proof imported-promotion boundary](./policy.md#imported-promotion-authority-boundary)
and the [Artifact imported-promotion boundary](../../mizar-artifact/en/manifest.md#imported-promotion-authority-boundary).
These references do not decide whether imported execution artifacts require
that authenticated publication; computation retains its separate
artifact/correspondence checks.

R7-A–E remain non-dispatchable implementation slices until these inputs
and their owner-specific test-first contracts are frozen. No R7 acceptance,
Kernel witness or coverage credit follows from this design.
