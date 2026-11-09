# Codex Agent Workflow

This repository uses Codex as a task orchestrator. This file is operational
policy; the canonical protocol is
[`doc/design/autonomous_crate_development.md`](doc/design/autonomous_crate_development.md).
Where the two differ, the protocol wins.

## Change Discipline (read first)

- Make the SMALLEST change that satisfies the frozen scope. Every changed line
  must trace to the task, `doc/spec/en/`, or a `.miz` test. No "while here"
  edits, refactors, renames, or reformatting of untouched code. If you write
  200 lines and it could be 50, rewrite it.
- Never add a file, public type, field, adapter, or abstraction for a single
  use. Inline first; abstract only at three or more real call sites.
- For changes exceeding 300 production lines or adding a public seam, present
  the plan first; this alone does not require user approval.
- Documentation records decisions and boundaries only. Per-run evidence,
  scores, gate tallies, digests, review outcomes, and model/reasoning settings
  belong in the commit body, never `doc/`. `cargo test` enforces the ratchet
  in `tests/coverage/doc_volume_baseline.tsv`; rows may only be lowered or
  removed.
- A task contract is at most 60 lines per language, linked only from
  `doc/design/todo.md`, crate plans, crate todos, or the coverage audit.
  Module documents never point back at contracts.
- Declare the gate tier (full, pattern, or light) in the contract or commit
  body. A slice that repeats a reviewed family's pattern for one more input
  shape is pattern tier; one family contract covers its slices through its
  ledger under `tests/coverage/`, and a slice updates its ledger row, never a
  contract line. Bundle three to five identical-shape slices (under about
  thirty production lines each, no profile or baseline retirement) into one
  task and one commit (protocol: "Gate Tiering", "Canonical Task Contracts").
- Module and harness documents change only on a boundary change (public API,
  type, or field; diagnostic class; owner module; accepted-input rule; exit
  criterion). A slice that makes a sentence stale deletes it; the
  `fixture_constants` ratchet refuses new byte ranges, `-byte` sizes, and
  Surface row counts under `doc/design`.
- End every commit body with `volume: prod +N test +N doc +N` (added lines from
  `git diff --numstat`: non-test `crates/**/src`, tests and `.miz`/`.toml`
  fixtures, `doc/`).
- Never modify `doc/spec` or tests merely to match implementation behavior.
  Specification-derived test corrections follow the protocol's
  [Specification-Derived Test Maintenance](doc/design/autonomous_crate_development.md#specification-derived-test-maintenance).

## User Invocation

A request ending with "Codex agent を使い、AGENTS.md のワークフローに従って、
完了まで進めてください。" invokes the full workflow, including sub-agent review
and delegation. Resolve implementation and test-infrastructure gaps
autonomously under existing specification, test intent, and authorized scope.
Ask only for unresolved semantic decisions or approvals required by user
constraints or the protocol.

## Task Workflow

Follow the protocol's [Task Workflow](doc/design/autonomous_crate_development.md#task-workflow),
[Review Standards](doc/design/autonomous_crate_development.md#review-standards),
and applicable authority, contract, gate, kickoff/exit, and migration rules.
Read relevant specification, tests, and owner documents; expand as needed,
without loading a full repository map or completed-task history by default.
For migrations, also read `doc/design/documentation_compaction_rules.md`.

## Delegation

The parent owns orchestration, integration, final verification, staging, and
committing. Follow [model routing](doc/design/autonomous_crate_development.md#delegation-and-model-routing)
and give each sub-agent a self-contained [review packet](doc/design/autonomous_crate_development.md#economical-review-packets)
with a clear write scope and no pre-stated conclusion.

After each task commit, save a concise checkpoint: commit, uncommitted changes,
next scope, authority/contract paths, unresolved issues, and verification state
(last full-success tree and pending task count). Reference completed work; do
not copy its history. After automatic context compaction, read the checkpoint
and continue without requesting manual compaction or resume instructions.
Respect user stops, usage limits, and protocol stop conditions. A bounded scope
instruction expires when its candidates are exhausted; then select the next
open item from `doc/design/todo.md` in sequential-plan order instead of waiting
for a new instruction.

## Verification

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test -p mizar-checker --test lint_policy
cargo test -p mizar-test --test lint_policy
cargo test
```

Follow the canonical [Verification Cadence](doc/design/autonomous_crate_development.md#verification-cadence).
Its operational summary is:

- Every changed task runs fmt, Clippy, both `lint_policy` suites (inventory,
  ledger, link, and doc-volume checks), the owner test group, and a short
  end-to-end check for executable changes. Reuse evidence only with unchanged
  relevant inputs and execution conditions.
- Pattern slice: one post-implementation review (a pre-implementation review
  only when the plan departs from the predecessor); no delta review or
  before/after measurement probes. Add the family oracle/metamorphic cases;
  corpus, Driver, and docs groups run every third slice, with the volume
  review, or with every bundle (protocol: "Pattern gates").
- Select function and cost separately using the protocol command table; heavy
  groups are additional, not cumulative. Run affected long tests immediately
  and record deferred groups with reasons and deadlines.
- Full `cargo test` is not a per-task step. Run it only on a named cadence
  trigger (tenth repository-changing task since the last full success,
  integration/exit, verification-policy or selection change, full-failure
  repair, named unbounded impact) and record the trigger. Full must also pass
  before shared push/merge, release, or crate/milestone exit. Never reset the
  pending count on restart or branch change; unknown state requires full.
  Local commits may be "focused checks passed; full regression pending".
- Full failure stops feature work until repair and full success. Keep CI
  running full on every push/PR.

Reviewers reuse the parent's command, checked revision/worktree, exit status,
and log location, rerunning only what changes or findings invalidate. Keep full
logs outside `doc/`; report concise results, failures, and reasons for unrun
commands.

## Documentation

- English is canonical. Bilingual EN/JA maintenance is mandatory only for
  `doc/spec/{en,ja}` and `doc/design/architecture/{en,ja}`; module documents in
  existing paired trees keep their pairing; other `doc/design/` documents are
  English-only with pointer stubs. Details: `.claude/CLAUDE.md` and
  `doc/design/documentation_compaction_rules.md`.
- Task contracts live at `doc/design/task_contracts/{en,ja}/<id>.md`; the
  Japanese file is a pointer stub.
- Each derived fact has one live owner; other documents link to it. Historical
  task sections are frozen logs migrated only through
  `doc/design/task_contracts/legacy_compactions.tsv`.

## Commits

Inspect the worktree; include only task-related changes and never revert
unrelated user changes. Use a Conventional Commits subject (`feat:`, `fix:`,
`refactor:`, `docs:`, `test:`) and a short body ending with the `volume:` line.
Commit by default when the full workflow or autonomous crate development was
requested; otherwise only when the user asks (`commit`, `コミットまで`).
After each authorized commit, push to the current branch's upstream before
starting the next task. Follow the pre-push verification requirements above;
if push is blocked, report the reason and leave the task incomplete.

## Final Response

Report what changed, which reviews ran and what the volume review removed,
which verification commands passed or could not run, the commit hash, and the
next-task handoff prompt with its recommended reasoning setting.
