# Task CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-INSTALLATION-257C4C6: Nested Fraenkel Capture-Identity Installation

> Canonical language: English. Japanese companion:
> [../ja/CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-INSTALLATION-257C4C6.md](../ja/CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-INSTALLATION-257C4C6.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index)
and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).
Durable owner sections are checker
[source formula composition](../../mizar-checker/en/source_formula_composition.md#task-257c4c6-capture-identity-installation-boundary),
[TypedAst](../../mizar-checker/en/typed_ast.md#task-257c4c6-capture-identity-installation),
[ResolvedTypedAst](../../mizar-checker/en/resolved_typed_ast.md#task-257c4c6-capture-identity-installation),
and the private test [harness](../../mizar-test/en/harness.md#checker-task-257c4c6-private-capture-identity-installation-probe).

## Status, decision, and purpose

**Status:** complete.

The human decision after completed C4C5 selects one checker-only,
zero-semantic successor. `TypedAst` and `ResolvedTypedAst` become the immutable
destinations of the already authenticated C4C5 receipt. Storage is one private
boxed installation wrapper in each AST; public access remains a borrowed view
of the existing C4C5 handoff. Installation authenticates the exact final
`TypedAst` against C4C5's retained pre-install snapshot, and final assembly
clones only from that authenticated typed owner.

This task does not decide capture semantics. It does not populate C4C4's
captured state, install a capture set, create a Core identity map, lower a
Fraenkel term, create a generated parameter/origin, select a generalized
parameter order, or activate Task 277B.

The prior absence of a selected installation owner/API is `design_drift`
resolved by the human decision. Missing exact final-owner and corruption tests
are a `test_gap`. Treating the receipt as capture semantics or Core readiness
would be a `boundary_violation`. There is no `spec_gap` in this zero-semantic
transport.

## Authority and protected meaning

Authority remains, in order:

1. canonical [Chapter 13 §§13.4.2, 13.4.4, and 13.8.6](../../../spec/en/13.term_expression.md#134-set-expressions);
2. existing [`pass_types_nested_comprehension_outer_generator_capture_001.miz`](../../../../tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.miz);
3. its sole [trace row](../../../../tests/coverage/spec_trace.toml);
4. its inactive [expectation](../../../../tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.expect.toml);
5. completed [C4C2](RESOLVE-FRAENKEL-NESTED-CAPTURE-257C4C2.md),
   [C4C3](CHECKER-FRAENKEL-NESTED-BINDER-USE-257C4C3.md),
   [C4C4](CHECKER-FRAENKEL-NESTED-MAPPER-PRIMARY-257C4C4.md), and
   [C4C5](CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-257C4C5.md), followed by
   the derived owner documents and source inventory.

The fixed meaning remains:

- inner mapper `x@94..95` refers to the resolved binding identity of outer
  generator `x@136..137`;
- inner generator `y@102..103` is local and is not captured;
- association is by resolved binding identity, not spelling or equality of
  checker, resolver, or future Core numeric IDs;
- C4C4's outer-x projection remains by value and its captured state remains
  empty;
- C4C5 `source_ordinal == 0` remains only its exact association coordinate.
  It is not a generalized capture, Core parameter, or application-argument
  order;
- Task 277B remains not-ready and zero-credit.

The protected `.miz`, expectation, and trace SHA-256 values are respectively
`c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431`,
`9ed000a30c1d519bd665f338c636fb9e529e9848a285209bebe6728f19961b92`, and
`d4d817e83aac78d19e729702b26c62604fc57581eec18672a5c26ec44efe7a81`.
They must remain byte-identical.

## Ownership, dependency, and storage

The existing C4C5 handoff remains solely produced and completely validated by
`source_formula_composition`. This task adds only a crate-private typed-owner
installation seam that:

1. runs C4C5 complete validation;
2. obtains its retained C4C3 pre-install `TypedAst` snapshot without exposing
   that snapshot publicly;
3. compares every field of the current final typed snapshot with that exact
   retained snapshot, ignoring only the destination slot when revalidating an
   already installed owner; and
4. rejects any missing, foreign, stale, recovered, partial, or additionally
   populated final snapshot.

C4C3 validation rejects an already installed C4C6 owner and any populated
resolved root, local context, type, fact, coercion, initial-obligation, or
typed-diagnostic table. This prevents a new C4C3/C4C4/C4C5 chain from
recursively retaining an installed receipt or carrying semantic typed state
across this zero-semantic boundary.

Both ASTs store a crate-private `InstalledSourceNestedFraenkelCaptureIdentity`
wrapper containing one boxed C4C5 handoff. The indirection is mandatory because
the handoff transitively retains the C4C3 `TypedAst`; a direct by-value slot
would be recursively sized. The wrapper has private construction, immutable
borrow, `Clone`/`PartialEq`/`Eq`, and a concise custom `Debug` implementation.
It is not public API. `TypedAstParts` and `ResolvedTypedAstInputs` gain no
field.

## Frozen public API and errors

`TypedAst` gains exactly:

```rust
pub const fn source_nested_fraenkel_capture_identity(
    &self,
) -> Option<&SourceNestedFraenkelCaptureIdentityHandoff>;

pub fn with_source_nested_fraenkel_capture_identity(
    self,
    handoff: SourceNestedFraenkelCaptureIdentityHandoff,
) -> Result<Self, TypedAstError>;
```

`ResolvedTypedAst` gains only the same read-only getter:

```rust
pub const fn source_nested_fraenkel_capture_identity(
    &self,
) -> Option<&SourceNestedFraenkelCaptureIdentityHandoff>;
```

There is no mutable getter, raw wrapper getter, `Default`, replacement API,
adapter, conversion, independent resolved input, caller-selected profile, or
unchecked installation.

`TypedAstError` gains exactly
`InvalidSourceNestedFraenkelCaptureIdentity` with display:

```text
typed AST source nested Fraenkel capture-identity handoff is inconsistent
```

`ResolvedTypedAstError` gains the same variant name with display:

```text
resolved typed AST source nested Fraenkel capture-identity handoff is inconsistent
```

Both enums remain non-exhaustive. No diagnostic is emitted.

## Installation and final-assembly oracle

Typed installation is consuming, one-shot, immutable, and atomic. It accepts
only when the input handoff is complete, the slot is absent, and the entire
pre-install `TypedAst` equals C4C5's retained C4C3 snapshot. Exact equality
authenticates source, module, arena/root, resolved links, every source owner,
all local contexts/types/facts/coercions/initial obligations/diagnostics, and
recovery state. Source/module equality alone is insufficient.

Every existing public `TypedAst::with_source_*` installer rejects when the
C4C6 slot is present, using that installer's existing error variant. The C4C6
installer in the reverse order rejects any pre-existing owner or table not
present in the retained snapshot. There is no sorting, deduplication,
inference, repair, overwrite, merge, or partial publication. Test-only
injection may create corrupt states solely for the rejection oracle.

Final assembly receives no independently replaceable receipt. When the typed
slot is present it:

1. repeats complete C4C5 and exact final-typed-snapshot validation;
2. requires syntax-only external assembly inputs: empty cluster facts,
   overload collections/expansions/viability/specificity/selection,
   expression metadata, and node hints, with no statement semantic/proof
   bundle;
3. clones the same immutable boxed owner into `ResolvedTypedAst`; and
4. publishes no type fact, overload result, coercion, obligation, diagnostic,
   checked formula, statement semantic, proof, capture, or Core payload.

Any failure returns the frozen Typed or Resolved error without a partial AST.
C4C4 captured state remains empty before and after typed installation, final
clone, deterministic replay, and every failed attempt.

## Debug and representation boundary

When absent, both existing debug renderings remain byte-identical. When
present, each appends exactly the existing C4C5 `debug_text()` plus one newline
immediately after the existing `source_formula_composition` position and
before `source_condition_formula_composition`. The chunk occurs exactly once.
The task does not change C4C5's standalone debug grammar.

The installation is only a checker-owned receipt destination. It supplies no
`CoreVarId`, sethood/membership evidence, mapper/predicate lowering graph,
generated owner/key/functor, `GeneratedOrigin`, `GeneratedOriginUse`, parameter,
argument, or durable Core provenance. The existing Core `Apply +
GeneratedOrigin` representation is unchanged and receives no new consumer.

## Tests and affected artifacts

Checker adds exactly six tests:

1. `task257c4c6_installation_authenticates_exact_typed_snapshot`;
2. `task257c4c6_typed_installation_is_boxed_one_shot_and_debug_stable`;
3. `task257c4c6_rejects_dependency_row_and_final_snapshot_corruption`;
4. `task257c4c6_reciprocal_installation_exclusion_is_atomic`;
5. `task257c4c6_resolved_clone_revalidates_and_preserves_receipt`; and
6. `task257c4c6_resolved_rejects_injected_stale_or_mismatched_receipt`.

Test 4 uses the test-only injection seam to place a C4C6 owner into the exact
pre-C4C3 typed profile, then requires
`SourceNestedFraenkelBinderUseProducer::build` to return
`InvalidTypedDependency` without publishing a new C4C3 handoff. It also covers
the ordinary existing-installer-after-C4C6 direction and rejects a populated
semantic typed table before C4C3 publication; test 3 covers C4C6 after a
mismatched or additionally populated typed owner. Tests 5 and 6 cover the
syntax-only final-input oracle, including representative nonempty-input
rejection, and directly preserve empty captured state across success, replay,
clone, and failure paths.

The existing private mizar-test leaf adds exactly
`task257c4c6_real_imported_fixture_installs_typed_capture_identity_receipt`.
It remains library-test-only and does not enter a runner registry or active
dispatch.

Production source changes are limited to:

- `crates/mizar-checker/src/source_formula_composition.rs`;
- `crates/mizar-checker/src/typed_ast.rs`; and
- `crates/mizar-checker/src/resolved_typed_ast.rs`.

Test source changes additionally include only the existing private
`crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs`.
Owner-document changes are limited to this contract pair, the two crate-plan
rows, checker typed/resolved/source-composition owner sections, checker todo,
the paired checker module-boundary and source-spec audits, the mizar-test
harness section, the paired checker and mizar-test bilingual audits, and the
zero-credit coverage-audit mapping.

At clean `HEAD ffc882675141a3e25bc78a47affc018bfe3685e1` the source baselines are:

| Path | Lines | SHA-256 |
|---|---:|---|
| `typed_ast.rs` | 6897 | `673ca701208e051071997dc3649628af2ed9344bff6e6be78ba9871e717762ba` |
| `resolved_typed_ast.rs` | 8908 | `c89d138f843885c8ea49139ce742f0e4b78bd0c5abc6865d4f9362b9f3ba68ae` |
| `source_formula_composition.rs` | 9940 | `1b4efce50a86f36357478f1dcf98f64bda96a710de6ed1b8caa79e056cc3a515` |
| private mizar-test leaf | 519 | `4c403bdc7b060e52b5ba6585b82d5f34485813a49d4d035ac7214239206b72cf` |

The paired module-boundary audits are `1918/1779` lines with SHA-256
`86accc2e478137ebae57c3851d726a9163de5be03e386e1257a0177bd6bbe558` /
`258fba5760d404dccbcea0f53979f520fc8ce12994e88ba7f7d68e3cc641621b`.
The paired source-spec audits are `6300/5946` lines with SHA-256
`abbb8deffe73a7e286688e09d144555258e2be9f892657f6f416f530825f722e` /
`28aedf8ccccbfac26ea5975c4c7172ceccc8ab2a7f06aecc0701e69fe9e024ec`.
All four audit hashes must change while preserving their owner-local inventory
and public-enum claims.

The checker bilingual audits are `1994/1840` lines with SHA-256
`47468c44fd462be1743f029dc7a1ba8573deedcc53dd84410b140189d9c969c4` /
`31c0df262189356e4571f0f45e727fc4c58667308b5b57011fde0b188d012436`.
The mizar-test bilingual audits are `2008/1855` lines with SHA-256
`5945f15d7bca346c50ce4beff89f4cc8023ca26f98088ee6097bbcfe6e40e628` /
`01aea1f8a59bb43aeb36475008d969fdba406c2a7aeb7424e1db0ab8d6526e55`.
All four bilingual hashes must change with English-canonical logical parity.

Checker production remains exactly 32 paths with path-list hash
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787`.
Raw library counts are expected to change checker `566 -> 572` and mizar-test
`622 -> 623`; paired contract counts change `100/100 -> 101/101`. All four
affected source hashes must change; protected authority hashes, checker path
count/hash, diagnostics, public module count, existing test names, and active
route counts must not.

`doc/design/spec_coverage_audit.md` receives one explicit zero-credit mapping
because the durable Typed/Resolved receipt owner becomes current. No trace
row/status/backlink, `.miz` intent, expectation, diagnostic, active route,
semantic result, or coverage credit changes.

## Reviews, verification, and exit

Before implementation, independent specification/equivalence and
bilingual/boundary/API reviews must report no blocking/high findings. After
implementation, independent test-sufficiency, implementation, and
source/documentation/API reviews must do the same; finding-specific re-review
is mandatory after repairs.

Verification requires focused seven tests, checker and mizar-test library and
lint-policy suites, metadata tests, `cargo fmt --check`, warnings-denied full
workspace Clippy, full `cargo test`, `git diff --check`, protected hashes,
contract counts, checker path count/hash, Task-277B zero-credit checks, exact
task-only staging, commit, and clean postcommit proof. Final read-only review
must pass all `9/9` hard gates and score at least `90/100`.

## Precommit implementation completion checkpoint

Final source measurements are:

| Path | Lines | SHA-256 |
|---|---:|---|
| `crates/mizar-checker/src/typed_ast.rs` | `7094` | `2c0365f77706e344672a2933a8f31e933e44286b2473c9fbb05bdfb74fc9071c` |
| `crates/mizar-checker/src/resolved_typed_ast.rs` | `8998` | `90e0cf4d73c5f7d92f1a2e9e83c15a2c9c75b244eef609e73f20e501e81bf7a7` |
| `crates/mizar-checker/src/source_formula_composition.rs` | `10342` | `dd27218581ebe6c252da33f6feb23480403afa88858de874970d88a9d1573d44` |
| `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs` | `589` | `86d9f5fcdc088fb678f5346fac01bf5f904821cf18455f75d2b7c6792a6e1e5a` |

Checker production remains `32` paths and measures `197561` lines. Its path
and content-manifest SHA-256 values are respectively
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787` and
`a3d99114263d46552a59a14055e60b5938c683a4dd555423a1bc409335712ccc`.
The paired contract trees measure `101/101`. Checker and mizar-test raw library
test-list hashes are respectively
`53472bd49c9f8d6cb2c6950aaa805a9652375e24953a489f1d3497ac6d97ab8a` and
`0a75e7fad8a2cbb0883b62a172163457f6fc66b8a28004b1f741567233f2348b`.

Final paired audit measurements are: module-boundary `1930/1790` lines with
hashes `7b41b7e953c17234b5d1a2aeb5715b1364ff56cf9678d37063211fc14a9a7b75` /
`5a6613239badc355d3f969a0683cac34f92ae05188373a82e3f71590a0692fa9`;
source-spec `6301/5947` with hashes
`0beada5949f303e022a7e76a16589888b357f8036b1a9d856afb63f2b2dd7ee5` /
`2fc762d56e6f9fa6c48343cc18a3f266b2630513f2a319f3e09c6f55df57011c`;
checker bilingual `2009/1853` with hashes
`30fb9590575f2bfc5bc28c8b78bfb5c47958a95d81b7254f003de7bb9e381bd6` /
`ba09fa5304459991b75f72b175d40a80478cd80310e64ef4650ecc02b937f51b`;
and mizar-test bilingual `2019/1865` with hashes
`9bc4131e4688dcedd77af5b33267c9672b47198e585d3c18dd4ef272685eab8c` /
`ce3b922766db5e4c7ce912f1f9ee5f26fd954ef675bb6696e816ab5ff88f9df1`.

Initial independent reviews found the missing pre-C4C3 injection proof,
final-input contamination assertion, captured-empty assertions, runner
debug/clone evidence, and semantic-table guard. All were repaired without
changing the frozen API, test names/count, or boundary; specification,
test-sufficiency, implementation, source/documentation/API, and bilingual/
boundary finding-specific re-reviews all report **NO FINDINGS**.

Focused tests pass `6/6` checker and `1/1` mizar-test. Full library suites pass
`572/572` and `623/623`; both lint-policy suites pass `15/15`; metadata passes
`137/137`; and public-enum policy passes `2/2`. Package and full-workspace
warnings-denied Clippy, `cargo fmt --check`, full `cargo test`, Cargo metadata,
and `git diff --check` pass. The protected `.miz`, expectation, and trace
hashes remain exact; `doc/spec`, those protected artifacts, and `mizar-core`
have zero diff. C4C4 captured state remains empty, Task 277B remains not-ready
and zero-credit, and no semantic, route, diagnostic, Core, or GeneratedOrigin
surface changed. Independent final-quality review reports **NO FINDINGS**;
all `9/9` hard gates pass with no score cap at a valid uncapped `100/100`
(`20/20/15/15/10/10/5/5`). Its finding-specific exact-scope correction
confirms `29` approved paths: `27` tracked modifications and the `2` new paired
contracts. Exact staging, commit, and clean postcommit proof were the next
gates for the implementation snapshot described above.

## Postcommit proof and fresh successor inventory

The reviewed task-only implementation committed as
`b17cbfe5dad0bcb11502b4c7feef814df6adf8fb` over baseline
`ffc882675141a3e25bc78a47affc018bfe3685e1`. `git show --check` passed; the
immediate worktree was clean; `origin/main...HEAD` was `0/1`; and protected
stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4`, all three authority hashes,
source measurements, contract counts, C4C4 empty captured state, and Task 277B
not-ready/zero-credit status remained unchanged. The closure-record commit
hash and its subsequent clean proof are reported in the final handoff because
a commit cannot contain its own hash.

Fresh independent authority, checker, and Core inventories select no unique
same-milestone successor. Canonical Chapter 13 fixes capture by resolved binder
identity and says generated `params` are the surrounding free variables, but
does not fix a generalized captured-parameter or application-argument order.
The exact fixture derives one captured outer `x`, so ordering is vacuous there;
it cannot authorize a multi-capture rule. Core Task 33 owns future Core context,
binder identity, and provenance, while Core Task 35 owns future term/formula
and generated-origin lowering and depends on unfinished Core Tasks 33 and 34.
The accepted Core descendant contract already requires both tasks to consume
a checker-owned, syntax-free, source-ordered final projection. It does not
assign the exact resolver-binding-to-`CoreVarId` or captured-parameter-to-
application-argument positional join.

The current explicit Core API independently accepts
`params: Vec<CoreVarId>` and `args: Vec<CoreTermSeedId>`. It preserves argument
order and checks reused parameter equality, but does not authenticate a
checker/resolver-identity-to-`CoreVarId` map, parameter/argument cardinality,
or positional correspondence. `GeneratedOriginUse` is a lowering output, not
a durable `CoreIr` table. C4C6 deliberately supplies none of those fields or
owners.

The remaining candidates are:

| Candidate | Boundary assessment |
|---|---|
| A checker-owned complete, source-ordered final projection, followed later by the Core-33/Core-35 consumers | **Recommended and aligned with the accepted Core descendant contract.** It carries authenticated binder identities and full generator/mapper/predicate provenance without `CoreVarId`; its exact fields, cardinality, generalized capture order, and corruption oracle still require a human freeze. |
| Treat the existing exact C4C6 receipt as the complete Core projection | It is minimal, but C4C6 intentionally omits inner generator `y`, the complete term graph, generated owner/key/functor, params/args, and a generalized ordering rule. |
| Allocate or infer the missing association directly in Core 33 or Core 35 | Current Core inputs accept caller-assigned `CoreVarId`s and do not authenticate this source join; reconstruction there would violate the checker-final-projection boundary, while Core 35 also bypasses unfinished Core 33/34 dependencies. |

The unassigned exact join and competing final-projection surfaces are
`design_drift`; missing general cardinality, order, mapping, and corruption
tests are a `test_gap`.
Treating the one-row `source_ordinal`, checker/resolver numeric IDs, or current
Core vector order as the missing rule would be a `boundary_violation`. The
canonical ordering rule is absent; if the next contract makes that order part
of normative transport rather than a private alpha-invariant Core convention,
the absence is also a `spec_gap`. There is no authority contradiction and no
repository-metadata conflict.

Therefore this inventory creates no task ID, API, field, adapter, installer,
route, or semantic implementation. The smallest human decision needed for a
successor is whether to freeze checker as sole owner of a complete Core-facing
projection with no Core IDs, order its distinct captured identities by their
authenticated binder declaration/source order, and require the later Core
consumer to preserve that order positionally in generated parameters and
application arguments with exact default-deny mismatch checks.

## Forbidden behavior and next handoff

Do not change `doc/spec`, any existing `.miz`, expectation, trace row,
diagnostic, C4C4 captured state, Task-255 participation, active behavior,
semantic result, or Task-277B readiness. Do not add actual capture semantics,
Core33/35 transport, numeric-ID reinterpretation, display-name joins,
parameter/argument order, generated origin, sorting, repair, inference, or
unchecked admission.

Fresh inventory selected no successor for the reasons recorded above. C4C5
`source_ordinal` must not be promoted to a general parameter order.
