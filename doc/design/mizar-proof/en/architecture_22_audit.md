# mizar-proof Architecture-22 Follow-Up Audit

> Canonical language: English. Japanese companion:
> [../ja/architecture_22_audit.md](../ja/architecture_22_audit.md).

## Scope

Task 18 re-audits the task-17 proof-reuse metadata export contract against:

- [`selection.md`](./selection.md) and `crates/mizar-proof/src/selection.rs`;
- [`status.md`](./status.md) and `crates/mizar-proof/src/status.rs`;
- [architecture 22](../../architecture/en/22.incremental_verification_contract.md);
- [architecture 11](../../architecture/en/11.artifact_and_incremental_build.md);
- [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md);
- paired English/Japanese documentation for `mizar-proof`.

The audit is documentation-only. It does not add cache lookup, artifact
publication stubs, ATP backend wiring, proof acceptance, or trusted-status
promotion.

## Result

No blocking `spec_gap`, `test_gap`, `design_drift`, `source_drift`,
`source_undocumented_behavior`, `test_expectation_drift`, or
`boundary_violation` remains in `mizar-proof` for the task-17 reuse metadata
contract.

One out-of-scope `repo_metadata_conflict` was observed in `mizar-atp` during
task 18: its task-28 closeout guard still treated `crates/mizar-proof` as a
forbidden placeholder crate even though the current `mizar-proof` workflow
explicitly authorized and completed the formal scaffold. This audit recorded
that conflict only. The conflict was later resolved by focused metadata
correction commit `36d1a9c` before task-20 closeout.

## Architecture-22 Trace

| Requirement | Source/doc evidence | Result |
|---|---|---|
| `ObligationAnchor` is a reuse-candidate key, not proof authority. | `ProofObligationIdentity` carries `ObligationAnchor`; `StatusReuseMetadata` exports it; status docs say anchor alone is insufficient. | consistent |
| Reuse requires canonical VC, local-context, dependency-slice, policy, and witness/discharge identity. | `StatusReuseMetadata` exports obligation, VC, context, dependency-slice, policy, selected witness, deterministic discharge, and validation hash fields. | consistent |
| Dependency artifact/schema compatibility participates in reuse validation. | `ProofReuseDependencyCompatibility` records dependency artifact fingerprint, dependency schema version, and proof-reuse schema version; tests mutate each independently. | consistent |
| Proof evidence identity is stable and deterministic. | `ProofEvidenceReuseIdentity` exposes selected candidate id, selected provenance hash, selected evidence/witness/discharge hashes, tie-break hash, and selection reason. | consistent |
| Arrival order, completion time, runtime duration, and cache timing are not identity. | Selection tie-breaks use stable priority/hash/provenance/source-id fields; determinism tests shuffle candidate order; no timing/cache-hit fields are present. | consistent |
| Externally attested evidence is never upgraded by cache reuse. | `PolicyPermittedExternal` projects to `ExternallyAttested`; trusted `used_axioms` stay absent; `cache_reuse_predicate_complete` returns false for all non-trusted classes. | consistent |
| Trusted reuse completeness requires accepted witness/discharge identity. | `KernelVerified` completeness requires `selected_proof_witness_hash`; `DischargedBuiltin` completeness requires `deterministic_discharge_hash`; missing dependency compatibility is incomplete. | consistent |
| Cache records are not proof authority. | `mizar-proof` exposes metadata and validation hashes only; no cache lookup, cache hit, cache record reader, or cache promotion API exists. | consistent |

## Bilingual Sync

The English canonical files changed by tasks 17 and 18 have synchronized
Japanese companions:

| English canonical | Japanese companion | Task-18 result |
|---|---|---|
| `00.crate_plan.md` | `00.crate_plan.md` | completed-audit inventory synchronized |
| `selection.md` | `selection.md` | task-17 reuse fields synchronized |
| `status.md` | `status.md` | validation hash and class-aware completeness synchronized |
| `todo.md` | `todo.md` | task-17 status synchronized; task-18 updated in this task |
| `task_ledger.md` | `task_ledger.md` | task-17 hash backfill and task-18 entry synchronized |
| `architecture_22_audit.md` | `architecture_22_audit.md` | created by this task |

Japanese companions intentionally keep Rust type names, enum variants, status
strings, command names, and gap identifiers in English. That is not drift.

## Remaining Gaps

| ID | Class | Evidence | Handling |
|---|---|---|---|
| `PROOF18-G001` | `external_dependency_gap` | `mizar-cache` now exists and owns proof-reuse validation, lookup, cache-hit/miss decisions, and policy compatibility checks. This proof milestone exports validation metadata but does not call those APIs. | Keep `mizar-proof` metadata as validation input only. Add proof/cache wiring in an owner-scoped integration task. |
| `PROOF18-G002` | resolved roadmap drift with remaining `external_dependency_gap` | The detailed cache proof-reuse document now exists under `doc/design/mizar-cache/`; no placeholder cache docs or APIs are needed in `mizar-proof`. | Keep the metadata API stable and let `mizar-cache` own validation semantics. |
| `PROOF18-G003` | `external_dependency_gap` | Current artifact witness schema still cannot publish `DischargedBuiltin` witness refs. | Keep `DischargedBuiltin` trusted internally only after accepted kernel evidence, export deterministic discharge hash, and leave artifact witness publication deferred. |
| `PROOF18-G004` | `external_dependency_gap` | `CommittedWitnessPublicationProof` remains opaque without an artifact-owned production token. | Witness publication stays blocked on artifact manifest reachability integration. |
| `PROOF18-G005` | `external_dependency_gap` | `TrustedKernelWitnessMetadata` remains opaque without copied kernel/artifact acceptance metadata. | `mizar-proof` must not trust caller-synthesized kernel acceptance metadata. |
| `PROOF18-G006` | `external_dependency_gap` | Live ATP early-stop adoption and backend cancellation remain outside `mizar-proof`. | Policy hooks remain stable metadata/API; no backend execution or cancellation stub is added. |
| `PROOF18-G007` | `deferred` | The task-17 completeness test covers kernel-with-witness, kernel-without-witness, built-in discharge, and externally attested evidence; other non-trusted classes share the same explicit match arm but are not separately enumerated in that test. | Treat as non-blocking branch-coverage follow-up only if task 19 or closeout wants exhaustive non-trusted class fixtures. |

## Repo Metadata Conflict

| ID | Classification | Evidence | Handling |
|---|---|---|---|
| `PROOF18-RM001` | resolved `repo_metadata_conflict` | During task 18, `cargo test -p mizar-atp` failed in `atp_task_twenty_eight_crate_exit_report_is_documented` because the `mizar-atp` closeout guard rejected workspace member `crates/mizar-proof` and the `crates/mizar-proof` directory as task-28 placeholders. | Report-only in task 18. Resolved later by focused metadata correction commit `36d1a9c`; proof policy ownership was not moved to `mizar-atp`. |

## Conclusion

The task-17 proof-reuse metadata export contract satisfies architecture 22 for
the `mizar-proof` ownership boundary. The crate now exports stable,
deterministic validation metadata for future cache consumers while keeping
trusted proof acceptance tied only to accepted kernel evidence. Remaining work
is downstream integration or documented branch-coverage follow-up, not a
permission to promote cache, external, diagnostic, or witness metadata into
trusted status.
