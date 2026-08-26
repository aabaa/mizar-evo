# Task RESOLVE-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C8R: Exact two-capture resolver identity prerequisite

> Canonical language: English. Japanese companion:
> [../ja/RESOLVE-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C8R.md](../ja/RESOLVE-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C8R.md).

Owning plans: [mizar-resolve](../../mizar-resolve/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).
Durable owner sections: resolver
[names](../../mizar-resolve/en/names.md#resolver-task-257c4c8r-exact-nested-multi-capture-identity)
and test
[harness](../../mizar-test/en/harness.md#resolver-task-257c4c8r-private-two-capture-probe).

## Status, purpose, and readiness

**Status:** implementation complete and task-only commit pending. The parser
[C4C8P prerequisite](PARSER-FRAENKEL-GENERATOR-DELIMITER-257C4C8P.md) committed
at `6bc8de3a007d0260d14d2c803dc335623b6aa912`, and the mandatory fresh
exact-source preflight proved the C4C7 AST diagnostics-free and unrecovered
before resolver source work began.

This is the dependency-minimal successor selected by the clean post-C4C7
inventory. It extends the existing resolver-owned
`FraenkelGeneratorVariableSourceCollection` for exactly the frozen C4C7
two-capture source. It adds no public Rust item or enum variant and creates no
checker capture, Typed/Resolved installation, Task-252 occurrence graph, type
or sethood result, semantic verdict, diagnostic, active runner, Core identity,
generated parameter/origin, or Task-277B credit.

The task is uniquely ready under the user-approved boundary because Chapter
13 and the exact C4C7 oracle fix three generator declarations and two mapper
references by resolved binder identity. The existing R2/C4C2 public tables
already own arbitrary row cardinality, resolved-node provenance, global source
ordering, dense ordinals, and deterministic debug output. A checker-private
identity producer would recreate or reinterpret resolver identity and is a
`boundary_violation`; a new resolver API is unnecessary.

Current rejection of the valid exact source is `source_drift`. Missing
resolver-unit and private real-fixture coverage is a `test_gap`. The later
standalone checker graph/API remains `design_drift` and is not part of this
task. There is no blocking `spec_gap` or `repo_metadata_conflict`.

## Authority and dependencies

Authority is, in order:

1. canonical Chapter 4
   [§4.6](../../../spec/en/04.variables_and_constants.md#46-scoping-and-shadowing)
   and Chapter 13
   [§§13.4.3--13.4.4](../../../spec/en/13.term_expression.md#1343-multiple-generators)
   and [§13.8.6](../../../spec/en/13.term_expression.md#1386-set-expression-encoding);
2. exact existing
   [`pass_types_nested_comprehension_two_outer_generator_captures_001.miz`](../../../../tests/miz/pass/types/pass_types_nested_comprehension_two_outer_generator_captures_001.miz);
3. its existing trace backlink and inactive expectation;
4. completed R2, C4C2, and C4C7 contracts and derived owner records;
5. current source inventory, which is non-normative.

Required commits are C4C7 artifact
`3d28af5f6678519fe8d764fb29f27eb664db8f39` and closure
`b4037c853632aed80a824d05b955e4ad6396f4e1`. C4C2's one-capture branch and
all C4C3--C4C6 checker artifacts are protected dependencies, not inputs to be
generalized here.

Implementation preflight after documentation commit `5b165dd38e5f1a560eeaff80ef65aa8e5eab0539`
found the exact source currently contains parser recovery because the first
outer generator's `Element of NAT` consumes the following generator comma.
That parser `source_drift` is owned exclusively by C4C8P; resolver must not
reconstruct, repair, or admit the recovered AST.

## Frozen exact resolver relation

Reuse byte-for-byte all existing R2/C4C2 public types, getters, enum variants,
error surface, constructor signature, table behavior, and debug grammar. Add
one private exact candidate branch only. For the unrecovered C4C7 source, the
collection yields these binding rows after the existing global segment-range
sort:

| Binding ID | Comprehension | Spelling | Segment range | Binder range | Source ordinal |
|---:|---|---|---|---|---:|
| `0` | inner | `z` | `110..129` | `110..111` | `0` |
| `1` | outer | `x` | `144..163` | `144..145` | `1` |
| `2` | outer | `y` | `165..184` | `165..166` | `2` |

The inner bracket mapper occupies `97..103`. Its `x@98..99` and `y@101..102`
identifier references yield two existing `Mapper` links owned by the inner
comprehension and the same mapper `TermExpression`. The links target binding
IDs `1` and `2` respectively, and receive global/mapper-local ordinals `0/0`
and `1/1` from the existing identifier-range sort. The inner `z` declaration
has no use link and is not a captured outer identity. Debug output keeps the
existing grammar and ends in `bindings=3|uses=2`.

The ordering above is inherited resolver source ordering, not new language
semantics and not the later checker capture-vector contract. No consumer may
sort, repair, infer, deduplicate, join by display name, or reinterpret binding
IDs. All public identities continue to come only from
`SurfaceResolvedArena::resolved_node_for`; `new` and `collect` revalidate the
complete arena.

Admission is default-deny and exact. The candidate has one definition/functor,
exactly two nested condition-free set comprehensions, exactly three generator
segments, inner `z`, outer `x` then `y`, all three exact normal `Element of
NAT` types, and one exact bracket application `[x, y]` as the inner mapper.
Its two direct arguments are one-identifier term references to the two outer
binders. Missing, extra, duplicate, reordered, renamed, alternate-type,
condition-bearing, wrapped, recovered, partially matched, additionally nested,
or otherwise unsupported shapes produce zero rows for that candidate.
Existing F5, R2 malformed-nested exclusion, and exact one-capture C4C2 output
remain byte-for-byte compatible.

## Frozen implementation and test scope

The documentation prerequisite changes exactly 11 paths: this paired contract,
both owner Task Index plan pairs, the paired resolver names and mizar-test
harness sections, and `doc/design/spec_coverage_audit.md`. The audit receives
one zero-credit mapping section; Chapter-13 summary status remains `partial`.

After documentation reviews, implementation changes exactly three Rust paths:

1. `crates/mizar-resolve/src/names.rs`;
2. `crates/mizar-resolve/src/names/tests.rs`; and
3. `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs`.

Artifact completion may additionally update only this paired contract's
status/evidence and the dedicated coverage-audit paragraph from planned to
completed state. The final implementation commit therefore changes exactly
six paths. The other durable owner sections use completion-neutral wording and
do not require status-only edits.

Resolver adds exactly four tests:

1. `task257c4c8r_collects_exact_nested_multi_capture_relation`;
2. `task257c4c8r_preserves_outer_scope_and_excludes_inner_generator`;
3. `task257c4c8r_rejects_near_miss_profiles`; and
4. `task257c4c8r_revalidates_arena_and_replays_deterministically`.

The existing private mizar-test leaf adds exactly
`task257c4c8r_real_imported_fixture_links_both_outer_generators`. It runs the
real imported C4C7 source through the existing frontend and resolver lowering,
invokes the public collector directly, and asserts only the exact 3-binding/
2-link resolved relation, normal node/range provenance, deterministic replay,
and unchanged empty type-elaboration import augmentation. It is not a
production route or advanced-semantics execution.

## Baseline, protected state, and expected impact

Clean baseline HEAD is `b4037c853632aed80a824d05b955e4ad6396f4e1`,
origin/main is `ffc882675141a3e25bc78a47affc018bfe3685e1`, and divergence
is `0/4`. Protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` remains untouched.

| Path | Baseline lines / bytes | Baseline SHA-256 |
|---|---:|---|
| `crates/mizar-resolve/src/names.rs` | `4183 / 131548` | `ac05067d09a8da784e6faa8f5078eb4e7b57c4dfa331d06b94594f7edc97254d` |
| `crates/mizar-resolve/src/names/tests.rs` | `4287 / 137729` | `feb8f5721131c5bc92ba8e04ced2cfe9634e16c21f64f876a2bafb27ed1858d1` |
| private mizar-test leaf | `589 / 23379` | `86d9f5fcdc088fb678f5346fac01bf5f904821cf18455f75d2b7c6792a6e1e5a` |

Library tests project resolver `160 -> 164` and mizar-test `623 -> 624`.
Sorted baseline list hashes are resolver
`c041a4a4c978ac484863ad6025f39490ffdc4b7aa61e34d8e6c7cb2ca5592211`
and mizar-test
`323cf492377c6213ddd9e8c654d8a57b7e2c22b1af1bc36851b38523f69f966c`.
Contract trees project `102/102 -> 103/103`.

The C4C7 source and sidecar remain exact at SHA-256
`b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`
and `277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`.
Trace remains exact at
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`.
No `doc/spec`, `.miz`, sidecar, trace, metadata count, parser, checker/Core
production, C4C4 captured state, diagnostic, active route, or Task-277B change
is authorized.

## Reviews, verification, exit, and handoff

Before source edits, independent specification/equivalence and bilingual/
boundary reviews must report **NO FINDINGS**. After implementation,
independent test-sufficiency, implementation, source/documentation/API, and
final-quality reviews must report **NO FINDINGS**, with finding-specific
re-review after every repair.

Required verification is focused C4C8R tests, existing C4C2/R2 compatibility,
resolver and mizar-test libraries, both lint-policy suites, metadata, parser
set-comprehension coverage, formatting, offline Cargo metadata, full-workspace
all-target/all-feature warnings-denied Clippy, full workspace tests,
`git diff --check`, exact hashes/counts, and protected-surface checks.

Exit requires all `9/9` hard gates and a valid score of at least `90/100`,
exact task-only staging/commit, clean postcommit proof, and fresh successor
inventory. The next checker C4C8 projection may be frozen only if the committed
resolver relation plus fresh checker/typed inventory uniquely fixes its
standalone graph/API/cardinality/default-deny contract. Core Tasks 33/35 and
Task 277B remain deferred.

## Precommit implementation completion evidence

Fresh post-C4C8P frontend preflight used the exact frozen C4C7 source and
reported zero diagnostics and recovery nodes, `95` AST nodes, two set
comprehensions, three generator segments, one bracket application, and the
frozen mapper, segment, binder, and type ranges. Resolver implementation then
changed only the three frozen Rust paths. It adds one private exact candidate,
reuses every existing public R2/C4C2 type and table, maps uses to binder node
identities before source-order ID assignment, and emits only resolver-owned
resolved-node provenance.

Final source measurements are:

| Path | Lines / bytes | SHA-256 |
|---|---:|---|
| `crates/mizar-resolve/src/names.rs` | `4415 / 140538` | `663ec040a0b9525cb79b532fe7ae6a548f67acb7510b8713df3b0cfe2b8d6166` |
| `crates/mizar-resolve/src/names/tests.rs` | `4798 / 153865` | `d53afc1d148b3ab55bdbf97a04d11f78f4fe454a0caf6ca43f8ea72d6a55c504` |
| `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs` | `704 / 27913` | `6a1717fec263e79d9295813b413d1ec323c3291297f9ee04e0bc7c8e59e2e754` |

Resolver and mizar-test library states are exactly `164` and `624` tests with
sorted raw-list SHA-256 values
`a01c16a16aead9868d30257e358a4e742dd7633a8da4f61c864d9197d9c1f1c8`
and
`21196d1cb959c5b6bd7b38f19efb83d334978ec7f1d0c99e35da19cec8afe385`.
All five exact C4C8R tests pass, as do the C4C2/R2 compatibility tests,
resolver library `164/164` and lint `11/11`, mizar-test library `624/624`,
metadata `137/137`, lint `15/15`, parser C4C8P compatibility, formatting,
offline Cargo metadata, and `git diff --check`.

Independent test-sufficiency and implementation reviews report
**NO FINDINGS**. The first real-fixture execution exposed an implementation-
local mismatch that rejected normal `end;` siblings; the candidate was aligned
with the existing C4C2 one-functor child boundary before review, temporary
debug output was removed, and the real fixture then passed. Broad workspace
warnings-denied all-target/all-feature Clippy and full all-feature workspace
tests pass. The bilingual/boundary review found one stale C4C8P audit sentence;
after the sentence was aligned with this completed zero-credit mapping, its
finding-specific re-review reported **NO FINDINGS**. Source/documentation/API
review also reported **NO FINDINGS**, with exact scope, counts, hashes, public
API, owner links, and protected boundaries confirmed. Final-quality scoring,
performed independently against the autonomous-development rubric, reports
**NO FINDINGS**, all `9/9` hard gates passing, and a valid uncapped `100/100`
score. Exact staging and the task-only commit remain exit steps.

The C4C7 source, sidecar, and trace reproduce their protected hashes, paired
contract trees are `104/104`, and no `doc/spec`, `.miz`, expectation, trace,
metadata, parser/checker/Core production, C4C4 captured state, diagnostic,
active route, or Task 277B state changes. This task closes only the resolver
`source_drift` and private `test_gap`; coverage credit remains zero.
