# Task CHECKER-FRAENKEL-NESTED-BINDER-USE-257C4C3: Nested Fraenkel Binder/Mapper-Use Transport

> Canonical language: English. Japanese companion:
> [../ja/CHECKER-FRAENKEL-NESTED-BINDER-USE-257C4C3.md](../ja/CHECKER-FRAENKEL-NESTED-BINDER-USE-257C4C3.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).
Durable owner sections: checker
[source formula composition](../../mizar-checker/en/source_formula_composition.md#task-257c4c3-nested-fraenkel-bindermapper-use-transport)
and test [harness](../../mizar-test/en/harness.md#checker-task-257c4c3-private-nested-binderuse-probe).

## Status, purpose, and readiness

**Status:** complete. All independent substantive reviews report **NO
FINDINGS**, all `9/9` hard gates pass, and the valid quality score is
`100/100`. Exact staging/cached review, the task-only implementation commit,
clean postcommit proof, and fresh successor inventory are recorded below.

Fresh read-only inventory at clean
`HEAD e5ffc6bc036ed5d7ba3c173e23671f1c4511ba6a` selects this task as the
dependency-minimal successor to completed C4C2. The human owner decision fixes
Task 257C and `source_formula_composition` as the first checker owner and
requires a zero-semantic identity transport before any Task-252 primary
occurrence. This task therefore maps the sole C4C2 inner mapper use to the
distinct outer generator binder in one immutable checker handoff. It neither
interprets nor installs that relation.

There is no `spec_gap`: Chapter 13 fixes the resolved outer-binder identity and
the existing `.miz`/inactive sidecar fix the exact positive profile. C4C2
completed the resolver relation. The missing checker identity handoff is
`source_drift`; its absent exact checker/private imported-fixture tests are a
`test_gap`; and the previously undecided first checker owner was
`design_drift`, now resolved by the human owner decision. Reusing exact-F5
C4A/C4B, creating or copying Task-252 occurrences, or publishing capture or a
semantic result is a `boundary_violation`.

## Authority, dependencies, and protected artifacts

Authority is, in order:

1. canonical [Chapter 4 §4.6](../../../spec/en/04.variables_and_constants.md#46-scoping-and-shadowing)
   and [Chapter 13 §§13.4.2, 13.4.4, and 13.8.6](../../../spec/en/13.term_expression.md#134-set-expressions);
2. exact existing
   [`pass_types_nested_comprehension_outer_generator_capture_001.miz`](../../../../tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.miz);
3. its sole existing [trace row](../../../../tests/coverage/spec_trace.toml);
4. its inactive [expectation](../../../../tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.expect.toml);
5. completed [C4C0](TEST-FRAENKEL-NESTED-CAPTURE-257C4C0.md),
   [C4C1](TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md), and
   [C4C2](RESOLVE-FRAENKEL-NESTED-CAPTURE-257C4C2.md), followed by derived
   design/source inventory.

The source, inactive sidecar, and trace remain byte-identical with SHA-256
`c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431`,
`9ed000a30c1d519bd665f338c636fb9e529e9848a285209bebe6728f19961b92`, and
`d4d817e83aac78d19e729702b26c62604fc57581eec18672a5c26ec44efe7a81`.
The sidecar stays inactive `advanced_semantics`, `pass/type_check`, with no
diagnostic codes or active tags. The trace row remains test-intent-only and
grants no execution or semantic credit.

Dependencies are the clean C4C2 implementation/closeout commits
`601db2ab8fbcfa736d4b619e0eacbbf1291cc800` and
`e5ffc6bc036ed5d7ba3c173e23671f1c4511ba6a`, the exact imported frontend
admission, the C4C2 `FraenkelGeneratorVariableSourceCollection`, and an
unrecovered one-to-one `ResolvedNodeId` to `TypedNodeId` projection. C4A/C4B
are compatibility-only negative profiles and are not dependencies.

## Selected decomposition and forbidden alternatives

The selected split is the smallest zero-semantic checker boundary: retain a
private clone of the complete resolver collection and `TypedAst`, authenticate
their environment and exact nested profile, and publish one row containing
only the resolver use/binding identities and their two typed node sites. This
unblocks a future separately contracted capture or occurrence consumer without
choosing either semantic owner now.

Rejected alternatives are:

- extend C4A/C4B: those public families and validators are exact-F5-only and
  require a different one-binding/three-use structural dependency;
- create a Task-252 term/reference row first: explicitly excluded by the human
  owner decision and not fixed by the current checker oracle;
- construct a `BindingEnv`, `BindingId`, or `CapturedFreeVariables`: each would
  add checker semantic state beyond identity transport;
- install the handoff in `TypedAst`/`ResolvedTypedAst` or expose it through a
  runner: no active consumer, request/result contract, or execution oracle
  exists.

## Frozen public API and ownership

`crates/mizar-checker/src/source_formula_composition.rs` is the sole production
owner. The exact new public family is:

```rust
SourceNestedFraenkelBinderUseId
SourceNestedFraenkelBinderUse
SourceNestedFraenkelBinderUseTable
SourceNestedFraenkelBinderUseHandoff
#[non_exhaustive] SourceNestedFraenkelBinderUseError
SourceNestedFraenkelBinderUseProducer
```

The dense ID exposes only:

```rust
new(index: usize) -> Self
index(self) -> usize
```

The immutable row exposes exactly:

```rust
resolver_use_index() -> usize
resolver_binding() -> FraenkelGeneratorVariableBindingId
outer_binder() -> TypedNodeId
inner_mapper_use() -> TypedNodeId
source_ordinal() -> usize
```

The table exposes exactly:

```rust
get(id: SourceNestedFraenkelBinderUseId) -> Option<&SourceNestedFraenkelBinderUse>
iter() -> impl Iterator<Item = (SourceNestedFraenkelBinderUseId, &SourceNestedFraenkelBinderUse)>
len() -> usize
is_empty() -> bool
```

The handoff exposes exactly:

```rust
source_id() -> SourceId
module_id() -> &ModuleId
resolver_summary() -> &str
binder_uses() -> &SourceNestedFraenkelBinderUseTable
debug_text() -> String
```

The producer signature is exactly:

```rust
SourceNestedFraenkelBinderUseProducer::build(
    resolver: &FraenkelGeneratorVariableSourceCollection,
    typed_ast: &TypedAst,
) -> Result<SourceNestedFraenkelBinderUseHandoff, SourceNestedFraenkelBinderUseError>
```

No public dependency getter, mutable accessor, unchecked constructor, role
enum, capture flag, semantic value, or installation API is added. The sole
current consumer is one private `mizar-test` library regression in the existing
`fraenkel_nested_capture_identity.rs` leaf. Any production or semantic consumer
requires a separate reviewed contract.

`resolver_summary()` is exactly the non-authoritative C4C2 string
`fraenkel-generator-variable-source-v1|module=<package>.<path>|bindings=2|uses=1`;
the retained resolver snapshot, not this string, is authoritative. The exact
handoff debug grammar is
`source-nested-fraenkel-binder-use-v1|module=<package>.<path>|binder-uses=1`.

## Exact row, validation, and default deny

The handoff contains exactly one row:

| ID | Resolver use | Resolver binding | Outer binder | Inner mapper use | Source ordinal |
|---:|---:|---:|---:|---:|---:|
| `0` | `0` | `1` | typed node for `x@136..137` | typed node for `x@94..95` | `0` |

The two typed sites are exact `TypedNodeId`s uniquely associated with C4C2
resolved nodes. `outer_binder` is a normal `Identifier` whose source range is
`136..137`; `inner_mapper_use` is the normal mapper identifier at `94..95`.
The retained dependency validation also authenticates the shared
definition/functor, distinct inner/outer comprehensions, both generator
segments and binders, inner mapper owner/reference/identifier chain, C4C2
binding order `inner y = 0`, `outer x = 1`, sole `Mapper` use 0 targeting
binding 1, source/role ordinals 0, exact ranges, complete typed child edges,
normal recovery, and unique resolved-to-typed mapping. The row never derives a
binding from spelling or range; it copies the already resolved identity only
after the complete relation validates.

The private snapshot version/domain are exactly
`source-nested-fraenkel-binder-use-dependencies-v1` and
`source-nested-fraenkel-binder-use`. The non-exhaustive error enum has exactly
these four variants, in this validation precedence:

```rust
EnvironmentMismatch
InvalidResolverDependency
InvalidTypedDependency
InvalidBinderUse { binder_use: SourceNestedFraenkelBinderUseId }
```

Wrapper/dependency source or module mismatch wins. Invalid snapshot
version/domain, wrong resolver count/field/order/summary, or a non-C4C2 profile
is next. Missing, duplicate, recovered, wrongly typed, wrongly ranged, or
detached resolved-to-typed nodes follow. Only then does the lowest invalid
dense row return `InvalidBinderUse`; a wrong total count reports ID 0.

Admission is atomic and default-deny. Missing/extra/reordered/duplicated
binding or use rows, C4A/C4B F5 shapes, equal binders, alternate generator
types, condition-bearing or extra nested comprehensions, recovery, duplicate
resolved mappings, detached parent/child edges, stale summaries/snapshots,
wrong sites, or partial matches publish no handoff.

## Implementation, tests, counts, and audit impact

Implementation changes exactly:

1. `crates/mizar-checker/src/source_formula_composition.rs`; and
2. existing private
   `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs`.

The checker owner adds exactly four tests:

1. `task257c4c3_builds_exact_nested_binder_use_handoff`;
2. `task257c4c3_rejects_environment_resolver_and_typed_dependency_corruption`;
3. `task257c4c3_rejects_row_cardinality_order_and_site_corruption`; and
4. `task257c4c3_replays_deterministically_and_rejects_f5_profiles`.

The existing private leaf adds exactly
`task257c4c3_real_imported_fixture_builds_checker_identity_handoff`. It uses
the existing C4C1 frontend and typed-profile helpers, calls the producer
directly, asserts the one public row and immutable dependencies, and remains
library-test-only. It does not activate the sidecar or route.

Raw library counts project `mizar-checker 554 -> 558` and `mizar-test 619 ->
620`. Baseline sorted raw test-list hashes are checker
`78f0291fb13aed8a8adbbc5aa1db9df1a7415fc9d8cf35820e1ad9e40aad2ace`
and mizar-test
`ad70984d911bd6ef84fc5efa15a50815acc7b4cc7daab1c89235263e022aa00b`.
The checker owner baseline is `7958` lines / SHA-256
`90b339d9707f9f8d847b678721e8db0ef6a00e4a15dbb41474a0cf6980f47168`;
the private leaf is `169` lines / SHA-256
`fa70dc53fb92376fcbd71b4058d9830355ab12fc4e5d6f67050d129cb7f46ae9`.
Contract trees project `92/92 -> 93/93`.

`doc/design/spec_coverage_audit.md` receives a zero-credit design-mapping
addendum because the first checker identity transport becomes covered by a
durable design/API/test owner. Trace and expectation metadata remain unchanged:
no executable, semantic, sethood, occurrence, capture, request/result,
diagnostic, route, or Task-277B credit changes.

## Reviews, verification, exit, and handoff

Required independent reviews are specification/contract, test sufficiency,
implementation, source/documentation/API, bilingual/boundary, and final
hard-gate quality. Every material finding requires repair and re-review.

Verification includes the five focused tests, checker and mizar-test library
tests, both lint-policy suites, metadata tests, formatting, warnings-denied
workspace Clippy, full workspace tests, five unchanged CLI replays, protected
artifact hashes, exact scope/diff checks, and post-commit proof.

Exit requires **NO FINDINGS**, all nine autonomous hard gates, valid quality
`>=90/100`, a task-only implementation commit, clean/stash-invariant
post-commit proof, and fresh successor inventory. Task 277B stays not-ready and
zero-credit.

The next inventory must first consider a separately owned Task-252
mapper-primary-occurrence prerequisite because this task deliberately does not
create one. It may select that or another zero-semantic structural transport
only if authority, exact oracle, dependencies, and sole ownership are unique.
Type/sethood, `CapturedFreeVariables`, generated-core parameters,
request/result, verdicts, diagnostics, production installation, runner
activation, and coverage credit remain deferred.

Recommended routing: GPT-5.6 Sol at `xhigh` owns authority, public API and
boundary acceptance, finding disposition, and final scoring. Terra at `high`
or `xhigh` may perform frozen implementation and independent reviews.

## Frozen documentation prerequisite checkpoint

Independent specification/API review initially found the missing exact debug
grammar and exact error-variant-set statement; independent bilingual/boundary
review found a duplicated audit addendum and premature implementation-tense
wording. Those findings were repaired. Finding-specific re-reviews report
**NO FINDINGS**. `git diff --check`, checker and mizar-test lint-policy suites,
and all `137/137` metadata tests pass. At that checkpoint implementation
remained pending until the documentation prerequisite commit and its clean
fresh inventory confirmed the frozen dependencies.

## Precommit implementation completion evidence

The documentation prerequisite is commit
`f985c9337e1bf59f93a9276abda72c5827924544`. The implementation changes only
the two frozen Rust paths. The checker owner retains private clones of the
C4C2 resolver collection and `TypedAst`, authenticates the exact environment,
snapshot tags, two-binding/one-use relation, full local typed child layout,
`Element of NAT` token shape, normal one-to-one projection, and one dense row,
then publishes only the frozen read-only identity surface. The private runner
test calls that producer directly against the real imported fixture without
installing or activating it.

Final source measurements are:

| Path | Lines | SHA-256 |
|---|---:|---|
| `crates/mizar-checker/src/source_formula_composition.rs` | `9358` | `eed8c480a2ddeceafd529ee4c37c333f6e36f8f23e62f4b53f782bc9df651b6c` |
| `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs` | `248` | `46bb3e63199d4b9794a9d56c214d76864a073cc35b0643ec64a8a1e412d5bb0a` |

The raw library lists are exactly `558` checker tests and `620` mizar-test
tests. Their raw-list SHA-256 values are respectively
`aa1eccf5bd93c9574082f7c888918ccb2bbc76167aa5ef0c672a6db931e42d8f`
and `95ff9e007bd474cad657e626f61424db408ec343f6f1a6c1b84d6fff50ee9a75`.
Contract trees are `93/93`, and corpus source/sidecar pairs remain `344/344`.
The protected source, inactive sidecar, and trace hashes remain the three
frozen values above.

The first test-sufficiency review found incomplete dependency-corruption,
precedence, row-field, and dense-iterator coverage. The first implementation
review found an incomplete typed containment/type subtree and missing resolver
range source identity. Repair added complete local typed structure and token
authentication, full-source-range checks, exact row/API validation, and the
required corruption matrix. Follow-up findings for extra typed children,
`Default`, module mismatch, direct retained-resolver corruption, exact root
scaffolds/reachability, source-spec public inventory, harness replay ownership,
lifecycle wording, and measurements were also repaired. Finding-specific test
sufficiency, implementation, source/documentation/API, and bilingual/boundary
re-reviews all report **NO FINDINGS**.

The five focused tests pass, including checker `4/4` and private mizar-test
`1/1`. Checker and mizar-test library totals pass `558/558` and `620/620`;
their lint-policy suites pass `15/15`, metadata passes `137/137`, and the full
workspace `cargo test` passes. `cargo fmt --all -- --check`, offline Cargo
metadata, warnings-denied all-target/all-feature Clippy, and `git diff --check`
pass. Plan/parse/declaration/type/proof CLI stdout hashes are respectively
`2d2accef2c6fc32c1b3372530f6136af1299ac6ae7db6a0158798336b779c7e7`,
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`;
each retains the existing `23` warnings and zero errors. All `9/9` hard gates
pass by parent review. Independent final-quality review reports **NO
FINDINGS**, no score cap, and `100/100`, split
`20/20/15/15/10/10/5/5`. It independently reran the five focused tests and
`git diff --check`, verified the exact 17-path scope, and authorized exact
staging.

## Postcommit proof and fresh successor inventory

The task-only implementation commit is
`4028e694e0d522ed31c2d00416860c82f2fc87b7` (`feat(checker): transport
nested Fraenkel binder use`) with documentation-prerequisite parent
`f985c9337e1bf59f93a9276abda72c5827924544`. It changes exactly the frozen
implementation/completion surface of 17 paths; the sorted committed path-list
SHA-256 is
`914eaf5b090955a8aefbe521cca952000fb9dd9f0fa967e1af35c300e512225c`,
and `git show --check` passes. Immediately afterward the worktree was clean,
`origin/main...HEAD` was `0/2`, and the pre-existing stash remained untouched
at `f65cf4a13752ec380710814a9ac6392ccb9d75d4`. The protected source, inactive
sidecar, and trace retained their frozen hashes.

Fresh independent authority-, source-owner-, and boundary-oriented inventories
all considered the required Task-252 mapper-primary occurrence first and found
no uniquely dependency-ready implementation successor. C4C3 fixes only the
one resolver/typed identity from mapper `x@94..95` to outer binder
`x@136..137`. The existing Task-252 producer instead derives lookup order from
bindings whose declaration ranges end before the occurrence and requires the
winning binding declaration to precede that occurrence. It therefore rejects
this deliberately forward-written Fraenkel binder unless a separately reviewed
Fraenkel-specific occurrence rule and dependency are selected.

A distinct zero-semantic nested binding-context transport is the most direct
possible prerequisite, but its exact oracle is also not fixed. Current
authority does not choose the context graph, whether the inner `y` is included,
checker binding cardinality/order, the outer binding identity/visibility
ordinal, mapper lookup context/ordinal, or whether this state belongs to a
separate Task-257C handoff or a specialized Task-252 transaction. These choices
also determine the later Task-252 occurrence test oracle and forward-binding
admission, so they are not merely an interchangeable split of an otherwise
frozen intermediate API.

The missing derived owner/profile is `design_drift`, and the absent exact
checker occurrence/binding oracle is a `test_gap`; checker source becomes
`source_drift` only after that profile is frozen. Creating a `BindingEnv`,
`BindingId`, Task-252 row, or capture table under an unselected profile, or
reusing the exact-F5 C4A/C4B API, would be a `boundary_violation`. Actual
capture, generated-core parameters, type/sethood, request/result, verdict,
diagnostic, production installation, runner activation, coverage credit, and
Task 277B all remain not ready and zero-credit.

Resumption requires a human owner/oracle decision that freezes: the sole owner
and consumer; exact context graph/ranges/order; one-versus-two binding rows and
the status of inner `y`; the outer `x` checker identity and visibility; mapper
lookup context, ordinal, and required result; the Task-252 forward-binding
admission and exact one-row occurrence/reference oracle (or its explicit
continued exclusion); validation/error precedence; and the same semantic,
installation, route, and credit prohibitions. No specification, `.miz`,
expectation, trace, active behavior, or semantic/coverage credit changes in this
closeout.
