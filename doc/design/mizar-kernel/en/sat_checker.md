# Module: sat_checker

> Canonical language: English. Japanese companion:
> [../ja/sat_checker.md](../ja/sat_checker.md).

## Purpose

The `sat_checker` module owns the small trusted wrapper around the audited
in-process Rust SAT checker selected by task 24. Task 24 selected direct
`batsat = { version = "=0.6.0", default-features = false }`; the full audit is
[sat_dependency_audit.md](./sat_dependency_audit.md). The module decides only
whether the kernel-derived SAT problem is unsatisfiable.

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

Task 24 records the selected pure-Rust dependency:

```text
batsat = { version = "=0.6.0", default-features = false }
```

Task 27 must add exactly this dependency, verify the lockfile resolution for
`batsat` and its `bit-vec` transitive dependency, and update the crate-local
dependency lint guard. The audit covers version pinning, license,
determinism, unsafe code, transitive dependencies, no process/network behavior,
resource limits, API surface, and failure mapping.

Solver errors, unsupported clauses, limit exhaustion, satisfiable results, and
internal inconsistency are non-acceptance outcomes. Only `Unsat` permits the
caller to accept formula/substitution evidence.

## Gap Classification

- `test_gap`: task 27 must cover satisfiable rejection, unsatisfiable
  acceptance, limit failures, solver errors, deterministic outcomes, and no
  external process/network behavior. It must also cover the exact
  dependency/lockfile lint guard and wrapper-owned pinning/non-exposure of
  deterministic `batsat` heuristic options.
- `source_drift`: task 24 is docs-only; `Cargo.toml`, `Cargo.lock`, and
  `src/sat_checker.rs` remain unchanged until task 27 integrates the wrapper.
- `deferred`: `batsat` has no public exact conflict/propagation budget setter;
  task 27 must either prove/test callback-based deterministic interruption or
  reject unsupported step-budget requests.
