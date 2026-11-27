# Mizar Evolution — Grand Design

> Research strategy; English canonical. This document is non-normative for
> language semantics, proof acceptance and implementation dispatch.

## Mission and vision

Make mathematics understandable and verifiable by both humans and machines.
Mizar Evolution is a human-centered, high-performance foundation for formal
mathematics and verification. Its primary identity remains a theorem prover
for people who learn, formulate and understand mathematics; AI and software
verification extend that mission rather than replace it.

## Positioning and research hypothesis

Mizar's declarative mathematical heritage, Lean/Isabelle's mathematical
abstraction and automation, and Why3/Viper/Verus's program-verification
architectures motivate complementary comparisons. Mizar-evo should not try
to reproduce their entire feature sets. Its hypothesis is that first-order
logic (FOL), soft types, library-defined reasoning and explicit evidence
boundaries can together support readable mathematics and efficient,
predictable verification. FOL alone does not imply speed or completeness.

The investigation at commit 8d212ab55f15602d562571bc21f84fbdb6027d3b found
Core/CFG/VC representations, scoped type facts, formula/substitution evidence,
SAT checking, dependency slices and cache interfaces. General source-derived
algorithm verification, executable MVM, native SMT theories and model
projection were incomplete or restricted. These are historical observations,
not current completion claims. Library scale, proof automation, adoption
and measured performance remain competitive risks.

## Design principles

| ID | Principle |
|---|---|
| GD-01 | Retain FOL and distinguish soft-type predicates from the proof logic; preserve the specified mathematical foundation. |
| GD-02 | Prioritize readable mathematical definitions and declarative proofs; machine interfaces must not dictate surface syntax. |
| GD-03 | Place domain semantics and reusable mathematical knowledge in libraries by default. Extend the core only for demonstrated general needs. |
| GD-04 | Choose reasoning procedures according to problem structure; do not build a universal replacement ATP/SMT solver. |
| GD-05 | Separate candidate search, computation, translation and proof acceptance. State the trust basis of each result. |
| GD-06 | Share immutable semantic structures and reuse validated results at the narrowest useful dependency scope. |
| GD-07 | Expose structured obligations, diagnostics and model information to people and agents without privileging AI-generated input. |
| GD-08 | Treat performance, usability and automation advantages as falsifiable research hypotheses. |

## Architectural responsibilities

| Layer | Responsibility |
|---|---|
| Applications | Formal mathematics, education, Oirase, optional scientific/CAD integrations and agent workflows |
| Language and libraries | Definitions, theorems, templates, registrations, reductions, algorithms and domain semantics |
| Semantic processing | Resolution, type checking, elaboration, Core IR, Control Flow IR and VC IR |
| Reasoning | Local inference, normalization, computation, SAT, SMT and ATP; see [reasoning strategy](reasoning_strategy.md) |
| Evidence and trust | Existing Kernel and Proof policy; separate validation of model/counterexample claims |
| Infrastructure (cross-cutting) | Source maps, dependency slices, fingerprints, artifacts, cache, scheduling, diagnostics and LSP |

This is a responsibility map, not a new crate hierarchy or required execution
order. Reuse the existing [pipeline](architecture/en/00.pipeline_overview.md)
and [IR layers](architecture/en/01.ir_layers.md). Source-to-VC correctness
and the authenticity of the check context are distinct from checking a
logical proof; do not claim end-to-end verification from Kernel success alone.

## Strategic priorities and boundaries

Human-centered mathematical formalization, high-performance predictable
verification, hybrid reasoning and library-driven extension are the core
priorities. Agent feedback and interoperability are complementary directions.
Lean translation is optional research, not a required verification backend.
Oirase is an independent application project, not a new execution language
or a reason to duplicate the existing verifier. Other domain projects may
adopt Mizar-evo when useful without mandatory migration.

Avoid premature generalization, full MML migration as a release gate,
unmeasured optimization, and new trusted machinery where existing evidence
suffices. Research prototypes must label unsupported cases and trust levels;
production acceptance must satisfy the existing specification and policy.

## Open questions

Which workloads demonstrate a practical advantage? Which mathematical
abstractions require additional mechanisms rather than libraries? Which
proof-search and model services merit stable public interfaces? Resolve
these through the [research roadmap](research_roadmap.md) and reviewed
owner designs; the [TODO](todo.md) remains the execution-order authority.
