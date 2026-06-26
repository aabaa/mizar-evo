# Module: backend

> Canonical language: English. Japanese companion:
> [../ja/backend.md](../ja/backend.md).

## Purpose

The `backend` module specifies how `mizar-atp` invokes one external ATP/SMT
backend for one already encoded `AtpProblem`. It owns child-process execution,
resource limits, captured-output metadata, backend version recording, graceful
timeout/cancellation handling, and policy-neutral backend-result
classification.

The backend module is still an evidence producer boundary. It does not accept
proofs, call `mizar-kernel`, run SAT checking, publish witnesses, select
portfolio winners, update caches, or turn backend proof methods, logs, unsat
cores, SMT proof objects, TSTP traces, or resolution traces into trusted
acceptance material.

## Scope

Task 13 is specification-only. It authorizes future source modules to expose a
backend runner API, a mock-backend test harness, deterministic run metadata,
and fail-closed result classification. It does not add Rust source, spawn
processes, integrate a real backend, parse backend proof languages, extract
formula/substitution candidates from a real backend, call the kernel, publish
artifact witnesses, run a portfolio, or implement proof policy.

Task-14 source may implement the generic process runner and mock backend
fixtures described here. Real backend adapters, backend-specific output
parsers, candidate evidence extraction, and backend-available integration
tests remain task-15 work. Broad classification fixtures and polarity
validation remain task-16 work, although task 14 may expose constructors and
invariants that make invalid `Proved` results unrepresentable.

## Inputs And Output

The conceptual task-14 API consumes:

```text
BackendRunInput
  run_id
  encoded_problem
  backend_profile
  command
  resource_limits
  io_mode
  cancellation
```

and produces:

```text
BackendRunResult
  run_id
  encoded_problem_hash
  backend_identity
  command_fingerprint
  status
  observed_result?
  candidate_evidence?
  counterexample?
  stdout_hash
  stderr_hash
  stdout_excerpt?
  stderr_excerpt?
  exit_status?
  termination
  timing
  resource_observations
  diagnostics
```

`encoded_problem` is concrete backend input plus the metadata needed after
execution: source `AtpProblem.problem_id`, target binding, expected result,
logic profile, concrete format, formula labels/assertion labels, symbol
bindings, provenance hash, and semantic input hash. The backend runner must
not accept caller-supplied instantiated formulas, SAT clauses, proof methods,
or backend `used_axioms` as trusted fields.

## Encoded Problem Contract

The runner consumes immutable encoded input produced by a concrete encoder:

| Field | Requirement |
|---|---|
| `problem_id` | The source `AtpProblem` identity. It ties backend metadata back to a backend-neutral problem but is not proof acceptance. |
| `target_binding` | Stable target binding carried through to candidate evidence. Mismatches fail closed before a `Proved` result is built. |
| `expected_result` | The backend success polarity, currently `ExpectedBackendResult::Unsat`. |
| `concrete_format` | TPTP or SMT-LIB profile selected by the encoder. |
| `input_text` | The exact backend input bytes. |
| `input_hash` | Hash of the exact backend input bytes and encoding profile. |
| `symbol_bindings` / `formula_labels` | Encoder side metadata used by later candidate extraction. |
| `provenance_hash` | Stable hash over ATP provenance metadata. |

The backend runner records `input_hash` and metadata hashes; it must not
rewrite the input, normalize backend text, add proof commands, request unsat
cores as trusted data, or infer a different expected result.

## Backend Profile And Command

`BackendProfile` is a deterministic configuration record:

```text
BackendProfile
  profile_id
  backend_kind
  concrete_format
  supported_observed_results
  candidate_evidence_formats
  version_probe
  default_args
  requires_candidate_evidence
  deterministic_priority
```

`BackendCommand` is a single executable invocation, not a shell string:

```text
BackendCommand
  executable
  args
  environment_policy
  working_directory_policy
  stdin_policy | problem_file_policy
  random_seed?
```

Task 14 must pass arguments directly to process-spawn APIs and must not invoke a
shell to interpret backend input or profile-provided command text. Environment
variables are allowlisted, sorted, and recorded by stable key. Temporary
directories are private to the run and are deleted after the process exits or
is killed. Absolute executable paths may be recorded in diagnostic metadata
when configured, but machine-local paths must not participate in semantic
problem identity.

## Process Model

The runner launches one child process per `BackendRunInput`.

Required behavior:

- provide input through stdin or a private temporary problem file according to
  `io_mode`;
- capture stdout and stderr up to configured byte limits;
- hash complete captured stdout/stderr or record that a capture limit truncated
  the stream;
- record exit code or platform signal/termination detail when available;
- record start/end monotonic timing, elapsed time, timeout budget, and kill
  grace duration;
- apply best-effort CPU, wall-clock, memory, process-count, stdout/stderr, and
  temporary-file limits exposed by the host platform;
- terminate the child process on timeout, cancellation, or portfolio stop;
- wait/reap the process after termination so task-14 tests can assert that no
  child process is left behind;
- classify spawn failure, version-probe failure, timeout, cancellation,
  non-zero crash, and malformed output without panicking the verifier.

Resource-limit mechanisms are platform-dependent. If a limit cannot be
enforced on the current platform, the runner records an unsupported-limit
diagnostic. A policy may later decide that unsupported enforcement is an error,
but the backend runner itself must not fabricate proof status because a limit
was unavailable.

## Backend Identity And Reproducibility

The runner records:

- backend kind and profile id;
- executable identity and command fingerprint;
- version probe command, version stdout/stderr hashes, and parsed version when
  available;
- selected concrete format and encoded input hash;
- normalized arguments, sorted allowlisted environment, working-directory
  policy, input-delivery mode, random seed, and resource limits;
- exit status, termination class, stdout/stderr hashes, timing, and resource
  observations.

`command_fingerprint` is deterministic and excludes process ids, temporary
paths, wall-clock timestamps, raw completion order, and machine-local absolute
paths unless a later config spec explicitly opts into path-sensitive
reproducibility. Diagnostic renderings may include local paths when useful, but
semantic hashes and proof-reuse identities must not depend on them.

## Result Classification

The backend runner distinguishes process status, observed backend result, and
candidate evidence availability.

```text
BackendRunStatus
  Proved
  Counterexample
  Timeout
  Unknown
  Error
  Cancelled
```

`Proved` is a candidate-evidence status only. It is not kernel acceptance and
must never be projected directly to artifact proof status.

`Proved` may be constructed only when all conditions hold:

1. the process completed without timeout, cancellation, crash, or capture-limit
   corruption that invalidates parsing;
2. backend output was parsed as an observed result matching
   `encoded_problem.expected_result`;
3. for the current interface, that means an unsatisfiable/refutation/theorem
   result corresponding to `ExpectedBackendResult::Unsat`;
4. candidate formula/substitution evidence is present in a supported candidate
   format;
5. the candidate evidence target binding, input hash, formula labels or
   assertion labels, symbol bindings, and provenance hash match the encoded
   problem metadata;
6. no backend proof method, backend log, unsat core, SMT proof object,
   resolution trace, TSTP trace, or backend-reported `used_axioms` is used as
   trusted acceptance material.

If the observed result matches the expected result but no supported candidate
formula/substitution evidence is present, the result is `Unknown` or `Error`
with a missing-evidence diagnostic, never `Proved`. If the backend reports
`sat`, `counter-satisfiable`, model data, or a counterexample, the result may be
`Counterexample` only when provenance mapping succeeds; otherwise it is
`Unknown` or `Error`. `unknown`, timeout, cancellation, malformed output, parse
failure, unsupported observed status, and polarity mismatch are never `Proved`.

Task 14 may expose these invariant checks and mock classifications. Task 15
adds the first real backend extractor. Task 16 adds full outcome and polarity
classification fixtures for real backend-style outputs.

## Candidate Evidence Boundary

Candidate evidence records are untrusted extraction outputs:

```text
BackendCandidateEvidence
  candidate_id
  schema_family
  payload_ref_or_bytes
  target_binding
  encoded_problem_hash
  provenance_hash
  formula_label_refs
  symbol_binding_refs
  extraction_diagnostics
```

The payload must be formula/substitution evidence compatible with the
kernel-owned schema. Backend proof objects and logs may be diagnostic inputs to
an extractor, but the candidate payload handed to later kernel checking must
not be a backend proof method, SMT proof object, unsat core, resolution trace,
or legacy certificate. Kernel acceptance, trusted `used_axioms`, proof witness
drafts, artifact status, and cache promotion belong to downstream tasks/crates.

## Failure Semantics

- `Timeout`: the timeout budget elapsed and the child process was terminated or
  confirmed no longer running. The VC remains open or proceeds to other
  candidates; no proof status is accepted.
- `Cancelled`: scheduler, portfolio, or user cancellation stopped the run. No
  proof status is accepted.
- `Error`: spawn failure, missing executable, permission failure, unsupported
  required resource limit, crash, non-UTF/parse failure when parsing is
  required, capture-limit corruption, temporary-file failure, or malformed
  backend output.
- `Unknown`: backend completed but reported unknown/unsupported status or
  produced insufficient evidence without a hard process error.
- `Counterexample`: diagnostic-only model/counterexample data mapped through
  provenance. It is not proof acceptance.

All statuses are attached to the originating `VcId` / problem identity and
produce deterministic diagnostics. Backend failures must not crash unrelated
VCs or mutate existing proof status.

## Determinism

Equivalent backend run inputs and equivalent mock process behavior must produce
byte-identical deterministic run metadata after non-semantic timings are
normalized or separately marked non-semantic. Determinism covers:

- command fingerprint and profile id;
- input hash and concrete format;
- resource-limit records;
- stdout/stderr hash and truncation flags;
- exit status and termination class;
- result classification;
- candidate evidence metadata ordering;
- diagnostic keys and ordering.

Raw completion order, process ids, temporary paths, wall-clock timestamps,
backend scheduling races, and host-specific absolute paths do not decide
canonical candidate ordering or proof status.

## Gap Classification

- resolved `deferred` spec gap: task 13 defines the backend runner and result
  classification contract before source exists.
- `deferred`: task 14 generic process runner source, mock backend fixtures,
  platform-specific resource enforcement, and no-zombie tests.
- `deferred`: task 15 first concrete backend adapter and formula/substitution
  candidate extraction.
- `deferred`: task 16 full real-output result classification and polarity
  fixtures.
- `external_dependency_gap`: proof policy, winner selection, proof witness
  publication, cache promotion, artifact projection, and backend availability
  are outside task 13.

## Task-14 Test Expectations

Task 14 must add focused Rust coverage for:

- mock backend process invocation through stdin and private problem-file modes;
- byte-exact delivery of `encoded_problem.input_text` in both modes, including
  input bytes that look like shell metacharacters, proof commands,
  unsat-core requests, or backend directives; those bytes remain inert problem
  data and must not be rewritten, normalized, appended to, or interpreted by
  the runner;
- direct executable/argument spawning without shell interpretation;
- deterministic command fingerprints excluding pids, temp paths, timestamps,
  raw completion order, and machine-local absolute executable/working-directory
  paths; shuffled environment-policy input must be recorded through the sorted
  allowlist and produce the same semantic fingerprint;
- version probe success/failure recording with stdout/stderr hashes;
- timeout, cancellation, kill-grace, crash, non-zero exit, missing executable,
  and spawn-permission fixtures;
- stdout/stderr capture hashing, truncation flags, and diagnostics when limits
  are exceeded;
- private temporary directory cleanup and no child process left running after
  timeout/cancellation/crash;
- resource-limit metadata recording, including unsupported-limit diagnostics;
- `Proved` constructor/classification rejection when expected-result polarity
  mismatches, candidate formula/substitution evidence is absent, candidate
  target binding, input hash, labels, symbols, or provenance mismatch, or an
  otherwise matching candidate arrives after timeout, cancellation, crash, or
  capture-limit corruption that invalidates parsing;
- mock `Proved` classification only when observed result matches
  `ExpectedBackendResult::Unsat` and candidate formula/substitution evidence
  metadata matches target binding, input hash, labels, symbols, and provenance;
- counterexample, unknown, timeout, cancelled, and error statuses never produce
  accepted proof status;
- absence of kernel/SAT checking, proof policy, witness/cache publication,
  backend proof method trust, resolution-trace trust, unsat-core trust, SMT
  proof-object trust, and trusted backend `used_axioms`.
