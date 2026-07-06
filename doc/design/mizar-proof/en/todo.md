# mizar-proof TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs were written by their own spec tasks (English and Japanese in the
same change) before the implementation tasks that cite them. Module names
follow the minimum split of
[internal 07](../../internal/en/07.crate_module_layout.md) (`policy`,
`witness_store`) plus the selection/status projections of internal 04; the
task-19 layout keeps those public module boundaries and moves crate-private
unit tests into child modules.

| Module | Spec | Source | Status |
|---|---|---|---|
| policy | `policy.md` (task 2) | `src/policy.rs`; private tests in `src/policy/tests.rs` | [x] |
| selection | `selection.md` (task 5) | `src/selection.rs`; private tests in `src/selection/tests.rs` | [x] |
| status | `status.md` (task 8) | `src/status.rs`; private tests in `src/status/tests.rs` | [x] |
| witness_store | `witness_store.md` (task 10) | `src/witness_store.rs`; private tests in `src/witness_store/tests.rs` | [x] |

`mizar-proof` owns the policy layer between untrusted evidence production
(`mizar-atp`, `mizar-vc` discharge) and trusted validation (`mizar-kernel`):
the `ProofPolicyEvaluator` (candidate classes, externally-attested rules,
`require_kernel_certificates`, open-obligation allowances), deterministic
winner selection over portfolio candidates, artifact-facing proof status
projection (`kernel_verified`, `discharged_builtin`, policy-controlled
external and open statuses), and the proof witness store staged for artifact
commit. Policy outcomes are always distinct from trusted proof status: this
crate never makes anything "more proven" — it decides what is recorded,
selected, and published.

Dependency order: `policy` → `selection` → `status` → `witness_store`.

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The crate depends on `mizar-session`, `mizar-kernel` (`KernelCheckResult`,
certificate schema), `mizar-vc` (`VcId`, discharge evidence), `mizar-atp`
(portfolio candidates), and `mizar-artifact` (witness reference schema). The
kernel never depends on this crate. Architecture:
[08.reasoning_boundary.md](../../architecture/en/08.reasoning_boundary.md),
[15.kernel_certificate_format.md](../../architecture/en/15.kernel_certificate_format.md);
internal: [04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md).

## Resolved And Open Decisions

- **Policy/trust split: resolved by internal 04.** The kernel returns
  policy-independent validation results; this crate evaluates policy on top
  of them. Externally attested evidence is policy-recorded evidence, never
  trusted status, and cannot win when `require_kernel_certificates` is set.
- **Discharge-evidence validation scope: resolved by task 6.**
  `DischargedBuiltin` enters trusted selection only through
  `TrustedKernelEvidence` created from a proof-obligation `KernelPolicyInput`.
  Public callers can construct `KernelPolicyInput` from a real
  `KernelCheckResult` plus explicit origin, but trusted selection admits it
  only when that result is accepted, untainted, and proof-obligation evidence.
  Pre-ATP discharge must therefore be kernel-replayed or represented as
  accepted proof-obligation kernel primitive evidence; accepted consistency
  checks and uncheckable discharge remain diagnostic/policy evidence and
  cannot publish trusted `used_axioms`.
- **Policy fingerprint surface: resolved by task 2, implemented by task 3.**
  `policy.md` defines the settings that enter `PolicyFingerprint`; coordinate
  future cache integration with `mizar-cache` task 2.

## Ordered Task List

Keep `cargo test -p mizar-proof` green after each task (see
[Recommended Verification](#recommended-verification)).

### Protocol prerequisite

0. **Crate plan and task ledger.** [x]
   - Added paired crate plans and ledgers:
     [00.crate_plan.md](./00.crate_plan.md) and
     [task_ledger.md](./task_ledger.md).
   - Status: complete for kickoff. Implementation starts at task 1 after this
     docs-only task is reviewed, verified, and committed.

### Policy

1. **Crate scaffold and lint-policy guard.** [x]
   - Add the `mizar-proof` workspace member depending on `mizar-session`,
     `mizar-kernel`, `mizar-vc`, `mizar-atp`, and `mizar-artifact`; add
     `tests/lint_policy.rs` mirroring the `mizar-frontend` guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: `mizar-kernel` task 1, `mizar-atp` task 1. Spec: internal 04.
   - Status: scaffold crate and lint-policy guard added. Policy, selection,
     status, and witness-store modules remain unavailable until their paired
     specs land in later tasks.

2. **Spec: `policy.md`.** [x]
   - Write the policy spec (English and Japanese, no code): verifier policy
     settings, `CandidatePolicyClass`, externally-attested admission rules,
     `require_kernel_certificates`, open-obligation allowances per build
     mode, and the rule that policy outcomes are distinct from trusted
     status.
   - Deps: 1. Spec: [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
     "Proof Policy Evaluator", architecture 08.
   - Status: paired specs added; implementation begins in task 3.

3. **Policy evaluator.** [x]
   - Implement `ProofPolicyEvaluator`: candidate classification,
     `can_schedule_kernel_check`, and the policy-fingerprint projection defined
     by `policy.md`; future cache integration remains coordinated with
     `mizar-cache` task 2.
   - Define the local normalized policy input wrapper that pairs kernel
     results with explicit evidence origin. Do not infer origin from
     `KernelCheckResult` alone.
   - External evidence support in this task is the base classifier shape only;
     the full admission matrix and stable rejection diagnostics remain task 4.
   - Tests: classification fixtures per evidence kind; fingerprint changes
     iff a policy-relevant setting changes.
   - Deps: 2. Spec: `policy.md`.
   - Status: implemented `src/policy.rs` with normalized kernel-origin inputs,
     base external classifier shape, schedulability checks, deterministic
     policy fingerprinting, and focused tests.

4. **Externally attested evidence handling.** [x]
   - Implement admission and labeling of externally attested evidence:
     recordable as development evidence when the profile allows, never
     winning under `require_kernel_certificates`, never producing trusted
     `used_axioms`.
   - Tests: admission matrices per profile; rejection diagnostics stable.
   - Deps: 3. Spec: `policy.md` (externally attested evidence section).
   - Status: implemented `ExternalEvidenceAdmission`, concrete publication
     labels, stable policy diagnostics, policy-tainted kernel-result routing,
     and profile/requirement matrix tests.

### Selection and status

5. **Spec: `selection.md`.** [x]
   - Write the winner-selection spec (English and Japanese, no code): the
     deterministic ordering classes (kernel-verified satisfying the active
     policy → discharged built-in → policy-permitted external → policy
     assumption → best-explained open), tie-break keys (backend profile
     priority, certificate format priority, encoded problem hash, profile id,
     stable candidate id), the selected proof witness hash or
     deterministic discharge hash exported for reuse validation, and the
     completion-time prohibition.
   - Deps: 2. Spec: [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
     "Winner Selection".
   - Status: paired specs added; implementation begins in task 6.

6. **Winner selection.** [x]
   - Implement deterministic winner selection over
     `ProofEvidenceSet`s; resolve and record the discharge-evidence
     validation-scope decision (with `mizar-kernel`), since it determines
     which class built-in discharge evidence enters.
   - Tests: ordering fixtures across classes and tie-breaks; shuffled
     candidate arrival never changes the winner.
   - Deps: 3, 5, `mizar-kernel` task 16. Spec: `selection.md`.
   - Status: implemented `src/selection.rs` with required stable candidate ids,
     trusted-kernel evidence markers, deterministic winner/rejection ordering,
     no-selectable diagnostic outcomes, reuse metadata, and focused tests.

7. **Artifact proof selection merge.** [x]
   - Merge portfolio results with phase-12 built-in discharge results per
     `VcId` into `kernel_verified` / `discharged_builtin` selections, with
     external and open statuses kept distinguishable.
   - Tests: merge fixtures per combination; no status collapses into
     another.
   - Deps: 6, `mizar-vc` task 12. Spec: `selection.md` (merge section),
     [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
     "Artifact Proof Selection".
   - Status: implemented `merge_artifact_proof_selections` with canonical
     `VcId` ordering, duplicate-source rejection, trusted class precedence,
     source/class compatibility validation, and preservation of non-trusted
     outcomes for later status projection.

8. **Spec: `status.md`.** [x]
   - Write the status-projection spec (English and Japanese, no code): the
     artifact- and diagnostics-facing proof status model, trusted
     `used_axioms` propagation (only from accepted proof-obligation kernel
     evidence), and
     explanation references for open/rejected obligations.
   - Deps: 5. Spec: [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
     "Proof Witness and Artifact Flow", architecture 19, architecture 22.
   - Status: added paired `status.md` specs covering projection inputs,
     selection-to-status mapping, trusted `used_axioms` boundaries,
     diagnostic/explanation references, artifact projection limits, proof
     reuse metadata, and deferred/external dependency gaps.

9. **Proof status projection.** [x]
   - Implement status projection for artifacts and diagnostics, including
     trusted `used_axioms` extraction boundaries.
   - Tests: projection fixtures per selection outcome; `used_axioms` only
     from accepted proof-obligation kernel evidence.
   - Deps: 7, 8. Spec: `status.md`.
   - Status: implemented `src/status.rs` with status projection inputs,
     projected/internal status classes, current artifact publication
     availability, trusted used-axiom references derived only from accepted
     proof-obligation kernel results, architecture-22 reuse metadata, and
     fixtures per selected outcome.

### Witness store

10. **Spec: `witness_store.md`.** [x]
   - Write the witness-store spec (English and Japanese, no code): the
     stage/publish flow (`stage` before commit, `publish_ref` only after
     the artifact manifest references the witness), stable content hashing
     used as proof witness hashes, and provenance metadata.
   - Deps: 2. Spec: [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
     "Proof Witness Store".
   - Status: added paired `witness_store.md` specs covering proof witness
     draft inputs, stage/publish state transitions, stable artifact-framed
     hashing, publication references after committed manifest reachability,
     provenance metadata, cache/reuse boundaries, and deferred
     `DischargedBuiltin` artifact-witness support.

11. **Witness store implementation.** [x]
    - Implement staging and publication of `ProofWitnessDraft`s against the
      `mizar-artifact` witness-reference schema.
    - Tests: stage/publish round-trips; publication before committed
      manifest-reachability proof fails; hashes recorded before commit;
      non-trusted evidence cannot publish trusted witnesses;
      `DischargedBuiltin` remains an unsupported/external-dependency gap until
      artifact schema support exists.
    - Deps: 9, 10, `mizar-artifact` task 9. Spec: `witness_store.md`.
   - Status: implemented `src/witness_store.rs` with `ProofWitnessDraft`
     construction from status projection plus opaque kernel-derived witness
     metadata, `ProofWitnessStagedRef`, unpublished `ProofWitnessRef`
     candidates for `KernelVerified`, opaque `CommittedWitnessPublicationProof`
     publication tokens, `publish_ref` reachability validation, stable witness
     payload artifact hashing, provenance/status consistency checks, invalid
     schema/path rejection, and the `DischargedBuiltin` unsupported-witness
     gap.

12. **Portfolio early-stop policy hooks.** [x]
    - Provide the policy queries the ATP portfolio uses for early stop
      (no-better-class-possible checks), keeping termination decisions
      policy-driven, not time-driven.
    - Tests: early-stop fixtures per policy; stopping never changes the
      selected winner versus running to completion.
    - Deps: 6, `mizar-atp` task 18. Spec: `policy.md`,
      [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
      "Early Stop and Cancellation".
   - Status: implemented `PortfolioEarlyStopInput`,
     `PortfolioEarlyStopDecision`, `PortfolioEarlyStopClass`, and stable
     `PortfolioEarlyStopReason` values in `src/policy.rs`, plus policy-driven
     `best_possible_early_stop_class` normalization and class-level finality
     decisions. Equal or higher pending selectable classes block early stop by
     rank, external evidence remains blocked when kernel certificates are
     required, policy-tainted kernel output stays non-trusted, and selector
     equivalence/public API tests cover the stable surface. Downstream
     `mizar-atp` adoption, process cancellation wiring, and live backend-state
     summaries remain deferred `external_dependency_gap` work.

### Hardening and cross-cutting follow-ups

13. **Determinism suite.** [x]
    - Property coverage that identical evidence sets produce identical
      classifications, winners, statuses, and witness references under
      shuffled arrival orders.
    - Deps: 11, 12. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).
   - Status: added `tests/determinism_suite.rs` for public policy
     classification, early-stop normalization, deterministic selection, status
     projection, and reuse-metadata stability across shuffled candidate order.
     Added `witness_store.rs` unit coverage for staged and published witness
     reference determinism using the existing crate-private publication-token
     fixture, preserving the artifact-boundary opacity.

14. **Public-enum forward-compatibility policy.** [x]
    - Apply the `mizar-frontend` task-25 procedure to each public enum;
      status enums additionally follow the artifact compatibility policy.
    - Deps: 11. Spec: all module specs.
   - Status: audited all 26 public enums owned by `policy`, `selection`,
     `status`, and `witness_store`; no exhaustive public enum exceptions are
     permitted. Added the missing `#[non_exhaustive]` markers for
     `SelectionInputError` and `ArtifactProofSelectionError`, documented each
     enum in the paired module specs, recorded artifact-compatibility review
     requirements for status-facing enums, and added a lint guard that scans
     every `src/**/*.rs` file and both EN/JA specs for enum-policy drift.

15. **Source/spec correspondence audit.** [x]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; verify the policy/trust split is restated
      in every module spec.
    - Deps: 14. Spec: all module specs and this TODO.
    - Status: added paired
      [`source_spec_audit.md`](./source_spec_audit.md) docs. The audit traces
      public API groups for policy, selection, status, and witness-store to
      their owning module specs and test coverage; verifies the policy/trust
      split in every module; and finds no blocking `spec_gap`, `test_gap`,
      `design_drift`, `source_drift`, `boundary_violation`, or
      `repo_metadata_conflict`. Focused unit tests now cover empty candidate
      source ids, empty obligation identity fields, and required canonical
      witness payload bytes. Remaining work is classified as deferred or
      `external_dependency_gap` for task-17 cache-facing reuse export,
      `DischargedBuiltin` artifact witness support, artifact publication
      tokens, copied kernel metadata, payload canonicality validators, and
      downstream ATP early-stop integration.

16. **Bilingual documentation sync audit.** [x]
    - Compare each English canonical document under
      `doc/design/mizar-proof/en/` with its Japanese companion and
      synchronize content.
    - Deps: 15. Spec: repository documentation policy.
    - Status: added paired
      [`bilingual_sync_audit.md`](./bilingual_sync_audit.md) docs. The audit
      confirms that all English canonical files have Japanese companions, no
      Japanese placeholder remains, trust-boundary and deferred-gap wording is
      synchronized, and the only required sync edits were task-local metadata
      updates for the crate plan, TODO, and task ledger.

17. **Proof-reuse metadata export contract.** [x]
    - Expose the proof-reuse metadata consumed by `mizar-cache`: compatible
      verifier-policy fingerprint, `ObligationAnchor`, canonical VC,
      local-context, and dependency-slice fingerprints, selected proof witness
      hash or deterministic discharge hash, matching proof-evidence identity,
      dependency artifact/schema compatibility, evidence class,
      selected-candidate provenance, and the selection reason. This metadata
      follows `status.md` and architecture 22; it is a validation predicate for
      reuse, not trusted proof status.
    - Tests: changing any exported reuse component changes or invalidates the
      reuse predicate; shuffled candidate arrival preserves the same exported
      metadata; externally attested evidence remains externally attested and is
      never upgraded by metadata reuse.
    - Deps: 6, 7, 9, 11, 13. Spec:
      [22.incremental_verification_contract.md](../../architecture/en/22.incremental_verification_contract.md),
      [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md),
      [11.artifact_and_incremental_build.md](../../architecture/en/11.artifact_and_incremental_build.md).
    - Status: extended selection/status metadata with selected-candidate
      provenance, stable selection reason, proof-evidence identity, dependency
      artifact/schema compatibility, and a proof-reuse validation hash.
      `cache_reuse_predicate_complete` is class-aware: `KernelVerified`
      requires a selected witness hash, `DischargedBuiltin` requires a
      deterministic discharge hash, and non-trusted classes remain metadata
      only. No cache lookup, cache authority, trusted-status promotion, or
      external-evidence upgrade was added.

18. **Architecture-22 follow-up audit.** [x]
    - Re-run the source/spec correspondence and bilingual documentation sync
      audits for the task-17 reuse-metadata export contract; record any
      remaining trust-boundary, witness-hash, deterministic discharge, or
      policy-selection gaps as follow-up tasks.
    - Deps: 17. Spec: all module specs, this TODO, and repository
      documentation policy.
    - Status: added paired
      [`architecture_22_audit.md`](./architecture_22_audit.md) docs. The audit
      confirms task-17 reuse metadata remains a validation contract, not proof
      authority; records remaining cache, artifact witness, committed
      publication, kernel-metadata, ATP integration, and non-blocking branch
      coverage gaps; and reports the `mizar-atp` closeout guard mismatch as a
      `repo_metadata_conflict` without repairing it in this task.

19. **Module-boundary refactor gate.** [x]
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
    - Deps: 18. Spec: this TODO,
      [internal 07](../../internal/en/07.crate_module_layout.md), all module
      specs.
    - Status: added paired
      [`module_boundary_audit.md`](./module_boundary_audit.md) docs and
      performed a move-only split of large inline unit-test modules into
      private child modules: `src/policy/tests.rs`,
      `src/selection/tests.rs`, `src/status/tests.rs`, and
      `src/witness_store/tests.rs`. Parent production modules now declare
      `#[cfg(test)] mod tests;`, public APIs and deterministic behavior are
      unchanged, and the lint-policy source-tree guard allows only these
      private test submodules as the task-19 layout change. Further production
      helper splits remain deferred unless task 20 or downstream consumers
      identify a concrete review bottleneck.

20. **Crate exit report and quality review.** [x]
    - Record hard gates, quality score, verification results, all task
      commits, deferred items, `repo_metadata_conflict` status, and the
      next-phase handoff.
    - Deps: 19. Spec: this TODO,
      [autonomous crate development](../../autonomous_crate_development.md),
      all module specs and audits.
    - Status: added paired
      [`crate_exit_report.md`](./crate_exit_report.md) docs. The closeout
      records `mizar-proof` as complete for the proof-policy milestone with
      quality score 94/100. The stale `mizar-atp` task-28
      `repo_metadata_conflict` discovered during closeout was resolved by
      focused metadata correction commit `36d1a9c`; full-workspace verification
      is clean after that correction.

### Kernel soundness-audit alignment (2026-07-03)

The kernel acceptance-boundary audit
([soundness_argument.md](../../mizar-kernel/en/soundness_argument.md))
corrected the kernel rejection vocabulary and evidence contract. The policy
layer consumes both; one alignment task is owned here.

21. **Policy alignment with the corrected kernel rejection taxonomy (kernel F1, F2).** [x]
    - Align proof-policy evaluation and status projection with the
      post-audit kernel contract: `invalid_sat_refutation`,
      `context_mismatch`, `missing_provenance`, and the legacy-path gate are
      terminal proof errors that policy must never upgrade, retry into
      acceptance, or reclassify as policy-open; selection/status reuse
      metadata records the goal polarity of accepted proof-obligation evidence
      so reuse consumers can validate it. No evidence-class upgrades (IV-005
      stays intact).
    - Acceptance: Rust regressions cover the source-backed corrected rejection
      reasons available through the public kernel evidence API
      (`invalid_sat_refutation`, goal-polarity `context_mismatch`, and
      `missing_provenance`) flowing through policy evaluation to failed
      status. The legacy unsupported-certificate gate must remain terminal
      when a real kernel record is supplied, but proof-side real-payload
      regression coverage is an `external_dependency_gap` until a public
      kernel legacy-certificate fixture or source runner is available. A
      polarity-bearing selection/status reuse metadata fixture round-trips
      through the task-17 reuse export.
    - Verify: `cargo test -p mizar-proof`,
      `cargo clippy -p mizar-proof --all-targets --all-features -- -D warnings`.
    - Deps: mizar-kernel tasks 30-31; coordinate with the mizar-test
      soundness-registry follow-up (kernel F7). Spec: architecture 15
      (post-audit), 19; soundness_argument.md rejection taxonomy.
    - Status: complete. `KernelPolicyInput` exposes accepted goal polarity only
      for accepted proof-obligation kernel results; policy decisions preserve
      rejected kernel-evidence check kind; selection records accepted polarity
      for trusted winners. Source-backed corrected terminal kernel rejections
      can displace an otherwise-open fallback but cannot outrank trusted
      accepted, policy-permitted external, or policy-assumed evidence.
      Target-binding `context_mismatch` and consistency-check goal-polarity
      mismatch remain ordinary rejected diagnostics and do not displace
      policy-open fallback. Legacy unsupported-certificate real-payload
      coverage is classified as an `external_dependency_gap`. Reviews reported
      no findings after focused fixes, full workspace verification passed, and
      `doc/design/spec_coverage_audit.md` remains unchanged because spec
      coverage ownership, traceability metadata, owner crates, and deferred
      coverage status did not change.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-proof
cargo clippy -p mizar-proof --all-targets --all-features -- -D warnings
```

For tasks that touch kernel, ATP, or artifact boundaries, also run:

```text
cargo test -p mizar-kernel
cargo test -p mizar-atp
cargo test -p mizar-artifact
```

For proof-reuse metadata tasks, also run:

```text
cargo test -p mizar-cache
```

Check the task off here once tests pass.

## Notes

- Policy outcomes are distinct from trusted proof status; this crate never
  upgrades evidence — the kernel's positive result is the only source of
  trust.
- Winner selection is deterministic and policy-driven; raw completion time
  is recorded but never decides anything.
- Externally attested evidence never produces trusted `used_axioms` and
  cannot win under `require_kernel_certificates`.
- Witnesses become publication-reachable only after the artifact manifest
  references them; staging alone publishes nothing.
