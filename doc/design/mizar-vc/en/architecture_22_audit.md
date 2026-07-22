# Architecture-22 Follow-Up Audit: mizar-vc

> Canonical language: English. Japanese companion:
> [../ja/architecture_22_audit.md](../ja/architecture_22_audit.md).

Task 21 audits the Task 20 implementation against
[architecture 22](../../architecture/en/22.incremental_verification_contract.md)
after the cross-edit reuse identity wiring landed in commit
`2f3eb323be8080bf231e1b69dfc9e9e729bb45f9`. This is a docs-only audit. It
does not change Rust source, `.miz` fixtures, expectations, `doc/spec`,
traceability metadata, runner support, or downstream ATP/kernel/proof/cache
integration.

## Scope And Method

The audit reviewed the architecture-22 requirements for:

- clean-build equivalence and cache-miss fallback;
- `VcId` versus `ObligationAnchor` identity boundaries;
- canonical VC and local-context fingerprints;
- dependency-slice completeness and reusable fingerprint payloads;
- verifier-policy and deterministic-discharge evidence gates;
- bilingual documentation synchronization after Task 20.

Reviewed mizar-vc documents:

- [vc_ir.md](./vc_ir.md)
- [generator.md](./generator.md)
- [discharge.md](./discharge.md)
- [dependency_slice.md](./dependency_slice.md)
- [source_spec_audit.md](./source_spec_audit.md)
- [bilingual_sync_audit.md](./bilingual_sync_audit.md)
- [todo.md](./todo.md)
- [task_ledger.md](./task_ledger.md)

## Architecture-22 Correspondence

| Architecture-22 requirement | mizar-vc status after Task 20 | Classification |
|---|---|---|
| `VcId`, source ranges, parser/node/arena ids, and task-local row ids are never sufficient for cross-edit proof reuse. | `VcId` remains snapshot-local. `ObligationAnchor` source-shape payloads exclude `VcId`, source range, `SourceId`, handoff id, candidate sort key, and dense owner row id. Dependency-slice reusable fingerprints hash stable payloads, not diagnostic local keys. | Implemented for current deterministic-discharge reuse candidates. |
| Matching `ObligationAnchor` alone is not sufficient. | `ProofReuseCandidateKey` additionally requires a complete anchor, current matching dependency slice, canonical VC fingerprint, local-context fingerprint, compatible policy fingerprint, and matching newly produced deterministic discharge evidence. | Implemented for deterministic discharge branch. |
| Canonical VC fingerprint covers goal, premises, proof hints, and generated formula payloads, or fails closed. | `CanonicalVcFingerprint` resolves generated formulas through the owning `VcSet`; raw core formula ids, definition ids through hints/premises, diagnostics, cycles, and quantified formulas without stable binder payloads fail closed. | Implemented for stable generated payloads; incomplete upstream payloads remain fail-closed. |
| Canonical local-context fingerprint covers stable context payloads and policy inputs, or fails closed. | `LocalContextFingerprint` covers sort keys, non-binder context kinds, resolved formula payloads, provenance, and explicit verifier-policy inputs. Binder declarations and unresolved core/generated formula payloads fail closed. | Implemented for stable local contexts; binder/core payloads remain fail-closed. |
| Dependency slices used for reuse must be complete; missing dependency data must never mean no dependency. | `DependencySlice` marks unknown coverage as `IncompleteUncacheable`. Raw `CoreFormulaId`, `CoreDefinitionId`, unresolved generated formulas, quantified formulas, binder declarations, opaque trace/import/computation markers, missing replay data, and incomplete anchors produce conservative unknowns. `ProofReuseCandidateKey` rejects incomplete slices. | Implemented for current slice families; missing upstream payloads stay external gaps. |
| Verifier policy and proof witness or deterministic discharge hash must match. | Task 20 implements the deterministic-discharge branch. The key includes policy inputs/status policy and requires a newly produced replayable deterministic evidence record whose status evidence matches the current VC. Proof-witness hashes and consumer validation are not implemented. | Deterministic-discharge branch implemented; proof-witness branch deferred/external. |
| Cache lookup, kernel acceptance, proof policy, ATP certificates, and artifact consumers must validate reuse before accepting a hit. | `mizar-vc` produces untrusted reusable inputs only. No downstream cache, ATP, kernel, proof, or artifact consumer accepts these keys in this crate. | `external_dependency_gap` / `deferred`. |

## Regression Evidence

Task 20 adds or updates Rust coverage for:

- cross-edit `VcId` shift with equal proof-reuse key;
- generated-formula id shift with equal deterministic-discharge reuse
  fingerprint before kernel evidence handoff identity is supplied; task 28
  intentionally makes current kernel-handoff proof-reuse keys conservative when
  the canonical handoff or context-identity hash shifts;
- policy and local-context changes changing reuse identity;
- stale slice sets, pre-existing evidence, incomplete anchors, generated-goal
  changes, missing stable evidence, and unresolved payloads failing closed;
- generated seed families and algorithm candidates keeping source-shape hashes
  available while raw core goals remain canonical-goal incomplete;
- unresolved core formula, definition, generated diagnostic, quantified, and
  binder payloads producing independent unknown coverage.

Task 21 adds no new Rust tests because it is an audit-only task. The Task 20
verification recorded in the ledger is the relevant source behavior evidence.

## Remaining Classified Gaps

- `external_dependency_gap`: active source-derived `proof_verification` support
  is absent from `mizar-test`; VC Task 31 / `MT10-VC-T180` owns the first exact
  route, and VC 32-55 own later `MT10-VC-PV/VC<n>` slices.
- `external_dependency_gap` / `deferred`: `mizar-kernel` owns corrected
  formula/substitution evidence checking, and completed VC Tasks 25-29 own the
  producer handoff and identity payloads. `mizar-atp`, `mizar-proof`, and
  `mizar-cache` exist, but they are not wired as active
  consumers of this VC milestone, so ATP translation, proof policy, cache
  lookup/reuse, artifact persistence, and proof-witness validation remain
  outside this crate.
- `external_dependency_gap`: upstream explicit/stable payloads remain
  incomplete for registration, redefinition, reduction, call-precondition,
  branch, match, range-loop, collection-loop, term-derived/recursive
  termination, Pick
  non-emptiness, ghost-isolation zero-VC integration, authenticated trace
  contexts, source-derived core formula payloads, definition payloads,
  quantified binder payloads, and source-derived obligation payload families.
- `spec_gap`: VC 53 has canonical partial-call admission semantics, but no
  canonical termination-evidence producer, reference identity/schema,
  authentication contract/rules, or owning tests. It remains blocked, and no
  transport or authentication mechanism is invented.
- `deferred`: proof-witness hashes, ATP/kernel/proof/cache validation, artifact
  consumers, and source-derived runner integration must be implemented by
  downstream or later tasks before architecture-22 reuse can be accepted outside
  the deterministic-discharge candidate key.

No `repo_metadata_conflict`, unclassified `source_drift`, `design_drift`,
`source_undocumented_behavior`, `test_expectation_drift`, or
`boundary_violation` was observed in the Task 20 identity contract after this
audit.

## VC Task 30 Source-Derived Identity Addendum

Task 30 changes follow-up ownership, not architecture-22 reuse eligibility.
The exact Task-31 anchor keeps source-shape and empty-context identity available
but remains incomplete for `CanonicalGoalHash`; it is therefore ineligible for
reuse. Tasks 32-55 own stable formula, context, substitution, trace, and
dependency ingredients only when their real source-derived payloads land.
Missing payloads remain conservative unknowns rather than ids, labels, ranges,
or markers disguised as canonical hashes. This update grants no cache lookup,
proof reuse, discharge, verification, or acceptance credit.
