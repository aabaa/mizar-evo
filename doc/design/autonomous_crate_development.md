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
```

If the repository later uses a different language/design layout, adapt the path
while preserving the same purpose.

The Crate Plan must include:

1. crate responsibility
2. relevant specification items
3. relevant tests
4. design/source inventory
5. known gaps and drift
6. task decomposition
7. exit criteria

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

## Task Decomposition

### Task <ID>: <title>

Purpose:

Spec refs:

Tests:

Affected design files:

Affected source files:

Completion condition:

Forbidden behavior:

## Exit Criteria

Hard gates:

Verification commands:

Review expectations:
```

Tasks inside a crate should be decomposed by specification requirement or test
obligation, not merely by source module.

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

## Crate Task Entries

Each crate task entry should identify:

- task id
- purpose
- spec refs
- tests
- affected design files
- affected source files
- completion condition
- forbidden behavior

Example:

```md
### Task P-001: Parse theorem statement syntax

Spec refs:
- spec.en.15.statements.theorem_statement

Tests:
- tests/miz/pass/parser/...
- tests/miz/fail/parser/...

Affected files:
- doc/design/mizar-parser/en/...
- crates/mizar-parser/src/...

Completion:
- positive syntax is accepted
- negative syntax is rejected at the correct phase
- no resolver/type/proof behavior is introduced

Forbidden:
- name resolution
- type inference
- theorem validity checking
- proof obligation generation
```

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
7. required verification commands pass, or any unrun command is explicitly
   justified
8. remaining risks are classified as deferred, out of scope, or human-owned

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
