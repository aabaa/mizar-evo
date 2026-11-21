# Task P11-ACCEPTED-DEFINITION: Bounded accepted-definition freeze

Canonical language: English; [Japanese pointer](../ja/P11-ACCEPTED-DEFINITION.md).
Status: design/contract frozen; implementation deferred. Tier: light, documentation only; no acceptance credit.
Primary owner: [Core plan](../../mizar-core/en/00.crate_plan.md).
Consumers: Proof, VC, Kernel, Artifact/Build R3.
Dependencies: none for this freeze; implementation retains existing producer and runtime gates.

## Authority and evidence

Authority: [§20.1.2](../../../spec/en/20.algorithm_and_verification.md#2012-parameters-and-return-type), [§20.2.6](../../../spec/en/20.algorithm_and_verification.md#2026-jump-statements-return-break-continue-fail), [§20.7.2](../../../spec/en/20.algorithm_and_verification.md#2072-terminating-algorithms-and-promotion-to-func), [E1–E5](../../../spec/en/20.algorithm_and_verification.md#20131-contract-axioms).
Predecessor: [ALGDEF-VC53-NORMAL-RETURN](ALGDEF-VC53-NORMAL-RETURN.md), including typing/single-valuedness schema associations; these are prerequisites, not whole-definition acceptance.
Existing oracle: [idalgo source](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz), [unchanged Open expectation](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.expect.toml), [source-return controls](../../../../crates/mizar-test/src/runner/proof_verification/term_proof_tests/source_algorithm_return.rs).
Classification: design_drift in correspondence after E1 schema association and native adoption; aggregate acceptance remains unimplemented.

## Frozen scope and ownership

Freeze the [bounded acceptance boundary and refusal matrix](../../mizar-core/en/mvm_correspondence.md#bounded-accepted-definition-boundary) for E1 typing, single-valuedness, all-typed-input inhabitation, E2 postcondition and E4 normal return.
That owner defines same-identity source/check/Core/snapshot/policy binding, atomic adoption and definition-side R3 eligibility inputs. It does not select an API, payload or implementation.
Lower stages retain their [VC profile](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design), [Proof original identities](../../mizar-proof/en/status.md#source-algorithm-direct-return-request-identity), [native policy](../../mizar-proof/en/policy.md#bounded-native-positive-result-policy) and [authenticated raw slices](../../mizar-vc/en/dependency_slice.md#authenticated-direct-return-core-contribution).
Affected artifacts: this paired contract and Core correspondence EN/JA; owning-plan links are indexing only.
Forbidden: Rust or test changes, public seams, spec/expectation/trace changes, status.md edits, runtime/export activation, fabricated evidence, weakening policy or reflection gates.
Deferred termination G10 stays parked and unregistered here; it gates P13, not this freeze. R3-Q2/R5-Q2 remain G/RESERVED/unassigned.
No coverage/audit ownership, status or deferral changes follow from the freeze.

## Future implementation test matrix

Cap: six case families; reuse existing controls. Additional families require a cited specification clause or .miz test.

| Input shape | Required outcome |
|---|---|
| Genuine bounded definition, all five accepted roles, same inputs/current policy | Atomic bounded adoption; preserve raw graph and residual eligibility blockers. |
| Each missing, duplicate, swapped, foreign or unaccepted role; wrong goal/context/polarity/schema | Refuse entire aggregate; no partial adoption or substituted E2/E4. |
| Edited source/check/symbols/body/contracts/signature/CFG/exits or different Core arena | Authentication failure or refusal; unchanged names/hashes cannot authorize it. |
| Different/stale module/snapshot/dependency provenance or current policy disabled/schema changed | Refuse; earlier fingerprints and local successes grant no authority. |
| Unsupported calls/loops/Pick/recursion/reflection/recovered or ghost flow | Refuse bounded profile; retain existing broader obligations. |
| Accepted local results with Open/Deferred/unknown/incomplete residuals; with/without typed invocation | No complete definition/R3 readiness or cache credit; invocation neither repairs nor gates definition adoption. |

## Freeze exit

Independent authority/equivalence and volume/scope reviews cover EN/JA, matrix sufficiency and source/design consistency; no executable checks are added.
Run `CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy` and `CARGO_BUILD_JOBS=4 cargo test -p mizar-test --test lint_policy`, plus affected local links/fragments and `git diff --check`.
Full regression follows lane cadence; fmt/Clippy/owner/E2E checks are inapplicable to this docs-only unit.
Exit: paired freeze and required indexes committed locally; no implementation or readiness claim. Mandatory T2 acceptance-boundary review remains an orchestrator completion gate.
