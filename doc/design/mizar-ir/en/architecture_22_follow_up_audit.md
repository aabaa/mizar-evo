# mizar-ir Architecture-22 Follow-Up Audit

> Canonical language: English. Japanese companion:
> [../ja/architecture_22_follow_up_audit.md](../ja/architecture_22_follow_up_audit.md).

## Scope

This task-18 audit re-runs the source/spec correspondence and bilingual
documentation sync scopes for the architecture-22 freshness contract owned by
`mizar-ir`: obsolete outputs must not publish as current, open-buffer outputs
must not become package artifacts, and old outputs may be reused only through
validated cache-input boundaries.

The audit found no source or documentation drift requiring code changes.
Downstream clean/incremental/parallel driver equivalence remains the existing
`external_dependency_gap` risk tag recorded by the crate plan.

## Source/Spec Trace

| Architecture-22 rule | mizar-ir specification | Source and tests | Result |
|---|---|---|---|
| Obsolete snapshot results cannot publish as current results. | `publisher.md` requires explicit current/obsolete state and forbids obsolete current publication; `identity.md` and `storage.md` state retained old handles cannot become current results; `projection.md` rejects obsolete drafts as current artifact candidates. | `publisher.rs::validate_snapshot_state` returns `ObsoleteSnapshot`; publisher tests `rejects_wrong_obsolete_open_and_stale_publication` and `snapshot_replacement_makes_old_outputs_stale_but_retained_until_release`; projection test `obsolete_snapshot_is_rejected_as_current_projection`. | Covered. |
| Open-buffer/editor-only outputs are not package artifacts. | `publisher.md` rejects `OpenBuffer`/editor-only origins for current/package publication; `cache_adapter.md` says open-buffer dry-run cache records are out of scope; `projection.md` returns no artifact draft for non-current package outputs. | `publisher.rs::validate_snapshot_state` returns `OpenBufferOutput`; projection tests `internal_only_open_output_is_rejected_before_draft_returns` and `internal_only_reseal_of_collected_current_output_is_rejected`. | Covered. |
| Old outputs may remain readable only as retained stale data or validated cache inputs. | `publisher.md`, `storage.md`, and `identity.md` allow retained old outputs for diagnostics/explanations/LSP or validated cache input, not current publication. | Storage retain/collect tests keep stale outputs readable only while retained; cache adapter test `superseded_snapshot_output_can_still_encode_as_cache_input` proves old output encoding remains a cache-input path while `validate_current_output` rejects current publication. | Covered. |
| Cache hit rehydration is optimization-only and fail-closed. | `cache_adapter.md` delegates lookup/key/dependency/proof validation to `mizar-cache`, treats all non-validated states as misses, and seals a current-snapshot handle only after payload, side-table, parent, schema, and storage checks. | `cache_adapter.rs::rehydrate` returns misses before allocation/sealing for cache misses, corrupt/incompatible records, hash mismatches, parent mismatch, stale/collected parents, storage failures, and decode errors; tests assert no target lineage or successful rehydration on these paths. | Covered. |
| Cache rehydration does not create proof or trust authority. | `cache_adapter.md`, `publisher.md`, `projection.md`, and `identity.md` state cache hits and rehydrated handles do not upgrade proof acceptance, trusted status, verifier policy, or kernel acceptance. | Tests `rehydrated_handles_do_not_carry_cache_or_proof_authority`, `publisher_handles_do_not_carry_proof_cache_or_trust_authority`, and `projection_does_not_expose_cache_or_proof_authority_markers` cover this boundary. | Covered. |
| Published artifacts expose stable projections, not raw IR or storage/kernel internals. | `projection.md` bans raw `SurfaceAst`, `TypedAst`, `CoreIr`, `ControlFlowIr`, `VcIr`, `AtpProblem`, kernel-internal state, storage handles, and inline proof-witness payloads. | Projection leakage tests cover raw marker and hash-ref rejection across exports, expressions, obligations, diagnostics, provenance, witnesses, and dependency artifact references. | Covered. |

## Bilingual Sync Trace

The scoped English and Japanese documents remain synchronized for architecture
22 terminology and ownership boundaries:

| Pair | Scoped result |
|---|---|
| `publisher.md` | Both languages cover obsolete/current validation, open-buffer rejection, retained stale output limits, and validated cache-input handoff. |
| `cache_adapter.md` | Both languages cover validated-hit-only rehydration, fail-closed miss states, stale-data freshness, no cache-key ownership, and no proof/trust elevation. |
| `identity.md` and `storage.md` | Both languages cover snapshot-scoped identity/storage, stale retained handles, and no current-result reuse after replacement. |
| `projection.md` | Both languages cover current draft validation, obsolete/open rejection, raw IR leakage guard, and external publication-token gaps. |
| `00.crate_plan.md` and `todo.md` | Both languages keep task 18 scope, completion conditions, and `external_dependency_gap` risk tags aligned. |

## Classified Gaps

No current `spec_gap`, `source_drift`, `test_expectation_drift`,
`boundary_violation`, or bilingual drift was found for the architecture-22
publisher/cache/snapshot-replacement scope.

`IR-G-007` remains the tracked system-level `test_gap` with
`external_dependency_gap` risk: full clean/incremental/parallel driver
equivalence requires downstream orchestration and real producer/cache/artifact
seams that are not wired to `mizar-ir` in this checkout. This task does not add
a `mizar-driver` dependency, placeholder diagnostics integration,
producer-token, cache-key, dependency-fingerprint, or proof-policy APIs.

## Audit Result

Task 18 closes with no source changes. The obsolete/open/incomplete/cache-miss
rules are traced to module specs, source, tests, and bilingual companion docs.
Task 19 should use this result as input when auditing module boundaries and
private helper placement.
