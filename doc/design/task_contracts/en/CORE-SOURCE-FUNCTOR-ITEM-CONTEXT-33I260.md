# Task CORE-SOURCE-FUNCTOR-ITEM-CONTEXT-33I260: Task260 functor item context

> Canonical language: English. Japanese companion:
> [../ja/CORE-SOURCE-FUNCTOR-ITEM-CONTEXT-33I260.md](../ja/CORE-SOURCE-FUNCTOR-ITEM-CONTEXT-33I260.md).

Status: implementation and verification complete; exact task-only commit
pending. This is the user-selected Task-260-specific successor to Core Task
33I259. It is zero-semantic and zero-credit and does not complete Core 33 or
activate `MT10-CIR-TE`.

## Identity, authority, and decision

| Field | Contract value |
|---|---|
| Task | `CORE-SOURCE-FUNCTOR-ITEM-CONTEXT-33I260` |
| Primary owner | `mizar-core::elaborator`, Core Task 33 |
| Owning plan | [`mizar-core` crate plan](../../mizar-core/en/00.crate_plan.md) |
| Checker dependency | Existing Task-248 Profile-B `SourceBindingContextHandoff` and active Task-260 `SourceFunctorDefinitionHandoff` |
| Core dependency | Completed `CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB` handoff |
| Prepared consumer | Future `MT10-CIR-TE`, only after complete Core 33--35 lowering produces one deterministic real `CoreIr` |
| User decision | Adopt the recommended exact Task-260 family-specific two-row standalone handoff; do not generalize across definition families |
| Coverage | Zero semantic/execution credit; Task277B remains not ready and receives zero credit |

Authority remains, in order, `doc/spec/en/`, existing `.miz` sources, trace
metadata, expectations, design, then source. Chapter 10 Sections 10.1--10.6
fix ordinary functor definition forms; Chapters 11 and 12 fix current-module
identity, visibility, source order, and the enclosing definition block. The
existing Task-260 source and checker handoff authenticate two normal public
functor definitions in source order inside one normal Task-248 Profile-B item.

Existing design did not order Task-260 ahead of every other ready Core-33
family. The user's adoption supplies that ordering decision. It also selects
the two-definition/two-Core-item representation rather than an aggregate block
item or two independent handoffs. There is no `spec_gap`: this is derived phase
transport. The missing Core association/test was a bounded `design_drift` and
`test_gap`; the implementation and private consumer now close that bounded
slice without changing language semantics.
The remote baseline mismatch remains a report-only `repo_metadata_conflict`.

## Implemented public API and ownership

`crates/mizar-core/src/elaborator.rs` adds only:

- immutable `SourceFunctorCoreItemAssociation`, with getters
  `source_item()`, `definition()`, `symbol()`, and `core_item()`;
- immutable source-ordered `SourceFunctorCoreItemAssociationTable`, with
  `get(SourceFunctorDefinitionId)`, `iter()`, `len()`, and `is_empty()`;
- immutable `SourceFunctorCoreContextHandoff`, retaining by value the complete
  33LB handoff, Task-248 source context, Task-260 owner handoff, and association
  table, with getters `source_id()`, `module_id()`, `context()`,
  `source_bindings()`, `source_context()`, `checker_owner()`, `items()`, and
  non-authoritative `debug_text()`;
- non-exhaustive `SourceFunctorCoreContextError`, in precedence order:
  `EnvironmentMismatch`, `InvalidSourceBindingContext`,
  `InvalidCheckerOwner`, `InvalidCoreContext`, and
  `InvalidItemAssociation`;
- `SourceFunctorCoreContextProducer::build(
  SourceBindingCoreContextHandoff,
  SourceBindingContextHandoff,
  SourceFunctorDefinitionHandoff,
  ) -> Result<SourceFunctorCoreContextHandoff,
  SourceFunctorCoreContextError>`.

All fields are private. The producer consumes all inputs by value and publishes
only after complete postvalidation. It adds no constructor, adapter, installer,
unchecked admission, compatibility layer, `CoreContextInput`/`CoreContext`/
`CoreIr` field, or Typed/Resolved slot. The completed 33I259 public API remains
unchanged. A private shared validator is permitted only if it preserves both
exact family profiles and reduces duplication; no public generic API is in
scope.

## Cardinality, identity, order, and provenance

The admitted profile is exact:

- Task-248: exact profile `1/2/2/2/2/2/0`: one normal `DefinitionBlock`
  `SourceItemId(0)`; two ordered normal definition-parameter declarations and
  bindings; module/definition binding contexts; module/definition local type
  contexts with exact ownership, parentage, layers, visible bindings, and
  normal state; two context links; and no diagnostics;
- Task-260: definitions `0/1`, parameters `2`, guard `1`, definientia `2`, and
  correctness rows `2`, with the exact retained Task-248 fingerprint;
- definition 0 is `Equals` with no correctness row; definition 1 is `Means`,
  and correctness rows 0/1 belong only to definition 1 in
  existence/uniqueness order;
- both definitions use `BindingContextId(1)`, whose exact
  `SourceContextLink` selects the same `SourceItemId(0)`;
- Core: exactly two valid public `Functor` items, selected by the two retained
  whole `SymbolId` values, with no dependencies, diagnostics, imports,
  generated origins, or partial/recovered state; each has one pending
  `DefinitionalItem` boundary and one pending worklist entry.

The association table has exactly two rows keyed by the typed
`SourceFunctorDefinitionId` values. Iteration is Task-260 definition-table
source order `0,1`; both rows retain source item `0`, while their whole symbols
and Core item ids remain distinct. No sorting or repair occurs in this
producer. Neither checker ids nor Core ids are numerically reinterpreted.

The Core item, source-map row, worklist row, and definition boundary for each
definition use its inner definition range (`61..118`, `121..179`), never the
outer `0..261` block range, and exactly one checker provenance key:
`source-functor-core-item-v1.definition.0` or
`source-functor-core-item-v1.definition.1`. Worklist order must equal the
Task-260 definition order after identity lookup; worklist or map iteration is
never a join mechanism.

`CoreItemStatus::Valid` records only an authenticated item shell. The Task-260
correctness rows retain typed references to the two `Pending` existence/
uniqueness obligations, but the obligation rows themselves remain owned by the
originating checker projection/`TypedAst` initial-obligation table. That table
is not retained by this Core handoff. Nothing makes either item `Partial`,
creates a Core obligation, or proves/accepts a definition. Each body boundary
stays `PendingBody`.

## Default-deny oracle

Validation rejects, without sorting, repair, inference, recovery, unchecked
admission, or partial publication:

1. source/module mismatch across retained handoffs or an unequal 33LB and
   Task-248 `BindingEnv`;
2. stale Task-260 source-context fingerprint or nonexact Task-248/260
   cardinality, role, context, source order, range, origin, recovery,
   diagnostic, style, definiens, correctness, or owner state;
3. a missing, `None`, foreign, or mismatched context link/source item;
4. missing, extra, duplicate, reordered, stale, mismatched, or orphan
   association rows, including a collapsed source-item-only association;
5. missing/extra Core items or wrong whole symbol, kind, visibility, status,
   inner source range, provenance, source-map row, worklist order/state,
   dependency, diagnostic, generated-origin, or boundary state;
6. any join by display name, spelling, FQN alone, range alone, numeric id,
   shell ordinal, seed order, map iteration, or worklist iteration.

## Installation boundary and deferrals

Only the existing private Task-260 real-source test leaf constructs the two
Core item seeds from authenticated Task-260 definitions, prepares the Core
context, applies 33LB to the retained complete `BindingEnv`, and invokes the
standalone producer. It verifies the two-row shared-source-item association,
exact Core item/source/boundary/worklist state, deterministic replay, retained
33LB/environment identity, and a default-deny mutation matrix.

There is no production runner branch or installation into `TypedAst`,
`ResolvedTypedAst`, `CoreContext`, or `CoreIr`. No `.miz`, expectation, trace,
active result, diagnostic, metadata count, or coverage state changes.

Task-261--264 owner families, a generic or complete Core-33 item inventory,
Core 34/35/36 types/terms/definition bodies, parameter/argument transport,
checker obligation conversion, proof or acceptance, `GeneratedOrigin`, C4C8
composition, snapshots, `MT10-CIR-TE`, diagnostics, and Task277B remain
deferred. Task-260 correctness stays checker-owned and earns zero Core credit.

## Affected artifacts and audit impact

Source changes are exactly:

1. `crates/mizar-core/src/elaborator.rs`;
2. `crates/mizar-test/src/runner/tests/type_elaboration/source_functor_definition.rs`.

Derived documentation is limited to this paired contract; paired Core plan,
source-family decomposition, TODO, elaborator, source/spec audit, bilingual
audit, and task ledger; paired mizar-test harness and bilingual audit; and
`doc/design/spec_coverage_audit.md`. Checker documents remain unchanged because
Task 248 and Task 260 ownership/API do not change.

The central audit records only a zero-credit Core mapping and narrowed
follow-up ownership. Specification, test intent, trace status/backlinks, and
coverage credit do not change.

At freeze, `elaborator.rs` is `19986 / 741842`, SHA-256
`82971830bd539f184a69675ac502aa317be3f7ebc3ffaab118b07870444ba161`;
the Task-260 test leaf is `1674 / 61207`, SHA-256
`af20ef00e78656f94f2cae4c410c29d804e0b9b655c47615f36ae60bc2340fa3`.
The paired task-contract trees are `111/111` and become exactly `112/112`.
The implementation adds exactly two Task-260 private tests; Core library test
count stays `163`, mizar-test library tests project `634 -> 636`, and metadata
tests stay `137`. Final changed-source counts and hashes are measured once in
this contract before commit.

Protected SHA-256 values are:

- Task-260 source/expectation: `9bbf50016c72faf8b86342a9a65f8d59bf7747b85b43b6c5bc3c624c7212416a` /
  `0d67ade4d069adaa1437dc74f39a75974626567529ac46d33d7f4edb9dec6108`;
- Task-259 source/expectation: `91bdb5f51c0ea5f07bdd831700cb9803f2aa57e005921c7e4e1798ecbbf2bd9f` /
  `1ca51e4e2794cf83a6ff1bd448a60ed762eb0ed088f514ceb384457dce31c035`;
- Task-248 Profile A source/expectation: `6dbc290927264821aaaf71362105bfd67db47386d7b37b278176ad7f63483343` /
  `928729e85aee741405dfb5965d9d534f05f5bd0e5c17d662e246be156460ff0b`;
- reserve source/expectation: `9a701b2c873d41602aaf304d0a9c9dd140157a422d6c1a3cfb3d1bdb912fd660` /
  `a23067f81c550ce3aef5a9b8827672777a064dc24edd708a005c4148e1ea65c7`;
- C4C7 source/expectation: `b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc` /
  `277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`;
- trace: `17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`;
- protected stash: `f65cf4a13752ec380710814a9ac6392ccb9d75d4`.

Entry HEAD is `de42b58f7322128566326c8ee1d3d1e9a5fe4d77`; actual
`origin/main` is `a18d7373be3fe7d2bebaa96dafd1a67da4d61c4c` with
divergence `0/2`. No fetch, push, stash mutation, or metadata repair is
authorized.

## Review, verification, and exit

Independent pre-source specification/equivalence and bilingual/boundary
reviews, followed by post-source test-sufficiency, implementation, and
source/documentation/API reviews, all ended with no findings after
finding-specific repair. Focused Task-260 tests and Core/mizar-test lint
suites passed, as did formatting, offline metadata, warnings-denied Clippy,
and all-feature tests including doctests. Protected hashes/counts and
`git diff --check` also passed. Exact task-only commit and clean postcommit
inventory remain pending; the fresh postcommit inventory will select the
successor.

Exit records `9/9` autonomous hard gates and a parent score of at least
`90/100`, with exact task-only commit state, protected invariance, Task277B
not-ready/zero-credit, and a fresh read-only successor inventory.

## Completion evidence

The standalone producer and exact Task-260 private consumer are complete.
Final source measurements are `elaborator.rs` `20805 / 775898`, SHA-256
`b8ca96a9ca86078b664a2f6f2581f45f820f13b9dff20ee624adbb32e04aa22e`, and the
Task-260 test leaf `2114 / 78646`, SHA-256
`79d16c928cda605ff210166dee8d13888b33de5b0e8cb8475207558cc59a97fd`.
The paired task-contract trees are exactly `112/112`; Core library tests are
`163`, mizar-test library tests are `636` (`634 + 2`), and metadata tests are
`137`.

The pre-source specification/equivalence review found obligation ownership,
`Equals`/`Means` correctness ownership, and the Task-248 local-context profile
requiring clarification. Those findings were fixed and its final re-review
had no findings. The pre-source bilingual/boundary review's final result had
no findings. The post-source test-sufficiency review found missing status/order
evidence; `InvalidStatus` assertions were added and the sealed deterministic-
order test was cited, after which re-review had no findings. Implementation
first/final reviews had no findings. The source/documentation/API review found
missing public-enum rows and an invalid Japanese marker/link; both were fixed,
Core lint `12/12` and mizar-test lint `15/15` passed, and re-review had no
findings.

Focused Task-260 Core-context tests pass `2/2`; Core lint passes `12/12`;
mizar-test lint passes `15/15`; metadata passes `137/137`. `cargo fmt --all --
--check`, offline Cargo metadata, `cargo clippy --all-targets --all-features
-- -D warnings`, and `cargo test --all-features`, including doctests, pass.
The protected Task-260, Task-259, Task-248 Profile-A, reserve, C4C7, and trace
hashes match the frozen contract, and protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4` is unchanged.

Parent hard gates pass `9/9`. The valid uncapped score is `99/100`:
specification `20/20`, test contract `19/20`, traceability `15/15`,
implementation `15/15`, design/source synchronization `10/10`, boundary
discipline `10/10`, verification `5/5`, and handoff `5/5`; no cap applies.
Task-261--264 owner families, a generic or complete Core-33 item inventory,
Core 34/35/36 types/terms/definition bodies, parameter/argument transport,
checker obligation conversion, proof or acceptance, `GeneratedOrigin`, C4C8
composition, snapshots, `MT10-CIR-TE`, diagnostics, and Task277B remain
deferred; Task277B remains not-ready and zero-credit, and Task-260 correctness
remains checker-owned.

The report-only `repo_metadata_conflict` remains: precommit `HEAD` is
`de42b58f7322128566326c8ee1d3d1e9a5fe4d77`. During this task an external
remote-tracking-ref update changed actual `origin/main` from the entry value
`a18d7373be3fe7d2bebaa96dafd1a67da4d61c4c` (`0/2`) to that same
`de42b58f7322128566326c8ee1d3d1e9a5fe4d77` (`0/0`); its reflog records
`update by push` at `2026-08-31T16:54:05+09:00`. This agent performed no
fetch, push, stash mutation, or metadata repair. The exact task-only commit is
pending. A fresh postcommit read-only inventory, not this contract, selects
the successor.
