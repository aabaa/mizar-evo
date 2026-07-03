# Module: harness

> Canonical language: English. Japanese companion: [../ja/harness.md](../ja/harness.md).

## Purpose

This module defines the test harness that discovers cases, runs compiler profiles, checks expectations, and reports deterministic results.

## Public API

```rust
pub struct DiscoveryConfig {
    pub workspace_root: PathBuf,
    pub tests_root: PathBuf,
    pub manifest_path: PathBuf,
    pub profile: TestProfile,
    pub validation_mode: ValidationMode,
}

pub struct TestPlan {
    pub cases: Vec<TestCase>,
    pub manifest: TraceManifest,
    pub coverage_report: CoverageReport,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub struct CoverageReport {
    pub requirements: Vec<RequirementCoverage>,
    pub stages: Vec<StageCoverage>,
    pub pass_fail_mix: PassFailMix,
    pub architecture22_matrix: Architecture22MatrixReport,
}

pub struct RequirementCoverage {
    pub id: SpecRequirementId,
    pub stage: Stage,
    pub coverage: CoverageShape,
    pub required: bool,
    pub stored_status: RequirementStatus,
    pub computed_status: RequirementStatus,
    pub evidence: CoverageEvidenceSummary,
    pub missing_shapes: Vec<CoverageShape>,
}

pub struct StageCoverage {
    pub stage: Stage,
    pub requirements: usize,
    pub covered: usize,
    pub partial: usize,
    pub planned: usize,
    pub deferred: usize,
    pub obsolete: usize,
    pub missing_shapes: usize,
}

pub struct PassFailMix {
    pub pass: usize,
    pub fail: usize,
    pub total: usize,
    pub target_pass_percent: u8,
    pub target_fail_percent: u8,
}

pub struct Architecture22MatrixReport {
    pub scenarios: Vec<Architecture22ScenarioReport>,
    pub missing_scenarios: Vec<String>,
}

pub struct Architecture22ScenarioReport {
    pub scenario_id: String,
    pub equivalence_class: String,
    pub planned: usize,
    pub active: usize,
}

pub struct TestCase {
    pub id: TestCaseId,
    pub source_path: PathBuf,
    pub expectation_path: PathBuf,
    pub expectation: Expectation,
}

#[non_exhaustive]
pub enum TestProfile {
    Fast,
    Full,
    Stress,
    FuzzRegression,
    SnapshotUpdate,
}

#[non_exhaustive]
pub enum ValidationMode {
    Metadata,
    Development,
    Release,
}

pub struct ParseOnlyRunReport {
    pub results: Vec<ParseOnlyCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub struct DeclarationSymbolRunReport {
    pub results: Vec<DeclarationSymbolCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub struct TypeElaborationRunReport {
    pub results: Vec<TypeElaborationCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}
```

The generic `TestOutcome`/snapshot-reporting surface is future API. Current
active runners expose stage-specific report records while sharing the metadata
plan and validation diagnostics shown above.

## Public Enum Forward Compatibility

Task 12 applies the `mizar-frontend` task-25 procedure to the harness-facing
enum surface. These enums are downstream API and must remain
`#[non_exhaustive]`; downstream callers must keep wildcard match arms, while
`mizar-test` may keep crate-internal matches exhaustive for the currently known
variants.

| Public enum | Owner | Decision |
|---|---|---|
| `ValidationSeverity` | `diagnostic` reporting used by harness plans and runner reports | `#[non_exhaustive]` downstream forward-compatible surface. |
| `TestProfile` | `harness` profile selection | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ValidationMode` | `harness` validation strictness | `#[non_exhaustive]` downstream forward-compatible surface. |
| `HarnessError` | `harness` infrastructure failure boundary | `#[non_exhaustive]` downstream forward-compatible surface. |
| `ParseOnlyCaseStatus` | `runner` parse-only report status | `#[non_exhaustive]` downstream forward-compatible surface. |
| `DeclarationSymbolCaseStatus` | `runner` declaration-symbol report status | `#[non_exhaustive]` downstream forward-compatible surface. |
| `TypeElaborationCaseStatus` | `runner` type-elaboration report status | `#[non_exhaustive]` downstream forward-compatible surface. |

No exhaustive public enum exceptions are owned by this module.

## Runner Modes

| Mode | Behavior |
|---|---|
| metadata plan | discover sidecars and validate layout, expectation schema, and traceability without executing payloads |
| parse-only | run active `.miz` parse-only cases through `mizar-frontend` and `MizarParserSeam` |
| declaration-symbol | run active `.miz` declaration-symbol cases through frontend parsing and resolver declaration/symbol collection |
| type-elaboration | run active `.miz` type-elaboration cases through frontend parsing and resolver declaration/symbol collection, execute the supported source-derived builtin type-expression bridge through checker-owned `ResolvedTypedAst`, and surface unsupported checker payload families as stable external dependency gaps |
| pass/fail | run `.miz` cases and match expected outcome |
| snapshot | compare canonical snapshot hashes |
| determinism | repeat runs and compare artifacts, diagnostics, and hashes |
| parallel-equivalence | compare sequential and parallel outputs |
| fuzz-regression | run minimized fuzz cases as ordinary committed tests |
| update | rewrite snapshots only when explicitly requested |

## Consumer Runner Pacing

Task 10 keeps runner support synchronized with consumer crates one increment at
a time. Prepared increments are implemented and verified; unprepared consumers
stay `paced/open` without placeholder runner modes, fake active fixtures, or
fabricated coverage.

| Consumer task | Stage / runner | mizar-test status | Next condition |
|---|---|---|---|
| `mizar-parser` task 3 | `parse_only` / `parse-only` | prepared/implemented; active `.miz` pass/fail sidecars use `active_parse_only`, and untagged parse-only metadata stays planned | Keep the transitional `SurfaceAst` snapshot shortcut until the general snapshot runner lands. |
| `mizar-resolve` task 23 | `declaration_symbol` / `declaration-symbol` | prepared/implemented; active sidecars use `active_declaration_symbol`, public resolver diagnostic-code matching remains gated | Open public diagnostic-code assertions only after resolver diagnostic ranges are specified. |
| `mizar-checker` task 12 plus task 16 source bridge and Campaign II task 17 | `type_elaboration` / `type-elaboration` | prepared/implemented; active sidecars use `active_type_elaboration`, lower stages run first, reserve-only builtin `set`/`object` type expressions are normalized through `mizar-checker`, assembled into `TypedAst`, projected to checker-owned `ResolvedTypedAst`, and unsupported checker payload families stay on `type_elaboration.external_dependency.ast_payload_extraction` | Broader type pass/fail semantic assertions wait for AST-wide source-to-checker payload extraction. |
| `mizar-checker` task 29 | `formula_statement` / `advanced_semantics` | paced/open; trace rows are deferred and no active fixture is fabricated | Add runner support only after statement/formula and advanced-semantics source payload seams exist. |
| `mizar-vc` task 15 | `proof_verification` | paced/open; VC/proof-verification obligations are deferred | Add runner support only after source-to-core/source-to-VC extraction and downstream verification seams exist. |
| `mizar-atp` task 20 | `advanced_semantics` metadata handoff | paced/open in `mizar-test`; metadata-only property fixtures may be consumed by `mizar-atp` Rust tests | Add active `.miz` ATP runner support only after source-derived ATP extraction and proof-policy/kernel handoff seams exist. |
| `mizar-kernel` task 17 | proof/certificate/kernel evidence | paced/open; fail/soundness metadata is validated without active proof/certificate/kernel execution | Add runner support only after source-to-evidence or certificate execution seams exist. |

## Algorithm / Logic

1. Discover tests through `layout` under the known payload roots
   `miz`, `lexical`, `certificates`, `generated`, `fuzz`, `property`,
   `stress`, and `snapshots`.
2. Parse and validate every discovered sidecar, then build a canonical
   `TestPlan` whose returned `cases` are filtered by `DiscoveryConfig.profile`.
   Missing `profiles` defaults to `["fast"]`; `Full` includes every valid
   parsed case. Duplicate ids, traceability links, and diagnostics are checked
   across all parsed sidecars, not only the filtered cases.
3. For `parse-only`, select only cases with `stage = "parse_only"`,
   `expected_phase = "parse"`, `.miz` payloads, pass/fail outcomes, and
   `tags = ["active_parse_only"]`. Untagged parse-only sidecars remain
   discovery and traceability metadata.
4. For `declaration-symbol`, select only cases with
   `stage = "declaration_symbol"`, `expected_phase = "resolve"`, `.miz`
   payloads, pass/fail outcomes, and `tags = ["active_declaration_symbol"]`.
   Untagged declaration-symbol sidecars remain discovery and traceability
   metadata.
5. For `type-elaboration`, select only cases with
   `stage = "type_elaboration"`, `expected_phase = "type_check"`, `.miz`
   payloads, pass/fail outcomes, and `tags = ["active_type_elaboration"]`.
   Untagged type-elaboration sidecars remain discovery and traceability
   metadata.
6. Run cases in deterministic display order, even when execution is parallel.
7. Capture compiler outputs as structured records.
8. Match pass/fail expectations before snapshot expectations.
9. Compare general `[[snapshots]]` entries by canonical hash; the current
   parse-only `SurfaceAst` shortcut compares committed text baselines
   byte-for-byte as described below.
10. Report failures with phase, failure category, rejection reason, diagnostic code, and snapshot diff summary.

The current parse-only runner copies each active corpus file into a temporary
`src/` package, runs the real frontend parser seam, requires pass cases to
produce an AST with no assertion diagnostics, and compares fail cases against
the expected bare syntax diagnostic keys. For this syntax-only mode, the runner
uses a harness provider that resolves every frontend import stub to a
`ResolvedImportEntry` with matching `stub_ordinal` and `stub_span`, plus one
`ModuleLexicalSummary` per distinct module id. Summaries contain no exported
symbols except for the narrow `parser.type_fixtures` fixture module, which
injects parser-owned attribute, mode, structure, predicate, and functor shapes
needed by type-expression and operator syntax fixtures. No other import summary
exports symbols; the summaries exist only to keep import syntax cases from
depending on semantic module availability. If parser syntax diagnostics and non-syntax
frontend recovery diagnostics both appear, the runner reports all diagnostic
codes unless the sidecar explicitly includes
`allow_frontend_recovery_diagnostics`. Active parse-only pass/fail sidecars may
also set the transitional `snapshots = "snapshots/parser/<id>.surface_ast.snap"`
field. For those cases, after diagnostics match, the runner requires a
`SurfaceAst` and compares `SurfaceAst::snapshot_text()` with the committed
baseline under `tests/snapshots/`. Snapshot baselines are never rewritten during
normal parse-only runs.

An expectation tagged `active_parse_only` but missing one of the runnable case
predicates is a harness error rather than a silent skip.

The current declaration-symbol runner copies each active `.miz` corpus file
into the same temporary package shape, runs the real frontend, then feeds the
resulting `SurfaceAst` through the resolver declaration-shell collector,
parser-backed signature projection extractor, and symbol collector. Pass cases
require no frontend assertion diagnostics and no resolver symbol diagnostics.
Fail cases compare the resolver's crate-local internal detail keys against
`diagnostic_payloads` when present, or `stable_detail_key` otherwise. The runner
does not require or invent public resolver diagnostic codes while the
diagnostic-code ownership gap remains open; active declaration-symbol
expectations with non-empty `diagnostic_codes` are harness errors.

An expectation tagged `active_declaration_symbol` but missing one of the
runnable case predicates is a harness error rather than a silent skip.

The current type-elaboration runner copies each active `.miz` corpus file into
the same temporary package shape, runs the real frontend, then feeds the
resulting `SurfaceAst` through the resolver declaration-shell collector,
parser-backed signature projection extractor, and symbol collector. This keeps
type-elaboration cases honest about lower-stage prerequisites before checker
payload extraction begins.

After lower stages pass, the runner extracts checker-owned
`TypeExpressionInput` payloads only for unrecovered source `TypeExpression`
nodes in reserve-only sources whose head is the builtin `set` or `object` and
which have no attributes, arguments, parameter prefixes, or non-builtin symbol
heads. Those payloads are normalized by `mizar-checker`'s `TypeNormalizer`
against the collected `SymbolEnv`, assembled into a minimal checker-owned
`TypedAst` shell with type-entry links, and projected into checker-owned
`ResolvedTypedAst` using empty-but-real cluster/overload predecessor outputs
plus source-preserved node hints and expression metadata. Active pass cases may
assert this supported source-derived slice with empty detail keys only when the
source contains at least one supported builtin `TypeExpression` site and runner
regression evidence confirms that normalization, `TypedAst` assembly, and
`ResolvedTypedAst` assembly were exercised.

The runner still does not fabricate the missing AST-wide source-to-checker
bridge. Parsed/resolved `.miz` declarations, terms, formulas, coercion sites,
attribute/mode/structure payloads, overload evidence, recorded facts, and proof
evidence remain outside the supported extraction slice. When an active case
needs one of those unsupported payload families, the runner reports the stable
detail key `type_elaboration.external_dependency.ast_payload_extraction`.
Active fail cases may assert that key through `diagnostic_payloads` or
`stable_detail_key`; active pass cases outside the supported slice remain
deferred rather than passing through a stub. This runner does not publish
`CoreIr`, `ControlFlowIr`, VC seeds, or public checker diagnostic codes.

Active type-elaboration expectations with non-empty `diagnostic_codes` are
harness errors until public checker diagnostic codes are specified. An
expectation tagged `active_type_elaboration` but missing one of the runnable
case predicates is a harness error rather than a silent skip.

General snapshot and determinism runner rows above are target-state harness
modes. Tasks 4 and 5 provide the shared `SnapshotRecord`, baseline
verify/update, and repeat-render comparison APIs, but this harness does not yet
parse general `[[snapshots]]` sidecar entries or run a general snapshot/update
subcommand. The active parse-only `SurfaceAst` shortcut remains the only
snapshot path wired into runner execution.

Architecture-22 matrix support is metadata/reporting-only in task 14. The
metadata plan validates `architecture22_scenarios`,
`architecture22_equivalence_class`, and `architecture22_gate`, then reports the
registry class plus planned/active counts for each required scenario. All
task-14 scenario rows have no active eligibility, so `architecture22_gate =
"active"` is rejected until a future consumer-specific increment wires real
clean/incremental/parallel/cache-race execution.

## Determinism Requirements

The harness checks that identical inputs produce:

- identical artifact hashes;
- identical snapshot hashes;
- identical diagnostic order;
- identical failure records;
- identical proof status;
- identical dependency slices.

Parallel execution may change runtime, not observable results.

Implemented task-11 coverage renders metadata plans and active runner reports
to deterministic byte strings and compares repeated builds/runs. Snapshot-level
determinism and parallel equivalence are covered by the general snapshot record
helpers; active parallel runner subcommands remain future work until a consumer
crate exposes parallel execution.

## Reporting

Reports must separate:

- unexpected success;
- unexpected failure;
- wrong failure category;
- wrong rejection reason;
- diagnostic order mismatch;
- snapshot mismatch;
- nondeterminism across repeated runs;
- harness infrastructure error.

## Tests

Key scenarios:

- fail test unexpectedly passes;
- pass test emits an error diagnostic;
- snapshot hash differs;
- metadata plan bytes differ across repeated builds;
- active runner report bytes differ across repeated runs;
- repeated run produces a different diagnostic order;
- generic snapshot parallel equivalence produces the same observable artifact
  as sequential snapshot generation.
- architecture-22 matrix metadata reports all required scenario ids as planned
  and rejects fake active rows before an owning consumer runner exists.

## Constraints and Assumptions

- Test execution order is not semantic ordering.
- The harness treats cache hits as compiler behavior to verify, not as proof authority.
- Snapshot update mode is opt-in and must be visible in command output.
