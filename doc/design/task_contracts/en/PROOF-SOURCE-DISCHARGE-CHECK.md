# Task PROOF-SOURCE-DISCHARGE-CHECK: fresh direct-premise classification
Canonical language: English; [Japanese pointer](../ja/PROOF-SOURCE-DISCHARGE-CHECK.md).
Status: implemented for bounded classification. Tier: full. Owner: [Proof plan](../../mizar-proof/en/00.crate_plan.md), task 26.
Consumers: source proof integration; general Proof23/Artifact25 publication remains open.
Authority: spec §§21.7.1/21.7.5, §23.4; [source owner](../../mizar-proof/en/status.md#source-discharge-checking), existing Proof policy and Kernel checking.
Dependencies: authentic Checker/Core/original VC/default Discharge, VC63 direct-premise association, normal local wire and Kernel result/policy producers.
Gap: source_drift/test_gap; actual supported discharge candidates stop at Kernel-only tests without a production Proof classification consumer.
Scope: one source facade taking SurfaceResolvedArena, TypedArena, SymbolEnv, BuildSnapshotId, VcId and VerifierPolicy; return existing KernelCheckResult/PolicyDecision or an error.
Freshly reconstruct source/Core/original VC/default output, call VC63, encode its original local handoff and require existing BuiltinKernelEvidence scheduling permission before normal Kernel checking.
Use the actual fresh result with BuiltinDischarge origin in the unchanged evaluator; retain its target, proof-obligation kind, taint, rejection details and classification.
Share only context construction and check_kernel_evidence in one private helper with exactly three real callers: existing ATP check, original built-in activation and this facade.
Preserve old caller-specific errors, parsers, wire/schema/instance selection, scheduling, origin, acceptance criteria, hashes and activation behavior; no broader refactor.
Tests: append real-source single/three-registration original/renamed/false-definition direct leaves, release/development/interactive policies, exact original target and actual DischargedBuiltin classification.
False-definition direct leaves remain valid identical premise/goal; existing correctness-leaf rejection and all ATP/activation controls remain unchanged.
Reject unsupported/correctness/missing targets, foreign source/typed/env and denied Builtin format; distinguish FormulaSubstitution-only from Builtin-only permissions.
Forbidden: new public type/field/file/format, caller evidence/result/receipt injection, Kernel criteria or proof-policy changes, fabricated acceptance/used_axioms, ATP relabeling, selection/activation/publication/reuse, protected spec/.miz/expectation/trace edits.
Source scope: status.rs plus appended runner/tests.rs; existing two callers change only the duplicated context/check block and its now-unused imports.
Audit: bounded fresh classification follow-up only; general by/built-in acceptance, accepted-witness metadata, distinct built-in Artifact schema, committed publication and Steps 6–8 stay open.
Reviews: specification, tests, code, volume/scope, consistency, scoped gates/quality; parent fmt, full Clippy -D warnings and cargo test.
Exit: actual selected direct candidates receive fresh normal checks and unchanged built-in policy classification, with no transferable publication authority.
