# Task AST-SPEC-02: Reflection prerequisites

Canonical language: English; [Japanese pointer](../ja/AST-SPEC-02.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-checker/en/00.crate_plan.md).
Consumers: parser/checker, core/VC, proof/MVM,
and artifact/build owners as applicable; implementation remains deferred.
Dependencies: AST-SPEC-01.
Authority: [2026-10-09 record](../../../spec/temp/algorithm_on_AST.md),
unit 2a/2b prerequisite tables, decisions 8/13/23, A.1/A.2/A.4, K1/K2/K6/K7; the user's explicit specification-change authorization.
Classification: spec transfer; existing opposing design is design_drift;
new behavior's source_drift and test_gap are deferred, without coverage credit.
Scope: paired §20 fixed-library and surface-classification tables.
Forbidden: Rust, authority record, existing .miz/expectations, trace status,
coverage achievement and volume-ledger increases; no withdrawn/deferred proposals.
Tests: existing algorithm corpus remains unchanged; specification examples
express obligations rather than parser/typechecker/MVM verification claims.
Acceptance: all existing term/formula forms classified; certification uses pinned definitions and proofs, never spelling; unsupported lowering remains explicit.
Derived ownership: specification chapters own normative rules; roadmap owns
implementation sequencing; coverage audit owns deferral, with credit unchanged.
Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, source/document consistency, full-gate quality assessment.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; changed local links, EN/JA and protected-scope checks.
Exit: acceptance and reviews resolved; task-only commit and resume checkpoint.
