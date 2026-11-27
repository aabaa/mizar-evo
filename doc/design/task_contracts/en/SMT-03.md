# Task SMT-03: Theory evidence checking

Canonical language: English; [Japanese pointer](../ja/SMT-03.md).
Status: planned; tier and implementation slice to be frozen before dispatch.
Owner: [kernel TODO](../../mizar-kernel/en/todo.md); consumers: ATP, Proof, VC.
Dependencies: Design alongside SMT-01 and PERF-D07; implementation per SMT-02 fragment and genuine evidence producer.
Authority: existing specification and [top-level roadmap](../../todo.md); any semantic or acceptance change requires separate reviewed specification/design before implementation.

## Scope

Compare reconstruction using verified FOL lemmas, checked arithmetic certificates, bit-blasting with justified CNF correspondence, and array theory lemmas. Reuse current SAT checking or separately approved PERF-D07 certificate path; prefer existing proof machinery before extending the trusted base. Bind evidence to the original VC, mapping and accepted context. Integer-specific reasoning requires more than rational linear combinations. External UNSAT alone is not kernel evidence; unsupported proof steps stay unaccepted.

## Tests and exit

Positive and adversarial tests for wrong goal, mapping, coefficients, width, guards and malformed/unsupported steps; measure evidence generation, transfer and checking separately. Exit per theory: reviewed soundness argument and actual independent acceptance route; do not replace the existing Kernel path without approval.

No production change or completion credit is implied by this planning contract. Preserve existing owner boundaries and review/test requirements; freeze the smallest genuine implementation slice before work.
