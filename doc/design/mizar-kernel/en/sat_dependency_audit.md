# SAT Dependency Audit: mizar-kernel Task 24

> Canonical language: English. Japanese companion:
> [../ja/sat_dependency_audit.md](../ja/sat_dependency_audit.md).

## Scope

Task 24 selects and audits the Rust SAT checker dependency that may be trusted
by `mizar-kernel` after task 27 integrates the wrapper. This task does not edit
`Cargo.toml`, `Cargo.lock`, or Rust source. Source integration is deliberately
deferred until the formula/substitution evidence schema and deterministic SAT
encoding tasks define the exact in-crate input type.

The dependency decision applies only to SAT checking over a SAT problem that the
kernel derives from validated formula/substitution evidence. It does not make a
backend proof method, resolution trace, SMT proof object, backend log, model, or
solver configuration trusted acceptance material.

## Decision

Selected direct dependency for task 27:

```toml
batsat = { version = "=0.6.0", default-features = false }
```

Rationale:

- `batsat` is a pure-Rust MiniSat-lineage SAT solver crate with MIT license,
  no build script, and no default features.
- The published library source contains no external process execution, network
  access, filesystem access, host/environment random-state reads, or wall-clock
  reads.
- Its direct normal dependency set is small: `bit-vec` only.
- The direct `batsat` API is smaller than the `rustsat-batsat` adapter path and
  avoids RustSAT's public external-solver process interface.
- Kernel code can construct a solver from the deterministic `sat_encoding`
  output, call the in-process solver, and accept only `UNSAT`.

Task 27 must add this dependency exactly, update the crate-local dependency
lint guard, and verify the resulting `Cargo.lock`. If Cargo resolves a
different `batsat` version, enables features, or resolves a different
`bit-vec` version than the one audited here, task 27 must stop and refresh this
audit before committing.

## Audit Sources

Audit date: 2026-06-26.

Commands and artifacts inspected:

- `cargo search "SAT solver" --limit 20`
- `cargo info batsat@0.6.0`
- `cargo info bit-vec@0.5.1`
- published source manifests under the local cargo registry for
  `batsat-0.6.0` and `bit-vec-0.5.1`
- source grep over the published library source for `unsafe`, FFI, process,
  network, filesystem, host/environment random, and wall-clock APIs

Published crate archive checksums observed in the local cargo registry:

| Crate | Version | SHA-256 |
|---|---:|---|
| `batsat` | `0.6.0` | `ec82b6bbce8ea42f5003417b699267860a9f4dd869fc9ba8faceac761d5afed1` |
| `bit-vec` | `0.5.1` | `f59bbe95d4e52a6398ec21238d31577f2b28a9d86807f06ca59d191d8440d0bb` |

These checksums are audit observations, not a replacement for `Cargo.lock`.
Task 27 owns the committed lockfile verification.

## Accepted Dependency Metadata

| Crate | Role | Version / requirement | License | Feature policy | Notes |
|---|---|---|---|---|---|
| `batsat` | direct SAT checker dependency | exact `=0.6.0` | MIT | `default-features = false`; do not enable `logging` | Pure-Rust MiniSat reimplementation. Manifest has `build = false`; normal dependency is `bit-vec = "0.5.0"`. |
| `bit-vec` | transitive bit-vector storage dependency | expected lock resolution `0.5.1` for `batsat`'s `0.5.0` requirement | MIT/Apache-2.0 | default `std` only | Library source has no process/network/filesystem/time/random APIs; dev-only benchmark code references `rand`. |

License conclusion: MIT and MIT/Apache-2.0 are compatible with the workspace MIT
metadata. No `repo_metadata_conflict` is recorded for the selected dependency
set.

## Unsafe-Code Audit

The kernel crate itself must keep `#![forbid(unsafe_code)]`; task 27 must not
add unsafe Rust to `mizar-kernel`.

The selected dependency tree is not unsafe-free:

- `batsat` uses `unsafe` in its clause storage representation, literal/extra
  field union access, raw-slice views, and watched-literal update path.
- `bit-vec` exposes unsafe methods such as direct storage access and length
  mutation.

The source grep found no FFI (`extern "C"`), no external process API, no
network API, and no filesystem API in the `batsat` library source. The unsafe
code therefore remains a dependency-internal memory-representation risk rather
than an external solver or host-environment trust expansion.

Task 27 must keep the wrapper narrow:

- do not expose `batsat` types in the public `mizar-kernel` API;
- do not call `batsat` DRAT/proof, theory, statistics-printing, logging, model
  enumeration, or callback surfaces;
- do not call `batsat` DIMACS parsing/printing, model printing,
  `print_stats`, or any file/string parser path; the wrapper constructs the
  solver directly from the kernel-derived `sat_encoding` data structure;
- do not call unsafe `bit-vec` APIs from kernel code.

Residual risk: `batsat` memory-safety correctness is part of the trusted SAT
checker dependency. The risk is accepted for this correction because the
dependency is small, in-process, pure Rust, MIT-licensed, process-free, and
closer to MiniSat compatibility than the rejected alternatives. Any future
upgrade requires a fresh audit.

## Process, Network, Time, And Randomness Audit

The selected dependency path must not create an external solver/process trust
edge.

Published-source grep for the `batsat` library found no use of:

- `std::process` or `Command`
- `std::net`, `TcpStream`, or `UdpSocket`
- `std::fs`, `File`, or `OpenOptions`
- `rand`, `thread_rng`, or random APIs
- `std::time`, `Instant`, or wall-clock timeout APIs

`batsat` does contain deterministic, seeded pseudo-random heuristic controls
inside `SolverOpts`: `random_var_freq`, `random_seed`, `rnd_pol`, and
`rnd_init_act`. The default source values are deterministic (`random_var_freq =
0.0`, fixed `random_seed`, `rnd_pol = false`, and `rnd_init_act = false`), but
task 27 must construct the solver with explicit options that pin those values
or reject any wrapper API that would let callers vary them. These controls are
not host random-state reads, but they are still solver heuristics and must stay
outside the trusted input/evidence schema.

`bit-vec` library source likewise has no process, network, filesystem,
host-random, or wall-clock API use. Its benchmark/dev material references
`rand`, which is not part of the production dependency path.

Task 27 must keep the SAT wrapper independent of wall-clock time. Resource
limits are deterministic count/size limits only.

## Determinism And Wrapper API

Task 27 should use the direct `batsat` API behind the `sat_checker` wrapper.
The public kernel-facing shape is refined by task 27 in
[sat_checker.md](./sat_checker.md):

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

Acceptance mapping:

- `batsat::lbool::FALSE` on the complete kernel-derived problem maps to
  `SatCheckResult::Unsat`.
- `batsat::lbool::TRUE` maps to `SatCheckResult::Sat` and is never acceptance.
- `batsat::lbool::UNDEF`, wrapper interruption, unsupported encoding shapes,
  solver errors, internal inconsistencies, and limit exhaustion map to
  `SatCheckResult::Rejected(reason)`.

The wrapper must not expose:

- model enumeration;
- assumptions or unsat-core extraction as trusted material;
- DRAT/proof production;
- DIMACS parsing/printing or model/statistics printing;
- backend profile names;
- solver command lines;
- proof-search configuration knobs;
- premise minimization.
- solver heuristic knobs, including `random_var_freq`, `random_seed`, `rnd_pol`,
  `rnd_init_act`, restart policy, and phase-saving settings.

## Resource Policy

The trusted wrapper must reject before constructing the solver when deterministic
input limits are exceeded:

- variable count;
- clause count;
- total literal count;
- maximum clause width;
- canonical SAT input byte length.

`batsat` exposes conflict/propagation counters and callback-based interruption,
but it does not expose a stable public setter for exact conflict or propagation
budgets. Task 27 selects the unsupported-step-budget branch of this audit:
it exposes only deterministic input limits and rejects any non-`None`
conflict or propagation budget before constructing the solver.

No path may fall back to wall-clock timeouts. Limit exhaustion is always
non-acceptance.

## Rejected Candidates

| Candidate | Version inspected | Reason not selected |
|---|---:|---|
| `varisat` | `0.2.2` | MIT/Apache-2.0 and functionally close, but the published build script probes `drat-trim` and `rate` external commands. It also has a larger transitive dependency tree and unsafe code. |
| `splr` | `0.17.2` | MPL-2.0 license and default `unsafe_access` feature add policy and trusted-boundary complexity. |
| `sat-solver` | `0.2.1` | MIT, but production dependencies include CLI/allocator/walkdir surfaces; source contains file/time/random APIs and broad unsafe usage. |
| `screwsat` | `2.1.5` | MIT and dependency-light, but library source uses wall-clock `Instant`/`Duration` timeout APIs and broad unsafe code. |
| `oxiz-sat` | `0.2.3` | Large solver surface with parallel/GPU/portfolio-related modules and default feature complexity; too broad for the kernel wrapper. |
| `microsat` | `0.0.1` | GPL-3.0 license is incompatible with the intended dependency policy. |
| `rsat` | `0.1.12` | MIT, but includes stochastic local search, `rand`, `rayon`, file input, and an unstable pre-1.0 API surface. |
| `rustsat-batsat` | `0.7.5` | MIT and viable as an adapter, but it pulls the broader RustSAT interface, including public external-solver process/file/tempfile surfaces. Direct `batsat` keeps the trusted dependency smaller. |

## Dependency And Lint Policy Revision

Task 27 must revise the task-1 dependency guard from:

```text
mizar-core
mizar-session
```

to the exact production dependency set:

```text
batsat = { version = "=0.6.0", default-features = false }
mizar-core = { path = "../mizar-core" }
mizar-session = { path = "../mizar-session" }
```

The guard must continue to reject dev/build/target dependency sections unless a
later task explicitly authorizes them. It must also guard that no alternate SAT
checker crate, RustSAT adapter, external solver wrapper, ATP crate,
proof/cache/artifact crate, or process-spawning dependency is added to
`mizar-kernel`.

Task 27 integrates the public `sat_checker` source module and this audit must
remain aligned with the exact dependency shape exposed by that wrapper. No
caller-facing API may expose `batsat` or `bit-vec` types.

## Failure Mapping

The wrapper separates solver evidence outcomes from rejection conditions:

| Wrapper condition | Kernel detail |
|---|---|
| derived SAT problem is satisfiable | `SatCheckResult::Sat(SatCheckReport)`; non-acceptance wrapper evidence |
| dependency returns `UNDEF` without an accepted UNSAT result | `invalid_sat_refutation` or deterministic budget detail, depending on the recorded cause |
| input count/size limit exceeded before solving | `resource_exhaustion` |
| unsupported conflict/propagation step-budget request | `resource_exhaustion` before solver construction |
| unsupported clause/literal shape after kernel derivation | `invalid_sat_refutation` |
| dependency panic caught by the wrapper, internal inconsistency, or unexpected API result | `invalid_sat_refutation`; never acceptance |

Task 27 returns wrapper evidence only. `SatCheckResult::Unsat` is necessary
for later acceptance, but task 28 owns wiring it into the kernel check service
and normal proof-policy acceptance.

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| KERNEL24-G001 | `deferred` | `batsat` has no public exact conflict/propagation budget setter. | Task 27 exposes only supported input limits and rejects unsupported step-budget requests before solver construction. |
| KERNEL24-G002 | `source_drift` resolved by task 27 | Task 27 integrates `sat_checker` source, the exact `batsat` manifest dependency, lockfile guards, and wrapper tests. | Keep the dependency and lockfile lint guards exact; future upgrades require a fresh audit. |
| KERNEL24-G003 | `external_dependency_gap` | No active ATP producer yet emits formula/substitution evidence candidates for the new pipeline. | Keep kernel tests synthetic and do not add producer placeholders. |
| KERNEL24-G004 | `deferred` | `batsat` exposes deterministic pseudo-random heuristic options even though it does not read host random state. | Task 27 pins all `SolverOpts` fields to audited defaults and does not expose heuristic knobs to callers. |
