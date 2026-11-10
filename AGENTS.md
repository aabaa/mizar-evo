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
- Documentation records decisions and boundaries only; per-run evidence,
  scores, digests, review outcomes, and model settings go in the commit body.
  The `doc_volume_baseline.tsv` ratchet enforces this (protocol:
  "Documentation Volume Ledger").
- Declare the gate tier (full, pattern, or light) before implementation. A
  further slice of a reviewed family is pattern tier unless it fails a named
  pattern criterion; bundle three to five small identical-shape slices into
  one task and commit. A slice updates its family ledger row; module and
  harness documents change only on a boundary change (protocol: "Gate
  Tiering", "Canonical Task Contracts").
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
with a clear write scope and no pre-stated conclusion. Each sub-agent serves
one review phase or implementation packet of one task: close it after
integration, never reuse it for a later task, and never let it spawn its own
sub-agents. Each review phase gets at most two re-reviews; then the parent
decides. Keep the parent context small (protocol: "Economical Review Packets",
"Delegation And Model Routing").

Run read-only commands such as `rg`, `sed`, `cat`, and `git diff/status/log`
inside the sandbox. Request escalation only for a command that failed there for
a permission reason, never preemptively. If the sandbox itself cannot start
(for example `bwrap: loopback: Failed RTM_NEWADDR`), report it to the user
once; do not treat escalating every command as normal operation.

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
In short: every changed task runs fmt, Clippy, both `lint_policy` suites, the
owner test group, and a short end-to-end check for executable changes; pattern
slices run the narrower set under "Pattern gates". Full `cargo test` runs only
on a named cadence trigger, which includes shared push; record the trigger. A
full failure stops feature work until it is repaired.

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
