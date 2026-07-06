# Crate Exit Report: mizar-kernel

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete.
Quality score: 95/100.
Score caps applied: none.

Post-closeout correction: commit `c6d94fe51923aa0363ea7297bfe4e9f905aef076`
supersedes the task-22 evidence target. Tasks 23-31 complete the corrected
formula/substitution evidence pipeline, trusted in-process SAT checking,
legacy path migration audit, explicit proof-obligation/consistency
goal-polarity binding, and kernel-side context-identity verification for
non-imported formula sources. Task 31 is the post-correction
soundness-contract closure point for F2; its self-hash is recorded by a later
bookkeeping point after commit.

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
- Legacy certificate/resolution-trace checking is gated behind
  `KernelCheckPolicy.allow_legacy_certificate_audit`; default normal proof
  policy rejects it before replay. Explicit audit mode may replay it for
  checked-record diagnostics, but still returns `Rejected` without trusted
  `final_goal` or `used_axioms`, so it is migration/audit-only.
- `check_kernel_evidence` now requires callers to declare
  `KernelEvidenceCheckKind`. Proof obligations require refutation polarity,
  consistency checks require consistency polarity, and accepted consistency
  checks are carried as diagnostic-only/non-trusted material for downstream
  `mizar-proof` policy, selection, status, and witness boundaries.
- `check_kernel_evidence` now requires the task-28 context-identity payload for
  local-hypothesis, cited-premise, and generated-VC-fact formula sources,
  verifies the target and documented context-identity hash, matches each
  non-imported formula entry against immutable source/id and fingerprint rows,
  and rejects missing, stale, or ambiguous rows before SAT encoding.

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
| 23 | `a326afc7a69913c1d716133620c2c608b78b0ae1` | `docs(kernel-task-23): correct evidence format` |
| 24 | `abc557d5f6f53b6530301a67c29570a23c67b874` | `docs(kernel-task-24): audit trusted SAT checker` |
| 25 | `35ef60ffba949254e71d86f9be2570b37e5f4a3c` | `feat(kernel-task-25): parse formula evidence` |
| 26 | `e48c4ffe78fa03c63f9ed60d4c3f81db95803af9` | `feat(kernel-task-26): encode formula evidence as SAT` |
| 27 | `222bf8bc30e59dd95818d828dd71ff823ff84f83` | `feat(kernel-task-27): wrap trusted SAT checker` |
| 28 | `43674a221dd5f43259c480846db7428f85ac9386` | `feat(kernel-task-28): check formula evidence with SAT` |
| 29 | `0cbcbf01c4b5c2e53c872d6edd35cf38065f90a8` | `fix(kernel-task-29): gate legacy certificate audit` |
| 30 | `f3197e12a8f7a2124da8ebbf0f678cf3cf6bd890` | `fix(kernel-task-30): bind evidence goal polarity` |
| 31 | `a62bae00bb23845e6636c8b39cebb9043898cc03` | `fix(kernel-task-31): verify context identity` |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | Module specs, source/spec audit, bilingual sync audit, module-boundary audit, and closeout reviews record no unresolved blocking/high specification inconsistency. |
| Source behavior documented or deferred | passed | Public modules, public items, tests, and promised behavior are traced in `source_spec_audit.md`; unsupported source-derived and downstream behavior is classified rather than implemented silently. |
| Milestone-owned coverage | passed | Crate-local Rust tests cover canonical clauses, certificate parsing, rejection records, resolution replay, substitution/alpha/FV replay, imported facts, cluster/reduction replay, checker orchestration, goal-polarity/check-kind binding, context-identity verification, determinism, replay cost, public enum policy, and soundness mutation failures. |
| Test expectation integrity | passed | No existing `.miz` fixture or expectation sidecar was changed to match implementation behavior. Source-derived certificate corpus support remains explicitly deferred. |
| Design/source synchronization | passed | Paired source/spec, bilingual, public enum, soundness, determinism, and module-boundary audits are synchronized with the source layout and public module table. |
| Boundary discipline | passed | The task-22 legacy milestone checks evidence only and contains no SAT solver. Post-correction tasks may add only the task-24 audited in-process SAT checker over kernel-derived SAT problems, with no ATP backend, proof search, proof-policy projection, cache/artifact coupling, overload resolution, cluster search, implicit coercion insertion, fallback inference, or global mutable state reads. |
| Verification | passed | Task-31 focused/crate-local checks, broad clippy/test, diff check, and cached-diff check passed before commit. |
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
| Implementation specification / documentation review | Task 31 spec/doc review initially found an invalid plan to recompute the opaque `mizar-vc` canonical handoff hash from parser bytes; follow-up review found the stable bridge hash algorithm underdocumented. Paired design docs now specify the canonical handoff hash as opaque caller context and document the task-31 context-identity hash grammar/bridge hashing; final focused re-review found no findings. |
| Test sufficiency review | Task 31 test review initially found helper-derived source-class coverage, golden grammar, and constructor-limit gaps. Hand-built local-hypothesis/cited-premise/generated-VC-fact rows, formula-id mismatch rejection, all-source golden grammar, constructor limit rejection, and PolicyBoundedBuiltin exemption coverage fixed them; final re-review found no findings. |
| Full implementation review | Task 31 full implementation review initially found the runtime `max_context_identity_entries` check occurred after hash/row scanning. The service now checks the runtime limit before recomputing context-identity hashes or scanning rows; final re-review found no findings. |
| Source/documentation consistency review | Task 31 source/doc review initially found stale ledger/exit-report bookkeeping and stale `mizar-proof` active-consumer wording. Paired docs now record the task-31 reviews, limited proof-obligation consumer reality, and unchanged `doc/design/spec_coverage_audit.md` status because no spec coverage ownership, traceability, owner-crate, or deferred-coverage classification changed. |
| Read-only crate quality review | The post-correction hard gates remain satisfied after task 31: trusted acceptance still flows only through parsed formula/substitution evidence, kernel-derived SAT checking, proof-obligation polarity binding, and verified context identity. Residual source-runner, producer, proof-policy, cache/artifact, and richer substitution gaps remain classified; valid quality score remains 95/100, which is >= 90. |

## Deferred Items

| ID | Class | Reason | Owner / unblock condition |
|---|---|---|---|
| KERNEL-CLOSEOUT-G001 | `external_dependency_gap` | No active source-to-kernel-evidence runner or `.miz` proof-verification corpus feeds formula/substitution evidence. | Add the owning staged-test/source-to-kernel-evidence runner before activating source-derived formula/substitution evidence corpus fixtures. |
| KERNEL-CLOSEOUT-G002 | `external_dependency_gap` | `mizar-atp` is not an active formula/substitution evidence candidate producer. MiniSAT-compatible backend traces are legacy migration/audit material, not trusted output. | Build the ATP crate around candidate formula/substitution evidence production after the VC handoff contract exists; do not add trusted backend proof translation. |
| KERNEL-CLOSEOUT-G003 | `external_dependency_gap` | `mizar-proof` now consumes accepted proof-obligation `KernelCheckResult` values at the status, trusted used-axiom, and witness-boundary fixture level, but richer proof-policy projection and externally authenticated evidence policy remain downstream. | Expand proof-policy consumers in `mizar-proof` under their own crate plan and consumer contract. |
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
| `cargo fmt --check` | passed |
| `cargo test -p mizar-kernel context_identity --lib` | passed |
| `cargo test -p mizar-kernel sat_backed_kernel_evidence --lib` | passed |
| `cargo test -p mizar-kernel --test lint_policy` | passed |
| `cargo test -p mizar-kernel` | passed |
| `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` | passed |
| `cargo test -p mizar-proof trusted_used_axioms_from_kernel_result` | passed |
| `cargo clippy -p mizar-proof --all-targets --all-features -- -D warnings` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test` | passed after fixing a source/spec audit lint phrase |
| `git diff --check` | passed |
| `git diff --cached --check` | passed after explicit task-31 path staging |

Unrun deferred commands:

- None. The broad workspace commands above cover the current workspace.

## Next-Task Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue Step 1 with `mizar-test` task 21 after the completed mizar-kernel
task-31 context-identity verification. Before editing, verify a clean worktree,
confirm the mizar-kernel task 31 commit in git log, and read
doc/design/todo.md,
doc/design/mizar-test/en/todo.md,
doc/design/mizar-test/en/fail_soundness.md,
doc/design/mizar-kernel/en/crate_exit_report.md,
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/checker.md,
doc/design/mizar-kernel/en/source_spec_audit.md,
doc/design/mizar-kernel/en/soundness_argument.md,
doc/design/mizar-vc/en/kernel_evidence_handoff.md,
doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md,
doc/design/internal/en/07.crate_module_layout.md,
doc/design/architecture/en/08.reasoning_boundary.md,
doc/design/architecture/en/15.kernel_certificate_format.md,
doc/design/architecture/en/19.failure_semantics.md, and
tests/coverage/spec_trace.toml. Begin with mizar-test task 21: extend the
required soundness-case registry and test documentation with the corrected-path
rejection vocabulary for F7. Keep the work docs-plus-corpus/registry scoped:
do not change checker/core/VC implementation semantics, fabricate payloads,
activate unverified fixtures, or rebaseline expectations to match current
implementation behavior. Preserve one task per commit.
```

Rationale: `mizar-test` task 21 is the next task in the requested Step 1 order
after the producer-side and kernel-side F2 closures. Keep `xhigh` because the
work closes F7 at the shared soundness vocabulary boundary and must preserve
existing spec/test intent while linking the corrected kernel rejection taxonomy
to the corpus registry. Lower reasoning is appropriate only for typo-only
documentation sync; raise only if repository metadata or specification
contradictions block the handoff.
