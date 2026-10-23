# Codex Agent Workflow

This repository uses Codex as a task orchestrator. This file is operational
policy; the canonical protocol is
[`doc/design/autonomous_crate_development.md`](doc/design/autonomous_crate_development.md).
Where the two differ, the protocol wins.

## Change Discipline (read first)

- Make the SMALLEST change that satisfies the frozen scope. Every changed line
  must trace to the task, `doc/spec/en/`, or a `.miz` test. No "while here"
  edits, refactors, renames, or reformatting of untouched code.
- Never add a file, public type, field, adapter, or abstraction for a single
  use. Inline first; abstract only at three or more real call sites.
- If you write 200 lines and it could be 50, rewrite it.
- For changes exceeding 300 production lines or adding a public seam, present
  the plan first; this alone does not require user approval.
- Documentation records decisions and boundaries only. Per-run evidence,
  scores, gate tallies, digests, review outcomes, and model/reasoning settings
  belong in the commit body, never `doc/`. Durable model-routing policy lives
  only in the protocol. `cargo test` enforces the documentation ratchet in
  `tests/coverage/doc_volume_baseline.tsv`; rows may only be lowered or removed.
- A task contract is at most 60 lines per language. Link it only from
  `doc/design/todo.md`, crate plans, crate todos, or the coverage audit.
  Module documents never point back at contracts.
- End every commit body with `volume: prod +N test +N doc +N` (added lines from
  `git diff --numstat`: non-test `crates/**/src`, tests and `.miz`/`.toml`
  fixtures, `doc/`).
- Never modify `doc/spec`, existing `.miz` tests, or expectations to match
  current implementation behavior.

## User Invocation

Requests such as the following invoke the full workflow, including sub-agent
review and delegation where available:

```text
<task description>

Codex agent を使い、AGENTS.md のワークフローに従って、完了まで進めてください。
```

Resolve implementation and test-infrastructure gaps autonomously under existing
specification, test intent, and authorized scope. Ask only for unresolved
semantic decisions or approvals required by user constraints or the protocol.

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
next scope, authority/contract paths, unresolved issues, and verification state.
Reference completed work; do not copy its history. Use automatic context
compaction, then read the checkpoint and continue without requesting manual
compaction or resume instructions. Respect user stops, usage limits, and
protocol stop conditions.

## Verification

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Run narrow tests first when sufficient, then the full set before finalizing
Rust changes. Reviewers reuse the parent's command, checked revision/worktree,
exit status, and log location; rerun affected checks if changes or environment
differences invalidate results, or findings need reproduction. Keep full logs
outside `doc/`; report concise results, relevant failures, and reasons/risks for
unrun commands. Required commands and independent reviews remain mandatory.

## Documentation

- English is canonical. Bilingual EN/JA maintenance is mandatory only for
  `doc/spec/{en,ja}` and `doc/design/architecture/{en,ja}` (user decision,
  2026-09-01). Module documents in existing paired trees keep their pairing;
  other `doc/design/` documents are English-only with pointer stubs where a
  companion path exists. Details: `.claude/CLAUDE.md` and
  `doc/design/documentation_compaction_rules.md`.
- Task contracts live at `doc/design/task_contracts/{en,ja}/<id>.md` (protocol:
  "Canonical Task Contracts"). The Japanese file is a pointer stub.
- Each derived fact has one live owner; other documents link to it. Historical
  task sections are frozen logs migrated only through
  `doc/design/task_contracts/legacy_compactions.tsv`.

## Commits

Inspect the worktree; include only task-related changes and never revert
unrelated user changes. Use a Conventional Commits subject (`feat:`, `fix:`,
`docs:`, `test:`) and a short body ending with the `volume:` line. Commit by
default when the full workflow or autonomous crate development was requested;
otherwise only when the user asks (`commit`, `commitまで`, `コミットまで`).

## Final Response

Report what changed, which reviews ran and what the volume review removed,
which verification commands passed or could not run, the commit hash, and the
next-task handoff prompt with its recommended reasoning setting.
