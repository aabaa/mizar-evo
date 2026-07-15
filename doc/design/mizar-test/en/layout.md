# Module: layout

> Canonical language: English. Japanese companion: [../ja/layout.md](../ja/layout.md).

## Purpose

This module defines the filesystem discovery boundary and adjacent-sidecar layout contract for Mizar Evo tests.

The layout is optimized for large `.miz` corpora, fail-heavy regression testing, deterministic snapshots, and clear ownership of expected outcomes.

## Public API

```rust
pub struct DiscoveredLayout {
    pub payloads: Vec<PathBuf>,
    pub sidecars: Vec<PathBuf>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub fn discover(
    tests_root: &Path,
) -> Result<DiscoveredLayout, std::io::Error>;

pub fn unknown_roots(
    tests_root: &Path,
) -> Result<Vec<PathBuf>, std::io::Error>;
```

This is the public `mizar_test::layout` module surface; these items are not
re-exported at the crate root. `DiscoveredLayout` contains deterministically
sorted payload and sidecar path inventories plus layout diagnostics.
`discover` reports missing optional known roots and missing adjacent sidecars,
while `unknown_roots` returns the sorted unknown-directory inventory. The
harness applies validation-mode policy to that inventory.

Authoritative expectations live in sidecar files. Fail, soundness, certificate, and snapshot expectations must use sidecars because they must be parsed without depending on `.miz` frontend correctness. Inline metadata is allowed only for non-authoritative tags that the parser can ignore safely.

The layout module does not own parsed test cases or expectation metadata.
`TestCase` and `TestPlan` are harness-owned; `TestKind` and `Expectation` are
expectation-schema-owned. See [harness.md](./harness.md) and
[expectation_schema.md](./expectation_schema.md).

## Directory Layout

Committed corpus roots:

```text
tests/lexical/pass/
tests/lexical/fail/

tests/miz/pass/parser/
tests/miz/pass/types/
tests/miz/pass/attributes/
tests/miz/pass/clusters/
tests/miz/pass/theorems/

tests/miz/fail/parser/
tests/miz/fail/types/
tests/miz/fail/clusters/
tests/miz/fail/overload/
tests/miz/fail/substitution/
tests/miz/fail/soundness/

tests/generated/
tests/fuzz/
tests/property/
tests/stress/

tests/certificates/
tests/snapshots/
```

Roots may be absent until their first committed test. Once a root contains
executable payloads, those payloads follow the same sidecar and deterministic
discovery rules.

Additional subdirectories may be added only when they preserve the
pass/fail/snapshot distinction.

### Executable Payloads

Executable payloads are files that represent test inputs:

| Extension | Meaning |
|---|---|
| `.miz` | Mizar source input. |
| `.src` | Lexical or parser source snippet input. |
| `.cert.json` | Certificate payload input. |
| `.fixture.toml` | Structured non-source fixture input. |

Every executable payload in the committed corpus must have an adjacent
`.expect.toml` with the same stem. Files such as `README.md`, `.gitkeep`, and
snapshot output files are not executable payloads.

### Certificate Test Layout

Certificate tests use their own pass/fail split because many certificate failures do not require a `.miz` source file.

```text
tests/certificates/pass/
tests/certificates/fail/malformed/
tests/certificates/fail/substitution/
tests/certificates/fail/sat/
tests/certificates/fail/symbols/
tests/certificates/fail/resources/
```

Certificate payloads use `.cert.json` unless a later schema defines a compact binary format. Every certificate test has an adjacent `.expect.toml`.

```text
tests/certificates/fail/sat/fail_certificate_sat_satisfiable_refutation_001.cert.json
tests/certificates/fail/sat/fail_certificate_sat_satisfiable_refutation_001.expect.toml
```

The expectation records the expected `certificate_rejection` or `kernel_rejection` category and the stable rejection reason, such as `invalid_sat_proof`, `invalid_sat_refutation`, `invalid_substitution`, `malformed_certificate`, `context_mismatch`, `missing_provenance`, `unsupported_certificate_format`, `unresolved_symbol`, `timeout`, or `resource_exhaustion`.

## Naming Rules

Test file names use stable snake_case names:

```text
fail_soundness_false_arithmetic_001.miz
fail_substitution_capture_001.miz
pass_cluster_chain_basic_001.miz
snapshot_vc_simple_theorem_001.miz
```

Rules:

- executable pass/fail/snapshot names start with the expected high-level outcome when they live under a pass/fail/snapshot split;
- names include the semantic domain;
- numeric suffixes are stable and never reused for unrelated cases;
- minimized fuzz regressions keep a short human-readable name plus original seed metadata.
- oversized generated `.miz` files use `tests/stress/` and the `stress` profile rather than the default fast corpus.

## Expected Result Files

For a source test:

```text
tests/miz/fail/substitution/fail_substitution_capture_001.miz
tests/miz/fail/substitution/fail_substitution_capture_001.expect.toml
```

`expect.toml` records expected phase, failure category, rejection reason, diagnostic codes, and snapshot profiles.

Expected output is a contract. It must not be regenerated silently from current compiler behavior.

## Algorithm / Logic

Test discovery:

1. Walk only known test roots.
2. Sort the discovered `PathBuf` values deterministically.
3. Collect executable payload and sidecar path inventories separately.
4. Record layout diagnostics for missing optional known roots or adjacent sidecars.
5. Return `DiscoveredLayout`; the harness parses expectations and builds the deterministic `TestCase`/`TestPlan` projection.

## Tests

Required layout-focused scenarios:

- discovery order is stable across filesystems;
- missing executable payload metadata is an error;
- missing optional known roots produce warnings;
- unknown-directory inventory is sorted deterministically.

Direct module-level coverage remains open for raw payload/sidecar ordering,
missing-known-root warnings, and multiple-unknown-root ordering. Current
harness/expectation integration covers missing-sidecar rejection, rejects
duplicate test ids, retains generated and fuzz-minimized origin metadata, and
applies explicit validation-mode policy to unknown directories.

## Constraints and Assumptions

- Test discovery must not depend on OS directory iteration order.
- Sidecar metadata schema is versioned.
- Pass tests record `diagnostic_codes = []` when they expect no diagnostics.
- Fail tests must state the expected failure category and stable detail key.
