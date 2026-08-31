# Task CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB: standalone source local-binder context

> Canonical language: English. Japanese companion:
> [../ja/CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB.md](../ja/CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB.md).

Status: implementation and verification complete; exact task-only commit
pending. This is a zero-semantic, zero-credit
prerequisite within Core Task 33. It does not complete Core 33 or activate
`MT10-CIR-TE`.

## Identity, authority, and decision

| Field | Frozen value |
|---|---|
| Task | `CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB` |
| Primary owner | `mizar-core::elaborator`, Core Task 33 |
| Owning plan | [`mizar-core` crate plan](../../mizar-core/en/00.crate_plan.md) |
| Checker dependency | Existing immutable `mizar_checker::binding_env::BindingEnv`; the reserve-only Task-20 route owns one directly and Checker Task 248 exposes one through `SourceBindingContextHandoff::binding_env()` |
| Prepared consumer | Future `MT10-CIR-TE`, only after the remaining Core 33 item association and applicable Core 34/35 payloads can produce one complete real `CoreIr` |
| User decision | Adopt the standalone Core-33 local-binder prerequisite rather than extending `CoreContextInput`, `CoreContext`, `CoreIr`, Typed, or Resolved |
| Coverage | Zero semantic/execution credit; broad Core rows and Task277B remain deferred/not-ready |

Authority remains, in order, `doc/spec/en/`, existing `.miz` tests, trace
metadata, expectations, design, and source. Chapter 4 defines `reserve` as a
module default context rather than a variable declaration. The existing
reserve-only expectation nevertheless requires binder-only `CoreContext`
readiness with no Core item or semantic result. Checker Task 248 authenticates
module reserve/default and declaration-local parameter identities, contexts,
visibility, source order, and structural shadowing. Treating those identities
as zero-semantic Core context transport is consistent with both authorities;
it must not be interpreted as quantification, closure, a Core item, or a fact.

The pre-freeze audit classified the current runner construction
`CoreVarId::new(binding_id.index())` as `boundary_violation` and `source_drift`.
Checker and Core numeric domains are distinct. The absence of a general
source-item-to-`SymbolId`/`CoreItemId` owner bridge and the absence of
module-level context tables in `CoreIr` remain `design_drift`; the absent first
real `MT10-CIR-TE` baseline remains `test_gap`. The observed
`origin/main == HEAD` state, rather than the requested one-commit divergence,
is a report-only `repo_metadata_conflict` and is not repaired by this task.

## Frozen public API and ownership

`crates/mizar-core/src/elaborator.rs` adds:

- immutable `SourceBindingCoreVariable` with getters `binding()`,
  `core_var()`, and no public constructor;
- immutable `SourceBindingCoreVariableTable` in exact checker `BindingTable`
  iteration order, with `get(BindingId)`, `iter()`, `len()`, and `is_empty()`;
- immutable `SourceBindingCoreContextHandoff`, retaining the updated
  `CoreContext`, the complete checker `BindingEnv`, and the association table,
  with getters `source_id()`, `module_id()`, `context()`, `binding_env()`,
  `variables()`, and non-authoritative `debug_text()`;
- non-exhaustive `SourceBindingCoreContextError` with variants, in precedence
  order: `EnvironmentMismatch`, `InvalidCoreContext`,
  `InvalidBindingEnvironment`, `CoreVariableAllocationOverflow`,
  `CoreVariableCollision { var: CoreVarId }`, and
  `InvalidBindingAssociation`;
- `SourceBindingCoreContextProducer::build(
  context: CoreContext,
  binding_env: BindingEnv,
  ) -> Result<SourceBindingCoreContextHandoff,
  SourceBindingCoreContextError>`.

The producer consumes both inputs by value and publishes only a completely
validated handoff. Table, row, and handoff fields are private. There is no
installer, adapter, unchecked constructor, numeric-ID conversion, mutable
public field, second Typed/Resolved slot, or `CoreContextInput`/`CoreIr` field.

The exact error messages are:

- `source binding Core context environment is invalid`;
- `source binding Core context is invalid`;
- `source binding environment is invalid for Core context transport`;
- `source binding Core variable allocation overflowed`;
- `source binding Core variable <index> collides`;
- `source binding Core variable association is invalid`.

## Cardinality, order, allocation, and provenance

The admitted checker payload is nonempty and complete. It has no diagnostics,
recovered/degraded contexts or bindings, binding diagnostics, captured-free
variable rows, or unsupported binding kinds. This prerequisite accepts only:

- `ReservedVariable` + `ReservedVariable` identity + `Reserved` status in the
  normal module context; and
- `DefinitionParameter` + exact `ResolverLocal` identity + `Active` status in
  a normal declaration context.

The binding identity, spelling, declaration range, visible ordinal, owner
context, local scope, and type site remain checker-owned. Core neither copies
display names into identity keys nor reconstructs source items. The association
has exactly one row per checker binding and iterates in the existing checker
`BindingTable::iter()` order. Core must not sort by display name, range, map
iteration, or a reconstructed ordinal.

Core allocates consecutive snapshot-local `CoreVarId`s beginning at checked
`max(all existing Core variable identities) + 1`, or zero when the context is
empty. Existing declared variables, binder sources, binder frames, type-fact
keys, and generated-origin parameters participate in the used-ID validation.
No checker or resolver numeric value participates in allocation.

Each admitted row is installed as `NormalizedVarClass::Free`,
`NormalizedVarSort::Term`, empty type facts, and role `reserved-variable` or
`definition-parameter`. This is transport metadata only. For checker binding
id `n`, the exact provenance key is
`source-binding-core-variable-v1.binding.<n>` at checker phase. The binding's
declaration range is the exact direct source anchor for both the binder source
and its single matching checker-owned provenance entry.

## Default-deny oracle

Validation rejects, without sorting, repair, inference, recovery, or partial
publication:

1. source/module mismatch between Core and checker payloads;
2. incoherent existing Core variable, binder-source, binder-frame,
   type-fact, or generated-origin references;
3. empty, diagnostic-bearing, recovered, degraded, captured, unsupported, or
   identity/status/context-mismatched checker binding state;
4. allocation overflow or collision;
5. missing, extra, duplicate, reordered, stale, mismatched, or orphan
   association rows;
6. wrong role, class, sort, source range, provenance, or nonempty type facts;
7. any reserved source-binding Core role outside the authenticated table.

`BindingEnv::try_new` and Checker Task 248 remain the sole validators of their
private construction invariants. Core validates its admission subset and its
own complete association; it does not duplicate or weaken the checker oracle.

## Installation boundary and deferrals

The reserve-only Task-20 runner replaces its caller-built variable/binder seeds
with this producer and validates the returned association by `BindingId`.
Checker Task 248 Profile A adds a private real-source consumer proving the exact
two-row reserve/local-parameter association, structural shadow distinction,
fresh Core allocation, deterministic replay, and zero-semantic context state.
Existing `.miz`, expectations, trace metadata, and coverage status remain
unchanged.

A future `MT10-CIR-TE` producer may consume the Task-248 `BindingEnv` through
this handoff before completing the remaining Core 33 item association and Core
34/35 lowering. This handoff itself is not serialized and cannot activate a
snapshot: `CoreIr` has no module-level context table, and the prepared consumer
requires complete deterministic `CoreIr::debug_text()` bytes. The first active
baseline must be separately frozen with a real source and its complete
Core33--35 payload.

The standalone C4C8 handoff remains unchanged and separate. If both are used,
the local-binder producer runs first and C4C8 may extend its resulting
`CoreContext`; neither handoff is installed into the other. Source-item/Core-item
association, types/evidence, terms/formulas, parameters/arguments,
`GeneratedOrigin`, diagnostics, active routes, snapshots, Core 34/35 semantics,
and Task277B readiness remain deferred.

## Scope, baselines, reviews, and exit

Implementation/test paths are limited to:

- `crates/mizar-core/src/elaborator.rs`;
- `crates/mizar-test/src/runner/type_elaboration/checker_handoff.rs`;
- `crates/mizar-test/src/runner/tests/type_elaboration/source_context.rs`.

Owned documentation changes are this contract pair; paired Core plan, TODO,
task-ledger, source-family, elaborator, source/spec audit, and bilingual-audit entries;
paired mizar-test harness and bilingual-audit entries; and the central coverage
audit. No specification, `.miz`,
expectation, trace row, checker source, C4C4/C4C8 state, diagnostic registry,
manifest, active route, or legacy-compaction record may change.

Entry source baselines are `elaborator.rs` `18124/669642` bytes SHA-256
`65ee229c9d490f2838c4ca28864acf7b48a8fbf30e2c9b08b53dc3f7288d368d`,
reserve runner `1299/50651`
`da96e0353d3f31feb326c4a3721d61aedbb175e4918a3e8554b43f8c8ea0622a`,
and Task-248 private leaf `3178/118223`
`f92e45ffe85dae739360c95904865c3a172a8d3f24c14baec02e70878e114814`.
The contract trees are `109/109` and become `110/110`. The existing Core lib
test inventory is `159`; four focused Core tests are expected, while existing
mizar-test tests receive additional assertions without changing their count.

Protected reserve source/expectation hashes are
`9a701b2c873d41602aaf304d0a9c9dd140157a422d6c1a3cfb3d1bdb912fd660` /
`a23067f81c550ce3aef5a9b8827672777a064dc24edd708a005c4148e1ea65c7`;
Task-248 Profile-A hashes are
`6dbc290927264821aaaf71362105bfd67db47386d7b37b278176ad7f63483343` /
`928729e85aee741405dfb5965d9d534f05f5bd0e5c17d662e246be156460ff0b`;
protected C4C7 hashes remain those frozen by 33C4C8; trace SHA-256 is
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`.
The protected stash is
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`.

Exit requires independent specification/equivalence and bilingual/boundary
reviews before source work; focused Core and real-source tests; independent
test-sufficiency, implementation, and source/documentation/API reviews; Core,
mizar-test, and metadata lint; formatting; warnings-denied all-target/all-feature
Clippy; full workspace tests; protected count/hash/status checks; all `9/9`
hard gates; parent quality at least `90/100`; exact task-only commit; clean
postcommit proof; and fresh successor inventory.

## Completion evidence

The standalone producer, immutable association handoff, and the two existing
private consumers are complete. Final source measurements are
`elaborator.rs` `19323/715066` bytes SHA-256
`2de75000b5a5fd280d7b1ba313b78551640c28e688f9bd36bf02b102e8129f7b`,
the reserve runner `1285/50148`
`f62fa1db0e9e9b7e20cbfb39529eb1138c647c2eafb18623fcabfeb09f4c8186`,
and the Task-248 private leaf `3290/122501`
`46ca0d25d4d39ff420489a19ac03ca7c571b6b1e7baa6f363e3c91aa6a8fa1c2`.
Core library tests are `163` (`159 + 4`), mizar-test library tests remain
`632`, and the paired contract trees are `110/110`.

Independent pre-source specification/equivalence, API-feasibility, and
bilingual/boundary reviews ended with no findings. Post-source implementation
review ended with no findings. Test-sufficiency and source/documentation/API
reviews found only in-scope test-matrix and derived-document drift; after the
unsafe opaque-ID fixture was removed and the synchronized fixes were applied,
both finding-specific re-reviews ended with no findings.

Focused Core `4/4`, real Task-248 `1/1`, reserve and C4C8 no-regression probes,
Core lint `12/12`, mizar-test lint `15/15`, metadata `137/137`, formatting,
offline metadata, warnings-denied all-target/all-feature workspace Clippy, and
full all-feature workspace tests and doctests pass. The protected reserve,
Task-248, C4C7, and trace hashes reproduce every frozen value above; the stash
remains `f65cf4a13752ec380710814a9ac6392ccb9d75d4`.

Parent review passes all autonomous hard gates `9/9`. The valid uncapped score
is `99/100`: specification `20/20`, test contract `19/20` (the unavailable
public shared 33LB-to-C4C8 receipt fixture remains a non-blocking integration
residual), traceability `15/15`, implementation `15/15`, design/source sync
`10/10`, boundary discipline `10/10`, verification `5/5`, and handoff `5/5`.
No score cap applies. General Core 33 item association, Core 34/35,
`GeneratedOrigin`, the first real `MT10-CIR-TE`, and Task277B remain explicitly
deferred with zero credit. The report-only `repo_metadata_conflict` remains:
before this task-only commit, actual `origin/main` and `HEAD` were both
`a18d7373be3fe7d2bebaa96dafd1a67da4d61c4c`, not the requested remote
`774a4781` state; no fetch, push, or metadata repair was attempted.
