# Task AST-MULTI-BINDER-SPEC: Multiple-argument Binder specification
Canonical language: English; [Japanese pointer](../ja/AST-MULTI-BINDER-SPEC.md).
Status: specification amendment. Tier: full gates.
Primary owner: [mizar-checker plan](../../mizar-checker/en/00.crate_plan.md).
Consumers: mizar-parser, mizar-core, mizar-test and reflection consumers.
Authority: user-approved product-domain Binder proposal; canonical
[§13.10](../../../spec/en/13.term_expression.md#1310-function-binder--),
[§20.5.3](../../../spec/en/20.algorithm_and_verification.md#2053-reflected-syntax)
and [Appendix A](../../../spec/en/appendix_a.grammar_summary.md#a13-term-expressions).
Scope: paired EN/JA syntax, ordered finite arity, left-nested products,
formation, simultaneous binding, capture/reconstruction, structure/meaning,
and a separate corpus acceptance test plan; synchronize the lexical reservation and roadmap.
Gap: spec_gap for multiple-argument Binder; existing reflection implementation
and executable corpus remain source_drift/test_gap under AST-PARSE/TYPE/LOWER/CORPUS.
Readiness: specification work is authorized; execution waits for authentic
syntax/type producers, pinned library certification and per-use proofs.
Test plan: the [Binder acceptance matrix](../../mizar-test/en/miz_corpus.md#multiple-argument-binder-acceptance-tests) owns test items; existing .miz files,
expectations and trace activation stay unchanged. New executable cases belong
to the stage-specific owners after this specification is frozen.
Owner boundaries: [parser](../../mizar-parser/en/grammar.md#reflected-syntax-input-boundary),
[typing](../../mizar-checker/en/typed_ast.md#reflected-syntax-formation),
[corpus](../../mizar-test/en/harness.md#reflection-corpus-boundary).
Excluded: Rust implementation, new public diagnostics, dependent heads,
arbitrary subset-domain multi-head construction, implicit currying,
partial-differentiation API, runtime or proof acceptance claims.
Audit: record extended deferred scope without increasing coverage credit.
Reviews: specification/parity, acceptance sufficiency, final document
consistency, independent volume/scope; no implementation-stage acceptance.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; local links, grammar parity and git diff --check.
Exit: language specifications own semantics; paired corpus design owns test items.
Implementation/certification deferrals remain explicit and existing tests fixed.
