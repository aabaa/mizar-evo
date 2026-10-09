# Task AST-SPEC-01: Normal return obligations

Canonical language: English; [Japanese pointer](../ja/AST-SPEC-01.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-vc/en/00.crate_plan.md).
Consumers: parser/checker, core/VC, proof/MVM,
and artifact/build owners as applicable; implementation remains deferred.
Dependencies: AST-SPEC-00.
Authority: [2026-10-09 record](../../../spec/temp/algorithm_on_AST.md),
unit 1 supplement, decision 6, A.3 E4/E5, V17, K9/K12; the user's explicit specification-change authorization.
Classification: spec transfer; existing opposing design is design_drift;
new behavior's source_drift and test_gap are deferred, without coverage credit.
Scope: paired §20 normal-return and Pick obligations.
Forbidden: Rust, authority record, existing .miz/expectations, trace status,
coverage achievement and volume-ledger increases; no withdrawn/deferred proposals.
Tests: existing algorithm corpus remains unchanged; specification examples
express obligations rather than parser/typechecker/MVM verification claims.
Acceptance: normal return differs from stopping; child calls and iteration ranges are included; no failed execution evidence.
Derived ownership: specification chapters own normative rules; roadmap owns
implementation sequencing; coverage audit owns deferral, with credit unchanged.
Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, source/document consistency, full-gate quality assessment.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; changed local links, EN/JA and protected-scope checks.
Exit: acceptance and reviews resolved; task-only commit and resume checkpoint.
