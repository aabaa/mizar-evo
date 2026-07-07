# Module: traceability

> Canonical language: English. Japanese companion: [../ja/traceability.md](../ja/traceability.md).

## Purpose

This module defines the traceability manifest that connects `doc/spec/`
requirements to committed tests without adding test links to the specification
text itself.

The specification remains a readable language reference. Test coverage is
tracked by a separate machine-readable manifest owned by `mizar-test`.

## Design Decision

Specification-to-test links live outside `doc/spec/`.

The traceability model is bidirectional:

- the manifest maps spec requirements to the tests that cover them;
- each test expectation sidecar maps the test back to one or more spec
  requirement ids.

The harness validates both directions.

```text
doc/spec/...                    pure specification text
tests/coverage/spec_trace.toml  spec requirement -> tests
*.expect.toml                   test -> spec requirement ids
```

## Manifest Location

The canonical manifest is:

```text
tests/coverage/spec_trace.toml
```

Additional generated reports may be written under `tests/coverage/reports/`,
but those reports are derived artifacts. The manifest is the source of truth.

## Requirement Record

Each requirement record describes a checkable unit of the specification.

```toml
[[requirement]]
id = "spec.en.02.lexical.identifiers.basic"
source = "doc/spec/en/02.lexical_structure.md"
section = "2.6 Identifiers"
stage = "lexical"
status = "planned"
required = true
coverage = "pass_and_fail"
tests = []
```

Fields:

| Field | Meaning |
|---|---|
| `id` | Stable requirement id. Never reused for unrelated semantics. |
| `source` | Specification file that owns the requirement. |
| `section` | Human-readable section heading or section number. |
| `stage` | Staged model stage that first owns executable coverage. |
| `status` | `planned`, `covered`, `partial`, `deferred`, or `obsolete`. |
| `required` | Whether release coverage requires this item. |
| `coverage` | Expected coverage shape. |
| `tests` | Canonical relative paths to expectation sidecars or fixture metadata. |

Optional fields:

| Field | Meaning |
|---|---|
| `anchors` | Stable heading anchors when available. |
| `notes` | Short human review notes. |
| `depends_on` | Other lower-stage requirement ids that must be covered first. |
| `built_in` | `true` when the requirement is supplied by built-ins and may satisfy another requirement's `depends_on` without executable coverage. |
| `deferred_reason` | Required when `status = "deferred"`. |
| `issue` | Tracking issue or design discussion reference. |

## Coverage Shapes

`coverage` records what kind of test evidence is expected.

Allowed values:

| Value | Meaning |
|---|---|
| `none` | No executable test is required. Used for explanatory text. |
| `pass` | At least one accepting test is required. |
| `fail` | At least one rejecting test is required. |
| `pass_and_fail` | Both accepting and rejecting tests are required. |
| `diagnostic` | A stable diagnostic or failure category must be checked. |
| `snapshot` | A deterministic snapshot must be checked. |
| `property` | A property or generated test family covers the item. |
| `manual_review` | Human review is required; executable coverage is not sufficient. |

Multiple shapes may be required by splitting a specification section into
several requirement records.

## Public Enum Forward Compatibility

Task 12 applies the `mizar-frontend` task-25 procedure to traceability enums.
These enums are stored in `spec_trace.toml`, reported by the harness, and
consumed by downstream tooling, so they must remain `#[non_exhaustive]`;
downstream callers must keep wildcard match arms. Crate-internal matches may
stay exhaustive for currently known variants.

| Public enum | Decision |
|---|---|
| `RequirementStatus` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `CoverageShape` | `#[non_exhaustive]` downstream forward-compatible surface. |

No exhaustive public enum exceptions are owned by this module.

## Test Sidecar Reference

Each expectation sidecar records the spec requirements it covers.

```toml
schema_version = 1
id = "pass_lexical_identifier_basic_001"
stage = "lexical"
spec_refs = [
  "spec.en.02.lexical.identifiers.basic",
]
```

Executable sidecar `stage` must match the requirement `stage`, unless the
requirement explicitly allows coverage from a later stage through a satisfied
`depends_on` chain. Earlier-stage executable sidecars cannot credit later-stage
requirements. `manual_review` entries are metadata anchors rather than
executable coverage, so they may record cross-stage handoff notes when any
declared prerequisites are satisfied.

## Validation

The harness validates:

1. Every manifest `source` exists.
2. Every `id` is unique.
3. Every listed test path exists.
4. Every listed test sidecar points back to the requirement id.
5. Every sidecar `spec_refs` entry exists in the manifest.
6. Stage names match the staged model.
7. Required coverage shapes are satisfied when the validation mode requires
   coverage completeness.
8. Deferred required items include a `deferred_reason`.
9. Obsolete items are not referenced by active tests.
10. Manifest records are sorted deterministically by `id`.
11. `depends_on` ids exist, do not point to the same requirement, and point only
    to lower-stage requirements.
12. Linked sidecars do not receive coverage credit when their declared stage
    prerequisites are unsatisfied or their executable stage does not match the
    credited requirement.

Validation must not parse `doc/spec/` prose beyond checking that referenced
files exist. The manifest owns the granularity of requirements.

## Validation Modes

Traceability validation has modes:

| Mode | Purpose | Coverage Completeness |
|---|---|---|
| `metadata` | Minimal crate and local editing. | Not required; planned items without tests are warnings at most. |
| `development` | Normal CI during implementation. | Required only for requirements marked `status = "covered"` or `partial`. |
| `release` | Release readiness gate. | Required for every `required = true` requirement unless `status = "deferred"` with a reason. |

All modes validate manifest syntax, unique ids, source file existence, known
stage ids, known sidecar references, and sidecar back-references.

Only `release` mode turns `required = true` coverage into an error solely
because it is required. `development` mode is stricter than `metadata` for
items already stored as `covered` or `partial`, but still allows planned
coverage map entries before the compiler pipeline exists.

Current mode behavior:

- `metadata` emits stored-status drift as warnings and does not fail missing
  coverage shapes.
- `development` fails missing coverage and status drift for requirements stored
  as `covered` or `partial`.
- `release` fails missing coverage and status drift for every
  `required = true` requirement unless it is `deferred` with a reason.

## Coverage Status

`status` is derived during reporting but stored in the manifest for review
workflow.

Rules:

- `planned` means the requirement is known but lacks sufficient tests.
- `partial` means some required coverage exists but not all coverage shapes are
  satisfied.
- `covered` means all required coverage shapes are satisfied by active tests.
- `deferred` means coverage is intentionally postponed.
- `obsolete` means the requirement no longer applies and active tests must not
  claim it.

The report flags stored status when it disagrees with computed status. The
severity of that disagreement is determined by validation mode.

The implemented coverage report derives evidence from valid bidirectional
links only: the manifest must list the sidecar path, and the parsed sidecar
must point back to the requirement id. Invalid sidecars, one-way links,
unsatisfied prerequisites, and invalid executable stage mismatches do not
receive coverage credit. Coverage evidence is computed as follows:

- `pass`: at least one linked sidecar with `expected_outcome = "pass"`;
- `fail`: at least one linked sidecar with `expected_outcome = "fail"`;
- `pass_and_fail`: both pass and fail evidence are present;
- `diagnostic`: a linked fail sidecar carries diagnostic/failure identity
  metadata;
- `snapshot`: a linked sidecar has snapshot output or snapshot outcome/kind;
- `property`: a linked sidecar has `kind = "property_seed"`;
- `manual_review`: a linked sidecar exists, while the stored status remains
  the human-reviewed status because executable evidence alone is not
  sufficient.

`none` and `manual_review` statuses are not inferred away from the manifest
when linked metadata exists. They remain review workflow states. Missing
linked metadata for `manual_review` computes as `planned`.

## Stage Interaction

Traceability uses the staged model from [staged_model.md](./staged_model.md).

Coverage credit is assigned only when lower-stage prerequisites are already
covered, declared as built-ins, or listed in `depends_on` with acceptable
status.

Task 7 enforces declared stage-prerequisite and `depends_on` credit rules. The
harness does not infer prerequisites from `doc/spec/` prose or source contents:
requirements that need explicit lower-stage credit list it in `depends_on`, and
built-in prerequisites use `built_in = true`.

For example, a parser fixture can cover the syntax of a cluster declaration,
but it does not cover cluster expansion semantics. The semantic requirement
remains planned until advanced semantic tests exist.

For the declaration-symbol stage, coverage is executable only for `.miz`
sidecars admitted by the active runner gate (`active_declaration_symbol`,
`stage = "declaration_symbol"`, `expected_phase = "resolve"`, and pass/fail
outcome). Fail coverage may use `diagnostic_payloads` or `stable_detail_key`
for resolver internal detail keys while public resolver diagnostic codes remain
unspecified; active sidecars must leave `diagnostic_codes` empty until that
range exists.

For the type-elaboration stage, coverage is executable only for `.miz`
sidecars admitted by the active runner gate (`active_type_elaboration`,
`stage = "type_elaboration"`, `expected_phase = "type_check"`, and pass/fail
outcome). The task 16-20 bridge continuation may credit the narrow
reserve-only builtin declaration pass slice: unrecovered top-level reserve
items whose segments contain one or more identifiers and exactly one bare
builtin `set` or `object` type-expression, with no attributes, arguments,
parameter prefixes, or non-builtin symbol heads. Task 55 may additionally
credit the narrow bare local-mode expansion pass slice: the reserve type head
is an un-attributed argument-free same-module local mode, and the runner derives
a real `ModeExpansion` from a unique unrecovered preceding same-module
no-argument `ModeDefinition` whose RHS is bare builtin `set` / `object` and
whose enclosing definition block has no definition-local context. Task 56 may
also credit the narrow one-edge local-mode expansion chain pass slice: the
reserve type head expands to a preceding same-module no-argument local mode
whose own preceding source definition has an accepted task-55 bare builtin RHS
expansion, and the runner inserts both real source-derived expansions before the
checker-owned reserve seam. Task 57 may credit a diagnostic-only fail slice
when the reserve head expands through a real same-module local-mode expansion
whose RHS is a same-module local structure head; the runner passes the real
expansion to the checker-owned seam, but the checker still reports the missing
base-shape/constructor-witness evidence query. Task 58 may credit the parallel
diagnostic-only fail slice when the reserve head expands through a real
same-module local-mode expansion whose RHS is an attributed builtin head; the
runner passes the real expansion to the checker-owned seam, but the checker
still reports the missing attributed-type existential evidence query. Task 60
may credit the direct attributed-root structure-RHS diagnostic-only fail slice:
the reserve head is an attributed argument-free same-module local mode, the
runner derives a real direct local-structure RHS expansion under the inherited
task-57 uniqueness/precedence/no-context constraints, and the checker still
reports the missing base-shape/constructor-witness plus full attributed-type
evidence query. Task 61 may credit the direct attributed-root
attributed-builtin-RHS diagnostic-only fail slice: the reserve head is an
attributed argument-free same-module local mode, the runner derives a real
direct attributed-builtin RHS expansion under the inherited task-58
uniqueness/precedence/no-context constraints, and the checker still reports
the missing full attributed-type existential evidence query. Task 62 may credit
the one-edge bare local-mode structure-RHS chain diagnostic-only fail slice:
the reserve head is an un-attributed argument-free same-module local mode, the
runner derives both real `A -> B` and `B -> LocalStruct` expansions from unique
unrecovered preceding same-module definitions in source order, and the checker
still reports the missing base-shape/constructor-witness evidence query. Task
63 may credit the one-edge bare local-mode attributed-builtin-RHS chain
diagnostic-only fail slice: the reserve head is an un-attributed
argument-free same-module local mode, the runner derives both real `A -> B`
and `B -> marked set` expansions from unique unrecovered preceding same-module
definitions in source order with argument-free same-module RHS attributes, and
the checker still reports the missing attributed-type existential evidence
query. Task 65 may credit the one-edge attributed-root structure-RHS chain
diagnostic-only fail slice: the reserve head is an attributed argument-free
same-module local mode, the runner derives both real `A -> B` and
`B -> LocalStruct` expansions from unique unrecovered preceding same-module
definitions in source order while the root is not mixed with a bare reserve
use and the dependency is not itself attributed, and the checker still reports
the missing base-shape/constructor-witness plus full attributed-type evidence
query. Task 66 may credit the one-edge attributed-root attributed-builtin-RHS
chain diagnostic-only fail slice: the reserve head is an attributed
argument-free same-module local mode, the runner derives both real `A -> B`
and `B -> marked set` expansions from unique unrecovered preceding same-module
definitions in source order with argument-free same-module RHS attributes while
the root is not mixed with a bare reserve use and the dependency is not itself
attributed, and the checker still reports the missing full attributed-type
evidence query. Task 67 may credit the structure-qualified attribute
extraction-gap boundary slice: a same-module structure-qualified attribute
reference is parser/resolver executable, but the runner still asserts
`type_elaboration.external_dependency.ast_payload_extraction` until checker
payloads preserve real qualifier and attribute-owner provenance. Task 68 may
credit the argument-bearing mode reserve extraction-gap boundary slice: a
same-module argument-bearing local mode surface and reserve use such as
`Element of a` are parser/resolver executable, but the runner still asserts
`type_elaboration.external_dependency.ast_payload_extraction` until checker
payloads preserve real type-argument and term-argument provenance. Task 69 may
credit the argument-bearing structure reserve extraction-gap boundary slice: a
same-module structure declaration with an `of` parameter surface and reserve
use such as `LocalStruct of a` are parser/resolver executable, but the runner
still asserts `type_elaboration.external_dependency.ast_payload_extraction`
until checker payloads preserve real type-argument and term-argument
provenance. Task 70 may credit the bracket-form local mode reserve
extraction-gap boundary slice: a source containing a same-module
bracket-parameter mode declaration and reserve use such as `Family[set]`
reaches parser/resolver, but the runner still asserts
`type_elaboration.external_dependency.ast_payload_extraction` before bracket
type-argument payload extraction or mode-head resolution. Task 71 may credit
the bracket-form local structure reserve extraction-gap boundary slice: a
source containing a same-module bracket-parameter structure declaration and
reserve use such as `LocalStruct[set]` reaches parser/resolver, but the runner
still asserts `type_elaboration.external_dependency.ast_payload_extraction`
before bracket type-argument payload extraction or structure-head resolution.
Task 72 may credit the two-edge bare local-mode chain pass slice, Task 73 may
credit the three-edge bare local-mode chain pass slice, and Task 74 may credit
the structural bare local-mode chain pass slice: the runner derives every real
mode expansion from unique unrecovered preceding same-module no-argument
definitions, continues the outer reserve through the existing checker handoff
and readiness path, and uses an AST-bounded structural traversal budget rather
than a semantic chain-length cap. Chains that violate the structural guards
remain on the checker missing mode-expansion diagnostic.
Task 75 may credit only the active-range/no-forward-reference boundary for a
forward local-mode reserve head: the runner observes
`type_elaboration.lower_stage.frontend:malformed_type_expression` before
checker handoff and does not credit checker mode-expansion payload extraction.
Task 76 may credit only the matching active-range/no-forward-reference
boundary for a forward local-structure reserve head: the runner observes the
same lower-stage detail before checker handoff and does not credit checker
structure type-head, base-shape, or constructor-witness payload extraction.
Task 77 may credit only the matching active-range/no-forward-reference
boundary for a forward local-attribute reserve type expression: the runner
observes the same lower-stage detail before checker handoff and does not credit
checker `AttributeInput` payload extraction or attributed-type evidence queries.
The supported reserve slices above, excluding task 67, task 68, task 69, task
70, task 71 external-gap boundary cases, and the task 75/task 76/task 77
lower-stage boundary cases, are converted into a syntax-free checker source reserve
payload, then the checker-owned seam builds the module `BindingEnv`, one
`DeclarationInput` per binding, binding-specific `TypeExpressionInput` sites,
and `DeclarationChecker` output.
The runner continues that handoff into checker-owned `TypedAst` and
`ResolvedTypedAst`, then reads it through `mizar-core`
`ResolvedTypedAstSummary::from_ast` and binder-only `CoreContext` preparation
for readiness only. Active pass tests may cover that slice only when the
listed source has at least one extracted reserve binding and runner regression
evidence confirms checker handoff construction, minimal `TypedAst`,
`ResolvedTypedAst`, summary-readiness, and binder-only core context paths were
exercised. The pass slice must have its own traceability row/test instead of
being credited from the diagnostic external-gap row.

Covered active fail tests may still assert the external-gap detail key
`type_elaboration.external_dependency.ast_payload_extraction` when a case needs
unsupported non-builtin declarations, imported symbols, attribute or
mode/structure arguments, structure-qualified attribute provenance,
type-argument or term-argument provenance, unresolved or ambiguous symbols,
terms, formulas, coercions, overload payloads, facts, CoreIr, ControlFlowIr,
VC payloads, or proof payload extraction. Supported checker-owned fail slices
may instead assert the checker
detail keys for same-module attributed builtin reserve heads missing evidence
or same-module local structure reserve heads missing base-shape evidence,
including attributed local structures that lack full normalized attributed-type
existential evidence, task-57 same-module local-mode expansions with local
structure RHSs missing base-shape evidence, task-58 same-module local-mode
expansions with attributed builtin RHSs missing attributed-type existential
evidence, task-59 attributed local-mode reserve heads with real direct
bare-builtin expansions missing attributed-type existential evidence, task-60
attributed local-mode reserve heads with real direct local-structure RHS
expansions missing base-shape/constructor-witness and full attributed-type
evidence, task-61 attributed local-mode reserve heads with real direct
attributed-builtin RHS expansions missing full attributed-type existential
evidence, task-62 one-edge bare local-mode chains ending in local structure
RHSs missing base-shape/constructor-witness evidence, task-63 one-edge bare
local-mode chains ending in attributed builtin RHSs missing attributed-type
existential evidence, task-64 attributed local-mode reserve heads with one-edge
bare-builtin chains missing attributed-type existential evidence, task-65
attributed local-mode reserve heads with one-edge structure-RHS chains missing
base-shape/constructor-witness and full attributed-type existential evidence,
task-66 attributed local-mode reserve heads with one-edge attributed-builtin-RHS
chains missing full attributed-type existential evidence, or same-module local
mode reserve heads, including mixed attributed/bare
local-mode sources or chains that violate task-74 structural guards, missing
mode-expansion payloads. Task 56's
attributed-chain-dependency fail case is part
of that same missing mode-expansion payload family and does not credit a partial
chain expansion; attributed-RHS chains beyond the task-58/task-61 direct
slices, task 63's bare one-edge chain slice, and task 66's attributed-root
one-edge chain slice remain outside this bridge, and
structure-RHS chains remain outside task 60's direct attributed-root slice, task
62's bare one-edge chain slice, and task 65's attributed-root chain slice.
Task 67 structure-qualified attribute cases are credited only as
extraction-gap boundary coverage, not as real qualified attribute payload
coverage. Task 68 argument-bearing mode cases are credited only as
extraction-gap boundary coverage, not as real mode-argument payload, arity
matching, mode expansion, or positive type-elaboration coverage. Task 69
argument-bearing structure cases are credited only as extraction-gap boundary
coverage, not as real structure-argument payload, arity matching, base-shape
evidence, or positive structure type-elaboration coverage. Task 70 bracket-form
mode cases are credited only as extraction-gap boundary coverage, not as real
bracket type-argument payload, `qua`-argument payload, mode-head resolution,
arity matching, mode expansion, or positive type-elaboration coverage.
Task 71 bracket-form structure cases are credited only as extraction-gap
boundary coverage, not as real bracket type-argument payload,
`qua`-argument payload, structure-head resolution, arity matching, base-shape
or constructor-witness evidence, or positive structure type-elaboration
coverage.
Task 72 pass cases credit only the source-derived two-edge bare local-mode
chain bridge, and task 73 pass cases credit only the corresponding three-edge
bare local-mode chain bridge, and task 74 pass cases credit only the structural bare-chain bridge; unsupported chains do not credit broader mode
expansion, structure/attributed-builtin terminals beyond the existing one-edge
diagnostics, or CoreIr/ControlFlowIr/VC/proof promotion. Task 75/76/77 fail
cases credit only the lower-stage active-range boundary for forward local-mode,
local-structure, or local-attribute references and do not credit checker
`ModeExpansion`, structure type-head, base-shape, constructor-witness,
`AttributeInput`, or attributed-type evidence production.

Those gap tests do not satisfy the broader task 7-11
semantic pass/fail coverage, and `CoreIr`, `ControlFlowIr`, and
`proof_verification` rows remain deferred until prepared consumer execution
exists; the summary/context readiness read is not a CoreIr/ControlFlowIr/VC/
proof promotion.

## Reporting

The default report groups results by:

- spec file;
- stage;
- status;
- missing coverage shape;
- tests with unknown spec refs;
- tests that cover obsolete requirements.

Reports must be deterministic and suitable for CI output.

The current `plan` CLI report prints deterministic totals, per-stage coverage
status counts, missing-shape counts, and the corpus-wide pass/fail mix against
the architecture test-strategy target of 40% pass and 60% fail. The pass/fail
mix counts unique valid sidecars, so a sidecar covering multiple requirements
is not counted multiple times.

Task 14 extends the report with an architecture-22 matrix summary. The summary
uses validated sidecar metadata rather than executing consumer crates. For each
required scenario id from the registry in
[expectation_schema.md](./expectation_schema.md), the report records the
registry equivalence class, planned metadata count, active execution count, and
whether the scenario is missing. The committed task-14 anchor is:

```text
tests/property/architecture22_matrix_001.expect.toml
```

It is a `property_seed` metadata-only sidecar at `stage =
"advanced_semantics"` and `domain = "incremental_verification"`, linked to the
manual-review requirement
`spec.en.architecture_22.regression_matrix.metadata`. This keeps all matrix
rows visible as planned metadata while every row stays inactive until a future
consumer-specific runner or integration test owns real execution.

## Constraints and Assumptions

- `doc/spec/` remains free of per-test links.
- Requirement ids are stable public identifiers for the test corpus.
- The manifest may be edited manually, but validation is automated.
- Generated tests can contribute coverage only through committed expectation
  metadata.
- Coverage is semantic evidence, not line or branch coverage.
