# Module: harness

> Canonical language: English. Japanese companion: [../ja/harness.md](../ja/harness.md).

## Purpose

This module defines the test harness that discovers cases, runs compiler profiles, checks expectations, and reports deterministic results.

## Public API

```rust
pub struct TestPlan {
    pub cases: Vec<TestCase>,
    pub profile: TestProfile,
}

pub enum TestProfile {
    Fast,
    Full,
    Stress,
    FuzzRegression,
    SnapshotUpdate,
}

pub struct TestOutcome {
    pub case: TestCaseId,
    pub status: TestStatus,
    pub diagnostics: Vec<Diagnostic>,
    pub snapshots: Vec<SnapshotRecord>,
}

pub struct ParseOnlyRunReport {
    pub results: Vec<ParseOnlyCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

pub struct DeclarationSymbolRunReport {
    pub results: Vec<DeclarationSymbolCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}
```

## Runner Modes

| Mode | Behavior |
|---|---|
| metadata plan | discover sidecars and validate layout, expectation schema, and traceability without executing payloads |
| parse-only | run active `.miz` parse-only cases through `mizar-frontend` and `MizarParserSeam` |
| declaration-symbol | run active `.miz` declaration-symbol cases through frontend parsing and resolver declaration/symbol collection |
| pass/fail | run `.miz` cases and match expected outcome |
| snapshot | compare canonical snapshot hashes |
| determinism | repeat runs and compare artifacts, diagnostics, and hashes |
| parallel-equivalence | compare sequential and parallel outputs |
| fuzz-regression | run minimized fuzz cases as ordinary committed tests |
| update | rewrite snapshots only when explicitly requested |

## Algorithm / Logic

1. Discover tests through `layout`.
2. Build a canonical `TestPlan`.
3. For `parse-only`, select only cases with `stage = "parse_only"`,
   `expected_phase = "parse"`, `.miz` payloads, pass/fail outcomes, and
   `tags = ["active_parse_only"]`. Untagged parse-only sidecars remain
   discovery and traceability metadata.
4. For `declaration-symbol`, select only cases with
   `stage = "declaration_symbol"`, `expected_phase = "resolve"`, `.miz`
   payloads, pass/fail outcomes, and `tags = ["active_declaration_symbol"]`.
   Untagged declaration-symbol sidecars remain discovery and traceability
   metadata.
5. Run cases in deterministic display order, even when execution is parallel.
6. Capture compiler outputs as structured records.
7. Match pass/fail expectations before snapshot expectations.
8. Compare general `[[snapshots]]` entries by canonical hash; the current
   parse-only `SurfaceAst` shortcut compares committed text baselines
   byte-for-byte as described below.
9. Report failures with phase, failure category, rejection reason, diagnostic code, and snapshot diff summary.

The current parse-only runner copies each active corpus file into a temporary
`src/` package, runs the real frontend parser seam, requires pass cases to
produce an AST with no assertion diagnostics, and compares fail cases against
the expected bare syntax diagnostic keys. For this syntax-only mode, the runner
uses a harness provider that resolves every frontend import stub to a
`ResolvedImportEntry` with matching `stub_ordinal` and `stub_span`, plus one
empty `ModuleLexicalSummary` per distinct module id. The summaries contain no
exported symbols and exist only to keep import syntax cases from depending on
semantic module availability. If parser syntax diagnostics and non-syntax
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

## Determinism Requirements

The harness checks that identical inputs produce:

- identical artifact hashes;
- identical snapshot hashes;
- identical diagnostic order;
- identical failure records;
- identical proof status;
- identical dependency slices.

Parallel execution may change runtime, not observable results.

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
- repeated run produces a different diagnostic order;
- parallel run produces the same artifacts as sequential run.

## Constraints and Assumptions

- Test execution order is not semantic ordering.
- The harness treats cache hits as compiler behavior to verify, not as proof authority.
- Snapshot update mode is opt-in and must be visible in command output.
