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
For the shared lint vocabulary, no SAT solving means no caller-supplied,
backend-supplied, or search-oriented SAT problem solving; this module performs
only the trusted in-process acceptance check over the kernel-derived problem.

The module must perform no proof search, no ATP search or backend invocation,
no premise selection, no formula selection, no substitution invention, no
overload resolution, no cluster search, no implicit coercion insertion, no
fallback inference, no acceptance from backend-reported success alone, no
source loading, no cache lookup, no artifact lookup, no network access, no
external process execution, no wall-clock or random-state reads, no unordered
iteration dependence, and no hidden reads of mutable compiler-global state.

## Wrapper API

The wrapper API implemented by task 27 is:

```text
SatCheckContext
  limits: SatCheckLimits

SatCheckLimits
  max_variables
  max_clauses
  max_literals
  max_literals_per_clause
  max_canonical_bytes
  max_conflicts = unsupported unless None
  max_propagations = unsupported unless None

SatCheckResult
  Unsat(SatCheckReport)
  Sat(SatCheckReport)
  Rejected(RejectionRecord)
```

The public entry point is `check_sat_problem(problem, context)`, where
`problem` is an `EncodedSatProblem` produced by `sat_encoding` and exposed to
the wrapper only through read-only accessors. The wrapper
exposes no `batsat` type, model enumeration, proof search configuration,
premise minimization, backend profile, solver heuristic option, or external
solver command. It also does not call or expose DRAT/proof production, theory
solver surfaces, callback surfaces, DIMACS parsing/printing, model/statistics
printing, `print_stats`, file parser paths, or string parser paths. Limits
cover variables, clauses, literals, clause width, and canonical input bytes
before constructing the solver.

Task 27 selects the dependency-audit alternative that does not use callback
interruption: `batsat` 0.6.0 has no stable exact public conflict/propagation
budget setter, so any non-`None` `max_conflicts` or `max_propagations` request
rejects deterministically before solver construction. This leaves exact solver
step budgets `deferred` until a dependency exposes a stable deterministic API.

## Dependency Requirements

Task 24 records the selected pure-Rust dependency:

```text
batsat = { version = "=0.6.0", default-features = false }
```

Task 27 must add exactly this dependency, verify the lockfile resolution for
`batsat 0.6.0` and its exact audited `bit-vec 0.5.1` transitive dependency,
and update the crate-local dependency lint guard. The audit covers version pinning, license,
determinism, unsafe code, transitive dependencies, no process/network behavior,
resource limits, API surface, and failure mapping.

Solver errors, unsupported clauses, limit exhaustion, satisfiable results, and
internal inconsistency are non-acceptance outcomes. Task 27 returns wrapper
evidence only; task 28 owns wiring `Unsat` into the kernel check service and
normal proof-policy acceptance. Until then, `Unsat` is necessary acceptance
evidence but does not by itself change the legacy service path.

Task 27 pins every `batsat::SolverOpts` field inside the wrapper to the
audited 0.6.0 defaults, with explicit assertions for the random and heuristic
surface: `random_var_freq = 0.0`, `random_seed = 91648253.0`, `rnd_pol =
false`, `rnd_init_act = false`, `phase_saving = 2`, `luby_restart = true`,
`restart_first = 100`, and `restart_inc = 2.0`. These deterministic heuristic
controls are tested but not exposed through `SatCheckContext` or evidence.

## Gap Classification

- `test_gap`: task 27 must cover satisfiable rejection, unsatisfiable
  acceptance, limit failures, solver errors, deterministic outcomes, and no
  external process/network behavior. It must also cover the exact
  dependency/lockfile lint guard and wrapper-owned pinning/non-exposure of
  deterministic `batsat` heuristic options.
- `source_drift`: task 24 is docs-only; `Cargo.toml`, `Cargo.lock`, and
  `src/sat_checker.rs` remain unchanged until task 27 integrates the wrapper.
- `deferred`: `batsat` has no public exact conflict/propagation budget setter;
  task 27 rejects unsupported step-budget requests instead of using wall-clock
  timeouts or unstable callback accounting.
