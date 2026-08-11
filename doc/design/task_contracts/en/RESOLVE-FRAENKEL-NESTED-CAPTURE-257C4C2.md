# Task RESOLVE-FRAENKEL-NESTED-CAPTURE-257C4C2: Exact Nested Fraenkel Resolver Identity

> Canonical language: English. Japanese companion: [../ja/RESOLVE-FRAENKEL-NESTED-CAPTURE-257C4C2.md](../ja/RESOLVE-FRAENKEL-NESTED-CAPTURE-257C4C2.md).

Owning plans: [mizar-resolve](../../mizar-resolve/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).
Durable owner sections: resolver [names](../../mizar-resolve/en/names.md#resolver-task-257c4c2-exact-nested-fraenkel-identity)
and test [harness](../../mizar-test/en/harness.md#resolver-task-257c4c2-private-imported-fixture-probe).

## Status, purpose, and readiness

**Status:** accepted precommit implementation. All required reviews have
reached **NO FINDINGS**, all `9/9` hard gates pass, and the valid uncapped
quality score is `100/100`; task-only commit, post-commit proof, and fresh
successor inventory remain.

This is the dependency-minimal successor selected by fresh inventory at clean
`HEAD b7f52dfa8d804c0adb4896cc5f1b9473ac99431c`. It extends the existing
resolver-owned `FraenkelGeneratorVariableSourceCollection` for exactly the
approved imported nested-comprehension oracle. It does not implement checker
capture, Task 252 occurrences, type or sethood interpretation, requests,
verdicts, diagnostics, an active runner, or Task 277B.

The task is dependency-ready because Chapter 13 §§13.4.4 and 13.8.6 uniquely
require the inner mapper `x` to refer to the outer generator by resolved binder
identity; the existing `.miz` and inactive pass sidecar fix that relation while
keeping inner generator `y` distinct; C4C1 completed zero-diagnostic frontend
admission; and completed R2 already owns the applicable resolver binding/use
tables without a public API extension.

There is no `spec_gap`. Current rejection of every nested comprehension by the
R2 collector is `source_drift` for this exact authority-backed profile; missing
exact resolver and real-fixture tests are a `test_gap`; and the owner document's
unqualified nested-exclusion statement is `design_drift`. Reusing C4A/C4B,
creating a Task-252 row, or publishing checker capture here would be a
`boundary_violation`.

## Authority and dependencies

Authority is, in order:

1. canonical [Chapter 4 §4.6](../../../spec/en/04.variables_and_constants.md#46-scoping-and-shadowing)
   and [Chapter 13 §§13.4.2, 13.4.4, and 13.8.6](../../../spec/en/13.term_expression.md#134-set-expressions);
2. exact existing
   [`pass_types_nested_comprehension_outer_generator_capture_001.miz`](../../../../tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.miz);
3. its sole existing [trace row](../../../../tests/coverage/spec_trace.toml);
4. its inactive [expectation](../../../../tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.expect.toml);
5. completed [R2](RESOLVE-FRAENKEL-GENERATOR-VAR-277R2.md),
   [C4C0](TEST-FRAENKEL-NESTED-CAPTURE-257C4C0.md), and
   [C4C1](TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md), plus
   derived owner documents and non-normative source inventory.

The existing source remains byte-identical at `164` bytes and SHA-256
`c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431`.
The sidecar remains inactive `advanced_semantics`, `pass/type_check`, with no
diagnostic codes or active tags. The trace row remains test-intent-only and
grants no execution or semantic credit.

## Frozen exact resolver relation

The existing public R2 types, getters, error, enum variants, and debug grammar
are reused byte-for-byte. No public item is added. For the exact normal C4C0
source, collection yields two binding rows and one mapper-use row:

| Binding ID | Comprehension | Spelling | Segment range | Binder range | Source ordinal |
|---:|---|---|---|---|---:|
| `0` | inner | `y` | `102..121` | `102..103` | `0` |
| `1` | outer | `x` | `136..155` | `136..137` | `1` |

The sole use is inner mapper identifier `x@94..95`. Its `comprehension()` and
`role_owner()` belong to the inner comprehension/mapper, its `binding()` is
outer binding `1`, its role is existing `Mapper`, and both its global and
mapper-local ordinals are `0`. The collection debug text remains the existing
grammar and ends in `bindings=2|uses=1`. This exact cross-comprehension link is
resolver identity only; it is not a checker capture row or generated-core
parameter.

Admission is default-deny. The task admits only the unrecovered C4C0 topology:
one definition/functor, an outer condition-free comprehension whose mapper is
one inner condition-free comprehension, exactly one generator per
comprehension, exact identifier term `x` as the inner mapper, distinct inner
`y` and outer `x`, and two exact `Element of NAT` type expressions. Each type
subtree has one normal `TypeExpression` with no attribute chain, one direct
`TypeHead`, one `QualifiedSymbol` whose sole spelling is `Element`, and one
direct `TypeArguments` beginning with reserved word `of` and containing one
normal term expression whose sole spelling is `NAT`; the source ranges are
`107..121` and `141..155`. Recovery, conditions, extra/missing/reordered
generators, equal inner/outer spellings, inner mapper spelling other than outer
`x`, an alternate or differently shaped generator type on either side,
ambiguous or additional identifier references, extra nesting, non-exact
wrappers, and partial matches produce no rows for that candidate. Completed R2
F5 output and its existing malformed synthetic nested exclusion remain
byte-for-byte compatible.

Binding and use ordering stays global source-range plus node-identity order.
All public rows continue to obtain identities only through
`SurfaceResolvedArena::resolved_node_for`; `new` and `collect` continue to
revalidate the complete arena. No raw `SurfaceNodeId` crosses the public API.

## Implementation and test scope

Implementation changes exactly these four Rust paths:

1. `crates/mizar-resolve/src/names.rs`;
2. `crates/mizar-resolve/src/names/tests.rs`;
3. `crates/mizar-test/src/runner/tests.rs`; and
4. new private
   `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs`.

Resolver adds exactly four tests:

1. `task257c4c2_collects_exact_nested_capture_relation`;
2. `task257c4c2_preserves_outer_scope_and_distinct_inner_binding`;
3. `task257c4c2_rejects_near_miss_nested_profiles`, including otherwise-normal
   alternate inner and outer generator types; and
4. `task257c4c2_revalidates_arena_and_replays_deterministically`.

The private mizar-test leaf adds exactly
`task257c4c2_real_imported_fixture_links_inner_mapper_to_outer_generator`. It
reuses the C4C1 frontend helper/provider, lowers the real admitted AST, invokes
the resolver collector directly, and asserts only the exact public relation,
normal node kinds/ranges, deterministic replay, and unchanged empty
type-elaboration resolver augmentation. It is not a production route or an
advanced-semantics execution.

Raw library test counts project `mizar-resolve 156 -> 160` and `mizar-test 618
-> 619`. Baseline file measurements are:

| Path | Lines | SHA-256 |
|---|---:|---|
| `crates/mizar-resolve/src/names.rs` | `3920` | `9a4b1a0e289c058a40c5af91d00fb836eca7af3a1d83bfcfa9b60227ce46d14a` |
| `crates/mizar-resolve/src/names/tests.rs` | `3601` | `31228c3502a08276a0c395715f74a6a5143a11c315145595ac88f93163e6863a` |
| `crates/mizar-test/src/runner/tests.rs` | `67` | `94bc44e8ba47ca568670adeec74d20f6738b3fc337da2422871095137040e8c4` |
| new private leaf | absent | absent |

Sorted raw test-list hashes are resolver
`7c84ee615616d7f0982454c8d04e9eef2fcb451efbb8fd576296e28af3cb6301`
and mizar-test
`d145e5bf5c8ae3f8231ffe73ee034b639001d349c99dd4f00f3c60b6382db4c1`.
Contract trees project `91/91 -> 92/92`.

## Precommit implementation completion evidence

The documentation prerequisite is commit
`ed46bcd550b9129818404b8f095dfa333c5f85cf`. The implementation changes only
the four frozen Rust paths. It reuses the public R2 tables unchanged, admits
the exact nested relation through private binding/use candidates, assigns
binding IDs after global source ordering, and resolves the inner mapper's
target through the outer binder identity. The required four resolver tests and
one private imported-fixture test are present; the default-deny matrix also
covers condition-bearing, extra/missing/reordered-generator, additional-
reference, extra-nesting, wrapper, recovery, binder, mapper, and both
alternate-type near misses.

Final precommit source measurements are:

| Path | Lines | SHA-256 |
|---|---:|---|
| `crates/mizar-resolve/src/names.rs` | `4183` | `ac05067d09a8da784e6faa8f5078eb4e7b57c4dfa331d06b94594f7edc97254d` |
| `crates/mizar-resolve/src/names/tests.rs` | `4287` | `feb8f5721131c5bc92ba8e04ced2cfe9634e16c21f64f876a2bafb27ed1858d1` |
| `crates/mizar-test/src/runner/tests.rs` | `68` | `c5d0395e69225fbf74492c5f58e122c829d7fd1a555f8f0db9abce3f7c7ef78c` |
| `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs` | `169` | `fa70dc53fb92376fcbd71b4058d9830355ab12fc4e5d6f67050d129cb7f46ae9` |

The raw library lists are exactly `160` resolver tests and `619` mizar-test
tests. Their sorted-list SHA-256 values are respectively
`c041a4a4c978ac484863ad6025f39490ffdc4b7aa61e34d8e6c7cb2ca5592211`
and `ad70984d911bd6ef84fc5efa15a50815acc7b4cc7daab1c89235263e022aa00b`.
The existing source, inactive sidecar, and trace remain byte-identical at
SHA-256 `c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431`,
`9ed000a30c1d519bd665f338c636fb9e529e9848a285209bebe6728f19961b92`,
and `d4d817e83aac78d19e729702b26c62604fc57581eec18672a5c26ec44efe7a81`.
The updated zero-credit coverage audit is `7021` lines / SHA-256
`390783d84ea52582fae037b78c0ab9ddd14d2fd7c4b04e1dd4676871b7164c01`;
contract trees are `92/92`, and corpus source/sidecar pairs remain `344/344`.

All focused C4C2 tests and the four existing R2 compatibility tests pass.
Resolver and mizar-test libraries pass `160/160` and `619/619`; their lint
suites pass `11/11` and `15/15`; metadata passes `137/137`. `cargo fmt --all
-- --check`, offline Cargo metadata, warnings-denied all-target/all-feature
Clippy, full workspace `cargo test`, and `git diff --check` pass. Replayed
plan/parse/declaration/type/proof CLI stdout hashes are respectively
`2d2accef2c6fc32c1b3372530f6136af1299ac6ae7db6a0158798336b779c7e7`,
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`;
each reports the unchanged `23` warnings and zero errors.

The first specification review's exact-type-shape finding and the first test
review's two provenance/default-deny findings were repaired; finding-specific
re-reviews reached **NO FINDINGS**. Independent bilingual/boundary, full
implementation, source/documentation/API, and completion-bilingual reviews
also report **NO FINDINGS**. Independent final-quality review likewise reports
**NO FINDINGS**: all `9/9` hard gates pass, no score cap applies, and the valid
score is `100/100`, split `20/20/15/15/10/10/5/5`. At that historical review
checkpoint the index contained exactly the six task paths, no unstaged path,
SHA-256 `90af71a41ac7ae346c20bc76fc6e74380ce808bf97a64e25de5148430118e149`
for its sorted path list, and `1278` insertions / `55` deletions. The parent
independently accepts the same hard-gate result. Commit, post-commit proof, and
fresh inventory remain lifecycle work rather than implementation findings.

## Protected scope and audit impact

The task must not edit `doc/spec`, the existing `.miz`, its expectation,
`tests/coverage/spec_trace.toml`, frontend/parser production, import-provider
behavior, resolver import augmentation, checker source, C4A/C4B, Task 252,
Typed/Resolved checker installation, Cargo metadata, diagnostics, runner
dispatch, or active coverage. It must not reinterpret the inactive expectation
or claim an executable pass.

`doc/design/spec_coverage_audit.md` receives one zero-credit design-mapping
addendum: the exact resolver identity owner is complete after implementation,
while checker capture transport, Task-252 occurrence ownership, type/sethood,
requests, verdicts, diagnostics, production routing, and Task 277B remain
deferred. Trace and expectation metadata stay byte-identical because their
test intent and executable status do not change.

## Reviews, verification, and exit

Required independent reviews are: specification/contract, test sufficiency,
implementation, source/documentation/API consistency, bilingual/boundary, and
final hard-gate quality. A re-review is required after every material finding.

Verification includes the five focused tests, resolver and mizar-test library
tests, both applicable lint-policy suites, metadata tests, formatting,
warnings-denied workspace Clippy, full workspace tests, all five CLI replays,
protected artifact hashes, exact scope/diff checks, and post-commit proof.

Exit requires **NO FINDINGS**, all nine autonomous hard gates passing, and a
valid quality score of at least `90/100`. The implementation receives a
task-only commit, followed by a clean post-commit inventory. Fresh successor
inventory must stop unless a separate checker-capture or other task is proved
dependency-ready. Task 277B stays not-ready with zero semantic credit.

Recommended routing: GPT-5.6 Sol at `xhigh` owns authority, API/boundary
acceptance, and final scoring. GPT-5.6 Terra at `high` or `xhigh` may perform
the frozen implementation and independent reviews. Any need for a new public
item, generalized nested/shadow semantics, checker payload, diagnostic, or
active route returns to the parent before editing.
