# Module: minimal_crate

> Canonical language: English. Japanese companion: [../ja/minimal_crate.md](../ja/minimal_crate.md).

## Purpose

This module fixes the first implementation scope for the `mizar-test` crate.

The minimal crate validates the test corpus metadata and traceability contracts
without running the compiler. It exists to make later test-first compiler work
safe: test files, expectation sidecars, and spec coverage metadata can be
checked before any language implementation is trusted.

Later active runner subcommands extend this first slice only through explicit
commands such as `parse-only` and `declaration-symbol`. The metadata `plan`
path remains the minimal payload-free contract described here.

## Scope

The first implementation provides:

- deterministic discovery of known test roots;
- parsing of `.expect.toml` sidecars;
- parsing of `tests/coverage/spec_trace.toml`;
- schema validation for expectations;
- traceability validation between sidecars and the manifest;
- duplicate id detection;
- missing source and missing sidecar detection;
- deterministic `TestPlan` construction;
- metadata-only reporting suitable for CI.

The first implementation and the metadata `plan` path do not execute `.miz`,
certificate, snapshot, fuzz, or property payloads. Active runner subcommands
may execute the narrow compiler seams owned by their stage.

## Non-Goals

The minimal metadata path must not implement:

- broad compiler execution outside explicit active runner subcommands;
- lexer or parser behavior beyond calling the owning pipeline seam from an
  active runner;
- pass/fail semantic checking outside stage-owned active runners;
- certificate replay;
- kernel checking;
- snapshot comparison or update;
- fuzz execution;
- property generation;
- parallel test execution;
- cache validation;
- coverage inference from `doc/spec/` prose.

These features are added only after the metadata and traceability contracts are
stable, and only through explicit runner modes with their own active gates.

## Public API

The minimal library exposes:

```rust
pub struct DiscoveryConfig {
    pub workspace_root: Utf8PathBuf,
    pub tests_root: Utf8PathBuf,
    pub manifest_path: Utf8PathBuf,
    pub profile: TestProfile,
    pub validation_mode: ValidationMode,
}

pub struct TestPlan {
    pub cases: Vec<TestCase>,
    pub manifest: TraceManifest,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub struct TestCase {
    pub id: TestCaseId,
    pub source_path: Utf8PathBuf,
    pub expectation_path: Utf8PathBuf,
    pub expectation: Expectation,
}

pub struct Expectation {
    pub schema_version: u32,
    pub id: TestCaseId,
    pub kind: TestKind,
    pub stage: Stage,
    pub domain: String,
    pub source: Utf8PathBuf,
    pub expected_outcome: ExpectedOutcome,
    pub spec_refs: Vec<SpecRequirementId>,
    pub profiles: Vec<String>,
    pub notes: Option<String>,
    pub ast_profile: Option<String>,
    pub snapshot_profiles: Vec<String>,
}

pub struct TraceManifest {
    pub requirements: Vec<SpecRequirement>,
}
```

The exact Rust types may evolve, but the ownership boundaries must remain:

- `layout` discovers files and pairs source with sidecar;
- `expectation` parses and validates `.expect.toml`;
- `traceability` parses and validates `spec_trace.toml`;
- `staged_model` owns stage names and ordering;
- `harness` builds and reports the metadata-only plan.

## CLI

The initial CLI command is:

```text
mizar-test plan --tests-root tests --manifest tests/coverage/spec_trace.toml \
  --validation-mode metadata
```

The default validation mode is `metadata`; the CLI also accepts `development`
and `release`. All modes perform discovery and validation, then print a
deterministic summary:

```text
test cases: 0
requirements: 0
errors: 0
warnings: 0
```

Output must be stable across filesystems. Human-readable output is sufficient
for the first implementation, but the internal result must be structured so a
JSON reporter can be added later.

## Discovery Rules

Discovery walks only known roots:

```text
tests/miz/
tests/lexical/
tests/certificates/
tests/generated/
tests/fuzz/
tests/property/
tests/snapshots/
```

Missing optional roots are allowed. Unknown roots are ignored in `metadata`
mode and reported in `development` and `release` mode. The `tests/coverage/`
directory is a metadata root for the trace manifest; it is allowed at the top
level but is not walked as a payload corpus root.

Files are sorted by canonical relative path before validation. Directory
iteration order must never affect the plan.

Executable payload extensions are inherited from [layout.md](./layout.md):
`.miz`, `.src`, `.cert.json`, and `.fixture.toml`. Every discovered executable
payload requires an adjacent `.expect.toml` in the same directory. Non-payload
files such as `README.md`, `.gitkeep`, and snapshot output files are ignored by
payload pairing.

## Empty Corpus Behavior

An empty corpus is valid when:

- `tests/coverage/spec_trace.toml` exists and parses;
- no malformed sidecars are present;
- no discovered payload requires a missing sidecar.

The current evo2 repository starts from an empty `.miz` corpus, so the minimal
crate must treat zero tests as success rather than as a harness error.

## Validation Rules

The minimal crate reports errors for:

- unsupported expectation schema version;
- expectation `id` not matching sidecar stem;
- missing source file;
- source stem not matching sidecar stem;
- duplicate test ids;
- invalid `kind`, `stage`, or `expected_outcome`;
- missing `spec_refs` in committed executable tests;
- `spec_refs` that do not exist in the manifest;
- manifest requirements with duplicate ids;
- manifest requirement `source` files that do not exist;
- manifest test paths that do not point back to the requirement id;
- invalid stage names;
- missing fail expectation identity fields;
- unknown fields in strict schema mode.

Warnings may be used for:

- requirements with `status = "planned"` and no tests;
- optional roots that are missing;
- stored coverage status that differs from computed status.

Warnings must not hide errors.

Coverage completeness is not an error in the minimal crate's default
`metadata` mode. The `development` and `release` selectors are accepted so the
strict layout policy can be exercised now; mode-aware coverage/status gates
remain traceability/reporting follow-up work and use the rules in
[traceability.md](./traceability.md).

## Determinism

All emitted diagnostics are ordered by:

1. canonical relative path;
2. record kind;
3. stable diagnostic code;
4. stable detail key.

The `TestPlan` order is the sorted order of expectation sidecar paths.

## Minimal Tests For The Crate

The crate's own tests cover:

- empty corpus succeeds;
- malformed TOML fails;
- duplicate expectation ids fail;
- missing source fails;
- missing sidecar for any executable payload fails;
- unknown `spec_refs` fail;
- manifest duplicate ids fail;
- deterministic ordering is stable for shuffled input files.

These tests should use temporary directories and should not depend on the
repository corpus contents.

## Implementation Order

1. Create the crate skeleton.
2. Implement enums for stage, kind, outcome, and validation severity.
3. Implement TOML parsing for `spec_trace.toml`.
4. Implement TOML parsing for `.expect.toml`.
5. Implement deterministic discovery and pairing.
6. Implement validation and diagnostics.
7. Add CLI summary output.
8. Add crate-local tests for the metadata-only behavior.

## Exit Codes

The CLI exits:

| Code | Meaning |
|---:|---|
| 0 | Validation succeeded. |
| 1 | Validation errors were found. |
| 2 | Harness infrastructure error, such as unreadable root path. |

Warnings alone do not produce a non-zero exit code.

## Constraints And Assumptions

- The minimal crate is untrusted test infrastructure, not proof authority.
- It must not infer expectations from compiler behavior.
- It must not mutate corpus files.
- It must be useful before any compiler crate exists.
- It must keep the empty corpus path boring and successful.
