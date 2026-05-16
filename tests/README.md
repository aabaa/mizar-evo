# Mizar Evo Test Corpus

This directory contains the evo2 test corpus.

The corpus is an implementation asset. Expectations are explicit sidecars and
must not be inferred from the current compiler behavior.

## Strategy

Committed `.miz` tests are added in specification order, following the
structure of `doc/spec/`.

This is a V-model contract:

- each language specification section owns corresponding pass/fail tests;
- a test is added only after the referenced syntax and semantic prerequisites
  are clear in `doc/spec/`;
- the expected failure phase must match the earliest phase that can soundly
  reject the input;
- tests must not rely on undefined library symbols unless the test explicitly
  targets name resolution;
- higher-level tests, such as overload, substitution, verification, and kernel
  soundness cases, are added only after their lower-level syntax and name
  dependencies have their own tests.

The previous draft `.miz` seed corpus has been removed. New corpus additions
should start from the earliest applicable spec chapter and grow forward.

## Layout

- `miz/pass/` contains `.miz` sources that are expected to compile and verify.
- `miz/fail/` contains `.miz` sources that must be rejected.
- `certificates/` contains certificate payload tests independent of `.miz`
  parsing.
- `generated/`, `fuzz/`, and `property/` are reserved for generated,
  minimized fuzz, and property-test seeds.
- `snapshots/` is reserved for deterministic snapshot baselines.

## Naming

Names are stable snake_case:

```text
pass_parser_empty_definition_001.miz
fail_soundness_false_arithmetic_001.miz
```

The prefix records the high-level outcome, the middle records the semantic
domain, and the numeric suffix is never reused for unrelated cases.

## Sidecars

Every committed `.miz` seed has an adjacent `.expect.toml`.

Required fields:

```toml
schema_version = 1
id = "fail_soundness_false_arithmetic_001"
kind = "fail"
domain = "soundness"
source = "fail_soundness_false_arithmetic_001.miz"
expected_outcome = "fail"
expected_phase = "verification"
failure_category = "kernel_rejection"
rejection_reason = "invalid_sat_proof"
diagnostic_codes = ["E-KERNEL-INVALID-PROOF"]
stable_detail_key = "soundness.false_arithmetic.one_eq_zero"
```

Pass cases use `expected_outcome = "pass"` and may use an empty
`diagnostic_codes` list.
