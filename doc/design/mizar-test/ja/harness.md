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
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub struct TestCase {
    pub id: TestCaseId,
    pub source_path: PathBuf,
    pub expectation_path: PathBuf,
    pub expectation: Expectation,
}

pub enum TestProfile {
    Fast,
    Full,
    Stress,
    FuzzRegression,
    SnapshotUpdate,
}

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

## Runner Modes

| Mode | Behavior |
|---|---|
| metadata plan | payload を実行せずに sidecar を discover し、layout、expectation schema、traceability を validate |
| parse-only | active な `.miz` parse-only case を `mizar-frontend` と `MizarParserSeam` で run |
| declaration-symbol | active な `.miz` declaration-symbol case を frontend parsing と resolver declaration/symbol collection で run |
| type-elaboration | active な `.miz` type-elaboration case を frontend parsing と resolver declaration/symbol collection まで run し、checker payload extraction bridge の不足を stable external dependency gap として surface する |
| pass/fail | `.miz` cases を run し expected outcome と match |
| snapshot | canonical snapshot hashes を compare |
| determinism | repeated runs を比較し artifacts、diagnostics、hashes を check |
| parallel-equivalence | sequential and parallel outputs を compare |
| fuzz-regression | minimized fuzz cases を ordinary committed tests として run |
| update | 明示要求された場合のみ snapshots を rewrite |

## Algorithm / Logic

1. `layout` を通して tests を discover する。
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
diagnostic と resolver symbol diagnostic がどちらも無いことを要求する。fail case
は、`diagnostic_payloads` が存在する場合はそれを、無い場合は
`stable_detail_key` を使って、resolver の crate-local internal detail key と比較する。
diagnostic-code ownership gap が open の間、この runner は public resolver
diagnostic code を要求せず、創作もしない。non-empty `diagnostic_codes` を持つ
active declaration-symbol expectation は harness error である。

`active_declaration_symbol` tag を持つ expectation が runnable case predicate の
いずれかを満たさない場合、runner は silent skip ではなく harness error として扱う。

現在の type-elaboration runner は、各 active `.miz` corpus file を同じ一時的な
package 形状へ copy し、実際の frontend を実行したうえで、得られた
`SurfaceAst` を resolver の declaration-shell collector、parser-backed signature
projection extractor、symbol collector に渡す。これにより checker payload extraction
へ進む前に lower-stage prerequisite を正直に確認する。task 12 は不足している
source-to-checker bridge を捏造しない。repository には parsed/resolved `.miz` の
declaration、type expression、term、formula、coercion site、type fact を
`mizar-checker` task 7-11 が公開する checker-owned payload に変換する AST-wide
extraction API がまだない。parsing と symbol collection が成功した場合、runner は
その bridge が存在するまで stable detail key
`type_elaboration.external_dependency.ast_payload_extraction` を report する。
active fail case はこの key を `diagnostic_payloads` または `stable_detail_key` で
assert してよい。real checker semantics を必要とする active pass case は stub で
pass させず deferred のままにする。

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

## Determinism Requirements

harness は identical inputs が次を生成することを check する。

- identical artifact hashes
- identical snapshot hashes
- identical diagnostic order
- identical failure records
- identical proof status
- identical dependency slices

parallel execution は runtime を変えてよいが、observable results を変えてはならない。

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
- repeated run が異なる diagnostic order を生成する
- parallel run が sequential run と同じ artifacts を生成する

## Constraints and Assumptions

- test execution order は semantic ordering ではない。
- harness は cache hits を検証対象の compiler behavior として扱い、proof authority としては扱わない。
- snapshot update mode は opt-in であり command output に見える形でなければならない。
