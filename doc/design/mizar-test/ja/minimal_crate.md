# Module: minimal_crate

> Canonical language: English. English canonical version: [../en/minimal_crate.md](../en/minimal_crate.md).

## Purpose

この module は `mizar-test` crate の first implementation scope を固定する。

Minimal crate は compiler を実行せずに test corpus metadata and traceability contracts を validate する。これにより、後続の test-first compiler work が安全になる。Language implementation を trust する前に、test files、expectation sidecars、spec coverage metadata を check できる。

後続の active runner subcommand は、`parse-only` や `declaration-symbol` のような
明示 command を通じてのみ、この first slice を拡張する。metadata `plan` path は、
ここで定義する minimal payload-free contract のままである。

## Scope

First implementation は次を提供する。

- known test roots の deterministic discovery
- `.expect.toml` sidecars の parsing
- `tests/coverage/spec_trace.toml` の parsing
- expectations の schema validation
- sidecars と manifest の traceability validation
- duplicate id detection
- missing source and missing sidecar detection
- deterministic `TestPlan` construction
- CI に適した metadata-only reporting

First implementation と metadata `plan` path は `.miz`、certificate、snapshot、
fuzz、property payloads を実行しない。Active runner subcommand は、対象 stage が
所有する狭い compiler seam を実行してよい。

## Non-Goals

Minimal metadata path は次を実装してはならない。

- explicit active runner subcommand の外にある broad compiler execution
- active runner から owning pipeline seam を呼ぶ以上の lexer or parser behavior
- stage-owned active runner の外にある pass/fail semantic checking
- certificate replay
- kernel checking
- snapshot comparison or update
- fuzz execution
- property generation
- parallel test execution
- cache validation
- `doc/spec/` prose からの coverage inference

これらの features は metadata and traceability contracts が stable になった後、
独自の active gate を持つ explicit runner mode としてのみ追加する。

## Public API

Minimal library exposes:

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

Exact Rust types は evolve してよいが、ownership boundaries は維持する。

- `layout` discovers files and pairs source with sidecar
- `expectation` parses and validates `.expect.toml`
- `traceability` parses and validates `spec_trace.toml`
- `staged_model` owns stage names and ordering
- `harness` builds and reports the metadata-only plan

## CLI

Initial CLI command:

```text
mizar-test plan --tests-root tests --manifest tests/coverage/spec_trace.toml \
  --validation-mode metadata
```

Default validation mode は `metadata` である。CLI は `development` と `release`
も受け付ける。すべての mode は discovery and validation を実行し、deterministic
summary を出力する。

```text
test cases: 0
requirements: 0
errors: 0
warnings: 0
```

Output は filesystems 間で stable でなければならない。First implementation では human-readable output で十分だが、後から JSON reporter を追加できるよう internal result は structured でなければならない。

## Discovery Rules

Discovery は known roots だけを walk する。

```text
tests/miz/
tests/lexical/
tests/certificates/
tests/generated/
tests/fuzz/
tests/property/
tests/snapshots/
```

Missing optional roots は allowed。Unknown roots は `metadata` mode では ignored、
`development` と `release` mode では reported とする。`tests/coverage/` directory
は trace manifest の metadata root である。top level には許可されるが、payload
corpus root としては walk しない。

Files は validation 前に canonical relative path で sort される。Directory iteration order が plan に影響してはならない。

Executable payload extensions は [layout.md](./layout.md) から継承する: `.miz`, `.src`, `.cert.json`, `.fixture.toml`。Every discovered executable payload は同じ directory に adjacent `.expect.toml` を必要とする。`README.md`、`.gitkeep`、snapshot output files のような non-payload files は payload pairing では ignore する。

## Empty Corpus Behavior

Empty corpus は次の場合 valid である。

- `tests/coverage/spec_trace.toml` exists and parses
- malformed sidecars が存在しない
- missing sidecar を必要とする discovered payload が存在しない

Current evo2 repository は empty `.miz` corpus から始めるため、minimal crate は zero tests を harness error ではなく success として扱う。

## Validation Rules

Minimal crate は次を errors として report する。

- unsupported expectation schema version
- expectation `id` not matching sidecar stem
- missing source file
- source stem not matching sidecar stem
- duplicate test ids
- invalid `kind`, `stage`, or `expected_outcome`
- committed executable tests の missing `spec_refs`
- manifest に存在しない `spec_refs`
- manifest requirements with duplicate ids
- manifest requirement `source` files that do not exist
- manifest test paths that do not point back to the requirement id
- invalid stage names
- missing fail expectation identity fields
- strict schema mode での unknown fields

Warnings may be used for:

- requirements with `status = "planned"` and no tests
- optional roots that are missing
- stored coverage status that differs from computed status

Warnings must not hide errors.

Coverage completeness は minimal crate の default `metadata` mode では error ではない。
`development` と `release` selector は strict layout policy を今から exercise できるよう
受け付ける。mode-aware coverage/status gate は traceability/reporting の follow-up work として残り、
[traceability.md](./traceability.md) の rules を使う。

## Determinism

All emitted diagnostics are ordered by:

1. canonical relative path
2. record kind
3. stable diagnostic code
4. stable detail key

`TestPlan` order は expectation sidecar paths の sorted order である。

## Minimal Tests For The Crate

Crate 自身の tests は次を cover する。

- empty corpus succeeds
- malformed TOML fails
- duplicate expectation ids fail
- missing source fails
- missing sidecar for any executable payload fails
- unknown `spec_refs` fail
- manifest duplicate ids fail
- deterministic ordering is stable for shuffled input files

これらの tests は temporary directories を使い、repository corpus contents に依存してはならない。

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

CLI exits:

| Code | Meaning |
|---:|---|
| 0 | Validation succeeded. |
| 1 | Validation errors were found. |
| 2 | Harness infrastructure error, such as unreadable root path. |

Warnings alone do not produce a non-zero exit code.

## Constraints And Assumptions

- Minimal crate は untrusted test infrastructure であり proof authority ではない。
- Compiler behavior から expectations を infer してはならない。
- Corpus files を mutate してはならない。
- Compiler crate が存在する前から useful でなければならない。
- Empty corpus path は boring and successful に保つ。
