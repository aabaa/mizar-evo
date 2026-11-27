# Hybrid Reasoning Strategy

> Research and engineering direction, not a normative inference rule set.
> See the [grand design](grand_design.md).

## Objective

Reduce end-to-end verification cost without weakening logical guarantees.
Exploit the structure of common mathematical and program-verification
obligations rather than delegate every goal to general-purpose proof search.

## Reasoning portfolio

| Class | Preferred candidate mechanism | Boundary |
|---|---|---|
| Type and attribute facts | Indexed registration closure | Scope, guards, accepted rule evidence |
| Term simplification | Oriented Reduction and explicit equality lemmas | Existing termination order and provenance |
| Concrete computation | Algorithm/MVM | Computability, admission, replay policy |
| Equality and propositional reasoning | Congruence/equality instances and SAT | Checked evidence; measure dedicated engines before adoption |
| Arithmetic, words and arrays | Native SMT theories | Exact mathematical correspondence and theory evidence |
| General quantified FOL | External ATP and ground-instance search | Candidate generation only |
| Domain-specific reasoning | Library lemmas, schemas, future tactics | No unchecked proof admission |

This table is not a fixed solver cascade. Cheap local processing should
normally precede costly search, but the selection, budget, fallback and
parallel strategy must be measured. A local failure is not a counterexample;
an external UNSAT result is not automatically a verified proof.

## Local reasoning and sharing

Use the restrictions of soft-type and registration rules to index subjects,
predicates and triggers; prefer worklist closure over repeated full scans.
Maintain scope, guard evidence and state-value identity. A fact about a
mutable variable's old value must not become a fact about its new value.
Apply reductions only within the specified ordering and applicability
conditions; strategy-normal form need not be confluent. General symbolic
rewriting and ground evaluation are different services.

Share semantic nodes separately from source occurrences; preserve binder
identity, provenance and dependency context. Investigate persistent local
contexts, template instance reuse, formula/Tseitin sharing and premise
indexes. For program VCs, measure branch-join precision, loop cutpoints and
formula growth; do not sacrifice soundness for a smaller VC.

Distinguish immutable static reuse (library/import scope), solver-session
reuse (compatible local contexts), and persistent verification reuse
(validated dependency and evidence identity). Session scopes must isolate
path assumptions and support reset, cancellation and independent fallback.
A cache miss changes work performed, not the meaning of verification.

## SMT theory bridge

Keep FOL as the mathematical authority. Map resolved definitions and
type guards to solver sorts/operators only under reviewed correspondence
conditions. Treat refutation soundness and countermodel validity separately;
partial operations, division, coercions, bit widths and finite-array bounds
must retain their source semantics. Start with linear integer arithmetic,
then select real arithmetic, words and arrays according to workload need.
Do not require an arbitrary library plugin interface before a narrow
working experiment.

The existing uninterpreted SMT path remains useful. Native theory extension
must preserve a sound fallback or refuse unsupported translation. For proof
acceptance, compare reconstruction using verified lemmas, arithmetic
certificates, justified bit-blasting and array theory lemmas. Reuse existing
evidence checking where possible. Do not enlarge the trusted computing base
without a separate soundness review. The present SAT-based Kernel path
recomputes SAT and trusts its pinned solver; a separately checked UNSAT
certificate route is a research alternative, not an existing guarantee.

## Results and agent feedback

Separate proved obligations, policy-permitted external results, computed
results, open obligations, unsupported input and resource exhaustion using
existing status semantics. Model extraction belongs to candidate/diagnostic
processing; abstract or partial models, including models of pruned premises,
must not be reported as concrete counterexamples without validation.
Domain adapters own source-language replay and correspondence. Provide
structured failure causes to human and agent clients without changing proof
status merely because a search failed.

## Implementation and evaluation

The [performance roadmap](performance_optimization_roadmap.md) owns existing
optimization packages (particularly B06, C03/C04 and D01–D07); the
[TODO](todo.md) owns SMT-01–05 and existing ATP counterexample work. Do not
create a duplicate dispatch queue. Measure local processing, solver startup,
search, evidence production/checking and reuse separately on matched goals.
Compare success, tail latency, memory and robustness as well as elapsed time.

## Open questions

When does local inference outperform solver inference? Which theory evidence
can be reconstructed economically? When does session reuse degrade solver
stability? Which domain-specific transformations can remain library-defined?
Resolve by controlled experiments and owner-reviewed contracts.
