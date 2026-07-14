# Module: harness

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

No exhaustive public enum exceptions are owned by this module.

## Runner Modes

| Mode | Behavior |
|---|---|
| metadata plan | discover sidecars and validate layout, expectation schema, and traceability without executing payloads |
| parse-only | run active `.miz` parse-only cases through `mizar-frontend` and `MizarParserSeam` |
| declaration-symbol | run active `.miz` declaration-symbol cases through frontend parsing and resolver declaration/symbol collection |
| type-elaboration | run active `.miz` type-elaboration cases through frontend parsing and resolver declaration/symbol collection, extract supported reserve-only declaration payloads, delegate checker-owned `BindingEnv`/`DeclarationInput`/`DeclarationChecker` handoff production to the syntax-free `mizar-checker` seam, continue successful bare-builtin, task-55 bare local-mode-expansion, task-56 one-edge local-mode chain, and task-74 structural bare local-mode chain cases through `TypedAst` and `ResolvedTypedAst`, confirm `mizar-core` summary-readiness through `ResolvedTypedAstSummary::from_ast`, prepare binder-only `CoreContext` input from the same reserve bindings, surface same-module attributed reserve declarations, local structure reserve heads, attributed local structure reserve heads, task-57 real local-mode expansions with local structure RHSs, task-58 real local-mode expansions with attributed builtin RHSs, task-59 attributed local-mode reserve heads with real direct bare-builtin expansions, task-60 attributed local-mode reserve heads with real direct local-structure RHS expansions, task-61 attributed local-mode reserve heads with real direct attributed-builtin RHS expansions, task-62 one-edge bare local-mode chains ending in local structure RHSs, task-63 one-edge bare local-mode chains ending in attributed builtin RHSs, task-64 attributed local-mode reserve heads with one-edge bare-builtin chains, task-65 attributed local-mode reserve heads with one-edge structure-RHS chains, and task-66 attributed local-mode reserve heads with one-edge attributed-builtin-RHS chains as checker evidence-query gaps, surface same-module local mode reserve heads that lack the narrow task-55/task-56/task-57/task-58/task-59/task-60/task-61/task-62/task-63/task-64/task-65/task-66/task-74 expansion slices, including mixed attributed/bare local-mode sources, attributed chain dependencies, and chains that violate task-74 structural guards, as checker mode-expansion payload gaps, surface task-67 structure-qualified attribute references, task-68 argument-bearing local-mode reserve heads, task-69 argument-bearing local-structure reserve heads, task-70 bracket-form local-mode reserve heads, and task-71 bracket-form local-structure reserve heads as source-to-checker extraction-gap boundary cases, surface task-75 forward local-mode reserve heads, task-76 forward local-structure reserve heads, and task-77 forward local-attribute reserve type expressions as lower-stage active-range boundary cases before checker handoff, and surface unsupported checker payload families as stable external dependency gaps |
| pass/fail | run `.miz` cases and match expected outcome |
| snapshot | compare canonical snapshot hashes |
| determinism | repeat runs and compare artifacts, diagnostics, and hashes |
| parallel-equivalence | compare sequential and parallel outputs |
| fuzz-regression | run minimized fuzz cases as ordinary committed tests |
| update | rewrite snapshots only when explicitly requested |

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
set-enumeration result-type/sethood payloads, and partial formula checking
until real set-enumeration result types, term inference, equality/formula
checking, recorded facts, theorem acceptance, and `formula_statement` runner
support exist. The runner must not synthesize sethood/result payloads, theorem
acceptance, CoreIr, ControlFlowIr, VC, or proof payloads.

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
| `mizar-vc` task 15 | `proof_verification` | paced/open; VC/proof-verification obligations are deferred | Add runner support only after source-to-core/source-to-VC extraction and downstream verification seams exist. |
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
result-type/sethood payloads, and partial formula checking. It does not satisfy
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
