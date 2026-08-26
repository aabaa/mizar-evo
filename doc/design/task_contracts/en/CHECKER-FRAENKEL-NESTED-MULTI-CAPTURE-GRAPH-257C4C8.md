# Task CHECKER-FRAENKEL-NESTED-MULTI-CAPTURE-GRAPH-257C4C8: Normalized nested Fraenkel capture graph

> Canonical language: English. Japanese companion:
> [../ja/CHECKER-FRAENKEL-NESTED-MULTI-CAPTURE-GRAPH-257C4C8.md](../ja/CHECKER-FRAENKEL-NESTED-MULTI-CAPTURE-GRAPH-257C4C8.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index). Durable owner
sections are checker
[source formula composition](../../mizar-checker/en/source_formula_composition.md#task-257c4c8-normalized-multi-capture-graph)
and the private test
[harness](../../mizar-test/en/harness.md#checker-task-257c4c8-private-normalized-capture-graph-probe).

## Status, decision, and purpose

**Status:** complete. Implementation commit
`c7595b60e7784728967cfbac9b02522f7290c942`, clean postcommit proof, and fresh
successor inventory are closed below. This postimplementation documentation
closure repairs stale lifecycle wording only.

After clean resolver commit `a710b4f1d99fd2efea36aecf9c2b00cf81437c57`,
fresh independent checker, Core/destination, and oracle inventories agreed that
the later owner and boundary were unique but two normalized representations
remained possible. The user accepted the parent recommendation and thereby
freezes this task's exact derived design decision:

- checker Task-257C `source_formula_composition` is the sole owner;
- the destination is one standalone immutable, syntax-free, Core-ID-free
  handoff, not a `TypedAst` or `ResolvedTypedAst` slot;
- five normalized dense tables contain exactly `3` generators, `1` mapper,
  `0` predicates, `2` distinct captures, and `2` capture occurrences;
- inner `z` remains a generator row but never a capture; outer `x` and `y`
  each form one distinct capture and one mapper occurrence;
- the retained C4C8R resolver snapshot is the only identity authority;
- private authenticated declaration/source order is a deterministic transport
  convention, not a language result or sidecar oracle; and
- validation fails in dependency, cardinality, layout, provenance, capture
  identity, then occurrence precedence.

This closes the remaining checker `design_drift`, bounded `source_drift`, and
the checker/private-fixture `test_gap`. Independent implementation and
test-sufficiency reviews reported **NO FINDINGS** after the implementation
repair. Independent source/documentation/API and bilingual/boundary
finding-specific re-reviews also reported **NO FINDINGS** after the
documentation repairs, and required broad verification passes. Independent
final-quality review reports **NO FINDINGS**, all nine hard gates pass, and the
valid uncapped score is `100/100`. There is no `spec_gap` or
`repo_metadata_conflict`.
Any capture semantics, C4C4 captured-state change,
second AST slot, Core identity/origin, active route, diagnostic, or Task-277B
credit is a `boundary_violation`.

## Authority and dependencies

Authority remains, in order:

1. canonical Chapter 4 [§4.6](../../../spec/en/04.variables_and_constants.md#46-scoping-and-shadowing)
   and Chapter 13 [§§13.4.3--13.4.4](../../../spec/en/13.term_expression.md#1343-multiple-generators)
   and [§13.8.6](../../../spec/en/13.term_expression.md#1386-set-expression-encoding);
2. exact existing
   [`pass_types_nested_comprehension_two_outer_generator_captures_001.miz`](../../../../tests/miz/pass/types/pass_types_nested_comprehension_two_outer_generator_captures_001.miz);
3. its unchanged inactive expectation and existing trace backlink;
4. completed [C4C5](CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-257C4C5.md),
   [C4C6](CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-INSTALLATION-257C4C6.md),
   [C4C7](TEST-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C7.md), and
   [C4C8R](RESOLVE-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C8R.md);
5. derived owner documents and current source inventory.

The exact C4C8R input contains binding IDs `0/1/2` for inner `z`, outer `x`,
and outer `y`, with segment/binder ranges `110..129`/`110..111`,
`144..163`/`144..145`, and `165..184`/`165..166`. Mapper uses `0/1` are
`x@98..99 -> binding 1` and `y@101..102 -> binding 2`, both `Mapper`, with
global/role ordinals `0/0` and `1/1`. The common mapper owner occupies
the exact `97..103` mapper subtree and the inner and outer comprehensions are
`95..131` and `93..186` in the frozen frontend preflight. The resolver
collection exposes their authenticated resolved-node identities but not those
owner ranges; this graph neither repeats nor reconstructs unavailable ranges.

Display spelling authenticates the already exact resolver dependency but is
never a checker join key. Resolver binding IDs, checker graph IDs, checker
binding IDs, and future `CoreVarId`s are separate domains and must not be
reinterpreted by numeric equality.

## Frozen public API

Add exactly these five dense IDs through the existing local `dense_id!` macro:

```rust
SourceNestedFraenkelCaptureGraphGeneratorId
SourceNestedFraenkelCaptureGraphMapperId
SourceNestedFraenkelCaptureGraphPredicateId
SourceNestedFraenkelCaptureGraphCaptureId
SourceNestedFraenkelCaptureGraphOccurrenceId
```

Each keeps private storage, derives the existing dense-ID trait set, and
exposes only `new(index: usize) -> Self` and `index(self) -> usize`.

Add exactly five immutable row/table pairs:

```rust
SourceNestedFraenkelCaptureGraphGenerator
SourceNestedFraenkelCaptureGraphGeneratorTable
SourceNestedFraenkelCaptureGraphMapper
SourceNestedFraenkelCaptureGraphMapperTable
SourceNestedFraenkelCaptureGraphPredicate
SourceNestedFraenkelCaptureGraphPredicateTable
SourceNestedFraenkelCaptureGraphCapture
SourceNestedFraenkelCaptureGraphCaptureTable
SourceNestedFraenkelCaptureGraphOccurrence
SourceNestedFraenkelCaptureGraphOccurrenceTable
```

Every table exposes only dense `get`, source-ordered `iter`, `len`, and
`is_empty`. Private source/declaration/role ordinals are validated but have no
public getter and must not become semantic order.

The generator row exposes only:

```rust
resolver_binding() -> FraenkelGeneratorVariableBindingId
definition_block() -> ResolvedNodeId
functor_definition() -> ResolvedNodeId
comprehension() -> ResolvedNodeId
segment() -> ResolvedNodeId
binder() -> ResolvedNodeId
segment_range() -> SourceRange
binder_range() -> SourceRange
```

The mapper and predicate rows each expose only `definition_block()`,
`functor_definition()`, `comprehension()`, and `owner()` as `ResolvedNodeId`.
The exact predicate table is empty; its public shape records only normalized
owner provenance and does not admit or define predicate semantics.

The capture row exposes only:

```rust
generator() -> SourceNestedFraenkelCaptureGraphGeneratorId
resolver_binding() -> FraenkelGeneratorVariableBindingId
mapper() -> SourceNestedFraenkelCaptureGraphMapperId
owner_context() -> ResolvedNodeId
```

The occurrence row exposes only:

```rust
mapper() -> SourceNestedFraenkelCaptureGraphMapperId
capture() -> SourceNestedFraenkelCaptureGraphCaptureId
resolver_use_index() -> usize
resolver_binding() -> FraenkelGeneratorVariableBindingId
comprehension() -> ResolvedNodeId
role_owner() -> ResolvedNodeId
term_reference() -> ResolvedNodeId
identifier() -> ResolvedNodeId
role() -> FraenkelGeneratorVariableUseRole
identifier_range() -> SourceRange
```

Add exactly this handoff/error/producer family:

```rust
SourceNestedFraenkelCaptureGraphHandoff
#[non_exhaustive] SourceNestedFraenkelCaptureGraphError
SourceNestedFraenkelCaptureGraphProducer
```

The handoff exposes only:

```rust
source_id() -> SourceId
module_id() -> &ModuleId
resolver_summary() -> &str
generators() -> &SourceNestedFraenkelCaptureGraphGeneratorTable
mappers() -> &SourceNestedFraenkelCaptureGraphMapperTable
predicates() -> &SourceNestedFraenkelCaptureGraphPredicateTable
captures() -> &SourceNestedFraenkelCaptureGraphCaptureTable
occurrences() -> &SourceNestedFraenkelCaptureGraphOccurrenceTable
debug_text() -> String
```

It retains one private version/domain-tagged clone of the complete resolver
collection. The only producer signature is:

```rust
SourceNestedFraenkelCaptureGraphProducer::build(
    resolver: &FraenkelGeneratorVariableSourceCollection,
) -> Result<SourceNestedFraenkelCaptureGraphHandoff,
          SourceNestedFraenkelCaptureGraphError>
```

There is no public/raw/unchecked row, table, dependency, or handoff
constructor; mutable accessor; caller DTO; profile selector; `Default`;
conversion; adapter; installer; or AST/Core route. A crate-private complete
validator may support a later checker-owned consumer without exposing state.

The exact debug grammar is:

```text
source-nested-fraenkel-capture-graph-v1|module=<package>.<path>|generators=3|mappers=1|predicates=0|captures=2|occurrences=2
```

## Exact graph and private order

The tables contain exactly:

- generators `0/1/2`: retained resolver bindings `z/x/y` in authenticated
  declaration/source order, with every resolver node and range copied exactly;
- mapper `0`: the common inner-comprehension `Mapper` role owner shared by
  resolver uses `0/1`;
- predicates: empty;
- captures `0/1`: generator rows `1/2`, resolver bindings `1/2`, mapper `0`,
  and the inner comprehension as owner context;
- occurrences `0/1`: resolver uses `0/1` associated respectively with capture
  rows `0/1`, preserving every mapper/link node, role, binding, and range.

The graph never derives a graph ID from an unrelated numeric ID. It selects
the resolver row by resolver identity and separately records the graph table
ID. Multiple future occurrences may point to one distinct capture row, but
this exact-only producer admits no profile other than the frozen `3/2`
dependency and does not generalize language behavior.

## Default-deny oracle and error precedence

The error variants and exact display strings are:

```rust
InvalidDependency
// "nested Fraenkel capture graph dependency is invalid"
InvalidCardinality
// "nested Fraenkel capture graph cardinality is invalid"
InvalidLayout
// "nested Fraenkel capture graph layout is invalid"
InvalidProvenance
// "nested Fraenkel capture graph provenance is invalid"
InvalidCaptureIdentity {
    capture: SourceNestedFraenkelCaptureGraphCaptureId,
}
// "nested Fraenkel capture graph identity <id> is invalid"
InvalidOccurrence {
    occurrence: SourceNestedFraenkelCaptureGraphOccurrenceId,
}
// "nested Fraenkel capture graph occurrence <id> is invalid"
```

Validation precedence is exact:

1. environment, snapshot version/domain/summary, and the complete exact C4C8R
   resolver relation; any failure is `InvalidDependency`;
2. exact `3/1/0/2/2` cardinalities; any failure is `InvalidCardinality`;
3. dense IDs and private ordinals/order; any failure is `InvalidLayout`;
4. every resolved owner/node identity, every dependency-exposed generator
   segment/binder and occurrence-identifier range, source/module, and
   dependency association; occurrence node/range provenance is authenticated
   here before capture/occurrence validation; any failure is
   `InvalidProvenance`;
5. each capture's graph-generator/resolver-binding/mapper/inner-owner identity,
   reporting the lowest invalid capture; and
6. each occurrence's mapper/capture/resolver-use/binding/role association,
   with defensive node/range rechecks permitted, reporting the lowest invalid
   occurrence.

Missing, extra, duplicate, reordered, stale, foreign, recovered, partial,
mismatched, numeric-ID-substituted, or display-name-joined state fails
atomically. The producer and validator never sort, repair, infer, merge,
unchecked-deduplicate, mutate the resolver collection, or publish a partial
handoff.

## Tests, files, and documentation ownership

Implementation changes exactly two Rust paths:

1. `crates/mizar-checker/src/source_formula_composition.rs`;
2. `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs`.

Checker adds exactly four tests:

1. `task257c4c8_builds_exact_normalized_capture_graph`;
2. `task257c4c8_rejects_dependency_cardinality_layout_and_provenance`;
3. `task257c4c8_rejects_capture_identity_and_occurrence_in_precedence`;
4. `task257c4c8_replays_immutably_and_rejects_near_miss_profiles`.

The existing private mizar-test leaf adds exactly
`task257c4c8_real_imported_fixture_builds_exact_normalized_capture_graph`. It
runs the unchanged C4C7 source through the existing frontend and resolver,
calls the public graph producer directly, and asserts only diagnostics-free and
unrecovered admission, exact `3/1/0/2/2`, resolver-identity links, private
deterministic iteration, local-`z` exclusion, replay, and unchanged empty
import augmentation. It is not registered in active dispatch.

The documentation prerequisite changes exactly 21 paths: this pair; paired
checker plan, `source_formula_composition`, TODO, source-spec, and bilingual
records; paired mizar-test plan, harness, TODO, and bilingual records; and the
central coverage audit. Each owner records only its local state and links this
contract. Completion may update exactly 15 paths: the two Rust paths; this
pair; paired checker `source_formula_composition`, TODO, source-spec, and
module-boundary records; paired mizar-test TODO; and the central audit.

The central audit receives one zero-credit mapping. Chapter 13 remains
`partial`; trace requirement/status/backlinks, `.miz`, expectation, active
route, and diagnostic counts do not change.

## Baseline and expected impact

Clean baseline HEAD is `a710b4f1d99fd2efea36aecf9c2b00cf81437c57`,
origin/main is `ffc882675141a3e25bc78a47affc018bfe3685e1`, divergence is
`0/8`, and protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` remains untouched.

| Path | Baseline lines / bytes | Baseline SHA-256 |
|---|---:|---|
| `crates/mizar-checker/src/source_formula_composition.rs` | `10342 / 404395` | `dd27218581ebe6c252da33f6feb23480403afa88858de874970d88a9d1573d44` |
| private mizar-test leaf | `704 / 27913` | `6a1717fec263e79d9295813b413d1ec323c3291297f9ee04e0bc7c8e59e2e754` |

Checker/mizar-test raw library tests project `572 -> 576` and `624 -> 625`.
Their sorted baseline list hashes are
`ac213696433d40a0649c3f6ca4eb7449ce7d053a40a7573209ef5c0af9716940`
and
`21196d1cb959c5b6bd7b38f19efb83d334978ec7f1d0c99e35da19cec8afe385`.
The current library counts are checker `576` and mizar-test `625`.
Paired contract trees project `104/104 -> 105/105`. Checker production remains
32 paths; its path-list hash remains
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787`.

Protected C4C7 source, sidecar, and trace hashes remain respectively
`b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`,
`277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`,
and `17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`.
`source_term.rs`, `typed_ast.rs`, `resolved_typed_ast.rs`, C4C4 captured state,
Core, `GeneratedOrigin`, diagnostics, active routes, and Task 277B are
protected.

## Implementation completion evidence

The documentation prerequisite is commit
`481a599877803e855307381901b82ae38365ce4a`. Current measured source values
are checker `12132 / 472546`, SHA-256
`e7242ebf7344b1e89646fefe2dd9e1ad41d40be22b526c872327540ba7abad12`, and the
private mizar-test leaf `816 / 32987`, SHA-256
`14f1db22b0d4a45cad31db5a1e11f4c28b89e0cab1047b6f8fd4982a8e7d8041`.
The focused four checker tests plus one imported-fixture probe pass. Checker
and mizar-test libraries pass `576/576` and `625/625`; their final sorted raw
test-list SHA-256 values are respectively
`20a2a07a078580b3253a0fbcb5ac8387c42df19fe568d1e5a97b3a709a7bdcd3`
and
`6679e5558b1a8884baacaa4c0bb1d6c000d7352002b98c5ff33642034f68f49e`.
Checker production measures `32` paths / `199351` lines, with unchanged path
SHA-256
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787`
and final content-manifest SHA-256
`9d41d59f42e05c46b084bb26e75b53091e8a4f59cbd83880cca4a91b46f8e5e0`.
The paired contract trees measure `105/105`, and all three protected C4C7
hashes match the frozen values above.

Independent implementation, test-sufficiency, source/documentation/API, and
bilingual/boundary reviews report **NO FINDINGS** after the two implementation
repairs and the exact source-spec inventory/status repair. Compatibility,
both package libraries and lint suites, metadata, formatting, offline Cargo
metadata, warnings-denied all-target/all-feature workspace Clippy, full
all-feature workspace tests and doctests, diff, scope, count, hash, and
protected-surface checks pass. An extra `cargo test --workspace --all-targets
--all-features` run completed all ordinary test targets without failure but its
long-running Criterion performance measurement was intentionally stopped; it
is not substituted for the passing required full-workspace test gate.
Independent final-quality review reports **NO FINDINGS**. All nine autonomous
crate exit gates pass with a valid uncapped `100/100`: specification
completeness `20/20`, test contract and coverage `20/20`, traceability `15/15`,
implementation correctness `15/15`, design/source synchronization `10/10`,
boundary discipline `10/10`, verification health `5/5`, and handoff quality
`5/5`. Exact 15-path staging and cached-diff review passed, and task-only
implementation commit `c7595b60e7784728967cfbac9b02522f7290c942` closed the
source change.

## Postimplementation closure and fresh successor inventory

Immediately after the implementation commit the worktree was clean, HEAD was
`c7595b60e7784728967cfbac9b02522f7290c942`, origin/main was
`481a599877803e855307381901b82ae38365ce4a`, and divergence was `0/1`. The
protected stash remained
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`; the protected C4C7 source,
sidecar, and trace retained the hashes frozen above. The implementation commit
contains exactly 15 paths with sorted path-list SHA-256
`23c102b51678e49f0fd749f9689d4e12dbf85d06dc2cfa870c22623e20b2b541`.

Fresh independent checker, Core, and oracle inventories select no uniquely
ready semantic successor. They confirm the fixed direction: Core Task 33 owns
context/item/binder identity, provenance, snapshot-local `CoreVarId`
allocation, and the durable checker-graph association; Core Task 34 owns
type/evidence/coercion/view lowering; Core Task 35 consumes the Task-33
association and later owns term/formula/Fraenkel `GeneratedOrigin` lowering.
They do not uniquely determine the exact Task-33 C4C8 handoff/API, allocator
and owner mapping, captured parameter/argument surface and order, or the
`GeneratedOrigin` key/functor/source corruption oracle. Reusing Typed/Resolved
slots or numeric IDs, or making Core Task 35 allocate or infer the association,
remains forbidden.

The candidates were: (1) first freeze the generic Core-33 context/item/binder
base contract with a reserved standalone immutable C4C8 association seam;
(2) freeze a C4C8-specific private Core-33 association directly; or (3) extend
the public `CoreContextInput`/`CoreContext` surface. Candidate 1 is recommended
because the prerequisite allocator and owner map do not yet exist and it
preserves a zero-semantic, default-deny boundary. Candidates 2 and 3 require a
separate explicit API decision. The user accepted candidate 1 in the current
continuation, selecting only the documentation-only Core-33 prerequisite
direction; its exact task identity, API, files, and oracle remain subject to a
fresh inventory and separately frozen contract. No successor task ID, API,
field, adapter, installer, route, semantics, or credit is created here. This is
`design_drift` plus a future source-derived `test_gap`, not authority to invent
semantic parameter order. Task 277B remains not ready and receives zero credit.

The stale precommit wording in the paired checker plan, mizar-test harness,
and TODO records is `design_drift`. This paired contract, those four EN/JA
owner/TODO pairs, and no other files form the exact 10-document closure scope.
The closure changes no specification, test intent, traceability, expectation,
source, public API, route, diagnostic, protected hash, or coverage credit.

## Core boundary, reviews, verification, and exit

Core Task 33 later owns context/item/binder identity, provenance, fresh
snapshot-local `CoreVarId` allocation, and its durable association to this
checker graph. Core Task 34 owns type/evidence/coercion/view lowering. Core Task
35 consumes the Task-33 association and owns term/formula and Fraenkel
generated-origin lowering after Task 34; it must not allocate or infer the
association. Generator domain operands remain separate from captured
parameter/argument subvectors. Only those captured subvectors may later form a
positional one-to-one join in this graph's private order. This task creates no
Core input, output, ID, parameter, argument, functor, origin, or use record.

Before source edits, independent specification/equivalence and bilingual/
boundary/API reviews must report **NO FINDINGS**. After implementation,
independent test-sufficiency, implementation, source/documentation/API, and
final-quality reviews must report **NO FINDINGS**, with finding-specific
re-review after every repair.

Required verification is the five exact C4C8 tests, C4C2--C4C8R and C4C5/C4C6
compatibility, checker and mizar-test libraries, both lint-policy suites,
metadata tests, formatting, offline Cargo metadata, full-workspace all-target/
all-feature warnings-denied Clippy, full workspace tests, `git diff --check`,
exact counts/hashes, protected surfaces, exact staging, and task-only commits.
All `9/9` implementation hard gates passed with valid uncapped `100/100`; the
clean postcommit proof and fresh same-milestone inventory above close this
task. Actual capture semantics, Typed/Resolved installation, Core Tasks
33--35, GeneratedOrigin, active execution, and Task 277B remain deferred.
