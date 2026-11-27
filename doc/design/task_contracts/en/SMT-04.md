# Task SMT-04: Structured solver model extraction

Canonical language: English; [Japanese pointer](../ja/SMT-04.md).
Status: planned; tier and implementation slice to be frozen before dispatch.
Owner: [atp TODO](../../mizar-atp/en/todo.md); consumers: Core, VC, Driver, LSP.
Dependencies: SMT-01 model interface for typed theories; real backend and supported encoding; coordinate with ATP 32 (AIREV-OPEN-STATEMENT-COUNTEREXAMPLE). UF extraction may precede native theories.
Authority: existing specification and [top-level roadmap](../../todo.md); any semantic or acceptance change requires separate reviewed specification/design before implementation.

## Scope

Acquire actual models/values for SAT results, decode supported values with exact sort/width and map through resolved identities and provenance. Distinguish partial/abstract/unmapped models, concrete validated counterexamples, unknown and timeout. A model of a weakened premise set or incomplete instantiation is not a counterexample to the original obligation without validation. Preserve proof status; no default search on every statement. ATP 32 owns the existing statement-review feature; Oirase owns C/Rust execution replay.

## Tests and exit

Test genuine SAT/UNSAT/unknown, invalid or incomplete model, wrong target, pruned premises and source projection. Exit: reusable typed model producer on supported fragments with explicit validation level; ATP 32 remains the consumer and retains its own exit.

No production change or completion credit is implied by this planning contract. Preserve existing owner boundaries and review/test requirements; freeze the smallest genuine implementation slice before work.
