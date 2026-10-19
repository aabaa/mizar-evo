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
proofs, invoke Kernel checking/acceptance APIs, publish witnesses, select
portfolio winners, update caches, or turn backend proof methods, logs, unsat
cores, SMT proof objects, TSTP traces, or resolution traces into trusted
acceptance material. The generic process runner does not run SAT; the separate bounded source query below invokes existing SAT services only to screen untrusted candidates.

## Scope

Task 13 is specification-only. It authorizes future source modules to expose a
backend runner API, a mock-backend test harness, deterministic run metadata,
and fail-closed result classification. It does not add Rust source, spawn
processes, integrate a real backend, parse backend proof languages, extract
formula/substitution candidates from a real backend, call the kernel, publish
artifact witnesses, run a portfolio, or implement proof policy.

Task-14 source implements the generic process runner and mock backend fixtures
described here. Task 15 re-evaluated the first real-backend route and recorded
it as `external_dependency_gap` / `deferred`: `mizar-atp` still needs a paired
evidence-extraction spec before it can parse real backend output into
kernel-owned formula/substitution candidate payloads. Broad classification
fixtures and polarity validation remain task-16 work, although task 14 may
expose constructors and invariants that make invalid `Proved` results
unrepresentable.

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

## Genuine encoded input production

`EncodedBackendProblem::encode(&AtpProblem, ConcreteFormat) -> Result<Self, String>` invokes the existing format's actual encoder (TPTP FOF or SMT-LIB Uninterpreted), propagating unsupported-profile errors without reprofiling or text repair. Copy exact returned UTF-8 bytes and raw label tokens; retain the actual problem ID, target, expected Unsat polarity and profile name, with explicit `fof` / `tff-like` / `smtlib-uninterpreted` fragment tags. The existing constructor and hash domains stay unchanged.
Normalize each actual encoder symbol row to `(ATP name, concrete name, typed source)`, then store a canonical metadata string: `a=<byte-length>:<lowercase-hex>;b=<byte-length>:<lowercase-hex>;s=<source>`. Source is `mizar=<byte-length>:<lowercase-hex>`, `binder=<byte-length>:<lowercase-hex>` or `guard=<decimal-id>` for the existing MizarSymbol, GeneratedBinder or TypeGuard variant. Lengths count UTF-8 bytes. This preserves the complete actual mapping without Debug rendering, delimiter ambiguity or a parallel symbol model.
Derive provenance_hash with the existing private StableHasher, domain `mizar-atp/backend-source-provenance/v1`, one tagged byte field `source-problem-id` containing the actual problem_id().hash() bytes. This deliberately conservative binding covers the canonical problem's complete typed provenance, formulas, declarations, target, symbol maps and profile; unrelated semantic changes may invalidate it too. Diagnostics remain excluded by the existing neutral identity. It is neither a provenance-only reuse key nor proof evidence; every input/metadata hash is computed from actual content, never a fixture constant or supplied receipt.
Factory inputs contain no separate encoded output, labels or hashes to mismatch. Concrete encoders own label selection and mangling; backend owns immutable input binding only. Test all typed source classes, fixed canonical mapping bytes, coherent metadata changes, diagnostic neutrality and real source pipelines for both formats. General source/backend integration, instance search/limits, advisory extraction, execution/classification and proof acceptance remain separate dependencies.

## Closed-choice reflexivity names

`encode_closed_choice_reflexivity_names(&ParsedKernelEvidence) -> Result<Vec<Vec<u8>>, String>` reads the actual Kernel-parsed Formula v1 data only. The caller authenticates its source/context; names are untrusted transport, not a proof or complete candidate. ATP owns this deterministic instance selection; Kernel alone decodes names and derives/checks formulas.
Require schema 1 and exactly one Equality manifest key. Every substitution must have payload kind 1, an empty rewrite path, exactly one replacement with role 1, and a zero-argument FunctorPredicate application as its actual term. Reject unsupported inputs without partial names. Deduplicate actual SymbolKeys in ascending SymbolKey order, then assign checked increasing nonzero u32 instance IDs. No substitutions yields no names; never derive numeric identities from source strings or hashes.
Each item preserves the existing Formula-v2 grammar: big-endian instance ID, reflexivity tag 1, actual equality symbol ID, term/application tags 2/2, actual choice symbol ID and zero argument count. The normal VC transport frames these items. No supplied instance formula, search, SAT check, backend run, candidate classification or verifier budget is involved.
[Proof's source facade](../../mizar-proof/en/status.md#source-existential-registration-proof) consumes this operation after its fresh v1 parse. Real Open/NeedsAtp source callers test exact bytes and unsupported parsed shapes; complete ATP extraction and policy-owned search limits remain deferred.

## Source registration candidate inputs

`build_source_registration_candidate_inputs(&CoreIr, &VcSet, VcId, LogicProfile, ConcreteFormat) -> Result<(EncodedBackendProblem, BackendCandidateEvidence), String>` creates bound untrusted transport only. Invoke the existing [source translator](translator.md#source-registration-projections) and genuine encoded factory, then independently rebuild the [same source handoff](../../mizar-vc/en/kernel_evidence_handoff.md#source-existential-registration-proof). Whole generator/default-discharge replay authenticates the selected NeedsAtp graph; original Open, selected Discharged and unsupported/coherently forged inputs yield an error without a returned pair. Keep the explicit profile/format; no pruning, fallback or reprofiling.
Parse the original handoff's local Formula v1 with its actual target and kernel profile/tautology policy for source formula/provenance data. Derive and write substitutions through [bounded structured extraction](#bounded-structured-source-substitutions), then parse that rebuilt handoff's v1 for the closed-choice name writer and frame that rebuilt handoff's v2. Parsing is structural validation, not acceptance. Preserve the original source profile's complete wire, canonical/context identities, formulas, provenance and refutation goal; original substitution records supply neither the new recipe nor final wire.
Construct the existing FormulaSubstitutionBytes candidate with the exact v2 bytes and the fresh encoded container's complete target binding, input hash, provenance hash, all labels and all symbol mappings. Inputs contain no separate candidate bytes, hashes, identities or encoded output. Unsupported source/encoding/framing/parsing/names abort without partial output.
Derive candidate_id using the existing private StableHasher domain `mizar-atp/source-registration-candidate/v1`, in order: byte fields `encoded-input-hash`, `encoded-metadata-hash`, u64 field `target-fingerprint-algorithm`, and byte field `formula-substitution-bytes`. Use actual container hashes, target algorithm ID and exact payload bytes, then existing lowercase hex with prefix `source-registration-v1:`. Metadata already binds the target digest/producer, labels, mappings and source provenance; explicitly include the algorithm without changing the old metadata hash. Input hash binds actual text/format/profile; payload binds the normal evidence fields. Never hash Debug, runtime data, reports or caller receipts.
The pair carries an unchecked attempt, including for a genuine false source that a fresh Kernel check rejects. It is no backend result or successful search. The caller owns current source/context authentication and ordinary independent [Kernel checking](../../mizar-kernel/en/checker.md), with acceptance owned by [Proof](../../mizar-proof/en/selection.md). Keep the existing original-Open built-in facade unchanged. Search limits, general finder, advisory extraction, backend execution, publication and reuse remain deferred.
Three distinct real-source regression callers cover direct concrete encoding/metadata, independent normal wire/fresh Kernel outcomes, and whole-source/context corruption. Exercise both formats and original/renamed positive/negative single/aggregate sources, deterministic replay/fixed identity, original Open/selected Discharged rejection and untrusted corrupted outputs. Preserve existing manual and producer oracles.

## Bounded structured source substitutions

The existing source candidate factory retains its public signature and uses the actual source translator's ATP trees, original parsed Kernel formulas and [complete ground correspondence](#source-ground-term-correspondence). Existing whole-source/default-discharge/profile/encoding gates run first with their errors unchanged. Core/VC inputs remain caller-owned; no new public operation, recipe type, field, dependency, lower API or freshness/acceptance token is introduced.
For every quantified axiom, resolve its actual GeneratedVcFact provenance marker to the selected actual VcFormulaRef. Require one original envelope entry with that producer ref and generated source class, and select the parsed formula by its original formula_id. Validate full source identity/class, fingerprint, provenance ID, target and payload agreement, including the ATP provenance anchor, as in the ground join. Independent row IDs, manifest order, spelling and hashes never supply numeric Kernel identities. Cover all and only actual quantified premises without missing, duplicate or extra recipes.
Require exactly one unsorted Forall binder. Align its body with the joined Kernel Atom/Not/And/Or tree: preserve Boolean constructors, ordered children and lengths; ATP Equality matches an arity-two Kernel Equality atom, builtin:set an arity-one BuiltinRelation, and source attribute atoms actual Predicate kind/arity. Numeric predicate identity stays with the joined original formula. Kernel has no Forall formula node. Reject nested quantifiers, True/False/Implies/Exists and unsupported term/formula shapes.
At aligned term paths, each occurrence of the ATP binder variable must correspond to an actual Kernel VariableId. Require at least one occurrence and one consistent formal mapping throughout the body and source profile; unrelated ATP variables or conflicting Kernel IDs refuse. A nonvariable ground occurrence must be in the complete existing correspondence map and equal its actual Kernel term. Only the observed nullary choice is supported; arbitrary functions, guessed symbols and manifest cardinality cannot establish correspondence.
Iterate the real VC premise order after the actual ground type fact and associate derived quantified records by actual source ref. Assign checked zero-based substitution IDs in that order, never from ATP row indices. Populate existing KernelSubstitutionPayload with that ref, the joined VariableId, observed nullary FunctorPredicate term and resolved original **formula** provenance payload. Old parsed/envelope substitutions are never a recipe oracle.
Use only the existing [binder/payload grammar](../../mizar-kernel/en/substitution_checker.md#binder-evidence-model): schema 1, no frames, one free variable containing that actual ID and no schematic variables; payload has its actual substitution ID, kind 1, empty root path, one formal-to-choice replacement with role 1 and empty side conditions. Inline the bounded nullary term wire (Application/FunctorPredicate tags, actual u32 symbol ID, zero arguments), not canonical term bytes; no general codec, new schema, hash domain or Kernel policy.
Rebuild existing KernelFormulaPayload/Projection values from original formula evidence getters and resolved provenance, including a separate final-goal projection. Call existing build_kernel_evidence_handoff with the same actual VcSet/VcId, profile, polarity and manifests, imported payloads empty, formula_context None and discharge_output None. Pass only the independently derived substitution vector; do not reconstruct a VC set or source context.
Require the rebuilt canonical/context identities to be consistent and match the original source profile; mismatch is a real error, with no old-candidate fallback or expected-hash/context rewrite. Parse the **returned new handoff's** v1, derive existing reflexivity names from its new parsed substitutions, and frame its v2. Preserve the old complete v1/v2 wire, candidate ID/hash tail, encoded metadata and source outcomes. Structural-only refusal is internal String `registration.atp.invalid_structured_substitution`; retain lower producer/parser/writer errors and public diagnostics.
A private primitive formula/term walker is permitted only for actual recursive Boolean/argument paths with at least three real call sites; an inline work stack is also valid. No one-use facade, alternate AST, test observer, injected writer or implementation-mirror lint is added. Extend only exact pure Kernel data-path allowances required by traversal, retaining the public inventory and every trusted/downstream denial.
Tests reuse genuine source helpers and source_candidate_check_bytes in mizar-test runner tests, preserving existing test bodies/oracles. Cover complete old-factory wire/container/canonical/context compatibility, both formats, renamed/single/aggregate/false sources, rebuilt writer plus fresh normal Kernel Accepted/Rejected outcomes, lower refusals and input purity. Preserve Proof's prebuilt None/zero behavior and SAT screening's None/zero/exact/MAX counts; do not duplicate adequate existing matrices. Private pure-walker relation tests may use existing untrusted ATP/Kernel types for consistent/conflicting occurrences, unsupported shapes and ground mappings, without fake Core, source capture or accepted results.
Exact legacy public outputs make extraction and validated old-copy behavior observationally equivalent. Runtime byte equality alone cannot prove internal origin: independent implementation/dataflow review must establish that the extracted vector feeds the existing builder, its returned handoff feeds both names and wire, and no original substitution read/copy or fallback supplies either. Tests establish observable compatibility and walker relations, without a test-only observation seam.
This is a bounded ATP30 prerequisite, including unchecked candidates for genuine false sources. Ordinary fresh Proof/Core/VC/context/policy/Kernel checks, metadata/selection/witness projections and source SAT screening remain unchanged. Full ATP30/31, arbitrary extraction, alternative enumeration/search/exhaustion, backend/advisory integration, finder policy enforcement, runtime/footprints, acceptance, witness/publication/reuse/cache and Steps 6–8 remain open.

## Source candidate SAT screening
`screen_source_registration_candidate(&CoreIr, &VcSet, VcId, LogicProfile, ConcreteFormat, Option<[u64; 4]>) -> Result<Option<(EncodedBackendProblem, BackendCandidateEvidence)>, String>` screens exactly the existing source-factory attempt; screening itself performs no backend execution, term/substitution generation or proof acceptance.
First call the existing source candidate factory, then source ground-term correspondence on the same immutable tuple; preserve all original whole-input/profile/encoding refusals before inspecting limits. The optional setting is supplied by the caller (for example the existing Proof policy getter), never obtained through a new dependency or default. None is unconfigured and returns None; literal zero/MAX retain their meanings. Existing Proof facades, policy fingerprints and prebuilt-candidate behavior stay unchanged.
Count the complete existing observed correspondence pool, actual parsed candidate substitution records and actual named equality records against the first three coordinates; the last coordinate must permit one SAT check. Use checked usize-to-u64 conversion, no truncation. These are bounded-attempt admission counts, not limits on prior source loading/building or general search enumeration. An exceeded count returns None before SAT encoding/solving, with no partial pair.
For configured limits, rebuild the original same-tuple handoff only to obtain its actual target/profile/tautology parser context; explicitly permit schema2 and parse the original candidate FormulaSubstitutionBytes. Keep exact wire, names, provenance, candidate ID, encoded bindings and all inputs. Parsing errors propagate as internal Strings. No private numeric name reconstruction, canonical-bytes-as-wire, alternative profile, candidate-derived source context or supplied SAT clauses.
Use only existing `mizar_kernel::sat_encoding::SatEncodingContext::v1`/`mizar_kernel::sat_encoding::encode_formula_evidence` and `mizar_kernel::sat_checker::SatCheckContext::v1`/`mizar_kernel::sat_checker::check_sat_problem`, with their unchanged internal limits and replay. Encoding errors propagate as internal Strings; only is_unsat returns the original pair unchanged. Sat or Rejected returns None. At most one query is made; a positive query is no KernelCheckResult, accepted status or substitute for an independent fresh normal Kernel check and Proof policy.
Implement one backend function insertion and only the necessary module-intro wording. Before Rust, extend only existing static public-function inventory and the four exact fully-qualified pure SAT service/context paths. Preserve every old prohibited token/assertion and other inventory bytes: generic runner/portfolio bodies remain query-free and Kernel acceptance/witness/cache/proof-policy calls remain denied. No new production type/helper/file/adapter/field/dependency/profile/hash/wire or lower-stage edit.
Append exactly three source_candidate_sat_screen_* families using existing genuine source helpers: independent old-factory pair/normal fresh Kernel oracle across positive/false/single/aggregate/both formats, replay and input preservation; actual policy-getter None/zero/exact/each-coordinate-underflow/MAX boundaries; original whole-input/profile refusals with configured and unconfigured limits. Reuse source_candidate_check_bytes with the original genuine handoff, original corruption matrices and immutable fixtures; no new context producer or duplicate one-use helper.
Caller-owned Core/VC replay and Unsat establish no original-source/current-runtime authentication, acceptance, witness/reuse/cache authority. This prerequisite does not enumerate alternatives, produce new instances, consume advisory reports, execute backends, enforce full finder search, integrate Proof scheduling or close ATP30/31/Steps6–8.

## Source ground term correspondence

`source_registration_ground_term_correspondence(&CoreIr, &VcSet, VcId, LogicProfile) -> Result<BTreeMap<AtpTerm, mizar_kernel::clause::Term>, String>` returns owned copies of existing terms only. Independently rebuild the existing source ATP problem and original VC handoff from the same immutable tuple; their existing whole-graph/default-discharge/profile gates apply first. Caller-owned Core/VC records and this query establish neither original-source freshness nor proof authority.
Parse the handoff's unchanged local Formula v1 with its actual target/profile/tautology policy, as in the existing source candidate factory. Preserve lower errors; no alternate profile, candidate construction, v2/equality-name writer, checker or search is invoked. The bounded source route must have exactly one observed ground term and one closed generated builtin:set axiom, plus its one duplicate type guard; reject other shapes without partial data.
Require the ATP axiom's sole argument to be that observed nullary Function. Join its resolved GeneratedVcFact provenance to a selected actual VcFormulaRef::Generated using the existing generated:N source marker; never parse or equate independent ATP/Kernel IDs. Require a unique original envelope entry with that producer_formula_ref and GeneratedVcFact source, then use its original formula_id to select the parsed entry. Compare source ID/class, full formula fingerprint and provenance ID; the resolved original/parsed provenance must agree in target, formula fingerprint and payload, and the ATP provenance anchor must equal the existing handoff payload encoding.
The joined Kernel formula must be an arity-one BuiltinRelation atom whose sole argument is an original nullary FunctorPredicate Application. Pair the existing ATP argument with that actual Kernel argument, cloning only for owned return. The type guard must equal the axiom and resolve to the same ATP provenance source/payload; all observed terms must be covered exactly and consistently. Do not infer Kernel numeric symbols from names/hashes, manifest order, declarations, cardinality or candidate substitutions.
Propagate existing producer/framing/parser errors first; use internal String `registration.atp.invalid_ground_term_correspondence` for correspondence-only refusal. No new public diagnostic, type, helper, alias, profile, wire/hash domain, dependency or lower-stage change. Insert one function in backend.rs and preserve every old byte outside it. Update only the existing lint_policy.rs inventory for this function and exact data-only Term/BuiltinRelation/Formula::Atom/FormulaSource::GeneratedVcFact paths; retain all trusted/downstream denials, including Kernel imports/checking/SAT/publication/reuse.
Append two genuine-source test families: exact complete pairs/deterministic replay/input preservation for original/renamed/single/all-aggregate/false definitions in both supported profiles, using actual original parsed goal/substitution terms as independent value oracles; and existing whole-input/profile/current-handle refusals with immutable actual Core/VC inputs. Reuse old source helpers and lower matrices, without synthetic Accepted/Complete/context/evidence or a duplicate parser/producer helper for one use.
This maps only the existing observed ground choices. Arbitrary function/variable/quantifier correspondence, Forall removal, term/substitution/equality generation, policy counting/enforcement/SAT/search, candidate acceptance, runtime, witness/publication/reuse and full ATP30/Steps6–8 remain later-owner work.

## Stable Hashes And Fingerprints

Task-14 source must implement a private deterministic hash helper for backend
metadata. It must not use `std::hash`, `DefaultHasher`, raw `Debug` rendering,
process ids, temporary paths, wall-clock timestamps, or completion order as
semantic hash input.

The helper uses length-prefixed canonical fields and explicit domain tags:

| Hash | Domain | Required fields |
|---|---|---|
| `input_hash` | `mizar-atp/backend-input/v1` | `concrete_format`, logic-profile name/fragment, exact `input_text` bytes |
| `command_fingerprint` | `mizar-atp/backend-command/v1` | backend kind, profile id, concrete format, semantic executable id, ordered args, sorted allowlisted environment, working-directory policy kind, input-delivery mode, random seed, resource-limit records |
| `stdout_hash` / `stderr_hash` | `mizar-atp/backend-stream/v1` | stream kind and the complete byte stream observed on that pipe |
| `metadata_hash` fields | `mizar-atp/backend-metadata/v1` | sorted labels, symbol bindings, provenance bytes, and target-binding fingerprint bytes as appropriate |

The semantic executable id may include a configured stable backend name or an
executable basename, but not a machine-local absolute path. Explicit working
directories are represented by policy kind in the command fingerprint; the
local directory path may appear in diagnostics but not in semantic hashes.

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
- for stdin mode, stage the byte-exact input in a private spool and connect a
  read handle as fd 0 without passing a path to the backend, so input delivery
  does not depend on a blocking writer thread;
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

Task-14 source must represent unsupported limits as either `best_effort` or
`required`. Unsupported best-effort limits are diagnostics. Unsupported
required limits classify the run as `Error` before any `Proved` result can be
constructed.

In private problem-file mode, the runner creates a per-run private directory
with race-resistant creation (`create_new`/exclusive creation semantics), writes
the problem file without reusing a preexisting path, applies best-effort
restrictive permissions on platforms that support them, and cleans the file and
directory after normal exit, timeout, cancellation, crash, or spawn failure.
If privacy or cleanup cannot be enforced, the run records diagnostics; privacy
failure is an `Error` for file-mode execution.

Stdout and stderr readers must avoid pipe backpressure deadlocks. After the
configured retained-byte limit is reached, readers keep draining the pipe until
EOF while retaining only the prefix and recording a truncation flag. Stream
hashes are computed over the complete observed stream, not merely the retained
prefix. If a stream cannot be completely drained because the process is killed
or the pipe fails, the result records an incomplete-stream diagnostic and must
not classify as `Proved`.

Stdin mode must not introduce a verifier-side writer thread that can be left
blocked by a backend descendant holding fd 0 open. If private stdin staging
cannot be created or opened, the run is `Error` before process spawn.

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

Task 19 adds a stable run-metadata projection for artifacts and reproducibility
notes. The projection records:

- run id, problem id, backend kind, profile id, concrete format, encoded input
  hash, encoded metadata hash, and command fingerprint;
- semantic executable id, normalized arguments, sorted allowlisted environment,
  working-directory policy kind, input-delivery mode, random seed, timeout
  settings, capture limits, and platform resource-limit records;
- version-probe success, parsed version, version stdout/stderr hashes, and
  version diagnostics when a probe was configured;
- terminal status, observed result, termination class, exit status,
  child-reaping flag, elapsed time, stdout/stderr hashes, total stream byte
  counts, truncation/incomplete flags, and backend diagnostics.

The projection is reproducibility metadata only. It is not candidate evidence,
kernel evidence, proof policy, artifact winner selection, witness publication,
or cache promotion. Runtime observations such as elapsed time, backend version
output, retained stdout/stderr bytes, and diagnostics are diagnostic-only and
must remain outside downstream candidate hashes, trusted-acceptance hashes, and
proof-acceptance material.

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
records the first real backend extractor as deferred until a paired extraction
spec and guarded backend route exist. Task 16 likewise remains deferred until
that route exists; it may then add full outcome and polarity classification
fixtures for real backend-style outputs.

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
kernel-owned schema, and task-14 mock classification must require explicit
payload bytes or an explicit payload reference. Label/symbol/provenance
metadata alone is not candidate evidence. Backend proof objects and logs may be
diagnostic inputs to a future extractor, but the candidate payload handed to
later kernel checking must not be a backend proof method, SMT proof object,
unsat core, TSTP trace, resolution trace, backend log, backend-reported
`used_axioms`, or legacy certificate. Kernel acceptance, trusted `used_axioms`,
proof witness drafts, artifact status, and cache promotion belong to
downstream tasks/crates.

Task 29 keeps the post-audit F1/F2/F6 kernel contract upstream of backend
classification. The backend classifier may compare candidate target bindings,
encoded-problem hashes, provenance hashes, formula labels, and symbol bindings,
but it must not invent or infer goal polarity, context-identity payloads, or
imported-statement projections from backend output. Candidates that lack the
translator/VC-handoff evidence required by the kernel contract remain
non-candidates until a real extractor can produce explicit formula/substitution
payloads.

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

## Public Enum Forward Compatibility

Task 22 applies the frontend task-25 policy to the `backend` module. Public
enums owned here are `#[non_exhaustive]` for downstream crates:
`BackendWorkingDirectoryPolicy`, `BackendIoMode`, `BackendLimitRequirement`,
`BackendRunStatus`, `BackendObservedResult`, `BackendTermination`,
`BackendCandidatePayload`, and `BackendConfigError`.

Public enum inventory: `BackendWorkingDirectoryPolicy`, `BackendIoMode`, `BackendLimitRequirement`, `BackendRunStatus`, `BackendObservedResult`, `BackendTermination`, `BackendCandidatePayload`, `BackendConfigError`.

Future runner policies, statuses, observed results, payload classes, or error
variants must be specified before source uses them. Inside `mizar-atp`, matches
that affect process execution, candidate evidence classification, result
metadata, or proof status must be explicit and fail closed unless a paired spec
documents an intentional fallback. New payload classes are diagnostic or
untrusted until a paired extraction spec maps them to formula/substitution
evidence.

## Gap Classification

- resolved `deferred` spec gap: task 13 defines the backend runner and result
  classification contract before source exists.
- resolved `source_drift`: task 14 implements the generic process runner, mock
  classification seam, deterministic run metadata, private input handling,
  drain-safe capture, and fail-closed process statuses.
- `external_dependency_gap` / `deferred`: task 15 cannot add the first concrete
  backend adapter yet because no paired `mizar-atp` evidence-extraction spec or
  source module defines how real backend output becomes kernel-parseable
  formula/substitution candidate bytes/refs, and the supported architecture-10
  backend executables were not available in the verification environment.
- `external_dependency_gap` / `deferred`: task 16 full real-output result
  classification and polarity fixtures depend on the task-15 extraction route;
  task 14's mock classification invariants remain the only implemented
  classifier surface until that route exists.
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
