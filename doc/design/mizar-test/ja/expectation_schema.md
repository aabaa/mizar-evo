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

tests/certificates/fail/sat/fail_certificate_invalid_resolution_001.cert.json
tests/certificates/fail/sat/fail_certificate_invalid_resolution_001.expect.toml
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
| `profiles` | array of strings | no | この test を含める harness profiles。Default は `["fast"]`。 |
| `tags` | array of strings | no | Non-authoritative grouping tags. |
| `notes` | string | no | Short review note. Matching には使わない。 |

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
| `snapshots` | no | Applicable な snapshot profiles and hashes. |

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

## Parse-Only Expectations

Parse-only fixtures は semantic validation なしで syntactic acceptance、rejection、AST shape を check する。

```toml
stage = "parse_only"
expected_phase = "parse"
ast_profile = "surface"
snapshot_profiles = ["surface_ast"]
```

Parse-only expectations は type、resolver、proof、certificate、kernel failure identities を含めてはならない。

## Declaration And Symbol Expectations

Declaration and symbol expectations は symbol table effects or resolver failures を assert してよい。

```toml
stage = "declaration_symbol"
expected_phase = "resolve"

[[symbols]]
name = "EmptyDef"
kind = "attribute"
visibility = "public"
```

Undefined symbol tests は `failure_category = "resolve_error"` と stable resolver diagnostic code を使う。

## Type And Elaboration Expectations

Type and elaboration expectations は normalized types、inserted views、type diagnostics を assert してよい。

```toml
stage = "type_elaboration"
expected_phase = "type_check"

[[types]]
subject = "X"
expected = "set"
```

These tests は、expectation が missing prerequisite を明示的に target しない限り、built-ins と lower stages で admitted された symbols だけを使う。

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
source = "fail_certificate_invalid_resolution_001.cert.json"
expected_outcome = "fail"
expected_phase = "kernel_check"
failure_category = "kernel_rejection"
rejection_reason = "invalid_sat_proof"
diagnostic_codes = ["E-KERNEL-INVALID-SAT-PROOF"]
stable_detail_key = "certificate.invalid_resolution.basic"
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

## Generated, Fuzz, And Property Metadata

Generated and fuzz/property regression tests は provenance を記録する。

```toml
[origin]
kind = "generated"
generator = "grammar-smoke"
generator_version = "0.1.0"
seed = "0000000000000001"
profile = "lexical-identifiers"
minimized = false
```

Promoted fuzz regressions は original failure category and seed metadata を preserve しなければならない。

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
10. Snapshot entries use supported hash algorithms.
11. Generated/fuzz/property tests include origin metadata.
12. Unknown fields are rejected unless the schema version explicitly permits extensions.

Coverage completeness の validation は [traceability.md](./traceability.md) で定義される validation mode に依存する。Schema validation 自体は mode independent である。

## Constraints And Assumptions

- Expectations are reviewed source, not generated truth from current compiler output.
- Diagnostic text is not matched by default; stable diagnostic codes and detail keys are matched.
- Sidecar parsing must work even when the corresponding source file is invalid.
- Schema migrations are explicit and versioned.
