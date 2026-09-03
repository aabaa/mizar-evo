# Task STEP5C1-VARIABLE-SEMANTICS: activate variable semantics

> Canonical language: English. Japanese pointer:
> [../ja/STEP5C1-VARIABLE-SEMANTICS.md](../ja/STEP5C1-VARIABLE-SEMANTICS.md).

Owning plans: [mizar-resolve](../../mizar-resolve/en/00.crate_plan.md#task-index), [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index), and [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Complete: hard gates 9/9; quality score 100/100; no caps |
| Purpose | Execute all 12 Step 5C.1 oracle pairs through source-derived resolver and checker seams |
| Tier | Full: production semantic APIs and previously inactive expectations change |
| Owner / consumer | `mizar-resolve::names` owns source walking and lexical resolution; checker `binding_env` maps its authenticated identities; `type_checker` alone owns type/thesis decisions; `mizar-test` only invokes and compares |
| Dependencies | Step 5A.2/G1 and Step 5B.2 complete |
| Classification | `source_drift` plus executable `test_gap`; no `spec_gap`, expectation drift, or metadata conflict |
| Coverage-audit impact | Update only the Chapter 4 row for active source-to-semantics coverage and retain broader deferrals |

Authority is [Chapter 4](../../../spec/en/04.variables_and_constants.md), [§3.5](../../../spec/en/03.type_system.md#35-subtyping-and-widening), [§§15.2/15.4/15.5/15.11](../../../spec/en/15.statements.md), and [§16.3](../../../spec/en/16.theorems_and_proofs.md#163-proof-skeleton-and-natural-deduction), followed by the exact tests, their 10 [trace records](../../../../tests/coverage/spec_trace.toml), and the ordered [activation map](../../../../tests/coverage/step5_activation_map.tsv).

## Single-owner handoff and public API

This task supersedes only the still-deferred semantic activation assigned to
Tasks 269--272/`MT10-FS`/`MT10-AS`; their completed zero-credit transports and
historical contracts stay frozen. Resolver source walking, not the runner,
derives every spelling, scope, ordinal, node/range, binding/reference identity,
formal, arity, body link, and captured binding from an unrecovered `SurfaceAst`.
It authenticates `ast.source_id`, `ModuleId`, `SymbolEnv::module_id`, opaque node
ids, ranges, and parent/child shape before credit. Checker `BindingEnv` remains
the sole checker-local identity/context projection and never repeats name lookup.

`names.rs` adds the following exact public seam. Struct fields are private;
dense ids expose only `index`, records expose read-only getters, public enums are
`#[non_exhaustive]`, and returned slices/maps are immutable.

```text
SourceVariableScopeInput<'a>::new(&SurfaceAst, &'a ModuleId, &'a SymbolEnv)
SourceVariableScopeResolver::resolve(input) -> Result<ResolvedVariableScope, SourceVariableScopeError>
ResolvedVariableScope::{source_id,module_id,bindings,references,thesis,statements}
SourceVariableBindingKind = Reserve | Quantifier | Let | Set | Reconsider | InlineFunctor | InlinePredicate | InlineParameter
SourceVariableReferenceKind = Term | InlineFunctor { arity } | InlinePredicate { arity }
SourceVariableBinding = { id, kind, spelling, node, range, scope, ordinal, declared_type, arity, captures }
SourceVariableReference = { id, kind, spelling, node, range, scope, ordinal, binding }
SourceVariableTypeRadix = Set | Object
SourceVariableType = { radix: SourceVariableTypeRadix, attributes: ordered spellings }
SourceVariableTerm = Binding { node, range, reference } | InlineFunctor { node, range, definition_reference, arguments }
SourceVariableFormula = Equality { node, range, left, right } | TypeAssertion { node, range, term, target } | InlinePredicate { node, range, definition_reference, arguments } | ForAll { node, range, binding, condition, body } | Exists { node, range, binding, body }
SourceVariableStatement = Let { node, range, binding, condition } | Set { node, range, binding, value } | Reconsider { node, range, binding, value, target, justified } | DefineFunctor { node, range, binding, formals, result, body } | DefinePredicate { node, range, binding, formals, body } | Assert { node, range, formula, conclusion } | Take { node, range, witness, existential_binding }
SourceVariableScopeError::{SourceMismatch,ModuleMismatch,RecoveredSyntax,InvalidShape,DuplicateLocalConstant,ForwardReference,UnreservedImplicitVariable,UnresolvedReference,ArityMismatch}
```

Here `node` is `SurfaceNodeId`, `range` is `SourceRange`, `scope` is `LocalTermScope`, owned text is `String`, ids/arity/ordinal are dense ids or `usize`, optional fields use `Option`, and ordered fields use `Vec`. Same-named getters return copied scalar/id/range values, `&str`, `&T`, or slices. Recursive term/formula children are boxed; output `binding_types` is `&BTreeMap<SourceVariableBindingId, SourceVariableType>` and other output collections are slices.

The resolver validates source/module/recovery/shape first, then declarations in
source order, then references in `(ordinal, range)` order; it returns the first
error. `detail_key()` maps the three semantic errors to
`variables.local_constant.duplicate_identifier`,
`variables.local_constant.forward_reference`, and
`variables.reserve.unreserved_implicit_variable`; structural errors cannot be
mistaken for an expected oracle key. Bound body references, explicit capture ids,
and ordered one-to-one formal/argument ids make inline substitution identity-based
and capture-safe; missing, recovered, invalid cross-scope provenance, forward,
duplicate, wrong-arity, or reordered payload fails closed before substitution.
Because the receipt exposes one optional thesis, it admits at most one
`TheoremItem`; multiple theorem transactions fail with `InvalidShape` rather
than being combined.
Only the exact `deffunc`/`defpred` fixture transfers capture execution from the
old deferral; Task 270's distinct shadowing fixture and broader `MT10-AS` capture
work remain deferred and separately owned.

`type_checker.rs` adds private-field `SourceVariableSemanticsInput::new(&ResolvedVariableScope)`,
unit `SourceVariableSemanticsChecker::check`, and immutable
`SourceVariableSemanticsOutput::{source_id,module_id,binding_types,assumptions,thesis,diagnostics}`.
`SourceVariableSemanticsDiagnostic::{source_range,detail_key}` carries only the
three checker keys `variables.reconsider.unjustified_narrowing`,
`variables.let.duplicate_generalization`, and
`variables.take.non_existential_thesis`. Statements run in source order and stop
state mutation after the first diagnostic. Explicit types override reservations;
only builtin set-to-object widening, declared facts, reflexivity, and
identity-based definitional reduction discharge checks. There is no proof search
or theorem acceptance.

## Exact activation and runner contract

| Case id | Tag / phase | Failure detail key |
|---|---|---|
| `pass_formula_statement_deffunc_defpred_local_001` | `active_formula_statement` / `statement_check` | -- |
| `pass_formula_statement_let_such_that_assumption_001` | `active_formula_statement` / `statement_check` | -- |
| `pass_formula_statement_set_local_constant_take_001` | `active_formula_statement` / `statement_check` | -- |
| `pass_formula_statement_reconsider_builtin_widening_001` | `active_formula_statement` / `statement_check` | -- |
| `fail_formula_statement_duplicate_generalization_001` | `active_formula_statement` / `statement_check` | `variables.let.duplicate_generalization` |
| `fail_formula_statement_take_non_existential_thesis_001` | `active_formula_statement` / `statement_check` | `variables.take.non_existential_thesis` |
| `pass_type_elaboration_reserve_shadow_explicit_type_001` | `active_type_elaboration` / `type_check` | -- |
| `pass_type_elaboration_reserve_implicit_typing_001` | `active_type_elaboration` / `type_check` | -- |
| `fail_type_elaboration_reconsider_unjustified_narrowing_001` | `active_type_elaboration` / `type_check` | `variables.reconsider.unjustified_narrowing` |
| `fail_type_elaboration_set_duplicate_local_constant_001` | `active_type_elaboration` / `resolve` | `variables.local_constant.duplicate_identifier` |
| `fail_type_elaboration_set_forward_reference_001` | `active_type_elaboration` / `resolve` | `variables.local_constant.forward_reference` |
| `fail_type_elaboration_unreserved_implicit_variable_001` | `active_type_elaboration` / `resolve` | `variables.reserve.unreserved_implicit_variable` |

The new CLI spelling is `mizar-test formula-statement`; its public API is
`run_formula_statement_corpus`, `active_formula_statement_cases`, and report,
case-result, and `#[non_exhaustive]` status types parallel to type elaboration.
Admission accepts only `.miz` pass/fail formula cases at
`formula_statement/statement_check`, or type cases at
`type_elaboration/type_check` plus fail-only `type_elaboration/resolve`; public
diagnostic codes stay empty. The dedicated variable route runs before generic
type dispatch and preserves bare resolver keys instead of adding the existing
`type_elaboration.lower_stage.` prefix. All other tag/stage/phase combinations
fail closed. Metadata-only exact `(case id, workspace-relative source)` lists
contain the six formula and six variable-route rows above; missing, extra, or
duplicate membership is rejected and can never choose a semantic result.
Pre-existing non-5C.1 type cases retain their old admission; the new `resolve`
phase admission is limited to the three exact fail rows above.

## Exact change boundary and exit

Rust allowlist: `names.rs`, `names/tests.rs`, and the mandatory R-026 public-enum
rows in `mizar-resolve/tests/lint_policy.rs`; `type_checker.rs` plus
its `tests/support/source_variable_semantics_unit.rs` unit-test include and
`checker_source_inventory.tsv`; new runner `formula_statement.rs` and test,
`runner.rs`, `type_elaboration/admission.rs`, `lib.rs`, `main.rs`, and mandatory
count/public-enum rows in `mizar-test/tests/{metadata,lint_policy}.rs`, plus the
count-only isolation assertions in `runner/tests/type_elaboration/{source_attribute_definition,source_functor_definition,source_mode_definition,source_property_implementation,source_statement}.rs`.
Test-data
allowlist is the 12 sidecars above, where only the exact tag and stale inactive
note wording change. Documentation allowlist is this pair; the paired crate
plans and `names.md`/`type_checker.md`/`harness.md`; the paired mizar-test TODO;
`todo.md`; and the Chapter 4
audit row. Compacted or historical Task-269--272/`MT10-*` text stays untouched;
this live contract is its chronological ownership override. No other file may
change.

Do not edit `doc/spec`, any `.miz`, expectation outcomes/phases/keys/backlinks,
trace content, activation map, soundness cases, completed contracts/addenda,
legacy ledgers, or `doc/design/archive/`. Task 277B remains not-ready/zero-credit.
Trace already says `covered`; active execution is evidenced only by the tags and
runners.

## Completion evidence

The source-derived route executes all 12 frozen oracles: formula-statement is
6/6 and type-elaboration is 211/211 overall, including the six variable rows.
Resolver and checker suites are 184/184 and 587/587; the mizar-test library is
664/664. Syntax smoke remains 360 cases with 353 passes, seven expected syntax
rejections, and no failures. Workspace fmt, Clippy with denied warnings, and
all tests pass, so all nine hard gates pass. The
resolver owns authenticated identities and structural failures; the checker
alone owns type/thesis state and the three frozen semantic keys. No `doc/spec`,
`.miz`, trace, activation-map, archive, or public diagnostic-code change is
part of this activation. Archive inventory remains 13 files and Task 277B
remains not-ready/zero-credit. Independent read-only scoring awarded 100/100
with no caps; broader `given`/`consider`, nested capture matrices, theorem
acceptance, and Core/VC semantics remain owned by later Step 5C tasks.

Independent spec/boundary, test-sufficiency, implementation, and source/docs/API
reviews must end without findings. Focused crates/stages, syntax smoke, metadata,
lint/link/ledger, workspace fmt/Clippy/tests, deterministic reruns, exact 12/120
joins, and protected hashes must pass. Exit requires all 12 oracles, nine hard
gates, score at least 90/100, exact task-only commit, clean postcommit proof, and
semantic throughput `1 task/week`. Fresh inventory then selects Step 5C.2.
