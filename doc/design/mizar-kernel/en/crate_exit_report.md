# Crate Exit Report: mizar-kernel

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete.
Quality score: 95/100.
Score caps applied: none.

Post-closeout correction: commit `c6d94fe51923aa0363ea7297bfe4e9f905aef076`
supersedes the task-22 evidence target. This report remains the closeout record
for the legacy resolution-trace milestone. Tasks 23-29 reopen the crate for the
formula/substitution evidence pipeline, trusted in-process SAT checking, and
legacy path migration audit.

## Scope

Milestone scope:

- Build the `mizar-kernel` crate from preliminary task 0 through task 22 and
  the closeout task.
- Own the task-22 legacy phase-14 surface: normalized certificate parsing,
  canonical clause validation, MiniSAT-compatible resolution trace replay,
  substitution/alpha/free-variable replay, imported fact checking, explicit
  cluster/reduction trace replay, and deterministic check-service
  orchestration.
- Consume immutable normalized certificates and explicit kernel contexts as
  evidence only; parsing or backend success never grants trust by itself.
- Keep SAT solving outside the task-22 legacy milestone, and keep ATP backend execution, proof search, premise selection,
  overload resolution, cluster search, implicit coercion insertion, fallback
  inference, global mutable compiler state, proof-policy projection, cache
  lookup, artifact publication, and unfinished producer/consumer integration
  outside the crate.

Post-correction scope:

- SAT checking over a kernel-derived SAT problem is now allowed and trusted only
  through the task-24 selected direct
  `batsat = { version = "=0.6.0", default-features = false }` dependency,
  after task 27 integrates the wrapper and tasks 25-28 derive the problem from
  validated formula/substitution evidence.
- Backend proof methods, resolution traces, SMT proof objects, and backend logs
  remain outside trusted acceptance material.
- Legacy certificate/resolution-trace acceptance is `source_drift` until task 29
  gates or retires it for normal proof policy.

Included:

- English/Japanese crate plan, module specifications, audits, and closeout
  report under `doc/design/mizar-kernel/{en,ja}/`.
- Rust source under `crates/mizar-kernel/src/`.
- Crate-local unit tests and lint-policy tests under `crates/mizar-kernel/`.
- Source/spec, bilingual, public enum, determinism, soundness, and
  module-boundary audits that classify remaining gaps instead of mocking them.

Excluded:

- Direct edits to `doc/spec` for the task-22 legacy milestone.
- Rebaselining existing `.miz` fixtures or expectation sidecars.
- Source-derived formula/substitution evidence corpus fixtures or a
  source-to-kernel-evidence runner.
- SAT solver or ATP backend implementation for the task-22 legacy milestone.
- Proof-policy projection, proof witness publication, cache hit acceptance, or
  artifact validation.
- Placeholder integration with unfinished `mizar-atp`, `mizar-proof`,
  `mizar-cache`, or `mizar-artifact` seams.
- Hidden use of resolver/checker mutable state, implicit coercion insertion,
  overload resolution, cluster search, fallback inference, or repair
  heuristics.

## Task Commits

| Task | Commit | Subject |
|---|---|---|
| 0 | `81ffb5561fc1b24ae355d216e1a455d2a487d923` | `docs(kernel-task-0): add autonomous crate plan` |
| 1 | `63cbcd83a82005d8ffe98f7c87928fa46e95649c` | `feat(kernel-task-1): scaffold mizar-kernel crate` |
| 2 | `b0fa89a9eecc85da96bf8351fc2e147423747730` | `docs(kernel-task-2): specify clause representation` |
| 3 | `4020ac12fafe24aa8205f7fd3df8ece37027804e` | `feat(kernel-task-3): add clause representation` |
| 4 | `b900639e4057ea2ba1a1158688a35e188ec991ec` | `docs(kernel-task-4): specify certificate parser` |
| 5 | `60c92cc53c77ec3240fe5410fc04c449bd04b267` | `feat(kernel-task-5): add certificate parser` |
| 6 | `f4b1abc63a46cd7d628911aff4a7ce91c0c5555b` | `docs(kernel-task-6): specify rejection records` |
| 7 | `acc8e7d62adbee21cb49b8d134fe0d846ee60603` | `feat(kernel-task-7): add rejection records` |
| 8 | `0b017553b3462eb78492d3aa84053b9d07a2fae4` | `docs(kernel-task-8): specify resolution trace replay` |
| 9 | `28b7e7122c8cad04a6526d8de8cdfd0394d8bb3c` | `feat(kernel-task-9): add resolution trace replay` |
| 10 | `d79506c6e0b7029fb1512454b0eff72579362df7` | `docs(kernel-task-10): specify substitution checker` |
| 11 | `b97c4a3a700fec986d3e203b1a88d23edcfba7f3` | `feat(kernel-task-11): add substitution checker` |
| 12 | `577f6f220b93d94c9796208829216f43a8e2e3d4` | `feat(kernel-task-12): add alpha and free-variable checks` |
| 13 | `865231081df7538faea132c499d9c57d5ecfa9cb` | `docs(kernel-task-13): specify checker orchestration` |
| 14 | `874881b42d5c008336a34cb4cfaf24f7b403a1fb` | `feat(kernel-task-14): add imported fact checking` |
| 15 | `77262c0ec36071bdab8ac5c1b22d14a4537ae68a` | `feat(kernel-task-15): add cluster trace replay` |
| 16 | `c0b8e6104f38d02e7bf8f6c1cda5900fb50bdfc1` | `feat(kernel-task-16): add kernel check service` |
| 17 | `b7e1493050ed49110e4ddf7a7a75d971bdf72c59` | `test(kernel-task-17): add soundness fail corpus` |
| 18 | `3d1942e97ea245d2fae09dac4e26cefd67c02bd1` | `test(kernel-task-18): add determinism replay-cost suite` |
| 19 | `981fa7a05fe8de11168bd862d81cbd7d486347c0` | `test(kernel-task-19): guard public enum policy` |
| 20 | `fb81213c33d5b2a31eb976a4fa6804bfc0ffe6c5` | `docs(kernel-task-20): audit source spec correspondence` |
| 21 | `73a919c16b48da82038fd7267e86e1a844cb4c6f` | `docs(kernel-task-21): audit bilingual docs sync` |
| 22 | `814e47bb9aaaff75ebfe4cc1be10d2eb4618498b` | `refactor(kernel-task-22): split module test boundaries` |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | Module specs, source/spec audit, bilingual sync audit, module-boundary audit, and closeout reviews record no unresolved blocking/high specification inconsistency. |
| Source behavior documented or deferred | passed | Public modules, public items, tests, and promised behavior are traced in `source_spec_audit.md`; unsupported source-derived and downstream behavior is classified rather than implemented silently. |
| Milestone-owned coverage | passed | Crate-local Rust tests cover canonical clauses, certificate parsing, rejection records, resolution replay, substitution/alpha/FV replay, imported facts, cluster/reduction replay, checker orchestration, determinism, replay cost, public enum policy, and soundness mutation failures. |
| Test expectation integrity | passed | No existing `.miz` fixture or expectation sidecar was changed to match implementation behavior. Source-derived certificate corpus support remains explicitly deferred. |
| Design/source synchronization | passed | Paired source/spec, bilingual, public enum, soundness, determinism, and module-boundary audits are synchronized with the source layout and public module table. |
| Boundary discipline | passed | The task-22 legacy milestone checks evidence only and contains no SAT solver. Post-correction tasks may add only the task-24 audited in-process SAT checker over kernel-derived SAT problems, with no ATP backend, proof search, proof-policy projection, cache/artifact coupling, overload resolution, cluster search, implicit coercion insertion, fallback inference, or global mutable state reads. |
| Verification | passed | Closeout broad commands, paired-document link/count checks, and diff checks passed before commit. |
| Residual risk | passed with classified items | Remaining risks are listed below as `external_dependency_gap` or `deferred`. |

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 19/20 |
| Traceability | 15/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 3/5 |
| Total | 95/100 |

The score deducts for unavailable source-derived certificate corpus coverage,
missing external certificate producers and downstream proof/cache/artifact
consumers, lack of real backend-generated MiniSAT traces in this milestone, and
large parent implementation files that remain a maintenance watchlist even
after the Task 22 test-module split. These are classified and do not cap the
score because the crate-local milestone does not own those seams and no hard
gate fails.

## Review Results

| Review | Result |
|---|---|
| Implementation specification / documentation review | High invalid Task 15 ledger hash finding fixed by backfilling the full commit hash; final focused re-review found no blocking/high/medium findings. |
| Test sufficiency review | Medium paired-document link/check recording finding fixed by running and recording the deterministic pair/link check; final focused re-review found no blocking/high/medium findings. |
| Full implementation review | No blocking/high/medium findings. Low provisional verification and abbreviated Task 4 hash notes were fixed before commit. |
| Source/documentation consistency review | Medium provisional status/verification drift finding fixed after closeout reviews and verification completed; final focused re-review found no blocking/high/medium findings. |
| Read-only crate quality review | Hard gates pass with no blocking/high/medium findings. Valid quality score: 95/100, which is >= 90. |

## Deferred Items

| ID | Class | Reason | Owner / unblock condition |
|---|---|---|---|
| KERNEL-CLOSEOUT-G001 | `external_dependency_gap` | No active source-to-kernel-evidence runner or `.miz` proof-verification corpus feeds formula/substitution evidence. | Add the owning staged-test/source-to-kernel-evidence runner before activating source-derived formula/substitution evidence corpus fixtures. |
| KERNEL-CLOSEOUT-G002 | `external_dependency_gap` | `mizar-atp` is not an active certificate/trace producer, and real MiniSAT-compatible backend traces are not available as a stable producer contract. | Build the ATP crate and trace extraction contract; kernel continues to replay normalized traces only. |
| KERNEL-CLOSEOUT-G003 | `external_dependency_gap` | `mizar-proof` is not an active policy consumer of `KernelCheckResult`; proof-status projection and externally authenticated evidence policy remain downstream. | Add proof-policy consumers in `mizar-proof` with their own crate plan and consumer contract. |
| KERNEL-CLOSEOUT-G004 | `external_dependency_gap` | `mizar-cache` and `mizar-artifact` do not provide active proof-cache/proof-witness consumer contracts for kernel outputs. | Downstream cache/artifact phases define validation and publication contracts before any kernel coupling is added. |
| KERNEL-CLOSEOUT-G005 | `external_dependency_gap` / `deferred` | Source-derived certificate/service envelopes, derived-fact payload schemas, service-envelope normalization, cancellation token plumbing, and external worker scheduling are integration concerns outside this crate. | Add producer/consumer tasks once upstream/downstream contracts exist; do not add placeholders here. |
| KERNEL-CLOSEOUT-G006 | `external_dependency_gap` / `deferred` | `mizar-checker` cluster/reduction payload production and richer semantic redex/LHS-to-RHS producer validation remain outside the trusted kernel. | Keep kernel replay limited to explicit normalized commitments; source-side cluster payload producers must land in their owning crate. |
| KERNEL-CLOSEOUT-G007 | `deferred` | Local-abbreviation closure/type-guard evidence, captured-free-variable closure evidence, inline substitution payload encoding, digest registry expansion beyond algorithm id 1, and downstream wildcard-arm checks remain future compatibility tasks. | Add separate spec-backed tasks if the owning producer or consumer contracts require them. |
| KERNEL-CLOSEOUT-G008 | `deferred` | Parent runtime modules are much smaller after Task 22, but `checker`, `substitution_checker`, and `certificate_parser` remain large trusted modules. | Future move-only maintenance tasks may split private runtime helpers only if reviewability becomes a bottleneck; do not mix behavior or API changes. |

No `repo_metadata_conflict` was observed.

## Human Review Surface

- Canonical English docs under `doc/design/mizar-kernel/en/`.
- Japanese companions under `doc/design/mizar-kernel/ja/`.
- Kernel source under `crates/mizar-kernel/src/`.
- Kernel lint-policy and unit tests under `crates/mizar-kernel/`.
- Upstream/downstream context:
  `doc/design/mizar-vc/en/crate_exit_report.md`,
  `doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md`,
  `doc/design/internal/en/07.crate_module_layout.md`,
  `doc/design/architecture/en/08.reasoning_boundary.md`,
  `doc/design/architecture/en/15.kernel_certificate_format.md`,
  `doc/design/architecture/en/16.substitution_and_binding.md`, and
  `doc/design/architecture/en/19.failure_semantics.md`.

## Test Expectation Summary

No existing `.miz` fixtures or expectation sidecars were changed to match
implementation behavior. Milestone-owned behavior is covered by Rust unit
tests, lint-policy guards, soundness mutation tests, determinism/replay-cost
tests, source/spec audits, or explicit deferred rows. Source-derived semantic
corpus coverage remains blocked by the external runner and producer gaps listed
above.

## Verification Commands

| Command | Result |
|---|---|
| deterministic paired-document and companion-link check | passed |
| `cargo fmt --check` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test` | passed |
| `git diff --check` | passed |
| `git diff --cached --check` | passed after explicit closeout path staging |

Unrun deferred commands:

- None. The broad workspace commands above cover the current workspace.

## Next-Task Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Start the next verification pipeline crate after the completed mizar-kernel
closeout. Before editing, verify a clean worktree, confirm the mizar-kernel
closeout commit in git log, and read
doc/design/mizar-kernel/en/crate_exit_report.md,
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-atp/en/todo.md,
doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md,
doc/design/internal/en/07.crate_module_layout.md,
doc/design/architecture/en/09.atp_interface_protocol.md,
doc/design/architecture/en/10.atp_backend_integration.md, and
doc/design/architecture/en/15.kernel_certificate_format.md. Begin with
preliminary task 0 for mizar-atp: create or update the paired English/Japanese
crate plan, classify specification gaps, test gaps, source/design drift,
external dependencies, and deferred items, and commit that plan as its own
task. Preserve the one-task-one-commit rule; do not scaffold mizar-atp source
until the task-0 plan commit exists.
```

Rationale: this task-22 handoff is superseded by the post-closeout correction
tracked in tasks 23-29. `mizar-atp` must now emit candidate formula/substitution
evidence, not trusted normalized certificates or MiniSAT-compatible traces.
Keep `xhigh` because the work crosses external backend execution, evidence
projection, kernel soundness, and downstream proof-policy boundaries.
Lower reasoning is appropriate only for typo-only documentation sync; raise
only if repository metadata or specification contradictions block the crate
plan.
