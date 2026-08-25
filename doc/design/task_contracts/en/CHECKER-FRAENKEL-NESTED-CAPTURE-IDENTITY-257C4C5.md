# Task CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-257C4C5: Nested Fraenkel Capture-Identity Receipt

> Canonical language: English. Japanese companion:
> [../ja/CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-257C4C5.md](../ja/CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-257C4C5.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).
Durable owner sections: checker
[source formula composition](../../mizar-checker/en/source_formula_composition.md#task-257c4c5-nested-fraenkel-capture-identity-receipt)
and test
[harness](../../mizar-test/en/harness.md#checker-task-257c4c5-private-capture-identity-receipt-probe).

## Status, decision, and purpose

**Status:** implementation, reviews, and repository verification complete;
exact staging, the task-only implementation commit, and its immediate clean
postcommit proof remain.

Independent specification/equivalence, bilingual/boundary, implementation,
test-sufficiency, source/documentation/API, and final-quality reviews report
no findings after their recorded repairs.

The human decision selects the existing Task-257C
`source_formula_composition` family as the sole owner of the first
capture-identity receipt after completed C4C4. This task transports one exact
resolved identity association and nothing semantic. It neither decides nor
installs captured variables.

The selected shape is deliberately narrow:

- the complete C4C4 handoff is consumed and retained by value;
- one immutable row associates the exact inner comprehension, mapper primary,
  mapper reference, projection-local checker binding, resolver use, and
  resolver binding;
- the row exists only for the exact C4C4 profile and is ordered only by its
  exact source ordinal;
- the destination is a standalone Task-257C handoff, not `TypedAst`,
  `ResolvedTypedAst`, `CoreIr`, or `GeneratedOrigin`;
- corruption is rejected atomically, without sorting, inference, repair, or a
  caller-selected profile.

There is no `spec_gap`: the human-confirmed meaning and existing authority are
unchanged. The previously absent unique owner/API/oracle was `design_drift`,
the absent exact regression was a `test_gap`, and any capture installation,
semantic result, or Core lowering in this task is a `boundary_violation`.

## Authority and protected meaning

Authority remains, in order:

1. canonical [Chapter 13 §§13.4.2, 13.4.4, and 13.8.6](../../../spec/en/13.term_expression.md#134-set-expressions);
2. existing
   [`pass_types_nested_comprehension_outer_generator_capture_001.miz`](../../../../tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.miz);
3. its sole [trace row](../../../../tests/coverage/spec_trace.toml);
4. its inactive [expectation](../../../../tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.expect.toml);
5. completed [C4C2](RESOLVE-FRAENKEL-NESTED-CAPTURE-257C4C2.md),
   [C4C3](CHECKER-FRAENKEL-NESTED-BINDER-USE-257C4C3.md), and
   [C4C4](CHECKER-FRAENKEL-NESTED-MAPPER-PRIMARY-257C4C4.md), followed by
   derived owner documents and source inventory.

The frozen meaning is:

- inner mapper `x@94..95` refers to the resolved binding identity of outer
  generator `x@136..137`;
- inner generator `y@102..103` is local to the inner comprehension and is not
  captured;
- association is by resolved binding identity, never by display spelling or
  coincidence between numeric IDs from different domains;
- C4C4 keeps its outer-x projection by value and its sole
  `BindingEntry::captured` remains empty;
- Task 277B remains not-ready and receives zero execution or semantic credit.

The protected `.miz`, expectation, and trace SHA-256 values are respectively
`c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431`,
`9ed000a30c1d519bd665f338c636fb9e529e9848a285209bebe6728f19961b92`, and
`d4d817e83aac78d19e729702b26c62604fc57581eec18672a5c26ec44efe7a81`.
They must remain byte-identical.

## Sole owner and dependency boundary

`crates/mizar-checker/src/source_formula_composition.rs` is the sole
production owner. This is a Task-257C cross-family association, not a new
Task-252 occurrence owner and not Task-255 set-term ownership.

The sole lower dependency is one complete, internally valid
`SourceNestedFraenkelMapperPrimaryHandoff`. C4C4 adds one crate-private
`validate_complete()` seam that delegates to its existing complete validator;
it exposes no mutable state or new public getter. The C4C5 producer consumes
the dependency by value, validates it before reading any association, and
retains it immutably.

Task 255 is not a dependency. Its current admitted comprehension profiles do
not own this nested `Element of NAT` outer-to-inner resolved identity
relation. C4C5 must not widen Task-255 admission or synthesize a Task-255 term.

## Exact association and ordering

The table has exactly one row, ID `0`:

| Field | Exact value | Authenticated source |
|---|---:|---|
| owner context | checker `BindingContextId(2)` | C4C4 inner-comprehension context |
| owner range | `92..123` | C4C4 `SourceComprehension` owner |
| mapper term | `SourcePrimaryTermId(0)` | C4C4 mapper primary `x@94..95` |
| mapper reference | `SourcePrimaryTermReferenceId(0)` | C4C4 outer-x reference |
| projected binding | checker `BindingId(0)` | C4C4 outer-x by-value projection |
| resolver use index | `0` | retained C4C3 mapper use |
| resolver binding | `FraenkelGeneratorVariableBindingId(1)` | retained C4C3 outer generator identity |
| source ordinal | `0` | the exact C4C3 association order |

The identity ID and checker binding ID are local dense IDs and are not equal
to resolver binding ID `1`. The resolver binding object is the cross-domain
identity evidence. Spelling `x` is only an authenticated property of the
already validated dependency and is not the join key.

Exactly-one means: row `0` must exist, row `1` must not exist, the table length
must be one, and iteration must yield only `(0, row0)`. Missing, extra,
duplicated, or reordered state is invalid. `source_ordinal == 0` is the only
ordering claim. This task defines no generalized capture order and no Core
generated-parameter order.

## Frozen public API and destination

The existing public module gains exactly:

```rust
SourceNestedFraenkelCaptureIdentityId
SourceNestedFraenkelCaptureIdentity
SourceNestedFraenkelCaptureIdentityTable
SourceNestedFraenkelCaptureIdentityHandoff
#[non_exhaustive] SourceNestedFraenkelCaptureIdentityError
SourceNestedFraenkelCaptureIdentityProducer
```

The dense ID has private storage, derives `Debug`, `Clone`, `Copy`,
`PartialEq`, `Eq`, `PartialOrd`, `Ord`, and `Hash`, and exposes exactly:

```rust
SourceNestedFraenkelCaptureIdentityId::new(index: usize) -> Self
SourceNestedFraenkelCaptureIdentityId::index(self) -> usize
```

The row exposes only:

```rust
owner_context() -> BindingContextId
owner_range() -> SourceRange
mapper_term() -> SourcePrimaryTermId
mapper_reference() -> SourcePrimaryTermReferenceId
projected_binding() -> BindingId
resolver_use_index() -> usize
resolver_binding() -> FraenkelGeneratorVariableBindingId
source_ordinal() -> usize
```

The table exposes exactly:

```rust
get(
    &self,
    id: SourceNestedFraenkelCaptureIdentityId,
) -> Option<&SourceNestedFraenkelCaptureIdentity>
iter(
    &self,
) -> impl Iterator<
    Item = (
        SourceNestedFraenkelCaptureIdentityId,
        &SourceNestedFraenkelCaptureIdentity,
    ),
>
len(&self) -> usize
is_empty(&self) -> bool
```

Iteration is dense/source-ordinal order. The handoff exposes only:

```rust
source_id() -> SourceId
module_id() -> &ModuleId
dependency() -> &SourceNestedFraenkelMapperPrimaryHandoff
dependency_fingerprint() -> &str
identities() -> &SourceNestedFraenkelCaptureIdentityTable
debug_text() -> String
```

The producer signature is exactly:

```rust
SourceNestedFraenkelCaptureIdentityProducer::build(
    dependency: SourceNestedFraenkelMapperPrimaryHandoff,
) -> Result<SourceNestedFraenkelCaptureIdentityHandoff,
          SourceNestedFraenkelCaptureIdentityError>
```

Apart from the dense ID's frozen `new`, there is no public/raw/unchecked
row, table, or handoff constructor, mutable accessor, caller DTO, profile
selector, `Default`, installer, adapter, or conversion. The handoff is the
complete current destination. `TypedAst` and `ResolvedTypedAst` gain no field,
method, or installation surface.

The exact deterministic debug grammar is:

```text
source-nested-fraenkel-capture-identity-v1|module=<package>.<path>|identities=1|dependency-fingerprint=<Debug quoted complete C4C4 debug text>
```

## Default-deny oracle

The handoff has one private complete validator and one crate-private
`validate_complete()` boundary for a later owner. The latter exposes no state
and must remain inaccessible outside `mizar-checker`.

The non-exhaustive error has exactly these variants:

```rust
InvalidDependency
InvalidCaptureIdentity {
    capture_identity: SourceNestedFraenkelCaptureIdentityId,
}
```

Their exact display strings are:

```text
nested Fraenkel capture-identity dependency is invalid
nested Fraenkel capture identity <id> is invalid
```

Validation first reauthenticates the complete retained C4C4 dependency,
source/module, and fingerprint. Every such failure is `InvalidDependency`.
It then requires exact table cardinality and dense ID layout; every wrong total
count or ID layout reports
`InvalidCaptureIdentity { capture_identity: ...Id::new(0) }`. Finally it
validates every row field, including the exact inner owner
context/range/parent/layer/scope/recovery/visibility association, against the
C4C4 term, reference, projected binding, and retained C4C3 use/binding. The
lowest invalid row is reported; for this exactly-one profile that is always ID
`0`. The projected binding's captured identities must remain empty.

Any missing, extra, duplicate, reordered, stale, mismatched, recovered, or
partially valid state fails in that precedence and publishes no handoff. The
validator does not sort, deduplicate, infer by spelling, mutate C4C4, or repair
state.

## Typed/Resolved and Core/GeneratedOrigin boundaries

C4C5 owns only the checker association receipt. It does not decide an eventual
capture set, change binding visibility, or install anything into `TypedAst` or
`ResolvedTypedAst`. A later checker owner must be selected by a separate human
decision before it may consume this handoff and create any capture semantics.

Core Task 33 remains the later owner of Core context/binder/source identity.
Core Task 35 remains the later owner of term/formula lowering, binder links,
generated comprehension origins, and source identity. C4C5 assigns no owner or
ordering to a future explicit free/generated-parameter transport: that join
between the Core-33 and Core-35 surfaces requires a separate human decision.
C4C5 adds no Core adapter, `CoreVar`, parameter, `GeneratedOrigin`,
sethood/membership evidence, or ordering rule. A future Core boundary must
consume a complete checker-owned input under its own separately frozen
contract; it must not reinterpret the checker dense IDs as Core identities.

## Scope, tests, and audit impact

Production changes are limited to:

1. `crates/mizar-checker/src/source_term.rs` for the crate-private C4C4
   complete-validation seam;
2. `crates/mizar-checker/src/source_formula_composition.rs` for the sole owner,
   oracle, producer, and checker tests; and
3. existing private
   `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs`
   for the sole current consumer.

The checker owner adds exactly four tests:

1. `task257c4c5_builds_exact_capture_identity_handoff`;
2. `task257c4c5_rejects_dependency_owner_and_precedence_corruption`;
3. `task257c4c5_rejects_identity_cardinality_order_and_field_corruption`; and
4. `task257c4c5_replays_deterministically_and_preserves_empty_capture_and_installation`.

The private leaf adds exactly
`task257c4c5_real_imported_fixture_builds_capture_identity_handoff`.
All tests call only the exact C4C4/C4C5 route. There is no active runner or
registry consumer.

Baselines at clean `HEAD 17b9af203fefe65d48ed88758d356ff8cdfcd0a3` are:

- `source_formula_composition.rs`: `9411` lines,
  SHA-256 `2b982a6ab418e63ee6996c428aea2f8d5a4b3fc6bb55c9e830043f07fec73e56`;
- `source_term.rs`: `7574` lines,
  SHA-256 `2ef60bd40d0ff147f1615d20bd3a9fff3980e916868da90f998b00c3b4d369fe`;
- private leaf: `416` lines,
  SHA-256 `7760e98cb9b6fb3ea26f232b34551119d6d084c0f4785cd11b3af7cf829be1f1`;
- raw library test counts: checker `562 -> 566`, mizar-test `621 -> 622`;
- paired task-contract counts: `99/99 -> 100/100`.

`doc/design/spec_coverage_audit.md` receives one explicit zero-credit mapping
after implementation because a new durable association owner and private
consumer become current. No trace row/status, test intent, active route,
diagnostic, semantic result, or coverage credit changes.

## Completion evidence

Final source measurements are:

| Path | Lines | SHA-256 |
|---|---:|---|
| `crates/mizar-checker/src/source_formula_composition.rs` | `9940` | `1b4efce50a86f36357478f1dcf98f64bda96a710de6ed1b8caa79e056cc3a515` |
| `crates/mizar-checker/src/source_term.rs` | `7583` | `f7703a170781fe0a2bd2840589ecab79ca56c2cd25006ba469abdebeac7012c0` |
| `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs` | `519` | `4c403bdc7b060e52b5ba6585b82d5f34485813a49d4d035ac7214239206b72cf` |

Checker production remains `32` paths and measures `196872` lines. Its path
hash remains
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787`;
its final content-manifest hash is
`47be280901c7feb00ce3454dc8d59d15fed71e741183b2f2201b034ef0e117a3`.
The paired contract trees measure `100/100`.

Focused tests pass `4/4` checker and `1/1` mizar-test. Full library suites pass
`566/566` and `622/622`; checker and mizar-test lint-policy suites pass
`15/15`; metadata passes `137/137`. `cargo fmt --check`, warnings-denied
workspace Clippy, full `cargo test`, and `git diff --check` pass. The protected
fixture, expectation, and trace hashes remain the exact values frozen above,
and `doc/spec` and those protected artifacts have no diff.

All required independent reviews conclude **NO FINDINGS**. The final read-only
quality review passes all `9/9` hard gates with no score cap and assigns a
valid uncapped `100/100` (`20/20/15/15/10/10/5/5`). It confirms the sole
owner, exact by-value identity receipt, default-deny oracle, empty captured
state, zero-credit audit treatment, and the absence of Typed/Resolved or Core
installation. Task 277B remains not-ready and zero-credit. Exact staging, the
task-only implementation commit, and its immediate clean postcommit proof
remain transactional exit work.

## Forbidden behavior, reviews, and exit

This task must not change `doc/spec`, any existing `.miz`, expectation, trace,
diagnostic, active behavior, semantic result, coverage credit, C4C4 captured
state, or Task-277B readiness. It must not add a capture set, a semantic
capture decision, type/sethood/membership output, generated parameter, Core
origin, installation, production dispatch, or a new fixture.

Required independent reviews are specification/equivalence,
bilingual/boundary, test sufficiency, implementation, and
source/documentation/API consistency. Any ambiguity that leaves multiple
owners, APIs, installation surfaces, or corruption oracles reopens
`design_drift`, `test_gap`, or `boundary_violation` and stops implementation
without inventing a replacement.

Exit requires all reviews to report no findings; focused and library tests;
checker and mizar-test lint/metadata tests; `cargo fmt --check`;
warnings-denied workspace Clippy; full workspace tests; protected-hash and
Task-277B checks; exact diff/staging review; task-only commit; and a clean
postcommit proof. Final measured counts and hashes are recorded once here.
