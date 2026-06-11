# mizar-atp TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs do not exist yet; each is written by its own spec task (English
and Japanese in the same change) before the implementation tasks that cite it.
The crate refines architecture 09, 10, 15, and 19 and internal 04.

| Module | Spec | Source | Status |
|---|---|---|---|
| problem | `problem.md` (task 2) | `src/problem.rs` | [ ] |
| translator | `translator.md` (task 4) | `src/translator.rs` | [ ] |
| property_encoding | `property_encoding.md` (task 7) | `src/property_encoding.rs` | [ ] |
| tptp_encoder | `tptp_encoder.md` (task 9) | `src/tptp_encoder.rs` | [ ] |
| smtlib_encoder | `smtlib_encoder.md` (task 11) | `src/smtlib_encoder.rs` | [ ] |
| backend | `backend.md` (task 13) | `src/backend.rs` | [ ] |
| portfolio | `portfolio.md` (task 17) | `src/portfolio.rs` | [ ] |

`mizar-atp` implements pipeline phase 13: open `VcIr` obligations in,
backend-neutral `AtpProblem`s, concrete prover protocol emissions, external
backend execution, and certificate candidates out. Everything this crate
produces is untrusted evidence: `Proved` claims become trusted only after
`mizar-kernel` checks the certificate, and winner/policy selection belongs to
`mizar-proof`. Determinism rules apply to everything Mizar-side (premise
order, encoding, problem ids); backend nondeterminism is recorded as
metadata, never absorbed silently.

Dependency order: `problem` data → `translator` / `property_encoding` →
protocol encoders → `backend` runner → `portfolio`.

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The crate depends on `mizar-session`, `mizar-core` (core formulas),
`mizar-vc` (`VcIr` with `NeedsAtp` status), and `mizar-kernel` (certificate
schema types, per the kernel task-4 ownership decision). Backend binaries are
external processes configured via `PATH` or explicit configuration; crate
tests use mock backends. Architecture:
[09.atp_interface_protocol.md](../../architecture/en/09.atp_interface_protocol.md),
[10.atp_backend_integration.md](../../architecture/en/10.atp_backend_integration.md);
integration: [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md).

## Resolved And Open Decisions

- **First backend and certificate route: open, resolved by task 15.** Choose
  the first concrete backend from the architecture-10 supported set. Default
  candidate: the route whose proofs map onto MiniSAT-compatible resolution
  traces, because it exercises the kernel acceptance path earliest. Record
  the decision and the version-pinning policy in `backend.md`.
- **Certificate schema ownership: follows `mizar-kernel` task 4.** This
  crate constructs certificate candidates against kernel-owned schema types;
  the kernel never depends on this crate.
- **Externally attested evidence: out of scope here.** Labeling is produced
  by this crate, but the acceptance policy is owned by `mizar-proof`
  (architecture 10 constraints). Registered at the top level.

## Ordered Task List

Keep `cargo test -p mizar-atp` green after each task (see
[Recommended Verification](#recommended-verification)).

### Problem layer

1. **Crate scaffold and lint-policy guard.** [ ]
   - Add the `mizar-atp` workspace member depending on `mizar-session`,
     `mizar-core`, `mizar-vc`, and `mizar-kernel`; add
     `tests/lint_policy.rs` mirroring the `mizar-frontend` guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: `mizar-vc` task 1, `mizar-kernel` task 1. Spec: architecture 09.

2. **Spec: `problem.md`.** [ ]
   - Write the `AtpProblem` data-shape spec (English and Japanese, no code):
     logic profiles, declarations, axioms, conjecture, type context, encoded
     properties, symbol map, `AtpProvenance`, and `expected_result`
     polarity.
   - Deps: 1. Spec: architecture 09 "Backend-Neutral Problem Layer",
     [01.ir_layers.md](../../architecture/en/01.ir_layers.md).

3. **Implement `problem` data shapes.** [ ]
   - Implement `AtpProblem` and provenance tables per task 2, plus a
     deterministic debug rendering.
   - Tests: construction round-trips; every formula traceable through
     provenance; rendering stability.
   - Deps: 2. Spec: `problem.md`.

### Translation

4. **Spec: `translator.md`.** [ ]
   - Write the `VcIr`→`AtpProblem` translation spec (English and Japanese,
     no code): premise materialization, deterministic premise ordering,
     soft-type fact preservation (sort encoding must not erase facts needed
     to justify the VC), and validity-checking polarity.
   - Deps: 2. Spec: architecture 09 "Encoding Strategy"/"Validity Checking
     Polarity".

5. **Declaration and symbol-map translation.** [ ]
   - Translate `VcIr` local contexts and referenced symbols into
     `AtpDeclaration`s with a reversible-enough symbol map for diagnostics.
   - Tests: declaration fixtures; symbol-map round-trips for diagnostics.
   - Deps: 3, 4. Spec: `translator.md`.

6. **Axiom and conjecture translation.** [ ]
   - Materialize cited premises into axioms in deterministic order, encode
     the goal as the conjecture, and attach provenance and
     `expected_result`.
   - Tests: premise-order determinism; provenance completeness; polarity
     fixtures.
   - Deps: 5. Spec: `translator.md`.

7. **Spec: `property_encoding.md`.** [ ]
   - Write the property-encoding spec (English and Japanese, no code): how
     definitional properties (commutativity, …) are encoded as axioms or
     native backend properties, and when each strategy applies.
   - Deps: 4. Spec: architecture 09 "Property Encoding".

8. **Property encoding.** [ ]
   - Implement the property-encoding rules with recorded encoding decisions
     in `EncodedProperty`.
   - Tests: per-property fixtures; backend-extension encodings only under
     profiles that record them.
   - Deps: 6, 7. Spec: `property_encoding.md`.

### Protocol encoders

9. **Spec: `tptp_encoder.md`.** [ ]
   - Write the TPTP emission spec (English and Japanese, no code): dialect
     coverage, name mangling, and deterministic output rules.
   - Deps: 2. Spec: architecture 09 "Supported Formats".

10. **TPTP encoder.** [ ]
    - Emit TPTP text from `AtpProblem` deterministically.
    - Tests: golden-file fixtures; byte-identical output across runs;
      mangling collisions rejected.
    - Deps: 6, 9. Spec: `tptp_encoder.md`.

11. **Spec: `smtlib_encoder.md`.** [ ]
    - Write the SMT-LIB emission spec (English and Japanese, no code): sort
      encoding, logic selection, and deterministic output rules.
    - Deps: 2. Spec: architecture 09 "Supported Formats".

12. **SMT-LIB encoder.** [ ]
    - Emit SMT-LIB text from `AtpProblem` deterministically.
    - Tests: golden-file fixtures; sort-encoding preserves required
      soft-type facts.
    - Deps: 6, 11. Spec: `smtlib_encoder.md`.

### Backend execution

13. **Spec: `backend.md`.** [ ]
    - Write the backend spec (English and Japanese, no code): backend trait,
      process model (spawn, resource limits, termination), configuration
      and version recording, crash handling, and result classification
      including the rule that `Proved` requires matching `expected_result`
      plus evidence.
    - Deps: 2. Spec: architecture 10 "Process Model"/"Result
      Classification", [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
      "Backend Runner".

14. **Backend runner.** [ ]
    - Implement process execution with resource limits, timeouts,
      cancellation, and graceful crash handling; mock backend for tests.
    - Tests: timeout/crash/kill fixtures via mock processes; no zombie
      processes; recorded resource metadata.
    - Deps: 13. Spec: `backend.md`.

15. **First concrete backend integration.** [ ]
    - Resolve the first-backend decision; integrate one real backend
      end-to-end: emit problem, run, parse output into certificate
      candidates against the kernel schema.
    - Tests: integration tests behind a backend-available guard; candidate
      certificates parse under `mizar-kernel`'s structural validation.
    - Deps: 10 or 12 (per chosen backend), 14, `mizar-kernel` task 5. Spec:
      `backend.md`.

16. **Result classification and polarity validation.** [ ]
    - Classify backend outcomes (proved, counterexample, timeout, unknown,
      error); emit `Proved` only when the observed result matches
      `expected_result` and proof evidence is present; counterexamples feed
      diagnostics only.
    - Tests: classification fixtures per outcome; polarity-mismatch cases
      never classify as proved.
    - Deps: 15. Spec: `backend.md` (classification section).

### Portfolio

17. **Spec: `portfolio.md`.** [ ]
    - Write the portfolio spec (English and Japanese, no code): per-VC
      portfolio tasks, candidate evidence collection, early stop, resource
      budgets, and the boundary that winner selection is `mizar-proof`
      policy — completion order never decides results.
    - Deps: 13. Spec: architecture 10 "Portfolio Execution",
      [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
      "ATP Portfolio Service".

18. **Portfolio execution.** [ ]
    - Implement portfolio construction and candidate collection with
      deterministic candidate ordering and cooperative cancellation.
    - Tests: shuffled completion produces identical candidate sets and
      orders; early stop leaves no partial state.
    - Deps: 14, 16, 17. Spec: `portfolio.md`.

19. **ATP run metadata recording.** [ ]
    - Record seeds, timeout settings, backend identities/versions, and
      resource usage for artifacts and reproducibility notes.
    - Tests: metadata completeness fixtures; metadata excluded from
      semantic hashes.
    - Deps: 16. Spec: architecture 00 "Incrementality and Reproducibility".

### Hardening and cross-cutting follow-ups

20. **Corpus and mock-backend integration suite.** [ ]
    - Add `advanced_semantics`-stage corpus cases driven through mock
      backends, plus `spec_trace.toml` entries.
    - Deps: 18. Spec: [staged_model.md](../../mizar-test/en/staged_model.md).

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

Check the task off here once tests pass.

## Notes

- Everything produced here is untrusted evidence; trusted status exists only
  after kernel certificate checking, and acceptance policy lives in
  `mizar-proof`.
- Encoding need not be reversible, but every backend-visible formula must be
  traceable through `AtpProvenance`, and backend-reported used axioms do not
  become artifact `used_axioms` until kernel checking validates them.
- Backend nondeterminism is recorded (seeds, versions, timings), never
  silently absorbed; Mizar-side translation and encoding are bit-stable.
- ATP unavailability must not break earlier phases; this crate degrades to
  `open` VC statuses, not to errors elsewhere in the pipeline.
