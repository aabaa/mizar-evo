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

## Autonomous Design Decisions And Stop Conditions

Within an explicitly requested autonomous crate-development run, the parent
agent must not stop merely because more than one derived implementation design
is possible. It may choose module ownership, API names and shapes, immutable
data representation, validation order, task sequencing, private test matrices,
and documentation placement without additional user confirmation when all of
the following hold:

- the choice does not change language behavior, test intent, diagnostics,
  parser recovery, soundness boundaries, proof or acceptance policy, or another
  higher-authority requirement;
- the choice stays within the requested crate/task scope and existing
  responsibility boundaries;
- the relevant specification and tests are present and non-contradictory;
- the selected design is the smallest fail-closed implementation that satisfies
  the frozen acceptance criteria; and
- the decision is recorded in the paired task contract or owning design
  document and passes the required independent reviews.

When several derived designs remain valid, prefer the option that adds the
least semantic surface, preserves existing public artifacts, avoids premature
generalization, and defers unowned semantics explicitly. A bounded derived
`design_drift`, `source_drift`, or `test_gap` is work to resolve, not by itself
a reason to request a design decision from the user.

Stop and request user or human-authority input only when continued work would
require at least one of the following:

- resolving a `spec_gap` or contradiction by choosing language or proof
  behavior not fixed by current higher-authority artifacts;
- changing existing `.miz` test intent, expectations, trace status, public
  diagnostics, parser recovery, or a soundness/acceptance boundary;
- overriding a `repo_metadata_conflict` or another protected artifact;
- expanding to a materially different task, external system, permission, or
  destructive action not already authorized; or
- choosing among alternatives whose observable semantic consequences cannot be
  separated or safely deferred by a minimal fail-closed design.

If a later milestone has unowned semantics but a safe derived prerequisite or
documentation-only deferral decision is available, complete that bounded work
and continue fresh inventory instead of stopping at the first design fork.

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
`doc/spec/en/` chapter. If a task has no audit impact, leave the audit unchanged
and write nothing: silence is the no-impact record. Mention it in the final
response only when a reviewer asks.

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

Use one paired task contract for a non-trivial autonomous task — one that
needs a frozen contract, crosses owner documents, or carries exact API,
test, file, diagnostic, count, or hash requirements:

```text
doc/design/task_contracts/en/<task-id>.md
doc/design/task_contracts/ja/<task-id>.md
```

`<task-id>` must match `[A-Za-z0-9][A-Za-z0-9._-]*` and be identical in both
trees. The English contract is canonical and at most 60 lines; the Japanese
companion is a pointer stub (title, canonical link, owner-plan links) created in
the same change. Both files link to the corresponding owning crate plans. If an owner has no
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
- `doc/design/spec_coverage_audit.md` impact when there is one
- required review roles, verification commands, and exit criteria

A contract never carries completion evidence, measured counts, digests, gate
tallies, scores, review outcomes, model names, reasoning settings, or the
next-task handoff. Those go in the commit body (see the `volume:` line in
AGENTS.md) and the final response. The
[Documentation Volume Ledger](#documentation-volume-ledger) lint rejects them.

### Single-owner documentation rule

Each derived fact has one live owner. The task contract owns orchestration and
indexes owner-local details; it does not become a second copy of module design.
The synchronized English/Japanese pair counts as one logical derived owner.
Module documents own durable public/private API, invariants, validation,
ownership, and module-local test design; harness documents own only their
private routes and consumer deltas; traceability documents own manifest relationships,
coverage audits own coverage status and follow-up ownership, boundary audits
own module-layout decisions, bilingual audits own parity evidence, and todo
documents own concise sequencing status. Crate-plan task entries contain only
the ordered task/contract links; the other mandatory crate-level plan sections
continue to own responsibility, specification/test inventory, gaps, audit
expectations, readiness, and exit criteria.

Update only documents whose owned state changes. There is no required fan-out
file count, and repeated boilerplate is not synchronization evidence. Measured
verification counts and hashes live in the commit body, never in live design
documents. If an audit has no impact, leave it unchanged and write nothing.
Contracts link to owner documents; module, harness, audit, and traceability
documents never link back to a contract. A contract is linked only from
`doc/design/todo.md`, crate plans, crate todos, and the coverage audit.

Stable module inventories and public-enum policies remain in their existing
owner documents when required by repository lint. A future task-contract lint
must scan the paired directory recursively before the workflow may claim that
nested task contracts are machine-enforced.

### Migration policy

Apply the contract structure to new tasks. Historical task appendices are
frozen logs and need not be rewritten. An ordinary semantic task does not
rewrite them. Migrate an active or reopened task only as a separate
documentation task: first move every unique contract, API, test-intent,
classification, deferral, traceability, and verification fact to its designated
owner; then replace duplicate blocks with links; finally run an equivalence and
EN/JA review. Git history is not a substitute for a live owner.

An explicitly user-authorized legacy-evidence compaction may migrate multiple
completed tasks in one logical documentation task only when fresh inventory
proves that they are one coherent duplication family. Before deletion, its
paired migration contract freezes:

- every source section and its language-local destination owner
- exact source-section/file/line baselines and the allowed plan/index deltas
- the per-task paired historical contracts that retain shared evidence
- owner-local API, invariant, runner, audit, traceability, coverage, bilingual,
  and sequencing sections that must remain
- protected specification, `.miz`, expectation, trace, source, diagnostic,
  active-result, count, and hash surfaces
- equivalence, EN/JA, local-link/fragment, and hard-gate verification

Replace only mapped shared evidence with links. Current-state plans, TODOs, and
audits remain concise replace-in-place summaries; module documents remain
durable product contracts rather than task diaries. Record final task-local
measurements once in the historical task contract or required exit report.
The batch must be behavior- and coverage-neutral, separately reviewed and
committed, and bounded enough that every removed fact has a live owner. It does
not authorize a wholesale repository history rewrite or let semantic work
absorb unrelated cleanup.

After a whole-section batch is committed, declare it in the versioned
`doc/design/task_contracts/legacy_compactions.tsv` ledger. Its generic lint
consumer validates exact explicit source paths, forbidden raw H2-H6 headings,
language-local completion redirects and fragments, nearest same-or-higher
heading anchors, paired task/batch contracts, owning Task Index rows, declared
counts, and the canonical expanded-inventory hash. Extend the data ledger for
later batches; do not add historical task/file lists to lint-policy Rust.

Ledger schema version 2 represents only replacement of a complete ATX heading
section through the next heading of equal or higher level. One global `task`
row remains the sole historical-contract owner. A later batch may declare a
`task_ref` only to an existing task owned by another batch, and only when every
batch for that task has a disjoint source-file set. The reference participates
in the declaring batch's task count and expanded-inventory hash; redirects
resolve through the canonical task row, and historical Task Index ownership is
not repeated. A second same-task section in one source file, or a section that
mixes shared completion evidence with owner-local API, invariant, runner,
audit, traceability, coverage, bilingual, or sequencing facts, must remain
intact until a separately reviewed schema/ownership prerequisite defines a
safe occurrence or paragraph-level migration. The ledger records and enforces
a migration already authorized by paired contracts; it cannot itself authorize
deletion or change semantic/coverage ownership.

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

## Volume And Scope Review

Every review sequence, full or light tier, includes one independent volume and
scope review after the implementation review. Reviews are otherwise one-way
ratchets: a gap finding adds text or code, and nothing removes it. This review
is the counterweight.

The reviewer receives the task diff, the contract, and the authority
references, and returns only a list of removable items: lines, files, public
types, tests, paragraphs, or pointer edits. An item is removable, and therefore
a blocking finding, when none of `doc/spec/en/`, a `.miz` test, its
expectation, or the frozen contract requires it. Duplicated AST or payload
representations, single-use abstractions, speculative fields, restated
evidence, and documentation fan-out are the expected finding classes. "Nothing
removable" must be stated with a one-line justification. The implementer
removes every confirmed item and repeats the review until it reports none.

Budgets the reviewer checks against the `volume:` commit line:

| Measure | Limit without prior user approval |
|---|---:|
| task-contract length, per language | 60 lines |
| documentation lines added per task, all commits | 200 |
| documentation added versus production added | 2 : 1 |
| production lines added per task | 300 without a presented plan |

## Documentation Volume Ledger

`tests/coverage/doc_volume_baseline.tsv` is a ratchet consumed by
`cargo test` (`lint_policy.rs`). It records, per file or contract, the ceiling
that existing documents already exceed. Rows may only be lowered or removed;
raising a row or adding one for a new file requires explicit user approval
recorded in the commit body. Files without a row are held at the default.

| Kind | Scope | Default |
|---|---|---:|
| `ceremony_tokens` | `doc/design/**/*.md` except `archive/` and this protocol: occurrences of `quality score`, `score cap`, `xhigh`, `<n>/100`, `hard gate(s) n/m`, the words `Luna`/`Sol`/`Terra`, and 64-hex digests | 0 |
| `contract_lines` | each `doc/design/task_contracts/{en,ja}/**/*.md` | 60 |
| `contract_fanout` | files outside `task_contracts/`, `todo.md`, crate plans, crate todos, and `spec_coverage_audit.md` that link a contract | 0 |

## Gate Tiering

Ceremony is tiered by what a task can put at risk (September 2026 audit 2).
The tier is declared in the task contract (or, for light-tier work without a
contract, in the commit message) before implementation starts.

### Full gates (default)

The full ceremony — the nine hard gates below, the independent multi-stage
reviews, frozen-doc prerequisite commits where the task class requires them,
and the ≥90/100 quality score — applies to every task that is, or touches, a
trust-boundary or semantic-credit change. That includes any change to:
language behavior or `doc/spec`; `.miz` tests, expectations, trace status, or
coverage/semantic credit; production Rust behavior, diagnostics, parser
recovery, or public API of a semantic authority; soundness or fail-closed
boundaries; or activation of a previously inactive case. When the tier is
ambiguous, the task is full-gate.

### Light gates (zero-credit structural transport)

A task qualifies for the light tier only when it moves, links, archives,
re-indexes, or re-formats existing text or metadata without changing any
behavior, test intent, expectation, trace status, or semantic/coverage
credit. Examples: documentation compaction batches under
[documentation_compaction_rules.md](./documentation_compaction_rules.md),
archive splits of frozen logs, pointer-stub conversions, mechanical ledger
updates, and link/fragment repairs.

Light-tier requirements (all mandatory):

1. the authority order is untouched: no edit to `doc/spec`, `.miz` sources,
   expectations, `spec_trace` rows/status, or production Rust behavior;
2. every removed or moved fact retains exactly one live owner (single-owner
   rule), with language-local redirects where the rulebook requires them;
3. one independent equivalence review (single pass; re-review only on
   findings) instead of the multi-stage review sequence;
4. required verification commands pass (at minimum `cargo test` for changes
   the doc/ledger lints cover, plus the local link/fragment checks);
5. a task-only commit with the tier named in the contract or commit body.

Not required in the light tier: the nine-gate ceremony, frozen-doc
prerequisite commits, the quality-score evaluation, and per-document review
repetition to "NO FINDINGS".

### One-way promotion rule

Tiering is monotonic toward safety. The moment a task gains semantic credit
or touches a protected surface — however small — it is promoted to full
gates for its entire remaining scope, including work already done under the
light tier, which must be re-reviewed at the full tier. A full-gate task is
never demoted mid-task, and a light-tier declaration never authorizes the
protected edits listed above.

## Crate Exit Gates

A crate-wide autonomous development task (full tier) is complete only when
all hard gates pass:

1. no blocking/high specification inconsistency remains
2. no source behavior remains that is absent from `doc/spec` and tests
3. milestone-owned specification items have existing or test-first `.miz`
   coverage, or explicit deferred reasons
4. test expectations were not changed merely to match current implementation
5. `doc/design` and source are synchronized within the target crate scope
6. crate responsibility boundaries are not violated
7. `doc/design/spec_coverage_audit.md` is updated for any changed spec/design
   coverage, follow-up ownership, or deferred coverage status (unchanged
   coverage leaves the file and every other document untouched)
8. required verification commands pass, or any unrun command is explicitly
   justified
9. remaining risks are classified as deferred, out of scope, or human-owned

In this protocol, "no findings" means no unresolved blocking/high findings. Low
notes may remain if documented. Medium findings must be fixed or explicitly
deferred with a reason.

## Quality Score

After hard gates pass, a read-only review agent should assign a crate quality
score out of 100. The score applies to full-tier work; a light-tier task (see
Gate Tiering) requires its single equivalence review instead of a score. The crate is complete only if:

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
| task contract longer than 60 lines in either language | 84 |
| documentation added exceeds twice production added, or 200 lines per task | 84 |
| evidence, scores, digests, or model routing written under `doc/` | 79 |
| volume and scope review skipped or its findings left unresolved | 79 |
| unapproved semantic behavior change | 69 |
| unapproved soundness-boundary change | 59 |
| implementation-derived spec/test expectation change | 49 |

Scoring rubric:

| Category | Points |
|---|---:|
| Economy: smallest diff, no duplicate representations, budgets met | 20 |
| Test contract and coverage | 20 |
| Specification completeness | 15 |
| Implementation correctness | 15 |
| Traceability | 10 |
| Boundary discipline | 10 |
| Design/source synchronization | 5 |
| Verification health | 5 |
| Total | 100 |

Economy is scored from the `volume:` commit line, the volume and scope review
result, and the public-type delta. A task that adds a parallel term, formula,
or statement representation where an existing one could be referenced scores
at most 5 of 20 in this category.

## Crate Exit Report

At the end of crate-wide autonomous development, produce a Crate Exit Report
in the final response and the pull-request description. It is not stored under
`doc/design/`; the volume ledger rejects its scores and digests there.

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
| Economy | /20 |
| Test contract and coverage | /20 |
| Specification completeness | /15 |
| Implementation correctness | /15 |
| Traceability | /10 |
| Boundary discipline | /10 |
| Design/source synchronization | /5 |
| Verification health | /5 |
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

## Delegation And Model Routing

The parent agent keeps the user-requested reasoning setting and decides
authority interpretation, specification or test intent, soundness boundaries,
public API ownership, semantic acceptance, and final scoring. A lower setting
for a sub-agent never lowers the parent setting or relaxes a gate.

When a session exposes per-agent GPT-5.6 selection, follow the
[current official OpenAI model guidance](https://developers.openai.com/api/docs/guides/latest-model)
and these routes; availability and quota are environment facts, not policy:

- Sol at the parent setting for the decisions above.
- Luna `xhigh` for bounded work after the parent has frozen the contract:
  deterministic inventory, count and hash checks, whole-section compaction,
  manifest and link validation, focused verification, localized
  contract-driven implementation, and first-pass independent reviews. Luna
  must not resolve a `spec_gap`, derive test intent, invent semantics,
  authorize a lower-stage change, expand scope, or accept a disputed finding;
  it escalates without editing beyond the frozen scope.
- Terra `high` (or `xhigh` with measured gain) as the intermediate route for
  cross-module implementation, precision review, or a disagreement above
  Luna's assignment that does not yet need a parent decision.
- Broaden a route only after representative repository trials comparing task
  success, missed findings, repair loops, gate agreement, tokens, latency, and
  cost against one lower setting. If a model is not exposed, use the closest
  eligible route without blocking.

Without per-agent model selection: parent setting for authority conflicts,
specification gaps, soundness, disputed semantics, and final scoring; `high`
for bounded implementation and independent reviews after the contract is
frozen; `medium` or lower only for deterministic inventory and mechanical
checks. Sub-agent output is subject to the same reviews, verification, gates,
and commit rules as any other output. Effective model and setting, when they
matter, are recorded in the final response, never under `doc/`.

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
