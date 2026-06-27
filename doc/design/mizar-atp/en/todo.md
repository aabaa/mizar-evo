# mizar-atp TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs are added by their own spec tasks (English and Japanese in the
same change) before the implementation tasks that cite them. Completed specs
and source-deferred modules are marked in the table. The crate refines
architecture 09, 10, 15, and 19 and internal 04.

| Module | Spec | Source | Status |
|---|---|---|---|
| problem | `problem.md` (task 2) | `src/problem.rs` | [x] |
| translator | `translator.md` (task 4) | `src/translator.rs` | [x] declaration, symbol-map, axiom, and conjecture translation source complete |
| property_encoding | `property_encoding.md` (task 7) | `src/property_encoding.rs` | [x] axiom-form property source complete; native declarations deferred |
| tptp_encoder | `tptp_encoder.md` (task 9) | `src/tptp_encoder.rs` | [x] deterministic FOF source complete; typed/native/backend routes deferred |
| smtlib_encoder | `smtlib_encoder.md` (task 11) | `src/smtlib_encoder.rs` | [x] deterministic uninterpreted SMT-LIB source complete; theory/sorted/native/backend routes deferred |
| backend | `backend.md` (task 13) | `src/backend.rs` | [x] generic runner and mock classification complete; real adapters/extraction deferred |
| portfolio | `portfolio.md` (task 17) | `src/portfolio.rs` | [x] task-18 no-early-stop source complete; proof policy, real extraction, kernel checks, witness/cache/artifact handoff deferred |

`mizar-atp` implements pipeline phase 13: ATP-eligible `VcStatus::NeedsAtp`
`VcIr` obligations in, backend-neutral `AtpProblem`s, concrete prover
protocol emissions, external backend execution, and formula/substitution
evidence candidates out. Everything this crate produces is untrusted evidence:
`Proved` claims become trusted only after `mizar-kernel` checks the
formula/substitution evidence, and winner/policy selection belongs to
`mizar-proof`. Determinism rules apply to everything Mizar-side (premise order,
encoding, problem ids); backend nondeterminism is recorded as metadata, never
absorbed silently.

Dependency order: `problem` data → `translator` / `property_encoding` →
protocol encoders → `backend` runner → `portfolio`.

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The crate depends on `mizar-session`, `mizar-core` (core formulas),
`mizar-vc` (`VcIr` with `NeedsAtp` status), and `mizar-kernel`
(formula/substitution evidence schema types after the kernel post-closeout
correction). Backend binaries are external processes configured via `PATH` or
explicit configuration; crate tests use mock backends. Architecture:
[09.atp_interface_protocol.md](../../architecture/en/09.atp_interface_protocol.md),
[10.atp_backend_integration.md](../../architecture/en/10.atp_backend_integration.md);
integration: [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md).

## Postponement Gate

Autonomous development of `mizar-atp` is deferred until `mizar-kernel` records
and implements the formula/substitution evidence schema and `mizar-vc` records
the corresponding handoff contract. New ATP work must not target
MiniSAT-compatible resolution traces as the trusted output. ATP backends may
use any proof-search method internally, but this crate's trusted handoff is a
candidate evidence package containing formulas, substitutions, provenance,
and target binding for kernel SAT-backed checking. Instantiated formulas and
SAT problems are derived by `mizar-kernel`, not produced as trusted ATP
payload.

Current gate status: satisfied for the generic runner path by `mizar-kernel`
tasks 23-28 and `mizar-vc` tasks 24-25. Tasks 1-14 may build only the
spec-backed problem, translation, encoding, and generic backend-runner slices.
Task 15 records the first real backend adapter and evidence extractor as
`external_dependency_gap` / `deferred` until a paired extraction spec and a
guarded supported backend route exist. Proof policy, witness publication, and
cache promotion remain deferred to their own crates/tasks. `mizar-proof` is
not a workspace crate, so policy and witness-publication integration is an
`external_dependency_gap`, not a reason to add placeholders here.

## Resolved And Open Decisions

- **First backend and evidence route: still deferred after the task-15 gate.**
  The kernel formula/substitution evidence schema and VC handoff are available,
  but this crate still lacks a paired `evidence.md` extractor spec and source
  module defining how a concrete backend output becomes kernel-parseable
  formula/substitution candidate bytes or refs. The architecture-10 supported
  backend binaries were not available in the task-15 verification environment.
  Do not add a real adapter, backend-output parser, or placeholder candidate
  schema until that extraction route is specified.
- **Evidence schema ownership: follows `mizar-kernel` tasks 23-25.** This
  crate constructs candidate evidence against kernel-owned schema types; the
  kernel never depends on this crate.
- **Externally attested evidence: out of scope here.** Labeling is produced
  by this crate, but the acceptance policy is owned by `mizar-proof`
  (architecture 10 constraints; its task 4).

## Ordered Task List

Keep `cargo test -p mizar-atp` green after each task (see
[Recommended Verification](#recommended-verification)).

### Problem layer

1. **Crate scaffold and lint-policy guard.** [x]
   - Add the `mizar-atp` workspace member depending on `mizar-session`,
     `mizar-core`, `mizar-vc`, and `mizar-kernel`; add
     `tests/lint_policy.rs` mirroring the `mizar-frontend` guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: `mizar-vc` task 24, `mizar-kernel` tasks 23-25. Spec:
     architecture 09 and the post-closeout evidence correction.
   - Status: complete as a scaffold-only task. The crate plan classifies the
     pre-existing missing crate as `source_drift`, keeps all semantic module
     implementation deferred until paired specs exist, and records the absent
     `mizar-proof` integration and first-backend route as
     `external_dependency_gap`/`deferred`.

2. **Spec: `problem.md`.** [x]
   - Write the `AtpProblem` data-shape spec (English and Japanese, no code):
     logic profiles, declarations, axioms, conjecture, type context, encoded
     properties, symbol map, `AtpProvenance`, and `expected_result`
     polarity.
   - Deps: 1. Spec: architecture 09 "Backend-Neutral Problem Layer",
     [01.ir_layers.md](../../architecture/en/01.ir_layers.md), architecture 15,
     architecture 19, and internal 04.
   - Status: complete as a docs-only task. `problem.md` defines the
     backend-neutral problem boundary, deterministic identity, provenance
     requirements, `Unsat` polarity contract, and prohibited trusted material.
     Rust data shapes were deferred at task-2 closeout and are implemented by
     task 3.

3. **Implement `problem` data shapes.** [x]
   - Implement `AtpProblem` and provenance tables per task 2, plus a
     deterministic debug rendering.
   - Tests: construction round-trips; every formula traceable through
     provenance; rendering stability.
   - Deps: 2. Spec: `problem.md`.
   - Status: complete. The module implements validated problem construction,
     deterministic identity/rendering, provenance and symbol-map checks,
     fail-closed missing required inputs, unsupported profile-feature
     classification, and the fixed `Unsat` expected-result contract. No
     backend runner, kernel checking, proof policy, witness, cache, or trusted
     backend proof material is introduced.

### Translation

4. **Spec: `translator.md`.** [x]
   - Write the `VcIr`→`AtpProblem` translation spec (English and Japanese,
     no code): premise materialization, deterministic premise ordering,
     soft-type fact preservation (sort encoding must not erase facts needed
     to justify the VC), and validity-checking polarity.
   - Deps: 2. Spec: architecture 09 "Encoding Strategy"/"Validity Checking
     Polarity".
   - Status: complete as a docs-only task. `translator.md` defines the
     deterministic `VcIr` / kernel-handoff to `AtpProblem` translation
     boundary, target-binding checks, premise materialization limits,
     structured projection inputs, duplicate-premise rejection, proof-hint
     non-pruning, soft-type preservation, declaration/symbol-map
     responsibilities, `Unsat` polarity, and prohibited trusted/backend
     material. Declaration/symbol-map translator source is implemented by
     task 5; axiom/conjecture problem construction is implemented by task 6.

5. **Declaration and symbol-map translation.** [x]
   - Translate structured declaration and soft-type projections derived from
     `VcIr` / handoff inputs into `AtpDeclaration`s with a reversible-enough
     symbol map for diagnostics.
   - Tests: reject non-`NeedsAtp` VCs and stale handoffs; fail closed on
     missing/malformed structured declaration and soft-type projections;
     produce deterministic declarations/symbol maps under shuffled equivalent
     inputs; fail closed on duplicate/missing/kind/arity-mismatched
     declarations; preserve explicit profile choice without silent profile
     switching; keep prohibited backend/kernel/SAT/proof-acceptance material
     out of the translator API/debug rendering.
   - Deps: 3, 4. Spec: `translator.md`.
   - Status: complete. `src/translator.rs` exposes a task-5 partial
     translation that checks `NeedsAtp` status and target handoff, consumes
     structured declaration / soft-type projections, derives deterministic
     declarations, symbol-map rows, type guards, provenance, diagnostics, and
     target binding, and validates type-guard signatures without constructing
     a final `AtpProblem`.

6. **Axiom and conjecture translation.** [x]
   - Materialize cited premises into axioms in deterministic order, encode
     the goal as the conjecture, and attach provenance and
     `expected_result`.
   - Tests: reject non-`NeedsAtp` VCs and mismatched target handoffs; fail
     closed on missing/malformed structured formula projections; report
     unsupported/open outcomes for unsupported formula/profile features or
     alpha-repair/substitution-invention requirements; reject duplicate
     premise refs/source identities; prove proof hints and `Only`/`Exclude`
     restrictions do not add/drop/prune premises; fail closed on imported
     facts missing required proof status, statement fingerprint, or formula
     context; check premise-order determinism, provenance completeness,
     soft-type preservation, fixed `ExpectedBackendResult::Unsat` polarity,
     and absence of prohibited backend/kernel/SAT/proof-acceptance material.
   - Deps: 5. Spec: `translator.md`.
   - Status: complete. `src/translator.rs` now exposes task-6
     `AtpTranslationInput`, structured `AtpFormulaProjection` targets for
     VC formula refs and imported facts, and `translate_problem`. The
     translator materializes the sorted immutable `vc.premises` list into
     axioms, materializes the VC goal as the conjecture, records
     `ExpectedBackendResult::Unsat`, checks final-goal handoff polarity for
     `AssertFalseForRefutation`, requires final-goal projection binding
     `goal:1`, rejects duplicate premise refs, duplicate resolved
     formula/source identities, and repeated imported source tuples, and checks
     projection fingerprints/provenance payloads against the matching VC kernel
     handoff without parsing handoff formula bytes. Local-context, cited,
     generated, and imported fact materialization is covered. Checker-owned and
     type-predicate premise materialization remains fail-closed unless the VC
     handoff exposes a matching explicit source class/projection; no placeholder
     source class is invented in `mizar-atp`.

7. **Spec: `property_encoding.md`.** [x]
   - Write the property-encoding spec (English and Japanese, no code): how
     definitional properties (commutativity, …) are encoded as axioms or
     native backend properties, and when each strategy applies.
   - Deps: 4. Spec: architecture 09 "Property Encoding".
   - Status: complete. `property_encoding.md` now specifies supported property
     families, axiom-form encoding, generated-binder declarations, native
     declaration gates, deterministic identity, provenance requirements,
     connectedness disjunction handling, fail-closed/deferred classes, and
     task-8 test expectations. No Rust source is added by this spec-only task.

8. **Property encoding.** [x]
   - Implement the property-encoding rules with recorded encoding decisions
     in `EncodedProperty`. Task 8 emits only axiom-form properties; native
     declarations remain deferred until concrete encoder specs define exact
     semantics.
   - Tests: per-property fixtures, generated-binder declaration/provenance
     coverage, connectedness disjunction coverage, deterministic ordering, and
     native-declaration deferred/fail-closed coverage.
   - Deps: 6, 7. Spec: `property_encoding.md`.
   - Status: complete. `src/property_encoding.rs` accepts structured explicit
     property projections, validates target declarations/symbol-map rows and
     profile capabilities, generates deterministic binder declarations,
     symbol-map rows, and provenance, emits only `EncodedProperty::axiom`
     rows, rejects duplicates and unsupported/deferred families fail-closed,
     and keeps native declarations deferred.

### Protocol encoders

9. **Spec: `tptp_encoder.md`.** [x]
   - Write the TPTP emission spec (English and Japanese, no code): dialect
     coverage, name mangling, and deterministic output rules.
   - Completed by paired `tptp_encoder.md` docs. Task-10 source is limited to
     deterministic FOF emission; TFF-like typed output, CNF, include files,
     native property declarations, backend pragmas, backend execution, and
     evidence extraction remain deferred.
   - Deps: 2. Spec: architecture 09 "Supported Formats".

10. **TPTP encoder.** [x]
    - Emit TPTP text from `AtpProblem` deterministically.
    - Tests: golden-file fixtures; byte-identical output across runs; exact
      separators, parenthesization, labels, and final newline; profile gates;
      native-property rejection; free-variable, duplicate-binder, and
      shadowing rejection; raw-name injection and mangling-collision
      rejection; provenance side metadata; lint/API boundary guards.
    - Status: complete. `src/tptp_encoder.rs` emits only deterministic FOF
      text from validated `AtpProblem` values, records symbol and formula-label
      side metadata, rejects unsupported profiles, sorted binders, native
      declarations, scope failures, malformed private formula cases, raw-name
      injection, and name/label collisions, and keeps diagnostics out of
      semantic text. No backend runner, kernel/SAT checking, proof acceptance,
      witness/cache integration, TFF/native shortcut, legacy certificate, or
      resolution-trace acceptance is added.
    - Deps: 6, 9. Spec: `tptp_encoder.md`.

11. **Spec: `smtlib_encoder.md`.** [x]
    - Write the SMT-LIB emission spec (English and Japanese, no code): sort
      encoding, logic selection, and deterministic output rules.
    - Completed by paired `smtlib_encoder.md` docs. Task-12 source is limited
      to deterministic uninterpreted SMT-LIB emission using one fixed
      `mizar_universe` sort plus explicit guard predicates/type-guard
      assertions. Arithmetic theories, arrays, datatypes, bit-vectors,
      sorted function/predicate signatures, `BackendSorts`, `SortsAndGuards`,
      native property declarations, solver options, proof/unsat-core commands,
      backend execution, and evidence extraction remain deferred.
    - Deps: 2. Spec: architecture 09 "Supported Formats".

12. **SMT-LIB encoder.** [x]
    - Emit SMT-LIB text from `AtpProblem` deterministically.
    - Tests: golden-file fixtures; `QF_UF` / `UF` logic selection; exact
      formula rendering; premises plus negated conjecture polarity; fixed
      `mizar_universe` sort plus explicit guard/type-guard preservation;
      profile gates for unsupported sort strategies, sorted binders, equality,
      quantifiers, and sort-dependent uses; unused sort declarations ignored
      and absent from output; native-property rejection; scope/arity/source failures;
      raw-name injection and SMT-LIB symbol collision rejection; provenance
      side metadata; no proof/unsat-core/backend-material trust.
    - Status: complete. `src/smtlib_encoder.rs` emits only deterministic
      uninterpreted SMT-LIB text from validated `AtpProblem` values, records
      symbol and assertion-label side metadata, emits premises/type
      guards/properties plus a negated conjecture under the `Unsat` contract,
      rejects unsupported profiles, sorted binders, native declarations, scope
      failures, malformed private formula cases, raw-name injection, and
      name/label collisions, and keeps diagnostics and proof/unsat-core
      material out of semantic text. No backend runner, kernel/SAT checking,
      proof acceptance, witness/cache integration, theory/sorted/native
      shortcut, legacy certificate, or resolution-trace acceptance is added.
    - Deps: 6, 11. Spec: `smtlib_encoder.md`.

### Backend execution

13. **Spec: `backend.md`.** [x]
    - Write the backend spec (English and Japanese, no code): backend trait,
      process model (spawn, resource limits, termination), configuration
      and version recording, crash handling, and result classification
      including the rule that `Proved` requires matching `expected_result`
      plus evidence.
    - Status: complete by paired `backend.md` docs. Task-14 source is limited
      to the generic child-process runner, mock backend fixtures,
      deterministic run metadata, resource/timeout/cancellation/crash
      handling, and invariant-preserving mock classification. Real backend
      adapters, backend-specific output parsing, candidate evidence
      extraction, portfolio execution, proof policy, witness/cache
      publication, and kernel checking remain deferred.
    - Deps: 2. Spec: architecture 10 "Process Model"/"Result
      Classification", [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
      "Backend Runner".

14. **Backend runner.** [x]
    - Implement process execution with resource limits, timeouts,
      cancellation, and graceful crash handling; mock backend for tests.
    - Tests: stdin and private problem-file modes; direct executable/argument
      spawning without shell interpretation; deterministic command
      fingerprints; version-probe success/failure metadata; timeout,
      cancellation, kill-grace, crash, non-zero exit, missing executable, and
      spawn-permission fixtures; byte-exact input delivery in both stdin and
      private-file modes without rewriting, normalization, appended proof
      commands, unsat-core requests, shell interpretation, or inferred polarity
      changes; stdin delivery through a private spool connected to fd 0 without
      backend path exposure or verifier-side writer-thread deadlock;
      deterministic fingerprints that exclude pids, temp paths, timestamps,
      raw completion order, and machine-local absolute executable/working-directory
      paths while recording sorted allowlisted environment variables;
      stdout/stderr hashes and truncation
      diagnostics; drain-after-retained-limit behavior so stream hashes cover
      complete observed streams; private temp creation with exclusive/private-path
      semantics and cleanup; no child process left after
      timeout/cancellation/crash; resource-limit records and unsupported-limit
      diagnostics, including unsupported required limits becoming `Error`;
      `Proved` rejection for polarity mismatch, missing formula/substitution
      evidence payload/ref, candidate metadata mismatches, unsupported required
      limits, incomplete streams, or otherwise matching evidence after
      timeout/cancellation/crash/parsing corruption; mock `Proved` only with
      matching `ExpectedBackendResult::Unsat`, supported payload/ref, and
      candidate metadata; no kernel/SAT checking, proof policy,
      witness/cache publication, backend proof-method trust, resolution-trace
      trust, unsat-core trust, SMT proof-object trust, or trusted backend
      `used_axioms`.
    - Status: complete. `src/backend.rs` implements the generic direct-spawn
      child-process runner, deterministic input/command/stream metadata
      hashing, stdin and private problem-file modes, version probes,
      timeout/cancellation/crash/missing-executable handling, drain-safe
      stdout/stderr capture, unsupported required-limit fail-closed behavior,
      private temp cleanup, and mock observation classification. `Proved`
      remains candidate-evidence-only and requires matching `Unsat`, supported
      formula/substitution payload/ref, and matching target/input/label/symbol/
      provenance metadata. Real backend adapters, backend-specific parsers,
      formula/substitution candidate extraction from real output, portfolio,
      proof policy, witness/cache publication, and kernel checking remain
      deferred.
    - Deps: 13. Spec: `backend.md`.

15. **First concrete backend integration.** [x] deferred / external_dependency_gap
    - Resolve the first-backend decision; integrate one real backend
      end-to-end: emit problem, run, and extract formula/substitution
      evidence candidates against the kernel schema.
    - Tests: integration tests behind a backend-available guard; candidate
      evidence parses under `mizar-kernel`'s structural validation and
      remains untrusted until kernel checking.
    - Deps: 10 or 12 (per chosen backend), 14, `mizar-kernel` tasks 25-28.
      Spec: `backend.md`.
    - Gate result: deferred. `mizar-kernel` tasks 25-28 and the `mizar-vc`
      handoff are present, but `mizar-atp` has no paired evidence-extraction
      spec/source module for translating real backend output into
      kernel-owned formula/substitution candidate payloads. The supported
      architecture-10 backend executables (`vampire`, `eprover`, `cvc5`, `z3`)
      were not found in the task-15 environment, so a backend-available
      integration fixture would be skipped and could not validate an adapter.
      No real adapter, backend-specific parser, fake candidate schema, kernel
      call, proof-policy hook, witness/cache output, or trusted backend proof
      material is added. Reopen this task only after an English/Japanese
      `evidence.md` spec (or equivalent backend-specific extraction spec)
      defines the candidate payload/ref contract and a supported backend route
      is available for guarded integration tests.

16. **Result classification and polarity validation.** [x] deferred / external_dependency_gap
    - Classify backend outcomes (proved, counterexample, timeout, unknown,
      error); emit `Proved` only when the observed result matches
      `expected_result` and candidate formula/substitution evidence is
      present; counterexamples feed diagnostics only.
    - Tests: classification fixtures per outcome; polarity-mismatch cases
      never classify as proved.
    - Deps: 15 plus the task-15 reopen condition: a paired
      evidence-extraction spec and a guarded supported backend route must exist
      before real-output classification is implemented. Spec: `backend.md`
      (classification section).
    - Gate result: deferred. Task 14 already covers process-status and mock
      candidate classification invariants. Real-output parsing and polarity
      fixtures require the task-15 extraction route, which remains blocked by
      the missing paired evidence-extraction spec/source module and missing
      guarded supported backend. No backend-specific parser, fake observed
      output schema, adapter-specific classification table, kernel call, or
      trusted backend proof material is added. Reopen with task 15 after the
      extraction route and supported backend fixture exist.

### Portfolio

17. **Spec: `portfolio.md`.** [x]
    - Write the portfolio spec (English and Japanese, no code): per-VC
      portfolio tasks, candidate evidence collection, early stop, resource
      budgets, and the boundary that winner selection is `mizar-proof`
      policy — completion order never decides results. Early stop is allowed
      only after the proof policy reports that no pending candidate can
      displace the selected class.
    - Deps: 13. Spec: architecture 10 "Portfolio Execution",
      [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
      "ATP Portfolio Service".
    - Status: complete as a docs-only task. `portfolio.md` defines
      policy-neutral portfolio planning, candidate collection, deterministic
      candidate ordering, resource budgets, cancellation and early-stop
      constraints, and kernel/proof-policy handoff boundaries. No Rust source,
      proof policy evaluator, kernel call, witness/cache publication, real
      backend evidence extractor, fake real-output schema, or trusted backend
      proof material is added.

18. **Portfolio execution.** [x]
    - Implement portfolio construction and candidate collection with
      deterministic candidate ordering and cooperative cancellation.
    - Tests: shuffled completion produces identical candidate sets and
      orders; cancellation leaves no partial candidates, and no early-stop
      oracle is fabricated.
    - Deps: 14, 16, 17. Spec: `portfolio.md`.
    - Status: complete within the task-17 no-early-stop boundary.
      `src/portfolio.rs` builds deterministic plans from prebuilt
      `BackendRunInput` values, validates same-problem membership, collects
      terminal `BackendRunResult` values into a policy-neutral
      `PortfolioEvidenceSet`, orders formula/substitution candidates
      independently of completion order, and cancels without emitting partial
      candidates. It does not call the kernel, evaluate proof policy, create
      witness/cache/artifact state, implement early-stop policy finality, add a
      real backend extractor, or trust backend proof material.

19. **ATP run metadata recording.** [x]
    - Record seeds, timeout settings, backend identities/versions, and
      resource usage for artifacts and reproducibility notes as a read-only
      backend run-metadata projection.
    - Include stream/resource usage and diagnostics, but keep runtime
      observations outside trusted acceptance material and downstream candidate
      hashes.
    - Tests: metadata completeness fixtures; metadata excluded from
      semantic hashes.
    - Deps: 16. Spec: architecture 00 "Incrementality and Reproducibility";
      `backend.md`.
    - Boundary: no artifact writing, proof policy, kernel checking,
      witness/cache publication, real backend extraction, or trusted backend
      proof material.
    - Status: complete within the backend-runner metadata boundary.
      `BackendRunMetadata` projects seeds, timeout settings, backend
      identity/version records, command fingerprints, stream/resource usage,
      elapsed time, and diagnostics from `BackendRunResult` without changing
      command identity, candidate evidence, kernel checks, proof policy, or
      artifact/cache/witness publication.

### Hardening and cross-cutting follow-ups

20. **Corpus and mock-backend integration suite.** [x]
    - Add `advanced_semantics`-stage corpus cases driven through mock
      backends, plus `spec_trace.toml` entries.
    - Use metadata-only `tests/property` fixtures because `mizar-test` has no
      active `advanced_semantics` runner/tag gate yet. Crate-local integration
      tests may read those fixtures and drive the existing mock backend runner,
      mock observed-result classification, and portfolio collection APIs.
    - Cover formula/substitution candidate handoff, counterexample recording, and
      unknown/open results without kernel checking, proof policy, real-output
      extraction, witness/cache/artifact publication, or a placeholder evidence
      schema.
    - Deps: 18. Spec: [staged_model.md](../../mizar-test/en/staged_model.md);
      `portfolio.md`.
    - Status: complete within the metadata-only corpus and crate-local mock
      backend boundary. `tests/property/atp_mock_backend_integration_001.*`
      records the inert `advanced_semantics` corpus anchor, and
      `tests/mock_backend_corpus.rs` drives formula/substitution,
      counterexample, and unknown/open cases through existing mock backend
      classification plus portfolio collection. Active `.miz`
      advanced-semantics execution, real-output extraction, kernel checking,
      proof policy, witness/cache/artifact publication, and placeholder evidence
      schemas remain deferred/external.

21. **Determinism suite.** [ ]
    - Property coverage that identical `VcIr` inputs produce identical
      problems, encodings, and candidate orderings with mock backends.
    - Deps: 18. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).

22. **Public-enum forward-compatibility policy.** [ ]
    - Apply the `mizar-frontend` task-25 procedure to each public enum;
      record decisions in the owning module specs.
    - Deps: 18. Spec: all module specs.

23. **Source/spec correspondence audit.** [ ]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 22. Spec: all module specs and this TODO.

24. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-atp/en/` with its Japanese companion and
      synchronize content.
    - Deps: 23. Spec: repository documentation policy.

25. **Portfolio completion-order independence gate.** [ ]
    - Add a portfolio-specific regression gate that runs mock backends with
      adversarial completion orders. Candidate collection may finish early only
      when `mizar-proof` policy reports that no pending candidate can displace
      the selected class; raw completion time must never become proof identity.
    - Tests: a later kernel-verifiable candidate beats an earlier externally
      attested result under release policy; ties use deterministic backend
      priority/evidence-strength/problem-hash keys; cancelled or killed
      losing backends leave no partial accepted state.
    - Deps: 18, 21, `mizar-proof` tasks 7, 9, 12, and 13. Spec:
      [10.atp_backend_integration.md](../../architecture/en/10.atp_backend_integration.md),
      [14.parallel_verification_and_scheduling.md](../../architecture/en/14.parallel_verification_and_scheduling.md),
      [22.incremental_verification_contract.md](../../architecture/en/22.incremental_verification_contract.md).

26. **Architecture-22 follow-up audit.** [ ]
    - Re-run the source/spec correspondence and bilingual documentation sync
      audits for the task-25 portfolio ordering and early-stop contract;
      record any remaining policy-boundary or completion-order gaps as
      follow-up tasks.
    - Deps: 25. Spec: all module specs, this TODO, and repository
      documentation policy.

27. **Module-boundary refactor gate.** [ ]
    - Before treating the crate as ready for downstream consumers, audit the
      source layout for oversized files, mixed responsibilities, and private
      helpers that should be split along the module table and spec boundaries.
      Split any review-bottleneck implementation files into private modules
      without changing public APIs, diagnostics, deterministic renderings,
      artifact-facing schemas, or consumer-visible behavior.
    - After any split, update this module table/source paths as needed and
      re-run the source/spec and bilingual documentation audit scopes for the
      moved APIs. Do not mix behavior cleanup or API exposure into the move;
      those require their own spec tasks.
    - Deps: 26. Spec: this TODO,
      [internal 07](../../internal/en/07.crate_module_layout.md), all module
      specs.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-atp
cargo clippy -p mizar-atp --all-targets -- -D warnings
```

For tasks that touch the VC or kernel boundary, also run:

```text
cargo test -p mizar-vc
cargo test -p mizar-kernel
```

For portfolio ordering and early-stop tasks, also run:

```text
cargo test -p mizar-proof
```

Check the task off here once tests pass.

## Notes

- Everything produced here is untrusted evidence; trusted status exists only
  after kernel evidence checking, and acceptance policy lives in
  `mizar-proof`.
- Encoding need not be reversible, but every backend-visible formula must be
  traceable through `AtpProvenance`. Backend-reported used axioms are not
  trusted `used_axioms`; only kernel-checked formula/provenance evidence may
  feed downstream witness material.
- Backend nondeterminism is recorded (seeds, versions, timings), never
  silently absorbed; Mizar-side translation and encoding are bit-stable.
- ATP unavailability must not break earlier phases; this crate degrades to
  `open` VC statuses, not to errors elsewhere in the pipeline.
- Backend proof methods and logs are diagnostic or extraction input only. They
  are not `AtpProvenance`, kernel evidence, trusted handoff material, or a
  resolution trace certificate.
