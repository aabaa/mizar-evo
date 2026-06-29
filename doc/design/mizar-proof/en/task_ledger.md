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
| 2. Spec: `policy.md` | complete | pending self-hash | Initial reviews found built-in discharge scheduling wording, `AssumedByPolicy`, artifact witness gap, fingerprint TODO, and ledger-finalization issues. After fixes, focused spec, test-sufficiency, full, and source-doc reviews reported no findings. | `git diff --check` passed; `git diff --cached --check` passed after explicit task-2 path staging. | Adds paired policy specs covering verifier policy settings, candidate classes, external evidence, open obligations, policy assumptions, fingerprint surface, scheduling, early stop, diagnostics, and trust boundary. |

## Current Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-proof autonomous crate development with task 3 after the task-2
commit exists. First verify a clean worktree and confirm the task-2 commit in
HEAD history. Then implement `ProofPolicyEvaluator` in `src/policy.rs` from
the paired `policy.md` specs: verifier policy settings, candidate
classification, `can_schedule_kernel_check`, and the `PolicyFingerprint`
projection. Add focused classification and fingerprint tests. Do not implement
winner selection, artifact status projection, witness staging, cache lookup,
artifact commit, ATP execution, SAT solving, proof search, premise selection,
substitution invention, externally attested winner behavior beyond the
task-3 base classifier, or placeholder downstream integration.
```

Rationale: task 3 is the first Rust behavior task in the proof-policy crate and
defines the reusable policy data model. Keep `xhigh` because mistakes could
blur trusted kernel acceptance with policy evidence or cache metadata. Lower
reasoning is appropriate only for comment-only cleanup; raise only if existing
upstream APIs cannot represent the spec without an additional gap record.
