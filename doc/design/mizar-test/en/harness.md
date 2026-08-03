# Module: harness

## Parser Task 46 Operator-Declaration Parse-Only Increment

The exact pass/fail pair is admitted by the ordinary parse-only runner. The
pass sidecar requires zero diagnostics; the fail sidecar pins six existing
syntax diagnostic codes. Parser unit tests, rather than the code-only fail
sidecar, pin every slot/delimiter diagnostic message/range and preserve the
definition's outer `end;` plus the following theorem. No new runner phase,
diagnostic vocabulary, or production harness path is introduced.

> Canonical language: English. Japanese companion: [../ja/harness.md](../ja/harness.md).

## Purpose

This module defines the test harness that discovers cases, runs compiler profiles, checks expectations, and reports deterministic results.

## Public API

```rust
pub struct DiscoveryConfig {
    pub workspace_root: PathBuf,
    pub tests_root: PathBuf,
    pub manifest_path: PathBuf,
    pub profile: TestProfile,
    pub validation_mode: ValidationMode,
}

pub struct TestPlan {
    pub cases: Vec<TestCase>,
    pub manifest: TraceManifest,
    pub coverage_report: CoverageReport,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub struct CoverageReport {
    pub requirements: Vec<RequirementCoverage>,
    pub stages: Vec<StageCoverage>,
    pub pass_fail_mix: PassFailMix,
    pub architecture22_matrix: Architecture22MatrixReport,
}

pub struct RequirementCoverage {
    pub id: SpecRequirementId,
    pub stage: Stage,
    pub coverage: CoverageShape,
    pub required: bool,
    pub stored_status: RequirementStatus,
    pub computed_status: RequirementStatus,
    pub evidence: CoverageEvidenceSummary,
    pub missing_shapes: Vec<CoverageShape>,
}

pub struct StageCoverage {
    pub stage: Stage,
    pub requirements: usize,
    pub covered: usize,
    pub partial: usize,
    pub planned: usize,
    pub deferred: usize,
    pub obsolete: usize,
    pub missing_shapes: usize,
}

pub struct PassFailMix {
    pub pass: usize,
    pub fail: usize,
    pub total: usize,
    pub target_pass_percent: u8,
    pub target_fail_percent: u8,
}

pub struct Architecture22MatrixReport {
    pub scenarios: Vec<Architecture22ScenarioReport>,
    pub missing_scenarios: Vec<String>,
}

pub struct Architecture22ScenarioReport {
    pub scenario_id: String,
    pub equivalence_class: String,
    pub planned: usize,
    pub active: usize,
}

pub struct TestCase {
    pub id: TestCaseId,
    pub source_path: PathBuf,
    pub expectation_path: PathBuf,
    pub expectation: Expectation,
}

#[non_exhaustive]
pub enum TestProfile {
    Fast,
    Full,
    Stress,
    FuzzRegression,
    SnapshotUpdate,
}

#[non_exhaustive]
pub enum ValidationMode {
    Metadata,
    Development,
    Release,
}

pub struct ParseOnlyRunReport {
    pub results: Vec<ParseOnlyCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub struct DeclarationSymbolRunReport {
    pub results: Vec<DeclarationSymbolCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub struct TypeElaborationRunReport {
    pub results: Vec<TypeElaborationCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub struct TypeElaborationCaseResult {
    pub id: TestCaseId,
    pub expectation_path: PathBuf,
    pub status: TypeElaborationCaseStatus,
    pub actual_detail_keys: Vec<String>,
    pub snapshot_failure: Option<String>,
}

pub struct ProofVerificationRunReport {
    pub results: Vec<ProofVerificationCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub struct ProofVerificationCaseResult {
    pub id: TestCaseId,
    pub expectation_path: PathBuf,
    pub status: ProofVerificationCaseStatus,
    pub failure: Option<String>,
}
```

The generic `TestOutcome`/snapshot-reporting surface is future API. Current
active runners expose stage-specific report records while sharing the metadata
plan and validation diagnostics shown above.

## Public Enum Forward Compatibility

Task 12 applies the `mizar-frontend` task-25 procedure to the harness-facing
enum surface. These enums are downstream API and must remain
`#[non_exhaustive]`; downstream callers must keep wildcard match arms, while
`mizar-test` may keep crate-internal matches exhaustive for the currently known
variants.

| Public enum | Owner | Decision |
|---|---|---|
| `ValidationSeverity` | `diagnostic` reporting used by harness plans and runner reports | `#[non_exhaustive]` downstream forward-compatible surface. |
| `TestProfile` | `harness` profile selection | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ValidationMode` | `harness` validation strictness | `#[non_exhaustive]` downstream forward-compatible surface. |
| `HarnessError` | `harness` infrastructure failure boundary | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ParseOnlyCaseStatus` | `runner` parse-only report status | `#[non_exhaustive]` downstream forward-compatible surface. |
| `DeclarationSymbolCaseStatus` | `runner` declaration-symbol report status | `#[non_exhaustive]` downstream forward-compatible surface. |
| `TypeElaborationCaseStatus` | `runner` type-elaboration report status | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ProofVerificationCaseStatus` | `runner` exact proof-verification report status | `#[non_exhaustive]` downstream forward-compatible surface. |

No exhaustive public enum exceptions are owned by this module.

## Runner Modes

| Mode | Behavior |
|---|---|
| metadata plan | discover sidecars and validate layout, expectation schema, and traceability without executing payloads |
| parse-only | run active `.miz` parse-only cases through `mizar-frontend` and `MizarParserSeam` |
| declaration-symbol | run active `.miz` declaration-symbol cases through frontend parsing and resolver declaration/symbol collection |
| type-elaboration | run active `.miz` type-elaboration cases through frontend parsing and resolver declaration/symbol collection, extract supported reserve-only declaration payloads, delegate checker-owned `BindingEnv`/`DeclarationInput`/`DeclarationChecker` handoff production to the syntax-free `mizar-checker` seam, continue successful bare-builtin, task-55 bare local-mode-expansion, task-56 one-edge local-mode chain, and task-74 structural bare local-mode chain cases through `TypedAst` and `ResolvedTypedAst`, confirm `mizar-core` summary-readiness through `ResolvedTypedAstSummary::from_ast`, prepare binder-only `CoreContext` input from the same reserve bindings, surface same-module attributed reserve declarations, local structure reserve heads, attributed local structure reserve heads, task-57 real local-mode expansions with local structure RHSs, task-58 real local-mode expansions with attributed builtin RHSs, task-59 attributed local-mode reserve heads with real direct bare-builtin expansions, task-60 attributed local-mode reserve heads with real direct local-structure RHS expansions, task-61 attributed local-mode reserve heads with real direct attributed-builtin RHS expansions, task-62 one-edge bare local-mode chains ending in local structure RHSs, task-63 one-edge bare local-mode chains ending in attributed builtin RHSs, task-64 attributed local-mode reserve heads with one-edge bare-builtin chains, task-65 attributed local-mode reserve heads with one-edge structure-RHS chains, and task-66 attributed local-mode reserve heads with one-edge attributed-builtin-RHS chains as checker evidence-query gaps, surface same-module local mode reserve heads that lack the narrow task-55/task-56/task-57/task-58/task-59/task-60/task-61/task-62/task-63/task-64/task-65/task-66/task-74 expansion slices, including mixed attributed/bare local-mode sources, attributed chain dependencies, and chains that violate task-74 structural guards, as checker mode-expansion payload gaps, surface task-67 structure-qualified attribute references, task-68 argument-bearing local-mode reserve heads, task-69 argument-bearing local-structure reserve heads, task-70 bracket-form local-mode reserve heads, and task-71 bracket-form local-structure reserve heads as source-to-checker extraction-gap boundary cases, surface task-75 forward local-mode reserve heads, task-76 forward local-structure reserve heads, and task-77 forward local-attribute reserve type expressions as lower-stage active-range boundary cases before checker handoff, and surface unsupported checker payload families as stable external dependency gaps |
| proof-verification | run only the exact Task-180 active proof-verification source through source-to-checker-to-Core-to-VC twice and compare the complete `VcSet` debug baseline; broader proof-verification families remain deferred |
| pass/fail | run `.miz` cases and match expected outcome |
| snapshot | compare canonical snapshot hashes |
| determinism | repeat runs and compare artifacts, diagnostics, and hashes |
| parallel-equivalence | compare sequential and parallel outputs |
| fuzz-regression | run minimized fuzz cases as ordinary committed tests |
| update | rewrite snapshots only when explicitly requested |

Core Task 31 adds one exact type-elaboration exception: after the Task-180
checker handoff succeeds, the runner lowers that bundle to CoreIr twice and
verify-compares its complete debug bytes with the committed baseline. A
missing, unreadable, mismatched, or absent CoreIr snapshot sets the public case
status to `Failed`, populates `snapshot_failure`, and emits internal diagnostic
code `E-TYPE-ELABORATION-SNAPSHOT` at
`type_elaboration.snapshot.<case-id>`. The ordinary detail-key result remains
unchanged, and no other type-elaboration case enters this path.

## Runner Source Ownership (Checker Task 250 Update)

The current production runner layout contains exactly 21 paths and 23,184
lines. Checker Task 250 adds one bounded source-attribute leaf beside the
existing Task-248 source-context and Task-249 source-type leaves while keeping
`runner.rs` limited to facade/top-level orchestration.

| Production path | Lines | Ownership |
|---|---:|---|
| `src/runner.rs` | 2,390 | Public reports/statuses including snapshot failure, corpus orchestration, public active iterators, proof-verification orchestration, parse/declaration admission, type-case execution, verify-only baseline comparison, and top-level detail dispatch. |
| `src/runner/shared.rs` | 265 | Cross-phase source/frontend/resolver staging and common diagnostic support, including exact internal resolver diagnostic-key projection and resolver shell retention. |
| `src/runner/parse_only.rs` | 119 | Parse-only case execution and failure projection. |
| `src/runner/declaration_symbol.rs` | 231 | Declaration-symbol execution, observation, payload, and failure projection. |
| `src/runner/import_fixtures.rs` | 410 | Fixture lexical summaries and import-summary adapters, including coherent resolver import projection for source-type authentication. |
| `src/runner/proof_verification.rs` | 170 | Exact Task-180 admission, source-to-VC execution, deterministic rerun, VcIr snapshot comparison, and failure diagnostics. |
| `src/runner/type_elaboration.rs` | 593 | Private type-elaboration facade over exactly fourteen private leaves. |
| `src/runner/type_elaboration/admission.rs` | 60 | Active type-case admission and tag validation. |
| `src/runner/type_elaboration/binary_routes.rs` | 3,791 | Reserved-variable binary route configs, extraction, output, and details. |
| `src/runner/type_elaboration/checker_handoff.rs` | 1,299 | Checker-owned binding/declaration plus exact Task-180 statement/proof/terminal handoff assembly, validation, legacy empty-later-payload assembly, and test-only real-bundle near-miss construction. |
| `src/runner/type_elaboration/long_chain_config.rs` | 82 | Shared exact long-chain definition tables. |
| `src/runner/type_elaboration/output.rs` | 1,571 | Checker outputs, validation, result/detail projection, diagnostics, and reusable exact Task-180 CoreIr construction plus deterministic Core rerun. |
| `src/runner/type_elaboration/parenthesized_routes.rs` | 745 | Parenthesized reserved-variable route ownership. |
| `src/runner/type_elaboration/result.rs` | 38 | Expected-key plus stable detail/snapshot failure projection. |
| `src/runner/type_elaboration/source_ast.rs` | 147 | Common exact AST and import projection. |
| `src/runner/type_elaboration/source_attribute.rs` | 1,575 | Exact Task-250 attribute-chain AST traversal, syntax-free chain/attribute/qualifier/group/actual projection, checker producer invocation, and pending-detail isolation. |
| `src/runner/type_elaboration/source_context.rs` | 592 | Exact Task-248 resolver-shell/source-context projection, route isolation, checker producer invocation, immutable handoff assembly, exact 2/2/0 source-type dependency co-installation, and explicit absence of a Task-250 source-attribute payload. |
| `src/runner/type_elaboration/source_formula.rs` | 2,651 | Common formula/source payload extraction, including exact theorem/formula sites/ranges and explicit Task-268 theorem intent. |
| `src/runner/type_elaboration/source_reserve.rs` | 1,474 | Reserve declaration, type, symbol, and mode-expansion extraction. |
| `src/runner/type_elaboration/source_type.rs` | 794 | Exact Task-249 source-type AST traversal, syntax-free 10/13/6 checker input projection, handoff assembly, and pending-detail isolation. |
| `src/runner/type_elaboration/type_assertion_routes.rs` | 4,187 | Reserved-variable type-assertion and asserted-head route ownership. |

For hashing, prefix every displayed path with `crates/mizar-test/`. From the
repository root, the exact input is the sorted tracked path list selected from
`crates/mizar-test/src/runner.rs` and `crates/mizar-test/src/runner`, excluding
`tests.rs` and every path below `tests/`. Its newline-delimited path-list hash is
`bd42d60f45e40526a785a6ebcc0df910b99f33a8a8b19371f678070b51bac1d6`.
Passing those same repository-relative paths in order to `sha256sum` and
hashing the corresponding ordered output lines yields
`d1421834a7c7613150634735c47aa2700ddf17a7ca2ffebd94f596664ee3a8eb`.
Production `runner.rs` owns no route config, source extractor, output builder,
or detail-wrapper definition; its route aliases remain test-only. The private
type-elaboration facade's fourteen `mod` declarations, the 21-path/hash pair,
the documented public API, and the exact discovered-test/CLI oracles are the
ownership guards. Test
sources remain under `src/runner/tests.rs`, `src/runner/tests/`, and existing
integration-test files so fully qualified names and nesting do not change.

Task 75/76/77 addendum for `type-elaboration`: forward same-module local-mode
reserve heads, local-structure reserve heads, and local-attribute reserve type
expressions that name later declarations are active lower-stage boundary cases.
The runner expects
`type_elaboration.lower_stage.frontend:malformed_type_expression` before
checker handoff and must not synthesize checker `ModeExpansion`, structure
type-head, base-shape, constructor-witness, `AttributeInput`, or
attributed-type evidence payloads from future declarations.

Task 78 addendum for `type-elaboration`: before task 83, the documented
`parser.type_fixtures` imported structure `R` reserve head was an active
source-to-checker extraction-gap boundary case. Task 83 supersedes that
documented `R` portion, and task 97 supersedes the documented
`TypeCaseStruct` portion. Broader imported structures outside the task-83 `R`
and task-97 `TypeCaseStruct` provenance/type-head bridges remain deferred; such
future cases should expect
`type_elaboration.external_dependency.ast_payload_extraction`. The runner must
not treat the summary as real imported module AST extraction or synthesize
base-shape or constructor-witness evidence, positive structure elaboration,
CoreIr, ControlFlowIr, VC, or proof payloads.

Tasks 83 and 97 addendum for `type-elaboration`: the documented
`parser.type_fixtures` imported structures `R` and `TypeCaseStruct` may be
passed as checker-owned imported structure type heads. The runner expects
`type_elaboration.checker.checker.declaration.deferred.evidence_query` and must
not treat the summary as imported module AST extraction or synthesize
base-shape/constructor-witness evidence, positive imported structure
elaboration, CoreIr, ControlFlowIr, VC, or proof payloads.

Task 79 addendum for `type-elaboration`: before task 82, an imported mode
reserve head from the documented `parser.type_fixtures` import summary was an
active source-to-checker extraction-gap boundary case. Imported modes outside
the task-82 `TypeCaseMode` provenance/type-head bridge still expect
`type_elaboration.external_dependency.ast_payload_extraction`. The runner must
not treat the summary as real imported module AST extraction or synthesize
`ModeExpansion` payloads, positive mode elaboration, CoreIr, ControlFlowIr, VC,
or proof payloads.

Task 80 addendum for `type-elaboration`: before tasks 84, 85, and 116, imported
attribute reserve types from the documented `parser.type_fixtures` import
summary were active source-to-checker extraction-gap boundary cases. Imported
attributes outside the task-84 `TypeCaseAttr` provenance/`AttributeInput`
bridge, task-85 negative `empty`/builtin-`set` bridge, and task-116 positive
`empty`/builtin-`set` bridge still expect
`type_elaboration.external_dependency.ast_payload_extraction`. The
runner must not treat the summary as real imported module AST extraction or
synthesize attributed-type evidence, positive attributed type elaboration,
CoreIr, ControlFlowIr, VC, or proof payloads.

Task 84 addendum for `type-elaboration`: the documented
`parser.type_fixtures` imported attribute `TypeCaseAttr` may be passed as a
checker-owned imported `AttributeInput` on builtin `set`. The runner expects
`type_elaboration.checker.checker.declaration.deferred.evidence_query` and must
not treat the summary as imported module AST extraction or synthesize
attributed-type existential/evidence payloads, positive imported attributed
type elaboration, generic imported attributes such as `empty`,
structure-qualified attribute owner provenance, attribute arguments, CoreIr,
ControlFlowIr, VC, or proof payloads.

Task 85 / task 116 / task 171 addendum for `type-elaboration`: the documented
`parser.type_fixtures` imported attribute `empty` may be passed as a
checker-owned imported `AttributeInput` only for negative/positive builtin
`set` on the existing `non empty set` / `empty set` fixtures and negative
builtin `object` on the existing `non empty object` fixture. The runner expects
`type_elaboration.checker.checker.declaration.deferred.evidence_query` and must
not treat the summary as imported module AST extraction or synthesize
attributed-type existential/evidence payloads, positive `empty object`, imported
`empty` on symbol heads, positive imported attributed type elaboration,
structure-qualified attribute owner provenance, attribute arguments, CoreIr,
ControlFlowIr, VC, or proof payloads.

Task 86 / task 115 / task 117 addendum for `type-elaboration`: formula-only theorem
sources may run through parser and resolver as active checker boundary cases.
Task 115 supersedes only the exact unrecovered
`theorem FormulaPayloadBoundary: thesis;` source by passing the source-derived
`thesis` formula constant site/range as a checker recovery `FormulaInput`.
Task 117 supersedes that recovery marker by passing the same source-derived
site/range as a `FormulaKind::Thesis` payload and expecting only
`type_elaboration.checker.checker.formula.external.formula_payload`. Non-exact
formula-only theorem shapes remain on
`type_elaboration.external_dependency.ast_payload_extraction`. The runner must
not synthesize formula constant semantics, child-formula graph payloads,
theorem acceptance, recorded facts, proof skeletons, `formula_statement`
execution, CoreIr, ControlFlowIr, VC, or proof payloads.

Task 106 addendum for `type-elaboration`: the task-87 term-bearing builtin
equality theorem source may now run through parser, resolver, and the checker
term/formula payload seam, but only for the exact unrecovered
`TheoremItem -> FormulaExpression -> BuiltinPredicateApplication("=")` shape
with the labelled source `theorem TermFormulaPayloadBoundary: 1 = 1;` and two
structural numeral operands spelling `1`. The runner must build a real
module-shell binding context, pass source-derived checker
`TermInput`/`FormulaInput` payloads, and fail closed on
`type_elaboration.checker.checker.term.external.numeric_type_payload` and
`type_elaboration.checker.checker.formula.term.partial`. It must not synthesize
numeric type payloads, equality facts/checking, theorem acceptance, proof
skeletons, `formula_statement` runner support, CoreIr, ControlFlowIr, VC, or
proof payloads.

Task 98 addendum for `type-elaboration`: a theorem formula using imported
predicate/functor surfaces from `parser.type_fixtures` may run through parser
and resolver as an active checker boundary case, but it must remain on
`type_elaboration.external_dependency.ast_payload_extraction` until imported
predicate/functor semantic payloads, term/formula payload extraction, term
inference, formula checking, recorded facts, theorem acceptance, and
`formula_statement` runner support exist. The runner must not synthesize
imported semantic payloads, term payloads, formula payloads, proof skeletons,
CoreIr, ControlFlowIr, VC, or proof payloads.

Task 100 addendum for `type-elaboration`: the builtin membership theorem source
may run through parser, resolver, and, as of task 108, the checker term/formula
payload seam, but only for the exact unrecovered
`TheoremItem -> FormulaExpression -> BuiltinPredicateApplication("in")` shape
with the labelled source `theorem BuiltinMembershipPayloadBoundary: 1 in 1;`
and structural numeral operands spelling `1` and `1`. The runner must build a
real module-shell binding context, pass source-derived checker
`TermInput`/`FormulaInput` payloads, and fail closed on
`type_elaboration.checker.checker.term.external.numeric_type_payload` and
`type_elaboration.checker.checker.formula.term.partial`. It must not synthesize
numeric type payloads, membership operand expected types, membership facts,
theorem acceptance, proof skeletons, `formula_statement` runner support,
CoreIr, ControlFlowIr, VC, or proof payloads.

Task 107 addendum for `type-elaboration`: the task-101 builtin inequality
theorem source may now run through parser, resolver, and the checker
term/formula payload seam, but only for the exact unrecovered
`TheoremItem -> FormulaExpression -> BuiltinPredicateApplication("<>")` shape
with the labelled source `theorem BuiltinInequalityPayloadBoundary: 1 <> 2;`
and structural numeral operands spelling `1` and `2`. The runner must build a
real module-shell binding context, pass source-derived checker
`TermInput`/`FormulaInput` payloads, and fail closed on
`type_elaboration.checker.checker.term.external.numeric_type_payload` and
`type_elaboration.checker.checker.formula.term.partial`. It must not synthesize
numeric type payloads, inequality desugaring/equality checking, facts, theorem
acceptance, proof skeletons, `formula_statement` runner support, CoreIr,
ControlFlowIr, VC, or proof payloads.

Task 109 addendum for `type-elaboration`: the exact builtin type-assertion
theorem source previously covered by task 102 may now pass real source-derived
checker `TermInput`, `FormulaInput`, and asserted builtin `set`
`TypeExpressionInput` payloads before failing closed on missing numeric type
payloads and partial formula checking. Broader asserted type payloads,
type-assertion semantic checking, recorded facts, theorem acceptance,
`formula_statement`, CoreIr, ControlFlowIr, VC, and proof payloads remain
deferred.

Task 113 addendum for `type-elaboration`: the exact theorem formula importing
`parser.type_fixtures` and using its documented `empty` attribute in
`ImportedAttributeAssertionPayloadBoundary: 1 is empty` may validate imported
attribute provenance, pass source-derived numeral and attribute-assertion
checker payloads, and fail closed on missing numeric type payload, missing
formula/attribute semantic payload, and partial formula checking. The runner
must not synthesize imported module AST extraction, attribute-chain semantic
payloads, theorem-formula `AttributeInput` payloads, attribute checking,
theorem acceptance, `formula_statement`, CoreIr, ControlFlowIr, VC, or proof
payloads, and it must leave broader imported attribute assertion surfaces on
the existing gap.

Task 114 addendum for `type-elaboration`: the exact theorem formula importing
`parser.type_fixtures` and using its documented `empty` attribute as the
attribute-level `non empty` assertion in the Chapter 14 attribute-assertion
form with a Chapter 13 numeral subject supersedes task 104. The active runner
validates the direct `non` surface and imported `empty` provenance, passes real
source-derived checker term/formula payloads, and fails closed on missing
numeric type payload, missing formula/attribute semantic payload, and partial
formula checking. The runner must not synthesize imported module AST
extraction, negated attribute-chain semantic payloads, theorem-formula
`AttributeInput` payloads, negated attribute admissibility/semantic checking,
theorem acceptance, `formula_statement`, CoreIr, ControlFlowIr, VC, or proof
payloads, and it must leave non-exact attribute-level non-empty assertion
surfaces on the existing gap.

Task 111 addendum for `type-elaboration`: the exact theorem formula
`SetEnumerationPayloadBoundary: {1, 2} = {1, 2}` may run through parser and
resolver, then the active runner passes source-derived checker payloads for the
four numeral item terms, two set-enumeration terms, and builtin equality
formula. It must fail closed on missing numeric type payloads, missing
set-enumeration result-type payloads, and partial formula checking
until real set-enumeration result types, term inference, equality/formula
checking, recorded facts, theorem acceptance, and `formula_statement` runner
support exist. The runner must not synthesize result payloads, theorem
acceptance, CoreIr, ControlFlowIr, VC, or proof payloads. Chapter 13 sethood
requirements belong to set-comprehension generator domains, not enumeration.

Task 112 / task 117 addendum for `type-elaboration`: the exact theorem formula
using Chapter 14 implication, universal quantification, negation, and
`contradiction` constants may run through parser and resolver, then the active
runner passes source-derived checker `FormulaInput` shells for the implication,
quantified formula, and negation, plus exact `FormulaKind::Contradiction`
payloads for the two constants. It must fail closed on missing formula payloads
and missing quantifier payloads until formula constant semantics, child-formula
graph payloads, binder/context payloads, formula checking, recorded facts,
theorem acceptance, and `formula_statement` runner support exist. The runner
must not synthesize constant semantics, child links, binder/context payloads,
facts, theorem acceptance, CoreIr, ControlFlowIr, VC, or proof payloads.

Task 88 addendum for `type-elaboration`: a proof-block theorem source may run
through parser and resolver as an active checker boundary case, but it must
remain on `type_elaboration.external_dependency.ast_payload_extraction` until
real proof skeleton payload extraction, local proof context, formula payload
extraction, recorded facts, theorem acceptance, and `formula_statement` runner
support exist. The runner must not synthesize proof skeleton payloads, formula
payloads, local facts, theorem acceptance, CoreIr, ControlFlowIr, VC, or proof
payloads.

Task 89 addendum for `type-elaboration`: a theorem proof containing
statement-level proof justifications may run through parser and resolver as an
active checker boundary case, but it must remain on
`type_elaboration.external_dependency.ast_payload_extraction` until real
statement proof payload extraction, nested proof skeleton payloads, local proof
context, formula payload extraction, label-reference semantic checking,
recorded facts, theorem acceptance, and `formula_statement` runner support
exist. The runner must not synthesize statement proof payloads, proof skeleton
payloads, formula payloads, local facts, theorem acceptance, CoreIr,
ControlFlowIr, VC, or proof payloads.

## Consumer Runner Pacing

Task 10 keeps runner support synchronized with consumer crates one increment at
a time. Prepared increments are implemented and verified; unprepared consumers
stay `paced/open` without placeholder runner modes, fake active fixtures, or
fabricated coverage.

| Consumer task | Stage / runner | mizar-test status | Next condition |
|---|---|---|---|
| `mizar-parser` task 3 | `parse_only` / `parse-only` | prepared/implemented; active `.miz` pass/fail sidecars use `active_parse_only`, and untagged parse-only metadata stays planned | Keep the transitional `SurfaceAst` snapshot shortcut until the general snapshot runner lands. |
| `mizar-resolve` task 23 | `declaration_symbol` / `declaration-symbol` | prepared/implemented; active sidecars use `active_declaration_symbol`, public resolver diagnostic-code matching remains gated | Open public diagnostic-code assertions only after resolver diagnostic ranges are specified. |
| `mizar-checker` task 12 plus task 16-20, task 48 source bridge continuation, task 50 attributed reserve evidence-gap bridge, task 51 local mode expansion-gap bridge, task 52 local structure evidence-gap bridge, task 53 attributed local structure evidence-gap bridge, task 54 attributed local mode expansion-gap bridge, task 55 bare local mode expansion bridge, task 56 local-mode expansion chain bridge, task 57 local-mode structure-RHS evidence-gap bridge, task 58 local-mode attributed-builtin-RHS evidence-gap bridge, task 59 attributed local-mode reserve evidence-gap bridge, task 60 attributed local-mode structure-RHS evidence-gap bridge, task 61 attributed local-mode attributed-builtin-RHS evidence-gap bridge, task 62 local-mode structure-RHS chain evidence-gap bridge, task 63 local-mode attributed-builtin-RHS chain evidence-gap bridge, task 64 attributed local-mode bare-builtin chain evidence-gap bridge, task 65 attributed local-mode structure-RHS chain evidence-gap bridge, task 66 attributed local-mode attributed-builtin-RHS chain evidence-gap bridge, task 67 structure-qualified attribute extraction-gap boundary, task 68 argument-bearing mode reserve extraction-gap boundary, task 69 argument-bearing structure reserve extraction-gap boundary, task 70 bracket-form local mode reserve extraction-gap boundary, task 71 bracket-form local structure reserve extraction-gap boundary, task 72 two-edge bare local-mode chain bridge, task 73 three-edge bare local-mode chain bridge, task 74 structural bare local-mode chain bridge, task 75 local-mode forward-reference active-range boundary, task 76 local-structure forward-reference active-range boundary, task 77 local-attribute forward-reference active-range boundary, task 78 imported structure reserve extraction-gap boundary, task 79 imported mode reserve extraction-gap boundary, task 80 imported attribute reserve extraction-gap boundary, task 82 imported mode provenance bridge, task 83 imported structure provenance bridge, task 97 imported TypeCaseStruct provenance bridge, task 84 imported attribute provenance bridge, task 85 imported non-empty attribute provenance bridge, task 116 imported positive empty attribute provenance bridge, task 86 theorem formula extraction-gap boundary, task 115 exact formula statement checker bridge, task 117 formula constant kind checker bridge, task 106 builtin equality term/formula checker bridge, task 110 imported predicate/functor theorem checker bridge, task 108 builtin membership term/formula checker bridge, task 107 builtin inequality term/formula checker bridge, task 109 builtin type assertion term/formula/type checker bridge, task 103 imported attribute assertion formula extraction-gap boundary, task 113 imported attribute assertion checker bridge, task 114 exact attribute-level non-empty imported attribute assertion theorem checker bridge, task 111 exact set-enumeration theorem checker bridge, task 112 exact formula connective/quantifier shell checker bridge, task 88 proof skeleton extraction-gap boundary, task 89 statement proof extraction-gap boundary, task 90 predicate/functor definition extraction-gap boundary, task 91 attribute definition extraction-gap boundary, task 92 mode/structure definition extraction-gap boundary, task 93 proof-local declaration extraction-gap boundary, task 94 proof-local inline definition extraction-gap boundary, task 95 registration block extraction-gap boundary, task 96 redefinition/notation extraction-gap boundary, reserve summary-readiness, and binder-only core context follow-up | `type_elaboration` / `type-elaboration` | prepared/implemented; active sidecars use `active_type_elaboration`, lower stages run first, reserve-only builtin `set`/`object` declarations are extracted from `.miz` AST into syntax-free checker payloads, same-module attribute symbols already present in `SymbolEnv` may be attached to builtin reserve type payloads, same-module local mode reserve heads, and same-module local structure reserve heads, same-module local mode and structure symbols may be used as argument-free reserve heads, task 55 additionally extracts a real mode expansion only for bare local-mode reserve uses whose unique preceding no-argument same-module mode definition has a bare builtin RHS and no definition-local context, task 56 extracts a one-edge local-mode chain only when the dependency mode already has that accepted task-55 builtin RHS expansion, task 57 extracts a real local-mode expansion whose RHS is a same-module local structure head as a terminal expansion payload, task 58 extracts a real local-mode expansion whose RHS is an attributed builtin head as a terminal expansion payload, task 59 extracts a real direct bare-builtin local-mode expansion for an attributed local-mode reserve head when the same mode is not also used as a bare reserve head, task 60 extracts a real direct local-structure RHS local-mode expansion for an attributed local-mode reserve head when the same mode is not also used as a bare reserve head, task 61 extracts a real direct attributed-builtin RHS local-mode expansion for an attributed local-mode reserve head when the same mode is not also used as a bare reserve head, task 62 extracts both real expansions for a one-edge bare local-mode chain ending in a same-module local structure RHS under the same unique/unrecovered/preceding/no-context source constraints, task 63 extracts both real expansions for a one-edge bare local-mode chain ending in an attributed builtin RHS under the same source constraints plus argument-free same-module RHS attributes, task 64 extracts both real expansions for an attributed local-mode reserve head whose one-edge dependency chain ends in a bare builtin RHS when the root is not mixed with a bare reserve use and the dependency is not itself attributed, task 65 extracts both real expansions for an attributed local-mode reserve head whose one-edge dependency chain ends in a same-module local structure RHS when the root is not mixed with a bare reserve use, the dependency is not itself attributed, and the structure definition is unique, unrecovered, same-module, and source-preceding, task 66 extracts both real expansions for an attributed local-mode reserve head whose one-edge dependency chain ends in an attributed builtin RHS when the root is not mixed with a bare reserve use, the dependency is not itself attributed, and RHS attributes are argument-free same-module symbols, task 67 proves structure-qualified attribute references are parser/resolver executable but must remain on the extraction-gap key until real qualifier and attribute-owner provenance exist, task 68 proves argument-bearing local-mode reserve heads are parser/resolver executable but must remain on the extraction-gap key until real type-argument and term-argument provenance exist, task 69 proves argument-bearing local-structure reserve heads are parser/resolver executable but must remain on the extraction-gap key until real type-argument and term-argument provenance exist, task 70 proves bracket-form local-mode reserve heads are parser/resolver executable but must remain on the extraction-gap key until real bracket type-argument and `qua`-argument provenance exist, task 71 proves bracket-form local-structure reserve heads are parser/resolver executable but must remain on the extraction-gap key until real bracket type-argument and `qua`-argument provenance exist, task 72 extracts real two-edge bare local-mode chains ending in builtin `set` / `object`, task 73 extracts real three-edge bare local-mode chains ending in builtin `set` / `object`, task 74 extracts AST-bounded structural bare local-mode chains ending in builtin `set` / `object`, task 75 records forward same-module local-mode reserve heads as lower-stage active-range rejections before checker handoff, task 76 records forward same-module local-structure reserve heads as the same lower-stage active-range rejection before checker handoff, task 77 records forward same-module local-attribute reserve type expressions as the same lower-stage active-range rejection before checker handoff, task 78 historically records the documented imported structure `R` reserve head as a source-to-checker extraction-gap boundary case before task 83 supersedes that `R` portion, task 79 records imported mode reserve heads from that same import summary as source-to-checker extraction-gap boundary cases, task 80 historically records imported attribute reserve types from that same import summary as source-to-checker extraction-gap boundary cases before task 84 supersedes the documented `TypeCaseAttr` portion, task 85 supersedes the negative `empty`/builtin-`set` portion, and task 116 supersedes the positive `empty`/builtin-`set` portion, task 82 promotes the `TypeCaseMode` imported mode summary symbol to a checker type-head payload that stops at the checker missing mode-expansion diagnostic, task 83 promotes the `R` imported structure summary symbol to a checker type-head payload that stops at the checker missing structure-evidence query, task 97 promotes the `TypeCaseStruct` imported structure summary symbol to the same checker type-head payload and missing structure-evidence query, task 84 promotes the `TypeCaseAttr` imported attribute summary symbol to a checker `AttributeInput` payload that stops at the checker missing attributed-type evidence query, task 85 promotes the imported `empty` attribute summary symbol for negative `non empty set` to a checker `AttributeInput` payload that stops at the same evidence-query diagnostic, task 116 promotes the same imported `empty` summary symbol for positive `empty set` to a checker `AttributeInput` payload that stops at the same evidence-query diagnostic, task 86 historically executes a formula-only theorem source through parser/resolver; task 117 supersedes task 115 for the exact `FormulaPayloadBoundary: thesis` sidecar by passing the source-derived `thesis` formula constant as a real `FormulaKind::Thesis` checker payload before failing closed on missing formula payload, task 106 executes the exact builtin equality theorem source through parser/resolver, passes real checker term/formula payloads, and fails closed on missing numeric type payloads plus partial formula checking, task 110 supersedes task 98 for the exact imported predicate/functor theorem formula source, passes real checker numeral, imported functor-application, and predicate-application payloads, and fails closed on missing numeric/signature payloads plus partial formula checking, task 108 executes the exact builtin membership theorem source through parser/resolver, passes real checker term/formula payloads, and fails closed on missing numeric type payloads plus partial formula checking, task 107 executes the exact builtin inequality theorem source through parser/resolver, passes real checker term/formula payloads, and fails closed on missing numeric type payloads plus partial formula checking, task 109 executes the exact builtin type-assertion theorem source through parser/resolver, passes real checker term/formula/asserted-type payloads, and fails closed on missing numeric type payloads plus partial formula checking, task 103 historically executes an imported attribute assertion theorem formula source through parser/resolver but keeps non-bridged variants on the checker source-to-payload extraction gap, task 113 executes the exact imported empty attribute assertion theorem source through parser/resolver, passes real checker term/formula payloads, and fails closed on missing numeric type payload, missing formula/attribute semantic payload, and partial formula checking, task 114 supersedes task 104 for the exact attribute-level non-empty imported attribute assertion theorem formula source, passes real checker term/formula payloads, and fails closed on missing numeric type payload, missing formula/attribute semantic payload, and partial formula checking, task 111 executes the exact set-enumeration theorem source through parser/resolver, passes real checker term/formula payloads, and fails closed on missing numeric/result-type payloads plus partial formula checking, task 112 executes the exact connective/quantifier theorem formula source through parser/resolver, passes real checker formula shell payloads, and fails closed on missing formula/quantifier payloads, task 88 executes a proof-block theorem source through parser/resolver but keeps it on the checker source-to-payload extraction gap, task 89 executes statement-level proof-justification theorem sources through parser/resolver but keeps them on the checker source-to-payload extraction gap, and task 90 executes predicate/functor definition sources through parser/resolver but keeps them on the checker source-to-payload extraction gap, and task 91 executes attribute definition sources through parser/resolver but keeps them on the checker source-to-payload extraction gap, and task 92 executes mode/structure definition sources through parser/resolver but keeps them on the checker source-to-payload extraction gap, and task 93 executes proof-local declaration statements through parser/resolver but keeps them on the checker source-to-payload extraction gap, and task 94 executes proof-local inline definitions through parser/resolver but keeps them on the checker source-to-payload extraction gap, and task 95 executes registration blocks through parser/resolver but keeps them on the checker source-to-payload extraction gap, and task 96 executes redefinition/notation surfaces through parser/resolver but keeps them on the checker source-to-payload extraction gap, `mizar-checker` produces the checker-owned `BindingEnv`, one `DeclarationInput` per binding, binding-specific `TypeExpressionInput` sites, and `DeclarationChecker` output, successful bare-builtin, task-55 bare local-mode, task-56 chain, and task-74 structural bare-chain cases continue through `TypedAst`, checker-owned `ResolvedTypedAst`, a `mizar-core` `ResolvedTypedAstSummary::from_ast` read, and binder-only `CoreContext` preparation, while attributed reserve, local-structure, task-57 structure-RHS expansion, task-58 attributed-RHS expansion, task-59 attributed local-mode expansion, task-60 attributed local-mode structure-RHS expansion, task-61 attributed local-mode attributed-RHS expansion, task-62 local-mode structure-RHS chain expansion, task-63 local-mode attributed-RHS chain expansion, task-64 attributed local-mode bare-builtin chain expansion, task-65 attributed local-mode structure-RHS chain expansion, and task-66 attributed local-mode attributed-RHS chain expansion cases stop at the checker `MissingEvidenceQuery` diagnostic and local-mode cases outside task 55/56/57/58/59/60/61/62/63/64/65/66/74, including mixed attributed/bare local-mode sources, attributed chain dependencies, or chains that violate task-74 structural guards, stop at the missing mode-expansion diagnostic; task-67 structure-qualified attribute cases, task-68 argument-bearing mode cases, task-69 argument-bearing structure cases, task-70 bracket-form mode cases, task-71 bracket-form structure cases, broader imported-structure cases outside the task-83 `R` bridge and task-97 `TypeCaseStruct` bridge remain deferred until a matching source-derived fixture exists; broader imported-attribute cases outside the task-84 `TypeCaseAttr` bridge and task-85/task-116 `empty`/builtin-`set` bridges and unsupported checker payload families stay on `type_elaboration.external_dependency.ast_payload_extraction`; task-82 `TypeCaseMode` imported mode cases stop at `type_elaboration.checker.checker.type.external.mode_expansion_payload`; task-83 `R` imported structure cases, task-97 `TypeCaseStruct` imported structure cases, task-84 `TypeCaseAttr` imported attribute cases, and task-85/task-116 `empty`/builtin-`set` imported attribute cases stop at `type_elaboration.checker.checker.declaration.deferred.evidence_query`; task-75 forward local-mode reserve heads, task-76 forward local-structure reserve heads, and task-77 forward local-attribute reserve type expressions stay on `type_elaboration.lower_stage.frontend:malformed_type_expression` before checker handoff; task-117 exact formula statement checker outputs still lack formula constant semantics, child-formula graph payloads, facts, theorem acceptance, proof skeletons, CoreIr, ControlFlowIr, VC, proof payloads, and `formula_statement` runner activation, while non-exact task-86 formula-only variants stay on `type_elaboration.external_dependency.ast_payload_extraction`; task-106 builtin equality theorem formulas fail closed on checker diagnostics without numeric type payloads, equality checking, facts, proof skeletons, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-109 builtin type-assertion theorem formulas fail closed on checker diagnostics without numeric type payloads, broader asserted type payloads, type-assertion semantic checking, facts, theorem acceptance, proof skeletons, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-103 historical imported attribute assertion theorem boundary cases outside task 113 stay on the same extraction gap without term/formula payloads, imported attribute assertion attribute-chain/provenance payload extraction, term inference, attribute admissibility/semantic checking, formula checking, facts, theorem acceptance, imported module AST extraction, checker `AttributeInput` payload extraction for theorem formulas, proof skeletons, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-88 proof-block theorem sources stay on that extraction gap without proof skeleton payloads, local proof contexts, formula payloads, facts, theorem acceptance, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-89 statement-proof theorem sources stay on that extraction gap without statement proof payloads, nested proof skeleton payloads, local proof contexts, formula payloads, label-reference semantic checking, facts, theorem acceptance, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-91 attribute definition sources stay on that extraction gap without definition declaration payloads, definition-local context, formula-definiens payloads, attributed-type evidence, facts, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-92 mode/structure definition sources stay on that extraction gap without definition declaration payloads, mode expansion, structure base-shape/constructor/selector evidence, definition-local context, facts, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-93 proof-local declaration statement sources stay on that extraction gap without proof-local declaration payloads, local proof contexts, formula/term payloads, RHS term inference, reconsider coercion/obligation evidence, facts, theorem acceptance, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-94 proof-local inline definition sources stay on that extraction gap without inline definition formal/body payloads, local abbreviation expansion, term/formula body payloads, guard evidence, facts, theorem acceptance, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-95 registration-block sources stay on that extraction gap without registration-item payloads, correctness-condition/proof-obligation payloads, accepted activation/evidence status, cluster/reduction semantics, Chapter 17 semantic rows, facts, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` / `advanced_semantics` runner activation; task-96 redefinition/notation sources stay on that extraction gap without redefinition payloads, notation alias relation payloads, redefinition target inference, coherence proof-obligation payloads, overload candidate payloads, Chapter 11 alias semantic resolution, Chapter 19 overload/redefinition semantics, facts, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` / `advanced_semantics` runner activation | Broader type/formula pass/fail semantic assertions wait for AST-wide source-to-checker payload extraction and real existential/evidence-query/mode-expansion/base-shape/imported-structure/imported-attribute/qualified-attribute/type-argument/term-argument/bracket-argument/theorem-formula/proof-context provenance inputs beyond the task-55 bare builtin RHS, task-56 one-edge chain, task-57 structure-RHS diagnostic slice, task-58 attributed-RHS diagnostic slice, task-59 attributed local-mode reserve diagnostic slice, task-60 attributed local-mode structure-RHS diagnostic slice, task-61 attributed local-mode attributed-RHS diagnostic slice, task-62 local-mode structure-RHS chain diagnostic slice, task-63 local-mode attributed-RHS chain diagnostic slice, task-64 attributed local-mode bare-builtin chain diagnostic slice, task-65 attributed local-mode structure-RHS chain diagnostic slice, task-66 attributed local-mode attributed-builtin-RHS chain diagnostic slice, task-67 extraction-gap boundary slice, task-68 extraction-gap boundary slice, task-69 extraction-gap boundary slice, task-70 extraction-gap boundary slice, task-71 extraction-gap boundary slice, task-72 two-edge bare local-mode pass slice, task-74 structural bare local-mode pass slice, task-78 historical extraction-gap boundary slice, task-79 extraction-gap boundary slice, task-80 historical extraction-gap boundary slice, task-82 TypeCaseMode provenance bridge, task-83 `R` imported-structure provenance bridge, task-97 `TypeCaseStruct` imported-structure provenance bridge, task-84 `TypeCaseAttr` imported-attribute provenance bridge, task-85 negative `empty`/builtin-`set` provenance bridge, task-116 positive `empty`/builtin-`set` provenance bridge, task-106 builtin equality theorem checker bridge, task-107 builtin inequality theorem checker bridge, task-108 builtin membership theorem checker bridge, task-109 builtin type assertion theorem checker bridge, task-110 checker bridge, task-112/task-117 formula-shell checker bridge, task-113 imported attribute assertion checker bridge, task-114 exact attribute-level non-empty imported attribute assertion checker bridge, task-117 exact formula statement checker bridge, and task-103/task-105 theorem/formula extraction-gap boundary slices, task-88 proof-skeleton extraction-gap boundary slice, task-89 statement-proof extraction-gap boundary slice, task-92 mode/structure definition extraction-gap boundary slice, task-93 proof-local declaration extraction-gap boundary slice, task-94 proof-local inline definition extraction-gap boundary slice, task-95 registration block extraction-gap boundary slice, task-96 redefinition/notation extraction-gap boundary slice, and task-75/task-76/task-77 active-range boundary slices. |
| `mizar-checker` task 29 | `formula_statement` / `advanced_semantics` | paced/open; trace rows are deferred and no active fixture is fabricated | Add runner support only after statement/formula and advanced-semantics source payload seams exist. |
| `mizar-vc` task 15 plus task 31 exact exception | `proof_verification` / `proof-verification` | exact Task-180 source-to-checker-to-Core-to-VC generation and full snapshot comparison are implemented; broader VC/proof-verification families remain paced/open | Activate each broader phase-11 generation route only when its exact source/Core/VC payload contract, owning task authority, and consumer readiness are present; downstream proof verification remains separately deferred. |
| `mizar-atp` task 20 | `advanced_semantics` metadata handoff | paced/open in `mizar-test`; metadata-only property fixtures may be consumed by `mizar-atp` Rust tests | Add active `.miz` ATP runner support only after source-derived ATP extraction and proof-policy/kernel handoff seams exist. |
| `mizar-kernel` task 17 | proof/certificate/kernel evidence | paced/open; fail/soundness metadata is validated without active proof/certificate/kernel execution | Add runner support only after source-to-evidence or certificate execution seams exist. |

Tasks 85, 116, and 171 refine the `type_elaboration` consumer row above: the
imported attribute gap list now excludes the documented negative and positive
`empty` over builtin `set` fixtures and the exact negative `empty` over builtin
`object` fixture, in addition to task 84's `TypeCaseAttr` fixture. Positive
`empty object`, imported attributes on symbol heads, generic imported
attributes, imported module AST extraction, arguments, owner provenance,
evidence payloads, CoreIr, ControlFlowIr, VC, and proof rows stay outside the
supported slice.

Task 86 refines the same row by proving formula-only theorem sources are
executable through the active `type_elaboration` runner. Task 115 supersedes
only the exact `FormulaPayloadBoundary: thesis` source by passing the
source-derived `thesis` formula constant site/range to the checker as a
recovery `FormulaInput`; task 117 supersedes that recovery marker with a real
`FormulaKind::Thesis` payload, then failing closed on missing formula payload.
It does not satisfy the deferred
`formula_statement` runner obligation and does not credit formula constant
semantic checking, child-formula graph payloads, facts, theorem acceptance,
proof skeletons, CoreIr, ControlFlowIr, VC, or proof payloads.

Task 87 originally refined the same row by proving that the term-bearing
equality theorem source was parser/resolver executable as an extraction-gap
boundary. Task 106 supersedes only the exact labelled
`TermFormulaPayloadBoundary: 1 = 1` portion by building real checker
term/formula payloads and failing closed on missing numeric type payloads plus
partial formula checking. It still does not satisfy the deferred
`formula_statement` runner obligation and does not credit numeric type payloads,
equality semantic checking, facts, proof skeletons, CoreIr, ControlFlowIr, VC,
or proof payloads.

Task 98 originally refined the same row by proving that imported
predicate/functor theorem formulas were parser/resolver executable as
extraction-gap boundaries. Task 110 supersedes only the exact labelled
`ImportedPredicateFunctorPayloadBoundary: 1 divides (1 ++ 2)` sidecar by
building real checker numeral, imported functor-application, and
predicate-application payloads and failing closed on missing numeric and
signature payloads plus partial formula checking. It still does not satisfy the
deferred `formula_statement` runner obligation and does not credit imported
module AST extraction, semantic predicate/functor signatures, term inference,
formula checking, facts, proof skeletons, CoreIr, ControlFlowIr, VC, or proof
payloads. In the row above, the theorem formula boundary entry now covers task
110's exact imported predicate/functor checker bridge.

Task 100 originally refined the same row by proving that builtin membership
theorem formulas were parser/resolver executable as extraction-gap boundaries.
Task 108 supersedes only the exact labelled
`BuiltinMembershipPayloadBoundary: 1 in 1` sidecar with a real checker
term/formula payload handoff that still fails closed on missing numeric type
payloads and partial formula checking. It does not satisfy the deferred
`formula_statement` runner obligation and does not credit membership operand
expected-type construction/checking, facts, theorem acceptance, CoreIr,
ControlFlowIr, VC, or proof payloads. In the row above, the theorem formula
boundary entry now covers task 108's exact builtin membership checker bridge.

Task 101 originally refined the same row by proving that builtin inequality
theorem formulas were parser/resolver executable as extraction-gap boundaries.
Task 107 supersedes only the exact labelled
`BuiltinInequalityPayloadBoundary: 1 <> 2` portion by building real checker
term/formula payloads and failing closed on missing numeric type payloads plus
partial formula checking. It still does not satisfy the deferred
`formula_statement` runner obligation and does not credit inequality
desugaring/equality semantic checking, facts, theorem acceptance, CoreIr,
ControlFlowIr, VC, or proof payloads.

Task 118 tightens the shared builtin-binary theorem producer for tasks 106, 107,
and 108: exact checker handoff remains limited to direct theorem tokens
`theorem <label> : ;`, while status-prefixed or extra-token theorem shapes stay
on `type_elaboration.external_dependency.ast_payload_extraction`. This is a
guard repair only and does not add active sidecar or traceability coverage.

Task 119 adds one exact no-diagnostic `type-elaboration` pass case:
`reserve x for set; theorem ReservedVariableEqualityPayloadBoundary: x = x;`.
The runner resolves both identifier terms through the real reserve
`BindingEnv` with separate source-order-derived use ordinals, preserves four
distinct source-anchored result/expected type role sites, and requires two
`Inferred` terms plus one `Checked` equality with empty candidates,
diagnostics, deferred reasons, and facts. Production runner validation checks
the exact binding/reference identities plus every role owner and normalized
type source range/spelling/head; an invariant mismatch reports
`type_elaboration.checker.reserved_variable_equality.invalid_payload`. A runner
unit test discovers the active sidecar and repeats these payload assertions on
the AST produced by the real frontend and resolver, rather than only on a
hand-built syntax tree. The pass result is not theorem acceptance and does not
activate implicit closure, `formula_statement`, proof, CoreIr, ControlFlowIr,
or VC consumers. Non-exact shapes continue to report the extraction-gap key.

Task 123 adds the exact distinct-binding equality pass case
`reserve x, y for set; theorem DistinctReservedVariableEqualityPayloadBoundary: x = y;`.
The active runner preserves the real two-binding reserve handoff and shared
written builtin `set` range, derives lookup ordinals 2 and 3 after both source
bindings, and resolves the operands to distinct checker binding ids.
Operand-specific result/expected roles reach two `Inferred` variables and one
fact-free `Checked` equality. A task-specific invalid-payload key, near-miss
matrix, and real frontend/resolver active-sidecar test validate the exact seam.
Separate reserve items, reversed or identical operands, wrong labels,
operators, types, extra bindings/items, status/recovery, and numerals stay on
the extraction gap. The pass does not credit implicit closure/order, equality
truth/facts, theorem acceptance, `formula_statement`, proof, CoreIr,
ControlFlowIr, or VC.

Task 124 adds the exact multiple-reserve-declaration equality pass case
`reserve x for set; reserve y for set; theorem MultipleReserveDeclarationEqualityPayloadBoundary: x = y;`.
The runner accepts exactly two ordered reserve items, keeps `BindingId(0)` and
`BindingId(1)`, and retains each declaration's distinct written builtin `set`
range in that operand's result and expected pre-normalization inputs. The real
checker interns the semantically equal inputs to one normalized type whose
canonical source is the earliest written range; production validation checks
the four original inputs before relying on that semantic identity. A dedicated
invalid-payload key, near-miss matrix, and real frontend/resolver active-sidecar
test guard the exact seam. Shared multi-name segments, reversed directives or
operands, mixed/extra declarations, wrong operators, status/recovery, extra
theorems, and numeral operands stay on the extraction gap. The pass does not
credit implicit closure/order, equality truth/facts, theorem acceptance,
`formula_statement`, proof, CoreIr, ControlFlowIr, or VC.

Task 125 adds the exact heterogeneous-reserve membership pass case
`reserve x for object; reserve y for set; theorem HeterogeneousReserveMembershipPayloadBoundary: x in y;`.
The runner accepts exactly two ordered reserve items, preserves `x` as a real
builtin-`object` binding and `y` as a real builtin-`set` binding, and retains
the two written ranges in the left result, right result, and sole right expected
input. Production validation requires two normalized identities: the right
result/expected roles share `set`, while the left `object` identity stays
distinct and both identities keep deterministic source representatives. A
task-specific invalid key, exact near-miss matrix, and real frontend/resolver
active-sidecar test guard the seam. Non-exact types/order/operands/operators,
extra declarations, status/recovery, and numeral operands stay on the extraction
gap. The pass does not credit membership truth/facts, object/set coercion,
implicit closure/order, theorem acceptance, `formula_statement`, proof,
CoreIr, ControlFlowIr, or VC.

Task 126 adds the exact direct-local-mode reserved-variable equality pass. The
runner admits one task-55-compatible mode definition, retains four raw
`LocalModeFormula` inputs, and supplies its real AST-derived bare-set expansion
to `TermFormulaChecker`; all roles normalize to one builtin-set identity. An
invalid key, withheld-mode near misses, and a real sidecar guard the slice. Mode
declaration acceptance/inhabitation, broader modes, closure/order, facts/truth,
theorem acceptance, proof, CoreIr, ControlFlowIr, and VC remain deferred.

Task 127 adds the exact one-edge local-mode-chain reserved-variable equality
pass. The runner admits two exact source-preceding definition blocks, retains
four raw outer `ChainModeFormula` inputs, and supplies both real task-56
expansions to `TermFormulaChecker`; recursive normalization yields one
builtin-set identity anchored at the terminal `set` RHS. An invalid-link key,
exact chain guards, withheld-family near misses, and a real sidecar guard the
slice. Mode declaration acceptance/inhabitation, object terminals, longer-chain
formulas, closure/order, facts/truth, theorem acceptance, proof, CoreIr,
ControlFlowIr, and VC remain deferred.

Task 128 adds the exact direct local-object-mode reserved-variable equality
pass. The runner admits one task-55-compatible `LocalObjectMode -> object`
definition, retains four raw local object-mode inputs, and supplies its real
AST-derived expansion to `TermFormulaChecker`; all roles normalize to one
builtin-object identity anchored at the real `object` RHS. An invalid key,
exact block/label guards, withheld-family near misses, and a real sidecar guard
the slice. Mode declaration acceptance/inhabitation, broader object-mode
formulas, closure/order, facts/truth, theorem acceptance, proof, CoreIr,
ControlFlowIr, and VC remain deferred.

Task 129 adds the exact one-edge local-object-mode-chain equality pass. The
runner retains four raw `ChainObjectMode` inputs, supplies both real task-56
expansions to `TermFormulaChecker`, and anchors one builtin-object identity at
the terminal `object` RHS. Invalid-link corruption, withheld-family near
misses, and a real sidecar guard the exact slice. Declaration
acceptance/inhabitation, longer chains, closure/order, facts/truth, theorem
acceptance, proof, CoreIr, ControlFlowIr, and VC remain deferred.

Task 130 adds the exact direct local-mode inequality pass. The runner retains
four raw `LocalModeInequality` inputs, supplies the real direct expansion to
`TermFormulaChecker`, anchors one builtin-set identity at the RHS, and requires
one fact-free pre-desugaring `Checked` inequality. Exact guards, corruption
coverage, and a real sidecar protect the slice; downstream semantics remain
deferred.

Task 131 adds the exact direct local-object-mode inequality pass. The runner
retains four raw `LocalObjectModeInequality` inputs, supplies the real direct
expansion to `TermFormulaChecker`, anchors one builtin-object identity at the
RHS, and requires one fact-free pre-desugaring `Checked` inequality. Exact
guards, present/missing expansion corruption coverage, and a real sidecar
protect the slice; downstream semantics remain deferred.

Task 132 adds the exact one-edge set-terminal local-mode-chain inequality pass.
The runner retains four raw `ChainModeInequality` inputs, supplies both real
task-56-compatible expansions to `TermFormulaChecker`, anchors one builtin-set
identity at the terminal `set` RHS, and requires one fact-free pre-desugaring
`Checked` inequality. Exact chain guards, missing-link corruption, withheld-
family near misses, and a real sidecar protect the slice; declaration
acceptance/inhabitation, desugaring, closure/order, theorem/proof/Core/VC, and
broader semantics remain deferred.

Task 133 adds the exact one-edge object-terminal local-mode-chain inequality
pass. The runner retains four raw `ChainObjectModeInequality` inputs, supplies
both real expansions to `TermFormulaChecker`, anchors one builtin-object
identity at the terminal `object` RHS, and requires one fact-free
pre-desugaring `Checked` inequality. Exact chain guards, missing-link
corruption, withheld-family near misses, and a real sidecar protect the slice;
declaration acceptance/inhabitation, desugaring, closure/order, truth/facts,
theorem/proof/Core/VC, and broader semantics remain deferred.

Task 134 adds the exact two-edge set-terminal local-mode-chain equality pass.
The runner retains four raw `OuterTwoEdgeModeEquality` inputs, supplies all
three real Task-72-compatible expansions to `TermFormulaChecker`, anchors one
builtin-set identity at the terminal `set` RHS, and requires one fact-free
`Checked` equality. Exact chain guards, missing-link corruption, withheld-family
near misses, and a real sidecar protect the slice; declaration
acceptance/inhabitation, implicit closure/order, theorem/proof/Core/VC, and
broader semantics remain deferred.

Task 135 adds the exact two-edge object-terminal local-mode-chain equality
pass. The runner retains four raw `OuterTwoEdgeObjectModeEquality` inputs,
supplies all three real Task-72-compatible expansions to `TermFormulaChecker`,
anchors one builtin-object identity at the terminal `object` RHS, and requires
one fact-free `Checked` equality. Exact chain guards, missing-link corruption,
withheld-family near misses, and a real sidecar protect the slice; declaration
acceptance/inhabitation, implicit closure/order, theorem/proof/Core/VC, and
broader semantics remain deferred.

Task 136 adds the exact two-edge set-terminal local-mode-chain inequality pass.
The runner retains four raw `OuterTwoEdgeModeInequality` inputs, supplies all
three real Task-72-compatible expansions to `TermFormulaChecker`, anchors one
builtin-set identity at the terminal `set` RHS, and requires one fact-free
pre-desugaring `Checked` inequality. Exact chain guards, missing-link
corruption, withheld-family near misses, and a real sidecar protect the slice;
mode declaration acceptance/inhabitation, inequality desugaring, implicit
closure/order, theorem/proof/Core/VC, and broader semantics remain deferred.

Task 137 adds the exact two-edge object-terminal local-mode-chain inequality
pass. The runner retains four raw `OuterTwoEdgeObjectModeInequality` inputs,
supplies all three real Task-72-compatible expansions to `TermFormulaChecker`,
anchors one builtin-object identity at the terminal `object` RHS, and requires
one fact-free pre-desugaring `Checked` inequality. Exact chain guards,
missing-link corruption, withheld-family near misses, and a real sidecar protect
the slice; declaration acceptance/inhabitation, inequality desugaring, implicit
closure/order, theorem/proof/Core/VC, and broader semantics remain deferred.

Task 138 adds the exact direct set-terminal local-mode reserved-variable type
assertion pass. The runner retains the raw `LocalModeTypeAssertion` subject
input and the independent formula-side builtin-set asserted input, supplies the
one real Task-55-compatible expansion to `TermFormulaChecker`, and requires one
terminal-RHS builtin-set identity, `BindingId(0)`, one `Inferred` term, and one
fact-free `Checked` type assertion. Exact source guards, missing-expansion
corruption, withheld-family near misses, and a real sidecar protect the slice;
mode declaration acceptance/inhabitation, formula-side local-mode asserted
heads, general reachability/widening/`qua`, theorem/proof/Core/VC, and broader
semantics remain deferred. The active type-elaboration runner contains 89
cases before Task 139.

Task 139 adds the exact direct set-terminal local-mode left reserved-variable
membership pass. The runner retains the raw `LocalModeMembership` left result
and the independent explicit-set right result/expected input, supplies the one
real Task-55-compatible expansion to `TermFormulaChecker`, and requires one
terminal-RHS builtin-set identity, `BindingId(0/1)`, two `Inferred` terms, one
fact-free `Checked` membership, only the right expected constraint, and no left
expected input. Exact source guards, independent expansion/right-expected
corruption, withheld-family near misses, and a real sidecar protect the slice;
mode declaration acceptance/inhabitation, membership truth/facts, implicit
closure/order, theorem/proof/Core/VC, and broader semantics remain deferred.
The active type-elaboration runner contains 90 cases.

Task 140 adds the exact direct object-terminal local-mode left reserved-variable
membership pass. The runner retains the raw `LocalObjectModeMembership` left
result and the independent explicit-set right result/expected input, supplies
the one real Task-55-compatible expansion to `TermFormulaChecker`, and requires
distinct terminal-RHS builtin-object and explicit-reserve builtin-set
identities, `BindingId(0/1)`, two `Inferred` terms, one fact-free `Checked`
membership, only the right expected constraint, and no left expected input.
Exact source guards, independent expansion/right-expected corruption,
withheld-family near misses, and a real sidecar protect the slice; mode
declaration acceptance/inhabitation, membership truth/facts, object/set
coercion, implicit closure/order, theorem/proof/Core/VC, and broader semantics
remain deferred. The active type-elaboration runner contains 91 cases.

Task 141 adds the exact one-edge set-terminal local-mode-chain left reserved-
variable membership pass. The runner retains the raw `ChainModeMembership`
left result and independent explicit-set right result/expected input, supplies
both real Task-56-compatible expansions to `TermFormulaChecker`, and requires
one terminal-RHS builtin-set identity, `BindingId(0/1)`, two `Inferred` terms,
one fact-free `Checked` membership, only the right expected constraint, and no
left expected input. Exact source guards, independent chain-link/right-expected
corruption, withheld-family near misses, and a real sidecar protect the slice;
mode declaration acceptance/inhabitation, membership truth/facts, implicit
closure/order, theorem/proof/Core/VC, and broader semantics remain deferred.
The active type-elaboration runner contains 92 cases.

Task 142 adds the exact one-edge object-terminal local-mode-chain left
reserved-variable membership pass. The runner retains the raw
`ChainObjectModeMembership` left result and independent explicit-set right
result/expected input, supplies both real Task-56-compatible expansions to
`TermFormulaChecker`, and requires distinct terminal-RHS builtin-object and
explicit-reserve builtin-set identities, `BindingId(0/1)`, two `Inferred`
terms, one fact-free `Checked` membership, only the right expected constraint,
and no left expected input. Exact source guards, independent chain-link/right-
expected corruption, withheld-family near misses, and a real sidecar protect
the slice; mode declaration acceptance/inhabitation, membership truth/facts,
object/set coercion, implicit closure/order, theorem/proof/Core/VC, and broader
semantics remain deferred. The active type-elaboration runner contains 93
cases.

Task 143 adds the exact two-edge set-terminal local-mode-chain left reserved-
variable membership pass. The runner retains the raw
`OuterTwoEdgeModeMembership` left result and independent explicit-set right
result/expected input, supplies all three real Task-72-compatible expansions
to `TermFormulaChecker`, and requires one terminal-RHS builtin-set identity,
`BindingId(0/1)`, two `Inferred` terms, one fact-free `Checked` membership,
only the right expected constraint, and no left expected input. Exact source
guards, independent three-link/right-expected corruption, withheld-family near
misses, and a real sidecar protect the slice; mode declaration
acceptance/inhabitation, membership truth/facts, implicit closure/order,
theorem/proof/Core/VC, and broader semantics remain deferred. The active
type-elaboration runner contains 94 cases.

Task 144 adds the exact two-edge object-terminal local-mode-chain left
reserved-variable membership pass. The runner retains the raw
`OuterTwoEdgeObjectModeMembership` left result and independent explicit-set
right result/expected input, supplies all three real Task-72-compatible
expansions to `TermFormulaChecker`, and requires distinct terminal-object-RHS
builtin-object and explicit-reserve builtin-set identities, `BindingId(0/1)`,
two `Inferred` terms, one fact-free `Checked` membership, only the right
expected constraint, no left expected input, and no object/set coercion. Exact
source guards, independent three-link/right-expected corruption,
withheld-family near misses, and a real sidecar protect the slice; mode
declaration acceptance/inhabitation, membership truth/facts, implicit
closure/order, theorem/proof/Core/VC, and broader semantics remain deferred.
The active type-elaboration runner contains 95 cases.

Task 145 adds the exact direct object-terminal local-mode reserved-variable
normalized-reflexive type assertion pass. The runner retains the raw
`LocalObjectModeTypeAssertion` subject result and independent formula-side
builtin-object asserted source, supplies the one real Task-55-compatible
expansion to `TermFormulaChecker`, and requires one terminal-RHS-anchored
builtin-object identity, `BindingId(0)`, source-order use ordinal 1, one
`Inferred` term, and one fact-free `Checked` type assertion without general
reachability or object/set coercion. Exact source guards, independent
definition/expansion corruption, withheld-family near misses, and a real
frontend/resolver sidecar protect the slice; mode declaration
acceptance/inhabitation, formula-side local-mode asserted heads, general
reachability/widening/`qua`, truth/facts, closure/order, theorem/proof/Core/VC,
and broader semantics remain deferred. The active type-elaboration runner
contains 96 cases.

Task 146 adds the exact one-edge set-terminal local-mode-chain reserved-
variable normalized-reflexive type assertion pass. The runner retains the raw
`ChainModeTypeAssertion` subject result and independent formula-side builtin-
set asserted source, supplies both real Task-56-compatible expansions to
`TermFormulaChecker`, and requires one terminal-RHS-anchored builtin-set
identity, `BindingId(0)`, source-order use ordinal 1, one `Inferred` term, and
one fact-free `Checked` type assertion without general reachability. Exact
source guards, independent definition/two-link corruption, withheld-family
near misses, and a real frontend/resolver sidecar protect the slice; mode
declaration acceptance/inhabitation, formula-side local-mode asserted heads,
general reachability/widening/`qua`, truth/facts, closure/order,
theorem/proof/Core/VC, and broader semantics remain deferred. The active type-
elaboration runner contains 97 cases.

Task 147 adds the exact one-edge object-terminal local-mode-chain reserved-
variable normalized-reflexive type assertion pass. The runner
retains the raw `ChainObjectModeTypeAssertion` subject result and independent
formula-side builtin-object asserted source, supplies both real Task-56-
compatible expansions to `TermFormulaChecker`, and requires one terminal-RHS-
anchored builtin-object identity, `BindingId(0)`, source-order use ordinal 1,
one `Inferred` term, and one fact-free `Checked` type assertion without general
reachability or object/set coercion. Exact source guards, independent
definition/two-link corruption, withheld-family near misses, and a real
frontend/resolver sidecar protect the slice; mode declaration acceptance/
inhabitation, formula-side local-mode asserted heads, general reachability/
widening/`qua`, truth/facts, closure/order, theorem/proof/Core/VC, and broader
semantics remain deferred. The active type-elaboration runner contains 98
cases.

Task 148 adds the exact two-edge set-terminal local-mode-chain reserved-
variable normalized-reflexive type assertion pass. The runner retains the
raw `OuterTwoEdgeModeTypeAssertion` subject result and independent formula-side
builtin-set asserted source, supplies all three real Task-72-compatible
expansions to `TermFormulaChecker`, and requires one terminal-RHS-anchored
builtin-set identity, `BindingId(0)`, source-order use ordinal 1, one
`Inferred` term, and one fact-free `Checked` type assertion without general
reachability. Exact source guards, independent definition/three-link
corruption, withheld-family near misses, and a real frontend/resolver sidecar
protect the slice; mode declaration acceptance/inhabitation, formula-side
local-mode asserted heads, general reachability/widening/`qua`, truth/facts,
closure/order, theorem/proof/Core/VC, and broader semantics remain deferred.
The active type-elaboration runner contains 99 cases.

Task 149 adds the exact two-edge object-terminal local-
mode-chain reserved-variable normalized-reflexive type assertion source. The
runner retains the raw `OuterTwoEdgeObjectModeTypeAssertion` subject
result and independent formula-side builtin-object asserted source, supplies
all three real Task-72-compatible expansions to `TermFormulaChecker`, and
requires one terminal-RHS-anchored builtin-object identity, `BindingId(0)`,
source-order use ordinal 1, one `Inferred` term, and one fact-free `Checked`
type assertion without general reachability or object/set coercion. Exact
source guards, independent definition/three-link corruption, withheld-family
near misses, and a real frontend/resolver sidecar protect the slice; mode
declaration acceptance/inhabitation, formula-side local-mode asserted heads,
general reachability/widening/`qua`, truth/facts, closure/order, theorem/proof/
Core/VC, and broader semantics remain deferred. The production route and real
sidecar pass, so the active type-elaboration runner contains 100 cases.

Task 150 adds the exact three-edge set-terminal local-mode-chain reserved-
variable normalized-reflexive type assertion source. The runner must retain the
raw `OuterThreeEdgeModeTypeAssertion` subject result and independent formula-
side builtin-set asserted source, supply all four real Task-73-compatible
expansions to `TermFormulaChecker`, and require one terminal-RHS-anchored
builtin-set identity, `BindingId(0)`, source-order use ordinal 1, one
`Inferred` term, and one fact-free `Checked` type assertion without general
reachability. Exact source guards, independent definition/four-link
corruption, withheld-family near misses, and a real frontend/resolver sidecar
protect the slice; mode declaration acceptance/inhabitation, formula-side
local-mode asserted heads, general reachability/widening/`qua`, truth/facts,
closure/order, theorem/proof/Core/VC, and broader semantics remain deferred.
The production route and real sidecar pass, so the active type-elaboration
runner contains 101 cases.

Task 151 adds the exact three-edge object-terminal local-mode-chain reserved-
variable normalized-reflexive type assertion source. The runner must retain the
raw `OuterThreeEdgeObjectModeTypeAssertion` subject result and independent
formula-side builtin-object asserted source, supply all four real Task-73-
compatible expansions to `TermFormulaChecker`, and require one terminal-RHS-
anchored builtin-object identity, `BindingId(0)`, source-order use ordinal 1,
one `Inferred` term, and one fact-free `Checked` type assertion without general
reachability or object/set coercion. Exact source guards, independent
definition/four-link corruption, withheld-family near misses, and a real
frontend/resolver sidecar protect the slice; mode declaration acceptance/
inhabitation, formula-side local-mode asserted heads, general reachability/
widening/`qua`, object/set coercion, truth/facts, closure/order, theorem/proof/
Core/VC, and broader semantics remain deferred. The active type-elaboration
runner contains 102 cases after the production route and real sidecar pass.

Task 152 adds the exact four-edge set-terminal local-mode-chain reserved-
variable normalized-reflexive type assertion source. The runner must retain the
raw `TooDeepFourEdgeModeTypeAssertion` subject result and independent formula-
side builtin-set asserted source, supply all five real Task-74-compatible
expansions to `TermFormulaChecker`, and require one terminal-RHS-anchored
builtin-set identity, `BindingId(0)`, source-order use ordinal 1, one
`Inferred` term, and one fact-free `Checked` type assertion without general
reachability. Exact source guards, independent definition/five-link corruption,
withheld-family near misses, and a real frontend/resolver sidecar must protect
the slice; mode declaration acceptance/inhabitation, formula-side local-mode
asserted heads, general reachability/widening/`qua`, truth/facts, closure/order,
theorem/proof/Core/VC, and broader semantics remain deferred. The production
route and real sidecar pass, so the active type-elaboration runner contains 103
cases.

Task 153 adds the exact four-edge object-terminal local-mode-chain reserved-
variable normalized-reflexive type assertion source. The runner must retain the
raw `TooDeepFourEdgeObjectModeTypeAssertion` subject result and independent
formula-side builtin-object asserted source, supply all five real Task-74-
compatible expansions to `TermFormulaChecker`, and require one terminal-RHS-
anchored builtin-object identity, `BindingId(0)`, source-order use ordinal 1,
one `Inferred` term, and one fact-free `Checked` type assertion without general
reachability or object/set coercion. Exact source guards, independent
definition/five-link corruption, withheld-family near misses, and a real
frontend/resolver sidecar must protect the slice; mode declaration acceptance/
inhabitation, formula-side local-mode asserted heads, general reachability/
widening/`qua`, object/set coercion, truth/facts, closure/order, theorem/proof/
Core/VC, and broader semantics remain deferred. The production route and real
sidecar pass, so the active type-elaboration runner contains 104 cases.

Task 154 adds the test-first exact three-edge set-terminal local-mode-chain
reserved-variable equality source. The runner must retain four raw
`OuterThreeEdgeModeEquality` result/expected inputs, resolve both `z` operands
independently to `BindingId(0)` at ordinals 1 and 2, supply all four real
Task-73-compatible expansions to `TermFormulaChecker`, and require one
terminal-RHS builtin-set identity, two `Inferred` variables, and one fact/
deferred-free `Checked` equality. Exact source, independent definition/radix/
expansion corruption, withheld-family near misses, and a real frontend/resolver
sidecar must protect the slice; mode declaration acceptance/inhabitation,
equality truth/facts, closure/order, theorem/proof/Core/VC, and broader
semantics remain deferred. The production route, full near-miss/corruption
matrix, and real frontend/resolver sidecar now pass, so the active type-
elaboration runner contains 105 cases.

Task 155 adds the test-first exact three-edge object-terminal local-mode-chain
reserved-variable equality source. The runner must retain four raw
`OuterThreeEdgeObjectModeEquality` result/expected inputs, resolve both `z`
operands independently to `BindingId(0)` at ordinals 1 and 2, supply all four
real Task-73-compatible expansions to `TermFormulaChecker`, and require one
terminal-RHS builtin-object identity, two `Inferred` variables, and one fact/
deferred-free `Checked` equality. Exact source, independent definition/radix/
expansion corruption, withheld-family near misses, and a real frontend/resolver
sidecar must protect the slice; mode declaration acceptance/inhabitation,
object/set coercion, equality truth/facts, closure/order, theorem/proof/Core/VC,
and broader semantics remain deferred. The production route, full near-miss/
corruption matrix, and real frontend/resolver sidecar now pass, so the active
type-elaboration runner contains 106 cases.

Task 156 adds the test-first exact three-edge set-terminal local-mode-chain
reserved-variable inequality source. The runner must retain four raw
`OuterThreeEdgeModeInequality` result/expected inputs, resolve both `z` operands
independently to `BindingId(0)` at ordinals 1 and 2, supply all four real Task-
73-compatible expansions to `TermFormulaChecker`, and require one terminal-RHS
builtin-set identity, two `Inferred` variables, and one fact/deferred-free pre-
desugaring `Checked` inequality. Exact source, independent definition/radix/
expansion corruption, withheld-family near misses, and a real frontend/resolver
sidecar must protect the slice; mode declaration acceptance/inhabitation,
inequality desugaring, truth/facts, closure/order, theorem/proof/Core/VC, and
broader semantics remain deferred. The production route, full near-miss/
corruption matrix, and real frontend/resolver sidecar now pass, so the active
type-elaboration runner contains 107 cases.

Task 157 adds the exact three-edge object-terminal local-mode-chain reserved-
variable inequality source. The runner retains four raw
`OuterThreeEdgeObjectModeInequality` result/expected inputs, resolves both `z`
operands independently to `BindingId(0)` at ordinals 1 and 2, supplies all four
real Task-73-compatible expansions to `TermFormulaChecker`, and requires one
terminal-RHS builtin-object identity, two `Inferred` variables, and one fact/
deferred-free pre-desugaring `Checked` inequality. Exact source, independent
definition/radix/expansion corruption, withheld-family near misses, and a real
frontend/resolver sidecar must protect the slice; mode declaration acceptance/
inhabitation, object/set coercion, inequality desugaring, truth/facts, closure/
order, theorem/proof/Core/VC, and broader semantics remain deferred. The
fixture, expectation, trace row, production route, full near-miss/corruption
matrix, and real frontend/resolver sidecar now guard the active contract, so
the active type-elaboration runner contains 108 cases.

Task 158 adds the exact active three-edge set-terminal local-mode-chain left
reserved-variable membership source. The runner must retain the raw
`OuterThreeEdgeModeMembership` left result and independent explicit-set right
result/sole expected input, keep the left expected input absent, resolve `x/y`
to `BindingId(0/1)` at ordinals 2/3, and supply all four real task-73-compatible
expansions to `TermFormulaChecker`. The active contract requires one terminal-
RHS builtin-set identity, two `Inferred` variables, one fact/deferred-free
`Checked` membership, and exactly one right-owned expected-type constraint.
Exact source and independent definition/radix/expansion corruption guards are
required; mode declaration acceptance/inhabitation, membership truth/facts,
closure/order, theorem/proof/Core/VC, object-terminal behavior, and broader
semantics remain deferred. The fixture, expectation, trace row, production
route, full near-miss/corruption matrix, and real frontend/resolver sidecar now
guard the contract, so the active type-elaboration runner contains 109 cases.

Task 159 adds the exact active distinct-binding shared-reserve membership
source
`reserve x, y for set; theorem DistinctReservedVariableMembershipPayloadBoundary: x in y;`.
The runner must retain distinct `BindingId(0/1)` lookups at ordinals 2/3 and one
shared written set range across both bindings and the left-result/right-result/
right-expected roles, keep the left expected input absent, intern all three
roles to one shared-source-anchored builtin-set identity, and require two
`Inferred` variables plus one fact/deferred-free `Checked` membership with
exactly one right-owned constraint. Exact guards, the corruption/near-miss
matrix, and a real frontend/resolver sidecar now guard the contract, so the
active type-elaboration runner contains 110 cases. Truth/facts, closure/order,
theorem/proof/Core/VC, separate declarations, non-set types, and broader source
shapes remain deferred.

Task 160 adds the exact active distinct-binding shared-reserve inequality
source `reserve x, y for set; theorem
DistinctReservedVariableInequalityPayloadBoundary: x <> y;`. The runner must
retain distinct `BindingId(0/1)` lookups at ordinals 2/3 and one shared written
set range across both bindings and all four operand-owned result/expected
roles, intern those roles to one shared-source-anchored builtin-set identity,
and require two `Inferred` variables plus one fact/deferred-free pre-desugaring
`Checked` inequality with two ordered constraints. Exact route guards, the
corruption/near-miss matrix, and a real frontend/resolver sidecar now guard the
contract, so the active type-elaboration runner contains 111 cases. Desugaring/
truth/facts, closure/order, theorem/proof/Core/VC, separate declarations,
non-set types, and broader source shapes remain deferred.

Task 161 adds the exact active multiple-reserve-declaration inequality
source `reserve x for set; reserve y for set; theorem
MultipleReserveDeclarationInequalityPayloadBoundary: x <> y;`. The runner must
retain `BindingId(0/1)` at ordinals 2/3 and distinct written ranges across the
two operand result/expected pairs, intern all four roles to one canonical
builtin-set identity anchored at the earlier `x` range, and require two
`Inferred` variables plus one fact/deferred-free pre-desugaring `Checked`
inequality with two ordered constraints. Exact route guards, corruption/near-
miss coverage, and a real sidecar now guard the contract, so active type-
elaboration contains 112 cases. Shared ranges, non-set types, desugaring/truth/facts,
closure/order, theorem/proof/Core/VC, and broader shapes remain deferred.

Task 162 adds the exact active multiple-reserve-declaration membership
source `reserve x for set; reserve y for set; theorem
MultipleReserveDeclarationMembershipPayloadBoundary: x in y;`. The runner must
retain `BindingId(0/1)` at ordinals 2/3, the first written range on the left
result, and the second on the right result plus sole right expected input, with
no left expected input. All three roles must intern to one canonical builtin-
set identity anchored at the earlier `x` range before two `Inferred` variables
and one fact/deferred-free `Checked` membership with exactly one right-owned
constraint. Exact route guards, corruption/near-miss coverage, and a real
frontend/resolver sidecar now guard the contract, so active type-elaboration
contains 113 cases. Shared ranges, non-set types, membership truth/facts,
closure/order, theorem/proof/Core/VC, and broader shapes remain deferred.

Task 163 records the active exact three-edge local-object-mode membership
source. The production runner must accept only the four-definition object-
terminal chain plus ordered outer-mode/set reserves and the exact `x in y`
label; consume all four real expansions; retain raw left and explicit-set right
provenance; resolve `BindingId(0/1)` at ordinals 2/3; and require two normalized
identities, no left expected input, two `Inferred` variables, and one fact/
deferred-free `Checked` membership with exactly one right-owned constraint.
Matched-output corruption, every definition-link near miss, and a real
frontend/resolver sidecar fail closed around active runner 114.
Object/set coercion, truth/facts, closure/order, theorem/proof/Core/VC, other
depths, and broader shapes remain deferred.

Task 164 records the active exact four-edge local-mode membership source.
The production runner must accept only the five-definition set-terminal chain
plus ordered outermost-mode/set reserves and the exact `x in y` label; consume
all five real expansions; retain raw left and explicit-set right provenance;
resolve `BindingId(0/1)` at ordinals 2/3; and require one terminal-set-RHS
identity, no left expected input, two `Inferred` variables, and one fact/
deferred-free `Checked` membership with exactly one right-owned constraint.
Matched-output corruption, every definition-link/order/depth near miss, and a
real frontend/resolver sidecar must fail closed. Truth/facts, closure/order,
theorem/proof/Core/VC, object-terminal behavior, other depths, and broader
shapes remain deferred. The exact route, full corruption/near-miss matrix, and
real sidecar now guard active runner 115.

Task 165 records the active exact four-edge local-object-mode membership
source. The production runner must accept only the five-definition object-
terminal chain plus ordered outermost-mode/set reserves and the exact `x in y`
label; consume all five real expansions; retain raw left and explicit-set right
provenance; resolve `BindingId(0/1)` at ordinals 2/3; and require distinct
terminal-object-RHS and explicit-set identities, no left expected input, two
`Inferred` variables, and one fact/deferred-free `Checked` membership with
exactly one right-owned constraint. Matched-output corruption, every definition-
link/order/depth near miss, and a real frontend/resolver sidecar must fail
closed. Truth/facts, object/set coercion, closure/order, theorem/proof/Core/VC,
other depths, and broader shapes remain deferred. Production routing, full
guards, and the real sidecar now protect active runner 116.

Task 166 records the active exact four-edge local-mode equality source. The
production runner must accept only the five-definition set-terminal chain, one
outermost-mode reserve, and the exact `z = z` label; consume all five real
expansions; retain four raw result/expected inputs; resolve `BindingId(0)` at
ordinals 1/2; and require one terminal-set-RHS identity, two `Inferred`
variables, one fact/deferred-free `Checked` equality, and two ordered operand-
owned expected constraints. Matched-output
corruption, every definition/link/order/depth near miss, and a real frontend/
resolver sidecar must fail closed. Declaration acceptance/inhabitation, truth/
facts, closure/order, theorem/proof/Core/VC, object-terminal behavior, other
depths, and broader shapes remain deferred. Production routing, full guards,
and the real sidecar now protect active runner 117.

Task 167 records the active exact four-edge local-object-mode equality
source. The production runner must accept only the five-definition object-
terminal chain, one outermost-mode reserve, and the exact `z = z` label;
consume all five real expansions; retain four raw result/expected inputs;
resolve `BindingId(0)` at ordinals 1/2; and require one terminal-object-RHS
identity, two `Inferred` variables, one fact/deferred-free `Checked` equality,
and two ordered operand-owned expected constraints without object/set
coercion. Matched-output corruption, every definition/link/order/depth near
miss, and a real frontend/resolver sidecar must fail closed. Declaration
acceptance/inhabitation, truth/facts, closure/order, theorem/proof/Core/VC,
set-terminal behavior, other depths, and broader shapes remain deferred. The
production route, full guard matrix, and real sidecar now protect active runner
118.

Task 168 records the active exact four-edge local-mode inequality source. The
production runner must accept only the five-definition set-terminal chain, one
outermost-mode reserve, and the exact `z <> z` label; consume all five real
expansions; retain four raw result/expected inputs; resolve `BindingId(0)` at
ordinals 1/2; and require one terminal-set-RHS identity, two `Inferred`
variables, one fact/deferred-free pre-desugaring `Checked` inequality, and two
ordered operand-owned expected constraints. Matched-output corruption, every
definition/link/order/depth near miss, and a real frontend/resolver sidecar
must fail closed. Declaration acceptance/inhabitation, inequality desugaring/
truth/facts, closure/order, theorem/proof/Core/VC, object-terminal behavior,
other depths, and broader shapes remain deferred. The fixture, expectation,
six trace backlinks, production route, full guard matrix, and real sidecar now
protect active runner 119.

Task 169 records the active exact four-edge local-object-mode inequality
source. The production runner must accept only the five-definition object-
terminal chain, one outermost-mode reserve, and the exact `z <> z` label;
consume all five real expansions; retain four raw result/expected inputs;
resolve `BindingId(0)` at ordinals 1/2; and require one terminal-object-RHS
identity, two `Inferred` variables, one fact/deferred-free pre-desugaring
`Checked` inequality, and two ordered operand-owned expected constraints
without object/set coercion. Matched-output corruption, every definition/link/
order/depth near miss, and a real frontend/resolver sidecar must fail closed.
Declaration acceptance/inhabitation, inequality desugaring/truth/facts,
closure/order, theorem/proof/Core/VC, set-terminal behavior, other depths, and
broader shapes remain deferred. The fixture, expectation, six trace backlinks,
production route, full guard matrix, and real sidecar now protect active runner
120.

Task 172 records the test-first exact local-mode long-chain equality source.
The production runner must accept only the seven-definition set-terminal chain,
one `ChainMode6` reserve, and the exact `z = z` label; consume all seven real
AST-derived expansions; retain four raw `ChainMode6` result/expected inputs;
resolve `BindingId(0)` at ordinals 1/2; and require one terminal-`BaseMode`-RHS
builtin-set identity, two `Inferred` variables, one fact/deferred-free
`Checked` equality, and two ordered operand-owned expected constraints. Full
matched-output, definition/link/order/depth/recovery/context/parameterization/
terminal/reserve/formula/symbol and expansion-corruption guards plus a real
frontend/resolver sidecar fail closed. Declaration acceptance/
inhabitation, truth/facts, closure/order, theorem/proof/Core/ControlFlow/VC,
imported/attributed/argument-bearing or other chain shapes, and general
unbounded semantics remain deferred. Production routing, full guards, and the
real sidecar now protect active runner 121.

Task 173 records the test-first long-chain inequality sibling. The production
runner must accept only the same seven definitions and `ChainMode6` reserve
with exact `z <> z`; consume seven real expansions; retain four raw roles;
resolve ordinal 1/2 `BindingId(0)`; and require one terminal-`BaseMode`-RHS
identity, two `Inferred` variables, two ordered constraints, and one fact/
deferred-free pre-desugaring `Checked` inequality. Task 172's full guard matrix
and real sidecar breadth now protect active runner 122. Desugaring/truth/facts
and downstream or general semantics remain deferred.

Task 174 records the test-first long-chain membership sibling. The production
runner must accept only the same seven definitions, ordered `ChainMode6`/`set`
reserves, and exact `x in y`; consume seven real expansions; retain the raw
left plus independent right result/sole expected input; resolve ordinal 2/3
`BindingId(0/1)`; and require one terminal-`BaseMode`-RHS identity, no left
expected input, two `Inferred` variables, one right-owned constraint, and one
fact/deferred-free `Checked` membership. Task 172's full structural guard
matrix plus membership-specific corruption and a real sidecar fail closed.
Truth/facts and downstream or general semantics remain deferred. Production
routing, full guards, and the real sidecar now protect active runner 123.

Task 175 records the test-first long-chain type-assertion sibling. The
production runner must accept only the same seven definitions, one
`ChainMode6` reserve, and exact `x is set`; consume seven real expansions;
retain raw subject and independent formula-side builtin-set asserted inputs;
resolve ordinal 1 `BindingId(0)`; and require one terminal-`BaseMode`-RHS
identity, one `Inferred` variable, and one fact/deferred-free normalized-
reflexive `Checked` type assertion without general reachability. Task 172's
full structural guard matrix plus type-assertion-specific corruption and a real
sidecar must fail closed. Widening/`qua`, truth/facts, and downstream or general
semantics remain deferred. The test-first row, production support, full guards,
and the real sidecar now protect active runner 124.

Task 176 records the test-first builtin-object-terminal long-chain equality
sibling. The production runner must accept only the exact seven definitions,
one `ChainObjectMode6` reserve, and exact `z = z`; consume seven real
expansions; retain four raw result/expected inputs; resolve ordinal 1/2
`BindingId(0)`; and require one terminal-`BaseObjectMode`-RHS identity, two
`Inferred` terms, two ordered operand-owned constraints, and one fact/deferred-
free `Checked` equality without object/set coercion. Task 172's shared full
structural guard matrix plus object-terminal/matched-output corruption and a
real sidecar must fail closed. Truth/facts and downstream or general semantics
remain deferred. The test-first row, production support, full guards, and the
real sidecar now protect active runner 125.

Task 177 records the test-first builtin-object-terminal long-chain inequality
sibling. The production runner must accept only the exact seven definitions,
one `ChainObjectMode6` reserve, and exact `z <> z`; consume seven real
expansions; retain four raw result/expected inputs; resolve ordinal 1/2
`BindingId(0)`; and require one terminal-`BaseObjectMode`-RHS identity, two
`Inferred` terms, two ordered operand-owned constraints, and one fact/deferred-
free pre-desugaring `Checked` inequality without object/set coercion. Task 172's
shared full structural guard matrix plus object-terminal/matched-output
corruption and a real sidecar fail closed. Inequality desugaring,
truth/facts, and downstream or general semantics remain deferred. The test-first
row, production support, full guards, and the real sidecar now protect active
runner 126.

Task 178 supports the builtin-object-terminal long-chain left-
membership sibling. The production runner must accept only the exact seven
definitions, ordered `x`/`y` reserves for `ChainObjectMode6`/explicit `set`, and
exact `x in y`; consume seven real expansions; retain the raw left and
independent right result/sole expected input; resolve ordinal 2/3
`BindingId(0/1)`; and require distinct terminal-object-RHS and explicit-set
identities, no left expected input, two `Inferred` terms, one right-owned
constraint, and one fact/deferred-free `Checked` membership without object/set
coercion. Task 172's shared full structural guard matrix plus membership/object-
specific corruption and a real sidecar fail closed. Truth/facts and downstream/
general semantics remain deferred. The fixture, production support, and guards
protect active runner 127.

Task 179 supports the builtin-object-terminal long-chain type-assertion
sibling. The production runner accepts only the exact seven
definitions, one `x` reserve for `ChainObjectMode6`, and exact `x is object`;
consumes seven real expansions; retains the raw subject and independent formula-
side builtin-object asserted input; resolves ordinal 1 `BindingId(0)`; and
requires one terminal-object-RHS identity, one `Inferred` term, and one fact/
deferred-free normalized-reflexive `Checked` type assertion without general
reachability or object/set coercion. Task 172's shared full structural guard
matrix and Task 153's real object consumer/source near misses are reused; Task
175's matched-output guards reject a builtin-set asserted head and corrupted raw
subject provenance, and a real sidecar fails closed.
Truth/facts, acceptance, downstream/general
semantics remain deferred. The fixture, production support, full guards, and
real sidecar protect active runner 128.

Task 180 supports only the standalone formula leaf
`theorem SourceDerivedContradictionConstantBoundary: contradiction;`. The
production route adds an exact extractor that preserves the real leaf site/
range and module-root context and passes `FormulaKind::Contradiction` to the
existing checker consumer without a deferred reason. It requires one `Checked`
formula and empty terms, asserted type, expected constraints, candidates,
facts, deferred reasons, and diagnostics. Wrong labels/constants, status or
recovery markers, extra items, and duplicate theorems remain on their existing
paths; a real frontend/resolver sidecar protects active runner 129. This is
formula type/well-formedness only, not falsehood/fact publication, theorem
acceptance, proof-goal closure, child-graph extraction, `formula_statement`,
proof, CoreIr, ControlFlowIr, or VC coverage.

Task 182 adds the first formula-side local-mode asserted-head pass case. The
production route accepts only one `definition` block containing `mode
LocalModeAssertedHeadDef: LocalModeAssertedHead is set;`, one matching reserve,
and exact `x is LocalModeAssertedHead`.
It retains distinct raw reserve-subject and formula-side asserted sites/
ranges for the same resolved mode symbol, consumes one real expansion, resolves
ordinal 1 to `BindingId(0)`, and requires three known type entries interned to
one terminal-RHS builtin-set identity, one `Inferred` variable, and one fact/
deferred-free normalized-reflexive `Checked` type assertion. Exact/near-miss,
matched-output corruption, route-order, and real frontend/resolver-sidecar
tests fail closed for collapsed provenance, other asserted heads, and
broader definitions/items. The new active pass case raises the runner from 129
to 130; the real sidecar protects that case. It does not credit declaration
acceptance/inhabitation,
widening/`qua`, truth/facts, theorem/proof/CoreIr/ControlFlowIr/VC, child graphs,
other asserted-head families, or general semantics.

Task 183 adds the direct object-terminal formula-side local-mode asserted-head
pass case. The production route accepts only one definition block with
`mode LocalObjectModeAssertedHeadDef: LocalObjectModeAssertedHead is object;`,
one matching reserve, and exact `x is LocalObjectModeAssertedHead`. It retains
distinct raw reserve-subject and formula-side asserted sites/ranges for the
same resolved symbol, consumes one real expansion, resolves ordinal 1 to
`BindingId(0)`, and requires three known type entries interned to one terminal-
RHS builtin-object identity, one `Inferred` variable, and one fact/deferred-free
normalized-reflexive `Checked` type assertion without general reachability or
object/set coercion. Exact/near-miss, matched-output corruption, route-order,
and real frontend/resolver-sidecar tests fail closed for set terminals,
builtin/other asserted heads, chains, attributes/arguments, recovery, extra
items, and collapsed provenance. The new active pass case raises the runner
from 130 to 131. Declaration acceptance/inhabitation, truth/facts, theorem/
proof/CoreIr/ControlFlowIr/VC, other asserted-head families, and general
semantics remain uncredited.

Task 184 adds the exact one-edge set-terminal same-outer-mode asserted-head
pass case. The production route accepts only two ordered definition blocks
with `mode BaseModeAssertedHeadDef: BaseModeAssertedHead is set;` and `mode
ChainModeAssertedHeadDef: ChainModeAssertedHead is BaseModeAssertedHead;`, one
matching outer-mode reserve, and exact `x is
ChainModeAssertedHead`. It retains distinct raw reserve-subject and formula-
side asserted sites/ranges for the same resolved outer symbol, consumes both
real expansions, resolves ordinal 1 to `BindingId(0)`, and requires three known
type entries interned to one terminal-base-definition-RHS builtin-set identity,
one `Inferred` variable, and one fact/deferred-free normalized-reflexive
`Checked` type assertion without general reachability. Exact/near-miss,
matched-output corruption, route-order, and real frontend/resolver-sidecar
tests fail closed for wrong links/terminals/order/depth, builtin/base/other
asserted heads, attributes/arguments, recovery, extra items, and collapsed
provenance. The active pass count rises from 131 to 132.
Declaration acceptance/inhabitation, widening/`qua`, truth/facts, closure/order,
theorem/proof/CoreIr/ControlFlowIr/VC, object/deeper/other asserted-head chains,
and general chain semantics remain uncredited.

Task 185 adds the exact one-edge object-terminal same-outer-mode asserted-head
pass case. The production route accepts only two ordered definition
blocks with `mode BaseObjectModeAssertedHeadDef: BaseObjectModeAssertedHead is
object;` and `mode ChainObjectModeAssertedHeadDef:
ChainObjectModeAssertedHead is BaseObjectModeAssertedHead;`, one matching outer-
mode reserve, and exact `x is ChainObjectModeAssertedHead`. It must retain
distinct raw reserve-subject and formula-side asserted ranges for the same
resolved outer symbol, consume both real expansions, resolve ordinal 1 to
`BindingId(0)`, and require three known type entries interned to one terminal-
base-definition-RHS builtin-object identity, one `Inferred` variable, and one
fact/deferred-free normalized-reflexive `Checked` type assertion without
general reachability, widening, `qua`, or object/set coercion. Exact/near-miss,
matched-output corruption, route-order, and real frontend/resolver-sidecar tests
must fail closed for wrong links/terminals/order/depth, builtin/base/other
asserted heads, attributes/arguments, imported provenance, recovery, extra
items, collapsed provenance, and builtin-set output corruption. Imported/
declaration/attribute, broader term/formula/child-
graph, truth/fact, theorem/proof/CoreIr/ControlFlowIr/VC, deeper/other asserted-
head, and general-chain coverage remain uncredited. Five shared trace backlinks
plus one dedicated row protect active count 133. No module layout update was
required.

Task 186 adds the exact two-edge set-terminal same-outer-mode asserted-head
pass case. The route accepts only ordered definitions
`BaseTwoEdgeModeAssertedHead -> set`, `MiddleTwoEdgeModeAssertedHead ->
BaseTwoEdgeModeAssertedHead`, and `OuterTwoEdgeModeAssertedHead ->
MiddleTwoEdgeModeAssertedHead`, a matching outer-mode reserve, and exact
`TwoEdgeLocalModeAssertedHeadPayloadBoundary: x is
OuterTwoEdgeModeAssertedHead`. It retains distinct reserve/asserted ranges for
the same symbol, consumes three real expansions, resolves ordinal 1 to
`BindingId(0)`, interns three known entries to one terminal-base-RHS builtin-set
identity, and requires one `Inferred` variable plus one fact/deferred-free
normalized-reflexive `Checked` assertion without reachability, widening, or
`qua`. Exact/near-miss, corruption, route-order, and real frontend/resolver-
sidecar guards reject structural chain failures, imported/ambiguous provenance,
collapsed provenance, and builtin-object corruption. Five shared plus one
dedicated trace row protect active count 134. Object/deeper/imported semantics,
declaration/attribute acceptance, broader terms/formulas/child graphs, truth/
facts, proof/CoreIr/ControlFlowIr/VC, and general chain semantics remain
uncredited. No module layout update is required.

Task 187 adds the exact two-edge object-terminal same-outer-mode asserted-head
pass case. The route accepts only ordered definitions
`mode BaseTwoEdgeObjectModeAssertedHeadDef: BaseTwoEdgeObjectModeAssertedHead is
object;`, `mode MiddleTwoEdgeObjectModeAssertedHeadDef:
MiddleTwoEdgeObjectModeAssertedHead is BaseTwoEdgeObjectModeAssertedHead;`, and
`mode OuterTwoEdgeObjectModeAssertedHeadDef: OuterTwoEdgeObjectModeAssertedHead
is MiddleTwoEdgeObjectModeAssertedHead;`, a matching outer reserve, and exact
`TwoEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
OuterTwoEdgeObjectModeAssertedHead`. It preserves distinct raw subject/asserted
ranges for the same local symbol, consumes three real expansions, resolves
ordinal 1 to `BindingId(0)`, interns three known entries to one base-definition-
RHS builtin-object identity, and requires one `Inferred` variable plus one fact/
deferred-free normalized-reflexive `Checked` assertion with no expected
constraints, reachability, widening, `qua`, or object/set coercion. Exact/near-
miss, corruption, route-order, and real frontend/resolver-sidecar guards reject
all non-exact link/depth/terminal/provenance shapes and `BuiltinSet` output
corruption, including wrong labels, attributed/argument-bearing formula-side
asserted heads, imported Base/Middle/Outer, and imported/ambiguous asserted
heads. Five shared plus one dedicated trace row protect active count 135.
Positive imported semantics, declaration/attribute acceptance, broader terms/
formulas/child graphs, truth/facts, implicit closure/order, theorem acceptance,
proof/CoreIr/ControlFlowIr/VC, and general chain semantics remain uncredited.
Step 5 remains active; Steps 6/7 remain deferred. No module layout update is
required.

Task 188 adds only the exact active builtin-object equality source `reserve x
for object; theorem ReservedObjectVariableEqualityPayloadBoundary: x = x;`.
The active route must reuse the real source-derived object reserve handoff and
the existing reserved-variable equality consumer, retain ordinal 1/2 local
lookups plus four distinct result/expected role sites on the written reserve
range, and accept only a single canonical builtin-object identity, two
`Inferred` variable terms, two ordered expected constraints, and one fact/
deferred-free `Checked` equality. Exact/near-miss/corruption and real frontend/
resolver-sidecar tests guard source shape, binding identity, lookup ordering,
role provenance, checker counts/status, constraints, canonical source, and
wrong builtin-set output. Five shared backlinks plus one dedicated trace row
protect active runner 136 without rebaselining an existing expectation. This
does not activate general object equality, coercion,
truth/facts, closure/order, theorem/proof acceptance, or downstream payloads.

Task 189 adds only the exact active builtin-object type-assertion source
`reserve x for object; theorem
ReservedObjectVariableTypeAssertionPayloadBoundary: x is object;`. The active
route must reuse the real source-derived object reserve handoff and the
existing reserved-variable type-assertion consumer, retain ordinal 1 local
lookup plus distinct reserve-subject result and formula-side asserted sites/
ranges, and accept only one reserve-anchored canonical builtin-object identity,
one `Inferred` variable term, three known type entries, no expected
constraints, and one fact/deferred-free `Checked` assertion. Exact/near-miss/
corruption and real frontend/resolver-sidecar tests must guard source shape,
binding identity, lookup order, raw input provenance, checker counts/status,
the absence of constraints, canonical source, and wrong builtin-set output.
Five shared backlinks plus one dedicated trace row must protect active runner
137 without rebaselining an existing expectation. This does not activate
reachability/widening/`qua`, object/set coercion, truth/facts, closure/order,
theorem/proof acceptance, or downstream payloads.

Task 190 adds only the exact active builtin-object inequality source `reserve x
for object; theorem ReservedObjectVariableInequalityPayloadBoundary: x <> x;`.
The active route must reuse the real source-derived object reserve handoff and
the existing reserved-variable inequality consumer, retain ordinal 1/2 local
lookups plus four distinct result/expected role sites on the written reserve
range, and accept only one canonical builtin-object identity, two `Inferred`
variable terms, six known type entries, two ordered expected constraints, and
one fact/candidate/diagnostic/deferred-free pre-desugaring `Checked`
inequality. Exact/near-miss/corruption and real frontend/resolver-sidecar tests
must guard source shape, binding identity, lookup ordering, role provenance,
checker counts/status, constraints, canonical source, and wrong builtin-set
output. Five shared backlinks plus one dedicated trace row must protect active
runner 138 without rebaselining an existing expectation. This does not
activate inequality desugaring/equality truth, object/set coercion, facts,
closure/order, theorem/proof acceptance, or downstream payloads.

Task 191 adds only the exact active distinct-binding shared-builtin-object
equality source `reserve x, y for object; theorem
DistinctReservedObjectVariableEqualityPayloadBoundary: x = y;`. The active
route composes the real one-item/two-binding shared-range reserve handoff with
the existing builtin-object equality consumer, retains ordinal 2/3 local
lookups plus four distinct result/expected role sites over the shared written
reserve range, and accepts only one reserve-range-anchored canonical builtin-
object identity, two `Inferred` variable terms, six known type entries, two
ordered expected constraints, and one fact/candidate/diagnostic/deferred-free
`Checked` equality. Exact/near-miss/corruption and real frontend/resolver-
sidecar tests guard source shape, distinct binding identity, lookup
ordering, shared-range role provenance, checker counts/status, constraints,
canonical source, and wrong builtin-set output. Five shared backlinks plus one
dedicated trace row protects active runner 139 without
rebaselining an existing expectation. This does not activate equality truth,
object/set coercion, facts, closure/order, theorem/proof acceptance, or
downstream payloads.

Task 192 is restricted to the exact active distinct-binding shared-builtin-
object inequality source `reserve x, y for object; theorem
DistinctReservedObjectVariableInequalityPayloadBoundary: x <> y;`. The active
route composes the real one-item/two-binding shared-range reserve handoff
with the existing pre-desugaring inequality consumer, retain ordinal 2/3 local
lookups plus four distinct result/expected role sites over the shared written
reserve range, and accept only one reserve-range-anchored canonical builtin-
object identity, two `Inferred` variable terms, six known type entries, two
ordered expected constraints, and one fact/candidate/diagnostic/deferred-free
`Checked` inequality. Exact/near-miss/corruption and real frontend/resolver-
sidecar tests guard source shape, distinct binding identity, lookup
ordering, shared-range role provenance, checker counts/status, constraints,
canonical source, and wrong builtin-set output. Five shared backlinks plus one
dedicated trace row protect active runner 140 without
rebaselining an existing expectation. This does not activate inequality
desugaring/equality truth, object/set coercion, facts, closure/order, theorem/
proof acceptance, or downstream payloads.

Task 193 is restricted to the exact active multiple-reserve-declaration
builtin-object equality source `reserve x for object; reserve y for object;
theorem MultipleObjectReserveDeclarationEqualityPayloadBoundary: x = y;`.
The active route composes the real two-item/two-binding/distinct-written-range
reserve handoff with the builtin-object equality consumer, retains ordinal 2/3
local lookups plus four distinct result/expected role sites over the two
binding-owned written ranges, and accepts only one canonical builtin-object
identity anchored at the earlier `x` range, two `Inferred` variable terms, six
known type entries, two ordered expected constraints, and one fact/candidate/
diagnostic/deferred-free `Checked` equality. Exact/near-miss/corruption and
real frontend/resolver-sidecar tests guard source shape, distinct bindings and
ranges, lookup ordering, raw role provenance, checker counts/status,
constraints, canonical source, route isolation, and wrong builtin-set output.
Five shared backlinks plus one dedicated trace row protect active runner 141
without rebaselining an existing expectation. This does not activate equality
truth, object/set coercion, facts, closure/order, theorem/proof acceptance,
shared-range shapes, or downstream payloads.

Task 194 is restricted to the exact active multiple-reserve-declaration
builtin-object inequality source `reserve x for object; reserve y for object;
theorem MultipleObjectReserveDeclarationInequalityPayloadBoundary: x <> y;`.
The active route composes Task 193's ordered two-item/two-binding/distinct-
written-object-range handoff with the pre-desugaring builtin-object inequality
consumer, retains ordinal 2/3 local lookups plus four distinct raw result/
expected roles over the two binding-owned written ranges, and accepts only one
canonical builtin-object identity anchored at the earlier `x` range, two
`Inferred` variable terms, six known type entries, two ordered expected
constraints, and one fact/candidate/diagnostic/deferred-free `Checked`
inequality. Exact/near-miss/corruption and real frontend/resolver-sidecar tests
guard source shape, distinct bindings and ordered ranges, lookup ordering, raw
role provenance, checker counts/status, constraints, canonical source, route
isolation, and wrong builtin-set output. Five shared backlinks plus one
dedicated trace row protect active runner 142 without rebaselining an existing
expectation. This does not activate inequality desugaring/equality truth,
object/set coercion, facts, closure/order, theorem/proof acceptance, shared-
range shapes, or downstream payloads.

Task 195 is restricted to the exact active three-edge set-terminal same-outer-
mode asserted-head source with four ordered definitions `Outer -> Middle ->
Inner -> Base -> set`, one outer-mode reserve, and
`ThreeEdgeLocalModeAssertedHeadPayloadBoundary: x is
OuterThreeEdgeModeAssertedHead;`. The active route consumes four real AST-
derived expansions and retains independent raw reserve-subject and formula-
side asserted-type sites/ranges for the same resolved outer symbol. It accepts
only ordinal 1 resolving to `BindingId(0)`, three known type entries normalizing
to one base-definition-RHS-anchored builtin-set identity, one `Inferred`
variable, zero expected constraints/candidates/facts/diagnostics/deferred
reasons, and one normalized-reflexive `Checked` assertion. Exact, structural,
provenance, corruption, immutable-output, route-isolation, and real frontend/
resolver-sidecar tests guard all four links and reject unrelated local,
imported, and ambiguous asserted heads. Five shared backlinks plus one
dedicated trace row protect active runner 143 without rebaselining an existing
expectation. This does not activate reachability/widening/`qua`, declaration or
theorem acceptance, truth/facts, closure/order, broader term/formula/child-
graph semantics, proof, or downstream IR.

Task 196 is restricted to the exact active three-edge object-terminal same-
outer-mode asserted-head source with four ordered definitions `Outer -> Middle
-> Inner -> Base -> object`, one outer-mode reserve, and
`ThreeEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
OuterThreeEdgeObjectModeAssertedHead;`. The active route consumes four real
AST-derived expansions and retains independent raw reserve-subject and formula-
side asserted-type sites/ranges for the same resolved outer symbol. It accepts
only ordinal 1 resolving to `BindingId(0)`, three known type entries normalizing
to one base-definition-RHS-anchored builtin-object identity, one `Inferred`
variable, zero expected constraints/candidates/facts/diagnostics/deferred
reasons, and one normalized-reflexive `Checked` assertion without object/set
coercion. Exact, structural, provenance, `BuiltinSet`/canonical corruption,
immutable-output, route-isolation, and real frontend/resolver-sidecar tests
guard all four links and reject unrelated local, imported, and ambiguous
asserted heads. Five shared backlinks plus one dedicated trace row protect
active runner 144 without rebaselining an existing expectation. This does not
activate reachability/widening/`qua`, declaration or theorem acceptance, truth/
facts, closure/order, broader term/formula/child-graph semantics, proof, or
downstream IR.

Task 197 is restricted to the exact active four-edge set-terminal same-
outermost-mode asserted-head source with five ordered definitions `TooDeep ->
Outer -> Middle -> Inner -> Base -> set`, one outermost-mode reserve, and
`FourEdgeLocalModeAssertedHeadPayloadBoundary: x is
TooDeepFourEdgeModeAssertedHead;`. The active route consumes five real AST-
derived expansions and preserves independent raw reserve-subject and formula-
side asserted-type sites/ranges for the same resolved outermost symbol. It
accepts only ordinal 1 resolving to `BindingId(0)`, three known type entries
normalizing to one base-definition-RHS-anchored builtin-set identity, one
`Inferred` variable, zero expected constraints/candidates/facts/diagnostics/
deferred reasons, and one normalized-reflexive `Checked` assertion. Exact,
full-reorder, connected-deeper, structural, provenance, `BuiltinObject`/
canonical corruption, immutable-output, route-isolation, and real frontend/
resolver-sidecar tests guard all five links and reject unrelated local,
imported, and ambiguous asserted heads. Five shared backlinks plus one
dedicated trace row protect active runner 145 without rebaselining an existing
expectation. This does not activate reachability/widening/`qua`, declaration or
theorem acceptance, truth/facts, closure/order, broader term/formula/child-
graph semantics, proof, or downstream IR.

Task 198 is restricted to the exact active four-edge object-terminal same-
outermost-mode asserted-head source with five ordered definitions `TooDeep ->
Outer -> Middle -> Inner -> Base -> object`, one outermost-mode reserve, and
`FourEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
TooDeepFourEdgeObjectModeAssertedHead;`. The active route consumes five real
AST-derived expansions and preserves independent raw reserve-subject and
formula-side asserted-type sites/ranges for the same resolved outermost
symbol. It accepts only ordinal 1 resolving to `BindingId(0)`, three known type
entries normalizing to one base-definition-RHS-anchored builtin-object
identity, one `Inferred` variable, zero expected constraints/candidates/facts/
diagnostics/deferred reasons, and one normalized-reflexive `Checked` assertion
without object/set coercion. Exact, full-reorder, connected-deeper, structural,
provenance, `BuiltinSet`/canonical corruption, immutable-output, route-
isolation, and real frontend/resolver-sidecar tests guard all five links and
reject unrelated local, imported, and ambiguous asserted heads. Five shared
backlinks plus one dedicated trace row protect active runner 146 without
rebaselining an existing expectation. This does not activate reachability/
widening/`qua`, declaration or theorem acceptance, truth/facts, closure/order,
broader term/formula/child-graph semantics, proof, or downstream IR.

Task 199 is restricted to the exact active seven-expansion set-terminal same-
`ChainMode6` asserted-head source with `BaseMode -> set`, six ordered links
through `ChainMode6 -> ChainMode5`, one `ChainMode6` reserve, and
`LongLocalModeAssertedHeadPayloadBoundary: x is ChainMode6;`. The active route
consumes seven real AST-derived expansions and preserves independent raw
reserve-subject and formula-side asserted-type sites/ranges for the same
resolved symbol. It accepts only ordinal 1 resolving to `BindingId(0)`, three
known type entries normalizing to one `BaseModeDef` RHS-anchored builtin-set
identity, one `Inferred` variable, zero expected constraints/candidates/facts/
diagnostics/deferred reasons, and one normalized-reflexive `Checked` assertion.
Exact, per-link removal/reorder, complete-reverse, connected-eighth,
structural, provenance, `BuiltinObject`/canonical corruption, immutable-output,
route-isolation, and real frontend/resolver-sidecar tests guard all seven links
and reject unrelated local, imported, and ambiguous asserted heads. Five shared
backlinks plus one dedicated trace row protect active runner 147 without
rebaselining an existing expectation. This does not activate object-terminal/
other-depth/imported/attributed/argument-bearing/other asserted heads,
reachability/widening/`qua`, declaration or theorem acceptance, truth/facts,
closure/order, broader term/formula/child-graph semantics, proof, or downstream
IR.

Task 200 is restricted to the exact active seven-expansion object-terminal same-
`ChainObjectMode6` asserted-head source with `BaseObjectMode -> object`, six
ordered links through `ChainObjectMode6 -> ChainObjectMode5`, one
`ChainObjectMode6` reserve, and
`LongLocalObjectModeAssertedHeadPayloadBoundary: x is ChainObjectMode6;`. The
active route consumes seven real AST-derived expansions and preserves
independent raw reserve-subject and formula-side asserted-type sites/ranges for
the same resolved symbol. It accepts only ordinal 1 resolving to `BindingId(0)`,
three known type entries normalizing to one `BaseObjectModeDef` RHS-anchored
builtin-object identity, one `Inferred` variable, zero expected constraints/
candidates/facts/diagnostics/deferred reasons, and one normalized-reflexive
`Checked` assertion without object/set coercion. Exact, per-link removal/
reorder, complete-reverse, connected-eighth, structural, provenance,
`BuiltinSet`/canonical corruption, immutable-output, route-isolation, and real
frontend/resolver-sidecar tests guard all seven links and reject unrelated
local, imported, and ambiguous asserted heads. Five shared backlinks plus one
dedicated trace row protect active runner 148 without rebaselining an existing
expectation. This does not activate set-terminal/other-depth/imported/
attributed/argument-bearing/other asserted heads, reachability/widening/`qua`,
declaration or theorem acceptance, truth/facts, closure/order, broader term/
formula/child-graph semantics, proof, or downstream IR.

Task 120 adds the matching exact membership pass case
`reserve x for set; theorem ReservedVariableMembershipPayloadBoundary: x in x;`.
The active runner shares Task 119's match-before-build and independent
source-order lookup path, but requires membership's exact payload shape: two
known `set` variable results, only the right operand's expected-`set`
constraint, three exact source-anchored roles, one `Checked` membership, and
empty candidates/facts/deferred reasons/diagnostics. Matched-source construction
or invariant drift reports
`type_elaboration.checker.reserved_variable_membership.invalid_payload`; other
near-misses remain on the extraction gap. A real frontend/resolver unit test
observes the active sidecar payload. This is well-formedness coverage only, not
membership truth, a recorded fact, implicit closure, theorem acceptance, or a
proof/Core/ControlFlow/VC promotion.

Task 121 adds the exact inequality sibling
`reserve x for set; theorem ReservedVariableInequalityPayloadBoundary: x <> x;`.
The checker-owned inequality API supplies two expected-type slots while task
119 supplies the real reserve binding/use producer; task 107's numeral
inequality bridge remains partial without expected types. The shared active
producer requires two linked result roles, two linked expected roles, two
`Inferred` variables, and one fact-free pre-desugaring `Checked` inequality. A
task-specific invalid-payload key, full near-miss matrix, and real
frontend/resolver payload test guard the slice. No inequality desugaring,
truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr, or VC is credited.

Task 122 adds the exact type-assertion sibling
`reserve x for set; theorem ReservedVariableTypeAssertionPayloadBoundary: x is set;`.
The active producer combines task 119's real reserve lookup/result input with
task 109's formula-side asserted-type AST input, preserves their distinct
pre-normalization source anchors, and requires the checker to admit only their
normalized reflexive identity. The output has one `Inferred` variable, one
fact-free `Checked` type assertion, and empty candidates/deferred reasons/
diagnostics. Known non-identical types use
`checker.formula.external.type_assertion_reachability_payload` and remain
partial. A task-specific invalid-payload key, the enumerated near-miss matrix,
and a real frontend/resolver payload test guard the slice. General
reachability/widening/`qua`, attributes, truth/facts, implicit closure, theorem
acceptance, proof, CoreIr, ControlFlowIr, and VC are not credited.

Task 109 supersedes the exact builtin type-assertion sidecar from task 102:
`BuiltinTypeAssertionPayloadBoundary: 1 is set` is executable through the
active `type_elaboration` runner and now passes source-derived checker
`TermInput`, `FormulaInput`, and asserted builtin `set` `TypeExpressionInput`
payloads before failing closed on missing numeric type payloads and partial
formula checking. It still does not satisfy the deferred `formula_statement`
runner obligation and does not credit broader asserted type payload extraction,
type-assertion semantic checking, facts, theorem acceptance, CoreIr,
ControlFlowIr, VC, or proof payloads.

Task 113 refines the same row by superseding task 103 for the exact imported
attribute assertion theorem formula using `parser.type_fixtures` `empty`. The
active `type_elaboration` runner validates imported `empty` provenance and
passes real source-derived checker term/formula payloads before failing closed
on missing numeric type payload, missing formula/attribute semantic payload,
and partial formula checking. It does not satisfy the deferred
`formula_statement` runner obligation and does not credit imported module AST
extraction, attribute-chain semantic payloads, theorem-formula `AttributeInput`
payloads, term inference, attribute admissibility/semantic checking, formula
checking, facts, theorem acceptance, CoreIr, ControlFlowIr, VC, or proof
payloads.

Task 114 refines the same row by superseding task 104 for the exact
attribute-level `non empty` imported attribute assertion theorem formula using
`parser.type_fixtures` `empty`. It satisfies only the active
`type_elaboration` checker handoff for that source: real source-derived
term/formula payloads are passed, then the run fails closed on missing numeric
type payload, missing formula/attribute semantic payload, and partial formula
checking. It does not satisfy the deferred `formula_statement` runner
obligation and does not credit imported module AST extraction, negated
attribute-chain semantic payloads, theorem-formula `AttributeInput` payloads,
term inference, negated attribute admissibility/semantic checking, formula
checking, facts, theorem acceptance, CoreIr, ControlFlowIr, VC, or proof
payloads. In the row above, the theorem formula boundary entry now also covers
task 114's exact attribute-level non-empty imported attribute assertion checker
bridge.

Task 111 supersedes the task-105 set-enumeration theorem formula boundary only
for the exact `SetEnumerationPayloadBoundary: {1, 2} = {1, 2}` source. The
active `type_elaboration` runner now passes real checker payloads for four
numeral item terms, two set-enumeration terms, and the builtin equality formula,
then fails closed on missing numeric type payloads, missing set-enumeration
result-type payloads, and partial formula checking. It does not satisfy
the deferred `formula_statement` runner obligation and does not credit broader
set-enumeration payloads, term inference, equality/formula checking, facts,
theorem acceptance, CoreIr, ControlFlowIr, VC, or proof payloads.

Task 112 refines the same row by superseding task 99 only for the exact
connective/quantifier theorem formula source. The active `type_elaboration`
runner now passes real checker `FormulaInput` shells for implication, universal
quantification, and negation, then fails closed on missing formula/quantifier
payloads. Task 117 extends only that exact source by also passing both
source-derived `contradiction` constants as `FormulaKind::Contradiction`
payloads before the same missing formula payload diagnostic. It does not
satisfy the deferred `formula_statement` runner obligation and does not credit
formula constant semantics, child-formula graph payloads, quantifier
binder/context payloads, formula checking, facts, theorem acceptance, CoreIr,
ControlFlowIr, VC, or proof payloads.

Task 88 refines the same row: proof-block theorem sources are executable
through the active `type_elaboration` runner only as extraction-gap boundaries.
They do not satisfy the deferred `formula_statement` runner obligation and do
not credit proof skeleton payloads, local proof contexts, formula payloads,
facts, theorem acceptance, CoreIr, ControlFlowIr, VC, or proof payloads. In the
row above, the theorem/proof boundary entry now covers task 86's formula-only
case, task 87's term-bearing equality case, and task 88's proof-block case.

Task 89 refines the same row: statement-level proof-justification theorem
sources are executable through the active `type_elaboration` runner only as
extraction-gap boundaries. They do not satisfy the deferred `formula_statement`
runner obligation and do not credit statement proof payloads, nested proof
skeleton payloads, local proof contexts, formula payloads, label-reference
semantic checking, facts, theorem acceptance, CoreIr, ControlFlowIr, VC, or
proof payloads. In the row above, the theorem/proof boundary entry now covers
task 86's formula-only case, task 87's term-bearing equality case, task 88's
proof-block case, and task 89's statement-proof case.

Task 90 refines the same row: predicate and functor definition sources are
executable through the active `type_elaboration` runner only as extraction-gap
boundaries. They do not satisfy the deferred `formula_statement` runner
obligation and do not credit definition declaration payloads, definition-local
contexts, definiens formula/term payloads, overload payloads, facts, CoreIr,
ControlFlowIr, VC, or proof payloads.

Task 91 refines the same row: attribute definition sources are executable
through the active `type_elaboration` runner only as extraction-gap boundaries.
They do not satisfy the deferred `formula_statement` runner obligation and do
not credit definition declaration payloads, definition-local contexts,
formula-definiens payloads, attributed-type evidence, facts, CoreIr,
ControlFlowIr, VC, or proof payloads.

Task 92 refines the same row: mode and structure definition sources are
executable through the active `type_elaboration` runner only as extraction-gap
boundaries. They do not satisfy the deferred `formula_statement` runner
obligation and do not credit definition declaration payloads, mode expansion,
structure base-shape/constructor/selector evidence, definition-local contexts,
facts, CoreIr, ControlFlowIr, VC, or proof payloads.

Task 93 refines the same row: proof-local declaration statements are executable
through the active `type_elaboration` runner only as extraction-gap boundaries.
They do not satisfy the deferred `formula_statement` runner obligation and do
not credit proof-local declaration payloads, local proof contexts, formula/term
payloads, RHS term inference, reconsider coercion/obligation evidence, facts,
theorem acceptance, CoreIr, ControlFlowIr, VC, or proof payloads.

Task 94 refines the same row: proof-local `deffunc` and `defpred` inline
definitions are executable through the active `type_elaboration` runner only as
extraction-gap boundaries. They do not satisfy the deferred `formula_statement`
runner obligation and do not credit inline definition formal/body payloads,
local abbreviation expansion, term/formula body payloads, guard evidence,
facts, theorem acceptance, CoreIr, ControlFlowIr, VC, or proof payloads.

Task 95 refines the same row: top-level registration blocks containing
existential and conditional clusters are executable through the active
`type_elaboration` runner only as extraction-gap boundaries. They do not
satisfy deferred `formula_statement` or `advanced_semantics` runner obligations
and do not credit registration-item payloads, correctness-condition/proof
obligation payloads, accepted activation/evidence status, cluster/reduction
semantics, Chapter 17 semantic rows, facts, CoreIr, ControlFlowIr, VC, or proof
payloads.

Task 96 refines the same row: top-level and definition-local synonym/antonym
aliases plus attribute, predicate, and functor redefinition declarations are
executable through the active `type_elaboration` runner only as extraction-gap
boundaries. They do not satisfy deferred `formula_statement` or
`advanced_semantics` runner obligations and do not credit redefinition payloads,
notation alias relation payloads, target inference, coherence proof-obligation
payloads, overload candidate payloads, Chapter 11 alias semantic resolution,
Chapter 19 overload/redefinition semantics, facts, CoreIr, ControlFlowIr, VC,
or proof payloads.

Task 81 addendum: the `type_elaboration` runner also owns the active
argument-bearing local attribute extraction-gap boundary. It may run a
same-module parameterized attribute declared with `param_prefix` syntax and
used as `attribute_name(args)` in a reserve type expression, and it must keep
that source on `type_elaboration.external_dependency.ast_payload_extraction`
until real term-argument provenance and checker `AttributeInput` argument
payload extraction exist. This runner support does not credit attributed-type
evidence, positive parameterized attribute elaboration, CoreIr, ControlFlowIr,
VC, or proof payloads.

Task 82 addendum: the `type_elaboration` runner may pass an imported mode
reserve head from the documented `parser.type_fixtures` import summary as a
checker-owned symbol head when the resolver `SymbolEnv` marks it as
`SymbolKind::Mode` with an `ImportedSource` contribution. The expected active
diagnostic for `TypeCaseMode` becomes
`type_elaboration.checker.checker.type.external.mode_expansion_payload`; the
runner still must not synthesize imported module AST extraction,
`ModeExpansion` payloads, positive imported mode elaboration, CoreIr,
ControlFlowIr, VC, or proof payloads.

## Algorithm / Logic

1. Discover tests through `layout` under the known payload roots
   `miz`, `lexical`, `certificates`, `generated`, `fuzz`, `property`,
   `stress`, and `snapshots`.
2. Parse and validate every discovered sidecar, then build a canonical
   `TestPlan` whose returned `cases` are filtered by `DiscoveryConfig.profile`.
   Missing `profiles` defaults to `["fast"]`; `Full` includes every valid
   parsed case. Duplicate ids, traceability links, and diagnostics are checked
   across all parsed sidecars, not only the filtered cases.
3. For `parse-only`, select only cases with `stage = "parse_only"`,
   `expected_phase = "parse"`, `.miz` payloads, pass/fail outcomes, and
   `tags = ["active_parse_only"]`. Untagged parse-only sidecars remain
   discovery and traceability metadata.
4. For `declaration-symbol`, select only cases with
   `stage = "declaration_symbol"`, `expected_phase = "resolve"`, `.miz`
   payloads, pass/fail outcomes, and `tags = ["active_declaration_symbol"]`.
   Untagged declaration-symbol sidecars remain discovery and traceability
   metadata.
5. For `type-elaboration`, select only cases with
   `stage = "type_elaboration"`, `expected_phase = "type_check"`, `.miz`
   payloads, pass/fail outcomes, and `tags = ["active_type_elaboration"]`.
   Untagged type-elaboration sidecars remain discovery and traceability
   metadata.
6. Run cases in deterministic display order, even when execution is parallel.
7. Capture compiler outputs as structured records.
8. Match pass/fail expectations before snapshot expectations.
9. Compare general `[[snapshots]]` entries by canonical hash; the current
   parse-only `SurfaceAst` shortcut compares committed text baselines
   byte-for-byte as described below.
10. Report failures with phase, failure category, rejection reason, diagnostic code, and snapshot diff summary.

The current parse-only runner copies each active corpus file into a temporary
`src/` package, runs the real frontend parser seam, requires pass cases to
produce an AST with no assertion diagnostics, and compares fail cases against
the expected bare syntax diagnostic keys. For this syntax-only mode, the runner
uses a harness provider that resolves every frontend import stub to a
`ResolvedImportEntry` with matching `stub_ordinal` and `stub_span`, plus one
`ModuleLexicalSummary` per distinct module id. Summaries contain no exported
symbols except for the narrow `parser.type_fixtures` fixture module, which
injects parser-owned attribute, mode, structure, predicate, and functor shapes
needed by type-expression and operator syntax fixtures. No other import summary
exports symbols; the summaries exist only to keep import syntax cases from
depending on semantic module availability. If parser syntax diagnostics and non-syntax
frontend recovery diagnostics both appear, the runner reports all diagnostic
codes unless the sidecar explicitly includes
`allow_frontend_recovery_diagnostics`. Active parse-only pass/fail sidecars may
also set the transitional `snapshots = "snapshots/parser/<id>.surface_ast.snap"`
field. For those cases, after diagnostics match, the runner requires a
`SurfaceAst` and compares `SurfaceAst::snapshot_text()` with the committed
baseline under `tests/snapshots/`. Snapshot baselines are never rewritten during
normal parse-only runs.

An expectation tagged `active_parse_only` but missing one of the runnable case
predicates is a harness error rather than a silent skip.

The current declaration-symbol runner copies each active `.miz` corpus file
into the same temporary package shape, runs the real frontend, then feeds the
resulting `SurfaceAst` through the resolver declaration-shell collector,
parser-backed signature projection extractor, and symbol collector. Pass cases
require no frontend assertion diagnostics and no resolver symbol diagnostics;
when `declaration_symbol_payloads` is present, the runner also compares those
expected keys against the exact sorted SymbolEnv-derived symbol/definition fact
keys. Fail cases compare the resolver's crate-local internal detail keys
against `diagnostic_payloads` when present, or `stable_detail_key` otherwise.
The runner does not require or invent public resolver diagnostic codes while
the diagnostic-code ownership gap remains open; active declaration-symbol
expectations with non-empty `diagnostic_codes` are harness errors.

An expectation tagged `active_declaration_symbol` but missing one of the
runnable case predicates is a harness error rather than a silent skip.

The current type-elaboration runner copies each active `.miz` corpus file into
the same temporary package shape, runs the real frontend, then feeds the
resulting `SurfaceAst` through the resolver declaration-shell collector,
parser-backed signature projection extractor, and symbol collector. This keeps
type-elaboration cases honest about lower-stage prerequisites before checker
payload extraction begins.

After lower stages pass, the runner extracts syntax-free reserve declaration
payloads only for unrecovered reserve sources whose segments have one or more
identifiers and a supported reserve type-expression head. Successful pass cases
still require the bare builtin `set` / `object` shape with no attributes,
arguments, parameter prefixes, or non-builtin symbol heads, except that task 55
adds a second pass slice for bare local-mode reserve heads when the runner can
derive a real `ModeExpansion` from a unique unrecovered preceding same-module
no-argument `ModeDefinition` with no definition-local context and a bare
builtin `set` / `object` RHS. Task 56 extends that pass slice to one-edge
same-module local-mode chains when the reserve head expands to a preceding
same-module no-argument local mode whose own preceding source definition has an
accepted task-55 bare builtin RHS expansion. Task 72 extends the pass slice to
two bare local-mode dependency edges, and task 73 extends it to three edges,
when the terminal expansion is builtin `set` / `object`. Task 74 replaces that
temporary depth guard with an AST-bounded structural rule for bare same-module
no-argument local-mode chains whose terminal expansion is exactly builtin
`set` / `object`; chains that violate those structural guards still report
`checker.type.external.mode_expansion_payload`. Task 50 adds one
active fail slice: a same-module attribute symbol that resolver declaration/
symbol collection has already put in `SymbolEnv` may be attached to the builtin
reserve type payload, causing checker declaration checking to emit
`checker.declaration.deferred.evidence_query` rather than the broader AST
payload extraction gap. Task 51 adds a second active fail slice: a unique
same-module local mode symbol with no attributes or type arguments may be used
as the reserve type head, causing checker type normalization to emit
`checker.type.external.mode_expansion_payload` when neither the task-55 bare
expansion slice, the task-56 one-edge chain slice, nor the task-74 structural
bare chain slice applies. Task 52 adds a third active fail slice: a
unique same-module local structure symbol with no
attributes or type arguments may be used as the reserve type head, causing
checker declaration checking to emit
`checker.declaration.deferred.evidence_query` because real
base-shape/constructor-witness evidence extraction is still absent. Task 53
adds a fourth active fail slice: same-module source-derived attributes may be
attached to that local structure head, still causing
`checker.declaration.deferred.evidence_query` because Chapter 17 requires
existential evidence for the full normalized attributed type. Task 54 adds a
fifth active fail slice: same-module source-derived attributes may be attached
to a same-module local mode reserve head, still causing
`checker.type.external.mode_expansion_payload` when no supported real
expansion is available or the same mode is mixed with a bare reserve use; the
runner withholds task-55/task-56/task-74 expansions from mixed
attributed/bare local-mode sources. Task 56 also adds an active fail
slice proving that an attributed dependency in a local-mode chain withholds the
whole chain and reaches the same missing mode-expansion diagnostic rather than
inserting a partial `B -> A` payload. Task 57 adds another active fail slice:
a same-module no-argument local mode definition may have a bare same-module
local structure RHS, so the runner passes the real expansion payload to the
checker seam; because the checker still lacks source-derived
base-shape/constructor-witness evidence for that expanded structure radix, the
case reaches `checker.declaration.deferred.evidence_query` instead of
`checker.type.external.mode_expansion_payload`. Task 58 adds the parallel
attributed-builtin RHS diagnostic slice: a same-module no-argument local mode
definition may have an attributed builtin RHS, so the runner passes the real
expansion payload to the checker seam; because the checker still lacks
source-derived attributed-type existential evidence, the case reaches
`checker.declaration.deferred.evidence_query` instead of
`checker.type.external.mode_expansion_payload`. Task 59 adds the matching
attributed local-mode reserve diagnostic slice: if a same-module attributed
local-mode reserve head has a real direct bare-builtin RHS expansion and the
same mode is not mixed with a bare reserve use, the runner passes that
expansion payload to the checker seam; because the checker still lacks
source-derived attributed-type existential evidence, the case reaches
`checker.declaration.deferred.evidence_query` instead of
`checker.type.external.mode_expansion_payload`. Task 60 adds the direct
attributed local-mode structure-RHS diagnostic slice: if a same-module
attributed local-mode reserve head has a real direct local-structure RHS
expansion and the same mode is not mixed with a bare reserve use, the runner
passes that expansion payload to the checker seam; because the checker still
lacks source-derived base-shape/constructor-witness and full attributed-type
evidence, the case reaches `checker.declaration.deferred.evidence_query`
instead of `checker.type.external.mode_expansion_payload`. Task 61 adds the
direct attributed local-mode attributed-builtin-RHS diagnostic
slice: if a same-module attributed local-mode reserve head has a real direct
attributed-builtin RHS expansion and the same mode is not mixed with a bare
reserve use, the runner passes that expansion payload to the checker seam;
because the checker still lacks source-derived full attributed-type evidence,
the case reaches `checker.declaration.deferred.evidence_query` instead of
`checker.type.external.mode_expansion_payload`. Task 62 adds a one-edge bare
local-mode structure-RHS chain diagnostic slice: if `A is B` and `B is
LocalStruct` are unique, unrecovered, same-module, no-argument mode
definitions in source order after the unique local structure definition and
before the reserve use, the runner passes both real expansion payloads to the
checker seam; because the checker still lacks source-derived base-shape/
constructor-witness evidence, the case reaches
`checker.declaration.deferred.evidence_query` instead of
`checker.type.external.mode_expansion_payload`. Task 63 adds a one-edge bare
local-mode attributed-builtin-RHS chain diagnostic slice: if `A is B` and the
terminal attributed-builtin mode definition (`B is marked set` or
`B is marked object`) are unique, unrecovered, same-module, no-argument mode
definitions in source order before the reserve use, and the RHS
attributes are argument-free same-module attributes, the runner passes both
real expansion payloads to the checker seam; because the checker still lacks
source-derived attributed-type existential evidence, the case reaches
`checker.declaration.deferred.evidence_query` instead of
`checker.type.external.mode_expansion_payload`. Task 64 adds a one-edge
attributed-root bare-builtin chain diagnostic slice: if `reserve z for marked
A` uses unique, unrecovered, same-module, no-argument `B is set` / `object` and
`A is B` definitions in source order before the reserve use, `A` is not mixed
with a bare reserve use, and `B` is not itself an attributed reserve head, the
runner passes both real expansion payloads and the reserve-head attribute to
the checker seam; because the checker still lacks source-derived
attributed-type existential evidence, the case reaches
`checker.declaration.deferred.evidence_query` instead of
`checker.type.external.mode_expansion_payload`. Task 65 adds the structure-RHS
counterpart: if `reserve z for marked A` uses unique, unrecovered,
same-module, no-argument `B is LocalStruct` and `A is B` definitions in source
order after a unique same-module `LocalStruct` definition and before the reserve
use, `A` is not mixed with a bare reserve use, and `B` is not itself an
attributed reserve head, the runner passes both real expansion payloads and the
reserve-head attribute to the checker seam; because the checker still lacks
source-derived structure base-shape/constructor-witness evidence and full
attributed-type existential evidence, the case reaches
`checker.declaration.deferred.evidence_query` instead of
`checker.type.external.mode_expansion_payload`. Task 66 adds the
attributed-builtin-RHS counterpart: if `reserve z for marked A` uses unique,
unrecovered, same-module, no-argument `B is marked set` / `marked object` and
`A is B` definitions in source order before the reserve use, `A` is not mixed
with a bare reserve use, `B` is not itself an attributed reserve head, and RHS
attributes are argument-free same-module attributes, the runner passes both
real expansion payloads, the reserve-head attribute, and terminal RHS
attributes to the checker seam; because the checker still lacks source-derived
full attributed-type existential evidence, the case reaches
`checker.declaration.deferred.evidence_query` instead of
`checker.type.external.mode_expansion_payload`. Task 67 adds a
structure-qualified attribute boundary: a reserve type expression such as
`LocalStruct.marked LocalStruct` is parser/resolver executable, but the runner
must keep it on `type_elaboration.external_dependency.ast_payload_extraction`
until checker payloads preserve real structure-qualifier and attribute-owner
provenance; it must not rewrite the reference to an unqualified same-module
attribute payload. Task 68 adds an argument-bearing mode boundary: a reserve
type expression such as `Element of a` is parser/resolver executable when the
same-module mode surface exists, but the runner must keep it on
`type_elaboration.external_dependency.ast_payload_extraction` until checker
payloads preserve real type-argument and term-argument provenance; it must not
claim arity matching, mode expansion, or positive type elaboration for that
source. Task 69 adds the matching argument-bearing structure boundary: a
reserve type expression such as `LocalStruct of a` is parser/resolver
executable when the same-module structure declaration has an `of` parameter
surface, but the runner must keep it on
`type_elaboration.external_dependency.ast_payload_extraction` until checker
payloads preserve real type-argument and term-argument provenance; it must not
claim structure argument payload extraction, arity matching, base-shape
evidence, or positive structure type elaboration for that source. Task 70 adds
the bracket-form local mode boundary: a source containing a same-module
bracket-parameter mode declaration plus a bracket-form reserve type head such
as `Family[set]` is parser/resolver executable, but the runner must keep it on
`type_elaboration.external_dependency.ast_payload_extraction` until checker
payloads preserve real bracket type-argument and `qua`-argument provenance; it
must not claim bracket payload extraction, mode-head resolution, arity
matching, mode expansion, or positive type elaboration for that source. Task
71 adds the bracket-form local structure boundary: a source containing a
same-module bracket-parameter structure declaration plus a bracket-form reserve
type head such as `LocalStruct[set]` is parser/resolver executable, but the
runner must keep it on
`type_elaboration.external_dependency.ast_payload_extraction` until checker
payloads preserve real bracket type-argument and `qua`-argument provenance; it
must not claim bracket payload extraction, structure-head resolution, arity
matching, base-shape or constructor-witness evidence, or positive structure
type elaboration for that source. Broader imported
attributes, imported modes, and imported structures outside the task-82
`TypeCaseMode`, task-83 `R`, task-97 `TypeCaseStruct`, task-84 `TypeCaseAttr`, and task-85 negative
`empty`/builtin-`set` bridges, unresolved or ambiguous symbols,
attribute arguments, qualified attribute disambiguation beyond the task-67
boundary, mode/structure
arguments, type-argument, term-argument, bracket `type_arg_list`, or
`qua`-argument provenance, parameterized or contextual mode definitions, attributed structure
RHSs outside the task-62 bare chain slice, structure-RHS chains outside the
task-60 direct attributed-root slice, task-62 bare chain slice, and task-65
attributed-root chain slice,
attributed-RHS chains outside the task-58/task-61 direct slices, task-63 bare
chain slice, and task-66 attributed-root chain slice,
forward-reference or
cyclic local-mode chains, and non-reserve declarations remain outside this
source bridge.

For extracted payloads, the runner passes source/module identity, reserve
source range, binding spelling/ranges, supported type-expression spelling/
ranges/heads, supported same-module attribute symbol/range/polarity data, and
supported same-module local-mode expansion payloads, including the task-57
terminal local-structure RHS diagnostic slice, task-58 terminal
attributed-builtin RHS diagnostic slice, and task-59 attributed local-mode
reserve diagnostic slice, and task-60 attributed local-mode structure-RHS
diagnostic slice, and task-61 attributed local-mode attributed-RHS diagnostic
slice, task-62 local-mode structure-RHS chain diagnostic slice, and task-63
local-mode attributed-RHS chain diagnostic slice, and task-64 attributed
local-mode bare-builtin chain diagnostic slice, and task-65 attributed
local-mode structure-RHS chain diagnostic slice, and task-66 attributed
local-mode attributed-builtin-RHS chain diagnostic slice, to
`mizar-checker`'s source
reserve declaration seam. That checker-owned seam
builds the module `BindingEnv`, one `DeclarationInput` per binding, and
binding-specific `TypeExpressionInput` sites, so `reserve x, y for set` keeps
the shared source range while giving each binding a distinct typed site, and
runs `DeclarationChecker` against the collected `SymbolEnv`. The runner may
use the same checker-owned assembly helper to collect stable diagnostic keys
for active fail slices. If checker diagnostics are emitted, the active fail
case compares those keys and the runner does not credit downstream readiness
assertions. For diagnostic-free supported output, including the bare builtin
and task-55/56/72 local-mode expansion slices, the returned checker handoff is
credited as a checker-owned `TypedAst` with declaration and type-entry links
and as checker-owned `ResolvedTypedAst` projected using empty-but-real
cluster/overload predecessor outputs plus source-preserved node hints and
declaration expression metadata. The runner then passes that real
`ResolvedTypedAst` payload to `mizar-core`'s
`ResolvedTypedAstSummary::from_ast` and checks that the summary preserves the
source/module identity and has no checker recovery/diagnostic sites for the
successful reserve-only slice. It then prepares binder-only `CoreContextInput`
from the same real reserve bindings, with one `CoreVariableSeed` and one
`CoreBinderSeed` per extracted binding, no `CoreItemSeed`, and checks
source/module identity, binder source ranges, checker provenance, empty item
registry, empty core diagnostics, and an empty core worklist. This is a
summary/context readiness check only: it does not construct `CoreIr`,
`ControlFlowIr`, obligation seeds, VCs, or proof rows.
Active pass cases may assert this supported
source-derived slice with empty detail keys only when the source contains at
least one supported reserve binding and runner regression evidence confirms
that checker handoff construction, declaration checking, `TypedAst` assembly,
`ResolvedTypedAst` assembly, summary-readiness, and binder-only core context
readiness were exercised.

The runner still does not fabricate the missing AST-wide source-to-checker
bridge. Non-builtin declarations, imported attributes beyond the task-84
`TypeCaseAttr` bridge, task-85 negative `empty`/builtin-`set` bridge,
task-116 positive `empty`/builtin-`set` bridge, and task-80 boundary,
imported structures beyond the task-83 `R` bridge and task-97 `TypeCaseStruct` bridge and task-78
boundary, imported mode expansions beyond task 82's provenance/type-head bridge, attribute arguments,
mode/structure arguments, qualified attribute provenance, type-argument, term-argument,
bracket `type_arg_list`, or `qua`-argument
provenance, structure base-shape evidence, term/formula payloads beyond the
task-specific theorem bridges, formula child/binder semantics beyond task 112,
coercion sites, overload evidence, recorded facts, CoreIr, ControlFlowIr, VC
payloads, and proof evidence remain outside the supported extraction slice.
When an active case needs an unsupported source-to-checker payload family, the
runner either reports the stable detail key
`type_elaboration.external_dependency.ast_payload_extraction` or, for a
task-specific exact bridge, a checker-owned fail-closed diagnostic key. Active
fail cases may assert those keys through `diagnostic_payloads` or
`stable_detail_key`; active pass cases outside the supported slice remain
deferred rather than passing through a stub. This runner does not publish
`CoreIr`, `ControlFlowIr`, VC seeds, proof rows, or public checker diagnostic
codes.

Active type-elaboration expectations with non-empty `diagnostic_codes` are
harness errors until public checker diagnostic codes are specified. An
expectation tagged `active_type_elaboration` but missing one of the runnable
case predicates is a harness error rather than a silent skip.

General snapshot and determinism runner rows above are target-state harness
modes. Tasks 4 and 5 provide the shared `SnapshotRecord`, baseline
verify/update, and repeat-render comparison APIs, but this harness does not yet
parse general `[[snapshots]]` sidecar entries or run a general snapshot/update
subcommand. The active parse-only `SurfaceAst` shortcut remains the only
snapshot path wired into runner execution.

Core Task 31 adds one exact exception to that last sentence: the active
Task-180 contradiction pass case may reference its fixed `CoreIr::debug_text()`
baseline through the existing singular `snapshots` field. The runner constructs
the exact CoreIr twice, requires structural/debug-text equality, and then
performs a verify-only byte comparison against the committed baseline. It
publishes no general CoreIr payload and exposes no snapshot update command.
All other CoreIr/ControlFlowIr cases and the general snapshot registry remain
unwired and deferred.

Architecture-22 matrix support is metadata/reporting-only in task 14. The
metadata plan validates `architecture22_scenarios`,
`architecture22_equivalence_class`, and `architecture22_gate`, then reports the
registry class plus planned/active counts for each required scenario. All
task-14 scenario rows have no active eligibility, so `architecture22_gate =
"active"` is rejected until a future consumer-specific increment wires real
clean/incremental/parallel/cache-race execution.

## Determinism Requirements

The harness checks that identical inputs produce:

- identical artifact hashes;
- identical snapshot hashes;
- identical diagnostic order;
- identical failure records;
- identical proof status;
- identical dependency slices.

Parallel execution may change runtime, not observable results.

Implemented task-11 coverage renders metadata plans and active runner reports
to deterministic byte strings and compares repeated builds/runs. Snapshot-level
determinism and parallel equivalence are covered by the general snapshot record
helpers; active parallel runner subcommands remain future work until a consumer
crate exposes parallel execution.

## Reporting

Reports must separate:

- unexpected success;
- unexpected failure;
- wrong failure category;
- wrong rejection reason;
- diagnostic order mismatch;
- snapshot mismatch;
- nondeterminism across repeated runs;
- harness infrastructure error.

## Tests

Key scenarios:

- fail test unexpectedly passes;
- pass test emits an error diagnostic;
- snapshot hash differs;
- metadata plan bytes differ across repeated builds;
- active runner report bytes differ across repeated runs;
- repeated run produces a different diagnostic order;
- generic snapshot parallel equivalence produces the same observable artifact
  as sequential snapshot generation.
- architecture-22 matrix metadata reports all required scenario ids as planned
  and rejects fake active rows before an owning consumer runner exists.

## Constraints and Assumptions

- Test execution order is not semantic ordering.
- The harness treats cache hits as compiler behavior to verify, not as proof authority.
- Snapshot update mode is opt-in and must be visible in command output.


## Task 201 Immediate-Radix Asserted-Head Harness Contract

The Task 201 route is exact: two labeled, ordered bare mode definitions ending in builtin `set`; one `x` reserve of the outer mode; and one Base-mode formula-side type assertion. A closed relation keeps builtin and same-mode routes isolated and compares the asserted resolved symbol with the outer binding expansion's real immediate radix. The harness rejects missing/reordered/extra/deeper/recovered/contextual/parameterized/argument-bearing/attributed definitions, non-exact reserves or theorems, builtin/same-outer/object/unrelated/imported/ambiguous asserted heads, and independent expansion/binding/ordinal/head/spelling/site/range/immediate-edge/canonical corruptions. The immutable positive output and real frontend/resolver sidecar protect active runner 149. No general reachability, widening, `qua`, acceptance, truth/facts, proof, or downstream IR is activated.


## Task 202 Object Immediate-Radix Harness Contract

The Task 202 route accepts only the two labeled ordered bare object-mode definitions, one outer reserve, and one formula assertion of the immediate Base radix. It rejects structural/provenance near misses, additional set-terminal/object-chain shapes, independent payload and `BuiltinSet` corruption, and unresolved/imported/ambiguous heads. Real owning-positive then Task202-negative checks isolate Tasks 147/185/201, while the Task202 exact source is rejected by those owning routes. Immutable output and the real frontend/resolver sidecar protect active runner 150. No coercion, reachability, acceptance, truth/facts, proof, or downstream IR is activated.


## Task 203 Two-Edge Immediate-Radix Harness Contract

The Task 203 route accepts only three labeled, ordered, bare set-terminal mode definitions, one Outer reserve, and one formula assertion of the immediate Middle radix. It rejects every nonidentity definition order, duplicate or misspelled definitions, direct/one-edge/object/deeper shapes, imported or ambiguous Base/Middle/Outer provenance, and independent expansion/binding/ordinal/head/site/range/immediate-edge/`BuiltinObject`/canonical corruption. Bidirectional real-route checks isolate Tasks 122/148/149/186/187/201/202. Immutable output and a real frontend/resolver sidecar protect active runner 151. The harness does not activate two-hop reachability, Base assertion, coercion, acceptance, truth/facts, proof, or downstream IR.


## Task 204 Two-Edge Object Immediate-Radix Harness Contract

The Task 204 route accepts only three labeled, ordered, bare object-terminal mode definitions, one Outer reserve, and one formula assertion of the immediate Middle radix. It rejects every nonidentity definition order, duplicate or misspelled definitions, direct/one-edge/set-terminal/deeper shapes, imported or ambiguous Base/Middle/Outer provenance, and independent expansion/binding/ordinal/head/site/range/immediate-edge/`BuiltinSet`/canonical corruption. Bidirectional real-route checks isolate Tasks 189/145/147/149/187/202 and set Tasks 148/186/203. Immutable output and a real frontend/resolver sidecar protect active runner 152. The harness does not activate object/set coercion, two-hop reachability, Base assertion, acceptance, truth/facts, proof, or downstream IR.

## Task 205 Three-Edge Set Immediate-Radix Harness Contract

The Task 205 route accepts only four labeled, ordered, bare set-terminal mode definitions, one Outer reserve, and one formula assertion of the immediate Middle radix. It rejects all 23 nonidentity definition orders; missing, duplicate, mislabeled, misspelled, or wrong-radix definitions; direct/one-edge/two-edge/object-terminal/deeper shapes; multi-hop Inner/Base assertions; imported or ambiguous Base/Inner/Middle/Outer provenance; and independent expansion/binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinObject`/canonical corruption. Bidirectional real-route checks isolate set Tasks 122/138/146/148/150/195/201/203 and object Tasks 189/145/147/149/151/196/202/204. Immutable output and a real frontend/resolver sidecar protect active runner 153. The harness does not activate multi-hop reachability, widening, `qua`, acceptance, truth/facts, proof, or downstream IR.

## Task 206 Three-Edge Object Immediate-Radix Harness Contract

The Task 206 route accepts only four labeled, ordered, bare object-terminal mode definitions, one Outer reserve, and one formula assertion of the immediate Middle radix. It rejects all 23 nonidentity definition orders; missing, duplicate, mislabeled, misspelled, or wrong-radix definitions; direct/one-edge/two-edge/set-terminal/deeper shapes; multi-hop Inner/Base, builtin, local-other, argument-bearing, or attributed assertions; imported or ambiguous Base/Inner/Middle/Outer provenance; and independent expansion/binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinSet`/canonical corruption. Bidirectional real-route checks isolate set Tasks 122/138/146/148/150/195/201/203/205 and object Tasks 189/145/147/149/151/196/202/204. Immutable output and a real frontend/resolver sidecar protect active runner 154. The harness does not activate object/set coercion, multi-hop reachability, widening, `qua`, acceptance, truth/facts, proof, or downstream IR.

## Task 207 Four-Edge Set Immediate-Radix Harness Contract

The Task 207 route accepts only five labeled, ordered, bare set-terminal mode definitions, one TooDeep reserve, and one formula assertion of the immediate Outer radix. It rejects all 119 nonidentity definition orders; every missing, duplicate, mislabeled, misspelled, wrong-radix, recovered, contextual, parameterized, argument-bearing, or attributed definition; shorter, object-terminal, or connected deeper shapes; same-TooDeep, multi-hop Middle/Inner/Base, builtin, local-other, argument-bearing, or attributed assertions; imported or ambiguous provenance for all five symbols; and independent expansion/binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinObject`/canonical corruption. Bidirectional real-route checks isolate the 10 declared set owners and 10 declared object owners. Immutable output and a real frontend/resolver sidecar protect active runner 155. The harness does not activate multi-hop reachability, widening, `qua`, acceptance, truth/facts, proof, or downstream IR.

## Task 208 Four-Edge Object Immediate-Radix Harness Contract

The Task 208 route accepts only five labeled, ordered, bare object-terminal mode definitions, one TooDeep reserve, and one formula assertion of the immediate Outer radix. It rejects all 119 nonidentity orders; every per-definition structural near miss; non-exact reserve/formula shapes; shorter, set-terminal, or connected deeper chains; same-TooDeep, multi-hop Middle/Inner/Base, builtin object/set, local-other, argument-bearing, or attributed assertions; imported or ambiguous provenance for all five symbols; every expansion removal; and independent payload/binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinSet`/canonical corruption. An unrelated-import positive prevents over-rejection. Bidirectional real-route checks isolate 11 declared set owners and 10 object owners. Immutable output and a real frontend/resolver sidecar protect active runner 156. The harness does not activate object/set coercion, multi-hop reachability, widening, `qua`, acceptance, truth/facts, proof, or downstream IR.

## Task 209 Seven-Expansion Set Immediate-Radix Harness Contract

The Task 209 route accepts only seven labeled, ordered, bare set-terminal definitions `BaseMode -> set` through `ChainMode6 -> ChainMode5`, one ChainMode6 reserve, and one assertion of immediate ChainMode5. It rejects all 5,039 nonidentity orders; each definition's missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variants; non-exact and multi-binding reserves; non-exact formulas; same/multi-hop/builtin/local-other/argument-bearing/attributed asserted heads; a connected eighth edge; imported or ambiguous provenance for all seven symbols; every expansion removal; and independent binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinObject`/canonical corruption. An unrelated-import positive prevents over-rejection. Bidirectional checks isolate all 34 pre-existing owner routes, immutable-output checks prevent mutation, and a real frontend/resolver sidecar protects active runner 157. The harness does not activate multi-hop reachability, widening, `qua`, acceptance, truth/facts, proof, or downstream IR.

## Task 210 Seven-Expansion Object Immediate-Radix Harness Contract

The Task 210 route accepts only seven labeled, ordered, bare object-terminal definitions `BaseObjectMode -> object` through `ChainObjectMode6 -> ChainObjectMode5`, one ChainObjectMode6 reserve, and one assertion of immediate ChainObjectMode5. It rejects all 5,039 nonidentity orders; each definition's missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variants; non-exact and multi-binding reserves; non-exact formulas; same/multi-hop/builtin/local-other/argument-bearing/attributed asserted heads; a connected eighth edge; imported or ambiguous provenance for all seven symbols; every expansion removal; and independent binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinSet`/canonical corruption. An unrelated-import positive prevents over-rejection. Bidirectional checks isolate all 35 pre-existing owner routes, immutable-output checks prevent mutation, and a real frontend/resolver sidecar protects active runner 158. The harness activates neither object/set coercion nor multi-hop reachability, widening, `qua`, acceptance, truth/facts, proof, or downstream IR.

## Task 211 Two-Edge Set Two-Hop Asserted-Head Harness Contract

The Task 211 route accepts only three labeled, ordered, bare set-terminal definitions `BaseTwoHopModeAssertedHead -> set`, `MiddleTwoHopModeAssertedHead -> BaseTwoHopModeAssertedHead`, and `OuterTwoHopModeAssertedHead -> MiddleTwoHopModeAssertedHead`, one Outer reserve, and one assertion of Base. It explicitly validates both real links and rejects all five nonidentity orders; per-definition missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variants; non-exact reserves/formulas; same-Outer/immediate-Middle/builtin/object/local-other/deeper asserted heads; imported or ambiguous provenance for all three symbols; every expansion removal; and independent binding/ordinal/head/spelling/site/range/two-link/terminal/canonical corruption. An unrelated-import positive prevents over-rejection. Bidirectional checks isolate all 36 prior owner routes, immutable-output checks prevent mutation, and a real frontend/resolver sidecar protects active runner 159. The harness does not activate generic reachability, widening, `qua`, acceptance, truth/facts, proof, or downstream IR.

## Task 212 Two-Edge Object Two-Hop Asserted-Head Harness Contract

The Task 212 route accepts only three labeled, ordered, bare object-terminal definitions `BaseTwoHopObjectModeAssertedHead -> object`, `MiddleTwoHopObjectModeAssertedHead -> BaseTwoHopObjectModeAssertedHead`, and `OuterTwoHopObjectModeAssertedHead -> MiddleTwoHopObjectModeAssertedHead`, one Outer reserve, and one assertion of Base. It explicitly validates both real links and rejects all five nonidentity orders; per-definition missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variants; non-exact reserves/formulas; same-Outer/immediate-Middle/builtin-object/builtin-set/local-other/deeper asserted heads; imported or ambiguous provenance for all three symbols; every expansion removal; and independent binding/ordinal/head/spelling/site/range/two-link/terminal/`BuiltinSet`/canonical corruption. An unrelated-import positive prevents over-rejection. Bidirectional checks isolate all 37 prior owner routes, immutable-output checks prevent mutation, and a real frontend/resolver sidecar protects active runner 160. The harness activates neither generic reachability, widening, `qua`, object/set coercion, acceptance, truth/facts, proof, nor downstream IR.

## Task 213 Three-Edge Set Two-Hop Asserted-Head Harness Contract

The Task 213 route accepts only four labeled, ordered, bare set-terminal definitions `BaseThreeEdgeModeTwoHopAssertedHead -> set`, `InnerThreeEdgeModeTwoHopAssertedHead -> BaseThreeEdgeModeTwoHopAssertedHead`, `MiddleThreeEdgeModeTwoHopAssertedHead -> InnerThreeEdgeModeTwoHopAssertedHead`, and `OuterThreeEdgeModeTwoHopAssertedHead -> MiddleThreeEdgeModeTwoHopAssertedHead`, one Outer reserve, and one assertion of Inner. It explicitly validates the two real relation links and reserves terminal traversal for the Inner-to-Base-to-set tail. It rejects all 23 nonidentity orders; per-definition missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variants; non-exact reserves/formulas; same-Outer/immediate-Middle/full-distance-Base/builtin/object/local-other/deeper asserted heads; imported or ambiguous provenance for all four symbols; every expansion removal; and independent binding/ordinal/head/spelling/site/range/two-link/tail/terminal/canonical corruption. An unrelated-import positive prevents over-rejection. Bidirectional checks isolate all 38 prior owner routes, focused Tasks 211/212 checks preserve the shorter set/object routes, immutable-output checks prevent mutation, and a real frontend/resolver sidecar protects active runner 161. The harness activates neither generic reachability, widening, `qua`, acceptance, truth/facts, proof, nor downstream IR.

## Task 214 Three-Edge Object Two-Hop Asserted-Head Harness Contract

The Task 214 route accepts only four labeled, ordered, bare object-terminal definitions `BaseThreeEdgeObjectModeTwoHopAssertedHead -> object`, `InnerThreeEdgeObjectModeTwoHopAssertedHead -> BaseThreeEdgeObjectModeTwoHopAssertedHead`, `MiddleThreeEdgeObjectModeTwoHopAssertedHead -> InnerThreeEdgeObjectModeTwoHopAssertedHead`, and `OuterThreeEdgeObjectModeTwoHopAssertedHead -> MiddleThreeEdgeObjectModeTwoHopAssertedHead`, one Outer reserve, and one assertion of Inner. It explicitly validates both real relation links and reserves terminal traversal for the Inner-to-Base-to-object tail. It rejects all 23 nonidentity orders; every definition's missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variants; non-exact reserves/formulas; same/immediate/full-distance/builtin/local-other/deeper asserted heads; imported or ambiguous provenance for all four symbols; every expansion removal; and independent binding/ordinal/head/spelling/site/range/two-link/tail/terminal/canonical corruption. An unrelated-import positive prevents over-rejection. Bidirectional checks isolate all 39 prior owner routes, focused Tasks 211/212/213 checks preserve shorter and set-terminal routes, immutable-output checks prevent mutation, and a real frontend/resolver sidecar protects active runner 162. The harness activates neither object/set coercion, generic reachability, widening, `qua`, acceptance, truth/facts, proof, nor downstream IR.

## Task 215 Four-Edge Set Two-Hop Asserted-Head Harness Contract

The Task 215 route accepts only five labeled, ordered, bare set-terminal definitions `BaseFourEdgeModeTwoHopAssertedHead -> set`, `InnerFourEdgeModeTwoHopAssertedHead -> BaseFourEdgeModeTwoHopAssertedHead`, `MiddleFourEdgeModeTwoHopAssertedHead -> InnerFourEdgeModeTwoHopAssertedHead`, `OuterFourEdgeModeTwoHopAssertedHead -> MiddleFourEdgeModeTwoHopAssertedHead`, and `TooDeepFourEdgeModeTwoHopAssertedHead -> OuterFourEdgeModeTwoHopAssertedHead`, one TooDeep reserve, and one assertion of Middle. It explicitly validates the TooDeep-to-Outer and Outer-to-Middle relation links and reserves terminal traversal for the Middle-to-Inner-to-Base-to-set tail. It rejects all 119 nonidentity orders; every definition's finite missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variants; non-exact reserves/formulas; alternative asserted heads; imported or ambiguous provenance for all five symbols; every expansion removal; and independent binding/ordinal/head/spelling/site/range/relation-link/tail/terminal/canonical corruption. An unrelated-import positive prevents over-rejection. Bidirectional checks isolate all 40 prior owner routes, focused Tasks 211-214 checks preserve shorter and terminal siblings, immutable-output checks prevent mutation, and a real frontend/resolver sidecar protects active runner 163. The harness activates neither object/set coercion, generic reachability, widening, `qua`, acceptance, truth/facts, proof, nor downstream IR.

## Task 216 Four-Edge Object Two-Hop Asserted-Head Harness Contract

The Task 216 route accepts only five labeled, ordered, bare object-terminal definitions `BaseFourEdgeObjectModeTwoHopAssertedHead -> object`, `InnerFourEdgeObjectModeTwoHopAssertedHead -> BaseFourEdgeObjectModeTwoHopAssertedHead`, `MiddleFourEdgeObjectModeTwoHopAssertedHead -> InnerFourEdgeObjectModeTwoHopAssertedHead`, `OuterFourEdgeObjectModeTwoHopAssertedHead -> MiddleFourEdgeObjectModeTwoHopAssertedHead`, and `TooDeepFourEdgeObjectModeTwoHopAssertedHead -> OuterFourEdgeObjectModeTwoHopAssertedHead`, one TooDeep reserve, and one assertion of Middle. It explicitly validates the TooDeep-to-Outer and Outer-to-Middle relation links and reserves terminal traversal for the Middle-to-Inner-to-Base-to-object tail. It rejects all 119 nonidentity orders; every definition's finite missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variants; non-exact reserves/formulas; alternative asserted heads; imported or ambiguous provenance for all five symbols; every expansion removal; and independent binding/ordinal/head/spelling/site/range/relation-link/tail/terminal/canonical corruption. An unrelated-import positive prevents over-rejection. Bidirectional checks isolate all 41 prior owner routes, focused Tasks 211-215 checks preserve shorter and terminal siblings, immutable-output checks prevent mutation, and a real frontend/resolver sidecar protects active runner 164. The harness activates neither object/set coercion, generic reachability, widening, `qua`, acceptance, truth/facts, proof, nor downstream IR.

## Task 217 Three-Edge Set Three-Hop Asserted-Head Harness Contract

The Task 217 route accepts only four labeled, ordered, bare set-terminal definitions `BaseThreeEdgeModeThreeHopAssertedHead -> set`, `InnerThreeEdgeModeThreeHopAssertedHead -> BaseThreeEdgeModeThreeHopAssertedHead`, `MiddleThreeEdgeModeThreeHopAssertedHead -> InnerThreeEdgeModeThreeHopAssertedHead`, and `OuterThreeEdgeModeThreeHopAssertedHead -> MiddleThreeEdgeModeThreeHopAssertedHead`, one Outer reserve, and one assertion of Base. It explicitly validates the Outer-to-Middle, Middle-to-Inner, and Inner-to-Base relation links and reserves terminal traversal for Base-to-set only. It rejects all 23 nonidentity orders; every definition's finite missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variants; non-exact reserves/formulas and alternative asserted heads; imported or ambiguous provenance for all four symbols; every expansion removal; and independent binding/ordinal/head/spelling/site/range/relation-link/terminal/canonical corruption. An unrelated-import positive prevents over-rejection. Bidirectional checks isolate all 42 prior owner routes, focused Tasks 211-216 checks preserve shorter and terminal siblings, immutable-output checks prevent mutation, and a real frontend/resolver sidecar protects active runner 165. The harness activates neither object/set coercion, generic reachability, widening, `qua`, acceptance, truth/facts, proof, nor downstream IR.

## Task 218 Three-Edge Object Three-Hop Asserted-Head Harness Contract

The Task 218 route accepts only four labeled, ordered, bare object-terminal definitions `BaseThreeEdgeObjectModeThreeHopAssertedHead -> object`, `InnerThreeEdgeObjectModeThreeHopAssertedHead -> BaseThreeEdgeObjectModeThreeHopAssertedHead`, `MiddleThreeEdgeObjectModeThreeHopAssertedHead -> InnerThreeEdgeObjectModeThreeHopAssertedHead`, and `OuterThreeEdgeObjectModeThreeHopAssertedHead -> MiddleThreeEdgeObjectModeThreeHopAssertedHead`, one Outer reserve, and one assertion of Base. It explicitly validates the Outer-to-Middle, Middle-to-Inner, and Inner-to-Base relation links and reserves terminal traversal for Base-to-object only. The matrix rejects all 23 nonidentity orders; every definition's finite missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variants; non-exact reserves/formulas and same/immediate/two-hop/builtin/local-other/deeper asserted heads; imported or ambiguous provenance for all four symbols; every expansion removal; and independent binding/ordinal/head/spelling/site/range/relation-link/terminal/`BuiltinSet`/canonical corruption. An unrelated-import positive prevents over-rejection. Bidirectional checks isolate all 43 prior owner routes, focused Tasks 211-217 checks preserve shorter and terminal siblings, immutable-output checks prevent mutation, and a real frontend/resolver sidecar protects active runner 166. The harness activates neither object/set coercion, generic reachability, widening, `qua`, acceptance, truth/facts, proof, nor downstream IR.

## Task 219 Four-Edge Set Three-Hop Asserted-Head Harness Contract

The Task 219 route accepts only five labeled, ordered, bare set-terminal definitions `BaseFourEdgeModeThreeHopAssertedHead -> set`, `InnerFourEdgeModeThreeHopAssertedHead -> BaseFourEdgeModeThreeHopAssertedHead`, `MiddleFourEdgeModeThreeHopAssertedHead -> InnerFourEdgeModeThreeHopAssertedHead`, `OuterFourEdgeModeThreeHopAssertedHead -> MiddleFourEdgeModeThreeHopAssertedHead`, and `TooDeepFourEdgeModeThreeHopAssertedHead -> OuterFourEdgeModeThreeHopAssertedHead`, one TooDeep reserve, and one assertion of Inner. It explicitly validates the TooDeep-to-Outer, Outer-to-Middle, and Middle-to-Inner relation links and reserves terminal traversal for the Inner-to-Base-to-set tail only. The matrix independently rejects (a) an unconnected unsupported deeper asserted head and (b) an actual connected sixth-definition/sixth-edge asserted head, in addition to all 119 nonidentity orders; every definition's finite missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variants; non-exact reserves/formulas and same/immediate/two-hop/full-distance/builtin/local-other asserted heads; imported or ambiguous provenance for all five symbols; every expansion removal; and independent binding/ordinal/head/spelling/site/range/relation-link/terminal/`BuiltinObject`/canonical corruption. An unrelated-import positive prevents over-rejection. Bidirectional checks isolate all 44 prior owner routes, focused Task 207 and Tasks 211-218 checks preserve shorter and terminal siblings, immutable-output checks prevent mutation, and a real frontend/resolver sidecar protects active runner 167. The harness activates neither object/set coercion, generic reachability, widening, `qua`, acceptance, truth/facts, proof, nor downstream IR.

## Task 220 Four-Edge Object Three-Hop Asserted-Head Harness Contract

The Task 220 route accepts only five labeled, ordered, bare object-terminal definitions `BaseFourEdgeObjectModeThreeHopAssertedHead -> object`, `InnerFourEdgeObjectModeThreeHopAssertedHead -> BaseFourEdgeObjectModeThreeHopAssertedHead`, `MiddleFourEdgeObjectModeThreeHopAssertedHead -> InnerFourEdgeObjectModeThreeHopAssertedHead`, `OuterFourEdgeObjectModeThreeHopAssertedHead -> MiddleFourEdgeObjectModeThreeHopAssertedHead`, and `TooDeepFourEdgeObjectModeThreeHopAssertedHead -> OuterFourEdgeObjectModeThreeHopAssertedHead`, one TooDeep reserve, and one assertion of Inner. It explicitly validates the TooDeep-to-Outer, Outer-to-Middle, and Middle-to-Inner relation links and reserves terminal traversal for the Inner-to-Base-to-object tail only. The matrix independently rejects (a) an unconnected unsupported deeper asserted head and (b) an actual connected sixth-definition/sixth-edge asserted head, in addition to all 119 nonidentity orders; every definition's finite missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variants; non-exact reserves/formulas and same/immediate/two-hop/full-distance/builtin/local-other asserted heads; imported or ambiguous provenance for all five symbols; every expansion removal; and independent binding/ordinal/head/spelling/site/range/relation-link/terminal/`BuiltinSet`/canonical corruption. An unrelated-import positive prevents over-rejection. Bidirectional checks isolate all 45 prior owner routes, focused Tasks 208 and 211-219 checks preserve shorter and terminal siblings, immutable-output checks prevent mutation, and a real frontend/resolver sidecar protects active runner 168. The harness activates neither object/set coercion, generic reachability, widening, `qua`, acceptance, truth/facts, proof, nor downstream IR.

## Task 221 Four-Edge Set Four-Hop Asserted-Head Active Harness Contract

The Task 221 route accepts only five labeled, ordered, bare set-terminal definitions `BaseFourEdgeModeFourHopAssertedHead -> set`, `InnerFourEdgeModeFourHopAssertedHead -> BaseFourEdgeModeFourHopAssertedHead`, `MiddleFourEdgeModeFourHopAssertedHead -> InnerFourEdgeModeFourHopAssertedHead`, `OuterFourEdgeModeFourHopAssertedHead -> MiddleFourEdgeModeFourHopAssertedHead`, and `TooDeepFourEdgeModeFourHopAssertedHead -> OuterFourEdgeModeFourHopAssertedHead`, one TooDeep reserve, and one assertion of Base. It explicitly validates all four relation links and reserves terminal traversal for Base-to-set only. The matrix rejects all 119 nonidentity orders; every definition's finite missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variants; non-exact reserves/formulas and same/immediate/two-hop/three-hop/builtin/local-other asserted heads; imported or ambiguous provenance for all five symbols; every expansion removal; independent binding/ordinal/head/spelling/site/range/each-link/terminal/`BuiltinObject`/canonical corruption; and separate unconnected-deeper and actual connected fifth-link heads. An unrelated-import positive prevents over-rejection. Bidirectional checks isolate all 46 prior owner routes, focused Task 207 and Tasks 211-220 checks preserve existing routes, immutable-output checks prevent mutation, and a real frontend/resolver sidecar protects active runner 169. The harness activates neither general reachability, widening, `qua`, acceptance, truth/facts, proof, nor downstream IR.

## Task 222 Four-Edge Object Four-Hop Asserted-Head Active Harness Contract

The Task 222 route accepts only five labeled, ordered, bare object-terminal definitions `BaseFourEdgeObjectModeFourHopAssertedHead -> object`, `InnerFourEdgeObjectModeFourHopAssertedHead -> BaseFourEdgeObjectModeFourHopAssertedHead`, `MiddleFourEdgeObjectModeFourHopAssertedHead -> InnerFourEdgeObjectModeFourHopAssertedHead`, `OuterFourEdgeObjectModeFourHopAssertedHead -> MiddleFourEdgeObjectModeFourHopAssertedHead`, and `TooDeepFourEdgeObjectModeFourHopAssertedHead -> OuterFourEdgeObjectModeFourHopAssertedHead`, one TooDeep reserve, and one assertion of Base. It explicitly validates all four relation links and reserves terminal traversal for Base-to-object only. The matrix rejects all 119 nonidentity orders; every definition's finite missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variants; non-exact reserves/formulas and same/immediate/two-hop/three-hop/builtin/local-other asserted heads; imported or ambiguous provenance for all five symbols; every expansion removal; independent binding/ordinal/head/spelling/site/range/each-link/terminal/`BuiltinSet`/canonical corruption; and separate unconnected-deeper and actual connected fifth-link heads. An unrelated-import positive prevents over-rejection. Bidirectional checks isolate all 47 prior owner routes, focused Task 208 and Tasks 211-221 checks preserve existing routes, immutable-output checks prevent mutation, and a real frontend/resolver sidecar protects active runner 170. The harness activates neither general reachability, widening, `qua`, object/set coercion, acceptance, truth/facts, proof, nor downstream IR.

## Task 223 Parenthesized Reserved-Variable Equality Active Harness Contract

The active Task 223 route accepts only one builtin-set reserve and one equality whose left operand is a single unrecovered `ParenthesizedTerm` containing exactly one identifier `x` and whose right operand is direct `x`. It preserves independent wrapper/inner/right source metadata, resolves only the inner and right references through the real reserve `BindingEnv`, and transparently feeds the inner value/type to the existing equality consumer without a separate parenthesis type or fabricated child payload. The matrix rejects direct/right/both/nested/empty/non-identifier/recovered/malformed wrappers and non-exact labels/operators/reserves/items; corrupts wrapper/inner/right metadata, lookup ordinals/bindings, result/expected inputs, and matched output independently; proves immutable output; isolates all 52 prior reserved-variable binary-formula owners in both directions; and uses a real frontend/resolver sidecar. Focused, relevant-crate, and workspace verification passed. The harness activates neither arbitrary parenthesization/precedence, formula grouping, closure materialization, equality truth/facts, acceptance, proof, child graphs, nor downstream IR.

## Task 224 Seven-Expansion Set Two-Hop Asserted-Head Active Harness Contract

The active Task 224 route accepts only the seven labeled, ordered, bare set-terminal long-chain definitions, one `ChainMode6` reserve, and one assertion of `ChainMode4`. It uses the unchanged `BindingTwoHopRadix` to validate `ChainMode6 -> ChainMode5` and `ChainMode5 -> ChainMode4` directly and uses the remaining tail only for terminal normalization. The matrix rejects all 5,039 nonidentity orders, non-exact definition/reserve/formula/head/provenance shapes, every expansion and relation/tail/terminal corruption, and connected deeper heads; retains an unrelated-import positive; proves immutable output; isolates all 48 prior owners bidirectionally; and uses a real frontend/resolver sidecar. The harness does not activate generic reachability, widening, `qua`, acceptance, truth/facts, proof, or downstream IR.

## Task 225 Seven-Expansion Object Two-Hop Asserted-Head Active Harness Contract

The active Task 225 route accepts only the seven labeled, ordered, bare object-terminal long-chain definitions, one `ChainObjectMode6` reserve, and one assertion of `ChainObjectMode4`. It uses the unchanged `BindingTwoHopRadix` to validate `ChainObjectMode6 -> ChainObjectMode5` and `ChainObjectMode5 -> ChainObjectMode4` directly and uses the remaining tail only for object-terminal normalization. The matrix rejects all 5,039 nonidentity orders, non-exact definition/reserve/formula/head/provenance shapes, every expansion and relation/tail/terminal corruption, set/object mixing, and connected deeper heads; retains an unrelated-import positive; proves immutable output; isolates all 49 prior owners bidirectionally; and uses a real frontend/resolver sidecar. Focused, relevant-crate, and workspace verification passed. The harness does not activate generic reachability, widening, `qua`, object/set coercion, acceptance, truth/facts, proof, or downstream IR.

## Task 226 Seven-Expansion Set Three-Hop Asserted-Head Active Harness Contract

The active Task 226 route accepts only the seven labeled, ordered, bare set-terminal long-chain definitions, one `ChainMode6` reserve, and one assertion of `ChainMode3`. It uses the unchanged `BindingThreeHopRadix` to validate `ChainMode6 -> ChainMode5`, `ChainMode5 -> ChainMode4`, and `ChainMode4 -> ChainMode3` directly and uses the remaining tail only for set-terminal normalization. The matrix rejects all 5,039 nonidentity orders, non-exact definition/reserve/formula/head/provenance shapes, every expansion and relation/tail/terminal corruption, object/set mixing, and connected deeper heads; retains an unrelated-import positive; proves immutable output; isolates all 50 prior owners bidirectionally; and uses a real frontend/resolver sidecar. Focused, relevant-crate, and workspace verification passed. The harness does not activate generic reachability, widening, `qua`, acceptance, truth/facts, proof, or downstream IR.

## Task 227 Seven-Expansion Object Three-Hop Asserted-Head Active Harness Contract

The active Task 227 route accepts only the seven labeled, ordered, bare object-terminal long-chain definitions, one `ChainObjectMode6` reserve, and one assertion of `ChainObjectMode3`. It uses the unchanged `BindingThreeHopRadix` to validate `ChainObjectMode6 -> ChainObjectMode5`, `ChainObjectMode5 -> ChainObjectMode4`, and `ChainObjectMode4 -> ChainObjectMode3` directly and uses the remaining tail only for object-terminal normalization. The matrix rejects all 5,039 nonidentity orders, non-exact definition/reserve/formula/head/provenance shapes, every expansion and relation/tail/terminal corruption, set/object mixing, and connected deeper heads; retains an unrelated-import positive; proves immutable output; isolates all 51 prior owners bidirectionally; and uses a real frontend/resolver sidecar. Focused, relevant-crate, and workspace verification passed. The harness does not activate generic reachability, object/set coercion, widening, `qua`, acceptance, truth/facts, proof, or downstream IR.

## Task 228 Seven-Expansion Set Four-Hop Asserted-Head Active Harness Contract

The active Task 228 route accepts only the seven labeled, ordered, bare set-terminal long-chain definitions, one `ChainMode6` reserve, and one assertion of `ChainMode2`. It uses the unchanged `BindingFourHopRadix` to validate `ChainMode6 -> ChainMode5`, `ChainMode5 -> ChainMode4`, `ChainMode4 -> ChainMode3`, and `ChainMode3 -> ChainMode2` directly and uses the remaining tail only for set-terminal normalization. The matrix rejects all 5,039 nonidentity orders, non-exact definition/reserve/formula/head/provenance shapes, every expansion and relation/tail/terminal corruption, object/set mixing, and connected deeper heads; retains an unrelated-import positive; proves immutable output; isolates all 52 prior owners bidirectionally; and uses a real frontend/resolver sidecar. Focused, relevant-crate, and workspace verification passed. The harness does not activate generic reachability, widening, `qua`, acceptance, truth/facts, proof, or downstream IR.

## Task 229 Seven-Expansion Object Four-Hop Asserted-Head Active Harness Contract

The active Task 229 route accepts only the seven labeled, ordered, bare object-terminal long-chain definitions, one `ChainObjectMode6` reserve, and one assertion of `ChainObjectMode2`. It uses the unchanged `BindingFourHopRadix` to validate `ChainObjectMode6 -> ChainObjectMode5`, `ChainObjectMode5 -> ChainObjectMode4`, `ChainObjectMode4 -> ChainObjectMode3`, and `ChainObjectMode3 -> ChainObjectMode2` directly and uses the remaining tail only for object-terminal normalization. The matrix rejects all 5,039 nonidentity orders, non-exact definition/reserve/formula/head/provenance shapes, every expansion and relation/tail/terminal corruption, object/set mixing, and connected deeper heads; retains an unrelated-import positive; proves immutable output; isolates all 53 prior owners bidirectionally; and uses a real frontend/resolver sidecar. Focused, relevant-crate, and workspace verification passed. The harness does not activate generic reachability, widening, `qua`, acceptance, truth/facts, proof, object/set coercion, or downstream IR.

## Task 230 Seven-Expansion Set Five-Hop Asserted-Head Active Harness Contract

The active Task 230 route accepts only the seven labeled, ordered, bare set-terminal long-chain definitions, one `ChainMode6` reserve, and one assertion of `ChainMode1`. The new closed `BindingFiveHopRadix` validates `ChainMode6 -> ChainMode5`, `ChainMode5 -> ChainMode4`, `ChainMode4 -> ChainMode3`, `ChainMode3 -> ChainMode2`, and `ChainMode2 -> ChainMode1` directly and uses `ChainMode1 -> BaseMode -> set` only for terminal normalization. The matrix rejects all 5,039 nonidentity orders, non-exact definition/reserve/formula/head/provenance shapes, every expansion and relation/tail/terminal corruption, object/set mixing, and connected deeper heads; retains an unrelated-import positive; proves immutable output; isolates all 54 prior owners bidirectionally; and uses a real frontend/resolver sidecar. Focused, relevant-crate, and workspace verification passed. The harness does not activate generic reachability, widening, `qua`, acceptance, truth/facts, proof, or downstream IR.

## Task 231 Seven-Expansion Object Five-Hop Asserted-Head Active Harness Contract

The active Task 231 route accepts only the seven labeled, ordered, bare object-terminal long-chain definitions, one `ChainObjectMode6` reserve, and one assertion of `ChainObjectMode1`. The byte-for-byte unchanged closed `BindingFiveHopRadix` validates `ChainObjectMode6 -> ChainObjectMode5`, `ChainObjectMode5 -> ChainObjectMode4`, `ChainObjectMode4 -> ChainObjectMode3`, `ChainObjectMode3 -> ChainObjectMode2`, and `ChainObjectMode2 -> ChainObjectMode1` directly and uses `ChainObjectMode1 -> BaseObjectMode -> object` only for terminal normalization. The matrix rejects all 5,039 nonidentity orders, non-exact definition/reserve/formula/head/provenance shapes, every expansion and relation/tail/terminal corruption, set/object mixing, and connected deeper heads; retains an unrelated-import positive; proves immutable output; isolates all 55 prior owners bidirectionally; and uses a real frontend/resolver sidecar. Focused, relevant-crate, and workspace verification passed. The harness does not activate generic reachability, object/set coercion, widening, `qua`, acceptance, truth/facts, proof, or downstream IR.

## Task 233 Parenthesized Builtin-Object Equality Active Harness Contract

The active Task 233 route accepts only one builtin-object reserve and one equality whose left operand is one unrecovered `ParenthesizedTerm` containing exactly one identifier `x` and whose right operand is direct `x`. It preserves independent wrapper/inner/right source metadata, resolves only the inner and right references through the real reserve `BindingEnv`, and transparently feeds the inner builtin-object value/type to the existing equality consumer without an independent wrapper payload or object/set coercion. The matrix rejects direct/right/both/nested/empty/non-identifier/recovered/malformed wrappers and non-exact labels/operators/reserves/items; corrupts wrapper/inner/right metadata, lookup ordinals/bindings, result/expected inputs, canonical type, and matched output independently; proves immutable output; isolates all 53 prior binary-formula owners bidirectionally; and uses a real frontend/resolver sidecar. The harness does not activate arbitrary parenthesization/precedence, formula grouping, closure materialization, equality truth/facts, acceptance, proof, child graphs, or downstream IR.

## Task 234 Six-Hop Set-Terminal Asserted-Head Active Harness Contract

The active Task 234 route accepts only the seven labeled, ordered, bare set-terminal long-chain definitions, one `ChainMode6` reserve, and one assertion of `BaseMode`. The new closed `BindingSixHopRadix` validates `ChainMode6 -> ChainMode5`, `ChainMode5 -> ChainMode4`, `ChainMode4 -> ChainMode3`, `ChainMode3 -> ChainMode2`, `ChainMode2 -> ChainMode1`, and `ChainMode1 -> BaseMode` directly and uses `BaseMode -> set` only for terminal normalization. The matrix rejects all 5,039 nonidentity orders, non-exact definition/reserve/formula/head/provenance shapes, every expansion and relation/terminal corruption, object mixing, and connected deeper heads; retains an unrelated-import positive; proves immutable output; isolates all 56 prior owners bidirectionally; and uses a real frontend/resolver sidecar. The harness does not activate generic reachability, widening, `qua`, acceptance, truth/facts, proof, child graphs, or downstream IR.

## Task 236 Object-Terminal Six-Hop Asserted-Head Active Harness Contract

The active Task 236 route accepts only the seven labeled, ordered, bare object-terminal long-chain definitions, one `ChainObjectMode6` reserve, and one assertion of `BaseObjectMode`. The unchanged closed `BindingSixHopRadix` validates `ChainObjectMode6 -> ChainObjectMode5`, `ChainObjectMode5 -> ChainObjectMode4`, `ChainObjectMode4 -> ChainObjectMode3`, `ChainObjectMode3 -> ChainObjectMode2`, `ChainObjectMode2 -> ChainObjectMode1`, and `ChainObjectMode1 -> BaseObjectMode` directly and uses `BaseObjectMode -> object` only for terminal normalization. The matrix rejects all 5,039 nonidentity orders, non-exact definition/reserve/formula/head/provenance shapes, every expansion and relation/terminal corruption, set mixing, and connected deeper heads; retains an unrelated-import positive; proves immutable output; isolates all 57 prior owners bidirectionally; and uses a real frontend/resolver sidecar. The harness does not activate object/set coercion, generic reachability, widening, `qua`, acceptance, truth/facts, proof, child graphs, or downstream IR.

## Task 241 Parenthesized Reserved-Variable Inequality Active Harness Contract

The active Task 241 route accepts only one builtin-set reserve and one inequality
whose left operand is one unrecovered `ParenthesizedTerm` containing exactly one
identifier `x` and whose right operand is direct `x`. It preserves independent
wrapper/inner/right metadata, resolves only the inner and right references, and
transparently feeds one canonical builtin-set identity to the existing inequality
consumer without an independent wrapper payload. The matrix rejects direct,
right/both/nested/empty/nonidentifier/recovered/malformed operands, wrong labels/
operators/reserves/types/status/items, exact parenthesized membership, and exact
builtin-object `<>`; corrupts provenance, binding/ordinal, roles/expected input,
canonical source, and matched config independently; proves immutable output;
retains focused equality behavior; isolates all 54 prior binary-formula owners
bidirectionally; and uses a real frontend/resolver sidecar. The harness does not
activate arbitrary parenthesization/precedence, formula grouping, inequality
desugaring/truth, acceptance, proof, child graphs, or downstream IR.

## Task 242 Parenthesized Builtin-Object Inequality Active Harness Contract

The active Task 242 route accepts only one builtin-object reserve and one
inequality whose left operand is one unrecovered `ParenthesizedTerm` containing
exactly one identifier `x` and whose right operand is direct `x`. It preserves
independent wrapper/inner/right metadata, resolves the two references at
ordinals 1/2 to `BindingId(0)`, and transparently feeds one written-`object`-
anchored canonical `BuiltinObject` to the existing inequality consumer without
an independent wrapper payload or object/set coercion. The matrix rejects all
direct/right/both/nested/empty/nonidentifier/recovered/malformed near misses,
wrong labels/operators/reserves/types/status/items, exact parenthesized
membership, and builtin-set variants; independently corrupts wrapper/source-
wrapper, inner/right provenance, lookup, builtin head, roles/source ranges,
canonical bridge, expected input, and matched Task 233/241 configs; proves
immutable output and mismatched-module rejection; isolates all 55 prior binary-
formula owners bidirectionally; retains focused Tasks 190/223/233/241; and uses
a real frontend/resolver sidecar. Parenthesized membership and active imported
provenance receive no Task 242 credit; missing imported expansion/evidence/
signature payloads, proof, and downstream IR remain deferred.

## Task 243 Parenthesized Reserved-Variable Membership Active Harness Contract

The active Task 243 route accepts only one builtin-set reserve and one membership
whose left operand is one unrecovered `ParenthesizedTerm` containing exactly one
identifier `x` and whose right operand is direct `x`. It preserves independent
wrapper/inner/right metadata, resolves the two references at ordinals 1/2 to
`BindingId(0)`, and transparently feeds one written-`set`-anchored canonical
`BuiltinSet` to the existing membership consumer. Its unchanged direct-right
producer supplies the sole expected-set input: exactly five type entries, no
left expected input, and one right-owned expected constraint. The wrapper has no
independent payload. The matrix rejects all direct/right/both/nested/empty/
nonidentifier/recovered/malformed near misses, wrong labels/operators/reserves/
types/status/items, prior parenthesized equality/inequality and object variants;
independently corrupts provenance, lookup, result head, roles/source ranges,
canonical bridge, unexpected-left/wrong-right/missing-right expected input, and
matched configs; proves immutable output and mismatched-module rejection;
isolates all 56 prior binary-formula owners bidirectionally; retains focused
Tasks 120/223/233/241/242; and uses a real frontend/resolver sidecar. Only this
exact source discharges the extraction gap. Object-left/set-right parenthesized
membership and active imported provenance receive no Task 243 credit; missing
imported expansion/evidence/signature payloads, proof, and downstream IR remain
deferred.

## Task 244 Parenthesized Heterogeneous Reserve Membership Active Harness Contract

The active Task 244 route accepts exactly two ordered reserves, `x` for written
`object` and `y` for a distinct written `set`, followed by the theorem
`ParenthesizedHeterogeneousReserveMembershipPayloadBoundary: (x) in y;`. The
real frontend must produce exactly one unrecovered `ParenthesizedTerm` around
the left identifier. The real resolver and complete binding environment must
resolve inner `x` and direct-right `y` at ordinals 2/3 to `BindingId(0/1)`.

The finite config-driven bridge preserves independent wrapper, inner, right,
formula, and two reserve-type provenances. The unchanged Task 125 direct-right
producer supplies the sole expected-set input. The output contract requires two
inferred terms, exactly five type entries, two normalized identities anchored to
their distinct written ranges, no left expected input, one right-owned expected-
set constraint, and one fact/candidate/diagnostic/deferred-free checked
membership. The wrapper owns no semantic reference and object/set coercion is
forbidden. The five prior parenthesized configs must retain their old contracts.

Focused coverage contains exact, near-miss, collapsed/reversed provenance,
payload corruption, immutable output, all 57 prior owner routes in both
directions, Tasks 120/125/223/233/241/242/243, the real active imported-mode-gap
fixture with unchanged diagnostics, and a real frontend/resolver sidecar. The
active runner count is 186; plan counts are 401/365, type 233/221, and pass/fail
217/184. Five shared references and one dedicated requirement trace the case
without rebaselining existing expectations. Only this exact source discharges
the extraction gap. Other parenthesized shapes and imported-positive provenance
receive no Task 244 credit; missing imported expansion/evidence/signature
payloads, proof, and downstream IR remain deferred.

## Task 245 Right-Parenthesized Reserved-Variable Membership Active Harness Contract

The active fixture is exactly `reserve x for set; theorem
RightParenthesizedReservedVariableMembershipPayloadBoundary: x in (x);`. The
real frontend must produce one unrecovered right-operand `ParenthesizedTerm`
containing one identifier, and the resolver must preserve distinct wrapper,
direct-left, right-inner, and formula provenance. Both semantic identifiers
resolve at ordinals 1/2 to `BindingId(0)`.

The route requires explicit `Right` side metadata and a Task-245-only config,
key, and result/expected roles. Task 120's consumer produces two inferred terms,
five type entries sharing one written-set identity, no left expected input, one
right-inner-owned expected-set constraint, and one clean checked membership.
The wrapper owns no semantic reference. Coverage independently rejects left,
direct, both, nested, malformed, recovered, wrong reserve/operator/label, side,
config, range, role, constraint, and term-order corruptions; cross-rejects Task
243; proves immutable/module boundaries; isolates 58 prior owners in both
directions; retains all six left routes; and exercises a real frontend/resolver
sidecar. Active runner 187 and plan 402/366, type 234/222, pass/fail 218/184 are
traced by four shared plus one dedicated reference without rebaselining. Other
shapes, imported-positive provenance, proof, and downstream IR remain outside
credit or deferred as applicable.

## Task 246 Parenthesized Two-Edge Local-Mode Equality Active Harness Contract

The active case is the exact ordered three-mode set-terminal source followed by
an Outer reserve and `(z) = z`. The runner must observe one unrecovered left
wrapper, distinct wrapper/formula/inner/right provenance, ordinals 1/2 to the
same binding, three real expansions, four raw Outer inputs, one Base-RHS set
identity, two inferred terms, six type entries, two ordered expected constraints,
and one clean equality. Mode AST nodes are admitted only for the Task-246
nonempty config; old empty-mode configs remain closed. Five definition orders,
finite near misses/corruptions, cross-route, immutable/module, 59-owner
bidirectional isolation, and a real frontend/resolver sidecar protect runner
188. Trace counts are 403/367, type 235/223, and pass/fail 219/184.

## Task 266 Exact Final Checker Handoff

For the existing Task-180 source only, the runner preserves the actual module
root range, contradiction leaf site/range, and normal recovery state. It uses
the actual theorem surface site to validate the resolver-owner range, selects
exactly one real local resolver theorem owner, obtains a checker-validated
owner, and builds the exact three-node typed tree
`module -> theorem -> formula`, performs formula inference once, and supplies
one owner/formula row to final `ResolvedTypedAst` assembly. The final assertion
requires identical owner symbol/origin/range, existing checked contradiction
id/site/range/state/recovery, and the separate final typed-node identities.

Missing/duplicate rows, an invalid formula, wrong owner node, recovered source,
or any owner/formula/tree/range/provenance/source/module mismatch fails closed.
A synthetic source AST without a real resolver theorem owner remains an
extraction gap. The existing `.miz`, expectation, detail keys, stage, 272-test
list, runner counts, and four CLI outputs are unchanged. No truth/fact,
acceptance, proof, terminal-goal, Core, CFG, or VC payload is inferred.

## Task 267 Exact Proof-Intent Authority

Task 267 changes documentation only. Task 268 extends the existing Task-180
extractor so `mizar-test`, and no downstream crate, classifies the source's
omitted status and justification. The exact extractor must retain its current
whole-tree allowlist and verify one unrecovered `TheoremItem`, direct token
sequence `theorem SourceDerivedContradictionConstantBoundary : ;`, exactly one
contradiction formula child, no leading theorem-status annotation, no
justification node, and no additional structural child. It then emits explicit
`TheoremPolicyIntent::Unmodified` and
`TheoremJustificationIntent::Omitted`; checker/core must not infer those facts
from absence.

The syntax-free proof-intent row has explicit dense id, source order,
`StatementSemanticId`, source/module, owner symbol/node/range/origin, real
checked formula id/site/range, separate compact formula node, recovery,
resolver visibility/export, and the two intent enums. For this source,
id/source order/statement are zero, the real formula site is a Node site,
recovery is Normal, and the unrecovered top-level resolver theorem is
Public/Exported. The extractor and handoff cross-check every field against the
Task-266 owner/formula data. `Exported` means resolver name visibility, not
proof acceptance; core later preserves only public visibility.

Task 268 runner coverage must reject an annotation or written justification,
a missing/duplicate/nonzero/non-dense/reordered intent row, Role formula site,
recovery, non-Public visibility, non-Exported export status, and every owner/
formula/source/module/range/provenance/reference mismatch. Each negative case
must assert that no proof, proof-node, or terminal-goal table is published. It
must mutate the authenticated owner independently from the duplicated row and
then each row visibility/export field independently from the owner. It
must assert the exact singleton pending proof, direct terminal
goal, empty citations/context, no label, and local path `proof/0`. It reuses the
existing `.miz` and expectation. It also asserts deterministic nonempty
`ResolvedTypedAst::debug_text()` rendering of all three proof tables and
byte-identical Task-266 debug output when they are empty. It does not change
trace status, runner stage,
truth/facts, acceptance, proof search, Core/CFG/VC behavior, or Steps 6/7.

## Task 268 Exact Proof-Intent Implementation

The Task-180 contradiction extractor now returns a dedicated statement wrapper
only after proving that the theorem is unannotated, has no written
justification or proof block, and otherwise satisfies the pre-existing exact
whole-tree allowlist. The wrapper carries explicit `Unmodified` and `Omitted`
intent into the syntax-free checker row. The runner asserts the exact pending
proof, direct terminal, real formula site plus separate compact node, empty
citations/context, absent label, and `proof/0` path.

The corruption matrix covers bundle omission/duplication/order, every copied
identity/range/provenance/recovery/intent field, authenticated owner visibility
and export status independently from row values, Role-site substitution, and
checker output cross-references. Justification-clause and proof-block near
misses remain extraction gaps. Existing `.miz`, expectations, admission, test
names, counts, trace status, and four CLI outputs remain unchanged. Core Task
31 is the next consumer; no acceptance or proof-verification credit is added.

## VC Task 30 Prepared Phase-11 Runner

Task 30 reserves `MT10-VC-T180` solely for VC Task 31. The first
`proof_verification` / `active_proof_verification` route must accept only the
distinct `pass_proof_verification_contradiction_formula_constant_001` source/
sidecar with `expected_phase = "vc_generation"`,
run the source-to-checker-to-Core-to-VC path twice, require whole-`VcSet`
equality, and compare the complete phase-11 debug bytes. It must not reclassify
or admit the existing type-elaboration sidecar. Task-31 admission tests reject
the wrong stage, missing/duplicate/wrong active tag, and wrong
`expected_phase`. The runner/tag guard and first real
baseline are one logical Task-31 change; no empty route is permitted.

The later shared `MT10-VC-PV` route admits only the bounded
`MT10-VC-PV/VC<n>` slices owned by VC Tasks 32-55. Each slice must reject wrong
stage/tag/phase, missing or duplicate producer data, diagnostic-bearing Core/CFG,
stale handoff/intake, corrupt seed accounting, and nondeterministic output.
Task 30 itself changes no runner source, admission, case, or report bytes.

## VC Task 31 Exact Phase-11 Runner

Task 31 implements the prepared exact route. `proof_verification.rs` owns the
single-case admission predicate and validation diagnostic, calls the reusable
exact CoreIr producer, passes the immutable CoreIr to the exact mizar-vc
adapter with versioned schemas, repeats the full generation, and compares both
structural equality and complete debug bytes before verify-only baseline
comparison. The public report mirrors existing runner reports with stable
passed/failed/error/warning counts and a failure reason per case; the CLI
returns failure when any diagnostic is an error.

Admission is exact-id plus exactly one `active_proof_verification` tag, stage
`proof_verification`, phase `vc_generation`, pass outcome, `.miz` source, and
present snapshot. The old type-elaboration Task-180 case is excluded. Missing,
unreadable, mismatched, or absent snapshots and any source/Core/VC error fail
the case and emit stable task-local diagnostics. This exact route is not a
general proof verifier and publishes no accepted theorem or fact.

## Resolver R-031 Declaration-Symbol Increment Completion

R-031 adds exactly the existing
`fail_resolve_same_signature_same_return_conflict_001` sidecar to the active
`declaration_symbol` set. The unchanged `.miz` source reaches the real frontend
and resolver collector. The appended internal resolver class
`SameSignatureDefinitionConflict` maps only to
`declaration_symbol.signature.same_signature_definition_conflict`; the existing
`SameSignatureReturnConflict` mapping and different-return expectation remain
byte-identical. The same-return sidecar gains the active tag, exact diagnostic
payload, and active wording. No public numeric diagnostic is allocated.

The resolver groups only ordinary functor definitions by its exact syntactic
namespace/spelling/pattern/definition-context/arity key. All-identical returns
produce one new-class diagnostic; mixed/different returns produce one existing
return-conflict diagnostic over the complete candidate group, with no overlap.
This increments the active declaration-symbol count from four to five and changes that CLI
output/hash, while parse-only, type-elaboration, and proof-verification
admission remain unchanged.

## Parser Task 47 Parse-Only Increment

The parse-only runner admits
`pass_parser_reconsider_tails_001` as its 97th case. It executes the real
frontend/parser path and requires no diagnostics for both the omitted tail and
the proof-block tail. The existing explicit-`by` control remains active, and
the unchanged mixed recovery source still reports every non-Task-47 parser
error after its obsolete omitted-tail diagnostic is removed.

This increment changes only parse admission and plan bytes: plan 405/369,
parse-only 97/97, pass/fail 221/184, warnings/errors 23/0. Declaration-symbol,
type-elaboration, and proof-verification admissions remain 5/188/1.

## Parser Task 48 Property-Implementation Parse-Only Increment

The parse-only runner adds the pass/fail pair
`pass_parser_property_implementations_001` and
`fail_parser_property_implementations_recovery_001`. Both execute the real
frontend/parser path. The pass case covers top-level means/equals property
implementations, simple and case/otherwise definientia, the exact single
`let identifier be mode_application;` parameter, ordered mandatory
existence/uniqueness conditions for means, optional coherence, and the
supported justification shapes. The fail case pins bounded recovery and
preservation of the following declaration for malformed parameters, dots,
correctness ordering, and forbidden equals conditions.

The exact requirement
`spec.en.07.modes.property_implementation.parser` is now `covered` with
`pass_and_fail`. This is parser/syntax-only credit: the harness extracts no
property payload, makes no overlap or coherence decision, and grants no proof
acceptance or discharge. The inactive semantic Task-39 case remains unchanged.
The active totals are plan 407/369, parse-only 99/99, pass/fail 222/185, and
warnings/errors 23/0; declaration-symbol, type-elaboration, and
proof-verification admissions remain 5/188/1.

## Checker Task 248 Source/Binding-Context Increment

The type-elaboration runner admits one exact reserve-plus-definition-parameter
pass as case 189. It keeps raw `SurfaceAst` inspection in `mizar-test`, matches
both items against the real resolver `DeclarationShellSet`, and passes only
syntax-free shell, order, range, local-scope, declaration-site, and
written-type-site projections to `mizar-checker`. The runner then verifies the
same immutable `SourceBindingContextHandoff` in `TypedAst` and
`ResolvedTypedAst`, including distinct same-spelling reserve/local identities
and the structural shadow link.

This route emits no type result, expression, fact, obligation, formula,
statement, proof, Core, CFG, or VC payload. Invalid matched payloads fail with
one task-local internal detail key; they do not allocate a public diagnostic.
The exact requirement is a new bounded covered pass row, while the broad
payload-extraction row remains unchanged.

## Checker Task 249 Runner Boundary

The implemented `type_elaboration` increment is owned by one private `source_type`
leaf. It extracts only syntax-free type-head/application/argument projections
from ten reserve written-type roots in the named broad fail fixture and
asserts the checker-owned 10/13/6 handoff. Definition/import scaffolding is
excluded. The runner stops after that handoff with the single internal
readiness detail
`type_elaboration.checker.source_type_application.semantic_dependencies_pending`;
it does not request or credit normalization or later semantics.

The unchanged Task-248 pass route is the dependency regression and must
co-install exactly two `Bare`/builtin-`set` rows with zero arguments beside
the actual Task-248 source-context handoff. No raw syntax crosses into checker,
no public diagnostic is allocated, and no existing expectation or trace row
is rebaselined. The resolver-required distinct scaffolding formal/field names
repair only task-local `design_drift` and the parse-only preflight `test_gap`;
they emit no handoff rows and change no semantic intent.

## Checker Task 250 Frozen Runner Boundary

The future private `source_attribute` leaf owns raw-AST extraction for exactly
the existing Task-81 argument-bearing, Task-67 structure-qualified, Task-84
imported, and Task-85 negative-nonempty fixtures. Each route co-installs the
real 1/1/0 Task-249 dependency handoff. Their aggregate immutable Task-250
handoff oracle is four nonempty chains, four attributes, one qualifier, one
parenthesized argument group, and one actual, with three positive/one negative
polarity and two local/two imported attribute identities.

The Task-81/67 sidecars advance only to the runner-owned source-attribute
semantic-dependency detail. Task-84/85 preserve their checker evidence-query
details and legacy `AttributeInput` routes. Exact sibling selectors prevent
Task-116, Task-171, Task-77, or any broader case from receiving credit.
Synthetic private-extractor tests additionally cover multi-attribute order
and single/parenthesized prefix punctuation and actuals from `SurfaceAst`
through the public checker handoff; checker-input-only tests are insufficient.
The exact probe is `p-ranked (q,2)-graded set`, with Task-249 1/1/0 and
Task-250 1/2/0/2/3 tables, one single identifier prefix, and one
parenthesized identifier/numeral prefix with exact comma/delimiter/hyphen
provenance.

No raw syntax crosses into checker, no new `.miz` or public diagnostic is
planned, and no semantic arity, prefix/list equivalence, admissibility,
evidence, or truth is inferred. This frozen boundary is documentation-only
until Checker Task 250 implementation.

## Checker Task 250 Source-Attribute Consumer

The private `type_elaboration::source_attribute` leaf now owns the frozen raw
attribute extraction and checker handoff. Only the existing Task-81/67/84/85
fixtures select it. The four routes publish aggregate Task-249 4/4/0 and
Task-250 4/4/1/1/1 tables with exact polarity, qualifier, punctuation, actual,
and local/imported provenance. A private synthetic `SurfaceAst` test runs
`p-ranked (q,2)-graded set` through the same extractor and public checker
producer to prove multi-attribute order and single/parenthesized prefix
projection.

Task 81/67 progress only to the runner-owned semantic-dependency detail;
Task 84/85 keep their evidence-query outcomes. No other route, `.miz` source,
semantic acceptance, public diagnostic, or later checker payload is changed.

The current production layout is 21 paths / 23,184 lines, with sorted
path/content hashes `bd42d60f...` / `d1421834...`; `runner.rs` remains
facade/top-level orchestration only and the new private leaf is the sole added
production path. The mizar-test library has 283 tests.

## Checker Task 251 Frozen Runner Boundary

The private `type_elaboration::source_evidence` leaf owns exact
Task-249-broad plus Task-84/85 dispatch. It publishes ten checker-owned
transport requests: five mode-expansion, three structure-inhabitation, and two attributed, all
missing and with no dependency reference. The three-route dependency oracles
are Task-249 12/15/6 and Task-250 2/2/0/0/0; broad alone remains 10/13/6.

This leaf owns request/response association and exact dispatch, not another
raw extractor. Existing `source_type`/`source_attribute` leaves retain their
selectors and AST traversal. Only the narrow crate-private Task-250 output
visibility/factor needed for production reuse may change; duplication,
movement, selector widening, and extraction behavior changes are forbidden.

The broad sidecar alone advances to the runner-owned missing-dependency detail.
Task 84/85 retain their checker evidence-query details. No sibling, `.miz`,
public diagnostic, semantic evidence result, or later payload is changed.

Library tests must use real source extraction and the production consumer for
the exact counts, sibling isolation, and requested/missing/rejected/supplied
injection through final `TypedAst`/`ResolvedTypedAst`. A supplied reference is
not evidence acceptance. Corrupt input fails atomically rather than publishing
`Rejected`. Implementation adds four tests, moving the documented library
total from 283 to 287. Exact selection, four-state injection, final ownership,
corruption, and deterministic replay pass on the production path.

## Checker Task 252 Frozen Runner Boundary

The private `type_elaboration::source_term` leaf has exactly three real
selectors: builtin numeral equality, bare reserved-variable equality, and
single-left-parenthesized reserved-variable equality. Their aggregate public
handoff oracle is seven primary terms, four binding references, and two
numeric-type requests. No source case is added and no current outcome,
diagnostic, or detail changes.

The leaf owns only raw primary-term occurrence, binding-role, parent edge, and
numeric-request extraction. It must preserve the legacy semantic producers
and formula routes, use their exact helpers where applicable, and publish the
new handoff through `TypedAst` and clone-only `ResolvedTypedAst`.
Parenthesized rows are source wrappers only; they add no independent semantic
term/type/FOL row.

Synthetic runner probes cover a `LocalAbbreviation` identifier, an `it`
surface with only its current-result source role, and eligible nested
parentheses through the same extractor/producer. References must authenticate
the exact lexical `BindingEnv::lookup` winner with scope derived from the
term context and use ordinal derived as the count of preceding completed
binding rows; prior references do not advance it. Exact consecutive
duplicate-priority binding groups share all lookup-priority inputs and use
their final dense row index as the shared visibility ordinal, so an ambiguous
winner is rejected by lookup. Parentheses containing a
later-family term are excluded until Tasks 253-255 freeze cross-family edges.
Real constant declaration ownership and real `it` owner/type are deferred to
Tasks 269 and 260/264. The probes create no
fixture, runner admission, semantic result, formula, fact, or coverage credit.
Implementation tests prove exact selection, corruption isolation,
deterministic replay, and final ownership. Four tests move the library total
from 287 to 291. The raw/normalized sorted-list hashes are
`d46edefebc54a2f2f170cbfce8143ed036fa7ce339ebb3a746d89b55293931e5` /
`f7b5babbf33e1e3e3afe4c49018744a4a0fe42968fd2e5edc411eb7bc49fc0a6`.
The private leaf is the sole new production path, producing 23 paths / 24,120
lines with path/content hashes
`562224fc62e93a256f5d3891e3a466a45ec23c24055e3a9f3f83848a0672a16b` /
`8a4b76e37a8a6921ed89e98372ccb037cd64ed583ac0bbe26466924ef0c4b028`.

## Checker Task 253 Frozen Runner Boundary

This is Checker Task 253, not the already-completed `mizar-test`
runner-refactor Tasks 253A/253B. The future private
`type_elaboration::source_application` leaf selects only the existing
imported `1 divides (1 ++ 2)` case and the exact module-local
`task253_local_source(x)` closure in the second definiens of the new source
frozen in the paired checker plan. It composes the Task-252 primary-term
producer for actual occurrences and publishes exact aggregate Task-253 tables
2 applications / 1 wrapper / 2 candidate references / 3 arguments /
4 requests, with Task-252 slice 3/1/2.

The leaf owns raw application/head/form/wrapper extraction, ordered edge
projection, and individual resolver-reference provenance. It does not
duplicate primary rows, claim a complete/viable candidate set, select a
winner, or publish a signature/result. The imported outer parentheses are a
Task-253 cross-family wrapper/origin. For the local source, it reuses the
Task-248 reserve-then-definition two-item/two-binding shadow handoff and
requires the actual to resolve to `BindingId(1)` /
`BindingContextId(1)` / `use_ordinal == 2`, never `BindingId(0)`. Existing
imported outcome/detail fields remain unchanged. The new local route validates
the application transport and then preserves the generic
`definition_declaration_payload_extraction_gap` /
`type_elaboration.external_dependency.ast_payload_extraction` boundary, with
no public diagnostic.

Synthetic `SurfaceAst` transactions use the same extractor/producer for
remaining ordinary forms, nested applications, primary/application
parentheses, definition-parameter actuals, and multiple authenticated
candidate references. Inline zero/one/two-actual cases split the test:
raw-AST probes assert only generic `ApplicationTerm` plus mandatory
parentheses, and caller-supplied producer DTOs assert schema with no candidate
or request. The production extractor never infers `Inline`; Task 270 owns
identity/formals/capture/substitution. Template application subtrees emit no
Task-253 row: Task 277
owns their direct role/actual/guard/request transport, while Task 278 owns
ordinary/template candidates and selection. Tests must also prove the frozen
corruption matrix, sibling isolation, deterministic replay, dependency
fingerprint substitution rejection for any non-equivalent same-source/module
Task-252 handoff, acceptance of an equivalent clone, and final
`TypedAst`/`ResolvedTypedAst` ownership. The
private-selector matrix rejects imported missing/wrong/duplicate provenance,
wrong `++` head/form/arity, wrong numeral order, and recovery; local functor
order reversal, forward use, wrong or extra head/actual/application/item,
recovery, and selection of outer `BindingId(0)`; and proves bidirectional
exclusion between both Task-253 routes and every existing Task-252 selector.

This documentation prerequisite changes no runner route, test list, layout,
or hash. The current 291-test and 23-path/24,120-line Task-252 baselines remain
exact; projected implementation counts are plan 412/376, type 242/230,
pass/fail 224/188, admissions 101/5/191/1, and warnings/errors 23/0, subject
to fresh measurement.

## Checker Task 253 Runner Completion

The private leaf is active for exactly the two frozen consumers. It composes
Task 252 rather than duplicating primary rows, installs the Task-253 handoff
after the exact dependency fingerprint validates, and verifies clone-only
final preservation. Real tests measure 2/1/2/3/4 and 3/1/2, the local inner
binding coordinates, the imported wrapper, corruption isolation,
deterministic replay, and exclusion of every other active type-elaboration
case. The measured corpus is plan 412/376, type 242/230, pass/fail 224/188,
admissions 101/5/191/1, and warnings/errors 23/0.
The 303-test raw/normalized list hashes are `a81f44fb...` / `1a621c56...`;
the 24-path/25,607-line production path/content hashes are `5cc36b8a...` /
`b9b6c678...`. Exact values and all five CLI hashes are recorded in the paired
module-boundary audit.

## Checker Task 254 Frozen Runner Boundary

The future private `type_elaboration::source_structure` leaf selects only the
three definientia in the exact
`fail_type_elaboration_local_structure_term_gap_001` source frozen by the
paired checker plan. It composes the existing Task-252 producer and must
publish Task-254 term/wrapper/root/member/field-update/edge/request
5/0/3/9/2/10/26 plus Task-252 primary/reference/numeric-request 8/0/8.
No Task-253 row or fingerprint occurs in the real route.

The leaf alone inspects `StructureConstructor`, `SelectorAccess`,
`StructureUpdate`, `FieldArgument`, and `FieldUpdate`. It preserves member
segments, repeated written labels/paths, ordered children, and transparent
wrappers, but does not classify fields/properties or compute a semantic
result. Synthetic transactions cover constructor cardinalities and nesting,
selector chains/calls, nested update paths, every Task-252/253/254 target
kind, wrappers, degraded recovery, local/imported roots, signature-shell
states, corruption, deterministic replay, dependency substitution, final AST
ownership, and whole-subtree exclusions. Cross-family Task-253 application
targets must be roots not targeted by any Task-253 argument edge and must
share the owning Task-254 context; nested Task-253 applications are rejected
rather than multiply owned. Reverse applications containing structure
children remain excluded.

The real case stays at
`definition_declaration_payload_extraction_gap` /
`type_elaboration.external_dependency.ast_payload_extraction`, with no public
diagnostic. Existing parser/resolver guard fixtures and all other active
cases keep their bytes, sidecars, stage, status, and credit. This prerequisite
changes no route, test list, layout, or hash; current 303-test and
24-path/25,607-line baselines remain exact. The separate implementation
projects plan 413/377, type 243/231, pass/fail 224/189, admissions
101/5/192/1, and warnings/errors 23/0, subject to fresh measurement.

## Checker Task 254 Runner Completion

The production dispatch now routes only the exact local structure-term
fixture through the private `source_structure` leaf before the Task-253
application route. The leaf consumes declaration shells and Task-248 binding
contexts, composes Task-252, publishes Task-254 5/0/3/9/2/10/26 plus
Task-252 8/0/8, and preserves the frozen external-gap detail. It creates no
generated definition context and no semantic structure/member/view result.

Nine focused tests cover the real oracle, corruption atomicity, isolation
from all 191 other active type cases, the frozen syntax/recovery/subtree matrix,
synthetic boundaries, imported producer provenance, Task-253 root/child
fingerprints, and unrelated-handoff preservation. The measured corpus is
413/377, 243/231, 224/189, admissions 101/5/192/1, and warnings/errors 23/0.
The 312-test raw/normalized list hashes are `b7f56668...` / `09acdf12...`;
the 25-path/27,317-line production path/content hashes are `e81c3b08...` /
`3046ae27...`. Exact values and all five CLI hashes are recorded in the
paired crate plan and module-boundary audit.

## Checker Task 255 Frozen Runner Boundary

The future private `source_set_term` runner leaf validates only the frozen
four-definiens local case. Its exact oracle is Task-255
terms/wrappers/generators/type-sites/edges/requests 4/0/1/3/4/7 plus
Task-252 terms/references/numeric requests 4/0/4, with no real Task-253/254
dependency. Raw syntax remains private; the checker receives only the six
syntax-free tables.

The route admits a condition-free comprehension whose mapper does not use its
written generator. Generator binding/capture, condition formulas, non-bare
target types, semantic sethood, choice nonemptiness/stability, and `qua`
widening are not fabricated. The implementation must prove exact-source
selection, all-active-case isolation, syntax/recovery/subtree exclusions,
corruption, deterministic replay, dependency fingerprints, and immutable
final preservation. This prerequisite changes no runner, fixture, sidecar,
trace, count, or hash.

## Checker Task 255 Runner Completion

The private route now implements that boundary and preserves the frozen
external-gap sidecar. The exact real aggregate is Task-255 4/0/1/3/4/7 plus
Task-252 4/0/4, with no Task-253/254 fingerprint and no semantic output.
Recursive extraction normalizes generator/type-site IDs after visiting nested
mappers so term-grouped public order and written type-site order are both
preserved.

Focused real and synthetic tests authenticate every public row association,
zero/many and nested shapes, wrappers/recovery, optional cross-family targets,
explicit exclusions, corruption atomicity, deterministic replay, final
ownership, and isolation from every other active type-elaboration case. The
active corpus is 414/378, 244/232, 224/190, and 101/5/193/1 with 23 warnings
and zero errors. Later binder, formula, and semantic owners remain deferred.

## Checker Task 256 Frozen Runner Boundary

The future private `source_atomic_formula` route reuses eight existing active
fail cases and adds no source case. It validates one syntax-free public
Task-256 transaction per source before preserving the current semantic
bridge and exact external-gap outcome.

The real aggregate is Task-256
formula/wrapper/head/candidate/type-site/attribute/edge/request
`8/0/1/1/1/2/13/11`, Task-252 `16/0/16`, Task-253
`1/1/1/2/2`, and Task-255 `2/0/0/0/4/2`, with no Task-254 target. Raw
syntax and selection stay private; the checker receives only dense rows,
resolver identities, cross-family IDs, and unresolved requests.

The route first forms the complete Task-252 primary union in one shared
handoff and arena, then builds its Task-253 and Task-255 dependencies against
those same objects before Task 256. It may reuse narrowly exposed private
helpers, but it does not widen or alter any existing lower-family standalone
selector or allowlist.

The runner must prove exact-source selection, all-active-case isolation,
candidate/attribute provenance, bare assertion-type ownership, direct-child
exhaustiveness, request associations, dependency fingerprints, both install
orders, corruption atomicity, deterministic replay, and immutable final
preservation. It must not change the existing sidecars except the separately
frozen reciprocal trace reference/transport note during implementation.
The positive oracle is an exact ordered vector of all thirteen edge rows and
eleven request rows, including IDs, ordinals, roles/kinds, targets/
associations, effective ranges, the Task-253 outer parenthesized range, and
the positive/negative attribute target plus `non` anchors. Tests also retain
the unchanged Task-252/253/255 standalone-selector isolation oracles.

Predicate chains/negation, inline/templates, general type graphs,
qualified/argument-bearing attributes, semantic facts/truth, theorem
acceptance, and conditioned comprehensions remain excluded. This
documentation prerequisite changes no runner, fixture, sidecar, trace,
count, test list, production manifest, or hash.

## Checker Task 256 Runner Completion

The private `source_atomic_formula` route now implements that frozen
boundary. For each exact source it forms one complete Task-252 primary
handoff and arena, builds any Task-253/255 dependency against those same
objects, validates the public Task-256 transaction, installs it through
`TypedAst` and `ResolvedTypedAst`, and then leaves the pre-existing semantic
detail route in control.

The eight transactions produce exactly Task-256 `8/0/1/1/1/2/13/11`,
Task-252 `16/0/16`, Task-253 `1/1/1/2/2`, and Task-255
`2/0/0/0/4/2`, with ten primary edges, one root-application edge, two root
set-term edges, and no Task-254 target. Tests assert every ordered edge and
request row, independent resolver/source anchors, the unchanged eight detail
vectors, selector isolation, atomic corruption, synthetic cardinality and
shape variants, installation/revalidation boundaries, and final ownership.

No `.miz`, outcome, phase, category, rejection reason, stable detail,
diagnostic payload, or tag changes. Predicate chains/operators/binders,
general type and attribute graphs, semantic facts/truth, conditioned
comprehensions, inline/templates, and overload selection remain deferred to
their frozen owners.

## Checker Task 257A Frozen Runner Boundary

Checker Tasks 257A-C here are checker producer slices, not the completed
mizar-test Tasks 257A-H test-layout series recorded elsewhere in this
document.

The one exact route is
`fail_type_elaboration_formula_connective_quantifier_gap_001`. Raw
`SurfaceAst` traversal stays private. `source_formula.rs` selects and retains
the five formula sites, explicit binder segment and identifier, bare `set`
type-expression/head, and frozen ranges; a private
`source_composite_formula` leaf may translate that data into the public
syntax-free transaction.

The runner first constructs the exact normal `1/0/4` module-shell
environment through public `BindingEnvParts` and table APIs, from the
authenticated source/resolver/symbol-module identities. It leaves the older
private `1/0/0` semantic helper unchanged. It then derives the exact `2/1/4`
environment from the same immutable Checker Task 257A input, with a dedicated source-formula
expression context and one resolver-shaped local quantifier binding. It does
not fabricate a Checker Task 248 source-context handoff. Because `x` is unused, it
builds no Checker Task 252/253/254/255/256 term-family dependency. It then builds and
installs the seven-table `5/0/1/1/1/4/6` handoff before invoking the
unchanged older semantic route.

Positive tests assert all formula, root, binder, type-site, edge, and request
rows in order, including exact sites/ranges, context 0-to-1 transition,
declaration/type provenance, and the unchanged two-key detail vector.
Negative tests cover selector isolation, recovery and spelling changes,
tree/parent/role/cardinality corruption, stale binding/context/type identity,
wrappers, deterministic replay, one-shot installation, dependency
revalidation, the sole standalone Checker Task 257A installation sequence,
synthetic preinstalled Checker Task 248 source-context rejection, and immutable final
ownership. The executable rejection starts with
`TypedAstParts { source_context: Some(task_248_handoff), .. }`; no
reverse-order test is claimed because no public source-context installer
exists. `source_context()` remains `None`; only the Checker Task 257A handoff owns the
extended environment. No lower-family selector or allowlist is widened.

The positive oracle uses one full literal handoff `debug_text()` snapshot
containing the complete embedded environment and every seven-table row, plus
the exact legacy `TypedAst::debug_text()` bytes without Checker Task 257A. Equality between
two reruns and substring presence are supplemental, not substitutes.

Broader formula shapes, bound use/capture, predicate chains, conditioned
comprehensions, theorem ownership, and all semantic answers remain deferred.
This documentation prerequisite changes no runner, fixture, sidecar, trace,
count, test list, production manifest, or hash.

### Checker Task 257A Implemented Route

The runner now executes the frozen route before the unchanged semantic-detail
owner. It uses the extended private extraction record to preserve the exact
formula, binder-segment, identifier, type-expression, and type-head sites;
assembles the public transaction without exporting syntax types; validates
and installs the handoff; and clone-preserves it into the resolved result.
Five private tests cover the exact real oracle/final ownership, independent
table corruption with recovery, all-active and lower-family isolation,
unchanged semantic details, and preinstalled Task-248 rejection.

## Checker Task 257B1 Frozen Runner Boundary

The exact 79-byte pass source is selected in the existing formula extraction
owner and composed in a dedicated private Task-257B1 leaf. The leaf must use
the Task-257 binding extension before building the two Task-252 primary
references, the Task-256 equality, the one-node Task-257 composite profile,
and the `1/2` formula-composition handoff in one arena.

Positive tests assert the exact parser ranges, both binding lookup winners,
all lower/composite/composition rows, dependency fingerprints, final typed and
resolved ownership, and absence of semantic output. Negative tests cover
selector isolation, recovery, every cross-family association, dependency
substitution, corruption recovery, and byte preservation of the existing
Task-257A route. They also construct `TypedAstParts` with a preinstalled
Task-248 source-context handoff and require the combined installer to fail
without publishing either Task-257B1 handoff. No current active selector or
semantic route is widened.

The positive composition oracle is one full literal
`source-formula-composition-debug-v1` rendering: module, exact Task-252/256/257
debug-string fingerprints, edge count/row, and both bound-use rows in field
order. Typed and resolved getters must expose the identical handoff after the
existing composite-formula debug section; an absent B1 handoff preserves all
legacy bytes.

The ownership-partition matrix also passes a valid second-profile handoff to
the Task-257A legacy installer and passes a combined B1 transaction to an AST
that already owns Task 257A; both must fail with byte-identical rollback. The
profile matrix mixes A cardinalities with B rows, B cardinalities with A rows,
and supplies a third otherwise valid shape. Only the two exact profiles may
build, and neither failed build/install path may publish partial B1 state.

### Checker Task 257B1 Implemented Route

The private leaf now executes the frozen selector and same-arena composition.
Its positive oracle covers every Task-252/256/257/B1 row, both lookup winners,
all dependency fingerprints, the full literal composition debug rendering,
and typed/resolved ownership. The isolation, corruption, ownership-partition,
profile-discriminator, Task-248, and Task-257A rollback cases remain
executable, and the external result contains no semantic formula output.

## Checker Task 257B2 Frozen Harness Boundary

The dedicated private formula-composition leaf may be extended for exactly one
additional case ID:
`pass_type_elaboration_formula_connective_grouping_payload_001`. Selection
requires the exact theorem label/tree, normal recovery, explicit `x being set`
binder, fixed/repeated operator flags and tokens, wrapper placement, equality
endpoints, and no extra item or justification.

The runner builds Task-252, Task-256, the third Task-257 profile, and `8/0`
composition in the same arena and installs the composite/composition pair
atomically. Positive assertions cover all `16/0/16`,
`8/0/0/0/0/0/16/16`, `8/6/1/1/1/7/9`, and `8/0` rows. Negative
assertions cover source isolation, each profile/association mutation,
fixed/repeated substitution, wrapper crossing/order, dependency replacement,
ownership collisions, and recovery by replaying valid input.

The source-selector near-miss matrix independently changes status or
justification, adds another item, changes the binder or binder type, reorders
or changes the operand count, changes wrapper depth or placement, substitutes
fixed/repeated tokens, supplies an alternate connective tree, and supplies
representative Task-257B3 binder and Task-257C
predicate-chain/conditioned-comprehension shapes. Every variant must remain
unselected. The checker profile-discriminator matrix also supplies one
coherent otherwise-valid fourth profile, which must fail without publishing
partial state.

The active case must pass with no detail key and no types, facts, checked
formula semantics, statement semantics, theorem acceptance, proof, or IR
credit. The existing Task-257A and Task-257B1 selectors and byte oracles remain
unchanged.

## Checker Task 257B2 Implemented Harness Route

The route now performs the frozen extraction and same-arena producer sequence,
then atomically installs the composite/composition pair. The active case
passes with no detail key; selector isolation, source-level near misses,
profile/association corruption and valid replay, A/B1 preservation, and final
typed/resolved ownership are executable tests. No semantic output or theorem
credit is produced.

## Checker Task 257B3 Frozen Harness Boundary

The existing private formula-composition leaf may later select only
`pass_type_elaboration_formula_nested_quantifier_payload_001`. It must require
the exact reserve/theorem pair, normal recovery, restricted `x`, existential
`y`, implicit-reserve nested `r`, three equality atoms, no extra item or
justification, and the final-LF source hash.

The route prepares the Task-48 reserve binding environment, keeps Task-248
source context absent, extends three nested quantifier binders, and builds the
exact `6/6/0`, `3/0/0/0/0/0/6/6`, `3/0/1/3/3/2/6`, and `3/6`
profiles in one arena. Near misses independently mutate reserve presence,
name/type/order, explicit versus implicit binders, quantifier kinds, `st`/
`holds`, nesting, atoms, lookup winners, recovered nodes, theorem label and
sidecar status/justification, an extra theorem/definition/non-reserve item,
formula wrappers, attributed or argument-bearing reserve/binder types, and
B2/C shapes. Passing asserts only immutable source transport: no truth,
witness, closure, fact, theorem, proof, or IR output.

## Checker Task 257B3 Implemented Route

The exact selector and same-arena transaction are active. Tests now cover the
real ranges/scopes/provenance, cumulative visibility and shadowing, all six
lookups and owning edges, active-corpus exclusivity, recovered and structural
near misses, aggregate corruption/replay, transport-only sidecar, and final
typed/resolved ownership. The selector receives the loaded source text and
requires the frozen 138 bytes including the final LF; missing-final-LF and
whitespace-only variants are independently rejected.

## Checker Task 257C1 Frozen Runner Route

The private exact route will recognize only the frozen 107-byte
`FormulaPredicateChainPayloadBoundary` source. It must verify two predicate
segments, same imported `divides` provenance, normal `does not` token ranges,
Task-252 `3/0/3`, extended Task-256
`1/0/2/2/2/0/0/3/2`, and one shared edge id for middle term `2`.
The selector receives loaded source text and guards its final LF.

Runner matrices cover every named source near miss, parser recovery,
mixed/built-in chains, segment/polarity/edge/candidate/request corruption,
active-corpus isolation, old route preservation, atomic install/rollback,
resolved clone, and empty semantic output. The future sidecar carries only
immutable source transport; it adds no truth, negation result, predicate
winner, theorem result, proof, or IR detail.

The extended input field reaches four current runner literals: the sole
atomic-formula constructor conditionally emits the exact two C1 segments and
stays empty for prior routes; all three formula-composition constructors
remain empty. These literal edits are required for compatibility only and do
not activate a Task-257 composition route.

### Task 257C1 runner status

The exact source route, pass sidecar, covered trace row, loaded-text near-miss
matrix, input-corruption matrix, active-corpus isolation, and unchanged prior
routes are executable. Its successful external detail vector is empty; all
semantic predicate and theorem work remains deferred.

## Checker Task 255C1 Frozen Runner Boundary

The private source-set leaf will recognize only the exact 191-byte
conditioned-comprehension definition. It receives loaded source text,
authenticates final LF and parser ranges, derives the unique imported `++`
candidate, and rejects every named structural, recovery, provenance, and byte
near miss before publication.

It produces Task-252 `4/0/4`, reuses the Task-253 private builder for
`1/0/1/2/2`, and produces Task-255
`1/0/1/1/1/1/2` in one arena. The condition colon is Task-255-owned; its
direct `FormulaExpression` wrapper is separately Task-255-anchored; and its
two numeral operands are Task-252-owned and untargeted by Task 255. No
Task-256/257 handoff or semantic table is produced. The future fail sidecar
retains the definition-extraction detail key after successful source
transport.

### Checker Task 255C1 Runner Status

The frozen route is implemented. Exact loaded bytes and parser/resolver
provenance select one transaction; every named near miss returns before
publication. The runner composes the shared `4/0/4`, `1/0/1/2/2`, and
`1/0/1/1/1/1/2` handoffs, leaves the inner condition formula unowned, and
preserves empty semantic output and every prior route.

## Checker Task 257C2 Frozen Runner Boundary

The private complete route reuses the exact Task-255C1 loaded-source selector,
Task-253 imported-application seam, and a reusable Task-256 equality builder
before the lower Task-256/255 diagnostic routes. It extends the same arena
with one atomic equality and one immutable condition-formula association.
Every Task-252/253/255 row and site remains unchanged.

At the frozen pre-Task-256C1 baseline, this target route was gated because the
lower validator rejected the enclosing condition set term in both set/atomic
install orders. The separately completed checker prerequisite now passes
those orders only for the authenticated condition relation and keeps
arbitrary overlap fail-closed. The route awaited fresh Task-257C2 preflight
and implementation at prerequisite exit. It is now complete:
production publishes the exact Task-256 equality and sole Task-257C2
association before the lower diagnostic-only routes.

The existing fail case retains its definition-intake detail and semantic
tables remain empty. Exact profiles, direct wrapper/equality ownership,
provenance, near misses, mutation rollback, bidirectional A/B/C2 installer
exclusion, isolation, and final cloning are the complete test boundary. This
prerequisite changes no executable runner artifact.

The separate implementation now passes four runner tests, retains the
unchanged extraction-gap sidecar detail and empty semantic tables, and
measures plan/type `419/386` / `252/240`, 361 runner tests, and the
29-path / 34,064-line production manifest.

## Checker Task 256C1 Frozen Harness Non-Ownership

No harness edit or test was part of Task 256C1. The checker-local syntax-free
fixture now authenticates the exact lower relation and both install orders;
the private runner continues to stop at the committed Task-255C1 route. The
already frozen Task-257C2 route remains the first runner consumer after fresh
post-commit preflight of the completed checker prerequisite; that consumer
has since been implemented and verified without changing the fixture or
semantic detail.

## Checker Task 257C3 Frozen Harness Boundary

The future harness route reuses the exact Task-257C1 selector and lower
builder, then installs the separate `1/1` predicate-chain composition before
the lower route can return. It introduces no raw traversal beyond the
existing predicate-chain extractor and no new fixture. Success requires equal
typed/resolved handoffs and the same empty semantic detail vector. The only
future metadata changes are one reciprocal sidecar reference/note and one
covered trace row; this prerequisite changes neither.

## Checker Task 257C3 Harness Result

The complete route now executes before the lower atomic-only return and
installs the exact syntax-free C3 handoff into fresh typed and resolved
owners. The extractor still owns only the existing predicate-chain syntax;
the new composition leaf consumes lower handoffs and copies no syntax or
resolver row. Primary/atomic/composition/arena mutation, exact replay,
selector near misses, route isolation, typed/resolved debug order, and clone
preservation are covered by exactly four runner tests. The fixture and empty
semantic detail remain byte-for-byte unchanged.

## Checker Task 258A Frozen Harness Boundary

The later harness leaf parses/resolves the exact 81-byte future `MT10-FS`
source and builds Task-252 `2/2/0`, Task-256
`1/0/0/0/0/0/2/2`, and Task-258A `1/1/1/1/1` in one transaction. It is
production-capable but has no corpus dispatch until `MT10-FS` adds its
distinct `.miz` and singular sidecar. Four library tests execute the real
frontend/resolver path now and freeze the left/right Task-252 stored use
ordinals as 1/1 independently from the upstream binding/use source-event
lookup ordinals 1/2.

No active type-elaboration route, detail vector, fixture, sidecar, trace row,
or admission rule changes. The existing reserved-variable equality case must
not select this exact-name route. Success requires equal typed/resolved
source-statement handoffs, the same owned binding environment/fingerprint,
absent Task-248 source context, and empty checked formula/statement/proof/
fact/diagnostic output. This prerequisite changes no executable harness
artifact.

## Checker Task 258A Harness Result

The dedicated private leaf now runs before the lower Task-257C1 route and
uses the real frontend, declaration-shell symbol collection, label resolver,
Task-48 binding bridge, and Task-252/256 builders. The exact
`1/1/1/1/1` handoff is installed into fresh typed/final owners with only
`source.statement.transport` source-preserved node hints. Four library tests
cover the real provenance path, dependency and row corruption, loaded-source
and subtree near misses, active-route isolation, ownership exclusion, and
atomic replay. The future corpus fixture and sidecar remain absent.

The runner library now has 369 tests with raw/normalized test-list hashes
`c5764bb1600242fe44db8c44b9c6bb18f39203a0de9ff60e301cbc6f172037f6` /
`1fd27b9bff190f95ac23d6de714a919a192fb0b7830aa3c98f960d4224c084aa`.
Production is 30 paths / 34,955 lines with path/content hashes
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`dd399648aecadf2e7a63f685ad87577b7ebae9a9064fbfaba429a07d25ed9912`.

## Checker Task 258B1 Frozen Harness Boundary

The later exact private leaf recognizes only the 139-byte final-LF
`FormulaStatementNestedContextSmoke` source and remains dormant outside
library tests. It uses the real parser and resolver, extends the reserve
binding base with outer/nested proof contexts, and composes Task-252
`8/8/0`, Task-256 `4/0/0/0/0/0/0/8/8`, Task-258B1 base
`1/4/4/4/4`, and local-reference `1/1` in one shared arena. The public
resolver result must contain exactly one proof-step label and one resolved
local citation; the runner cannot synthesize either row. A preliminary real
77-node/root-76 resolver arena supplies genuine node 68, then a second
same-index pass attaches the resolver-produced `Label(0)` key and resolved
state only to that node before `ResolvedAst::try_new`. The exact projection
uses label-token node 12, the reference candidate uses node 68, and the
validated `ResolvedAst`, projection, candidate, and result cross as one
replayable syntax-free bundle rather than relying on the lossy result table
alone. Exact `SurfaceNodeKind` parity remains runner-validated.

The leaf runs before Task 258A and lower Task-257 selectors, copies no syntax
or resolver object into checker-owned rows, and publishes only equal
typed/resolved syntax-free handoffs. Exactly five future library tests cover
real ranges and provenance, two-pass/final keyed resolver-AST identity,
contexts/visibility, dependency and arena corruption, exact selector/subtree
near misses, Task-258A and active-route isolation, rollback/replay, and empty
semantic output.

This prerequisite changes no runner source/test, fixture, sidecar,
expectation, trace metadata/status/count, route, production manifest, test
list, or hash. Assumptions, witnesses, composite formulas, broader label
visibility, proof meaning, and acceptance remain outside Task 258B1.

### Task 258B1 Implemented Harness Boundary

The frozen route is now implemented and remains corpus-dormant. The private
leaf performs exact source selection, real parser/resolver extraction,
proof-context binding extension, per-context Task-252 lowering, Task-256 and
statement/reference assembly, and atomic final publication. It returns no
semantic statement/proof payload and cannot activate an existing case.

Five library tests raise the runner list to 374. Raw/normalized hashes are
`e8b5f54f219f8aa091014557c38ff8018d229ffbbc01cfa449bdc215826ca105` /
`99e6b7199e007707d1b4074b7079885e58378c4900a6811a7e1eb6cc02f9a2bf`.
Production remains 30 paths / 35,854 lines.

### Task 258B2 Frozen Harness Boundary

The Task-258B2 leaf recognizes only the exact 113-byte final-LF source and
the measured 55-node/root-54 unrecovered parser tree. It validates the
reserve, theorem label, theorem/proof/assumption/conclusion ranges, all six
primary `x` term sites, and the one public/exported resolver theorem at
origin `[2, 1]`, contribution 0. Any label/citation/reference key, witness,
nested proof, composite root, selector, or extra statement is a near miss.

The leaf runs before Task 258A/B1 and lower Task-257 selectors. It privately
translates raw syntax into Task-48 `2/1/0`, Task-252 `6/6/0`, Task-256
`3/0/0/0/0/0/0/6/6`, and the base-only Task-258B2 `1/3/3/3/3` handoff,
without copying raw syntax into checker rows. Exactly five future tests cover
the exact route, lower/resolver/table mutation and rollback, near misses,
route isolation, typed/final clone, and invariant empty fact/premise/
checked-formula/statement-semantic/proof/goal/diagnostic output.

This documentation prerequisite changes no runner source, executable route,
test, corpus artifact, trace metadata, production manifest, test list, or
hash. Witnesses, composite roots, broader visibility, and proof meaning stay
with Tasks 258B3–B5 and 269–272.

### Task 258B2 Implemented Harness Boundary

The private leaf and facade are implemented at 2,120 and 678 lines;
`runner.rs` is 2,491 lines and the statement test leaf is 2,884. The route
stays corpus-dormant and precedes the prior statement/lower selectors.
All-index parity and the complete resolver mutation matrix prevent raw-syntax
or provenance drift from crossing the syntax-free checker boundary.

### Task 258B3 Frozen Harness Boundary

The future leaf accepts only:

```mizar
reserve x for set;
theorem FormulaStatementSingleWitnessSmoke: x = x proof
  take x;
  thus x = x;
end;
```

It checks exactly 104 bytes, final LF, SHA-256
`76fb48354fc0dfb17047900a047a5b28b806df60d139a3133e606f0ef12a3f82`,
49 real unrecovered nodes/root 48, theorem node 45, proof 44, take 35,
witness 34, transparent term wrapper 33, Task-252 term/reference site 32,
the complete Task-252 site sequence 26/28/32/36/38, transparent wrappers
27/29/33/37/39, Task-256 atomic sites 30/40 under wrappers 31/41, conclusion
43, and the one
public/exported theorem owner. There is no resolver label/reference bundle.
Every surface node must retain same-index range/children/recovery parity in
the typed arena.

The leaf precedes B2/B1/A and lower selectors. It assigns distinct typed
ownership to base theorem/conclusion, witness take/item, Task-252 term 2,
and the two Task-256 formulas, then publishes the exact paired
`1/2/2/2/2` + one-row handoff. Source ordinals must partition `[0,1,2]`.
Raw syntax and resolver objects never enter checker rows.

Five compound tests freeze exact output; every lower/base/witness/
fingerprint/arena mutation and replay; source/hash/subtree near misses
including named/multiple/missing/extra witnesses, `take y`, reordered
statements, and composite roots; both-order family/active-route isolation;
and typed/final clone/debug with empty facts, obligations, checked formulas,
statement semantics, proofs, goals, and diagnostics.

The source's equality goal means it is not a valid semantic `take` proof.
The route stays corpus-dormant and cannot claim formula-statement coverage.
This documentation prerequisite leaves the existing 2,120/678/2,491/2,884
line statement/facade/runner/test modules, 379-test list, 30-path /
36,479-line manifest, and every hash unchanged.

Tasks 258B3N/M retain named, multiple, and other witness-term consumers
after B3 and before B4; Tasks 269–272 retain their semantic effects.

The B3 consumer is implemented as the exact private dormant selector and
five-test matrix. The runner now has 384 library tests; production remains
30 paths / 37,172 lines with path/content hashes
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`adfc81c21e69a91b194161525856aa40eb0e3ea76facfc2146dcb00b473ab3c2`.
No corpus artifact or active route changed.

## Checker Task 258B3N Dormant Harness Contract

The future private selector matches only the exact 107-byte
`FormulaStatementNamedWitnessSmoke` source and runs before B3/B2/B1/A and
lower formula routes. It must build the measured 51-node shared arena,
Task-48/252/256/base, one named witness and one name row, then install only
the authenticated pair. Five compound tests cover real identity, exhaustive
mutation/replay, byte/subtree near misses, route/ownership isolation, and
final empty semantics. No active corpus or external detail key is added.

## Checker Task 258B3N Dormant Harness Result

The private exact selector and five-test matrix are implemented. B3N runs
before B3/B2/B1/A, accepts only the frozen bytes and 51-node identity, and
publishes syntax-only witness/name tables. Exhaustive lower/base/name,
resolver, all-node, near-miss, cross-family, active-corpus, replay, rollback,
and final-empty-semantic checks pass. The library contains 389 tests;
production stays 30 paths / 37,555 lines.

## Checker Task 258B3M1 Planned Dormant Harness Contract

The future private selector will match only the final-LF 113-byte
`FormulaStatementMultipleWitnessSmoke` source and complete 56-node arena,
then dispatch ahead of B3N/B3/B2/B1/A. It will reconstruct the existing
lower/base handoffs and publish exactly named witness 0, unnamed witness 1,
and name row 0 without binding or semantic effects. Both witness rows will
share the one `take` source ordinal and retain dense ordinals 0/1.

Five future compound tests will cover source/hash/parser/resolver/lower
identity, exhaustive mutation/replay, all byte/subtree near misses,
active/cross-route isolation in both orders, and typed/final empty
semantics. No fixture, sidecar, trace row, external detail key, or active
dispatch will be added.

## Checker Task 258B3M1 Dormant Harness Result

The private selector and five compound tests are implemented. The route
authenticates the exact raw 56-node parser tree, theorem-only resolver
provenance, Task-48/252/256/base dependencies, and the dense `2/1`
witness/name transaction before publishing equal typed/final handoffs.
Mutation, near-miss, cross-family, active-route, replay, rollback, and
empty-semantic checks pass. Private stale fingerprints remain checker-test
owned; the runner proves their public equality and copied cross-profile
rejection without adding a mutation API.

The runner library now has 394 tests. Production remains 30 paths / 38,103
lines with statement leaf/facade/root/test sizes `3724/688/2501/7246`.
No fixture, sidecar, trace row/status/count, external detail key, active
route, or semantic result changed.

## Checker Task 258B3M2A Planned Dormant Harness Contract

The private statement selector will recognize only the final-LF 107-byte
numeral-witness source with SHA-256
`7b424949e98761b0179758065db5d164ad7d0a640f082801986683a54c43a2d1`.
Before dispatch it authenticates all 49 unrecovered parser nodes, exact
theorem-only resolver provenance, zero frontend diagnostics, and the
Task-48/252/256/base profiles. It then publishes one unnamed witness row
targeting Task-252 primary numeral term 2 and numeric request 0, with no
name, binding, atomic edge, active route, or semantic output.

The selector precedes B3M1/B3N/B3/B2/B1/A and fails closed for every
byte, node, subtree, resolver, lower-table, numeral, numeric-request,
recovery, statement-shape, or cross-family near miss. The exact five tests
are:

1. `task258b3m2a_real_frontend_freezes_numeral_witness_contract`;
2. `task258b3m2a_validation_precedence_mutation_and_replay_fail_closed`;
3. `task258b3m2a_selector_and_byte_subtree_near_misses_are_exact`;
4. `task258b3m2a_family_and_active_route_isolation_is_atomic_in_both_orders`;
5. `task258b3m2a_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`.

This documentation task adds no fixture, sidecar, trace row/status/count,
external detail key, active dispatch, or public mutation API. The runner
baseline remains 394 tests and 30 production paths / 38,103 lines;
implementation projects 399 tests.

## Checker Task 258B3M2A Dormant Harness Result

The private selector and five compound tests now implement the frozen
contract. The route authenticates the exact raw 49-node parser tree,
theorem-only resolver provenance, Task-48/252/256/base dependencies, dense
reference/numeric-request partition, and `1 witness / 0 names` before
publishing equal typed/final handoffs. Precedence, mutation/replay, every
byte/node/subtree near miss, family and active-route isolation in both
orders, rollback, debug compatibility, `Some(Vec::new())`, and empty
semantics pass.

The runner has 399 tests and 30 production paths / 38,571 lines, with
statement leaf/facade/root/test sizes `4185/691/2505/8611`. No fixture,
sidecar, trace row/status/count, external key, public mutation API, active
route, or semantic result changed.

## Task 258B3M2B1 Dormant Consumer Contract

The private runner will select only the final-LF 113-byte/hash
`FormulaStatementParenthesizedWitnessSmoke` source and authenticate all 53
nodes/root 52 plus theorem-owner provenance. It lowers five roots to six
primary rows: wrapper term 2 contains child variable term 3, while refs
`0..4` target `0/1/3/4/5`. Atomic equalities use only `[0,1]` and `[4,5]`;
one unnamed witness targets term 2 and adds no name or binding.

Exactly five compound tests are frozen:

1. `task258b3m2b1_real_frontend_freezes_parenthesized_witness_contract`;
2. `task258b3m2b1_validation_precedence_mutation_and_replay_fail_closed`;
3. `task258b3m2b1_selector_and_byte_subtree_near_misses_are_exact`;
4. `task258b3m2b1_family_and_active_route_isolation_is_atomic_in_both_orders`;
5. `task258b3m2b1_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`.

Test 2 independently mutates a new wrapper-term-2 reference while retaining
the child reference, removes/remaps/duplicates child reference 2, and
contaminates a Task-256 edge/request first with term 2 and then with term 3.
Test 3 confirms selector/subtree near misses cannot publish either partial
wrapper/child ownership or a detached child reference.

Near misses include unparenthesized/nested/numeral/other-child, named or
multiple, application/structure/selector/update/set/choice, recovered,
changed theorem, and composite/existential shapes. Authority-invalid
theorem-proof `take it;` is also a near miss. The exact output keeps
`Some(Vec::new())`, lookups `1/1`, reference uses `[1; 5]`, and empty
semantics. The prerequisite changes no route, key, source/test artifact,
count, or hash.

## Task 258B3M2B1 Dormant Consumer Implementation

The exact selector is implemented before prior statement profiles and
accepts only the final-LF 113-byte source, all 53 unrecovered nodes/root 52,
and zero frontend diagnostics. Five roots become six primaries; wrapper
term 2 owns child/reference term 3, equalities use only `[0,1]` and
`[4,5]`, and the one unnamed witness targets outer term 2. The result keeps
`Some(Vec::new())`, lookups `1/1`, uses `[1; 5]`, and empty semantics.

Five tests cover exact identity, independent lower mutations,
selector/subtree near misses, prior statement plus Tasks 253–255 and active
isolation in both orders, rollback/replay, and typed/final clone/debug.
Malformed lower rows that cannot form Task-252/256 handoffs reject at their
owning public producer; constructible lower handoffs reach the paired
consumer. No public route/key/fixture/sidecar/trace/active/binding/semantic
owner changed.

### Checker Task 258B3M2B2A Dormant Nested-Parentheses Contract

The future dormant selector accepts only the final-LF 121-byte/hash
`FormulaStatementNestedParenthesizedWitnessSmoke` source and authenticates
zero diagnostics plus 57 nodes/root 56. Five extraction roots expand to
seven Task-252 primaries: outer wrapper 2 parents inner wrapper 3, which
parents reserved-variable term 4. References target `0/1/4/5/6`;
equalities target only `[0,1]` / `[5,6]`; one unnamed witness targets
outer term 2. The complete `2/3/4` subtree remains outside Task-256.

Five future tests cover exact frontend/resolver/lower identity, both
parent links, reference and independent subtree contamination, selector
and byte near misses, prior statement plus Tasks 253–255 and active
isolation in both orders, replay/rollback, and typed/final clone/debug.
Malformed Task-252/256 inputs retain lower-producer-first fail-close.
Successful transport returns `Some(Vec::new())`, lookups `1/1`, uses
`[1; 5]`, and no binding or semantic output. No active corpus, public
route/key, fixture, sidecar, expectation, or trace metadata changes.

## Checker Task 258B3M2B2A Dormant Consumer Implementation

The exact selector is implemented before prior statement profiles and
accepts only the 121-byte source, all 57 unrecovered nodes/root 56, and zero
frontend diagnostics. Five roots expand to seven primaries with the complete
`2 -> 3 -> 4` chain; references target `0/1/4/5/6`, equalities only
`[0,1]` / `[5,6]`, and the unnamed witness targets outer term 2.

All five frozen tests pass. Successful transport remains
`Some(Vec::new())`, lookups `1/1`, uses `[1; 5]`, and empty binding and
semantic output. Invalid Task-252/256 rows reject at their lower producer
when no handoff can form; constructible corruptions reach the paired
statement consumer. No active corpus, public route/key, fixture, sidecar,
expectation, or trace metadata changed.

## Checker Task 258B3M2B2B1P Private Task-253 Seam Contract

The future private helper
`unwrapped_imported_source_application_handoff_in_context` receives the
surface AST, module, symbol/binding environments, shared Task-252 source
terms, application node, and explicit `BindingContextId`. It reuses the
existing unwrapped imported extraction and handoff builder. The legacy
helper delegates with context 0.

For the 143-byte `take 1 ++ 2;` probe, context 1 produces exactly one
symbolic infix application/no wrappers, imported
`parser.type_fixtures::++`, arguments `Primary(2)` then `Primary(3)`, and
two requests. Exactly two compound tests are frozen:

1. `task258b3m2b2b1p_proof_context_reuses_exact_unwrapped_imported_application`;
2. `task258b3m2b2b1p_context_provenance_and_legacy_replay_fail_closed`.

They reject missing/nonexistent or mixed contexts, wrong node/range/head/
form/argument order, wrappers, import/candidate/provenance substitution,
stale Task-252 fingerprints, and replay, while retaining byte-identical
context-0 outputs. B1P publishes no statement, typed/final statement
coexistence, diagnostic detail, semantic, proof, or goal output.

## Task 258B3M2B2B1P Harness Result

Both frozen compound tests are implemented and pass. The first fixes the
143-byte source SHA-256, Task-48 `2/1/0`, Task-252 `6/4/2`, Task-253
`1/0/1/2/2`, ordered targets/provenance, empty downstream tables, and
legacy context-0 debug SHA-256
`9f1449159bf362bc90c4b41f3e4befb9a6d54f4152b836063f5cc07083d82a8d`.
The second rejects every frozen context/root, wrapper/shape, range, form,
target, candidate/contribution, ambiguous provenance, and stale replay
case, then proves clean replay. No fixture, expectation, sidecar, trace
row, active case, or public diagnostic detail was added.

## Checker Task 258B3M2B2B1A Dormant Harness Contract

The private selector recognizes only:

```mizar
import parser.type_fixtures;
reserve x for set;
theorem FormulaStatementApplicationWitnessSmoke: x = x proof
  take 1 ++ 2;
  thus x = x;
end;
```

The final LF is part of the 143-byte identity; SHA-256 is
`22ce235030bc56720bfe7f52830182144ca6e4eee4414b7f8c2823e3d0f82c1b`.
Before publication it authenticates zero diagnostics, all 63 nodes/root 62,
the theorem node/range/path `59/48..142/[2,1]`, proof context 1 at
`103..141`, imported `++` contribution/path `2/[12]`, Task-252 `6/4/2`,
Task-253 `1/0/1/2/2`, Task-256 equalities over `[0,1]` and `[4,5]`, base
statement `1/2/2/2/2`, and witness `1/0`.

The extractor owns take/witness nodes 49/48, authenticates and traverses
unowned transparent node 47, and targets Task-253 application node/row
`46/0`. Node 47 is not a wrapper or primary. Task-252 owns numeral nodes
44/45 as terms 2/3; Task-253
owns the infix application/head/candidate/arguments/requests; Task-256 owns
only the theorem/conclusion equality terms. The existing context-aware B1P
helper is the sole application producer.

The exact five tests are:

1. `task258b3m2b2b1a_real_frontend_freezes_application_witness_contract`;
2. `task258b3m2b2b1a_validation_precedence_mutation_and_replay_fail_closed`;
3. `task258b3m2b2b1a_selector_and_byte_subtree_near_misses_are_exact`;
4. `task258b3m2b2b1a_family_and_active_route_isolation_is_atomic`;
5. `task258b3m2b2b1a_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`.

They freeze every node and lower fingerprint; target/context/range/form,
candidate/contribution/provenance, numeric request, equality-subtree, base
statement, and witness precedence; application/statement installation in
both directions; prior Task-258 profiles and Tasks 253-255; every byte
mutation; operand/operator/name/cardinality/parenthesis/theorem/import/
recovery near misses; replay, rollback, and final clone. Malformed lower rows
reject at their owning producer. Constructible wrong bundles reach the
combined consumer and fail without partial publication.

Success returns `Some(Vec::new())`, statement lookups `1/1`, reference-use
ordinals `[1; 4]`, and equal typed/final handoffs. All semantic term types,
witness obligations, goal matching/substitution, formula truth, proofs,
terminal goals, Core/ControlFlow/VC, cluster facts, diagnostics, and active
outputs stay empty. The harness adds no fixture, sidecar, expectation, trace,
detail key, public route, or active case.

## Checker Task 258B3M2B2B1A Dormant Harness Implementation

The dormant statement route now composes the exact real frontend/resolver
outputs with the Task-48 binding environment and Task-252/253/256 public
handoffs, then calls the atomic checker application/statement/witness
installer. Its successful profile retains contexts/bindings/diagnostics
`2/1/0`, imported candidate/application/argument/request provenance, equality
edge pairs, lookups `1/1`, and `Some(Vec::new())` transport details. Every
one of the 143 loaded-source bytes and reparsed operator/name/import/recovery
near misses fails selector and route admission; dependency, provenance,
precedence, family-order, replay, rollback, and final-clone corruptions also
fail closed.

The five exact runner tests pass. Expression semantics, inferred types,
substitutions, obligations, proof steps, terminal goals, Core/ControlFlow/VC,
cluster facts, diagnostics, and active outputs remain empty. No fixture,
sidecar, expectation, trace, detail key, public route, or active case was
added.

## Checker Task 258B3M2B2B1B1P Dormant Lower Harness Contract

The motivating 158-byte source contains only one new lower shape:
`ParenthesizedTerm 129..137 -> InfixExpression 130..136`. The private
B1B1P harness composes shared Task-252 `6/4/2` with Task-253
`1/1/1/2/2` in proof context 1, authenticating the same imported `++`
candidate and wrapper/application containment. It stops before building a
Task-258 witness, statement, semantic term, proof step, substitution, or
goal.

The two future compound tests are
`task258b3m2b2b1b1p_wrapped_imported_application_proof_context_reuse_is_exact`
and
`task258b3m2b2b1b1p_wrapper_corruption_replay_and_legacy_outputs_fail_closed`.
Together they mutate all 158 loaded-source byte positions including final
LF, all 67 nodes' kind/range/recovery/ordered children and root identity,
and parsed operator/name/import/parenthesis/recovery near misses. The
public/active Task-253 route remains unselected throughout.

Success asserts every application/wrapper/candidate/argument/request field,
the complete imported symbol and origin identity, typed/final clone parity,
and empty semantic/proof/goal/diagnostic tables. Failure precedence is
selector before Task-252 before Task-253 before stale-fingerprint typed
installation; combined corruption proves that order and every failure proves
atomic absent/unchanged typed and resolved publication before clean replay.
The pre-change unwrapped context-0 and context-1 row hashes are respectively
`9f1449159bf362bc90c4b41f3e4befb9a6d54f4152b836063f5cc07083d82a8d`
and
`0fd83f61a40d3fd43816a52b70fca4fa4cf7f1d6e9172d3c5fe558c5d4add80d`;
their separate `2/0/2` with `Primary(0/1)` and `6/4/2` with
`Primary(2/3)` rows remain exact. No active case, fixture, sidecar,
expectation, trace row, detail key, or public dispatch is added.

## Checker Task 258B3M2B2B1B1P Dormant Lower Harness Implementation

The private harness now selects the exact 158-byte/67-node wrapped
application and reuses Task-252 `6/4/2` plus Task-253 `1/1/1/2/2` in proof
context 1. It authenticates every frozen candidate and origin-provenance
field, including contribution 2 and structural path `[12]`, while leaving
the legacy unwrapped paths byte-compatible.

The two exact tests pass. They freeze every loaded-source byte and AST field,
five same-source resolver substitutions, empty source families and all
semantic/proof/goal/diagnostic tables, atomic failure and clean replay, plus
the exact reparsed `(diagnostics,nodes)` matrix
`[(0,63),(0,71),(0,67),(12,72),(1,64),(0,72),(0,67),(14,73)]`.
No statement consumer, active case, fixture, sidecar, expectation, trace
row, detail key, public dispatch, or downstream semantic owner was added.

## Checker Task 258B3M2B2B1B1 Dormant Harness Contract

The B1B1 harness selects only the frozen 158-byte/67-node
`take (1 ++ 2);` source. It reuses the B1B1P wrapped Task-253 seam, then
publishes base `1/2/2/2/2` and one unnamed witness targeting
`Application(0)` through the existing atomic checker path. Wrapper 0 remains
Task-253 containment and is not the witness target.

The exact successful witness is owner/context/source/witness ordinal
`0/1/1/0`, take `53/124..138`, item `52/129..137`, normalized spelling
`( 1 ++ 2 )`, normal/unnamed/no name. The exact lower handoff is application
`48/130..136`, wrapper `50/129..137`, head `20/132..134/++`, ordered
`Primary(2/3)`, and the imported `parser.type_fixtures::++#12` candidate.
The theorem owner is contribution 0 with `LocalSource` anchor `29..47`,
origin `48..157/[2,1]`, and label `56..108`.

The five named runner tests exhaust all source bytes and all arena fields,
the five same-source resolver substitutions, the exact reparse matrix,
selector/lower/aggregate/witness/typed/final precedence, B1A compatibility,
all family and active-route isolation, atomic rollback and clean replay,
final clone equality, and empty semantic/proof/goal/overload output. Exact
success still returns no detail keys. No fixture, sidecar, expectation,
trace row, active case, or semantic consumer is authorized.

## Checker Task 258B3M2B2B1B1 Dormant Harness Result

The exact B1B1 selector, resolver substitutions, full byte/node mutation
matrix, family/active isolation, atomic replay/rollback, clone, and empty
upper-table assertions are implemented in five passing runner tests. The
runner library now has 423 tests and the statement-test leaf is 13,381 lines.
No fixture, expectation, sidecar, trace row, active detail key, or semantic
consumer changed.

## Checker Task 258B3M2B2B2P Dormant Lower Harness Contract

The B2P harness freezes only the exact final-LF 172-byte/76-node source with
`take TypeCaseStruct(x: 1, y: 2);`. It stops after composing an existing
proof binding context, shared Task-252 `6/4/2`, and Task-254
`1/0/1/2/0/2/6`; it does not construct a Task-258 statement or witness.

Successful lower ownership is exact. Constructor node 59 alone owns
`source.term.structure.constructor`; member token nodes 20/24 alone own
`source.term.structure.member.constructor-assignment`. Qualified root 52 is
authenticated imported resolver traversal but remains
`source.surface.unowned`. Task 252 uses nodes 54/57 only as private
extraction roots and publishes numeral rows at sites 53/56, so 53/56 are
`source.term.numeral` while 54/57 remain `source.surface.unowned`. The
constructor has proof context 1, members `x/y`, ordered
`ConstructorValue` edges to `Primary(2/3)`, six ordered unresolved requests,
and no application fingerprint.

The imported root is exactly
`summary:parser.type_fixtures#parse-only#TypeCaseStruct:5` /
`parser.type_fixtures::TypeCaseStruct#5`, contribution 2, origin
`7..27/[5]`, public/exported, with no signature. The runner-private seam
must reuse the public Task-254 producer and the already built binding/source
term parts; it may not duplicate Task-252 rows, synthesize field identity, or
broaden the existing Task-254 real route.

The two future compound tests are
`task258b3m2b2b2p_structure_constructor_proof_context_reuse_is_exact` and
`task258b3m2b2b2p_structure_constructor_corruption_replay_and_legacy_output_fail_closed`.
They cover all 172 bytes including final LF; all 76 nodes' kind, range,
recovery, ordered children, and root identity; reparsed
import/root/member/value/recovery near misses; exact rows, owned kinds, and
imported provenance; context/root/member/edge/request substitutions; lower
validation precedence; stale failure followed by clean replay; and
byte-identical legacy Task-254 output. All upper source families and
semantic/proof/goal/IR tables remain empty.

No active case, public dispatch, statement consumer, fixture, sidecar,
expectation, trace row, detail key, or checker test is added. The future B2A
contract alone may attach a witness to `Structure(0)`; selector and
functional-update/`FieldUpdate` witnesses remain B2B and B2C.

## Checker Task 258B3M2B2B2P Dormant Harness Result

The exact-source selector, owned-kind map, shared Task-252 parts, existing
proof context, imported provenance, Task-254 handoff, mutation matrix, stale
replay, and legacy hashes are implemented in two passing runner tests.
All upper source/semantic/proof/goal/IR tables remain empty. No active case,
fixture, sidecar, expectation, trace row, detail key, checker test, or
statement consumer changed.

The completed pair specifically pins Task-48/252/254
`2/1/0`/`6/4/2`/`1/0/1/2/0/2/6`, ownership 59/20/24, numerals 53/56,
unowned 52/54/57, exact `TypeCaseStruct#5` provenance, and malformed recovery
`diagnostics=1, nodes=74, root=73, recovered=[52]`. Current Task-254
source-structure/typed/final hashes remain
`0d6af57b89e6156d8e5de6831568c81ec110880bebf1e4aeb4ab00563f4da6c8`,
`8264d1574faf67e19b6b84d6e11fa7ab6435335238b398fa0966bbfbc63d0599`,
and
`118a998bc5edb770c7818be1d74cbece0f566353bf9d3e6aabb817d994a3db40`.

## Checker Task 258B3M2B2B2A Frozen Dormant Harness

The future private statement harness selects only the unchanged
final-LF 172-byte/76-node/root-75 zero-diagnostic constructor-witness source
with hash
`24e2ee2332ead5c0d46025df6044450eeab3ebb5733ebe83587ceae3ba129eb6`.
It first reuses B2P's exact owned-kind selector and proof-context Task-254
seam, then authenticates Task-48 `2/1/0`, Task-252 `6/4/2`, Task-254
`1/0/1/2/0/2/6`, equality-only Task-256
`2/0/0/0/0/0/0/4/4`, Task-258 base `1/2/2/2/2`, and one unnamed
`Structure(0)` witness/no names.

The Task-258 base transaction owns theorem/conclusion statement rows 72/70.
The B2A extension owns take/witness nodes 62/61 and the
witness-to-structure edge only. Constructor/member nodes 59/20/24 stay
Task 254, term/numeral sites 45/47/53/56/63/65 stay Task 252, equality nodes
49/67 stay Task 256, and root/extraction/transparent/container nodes remain
unowned. Current theorem provenance is local anchor `29..47`, checked owner
`48..171/[2,1]`, owner
and contribution 0, public/exported/normal label, and no import edge or
recovery. Imported `parser.type_fixtures::TypeCaseStruct#5` provenance is
contribution 2, origin `7..27/[5]`, public/exported, signature-free, and
normal. Both are exact selectors. Task 256 has no direct structure edge or
fingerprint and is structure-aware revalidated only at the combined
typed/final boundary.

The runner passes authenticated syntax-free lower handoffs into the new
checker-owned full structure-aware builder and full atomic
structure/statement/witness installer frozen in the canonical checker plan.
It never exports parser or resolver values or duplicates B2P extraction.
Lower installation
precedes aggregate/base-row validation, which precedes witness publication,
typed publication, and final clone. Failure leaves no partial state and a
fresh replay succeeds.

The five frozen tests are:

1. `task258b3m2b2b2a_real_frontend_freezes_structure_constructor_witness_contract`;
2. `task258b3m2b2b2a_validation_precedence_mutation_and_replay_fail_closed`;
3. `task258b3m2b2b2a_selector_and_byte_subtree_near_misses_are_exact`;
4. `task258b3m2b2b2a_family_and_active_route_isolation_is_atomic`;
5. `task258b3m2b2b2a_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`.

They mutate all 172 bytes and all fields of 76 nodes, exercise exact local
and imported resolver substitutions, lower/base/witness rows and
fingerprints, dependency order and family hybrids, all ownership/active
orders, rollback/replay/final clone, legacy/application compatibility, and
malformed recovery `1/74/root 73/[52]`. Semantic, proof, goal, overload,
Core, CFG, and VC outputs remain empty. No active case, fixture, expectation,
sidecar, trace row/credit, diagnostic detail, or public runner route changes.

Documentation baselines remain checker/runner tests `378/425`; runner
statement/structure/facade/root/statement-test/structure-test sizes are
`5962/2857/715/2531/13381/2991`. Implementation projects 430 runner tests.

## Checker Task 258B3M2B2B2A Dormant Harness Result

The private harness now recognizes only the frozen constructor-witness
source and composes Task-48/252/254/256/base with one `Structure(0)`
witness. It reuses the B2P ownership/provenance selector rather than copying
parser or resolver tables. The five exact named tests pass, including all
172 bytes, all 76 node/root fields through the B2P seam, malformed recovery,
dependency/base/witness mutation and replay, family isolation, and typed/
final clone/empty semantics.

The runner library is 430 tests. No active case, fixture, expectation,
sidecar, trace row/backlink/credit, diagnostic detail, or public runner
route was added. B2B/B2C and all semantic/proof/goal/overload/Core/CFG/VC
families remain deferred or empty.

## Checker Task 258B3M2B2B2BP Frozen Private Selector Harness

This harness freezes only Task-254 proof-context lower reuse for the exact
171-byte/79-node direct-selector source. It authenticates Task-48 `2/1/0`,
Task-252 `6/4/2`, and Task-254 `2/0/1/3/0/3/9`, including imported
`TypeCaseStruct#5`, owned nodes `62/61/29/20/24`, and the chain
`Structure(0) -> Structure(1) -> Primary(2/3)`. It publishes no Task-256 or
Task-258 output.

The two frozen tests are
`task258b3m2b2b2bp_structure_selector_proof_context_reuse_is_exact` and
`task258b3m2b2b2bp_structure_selector_corruption_replay_and_constructor_compatibility_fail_closed`.
They cover all source bytes/node fields, every lower row/fingerprint,
provenance and owned-kind map, context/range/source/member/edge/request
corruption, and the exact 170-byte missing-selector-name near miss
(`malformed_term_expression` at `149..150`, 78 nodes/root 77, recovered
`[]`). Valid but excluded selector/call/chain/wrapped/base/update forms,
precedence, rollback/replay, constructor compatibility, and all empty upper
tables are also covered.

No active case, public route, fixture, expectation, sidecar, trace credit,
diagnostic detail, checker test, or semantic behavior is added.

## Checker Task 258B3M2B2B2BP Private Selector Harness Result

The frozen private harness is implemented and both named tests pass. The
valid path publishes the exact Task-254 `2/0/1/3/0/3/9` bytes only after
authenticating Task-48/252, all 79 surface nodes, imported root provenance,
owned nodes `62/61/29/20/24`, every lower row, and the current Task-252
fingerprint. Every mutation fails closed and a clean replay reproduces the
handoff, TypedAst, and ResolvedTypedAst debug bytes.

The missing-selector near miss is directly authenticated as sole syntax
diagnostic `malformed_term_expression` at `149..150`. B2P/B2A and legacy
Task-254 compatibility hashes remain exact. The runner library is `432`;
active cases, fixtures, sidecars, expectations, trace credit, diagnostics,
and semantic outputs remain unchanged.

## Checker Task 258B3M2B2B2B Frozen Runner Harness

The harness uses the exact 171-byte final-LF source whose selector witness is
`TypeCaseStruct(x: 1, y: 2).x`. It authenticates 79 nodes/root `78`, Task-48
`2/1/0`, Task-252 `6/4/2`, Task-254 `2/0/1/3/0/3/9`, Task-256
`2/0/0/0/0/0/0/4/4`, Task-258 base `1/2/2/2/2`, and witness `1/0`.
Replacing `.x` with `.` yields the exact 170-byte near miss with sole
`malformed_term_expression` at `149..150`, 78 nodes/root `77`, and
`recovered = []`.

The five required runner tests are:

- `task258b3m2b2b2b_real_frontend_freezes_structure_selector_witness_contract`
- `task258b3m2b2b2b_validation_precedence_mutation_and_replay_fail_closed`
- `task258b3m2b2b2b_selector_and_byte_subtree_near_misses_are_exact`
- `task258b3m2b2b2b_family_and_active_route_isolation_is_atomic`
- `task258b3m2b2b2b_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`

They must cover every source byte/node field, provenance and ownership row,
subtree exclusions, all lower fingerprints, validation precedence, clean
replay, B2A/B2B hybrids and family orders, active-route isolation, debug
stability, atomic rollback, final clone, and empty semantic tables. Existing
fixtures, expectations, sidecars, trace metadata, active cases, diagnostic
credit, and CLI behavior remain unchanged.

## Checker Task 258B3M2B2B2B Dormant Harness Result

The private harness now recognizes only the frozen 171-byte/79-node
selector-witness source. It consumes the existing B2BP owned-kind and
proof-context handoff seams, composes the exact Task-48/252/254/256/258
tables, and installs one unnamed witness targeting selector
`Structure(0)`. No lower parser/resolver row is copied or relaxed.

All five exact runner tests pass. Their matrices cover all source bytes and
node fields, complete local/imported provenance and ownership, lower/base/
witness corruption and validation precedence, the exact
`malformed_term_expression` near miss, valid excluded selector forms,
B2A/B2B and active-family isolation, rollback/replay, final clone, and empty
semantic/proof/goal/overload/Core/CFG/VC outputs.

The runner library is `437`. No public or active route, fixture,
expectation, sidecar, trace row/backlink/credit, diagnostic credit, or
semantic behavior was added.

B2B closed as implementation commit `8311502c`; its clean fresh inventory
selects B2CP before B2C.

## Checker Task 258B3M2B2B2CP Frozen Private Harness

The dormant harness is frozen only for the final-LF 181-byte,
SHA-256
`03f14a98bffb557ea4dda4f879bf504d241aaebae0552a97f0f2417ef4b43560`,
86-node/root-85 `FormulaStatementStructureUpdateWitnessSmoke` source:

```text
import parser.type_fixtures;
reserve x for set;
theorem FormulaStatementStructureUpdateWitnessSmoke: x = x proof
  take TypeCaseStruct(x: 1, y: 2) with (x := 3);
  thus x = x;
end;
```

It must reuse proof context 1 at `107..179` and exact Task-48 `2/1/0`,
Task-252 `7/4/3`, and Task-254 `2/0/1/3/1/4/9`. The Task-252 extraction
roots are `51/53/60/63/67/73/75` and published sites are
`51/53/59/62/66/73/75`. Task 254 owns update/constructor/member/
`FieldUpdate` nodes `69/65/30/20/24/68`, authenticates imported
`TypeCaseStruct#5` contribution 2 at `7..27/[5]`, and preserves the exact
update-base, update-value, constructor-value, and nine-request ordering.

Task 256 owns only `BuiltinPredicateApplication` nodes `55/77`; formula
containers `56/78` and the complete update subtree are excluded. The
harness owns no Task-256/258, statement, witness, checker/public API, active
route, diagnostic, or semantic output.

The two exact future tests are:

- `task258b3m2b2b2cp_structure_update_proof_context_reuse_is_exact`
- `task258b3m2b2b2cp_structure_update_corruption_replay_and_prior_sibling_compatibility_fail_closed`

They cover every source byte/node field; imported root, lower rows,
update-path and `FieldUpdate` ownership; edge/request order; all corruption
and precedence classes; and stale/clean replay. Replacing the complete
`with (x := 3)` fragment with `with (x := )` freezes the exact 180-byte
SHA-256
`8310de3b172cea98e4e85ebc6021c85c4e1bd7c2a74f8cd99413ae5a80569d67`
near miss with one `malformed_term_expression` at `158..159`, 84 nodes/root
83, and `recovered = [65]`.

Valid excluded base-only, selector, wrapped, multi-update, and nested-path
forms remain outside the seam. Both B2P constructor and B2BP selector
compatibility are exact. No checker test, statement consumer, active case,
fixture, sidecar, expectation, trace row, detail key, or semantic behavior
is added. Functional-copy semantics, update result typing/identity, witness
obligations, theorem/proof acceptance, goals, and IR are deferred. In
particular, the `take` under the `x = x` goal is not a semantic acceptance
claim.

## Checker Task 258B3M2B2B2CP Private Harness Implementation

CPC1 commit `ee267d9c` is complete. The four frozen runner files now
implement only the private, corpus-dormant update reuse seam, and exactly
the two frozen B2CP tests pass. Direct table comparisons authenticate all
Task-48/252/254 rows, replay/corruption, and B2P/B2BP compatibility. This
closes the prerequisite `design_drift`, bounded `source_drift`, and
`test_gap`. Final test-sufficiency and implementation re-reviews have no
findings.

Checker/runner libraries are `386/439`; runner sizes are
`6826/6065/730/2546/17120/5848`. Production is 30 paths / 46,788 lines with
hashes
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`bbcc55ab769fb5b725de83a27ae13243000a1610a12064907c06187417e45b5f`;
test-list hashes are
`ea3e854c1b741ab4b642000df6610a15e521f0849b39e7480820ca86680a1d0e` /
`11e6de35b422b913c235d8193fb2629da5aff39d1cf251af1c6cec2824301c8d`.
Checker/corpus/CLI hashes remain unchanged.

There is no fixture, sidecar, expectation, trace
status/count/backlink/credit, public/active route, or semantic change.
Formula credit remains `deferred`, `tests = []`; audit impact is
narrative-only. B2C and all functional-copy/type/proof/goal/IR deferrals
remain unchanged. Concurrent ownership remains report-only
`repo_metadata_conflict` with no metadata repair. Broad formatting, Clippy,
tests, and all count/hash gates pass. The final source/documentation
re-review has no findings. Independent final quality has no findings, all
nine hard gates PASS, and valid `98/100`. B2CP implementation commit
`b146f0f72dceac2233c9d679b7820e264974b227` and clean B2C fresh inventory
are complete.

## Checker Task 258B3M2B2B2C Frozen Runner Harness

B2CP implementation commit
`b146f0f72dceac2233c9d679b7820e264974b227` is complete. B2C uses the
exact final-LF 181-byte SHA-256
`03f14a98bffb557ea4dda4f879bf504d241aaebae0552a97f0f2417ef4b43560`
source:

```text
import parser.type_fixtures;
reserve x for set;
theorem FormulaStatementStructureUpdateWitnessSmoke: x = x proof
  take TypeCaseStruct(x: 1, y: 2) with (x := 3);
  thus x = x;
end;
```

It has zero diagnostics, 86 unrecovered nodes, root 85.
Theorem/label/proof/take/witness/transparent/update/constructor/root are
`82/11/81/72/71/70/69/65/58`; `FieldUpdate`/update member/constructor
members are `68/30/20/24`; numerals are `59/62/66`; conclusion/equalities/
containers are `80/55,77/56,78`. Their exact ranges are
`48..180`, `56..99`, `107..179`, `115..161`, `120..160`,
`120..160`, `120..160`, `120..146`, `120..134`, `153..159`,
`153..154`, `135..136`/`141..142`,
`138..139`/`144..145`/`158..159`, `164..175`,
`101..106`/`169..174`, and the same two formula ranges.

The missing-value source is exactly 180 bytes, SHA-256
`8310de3b172cea98e4e85ebc6021c85c4e1bd7c2a74f8cd99413ae5a80569d67`,
with one `malformed_term_expression` at `158..159`, 84 nodes/root 83, and
`recovered = [65]`. Five zero-diagnostic excluded profiles remain outside
the exact route:

- base-only: 167 bytes, SHA-256
  `bb26a425d2bc16e6518d6366128de138862c4525af6eb82b748e4cb28f1b8bc9`,
  `76/root75/[]`;
- selector: 169 bytes, SHA-256
  `64039fca35d6199fea281d43df6dafdfeff78f1d97139d6286a3082115552747`,
  `79/root78/[]`;
- wrapped: 183 bytes, SHA-256
  `e1a2b79cb03a4aebc5e0e29150cde382da457aa31cb8e66643eecce6e8296ae6`,
  `90/root89/[]`;
- multi-update: 189 bytes, SHA-256
  `a95336dc08b9534d7c5c16ca5070384e2610f0db31841187878b68b4403666b6`,
  `93/root92/[]`;
- nested-path: 183 bytes, SHA-256
  `92440b4b3814d7b8a738bf71b2e89b9056fbb382301e12b5f4a4ccab17e0f082`,
  `88/root87/[]`.

The harness compares Task 48 `2/1/0`, Task 252 `7/4/3`, Task 254
`2/0/1/3/1/4/9`, Task 256 `2/0/0/0/0/0/0/4/4`, Task-258 base
`1/2/2/2/2`, and witness `1/0`. Task-258 base has two
`ReservedTypeGuard` rows with reference uses `[0,1]`/`[2,3]`; the latter
resolve to primary terms `5/6`. It verifies LocalSource contribution 0
anchor `29..47`, owner origin
`48..180/[2,1]`, public/exported label `56..99`, statements
`82/48..180/Atomic(0)/ordinal 0` and
`80/164..175/Atomic(1)/ordinal 2`, and two unverified candidates. The
unnamed witness at take 72/item 71 uses proof context 1 and targets only
update `Structure(0)`.

The runner consumes existing B2CP `ImportedStructureUpdateSite`,
`imported_structure_update_owned_node_kinds`, and
`imported_structure_update_handoff_in_context` unchanged. It verifies
update edges to `Structure(1)`/`Primary(4)`, constructor edges to
`Primary(2/3)`, imported `TypeCaseStruct#5` contribution 2 origin
`7..27/[5]`, Task-256 operands `Primary(0/1)`/`Primary(5/6)`, local
theorem/label provenance, fingerprints, and disjoint ownership.

Exactly five runner tests are frozen:

- `task258b3m2b2b2c_real_frontend_freezes_structure_update_witness_contract`
- `task258b3m2b2b2c_validation_precedence_mutation_and_replay_fail_closed`
- `task258b3m2b2b2c_update_and_byte_subtree_near_misses_are_exact`
- `task258b3m2b2b2c_family_and_active_route_isolation_is_atomic`
- `task258b3m2b2b2c_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`

The paired checker transaction freezes exactly four tests:

- `task258b3m2b2b2c_exact_structure_update_witness_api_debug_and_legacy_compatibility_are_stable`
- `task258b3m2b2b2c_dependencies_structure_update_witness_precedence_and_all_nodes_fail_closed`
- `task258b3m2b2b2c_combined_ownership_hybrids_and_all_family_orders_are_atomic`
- `task258b3m2b2b2c_final_clone_revalidation_and_semantic_deferrals_are_stable`

Together, the exact nine tests exhaust bytes, nodes, lower
and upper rows, provenance, ownership, malformed/excluded profiles,
B2A/B2B/B2CP/legacy isolation, mutations, order, fingerprints, hybrids,
rollback/replay/final clone, and empty semantic/proof/goal/IR surfaces.

No active case, fixture, expectation, sidecar, trace entry/credit,
diagnostic contract, public route, or semantic behavior changes. Section
13.3.3 and complete postfix grammar authorize the source without a
normative `spec_gap`; `take` under `x = x` is source transport only.
Baseline is `386/439`, projection `390/444`; sizes and all
production/test-list/corpus/CLI hashes remain unchanged. All four independent
reviews have no findings and complete documentation/count/hash verification
passes. Independent final quality has no findings, all nine hard gates PASS,
and the valid score is `98/100`. The commit and implementation inventory
remain open.

## Checker Task 258B3M2B2B2C Implemented Runner Harness

The harness now recognizes only the frozen 181-byte/86-node functional-update
source, reuses the B2CP update extractor/producer boundary, and assembles the
exact lower tables before the Task-258 statement and witness producers. The
witness targets `Structure(0)`; the constructor, update value, equality
operands, resolver provenance, and ownership exclusions remain with their
existing producers.

The five frozen runner tests pass: real frontend, validation precedence and
replay, malformed/valid-excluded byte and subtree near misses, family/active
isolation, and typed/final/debug/rollback/empty semantics. The paired four
checker tests pass as well. Final test-sufficiency and implementation reviews
have no findings. Runner library `444` and its policy suites pass; broad
workspace and remaining final reviews are pending.

No active fixture, expectation, sidecar, trace row/credit, diagnostic,
semantic, proof, goal, or IR surface changed. The formula-statement trace row
remains `deferred`, `tests = []`.

## Checker Task 258B3M2B2B2C Broad Harness Verification

Format, workspace Clippy, checker and runner crate/policy suites, full
workspace tests, focused `4/4` and `5/5`, and sibling `12/12` and `21/21`
suites pass. Fresh counts and hashes match the paired plans. No active,
fixture, trace-credit, diagnostic, or semantic harness surface changes;
independent final consistency/quality, commit, and post-commit gates remain
pending.

## Checker Task 258B3M2B2B2C Final Harness Review Status

Independent final source/documentation consistency and final quality both
report **NO FINDINGS**. All nine hard gates PASS and the valid score is
`98/100`; exact harness evidence and boundaries remain unchanged. Only
cached-diff/staging audit, implementation commit, and post-commit inventory/
fresh-next-task gates remain pending.

## Checker Task 258B3M2B2B3P Frozen Private Harness

After B2C commit `e8373c683448e524cb98edde83fdf8de83a125cd`, B3P freezes
a private proof-context reuse harness for the exact 117-byte set-enumeration
source. The real frontend must reproduce 57 nodes/root 56, local-only
resolver provenance, Task-48 `2/1/0`, Task-252 `6/4/2`, and Task-255
`1/0/0/0/0/2/1`. Enumeration term 0 is site `Node(40)`, range
`90..96`, source ordinal 0, context 1, recovery `Normal`, spelling
`{ 1 , 2 }`, and kind `Enumeration`. Its `EnumerationElement` edges are
exactly `(term 0, ordinal 0, Primary(2))` and
`(term 0, ordinal 1, Primary(3))`. Request 0 is term 0, ordinal 0,
`ResultType`, `generator = None`, `type_site = None`. Its primary
fingerprint is the exact Task-252 handoff fingerprint; application and
structure fingerprints are absent.

The private explicit-context helper must not alter the pre-existing
context-0 helper or its output bytes. Across the exact two tests, the
harness must:

- mutate all 117 loaded-source bytes including final LF and reject stripped/
  extra-LF variants;
- mutate kind, range, recovery, and ordered children for all 57 nodes and
  root identity;
- assert/substitute every local resolver shell, symbol, contribution, and
  provenance field;
- assert/mutate every Task-48 context/binding field, every Task-252 primary
  term/reference/numeric-request field, and every Task-255 term/
  `EnumerationElement` edge/request/fingerprint field;
- assert exact owner sets Task-252 `{30,32,36,38,44,46}`, Task-255 `{40}`,
  and unowned `0..29,31,33..35,37,39,41..43,45,47..56`;
- freeze precedence from source/module selector through arena/root,
  resolver, Task 48, Task 252, Task 255, stale fingerprint, typed/final
  clone validation; prove atomic rollback, clean replay, and exact final
  clones;
- assert empty Tasks 253/254/256/258, active/adjacent isolation, and empty
  semantic/proof/goal outputs.

The legacy Task-111 context-0 oracle is asserted literally: Task-255 handoff
debug SHA-256
`30b72230bb7ff39464962133b58df212e23afccccc8f4e4788ab9a9d0481c43a`,
full typed debug
`1bb296c06ab62691684260aa94987adee23081baa4a35aac9e485d95370d2cb9`,
and resolved debug
`cdb4eaae9605f62269d6a74d64267a8fcb1e8d8008564d8b9e014037665df1e4`.
Old/new equality measured in the implementation build is not an oracle.

No checker test, active fixture/sidecar/route, expectation, trace row/credit,
public API, statement witness, imported behavior, or semantic behavior is
added. Upper B3A owns the later witness-to-set-term consumer and its separate
checker/runner tests.

## Checker Task 258B3M2B2B3P Documentation Review Status

All four review tracks report **NO FINDINGS** and all recorded
source/count/hash/scope/trace-no-op checks pass. The exhaustive two-test
harness contract is frozen; its future implementation remains planned
`source_drift`/`test_gap`. Final quality, commit, post-commit, and fresh
implementation inventory are pending.

## Checker Task 258B3M2B2B3P Final Quality Status

Final quality has **NO FINDINGS**, all nine hard gates PASS, and valid
`98/100` (`20/20/15/14/10/10/5/4`). Only stage/commit, post-commit, and
fresh implementation inventory remain pending.

## Checker Task 258B3M2B2B3P Implemented Private Harness

The prerequisite contract at
`285a1f11c310bb313c4c6b4feae914eb11f74754` now has exactly two passing
tests:

- `task258b3m2b2b3p_set_enumeration_proof_context_reuse_is_exact`
- `task258b3m2b2b3p_set_enumeration_corruption_replay_and_legacy_output_fail_closed`

Together they cover every 117-byte/final-LF mutation, stripped/extra LF,
all kind/range/recovery/children fields of 57 nodes and root identity, an
independent 63-field resolver oracle, all 39 binding fields, every
Task-252/255 row, real prior-binding use-ordinal substitution, and coherent
application/structure dependencies. A shared fingerprint-only subprofile
makes each absent dependency clause independently observable. Every rejected
mutation is followed immediately by clean replay; stale and simultaneous
precedence, typed/resolved clone rollback, exact owner partitions, semantic
emptiness, legacy hashes, and active/adjacent isolation are fixed.

Focused `2/2`, runner library `446/446`, formatting, package and workspace
Clippy/tests, lint-policy `15/14`, metadata `137`, five CLI/current
manifest/test-list hashes, diff check, and exact 30-file scope PASS.
Test-sufficiency, implementation, source/documentation consistency repeat,
and documentation/boundary repeat are **NO FINDINGS**. Independent final
quality reports **NO FINDINGS**; all nine hard gates PASS with valid
`98/100` (`20/20/15/14/10/10/5/4`). Only commit/post-commit and fresh B3A
inventory remain pending.

## Checker Task 258B3M2B2B3A Frozen Runner Harness

The five exact runner tests are:

1. `task258b3m2b2b3a_real_frontend_freezes_set_enumeration_witness_contract`
2. `task258b3m2b2b3a_validation_precedence_mutation_and_replay_fail_closed`
3. `task258b3m2b2b3a_set_enumeration_and_byte_subtree_near_misses_are_exact`
4. `task258b3m2b2b3a_family_and_active_route_isolation_is_atomic`
5. `task258b3m2b2b3a_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`

They authenticate final-LF/all `117` bytes, all fields/root of `57` nodes,
local resolver including label/owner provenance, every Task-48/252/255/256/
258 row, the one witness/zero names, exact partition/graph, fingerprints,
empty/singleton/three-plus/parenthesized/nested/comprehension/choice/`qua`/
label near misses, all family hybrids/orders, immediate replay/rollback,
final clone, debug compatibility, isolation, and empty semantics.

Every failure follows precedence source/AST, resolver plus label, Tasks
48,252,255,256,258 base, witness, atomic publication, final clone. The
real fixture/expectation/trace remain unchanged and inactive; no existential
goal matching, proof acceptance, or active-route credit is asserted.

## Checker Task 258B3M2B2B3A Implemented Runner Harness

The exact route now consumes the unchanged B3P set-enumeration handoff and
publishes the frozen one-witness/zero-name set edge. The five named runner
tests cover the real frontend, exact resolver label, every lower/upper row,
Task-256 `72`, Task-258 `62`, and witness `21` field matrices, all `57`
surface nodes/root, both final-LF near misses, family tuples/routes,
rollback/replay, final clone/debug, and empty semantics. The paired four
checker tests remain exact. No fixture/expectation/trace/corpus activation or
semantic/proof credit changed. The second source/documentation consistency
repeat and final documentation/boundary reread report **NO FINDINGS**;
parent final verification listed in the crate plans passes, including exact
`39`-file scope. Independent final read-only quality review reports
**NO FINDINGS**. All nine hard gates PASS with no score cap; the valid score
is `98/100` (`20/20/15/14/10/10/5/4`). The stated semantic and coverage
deferrals remain unchanged as residual risk. Only the dedicated
implementation commit, post-commit invariant verification, and fresh
next-task inventory remain pending.

## Task 258B3M2B2B3B Dormant Runner Surface

B3B adds only an exact dormant source selector for the 118-byte
`FormulaStatementEmptySetEnumerationWitnessSmoke` input and five compound
runner tests. The selector must authenticate every byte/node, resolver and
lower handoff field, owner partition, zero-edge graph, family isolation,
replay/rollback, final clone, and empty semantics before publishing the
existing private detail route. It must not enter active discovery, change a
diagnostic/detail key, or reinterpret the inactive template fixture.

## Task 258B3M2B2B3B Implemented Runner Harness

The five frozen runner tests exercise the exact 118-byte input and all 50
nodes/root 49. Their byte matrix covers all 118 positions; their node
matrix covers all 50 nodes across the four surface mutation axes of kind,
range, recovery, and children. They
cover eight base-resolver and ten label-resolver mutations, every
currently constructible Task-48/252/255/256/258 handoff field, the frozen
Task-256 `72`-field and Task-258 `62`-field matrices, and the `21`-field
witness matrix. The four omitted Task-258 kind/role/status fields each have
only one safely constructible public variant, so the repeat reviewer
retracted that candidate finding as **NO DISAGREEMENT**.

The suite includes a non-vacuous two-edge rejection for the frozen
zero-edge contract and both B3A-before-B3B and B3B-before-B3A family
orders. It also verifies active-route isolation, immediate replay and
rollback, final clone/debug stability, and empty semantics. Initial
findings for resolver coverage, bidirectional ordering, and non-vacuous
zero-edge validation were closed inside the same five runner and four
checker tests. The repeated review then found one remaining B3B-specific
gap in currently mutable Task-48/252/255 mutation/replay coverage. The
bounded test-only remediation adds exact Task-48 `32`, Task-252 `55`, and
Task-255 `23` mutation/replay matrices without changing the test count,
fixture, expectation, trace row, or active route. Focused runner `5/5`,
checker `4/4`, libraries `398/456`, format/diff, workspace Clippy with
warnings denied, and final `cargo test -q` PASS. Post-auth injection plus
stage-prefix and non-generic-guard assertions complete authentication; all
test-sufficiency repeats and the final implementation repeat report
**NO FINDINGS**.

The source/documentation consistency repeat also reports **NO FINDINGS**
after confirming the four surface axes and recording the test-list hashes
as final remeasurements.

## Task 258B3M2B2B3C Frozen Harness

The dormant harness selector must authenticate only the exact `110`-byte,
`52`-node/root-`51` source and local theorem provenance before assembling the
existing explicit-context Task-255 choice handoff in proof context `1`.
Future tests exhaust all bytes/LF, four surface axes per node/root, resolver,
`32/55/39/72/62/21` lower/upper fields with replay and stage prefixes,
non-vacuous zero-edge corruption, choice target/request ordering, ownership,
family isolation, clone/rollback/debug stability, and empty semantic tables.
No corpus case, expectation, sidecar, trace count, CLI output, or active route
changes in this prerequisite or future transport.

## Task 258B3M2B2B3C Implemented Runner Harness

The dormant selector and frozen five runner tests now authenticate the exact
110-byte source and final LF, all 52 nodes/root 51 across kind/range/
recovery/children axes, eight base and ten label resolver mutations with
typed/resolved replay, and all safely mutable Task-48/252/255/256/258/
witness fields in exact `32/55/39/72/62/21` matrices. Exact
`Task256:`/`Task258:`/`B3C:` failure prefixes and generic-guard rejection
prevent fallback acceptance.

The suite also checks non-vacuous zero-edge rejection, choice target/request
order, ownership/subtree near misses, all six B3A/B3B/B3C family orders,
active-route isolation, immediate replay/rollback, final clone/debug
stability, and empty semantics. Initial two test gaps and the
B3A-hard-coded route finding are remediated; repeated test-sufficiency and
implementation reviews report **NO FINDINGS**.

Final measured sizes are statement `10305`, unchanged set leaf `4517`,
facade `779`, root `2595`, statement tests `23583`, and unchanged set tests
`2528`. Runner library is `461`; focused `5/5` and package
`461+3/14/137/2/21` pass. No active fixture, expectation, sidecar, trace,
CLI, diagnostic, or semantic harness surface changes.

## Task 258B3M2B2B3D Frozen Harness

The dormant harness selector must authenticate only the exact 109-byte,
54-node/root-53 qua source and local theorem/label provenance before
assembling the existing context-1 Task-255 handoff. Future tests exhaust
bytes/LF, four surface axes per node/root, resolver, the exact
`32/70/44/72/62/21` lower/upper matrices with replay and owning prefixes,
`QuaBase` and ordered requests, ownership/subtree isolation, all four B3
family orders, clone/rollback/debug stability, and empty semantic tables.
No active fixture, expectation, sidecar, trace, CLI, diagnostic, or semantic
harness surface changes.

## Task 258B3M2B2B3D Implemented Runner Harness Inventory

The dormant selector and five frozen runner tests now authenticate the exact
109-byte/final-LF source, all 54 nodes/root 53 across kind/range/recovery/
children axes, eight base plus ten label resolver mutations, and every
safely mutable Task-48/252/255/256/258/witness field in exact
`32/70/44/72/62/21` matrices. Each field mutation replays the clean route,
uses its owning stage prefix, and cannot pass through the generic
lower-dependency guard.

The suite also covers `QuaBase`, target/request and complete-subtree near
misses, all B3A/B3B/B3C/B3D pairings and 24 orders, active-route isolation,
clone/rollback/debug stability, and empty semantics. Test-sufficiency review
reports **NO FINDINGS**. Statement tests are now 24,769 lines; focused
runner `5/5` and package `466+3/14/137/2/21` pass. No fixture,
expectation, sidecar, trace, CLI, active diagnostic, or semantic harness
surface changed. Independent implementation review reports
**NO FINDINGS**. Source/documentation and boundary review also report
**NO FINDINGS** after the stale-review, 24-order, and qua-edge documentation
corrections. Both packages, formatting, full Clippy, workspace tests, five
CLIs, and count/hash reruns PASS. Independent final read-only quality review
reports **NO FINDINGS**; all nine hard gates PASS with no cap at valid
`100/100` (`20/20/15/15/10/10/5/5`). Only exact staging/cached-diff
review, implementation commit, and post-commit/fresh-next-task gates remain
pending.

## Task 258B3M2B2B3E Frozen Harness Route

The future private selector recognizes only the exact final-LF 139-byte
source/hash and full 60-node/root-59 surface/resolver profile. Four checker
and five runner tests must exhaust bytes/LF, `60 x 4` node surfaces/root,
resolver fields, `32/70/53/72/62/21` lower/upper fields, ownership,
generator-without-binding, all 120 B3A-E orders, replay, rollback, clone,
debug stability, active-route isolation, and empty semantics. No active test
case selects this source; fixtures, sidecars, expectations, trace metadata,
counts, and coverage credit remain unchanged.

The node/field matrices are supplemented by explicit post-auth negatives for
a present condition, zero/multiple generators, a nested comprehension,
a generator-referencing mapper, wrong/extra generator or type-site rows,
nonzero condition cardinality, and partial/extra ownership in the complete
`38..46` subtree. Resolver tests mutate all base-owner and enriched-label
fields. Every Task-48/252/255/256/258/B3E mutation asserts its owning-stage
prefix, rejects generic-guard-only failure, and repeats the failure
immediately against the authenticated baseline.

## Task 258B3M2B2B3E Implemented Runner Harness Inventory

The dormant selector and five tests authenticate all 139 bytes/final LF, 60
nodes/root 59 on four axes, resolver provenance, and every frozen field.
Post-auth negatives are successful coherent same-provenance Task-255
handoffs: B3A zero-generator, B3C wrong type-site, Task-255C1 condition,
synthetic valid multiple/nested, and successful empty generator-reference
exclusion. Each asserts the authenticated source/module identity, repeats
the B3E dependency/invalid failure, and cleanly replays. The separate
`32/70/53/72/62/21` mutation matrices assert owning-stage prefixes and
reject generic-guard-only failure.

All 120 orders, ownership/subtree and active isolation, clone/rollback/debug,
and empty semantics pass. Statement tests are 26,141 lines; focused `5/5`
and library `471` pass. Corpus/trace/CLI/semantics remain unchanged. Reviews
report **NO FINDINGS**.

The final consistency repeat reports **NO FINDINGS** after correcting the
harness responsibility overclaim and two synchronized documentation drifts.
Complete verification PASSes; independent quality reports **NO FINDINGS**,
all nine gates PASS, valid `100/100`. Staging and post-commit gates
subsequently closed in implementation commit
`e4479691db3b0a8785bb16e94d386bd71a394274`; fresh inventory selected
Task 258B4A.

## Checker Task 258B4A Frozen Harness Route

The harness recognizes only the exact private 80-byte/double-LF
explicit-universal theorem. It authenticates all 26 Surface nodes, the
local theorem resolver provenance, and the complete Task-252/256/257/B1
handoffs before calling the new Task-258 composite statement constructor and
paired typed installer. It consumes the existing lower output through the
single crate-private production-helper visibility seam and does not copy or
rebuild Task-257 rows. The 79-byte active Task-257B1 source must remain on
the lower-only route.

The four stage vectors cover statement selection, typed paired installation,
final construction, and stable reporting. Five frozen tests exhaust bytes/
nodes, lower and upper fields, stage-prefix failure and replay, coherent
near misses, both family orders, clone/debug, and empty semantic outputs.
The harness does not claim theorem truth, acceptance, proof, facts, or
formula-statement coverage.

Documentation-only test-sufficiency review reports **NO FINDINGS**. The
future implementation test review remains a separate task.

## Checker Task 258B4A Implemented Harness Route

The dormant selector now authenticates every frozen source byte and Surface
row, exact resolver owner, complete lower profiles, rootless lower typed
arena, and lower owned sites/ranges before publishing the composite
statement transaction. It reuses the production Task-257B1 handoff through
the crate-private seam rather than copying lower rows.

Five tests cover exact output; 142 lower mutations; 34 upper statement
mutations and owned-node substitutions; the coherent rooted-arena near
miss; resolver and active 79-byte isolation; family-order atomicity;
failure/replay; and final clone/debug/empty semantics. The checker suite
separately covers the coherent relocated-term near miss. Focused runner
`5/5` and separate test-sufficiency and implementation reviews report
**NO FINDINGS**. The harness adds no active route, truth, fact, acceptance,
proof, or coverage claim.

## Checker Task 258B4B Frozen Harness Route

The dormant selector authenticates the exact 167-byte/double-LF hash, all
kind/range/recovery/ordered-child fields for 124 Surface nodes and root 123,
the raw local theorem resolver provenance, and the enriched
`1/1/1/1/0` resolver environment including its one theorem label projection
and contribution label effect. It rebuilds the Task-257B2 lower handoffs in
one rootless arena and publishes only upper `1/1/1/0/1` with two
`Composite(0)` links. Its private route telemetry is the exact zero-reference
sentinel `0/0/[]`; the profile-aware detail guard accepts it only for matched
Task-257B2/B4B and leaves B4A at `1/1/[1,1]`.

The five exact runner tests named in the checker plan cover the complete
byte/LF, `124 x 4` node, raw/enriched resolver, label-effect, `0/0/[]`
telemetry/detail-guard, lower-row, upper-row, 42/1/81 ownership,
cardinality/fingerprint, coherent lower near-miss, active 166-byte/B4A/
atomic-family, order, rollback/replay, clone/debug, and empty-semantic
matrices. The paired checker suite owns the syntax-free corruption and final
allowlist checks. No active fixture, sidecar, trace, diagnostic, public
runner schema, connective truth, repetition expansion, theorem acceptance,
or proof result is added.

## Checker Task 258B4B Implemented Harness Route

The runner now authenticates the exact private 167-byte/double-LF source and
all 124 Surface rows/root 123. Raw resolver preflight is label-free; the
runner supplies the frozen theorem label projection and contribution effect
to reach exact `1/1/1/1/0`, then reuses the Task-257B2 lower handoffs in one
rootless `42/1/81` arena. Publication remains upper `1/1/1/0/1` with both
edges `Composite(0)`.

The route guard accepts `0/0/[]` only for the exact B2/B4B pair while B1/B4A
retains `1/1/[1,1]`. Five runner tests pass the exact output, mutation,
profile-isolation, both-order, rollback/replay, clone/debug, and empty
semantic matrices; the four checker tests cover syntax-free and final
allowlist rejection. Test-sufficiency and implementation reviews report
**NO FINDINGS**. The active 166-byte fixture remains lower-only, and no
public runner schema, active route, diagnostic, corpus/trace artifact, or
semantic result changes.

Final source/documentation, bilingual, and boundary review repeats report
**NO FINDINGS**. Focused checker `4/4` and runner `5/5`, full
`cargo test --offline`, `cargo fmt --all -- --check`, and full offline
Clippy with warnings denied pass. Checker/runner counts remain `418/481`;
production, test-list, and all five CLI counts/hashes reproduce the recorded
values, including `419/387`, `228/191`, `101/5/198/1`, and warnings/errors
`23/0`. Seven-file scope, spec-coverage-audit no-op, and unchanged-stash
gates pass. Independent final quality reports **NO FINDINGS**; all nine hard
gates PASS with no score cap at valid `100/100`
(`20/20/15/15/10/10/5/5`). Exact staging/cached-diff review, the
implementation commit, post-commit invariants, and fresh-B4C inventory
remain pending.

## Checker Task 258B4C Frozen Harness Route

B4B subsequently committed as
`752c17ae7d552d5268d1028612b8174e480b6f3e`; the clean ahead-1/behind-0
post-commit inventory and unchanged stash select B4C. The harness will
recognize only the exact private 139-byte/two-LF source with hash
`36e5a68a92451590644951838a9af8926212bd78f88d1f90563f12b650b161c1`.
The active 138-byte/one-LF source and hash
`cbfd7077713e8e9630900e349d5f579251c19fba55434acb62170ea1dd940237`
remain lower-only.

Before the upper route is implemented, an independent lower-stage
prerequisite must make the existing Task-257B3 selector accept exactly the
138- and 139-byte variants and reject zero or three trailing LFs. Its write
scope is only `type_elaboration/source_formula.rs` and
`runner/tests/type_elaboration/source_formula_composition.rs`; production
`source_formula_composition.rs` remains unchanged. Its added-test count is
not projected here and must be measured after fresh inventory.

The future upper selector authenticates all 66 Surface rows/root `65`,
theorem `62` at `19..137`, label token `6` at `27..65`, outer formula `60`
at `67..136`, raw resolver `1/0/1/1/0`, theorem path `[2,1]`, and reserve
contribution `0` anchored at `0..18`; only then may it enrich to
`1/1/1/1/0`. It reuses exact lower profiles binding `4/4/0`, primary
`6/6/0`, atomic `3/0/0/0/0/0/0/6/6`, composite
`3/0/1/3/3/2/6`, and composition `3/6`.

Publication is upper `1/1/1/0/1`: context `0` exposes binding `[0]`, there
are no input facts, and both statement and candidate target `Composite(0)`.
The rootless typed arena partition is `24/1/41`. The profile-aware detail
guard accepts telemetry `2/2/[2,2,4,4,4,4]` only for the exact B3/B4C pair.

Four projected checker and five projected runner tests cover source/LF
isolation, every frozen Surface/resolver/lower/upper field, ownership,
telemetry, coherent near misses, B4A/B4B and active-route isolation, order,
rollback/replay, clone/debug, and empty semantics. No fixture, expectation,
sidecar, trace status/count, active route, public schema, formula truth,
witness or restriction semantics, theorem acceptance, fact, proof/Core/CFG/
VC result, or coverage credit changes. Baselines remain libraries
`418/481`, checker production `23/140821`, runner production `30/56007`,
and the previously recorded production/test-list/five-CLI hashes.

## Checker Task 258B4C Implemented Harness Route

The runner now authenticates the exact 139-byte/two-LF source, all 66
Surface rows/root 65, raw owner/contribution provenance, and enriched
`1/1/1/1/0`. It reuses the separately admitted Task-257B3 lower handoffs,
marks only theorem node 62 as upper-owned, and publishes exact
`1/1/1/0/1` with both links `Composite(0)`.

The five frozen runner tests cover raw and enriched resolver mutations,
every Surface axis, all lower/upper fingerprints and rows, exact
`24/1/41` ownership including unowned anchor/recovery corruption, zero/
triple-LF and active-route isolation, B4A/B4B/atomic/Task-248 orders,
transport-detail telemetry hybrids, replay, clone/debug, and empty semantic
outputs. The shared production guard accepts only B4A `1/1/[1,1]`, B4B
`0/0/[]`, and B4C `2/2/[2,2,4,4,4,4]` for their exact matched profiles.

No public harness schema, active route, fixture, expectation, sidecar,
trace/coverage state, diagnostic, or semantic result changed.

## Checker Task 258B5A Frozen Private Route

The runner selector may admit only the exact 185-byte/final-LF private source
and must reject near-miss bytes and both unchanged authoritative fixtures.
It runs the real parser, resolver two-pass replay, BindingEnv, Task-252,
Task-256, and Task-258 producers; authenticates every row before installing
the paired base/reference handoffs; and assembles the final clone without
syntax fallback.

The output is exact one proof-step label, one simple-local citation,
left/right lookup ordinals `1/1`, ten reference-use ordinals all equal to
1, and empty semantic tables. The five frozen runner tests cover source
identity, raw/enriched resolver and lower mutations, scope/range/ordinal/
ownership corruption at the owning stage, B1/B5A cross-pairing and
installation order, replay/debug stability, selector isolation, and empty
semantics. No public harness field or error/debug grammar changes.

## Checker Task 258B5A Implemented Private Route

The private selector now admits only the exact 185-byte/final-LF source and
authenticates all 93 Surface rows before running the real resolver two-pass
replay. It reuses unchanged BindingEnv, Task-252, and Task-256 producers,
constructs exact base/reference handoffs, installs only matched B5A state,
and revalidates the immutable final clone without a syntax fallback.

The output preserves exact `20/73` ownership, one private/local label at
scope `[0]`, one simple-local citation at scope `[0,1]`, resolver node 82 to
label key 0, lookup ordinals `1/1`, ten reference-use ordinals equal to 1,
and empty semantic tables. Source, Surface, resolver, lower, row, scope,
ownership, cross-profile, replay, and clone near misses remain isolated at
their owning boundary. Public harness fields, active selectors, diagnostics,
facts, accepted statements, proofs, goals, and IR remain unchanged.

## Checker Task 258B5B Frozen Imported Route

The harness must not install B5B until a separately committed crate-private
opt-in import helper authenticates the exact public/exported imported
theorem label `Ref`. That helper is restricted to `import_fixtures.rs` and
the statement test leaf, produces exact resolver profile `8/1/1/3/1` from
normal `8/0/1/3/1`, preserves default callers, and is protected by the two
frozen `task258b5b_opt_in_*` tests.

After the lower commit, the private selector may admit only the exact
146-byte/final-LF source with SHA-256
`671e940c9dc749757dc8fddcc30a1a230aecb650058e64d6f1e73c1c66e93e9e`.
It authenticates all 57 Surface/resolver rows, Binding `2/1/0`, Task-252
`4/4/0`, Task-256 `2/0/0/0/0/0/0/4/4`, Task-258
`1/2/2/2/2 + 0/1`, and exact `8/49` ownership before final assembly.

Citation id 0 has dense citation-row ordinal 0, node/range `48 / 136..139`,
scope `[0]`, `LabelRefId(0)`, and the singular
imported/public/exported theorem projection with contribution 2, anchor
`7..27`, path `[1,0]`, and origin
`summary:parser.type_fixtures::Ref:label:Ref`. Telemetry is exact
`1/1/[1,1,1,1]`; the resolver reference candidate independently has
source-statement ordinal 1. The five runner tests cover exact output, all
lower/import/resolver/upper corruption, including independent citation-row
and resolver source-statement ordinal mutations and the coherent
`Exported`-to-`ReExported` near miss in runner test 2 and final-clone
coverage, fixture and visibility isolation, B5A cross-profile atomicity,
replay/debug stability, and empty semantics.

The upper runner reconstructs exact resolved import id 0: owner node 29,
range `7..27`, spelling `import parser.type_fixtures;`, alias `None`,
resolved module `<package>::parser.type_fixtures`, current-source/
current-module origin anchor `7..27`, path `[0]`, no import edge, and normal
recovery. Nodes 28/29/30 remain unkeyed `NotApplicable`; node 48 alone is
keyed. The imported projection origin is current-source/imported-module,
anchor `7..27`, path `[1,0]`, no import edge, normal; the reference origin
is current-source/current-module, anchor `136..139`, path `[48]`, no import
edge, normal. Runner test 2 and final-clone coverage mutate every field
independently.

Tests 1 and 5 assert the full ordered debug schema frozen by the checker
plan: literal `label_node=absent` and `source=imported`, complete imported
projection fields, resolver-ast/reference/result lines, the
`target=imported` citation line, and no `label#0` line. B1/B5A bytes remain
unchanged.

No public runner field, active selector, fixture, expectation, trace row,
diagnostic, fact, theorem acceptance, proof result, or downstream IR
changes. B5C and qualified/grouped/bulk citations remain deferred.

## Checker Task 258B5B Implemented Imported Route

Prerequisite commits `141dc44a` and `46dd9db5` now precede the upper route.
Production performs the special imported-label augmentation only for exact
source equality with the frozen 146-byte/final-LF text; every near miss keeps
the default augmentation path and cannot select B5B.

The implemented route preserves all 57 Surface/resolver identities and root
56, then validates raw/enriched resolver `1/0/1/1/0` and `8/1/1/3/1`,
Binding `2/1/0`, Task-252 `4/4/0`, Task-256 two formulas/four edges/four
requests, Task-258 `1/2/2/2/2`, reference `0/1`, and `8/49` ownership. The
sole citation is `Imported`/`SimpleImported`, resolves the public/exported
theorem `Ref` through contribution 2 and exact import/projection/reference
provenance, and emits no local label row. Typed and final clones retain empty
semantic outputs.

The five upper and two lower runner tests pass together as `7/7`; checker
B5B passes `4/4`, and the full libraries measure `500/500` and `430/430`.
B1/B5A selectors, target wrapping, public debug bytes, and cross-profile
atomicity remain unchanged. No public harness/CLI field, corpus case,
expectation, sidecar, trace row, diagnostic, semantic acceptance, proof, or
IR output is promoted.

## Checker Task 258B5C Frozen Declaration-Symbol Route

B5C is an active fail route, not another private checker statement profile.
It is blocked first on separately committed resolver R-032A structural
lowering, then R-032B proof-label collection. The runner calls
`SurfaceResolvedArena::lower(&ast, &module)` and
`validate_against(&ast, &module)`, then uses the exact linked
`impl<'a> ProofLabelSourceCollector<'a>` declaration:
`new(ast: &'a SurfaceAst, module: &ModuleId, namespace, contribution,
resolved: &'a SurfaceResolvedArena) -> Result<Self, ...>`, followed by
`collect(&self)`. Only ast/resolved share and store `'a`; module is
validation-only/not stored, and namespace/contribution are owned. It feeds
only returned
`projections()`/`references()` to `LabelResolver`. Both lower APIs fail
closed with the exact canonical resolver enums: R-032A includes state/key
mismatch variants and all node/child/overflow payloads are `SurfaceNodeId`.
The runner never computes a
`LabelScopePath`, source ordinal, structural origin, or `ResolvedNodeId`.

The exact 173-byte inner-to-outer and 197-byte sibling sources, hashes,
normal Surface nodes, projection `A` at scope `[0,0]`/visible-after ordinal
3, and candidates at `[0]`/ordinal 5 and `[0,1]`/ordinal 6 are frozen in the
crate plan. Each observation requires `1/1/[0]`, zero resolver diagnostics,
`has_unresolved = true`, and one `UnresolvedLabelRef` for `A` with
`ProofOrTheorem` expectation. Exact projection/reference structural paths
are `[57,42,8]`/`[57,55,52]` and
`[67,47,8]`/`[67,63,60]`. Any structural-map, overflow, resolved,
ambiguous, additional, recovered, or provenance-mismatched result fails the
route.

R-032B collection is deliberately narrower than the parser: normal
top-level theorem/direct-proof owners, exact labelled compact statements,
self unqualified `JustificationClause -> ReferenceList -> Reference` chains
under `by`, supported nested proofs, and normal `CompactStatement`/
`ConclusionStatement` ordinals only. The module-global one-based counter is
not reset across theorem roots; theorem/transparent/excluded subtrees
consume none, references use owning-statement ordinals, and visibility is
the labelled-subtree maximum. Exact B5C offsets are `2/3/3/4/5` and sibling
reference 6. Same-block post-completion is positive; own-proof
self-reference, inner-to-enclosing, sibling, and an earlier theorem `[0]`
label cited from later theorem `[1]` are unresolved. Same-spelling
declarations in distinct theorem roots do not conflict. These boundaries
and origin stability/uniqueness are lower tests, not runner-derived logic.
Origins use only resolver `labels.md`'s collision-free `proof-step-v1`
length-framed grammar, exact token bytes, zero-based occurrences, and
owner-relative proof paths.

The branch is selected solely by `FrontendRun.source_text` byte equality to
a frozen constant plus its exact normal AST profile—never metadata or
expectations. After shared `resolver_symbol_collection`, it requires matching
env/module, derives namespace only from the module path, and selects exactly
one matching local-source contribution/source id, with B5C contribution 0.
Input/provenance corruption emits only
`declaration_symbol.label.proof_scope_input`; only fully authenticated
unresolved confinement emits
`declaration_symbol.label.proof_scope_confinement`. Public codes stay empty,
and tests copy/mutate expectations to prove they cannot select the route.

The future sidecars are declaration-symbol/resolve failures with empty public
diagnostic codes, tag `active_declaration_symbol`, and private detail key
`declaration_symbol.label.proof_scope_confinement`; the two rejection reasons
remain distinct. The write scope is only `declaration_symbol.rs`,
`runner/tests.rs`, new `runner/tests/declaration_symbol.rs`, the two new
fixture/sidecar pairs, two trace rows, and synchronized derived documents.
Public harness/CLI schemas and checker, type, proof, Core/CFG/VC output are
no-ops. This exactly 48-file documentation prerequisite creates no active
case or coverage credit.

The collector is additionally frozen to the resolver's closed Surface edge
table: exact upper chain
`Root -> CompilationUnit -> ItemList -> direct TheoremItem -> ProofBlock`,
then only direct normal
`CompactStatement`/`ConclusionStatement`; compact proposition-label
inspection only; and from either statement only direct `ProofBlock` and
`JustificationClause`. A candidate requires the exact
`JustificationClause -> ReferenceList -> simple Reference -> sole identifier
token` chain. Formulae, tokens, wrappers, unsupported/recovered/malformed
nodes, qualified/grouped/bulk citations, and templates are skipped without
an ordinal or descent. Positive tests cover every allowed edge, including
each upper edge. Upper negatives cover missing/additional/wrong Root and
CompilationUnit children, direct Root/CompilationUnit theorem relocation,
and `VisibleItem` wrapping; other forbidden relocation proves default
denial. Mixed-list tests preserve only exact simple `Reference` siblings in
source order while unsupported siblings add no row or descent.

Runner authentication is field-by-field. It requires
`env.module_id() == resolver.module`, derives
`NamespacePath::new(module.path().as_str())`, validates every projection
namespace, acquires exactly one contribution, and validates id 0,
`LocalSource`, record module, and LocalSource source id against public field
`ast.source_id`. Every projection module, namespace, and contribution must
equal those authenticated values. Independent mutations cover environment
module; projection module/namespace/contribution id; zero and multiple
contributions; contribution id; each of `ImportedSource`, `Summary`, and
`Builtin`; contribution record module; and LocalSource source id. Each
mutation emits only `declaration_symbol.label.proof_scope_input`, never
confinement or a public code. Source bytes plus normal AST remain the only
selector, including under copied or mutated expectations.

## Checker Task 258B5C Implemented Declaration-Symbol Route

The current implementation adds exactly two active fail cases. Their runner
path consumes the unchanged R-032A `SurfaceResolvedArena`, R-032B
`ProofLabelSourceCollector`, and `LabelResolver`; it never constructs a
scope, ordinal, origin, contribution id, or resolved node. Exact
source-plus-normal-AST authentication precedes the shared declaration-symbol
environment, projection/reference provenance, and unresolved-result checks.

Both cases produce only
`declaration_symbol.label.proof_scope_confinement` with an empty payload
table and no public code. The corruption matrix proves every mismatched
environment, contribution, projection, reference, or resolver result emits
only `declaration_symbol.label.proof_scope_input`. Expectation fields cannot
select the route, replay/order is deterministic, and all five earlier active
declaration-symbol cases retain their results.

## Checker Task 259 Frozen Runner Route

The future type-elaboration leaf selects only the exact 165-byte source and
its normal 71-row/root-70 AST. It authenticates the current source/module,
predicate shell ordinal 1 beneath definition-block shell 0, predicate
Symbol/Definition ids and kinds, contribution, normal origin `61..122`,
structural path `[4,0,8,0]`, notation spelling, and the same-block property
shell at `125..159`. The resolver's generic property Attribute/Attribute
projection is observed only as raw-profile evidence and is never supplied as
predicate-property semantics.

After the separate Task-248 extension, the leaf reuses its existing handoff
for definition parameters `x` and `y`, then builds Task 249 `2/2/0`, Task 252
`4/4/0`, and Task 256 `2/0/0/0/0/0/0/4/4`. It supplies Task 259 with one
predicate, two parameters, one guard, one property, and one correctness row.
The one pending obligation is a pass result with empty diagnostics. The
computation justification is not run, accepted, or discharged.

Focused tests must cover the exact payload/ranges/order, every independent
resolver and lower-handoff field mutation, missing/duplicate/reordered/
cross-owner rows, guard/definiens swaps, property kind/range/owner changes,
obligation id/kind/owner/range/assumptions/goal/provenance/status mutations,
transactional failure, typed/final clone/debug preservation, same and reverse
corpus order, exact-source/AST near misses, expectation non-selection, and
isolation from absent Tasks 253--255, 257, 258, and 260+. `Blocked` or
`Invalidated` may not substitute for `Pending`. The existing mixed
predicate-plus-functor route must keep its old extraction gap and never
select this leaf.

Only the later implementation adds one pass sidecar, one trace row, and the
mechanical active-type metadata increments. This frozen runner contract
changes no runner source, fixture, sidecar, trace, diagnostic, or count.

## Checker Task 248 Two-Parameter Dormant Extractor

The prerequisite adds no active harness route. The later private helper is
called only by tests and, afterward, by the exact Task-259 leaf after that
leaf selects source/AST/definition identity. It authenticates real
DefinitionBlock shell 0, direct leading parameter nodes 41/45, exact
`x`/`y` and bare `set` token/range shapes, scope/ordinals, and four sites in
one shared typed arena. It builds only the existing Task-248 projection.

Default denial rejects a third/non-leading parameter, reserve or extra-item
contamination, recovery, wrong shell/module/range/type/token/local identity,
stale or duplicate sites, and any attempt to treat excluded descendants as
bindings. The existing Profile-A selector/output/recovery/debug and active
fixture remain byte-compatible. No expectation field can select the dormant
helper, and no diagnostic detail key is added.

## Checker Task 259 Active Predicate-Definition Route

The private route is selected only by byte-exact source plus the complete
normal 71-row/root-70 surface profile. It authenticates the definition block,
ordered `x`/`y` parameters, guard, predicate pattern/definiens, symmetry
property, raw predicate resolver entry, same-block siblings, and exclusion of
pattern/label/justification descendants. The sidecar outcome, stage, tags,
diagnostics, and expectation data cannot select the route.

After selection the route creates one shared surface-indexed `TypedArena` and
calls the existing owners in the exact order Task 248, Task 249 `2/2/0`, Task
252 `4/4/0`, Task 256 `2/0/0/0/0/0/0/4/4`, and Task 259 `1/2/1/1/1`. Task
259 preserves the input obligation baseline and appends one `Pending`
`PredicatePropertyCorrectness` row with empty assumptions; typed/final
installation remains all-or-nothing. The route publishes no property proof,
fact, axiom, VC, acceptance, public diagnostic, or Task-260 mixed-family
payload.

The four runner tests are
`task259_real_source_surface_resolver_and_lower_bundle_is_exact`,
`task259_source_ast_resolver_and_lower_mutations_fail_at_the_owner`,
`task259_expectation_selection_and_mixed_definition_route_stay_isolated`, and
`task259_route_publishes_no_property_proof_fact_or_acceptance`. They pass
`4/4`; the full runner library count is `512`. The only adjacent active-count
change is an independently reviewed `198 -> 199` in two source-statement
selection tests, whose empty-selection assertions remain unchanged.

## Checker Task 260 Frozen Functor-Definition Route

The future private route is selected only by the exact 262-byte source and
the complete normal 108-row/root-107 surface profile frozen in
`mizar-checker/en/source_functor_definition.md`. It authenticates the
definition block, ordered `x`/`y` parameters, one `assume` guard, the
`equals` and `means` functor definitions, their explicit return types, raw
resolver provenance, two explicit correctness clauses, and all specified
excluded descendants. Sidecar outcome, stage, tags, diagnostics, and
expectation data cannot select the route.

After selection the route creates one shared surface-indexed `TypedArena` and
calls only existing lower owners in order: Task 248 Profile B
`1/2/2/2/2/2/0`, Task 249 + 249R `2/4/0/2`, Task 252 `5/5/0`, and Task 256
`2/0/0/0/0/0/0/4/4`; Task 253 through 255 remain absent for the frozen
source. Task 260 then publishes tables `2/2/1/2/2`, preserves the input
obligation baseline, and appends pending `FunctorExistence` and
`FunctorUniqueness` rows only for the `means` definition. Typed/final
installation is all-or-nothing.

The four future runner tests are
`task260_real_source_surface_resolver_and_lower_bundle_is_exact`,
`task260_source_ast_resolver_and_lower_mutations_fail_at_the_owner`,
`task260_expectation_selection_and_predicate_route_stay_isolated`, and
`task260_route_publishes_no_proof_fact_acceptance_or_vc`. They project the
runner library count from `512` to `516`. The exact implementation also
updates the six mechanical active-type consumers from `199` to `200`; it does
not publish goal composition, proof, discharge, acceptance, facts/axioms,
VC/IR, or Task-259 predicate payload.

Task 249R is checker-only and adds no runner route or test. The runner begins
Task 260 only after fresh inventory confirms the combined source-type handoff
has two binding applications, four expressions, zero arguments, and two
definition-return rows; it fingerprints that complete handoff and never
fabricates a return-type binding.

The first test asserts every source byte/final LF/hash, all 108 Surface rows
and ordered children, the root/sibling/subtree partition, resolver profile,
lower bundle, and final output. The second mutates each source/AST/resolver/
lower family and every excluded descendant so failure stays at the owner. The
third proves expectation non-selection, Task-259 and mixed-route isolation,
and—together with metadata `137/137`—the sole reciprocal trace backlink. The
fourth proves the computation subtrees and all proof/discharge/acceptance/
fact/VC outputs remain absent and audits all six count consumers.

## Task 249M No-Consumer Harness Boundary

Task 249M adds no harness route or test. Its four checker-only tests freeze the
standalone RHS lower handoff; the repository corpus, metadata test `137`, five
CLI outputs, runner list `520`, fixture/sidecar/trace hashes, and mixed mode/
structure gap remain byte-identical. The later Task-262 runner alone may build
and fingerprint the `2/3/0/0/1` handoff from real source.
