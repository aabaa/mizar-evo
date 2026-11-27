# Mathematical Library Strategy

> Research direction; English canonical. Does not define new language syntax
> or change the authority of the mathematical foundation.

## Objective

Build a library that makes ordinary mathematical reasoning concise and
effective, not merely a large inventory of theorems. A useful definition
should have an understandable interface, essential lemmas and appropriate
automatic reasoning support. The library is also the primary extension
surface for domain-specific verification.

## Library organization

The following logical layers are proposed; package names and splits are
not fixed.

| Layer | Contents | Initial priority |
|---|---|---|
| L0 Foundations and computation | Sets, logic, relations, functions, natural numbers, integers, rationals, reals, induction and basic arithmetic | Small usable foundation; stage real analysis later |
| L1 Mathematical abstractions | Orders, algebraic structures, finite sets/sequences, maps, products, matrices and vector spaces | Reusable interfaces and examples |
| L2 Domain packages | Program semantics, machine integers, arrays, memory, geometry, cryptography, probability and other applications | Driven by concrete research needs |

Dependencies flow from domain packages to abstractions and foundations;
avoid cycles and unnecessary imports. Shared mathematical definitions should
have one owner, with domain-specific interpretations layered above them.
Oirase owns C/Rust semantics and adapters; reusable word/array mathematics
may be promoted to shared libraries after demonstrated reuse.

## Knowledge representation

Use ordinary definitions and theorems as the baseline. Add automation when
it improves common tasks:

- Templates express parameterized types, predicates, functors, theorems and
  algorithms under the existing parameter and constraint rules.
- Registrations propagate proved type/attribute facts under applicable guards.
  Do not assume arbitrary term-argument attributes are permitted in cluster
  rules or that state-dependent facts survive mutation.
- Reductions express proved, oriented, strictly decreasing term rewrites;
  equal-size or expanding transformations require other justified mechanisms.
- Algorithms provide executable mathematics where supported, distinguishing
  partial correctness, normal return, unique results and computation policy.
- Theory mappings connect eligible mathematical definitions to SMT built-ins
  only through the reviewed mechanism in the [reasoning strategy](reasoning_strategy.md).

These mechanisms are specified in
[Ch.17](../spec/en/17.clusters_and_registrations.md),
[Ch.18](../spec/en/18.templates.md) and
[Ch.20](../spec/en/20.algorithm_and_verification.md). Their general
implementation is not assumed complete.

## Initial library work

Start with a small vertical slice: integer arithmetic and ordering,
finite sequences, elementary functions/relations, and a few algebraic
interfaces. Provide mathematical examples readable independently of the
verifier internals and small algorithm examples for Oirase. Prioritize
definitions and lemmas repeatedly needed in these examples. Expand to
additional mathematics through actual use rather than speculative coverage.

For each library unit, document the mathematical definitions, hypotheses,
notation, dependencies and main usable results. Test representative
successful and failing proof obligations, automation applicability and
import costs. Distinguish verified facts from assumptions and experimental
external results. Measure proof length, required manual lemmas, validation
time and reuse; theorem count alone is not a success metric.

## Existing Mizar Mathematical Library (MML)

Treat MML as a valuable source of definitions, theorems and mathematical
organization, not as a mandatory full migration before release. Inventory a
small coherent subset, determine license/attribution requirements, map
resolved definitions and assumptions, and compare source and destination
statements. Preserve relevant hypotheses and reverify translated proofs or
explicitly identify unproved assumptions. AI-assisted translation may propose
candidates but cannot establish equivalence or proof acceptance.

Compare selective migration, re-formalization with modern abstractions and
interoperability. Expand migration only when the pilot demonstrates useful
coverage and manageable maintenance. Avoid maintaining duplicate versions
of the same mathematical definition without an explicit correspondence.

## Open questions

Which foundation and abstraction interfaces are already available and
usable? What is the best package split? Which MML subset and license policy
are suitable for a pilot? Which automation rules reduce effort without
increasing global search cost? Resolve with representative examples and
actual supported language features before freezing library APIs.
