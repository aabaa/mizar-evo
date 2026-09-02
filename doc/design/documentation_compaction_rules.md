# Documentation Compaction Rulebook (September 2026, Audit 2)

> Canonical language: English (status/process document; no Japanese
> companion per the September 2026 status-document language policy,
> user-approved 2026-09-01).

## Purpose And Authority

Between 2026-07 and 2026-09, `doc/design` grew from 239k to 424k lines
(+78%) while Step 5 executed micro-tasks: each task added ~550-750 doc
lines across up to 14 copies (7 documents x en/ja), and review boilerplate
such as "NO FINDINGS" accumulated 503 occurrences in the `mizar-checker`
English documents alone. This rulebook defines the compaction rules and
the migration inventory. It is a derived process document under
[autonomous_crate_development.md](./autonomous_crate_development.md): it
cannot change language behavior, test intent, expectations, trace status,
or semantic/coverage credit, and no rule here weakens the authority order
or the fail-closed rules.

Execution is delegated to Codex. The audit that authored this rulebook
does not execute the inventory; each batch is a separate light-tier task
(see Gate Tiering in the protocol) with one-way promotion to full gates
if it turns out to touch a protected surface.

## Rules

### Rule 1 — Single ownership of status facts

A task's status, completion evidence, measured counts, and review
outcomes live in exactly one place: the paired task contract (the EN/JA
pair counts as one logical owner; post-policy status documents are
EN-only). Every other document — crate plans, todos, module docs, audits,
harness docs — links to the contract instead of restating status. In
particular:

- per-task completion narratives in crate todos are replaced by one
  sequencing row plus a contract link;
- "NO FINDINGS" and equivalent review-outcome boilerplate is recorded
  once in the contract's review section and nowhere else;
- roadmap files record ordering and open/closed state only.

### Rule 2 — Tabular ledgers for mechanical measurements

Mechanical, regenerable measurements (case totals, requirement counts,
line counts, hashes, per-case activation state) are kept in versioned
TSV/table ledgers, one ledger per measurement family, not in prose.
Existing precedents:
[`tests/coverage/audit1_frontend_gaps.tsv`](../../tests/coverage/audit1_frontend_gaps.tsv),
[`tests/coverage/step5_activation_map.tsv`](../../tests/coverage/step5_activation_map.tsv),
[`task_contracts/legacy_compactions.tsv`](./task_contracts/legacy_compactions.tsv).
Prose cites a ledger row; it does not repeat the numbers in every
affected document. A ledger states its generator or source of truth in a
header comment. Ledger tooling that parses `.miz` sources must not assume
every committed `.miz` parses until frontend gaps G5 and G1 close (see
the Step 5A tasks in [todo.md](./todo.md)).

### Rule 3 — Archive split for frozen historical logs

Frozen historical logs (completed-task narratives, superseded audit
waves, closed checklists kept only as evidence) move verbatim to
`doc/design/archive/` with a dated header naming the source document and
move date; the live document keeps a one-line pointer. Archive files are
frozen: never extended, never cited as sequencing authority, EN-only.
Precedent:
[archive/step5_microtask_narrative.md](./archive/step5_microtask_narrative.md).
A live document that mixes frozen narrative with owned current state is
split, not archived whole; the owned current state stays live.

### Rule 4 — Language scope for status and audit documents

Bilingual EN/JA maintenance is mandatory only for `doc/spec/{en,ja}` and
`doc/design/architecture/{en,ja}`. Status, audit, process, roadmap, and
archive documents are English-only; where a Japanese companion path
already exists (or a bilingual-area layout requires one), it is a pointer
stub linking the canonical English file, per the audit-1 precedent
(`doc/design/mizar-test/ja/semantic_bridge_corpus_map.md`). Module design
documents in existing bilingual crate trees keep their current pairing;
converting them is out of scope for this rulebook. This scope is
formalized in `.claude/CLAUDE.md` and AGENTS.md (user approval
2026-09-01).

### Rule 5 — Budgets

After compaction, a semantic task should add at most ~150 doc lines per
commit (contract + owner-document deltas combined); a compaction batch
removes one coherent family per commit. Steady-state targets: each crate
todo below 1,500 lines; no document restates another owner's status
facts.

## Migration Inventory

The migration inventory is the versioned ledger
[`task_contracts/audit2_compaction_inventory.tsv`](./task_contracts/audit2_compaction_inventory.tsv)
(Rule 2 applies to the inventory itself). Each row is one Codex batch:
one coherent duplication family, one light-tier task-only commit, sized
so the batch is reviewable in a single equivalence pass. Whole-ATX-section
migrations continue to register in
[`task_contracts/legacy_compactions.tsv`](./task_contracts/legacy_compactions.tsv)
under schema 2; shapes schema 2 cannot express (same-file same-task
second sections, mixed owner-local sections, paragraph-level duplicates)
are classified in the batch contract and left intact until a separately
reviewed schema/ownership prerequisite exists, exactly as the temporary
gate closeout recorded.

Batch constraints (all batches):

- never edit `doc/spec`, `.miz` sources, expectations,
  `spec_trace.toml` rows/status, or production Rust;
- never close a gap or activate an oracle case;
- every removed fact retains one live owner or an archive location;
- local link/fragment checks and `cargo test --workspace` pass after
  every batch;
- measured baselines (line counts before/after) are recorded in the
  batch contract, not fanned out.

## Measured Baseline (2026-09-02)

| Surface | Measure |
|---|---:|
| `doc/design` total | 424,547 lines (pre-audit-2) |
| `doc/design/todo.md` frozen addenda (post-audit-2 lines 850-3819) | ~2,970 lines |
| `mizar-checker` EN docs total | 68,000 lines |
| `mizar-checker/en/00.crate_plan.md` | 17,786 lines |
| `mizar-checker` todo EN/JA | 7,161 / 6,730 lines |
| `mizar-test/en/module_boundary_audit.md` | 13,430 lines |
| `mizar-test/en/00.crate_plan.md` | 8,578 lines |
| `mizar-test` todo EN/JA | 4,042 / 3,718 lines |
| crate todo JA companions total | 20,464 lines |
| "NO FINDINGS" occurrences (`mizar-checker` EN) | 503 |
| files containing "NO FINDINGS" (`doc/design`) | 287 |
| task contracts | 125 EN/JA pairs, 3.5 MB |
