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

Task 80 addendum for `type-elaboration`: before tasks 84 and 85, imported
attribute reserve types from the documented `parser.type_fixtures` import
summary were active source-to-checker extraction-gap boundary cases. Imported
attributes outside the task-84 `TypeCaseAttr` provenance/`AttributeInput`
bridge and task-85 negative `empty`/builtin-`set` bridge still expect
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

Task 85 addendum for `type-elaboration`: the documented
`parser.type_fixtures` imported attribute `empty` may be passed as a
checker-owned imported negative `AttributeInput` only on builtin `set` for the
existing `non empty set` fixture. The runner expects
`type_elaboration.checker.checker.declaration.deferred.evidence_query` and must
not treat the summary as imported module AST extraction or synthesize
attributed-type existential/evidence payloads, positive `empty set`, imported
`empty` on non-`set` heads, positive imported attributed type elaboration,
structure-qualified attribute owner provenance, attribute arguments, CoreIr,
ControlFlowIr, VC, or proof payloads.

Task 86 addendum for `type-elaboration`: a formula-only theorem source may run
through parser and resolver as an active checker boundary case, but it must
remain on `type_elaboration.external_dependency.ast_payload_extraction` until
real theorem/formula payload extraction, local proof context, recorded facts,
theorem acceptance, and `formula_statement` runner support exist. The runner
must not synthesize formula payloads, proof skeletons, CoreIr, ControlFlowIr,
VC, or proof payloads.

Task 87 addendum for `type-elaboration`: a term-bearing theorem formula source
may run through parser and resolver as an active checker boundary case, but it
must remain on `type_elaboration.external_dependency.ast_payload_extraction`
until real term/formula payload extraction, term inference, formula checking,
recorded facts, theorem acceptance, and `formula_statement` runner support
exist. The runner must not synthesize term payloads, formula payloads, proof
skeletons, CoreIr, ControlFlowIr, VC, or proof payloads.

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
| `mizar-checker` task 12 plus task 16-20, task 48 source bridge continuation, task 50 attributed reserve evidence-gap bridge, task 51 local mode expansion-gap bridge, task 52 local structure evidence-gap bridge, task 53 attributed local structure evidence-gap bridge, task 54 attributed local mode expansion-gap bridge, task 55 bare local mode expansion bridge, task 56 local-mode expansion chain bridge, task 57 local-mode structure-RHS evidence-gap bridge, task 58 local-mode attributed-builtin-RHS evidence-gap bridge, task 59 attributed local-mode reserve evidence-gap bridge, task 60 attributed local-mode structure-RHS evidence-gap bridge, task 61 attributed local-mode attributed-builtin-RHS evidence-gap bridge, task 62 local-mode structure-RHS chain evidence-gap bridge, task 63 local-mode attributed-builtin-RHS chain evidence-gap bridge, task 64 attributed local-mode bare-builtin chain evidence-gap bridge, task 65 attributed local-mode structure-RHS chain evidence-gap bridge, task 66 attributed local-mode attributed-builtin-RHS chain evidence-gap bridge, task 67 structure-qualified attribute extraction-gap boundary, task 68 argument-bearing mode reserve extraction-gap boundary, task 69 argument-bearing structure reserve extraction-gap boundary, task 70 bracket-form local mode reserve extraction-gap boundary, task 71 bracket-form local structure reserve extraction-gap boundary, task 72 two-edge bare local-mode chain bridge, task 73 three-edge bare local-mode chain bridge, task 74 structural bare local-mode chain bridge, task 75 local-mode forward-reference active-range boundary, task 76 local-structure forward-reference active-range boundary, task 77 local-attribute forward-reference active-range boundary, task 78 imported structure reserve extraction-gap boundary, task 79 imported mode reserve extraction-gap boundary, task 80 imported attribute reserve extraction-gap boundary, task 82 imported mode provenance bridge, task 83 imported structure provenance bridge, task 97 imported TypeCaseStruct provenance bridge, task 84 imported attribute provenance bridge, task 85 imported non-empty attribute provenance bridge, task 86 theorem formula extraction-gap boundary, task 87 term formula extraction-gap boundary, task 88 proof skeleton extraction-gap boundary, task 89 statement proof extraction-gap boundary, task 90 predicate/functor definition extraction-gap boundary, task 91 attribute definition extraction-gap boundary, task 92 mode/structure definition extraction-gap boundary, task 93 proof-local declaration extraction-gap boundary, task 94 proof-local inline definition extraction-gap boundary, task 95 registration block extraction-gap boundary, task 96 redefinition/notation extraction-gap boundary, reserve summary-readiness, and binder-only core context follow-up | `type_elaboration` / `type-elaboration` | prepared/implemented; active sidecars use `active_type_elaboration`, lower stages run first, reserve-only builtin `set`/`object` declarations are extracted from `.miz` AST into syntax-free checker payloads, same-module attribute symbols already present in `SymbolEnv` may be attached to builtin reserve type payloads, same-module local mode reserve heads, and same-module local structure reserve heads, same-module local mode and structure symbols may be used as argument-free reserve heads, task 55 additionally extracts a real mode expansion only for bare local-mode reserve uses whose unique preceding no-argument same-module mode definition has a bare builtin RHS and no definition-local context, task 56 extracts a one-edge local-mode chain only when the dependency mode already has that accepted task-55 builtin RHS expansion, task 57 extracts a real local-mode expansion whose RHS is a same-module local structure head as a terminal expansion payload, task 58 extracts a real local-mode expansion whose RHS is an attributed builtin head as a terminal expansion payload, task 59 extracts a real direct bare-builtin local-mode expansion for an attributed local-mode reserve head when the same mode is not also used as a bare reserve head, task 60 extracts a real direct local-structure RHS local-mode expansion for an attributed local-mode reserve head when the same mode is not also used as a bare reserve head, task 61 extracts a real direct attributed-builtin RHS local-mode expansion for an attributed local-mode reserve head when the same mode is not also used as a bare reserve head, task 62 extracts both real expansions for a one-edge bare local-mode chain ending in a same-module local structure RHS under the same unique/unrecovered/preceding/no-context source constraints, task 63 extracts both real expansions for a one-edge bare local-mode chain ending in an attributed builtin RHS under the same source constraints plus argument-free same-module RHS attributes, task 64 extracts both real expansions for an attributed local-mode reserve head whose one-edge dependency chain ends in a bare builtin RHS when the root is not mixed with a bare reserve use and the dependency is not itself attributed, task 65 extracts both real expansions for an attributed local-mode reserve head whose one-edge dependency chain ends in a same-module local structure RHS when the root is not mixed with a bare reserve use, the dependency is not itself attributed, and the structure definition is unique, unrecovered, same-module, and source-preceding, task 66 extracts both real expansions for an attributed local-mode reserve head whose one-edge dependency chain ends in an attributed builtin RHS when the root is not mixed with a bare reserve use, the dependency is not itself attributed, and RHS attributes are argument-free same-module symbols, task 67 proves structure-qualified attribute references are parser/resolver executable but must remain on the extraction-gap key until real qualifier and attribute-owner provenance exist, task 68 proves argument-bearing local-mode reserve heads are parser/resolver executable but must remain on the extraction-gap key until real type-argument and term-argument provenance exist, task 69 proves argument-bearing local-structure reserve heads are parser/resolver executable but must remain on the extraction-gap key until real type-argument and term-argument provenance exist, task 70 proves bracket-form local-mode reserve heads are parser/resolver executable but must remain on the extraction-gap key until real bracket type-argument and `qua`-argument provenance exist, task 71 proves bracket-form local-structure reserve heads are parser/resolver executable but must remain on the extraction-gap key until real bracket type-argument and `qua`-argument provenance exist, task 72 extracts real two-edge bare local-mode chains ending in builtin `set` / `object`, task 73 extracts real three-edge bare local-mode chains ending in builtin `set` / `object`, task 74 extracts AST-bounded structural bare local-mode chains ending in builtin `set` / `object`, task 75 records forward same-module local-mode reserve heads as lower-stage active-range rejections before checker handoff, task 76 records forward same-module local-structure reserve heads as the same lower-stage active-range rejection before checker handoff, task 77 records forward same-module local-attribute reserve type expressions as the same lower-stage active-range rejection before checker handoff, task 78 historically records the documented imported structure `R` reserve head as a source-to-checker extraction-gap boundary case before task 83 supersedes that `R` portion, task 79 records imported mode reserve heads from that same import summary as source-to-checker extraction-gap boundary cases, task 80 historically records imported attribute reserve types from that same import summary as source-to-checker extraction-gap boundary cases before task 84 supersedes the documented `TypeCaseAttr` portion and task 85 supersedes the negative `empty`/builtin-`set` portion, task 82 promotes the `TypeCaseMode` imported mode summary symbol to a checker type-head payload that stops at the checker missing mode-expansion diagnostic, task 83 promotes the `R` imported structure summary symbol to a checker type-head payload that stops at the checker missing structure-evidence query, task 97 promotes the `TypeCaseStruct` imported structure summary symbol to the same checker type-head payload and missing structure-evidence query, task 84 promotes the `TypeCaseAttr` imported attribute summary symbol to a checker `AttributeInput` payload that stops at the checker missing attributed-type evidence query, task 85 promotes the imported `empty` attribute summary symbol only for negative `non empty set` to a checker `AttributeInput` payload that stops at the same evidence-query diagnostic, task 86 executes a formula-only theorem source through parser/resolver but keeps it on the checker source-to-payload extraction gap, task 87 executes a term-bearing equality theorem source through parser/resolver but keeps it on the checker source-to-payload extraction gap, task 88 executes a proof-block theorem source through parser/resolver but keeps it on the checker source-to-payload extraction gap, task 89 executes statement-level proof-justification theorem sources through parser/resolver but keeps them on the checker source-to-payload extraction gap, and task 90 executes predicate/functor definition sources through parser/resolver but keeps them on the checker source-to-payload extraction gap, and task 91 executes attribute definition sources through parser/resolver but keeps them on the checker source-to-payload extraction gap, and task 92 executes mode/structure definition sources through parser/resolver but keeps them on the checker source-to-payload extraction gap, and task 93 executes proof-local declaration statements through parser/resolver but keeps them on the checker source-to-payload extraction gap, and task 94 executes proof-local inline definitions through parser/resolver but keeps them on the checker source-to-payload extraction gap, and task 95 executes registration blocks through parser/resolver but keeps them on the checker source-to-payload extraction gap, and task 96 executes redefinition/notation surfaces through parser/resolver but keeps them on the checker source-to-payload extraction gap, `mizar-checker` produces the checker-owned `BindingEnv`, one `DeclarationInput` per binding, binding-specific `TypeExpressionInput` sites, and `DeclarationChecker` output, successful bare-builtin, task-55 bare local-mode, task-56 chain, and task-74 structural bare-chain cases continue through `TypedAst`, checker-owned `ResolvedTypedAst`, a `mizar-core` `ResolvedTypedAstSummary::from_ast` read, and binder-only `CoreContext` preparation, while attributed reserve, local-structure, task-57 structure-RHS expansion, task-58 attributed-RHS expansion, task-59 attributed local-mode expansion, task-60 attributed local-mode structure-RHS expansion, task-61 attributed local-mode attributed-RHS expansion, task-62 local-mode structure-RHS chain expansion, task-63 local-mode attributed-RHS chain expansion, task-64 attributed local-mode bare-builtin chain expansion, task-65 attributed local-mode structure-RHS chain expansion, and task-66 attributed local-mode attributed-RHS chain expansion cases stop at the checker `MissingEvidenceQuery` diagnostic and local-mode cases outside task 55/56/57/58/59/60/61/62/63/64/65/66/74, including mixed attributed/bare local-mode sources, attributed chain dependencies, or chains that violate task-74 structural guards, stop at the missing mode-expansion diagnostic; task-67 structure-qualified attribute cases, task-68 argument-bearing mode cases, task-69 argument-bearing structure cases, task-70 bracket-form mode cases, task-71 bracket-form structure cases, broader imported-structure cases outside the task-83 `R` bridge and task-97 `TypeCaseStruct` bridge remain deferred until a matching source-derived fixture exists; broader imported-attribute cases outside the task-84 `TypeCaseAttr` bridge and task-85 negative `empty`/builtin-`set` bridge and unsupported checker payload families stay on `type_elaboration.external_dependency.ast_payload_extraction`; task-82 `TypeCaseMode` imported mode cases stop at `type_elaboration.checker.checker.type.external.mode_expansion_payload`; task-83 `R` imported structure cases, task-97 `TypeCaseStruct` imported structure cases, task-84 `TypeCaseAttr` imported attribute cases, and task-85 negative `empty`/builtin-`set` imported attribute cases stop at `type_elaboration.checker.checker.declaration.deferred.evidence_query`; task-75 forward local-mode reserve heads, task-76 forward local-structure reserve heads, and task-77 forward local-attribute reserve type expressions stay on `type_elaboration.lower_stage.frontend:malformed_type_expression` before checker handoff; task-86 formula-only theorem sources stay on `type_elaboration.external_dependency.ast_payload_extraction` without formula payloads, facts, proof skeletons, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-87 term-bearing theorem formulas stay on the same extraction gap without term/formula payloads, term inference, formula checking, facts, proof skeletons, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-88 proof-block theorem sources stay on that extraction gap without proof skeleton payloads, local proof contexts, formula payloads, facts, theorem acceptance, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-89 statement-proof theorem sources stay on that extraction gap without statement proof payloads, nested proof skeleton payloads, local proof contexts, formula payloads, label-reference semantic checking, facts, theorem acceptance, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-91 attribute definition sources stay on that extraction gap without definition declaration payloads, definition-local context, formula-definiens payloads, attributed-type evidence, facts, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-92 mode/structure definition sources stay on that extraction gap without definition declaration payloads, mode expansion, structure base-shape/constructor/selector evidence, definition-local context, facts, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-93 proof-local declaration statement sources stay on that extraction gap without proof-local declaration payloads, local proof contexts, formula/term payloads, RHS term inference, reconsider coercion/obligation evidence, facts, theorem acceptance, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-94 proof-local inline definition sources stay on that extraction gap without inline definition formal/body payloads, local abbreviation expansion, term/formula body payloads, guard evidence, facts, theorem acceptance, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` runner activation; task-95 registration-block sources stay on that extraction gap without registration-item payloads, correctness-condition/proof-obligation payloads, accepted activation/evidence status, cluster/reduction semantics, Chapter 17 semantic rows, facts, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` / `advanced_semantics` runner activation; task-96 redefinition/notation sources stay on that extraction gap without redefinition payloads, notation alias relation payloads, redefinition target inference, coherence proof-obligation payloads, overload candidate payloads, Chapter 11 alias semantic resolution, Chapter 19 overload/redefinition semantics, facts, CoreIr, ControlFlowIr, VC, proof payloads, or `formula_statement` / `advanced_semantics` runner activation | Broader type/formula pass/fail semantic assertions wait for AST-wide source-to-checker payload extraction and real existential/evidence-query/mode-expansion/base-shape/imported-structure/imported-attribute/qualified-attribute/type-argument/term-argument/bracket-argument/theorem-formula/proof-context provenance inputs beyond the task-55 bare builtin RHS, task-56 one-edge chain, task-57 structure-RHS diagnostic slice, task-58 attributed-RHS diagnostic slice, task-59 attributed local-mode reserve diagnostic slice, task-60 attributed local-mode structure-RHS diagnostic slice, task-61 attributed local-mode attributed-RHS diagnostic slice, task-62 local-mode structure-RHS chain diagnostic slice, task-63 local-mode attributed-RHS chain diagnostic slice, task-64 attributed local-mode bare-builtin chain diagnostic slice, task-65 attributed local-mode structure-RHS chain diagnostic slice, task-66 attributed local-mode attributed-builtin-RHS chain diagnostic slice, task-67 extraction-gap boundary slice, task-68 extraction-gap boundary slice, task-69 extraction-gap boundary slice, task-70 extraction-gap boundary slice, task-71 extraction-gap boundary slice, task-72 two-edge bare local-mode pass slice, task-74 structural bare local-mode pass slice, task-78 historical extraction-gap boundary slice, task-79 extraction-gap boundary slice, task-80 historical extraction-gap boundary slice, task-82 TypeCaseMode provenance bridge, task-83 `R` imported-structure provenance bridge, task-97 `TypeCaseStruct` imported-structure provenance bridge, task-84 `TypeCaseAttr` imported-attribute provenance bridge, task-85 negative `empty`/builtin-`set` provenance bridge, task-86/task-87 theorem/formula extraction-gap boundary slices, task-88 proof-skeleton extraction-gap boundary slice, task-89 statement-proof extraction-gap boundary slice, task-92 mode/structure definition extraction-gap boundary slice, task-93 proof-local declaration extraction-gap boundary slice, task-94 proof-local inline definition extraction-gap boundary slice, task-95 registration block extraction-gap boundary slice, task-96 redefinition/notation extraction-gap boundary slice, and task-75/task-76/task-77 active-range boundary slices. |
| `mizar-checker` task 29 | `formula_statement` / `advanced_semantics` | paced/open; trace rows are deferred and no active fixture is fabricated | Add runner support only after statement/formula and advanced-semantics source payload seams exist. |
| `mizar-vc` task 15 | `proof_verification` | paced/open; VC/proof-verification obligations are deferred | Add runner support only after source-to-core/source-to-VC extraction and downstream verification seams exist. |
| `mizar-atp` task 20 | `advanced_semantics` metadata handoff | paced/open in `mizar-test`; metadata-only property fixtures may be consumed by `mizar-atp` Rust tests | Add active `.miz` ATP runner support only after source-derived ATP extraction and proof-policy/kernel handoff seams exist. |
| `mizar-kernel` task 17 | proof/certificate/kernel evidence | paced/open; fail/soundness metadata is validated without active proof/certificate/kernel execution | Add runner support only after source-to-evidence or certificate execution seams exist. |

Task 85 refines the `type_elaboration` consumer row above: the imported
attribute gap list now excludes only the documented negative `empty` over
builtin `set` fixture in addition to task 84's `TypeCaseAttr` fixture. The
runner keeps active external-gap sidecars for positive `empty set` and imported
`empty` on builtin `object`, while generic imported attributes, imported module
AST extraction, attribute arguments, owner provenance, evidence payloads,
CoreIr, ControlFlowIr, VC, and proof rows stay outside the supported slice.

Task 86 refines the same row: formula-only theorem sources are executable
through the active `type_elaboration` runner only as extraction-gap boundaries.
They do not satisfy the deferred `formula_statement` runner obligation and do
not credit theorem/formula payloads, facts, proof skeletons, CoreIr,
ControlFlowIr, VC, or proof payloads.

Task 87 refines the same row: term-bearing theorem formulas are executable
through the active `type_elaboration` runner only as extraction-gap boundaries.
They do not satisfy the deferred `formula_statement` runner obligation and do
not credit term/formula payloads, term inference, formula checking, facts, proof
skeletons, CoreIr, ControlFlowIr, VC, or proof payloads. In the row above, the
task-86 theorem formula boundary entry now covers both task 86's formula-only
case and task 87's term-bearing equality case.

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
`TypeCaseAttr` bridge, task-85 negative `empty`/builtin-`set` bridge, and
task-80 boundary, imported structures beyond the task-83 `R` bridge and task-97 `TypeCaseStruct` bridge and task-78
boundary, imported mode expansions beyond task 82's provenance/type-head bridge, attribute arguments,
mode/structure arguments, qualified attribute provenance, type-argument, term-argument,
bracket `type_arg_list`, or `qua`-argument
provenance, structure base-shape evidence, terms, formulas, coercion sites,
overload evidence, recorded facts, CoreIr, ControlFlowIr, VC payloads, and proof
evidence remain outside the supported extraction slice. When an active case
needs one of those
unsupported payload families, the runner reports the stable detail key
`type_elaboration.external_dependency.ast_payload_extraction`. Active fail
cases may assert that key through `diagnostic_payloads` or `stable_detail_key`;
active pass cases outside the supported slice remain deferred rather than
passing through a stub. This runner does not publish `CoreIr`, `ControlFlowIr`,
VC seeds, proof rows, or public checker diagnostic codes.

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
