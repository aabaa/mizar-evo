# Module: traceability

## Parser Task 46 Operator-Declaration Trace Completion

`spec.en.10.operator_declarations.parser` is covered by exactly the new pass
and fail sidecars with `pass_and_fail` mode. This credit is limited to lexical/
parser-facing declaration shape, placement, source preservation, and recovery.
Operator activation, active-functor validation, overload meaning, resolution,
precedence-range semantics, and Pratt-metadata mutation remain outside the row.

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
Task 179 adds the matching exact active sibling row ending in
`LongLocalObjectModeReservedVariableTypeAssertionPayloadBoundary: x is object;`,
with Chapter 3/4/13/14.2.3/16 and structural-chain links plus a dedicated checker
row. Intended credit is seven real expansions, a raw `ChainObjectMode6` subject
result, an independent formula-side builtin-object asserted input, ordinal 1
`BindingId(0)`, one terminal-object-RHS identity, one inferred term, and one
fact/deferred-free normalized-reflexive checked type assertion without general
reachability or object/set coercion. Six shared backlinks, the dedicated row,
production routing, full guards, and the real sidecar protect active count 128. Truth/
facts, acceptance, and broader semantics receive no credit.
Task 180 adds the exact standalone pass row
`SourceDerivedContradictionConstantBoundary: contradiction` with Chapter 14/16
and a dedicated checker bridge reference. Intended credit is the real formula-
leaf site/range, module-root context, one checked `FormulaKind::Contradiction`,
and empty term/type/constraint/candidate/fact/deferred/diagnostic payload. The
dedicated row, production route, exact/near-miss/corruption guards, and real
sidecar protect active count 129. Falsehood/fact publication, theorem
acceptance, proof-goal closure, implicit closure/child graphs,
`formula_statement`, proof, CoreIr, ControlFlowIr, and VC receive no credit.
Task 182 adds the exact active formula-side local-mode asserted-head row
`LocalModeAssertedHeadPayloadBoundary: x is LocalModeAssertedHead;`, with
Chapter 3/4/7/13/14.2.3/16 and a dedicated checker bridge reference. Credit is
limited to one real direct set-terminal expansion, independent raw reserve-
subject and formula-side asserted inputs resolving to the same local-mode
symbol, ordinal 1 `BindingId(0)`, three known type entries interned to one
terminal-definition-RHS builtin-set identity, one inferred term, and one fact/
deferred-free normalized-reflexive checked type assertion without general
reachability. Five shared backlinks plus the dedicated row, production routing,
exact/near-miss/corruption guards, and the real frontend/resolver sidecar
protect active count 130. Declaration acceptance/inhabitation, widening/`qua`,
truth/facts, theorem/proof/CoreIr/ControlFlowIr/VC, other asserted-head families,
and general semantics receive no credit.
Task 183 adds the exact active object-terminal formula-side local-mode
asserted-head row `LocalObjectModeAssertedHeadPayloadBoundary: x is
LocalObjectModeAssertedHead;`, with Chapter 3/4/7/13/14.2.3/16 and a dedicated
checker bridge reference. Credit is limited to one real direct object-terminal
expansion, independent raw reserve-subject and formula-side asserted inputs for
the same resolved mode symbol, ordinal 1 `BindingId(0)`, three known type
entries interned to one terminal-definition-RHS builtin-object identity, one
inferred term, and one fact/deferred-free normalized-reflexive checked type
assertion without general reachability or object/set coercion. Five shared
backlinks plus the dedicated row, production routing, exact/near-miss/
corruption guards, and the real frontend/resolver sidecar protect active count
131. Declaration acceptance/inhabitation, truth/facts, theorem/proof/CoreIr/
ControlFlowIr/VC, other asserted-head families, and general semantics receive
no credit.
Task 184 adds the exact active one-edge set-terminal same-outer-mode formula-
side asserted-head row `ChainedLocalModeAssertedHeadPayloadBoundary: x is
ChainModeAssertedHead;`, with Chapter 3/4/7/13/14.2.3/16 and a dedicated
checker bridge reference. Credit is limited to two real expansions,
independent raw reserve-subject and formula-side asserted inputs for the same
resolved outer symbol, ordinal 1 `BindingId(0)`, three known type entries
interned to one terminal-base-definition-RHS builtin-set identity, one inferred
term, and one fact/deferred-free normalized-reflexive checked type assertion
without general reachability. Five shared backlinks plus the dedicated row,
production routing, exact/near-miss/corruption guards, and the real frontend/
resolver sidecar protect active count 132. Declaration acceptance/inhabitation,
widening/`qua`, truth/facts, closure/order, theorem/proof/CoreIr/ControlFlowIr/
VC, object/deeper/other asserted-head chains, and general chain semantics
receive no credit.
Task 185 adds the exact active one-edge object-terminal same-outer-mode formula-
side asserted-head row `ChainedLocalObjectModeAssertedHeadPayloadBoundary: x is
ChainObjectModeAssertedHead;`, with Chapter 3/4/7/13/14.2.3/16 and a dedicated
checker bridge reference. Credit is limited to two real expansions, independent
raw reserve-subject and formula-side asserted inputs for the same resolved outer
symbol, ordinal 1 `BindingId(0)`, three known type entries interned to one
terminal-base-definition-RHS builtin-object identity, one inferred term, and one
fact/deferred-free normalized-reflexive checked type assertion without general
reachability, widening/`qua`, or object/set coercion. Five shared backlinks plus
the dedicated row, production routing, exact/near-miss/corruption guards, and
the real frontend/resolver sidecar protect active count 133. Declaration/
attribute acceptance, truth/facts, closure/order, theorem/proof/CoreIr/
ControlFlowIr/VC, set-terminal/deeper/other asserted-head chains, and general
chain semantics receive no credit.
Task 186 adds the exact active two-edge set-terminal same-outer-mode formula-
side asserted-head row `TwoEdgeLocalModeAssertedHeadPayloadBoundary: x is
OuterTwoEdgeModeAssertedHead;`, with Chapter 3/4/7/13/14.2.3/16 and a dedicated
checker bridge reference. Credit is limited to three real expansions,
independent raw reserve-subject and formula-side asserted inputs for the same
resolved outer symbol, ordinal 1 `BindingId(0)`, three known type entries
interned to one terminal-base-definition-RHS builtin-set identity, one inferred
term, and one fact/deferred-free normalized-reflexive checked type assertion
without reachability, widening, or `qua`. Five shared backlinks plus the
dedicated row, production routing, exact/near-miss/corruption guards, and the
real frontend/resolver sidecar protect active count 134. Declaration/attribute
acceptance, truth/facts, closure/order, theorem/proof/CoreIr/ControlFlowIr/VC,
object-terminal/deeper/imported/other asserted-head chains, and general chain
semantics receive no credit.
Task 187 adds the exact active two-edge object-terminal same-outer-mode formula-
side asserted-head row `TwoEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
OuterTwoEdgeObjectModeAssertedHead;`, with Chapter 3/4/7/13/14.2.3/16 and a
dedicated checker bridge reference. Credit is limited to three real expansions,
independent raw reserve-subject and formula-side asserted sites/ranges for the
same resolved local outer symbol, ordinal 1 `BindingId(0)`, three known type
entries interned to one terminal-base-definition-RHS builtin-object identity,
one inferred term, and one fact/deferred-free normalized-reflexive checked type
assertion without reachability, widening, `qua`, or object/set coercion. Five
shared backlinks plus the dedicated row, production routing, exact/near-miss/
corruption guards, and the real frontend/resolver sidecar protect active count
135. Positive imported semantics, declaration/attribute acceptance, truth/
facts, closure/order, theorem/proof/CoreIr/ControlFlowIr/VC, set-terminal/
deeper/other asserted-head chains, general chain semantics, and downstream
payloads receive no credit.
Task 188 adds the exact active builtin-object reserved-variable equality row
`ReservedObjectVariableEqualityPayloadBoundary: x = x;`, with five shared
Chapter 4/13/14.5.2/16/checker bridge references plus one dedicated checker
bridge reference. Credit is limited to source-order ordinals 1 and 2 resolving
to `BindingId(0)`, four distinct result/expected role sites preserving the one
written `object` type range, one canonical normalized builtin-object identity
anchored at that reserve, two inferred variable terms, two ordered expected
constraints, and one fact/deferred-free `Checked` equality without object/set
coercion. Exact-route, provenance near-miss, mutable payload-corruption, shared
immutable-output validation, and real frontend/resolver sidecar guards protect
active count 136. General or non-reflexive object equality, truth/facts,
closure/order, declaration/theorem acceptance, `formula_statement`, proof,
CoreIr, ControlFlowIr, VC, and downstream payloads receive no credit.
Task 189 adds the exact active builtin-object reserved-variable type-assertion
row `ReservedObjectVariableTypeAssertionPayloadBoundary: x is object;`, with
five shared Chapter 3/4/13/14.2.3/16 references plus one dedicated checker
bridge reference. Credit is limited to source-order ordinal 1 resolving to
`BindingId(0)`, distinct reserve-result and formula-side asserted object sites/
ranges, two raw argument/attribute-free `BuiltinObject` inputs, one canonical
normalized identity anchored at the written reserve type, one inferred
variable, three known type entries, zero expected constraints, and one fact/
deferred-free `Checked` assertion. Exact-route, provenance near-miss, mutable
payload-corruption, shared immutable-output validation, and real frontend/
resolver sidecar guards protect active count 137. Reachability/widening/`qua`,
object/set coercion, truth/facts, closure/order, declaration/theorem acceptance,
`formula_statement`, proof, CoreIr, ControlFlowIr, VC, and downstream payloads
receive no credit.
Task 190 adds the exact active builtin-object reserved-variable inequality row
`ReservedObjectVariableInequalityPayloadBoundary: x <> x;`, with five shared
Chapter 4/13/14.5.2/16/checker bridge references plus one dedicated checker
bridge reference. Credit is limited to source-order ordinals 1 and 2 resolving
to `BindingId(0)`, four distinct result/expected role sites preserving the one
written `object` type range, four raw argument/attribute-free `BuiltinObject`
inputs, one canonical normalized builtin-object identity anchored at that
reserve, two inferred variable terms, six known type entries, two ordered
expected constraints, and one fact/candidate/diagnostic/deferred-free pre-
desugaring `Checked` inequality. Exact-route, structural/provenance near-miss,
mutable payload-corruption, positive immutable-output validation, and real
frontend/resolver sidecar guards protect active count 138. Inequality
desugaring/equality truth, object/set coercion, facts, closure/order,
declaration/theorem acceptance, `formula_statement`, proof, CoreIr,
ControlFlowIr, VC, and downstream payloads receive no credit.
Task 191 adds the exact active distinct-binding shared-builtin-object equality
row `DistinctReservedObjectVariableEqualityPayloadBoundary: x = y;`, with five
shared Chapter 4/13/14.5.2/16/builtin-type bridge references plus one dedicated
checker bridge reference. Credit is limited to source-order ordinals 2 and 3
resolving to `BindingId(0/1)`, one shared written `object` range across both
bindings and four distinct result/expected role sites, four raw argument/
attribute-free `BuiltinObject` inputs, one reserve-range-anchored canonical
normalized builtin-object identity, two inferred variable terms, six known
type entries, two ordered expected constraints, and one fact/candidate/
diagnostic/deferred-free `Checked` equality. Exact-route, structural/provenance
near-miss, mutable payload-corruption, positive immutable-output validation,
and a real frontend/resolver sidecar protect active count 139.
Equality truth, object/set coercion, facts, closure/order, declaration/theorem
acceptance, `formula_statement`, proof, CoreIr, ControlFlowIr, VC, and
downstream payloads receive no credit.
Task 192 adds one exact active distinct-binding shared-builtin-object
inequality row `DistinctReservedObjectVariableInequalityPayloadBoundary: x <>
y;`, with five shared Chapter 4/13/14.5.2/16/builtin-type bridge references
plus one dedicated checker bridge reference. Credit is limited to source-order
ordinals 2 and 3 resolving to `BindingId(0/1)`, one shared written `object`
range across both bindings and four distinct result/expected role sites, four
raw argument/attribute-free `BuiltinObject` inputs, one reserve-range-anchored
canonical normalized builtin-object identity, two inferred variable terms,
six known type entries, two ordered expected constraints, and one fact/
candidate/diagnostic/deferred-free `Checked` inequality. Exact-route,
structural/provenance near-miss, mutable payload-corruption, positive immutable-
output validation, and a real frontend/resolver sidecar protect active count
140. Inequality desugaring/equality truth, object/set coercion, facts,
closure/order, declaration/theorem acceptance, `formula_statement`, proof,
CoreIr, ControlFlowIr, VC, and downstream payloads receive no credit.
Task 193 adds one exact active multiple-reserve-declaration builtin-object
equality row `reserve x for object; reserve y for object; theorem
MultipleObjectReserveDeclarationEqualityPayloadBoundary: x = y;`, with five
shared Chapter 4/13/14.5.2/16/builtin-type bridge references plus one dedicated
checker bridge reference. Credit is limited to source-order ordinals 2 and 3
resolving to `BindingId(0/1)`, two distinct binding-owned written `object`
ranges across four distinct result/expected role sites, four raw argument/
attribute-free `BuiltinObject` inputs, one earlier-`x`-range-anchored canonical
normalized builtin-object identity, two inferred variable terms, six known
type entries, two ordered expected constraints, and one fact/candidate/
diagnostic/deferred-free `Checked` equality. Exact-route, structural/
provenance near-miss, mutable payload-corruption, positive immutable-output
validation, and a real frontend/resolver sidecar protect active count 141.
Equality truth, object/set coercion, facts, closure/order, declaration/theorem
acceptance, `formula_statement`, proof, CoreIr, ControlFlowIr, VC, shared-range
shapes, and downstream payloads receive no credit.
Task 194 adds one exact active multiple-reserve-declaration builtin-object
inequality row `reserve x for object; reserve y for object; theorem
MultipleObjectReserveDeclarationInequalityPayloadBoundary: x <> y;`, with five
shared Chapter 4/13/14.5.2/16/builtin-type bridge references plus one dedicated
checker bridge reference. Credit is limited to source-order ordinals 2 and 3
resolving to `BindingId(0/1)`, two ordered binding-owned written `object`
ranges across four distinct raw result/expected roles, four argument/attribute-
free `BuiltinObject` inputs, one earlier-`x`-range-anchored canonical normalized
builtin-object identity, two inferred variable terms, six known type entries,
two ordered expected constraints, and one fact/candidate/diagnostic/deferred-
free pre-desugaring `Checked` inequality. Exact-route, structural/provenance
near-miss, mutable payload-corruption, immutable-output validation, and a real
frontend/resolver sidecar protect active count 142. Inequality desugaring/
equality truth, object/set coercion, facts, closure/order, declaration/theorem
acceptance, `formula_statement`, proof, CoreIr, ControlFlowIr, VC, shared-range
shapes, and downstream payloads receive no credit.

Task 195 adds one exact active three-edge set-terminal same-outer-mode asserted-
head row with four ordered local definitions `Outer -> Middle -> Inner -> Base
-> set`, one outer-mode reserve, and
`ThreeEdgeLocalModeAssertedHeadPayloadBoundary: x is
OuterThreeEdgeModeAssertedHead;`, with five shared Chapter 4/7/13/14.2.3/16
references plus one dedicated checker bridge reference. Credit is limited to
four real AST-derived expansions, ordinal 1 resolving to `BindingId(0)`,
distinct raw reserve-subject and formula asserted-type sites/ranges for the
same outer symbol, three known type entries normalizing to one base-definition-
RHS-anchored `BuiltinSet` identity, one inferred variable, zero expected
constraints/candidates/facts/diagnostics/deferred reasons, and one normalized-
reflexive `Checked` type assertion. Exact-route, structural/provenance near-
misses including unrelated local/imported/ambiguous asserted heads, mutable
corruption, immutable-output validation, and a real frontend/resolver sidecar
protect active count 143. Reachability/widening/`qua`, declaration/theorem
acceptance, truth/facts, closure/order, `formula_statement`, proof, CoreIr,
ControlFlowIr, VC, broader formula/child-graph semantics, and general chains
receive no credit.

Task 196 adds one exact active three-edge object-terminal same-outer-mode
asserted-head row with four ordered local definitions `Outer -> Middle -> Inner
-> Base -> object`, one outer-mode reserve, and
`ThreeEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
OuterThreeEdgeObjectModeAssertedHead;`, with five shared Chapter 4/7/13/14.2.3/
16 references plus one dedicated checker bridge reference. Credit is limited
to four real AST-derived expansions, ordinal 1 resolving to `BindingId(0)`,
distinct raw reserve-subject and formula asserted-type sites/ranges for the
same outer symbol, three known type entries normalizing to one base-definition-
RHS-anchored `BuiltinObject` identity, one inferred variable, zero expected
constraints/candidates/facts/diagnostics/deferred reasons, and one normalized-
reflexive `Checked` type assertion without object/set coercion. Exact-route,
structural/provenance near-misses including unrelated local/imported/ambiguous
asserted heads, `BuiltinSet`/canonical mutable corruption, immutable-output
validation, and a real frontend/resolver sidecar protect active count 144.
Reachability/widening/`qua`, declaration/theorem acceptance, truth/facts,
closure/order, `formula_statement`, proof, CoreIr, ControlFlowIr, VC, broader
formula/child-graph semantics, and general chains receive no credit.

Task 197 adds one exact active four-edge set-terminal same-outermost-mode
asserted-head row with five ordered local definitions `TooDeep -> Outer ->
Middle -> Inner -> Base -> set`, one outermost-mode reserve, and
`FourEdgeLocalModeAssertedHeadPayloadBoundary: x is
TooDeepFourEdgeModeAssertedHead;`, with four shared Chapter 4/13/14.2.3/16
references, one shared Task 74 structural-chain reference, and one dedicated
checker bridge reference. Credit is limited to five real AST-derived
expansions, ordinal 1 resolving to `BindingId(0)`, distinct raw reserve-subject
and formula asserted-type sites/ranges for the same outermost symbol, three
known type entries normalizing to one base-definition-RHS-anchored `BuiltinSet`
identity, one inferred variable, zero expected constraints/candidates/facts/
diagnostics/deferred reasons, and one normalized-reflexive `Checked` type
assertion. Exact-route, full-reorder, connected-deeper, structural/provenance
near-misses including unrelated local/imported/ambiguous asserted heads,
`BuiltinObject`/canonical mutable corruption, immutable-output validation, and
a real frontend/resolver sidecar protect active count 145. Reachability/
widening/`qua`, declaration/theorem acceptance, truth/facts, closure/order,
`formula_statement`, proof, CoreIr, ControlFlowIr, VC, broader formula/child-
graph semantics, and general chains receive no credit.

Task 198 adds one exact active four-edge object-terminal same-outermost-mode
asserted-head row with five ordered local definitions `TooDeep -> Outer ->
Middle -> Inner -> Base -> object`, one outermost-mode reserve, and
`FourEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
TooDeepFourEdgeObjectModeAssertedHead;`, with four shared Chapter 4/13/14.2.3/
16 references, one shared Task 74 structural-chain reference, and one dedicated
checker bridge reference. Credit is limited to five real AST-derived
expansions, ordinal 1 resolving to `BindingId(0)`, distinct raw reserve-subject
and formula asserted-type sites/ranges for the same outermost symbol, three
known type entries normalizing to one base-definition-RHS-anchored
`BuiltinObject` identity, one inferred variable, zero expected constraints/
candidates/facts/diagnostics/deferred reasons, and one normalized-reflexive
`Checked` type assertion without object/set coercion. Exact-route, full-
reorder, connected-deeper, structural/provenance near misses including
unrelated local/imported/ambiguous asserted heads, `BuiltinSet`/canonical
mutable corruption, immutable-output validation, and a real frontend/resolver
sidecar protect active count 146. Reachability/widening/`qua`, declaration/
theorem acceptance, truth/facts, closure/order, `formula_statement`, proof,
CoreIr, ControlFlowIr, VC, broader formula/child-graph semantics, and general
chains receive no credit.

Task 199 adds one exact active seven-expansion set-terminal same-`ChainMode6`
asserted-head row with `BaseMode -> set`, six ordered local links through
`ChainMode6 -> ChainMode5`, one `ChainMode6` reserve, and
`LongLocalModeAssertedHeadPayloadBoundary: x is ChainMode6;`, with four shared
Chapter 4/13/14.2.3/16 references, one shared Task 74 structural-chain
reference, and one dedicated checker bridge reference. Credit is limited to
seven real AST-derived expansions, ordinal 1 resolving to `BindingId(0)`,
distinct raw reserve-subject and formula asserted-type sites/ranges for the same
symbol, three known type entries normalizing to one `BaseModeDef` RHS-anchored
`BuiltinSet` identity, one inferred variable, zero expected constraints/
candidates/facts/diagnostics/deferred reasons, and one normalized-reflexive
`Checked` type assertion. Exact-route, per-link removal/reorder, complete-
reverse, connected-eighth, structural/provenance near misses including
unrelated local/imported/ambiguous asserted heads, `BuiltinObject`/canonical
mutable corruption, immutable-output validation, and a real frontend/resolver
sidecar protect active count 147. Object-terminal/other-depth/imported/
attributed/argument-bearing/other asserted heads, reachability/widening/`qua`,
declaration/theorem acceptance, truth/facts, closure/order,
`formula_statement`, proof, CoreIr, ControlFlowIr, VC, broader formula/child-
graph semantics, and general unbounded chains receive no credit.

Task 200 adds one exact active seven-expansion object-terminal same-
`ChainObjectMode6` asserted-head row with `BaseObjectMode -> object`, six
ordered local links through `ChainObjectMode6 -> ChainObjectMode5`, one
`ChainObjectMode6` reserve, and
`LongLocalObjectModeAssertedHeadPayloadBoundary: x is ChainObjectMode6;`, with
four shared Chapter 4/13/14.2.3/16 references, one shared Task 74 structural-
chain reference, and one dedicated checker bridge reference. Credit is limited
to seven real AST-derived expansions, ordinal 1 resolving to `BindingId(0)`,
distinct raw reserve-subject and formula asserted-type sites/ranges for the same
symbol, three known type entries normalizing to one `BaseObjectModeDef` RHS-
anchored `BuiltinObject` identity, one inferred variable, zero expected
constraints/candidates/facts/diagnostics/deferred reasons, and one normalized-
reflexive `Checked` type assertion without object/set coercion. Exact-route, per-
link removal/reorder, complete-reverse, connected-eighth, structural/provenance
near misses including unrelated local/imported/ambiguous asserted heads,
`BuiltinSet`/canonical mutable corruption, immutable-output validation, and a
real frontend/resolver sidecar protect active count 148. Set-terminal/other-
depth/imported/attributed/argument-bearing/other asserted heads, reachability/
widening/`qua`, declaration/theorem acceptance, truth/facts, closure/order,
`formula_statement`, proof, CoreIr, ControlFlowIr, VC, broader formula/child-
graph semantics, and general unbounded chains receive no credit.

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
set-enumeration term extraction, result-type payloads, term inference,
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

Those gap tests do not satisfy the broader task 7-11 semantic pass/fail
coverage. Core Task 31 separately promotes exactly
`spec.en.mizar_core.core_ir.task180_type_elaboration_snapshot`: its existing
Task-180 sidecar is the sole backlink and the runner verify-compares the real
exact CoreIr. The broad `CoreIr`, every `ControlFlowIr`, and
`proof_verification` rows remain deferred until their prepared consumer
execution exists; summary/context readiness alone is not a
CoreIr/ControlFlowIr/VC/proof promotion.

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


## Task 201 Traceability

Task 201 adds one exact active immediate-radix asserted-head row with four shared Chapter 4/13/14.2.3/16 references, the shared Task 56 chain-producer reference, and one dedicated checker bridge reference. Credit is limited to two real AST-derived expansions, distinct Outer reserve-subject and Base formula asserted-type symbols/sites/ranges, ordinal 1 resolving to `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinSet`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred `Checked` type assertion. Exact route, provenance/corruption, Task 146/184 isolation, immutable-output, and real sidecar tests protect active count 149. The plan contains 364 cases and 328 requirements. Broader asserted-head, declaration, theorem, formula-child, proof, CoreIr, ControlFlowIr, and VC semantics receive no credit.


## Task 202 Traceability

Task 202 adds one exact object-terminal immediate-radix active row with four shared Chapter 4/13/14.2.3/16 references, the shared Task 56 chain-producer reference, and one dedicated checker reference. Credit is limited to two real expansions, distinct Outer/Base symbols/sites/ranges, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinObject`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without coercion. Exact/corruption, real Tasks 147/185/201 isolation, immutable-output, and sidecar tests protect active count 150. The plan has 365 cases and 329 requirements; broader semantics receive no credit.


## Task 203 Traceability

Task 203 adds one exact two-edge set-terminal immediate-radix active row with five shared Chapter 4/7/13/14.2.3/16 references and one dedicated checker reference. Credit is limited to three real expansions, distinct Outer/Middle symbols/sites/ranges, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinSet`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Exact/corruption/order/duplicate/spelling/imported/ambiguous/deeper coverage, real Tasks 122/148/149/186/187/201/202 isolation, immutable output, and a real sidecar protect active count 151. The plan has 366 cases and 330 requirements without rebaselining existing expectations; two-hop Base assertion and broader semantics receive no credit.


## Task 204 Traceability

Task 204 adds one exact two-edge object-terminal immediate-radix active row with five shared Chapter 4/7/13/14.2.3/16 references and one dedicated checker reference. Credit is limited to three real object expansions, distinct Outer/Middle symbols/sites/ranges, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinObject`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Exact/corruption/order/duplicate/spelling/imported/ambiguous/deeper coverage, real Tasks 189/145/147/149/187/202 and set Tasks 148/186/203 isolation, immutable output, and a real sidecar protect active count 152. The plan has 367 cases and 331 requirements without rebaselining existing expectations; two-hop Base assertion and broader semantics receive no credit.

## Task 205 Traceability

Task 205 adds one exact three-edge set-terminal immediate-radix active row with five shared Chapter 4/7/13/14.2.3/16 references and one dedicated checker reference. Credit is limited to four real set-terminal expansions, distinct Outer/Middle symbols/sites/ranges, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinSet`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Exact/corruption/all-23-orders/missing/duplicate/label/spelling/radix/imported/ambiguous/deeper/multi-hop coverage, bidirectional isolation against all 16 declared owner routes, immutable output, and a real sidecar protect active count 153. The plan has 368 cases and 332 requirements without rebaselining existing expectations; multi-hop Inner/Base assertion, the matching object sibling, and broader semantics receive no credit.

## Task 206 Traceability

Task 206 adds one exact three-edge object-terminal immediate-radix active row with five shared Chapter 4/7/13/14.2.3/16 references and one dedicated checker reference. Credit is limited to four real object-terminal expansions, distinct Outer/Middle symbols/sites/ranges, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinObject`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Exact/corruption/all-23-orders/per-definition/imported/ambiguous/deeper/multi-hop/local-other coverage, bidirectional isolation against all 17 declared owner routes, immutable output, and a real sidecar protect active count 154. The plan has 369 cases and 333 requirements without rebaselining existing expectations; multi-hop Inner/Base assertion and broader semantics receive no credit.

## Task 207 Traceability

Task 207 adds one exact four-edge set-terminal immediate-radix active row with four shared Chapter 4/13/14.2.3/16 references, one shared Task 74 producer reference, and one dedicated checker reference. Credit is limited to five real set-terminal expansions, distinct TooDeep/Outer symbols/sites/ranges, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinSet`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Exact/corruption/all-119-orders/per-definition/imported/ambiguous/deeper/multi-hop/local-other coverage, bidirectional isolation against all 20 declared owner routes, immutable output, and a real sidecar protect active count 155. The plan has 370 cases and 334 requirements without rebaselining existing expectations; multi-hop Middle/Inner/Base assertions, the matching object sibling, and broader semantics receive no credit.

## Task 208 Traceability

Task 208 adds one exact four-edge object-terminal immediate-radix active row with four shared Chapter 4/13/14.2.3/16 references, one shared Task 74 producer reference, and one dedicated checker reference. Credit is limited to five real object-terminal expansions, distinct TooDeep/Outer symbols/sites/ranges, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinObject`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Exhaustive source/provenance/corruption coverage, bidirectional isolation against all 21 declared owner routes, immutable output, and a real sidecar protect active count 156. The plan has 371 cases and 335 requirements without rebaselining existing expectations; multi-hop Middle/Inner/Base assertions and broader semantics receive no credit.

## Task 209 Traceability

Task 209 adds one exact seven-expansion set-terminal immediate-radix active row with four shared Chapter 4/13/14.2.3/16 references, one shared Task 74 structural-producer reference, and one dedicated checker reference. Credit is limited to seven real expansions, distinct ChainMode6/ChainMode5 symbols/sites/ranges, ordinal 1 / `BindingId(0)`, three known entries normalizing to one BaseModeDef-RHS `BuiltinSet`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. All 5,039 nonidentity orders, the declared finite source/provenance/corruption matrix, all 34 pre-existing owner routes, immutable output, and a real sidecar protect active count 157. The plan has 372 cases and 336 requirements without rebaselining existing expectations; multi-hop and broader semantics receive no credit.

## Task 210 Traceability

Task 210 adds one exact seven-expansion object-terminal immediate-radix active row with four shared Chapter 4/13/14.2.3/16 references, one shared Task 74 structural-producer reference, and one dedicated checker reference. Credit is limited to seven real object-terminal expansions, distinct ChainObjectMode6/ChainObjectMode5 symbols/sites/ranges, ordinal 1 / `BindingId(0)`, three known entries normalizing to one BaseObjectModeDef-RHS `BuiltinObject`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. All 5,039 nonidentity orders, the declared finite source/provenance/corruption matrix, all 35 pre-existing owner routes, immutable output, and a real sidecar protect active count 158. The plan has 373 cases and 337 requirements without rebaselining existing expectations; multi-hop and broader semantics receive no credit.

## Task 211 Traceability

Task 211 adds one exact two-edge set-terminal two-hop asserted-head active row with four shared Chapter 4/13/14.2.3/16 references, one shared Task 72 two-edge structural-producer reference, and one dedicated checker reference. Credit is limited to three real expansions, distinct Outer/Base symbols/sites/ranges, both actual bare links, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinSet`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. All five nonidentity definition orders, the finite structural/provenance/corruption matrix, all 36 prior owner routes, immutable output, and a real sidecar protect active count 159. The plan has 374 cases and 338 requirements without rebaselining existing expectations; other distances, generic reachability, and broader semantics receive no credit.

## Task 212 Traceability

Task 212 adds one exact two-edge object-terminal two-hop asserted-head active row with four shared Chapter 4/13/14.2.3/16 references, one shared Task 72 two-edge structural-producer reference, and one dedicated checker reference. Credit is limited to three real object expansions, distinct Outer/Base symbols/sites/ranges, both actual bare links, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinObject`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. All five nonidentity definition orders, the finite structural/provenance/corruption matrix, all 37 prior owner routes, immutable output, and a real sidecar protect active count 160. The plan has 375 cases and 339 requirements without rebaselining existing expectations; other distances, generic reachability, object/set coercion, and broader semantics receive no credit.

## Task 213 Traceability

Task 213 adds one exact three-edge set-terminal two-hop asserted-head active row with four shared Chapter 4/13/14.2.3/16 references, one shared Task 73 structural-producer reference, and one dedicated checker reference. Credit is limited to four real expansions, distinct Outer/Inner symbols/sites/ranges, the two explicitly validated relation links, a terminal-only Inner-to-Base-to-set tail, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinSet`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. All 23 nonidentity definition orders, the finite structural/provenance/corruption matrix, focused Task 211/212 regressions, all 38 prior owner routes, immutable output, and a real sidecar protect active count 161. The plan has 376 cases and 340 requirements, with type-elaboration coverage 208/196, without rebaselining existing expectations; the object sibling, other distances, generic reachability, and broader semantics receive no credit.

## Task 214 Traceability

Task 214 adds one exact three-edge object-terminal two-hop asserted-head active row with four shared Chapter 4/13/14.2.3/16 references, one shared Task 73 structural-producer reference, and one dedicated checker reference. Credit is limited to four real object expansions, distinct Outer/Inner symbols/sites/ranges, the two explicitly validated relation links, a terminal-only Inner-to-Base-to-object tail, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinObject`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. All 23 nonidentity definition orders, the finite structural/provenance/corruption matrix, focused Task 211/212/213 regressions, all 39 prior owner routes, immutable output, and a real sidecar protect active count 162. The plan has 377 cases and 341 requirements, type-elaboration coverage 209/197, and pass/fail 193/184 without rebaselining existing expectations; other distances, generic reachability, object/set coercion, and broader semantics receive no credit.

## Task 215 Traceability

Task 215 adds one exact four-edge set-terminal two-hop asserted-head active row with five shared Chapter 4/13/14.2.3/16 and Task 74 structural-producer references plus one dedicated checker reference. Credit is limited to five real set expansions, distinct TooDeep/Middle symbols/sites/ranges, the two explicitly validated relation links, a terminal-only Middle-to-Inner-to-Base-to-set tail, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinSet`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. All 119 nonidentity definition orders, the finite structural/provenance/corruption matrix, focused Tasks 211-214 regressions, all 40 prior owner routes, immutable output, and a real sidecar protect active count 163. The plan has 378 cases and 342 requirements, type-elaboration coverage 210/198, and pass/fail 194/184 without rebaselining existing expectations; the object sibling, other distances, generic reachability, object/set coercion, and broader semantics receive no credit.

## Task 216 Traceability

Task 216 adds one exact four-edge object-terminal two-hop asserted-head active row with five shared Chapter 4/13/14.2.3/16 and Task 74 structural-producer references plus one dedicated checker reference. Credit is limited to five real object expansions, distinct TooDeep/Middle symbols/sites/ranges, the two explicitly validated relation links, a terminal-only Middle-to-Inner-to-Base-to-object tail, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinObject`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. All 119 nonidentity definition orders, the finite structural/provenance/corruption matrix, focused Tasks 211-215 regressions, all 41 prior owner routes, immutable output, and a real sidecar protect active count 164. The plan has 379 cases and 343 requirements, type-elaboration coverage 211/199, and pass/fail 195/184 without rebaselining existing expectations; other distances, generic reachability, object/set coercion, and broader semantics receive no credit.

## Task 217 Traceability

Task 217 adds one exact three-edge set-terminal three-hop asserted-head active row with four shared Chapter 4/13/14.2.3/16 references, one shared Task 73 structural-producer reference, and one dedicated checker reference. Credit is limited to four real set expansions, distinct Outer/Base symbols/sites/ranges, the three explicitly validated relation links, terminal-only Base-to-set normalization, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinSet`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. All 23 nonidentity definition orders, the finite structural/provenance/corruption matrix, focused Tasks 211-216 regressions, all 42 prior owner routes, immutable output, and a real sidecar protect active count 165. The plan has 380 cases and 344 requirements, type-elaboration coverage 212/200, and pass/fail 196/184 without rebaselining existing expectations; the object sibling, other depths, generic reachability, object/set coercion, and broader semantics receive no credit.

## Task 218 Traceability

Task 218 adds one exact three-edge object-terminal three-hop asserted-head active row with four shared Chapter 4/13/14.2.3/16 references, one shared Task 73 structural-producer reference, and one dedicated checker reference. Credit is limited to four real object expansions, distinct Outer/Base symbols/sites/ranges, the three explicitly validated relation links, terminal-only Base-to-object normalization, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinObject`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. All 23 nonidentity definition orders, the finite structural/provenance/corruption matrix, focused Tasks 211-217 regressions, all 43 prior owner routes, immutable output, and a real sidecar protect active count 166. The plan has 381 cases and 345 requirements, type-elaboration coverage 213/201, and pass/fail 197/184 without rebaselining existing expectations; other depths, generic reachability, object/set coercion, and broader semantics receive no credit.

## Task 219 Traceability

Task 219 adds one exact four-edge set-terminal three-hop asserted-head active row with four shared Chapter 4/13/14.2.3/16 references, one shared Task 74 structural-producer reference, and one dedicated checker reference. Credit is limited to five real set expansions, distinct TooDeep/Inner symbols/sites/ranges, the three explicitly validated relation links, terminal-only Inner-to-Base-to-set normalization, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinSet`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. All 119 nonidentity definition orders, the finite structural/provenance/corruption matrix with independent guards for an unconnected unsupported deeper asserted head and an actual connected sixth-definition/sixth-edge asserted head, focused Task 207 and Tasks 211-218 regressions, all 44 prior owner routes, immutable output, and a real sidecar protect active count 167. The plan has 382 cases and 346 requirements, type-elaboration coverage 214/202, and pass/fail 198/184 without rebaselining existing expectations; the object sibling, Base full-distance assertion, generic reachability, object/set coercion, and broader semantics receive no credit.

## Task 220 Traceability

Task 220 adds one exact four-edge object-terminal three-hop asserted-head active row with four shared Chapter 4/13/14.2.3/16 references, one shared Task 74 structural-producer reference, and one dedicated checker reference. Credit is limited to five real object expansions, distinct TooDeep/Inner symbols/sites/ranges, the three explicitly validated relation links, terminal-only Inner-to-Base-to-object normalization, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinObject`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. All 119 nonidentity definition orders, the finite structural/provenance/corruption matrix with independent guards for an unconnected unsupported deeper asserted head and an actual connected sixth-definition/sixth-edge asserted head, focused Tasks 208 and 211-219 regressions, all 45 prior owner routes, immutable output, and a real sidecar protect active count 168. The plan has 383 cases and 347 requirements, type-elaboration coverage 215/203, and pass/fail 199/184 without rebaselining existing expectations; the Base full-distance assertion, generic reachability, object/set coercion, and broader semantics receive no credit.

## Task 221 Traceability

Task 221 adds one exact four-edge set-terminal full-distance four-hop asserted-head active row with four shared Chapter 4/13/14.2.3/16 references, one shared Task 74 structural-producer reference, and one dedicated checker reference. Credit is limited to five real set expansions, distinct TooDeep/Base symbols/sites/ranges, four explicitly validated relation links, terminal-only Base-to-set normalization, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinSet`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. All 119 nonidentity orders, exhaustive finite structural/provenance/corruption coverage with independent unconnected-deeper and actual connected fifth-link guards, focused Task 207 and Tasks 211-220 regressions, all 46 prior owner routes, immutable output, and a real sidecar protect active count 169. The plan has 384 cases and 348 requirements, type-elaboration coverage 216/204, and pass/fail 200/184 without rebaselining existing expectations; the object sibling, longer chains, imported-positive definitions, attributed/argument-bearing behavior, generic reachability, and broader semantics receive no credit.

## Task 222 Traceability

Task 222 adds one exact four-edge object-terminal full-distance four-hop asserted-head active row with four shared Chapter 4/13/14.2.3/16 references, one shared Task 74 structural-producer reference, and one dedicated checker reference. Credit is limited to five real object expansions, distinct TooDeep/Base symbols/sites/ranges, four explicitly validated relation links, terminal-only Base-to-object normalization, ordinal 1 / `BindingId(0)`, three known entries normalizing to one Base-definition-RHS `BuiltinObject`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. All 119 nonidentity orders, exhaustive finite structural/provenance/corruption coverage with independent unconnected-deeper and actual connected fifth-link guards, focused Task 208 and Tasks 211-221 regressions, all 47 prior owner routes, immutable output, and a real sidecar protect active count 170. The active corpus has 385 cases and 349 requirements, type-elaboration coverage 217/205, and pass/fail 201/184 without rebaselining existing expectations; longer chains, imported-positive definitions, attributed/argument-bearing behavior, generic reachability, object/set coercion, and broader semantics receive no credit.

## Task 223 Traceability

Task 223 adds one exact single-left-parenthesized reserved-variable equality active row with four shared Chapter 4/13/14.5.2/16 references and one dedicated checker reference. Chapter 13's shared row expands its section label to include §§13.1.3 and 13.8.8 without changing prior credit. New credit is limited to one real `ParenthesizedTerm` wrapper, one inner and one direct-right `x` reference, independent wrapper/inner/right source metadata, ordinal 1/2 `BindingId(0)` lookup, and transparent reuse of the inner reference's real reserve-derived builtin-set value/type in the existing equality consumer. Parentheses receive no independent type, axiom, fact, FOL node, or child-graph credit. The finite exact/near-miss/corruption matrix, all 52 prior reserved-variable binary-formula owners, immutable output, and a real sidecar protect active count 171. The active corpus has 386 cases and 350 requirements, type-elaboration coverage 218/206, and pass/fail 202/184 without rebaselining existing expectations. Focused, relevant-crate, and workspace verification passed; arbitrary parentheses/precedence, formula grouping, closure/order, truth/facts, acceptance, proof/IR/VC, and broader semantics receive no credit.

## Task 224 Traceability

Task 224 adds one active seven-expansion set-terminal two-hop asserted-head row with shared Chapter 4/13/14/16 and Task 74 structural-producer references plus one dedicated checker reference. Credit is limited to seven real expansions, distinct `ChainMode6`/`ChainMode4` provenance, two directly validated bare links, terminal-only tail normalization, ordinal 1 / `BindingId(0)`, one BaseModeDef-RHS `BuiltinSet`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. The finite matrix, all 48 prior owners, immutable output, focused siblings, and a real sidecar protect active count 172. The active corpus has 387 cases / 351 requirements, type-elaboration 219/207, and pass/fail 203/184 without rebaselining existing expectations. Focused, relevant-crate, and workspace verification passed; broader semantics receive no credit.

## Task 225 Traceability

Task 225 adds one active seven-expansion object-terminal two-hop asserted-head row with shared Chapter 4/13/14/16 and Task 74 structural-producer references plus one dedicated checker reference. Credit is limited to seven real object expansions, distinct `ChainObjectMode6`/`ChainObjectMode4` provenance, two directly validated bare links, terminal-only tail normalization, ordinal 1 / `BindingId(0)`, one BaseObjectModeDef-RHS `BuiltinObject`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. The finite matrix, all 49 prior owners, immutable output, focused siblings, and a real sidecar protect active count 173. The active corpus has 388 cases / 352 requirements, type-elaboration 220/208, and pass/fail 204/184 without rebaselining existing expectations; focused, relevant-crate, and workspace verification passed; broader semantics receive no credit.

## Task 226 Traceability

Task 226 adds one active seven-expansion set-terminal three-hop asserted-head row with shared Chapter 4/13/14/16 and Task 74 structural-producer references plus one dedicated checker reference. Credit is limited to seven real set expansions, distinct `ChainMode6`/`ChainMode3` provenance, three directly validated bare links, terminal-only tail normalization, ordinal 1 / `BindingId(0)`, one BaseModeDef-RHS `BuiltinSet`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. The finite matrix, all 50 prior owners, immutable output, focused siblings, and a real sidecar protect active count 174. The active corpus has 389 cases / 353 requirements, type-elaboration 221/209, and pass/fail 205/184 without rebaselining existing expectations; focused, relevant-crate, and workspace verification passed; broader semantics receive no credit.

## Task 227 Active Traceability

Task 227 adds one active seven-expansion object-terminal three-hop asserted-head row with shared Chapter 4/13/14/16 and Task 74 structural-producer references plus one dedicated checker reference. Credit is limited to seven real object expansions, distinct `ChainObjectMode6`/`ChainObjectMode3` provenance, three directly validated bare links, terminal-only tail normalization, ordinal 1 / `BindingId(0)`, one BaseObjectModeDef-RHS `BuiltinObject`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. The finite matrix, all 51 prior owners, immutable output, focused siblings, and a real sidecar protect active count 175. The active corpus has 390 cases / 354 requirements, type-elaboration 222/210, and pass/fail 206/184 without rebaselining existing expectations; focused, relevant-crate, and workspace verification passed; broader semantics receive no credit.

## Task 228 Active Traceability

Task 228 adds one active seven-expansion set-terminal four-hop asserted-head row with shared Chapter 4/13/14/16 and Task 74 structural-producer references plus one dedicated checker reference. Credit is limited to seven real set expansions, distinct `ChainMode6`/`ChainMode2` provenance, four directly validated bare links, terminal-only tail normalization, ordinal 1 / `BindingId(0)`, one BaseModeDef-RHS `BuiltinSet`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. The finite matrix, all 52 prior owners, immutable output, focused siblings, and a real sidecar protect active count 176. The active corpus has 391 cases / 355 requirements, type-elaboration 223/211, and pass/fail 207/184 without rebaselining existing expectations; focused, relevant-crate, and workspace verification passed; broader semantics receive no credit.

## Task 229 Active Traceability

Task 229 adds one active seven-expansion object-terminal four-hop asserted-head row with shared Chapter 4/13/14/16 and Task 74 structural-producer references plus one dedicated checker reference. Credit is limited to seven real object expansions, distinct `ChainObjectMode6`/`ChainObjectMode2` provenance, four directly validated bare links, terminal-only tail normalization, ordinal 1 / `BindingId(0)`, one BaseObjectModeDef-RHS `BuiltinObject`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. The finite matrix, all 53 prior owners, immutable output, focused siblings, and a real sidecar protect active count 177. The active corpus has 392 cases / 356 requirements, type-elaboration 224/212, and pass/fail 208/184 without rebaselining existing expectations; focused, relevant-crate, and workspace verification passed; broader semantics receive no credit.

## Task 230 Active Traceability

Task 230 adds one active seven-expansion set-terminal five-hop asserted-head row with shared Chapter 4/13/14/16 and Task 74 structural-producer references plus one dedicated checker reference. Credit is limited to seven real set expansions, distinct `ChainMode6`/`ChainMode1` provenance, five directly validated bare links, terminal-only `ChainMode1 -> BaseMode -> set` normalization, ordinal 1 / `BindingId(0)`, one BaseModeDef-RHS `BuiltinSet`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. The finite matrix, all 54 prior owners, immutable output, focused siblings, and a real sidecar protect active count 178. The active corpus has 393 cases / 357 requirements, type-elaboration 225/213, and pass/fail 209/184 without rebaselining existing expectations; focused, relevant-crate, and workspace verification passed; broader semantics receive no credit.

## Task 231 Active Traceability

Task 231 adds one active seven-expansion object-terminal five-hop asserted-head row with shared Chapter 4/13/14/16 and Task 74 structural-producer references plus one dedicated checker reference. Credit is limited to seven real object expansions, distinct `ChainObjectMode6`/`ChainObjectMode1` provenance, five directly validated bare links, terminal-only `ChainObjectMode1 -> BaseObjectMode -> object` normalization, ordinal 1 / `BindingId(0)`, one BaseObjectModeDef-RHS `BuiltinObject`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. The finite matrix, all 55 prior owners, immutable output, focused siblings, and a real sidecar protect active count 179. The active corpus has 394 cases / 358 requirements, type-elaboration 226/214, and pass/fail 210/184 without rebaselining existing expectations; focused, relevant-crate, and workspace verification passed; broader semantics receive no credit.

## Task 233 Active Traceability

Task 233 adds one active exact single-left-parenthesized builtin-object reserved-variable equality row with shared Chapter 4/13/14/16 and builtin-type-expression references plus one dedicated checker reference. Credit is limited to one real unrecovered `ParenthesizedTerm`, one inner and one direct-right `x`, independent wrapper/inner/right source metadata, ordinal 1/2 `BindingId(0)` lookup, and transparent reuse of one canonical reserve-derived `BuiltinObject` identity in the existing equality consumer without object/set coercion or an independent wrapper payload. Six backlinks, the finite exact/near-miss/provenance/corruption matrix, all 53 prior binary-formula owners, immutable output, and a real sidecar protect active count 180. The active corpus has 395 cases / 359 requirements, type-elaboration 227/215, and pass/fail 211/184 without rebaselining existing expectations. Arbitrary parentheses/precedence, formula grouping, closure/order, truth/facts, acceptance, proof/IR/VC, child graphs, and broader semantics receive no credit.

## Task 234 Active Traceability

Task 234 adds one active test-first seven-expansion set-terminal full-distance six-hop asserted-head row with shared Chapter 4/13/14/16 and Task 74 structural-producer references plus one dedicated checker reference. Credit is limited to seven real expansions, distinct `ChainMode6`/`BaseMode` provenance, six directly validated bare links, terminal-only `BaseMode -> set` normalization, ordinal 1 / `BindingId(0)`, one BaseModeDef-RHS `BuiltinSet`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion. Six backlinks, the all-5,039-order finite matrix, all 56 prior owners, immutable output, focused siblings, and a real sidecar protect active count 181. The active corpus has 396 cases / 360 requirements, type-elaboration 228/216, and pass/fail 212/184 without rebaselining existing expectations; broader semantics receive no credit.

## Task 236 Active Traceability

Task 236 adds one active test-first seven-expansion object-terminal full-distance six-hop asserted-head row with shared Chapter 4/13/14/16 and Task 74 structural-producer references plus one dedicated checker reference. Credit is limited to seven real object expansions, distinct `ChainObjectMode6`/`BaseObjectMode` provenance, six directly validated bare links, terminal-only `BaseObjectMode -> object` normalization, ordinal 1 / `BindingId(0)`, one BaseObjectModeDef-RHS `BuiltinObject`, one inferred variable, and one zero-constraint/fact/candidate/diagnostic/deferred checked assertion without object/set coercion. Six backlinks, the all-5,039-order finite matrix, all 57 prior owners, immutable output, focused siblings, and a real sidecar protect active count 182. The active corpus has 397 cases / 361 requirements, type-elaboration 229/217, and pass/fail 213/184 without rebaselining existing expectations; broader semantics receive no credit.

## Task 241 Active Traceability

Task 241 adds one exact single-left-parenthesized builtin-set reserved-variable
inequality row with shared Chapter 04/13/14/16 references plus one dedicated
checker reference. Credit is limited to one real unrecovered `ParenthesizedTerm`,
one inner and one direct-right `x`, independent wrapper/inner/right metadata,
ordinal 1/2 `BindingId(0)` lookup, transparent reuse of one canonical reserve-
derived `BuiltinSet`, two inferred terms, two ordered expected constraints, and
one checked inequality without an independent wrapper payload. Four shared plus
one dedicated backlink, the finite matrix, all 54 prior binary-formula owners,
immutable output, focused equality regressions, and a real sidecar protect active
count 183. The active corpus has 398 cases / 362 requirements, type-elaboration
230/218, and pass/fail 214/184 without rebaselining existing expectations.
Parenthesized membership, imported/other parenthesized variants, desugaring/
truth, acceptance, proof/IR/VC, child graphs, and broader semantics receive no
Task 241 credit.

## Task 242 Active Traceability

Task 242 adds one exact single-left-parenthesized builtin-object reserved-
variable inequality row with one Chapter-03-derived builtin-type bridge plus
shared Chapter 04/13/14/16 references and one dedicated checker reference.
Credit is limited to one real unrecovered `ParenthesizedTerm`, one inner and one
direct-right `x`,
independent wrapper/inner/right metadata, ordinal 1/2 `BindingId(0)` lookup,
transparent reuse of one written-`object`-anchored canonical `BuiltinObject`,
two inferred terms, six type entries, two ordered expected constraints, and one
checked inequality without object/set coercion or an independent wrapper
payload. Five shared plus one dedicated backlink, the finite matrix, all 55
prior binary-formula owners, immutable output, focused Tasks 190/223/233/241,
and a real sidecar protect active count 184. The active corpus has 399 cases /
363 requirements, type-elaboration 231/219, and pass/fail 215/184 without
rebaselining existing expectations. Parenthesized membership and active imported
provenance receive no Task 242 credit; missing imported expansion/evidence/
signature payloads and proof/CoreIr/ControlFlowIr/VC remain deferred.

## Task 243 Active Traceability

Task 243 adds one exact single-left-parenthesized builtin-set reserved-variable
membership row with shared Chapter 04/13/14/16 references plus one dedicated
checker reference. Credit is limited to one real unrecovered `ParenthesizedTerm`,
one inner and one direct-right `x`, independent wrapper/inner/right metadata,
ordinal 1/2 `BindingId(0)` lookup, transparent reuse of one written-`set`-
anchored canonical `BuiltinSet`, two inferred terms, five type entries, no left
expected input, one right-owned expected-set constraint supplied by Task 120's
unchanged direct-right producer, and one checked membership without an
independent wrapper payload. Four shared plus one dedicated backlink, the finite
matrix, all 56 prior binary-formula owners, immutable output, focused Tasks 120/
223/233/241/242, and a real sidecar protect active count 185. The active corpus
has 400 cases / 364 requirements, type-elaboration 232/220, and pass/fail 216/184
without rebaselining existing expectations. Only this exact source discharges
the extraction gap. Object-left/set-right parenthesized membership and active
imported provenance receive no Task 243 credit; missing imported expansion/
evidence/signature payloads and proof/CoreIr/ControlFlowIr/VC remain deferred.

## Task 244 Active Traceability

Task 244 adds one exact two-reserve single-left-parenthesized heterogeneous
membership fixture. Its expectation cites the shared Chapter 04 reserved-
variable closure/source-order requirement, Chapter 13 transparent term
parenthesis requirement, Chapter 14 membership formula requirement, Chapter 16
theorem requirement, the builtin type-expression bridge, and the dedicated
`spec.en.checker.type_elaboration.parenthesized_heterogeneous_reserve_membership_source_bridge`
requirement. Therefore the fixture appears in five shared requirement lists and
one dedicated row.

The route composes the real Task 233 object `ParenthesizedTerm` producer with
Task 125's real two-binding consumer and unchanged direct-right expected-set
producer. It preserves ordered distinct written object/set ranges, ordinals 2/3,
`BindingId(0/1)`, two normalized identities, two inferred terms, five type
entries, a right-only expected constraint, and a checked membership without
facts, candidates, diagnostics, deferred work, coercion, or wrapper semantic
references. The finite helper config also preserves the five earlier
parenthesized routes.

The exact/near-miss/provenance/corruption matrix, immutable-output probes, all
57 prior owners, focused Tasks 120/125/223/233/241/242/243, real imported-gap
diagnostic preservation, and a real frontend/resolver sidecar guard the trace.
The active runner is 186; repository metadata is 401 cases / 365 requirements,
type 233/221, and pass/fail 217/184. Only this exact source discharges the gap.
Other parenthesized shapes and imported-positive provenance receive no Task 244
credit; missing imported expansion/evidence/signature payloads and proof/CoreIr/
ControlFlowIr/VC remain deferred.

## Task 245 Active Traceability

Task 245 adds one exact right-parenthesized builtin-set membership fixture. Its
expectation cites shared Chapter 04 reserved-variable, Chapter 13 transparent-
parenthesis, Chapter 14 membership, and Chapter 16 theorem requirements plus
`spec.en.checker.type_elaboration.right_parenthesized_reserved_variable_membership_source_bridge`.
Thus it has four shared backlinks and one dedicated row.

The real frontend/resolver sidecar proves the right-side wrapper producer; Task
120 supplies the real membership and expected-set consumer. The route preserves
explicit `Right` side/config identity, ordinals 1/2 to `BindingId(0)`, one
written-set identity, two inferred terms, five type entries, right-inner-only
expected ownership, and no wrapper semantic reference. The finite matrix,
Task-243 cross-route rejection, all 58 prior owners bidirectionally, and six
left-route regressions guard active runner 187. Repository metadata is 402 cases
/ 366 requirements, type 234/222, and pass/fail 218/184. Only this exact shape
receives credit; other shapes/imported-positive provenance remain uncredited and
missing imported/proof/downstream payloads remain deferred.

## Task 246 Active Traceability

Task 246 adds one exact parenthesized two-edge set-terminal local-mode equality
fixture. Five shared references (Chapters 04/13/14/16 and the Task-72 structural
producer) plus one dedicated requirement trace the closed source. The executable
contract preserves three expansions, four raw Outer inputs, ordinals 1/2 to
`BindingId(0)`, six type entries normalized to one Base-RHS `BuiltinSet`, two
ordered constraints, one checked equality, and no wrapper semantic reference.
The finite matrix, Tasks 134/223 cross-rejection, and all 59 prior owners guard
runner 188. Metadata is 403/367, type 235/223, pass/fail 219/184. Broader and
downstream behavior remains uncredited or deferred.

## Task 247 Source-Payload Ownership Traceability

Checker Task 247 adds no test or coverage backlink. It assigns the deferred
formula-statement row to checker Tasks 256-258/269-272 plus Task-10 increment
`MT10-FS`, and the deferred registration/cluster/reduction and overload rows to
Tasks 273-279 plus `MT10-AS` and their explicit accepted-status/scheme-role
gates. The capture-avoidance row is owned by Task 270 and `MT10-AS`. The
type-soundness escape/guard row is split among Tasks 258/272
(witness visibility), Task 270 (local definition guards), and Tasks
251/255/271 (sethood and `qua`).
The existing omitted-`reconsider` advanced-semantics fixture is assigned to
parser Task 47, checker Tasks 251/271-272, and `MT10-AS`; it is not moved to the
formula-statement stage.

Resolver Task 31 is the sole activation owner for the same-return member of the
exact 24-fixture reconciliation set and uses `declaration_symbol`. Task 49 later
activates the other 23 and reconciles/deduplicates all 24. The active
different-return conflict is not reactivated, while capture-avoidance,
witness/guard, and unrelated template seeds are not silently added to that set.
`spec_trace.toml` status, test lists, coverage classes, cases/requirements,
runner counts, and existing expectations remain unchanged by Task 247.

## Core Task 32 Core/CFG Ownership Traceability

Core Task 32 adds no backlink or coverage credit. It assigns Core Tasks 33-53
and prepared Task-10 consumers `MT10-CIR-TE`, `MT10-CIR-FS`, `MT10-CIR-AS`,
`MT10-CIR-ALG`, and `MT10-CFG-PV`. The exact consumer stage/tag/phase/artifact
and corruption contracts are canonical in Core
[source_family_decomposition.md](../../mizar-core/en/source_family_decomposition.md).

The broad non-Task-180 CoreIr and all ControlFlowIr rows remain deferred with
empty tests. Existing Chapter-20 parser sources and expectations stay
parse-only and are not reused as semantic baselines. The first general Core
snapshot path and first `SnapshotKind::ControlFlowIr` path must each be paired
with a distinct real semantic source and baseline in the owning descendant.
Task 32 changes no case/requirement count, runner count, status, test list,
expectation, or production source.

## VC Task 30 VC Ownership Traceability

VC Task 30 adds no backlink or coverage credit. Its exact Task-31 consumer is
`MT10-VC-T180`: a distinct `proof_verification` /
`active_proof_verification`, `expected_phase = "vc_generation"`, phase-11
source/sidecar that compares the complete `SnapshotKind::VcIr` /
`VcSet::debug_text()` bytes. The existing Task-180 type-elaboration case and
Core snapshot remain unchanged. Task 31 owns the first real runner/guard,
fixture, baseline, and exactly one trace row together: id
`spec.en.mizar_vc.vc_ir.task180_proof_verification_snapshot`, canonical source
`doc/design/mizar-vc/en/source_vc_decomposition.md`, section `VC Task 31; exact
Task-180 open VcIr proof-verification snapshot`, stage `proof_verification`,
status `covered`, required snapshot coverage, and sole backlink
`tests/miz/pass/theorems/pass_proof_verification_contradiction_formula_constant_001.expect.toml`.

Task 31 now lands that row exactly as specified. The plan contains 404 cases
and 369 requirements. Proof-verification coverage has four requirements, one
covered and three deferred; its active runner has one passing case. The exact
snapshot is the sole new credit. The broad proof-verification and algorithm
rows retain deferred status and empty tests, and the existing type-elaboration
Task-180 backlink remains unchanged.

VC Tasks 32-55 use shared `MT10-VC-PV` slices `MT10-VC-PV/VC<n>`, each with a
distinct real source, sidecar, narrow trace row, full VcIr baseline, and
corruption coverage plus every applicable task-local zero-VC/near-miss and
diagnostic negative required by its owning row. VC 40 remains blocked on
completed VC 37/39 outputs plus Core 40/A1; VC 53 remains blocked because
canonical authority does not name its evidence producer/reference schema/
authentication contract/tests. Missing scheme/theorem-role slices remain
outside direct VC 41 behind S1. The
broad VC corpus and algorithm rows stay deferred; Task 30 changes only their
owner/deferred-reason text and preserves 403/368 plus all count/hash oracles.

## Resolver R-031 Same-Return Declaration Traceability Completion

R-031 is the sole activation owner for
`spec.en.19.overload.definition_conflict.same_return.declaration`. Its only
backlink remains
`tests/miz/fail/resolve/fail_resolve_same_signature_same_return_conflict_001.expect.toml`,
which is active because the real declaration-symbol runner observes the
exact `declaration_symbol.signature.same_signature_definition_conflict` key.
The existing different-return row, sidecar, detail key, and coverage credit are
unchanged. No other member of Task 49's exact 24-fixture reconciliation set is
activated or credited. The exact row is covered and declaration-symbol
admission is five cases; plan and pass/fail counts remain 404/369 and 220/184.

## Parser Task 47 `reconsider` Trace Completion

Exactly `spec.en.15.reconsider.omitted_justification.parser` and
`spec.en.15.reconsider.proof_block.parser` change from deferred to covered.
Their exact backlink is the new
`tests/miz/pass/parser/pass_parser_reconsider_tails_001.expect.toml`; the
general Chapter-15 statement row also lists that case without adding a third
requirement. The sidecar explicitly limits credit to syntax and grants no
semantic reconsider, proof-obligation, theorem-acceptance, or E0102 coverage.
The plan becomes 405/369 with parse coverage 43/42 and pass/fail 221/184.

## Parser Task 48 Property-Implementation Trace Completion

Exactly `spec.en.07.modes.property_implementation.parser` changes from
deferred to `covered` with `coverage = "pass_and_fail"`. Its complete backlink
set is:

- `tests/miz/pass/parser/pass_parser_property_implementations_001.expect.toml`
- `tests/miz/fail/parser/fail_parser_property_implementations_recovery_001.expect.toml`

The two active cases exercise the dedicated top-level declaration, means and
equals simple/case/otherwise definientia, exact parameter restrictions,
correctness conditions, and recovery. Credit is limited to parser/syntax
ownership. Property payload extraction, overlap/coherence semantics, proof
acceptance/discharge, and the inactive semantic Task-39 seed remain deferred
and unchanged. The plan is 407/369, parse-only is 99/99, and pass/fail is
222/185; declaration/type/proof admissions remain 5/188/1.
