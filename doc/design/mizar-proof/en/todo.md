# mizar-proof TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs do not exist yet; each is written by its own spec task (English
and Japanese in the same change) before the implementation tasks that cite it.
Module names follow the minimum split of
[internal 07](../../internal/en/07.crate_module_layout.md) (`policy`,
`witness_store`) plus the selection/status projections of internal 04; the
crate refines architecture 08, 15, and 19 and internal 04.

| Module | Spec | Source | Status |
|---|---|---|---|
| policy | `policy.md` (task 2) | `src/policy.rs` | [ ] |
| selection | `selection.md` (task 5) | `src/selection.rs` | [ ] |
| status | `status.md` (task 8) | `src/status.rs` | [ ] |
| witness_store | `witness_store.md` (task 10) | `src/witness_store.rs` | [ ] |

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
- **Discharge-evidence validation scope: open, resolved by task 6.**
  Whether `mizar-vc` pre-ATP discharge evidence is kernel-replayed or
  accepted as deterministic built-in evidence per policy. Decided here with
  `mizar-kernel` (registered at the top level and in both crates' todos).
- **Policy fingerprint surface: open, resolved by task 3.** Which policy
  settings enter the `PolicyFingerprint` used by cache keys and proof
  reuse; coordinate with `mizar-cache` task 2.

## Ordered Task List

Keep `cargo test -p mizar-proof` green after each task (see
[Recommended Verification](#recommended-verification)).

### Policy

1. **Crate scaffold and lint-policy guard.** [ ]
   - Add the `mizar-proof` workspace member depending on `mizar-session`,
     `mizar-kernel`, `mizar-vc`, `mizar-atp`, and `mizar-artifact`; add
     `tests/lint_policy.rs` mirroring the `mizar-frontend` guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: `mizar-kernel` task 1, `mizar-atp` task 1. Spec: internal 04.

2. **Spec: `policy.md`.** [ ]
   - Write the policy spec (English and Japanese, no code): verifier policy
     settings, `CandidatePolicyClass`, externally-attested admission rules,
     `require_kernel_certificates`, open-obligation allowances per build
     mode, and the rule that policy outcomes are distinct from trusted
     status.
   - Deps: 1. Spec: [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
     "Proof Policy Evaluator", architecture 08.

3. **Policy evaluator.** [ ]
   - Implement `ProofPolicyEvaluator`: candidate classification,
     `can_schedule_kernel_check`, and the policy-fingerprint projection
     (resolving that decision with `mizar-cache`).
   - Tests: classification fixtures per evidence kind; fingerprint changes
     iff a policy-relevant setting changes.
   - Deps: 2. Spec: `policy.md`.

4. **Externally attested evidence handling.** [ ]
   - Implement admission and labeling of externally attested evidence:
     recordable as development evidence when the profile allows, never
     winning under `require_kernel_certificates`, never producing trusted
     `used_axioms`.
   - Tests: admission matrices per profile; rejection diagnostics stable.
   - Deps: 3. Spec: `policy.md` (externally attested evidence section).

### Selection and status

5. **Spec: `selection.md`.** [ ]
   - Write the winner-selection spec (English and Japanese, no code): the
     deterministic ordering classes (kernel-verified satisfying release
     policy → policy-permitted external → best-explained open), tie-break
     keys (backend profile priority, certificate format priority, encoded
     problem hash, profile id), the selected proof witness hash or
     deterministic discharge hash exported for reuse validation, and the
     completion-time prohibition.
   - Deps: 2. Spec: [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
     "Winner Selection".

6. **Winner selection.** [ ]
   - Implement deterministic winner selection over
     `ProofEvidenceSet`s; resolve and record the discharge-evidence
     validation-scope decision (with `mizar-kernel`), since it determines
     which class built-in discharge evidence enters.
   - Tests: ordering fixtures across classes and tie-breaks; shuffled
     candidate arrival never changes the winner.
   - Deps: 3, 5, `mizar-kernel` task 16. Spec: `selection.md`.

7. **Artifact proof selection merge.** [ ]
   - Merge portfolio results with phase-12 built-in discharge results per
     `VcId` into `kernel_verified` / `discharged_builtin` selections, with
     external and open statuses kept distinguishable.
   - Tests: merge fixtures per combination; no status collapses into
     another.
   - Deps: 6, `mizar-vc` task 12. Spec: `selection.md` (merge section),
     [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
     "Artifact Proof Selection".

8. **Spec: `status.md`.** [ ]
   - Write the status-projection spec (English and Japanese, no code): the
     artifact- and diagnostics-facing proof status model, trusted
     `used_axioms` propagation (only from kernel-accepted evidence), and
     explanation references for open/rejected obligations.
   - Deps: 5. Spec: [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
     "Proof Witness and Artifact Flow", architecture 19.

9. **Proof status projection.** [ ]
   - Implement status projection for artifacts and diagnostics, including
     trusted `used_axioms` extraction boundaries.
   - Tests: projection fixtures per selection outcome; `used_axioms` only
     from kernel-accepted evidence.
   - Deps: 7, 8. Spec: `status.md`.

### Witness store

10. **Spec: `witness_store.md`.** [ ]
    - Write the witness-store spec (English and Japanese, no code): the
      stage/publish flow (`stage` before commit, `publish_ref` only after
      the artifact manifest references the witness), stable content hashing
      used as proof witness hashes, and provenance metadata.
    - Deps: 2. Spec: [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
      "Proof Witness Store".

11. **Witness store implementation.** [ ]
    - Implement staging and publication of `ProofWitnessDraft`s against the
      `mizar-artifact` witness-reference schema.
    - Tests: stage/publish round-trips; publication before manifest
      reference fails; hashes recorded before commit.
    - Deps: 9, 10, `mizar-artifact` task 9. Spec: `witness_store.md`.

12. **Portfolio early-stop policy hooks.** [ ]
    - Provide the policy queries the ATP portfolio uses for early stop
      (no-better-class-possible checks), keeping termination decisions
      policy-driven, not time-driven.
    - Tests: early-stop fixtures per policy; stopping never changes the
      selected winner versus running to completion.
    - Deps: 6, `mizar-atp` task 18. Spec: `policy.md`,
      [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md)
      "Early Stop and Cancellation".

### Hardening and cross-cutting follow-ups

13. **Determinism suite.** [ ]
    - Property coverage that identical evidence sets produce identical
      classifications, winners, statuses, and witness references under
      shuffled arrival orders.
    - Deps: 11, 12. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).

14. **Public-enum forward-compatibility policy.** [ ]
    - Apply the `mizar-frontend` task-25 procedure to each public enum;
      status enums additionally follow the artifact compatibility policy.
    - Deps: 11. Spec: all module specs.

15. **Source/spec correspondence audit.** [ ]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; verify the policy/trust split is restated
      in every module spec.
    - Deps: 14. Spec: all module specs and this TODO.

16. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-proof/en/` with its Japanese companion and
      synchronize content.
    - Deps: 15. Spec: repository documentation policy.

17. **Proof-reuse metadata export contract.** [ ]
    - Expose the proof-reuse metadata consumed by `mizar-cache`: compatible
      verifier-policy fingerprint, selected proof witness hash or deterministic
      discharge hash, evidence class, selected-candidate provenance, and the
      selection reason. This metadata is a validation predicate for reuse, not
      trusted proof status.
    - Tests: changing any exported reuse component changes or invalidates the
      reuse predicate; shuffled candidate arrival preserves the same exported
      metadata; externally attested evidence remains externally attested and is
      never upgraded by metadata reuse.
    - Deps: 6, 7, 9, 11, 13. Spec:
      [22.incremental_verification_contract.md](../../architecture/en/22.incremental_verification_contract.md),
      [internal 04](../../internal/en/04.atp_portfolio_and_kernel_check_integration.md),
      [11.artifact_and_incremental_build.md](../../architecture/en/11.artifact_and_incremental_build.md).

18. **Architecture-22 follow-up audit.** [ ]
    - Re-run the source/spec correspondence and bilingual documentation sync
      audits for the task-17 reuse-metadata export contract; record any
      remaining trust-boundary, witness-hash, deterministic discharge, or
      policy-selection gaps as follow-up tasks.
    - Deps: 17. Spec: all module specs, this TODO, and repository
      documentation policy.

19. **Module-boundary refactor gate.** [ ]
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

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-proof
cargo clippy -p mizar-proof --all-targets -- -D warnings
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
