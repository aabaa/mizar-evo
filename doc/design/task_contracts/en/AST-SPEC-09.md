# Task AST-SPEC-09: Certified import translation and whole-candidate inspection

Canonical language: English; [Japanese pointer](../ja/AST-SPEC-09.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-proof/en/00.crate_plan.md).
Consumers: parser/checker, core/VC, proof/MVM,
and artifact/build owners as applicable; implementation remains deferred.
Dependencies: AST-SPEC-14.
Authority: [2026-10-09 record](../../../spec/temp/algorithm_on_AST.md),
unit 3, decisions 2/4/16/18/20, A.4 T1–T8, V15–V17, K2–K11; the user's explicit specification-change authorization.
Classification: spec transfer; existing opposing design is design_drift;
new behavior's source_drift and test_gap are deferred, without coverage credit.
Scope: paired §20 finite directed import procedure and all public candidate inspection.
Forbidden: Rust, authority record, existing .miz/expectations, trace status,
coverage achievement and volume-ledger increases; no withdrawn/deferred proposals.
Tests: existing algorithm corpus remains unchanged; specification examples
express obligations rather than parser/typechecker/MVM verification claims.
Acceptance: term/truth distinction, typed UID substitution/guards and Binder construction preserved; ensures never partially deleted; no definition rewrite/self-evaluation loop.
Derived ownership: specification chapters own normative rules; roadmap owns
implementation sequencing; coverage audit owns deferral, with credit unchanged.
Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, source/document consistency, full-gate quality assessment.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; changed local links, EN/JA and protected-scope checks.
Exit: acceptance and reviews resolved; task-only commit and resume checkpoint.
