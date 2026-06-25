# Task Ledger: mizar-kernel

> Canonical language: English. Japanese companion:
> [../ja/task_ledger.md](../ja/task_ledger.md).

This ledger is the restart point for autonomous `mizar-kernel` crate work.
Before starting any task, check `git status`, `git log`, this table, and
[todo.md](./todo.md). A task is complete only when its commit exists in
history, final review outcomes are known, verification results are known, and
deferred reasons are recorded. A commit cannot contain its own final hash, so
self-hashes are verified from `git log` before the next task starts and
backfilled by a later committed bookkeeping point or the closeout task.

| Task | Status | Commit | Reviews | Verification | Deferred / notes |
|---|---|---|---|---|---|
| 0. Crate plan | complete | `81ffb5561fc1b24ae355d216e1a455d2a487d923` | Spec/doc review: low pending-status finding fixed; final re-review no findings. Test sufficiency review: medium `--all-features` and conditional cross-crate verification findings fixed; final re-review no findings. Full implementation review: high sequencing and medium cluster-gate/status findings fixed; final re-review no findings. Source/doc consistency review: medium internal-04 rejection-reason and low JA companion-link findings fixed; final re-review no findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Docs-only. Creates paired crate plan and ledger, classifies initial `spec_gap`, `test_gap`, `design_drift`, `source_drift`, `external_dependency_gap`, `deferred`, and `repo_metadata_conflict` state, records kernel prohibitions and trusted-baseline lint policy, strict linear task sequencing, internal-04 rejection reason coverage, and cluster trace external-readiness gates, and does not create crate source. |
| 1. Crate scaffold and trusted-baseline lint policy | complete | `63cbcd83a82005d8ffe98f7c87928fa46e95649c` | Spec/doc review: medium public-surface and dependency-escape findings fixed; low TODO/ledger timing finding resolved by final ledger update. Test sufficiency review: medium dependency-subtable and low workspace-member scanner findings fixed; final re-review no findings. Full implementation review: high task-0 hash, medium dependency-subtable, medium split-public-surface, and medium extern-ABI public-surface findings fixed; final re-review no findings. Source/doc consistency review: medium dependency-guard and low trusted-baseline decision findings fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Scaffold-only. Adds workspace member, lockfile entry, minimal crate manifest, `#![forbid(unsafe_code)]` crate-root trust statement, and lint-policy guard. Production dependencies are exactly `mizar-core` and `mizar-session`; dev/build/target dependency sections, public semantic surface, downstream ATP/proof/cache/artifact coupling, module specs, semantic modules, `.miz` fixtures, expectations, and `doc/spec` edits remain absent/out of scope. |
| 2. Spec: `clause.md` | complete | `b0fa89a9eecc85da96bf8351fc2e147423747730` | Spec/doc review: high empty-clause, medium test-coverage, low trust-prohibition, and medium validation-context signature findings fixed; final re-review no findings. Test sufficiency review: medium planned-test coverage, medium hash-test coverage, and low symbol-kind ordering coverage findings fixed; final re-review no findings. Full implementation review: high empty-clause plus medium tautology-marker, validation-context, and canonical-ordering findings fixed; final re-review no findings. Source/doc consistency review: medium validation-context signature finding fixed; final re-review no findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Spec-only. Adds paired clause module specs for literals, canonical ordering, structural well-formedness, explicit empty and tautology forms, clause-local validation context, deterministic rendering/hash inputs, trust statement, planned task-3 tests, and clause-specific gaps. No Rust source, `.miz` fixtures, expectations, or `doc/spec` edits. |
| 3. Implement clause representation | complete | `4020ac12fafe24aa8205f7fd3df8ece37027804e` | Spec/doc review: medium public `Term` ordering and medium clause-hash preallocation findings fixed; final re-review no blocking/high/medium findings. Test sufficiency review: high canonical-order coverage, medium hash-field and hash-exclusion coverage, and low marker/single/empty coverage findings fixed; final re-review no findings. Full implementation review: medium `Term` ordering, medium unchecked length casts, low missing `#[non_exhaustive]`, and medium preallocation resource-bound findings fixed; final re-review no blocking/high/medium findings. Source/doc consistency review: medium `Term` ordering drift fixed; final re-review no blocking findings, with ledger/TODO backfill completed in this task. | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Rust source task. Implements `clause` data model, validation context, deterministic ordering/rendering/hash input, explicit empty and zero-payload tautology marker forms, checked canonical byte framing, resource-bound validation before large allocation, module exposure, and lint guard update. No SAT/ATP/proof search, downstream ATP/proof/cache/artifact coupling, `.miz` fixtures, expectations, or `doc/spec` edits. Cross-crate `mizar-core`/`mizar-checker` tests not required because this task does not touch the binder contract or checker/trace boundary. |
| 4. Spec: `certificate_parser.md` | complete | `b900639e4057ea2ba1a1158688a35e188ec` | Spec/doc review: high concrete-byte/schema gap and imported proof-status gap plus medium hash-algorithm and failure-location findings fixed; later high imported-fact id and medium non-parent `ClauseRef` findings fixed; final re-review no blocking/high/medium findings. Test sufficiency review: high stable-failure and hash-coverage findings plus medium reference/resource/ordering findings fixed; final re-review no findings. Full implementation-boundary review: high concrete-byte gap and medium generated-clause/hash-dependency findings fixed; final re-review no blocking/high/medium findings. Source/doc consistency review: medium task-3 backfill and schema-ownership/TODO findings fixed; final re-review no blocking findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Spec-only. Adds paired certificate parser specs with schema ownership, concrete schema/encoding v1 envelope, section tags, directory and item payload layouts, parser-owned manifests, public clause integration, stable failure detail/location mapping, hash-input rules without digest dependency, planned task-5 tests, and `external_dependency_gap`/`deferred` records. No Rust source, `.miz` fixtures, expectations, or `doc/spec` edits. |
| 5. Implement certificate parsing and structural validation | complete | `60c92cc53c77ec3240fe5410fc04c449bd04b267` | Spec/doc review: final re-review no findings after EN/JA clarification for range out-of-bounds and noncanonical/shuffled hash-input semantics. Test sufficiency review: medium location/hash/resource/ordering gaps and later medium resolution-step ordering gap fixed; final re-review no findings. Full implementation review: high resource preallocation and generated-literal budget findings plus medium term-budget and item-location findings fixed; final re-review no findings. Source/doc consistency review: final re-review no findings, with only expected pre-commit bookkeeping noted. | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. `cargo test -p mizar-core` and `cargo test -p mizar-checker` were not run because this task does not alter the `mizar-core` binder contract or the checker/trace boundary semantics; it only decodes certificate bytes and delegates generated clause validation to the existing local `clause` API. | Rust source task. Implements schema/encoding v1 envelope parsing, deterministic fixed-section directory validation, item-frame parsing, parser-owned manifest/reference/schema types, generated clause structural validation through `clause`, stable rejection category/detail/location reporting, canonical hash input bytes, parser resource limits before large allocation, and expanded lint coverage. Updates paired certificate parser docs only to clarify implemented range/hash semantics. No SAT/ATP/proof search, producer/cache/artifact coupling, `.miz` fixtures, expectations, or `doc/spec` edits. External producer/consumer integration remains `external_dependency_gap`/`deferred`. |
| 6. Spec: `rejection.md` | complete | `f4b1abc63a46cd7d628911aff4a7ce91c0c5555b` | Spec/doc review: medium EN/JA planned-test sync, context-mismatch/profile ambiguity, target sort-key, cluster mapping, witness-normalization ownership, and ordering-id findings fixed; final re-review no findings. Test sufficiency review: medium ordering, checker-location, category/detail ownership, parser target-key fallback, `clause_ref`, and checker mapping findings fixed; final re-review no findings. Full implementation-boundary review: medium profile ambiguity, imported proof-status, internal-04 reason coverage, target sort-key ownership, witness-normalization ownership, and planned mapping wording findings fixed; final re-review no findings. Source/doc consistency review: high context-mismatch drift and medium domain-separator mapping finding fixed; final re-review no findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Spec-only. Adds paired rejection specs defining `certificate_rejection`/`kernel_rejection`, stable detail keys, parser/checker mappings, target-owned deterministic record ordering, evidence locations, compatibility policy, planned task-7 tests, and internal-04 `missing_provenance`/`malformed_witness_data` coverage. Also syncs certificate parser docs to include unsupported domain separator mapping. No Rust source, `.miz` fixtures, expectations, or `doc/spec` edits. Downstream proof/cache/artifact consumers remain `external_dependency_gap`/`deferred`. |
| 7. Implement rejection records | complete | `acc8e7d62adbee21cb49b8d134fe0d846ee60603` | Spec/doc review: medium fixed-width target sort-key and evidence-id ordering/doc-sync findings fixed; final re-review no findings. Test sufficiency review: medium parser conversion, category/detail mapping, ordering, and lint coverage findings plus low atomic-token and isolated category/byte-offset/section ordering findings fixed; final re-review no findings. Full implementation review: high public-field constructor-bypass finding fixed; final re-review no findings. Source/doc consistency review: low reduction/derived/final ordering coverage finding fixed; final re-review no findings. | `cargo fmt --check` passed; `cargo test -p mizar-kernel` passed; `cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings` passed; `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. `cargo test -p mizar-core` and `cargo test -p mizar-checker` were not run because this task does not alter the `mizar-core` binder contract or checker/trace replay boundary; it only adds shared rejection record types and parser-error conversion over existing parser data. | Rust source task. Implements stable rejection categories/details, category/detail ownership validation, target VC fingerprint ordering, structured evidence locations, parser-error conversion, read-only rejection records, deterministic total ordering, module exposure, and trusted-boundary lint expansion. Syncs paired rejection docs only for implemented ordering details. No SAT/ATP/proof search, fallback inference, implicit coercion insertion, global mutable state, downstream proof/cache/artifact coupling, `.miz` fixtures, expectations, or `doc/spec` edits. Downstream proof/cache/artifact consumers remain `external_dependency_gap`/`deferred`. |
| 8. Spec: `resolution_trace.md` | ready to commit | pending self-hash | Spec/doc review: medium imported-parent context and final-goal checkedness findings fixed; initial stale sequencing concern resolved by clean status/log; final re-review no findings. Test sufficiency review: medium rejection-record shape and provenance planned-test findings plus low final-goal checkedness coverage finding fixed; final re-review no findings. Full implementation-boundary review: high preallocation and term-depth findings plus medium clause-context, clause-owned helper, imported-context validation, and resource-classification findings fixed; final re-review no findings. Source/doc consistency review: medium internal-04 `MissingProvenance` drift and low remaining internal-04 rejection-detail drift fixed; final re-review no findings. | `git diff --check` passed before explicit staging; `git diff --cached --check` passed after explicit path staging. | Spec-only. Adds paired resolution-trace specs defining deterministic MiniSAT-compatible replay, explicit imported clause context, clause-reference ownership, pivot orientation, bounded resolvent construction, clause-owned non-allocating/depth-bounded helper requirements, final-goal checkedness, replay rejection mapping, planned task-9 tests, and `external_dependency_gap`/`deferred` records. Syncs internal-04 EN/JA `RejectionReason` sketch with `MissingProvenance`, `MalformedWitnessData`, and `InvalidClusterTrace`. No Rust source, `.miz` fixtures, expectations, or `doc/spec` edits. Backend proof translation and proof/cache/artifact consumers remain `external_dependency_gap`/`deferred`. |
| 9. Implement resolution trace checker | not started | pending | pending | pending | Requires task 8 commit. Semantic dependency: task 7 rejection records. Rust source task. |
| 10. Spec: `substitution_checker.md` | not started | pending | pending | pending | Requires task 9 commit. Semantic dependency: task 4 certificate spec. Spec-only. |
| 11. Implement substitution checking | not started | pending | pending | pending | Requires task 10 commit. Semantic dependency: task 7 rejection records. Rust source task. |
| 12. Implement alpha-conversion and free-variable checks | not started | pending | pending | pending | Requires task 11 commit. Rust source task. |
| 13. Spec: `checker.md` | not started | pending | pending | pending | Requires task 12 commit. Semantic dependencies: task 6 rejection spec, task 8 resolution spec, and task 10 substitution spec. Spec-only. |
| 14. Implement imported-fact checking | not started | pending | pending | pending | Requires task 13 commit. Rust source task. |
| 15. Implement cluster trace replay | not started | pending | pending | pending | Requires task 14 commit. Semantic dependency: task 13 checker spec plus external `mizar-checker` cluster trace payload readiness review or deferred record. Rust source task. |
| 16. Kernel check service and deterministic batch ordering | not started | pending | pending | pending | Requires task 15 commit. Semantic dependencies: task 9 resolution checker, task 12 substitution checker, task 14 imported-fact checking, and task 15 cluster replay. Rust source task. |
| 17. Soundness fail-test corpus | not started | pending | pending | pending | Requires task 16 commit. Test/audit task; source-derived corpus runner gaps may remain `external_dependency_gap`. |
| 18. Determinism and replay-cost suite | not started | pending | pending | pending | Requires task 17 commit. Semantic dependency: task 16 checker service. Test task. |
| 19. Public-enum forward-compatibility policy | not started | pending | pending | pending | Requires task 18 commit. Semantic dependency: task 16 public API surface. Test/docs task. |
| 20. Source/spec correspondence and prohibition audit | not started | pending | pending | pending | Requires task 19 commit. Audit task. |
| 21. Bilingual documentation sync audit | not started | pending | pending | pending | Requires task 20 commit. Docs audit task. |
| 22. Module-boundary refactor gate | not started | pending | pending | pending | Requires task 21 commit. Audit or move-only task. |
| Closeout. Crate exit report and quality review | not started | pending | pending | pending | Requires task 22 commit, all hard gates passing, and read-only quality review score >= 90/100. |

## Task 8 Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-kernel autonomous crate development from the completed task 8
resolution-trace spec commit. Before starting task 9, verify a clean worktree,
confirm the task 8 commit exists in git log, and re-read
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/task_ledger.md, doc/design/mizar-kernel/en/todo.md,
doc/design/mizar-kernel/en/resolution_trace.md,
doc/design/mizar-kernel/en/certificate_parser.md,
doc/design/mizar-kernel/en/clause.md,
doc/design/mizar-kernel/en/rejection.md,
doc/design/architecture/en/15.kernel_certificate_format.md,
doc/design/architecture/en/19.failure_semantics.md, and
doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md.
Implement task 9 only: add the resolution trace replay checker, expose its
module, and add focused tests for valid replay, pivot polarity, resolvent
mismatch, imported-clause context/provenance, checked final-goal behavior,
stable rejection records, deterministic output, and replay resource limits.
Add only the small clause-owned non-allocating canonical length / bounded-writer
and depth-bounded validation helper(s) needed by the spec, plus a crate-private
certificate helper only if needed to derive the replay validation context from
public parsed data. Do not implement SAT solving, ATP proof translation, proof
search, imported-fact availability checking beyond explicit context validation,
substitution checking, cluster replay, checker orchestration, proof/cache/
artifact integration, fallback inference, implicit coercion insertion, or
global mutable state reads. Run cargo fmt --check, cargo test -p mizar-kernel,
cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings,
git diff --check, and git diff --cached --check after explicit path staging.
Use review-only agents for the required AGENTS.md review phases and commit
task 9 by itself.
```

Rationale: task 9 implements the first semantic replay checker inside the
trusted kernel, so allocation bounds, depth bounds, parent orientation, and
stable rejection locations are soundness-critical. Keep `xhigh`; lower only for
comment-only follow-up, and raise only if the existing clause/parser APIs make
the specified helper boundaries impossible without a broader design update.

## Task 7 Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-kernel autonomous crate development from the completed task 7
rejection-records commit. Before starting task 8, verify a clean worktree,
confirm the task 7 commit exists in git log, and re-read
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/task_ledger.md, doc/design/mizar-kernel/en/todo.md,
doc/design/mizar-kernel/en/rejection.md,
doc/design/mizar-kernel/en/certificate_parser.md,
doc/design/architecture/en/15.kernel_certificate_format.md,
doc/design/architecture/en/19.failure_semantics.md, and
doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md.
Implement task 8 only: write paired English/Japanese resolution-trace specs
doc/design/mizar-kernel/en/resolution_trace.md and
doc/design/mizar-kernel/ja/resolution_trace.md. Specify deterministic
MiniSAT-compatible resolution trace replay/checking over the normalized
certificate schema, clause-reference ownership, antecedent/pivot validation,
linear replay/resource bounds, stable rejection mapping through
invalid_sat_proof/resource_exhaustion/missing_provenance, and explicit
kernel prohibitions. Do not add Rust source and do not implement a SAT solver,
ATP backend, proof search, premise selection, overload resolution, cluster
search, implicit coercion insertion, fallback inference, or global mutable
state. Run git diff --check and git diff --cached --check after explicit path
staging. Use review-only agents for the required AGENTS.md review phases and
commit task 8 by itself.
```

Rationale: task 8 defines the replay contract that the trusted checker will
implement in task 9. Keep `xhigh` because trace replay is part of the soundness
boundary and must stay a deterministic evidence checker rather than a search
or solver. Lower reasoning is appropriate only for typo-only docs sync; raise
only if architecture 15 or the existing certificate schema contradicts the
planned trace model.

## Task 6 Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-kernel autonomous crate development from the completed task 6
rejection spec commit. Before starting task 7, verify a clean worktree, confirm
the task 6 commit exists in git log, and re-read
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/task_ledger.md, doc/design/mizar-kernel/en/todo.md,
doc/design/mizar-kernel/en/rejection.md,
doc/design/mizar-kernel/en/certificate_parser.md,
doc/design/architecture/en/15.kernel_certificate_format.md,
doc/design/architecture/en/19.failure_semantics.md, and
doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md.
Implement task 7 only: add src/rejection.rs, expose it from src/lib.rs, and
add focused tests for stable category/detail keys, parser conversion preserving
target_vc_fingerprint and locations, deterministic ordering, allowed and
disallowed category/detail mappings, #[non_exhaustive] public enums, and
trusted-boundary lint coverage. Do not implement resolution, substitution,
imported-fact, cluster, or checker-service replay logic beyond record types and
test fixtures. Run cargo fmt --check, cargo test -p mizar-kernel,
cargo clippy -p mizar-kernel --all-targets --all-features -- -D warnings,
git diff --check, and git diff --cached --check after explicit path staging.
Use review-only agents for the required AGENTS.md review phases and commit
task 7 by itself.
```

Rationale: task 7 turns the stable failure vocabulary into the shared record
surface that later checkers consume. Keep `xhigh` because category/detail
ownership, target-owned ordering, and parser conversion are part of the trusted
soundness boundary. Lower reasoning is appropriate only for typo-only docs
sync; raise only if parser APIs or architecture 19 conflict with the spec.

## Task 5 Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-kernel autonomous crate development from the completed task 5
certificate parser implementation commit. Before starting task 6, verify a clean
worktree, confirm the task 5 commit exists in git log, and re-read
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/task_ledger.md, doc/design/mizar-kernel/en/todo.md,
doc/design/mizar-kernel/en/certificate_parser.md,
doc/design/architecture/en/15.kernel_certificate_format.md, and
doc/design/architecture/en/19.failure_semantics.md. Implement task 6 only:
write paired English/Japanese rejection semantics specs
doc/design/mizar-kernel/en/rejection.md and
doc/design/mizar-kernel/ja/rejection.md. Define stable parser/checker rejection
categories and structured details/locations without adding Rust source. Keep the
kernel as an evidence checker: no proof search, ATP search, premise selection,
overload resolution, cluster search, implicit coercion insertion, fallback
inference, or global mutable compiler state. Run git diff --check and
git diff --cached --check after explicit path staging. Use review-only agents
for the required AGENTS.md review phases and commit task 6 by itself.
```

Rationale: task 6 specifies the stable rejection vocabulary consumed by every
later checker. Keep `xhigh` because rejection categories are part of the trusted
boundary and architecture 19 failure-semantics compatibility policy. Lower
reasoning is appropriate only for typo-only synchronization; raise only if the
architecture documents contradict the existing parser rejection surface.

## Task 0 Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-kernel autonomous crate development from the completed task 0
crate-plan commit. Before starting task 1, verify a clean worktree, confirm the
task 0 commit exists in git log, and re-read
doc/design/mizar-kernel/en/00.crate_plan.md,
doc/design/mizar-kernel/en/task_ledger.md, doc/design/mizar-kernel/en/todo.md,
doc/design/internal/en/07.crate_module_layout.md,
doc/design/architecture/en/08.reasoning_boundary.md,
doc/design/architecture/en/15.kernel_certificate_format.md, and
doc/design/architecture/en/19.failure_semantics.md. Implement task 1 only: add
the mizar-kernel workspace member, minimal crate manifest, crate-root trust
statement, and trusted-baseline lint-policy guard. Keep production dependencies
limited to mizar-session and mizar-core, forbid unsafe code, and do not expose
semantic modules until paired module specs exist. Run cargo fmt --check,
cargo test -p mizar-kernel, cargo clippy -p mizar-kernel --all-targets
--all-features -- -D warnings, git diff --check, and git diff --cached --check
after explicit path staging. Use review-only agents for the required AGENTS.md
review phases.
```

Rationale: task 1 creates the trusted crate boundary and dependency guard that
all later kernel work relies on. Keep `xhigh` because dependency discipline,
trusted lint policy, and no-search/no-ATP boundaries are soundness-critical.
Lower reasoning is appropriate only for typo-only documentation cleanup; raise
only if repository metadata or contradictory specifications block the scaffold.
