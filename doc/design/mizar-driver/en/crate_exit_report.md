# Crate exit report: mizar-driver

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

Status: completed by task D-022.

## Result

Status: complete.
Quality score: 94/100.
Score caps applied: none. No hard gate failed, no blocking/high finding remains,
no `source_undocumented_behavior` or `test_expectation_drift` remains, required
verification passes, and no unapproved semantic behavior change is present.

## Scope

This report closes the autonomous `mizar-driver` task stream. It records the
completed task commits, hard-gate evidence, quality score, remaining classified
gaps, final ownership shape, verification, and next-phase handoff.

Milestone scope: make `mizar-driver` the orchestration owner for build requests,
build sessions, phase service registration, driver-owned query boundaries,
scheduler submission, protocol-agnostic build events, CLI batch entry, and
watch-mode orchestration.

Included:

- paired EN/JA crate plan, task TODOs, owner-boundary specs, audits, and this
  exit report;
- Rust source and tests for request/session, registry, driver core, events,
  CLI, watch orchestration, determinism, and module-boundary guards;
- integration seams that consume existing `mizar-build`, `mizar-ir`,
  `mizar-diagnostics`, and adjacent owner APIs without taking their authority.

Excluded:

- phase semantics, type checking, name resolution, proof acceptance, trusted
  status, kernel acceptance, cache compatibility decisions, artifact
  serialization, artifact publication tokens, real producer payload ownership,
  LSP protocol conversion, and owner seams that are still unavailable;
- repair of report-only repository metadata conflicts outside `mizar-driver`,
  including the missing `mizar-artifact` closeout report;
- `.miz` language tests, language specification files, coverage traceability
  metadata, and expectation sidecars, because this milestone does not alter
  language behavior.

## Task Commits

| Task | Commit | Summary |
|---|---|---|
| D-000 | `b359f54` | add autonomous crate plan |
| D-001 | `c44fc47` | scaffold `mizar-driver` crate |
| D-002 | `0e6cead` | specify request sessions |
| D-003 | `34ba74a` | implement request sessions |
| D-004 | `02715b8` | specify phase registry |
| D-005 | `1083b02` | implement phase registry |
| D-006 | `d043db6` | classify frontend adapter gap |
| D-007 | `0ce3356` | specify driver core |
| D-008 | `9dce878` | implement driver core |
| D-009 | `b532e82` | specify build events |
| D-010 | `5702ab5` | implement build events |
| D-011 | `9ebe2b4` | complete cancellation flow |
| D-012 | `54fac26` | specify CLI surface |
| D-013 | `e817f5e` | add CLI batch entry point |
| D-014 | `cc5560d` | add watch orchestration |
| D-015 | `f4914a2` | classify phase adapter readiness |
| D-016 | `b3fafeb` | add determinism suite |
| D-017 | `cdbc4a1` | document enum compatibility policy |
| D-018 | `8b7b59e` | add source/spec audit |
| D-019 | `b70fab1` | audit bilingual driver docs |
| D-020 | `e1d71e9` | audit architecture-22 follow-up |
| D-021 | `d0045c5` | split driver helper modules |

The D-022 closeout commit is reported in the final handoff after this file is
committed.

## Final Shape

`mizar-driver` now owns:

- `BuildRequest` / `BuildSession` snapshot-boundary envelopes, lanes,
  generations, lifecycle transitions, and obsolete-publication decisions;
- deterministic `PhaseService` registration, duplicate rejection, requirement
  classification, and the driver-owned salsa/query-compatible boundary;
- `CompilerDriver` submission, cancellation, scheduler authority consumption,
  missing service classification, dispatch-gap blocking, and stored session
  event replay;
- protocol-agnostic `BuildEventStream` records with deterministic ordering,
  currentness checks, stale-output suppression, diagnostics owner records, and
  artifact-boundary readiness events without publication authority;
- library-level `mizar build` batch entry points, stable human/JSON rendering,
  exit-code mapping, and explicit owner-gap output;
- watch-mode orchestration over owner-provided changed paths/snapshot inputs,
  superseded replay suppression, and optional real `mizar-ir`
  `PhaseOutputPublisher` snapshot replacement;
- private helper modules for CLI output rendering, driver event/scheduler/watch
  helpers, and registry phase catalog/fingerprinting.

`mizar-driver` does not own:

- phase semantics, type checking, name resolution, overload resolution, VC
  generation, ATP solving, kernel acceptance, proof acceptance, trusted status,
  proof reuse, cache compatibility decisions, artifact serialization,
  artifact publication tokens, real producer output payloads, or LSP protocol
  conversion.

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| No blocking/high specification inconsistency remains | Pass | D-018 source/spec audit, D-019 bilingual audit, D-020 architecture-22 audit, D-021 module-boundary gate, and final reviews report no blocking/high findings. |
| Public driver behavior is documented and traced | Pass | `source_spec_correspondence.md` traces public APIs and promised behavior to source/tests or classified gaps. |
| Source behavior matches design/spec/test intent | Pass | `cargo test -p mizar-driver`, lint-policy guards, source/spec audit, and full implementation reviews passed. |
| No `.miz` tests/spec metadata changed merely to match implementation | Pass | This stream changed Rust driver code/tests and design docs only. No `.miz`, expectation, traceability, or language spec files were edited. |
| Driver ownership boundaries preserved | Pass | Source guards and module specs keep phase semantics, proof/cache/artifact authority, diagnostics identity, and LSP conversion outside the driver. |
| Required verification passes | Pass | See verification table below. |
| Remaining risks are classified | Pass | See remaining classified gaps below. |

## Verification

Final D-021/D-022 closeout verification:

| Command | Status |
|---|---|
| `cargo fmt --check` | Passed |
| `cargo test -p mizar-driver` | Passed |
| `cargo clippy -p mizar-driver --all-targets -- -D warnings` | Passed |
| `cargo test -p mizar-build` | Passed |
| `cargo test -p mizar-ir` | Passed |
| `cargo test -p mizar-diagnostics` | Passed |
| `cargo test -p mizar-frontend` | Passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | Passed |
| `cargo test` | Passed |
| `git diff --check` | Passed |
| `git diff --cached --check` | Passed after staging the D-022 task paths |

The package-specific adjacent-crate tests above are also covered by the final
workspace `cargo test`, but they are recorded separately because the crate plan
lists them as closeout commands for orchestration/integration seam confidence.

## Quality Score

Quality score: **94/100**.

Rationale:

- hard gates pass;
- public driver APIs and promised behavior are documented in paired EN/JA docs;
- implemented seams have targeted Rust tests plus determinism coverage;
- source guards cover public enum compatibility, dependency boundaries, source
  surface, private helper visibility, and non-owner authority terms;
- full workspace clippy and tests pass;
- remaining risks are intentionally classified owner-seam gaps rather than fake
  implementations.

Residual score deductions:

- full real clean/incremental/parallel equivalence remains deferred until real
  producer/cache/artifact/proof seams exist;
- semantic/proof/artifact phase adapters remain unavailable owner seams;
- `mizar-artifact` closeout metadata remains a report-only repository metadata
  conflict;
- no real LSP bridge is wired yet.

The score is invalid if any hard gate above later fails.

## Score Breakdown

| Category | Points |
|---|---:|
| Specification completeness | 19/20 |
| Test contract and coverage | 18/20 |
| Traceability | 14/15 |
| Implementation correctness | 14/15 |
| Design/source synchronization | 10/10 |
| Boundary discipline | 10/10 |
| Verification health | 5/5 |
| Handoff quality | 4/5 |
| Total | 94/100 |

## Remaining Classified Gaps

| Gap | Classification | Closeout status |
|---|---|---|
| `DRIVER-G-001` | `repo_metadata_conflict` | `mizar-artifact` closeout report is absent. Report only; not repaired in this stream. |
| `DRIVER-G-005` | `external_dependency_gap` | Artifact publication token and full phase-15 producer emission remain unavailable. |
| `DRIVER-G-007` | `deferred` | Full real clean/incremental/parallel equivalence requires real cache, producer, artifact, proof, worker-race, and multi-task dispatch seams. |
| `DRIVER-G-009` | `repo_metadata_conflict` | Existing artifact metadata drift around `mizar-proof` remains report-only. |
| `DRIVER-G-010` | `external_dependency_gap` | Frontend canonical producer payload and diagnostics bridge readiness remain unavailable. |
| `DRIVER-G-011` | `external_dependency_gap` | Scheduler-selected real phase dispatch callback is not exposed by `mizar-build`. |
| `DRIVER-G-012` | `external_dependency_gap` / `deferred` | Real file watcher/coalescing owner and LSP build bridge remain outside the driver. |
| `DRIVER-G-013` | `external_dependency_gap` | Semantic/proof/artifact phase adapters lack complete owner-provided driver-callable seams. |
| `DRIVER-G-014` | `deferred` | Documentation extraction waits for a `mizar-doc` owner crate/surface. |

## Human Review Surface

The human reviewer should primarily inspect:

- `doc/design/mizar-driver/en/` and `doc/design/mizar-driver/ja/`, especially
  `00.crate_plan.md`, `todo.md`, `request.md`, `registry.md`, `driver.md`,
  `events.md`, `cli.md`, audit reports, and this exit report;
- `crates/mizar-driver/src/` public modules and private helper modules;
- `crates/mizar-driver/tests/`, especially the lint/source-boundary guards and
  determinism coverage;
- the closeout reports for `mizar-build`, `mizar-ir`, `mizar-cache`, and
  `mizar-diagnostics`, plus the report-only absence of a `mizar-artifact`
  closeout report.

No `doc/spec`, `.miz` test, test expectation, or spec-trace metadata changes are
included in the human review surface for this milestone.

## Test Expectation Summary

| Test surface | Intent | Expected outcome | Expected phase | Diagnostics | Spec refs |
|---|---|---|---|---|---|
| `crates/mizar-driver` unit tests | Request/session, registry, driver, event, CLI, and watch behavior matches documented orchestration contracts. | Pass. | Driver orchestration only. | Uses driver events and owner readiness records; does not invent diagnostic identity. | Spec 23 plus `doc/design/mizar-driver/en/*.md`. |
| `crates/mizar-driver/tests/lint_policy.rs` | Keep dependency, public enum, source-surface, private-helper, and non-owner boundary rules enforced. | Pass. | Source-boundary guard. | None; structural source checks only. | Crate plan, module layout, and driver design docs. |
| Determinism tests | Stable replay/rendering for clean/incremental/parallel projections over implemented seams. | Pass for crate-local implemented seams; real full equivalence remains deferred. | Driver event/CLI projection. | Owner records are preserved; message text is presentation only. | Architecture 20/22 and driver event docs. |
| Workspace verification | Adjacent owner crates still compile and test with the driver integration surface. | Pass. | Cross-crate integration confidence. | Diagnostics authority remains in `mizar-diagnostics`. | Internal 01, architecture 22, and closeout reports for adjacent crates. |
| `.miz` / expectation corpus | No language behavior was changed by this stream. | Not changed. | Not applicable. | Not changed. | Existing `doc/spec/en/` and test corpus remain authoritative. |

## Reviews

The autonomous workflow used review-only agents for specification/docs,
test sufficiency, full implementation, source-doc consistency, and
source-boundary/quality checks across the task stream. Blocking/high/medium
findings were fixed before their task commits. The final D-022 closeout review
found missing protocol-template fields; this report fixes them. The repeated
D-022 review must report no blocking/high findings for this closeout score to
stand.

## Next-Phase Handoff

Recommended reasoning: **high**.

Rationale: the next phase is likely an owner-seam integration task. It needs
careful boundary reasoning and source/test review, but it is narrower than the
crate-wide autonomous buildout that required xhigh.

Raise to **xhigh** if the next task attempts real semantic/proof/artifact phase
adapter integration, cache/proof reuse validation, artifact publication, or LSP
bridge orchestration. Lower to **medium** for a docs-only follow-up or a narrow
test-only source guard.

Suggested prompt:

```text
Continue from the completed mizar-driver closeout. Pick one remaining
classified owner seam, preferably DRIVER-G-011 scheduler-selected real phase
dispatch or DRIVER-G-013 one real phase adapter whose owner crate now exposes a
complete driver-callable seam. Follow AGENTS.md. Do not create fake adapters,
provisional publication tokens, stub producer outputs, cache/proof authority,
artifact serialization, or LSP protocol conversion in mizar-driver. Update
paired EN/JA docs, add focused tests, run required verification, and commit one
task-sized change.
```
