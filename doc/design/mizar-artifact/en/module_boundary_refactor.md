# mizar-artifact Module-Boundary Refactor Gate

> Canonical language: English. Japanese companion:
> [../ja/module_boundary_refactor.md](../ja/module_boundary_refactor.md).

Status: task 22 gate complete.

## Scope

This gate audits the `mizar-artifact` source layout after tasks 1-21. It looks
for oversized files, mixed responsibilities, and private helpers that should be
split along the module table and module-spec boundaries.

The refactor is behavior-preserving. It does not change public module names,
public types, public functions, schema field names, canonical JSON ordering,
hash inputs, diagnostics, path validation, manifest transaction behavior, or
artifact-facing schemas.

## Result

- Public module paths remain `mizar_artifact::{store, module_summary,
  registration_summary, proof_witness, verified_artifact, manifest}`.
- Inline unit-test blocks moved to per-module private `tests.rs` files:
  `store/tests.rs`, `module_summary/tests.rs`, `registration_summary/tests.rs`,
  `proof_witness/tests.rs`, `verified_artifact/tests.rs`, and
  `manifest/tests.rs`.
- Production roots remain intentionally aligned with the existing module specs.
  The largest post-split roots are `verified_artifact.rs` (about 3.3k lines),
  `registration_summary.rs` (about 2.5k lines), and `manifest.rs` (about
  2.3k lines). They are large because each owns one stable schema boundary with
  validation, canonical JSON construction, hash participation, and reader error
  reporting that must be reviewed together.
- No additional production helper split was made because the candidate helper
  blocks are schema-coupled. Moving them now would increase visibility between
  private modules without reducing a mixed-responsibility boundary.
- The task-19 public-enum guard now scans source files recursively while
  excluding test-only `tests.rs` files, so future private production modules are
  not missed by the policy check.
- Source/spec correspondence and bilingual documentation synchronization scopes
  were re-run for the moved tests and this audit file; no new drift was found.

## Source Layout

| Public module | Public source | Private tests after task 22 | Gate result |
|---|---|---|---|
| `store` | `src/store.rs` | `src/store/tests.rs` | Canonical JSON, hash framing, path safety, and store I/O APIs stayed in the module root; tests moved out of the implementation body. |
| `module_summary` | `src/module_summary.rs` | `src/module_summary/tests.rs` | Module-summary schema, canonical writer/reader, and interface-hash helpers stayed in the module root; tests moved out. |
| `registration_summary` | `src/registration_summary.rs` | `src/registration_summary/tests.rs` | Registration-summary schema, trace-reference validation, and registration-interface hash helpers stayed in the module root; tests moved out. |
| `proof_witness` | `src/proof_witness.rs` | `src/proof_witness/tests.rs` | Proof-witness reference schema and validation stayed in the module root; tests moved out. |
| `verified_artifact` | `src/verified_artifact.rs` | `src/verified_artifact/tests.rs` | Verified-artifact schema, provenance, hash-input helpers, witness validation, and reader/writer rules stayed in the module root; tests moved out. |
| `manifest` | `src/manifest.rs` | `src/manifest/tests.rs` | Manifest schema, file I/O, transaction writer, and reference validation stayed in the module root; tests moved out. |

## Re-Run Audits

- Source/spec correspondence: public API source roots still match the module
  specs. The task-22 row in
  [source_spec_correspondence.md](./source_spec_correspondence.md) records the
  private test split and confirms no public API, behavior, diagnostic,
  rendering, artifact-schema, or boundary drift.
- Bilingual documentation synchronization: this file was added in both language
  directories, and [bilingual_documentation_sync.md](./bilingual_documentation_sync.md)
  now includes the new pair.
- Boundary discipline: the split is local to `mizar-artifact`; it does not add
  raw IR ownership, cache-record ownership, scheduler state, proof authority, or
  kernel acceptance behavior.

## Verification

The gate requires both the focused artifact verification and the broader
workspace checks because task 22 touches every source module in the crate:

```text
cargo fmt --check
cargo test -p mizar-artifact
cargo clippy -p mizar-artifact --all-targets -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
