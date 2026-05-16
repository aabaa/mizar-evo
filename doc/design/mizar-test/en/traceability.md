# Module: traceability

> Canonical language: English. Japanese companion: [../ja/traceability.md](../ja/traceability.md).

## Purpose

This module defines the traceability manifest that connects `doc/spec/`
requirements to committed tests without adding test links to the specification
text itself.

The specification remains a readable language reference. Test coverage is
tracked by a separate machine-readable manifest owned by `mizar-test`.

## Design Decision

Specification-to-test links live outside `doc/spec/`.

The traceability model is bidirectional:

- the manifest maps spec requirements to the tests that cover them;
- each test expectation sidecar maps the test back to one or more spec
  requirement ids.

The harness validates both directions.

```text
doc/spec/...                    pure specification text
tests/coverage/spec_trace.toml  spec requirement -> tests
*.expect.toml                   test -> spec requirement ids
```

## Manifest Location

The canonical manifest is:

```text
tests/coverage/spec_trace.toml
```

Additional generated reports may be written under `tests/coverage/reports/`,
but those reports are derived artifacts. The manifest is the source of truth.

## Requirement Record

Each requirement record describes a checkable unit of the specification.

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
| `id` | Stable requirement id. Never reused for unrelated semantics. |
| `source` | Specification file that owns the requirement. |
| `section` | Human-readable section heading or section number. |
| `stage` | Staged model stage that first owns executable coverage. |
| `status` | `planned`, `covered`, `partial`, `deferred`, or `obsolete`. |
| `required` | Whether release coverage requires this item. |
| `coverage` | Expected coverage shape. |
| `tests` | Canonical relative paths to expectation sidecars or fixture metadata. |

Optional fields:

| Field | Meaning |
|---|---|
| `anchors` | Stable heading anchors when available. |
| `notes` | Short human review notes. |
| `depends_on` | Other requirement ids that must be covered first. |
| `deferred_reason` | Required when `status = "deferred"`. |
| `issue` | Tracking issue or design discussion reference. |

## Coverage Shapes

`coverage` records what kind of test evidence is expected.

Allowed values:

| Value | Meaning |
|---|---|
| `none` | No executable test is required. Used for explanatory text. |
| `pass` | At least one accepting test is required. |
| `fail` | At least one rejecting test is required. |
| `pass_and_fail` | Both accepting and rejecting tests are required. |
| `diagnostic` | A stable diagnostic or failure category must be checked. |
| `snapshot` | A deterministic snapshot must be checked. |
| `property` | A property or generated test family covers the item. |
| `manual_review` | Human review is required; executable coverage is not sufficient. |

Multiple shapes may be required by splitting a specification section into
several requirement records.

## Test Sidecar Reference

Each expectation sidecar records the spec requirements it covers.

```toml
schema_version = 1
id = "pass_lexical_identifier_basic_001"
stage = "lexical"
spec_refs = [
  "spec.en.02.lexical.identifiers.basic",
]
```

The sidecar `stage` must match the requirement `stage`, unless the requirement
explicitly allows coverage from a later stage through a `depends_on` chain.

## Validation

The harness validates:

1. Every manifest `source` exists.
2. Every `id` is unique.
3. Every listed test path exists.
4. Every listed test sidecar points back to the requirement id.
5. Every sidecar `spec_refs` entry exists in the manifest.
6. Stage names match the staged model.
7. Required coverage shapes are satisfied when the validation mode requires
   coverage completeness.
8. Deferred required items include a `deferred_reason`.
9. Obsolete items are not referenced by active tests.
10. Manifest records are sorted deterministically by `id`.

Validation must not parse `doc/spec/` prose beyond checking that referenced
files exist. The manifest owns the granularity of requirements.

## Validation Modes

Traceability validation has modes:

| Mode | Purpose | Coverage Completeness |
|---|---|---|
| `metadata` | Minimal crate and local editing. | Not required; planned items without tests are warnings at most. |
| `development` | Normal CI during implementation. | Required only for requirements marked `status = "covered"` or `partial`. |
| `release` | Release readiness gate. | Required for every `required = true` requirement unless `status = "deferred"` with a reason. |

All modes validate manifest syntax, unique ids, source file existence, known
stage ids, known sidecar references, and sidecar back-references.

Only `release` mode turns missing required coverage into an error. This allows
the project to maintain a complete planned coverage map before the compiler
pipeline exists.

## Coverage Status

`status` is derived during reporting but stored in the manifest for review
workflow.

Rules:

- `planned` means the requirement is known but lacks sufficient tests.
- `partial` means some required coverage exists but not all coverage shapes are
  satisfied.
- `covered` means all required coverage shapes are satisfied by active tests.
- `deferred` means coverage is intentionally postponed.
- `obsolete` means the requirement no longer applies and active tests must not
  claim it.

The report flags stored status when it disagrees with computed status. The
severity of that disagreement is determined by validation mode.

## Stage Interaction

Traceability uses the staged model from [staged_model.md](./staged_model.md).

Coverage credit is assigned only when lower-stage prerequisites are already
covered, declared as built-ins, or listed in `depends_on` with acceptable
status.

For example, a parser fixture can cover the syntax of a cluster declaration,
but it does not cover cluster expansion semantics. The semantic requirement
remains planned until advanced semantic tests exist.

## Reporting

The default report groups results by:

- spec file;
- stage;
- status;
- missing coverage shape;
- tests with unknown spec refs;
- tests that cover obsolete requirements.

Reports must be deterministic and suitable for CI output.

## Constraints and Assumptions

- `doc/spec/` remains free of per-test links.
- Requirement ids are stable public identifiers for the test corpus.
- The manifest may be edited manually, but validation is automated.
- Generated tests can contribute coverage only through committed expectation
  metadata.
- Coverage is semantic evidence, not line or branch coverage.
