# Module: formula_evidence

> Canonical language: English. Japanese companion:
> [../ja/formula_evidence.md](../ja/formula_evidence.md).

## Purpose

The `formula_evidence` module owns the corrected kernel evidence schema that
supersedes legacy resolution-trace certificates for normal proof acceptance.
It refines architecture 08, architecture 15, architecture 16, architecture 19,
and internal 04.

The schema is trusted input only in the narrow sense that it is the material the
kernel is asked to check. It is not accepted until provenance, substitutions,
target binding, formula identity, deterministic SAT encoding, and SAT
refutation all pass.

## Trust Statement

This module is trusted kernel code. It may parse and structurally validate
formula/substitution evidence, but parsing never grants proof acceptance.

The module must not perform proof search, premise selection, ATP search,
backend invocation, overload resolution, cluster search, implicit coercion
insertion, fallback inference, source loading, cache lookup, artifact lookup,
wall-clock or random-state reads, unordered iteration dependence, or hidden
reads of mutable compiler-global state. It must not accept instantiated
formulas, SAT clauses, backend proof methods, resolution traces, SMT proof
objects, or backend logs as trusted payload.

## Evidence Shape

The corrected evidence object is:

```text
KernelEvidence
  version
  target_vc
  kernel_profile
  formula_evidence
  substitutions
  final_goal
  provenance
```

`formula_evidence` entries record formulas available to the target VC:

```text
FormulaEvidenceEntry
  formula_id
  source_class
  formula
  formula_fingerprint
  required_proof_status?
  imported_fact_ref?
  local_context_ref?
  vc_fact_ref?
  provenance_ref
```

`source_class` is one of local hypothesis, cited premise, generated VC fact,
accepted imported axiom, accepted imported theorem, or policy-bounded built-in
fact. The first implementation may model formulas as a kernel-owned
propositional formula tree over normalized `clause::Atom` values; richer
Mizar/core formulas must be added only with a paired spec update and tests.

Each entry has a canonical formula identity and hash input derived from the
stored formula, source class, and provenance binding. The hash must not include
source paths, backend logs, timestamps, worker order, display names after
binding, or SAT clauses.

## Substitutions

Substitution records are explicit evidence. They identify the formula they are
applied to, the normalized binder context, side conditions, and the substitution
payload needed by `substitution_checker`.

Instantiated formulas are not trusted fields. Task 26 derives instantiated
formulas by applying checked substitutions to the source formulas. Missing,
stale, duplicate, or inconsistent substitution provenance is a kernel
rejection, not an invitation to infer a repair.

## Provenance And Target Binding

Every formula must bind to one available proof source. Imported facts must bind
to stable package/module/item identity, statement fingerprint, and required
proof status. Local context and generated VC facts must bind to caller-supplied
target and VC provenance. The kernel must derive trusted `used_axioms` only
from accepted formula evidence whose source class is an accepted imported axiom
or theorem.

`final_goal` records the target formula and refutation polarity. The kernel
checks the supplied formulas plus the negated goal according to the profile. It
must reject evidence whose target VC, goal fingerprint, kernel profile, or
context identity does not match the caller's immutable context.

## Legacy Evidence

Legacy `Certificate`, `generated_clauses`, `resolution_trace`, backend proof
method fields, and backend logs are compatibility or migration-audit material
only. Under normal proof policy they map to unsupported evidence and cannot
produce accepted `KernelCheckResult`, `used_axioms`, proof witnesses, cache
promotion, or artifact `kernel_verified` status.

## Gap Classification

- `design_drift` / `source_drift`: the task-22 source still accepts legacy
  resolution-trace certificates through `checker`; tasks 25-29 replace or gate
  that path.
- `test_gap`: task 25 must add round-trip, malformed evidence, provenance-gap,
  deterministic rendering, and hash-stability tests.
- `external_dependency_gap`: full source-derived formula payloads from VC/ATP
  producers are not complete yet; the kernel schema must reject missing
  producer payloads instead of fabricating them.

## Planned Tests

Task 25 must test structural round-trips, stable hash/rendering, unknown schema
versions, duplicate ids, malformed formulas, missing provenance, imported fact
identity/fingerprint mismatches, missing target/goal binding, and legacy
evidence rejected outside migration/audit mode.
