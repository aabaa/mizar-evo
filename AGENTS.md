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

## Delivery Efficiency And Implementation Minimality

Release throughput is a quality constraint. Implement the smallest change that
fully satisfies the frozen scope and required gates.

- Do not add speculative generality, premature abstractions, compatibility
  layers, adapters, fields, tests, documentation, or refactors unless required
  by authority, frozen acceptance criteria, a reproducible defect, or an
  in-scope review finding.
- Reviews must separate blocking in-scope findings from optional follow-up.
  Fix blocking findings, but do not expand the active task for optional polish
  or reopen settled decisions without new contradictory evidence.
- Every additional probe or verification rerun must resolve a stated
  uncertainty. Prefer focused checks first; after a documentation-only or
  evidence-only edit, do not rerun unchanged broad suites unless policy, CI
  parity, or changed risk requires it.
- Once the acceptance criteria and required gates pass with no blocking
  findings, stage and complete promptly. "While here" cleanup is a separate
  task.
- These limits never relax the authority order, soundness, fail-closed
  behavior, protected artifacts, required independent reviews, or hard gates.

## Specification-Driven Autonomous Crate Development

For crate-wide autonomous development, follow the protocol in
[`doc/design/autonomous_crate_development.md`](doc/design/autonomous_crate_development.md).
That protocol constrains the workflow above when the task touches language
behavior or crate-level implementation scope, and it is the canonical owner
of: the authority order (`doc/spec/en/` > `tests/**/*.miz` >
`tests/coverage/spec_trace.toml` > `tests/**/*.expect.toml` > `doc/design/`
> `crates/`), the test-first `.miz` addition rules, the
no-chat-only-language-changes list, crate kickoff and the Crate Plan
(create or update `doc/design/<crate>/en/00.crate_plan.md` before starting;
do not begin implementation if the plan finds blocking specification
problems; keep the plan a compact index and put task orchestration in the
paired task contract), drift and gap classification (classify before
editing; `repo_metadata_conflict` is report-only), Gate Tiering, the crate
exit gates, and the quality score. Read those protocol sections rather than
relying on this summary.

Non-negotiables restated for convenience (the protocol text is canonical):

- `doc/spec/en/` and `.miz` tests are the primary human-reviewed artifacts;
  `doc/design/` and `crates/` are derived. Observed source behavior is not
  normative; repair derived artifacts toward the specification and tests,
  never the reverse.
- Never modify `doc/spec`, existing `.miz` tests, or test expectations
  merely to match current implementation behavior. Changes to language
  behavior (syntax, semantics, types, resolution, overloads, diagnostics,
  parser recovery, expectations, soundness boundaries) must be represented
  in `doc/spec`, tests, or traceability metadata, and are allowed only when
  the task explicitly changes specification or test intent.
- Work is complete only when the gates for its declared tier pass: full
  tier needs all hard gates plus a read-only quality score of at least
  90/100 (a score is invalid if any hard gate fails); zero-credit
  structural transport may use the documented light tier, with one-way
  promotion to full gates the moment the task gains semantic credit or
  touches a protected surface.

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

#### GPT-5.6 Model Routing

Use the [current official OpenAI model guidance](https://developers.openai.com/api/docs/guides/latest-model)
when a session exposes per-agent GPT-5.6 model selection. Model availability,
account limits, and quota accounting are environment properties and must not
be inferred from API pricing or assumed from this policy.

For exposed GPT-5.6 per-agent selection, this subsection takes precedence over
the generic reasoning defaults below. A complete frozen assignment is
sufficient when repository policy does not require a task contract; when a
contract is required, freeze it before delegation.

- Keep GPT-5.6 Sol at the user-requested reasoning setting for the parent when
  the work can decide authority interpretation, unresolved specification or
  test intent, soundness boundaries, public API ownership, semantic acceptance,
  or final hard-gate scoring.
- GPT-5.6 Luna with `xhigh` is eligible for bounded autonomous work only after
  the parent has frozen a complete assignment or task contract. Suitable work
  includes deterministic inventory, count and hash checks, whole-section
  documentation compaction, manifest and link validation, focused verification,
  localized contract-driven implementation, and first-pass independent reviews.
- A Luna agent must not independently resolve a `spec_gap`, derive new test
  intent, invent language or proof semantics, authorize a lower-stage change,
  expand task scope, or accept a disputed finding. Escalate those decisions to
  the parent without editing beyond the frozen scope.
- Use GPT-5.6 Terra with `high` or `xhigh`, when available, as the intermediate
  escalation route for cross-module implementation, precision review, or a
  disagreement that exceeds Luna's bounded assignment but does not yet require
  a parent semantic decision. Treat `high` as the trial baseline and use
  `xhigh` only when representative work shows a measured quality gain. Compare
  the selected setting with one lower setting where the task can be repeated
  safely, and do not broaden the route without no-regression evidence.
- Luna output is subject to the same independent reviews, verification, hard
  gates, and commit rules as output from any other model. Lower model cost or a
  higher reasoning setting never substitutes for evidence and never lowers an
  acceptance threshold.
- Introduce or broaden Luna routing only through representative repository
  trials. Compare task success, material missed findings, repair loops, hard-
  gate agreement, token use, latency, and cost at `xhigh` and, where safe, one
  lower effort. Keep Sol or Terra for a task class until the trial shows no
  material quality regression.
- If Luna is not exposed by the current agent runtime, use the closest eligible
  route above without blocking the task. Record the effective model and
  reasoning setting in the review packet or handoff when model choice matters.

When per-agent reasoning controls are available:

- use the parent setting for authority conflicts, specification gaps,
  soundness boundaries, disputed semantics, and final quality scoring
- use `high` for bounded implementation work and independent specification,
  test-sufficiency, implementation, bilingual, and source/documentation
  reviews after the parent has frozen the task contract, unless the
  model-specific routing above requires `xhigh`
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
- Do not label optional polish or an out-of-scope improvement as a finding;
  report it separately as optional follow-up.
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
- Bilingual EN/JA maintenance is mandatory only for `doc/spec/{en,ja}` and `doc/design/architecture/{en,ja}` (language-scope decision, user-approved 2026-09-01).
- When updating documentation in those bilingual areas, keep the English canonical document and Japanese companion document synchronized in the same change.
- For language specifications, update matching files under `doc/spec/en/` and `doc/spec/ja/`.
- For architecture specifications, update matching files under `doc/design/architecture/en/` and `doc/design/architecture/ja/`.
- Status, audit, process, roadmap, and archive documents elsewhere under `doc/design/` are English-only; where a Japanese companion path exists or a bilingual crate-tree layout expects one, add or keep a pointer stub linking the canonical English file (precedent: `doc/design/mizar-test/ja/semantic_bridge_corpus_map.md`; scope details in `doc/design/documentation_compaction_rules.md`).
- Module design documents under paired `doc/design/<component>/en/` and `doc/design/<component>/ja/` directories keep their current pairing: update matching files when both exist.
- Keep file names aligned across language directories whenever possible.
- If an English document in a bilingual area changes but the Japanese companion cannot be updated in the same change, explicitly note the reason and mark the Japanese document as needing synchronization.
- When adding a new English documentation file in a bilingual area, add the corresponding Japanese companion or a clearly marked Japanese placeholder that links to the canonical English file.

### Canonical Task Contracts And Minimal Deltas

Task contracts, the single-owner documentation rule, the migration policy,
and the `legacy_compactions.tsv` ledger (schema 2) are owned by the
protocol: see "Canonical Task Contracts", "Single-owner documentation
rule", and "Migration policy" in
[`doc/design/autonomous_crate_development.md`](doc/design/autonomous_crate_development.md),
plus
[`doc/design/documentation_compaction_rules.md`](doc/design/documentation_compaction_rules.md)
for status-fact ownership, tabular ledgers, archive splits, and the
language scope. Read those sections rather than relying on this summary.

In brief (the protocol text is canonical):

- A non-trivial autonomous task uses one paired record
  `doc/design/task_contracts/{en,ja}/<task-id>.md` (same id in both trees,
  English canonical; the synchronized pair is one logical derived owner).
  Contracts are derived orchestration records and never override
  `doc/spec/en/`, `.miz` tests, traceability metadata, expectations, or
  the authority order.
- Each derived fact has exactly one live owner; other documents link to it.
  There is no required documentation fan-out count; update a document only
  when its owned durable state changes, and record an explicit no-impact
  decision in the contract instead of making no-op audit edits.
- Historical task sections are frozen logs. They are migrated only by
  separately reviewed compaction batches that preserve every unique fact
  with a live owner, and whole-section migrations are registered in
  `doc/design/task_contracts/legacy_compactions.tsv` (schema 2; the ledger
  records approved migrations and cannot itself authorize deletion).

When a component first links a central task contract, include that EN/JA
pair in the component's bilingual review surface. Claim recursive
task-contract or local-link enforcement only after the repository lint
that performs it passes.

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
