# Task AST-SPEC-11: Mathematical contracts and examples

Canonical language: English; [Japanese pointer](../ja/AST-SPEC-11.md).
Status: complete. Tier: full gates. Owner: [owner plan](../../mizar-test/en/00.crate_plan.md).
Consumers: parser/checker, core/VC, proof/MVM,
and artifact/build owners as applicable; implementation remains deferred.
Dependencies: AST-SPEC-10.
Authority: [2026-10-09 record](../../../spec/temp/algorithm_on_AST.md),
unit 4, decisions 1–4/12–17/19–23, D1–D3 G/S/V/K, source examples; the user's explicit specification-change authorization.
Classification: spec transfer; existing opposing design is design_drift;
new behavior's source_drift and test_gap are deferred, without coverage credit.
Scope: paired §18/§20 examples, Appendix D recommendations and sample links.
Forbidden: Rust, authority record, existing .miz/expectations, trace status,
coverage achievement and volume-ledger increases; no withdrawn/deferred proposals.
Tests: existing algorithm corpus remains unchanged; specification examples
express obligations rather than parser/typechecker/MVM verification claims.
Acceptance: order chain shows explicit intermediate claim; occurrence contract preserves guards; full-real-domain differentiation proves result updates/type/scope; snippets claim obligations only.
Derived ownership: specification chapters own normative rules; roadmap owns
implementation sequencing; coverage audit owns deferral, with credit unchanged.
Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, source/document consistency, full-gate quality assessment.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; changed local links, EN/JA and protected-scope checks.
Exit: acceptance and reviews resolved; task-only commit and resume checkpoint.
