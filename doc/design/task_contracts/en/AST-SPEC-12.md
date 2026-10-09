# Task AST-SPEC-12: MVM trust replay and publication policy

Canonical language: English; [Japanese pointer](../ja/AST-SPEC-12.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-proof/en/00.crate_plan.md).
Consumers: parser/checker, core/VC, proof/MVM,
and artifact/build owners as applicable; implementation remains deferred.
Dependencies: AST-SPEC-11.
Authority: [2026-10-09 record](../../../spec/temp/algorithm_on_AST.md),
unit 5, A.3 E5/decision 11, V15–V17 K8–K10; user supplemental decisions: replay config false with local override, evidence-required replay, strict-release rejection of computed; the user's explicit specification-change authorization.
Classification: spec transfer; existing opposing design is design_drift;
new behavior's source_drift and test_gap are deferred, without coverage credit.
Scope: paired §§20/21/23 replay/options/status/config/tooling and architecture08 policy boundary.
Forbidden: Rust, authority record, existing .miz/expectations, trace status,
coverage achievement and volume-ledger increases; no withdrawn/deferred proposals.
Tests: existing algorithm corpus remains unchanged; specification examples
express obligations rather than parser/typechecker/MVM verification claims.
Acceptance: deterministic complete input and validated execution identity; exact resources; no replay fallback; computed distinct from kernel_verified and strict policy preserved.
Derived ownership: specification chapters own normative rules; roadmap owns
implementation sequencing; coverage audit owns deferral, with credit unchanged.
Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, source/document consistency, full-gate quality assessment.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; changed local links, EN/JA and protected-scope checks.
Exit: acceptance and reviews resolved; task-only commit and resume checkpoint.
