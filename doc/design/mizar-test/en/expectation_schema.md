# Module: expectation_schema

> Canonical language: English. Japanese companion: [../ja/expectation_schema.md](../ja/expectation_schema.md).

## Purpose

This module defines the `.expect.toml` sidecar schema used by `mizar-test`.

Expectation files are the authoritative contract for committed tests. They are
parsed before compiler execution and must not depend on the `.miz` frontend.

## Design Decision

Every executable corpus item has exactly one expectation sidecar.

The sidecar owns:

- test identity;
- staged model placement;
- spec traceability back-references;
- expected outcome;
- expected failure identity;
- deterministic diagnostics and snapshot requirements.

The source file owns only the input program or fixture payload.

## File Pairing

Expectation sidecars use the same stem as the input file:

```text
tests/miz/pass/parser/pass_parser_block_001.miz
tests/miz/pass/parser/pass_parser_block_001.expect.toml

tests/lexical/pass/pass_lexical_identifier_001.src
tests/lexical/pass/pass_lexical_identifier_001.expect.toml

tests/certificates/fail/sat/fail_certificate_invalid_resolution_001.cert.json
tests/certificates/fail/sat/fail_certificate_invalid_resolution_001.expect.toml
```

The harness rejects missing sidecars for fail, soundness, certificate,
snapshot, generated, fuzz-regression, and property-regression tests. Pass tests
may omit sidecars only when an explicit harness mode allows legacy discovery;
the committed evo2 corpus should include sidecars for all executable tests.

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
| `id` | string | yes | Stable test id. Must equal the file stem. |
| `kind` | string | yes | `pass`, `fail`, `snapshot`, `generated`, `fuzz_seed`, or `property_seed`. |
| `stage` | string | yes | Staged model stage. |
| `domain` | string | yes | Human-readable test domain. |
| `source` | string | yes | Input file path relative to the sidecar directory. |
| `expected_outcome` | string | yes | `pass`, `fail`, `snapshot`, or `metadata_only`. |
| `spec_refs` | array of strings | yes | Requirement ids from `tests/coverage/spec_trace.toml`. |
| `profiles` | array of strings | no | Harness profiles that include this test. Defaults to `["fast"]`. |
| `tags` | array of strings | no | Non-authoritative grouping tags. |
| `notes` | string | no | Short review note. Not used for matching. |

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

The string values match [staged_model.md](./staged_model.md).

## Pass Expectations

Pass expectations require no failure identity.

```toml
expected_outcome = "pass"
expected_phase = "parse"
diagnostic_codes = []
```

Fields:

| Field | Required | Meaning |
|---|---:|---|
| `expected_phase` | yes | Latest phase the harness must execute for this test. |
| `diagnostic_codes` | yes | Expected diagnostics. Empty means no diagnostics. |
| `snapshots` | no | Snapshot profiles and hashes, when applicable. |

The harness fails a pass test if an error diagnostic is emitted unless that
diagnostic is explicitly allowed by the expectation.

## Fail Expectations

Fail expectations must state the stable failure identity.

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
| `expected_phase` | yes | Earliest phase that must soundly reject the input. |
| `failure_category` | yes | Stable category from failure semantics. |
| `rejection_reason` | conditional | Required for certificate and kernel rejection; optional otherwise. |
| `diagnostic_codes` | yes | Stable diagnostic codes in deterministic order. |
| `stable_detail_key` | yes | Stable detail identity independent of diagnostic wording. |

A fail test that succeeds is a harness failure. A fail test that fails earlier
than expected is also a harness failure unless the expectation is deliberately
updated to the earlier sound boundary.

## Lexical Expectations

Lexical fixtures may check tokens and lexical diagnostics without invoking the
parser.

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

Token expectations are optional for smoke tests but required for fixtures that
claim token-level coverage.

## Parse-Only Expectations

Parse-only fixtures check syntactic acceptance, rejection, or AST shape without
semantic validation.

```toml
stage = "parse_only"
expected_phase = "parse"
ast_profile = "surface"
snapshot_profiles = ["surface_ast"]
```

Parse-only expectations must not include type, resolver, proof, certificate, or
kernel failure identities.

## Declaration And Symbol Expectations

Declaration and symbol expectations may assert symbol table effects or resolver
failures.

```toml
stage = "declaration_symbol"
expected_phase = "resolve"

[[symbols]]
name = "EmptyDef"
kind = "attribute"
visibility = "public"
```

Undefined symbol tests use `failure_category = "resolve_error"` and a stable
resolver diagnostic code.

## Type And Elaboration Expectations

Type and elaboration expectations may assert normalized types, inserted views,
or type diagnostics.

```toml
stage = "type_elaboration"
expected_phase = "type_check"

[[types]]
subject = "X"
expected = "set"
```

These tests may use only built-ins and symbols admitted by lower stages unless
the expectation explicitly targets a missing prerequisite.

## Formula, Statement, And Proof Expectations

Formula and statement expectations check typed formulas, statement structure,
labels, and local proof context.

Proof expectations add verification outcome checks:

```toml
stage = "proof_verification"
expected_phase = "verification"
expected_outcome = "fail"
failure_category = "kernel_rejection"
rejection_reason = "invalid_sat_proof"
diagnostic_codes = ["E-KERNEL-INVALID-PROOF"]
stable_detail_key = "soundness.false_arithmetic.one_eq_zero"
```

Soundness tests belong here or to `advanced_semantics`, depending on whether
they require certificate/kernel payloads.

## Certificate Expectations

Certificate tests use certificate payloads and never depend on parsing `.miz`.

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

Certificate expectations must include `rejection_reason`.

## Snapshot Expectations

Snapshot expectations compare deterministic artifact hashes.

```toml
expected_outcome = "snapshot"

[[snapshots]]
profile = "surface_ast"
path = "pass_parser_block_001.surface_ast.json"
hash = "sha256:..."
```

Snapshot update mode is explicit. The harness must not rewrite snapshots during
normal pass/fail execution.

## Generated, Fuzz, And Property Metadata

Generated and fuzz/property regression tests record provenance.

```toml
[origin]
kind = "generated"
generator = "grammar-smoke"
generator_version = "0.1.0"
seed = "0000000000000001"
profile = "lexical-identifiers"
minimized = false
```

Promoted fuzz regressions must preserve their original failure category and
seed metadata.

## Validation

The harness validates:

1. The sidecar parses as TOML.
2. `schema_version` is supported.
3. `id` equals the sidecar stem.
4. `source` exists and has the same stem.
5. `kind`, `stage`, and `expected_outcome` are compatible.
6. `spec_refs` are non-empty for committed tests and exist in the traceability
   manifest.
7. Fail expectations include failure identity fields.
8. Certificate and kernel rejections include `rejection_reason`.
9. Diagnostic codes are sorted in the expected deterministic order.
10. Snapshot entries use supported hash algorithms.
11. Generated/fuzz/property tests include origin metadata.
12. Unknown fields are rejected unless the schema version explicitly permits
   extensions.

## Constraints And Assumptions

- Expectations are reviewed source, not generated truth from current compiler
  output.
- Diagnostic text is not matched by default; stable diagnostic codes and detail
  keys are matched.
- Sidecar parsing must work even when the corresponding source file is invalid.
- Schema migrations are explicit and versioned.
