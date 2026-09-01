# Task CORE-SOURCE-STRUCTURE-ITEM-CONTEXT-33I263: Task263 structure item context

> Canonical language: English. Japanese companion:
> [../ja/CORE-SOURCE-STRUCTURE-ITEM-CONTEXT-33I263.md](../ja/CORE-SOURCE-STRUCTURE-ITEM-CONTEXT-33I263.md).

Status: complete on the exact task-only commit. All independent reviews ended
with no findings after repairs, all required focused and broad verification
passed, and final read-only quality passed all nine hard gates at a valid
`98/100` with no score cap. The commit hash is reported in the final handoff
because a commit cannot embed its own hash. This is a zero-semantic/zero-credit
Core-33 prerequisite; it does not complete Core 33 or activate `MT10-CIR-TE`.

## Identity, authority, and readiness

| Field | Contract value |
|---|---|
| Task | `CORE-SOURCE-STRUCTURE-ITEM-CONTEXT-33I263` |
| Primary owner | `mizar-core::elaborator`, Core Task 33 |
| Owning plan | [`mizar-core` crate plan](../../mizar-core/en/00.crate_plan.md) |
| Checker dependency | Active exact Task-263 `SourceStructureDefinitionHandoff` |
| Core precedents | Completed Tasks 33LB and 33I259--262 are protected precedents, not inputs |
| Prepared consumer | Future `MT10-CIR-TE`, only after complete Core 33--35 lowering produces deterministic real `CoreIr` |
| Coverage | Zero semantic/execution credit; Task277B remains not-ready/zero-credit |

Authority remains `doc/spec/en/`, existing `.miz`, trace metadata,
expectations, design, then source. Chapter 5 fixes structure identity,
fields-only constructor order, inheritance, root/path/view mapping, and the
mapped-member type-inclusion boundary. Chapters 11 and 12 fix current-module
symbol identity, visibility, and source order. Chapter 16 keeps correctness as
an obligation boundary. The existing 320-byte Task-263 source and checker
handoff authenticate exact `2/4/1/2/0` transport.

There is no `spec_gap`. The missing Core association and private consumer are
bounded `design_drift` and `test_gap`. The user selected the bounded derived-to-
base local dependency: the Core `Task263Derived` item records its one local
dependency on the Core `Task263Base` item. Task-248 and 33LB are absent because
Task 263 has no parameter or source-context payload. Task 264 property
implementation remains a later distinct owner.

## Frozen public API and ownership

`crates/mizar-core/src/elaborator.rs` may add only:

- immutable `SourceStructureCoreItemAssociation`, with getters `definition()`,
  `symbol()`, and `core_item()`;
- immutable source-ordered `SourceStructureCoreItemAssociationTable`, with
  `get(SourceStructureDefinitionId)`, `iter()`, `len()`, and `is_empty()`;
- immutable `SourceStructureCoreContextHandoff`, retaining by value the
  prepared `CoreContext`, exact Task-263 checker handoff, and association table,
  with getters `source_id()`, `module_id()`, `context()`, `checker_owner()`,
  `items()`, and non-authoritative `debug_text()`;
- non-exhaustive `SourceStructureCoreContextError`, in precedence order:
  `EnvironmentMismatch`, `InvalidCheckerOwner`, `InvalidCoreContext`, and
  `InvalidItemAssociation`;
- `SourceStructureCoreContextProducer::build(CoreContext,
  SourceStructureDefinitionHandoff) ->
  Result<SourceStructureCoreContextHandoff,
  SourceStructureCoreContextError>`.

All fields are private. The producer consumes both inputs by value and
publishes only after complete postvalidation. It adds no generic definition
adapter, constructor, installer, compatibility layer, unchecked admission,
`CoreContextInput`/`CoreContext`/`CoreIr` field, or Typed/Resolved slot. It does
not alter 33LB or Tasks 33I259--262.

`SourceStructureCoreContextError` is a downstream forward-compatible public
surface and must remain `#[non_exhaustive]`. The synchronized EN/JA public-enum
policy tables and source/spec public API inventories must add the exact new
enum and all five API groups before Rust source is edited.

## Exact profile, identity, dependency, and provenance

- Source: existing 320-byte final-LF fixture, SHA-256
  `078eaee4b17341c9d8ebeb8a1f631ca984873bd07eb4e5d9c1a9486b39ac6671`.
- Task 263: definitions/members/inheritances/mappings/coherence requests are
  exactly `2/4/1/2/0`; the source-type fingerprint is nonempty and the base
  initial-obligation count is zero.
- Definition 0 is `Task263Base`: whole Structure symbol, resolver definition
  0/contribution 0, site 57, range `13..98`, ordinal 0, normal, members
  `[0,1]`, constructor fields `[0]`, origin `[4,0,11,0]`, and exact spelling.
- Definition 1 is `Task263Derived`: whole Structure symbol, resolver definition
  3/contribution 0, site 65, range `102..190`, ordinal 1, normal, members
  `[2,3]`, constructor fields `[2]`, origin `[4,0,11,1]`, and exact spelling.
- Members 0--3 remain checker-owned field/property rows. Their owner/ordinal/
  kind/site/range/written-type/constructor-ordinal profiles are exactly
  `0/0/Field/53/42..63/0/Some(0)`,
  `0/1/Property/56/68..91/1/None`,
  `1/0/Field/61/134..155/2/Some(0)`, and
  `1/1/Property/64/160..183/3/None`.
- Inheritance 0 is exactly child definition 1, parent definition 0, site 70,
  range `194..314`, ordinal 0, normal, mappings `[0,1]`, and exact spelling.
  Mappings 0/1 preserve Field/Property, view-parent-root `2/0/0` and `3/1/1`,
  direct path `[0]`, sites 68/69, ranges `247..274`/`279..307`, and exact
  spellings. They authorize validation only, not Core member or view nodes.
- Core has exactly two valid public `Structure` items. Base has no dependency;
  Derived has exactly one local dependency on Base. There are no external or
  missing dependencies, diagnostics, imports, generated origins, obligation
  seeds, binders, partial/recovered states, or other nodes. Each item has one
  pending `DefinitionalItem` boundary and one pending worklist entry.

The association table has two rows keyed by typed
`SourceStructureDefinitionId(0/1)`. Each Core item is selected only through
exact whole-`SymbolId` registry lookup. Checker, resolver, and Core numeric ids
are never reinterpreted; names, FQN alone, ranges, seed order, maps, and
worklists are not joins. Item, source-map row, boundary, and worklist use each
definition's inner range and one checker provenance key:
`source-structure-core-item-v1.definition.0` or
`source-structure-core-item-v1.definition.1`.

The direct inheritance row is the sole authority for the Derived-to-Base Core
dependency. No member dependency is created. Fields-only constructor order,
member mappings, identical bare-`set` types, and zero coherence requests are
revalidated but not lowered. No constructor, selector, update, reduct,
coherence goal, fact, obligation, proof, discharge, or acceptance is produced.

## Default-deny oracle and installation boundary

Validation rejects without sorting, repair, inference, recovery, unchecked
admission, or partial publication:

1. any Core/checker source or module mismatch;
2. nonexact Task-263 cardinality, base count, empty source-type fingerprint,
   resolver identity/origin, or any definition/member/inheritance/mapping row;
3. missing/extra/duplicate/reordered/stale/mismatched/orphan association rows;
4. missing/extra Core items or wrong symbol, kind, visibility, status, inner
   source, provenance, source-map domain, diagnostic, generated-origin,
   boundary, or worklist state;
5. any Base dependency, any Derived dependency other than the one local Base
   item, or any external/missing dependency; and
6. name-, range-, numeric-, seed-, map-, or worklist-order joins.

Only the existing private Task-263 real-source test leaf may derive the two
Core `Structure` seeds from authenticated definitions, attach the one
Derived-to-Base symbol dependency from authenticated inheritance 0, prepare
the Core context, and call the standalone producer. Exactly two new private
tests verify retained inputs, two associations, local dependency, item/source-
map/boundary/worklist state, deterministic replay, Core mutations, and foreign
environment rejection.

There is no production runner branch or installation into Typed, Resolved,
CoreContext, or CoreIr. No `.miz`, expectation, trace, active result,
diagnostic, metadata count, or coverage state changes. Task 264, generic or
complete Core 33, Core 34 structure/member/type/view semantics, Core 35 terms
and formulas, Core 36 constructors/body/correctness/obligations, proof/
discharge/acceptance, `GeneratedOrigin`, snapshots, `MT10-CIR-TE`, diagnostics,
and Task277B remain deferred. Task 263 earns zero Core credit.

## Artifacts, baselines, reviews, and exit

Source changes are exactly:

1. `crates/mizar-core/src/elaborator.rs`;
2. `crates/mizar-test/src/runner/tests/type_elaboration/source_structure_definition.rs`.

Derived docs are this paired contract; paired Core plan, decomposition, TODO,
elaborator, source/spec audit, bilingual audit, and ledger; paired mizar-test
harness and bilingual audit; and `doc/design/spec_coverage_audit.md`. Checker
docs remain unchanged. The central audit records zero-credit mapping and
follow-up ownership only.

At freeze, `elaborator.rs` is `22350 / 839135`, SHA-256
`3fe6e32d621f6516b54a67fd7649e6504b619c3e5e570ed26143060b5e849510`;
the Task-263 test leaf is `218 / 8495`, SHA-256
`144bb7b9e98d7a9ae7b1824a4b6a489b840efe54b11fdcbe8f202a2b9d2816b0`.
Task-contract trees are `114/114 -> 115/115`. Core library tests stay `163`;
mizar-test library tests project `640 -> 642`; metadata stays `137`.

Protected values include the Task-263 source/expectation hashes
`078eaee4b17341c9d8ebeb8a1f631ca984873bd07eb4e5d9c1a9486b39ac6671` /
`d82c8d3102ea34fdb4a32792167c4b109b96b9c05265d3f04e6310278178e8ac`,
trace `17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`,
and stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4`.

Entry `HEAD` is the separately reviewed autonomy-policy commit
`74208cf797f2a9a24716f5b93d2189986f111109`; `origin/main` is
`5c0488382989af76ef13a281a81d5630ee7eff68`, with divergence `0/1` before the
Task-263 worktree edits. No fetch, push, or stash mutation is authorized.

Before source edits, independent specification/equivalence and bilingual/
boundary reviews must end with no findings. After source edits, independent
test-sufficiency, implementation, and source/documentation/API reviews must
end with no findings after repair. Verification runs the Task-263 probes,
checker route, protected 33LB/259--262 probes, lint/metadata, fmt, offline
metadata, full warnings-denied Clippy, and all-feature tests including
doctests. Exit requires hard gates `9/9`, read-only score `>=90/100`, exact
task-only commit, clean postcommit proof, protected invariance, Task277B
not-ready/zero-credit, and fresh successor inventory.

## Completion evidence and next handoff

The standalone producer and exactly two private Task-263 tests are complete.
Current source measurements are `elaborator.rs` `22947 / 862541`, SHA-256
`e9ea1d6eabb191d7d3b8c22fe1fc11626d2e0dab86690dee662f851bb487f85c`, and the
Task-263 test leaf `731 / 28867`, SHA-256
`085116fa94e344eb353084c5f5511f3a007cd9a9168277dc995a5ca4ef86ec80`.
The paired task-contract trees are `115/115`; Core library tests remain `163`,
mizar-test library tests are `642`, and metadata remains `137`.

Pre-source review repaired the public-enum policy and API inventory, then ended
with no findings. Test-sufficiency review repaired inheritance-derived test
setup, retained-context equality, exact source/boundary/worklist assertions,
and the default-deny matrix, then ended with no findings. Implementation
review repaired forbidden binder-state admission, then ended with no findings.
Focused Task-263 Core probes pass `2/2`; the complete Task-263 route passes
`6/6`; protected item-context probes pass `10/10`; checker Task-263 passes
`5/5`; Core/mizar-test library tests pass `163/163` and `642/642`; Core and
mizar-test lint pass `12/12` and `15/15`; focused warnings-denied Clippy and
`git diff --check` pass. Protected source/expectation/trace hashes and stash
match the frozen values.

Final source/documentation/API review ended with no findings. `cargo fmt --all
-- --check`, offline metadata, full warnings-denied Clippy, `cargo test
--all-features` including doctests, and metadata `137/137` all pass. Independent
final quality found no blocking/high/medium findings; all nine hard gates pass
at a valid `98/100` with no score cap. Exact staging, the task-only commit, and
clean postcommit/fresh-successor inventory are the remaining transactional
steps and do not reopen semantic acceptance. The next dependency-ordered
candidate is Task 264 property implementation item context, subject to fresh
authority and readiness inventory; generic Core-33 installation remains
deferred until all owner families and its separate contract are ready.
