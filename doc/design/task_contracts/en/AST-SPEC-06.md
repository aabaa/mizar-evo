# Task AST-SPEC-06: Binder grammar and function formation

Canonical language: English; [Japanese pointer](../ja/AST-SPEC-06.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-checker/en/00.crate_plan.md).
Consumers: parser/checker, core/VC, proof/MVM,
and artifact/build owners as applicable; implementation remains deferred.
Dependencies: AST-SPEC-05.
Authority: [2026-10-09 record](../../../spec/temp/algorithm_on_AST.md),
unit 2b, decision 23, A.1 S5/A.2 M4, G1–G7, V2/V3/V5/V14/V18, K3/K4/K6; the user's explicit specification-change authorization.
Classification: spec transfer; existing opposing design is design_drift;
new behavior's source_drift and test_gap are deferred, without coverage credit.
Scope: paired §13 Binder formation/meaning, associated §3 boundary and grammar/precedence copies in Appendices A/B and §19.
Forbidden: Rust, authority record, existing .miz/expectations, trace status,
coverage achievement and volume-ledger increases; no withdrawn/deferred proposals.
Tests: existing algorithm corpus remains unchanged; specification examples
express obligations rather than parser/typechecker/MVM verification claims.
Acceptance: fresh UID and all-input domain/range obligations; certified exact function construction handles empty domains and dependent body types; no kernel lambda.
Derived ownership: specification chapters own normative rules; roadmap owns
implementation sequencing; coverage audit owns deferral, with credit unchanged.
Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, source/document consistency, full-gate quality assessment.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; changed local links, EN/JA and protected-scope checks.
Exit: acceptance and reviews resolved; task-only commit and resume checkpoint.
