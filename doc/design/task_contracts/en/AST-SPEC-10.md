# Task AST-SPEC-10: Reflection diagnostics

Canonical language: English; [Japanese pointer](../ja/AST-SPEC-10.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-diagnostics/en/00.crate_plan.md).
Consumers: parser/checker, core/VC, proof/MVM,
and artifact/build owners as applicable; implementation remains deferred.
Dependencies: AST-SPEC-09.
Authority: [2026-10-09 record](../../../spec/temp/algorithm_on_AST.md),
unit 3, A/B diagnostics B1–B8, G/S/V/K, decisions 4/6/10/18/20; the user's explicit specification-change authorization.
Classification: spec transfer; existing opposing design is design_drift;
new behavior's source_drift and test_gap are deferred, without coverage credit.
Scope: paired §22 diagnostic mapping and code index; no diagnostic registry activation.
Forbidden: Rust, authority record, existing .miz/expectations, trace status,
coverage achievement and volume-ledger increases; no withdrawn/deferred proposals.
Tests: existing algorithm corpus remains unchanged; specification examples
express obligations rather than parser/typechecker/MVM verification claims.
Acceptance: rejection causes/stages/positions preserved; codes assigned only after repository uniqueness check; existing semantic outcomes and coverage are not rebaselined.
Derived ownership: specification chapters own normative rules; roadmap owns
implementation sequencing; coverage audit owns deferral, with credit unchanged.
Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, source/document consistency, full-gate quality assessment.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; changed local links, EN/JA and protected-scope checks.
Exit: acceptance and reviews resolved; task-only commit and resume checkpoint.
