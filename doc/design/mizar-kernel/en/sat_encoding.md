# Module: sat_encoding

> Canonical language: English. Japanese companion:
> [../ja/sat_encoding.md](../ja/sat_encoding.md).

## Purpose

The `sat_encoding` module derives deterministic SAT problems from checked
formula/substitution evidence. It refines architecture 15 and architecture 16.

## Trust Statement

This module is trusted kernel code. It derives check artifacts from accepted
schema data; it does not consume SAT clauses supplied by a caller.

The module must not perform proof search, premise selection, ATP search,
backend invocation, overload resolution, cluster search, implicit coercion
insertion, fallback inference, source loading, cache lookup, artifact lookup,
wall-clock or random-state reads, unordered iteration dependence, or hidden
reads of mutable compiler-global state.

## Owned Behavior

The module owns:

- validating substitution side conditions by consuming checked
  `formula_evidence` and `substitution_checker` results;
- deriving instantiated formulas from source formulas and explicit checked
  substitutions;
- assigning SAT variables deterministically by canonical atom bytes;
- producing a deterministic CNF/Tseitin problem from formulas plus the negated
  target goal;
- exposing canonical bytes for diagnostics and replay checks.

The module does not own SAT solving, ATP encoding, premise selection, formula
selection, or backend proof extraction.

## Encoding Rules

The first schema version uses a propositional encoding over normalized atoms.
Atom identity is the canonical `clause::Atom` byte encoding under the evidence
profile. Variables are assigned in sorted canonical atom-byte order, with any
auxiliary Tseitin variables allocated in deterministic traversal order after
all atom variables.

The encoded problem contains all evidence formulas asserted true and the target
goal asserted false for refutation. Equivalent caller order must produce
identical canonical SAT bytes.

Instantiated formulas and SAT clauses are kernel-derived artifacts. They may be
recorded as diagnostic check traces, but they are never trusted input fields.

## Rejections

Malformed formula structure, unsupported formula operators, inconsistent
substitution reports, missing provenance, target mismatch, unsupported atom
encoding, and canonical byte budget failures reject before SAT checking.
Resource limits reject as `resource_exhaustion`; semantic encoding failures
map to `invalid_sat_refutation` or `missing_provenance` according to the
owning evidence field.

## Gap Classification

- `test_gap`: task 26 must cover stable encoding, substitution mutation
  rejection, equivalent-order determinism, target polarity, and resource
  limits.
- `external_dependency_gap`: richer quantified or theory-aware encodings wait
  for producer-owned formula payloads and paired specs; this module must not
  invent them.
