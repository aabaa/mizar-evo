# Crate Exit Report: mizar-proof

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: complete for the `mizar-proof` proof-policy milestone.
Quality score: 94/100.
Score caps applied: none.

Post-closeout metadata correction: task 20 found a stale `mizar-atp` task-28
closeout guard that still treated formal `crates/mizar-proof` as a placeholder.
The focused correction commit `36d1a9c` resolved that repository metadata drift
before finalizing this report.

## Scope

Milestone scope:

- Build the `mizar-proof` workspace crate from task 0 through task 19 and this
  closeout task.
- Own proof policy evaluation over untrusted candidate evidence, explicit
  kernel-check outputs, deterministic built-in discharge evidence, externally
  attested records, policy assumptions, and open obligations.
- Own deterministic winner selection and artifact-facing proof-selection merge
  without using arrival order, completion time, or runtime duration as proof
  identity.
- Own proof status projection, including trusted `used_axioms` propagation only
  from `mizar-kernel::checker::KernelCheckResult` values whose status is
  `Accepted` and whose evidence check kind is `ProofObligation`.
- Own proof witness staging, manifest-gated publication references, and stable
  proof-reuse metadata exported as validation predicates only.
- Own ATP early-stop policy queries that are class/rank based and never turn
  backend progress, diagnostics, cache records, or external attestations into
  trusted acceptance.

Included:

- English/Japanese crate plan, module specifications, source/spec audit,
  bilingual sync audit, architecture-22 audit, module-boundary audit, and this
  exit report under `doc/design/mizar-proof/{en,ja}/`.
- Rust source under `crates/mizar-proof/src/`.
- Private unit tests under `crates/mizar-proof/src/<module>/tests.rs`.
- Crate-local determinism and lint-policy integration tests under
  `crates/mizar-proof/tests/`.

Excluded:

- Kernel acceptance, SAT solving, ATP backend execution, proof search, premise
  selection, substitution invention, overload resolution, cluster search,
  implicit coercion insertion, fallback inference, cache lookup, artifact
  manifest commit, or source-derived corpus extraction.
- Promotion of backend proof methods, resolution traces, SMT proof objects,
  backend logs, externally attested records, cache records, or backend
  diagnostics into trusted proof status or trusted `used_axioms`.
- Placeholder downstream integration with unfinished `mizar-cache`,
  `mizar-artifact`, or `mizar-atp` consumers/producers.
- Downstream adoption of `mizar-proof` APIs by `mizar-atp`; commit `36d1a9c`
  only corrected stale ATP closeout metadata and did not wire integration.

## Task Commits

| Task | Commit | Subject |
|---|---|---|
| 0 | `a1f6d1dba4ecc46aa2d434c8da8ae0279a2c23ec` | `docs(proof-task-0): add autonomous crate plan` |
| 1 | `bc7d33efd8b7427fa026807e729642219f62f809` | `feat(proof-task-1): scaffold proof policy crate` |
| 2 | `193598cd5db448b17c5cddb721a5e22010b241b6` | `docs(proof-task-2): specify proof policy` |
| 3 | `f3ac274d9cf7266f5ebdc4de94fe963f72ba67d5` | `feat(proof-task-3): implement policy evaluator` |
| 4 | `986f62a184fa8d0a7fa0c98ef8f6c9669d85d844` | `feat(proof-task-4): handle external evidence admission` |
| 5 | `6802bbec9f75d33f6c28b06391d60101726b2d51` | `docs(proof-task-5): specify winner selection` |
| 6 | `9230c36464a58a4f35a43ae1f7dc9fcde6e5e94d` | `feat(proof-task-6): implement winner selection` |
| 7 | `6e9a5a0400aae2c4c8b5b8098594ecc0bd3d2949` | `feat(proof-task-7): merge artifact proof selections` |
| 8 | `76f112237c55dab875cd3e26cbfa11d45439fe5e` | `docs(proof-task-8): specify status projection` |
| 9 | `ccd0e05820f61c02614ac523bf444556c5b29fa5` | `feat(proof-task-9): project proof statuses` |
| 10 | `d77163f8bd1f2c254a7935164e2135567ba9b3a0` | `docs(proof-task-10): specify witness store` |
| 11 | `6d2efc72e9514a44e5f4e81fbbaf7e78ecd73dba` | `feat(proof-task-11): implement witness store` |
| 12 | `b575fd0cf2e176e5c76fd117c56b18142059b997` | `feat(proof-task-12): add portfolio early-stop hooks` |
| 13 | `ba8c696e7b154a6fd9389222334d6ea8f3b5d6d7` | `test(proof-task-13): add determinism suite` |
| 14 | `c3cd67512a460afde3319101504ef45524ccb302` | `test(proof-task-14): guard public enum compatibility` |
| 15 | `7fdfe4945ba885f8d4f6990d023a6ee0aa35744d` | `docs(proof-task-15): audit source spec correspondence` |
| 16 | `f58c9a6203179da1b360024fbc3a071263271c3b` | `docs(proof-task-16): audit bilingual sync` |
| 17 | `f53d6e2e9adc4db740c94f480f49a064662bd190` | `feat(proof-task-17): export proof reuse metadata` |
| 18 | `aaf14d6f83357691d01ea2ce60b8fda99e89ac9c` | `docs(proof-task-18): audit architecture 22 reuse metadata` |
| 19 | `fe0735bc527935532a9ce6038f5597c7a03ecf57` | `refactor(proof-task-19): split private test modules` |
| 20 | pending self-hash | `docs(proof-task-20): add crate exit report` |

## Final Owned Surfaces

| Surface | Final shape |
|---|---|
| Proof policy | `ProofPolicyEvaluator` classifies explicit policy candidates, records policy diagnostics, computes policy fingerprints, controls external evidence admission, and answers ATP early-stop queries. Accepted kernel input becomes trusted only through explicit `KernelPolicyInput` origin plus accepted proof-obligation `KernelCheckResult`; accepted consistency checks are diagnostic-only, and policy-tainted kernel output is non-trusted. |
| Deterministic selection | `select_winner` and artifact merge APIs select by class rank and stable tie-break identity. Raw completion order, runtime, and backend timing never participate in proof identity. `require_kernel_certificates` blocks externally attested winners. |
| Status projection | `project_status` maps selected proof evidence to artifact/diagnostic status and propagates trusted `used_axioms` only from accepted proof-obligation kernel evidence with matching hashes. External, backend diagnostic, cache, rejected, consistency-check, and open statuses stay distinguishable and non-trusted. |
| Witness store | The `witness_store` module stages deterministic witness payload hashes with `stage`, provides unpublished `ProofWitnessRef` candidates for kernel-verified formula/substitution evidence, and returns committed publication refs only through `publish_ref` with opaque committed artifact-manifest reachability proof. `selected_proof_witness_hash` is selection/status metadata, not a committed publication ref. `DischargedBuiltin` witness publication remains unsupported until artifact schema support exists. |
| ATP early stop | Early-stop decisions are policy/class based, require an observed selectable class, and are blocked by equal or higher pending selectable classes. They do not cancel based on time or backend partial diagnostics. Downstream `mizar-atp` adoption remains an `external_dependency_gap`. |
| Proof-reuse metadata | Status metadata exports policy fingerprint compatibility, obligation/context/dependency fingerprints, selected witness or deterministic discharge hash, evidence identity, dependency artifact/schema compatibility, selected provenance, stable selection reason, and validation hash. The metadata is a cache validation predicate, not proof authority. |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | Paired module specs, source/spec audit, architecture-22 audit, module-boundary audit, and closeout reviews record no unresolved blocking/high specification inconsistency in `mizar-proof`. |
| Source behavior documented or deferred | passed | Public modules, public items, tests, and promised behavior are traced in `source_spec_audit.md`; remaining cache/artifact/ATP integrations are classified rather than stubbed. |
| Milestone-owned coverage | passed | Crate-local Rust tests cover policy classification, external admission, deterministic selection, merge behavior, status projection, trusted used-axiom boundaries, witness staging/publication gating, early-stop queries, public enum policy, determinism, and proof-reuse metadata invalidation. |
| Test expectation integrity | passed | No existing `.miz` fixture, traceability row, or expectation sidecar was changed to match implementation behavior. No `.miz` tests were added because this crate owns Rust policy/projection/store behavior over upstream data shapes. |
| Design/source synchronization | passed | Paired source/spec, bilingual, architecture-22, and module-boundary audits match the source layout and public module table. |
| Boundary discipline | passed | `mizar-proof` records, selects, projects, stages, and exports metadata; it does not accept proofs, call SAT/ATP backends, run proof search, perform cache lookup, commit artifacts, or promote external/cache/backend material to trusted status. |
| Verification | passed | `mizar-proof`, adjacent kernel/VC/artifact/checker/ATP tests, full clippy, fmt, diff checks, and full workspace `cargo test` passed after the focused ATP metadata correction. |
| Residual risk | passed with classified items | Remaining risks are listed below as `external_dependency_gap` or `deferred`; no unresolved `repo_metadata_conflict` remains. |

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

The score deducts for unavailable downstream cache/artifact/ATP integrations,
deferred `DischargedBuiltin` artifact witness publication, copied
kernel/artifact acceptance metadata still needing a downstream production
token, producer-owned byte-level witness payload canonicality validation,
missing active source-derived proof-verification corpus coverage, remaining
large production modules after the private-test split, and future downstream
handoff work. These items are classified and outside this crate's proof-policy
ownership; they do not cap the score because full workspace verification is
clean.

## Review Results

| Review | Result |
|---|---|
| Implementation specification / documentation review | Task-20 review initially found stale ATP metadata-conflict wording after `36d1a9c` and missing closeout carry-forward for witness metadata/canonicality gaps; focused updates resolved them. Final re-review reported no findings. |
| Test sufficiency review | Task-20 review reported no additional proof tests required for docs-only closeout after full verification and ATP metadata correction passed. |
| Full implementation review | Task-20 review found witness metadata wording drift in docs but no required production API/behavior change; focused docs updates resolved it. |
| Source/documentation consistency review | Task-20 review initially found stale closeout wording and a `ProofWitnessStore` type-name overclaim; focused re-review reported no findings after EN/JA docs were synchronized. |
| Read-only crate quality review | Valid quality score: 94/100. Hard gates pass with no blocking/high findings and no unresolved repo metadata conflict. |

## Deferred Items

| ID | Class | Reason | Owner / unblock condition |
|---|---|---|---|
| PROOF-CLOSEOUT-G001 | `external_dependency_gap` | `mizar-cache` now exists and owns proof-reuse validation, but this proof-policy closeout does not call cache lookup, hit/miss decisions, or cache promotion APIs. | Keep `mizar-proof` metadata as validation input only; add any proof/cache consumer wiring in a separate integration task. |
| PROOF-CLOSEOUT-G002 | `external_dependency_gap` | Downstream `mizar-atp` adoption of early-stop queries and live backend cancellation is not wired. | Update `mizar-atp` in a separate task to call the policy API without moving policy or winner selection into ATP. |
| PROOF-CLOSEOUT-G003 | `external_dependency_gap` | Artifact committed publication tokens are intentionally opaque in `mizar-proof`; real phase-15 emission and manifest transaction integration remain downstream. | `mizar-artifact` / emitter tasks must supply committed reachability proof and exact witness coverage. |
| PROOF-CLOSEOUT-G004 | `external_dependency_gap` | `mizar-artifact` currently lacks trusted `DischargedBuiltin` witness publication support. | Extend artifact witness schema in its owning task before publishing built-in discharge witness refs. |
| PROOF-CLOSEOUT-G005 | `deferred` | Source-derived proof-verification corpus and producer extraction are unavailable for this policy/projection crate. | Add the source runner and producer contracts before adding `.miz` proof-policy fixtures. |
| PROOF-CLOSEOUT-G006 | `deferred` | Remaining production modules exceed 1,100 lines after task 19, but no smaller production helper boundary was identified that would reduce complexity without adding review risk. | Run a later move-only refactor only if quality review or downstream work identifies a concrete bottleneck. |
| PROOF-CLOSEOUT-G007 | resolved `repo_metadata_conflict` | `mizar-atp` task-28 closeout guard previously rejected workspace member `crates/mizar-proof` and the `crates/mizar-proof` directory as forbidden placeholders. | Resolved by focused metadata correction commit `36d1a9c`; no proof policy moved into `mizar-atp`. |
| PROOF-CLOSEOUT-G008 | `external_dependency_gap` | `TrustedKernelWitnessMetadata` remains opaque and has no production constructor until the kernel/artifact boundary exposes copied kernel acceptance metadata. | Keep trusted witness drafts blocked on artifact/kernel-owned metadata; do not accept caller-synthesized acceptance metadata. |
| PROOF-CLOSEOUT-G009 | `deferred` | The witness store hashes exact payload bytes and schema identity, but byte-level canonicality validation remains producer-owned until concrete payload schemas expose validators. | Keep the hash/attestation contract stable; add validators only in producer-owned schema tasks. |

## Human Review Surface

- Canonical English docs under `doc/design/mizar-proof/en/`.
- Japanese companions under `doc/design/mizar-proof/ja/`.
- Proof source under `crates/mizar-proof/src/`.
- Proof tests under `crates/mizar-proof/tests/` and private module tests under
  `crates/mizar-proof/src/<module>/tests.rs`.
- Upstream/downstream context:
  `doc/design/mizar-kernel/en/crate_exit_report.md`,
  `doc/design/mizar-vc/en/crate_exit_report.md`,
  `doc/design/mizar-atp/en/crate_exit_report.md`,
  `doc/design/mizar-artifact/en/todo.md`,
  `doc/design/internal/en/04.atp_portfolio_and_kernel_check_integration.md`,
  `doc/design/internal/en/07.crate_module_layout.md`,
  `doc/design/architecture/en/08.reasoning_boundary.md`,
  `doc/design/architecture/en/15.kernel_certificate_format.md`,
  `doc/design/architecture/en/19.failure_semantics.md`, and
  `doc/design/architecture/en/22.incremental_verification_contract.md`.

## Test Expectation Summary

No existing `.miz` fixtures or expectation sidecars were changed to match
implementation behavior. Milestone-owned behavior is covered by Rust unit
tests, integration tests, lint-policy guards, determinism tests, source/spec
audits, architecture-22 audit, module-boundary audit, or explicit deferred gap
records. Source-derived semantic corpus coverage remains blocked by producer
and runner gaps listed above.

## Verification Commands

| Command | Result |
|---|---|
| `cargo fmt --check` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test -p mizar-proof` | passed |
| `cargo test -p mizar-kernel` | passed |
| `cargo test -p mizar-vc` | passed |
| `cargo test -p mizar-artifact` | passed |
| `cargo test -p mizar-checker` | passed |
| `cargo test -p mizar-atp` | passed after focused metadata correction commit `36d1a9c` |
| `cargo test` | passed after focused metadata correction commit `36d1a9c` |
| `git diff --check` | passed |
| `git diff --cached --check` | passed after explicit task-20 path staging |

Unrun deferred commands:

- `cargo test -p mizar-cache` was not run in the original closeout because
  `mizar-cache` was not yet a workspace crate. The roadmap sync records that
  cache now exists; future proof/cache integration should run the dedicated
  cache checks in that task.

## Next-Phase Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Start the next evidence-pipeline integration phase after the mizar-proof task
20 closeout commit exists and `36d1a9c` is in HEAD history. Keep
`mizar-proof` as the proof-policy/winner/status/witness/reuse owner. A good
next task is a focused integration task, such as wiring `mizar-cache`
proof-reuse validation into an owning cache/driver path or wiring `mizar-atp`
to consume the existing early-stop policy APIs without moving policy or winner
selection into ATP. Do not add placeholder cache behavior, artifact witness
publication, kernel acceptance, SAT solving, backend proof trust, or proof
search. Run the AGENTS.md review phases and full workspace verification before
committing.
```

Rationale: downstream integration crosses proof/cache/artifact/ATP ownership
boundaries and must preserve the rule that trusted acceptance comes only from
accepted proof-obligation kernel results. Keep `xhigh`; lower only for
docs-only typo synchronization that does not change integration contracts.
