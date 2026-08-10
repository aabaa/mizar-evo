# Task CHECKER-FRAENKEL-TEMPLATE-STRUCTURAL-277C: Fraenkel Template Structural Composition

> Canonical language: English. Japanese companion: [../ja/CHECKER-FRAENKEL-TEMPLATE-STRUCTURAL-277C.md](../ja/CHECKER-FRAENKEL-TEMPLATE-STRUCTURAL-277C.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

Stable owner sections: checker [module API](../../mizar-checker/en/source_template_type_parameter_association.md#task-277c-frozen-planned-public-extension), [source/spec mapping](../../mizar-checker/en/source_spec_audit.md#task-277c-frozen-sourcespecification-mapping), [boundary](../../mizar-checker/en/module_boundary_audit.md#task-277c-frozen-module-boundary), [TODO](../../mizar-checker/en/todo.md#task-277c-fraenkel-structural-composition), and [bilingual record](../../mizar-checker/en/bilingual_sync_audit.md#task-277c-frozen-contract-parity); mizar-test [harness](../../mizar-test/en/harness.md#checker-task-277c-private-structural-composition-probe), [boundary](../../mizar-test/en/module_boundary_audit.md#checker-task-277c-frozen-module-boundary), [TODO](../../mizar-test/en/todo.md#checker-task-277c-private-structural-composition-probe), and [bilingual audit](../../mizar-test/en/bilingual_sync_audit.md#checker-task-277c-frozen-contract-parity).

## Status and authority

**Status:** implementation, broad verification, and all independent reviews are
complete. The final-quality review reports **NO FINDINGS**; all nine hard gates
pass without a score cap at valid `100/100` (`20/20/15/15/10/10/5/5`). Only
task-closeout evidence remains pending. This contract is canonical; its
synchronized companion is a translation, not competing authority.

This task is a neutral checker-only structural composition after completed
`RESOLVE-TEMPLATE-TYPEPARAM-277R1`, `277B-L`, and
`RESOLVE-FRAENKEL-GENERATOR-VAR-277R2`. It does **not** make Task 277B ready,
award semantic coverage credit, or change the specification, fixtures,
expectations, traceability, diagnostics, proof language, or source behavior.
The authority remains, in order, `doc/spec/en/13.term_expression.md`
§§13.4.2, 13.4.4, and 13.8.6; `doc/spec/en/18.templates.md` §§18.2.1,
18.2.2, 18.2.6, and 18.10.2; the immutable F5 `.miz` fixture, expectation,
and trace entry; then derived design and source records. Existing authorities
are read-only for this task.

The completed implementation closes the recorded `design_drift`, `source_drift`,
and Rust `test_gap` for this structural handoff. There is no `spec_gap`; no test
intent is newly derived. All language, type, proof, diagnostic, source-producer,
and production-route decisions remain deferred to a separately authorized task.

## Frozen boundary and public ABI

The completed implementation is one neutral standalone composition in the existing
`crates/mizar-checker/src/source_template_type_parameter_association.rs`
module. It consumes exactly:

- `&SourceTemplateTypeParameterAssociationHandoff`;
- `&FraenkelGeneratorVariableSourceCollection`; and
- `&TypedAst`.

It takes no R1 direct input and must not add a TypedAst/ResolvedAst slot,
install step, facade, production runner route, source-owner change, semantic
interpretation, diagnostic, trace/coverage credit, `lib.rs` update, or lint
Rust change.

The public family is exactly:

```rust
SourceTemplateFraenkelStructuralCompositionId
SourceTemplateFraenkelStructuralComposition
SourceTemplateFraenkelStructuralCompositionTable
SourceTemplateFraenkelStructuralCompositionHandoff
#[non_exhaustive] SourceTemplateFraenkelStructuralCompositionError
SourceTemplateFraenkelStructuralCompositionProducer
```

`SourceTemplateFraenkelStructuralCompositionProducer::build` has the exact
shape `build(template, generators, typed_ast) -> Result<
SourceTemplateFraenkelStructuralCompositionHandoff,
SourceTemplateFraenkelStructuralCompositionError>`, with the three inputs in
the order shown above. Its errors are checked and exposed in this order:

```rust
EnvironmentMismatch
InvalidTemplateAssociation { association }
InvalidGeneratorBinding { binding }
InvalidGeneratorUse { use_index }
InvalidComposition { composition: SourceTemplateFraenkelStructuralCompositionId }
UnmatchedTemplateAssociation { association: SourceTemplateTypeParameterAssociationId }
```

`InvalidComposition` deliberately carries a composition ID, rather than an
association ID, so a rejected orphan generator binding remains representable.
`UnmatchedTemplateAssociation` reports the lowest dense association ID left
unconsumed after every R2 binding candidate. No caller may infer row identity
from spelling, source range, equal table position, or a cast.

An ID has only `new` and `index`. A row has these exact immutable getters:

- `template_association() -> SourceTemplateTypeParameterAssociationId`;
- `template_binding() -> TemplateTypeParameterBindingId`;
- `generator_binding() -> FraenkelGeneratorVariableBindingId`;
- `definition_block`, `parameter`, `template_binder`, `type_head`,
  `template_identifier`, `functor_definition`, `comprehension`, `segment`,
  `generator_binder`, `type_expression`, `mapper_role_owner`,
  `mapper_term_reference`, `mapper_identifier`, `first_condition_role_owner`,
  `first_condition_term_reference`, `first_condition_identifier`,
  `second_condition_role_owner`, `second_condition_term_reference`, and
  `second_condition_identifier`, each returning `TypedNodeId`;
- `mapper_source_ordinal`, `mapper_role_source_ordinal`,
  `first_condition_source_ordinal`, `first_condition_role_source_ordinal`,
  `second_condition_source_ordinal`, and `second_condition_role_source_ordinal`,
  each returning `usize`.

The table exposes `get`, `iter`, `len`, and `is_empty`. The handoff exposes
`source_id`, `module_id`, `compositions`, and `debug_text`. For F5, exact
debug output is
`source-template-fraenkel-structural-composition-v1|module=<module>|compositions=1|uses=3`.

## Structural validation and F5 oracle

Validation is deterministic and fail-closed: first revalidate a common
source/module environment, then the template association, then the generator
binding, then each generator use, then the completed composition. The producer
rescans resolved-to-typed associations uniquely; it does not trust precomputed
IDs. It requires normal recovery, exact node kinds, exact range
anchors/equality/containment, exact direct AST edges, and exact resolver
provenance. For each R2 binding in source order, the producer first maps the
binding's definition block and segment uniquely into `TypedAst`, follows the
single direct normal `segment -> TypeExpression -> TypeHead` chain, and then
matches the unique template association whose typed `definition_block()` and
`type_head()` equal those two reached typed nodes. It never zips equal-length
tables. Zero or multiple matches, or reuse of one association by another R2
binding, returns `InvalidComposition` for that R2 candidate. After all R2
bindings, the lowest unconsumed association returns
`UnmatchedTemplateAssociation`. Empty/empty input is valid; every other orphan
or multiple relation is rejected atomically. Candidate compositions have
deterministic dense IDs in R2 generator-binding source order. Template
association IDs and R2 binding/use IDs and ordinals remain unchanged; the
producer performs no spelling/range inference or reordering and returns no
partial handoff after an error.

For F5 the sole row uses association `0`, template binding `0`, and generator
binding `0`. Its TypedNodeId getters return respectively:

| Getter sequence | Exact values |
|---|---|
| definition, parameter, template binder, type head, template identifier | 53, 31, 2, 39, 21 |
| functor definition, comprehension, segment, generator binder, type expression | 52, 49, 41, 19, 40 |
| mapper owner, reference, identifier | 38, 37, 17 |
| first condition owner, reference, identifier | 48, 42, 24 |
| second condition owner, reference, identifier | 48, 44, 26 |
| mapper source/role, first-condition source/role, second-condition source/role ordinals | 0/0, 1/0, 2/1 |

The exact direct-edge chains are definition block → template parameter and
functor definition; template parameter → template binder; functor definition
→ `TermDefiniens#51` → `TermExpression#50` → comprehension;
comprehension → mapper owner, segment, and condition owner; segment →
generator binder and type expression; type expression → type head →
template identifier; mapper owner → term reference → identifier; and
condition owner `FormulaExpression#48` → `PrefixFormula#47` →
`BuiltinPredicateApplication#46` → the respective `TermExpression#43/#45`
→ term reference `#42/#44` → identifier `#24/#26`. Edge kind, range,
recovery, and resolver provenance must be checked exactly, not reconstructed
from text.

## Frozen implementation and test scope

The completed implementation changed exactly three Rust paths:

1. `crates/mizar-checker/src/source_template_type_parameter_association.rs`;
2. `crates/mizar-test/src/runner/tests.rs`; and
3. new private `crates/mizar-test/src/runner/tests/type_elaboration/template_fraenkel_structural_composition.rs`.

The test matrix is exactly four public-facing test functions plus one private
fixture leaf:

1. `task277c_composes_exact_template_fraenkel_structural_handoff`;
2. `task277c_rejects_environment_missing_and_ambiguous_resolved_nodes`;
3. `task277c_rejects_recovery_kind_range_edge_and_provenance_corruption`;
4. `task277c_rebuilds_deterministically_without_mutating_typed_ast` (also
   empty/empty, orphan R2 binding, zero/multiple structural matches, reused
   association, and association-side orphan cases); and
5. `task277c_real_fixture_builds_exact_template_fraenkel_structural_composition`.

The test registrations changed exactly from checker raw list `542 -> 546` and
mizar-test raw list `611 -> 612`. No fixture, sidecar, expectation, trace,
coverage, Cargo, production runner, or lint-policy source changed.

## Baseline and protected evidence

The checker production inventory is 32 regular paths / 189180 lines, path
SHA-256 `9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787`,
and content SHA-256 `560c15585dd85de320c42c15668657cf3d03a967dfe677ea03be33a0ae905861`.
The narrower Rust-only subinventory is 30 paths / 189124 lines. The owner
module is 1224 lines, SHA-256
`7ff46174cf7818722ea8acf6a2a55be77659ce821d68c531b583134ac12f8018`; the
mizar-test registration is 64 lines, SHA-256
`8ae81a6ca4dadd9a58165f09bdde4d2ad3cdcd0884ad7521fe5d1ea90539b316`.
The protected checker lint policy is 1955 lines, SHA-256
`f8c0c2c196e476b744716d51d8252a61f667536ef97a441246519b3b1a6dd2a0`;
the completed 277B-L and 277R2 private leaves are respectively 249 / 106
lines, SHA-256 `5fb342d357fb8cb92bd88278c019b276741cd1d6edb255e16e4f231f578dfe04`
and `69b54a4effcb7a740d6588070e6951e3a772cd1818ef9fedcb36426642bf3bf4`.
The mizar-test production inventory remains 38 paths / 80090 lines, path
SHA-256 `0ef395004f7feaadf60da0daba7b5da9c52ea4974850adfa2bd9d09081b242aa`
and content SHA-256 `990b5ad4798786d9f87c03f76fdbad92fc2edf1f6d84ef3baad67254c79fdd70`.

Protected authorities remain: 64 English specs
(`d900ba9e43ab094925f36493f830e0c6a2964be2911d5d229014a58842067a25` /
`b30dd5519191a4407399826faf91cc58853d7944df7630734b5ab05de48c9f7b` path/content),
343 `.miz` files
(`d94980e167b4b8ac196f91e7694ff044080c6fb4d04c135b3cd5e65b9a019f22` /
`54e6ea1254a0bd963c39026d788711507f353e9c6df3b4f9fd268b2e9f240afb`),
435 expectations
(`22a5ee257a294e3f2ed4b24bf9ca243d037bbd798f009d0d1e3a176dad8b4fea` /
`b5f0ed1a8d73bbfb78af5cad87a8d426e97269f7f70163750893a9fe1f39d2ea`),
and 21 Cargo files
(`d93f2816b760d8ba45430d6d8570e598864aa7b20b19a001f45171d36fd3a030` /
`146e9b4024d2c344b2eca9c6f5ca6d6a80a3de427e382953a2280bc63cb3ecca`).
F5 fixture/expectation/trace hashes are respectively
`32c4a1c1b6c9d98dcb085558a084929e07d4005bf92595865f144456e95854ec`,
`b47ac5113c89cd5703adb0ffd660b52d3e16908c92623dd2f63196aa6a215cb2`, and
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.
The coverage audit remains immutable at SHA-256
`a31f6fb3bd2b561610630497c58284484d00716dd0b7f210f55bef3bc4bfa6db`.
Raw checker tests are 542, SHA-256
`e2b0e67d6066c7157b491e4c57c1f61200dc9339d0b03592af13b551ebfa4410`;
raw mizar-test tests are 611, SHA-256
`6eaaca04215420028c57731bc14144e2b73ca719dc6cc35f64a5a421e2a1c426`.

The five frozen CLI hashes are plan
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
parse-only `a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
declaration-symbol `71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
type-elaboration `4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and proof-verification `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

## Completion evidence

The measured checker production manifest is 32 regular paths / 191068 lines,
with the protected path SHA-256 unchanged at
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787` and
content-manifest SHA-256
`cf4e43bb5671f863d9af36f99592ca188bab28b2480acb886e1171d65f57fe8a`.
The mizar-test production manifest remains 38 regular paths / 80090 lines,
with path/content SHA-256
`0ef395004f7feaadf60da0daba7b5da9c52ea4974850adfa2bd9d09081b242aa` /
`990b5ad4798786d9f87c03f76fdbad92fc2edf1f6d84ef3baad67254c79fdd70`.

The exact changed Rust paths measure:

| Path | Lines | SHA-256 |
|---|---:|---|
| `crates/mizar-checker/src/source_template_type_parameter_association.rs` | 3112 | `0ff5b20f8c9a420149af232947ddd4f09924d31631aea22eabdc24d2daa91145` |
| `crates/mizar-test/src/runner/tests.rs` | 65 | `6d07a5ba5efe0be8f058eb52028e90c0bbb279b5d088604c55e1a9d1ca5e75ba` |
| `crates/mizar-test/src/runner/tests/type_elaboration/template_fraenkel_structural_composition.rs` | 134 | `64dd80f1d4501c3ab5735a215fb0301bec6d85ca67258aebc195cc898be31d44` |

`cargo test -q -p mizar-checker --lib -- --list` reports 546 entries with
SHA-256 `2477c548993fcbfffa817814f462ab5d7ce1549a083b6d65aa87091f08bbc9ed`.
`cargo test -q -p mizar-test --lib -- --list` reports 612 entries with SHA-256
`5a8c1170208533ed4d1723acd05a07ab9f62569b47507129d56c14f7fc2af65a`.

`cargo fmt --check`, focused 277C tests, both package library suites, both
package library Clippy checks, and `git diff --check` passed. Independent
test-sufficiency and implementation reviews report **NO FINDINGS**. The five
post-implementation CLI replays also passed with their frozen hashes and 23
warnings / 0 errors. Parent-owned broad verification also passed: `cargo fmt
--check`, `cargo clippy --all-targets --all-features -- -D warnings`, full
`cargo test`, mizar-test metadata 137/137, the five CLI replays, and the frozen
counts/hashes. The final source/documentation re-review and independent
bilingual/boundary review report **NO FINDINGS**. The independent final-quality
review also reports **NO FINDINGS**; all nine hard gates pass uncapped at valid
`100/100` (`20/20/15/15/10/10/5/5`). No task-only staging, commit, post-commit
proof, or fresh successor inventory is claimed.

## Documentation, gates, and handoff

The completed documentation surface changes exactly 20 Markdown paths: this
EN/JA contract pair and the paired checker and mizar-test owner records. The
four plan rows were prerequisite-only index changes and remain untouched.
`spec_coverage_audit.md` has no impact and remains unchanged; neither the
legacy-compaction ledger nor its policy source changed.

The implementation-specific authority/scope, dependency, ABI, structural,
test-sufficiency, implementation-review, CLI-replay, broad-workspace
verification, source/documentation, bilingual, and boundary evidence is
complete; the three independent reviews report **NO FINDINGS**. Final
independent quality review reports **NO FINDINGS**, all nine hard gates pass,
and the valid uncapped score is `100/100` (`20/20/15/15/10/10/5/5`). The
remaining parent-owned actions are exact staging/cached review, task-only
commit, post-commit proof, and fresh successor inventory. Sol xhigh owns
authority, integration, final reviews, staging, and commit; Terra high is the
bounded inventory/review route; Luna is unavailable and the effective routing
must be recorded. No `doc/design/spec_coverage_audit.md` or
legacy-compaction/ledger delta is authorized.

**Next handoff:** perform the exact staging/cached review, task-only commit,
post-commit proof, and fresh successor inventory. Keep the scope to the three
implemented Rust paths and these 20 completion records; stop for any authority
contradiction, public dependency issue, scope expansion, or failed protected
replay before staging a task-only commit.
