# mizar-proof Task Ledger

> Canonical language: English. Japanese companion:
> [../ja/task_ledger.md](../ja/task_ledger.md).

This ledger records one commit per autonomous crate-development task. A task
commit cannot contain its own hash, so self-hashes are backfilled only in the
next numbered task that edits the ledger or in the final closeout task. No
extra unnumbered bookkeeping commit is permitted.

| Task | Status | Commit | Review result | Verification | Summary |
|---|---|---|---|---|---|
| 0. Crate plan | complete | pending self-hash | Initial reviews found ledger/TODO status drift, broad ATP gap classification, and missing `discharged_builtin` artifact-witness gap; docs were synchronized. Focused re-reviews reported no findings. Test sufficiency review reported no findings for docs-only verification. | `git diff --check` passed; `git diff --cached --check` passed after explicit task-0 path staging. | Adds paired crate plan and task ledger; classifies kickoff gaps; does not scaffold source. |

## Current Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-proof autonomous crate development with task 1 after the task-0
commit exists. First verify a clean worktree and confirm the task-0 commit in
HEAD history. Then scaffold `crates/mizar-proof` as a workspace member with
dependencies on `mizar-session`, `mizar-kernel`, `mizar-vc`, `mizar-atp`, and
`mizar-artifact`; add a lint-policy guard mirroring the repository pattern.
Do not implement policy evaluator behavior, winner selection, status
projection, witness staging, cache lookup, artifact commit, ATP execution, SAT
solving, proof search, premise selection, substitution invention, or any
placeholder downstream integration in task 1.
```

Rationale: task 1 is a narrow scaffold/lint step, but it sits on the proof
trust boundary and should preserve the one-task-one-commit discipline. Lower
reasoning is appropriate only for typo-only documentation follow-up; raise only
if repository metadata or workspace dependency policy conflicts block the
scaffold.
