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
- If a change needs more than 300 production lines or a new public seam, STOP
  and present the plan before writing code.
- Documentation records decisions and boundaries only. Never write evidence,
  scores, gate tallies, digests, review outcomes, model names, or reasoning
  settings into `doc/`. They go in the commit body. `cargo test` enforces this
  through `tests/coverage/doc_volume_baseline.tsv`; its rows may only be
  lowered or removed.
- A task contract is at most 60 lines per language. Link it only from
  `doc/design/todo.md`, crate plans, crate todos, or the coverage audit.
  Module documents never point back at contracts.
- End every commit body with `volume: prod +N test +N doc +N` (added lines from
  `git diff --numstat`: non-test `crates/**/src`, tests and `.miz`/`.toml`
  fixtures, `doc/`).
- Never modify `doc/spec`, existing `.miz` tests, or expectations to match
  current implementation behavior.

## User Invocation

When the user writes something like the following, run the full workflow:

```text
<task description>

Codex agent を使い、AGENTS.md のワークフローに従って、完了まで進めてください。
```

This wording asks for sub-agents in the review and delegation phases where
available. If the task is ambiguous enough that implementation would be risky,
ask one concise clarifying question; otherwise make reasonable assumptions and
proceed.

## Task Workflow

1. Specify. A small localized change is specified in chat. A change to
   documented behavior, architecture, or language semantics updates the
   owning file under `doc/` (protocol: authority order, test-first `.miz`
   additions, task contracts, gate tiering).
2. Specification/documentation review; fix and repeat until no findings.
3. Implement.
4. Test-sufficiency review against the specification; fix and repeat.
5. Implementation review for bugs, regressions, and design mismatches; fix and
   repeat.
6. Volume and scope review (protocol section "Volume And Scope Review"). The
   reviewer returns only removable lines, files, types, tests, and paragraphs.
   Anything not required by `doc/spec`, a `.miz` test, or the frozen contract is
   a blocking finding. Remove and repeat.
7. Source/documentation consistency review, including whether
   `doc/design/spec_coverage_audit.md` coverage, owner, or deferral changed.
   If nothing changed, leave it untouched and write no note.
8. Run verification (below).
9. Commit (below). The next-task handoff, with its recommended reasoning
   setting, goes in the final response only.

## Review Standards

Review-only sub-agents lead with findings ordered by severity, cite file and
line, state "no findings" explicitly, and report optional polish separately.
Excess is a finding of the same rank as a gap. "No findings" means no
unresolved blocking or high finding; medium findings are fixed or deferred with
a reason. Repeat a review phase after fixes until it reports no findings or the
user accepts a remaining issue.

## Delegation

The parent agent owns orchestration, integration, final verification, staging,
and committing. Give each sub-agent a compact self-contained packet (protocol:
"Economical Review Packets") with a clear write scope; never pre-state the
desired conclusion. Model and reasoning routing is defined in the protocol
section "Delegation And Model Routing".

## Verification

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Run narrow tests first when clearly sufficient, then the full set before
finalizing Rust changes. If a command cannot run, say why and state the
remaining risk.

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
