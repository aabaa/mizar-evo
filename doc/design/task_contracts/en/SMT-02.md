# Task SMT-02: Native SMT theory encoding

Canonical language: English; [Japanese pointer](../ja/SMT-02.md).
Status: planned; tier and implementation slice to be frozen before dispatch.
Owner: [atp TODO](../../mizar-atp/en/todo.md); consumers: Core, VC, Proof.
Dependencies: SMT-01 reviewed; genuine VC/translation producers for each supported fragment; SMT-03 evidence contract before trusted adoption.
Authority: existing specification and [top-level roadmap](../../todo.md); any semantic or acceptance change requires separate reviewed specification/design before implementation.

## Scope

Extend backend-neutral problem/profile and SMT-LIB emission in small slices: linear integer arithmetic first, then linear real arithmetic, fixed-width bit-vectors and arrays as demand and benchmarks justify. Nonlinear arithmetic, datatypes and floating point are separate later decisions. Preserve current UF path. Bind exact library semantics, sorts, widths, guards, translations and target identities. External results are candidate/diagnostic only until an existing approved policy or checked evidence authorizes the relevant status.

## Tests and exit

Test supported and unsupported fragments, signedness, widths, overflow, division, coercions, array domains, deterministic encoding and genuine solver runs. Compare equivalent small tasks with UF baseline; no fixed speed target. Exit per slice: reviewed translation contract, tests and measured supported route; no implicit kernel_verified status.

No production change or completion credit is implied by this planning contract. Preserve existing owner boundaries and review/test requirements; freeze the smallest genuine implementation slice before work.
