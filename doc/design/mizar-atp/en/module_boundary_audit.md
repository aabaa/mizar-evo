# Module-Boundary Audit: mizar-atp

> Canonical language: English. Japanese companion:
> [../ja/module_boundary_audit.md](../ja/module_boundary_audit.md).

## Task 27 Module-Boundary Refactor Gate

Task 27 audits the `mizar-atp` source layout after the source/spec and
bilingual documentation audits. The refactor is a private test module split:
it moves large inline unit-test suites out of production module files while
preserving every public API, diagnostic, deterministic rendering,
artifact-facing schema, candidate-evidence shape, and trust-boundary rule.

No public API change is introduced. No production behavior is changed. The
crate remains an untrusted formula/substitution evidence candidate producer and
does not add kernel checking, proof policy, witness publication, proof-cache
promotion, backend proof material, resolution-trace acceptance, or SAT problem
payload trust.

## Method

The audit reviewed:

- the module table in [todo.md](./todo.md);
- the documented ownership rules in
  [internal 07](../../internal/en/07.crate_module_layout.md);
- the paired module specifications for all exported modules;
- the production source files and their inline test sizes;
- the existing lint-policy public API allowlists and crate-file guards.

Oversized production module files were classified as a review-bottleneck layout
issue caused by inline unit tests, not as source/spec behavior drift. No mixed
production responsibility requiring a new public module was found. Production
helper extraction remains deferred until a paired module spec defines a
concrete helper boundary.

## Layout Result

The following private `cfg(test)` child modules now own the unit-test suites:

| Public module | Production source | Private test module | Test gate | Result |
|---|---|---|---|---|
| `backend` | `src/backend.rs` | `src/backend/tests.rs` | `cfg(all(test, unix))` | private test module split |
| `portfolio` | `src/portfolio.rs` | `src/portfolio/tests.rs` | `cfg(test)` | private test module split |
| `problem` | `src/problem.rs` | `src/problem/tests.rs` | `cfg(test)` | private test module split |
| `property_encoding` | `src/property_encoding.rs` | `src/property_encoding/tests.rs` | `cfg(test)` | private test module split |
| `smtlib_encoder` | `src/smtlib_encoder.rs` | `src/smtlib_encoder/tests.rs` | `cfg(test)` | private test module split |
| `tptp_encoder` | `src/tptp_encoder.rs` | `src/tptp_encoder/tests.rs` | `cfg(test)` | private test module split |
| `translator` | `src/translator.rs` | `src/translator/tests.rs` | `cfg(test)` | private test module split |

The production modules still match the existing module specs one-to-one:
`backend`, `portfolio`, `problem`, `property_encoding`, `smtlib_encoder`,
`tptp_encoder`, and `translator`. The private test modules are not exported by
`src/lib.rs`, do not define public API, and remain implementation-local test
fixtures for the owning module.

## Classification

No new `spec_gap`, `test_gap`, `design_drift`, `source_drift`,
`source_undocumented_behavior`, `test_expectation_drift`,
`boundary_violation`, `repo_metadata_conflict`, or bilingual drift was found.
No new ATP-AUDIT gap is required.

Remaining external/deferred follow-ups are unchanged from
[source_spec_audit.md](./source_spec_audit.md): real backend output extraction,
active source-derived corpus execution, downstream proof/cache/artifact
integration, typed/native encoder extensions, and proof-policy finality remain
outside this behavior-preserving layout refactor.

## Verification Expectations

Task 27 must keep the existing behavior tests passing after the split:

- `cargo test -p mizar-atp`
- `cargo clippy -p mizar-atp --all-targets --all-features -- -D warnings`
- `cargo fmt --check`

The lint-policy guard additionally checks that the source tree contains only
the documented production modules plus the private test module split, and that
the paired source/spec and bilingual audits record this task.
