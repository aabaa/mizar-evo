# mizar-proof Task Ledger

> Canonical language: English. Japanese companion:
> [../ja/task_ledger.md](../ja/task_ledger.md).

This ledger records one commit per autonomous crate-development task. A task
commit cannot contain its own hash, so self-hashes are backfilled only in the
next numbered task that edits the ledger or in the final closeout task. No
extra unnumbered bookkeeping commit is permitted.

| Task | Status | Commit | Review result | Verification | Summary |
|---|---|---|---|---|---|
| 0. Crate plan | complete | `a1f6d1dba4ecc46aa2d434c8da8ae0279a2c23ec` | Initial reviews found ledger/TODO status drift, broad ATP gap classification, and missing `discharged_builtin` artifact-witness gap; docs were synchronized. Focused re-reviews reported no findings. Test sufficiency review reported no findings for docs-only verification. | `git diff --check` passed; `git diff --cached --check` passed after explicit task-0 path staging. | Adds paired crate plan and task ledger; classifies kickoff gaps; does not scaffold source. |
| 1. Crate scaffold and lint-policy guard | complete | `bc7d33efd8b7427fa026807e729642219f62f809` | Initial reviews found only stale ledger/handoff finalization wording; scaffold, dependency boundary, lockfile, crate root, and lint guard had no findings. Focused re-reviews reported no scaffold or handoff findings after updates. Test sufficiency review reported no findings. | `cargo test -p mizar-proof` passed; `cargo clippy -p mizar-proof --all-targets -- -D warnings` passed; `cargo fmt --check` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit task-1 path staging. | Adds the workspace crate, scaffold-only crate root, dependency manifest, and lint-policy guard. |
| 2. Spec: `policy.md` | complete | `193598cd5db448b17c5cddb721a5e22010b241b6` | Initial reviews found built-in discharge scheduling wording, `AssumedByPolicy`, artifact witness gap, fingerprint TODO, and ledger-finalization issues. After fixes, focused spec, test-sufficiency, full, and source-doc reviews reported no findings. | `git diff --check` passed; `git diff --cached --check` passed after explicit task-2 path staging. | Adds paired policy specs covering verifier policy settings, candidate classes, external evidence, open obligations, policy assumptions, fingerprint surface, scheduling, early stop, diagnostics, and trust boundary. |
| 3. Policy evaluator | complete | pending self-hash | Initial reviews found synthesized accepted-kernel input risk, incomplete evidence-kind test coverage, negative scheduling test gaps, kernel/policy rejection layering drift, and task-4 external-admission overreach. After fixes, focused spec, test-sufficiency, full, and source-doc reviews reported no findings. | `cargo test -p mizar-proof` passed; `cargo clippy -p mizar-proof --all-targets -- -D warnings` passed; `cargo fmt --check` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit task-3 path staging. | Implements the policy evaluator, normalized kernel-origin wrapper, candidate classes, schedulability checks, kernel rejection separation, and deterministic policy fingerprint; updates the lint guard for the policy module. |

## Current Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-proof autonomous crate development with task 4 after the task-3
commit exists. First verify a clean worktree and confirm the task-3 commit in
HEAD history. Then implement externally attested evidence handling on top of
the task-3 base classifier: admission and labeling by policy profile,
recordable development evidence, never winning under
`require_kernel_certificates`, never producing trusted `used_axioms`, and
stable policy rejection diagnostics. Do not implement winner selection,
artifact status projection, witness staging, cache lookup, artifact commit, ATP
execution, SAT solving, proof search, premise selection, substitution
invention, or placeholder downstream integration.
```

Rationale: task 4 completes the external-evidence admission matrix while
preserving the trust boundary. Keep `xhigh` because mistakes could let external
or policy-tainted evidence satisfy trusted acceptance. Lower reasoning is
appropriate only for wording-only cleanup; raise only if upstream APIs require
a new explicit gap classification.
