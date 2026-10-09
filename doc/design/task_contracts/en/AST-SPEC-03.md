# Task AST-SPEC-03: Expr types and structural match

Canonical language: English; [Japanese pointer](../ja/AST-SPEC-03.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-parser/en/00.crate_plan.md).
Consumers: parser/checker, core/VC, proof/MVM,
and artifact/build owners as applicable; implementation remains deferred.
Dependencies: AST-SPEC-02.
Authority: [2026-10-09 record](../../../spec/temp/algorithm_on_AST.md),
unit 2a, decisions 9/10/13–15/18, A.1 S4/S5, G/S/V12, K2/K12; the user's explicit specification-change authorization.
Classification: spec transfer; existing opposing design is design_drift;
new behavior's source_drift and test_gap are deferred, without coverage credit.
Scope: paired lexical/type/expected-input/match grammar and §19 covariance.
User-approved exception: metadata.rs deferred-word list adds expr; parser
audit records its unimplemented state without corpus or coverage activation.
Forbidden: Rust, authority record, existing .miz/expectations, trace status,
coverage achievement and volume-ledger increases; no withdrawn/deferred proposals.
Tests: existing algorithm corpus remains unchanged; specification examples
express obligations rather than parser/typechecker/MVM verification claims.
Acceptance: expr is reserved; formula reflection stays contextual; match is expr-only with immutable branch captures and otherwise; no annotated-pattern syntax invented.
Derived ownership: specification chapters own normative rules; roadmap owns
implementation sequencing; coverage audit owns deferral, with credit unchanged.
Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, source/document consistency, full-gate quality assessment.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; changed local links, EN/JA and protected-scope checks.
Exit: acceptance and reviews resolved; task-only commit and resume checkpoint.
