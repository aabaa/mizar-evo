# Shared pipeline labels

Canonical language: English; [Japanese companion](../ja/pipeline.md).

`src/pipeline.rs` owns `PipelinePhase`, re-exported by Build's task graph.
Its non-exhaustive scheduler vocabulary preserves the existing variant order,
`as_str()` identity spelling and `as_cli_str()` snake-case spelling.
Cache and IR keep open string labels; their `From<PipelinePhase>` conversions
use the identity spelling. Diagnostic attribution and test expectation phases
remain separate vocabularies.

`module_work_unit_label(package, module)` returns the existing colon-separated
Rust-debug string pair, including quoting and escaping. It does not construct
Build work units or change specialized task identities. Session owns neither
scheduling nor phase semantics. Literal compatibility vectors live in
`crates/mizar-session/tests/common_utilities.rs`.
