# Codex Agent Workflow

This repository uses Codex as a task orchestrator. For implementation tasks, follow the workflow below unless the user gives a more specific instruction.

## User Invocation

When the user says something like the following, treat it as a request to run the full workflow in this file:

```text
<task description>

Codex agent を使い、AGENTS.md のワークフローに従って、完了まで進めてください。
```

This wording is an explicit request to use sub-agents for the review and delegation phases where they are available and useful.

If the task is ambiguous enough that implementation would be risky, ask one concise clarifying question. Otherwise, make reasonable assumptions and proceed.

## Full Task Workflow

For each task, complete these phases in order:

1. Write or update the implementation specification for the requested task. For small, localized changes, a concise specification in the chat is enough. For changes that alter documented behavior, architecture, or language semantics, update the relevant file under `doc/` instead.
2. Review the implementation specification and relevant documentation for completeness, clarity, and consistency.
3. If the specification or documentation review finds gaps, update the implementation specification or documentation carefully and repeat the review until there are no findings.
4. Implement the requested task.
5. Review whether tests are sufficient compared with the relevant specification.
6. If the test review finds gaps, expand tests carefully and repeat the test review until there are no findings.
7. Review the full implementation for bugs, regressions, design mismatches, and missing edge cases.
8. If the implementation review finds issues, fix them and repeat the implementation review until there are no findings.
9. Review whether source code and documentation still agree, including whether
   `doc/design/spec_coverage_audit.md` must be updated for changed
   specification/design coverage, follow-up ownership, or deferred coverage
   status.
10. If the source-documentation consistency review finds issues, fix them and repeat the consistency review until there are no findings.
11. Update `doc/design/spec_coverage_audit.md` when the task changes how
    `doc/spec/en/` chapters are covered by design documents, tests,
    traceability metadata, owner crates, or follow-up tasks. If no audit change
    is required, keep the file unchanged and mention that in the review or
    final response when relevant.
12. Run the relevant verification commands.
13. Prepare a handoff prompt for the next task so it can be started in a separate chat. Include a recommended reasoning setting for the next task, a short rationale, and any conditions that would justify raising or lowering that setting.
14. Inspect the worktree, prepare a commit message, and commit the completed change when the user invoked this full workflow or requested autonomous crate development, unless the user asks not to commit. For tasks outside this workflow, commit only when the user explicitly requested committing, for example by saying `commit`, `commitまで`, or `コミットまで`.

## Specification-Driven Autonomous Crate Development

For crate-wide autonomous development, follow the protocol in
[`doc/design/autonomous_crate_development.md`](doc/design/autonomous_crate_development.md).
That protocol constrains the workflow above when the task touches language
behavior or crate-level implementation scope.

For language behavior, the authority order is:

1. `doc/spec/en/`
2. `tests/**/*.miz`
3. `tests/coverage/spec_trace.toml`
4. `tests/**/*.expect.toml`
5. `doc/design/`
6. `crates/`

`doc/spec/en/` and `.miz` tests are the primary human-reviewed artifacts.
`doc/design/` and `crates/` are derived artifacts. Source behavior may be
observed during inventory, but it is not normative. If derived artifacts
disagree with the specification or tests, repair the derived artifacts toward
the specification and tests.

Agents must not modify `doc/spec`, existing `.miz` tests, or test expectations
merely to match current implementation behavior. Test-first `.miz` additions
are allowed only under the protocol rules when they are derived from existing
`doc/spec/en/` requirements or close a classified `test_gap`; expectation and
traceability metadata for those new tests may be added when they express the
spec-derived test intent.

Changes to syntax, static semantics, proof semantics, type behavior, name
resolution, overload behavior, diagnostics, parser recovery, existing test
expectations, or soundness-boundary behavior must be represented in `doc/spec`,
tests, or traceability metadata as appropriate, and are allowed only when the
task explicitly changes specification or test intent.

Before crate-wide autonomous work starts, create or update
`doc/design/<crate>/en/00.crate_plan.md`. The plan must cover crate
responsibility, specification references, relevant tests, design/source
inventory, known gaps and drift, task decomposition, the expected impact on
`doc/design/spec_coverage_audit.md`, and exit criteria. Do not begin
implementation if the plan finds missing or contradictory specification that
blocks the crate. Keep the plan as a compact crate-level index. For a
non-trivial logical task, put detailed task-specific orchestration in the
paired task contract described under Documentation Expectations and link it
from the plan instead of copying the contract into the plan and multiple
component documents.

Before editing, classify disagreements as `spec_gap`, `test_gap`,
`design_drift`, `source_drift`, `source_undocumented_behavior`,
`test_expectation_drift`, `boundary_violation`, or `repo_metadata_conflict`.
Report `repo_metadata_conflict` only; do not repair it automatically.

Crate-wide autonomous work is complete only when all hard gates in the protocol
pass and a read-only review assigns a quality score of at least 90/100. A score
is invalid if any hard gate fails.

## Agent Delegation

Use sub-agents when the current Codex session has access to agent delegation and the user has requested this workflow. Keep the parent agent responsible for orchestration, integration, final verification, and the final user response.

Recommended delegation pattern:

- Use a worker agent for bounded implementation subtasks only when write scopes can be kept clear.
- Use a review-only sub-agent prompt for the implementation specification and documentation review.
- Use a review-only sub-agent prompt for the test sufficiency review.
- Use a review-only sub-agent prompt for the full implementation review.
- Use a review-only sub-agent prompt for the source/documentation consistency review.

When delegating, give each agent a concrete, self-contained task. If an agent edits files, assign a clear ownership area and tell it not to revert unrelated edits or changes made by other agents.

Do not delegate the immediate critical-path task if the parent agent needs that result before it can make progress. In that case, do the work locally and use agents for sidecar review or independent checks.

### Reasoning And Context Economy

The parent agent keeps the reasoning setting requested by the user and remains
responsible for authority interpretation, semantic decisions, integration,
final verification, staging, and committing. A lower reasoning setting for a
sub-agent must never silently lower the parent setting or relax a review gate.

When per-agent reasoning controls are available:

- use the parent setting for authority conflicts, specification gaps,
  soundness boundaries, disputed semantics, and final quality scoring
- use `high` for bounded implementation work and independent specification,
  test-sufficiency, implementation, bilingual, and source/documentation
  reviews after the parent has frozen the task contract
- use `medium` or lower only for deterministic inventory or mechanical checks
  that cannot decide language behavior, test intent, public API, or acceptance
- escalate any ambiguity, authority disagreement, or proposed scope expansion
  back to the parent before editing

Prefer a no-history or short-history sub-agent context when the task can be
made self-contained. Give the agent a compact review packet containing the
task-contract path, exact authority references, affected-file scope,
prohibited behavior, relevant diff, and review question. Do not copy the full
conversation or pre-state the desired review conclusion. Reuse the same
reviewer for a finding-specific re-review when appropriate; keep the required
first-pass review roles independent.

## Review Standards

Review-only sub-agents must use a code-review stance:

- Lead with findings.
- Order findings by severity.
- Include file and line references where applicable.
- Focus on bugs, behavioral regressions, specification mismatches, missing tests, and documentation drift.
- If there are no findings, say so clearly.
- Mention residual risk or unrun tests briefly.

The parent agent should treat reviewer findings as actionable until resolved. After fixes, repeat the relevant review phase. Stop repeating only when the reviewer reports no findings or when a remaining issue is explicitly accepted by the user.

## Test And Verification Expectations

Prefer the repository's existing commands and patterns. For this Rust workspace, the default verification command is:

```sh
cargo test
```

Check formatting before finalizing Rust changes:

```sh
cargo fmt --check
```

Also run Clippy before finalizing Rust changes because CI commonly reports Clippy findings:

```sh
cargo clippy --all-targets --all-features -- -D warnings
```

Run narrower tests first when they are clearly sufficient for the active change, then run broader verification before finalizing if the change has meaningful cross-module risk. For Rust source changes, run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` before finalizing unless the change is documentation-only or the commands cannot be run.

If a command cannot be run, explain why in the final response and describe the remaining risk.

## Documentation Expectations

Follow the repository documentation policy:

- English documentation is canonical.
- When updating bilingual documentation, keep the English canonical document and Japanese companion document synchronized in the same change.
- For language specifications, update matching files under `doc/spec/en/` and `doc/spec/ja/`.
- For architecture specifications, update matching files under `doc/design/architecture/en/` and `doc/design/architecture/ja/`.
- For component design documents, update matching files under paired `doc/design/<component>/en/` and `doc/design/<component>/ja/` directories when both exist.
- Keep file names aligned across language directories whenever possible.
- If an English document changes but the Japanese companion cannot be updated in the same change, explicitly note the reason and mark the Japanese document as needing synchronization.
- When adding a new English documentation file in a bilingual area, add the corresponding Japanese companion or a clearly marked Japanese placeholder that links to the canonical English file.

### Canonical Task Contracts And Minimal Deltas

For an autonomous logical task that needs a frozen contract, crosses owner
documents, or carries exact API, test, file, diagnostic, count, or hash
requirements, create one paired task-contract record:

```text
doc/design/task_contracts/en/<task-id>.md
doc/design/task_contracts/ja/<task-id>.md
```

The English file is canonical and the Japanese companion must be logically
synchronized in the same change. Use the same task id in both trees, matching
`[A-Za-z0-9][A-Za-z0-9._-]*`. These files are derived planning and review
records; they do not override `doc/spec/en/`, `.miz` tests, traceability
metadata, expectations, or the authority order.

Document each fact once in its owning derived document:

- the task contract owns task identity, authority links, dependencies,
  readiness or blockers, primary and lower-stage owners, consumers, relevant
  existing or test-first tests, gap classifications, scope, forbidden behavior,
  semantic deferrals, affected-artifact index, audit-impact decision, review
  and verification plans, baseline and expected count/hash impact, exit
  criteria, completion evidence, status, stable owner-section links, and next
  handoff
- the crate plan owns stable crate responsibility, crate-level readiness, and
  the ordered compact task-link index
- a module design document owns durable module API, invariants, validation,
  ownership, and module-local test design
- a runner or harness document owns only its private route and consumer delta
- todo, boundary, bilingual, traceability, source, semantic, and coverage
  audits record only changes to the state they own, or remain unchanged
- baseline and expected count/hash impact are frozen in the task contract;
  final measured counts and hashes are recorded once in the contract or
  required exit report and linked rather than copied into every affected
  document

The synchronized EN/JA contract pair is one logical derived owner for this
rule, not two competing sources.

There is no required documentation fan-out count. Do not append an identical
task narrative to every component document merely to demonstrate
synchronization. Update a document only when its owned durable state changes,
and use stable links to the task contract and owner-local sections for shared
context. An explicit no-impact decision belongs in the task contract; it does
not require a no-op edit to the corresponding audit.

Apply this structure prospectively. An ordinary semantic task must not
bulk-rewrite historical task logs. Migrate an active or reopened task only in a
separately reviewed documentation change that preserves every unique API,
test-intent, classification, deferral, and traceability claim before removing
duplicates.

An explicitly user-authorized legacy-evidence compaction may batch multiple
completed tasks in one logical documentation task only when they form one
coherent duplication family. Freeze the exact old-section-to-new-owner redirect
map and baseline counts first; create a paired EN/JA contract for each migrated
task; preserve every adjacent owner-local API, invariant, runner, audit,
traceability, coverage, and sequencing fact; replace only the mapped shared
evidence with language-local links; validate local targets and fragments; and
prove that specification, test intent, trace status, coverage credit, source,
and active behavior are unchanged. The batch needs its own paired contract,
equivalence and bilingual reviews, hard gates, and task-only commit. It does not
authorize a repository-wide historical rewrite or let a semantic task absorb
unrelated documentation cleanup.

Record every completed whole-section legacy compaction in
`doc/design/task_contracts/legacy_compactions.tsv`; keep the lint consumer
generic rather than adding task IDs, file lists, or index rows to Rust. Schema
version 1 covers only exact whole ATX H2-H6 section replacements with explicit
source paths, forbidden headings, language-local redirects, neighboring
heading anchors, Task Index rows, counts, and an expanded-inventory hash. Do
not force a mixed owner-local section or paragraph-only cleanup into that
schema. Extend the schema only in a separately reviewed prerequisite. The
manifest is derived enforcement data and cannot authorize a migration that its
paired task contracts and equivalence review did not already approve.

When a component first links a central task contract, include that EN/JA pair
in the component's bilingual review surface. Claim recursive task-contract or
local-link enforcement only after the repository lint that performs it passes.

## Commit Expectations

Before committing, inspect the worktree and make sure only task-related changes are included. Do not revert unrelated user changes.

When the user invokes the full AGENTS.md workflow or requests autonomous
crate-level development, committing the completed change is permitted and is the
default final step after required reviews and verification pass, unless the user
asks not to commit. For smaller ad hoc tasks outside this workflow, commit only
when the user explicitly requests it.

Use a concise Conventional Commits-style subject, for example:

```text
feat: add lexer token coverage report
fix: correct parser recovery for nested blocks
docs: sync parser design notes
```

When the change is broad enough to need context, include a short commit body explaining the main changes.

## Final Response

When the task is complete, report:

- What changed.
- Which reviews were run and whether they ended with no findings.
- Which verification commands passed or could not be run.
- The commit hash if a commit was created.
- The next-task handoff prompt, including the recommended reasoning setting for that task.
