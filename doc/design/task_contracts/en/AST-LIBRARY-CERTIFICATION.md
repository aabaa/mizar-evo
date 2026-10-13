# Task AST-LIBRARY-CERTIFICATION: pinned reflected library certification
Canonical language: English; [Japanese pointer](../ja/AST-LIBRARY-CERTIFICATION.md).
Status: dependency-blocked. Tier: full gates. Owners: Checker and Proof; inputs: Resolver/Artifact authentic library declarations and proof material.
Authority: [§20.5.3 correspondence and M1–M7](../../../spec/en/20.algorithm_and_verification.md#2053-reflected-syntax), [§13.10](../../../spec/en/13.term_expression.md#1310-function-binder--), [architecture15](../../architecture/en/15.kernel_certificate_format.md#reflection-rule-and-execution-boundary).
Classification: source_drift/test_gap; existing pinned correspondence table is authority, not a certification producer.
Readiness: inventory/design of exact library inputs and proof checking can start; implementation adoption waits for authentic resolved definitions and accepted premises/proofs from Mizar8.1.15/MML5.94.1493.
Scope: produce and validate the finite pinned library correspondences and foundational proofs needed by the supported classification, valuation and Binder construction; preserve exact identities, versions and conditional premises.
Boundary: Resolver/Artifact supply authentic definitions/proofs; Proof validates through existing acceptance authority; Checker adopts only validated supported correspondences. Per-use conditional instances belong to AST-FORMATION-PROOFS; E1/E2/E4 remain AST-CONTRACT.
Prerequisites: inventory each required table entry, real declaration/proof source and checker/proof consumer; freeze the smallest genuine validation path before implementation. Missing paths remain named dependencies, never synthesized from spelling.
Invariants: identity/premise validation follows the linked Proof boundary; no broad MML migration, same-name certification or unsupported carrier promotion.
Failure: missing/stale/invalid correspondence or proof keeps the affected classification/meaning unavailable; an unresolved foundation or acceptance meaning is spec_gap with a precise human-owned release condition.
Forbidden: inventing library definitions, proofs, axioms, schemes, proof APIs or kernel rules; using completed AST-SPEC documents as acceptance evidence; broadening supported carriers/structure updates.
Owner details/tests: [typed formation](../../mizar-checker/en/typed_ast.md#reflected-syntax-formation), [Proof formation](../../mizar-proof/en/policy.md#formation-evidence-boundary).
Tests: authentic pinned positive correspondences, version/content/identity mismatch, missing conditional premise/proof and same-spelling impostor; supported versus deferred carrier cases. Record each real source-to-acceptance route before executable tests.
Acceptance: only genuine validated library material enables its bounded correspondence; absent inputs do not silently certify or broaden classification.
Audit: chapter13/20 ownership and unavailable producers only, no proof/coverage credit.
Reviews: independent specification, test sufficiency, implementation, volume/scope and source/EN-JA consistency; full protocol gates.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test.
Owner indexes: [mizar-checker index](../../mizar-checker/en/00.crate_plan.md), [mizar-proof index](../../mizar-proof/en/00.crate_plan.md), [mizar-resolve index](../../mizar-resolve/en/00.crate_plan.md), [mizar-artifact index](../../mizar-artifact/en/00.crate_plan.md).
Existing foundations: [binding identity](../../../../crates/mizar-checker/src/binding_env.rs), [kernel evidence tests](../../../../crates/mizar-kernel/src/formula_evidence/tests.rs), [proof status tests](../../../../crates/mizar-proof/src/status/tests.rs); these provide no reflected certification/formation route.
