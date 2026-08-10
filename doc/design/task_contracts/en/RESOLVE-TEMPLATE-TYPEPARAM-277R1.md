# Task RESOLVE-TEMPLATE-TYPEPARAM-277R1: Resolver Template Type-Parameter Identity Prerequisite

> Canonical language: English. Japanese companion: [../ja/RESOLVE-TEMPLATE-TYPEPARAM-277R1.md](../ja/RESOLVE-TEMPLATE-TYPEPARAM-277R1.md).

Owning plans: [mizar-resolve](../../mizar-resolve/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

Stable owner sections:

- resolver [names](../../mizar-resolve/en/names.md#resolver-task-277r1-template-type-parameter-identity), [source/spec correspondence](../../mizar-resolve/en/source_spec_correspondence.md#resolver-task-277r1-sourcespecification-correspondence), [boundary](../../mizar-resolve/en/module_boundary_refactor.md#resolver-task-277r1-module-boundary), [TODO](../../mizar-resolve/en/todo.md#resolver-task-277r1-frozen-documentation-prerequisite), [bilingual record](../../mizar-resolve/en/bilingual_documentation_synchronization.md#resolver-task-277r1-bilingual-synchronization), and [exit addendum](../../mizar-resolve/en/crate_exit_report.md#resolver-task-277r1-post-exit-prerequisite);
- test [harness](../../mizar-test/en/harness.md#resolver-task-277r1-test-only-fixture-probe), [module boundary](../../mizar-test/en/module_boundary_audit.md#resolver-task-277r1-test-module-boundary), [TODO](../../mizar-test/en/todo.md#resolver-task-277r1-test-only-fixture-probe), and [bilingual audit](../../mizar-test/en/bilingual_sync_audit.md#resolver-task-277r1-contract-parity).

## Status, authority, and classification

| Field | Frozen value |
|---|---|
| Status | The documentation prerequisite is committed as `2438cbb7d39c1844557293b270ef1784cfc31ece`; the task-only implementation is committed as `b22033c38249326e366ceb9e19b1a9100da2248e`. Task 277R1 is complete. Independent source/documentation, bilingual, and final-quality reviews report **NO FINDINGS**; all nine hard gates pass without a score cap at valid `100/100`. Task 277B remains not ready. |
| Pre-implementation selection checkpoint | `HEAD=0827e494df96afacba4f35b9cc23dfbbb737d141`; `origin/main...HEAD=0/5`; protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`, unchanged. |
| Post-implementation checkpoint | Immediately after task-only implementation commit `b22033c38249326e366ceb9e19b1a9100da2248e`, read-only inventory observed `HEAD=b22033c38249326e366ceb9e19b1a9100da2248e`, a clean worktree, `origin/main...HEAD=0/7`, and unchanged protected `stash@{0}=f65cf4a13752ec380710814a9ac6392ccb9d75d4`. Task 277R1 is complete; any successor must be separately frozen and reviewed. |
| Authority | `doc/spec/en/18.templates.md` §§18.2.1, 18.2.2, 18.2.6, 18.10.2 and `doc/spec/en/13.term_expression.md` §13.4.2. The parser prerequisite is [PARSER-TEMPLATE-TYPEHEAD-277P1](./PARSER-TEMPLATE-TYPEHEAD-277P1.md). |
| Classification | `source_drift`, `design_drift`, and Rust `test_gap`; no `spec_gap`. The later missing-sethood verdict remains checker-owned. |
| Consumer | This remains only the resolver prerequisite for later Task 277B. Its completion neither makes Task 277B ready nor selects a checker implementation. |

The immutable semantic seed remains
`tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.miz`, 701
bytes with final LF, SHA-256
`32c4a1c1b6c9d98dcb085558a084929e07d4005bf92595865f144456e95854ec`.
Its 839-byte sidecar remains SHA-256
`b47ac5113c89cd5703adb0ffd660b52d3e16908c92623dd2f63196aa6a215cb2`
and inactive at `advanced_semantics`.

## Frozen real-source profile

The real parser profile is root `56` of 57 nodes with no diagnostics:

| Role | Surface node | Range |
|---|---:|---:|
| enclosing declaration | `DefinitionBlockItem#53` | `593..700` |
| direct template parameter | `TemplateParameter#31` | `606..620` |
| declaration binder | `Identifier` token `#2`, text `T` | `610..611` |
| generator type head | `TypeHead#39` → identifier token `#21`, text `T` | `678..679` |
| enclosing generator / term | `ComprehensionVariableSegment#41`, `SetComprehension#49`, `FunctorDefinition#52` | `673..679`, `663..694`, `623..695` |

The declaration wrapper is explicitly **not** `DefinitionParameter`; a leading
template `let T be type;` is `TemplateParameter#31`. This task transports only
the validated declaration/use structural relation. It does not decide that the
bare type is a set, interpret template actuals or substitution, or emit the
later Chapter-18/13 rejection.

## Frozen resolver API and validation

Only `crates/mizar-resolve/src/names.rs` may gain these public data/API names:

- `TemplateTypeParameterBindingId` with `new` and `index`;
- `TemplateTypeParameterBinding`;
- `TemplateGeneratorTypeHeadLink`;
- `TemplateTypeParameterBindingTable` and `TemplateGeneratorTypeHeadLinkTable`;
- `TemplateTypeParameterSourceCollection`; and
- `TemplateTypeParameterSourceCollector`.

`TemplateTypeParameterBinding` has fields/getters for `definition_block`,
`parameter`, `binder`, `spelling`, `source_range`, and `source_ordinal`.
`TemplateGeneratorTypeHeadLink` has fields/getters for `definition_block`,
`type_head`, `identifier`, `binding`, `source_range`, and `source_ordinal`.
`TemplateTypeParameterBindingTable` exposes `get(BindingId)`, `iter` as
`(id, row)`, `len`, and `is_empty`; `TemplateGeneratorTypeHeadLinkTable`
exposes `get(usize)`, `iter` as rows, `len`, and `is_empty`. There is no link
ID. The collection exposes `source_id`, `module`, `bindings`,
`generator_links`, and `debug_text`.

The exact collector signatures are
`new(&SurfaceAst, &ModuleId, &SurfaceResolvedArena) -> Result<Self, SurfaceResolvedArenaError>`
and `collect(&self) -> Result<TemplateTypeParameterSourceCollection, SurfaceResolvedArenaError>`.
They validate the complete structural arena at both boundaries. No custom
public error enum or lint-policy change is allowed.

Collection is default-deny per `DefinitionBlockItem` owner. It admits each
unrecovered, unbounded, single-binder direct `TemplateParameter` and only a
same-owner generator role whose `TypeHead` is under `TypeExpression`,
`ComprehensionVariableSegment`, and `SetComprehension`. This fixture yields
exactly one binding and one link. Exact `Identifier` token/text equality is an
explicitly authorized **non-inferential resolver structural match**. Duplicate
same-owner spellings still yield their bindings but no ambiguous link. Recovery, bounds, constraints, multiple binders,
predicate/functor forms, wrappers, cross-owner references, and non-generator
roles are ignored.

The implementation may obtain resolver-owned node identity only through
`SurfaceResolvedArena::resolved_node_for`. It must not construct IDs, allocate
`SymbolId`, resolve a `NameRef`, extend `ResolvedAst`, infer spelling aliases,
or introduce diagnostics, type facts, sethood, verdicts, checker state, or a
public route.

## Frozen implementation, tests, and inventory

After this documentation prerequisite, the exact Rust scope is four paths:

1. `crates/mizar-resolve/src/names.rs`
2. `crates/mizar-resolve/src/names/tests.rs`
3. `crates/mizar-test/src/runner/tests.rs`
4. `crates/mizar-test/src/runner/tests/type_elaboration/template_parameter_identity.rs`

The resolver adds exactly these four tests, changing `148 -> 152`:

- `task277r1_collects_exact_template_generator_identity`
- `task277r1_isolates_scope_and_ignores_non_generator_roles`
- `task277r1_rejects_unsupported_parameter_and_recovery_shapes`
- `task277r1_revalidates_surface_resolved_arena_and_replays_deterministically`

`mizar-test` adds exactly
`task277r1_real_fixture_links_exact_template_generator_identity`, changing
`608 -> 609`. It parses the immutable fixture through the existing helper and
asserts the frozen real profile only. It is no production route, detail key,
checker output, Typed/Resolved slot, active-stage selection, or semantic test.

Resolver baseline is 23 Rust paths / 33,402 lines, path hash
`4d8c3c499b238814a839ae11994503bbb28f54a3690921c66429dccd298d47c8`,
content hash
`894297b7f5e7a1ba387c1bcf1c34d528b60482e7f0ac8a623a9c452aaf26d633`.
`names.rs` is 2,749 lines / `eff47c86f043c83daecef2631e0a53472bacd79a8288adb125bcc7139c762081`;
`names/tests.rs` is 2,197 lines /
`6770e085061c29cad9d571d09741b7384175189b5a9d0bfdf1de6c765cdc0a7f`.
The exact raw-list command `cargo test -q -p mizar-resolve --lib -- --list |
sha256sum` yields
`c99d9d179cf14ab9ccd274b11d0404bdc47a64d23a2aa914c69ba674d01a3fee`.

The production mizar-test inventory is unchanged at 38 paths / 80,090 lines,
path/content hashes
`0ef395004f7feaadf60da0daba7b5da9c52ea4974850adfa2bd9d09081b242aa` /
`990b5ad4798786d9f87c03f76fdbad92fc2edf1f6d84ef3baad67254c79fdd70`;
production runner source is excluded from the implementation diff. `runner/tests.rs` is 61 lines /
`8eb35411834b0a6af48f935839f5c83d063fd7226565fd35478fe9e4a3f7c659`.
The raw-list command `cargo test -q -p mizar-test --lib -- --list | sha256sum`
yields `0245b6b6d3f5f0687b5df3f8c7d1edc25cefe2e95ac04b2d7c4a89b141f99aa2`.

The contract trees grow `83/83 -> 84/84`. This documentation prerequisite
changes exactly these 26 Markdown paths:

1. `doc/design/task_contracts/en/RESOLVE-TEMPLATE-TYPEPARAM-277R1.md`
2. `doc/design/task_contracts/ja/RESOLVE-TEMPLATE-TYPEPARAM-277R1.md`
3. `doc/design/mizar-resolve/en/00.crate_plan.md`
4. `doc/design/mizar-resolve/ja/00.crate_plan.md`
5. `doc/design/mizar-resolve/en/names.md`
6. `doc/design/mizar-resolve/ja/names.md`
7. `doc/design/mizar-resolve/en/source_spec_correspondence.md`
8. `doc/design/mizar-resolve/ja/source_spec_correspondence.md`
9. `doc/design/mizar-resolve/en/bilingual_documentation_synchronization.md`
10. `doc/design/mizar-resolve/ja/bilingual_documentation_synchronization.md`
11. `doc/design/mizar-resolve/en/todo.md`
12. `doc/design/mizar-resolve/ja/todo.md`
13. `doc/design/mizar-resolve/en/crate_exit_report.md`
14. `doc/design/mizar-resolve/ja/crate_exit_report.md`
15. `doc/design/mizar-resolve/en/module_boundary_refactor.md`
16. `doc/design/mizar-resolve/ja/module_boundary_refactor.md`
17. `doc/design/mizar-test/en/00.crate_plan.md`
18. `doc/design/mizar-test/ja/00.crate_plan.md`
19. `doc/design/mizar-test/en/harness.md`
20. `doc/design/mizar-test/ja/harness.md`
21. `doc/design/mizar-test/en/module_boundary_audit.md`
22. `doc/design/mizar-test/ja/module_boundary_audit.md`
23. `doc/design/mizar-test/en/bilingual_sync_audit.md`
24. `doc/design/mizar-test/ja/bilingual_sync_audit.md`
25. `doc/design/mizar-test/en/todo.md`
26. `doc/design/mizar-test/ja/todo.md`

Future implementation completion documentation is exactly the 22 paths in
this list excluding only the four plan-index files numbered 3, 4, 17, and 18;
with the four Rust files, its total scope is exactly 26 paths.

## Protected scope and exit

`doc/spec`, all `.miz`/expectation/sidecar files, trace metadata
(`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`),
`doc/design/spec_coverage_audit.md`
(`a31f6fb3bd2b561610630497c58284484d00716dd0b7f210f55bef3bc4bfa6db`),
Cargo files, parser, frontend, checker, Core, production runner, diagnostic
surface, active stage, coverage credit, formal/actual substitution, overload,
and all sethood or verdict semantics stay unchanged.

Stop and return authority to the parent if the work requires generic template
resolution, spelling inference beyond exact token equality, aliases or
shadowing, a bounded/constraint interpretation, semantic sethood, checker
activation, a public diagnostic, fixture activation, parser/frontend/cache
change, or Task 277B readiness. The audit has no coverage-audit delta.

This documentation prerequisite exits only after exact-scope review,
`git diff --check`, and recursive task-contract/link lint pass. A later fresh
preflight must remeasure every baseline before implementation; it then runs the
five frozen tests, relevant crate/workspace checks, and the same protected
hash/count gates before any separately authorized commit.

## Implementation evidence and current status

Fresh preflight reproduced every frozen baseline before the implementation.
The exact four Rust paths now implement the seven-name public resolver API,
two-boundary `SurfaceResolvedArena` validation, resolver-only node identity,
per-owner exact token matching, duplicate-binding ambiguity omission, global
generator-link source ordering, and fail-closed recovery across the complete
candidate `SetComprehension` subtree. The one private mizar-test leaf observes
the immutable real fixture without adding a production route or semantic
verdict.

Final measurements are:

- resolver source: 23 paths / 34,661 lines, unchanged path SHA-256
  `4d8c3c499b238814a839ae11994503bbb28f54a3690921c66429dccd298d47c8`
  and content-manifest SHA-256
  `d3f423448046180bb2db90f50d12518937fe00f5d0fb2ba188348db9bd08ab0e`;
- `names.rs`: 3,248 lines /
  `de87c34a9afedd3649b410f4cf422b883a6fd567a1d61dc78221945320476548`;
  `names/tests.rs`: 2,957 /
  `6d7c6c03fb15edd28af5428cf134bebb7d91686941429ea48d2e432837b55b40`;
- resolver library: 152 tests, raw-list SHA-256
  `924e4652edfc9303d5d5742d3e3eb2b9a095ee6f0f543c8b7caa0a78f0c7b747`;
- `runner/tests.rs`: 62 lines /
  `7c5cc9541b1cd2aabe050d3791e9153faeb302803cfa79abe39bfb58cb181d60`;
  the new leaf: 67 /
  `5cafa3b0cd46ed29b8981f509b3fbec98f40be14e2ce8eee83bc7f10314bc1b8`;
- mizar-test library: 609 tests, raw-list SHA-256
  `ea6e33af0de7353fa13517962c3b0e182cbcb3fc64bb06e5a61e3113daadb82c`;
  production remains 38 paths / 80,090 lines with the frozen path/content
  hashes.

After finding-specific repairs, independent test-sufficiency and implementation
reviews report **NO FINDINGS**. Focused tests pass `4/4 + 1/1`; resolver package
tests pass `152` library, `11` lint-policy, and the existing doctest; mizar-test
passes `609` library, `15` lint-policy, and `137` metadata tests. Full workspace
`cargo test`, `cargo fmt --all --check`, all-target/all-feature warnings-denied
Clippy, offline Cargo metadata, all five CLIs with their frozen stdout hashes,
protected hash/count replay, and `git diff --check` pass. Fixture, sidecar,
trace, coverage audit, active stage, diagnostics, semantic coverage, checker,
and production runner remain unchanged. Independent source/documentation
consistency and bilingual reviews report **NO FINDINGS**. Independent
final-quality review also reports **NO FINDINGS**; all nine hard gates pass
without a score cap at valid `100/100` (`20/20/15/15/10/10/5/5`). Exact
staging, the implementation commit, and the immediate post-implementation
inventory are complete at the historical checkpoint above. Task 277B remains
not ready and no successor is selected by this task.

The five CLI stdout SHA-256 values for plan, parse-only, declaration-symbol,
type-elaboration, and proof-verification are respectively
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

## Next-task handoff

Start with fresh authority, dependency, and scope inventory. Do not start Task
277B until a separately frozen and reviewed lower-owner association task makes
its checker consumer dependency-ready. Parent authority interpretation and
final scoring remain GPT-5.6 Sol `xhigh`. Luna is not exposed in this runtime;
the effective bounded inventory/review route is GPT-5.6 Terra `xhigh`.
Escalate every authority or boundary ambiguity to Sol.
