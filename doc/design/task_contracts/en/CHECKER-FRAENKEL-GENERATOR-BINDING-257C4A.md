# Task CHECKER-FRAENKEL-GENERATOR-BINDING-257C4A: Fraenkel Generator Binding Context

> Canonical language: English. Japanese companion: [../ja/CHECKER-FRAENKEL-GENERATOR-BINDING-257C4A.md](../ja/CHECKER-FRAENKEL-GENERATOR-BINDING-257C4A.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

Stable owner sections: checker [binding environment](../../mizar-checker/en/binding_env.md#task-257c4a-fraenkel-generator-binding-environment), [formula composition](../../mizar-checker/en/source_formula_composition.md#task-257c4a-fraenkel-generator-binding-context), [dependency boundary](../../mizar-checker/en/source_template_type_parameter_association.md#task-257c4a-fraenkel-generator-dependency-boundary), [source/spec audit](../../mizar-checker/en/source_spec_audit.md#task-257c4a-fraenkel-generator-source-spec-audit), [module boundary](../../mizar-checker/en/module_boundary_audit.md#task-257c4a-fraenkel-generator-module-boundary), [TODO](../../mizar-checker/en/todo.md#task-257c4a-fraenkel-generator-binding-context), and [bilingual record](../../mizar-checker/en/bilingual_sync_audit.md#task-257c4a-frozen-contract-parity); mizar-test [harness](../../mizar-test/en/harness.md#checker-task-257c4a-private-binding-context-probe), [boundary](../../mizar-test/en/module_boundary_audit.md#checker-task-257c4a-frozen-module-boundary), [TODO](../../mizar-test/en/todo.md#checker-task-257c4a-private-binding-context-probe), and [bilingual audit](../../mizar-test/en/bilingual_sync_audit.md#checker-task-257c4a-frozen-contract-parity).

## Status, authority, and readiness

**Status:** the exact four-path Rust implementation, focused tests, package
libraries, formatting, package Clippy, full-workspace Clippy, and independent
implementation/test-sufficiency reviews and broad workspace verification are
complete. Independent source/documentation and bilingual/boundary reviews are
also complete at **NO FINDINGS** after one Low stale-tense finding was repaired.
Independent final-quality review is **NO FINDINGS**: all `9/9` hard gates pass
at valid uncapped `100/100` (`20/20/15/15/10/10/5/5`). The historical
pre-commit staging/cached review, the task-only implementation commit,
immediate post-commit proof, and accepted fresh successor inventory are
complete. The inventory found C4B
authority-ready in principle but sequence-blocked until this documentation-only
C4A closure; no C4B implementation or semantic successor is selected here, and
a fresh post-closure inventory must freeze a separate C4B documentation
prerequisite. This is a Task-257C owner slice, not a Task-277B readiness
decision.

Authority is, in order: canonical [Chapter 13 §13.4](../../../spec/en/13.term_expression.md#134-set-expressions),
§§13.4.2, 13.4.4, and 13.8.6; [Chapter 18](../../../spec/en/18.templates.md)
§18.10.2; immutable F5
[`fail_template_fraenkel_over_type_param_001.miz`](../../../../tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.miz),
its [trace entry](../../../../tests/coverage/spec_trace.toml), and its
[expectation](../../../../tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.expect.toml);
[Architecture 16 canonical binder normalization](../../architecture/en/16.substitution_and_binding.md#canonical-binder-normalization);
completed [R2](RESOLVE-FRAENKEL-GENERATOR-VAR-277R2.md) and
[277C](CHECKER-FRAENKEL-TEMPLATE-STRUCTURAL-277C.md); then derived design and
source records. The source grammar permits the generator; §§13.4.2 and
18.10.2 do not authorize a sethood verdict here. Section 13.4.4 requires a
resolved binder identity for capture later, while §13.8.6 gives the generator
the existential-bound `x` role. Architecture 16 gives the binder-local,
non-spelling normalization role and distinguishes bound variables from
generated-fresh variables. Accordingly C4A uses `QuantifierBinder`, not
`Generated`.

The completed 277C handoff establishes the exact structural relation and typed
binder; completed R2 establishes the exact resolver collection. Task 257C
already owns the deferred generator binding/reference/capture work, so this
slice neither duplicates R2/277C nor crosses an existing owner. The resolved
planning disagreement was `design_drift`; implementation closes the frozen
`source_drift` and Rust `test_gap`. There is no `spec_gap` and no new semantic
test intent.

## Completed implementation boundary

The completed Rust change has exactly four paths:

1. `crates/mizar-checker/src/binding_env.rs`;
2. `crates/mizar-checker/src/source_formula_composition.rs`;
3. `crates/mizar-test/src/runner/tests.rs`; and
4. new private `crates/mizar-test/src/runner/tests/type_elaboration/template_fraenkel_generator_binding_context.rs`.

It adds this exact public family to `source_formula_composition.rs`:

```rust
SourceFraenkelGeneratorBindingContextId
SourceFraenkelGeneratorBindingContext
SourceFraenkelGeneratorBindingContextTable
SourceFraenkelGeneratorUsePositionId
SourceFraenkelGeneratorUsePosition
SourceFraenkelGeneratorUsePositionTable
SourceFraenkelGeneratorBindingContextHandoff
#[non_exhaustive] SourceFraenkelGeneratorBindingContextError
SourceFraenkelGeneratorBindingContextProducer
```

Each ID exposes only `new(usize) -> Self` and `index() -> usize`. The
binding-context row getters are exactly:

```rust
composition() -> SourceTemplateFraenkelStructuralCompositionId
resolver_binding() -> FraenkelGeneratorVariableBindingId
context() -> BindingContextId
binding() -> BindingId
source_ordinal() -> usize
```

The use-position row getters are exactly:

```rust
binding_context() -> SourceFraenkelGeneratorBindingContextId
resolver_use_index() -> usize
source_ordinal() -> usize
lookup_ordinal() -> usize
```

Each table exposes `get(id) -> Option<&Row>`,
`iter() -> impl Iterator<Item = (Id, &Row)>`, `len() -> usize`, and
`is_empty() -> bool`. The handoff getters are exactly:

```rust
source_id() -> SourceId
module_id() -> &ModuleId
structural_summary() -> &str
resolver_summary() -> &str
binding_env() -> &BindingEnv
bindings() -> &SourceFraenkelGeneratorBindingContextTable
use_positions() -> &SourceFraenkelGeneratorUsePositionTable
debug_text() -> String
```

`structural_summary()` is exactly the existing 277C human summary
`source-template-fraenkel-structural-composition-v1|module=<module>|compositions=1|uses=3`;
`resolver_summary()` is exactly the existing R2 human summary
`fraenkel-generator-variable-source-v1|module=<module>|bindings=1|uses=3`.
They are non-authoritative summaries and are never trusted alone.
`debug_text()` is exactly
`source-fraenkel-generator-binding-context-v1|module=<package>.<path>|bindings=1|use-positions=3`.

The producer signature is exactly:

```rust
SourceFraenkelGeneratorBindingContextProducer::build(
    &SourceTemplateFraenkelStructuralCompositionHandoff,
    &FraenkelGeneratorVariableSourceCollection,
    &TypedAst,
) -> Result<
    SourceFraenkelGeneratorBindingContextHandoff,
    SourceFraenkelGeneratorBindingContextError,
>
```

It retains lower dependency clones only as opaque private fields. No public
getter or debug output exposes a raw resolver node.

`BindingContextOwner` gains
`SourceComprehension { source_range: SourceRange }`; its range participates in
context source-range validation. `BinderIdentity` gains
`SourceBound { context: BindingContextId, ordinal: u32 }`. The coherence rule is deliberately
one-way: `SourceBound => QuantifierBinder`, never the converse, so existing
`QuantifierBinder`/`ResolverLocal` remains valid. A `SourceBound` identity
requires that its context exists and equals its binding owner, and
`usize::try_from(ordinal) == visible_after_ordinal`; captured `SourceBound`
identities also validate their context even though C4A captures none. Its
lookup priority has depth zero; source validity is inherited through the
validated context. The context owner renders exactly
`source-comprehension(<start>..<end>)`; the identity renders exactly
`source_bound(context#<id>, ordinal=<u32>)`. Existing canonical identity
ordering remains debug-key lexical, with exact variant order
`DefinitionShell < Generated < ReservedVariable < ResolverLocal < SourceBound`.
`SourceBound` is distinct and last. Debug and canonical ordering are
domain-separated.

## Exact F5 environment and dependency validation

F5 builds one normal environment: context 0 is empty `Module`; context 1 is
`SourceComprehension { source_range: 663..694 }`, parent 0, `Expression`, no
lexical scope, and owns/visibly contains binding 0. Binding 0 is active,
normal, spelling `x`, `QuantifierBinder`,
`SourceBound { context: context1, ordinal: 0 }`, owner context 1, declaration
`673..674`, `visible_after_ordinal` 0, and type site `Source(678..679)`, with
empty captures and diagnostics. Its sole binding row is composition 0 / R2
binding 0 / context 1 / binding 0 / source ordinal 0.

The F5 `BindingEnv` debug oracle is exactly (with its module placeholder):

```text
binding-env-debug-v1
module: <package>::<path>
contexts:
  context#0 owner=module parent=none layer=module scope=none bindings=[] visible=[] recovery=normal
  context#1 owner=source-comprehension(663..694) parent=context#0 layer=expression scope=none bindings=[binding#0] visible=[binding#0] recovery=normal
bindings:
  binding#0 spelling="x" kind=quantifier_binder owner=context#1 identity=source_bound(context#1, ordinal=0) range=673..674 visible_after=0 type=source(678..679) status=active captured=[] diagnostics=[] recovery=normal
diagnostics:
```

The three use-position rows are normalization only, never term ownership:
resolver use indices and source ordinals 0/1/2 map to lookup ordinals 1/2/3.
Lookup ordinal 0 is a separate pre-visibility probe, not a mapper row, and
must yield `ForwardReference`; ordinals 1/2/3 yield `Local(0)`. C4A creates no
`SourcePrimaryTerm`, reference, capture row, formula, type interpretation,
sethood evidence/request/verdict, diagnostic, Typed/Resolved installation,
production route, sidecar/trace/coverage credit, or Task-277B readiness.

At build and handoff validation, canonical version/domain-tagged opaque
dependency snapshots must fully revalidate the source/module, counts, dense
IDs, every R2 getter (role, spelling, ranges, resolved nodes, and ordinals),
and every 277C getter. Existing debug summaries are never trusted. The actual
R2 binder must map through 277C's unique resolved-to-typed relation to typed
binder 19 with no ID cast. Raw resolver nodes may occur only in this opaque
dependency snapshot/validation, never in `BindingEnv`, a public getter, or
debug output. Default-deny permits exactly one composition, one normal binding,
and three normal uses: no nested/multiple/shadow/recovery form and no partial
handoff. Error precedence is frozen as:

```text
EnvironmentMismatch
InvalidStructuralDependency
InvalidResolverDependency
InvalidBindingContext { binding_context: SourceFraenkelGeneratorBindingContextId }
InvalidUsePosition { use_position: SourceFraenkelGeneratorUsePositionId }
InvalidEnvironment
```

Later C4B must consume this exact handoff before it maps any use or capture.

## Completed tests and protected baselines

The implemented checker tests are exactly these four:

1. `task257c4a_builds_exact_fraenkel_generator_binding_context`;
2. `task257c4a_rejects_environment_structural_and_resolver_corruption`;
3. `task257c4a_rejects_context_identity_range_position_and_profile_corruption`;
4. `task257c4a_rebuilds_deterministically_without_mutation`.

The sole private mizar-test test is
`task257c4a_real_fixture_builds_exact_fraenkel_generator_binding_context`.

Together they prove the single F5 environment, full-field dependency revalidation,
identity/range/position/profile rejection, probe/local lookup behavior, and
determinism without mutation. Test registrations moved only from checker raw
list `546 -> 550` and mizar-test raw list `612 -> 613`.

| Prerequisite Rust path | Baseline lines | Baseline SHA-256 |
| --- | ---: | --- |
| `binding_env.rs` | 3168 | `66454dcc8bc864c15e86d736a3e85deb0b095d3037d757a045055b3e04cebfc5` |
| `source_formula_composition.rs` | 5366 | `827328853d2c74e8287b624adcf18d7a1efd5e6a76c35bde347e06237644d64f` |
| `runner/tests.rs` | 65 | `6d07a5ba5efe0be8f058eb52028e90c0bbb279b5d088604c55e1a9d1ca5e75ba` |
| private leaf | absent | absent |

Fresh completion measurements are:

| Implemented Rust path | Final lines | Final SHA-256 |
| --- | ---: | --- |
| `binding_env.rs` | 3266 | `91c921f1527866692e62e4d542905a02fbb4d8ae695e048c92282ffe793faf39` |
| `source_formula_composition.rs` | 7303 | `f6da763061479e74e7b8f39169ecad311bb9bf879e91e93824d9899798017abc` |
| `runner/tests.rs` | 66 | `85ae891b185ed1eeb5940998c5eef5ece793b472b8f3fa4be3c0b96d217e1f07` |
| private leaf | 156 | `a25b06e7e40bf9922207708f8dfae7d1f97633f744ffba6b9c0578aa31cde4c1` |

Checker production is 32 paths / 193103 lines with unchanged path SHA-256
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787`
and fresh content SHA-256
`cfc9a2bc5359f9baeea39f304e3c9dd15fcbd27749f1c746eb3ab695b84f8dab`.
Mizar-test production remains 38 paths / 80090 lines with unchanged path/content
SHA-256 `0ef395004f7feaadf60da0daba7b5da9c52ea4974850adfa2bd9d09081b242aa` /
`990b5ad4798786d9f87c03f76fdbad92fc2edf1f6d84ef3baad67254c79fdd70`.

The locally rechecked inherited production protections are checker 32 paths /
191068 lines, path SHA-256 `9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787`,
content SHA-256 `cf4e43bb5671f863d9af36f99592ca188bab28b2480acb886e1171d65f57fe8a`; and
mizar-test 38 paths / 80090 lines, path SHA-256
`0ef395004f7feaadf60da0daba7b5da9c52ea4974850adfa2bd9d09081b242aa`, content
SHA-256 `990b5ad4798786d9f87c03f76fdbad92fc2edf1f6d84ef3baad67254c79fdd70`.
Frozen protected authority inventories are 64 English specs, path/content
SHA-256 `d900ba9e43ab094925f36493f830e0c6a2964be2911d5d229014a58842067a25` /
`b30dd5519191a4407399826faf91cc58853d7944df7630734b5ab05de48c9f7b`;
343 `.miz` files,
`d94980e167b4b8ac196f91e7694ff044080c6fb4d04c135b3cd5e65b9a019f22` /
`54e6ea1254a0bd963c39026d788711507f353e9c6df3b4f9fd268b2e9f240afb`;
435 expectations,
`22a5ee257a294e3f2ed4b24bf9ca243d037bbd798f009d0d1e3a176dad8b4fea` /
`b5f0ed1a8d73bbfb78af5cad87a8d426e97269f7f70163750893a9fe1f39d2ea`;
and 21 Cargo files,
`d93f2816b760d8ba45430d6d8570e598864aa7b20b19a001f45171d36fd3a030` /
`146e9b4024d2c344b2eca9c6f5ca6d6a80a3de427e382953a2280bc63cb3ecca`.
`doc/design/spec_coverage_audit.md` is protected at SHA-256
`a31f6fb3bd2b561610630497c58284484d00716dd0b7f210f55bef3bc4bfa6db`.
The prerequisite raw checker list was 546, SHA-256
`2477c548993fcbfffa817814f462ab5d7ce1549a083b6d65aa87091f08bbc9ed`; the
prerequisite raw mizar-test list was 612, SHA-256
`5a8c1170208533ed4d1723acd05a07ab9f62569b47507129d56c14f7fc2af65a`.
Fresh post-implementation raw lists are checker 550, SHA-256
`ba24ea98b25617e41c832ef2dc0878f0502249d68d281fdb4e4c1a7e66c71885`, and
mizar-test 613, SHA-256
`a408a7099e886be8c6f4173325e40e4d9b3e28e42e8cc6cbad9bf88ce95e2741`.
Protected F5 fixture/expectation/trace hashes remain
`32c4a1c1b6c9d98dcb085558a084929e07d4005bf92595865f144456e95854ec`,
`b47ac5113c89cd5703adb0ffd660b52d3e16908c92623dd2f63196aa6a215cb2`, and
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.
The five inherited CLI hash values remain those recorded in completed 277C:
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`, and
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

## Historical immediate post-implementation checkpoint

This completion documentation changes exactly 24 Markdown paths: this paired
contract and the 22 existing owner documents linked above. The four checker/test
crate plans, `doc/design/spec_coverage_audit.md`, the compaction ledger, Rust,
Cargo, specification, fixtures, expectations, trace, and coverage artifacts are
unchanged by the documentation pass. Contract-pair count remains `88/88`; no
authority, traceability, owner crate, or semantic coverage credit changes.

Completed evidence is the focused exact `4 + 1` test set, checker and mizar-test
package libraries at `550/550` and `613/613`, `cargo fmt --all -- --check`, both
package Clippy runs, full-workspace
`cargo clippy --all-targets --all-features -- -D warnings`, and independent
implementation and test-sufficiency reviews at **NO FINDINGS**. Full
`cargo test` passes after documentation integration, including checker and
mizar-test lint policy `15/15` each and metadata `137/137`. All five CLI
replays match the frozen plan/parse/declaration/type/proof hashes with the
expected 23 warnings and zero errors.

The first source/documentation review reported one Low wording finding: the
completed baseline table still said `Future Rust path`. It now says
`Prerequisite Rust path`. The finding-specific re-review and the independent
bilingual/boundary review both report **NO FINDINGS**.

Independent final-quality review reports **NO FINDINGS**; all `9/9` hard gates
pass at valid uncapped `100/100` (`20/20/15/15/10/10/5/5`). The task-only
implementation commit, immediate post-commit proof, and accepted fresh
successor inventory are complete.

Before that commit, exact staging/cached review passed with exactly 28 cached
paths: 4 Rust and 24 documentation paths, including one new private leaf, with
no unstaged paths at review time. `git diff --cached --check` passed and the
pre-commit cached stat was exactly 2435 insertions / 101 deletions.

Immediately after the task-only implementation commit
`5f8eece10236b06c777e29ec80a2e82e3c1e6635`, the observed `HEAD` was that
commit, the worktree was clean, `origin/main...HEAD` was `0/19`, and
`stash@{0}` remained unchanged at
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`. The commit contained exactly 28
paths; its reproducible sorted-path SHA-256 was
`b4aae2c38d47c08d7b2acc77b9f0e06669120aba748730bf041e51d70884598b`, and its
final stat was 2464 insertions / 101 deletions. These are historical,
immediate-post-implementation observations from before this documentation-only
closure, not claims about any later or current `HEAD`, index, worktree, origin,
or stash state.

The accepted fresh inventory found C4B authority-ready in principle but
sequence-blocked until this documentation-only C4A closure. It selects neither
a C4B implementation nor any semantic successor. A fresh post-closure inventory
must separately freeze the C4B documentation prerequisite before implementation.
Sol `xhigh` retains integration/final acceptance; Terra
`xhigh` reviews the completed API/validation/test boundary. Task 277B remains
not ready with zero semantic credit. Only after that separate freeze may C4B
consume the opaque C4A handoff for use/capture mapping.
