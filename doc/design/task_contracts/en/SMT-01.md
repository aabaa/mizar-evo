# Task SMT-01: Theory mapping design

Canonical language: English; [Japanese pointer](../ja/SMT-01.md).
Status: planned; tier and implementation slice to be frozen before dispatch.
Owner: [atp TODO](../../mizar-atp/en/todo.md); consumers: Core, Checker, Kernel, Proof.
Dependencies: No new language syntax. Design may start at an owner-approved boundary after reconciliation with active work; implementation requires reviewed contracts.
Authority: existing specification and [top-level roadmap](../../todo.md); any semantic or acceptance change requires separate reviewed specification/design before implementation.

## Scope

Define typed, resolved-symbol mappings between library definitions and solver sorts/operators, including guards, coercions, partial operations, quantifier domains and mixed-theory boundaries. Establish the direction of semantic preservation required for refutation and for countermodels separately. Specify mapping identity, dependencies, provenance and evidence/model projection interfaces. Compare a small built-in mapping table with a later library-supplied extension; do not trust arbitrary declarations or require a general plugin framework.

## Tests and exit

Review examples of integer division/modulo, signed/unsigned operations, finite array bounds and unsupported/mixed sorts. Require explicit rejection or a sound fallback, with no silent reinterpretation. Exit: reviewed paired design, selected initial fragment and owner boundaries; no proof acceptance or implementation credit.

No production change or completion credit is implied by this planning contract. Preserve existing owner boundaries and review/test requirements; freeze the smallest genuine implementation slice before work.
