# Task AST-SPEC-05: Typed meaning and valuation

Canonical language: English; [Japanese pointer](../ja/AST-SPEC-05.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-checker/en/00.crate_plan.md).
Consumers: parser/checker, core/VC, proof/MVM,
and artifact/build owners as applicable; implementation remains deferred.
Dependencies: AST-SPEC-04.
Authority: [2026-10-09 record](../../../spec/temp/algorithm_on_AST.md),
unit 2a, decisions 8/13/15–17, A.2 M1–M3/M5–M8, V1–V4/V10/V11/V15/V18, K2/K4–K6; the user's explicit specification-change authorization.
Classification: spec transfer; existing opposing design is design_drift;
new behavior's source_drift and test_gap are deferred, without coverage credit.
Scope: paired §20 environment/meaning/valuation/contract interpretation rules.
Forbidden: Rust, authority record, existing .miz/expectations, trace status,
coverage achievement and volume-ledger increases; no withdrawn/deferred proposals.
Tests: existing algorithm corpus remains unchanged; specification examples
express obligations rather than parser/typechecker/MVM verification claims.
Acceptance: typed updates retain dependencies and guards; opaque substitution is UID-based capture avoidance; ghost/runtime boundary and fixed return types preserved.
Derived ownership: specification chapters own normative rules; roadmap owns
implementation sequencing; coverage audit owns deferral, with credit unchanged.
Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, source/document consistency, full-gate quality assessment.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; changed local links, EN/JA and protected-scope checks.
Exit: acceptance and reviews resolved; task-only commit and resume checkpoint.
