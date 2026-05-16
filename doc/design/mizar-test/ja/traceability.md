# Module: traceability

> Canonical language: English. English canonical version: [../en/traceability.md](../en/traceability.md).

## Purpose

この module は、`doc/spec/` requirements と committed tests を接続する traceability manifest を定義する。Specification text 自体には test links を追加しない。

Specification は読みやすい language reference として維持する。Test coverage は `mizar-test` が所有する machine-readable manifest で管理する。

## Design Decision

Specification-to-test links は `doc/spec/` の外に置く。

Traceability model は bidirectional である。

- manifest は spec requirements から、それを cover する tests へ map する
- each test expectation sidecar は test から one or more spec requirement ids へ map する

Harness は両方向を validate する。

```text
doc/spec/...                    pure specification text
tests/coverage/spec_trace.toml  spec requirement -> tests
*.expect.toml                   test -> spec requirement ids
```

## Manifest Location

Canonical manifest:

```text
tests/coverage/spec_trace.toml
```

Additional generated reports は `tests/coverage/reports/` に出力してよいが、それらは derived artifacts である。Manifest が source of truth である。

## Requirement Record

各 requirement record は specification の checkable unit を表す。

```toml
[[requirement]]
id = "spec.en.02.lexical.identifiers.basic"
source = "doc/spec/en/02.lexical_structure.md"
section = "2.6 Identifiers"
stage = "lexical"
status = "planned"
required = true
coverage = "pass_and_fail"
tests = []
```

Fields:

| Field | Meaning |
|---|---|
| `id` | Stable requirement id。Unrelated semantics に再利用してはならない。 |
| `source` | Requirement を所有する specification file。 |
| `section` | Human-readable section heading or section number。 |
| `stage` | Executable coverage を最初に所有する staged model stage。 |
| `status` | `planned`, `covered`, `partial`, `deferred`, or `obsolete`。 |
| `required` | Release coverage がこの item を要求するか。 |
| `coverage` | Expected coverage shape。 |
| `tests` | Expectation sidecars または fixture metadata への canonical relative paths。 |

Optional fields:

| Field | Meaning |
|---|---|
| `anchors` | Available な stable heading anchors。 |
| `notes` | Short human review notes。 |
| `depends_on` | 先に cover されるべき other requirement ids。 |
| `deferred_reason` | `status = "deferred"` の場合に必須。 |
| `issue` | Tracking issue or design discussion reference。 |

## Coverage Shapes

`coverage` は期待される test evidence の種類を記録する。

Allowed values:

| Value | Meaning |
|---|---|
| `none` | Executable test は不要。Explanatory text に使う。 |
| `pass` | 少なくとも one accepting test が必要。 |
| `fail` | 少なくとも one rejecting test が必要。 |
| `pass_and_fail` | Accepting and rejecting tests の両方が必要。 |
| `diagnostic` | Stable diagnostic or failure category を check する必要がある。 |
| `snapshot` | Deterministic snapshot を check する必要がある。 |
| `property` | Property or generated test family が cover する。 |
| `manual_review` | Human review が必要。Executable coverage だけでは不十分。 |

複数の shapes が必要な場合、specification section を複数の requirement records に分割する。

## Test Sidecar Reference

Each expectation sidecar は cover する spec requirements を記録する。

```toml
schema_version = 1
id = "pass_lexical_identifier_basic_001"
stage = "lexical"
spec_refs = [
  "spec.en.02.lexical.identifiers.basic",
]
```

Sidecar の `stage` は requirement の `stage` と一致しなければならない。ただし requirement が `depends_on` chain によって later stage からの coverage を明示的に許す場合を除く。

## Validation

Harness は次を validate する。

1. Every manifest `source` exists.
2. Every `id` is unique.
3. Every listed test path exists.
4. Every listed test sidecar points back to the requirement id.
5. Every sidecar `spec_refs` entry exists in the manifest.
6. Stage names match the staged model.
7. Validation mode が coverage completeness を要求する場合、required coverage shapes are satisfied.
8. Deferred required items include a `deferred_reason`.
9. Obsolete items are not referenced by active tests.
10. Manifest records are sorted deterministically by `id`.

Validation は referenced files が存在すること以外に `doc/spec/` prose を parse してはならない。Requirement granularity は manifest が所有する。

## Validation Modes

Traceability validation は modes を持つ。

| Mode | Purpose | Coverage Completeness |
|---|---|---|
| `metadata` | Minimal crate and local editing. | Required ではない。Tests なしの planned items は最大でも warnings。 |
| `development` | Implementation 中の normal CI. | `status = "covered"` or `partial` の requirements にのみ required。 |
| `release` | Release readiness gate. | `status = "deferred"` with reason を除き、every `required = true` requirement に required。 |

All modes は manifest syntax、unique ids、source file existence、known stage ids、known sidecar references、sidecar back-references を validate する。

Missing required coverage を error にするのは `release` mode だけである。これにより compiler pipeline が存在する前から complete planned coverage map を維持できる。

## Coverage Status

`status` は reporting 時に derive されるが、review workflow のため manifest にも保存する。

Rules:

- `planned` means the requirement is known but lacks sufficient tests.
- `partial` means some required coverage exists but not all coverage shapes are satisfied.
- `covered` means all required coverage shapes are satisfied by active tests.
- `deferred` means coverage is intentionally postponed.
- `obsolete` means the requirement no longer applies and active tests must not claim it.

Report は stored status と computed status が一致しない場合に flag する。その severity は validation mode によって決まる。

## Stage Interaction

Traceability は [staged_model.md](./staged_model.md) の staged model を使う。

Coverage credit は lower-stage prerequisites が既に covered、built-ins として declared、または acceptable status を持つ `depends_on` に listed されている場合にのみ与える。

例えば parser fixture は cluster declaration の syntax を cover できるが、cluster expansion semantics は cover しない。Semantic requirement は advanced semantic tests が存在するまで planned のままである。

## Reporting

Default report は次で group する。

- spec file
- stage
- status
- missing coverage shape
- tests with unknown spec refs
- tests that cover obsolete requirements

Reports は deterministic で CI output に適していなければならない。

## Constraints and Assumptions

- `doc/spec/` は per-test links を持たない。
- Requirement ids は test corpus の stable public identifiers である。
- Manifest は manually edit してよいが、validation は automated である。
- Generated tests は committed expectation metadata を通じてのみ coverage に寄与できる。
- Coverage は semantic evidence であり、line or branch coverage ではない。
