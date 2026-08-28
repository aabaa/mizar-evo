# Task CORE-FRAENKEL-CAPTURE-CONTEXT-33C4C8: Core capture-context association

> Canonical language: English. Japanese companion:
> [../ja/CORE-FRAENKEL-CAPTURE-CONTEXT-33C4C8.md](../ja/CORE-FRAENKEL-CAPTURE-CONTEXT-33C4C8.md).

Status: complete in task-only commit
`774a4781ccaedfbba2b5c9ecbf7bf26b79724616`. No uniquely ready successor was
found in the required fresh inventory. This is the dependency-minimal,
zero-semantic Core-33 successor to checker Task33C. The user selected this
contract after the unresolved choices recorded by
[CORE-SOURCE-CONTEXT-33P](CORE-SOURCE-CONTEXT-33P.md) were reviewed.

## Identity, authority, and ownership

| Field | Frozen value |
|---|---|
| Task | `CORE-FRAENKEL-CAPTURE-CONTEXT-33C4C8` |
| Primary owner | `mizar-core::elaborator`, Core Task 33 |
| Owning plans | [`mizar-core` crate plan](../../mizar-core/en/00.crate_plan.md); private consumer in [`mizar-test` crate plan](../../mizar-test/en/00.crate_plan.md) |
| Sole checker admission | Existing immutable `SourceNestedFraenkelCaptureGraphOwnerHandoff` from checker Task33C; its complete validator remains checker-private |
| Core destination | One standalone immutable `SourceNestedFraenkelCaptureCoreContextHandoff`; no `CoreContextInput`, Typed, or Resolved field |
| Owner join | Exact retained Task33R `SymbolId` to the existing `CoreItemRegistry` entry; no display-name, range-only, FQN-only, or numeric join |
| Captures | Exactly the C4C8 capture-table rows for outer `x,y`; inner generator `z` is excluded |
| Order | C4C8 authenticated private capture-table order, used only as deterministic transport |
| Allocation | Fresh snapshot-local `CoreVarId`s from checked `max(existing Core variable identity) + 1`, then consecutive in capture order; empty context starts at zero |
| Coverage | Zero semantic and execution credit; Task277B remains not ready and receives zero credit |

Authority remains, in order, `doc/spec/en/`, the exact C4C7 `.miz`, its trace
row and expectation, C4C8/Task33R/Task33C contracts, then Core design and
source. Chapter 13 fixes capture by resolved binding identity but does not make
parameter order observable. This task therefore treats the checker-authenticated
`x,y` order as a private, alpha-invariant canonical transport rule, not as new
language semantics.

## Frozen public API

`crates/mizar-core/src/elaborator.rs` adds:

- immutable `SourceNestedFraenkelCaptureCoreVariable` with getters
  `capture()`, `generator()`, `resolver_binding()`, and `core_var()`;
- immutable `SourceNestedFraenkelCaptureCoreVariableTable`, keyed directly by
  `SourceNestedFraenkelCaptureGraphCaptureId`, with
  `get(id) -> Option<&SourceNestedFraenkelCaptureCoreVariable>`,
  `iter() -> impl Iterator<Item = (SourceNestedFraenkelCaptureGraphCaptureId,
  &SourceNestedFraenkelCaptureCoreVariable)>`, `len() -> usize`, and
  `is_empty() -> bool`;
- immutable `SourceNestedFraenkelCaptureCoreContextHandoff` retaining the
  updated `CoreContext`, the checker Task33C receipt, the exact owner
  `CoreItemId`, and the capture-variable table, with getters `source_id()`,
  `module_id()`, `context()`, `checker_receipt()`, `owner_item()`,
  `captured_variables()`, and non-authoritative `debug_text()`;
- non-exhaustive `SourceNestedFraenkelCaptureCoreContextError` with exact
  precedence variants `EnvironmentMismatch`, `InvalidCoreContext`,
  `InvalidOwnerAssociation`, `CoreVariableAllocationOverflow`,
  `CoreVariableCollision { var }`, and `InvalidCaptureAssociation`;
- `SourceNestedFraenkelCaptureCoreContextProducer::build(
  context: CoreContext,
  checker_receipt: SourceNestedFraenkelCaptureGraphOwnerHandoff,
  ) -> Result<SourceNestedFraenkelCaptureCoreContextHandoff,
  SourceNestedFraenkelCaptureCoreContextError>`.

The collision payload is `var: CoreVarId`. The error derives `Debug`, `Clone`,
`Copy`, `PartialEq`, and `Eq`, implements `std::error::Error`, and has exact
display strings, in variant order:

- `nested Fraenkel capture Core context environment is invalid`;
- `nested Fraenkel capture Core context is invalid`;
- `nested Fraenkel capture Core owner association is invalid`;
- `nested Fraenkel capture Core variable allocation overflowed`;
- `nested Fraenkel capture Core variable <index> collides`;
- `nested Fraenkel capture Core association is invalid`.

The producer consumes both inputs by value and publishes nothing until the
complete handoff validates. Row/table/handoff fields and constructors are
private. The handoff does not expose a public installer, adapter, mutable field,
unchecked constructor, numeric conversion, or parameter/argument vector.

## Installation and validation

The existing Task33C value is the proof-carrying checker capability. Core does
not duplicate or expose its private graph validator. Core performs only its
own boundary checks in this order:

1. exact checker receipt/Core-context `SourceId` and `ModuleId` equality;
2. coherent existing Core variable metadata and used-ID inventory;
3. exact Task33R whole-`SymbolId` lookup to one valid `Functor` Core item, with
   matching local semantic-origin source/module and source-range anchor;
4. checked allocation above every existing Core variable identity, rejecting
   overflow or collision;
5. exact two-row positional association with the checker capture table,
   matching capture id, generator id, resolver binding, and generator binder
   source; captured `z`, missing, extra, duplicate, or reordered rows fail.

The declared-variable set is exactly the equality of
`BinderContext.free_variables` and the key sets of `variable_classes`,
`variable_roles`, `variable_sorts`, and `binder_type_facts`. Every
`BinderSourceRegistry` key and every `BinderFrame.original_var` must belong to
that set. Every existing `GeneratedOrigin.params` entry must also be declared;
otherwise the context is invalid. The allocator's used-ID union is this
declared set plus binder-source keys, frame original variables, and generated-
origin params. No term/formula or resolver/checker numeric field participates.

Each accepted `x,y` row is installed in the retained context as
`NormalizedVarClass::Free`, `NormalizedVarSort::Term`, role
`fraenkel-captured-parameter`, exact generator-binder `CoreSourceRef`, checker
provenance, and an empty type-fact vector. Core 34 remains the type/evidence
owner. The handoff revalidates these invariants, proves consecutive allocation
relative to the retained non-capture variables, and rejects any extra variable
carrying the reserved role.

For capture id `n`, the only new provenance key is exactly
`source-nested-fraenkel-capture-core-variable-v1.capture.<n>` with
`CoreProvenancePhase::Checker`. The generator `binder_range()` becomes
`CoreSourceRef::direct(range)` with exactly that one provenance entry, and the
matching `BinderSourceRecord` carries exactly one `CheckerOwnedProvenance`
entry with the same phase/key. No resolver-id text, spelling, FQN, or range is
encoded into the key.

The public producer can reach collision only defensively after a future
allocator change because coherent max-plus-one allocation cannot collide.
Current private postvalidation/helper tests cover the variant and ordering;
overflow is publicly reachable through an existing `CoreVarId(usize::MAX)`.

Ranges authenticate local provenance only; they never replace resolver binding
or owner identity. Resolver, checker, and Core numeric ID domains are never
reinterpreted. No sort, repair, inference, recovery, partial publication, or
unchecked admission is permitted.

## Scope and deferrals

Affected implementation/test paths are limited to:

- `crates/mizar-core/src/elaborator.rs`;
- `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs`;
- `crates/mizar-core/tests/lint_policy.rs` only if the generic public-API guard
  requires a task-neutral adjustment.

Owned documentation deltas are this contract pair, paired Core plan/TODO,
paired `source_family_decomposition.md`, paired `elaborator.md`, paired
source/spec, bilingual, and module-boundary audits, paired mizar-test plan,
harness, and bilingual audit, and the central coverage audit. `doc/spec`,
existing `.miz`, expectations, trace metadata,
checker source, C4C4 captured state, manifests, diagnostics, active runner
routes, and the legacy-compaction ledger are protected.

This task creates no generated parameter/application argument, term, formula,
functor, generated key, `GeneratedOrigin`, sethood result, type evidence,
semantic result, snapshot, or coverage credit. Core 35 may later consume this
handoff only after the applicable Core-33 local-binder and Core-34 type/evidence
prerequisites; it must preserve the capture order for both captured parameter
and captured argument subvectors and must never allocate, infer, reorder, or
repair them. Domain operands remain separate. Exact Core35/GeneratedOrigin
semantics and active routing remain deferred.

## Tests, baselines, and exit

Rust tests must cover the exact real C4C7 receipt, an empty context (`x,y ->
CoreVarId(0), CoreVarId(1)` despite resolver ids `1,2`), a populated context
using checked max-plus-one, deterministic replay, exact owner/source
association, retained zero-semantic context state, environment mismatch,
missing/wrong owner, allocator overflow, and public error text. Private helper
tests may cover malformed local metadata/capture rows; checker corruption stays
with the existing C4C8/Task33C tests.

Entry state is clean `1bf83e3b9275283cf7bd2f40915fc98b057fc693`, equal to
`origin/main`, with divergence `0/0` and protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`. Contract trees are `108/108` and
become `109/109`. Baseline `mizar-core` lib inventory has 155 listed tests and
raw-list SHA-256 `dd8c3a3d78413f2dae4f10019bf84e8966ebd3539d6854ef994e4825e01712c6`.
Baseline source measurements are `elaborator.rs` `17132/631992` bytes with
SHA-256 `55a74c67e1d1a1dc79134d3835f7aa9c7a1ed70c040848abb1f03f0fb6d421a7`
and the private mizar-test leaf `1008/40967` with SHA-256
`c38e7c2c99d3b81fb8906edaf90244c7d08eca913c6758bc5f7064a10bfcbcd8`.

Protected C4C7 source, expectation, and trace hashes remain respectively
`b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`,
`277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`,
and `17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`.

Exit requires independent specification/equivalence, bilingual/boundary,
test-sufficiency, implementation, and source/documentation/API reviews with no
findings; focused Core/mizar-test tests; library/lint/metadata checks; formatting;
warnings-denied workspace Clippy; full workspace tests; protected count/hash/link
checks; all autonomous hard gates `9/9`; quality at least `90/100`; exact
task-only commit; clean postcommit proof; and fresh successor inventory.

## Completion evidence

The standalone Core association, four private Core helper/postvalidation tests,
and five private real-receipt consumer tests are complete. Final Rust
measurements are:

| Path | Lines / bytes | SHA-256 |
|---|---:|---|
| `crates/mizar-core/src/elaborator.rs` | `18124 / 669642` | `65ee229c9d490f2838c4ca28864acf7b48a8fbf30e2c9b08b53dc3f7288d368d` |
| private mizar-test leaf | `1408 / 56492` | `e131f7bfdf015c820026061865d3a052542e37396de46e700571b3a97dc604ee` |

Contract trees are exactly `109/109`. Final raw library-test inventories are
`mizar-core` `159` / SHA-256
`aff91928e457018533af9bd8712b81aa1e58e58ec098fa12348fcab73d45a336`,
`mizar-test` `632` /
`a9464d8d30aed8fafc5ed0b066903ce30140bcef82fb97a500cdffed88e2b9e1`,
and unchanged checker `580` /
`269021fdb0a7b7d1f30bb4a82ffc4fa544d6224ed7ecfcd8bf27186eef254d7c`.

Independent pre-source specification/equivalence and bilingual/boundary
reviews, followed by post-source test-sufficiency, implementation, and
source/documentation/API reviews, report **NO FINDINGS** after
finding-specific repairs. Focused Core `4/4`, private real-receipt `5/5`,
Task33C `4/4` plus lint, Task33R `7/7`, Core `159/159`, mizar-test `632/632`,
lint `15/15`, metadata `137/137`, formatting, offline Cargo metadata,
warnings-denied all-target/all-feature workspace Clippy, full all-feature
workspace tests and doctests, and `git diff --check` pass.

Protected C4C7 source, expectation, and trace hashes remain exactly the three
frozen values above. Checker Task33C source and lint remain respectively
`dcff2322170389f17d4ed01e00e47ea70a07008906d9ad4358dfeca2e232a7a8` and
`3c726af3c41a0a28faf0c8ca0770a815293624ee1424ce31bd8575b97f299d30`;
the Core lint source remains
`4aea1816db81c1625b7353f4e7829528020ec2d69f054360004234ea28201103`.
No specification, existing `.miz`/expectation/trace, checker, C4C4 capture,
manifest, Typed/Resolved, diagnostic, semantic route, generated origin,
coverage credit, or Task277B state changed.

The exact task-only commit is
`774a4781ccaedfbba2b5c9ecbf7bf26b79724616`. Its 25 sorted paths have SHA-256
`64e9e45ea2e53832b8785e365c517cd1c408f4d86240eadcee78f8a92ee0f44f`.
Parent final scoring passed all hard gates `9/9` at `100/100`. The postcommit
worktree was clean; `HEAD` and `origin/main` were the task commit with divergence
`0/0`, and protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` was unchanged.

Fresh read-only inventory found no uniquely ready successor. General Core 33,
34, and 35 remain open, and Core 35 depends on both 33 and 34. The exact
generated-origin key, functor, source, sethood, and parameter/argument oracle
remain unfrozen. A second Typed/Resolved slot is forbidden, and a direct Core-35
step would violate the boundary. The remaining gaps are `design_drift`,
`source_drift`, and `test_gap`; a `spec_gap` arises only if parameter order is
made normative. No authority contradiction or `repo_metadata_conflict` was
found. Task277B remains not ready and receives zero credit.

## Next handoff

In a separate chat, begin with a read-only audit to freeze the general Core-33
source-derived context and local-binder transport for Checker 248 and
`MT10-CIR-TE`. Preserve this standalone C4C8 handoff. Do not add a second
Typed/Resolved slot, Core-34/35 semantics, generated-origin transport, or an
active route until owner, dependency, scope, API, and default-deny oracle are
unique. Use GPT-5.6 Sol `xhigh` for authority and boundary decisions and Luna
`xhigh` for frozen mechanical inventory and first-pass review. Sol remains
`xhigh` because the first decision may fix public ownership, API, and a
soundness boundary. Lower Luna only for deterministic inventory after a
representative no-regression trial; escalate cross-module precision gaps to
Terra `high` and any authority ambiguity back to Sol.
