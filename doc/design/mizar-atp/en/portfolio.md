# Module: portfolio

> Canonical language: English. Japanese companion:
> [../ja/portfolio.md](../ja/portfolio.md).

## Purpose

The `portfolio` module specifies phase-13 ATP portfolio planning and candidate
collection for one VC that already reached `VcStatus::NeedsAtp`.

The module is an evidence producer and handoff boundary. It may plan backend
runs, collect backend results, normalize candidate ordering, record
reproducibility metadata, and hand candidate evidence to the later kernel and
proof-policy stages. It does not accept a proof, select an artifact-facing
winner, call `mizar-kernel`, publish witnesses, update caches, or turn backend
proof methods, logs, unsat cores, SMT proof objects, TSTP traces, resolution
traces, completion order, or instantiated formulas into trusted acceptance
material.

## Scope

Task 17 is specification-only. It authorizes a future `src/portfolio.rs` module
to implement policy-neutral portfolio planning and candidate collection after
this specification exists. It does not add Rust source, spawn new backend
adapters, parse real backend output, invent formula/substitution evidence, call
the kernel, evaluate proof policy, publish artifact witnesses, or implement
proof-cache promotion.

Task 18 may implement the no-early-stop collection path. Early-stop mechanics
may be implemented only after a stable external proof-policy finality contract
exists; until then, no early stop is the only source-implementation path. It
must not implement `mizar-proof` policy locally. Because `mizar-proof` is not
currently a workspace crate, policy evaluation, witness publication, and proof
cache promotion remain `external_dependency_gap` items.

Task-18 source is limited to deterministic plan construction from prebuilt
`BackendRunInput` values, validation that every run belongs to the same
`AtpProblem`, portfolio and candidate hashes, and collection of terminal
`BackendRunResult` values without early stop. Full backend-profile selection,
encoding orchestration, and scheduler integration remain conceptual/future
scope. Task 18 may honor cooperative cancellation by returning a cancelled
evidence set with no candidates. It does not execute backend processes
directly, add backend adapters, parse real output, build kernel evidence, call
the kernel, infer proof-policy finality, or publish witness/cache/artifact
state.

Task 20 may add an `advanced_semantics` metadata-only corpus fixture under
`tests/property/` and a crate-local integration test that reads that fixture and
drives the existing mock backend, backend classification, and portfolio
collection APIs. Because `mizar-test` has no active `advanced_semantics` runner
or tag gate yet, the corpus sidecar must remain `metadata_only` and must not use
an active tag. The integration test may use only the already implemented mock
classification seam; it must not add `.miz` semantic execution, real backend
output extraction, kernel calls, proof-policy decisions, witness/cache/artifact
publication, or a temporary evidence schema.

Task 21 may add a crate-local determinism integration test that starts from
identical public `VcIr` inputs, rebuilds the public VC kernel handoff, translates
the obligation into `AtpProblem`, encodes it through each implemented concrete
encoder profile, classifies mock formula/substitution candidates, and verifies
portfolio candidate ordering under shuffled backend completion order. The test
is regression coverage only. It must not add a scheduler, real backend output
extraction, kernel calls, proof-policy decisions, artifact witnesses, cache
publication, or any trusted use of backend proof material.

## Inputs And Outputs

The conceptual portfolio API consumes:

```text
PortfolioInput
  portfolio_id
  vc_id
  vc_hash
  atp_problem
  backend_profiles
  encoded_problem_set
  obligation_budget
  scheduler_budget
  proof_hint?
  build_snapshot
  policy_constraints
  cancellation
```

and produces:

```text
PortfolioEvidenceSet
  portfolio_id
  vc_id
  vc_hash
  plan_hash
  backend_results
  candidates
  pending_capabilities
  stop_summary
  diagnostics
  metadata
```

`PortfolioEvidenceSet` is not an accepted proof result. It is a deterministic
evidence set for later kernel checking and proof-policy selection.

## Boundary Rules

The portfolio layer may:

- select configured backend profiles that are allowed by policy constraints,
  source hints, backend availability, logic profile, concrete encoders, and the
  obligation budget;
- request or consume concrete TPTP / SMT-LIB encodings that were already built
  from the same `AtpProblem`;
- dispatch `BackendRunInput` records to the backend runner;
- collect `BackendRunResult` values, candidate evidence refs or payload bytes,
  counterexample diagnostics, stdout/stderr hashes, timing summaries, resource
  observations, and cancellation records;
- build deterministic candidate ids and candidate ordering keys;
- stop remaining backend runs only when cancellation is requested or an
  external proof-policy finality decision says no pending candidate can displace
  the selected policy class.

It must not:

- evaluate proof policy, choose the canonical proof winner, or project artifact
  proof status;
- call `mizar-kernel`, run SAT checking, or derive instantiated formulas;
- select additional premises, invent substitutions, repair binders, resolve
  overloads, search clusters, insert implicit coercions, or perform fallback
  inference;
- classify externally attested backend output as equivalent to kernel-verified
  proof status;
- trust backend-reported `used_axioms`, backend proof methods, proof logs,
  unsat cores, SMT proof objects, TSTP traces, MiniSAT-compatible resolution
  traces, or legacy certificates;
- make raw completion order, wall-clock timing, process id, temporary path, or
  backend output order part of semantic candidate identity;
- publish proof witnesses, cache entries, or artifact proof status.

## Portfolio Planning

Planning is deterministic for equivalent inputs. The plan contains one
`BackendRunInput` per selected backend profile and concrete encoding. Selection
uses only stable inputs:

- `AtpProblem.problem_id`, `vc_id`, `vc_hash`, target binding, logic profile,
  and expected result;
- profile id, backend kind, concrete format, supported observed results,
  evidence formats, deterministic priority, and required resource limits;
- proof hints and policy constraints that are already materialized before the
  portfolio starts;
- backend availability records and configured executable identities;
- explicit obligation and scheduler budgets.

The plan must reject or skip profiles that cannot consume the selected logic
profile or concrete encoding. A profile that can only produce externally
attested or diagnostic output may be scheduled only when the policy constraints
allow such evidence to be recorded; it still cannot become kernel-verified
inside `mizar-atp`.

Task 18 implements only the already-built run slice of planning: callers supply
the `BackendRunInput` records, and the portfolio validates same-problem
membership, deterministic order, and budget/cancellation gates. Selecting
profiles from availability, constructing encodings, and dispatching scheduler
work remain outside task 18.

If no profile is schedulable, the portfolio returns an open evidence set with a
stable diagnostic reason. It must not fabricate a backend result or proof
candidate.

## Candidate Model

A portfolio candidate is a normalized record derived from one backend result:

```text
PortfolioCandidate
  candidate_id
  source_run_id
  backend_profile_id
  encoded_problem_hash
  target_binding
  candidate_kind
  evidence_format
  evidence_payload_or_ref?
  counterexample_ref?
  observed_result?
  provenance_hash
  candidate_hash
  diagnostics
```

Candidate kinds are policy-neutral:

- `FormulaSubstitution`: formula/substitution evidence candidate compatible
  with the kernel-owned schema, once the evidence-extraction route exists;
- `ExternallyAttested`: backend evidence that policy may record but that is not
  kernel acceptance;
- `Counterexample`: diagnostic model or counterexample evidence;
- `Unknown` / `Error`: no proof candidate, only diagnostics.

`FormulaSubstitution` candidates are still untrusted until `mizar-kernel`
checks them. `ExternallyAttested` candidates are never silently upgraded to
kernel-verified status. Backend proof logs may be retained as diagnostics or as
inputs to a future extractor, but the candidate evidence handed onward must be
formula/substitution evidence bytes or refs plus target binding and provenance.

## Deterministic Ordering And Identity

The portfolio builds stable hashes with private, length-prefixed canonical
fields and explicit domain tags:

| Hash | Domain | Required fields |
|---|---|---|
| `plan_hash` | `mizar-atp/portfolio-plan/v1` | `vc_hash`, `AtpProblem.problem_id`, selected profile ids, concrete input hashes, policy-constraint fingerprint, and budget records |
| `candidate_hash` | `mizar-atp/portfolio-candidate/v1` | candidate kind, evidence format, candidate payload/ref hash, target binding, provenance hash, encoded problem hash, backend profile id, and observed result |
| `evidence_set_hash` | `mizar-atp/portfolio-evidence-set/v1` | `plan_hash`, sorted backend-result metadata hashes, sorted candidate hashes, and stop summary |

Candidate ordering is independent of raw completion order. The canonical order
is:

1. candidate kind tag (`FormulaSubstitution`, `ExternallyAttested`,
   `Counterexample`) as a handoff grouping, without evaluating proof policy;
2. backend profile deterministic priority;
3. evidence format priority;
4. encoded problem hash;
5. candidate hash;
6. backend profile id;
7. source run id.

This order is for reproducible candidate handoff only. It is not the
artifact-facing winner order, and it must not override `mizar-proof` policy.

## Public Enum Forward Compatibility

Task 22 applies the frontend task-25 policy to the `portfolio` module. Public
enums owned here are `#[non_exhaustive]` for downstream crates:
`PortfolioCandidateKind`, `PortfolioEvidenceFormat`, `PortfolioStopReason`,
and `PortfolioError`.

Public enum inventory: `PortfolioCandidateKind`, `PortfolioEvidenceFormat`, `PortfolioStopReason`, `PortfolioError`.

Future candidate kinds, evidence formats, stop reasons, or error variants must
be specified before source uses them. Inside `mizar-atp`, matches that affect
candidate ordering, evidence-set identity, cancellation, result matching, or
proof status must be explicit and fail closed unless a paired spec documents an
intentional fallback. New candidate or evidence classes stay untrusted until
their owning kernel/proof-policy contract exists.

## Early Stop And Cancellation

The portfolio may stop remaining backend processes only in these cases:

- caller cancellation supersedes the build snapshot;
- the obligation or scheduler budget is exhausted;
- an external proof-policy finality decision states that no pending candidate
  can displace the selected policy class under the active policy.

Without an external finality decision, the safe default is to collect every
scheduled backend result until it reaches a terminal status or cancellation is
requested. `mizar-atp` must not infer policy finality from backend completion
order, backend priority alone, externally attested success, or the presence of a
candidate that has not yet been checked by the kernel.

Cancellation is cooperative for in-process portfolio work. Child backend
processes are terminated through the backend runner. Cancelled runs leave
diagnostic metadata but never partial accepted proof state.

## Result Matching

For the no-early-stop task-18 path, backend results must match the planned run
set bijectively before candidates are handed onward:

- planned run ids must be unique;
- every non-cancelled planned run must have exactly one terminal result;
- unknown result run ids, duplicate results, missing results, and mismatched
  result metadata fail closed as `PortfolioError` before any
  `PortfolioEvidenceSet` is emitted;
- result metadata must match the planned run's run id, problem id, input hash,
  metadata hash, command fingerprint, backend kind, and profile id;
- candidate metadata mismatches fail closed before that candidate is included.

When cancellation is observed, task 18 may collect any already-returned
matching results for diagnostics, but it must emit no candidates and must record
a cancelled stop summary. This is cancellation handling, not early-stop policy
finality.

## Resource Budgets

The portfolio records both the obligation-level budget and the per-backend
budget assigned to each run. Budget assignment is deterministic and based on
stable profile configuration and explicit input budgets. Timeout, memory,
process-count, stdout/stderr, and temporary-file limits are forwarded to the
backend runner as required or best-effort limits.

Unsupported required limits make the affected run an error before a `Proved`
candidate can be constructed. Budget exhaustion leaves the obligation open or
diagnostic; it does not create trusted proof status.

## Kernel And Policy Handoff

The portfolio hands off:

- formula/substitution candidates to the kernel check scheduler, when such
  candidates exist and the external policy says kernel-checked evidence can be
  useful;
- externally attested or diagnostic records to proof policy and diagnostics
  without treating them as accepted proof material;
- reproducibility metadata, hashes, and stdout/stderr refs for artifact and
  cache layers to consume after their owning crates define stable contracts.

The portfolio does not construct `KernelCheckResult`, `ProofWitnessDraft`,
trusted `used_axioms`, artifact proof selections, or proof-cache entries.
Backend-reported used axioms remain advisory until validated by the kernel.

## Failure Semantics

Each backend failure is local to its run:

| Condition | Portfolio handling |
|---|---|
| no schedulable profile | open evidence set with deterministic diagnostic |
| timeout or budget exhaustion | terminal run status plus diagnostic; other useful runs may continue |
| process crash or spawn failure | backend error diagnostic; no verifier crash |
| malformed backend output | unknown or error; no `FormulaSubstitution` candidate unless extraction succeeds |
| candidate metadata mismatch | fail closed with `PortfolioError` before emitting a `PortfolioEvidenceSet` |
| kernel rejection reported later | candidate-specific proof error; portfolio evidence set remains reproducible |
| policy rejection reported later | policy error distinct from backend and kernel rejection |

An all-failed portfolio is an open proof obligation, not an accepted proof.

## Gap Classification

- resolved `deferred` spec gap: task 17 specifies portfolio planning,
  candidate collection, deterministic ordering, early-stop constraints, budgets,
  and handoff boundaries before `src/portfolio.rs` exists.
- task-18 source scope: no-early-stop deterministic planning and candidate
  collection over prebuilt backend runs/results, plus cancellation and
  fail-closed validation. It does not implement proof policy, kernel checks,
  real-output evidence extraction, witness publication, or cache promotion.
- `external_dependency_gap`: `mizar-proof` is not a workspace crate, so proof
  policy finality, artifact-facing winner selection, and witness publication
  cannot be implemented here.
- `external_dependency_gap` / `deferred`: first real-backend formula/substitution
  extraction remains blocked by ATP-G-015. Task 18 must use existing mock
  candidates or already-specified candidate inputs; it must not invent a fake
  real-output schema.
- `external_dependency_gap`: proof witness storage, artifact projection, and
  proof-cache promotion remain outside `mizar-atp`.
- `external_dependency_gap`: active `advanced_semantics` corpus execution is not
  available in `mizar-test`; task 20 therefore records corpus intent with
  metadata-only sidecars and exercises the ATP path through crate-local mock
  backend integration tests.

## Task-18 Test Coverage

Task 18 adds Rust coverage for:

- deterministic portfolio planning under shuffled backend availability and
  profile input order;
- identical candidate ordering under shuffled backend completion order;
- no-early-stop collection when no external policy finality decision exists;
- honoring explicit cancellation without leaving partial accepted proof state;
- proving that the implementation does not fabricate an early-stop oracle while
  the stable external policy finality contract is absent;
- timeout, crash, malformed output, unsupported-limit propagation,
  same-problem validation, stopped-plan result matching, and result/candidate
  metadata mismatch from backend results;
- absence of kernel calls, proof policy evaluation, witness/cache publication,
  accepted proof status, trusted backend proof material, caller-supplied
  instantiated formulas, and SAT problems from the portfolio API.

## Task-20 Corpus And Mock-Backend Coverage

Task 20 adds coverage that binds the `advanced_semantics` corpus manifest to the
mock backend integration path:

- the committed fixture is metadata-only, uses `stage = "advanced_semantics"`,
  and is linked from `tests/coverage/spec_trace.toml`;
- the crate-local integration test reads the fixture, builds deterministic
  prebuilt backend runs for its cases, executes the mock backend runner, applies
  mock observed-result classification, and collects portfolio evidence;
- fixture cases cover a formula/substitution candidate, a counterexample, and an
  unknown/open result without turning any backend observation into trusted
  acceptance material;
- the suite asserts deterministic candidate handoff and metadata-only corpus
  boundaries, while leaving active `.miz` advanced-semantic execution,
  real-output extraction, kernel checking, proof policy, artifact witnesses, and
  cache promotion deferred to their owning tasks.

## Task-21 Determinism Suite

Task 21 adds cross-module determinism coverage for the already implemented
candidate-production path:

- identical `VcSet` / `VcIr` fixtures constructed through public `mizar-vc`
  APIs produce identical kernel handoffs, `AtpProblem` ids, and debug
  renderings for each concrete encoder profile;
- both implemented encoders are exercised explicitly: TPTP FOF for the FOF
  profile and SMT-LIB uninterpreted for the SMT profile;
- emitted encoder text, formula labels, and symbol-binding side metadata are
  byte-identical for identical inputs;
- mock formula/substitution candidates with matching target binding,
  encoded-problem hash, provenance hash, formula labels, and symbol bindings
  are classified and collected without becoming trusted acceptance material;
- portfolio plan hashes, candidate ids, candidate hashes, evidence-set hashes,
  and candidate ordering remain identical when planned run input order and
  backend completion order are reversed.

The suite intentionally leaves real backend extraction, kernel checking,
proof-policy winner selection, artifact witnesses, proof-cache promotion, and
active `.miz` advanced-semantics execution deferred to their owning tasks.
