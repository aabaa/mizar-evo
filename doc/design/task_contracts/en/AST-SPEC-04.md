# Task AST-SPEC-04: Immutable syntax structure and executable traversal

Canonical language: English; [Japanese pointer](../ja/AST-SPEC-04.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-core/en/00.crate_plan.md).
Consumers: parser/checker, core/VC, proof/MVM,
and artifact/build owners as applicable; implementation remains deferred.
Dependencies: AST-SPEC-03.
Authority: [2026-10-09 record](../../../spec/temp/algorithm_on_AST.md),
unit 2a, decisions 8/9/13/17/19/21, A.1 S1–S7, S1–S6, V12/V13, K1/K8/K12; the user's explicit specification-change authorization.
Classification: spec transfer; existing opposing design is design_drift;
new behavior's source_drift and test_gap are deferred, without coverage credit.
Scope: paired §20 structural rules, ordered sequence for-in, renamed expr_size grammar and type/value argument connection for typed sequences.
Forbidden: Rust, authority record, existing .miz/expectations, trace status,
coverage achievement and volume-ledger increases; no withdrawn/deferred proposals.
Tests: existing algorithm corpus remains unchanged; specification examples
express obligations rather than parser/typechecker/MVM verification claims.
Acceptance: structural identity and exact occurrence size are sharing-invariant; children executable; ghost cannot flow into runtime; selected recursion measure may decrease beyond direct children.
Derived ownership: specification chapters own normative rules; roadmap owns
implementation sequencing; coverage audit owns deferral, with credit unchanged.
Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, source/document consistency, full-gate quality assessment.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; changed local links, EN/JA and protected-scope checks.
Exit: acceptance and reviews resolved; task-only commit and resume checkpoint.
