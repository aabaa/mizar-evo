# Task CHECKER278-OVERLOAD-SELECTION: source overload selection
Canonical language: English; [Japanese pointer](../ja/CHECKER278-OVERLOAD-SELECTION.md).
Status: implemented for unary builtin and direct type-template inputs. Tier: full (new source owner and public producer seam).
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: [runner plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: Tasks249–257,259–264,277 direct roles; prepared MT10-AS. Task279 consumes these roots afterwards.
Authority: [Task278 and acyclic graph](../../mizar-checker/en/payload_family_decomposition.md#accepted-producer-graph), [queue](../../mizar-checker/en/todo.md), Specs [08](../../../spec/en/08.type_inference.md), [18](../../../spec/en/18.templates.md), [19.1–19.4/19.6](../../../spec/en/19.overload_resolution.md#191-overview); MC-G027.
## Scope and representation
Close source_drift/test_gap between source overload sites/candidate payloads and the existing selection engine.
New source_overload owner authenticates original source/module, typed sites, anchors, ordinary/template candidate identities and written order.
Reuse source extraction and Task277 direct roles; do not duplicate term, formula or template representations.
Carry explicit signatures/results, template actual/constraint inputs, viability evidence, pair comparisons and inserted-view inputs into existing collection, expansion, viability, specificity and ordinary-root selection APIs.
Comparison reasons and evidence remain explicit upstream inputs; absence cannot imply ordering or a winner. Result types remain payload only.
Reject missing/foreign/recovered anchors, missing payloads, duplicate/conflicting identities, reordered candidates and redefinition/refinement inputs transactionally.
Explicit source views retain their written basis; no hidden qua, widening path, inference or accepted evidence is invented.
Install the immutable handoff through TypedAst and ResolvedTypedAst; final assembly preserves and consumes matching pipeline outputs.
MT10-AS executes the producer and observes final roots on a real source, with deterministic rerun and refusal checks.
No selection-semantic, doc/spec, existing expectation, trace-status, diagnostic, proof/acceptance or semantic-credit change.
Return-type tie-breaks, Task279 refinement, Resolver31 same-return conflicts, and scheme/theorem roles (Gate S1) remain excluded.
## Owners and tests
Selection API/invariants: [overload engine](../../mizar-checker/en/overload_resolution.md#inputs-and-outputs); provenance: [direct template roles](../../mizar-checker/en/source_template.md).
Artifacts: source_overload.rs, typed_ast.rs, resolved_typed_ast.rs, lib.rs; runner.rs, focused regressions, [owner boundary](../../mizar-checker/en/source_overload.md) and [family ledger](../../../../tests/coverage/overload_selection.tsv).
Existing native overload/template extraction tests stay unchanged; no Task49 reconciliation fixture is activated.
New smoke: tests/miz/pass/overload/pass_advanced_semantics_ordinary_root_001.miz with advanced_semantics sidecar.
Frozen spelling under Specs [10.1](../../../spec/en/10.functors.md#101-overview-and-syntax), [13.2](../../../spec/en/13.term_expression.md#132-functor-applications), [14](../../../spec/en/14.formulas.md):
`definition let x be set; func OvRoot: ovroot(x) -> set equals x; coherence; end; reserve x for set; theorem OvUse: ovroot(x) = x;`
One local identity root, set argument/result and reserve, one application in an equality; no proof acceptance.
Coverage audit and semantic deferrals remain unchanged: producer/consumer execution only.
## Frozen test matrix
Cap: 24 categories; exactly these 11 dispatched categories.
1. Ordinary functor site, multiple candidates: all pipeline stages yield one root; explicit comparisons and source anchors survive.
2. Template site using delivered Task277 roles: same pipeline yields a template root.
3. Single candidate: select its root with intact payload.
4. Explicit inserted-view site: view-adjusted result through the existing inserted-view API.
5. Contested site: explicit specificity/comparison evidence selects deterministically; preserve source order.
6. Missing candidate payload or missing/foreign/recovered/cross-source site or anchor: refuse.
7. Contested site without comparison evidence: refuse or blocked, never fabricate a winner.
8. Selection requiring return-type tie-break: explicit refusal, no winner.
9. Inserted view without explicit source basis (hidden qua): refuse.
10. Redefinition-refinement candidate: refuse; Task279 owns it.
11. Duplicate/conflicting identity or out-of-source-order candidates: refuse.
## Review, verification and exit
Independent specification/test-intent review precedes freeze; sufficiency, precision, volume and source/document consistency follow.
Run fmt, workspace all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, overload/template and Typed/final owners, focused Task10 tests and short advanced-semantics CLI.
Heavy evidence/cache/resolver obligations remain unchanged; full cargo test follows protocol cadence.
Exit requires the matrix, transactional final consumer, nine hard gates and score at least90.
