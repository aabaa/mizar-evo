# Module: miz_corpus

> Canonical language: English. Japanese companion: [../ja/miz_corpus.md](../ja/miz_corpus.md).

## Purpose

This module defines the strategy for building and maintaining a large `.miz` corpus.

The corpus is an implementation asset. It is used to pin syntax, type behavior, cluster behavior, overload behavior, proof rejection, dependency fingerprints, and deterministic artifact output.

## Corpus Classes

| Class | Purpose | Ownership |
|---|---|---|
| handwritten minimal cases | focused parser/type/cluster/proof behavior | reviewed by developers |
| migrated examples | realistic source patterns from existing Mizar-like material | reviewed before acceptance |
| generated `.miz` | broad coverage of grammar and semantic combinations | generated with stored seeds |
| fuzz-minimized reproducers | permanent regression cases from fuzz failures | committed after minimization |
| bug regressions | protect fixed bugs and soundness failures | linked to issue/PR when possible |
| stress/integration articles | large-file and incremental rebuild behavior | reviewed for stability |

## Growth Targets

| Stage | Target |
|---|---|
| early module development | 100-300 `.miz` files |
| evo2 alpha | 300-1,000 `.miz` files |
| evo2 beta / release candidate | 1,000-5,000 `.miz` files |
| mature ecosystem | 500,000-1,000,000 LOC equivalent including generated/fuzz/property corpora |

Growth should favor high-signal fail/soundness tests near the kernel over large numbers of shallow pass tests.

## File Size Guidelines

| Purpose | Lines |
|---|---:|
| parser test | 5-30 |
| type test | 10-50 |
| cluster test | 20-80 |
| theorem test | 30-150 |
| integration test | 100-300 |
| stress test | 500-1,000 |

Validation treats these ranges as upper-bound review gates. Short minimal
regressions are allowed without diagnostics because many fail/soundness cases
are intentionally tiny. Oversized generated `.miz` files are errors unless they
live under `tests/stress/`; oversized handwritten `.miz` files are warnings.

## Generation Policy

Generated `.miz` files must record:

- generator name and version;
- seed;
- generation profile;
- expected outcome;
- minimization status;
- schema version for metadata.

Generated, fuzz, and property sidecars record this provenance in `[origin]`.
Metadata-only handoff anchors may use the crate-local test family or harness
handoff as the generator, `generator_version = "handoff"`, and the fixture's
stable phase/name as the seed. `origin.expected_outcome` mirrors the harness
sidecar outcome. All fuzz seeds record `origin.original_failure_category`; for
promoted fuzz failures, that original fuzz failure class must match the
executable `failure_category`.

Generated tests are committed only when they add coverage, reproduce a bug, or serve as stable stress cases. Bulk generated corpora may live outside the default fast test set until they are minimized or promoted.

## Review Rules

Corpus additions are reviewed for:

- stable expected outcome;
- deterministic diagnostics and snapshots;
- absence of hidden reliance on test execution order;
- minimality for fail/soundness regressions;
- clear domain placement and naming.

Generated sidecars live under `tests/generated/` unless they are stress cases
under `tests/stress/`. Fuzz seeds live under `tests/fuzz/`; property seeds live
under `tests/property/`. Unminimized generated/fuzz/property seeds stay outside
the default `fast` profile. Metadata-only fuzz handoff seeds use the
`fuzz_regression` profile. Stress cases use `profiles = ["stress"]` and do not
also opt into `fast`.

## Tests

Key scenarios:

- generated tests can be reproduced from stored seed metadata;
- minimized fuzz reproducers preserve the original failure category;
- corpus manifest counts pass/fail ratios by domain;
- stress tests are excluded from default fast runs unless requested.

## Constraints and Assumptions

- `.miz` corpus files are long-lived compatibility inputs.
- Fail tests are not loosened to match current compiler behavior.
- A soundness regression case is never deleted without architecture-level review.

## Checker Task 263 Frozen Corpus Increment

After the separate documentation prerequisite, Task 263 may add exactly one
canonical-derived pass source/sidecar pair named
`pass_type_elaboration_structure_definition_payload_001`. Its source is the
frozen 320-byte/final-LF text with SHA-256
`078eaee4b17341c9d8ebeb8a1f631ca984873bd07eb4e5d9c1a9486b39ac6671`.
The sidecar is pass/type_elaboration/type_check with empty public diagnostics
and payloads and one new Task-263 trace reference.

The existing mixed mode/structure-definition failure pair and every parser/
resolver structure fixture remain byte-identical. The docs prerequisite adds
no corpus file; implementation projects cases `425 -> 426` and pass cases
`232 -> 233` only after fresh inventory.

## Checker Task 263 Active Corpus Increment

The sole new pair is
`pass_type_elaboration_structure_definition_payload_001.miz` and its matching
sidecar. The source is exactly 320 bytes/16 lines with SHA-256
`078eaee4b17341c9d8ebeb8a1f631ca984873bd07eb4e5d9c1a9486b39ac6671`;
the sidecar hash is
`d82c8d3102ea34fdb4a32792167c4b109b96b9c05265d3f04e6310278178e8ac`.
All existing `.miz` and expectations remain byte-identical. Cases are
`426`, pass/fail is `233/193`, and active type cases are `203`.

## Task 257C4C0 frozen corpus increment

The [C4C0 contract](../../task_contracts/en/TEST-FRAENKEL-NESTED-CAPTURE-257C4C0.md)
owns one implemented handwritten pass pair under `tests/miz/pass/types/` for
the Chapter 13 nested-comprehension outer-generator capture oracle. The pair
inventory is `344/344`; the new pair remains inactive and earns no
current parser, semantic, route, warning/error, or Task-277B credit. The exact
source bytes/hash and sidecar are owned by the contract; local-lookalike and
builtin-set variants are forbidden.

## Task 257C4C1 explicit-import corpus repair

The [C4C1 contract](../../task_contracts/en/TEST-FRAENKEL-NESTED-CAPTURE-LEXICAL-ADMISSION-257C4C1.md)
modifies that existing oracle in place to the exact 164-byte explicit-import
source and updates only its inactive sidecar note. One separate 140-byte module
at `crates/mizar-test/tests/testdata/parser/nested_capture_fixtures.miz`
defines the ordered `Element`/`NAT` lexical shapes in separate definition
blocks. It is crate-local testdata, not an ordinary corpus payload, and has no
sidecar. Corpus pairs therefore remain `344/344`; active cases, outcomes,
routes, warnings/errors, capture credit, and Task-277B credit do not change.
