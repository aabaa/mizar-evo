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
```

## Runner Modes

| Mode | Behavior |
|---|---|
| pass/fail | run `.miz` cases and match expected outcome |
| snapshot | compare canonical snapshot hashes |
| determinism | repeat runs and compare artifacts, diagnostics, and hashes |
| parallel-equivalence | compare sequential and parallel outputs |
| fuzz-regression | run minimized fuzz cases as ordinary committed tests |
| update | rewrite snapshots only when explicitly requested |

## Algorithm / Logic

1. Discover tests through `layout`.
2. Build a canonical `TestPlan`.
3. Run cases in deterministic display order, even when execution is parallel.
4. Capture compiler outputs as structured records.
5. Match pass/fail expectations before snapshot expectations.
6. Compare snapshots by canonical hash.
7. Report failures with phase, failure category, rejection reason, diagnostic code, and snapshot diff summary.

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
