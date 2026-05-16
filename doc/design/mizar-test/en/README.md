# Module Specifications: mizar-test

> Canonical language: English. Japanese companion: [../ja/README.md](../ja/README.md).

`mizar-test` owns the test corpus layout, `.miz` corpus strategy, fail/soundness contracts, snapshot format, and test harness behavior used to develop Mizar Evo in a test-first style.

It does not own compiler semantics. It encodes expected behavior so implementation crates can be built against deterministic pass/fail, failure category, rejection reason, and snapshot expectations.

## Context

- [doc/design/architecture/en/00.pipeline_overview.md](../../architecture/en/00.pipeline_overview.md) - evo2 correctness principles
- [doc/design/architecture/en/15.kernel_certificate_format.md](../../architecture/en/15.kernel_certificate_format.md) - certificate rejection reasons
- [doc/design/architecture/en/16.substitution_and_binding.md](../../architecture/en/16.substitution_and_binding.md) - substitution and binder failure cases
- [doc/design/architecture/en/17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md) - cluster loop and trace cases
- [doc/design/architecture/en/18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md) - dependency slices and rebuild triggers
- [doc/design/architecture/en/19.failure_semantics.md](../../architecture/en/19.failure_semantics.md) - failure categories and stable error ordering
- [doc/design/architecture/en/20.test_strategy.md](../../architecture/en/20.test_strategy.md) - overall evo2 testing strategy
- [doc/design/internal/en/07.crate_module_layout.md](../../internal/en/07.crate_module_layout.md) - crate/module ownership map

## Index

| Document | Maps To | Description | Status |
|---|---|---|---|
| [layout.md](./layout.md) | `crates/mizar-test/src/layout.rs` | Test directory layout, case metadata, naming rules, and expected result files | Draft |
| [expectation_schema.md](./expectation_schema.md) | `crates/mizar-test/src/expectation.rs` | `.expect.toml` sidecar schema, outcome contracts, and validation rules | Draft |
| [staged_model.md](./staged_model.md) | `crates/mizar-test/src/staged_model.rs` | Pipeline-first staged test admission model and spec coverage mapping | Draft |
| [traceability.md](./traceability.md) | `crates/mizar-test/src/traceability.rs` | Spec-to-test manifest schema, bidirectional validation, and coverage reporting | Draft |
| [miz_corpus.md](./miz_corpus.md) | `crates/mizar-test/src/corpus.rs` | `.miz` corpus classes, growth targets, generation policy, and review rules | Draft |
| [fail_soundness.md](./fail_soundness.md) | `crates/mizar-test/src/fail_soundness.rs` | Negative tests that must not pass, expected failure categories, and rejection reasons | Draft |
| [snapshot.md](./snapshot.md) | `crates/mizar-test/src/snapshot.rs` | Snapshot artifact format, canonical hashing, and update policy | Draft |
| [harness.md](./harness.md) | `crates/mizar-test/src/harness.rs` | Pass/fail runner, snapshot runner, determinism checks, and reporting | Draft |

## Crate Boundary

`mizar-test` provides:

- discovery of test cases and `.miz` corpora;
- validation of pass/fail expectations;
- snapshot read/write/update tooling;
- deterministic repeated-run checks;
- sequential-vs-parallel equivalence checks;
- fuzz/property minimization handoff into committed regression tests.

It must not:

- weaken compiler failures into test passes;
- trust ATP success without kernel evidence;
- accept nondeterministic snapshot hashes;
- infer missing expected failures from current compiler behavior;
- mutate compiler output outside explicit snapshot update mode.
