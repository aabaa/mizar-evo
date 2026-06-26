# Module: sat_checker

> Canonical language: English. Japanese companion:
> [../ja/sat_checker.md](../ja/sat_checker.md).

## Purpose

The `sat_checker` module owns the small trusted wrapper around the audited
in-process Rust SAT checker selected by task 24. It decides only whether the
kernel-derived SAT problem is unsatisfiable.

## Trust Statement

This module is trusted kernel code. SAT checking is allowed only over a SAT
problem derived by `sat_encoding` from validated formula/substitution evidence.

The module must not perform ATP search, premise selection, formula selection,
substitution invention, overload resolution, cluster search, implicit coercion
insertion, fallback inference, source loading, cache lookup, artifact lookup,
network access, external process execution, wall-clock or random-state reads,
unordered iteration dependence, or hidden reads of mutable compiler-global
state.

## Wrapper API

The wrapper API expected by task 27 is:

```text
SatCheckInput
  problem
  limits

SatCheckResult
  Unsat
  Sat
  Rejected(reason)
```

The wrapper exposes no model enumeration, proof search configuration, premise
minimization, backend profile, or external solver command. Limits cover
variables, clauses, literals, propagation/conflict steps when supported by the
dependency, and canonical input bytes.

## Dependency Requirements

Task 24 must record the selected pure-Rust dependency or an explicit audited
decision not to add one. The audit must cover version pinning, license,
determinism, unsafe code, transitive dependencies, no process/network behavior,
resource limits, API surface, and failure mapping.

Solver errors, unsupported clauses, limit exhaustion, satisfiable results, and
internal inconsistency are non-acceptance outcomes. Only `Unsat` permits the
caller to accept formula/substitution evidence.

## Gap Classification

- `test_gap`: task 27 must cover satisfiable rejection, unsatisfiable
  acceptance, limit failures, solver errors, deterministic outcomes, and no
  external process/network behavior.
- `repo_metadata_conflict`: any dependency metadata conflict found in task 24
  must be reported only unless the user explicitly authorizes repository
  metadata repair.
