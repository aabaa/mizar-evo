# Research Roadmap

> Research planning, not implementation sequencing or a release commitment.
> The [TODO](todo.md) and [performance roadmap](performance_optimization_roadmap.md)
> remain the authoritative work-order and optimization inventories.

## Research program

| ID | Question | Evaluation |
|---|---|---|
| R1 | Can soft-type inference replace expensive general search? | Rule/fact scaling, solver calls avoided, proof success and checking cost |
| R2 | How should FOL, SAT, SMT and computation cooperate? | Matched portfolio and theory benchmarks, trust-level comparison |
| R3 | Can template proof work be reused across instances? | Instantiation cost, evidence size, valid reuse and rejection cases |
| R4 | Can certified symbolic processing be inexpensive? | Normalization and computation time, evidence production and replay |
| R5 | Can mathematical and program verification scale incrementally? | Cold/warm/edit latency, invalidation, memory, clean/edit equivalence |
| R6 | Can verification reduce AI generation/repair cost? | Fixed-spec success, time, tokens/cost, iterations and counterexamples |
| R7 | Can formal mathematics remain understandable to people? | Matched formalization tasks, comprehension, annotation and repair effort |
| R8 | Can a general FOL verifier compete in program verification? | Oirase proof-of-concept and comparable Why3/Dafny/other workloads |
| R9 | Can formal systems exchange statements and evidence reliably? | Explicit translation fragment, semantic correspondence and independent checking |

R1, R2, R5 and R7 are core priorities. R3/R4 support these priorities;
R6/R8/R9 are research extensions and need not delay existing feature work.

## Milestones

| Stage | Target window (tentative) | Exit intent |
|---|---|---|
| Feature-complete baseline | 2027 H1 | Existing agreed feature scope; genuine source-to-verification examples; outstanding unsupported paths documented |
| Competitiveness experiments | 2027 | Run existing PERF baselines and selected improvements, small native theory experiments, useful foundation libraries and Oirase initial examples |
| Domain and agent expansion | 2028 onward | Expand validated C/Rust subset, model replay, human/agent feedback and optional interoperability only as justified |

The 2027 H1 goal is retained, not forecast from measured throughput. New SMT
work is not automatically a release blocker. Do not reorder active
VC53/MVM/PERF dependencies or expand the feature-completion contract here.
Design and experimental preparation may proceed in parallel when appropriate.

## Evaluation protocol

Use identical mathematical assumptions and specifications for comparisons.
Record system versions, solver versions/configuration, hardware, supported
fragment and trust basis. Distinguish feature coverage from proof success,
and external solver acceptance from independently checked evidence.
Measure frontend, VC preparation, search, checking and reuse separately;
report end-to-end wall time, tail latency, CPU, memory and success/timeout
rates. Use repeated cold/warm/edit workloads and scaling series; compare
both successful and unsuccessful cases. No speedup target is a prerequisite
before establishing a baseline.

Compare readability and formalization effort with Mizar, Lean and Isabelle
on matched mathematical tasks; use Why3/Dafny for small sequential program
verification and specialized C/Rust systems when source semantics match.
Evaluate human comprehension with defined tasks and appropriate participants,
rather than assuming declarative notation is universally superior.
In AI experiments fix specification, model and budget where possible;
count weakened/vacuous specifications and unverified results separately.
Report null or negative findings. Keep benchmark scripts, inputs and raw
measurements reproducible without creating unnecessary evidence infrastructure.

## Application program: Oirase

[Oirase](https://github.com/aabaa/oirase) is a separate research project.
Its initial scope is a small sequential program semantics, mathematical
word/array/state libraries and verified examples using the existing
Algorithm/VC architecture. Compare against established systems before
generalizing C/Rust adapters, memory ownership and agent repair loops.
The Oirase issue tracker owns application tasks; this document does not
duplicate its queue. Upstream generic theory, model, evidence and performance
improvements belong to Mizar-evo. FCL and other projects are optional
collaborators, not mandatory migrations.

## Research management

Protect soundness-critical boundaries while allowing inexpensive exploratory
experiments. Distinguish specification, design, working implementation,
validated integration and empirical research findings. Prefer small genuine
vertical slices over many disconnected fixtures or infrastructure layers.
Use the [library strategy](library_strategy.md) and
[reasoning strategy](reasoning_strategy.md) to guide proposals, and map
accepted work into existing owner contracts rather than duplicate queues.

## Open questions

Actual post-baseline throughput, attainable coverage, research staffing,
library readiness and comparative results remain unknown. Revisit milestones
after the feature baseline, the first scaling experiments and Oirase's
initial proof-of-concept; do not infer a delivery date from this roadmap.
