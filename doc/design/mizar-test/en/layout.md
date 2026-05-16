# Module: layout

> Canonical language: English. Japanese companion: [../ja/layout.md](../ja/layout.md).

## Purpose

This module defines the filesystem layout and metadata contract for Mizar Evo tests.

The layout is optimized for large `.miz` corpora, fail-heavy regression testing, deterministic snapshots, and clear ownership of expected outcomes.

## Public API

```rust
pub struct TestCase {
    pub id: TestCaseId,
    pub path: TestPath,
    pub kind: TestKind,
    pub domain: TestDomain,
    pub metadata: TestMetadata,
}

pub enum TestKind {
    Pass,
    Fail,
    Snapshot,
    FuzzSeed,
    PropertySeed,
    Generated,
}

pub struct TestMetadata {
    pub expected_phase: Option<PipelinePhase>,
    pub expected_failure: Option<FailureCategory>,
    pub expected_rejection: Option<RejectionReason>,
    pub expected_diagnostics: Vec<DiagnosticExpectation>,
    pub snapshot_profiles: Vec<SnapshotProfile>,
}
```

Authoritative expectations live in sidecar files. Fail, soundness, certificate, and snapshot expectations must use sidecars because they must be parsed without depending on `.miz` frontend correctness. Inline metadata is allowed only for non-authoritative tags that the parser can ignore safely.

## Directory Layout

Required directories:

```text
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

tests/certificates/
tests/snapshots/
```

Additional subdirectories may be added only when they preserve the pass/fail/snapshot distinction.

## Naming Rules

Test file names use stable snake_case names:

```text
fail_soundness_false_arithmetic_001.miz
fail_substitution_capture_001.miz
pass_cluster_chain_basic_001.miz
snapshot_vc_simple_theorem_001.miz
```

Rules:

- names start with the expected high-level outcome;
- names include the semantic domain;
- numeric suffixes are stable and never reused for unrelated cases;
- minimized fuzz regressions keep a short human-readable name plus original seed metadata.

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
2. Sort paths by canonical relative path.
3. Pair `.miz` files with sidecar metadata.
4. Reject missing metadata for fail, soundness, certificate, and snapshot tests.
5. Build a deterministic `TestPlan`.

## Tests

Key scenarios:

- discovery order is stable across filesystems;
- missing fail metadata is an error;
- duplicate test ids are rejected;
- generated and fuzz-minimized tests are discoverable but marked by origin;
- unknown directories are ignored or rejected according to explicit harness mode.

## Constraints and Assumptions

- Test discovery must not depend on OS directory iteration order.
- Sidecar metadata schema is versioned.
- Pass tests may omit expected diagnostics only when they expect no diagnostics.
- Fail tests must state the expected failure category.
