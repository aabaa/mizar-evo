# Task PROOF-SOURCE-DISCHARGE-METADATA: raw same-call acceptance association
Canonical language: English; [Japanese pointer](../ja/PROOF-SOURCE-DISCHARGE-METADATA.md).
Status: implemented (bounded raw same-call association). Tier: full. Owner: [Proof plan](../../mizar-proof/en/00.crate_plan.md), task 27.
Authority: spec §§21.7.1/21.7.5/23.5.1; [sole source owner](../../mizar-proof/en/status.md#source-discharge-metadata).
Dependencies: Proof26 fresh direct-premise checking and VC64 raw component identities; full Proof23/Artifact25 remain open.
Gap: source_drift/test_gap; real source checks have no producer-owned plain KernelAcceptanceMetadata association.
Scope: check_source_registration_discharge_metadata takes the same six inputs as the existing discharge check and returns Result<(KernelCheckResult, PolicyDecision, Option<KernelAcceptanceMetadata>), String>.
Reuse one unchanged source reconstruction/check pipeline: the old check facade delegates to the new entry and discards metadata, retaining its signature, errors, scheduling, normal checks and classifications.
Associate only this call's original handoff, normal wire, actual untainted accepted ProofObligation/BuiltinDischarge result and supplied immutable policy; never accept caller evidence/results/metadata.
Use the sole owner's exact existing digest/version mappings. Only DischargedBuiltin plus actual accepted identity and refutation polarity permits metadata; unavailable/unsupported association yields None, no fabricated fallback, without upgrading classification.
Tests: append genuine single/three registration original/renamed/false definitions at snapshots 0/1 under release/development/interactive and changed schema/profile policies; compare actual handoff components/whole binding, policy and accepted-result identities, deterministic data and unchanged old facade/check outcomes.
Cover actual empty substitutions/absent imports; foreign/missing/correctness/unsupported inputs and denied formats supply no metadata. Existing false-definition direct-premise acceptance remains valid; no synthetic accepted control.
Write scope: existing status.rs facade and in-line association plus appended runner source tests; no new type/field/file/adapter/private helper/dependency or protected spec/.miz/expectation/trace/policy/Artifact guards.
Audit: raw association prerequisite only, no semantic coverage promotion. Checker implementation/toolchain-version authentication, full obligation/source-currentness authentication, trusted witness tokens, distinct Builtin schema, staging/committed publication/reuse and Steps6–8 remain open.
Reviews: independent specification/tests/code/volume/consistency, scoped gates/quality; parent fmt, full Clippy -D warnings and cargo test.
Exit: actual permitted direct candidates provide exact raw metadata from the same check; every unsupported association fails closed and existing behavior/trust boundaries are retained.
