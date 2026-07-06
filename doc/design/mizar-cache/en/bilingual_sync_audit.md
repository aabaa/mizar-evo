# Bilingual Documentation Sync Audit: mizar-cache

> Canonical language: English. Japanese companion:
> [../ja/bilingual_sync_audit.md](../ja/bilingual_sync_audit.md).

Task 19 introduced this audit for the `mizar-cache` design documentation pairs
after the source/spec correspondence audit. Task 24 updates it as a living
sync record after the proof-reuse accepted goal-polarity change; the audit
still changes no cluster-db behavior, artifact publication policy, scheduler
integration, `mizar-ir` integration, or proof-authority boundary.

## Scope And Method

The audit covers every current Markdown document under
`doc/design/mizar-cache/en/` and its companion under
`doc/design/mizar-cache/ja/`. For each pair, the review checked:

- same filename in both language directories;
- canonical/companion link at the top of each document;
- synchronized meaning for module responsibility, public API shape,
  fail-closed behavior, cache/proof trust boundaries, public enum policy,
  planned tests, source/spec audit inventories, task ledger rows, TODO task
  status, and deferred/external dependency classifications;
- preservation of owner-gated downstream integration records rather than
  silently resolving them or adding placeholders.

The Japanese companion may use idiomatic translation and may retain Rust
identifiers, phase names, gap IDs, and task names in English. Synchronization
is semantic: the Japanese companion must not omit, weaken, or add normative
meaning relative to the English canonical document.

Result after task 24: all current document pairs exist and are semantically
synchronized. No meaning-changing bilingual drift, missing companion, stale
sync placeholder, or `repo_metadata_conflict` was observed. Task 19 added this
paired audit document, task 23 backfilled the closeout hash, and task 24
records the proof-reuse accepted goal-polarity update in paired documents.

## Pair Inventory

| Document | Synchronized content checked | Result |
|---|---|---|
| `00.crate_plan.md` | Crate responsibility, authority boundaries, specification references, known gaps/drift, relevant tests/source inventory through task 24, task decomposition through task 24, task 20-24 results, and task 24 proof-reuse identity plan. | Synchronized by task 24 updates. |
| `architecture_22_audit.md` | Task 21 architecture-22 follow-up audit, crate-owned contract checks, bilingual/source-spec result, residual gap classification, and no-blocking-finding conclusion. | Synchronized by task 21 updates. |
| `cache_key.md` | Cache-key purpose, public API, validation inputs, canonical ordering/hashing, fail-closed rules, proof-reuse boundary, tests, public enum policy, deferred gaps, and non-goals. | Synchronized. |
| `dependency_fingerprint.md` | Fingerprint responsibility, public conceptual API, target taxonomy, stable inputs/exclusions, completeness states, rebuild triggers, API compatibility diff, trust boundary, planned tests, public enum policy, deferred gaps, and non-goals. | Synchronized. |
| `cache_store.md` | Record/blob store API, on-disk layout, record identity, header and encoding rules, lookup/insert fail-closed behavior, miss reasons, deletability, trust boundary, tests, public enum policy, deferred gaps, and non-goals. | Synchronized. |
| `proof_reuse.md` | Proof-reuse metadata inputs including accepted goal-polarity key, reusable classes, validation predicate, determinism, failure semantics, output contract, public enum policy, tests, deferred/external gaps, and non-goals. | Synchronized by task 24 updates. |
| `cluster_db.md` | Accepted-only cluster-db purpose, authority inputs, conceptual surface, store layout, origin metadata, importer-visible filtering, aggregate indexes, import-scoped views, invalidation, failure semantics, public enum policy, deferred gaps, tests, and non-goals. | Synchronized. |
| `integration_readiness.md` | Task 15 readiness scope, current cache surface, `mizar-build`/`mizar-ir`/publication-token external dependency gaps, deferred work, and docs-only verification. | Synchronized. |
| `source_spec_audit.md` | Task 18 public module exports, public API and method inventory, trust boundary, task-20 cross-module evidence, task-22 private test module paths, task-24 accepted-polarity proof-reuse evidence, guarded test references, full gap table parity with ledgers, and no-drift conclusion. | Synchronized by task 24 updates. |
| `module_boundary_audit.md` | Task 22 source layout audit, private test-module split decision, public API invariants, source/spec and bilingual result, and verification scope. | Synchronized by task 22 updates. |
| `crate_exit_report.md` | Task 23 closeout scope plus task 24 post-closeout update, task commits, final owned surfaces, hard gates, quality score, review results, deferred/external dependency items, verification, and handoff. | Synchronized by task 24 updates. |
| `bilingual_sync_audit.md` | Task 19 scope, method, pair inventory, classification, and sync edits. | Synchronized by this paired audit document. |
| `task_ledger.md` | Task status and commit hashes through task 23, pending task 24 self-hash, review/verification rows, quality score, and complete deferred/external dependency gap register. | Synchronized by task 24 updates; task 24 self-hash is post-closeout self-reference metadata. |
| `todo.md` | Ordered tasks, completed tasks through task 24, recommended verification, and notes. | Synchronized by task 24 updates. |

## Classification

Task 24 records no new `spec_gap`, `test_gap`, `design_drift`,
`source_drift`, `source_undocumented_behavior`, `test_expectation_drift`,
`boundary_violation`, or `repo_metadata_conflict`.

Existing classified records remain the deferred and external dependency gaps
listed in [task_ledger.md](./task_ledger.md) and repeated by
[source_spec_audit.md](./source_spec_audit.md). In particular:

- `mizar-build` scheduler seams now exist, while end-to-end scheduler/cache
  lookup integration remains an `external_dependency_gap`;
- `mizar-ir` cache-adapter validation boundaries now exist, while end-to-end
  rehydration integration remains an `external_dependency_gap`;
- artifact/proof committed publication-token linkage remains an
  `external_dependency_gap`;
- fine-grained producer slices, persistent cluster-db storage, and persistent
  import-scoped view files remain `deferred` until their owning tasks land;
  cross-crate clean/incremental equivalence beyond the crate-owned task-20
  contract remains an `external_dependency_gap` on scheduler and artifact
  publication owners.

No task-24 edit weakens the rule that `mizar-cache` is an optimization owner,
not proof authority. Cache records, externally attested evidence, backend
diagnostics/logs, timing metadata, accepted goal-polarity keys, and cluster-db
data still do not become kernel-verified status or trusted `used_axioms`.

## Task 24 Sync Edits

Task 24 updates the paired crate plan, proof-reuse design, source/spec audit,
task ledger, todo, crate exit report, and this audit for the accepted
goal-polarity proof-reuse identity update. It backfills Task 23's commit hash
and leaves Task 24's pending self-hash for the completing commit.

No unpaired or meaning-changing content remains.
