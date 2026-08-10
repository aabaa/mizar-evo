# Task PARSER-TEMPLATE-TYPEHEAD-277P1: Template Type-Head Preservation Prerequisite

> Canonical language: English. Japanese companion: [../ja/PARSER-TEMPLATE-TYPEHEAD-277P1.md](../ja/PARSER-TEMPLATE-TYPEHEAD-277P1.md).

Owning plans: [mizar-parser](../../mizar-parser/en/00.crate_plan.md) and
[mizar-frontend](../../mizar-frontend/en/00.crate_plan.md).

Stable owner sections:

- parser [grammar](../../mizar-parser/en/grammar.md#parser-template-typehead-277p1-required-template-type-head), [source/spec audit](../../mizar-parser/en/source_spec_audit.md#parser-template-typehead-277p1-audit-freeze), [TODO](../../mizar-parser/en/todo.md#independently-authorized-template-type-head-prerequisite), [bilingual audit](../../mizar-parser/en/bilingual_documentation_synchronization.md#parser-template-typehead-277p1-pair-freeze), and [exit qualification](../../mizar-parser/en/crate_exit_report.md#post-closeout-qualification-parser-template-typehead-277p1);
- frontend [parsing](../../mizar-frontend/en/parsing.md#parser-template-typehead-277p1-parser-version-freeze), [cache key](../../mizar-frontend/en/cache_key.md#parser-template-typehead-277p1-cache-assessment), [orchestration](../../mizar-frontend/en/orchestration.md#parser-template-typehead-277p1-replay-contract), [source/spec audit](../../mizar-frontend/en/source_spec_correspondence.md#parser-template-typehead-277p1-follow-through-audit), [TODO](../../mizar-frontend/en/todo.md#independently-authorized-template-type-head-follow-through), [bilingual audit](../../mizar-frontend/en/bilingual_documentation_synchronization.md#parser-template-typehead-277p1-pair-freeze), and [exit qualification](../../mizar-frontend/en/crate_exit_report.md#post-closeout-qualification-parser-template-typehead-277p1).

## Status, authority, and readiness

| Field | Frozen value |
|---|---|
| Status | Documentation frozen; implementation pending in a separate commit. |
| Selection checkpoint | `HEAD=2a6bf6ea`, `origin/main...HEAD=0/2`, and protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4` were observed during the read-only selection inventory. This is historical selection evidence, not implementation evidence. |
| Authority | `doc/spec/en/18.templates.md` §§18.2.2, 18.2.6, 18.10.2 and `doc/spec/en/13.term_expression.md` §13.4 and §13.4.2. |
| Classification | `source_drift` plus Rust `test_gap`; no `spec_gap`. |
| Upper consumer | Checker Task 277B remains blocked after this work by its separate resolver declaration/use-identity prerequisite. This task does not make 277B ready. |

The immutable semantic seed is
`tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.miz`: 701
bytes, final LF, SHA-256
`32c4a1c1b6c9d98dcb085558a084929e07d4005bf92595865f144456e95854ec`.
Its 839-byte sidecar has SHA-256
`b47ac5113c89cd5703adb0ffd660b52d3e16908c92623dd2f63196aa6a215cb2`.
Both stay byte-identical and inactive at `advanced_semantics`.

## Frozen behavior and boundaries

Inside exactly one template-shaped `definition` block, the parser must preserve
a bare one-token `Identifier` as a `TypeHead` when a comprehension generator
grammar requires `identifier "is" type_expression`. It uses parse-local
template-definition scope/depth and an explicit required-type policy. Ordinary
strict type parsing is attempted first; only then may the scoped one-token
identifier fallback apply. A `UserSymbol` keeps its existing `QualifiedSymbol`
shape. The parser does not bind a parameter, resolve a name, infer a type,
create semantic identity, or decide sethood.

The fallback is forbidden in global speculative type parsing, template
arguments, bracket type arguments, and `is`-assertion alternatives. It must
not accept a multi-token identifier expression, leak outside the matching
definition block, change an ordinary definition, alter diagnostics outside the
frozen shape, or modify parser public API/diagnostic vocabulary.

The successful seed parse is exactly: root node `56` of `57` nodes, no syntax
diagnostics or recovery; `TypeExpression` and `TypeHead` at `678..679`,
generator at `673..679`, condition at `682..692`, `SetComprehension` at
`663..694`, `FunctorDefinition` at `623..695`, and `DefinitionBlockItem` at
`593..700`. Semantic verification must later reject bare `T` for missing
sethood under §18.10.2.

## Exact implementation and tests

Only these five Rust paths are authorized in the later implementation commit:

1. `crates/mizar-parser/src/grammar.rs`
2. `crates/mizar-parser/src/module.rs`
3. `crates/mizar-parser/src/module/tests.rs`
4. `crates/mizar-frontend/src/parsing.rs`
5. `crates/mizar-frontend/src/orchestration.rs`

Parser tests grow `226 -> 229` with exactly
`parser_parses_template_type_parameter_as_set_comprehension_generator_type`,
`parser_scopes_required_identifier_type_fallback_to_template_definitions`, and
`parser_keeps_template_required_type_fallback_single_token_and_ambiguity_bounded`.
Frontend tests grow `153 -> 154` with
`task277p1_template_typehead_changes_ast_cache_namespace` in
`orchestration.rs`.

Because the old parser returned a recovered AST for the same inputs, frontend
must change `MIZAR_PARSER_CACHE_KEY_VERSION` from v2 to v3 and prove replay
uses v3 while an unchanged control preserves its behavior. This is a cache
namespace change only, not a frontend grammar or semantic decision.

## Frozen inventory and documentation scope

Parser baseline: 12 paths / 39,159 lines, path hash
`192f9d0b5e6534c4daab010ec51a9356e9e0fd6fb86876bd2600a75844e7566a`,
content hash `60a82837ae275862949212d21e3429cbe489763400f0f503dc1fa3daa09d761e`;
`grammar.rs` is 243 lines,
`a4673e82d9866c5499ae919b09189b3902e9d62f293c3e7c9f8b96b1190ef476`;
`module.rs` is 16,761 lines,
`e5464621161b45e6f7744c97b938d7ac82bd78f62870a39e36a45c4eace11b8d`; and
`module/tests.rs` is 18,490 lines,
`bac4e49eaa094708aa6d7bcede96ca304b95c4d41366ef199f4cb59da469c085`.
Its terse raw list hash is
`05194f5916812d24c36130e60275e4ba9933ad9d2ee81cf4026e2379e383dbad`.
Frontend baseline: 9 paths / 11,030 lines, path hash
`c7b20f1ab8f9414109dce18eb6591a6f18c0b413cd5a11beb84fa1b785822101`,
content hash `03a81d83e7af3b12ce21eb028de2920cbf8e4a9f5a873aaad53cdbf1487b7b6e`;
`parsing.rs` is 1,459 lines,
`c26f56bc2a052fd1d2660f0b2f545a48c1f4390da34feba534ebb5d750c17824`;
`orchestration.rs` is 1,908 lines,
`84b49e07a831c87484a725d457e4f6611ed03e422d1417d2d2d71ee2a091166b`;
and its terse raw list hash is
`298dbed058e39cfe83d5381d49fe1d3d014cf3412d148c568f7470d440731d30`.
Contract pairs grow `82/82 -> 83/83`.

This documentation commit changes exactly 30 Markdown paths: this EN/JA
contract pair; paired parser `00.crate_plan`, `grammar`, `source_spec_audit`,
`todo`, `bilingual_documentation_synchronization`, and `crate_exit_report`;
and paired frontend `00.crate_plan`, `parsing`, `cache_key`, `orchestration`,
`source_spec_correspondence`, `todo`, `bilingual_documentation_synchronization`,
and `crate_exit_report`. The later completion-document set is exactly 26 paths:
this pair plus every listed non-plan owner path. With the five Rust paths it is
31 paths.

`doc/spec`, existing `.miz`, expectations, trace metadata, active stages/routes,
coverage status, diagnostic vocabulary/order outside the frozen target,
public API shape other than the explicitly changed public cache-version value,
`doc/design/spec_coverage_audit.md` (baseline
`a31f6fb3bd2b561610630497c58284484d00716dd0b7f210f55bef3bc4bfa6db`),
Cargo files, and all mizar-test/syntax/resolve/checker/Core sources and docs are
protected. The frozen target diagnostic vector changes from recovered syntax
diagnostics to zero exactly as specified above; no diagnostic code or message
vocabulary is added.
Trace hash `55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`
remains unchanged.

## Exit evidence

The implementation must run focused parser and frontend tests, frontend replay
and cache assertions, format, denied-warning Clippy, workspace tests, five
CLIs, frozen count/hash remeasurement, recursive contract/link lint, and
`git diff --check`. Independent specification/test, implementation, and
source/documentation reviews must end with no findings before the separate
implementation commit.

## Next-task handoff

Implement `PARSER-TEMPLATE-TYPEHEAD-277P1` in exactly the five frozen Rust
paths, add exactly the three parser tests and one frontend replay/cache test,
keep the semantic seed, expectation, trace, coverage, diagnostics vocabulary,
and resolver/checker behavior unchanged, then synchronize exactly the 26
completion documents and run every frozen review and verification gate. Keep
the parent on GPT-5.6 Sol `xhigh` for authority, integration, staging, and final
scoring. GPT-5.6 Luna is not exposed at this checkpoint; use GPT-5.6 Terra
`high` for each independently bounded parser/frontend implementation or review
assignment, escalating any scope or authority ambiguity to the Sol `xhigh`
parent.
