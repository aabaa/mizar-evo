# Bilingual Documentation Sync Audit: mizar-core

> Canonical language: English. Japanese companion:
> [../ja/bilingual_sync_audit.md](../ja/bilingual_sync_audit.md).

Task 23 audits the English canonical `mizar-core` design documents and their
Japanese companions after the Task 22 source/spec audit. This task is
documentation-only: it does not change Rust source, public APIs, `.miz`
fixtures, expectations, traceability metadata, or behavior.

## Scope And Method

The audit compares every current file directly under
`doc/design/mizar-core/en/` with the same filename under
`doc/design/mizar-core/ja/`.

Current paired file set:

- `00.crate_plan.md`
- `binder_normalization.md`
- `bilingual_sync_audit.md`
- `control_flow.md`
- `core_ir.md`
- `elaborator.md`
- `module_boundary_audit.md`
- `source_spec_audit.md`
- `task_ledger.md`
- `todo.md`

Comparison is structural and semantic rather than word-for-word. The Japanese
companions may keep technical English terms and localize prose. Expected
language-specific differences are allowed:

- English documents link to `../../architecture/en/`, `../../../spec/en/`,
  and other English canonical files.
- Japanese documents link to `../../architecture/ja/`, `../../../spec/ja/`,
  and Japanese companion files when they exist.
- Task ledger prose is localized but must preserve the same status, review,
  verification, and deferred/external meaning.
- Markdown headings may be localized, but the section intent and task/gap
  coverage must remain aligned.

Result: no blocking bilingual documentation drift is observed for the current
paired file set. Remaining unavailable documents are future task outputs and
are recorded below instead of being treated as missing companions.

## Pair Inventory

| File | English canonical status | Japanese companion status | Sync result |
|---|---|---|---|
| `00.crate_plan.md` | Defines responsibility, authority order, references, tests, gaps, task decomposition, and exit criteria. | Mirrors the plan with localized prose and Japanese reference links. | No drift. Task 23 updates both inventories to include this audit. |
| `binder_normalization.md` | Specifies representation, normalization, alpha-equivalence, substitution, closures, diagnostics, tests, enum policy, and forbidden behavior. | Mirrors the same module spec and gap classifications. | No drift. Technical terms intentionally remain English-heavy. |
| `bilingual_sync_audit.md` | Records this paired-document inventory, allowed language-specific differences, deferred future docs, and docs-only verification. | Mirrors the same audit structure and classifications. | No drift. This row is self-referential by design for restart/closeout inventory. |
| `control_flow.md` | Specifies `ControlFlowIr`, blocks, locals, contexts, contracts, ghost effects, termination, diagnostics, handoff sites, determinism, enum policy, and tests. | Mirrors the same phase-10 design with localized prose. | No drift. Architecture-07 ownership drift is classified in both files. |
| `core_ir.md` | Specifies `CoreIr` data shapes, generated origins, obligation seeds, source maps, diagnostics, validation, enum policy, gaps, and forbidden behavior. | Mirrors the same data-shape and boundary policy. | No drift. |
| `elaborator.md` | Specifies phase-9 input/output contracts, six lowering steps, diagnostics, determinism, enum policy, and forbidden behavior. | Mirrors the same six-step elaboration design and external/deferred classifications. | No drift. |
| `module_boundary_audit.md` | Records the Task 24 source-layout gate, large review-risk files, no required split before closeout, and deferred move-only follow-ups. | Mirrors the same audit-only decision and classifications. | No drift. Added by Task 24. |
| `source_spec_audit.md` | Records public module/API inventory, source/spec/test/deferred correspondence, `source_undocumented_behavior` pass, and CORE-AUDIT follow-up register. | Mirrors the same audit structure and CORE-AUDIT gap IDs/classes. | No drift. Task 22 lint guard also checks the source/spec audit pair. |
| `task_ledger.md` | Records task restart status, review results, verification, and deferred reasons through the current task. | Mirrors the same ledger rows with localized prose. | No drift. Task 24 row is updated in this commit before staging. |
| `todo.md` | Defines the ordered task list, status legend, verification, and notes. | Mirrors the ordered task list and verification policy. | No drift. Task 24 status is updated in this commit before staging. |

## Resolved Pair Updates

| ID | Prior class | Resolution |
|---|---|---|
| CORE-BILINGUAL-G001 | `deferred` | Resolved by Task 24: `module_boundary_audit.md` now exists in both languages and is listed in the paired-file inventory. Future edits must keep the pair synchronized. |

## Remaining Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| CORE-BILINGUAL-G002 | `deferred` | `crate_exit_report.md` does not exist yet in either language because it is closeout output. | Closeout must add English and Japanese companions together. |

No `spec_gap`, `source_drift`, `source_undocumented_behavior`,
`test_expectation_drift`, `repo_metadata_conflict`, or `boundary_violation` is
observed by this bilingual audit. No existing `.miz` tests, expectations, Rust
source, or traceability rows are changed.

## Guard Decision

No new Rust lint guard is added in Task 23. The file-pair set is small and all
current pairs are enumerated in this audit. Task 22 already added a focused
guard for the highest-risk source/spec audit pair: public module coverage,
public-item mentions, and CORE-AUDIT gap synchronization. A broader bilingual
guard can be added later if Task 24 splits documents or closeout introduces a
larger documentation matrix, but adding one here would turn this planned
docs-only synchronization task into a Rust-test change without a concrete
coverage gap.

## Verification

Docs-only verification for this task:

- `git diff --check` before staging.
- `git diff --cached --check` after explicit path staging.
