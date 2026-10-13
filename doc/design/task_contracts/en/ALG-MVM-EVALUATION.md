# Task ALG-MVM-EVALUATION: Verified-body evaluation producer
Canonical language: English; [Japanese pointer](../ja/ALG-MVM-EVALUATION.md).
Status: blocked. Tier: full gates. Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md) (verification-time execution producer).
Consumers: mizar-proof trust/import policy, artifact/build runtime interfaces and cache, mizar-test.
Dependencies: ALGDEF-COMPUTATION-ADMISSION before any computation-based discharge; AST-LOWER complete inputs, verified body/call correspondence, typed computability facts and validating runtime artifacts; exact executable/body and loader boundaries must be frozen before code.
Readiness: none of those runtime producers is supplied by an open computation hint; implement only after the chosen source slice has authentic inputs and a reviewed execution-format/correspondence design.
Authority: [§20.9.1–20.9.4 and E5](../../../spec/en/20.algorithm_and_verification.md); [pre-ATP boundary](../../architecture/en/07.vc_generation.md#pre-atp-discharge-is-deterministic-and-explainable).
Existing discharge caller-supplied outcomes are not execution evidence or the MVM resource policy.
Gap: source_drift/test_gap; existing [bounded request](../../mizar-vc/en/generator.md#bounded-computation-request-generation) is not execution.
Boundary: MVM produces actual normal-return reports for the verified executable, complete logical input/context/C, exact versions and transitive calls. AST-COMPUTE owns contract conversion, public inspection and trust/replay acceptance; the kernel does not run the VM.
Scope: execute supported ground/direct or explicit calls under the specified steps/timeout/nest limits; erase ghost state and forbid ghost flow into runtime. Partial algorithms may execute; requires and computability remain mandatory. Missing artifacts cannot be replaced by available dependency source.
Failure/resource exhaustion produces no new returns, contract fact or non-return proof. Preserve exact failure stage; defaults and diagnostics come only from the specification. No new report/API/schema is designed here.
First prerequisite: freeze the private executor and executable/body correspondence boundary with Core/Artifact producers, without introducing a second logical term/AST representation or public seam before real consumers require it.
Tests: linked [request regression](../../../../tests/miz/pass/algorithms/pass_proof_verification_computation_justification_001.miz) remains only an open request; design actual normal return versus explicit/implicit failure, partial child/range failure, requires rejection, gas/time/nest exhaustion, ghost erasure and missing/stale/transitive artifact or input/context/C corruption.
Replay-specific proof construction and strict computed-dependency publication belong to AST-COMPUTE/AST-ARTIFACT; execution never fabricates them. Code extraction/export backends remain parked outside this task.
Audit: §20 runtime stays deferred until genuine execution producers and source routes land.
Reviews: independent specification/documentation, test sufficiency, implementation, volume/scope, consistency and EN/JA, full protocol gates.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test.
Exit: real execution reports are bound to the verified body and complete call; consumer policy remains proof-owned; no unimplemented operation or absent protocol is credited.
