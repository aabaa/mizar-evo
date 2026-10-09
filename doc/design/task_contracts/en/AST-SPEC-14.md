# Task AST-SPEC-14: Contextual syntax construction grammar

Canonical language: English; [Japanese pointer](../ja/AST-SPEC-14.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-parser/en/00.crate_plan.md).
Consumers: parser/checker, core/VC, proof/MVM,
and artifact/build owners as applicable; implementation remains deferred.
Dependencies: AST-SPEC-08.
Authority: [2026-10-09 record](../../../spec/temp/algorithm_on_AST.md),
unit 2a prerequisite correction, decisions 2/13/18, A.1 S1/A.2 M2, formal R1/match rules; the user's explicit specification-change authorization.
Classification: spec transfer; existing opposing design is design_drift;
new behavior's source_drift and test_gap are deferred, without coverage credit.
Scope: paired §20 contextual RHS/boolean operands, qualified call grammar, Appendix A copies and initialization/sequence guidance correction.
Forbidden: Rust, authority record, existing .miz/expectations, trace status,
coverage achievement and volume-ledger increases; no withdrawn/deferred proposals.
Tests: existing algorithm corpus remains unchanged; specification examples
express obligations rather than parser/typechecker/MVM verification claims.
Acceptance: formula construction reaches expected expr of boolean RHS; captures stay immutable branch-local syntax, never initialize outer vars; existing boolean formula returns preserved; other RHS term-only.
Derived ownership: specification chapters own normative rules; roadmap owns
implementation sequencing; coverage audit owns deferral, with credit unchanged.
Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, source/document consistency, full-gate quality assessment.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; changed local links, EN/JA and protected-scope checks.
Exit: acceptance and reviews resolved; task-only commit and resume checkpoint.
