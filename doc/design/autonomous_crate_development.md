# Autonomous Crate Development Protocol

This document defines the minimal protocol for specification-driven autonomous
development at crate scope. It applies when an agent is asked to inventory,
plan, implement, or finish a crate-wide body of work.

The goal is to keep human review focused on the language specification and
`.miz` tests while allowing agents to maintain derived design documents, source,
expectation files, and traceability metadata.

## Authority Order

For language behavior, use this authority order:

1. `doc/spec/en/`
2. `tests/**/*.miz`
3. `tests/coverage/spec_trace.toml`
4. `tests/**/*.expect.toml`
5. `doc/design/`
6. `crates/`

`doc/spec/en/` and `tests/**/*.miz` are the primary human-reviewed artifacts.
`doc/design/` and `crates/` are derived artifacts.

When derived artifacts disagree with higher authority artifacts, repair the
derived artifacts toward `doc/spec` and tests. Agents must not modify
`doc/spec`, existing `.miz` tests, or test expectations merely to match current
implementation behavior.

Changing `doc/spec` or existing `.miz` tests is allowed only when the task is
explicitly a specification or test-intent change.

## Test-First `.miz` Additions

Agents are expected to add `.miz` tests before implementation when an existing
`doc/spec/en/` requirement lacks coverage or the Crate Plan classifies a
`test_gap`. These additions are allowed because they make the specification
reviewable as executable intent before source changes.

Test-first additions must:

- be derived from existing `doc/spec/en/` requirements
- cite the relevant specification or traceability metadata
- include expectation and traceability metadata for new tests as appropriate
- express expected outcomes from the specification, not from current source
  behavior
- be included in the human review surface and Crate Exit Report

Agents must not edit or rebaseline existing `.miz` tests or existing
expectations merely to match current implementation behavior. If adding a test
requires deciding new language behavior, classify the issue as `spec_gap` and do
not invent the behavior.

## No Chat-Only Language Changes

Agents must not introduce or change the following only in chat or only in
implementation:

- syntax
- static semantics
- proof semantics
- type behavior
- name resolution behavior
- overload behavior
- diagnostics
- parser recovery behavior
- test expectations
- soundness-boundary behavior

Such changes must be represented in `doc/spec`, tests, or traceability metadata
as appropriate.

## Crate Kickoff

Before crate-wide autonomous development starts, create or update:

```text
doc/design/<crate>/en/00.crate_plan.md
doc/design/<crate>/ja/00.crate_plan.md
```

English is canonical. Update the Japanese plan in the same change when the
component has a bilingual design tree. If the repository later uses a different
language/design layout, adapt the paths while preserving the same purpose and
logical synchronization.

The Crate Plan must include:

1. crate responsibility
2. relevant specification items
3. relevant tests
4. design/source inventory
5. known gaps and drift
6. expected `doc/design/spec_coverage_audit.md` impact
7. task decomposition
8. exit criteria

Do not begin implementation if the Crate Plan concludes that the crate is
blocked by missing or contradictory specification.

### Crate Plan Template

```md
# Crate Plan: <crate>

## Responsibility

Owned behavior:

Out of scope:

## Specification Items

| Spec ref | Requirement | Status |
|---|---|---|

## Relevant Tests

| Test path | Intent | Spec refs |
|---|---|---|

Planned test-first additions:

## Design And Source Inventory

Design files:

Source files:

Observed behavior:

## Known Gaps And Drift

| ID | Class | Evidence | Action |
|---|---|---|---|

## Specification Coverage Audit Impact

Expected updates to `doc/design/spec_coverage_audit.md`:

## Task Index

| Task | Contract |
|---|---|
| <ID> | [EN contract](../../task_contracts/en/<ID>.md) |

## Exit Criteria

Hard gates:

Verification commands:

Review expectations:
```

Tasks inside a crate should be decomposed by specification requirement or test
obligation, not merely by source module.

The crate plan is the ordered compact crate-level index. Every task placed in
that index has a paired contract: the English plan links the English contract
and the Japanese plan links `../../task_contracts/ja/<ID>.md`. Small localized
work that does not require a contract is not added as a crate-plan task row.
Do not copy purpose, task readiness, status, audit impact, or the complete task
contract into the plan; those facts belong to the contract. Crate-level
responsibility and readiness remain plan-owned.

Each task must update `doc/design/spec_coverage_audit.md` when it changes the
coverage status, design mapping, follow-up owner, or deferred rationale for any
`doc/spec/en/` chapter. If a task has no audit impact, record that explicitly
in its task contract instead of making a no-op audit edit. For small localized
work that requires neither a task contract nor a plan row, record the no-impact
decision in the review or final response when relevant.

## Source Observation

Agents may reverse engineer current source code only to inventory observed
behavior. Observed behavior is not normative.

Any behavior found in source but absent from `doc/spec` and tests must be
classified as `source_undocumented_behavior`. The agent must not convert
observed behavior into intended design unless it is supported by `doc/spec`,
tests, traceability metadata, or explicit human approval.

## Drift And Gap Classification

When artifacts disagree, classify the issue before editing:

- `spec_gap`: behavior appears necessary but is absent from `doc/spec`.
- `test_gap`: behavior is specified but lacks test coverage.
- `design_drift`: `doc/design` disagrees with `doc/spec`, tests, or source.
- `source_drift`: source disagrees with `doc/spec` or tests.
- `source_undocumented_behavior`: source implements behavior absent from
  `doc/spec` and tests.
- `test_expectation_drift`: `.expect.toml` or snapshot expectations disagree
  with `doc/spec` or test intent.
- `boundary_violation`: a crate implements behavior owned by another phase or
  crate.
- `repo_metadata_conflict`: repository metadata, license metadata, dependency
  policy, or release metadata appears inconsistent.

For `repo_metadata_conflict`, report only. Do not repair it automatically.

## Canonical Task Contracts

Use one paired task contract for a non-trivial autonomous task:

```text
doc/design/task_contracts/en/<task-id>.md
doc/design/task_contracts/ja/<task-id>.md
```

`<task-id>` must match `[A-Za-z0-9][A-Za-z0-9._-]*` and be identical in both
trees. The English contract is canonical; the Japanese companion is updated in
the same logical change and links back to it. The English and Japanese
contracts link to the corresponding owning crate plans. If an owner has no
Japanese plan under an adapted non-bilingual layout, the Japanese contract
links the canonical English plan and records that exception. A task contract
is a derived orchestration record and cannot introduce or override language
semantics, test intent, diagnostics, or soundness policy.

The contract must identify:

- task id, status, purpose, primary owner, consumers, dependencies, and
  readiness or blockers
- exact authority references and relevant existing or test-first tests
- classified gaps and drift with evidence
- in-scope and forbidden behavior, semantic deferrals, affected design/source/
  test artifacts, and lower-stage ownership
- stable links to owner-local API, invariant, runner, and test-design sections
  instead of copies of those sections
- `doc/design/spec_coverage_audit.md` impact, or an explicit no-impact decision
- required review roles, verification commands, count/hash impact, exit
  criteria, completion evidence, and next-task handoff

### Single-owner documentation rule

Each derived fact has one live owner. The task contract owns orchestration and
indexes owner-local details; it does not become a second copy of module design.
The synchronized English/Japanese pair counts as one logical derived owner.
Module documents own durable public/private API and invariants, harness
documents own runner routes, traceability documents own manifest relationships,
coverage audits own coverage status and follow-up ownership, boundary audits
own module-layout decisions, bilingual audits own parity evidence, and todo
documents own concise sequencing status. Crate-plan task entries contain only
the ordered task/contract links; the other mandatory crate-level plan sections
continue to own responsibility, specification/test inventory, gaps, audit
expectations, readiness, and exit criteria.

Update only documents whose owned state changes. There is no required fan-out
file count, and repeated boilerplate is not synchronization evidence. Record
measured verification counts and hashes once in the task contract or required
exit report and link to them elsewhere. If an audit has no impact, leave it
unchanged and record the no-impact decision in the contract.

Stable module inventories and public-enum policies remain in their existing
owner documents when required by repository lint. A future task-contract lint
must scan the paired directory recursively before the workflow may claim that
nested task contracts are machine-enforced.

### Migration policy

Apply the contract structure to new tasks. Historical task appendices are
frozen logs and need not be rewritten. Migrate an active or reopened task only
as a separate documentation task: first move every unique contract, API,
test-intent, classification, deferral, traceability, and verification fact to
its designated owner; then replace duplicate blocks with links; finally run an
equivalence and EN/JA review. Git history is not a substitute for a live owner.

## Economical Review Packets

Keep all independent review phases and hard gates. Context economy changes how
evidence is delivered, not what must be reviewed. A compact review packet
contains:

- canonical authority and exact specification/test references
- the EN/JA task-contract paths and diff
- linked owner-document and implementation/test diffs
- gap classifications, scope exclusions, audit/traceability impact, and
  verification status
- the single review question and, for a re-review, the prior finding ids

Prefer independent no-history review agents with self-contained packets. Do
not include the implementer's conclusion or unrelated historical narrative.
Use a finding-specific follow-up review after fixes. Any authority ambiguity,
semantic choice, public-API expansion, lower-stage change, or soundness issue
returns to the parent agent at the user's requested reasoning setting.

## Crate Exit Gates

A crate-wide autonomous development task is complete only when all hard gates
pass:

1. no blocking/high specification inconsistency remains
2. no source behavior remains that is absent from `doc/spec` and tests
3. milestone-owned specification items have existing or test-first `.miz`
   coverage, or explicit deferred reasons
4. test expectations were not changed merely to match current implementation
5. `doc/design` and source are synchronized within the target crate scope
6. crate responsibility boundaries are not violated
7. `doc/design/spec_coverage_audit.md` is updated for any changed spec/design
   coverage, follow-up ownership, or deferred coverage status
8. required verification commands pass, or any unrun command is explicitly
   justified
9. remaining risks are classified as deferred, out of scope, or human-owned

In this protocol, "no findings" means no unresolved blocking/high findings. Low
notes may remain if documented. Medium findings must be fixed or explicitly
deferred with a reason.

## Quality Score

After hard gates pass, a read-only review agent should assign a crate quality
score out of 100. The crate is complete only if:

```text
hard gates pass
quality score >= 90
```

A score is invalid if hard gates do not pass.

Score caps:

| Condition | Maximum score |
|---|---:|
| hard gate failure | 89 |
| unresolved blocking finding | 79 |
| unresolved high finding | 84 |
| `source_undocumented_behavior` remains | 84 |
| `test_expectation_drift` remains | 79 |
| required verification failure | 74 |
| unapproved semantic behavior change | 69 |
| unapproved soundness-boundary change | 59 |
| implementation-derived spec/test expectation change | 49 |

Scoring rubric:

| Category | Points |
|---|---:|
| Specification completeness | 20 |
| Test contract and coverage | 20 |
| Traceability | 15 |
| Implementation correctness | 15 |
| Design/source synchronization | 10 |
| Boundary discipline | 10 |
| Verification health | 5 |
| Handoff quality | 5 |
| Total | 100 |

## Crate Exit Report

At the end of crate-wide autonomous development, produce a Crate Exit Report.

The report must include:

- status: complete / conditionally complete / incomplete
- quality score
- score caps applied
- milestone scope
- included items
- excluded items
- hard gate status
- score breakdown
- remaining deferred items
- human review surface
- test expectation summary
- verification commands and results
- next-task handoff

### Crate Exit Report Template

```md
# Crate Exit Report: <crate>

## Result

Status:
Quality score:
Score caps applied:

## Scope

Milestone scope:
Included:
Excluded:

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency |  |  |
| Test contract |  |  |
| Traceability |  |  |
| Design/source sync |  |  |
| Boundary discipline |  |  |
| Verification |  |  |
| Residual risk |  |  |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | /20 |
| Test contract and coverage | /20 |
| Traceability | /15 |
| Implementation correctness | /15 |
| Design/source synchronization | /10 |
| Boundary discipline | /10 |
| Verification health | /5 |
| Handoff quality | /5 |
| Total | /100 |

## Deferred Items

| ID | Reason | Owner | Unblock condition |
|---|---|---|---|

## Human Review Surface

The human reviewer should primarily inspect:

- doc/spec/en/...
- tests/**/*.miz
- summarized expectation changes, if any

## Test Expectation Summary

| Test | Intent | Expected outcome | Expected phase | Diagnostics | Spec refs |
|---|---|---|---|---|---|

## Verification

Commands run:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -p mizar-test -- plan --tests-root tests --manifest tests/coverage/spec_trace.toml
```

Results:

## Handoff

Next recommended work:
Known constraints:
Open questions:
```

## PR Type Guidance

Keep these changes separate when practical:

```text
Crate Plan PR:
  crate inventory, readiness, task split, exit criteria
  no implementation

Spec/Test PR:
  doc/spec and .miz tests
  human review target

Implementation PR:
  doc/design, crates, .expect.toml
  agent review and CI target
  includes Crate Exit Report
```

This is guidance, not a requirement to create unnecessary PRs.
