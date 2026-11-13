# Codex Agent Workflow

Muse is the outer orchestrator; Codex executes one dispatched task per run.
This file is operational policy; the canonical protocol is
[`doc/design/autonomous_crate_development.md`](doc/design/autonomous_crate_development.md).
Where the two differ, the protocol wins, except on the Dispatch Boundary below.
Read only the protocol sections a task needs; crate kickoff, exit, scoring, and
migration sections apply only to those task kinds.

## Dispatch Boundary

A dispatch comes from Muse, or from the user acting in that role (including a
request ending with "Codex agent を使い、AGENTS.md のワークフローに従って、
完了まで進めてください。"). It invokes the full workflow, including sub-agent
review and delegation, for one unit: a single task contract, which may bundle
three to five identical-shape slices.

A dispatch carries:

- the task contract path (or task id and scope for a chat-specified change)
  and its authority paths;
- the gate tier, when already decided;
- verification state: last full-success revision and outstanding changed-task
  count;
- `push: yes|no`;
- the model and reasoning setting.

Derive a missing field from the contract and `git log -5 --format=full`, and
say so in the final response; a missing field alone is not a reason to stop.

- Muse owns task selection, cross-task sequencing, continuation, and push
  timing. Codex owns the dispatched unit: workflow, sub-agents, integration,
  verification, and commit.
- Never select another task or read `doc/design/todo.md` to find one. When the
  unit is committed or stopped, write the final response and end the run.
- Resolve implementation and test-infrastructure gaps autonomously under the
  existing specification, test intent, and scope. Stop only when a protocol
  [stop condition](doc/design/autonomous_crate_development.md#autonomous-design-decisions-and-stop-conditions)
  applies or the contract cannot be identified from the dispatch; then report
  instead of substituting other work.
- The dispatch's model and reasoning setting override every default. The
  protocol's routing table governs only sub-agents inside the task.

## Change Discipline

- Make the SMALLEST change that satisfies the frozen scope. Every changed line
  must trace to the task, `doc/spec/en/`, or a `.miz` test. No "while here"
  edits, refactors, renames, or reformatting of untouched code. If you write
  200 lines and it could be 50, rewrite it.
- Never add a file, public type, field, adapter, or abstraction for a single
  use. Inline first; abstract only at three or more real call sites.
- For changes exceeding 300 production lines or adding a public seam, present
  the plan first; this alone does not require approval.
- Documentation records decisions and boundaries only; per-run evidence,
  scores, digests, review outcomes, and model settings go in the commit body.
  The `doc_volume_baseline.tsv` ratchet enforces this.
- Declare the gate tier (full, pattern, or light) before implementation. A
  further slice of a reviewed family is pattern tier unless it fails a named
  pattern criterion. A slice updates its family ledger row; module and harness
  documents change only on a boundary change (protocol: "Gate Tiering",
  "Canonical Task Contracts").
- Never modify `doc/spec` or tests merely to match implementation behavior.
  Specification-derived test corrections follow the protocol's
  [Specification-Derived Test Maintenance](doc/design/autonomous_crate_development.md#specification-derived-test-maintenance).

## Task Workflow And Delegation

Follow the protocol's [Task Workflow](doc/design/autonomous_crate_development.md#task-workflow)
and [Review Standards](doc/design/autonomous_crate_development.md#review-standards).
Read the relevant specification, tests, and owner documents; do not load a
repository map or completed-task history by default. For migrations, also read
`doc/design/documentation_compaction_rules.md`.

Codex is the parent: it owns integration, final verification, staging, and
committing. Follow [model routing](doc/design/autonomous_crate_development.md#delegation-and-model-routing)
and give each sub-agent a self-contained [review packet](doc/design/autonomous_crate_development.md#economical-review-packets)
with a clear write scope and no pre-stated conclusion. A sub-agent serves one
review phase or implementation packet, is closed after integration, and never
spawns sub-agents. Each review phase gets at most two re-reviews; then the
parent decides. Keep the parent context small.

Run read-only commands (`rg`, `sed`, `cat`, `git diff/status/log`) inside the
sandbox. Request escalation only for a command that failed there for a
permission reason. If the sandbox itself cannot start (for example
`bwrap: loopback: Failed RTM_NEWADDR`), report it once; do not escalate every
command.

After automatic context compaction, continue the dispatched unit from the
worktree and commit log without asking for resume instructions. Respect user
stops, usage limits, and protocol stop conditions.

## Verification

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test -p mizar-checker --test lint_policy
cargo test -p mizar-test --test lint_policy
cargo test
```

Follow the protocol's [Verification Cadence](doc/design/autonomous_crate_development.md#verification-cadence).
Every changed task runs fmt, Clippy, both `lint_policy` suites, the owner test
group, and a short end-to-end check for executable changes; pattern slices run
the narrower "Pattern gates" set. Full `cargo test` runs only on a named
cadence trigger (including any push and the tenth outstanding task); record
the trigger. A full failure stops feature work until it is repaired.

Reviewers reuse the parent's command, checked tree, exit status, and log
location, rerunning only what changes or findings invalidate. Keep full logs
outside `doc/`.

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

## Commits And Push

Inspect the worktree; include only task-related changes and never revert
unrelated user changes. Commit each dispatched unit locally with a
Conventional Commits subject (`feat:`, `fix:`, `refactor:`, `docs:`, `test:`)
and a short body. The body is the durable checkpoint: it records verification
state (last full-success revision, outstanding changed-task count, trigger)
and ends with `volume: prod +N test +N doc +N` (added lines from
`git diff --numstat`: non-test `crates/**/src`, tests and `.miz`/`.toml`
fixtures, `doc/`).

Push only when the dispatch says `push: yes`. Muse requests a push once per
ten changed tasks, on the task whose full `cargo test` the cadence requires;
the full run must pass on the pushed tree first. If a requested push is
blocked, report the reason and leave the task incomplete.

## Final Response

Use this form, one line per field, and nothing else:

```text
result: done | stopped
commit: <hash> (pushed | local)
changed: <one sentence>
reviews: <phases run; what the volume review removed>
verification: <passed/unrun commands; trigger>; full success <rev>; outstanding <n>
settings: parent <model/effort>; sub-agents <model/effort per role>
derived: <dispatch fields Codex filled in, or none>
next: <recommended task, model, effort>
blocker: <stopped runs only: reason and the decision Muse must make>
```
