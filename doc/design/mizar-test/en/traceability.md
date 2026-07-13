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
Task 78 historically credited only the imported-structure reserve-head
external-gap boundary for the documented `R` fixture before task 83 superseded
that portion. Task 97 supersedes the documented `TypeCaseStruct` portion.
Broader imported structures outside the task-83 `R` and task-97
`TypeCaseStruct` bridges are deferred; future active cases should observe
`type_elaboration.external_dependency.ast_payload_extraction` and must not
credit real imported structure provenance, structure type-head payload
extraction, base-shape or constructor-witness evidence, positive structure
elaboration, CoreIr, ControlFlowIr, VC, or proof payloads.
Tasks 83 and 97 may credit the imported-structure reserve-head provenance
bridge only: the runner observes
`type_elaboration.checker.checker.declaration.deferred.evidence_query` for the
documented `parser.type_fixtures` imported structures `R` and `TypeCaseStruct`
after passing the real imported `SymbolKind::Structure` symbols as checker type
heads. They credit imported structure provenance and type-head payload
extraction, but not imported module AST extraction, base-shape or
constructor-witness evidence, positive structure elaboration, CoreIr,
ControlFlowIr, VC, or proof payloads.
Task 79 may credit only the imported-mode reserve-head external-gap boundary
outside the task-82 `TypeCaseMode` bridge: those cases observe
`type_elaboration.external_dependency.ast_payload_extraction` for the
documented `parser.type_fixtures` imported mode summary and do not credit
imported mode provenance, mode type-head payload extraction, `ModeExpansion`
payloads, positive mode elaboration, CoreIr, ControlFlowIr, VC, or proof
payloads.
Task 82 may credit the imported-mode reserve-head provenance bridge only: the
runner observes
`type_elaboration.checker.checker.type.external.mode_expansion_payload` for the
same documented `parser.type_fixtures` imported mode summary after passing the
real imported `SymbolKind::Mode` symbol as a checker type head. It credits
imported mode provenance and type-head payload extraction, but not imported
module AST extraction, `ModeExpansion` payloads, positive mode elaboration,
CoreIr, ControlFlowIr, VC, or proof payloads.
Task 80 historically credited the imported-attribute reserve external-gap
boundary before the exact task-84 `TypeCaseAttr`, task-85 negative
`empty`/builtin-`set`, task-116 positive `empty`/builtin-`set`, and task-171
negative `empty`/builtin-`object` bridges superseded all active fixtures on that
row. Positive `empty object`, imported attributes on symbol heads, and broader
source shapes remain deferred extraction gaps without active fixture credit;
future tests must stay deferred until real source-derived payload producers
exist and must not credit imported provenance, `AttributeInput` extraction,
evidence, acceptance, or downstream payloads prematurely.
Task 84 may credit the imported-attribute provenance/`AttributeInput` bridge
only: the runner observes
`type_elaboration.checker.checker.declaration.deferred.evidence_query` for the
documented `parser.type_fixtures` imported attribute `TypeCaseAttr` after
passing the real imported `SymbolKind::Attribute` symbol as a checker
`AttributeInput` on builtin `set`. It credits imported attribute provenance and
no-argument `AttributeInput` payload extraction, but not imported module AST
extraction, attributed-type existential/evidence payloads, positive imported
attributed type elaboration, generic imported attributes such as `empty`,
structure-qualified attribute owner provenance, attribute arguments, CoreIr,
ControlFlowIr, VC, or proof payloads.
Task 85 may credit the imported negative `empty` attribute provenance/
`AttributeInput` bridge only: the runner observes
`type_elaboration.checker.checker.declaration.deferred.evidence_query` for the
existing `non empty set` fixture after passing the real imported
`SymbolKind::Attribute` symbol `empty` as a negative checker `AttributeInput`
over builtin `set`. It credits imported attribute provenance and argument-free
negative `AttributeInput` payload extraction for that fixture only, but not
imported module AST extraction, attributed-type existential/evidence payloads,
broader imported attributes,
structure-qualified attribute owner provenance, attribute arguments, CoreIr,
ControlFlowIr, VC, or proof payloads. Task 116 may credit the matching positive
`empty`/builtin-`set` provenance/`AttributeInput` bridge for the existing
`empty set` fixture and the same evidence-query diagnostic. Task 171 may credit
the exact negative `empty`/builtin-`object` provenance/`AttributeInput` bridge
for the existing `non empty object` fixture and the same evidence-query
diagnostic. Neither task credits attribute admissibility/evidence, positive
`empty object`, imported attributes on symbol heads, or accepted attributed
types.
Task 81 may credit only the argument-bearing local-attribute extraction-gap
boundary: the runner observes
`type_elaboration.external_dependency.ast_payload_extraction` for a
same-module parameterized attribute declared with `param_prefix` syntax and
used through `attribute_name(args)`, and does not credit real term-argument
provenance, checker `AttributeInput` argument payloads, attributed-type
evidence, positive attributed type elaboration, CoreIr, ControlFlowIr, VC, or
proof payloads.
The declaration-symbol runner may separately credit the resolver suffix-primary
projection for that declaration, including imported-lexicon visibility under
the suffix, but this is not checker argument payload extraction.
The supported reserve slices above, excluding task 67, task 68, task 69, task
70, task 71 external-gap boundary cases, the task 75/task 76/task 77
lower-stage boundary cases, the task 78 imported-structure external-gap case,
the task 79 imported-mode external-gap case, the task 80 imported-attribute
external-gap case outside task 84, task 85, and task 116, the task 81
argument-bearing local-attribute external-gap case, the task 86 formula-only theorem
external-gap case, the task 106 builtin equality theorem checker-payload numeric-type gap
case, the task 110 imported predicate/functor theorem checker bridge, the task 108 builtin membership theorem checker bridge, the task 107
builtin inequality theorem checker bridge, the task 109 builtin type-assertion theorem checker
bridge, the task 103 imported attribute assertion theorem formula external-gap
case outside the exact task 113 bridge, the task 113 imported attribute assertion theorem checker bridge, the task 114 exact attribute-level non-empty imported attribute assertion theorem checker bridge
case, the task 111 exact set-enumeration theorem checker bridge
case, the task 112 connective/quantifier formula shell checker bridge
case, the task 88 proof-block theorem external-gap case, the task 89
statement-proof external-gap case, the task 90 predicate/functor definition
external-gap case, the task 91 attribute definition external-gap case, the task
92 mode/structure definition external-gap case, the task 93 proof-local
declaration external-gap case, the task 94 proof-local inline definition
external-gap case, the task 95 registration-block external-gap case, and the
task 96 redefinition/notation external-gap case, but including the task 85
imported negative `empty`/builtin-`set` provenance slice and the task 116
imported positive `empty`/builtin-`set` provenance slice, are converted into a
syntax-free checker source reserve payload, then the
checker-owned seam builds the module `BindingEnv`, one
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
unsupported non-builtin declarations beyond task 96's redefinition/notation
extraction-gap boundary, task 95's registration block extraction-gap boundary, task 94's proof-local inline definition boundary,
task 93's proof-local declaration boundary, and task 92's mode/structure
definition boundary, imported symbols, attribute or mode/structure arguments,
structure-qualified attribute provenance, type-argument or term-argument
provenance, unresolved or ambiguous symbols, proof-local declaration payloads,
inline definition payloads, registration payloads, activation/correctness
payloads, redefinition/notation payloads, notation alias relation payloads,
target inference payloads, terms, formulas, coercions, overload payloads, facts, CoreIr,
ControlFlowIr, VC payloads, or proof payload extraction. Supported checker-owned fail slices
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
diagnostics, or CoreIr/ControlFlowIr/VC/proof promotion. Broader
imported-structure cases outside the task-83 `R` and task-97 `TypeCaseStruct`
bridges are deferred;
future fail cases should credit only the imported structure extraction-gap
boundary and must not credit real imported structure provenance or structure
evidence. Task 83/task 97 fail cases credit real imported structure provenance
and checker type-head payload extraction only; they do not credit imported
module AST extraction, base-shape or constructor-witness evidence, positive
imported structure elaboration, or downstream payload promotion. Task 79 fail cases
credit only the imported mode extraction-gap boundary outside the task-82
`TypeCaseMode` bridge. Task 82 fail cases credit real imported mode provenance
and checker type-head payload extraction only; they do not credit imported
module AST extraction, imported mode expansion, arity checking, positive
imported mode elaboration, or downstream payload promotion. Task 80 fail cases
outside the task-84 `TypeCaseAttr` bridge, task-85 negative
`empty`/builtin-`set` bridge, and task-116 positive `empty`/builtin-`set`
bridge credit only the imported attribute extraction-gap
boundary and do not credit real imported attribute provenance or
attributed-type evidence. Task 84 fail cases credit real imported attribute
provenance and checker `AttributeInput` payload extraction only; they do not
credit imported module AST extraction, attributed-type existential/evidence
payloads, positive imported attributed type elaboration, generic imported
attributes, qualified owner provenance, attribute arguments, or downstream
payload promotion. Task 85 fail cases credit real imported negative `empty`
provenance and checker `AttributeInput` payload extraction only for builtin
`set`; task 116 credits the matching positive `empty`/builtin-`set` payload;
task 171 credits the exact negative `empty`/builtin-`object` payload. Positive
`empty object`, imported attributes on symbol heads, broader imported
attributes, imported module AST extraction, attributed-type evidence, owner
provenance, attribute arguments, and downstream payloads remain extraction/
deferred gaps.
Task 81 fail
cases credit only the argument-bearing local-attribute extraction-gap boundary
and do not credit real term-argument provenance, checker `AttributeInput`
argument payloads, attributed-type evidence, or positive attributed type
elaboration. Task 86 fail cases credit only the formula-only theorem
extraction-gap boundary after parser/resolver execution and do not credit
checker theorem/formula payload extraction, recorded facts, theorem acceptance,
proof skeletons, `formula_statement` runner support, CoreIr, ControlFlowIr,
VC, or proof payloads. Task 87 originally credited only the term-bearing theorem
formula extraction-gap boundary after parser/resolver execution. Task 106
supersedes the exact `TermFormulaPayloadBoundary: 1 = 1` sidecar by crediting
real checker term/formula payload extraction while still failing closed before
numeric type payloads, equality checking, recorded facts, theorem acceptance,
proof skeletons, `formula_statement` runner support, CoreIr, ControlFlowIr, VC,
or proof payloads. Task 98 originally credited only the imported
predicate/functor theorem formula extraction-gap boundary after parser/resolver
execution. Task 110 supersedes the exact
`ImportedPredicateFunctorPayloadBoundary: 1 divides (1 ++ 2)` sidecar by
crediting real checker numeral, imported functor-application, and
predicate-application payload extraction, but still does not credit imported
module AST extraction, semantic predicate/functor signatures, term inference,
formula checking, recorded facts, theorem acceptance, proof skeletons,
`formula_statement` runner support, CoreIr, ControlFlowIr, VC, or proof
payloads. Task 100 originally credited only the builtin membership theorem
formula extraction-gap boundary after parser/resolver execution. Task 108
supersedes the exact `BuiltinMembershipPayloadBoundary: 1 in 1` sidecar by
crediting real checker term/formula payload extraction, but still does not
credit numeric type payloads, membership operand expected-type
construction/checking, recorded facts, theorem acceptance, `formula_statement`
runner support, CoreIr, ControlFlowIr, VC, or proof payloads. Task 101 originally credited only the builtin inequality theorem
formula extraction-gap boundary after parser/resolver execution. Task 107
supersedes the exact `BuiltinInequalityPayloadBoundary: 1 <> 2` sidecar by
crediting real checker term/formula payload extraction while still failing
closed before numeric type payloads, inequality desugaring or equality semantic
checking, recorded facts, theorem acceptance, `formula_statement` runner
support, CoreIr, ControlFlowIr, VC, or proof payloads. Task 119 adds a separate
exact pass row for
`reserve x for set; theorem ReservedVariableEqualityPayloadBoundary: x = x;`:
both identifier terms resolve through the real reserve `BindingEnv`, their
result and equality-expected types derive from the written builtin `set`
reserve, and checker type/well-formedness completes without diagnostics or
facts. It does not credit implicit universal closure, equality truth, theorem
acceptance, `formula_statement`, proof, CoreIr, ControlFlowIr, or VC. Task 123
adds the distinct-binding exact pass row
`reserve x, y for set; theorem DistinctReservedVariableEqualityPayloadBoundary: x = y;`:
the real multi-reserve producer preserves two binding identities and one shared
written builtin `set` type range, while independent source-order lookups and
operand-specific result/expected roles reach a fact-free `Checked` equality.
It does not credit implicit closure/order, equality truth/facts, theorem
acceptance, proof, CoreIr, ControlFlowIr, or VC. Task 124 adds a separate exact
pass row
`reserve x for set; reserve y for set; theorem MultipleReserveDeclarationEqualityPayloadBoundary: x = y;`.
That row preserves two declaration-specific written type ranges across four
pre-normalization result/expected inputs, while the checker deterministically
interns their identical builtin `set` semantics to one normalized type. It
credits only exact multiple-declaration type/well-formedness and does not credit
implicit closure/order, equality truth/facts, theorem acceptance, proof,
CoreIr, ControlFlowIr, or VC. Task 125 adds the heterogeneous exact pass row
`reserve x for object; reserve y for set; theorem HeterogeneousReserveMembershipPayloadBoundary: x in y;`.
The left result retains `object`; the right result and only expected input retain
`set`. The checker records two normalized identities and shares the `set`
identity across both right roles. This credits exact heterogeneous membership
type/well-formedness only, not membership truth/facts, object/set coercion,
implicit closure/order, theorem acceptance, proof, CoreIr, ControlFlowIr, or VC.
Task 126 adds the exact direct-local-mode pass row
`definition mode LocalModeFormulaDef: LocalModeFormula is set; end; reserve x for LocalModeFormula; theorem LocalModeReservedVariableEqualityPayloadBoundary: x = x;`.
All four raw result/expected inputs retain the written local-mode symbol and
reserve range. The checker consumes the real AST-derived bare-set mode
expansion, anchors the single normalized builtin-`set` identity at that
expansion RHS, and records two `Inferred` variables plus one fact-free
`Checked` equality. This credits only the exact direct local-mode
type/well-formedness handoff, not mode-definition declaration checking or
acceptance, inhabitation evidence, implicit closure/order, equality
truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr, or VC.
Task 127 adds the exact one-edge local-mode-chain pass row
`definition mode BaseModeFormulaDef: BaseModeFormula is set; end; definition mode ChainModeFormulaDef: ChainModeFormula is BaseModeFormula; end; reserve x for ChainModeFormula; theorem ChainedLocalModeReservedVariableEqualityPayloadBoundary: x = x;`.
All four raw result/expected inputs retain the written outer-mode symbol and
reserve range. The checker consumes both real AST-derived expansion links,
anchors the single normalized builtin-`set` identity at the terminal `set` RHS,
and records two `Inferred` variables plus one fact-free `Checked` equality. This
credits only the exact one-edge-chain type/well-formedness handoff, not mode-
definition declaration checking/acceptance, inhabitation evidence, object
terminals, longer-chain formulas, closure/order, equality truth/facts, theorem
acceptance, proof, CoreIr, ControlFlowIr, or VC.
Task 128 adds the exact direct local-object-mode pass row
`definition mode LocalObjectModeDef: LocalObjectMode is object; end; reserve x for LocalObjectMode; theorem LocalObjectModeReservedVariableEqualityPayloadBoundary: x = x;`.
All four raw result/expected inputs retain the written object-mode symbol and
reserve range. The checker consumes the real AST-derived expansion, anchors
the single normalized builtin-`object` identity at the real `object` RHS, and
records two `Inferred` variables plus one fact-free `Checked` equality. This
credits only the exact direct local-object-mode type/well-formedness handoff,
not mode-definition declaration checking/acceptance, inhabitation evidence,
broader object-mode formulas, closure/order, equality truth/facts, theorem
acceptance, proof, CoreIr, ControlFlowIr, or VC.
Task 129 adds the exact one-edge local-object-mode-chain pass row
`definition mode BaseObjectModeDef: BaseObjectMode is object; end; definition mode ChainObjectModeDef: ChainObjectMode is BaseObjectMode; end; reserve z for ChainObjectMode; theorem ChainedLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;`.
Four raw outer-mode roles survive while both real expansions normalize to one
builtin-object identity anchored at the terminal RHS. This credits exact
type/well-formedness only, not declaration acceptance/inhabitation, longer
chains, closure/order, truth/facts, theorem acceptance, proof, CoreIr,
ControlFlowIr, or VC.
Task 130 adds the exact direct local-mode inequality pass row
`definition mode LocalModeInequalityDef: LocalModeInequality is set; end; reserve x for LocalModeInequality; theorem LocalModeReservedVariableInequalityPayloadBoundary: x <> x;`.
Four raw roles normalize through one real RHS expansion to one builtin-set
identity; a fact-free pre-desugaring `Checked` inequality credits only exact
type/well-formedness, not declaration acceptance, truth, proof, Core, or VC.
Task 131 adds the exact direct local-object-mode inequality pass row
`definition mode LocalObjectModeInequalityDef: LocalObjectModeInequality is object; end; reserve x for LocalObjectModeInequality; theorem LocalObjectModeReservedVariableInequalityPayloadBoundary: x <> x;`.
Four raw object-mode roles normalize through one real RHS expansion to one
builtin-object identity; a fact-free pre-desugaring `Checked` inequality credits
only exact type/well-formedness, not mode declaration acceptance/inhabitation,
truth, proof, Core, or VC.
Task 132 adds the exact one-edge local-mode-chain inequality pass row
`definition mode BaseModeInequalityDef: BaseModeInequality is set; end; definition mode ChainModeInequalityDef: ChainModeInequality is BaseModeInequality; end; reserve x for ChainModeInequality; theorem ChainedLocalModeReservedVariableInequalityPayloadBoundary: x <> x;`.
Four raw outer-mode roles normalize through both real expansion links to one
terminal-RHS builtin-set identity; a fact-free pre-desugaring `Checked`
inequality credits only exact type/well-formedness, not declaration
acceptance/inhabitation, desugaring, truth, proof, Core, or VC.
Task 133 adds the exact one-edge local-object-mode-chain inequality pass row
`definition mode BaseObjectModeInequalityDef: BaseObjectModeInequality is object; end; definition mode ChainObjectModeInequalityDef: ChainObjectModeInequality is BaseObjectModeInequality; end; reserve z for ChainObjectModeInequality; theorem ChainedLocalObjectModeReservedVariableInequalityPayloadBoundary: z <> z;`.
Four raw outer-mode roles normalize through both real expansion links to one
terminal-RHS builtin-object identity; a fact-free pre-desugaring `Checked`
inequality credits only exact type/well-formedness, not declaration
acceptance/inhabitation, desugaring, truth, proof, Core, or VC.
Task 134 adds the exact two-edge local-mode-chain equality pass row
`definition mode BaseTwoEdgeModeEqualityDef: BaseTwoEdgeModeEquality is set; end; definition mode MiddleTwoEdgeModeEqualityDef: MiddleTwoEdgeModeEquality is BaseTwoEdgeModeEquality; end; definition mode OuterTwoEdgeModeEqualityDef: OuterTwoEdgeModeEquality is MiddleTwoEdgeModeEquality; end; reserve z for OuterTwoEdgeModeEquality; theorem TwoEdgeLocalModeReservedVariableEqualityPayloadBoundary: z = z;`.
Four raw outer-mode roles normalize through all three real expansion links to
one terminal-RHS builtin-set identity; a fact-free `Checked` equality credits
only exact type/well-formedness, not declaration acceptance/inhabitation,
implicit closure/order, truth, proof, Core, or VC.
Task 135 adds the exact two-edge local-object-mode-chain equality pass row
`definition mode BaseTwoEdgeObjectModeEqualityDef: BaseTwoEdgeObjectModeEquality is object; end; definition mode MiddleTwoEdgeObjectModeEqualityDef: MiddleTwoEdgeObjectModeEquality is BaseTwoEdgeObjectModeEquality; end; definition mode OuterTwoEdgeObjectModeEqualityDef: OuterTwoEdgeObjectModeEquality is MiddleTwoEdgeObjectModeEquality; end; reserve z for OuterTwoEdgeObjectModeEquality; theorem TwoEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;`.
Four raw outer-mode roles normalize through all three real expansion links to
one terminal-RHS builtin-object identity; a fact-free `Checked` equality
credits only exact type/well-formedness, not declaration
acceptance/inhabitation, implicit closure/order, truth, proof, Core, or VC.
Task 136 adds the exact two-edge local-mode-chain inequality pass row
`definition mode BaseTwoEdgeModeInequalityDef: BaseTwoEdgeModeInequality is set; end; definition mode MiddleTwoEdgeModeInequalityDef: MiddleTwoEdgeModeInequality is BaseTwoEdgeModeInequality; end; definition mode OuterTwoEdgeModeInequalityDef: OuterTwoEdgeModeInequality is MiddleTwoEdgeModeInequality; end; reserve z for OuterTwoEdgeModeInequality; theorem TwoEdgeLocalModeReservedVariableInequalityPayloadBoundary: z <> z;`.
Four raw outer-mode roles normalize through all three real expansion links to
one terminal-RHS builtin-set identity; a fact-free pre-desugaring `Checked`
inequality credits only exact type/well-formedness, not mode declaration
acceptance/inhabitation, inequality desugaring, implicit closure/order, truth,
proof, Core, or VC.
Task 137 adds the exact two-edge local-object-mode-chain inequality pass row
`definition mode BaseTwoEdgeObjectModeInequalityDef: BaseTwoEdgeObjectModeInequality is object; end; definition mode MiddleTwoEdgeObjectModeInequalityDef: MiddleTwoEdgeObjectModeInequality is BaseTwoEdgeObjectModeInequality; end; definition mode OuterTwoEdgeObjectModeInequalityDef: OuterTwoEdgeObjectModeInequality is MiddleTwoEdgeObjectModeInequality; end; reserve z for OuterTwoEdgeObjectModeInequality; theorem TwoEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary: z <> z;`.
Four raw outer-mode roles normalize through all three real expansion links to
one terminal-RHS builtin-object identity; a fact-free pre-desugaring `Checked`
inequality credits only exact type/well-formedness, not declaration
acceptance/inhabitation, inequality desugaring, implicit closure/order, truth,
proof, Core, or VC.
Task 138 adds the exact direct local-mode reserved-variable type-assertion pass
row
`definition mode LocalModeTypeAssertionDef: LocalModeTypeAssertion is set; end; reserve x for LocalModeTypeAssertion; theorem LocalModeReservedVariableTypeAssertionPayloadBoundary: x is set;`.
The raw subject retains its written local-mode provenance while the asserted
builtin `set` retains its independent formula source; one real expansion
normalizes both to one terminal-RHS builtin-set identity. One `Inferred` term
and one fact-free `Checked` type assertion credit only exact normalized-
reflexive type/well-formedness, not mode declaration acceptance/inhabitation,
formula-side local-mode asserted heads, general reachability/widening/`qua`,
truth, proof, Core, or VC.
Task 139 adds the exact direct local-mode left reserved-variable membership
pass row
`definition mode LocalModeMembershipDef: LocalModeMembership is set; end; reserve x for LocalModeMembership; reserve y for set; theorem LocalModeReservedVariableMembershipPayloadBoundary: x in y;`.
The raw left result retains its written local-mode provenance, while the right
result and sole expected-set input retain the independent explicit reserve
provenance. One real expansion normalizes the left role, the right builtin-set
roles normalize directly, and all three intern to one terminal-RHS builtin-set
identity. Two `Inferred` terms and one fact-free `Checked` membership credit
only exact type/well-formedness, not mode declaration acceptance/inhabitation,
membership truth/facts, implicit closure/order, theorem acceptance, proof,
Core, or VC.
Task 140 adds the exact direct local-object-mode left reserved-variable
membership pass row
`definition mode LocalObjectModeMembershipDef: LocalObjectModeMembership is object; end; reserve x for LocalObjectModeMembership; reserve y for set; theorem LocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`.
The raw left result retains its written local object-mode provenance, while the
right result and sole expected-set input retain the independent explicit
reserve provenance. One real expansion normalizes the left to a terminal-RHS
builtin-object identity, while the right roles normalize directly to a
distinct explicit-reserve-anchored builtin-set identity. Two `Inferred` terms
and one fact-free `Checked` membership credit only exact
type/well-formedness, not mode declaration acceptance/inhabitation, membership
truth/facts, object/set coercion, implicit closure/order, theorem acceptance,
proof, Core, or VC.
Task 141 adds the exact one-edge local-mode-chain left reserved-variable
membership pass row
`definition mode BaseModeMembershipDef: BaseModeMembership is set; end; definition mode ChainModeMembershipDef: ChainModeMembership is BaseModeMembership; end; reserve x for ChainModeMembership; reserve y for set; theorem ChainedLocalModeReservedVariableMembershipPayloadBoundary: x in y;`.
The raw left result retains its written outer-mode provenance, while the right
result and sole expected-set input retain independent explicit reserve
provenance. Both real expansions recursively normalize the left to the
terminal set RHS; the right roles normalize directly, and all three intern to
one terminal-RHS builtin-set identity. Two `Inferred` terms and one fact-free
`Checked` membership credit only exact type/well-formedness, not mode
declaration acceptance/inhabitation, membership truth/facts, implicit
closure/order, theorem acceptance, proof, Core, or VC.
Task 142 adds the exact one-edge local-object-mode-chain left reserved-variable
membership pass row
`definition mode BaseObjectModeMembershipDef: BaseObjectModeMembership is object; end; definition mode ChainObjectModeMembershipDef: ChainObjectModeMembership is BaseObjectModeMembership; end; reserve x for ChainObjectModeMembership; reserve y for set; theorem ChainedLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`.
The raw left result retains its written outer-mode provenance, while the right
result and sole expected-set input retain independent explicit reserve
provenance. Both real expansions recursively normalize the left to the
terminal object RHS; the right roles normalize directly to one distinct
explicit-reserve-anchored builtin-set identity. Two `Inferred` terms and one
fact-free `Checked` membership credit only exact type/well-formedness, not mode
declaration acceptance/inhabitation, membership truth/facts, object/set
coercion, implicit closure/order, theorem acceptance, proof, Core, or VC.
Task 143 adds the exact two-edge local-mode-chain left reserved-variable
membership pass row
`definition mode BaseTwoEdgeModeMembershipDef: BaseTwoEdgeModeMembership is set; end; definition mode MiddleTwoEdgeModeMembershipDef: MiddleTwoEdgeModeMembership is BaseTwoEdgeModeMembership; end; definition mode OuterTwoEdgeModeMembershipDef: OuterTwoEdgeModeMembership is MiddleTwoEdgeModeMembership; end; reserve x for OuterTwoEdgeModeMembership; reserve y for set; theorem TwoEdgeLocalModeReservedVariableMembershipPayloadBoundary: x in y;`.
The raw left result retains its written outer-mode provenance, while the right
result and sole expected-set input retain independent explicit reserve
provenance. All three real expansions recursively normalize the left to the
terminal set RHS; the right roles normalize directly, and all three intern to
one terminal-RHS builtin-set identity. Two `Inferred` terms and one fact-free
`Checked` membership credit only exact type/well-formedness, not mode
declaration acceptance/inhabitation, membership truth/facts, implicit
closure/order, theorem acceptance, proof, Core, or VC.
Task 144 adds the exact two-edge local-object-mode-chain left reserved-variable
membership pass row
`definition mode BaseTwoEdgeObjectModeMembershipDef: BaseTwoEdgeObjectModeMembership is object; end; definition mode MiddleTwoEdgeObjectModeMembershipDef: MiddleTwoEdgeObjectModeMembership is BaseTwoEdgeObjectModeMembership; end; definition mode OuterTwoEdgeObjectModeMembershipDef: OuterTwoEdgeObjectModeMembership is MiddleTwoEdgeObjectModeMembership; end; reserve x for OuterTwoEdgeObjectModeMembership; reserve y for set; theorem TwoEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`.
The raw left result retains its written outer-mode provenance, while the right
result and sole expected-set input retain independent explicit reserve
provenance. All three real expansions recursively normalize the left to the
terminal object RHS; the right roles normalize directly to a distinct
explicit-reserve-anchored builtin-set identity. Two `Inferred` terms and one
fact-free `Checked` membership credit only exact type/well-formedness, not mode
declaration acceptance/inhabitation, membership truth/facts, object/set
coercion, implicit closure/order, theorem acceptance, proof, Core, or VC.
Task 145 adds the exact direct local-object-mode reserved-variable type
assertion pass row
`definition mode LocalObjectModeTypeAssertionDef: LocalObjectModeTypeAssertion is object; end; reserve x for LocalObjectModeTypeAssertion; theorem LocalObjectModeReservedVariableTypeAssertionPayloadBoundary: x is object;`.
The raw subject result retains its written local-mode provenance, while the
asserted builtin `object` retains its independently formula-anchored source
node. The one real expansion normalizes both inputs to one terminal-RHS
builtin-object identity before one `Inferred` term and one fact-free `Checked`
type assertion credit exact normalized-reflexive type/well-formedness only.
Mode declaration acceptance/inhabitation, formula-side local-mode asserted
heads, general reachability/widening/`qua`, object/set coercion, truth/facts,
closure/order, theorem acceptance, proof, Core, and VC remain uncredited.
Task 146 adds the exact one-edge local-mode-chain reserved-variable type
assertion pass row
`definition mode BaseModeTypeAssertionDef: BaseModeTypeAssertion is set; end; definition mode ChainModeTypeAssertionDef: ChainModeTypeAssertion is BaseModeTypeAssertion; end; reserve x for ChainModeTypeAssertion; theorem ChainedLocalModeReservedVariableTypeAssertionPayloadBoundary: x is set;`.
The raw subject result retains its written outer-mode provenance, while the
asserted builtin `set` retains its independently formula-anchored source node.
Both real expansions recursively normalize both inputs to one terminal-RHS
builtin-set identity before one `Inferred` term and one fact-free `Checked`
type assertion credit exact normalized-reflexive type/well-formedness only.
Mode declaration acceptance/inhabitation, formula-side local-mode asserted
heads, general reachability/widening/`qua`, truth/facts, closure/order, theorem
acceptance, proof, Core, and VC remain uncredited.
Task 147 adds the exact one-edge local-object-mode-chain reserved-variable
type assertion pass row
`definition mode BaseObjectModeTypeAssertionDef: BaseObjectModeTypeAssertion is object; end; definition mode ChainObjectModeTypeAssertionDef: ChainObjectModeTypeAssertion is BaseObjectModeTypeAssertion; end; reserve x for ChainObjectModeTypeAssertion; theorem ChainedLocalObjectModeReservedVariableTypeAssertionPayloadBoundary: x is object;`.
The raw subject result retains its written outer-mode provenance, while the
asserted builtin `object` retains its independently formula-anchored source
node. Both real expansions recursively normalize both inputs to one
terminal-RHS builtin-object identity before one `Inferred` term and one fact-
free `Checked` type assertion credit exact normalized-reflexive type/well-
formedness only. Mode declaration acceptance/inhabitation, formula-side local-
mode asserted heads, general reachability/widening/`qua`, object/set coercion,
truth/facts, closure/order, theorem acceptance, proof, Core, and VC remain
uncredited.
Task 148 adds the exact two-edge local-mode-chain reserved-variable type
assertion pass row
`definition mode BaseTwoEdgeModeTypeAssertionDef: BaseTwoEdgeModeTypeAssertion is set; end; definition mode MiddleTwoEdgeModeTypeAssertionDef: MiddleTwoEdgeModeTypeAssertion is BaseTwoEdgeModeTypeAssertion; end; definition mode OuterTwoEdgeModeTypeAssertionDef: OuterTwoEdgeModeTypeAssertion is MiddleTwoEdgeModeTypeAssertion; end; reserve x for OuterTwoEdgeModeTypeAssertion; theorem TwoEdgeLocalModeReservedVariableTypeAssertionPayloadBoundary: x is set;`.
The raw subject result retains its written outer-mode provenance, while the
asserted builtin `set` retains its independently formula-anchored source node.
All three real expansions recursively normalize both inputs to one
terminal-RHS builtin-set identity before one `Inferred` term and one fact-free
`Checked` type assertion are recorded for exact normalized-reflexive type/well-
formedness only. Mode declaration acceptance/inhabitation, formula-side local-
mode asserted heads, general reachability/widening/`qua`, truth/facts,
closure/order, theorem acceptance, proof, Core, and VC remain uncredited.
Task 149 adds the following exact two-edge local-object-mode-chain reserved-
variable type assertion pass row:
`definition mode BaseTwoEdgeObjectModeTypeAssertionDef: BaseTwoEdgeObjectModeTypeAssertion is object; end; definition mode MiddleTwoEdgeObjectModeTypeAssertionDef: MiddleTwoEdgeObjectModeTypeAssertion is BaseTwoEdgeObjectModeTypeAssertion; end; definition mode OuterTwoEdgeObjectModeTypeAssertionDef: OuterTwoEdgeObjectModeTypeAssertion is MiddleTwoEdgeObjectModeTypeAssertion; end; reserve x for OuterTwoEdgeObjectModeTypeAssertion; theorem TwoEdgeLocalObjectModeReservedVariableTypeAssertionPayloadBoundary: x is object;`.
The raw subject result retains its written outer-mode provenance,
while the asserted builtin `object` retains its independently formula-anchored
source node. All three real expansions must recursively normalize both inputs
to one terminal-RHS builtin-object identity before one `Inferred` term and one
fact-free `Checked` type assertion credit exact normalized-reflexive type/well-
formedness only. Mode declaration acceptance/inhabitation, formula-side local-
mode asserted heads, general reachability/widening/`qua`, object/set coercion,
truth/facts, closure/order, theorem acceptance, proof, Core, and VC remain
uncredited. Exact source guards, independent definition/three-link corruption,
and the real frontend/resolver sidecar protect the active row.
Task 150 adds the following exact three-edge local-mode-chain reserved-
variable type assertion pass row:
`definition mode BaseThreeEdgeModeTypeAssertionDef: BaseThreeEdgeModeTypeAssertion is set; end; definition mode InnerThreeEdgeModeTypeAssertionDef: InnerThreeEdgeModeTypeAssertion is BaseThreeEdgeModeTypeAssertion; end; definition mode MiddleThreeEdgeModeTypeAssertionDef: MiddleThreeEdgeModeTypeAssertion is InnerThreeEdgeModeTypeAssertion; end; definition mode OuterThreeEdgeModeTypeAssertionDef: OuterThreeEdgeModeTypeAssertion is MiddleThreeEdgeModeTypeAssertion; end; reserve x for OuterThreeEdgeModeTypeAssertion; theorem ThreeEdgeLocalModeReservedVariableTypeAssertionPayloadBoundary: x is set;`.
The raw subject result retains its written outer-mode provenance, while the
asserted builtin `set` retains its independently formula-anchored source node.
All four real expansions must recursively normalize both inputs to one
terminal-RHS builtin-set identity before one `Inferred` term and one fact-free
`Checked` type assertion credit exact normalized-reflexive type/well-formedness
only. Mode declaration acceptance/inhabitation, formula-side local-mode
asserted heads, general reachability/widening/`qua`, truth/facts, closure/order,
theorem acceptance, proof, Core, and VC remain uncredited. Exact source guards,
independent definition/four-link corruption, and the real frontend/resolver
sidecar protect the active row.
Task 151 adds the following exact three-edge local-object-mode-chain reserved-
variable type assertion pass row:
`definition mode BaseThreeEdgeObjectModeTypeAssertionDef: BaseThreeEdgeObjectModeTypeAssertion is object; end; definition mode InnerThreeEdgeObjectModeTypeAssertionDef: InnerThreeEdgeObjectModeTypeAssertion is BaseThreeEdgeObjectModeTypeAssertion; end; definition mode MiddleThreeEdgeObjectModeTypeAssertionDef: MiddleThreeEdgeObjectModeTypeAssertion is InnerThreeEdgeObjectModeTypeAssertion; end; definition mode OuterThreeEdgeObjectModeTypeAssertionDef: OuterThreeEdgeObjectModeTypeAssertion is MiddleThreeEdgeObjectModeTypeAssertion; end; reserve x for OuterThreeEdgeObjectModeTypeAssertion; theorem ThreeEdgeLocalObjectModeReservedVariableTypeAssertionPayloadBoundary: x is object;`.
The raw subject result retains its written outer-mode provenance, while the
asserted builtin `object` retains its independently formula-anchored source
node. All four real expansions must recursively normalize both inputs to one
terminal-RHS builtin-object identity before one `Inferred` term and one fact-
free `Checked` type assertion credit exact normalized-reflexive type/well-
formedness only. Mode declaration acceptance/inhabitation, formula-side local-
mode asserted heads, general reachability/widening/`qua`, object/set coercion,
truth/facts, closure/order, theorem acceptance, proof, Core, and VC remain
uncredited. Exact source guards, independent definition/four-link corruption,
and the real frontend/resolver sidecar protect the active row.
Task 152 adds the following exact four-edge local-mode-chain reserved-variable
type assertion pass row:
`definition mode BaseFourEdgeModeTypeAssertionDef: BaseFourEdgeModeTypeAssertion is set; end; definition mode InnerFourEdgeModeTypeAssertionDef: InnerFourEdgeModeTypeAssertion is BaseFourEdgeModeTypeAssertion; end; definition mode MiddleFourEdgeModeTypeAssertionDef: MiddleFourEdgeModeTypeAssertion is InnerFourEdgeModeTypeAssertion; end; definition mode OuterFourEdgeModeTypeAssertionDef: OuterFourEdgeModeTypeAssertion is MiddleFourEdgeModeTypeAssertion; end; definition mode TooDeepFourEdgeModeTypeAssertionDef: TooDeepFourEdgeModeTypeAssertion is OuterFourEdgeModeTypeAssertion; end; reserve x for TooDeepFourEdgeModeTypeAssertion; theorem FourEdgeLocalModeReservedVariableTypeAssertionPayloadBoundary: x is set;`.
The raw subject result must retain its written outermost-mode provenance, while
the asserted builtin `set` retains its independently formula-anchored source
node. All five real expansions must recursively normalize both inputs to one
terminal-RHS builtin-set identity before one `Inferred` term and one fact-free
`Checked` type assertion credit exact normalized-reflexive type/well-formedness
only. Mode declaration acceptance/inhabitation, formula-side local-mode
asserted heads, general reachability/widening/`qua`, truth/facts, closure/order,
theorem acceptance, proof, Core, and VC remain uncredited. Exact source guards,
independent definition/five-link corruption, and the real frontend/resolver
sidecar protect the active row.
Task 153 adds the following exact four-edge local-object-mode-chain reserved-
variable type assertion pass row:
`definition mode BaseFourEdgeObjectModeTypeAssertionDef: BaseFourEdgeObjectModeTypeAssertion is object; end; definition mode InnerFourEdgeObjectModeTypeAssertionDef: InnerFourEdgeObjectModeTypeAssertion is BaseFourEdgeObjectModeTypeAssertion; end; definition mode MiddleFourEdgeObjectModeTypeAssertionDef: MiddleFourEdgeObjectModeTypeAssertion is InnerFourEdgeObjectModeTypeAssertion; end; definition mode OuterFourEdgeObjectModeTypeAssertionDef: OuterFourEdgeObjectModeTypeAssertion is MiddleFourEdgeObjectModeTypeAssertion; end; definition mode TooDeepFourEdgeObjectModeTypeAssertionDef: TooDeepFourEdgeObjectModeTypeAssertion is OuterFourEdgeObjectModeTypeAssertion; end; reserve x for TooDeepFourEdgeObjectModeTypeAssertion; theorem FourEdgeLocalObjectModeReservedVariableTypeAssertionPayloadBoundary: x is object;`.
The raw subject result must retain its written outermost-mode provenance, while
the asserted builtin `object` retains its independently formula-anchored source
node. All five real expansions must recursively normalize both inputs to one
terminal-RHS builtin-object identity before one `Inferred` term and one fact-
free `Checked` type assertion credit exact normalized-reflexive type/well-
formedness only. Mode declaration acceptance/inhabitation, formula-side local-
mode asserted heads, general reachability/widening/`qua`, object/set coercion,
truth/facts, closure/order, theorem acceptance, proof, Core, and VC remain
uncredited. Exact source guards, independent definition/five-link corruption,
and the real frontend/resolver sidecar protect the active row.
Task 154 adds the following test-first exact three-edge local-mode-chain
reserved-variable equality pass row:
`definition mode BaseThreeEdgeModeEqualityDef: BaseThreeEdgeModeEquality is set; end; definition mode InnerThreeEdgeModeEqualityDef: InnerThreeEdgeModeEquality is BaseThreeEdgeModeEquality; end; definition mode MiddleThreeEdgeModeEqualityDef: MiddleThreeEdgeModeEquality is InnerThreeEdgeModeEquality; end; definition mode OuterThreeEdgeModeEqualityDef: OuterThreeEdgeModeEquality is MiddleThreeEdgeModeEquality; end; reserve z for OuterThreeEdgeModeEquality; theorem ThreeEdgeLocalModeReservedVariableEqualityPayloadBoundary: z = z;`.
Four raw result/expected inputs retain the written outer-mode provenance; both
operands resolve to `BindingId(0)` at ordinals 1 and 2, and all four real
expansions normalize every role to one terminal-RHS builtin-set identity before
two `Inferred` terms and one fact/deferred-free `Checked` equality credit exact
type/well-formedness only. Mode declaration acceptance/inhabitation, equality
truth/facts, closure/order, theorem acceptance, proof, Core, and VC remain
uncredited. The trace row, production route, independent corruption matrix,
and real frontend/resolver sidecar now protect the active row.
Task 155 adds the following test-first exact three-edge local-object-mode-chain
reserved-variable equality pass row:
`definition mode BaseThreeEdgeObjectModeEqualityDef: BaseThreeEdgeObjectModeEquality is object; end; definition mode InnerThreeEdgeObjectModeEqualityDef: InnerThreeEdgeObjectModeEquality is BaseThreeEdgeObjectModeEquality; end; definition mode MiddleThreeEdgeObjectModeEqualityDef: MiddleThreeEdgeObjectModeEquality is InnerThreeEdgeObjectModeEquality; end; definition mode OuterThreeEdgeObjectModeEqualityDef: OuterThreeEdgeObjectModeEquality is MiddleThreeEdgeObjectModeEquality; end; reserve z for OuterThreeEdgeObjectModeEquality; theorem ThreeEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;`.
Four raw result/expected inputs retain the written outer-mode provenance; both
operands resolve to `BindingId(0)` at ordinals 1 and 2, and all four real
expansions normalize every role to one terminal-RHS builtin-object identity
before two `Inferred` terms and one fact/deferred-free `Checked` equality
credit exact type/well-formedness only. Mode declaration acceptance/
inhabitation, object/set coercion, equality truth/facts, closure/order, theorem
acceptance, proof, Core, and VC remain uncredited. The trace row, production
route, independent corruption matrix, and real frontend/resolver sidecar now
protect the active row.
Task 156 adds the following test-first exact three-edge local-mode-chain
reserved-variable inequality pass row:
`definition mode BaseThreeEdgeModeInequalityDef: BaseThreeEdgeModeInequality is set; end; definition mode InnerThreeEdgeModeInequalityDef: InnerThreeEdgeModeInequality is BaseThreeEdgeModeInequality; end; definition mode MiddleThreeEdgeModeInequalityDef: MiddleThreeEdgeModeInequality is InnerThreeEdgeModeInequality; end; definition mode OuterThreeEdgeModeInequalityDef: OuterThreeEdgeModeInequality is MiddleThreeEdgeModeInequality; end; reserve z for OuterThreeEdgeModeInequality; theorem ThreeEdgeLocalModeReservedVariableInequalityPayloadBoundary: z <> z;`.
Four raw result/expected inputs must retain the written outer-mode provenance;
both operands must resolve to `BindingId(0)` at ordinals 1 and 2, and all four
real expansions must normalize every role to one terminal-RHS builtin-set
identity before two `Inferred` terms and one fact/deferred-free pre-desugaring
`Checked` inequality credit exact type/well-formedness only. Mode declaration
acceptance/inhabitation, inequality desugaring, truth/facts, closure/order,
theorem acceptance, proof, Core, and VC remain uncredited. The trace row,
production route, independent corruption matrix, and real frontend/resolver
sidecar now protect the active row.
Task 157 adds the following exact three-edge local-object-mode-chain
reserved-variable inequality pass row:
`definition mode BaseThreeEdgeObjectModeInequalityDef: BaseThreeEdgeObjectModeInequality is object; end; definition mode InnerThreeEdgeObjectModeInequalityDef: InnerThreeEdgeObjectModeInequality is BaseThreeEdgeObjectModeInequality; end; definition mode MiddleThreeEdgeObjectModeInequalityDef: MiddleThreeEdgeObjectModeInequality is InnerThreeEdgeObjectModeInequality; end; definition mode OuterThreeEdgeObjectModeInequalityDef: OuterThreeEdgeObjectModeInequality is MiddleThreeEdgeObjectModeInequality; end; reserve z for OuterThreeEdgeObjectModeInequality; theorem ThreeEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary: z <> z;`.
Four raw result/expected inputs must retain the written outer-mode provenance;
both operands must resolve to `BindingId(0)` at ordinals 1 and 2, and all four
real expansions must normalize every role to one terminal-RHS builtin-object
identity before two `Inferred` terms and one fact/deferred-free pre-desugaring
`Checked` inequality credit exact type/well-formedness only. Mode declaration
acceptance/inhabitation, object/set coercion, inequality desugaring, truth/
facts, closure/order, theorem acceptance, proof, Core, and VC remain uncredited.
The trace row, production route, independent corruption matrix, and real
frontend/resolver sidecar now protect the active row.
Task 158 adds the following exact active three-edge local-mode-chain left
reserved-variable membership pass row:
`definition mode BaseThreeEdgeModeMembershipDef: BaseThreeEdgeModeMembership is set; end; definition mode InnerThreeEdgeModeMembershipDef: InnerThreeEdgeModeMembership is BaseThreeEdgeModeMembership; end; definition mode MiddleThreeEdgeModeMembershipDef: MiddleThreeEdgeModeMembership is InnerThreeEdgeModeMembership; end; definition mode OuterThreeEdgeModeMembershipDef: OuterThreeEdgeModeMembership is MiddleThreeEdgeModeMembership; end; reserve x for OuterThreeEdgeModeMembership; reserve y for set; theorem ThreeEdgeLocalModeReservedVariableMembershipPayloadBoundary: x in y;`.
The raw left result must retain the written outer-mode provenance; the right
result and sole expected-set input retain independent explicit reserve
provenance, with no left expected type. The operands resolve to
`BindingId(0/1)` at ordinals 2/3; four real expansions then normalize the three
roles to one terminal-RHS builtin-set identity before two `Inferred` terms, one
fact/deferred-free `Checked` membership, and exactly one right-owned constraint
credit exact type/well-formedness only.
Mode declaration acceptance/inhabitation, membership truth/facts, closure/order,
theorem acceptance, proof, Core, VC, object-terminal behavior, and broader chain
depths remain uncredited. The trace row, production route, independent
corruption matrix, and real frontend/resolver sidecar now protect the active row.
Task 159 adds an active row for exactly
`reserve x, y for set; theorem DistinctReservedVariableMembershipPayloadBoundary: x in y;`.
It credits Chapters 4, 13, 14.5.3, and 16 only: one reserve item creates
distinct bindings over one shared written set range, the two identifier uses
resolve at ordinals 2/3, and the left result plus right result/sole expected-set
input must preserve that range before interning to one shared-source-anchored
builtin-set identity. The intended checker result is two `Inferred` variables
and one fact/deferred-free `Checked` membership with exactly one right-owned
constraint and no left expected type. Production routing, corruption/near-miss
coverage, and a real frontend/resolver sidecar now protect the active row.
Chapter 3, truth/facts, closure/order, theorem acceptance,
proof/Core/VC, separate declarations, and broader source shapes remain
uncredited.
Task 160 adds an active row for exactly
`reserve x, y for set; theorem DistinctReservedVariableInequalityPayloadBoundary: x <> y;`.
It credits Chapters 4, 13, 14.5.2, and 16 only: one reserve item creates
distinct bindings over one shared written set range, the two identifier uses
resolve at ordinals 2/3, and both result/expected role pairs must preserve that
range before interning to one shared-source-anchored builtin-set identity. The
intended checker result is two `Inferred` variables and one fact/deferred-free
pre-desugaring `Checked` inequality with two ordered operand-owned constraints.
Production routing, corruption/near-miss coverage, and the real frontend/
resolver sidecar now protect the active row. Chapter 3,
desugaring/truth/facts, closure/order, theorem acceptance, proof/Core/VC,
separate declarations, and broader source shapes remain uncredited.
Task 161 adds an active row for exactly `reserve x for set; reserve y
for set; theorem MultipleReserveDeclarationInequalityPayloadBoundary: x <> y;`.
It credits Chapters 4, 13, 14.5.2, and 16 only: two reserve items create
distinct bindings and written ranges, uses resolve at ordinals 2/3, and both
result/expected role pairs retain the corresponding range before interning to
one canonical builtin-set identity anchored at the earlier `x` range. The
intended result is two `Inferred` variables and one fact/deferred-free pre-
desugaring `Checked` inequality with two ordered constraints. Production
routing, corruption/near-miss coverage, and the real sidecar now protect the
active row. Chapter 3, shared-range behavior, desugaring/truth/
facts, closure/order, theorem acceptance, proof/Core/VC, and broader shapes
remain uncredited.
Task 162 adds an active row for exactly `reserve x for set; reserve y for
set; theorem MultipleReserveDeclarationMembershipPayloadBoundary: x in y;`.
It references Chapters 4, 13, 14.5.3, and 16 only: two reserve items create
distinct bindings and written ranges; uses resolve at ordinals 2/3; the left
result retains the first range while the right result and sole right expected
input retain the second, and no left expected input exists. The intended result
is one earlier-x-anchored canonical builtin-set identity, two `Inferred`
variables, and one fact/deferred-free `Checked` membership with exactly one
right-owned constraint. Production routing, corruption/near-miss coverage, and
the real sidecar now protect the active row, so active credit contains 113
cases. Chapter 3, shared-range behavior, membership truth/facts, closure/order,
theorem acceptance, proof/Core/VC, and broader shapes remain uncredited.
Task 163 adds an active row for exactly the four-definition object-terminal
chain ending in `object`, `reserve x for OuterThreeEdgeObjectModeMembership;`,
`reserve y for set;`, and
`ThreeEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`.
It references Chapters 3, 4, 7, 13, 14.5.3, and 16 through the existing Chapter
4/13/14/16 requirement rows plus the three-edge-chain and dedicated checker
rows. Intended credit is limited to four real expansions, raw left and explicit-
set right provenance, `BindingId(0/1)` at ordinals 2/3, distinct object/set
identities, no left expected input, two inferred variables, and one fact-free
checked membership with exactly one right-owned constraint. Production routing,
corruption/near-miss coverage, a real sidecar, and active count 114 now guard it;
coercion, truth/facts, closure/order, theorem/proof/Core/VC, other depths, and
broader shapes remain uncredited.
Task 164 adds an active row for exactly the five-definition set-terminal
chain ending in `set`, `reserve x for TooDeepFourEdgeModeMembership;`,
`reserve y for set;`, and
`FourEdgeLocalModeReservedVariableMembershipPayloadBoundary: x in y;`. It
references Chapters 4, 7, 13, 14.5.3, and 16 through the existing Chapter
4/13/14/16 requirement rows plus the structural-chain and dedicated checker
rows. Intended credit is limited to five real expansions, raw left and
explicit-set right provenance, `BindingId(0/1)` at ordinals 2/3, one terminal-
set-RHS identity, no left expected input, two inferred variables, and one
fact-free checked membership with exactly one right-owned constraint. Six trace
backlinks, production routing, corruption/near-miss coverage, and a real
sidecar now guard active count 115. Truth/facts, closure/order,
theorem/proof/Core/VC, object-terminal behavior, other depths, and broader
shapes remain uncredited.
Task 165 adds an active row for exactly the five-definition object-terminal
chain ending in `object`,
`reserve x for TooDeepFourEdgeObjectModeMembership;`, `reserve y for set;`, and
`FourEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`.
It references Chapters 3, 4, 7, 13, 14.5.3, and 16 through the existing Chapter
4/13/14/16 requirement rows plus the structural-chain and dedicated checker
rows. Intended credit is limited to five real expansions, raw left and
explicit-set right provenance, `BindingId(0/1)` at ordinals 2/3, distinct
terminal-object-RHS and explicit-set identities, no left expected input, two
inferred variables, and one fact-free checked membership with exactly one
right-owned constraint. Six trace backlinks, production routing, corruption/
near-miss coverage, and a real sidecar now guard active count 116. Truth/facts,
object/set coercion, closure/order,
theorem/proof/Core/VC, other depths, and broader shapes remain uncredited.
Task 166 adds an active row for exactly the five-definition set-terminal
chain ending in `set`, `reserve z for TooDeepFourEdgeModeEquality;`, and
`FourEdgeLocalModeReservedVariableEqualityPayloadBoundary: z = z;`. It
references Chapters 4, 7, 13, 14.5.2, and 16 through the existing Chapter
4/13/14/16 requirement rows plus the structural-chain and dedicated checker
rows. Intended credit is limited to five real expansions, four raw outermost-
mode result/expected inputs, `BindingId(0)` at ordinals 1/2, one terminal-set-
RHS identity, two inferred variables, one fact/deferred-free checked equality,
and two ordered operand-owned expected constraints. Six trace backlinks,
production routing, corruption/near-miss coverage, and a real sidecar now
protect active count 117. Declaration acceptance/
inhabitation, truth/facts, closure/order, theorem/proof/Core/VC, object-terminal
behavior, other depths, and broader shapes receive no credit.
Task 167 adds an active row for exactly the five-definition object-terminal
chain ending in `object`,
`reserve z for TooDeepFourEdgeObjectModeEquality;`, and
`FourEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;`. It
references Chapters 3, 4, 7, 13, 14.5.2, and 16 through the existing Chapter
4/13/14/16 requirement rows plus the structural-chain and dedicated checker
rows. Intended credit is limited to five real expansions, four raw outermost-
mode result/expected inputs, `BindingId(0)` at ordinals 1/2, one terminal-
object-RHS identity, two inferred variables, one fact/deferred-free checked
equality, and two ordered operand-owned expected constraints without object/
set coercion. Six trace backlinks, production routing, corruption/near-miss
coverage, and a real sidecar now protect active count 118. Declaration
acceptance/inhabitation, truth/facts, closure/
order, theorem/proof/Core/VC, set-terminal behavior, other depths, and broader
shapes receive no credit.
Task 168 adds an active row for exactly the five-definition set-terminal chain
ending in `set`, `reserve z for TooDeepFourEdgeModeInequality;`, and
`FourEdgeLocalModeReservedVariableInequalityPayloadBoundary: z <> z;`. It
references Chapters 4, 7, 13, 14.5.2, and 16 through the existing Chapter
4/13/14/16 requirement rows plus the structural-chain and dedicated checker
rows. Intended credit is limited to five real expansions, four raw outermost-
mode result/expected inputs, `BindingId(0)` at ordinals 1/2, one terminal-set-
RHS identity, two inferred variables, one fact/deferred-free pre-desugaring
checked inequality, and two ordered operand-owned expected constraints. Six
trace backlinks, production routing, corruption/near-miss coverage, and a real
sidecar now protect active count 119. Declaration acceptance/
inhabitation, inequality desugaring/truth/facts, closure/order, theorem/proof/
Core/VC, object-terminal behavior, other depths, and broader shapes receive no
credit.
Task 169 adds an active row for exactly the five-definition object-terminal
chain ending in `object`,
`reserve z for TooDeepFourEdgeObjectModeInequality;`, and
`FourEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary: z <> z;`.
It references Chapters 3, 4, 7, 13, 14.5.2, and 16 through the existing Chapter
4/13/14/16 requirement rows plus the structural-chain and dedicated checker
rows. Intended credit is limited to five real expansions, four raw outermost-
mode result/expected inputs, `BindingId(0)` at ordinals 1/2, one terminal-
object-RHS identity, two inferred variables, one fact/deferred-free pre-
desugaring checked inequality, and two ordered operand-owned expected
constraints without object/set coercion. Six trace backlinks, production
routing, corruption/near-miss coverage, and a real sidecar now protect active
count 120. Declaration acceptance/inhabitation, inequality desugaring/
truth/facts, closure/order, theorem/proof/Core/VC, set-terminal behavior, other
depths, and broader shapes receive no credit.
Task 172 adds an active row for exactly the seven-definition set-terminal chain
ending in `BaseMode -> set`, `reserve z for ChainMode6;`, and
`LongLocalModeReservedVariableEqualityPayloadBoundary: z = z;`. It references
Chapters 4, 7, 13, 14.5.2, and 16 through the existing Chapter 4/13/14/16
requirement rows plus the structural-chain and dedicated checker rows. Intended
credit is limited to seven real AST-derived expansions, four raw `ChainMode6`
result/expected inputs, `BindingId(0)` at ordinals 1/2, one terminal-
`BaseMode`-RHS builtin-set identity, two inferred variables, one fact/deferred-
free checked equality, and two ordered operand-owned expected constraints.
Exact routing, corruption/near-miss coverage, and a real frontend/resolver
sidecar now protect active count 121. Declaration acceptance/inhabitation,
truth/facts, closure/order, theorem/proof/Core/ControlFlow/VC, imported/attributed/argument-
bearing or other chain shapes, and general unbounded semantics receive no
credit.
Task 173 adds the exact sibling active row ending in
`LongLocalModeReservedVariableInequalityPayloadBoundary: z <> z;`, with the
same Chapter 4/7/13/14.5.2/16 and structural-chain links plus a dedicated
checker row. Intended credit is seven real expansions, four raw `ChainMode6`
roles, ordinal 1/2 `BindingId(0)`, one terminal-`BaseMode`-RHS identity, two
inferred variables, two ordered constraints, and one fact/deferred-free pre-
desugaring checked inequality. Six backlinks, full guards, and a real sidecar
now protect active count 122; desugaring/truth/facts and broader semantics receive no credit.
Task 174 adds the exact test-first sibling row ending in
`LongLocalModeReservedVariableMembershipPayloadBoundary: x in y;`, with
Chapter 4/7/13/14.5.3/16 and structural-chain links plus a dedicated checker
row. Intended credit is seven real expansions, a raw `ChainMode6` left result,
independent explicit-set right result and sole right expected input, ordinal
2/3 `BindingId(0/1)`, one terminal-`BaseMode`-RHS identity, no left expected
input, two inferred variables, one right-owned constraint, and one fact/
deferred-free checked membership. Six backlinks, production routing, full
guards, and the real sidecar now protect active count 123. Membership truth/
facts and broader semantics receive no credit.
Task 175 adds the exact test-first sibling row ending in
`LongLocalModeReservedVariableTypeAssertionPayloadBoundary: x is set;`, with
Chapter 3/4/7/13/14.2.3/16 and structural-chain links plus a dedicated checker
row. Intended credit is seven real expansions, a raw `ChainMode6` subject
result, independent formula-side builtin-set asserted input, ordinal 1
`BindingId(0)`, one terminal-`BaseMode`-RHS identity, one inferred variable,
and one fact/deferred-free normalized-reflexive checked type assertion without
general reachability. Seven backlinks are present; production routing, full
guards, and the real sidecar now protect active count 124. Widening/`qua`,
truth/facts, and broader semantics receive no credit.
Task 176 adds the exact test-first sibling row ending in
`LongLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;`, with
Chapter 4/13/14.5.2/16 and structural-chain links plus a dedicated checker row.
Intended credit is seven real expansions, four raw `ChainObjectMode6` result/
expected inputs, ordinal 1/2 `BindingId(0)`, one terminal-`BaseObjectMode`-RHS
identity, two inferred terms, two ordered operand-owned constraints, and one
fact/deferred-free checked equality without object/set coercion. Six backlinks
are present; production routing, full guards, and the real sidecar now protect
active count 125. Truth/facts and broader semantics receive no credit.
Task 177 adds the matching exact test-first sibling row ending in
`LongLocalObjectModeReservedVariableInequalityPayloadBoundary: z <> z;`, with
Chapter 4/13/14.5.2/16 and structural-chain links plus a dedicated checker row.
Intended credit is seven real expansions, four raw `ChainObjectMode6` result/
expected inputs, ordinal 1/2 `BindingId(0)`, one terminal-`BaseObjectMode`-RHS
identity, two inferred terms, two ordered operand-owned constraints, and one
fact/deferred-free pre-desugaring checked inequality without object/set
coercion. Six backlinks, production routing, full guards, and the real sidecar
now protect active count 126. Desugaring, truth/facts, and broader semantics
receive no credit.
Task 178 adds the matching exact active sibling row ending in
`LongLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`, with
Chapter 4/13/14.5.3/16 and structural-chain links plus a dedicated checker row.
Intended credit is seven real expansions, a raw `ChainObjectMode6` left result,
an independent explicit-set right result/sole expected input, ordinal 2/3
`BindingId(0/1)`, distinct terminal-object-RHS and explicit-set identities, no
left expected input, two inferred terms, one right-owned constraint, and one
fact/deferred-free checked membership without object/set coercion. Six
backlinks, production routing, full guards, and the real sidecar protect active
count 127.
Truth/facts and broader semantics receive no credit.
Task 120 adds the matching exact pass row for
`reserve x for set; theorem ReservedVariableMembershipPayloadBoundary: x in x;`:
both identifier results and the right membership expected type derive from the
written `set` reserve, and a no-fact `Checked` membership records
type/well-formedness only. Membership truth/facts, implicit closure, theorem
acceptance, proof, CoreIr, ControlFlowIr, and VC remain uncredited. Task 121
adds the exact pass row
`reserve x for set; theorem ReservedVariableInequalityPayloadBoundary: x <> x;`.
The checker-owned inequality API supplies two expected-type slots while task
119 supplies the real reserve binding/use producer, so the runner records two
linked result/expected role pairs and one fact-free pre-desugaring `Checked`
inequality. Task 107's numeral inequality bridge remains partial without
expected types. Inequality desugaring/truth/facts, implicit closure, theorem
acceptance, proof, CoreIr, ControlFlowIr, and VC remain uncredited. Task 122
adds the exact pass row
`reserve x for set; theorem ReservedVariableTypeAssertionPayloadBoundary: x is set;`.
Task 119 supplies the real reserve lookup/result input and task 109 supplies the
independently source-anchored formula asserted-type input. The checker accepts
only normalized reflexive identity and records one fact-free `Checked` type
assertion; known non-identical types remain partial on the external reachability
payload gap. General reachability/widening/`qua`, attributes, truth/facts,
implicit closure, theorem acceptance, proof, CoreIr, ControlFlowIr, and VC
remain uncredited. Task 109
supersedes the exact
builtin type-assertion sidecar from task 102 by crediting real checker
term/formula payload extraction and the asserted builtin `set`
`TypeExpressionInput` while still failing closed before numeric type payloads,
broader asserted type payloads, type-assertion semantic checking, recorded
facts, theorem acceptance, `formula_statement` runner support, CoreIr,
ControlFlowIr, VC, or proof payloads. Task 113 supersedes task 103 for the
exact imported attribute assertion theorem formula by crediting imported
`empty` provenance validation plus real checker term/formula payload handoff
while still failing closed before numeric type payloads, attribute-chain
semantic payload extraction, theorem-formula `AttributeInput` payload
extraction, term inference, attribute admissibility/semantic checking, formula
checking, recorded facts, theorem acceptance, imported module AST extraction,
`formula_statement` runner support, CoreIr, ControlFlowIr, VC, or proof
payloads. Task 114 fail cases supersede task 104 only for the exact
attribute-level non-empty imported attribute assertion theorem formula after
parser/resolver execution: the runner validates the direct `non` surface and
imported `empty` provenance, passes real checker term/formula payloads, and
fails closed before numeric type payloads, negated attribute-chain semantic
payload extraction, theorem-formula `AttributeInput` payload extraction, term
inference, negated attribute admissibility/semantic checking, formula checking,
recorded facts, theorem acceptance, imported module AST extraction,
`formula_statement` runner support, CoreIr, ControlFlowIr, VC, or proof
payloads. Task 111 fail cases credit only the exact
`SetEnumerationPayloadBoundary: {1, 2} = {1, 2}` checker handoff after
parser/resolver execution: real checker payloads for four numeral item terms,
two set-enumeration terms, and one equality formula. They do not credit broader
set-enumeration term extraction, result-type/sethood payloads, term inference,
equality/formula checking, recorded facts, theorem acceptance,
`formula_statement` runner support, CoreIr, ControlFlowIr, VC, or proof
payloads. Task 112 / task 117 fail cases supersede task 99 only for the exact
connective/quantifier theorem formula checker shell handoff after
parser/resolver execution: real checker `FormulaInput` shells for implication,
universal quantification, and negation, plus exact `FormulaKind::Contradiction`
payloads for the two source constants. They do not credit formula constant
semantic truth values, child-formula graph payloads, quantifier binder/context
payloads, formula checking, recorded facts, theorem acceptance,
`formula_statement` runner support, CoreIr, ControlFlowIr, VC, or proof
payloads. Task 88 fail cases
credit only the proof-block/proof-skeleton
extraction-gap boundary after parser/resolver execution and do not credit
checker proof skeleton payload extraction, local proof context, formula payload
extraction, recorded facts, theorem acceptance, `formula_statement` runner
support, CoreIr, ControlFlowIr, VC, or proof payloads. Task 89 fail cases
credit only the statement-level proof-justification extraction-gap boundary
after parser/resolver execution and do not credit checker statement proof
payload extraction, nested proof skeleton payloads, local proof context, formula
payload extraction, label-reference semantic checking, recorded facts, theorem
acceptance, `formula_statement` runner support, CoreIr, ControlFlowIr, VC, or
proof payloads. Task 90 fail cases credit only the predicate/functor definition
extraction-gap boundary after parser/resolver execution and do not credit
checker definition declaration payload extraction, definition-local context,
definiens formula/term payload extraction, overload payloads, recorded facts,
`formula_statement` runner support, CoreIr, ControlFlowIr, VC, or proof
payloads. Task 91 fail cases credit only the attribute definition
extraction-gap boundary after parser/resolver execution and do not credit
checker attribute definition declaration payload extraction, definition-local
context, formula-definiens payload extraction, attributed-type evidence,
recorded facts, `formula_statement` runner support, CoreIr, ControlFlowIr, VC,
or proof payloads. Task 92 fail cases credit only the mode/structure definition
extraction-gap boundary after parser/resolver execution and do not credit
checker mode/structure definition declaration payload extraction, mode
expansion, structure base-shape/constructor/selector evidence,
definition-local context, recorded facts, `formula_statement` runner support,
CoreIr, ControlFlowIr, VC, or proof payloads. Task 93 fail cases credit only
the proof-local declaration statement extraction-gap boundary after
parser/resolver execution and do not credit checker proof-local declaration
payload extraction, local proof context, formula/term payload extraction, RHS
term inference, reconsider coercion/obligation evidence, recorded facts,
theorem acceptance, `formula_statement` runner support, CoreIr, ControlFlowIr,
VC, or proof payloads. Task 94 fail cases credit only the proof-local inline
definition extraction-gap boundary after parser/resolver execution and do not
credit checker inline definition formal/body payload extraction, local
abbreviation expansion, term/formula body payload extraction, guard evidence,
recorded facts, theorem acceptance, `formula_statement` runner support, CoreIr,
ControlFlowIr, VC, or proof payloads. Task 75/76/77 fail
cases credit only the lower-stage active-range boundary for forward local-mode,
local-structure, or local-attribute references and do not credit checker
`ModeExpansion`, structure type-head, base-shape, constructor-witness,
`AttributeInput`, or attributed-type evidence production.
Task 95 fail cases credit only the registration-block extraction-gap boundary
after parser/resolver execution and do not credit checker registration-item
payload extraction, correctness-condition/proof-obligation payloads, accepted
activation/evidence status, cluster/reduction semantics, Chapter 17 semantic
rows, facts, `formula_statement` or `advanced_semantics` runner support,
CoreIr, ControlFlowIr, VC, or proof payloads.
Task 96 fail cases credit only the redefinition/notation extraction-gap
boundary after parser/resolver execution and do not credit checker
redefinition payload extraction, notation alias relation payloads,
redefinition target inference, coherence proof-obligation payloads, overload
candidate payloads, Chapter 11 alias semantic resolution, Chapter 19
overload/redefinition semantics, facts, `formula_statement` or
`advanced_semantics` runner support, CoreIr, ControlFlowIr, VC, or proof
payloads.

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
