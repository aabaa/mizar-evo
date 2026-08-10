# Task RESOLVE-FRAENKEL-GENERATOR-VAR-277R2: Fraenkel Generator-Variable Identity Prerequisite

> Canonical language: English. Japanese companion: [../ja/RESOLVE-FRAENKEL-GENERATOR-VAR-277R2.md](../ja/RESOLVE-FRAENKEL-GENERATOR-VAR-277R2.md).

Owning plans: [mizar-resolve](../../mizar-resolve/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

Stable owner sections:

- resolver [names](../../mizar-resolve/en/names.md#resolver-task-277r2-fraenkel-generator-variable-identity), [source/spec correspondence](../../mizar-resolve/en/source_spec_correspondence.md#resolver-task-277r2-sourcespecification-correspondence), [boundary](../../mizar-resolve/en/module_boundary_refactor.md#resolver-task-277r2-module-boundary), [TODO](../../mizar-resolve/en/todo.md#resolver-task-277r2-frozen-documentation-prerequisite), [bilingual record](../../mizar-resolve/en/bilingual_documentation_synchronization.md#resolver-task-277r2-bilingual-synchronization), and [exit addendum](../../mizar-resolve/en/crate_exit_report.md#resolver-task-277r2-post-exit-prerequisite);
- test [harness](../../mizar-test/en/harness.md#resolver-task-277r2-test-only-fixture-probe), [module boundary](../../mizar-test/en/module_boundary_audit.md#resolver-task-277r2-test-module-boundary), [TODO](../../mizar-test/en/todo.md#resolver-task-277r2-test-only-fixture-probe), and [bilingual audit](../../mizar-test/en/bilingual_sync_audit.md#resolver-task-277r2-contract-parity).

## Status, authority, and classification

| Field | Frozen value |
|---|---|
| Status | The exact five-path Rust implementation is complete. The exact `4 + 1` regressions, resolver and mizar-test library and lint suites, package and workspace Clippy, workspace tests, offline metadata and its suite, formatting, diff, five frozen CLI hashes, and protected path-hash checks are complete. Independent test-sufficiency, implementation, source/documentation/API, bilingual, boundary, and final-quality reviews report **NO FINDINGS**. All nine hard gates pass with no score cap at valid `100/100` (`20/20/15/15/10/10/5/5`). Exact staging/cached-diff review, task-only implementation commit, post-commit proof, and fresh inventory are complete in the historical pre-closure checkpoint below. No successor is selected; Task 277B remains not ready with zero semantic credit. |
| Authority | `doc/spec/en/13.term_expression.md` §§13.4.2, 13.4.4, and 13.8.6; `doc/spec/en/18.templates.md` §18.10.2; the immutable F5 source, expectation, and trace row. |
| Dependencies | Completed [277R1](./RESOLVE-TEMPLATE-TYPEPARAM-277R1.md) and [277B-L](./277B-L.md) are read-only context. This task creates an independent resolver-owned generator-variable relation and neither extends nor consumes their IDs. |
| Classification | The classified `source_drift` and Rust `test_gap` are resolved by the bounded implementation and exact regressions. Completion documentation reconciles the corresponding `design_drift`; no `spec_gap` exists. The later sethood decision and missing-sethood verdict remain checker-owned. |
| Consumer | A separately frozen lower transport or checker task may consume this structural collection. This task does not select that consumer or make Task 277B ready. |

The immutable semantic seed is
`tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.miz`, 701
bytes with final LF, SHA-256
`32c4a1c1b6c9d98dcb085558a084929e07d4005bf92595865f144456e95854ec`.
Its 839-byte expectation remains SHA-256
`b47ac5113c89cd5703adb0ffd660b52d3e16908c92623dd2f63196aa6a215cb2`,
inactive at `advanced_semantics`. Its mapped trace seed remains inactive while
checker-plan gaps `MC-G020` and `MC-G021` remain deferred. No source,
expectation, trace, stage, or coverage-credit change is authorized.

## Frozen real-source profile

The parser profile is 57 normal nodes with root `56` and no diagnostics:

| Role | Surface node | Range / ordinal |
|---|---:|---:|
| enclosing declaration | `DefinitionBlockItem#53` | `593..700` |
| enclosing functor | `FunctorDefinition#52` | `623..695` |
| comprehension | `SetComprehension#49` | `663..694` |
| generator segment / binder | `ComprehensionVariableSegment#41` / `Identifier#19`, text `x` | `673..679` / `673..674`; binding ordinal `0` |
| mapper role owner / reference / identifier | `TermExpression#38` / `TermReference#37` / `Identifier#17` | identifier `665..666`; global use `0`, role use `0` |
| condition role owner / first reference / identifier | `FormulaExpression#48` / `TermReference#42` / `Identifier#24` | identifier `686..687`; global use `1`, role use `0` |
| condition second reference / identifier | `TermReference#44` / `Identifier#26` | identifier `691..692`; global use `2`, role use `1` |

The binder scopes both the mapper occurrence that precedes its declaration in
source order and the later condition occurrences. Identity is the sole bounded
same-spelling structural match inside this exact comprehension; it is not a
general lexical, template, alias, or shadow resolver.

## Frozen resolver API and validation

Only `crates/mizar-resolve/src/names.rs` may add this public surface:

- `FraenkelGeneratorVariableBindingId` with `new(index: usize) -> Self` and
  `index(self) -> usize`;
- immutable `FraenkelGeneratorVariableBinding`;
- non-exhaustive `FraenkelGeneratorVariableUseRole::{Mapper, Condition}`;
- immutable `FraenkelGeneratorVariableUseLink`;
- `FraenkelGeneratorVariableBindingTable` and
  `FraenkelGeneratorVariableUseLinkTable`;
- `FraenkelGeneratorVariableSourceCollection`; and
- `FraenkelGeneratorVariableSourceCollector`.

Binding getters are `definition_block()`, `functor_definition()`,
`comprehension()`, `segment()`, and `binder()`, each returning
`ResolvedNodeId`; `spelling() -> &str`; `segment_range()` and
`binder_range()`, each returning `SourceRange`; and `source_ordinal() ->
usize`. Use-link getters are `definition_block()`, `functor_definition()`,
`comprehension()`, `role_owner()`, `term_reference()`, and `identifier()`,
each returning `ResolvedNodeId`; `binding() ->
FraenkelGeneratorVariableBindingId`; `role() ->
FraenkelGeneratorVariableUseRole`; `source_ordinal() -> usize`;
`role_source_ordinal() -> usize`; and `identifier_range() -> SourceRange`.

The binding table exposes `get(id) ->
Option<&FraenkelGeneratorVariableBinding>`, `iter() -> impl Iterator<Item =
(FraenkelGeneratorVariableBindingId, &FraenkelGeneratorVariableBinding)>`,
`len() -> usize`, and `is_empty() -> bool`. The use-link table has no separate
ID and exposes `get(index: usize) -> Option<&FraenkelGeneratorVariableUseLink>`,
row `iter`, `len`, and `is_empty`. Both tables are dense and deterministic.
The collection exposes `source_id() -> SourceId`, `module() -> &ModuleId`,
`bindings() -> &FraenkelGeneratorVariableBindingTable`, `uses() ->
&FraenkelGeneratorVariableUseLinkTable`, and `debug_text() -> String`. The F5
summary is exactly
`fraenkel-generator-variable-source-v1|module=<module>|bindings=1|uses=3`,
where `<module>` uses the existing package-and-path rendering.

The exact collector signatures are
`FraenkelGeneratorVariableSourceCollector::new(&SurfaceAst, &ModuleId,
&SurfaceResolvedArena) -> Result<Self, SurfaceResolvedArenaError>` and
`collect(&self) -> Result<FraenkelGeneratorVariableSourceCollection,
SurfaceResolvedArenaError>`. The complete arena is validated in both `new` and
`collect`, and resolver-owned identity comes only from
`SurfaceResolvedArena::resolved_node_for`. No custom error or diagnostic is
introduced.

Collection is default-deny. It admits only the exact normal single-generator,
single-binder Fraenkel shape, assigns the binder to mapper and condition
identifier-term references by sole bounded exact spelling, sorts bindings and
uses by source range plus node identity, assigns dense global ordinals, and
assigns dense role-local ordinals independently for mapper and condition.
Recovery anywhere in the candidate shape, non-exact wrappers or edges,
multiple generators or binders, nested comprehensions, nested binders,
shadowing, ambiguous same-spelling candidates, and every unsupported shape
produce zero binding and use rows. No partial row survives a rejected
candidate.

The task does not construct resolver IDs, allocate `SymbolId`, resolve
`NameRef`, alter `ResolvedAst`, or publish types, `SourceSetTerm`,
`SourceFormula`, another task's `BindingId`, sethood, evidence, diagnostics,
or a verdict. In particular it carries no template parameter, R1, or 277B-L ID.

## Implementation, tests, and measured inventory

The implementation changes exactly five Rust paths:

1. `crates/mizar-resolve/src/names.rs`
2. `crates/mizar-resolve/src/names/tests.rs`
3. `crates/mizar-resolve/tests/lint_policy.rs`
4. `crates/mizar-test/src/runner/tests.rs`
5. `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_generator_variable_identity.rs`

Resolver adds exactly four tests, changing `152 -> 156`:

- `task277r2_collects_exact_f5_generator_binding_and_uses`
- `task277r2_scopes_mapper_before_binder_and_orders_condition_uses`
- `task277r2_ignores_unsupported_and_recovered_fraenkel_shapes`
- `task277r2_revalidates_surface_resolved_arena_and_replays_deterministically`

`mizar-test` adds exactly
`task277r2_real_fixture_links_exact_fraenkel_generator_binding_and_uses`,
changing `610 -> 611`. The private leaf parses the immutable fixture through
the existing helper and directly calls the resolver collector. It is not a
production route, dispatcher, detail key, checker output, typed/resolved slot,
active-stage selection, or semantic test.

| Inventory | Final measured value |
|---|---|
| resolver production source | 23 paths / 35,977 lines; path `4d8c3c499b238814a839ae11994503bbb28f54a3690921c66429dccd298d47c8`; content `8bf8f814a852adce43de16711d7ca7529263b9de79ee002438162c2234b6e4d8` |
| `names.rs` | 3,920 lines / `9a4b1a0e289c058a40c5af91d00fb836eca7af3a1d83bfcfa9b60227ce46d14a` |
| `names/tests.rs` | 3,601 lines / `31228c3502a08276a0c395715f74a6a5143a11c315145595ac88f93163e6863a` |
| resolver lint policy | 1,037 lines / `1a84ba67b715b8df752accd18895fc89a8e727769061a89570b2b4fe15d1182d` |
| resolver library list | 156 / `7c84ee615616d7f0982454c8d04e9eef2fcb451efbb8fd576296e28af3cb6301` |
| mizar-test library list | 611 / `6eaaca04215420028c57731bc14144e2b73ca719dc6cc35f64a5a421e2a1c426` |
| `runner/tests.rs` | 64 lines / `8ae81a6ca4dadd9a58165f09bdde4d2ad3cdcd0884ad7521fe5d1ea90539b316` |
| private fixture leaf | 106 lines / `69b54a4effcb7a740d6588070e6951e3a772cd1818ef9fedcb36426642bf3bf4` |

The protected production runner remains 38 paths / 80,090 lines with path /
content SHA-256
`0ef395004f7feaadf60da0daba7b5da9c52ea4974850adfa2bd9d09081b242aa` /
`990b5ad4798786d9f87c03f76fdbad92fc2edf1f6d84ef3baad67254c79fdd70`.

Completion documentation changes exactly 22 Markdown paths: this pair; paired
resolver names, source/spec, module-boundary, TODO, bilingual, and exit
records; and paired mizar-test harness, module-boundary, TODO, and bilingual
records. The four frozen plan indexes remain unchanged. Together with the five
Rust paths, the implementation change is exactly 27 paths.

## Protected scope and audit decision

`doc/spec`, every `.miz`/expectation artifact, trace metadata
(`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`),
and `doc/design/spec_coverage_audit.md`
(`a31f6fb3bd2b561610630497c58284484d00716dd0b7f210f55bef3bc4bfa6db`)
remain unchanged. The audit has no delta because the F5 seed stays inactive
under `MC-G020` / `MC-G021`, with no coverage or follow-up-owner change.
Cargo, parser, frontend, checker, Core, production runner, diagnostics, active
stage, semantic credit, legacy sections/redirects/ledger, and completed
R1/277B-L APIs and evidence are protected.

Stop and return authority to the parent if implementation requires a new
language or test-intent decision, generic name resolution, nested/shadow
support, partial recovery admission, type/sethood/evidence/verdict semantics,
diagnostics, checker activation, fixture activation, production routing,
protected-artifact changes, or Task 277B readiness.

## Reviews, verification, and exit

The exact four resolver tests and one real-fixture test pass. Resolver and
mizar-test library and lint suites reach 156/611; package Clippy, formatting,
and implementation-time diff checks pass. Independent test-sufficiency and
implementation reviews end with **NO FINDINGS**.

Workspace-wide `cargo test` and warnings-denied Clippy, offline Cargo metadata
and its suite, unchanged five CLI outputs/hashes, and protected count/hash
replay also pass: the frozen F5 source and expectation, trace, coverage audit,
production runner, resolver manifest/test-list hashes, and full protected
path sets reproduce exactly. Independent source/documentation/API, bilingual,
boundary, and final-quality reviews end with **NO FINDINGS**; all nine hard
gates pass with no score cap at valid `100/100`
(`20/20/15/15/10/10/5/5`). Exact staging/cached-diff review, task-only
implementation commit, post-commit proof, and fresh successor inventory are
complete in the historical pre-closure checkpoint below.

## Historical Immediate-Post-Implementation Checkpoint

This is a historical, pre-closure record for task-only implementation commit
`534a8797dc4066f0b07f47dbf440e35369ab80c5`; it does not represent or claim
the current `HEAD`. Immediately after that commit, read-only inventory observed
`HEAD=534a8797dc4066f0b07f47dbf440e35369ab80c5`, a clean worktree,
`origin/main...HEAD=0/13`, and unchanged
`stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`.

The exact 27-path implementation commit has path hash
`43610037f566f01659392ec43c721af6b85ccb28fdbf43a5973b155e6937d62e`.
Exact staging/cached-diff review, the task-only implementation commit,
post-commit proof, and fresh successor inventory completed at this historical
checkpoint. No successor is selected, and Task 277B remains not ready with
zero semantic credit.

## Next-task handoff

No successor is selected by `RESOLVE-FRAENKEL-GENERATOR-VAR-277R2`. Preserve
Task 277B not-ready status and zero semantic credit. Any later lower transport
or checker work requires its own frozen authority, scope, and review record.
