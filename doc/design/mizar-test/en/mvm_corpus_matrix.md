# MVM R8 — Staged Verification and Integration Corpus

> Canonical English; [Japanese companion](../ja/mvm_corpus_matrix.md).
> Owned by `MVM-R8-CORPUS-DESIGN`. **Design only**, not a set of
> executable fixtures, coverage claims or new test-runner identifiers.

## Stage and source authority

The [staged model](./staged_model.md) defines exactly seven stage IDs:
`lexical`, `parse_only`, `declaration_symbol`,
`type_elaboration`, `formula_statement`, `proof_verification`,
`advanced_semantics`. The [expectation schema](./expectation_schema.md)
has fixed `expected_phase` values ending at `kernel_check`; it
currently has **no runtime/MVM phase**. A future runner may use existing
stages for integrated verification only where semantics and test result
boundaries are supported. Introducing a runtime-specific stage/phase is a
separate reviewed test-infrastructure decision, not this task's authority.

`AST-CORPUS` retains ownership of
reflection-specific test preparation. This R8 matrix also covers ordinary
ground algorithms, but does not duplicate AST-CORPUS's implementation
contract or alter [traceability](./traceability.md) by assertion.
Each future accepted case needs one actual source, matching sidecar,
independently grounded oracle, active owning runner, complete `spec_refs`
and prerequisite stage evidence. Existing fixtures remain unchanged.

## Test-family matrix and completion gates

| R8 design family / producer gate | Positive oracle (future) | Rejection, corruption or metamorphic controls |
|---|---|---|
| **A — original source, resolution, typed call** (R1/R2; Parser/Resolver/Checker) | Genuine declaration and contextual argument rules; separate ordinary term from `expr`/Binder/captured `C`, with resolved identity and source provenance. | Missing/ambiguous target, argument type/order, recovered/edited AST, stale source/snapshot, invalid reflection formation; parser pass never counts as semantic success. |
| **B — body, CFG, contract and VC** (R2; Core/Checker/VC) | Real `idalgo` return profile and then `swapalgo` state / `loopalgo` control profiles; exact statement order, exits and requires/ensures/ghost placement. | Changed body, swapped statement/CFG, wrong local binding or ghost-to-runtime flow; generated Open VC must not become proof/termination/executable credit. |
| **C — trusted algorithm verification** (R2; VC/Kernel/Proof) | All applicable E1 typing/inhabitation/single-valuedness, E2 partial correctness, and E4 for promoted `terminating` algorithm are genuinely accepted and bound to the same body. | Missing/forged/tainted proof, wrong goal/context/polarity or policy, E4 inferred from `return` alone. |
| **D — runtime export and loader** (R3; Artifact/Build/Driver) | Exact accepted executable, signature, contract, private-helper closure, VM version and manifest load consistently for public computational entry. | Tampered bytes/hashes, changed dependencies/versions, private external target, missing runtime artifact, stale manifest, path/symlink violation and prohibited source fallback. |
| **E — real execution / failure and budgets** (R4/R5; VC-owned MVM) | Authentic typed ground direct call actually returns its independently expected value; later local state, branching, loops, calls and supported Pick/match source cases execute. | Noncomputable operation, explicit/implicit `fail`, zero divisor, ghost leak, wrong frame, `steps`/`timeout`/`nest` exhaustion, cancellation; no E5 on failure and no proof of non-return. |
| **F — complete computation admission** (R6; Resolver/Core/VC/Proof) | Genuine direct/transitive closure, exact non-ghost Pick/unordered-set triggers; accepted uniqueness proof or specified `result = t` / boolean-iff exemption per applicable value callee. | Open or forged proof/exemption, omitted imported/private helper, wrong set/sequence classifier, missing contract, fake E0330 on absent producer; no computation discharge. |
| **G — certified conversion, replay and reuse** (R7; Proof/Artifact/Cache/Build) | Independently justified requires/Adm/InputOK; entire ensures and ordinary equality candidates converted by T1–T8 and fully inspected; computed/default versus genuinely all-call replayed kernel_verified under matching policy and exact identity. | UID/context/`C`, T4 capture/empty Binder, T5/6 update-order/KΓ, T7 non-exact size, T8 partial ensures/residual AST; replay failure, strict computed-taint, stale executable, weaker cache hit. |
| **H — end-to-end verified computation** (R2–R7; Driver/test owner) | Real source → checked body/proof → validated executable → actual E5 normal return → accepted R6 → entire certified import → permitted Proof status → atomic Artifact/manifest read-back, with fresh and cached equivalence. | One missing stage, forged witness, stale edited source/dependency, failed/interrupted publication or cache, different replay configuration, mismatched output; no status or coverage promotion. |

R8-A–H are **test-design families**, not eight new `mizar-test`
stages or runnable task IDs. Every positive corpus case is blocked
until its **entire** actual producer chain and independent semantic
oracle exist. A negative test may be installed earlier at its authentic
stage, but may not pretend later phases were reached.

## Existing non-execution baselines and evidence rules

- [`idalgo`](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz)
  and its [sidecar](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.expect.toml)
  claim **VC generation with Open obligations**, not accepted whole-body
  proof or MVM execution.
- [`swapalgo`](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz),
  [`loopalgo`](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz)
  and their sidecars likewise do not claim runtime success.
- [`by computation` source](../../../../tests/miz/pass/algorithms/pass_proof_verification_computation_justification_001.miz)
  and [sidecar](../../../../tests/miz/pass/algorithms/pass_proof_verification_computation_justification_001.expect.toml)
  transport an **Open symbolic request** only, not genuine E5,
  `computed` or `kernel_verified`.
- A self-made synthetic witness, manually marked `Accepted`, mock
  runtime result, reused source text of an unavailable imported artifact
  or a changed expectation is never a positive oracle. Require precise
  source/goal/contract identity, independent expected semantics and
  positive/negative/corruption controls before activation.

## Open questions and dispatch order

- **R8-Q1 — stage and phase admission:** decide the minimal honest
  runner for actual MVM execution, computation acceptance and manifest
  readback. Current `expected_phase` has no MVM variant. Review any
  additional external stage/phase and its dependency/coverage semantics
  explicitly; do not coerce runtime errors to `vc_generation`.
- **R8-Q2 — first end-to-end oracle:** obtain the authentic typed ground
  call (the `idalgo(0)` proposal is not established), complete accepted
  E1/E2/E4 where required, verified export, real evaluation and Proof
  admission. A source snippet is not sufficient.
- **R8-Q3 — release and reuse:** freeze replay/strict-policy positive
  and rejection results plus exact transitive dependency/cache identities
  and proof/witness/manifest publication order, including failure on
  stale/corrupt inputs.

R8-E–H activation awaits R4–R7
real producers and required stage runner. No change to existing
`.miz`, expectations, `spec_trace.toml`, acceptance status,
proof policy or compiler behavior follows from this design.
