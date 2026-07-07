# Module: expectation_schema

> Canonical language: English. English canonical version: [../en/expectation_schema.md](../en/expectation_schema.md).

## Purpose

この module は `mizar-test` が使う `.expect.toml` sidecar schema を定義する。

Expectation files は committed tests の authoritative contract である。Compiler execution より前に parse され、`.miz` frontend に依存してはならない。

## Design Decision

Every executable corpus item は exactly one expectation sidecar を持つ。

Sidecar は次を所有する。

- test identity
- staged model placement
- spec traceability back-references
- expected outcome
- expected failure identity
- deterministic diagnostics and snapshot requirements

Source file は input program or fixture payload だけを所有する。

## File Pairing

Expectation sidecars は input file と同じ stem を使う。

```text
tests/miz/pass/parser/pass_parser_block_001.miz
tests/miz/pass/parser/pass_parser_block_001.expect.toml

tests/lexical/pass/pass_lexical_identifier_001.src
tests/lexical/pass/pass_lexical_identifier_001.expect.toml

tests/certificates/fail/sat/fail_certificate_sat_satisfiable_refutation_001.cert.json
tests/certificates/fail/sat/fail_certificate_sat_satisfiable_refutation_001.expect.toml
```

Harness は fail、soundness、certificate、snapshot、generated、fuzz-regression、property-regression tests の missing sidecars を reject する。Pass tests は explicit harness mode が legacy discovery を許す場合だけ sidecar を省略してよいが、committed evo2 corpus は all executable tests に sidecars を含めるべきである。

## Common Fields

All expectation files include:

```toml
schema_version = 1
id = "pass_lexical_identifier_basic_001"
kind = "pass"
stage = "lexical"
domain = "lexical"
source = "pass_lexical_identifier_basic_001.src"
expected_outcome = "pass"
spec_refs = [
  "spec.en.02.lexical.identifiers.basic",
]
```

Fields:

| Field | Type | Required | Meaning |
|---|---|---:|---|
| `schema_version` | integer | yes | Sidecar schema version. |
| `id` | string | yes | Stable test id. File stem と一致しなければならない。 |
| `kind` | string | yes | `pass`, `fail`, `snapshot`, `generated`, `fuzz_seed`, or `property_seed`. |
| `stage` | string | yes | Staged model stage. |
| `domain` | string | yes | Human-readable test domain. |
| `source` | string | yes | Sidecar directory からの relative input file path. |
| `expected_outcome` | string | yes | `pass`, `fail`, `snapshot`, or `metadata_only`. |
| `spec_refs` | array of strings | yes | `tests/coverage/spec_trace.toml` の requirement ids. |
| `profiles` | non-empty array of strings | no | この test を含める harness profiles。値は `fast`、`full`、`stress`、`fuzz_regression`/`fuzz-regression`、`snapshot_update`/`snapshot-update`。Default は `["fast"]`。 |
| `tags` | array of strings | no | Non-authoritative grouping tags. |
| `notes` | string | no | Short review note. Matching には使わない。 |
| `ast_profile` | string | no | Parser-facing snapshot tests が要求する AST rendering profile。 |
| `snapshot_profiles` | non-empty array of strings | no | Sidecar metadata として保持する snapshot profile ids。 |
| `architecture22_scenarios` | non-empty sorted array of strings | no | この metadata sidecar が覆う architecture-22 regression scenario ids。 |
| `architecture22_equivalence_class` | string | no | Optional registry equivalence class。listed scenario がすべて同じ class を持つ場合だけ有効。 |
| `architecture22_gate` | string | no | `planned` または `active`。scenarios がある場合の default は `planned`。 |

Allowed `stage` values:

```text
lexical
parse_only
declaration_symbol
type_elaboration
formula_statement
proof_verification
advanced_semantics
```

String values は [staged_model.md](./staged_model.md) と一致する。

## Kind And Outcome Compatibility

`kind` は corpus role を表す。`expected_outcome` は harness result contract を表す。

Allowed `kind` values:

| Kind | Meaning |
|---|---|
| `pass` | Ordinary accepting test. |
| `fail` | Ordinary rejecting test. |
| `snapshot` | Snapshot comparison test. |
| `generated` | Generated test with stored origin metadata. |
| `fuzz_seed` | Fuzz seed or promoted fuzz regression. |
| `property_seed` | Property-test seed or promoted property regression. |

Allowed `expected_outcome` values:

| Outcome | Meaning |
|---|---|
| `pass` | Payload は `expected_phase` まで accepted されなければならない。 |
| `fail` | Payload は `expected_phase` で rejected されなければならない。 |
| `snapshot` | Snapshot hashes が一致しなければならない。 |
| `metadata_only` | Sidecar は validate されるが payload execution は期待しない。 |

Compatibility:

| `kind` | Allowed `expected_outcome` |
|---|---|
| `pass` | `pass`, `snapshot` |
| `fail` | `fail`, `snapshot` |
| `snapshot` | `snapshot` |
| `generated` | `pass`, `fail`, `snapshot` |
| `fuzz_seed` | `fail`, `metadata_only` |
| `property_seed` | `pass`, `fail`, `metadata_only` |

`metadata_only` は current profile で実行されない seed metadata にのみ許可する。Default fast profile の committed `.miz`、`.src`、`.cert.json` payloads では valid ではない。

## Pipeline Phase Values

Allowed `expected_phase` values:

| Phase | Meaning |
|---|---|
| `lex` | Lexical analysis. |
| `parse` | Parsing and surface syntax recovery. |
| `resolve` | Declaration collection and name/module resolution. |
| `type_check` | Type checking, attribute/mode checking, and early elaboration. |
| `elaboration` | Core elaboration and binder normalization. |
| `cluster_resolution` | Registration and cluster expansion. |
| `overload_resolution` | Overload and template candidate selection. |
| `statement_check` | Typed statement and local context checking. |
| `vc_generation` | Verification-condition generation. |
| `verification` | Proof search/policy verification boundary. |
| `certificate_check` | Certificate parsing and structural validation. |
| `kernel_check` | Kernel replay and rejection boundary. |

Later compiler crates は internal phases を refine してよいが、expectation files はこれら stable external phase ids を使う。

## Architecture-22 Matrix Metadata

task 14 は、placeholder runner を追加せずに architecture-22 incremental/parallel
verification regression matrix を記録する。Scenario metadata は expectation parsing と
metadata `plan` command の間に validate される。

`architecture22_scenarios` は optional である。これが無い場合、
`architecture22_gate` と `architecture22_equivalence_class` も無ければならない。
存在する場合、scenario ids は known、non-empty、unique、lexicographic sort 済みで
なければならない。duplicate または unsorted array は silently normalize せず
validation error とする。

`architecture22_equivalence_class` が存在する場合、known registry class であり、
listed scenario すべてと一致しなければならない。複数 class を覆う sidecar はこの
field を省略し、reporting は scenario ごとの registry class を使う。

`architecture22_gate` は scenarios がある場合 `planned` を default とする。
`active` は、将来の consumer-specific increment が listed scenario ids に active
eligibility を与えた後だけ受理する。既存の active parse-only、
declaration-symbol、type-elaboration tag は architecture-22 matrix row を active
にしない。

task 14 registry:

| Scenario id | Equivalence class | Active eligibility |
|---|---|---|
| `artifact_manifest_atomicity` | `atomic_publication` | none |
| `cache_hit_miss_timing` | `observable_outputs_equal` | none |
| `cache_key_race` | `single_canonical_publication` | none |
| `clean_incremental_artifact_equivalence` | `observable_outputs_equal` | none |
| `clean_parallel_equivalence` | `observable_outputs_equal` | none |
| `externally_attested_non_upgrade` | `evidence_class_not_upgraded` | none |
| `incremental_parallel_equivalence` | `observable_outputs_equal` | none |
| `missing_dependency_slice_cache_miss` | `cache_miss_only` | none |
| `notation_operator_invalidation` | `downstream_invalidation` | none |
| `proof_witness_mismatch` | `cache_miss_only` | none |
| `randomized_atp_completion_order` | `deterministic_policy_selection` | none |
| `randomized_ready_task_scheduling` | `canonical_order_equal` | none |
| `registration_cluster_invalidation` | `downstream_invalidation` | none |
| `registration_origin_deletion` | `downstream_invalidation` | none |
| `stale_snapshot_non_publication` | `stale_result_not_published` | none |
| `theorem_proof_body_invalidation` | `local_refresh_only` | none |
| `theorem_status_invalidation` | `downstream_invalidation` | none |
| `vcid_reorder_anchor_reuse` | `reuse_requires_full_identity` | none |

## Public Enum Forward Compatibility

task 12 は `mizar-frontend` task 25 の手続きを expectation-schema enum と、public
module tree から見える crate-local TOML support enum に適用する。これらは downstream-facing
metadata surface であり、`#[non_exhaustive]` を維持しなければならない。downstream
caller は wildcard match arm を保つ必要がある。一方、`mizar-test` 内部の match は現在
知られている variant に対して exhaustive のままでよい。

| Public enum | Owner | Decision |
|---|---|---|
| `TestKind` | `expectation` corpus role と layout surface | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ExpectedOutcome` | `expectation` result contract | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `PipelinePhase` | `expectation` phase boundary ids | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `Architecture22Gate` | `expectation` architecture-22 planned/active metadata gate | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `TomlValue` | expectation と manifest metadata 用の `toml_lite` parser support | `#[non_exhaustive]` downstream forward-compatible surface。 |

この module が所有する exhaustive public enum exception はない。

## Pass Expectations

Pass expectations は failure identity を必要としない。

```toml
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
```

Fields:

| Field | Required | Meaning |
|---|---:|---|
| `expected_phase` | yes | Harness がこの test で実行すべき latest phase. |
| `diagnostic_codes` | yes | Expected diagnostics。Empty は diagnostics なしを意味する。 |
| `snapshots` | no | 現在の parse-only `SurfaceAst` baseline path。該当する場合のみ。 |

Expectation で明示的に許可されていない error diagnostic が出た場合、pass test は fail する。

## Fail Expectations

Fail expectations は stable failure identity を記録しなければならない。

```toml
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "type_error"
rejection_reason = "invalid_type_argument"
diagnostic_codes = ["E-TYPE-INVALID-ARGUMENT"]
stable_detail_key = "types.dependent_mode.invalid_argument"
```

Fields:

| Field | Required | Meaning |
|---|---:|---|
| `expected_phase` | yes | Input を soundly reject すべき earliest phase. |
| `failure_category` | yes | Failure semantics の stable category. |
| `rejection_reason` | conditional | Certificate and kernel rejection では必須。それ以外では optional. |
| `diagnostic_codes` | yes | Deterministic order の stable diagnostic codes. |
| `diagnostic_payloads` | no | Deterministic order の machine-readable diagnostic payload summaries. |
| `snapshots` | no | 現在の parse-only `SurfaceAst` baseline path。該当する場合のみ。 |
| `stable_detail_key` | yes | Diagnostic wording から独立した stable detail identity. |

Fail test が成功した場合は harness failure である。Expected より早い phase で fail した場合も、その earlier sound boundary に expectation を意図的に更新しない限り harness failure である。

## Lexical Expectations

Lexical fixtures は parser を呼ばずに tokens and lexical diagnostics を check できる。

```toml
stage = "lexical"
expected_outcome = "pass"
expected_phase = "lex"

[[tokens]]
kind = "identifier"
lexeme = "alpha"

[[tokens]]
kind = "reserved"
lexeme = "definition"
```

Token expectations は smoke tests では optional だが、token-level coverage を claim する fixtures では required である。

Lexical fixtures は、machine-readable diagnostic payload の契約を所有する場合、`diagnostic_payloads` も指定してよい。これは `diagnostic_codes` を補足し、人間向けメッセージ文への依存を避ける。

## Parse-Only Expectations

Parse-only fixtures は semantic validation なしで syntactic acceptance、rejection、AST shape を check する。

```toml
stage = "parse_only"
expected_phase = "parse"
tags = ["active_parse_only"]
ast_profile = "surface"
snapshot_profiles = ["surface_ast"]
```

parse-only corpus runner は、`active_parse_only` tag を持つ `.miz` の pass/fail
sidecar だけを実行する。tag のない parse-only sidecar は引き続き discover と
trace の対象だが、将来の grammar 作業用の inactive seed metadata に留める。
現在の runner では、`diagnostic_codes` は `missing_end` のような bare parser
syntax key と比較する。

active parse-only の pass/fail sidecar は、移行用の `snapshots` field も使用してよい。

```toml
snapshots = "snapshots/parser/pass_parser_minimal_token_stream_001.surface_ast.snap"
```

この path は `tests/` からの相対 path で、`tests/snapshots/` 配下に留めなければ
ならず、commit 済みの `SurfaceAst::snapshot_text()` baseline を指す。parse-only
runner は diagnostics が一致した後、その baseline を byte-for-byte で比較する。
snapshot を要求したのに parser AST がない場合、baseline が欠落または unreadable
な場合、または内容が一致しない場合は harness failure である。通常の
parse-only run は snapshot baseline を rewrite しない。

現在の parser recovery case が preprocess や lexing 由来の frontend recovery
diagnostic も同時に出す場合、sidecar は `allow_frontend_recovery_diagnostics` を
追加して parser syntax key だけを assert してよい。この opt-in がない場合、
syntax 以外の diagnostic も assertion result に含まれる。

Parse-only expectations は type、resolver、proof、certificate、kernel failure identities を含めてはならない。

## Declaration And Symbol Expectations

Declaration and symbol expectations は、clean resolver execution、選択された
positive declaration-symbol fact、または resolver failures を assert する。

```toml
stage = "declaration_symbol"
expected_phase = "resolve"
expected_outcome = "pass"
diagnostic_codes = []
tags = ["active_declaration_symbol"]
```

Declaration-symbol corpus runner は `active_declaration_symbol` tag を持つ `.miz`
pass/fail sidecar だけを実行する。tag のない sidecar は発見と traceability の対象だが、
inactive seed metadata のままにする。

現在の runner は、pass case が frontend assertion diagnostic と resolver symbol
diagnostic を出さないことを check する。pass sidecar は追加で
`declaration_symbol_payloads` を設定し、SymbolEnv 由来の exact / sorted fact key を
assert してよい。対応する fact key は、stable な symbol / definition data のみから
作る: primary spelling（percent-escaped）、symbol kind、definition kind、visibility、
export status。source range、id、signature、snapshot、import、name reference、
label reference、Core IR、VC、proof payload は使わない。sidecar は未実装の
`[[symbols]]` table assertion を含めてはならない。

```toml
declaration_symbol_payloads = [
  "declaration_symbol.symbol.kind.VisibleTheorem.theorem",
  "declaration_symbol.definition.kind.VisibleTheorem.theorem",
]
```

`declaration_symbol_payloads` は active declaration-symbol pass expectation でのみ
有効であり、各 entry は non-empty でなければならない。

public resolver diagnostic code が仕様化されるまで、すべての active
declaration-symbol case は `diagnostic_codes = []` とし、active gate は non-empty
値を拒否する。active fail case は resolver-owned internal detail key を
`diagnostic_payloads` で assert する。payload list が無い場合は `stable_detail_key`
へ fallback する。

```toml
expected_outcome = "fail"
expected_phase = "resolve"
failure_category = "resolve_error"
stable_detail_key = "declaration_symbol.symbol.duplicate_declaration"
diagnostic_codes = []
diagnostic_payloads = [
  "declaration_symbol.symbol.duplicate_declaration",
]
tags = ["active_declaration_symbol"]
```

stable resolver diagnostic-code range が存在した後は、resolver fail case も
`diagnostic_codes` で user-facing code を assert してよい。

## Type And Elaboration Expectations

Type and elaboration expectations は normalized types、inserted views、type diagnostics を assert してよい。

```toml
stage = "type_elaboration"
expected_phase = "type_check"

[[types]]
subject = "X"
expected = "set"
```

これらの tests は、expectation が missing prerequisite を明示的に target しない限り、built-ins と lower stages で admitted された symbols だけを使う。

`type-elaboration` runner の active `.miz` sidecar は
`tags = ["active_type_elaboration"]`、`stage = "type_elaboration"`、
`expected_phase = "type_check"` を持ち、public checker diagnostic code が指定されるまで
`diagnostic_codes = []` を保つ。runner は checker work の前に frontend parsing と
resolver symbol collection を実行する。

対応済み source-derived pass slice は reserve-only builtin `set` / `object`
declaration に限定する。対象は top-level の unrecovered reserve item で、segment が
1 個以上の identifier と exactly one bare builtin type-expression head を持ち、
attribute、argument、parameter prefix、non-builtin symbol head を含まないものだけである。
この pass case は、runner が syntax-free な source reserve payload へ抽出する reserve
binding を少なくとも 1 つ含む必要がある。checker-owned source reserve seam は
module `BindingEnv`、binding ごとの `DeclarationInput`、binding 固有の
`TypeExpressionInput` site、`DeclarationChecker` output を構築し、runner はその後
`TypedAst`、`ResolvedTypedAst`、summary-only の `mizar-core`
`ResolvedTypedAstSummary::from_ast` readiness read と binder-only `CoreContext`
preparation へ継続する。同じ source type-expression range を共有する複数 identifier
でも distinct typed site を使わなければならない。
この summary/context readiness check を `CoreIr`、`ControlFlowIr`、VC、proof execution として扱ってはならない。
pass-slice traceability row に cover され、empty `diagnostic_codes` と internal detail
payload なしを assert する。

```toml
expected_outcome = "pass"
expected_phase = "type_check"
diagnostic_codes = []
diagnostic_payloads = []
tags = ["active_type_elaboration"]
```

lower stages が成功したが、case が未対応 source-to-checker payload family を必要とする場合、
runner は stable external-gap detail key
`type_elaboration.external_dependency.ast_payload_extraction` を report する。

```toml
expected_outcome = "fail"
expected_phase = "type_check"
failure_category = "external_dependency_gap"
stable_detail_key = "type_elaboration.external_dependency.ast_payload_extraction"
diagnostic_codes = []
diagnostic_payloads = [
  "type_elaboration.external_dependency.ast_payload_extraction",
]
tags = ["active_type_elaboration"]
```

対応済み checker-owned diagnostic slice は、source reserve seam が生成する checker
detail key を代わりに assert してよい。task 50 は same-module attributed builtin
reserve head が
`type_elaboration.checker.checker.declaration.deferred.evidence_query` で停止することを
許可し、task 51 は attribute や type argument のない unique な same-module
`LocalSource` `SymbolKind::Mode` reserve head が
`type_elaboration.checker.checker.type.external.mode_expansion_payload` と、出力される場合は
paired recovery key で停止することを許可する。task 52 は attribute や type argument のない
unique な same-module `LocalSource` `SymbolKind::Structure` reserve head が
`type_elaboration.checker.checker.declaration.deferred.evidence_query` で停止することを許可する。
task 53 は same-module no-argument attribute payload をその local structure head に attach し、
full attributed-type existential evidence がないため同じ evidence-query key で停止することを許可する。
task 54 は local mode head に same-module no-argument attribute payload を attach し、
supported real mode expansion がない場合や同じ mode が bare reserve use と mixed の場合に
evidence-query key なしで mode-expansion key に停止することを許可する。
task 57 は RHS が same-module local structure head である real local-mode expansion が、
base-shape / constructor-witness evidence 欠落のため evidence-query key で停止することを許可する。
task 58 は RHS が attributed builtin head である real local-mode expansion が、
attributed-type existential evidence 欠落のため evidence-query key で停止することを許可する。
task 59 は real direct bare-builtin expansion を持つ attributed local-mode reserve head が、
attributed-type existential evidence 欠落のため evidence-query key で停止することを許可する。
task 60 は real direct local-structure RHS expansion を持つ attributed local-mode reserve head が、
base-shape / constructor-witness と full attributed-type evidence 欠落のため evidence-query key で停止することを許可する。
これらは fail case であり、pass-slice coverage ではない。

detailed type assertion table とより広い type pass expectation は、runner が `.miz` source
から checker-owned payload を non-builtin declaration、imported symbol、unresolved /
ambiguous symbol、attribute / mode / structure argument、imported attributed structure head、
structure base-shape evidence、term、formula、
coercion、fact、overload evidence、CoreIr、ControlFlowIr、VC payload、proof evidence を捏造せず
構築できるまで deferred のままにする。

## Formula, Statement, And Proof Expectations

Formula and statement expectations は typed formulas、statement structure、labels、local proof context を check する。

Proof expectations は verification outcome checks を追加する。

```toml
stage = "proof_verification"
expected_phase = "verification"
expected_outcome = "fail"
failure_category = "kernel_rejection"
rejection_reason = "invalid_sat_proof"
diagnostic_codes = ["E-KERNEL-INVALID-PROOF"]
stable_detail_key = "soundness.false_arithmetic.one_eq_zero"
```

Soundness tests は、certificate/kernel payloads を必要とするかどうかに応じて、ここまたは `advanced_semantics` に属する。

## Certificate Expectations

Certificate tests は certificate payloads を使い、`.miz` parsing に依存しない。

```toml
kind = "fail"
stage = "advanced_semantics"
domain = "certificate"
source = "fail_certificate_sat_satisfiable_refutation_001.cert.json"
expected_outcome = "fail"
expected_phase = "kernel_check"
failure_category = "kernel_rejection"
rejection_reason = "invalid_sat_refutation"
diagnostic_codes = []
stable_detail_key = "soundness.certificate.invalid_sat_refutation"
```

Certificate expectations は `rejection_reason` を必ず含める。

## Snapshot Expectations

Snapshot expectations は deterministic artifact hashes を compare する。

```toml
expected_outcome = "snapshot"

[[snapshots]]
profile = "surface_ast"
path = "pass_parser_block_001.surface_ast.json"
hash = "sha256:..."
```

Snapshot update mode は explicit である。Harness は normal pass/fail execution 中に snapshots を rewrite してはならない。

`[[snapshots]]` hash registry は将来の general snapshot contract である。上記の
current parser task-38 slice は active pass/fail sidecar 向けの parse-only
`SurfaceAst` shortcut に限られ、general `kind = "snapshot"` execution や
hash-registry update mode を完了するものではない。

## Generated, Fuzz, And Property Metadata

Generated and fuzz/property regression tests は provenance を記録する。

```toml
[origin]
schema_version = 1
kind = "generated"
generator = "grammar-smoke"
generator_version = "0.1.0"
seed = "0000000000000001"
profile = "lexical-identifiers"
expected_outcome = "pass"
minimized = false
```

`[origin]` は `kind = "generated"`、`kind = "fuzz_seed"`、`kind = "property_seed"` の
sidecars で必須である。`origin.kind` と `origin.expected_outcome` は sidecar の
top-level `kind` と `expected_outcome` に一致する。したがって metadata-only handoff
anchors は両方で `expected_outcome = "metadata_only"` を使う。Unknown origin fields
は reject される。`origin.schema_version` は `1`、`generator`、
`generator_version`、`seed`、`profile` は required non-empty strings、
`minimized` は boolean である。Promoted fuzz failures は `kind = "fuzz_seed"` と
`expected_outcome = "fail"` のまま、top level に executable fail identity を保持する。

すべての fuzz seed sidecars は `origin.original_failure_category` を通して original
failure category family を preserve する。Promoted executable fuzz failures では、その
origin category が top-level `failure_category` と一致しなければならない。

## Validation

Harness は次を validate する。

1. The sidecar parses as TOML.
2. `schema_version` is supported.
3. `id` equals the sidecar stem.
4. `source` exists and has the same stem.
5. `kind`, `stage`, and `expected_outcome` are compatible.
6. `spec_refs` are non-empty for committed tests and exist in the traceability manifest.
7. Fail expectations include failure identity fields.
8. Certificate and kernel rejections include `rejection_reason`.
9. Diagnostic codes are sorted in the expected deterministic order.
10. 移行用 parse-only `snapshots` path は active parse-only pass/fail に限定され、
    `snapshots/` 配下の clean な tests-root-relative path かつ `.snap` file でなければ
    ならない。missing、unreadable、または mismatched baseline は harness failure である。
11. General snapshot entries use supported hash algorithms.
12. Generated/fuzz/property tests include origin metadata.
13. Architecture-22 matrix metadata は known sorted scenario ids、known gate values、
    matching equivalence classes を使う。orphan gate/class fields は reject する。
14. Unknown fields are rejected unless the schema version explicitly permits extensions.

Coverage completeness の validation は [traceability.md](./traceability.md) で定義される validation mode に依存する。Schema validation 自体は mode independent である。

## Constraints And Assumptions

- Expectations are reviewed source, not generated truth from current compiler output.
- Diagnostic text is not matched by default; stable diagnostic codes and detail keys are matched.
- Sidecar parsing must work even when the corresponding source file is invalid.
- Schema migrations are explicit and versioned.
