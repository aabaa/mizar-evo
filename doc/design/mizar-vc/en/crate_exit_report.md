# Crate Exit Report: mizar-vc

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete.
Quality score: 94/100.
Score caps applied: none.

## Scope

Milestone scope:

- Build the `mizar-vc` crate from preliminary task 0 through task 22 and the
  closeout task.
- Own pipeline phases 11-12: VC generation, deterministic `VcId` assignment,
  status-policy projection, deterministic pre-ATP discharge, replayable
  in-memory discharge evidence, conservative dependency slices, and reusable
  proof-reuse candidate keys.
- Consume explicit `mizar-core` `CoreIr`, `ControlFlowIr`, and
  `ObligationSeedHandoff` payloads without reconstructing missing source,
  checker, or registration data.
- Keep proof acceptance, ATP execution, cache lookup, artifact publication,
  kernel validation, source-derived corpus extraction, and unavailable upstream
  payload families classified as external dependency gaps or deferred work.

Included:

- English/Japanese crate plan, module specifications, audits, and closeout
  report under `doc/design/mizar-vc/{en,ja}/`.
- Rust source under `crates/mizar-vc/src/`.
- Crate-local unit tests and integration tests under `crates/mizar-vc/`.
- Deferred proof-verification corpus traceability rows in
  `tests/coverage/spec_trace.toml`.

Excluded:

- Direct edits to `doc/spec`.
- Rebaselining existing `.miz` fixtures or expectation sidecars.
- Active source-derived `proof_verification` fixtures or snapshots.
- ATP problem encoding, backend process execution, certificate validation,
  trusted proof acceptance, cache hit acceptance, or artifact publication.
- Placeholder crates or placeholder consumers for unavailable `mizar-atp`,
  `mizar-kernel`, `mizar-proof`, or `mizar-cache` seams.
- Fabricated registration/redefinition/reduction, branch, match, loop,
  termination, ghost-erasure, trace, source-derived core formula, definition,
  quantified binder, or source-derived obligation payloads.

## Task Commits

| Task | Commit | Subject |
|---|---|---|
| 0 | `9697036b0f012cfc578a015dc5a0d6f37bf85143` | `docs(vc-task-0): add autonomous crate plan` |
| 1 | `adfff1cbc3ebce9db13e73d4d29bfd9b1ac1971d` | `feat(vc-task-1): scaffold mizar-vc crate` |
| 2 | `ac778b008be75ea21eda4d2e69c7713a88b0d4ea` | `docs(vc-task-2): specify vc ir data shapes` |
| 3 | `c32d767368ef9d16fdcf92620c2b2afecb13fc9d` | `feat(vc-task-3): add vc ir data shapes` |
| 4 | `ba20db550cf92979bdb8809e9f64fbe5cd193c1b` | `feat(vc-task-4): add seed intake table` |
| 5 | `e324beab799f972dcf78e897b163aebd9414725e` | `docs(vc-task-5): specify generator contract` |
| 6 | `b5634eb878b39558b981bcbba972e8b36c3203c9` | `feat(vc-task-6): add core generation candidates` |
| 7 | `a15a2ee3e21974727fab2f8406b2e161b3f3c2f7` | `feat(vc-task-7): add algorithm generation candidates` |
| 8 | `6b4a7ef661886d6339f8ac24e21ad68e9f7ac302` | `feat(vc-task-8): normalize vc generation candidates` |
| 9 | `30c8e303c2c88d70a0dd69295ec001280471519a` | `feat(vc-task-9): add vc status policy projection` |
| 10 | `18c86f9b03318c28e39311162ae3e89adc0e2d2a` | `docs(vc-task-10): specify discharge contract` |
| 11 | `d4643a7f1078ec330640e63021942bc245d9a609` | `feat(vc-task-11): add deterministic discharge engine` |
| 12 | `57c4e247ca13cdcf05e92d9854e41f60fa5e0f49` | `feat(vc-task-12): add discharge evidence records` |
| 13 | `6238217eedc55e76ec277ab14bd1d78a3b57c6a6` | `docs(vc-task-13): specify dependency slices` |
| 14 | `26e5fea26769e1bf7ccb47e99d814709f035801f` | `feat(vc-task-14): compute dependency slices` |
| 15 | `beee07a8009245e2bc0096d98df3968ea1212ac3` | `docs(vc-task-15): record proof verification corpus gaps` |
| 16 | `8b183e538fa4007e82b0c2b2af058ebe566fca22` | `test(vc-task-16): add determinism suite` |
| 17 | `f65ff56d9a3a555586cf21189780aaaa1017359d` | `test(vc-task-17): guard public enum policy` |
| 18 | `373e943b43e2c17b5a1cad282160e71c4c51de89` | `docs(vc-task-18): add source spec audit` |
| 19 | `f36852c74d5f1d0724514f7ecda0b1a539ab6561` | `docs(vc-task-19): audit bilingual docs sync` |
| 20 | `2f3eb323be8080bf231e1b69dfc9e9e729bb45f9` | `feat(vc-task-20): wire cross-edit reuse identity` |
| 21 | `a8243c3498249fe75d3619fbbe4f5a2dc94b86a2` | `docs(vc-task-21): add architecture 22 follow-up audit` |
| 22 | `76f286f9a3d1e6d6f096b84be7b5f38873e48d42` | `docs(vc-task-22): add module boundary audit` |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | Module specs, source/spec audit, architecture-22 audit, module-boundary audit, and closeout reviews record no unresolved blocking/high specification inconsistency. |
| Source behavior documented or deferred | passed | Public modules and promised behavior are traced in `source_spec_audit.md`; unsupported source-derived and downstream behavior is classified rather than implemented silently. |
| Milestone-owned coverage | passed | Crate-local Rust tests cover VC IR validation/rendering, seed intake, generation, normalization, status policy, discharge, evidence, dependency slices, determinism, public enum policy, and reuse gating. |
| Test expectation integrity | passed | No existing `.miz` fixture or expectation sidecar was changed to match implementation behavior. Task 15 records corpus gaps instead of fake active proof-verification fixtures. |
| Design/source synchronization | passed | Paired source/spec, bilingual, architecture-22, and module-boundary audits are synchronized with the source layout and public module table. |
| Boundary discipline | passed | `mizar-vc` produces untrusted prover-independent obligations, evidence, slices, and reuse candidates only; ATP, kernel, proof, cache, artifact, and source-extraction ownership stays downstream or external. |
| Verification | passed | Closeout broad commands and diff checks passed before commit. |
| Residual risk | passed with classified items | Remaining risks are listed below as `external_dependency_gap` or `deferred`. |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 18/20 |
| Traceability | 15/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 3/5 |
| Total | 94/100 |

The score deducts for unavailable source-derived proof-verification coverage,
missing downstream ATP/kernel/proof/cache consumers, incomplete upstream stable
payload families, and large implementation files that remain a maintenance
watchlist. These are classified and do not cap the score because the crate-local
milestone does not own those seams and no hard gate fails.

## Review Results

| Review | Result |
|---|---|
| Implementation specification / documentation review | No findings after the closeout report, audit inventory, Task 22 hash backfill, and closeout ledger wording were synchronized. |
| Test sufficiency review | No findings. Broad workspace verification plus diff checks are sufficient for the docs-only closeout task; no new Rust, `.miz`, expectation, `doc/spec`, or traceability changes are required. |
| Full implementation review | No findings. The closeout commit is docs-only, leaves source unchanged, and records all task commits, hard gates, residual gaps, and verification. |
| Source/documentation consistency review | No findings. English canonical docs and Japanese companions are paired; source/public module boundaries match the audits; Task 22 is backfilled. |
| Read-only crate quality review | Hard gates pass with no blocking/high/medium findings. Valid quality score: 94/100, which is >= 90. |

## Deferred Items

| ID | Class | Reason | Owner / unblock condition |
|---|---|---|---|
| VC-CLOSEOUT-G001 | `external_dependency_gap` | `mizar-test` still lacks an active `proof_verification` runner/tag gate and source-to-core/source-to-VC extraction seams for real `.miz` corpus inputs. | Add runner and extraction support in the owning staged-test and upstream extraction tasks before activating source-derived VC fixtures. |
| VC-CLOSEOUT-G002 | `external_dependency_gap` | `mizar-atp`, `mizar-kernel`, `mizar-proof`, and `mizar-cache` are not workspace crates or active consumers, so ATP translation, certificate acceptance, proof policy, cache hit acceptance, and proof-reuse validation remain downstream. | Create or wire the owning downstream crates with their own crate plans and consumer contracts. |
| VC-CLOSEOUT-G003 | `external_dependency_gap` | Upstream explicit/stable payloads remain incomplete for registration/redefinition/reduction details, call preconditions, branch/match/range/collection loop obligations, term-only and partial termination, Pick non-emptiness, ghost erasure, complete trace families, source-derived core formula payloads, definition payloads, quantified binder payloads, and source-derived obligation payload families. | Upstream checker/core/control-flow tasks expose stable explicit payloads; `mizar-vc` can then add spec-backed generation/discharge/slice tasks. |
| VC-CLOSEOUT-G004 | `deferred` | Proof-witness hashes, ATP/kernel/proof/cache validation, artifact consumers, and source-derived runner integration must exist before architecture-22 reuse is accepted outside deterministic-discharge candidate keys. | Downstream proof/cache/artifact phases validate the untrusted reusable inputs produced here. |
| VC-CLOSEOUT-G005 | `deferred` | Large `vc_ir`, `generator`, and `dependency_slice` files may benefit from private helper/test splits, but Task 22 found no required move-only split before crate exit. | Run future move-only maintenance tasks only if reviewability becomes a bottleneck; do not mix behavior or API changes. |

No `repo_metadata_conflict` was observed.

## Human Review Surface

- Canonical English docs under `doc/design/mizar-vc/en/`.
- Japanese companions under `doc/design/mizar-vc/ja/`.
- VC source under `crates/mizar-vc/src/`.
- VC tests under `crates/mizar-vc/tests/` and module-local Rust tests.
- Deferred proof-verification traceability rows in
  `tests/coverage/spec_trace.toml`.
- Upstream inputs:
  `doc/design/mizar-core/en/crate_exit_report.md`,
  `doc/design/mizar-core/en/core_ir.md`, and
  `doc/design/mizar-core/en/control_flow.md`.

## Test Expectation Summary

No existing `.miz` fixtures or expectation sidecars were changed to match
implementation behavior. Milestone-owned behavior is covered by Rust unit tests,
integration tests, lint-policy guards, determinism tests, source/spec audits, or
explicit deferred traceability rows. Source-derived semantic corpus coverage
remains blocked by the external runner and extraction gaps listed above.

## Verification Commands

| Command | Result |
|---|---|
| `cargo fmt --check` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test` | passed |
| `git diff --check` | passed |
| `git diff --cached --check` | passed after explicit closeout path staging |

Unrun deferred commands:

- `cargo test -p mizar-cache` and `cargo test -p mizar-proof` were not run as
  dedicated consumer checks because those crates do not exist in the workspace.
  The broad `cargo test` command covers the current workspace.
- Dedicated `mizar-atp` and `mizar-kernel` checks were not run for the same
  reason: those crates are external gaps, not current workspace members.

## Next-Task Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Start the next verification pipeline crate after the completed mizar-vc closeout.
Before editing, verify a clean worktree, confirm the mizar-vc closeout commit in
git log, and read doc/design/mizar-vc/en/crate_exit_report.md,
doc/design/mizar-vc/en/00.crate_plan.md, doc/design/mizar-atp/en/todo.md,
doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md,
doc/design/internal/en/07.crate_module_layout.md,
doc/design/architecture/en/09.atp_interface_protocol.md, and
doc/design/architecture/en/10.atp_backend_integration.md. Begin with preliminary
task 0 for mizar-atp: create or update the paired English/Japanese crate plan,
classify specification gaps, test gaps, source/design drift, external
dependencies, and deferred items, and commit that plan as its own task. Preserve
the one-task-one-commit rule; do not scaffold mizar-atp source until the task-0
plan commit exists.
```

Rationale: `mizar-atp` is the next phase after `mizar-vc` in the pipeline and
owns phase-13 ATP translation/backend execution. Keep `xhigh` because the work
crosses proof evidence, external process, certificate, and downstream policy
boundaries. Lower reasoning is appropriate only for typo-only documentation
sync; raise only if repository metadata or specification contradictions block
the crate plan.
