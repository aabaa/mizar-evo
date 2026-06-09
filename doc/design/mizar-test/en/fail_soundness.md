# Module: fail_soundness

> Canonical language: English. Japanese companion: [../ja/fail_soundness.md](../ja/fail_soundness.md).

## Purpose

This module defines fail and soundness regression contracts.

The priority is to reject what must not pass. A soundness failure is more serious than failing to accept some valid programs.

## Required Soundness Cases

The following theorem must never pass:

```mizar
theorem
  1 = 0;
```

The test metadata must expect a proof failure, certificate rejection, or kernel rejection according to the phase that first detects the invalid proof attempt.

## Required Domains

| Domain | Required Cases |
|---|---|
| substitution | variable capture, binder collision, malformed substitution, alpha-conversion failure |
| certificate | malformed certificate, invalid substitution, invalid SAT proof, unresolved symbol, timeout, resource exhaustion |
| cluster | infinite chain, cyclic registration, unintended coercion, hidden transitive expansion |
| overload | ambiguous notation, hidden coercion, unstable resolution order; accepted coherent same-root refinement joins must not be classified as ambiguity failures |
| dependency | stale theorem statement fingerprint, stale cluster semantics, stale notation parse result |
| policy | externally attested evidence rejected under `require_kernel_certificates` |

## Expected Failure Contract

Every fail/soundness test declares:

```rust
pub struct FailExpectation {
    pub expected_phase: PipelinePhase,
    pub category: FailureCategory,
    pub rejection_reason: Option<RejectionReason>,
    pub diagnostic_codes: Vec<DiagnosticCode>,
    pub stable_detail_key: String,
}
```

The harness checks category and rejection reason before diagnostic wording. Diagnostic text may improve; failure identity must remain stable.

## Algorithm / Logic

Fail test execution:

1. Run the compiler under the requested profile.
2. Collect structured failure records and diagnostics.
3. Sort records by architecture-defined deterministic ordering.
4. Match expected phase, category, rejection reason, diagnostic code, and stable detail key.
5. Reject the test if the compiler succeeds, times out unexpectedly, emits only weaker policy evidence, or reports a different failure boundary.

## Regression Rules

- A fail/soundness test that starts passing is a release blocker unless the expectation is explicitly changed with proof of soundness.
- A test may move to an earlier failure phase only when the earlier phase soundly detects the same invalid input.
- A timeout is not success.
- Resource exhaustion is not success.
- Externally attested evidence is never equivalent to kernel verification.

## Tests

Key scenarios:

- false arithmetic theorem fails;
- malformed certificate is classified separately from invalid SAT proof;
- substitution capture fails with `invalid_substitution`;
- cluster cycle fails with `cluster_loop`;
- ambiguous ordinary overload fails without depending on candidate iteration order;
- coherent same-root redefinitions with compatible joined facts pass rather than being misclassified as ambiguous overloads.

## Constraints and Assumptions

- Fail expectations are explicit and versioned.
- The harness never infers expected failure from current compiler output.
- Soundness cases are run in the default test profile.
