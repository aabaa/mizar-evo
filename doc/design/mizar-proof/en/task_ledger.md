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
| 3. Policy evaluator | complete | `f3ac274d9cf7266f5ebdc4de94fe963f72ba67d5` | Initial reviews found synthesized accepted-kernel input risk, incomplete evidence-kind test coverage, negative scheduling test gaps, kernel/policy rejection layering drift, and task-4 external-admission overreach. After fixes, focused spec, test-sufficiency, full, and source-doc reviews reported no findings. | `cargo test -p mizar-proof` passed; `cargo clippy -p mizar-proof --all-targets -- -D warnings` passed; `cargo fmt --check` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit task-3 path staging. | Implements the policy evaluator, normalized kernel-origin wrapper, candidate classes, schedulability checks, kernel rejection separation, and deterministic policy fingerprint; updates the lint guard for the policy module. |
| 4. Externally attested evidence handling | complete | `986f62a184fa8d0a7fa0c98ef8f6c9669d85d844` | Initial reviews found missing concrete publication-status mapping, unclear `PolicyDecision.class` and diagnostic ownership, public `trusted_used_axioms_allowed` invariant risk, module-status drift, matrix coverage gaps, and policy-tainted origin coverage gaps. After fixes, focused spec, test-sufficiency, full, and source-doc reviews reported no findings. | `cargo test -p mizar-proof` passed; `cargo clippy -p mizar-proof --all-targets -- -D warnings` passed; `cargo fmt --check` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit task-4 path staging. | Implements external evidence admission labels, stable policy diagnostics, non-trusted admission matrix, and policy-tainted kernel-result routing without trusted `used_axioms`. |
| 5. Spec: `selection.md` | complete | `6802bbec9f75d33f6c28b06391d60101726b2d51` | Initial reviews found underspecified rejected/all-diagnostic outcomes, non-total tie-break inputs, `DischargedBuiltin` trust wording, missing policy-assumption handling, TODO ordering drift, and external recordable/selectable wording drift. After fixes, focused spec, test-sufficiency, full, and source-doc reviews reported no findings. | `git diff --check` passed; `git diff --cached --check` passed after explicit task-5 path staging. | Adds paired winner-selection specs and clarifies policy wording for deterministic winner classes, total tie-break identity, rejected/no-selectable diagnostics, reuse metadata, trusted/non-trusted boundaries, completion-time prohibition, and deferred downstream integrations. |
| 6. Winner selection | complete | pending self-hash | Initial reviews found trusted-marker spoofing, non-trusted winners with pending kernel-checkable inputs, duplicate-id diagnostics not surfaced, rejected-category ordering gaps, missing witness publication gap, underspecified tests, and accepted-evidence hash identity weakness. After fixes, focused spec, test-sufficiency, full, and source-doc reviews reported no findings. | `cargo test -p mizar-proof` passed; `cargo clippy -p mizar-proof --all-targets -- -D warnings` passed; `cargo fmt --check` passed; `git diff --check` passed; `git diff --cached --check` passed after explicit task-6 path staging. | Implements deterministic winner selection, trusted-kernel evidence markers bound to kernel-derived evidence hashes, total tie-break/rejection ordering, pending-kernel gating, duplicate-id diagnostics, no-selectable diagnostics, reuse metadata, `discharged_builtin` witness-publication gap marking, and selection module lint guard updates. |

## Current Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-proof autonomous crate development with task 7 after the task-6
commit exists. First verify a clean worktree and confirm the task-6 commit in
HEAD history. Then implement artifact proof selection merge per
`doc/design/mizar-proof/en/selection.md` and internal 04: merge portfolio
selection results with phase-12 built-in discharge results per `VcId`, keep
`kernel_verified` and `discharged_builtin` distinct, and preserve external,
assumed, open, rejected, and no-selectable statuses as non-trusted outcomes.
Do not implement status projection, witness staging, cache lookup, artifact
commit, ATP execution, SAT solving, proof search, premise selection,
substitution invention, or placeholder downstream integration.
```

Rationale: task 7 crosses the portfolio/built-in-discharge merge boundary and
must preserve trusted/non-trusted status distinctions for later status
projection. Keep `xhigh`; lower reasoning is appropriate only for typo-only
cleanup, and raise only if downstream artifact or VC APIs expose a new
external dependency gap.
