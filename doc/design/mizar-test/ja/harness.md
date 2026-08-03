# Module: harness

## Parser Task 46 operator-declaration parse-only increment

exact pass/fail pairはordinary parse-only runnerへadmitする。pass sidecarはdiagnostic 0、
fail sidecarは既存syntax diagnostic code 6件をpinする。code-only fail sidecarではなく
parser unit testが各slot/delimiter diagnosticのmessage/range、definitionのouter `end;`、
following theoremをpinする。new runner phase、diagnostic vocabulary、production harness
pathは追加しない。

> Canonical language: English. English canonical version: [../en/harness.md](../en/harness.md).

## 目的

この module は test cases を discover し、compiler profiles を run し、expectations を check し、deterministic results を report する test harness を定義する。

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

generic な `TestOutcome` / snapshot reporting surface は future API である。
現在の active runner は stage-specific report record を公開し、上記の metadata
plan と validation diagnostics を共有する。

## Public Enum Forward Compatibility

task 12 は `mizar-frontend` task 25 の手続きを harness-facing enum surface に適用する。
これらの enum は downstream API であり、`#[non_exhaustive]` を維持しなければならない。
downstream caller は wildcard match arm を保つ必要がある。一方、`mizar-test` 内部の
match は現在知られている variant に対して exhaustive のままでよい。

| Public enum | Owner | Decision |
|---|---|---|
| `ValidationSeverity` | harness plan と runner report が使う `diagnostic` reporting | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `TestProfile` | `harness` profile selection | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ValidationMode` | `harness` validation strictness | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `HarnessError` | `harness` infrastructure failure boundary | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ParseOnlyCaseStatus` | `runner` parse-only report status | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `DeclarationSymbolCaseStatus` | `runner` declaration-symbol report status | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `TypeElaborationCaseStatus` | `runner` type-elaboration report status | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ProofVerificationCaseStatus` | `runner` exact proof-verification report status | `#[non_exhaustive]` downstream forward-compatible surface。 |

この module が所有する exhaustive public enum exception はない。

## Runner Modes

| Mode | Behavior |
|---|---|
| metadata plan | payload を実行せずに sidecar を discover し、layout、expectation schema、traceability を validate |
| parse-only | active な `.miz` parse-only case を `mizar-frontend` と `MizarParserSeam` で run |
| declaration-symbol | active な `.miz` declaration-symbol case を frontend parsing と resolver declaration/symbol collection で run |
| type-elaboration | active な `.miz` type-elaboration case を frontend parsing と resolver declaration/symbol collection で run し、対応済み reserve-only declaration payload を抽出し、checker-owned `BindingEnv` / `DeclarationInput` / `DeclarationChecker` handoff production を syntax-free な `mizar-checker` seam に委譲し、successful bare-builtin case、task-55 bare local-mode-expansion case、task-56 one-edge local-mode chain case、task-74 structural bare local-mode chain case は `TypedAst` と `ResolvedTypedAst` まで継続し、`mizar-core` の `ResolvedTypedAstSummary::from_ast` で summary-readiness を確認し、同じ reserve binding から binder-only `CoreContext` input を準備し、same-module attributed reserve declaration、local structure reserve head、attributed local structure reserve head、task-57 の local structure RHS を持つ real local-mode expansion、task-58 の attributed builtin RHS を持つ real local-mode expansion、task-59 の real direct bare-builtin expansion を持つ attributed local-mode reserve head、task-60 の real direct local-structure RHS expansion を持つ attributed local-mode reserve head、task-61 の real direct attributed-builtin RHS expansion を持つ attributed local-mode reserve head、task-62 の local structure RHS で終端する one-edge bare local-mode chain、task-63 の attributed builtin RHS で終端する one-edge bare local-mode chain、task-64 の one-edge bare-builtin chain を持つ attributed local-mode reserve head、task-65 の one-edge structure-RHS chain を持つ attributed local-mode reserve head、task-66 の one-edge attributed-builtin-RHS chain を持つ attributed local-mode reserve head は checker evidence-query gap、narrow な task-55/task-56/task-57/task-58/task-59/task-60/task-61/task-62/task-63/task-64/task-65/task-66/task-74 expansion slice を持たない same-module local mode reserve head（mixed attributed/bare local-mode source、attributed chain dependency、task-74 structural guard violation chain を含む）は checker mode-expansion payload gap として surface し、task-67 structure-qualified attribute reference、task-68 argument-bearing local-mode reserve head、task-69 argument-bearing local-structure reserve head、task-70 bracket-form local-mode reserve head、task-71 bracket-form local-structure reserve head は source-to-checker extraction-gap boundary case として surface し、task-75 forward local-mode reserve head、task-76 forward local-structure reserve head、task-77 forward local-attribute reserve type expression は checker handoff 前の lower-stage active-range boundary case として surface し、未対応 checker payload family は stable external dependency gap として surface する |
| proof-verification | exact Task-180 active proof-verification source だけを source-to-checker-to-Core-to-VC へ2回通し、complete `VcSet` debug baseline を比較する。broader proof-verification family は deferred のまま |
| pass/fail | `.miz` cases を run し expected outcome と match |
| snapshot | canonical snapshot hashes を compare |
| determinism | repeated runs を比較し artifacts、diagnostics、hashes を check |
| parallel-equivalence | sequential and parallel outputs を compare |
| fuzz-regression | minimized fuzz cases を ordinary committed tests として run |
| update | 明示要求された場合のみ snapshots を rewrite |

Core Task 31はexact type-elaboration exceptionを1件追加する。Task-180 checker
handoff成功後、runnerはbundleをCoreIrへ2回lowerし、complete debug bytesをcommitted
baselineとverify-compareする。missing/unreadable/mismatched/absent CoreIr snapshotは
public case statusを`Failed`にし、`snapshot_failure`を設定してinternal diagnostic
code `E-TYPE-ELABORATION-SNAPSHOT`を`type_elaboration.snapshot.<case-id>`でemitする。
ordinary detail-key resultは不変で、他のtype-elaboration caseはこのpathに入らない。

## Runner Source Ownership (Checker Task 250 update)

current production runner layoutは正確に21 path、23,184行である。Checker Task 250は
existing Task-248 source-context leafとTask-249 source-type leafに並べてbounded
source-attribute leafを1件追加し、`runner.rs` facade/top-level
orchestration-only boundaryを維持する。

| Production path | Lines | Ownership |
|---|---:|---|
| `src/runner.rs` | 2,390 | snapshot failureを含むpublic report/status、corpus orchestration、public active iterator、proof-verification orchestration、parse/declaration admission、type-case execution、verify-only baseline comparison、top-level detail dispatch。 |
| `src/runner/shared.rs` | 265 | cross-phase source/frontend/resolver staging、exact internal resolver diagnostic-key projection、resolver shell retentionを含むcommon diagnostic support。 |
| `src/runner/parse_only.rs` | 119 | parse-only case executionとfailure projection。 |
| `src/runner/declaration_symbol.rs` | 231 | declaration-symbol execution、observation、payload、failure projection。 |
| `src/runner/import_fixtures.rs` | 410 | fixture lexical summary、import-summary adapter、source-type authentication用coherent resolver import projection。 |
| `src/runner/proof_verification.rs` | 170 | exact Task-180 admission、source-to-VC execution、deterministic rerun、VcIr snapshot comparison、failure diagnostic。 |
| `src/runner/type_elaboration.rs` | 593 | 正確に14個のprivate leafを持つprivate type-elaboration facade。 |
| `src/runner/type_elaboration/admission.rs` | 60 | active type-case admissionとtag validation。 |
| `src/runner/type_elaboration/binary_routes.rs` | 3,791 | reserved-variable binary route config、extraction、output、detail。 |
| `src/runner/type_elaboration/checker_handoff.rs` | 1,299 | checker-owned binding/declaration、exact Task-180 statement/proof/terminal handoff assembly/validation、legacy empty-later-payload assembly、test-only real-bundle near-miss construction。 |
| `src/runner/type_elaboration/long_chain_config.rs` | 82 | shared exact long-chain definition table。 |
| `src/runner/type_elaboration/output.rs` | 1,571 | checker output、validation、result/detail projection、diagnostic、reusable exact Task-180 CoreIr constructionとdeterministic Core rerun。 |
| `src/runner/type_elaboration/parenthesized_routes.rs` | 745 | parenthesized reserved-variable route ownership。 |
| `src/runner/type_elaboration/result.rs` | 38 | expected-keyとstable detail/snapshot failure projection。 |
| `src/runner/type_elaboration/source_ast.rs` | 147 | common exact AST/import projection。 |
| `src/runner/type_elaboration/source_attribute.rs` | 1,575 | exact Task-250 attribute-chain AST traversal、syntax-free chain/attribute/qualifier/group/actual projection、checker producer invocation、pending-detail isolation。 |
| `src/runner/type_elaboration/source_context.rs` | 592 | exact Task-248 resolver-shell/source-context projection、route isolation、checker producer invocation、immutable handoff assembly、exact 2/2/0 source-type dependency co-installation、Task-250 source-attribute payloadの明示的absence。 |
| `src/runner/type_elaboration/source_formula.rs` | 2,651 | common formula/source payload extraction、exact theorem/formula site/range、explicit Task-268 theorem intent。 |
| `src/runner/type_elaboration/source_reserve.rs` | 1,474 | reserve declaration/type/symbol/mode-expansion extraction。 |
| `src/runner/type_elaboration/source_type.rs` | 794 | exact Task-249 source-type AST traversal、syntax-free 10/13/6 checker input projection、handoff assembly、pending-detail isolation。 |
| `src/runner/type_elaboration/type_assertion_routes.rs` | 4,187 | reserved-variable type-assertion/asserted-head route ownership。 |

hash時は各表示pathに`crates/mizar-test/`をprefixする。repository rootから
`crates/mizar-test/src/runner.rs`と`crates/mizar-test/src/runner`のtracked pathを
選び、`tests.rs`と`tests/`配下を除外してsortした正確なnewline-delimited path-
list hashは
`bd42d60f45e40526a785a6ebcc0df910b99f33a8a8b19371f678070b51bac1d6`、
同じrepository-relative pathを順に`sha256sum`したordered output lineのhashは
`d1421834a7c7613150634735c47aa2700ddf17a7ca2ffebd94f596664ee3a8eb`。
production `runner.rs`はroute config、source extractor、output builder、detail-
wrapper definitionを所有せず、route aliasはtest-onlyである。private facadeの
14 `mod` declaration、21-path/hash pair、documented public API、exact discovered-
test/CLI oracleをownership guardとする。fully qualified name/nestingを変えない
ため、test sourceは`src/runner/tests.rs`、`src/runner/tests/`、既存integration-
test fileに維持する。

Task 75/76/77 addendum for `type-elaboration`: later declaration を名前参照する
forward same-module local-mode reserve head、local-structure reserve head、
local-attribute reserve type expression は active lower-stage boundary case
である。runner は checker handoff 前の
`type_elaboration.lower_stage.frontend:malformed_type_expression` を期待し、
future declaration から checker `ModeExpansion`、structure type-head、
base-shape、constructor-witness、`AttributeInput`、attributed-type evidence
payload を合成してはならない。

Task 78 addendum for `type-elaboration`: task 83 より前は、documented
`parser.type_fixtures` imported structure `R` reserve head が active
source-to-checker extraction-gap boundary case だった。task 83 はその documented
`R` 部分を supersede し、task 97 は documented `TypeCaseStruct` 部分を supersede
する。task-83 `R` と task-97 `TypeCaseStruct` provenance/type-head bridge 外の
broader imported structure は deferred のままとし、将来の case は
`type_elaboration.external_dependency.ast_payload_extraction` を期待する。runner は
summary を real imported module AST extraction と扱ったり、base-shape /
constructor-witness evidence、positive structure elaboration、CoreIr、
ControlFlowIr、VC、proof payload を合成してはならない。

Tasks 83 and 97 addendum for `type-elaboration`: documented
`parser.type_fixtures` imported structure `R` と `TypeCaseStruct` は checker-owned
imported structure type head として渡してよい。runner は
`type_elaboration.checker.checker.declaration.deferred.evidence_query` を期待し、
summary を imported module AST extraction と扱ったり、base-shape /
constructor-witness evidence、positive imported structure elaboration、CoreIr、
ControlFlowIr、VC、proof payload を合成してはならない。

Task 79 addendum for `type-elaboration`: task 82 より前は、documented
`parser.type_fixtures` import summary 由来の imported mode reserve head は
active source-to-checker extraction-gap boundary case だった。task-82
`TypeCaseMode` provenance/type-head bridge 外の imported mode は引き続き
`type_elaboration.external_dependency.ast_payload_extraction` を期待する。runner は
summary を real imported module AST extraction と扱ったり、`ModeExpansion`
payload、positive mode elaboration、CoreIr、ControlFlowIr、VC、proof payload を
合成してはならない。

Task 80 addendum for `type-elaboration`: task 84 / task 85 / task 116 より前は、documented
`parser.type_fixtures` import summary 由来の imported attribute reserve type は
active source-to-checker extraction-gap boundary case だった。task-84
`TypeCaseAttr` provenance / `AttributeInput` bridge、task-85 negative
`empty`/builtin-`set` bridge、task-116 positive `empty`/builtin-`set`
bridge 外の imported attribute は引き続き
`type_elaboration.external_dependency.ast_payload_extraction` を期待する。
runner は summary を real imported module AST extraction と扱ったり、
attributed-type evidence、positive attributed type elaboration、CoreIr、
ControlFlowIr、VC、proof payload を合成してはならない。

Task 84 addendum for `type-elaboration`: documented `parser.type_fixtures`
imported attribute `TypeCaseAttr` は builtin `set` 上の checker-owned imported
`AttributeInput` として渡してよい。runner は
`type_elaboration.checker.checker.declaration.deferred.evidence_query` を期待し、
summary を imported module AST extraction と扱ったり、attributed-type
existential/evidence payload、positive imported attributed type elaboration、
`empty` のような generic imported attribute、structure-qualified attribute owner
provenance、attribute argument、CoreIr、ControlFlowIr、VC、proof payload を合成してはならない。

Task 85 / task 116 / task 171 addendum for `type-elaboration`: documented
`parser.type_fixtures` imported attribute `empty` は、既存 `non empty set` と
`empty set` fixture について builtin `set` 上の checker-owned imported negative /
positive `AttributeInput` として、既存 `non empty object` fixture について builtin
`object` 上の negative `AttributeInput` としてだけ渡してよい。
runner は
`type_elaboration.checker.checker.declaration.deferred.evidence_query` を期待し、
summary を imported module AST extraction と扱ったり、attributed-type
existential/evidence payload、positive `empty object`、symbol head 上の imported
`empty`、positive imported attributed type elaboration、structure-qualified
attribute owner provenance、attribute argument、CoreIr、ControlFlowIr、VC、proof
payload を合成してはならない。

Task 86 / task 115 / task 117 addendum for `type-elaboration`: formula-only theorem
source は active checker boundary case として parser / resolver まで実行してよい。
task 115 は exact unrecovered
`theorem FormulaPayloadBoundary: thesis;` source だけを supersede し、source-derived
`thesis` formula constant site/range を checker recovery `FormulaInput` として渡す。
task 117 はこの recovery marker を real `FormulaKind::Thesis` payload に進め、
`type_elaboration.checker.checker.formula.external.formula_payload` だけを期待する。
non-exact formula-only theorem shape は
`type_elaboration.external_dependency.ast_payload_extraction` に残す。runner は
formula constant semantics、child-formula graph payload、theorem acceptance、
recorded fact、proof skeleton、`formula_statement` execution、CoreIr、ControlFlowIr、
VC、proof payload を合成してはならない。

Task 106 addendum for `type-elaboration`: task-87 の term-bearing builtin
equality theorem source は、exact unrecovered
`TheoremItem -> FormulaExpression -> BuiltinPredicateApplication("=")` shape と
labelled source `theorem TermFormulaPayloadBoundary: 1 = 1;`、および `1` と
綴られる 2 つの structural numeral operand に限って checker term/formula payload
seam まで実行してよい。runner は real module-shell binding context を作り、
source-derived checker `TermInput` / `FormulaInput` payload を渡し、
`type_elaboration.checker.checker.term.external.numeric_type_payload` と
`type_elaboration.checker.checker.formula.term.partial` で fail closed
しなければならない。numeric type payload、equality fact/checking、theorem
acceptance、proof skeleton、`formula_statement` runner support、CoreIr、
ControlFlowIr、VC、proof payload を合成してはならない。

Task 98 addendum for `type-elaboration`: task 98 は `parser.type_fixtures`
由来の imported predicate/functor surface を使う theorem formula source が parser /
resolver まで実行可能な extraction-gap boundary であることを記録した。task 110 は
exact labelled
`ImportedPredicateFunctorPayloadBoundary: 1 divides (1 ++ 2)` sidecar だけを
supersede し、real checker numeral、imported functor-application、
predicate-application payload を作って missing numeric/signature payload と
partial formula checking で fail closed してよい。これは dedicated
`formula_statement` runner obligation、imported module AST extraction、semantic
predicate/functor signature、term inference、formula checking、recorded fact、
theorem acceptance、proof skeleton、CoreIr、ControlFlowIr、VC、proof payload を
昇格しない。

Task 100 addendum for `type-elaboration`: builtin membership theorem source は、
task 108 時点で exact unrecovered
`TheoremItem -> FormulaExpression -> BuiltinPredicateApplication("in")` shape、
labelled source `theorem BuiltinMembershipPayloadBoundary: 1 in 1;`、および
`1` と `1` と綴られる structural numeral operand に限って checker
term/formula payload seam まで実行してよい。runner は real module-shell
binding context を作り、source-derived checker `TermInput` / `FormulaInput`
payload を渡し、
`type_elaboration.checker.checker.term.external.numeric_type_payload` と
`type_elaboration.checker.checker.formula.term.partial` で fail closed
しなければならない。numeric type payload、membership operand expected type、
membership fact、theorem acceptance、proof skeleton、`formula_statement` runner
support、CoreIr、ControlFlowIr、VC、proof payload を合成してはならない。

Task 107 addendum for `type-elaboration`: task-101 の builtin inequality theorem
source は、exact unrecovered
`TheoremItem -> FormulaExpression -> BuiltinPredicateApplication("<>")` shape、
labelled source `theorem BuiltinInequalityPayloadBoundary: 1 <> 2;`、および
`1` と `2` と綴られる structural numeral operand に限って checker
term/formula payload seam まで実行してよい。runner は real module-shell
binding context を作り、source-derived checker `TermInput` / `FormulaInput`
payload を渡し、
`type_elaboration.checker.checker.term.external.numeric_type_payload` と
`type_elaboration.checker.checker.formula.term.partial` で fail closed
しなければならない。numeric type payload、inequality desugaring / equality
checking、fact、theorem acceptance、proof skeleton、`formula_statement` runner
support、CoreIr、ControlFlowIr、VC、proof payload を合成してはならない。

Task 109 addendum for `type-elaboration`: task 102 の exact builtin type-assertion
sidecar `BuiltinTypeAssertionPayloadBoundary: 1 is set` は active
`type_elaboration` runner で parser / resolver まで実行し、source-derived checker
`TermInput`、`FormulaInput`、asserted builtin `set` `TypeExpressionInput` payload
を渡してから missing numeric type payload と partial formula checking で fail
closed してよい。deferred `formula_statement` runner obligation は満たさず、より
広い asserted type payload extraction、type-assertion semantic checking、fact、
theorem acceptance、CoreIr、ControlFlowIr、VC、proof payload は credit しない。

Task 113 addendum for `type-elaboration`: `parser.type_fixtures` を import し、
documented `empty` attribute を
`ImportedAttributeAssertionPayloadBoundary: 1 is empty` で使う exact theorem
formula は imported attribute provenance を検証し、source-derived numeral と
attribute-assertion checker payload を渡し、missing numeric type payload、missing
formula / attribute semantic payload、partial formula checking で fail closed
してよい。runner は imported module AST extraction、attribute-chain semantic
payload、theorem-formula `AttributeInput` payload、attribute checking、theorem
acceptance、`formula_statement`、CoreIr、ControlFlowIr、VC、proof payload を合成しては
ならず、broader imported attribute assertion surface は existing gap に残す。

Task 114 addendum for `type-elaboration`: `parser.type_fixtures` を import し、
documented `empty` attribute を Chapter 14 の attribute-assertion form と Chapter
13 の numeral subject で attribute-level `non empty` assertion として使う exact
theorem formula は task 104 を supersede する。active runner は direct `non`
surface と imported `empty` provenance を検証し、real source-derived checker
term/formula payload を渡してから missing numeric type payload、missing formula /
attribute semantic payload、partial formula checking で fail closed する。runner
は imported module AST extraction、negated attribute-chain semantic payload、
theorem formula 向け checker `AttributeInput` payload、negated attribute
admissibility/semantic checking、theorem acceptance、`formula_statement`、
CoreIr、ControlFlowIr、VC、proof payload を合成してはならない。

Task 111 addendum for `type-elaboration`: exact theorem formula
`SetEnumerationPayloadBoundary: {1, 2} = {1, 2}` は parser / resolver まで実行し、
active runner が 4 つの numeral item term、2 つの set-enumeration term、
builtin equality formula の source-derived checker payload を渡してよい。real
set-enumeration result type、term inference、equality/formula checking、recorded
fact、theorem acceptance、`formula_statement` runner support が存在するまでは、
missing numeric type payload、missing set-enumeration result-type
payload、partial formula checking で fail closed しなければならない。runner は
result payload、theorem acceptance、CoreIr、ControlFlowIr、VC、proof payload を
合成してはならない。Chapter 13 の sethood requirement は enumeration ではなく
set-comprehension generator domain に属する。

Task 112 addendum for `type-elaboration`: Chapter 14 の implication、universal
quantification、negation を使う exact theorem formula は parser / resolver まで
実行し、active runner が implication、quantified formula、negation の
source-derived checker `FormulaInput` shell を渡してよい。child-formula graph
payload、binder/context payload、formula checking、recorded fact、theorem
acceptance、`formula_statement` runner support が存在するまでは missing formula
payload と missing quantifier payload で fail closed しなければならない。task 117
はこの exact source の 2 つの `contradiction` constant だけを
`FormulaKind::Contradiction` payload に進める。runner は formula constant semantic
truth value、child link、binder/context payload、fact、theorem acceptance、
CoreIr、ControlFlowIr、VC、proof payload を合成してはならない。

Task 88 addendum for `type-elaboration`: proof-block theorem source は active
checker boundary case として parser / resolver まで実行してよいが、real proof
skeleton payload extraction、local proof context、formula payload extraction、
recorded fact、theorem acceptance、`formula_statement` runner support が存在するまでは
`type_elaboration.external_dependency.ast_payload_extraction` に留めなければならない。
runner は proof skeleton payload、formula payload、local fact、theorem acceptance、
CoreIr、ControlFlowIr、VC、proof payload を合成してはならない。

Task 89 addendum for `type-elaboration`: statement-level proof justification
を含む theorem proof は active checker boundary case として parser / resolver
まで実行してよいが、real statement proof payload extraction、nested proof skeleton
payload、local proof context、formula payload extraction、label-reference
semantic checking、recorded fact、theorem acceptance、`formula_statement` runner
support が存在するまでは
`type_elaboration.external_dependency.ast_payload_extraction` に留めなければならない。
runner は statement proof payload、proof skeleton payload、formula payload、
local fact、theorem acceptance、CoreIr、ControlFlowIr、VC、proof payload を合成しては
ならない。

## Consumer Runner Pacing

task 10 は consumer crate と runner support を 1 increment ずつ同期する。
prepared increments は実装・検証済みにし、未準備 consumer は placeholder runner
mode、fake active fixture、fabricated coverage を作らず `paced/open` のままにする。

| Consumer task | Stage / runner | mizar-test status | Next condition |
|---|---|---|---|
| `mizar-parser` task 3 | `parse_only` / `parse-only` | prepared/implemented。active `.miz` pass/fail sidecars は `active_parse_only` を使い、tag のない parse-only metadata は planned のまま | general snapshot runner が着地するまで transitional `SurfaceAst` snapshot shortcut を保つ。 |
| `mizar-resolve` task 23 | `declaration_symbol` / `declaration-symbol` | prepared/implemented。active sidecars は `active_declaration_symbol` を使い、public resolver diagnostic-code matching は gate されたまま | resolver diagnostic range が仕様化された後に public diagnostic-code assertions を開く。 |
| `mizar-checker` task 12 plus task 16-20、task 48 source bridge continuation、task 50 attributed reserve evidence-gap bridge、task 51 local mode expansion-gap bridge、task 52 local structure evidence-gap bridge、task 53 attributed local structure evidence-gap bridge、task 54 attributed local mode expansion-gap bridge、task 55 bare local mode expansion bridge、task 56 local-mode expansion chain bridge、task 57 local-mode structure-RHS evidence-gap bridge、task 58 local-mode attributed-builtin-RHS evidence-gap bridge、task 59 attributed local-mode reserve evidence-gap bridge、task 60 attributed local-mode structure-RHS evidence-gap bridge、task 61 attributed local-mode attributed-builtin-RHS evidence-gap bridge、task 62 local-mode structure-RHS chain evidence-gap bridge、task 63 local-mode attributed-builtin-RHS chain evidence-gap bridge、task 64 attributed local-mode bare-builtin chain evidence-gap bridge、task 65 attributed local-mode structure-RHS chain evidence-gap bridge、task 66 attributed local-mode attributed-builtin-RHS chain evidence-gap bridge、task 67 structure-qualified attribute extraction-gap boundary、task 68 argument-bearing mode reserve extraction-gap boundary、task 69 argument-bearing structure reserve extraction-gap boundary、task 70 bracket-form local mode reserve extraction-gap boundary、task 71 bracket-form local structure reserve extraction-gap boundary、task 72 two-edge bare local-mode chain bridge、task 73 three-edge bare local-mode chain bridge、task 74 structural bare local-mode chain bridge、task 75 local-mode forward-reference active-range boundary、task 76 local-structure forward-reference active-range boundary、task 77 local-attribute forward-reference active-range boundary、task 78 imported structure reserve extraction-gap boundary、task 79 imported mode reserve extraction-gap boundary、task 80 imported attribute reserve extraction-gap boundary、task 82 imported mode provenance bridge、task 83 imported structure provenance bridge, task 97 imported TypeCaseStruct provenance bridge、task 84 imported attribute provenance bridge、task 85 imported non-empty attribute provenance bridge、task 116 imported positive empty attribute provenance bridge、task 86 theorem formula extraction-gap boundary、task 115 exact formula statement checker bridge、task 117 formula constant kind checker bridge、task 106 builtin equality term/formula checker bridge、task 110 imported predicate/functor theorem checker bridge、task 108 builtin membership term/formula checker bridge、task 107 builtin inequality term/formula checker bridge、task 109 builtin type assertion term/formula/type checker bridge、task 103 imported attribute assertion formula extraction-gap boundary、task 113 imported attribute assertion checker bridge、task 114 exact attribute-level non-empty imported attribute assertion theorem checker bridge、task 111 exact set-enumeration theorem checker bridge、task 112 exact formula connective/quantifier shell checker bridge、task 88 proof skeleton extraction-gap boundary、task 89 statement proof extraction-gap boundary、task 90 predicate/functor definition extraction-gap boundary, task 91 attribute definition extraction-gap boundary、task 92 mode/structure definition extraction-gap boundary、task 93 proof-local declaration extraction-gap boundary、task 94 proof-local inline definition extraction-gap boundary、task 95 registration block extraction-gap boundary、task 96 redefinition/notation extraction-gap boundary、reserve summary-readiness、binder-only core context follow-up | `type_elaboration` / `type-elaboration` | prepared/implemented。active sidecars は `active_type_elaboration` を使い、lower stages を先に実行し、reserve-only の builtin `set` / `object` declaration を `.miz` AST から syntax-free checker payload に抽出し、`SymbolEnv` にすでに存在する same-module attribute symbol は builtin reserve type payload、same-module local mode reserve head、same-module local structure reserve head に attach してよく、same-module local mode / structure symbol は argument のない reserve head として使ってよい。task 55 はさらに、unique な preceding no-argument same-module mode definition が bare builtin RHS を持ち definition-local context を持たない bare local-mode reserve use だけに real mode expansion を抽出し、task 56 は dependency mode がその accepted task-55 builtin RHS expansion をすでに持つ場合だけ one-edge local-mode chain を抽出し、task 57 は RHS が same-module local structure head である real local-mode expansion を terminal expansion payload として抽出し、task 58 は RHS が attributed builtin head である real local-mode expansion を terminal expansion payload として抽出し、task 59 は同じ mode が bare reserve head としても使われていない場合だけ attributed local-mode reserve head に real direct bare-builtin local-mode expansion を抽出し、task 60 は同じ mode が bare reserve head としても使われていない場合だけ attributed local-mode reserve head に real direct local-structure RHS local-mode expansion を抽出し、task 61 は同じ mode が bare reserve head としても使われていない場合だけ attributed local-mode reserve head に real direct attributed-builtin RHS local-mode expansion を抽出し、task 62 は unique / unrecovered / preceding / no-context source constraint の下で same-module local structure RHS に至る one-edge bare local-mode chain の両方の real expansion を抽出し、task 63 は同じ source constraint と argument-free same-module RHS attributes の下で attributed builtin RHS に至る one-edge bare local-mode chain の両方の real expansion を抽出し、task 64 は root が bare reserve use と mixed でなく dependency が attributed reserve head ではない場合に、bare builtin RHS へ終端する one-edge dependency chain を持つ attributed local-mode reserve head の両方の real expansion を抽出し、task 65 は root が bare reserve use と mixed でなく dependency が attributed reserve head ではなく、structure definition が unique / unrecovered / same-module / source-preceding である場合に、same-module local structure RHS へ終端する one-edge dependency chain を持つ attributed local-mode reserve head の両方の real expansion を抽出し、task 66 は root が bare reserve use と mixed でなく dependency が attributed reserve head ではなく、RHS attributes が argument-free same-module symbol である場合に、attributed builtin RHS へ終端する one-edge dependency chain を持つ attributed local-mode reserve head の両方の real expansion を抽出し、task 67 は structure-qualified attribute reference が parser/resolver executable だが real qualifier と attribute-owner provenance が存在するまで extraction-gap key に残ることを証明し、task 68 は argument-bearing local-mode reserve head が parser/resolver executable だが real type-argument と term-argument provenance が存在するまで extraction-gap key に残ることを証明し、task 69 は argument-bearing local-structure reserve head が parser/resolver executable だが real type-argument と term-argument provenance が存在するまで extraction-gap key に残ることを証明し、task 70 は bracket-form local-mode reserve head が parser/resolver executable だが real bracket type-argument と `qua`-argument provenance が存在するまで extraction-gap key に残ることを証明し、task 71 は bracket-form local-structure reserve head が parser/resolver executable だが real bracket type-argument と `qua`-argument provenance が存在するまで extraction-gap key に残ることを証明し、task 72 は builtin `set` / `object` に終端する real two-edge bare local-mode chain を抽出し、task 73 は builtin `set` / `object` に終端する real three-edge bare local-mode chain を抽出し、task 74 は builtin `set` / `object` に終端する AST-bounded structural bare local-mode chain を抽出し、task 75 は forward same-module local-mode reserve head を checker handoff 前の lower-stage active-range rejection として記録し、task 76 は forward same-module local-structure reserve head を checker handoff 前の同じ lower-stage active-range rejection として記録し、task 77 は forward same-module local-attribute reserve type expression を checker handoff 前の同じ lower-stage active-range rejection として記録し、task 78 は、task 83 がその `R` 部分を supersede する前に documented imported structure `R` reserve head を source-to-checker extraction-gap boundary case として記録し、task 79 は同じ import summary 由来の imported mode reserve head を source-to-checker extraction-gap boundary case として記録し、task 80 は同じ import summary 由来の imported attribute reserve type を source-to-checker extraction-gap boundary case として historical に記録し、task 84 は documented `TypeCaseAttr` 部分を supersede し、task 85 は negative `empty`/builtin-`set` 部分を supersede し、task 116 は positive `empty`/builtin-`set` 部分を supersede する。task 82 は `TypeCaseMode` imported mode summary symbol を checker type-head payload に昇格して checker missing mode-expansion diagnostic で停止し、task 83 は `R` imported structure summary symbol を checker type-head payload に昇格して checker missing structure-evidence query で停止し、task 97 は `TypeCaseStruct` imported structure summary symbol を同じ checker type-head payload に昇格して同じ checker missing structure-evidence query で停止し、task 84 は `TypeCaseAttr` imported attribute summary symbol を checker `AttributeInput` payload に昇格して checker missing attributed-type evidence query で停止し、task 85 は imported `empty` attribute summary symbol を negative `non empty set` だけ checker `AttributeInput` payload に昇格して同じ evidence-query diagnostic で停止し、task 116 は同じ imported `empty` summary symbol を positive `empty set` だけ checker `AttributeInput` payload に昇格して同じ evidence-query diagnostic で停止し、task 86 は formula-only theorem source を parser / resolver まで実行する historical boundary であり、task 117 は exact `FormulaPayloadBoundary: thesis` sidecar を real `FormulaKind::Thesis` checker payload に進めて missing formula payload で fail closed し、task 106 は exact builtin equality theorem source を parser / resolver まで実行し、real checker term/formula payload を渡して missing numeric type payload と partial formula checking で fail closed し、task 110 は task 98 の exact imported predicate/functor theorem formula source を supersede し、real checker numeral、imported functor-application、predicate-application payload を渡して missing numeric/signature payload と partial formula checking で fail closed し、task 108 は exact builtin membership theorem source を parser / resolver まで実行し、real checker term/formula payload を渡して missing numeric type payload と partial formula checking で fail closed し、task 107 は exact builtin inequality theorem source を parser / resolver まで実行し、real checker term/formula payload を渡して missing numeric type payload と partial formula checking で fail closed し、task 109 は exact builtin type-assertion theorem source を parser / resolver まで実行し、real checker term/formula/asserted-type payload を渡して missing numeric type payload と partial formula checking で fail closed し、task 103 は imported attribute assertion theorem formula source を parser / resolver まで実行する historical boundary として残し、task 113 は exact imported empty attribute assertion theorem source を parser / resolver まで実行して real checker term/formula payload を渡し、missing numeric type payload、missing formula/attribute semantic payload、partial formula checking で fail closed し、task 114 は exact attribute-level non-empty imported attribute assertion theorem formula source について task 104 を supersede し、real checker term/formula payload を渡して missing numeric type payload、missing formula/attribute semantic payload、partial formula checking で fail closed し、task 111 は exact set-enumeration theorem source を parser / resolver まで実行し、real checker term/formula payload を渡して missing numeric/result-type payload と partial formula checking で fail closed、task 112 は exact connective/quantifier theorem formula source を parser / resolver まで実行し、real checker formula shell payload を渡して missing formula/quantifier payload で fail closed し、task 88 は proof-block theorem source を parser / resolver まで実行するが checker source-to-payload extraction gap に留め、task 89 は statement-level proof-justification theorem source を parser / resolver まで実行するが checker source-to-payload extraction gap に留め、task 90 は predicate/functor definition source を parser / resolver まで実行するが checker source-to-payload extraction gap に留め、task 91 は attribute definition source を parser / resolver まで実行するが同じ extraction gap に留め、task 92 は mode/structure definition source を parser / resolver まで実行するが同じ extraction gap に留め、task 93 は proof-local declaration statement を parser / resolver まで実行するが同じ extraction gap に留め、task 94 は proof-local inline definition を parser / resolver まで実行するが同じ extraction gap に留め、task 95 は top-level registration block を parser / resolver まで実行するが同じ extraction gap に留め、task 96 は redefinition/notation surface を parser / resolver まで実行するが同じ extraction gap に留める。`mizar-checker` は checker-owned `BindingEnv`、binding ごとの `DeclarationInput`、binding 固有の `TypeExpressionInput` site、`DeclarationChecker` output を生成し、successful bare-builtin case、task-55 bare local-mode case、task-56 chain case、task-74 structural bare-chain case は `TypedAst`、checker-owned `ResolvedTypedAst`、`mizar-core` `ResolvedTypedAstSummary::from_ast` read、binder-only `CoreContext` preparation へ継続し、attributed reserve case、local-structure case、task-57 structure-RHS expansion case、task-58 attributed-RHS expansion case、task-59 attributed local-mode expansion case、task-60 attributed local-mode structure-RHS expansion case、task-61 attributed local-mode attributed-RHS expansion case、task-62 local-mode structure-RHS chain expansion case、task-63 local-mode attributed-RHS chain expansion case、task-64 attributed local-mode bare-builtin chain expansion case、task-65 attributed local-mode structure-RHS chain expansion case、task-66 attributed local-mode attributed-RHS chain expansion case は checker `MissingEvidenceQuery` diagnostic、task 55/56/57/58/59/60/61/62/63/64/65/66/74 外の local-mode case（mixed attributed/bare local-mode source、attributed chain dependency、task-74 structural guard violation chain を含む）は missing mode-expansion diagnostic で停止する。task-67 structure-qualified attribute case、task-68 argument-bearing mode case、task-69 argument-bearing structure case、task-70 bracket-form mode case、task-71 bracket-form structure case、broader imported-structure case（task-83 `R` と task-97 `TypeCaseStruct` provenance/type-head bridge 外）は deferred に残し、broader imported-attribute case（task-84 `TypeCaseAttr` bridge、task-85 negative `empty`/builtin-`set` bridge、task-116 positive `empty`/builtin-`set` bridge 外）と未対応 checker payload family は `type_elaboration.external_dependency.ast_payload_extraction` に残し、task-82 `TypeCaseMode` imported mode case は `type_elaboration.checker.checker.type.external.mode_expansion_payload` に到達し、task-83 `R` imported structure case、task-97 `TypeCaseStruct` imported structure case、task-84 `TypeCaseAttr` imported attribute case、task-85 negative `empty`/builtin-`set` imported attribute case は `type_elaboration.checker.checker.declaration.deferred.evidence_query` に到達する。task-75 forward local-mode reserve head、task-76 forward local-structure reserve head、task-77 forward local-attribute reserve type expression は checker handoff 前の `type_elaboration.lower_stage.frontend:malformed_type_expression` に残し、task-117 exact formula statement checker output は formula constant semantic truth value、child-formula graph payload、fact、theorem acceptance、proof skeleton、CoreIr、ControlFlowIr、VC、proof payload、`formula_statement` runner activation をまだ持たず、non-exact task-86 formula-only variants は `type_elaboration.external_dependency.ast_payload_extraction` に残し、task-106 builtin equality theorem formula は numeric type payload、equality checking、fact、proof skeleton、CoreIr、ControlFlowIr、VC、proof payload、`formula_statement` runner activation なしに checker diagnostics で fail closed し、task-109 builtin type-assertion theorem formula は numeric type payload、broader asserted type payload、type-assertion semantic checking、fact、theorem acceptance、proof skeleton、CoreIr、ControlFlowIr、VC、proof payload、`formula_statement` runner activation なしに checker diagnostics で fail closed し、task-103 historical imported attribute assertion theorem boundary cases outside task 113 は term/formula payload、imported attribute assertion attribute-chain/provenance payload extraction、term inference、attribute admissibility/semantic checking、formula checking、fact、theorem acceptance、imported module AST extraction、theorem formula 向け checker `AttributeInput` payload extraction、proof skeleton、CoreIr、ControlFlowIr、VC、proof payload、`formula_statement` runner activation なしに同じ extraction gap に残し、task-88 proof-block theorem source は proof skeleton payload、local proof context、formula payload、fact、theorem acceptance、CoreIr、ControlFlowIr、VC、proof payload、`formula_statement` runner activation なしに同じ extraction gap に残し、task-89 statement-proof theorem source は statement proof payload、nested proof skeleton payload、local proof context、formula payload、label-reference semantic checking、fact、theorem acceptance、CoreIr、ControlFlowIr、VC、proof payload、`formula_statement` runner activation なしに同じ extraction gap に残し、task-91 attribute definition source は definition declaration payload、definition-local context、formula-definiens payload、attributed-type evidence、fact、CoreIr、ControlFlowIr、VC、proof payload、`formula_statement` runner activation なしに同じ extraction gap に残し、task-92 mode/structure definition source は definition declaration payload、mode expansion、structure base-shape / constructor / selector evidence、definition-local context、fact、CoreIr、ControlFlowIr、VC、proof payload、`formula_statement` runner activation なしに同じ extraction gap に残し、task-93 proof-local declaration statement source は proof-local declaration payload、local proof context、formula/term payload、RHS term inference、reconsider coercion / obligation evidence、fact、theorem acceptance、CoreIr、ControlFlowIr、VC、proof payload、`formula_statement` runner activation なしに同じ extraction gap に残し、task-94 proof-local inline definition source は inline definition formal/body payload、local abbreviation expansion、term / formula body payload、guard evidence、fact、theorem acceptance、CoreIr、ControlFlowIr、VC、proof payload、`formula_statement` runner activation なしに同じ extraction gap に残し、task-95 registration-block source は registration-item payload、correctness-condition / proof-obligation payload、accepted activation / evidence status、cluster / reduction semantics、Chapter 17 semantic row、fact、CoreIr、ControlFlowIr、VC、proof payload、`formula_statement` / `advanced_semantics` runner activation なしに同じ extraction gap に残し、task-96 redefinition/notation source は redefinition payload、notation alias relation payload、redefinition target inference、coherence proof-obligation payload、overload candidate payload、Chapter 11 alias semantic resolution、Chapter 19 overload/redefinition semantics、fact、CoreIr、ControlFlowIr、VC、proof payload、`formula_statement` / `advanced_semantics` runner activation なしに同じ extraction gap に残す | より広い type/formula pass/fail semantic assertions は、task-55 bare builtin RHS slice、task-56 one-edge chain slice、task-57 structure-RHS diagnostic slice、task-58 attributed-RHS diagnostic slice、task-59 attributed local-mode reserve diagnostic slice、task-60 attributed local-mode structure-RHS diagnostic slice、task-61 attributed local-mode attributed-RHS diagnostic slice、task-62 local-mode structure-RHS chain diagnostic slice、task-63 local-mode attributed-RHS chain diagnostic slice、task-64 attributed local-mode bare-builtin chain diagnostic slice、task-65 attributed local-mode structure-RHS chain diagnostic slice、task-66 attributed local-mode attributed-builtin-RHS chain diagnostic slice、task-67 extraction-gap boundary slice、task-68 extraction-gap boundary slice、task-69 extraction-gap boundary slice、task-70 extraction-gap boundary slice、task-71 extraction-gap boundary slice、task-72 two-edge bare local-mode pass slice、task-74 structural bare local-mode pass slice、task-78 historical extraction-gap boundary slice、task-79 extraction-gap boundary slice、task-80 historical extraction-gap boundary slice、task-82 TypeCaseMode imported-mode provenance/type-head bridge、task-83 `R` imported-structure provenance/type-head bridge、task-97 `TypeCaseStruct` imported-structure provenance/type-head bridge、task-84 `TypeCaseAttr` imported-attribute provenance bridge、task-85 negative `empty`/builtin-`set` provenance bridge、task-116 positive `empty`/builtin-`set` provenance bridge、task-106 builtin equality theorem checker bridge、task-107 builtin inequality theorem checker bridge、task-108 builtin membership theorem checker bridge、task-109 builtin type assertion theorem checker bridge、task-110 checker bridge、task-112/task-117 formula-shell checker bridge、task-113 imported attribute assertion checker bridge、task-114 exact attribute-level non-empty imported attribute assertion checker bridge、task-117 exact formula statement checker bridge、および task-103/task-105 theorem/formula extraction-gap boundary slices、task-88 proof-skeleton extraction-gap boundary slice、task-89 statement-proof extraction-gap boundary slice、task-92 mode/structure definition extraction-gap boundary slice、task-93 proof-local declaration extraction-gap boundary slice、task-94 proof-local inline definition extraction-gap boundary slice、task-95 registration block extraction-gap boundary slice、task-96 redefinition/notation extraction-gap boundary slice、task-75/task-76/task-77 active-range boundary slice を超える AST-wide source-to-checker payload extraction と real existential / evidence-query / mode-expansion / base-shape / imported-structure / imported-attribute / qualified-attribute / type-argument / term-argument / bracket-argument / theorem-formula / proof-context provenance input を待つ。 |
| `mizar-checker` task 29 | `formula_statement` / `advanced_semantics` | paced/open。trace rows は deferred であり、active fixture は捏造しない | statement/formula と advanced-semantics source payload seams が存在した後に runner support を追加する。 |
| `mizar-vc` task 15 plus task 31 exact exception | `proof_verification` / `proof-verification` | exact Task-180 source-to-checker-to-Core-to-VC generation と full snapshot comparison は実装済み。broader VC/proof-verification family は paced/open のまま | exact source/Core/VC payload contract、owning task authority、consumer readiness が揃った phase-11 generation route だけを activate し、downstream proof verification は別途 deferred に保つ。 |
| `mizar-atp` task 20 | `advanced_semantics` metadata handoff | `mizar-test` では paced/open。metadata-only property fixtures は `mizar-atp` Rust tests が消費してよい | source-derived ATP extraction と proof-policy/kernel handoff seams が存在した後に active `.miz` ATP runner support を追加する。 |
| `mizar-kernel` task 17 | proof/certificate/kernel evidence | paced/open。fail/soundness metadata は active proof/certificate/kernel execution なしで検証する | source-to-evidence または certificate execution seams が存在した後に runner support を追加する。 |

task 85、task 116、task 171 は上の `type_elaboration` consumer row を refine
する。imported attribute gap list から除外されるのは task 84 の
`TypeCaseAttr` fixture に加えて、documented negative / positive `empty` over
builtin `set` fixtures と exact negative `empty` over builtin `object` fixture
である。positive `empty object`、symbol head 上の imported attribute、generic
imported attribute、imported module AST extraction、argument、owner provenance、
evidence payload、CoreIr、ControlFlowIr、VC、proof row は supported slice 外に残す。

task 86 も同じ row を refine し、formula-only theorem source が active
`type_elaboration` runner を通じて実行可能であることを示す。task 117 は task 115
の exact `FormulaPayloadBoundary: thesis` recovery marker を supersede し、
source-derived `thesis` formula constant を real `FormulaKind::Thesis` checker
payload として渡してから、missing formula payload で fail closed する。
deferred `formula_statement` runner obligation を満たさず、formula constant semantic
checking、child-formula graph payload、fact、theorem acceptance、proof skeleton、
CoreIr、ControlFlowIr、VC、proof payload は credit しない。

task 87 は当初、term-bearing theorem formula が active `type_elaboration`
runner を通じて parser / resolver executable な extraction-gap boundary であることを
示した。task 106 は exact labelled `TermFormulaPayloadBoundary: 1 = 1` portion
だけを supersede し、real checker term/formula payload を構築して missing numeric
type payload と partial formula checking で fail closed する。これは deferred
`formula_statement` runner obligation を満たさず、numeric type payload、equality
semantic checking、fact、proof skeleton、CoreIr、ControlFlowIr、VC、proof payload
を credit しない。

task 98 は同じ row で imported predicate/functor theorem formula が parser /
resolver executable な extraction-gap boundary であることを記録した。task 110 は
exact labelled `ImportedPredicateFunctorPayloadBoundary: 1 divides (1 ++ 2)`
sidecar だけを supersede し、real checker numeral、imported
functor-application、predicate-application payload を構築して missing
numeric/signature payload と partial formula checking で fail closed する。これは
deferred `formula_statement` runner obligation を満たさず、imported module AST
extraction、semantic predicate/functor signature、term inference、formula
checking、fact、proof skeleton、CoreIr、ControlFlowIr、VC、proof payload は credit
しない。上の theorem formula boundary entry は task 110 の exact imported
predicate/functor checker bridge を含む。

task 100 は当初、builtin membership theorem formula が parser / resolver
executable な extraction-gap boundary であることを示した。task 108 は exact
labelled `BuiltinMembershipPayloadBoundary: 1 in 1` sidecar だけを supersede し、
real checker term/formula payload を構築して missing numeric type payload と
partial formula checking で fail closed する。これは deferred
`formula_statement` runner obligation を満たさず、membership operand
expected-type construction/checking、fact、theorem acceptance、CoreIr、
ControlFlowIr、VC、proof payload は credit しない。上の theorem formula boundary
entry は task 108 の exact builtin membership checker bridge も含む。

task 101 は当初、builtin inequality theorem formula が parser / resolver
executable な extraction-gap boundary であることを示した。task 107 は exact
labelled `BuiltinInequalityPayloadBoundary: 1 <> 2` portion だけを supersede し、
real checker term/formula payload を構築して missing numeric type payload と
partial formula checking で fail closed する。これは deferred
`formula_statement` runner obligation を満たさず、inequality desugaring /
equality semantic checking、fact、theorem acceptance、CoreIr、ControlFlowIr、VC、
proof payload を credit しない。

task 118 は task 106、107、108 が共有する builtin-binary theorem producer を
厳密化する。exact checker handoff は direct theorem token が
`theorem <label> : ;` である場合に限定され、status-prefixed または extra-token
theorem shape は `type_elaboration.external_dependency.ast_payload_extraction`
に残る。これは guard repair のみであり、active sidecar や traceability coverage
は追加しない。

task 119 は exact な diagnostic なしの `type-elaboration` pass case
`reserve x for set; theorem ReservedVariableEqualityPayloadBoundary: x = x;`
を追加する。runner は 2 つの identifier term を source 順から別々に導出した
use ordinal で real reserve `BindingEnv` に問い合わせ、4 つの distinct
source-anchored result/expected type role site を保持し、2 つの `Inferred` term と
1 つの `Checked` equality、empty candidate/diagnostic/deferred/fact を要求する。
production runner validation は exact binding/reference identity、すべての role
owner、normalized type の source range/spelling/head を検証し、不一致なら
`type_elaboration.checker.reserved_variable_equality.invalid_payload` を報告する。
runner unit test は active sidecar を discover し、hand-built syntax tree だけでなく
real frontend/resolver が生成した AST について同じ payload assertion を反復する。
この pass result は theorem acceptance ではなく、implicit closure、
`formula_statement`、proof、CoreIr、ControlFlowIr、VC consumer を activate
しない。non-exact shape は extraction-gap key を報告し続ける。

task 123 は exact distinct-binding equality pass case
`reserve x, y for set; theorem DistinctReservedVariableEqualityPayloadBoundary: x = y;`
を追加する。active runner は real two-binding reserve handoff と共有された記述上の
builtin `set` range を保持し、両 source binding の後に lookup ordinal 2 と 3 を
導出して、operand を distinct checker binding id に解決する。operand ごとの
result/expected role は 2 `Inferred` variable と 1 fact-free `Checked` equality
へ到達する。task-specific invalid-payload key、near-miss matrix、real
frontend/resolver active-sidecar test が exact seam を検証する。separate reserve
item、reversed/identical operand、wrong label/operator/type、extra binding/item、
status/recovery、numeral は extraction gap に残る。この pass は implicit
closure/order、equality truth/fact、theorem acceptance、`formula_statement`、proof、
CoreIr、ControlFlowIr、VC を credit しない。

task 124 は exact multiple-reserve-declaration equality pass case
`reserve x for set; reserve y for set; theorem MultipleReserveDeclarationEqualityPayloadBoundary: x = y;`
を追加する。runner は exact 2 ordered reserve item だけを受理し、`BindingId(0)` と
`BindingId(1)` を保持し、各 declaration の distinct written builtin `set` range を
対応する operand の result/expected pre-normalization input に保持する。real checker
は semantically equal input を、最初の written range を canonical source とする
1 normalized type に intern する。production validation はその semantic identity に
依存する前に 4 original input を検証する。専用 invalid-payload key、near-miss
matrix、real frontend/resolver active-sidecar test が exact seam を guard する。
shared multi-name segment、reversed directive/operand、mixed/extra declaration、wrong
operator、status/recovery、extra theorem、numeral operand は extraction gap に残る。
この pass は implicit closure/order、equality truth/fact、theorem acceptance、
`formula_statement`、proof、CoreIr、ControlFlowIr、VC を credit しない。

task 125 は exact heterogeneous-reserve membership pass case
`reserve x for object; reserve y for set; theorem HeterogeneousReserveMembershipPayloadBoundary: x in y;`
を追加する。runner は exactly two ordered reserve item だけを受理し、`x` を real
builtin-`object` binding、`y` を real builtin-`set` binding として保持し、2 written
range を左 result、右 result、唯一の右 expected input に保持する。production
validation は 2 normalized identity を要求する。右 result/expected role は `set` を
共有し、左 `object` identity は distinct のままで、両 identity は deterministic
source representative を保持する。task-specific invalid key、exact near-miss matrix、
real frontend/resolver active-sidecar test が seam を guard する。non-exact
type/order/operand/operator、extra declaration、status/recovery、numeral operand は
extraction gap に残る。この pass は membership truth/fact、object/set coercion、
implicit closure/order、theorem acceptance、`formula_statement`、proof、CoreIr、
ControlFlowIr、VC を credit しない。

task 126 は exact direct-local-mode reserved-variable equality pass を追加する。
runner は task-55-compatible mode definition 1 個を受理し、4 raw
`LocalModeFormula` input を保持して、その real AST-derived bare-set expansion を
`TermFormulaChecker` に渡す。全 role は 1 builtin-set identity に normalize される。
invalid key、withheld-mode near miss、real sidecar が slice を guard する。mode
declaration acceptance/inhabitation、broader mode、closure/order、fact/truth、
theorem acceptance、proof、CoreIr、ControlFlowIr、VC は deferred のままである。

task 127 は exact one-edge local-mode-chain reserved-variable equality pass を
追加する。runner は exact source-preceding definition block 2 個を受理し、4 raw
outer `ChainModeFormula` input を保持して task-56 real expansion 2 個を
`TermFormulaChecker` に渡す。recursive normalization は terminal `set` RHS に
anchor された 1 builtin-set identity を生成する。invalid-link key、exact chain
guard、withheld-family near miss、real sidecar が slice を guard する。mode
declaration acceptance/inhabitation、object terminal、longer-chain formula、
closure/order、fact/truth、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は
deferred のままである。

task 130 は exact direct local-mode inequality pass を追加する。runner は 4 raw
`LocalModeInequality` input を保持し、real direct expansion を
`TermFormulaChecker` に渡して RHS 起点の builtin-set identity 1 個と fact-free
pre-desugaring `Checked` inequality を要求する。exact guard、corruption coverage、
real sidecar が slice を保護し、downstream semantics は deferred のままである。

task 131 は exact direct local-object-mode inequality pass を追加する。runner は
4 raw `LocalObjectModeInequality` input を保持し、real direct expansion を
`TermFormulaChecker` に渡して RHS 起点の builtin-object identity 1 個と fact-free
pre-desugaring `Checked` inequality を要求する。exact guard、present/missing
expansion corruption coverage、real sidecar が slice を保護し、downstream
semantics は deferred のままである。

task 132 は exact one-edge set-terminal local-mode-chain inequality pass を追加する。
runner は 4 raw `ChainModeInequality` input を保持し、task-56-compatible real
expansion 2 本を `TermFormulaChecker` に渡して terminal `set` RHS 起点の
builtin-set identity 1 個と fact-free pre-desugaring `Checked` inequality を
要求する。exact chain guard、missing-link corruption、withheld-family near miss、
real sidecar が slice を保護し、declaration acceptance/inhabitation、desugaring、
closure/order、theorem/proof/Core/VC、broader semantics は deferred のままである。

task 133 は exact one-edge object-terminal local-mode-chain inequality pass を
追加する。runner は 4 raw `ChainObjectModeInequality` input を保持し、real
expansion 2 本を `TermFormulaChecker` に渡して terminal `object` RHS 起点の
builtin-object identity 1 個と fact-free pre-desugaring `Checked` inequality を
要求する。exact chain guard、missing-link corruption、withheld-family near miss、
real sidecar が slice を保護し、declaration acceptance/inhabitation、desugaring、
closure/order、truth/fact、theorem/proof/Core/VC、broader semantics は deferred のままである。

task 134 は exact two-edge set-terminal local-mode-chain equality pass を追加する。
runner は 4 raw `OuterTwoEdgeModeEquality` input を保持し、Task-72-compatible
real expansion 3 本を `TermFormulaChecker` に渡して terminal `set` RHS 起点の
builtin-set identity 1 個と fact-free `Checked` equality を要求する。exact chain
guard、missing-link corruption、withheld-family near miss、real sidecar が slice を
保護し、declaration acceptance/inhabitation、implicit closure/order、
theorem/proof/Core/VC、broader semantics は deferred のままである。

task 135 は exact two-edge object-terminal local-mode-chain equality pass を
追加する。runner は 4 raw `OuterTwoEdgeObjectModeEquality` input を保持し、
Task-72-compatible real expansion 3 本を `TermFormulaChecker` に渡して terminal
`object` RHS 起点の builtin-object identity 1 個と fact-free `Checked` equality を
要求する。exact chain guard、missing-link corruption、withheld-family near miss、
real sidecar が slice を保護し、declaration acceptance/inhabitation、implicit
closure/order、theorem/proof/Core/VC、broader semantics は deferred のままである。

task 136 は exact two-edge set-terminal local-mode-chain inequality pass を
追加する。runner は 4 raw `OuterTwoEdgeModeInequality` input を保持し、
Task-72-compatible real expansion 3 本を `TermFormulaChecker` に渡して terminal
`set` RHS 起点の builtin-set identity 1 個と fact-free pre-desugaring `Checked`
inequality を要求する。exact chain guard、missing-link corruption、withheld-family
near miss、real sidecar が slice を保護し、mode declaration
acceptance/inhabitation、inequality desugaring、implicit closure/order、
theorem/proof/Core/VC、broader semantics は deferred のままである。

task 137 は exact two-edge object-terminal local-mode-chain inequality pass を
追加する。runner は 4 raw `OuterTwoEdgeObjectModeInequality` input を保持し、
Task-72-compatible real expansion 3 本を `TermFormulaChecker` に渡して terminal
`object` RHS 起点の builtin-object identity 1 個と fact-free pre-desugaring
`Checked` inequality を要求する。exact chain guard、missing-link corruption、
withheld-family near miss、real sidecar が slice を保護し、declaration
acceptance/inhabitation、inequality desugaring、implicit closure/order、
theorem/proof/Core/VC、broader semantics は deferred のままである。

task 138 は exact direct set-terminal local-mode reserved-variable type assertion
pass を追加する。runner は raw `LocalModeTypeAssertion` subject input と独立した
formula-side builtin-set asserted input を保持し、Task-55-compatible real
expansion 1 本を `TermFormulaChecker` に渡して、terminal-RHS builtin-set identity
1 個、`BindingId(0)`、1 `Inferred` term、1 fact-free `Checked` type assertion を
要求する。exact source guard、missing-expansion corruption、withheld-family near
miss、real sidecar が slice を保護し、mode declaration
acceptance/inhabitation、formula-side local-mode asserted head、general
reachability/widening/`qua`、theorem/proof/Core/VC、broader semantics は deferred
のままである。task 139 より前の active type-elaboration runner は 89 件である。

task 139 は exact direct set-terminal local-mode left reserved-variable
membership pass を追加する。runner は raw `LocalModeMembership` left result と
独立した explicit-set right result/expected input を保持し、Task-55-compatible
real expansion 1 本を `TermFormulaChecker` に渡して、terminal-RHS builtin-set
identity 1 個、`BindingId(0/1)`、2 `Inferred` term、1 fact-free `Checked`
membership、right expected constraint だけ、left expected input なしを要求する。
exact source guard、独立した expansion/right-expected corruption、withheld-family
near miss、real sidecar が slice を保護し、mode declaration
acceptance/inhabitation、membership truth/fact、implicit closure/order、
theorem/proof/Core/VC、broader semantics は deferred のままである。active
type-elaboration runner は 90 件である。

task 140 は exact direct object-terminal local-mode left reserved-variable
membership pass を追加する。runner は raw `LocalObjectModeMembership` left
result と独立した explicit-set right result/expected input を保持し、
Task-55-compatible real expansion 1 本を `TermFormulaChecker` に渡して、distinct
terminal-RHS builtin-object / explicit-reserve builtin-set identity、
`BindingId(0/1)`、2 `Inferred` term、1 fact-free `Checked` membership、right
expected constraint だけ、left expected input なしを要求する。exact source
guard、独立した expansion/right-expected corruption、withheld-family near miss、
real sidecar が slice を保護し、mode declaration acceptance/inhabitation、
membership truth/fact、object/set coercion、implicit closure/order、
theorem/proof/Core/VC、broader semantics は deferred のままである。active
type-elaboration runner は 91 件である。

task 141 は exact one-edge set-terminal local-mode-chain left reserved-variable
membership pass を追加する。runner は raw `ChainModeMembership` left result と
独立した explicit-set right result/expected input を保持し、Task-56-compatible
real expansion 2 本を `TermFormulaChecker` に渡して、terminal-RHS builtin-set
identity 1 個、`BindingId(0/1)`、2 `Inferred` term、1 fact-free `Checked`
membership、right expected constraint だけ、left expected input なしを要求する。
exact source guard、独立した chain-link/right-expected corruption、
withheld-family near miss、real sidecar が slice を保護し、mode declaration
acceptance/inhabitation、membership truth/fact、implicit closure/order、
theorem/proof/Core/VC、broader semantics は deferred のままである。active
type-elaboration runner は 92 件である。

task 142 は exact one-edge object-terminal local-mode-chain left reserved-
variable membership pass を追加する。runner は raw
`ChainObjectModeMembership` left result と独立した explicit-set right
result/expected input を保持し、Task-56-compatible real expansion 2 本を
`TermFormulaChecker` に渡して、distinct terminal-RHS builtin-object / explicit-
reserve builtin-set identity、`BindingId(0/1)`、2 `Inferred` term、1 fact-free
`Checked` membership、right expected constraint だけ、left expected input なし
を要求する。exact source guard、独立した chain-link/right-expected corruption、
withheld-family near miss、real sidecar が slice を保護し、mode declaration
acceptance/inhabitation、membership truth/fact、object/set coercion、implicit
closure/order、theorem/proof/Core/VC、broader semantics は deferred のままで
ある。active type-elaboration runner は 93 件である。

task 128 は exact direct local-object-mode reserved-variable equality pass を
追加する。runner は task-55-compatible `LocalObjectMode -> object` definition 1 個を
受理し、4 raw local object-mode input を保持して、その real AST-derived expansion
を `TermFormulaChecker` に渡す。全 role は real `object` RHS に anchor された 1
builtin-object identity に normalize される。invalid key、exact block/label guard、
withheld-family near miss、real sidecar が slice を guard する。mode declaration
acceptance/inhabitation、broader object-mode formula、closure/order、fact/truth、
theorem acceptance、proof、CoreIr、ControlFlowIr、VC は deferred のままである。

task 129 は exact one-edge local-object-mode-chain equality pass を追加する。runner
は 4 raw `ChainObjectMode` input を保持し、task-56 real expansion 2 個を
`TermFormulaChecker` に渡して terminal `object` RHS に 1 builtin-object identity を
anchor する。invalid-link corruption、withheld-family near miss、real sidecar が
exact slice を guard する。declaration acceptance/inhabitation、longer chain、
closure/order、fact/truth、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は
deferred のままである。

task 143 は exact two-edge set-terminal local-mode-chain left reserved-variable
membership pass を追加する。runner は raw `OuterTwoEdgeModeMembership` left
result と独立した explicit-set right result/expected input を保持し、real Task 72
compatible expansion 3 本を `TermFormulaChecker` に渡し、terminal-RHS
builtin-set identity 1 個、`BindingId(0/1)`、2 `Inferred` term、1 fact-free
`Checked` membership、right expected constraint だけ、left expected input なしを
要求する。exact source guard、独立した 3 link/right-expected corruption、
withheld-family near miss、real sidecar が slice を保護する。mode declaration
acceptance/inhabitation、membership truth/fact、implicit closure/order、
theorem/proof/Core/VC、broader semantics は deferred のままである。active
type-elaboration runner は 94 cases である。

task 144 は exact two-edge object-terminal local-mode-chain left reserved-
variable membership pass を追加する。runner は raw
`OuterTwoEdgeObjectModeMembership` left result と独立した explicit-set right
result/expected input を保持し、real Task 72 compatible expansion 3 本を
`TermFormulaChecker` に渡し、distinct terminal-object-RHS builtin-object /
explicit-reserve builtin-set identity、`BindingId(0/1)`、2 `Inferred` term、1
fact-free `Checked` membership、right expected constraint だけ、left expected
input なし、object/set coercion なしを要求する。exact source guard、独立した
3 link/right-expected corruption、withheld-family near miss、real sidecar が
slice を保護する。mode declaration acceptance/inhabitation、membership
truth/fact、implicit closure/order、theorem/proof/Core/VC、broader semantics は
deferred のままである。active type-elaboration runner は 95 cases である。

task 145 は exact direct object-terminal local-mode reserved-variable
normalized-reflexive type assertion pass を追加する。runner は raw
`LocalObjectModeTypeAssertion` subject result と独立した formula-side
builtin-object asserted source を保持し、real Task 55 compatible expansion 1
本を `TermFormulaChecker` に渡し、terminal-RHS-anchored builtin-object identity
1 個、`BindingId(0)`、source-order use ordinal 1、1 `Inferred` term、1
fact-free `Checked` type assertion を general reachability / object-set coercion
なしで要求する。exact source guard、独立した definition/expansion
corruption、withheld-family near miss、real frontend/resolver sidecar が slice
を保護する。mode declaration acceptance/inhabitation、formula-side local-mode
asserted head、general reachability/widening/`qua`、truth/fact、closure/order、
theorem/proof/Core/VC、broader semantics は deferred のままである。active
type-elaboration runner は 96 cases である。

task 146 は exact one-edge set-terminal local-mode-chain reserved-variable
normalized-reflexive type assertion pass を追加する。runner は raw
`ChainModeTypeAssertion` subject result と独立した formula-side builtin-set
asserted source を保持し、real Task 56 compatible expansion 2 本を
`TermFormulaChecker` に渡し、terminal-RHS-anchored builtin-set identity 1 個、
`BindingId(0)`、source-order use ordinal 1、1 `Inferred` term、1 fact-free
`Checked` type assertion を general reachability なしで要求する。exact source
guard、独立した definition/two-link corruption、withheld-family near miss、real
frontend/resolver sidecar が slice を保護する。mode declaration acceptance/
inhabitation、formula-side local-mode asserted head、general reachability/
widening/`qua`、truth/fact、closure/order、theorem/proof/Core/VC、broader
semantics は deferred のままである。active type-elaboration runner は 97
cases である。

task 147 は exact one-edge object-terminal local-mode-chain reserved-variable
normalized-reflexive type assertion pass を追加する。runner は raw
`ChainObjectModeTypeAssertion` subject result と独立した formula-side builtin-
object asserted source を保持し、real Task 56 compatible expansion 2 本を
`TermFormulaChecker` に渡し、terminal-RHS-anchored builtin-object identity 1
個、`BindingId(0)`、source-order use ordinal 1、1 `Inferred` term、1 fact-free
`Checked` type assertion を general reachability / object-set coercion なしで
要求する。exact source guard、独立した definition/two-link corruption、
withheld-family near miss、real frontend/resolver sidecar が slice を保護する。
mode declaration acceptance/inhabitation、formula-side local-mode asserted
head、general reachability/widening/`qua`、truth/fact、closure/order、theorem/
proof/Core/VC、broader semantics は deferred のままである。active type-
elaboration runner は 98 cases である。

task 148 は exact two-edge set-terminal local-mode-chain reserved-variable
normalized-reflexive type assertion pass を追加する。runner は raw
`OuterTwoEdgeModeTypeAssertion` subject result と独立した formula-side
builtin-set asserted source を保持し、real task 72 compatible expansion 3 本
を `TermFormulaChecker` に渡し、terminal-RHS-anchored builtin-set identity 1
個、`BindingId(0)`、source-order use ordinal 1、1 `Inferred` term、1 fact-free
`Checked` type assertion を general reachability なしで要求する。exact source
guard、独立した definition/three-link corruption、withheld-family near miss、
real frontend/resolver sidecar が slice を保護する。mode
declaration acceptance/inhabitation、formula-side local-mode asserted head、
general reachability/widening/`qua`、truth/fact、closure/order、theorem/proof/
Core/VC、broader semantics は deferred のままである。active type-
elaboration runner は 99 cases である。

task 149 は exact two-edge object-terminal local-mode-chain
reserved-variable normalized-reflexive type assertion source を追加する。runner
は raw `OuterTwoEdgeObjectModeTypeAssertion` subject result と独立した
formula-side builtin-object asserted source を保持し、real task 72 compatible
expansion 3 本を `TermFormulaChecker` に渡して、terminal-RHS-anchored builtin-
object identity 1 個、`BindingId(0)`、source-order use ordinal 1、1 `Inferred`
term、1 fact-free `Checked` type assertion を general reachability / object-set
coercion なしで要求する。exact source guard、独立した definition/three-link
corruption、withheld-family near miss、real frontend/resolver sidecar が slice
を保護する。mode declaration acceptance/inhabitation、formula-side local-mode
asserted head、general reachability/widening/`qua`、truth/fact、closure/order、
theorem/proof/Core/VC、broader semantics は deferred のままである。production
route と real sidecar が pass したため、active type-elaboration runner は
100 cases である。

task 150 は exact three-edge set-terminal local-mode-chain reserved-variable
normalized-reflexive type assertion source を追加する。runner は raw
`OuterThreeEdgeModeTypeAssertion` subject result と独立した formula-side
builtin-set asserted source を保持し、real task 73 compatible expansion 4 本を
`TermFormulaChecker` に渡して、terminal-RHS-anchored builtin-set identity 1
個、`BindingId(0)`、source-order use ordinal 1、1 `Inferred` term、1 fact-free
`Checked` type assertion を general reachability なしで要求しなければならない。
exact source guard、独立した definition/four-link corruption、withheld-family
near miss、real frontend/resolver sidecar で slice を保護する。mode declaration
acceptance/inhabitation、formula-side local-mode asserted head、general
reachability/widening/`qua`、truth/fact、closure/order、theorem/proof/Core/VC、
broader semantics は deferred のままである。production route と real sidecar が
pass したため active type-elaboration runner は 101 cases である。

task 151 は exact three-edge object-terminal local-mode-chain reserved-variable
normalized-reflexive type assertion source を追加する。runner は raw
`OuterThreeEdgeObjectModeTypeAssertion` subject result と独立した formula-side
builtin-object asserted source を保持し、real task 73 compatible expansion 4 本を
`TermFormulaChecker` に渡して、terminal-RHS-anchored builtin-object identity 1
個、`BindingId(0)`、source-order use ordinal 1、1 `Inferred` term、1 fact-free
`Checked` type assertion を general reachability / object-set coercion なしで
要求しなければならない。exact source guard、独立した definition/four-link
corruption、withheld-family near miss、real frontend/resolver sidecar で slice を
保護する。mode declaration acceptance/inhabitation、formula-side local-mode
asserted head、general reachability/widening/`qua`、object/set coercion、truth/
fact、closure/order、theorem/proof/Core/VC、broader semantics は deferred の
ままである。production route と real sidecar が pass したため active type-
elaboration runner は 102 cases である。

task 152 は exact four-edge set-terminal local-mode-chain reserved-variable
normalized-reflexive type assertion source を追加する。runner は raw
`TooDeepFourEdgeModeTypeAssertion` subject result と独立した formula-side
builtin-set asserted source を保持し、real task 74 compatible expansion 5 本を
`TermFormulaChecker` に渡して、terminal-RHS-anchored builtin-set identity 1
個、`BindingId(0)`、source-order use ordinal 1、1 `Inferred` term、1 fact-free
`Checked` type assertion を general reachability なしで要求する。exact source
guard、独立した definition/five-link corruption、withheld-family near miss、
real frontend/resolver sidecar で slice を保護する。mode declaration acceptance/
inhabitation、formula-side local-mode asserted head、general reachability/
widening/`qua`、truth/fact、closure/order、theorem/proof/Core/VC、broader
semantics は deferred のままである。production route と real sidecar が pass
したため active type-elaboration runner は 103 cases である。

task 153 は exact four-edge object-terminal local-mode-chain reserved-variable
normalized-reflexive type assertion source を追加する。runner は raw
`TooDeepFourEdgeObjectModeTypeAssertion` subject result と独立した formula-side
builtin-object asserted source を保持し、real task 74 compatible expansion 5 本
を `TermFormulaChecker` に渡して、terminal-RHS-anchored builtin-object identity
1 個、`BindingId(0)`、source-order use ordinal 1、1 `Inferred` term、1 fact-
free `Checked` type assertion を general reachability/object-set coercion なしで
要求する。exact source guard、独立した definition/five-link corruption、
withheld-family near miss、real frontend/resolver sidecar で slice を保護する。
declaration acceptance/inhabitation、formula-side local asserted head、general
reachability/widening/`qua`、object/set coercion、truth/fact、closure/order、
theorem/proof/Core/VC、broader semantics は deferred のままである。production
route と sidecar が pass したため active runner は 104 cases である。

task 154 は test-first exact three-edge set-terminal local-mode-chain reserved-
variable equality source を追加する。runner は raw `OuterThreeEdgeModeEquality`
result/expected input 4 個を保持し、両 `z` operand を ordinal 1、2 で独立に
`BindingId(0)` へ解決し、real task 73 compatible expansion 4 本を
`TermFormulaChecker` へ渡して terminal-RHS builtin-set identity 1 個、2
`Inferred` variable、1 fact/deferred-free `Checked` equality を要求する。exact
source、独立した definition/radix/expansion corruption、withheld-family near
miss、real frontend/resolver sidecar で slice を保護する。mode declaration
acceptance/inhabitation、equality truth/fact、closure/order、theorem/proof/Core/VC
は deferred のままである。production route、full near-miss/corruption matrix、
real frontend/resolver sidecar が pass したため active runner は 105 cases である。

task 155 は test-first exact three-edge object-terminal local-mode-chain
reserved-variable equality source を追加する。runner は raw
`OuterThreeEdgeObjectModeEquality` result/expected input 4 個を保持し、両 `z`
operand を ordinal 1、2 で独立に `BindingId(0)` へ解決し、real task 73
compatible expansion 4 本を `TermFormulaChecker` へ渡して terminal-RHS
builtin-object identity 1 個、2 `Inferred` variable、1 fact/deferred-free
`Checked` equality を要求する。exact source、独立した definition/radix/
expansion corruption、withheld-family near miss、real frontend/resolver sidecar
で slice を保護する。mode declaration acceptance/inhabitation、object/set
coercion、equality truth/fact、closure/order、theorem/proof/Core/VC は deferred
のままである。production route、full near-miss/corruption matrix、real
frontend/resolver sidecar が pass したため active runner は 106 cases である。

task 156 は test-first exact three-edge set-terminal local-mode-chain reserved-
variable inequality source を追加する。runner は raw
`OuterThreeEdgeModeInequality` result/expected input 4 個を保持し、両 `z`
operand を ordinal 1、2 で独立に `BindingId(0)` へ解決し、real task 73
compatible expansion 4 本を `TermFormulaChecker` へ渡して terminal-RHS
builtin-set identity 1 個、2 `Inferred` variable、1 fact/deferred-free pre-
desugaring `Checked` inequality を要求する。exact source、独立した definition/
radix/expansion corruption、withheld-family near miss、real frontend/resolver
sidecar で slice を保護する。mode declaration acceptance/inhabitation、
inequality desugaring、truth/fact、closure/order、theorem/proof/Core/VC は
deferred のままである。production route、full near-miss/corruption matrix、
real frontend/resolver sidecar が pass したため active runner は 107 cases で
ある。

task 157 は exact three-edge object-terminal local-mode-chain reserved-variable
inequality source を追加する。runner は raw
`OuterThreeEdgeObjectModeInequality` result/expected input 4 個を保持し、両 `z`
operand を ordinal 1、2 で独立に `BindingId(0)` へ解決し、real task 73
compatible expansion 4 本を `TermFormulaChecker` へ渡して terminal-RHS
builtin-object identity 1 個、2 `Inferred` variable、1 fact/deferred-free pre-
desugaring `Checked` inequality を要求する。exact source、独立した definition/
radix/expansion corruption、withheld-family near miss、real frontend/resolver
sidecar で slice を保護する。mode declaration acceptance/inhabitation、object/
set coercion、inequality desugaring、truth/fact、closure/order、theorem/proof/
Core/VC は deferred のままである。fixture、expectation、trace row、production
route、full near-miss/corruption matrix、real frontend/resolver sidecar が active
contract を guard するため active runner は 108 cases である。

task 158 は exact active three-edge set-terminal local-mode-chain left
reserved-variable membership source を追加する。runner は raw
`OuterThreeEdgeModeMembership` left result と独立した explicit-set right result/
sole expected input を保持し、left expected input は持たず、`x/y` を ordinal
2/3 で `BindingId(0/1)` へ解決し、real task-73-compatible expansion 4 本を
`TermFormulaChecker` に渡す。active contract は terminal-RHS builtin-set
identity 1 個、2 `Inferred` variable、1 fact/deferred-free `Checked` membership、
exactly one right-owned expected-type constraint を要求する。exact source と
独立した definition/radix/expansion corruption guard を必須とし、mode
declaration acceptance/inhabitation、membership truth/fact、closure/order、
theorem/proof/Core/VC、object-terminal behavior、broader semantics は deferred
のままである。fixture、expectation、trace row、production route、full near-
miss/corruption matrix、real frontend/resolver sidecar が contract を guard
するため active runner は 109 cases である。

task 159 は exact active distinct-binding shared-reserve membership source
`reserve x, y for set; theorem DistinctReservedVariableMembershipPayloadBoundary: x in y;`
を規定する。runner は ordinal 2/3 の distinct `BindingId(0/1)` lookup と、両
binding および left-result/right-result/right-expected role にわたる shared
written set range 1 個を保持し、left expected input は持たず、raw role 3 個を
shared-source-anchored builtin-set identity 1 個へ intern して、2 `Inferred`
variable と 1 fact/deferred-free `Checked` membership、exactly one right-owned
constraint を要求しなければならない。exact guard、corruption/near-miss
matrix、real frontend/resolver sidecar が contract を guard するため active
runner は 110 cases である。truth/fact、closure/order、theorem/proof/Core/VC、
separate declaration、non-set type、broader source shape は deferred のままにする。

task 160 は exact active distinct-binding shared-reserve inequality source
`reserve x, y for set; theorem DistinctReservedVariableInequalityPayloadBoundary: x <> y;`
を規定する。runner は ordinal 2/3 の distinct `BindingId(0/1)` lookup と、両 binding
および operand-owned result/expected role 4 個にわたる shared written set range 1 個
を保持し、それらを shared-source-anchored builtin-set identity 1 個へ intern して、
2 `Inferred` variable と 2 ordered constraint を持つ 1 fact/deferred-free pre-
desugaring `Checked` inequality を要求しなければならない。exact route guard、
corruption/near-miss matrix、real frontend/resolver sidecar が contract を guard する
ため active type-elaboration runner は 111 cases である。desugaring/truth/fact、closure/
order、theorem/proof/Core/VC、separate declaration、non-set type、broader source shape
は deferred のままにする。

task 161 は exact active multiple-reserve-declaration inequality source
`reserve x for set; reserve y for set; theorem
MultipleReserveDeclarationInequalityPayloadBoundary: x <> y;` を規定する。runner
は ordinal 2/3 の `BindingId(0/1)` と operand result/expected pair 2 組の distinct
written range を保持し、全 4 role を earlier `x` range に canonical anchor された
builtin-set identity 1 個へ intern して、2 `Inferred` variable と 2 ordered
constraint を持つ 1 fact/deferred-free pre-desugaring `Checked` inequality を要求する。
exact route guard、corruption/near-miss coverage、real sidecar が contract を guard する
ため active type-elaboration は 112 cases である。shared range、non-set type、desugaring/
truth/fact、closure/order、theorem/proof/Core/VC、broader shape は deferred のままとする。

task 162 は exact active multiple-reserve-declaration membership source
`reserve x for set; reserve y for set; theorem
MultipleReserveDeclarationMembershipPayloadBoundary: x in y;` を記録する。runner
は ordinal 2/3 の `BindingId(0/1)`、first written range を left result、second
range を right result と sole right expected input に保持し、left expected input を
持たない。3 role は earlier `x` range に canonical anchor された builtin-set
identity 1 個へ intern してから、2 `Inferred` variable と exactly one right-owned
constraint を持つ 1 fact/deferred-free `Checked` membership を生成する。exact
route guard、corruption/near-miss coverage、real frontend/resolver sidecar が
contract を guard するため active type-elaboration は 113 cases である。shared
range、non-set type、membership truth/fact、closure/order、theorem/proof/Core/VC、
broader shape は deferred のままとする。

task 163 は active exact three-edge local-object-mode membership source を記録
する。production runner は object-terminal definition chain 4 本、ordered outer-
mode/set reserve、exact `x in y` label だけを受理し、real expansion 4 本をすべて
消費し、raw left / explicit-set right provenance、ordinal 2/3 の
`BindingId(0/1)`、normalized identity 2 個、no left expected input、2
`Inferred` variable、exactly one right-owned constraint を持つ 1 fact/deferred-
free `Checked` membership を要求しなければならない。matched-output
corruption、各 definition-link near miss、real frontend/resolver sidecar が active
runner 114 を fail closed で保護する。object/set coercion、truth/fact、
closure/order、theorem/proof/Core/VC、他の depth、broader shape は deferred の
ままとする。

task 164 は active exact four-edge local-mode membership source を記録する。
production runner は set-terminal definition chain 5 本、ordered outermost-
mode/set reserve、exact `x in y` label だけを受理し、real expansion 5 本を
すべて消費し、raw left / explicit-set right provenance、ordinal 2/3 の
`BindingId(0/1)`、terminal-set-RHS identity 1 個、no left expected input、2
`Inferred` variable、exactly one right-owned constraint を持つ 1 fact/deferred-
free `Checked` membership を要求しなければならない。matched-output
corruption、各 definition-link/order/depth near miss、real frontend/resolver
sidecar は fail closed でなければならない。truth/fact、closure/order、
theorem/proof/Core/VC、object-terminal behavior、他 depth、broader shape は
deferred のままである。exact route、full corruption/near-miss matrix、real
sidecar が active runner 115 を保護する。

task 165 は active exact four-edge local-object-mode membership source を
記録する。production runner は object-terminal definition chain 5 本、ordered
outermost-mode/set reserve、exact `x in y` label だけを受理し、real expansion
5 本をすべて消費し、raw left / explicit-set right provenance、ordinal 2/3 の
`BindingId(0/1)`、distinct terminal-object-RHS / explicit-set identity、no left
expected input、2 `Inferred` variable、exactly one right-owned constraint を持つ
1 fact/deferred-free `Checked` membership を要求しなければならない。matched-
output corruption、各 definition-link/order/depth near miss、real frontend/
resolver sidecar は fail closed でなければならない。truth/fact、object/set
coercion、closure/order、theorem/proof/Core/VC、他 depth、broader shape は
deferred のままである。production routing、full guard、real sidecar が active
runner 116 を保護する。

task 166 は active exact four-edge local-mode equality source を記録する。
production runner は set-terminal definition chain 5 本、outermost mode reserve
1 個、exact `z = z` label だけを受理し、real expansion 5 本を消費し、raw
result/expected input 4 個、ordinal 1/2 の `BindingId(0)`、terminal-set-RHS
identity 1 個、2 `Inferred` variable、1 fact/deferred-free `Checked` equality、
ordered operand-owned expected constraint 2 個を要求しなければならない。
matched-output corruption、各 definition/link/order/
depth near miss、real frontend/resolver sidecar は fail closed でなければならない。
declaration acceptance/inhabitation、truth/fact、closure/order、theorem/proof/
Core/VC、object-terminal behavior、他 depth、broader shape は deferred のまま
である。production routing、full guard、real sidecar が active runner 117 を
保護する。

task 167 は active exact four-edge local-object-mode equality source を
記録する。production runner は object-terminal definition chain 5 本、
outermost mode reserve 1 個、exact `z = z` label だけを受理し、real expansion
5 本を消費し、raw result/expected input 4 個、ordinal 1/2 の
`BindingId(0)`、terminal-object-RHS identity 1 個、2 `Inferred` variable、1
fact/deferred-free `Checked` equality、ordered operand-owned expected
constraint 2 個を object/set coercion なしで要求しなければならない。
matched-output corruption、各 definition/link/order/depth near miss、real
frontend/resolver sidecar は fail closed でなければならない。declaration
acceptance/inhabitation、truth/fact、closure/order、theorem/proof/Core/VC、set-
terminal behavior、他 depth、broader shape は deferred のままである。
production routing、full guard matrix、real sidecar が active runner 118 を
保護する。

task 168 は active exact four-edge local-mode inequality source を記録する。
production runner は set-terminal definition chain 5 本、outermost mode reserve
1 個、exact `z <> z` label だけを受理し、real expansion 5 本を消費し、raw
result/expected input 4 個、ordinal 1/2 の `BindingId(0)`、terminal-set-RHS
identity 1 個、2 `Inferred` variable、1 fact/deferred-free pre-desugaring
`Checked` inequality、ordered operand-owned expected constraint 2 個を要求し
なければならない。matched-output corruption、各 definition/link/order/depth
near miss、real frontend/resolver sidecar は fail closed でなければならない。
declaration acceptance/inhabitation、inequality desugaring/truth/fact、closure/
order、theorem/proof/Core/VC、object-terminal behavior、他 depth、broader shape
は deferred のままである。fixture/expectation、trace backlink 6 件、production
route、full guard matrix、real sidecar が active runner 119 を保護する。

task 169 は active exact four-edge local-object-mode inequality source を記録
する。production runner は object-terminal definition chain 5 本、outermost
mode reserve 1 個、exact `z <> z` label だけを受理し、real expansion 5 本を
消費し、raw result/expected input 4 個、ordinal 1/2 の `BindingId(0)`、
terminal-object-RHS identity 1 個、2 `Inferred` variable、1 fact/deferred-free
pre-desugaring `Checked` inequality、ordered operand-owned expected constraint
2 個を object/set coercion なしで要求しなければならない。matched-output
corruption、各 definition/link/order/depth near miss、real frontend/resolver
sidecar は fail closed でなければならない。declaration acceptance/
inhabitation、inequality desugaring/truth/fact、closure/order、theorem/proof/
Core/VC、set-terminal behavior、他 depth、broader shape は deferred のままで
ある。fixture/expectation、trace backlink 6 件、production route、full guard
matrix、real sidecar が active runner 120 を保護する。

task 172 は test-first exact local-mode long-chain equality source を記録する。
production runner は set-terminal definition chain 7 本、`ChainMode6` reserve
1 個、exact `z = z` label だけを受理し、real AST-derived expansion 7 本を
すべて消費し、raw `ChainMode6` result/expected input 4 個、ordinal 1/2 の
`BindingId(0)`、terminal `BaseMode` RHS の builtin-set identity 1 個、2
`Inferred` variable、1 fact/deferred-free `Checked` equality、ordered operand-
owned expected constraint 2 個を要求しなければならない。matched-output、
definition/link/order/depth/recovery/context/parameterization/terminal/reserve/
formula/symbol、expansion-corruption の full guard と real frontend/resolver
sidecar は fail closed である。declaration acceptance/
inhabitation、truth/fact、closure/order、theorem/proof/Core/ControlFlow/VC、
imported/attributed/argument-bearing または別 chain shape、general unbounded
semantics は deferred のままである。
production routing、full guard、real sidecar が active runner 121 を保護する。

task 173 は test-first long-chain inequality sibling を記録する。production
runner は同じ definition 7 本と `ChainMode6` reserve に exact `z <> z` だけを
受理し、real expansion 7 本、raw role 4 個、ordinal 1/2 の `BindingId(0)`、
terminal `BaseMode` RHS identity 1 個、2 `Inferred` variable、ordered
constraint 2 個、1 fact/deferred-free pre-desugaring `Checked` inequality を
要求する。task 172 の full guard matrix と real sidecar breadth が active
runner 122 を保護する。desugaring/truth/fact と downstream/general semantics は deferred のままで
ある。

task 174 は test-first long-chain membership sibling を記録する。production
runner は同じ definition 7 本、ordered `ChainMode6`/`set` reserve、exact
`x in y` だけを受理し、real expansion 7 本、raw left と独立した right result/
sole expected input、ordinal 2/3 の `BindingId(0/1)`、terminal `BaseMode` RHS
identity 1 個、left expected input なし、2 `Inferred` variable、right-owned
constraint 1 個、1 fact/deferred-free `Checked` membership を要求する。task 172
の full structural guard matrix、membership-specific corruption、real sidecar は
fail closed する。truth/fact と downstream/general semantics は deferred のまま
である。production routing、full guard、real sidecar が active runner 123 を
保護する。

task 175 は test-first long-chain type-assertion sibling を記録する。production
runner は同じ definition 7 本、`ChainMode6` reserve 1 個、exact `x is set` だけ
を受理し、real expansion 7 本、raw subject と独立した formula-side builtin-set
asserted input、ordinal 1 の `BindingId(0)`、terminal `BaseMode` RHS identity
1 個、1 `Inferred` variable、general reachability を用いない 1 fact/deferred-
free normalized-reflexive `Checked` type assertion を要求する。task 172 の full
structural guard matrix、type-assertion-specific corruption、real sidecar は fail
closed しなければならない。widening/`qua`、truth/fact、downstream/general
semantics は deferred のままである。test-first row、production support、full
guard、real sidecar が active runner 124 を保護する。

task 176 は test-first builtin-object-terminal long-chain equality sibling を
記録する。production runner は exact definition 7 本、`ChainObjectMode6`
reserve 1 個、exact `z = z` だけを受理し、real expansion 7 本、raw result/
expected input 4 個、ordinal 1/2 の `BindingId(0)`、terminal
`BaseObjectMode` RHS identity 1 個、2 `Inferred` term、ordered operand-owned
constraint 2 個、object/set coercion を用いない 1 fact/deferred-free `Checked`
equality を要求する。task 172 の shared full structural guard matrix、object-
terminal/matched-output corruption、real sidecar は fail closed しなければならない。
truth/fact と downstream/general semantics は deferred のままである。test-first
row、production support、full guard、real sidecar が active runner 125 を保護する。

task 177 は test-first builtin-object-terminal long-chain inequality sibling を
記録する。production runner は exact definition 7 本、`ChainObjectMode6`
reserve 1 個、exact `z <> z` だけを受理し、real expansion 7 本、raw result/
expected input 4 個、ordinal 1/2 の `BindingId(0)`、terminal
`BaseObjectMode` RHS identity 1 個、2 `Inferred` term、ordered operand-owned
constraint 2 個、object/set coercion を用いない 1 fact/deferred-free pre-
desugaring `Checked` inequality を要求する。task 172 の shared
full structural guard matrix、object-terminal/matched-output corruption、real
sidecar は fail closed する。inequality desugaring、truth/fact、
downstream/general semantics は deferred のままである。test-first row は存在し、
production support、full guard、real sidecar が active runner 126 を保護する。

task 178 は builtin-object-terminal long-chain left-membership sibling を support
する。production runner は exact definition 7 本、ordered `x`/`y` reserve
for `ChainObjectMode6`/explicit `set`、exact `x in y` だけを受理し、real
expansion 7 本、raw left result、独立した right result/sole expected input、
ordinal 2/3 の `BindingId(0/1)`、distinct terminal-object-RHS と explicit-set
identity、left expected input なし、2 `Inferred` term、right-owned constraint 1
個、object/set coercion を用いない 1 fact/deferred-free `Checked` membership を
要求しなければならない。task 172 の shared full structural guard matrix、
membership/object-specific corruption、real sidecar は fail closed する。truth/fact、
downstream/general semantics は deferred のままである。fixture、production
support、guard が active runner 127 を保護する。

task 179 は builtin-object-terminal long-chain type-assertion sibling を support
する。production runner は exact definition 7 本、`x` reserve for
`ChainObjectMode6` 1 個、exact `x is object` だけを受理し、real expansion 7 本、
raw subject result、独立した formula-side builtin-object asserted input、ordinal
1 の `BindingId(0)`、terminal-object-RHS identity 1 個、1 `Inferred` term、
general reachability と object/set coercion を用いない 1 fact/deferred-free
normalized-reflexive `Checked` type assertion を要求しなければならない。task
172 shared full structural guard matrix と task 153 の real object consumer/source
near miss を再利用し、task 175 の matched-output guard は builtin-set asserted
head と raw subject provenance corruption を reject し、real sidecar は fail
closed する。truth/
fact、acceptance、downstream/general semantics は deferred のままである。
fixture、production support、full guard、real sidecar が active runner 128 を保護する。

task 180 は standalone formula leaf
`theorem SourceDerivedContradictionConstantBoundary: contradiction;` だけを
support する。production route は real leaf site/range と module-root context を
保持する exact extractor を追加し、deferred reason なしに
`FormulaKind::Contradiction` を既存 checker consumer へ渡す。1 `Checked`
formula と、空の term、asserted type、expected constraint、candidate、fact、
deferred reason、diagnostic を要求する。wrong label/constant、status/recovery
marker、extra item、duplicate theorem は既存 path に残り、real frontend/
resolver sidecar が active runner 129 を保護する。これは formula type/well-
formedness だけであり、falsehood/fact publication、theorem acceptance、proof-
goal closure、child-graph extraction、`formula_statement`、proof、CoreIr、
ControlFlowIr、VC coverage ではない。

task 182 は最初の formula-side local-mode asserted-head pass case を追加する。
production route は `mode LocalModeAssertedHeadDef: LocalModeAssertedHead is
set;` を含む `definition` block、matching reserve 1 個、exact `x is
LocalModeAssertedHead` だけを受理する。
同じ resolved mode symbol について distinct raw reserve-subject と formula-side
asserted site/range を保持し、real expansion 1 個を消費し、ordinal 1 を
`BindingId(0)` に解決し、terminal-RHS builtin-set identity 1 個へ intern する
known type entry 3 個、1 `Inferred` variable、1 fact/deferred-free normalized-
reflexive `Checked` type assertion を要求する。exact/near-miss、
matched-output corruption、route-order、real frontend/resolver-sidecar tests は
collapsed provenance、他 asserted head、broader definition/item で fail closed
する。新規 active pass case は runner を 129 から 130 に増やし、
real sidecar はその case を保護する。declaration acceptance/inhabitation、
widening/`qua`、truth/fact、theorem/
proof/CoreIr/ControlFlowIr/VC、child graph、他 asserted-head family、general
semantics は credit しない。

task 183 は direct object-terminal formula-side local-mode asserted-head pass
case を追加する。production route は `mode LocalObjectModeAssertedHeadDef:
LocalObjectModeAssertedHead is object;` を持つ definition block 1 個、matching
reserve 1 個、exact `x is LocalObjectModeAssertedHead` だけを受理する。
同じ resolved symbol について distinct raw reserve-subject/formula-side asserted
site/range を保持し、real expansion 1 個を消費し、ordinal 1 を `BindingId(0)`
に解決し、known type entry 3 個を terminal-RHS builtin-object identity 1 個へ
intern し、general reachability と object/set coercion を用いず、1 `Inferred`
variable と 1 fact/deferred-free normalized-reflexive `Checked` type assertion を
要求する。exact/near-miss、matched-output corruption、route-order、
real frontend/resolver-sidecar test は set terminal、builtin/other asserted head、
chain、attribute/argument、recovery、extra item、collapsed provenance で fail
closed する。新規 active pass case は runner を 130 から 131 に
増やす。declaration acceptance/inhabitation、truth/fact、theorem/proof/CoreIr/
ControlFlowIr/VC、他 asserted-head family、general semantics は credit しない。

task 184 は exact one-edge set-terminal same-outer-mode asserted-head pass case
を追加する。production route は `mode BaseModeAssertedHeadDef:
BaseModeAssertedHead is set;` と `mode ChainModeAssertedHeadDef:
ChainModeAssertedHead is BaseModeAssertedHead;` を含む ordered definition
block 2 個、matching outer-mode reserve 1 個、exact `x is ChainModeAssertedHead` だけを
受理する。同じ resolved outer symbol について distinct raw reserve-
subject/formula-side asserted site/range を保持し、real expansion 2 個を消費し、
ordinal 1 を `BindingId(0)` に解決し、known type entry 3 個を terminal base-
definition-RHS builtin-set identity 1 個へ intern し、general reachability を
用いず、1 `Inferred` variable と 1 fact/deferred-free normalized-reflexive
`Checked` type assertion を要求する。exact/near-miss、matched-
output corruption、route-order、real frontend/resolver-sidecar test は wrong
link/terminal/order/depth、builtin/base/other asserted head、attribute/argument、
recovery、extra item、collapsed provenance で fail closed する。
active pass count は 131 から 132 に増える。declaration acceptance/
inhabitation、widening/`qua`、truth/fact、closure/order、theorem/proof/CoreIr/
ControlFlowIr/VC、object/deeper/他 asserted-head chain、general chain semantics は
credit しない。

task 185 は exact one-edge object-terminal same-outer-mode asserted-head pass
case を追加する。production route は `mode BaseObjectModeAssertedHeadDef:
BaseObjectModeAssertedHead is object;` と `mode ChainObjectModeAssertedHeadDef:
ChainObjectModeAssertedHead is BaseObjectModeAssertedHead;` を含む ordered
definition block 2 個、matching outer-mode reserve 1 個、exact `x is
ChainObjectModeAssertedHead` だけを受理してよい。同じ resolved outer symbol に
ついて distinct raw reserve-subject/formula-side asserted range を保持し、real
expansion 2 個を消費し、ordinal 1 を `BindingId(0)` に解決し、known type entry
3 個を terminal base-definition-RHS builtin-object identity 1 個へ intern し、
general reachability、widening、`qua`、object/set coercion を用いず、1
`Inferred` variable と 1 fact/deferred-free normalized-reflexive `Checked` type
assertion を要求する。exact/near-miss、matched-output corruption、route-order、
real frontend/resolver-sidecar test は wrong link/terminal/order/depth、builtin/
base/other asserted head、attribute/argument、imported provenance、recovery、
extra item、collapsed provenance、builtin-set output corruption で fail closed
する。shared trace backlink 5 個と dedicated row 1 個が active count 133 を
保護する。imported/declaration/attribute、broader
term/formula/child graph、truth/fact、theorem/proof/CoreIr/ControlFlowIr/VC、
deeper/他 asserted head、general chain coverage は credit しない。module layout
更新は不要だった。

task 186 は exact two-edge set-terminal same-outer-mode asserted-head pass case
を追加する。route は `BaseTwoEdgeModeAssertedHead -> set`、
`MiddleTwoEdgeModeAssertedHead -> BaseTwoEdgeModeAssertedHead`、
`OuterTwoEdgeModeAssertedHead -> MiddleTwoEdgeModeAssertedHead` の ordered
definition、matching outer-mode reserve、exact
`TwoEdgeLocalModeAssertedHeadPayloadBoundary: x is
OuterTwoEdgeModeAssertedHead` だけを受理する。同じ symbol 向けの distinct
reserve/asserted range、real expansion 3 個、ordinal 1 の `BindingId(0)`、terminal
base RHS builtin-set identity 1 個へ intern する known entry 3 個、reachability、
widening、`qua` を用いない 1 `Inferred` variable と 1 fact/deferred-free
normalized-reflexive `Checked` assertion を要求する。exact/near-miss、corruption、
route-order、real frontend/resolver-sidecar guard は structural chain failure、
imported/ambiguous provenance、collapsed provenance、builtin-object corruption
を reject する。shared 5 + dedicated 1 trace row が active count 134 を保護する。
object/deeper/imported semantics、declaration/attribute acceptance、broader
term/formula/child graph、truth/fact、proof/CoreIr/ControlFlowIr/VC、general chain
semantics は credit しない。module layout 更新は不要である。

task 187 は exact two-edge object-terminal same-outer-mode asserted-head pass
case を追加する。route は ordered definition
`mode BaseTwoEdgeObjectModeAssertedHeadDef: BaseTwoEdgeObjectModeAssertedHead is
object;`、`mode MiddleTwoEdgeObjectModeAssertedHeadDef:
MiddleTwoEdgeObjectModeAssertedHead is BaseTwoEdgeObjectModeAssertedHead;`、
`mode OuterTwoEdgeObjectModeAssertedHeadDef: OuterTwoEdgeObjectModeAssertedHead
is MiddleTwoEdgeObjectModeAssertedHead;`、
matching outer reserve、exact `TwoEdgeLocalObjectModeAssertedHeadPayloadBoundary:
x is OuterTwoEdgeObjectModeAssertedHead` だけを受理する。同じ local symbol
向けの distinct raw subject/asserted range、real expansion 3 個、ordinal 1 の
`BindingId(0)`、base-definition-RHS builtin-object identity 1 個へ intern する
known entry 3 個、expected constraint、reachability、widening、`qua`、object/set
coercion を持たない 1 `Inferred` variable と 1 fact/deferred-free normalized-
reflexive `Checked` assertion を要求する。exact/near-miss、corruption、route-
order、real frontend/resolver-sidecar guard は imported Base/Middle/Outer、
imported/ambiguous asserted head を含む全 non-exact link/depth/terminal/provenance
shape、wrong label、attributed/argument-bearing formula-side asserted head と
`BuiltinSet` output corruption を reject する。shared 5 + dedicated 1 trace row
が active count 135 を保護する。positive imported semantics、
declaration/attribute acceptance、broader term/formula/child graph、truth/fact、
implicit closure/order、theorem acceptance、proof/CoreIr/ControlFlowIr/VC、
general chain semantics は credit しない。Step 5 は active、Steps 6/7 は deferred
のまま。module layout 更新は不要である。

task 188 は exact active builtin-object equality source `reserve x for object;
theorem ReservedObjectVariableEqualityPayloadBoundary: x = x;` だけを追加する。
active route は real source-derived object reserve handoff と既存 reserved-variable
equality consumer を再利用し、ordinal 1/2 local lookup と written reserve range
上の distinct result/expected role site 4 個を保持し、canonical builtin-object
identity 1 個、`Inferred` variable term 2 個、ordered expected constraint 2 個、
fact/deferred-free `Checked` equality 1 個だけを受理する。exact/near-miss/
corruption と real frontend/resolver-sidecar test は source shape、binding identity、
lookup order、role provenance、checker count/status、constraint、canonical source、
wrong builtin-set output を guard する。既存 expectation を rebaseline せず、shared
backlink 5 個 + dedicated trace row 1 個により active runner 136 を保護する。
general object equality、coercion、truth/fact、closure/order、theorem/proof
acceptance、downstream payload は active 化しない。

task 189 は exact active builtin-object type-assertion source
`reserve x for object; theorem
ReservedObjectVariableTypeAssertionPayloadBoundary: x is object;` だけを追加する。
active route は real source-derived object reserve handoff と既存 reserved-variable
type-assertion consumer を再利用し、ordinal 1 local lookup と distinct reserve-
subject result/formula-side asserted site/range を保持し、reserve を anchor とする
canonical builtin-object identity 1 個、`Inferred` variable term 1 個、known type
entry 3 個、expected constraint 0 個、fact/deferred-free `Checked` assertion 1 個
だけを受理する。exact/near-miss/corruption と real frontend/resolver-sidecar test
は source shape、binding identity、lookup order、raw input provenance、checker
count/status、constraint 不在、canonical source、wrong builtin-set output を guard
しなければならない。既存 expectation を rebaseline せず、shared backlink 5 個 +
dedicated trace row 1 個により active runner 137 を保護する。reachability/
widening/`qua`、object/set coercion、truth/fact、closure/order、theorem/proof
acceptance、downstream payload は active 化しない。

task 190 は exact active builtin-object inequality source `reserve x for
object; theorem ReservedObjectVariableInequalityPayloadBoundary: x <> x;`
だけを追加する。active route は real source-derived object reserve handoff と
既存 reserved-variable inequality consumer を再利用し、ordinal 1/2 local
lookup と written reserve range 上の distinct result/expected role site 4 個を
保持し、canonical builtin-object identity 1 個、`Inferred` variable term 2 個、
known type entry 6 個、ordered expected constraint 2 個、fact/candidate/
diagnostic/deferred-free pre-desugaring `Checked` inequality 1 個だけを受理する。
exact/near-miss/corruption と real frontend/resolver-sidecar test は source shape、
binding identity、lookup ordering、role provenance、checker count/status、
constraint、canonical source、wrong builtin-set output を guard しなければ
ならない。既存 expectation を rebaseline せず shared backlink 5 個 + dedicated
trace row 1 個により active runner 138 を保護する。inequality desugaring/
equality truth、object/set coercion、fact、closure/order、theorem/proof acceptance、
downstream payload は active 化しない。

task 191 は exact active distinct-binding shared-builtin-object equality source
`reserve x, y for object; theorem
DistinctReservedObjectVariableEqualityPayloadBoundary: x = y;` だけを追加する。
active route は real one-item/two-binding shared-range reserve handoff と既存
builtin-object equality consumer を合成し、ordinal 2/3 の local lookup と
shared written reserve range 上の distinct result/expected role site 4 個を
保持し、reserve range を anchor とする canonical builtin-object identity
1 個、`Inferred` variable term 2 個、known type entry 6 個、ordered expected
constraint 2 個、fact/candidate/diagnostic/deferred-free `Checked` equality
1 個だけを受理する。exact/near-miss/corruption と real
frontend/resolver-sidecar test は source shape、distinct binding identity、
lookup ordering、shared-range role provenance、checker count/status、constraint、
canonical source、wrong builtin-set output を guard する。shared backlink 5 個 +
dedicated trace row 1 個により既存 expectation を rebaseline せず active runner
139 を保護する。これは equality truth、object/set coercion、fact、
closure/order、theorem/proof acceptance、downstream payload を activate しない。

task 192 は exact active distinct-binding shared-builtin-object inequality source
`reserve x, y for object; theorem
DistinctReservedObjectVariableInequalityPayloadBoundary: x <> y;` だけを対象と
する。active route は real one-item/two-binding shared-range reserve handoff と
既存 pre-desugaring inequality consumer を合成し、ordinal 2/3 の local lookup
と shared written reserve range 上の distinct result/expected role site 4 個を
保持し、reserve range を anchor とする canonical builtin-object identity 1 個、
`Inferred` variable term 2 個、known type entry 6 個、ordered expected constraint
2 個、fact/candidate/diagnostic/deferred-free `Checked` inequality 1 個だけを
受理する。exact/near-miss/corruption と real frontend/resolver-sidecar test は
source shape、distinct binding identity、lookup ordering、shared-range role
provenance、checker count/status、constraint、canonical source、wrong builtin-set
output を guard する。shared backlink 5 個 + dedicated trace row 1 個により既存
expectation を rebaseline せず active runner 140 を保護する。これは
inequality desugaring/equality truth、object/set coercion、fact、closure/order、
theorem/proof acceptance、downstream payload を activate しない。

task 193 は exact active multiple-reserve-declaration builtin-object equality
source `reserve x for object; reserve y for object; theorem
MultipleObjectReserveDeclarationEqualityPayloadBoundary: x = y;` だけを対象と
する。active route は real two-item/two-binding/distinct-written-range reserve
handoff と builtin-object equality consumer を合成し、ordinal 2/3 の local
lookup と binding ごとの written range 2 個上の distinct result/expected role
site 4 個を保持し、先行する `x` range を anchor とする canonical builtin-
object identity 1 個、`Inferred` variable term 2 個、known type entry 6 個、
ordered expected constraint 2 個、fact/candidate/diagnostic/deferred-free
`Checked` equality 1 個だけを受理する。exact/near-miss/corruption と real
frontend/resolver-sidecar test は source shape、distinct binding/range、lookup
ordering、raw role provenance、checker count/status、constraint、canonical
source、route isolation、wrong builtin-set output を guard する。shared backlink
5 個 + dedicated trace row 1 個により既存 expectation を rebaseline せず active
runner 141 を保護する。これは equality truth、object/set coercion、fact、
closure/order、theorem/proof acceptance、shared-range shape、downstream payload
を activate しない。

task 194 は exact active multiple-reserve-declaration builtin-object inequality
source `reserve x for object; reserve y for object; theorem
MultipleObjectReserveDeclarationInequalityPayloadBoundary: x <> y;` だけを対象と
する。active route は Task 193 の ordered two-item/two-binding/distinct-written-
object-range handoff と pre-desugaring builtin-object inequality consumer を
合成し、ordinal 2/3 の local lookup と binding ごとの written range 2 個上の
distinct raw result/expected role 4 個を保持し、先行する `x` range を anchor と
する canonical builtin-object identity 1 個、`Inferred` variable term 2 個、
known type entry 6 個、ordered expected constraint 2 個、fact/candidate/
diagnostic/deferred-free `Checked` inequality 1 個だけを受理する。exact/near-
miss/corruption と real frontend/resolver-sidecar test は source shape、distinct
binding と ordered range、lookup ordering、raw role provenance、checker count/
status、constraint、canonical source、route isolation、wrong builtin-set output
を guard する。shared backlink 5 個 + dedicated trace row 1 個により既存
expectation を rebaseline せず active runner 142 を保護する。これは inequality
desugaring/equality truth、object/set coercion、fact、closure/order、theorem/
proof acceptance、shared-range shape、downstream payload を activate しない。

task 195 は ordered definition 4 個 `Outer -> Middle -> Inner -> Base -> set`、
outer-mode reserve 1 個、`ThreeEdgeLocalModeAssertedHeadPayloadBoundary: x is
OuterThreeEdgeModeAssertedHead;` を持つ exact active three-edge set-terminal
same-outer-mode asserted-head source だけを対象とする。active route は real
AST-derived expansion 4 個を消費し、同じ resolved outer symbol の raw reserve-
subject と formula-side asserted-type の独立した site/range を保持する。ordinal
1 から解決する `BindingId(0)`、base-definition-RHS anchor の builtin-set
identity 1 個へ normalize する known type entry 3 個、`Inferred` variable 1
個、expected constraint/candidate/fact/diagnostic/deferred reason 0 個、
normalized-reflexive `Checked` assertion 1 個だけを受理する。exact、
structural、provenance、corruption、immutable-output、route-isolation、real
frontend/resolver-sidecar test は全 4 link を guard し、unrelated local、
imported、ambiguous asserted head を reject する。shared backlink 5 個 +
dedicated trace row 1 個により既存 expectation を rebaseline せず active runner
143 を保護する。これは reachability/widening/`qua`、declaration/theorem
acceptance、truth/fact、closure/order、broader term/formula/child-graph
semantics、proof、downstream IR を activate しない。

task 196 は ordered definition 4 個 `Outer -> Middle -> Inner -> Base ->
object`、outer-mode reserve 1 個、
`ThreeEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
OuterThreeEdgeObjectModeAssertedHead;` を持つ exact active three-edge object-
terminal same-outer-mode asserted-head source だけを対象とする。active route は
real AST-derived expansion 4 個を消費し、同じ resolved outer symbol の raw
reserve-subject と formula-side asserted-type の独立した site/range を保持する。
ordinal 1 から解決する `BindingId(0)`、base-definition-RHS anchor の builtin-
object identity 1 個へ normalize する known type entry 3 個、`Inferred`
variable 1 個、expected constraint/candidate/fact/diagnostic/deferred reason 0
個、normalized-reflexive `Checked` assertion 1 個だけを object/set coercion
なしで受理する。exact、structural、provenance、`BuiltinSet`/canonical
corruption、immutable-output、route-isolation、real frontend/resolver-sidecar
test は全 4 link を guard し、unrelated local、imported、ambiguous asserted
head を reject する。shared backlink 5 個 + dedicated trace row 1 個により既存
expectation を rebaseline せず active runner 144 を保護する。これは
reachability/widening/`qua`、declaration/theorem acceptance、truth/fact、closure/
order、broader term/formula/child-graph semantics、proof、downstream IR を
activate しない。

task 197 は ordered definition 5 個 `TooDeep -> Outer -> Middle -> Inner ->
Base -> set`、outermost-mode reserve 1 個、
`FourEdgeLocalModeAssertedHeadPayloadBoundary: x is
TooDeepFourEdgeModeAssertedHead;` を持つ exact active four-edge set-terminal
same-outermost-mode asserted-head source だけを対象とする。active route は real
AST-derived expansion 5 個を消費し、同じ resolved outermost symbol の raw
reserve-subject と formula-side asserted-type の独立した site/range を保持する。
ordinal 1 から解決する `BindingId(0)`、base-definition-RHS anchor の builtin-
set identity 1 個へ normalize する known type entry 3 個、`Inferred` variable
1 個、expected constraint/candidate/fact/diagnostic/deferred reason 0 個、
normalized-reflexive `Checked` assertion 1 個だけを受理する。exact、full-
reorder、connected-deeper、structural、provenance、`BuiltinObject`/canonical
corruption、immutable-output、route-isolation、real frontend/resolver-sidecar
test は全 5 link を guard し、unrelated local、imported、ambiguous asserted
head を reject する。shared backlink 5 個 + dedicated trace row 1 個により
既存 expectation を rebaseline せず active runner 145 を保護する。これは
reachability/widening/`qua`、declaration/theorem acceptance、truth/fact、
closure/order、broader term/formula/child-graph semantics、proof、downstream IR
を activate しない。

task 198 は ordered definition 5 個 `TooDeep -> Outer -> Middle -> Inner ->
Base -> object`、outermost-mode reserve 1 個、
`FourEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
TooDeepFourEdgeObjectModeAssertedHead;` を持つ exact active four-edge object-
terminal same-outermost-mode asserted-head source だけを対象とする。active
route は real AST-derived expansion 5 個を消費し、同じ resolved outermost
symbol の raw reserve-subject と formula-side asserted-type の独立した site/
range を保持する。ordinal 1 から解決する `BindingId(0)`、base-definition-RHS
anchor の builtin-object identity 1 個へ normalize する known type entry 3
個、`Inferred` variable 1 個、expected constraint/candidate/fact/diagnostic/
deferred reason 0 個、normalized-reflexive `Checked` assertion 1 個だけを
object/set coercion なしで受理する。exact、full-reorder、connected-deeper、
structural、provenance、`BuiltinSet`/canonical corruption、immutable-output、
route-isolation、real frontend/resolver-sidecar test は全 5 link を guard し、
unrelated local、imported、ambiguous asserted head を reject する。shared
backlink 5 個 + dedicated trace row 1 個により既存 expectation を rebaseline
せず active runner 146 を保護する。これは reachability/widening/`qua`、
declaration/theorem acceptance、truth/fact、closure/order、broader term/formula/
child-graph semantics、proof、downstream IR を activate しない。

task 199 は `BaseMode -> set`、`ChainMode6 -> ChainMode5` までの ordered link 6
個、`ChainMode6` reserve 1 個、
`LongLocalModeAssertedHeadPayloadBoundary: x is ChainMode6;` を持つ exact
active seven-expansion set-terminal same-`ChainMode6` asserted-head source だけ
を対象とする。active route は real AST-derived expansion 7 個を消費し、同じ
resolved symbol の raw reserve-subject と formula-side asserted-type の独立した
site/range を保持する。ordinal 1 から解決する `BindingId(0)`、`BaseModeDef`
RHS anchor の builtin-set identity 1 個へ normalize する known type entry 3
個、`Inferred` variable 1 個、expected constraint/candidate/fact/diagnostic/
deferred reason 0 個、normalized-reflexive `Checked` assertion 1 個だけを受理
する。exact、per-link removal/reorder、complete-reverse、connected-eighth、
structural、provenance、`BuiltinObject`/canonical corruption、immutable-output、
route-isolation、real frontend/resolver-sidecar test は全 7 link を guard し、
unrelated local、imported、ambiguous asserted head を reject する。shared
backlink 5 個 + dedicated trace row 1 個により既存 expectation を rebaseline
せず active runner 147 を保護する。これは object-terminal/other-depth/
imported/attributed/argument-bearing/other asserted head、reachability/
widening/`qua`、declaration/theorem acceptance、truth/fact、closure/order、
broader term/formula/child-graph semantics、proof、downstream IR を activate
しない。

task 200 は `BaseObjectMode -> object`、`ChainObjectMode6 -> ChainObjectMode5`
までの ordered link 6 個、`ChainObjectMode6` reserve 1 個、
`LongLocalObjectModeAssertedHeadPayloadBoundary: x is ChainObjectMode6;` を
持つ exact active seven-expansion object-terminal same-`ChainObjectMode6`
asserted-head source だけを対象とする。active route は real AST-derived
expansion 7 個を消費し、同じ resolved symbol の raw reserve-subject と
formula-side asserted-type の独立した site/range を保持する。ordinal 1 から
解決する `BindingId(0)`、`BaseObjectModeDef` RHS anchor の builtin-object
identity 1 個へ normalize する known type entry 3 個、`Inferred` variable 1
個、expected constraint/candidate/fact/diagnostic/deferred reason 0 個、object/
set coercion のない normalized-reflexive `Checked` assertion 1 個だけを受理
する。exact、per-link removal/reorder、complete-reverse、connected-eighth、
structural、provenance、`BuiltinSet`/canonical corruption、immutable-output、
route-isolation、real frontend/resolver-sidecar test は全 7 link を guard し、
unrelated local、imported、ambiguous asserted head を reject する。shared
backlink 5 個 + dedicated trace row 1 個により既存 expectation を rebaseline
せず active runner 148 を保護する。これは set-terminal/other-depth/imported/
attributed/argument-bearing/other asserted head、reachability/widening/`qua`、
declaration/theorem acceptance、truth/fact、closure/order、broader term/
formula/child-graph semantics、proof、downstream IR を activate しない。

task 120 は matching exact membership pass case
`reserve x for set; theorem ReservedVariableMembershipPayloadBoundary: x in x;`
を追加する。active runner は task 119 の match-before-build と独立した source-order
lookup path を共有するが、membership の exact payload shape、すなわち 2 つの known
`set` variable result、右 operand だけの expected-`set` constraint、3 つの exact
source-anchored role、1 つの `Checked` membership、empty
candidate/fact/deferred/diagnostic を要求する。matched-source construction または
invariant drift は
`type_elaboration.checker.reserved_variable_membership.invalid_payload` を報告し、
その他の near-miss は extraction gap に残る。real frontend/resolver unit test は
active sidecar payload を観測する。これは well-formedness coverage だけであり、
membership truth、recorded fact、implicit closure、theorem acceptance、
proof/Core/ControlFlow/VC promotion ではない。

task 121 は exact inequality sibling
`reserve x for set; theorem ReservedVariableInequalityPayloadBoundary: x <> x;`
を追加する。checker-owned inequality API が 2 つの expected-type slot を提供し、
task 119 が real reserve binding/use producer を提供する。task 107 の numeral
inequality bridge は expected type なしの partial のままである。shared active
producer は 2 つの linked result role、2 つの linked expected role、2 つの
`Inferred` variable、1 つの fact-free pre-desugaring `Checked` inequality を要求する。
task-specific invalid-payload key、full near-miss matrix、real frontend/resolver
payload test が slice を guard する。inequality desugaring、truth/fact、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC は credit しない。

task 122 は exact type-assertion sibling
`reserve x for set; theorem ReservedVariableTypeAssertionPayloadBoundary: x is set;`
を追加する。active producer は task 119 の real reserve lookup/result input と
task 109 の formula-side asserted-type AST input を結合し、normalization 前の
distinct source anchor を保持して、checker が normalized reflexive identity
だけを受理することを要求する。output は 1 `Inferred` variable、1 fact-free
`Checked` type assertion、empty candidate/deferred/diagnostic を持つ。known
non-identical type は
`checker.formula.external.type_assertion_reachability_payload` を使って partial
に残る。task-specific invalid-payload key、列挙済み near-miss matrix、real
frontend/resolver payload test が slice を guard する。general
reachability/widening/`qua`、attribute、truth/fact、implicit closure、theorem
acceptance、proof、CoreIr、ControlFlowIr、VC は credit しない。

task 109 は task 102 の exact builtin type-assertion theorem sidecar を
supersede する。active `type_elaboration` runner は real source-derived checker
`TermInput`、`FormulaInput`、asserted builtin `set` `TypeExpressionInput`
payload を渡してから missing numeric type payload と partial formula checking で
fail closed する。deferred `formula_statement` runner obligation は満たさず、
broader asserted type payload、type-assertion semantic checking、fact、theorem
acceptance、CoreIr、ControlFlowIr、VC、proof payload は credit しない。

task 113 は task 103 の exact imported attribute assertion theorem formula を
supersede して同じ row を refine する。active `type_elaboration` runner は
`parser.type_fixtures` の imported `empty` provenance を検証し、real
source-derived checker term/formula payload を渡してから missing numeric type
payload、missing formula / attribute semantic payload、partial formula checking で
fail closed する。deferred `formula_statement` runner obligation を満たさず、
imported module AST extraction、attribute-chain semantic payload、theorem formula
向け checker `AttributeInput` payload、term inference、attribute
admissibility/semantic checking、formula checking、fact、theorem acceptance、
CoreIr、ControlFlowIr、VC、proof payload は credit しない。

task 114 は同じ row を refine し、`parser.type_fixtures` の `empty` を使う exact
attribute-level `non empty` imported attribute assertion theorem formula について
task 104 を supersede する。active `type_elaboration` runner は direct `non`
surface と imported `empty` provenance を検証し、real source-derived checker
term/formula payload を渡してから、missing numeric type payload、missing formula /
attribute semantic payload、partial formula checking で fail closed する。deferred
`formula_statement` runner obligation を満たさず、imported module AST extraction、
negated attribute-chain semantic payload、theorem formula 向け checker
`AttributeInput` payload、term inference、negated attribute admissibility/semantic
checking、formula checking、fact、theorem acceptance、CoreIr、ControlFlowIr、VC、
proof payload は credit しない。上の theorem formula boundary entry は task 114
の exact attribute-level non-empty imported attribute assertion checker bridge も
含む。

task 111 は task-105 set-enumeration theorem formula boundary のうち exact
`SetEnumerationPayloadBoundary: {1, 2} = {1, 2}` source だけを supersede する。
active `type_elaboration` runner は 4 つの numeral item term、2 つの
set-enumeration term、builtin equality formula の real checker payload を渡し、
missing numeric type payload、missing set-enumeration result-type
payload、partial formula checking で fail closed する。deferred
`formula_statement` runner obligation は満たさず、broader set-enumeration
payload、term inference、equality/formula checking、fact、theorem acceptance、
CoreIr、ControlFlowIr、VC、proof payload は credit しない。

task 112 は exact connective/quantifier theorem formula source だけについて
task 99 を supersede し、同じ row を refine する。active `type_elaboration`
runner は implication、universal quantification、negation の real checker
`FormulaInput` shell を渡し、missing formula/quantifier payload で fail closed
する。deferred `formula_statement` runner obligation は満たさず、formula
constant、child-formula graph payload、quantifier binder/context payload、formula
checking、fact、theorem acceptance、CoreIr、ControlFlowIr、VC、proof payload は
credit しない。

task 88 も同じ row を refine する。proof-block theorem source は active
`type_elaboration` runner を通じて extraction-gap boundary としてだけ実行可能で
ある。deferred `formula_statement` runner obligation を満たさず、proof skeleton
payload、local proof context、formula payload、fact、theorem acceptance、CoreIr、
ControlFlowIr、VC、proof payload は credit しない。上の row の theorem/proof
boundary entry は task 86 の formula-only case、task 87 の term-bearing equality
case、task 88 の proof-block case を含む。

task 89 も同じ row を refine する。statement-level proof-justification theorem
source は active `type_elaboration` runner を通じて extraction-gap boundary として
だけ実行可能である。deferred `formula_statement` runner obligation を満たさず、
statement proof payload、nested proof skeleton payload、local proof context、
formula payload、label-reference semantic checking、fact、theorem acceptance、
CoreIr、ControlFlowIr、VC、proof payload は credit しない。上の row の
theorem/proof boundary entry は task 86 の formula-only case、task 87 の
term-bearing equality case、task 88 の proof-block case、task 89 の
statement-proof case を含む。

task 90 も同じ row を refine する。predicate definition と functor definition
source は active `type_elaboration` runner を通じて extraction-gap boundary として
だけ実行可能である。deferred `formula_statement` runner obligation を満たさず、
definition declaration payload、definition-local context、definiens formula/term
payload、overload payload、fact、CoreIr、ControlFlowIr、VC、proof payload は
credit しない。

task 91 も同じ row を refine する。attribute definition source は active
`type_elaboration` runner を通じて extraction-gap boundary としてだけ実行可能である。
deferred `formula_statement` runner obligation を満たさず、definition declaration
payload、definition-local context、formula-definiens payload、attributed-type
evidence、fact、CoreIr、ControlFlowIr、VC、proof payload は credit しない。

task 92 も同じ row を refine する。mode definition と structure definition source は
active `type_elaboration` runner を通じて extraction-gap boundary としてだけ実行可能である。
deferred `formula_statement` runner obligation を満たさず、definition declaration
payload、mode expansion、structure base-shape / constructor / selector evidence、
definition-local context、fact、CoreIr、ControlFlowIr、VC、proof payload は
credit しない。

task 93 も同じ row を refine する。proof-local declaration statement は active
`type_elaboration` runner を通じて extraction-gap boundary としてだけ実行可能である。
deferred `formula_statement` runner obligation を満たさず、proof-local declaration
payload、local proof context、formula/term payload、RHS term inference、reconsider
coercion / obligation evidence、fact、theorem acceptance、CoreIr、ControlFlowIr、
VC、proof payload は credit しない。

task 94 も同じ row を refine する。proof-local `deffunc` と `defpred` inline
definition は active `type_elaboration` runner を通じて extraction-gap boundary
としてだけ実行可能である。deferred `formula_statement` runner obligation を満たさず、
inline definition formal/body payload、local abbreviation expansion、term/formula
body payload、guard evidence、fact、theorem acceptance、CoreIr、ControlFlowIr、VC、proof payload は
credit しない。

task 95 も同じ row を refine する。existential cluster と conditional cluster を
含む top-level registration block は active `type_elaboration` runner を通じて
extraction-gap boundary としてだけ実行可能である。deferred `formula_statement`
または `advanced_semantics` runner obligation を満たさず、registration-item
payload、correctness-condition / proof-obligation payload、accepted activation /
evidence status、cluster / reduction semantics、Chapter 17 semantic row、fact、
CoreIr、ControlFlowIr、VC、proof payload は credit しない。

task 96 も同じ row を refine する。top-level と definition-local の synonym /
antonym alias、および attribute、predicate、functor redefinition declaration は
active `type_elaboration` runner を通じて extraction-gap boundary としてだけ
実行可能である。deferred `formula_statement` または `advanced_semantics`
runner obligation を満たさず、redefinition payload、notation alias relation
payload、target inference、coherence proof-obligation payload、overload
candidate payload、Chapter 11 alias semantic resolution、Chapter 19 overload /
redefinition semantics、fact、CoreIr、ControlFlowIr、VC、proof payload は credit
しない。

task 81 addendum: `type_elaboration` runner は argument-bearing local attribute
extraction-gap boundary も所有する。`param_prefix` 構文で宣言され、
reserve type expression 内で `attribute_name(args)` として使われる same-module
parameterized attribute を実行してよいが、real term-argument provenance と checker
`AttributeInput` argument payload extraction が存在するまでは
`type_elaboration.external_dependency.ast_payload_extraction` に留めなければならない。
この runner support は attributed-type evidence、positive parameterized attribute
elaboration、CoreIr、ControlFlowIr、VC、proof payload を credit しない。

task 82 addendum: `type_elaboration` runner は、documented
`parser.type_fixtures` import summary 由来の imported mode reserve head を、
resolver `SymbolEnv` が `SymbolKind::Mode` と `ImportedSource` contribution を
記録している場合に checker-owned symbol head として渡してよい。`TypeCaseMode`
の expected active diagnostic は
`type_elaboration.checker.checker.type.external.mode_expansion_payload` になる。
runner は引き続き imported module AST extraction、`ModeExpansion` payload、
positive imported mode elaboration、CoreIr、ControlFlowIr、VC、proof payload を
合成してはならない。

## Algorithm / Logic

1. `layout` を通して、known payload roots `miz`、`lexical`、
   `certificates`、`generated`、`fuzz`、`property`、`stress`、`snapshots`
   配下の tests を discover する。
2. 発見したすべての sidecar を parse/validate したうえで、
   `DiscoveryConfig.profile` で filter された `cases` を持つ canonical
   `TestPlan` を構築する。`profiles` が無い場合は `["fast"]` が default で、
   `Full` は valid に parse された全 case を含む。duplicate id、traceability
   link、diagnostics は filtered case だけでなく parse 済み sidecar 全体で
   check する。
3. `parse-only` では、`stage = "parse_only"`、`expected_phase = "parse"`、
   `.miz` payload、pass/fail outcome、`tags = ["active_parse_only"]` を持つ
   case だけを選ぶ。tag のない parse-only sidecar は discovery と traceability
   metadata のままにする。
4. `declaration-symbol` では、`stage = "declaration_symbol"`、
   `expected_phase = "resolve"`、`.miz` payload、pass/fail outcome、
   `tags = ["active_declaration_symbol"]` を持つ case だけを選ぶ。tag のない
   declaration-symbol sidecar は discovery と traceability metadata のままにする。
5. `type-elaboration` では、`stage = "type_elaboration"`、
   `expected_phase = "type_check"`、`.miz` payload、pass/fail outcome、
   `tags = ["active_type_elaboration"]` を持つ case だけを選ぶ。tag のない
   type-elaboration sidecar は discovery と traceability metadata のままにする。
6. execution が parallel でも deterministic display order で cases を run する。
7. compiler outputs を structured records として capture する。
8. snapshot expectations より先に pass/fail expectations を match する。
9. general `[[snapshots]]` entries は canonical hash で compare する。現在の
   parse-only `SurfaceAst` shortcut は後述の通り、commit 済み text baseline を
   byte-for-byte で比較する。
10. phase、failure category、rejection reason、diagnostic code、snapshot diff summary 付きで failures を report する。

現在の parse-only runner は、各 active corpus file を一時的な `src/` package に
copy し、実際の frontend parser seam を実行する。pass case では AST が生成され、
assertion 対象の diagnostics がないことを要求する。fail case では、期待値を bare
syntax diagnostic key と比較する。この syntax-only mode では、runner は frontend の
各 import stub を、一致する `stub_ordinal` と `stub_span` を持つ
`ResolvedImportEntry` に解決する harness provider を使う。さらに distinct な
module id ごとに `ModuleLexicalSummary` を 1 つ返す。summary は、狭い
`parser.type_fixtures` fixture module を除き exported symbol を含まない。この
fixture module だけは、type-expression と operator syntax fixtures に必要な
parser-owned attribute、mode、structure、predicate、functor shape を注入する。
その他の import summary は symbol を export せず、summary は import 構文ケースが
意味的な module availability に依存しないようにするためだけに存在する。
parser syntax diagnostic と syntax 以外の frontend recovery
diagnostic が同時に存在する場合、sidecar が明示的に
`allow_frontend_recovery_diagnostics` を含めていない限り、runner はすべての
diagnostic code を report する。active parse-only の pass/fail sidecar は、移行用の
`snapshots = "snapshots/parser/<id>.surface_ast.snap"` field も設定してよい。その場合、
diagnostics が一致した後、runner は `SurfaceAst` を要求し、`SurfaceAst::snapshot_text()`
を `tests/snapshots/` 配下の commit 済み baseline と比較する。snapshot baseline は
通常の parse-only run では rewrite されない。

`active_parse_only` tag を持つ expectation が runnable case predicate のいずれかを
満たさない場合、runner は silent skip ではなく harness error として扱う。

現在の declaration-symbol runner は、各 active `.miz` corpus file を同じ一時的な
package 形状へ copy し、実際の frontend を実行したうえで、得られた
`SurfaceAst` を resolver の declaration-shell collector、parser-backed signature
projection extractor、symbol collector に渡す。pass case は frontend assertion
diagnostic と resolver symbol diagnostic がどちらも無いことを要求し、
`declaration_symbol_payloads` が存在する場合は、期待 key と SymbolEnv 由来の
symbol / definition fact key の exact sorted list も比較する。fail case は、
`diagnostic_payloads` が存在する場合はそれを、無い場合は `stable_detail_key` を
使って、resolver の crate-local internal detail key と比較する。diagnostic-code
ownership gap が open の間、この runner は public resolver diagnostic code を要求せず、
創作もしない。non-empty `diagnostic_codes` を持つ active declaration-symbol
expectation は harness error である。

`active_declaration_symbol` tag を持つ expectation が runnable case predicate の
いずれかを満たさない場合、runner は silent skip ではなく harness error として扱う。

現在の type-elaboration runner は、各 active `.miz` corpus file を同じ一時的な
package 形状へ copy し、実際の frontend を実行したうえで、得られた
`SurfaceAst` を resolver の declaration-shell collector、parser-backed signature
projection extractor、symbol collector に渡す。これにより checker payload extraction
へ進む前に lower-stage prerequisite を正直に確認する。

lower stages が pass した後、runner は syntax-free な reserve declaration
payload を、unrecovered な reserve source のうち reserve segment が 1 個以上の
identifier と対応済み reserve type-expression head を持つものから抽出する。
successful pass case は引き続き attribute、argument、parameter prefix、non-builtin
symbol head を含まない bare builtin `set` / `object` shape に限る。ただし task 55 は、
runner が unique / unrecovered / preceding / same-module / no-argument `ModeDefinition`
から、definition-local context を持たず bare builtin `set` / `object` RHS を持つ real
`ModeExpansion` を導ける bare local-mode reserve head について、2 つ目の pass slice を追加する。
task 56 は reserve head が、accepted task-55 bare builtin RHS expansion を持つ preceding
same-module no-argument local mode へ expand する場合の one-edge same-module local-mode
chain まで、この pass slice を拡張する。task 72 は terminal expansion が builtin
`set` / `object` である場合に、この pass slice を two-edge bare local-mode chain
へ拡張し、task 73 は three-edge へ拡張する。task 74 はその一時的な depth guard を、
terminal expansion が正確に builtin `set` / `object` である bare same-module
no-argument local-mode chain の AST-bounded structural rule に置き換える。この
structural guard に違反する chain は引き続き
`checker.type.external.mode_expansion_payload` を出す。task 50 は active
fail slice を 1 つ追加する: resolver declaration/symbol collection がすでに `SymbolEnv`
に入れた same-module attribute symbol は builtin reserve type payload に attach してよく、
checker declaration checking はより広い AST payload extraction gap ではなく
`checker.declaration.deferred.evidence_query` を出す。task 51 は 2 つ目の active fail
slice を追加する: attribute や type argument を持たない unique な same-module local mode
symbol は reserve type head として使ってよく、task 55 の bare expansion slice、
task 56 の one-edge chain slice、task 74 の structural bare chain slice のいずれも適用されない場合、checker type normalization は
`checker.type.external.mode_expansion_payload` を出す。task 52 は 3 つ目の active fail slice を追加する:
attribute や type argument を持たない unique な same-module local structure symbol は reserve
type head として使ってよく、real base-shape / constructor-witness evidence extraction が
まだ無いため checker declaration checking は
`checker.declaration.deferred.evidence_query` を出す。task 53 は 4 つ目の active fail
slice を追加する: same-module source-derived attribute をその local structure head に attach
してよく、第 17 章が full normalized attributed type の existential evidence を要求するため、
引き続き `checker.declaration.deferred.evidence_query` を出す。task 54 は 5 つ目の
active fail slice を追加する: same-module source-derived attribute を
same-module local mode reserve head に attach してよく、supported real expansion がない場合や
同じ mode が bare reserve use と mixed の場合は引き続き
`checker.type.external.mode_expansion_payload` を出す。runner は mixed attributed/bare
local-mode source では task-55/task-56/task-72 expansion を渡さない。task 56 は attributed dependency を持つ
local-mode chain では chain 全体を withheld し、partial `B -> A` payload を挿入せずに
同じ missing mode-expansion diagnostic へ到達する active fail slice も追加する。
task 57 は別の active fail slice を追加する: same-module no-argument local mode
definition が bare same-module local structure RHS を持つ場合、runner は real expansion
payload を checker seam に渡す。checker は expanded structure radix の source-derived
base-shape / constructor-witness evidence をまだ持たないため、
`checker.type.external.mode_expansion_payload` ではなく
`checker.declaration.deferred.evidence_query` に到達する。
task 58 は parallel な attributed-builtin RHS diagnostic slice を追加する:
same-module no-argument local mode definition が attributed builtin RHS を持つ場合、
runner は real expansion payload を checker seam に渡す。checker は source-derived
attributed-type existential evidence をまだ持たないため、
`checker.type.external.mode_expansion_payload` ではなく
`checker.declaration.deferred.evidence_query` に到達する。
task 59 は対応する attributed local-mode reserve diagnostic slice を追加する:
same-module attributed local-mode reserve head が real direct bare-builtin RHS expansion を持ち、
同じ mode が bare reserve use と mixed でない場合、runner はその expansion payload を
checker seam に渡す。checker は source-derived attributed-type existential evidence をまだ持たないため、
`checker.type.external.mode_expansion_payload` ではなく
`checker.declaration.deferred.evidence_query` に到達する。
task 60 は direct attributed local-mode structure-RHS diagnostic slice を追加する:
same-module attributed local-mode reserve head が real direct local-structure RHS expansion を持ち、
同じ mode が bare reserve use と mixed でない場合、runner はその expansion payload を
checker seam に渡す。checker は source-derived base-shape / constructor-witness evidence と
full attributed-type evidence をまだ持たないため、`checker.type.external.mode_expansion_payload`
ではなく `checker.declaration.deferred.evidence_query` に到達する。
task 61 は direct attributed local-mode attributed-builtin-RHS diagnostic slice を追加する:
same-module attributed local-mode reserve head が real direct attributed-builtin RHS expansion を持ち、
同じ mode が bare reserve use と mixed でない場合、runner はその expansion payload を
checker seam に渡す。checker は source-derived full attributed-type evidence をまだ持たないため、
`checker.type.external.mode_expansion_payload` ではなく
`checker.declaration.deferred.evidence_query` に到達する。
task 62 は one-edge bare local-mode structure-RHS chain diagnostic slice を追加する:
`A is B` と `B is LocalStruct` が unique / unrecovered / same-module / no-argument mode
definition で、unique local structure definition の後かつ reserve use の前という source order を満たす場合、
runner は両方の real expansion payload を checker seam に渡す。checker は source-derived
base-shape / constructor-witness evidence をまだ持たないため、
`checker.type.external.mode_expansion_payload` ではなく
`checker.declaration.deferred.evidence_query` に到達する。
task 63 は one-edge bare local-mode attributed-builtin-RHS chain diagnostic slice を追加する:
`A is B` と terminal attributed-builtin mode definition（`B is marked set` または
`B is marked object`）が unique / unrecovered / same-module / no-argument mode
definition で、reserve use より前という source order を満たし、
RHS attributes が argument-free same-module attributes である場合、runner は両方の
real expansion payload を checker seam に渡す。checker は source-derived
attributed-type existential evidence をまだ持たないため、
`checker.type.external.mode_expansion_payload` ではなく
`checker.declaration.deferred.evidence_query` に到達する。
task 64 は one-edge attributed-root bare-builtin chain diagnostic slice を追加する:
`reserve z for marked A` が、reserve use より前に source order を満たす unique /
unrecovered / same-module / no-argument な `B is set` / `object` と `A is B`
definition を使い、`A` が bare reserve use と mixed でなく、`B` が attributed reserve
head でない場合、runner は両方の real expansion payload と reserve-head attribute を
checker seam に渡す。checker は source-derived attributed-type existential evidence を
まだ持たないため、`checker.type.external.mode_expansion_payload` ではなく
`checker.declaration.deferred.evidence_query` に到達する。
task 65 は structure-RHS counterpart を追加する: `reserve z for marked A` が、
unique same-module `LocalStruct` definition の後かつ reserve use より前に source order を
満たす unique / unrecovered / same-module / no-argument な `B is LocalStruct` と
`A is B` definition を使い、`A` が bare reserve use と mixed でなく、`B` が attributed
reserve head でない場合、runner は両方の real expansion payload と reserve-head attribute
を checker seam に渡す。checker は source-derived structure base-shape /
constructor-witness evidence と full attributed-type existential evidence をまだ持たないため、
`checker.type.external.mode_expansion_payload` ではなく
`checker.declaration.deferred.evidence_query` に到達する。
task 66 は attributed-builtin-RHS counterpart を追加する: `reserve z for marked A` が、
reserve use より前に source order を満たす unique / unrecovered / same-module /
no-argument な `B is marked set` / `marked object` と `A is B` definition を使い、
`A` が bare reserve use と mixed でなく、`B` が attributed reserve head でなく、
RHS attributes が argument-free same-module attributes である場合、runner は両方の
real expansion payload、reserve-head attribute、terminal RHS attribute を checker seam に
渡す。checker は source-derived full attributed-type existential evidence をまだ持たないため、
`checker.type.external.mode_expansion_payload` ではなく
`checker.declaration.deferred.evidence_query` に到達する。
task 67 は structure-qualified attribute boundary を追加する:
`LocalStruct.marked LocalStruct` のような reserve type expression は parser/resolver
executable だが、checker payload が real structure-qualifier と attribute-owner
provenance を保持するまで、runner は
`type_elaboration.external_dependency.ast_payload_extraction` に残さなければならない。
runner はこの reference を unqualified same-module attribute payload に書き換えては
ならない。
task 68 は argument-bearing mode boundary を追加する:
`Element of a` のような reserve type expression は same-module local mode surface として
parser/resolver executable だが、checker payload が real type-argument と
term-argument provenance を保持するまで、runner は
`type_elaboration.external_dependency.ast_payload_extraction` に残さなければならない。
runner はこの reference を argument-free mode payload に書き換えたり、arity matching、
mode expansion、positive type elaboration を主張したりしてはならない。
task 69 は対応する argument-bearing structure boundary を追加する:
`LocalStruct of a` のような reserve type expression は same-module structure declaration が
`of` parameter surface を持つ場合 parser/resolver executable だが、checker payload が real
type-argument と term-argument provenance を保持するまで、runner は
`type_elaboration.external_dependency.ast_payload_extraction` に残さなければならない。
runner はこの reference について structure argument payload extraction、arity matching、
base-shape evidence、positive structure type elaboration を主張してはならない。
task 70 は bracket-form local mode boundary を追加する:
same-module bracket-parameter mode declaration と `Family[set]` のような
bracket-form reserve type head を含む source は parser/resolver executable だが、
checker payload が real bracket type-argument と `qua`-argument provenance を保持するまで、
runner は `type_elaboration.external_dependency.ast_payload_extraction` に残さなければならない。
runner はこの source について bracket payload extraction、mode-head resolution、arity matching、
mode expansion、positive type elaboration を主張してはならない。
task 71 は bracket-form local structure boundary を追加する:
same-module bracket-parameter structure declaration と `LocalStruct[set]` のような
bracket-form reserve type head を含む source は parser/resolver executable だが、
checker payload が real bracket type-argument と `qua`-argument provenance を保持するまで、
runner は `type_elaboration.external_dependency.ast_payload_extraction` に残さなければならない。
runner はこの source について bracket payload extraction、structure-head resolution、
arity matching、base-shape / constructor-witness evidence、positive structure type
elaboration を主張してはならない。
task-82 `TypeCaseMode`、task-83 `R`、task-84 `TypeCaseAttr`、task-85 negative
`empty`/builtin-`set` bridge、task-116 positive `empty`/builtin-`set` bridge 外の broader imported attribute / mode / structure、
unresolved / ambiguous symbol、attribute argument、qualified
attribute disambiguation のうち task-67 boundary を超えるもの、mode / structure
argument、type-argument / term-argument / bracket `type_arg_list` / `qua`-argument
provenance、parameterized / contextual mode definition、
task-62 bare chain slice 外の attributed structure RHS、task-60 direct attributed-root slice と
task-62 bare chain slice と task-65 attributed-root chain slice 外の structure-RHS chain、
task-58/task-61 direct slice 外の attributed-RHS chain、
task-63 bare chain slice 外の attributed-RHS chain、
task-66 attributed-root chain slice 外の attributed-root attributed-RHS chain、
forward-reference または cyclic local-mode chain、
non-reserve declaration はこの source bridge の外に残る。

抽出された payload について、runner は source/module identity、reserve source range、
binding spelling/range、対応済み type-expression spelling/range/head、対応済み
same-module attribute の symbol/range/polarity、対応済み same-module local-mode expansion
payload（task-57 terminal local-structure RHS diagnostic slice、task-58 terminal
attributed-builtin RHS diagnostic slice、task-59 attributed local-mode reserve diagnostic
slice、task-60 attributed local-mode structure-RHS diagnostic slice、task-61 attributed
local-mode attributed-builtin-RHS diagnostic slice、task-62 local-mode structure-RHS chain
diagnostic slice、task-63 local-mode attributed-RHS chain diagnostic slice、task-64
attributed local-mode bare-builtin chain diagnostic slice、task-65 attributed local-mode
structure-RHS chain diagnostic slice、task-66 attributed local-mode attributed-builtin-RHS
chain diagnostic slice を含む）を
`mizar-checker` の source reserve declaration seam に渡す。その checker-owned seam は reserve binding を含む module
`BindingEnv`、binding ごとの `DeclarationInput`、binding 固有の
`TypeExpressionInput` site を構築し、`reserve x, y for set` は source range を共有しつつ
binding ごとに distinct typed site を持ち、collected `SymbolEnv` に対して
`DeclarationChecker` を実行する。runner は active fail slice の stable diagnostic key を
集めるために、同じ checker-owned assembly helper を使ってよい。checker diagnostic が
出た場合、active fail case はその key を比較し、runner は downstream readiness assertion
として credit しない。diagnostic-free な対応済み output（bare builtin と task-55/56/72
local-mode expansion slice を含む）について、返された checker handoff は declaration と
type-entry link を持つ checker-owned `TypedAst`、および empty-but-real な cluster/overload
predecessor output と source-preserved node hint / declaration expression metadata により
投影された checker-owned `ResolvedTypedAst` として credit される。
runner はその real `ResolvedTypedAst` payload を `mizar-core` の
`ResolvedTypedAstSummary::from_ast` に渡し、successful reserve-only slice について
summary が source/module identity を保ち、checker recovery/diagnostic site を持たない
ことを確認する。さらに同じ real reserve binding から binder-only
`CoreContextInput` を準備し、抽出済み binding ごとに 1 個の `CoreVariableSeed` と
`CoreBinderSeed` を与え、`CoreItemSeed` は渡さず、source/module identity、binder
source range、checker provenance、empty item registry、empty core diagnostics、
empty core worklist を確認する。これは summary/context readiness の確認だけであり、
`CoreIr`、`ControlFlowIr`、obligation seed、VC、proof row は構築しない。
active pass case は、source が少なくとも 1 個の対応済み reserve binding を持ち、runner
regression evidence が checker handoff construction、declaration checking、`TypedAst`
assembly、`ResolvedTypedAst` assembly、summary-readiness、binder-only core context
readiness の実行を確認する場合だけ、
この対応済み source-derived slice を empty detail key で assert してよい。

runner は不足している AST-wide source-to-checker bridge を引き続き捏造しない。
non-builtin declaration、task-84 `TypeCaseAttr` bridge、task-85 negative
`empty`/builtin-`set` bridge、task-116 positive `empty`/builtin-`set`
bridge、task-80 boundary を超える imported attribute、
task-83 `R` bridge、task-97 `TypeCaseStruct` bridge、と task-78 boundary を超える imported structure、task 82 の provenance/type-head
bridge を超える imported mode expansion、attribute argument、mode / structure
argument、qualified attribute provenance、type-argument / term-argument
provenance、bracket `type_arg_list` / `qua`-argument provenance、structure base-shape evidence、
task 固有 theorem bridge を超える term/formula payload、task 112 を超える formula
child/binder semantics、coercion site、overload evidence、recorded fact、CoreIr、
ControlFlowIr、VC payload、proof evidence は対応済み extraction slice の外に残る。
active case が未対応 source-to-checker payload family を必要とする場合、runner は
stable detail key `type_elaboration.external_dependency.ast_payload_extraction`、または
task 固有 exact bridge では checker-owned fail-closed diagnostic key を report する。
active fail case はそれらの key を `diagnostic_payloads` または `stable_detail_key` で
assert してよい。対応済み slice の外にある active pass case は stub で pass させず
deferred のままにする。この runner は `CoreIr`、`ControlFlowIr`、VC seed、proof row、
public checker diagnostic code を publish しない。

public checker diagnostic code が指定されるまで、non-empty `diagnostic_codes` を持つ
active type-elaboration expectation は harness error である。
`active_type_elaboration` tag を持つ expectation が runnable case predicate の
いずれかを満たさない場合、runner は silent skip ではなく harness error として扱う。

上記の general snapshot と determinism runner rows は target-state harness modes である。
task 4 と task 5 は shared `SnapshotRecord`、baseline verify/update、
repeat-render comparison API を提供するが、この harness はまだ general
`[[snapshots]]` sidecar entries を parse せず、general snapshot/update subcommand も
実行しない。active parse-only `SurfaceAst` shortcut が runner execution に接続済みの
唯一の snapshot path である。

Core Task 31は最後の文にexactな例外を1件だけ追加する。active Task-180
contradiction pass caseはexisting singular `snapshots` fieldでfixed
`CoreIr::debug_text()` baselineを参照してよい。runnerはexact CoreIrを2回constructし、
structural/debug-text equalityを要求した後、committed baselineとverify-only byte
comparisonを行う。general CoreIr payloadやsnapshot update commandは公開しない。
その他すべてのCoreIr/ControlFlowIr caseとgeneral snapshot registryはunwired/deferred
のままとする。

architecture-22 matrix support は task 14 では metadata/reporting-only である。
metadata plan は `architecture22_scenarios`、
`architecture22_equivalence_class`、`architecture22_gate` を validate し、
required scenario ごとに registry class と planned/active count を report する。
task 14 の scenario row はすべて active eligibility を持たないため、将来の
consumer-specific increment が real clean/incremental/parallel/cache-race execution を
配線するまで、`architecture22_gate = "active"` は reject される。

## Determinism Requirements

harness は identical inputs が次を生成することを check する。

- identical artifact hashes
- identical snapshot hashes
- identical diagnostic order
- identical failure records
- identical proof status
- identical dependency slices

parallel execution は runtime を変えてよいが、observable results を変えてはならない。

task 11 の implemented coverage は、metadata plan と active runner report を
deterministic byte strings に render し、repeated build/run を比較する。
snapshot-level determinism と parallel equivalence は general snapshot record helper
で cover する。active parallel runner subcommands は、consumer crate が parallel
execution を公開するまで future work のままである。

## Reporting

reports は次を区別する。

- unexpected success
- unexpected failure
- wrong failure category
- wrong rejection reason
- diagnostic order mismatch
- snapshot mismatch
- nondeterminism across repeated runs
- harness infrastructure error

## Tests

key scenarios:

- fail test が unexpected pass する
- pass test が error diagnostic を emit する
- snapshot hash が異なる
- metadata plan bytes が repeated build 間で異なる
- active runner report bytes が repeated run 間で異なる
- repeated run が異なる diagnostic order を生成する
- generic snapshot parallel equivalence が sequential snapshot generation と同じ
  observable artifact を生成する
- architecture-22 matrix metadata が required scenario ids をすべて planned として
  report し、owning consumer runner が存在する前の fake active row を reject する

## Constraints and Assumptions

- test execution order は semantic ordering ではない。
- harness は cache hits を検証対象の compiler behavior として扱い、proof authority としては扱わない。
- snapshot update mode は opt-in であり command output に見える形でなければならない。


## task 201 immediate-radix asserted-head harness contract

task 201 route は exact である。builtin `set` で終わる labeled/ordered bare mode definition 2 個、outer mode の `x` reserve 1 個、Base-mode formula-side type assertion 1 個だけを受理する。closed relation は builtin/same-mode route を isolate し、asserted resolved symbol と outer binding expansion の real immediate radix を比較する。harness は missing/reordered/extra/deeper/recovered/contextual/parameterized/argument-bearing/attributed definition、non-exact reserve/theorem、builtin/same-outer/object/unrelated/imported/ambiguous asserted head、独立した expansion/binding/ordinal/head/spelling/site/range/immediate-edge/canonical corruption を reject する。immutable positive output と real frontend/resolver sidecar が active runner 149 を保護する。general reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR は activate しない。


## task 202 object immediate-radix harness contract

task 202 route は labeled/ordered bare object-mode definition 2 個、outer reserve 1 個、immediate Base radix の formula assertion 1 個だけを受理する。structural/provenance near miss、追加 set-terminal/object-chain shape、独立 payload/`BuiltinSet` corruption、unresolved/imported/ambiguous head を reject する。real owning-positive 後の Task202-negative check が Tasks 147/185/201 を isolate し、Task202 exact source も各 owning route から reject される。immutable output と real frontend/resolver sidecar が active runner 150 を保護する。coercion、reachability、acceptance、truth/fact、proof、downstream IR は activate しない。


## task 203 two-edge immediate-radix harness contract

task 203 route は labeled/ordered/bare set-terminal mode definition 3 個、Outer reserve 1 個、immediate Middle radix の formula assertion 1 個だけを受理する。全 nonidentity definition order、duplicate/misspelled definition、direct/one-edge/object/deeper shape、imported/ambiguous Base/Middle/Outer provenance、独立した expansion/binding/ordinal/head/site/range/immediate-edge/`BuiltinObject`/canonical corruption を reject する。bidirectional real-route check が Tasks 122/148/149/186/187/201/202 を isolate する。immutable output と real frontend/resolver sidecar が active runner 151 を保護する。harness は two-hop reachability、Base assertion、coercion、acceptance、truth/fact、proof、downstream IR を activate しない。


## task 204 two-edge object immediate-radix harness contract

task 204 route は labeled/ordered/bare object-terminal mode definition 3 個、Outer reserve 1 個、immediate Middle radix の formula assertion 1 個だけを受理する。全 nonidentity definition order、duplicate/misspelled definition、direct/one-edge/set-terminal/deeper shape、imported/ambiguous Base/Middle/Outer provenance、独立した expansion/binding/ordinal/head/site/range/immediate-edge/`BuiltinSet`/canonical corruption を reject する。bidirectional real-route check が Tasks 189/145/147/149/187/202 および set Tasks 148/186/203 を isolate する。immutable output と real frontend/resolver sidecar が active runner 152 を保護する。harness は object/set coercion、two-hop reachability、Base assertion、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 205 three-edge set immediate-radix harness contract

task 205 route は labeled/ordered/bare set-terminal mode definition 4 個、Outer reserve 1 個、immediate Middle radix の formula assertion 1 個だけを受理する。全 23 nonidentity definition order、missing/duplicate/mislabeled/misspelled/wrong-radix definition、direct/one-edge/two-edge/object-terminal/deeper shape、multi-hop Inner/Base assertion、imported/ambiguous Base/Inner/Middle/Outer provenance、独立した expansion/binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinObject`/canonical corruption を reject する。bidirectional real-route check が set Tasks 122/138/146/148/150/195/201/203 および object Tasks 189/145/147/149/151/196/202/204 を isolate する。immutable output と real frontend/resolver sidecar が active runner 153 を保護する。harness は multi-hop reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 206 three-edge object immediate-radix harness contract

task 206 route は labeled/ordered/bare object-terminal mode definition 4 個、Outer reserve 1 個、immediate Middle radix の formula assertion 1 個だけを受理する。全 23 nonidentity definition order、missing/duplicate/mislabeled/misspelled/wrong-radix definition、direct/one-edge/two-edge/set-terminal/deeper shape、multi-hop Inner/Base、builtin、local-other、argument-bearing、attributed assertion、imported/ambiguous Base/Inner/Middle/Outer provenance、独立した expansion/binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinSet`/canonical corruption を reject する。bidirectional real-route check が set Tasks 122/138/146/148/150/195/201/203/205 および object Tasks 189/145/147/149/151/196/202/204 を isolate する。immutable output と real frontend/resolver sidecar が active runner 154 を保護する。harness は object/set coercion、multi-hop reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 207 four-edge set immediate-radix harness contract

task 207 route は labeled/ordered/bare set-terminal mode definition 5 個、TooDeep reserve 1 個、immediate Outer radix の formula assertion 1 個だけを受理する。全 119 nonidentity definition order、全 missing/duplicate/mislabeled/misspelled/wrong-radix/recovered/contextual/parameterized/argument-bearing/attributed definition、shorter/object-terminal/connected deeper shape、same-TooDeep、multi-hop Middle/Inner/Base、builtin、local-other、argument-bearing、attributed assertion、全 symbol の imported/ambiguous provenance、独立した expansion/binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinObject`/canonical corruption を reject する。bidirectional real-route check が declared set owner 10 件と object owner 10 件を isolate する。immutable output と real frontend/resolver sidecar が active runner 155 を保護する。harness は multi-hop reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 208 four-edge object immediate-radix harness contract

task 208 route は labeled/ordered/bare object-terminal mode definition 5 個、TooDeep reserve 1 個、immediate Outer radix の formula assertion 1 個だけを受理する。全 119 nonidentity order、全 per-definition structural near miss、non-exact reserve/formula shape、shorter/set-terminal/connected deeper chain、same-TooDeep、multi-hop Middle/Inner/Base、builtin object/set、local-other、argument-bearing/attributed assertion、全 symbol の imported/ambiguous provenance、全 expansion removal、独立した payload/binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinSet`/canonical corruption を reject する。unrelated-import positive が over-rejection を防止する。bidirectional real-route check が declared set owner 11 件と object owner 10 件を isolate する。immutable output と real frontend/resolver sidecar が active runner 156 を保護する。harness は object/set coercion、multi-hop reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 209 seven-expansion set immediate-radix harness contract

task 209 route は labeled/ordered/bare set-terminal definition 7 個 `BaseMode -> set` から `ChainMode6 -> ChainMode5`、ChainMode6 reserve 1 個、immediate ChainMode5 assertion 1 個だけを受理する。全 5,039 nonidentity order、各 definition の missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variant、non-exact/multi-binding reserve、non-exact formula、same/multi-hop/builtin/local-other/argument-bearing/attributed asserted head、connected eighth edge、全7 symbol の imported/ambiguous provenance、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinObject`/canonical corruption を reject する。unrelated-import positive が over-rejection を防止する。bidirectional check は Task 209 実装前の owner route 34 件すべてを isolate し、immutable-output check は mutation を防ぎ、real frontend/resolver sidecar が active runner 157 を保護する。harness は multi-hop reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 210 seven-expansion object immediate-radix harness contract

task 210 route は labeled/ordered/bare object-terminal definition 7 個 `BaseObjectMode -> object` から `ChainObjectMode6 -> ChainObjectMode5`、ChainObjectMode6 reserve 1 個、immediate ChainObjectMode5 assertion 1 個だけを受理する。全 5,039 nonidentity order、各 definition の missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variant、non-exact/multi-binding reserve、non-exact formula、same/multi-hop/builtin/local-other/argument-bearing/attributed asserted head、connected eighth edge、全7 symbol の imported/ambiguous provenance、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinSet`/canonical corruption を reject する。unrelated-import positive が over-rejection を防止する。bidirectional check は Task 210 実装前の owner route 35 件すべてを isolate し、immutable-output check は mutation を防ぎ、real frontend/resolver sidecar が active runner 158 を保護する。harness は object/set coercion、multi-hop reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 211 two-edge set two-hop asserted-head harness contract

task 211 route は labeled/ordered/bare set-terminal definition 3 個 `BaseTwoHopModeAssertedHead -> set`、`MiddleTwoHopModeAssertedHead -> BaseTwoHopModeAssertedHead`、`OuterTwoHopModeAssertedHead -> MiddleTwoHopModeAssertedHead`、Outer reserve 1 個、Base assertion 1 個だけを受理する。real link 2 本を明示検証し、全5 nonidentity order、各 definition の missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variant、non-exact reserve/formula、same-Outer/immediate-Middle/builtin/object/local-other/deeper asserted head、全3 symbol の imported/ambiguous provenance、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/two-link/terminal/canonical corruption を reject する。unrelated-import positive が over-rejection を防止する。bidirectional check は既存 owner route 36 件すべてを isolate し、immutable-output check は mutation を防ぎ、real frontend/resolver sidecar が active runner 159 を保護する。harness は generic reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 212 two-edge object two-hop asserted-head harness contract

task 212 route は labeled/ordered/bare object-terminal definition 3 個 `BaseTwoHopObjectModeAssertedHead -> object`、`MiddleTwoHopObjectModeAssertedHead -> BaseTwoHopObjectModeAssertedHead`、`OuterTwoHopObjectModeAssertedHead -> MiddleTwoHopObjectModeAssertedHead`、Outer reserve 1 個、Base assertion 1 個だけを受理する。real link 2 本を明示検証し、全5 nonidentity order、各 definition の missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variant、non-exact reserve/formula、same-Outer/immediate-Middle/builtin-object/builtin-set/local-other/deeper asserted head、全3 symbol の imported/ambiguous provenance、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/two-link/terminal/`BuiltinSet`/canonical corruption を reject する。unrelated-import positive が over-rejection を防止する。bidirectional check は既存 owner route 37 件すべてを isolate し、immutable-output check は mutation を防ぎ、real frontend/resolver sidecar が active runner 160 を保護する。harness は generic reachability、widening、`qua`、object/set coercion、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 213 three-edge set two-hop asserted-head harness contract

task 213 route は labeled/ordered/bare set-terminal definition 4 個 `BaseThreeEdgeModeTwoHopAssertedHead -> set`、`InnerThreeEdgeModeTwoHopAssertedHead -> BaseThreeEdgeModeTwoHopAssertedHead`、`MiddleThreeEdgeModeTwoHopAssertedHead -> InnerThreeEdgeModeTwoHopAssertedHead`、`OuterThreeEdgeModeTwoHopAssertedHead -> MiddleThreeEdgeModeTwoHopAssertedHead`、Outer reserve 1 個、Inner assertion 1 個だけを受理する。real relation link 2 本を明示検証し、terminal traversal は Inner-to-Base-to-set tail だけに使う。全23 nonidentity order、各 definition の missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variant、non-exact reserve/formula、same-Outer/immediate-Middle/full-distance-Base/builtin/object/local-other/deeper asserted head、全4 symbol の imported/ambiguous provenance、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/two-link/tail/terminal/canonical corruption を reject する。unrelated-import positive が over-rejection を防止する。bidirectional check は既存 owner route 38 件すべてを isolate し、Tasks 211/212 focused check は短い set/object route を保持し、immutable-output check は mutation を防ぎ、real frontend/resolver sidecar が active runner 161 を保護する。harness は generic reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 214 three-edge object two-hop asserted-head harness contract

task 214 route は labeled/ordered/bare object-terminal definition 4 個 `BaseThreeEdgeObjectModeTwoHopAssertedHead -> object`、`InnerThreeEdgeObjectModeTwoHopAssertedHead -> BaseThreeEdgeObjectModeTwoHopAssertedHead`、`MiddleThreeEdgeObjectModeTwoHopAssertedHead -> InnerThreeEdgeObjectModeTwoHopAssertedHead`、`OuterThreeEdgeObjectModeTwoHopAssertedHead -> MiddleThreeEdgeObjectModeTwoHopAssertedHead`、Outer reserve 1 個、Inner assertion 1 個だけを受理する。real relation link 2 本を明示検証し、terminal traversal は Inner-to-Base-to-object tail だけに使う。全23 nonidentity order、各 definition の missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variant、non-exact reserve/formula、same/immediate/full-distance/builtin/local-other/deeper asserted head、全4 symbol の imported/ambiguous provenance、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/two-link/tail/terminal/canonical corruption を reject する。unrelated-import positive が over-rejection を防止する。bidirectional check は既存 owner route 39 件すべてを isolate し、Tasks 211/212/213 focused check は短い route と set-terminal route を保持し、immutable-output check は mutation を防ぎ、real frontend/resolver sidecar が active runner 162 を保護する。harness は object/set coercion、generic reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 215 four-edge set two-hop asserted-head harness contract

task 215 route は labeled/ordered/bare set-terminal definition 5 個 `BaseFourEdgeModeTwoHopAssertedHead -> set`、`InnerFourEdgeModeTwoHopAssertedHead -> BaseFourEdgeModeTwoHopAssertedHead`、`MiddleFourEdgeModeTwoHopAssertedHead -> InnerFourEdgeModeTwoHopAssertedHead`、`OuterFourEdgeModeTwoHopAssertedHead -> MiddleFourEdgeModeTwoHopAssertedHead`、`TooDeepFourEdgeModeTwoHopAssertedHead -> OuterFourEdgeModeTwoHopAssertedHead`、TooDeep reserve 1 個、Middle assertion 1 個だけを受理する。TooDeep-to-Outer/Outer-to-Middle relation link を明示検証し、terminal traversal は Middle-to-Inner-to-Base-to-set tail だけに使う。全119 nonidentity order、各 definition の finite missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variant、non-exact reserve/formula、alternative asserted head、全5 symbol の imported/ambiguous provenance、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/relation-link/tail/terminal/canonical corruption を reject する。unrelated-import positive が over-rejection を防止する。bidirectional check は既存 owner route 40 件すべてを isolate し、Tasks 211-214 focused check は短い route と terminal sibling を保持し、immutable-output check は mutation を防ぎ、real frontend/resolver sidecar が active runner 163 を保護する。harness は object/set coercion、generic reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 216 four-edge object two-hop asserted-head harness contract

task 216 route は labeled/ordered/bare object-terminal definition 5 個 `BaseFourEdgeObjectModeTwoHopAssertedHead -> object`、`InnerFourEdgeObjectModeTwoHopAssertedHead -> BaseFourEdgeObjectModeTwoHopAssertedHead`、`MiddleFourEdgeObjectModeTwoHopAssertedHead -> InnerFourEdgeObjectModeTwoHopAssertedHead`、`OuterFourEdgeObjectModeTwoHopAssertedHead -> MiddleFourEdgeObjectModeTwoHopAssertedHead`、`TooDeepFourEdgeObjectModeTwoHopAssertedHead -> OuterFourEdgeObjectModeTwoHopAssertedHead`、TooDeep reserve 1 個、Middle assertion 1 個だけを受理する。TooDeep-to-Outer/Outer-to-Middle relation link を明示検証し、terminal traversal は Middle-to-Inner-to-Base-to-object tail だけに使う。全119 nonidentity order、各 definition の finite missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variant、non-exact reserve/formula、alternative asserted head、全5 symbol の imported/ambiguous provenance、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/relation-link/tail/terminal/canonical corruption を reject する。unrelated-import positive が over-rejection を防止する。bidirectional check は既存 owner route 41 件すべてを isolate し、Tasks 211-215 focused check は短い route と terminal sibling を保持し、immutable-output check は mutation を防ぎ、real frontend/resolver sidecar が active runner 164 を保護する。harness は object/set coercion、generic reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 217 three-edge set three-hop asserted-head harness contract

task 217 route は labeled/ordered/bare set-terminal definition 4 個 `BaseThreeEdgeModeThreeHopAssertedHead -> set`、`InnerThreeEdgeModeThreeHopAssertedHead -> BaseThreeEdgeModeThreeHopAssertedHead`、`MiddleThreeEdgeModeThreeHopAssertedHead -> InnerThreeEdgeModeThreeHopAssertedHead`、`OuterThreeEdgeModeThreeHopAssertedHead -> MiddleThreeEdgeModeThreeHopAssertedHead`、Outer reserve 1 個、Base assertion 1 個だけを受理する。Outer-to-Middle/Middle-to-Inner/Inner-to-Base relation link を明示検証し、terminal traversal は Base-to-set だけに使う。全23 nonidentity order、各 definition の finite missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variant、non-exact reserve/formula と alternative asserted head、全4 symbol の imported/ambiguous provenance、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/relation-link/terminal/canonical corruption を reject する。unrelated-import positive が over-rejection を防止する。bidirectional check は既存 owner route 42 件すべてを isolate し、Tasks 211-216 focused check は shorter route と terminal sibling を保持し、immutable-output check は mutation を防ぎ、real frontend/resolver sidecar が active runner 165 を保護する。harness は object/set coercion、generic reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 218 three-edge object three-hop asserted-head harness contract

task 218 route は labeled/ordered/bare object-terminal definition 4 個 `BaseThreeEdgeObjectModeThreeHopAssertedHead -> object`、`InnerThreeEdgeObjectModeThreeHopAssertedHead -> BaseThreeEdgeObjectModeThreeHopAssertedHead`、`MiddleThreeEdgeObjectModeThreeHopAssertedHead -> InnerThreeEdgeObjectModeThreeHopAssertedHead`、`OuterThreeEdgeObjectModeThreeHopAssertedHead -> MiddleThreeEdgeObjectModeThreeHopAssertedHead`、Outer reserve 1 個、Base assertion 1 個だけを受理する。Outer-to-Middle/Middle-to-Inner/Inner-to-Base relation link を明示検証し、terminal traversal は Base-to-object だけに使う。matrix は全23 nonidentity order、各 definition の finite missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variant、non-exact reserve/formula と same/immediate/two-hop/builtin/local-other/deeper asserted head、全4 symbol の imported/ambiguous provenance、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/relation-link/terminal/`BuiltinSet`/canonical corruption を reject する。unrelated-import positive が over-rejection を防止する。bidirectional check は既存 owner route 43 件すべてを isolate し、Tasks 211-217 focused check は shorter route と terminal sibling を保持し、immutable-output check は mutation を防ぎ、real frontend/resolver sidecar が active runner 166 を保護する。harness は object/set coercion、generic reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 219 four-edge set three-hop asserted-head harness contract

task 219 route は labeled/ordered/bare set-terminal definition 5 個 `BaseFourEdgeModeThreeHopAssertedHead -> set`、`InnerFourEdgeModeThreeHopAssertedHead -> BaseFourEdgeModeThreeHopAssertedHead`、`MiddleFourEdgeModeThreeHopAssertedHead -> InnerFourEdgeModeThreeHopAssertedHead`、`OuterFourEdgeModeThreeHopAssertedHead -> MiddleFourEdgeModeThreeHopAssertedHead`、`TooDeepFourEdgeModeThreeHopAssertedHead -> OuterFourEdgeModeThreeHopAssertedHead`、TooDeep reserve 1 個、Inner assertion 1 個だけを受理する。TooDeep-to-Outer/Outer-to-Middle/Middle-to-Inner relation link を明示検証し、terminal traversal は Inner-to-Base-to-set tail だけに使う。matrix は (a) unconnected unsupported deeper asserted head と (b) actual connected sixth-definition/sixth-edge asserted head を独立に reject し、さらに全119 nonidentity order、各 definition の finite missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variant、non-exact reserve/formula と same/immediate/two-hop/full-distance/builtin/local-other asserted head、全5 symbol の imported/ambiguous provenance、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/relation-link/terminal/`BuiltinObject`/canonical corruption を reject する。unrelated-import positive が over-rejection を防止する。bidirectional check は既存 owner route 44 件すべてを isolate し、Task 207 と Tasks 211-218 focused check は shorter route と terminal sibling を保持し、immutable-output check は mutation を防ぎ、real frontend/resolver sidecar が active runner 167 を保護する。harness は object/set coercion、generic reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 220 four-edge object three-hop asserted-head harness contract

task 220 route は labeled/ordered/bare object-terminal definition 5 個 `BaseFourEdgeObjectModeThreeHopAssertedHead -> object`、`InnerFourEdgeObjectModeThreeHopAssertedHead -> BaseFourEdgeObjectModeThreeHopAssertedHead`、`MiddleFourEdgeObjectModeThreeHopAssertedHead -> InnerFourEdgeObjectModeThreeHopAssertedHead`、`OuterFourEdgeObjectModeThreeHopAssertedHead -> MiddleFourEdgeObjectModeThreeHopAssertedHead`、`TooDeepFourEdgeObjectModeThreeHopAssertedHead -> OuterFourEdgeObjectModeThreeHopAssertedHead`、TooDeep reserve 1 個、Inner assertion 1 個だけを受理する。TooDeep-to-Outer/Outer-to-Middle/Middle-to-Inner relation link を明示検証し、terminal traversal は Inner-to-Base-to-object tail だけに使う。matrix は (a) unconnected unsupported deeper asserted head と (b) actual connected sixth-definition/sixth-edge asserted head を独立に reject し、さらに全119 nonidentity order、各 definition の finite missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variant、non-exact reserve/formula と same/immediate/two-hop/full-distance/builtin/local-other asserted head、全5 symbol の imported/ambiguous provenance、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/relation-link/terminal/`BuiltinSet`/canonical corruption を reject する。unrelated-import positive は over-rejection を防止する。bidirectional check は既存 owner route 45 件すべてを isolate し、Tasks 208 と 211-219 focused check は shorter route と terminal sibling を保持し、immutable-output check は mutation を防ぎ、real frontend/resolver sidecar は active runner 168 を保護する。harness は object/set coercion、generic reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 221 four-edge set four-hop asserted-head active harness contract

task 221 route は labeled/ordered/bare set-terminal definition 5 個 `BaseFourEdgeModeFourHopAssertedHead -> set`、`InnerFourEdgeModeFourHopAssertedHead -> BaseFourEdgeModeFourHopAssertedHead`、`MiddleFourEdgeModeFourHopAssertedHead -> InnerFourEdgeModeFourHopAssertedHead`、`OuterFourEdgeModeFourHopAssertedHead -> MiddleFourEdgeModeFourHopAssertedHead`、`TooDeepFourEdgeModeFourHopAssertedHead -> OuterFourEdgeModeFourHopAssertedHead`、TooDeep reserve 1 個、Base assertion 1 個だけを受理する。relation link 4 本を明示検証し、terminal traversal は Base-to-set だけに使う。matrix は全119 nonidentity order、各 definition の finite variant、non-exact reserve/formula/head、全5 symbol の imported/ambiguous provenance、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/各 link/terminal/`BuiltinObject`/canonical corruption、unconnected-deeper と actual connected fifth-link head を reject する。unrelated-import positive は over-rejection を防止する。bidirectional check は既存 owner route 46 件すべてを isolate し、Task 207 と Tasks 211-220 focused check は既存 route を保持し、immutable-output check は mutation を防ぎ、real frontend/resolver sidecar は active runner 169 を保護する。harness は generic reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 222 four-edge object four-hop asserted-head active harness contract

task 222 route は labeled/ordered/bare object-terminal definition 5 個 `BaseFourEdgeObjectModeFourHopAssertedHead -> object`、`InnerFourEdgeObjectModeFourHopAssertedHead -> BaseFourEdgeObjectModeFourHopAssertedHead`、`MiddleFourEdgeObjectModeFourHopAssertedHead -> InnerFourEdgeObjectModeFourHopAssertedHead`、`OuterFourEdgeObjectModeFourHopAssertedHead -> MiddleFourEdgeObjectModeFourHopAssertedHead`、`TooDeepFourEdgeObjectModeFourHopAssertedHead -> OuterFourEdgeObjectModeFourHopAssertedHead`、TooDeep reserve 1 個、Base assertion 1 個だけを受理する。relation link 4 本を明示検証し、terminal traversal は Base-to-object だけに使う。matrix は全119 nonidentity order、各 definition の finite variant、non-exact reserve/formula/head、全5 symbol の imported/ambiguous provenance、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/各 link/terminal/`BuiltinSet`/canonical corruption、unconnected-deeper と actual connected fifth-link head を reject する。unrelated-import positive は over-rejection を防止する。bidirectional check は既存 owner route 47 件すべてを isolate し、Task 208 と Tasks 211-221 focused check は既存 route を保持し、immutable-output check は mutation を防ぎ、real frontend/resolver sidecar は active runner 170 を保護する。harness は generic reachability、widening、`qua`、object/set coercion、acceptance、truth/fact、proof、downstream IR を activate しない。

## task 223 parenthesized reserved-variable equality active harness contract

active task 223 route は builtin-set reserve 1 個と、left operand が identifier `x` だけを含む single unrecovered `ParenthesizedTerm`、right operand が direct `x` である equality 1 個だけを受理する。独立 wrapper/inner/right source metadata を保持し、inner/right reference だけを real reserve `BindingEnv` で解決し、別個の parenthesis type または fabricated child payload なしで inner value/type を既存 equality consumer へ透明に渡す。matrix は direct/right/both/nested/empty/non-identifier/recovered/malformed wrapper と non-exact label/operator/reserve/item を reject し、wrapper/inner/right metadata、lookup ordinal/binding、result/expected input、matched output を独立に corrupt し、immutable output、先行 reserved-variable binary-formula owner 52 件との双方向 isolation、real frontend/resolver sidecar を検証する。focused、relevant-crate、workspace verification は成功した。harness は arbitrary parenthesization/precedence、formula grouping、closure materialization、equality truth/fact、acceptance、proof、child graph、downstream IR を activate しない。

## task 224 seven-expansion set two-hop asserted-head active harness contract

active task 224 route は labeled/ordered/bare set-terminal long-chain definition 7 個、`ChainMode6` reserve 1 個、`ChainMode4` assertion 1 個だけを受理する。変更しない `BindingTwoHopRadix` は `ChainMode6 -> ChainMode5` と `ChainMode5 -> ChainMode4` を直接検証し、残る tail は terminal normalization のみに使う。matrix は全5,039 nonidentity order、non-exact definition/reserve/formula/head/provenance shape、各 expansion/relation/tail/terminal corruption、connected deeper head を reject し、unrelated-import positive、immutable output、先行 owner 48 件との bidirectional isolation、real sidecar を検証する。generic reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR は activate しない。

## task 225 seven-expansion object two-hop asserted-head active harness contract

active task 225 route は labeled/ordered/bare object-terminal long-chain definition 7 個、`ChainObjectMode6` reserve 1 個、`ChainObjectMode4` assertion 1 個だけを受理する。変更しない `BindingTwoHopRadix` は `ChainObjectMode6 -> ChainObjectMode5` と `ChainObjectMode5 -> ChainObjectMode4` を直接検証し、残る tail は object-terminal normalization のみに使う。matrix は全5,039 nonidentity order、non-exact definition/reserve/formula/head/provenance shape、各 expansion/relation/tail/terminal corruption、set/object mixing、connected deeper head を reject し、unrelated-import positive、immutable output、先行 owner 49 件との bidirectional isolation、real sidecar を検証する。focused、relevant-crate、workspace verification は成功した。generic reachability、widening、`qua`、object/set coercion、acceptance、truth/fact、proof、downstream IR は activate しない。

## task 226 seven-expansion set three-hop asserted-head active harness contract

active task 226 route は labeled/ordered/bare set-terminal long-chain definition 7 個、`ChainMode6` reserve 1 個、`ChainMode3` assertion 1 個だけを受理する。変更しない `BindingThreeHopRadix` は `ChainMode6 -> ChainMode5`、`ChainMode5 -> ChainMode4`、`ChainMode4 -> ChainMode3` を直接検証し、残る tail は set-terminal normalization のみに使う。matrix は全5,039 nonidentity order、non-exact definition/reserve/formula/head/provenance shape、各 expansion/relation/tail/terminal corruption、object/set mixing、connected deeper head を reject し、unrelated-import positive、immutable output、先行 owner 50 件との bidirectional isolation、real sidecar を検証する。focused、relevant-crate、workspace verification は成功した。generic reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR は activate しない。

## task 227 seven-expansion object three-hop asserted-head active harness contract

active task 227 route は labeled/ordered/bare object-terminal long-chain definition 7 個、`ChainObjectMode6` reserve 1 個、`ChainObjectMode3` assertion 1 個だけを受理する。変更しない `BindingThreeHopRadix` は `ChainObjectMode6 -> ChainObjectMode5`、`ChainObjectMode5 -> ChainObjectMode4`、`ChainObjectMode4 -> ChainObjectMode3` を直接検証し、残る tail は object-terminal normalization のみに使う。matrix は全5,039 nonidentity order、non-exact definition/reserve/formula/head/provenance shape、各 expansion/relation/tail/terminal corruption、set/object mixing、connected deeper head を reject し、unrelated-import positive、immutable output、先行 owner 51 件との bidirectional isolation、real sidecar を検証する。focused、relevant-crate、workspace verification は成功した。generic reachability、object/set coercion、widening、`qua`、acceptance、truth/fact、proof、downstream IR は activate しない。

## task 228 seven-expansion set four-hop asserted-head active harness contract

active task 228 route は labeled/ordered/bare set-terminal long-chain definition 7 個、`ChainMode6` reserve 1 個、`ChainMode2` assertion 1 個だけを受理する。変更しない `BindingFourHopRadix` は `ChainMode6 -> ChainMode5`、`ChainMode5 -> ChainMode4`、`ChainMode4 -> ChainMode3`、`ChainMode3 -> ChainMode2` を直接検証し、残る tail は set-terminal normalization のみに使う。matrix は全5,039 nonidentity order、non-exact definition/reserve/formula/head/provenance shape、各 expansion/relation/tail/terminal corruption、object/set mixing、connected deeper head を reject し、unrelated-import positive、immutable output、先行 owner 52 件との bidirectional isolation、real sidecar を検証する。focused、relevant-crate、workspace verification は成功した。generic reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR は activate しない。

## task 229 seven-expansion object four-hop asserted-head active harness contract

active task 229 route は labeled/ordered/bare object-terminal long-chain definition 7 個、`ChainObjectMode6` reserve 1 個、`ChainObjectMode2` assertion 1 個だけを受理する。変更しない `BindingFourHopRadix` は `ChainObjectMode6 -> ChainObjectMode5`、`ChainObjectMode5 -> ChainObjectMode4`、`ChainObjectMode4 -> ChainObjectMode3`、`ChainObjectMode3 -> ChainObjectMode2` を直接検証し、残る tail は object-terminal normalization のみに使う。matrix は全5,039 nonidentity order、non-exact definition/reserve/formula/head/provenance shape、各 expansion/relation/tail/terminal corruption、object/set mixing、connected deeper head を reject し、unrelated-import positive、immutable output、先行 owner 53 件との bidirectional isolation、real sidecar を検証する。focused、relevant-crate、workspace verification は成功した。generic reachability、widening、`qua`、acceptance、truth/fact、proof、object/set coercion、downstream IR は activate しない。

## task 230 seven-expansion set five-hop asserted-head active harness contract

active task 230 route は labeled/ordered/bare set-terminal long-chain definition 7 個、`ChainMode6` reserve 1 個、`ChainMode1` assertion 1 個だけを受理する。新規 closed `BindingFiveHopRadix` は `ChainMode6 -> ChainMode5`、`ChainMode5 -> ChainMode4`、`ChainMode4 -> ChainMode3`、`ChainMode3 -> ChainMode2`、`ChainMode2 -> ChainMode1` を直接検証し、`ChainMode1 -> BaseMode -> set` は terminal normalization のみに使う。matrix は全5,039 nonidentity order、non-exact definition/reserve/formula/head/provenance shape、各 expansion/relation/tail/terminal corruption、object/set mixing、connected deeper head を reject し、unrelated-import positive、immutable output、先行 owner 54 件との bidirectional isolation、real sidecar を検証する。focused、relevant-crate、workspace verification は成功した。generic reachability、widening、`qua`、acceptance、truth/fact、proof、downstream IR は activate しない。

## task 231 seven-expansion object five-hop asserted-head active harness contract

active task 231 route は labeled/ordered/bare object-terminal long-chain definition 7 個、`ChainObjectMode6` reserve 1 個、`ChainObjectMode1` assertion 1 個だけを受理する。byte-for-byte unchanged closed `BindingFiveHopRadix` は `ChainObjectMode6 -> ChainObjectMode5`、`ChainObjectMode5 -> ChainObjectMode4`、`ChainObjectMode4 -> ChainObjectMode3`、`ChainObjectMode3 -> ChainObjectMode2`、`ChainObjectMode2 -> ChainObjectMode1` を直接検証し、`ChainObjectMode1 -> BaseObjectMode -> object` は terminal normalization のみに使う。matrix は全5,039 nonidentity order、non-exact definition/reserve/formula/head/provenance shape、各 expansion/relation/tail/terminal corruption、set/object mixing、connected deeper head を reject し、unrelated-import positive、immutable output、先行 owner 55 件との bidirectional isolation、real sidecar を検証する。focused、relevant-crate、workspace verification は成功した。generic reachability、object/set coercion、widening、`qua`、acceptance、truth/fact、proof、downstream IR は activate しない。

## task 233 parenthesized builtin-object equality active harness contract

active task 233 route は builtin-object reserve 1 個と、left operand が identifier `x` 1 個だけを含む unrecovered `ParenthesizedTerm` 1 個、right operand が direct `x` である equality 1 個だけを受理する。独立 wrapper/inner/right source metadata を保持し、inner/right reference だけを real reserve `BindingEnv` で解決し、inner builtin-object value/type を独立 wrapper payload と object/set coercion なしで既存 equality consumer へ透明に渡す。matrix は direct/right/both/nested/empty/non-identifier/recovered/malformed wrapper と non-exact label/operator/reserve/item を reject し、wrapper/inner/right metadata、lookup ordinal/binding、result/expected input、canonical type、matched output を独立に corrupt し、immutable output、先行 binary-formula owner 53 件との bidirectional isolation、real frontend/resolver sidecar を検証する。arbitrary parenthesization/precedence、formula grouping、closure materialization、equality truth/fact、acceptance、proof、child graph、downstream IR は activate しない。

## task 234 six-hop set-terminal asserted-head active harness contract

active task 234 route は labeled/ordered/bare set-terminal long-chain definition 7 個、`ChainMode6` reserve 1 個、`BaseMode` assertion 1 個だけを受理する。新規 closed `BindingSixHopRadix` は `ChainMode6 -> ChainMode5`、`ChainMode5 -> ChainMode4`、`ChainMode4 -> ChainMode3`、`ChainMode3 -> ChainMode2`、`ChainMode2 -> ChainMode1`、`ChainMode1 -> BaseMode` を直接検証し、`BaseMode -> set` は terminal normalization のみに使う。matrix は全5,039 nonidentity order、non-exact definition/reserve/formula/head/provenance shape、各 expansion/relation/terminal corruption、object mixing、connected deeper head を reject し、unrelated-import positive、immutable output、先行 owner 56 件との bidirectional isolation、real frontend/resolver sidecar を検証する。generic reachability、widening、`qua`、acceptance、truth/fact、proof、child graph、downstream IR は activate しない。

## task 236 object-terminal six-hop asserted-head active harness contract

active task 236 route は labeled/ordered/bare object-terminal long-chain definition 7 個、`ChainObjectMode6` reserve 1 個、`BaseObjectMode` assertion 1 個だけを受理する。unchanged closed `BindingSixHopRadix` は `ChainObjectMode6 -> ChainObjectMode5`、`ChainObjectMode5 -> ChainObjectMode4`、`ChainObjectMode4 -> ChainObjectMode3`、`ChainObjectMode3 -> ChainObjectMode2`、`ChainObjectMode2 -> ChainObjectMode1`、`ChainObjectMode1 -> BaseObjectMode` を直接検証し、`BaseObjectMode -> object` は terminal normalization のみに使う。matrix は全5,039 nonidentity order、non-exact definition/reserve/formula/head/provenance shape、各 expansion/relation/terminal corruption、set mixing、connected deeper head を reject し、unrelated-import positive、immutable output、先行 owner 57 件との bidirectional isolation、real frontend/resolver sidecar を検証する。object/set coercion、generic reachability、widening、`qua`、acceptance、truth/fact、proof、child graph、downstream IR は activate しない。

## Task 241 Parenthesized Reserved-Variable Inequality Active Harness Contract

active Task 241 route は builtin-set reserve 1 個と、left operand が identifier
`x` 1 個だけを含む unrecovered `ParenthesizedTerm`、right operand が direct `x`
である inequality 1 個だけを受理する。独立した wrapper/inner/right metadata
を保持し、inner/right reference だけを解決し、独立 wrapper payload なしで
canonical builtin-set identity 1 個を既存 inequality consumer へ透過的に渡す。
matrix は direct/right/both/nested/empty/nonidentifier/recovered/malformed
operand、wrong label/operator/reserve/type/status/item、exact parenthesized
membership、exact builtin-object `<>` を reject し、provenance、binding/ordinal、
role/expected input、canonical source、matched config を独立に corrupt し、
immutable output、focused equality behavior、先行 binary-formula owner 54 件との
bidirectional isolation、real frontend/resolver sidecar を検証する。arbitrary
parenthesization/precedence、formula grouping、inequality desugaring/truth、
acceptance、proof、child graph、downstream IR は activate しない。

## Task 242 Parenthesized Builtin-Object Inequality Active Harness Contract

active Task 242 route は builtin-object reserve 1 個と、left operand が
identifier `x` 1 個だけを含む unrecovered `ParenthesizedTerm`、right operand が
direct `x` である inequality 1 個だけを受理する。独立した wrapper/inner/
right metadata を保持し、二つの reference を ordinal 1/2 で `BindingId(0)`
へ解決し、独立 wrapper payload と object/set coercion なしで written
`object` anchor の canonical `BuiltinObject` 1 個を既存 inequality consumer
へ透過的に渡す。matrix は全 direct/right/both/nested/empty/nonidentifier/
recovered/malformed near miss、wrong label/operator/reserve/type/status/item、
exact parenthesized membership、builtin-set variant を reject し、wrapper/
source-wrapper、inner/right provenance、lookup、builtin head、role/source
range、canonical bridge、expected input、matched Task 233/241 config を独立に
corrupt し、immutable output、mismatched-module rejection、先行 binary-
formula owner 55 件との bidirectional isolation、focused Tasks 190/223/233/
241、real frontend/resolver sidecar を検証する。parenthesized membership と
active imported provenance は Task 242 の credit 外。未成立 imported
expansion/evidence/signature payload、proof、downstream IR は deferred。

## Task 243 Parenthesized Reserved-Variable Membership Active Harness Contract

active Task 243 route は builtin-set reserve 1 個と、left operand が identifier
`x` 1 個だけを含む unrecovered `ParenthesizedTerm`、right operand が direct `x`
である membership 1 個だけを受理する。独立 wrapper/inner/right metadata を
保持し、二つの reference を ordinal 1/2 で `BindingId(0)` へ解決し、written
`set` anchor の canonical `BuiltinSet` 1 個を既存 membership consumer へ透過
的に渡す。変更しない direct-right producer が唯一の expected-set input を
供給し、exact contract は type entry 5 個、left expected input 0 個、right-
owned expected constraint 1 個を持つ。wrapper は独立 payload を持たない。
matrix は全 direct/right/both/nested/empty/nonidentifier/recovered/malformed
near miss、wrong label/operator/reserve/type/status/item、先行 parenthesized
equality/inequality と object variant を reject し、provenance、lookup、result
head、role/source range、canonical bridge、unexpected-left/wrong-right/missing-
right expected input、matched config を独立に corrupt する。immutable output、
mismatched module、先行 binary-formula owner 56 件との bidirectional isolation、
focused Tasks 120/223/233/241/242、real frontend/resolver sidecar も検証する。
extraction gap の解除はこの exact source だけ。object-left/set-right
parenthesized membership と active imported provenance は Task 243 credit 外。
未成立 imported expansion/evidence/signature payload、proof、downstream IR は
deferred。

## Task 244 Parenthesized Heterogeneous Reserve Membership Active Harness Contract

active Task 244 route は ordered reserve 2件、すなわち written `object` の `x`
と distinct written `set` の `y`、続く theorem
`ParenthesizedHeterogeneousReserveMembershipPayloadBoundary: (x) in y;` だけを
受理する。real frontend は left identifier を包む unrecovered
`ParenthesizedTerm` exactly 1件を生成する。real resolver と complete binding
environment は inner `x` / direct-right `y` を ordinal 2/3 で
`BindingId(0/1)` に解決する。

finite config-driven bridge は wrapper、inner、right、formula、reserve type 2件
の provenance を独立に保つ。unchanged Task 125 direct-right producer が唯一の
expected-set input を供給する。output contract は inferred term 2件、type
entry exactly 5件、distinct written range に anchor された normalized identity
2件、left expected なし、right-owned expected-set constraint 1件、fact/
candidate/diagnostic/deferred なしの checked membership を要求する。wrapper
semantic reference と object/set coercion は禁止。従来5 parenthesized config
の contract は維持する。

focused coverage は exact、near-miss、collapsed/reversed provenance、payload
corruption、immutable output、既存 owner route 57件の双方向、Tasks
120/125/223/233/241/242/243、diagnostic 不変の real active imported-mode-gap
fixture、real frontend/resolver sidecar を含む。active runner は186件、plan は
401/365、type 233/221、pass/fail 217/184。既存 expectation の rebaseline なし
で shared reference 5件と dedicated requirement 1件を trace する。extraction
gap の解除は exact source だけ。その他 parenthesized shape と imported-
positive provenance は Task 244 credit 外。未成立 imported expansion/evidence/
signature payload、proof、downstream IR は deferred。

## Task 245 Right-Parenthesized Reserved-Variable Membership Active Harness Contract

active fixture は exact `reserve x for set; theorem
RightParenthesizedReservedVariableMembershipPayloadBoundary: x in (x);`。
frontend は right operand の one-child unrecovered `ParenthesizedTerm` 1件、
resolver は distinct wrapper/direct-left/right-inner/formula provenance と
ordinal 1/2 の双方 `BindingId(0)` を保持する。

Task-245-only config/key/roles と explicit `Right` side を要求する。Task 120
consumer は one written-set identity、inferred term 2件、type entry 5件、left
expected なし、right-inner-owned sole expected-set constraint、clean checked
membership を生成し、wrapper は semantic reference を持たない。finite
left/direct/both/nested/malformed と side/config/range/constraint corruption、
Task-243 cross-route、immutable/module、既存 owner 58件の双方向、Left route
6件、real sidecar を検証。runner 187、plan 402/366、type 234/222、pass/fail
218/184、shared 4 + dedicated 1 trace。その他 shape/imported-positive/proof/
downstream は credit 外または deferred。

## Task 246 Parenthesized Two-Edge Local-Mode Equality Active Harness Contract

active case は ordered 3-mode set-terminal source、Outer reserve、`(z) = z`
の exact shape。runner は unrecovered Left wrapper 1件、distinct wrapper/
formula/inner/right provenance、ordinal 1/2 の same binding、real expansion
3件、raw Outer input 4件、Base-RHS set identity 1件、inferred term 2件、type
entry 6件、ordered expected constraint 2件、clean equality 1件を検証する。
mode AST node は Task-246 nonempty config だけ許可し、旧 empty-mode config
は closed のまま。全5 definition order、finite near-miss/corruption、cross-
route、immutable/module、既存 owner 59件の双方向、real sidecar が runner
188 を保護。trace は 403/367、type 235/223、pass/fail 219/184。

## Task 266 exact final checker handoff

既存 Task-180 source だけについて、runner は actual module-root range、
contradiction leaf site/range、normal recovery state を保持する。actual theorem
surface site はresolver-owner rangeのvalidationに使い、real local resolver theorem
owner を exactly 1件選び、checker-validated owner を取得し、
`module -> theorem -> formula` の exact three-node typed tree を構築して
formula inference を1回実行し、owner/formula row 1件を final
`ResolvedTypedAst` assembly へ渡す。final assertion は owner symbol/origin/range、
existing checked contradiction id/site/range/state/recovery、separate final
typed-node identity の一致を要求する。

missing/duplicate row、invalid formula、wrong owner node、recovered source、
owner/formula/tree/range/provenance/source/module mismatch は fail closed。
real resolver theorem owner を持たない synthetic source AST は extraction gap の
まま。既存 `.miz`、expectation、detail key、stage、272-test list、runner count、
4 CLI output は不変。truth/fact、acceptance、proof、terminal-goal、Core、CFG、VC
payload は推論しない。

## Task 267 exact proof-intent authority

Task 267 は documentation だけを変更する。Task 268 は existing Task-180
extractor を拡張し、source の omitted status/justification を分類する責任を
`mizar-test` だけに持たせる。exact extractor は current whole-tree allowlist を
維持し、unrecovered `TheoremItem` 1件、direct token sequence
`theorem SourceDerivedContradictionConstantBoundary : ;`、contradiction
formula child exactly 1件、leading theorem-status annotation なし、justification
node なし、additional structural child なしを検証する。その後 explicit
`TheoremPolicyIntent::Unmodified` と
`TheoremJustificationIntent::Omitted` を emit する。checker/core は absence から
これらを推測しない。

syntax-free proof-intent row は explicit dense id/source order、
`StatementSemanticId`、source/module、owner symbol/node/range/origin、real
checked formula id/site/range、separate compact formula node、recovery、resolver
visibility/export、2 intent enum を持つ。この source では id/source
order/statement は zero、real formula site は Node site、recovery は Normal、
unrecovered top-level resolver theorem は Public/Exported である。extractor と
handoff は全 field を Task-266 owner/formula data と cross-check する。
`Exported` は resolver name visibility であり proof acceptance ではない。core
は後で public visibility だけを保持する。

Task 268 runner coverage は annotation/written justification、missing/duplicate/
nonzero/non-dense/reordered intent row、Role formula site、recovery、全 owner/
formula/source/module/range/provenance/reference mismatch、non-Public visibility、
non-Exported export statusをrejectする。各negative caseはproof/proof-node/
terminal-goal tableが一切publishされないことをassertする。
authenticated ownerをduplicated rowとは独立にmutateし、各row visibility/export
fieldもownerとは独立にmutateする。exact singleton
pending proof、direct terminal goal、empty citations/context、no label、
local path `proof/0` を assert する。existing `.miz`/expectation を reuseする。
さらに3 proof tableすべてのdeterministic nonempty
`ResolvedTypedAst::debug_text()` renderingと、tableがemptyの場合のTask-266
debug outputのbyte-identical性をassertする。
trace status、runner stage、truth/fact、acceptance、proof search、Core/CFG/VC、
Steps 6/7 を変更しない。

## Task 268 exact proof-intent implementation

Task-180 contradiction extractorは、theoremがannotationなし、written
justification/proof blockなしで、既存exact whole-tree allowlistを満たすことを
確認した後だけdedicated statement wrapperを返す。wrapperはexplicit
`Unmodified`/`Omitted` intentをsyntax-free checker rowへ渡す。runnerはexact
pending proof、direct terminal、real formula siteとseparate compact node、empty
citation/context、absent label、`proof/0` pathをassertする。

corruption matrixはbundle omission/duplication/order、全copied identity/range/
provenance/recovery/intent field、row valueとは独立したauthenticated owner
visibility/export status、Role-site substitution、checker output cross-referenceを
coverする。JustificationClause/proof-block near missはextraction gapのまま。
existing `.miz`、expectation、admission、test name、count、trace status、4 CLI
outputは不変。次のconsumerはCore Task 31で、acceptance/proof-verification creditは
追加しない。

## VC Task 30 prepared phase-11 runner

Task 30 は `MT10-VC-T180` を VC Task 31 だけに予約する。最初の
`proof_verification` / `active_proof_verification` route は distinct
`pass_proof_verification_contradiction_formula_constant_001` source/sidecar
(`expected_phase = "vc_generation"`) だけを受け入れ、
source-to-checker-to-Core-to-VC path を2回実行し、whole-`VcSet` equality と complete
phase-11 debug bytes を比較しなければならない。既存 type-elaboration sidecar を再分類
も admit もしない。Task-31 admission test は wrong stage、missing/duplicate/wrong active
tag、wrong `expected_phase` を reject する。runner/tag guard と first real baseline は
1 logical Task-31 change であり、empty
route は禁止する。

後続 shared `MT10-VC-PV` route は VC Tasks 32-55 が所有する bounded
`MT10-VC-PV/VC<n>` slice だけを admit する。各 slice は wrong stage/tag/phase、
missing/duplicate producer data、diagnostic-bearing Core/CFG、stale handoff/intake、
corrupt seed accounting、nondeterministic output を reject する。Task 30 自体は runner
source、admission、case、report bytes を変更しない。

## VC Task 31 exact phase-11 runner

Task 31 は prepared exact route を実装する。`proof_verification.rs` は single-case
admission predicate/validation diagnostic を所有し、reusable exact CoreIr producer を
呼び、immutable CoreIr を versioned schema とともに exact mizar-vc adapter へ渡し、
full generation を反復し、verify-only baseline comparison 前に structural equality と
complete debug bytes の両方を比較する。public report は existing runner report と同様に
stable passed/failed/error/warning count と case ごとの failure reason を持ち、error
diagnostic があれば CLI は failure を返す。

admission は exact id、exactly one `active_proof_verification` tag、stage
`proof_verification`、phase `vc_generation`、pass outcome、`.miz` source、present snapshot
の conjunction である。old type-elaboration Task-180 case は除外する。missing、
unreadable、mismatched、absent snapshot、または source/Core/VC error は case を fail
させ、stable task-local diagnostic を emit する。この exact route は general proof
verifier ではなく、accepted theorem/fact を publish しない。

## Resolver R-031 declaration-symbol increment completion

R-031は既存`fail_resolve_same_signature_same_return_conflict_001` sidecar exactly 1件を
active `declaration_symbol` setへ追加する。変更しない`.miz` sourceはreal frontendと
resolver collectorへ到達する。appendしたinternal resolver class
`SameSignatureDefinitionConflict`は
`declaration_symbol.signature.same_signature_definition_conflict`だけへmapし、既存
`SameSignatureReturnConflict` mappingとdifferent-return expectationはbyte-identicalに保つ。
same-return sidecarにはactive tag、exact diagnostic payload、active wordingを追加する。public
numeric diagnosticは割り当てない。

resolverはordinary functor definitionだけをexact syntactic
namespace/spelling/pattern/definition-context/arity keyでgroup化する。all-identical returnは
new-class diagnostic 1件、mixed/different returnはcomplete candidate groupを含む既存
return-conflict diagnostic 1件を生成し、overlapしない。これによりactive declaration-symbol
countは4件から5件へ増え、そのCLI output/hashは変わるが、parse-only、type-elaboration、
proof-verification admissionは変わらない。

## Parser Task 47 parse-only increment

parse-only runnerは`pass_parser_reconsider_tails_001`を97件目としてadmitする。real
frontend/parser pathを実行し、omitted tailとproof-block tailの両方でdiagnosticなしを要求する。
existing explicit-`by` controlはactiveのままで、変更しないmixed recovery sourceはobsolete
omitted-tail diagnosticを除いたすべてのnon-Task-47 parser errorを引き続き報告する。

このincrementで変わるのはparse admissionとplan bytesだけで、plan 405/369、parse-only
97/97、pass/fail 221/184、warnings/errors 23/0となる。declaration-symbol、
type-elaboration、proof-verification admissionは5/188/1のままである。

## Parser Task 48 property-implementation parse-only increment

parse-only runnerはpass/fail pair
`pass_parser_property_implementations_001`と
`fail_parser_property_implementations_recovery_001`を追加し、両方ともreal
frontend/parser pathを実行する。pass caseはtop-level means/equals property
implementation、simple/case/otherwise definiens、exact single
`let identifier be mode_application;` parameter、meansのordered mandatory
existence/uniqueness condition、optional coherence、supported justification shapeをcoverする。
fail caseはmalformed parameter/dot/correctness ordering/forbidden equals conditionに対する
bounded recoveryとfollowing declaration preservationをpinする。

exact requirement `spec.en.07.modes.property_implementation.parser`は
`pass_and_fail`付き`covered`となる。これはparser/syntax-only creditであり、harnessは
property payloadをextractせず、overlap/coherence decisionを行わず、proof
acceptance/dischargeもcreditしない。inactive semantic Task-39 caseは変更しない。
active totalはplan 407/369、parse-only 99/99、pass/fail 222/185、warnings/errors
23/0であり、declaration-symbol/type-elaboration/proof-verification admissionは5/188/1のままである。

## Checker Task 248 source/binding-context increment

type-elaboration runnerはexact reserve-plus-definition-parameter pass 1件をcase 189として
admitする。raw `SurfaceAst` inspectionは`mizar-test`に保持し、両itemをreal resolver
`DeclarationShellSet`へmatchして、syntax-free shell/order/range/local-scope/
declaration-site/written-type-site projectionだけを`mizar-checker`へ渡す。その後runnerは
same-spelling reserve/localのdistinct identityとstructural shadow linkを含む同じimmutable
`SourceBindingContextHandoff`を`TypedAst`と`ResolvedTypedAst`でverifyする。

このrouteはtype result、expression、fact、obligation、formula、statement、proof、Core、
CFG、VC payloadを生成しない。matched payloadがinvalidならtask-local internal detail key
1件でfailし、public diagnosticは割り当てない。exact requirementはnew bounded covered
pass rowとなるが、broad payload-extraction rowは変更しない。

## Checker Task 249 runner boundary

implemented `type_elaboration` incrementはprivate `source_type` leaf 1件がownする。
named broad fail fixtureのreserve written-type root 10件だけからsyntax-free
type-head/application/argument projectionをextractし、checker-owned 10/13/6
handoffをassertする。definition/import scaffoldingは除外する。runnerはそのhandoff
直後にsingle internal readiness detail
`type_elaboration.checker.source_type_application.semantic_dependencies_pending`
で停止し、normalization/later semanticsをrequest/creditしない。

unchanged Task-248 pass routeがdependency regressionであり、actual Task-248
source-context handoffと並べてexact 2件の`Bare`/builtin-`set` row、argument 0件を
co-installする。raw syntaxはcheckerへ渡さず、public diagnosticはallocateせず、
existing expectation/trace rowはrebaselineしない。resolverが要求するdistinctな
scaffolding formal/field nameはtask-local `design_drift`とparse-only preflightの
`test_gap`だけをrepairし、handoff rowをemitせずsemantic intentも変えない。

## Checker Task 250 frozen runner boundary

future private `source_attribute` leafはexisting Task-81 argument-bearing、
Task-67 structure-qualified、Task-84 imported、Task-85 negative-nonempty
fixtureだけのraw-AST extractionをownする。各routeはreal 1/1/0 Task-249
dependency handoffをco-installする。aggregate immutable Task-250 handoff oracleは
nonempty chain 4件、attribute 4件、qualifier 1件、parenthesized argument group
1件、actual 1件、polarity positive 3/negative 1、attribute identity local 2/
imported 2である。

Task-81/67 sidecarはrunner-owned source-attribute semantic-dependency detailへだけ
進む。Task-84/85はchecker evidence-query detailとlegacy `AttributeInput` routeを
維持する。exact sibling selectorによりTask-116、Task-171、Task-77、broader caseへ
creditしない。synthetic private-extractor testはさらに`SurfaceAst`からpublic
checker handoffまでmulti-attribute order、single/parenthesized prefix
punctuation/actualをcoverし、checker-input-only testでは不十分である。exact probeは
`p-ranked (q,2)-graded set`で、Task-249 1/1/0、Task-250 1/2/0/2/3 table、
single identifier prefix 1件、exact comma/delimiter/hyphen provenance付き
parenthesized identifier/numeral prefix 1件を要求する。

raw syntaxはcheckerへ渡さず、new `.miz`/public diagnosticはplannedせず、
semantic arity、prefix/list equivalence、admissibility、evidence、truthをinferしない。
このfrozen boundaryはChecker Task 250 implementationまでdocumentation-only。

## Checker Task 250 source-attribute consumer

private `type_elaboration::source_attribute` leafはfrozen raw attribute
extractionとchecker handoffをownする。existing Task-81/67/84/85 fixtureだけが
selectする。4 routeはaggregate Task-249 4/4/0とTask-250 4/4/1/1/1 tableを、
exact polarity、qualifier、punctuation、actual、local/imported provenance付きで
publishする。private synthetic `SurfaceAst` testは
`p-ranked (q,2)-graded set`を同じextractor/public checker producerまで実行し、
multi-attribute orderとsingle/parenthesized prefix projectionを証明する。

Task 81/67はrunner-owned semantic-dependency detailへだけprogressし、Task
84/85はevidence-query outcomeを維持する。他route、`.miz` source、semantic
acceptance、public diagnostic、later checker payloadは変更しない。

current production layoutは21 path / 23,184行、sorted path/content hashは
`bd42d60f...` / `d1421834...`。`runner.rs`はfacade/top-level orchestration
onlyのままで、new private leafがsole added production pathである。mizar-test
library testは283件。

## Checker Task 251 frozen runner boundary

private `type_elaboration::source_evidence` leafはexact Task-249-broad +
Task-84/85 dispatchをownする。checker-owned transport request 10件
（mode-expansion 5、structure-inhabitation 3、attributed 2）を全てmissing、
dependency reference 0件としてpublish
する。3-route dependency oracleはTask-249 12/15/6とTask-250 2/2/0/0/0で、
broad単体は10/13/6を維持する。

このleafがownするのはrequest/response associationとexact dispatchであり、new
raw extractorではない。existing `source_type`/`source_attribute` leafがselector/
AST traversalを維持する。production reuseに必要なnarrow crate-private Task-250
output visibility/factorだけを変更でき、duplicate/move/selector widening/
extraction behavior changeは禁止する。

broad sidecarだけをrunner-owned missing-dependency detailへadvanceし、Task
84/85はchecker evidence-query detailを維持する。sibling、`.miz`、public
diagnostic、semantic evidence result、later payloadは変更しない。

library testはreal source extractionとproduction consumerを使い、exact count、
sibling isolation、requested/missing/rejected/supplied injectionをfinal
`TypedAst`/`ResolvedTypedAst`までproveする。supplied referenceはevidence
acceptanceではない。corrupt inputは`Rejected`をpublishせずatomic failする。
implementationはtest 4件を追加し、documented library totalを283から287へ進める。
exact selection、four-state injection、final ownership、corruption、
deterministic replayはproduction pathでpassする。

## Checker Task 252 frozen runner boundary

private `type_elaboration::source_term` leafのreal selectorはbuiltin
numeral equality、bare reserved-variable equality、single-left-parenthesized
reserved-variable equalityの3件だけである。aggregate public handoff oracleは
primary term 7、binding reference 4、numeric-type request 2である。source case、
current outcome/diagnostic/detailは変更しない。

leafがownするのはraw primary-term occurrence、binding role、parent edge、
numeric request extractionだけである。legacy semantic producer/formula routeを
維持し、applicable exact helperをreuseして、new handoffを`TypedAst`から
clone-only `ResolvedTypedAst`へpublishする。parenthesized rowはsource wrapper
だけで、independent semantic term/type/FOL rowを追加しない。

synthetic runner probeはsame extractor/producerで`LocalAbbreviation` identifier、
current-result source roleだけの`it` surface、eligible nested parenthesisを
coverする。referenceはexact lexical `BindingEnv::lookup` winnerをauthenticateし、
scopeはterm context、use ordinalは先行して完了したbinding row数からderiveし、
prior referenceはordinalを進めない。exact consecutive duplicate-priority
binding groupは全lookup-priority inputを共有し、final dense row indexをshared
visibility ordinalとして使うため、lookupはambiguous winnerをrejectできる。
later-family termを含むparenthesisはTasks 253-255がcross-family edgeをfreezeする
まで除外する。real constant declaration ownerとreal `it` owner/typeはTasks
269/260/264へdeferする。
probeはfixture/admission/semantic result/formula/fact/coverage creditを作らない。
implementation testはexact selection、corruption isolation、deterministic
replay、final ownershipもproveする。4 testsによりlibrary totalは287から291へ
進み、raw/normalized sorted-list hashは
`d46edefebc54a2f2f170cbfce8143ed036fa7ce339ebb3a746d89b55293931e5` /
`f7b5babbf33e1e3e3afe4c49018744a4a0fe42968fd2e5edc411eb7bc49fc0a6`である。
private leafだけがnew production pathで、23 paths / 24,120 lines、
path/content hashは
`562224fc62e93a256f5d3891e3a466a45ec23c24055e3a9f3f83848a0672a16b` /
`8a4b76e37a8a6921ed89e98372ccb037cd64ed583ac0bbe26466924ef0c4b028`である。

## Checker Task 253 frozen runner boundary

これはChecker Task 253であり、既に完了した`mizar-test` runner-refactor
Tasks 253A/253Bではない。future private
`type_elaboration::source_application` leafは、paired checker planでfreezeした
既存imported `1 divides (1 ++ 2)` caseと、新sourceのsecond definiensにあるexact
module-local `task253_local_source(x)` closureだけをselectする。actual
occurrenceにTask-252 primary-term producerをcomposeし、aggregate Task-253
tableをapplication 2 / wrapper 1 / candidate-reference 2 / argument 3 /
request 4、Task-252 sliceを3/1/2でpublishする。

leafはraw application/head/form/wrapper extraction、ordered edge projection、
individual resolver-reference provenanceを所有する。primary rowを複製せず、
complete/viable candidate setをclaimせず、winnerを選択せず、signature/resultを
publishしない。imported outer parenthesesはTask-253 cross-family
wrapper/originである。local sourceではTask-248
reserve-then-definition two-item/two-binding shadow handoffをreuseし、actualが
`BindingId(0)`ではなく`BindingId(1)` / `BindingContextId(1)` /
`use_ordinal == 2`へresolveすることをrequireする。既存imported outcome/detailは
不変である。new local routeはapplication transportをvalidateしてからgeneric
`definition_declaration_payload_extraction_gap` /
`type_elaboration.external_dependency.ast_payload_extraction` boundaryを維持し、
public diagnosticを追加しない。

synthetic `SurfaceAst` transactionは同じextractor/producerでremaining ordinary
form、nested application、primary/application parenthesis、definition-parameter
actual、multiple authenticated candidate referenceをcoverする。inline
zero/one/two-actual caseはtestを分け、raw-AST probeはgeneric
`ApplicationTerm`とmandatory parenthesesだけをassertし、caller-supplied producer
DTOがcandidate/requestなしのschemaをassertする。production extractorは`Inline`を
inferせず、identity/formal/capture/substitutionはTask 270が所有する。template application
subtreeはTask-253 rowをemitせず、direct role/actual/guard/request transportは
Task 277、ordinary/template candidate/selectionはTask 278が所有する。testは
frozen corruption matrix、sibling isolation、deterministic replay、
non-equivalent same-source/module Task-252 handoffのfingerprint substitution
rejection、equivalent clone acceptance、final ownershipも証明する。private-selector matrixは
imported missing/wrong/duplicate provenance、wrong `++` head/form/arity、wrong
numeral order、recovery、local functor order reversal、forward use、wrong/extra
head/actual/application/item、recovery、outer `BindingId(0)` selectionをrejectし、
両Task-253 routeと全existing Task-252 selector間のbidirectional exclusionを証明する。

本documentation prerequisiteはrunner route/test-list/layout/hashを変更しない。
current 291-test、23-path/24,120-line Task-252 baselineはexactに不変で、
implementationのprojected countはplan 412/376、type 242/230、pass/fail
224/188、admission 101/5/191/1、warnings/errors 23/0でありfresh measurementを
必要とする。

## Checker Task 253 runner completion

private leafはfrozen consumerのexactly 2件でactiveになった。primary rowを
複製せずTask 252をcomposeし、exact dependency fingerprint検証後にTask-253
handoffをinstallしてclone-only final preservationを確認する。real testsは
2/1/2/3/4と3/1/2、local inner binding coordinates、imported wrapper、
corruption isolation、deterministic replay、他の全active type-elaboration
case exclusionを実測する。measured corpusはplan 412/376、type 242/230、
pass/fail 224/188、admission 101/5/191/1、warnings/errors 23/0である。
303-test raw/normalized list hashは`a81f44fb...` / `1a621c56...`、
24-path/25,607-line production path/content hashは`5cc36b8a...` /
`b9b6c678...`である。exact値と5 CLI hashはpaired module-boundary auditに
記録する。

## Checker Task 254 frozen runner boundary

future private `type_elaboration::source_structure` leafは、paired checker planが
freezeするexact `fail_type_elaboration_local_structure_term_gap_001` sourceの
3 definiensだけをselectする。existing Task-252 producerをcomposeし、Task-254
term/wrapper/root/member/field-update/edge/request = 5/0/3/9/2/10/26と
Task-252 primary/reference/numeric-request = 8/0/8をpublishする。real routeに
Task-253 row/fingerprintはない。

leafだけが`StructureConstructor`、`SelectorAccess`、`StructureUpdate`、
`FieldArgument`、`FieldUpdate`をinspectする。member segment、repeated written
label/path、ordered child、transparent wrapperを保存するが、field/propertyを
classifyせずsemantic resultを計算しない。synthetic transactionはconstructor
cardinality/nesting、selector chain/call、nested update path、全Task-252/253/254
target kind、wrapper、degraded recovery、local/imported root、signature-shell
state、corruption、deterministic replay、dependency substitution、final AST
ownership、whole-subtree exclusionをcoverする。cross-family Task-253 application
targetはどのTask-253 argument edgeからもtargetにされないrootで、owning Task-254
contextと一致しなければならない。nested Task-253 applicationはmultiply ownせず
rejectする。structure childを含むreverse applicationはexcludedのままである。

real caseはpublic diagnosticなしで
`definition_declaration_payload_extraction_gap` /
`type_elaboration.external_dependency.ast_payload_extraction`に留まる。existing
parser/resolver guard fixtureと他の全active caseはbytes/sidecar/stage/status/credit
不変である。本prerequisiteはroute/test list/layout/hashを変更せず、current
303-test、24-path/25,607-line baselineはexactに不変である。別implementationは
plan 413/377、type 243/231、pass/fail 224/189、admission 101/5/192/1、
warnings/errors 23/0をprojectするがfresh measurementを要求する。

## Checker Task 254 runner completion

production dispatchはexact local structure-term fixtureだけをprivate
`source_structure` leafへTask-253 application routeより先にrouteする。leafは
declaration shellとTask-248 binding contextをconsumeし、Task-252をcomposeして
Task-254 5/0/3/9/2/10/26 + Task-252 8/0/8をpublishし、frozen external-gap
detailを維持する。generated definition contextやsemantic structure/member/view
resultは作らない。

focused test 9件はreal oracle、corruption atomicity、他active type case 191件からの
isolation、frozen syntax/recovery/subtree matrix、synthetic boundary、imported
producer provenance、Task-253 root/child fingerprint、unrelated-handoff
preservationをcoverする。measured corpusは413/377、243/231、224/189、
admission 101/5/192/1、warnings/errors 23/0である。312-test raw/normalized
list hashは`b7f56668...` / `09acdf12...`、25-path/27,317-line production
path/content hashは`e81c3b08...` / `3046ae27...`である。exact値と5 CLI
hashはpaired crate plan/module-boundary auditに記録する。

## Checker Task 255 frozen runner boundary

future private `source_set_term` runner leafはfrozen four-definiens local case
だけをvalidateする。exact oracleはTask-255 terms/wrappers/generators/
type-sites/edges/requests 4/0/1/3/4/7 + Task-252
terms/references/numeric requests 4/0/4で、real Task-253/254 dependencyはない。
raw syntaxはprivateに留め、checkerにはsyntax-free table 6個だけを渡す。

routeはwritten generatorを使わないmapperを持つcondition-free comprehensionだけを
admitする。generator binding/capture、condition formula、non-bare target type、
semantic sethood、choice nonemptiness/stability、`qua` wideningを捏造しない。
implementationはexact-source selection、all-active isolation、syntax/recovery/
subtree exclusion、corruption、deterministic replay、dependency fingerprint、
immutable final preservationを証明する。本prerequisiteはrunner/fixture/sidecar/
trace/count/hashを変更しない。

## Checker Task 255 runner completion

private routeはこのboundaryを実装し、frozen external-gap sidecarを維持する。
exact real aggregateはTask-255 4/0/1/3/4/7 + Task-252 4/0/4で、
Task-253/254 fingerprintとsemantic outputはない。recursive extractionはnested
mapper visit後にgenerator/type-site IDをnormalizeし、term-grouped public orderと
written type-site orderを同時に保存する。

focused real/synthetic testは全public row association、zero/many/nested shape、
wrapper/recovery、optional cross-family target、explicit exclusion、corruption
atomicity、deterministic replay、final ownership、全他active type caseからの
isolationをauthenticateする。active corpusは414/378、244/232、224/190、
101/5/193/1、warnings 23/errors 0である。later binder/formula/semantic ownerは
deferredのままである。

## Checker Task 256 frozen runner boundary

future private `source_atomic_formula` routeは既存active fail 8件をreuseし、
source caseを追加しない。current semantic bridgeとexact external-gap outcomeを
維持する前に、sourceごとのsyntax-free public Task-256 transactionをvalidateする。

real aggregateはTask-256 formula/wrapper/head/candidate/type-site/attribute/
edge/request `8/0/1/1/1/2/13/11`、Task-252 `16/0/16`、Task-253
`1/1/1/2/2`、Task-255 `2/0/0/0/4/2`で、Task-254 targetはない。
raw syntax/selectionはprivateに留め、checkerにはdense row、resolver identity、
cross-family ID、unresolved requestだけを渡す。

routeはcomplete Task-252 primary unionをsingle shared handoff/arenaへ先に
形成し、同じobjectに対してTask-253/255 dependencyをbuildした後Task 256を
buildする。narrow private helper reuseは許すが、既存lower-family standalone
selector/allowlistを拡張・変更しない。

runnerはexact-source selection、all-active isolation、candidate/attribute
provenance、bare assertion-type ownership、direct-child exhaustiveness、
request association、dependency fingerprint、両install順、corruption atomicity、
deterministic replay、immutable final preservationを証明する。implementation時の
frozen reciprocal trace reference/transport note以外は既存sidecarを変更しない。
positive oracleはedge 13件とrequest 11件すべてのID、ordinal、role/kind、
target/association、effective range、Task-253 outer parenthesized range、
positive/negative attribute targetと`non` anchorのexact ordered vectorである。
既存Task-252/253/255 standalone-selector isolation oracleも不変で証明する。

predicate chain/negation、inline/template、general type graph、qualified/
argument-bearing attribute、semantic fact/truth、theorem acceptance、
conditioned comprehensionはexcludedのままである。本documentation prerequisiteは
runner、fixture、sidecar、trace、count、test list、production manifest、hashを
変更しない。

## Checker Task 258B3M2B2B1P private Task-253 seam contract

future private helper
`unwrapped_imported_source_application_handoff_in_context`はsurface AST、
module、symbol/binding environments、shared Task-252 source terms、
application node、explicit `BindingContextId`を受ける。既存unwrapped
imported extraction/handoff builderをreuseし、legacy helperはcontext 0で
delegateする。

143-byte `take 1 ++ 2;` probeではcontext 1がsymbolic infix application
1/wrapper 0、imported `parser.type_fixtures::++`、arguments
`Primary(2)`/`Primary(3)`、requests 2をexactにproduceする。freezeする
compound testsはちょうど2件:

1. `task258b3m2b2b1p_proof_context_reuses_exact_unwrapped_imported_application`;
2. `task258b3m2b2b1p_context_provenance_and_legacy_replay_fail_closed`.

missing/nonexistent/mixed contexts、wrong node/range/head/form/argument
order、wrapper、import/candidate/provenance substitution、stale Task-252
fingerprint、replayをrejectし、context-0 outputsをbyte-identicalに維持する。
B1Pはstatement、typed/final statement coexistence、diagnostic detail、
semantic/proof/goal outputをpublishしない。

## Checker Task 258B3M2B2A dormant consumer implementation

exact selectorはprior statement profilesより先に実装し、121-byte source、
all 57 unrecovered nodes/root 56、frontend diagnostics 0だけをacceptする。
five rootsはcomplete `2 -> 3 -> 4` chainを持つseven primariesへexpandし、
refsは`0/1/4/5/6`、equalitiesは`[0,1]` / `[5,6]`だけ、unnamed
witnessはouter term 2をtargetする。

frozen tests 5本は全pass。successは`Some(Vec::new())`、lookups `1/1`、
uses `[1; 5]`、binding/semantic output emptyを維持する。invalid
Task-252/256 rowがhandoffを形成できなければlower producerでrejectし、
constructible corruptionはpaired statement consumerへ到達する。active
corpus/public route/key/fixture/sidecar/expectation/trace changeなし。

## Checker Task 256 runner completion

private `source_atomic_formula` routeはこのfrozen boundaryを実装した。exact
sourceごとにcomplete Task-252 primary handoff/arenaを1個形成し、同じobjectに
対してTask-253/255 dependencyをbuildし、public Task-256 transactionをvalidate
して`TypedAst`/`ResolvedTypedAst`へinstallした後、既存semantic detail routeへ
ownershipを戻す。

8 transactionはTask-256 `8/0/1/1/1/2/13/11`、Task-252 `16/0/16`、
Task-253 `1/1/1/2/2`、Task-255 `2/0/0/0/4/2`をexactにproduceする。
direct edgeはprimary 10件、root application 1件、root set term 2件で、
Task-254 targetはない。testは全ordered edge/request row、独立resolver/source
anchor、不変のdetail vector 8件、selector isolation、atomic corruption、
synthetic cardinality/shape、install/revalidation boundary、final ownershipを
assertする。

`.miz`、outcome、phase、category、rejection reason、stable detail、diagnostic
payload、tagは変更しない。predicate chain/operator/binder、general type/
attribute graph、semantic fact/truth、conditioned comprehension、inline/template、
overload selectionはfrozen ownerへdeferしたままである。

## Checker Task 257A frozen runner boundary

ここでのChecker Tasks 257A-Cはchecker producer sliceで、本文中の完了済み
mizar-test Tasks 257A-H test-layout系列とは別である。

唯一のexact routeは
`fail_type_elaboration_formula_connective_quantifier_gap_001`である。raw
`SurfaceAst` traversalはprivateに留める。`source_formula.rs`はformula site
5件、explicit binder segment/identifier、bare `set` type-expression/head、
frozen rangeをselect/retainし、private `source_composite_formula` leafはそのdataを
public syntax-free transactionへtranslateできる。

runnerはauthenticated source/resolver/symbol-module identityとpublic
`BindingEnvParts`/table APIからexact normal `1/0/4` module-shell environmentを
構築し、older private `1/0/0` semantic helperを変更しない。同じimmutable
Checker Task 257A inputからdedicated source-formula expression contextとresolver-shaped local
quantifier binding 1件を持つexact `2/1/4`へextendする。Checker Task 248
source-context handoffを捏造しない。
`x`はunusedなのでChecker Task 252/253/254/255/256 term-family dependencyをbuildしない。
その後7-table `5/0/1/1/1/4/6` handoffをbuild/installしてから、不変のolder
semantic routeを呼ぶ。

positive testはformula/root/binder/type-site/edge/request rowをすべて順序どおり
assertし、exact site/range、context 0-to-1 transition、declaration/type
provenance、不変two-key detail vectorを含む。negative testはselector
isolation、recovery/spelling変更、tree/parent/role/cardinality corruption、
stale binding/context/type identity、wrapper、deterministic replay、one-shot
install、dependency revalidation、sole standalone Checker Task 257A install
sequence、synthetic preinstalled Checker Task 248 source-context rejection、immutable final
ownershipをcoverする。実行可能なrejectionは
`TypedAstParts { source_context: Some(task_248_handoff), .. }`から始める。
public source-context installerがないためreverse-order testを要求しない。
`source_context()`は`None`のままで、extended environmentはChecker Task 257A handoff
だけがownする。lower-family selector/allowlistを拡張しない。

positive oracleはcomplete embedded environmentと7 table全rowを含むsingle full
literal handoff `debug_text()` snapshot、およびChecker Task 257Aなしのexact legacy
`TypedAst::debug_text()` bytesを使う。2 rerun間のequality/substring presenceは
補助であり代替ではない。

broader formula shape、bound use/capture、predicate chain、conditioned
comprehension、theorem ownership、全semantic answerはdeferredのままである。
本documentation prerequisiteはrunner、fixture、sidecar、trace、count、test
list、production manifest、hashを変更しない。

### Checker Task 257A implemented route

runnerはfrozen routeを不変semantic-detail ownerより先に実行する。extended
private extraction recordでexact formula、binder segment、identifier、type
expression/head siteを保持し、syntax typeをexportせずpublic transactionを
assembleし、handoffをvalidate/installしてresolved resultへclone-preserveする。
private test 5件はexact real oracle/final ownership、independent table
corruption後のrecovery、all-active/lower-family isolation、不変semantic detail、
preinstalled Task-248 rejectionをcoverする。

## Checker Task 257B1 frozen runner boundary

exact 79-byte pass sourceは既存formula extraction ownerでselectし、専用private
Task-257B1 leafでcomposeする。leafはTask-257 binding extensionを先に使い、
one arenaでTask-252 primary reference 2件、Task-256 equality、第2
one-node Task-257 composite profile、`1/2` formula-composition handoffをbuild
する。

positive testはexact parser range、binding lookup winner 2件、全lower/
composite/composition row、dependency fingerprint、final typed/resolved
ownership、semantic output非生成をassertする。negative testはselector
isolation、recovery、各cross-family association、dependency substitution、
corruption recovery、既存Task-257A route byte不変をcoverする。さらに
`TypedAstParts`へTask-248 source-context handoffをpreinstallし、combined
installerがTask-257B1 handoffを一切publishせずfailすることを要求する。current active
selector/semantic routeをwidenしない。

positive composition oracleはsingle full literal
`source-formula-composition-debug-v1` renderingで、module、exact
Task-252/256/257 debug-string fingerprint、edge count/row、bound-use row 2件を
field順で含む。typed/resolved getterはexisting composite-formula debug section
後のidentical handoffを公開し、B1 absent時は全legacy byteを保持する。

ownership-partition matrixはvalid second-profile handoffをTask-257A legacy
installerへ渡し、Task 257Aを既にownするASTへcombined B1 transactionを渡す。
両方byte-identical rollbackでfailする。profile matrixはA cardinality+B row、
B cardinality+A row、otherwise valid third shapeを検証する。exact 2 profileだけ
build可能で、failed build/install pathはpartial B1 stateをpublishしない。

### Checker Task 257B1 Implemented Route

private leafはfrozen selector/same-arena compositionを実行する。positive oracleは
Task-252/256/257/B1の全row、lookup winner 2件、全dependency fingerprint、full
literal composition debug rendering、typed/resolved ownershipをcoverする。
isolation、corruption、ownership-partition、profile-discriminator、Task-248、
Task-257A rollback caseはexecutableで、external resultはsemantic formula
outputを持たない。

## Checker Task 257B2 frozen harness boundary

dedicated private formula-composition leafはexact case ID
`pass_type_elaboration_formula_connective_grouping_payload_001` 1件だけを追加
selectできる。exact theorem label/tree、normal recovery、explicit
`x being set`、fixed/repeated flag/token、wrapper placement、equality endpoint、
extra item/justificationなしを要求する。

runnerはsame arenaでTask-252、Task-256、第3 Task-257 profile、`8/0`
compositionをbuildし、composite/composition pairをatomic installする。positive
assertionは`16/0/16`、`8/0/0/0/0/0/16/16`、
`8/6/1/1/1/7/9`、`8/0`の全rowをcover。negativeはsource isolation、
各profile/association mutation、fixed/repeated substitution、wrapper
cross/order、dependency replacement、ownership collision、valid replay
recoveryをcoverする。

source-selector near-miss matrixはstatus/justification変更、別item追加、
binder/binder type変更、operand reorder/count変更、wrapper depth/place変更、
fixed/repeated token substitution、alternate connective tree、代表的な
Task-257B3 binder shapeとTask-257C
predicate-chain/conditioned-comprehension shapeをそれぞれ独立に与え、全variant
をunselectedに保つ。checker profile-discriminator matrixもcoherentな
otherwise-valid第4 profileを1件与え、partial stateをpublishせず失敗させる。

active caseはdetail keyなしでpassし、type/fact/checked formula semantics/
statement semantics/theorem acceptance/proof/IR creditを持たない。既存
Task-257A/B1 selector/byte oracleは不変。

## Checker Task 257B2 Implemented Harness Route

routeはfrozen extraction/same-arena producer sequenceを実行し、composite/
composition pairをatomic installする。active caseはdetail keyなしでpassし、
selector isolation、source near-miss、profile/association corruptionとvalid
replay、A/B1 preservation、final typed/resolved ownershipをtestする。semantic
output/theorem creditは生成しない。

## Checker Task 257B3 Frozen Harness Boundary

existing private formula-composition leafは将来
`pass_type_elaboration_formula_nested_quantifier_payload_001`だけをselectできる。
exact reserve/theorem pair、normal recovery、restricted `x`、existential `y`、
implicit-reserve nested `r`、equality atom 3件、extra item/justificationなし、
final-LF source hashを要求する。

routeはTask-48 reserve binding environmentをprepareし、Task-248 source
contextをabsentに保ち、nested quantifier binder 3件をextendしてexact
`6/6/0`、`3/0/0/0/0/0/6/6`、`3/0/1/3/3/2/6`、`3/6`をone arenaで
buildする。near missはreserve presence/name/type/order、explicit/implicit
binder、quantifier kind、`st`/`holds`、nesting、atom、lookup winner、
recovered node、theorem labelとsidecar status/justification、extra theorem/
definition/non-reserve item、formula wrapper、attributed/argument-bearing
reserve/binder type、B2/C shapeを独立にmutateする。passはimmutable source
transportだけをassertし、truth、witness、closure、fact、theorem、proof、IR
outputを持たない。

## Checker Task 257B3 implemented route

exact selector/same-arena transactionはactive。testsはreal range/scope/
provenance、cumulative visibility/shadowing、lookup/owning edge 6件、
active-corpus exclusivity、recovered/structural near miss、aggregate
corruption/replay、transport-only sidecar、final typed/resolved ownershipを
coverする。selectorはloaded source textを受け取り、final LF込みfrozen
138 bytesを要求する。missing-final-LFとwhitespace-only variantを独立に
rejectする。

## Checker Task 257C1 frozen runner route

private exact routeはfrozen 107-byte
`FormulaPredicateChainPayloadBoundary` sourceだけをrecognizeする。predicate
segment 2件、同じimported `divides` provenance、normal `does not` token
range、Task-252 `3/0/3`、extended Task-256
`1/0/2/2/2/0/0/3/2`、middle term `2`に対するshared edge id 1件を
verifyする。selectorはloaded source textを受け、final LFをguardする。

runner matrixは全named source near miss、parser recovery、mixed/built-in
chain、segment/polarity/edge/candidate/request corruption、active-corpus
isolation、old route preservation、atomic install/rollback、resolved clone、
empty semantic outputをcoverする。future sidecarはimmutable source transport
だけを持ち、truth、negation result、predicate winner、theorem result、proof、
IR detailを追加しない。

extended input fieldはcurrent runner literal 4件に到達する。sole
atomic-formula constructorはexact C1 segment 2件を条件付きemitし、prior
routeではemptyのまま。formula-composition constructor 3件もすべてemptyを
保つ。これらliteral editはcompatibilityだけに必要で、Task-257 composition
routeをactivateしない。

### Task 257C1 runner status

exact source route、pass sidecar、covered trace row、loaded-text near-miss
matrix、input-corruption matrix、active-corpus isolation、prior route不変は
executableである。successful external detail vectorはemptyで、全semantic
predicate/theorem workはdeferredのまま。

## Checker Task 255C1 frozen runner boundary

private source-set leafがrecognizeするのはexact 191-byte
conditioned-comprehension definitionだけである。loaded source textを受け取り、
final LF/parser rangeをauthenticateし、unique imported `++` candidateをderiveし、
全named structural/recovery/provenance/byte near missをpublication前にrejectする。

one arenaでTask-252 `4/0/4`をproduceし、Task-253 private builderをreuseして
`1/0/1/2/2`を作り、Task-255 `1/0/1/1/1/1/2`をproduceする。condition
colonはTask-255-owned、direct `FormulaExpression` wrapperは別のTask-255
anchorを持ち、2 numeral operandsはTask-252-ownedでTask 255からuntargetedである。
Task-256/257 handoff/semantic tableは作らない。future fail sidecarはsuccessful
source transport後にdefinition-extraction detail keyを保持する。

### Checker Task 255C1 runner status

frozen routeを実装した。exact loaded byteとparser/resolver provenanceがone
transactionだけをselectし、全named near missはpublication前にreturnする。
runnerはshared `4/0/4`、`1/0/1/2/2`、`1/0/1/1/1/1/2` handoffをcomposeし、
inner condition formulaをunownedに保ち、empty semantic outputと全prior
routeを保持する。

## Checker Task 257C2 frozen runner boundary

private complete routeはlower Task-256/255 diagnostic routeより先にexact
Task-255C1 loaded-source selector、Task-253 imported-application seam、
reusable Task-256 equality builderをreuseする。同じarenaをatomic equality
1件/immutable condition-formula association 1件でextendし、全Task-252/253/255
row/siteを不変に保つ。

frozen pre-Task-256C1 baselineでは、lower validatorがenclosing condition
set termをset/atomic両install orderでrejectしたため、このtarget routeは
gateされていた。separate checker prerequisiteは完了し、authenticated
condition relationだけで両orderをpassさせ、arbitrary overlapをfail-closedに
保つ。routeはprerequisite exit時点でfresh Task-257C2
preflight/implementation待ちだったが、現在はcompleteし、lower
diagnostic-only routeより先にexact Task-256 equalityとsole Task-257C2
associationをpublishする。

existing fail caseはdefinition-intake detailを保持し、semantic tableはempty。
exact profile、direct wrapper/equality ownership、provenance、near miss、
mutation rollback、bidirectional A/B/C2 installer exclusion、isolation、
final cloneがcomplete test boundary。本prerequisiteはexecutable runner
artifactを変更しない。

separate implementationはrunner tests 4件をpassし、unchanged
extraction-gap sidecar detail/empty semantic tableを保持する。measured
plan/typeは`419/386` / `252/240`、runner tests 361件、production manifestは
29 paths / 34,064 linesである。

## Checker Task 256C1 frozen harness non-ownership

Task 256C1にharness edit/testは含まれなかった。checker-local syntax-free
fixtureはexact lower relation/両install orderをauthenticateし、private runner
はcommitted Task-255C1 routeまでを保持する。already frozen Task-257C2 route
がcompleted checker prerequisiteのfresh post-commit preflight後の最初の
runner consumerであり、そのconsumerはfixture/semantic detailを変更せず
現在実装・検証済みである。

## Checker Task 257C3 frozen harness boundary

future harness routeはexact Task-257C1 selector/lower builderをreuseし、lower
routeがreturnする前にseparate `1/1` predicate-chain compositionをinstallする。
existing extractor以外のraw traversal/new fixtureは追加しない。successはequal
typed/resolved handoffとsame empty semantic detail vectorを要求する。future
metadata変更はreciprocal sidecar reference/note 1件とcovered trace row 1件
だけで、本prerequisiteではどちらも変更しない。

## Checker Task 257C3 harness result

complete routeはlower atomic-only returnより先に実行し、exact syntax-free C3
handoffをfresh typed/resolved ownerへinstallする。extractorはexisting
predicate-chain syntaxだけをownし、new composition leafはlower handoffを
consumeしてsyntax/resolver rowをcopyしない。primary/atomic/composition/
arena mutation、exact replay、selector near miss、route isolation、
typed/resolved debug order、clone preservationをexactly runner tests 4件で
coverする。fixture/empty semantic detailはbyte-for-byte unchanged。

## Checker Task 258A frozen harness boundary

later harness leafはexact 81-byte future `MT10-FS` sourceをparse/resolveし、
Task-252 `2/2/0`、Task-256 `1/0/0/0/0/0/2/2`、Task-258A
`1/1/1/1/1`をone transactionで構築する。production-capableだが、`MT10-FS`
がdistinct `.miz`/singular sidecarを追加するまでcorpus dispatchはない。
library tests 4件がreal frontend/resolver pathをexecuteし、left/right
Task-252 stored use ordinalをexact 1/1、upstream binding/use source-event
lookup ordinalをseparate exact 1/2にfreezeする。

active type-elaboration route/detail/fixture/sidecar/trace row/admission ruleは
変更しない。existing reserved-variable equality caseはexact-name routeを
selectしてはならない。successはtyped/resolved same source-statement handoffと
same owned binding environment/fingerprint、absent Task-248 source context、
empty checked formula/statement/proof/fact/diagnosticを要求。本prerequisiteは
executable harness artifactを変更しない。

## Checker Task 258A harness result

dedicated private leafはlower Task-257C1 routeより先に実行し、real frontend、
declaration-shell symbol collection、label resolver、Task-48 binding bridge、
Task-252/256 builderを使用する。exact `1/1/1/1/1` handoffを
`source.statement.transport` source-preserved node hintだけとともにfresh
typed/final ownerへinstallする。library tests 4件がreal provenance path、
dependency/row corruption、loaded-source/subtree near miss、active-route
isolation、ownership exclusion、atomic replayをcoverする。future corpus
fixture/sidecarはabsentのまま。

runner libraryは369 tests、raw/normalized test-list hashは
`c5764bb1600242fe44db8c44b9c6bb18f39203a0de9ff60e301cbc6f172037f6` /
`1fd27b9bff190f95ac23d6de714a919a192fb0b7830aa3c98f960d4224c084aa`。
productionは30 paths / 34,955 lines、path/content hashは
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`dd399648aecadf2e7a63f685ad87577b7ebae9a9064fbfaba429a07d25ed9912`。

## Checker Task 258B1 frozen harness boundary

later exact private leafは139-byte final-LF
`FormulaStatementNestedContextSmoke` sourceだけをrecognizeし、library test外
ではdormantのまま。real parser/resolverを使用し、reserve binding baseへ
outer/nested proof contextをextendし、shared arena 1件でTask-252 `8/8/0`、
Task-256 `4/0/0/0/0/0/0/8/8`、Task-258B1 base `1/4/4/4/4`、
local-reference `1/1`をcomposeする。public resolver resultはexactly
proof-step label 1件/resolved local citation 1件を含み、runnerはどちらの
rowもsynthesizeできない。preliminary real 77-node/root-76 resolver arenaが
genuine node 68を供給し、second same-index passがresolver-produced
`Label(0)` key/resolved stateをそのnodeだけへattachしてから
`ResolvedAst::try_new`を呼ぶ。exact projectionはlabel-token node 12、
reference candidateはnode 68を使い、validated `ResolvedAst`/projection/
candidate/resultをreplayable syntax-free bundleとしてpassし、lossy result
tableだけに依存しない。exact `SurfaceNodeKind` parityはrunnerがvalidateする。

leafはTask 258A/lower Task-257 selectorより先に実行し、syntax/resolver
objectをchecker-owned rowへcopyせず、equal typed/resolved syntax-free
handoffだけをpublishする。future library tests exactly 5件がreal range/
provenance、two-pass/final keyed resolver-AST identity、context/visibility、
dependency/arena corruption、exact selector/subtree near miss、Task-258A/
active-route isolation、rollback/replay、empty semantic outputをcoverする。

本prerequisiteはrunner source/test、fixture、sidecar、expectation、trace
metadata/status/count、route、production manifest、test list、hashを変更しない。
assumption、witness、composite formula、broader label visibility、proof
meaning、acceptanceはTask 258B1外に残る。

### Task 258B1 implemented harness boundary

frozen routeを実装し、corpus-dormantのまま維持した。private leafはexact
source selection、real parser/resolver extraction、proof-context binding
extension、per-context Task-252 lowering、Task-256とstatement/reference
assembly、atomic final publicationを行う。semantic statement/proof payloadを
返さず、既存caseをactivateできない。

library test 5本でrunner listは374となった。raw/normalized hashは
`e8b5f54f219f8aa091014557c38ff8018d229ffbbc01cfa449bdc215826ca105` /
`99e6b7199e007707d1b4074b7079885e58378c4900a6811a7e1eb6cc02f9a2bf`。
productionは30 paths / 35,854 lines。

### Task 258B2 frozen harness boundary

Task-258B2 leafはexact 113-byte final-LF sourceとmeasured
55-node/root-54 unrecovered parser treeだけをrecognizeする。reserve、theorem
label、theorem/proof/assumption/conclusion range、全primary `x` term site
6件、origin `[2, 1]` / contribution 0のpublic/exported resolver theorem
1件をvalidateする。label/citation/reference key、witness、nested proof、
composite root、selector、extra statementはすべてnear miss。

leafはTask 258A/B1とlower Task-257 selectorより前に動く。raw syntaxをprivateに
Task-48 `2/1/0`、Task-252 `6/6/0`、Task-256
`3/0/0/0/0/0/0/6/6`、base-only Task-258B2 `1/3/3/3/3` handoffへ
translateし、raw syntaxをchecker rowへcopyしない。future test exactly 5本が
exact route、lower/resolver/table mutationとrollback、near miss、route
isolation、typed/final clone、empty fact/premise/checked-formula/
statement-semantic/proof/goal/diagnostic output不変をcoverする。

本documentation prerequisiteはrunner source、executable route、test、
corpus artifact、trace metadata、production manifest、test list/hashを
変更しない。witness、composite root、broader visibility、proof meaningは
Tasks 258B3–B5と269–272に残る。

### Task 258B2 implemented harness boundary

private leaf/facadeは2,120/678 lines、`runner.rs`は2,491 lines、statement
test leafは2,884 linesで実装済み。routeはcorpus-dormantのままprior
statement/lower selectorより先に実行する。all-index parityとcomplete
resolver mutation matrixがraw-syntax/provenance driftのsyntax-free checker
boundary越えを防ぐ。

### Task 258B3 frozen harness boundary

future leafは次だけをacceptする:

```mizar
reserve x for set;
theorem FormulaStatementSingleWitnessSmoke: x = x proof
  take x;
  thus x = x;
end;
```

exact 104 bytes、final LF、SHA-256
`76fb48354fc0dfb17047900a047a5b28b806df60d139a3133e606f0ef12a3f82`、
real unrecovered 49 nodes/root 48、theorem node 45、proof 44、take 35、
witness 34、transparent term wrapper 33、Task-252 term/reference site 32、
complete Task-252 sites 26/28/32/36/38、transparent wrappers
27/29/33/37/39、wrappers 31/41配下のTask-256 atomic sites 30/40、
conclusion 43、public/exported
theorem owner 1件をcheckする。resolver label/reference bundleはない。全
surface nodeはtyped arenaでsame-index range/children/recovery parityを保持。

leafはB2/B1/A/lower selectorより先に実行する。base
theorem/conclusion、witness take/item、Task-252 term 2、Task-256 formula
2件へdistinct typed ownershipを割り当て、exact paired
`1/2/2/2/2` + one-row handoffをpublishする。source ordinalsは
`[0,1,2]`をpartitionし、raw syntax/resolver objectはchecker rowへ渡さない。

compound tests 5本がexact output、全lower/base/witness/fingerprint/arena
mutation+replay、named/multiple/missing/extra witness、`take y`、reordered
statement、composite rootを含むsource/hash/subtree near miss、両order
family/active-route isolation、typed/final clone/debug、empty fact/
obligation/checked formula/statement semantic/proof/goal/diagnosticをfreeze。

sourceのequality goalによりvalid semantic `take` proofではない。routeは
corpus-dormantでformula-statement coverageをclaimできない。本docs
prerequisiteはexisting statement/facade/runner/test module
2,120/678/2,491/2,884 lines、379-test list、30-path / 36,479-line
manifest、全hashを不変に保つ。

Tasks 258B3N/MがB3後/B4前のnamed/multiple/other witness-term consumer、
Tasks 269–272がsemantic effectを保持する。

B3 consumerをexact private dormant selector/tests 5本としてimplementした。
runnerは384 library tests、productionは30 paths / 37,172 linesで、
path/content hashは
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`adfc81c21e69a91b194161525856aa40eb0e3ea76facfc2146dcb00b473ab3c2`。
corpus artifact/active routeは変更しない。

## Checker Task 258B3N dormant harness contract

future private selectorはexact 107-byte
`FormulaStatementNamedWitnessSmoke` sourceだけをmatchし、B3/B2/B1/Aとlower
formula routeより先にrunする。measured 51-node shared arena、
Task-48/252/256/base、named witness 1件、name row 1件をbuildし、
authenticated pairだけをinstallする。compound tests 5本がreal identity、
exhaustive mutation/replay、byte/subtree near miss、route/ownership isolation、
final empty semanticsをcoverする。active corpusやexternal detail keyは追加しない。

## Checker Task 258B3N dormant harness結果

private exact selectorとfive-test matrixを実装した。B3NはB3/B2/B1/Aより先に
走り、frozen bytesと51-node identityだけをacceptし、syntax-only
witness/name tableをpublishする。exhaustive lower/base/name、resolver、
all-node、near-miss、cross-family、active-corpus、replay、rollback、
final-empty-semantic checkがpassする。libraryは389 tests、productionは
30 paths / 37,555 lines。

## Checker Task 258B3M1 planned dormant harness contract

future private selectorはfinal-LF 113-byte
`FormulaStatementMultipleWitnessSmoke` sourceとcomplete 56-node arenaだけを
matchし、B3N/B3/B2/B1/Aより先にdispatchする予定である。existing
lower/base handoffをreconstructし、named witness 0、unnamed witness 1、
name row 0だけをbinding/semantic effectなしでpublishする予定である。両
witness rowsはone `take` source ordinalを共有し、dense ordinals 0/1を
保持する。

future compound tests 5本はsource/hash/parser/resolver/lower identity、
exhaustive mutation/replay、全byte/subtree near miss、active/cross-route
isolationのboth order、typed/final empty semanticsをcoverする予定である。
fixture、sidecar、trace row、external detail key、active dispatchは追加しない。

## Checker Task 258B3M1 dormant harness結果

private selectorとcompound tests 5本を実装した。routeはexact raw 56-node
parser tree、theorem-only resolver provenance、Task-48/252/256/base
dependency、dense `2/1` witness/name transactionをauthenticateしてから、
equalなtyped/final handoffをpublishする。mutation、near-miss、
cross-family、active-route、replay、rollback、empty-semantic checkがpassする。
private stale fingerprintはchecker test ownerに残し、runnerはmutation APIを
追加せずpublic equalityとcopied cross-profile rejectionを実証する。

runner libraryは394 tests。productionは30 paths / 38,103 lines、
statement leaf/facade/root/test sizesは`3724/688/2501/7246`。fixture、
sidecar、trace row/status/count、external detail key、active route、
semantic resultは変更していない。

## Checker Task 258B3M2A planned dormant harness contract

private statement selectorはSHA-256
`7b424949e98761b0179758065db5d164ad7d0a640f082801986683a54c43a2d1`
のfinal-LF 107-byte numeral-witness sourceだけをrecognizeする。dispatch前に
49 unrecovered parser nodes全部、exact theorem-only resolver provenance、
frontend diagnostics 0、Task-48/252/256/base profileをauthenticateする。
その後Task-252 primary numeral term 2 / numeric request 0をtargetするone
unnamed witness rowをpublishし、name、binding、atomic edge、active route、
semantic outputは追加しない。

selectorはB3M1/B3N/B3/B2/B1/Aより先で、byte、node、subtree、resolver、
lower table、numeral、numeric request、recovery、statement shape、
cross-familyの全near missをfail closedにする。exact five tests:

1. `task258b3m2a_real_frontend_freezes_numeral_witness_contract`;
2. `task258b3m2a_validation_precedence_mutation_and_replay_fail_closed`;
3. `task258b3m2a_selector_and_byte_subtree_near_misses_are_exact`;
4. `task258b3m2a_family_and_active_route_isolation_is_atomic_in_both_orders`;
5. `task258b3m2a_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`。

本documentation taskはfixture、sidecar、trace row/status/count、external
detail key、active dispatch、public mutation APIを追加しない。runner
baselineは394 tests / production 30 paths / 38,103 lines、
implementationは399 testsをprojectする。

## Checker Task 258B3M2A dormant harness result

private selectorとcompound tests 5本でfrozen contractを実装した。routeは
exact raw 49-node parser tree、theorem-only resolver provenance、
Task-48/252/256/base dependencies、dense reference/numeric-request
partition、`1 witness / 0 names`をauthenticateしてからequal typed/final
handoffをpublishする。precedence、mutation/replay、全byte/node/subtree
near miss、family/active-route isolationのboth order、rollback、debug
compatibility、`Some(Vec::new())`、empty semanticsがpassする。

runnerは399 tests、production 30 paths / 38,571 lines、statement
leaf/facade/root/test sizesは`4185/691/2505/8611`。fixture、sidecar、
trace row/status/count、external key、public mutation API、active route、
semantic resultは変更していない。

## Task 258B3M2B1 dormant consumer contract

private runnerはonly final-LF 113-byte/hash
`FormulaStatementParenthesizedWitnessSmoke`をselectし、53 nodes/root 52と
theorem-owner provenanceをauthenticateする。five rootsからsix primary
rows：wrapper term 2はchild term 3をcontainし、refs `0..4`は
`0/1/3/4/5`。atomicは`[0,1]` / `[4,5]`だけ、one unnamed witnessは
term 2をtargetしname/bindingなし。

exactly five compound tests：

1. `task258b3m2b1_real_frontend_freezes_parenthesized_witness_contract`;
2. `task258b3m2b1_validation_precedence_mutation_and_replay_fail_closed`;
3. `task258b3m2b1_selector_and_byte_subtree_near_misses_are_exact`;
4. `task258b3m2b1_family_and_active_route_isolation_is_atomic_in_both_orders`;
5. `task258b3m2b1_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`。

test 2はchild referenceを保持したwrapper term 2 reference追加、child
reference 2のremove/remap/duplicate、Task-256 edge/requestへのterm 2と
term 3の個別contaminationを独立にmutateする。test 3は
selector/subtree near missがpartial wrapper/child ownershipまたはdetached
child referenceをpublishできないことを確認する。

unparenthesized/nested/numeral/other-child、named/multiple、
application/structure/selector/update/set/choice、recovery、changed theorem、
composite/existentialはnear miss。authority-invalid theorem-proof
`take it;`もnear miss。exact outputは`Some(Vec::new())`、lookups `1/1`、
uses `[1; 5]`、empty semantics。route/key/artifact/count/hash変更なし。

## Task 258B3M2B1 dormant consumer implementation

exact selectorはprior statement profiles前に実装され、final-LF 113-byte
source、53 unrecovered nodes/root 52、frontend diagnostics 0だけをaccept
する。five rootsからsix primariesを作り、wrapper term 2はchild/reference
term 3をownし、equalitiesは`[0,1]` / `[4,5]`だけ、one unnamed witnessは
outer term 2をtargetする。resultは`Some(Vec::new())`、lookups `1/1`、
uses `[1; 5]`、empty semanticsを維持する。

five testsはexact identity、independent lower mutations、selector/subtree
near miss、prior statement + Tasks 253–255 + active isolationのboth
orders、rollback/replay、typed/final clone/debugをcoverする。valid
Task-252/256 handoffを作れないmalformed rowはowning public producerで
rejectし、constructible handoffだけpaired consumerへ進む。public
route/key/fixture/sidecar/trace/active/binding/semantic ownerは不変。

### Checker Task 258B3M2B2A dormant nested-parentheses contract

future dormant selectorはfinal-LF 121-byte/hash
`FormulaStatementNestedParenthesizedWitnessSmoke`だけをacceptし、
diagnostics 0と57 nodes/root 56をauthenticateする。five rootsからseven
Task-252 primariesを作り、outer wrapper 2 -> inner wrapper 3 -> reserved
variable term 4。refsは`0/1/4/5/6`、equalitiesは`[0,1]` / `[5,6]`だけ、
one unnamed witnessはouter term 2。complete `2/3/4` subtreeをTask-256外に
維持する。

future five testsはfrontend/resolver/lower identity、both parent links、
ref/independent subtree contamination、selector/byte near miss、prior
statement + Tasks 253–255 + active isolationのboth orders、replay/
rollback、typed/final clone/debugをcover。malformed Task-252/256は
lower-producer-first fail-closeを維持する。successは
`Some(Vec::new())`、lookups `1/1`、uses `[1;5]`、binding/semantic output
なし。active corpus/public route/key/fixture/sidecar/expectation/traceは
変更しない。

## Task 258B3M2B2B1P harness result

frozen compound tests 2件を実装してpassした。第1 testは143-byte source
SHA-256、Task-48 `2/1/0`、Task-252 `6/4/2`、Task-253
`1/0/1/2/2`、ordered targets/provenance、empty downstream tables、
legacy context-0 debug SHA-256
`9f1449159bf362bc90c4b41f3e4befb9a6d54f4152b836063f5cc07083d82a8d`
を固定。第2 testは全frozen context/root、wrapper/shape、range、form、
target、candidate/contribution、ambiguous provenance、stale replayを
rejectし、その後clean replayを証明する。fixture、expectation、sidecar、
trace row、active case、public diagnostic detailは追加しない。

## Checker Task 258B3M2B2B1A dormant harness contract

private selectorはfinal-LF 143-byte/hash
`FormulaStatementApplicationWitnessSmoke` sourceだけを認識する。SHA-256は
`22ce235030bc56720bfe7f52830182144ca6e4eee4414b7f8c2823e3d0f82c1b`。
publish前にdiagnostics 0、全63 nodes/root 62、theorem
node/range/path `59/48..142/[2,1]`、proof context 1 `103..141`、imported
`++` contribution/path `2/[12]`、Task-252 `6/4/2`、Task-253
`1/0/1/2/2`、Task-256 equalities `[0,1]`/`[4,5]`、base
`1/2/2/2/2`、witness `1/0`をauthenticateする。

extractorはtake/witness nodes 49/48をownし、unowned transparent node 47を
authenticate/traverseしてTask-253 application node/row `46/0`をtargetする。
node 47はwrapper/primaryではない。Task-252はnumerals 44/45をterms 2/3として、
Task-253はapplication/head/candidate/arguments/requests、Task-256は
theorem/conclusion equality termsだけをownする。B1P context-aware helperが
sole application producer。

exact five testsは
`task258b3m2b2b1a_real_frontend_freezes_application_witness_contract`、
`task258b3m2b2b1a_validation_precedence_mutation_and_replay_fail_closed`、
`task258b3m2b2b1a_selector_and_byte_subtree_near_misses_are_exact`、
`task258b3m2b2b1a_family_and_active_route_isolation_is_atomic`、
`task258b3m2b2b1a_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`。
全node/lower fingerprint、target/context/range/form、candidate/provenance、
numeric request、equality exclusion、base/witness precedence、both install
directions、prior Tasks 258/253-255、every byte、operand/operator/name/
cardinality/parenthesis/theorem/import/recovery near miss、replay/rollback/
final cloneをfreezeする。malformed lower rowsはowning producerでrejectし、
constructible wrong bundleはcombined consumerでpartial publicationなしに
rejectする。

successは`Some(Vec::new())`、lookups `1/1`、uses `[1; 4]`、equal
typed/final handoff。semantic term types、witness obligations、goal
matching/substitution、formula truth、proofs、terminal goals、
Core/ControlFlow/VC、cluster facts、diagnostics、active outputsは空。
fixture、sidecar、expectation、trace、detail key、public route、active caseは
追加しない。

## Checker Task 258B3M2B2B1A dormant harness implementation

dormant statement routeはexact real frontend/resolver outputs、Task-48
binding environment、Task-252/253/256 public handoffsをcomposeし、atomic
checker application/statement/witness installerをcallする。success profileは
contexts/bindings/diagnostics `2/1/0`、imported candidate/application/
argument/request provenance、equality edge pairs、lookups `1/1`、
`Some(Vec::new())` transport detailsを維持する。143 loaded-source bytes
全てとreparsed operator/name/import/recovery near missはselector/route
admissionにfailし、dependency/provenance/precedence/family-order/replay/
rollback/final-clone corruptionもfail-closeする。

exact runner tests 5件はpass。expression semantics、inferred types、
substitutions、obligations、proof steps、terminal goals、Core/ControlFlow/VC、
cluster facts、diagnostics、active outputsはempty。fixture、sidecar、
expectation、trace、detail key、public route、active caseは追加していない。

## Checker Task 258B3M2B2B1B1P dormant lower harness contract

motivating 158-byte sourceが含むnew lower shapeは1件だけ:
`ParenthesizedTerm 129..137 -> InfixExpression 130..136`。private B1B1P
harnessは、same imported `++` candidateとwrapper/application containmentを
authenticateしながら、proof context 1でshared Task-252 `6/4/2`とTask-253
`1/1/1/2/2`をcomposeする。Task-258 witness、statement、semantic term、
proof step、substitution、goalをbuildする前に停止する。

future compound tests 2件:
`task258b3m2b2b1b1p_wrapped_imported_application_proof_context_reuse_is_exact`
と
`task258b3m2b2b1b1p_wrapper_corruption_replay_and_legacy_outputs_fail_closed`。
共同で、final LFを含むloaded-source byte positions 158件全て、全67
nodesのkind/range/recovery/ordered childrenとroot identity、parsed
operator/name/import/parenthesis/recovery near missesをmutateする。
public/active Task-253 routeは全体を通じてunselectedのまま。

successは全application/wrapper/candidate/argument/request fields、complete
imported symbol/origin identity、typed/final clone parity、empty
semantic/proof/goal/diagnostic tablesをassertする。failure precedenceは
selector、Task-252、Task-253、stale-fingerprint typed installationの順。
combined corruptionがこの順序をproveし、全failureはclean replay前の
typed/resolved publicationがatomicにabsent/unchangedであることをprove
する。pre-change unwrapped context-0/context-1 row hashesはそれぞれ
`9f1449159bf362bc90c4b41f3e4befb9a6d54f4152b836063f5cc07083d82a8d`
と
`0fd83f61a40d3fd43816a52b70fca4fa4cf7f1d6e9172d3c5fe558c5d4add80d`。
separateな`Primary(0/1)`付き`2/0/2`と`Primary(2/3)`付き`6/4/2` rowsは
exactのまま。active case、fixture、sidecar、expectation、trace row、
detail key、public dispatchは追加しない。

## Checker Task 258B3M2B2B1B1P dormant lower harness implementation

private harnessはexact 158-byte/67-node wrapped applicationをselectし、
proof context 1でTask-252 `6/4/2`とTask-253 `1/1/1/2/2`をreuseする。
contribution 2、structural path `[12]`を含む全frozen candidate/origin
provenance fieldsをauthenticateし、legacy unwrapped pathsを
byte-compatibleに保つ。

exact tests 2件はpass。全loaded-source bytes/AST fields、same-source
resolver substitutions 5件、empty source familiesと全semantic/proof/goal/
diagnostic tables、atomic failure/clean replay、およびexact reparsed
`(diagnostics,nodes)` matrix
`[(0,63),(0,71),(0,67),(12,72),(1,64),(0,72),(0,67),(14,73)]`
をfreezeする。statement consumer、active case、fixture、sidecar、
expectation、trace row、detail key、public dispatch、downstream semantic
ownerは追加していない。

## Checker Task 258B3M2B2B1B1 dormant harness contract

B1B1 harnessはfrozen 158-byte/67-node `take (1 ++ 2);` sourceだけを
selectする。B1B1P wrapped Task-253 seamをreuseし、existing atomic checker
path経由でbase `1/2/2/2/2`と`Application(0)`をtargetするone unnamed
witnessをpublishする。wrapper 0はTask-253 containmentのままでwitness
targetではない。

exact successful witnessはowner/context/source/witness ordinal
`0/1/1/0`、take `53/124..138`、item `52/129..137`、normalized spelling
`( 1 ++ 2 )`、normal/unnamed/nameなし。exact lower handoffはapplication
`48/130..136`、wrapper `50/129..137`、head `20/132..134/++`、ordered
`Primary(2/3)`、imported `parser.type_fixtures::++#12` candidate。
theorem ownerはcontribution 0、`LocalSource` anchor `29..47`、origin
`48..157/[2,1]`、label `56..108`。

named runner tests 5件は全source bytes/arena fields、same-source resolver
substitutions 5件、exact reparse matrix、selector/lower/aggregate/witness/
typed/final precedence、B1A compatibility、全family/active-route isolation、
atomic rollback/clean replay、final clone equality、empty semantic/proof/
goal/overload outputをexhaustする。exact successはdetail keysなし。
fixture、sidecar、expectation、trace row、active case、semantic consumerは
authorizeしない。

## Checker Task 258B3M2B2B1B1 dormant harness result

exact B1B1 selector、resolver substitutions、full byte/node mutation matrix、
family/active isolation、atomic replay/rollback、clone、empty upper-table
assertionsをpassing runner tests 5件として実装した。runner libraryは423
tests、statement-test leafは13,381 lines。fixture、expectation、sidecar、
trace row、active detail key、semantic consumerは変更していない。

## Checker Task 258B3M2B2B2P dormant lower harness contract

B2P harnessがfreezeするのは、`take TypeCaseStruct(x: 1, y: 2);`を含む
exact final-LF 172-byte/76-node sourceだけ。existing proof binding context、
shared Task-252 `6/4/2`、Task-254 `1/0/1/2/0/2/6`をcomposeした時点で
停止し、Task-258 statement/witnessはconstructしない。

successful lower ownershipはexact。constructor node 59だけが
`source.term.structure.constructor`、member token nodes 20/24だけが
`source.term.structure.member.constructor-assignment`をownする。
qualified root 52はauthenticated imported resolver traversalだが
`source.surface.unowned`のまま。Task 252はnodes 54/57をprivate extraction
rootsとしてのみ使用してnumeral rowsをsites 53/56でpublishするため、
53/56は`source.term.numeral`、54/57は`source.surface.unowned`のまま。
constructorはproof context 1、members `x/y`、
ordered `ConstructorValue` edges -> `Primary(2/3)`、ordered unresolved
requests 6件、application fingerprintなし。

imported rootはexact
`summary:parser.type_fixtures#parse-only#TypeCaseStruct:5` /
`parser.type_fixtures::TypeCaseStruct#5`、contribution 2、origin
`7..27/[5]`、public/exported、signatureなし。runner-private seamはpublic
Task-254 producerとalready-built binding/source-term partsをreuseし、
Task-252 rowsをduplicateせず、field identityをsynthesizeせず、existing
Task-254 real routeをbroadenしない。

future compound tests 2件:
`task258b3m2b2b2p_structure_constructor_proof_context_reuse_is_exact`と
`task258b3m2b2b2p_structure_constructor_corruption_replay_and_legacy_output_fail_closed`。
final LFを含む172 bytes全て、76 nodesのkind/range/recovery/ordered
children/root identity、reparsed import/root/member/value/recovery near
misses、exact rows/owned kinds/imported provenance、context/root/member/
edge/request substitutions、lower validation precedence、stale failure後の
clean replay、byte-identical legacy Task-254 outputをcoverする。全upper
source familiesとsemantic/proof/goal/IR tablesはemptyのまま。

active case、public dispatch、statement consumer、fixture、sidecar、
expectation、trace row、detail key、checker testは追加しない。future B2A
contractだけがwitnessを`Structure(0)`へattachできる。selectorおよび
functional-update/`FieldUpdate` witnessesはB2B/B2Cのまま。

## Checker Task 258B3M2B2B2P dormant harness result

exact-source selector、owned-kind map、shared Task-252 parts、existing proof
context、imported provenance、Task-254 handoff、mutation matrix、stale replay、
legacy hashesをpassing runner tests 2件でimplementした。全upper source/
semantic/proof/goal/IR tablesはempty。active case、fixture、sidecar、
expectation、trace row、detail key、checker test、statement consumerは
変更していない。

completed pairはTask-48/252/254
`2/1/0`/`6/4/2`/`1/0/1/2/0/2/6`、ownership 59/20/24、numerals
53/56、unowned 52/54/57、exact `TypeCaseStruct#5` provenance、
malformed recovery `diagnostics=1, nodes=74, root=73, recovered=[52]`を
pinする。current Task-254 source-structure/typed/final hashes
`0d6af57b89e6156d8e5de6831568c81ec110880bebf1e4aeb4ab00563f4da6c8`,
`8264d1574faf67e19b6b84d6e11fa7ab6435335238b398fa0966bbfbc63d0599`,
`118a998bc5edb770c7818be1d74cbece0f566353bf9d3e6aabb817d994a3db40`
は不変。

## Checker Task 258B3M2B2B2A frozen dormant harness

future private statement harnessはhash
`24e2ee2332ead5c0d46025df6044450eeab3ebb5733ebe83587ceae3ba129eb6`
のunchanged final-LF 172-byte/76-node/root-75 zero-diagnostic
constructor-witness sourceだけをselectする。B2P exact owned-kind selector/
proof-context Task-254 seamを先にreuseし、Task-48 `2/1/0`、Task-252
`6/4/2`、Task-254 `1/0/1/2/0/2/6`、equality-only Task-256
`2/0/0/0/0/0/0/4/4`、Task-258 base `1/2/2/2/2`、one unnamed
`Structure(0)` witness/no namesをauthenticateする。

Task-258 base transactionはtheorem/conclusion statement rows 72/70をown。
B2A extensionはtake/witness nodes 62/61とwitness-to-structure edgeだけを
ownする。constructor/member 59/20/24はTask 254、term/numeral sites
45/47/53/56/63/65はTask 252、equality 49/67はTask 256、
root/extraction/transparent/container nodesはunowned。current theorem
provenanceはlocal anchor `29..47`、checked owner `48..171/[2,1]`、
owner/contribution 0、public/exported/normal label、no import edge/recovery。
imported `parser.type_fixtures::TypeCaseStruct#5` provenanceはcontribution
2、origin `7..27/[5]`、public/exported/signature-free/normal。両方exact
selector。Task 256はdirect structure edge/fingerprintを持たず、
combined typed/final boundaryだけがexact structure handoffでrevalidateする。

runnerはauthenticated syntax-free lower handoffsをnew checker
canonical planでfreezeしたchecker-owned full structure-aware builder/full
atomic structure-statement-witness installerへ渡し、parser/resolver valuesを
exportせずB2P extractionをduplicateしない。lower installation、
aggregate/base rows、witness、typed publication、final cloneの順でvalidate
する。failureはpartial stateを残さずfresh replayはsuccess。

frozen tests 5件は
`task258b3m2b2b2a_real_frontend_freezes_structure_constructor_witness_contract`、
`task258b3m2b2b2a_validation_precedence_mutation_and_replay_fail_closed`、
`task258b3m2b2b2a_selector_and_byte_subtree_near_misses_are_exact`、
`task258b3m2b2b2a_family_and_active_route_isolation_is_atomic`、
`task258b3m2b2b2a_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`。

全172 bytes/76 nodes fields、local/imported resolver substitutions、全lower/
base/witness rows/fingerprints、dependency order/family hybrids、全ownership/
active orders、rollback/replay/final clone、legacy/application compatibility、
malformed recovery `1/74/root 73/[52]`をcoverする。semantic/proof/goal/
overload/Core/CFG/VC outputsはempty。active case、fixture、expectation、
sidecar、trace row/credit、diagnostic detail、public runner routeは変更しない。

documentation baselinesはchecker/runner tests `378/425`、runner
statement/structure/facade/root/statement-test/structure-test sizesは
`5962/2857/715/2531/13381/2991`。implementationはrunner 430 testsを
projectする。

## Checker Task 258B3M2B2B2A dormant harness result

private harnessはfrozen constructor-witness sourceだけをrecognizeし、
Task-48/252/254/256/baseと`Structure(0)` witness 1件をcomposeする。parser/
resolver tablesをcopyせずB2P ownership/provenance selectorをreuseする。
exact five named testsは全172 bytes、B2P seam経由の76 node/root fields、
malformed recovery、dependency/base/witness mutation/replay、family
isolation、typed/final clone/empty semanticsを含めPASS。

runner libraryは430 tests。active case、fixture、expectation、sidecar、
trace row/backlink/credit、diagnostic detail、public runner routeは追加なし。
B2B/B2Cとsemantic/proof/goal/overload/Core/CFG/VCはdeferredまたはempty。

## Checker Task 258B3M2B2B2BP frozen private selector harness

exact 171-byte/79-node direct-selector sourceのTask-254 proof-context lower
reuseだけをfreezeする。Task-48 `2/1/0`、Task-252 `6/4/2`、Task-254
`2/0/1/3/0/3/9`、imported `TypeCaseStruct#5`、owned nodes
`62/61/29/20/24`、chain
`Structure(0) -> Structure(1) -> Primary(2/3)`をauthenticateし、
Task-256/258 outputはpublishしない。

frozen testsは
`task258b3m2b2b2bp_structure_selector_proof_context_reuse_is_exact`と
`task258b3m2b2b2bp_structure_selector_corruption_replay_and_constructor_compatibility_fail_closed`。
全source bytes/node fields、lower rows/fingerprints、provenance/owned map、
context/range/source/member/edge/request corruption、exact 170-byte
missing-selector-name near miss（`149..150`の
`malformed_term_expression`、78 nodes/root 77、recovered `[]`）をcover。
validだがexcludedなselector/call/chain/wrapped/base/update、precedence、
rollback/replay、constructor compatibility、empty upper tablesもcover。

active case、public route、fixture、expectation、sidecar、trace credit、
diagnostic detail、checker test、semantic behaviorは追加しない。

## Checker Task 258B3M2B2B2BP private selector harness result

frozen private harnessを実装し、named tests 2件はPASSする。valid pathは
Task-48/252、全79 surface nodes、imported root provenance、owned nodes
`62/61/29/20/24`、全lower rows、current Task-252 fingerprintを認証した
後だけexact Task-254 `2/0/1/3/0/3/9` bytesをpublishする。全mutationは
fail closedし、clean replayはhandoff/TypedAst/ResolvedTypedAst debug
bytesを再現する。

missing-selector near missはsole syntax diagnostic
`malformed_term_expression` at `149..150`として直接認証する。B2P/B2Aと
legacy Task-254 compatibility hashesはexact。runner libraryは`432`で、
active cases、fixtures、sidecars、expectations、trace credit、
diagnostics、semantic outputsは不変。

## Checker Task 258B3M2B2B2B frozen runner harness

harnessはselector witness
`TypeCaseStruct(x: 1, y: 2).x`を持つexact 171-byte final-LF sourceを使う。
79 nodes/root `78`、Task-48 `2/1/0`、Task-252 `6/4/2`、Task-254
`2/0/1/3/0/3/9`、Task-256 `2/0/0/0/0/0/0/4/4`、Task-258 base
`1/2/2/2/2`、witness `1/0`をauthenticateする。`.x`を`.`へreplaceした
exact 170-byte near missはsole `malformed_term_expression` at
`149..150`、78 nodes/root `77`、`recovered = []`。

required runner tests 5件:

- `task258b3m2b2b2b_real_frontend_freezes_structure_selector_witness_contract`
- `task258b3m2b2b2b_validation_precedence_mutation_and_replay_fail_closed`
- `task258b3m2b2b2b_selector_and_byte_subtree_near_misses_are_exact`
- `task258b3m2b2b2b_family_and_active_route_isolation_is_atomic`
- `task258b3m2b2b2b_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`

全source byte/node field、provenance/ownership row、subtree exclusions、
全lower fingerprints、validation precedence、clean replay、B2A/B2B hybrids/
family orders、active-route isolation、debug stability、atomic rollback、
final clone、empty semantic tablesをcoverする。existing fixtures、
expectations、sidecars、trace metadata、active cases、diagnostic credit、
CLI behaviorはunchanged。

## Checker Task 258B3M2B2B2B dormant harness result

private harnessはfrozen 171-byte/79-node selector-witness sourceだけを
recognizeする。existing B2BP owned-kind/proof-context handoff seamsを
consumeし、exact Task-48/252/254/256/258 tablesをcomposeして、selector
`Structure(0)`をtargetとするunnamed witness 1件をinstallする。lower
parser/resolver rowはcopyもrelaxもしない。

exact runner tests 5件は全てPASS。all source bytes/node fields、complete
local/imported provenance/ownership、lower/base/witness corruptionと
validation precedence、exact `malformed_term_expression` near miss、
valid excluded selector forms、B2A/B2B/active-family isolation、rollback/
replay、final clone、empty semantic/proof/goal/overload/Core/CFG/VC
outputsをcoverする。

runner libraryは`437`。public/active route、fixture、expectation、sidecar、
trace row/backlink/credit、diagnostic credit、semantic behaviorは追加なし。

B2Bはimplementation commit `8311502c`でcloseし、clean fresh inventoryは
B2Cより先にB2CPをselectする。

## Checker Task 258B3M2B2B2CP frozen private harness

dormant harnessはfinal-LF 181-byte、SHA-256
`03f14a98bffb557ea4dda4f879bf504d241aaebae0552a97f0f2417ef4b43560`、
86-node/root-85 `FormulaStatementStructureUpdateWitnessSmoke` sourceだけを
freezeする:

```text
import parser.type_fixtures;
reserve x for set;
theorem FormulaStatementStructureUpdateWitnessSmoke: x = x proof
  take TypeCaseStruct(x: 1, y: 2) with (x := 3);
  thus x = x;
end;
```

proof context 1 at `107..179`とexact Task-48 `2/1/0`、Task-252
`7/4/3`、Task-254 `2/0/1/3/1/4/9`をreuseする。Task-252 extraction
rootsは`51/53/60/63/67/73/75`、published sitesは
`51/53/59/62/66/73/75`。Task 254はupdate/constructor/member/
`FieldUpdate` nodes `69/65/30/20/24/68`をownし、imported
`TypeCaseStruct#5` contribution 2 at `7..27/[5]`をauthenticateし、
exact update-base/update-value/constructor-value edgesとnine-request
orderをpreserveする。

Task 256がownするのは`BuiltinPredicateApplication` nodes `55/77`だけ。
formula containers `56/78`とcomplete update subtreeはexcluded。harnessは
Task-256/258、statement、witness、checker/public API、active route、
diagnostic、semantic outputをownしない。

future tests exactly 2件:

- `task258b3m2b2b2cp_structure_update_proof_context_reuse_is_exact`
- `task258b3m2b2b2cp_structure_update_corruption_replay_and_prior_sibling_compatibility_fail_closed`

全source byte/node field、imported root、lower rows、update-path/
`FieldUpdate` ownership、edge/request order、corruption/precedence、
stale/clean replayをcoverする。complete `with (x := 3)` fragmentを
`with (x := )`へreplaceしたnear missはexact 180-byte SHA-256
`8310de3b172cea98e4e85ebc6021c85c4e1bd7c2a74f8cd99413ae5a80569d67`、
sole `malformed_term_expression` at `158..159`、84 nodes/root 83、
`recovered = [65]`。

valid excluded base-only/selector/wrapped/multi-update/nested-path formsは
seam scope外。B2P constructor/B2BP selector双方のcompatibilityをexactに
preserveする。checker test、statement consumer、active case、fixture、
sidecar、expectation、trace row、detail key、semantic behaviorは追加なし。
functional-copy semantics、update result typing/identity、witness
obligations、theorem/proof acceptance、goals、IRはdeferred。特に
`x = x` goal下の`take`はsemantic acceptance claimではない。

## Checker Task 258B3M2B2B2CP private harness implementation

CPC1 commit `ee267d9c`はcomplete。frozen runner files 4件はprivate/
corpus-dormantなupdate reuse seamだけをimplementし、frozen B2CP tests
exactly 2件がPASS。direct table comparisonは全Task-48/252/254 rows、
replay/corruption、B2P/B2BP compatibilityをauthenticateする。これで
prerequisite `design_drift`、bounded `source_drift`、`test_gap`はclose。
final test-sufficiency/implementation re-reviewsはfindingsなし。

checker/runner librariesは`386/439`、runner sizesは
`6826/6065/730/2546/17120/5848`。productionは30 paths / 46,788 lines、
hashesは
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`bbcc55ab769fb5b725de83a27ae13243000a1610a12064907c06187417e45b5f`、
test-list hashesは
`ea3e854c1b741ab4b642000df6610a15e521f0849b39e7480820ca86680a1d0e` /
`11e6de35b422b913c235d8193fb2629da5aff39d1cf251af1c6cec2824301c8d`。
checker/corpus/CLI hashesはunchanged。

fixture、sidecar、expectation、trace status/count/backlink/credit、
public/active route、semantic changeはなし。formula creditは`deferred`、
`tests = []`、audit impactはnarrative-only。B2Cと全functional-copy/type/
proof/goal/IR deferralはunchanged。concurrent ownershipはreport-only
`repo_metadata_conflict`でmetadata repairなし。fmt、Clippy、tests、
全count/hash gatesはPASS。final source/documentation re-reviewは
findingsなし。independent final qualityはfindingsなし、全9 hard gates
PASS、valid `98/100`。B2CP implementation commit
`b146f0f72dceac2233c9d679b7820e264974b227`とclean B2C fresh inventoryは
complete。

## Checker Task 258B3M2B2B2C frozen runner harness

B2CP commit `b146f0f72dceac2233c9d679b7820e264974b227`はcomplete。B2Cは
exact final-LF 181-byte SHA-256
`03f14a98bffb557ea4dda4f879bf504d241aaebae0552a97f0f2417ef4b43560`
source、diagnostics 0、86 unrecovered nodes/root 85を使う。

theorem/label/proof/take/witness/transparent/update/constructor/rootは
`82/11/81/72/71/70/69/65/58`、`FieldUpdate`/update member/constructor
membersは`68/30/20/24`、numeralsは`59/62/66`、conclusion/equalities/
containersは`80/55,77/56,78`。exact rangesはcanonical EN planと同一。

missing-valueは180 bytes、SHA-256
`8310de3b172cea98e4e85ebc6021c85c4e1bd7c2a74f8cd99413ae5a80569d67`、
sole malformed `158..159`、84/root83、recovered `[65]`。valid excluded
profilesはbase-only 167/
`bb26a425d2bc16e6518d6366128de138862c4525af6eb82b748e4cb28f1b8bc9`/
`76/75/[]`、selector 169/
`64039fca35d6199fea281d43df6dafdfeff78f1d97139d6286a3082115552747`/
`79/78/[]`、wrapped 183/
`e1a2b79cb03a4aebc5e0e29150cde382da457aa31cb8e66643eecce6e8296ae6`/
`90/89/[]`、multi 189/
`a95336dc08b9534d7c5c16ca5070384e2610f0db31841187878b68b4403666b6`/
`93/92/[]`、nested 183/
`92440b4b3814d7b8a738bf71b2e89b9056fbb382301e12b5f4a4ccab17e0f082`/
`88/87/[]`で全てdiagnostics 0。

harnessはTask 48 `2/1/0`、Task 252 `7/4/3`、Task 254
`2/0/1/3/1/4/9`、Task 256 `2/0/0/0/0/0/0/4/4`、Task-258 base
`1/2/2/2/2`、witness `1/0`をcompare。base input factsは2件でreference
uses `[0,1]`/`[2,3]`、後者はprimary terms `5/6`をresolveする。
LocalSource contribution anchor `29..47`、owner origin
`48..180/[2,1]`、label `56..99`、statements
`82/48..180/Atomic(0)/ordinal 0`と
`80/164..175/Atomic(1)/ordinal 2`、two candidatesもverify。unnamed
witness at 72/71はproof context 1でonly `Structure(0)` target。
existing B2CP three private seamsはunchanged consume。

runner tests exactly 5:

- `task258b3m2b2b2c_real_frontend_freezes_structure_update_witness_contract`
- `task258b3m2b2b2c_validation_precedence_mutation_and_replay_fail_closed`
- `task258b3m2b2b2c_update_and_byte_subtree_near_misses_are_exact`
- `task258b3m2b2b2c_family_and_active_route_isolation_is_atomic`
- `task258b3m2b2b2c_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`

paired checker tests exactly 4:

- `task258b3m2b2b2c_exact_structure_update_witness_api_debug_and_legacy_compatibility_are_stable`
- `task258b3m2b2b2c_dependencies_structure_update_witness_precedence_and_all_nodes_fail_closed`
- `task258b3m2b2b2c_combined_ownership_hybrids_and_all_family_orders_are_atomic`
- `task258b3m2b2b2c_final_clone_revalidation_and_semantic_deferrals_are_stable`

exact nine testsで全byte/node/table/provenance/ownership/
excluded/malformed/family/mutation/order/fingerprint/hybrid/rollback/
replay/final clone/empty semantic surfacesをcoverする。public/active/
fixture/expectation/sidecar/trace/semantic changeなし。§13.3.3 authority
から`spec_gap`なし。`x = x`下の`take`はsource transport only。
baseline/projection `386/439` -> `390/444`、all hashes unchanged。
4 independent reviewsはfindingsなしで、complete documentation/count/hash
verificationもPASS。independent final qualityはfindingsなし、全9 hard
gates PASS、valid `98/100`。commitとimplementation inventoryはopen。

## Checker Task 258B3M2B2B2C implemented runner harness

harnessはfrozen 181-byte/86-node functional-update sourceだけをrecognizeし、
B2CP update extractor/producer boundaryをreuseして、Task258
statement/witness producers前にexact lower tablesをassembleする。witnessは
`Structure(0)`をtargetとし、constructor/update value/equality operands/
resolver provenance/ownership exclusionsはexisting producersのまま。

frozen runner tests 5件はreal frontend、validation precedence/replay、
malformed/valid-excluded byte/subtree near misses、family/active isolation、
typed/final/debug/rollback/empty semanticsをcoverしてPASS。paired checker
tests 4件もPASS。final test-sufficiency/implementation reviewsはfindingsなし。
runner library `444`+policy suitesはPASSし、broad workspace/remaining final
reviewsはpending。

active fixture、expectation、sidecar、trace row/credit、diagnostic、
semantic/proof/goal/IRはunchanged。formula-statement trace rowは
`deferred`, `tests = []`のまま。

## Checker Task 258B3M2B2B2C broad harness verification

fmt、workspace Clippy、checker/runner crate/policy suites、full workspace
tests、focused `4/4`/`5/5`、sibling `12/12`/`21/21` suitesはPASS。
fresh counts/hashesはpaired plansと一致する。active/fixture/trace-credit/
diagnostic/semantic harness surface変更なし。independent final
consistency/quality、commit、post-commit gatesはpending。

## Checker Task 258B3M2B2B2C final harness review status

independent final source/docs consistency/final qualityはどちらも
**NO FINDINGS**。全9 hard gates PASS、valid score `98/100`で、exact
harness evidence/boundariesはunchanged。pendingはcached-diff/staging
audit、implementation commit、post-commit inventory/fresh-next-task
gatesだけ。

## Checker Task 258B3M2B2B3P frozen private harness

B2C commit `e8373c683448e524cb98edde83fdf8de83a125cd`後、B3Pはexact
117-byte set-enumeration sourceのprivate proof-context reuse harnessを
freeze。real frontendは57 nodes/root 56、local-only resolver、Task48
`2/1/0`、Task252 `6/4/2`、Task255 `1/0/0/0/0/2/1`をreproduceする。
enumeration term 0はsite `Node(40)`、range `90..96`、source ordinal 0、
context 1、recovery `Normal`、spelling `{ 1 , 2 }`、kind
`Enumeration`。`EnumerationElement` edgesはexactly
`(term 0, ordinal 0, Primary(2))`と
`(term 0, ordinal 1, Primary(3))`。request 0はterm 0、ordinal 0、
`ResultType`、`generator = None`、`type_site = None`。primary fingerprint
はexact Task252 handoff、application/structure fingerprintsはabsent。

private explicit-context helperはpre-existing context-0 helper/output bytesを
変更しない。exact 2 tests togetherで:

- final LFを含むloaded-source 117 bytes全件をmutateしstripped/extra-LFをreject;
- 57 nodes全件のkind/range/recovery/ordered childrenとroot identityをmutate;
- local resolver shell/symbol/contribution/provenance全fieldsをassert/substitute;
- Task48 context/binding、Task252 primary term/reference/numeric request、
  Task255 term/`EnumerationElement` edge/request/fingerprintの全fieldsを
  assert/mutate;
- owner Task252 `{30,32,36,38,44,46}`、Task255 `{40}`、unowned
  `0..29,31,33..35,37,39,41..43,45,47..56`をexact assert;
- source/module selectorからarena/root、resolver、Task48、Task252、
  Task255、stale fingerprint、typed/final clone validationまでのprecedence、
  atomic rollback、clean replay、exact final clonesをfreeze;
- empty Tasks253/254/256/258、active/adjacent isolation、empty semantic/
  proof/goal outputsをassert。

legacy Task111 context-0 oracleはliteral assertする。Task255 handoff debug
SHA-256
`30b72230bb7ff39464962133b58df212e23afccccc8f4e4788ab9a9d0481c43a`、
full typed debug
`1bb296c06ab62691684260aa94987adee23081baa4a35aac9e485d95370d2cb9`、
resolved debug
`cdb4eaae9605f62269d6a74d64267a8fcb1e8d8008564d8b9e014037665df1e4`。
implementation build内のold/new equalityはoracleではない。

checker test、active fixture/sidecar/route、expectation、trace row/credit、
public API、statement witness、imported behavior、semantic behaviorを追加しない。
upper B3Aがlater witness-to-set-term consumerとseparate checker/runner testsを
ownする。

## Checker Task 258B3M2B2B3P documentation review status

4 review tracksはすべて**NO FINDINGS**で、record済み
source/count/hash/scope/trace-no-op checksはPASS。exhaustive two-test
harness contractはfrozen、future implementationはplanned
`source_drift`/`test_gap`。final quality、commit、post-commit、
fresh implementation inventoryはpending。

## Checker Task 258B3M2B2B3P final quality status

final qualityは**NO FINDINGS**、全9 hard gates PASS、valid `98/100`
（`20/20/15/14/10/10/5/4`）。pendingはstage/commit、post-commit、
fresh implementation inventoryだけ。

## Checker Task 258B3M2B2B3P implemented private harness

`285a1f11c310bb313c4c6b4feae914eb11f74754`のcontractに対しexact
2 testsがPASS：

- `task258b3m2b2b3p_set_enumeration_proof_context_reuse_is_exact`
- `task258b3m2b2b3p_set_enumeration_corruption_replay_and_legacy_output_fail_closed`

117 bytes/final-LFとstripped/extra LF、57 nodesのkind/range/recovery/
children/root、independent resolver 63 fields、binding 39 fields、
Task-252/255全rows、real prior-binding use-ordinal substitution、coherent
application/structure dependenciesをcover。shared fingerprint-only
subprofileにより各absence clauseはindependently observable。各reject直後
clean replay、stale/simultaneous precedence、typed/resolved clone rollback、
owner partitions、semantic emptiness、legacy hashes、active/adjacent
isolationもfixed。

focused `2/2`、runner library `446/446`、fmt、package/workspace
Clippy/tests、lint-policy `15/14`、metadata `137`、5 CLI/current
manifest/test-list hashes、diff、exact30 scopeはPASS。test-sufficiency/
implementation/source-docs consistency repeat/documentation-boundary
repeatは**NO FINDINGS**。independent final qualityは**NO FINDINGS**、
全9 hard gates PASS、valid `98/100`（`20/20/15/14/10/10/5/4`）。
pendingはcommit/post-commit、fresh B3Aだけ。

## Checker Task 258B3M2B2B3A frozen runner harness

exact runner testsは次の5件：

1. `task258b3m2b2b3a_real_frontend_freezes_set_enumeration_witness_contract`
2. `task258b3m2b2b3a_validation_precedence_mutation_and_replay_fail_closed`
3. `task258b3m2b2b3a_set_enumeration_and_byte_subtree_near_misses_are_exact`
4. `task258b3m2b2b3a_family_and_active_route_isolation_is_atomic`
5. `task258b3m2b2b3a_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`

final LF/117 bytes、57 nodes全fields/root、local resolver label/owner、
全Task48/252/255/256/258 rows、witness1/names0、partition/graph、
fingerprints、empty/singleton/3+/parenthesized/nested/comprehension/choice/
`qua`/label near misses、全family hybrid/order、immediate replay/rollback、
final clone/debug/isolation/empty semanticsをauthenticate。

precedenceはsource/AST、resolver+label、Tasks48、252、255、256、258 base、
witness、atomic publication、final clone。real fixture/expectation/traceは
unchanged/inactiveで、existential goal/proof acceptance/active creditなし。

## Checker Task 258B3M2B2B3A implemented runner harness

exact routeはunchanged B3P set-enumeration handoffをconsumeし、frozen
witness1/names0 set edgeをpublishする。named runner5 testsはreal frontend、
exact resolver label、全lower/upper rows、Task-256 `72`、Task-258 `62`、
witness `21` field matrices、全`57` surface nodes/root、両final-LF near
miss、family tuples/routes、rollback/replay、final clone/debug、empty
semanticsをcoverする。paired checker4 testsもexactのまま。
fixture/expectation/trace/corpus activationやsemantic/proof creditは変更なし。
2回目のsource/documentation consistency repeatとfinal documentation/
boundary rereadは**NO FINDINGS**で、crate plans記載のparent final
verificationはexact `39`-file scopeを含めPASS。independent final
read-only quality reviewは**NO FINDINGS**。全9 hard gates PASS、score
capなし、valid `98/100`（`20/20/15/14/10/10/5/4`）。記載済み
semantic/coverage deferralsはunchanged residual risk。pendingは
dedicated implementation commit、postcommit invariant verification、
fresh next-task inventoryだけ。

## Task 258B3M2B2B3B dormant runner surface

B3Bは118-byte
`FormulaStatementEmptySetEnumerationWitnessSmoke` inputのexact dormant
source selectorとcompound runner tests 5件だけを追加する。selectorは
existing private detail routeをpublishする前に、全byte/node、resolverと
lower handoff fields、owner partition、zero-edge graph、family isolation、
replay/rollback、final clone、empty semanticsをauthenticateしなければ
ならない。active discoveryへenterせず、diagnostic/detail keyを変更せず、
inactive template fixtureをreinterpretしてはならない。

## Task 258B3M2B2B3B implemented runner harness

frozen runner tests 5件はexact 118-byte inputと全50 nodes/root 49をexercise
する。byte matrixはall 118 positions、node matrixはkind/range/recovery/
childrenの4 surface mutation axesでall 50 nodesをcoverする。8
base-resolver mutations、10
label-resolver mutations、currently constructibleな全Task-48/252/255/
256/258 handoff fields、frozen Task-256 `72`、Task-258 `62`、witness
`21` field matricesをcoverする。omitted Task-258 kind/role/status 4 fields
は各々safely constructibleなpublic variantが1件だけなので、repeat
reviewerはcandidate findingを**NO DISAGREEMENT**としてretractした。

suiteはfrozen zero-edge contractに対するnon-vacuous 2-edge rejectionと、
B3A-before-B3B / B3B-before-B3Aの両family ordersを含む。active-route
isolation、immediate replay/rollback、final clone/debug stability、empty
semanticsもverifyする。resolver coverage、bidirectional ordering、
non-vacuous zero-edge validationのinitial findingsは同じrunner 5 /
checker 4 tests内でcloseした。repeat reviewで残ったB3B-specificな
currently mutable Task-48/252/255 mutation/replay coverage gapも、test数、
fixture、expectation、trace row、active routeを変えず、exact Task-48
`32`、Task-252 `55`、Task-255 `23` matricesでremediateした。focused
runner `5/5`、checker `4/4`、libraries `398/456`、format/diff、workspace
Clippy `-D warnings`、final `cargo test -q`はPASS。post-auth injectionと
stage-prefix/non-generic-guard assertionsでauthenticationをcompleteし、
全test-sufficiency repeatsとfinal implementation repeatは
**NO FINDINGS**である。

four surface axesとtest-list hashesのfinal remeasurement記録をconfirmした
source/documentation consistency repeatも**NO FINDINGS**である。

## Task 258B3M2B2B3C frozen harness

dormant harness selectorはexact `110`-byte、`52`-node/root-`51` sourceと
local theorem provenanceだけをauthenticateし、proof context `1`でexisting
explicit-context Task-255 choice handoffをassembleする。future testsは全
bytes/LF、各node/rootのfour surface axes、resolver、
`32/55/39/72/62/21` lower/upper fieldsのreplay/stage prefix、
non-vacuous zero-edge、choice target/request order、ownership、family
isolation、clone/rollback/debug stability、empty semanticsをexhaustする。
corpus/expectation/sidecar/trace count/CLI/active routeは変更しない。

## Task 258B3M2B2B3C implemented runner harness

dormant selectorとfrozen runner tests 5件はexact 110-byte source/final LF、
全52 nodes/root 51のkind/range/recovery/children axes、base 8 + label 10
resolver mutationsのtyped/resolved replay、全currently mutable
Task-48/252/255/256/258/witness fieldsのexact
`32/55/39/72/62/21` matricesをauthenticateする。exact
`Task256:`/`Task258:`/`B3C:` failure prefixとgeneric-guard rejectionは
fallback acceptanceを防ぐ。

suiteはnon-vacuous zero-edge rejection、choice target/request order、
ownership/subtree near miss、全6 B3A/B3B/B3C family orders、active-route
isolation、immediate replay/rollback、final clone/debug stability、empty
semanticsもcheckする。initial test gaps 2件とB3A-hard-coded route findingは
remediateし、repeat test/implementation reviewsは**NO FINDINGS**。

final sizesはstatement `10305`、unchanged set leaf `4517`、facade `779`、
root `2595`、statement tests `23583`、unchanged set tests `2528`。runner
library `461`、focused `5/5`とpackage `461+3/14/137/2/21`はPASS。active
fixture/expectation/sidecar/trace/CLI/diagnostic/semantic harness surfaceは
変更しない。

## Task 258B3M2B2B3D frozen harness

dormant harness selectorはexisting context-1 Task-255 handoffをassemble
する前に、exact 109-byte、54-node/root-53 qua sourceとlocal theorem/
label provenanceだけをauthenticateしなければならない。future testsは
bytes/LF、各node/rootのfour surface axes、resolver、exact
`32/70/44/72/62/21` lower/upper matricesとreplay/owning prefixes、
`QuaBase`とordered requests、ownership/subtree isolation、全4 B3
family orders、clone/rollback/debug stability、empty semantic tablesを
exhaustする。active fixture、expectation、sidecar、trace、CLI、
diagnostic、semantic harness surfaceは変更しない。

## Task 258B3M2B2B3D implemented runner harness

dormant selectorとfrozen runner tests 5件はexact 109-byte source/final LF、
全54 nodes/root 53のkind/range/recovery/children axes、base resolver 8 +
label resolver 10 mutations、全currently mutable
Task-48/252/255/256/258/witness fieldsのexact
`32/70/44/72/62/21` matricesをauthenticateする。replayは
`Task48:`/`Task252:`/`Task255:`/`Task256:`/`Task258:`/`B3D:` owning
prefixとnon-generic-guard rejectionをrequireする。

suiteは`QuaBase`/`QuaTarget`/ordered request corruption、owner/unowned/
subtree near misses、B3A/B3B/B3C/B3D pairingsと24 orders、active-route
isolation、immediate replay/rollback、final clone/debug stability、empty
semantic tablesもcheckする。test-sufficiency reviewは**NO FINDINGS**。
focused runner `5/5`、package `466+3/14/137/2/21`、format/full Clippyは
PASS。active fixture/expectation/sidecar/trace/CLI/diagnostic/semantic
harness surfaceはunchanged。independent implementation reviewは
**NO FINDINGS**。3件のbounded wording/status修正後、source/docs
consistencyとboundary repeatも**NO FINDINGS**。package/fmt/full Clippy/
workspace tests/5 CLI/count/hash final rerunsはPASS。

independent final read-only quality reviewは**NO FINDINGS**、全9 hard
gates PASS、no cap、valid `100/100`
（`20/20/15/15/10/10/5/5`）。CLI `23/0` warnings/errorsとlarge
repeated-test diff review volumeはharness behaviorを変えないnonblocking
residual。staging/cached diff、commit、post-commit/fresh-nextだけがpending。

## Task 258B3M2B2B3E frozen harness

dormant harness selectorはexisting context-1 Task-255 handoffをassembleする
前に、exact final-LF 139-byte/hash、28-token、60-node/root-59
condition-free comprehension sourceとlocal theorem/label/proof-context
provenanceだけをauthenticateしなければならない。

future runner tests 5件は全source bytes/LF、各node/rootのkind/range/
recovery/children axes、resolver base/label mutation、Task-48/252/255/256/
258/witnessのexact `32/70/53/72/62/21` matricesとimmediate replayを
exhaustする。Task-255 assertionsはgenerator node `16`、type
expression/head `41/40`、`SetComprehension(43)`、condition 0件、
`ComprehensionMapper -> Primary(2)`、ordered
`GeneratorSethood`/`ResultType`をfreezeする。

suiteはowner partition Task-252 `{32,34,38,47,49}`、Task-255
`{16,40,41,43}`、Task-256 `{36,51}`、Task-258 `{54,56}`、B3E
`{45,46}`をcheckし、generator segment `42`をunownedに保つ。byte/subtree
near misses、generator/condition/request validation precedence、all
pairings、five-family `120` orders、owning-stage prefixes、non-generic
guard、active-route isolation、typed/final clone/debug/rollback、empty
semantic tablesをfail closedでcoverする。
post-auth shape/cardinality negativesとして、present condition、
zero/multiple generators、nested comprehension、generator-referencing
mapper、wrong/extra generator rows、wrong/extra type-site rows、nonzero
condition cardinality、complete `38..46` subtreeのincomplete/additional
ownershipをすべて明示的にrejectする。exact byte/subtree mutationだけで
これらを代用しない。これはB3E selector boundaryであり、existing
Task-255C1 exact independent condition-bearing source transportとそのcovered
creditはunchangedである。

active fixtures、expectations、sidecars、trace、CLI、diagnostic、
coverage/semantic harness surfaceは変更しない。generator binding/capture、
sethood/result typing、condition-bearing B3E statement-witness profileと
broader multiple/nested/generator-reference semantics、goal/proof/theorem
semanticsをfabricateしない。documentation-only test-sufficiency reviewは
**NO FINDINGS**、focused/package/workspace/CLI/count/hash verificationは
PASSした。future implementation test reviewはseparate taskに残す。

## Task 258B3M2B2B3E implemented runner harness inventory

dormant selectorと5 testsは139 bytes/final LF、60 nodes/root 59の4 axes、
resolver、全frozen fieldsをauthenticateする。post-auth negativesはsame
provenanceのsuccessful coherent Task-255 handoff（B3A zero、B3C wrong
type、Task-255C1 condition、synthetic valid multiple/nested、successful
empty generator-reference exclusion）で、authenticated source/module
identity、repeated B3E dependency/invalid failure、clean replayを検査する。
別の`32/70/53/72/62/21` mutation matricesがowning-stage prefixと
generic-guard-only failure rejectionを検査する。

120 orders、ownership/subtree、active isolation、clone/rollback/debug、
empty semanticsがPASSする。statement tests 26,141 lines、focused `5/5`、
library `471` PASS。corpus/trace/CLI/semanticはunchangedで、reviewは
**NO FINDINGS**。

harness responsibility overclaimと同期docs drift 2件の修正後、final
consistency repeatは**NO FINDINGS**である。complete verificationはPASSし、
independent qualityも**NO FINDINGS**、全9 gates PASS、valid `100/100`。
staging/post-commit gatesはimplementation commit
`e4479691db3b0a8785bb16e94d386bd71a394274`でcloseし、fresh inventoryは
Task 258B4Aをselectした。

## Checker Task 258B4A frozen harness route

harnessはexact private 80-byte/double-LF explicit-universal theoremだけを
recognizeする。new Task-258 composite statement constructorとpaired typed
installerをcallする前に、全26 Surface nodes、local theorem resolver
provenance、complete Task-252/256/257/B1 handoffをauthenticateする。existing
lower outputはsingle crate-private production-helper visibility seam経由で
consumeし、Task-257 rowsをcopyまたはrebuildしない。79-byte active
Task-257B1 sourceはlower-only routeに留めなければならない。

4 stage vectorsはstatement selection、typed paired installation、final
construction、stable reportingをcoverする。5 frozen testsはbytes/nodes、
lower/upper fields、stage-prefix failure/replay、coherent near miss、両family
orders、clone/debug、empty semantic outputをexhaustする。harnessはtheorem
truth、acceptance、proof、fact、formula-statement coverageをclaimしない。
documentation-only test-sufficiency reviewは**NO FINDINGS**である。future
implementation test reviewはseparate taskに残す。

## Checker Task 258B4A implemented harness route

dormant selectorはcomposite statement transactionをpublishする前に全frozen
source byte/Surface row、exact resolver owner、complete lower profiles、
rootless lower typed arena、lower owned sites/rangesをauthenticateする。lower
rowをcopyせず、crate-private seamを通じてproduction Task-257B1 handoffを
reuseする。

5 testsはexact output、lower mutations 142件、upper statement mutations
34件/owned-node substitutions、coherent rooted-arena near miss、
resolver/active 79-byte isolation、family-order atomicity、failure/replay、
final clone/debug/empty semanticsをcoverする。checker suiteがcoherent
relocated-term near missをseparately coverする。focused runner `5/5`と
separate test-sufficiency/implementation reviewsは**NO FINDINGS**である。
harnessはactive route、truth、fact、acceptance、proof、coverage claimを
追加しない。

## Checker Task 258B4B frozen harness route

dormant selectorはexact 167-byte/double-LF hash、124 Surface nodes/root
123の全kind/range/recovery/ordered-child fields、raw local theorem
resolver provenance、one theorem label projectionとcontribution label
effectを含むenriched `1/1/1/1/0` resolver environmentをauthenticateする。
Task-257B2 lower handoffsをone rootless arena内でrebuildし、両
`Composite(0)` linksを持つupper `1/1/1/0/1`だけをpublishする。private
route telemetryはexact zero-reference sentinel `0/0/[]`で、profile-aware
detail guardはmatched Task-257B2/B4Bだけでこれをacceptし、B4Aを
`1/1/[1,1]`のまま保つ。

checker planでnamedされたexact runner tests 5件はcomplete byte/LF、
`124 x 4` node、raw/enriched resolver、label-effect、`0/0/[]`
telemetry/detail-guard、lower-row、upper-row、42/1/81 ownership、
cardinality/fingerprint、coherent lower near-miss、active 166-byte/B4A/
atomic-family、order、rollback/replay、clone/debug、empty-semantic
matricesをcoverする。paired checker suiteがsyntax-free corruption/final
allowlist checksをownする。active fixture、sidecar、trace、diagnostic、
public runner schema、connective truth、repetition expansion、theorem
acceptance、proof resultは追加しない。

## Checker Task 258B4B implemented harness route

documentation prerequisite
`b8a7b8257a682f7c88de943ceaa35b67c0585bc4`後、dormant runner selectorは
exact 167 bytesと124 Surface nodes/root 123をauthenticateし、raw
label-free resolverをB4B専用guardで検査してからmatching theorem labelを
enrichする。resulting resolver cardinalitiesは`1/1/1/1/0`である。
generic enrichmentとTask-257B2 lower helperはunchangedである。

routeはlower Task-257B2 transactionをreuseしてrootless 124-node arenaを
constructし、ownership `42/1/81`、upper `1/1/1/0/1`と両
`Composite(0)`をpublishする。zero-reference telemetryは`0/0/[]`で、
profile-aware guardはB1/B4Aの`1/1/[1,1]`と混同しない。active 166-byte
sourceはlower-onlyである。

exact runner 5 testsとchecker 4 testsはPASSした。complete
surface/resolver/lower/upper/ownership/isolation/order/replay/final-empty
matricesに対するtest-sufficiency/implementation reviewsは**NO FINDINGS**
である。runner libraryは`481`、production 30 paths/56,007 linesである。
public route/schema、semantic result、corpus、sidecar、trace、coverage
creditは追加しない。final source/documentation、bilingual、boundary
consistency repeatは**NO FINDINGS**である。focused checker `4/4` /
runner `5/5`、full `cargo test --offline`、
`cargo fmt --all -- --check`、warnings deny付きfull offline Clippyは
PASSした。checker/runner countsは`418/481`、production/test-listと5 CLI
counts/hashesはrecorded valuesを再現し、plan/requirements `419/387`、
pass/fail `228/191`、parse/declaration/type/proof `101/5/198/1`、
warnings/errors `23/0`である。seven-file scope、spec coverage audit
no-op、unchanged stash gatesもPASSした。independent final qualityは
**NO FINDINGS**、全9 hard gates PASS、score capなし、valid `100/100`
（`20/20/15/15/10/10/5/5`）である。exact staging/cached-diff review、
implementation commit、post-commit invariants、fresh-B4C inventoryは
pendingである。

## Checker Task 258B4C frozen harness route

B4Bはsubsequently
`752c17ae7d552d5268d1028612b8174e480b6f3e`としてcommitされ、clean
ahead-1/behind-0 post-commit inventoryとunchanged stashがB4Cをselectする。
harnessがrecognizeするのはprivate 139-byte/two-LF source/hash
`36e5a68a92451590644951838a9af8926212bd78f88d1f90563f12b650b161c1`
だけである。active 138-byte/one-LF source/hash
`cbfd7077713e8e9630900e349d5f579251c19fba55434acb62170ea1dd940237`
はlower-onlyのままである。

upper route実装前にindependent lower-stage prerequisiteとして、existing
Task-257B3 selectorにexact 138/139-byte variantsだけをacceptさせ、zero/
three trailing LFsをrejectさせる。write scopeは
`type_elaboration/source_formula.rs`と
`runner/tests/type_elaboration/source_formula_composition.rs`だけで、
production `source_formula_composition.rs`はunchangedである。added-test
countはここでprojectせずfresh inventory後にmeasureする。

future upper selectorはSurface rows 66件/root `65`、theorem `62`
`19..137`、label token `6` `27..65`、outer formula `60` `67..136`、
raw resolver `1/0/1/1/0`、theorem path `[2,1]`、reserve contribution
`0` anchor `0..18`をauthenticateしてからだけ`1/1/1/1/0`へenrichする。
exact lower profiles binding `4/4/0`、primary `6/6/0`、atomic
`3/0/0/0/0/0/0/6/6`、composite `3/0/1/3/3/2/6`、
composition `3/6`をreuseする。

publicationはupper `1/1/1/0/1`、context `0` visible binding `[0]`、
input facts 0件、statement/candidateの両方が`Composite(0)`である。
rootless typed arena partitionは`24/1/41`。profile-aware detail guardは
telemetry `2/2/[2,2,4,4,4,4]`をexact B3/B4C pairだけでacceptする。

projected checker 4/runner 5 testsはsource/LF isolation、全frozen
Surface/resolver/lower/upper field、ownership、telemetry、coherent near
miss、B4A/B4B/active-route isolation、order、rollback/replay、clone/debug、
empty semanticsをcoverする。fixture、expectation、sidecar、trace
status/count、active route、public schema、formula truth、witness/
restriction semantics、theorem acceptance、fact、proof/Core/CFG/VC result、
coverage creditは変更しない。baselineはlibraries `418/481`、checker
production `23/140821`、runner production `30/56007`、および既記録の
production/test-list/five-CLI hashesのままである。

## Checker Task 258B4C 実装済み Harness Route

runner は exact 139-byte/two-LF source、全66 Surface row/root 65、raw
owner/contribution provenance、enriched `1/1/1/1/0` を認証する。別 commit
で admit 済みの Task-257B3 lower handoff を reuse し、upper-owned は theorem
node 62 のみ、両 link が `Composite(0)` の exact `1/1/1/0/1` を publish
する。

frozen runner 5 test は raw/enriched resolver mutation、全 Surface axis、
全 lower/upper fingerprint と row、unowned anchor/recovery corruption を含む
exact `24/1/41` ownership、zero/triple-LF と active-route isolation、
B4A/B4B/atomic/Task-248 order、transport-detail telemetry hybrid、replay、
clone/debug、empty semantic output を cover する。shared production guard
は exact matched profile ごとに B4A `1/1/[1,1]`、B4B `0/0/[]`、B4C
`2/2/[2,2,4,4,4,4]` だけを accept する。

public harness schema、active route、fixture、expectation、sidecar、
trace/coverage state、diagnostic、semantic result は変更しない。

## Checker Task 258B5A frozen private route

runner selectorはexact 185-byte/final-LF private sourceだけをadmitし、
near-miss bytesとunchanged authoritative fixtures 2件をrejectする。real
parser、resolver two-pass replay、BindingEnv、Task-252/256/258 producerを
runし、全rowをauthenticateしてからpaired base/reference handoffをinstallし、
syntax fallbackなしでfinal cloneをassembleする。

outputはexact one proof-step label、one simple-local citation、left/right
lookup ordinals `1/1`、all-one ten reference-use ordinals、empty semantic
tablesである。frozen runner five testsはsource identity、raw/enriched
resolver/lower mutation、owning-stage scope/range/ordinal/ownership
corruption、B1/B5A cross-pair/order、replay/debug、selector isolation、
empty semanticsをcoverする。public harness field/error/debug grammarは
変更しない。

## Checker Task 258B5A implemented private route

private selectorはexact 185-byte/final-LF sourceだけをadmitし、real resolver
two-pass replay前に全93 Surface rowをauthenticateする。unchanged
BindingEnv/Task-252/Task-256 producerをreuseし、exact base/reference
handoffをconstructし、matched B5A stateだけをinstallし、syntax fallback
なしでimmutable final cloneを再検証する。

outputはexact `20/73` ownership、scope `[0]`のprivate/local label 1件、
scope `[0,1]`のsimple-local citation 1件、resolver node 82からlabel key
0、lookup ordinal `1/1`、10 reference-use ordinalを全て1、empty semantic
tableを維持する。source、Surface、resolver、lower、row、scope、ownership、
cross-profile、replay、clone near missはowning boundaryでisolateする。
public harness field、active selector、diagnostic、fact、accepted statement、
proof、goal、IRはunchanged。

## Checker Task 258B5B frozen imported route

harnessはseparate commitされたcrate-private opt-in helperがpublic/exported
imported theorem label `Ref`をauthenticateするまでB5Bをinstallしない。
helperは`import_fixtures.rs`とstatement test leafだけをownし、normal
`8/0/1/3/1`から`8/1/1/3/1`を作り、default callerを維持し、2 frozen
`task258b5b_opt_in_*` testsでguardする。

lower commit後、selectorはSHA
`671e940c9dc749757dc8fddcc30a1a230aecb650058e64d6f1e73c1c66e93e9e`
の146-byte sourceだけをadmitし、57 Surface/resolver、Binding `2/1/0`、
Task-252 `4/4/0`、Task-256 `2/0/0/0/0/0/0/4/4`、Task-258
`1/2/2/2/2 + 0/1`、`8/49`をauthenticateする。citation id 0/dense
citation-row ordinal 0はnode/range `48 / 136..139`、scope `[0]`、
`LabelRefId(0)`、
contribution 2、anchor `7..27`、path `[1,0]`、exact imported originを
持ち、resolver reference candidateは独立にsource-statement ordinal 1、
telemetryは`1/1/[1,1,1,1]`。5 runner testsがcitation-row/resolver
source-statement ordinalの独立mutationとrunner test 2/final-cloneでの
coherent `Exported`から`ReExported`へのnear missを含むcorruption、
isolation、B5A atomicity、replay/debug、empty semanticsをcoverする。
upper runnerはexact resolved import id 0をreconstructする: owner node 29、
range `7..27`、spelling `import parser.type_fixtures;`、alias `None`、
resolved module `<package>::parser.type_fixtures`、current-source/
current-module origin anchor `7..27`、path `[0]`、import edgeなし、normal
recovery。nodes 28/29/30はunkeyed `NotApplicable`、node 48だけがkeyed。
imported projection originはcurrent-source/imported-module、anchor
`7..27`、path `[1,0]`、import edgeなし、normal、reference originは
current-source/current-module、anchor `136..139`、path `[48]`、import
edgeなし、normal。runner test 2/final-clone coverageは全fieldを独立に
mutateする。
tests 1/5はchecker planでfreezeしたordered debug schema全体、literal
`label_node=absent`/`source=imported`、complete imported projection fields、
resolver-ast/reference/result line、`target=imported` citation line、
`label#0` lineなしをassertし、B1/B5A bytesをunchangedに維持する。

public runner、active fixture/selector、expectation、trace、diagnostic、
semantic/proof/IR outputはno-op。B5Cとqualified/grouped/bulkはdeferred。

## Checker Task 258B5B implemented imported route

prerequisite commits `141dc44a`/`46dd9db5`がupper routeに先行する。
productionはfrozen 146-byte/final-LF textとのexact source equalityの場合だけ
special imported-label augmentationを実行し、全near missはdefault
augmentationを維持してB5Bをselectできない。

implemented routeは57 Surface/resolver identities/root 56を維持し、raw/
enriched resolver `1/0/1/1/0`/`8/1/1/3/1`、Binding `2/1/0`、
Task-252 `4/4/0`、Task-256 two formulas/four edges/four requests、
Task-258 `1/2/2/2/2`、reference `0/1`、ownership `8/49`をvalidateする。
sole citationは`Imported`/`SimpleImported`で、contribution 2とexact
import/projection/reference provenanceを通してpublic/exported theorem
`Ref`をresolveし、local label rowをemitしない。typed/final cloneの
semantic outputはempty。

upper five/lower two runner testsは合わせて`7/7`、checker B5Bは`4/4`、
full librariesはrunner `500/500`/checker `430/430`をPASSする。B1/B5A
selector、target wrapping、public debug bytes、cross-profile atomicityは
unchanged。public harness/CLI field、corpus case、expectation、sidecar、
trace row、diagnostic、semantic acceptance、proof、IR outputはpromoteしない。

## Checker Task 258B5C frozen declaration-symbol route

B5Cはactive fail routeであり、別のprivate checker statement profileではない。
separate commitされるresolver R-032A structural lowering、次にR-032B
proof-label collectionへblockされる。runnerは
`SurfaceResolvedArena::lower(&ast, &module)`/
`validate_against(&ast, &module)`後、linked exact
`impl<'a> ProofLabelSourceCollector<'a>` declarationの
`new(ast: &'a SurfaceAst, module: &ModuleId, namespace, contribution,
resolved: &'a SurfaceResolvedArena) -> Result<Self, ...>`でconstructし、
`collect(&self)`をcallする。same `'a`でstoreするのは
ast/resolvedだけ、moduleはvalidation-only/not stored、namespace/
contributionはowned。returned
`projections()`/`references()`だけを`LabelResolver`へ渡す。両lower APIは
canonical exact resolver enumでfail closedし、R-032A state/key mismatchを含み
全node/child/overflow payloadは`SurfaceNodeId`。runnerは`LabelScopePath`、source ordinal、
structural origin、`ResolvedNodeId`をcompute/fabricateしない。

exact 173-byte inner-to-outer/197-byte sibling source、hash、normal Surface
nodes、scope `[0,0]`/visible-after ordinal 3のprojection `A`、scope
`[0]`/ordinal 5と`[0,1]`/ordinal 6のcandidateはcrate planでfreeze済み。
各observationは`1/1/[0]`、zero resolver diagnostics、
`has_unresolved = true`、one `UnresolvedLabelRef` `A`/expectation
`ProofOrTheorem`をrequireする。exact projection/reference structural pathは
`[57,42,8]`/`[57,55,52]`と`[67,47,8]`/`[67,63,60]`。
structural-map/overflow/resolved/ambiguous/additional/recovered/
provenance-mismatched resultはrouteをfailさせる。

R-032B collectionはparserよりnarrowで、normal top-level theorem/direct-proof
owner、exact labelled compact statement、`by`配下のsimple unqualified
`JustificationClause -> ReferenceList -> Reference` chain、supported nested
proof、normal `CompactStatement`/`ConclusionStatement` ordinalだけ。
module-global one-based counterはtheorem rootでresetせず、theorem/
transparent/excluded subtreeはconsumeしない。referenceはowning-statement
ordinal、visibilityはlabelled-subtree最大。B5C offsetは`2/3/3/4/5`、
sibling reference 6。
same-block completion後はpositive、own-proof self-reference、
inner-to-enclosing、sibling、earlier theorem `[0]` labelをlater theorem
`[1]`からciteするrouteはunresolved。distinct theorem rootのsame-spelling
declarationはconflictしない。これらboundaryとorigin
stability/uniquenessはlower testでありrunner-derived logicではない。
originはresolver `labels.md` canonical collision-free `proof-step-v1`
length-framed grammar、exact token bytes、zero-based occurrence、
owner-relative proof pathだけを使う。

branch selectionは`FrontendRun.source_text` frozen constant byte equalityと
exact normal AST profileだけで、metadata/expectationを使わない。shared
`resolver_symbol_collection`後はenv/module一致、module pathだけからnamespace、
matching local-source contribution/source id exact one、B5C contribution 0を
要求する。input/provenance corruptionは
`declaration_symbol.label.proof_scope_input`だけ、fully authenticated
unresolved confinementだけが
`declaration_symbol.label.proof_scope_confinement`をemitする。public codeは
emptyで、expectation copy/mutationがselection不能なtestを持つ。

future sidecarはempty public diagnostic code、tag
`active_declaration_symbol`、private detail key
`declaration_symbol.label.proof_scope_confinement`を持つ
declaration-symbol/resolve failureで、rejection reason 2件はdistinct。
write scopeは`declaration_symbol.rs`、`runner/tests.rs`、new
`runner/tests/declaration_symbol.rs`、new fixture/sidecar pair 2件、trace
row 2件、synchronized derived documentsだけ。public harness/CLI schema、
checker/type/proof/Core/CFG/VC outputはno-op。このexact 48-file documentation
prerequisiteはactive case/coverage creditを作らない。

collectorはresolver closed Surface edge tableにもfreezeする。exact upper
chain
`Root -> CompilationUnit -> ItemList -> direct TheoremItem -> ProofBlock`、
その後はdirect normal
`CompactStatement`/`ConclusionStatement`だけ、compact proposition-label
inspectionだけ、両statementからはdirect `ProofBlock`/
`JustificationClause`だけである。candidateはexact
`JustificationClause -> ReferenceList -> simple Reference -> sole identifier
token` chainを要求する。formula、token、wrapper、unsupported/recovered/
malformed node、qualified/grouped/bulk citation、templateはordinal/descent
なしでskipする。positive testは全upper edgeを含む各allowed edgeをcover
する。upper negativeはRoot/CompilationUnit childのmissing/additional/wrong、
direct Root/CompilationUnit theorem relocation、`VisibleItem` wrappingを
coverし、other forbidden relocationはdefault denialを証明する。mixed-list
testはexact simple `Reference` siblingだけをsource orderで保持し、
unsupported siblingはrow/descentを追加しない。

runner authenticationはfield-by-fieldである。
`env.module_id() == resolver.module`を要求し、
`NamespacePath::new(module.path().as_str())`をderiveして全projection
namespaceをvalidateする。contributionをexact one acquireし、id 0、
`LocalSource`、record module、LocalSource source idをpublic field
`ast.source_id`に対してvalidateする。全projection module/namespace/
contributionはauthenticated valueと一致する。independent mutationは
environment module、projection module/namespace/contribution id、
contribution zero/multiple、contribution id、`ImportedSource`/`Summary`/
`Builtin`の各kind、contribution record module、LocalSource source idをcover
する。各mutationは`declaration_symbol.label.proof_scope_input`だけをemitし、
confinement/public codeはemitしない。copied/mutated expectation下でもsource
bytes+normal ASTだけがselectorである。

## Checker Task 258B5C implemented declaration-symbol route

current implementationはactive fail caseをexact 2件追加する。runner pathは
unchanged R-032A `SurfaceResolvedArena`、R-032B
`ProofLabelSourceCollector`、`LabelResolver`をconsumeし、scope、ordinal、
origin、contribution id、resolved nodeをconstructしない。exact
source-plus-normal-AST authenticationの後でshared declaration-symbol
environment、projection/reference provenance、unresolved resultをcheckする。

両caseはempty payload table/public codeなしで
`declaration_symbol.label.proof_scope_confinement`だけをproduceする。
corruption matrixはmismatched environment/contribution/projection/reference/
resolver resultがすべて
`declaration_symbol.label.proof_scope_input`だけをemitすることをproveする。
expectation fieldはrouteをselectできず、replay/orderはdeterministicで、
earlier active declaration-symbol 5 caseのresultも維持される。

## Checker Task 259 Frozen Runner Route

future type-elaboration leafはexact 165-byte sourceとnormal
71-row/root-70 ASTだけをselectする。current source/module、definition-block
shell 0配下predicate shell ordinal 1、predicate Symbol/Definition id/kind、
contribution、normal origin `61..122`、structural path `[4,0,8,0]`、notation
spelling、same-block property shell `125..159`をauthenticateする。resolver
generic property Attribute/Attribute projectionはraw-profile evidenceとして
observeするだけで、predicate-property semanticsとしてsupplyしない。

separate Task-248 extension後、leafはdefinition parameter `x`/`y`にexisting
handoffをreuseし、Task 249 `2/2/0`、Task 252 `4/4/0`、Task 256
`2/0/0/0/0/0/0/4/4`をbuildする。Task 259へone predicate、two parameters、
one guard、one property、one correctness rowをsupplyする。one pending
obligationはempty diagnosticsのpass resultである。computation
justificationをrun/accept/dischargeしない。

focused testはexact payload/range/order、全independent resolver/lower-handoff
field mutation、missing/duplicate/reordered/cross-owner row、guard/definiens
swap、property kind/range/owner change、obligation id/kind/owner/range/
assumptions/goal/provenance/status mutation、transactional failure、
typed/final clone/debug preservation、same/reverse corpus order、
exact-source/AST near miss、expectation non-selection、absent
Tasks 253--255、257、258、260+からのisolationをcoverする。
`Blocked`/`Invalidated`を`Pending`の代わりにしてはならない。existing
mixed predicate-plus-functor routeはold extraction gapを維持し、このleafを
selectしない。

later implementationだけがone pass sidecar、one trace row、mechanical
active-type metadata incrementを追加する。このfrozen runner contractはrunner
source、fixture、sidecar、trace、diagnostic、countを変更しない。

## Checker Task 248 Two-Parameter Dormant Extractor

prerequisiteはactive harness routeを追加しない。later private helperはtests、および
その後exact Task-259 leafがsource/AST/definition identityをselectした後だけcall
される。real DefinitionBlock shell 0、direct leading parameter nodes 41/45、
exact `x`/`y`/bare `set` token/range shape、scope/ordinal、one shared typed
arena内four siteをauthenticateし、existing Task-248 projectionだけをbuildする。

default denialはthird/non-leading parameter、reserve/extra-item contamination、
recovery、wrong shell/module/range/type/token/local identity、stale/duplicate
site、excluded descendantをbindingとして扱う試みをrejectする。existing Profile-A
selector/output/recovery/debugとactive fixtureはbyte-compatible。expectation fieldは
dormant helperをselectできずdiagnostic detail keyも追加しない。

## Checker Task 259 active predicate-definition route

private routeはbyte-exact sourceとcomplete normal 71-row/root-70 surface profile
だけでselectする。definition block、ordered `x`/`y` parameter、guard、
predicate pattern/definiens、symmetry property、raw predicate resolver entry、
same-block sibling、pattern/label/justification descendant exclusionをauthenticateする。
sidecar outcome、stage、tag、diagnostic、expectation dataはrouteをselectできない。

selection後にone shared surface-indexed `TypedArena`を作成し、existing ownerを
Task 248、Task 249 `2/2/0`、Task 252 `4/4/0`、Task 256
`2/0/0/0/0/0/0/4/4`、Task 259 `1/2/1/1/1`のexact orderでcallする。
Task 259はinput obligation baselineをpreserveし、empty assumptionを持つ
`Pending` `PredicatePropertyCorrectness` row 1件をappendする。typed/final
installationはall-or-nothingのまま。routeはproperty proof、fact、axiom、
VC、acceptance、public diagnostic、Task-260 mixed-family payloadをpublishしない。

runner test 4件は
`task259_real_source_surface_resolver_and_lower_bundle_is_exact`、
`task259_source_ast_resolver_and_lower_mutations_fail_at_the_owner`、
`task259_expectation_selection_and_mixed_definition_route_stay_isolated`、
`task259_route_publishes_no_property_proof_fact_or_acceptance`である。`4/4`を
PASSし、full runner library countは`512`である。adjacent active-count変更は
source-statement selection test 2件のindependently reviewed `198 -> 199`だけで、
empty-selection assertionは不変である。

## Checker Task 260 frozen functor-definition route

future private routeは
`mizar-checker/en/source_functor_definition.md`でfreezeしたexact 262-byte
sourceとcomplete normal 108-row/root-107 surface profileだけでselectする。
definition block、ordered `x`/`y` parameter、one `assume` guard、`equals` /
`means` functor definition、explicit return type、raw resolver provenance、
explicit correctness clause 2件、指定されたexcluded descendantをauthenticate
する。sidecar outcome、stage、tag、diagnostic、expectation dataはrouteをselect
できない。

selection後にone shared surface-indexed `TypedArena`を作成し、existing lower
ownerだけをTask 248 Profile B `1/2/2/2/2/2/0`、Task 249 + 249R
`2/4/0/2`、Task 252
`5/5/0`、Task 256 `2/0/0/0/0/0/0/4/4`の順にcallする。frozen sourceでは
Task 253-255はabsentのままである。Task 260はtable `2/2/1/2/2`をpublishし、
input obligation baselineをpreserveして、`means` definitionだけにpending
`FunctorExistence` / `FunctorUniqueness` rowをappendする。typed/final
installationはall-or-nothingである。

future runner test 4件は
`task260_real_source_surface_resolver_and_lower_bundle_is_exact`、
`task260_source_ast_resolver_and_lower_mutations_fail_at_the_owner`、
`task260_expectation_selection_and_predicate_route_stay_isolated`、
`task260_route_publishes_no_proof_fact_acceptance_or_vc`であり、runner library
countを`512 -> 516`とprojectする。exact implementationはmechanical active-type
consumer 6箇所も`199 -> 200`へ更新するが、goal composition、proof、discharge、
acceptance、fact/axiom、VC/IR、Task-259 predicate payloadをpublishしない。

Task 249Rはchecker-onlyでrunner route/testを追加しない。runnerはfresh
inventoryでcombined source-type handoffがbinding application 2、expression 4、
argument 0、definition-return row 2であることを確認した後だけTask 260を開始し、
complete handoffをfingerprintしてreturn-type bindingを捏造しない。

test 1は全source byte/final LF/hash、108 Surface row/ordered child、root/sibling/
subtree partition、resolver profile、lower bundle、final outputをassertします。
test 2は各source/AST/resolver/lower familyとexcluded descendantをmutateしownerで
停止させます。test 3はexpectation non-selection、Task-259/mixed isolation、
metadata `137/137`と合わせたsole reciprocal backlinkをproveします。test 4は
computation subtree、proof/discharge/acceptance/fact/VC output absenceとsix count
consumerをauditします。

## Task 249M no-consumer harness boundary

Task 249Mはharness route/testを追加しない。checker-only test 4件がstandalone RHS
lower handoffをfreezeし、corpus、metadata test `137`、CLI 5本、runner list
`520`、fixture/sidecar/trace hash、mixed mode/structure gapはbyte-identical。
later Task-262 runnerだけがreal sourceから`2/3/0/0/1`をbuild/fingerprintできる。

## Checker Task 249M active harness no-op

lower checker producer/test 4件は実装済み。本harnessはroute/testを追加せず、
runner `520`、metadata test `137`、corpus/sidecar/expectation/trace hash、CLI
5本はTask-261 frozen baseline不変。real consumerはTask 262だけが追加できる。

## Checker Task 262 active harness consumer

dedicated exact-source routeはactiveで、checker-owned mode-definition payloadと
linked Pending obligationだけを返す。expectation selection、全source/Surface/
resolver/lower/payload corruption、Tasks-259--261 sibling route、justification/
semantic descendantをrejectする。runner testはexact 4件追加されlibraryは
`520 -> 524`、metadata testは`137`のままである。metadata-mode CLI 5本は
`425/393`、`232/193`、`101/7/202/1`、type `257/245`、warnings/errors
`23/0`でpassする。
## Checker Task 249S no-consumer harness boundary

Task 249Sはharness route/production consumerを持たない。checker-local test 4件が
standalone structure-member type handoffをfreezeする。exact Task-263 runner、
pass source/sidecar/covered trace row、diagnostic behavior、全structure/inheritance
semanticsはTask 263へdeferする。

## Checker Task 249S active harness no-op

Task-249S test 4件は`mizar-checker::source_type`内だけで実行する。discovery、
selector、stage、loaded-source extractor、runner assertion、snapshot pathは
追加しない。harness `524` testsと5 CLI outputはbyte-stableで、最初のreal
consumerはTask 263が所有する。
