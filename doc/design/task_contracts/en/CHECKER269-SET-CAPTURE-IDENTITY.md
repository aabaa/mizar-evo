# Task CHECKER269-SET-CAPTURE-IDENTITY: direct definition-site captures
Canonical language: English; [Japanese pointer](../ja/CHECKER269-SET-CAPTURE-IDENTITY.md).
Status: complete. Tier: full. Bounded Task 269 increment; Task 269 remains open.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: [test plan](../../mizar-test/en/00.crate_plan.md).
Dependency/readiness: completed CHECKER269-SET-BINDINGS and its sealed RHS authority.
Authority: EN04 §4.4.1 and EN15 §15.2.2 require definition-time resolved-binding captures.
Gap: `source_drift`: authenticated Set RHS references exist but captures remain empty.
The [proof-local owner](../../mizar-checker/en/source_proof_local_declaration.md#descendant-set-name-binding-boundary) owns build/replay; use existing BindingDraft.captured.
Scope: reconstruct direct captured BinderIdentity values from validated RHS reference targets.
Preserve genuine identities, declaration clocks and lexical scopes through existing Typed/Resolved paths.
The old Given-only build remains unchanged; no new type, field, API, or runner route.
No closure expansion, transitive capture flattening, type/fact/proof production or FOL choice.
EN04 §4.7.3 encoding conflict remains outside this metadata scope; no spec/fixture/trace/credit edits.
Existing new-path Rust assertions intentionally expected empty captures for the previous names-only boundary;
replace only those expectations with exact definition-site identities, preserving old-path checks.
Add coherent captured-identity/fingerprint corruption refusal and replay/layout controls.
Reviews: independent authority/test intent, test sufficiency, implementation, volume and consistency.
Verify fmt, Clippy, checker owner/type/term, runner269, type/formula corpus, docs and Driver; cadence governs full/heavy.
Exit: both direct captures persist and replay exactly; unchanged semantic effects/old routes; audit credit unchanged.
