# Task STEP5C2-STRUCTURE-SEMANTICS: activate structure semantics

> Canonical language: English. Japanese pointer:
> [../ja/STEP5C2-STRUCTURE-SEMANTICS.md](../ja/STEP5C2-STRUCTURE-SEMANTICS.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index),
[mizar-core](../../mizar-core/en/00.crate_plan.md#task-index), and
[mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Ready: the user authorized the no-spec-change test-intent and trace repair on 2026-09-04; implementation not yet started |
| Purpose | Execute all 14 Step 5C.2 oracle pairs through source-derived checker and Core seams |
| Tier | Full: structure semantics, Core normalization, and inactive expectations change |
| Owner / consumer | `mizar-checker` owns definition, inheritance, type, constructor, selector, and update validity; `mizar-core` owns definitional term normalization; `mizar-test` only extracts authenticated syntax-free inputs, invokes, and compares |
| Dependencies | Step 5B.2 complete; no listed gap blocker |
| Classification | Authorized `test_expectation_drift` and traceability repair, followed by `source_drift` plus executable `test_gap`; no specification change |
| Coverage-audit impact | After the human decision, update the Chapter 5 and Chapter 13 rows for bounded active coverage; retain broader deferrals in both rows |

Authority is [Chapter 5](../../../spec/en/05.structures.md) and
[Chapter 13 §§13.3--13.3.3](../../../spec/en/13.term_expression.md#133-structure-expressions), followed by the
exact `.miz` sources, their 11
[trace records](../../../../tests/coverage/spec_trace.toml), expectations, and
ordered [activation map](../../../../tests/coverage/step5_activation_map.tsv).
The approved Step 5A.8 decision changes no language specification, but it does
not resolve the separate 5C.2 contradiction below. Task 254/263 transports
remain frozen prerequisites, not semantic owners reopened by this task.

## Authorized authority repair

Chapter 5 §5.3.1 permits same-root distinct renamed views, so the frozen
diamond failure source contradicted its sidecar. On 2026-09-04 the user
authorized replacing only that `.miz` with an incompatible joined-type case
while retaining its id/outcome/phase/key and leaving EN/JA spec unchanged.
The same approval moves the `with` requirement from its incorrect Chapter 5
§5.7 reference to Chapter 13 §13.3.3 across trace, sidecar, activation map, and
corpus map without changing stage, status, outcome, or behavior.

## Single-owner handoff and public API

The runner converts an unrecovered `SurfaceAst` to source-order syntax-free
records carrying range, spelling, ordinal, and available resolver `SymbolId`.
The checker authenticates source/module, unique resolver entries, symbol kinds,
ranges, and order against `SymbolEnv`; missing, extra, duplicate, reordered,
recovered, cross-module, unresolved-required, or unsupported shapes fail closed.
Spelling never substitutes for identity; unresolved mapping/selector spelling is
retained only for its frozen negative key.

`source_structure_semantics.rs` adds private-field, getter-only records under
these public entry points (public enums are `#[non_exhaustive]`):

```text
SourceStructureProgramInput::new(source_id, module_id,
  definitions, inheritances, variables, terms, claims)
SourceStructureSemanticsChecker::check(input, &SymbolEnv)
  -> Result<SourceStructureSemanticsOutput, SourceStructurePayloadError>
SourceStructureSemanticsOutput::{source_id,module_id,structures,terms,claims,diagnostics}
SourceStructureDiagnostic::{phase,source_range,detail_key}
SourceStructureDiagnosticPhase = Resolve | TypeCheck
SourceStructureType = Set | Structure { symbol, arguments }
SourceStructureTerm = Variable | Constructor | Select | Update
```

Records preserve bracket parameters, ordered field/property members, explicit
or shorthand inheritance (`set` only through explicit `it`), named constructor
arguments, selector/update occurrences, variables, one theorem proposition,
and ordered `thus` conclusions. Output collections are immutable and semantic
identities remain checker-owned.

The checker stops atomically at the first source-order error without later state
mutation. It enforces unique members, every field exactly once for no-default
fields-only constructors, parameter arity, member roles, exact base
coverage, mapping/root/path/diamond consistency, selector lookup, and field-only
update typing. Types are `set`, declared structure applications, and identity;
coercion, overloads, property implementations, proof search, and inference stay out.

`mizar-core::elaborator` adds a normalizer over diagnostic-free checker output:
constructor selection reduces to its field value and the exact single update
is applied before selection. A receipt requires the proposition and every
`thus` equality to normalize reflexively; these two cases leave zero residual
VCs. Malformed/unequal/unsupported input fails closed. Generic `CoreIr`,
Task-180 VC/snapshot behavior, and `mizar-vc` APIs remain unchanged.

## Exact activation and runner contract

| Case id | Tag / phase | Failure detail key |
|---|---|---|
| `pass_proof_verification_struct_constructor_access_001` | `active_proof_verification` / `vc_generation` | -- |
| `fail_type_elaboration_struct_constructor_missing_field_001` | `active_type_elaboration` / `type_check` | `structures.constructor.missing_field_argument` |
| `fail_type_elaboration_struct_duplicate_member_001` | `active_type_elaboration` / `resolve` | `structures.definition.duplicate_member` |
| `pass_type_elaboration_struct_definition_basic_001` | `active_type_elaboration` / `type_check` | -- |
| `pass_type_elaboration_struct_property_member_001` | `active_type_elaboration` / `type_check` | -- |
| `pass_type_elaboration_struct_dependent_bracket_params_001` | `active_type_elaboration` / `type_check` | -- |
| `fail_type_elaboration_struct_diamond_inconsistent_001` | `active_type_elaboration` / `type_check` | `structures.inherit.diamond_inconsistency` |
| `pass_type_elaboration_struct_diamond_consistent_001` | `active_type_elaboration` / `type_check` | -- |
| `pass_type_elaboration_struct_inherit_from_set_001` | `active_type_elaboration` / `type_check` | -- |
| `fail_type_elaboration_struct_inherit_uncovered_member_001` | `active_type_elaboration` / `type_check` | `structures.inherit.uncovered_base_member` |
| `fail_type_elaboration_struct_inherit_unknown_source_001` | `active_type_elaboration` / `type_check` | `structures.inherit.unknown_source_member` |
| `pass_type_elaboration_struct_inherit_rename_001` | `active_type_elaboration` / `type_check` | -- |
| `fail_type_elaboration_struct_unknown_selector_001` | `active_type_elaboration` / `type_check` | `structures.selector.unknown_field` |
| `pass_proof_verification_struct_with_update_001` | `active_proof_verification` / `vc_generation` | -- |

The type runner admits exactly these 12 rows in map order; proof admits these two
plus unchanged Task-180. Admission binds id/path/stage/phase/outcome/tag/empty
public codes/key and rejects inventory drift. Metadata cannot choose semantics;
proof reruns checker/Core and requires identical zero-VC receipts.

## Change boundary and exit

Rust scope is one checker module/tests/inventory lint, bounded private runner
extraction/admission/dispatch/count tests, and one Core normalizer/tests/lint.
Data scope is the authorized diamond `.miz`; `with` sidecar/trace/map/corpus ref;
14 tags/notes; this pair/owner links/todo; and Chapter 5/13 audit rows.
Prior Task 254/263 sections and completed contracts remain frozen.

Except for the exact authorized diamond source and `with` reference repair, do
not edit `doc/spec`, any `.miz`, expectation outcomes/phases/keys/backlinks,
trace content, activation-map content/order, public diagnostics, soundness
cases, legacy ledgers, completed addenda, or `doc/design/archive/`. Task 277B
stays not-ready/zero-credit. Independent spec/boundary, test, implementation,
and source/docs/API reviews must end without findings. Focused tests precede
workspace fmt, warnings-denied Clippy, full tests, metadata/link/ledger lint,
deterministic reruns, exact 14/120 joins, protected hashes, nine hard gates and
an independent score of at least 90/100. Exit requires an exact task-only local
commit, clean postcommit proof, and throughput `2 semantic-credit tasks/week`.
