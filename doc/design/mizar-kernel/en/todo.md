# mizar-kernel TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs are written by their own spec tasks (English and Japanese in the
same change) before the implementation tasks that cite them.
Module names follow the minimum split of
[internal 07](../../internal/en/07.crate_module_layout.md); the crate refines
architecture 15, 16, and 19 and internal 04. Every module spec must restate
the kernel prohibitions: no proof search, no heuristic selection, no overload
resolution, no cluster search, no ATP search, no implicit coercion insertion,
no fallback inference. The post-closeout SAT-backed correction refines this:
trusted SAT checking is allowed only over the SAT problem that the kernel
derives from caller-supplied formulas, substitutions, provenance, and
target/goal binding; selecting formulas or substitutions remains prohibited.

| Module | Spec | Source | Status |
|---|---|---|---|
| clause | `clause.md` (task 2) | `src/clause.rs` | [x] |
| certificate_parser | `certificate_parser.md` (task 4) | `src/certificate_parser.rs` | [x] |
| formula_evidence | `formula_evidence.md` (task 23) | `src/formula_evidence.rs` | [x] |
| rejection | `rejection.md` (task 6) | `src/rejection.rs` | [x] |
| resolution_trace | `resolution_trace.md` (task 8) | `src/resolution_trace.rs` | [x] |
| sat_encoding | `sat_encoding.md` (task 23) | `src/sat_encoding.rs` | [x] |
| sat_checker | `sat_checker.md` (task 23) | `src/sat_checker.rs` | [x] |
| substitution_checker | `substitution_checker.md` (task 10) | `src/substitution_checker.rs` | [x] |
| checker | `checker.md` (task 13) | `src/checker.rs` | [x] |

`mizar-kernel` implements pipeline phase 14: kernel evidence and immutable
kernel context in, trusted proof status out. It is the trusted core of the whole
verifier (Small Kernel Principle): it verifies evidence only. The
post-closeout target is formula/substitution evidence in, trusted proof status
out: parse and validate the evidence, check provenance, validate and apply
substitutions, derive instantiated formulas, deterministically encode them as
SAT, and accept only when the trusted in-process Rust SAT checker reports
refutation.
The task-22 implementation remains a MiniSAT-compatible resolution-trace
checker and is classified as `source_drift` / `design_drift` until tasks
23-29 replace the acceptance path. A certificate or evidence record is not
acceptance; only this crate's positive result is trusted, and policy
projection on top of it belongs to `mizar-proof`, not here.

Dependency order: `clause` → `certificate_parser` / `rejection` →
`resolution_trace` / `substitution_checker` → legacy `checker`; post-closeout
correction order: `formula_evidence` → `substitution_checker` →
`sat_encoding` / `sat_checker` → `checker` (orchestration, imported facts,
SAT-backed acceptance, cluster replay where explicit evidence exists).

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The crate depends on `mizar-session` and `mizar-core` (core formula
representation and the binder contract it independently re-checks). Task 24
selects the only planned production dependency addition:
`batsat = { version = "=0.6.0", default-features = false }`. Task 27 may add it
only behind the audited `sat_checker` wrapper; it must remain deterministic,
in-process, resource-bounded, and free of backend process execution. Any other
dependency addition requires a recorded justification. `mizar-atp` and
`mizar-proof` depend on this crate, never the reverse. Architecture:
[15.kernel_certificate_format.md](../../architecture/en/15.kernel_certificate_format.md),
[16.substitution_and_binding.md](../../architecture/en/16.substitution_and_binding.md),
[17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md),
[08.reasoning_boundary.md](../../architecture/en/08.reasoning_boundary.md);
integration: [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md).

## Resolved And Open Decisions

- **Formula/substitution evidence correction: open, owned by tasks 23-29.**
  Architecture 15 now supersedes the resolution-trace certificate acceptance
  path for ATP-bound VCs. Kernel evidence stores formulas, substitutions,
  provenance, and target/goal binding; the kernel derives instantiated
  formulas and the deterministic SAT problem. Backend proof methods are not
  trusted evidence. The current source is classified as `source_drift` until
  the post-closeout correction tasks land.
- **Trusted SAT dependency: resolved by task 24, source integration pending
  task 27.** The kernel may trust direct
  `batsat = { version = "=0.6.0", default-features = false }` as part of the
  small kernel after task 27 integrates the wrapper. The audit records version
  pinning, determinism, limits, unsafe usage, no-process/no-network
  constraints, rejected candidates, and lockfile expectations. The kernel must
  not call external SAT/ATP processes or implement ATP search.
- **Legacy certificate schema ownership: resolved by task 4.** Architecture 15
  previously defined the normalized certificate format, and `mizar-kernel`
  owns those legacy certificate schema types, schema-version table, section
  tags, byte grammar, and parser-owned failure locations until the migration
  gate retires or isolates them. Future evidence producers such as `mizar-atp`
  must construct the formula/substitution evidence schema instead; the kernel
  never depends on evidence producers. Producer/consumer integration remains an
  `external_dependency_gap` until those crates exist. This decision is legacy
  for the resolution-trace acceptance path and is superseded by the task-23
  formula/substitution evidence schema for normal proof acceptance.
- **Trusted-baseline crate policy: resolved by task 1 and revised by task 24,
  source guard update pending task 27.** Trusted kernel source
  forbids unsafe code, uses workspace lint denial, keeps production
  dependencies limited to `mizar-session` and `mizar-core` with no
  dev/build/target dependency escape hatches, requires a crate-root trust
  statement, blocks public semantic surface until paired module specs exist,
  and guards against downstream ATP/proof/cache/artifact coupling. Task 24
  revises the policy to allow exactly one audited direct SAT checker dependency
  in addition to `mizar-session` and `mizar-core`; task 27 must encode that
  exact allow-list in `tests/lint_policy.rs` when it edits the manifest.
- **Discharge-evidence validation scope: open, owned by `mizar-proof`
  task 6.** Whether `mizar-vc` pre-ATP discharge evidence is
  kernel-replayed or accepted as policy-level built-in evidence; if
  replay is chosen, the replay checker lands here as a follow-up task.
  Tracked at the top level.

## Ordered Task List

Keep `cargo test -p mizar-kernel` green after each task (see
[Recommended Verification](#recommended-verification)).

### Clause and certificate foundation

1. **Crate scaffold and trusted-baseline lint policy.** [x]
   - Add the `mizar-kernel` workspace member depending on `mizar-session`
     and `mizar-core` only; resolve the trusted-baseline decision and encode
     it in `tests/lint_policy.rs` (deny baseline plus the trusted-code
     additions).
   - Tests: lint-policy guard passes; dependency set is exactly the declared
     one.
   - Deps: `mizar-core` task 5. Spec: internal 07 "Kernel and Proof".

2. **Spec: `clause.md`.** [x]
   - Write the clause-representation spec (English and Japanese, no code)
     per architecture 15 "Clause Representation": literals, canonical
     ordering, structural well-formedness, and the trust statement.
   - Deps: 1. Spec: architecture 15.

3. **Implement clause representation.** [x]
   - Implement clauses with structural validation and deterministic
     rendering.
   - Tests: well-formed/malformed fixtures; canonical ordering; rendering
     stability.
   - Deps: 2. Spec: `clause.md`.

4. **Spec: `certificate_parser.md`.** [x]
   - Write the certificate spec (English and Japanese, no code): top-level
     schema per architecture 15, format tags, backend metadata, structural
     validation rules, and the schema-ownership decision.
   - Deps: 2. Spec: architecture 15 "Certificate Top Level"/"Trust Scope".

5. **Implement certificate parsing and structural validation.** [x]
   - Parse certificates into schema types with structural validation only —
     no semantic trust is granted by parsing.
   - Tests: round-trips; malformed certificates rejected with positions;
     unknown format tags rejected.
   - Deps: 4. Spec: `certificate_parser.md`.

6. **Spec: `rejection.md`.** [x]
   - Write the rejection-semantics spec (English and Japanese, no code):
     stable rejection categories and structured reasons per architecture 15
     "Kernel Rejection Semantics" and architecture 19.
   - Deps: 1. Spec: architecture 15, 
     [19.failure_semantics.md](../../architecture/en/19.failure_semantics.md).

7. **Implement rejection records.** [x]
   - Implement the rejection categories/reasons used by every later checker;
     rejection is a proof error even when a backend reported success.
   - Tests: category stability; reasons carry certificate locations.
   - Deps: 5, 6. Spec: `rejection.md`.

### Checkers

8. **Spec: `resolution_trace.md`.** [x]
   - Write the resolution-trace checking spec (English and Japanese, no
     code): MiniSAT-compatible trace steps, clause-resolution validation,
     and linear replay bounds per architecture 15 "Resolution Trace".
   - Deps: 4. Spec: architecture 15.

9. **Implement the resolution trace checker.** [x]
   - Check clause resolution traces step by step; reject any step that does
     not follow.
   - Tests: valid traces accepted; each single-step mutation rejected;
     replay cost linear in trace size.
   - Deps: 7, 8. Spec: `resolution_trace.md`.

10. **Spec: `substitution_checker.md`.** [x]
    - Write the substitution-checking spec (English and Japanese, no code):
      substitution validation, alpha-conversion checking, and free-variable
      conditions per architecture 15 "Substitution Rule" and architecture
      16, independently re-checking (not reusing the logic of) the
      `mizar-core` binder library.
    - Deps: 4. Spec: architecture 15, 16.

11. **Implement substitution checking.** [x]
    - Validate substitution applications against the certificate's claimed
      results.
    - Tests: valid substitutions accepted; capture violations rejected;
      mismatched results rejected.
    - Deps: 7, 10. Spec: `substitution_checker.md`.

12. **Implement alpha-conversion, freshness, and free-variable checks.** [x]
    - Check alpha-equivalence claims, deterministic freshness witnesses, and
      free-variable side conditions.
    - Tests: equivalence fixtures; freshness-counter mismatches and
      FV-condition violations rejected.
    - Deps: 11. Spec: `substitution_checker.md`.

### Orchestration and acceptance

13. **Spec: `checker.md`.** [x]
    - Write the kernel check-service spec (English and Japanese, no code):
      `KernelCheckInput`/`KernelCheckResult`, the check pipeline over the
      sub-checkers, imported-fact checking per architecture 15, cluster
      trace replay per architecture 17, and acceptance conditions —
      restating the kernel prohibitions.
    - Deps: 6, 8, 10. Spec: architecture 15 "Imported Facts",
      [17.cluster_trace_format.md](../../architecture/en/17.cluster_trace_format.md),
      [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
      "Kernel Check Service".

14. **Implement imported-fact checking.** [x]
    - Validate that facts a certificate uses are exactly the declared
      imported facts (content-addressed references, no silent extras).
    - Tests: undeclared-fact use rejected; hash mismatches rejected.
    - Deps: 13. Spec: `checker.md` (imported-facts section).

15. **Implement cluster trace replay.** [x]
    - Replay `ResolutionTrace` cluster/reduction steps in linear time,
      rejecting traces whose steps do not re-derive their claimed facts.
    - Tests: valid traces replay; mutated antecedents/derived facts
      rejected; replay cost bound enforced.
    - Deps: 13, 14. Spec: `checker.md` (cluster-replay section),
      architecture 17. Upstream `mizar-checker` trace production remains an
      `external_dependency_gap` unless a ready payload contract exists.

16. **Kernel check service and deterministic batch ordering.** [x]
    - Implement the service API: one certificate in, one trusted result out;
      in-crate batch checking with deterministic result ordering by target VC
      fingerprint and caller input order for equal targets.
    - Tests: service round-trips; batch order determinism under shuffled
      caller input order and equal-target ties.
    - Deps: 9, 12, 14, 15. Spec: `checker.md`,
      [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md).

### Hardening and cross-cutting follow-ups

17. **Soundness fail-test corpus.** [x]
    - Build the mutation-based soundness suite: every checker gets
      systematically mutated certificates/traces that must be rejected
      (fail-heavy per the test strategy and
      [fail_soundness.md](../../mizar-test/en/fail_soundness.md)).
    - Deps: 16. Spec: [fail_soundness.md](../../mizar-test/en/fail_soundness.md),
      [20.test_strategy.md](../../architecture/en/20.test_strategy.md).

18. **Determinism and replay-cost suite.** [x]
    - Property coverage that identical inputs produce identical results and
      rejection reasons, and that replay stays within the documented cost
      bounds.
    - Deps: 16. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).

19. **Public-enum forward-compatibility policy.** [x]
    - Apply the `mizar-frontend` task-25 procedure to each public enum;
      rejection categories additionally follow the architecture 19
      compatibility policy.
    - Deps: 16. Spec: [public_enum_policy.md](./public_enum_policy.md)
      and module specs referenced by its inventory.

20. **Source/spec correspondence and prohibition audit.** [x]
    - Trace every public API and promised behavior to implementation and
      tests; verify every module spec restates the kernel prohibitions and
      its trust statement.
    - Deps: 19. Spec: all module specs and this TODO.

21. **Bilingual documentation sync audit.** [x]
    - Compare each English canonical document under
      `doc/design/mizar-kernel/en/` with its Japanese companion and
      synchronize content.
    - Deps: 20. Spec: repository documentation policy.

22. **Module-boundary refactor gate.** [x]
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
    - Deps: 21. Spec: this TODO,
      [internal 07](../../internal/en/07.crate_module_layout.md), all module
      specs.

### Post-closeout SAT-backed evidence correction

23. **Spec: kernel evidence format correction.** [x]
    - Update the paired module specs to supersede resolution-trace
      certificates with formula/substitution kernel evidence. Classify the
      legacy resolution-trace acceptance path as `design_drift` /
      `source_drift`, record external producer gaps, and restate that SAT
      checking over supplied evidence is allowed while proof search remains
      prohibited.
      Task 23 adds paired `formula_evidence.md`, `sat_encoding.md`, and
      `sat_checker.md`; updates the proof-related language spec text; updates
      checker/rejection/resolution-trace docs; and records that legacy
      certificate/trace inputs are normal-policy unsupported and
      migration/audit-only.
    - Tests: docs-only verification.
    - Deps: 22. Spec:
      [15.kernel_certificate_format.md](../../architecture/en/15.kernel_certificate_format.md),
      [08.reasoning_boundary.md](../../architecture/en/08.reasoning_boundary.md),
      [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md).

24. **Spec and audit: trusted SAT checker dependency.** [x]
    - Choose and justify the pure-Rust MiniSAT-compatible SAT checker to trust
      inside the kernel. Record version pinning, determinism requirements,
      resource limits, unsafe-code audit, no-process/no-network constraints,
      the lint/dependency policy revision from the task-1 baseline, and the
      wrapper API expected by `sat_checker.md`.
      Result: task 24 selects
      `batsat = { version = "=0.6.0", default-features = false }`, with
      expected transitive lockfile resolution `bit-vec 0.5.1`; integration is
      deferred to task 27.
    - Tests: docs-only verification plus dependency metadata audit once a
      candidate crate is selected.
    - Deps: 23. Spec: architecture 15 "Post-Closeout Correction".

25. **Formula/substitution evidence schema and parser.** [x]
    - Implement the kernel-owned evidence schema for formula refs or formulas,
      substitution records, provenance bindings, target/goal binding, and
      stable hashes. Legacy certificate parsing may remain for compatibility
      only if it is clearly outside the new acceptance path.
    - Tests: structural round-trips; malformed evidence rejected; provenance
      gaps reject fail-closed; deterministic rendering and hashing.
    - Deps: 23, 24. Spec: `formula_evidence.md` from task 23.

26. **Formula instantiation and deterministic SAT encoding.** [x]
    - Validate substitution side conditions, derive instantiated formulas
      from the evidence formulas, and encode the resulting formula set plus
      negated/target goal as a deterministic SAT problem. Instantiated
      formulas and SAT clauses are kernel-derived check artifacts, not
      trusted input. Encoding must not choose premises, invent substitutions,
      or hide backend-method proof traces in the trusted input.
    - Tests: valid instantiations encode stably; capture and provenance
      mutations reject; equivalent caller order produces identical SAT bytes.
    - Deps: 25. Spec: `formula_evidence.md`, `sat_encoding.md`,
      [16.substitution_and_binding.md](../../architecture/en/16.substitution_and_binding.md).

27. **Trusted SAT checker wrapper.** [x]
    - Integrate the audited Rust SAT checker behind a small deterministic
      wrapper. Expose only the operations needed to decide whether the
      kernel-built SAT problem is unsatisfiable; enforce limits and convert
      solver errors to stable kernel rejections.
    - Tests: satisfiable kernel-derived SAT problems return non-acceptance
      wrapper evidence; unsatisfiable problems return UNSAT wrapper evidence;
      limits, unsupported clauses, and solver errors reject deterministically;
      dependency and lockfile guards enforce exact `batsat`/`bit-vec`
      resolution and reject alternate SAT/process dependencies; wrapper tests
      prove deterministic `batsat` heuristic options are pinned and not exposed
      to callers.
    - Deps: 24, 26. Spec: `sat_checker.md` from task 23.

28. **SAT-backed kernel check service.** [x]
    - Replace the trusted acceptance path so `checker` accepts only validated
      formula/substitution evidence whose kernel-derived SAT problem is
      refuted by the trusted SAT checker. Keep imported-fact, provenance,
      cluster-trace, and used-axiom extraction fail-closed.
    - Tests: end-to-end accepted/rejected evidence fixtures; mutated
      substitutions, missing premises, satisfiable goals, and context
      mismatches reject; batch ordering remains deterministic.
    - Deps: 25, 26, 27. Spec: `checker.md`,
      [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md).

29. **Migration audit and quality re-review.** [x]
    - Retire, gate, or explicitly mark the legacy resolution-trace public
      surface so downstream crates cannot mistake it for the target
      acceptance path. Re-run source/spec, bilingual, prohibition, dependency,
      and quality audits; record remaining `external_dependency_gap` items
      for `mizar-vc`, `mizar-atp`, `mizar-proof`, `mizar-cache`, and
      `mizar-artifact`.
    - Tests: full `mizar-kernel` verification, workspace verification when
      practical, docs diff checks, and a quality review score of at least
      90/100 before any new closeout claim.
    - Deps: 28. Spec: this TODO and autonomous crate exit criteria.

### Soundness-audit follow-ups (2026-07-03)

[soundness_argument.md](./soundness_argument.md) audited the trusted
acceptance boundary before further implementation and recorded findings F1-F9
plus the 23-case reject-first corpus under `tests/certificates/fail/`. Two
High findings were patched at the architecture level in the same change
(`f75af877`); the tasks below carry the implementation and schema work.
Every finding maps to a task or a recorded disposition:

| Finding | Disposition |
|---|---|
| F1 (goal polarity) | architecture 15 patched; kernel implementation is task 30; producer-side polarity declaration/rejection is resolved by [mizar-vc task 27](../../mizar-vc/en/todo.md) |
| F2 (non-imported source bindings) | architecture 15 patched fail-closed; schema and verification are task 31, paired with the mizar-vc context-identity payload |
| F3 (solver step budget) | deferred by design in `sat_checker.md` (batsat 0.6.0 exposes no stable budget API); revisit trigger recorded as task 32 |
| F4 (KernelEvidence field drift) | resolved in `f75af877`; no further task |
| F5 (fingerprint collision resistance) | constraint added to architecture 15 in `f75af877`; no further task — future fingerprint registrations must satisfy it |
| F6 (imported-statement projection) | task 33, paired with mizar-vc |
| F7 (mizar-test soundness vocabulary) | owned by [mizar-test](../../mizar-test/en/todo.md) audit follow-up |
| F8 (corpus directory naming) | owned by [mizar-test](../../mizar-test/en/todo.md) audit follow-up |
| F9 (legacy tautology marker) | task 34 |

30. **Goal-polarity binding in the check service (F1, invariant B4).** [ ]
    - Implement architecture 15 "Goal Polarity Is Bound By The Target
      Obligation": the task-28 check service must read the check kind from
      the caller's immutable kernel context and reject evidence whose
      declared goal polarity does not match it as `context_mismatch`.
      Proof obligations require refutation polarity; `AssertTrueForConsistency`
      is acceptable only for explicitly consistency-kind checks.
    - Acceptance: `fail_certificate_sat_goal_polarity_mismatch_001` rejects
      for the polarity reason (not an earlier structural reason); a Rust
      regression covers both polarities against both check kinds; invariant
      B4 in `soundness_argument.md` is marked implemented.
    - Verify: `cargo test -p mizar-kernel`, `cargo test -p mizar-test`,
      `cargo clippy -p mizar-kernel --all-targets -- -D warnings`.
    - Deps: 28. Spec: architecture 15 (post-audit text), `checker.md`,
      `sat_encoding.md`; soundness_argument.md F1/B4.

31. **Context-identity verification for non-imported source bindings (F2, invariant P-class).** [ ]
    - Specify and implement the verification data for local-hypothesis,
      cited-premise, and generated-VC-fact source bindings: extend
      `FormulaEvidenceContext` (or the immutable kernel context) to carry the
      canonical `mizar-vc` kernel-evidence handoff hash, and require each
      non-imported binding to be checkable against that hash before
      acceptance. Keep the current fail-closed `missing_provenance` behavior
      for evidence that lacks the payload. Update `formula_evidence.md` and
      architecture 15 in the same change; the producer-side payload is the
      paired mizar-vc task.
    - Acceptance:
      `fail_certificate_symbols_unverifiable_local_hypothesis_001` rejects
      for the provenance reason; a pass fixture with a valid context-identity
      payload accepts once the mizar-vc payload contract exists; an
      ATP-labeled goal-as-hypothesis mutation rejects.
    - Verify: `cargo test -p mizar-kernel`, `cargo test -p mizar-test`,
      `cargo clippy -p mizar-kernel --all-targets -- -D warnings`.
    - Deps: 30; paired: mizar-vc context-identity payload task. Spec:
      architecture 15 "Context Identity Covers Non-Imported Source Bindings",
      `formula_evidence.md`; soundness_argument.md F2, edge case 5.

32. **Solver step-budget deferral revisit (F3).** [ ]
    - Availability-only gap: checker `timeout` cannot fire during
      `batsat` solving because 0.6.0 exposes no deterministic
      conflict/propagation budget. Re-evaluate when the pinned dependency
      changes: either adopt a budget API (new pinned version or replacement
      audited per the task-24 procedure) or re-record the deferral with the
      size-limit rationale. Do not weaken determinism or add process
      execution to gain interruptibility.
    - Acceptance: a recorded decision in `sat_checker.md` (en+ja) citing the
      dependency version; if a budget lands, resource-rejection tests cover
      mid-solve budget exhaustion deterministically.
    - Verify: `cargo test -p mizar-kernel`; dependency/lockfile guards.
    - Deps: triggered by any `batsat` version change (task-24 audit
      procedure). Spec: `sat_checker.md`; soundness_argument.md F3.

33. **Imported-statement projection specification (F6).** [ ]
    - Specify the projection from arch-18 imported statement fingerprints
      (rich formulas) to the propositional formula-tree fingerprints the
      evidence checker compares, so realistic imported facts become citable.
      Until this lands the fingerprint-equality rule keeps import citations
      fail-closed (sound). Kernel side: projection validation rules in
      `formula_evidence.md` + architecture 15; producer side is the paired
      mizar-vc/mizar-atp schema work.
    - Acceptance: the projection is deterministic and collision-resistant
      per the F5 constraint; a pass fixture cites a projected imported
      statement; mutation fixtures (wrong projection, stale fingerprint)
      reject; invariant F-class rows updated in `soundness_argument.md`.
    - Verify: `cargo test -p mizar-kernel`, `cargo test -p mizar-test`.
    - Deps: 31; paired: mizar-vc dependency-slice/import projection task.
      Spec: architecture 15, 18; soundness_argument.md F6.

34. **Legacy tautology-marker semantics (F9, low).** [ ]
    - Pin down or retire the legacy resolution-trace tautology marker: its
      current meaning is profile-dependent and thinly specified. Preferred:
      retire it together with the migration/audit-only legacy path; if kept
      for the audit profile, specify its exact acceptance effect in
      `resolution_trace.md` (en+ja).
    - Acceptance: either the marker is unreachable under every policy and
      documented as such, or its semantics have a spec section and
      mutation-rejection tests. Mislabeling must remain premise-weakening
      only (no acceptance strengthening).
    - Verify: `cargo test -p mizar-kernel`.
    - Deps: 29. Spec: `resolution_trace.md`; soundness_argument.md F9, L-class
      invariants.

35. **Soundness-argument revisit for the reduct-view encoding.** [ ]
    - The template-encoding audit
      ([template_encoding_audit.md](../../mizar-core/en/template_encoding_audit.md))
      replaced flattened structure widening with reduct-view terms
      (`view_{D→B}`) in spec 05/13. Once the mizar-core view lowering lands,
      revisit `soundness_argument.md`'s assumption that attribute predicates
      are atomic per subject: certificates mentioning structure widening
      change shape, and the F-class formula invariants plus the corpus seeds
      touching attribute atoms must be re-checked against view terms.
    - Acceptance: `soundness_argument.md` (en+ja) records the re-audit
      result; any invariant change updates the corpus sidecar notes in the
      same change (per the document's constraint section).
    - Verify: `cargo test -p mizar-kernel`, `cargo test -p mizar-test`.
    - Deps: external — mizar-core reduct/view lowering task; then 31. Spec:
      spec 05 §5.8.3, 13 §13.8.7; template_encoding_audit.md F1/F3.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-kernel
cargo clippy -p mizar-kernel --all-targets -- -D warnings
```

For tasks that touch the core binder contract or cluster replay, also run:

```text
cargo test -p mizar-core
cargo test -p mizar-checker
```

Check the task off here once tests pass.

## Notes

- The kernel verifies evidence only. It must never perform proof search,
  heuristic selection, overload resolution, cluster search, ATP search,
  implicit coercion insertion, or fallback inference.
- Trusted SAT checking over already supplied formula/substitution evidence is
  an evidence check, not proof search. The kernel still must not choose
  premises, invent substitutions, call ATP backends, or perform fallback
  inference.
- Kernel evidence validation failure is a proof error even if the backend
  reported success; externally attested evidence is `mizar-proof` policy,
  never a kernel result.
- Keep the dependency set minimal and audited; soundness-relevant code
  favors duplication over shared cleverness (the substitution checker
  re-checks, it does not reuse).
- Fail/soundness tests take priority over pass tests near this crate.
