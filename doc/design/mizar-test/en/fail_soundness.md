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

The test metadata must use stable detail key
`soundness.false_arithmetic.one_eq_zero` and expect a proof failure,
certificate rejection, or kernel rejection according to the phase that first
detects the invalid proof attempt. The case stays in the default `fast`
profile.

## Required Domains

| Domain | Required Cases |
|---|---|
| substitution | variable capture, binder collision, malformed substitution, alpha-conversion failure |
| certificate | malformed certificate, invalid substitution, invalid SAT proof, unresolved symbol, timeout, resource exhaustion |
| cluster | infinite chain, cyclic registration, unintended coercion, hidden transitive expansion |
| overload | ambiguous notation, hidden coercion, unstable resolution order; accepted coherent same-root refinement joins must not be classified as ambiguity failures |
| dependency | stale theorem statement fingerprint, stale cluster semantics, stale notation parse result |
| policy | externally attested evidence rejected under `require_kernel_certificates` |

Required fail/soundness cases are tracked at case granularity, not merely by
domain presence. The harness recognizes these stable failure identities:

| Stable detail key | Domain | Outcome | Failure categories | Rejection reasons | Stages | Expected phases |
|---|---|---|---|---|---|---|
| `soundness.false_arithmetic.one_eq_zero` | `soundness` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | phase-specific | `proof_verification`, `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |
| `soundness.substitution.variable_capture` | `substitution` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | `invalid_substitution` | `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |
| `soundness.substitution.binder_collision` | `substitution` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | `invalid_substitution` | `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |
| `soundness.substitution.malformed_substitution` | `substitution` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | `invalid_substitution` | `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |
| `soundness.substitution.alpha_conversion_failure` | `substitution` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | `invalid_substitution` | `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |
| `soundness.certificate.malformed_certificate` | `certificate` | fail | `certificate_rejection` | `malformed_certificate` | `advanced_semantics` | `certificate_check`, `kernel_check` |
| `soundness.certificate.invalid_substitution` | `certificate` | fail | `kernel_rejection` | `invalid_substitution` | `advanced_semantics` | `certificate_check`, `kernel_check` |
| `soundness.certificate.invalid_sat_proof` | `certificate` | fail | `kernel_rejection` | `invalid_sat_proof` | `advanced_semantics` | `certificate_check`, `kernel_check` |
| `soundness.certificate.unresolved_symbol` | `certificate` | fail | `certificate_rejection`, `kernel_rejection` | `unresolved_symbol` | `advanced_semantics` | `certificate_check`, `kernel_check` |
| `soundness.certificate.timeout` | `certificate` | fail | `certificate_rejection`, `kernel_rejection` | `timeout` | `advanced_semantics` | `certificate_check`, `kernel_check` |
| `soundness.certificate.resource_exhaustion` | `certificate` | fail | `certificate_rejection`, `kernel_rejection` | `resource_exhaustion` | `advanced_semantics` | `certificate_check`, `kernel_check` |
| `soundness.cluster.infinite_chain` | `cluster` | fail | `cluster_error` | `cluster_loop` | `advanced_semantics` | `cluster_resolution` |
| `soundness.cluster.cyclic_registration` | `cluster` | fail | `cluster_error` | `cluster_loop` | `advanced_semantics` | `cluster_resolution` |
| `soundness.cluster.unintended_coercion` | `cluster` | fail | `cluster_error` | `unintended_coercion` | `advanced_semantics` | `cluster_resolution` |
| `soundness.cluster.hidden_transitive_expansion` | `cluster` | fail | `cluster_error` | `hidden_transitive_expansion` | `advanced_semantics` | `cluster_resolution` |
| `soundness.overload.ambiguous_notation` | `overload` | fail | `overload_error` | `ambiguous_notation` | `advanced_semantics` | `overload_resolution` |
| `soundness.overload.hidden_coercion` | `overload` | fail | `overload_error` | `hidden_coercion` | `advanced_semantics` | `overload_resolution` |
| `soundness.overload.unstable_resolution_order` | `overload` | fail | `overload_error` | `unstable_resolution_order` | `advanced_semantics` | `overload_resolution` |
| `soundness.dependency.stale_theorem_statement_fingerprint` | `dependency` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | `stale_theorem_statement_fingerprint` | `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |
| `soundness.dependency.stale_cluster_semantics` | `dependency` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | `stale_cluster_semantics` | `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |
| `soundness.dependency.stale_notation_parse_result` | `dependency` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | `stale_notation_parse_result` | `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |
| `soundness.policy.externally_attested_evidence_rejected` | `policy` | fail | `proof_failure`, `certificate_rejection`, `kernel_rejection` | `externally_attested_evidence_rejected` | `advanced_semantics` | `verification`, `certificate_check`, `kernel_check` |

The coherent same-root overload refinement case is a required positive guard
against false ambiguity diagnostics. It is not part of the fail-case list
above; the advanced-semantics runner that can execute overload assertions owns
activating that pass fixture.

## Expected Failure Contract

Every fail/soundness test declares:

```rust
pub struct FailExpectation {
    pub expected_phase: PipelinePhase,
    pub category: FailureCategory,
    pub rejection_reason: Option<RejectionReason>,
    pub diagnostic_codes: Vec<DiagnosticCode>,
    pub diagnostic_payloads: Vec<String>,
    pub stable_detail_key: String,
}
```

The harness checks category and rejection reason before diagnostic wording.
Diagnostic text may improve; failure identity must remain stable. A
`stable_detail_key` is always required for fail expectations; `diagnostic_codes`
and `diagnostic_payloads` are ordered assertion channels when public diagnostic
codes or machine-readable payloads exist. Certificate and kernel rejections
must include `rejection_reason`; ordinary parser/resolver/type runners may use
the stable detail key as the fallback identity while public diagnostic-code
ranges are still intentionally absent.

## Algorithm / Logic

Fail test execution:

1. Run the compiler under the requested profile.
2. Collect structured failure records and diagnostics.
3. Sort records by architecture-defined deterministic ordering.
4. Match expected phase, category, rejection reason, diagnostic code, and stable detail key.
5. Reject the test if the compiler succeeds, times out unexpectedly, emits only weaker policy evidence, or reports a different failure boundary.

Metadata validation enforces the case table above whenever fail/soundness
bookkeeping is active. Bookkeeping is active when the trace manifest contains a
fail/soundness requirement, or when a sidecar uses one of the recognized
`soundness.*` stable detail keys. Metadata mode reports missing required cases
as warnings; development and release modes report them as errors. This is a
metadata gate only: until proof, certificate, kernel, and advanced-semantics
runners exist, the harness does not fabricate active execution for these cases.

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
