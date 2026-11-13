# Task ATPK-INSTANCE-FINDER: untrusted instance finder for kernel evidence
Canonical language: English; [Japanese pointer](../ja/ATPK-INSTANCE-FINDER.md).
Status: complete. Tier: full. Owner: [atp plan](../../mizar-atp/en/00.crate_plan.md); consumers: [kernel plan](../../mizar-kernel/en/00.crate_plan.md), [proof plan](../../mizar-proof/en/00.crate_plan.md), [test plan](../../mizar-test/en/00.crate_plan.md).
Dependencies: structured source substitutions, encoded-problem producer, local evidence wire, observed ground terms, PROOF-INSTANCE-SEARCH-LIMITS and ATPK-KERNEL-EQUALITY-INSTANCES. Backend advisory extraction remains optional and deferred.
Readiness: Backend-free core implemented through the [finder API](../../mizar-atp/en/instance_finder.md). Backend-assisted integration still needs real advisory output extraction.
Authority: specification §21.7.1-§21.7.5; [architecture 10](../../architecture/en/10.atp_backend_integration.md#evidence-extraction-by-instance-finding); [translator](../../mizar-atp/en/translator.md).
Gap: backend-free core resolved; real backend extraction and portfolio execution remain deferred (task 15 and Step 7 item 1).

## Scope

Given an encoded problem and, when available, the backend's advisory
used-premise report, search for ground substitutions of the premises and for
named equality instances whose derived formulas make the kernel's SAT problem
unsatisfiable, and emit them as `CandidateKernelEvidence` with provenance.
Search limits come from the verifier policy; results are deterministic for
fixed inputs and limits.

## Forbidden

Treating backend traces, logs, or used-premise reports as trusted; accepting
anything without the kernel; nondeterministic search order; widening
premises beyond the encoded problem.

## Tests and exit

Real encoded problems: pure first-order instantiation, an equational chain
needing congruence and transitivity, a problem solved without a backend, and
a limit-exceeded case that yields no evidence. Every emitted candidate must
be accepted by the kernel; corruption tests cover stale problems and forged
reports. Require independent specification, test-sufficiency,
implementation, volume/scope, and consistency reviews; `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
Exit: proved problems within the limits yield kernel-accepted evidence, and
failures yield none.
