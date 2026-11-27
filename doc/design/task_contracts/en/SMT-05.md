# Task SMT-05: Persistent SMT sessions and context reuse

Canonical language: English; [Japanese pointer](../ja/SMT-05.md).
Status: planned; tier and implementation slice to be frozen before dispatch.
Owner: [atp TODO](../../mizar-atp/en/todo.md); consumers: Build, Driver, VC, Proof.
Dependencies: Design after SMT-01 interface; implementation after a genuine solver route and PERF-A01/A06 measurement method; coordinate with PERF-D05 and C03.
Authority: existing specification and [top-level roadmap](../../todo.md); any semantic or acceptance change requires separate reviewed specification/design before implementation.

## Scope

Compare isolated runs against bounded persistent sessions using push/pop or assumptions, initially per function or equivalent dependency scope. Separate reusable declarations/axioms from local path conditions and solver state; ensure cancellation, reset, resource limits, changed mapping/context invalidation and no cross-VC assumption leakage. Preserve isolated fallback and deterministic acceptance identity. Do not require identical solver search behavior; measure stability.

## Tests and exit

Benchmark cold/warm/edit, startup, solve time, success/timeout rate, memory and evidence validation on matched obligations. Test scope isolation, stale context, cancellation/recovery and evidence target binding. Exit: measured benefit and reviewed isolation contract before enabling by default.

No production change or completion credit is implied by this planning contract. Preserve existing owner boundaries and review/test requirements; freeze the smallest genuine implementation slice before work.
