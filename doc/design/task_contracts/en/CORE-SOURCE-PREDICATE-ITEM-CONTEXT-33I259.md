# Task CORE-SOURCE-PREDICATE-ITEM-CONTEXT-33I259: Task259 predicate item context

> Canonical language: English. Japanese companion:
> [../ja/CORE-SOURCE-PREDICATE-ITEM-CONTEXT-33I259.md](../ja/CORE-SOURCE-PREDICATE-ITEM-CONTEXT-33I259.md).

Status: implementation and verification complete; exact task-only commit
pending. This is the dependency-minimal first
nonempty source-item/Core-item association within Core Task 33. It is
zero-semantic and zero-credit and does not complete Core 33 or activate
`MT10-CIR-TE`.

## Identity, authority, and decision

| Field | Frozen value |
|---|---|
| Task | `CORE-SOURCE-PREDICATE-ITEM-CONTEXT-33I259` |
| Primary owner | `mizar-core::elaborator`, Core Task 33 |
| Owning plan | [`mizar-core` crate plan](../../mizar-core/en/00.crate_plan.md) |
| Checker dependency | Existing Task-248 Profile-B `SourceBindingContextHandoff` and active Task-259 `SourcePredicateDefinitionHandoff` |
| Core dependency | Completed `CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB` handoff |
| Prepared consumer | Future `MT10-CIR-TE`, only after complete Core 33--35 lowering produces one deterministic real `CoreIr` |
| User decision | Adopt a checker-authenticated owner link consumed by a standalone immutable Core handoff; do not add a public Core input/field |
| Coverage | Zero semantic/execution credit; Task277B remains not ready and receives zero credit |

Authority remains, in order, `doc/spec/en/`, existing `.miz` sources, trace
metadata, expectations, design, then source. Chapters 4, 11, and 12 fix lexical
scope, declaration/source order, current-module identity, and the definition
block boundary. The existing Task-259 source and handoff authenticate the first
nonempty exact owner slice: one normal public predicate definition with a full
`SymbolId` in the one normal Task-248 Profile-B definition context.

Task 259 already proves the complete identity chain without a new checker
adapter: `SourcePredicateDefinition.context()` selects the exact
`SourceContextLink`, that link carries one `SourceItemId`, and the definition's
whole `SymbolId` selects one existing Core item. Source range and provenance
validate the selected identities but never perform the join.

This closes bounded `design_drift`, `source_drift`, and `test_gap` for the exact
Task-259 association. There is no `spec_gap`: the association is derived phase
transport, not new language behavior. The actual remote metadata remains a
report-only `repo_metadata_conflict` against the earlier requested remote
baseline and is not repaired.

## Frozen public API and ownership

`crates/mizar-core/src/elaborator.rs` adds:

- immutable `SourcePredicateCoreItemAssociation`, with getters
  `source_item()`, `definition()`, `symbol()`, and `core_item()`;
- immutable source-ordered `SourcePredicateCoreItemAssociationTable`, with
  `get(SourcePredicateDefinitionId)`, `iter()`, `len()`, and `is_empty()`;
- immutable `SourcePredicateCoreContextHandoff`, retaining by value the
  complete 33LB handoff, Task-248 source context, Task-259 owner handoff, and
  association table, with getters `source_id()`, `module_id()`, `context()`,
  `source_bindings()`, `source_context()`, `checker_owner()`, `items()`, and
  non-authoritative `debug_text()`;
- non-exhaustive `SourcePredicateCoreContextError`, in precedence order:
  `EnvironmentMismatch`, `InvalidSourceBindingContext`,
  `InvalidCheckerOwner`, `InvalidCoreContext`, and
  `InvalidItemAssociation`;
- `SourcePredicateCoreContextProducer::build(
  SourceBindingCoreContextHandoff,
  SourceBindingContextHandoff,
  SourcePredicateDefinitionHandoff,
  ) -> Result<SourcePredicateCoreContextHandoff,
  SourcePredicateCoreContextError>`.

All fields are private. The producer consumes all three inputs by value and
publishes only after complete postvalidation. It adds no constructor, adapter,
installer, unchecked admission, `CoreContextInput`/`CoreContext`/`CoreIr`
field, or Typed/Resolved slot.

## Cardinality, identity, order, and provenance

This task admits exactly the existing Task-259 profile:

- Task-248 source context: one normal `DefinitionBlock` item, two normal
  ordered definition-parameter declarations/bindings, two context links, and
  no diagnostic or recovery state;
- Task-259 owner: one normal predicate definition, two parameters, one guard,
  one property, and one correctness row, with a fingerprint equal to the
  retained source context;
- Core: exactly one valid public `Predicate` item for the retained whole
  `SymbolId`, no extra or missing Core item, no item diagnostic or dependency,
  and one pending definitional boundary.

The association has exactly one row. Its identities are the Task-259
`SourcePredicateDefinitionId`, the Task-248 `SourceItemId` selected through the
definition's exact `BindingContextId`/`SourceContextLink`, the retained whole
`SymbolId`, and the `CoreItemId` selected by exact Core registry lookup.
Iteration follows the Task-259 definition-table order; no sorting occurs.

The Core item and definition boundary use the exact Task-259 definition range
and one checker provenance key
`source-predicate-core-item-v1.definition.0`. The containing Task-248 source
item remains the outer definition block; its range is not substituted for the
predicate definition range.

## Default-deny oracle

Validation rejects, without sorting, repair, inference, recovery, or partial
publication:

1. source/module mismatch across any retained handoff;
2. a 33LB `BindingEnv` unequal to the Task-248 environment;
3. stale Task-259 source-context fingerprint or nonexact Task-248/259
   cardinality, recovery, diagnostic, role, context, or owner state;
4. missing, extra, duplicate, reordered, stale, mismatched, or orphan
   association rows;
5. a missing/`None`/foreign context link or a link whose binding context does
   not equal the Task-259 definition context;
6. missing/extra Core items, wrong full `SymbolId`, kind, visibility, status,
   source, provenance, source-map entry, worklist state, dependency,
   diagnostic, or definition-boundary state;
7. any join by display name, spelling, FQN alone, range alone, numeric index,
   shell ordinal, Core seed order, or map iteration.

## Installation boundary and deferrals

The existing private Task-259 real-source test constructs the Core item seed
only from the authenticated Task-259 definition, prepares the Core context,
applies 33LB to the retained complete `BindingEnv`, then invokes this producer.
It verifies the exact one-row chain and deterministic replay plus the
default-deny Core/input matrix. The `.miz`, expectation, trace row, active
runner selection, diagnostics, and coverage status remain unchanged.

This task does not claim a generic multi-definition-block association. A block
containing multiple predicate/functor/mode/attribute/structure definitions,
other Task-259+ owner families, reserve/property items, and any aggregation
policy remain separate descendants. Core 34/35/36 semantics, type/fact/term/
formula/definition bodies, parameters as semantic arguments, `GeneratedOrigin`,
C4C8, active snapshot installation, diagnostics, and Task277B are deferred.

## Affected artifacts and audit impact

Source changes are limited to:

1. `crates/mizar-core/src/elaborator.rs`;
2. `crates/mizar-test/src/runner/tests/type_elaboration/source_predicate_definition.rs`.

Derived documentation changes are limited to this paired contract; the paired
Core plan, source-family decomposition, TODO, elaborator, source/spec audit,
bilingual audit, and task ledger; the paired mizar-test harness and bilingual
audit; and `doc/design/spec_coverage_audit.md`. No checker document changes
because Task 248 and Task 259 APIs/ownership remain unchanged.

The central audit records a zero-credit Core-33 mapping and narrows the open
general-item follow-up; it does not change any specification, test, trace, or
coverage status.

At freeze, `elaborator.rs` is `19323 / 715066` with SHA-256
`2de75000b5a5fd280d7b1ba313b78551640c28e688f9bd36bf02b102e8129f7b`;
the Task-259 test leaf is `517 / 17989` with SHA-256
`95eca63c134d2a367e35f4feb277ff0f9bc4197ea254cc42e0445e383312b201`.
The paired contract trees are `110/110` and become exactly `111/111`.

Protected SHA-256 values are:

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

Entry HEAD is `9795ca073e081c23193cb7d51411fa00fddcfd6b`;
actual `origin/main` is `a18d7373be3fe7d2bebaa96dafd1a67da4d61c4c`
with divergence `0/1`. No fetch, push, or metadata repair is authorized.

## Review, verification, and exit

Before source edits, independent specification/equivalence and bilingual/
boundary reviews must end with no findings. Post-source independent test-
sufficiency, implementation, and source/documentation/API reviews must end with
no findings after finding-specific repair.

Focused checks cover the new Core producer and exact real Task-259 consumer,
then Core and mizar-test lint suites. Required broad checks are
`cargo fmt --all -- --check`, offline Cargo metadata,
`cargo clippy --all-targets --all-features -- -D warnings`, and
`cargo test --all-features`, followed by protected hash/count, diff, exact
staging, cached-diff, commit, and clean postcommit checks.

Exit requires all `9/9` autonomous hard gates, a parent score of at least
`90/100`, exact task-only staging/commit, protected invariance, Task277B
not-ready/zero-credit, and a fresh read-only successor inventory.

## Completion evidence

The standalone producer and exact Task-259 private consumer are complete.
Final source measurements are `elaborator.rs` `19986 / 741842`, SHA-256
`82971830bd539f184a69675ac502aa317be3f7ebc3ffaab118b07870444ba161`,
and the Task-259 test leaf `877 / 31757`, SHA-256
`309ef24a97f8d55212fea6c655bab1a7374f7b120dd4afe9e70fa0e0885cd4a9`.
Core library tests remain `163`; mizar-test library tests are `634` (`632 +
2`), and the paired contract trees are exactly `111/111`.

The pre-source specification/equivalence review had no findings. The initial
bilingual/boundary review found two JA contract synchronization omissions;
after repair its finding-specific re-review had no findings. Post-source
implementation and source/documentation/API reviews found the same prohibited
iteration-selected Core item; exact whole-`SymbolId` registry lookup replaced
it. Test-sufficiency review found missing 33LB-retention, item/boundary, and
source-map assertions. All were repaired, and all three finding-specific
re-reviews ended with no findings.

Focused Task-259 Core-context tests pass `2/2`; Core lint passes `12/12`;
mizar-test lint passes `15/15`; metadata passes `137/137`. `cargo fmt --all --
--check`, offline Cargo metadata, `cargo clippy --all-targets --all-features --
-D warnings`, and `cargo test --all-features`, including doctests, pass. Every
protected Task-259, Task-248, reserve, C4C7, and trace hash reproduces the
frozen value above, and the protected stash remains
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`.

Parent review passes the autonomous hard gates `9/9`. The valid uncapped score
is `99/100`: specification `20/20`, test contract `19/20` (immutable upstream
handoffs rely on their existing producer mutation suites), traceability
`15/15`, implementation `15/15`, design/source synchronization `10/10`,
boundary discipline `10/10`, verification `5/5`, and handoff `5/5`. No score
cap applies. Multi-definition association, Task-260+, Core 34/35/36,
`GeneratedOrigin`, `MT10-CIR-TE`, active routes, diagnostics, coverage credit,
and Task277B remain deferred/not-ready with zero credit.

The report-only `repo_metadata_conflict` remains: precommit `HEAD` is
`9795ca073e081c23193cb7d51411fa00fddcfd6b`, actual `origin/main` is
`a18d7373be3fe7d2bebaa96dafd1a67da4d61c4c`, and divergence is `0/1`, rather
than the earlier requested remote `774a4781` state. No fetch, push, or metadata
repair was attempted. A fresh postcommit inventory, not this contract,
determines whether a dependency-minimal successor is uniquely ready.
