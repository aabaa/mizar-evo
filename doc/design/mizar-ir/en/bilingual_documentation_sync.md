# mizar-ir Bilingual Documentation Sync

> Canonical language: English. Japanese companion:
> [../ja/bilingual_documentation_sync.md](../ja/bilingual_documentation_sync.md).

## Scope

This task-17 audit compares every English canonical document under
`doc/design/mizar-ir/en/` with the matching Japanese companion under
`doc/design/mizar-ir/ja/`. It checks filename parity, heading parity, task
status parity, gap classification parity, and the crate boundary statements
that keep `mizar-ir` from owning proof authority, `mizar-cache` identity, or
downstream driver/diagnostic integration.

The audit found no unresolved blocking/high bilingual drift. No English
canonical content was left without a Japanese companion.

## Audit Checks

| Check | Evidence | Result |
|---|---|---|
| File coverage | Both language directories contain `00.crate_plan.md`, `cache_adapter.md`, `identity.md`, `projection.md`, `publisher.md`, `source_spec_correspondence.md`, `storage.md`, and `todo.md`; this audit adds `bilingual_documentation_sync.md` to both. | Synchronized. |
| Heading parity | Paired heading counts match for each pre-existing file: crate plan 27, cache adapter 13, identity 10, projection 15, publisher 11, source/spec audit 6, storage 12, TODO 11. This audit pair has matching sections. | Synchronized. |
| Task status parity | TODO task states are aligned: tasks 1-17 complete; tasks 18-19 remain open. | Synchronized. |
| Gap classification parity | Crate-plan `IR-G-001` through `IR-G-009` and source/spec `IR-G-004` through `IR-G-009` appear in both languages with matching classes and dispositions. | Synchronized. |
| Ownership boundaries | Both language sets state that `mizar-ir` owns internal IR storage, snapshot handles, phase output references, publication, cache adaptation, projection, and snapshot replacement, but not proof acceptance, trusted status, kernel acceptance, `CacheKey`, dependency fingerprints, or proof-reuse validation. | Synchronized. |
| External dependency gaps | Both language sets classify real `mizar-driver`, `mizar-diagnostics`, producer projection payloads, artifact publication tokens, and system equivalence as deferred external gaps rather than placeholder APIs. | Synchronized. |
| Cache and snapshot freshness | Both language sets require fail-closed cache miss behavior before handle reconstruction and prohibit publishing obsolete/open outputs as current artifacts. | Synchronized. |
| Projection boundary | Both language sets prohibit publishing raw `SurfaceAst`, `TypedAst`, `CoreIr`, `ControlFlowIr`, `VcIr`, `AtpProblem`, kernel-internal state, storage handles, and inline proof-witness payloads. | Synchronized. |

## Classified Drift

No current bilingual-specific `spec_gap`, `source_drift`,
`test_expectation_drift`, `boundary_violation`, or `repo_metadata_conflict` was
found in the mizar-ir document set. Existing crate-plan gap rows retain their
protocol classes, including `design_drift`, and those classes are synchronized
between languages. Deferred integration remains recorded as
`external_dependency_gap` risk tags in the existing crate plan and module
specifications.

## Audit Result

Task 17 closes without synchronized content changes beyond this audit record,
the crate-plan inventory entry, and the TODO status update. Task 18 should
reuse this bilingual scope while focusing specifically on the architecture-22
publisher, cache-adapter, and snapshot-replacement contract.
