# Task RESOLVE-FRAENKEL-CONTAINING-OWNER-33R: Exact containing-functor owner receipt

> Canonical language: English. Japanese companion:
> [../ja/RESOLVE-FRAENKEL-CONTAINING-OWNER-33R.md](../ja/RESOLVE-FRAENKEL-CONTAINING-OWNER-33R.md).

Owning plans: [mizar-resolve](../../mizar-resolve/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index). Durable
owner sections: resolver [names](../../mizar-resolve/en/names.md#resolver-task-33r-exact-surface-fingerprint),
resolver [symbols](../../mizar-resolve/en/symbols.md#resolver-task-33r-exact-containing-functor-owner-receipt),
and test [harness](../../mizar-test/en/harness.md#resolver-task-33r-private-containing-owner-probe).

## Status, decision, and readiness

**Status:** precommit implementation complete; exact staging and task-only commit pending.

This is the dependency-minimal zero-semantic successor to completed C4C8 and
the documentation-only Core-33P boundary. Fresh inventory proves that only
resolver symbol collection simultaneously owns the validated declaration
shell, its `ResolvedNodeId`, the final `SymbolId`, the matching `DefinitionId`,
the `SourceContributionId`, and `SemanticOrigin`. Checker, Typed/Resolved, and
Core cannot recreate that relation from names, ranges, or numeric ids.

Repository authority did not previously choose owner cardinality or a public
API. The user has adopted the reviewed recommendation, and this request is the
decision authority for the following narrow choice: the exact C4C8 profile has
exactly one containing functor owner; resolver publishes one opaque immutable
receipt at its existing symbol/definition allocation boundary; every mismatch
fails closed. This decision does not generalize Fraenkel semantics, activate a
route, or select a Core destination.

The absent authenticated containing-owner link is `design_drift`; missing
freshness, owner-association, corruption, and real-fixture probes are a
`test_gap`. There is no blocking `spec_gap` or `repo_metadata_conflict`.

## Authority, dependencies, and fixed meaning

Authority remains, in order: `doc/spec/en/`; the exact existing C4C7 `.miz`;
`tests/coverage/spec_trace.toml`; its unchanged expectation; `doc/design/`;
then non-normative source inventory. Required completed dependencies are C4C5
(`72662d38`), C4C7 (`3d28af5f`), C4C8R (`a710b4f1`), C4C8 plus closure
(`c7595b60`, `c5792708`), and Core-33P (`332d752c`).

The fixed semantic meaning is unchanged: inner mapper `x` and `y` refer to the
outer generator binding identities, inner generator `z` is local and is not
captured, and association never uses display spelling. C4C4 remains by-value
with an empty `captured` vector. Resolver, checker, and Core numeric ids remain
separate domains. C4C5 remains a separate one-capture checker receipt and is
not inferred to own or identify the C4C8 containing functor.

## Frozen public API and ownership

`FraenkelGeneratorVariableSourceCollection` gains one private exact
`surface_fingerprint: String`, set only to the complete deterministic
`SurfaceAst::snapshot_text()` used by the collector, plus immutable
`surface_fingerprint(&self) -> &str`. Its existing constructor path, table
types, row order, ids, and `debug_text()` grammar remain unchanged. Equality
includes the new fingerprint, so equal count-only summaries do not authenticate
stale same-`SourceId` snapshots.

The existing `symbols` module owns exactly these new public items:

- `SourceNestedFraenkelFunctorOwnerHandoff`;
- non-exhaustive `SourceNestedFraenkelFunctorOwnerError`;
- `SourceNestedFraenkelFunctorOwnerProducer`.

Only the producer constructs the handoff. Its frozen entry point is:

```rust
pub fn build(
    ast: &SurfaceAst,
    module: &ModuleId,
    resolved: &SurfaceResolvedArena,
    resolver: &FraenkelGeneratorVariableSourceCollection,
) -> Result<SourceNestedFraenkelFunctorOwnerHandoff,
            SourceNestedFraenkelFunctorOwnerError>
```

The handoff is `#[derive(Clone, PartialEq, Eq)]`, has private fields, no
builder/mutator/default/unchecked constructor, and freezes this complete
immutable API:

```rust
pub const fn source_id(&self) -> SourceId
pub const fn module_id(&self) -> &ModuleId
pub fn surface_fingerprint(&self) -> &str
pub const fn definition_block(&self) -> ResolvedNodeId
pub const fn functor_definition(&self) -> ResolvedNodeId
pub const fn declaration_shell(&self) -> DeclarationShellId
pub const fn symbol(&self) -> &SymbolId
pub const fn definition(&self) -> DefinitionId
pub const fn contribution(&self) -> SourceContributionId
pub const fn origin(&self) -> &SemanticOrigin
pub fn debug_text(&self) -> String
pub fn validate_complete(
    &self,
) -> Result<(), SourceNestedFraenkelFunctorOwnerError>
pub fn validate_resolver_collection(
    &self,
    resolver: &FraenkelGeneratorVariableSourceCollection,
) -> Result<(), SourceNestedFraenkelFunctorOwnerError>
```

The resolver-collection oracle first runs the complete oracle and then
requires exact retained-collection equality for the later checker receipt
boundary.

Private dependencies retain a version/domain tag, cloned surface AST,
structural resolved arena, exact resolver collection, declaration shell set,
parser-backed projections, and symbol collection result. The producer derives
all of them internally. A caller cannot supply or forge a shell, projection,
symbol environment, `SymbolId`, `DefinitionId`, contribution, or origin.

The allocation association is captured in flight, not reconstructed from the
finished environment. Existing public `SymbolCollector::collect()` delegates
to a new private collection path with no target and remains behaviorally
unchanged. The producer uses that private path with the already authenticated
target `DeclarationShellId`; it records the exact `CollectedProjection::new`
symbol/origin/contribution and the `DefinitionId` returned while that same row
is inserted. A private allocation row is returned beside the unchanged
`SymbolCollectionResult`. Complete validation reruns this same targeted
collection and compares the allocation row and final indexes. No post-hoc
name/range search may create the association.

## Frozen association and default-deny oracle

Validation precedence is fixed:

1. `InvalidDependency`: version/domain, source/module, complete structural
   arena, exact surface fingerprint, fresh resolver recollection, or retained
   dependency mismatch;
2. `InvalidResolverProfile`: anything other than the exact C4C8R `3` binding /
   `2` mapper-use identity graph with dense source/role order, local inner
   binding `0`, captures of outer bindings `1/2`, one inner mapper owner, and
   one common definition-block/functor identity;
3. `InvalidOwnerCardinality`: not exactly one functor declaration shell and
   one matching parser-backed projection for that resolved functor node;
4. `InvalidOwnerProvenance`: wrong parent definition block, kind, module,
   recovery state, or resolved shell-to-node projection;
5. `InvalidSymbolAssociation`: diagnostics, missing/duplicate symbol entry,
   non-functor kind, wrong contribution/origin, recovery/conflict, or a symbol
   not produced through the canonical `CollectedProjection::new` allocation
   path;
6. `InvalidDefinitionAssociation`: missing/duplicate definition, wrong
   `DefinitionKind::Functor`, symbol, contribution, origin, or conflict state;
7. `InvalidAssociation`: any exposed handoff field differs from the freshly
   derived exact row.

All seven variants are fieldless. The error type is
`#[derive(Debug, Clone, Copy, PartialEq, Eq)]`, `#[non_exhaustive]`, implements
`std::fmt::Display` and `std::error::Error`, and has these exact display strings
in the same order:

```text
nested Fraenkel functor owner dependency is invalid
nested Fraenkel functor owner resolver profile is invalid
nested Fraenkel functor owner cardinality is invalid
nested Fraenkel functor owner provenance is invalid
nested Fraenkel functor owner symbol association is invalid
nested Fraenkel functor owner definition association is invalid
nested Fraenkel functor owner association is invalid
```

`SourceNestedFraenkelFunctorOwnerProducer` is
`#[derive(Debug, Clone, Copy)]`.

The definition-block and functor ids are converted only through
`SurfaceResolvedArena::resolved_node_for(DeclarationShell::node_id())`; no
numeric reinterpretation is permitted. Final symbol association uses the
existing `CollectedProjection::new` identity construction at the collection
boundary, never spelling or range lookup. Display spelling and token text may
be retained in the exact public surface fingerprint or rendered for debugging,
but neither is an association, join, or owner-admission key.

Missing, extra, duplicate, reordered, stale, foreign-source, foreign-module,
recovered, partial, mismatched, orphan, or retained-row-corrupt input is
rejected. The implementation must not sort for repair, infer an owner, join by
display name, reinterpret ids, recover a partial receipt, or admit unchecked
input.

## Exact scope, tests, and protected surfaces

This logical task may change exactly these 23 paths:

1. this paired EN/JA contract;
2. paired `doc/design/mizar-resolve/*/00.crate_plan.md` Task Index rows;
3. paired `doc/design/mizar-test/*/00.crate_plan.md` Task Index rows;
4. paired resolver `names.md` owner sections;
5. paired resolver `symbols.md` owner sections;
6. paired mizar-test `harness.md` private-probe sections;
7. `doc/design/spec_coverage_audit.md` zero-credit owner mapping;
8. `crates/mizar-resolve/src/names.rs`;
9. `crates/mizar-resolve/src/names/tests.rs`;
10. `crates/mizar-resolve/src/symbols.rs`;
11. `crates/mizar-resolve/src/symbols/tests.rs`;
12. `crates/mizar-resolve/tests/lint_policy.rs`; and
13. the existing private
    `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs`;
14. paired resolver
    `bilingual_documentation_synchronization.md` audit files; and
15. paired mizar-test `bilingual_sync_audit.md` audit files.

The resolver adds exactly seven tests:

- `task33r_surface_fingerprint_binds_exact_ast_snapshot`;
- `task33r_surface_fingerprint_distinguishes_stale_same_source_ast`;
- `task33r_builds_exact_containing_functor_owner`;
- `task33r_rejects_dependency_and_resolver_profile_mismatch`;
- `task33r_rejects_owner_cardinality_and_provenance`;
- `task33r_rejects_symbol_definition_and_retained_association_corruption`;
- `task33r_enforces_default_deny_precedence_and_replay`.

The existing private mizar-test leaf adds exactly
`task33r_real_fixture_links_capture_graph_to_exact_functor_owner`. It uses the
unchanged real C4C7 source, builds the resolver collection, C4C8 graph, and new
owner receipt, validates exact resolver equality, and asserts that the receipt
links the graph's common resolved owner to the final functor symbol/definition
without mutating the graph or import augmentation. It remains unregistered and
library-test-only.

No checker source, Core source, Typed/Resolved field, installer, active runner,
diagnostic, Cargo manifest, `doc/spec`, existing `.miz`, expectation, or trace
metadata change is authorized. No C4C4 captured state, generated parameter/
argument, `GeneratedOrigin`, semantic result, coverage credit, or Task-277B
readiness is created.

## Baseline and expected impact

Entry HEAD and origin/main are both
`332d752c03292a1a100472322ce86e99080ce1bd`; divergence is `0/0`, worktree and
index are clean, and protected stash is
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`. Contract trees are `106/106` and
become `107/107`.

| Rust path | Baseline lines / bytes | Baseline SHA-256 |
|---|---:|---|
| `names.rs` | `4415 / 140538` | `663ec040a0b9525cb79b532fe7ae6a548f67acb7510b8713df3b0cfe2b8d6166` |
| `names/tests.rs` | `4798 / 153865` | `d53afc1d148b3ab55bdbf97a04d11f78f4fe454a0caf6ca43f8ea72d6a55c504` |
| `symbols.rs` | `2088 / 70944` | `ee06c915ce23fae3084aeda05385b2a1ee75142b8bf89465b070a17b6b3ca7b2` |
| `symbols/tests.rs` | `3212 / 97052` | `c8657275ab22d3c7fe2bbe4134eb99d1a6e1cf88b9cd4eb86984a0abf7670fb7` |
| resolver lint policy | `1037 / 31476` | `1a84ba67b715b8df752accd18895fc89a8e727769061a89570b2b4fe15d1182d` |
| private mizar-test leaf | `816 / 32987` | `14f1db22b0d4a45cad31db5a1e11f4c28b89e0cab1047b6f8fd4982a8e7d8041` |

Resolver library tests project `164 -> 171`; mizar-test library tests project
`625 -> 626`. Sorted baseline raw-list hashes are resolver
`a01c16a16aead9868d30257e358a4e742dd7633a8da4f61c864d9197d9c1f1c8`
and mizar-test
`602a80e3a0ad30084154d2f857bd00251494ad40a79549aca0a76db9b9cde711`.

Protected C4C7 source, expectation, and trace hashes are respectively
`b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`,
`277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`,
and `17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`.

## Reviews, verification, exit, and handoff

Before Rust edits, independent specification/equivalence and bilingual/
boundary reviews must report **NO FINDINGS**. After implementation,
independent test-sufficiency, implementation, source/documentation/API, and
final-quality reviews must report **NO FINDINGS**, with finding-specific
re-review after every repair.

Required verification includes all seven resolver tests, the private real-
fixture probe, C4C8R/C4C8 compatibility tests, resolver and mizar-test library
and lint suites, mizar-test metadata, parser set-comprehension coverage,
formatting, offline Cargo metadata, workspace all-target/all-feature warnings-
denied Clippy, full workspace all-feature tests, `git diff --check`, exact
scope/count/hash/link checks, and protected invariance.

Exit requires all autonomous hard gates `9/9`, a valid score at least `90/100`,
exact task-only staging and commit, clean postcommit proof, and fresh successor
inventory. The next candidate is a checker-owned opaque receipt that pairs the
unchanged C4C8 graph with this resolver owner receipt; it may be frozen only if
fresh inventory uniquely fixes its association and complete oracle. Core33
installation, Core34/35 transport, parameter order, GeneratedOrigin, actual
semantics, active route, and Task277B remain deferred.

## Precommit implementation completion evidence

Implementation changes exactly the frozen 23 paths. Paired contract trees are
`107/107`; resolver and mizar-test library inventories are exactly `171` and
`626` tests. Their final sorted raw-list SHA-256 values are respectively
`1e4b48bf53e4ad6ead624ac40d6fe8e6aeef90166c77fd4974b9c849c955d5ba`
and
`e54d5c97f46e65d4657d5e99b7efa609cd39a096020c4339b512fdbf039b0694`.

Final Rust measurements are:

| Path | Lines / bytes | SHA-256 |
|---|---:|---|
| `crates/mizar-resolve/src/names.rs` | `4423 / 140806` | `5ef275703e6cb44fb5f54e89652ddae3e349825cca5b960f4549f348c69f27fe` |
| `crates/mizar-resolve/src/names/tests.rs` | `4853 / 156132` | `674b292f71e75a6b50920baa855cd8bb14602a378afe5da99d833e61d8fe0cd0` |
| `crates/mizar-resolve/src/symbols.rs` | `2634 / 92065` | `2ca49b82cc4a2d69c2e325482aab4eac56af31c446e73972dc2a7da71eaf45ee` |
| `crates/mizar-resolve/src/symbols/tests.rs` | `4015 / 123699` | `7514ecd91680e79fb356f2c61fa93334d6214ab879339fb9eb61ef2a28defb50` |
| `crates/mizar-resolve/tests/lint_policy.rs` | `1042 / 31642` | `d8bdda347793d0708897463152b3dbba7450dcb6a1b50389185399ffaefedb3e` |
| private mizar-test leaf | `932 / 37659` | `1b881477a9a17f5bab425e03d8ea59c656c1f28f56bdc57e211652b30a00d2c4` |

Independent pre-source specification/equivalence and bilingual/boundary
reviews, followed by post-source test-sufficiency, implementation,
source/documentation/API, and bilingual/boundary reviews, report **NO
FINDINGS** after finding-specific repairs. The independent final-quality
review also reports **NO FINDINGS**, all `9/9` hard gates pass, and the valid
uncapped score is `100/100` (`20/20`, `20/20`, `15/15`, `15/15`, `10/10`,
`10/10`, `5/5`, `5/5`).

All seven focused resolver tests, the one private real-fixture probe, C4C8R
`4/4`, checker C4C8 `4/4`, runner C4C8 `2/2`, resolver `171/171` and lint
`11/11`, mizar-test `626/626`, lint `15/15`, and metadata `137/137`, parser
set-comprehension coverage, formatting, offline Cargo metadata, warnings-denied
workspace all-target/all-feature Clippy, full all-feature workspace tests and
doctests, and `git diff --check` pass. Exact scope/count/hash/link checks pass.

The protected C4C7 source, expectation, and trace reproduce all three frozen
hashes. No specification, existing corpus/expectation/trace, checker/Core,
Typed/Resolved, active route, diagnostic, C4C4 capture, semantic or coverage
state changed. Task277B remains not-ready with zero credit. Exact staging,
task-only commit, clean postcommit proof, and fresh successor inventory remain
the exit operations.
