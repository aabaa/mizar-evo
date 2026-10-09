# Task AST-SPEC-07: Binder capture and reconstruction

Canonical language: English; [Japanese pointer](../ja/AST-SPEC-07.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-checker/en/00.crate_plan.md).
Consumers: parser/checker, core/VC, proof/MVM,
and artifact/build owners as applicable; implementation remains deferred.
Dependencies: AST-SPEC-06.
Authority: [2026-10-09 record](../../../spec/temp/algorithm_on_AST.md),
unit 2b, decisions 16/17/21/23, A.1 S1/S2/S6, G6/S6, V5–V9/V18, K3/K4/K6/K8; the user's explicit specification-change authorization.
Classification: spec transfer; existing opposing design is design_drift;
new behavior's source_drift and test_gap are deferred, without coverage credit.
Scope: paired §13 Binder pattern/capture/reconstruction and R2 scope rules.
Forbidden: Rust, authority record, existing .miz/expectations, trace status,
coverage achievement and volume-ledger increases; no withdrawn/deferred proposals.
Tests: existing algorithm corpus remains unchanged; specification examples
express obligations rather than parser/typechecker/MVM verification claims.
Acceptance: stored UIDs preserved; C default codomain and complete-input correspondence retained; alpha avoidance changes only colliding bound occurrences; roundtrip structural equality.
Derived ownership: specification chapters own normative rules; roadmap owns
implementation sequencing; coverage audit owns deferral, with credit unchanged.
Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, source/document consistency, full-gate quality assessment.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; changed local links, EN/JA and protected-scope checks.
Exit: acceptance and reviews resolved; task-only commit and resume checkpoint.
