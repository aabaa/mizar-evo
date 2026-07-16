# Module Specifications: mizar-test

> Canonical language: English. English canonical version: [../en/README.md](../en/README.md).

`mizar-test` は、test-first style で Mizar Evo を開発するための test corpus layout、`.miz` corpus strategy、fail/soundness contracts、snapshot format、test harness behavior を所有する。

compiler semantics は所有しない。implementation crates が deterministic pass/fail、failure category、rejection reason、snapshot expectations に基づいて実装できるよう、expected behavior を符号化する。

## Context

- [doc/design/architecture/ja/00.pipeline_overview.md](../../architecture/ja/00.pipeline_overview.md) — evo2 correctness principles
- [doc/design/architecture/ja/15.kernel_certificate_format.md](../../architecture/ja/15.kernel_certificate_format.md) — certificate rejection reasons
- [doc/design/architecture/ja/16.substitution_and_binding.md](../../architecture/ja/16.substitution_and_binding.md) — substitution and binder failure cases
- [doc/design/architecture/ja/17.cluster_trace_format.md](../../architecture/ja/17.cluster_trace_format.md) — cluster loop and trace cases
- [doc/design/architecture/ja/18.dependency_fingerprint.md](../../architecture/ja/18.dependency_fingerprint.md) — dependency slices and rebuild triggers
- [doc/design/architecture/ja/19.failure_semantics.md](../../architecture/ja/19.failure_semantics.md) — failure categories and stable error ordering
- [doc/design/architecture/ja/20.test_strategy.md](../../architecture/ja/20.test_strategy.md) — overall evo2 testing strategy
- [doc/design/internal/ja/07.crate_module_layout.md](../../internal/ja/07.crate_module_layout.md) — crate/module ownership map

## Index

| Document | Maps To | Description | Status |
|---|---|---|---|
| [layout.md](./layout.md) | `crates/mizar-test/src/layout.rs` | test directory layout、case metadata、naming rules、expected result files | Draft |
| [expectation_schema.md](./expectation_schema.md) | `crates/mizar-test/src/expectation.rs` | `.expect.toml` sidecar schema、outcome contracts、validation rules | Draft |
| [staged_model.md](./staged_model.md) | `crates/mizar-test/src/staged_model.rs` | pipeline-first staged test admission model and spec coverage mapping | Draft |
| [traceability.md](./traceability.md) | `crates/mizar-test/src/traceability.rs` | spec-to-test manifest schema、bidirectional validation、coverage reporting | Draft |
| [minimal_crate.md](./minimal_crate.md) | `crates/mizar-test` | metadata-only discovery and validation の first implementation scope | Draft |
| [miz_corpus.md](./miz_corpus.md) | `crates/mizar-test/src/layout.rs`, `crates/mizar-test/src/expectation.rs`, `crates/mizar-test/src/harness.rs` | `.miz` corpus classes、growth targets、generation policy、review rules | Draft |
| [fail_soundness.md](./fail_soundness.md) | `crates/mizar-test/src/expectation.rs`, `crates/mizar-test/src/harness.rs` | pass してはならない negative tests、expected failure categories、rejection reasons | Draft |
| [snapshot.md](./snapshot.md) | `crates/mizar-test/src/snapshot.rs` | snapshot artifact format、canonical hashing、update policy | Draft |
| [harness.md](./harness.md) | `crates/mizar-test/src/harness.rs` | pass/fail runner、snapshot runner、determinism checks、reporting | Draft |
| [bilingual_sync_audit.md](./bilingual_sync_audit.md) | `doc/design/mizar-test/en`, `doc/design/mizar-test/ja` | task 13 EN/JA synchronization baseline と task 15 architecture-22 follow-up audit | Draft |

## Crate Boundary

`mizar-test` は次を提供する。

- test cases and `.miz` corpora の discovery
- pass/fail expectations の validation
- snapshot read/write/update tooling
- deterministic repeated-run checks
- sequential-vs-parallel equivalence checks
- fuzz/property minimization から committed regression tests への handoff

次を行ってはならない。

- compiler failures を test pass に弱める
- kernel evidence なしに ATP success を trust する
- nondeterministic snapshot hashes を受理する
- current compiler behavior から missing expected failures を推測する
- explicit snapshot update mode 以外で compiler output を mutate する
