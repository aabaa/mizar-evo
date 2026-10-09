# Task AST-SPEC-13: Final integration and implementation ownership

Canonical language: English; [Japanese pointer](../ja/AST-SPEC-13.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-core/en/00.crate_plan.md).
Consumers: parser/checker, core/VC, proof/MVM,
and artifact/build owners as applicable; implementation remains deferred.
Dependencies: AST-SPEC-12.
Authority: [2026-10-09 record](../../../spec/temp/algorithm_on_AST.md),
unit 6, decisions 1–23 A/B G/S/V/K and finalized formal specs; the user's explicit specification-change authorization.
Classification: spec transfer; existing opposing design is design_drift;
new behavior's source_drift and test_gap are deferred, without coverage credit.
Scope: architecture EN/JA boundaries, roadmap/coverage implementation TODOs and paired §7/11/18/19/22 grammar/R3/E1/termination consistency and sequence guidance.
Forbidden: Rust, authority record, existing .miz/expectations, trace status,
coverage achievement and volume-ledger increases; no withdrawn/deferred proposals.
Tests: existing algorithm corpus remains unchanged; specification examples
express obligations rather than parser/typechecker/MVM verification claims.
Acceptance: source/tests/trace achievement preserved; owners retain unimplemented stages without duplicate rules; generated rules and MVM trust separated from final public inspection.
Derived ownership: specification chapters own normative rules; roadmap owns
implementation sequencing; coverage audit owns deferral, with credit unchanged.
Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, source/document consistency, full-gate quality assessment.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; changed local links, EN/JA and protected-scope checks.
Exit: acceptance and reviews resolved; task-only commit and resume checkpoint.
