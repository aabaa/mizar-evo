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
- `crate_exit_report.md`
- `elaborator.md`
- `module_boundary_audit.md`
- `source_family_decomposition.md`
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
paired file set. All current closeout documents have English/Japanese
companions; resolved pair updates are recorded below.

## Pair Inventory

| File | English canonical status | Japanese companion status | Sync result |
|---|---|---|---|
| `00.crate_plan.md` | Defines responsibility, authority order, references, tests, gaps, task decomposition, and exit criteria. | Mirrors the plan with localized prose and Japanese reference links. | No drift. Task 23 updates both inventories to include this audit. |
| `binder_normalization.md` | Specifies representation, normalization, alpha-equivalence, substitution, closures, diagnostics, tests, enum policy, and forbidden behavior. | Mirrors the same module spec and gap classifications. | No drift. Technical terms intentionally remain English-heavy. |
| `bilingual_sync_audit.md` | Records this paired-document inventory, allowed language-specific differences, resolved pair updates, remaining classifications, and docs-only verification. | Mirrors the same audit structure and classifications. | No drift. This row is self-referential by design for restart/closeout inventory. |
| `control_flow.md` | Specifies `ControlFlowIr`, blocks, locals, contexts, contracts, ghost effects, termination, diagnostics, handoff sites, determinism, enum policy, and tests. | Mirrors the same phase-10 design with localized prose. | No drift. Architecture-07 ownership drift is classified in both files. |
| `core_ir.md` | Specifies `CoreIr` data shapes, generated origins, obligation seeds, source maps, diagnostics, validation, enum policy, the Task-31 pending-proof projection, gaps, and forbidden behavior. | Mirrors the same data-shape, exact projection, and boundary policy. | No drift. Task-31 additions are synchronized. |
| `crate_exit_report.md` | Records closeout status, task commits, hard gates, score, deferred items, verification, and handoff. | Mirrors the same closeout evidence and classifications. | No drift. Added by closeout. |
| `elaborator.md` | Specifies phase-9 input/output contracts, six lowering steps, the exact Task-180 adapter, diagnostics, determinism, enum policy, and forbidden behavior. | Mirrors the same six-step design, exact adapter, and external/deferred classifications. | No drift. Task-31 additions are synchronized. |
| `module_boundary_audit.md` | Records the Task 24 source-layout gate, large review-risk files, no required split before closeout, and deferred move-only follow-ups. | Mirrors the same audit-only decision and classifications. | No drift. Added by Task 24. |
| `source_family_decomposition.md` | Records Task 32's Core 33-53 graph, joint algorithm producer/lowerer contract, five prepared consumers, gates, corruption boundaries, and no-credit exit. | Mirrors the same task/dependency authority and forbidden boundaries. | No drift. Added by Task 32. |
| `source_spec_audit.md` | Records public module/API inventory including Task 31, source/spec/test/deferred correspondence, `source_undocumented_behavior` pass, and CORE-AUDIT follow-up register. | Mirrors the same audit structure, exact Task-180 coverage, and CORE-AUDIT gap IDs/classes. | No drift. Task 22 lint guard also checks the source/spec audit pair. |
| `task_ledger.md` | Records task restart status, review results, verification, and deferred reasons through the current task. | Mirrors the same ledger rows with localized prose. | No drift. Closeout row and task hash backfill are updated in this commit before staging. |
| `todo.md` | Defines the ordered task list, status legend, verification, and notes. | Mirrors the ordered task list and verification policy. | No drift. Closeout status is updated in this commit before staging. |

## Resolved Pair Updates

| ID | Prior class | Resolution |
|---|---|---|
| CORE-BILINGUAL-G001 | `deferred` | Resolved by Task 24: `module_boundary_audit.md` now exists in both languages and is listed in the paired-file inventory. Future edits must keep the pair synchronized. |
| CORE-BILINGUAL-G002 | `deferred` | Resolved by closeout: `crate_exit_report.md` now exists in both languages and is listed in the paired-file inventory. Future edits must keep the pair synchronized. |

## Remaining Classification

No active bilingual-documentation gaps remain for the current paired file set.

Core Task 31 rechecked every changed pair: `00.crate_plan.md`, `core_ir.md`,
`elaborator.md`, `source_spec_audit.md`, `module_boundary_audit.md`,
`crate_exit_report.md`, `task_ledger.md`, and `todo.md`. The exact adapter,
snapshot exception, remaining broad deferred ownership, and forbidden scope
agree in English and Japanese.

No `spec_gap`, `source_drift`, `source_undocumented_behavior`,
`test_expectation_drift`, `repo_metadata_conflict`, or `boundary_violation` is
observed by this bilingual audit. Task 31 changes Rust source, the exact
Task-180 expectation sidecar, and one new exact traceability row in both the
implementation and paired owning documentation; those changes agree. The
existing `.miz` source and its semantic pass intent remain unchanged, and the
older broad CoreIr trace row remains deferred.

Core Task 32 adds and rechecks the `source_family_decomposition.md` pair and
synchronizes the plan, TODO, source audit, module specifications, ledger, and
cross-crate ownership notes. Both languages preserve the 33-53 task split,
prepared consumer stages, Gates A1/S1, VC-owned substitution boundary, and
zero coverage promotion.

Core Task 33C4C8 rechecks the changed plan, TODO, source-family, elaborator,
source/spec audit, module-boundary audit, and contract pairs. Both languages
preserve the sole Task33C capability, exact owner join, max-plus-one `x,y`
allocation, private canonical order, zero-semantic boundary, Core35 deferral,
and unchanged Task277B/protected artifacts.

Core Task 33LB rechecks the changed plan, TODO, source-family, elaborator,
source/spec audit, and the canonical
[`CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB`](../../task_contracts/en/CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB.md)
contract with its [Japanese companion](../../task_contracts/ja/CORE-SOURCE-LOCAL-BINDER-CONTEXT-33LB.md).
Both languages preserve checker binding identity/order, checked fresh Core
allocation, the standalone immutable handoff, default-deny validation, the
separate C4C8 boundary, zero semantic/coverage credit, deferred Core 33--35
and `MT10-CIR-TE`, and unchanged Task277B/protected artifacts.

Core Task 33I259 rechecks the plan, TODO, source-family, elaborator,
source/spec audit, ledger, and canonical
[`CORE-SOURCE-PREDICATE-ITEM-CONTEXT-33I259`](../../task_contracts/en/CORE-SOURCE-PREDICATE-ITEM-CONTEXT-33I259.md)
contract with its [Japanese companion](../../task_contracts/ja/CORE-SOURCE-PREDICATE-ITEM-CONTEXT-33I259.md).
Both languages preserve the exact Task-259 one-predicate cardinality, context-
link identity join, whole-symbol Core lookup, standalone 33LB composition,
default-deny boundary, multi-definition deferral, zero credit, and unchanged
Task277B/protected artifacts.

Core Task 33I260 implementation, verification, and exact task-only commit
`f8e9fc21` are complete. The canonical
[`CORE-SOURCE-FUNCTOR-ITEM-CONTEXT-33I260`](../../task_contracts/en/CORE-SOURCE-FUNCTOR-ITEM-CONTEXT-33I260.md)
contract and [Japanese companion](../../task_contracts/ja/CORE-SOURCE-FUNCTOR-ITEM-CONTEXT-33I260.md)
own the detailed profile, boundary, review, verification, protected-artifact,
and deferral evidence. Final bilingual review has no findings.

## Core Task 33I261 Bilingual Contract Parity

The frozen [EN contract](../../task_contracts/en/CORE-SOURCE-ATTRIBUTE-ITEM-CONTEXT-33I261.md)
and [JA companion](../../task_contracts/ja/CORE-SOURCE-ATTRIBUTE-ITEM-CONTEXT-33I261.md)
define the same one-row ordinary-attribute Core association, immutable
handoff/error/producer surface, exact identity and source-order boundary, and
default-deny validation. Implementation, verification, and the exact task-only
commit `4c6ecafc2a9bee7a4eb6e3f27336733fc672bd57` are complete. Both languages preserve
zero semantic/execution/coverage credit, no production install or generic API/
slot/installer, deferred Core 34--36/`GeneratedOrigin`/`MT10-CIR-TE` work, and
Task277B not-ready/zero-credit status. English is canonical; no exception is
recorded.

## Core Task 33I262 Bilingual Contract Parity

The frozen [EN contract](../../task_contracts/en/CORE-SOURCE-MODE-ITEM-CONTEXT-33I262.md)
and [JA companion](../../task_contracts/ja/CORE-SOURCE-MODE-ITEM-CONTEXT-33I262.md)
define the same one-row ordinary-mode Core association, immutable
handoff/error/producer surface, exact identity/source-order boundary, and
default-deny validation. Implementation and verification are complete; all
independent reviews ended with no findings after documentation repairs. Both
languages preserve zero semantic/execution/coverage credit, no checker-doc or
production install/generic API/slot/installer change, deferred Core 34--36/
`GeneratedOrigin`/`MT10-CIR-TE` work, and Task277B not-ready/zero-credit status.
English is canonical; no exception is recorded.

## Task 36P264 Bilingual Contract Parity

The [EN contract](../../task_contracts/en/CORE-SOURCE-PROPERTY-IMPLEMENTATION-OWNER-36P264.md)
and [JA companion](../../task_contracts/ja/CORE-SOURCE-PROPERTY-IMPLEMENTATION-OWNER-36P264.md)
synchronize the same no-Core-33-item disposition, Core-36 ownership,
selector/carrier anti-alias boundary, gap classifications, protected artifacts,
zero-credit state, same-source lower-checker prerequisite, Task263 exclusion,
and later Core-33/34 owner handoff. English is canonical; no exception is
recorded.

## Guard Decision

No new Rust lint guard is added in Task 23 or closeout. The file-pair set is
small and all current pairs are enumerated in this audit. Task 22 already added a focused
guard for the highest-risk source/spec audit pair: public module coverage,
public-item mentions, and CORE-AUDIT gap synchronization. A broader bilingual
guard can be added later if future tasks introduce a larger documentation
matrix, but adding one here would turn this planned docs-only closeout update
into a Rust-test change without a concrete coverage gap.

## Verification

Docs-only verification for this task:

- `git diff --check` before staging.
- `git diff --cached --check` after explicit path staging.

## Task 33I263 Bilingual Contract Parity

The [EN contract](../../task_contracts/en/CORE-SOURCE-STRUCTURE-ITEM-CONTEXT-33I263.md)
and [JA companion](../../task_contracts/ja/CORE-SOURCE-STRUCTURE-ITEM-CONTEXT-33I263.md)
synchronize the same two structure associations, sole Derived-to-Base local
dependency, standalone by-value API, default-deny boundary, prohibited
semantics, protected artifacts, zero-credit status, and Task277B deferral.
English is canonical; no exception is recorded.

## Task 33I264 Bilingual Contract Parity

The [EN contract](../../task_contracts/en/CORE-SOURCE-PROPERTY-CARRIER-ITEM-CONTEXT-33I264.md)
and [JA companion](../../task_contracts/ja/CORE-SOURCE-PROPERTY-CARRIER-ITEM-CONTEXT-33I264.md)
synchronize the same singleton Task264 carrier/Core association, complete
by-value checker owner, scalar `carrier_item()` API, error precedence,
default-deny oracle, exact debug grammar, private two-test boundary, Core34
handoff, zero-credit status, and semantic exclusions. English is canonical;
no exception is recorded.
