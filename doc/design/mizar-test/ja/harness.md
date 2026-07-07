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
| type-elaboration | active な `.miz` type-elaboration case を frontend parsing と resolver declaration/symbol collection で run し、対応済み reserve-only declaration payload を抽出し、checker-owned `BindingEnv` / `DeclarationInput` / `DeclarationChecker` handoff production を syntax-free な `mizar-checker` seam に委譲し、successful bare-builtin case は `TypedAst` と `ResolvedTypedAst` まで継続し、`mizar-core` の `ResolvedTypedAstSummary::from_ast` で summary-readiness を確認し、同じ reserve binding から binder-only `CoreContext` input を準備し、same-module attributed reserve declaration と local structure reserve head は checker evidence-query gap、same-module local mode reserve head は checker mode-expansion payload gap として surface し、未対応 checker payload family は stable external dependency gap として surface する |
| pass/fail | `.miz` cases を run し expected outcome と match |
| snapshot | canonical snapshot hashes を compare |
| determinism | repeated runs を比較し artifacts、diagnostics、hashes を check |
| parallel-equivalence | sequential and parallel outputs を compare |
| fuzz-regression | minimized fuzz cases を ordinary committed tests として run |
| update | 明示要求された場合のみ snapshots を rewrite |

## Consumer Runner Pacing

task 10 は consumer crate と runner support を 1 increment ずつ同期する。
prepared increments は実装・検証済みにし、未準備 consumer は placeholder runner
mode、fake active fixture、fabricated coverage を作らず `paced/open` のままにする。

| Consumer task | Stage / runner | mizar-test status | Next condition |
|---|---|---|---|
| `mizar-parser` task 3 | `parse_only` / `parse-only` | prepared/implemented。active `.miz` pass/fail sidecars は `active_parse_only` を使い、tag のない parse-only metadata は planned のまま | general snapshot runner が着地するまで transitional `SurfaceAst` snapshot shortcut を保つ。 |
| `mizar-resolve` task 23 | `declaration_symbol` / `declaration-symbol` | prepared/implemented。active sidecars は `active_declaration_symbol` を使い、public resolver diagnostic-code matching は gate されたまま | resolver diagnostic range が仕様化された後に public diagnostic-code assertions を開く。 |
| `mizar-checker` task 12 plus task 16-20、task 48 source bridge continuation、task 50 attributed reserve evidence-gap bridge、task 51 local mode expansion-gap bridge、task 52 local structure evidence-gap bridge、reserve summary-readiness、binder-only core context follow-up | `type_elaboration` / `type-elaboration` | prepared/implemented。active sidecars は `active_type_elaboration` を使い、lower stages を先に実行し、reserve-only の builtin `set` / `object` declaration を `.miz` AST から syntax-free checker payload に抽出し、`SymbolEnv` にすでに存在する same-module attribute symbol は builtin reserve type payload に attach してよく、same-module local mode / structure symbol は attribute と argument のない reserve head として使ってよい。`mizar-checker` は checker-owned `BindingEnv`、binding ごとの `DeclarationInput`、binding 固有の `TypeExpressionInput` site、`DeclarationChecker` output を生成し、successful bare-builtin case は `TypedAst`、checker-owned `ResolvedTypedAst`、`mizar-core` `ResolvedTypedAstSummary::from_ast` read、binder-only `CoreContext` preparation へ継続し、attributed reserve case と local-structure case は checker `MissingEvidenceQuery` diagnostic、local-mode case は missing mode-expansion diagnostic で停止する。未対応 checker payload family は `type_elaboration.external_dependency.ast_payload_extraction` に残す | より広い type pass/fail semantic assertions は AST-wide source-to-checker payload extraction と real existential / evidence-query / mode-expansion / base-shape input を待つ。 |
| `mizar-checker` task 29 | `formula_statement` / `advanced_semantics` | paced/open。trace rows は deferred であり、active fixture は捏造しない | statement/formula と advanced-semantics source payload seams が存在した後に runner support を追加する。 |
| `mizar-vc` task 15 | `proof_verification` | paced/open。VC/proof-verification obligations は deferred | source-to-core/source-to-VC extraction と downstream verification seams が存在した後に runner support を追加する。 |
| `mizar-atp` task 20 | `advanced_semantics` metadata handoff | `mizar-test` では paced/open。metadata-only property fixtures は `mizar-atp` Rust tests が消費してよい | source-derived ATP extraction と proof-policy/kernel handoff seams が存在した後に active `.miz` ATP runner support を追加する。 |
| `mizar-kernel` task 17 | proof/certificate/kernel evidence | paced/open。fail/soundness metadata は active proof/certificate/kernel execution なしで検証する | source-to-evidence または certificate execution seams が存在した後に runner support を追加する。 |

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
symbol head を含まない bare builtin `set` / `object` shape に限る。task 50 は active
fail slice を 1 つ追加する: resolver declaration/symbol collection がすでに `SymbolEnv`
に入れた same-module attribute symbol は builtin reserve type payload に attach してよく、
checker declaration checking はより広い AST payload extraction gap ではなく
`checker.declaration.deferred.evidence_query` を出す。task 51 は 2 つ目の active fail
slice を追加する: attribute や type argument を持たない unique な same-module local mode
symbol は reserve type head として使ってよく、real mode-expansion payload extraction が
まだ無いため checker type normalization は
`checker.type.external.mode_expansion_payload` を出す。imported attribute、imported
mode、unresolved / ambiguous symbol、attribute argument、qualified attribute
disambiguation、attributed mode / structure head、mode / structure argument、non-reserve
declaration はこの source bridge の外に残る。task 52 は 3 つ目の active fail slice を追加する:
attribute や type argument を持たない unique な same-module local structure symbol は reserve
type head として使ってよく、real base-shape / constructor-witness evidence extraction が
まだ無いため checker declaration checking は
`checker.declaration.deferred.evidence_query` を出す。

抽出された payload について、runner は source/module identity、reserve source range、
binding spelling/range、対応済み type-expression spelling/range/head、対応済み
same-module attribute の symbol/range/polarity を `mizar-checker` の source reserve
declaration seam に渡す。その checker-owned seam は reserve binding を含む module
`BindingEnv`、binding ごとの `DeclarationInput`、binding 固有の
`TypeExpressionInput` site を構築し、`reserve x, y for set` は source range を共有しつつ
binding ごとに distinct typed site を持ち、collected `SymbolEnv` に対して
`DeclarationChecker` を実行する。runner は active fail slice の stable diagnostic key を
集めるために、同じ checker-owned assembly helper を使ってよい。checker diagnostic が
出た場合、active fail case はその key を比較し、runner は downstream readiness assertion
として credit しない。diagnostic-free な bare-builtin output について、返された checker
handoff は declaration と type-entry link を持つ checker-owned `TypedAst`、および
empty-but-real な cluster/overload predecessor output と source-preserved node hint /
declaration expression metadata により投影された checker-owned `ResolvedTypedAst` として
credit される。
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
non-builtin declaration、imported attribute、imported mode、attribute argument、
attributed mode / structure head、mode / structure argument、structure payload、term、formula、coercion site、
overload evidence、recorded fact、CoreIr、ControlFlowIr、VC payload、proof evidence は
対応済み extraction slice の外に残る。
active case がこれら未対応 payload family のいずれかを必要とする場合、runner は stable detail key
`type_elaboration.external_dependency.ast_payload_extraction` を report する。active fail
case はこの key を `diagnostic_payloads` または `stable_detail_key` で assert してよい。
対応済み slice の外にある active pass case は stub で pass させず deferred のままにする。
この runner は `CoreIr`、`ControlFlowIr`、VC seed、proof row、public checker diagnostic code を
publish しない。

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
