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
| 1. Crate scaffold and lint-policy guard | complete | pending self-hash | Initial reviews found only stale ledger/handoff finalization wording; scaffold, dependency boundary, lockfile, crate root, and lint guard had no findings. Focused re-reviews reported no scaffold or handoff findings after updates. Test sufficiency review reported no findings. | `cargo test -p mizar-proof` passed; `cargo clippy -p mizar-proof --all-targets -- -D warnings` passed; `cargo fmt --check` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit task-1 path staging. | Adds the workspace crate, scaffold-only crate root, dependency manifest, and lint-policy guard. |

## Current Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-proof autonomous crate development with task 2 after the task-1
commit exists. First verify a clean worktree and confirm the task-1 commit in
HEAD history. Then write the paired English/Japanese `policy.md` module spec
only: verifier policy settings, `CandidatePolicyClass`, externally attested
admission rules, `require_kernel_certificates`, build-mode open-obligation
allowances, policy fingerprint surface, kernel-check scheduling policy, and the
rule that policy outcomes are distinct from trusted proof status. Do not add
Rust policy evaluator behavior, winner selection, status projection, witness
staging, cache lookup, artifact commit, ATP execution, SAT solving, proof
search, premise selection, substitution invention, or placeholder downstream
integration in task 2.
```

Rationale: task 2 is spec-only, but it fixes the trust and policy vocabulary
used by every implementation task after it. Keep `xhigh` because the policy
surface affects proof reuse, external evidence, open obligations, and kernel
certificate requirements. Lower reasoning is appropriate only for typo-only
documentation sync; raise only if architecture/internal specs contradict each
other.
