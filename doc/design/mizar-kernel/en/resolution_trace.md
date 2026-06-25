# Module: resolution_trace

> Canonical language: English. Japanese companion:
> [../ja/resolution_trace.md](../ja/resolution_trace.md).

## Purpose

The `resolution_trace` module owns deterministic replay of the
MiniSAT-compatible resolution steps carried by a normalized kernel
certificate. It refines
[architecture 15](../../architecture/en/15.kernel_certificate_format.md)
"Resolution Trace" and consumes only certificate data already parsed by
`certificate_parser`, normalized clauses owned by `clause`, and shared
rejection records owned by `rejection`.

Resolution replay is evidence checking, not solving. A successful replay proves
only that the listed steps follow from their listed parents; final proof
acceptance remains owned by the later `checker` module.

## Trust Statement

This module is trusted kernel code. It must replay explicit resolution steps in
certificate order, recompute each claimed resolvent, and fail closed when a
step does not match.

The module must not perform SAT solving, ATP search, proof search, premise
selection, overload resolution, cluster search, implicit coercion insertion,
fallback inference, imported-fact discovery, cache lookup, artifact lookup,
wall-clock or random-state reads, unordered iteration, or hidden reads of
mutable compiler-global state. It must not repair a trace by trying alternate
parents, alternate pivots, alternate generated clauses, or backend-reported
used axioms.

Task 20 audits this trust boundary as including no proof search, no SAT
solving, no ATP search or backend invocation, no premise selection, no overload
resolution, no cluster search, no implicit coercion insertion, no fallback
inference, no acceptance from backend-reported success alone, no source
loading, no cache lookup, no artifact lookup, no wall-clock or random-state
reads, no unordered iteration dependence, and no hidden reads of mutable
compiler-global state.

## Owned Behavior

The module owns:

- replaying resolution steps in parsed certificate order;
- resolving parent references against explicit generated, imported, and
  previously checked step clauses;
- checking pivot polarity and parent occurrence;
- recomputing and canonicalizing the resolvent using the `clause` module;
- comparing the recomputed resolvent with the referenced generated clause;
- recording checked step clauses for later steps and checker orchestration;
- enforcing deterministic replay-count and replay-size limits;
- mapping replay failures to stable `rejection` records.

The module does not own:

- normalized certificate byte parsing or structural reference validation;
- construction of backend-specific MiniSAT traces from ATP proof formats;
- imported fact availability, content fingerprint, or proof-status validation;
- substitution, alpha-conversion, free-variable, derived-fact, or cluster trace
  checking;
- proof-policy projection, witness storage, cache reuse, or artifact emission;
- selecting a winning proof candidate among multiple backend results.

## Input And Context

Task 9 should implement replay from explicit immutable inputs:

```text
ResolutionTraceInput
  target_vc_fingerprint
  parsed_certificate
  imported_clause_context
  replay_limits
```

`target_vc_fingerprint` is caller-owned and is copied only into stable
rejection records or private report binding checks. It is not derived from
backend output, cache state, artifact state, or mutable compiler-global state.

`parsed_certificate` is a `certificate_parser::ParsedCertificate`. The parser
has already checked section order, stable id uniqueness, parent reference
shape, generated-clause reference existence, self/forward resolution-step
references, and final-goal reference shape. The replay checker may assert these
in debug-oriented tests, but it must not duplicate byte parsing.

`imported_clause_context` is caller-supplied immutable data:

```text
ImportedClauseContext
  imported_axiom_clauses: sorted map imported_fact_id -> Clause
  imported_theorem_clauses: sorted map imported_fact_id -> Clause
  provenance_fingerprint
```

The concrete Rust type may avoid map dependencies by using sorted vectors, as
long as lookup and iteration are deterministic. The context is not populated
from resolver state, ATP output, cache state, artifact state, or global
compiler state. Missing imported-clause context is `missing_provenance`.
The implementation must make imported context ordering deterministic by using a
constructor or type invariant that stores each imported namespace as sorted
unique ids. It may canonicalize input order, but duplicate ids are invalid
context shape and must be rejected deterministically before replay.
If the context exists but does not contain the parsed imported namespace/id
needed by a parent reference, task 9 must also return `missing_provenance`
because resolution replay lacks the immutable clause evidence required to
check the step. Unavailable or fingerprint-mismatched imported facts are owned
by the later `checker` imported-fact task and map to `unresolved_symbol` there
before or around replay orchestration.

Supplied imported `Clause` values used by the parsed trace must already be
normalized for the same profile, symbol manifest, and variable manifest as the
replay-derived context. Task 9 validates imported clauses at first use in
deterministic trace order, not by scanning and rejecting unused context entries.
The implementation checks profile agreement cheaply, then validates the used
imported clause through the same bounded parent `ClauseValidationContext` used
for generated and previously checked step parents before cloning it for replay.
Non-resource profile, symbol, variable, or canonical-form incompatibility is
`missing_provenance`, not an opportunity to repair or reinterpret the parent.
Parent literal count, term-size, term-depth, and canonical-byte budget failures
remain replay resource checks and map to `resource_exhaustion`.

`replay_limits` are deterministic:

```text
ResolutionReplayLimits
  max_checked_steps
  max_parent_literals
  max_resolvent_literals
  max_resolvent_canonical_bytes
  max_term_encoding_bytes
  max_term_recursion_depth
```

Exceeding a limit is `kernel_rejection` with `resource_exhaustion`. A budget
must be checked before allocating a large temporary resolvent.

Replay normalization must use a `ClauseValidationContext` derived only from
public parsed certificate data and replay limits:

- `ClauseProfile` is built from `ParsedCertificate.kernel_profile`
  `clause_schema_version`, `clause_encoding_version`, and
  `clause_tautology_policy`;
- allowed and known symbols come from `ParsedCertificate.symbol_manifest`;
- canonical variables come from `ParsedCertificate.variable_manifest`;
- literal, term-size, and term-recursion/depth limits come from
  `ResolutionReplayLimits`, and the depth limit applies to caller-supplied
  imported clauses as well as parsed/generated clauses.

If task 9 needs a helper for this derivation, it may add a small public or
crate-private helper in `certificate_parser`, but it must not add global lookup,
downstream crate dependencies, or a second parser.

Canonical-byte limit accounting must use clause-owned non-allocating helpers.
If the current `clause` API does not expose enough information to compute
literal or clause canonical lengths without allocating canonical byte vectors,
task 9 must add a small public or crate-private length/bounded-writer helper to
`clause`. The resolution checker must not duplicate the clause encoder or
allocate canonical bytes merely to discover that a replay limit was exceeded.

Imported-clause validation must also be depth-bounded. If the current `clause`
API cannot validate borrowed `Term` values with an explicit recursion-depth
budget, task 9 must add a clause-owned depth-bounded validation helper or extend
the clause validation context. The resolution checker must not recursively walk
caller-supplied imported terms without a deterministic depth budget.

## Clause Reference Resolution

The checker accepts the same clause-reference namespaces parsed from the
certificate:

| Namespace | Replay source |
|---|---|
| `generated_clause` | A generated clause in the parsed certificate. |
| `resolution_step` | The checked clause produced by an earlier replayed step. |
| `imported_axiom` | A normalized clause supplied by `imported_clause_context`. |
| `imported_theorem` | A normalized clause supplied by `imported_clause_context`. |

Resolution-step parents must refer only to earlier checked steps. The parser
rejects self and forward references as malformed certificates; task 9 tests
should still cover that replay never consults an unchecked future step.

The checker must not synthesize a clause for a missing reference, scan
alternate namespaces, consult imported facts by display name, or accept a
backend-provided used-axiom list as a parent table.

## Replay Algorithm

For each `ResolutionStep` in certificate order:

1. Check replay limits for the step count and both parent clause sizes.
2. Look up `parent_a` and `parent_b` from the explicit reference sources.
3. Check that `pivot_literal` occurs exactly by canonical literal identity in
   `parent_a`.
4. Check that the same atom with opposite polarity occurs in `parent_b`.
5. Compute an allocation-free upper bound for the resolvent literal count from
   the parent literal counts after the matched pivots are removed. Reject before
   allocation if that bound exceeds `max_resolvent_literals`.
6. Build the raw resolvent through a bounded accumulator from all literals of
   `parent_a` except the pivot and all literals of `parent_b` except the
   opposite-polarity pivot. The accumulator checks literal count and
   canonical-byte total before each push and stops before growing past any
   replay limit.
7. Normalize the bounded raw resolvent with the derived
   `ClauseValidationContext`.
8. Compare the normalized resolvent with `generated_clause`.
9. Record the checked step id and normalized resolvent for later steps.

The parent orientation is semantic: `parent_a` must contain the pivot as
encoded, and `parent_b` must contain the opposite-polarity literal. A producer
that wants the opposite orientation must swap parents or encode the pivot with
the opposite polarity. The checker must not silently swap parents.

Comparison is structural over normalized clause values and canonical bytes. It
must not use rendered text, display names, source ranges, backend logs,
allocation addresses, hash-map iteration order, or worker completion order.

## Final-Goal Interaction

The resolution checker may report the checked clause for every replayed step to
the later `checker` module. It does not by itself produce trusted proof
acceptance.

Task 9's success report should expose deterministic checked-step data only:

```text
ResolutionReplayReport
  checked_steps: sorted Vec<CheckedResolutionStep>

CheckedResolutionStep
  step_id
  generated_clause_id
  clause
```

The report is evidence-replay output for later checker orchestration. It must
not contain an accepted proof status, used-axiom projection, policy outcome, or
artifact-facing witness decision. The implementation may carry private replay
binding data, such as the caller-owned target fingerprint and certificate hash
input, solely to reject accidental pairing of a report with a different replay
input. Accessors must still expose only checked-step data.

When task 9 includes a helper that validates a final goal in the
`generated_clause` or `resolution_step` namespace, that helper must require the
referenced clause to be checked by successful replay. A `resolution_step`
reference is checked only after that step has replayed successfully. A
`generated_clause` reference is checked only when the generated clause id is the
claimed output of at least one successfully replayed step. Merely existing in
the parsed `generated_clauses` section is not enough. The checked final-goal
clause must be the profile's `empty` contradiction form unless the later
`checker.md` spec explicitly assigns a different final-goal rule. An unchecked
or non-empty resolution final goal is `invalid_sat_proof`.

`derived_fact` final goals are outside this module and remain deferred to the
later checker orchestration and substitution/cluster-derived fact tasks.

## Rejection Mapping

Replay failures produce `kernel_rejection` records:

| Failure | Detail | Location |
|---|---|---|
| Missing imported-clause context, missing context provenance, imported parent namespace/id absent from the supplied context, or used imported clause incompatible with the replay profile/manifest context | `missing_provenance` | `resolution_step_id` plus the parent `clause_ref` when known |
| Missing generated or earlier-step parent after parsing invariants are broken | `invalid_sat_proof` | `resolution_step_id` plus parent `clause_ref` |
| Pivot absent from `parent_a` | `invalid_sat_proof` | `resolution_step_id`, parent `clause_ref`, and pivot field path |
| Opposite-polarity pivot absent from `parent_b` | `invalid_sat_proof` | `resolution_step_id`, parent `clause_ref`, and pivot field path |
| Recomputed resolvent differs from the referenced generated clause | `invalid_sat_proof` | `resolution_step_id` plus generated `clause_ref` |
| Resolution final goal references an unchecked or non-empty checked clause | `invalid_sat_proof` | `resolution_step_id` or final-goal marker |
| Replay count, parent-size, term-size, term-depth, resolvent-size, or canonical-byte limit exceeded | `resource_exhaustion` | most precise step, parent, or generated-clause location available |

Every rejection location must be deterministic and must use the shared
`RejectionLocation` fields from `rejection.md`. Human diagnostics may include
extra text, but extra text must not affect acceptance, ordering, or stable
detail keys.

## Determinism And Cost

Replay cost must be linear in the declared trace size plus the total literal
payload size of parents and resolvents, within explicit per-step limits. The
checker must use deterministic data structures or sorted vectors for any
temporary index.

The result for identical parsed certificates, imported clause contexts, and
limits must be identical across platforms and worker schedules. Parallel batch
checking is owned by `checker`, but this module must expose outputs whose
ordering is independent of worker completion order.

## Gap Classification

- `spec_gap`: architecture 15 describes high-level resolution replay but not
  the module-owned input context, parent-reference ownership, final-goal helper
  boundary, rejection mapping, or replay limits. This module spec closes that
  gap for task 8.
- `test_gap`: task 9 still needs Rust tests for valid replay, every single-step
  mutation class, imported parent context handling, final-goal helper behavior,
  deterministic outputs, and replay-cost limits.
- `external_dependency_gap`: translation from backend-specific proofs into the
  normalized MiniSAT-compatible trace is owned by future `mizar-atp` work;
  proof-policy projection and witness publication are owned by future
  `mizar-proof`, `mizar-cache`, and `mizar-artifact` work. Do not add placeholder
  integrations in `mizar-kernel`.
- `deferred`: imported fact availability, content-fingerprint validation, and
  required proof-status validation land in later `checker` imported-fact tasks;
  source-derived `.miz` snapshots and expectation sidecars land in the later
  soundness corpus task.

## Planned Tests

Task 9 must add Rust tests for:

- a valid single-step replay deriving the empty clause from two explicit parent
  clauses;
- valid replay using generated-clause, imported-axiom, imported-theorem, and
  earlier resolution-step parents supplied by explicit immutable context;
- pivot absent from `parent_a` rejected as `invalid_sat_proof`;
- opposite-polarity pivot absent from `parent_b` rejected as
  `invalid_sat_proof`;
- swapped parent orientation rejected unless the certificate also swaps the
  parents or pivot polarity explicitly;
- generated-clause mismatch rejected when the recomputed resolvent has an
  extra literal, a missing literal, different polarity, or different canonical
  literal bytes;
- tautology and empty-clause outcomes following the active clause profile;
- missing imported-clause context rejected as `missing_provenance` without
  consulting global state;
- context-present but provenance-missing input rejected as `missing_provenance`;
- imported parent namespace/id absent from the supplied context rejected as
  `missing_provenance`;
- imported context construction canonicalizing sorted input order and rejecting
  duplicate ids deterministically before replay;
- used imported context clauses whose profile, symbols, variables, or canonical
  form do not validate against the replay-derived context rejected as
  `missing_provenance`;
- unused or extra imported context entries ignored by `resolution_trace` replay,
  with exact imported-fact auditing deferred to `checker`;
- broken generated or earlier-step parent invariants rejected as
  `invalid_sat_proof` in defensive constructors or test fixtures;
- resolution final-goal helper accepting only the checked empty clause for
  `generated_clause` or `resolution_step` final goals;
- `generated_clause` final goals rejected when the generated clause exists in
  the parsed section but was not produced by a successfully replayed step;
- `generated_clause` final goals rejected when produced by successful replay
  but the checked clause is non-empty;
- `resolution_step` final goals rejected when the step is unchecked or the
  checked step clause is non-empty;
- every replay rejection asserting `kernel_rejection`, stable detail key,
  caller-owned target fingerprint, and the most precise deterministic
  `RejectionLocation` fields promised by the rejection table;
- replay count, parent literal, term encoding, term recursion depth, resolvent
  literal, and resolvent byte limits rejected as `resource_exhaustion` before
  large allocation or deep recursion;
- deeply nested imported context terms rejected as `resource_exhaustion` through
  a clause-owned depth-bounded validation path, not by stack overflow or panic;
- replay context derivation from parsed kernel profile, symbol manifest,
  variable manifest, literal limit, term encoding limit, and term-depth limit,
  using only public parsed data or an explicit crate-private helper;
- clause-owned non-allocating canonical length or bounded-writer helper used for
  canonical-byte accounting, with no duplicated clause encoder;
- bounded accumulator behavior showing oversized resolvents are rejected before
  collecting an unbounded raw literal vector;
- deterministic checked-step output and rejection ordering under shuffled test
  fixture construction or simulated worker completion order;
- success reports exposing checked step ids, generated clause ids, and clauses
  without proof-acceptance or policy-status fields;
- lint coverage showing no SAT solver, ATP/proof/cache/artifact coupling,
  proof search, premise selection, overload resolution, cluster search,
  implicit coercion insertion, fallback inference, unordered iteration,
  wall-clock/random reads, or global mutable-state reads.

No `.miz` fixture, expectation sidecar, `doc/spec`, or Rust source change is
required for this module-spec task.
