# Module: harness

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

この module が所有する exhaustive public enum exception はない。

## Runner Modes

| Mode | Behavior |
|---|---|
| metadata plan | payload を実行せずに sidecar を discover し、layout、expectation schema、traceability を validate |
| parse-only | active な `.miz` parse-only case を `mizar-frontend` と `MizarParserSeam` で run |
| declaration-symbol | active な `.miz` declaration-symbol case を frontend parsing と resolver declaration/symbol collection で run |
| type-elaboration | active な `.miz` type-elaboration case を frontend parsing と resolver declaration/symbol collection で run し、対応済み reserve-only declaration payload を抽出し、checker-owned `BindingEnv` / `DeclarationInput` / `DeclarationChecker` handoff production を syntax-free な `mizar-checker` seam に委譲し、successful bare-builtin case、task-55 bare local-mode-expansion case、task-56 one-edge local-mode chain case、task-74 structural bare local-mode chain case は `TypedAst` と `ResolvedTypedAst` まで継続し、`mizar-core` の `ResolvedTypedAstSummary::from_ast` で summary-readiness を確認し、同じ reserve binding から binder-only `CoreContext` input を準備し、same-module attributed reserve declaration、local structure reserve head、attributed local structure reserve head、task-57 の local structure RHS を持つ real local-mode expansion、task-58 の attributed builtin RHS を持つ real local-mode expansion、task-59 の real direct bare-builtin expansion を持つ attributed local-mode reserve head、task-60 の real direct local-structure RHS expansion を持つ attributed local-mode reserve head、task-61 の real direct attributed-builtin RHS expansion を持つ attributed local-mode reserve head、task-62 の local structure RHS で終端する one-edge bare local-mode chain、task-63 の attributed builtin RHS で終端する one-edge bare local-mode chain、task-64 の one-edge bare-builtin chain を持つ attributed local-mode reserve head、task-65 の one-edge structure-RHS chain を持つ attributed local-mode reserve head、task-66 の one-edge attributed-builtin-RHS chain を持つ attributed local-mode reserve head は checker evidence-query gap、narrow な task-55/task-56/task-57/task-58/task-59/task-60/task-61/task-62/task-63/task-64/task-65/task-66/task-74 expansion slice を持たない same-module local mode reserve head（mixed attributed/bare local-mode source、attributed chain dependency、task-74 structural guard violation chain を含む）は checker mode-expansion payload gap として surface し、task-67 structure-qualified attribute reference、task-68 argument-bearing local-mode reserve head、task-69 argument-bearing local-structure reserve head、task-70 bracket-form local-mode reserve head、task-71 bracket-form local-structure reserve head は source-to-checker extraction-gap boundary case として surface し、task-75 forward local-mode reserve head、task-76 forward local-structure reserve head、task-77 forward local-attribute reserve type expression は checker handoff 前の lower-stage active-range boundary case として surface し、未対応 checker payload family は stable external dependency gap として surface する |
| pass/fail | `.miz` cases を run し expected outcome と match |
| snapshot | canonical snapshot hashes を compare |
| determinism | repeated runs を比較し artifacts、diagnostics、hashes を check |
| parallel-equivalence | sequential and parallel outputs を compare |
| fuzz-regression | minimized fuzz cases を ordinary committed tests として run |
| update | 明示要求された場合のみ snapshots を rewrite |

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
| `mizar-vc` task 15 | `proof_verification` | paced/open。VC/proof-verification obligations は deferred | source-to-core/source-to-VC extraction と downstream verification seams が存在した後に runner support を追加する。 |
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
