# Module-Boundary Refactor Gate: mizar-diagnostics

> Canonical language: English. Japanese companion:
> [../ja/module_boundary_refactor_gate.md](../ja/module_boundary_refactor_gate.md).

## Scope

This task-21 gate audits the source layout after the source/spec and bilingual
documentation audits. It checks for oversized files, mixed responsibilities, and
private helper groups that should be split without changing public APIs,
diagnostic behavior, deterministic rendering, artifact-facing schemas, or
consumer-visible behavior.

## Source Layout Audit

| File or group | Observation | Decision |
|---|---|---|
| `src/registry.rs` | The file mixed public registry API and the large spec-22 built-in descriptor table. | Split the descriptor table and local allocation macro into private `src/registry/builtin.rs`; keep public `BUILTIN_DESCRIPTORS` re-exported from `registry`. |
| `src/failure_record.rs` | The file mixed public record/draft/span/detail types with validation helpers and deterministic debug rendering helpers. | Split validation into private `src/failure_record/validation.rs` and debug rendering into private `src/failure_record/debug.rs`; keep all public record APIs in `failure_record`. |
| `src/explain.rs` | Large but still cohesive around explanation handles, preview bounds, store resolution, canonical keys, and rendering. | No split in this task. A future task may split validation/debug helpers if explanation behavior grows, but no current review-blocking mixed responsibility remains after task 21. |
| `src/fix.rs` | Moderate size and cohesive around structured fix payloads. | No split. |
| `src/aggregator.rs` | Moderate size and cohesive around index construction, ordering, dedup, and stale accounting. | No split. |
| `src/render.rs` | Moderate size and cohesive around CLI projection. | No split. |
| `src/sink.rs` | Small and cohesive around producer collection. | No split. |

## Refactor Result

The task performs behavior-preserving private moves only:

- `registry::BUILTIN_DESCRIPTORS` remains publicly available at the same path.
- `failure_record` public types, constructors, accessors, errors, and debug
  snapshot strings remain unchanged.
- No public module is added from crate root.
- No diagnostic code, message text, ordering rule, deduplication key, render
  output, fix payload, explanation payload, or freshness rule is changed.
- No LSP, driver, artifact, proof, kernel, cache, or producer adoption boundary
  is added.

The source/spec audit scope was re-run for moved APIs: the moved items are
private helpers or re-exported data, so the public API trace remains valid after
updating the source inventory. The bilingual documentation audit scope was
re-run for this task's documentation additions; English and Japanese companions
remain paired.

## Updated Source Inventory

The module table keeps the crate-owned public modules unchanged, while noting
private helper submodules:

- registry: `src/registry.rs`, private `src/registry/builtin.rs`;
- failure records: `src/failure_record.rs`, private
  `src/failure_record/{validation,debug}.rs`;
- sink: `src/sink.rs`;
- aggregator: `src/aggregator.rs`;
- render: `src/render.rs`;
- fix: `src/fix.rs`;
- explain: `src/explain.rs`.

## Verification

Focused verification for the behavior-preserving source move:

```text
cargo fmt --check
cargo test -p mizar-diagnostics
cargo clippy -p mizar-diagnostics --all-targets -- -D warnings
git diff --check
git diff --cached --check
```
