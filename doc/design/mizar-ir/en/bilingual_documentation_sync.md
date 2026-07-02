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
| File coverage | Both language directories contain the paired crate plan, TODO, module specs, dispatch input spec, source/spec audit, bilingual audit, architecture-22 audit, module-boundary gate, and crate exit report. | Synchronized for task 20 scope. |
| Heading parity | Paired task-20 documents keep matching sections; pre-existing bilingual pairs remain aligned after the dispatch input and closeout updates. | Synchronized. |
| Task status parity | TODO task states are aligned: tasks 1-20 complete. | Synchronized for task 20 scope. |
| Gap classification parity | Crate-plan `IR-G-001` through `IR-G-012` and source/spec `IR-G-004` through `IR-G-012` appear in both languages with matching classes and dispositions. | Synchronized for task 20 scope. |
| Ownership boundaries | Both language sets state that `mizar-ir` owns internal IR storage, snapshot handles, phase output references, publication, cache adaptation, projection, and snapshot replacement, but not proof acceptance, trusted status, kernel acceptance, `CacheKey`, dependency fingerprints, or proof-reuse validation. | Synchronized. |
| External dependency gaps | Both language sets classify remaining producer dispatch, diagnostics rendering, producer projection payloads, artifact publication tokens, cache-compatibility wiring, LSP conversion, and system equivalence as deferred external gaps rather than placeholder APIs. | Synchronized. |
| Cache and snapshot freshness | Both language sets require fail-closed cache miss behavior before handle reconstruction and prohibit publishing obsolete/open outputs as current artifacts. | Synchronized. |
| Projection boundary | Both language sets prohibit publishing raw `SurfaceAst`, `TypedAst`, `CoreIr`, `ControlFlowIr`, `VcIr`, `AtpProblem`, kernel-internal state, storage handles, and inline proof-witness payloads. | Synchronized. |

## Classified Drift

No current bilingual-specific `spec_gap`, `source_drift`,
`test_expectation_drift`, `boundary_violation`, or `repo_metadata_conflict` was
found in the mizar-ir document set. Existing crate-plan gap rows retain their
protocol classes, including `design_drift`, and those classes are synchronized
between languages. Deferred integration remains recorded as
`external_dependency_gap` risk tags in the existing crate plan and module
specifications. Task 20 adds `dispatch_input.md` and reclassifies the old
driver-absence wording as current downstream integration gaps rather than a
missing driver crate.

## Audit Result

Task 20 preserves the bilingual sync invariant: the new dispatch input
boundary, closeout status, verification records, and gap classifications are
represented in both language directories.
