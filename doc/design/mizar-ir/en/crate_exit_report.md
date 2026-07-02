# Crate Exit Report: mizar-ir

> Canonical language: English. Japanese companion:
> [../ja/crate_exit_report.md](../ja/crate_exit_report.md).

## Result

Status: completed after task 20 dispatch-input follow-up.
Quality score: 95/100.
Score caps applied: none.

## Scope

Milestone scope:

- build the `mizar-ir` workspace crate from task 0 through task 19 and this
  closeout task;
- own compiler-internal immutable IR output storage and typed
  `PhaseOutputRef<T>` handles;
- own snapshot-scoped IR identity tables and parent/derived phase-output
  lineage while consuming `mizar-session` snapshot/source identities;
- own the phase-output publisher that seals complete outputs, validates
  current snapshots/work units, and hides unsealed, partial, or obsolete
  outputs from current publication;
- own content-addressed internal blobs, retention, collection, and snapshot
  replacement behavior for retained outputs;
- own an IR cache adapter boundary that consumes `mizar-cache` records and
  validated lookup outcomes, and rehydrates handles only after fail-closed
  validation;
- own artifact projection from sealed internal outputs into stable
  `mizar-artifact` draft schemas.

Excluded:

- proof acceptance, trusted status, verifier-policy selection, deterministic
  proof winner selection, kernel acceptance, trusted `used_axioms`, or any
  proof-authority decision;
- `mizar-cache` `CacheKey` construction, dependency-fingerprint construction,
  dependency-slice ownership, or proof-reuse validation policy;
- raw `SurfaceAst`, `TypedAst`, `CoreIr`, `ControlFlowIr`, `VcIr`,
  `AtpProblem`, kernel-internal state, storage references, or inline proof
  witness payloads in published artifacts;
- placeholder producer/diagnostics integration APIs, producer-publication
  tokens, artifact-publication tokens, or any `mizar-driver` dependency inside
  `mizar-ir`.

## Task Commits

| Task | Commit | Subject |
|---:|---|---|
| 0 | `58c515e05f1ffaaee16e080c6016254e671b30e8` | `docs(ir-task-0): add autonomous crate plan` |
| 1 | `ad26742f4a7ff11c9728defb17bd211da8911193` | `feat(ir-task-1): scaffold ir crate` |
| 2 | `992d5c66eb0726489762d78b224403f9eabb9388` | `docs(ir-task-2): specify ir identity` |
| 3 | `d395dd8399cc0285ce3a9a746ff445a543c270a5` | `feat(ir-task-3): add snapshot handle registry` |
| 4 | `fc93438743a753b39f080fe25a4690ce2b3557f0` | `docs(ir-task-4): specify ir storage` |
| 5 | `2200974e8f88245ec49e4ab31be1f70a78360e32` | `feat(ir-task-5): add sealed storage handles` |
| 6 | `2a987b3f0d8b0329422d0998457b364e5cbc7b85` | `feat(ir-task-6): add blob storage collection` |
| 7 | `585a84aa92a49d585334a78473cc6e11db4f351a` | `docs(ir-task-7): specify phase publisher` |
| 8 | `15fbb29342029d251176dafddf1d18f0b1122dbf` | `feat(ir-task-8): add phase output publisher` |
| 9 | `7149cc6cbbdf52ff858d76ab2a44c93db23722fe` | `docs(ir-task-9): specify cache adapter` |
| 10 | `8782f80b11cc1039a531e816f356d354709e8b48` | `feat(ir-task-10): add cache adapter` |
| 11 | `77d28cc0ab6de03df72b5da815b754f5f997f6a1` | `docs(ir-task-11): specify artifact projection` |
| 12 | `c96a8980d1634866f3ce3afb4a4155e4704cface` | `feat(ir-task-12): add artifact projection service` |
| 13 | `4ba76511c16630d63463d9bcd8b1f251dbaf503e` | `feat(ir-task-13): add snapshot replacement` |
| 14 | `89b36ab6a6de778dcc3d50f88848d0a8adce9492` | `test(ir-task-14): add determinism lifetime suite` |
| 15 | `c904f46ca806b1e4f18cffa26de26a2ed75c670f` | `docs(ir-task-15): record enum compatibility policy` |
| 16 | `331215b976d1896e5a4670ef8fbd89ec5ce56c2e` | `docs(ir-task-16): audit source spec correspondence` |
| 17 | `7a8d5efab3256e7fc7079cc63a367b61be01817c` | `docs(ir-task-17): audit bilingual documentation sync` |
| 18 | `ba01d4f6e6978c62e520292e314cd39ea412c89b` | `docs(ir-task-18): audit architecture 22 freshness` |
| 19 | `b0a0201bf783797ed03ca2ceeac3e500c9c322db` | `docs(ir-task-19): audit module boundaries` |
| 20 | pending self-hash | `feat(ir-task-20): add dispatch input boundary` |

## Final Owned Surfaces

| Surface | Final shape |
|---|---|
| Snapshot identity and handles | `SnapshotHandleRegistry` deterministically assigns IR-local ids from exact snapshot, source/input, phase, work-unit, producer-path, and parent identity inputs. `BuildSnapshotId` and source identity construction remain in `mizar-session`; incompatible snapshots cannot reuse lineage as the same handle family. |
| Storage | `IrStorageService` allocates pending slots, seals complete outputs, returns typed `PhaseOutputRef<T>` handles, validates erased handles through `typed_handle`, stores side tables, spills large canonical payloads to content-addressed internal blobs, and fails closed for stale, collected, unsealed, wrong-type, or foreign handles. |
| Phase output publisher | `PhaseOutputPublisher` validates current snapshots, allowed work units, slot metadata, parent handles, deterministic content hashes, and side-table hashes before sealing. Unsealed, partial, obsolete, wrong-snapshot, and superseded outputs are invisible to current publication. |
| Cache adapter | `IrCacheAdapter` consumes caller-supplied `mizar-cache` `CacheKey` values, `CacheRecord` payloads, and validated lookup outcomes. It consumes key/header/dependency/proof compatibility from `mizar-cache`, then validates adapter schema, cacheability markers, parents, payload hash, side-table hash, storage freshness, and publisher acceptance before returning a rehydrated handle; all unknown, incomplete, uncacheable, incompatible, corrupt, tampered, stale, or undecodable states are miss results before handle exposure. |
| Artifact projection | `ArtifactProjectionService` validates current sealed handles and creates unpublished `VerifiedArtifactDraft` values using stable `mizar-artifact` schemas. It rejects raw internal IR names, storage handles, kernel-internal state, inline proof witness payloads, duplicate projected rows, and schema mismatches. |
| Snapshot replacement | `replace_current_snapshot` supersedes old snapshots for current publication while retained old outputs remain readable or cache-encodable until release and collection. Obsolete outputs cannot become current results except through the validated cache-rehydration boundary for a new snapshot. |
| Dispatch input boundary | `PhaseInputIdentities`, snapshot-bound `PhaseDispatchInputBundle`, `SealedParentOutputHandle`, and `PhaseDispatchInputProvider<Task>` are owned by `mizar-ir`. Parent output identities are derived only from validated sealed handles; bundle/provider validation distinguishes missing owner inputs from invalid snapshot/storage/currentness failures. |
| Integration boundary | Real driver/front-door and diagnostics crates now exist, but diagnostics registry/rendering integration, producer projection payloads, artifact publication tokens, semantic/proof adapters, cache-compatibility wiring, LSP conversion, and system-level clean/incremental/parallel equivalence remain classified as `external_dependency_gap` or `deferred`. No placeholder producer, stub API, fake token, or `mizar-driver` dependency was added to `mizar-ir`. |

## Hard Gates

| Gate | Status | Evidence |
|---|---|---|
| Specification consistency | passed | `00.crate_plan.md`, task-scoped module specs, TODO, architecture-22 audit, source/spec audit, module-boundary gate, and closeout reviews report no unresolved blocking/high inconsistency. |
| Source behavior documented or deferred | passed | Public APIs, private canonicalization/retention/fail-closed/projection-filtering behavior, tests, and residual gaps are traced in `source_spec_correspondence.md`, `bilingual_documentation_sync.md`, `architecture_22_follow_up_audit.md`, `module_boundary_refactor_gate.md`, and this report. |
| Deterministic snapshot identity | passed | Identity specs and tests cover deterministic ids from exact inputs, duplicate/conflicting key rejection, parent lineage, incompatible snapshot rejection, and the rule that IR-local ids are not proof-reuse authority. |
| Sealed immutable outputs | passed | Storage and publisher specs/tests require only sealed handles to escape, reject pre-seal and double-seal access, validate type/generation/storage ownership, and keep partial or obsolete outputs out of current publication. |
| Fail-closed cache rehydration | passed | Cache adapter specs/tests reject cache misses, incomplete/unknown/uncacheable/incompatible records, corrupt payloads, tampered payload/side-table hashes, parent mismatch, stale/collected parents, decode errors, and publisher/storage failures before returning a `PhaseOutputRef<T>`. |
| Artifact projection boundary | passed | Projection specs/tests expose stable draft schemas only and reject raw `SurfaceAst`, `TypedAst`, `CoreIr`, `ControlFlowIr`, `VcIr`, `AtpProblem`, storage handles, kernel-internal state, and inline witness payloads. |
| Proof and cache authority boundary | passed | `mizar-ir` does not own proof acceptance, trusted status, policy selection, kernel acceptance, `CacheKey`, dependency fingerprints, or proof-reuse validation; lint guards and module tests cover this boundary. |
| Test expectation integrity | passed | No `doc/spec` language file, `.miz` fixture, traceability row, or expectation sidecar was changed to match current implementation behavior. |
| Design/source synchronization | passed | English canonical docs and Japanese companions are paired and synchronized; source/docs audits and lint guards record no current drift. |
| Downstream gaps classified | passed | Real producer, diagnostics rendering, artifact publication token, semantic/proof adapter, cache-compatibility, LSP conversion, and full system-equivalence work is classified as `external_dependency_gap` or `deferred`, not stubbed. |
| Verification | passed | Crate-local and workspace Rust verification, adjacent cache/artifact/build/driver tests, diff checks, staged diff checks, and task-20 reviews passed. Driver verification is included because `mizar-driver` consumes the IR-owned dispatch input bundle. |

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

The score deducts for downstream gaps that are intentionally outside the
milestone: real diagnostics rendering, producer-owned projection payloads,
artifact-publication-token integration, semantic/proof adapters,
cache-compatibility wiring, LSP conversion, and full clean/incremental/parallel
system equivalence. These gaps do not cap the score because they are
classified, not stubbed. Task 20 restored the score after adding the
dispatch-input boundary and passing driver front-door verification.

## Review Results

| Review | Result |
|---|---|
| Implementation specification / documentation review | No findings. The closeout scope, owned/excluded boundaries, hard gates, external gaps, quality score rationale, and handoff are complete and consistent with the crate plan and module specs. |
| Test sufficiency review | No findings. Tests cover deterministic identity, sealed storage, publisher currentness, fail-closed cache rehydration, projection leakage, snapshot replacement, lifetime collection, enum policy, dispatch input validation, provider missing/error branches, driver consumption, and boundary guards; system-level equivalence remains correctly classified as `external_dependency_gap`. |
| Full implementation review | No findings after the low documentation wording fix. The implemented source boundary owns dispatch input identities and sealed parent handles in `mizar-ir` while the driver consumes them at scheduler-selected dispatch. |
| Source/documentation consistency review | No findings. English and Japanese closeout reports are synchronized, task status and verification records match the source/tests, and source/documentation ownership statements agree. |
| Read-only crate quality review | 95/100. Hard gates pass; no unresolved blocking/high/medium findings remain. |

## Deferred And External Dependency Items

| ID | Crate-plan class | Risk tag / status | Owner / unblock condition |
|---|---|---|---|
| IR-G-004 | `design_drift` | `external_dependency_gap` | `mizar-driver` now provides the registry/front door, but real producer dispatch inputs, driver-owned cache lookup over real records, cache scheduling integration, and publication freshness wiring still require downstream integration. `mizar-ir` must continue to have no `mizar-driver` dependency. |
| IR-G-005 | `design_drift` | `external_dependency_gap` | Real `mizar-diagnostics` registry/rendering integration must be provided by the diagnostics owner. `mizar-ir` stores stable side-table/projection references only and does not add a stub diagnostics crate or API. |
| IR-G-006 | `design_drift` | `external_dependency_gap` | Real resolver/checker/core/VC/ATP/kernel/proof producer projection payloads, producer publication tokens, and artifact publication tokens must be supplied by their owning crates. Projection stays on stable draft schemas and does not mint tokens. |
| IR-G-007 | `test_gap` | `external_dependency_gap` | Full clean/incremental/parallel driver equivalence requires downstream orchestration and real producer/cache/artifact/driver seams. Crate-local deterministic and lifetime tests cover the implemented boundary until that phase exists. |
| IR-G-008 | `boundary_violation` | guarded ownership constraint | Reimplementing `mizar-cache` keys, dependency fingerprints, proof-reuse validation, proof trusted status, or kernel acceptance would be a boundary violation. Current source/spec/tests guard against it. |
| IR-G-009 | `design_drift` | resolved locally | Older internal API sketches that could assign cache-key or snapshot-identity ownership to `mizar-ir` are resolved locally by consuming `mizar-session` identities and `mizar-cache` validated records only. |

## Test Expectation Summary

No language specification, `.miz` test, coverage traceability metadata, or
expectation sidecar was changed for the `mizar-ir` milestone. The crate-owned
behavior is covered by Rust unit tests, integration tests, lint-policy guards,
the determinism/lifetime suite, source/spec audits, architecture-22 audit,
module-boundary audit, bilingual audit, and explicit gap records.

## Verification Commands

| Command | Result |
|---|---|
| `cargo test -p mizar-ir` | passed |
| `cargo clippy -p mizar-ir --all-targets -- -D warnings` | passed |
| `cargo fmt --check` | passed |
| `cargo test -p mizar-cache` | passed |
| `cargo test -p mizar-artifact` | passed |
| `cargo test -p mizar-build` | passed |
| `cargo test -p mizar-driver` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test` | passed |
| `git diff --check` | passed |
| `git diff --cached --check` | passed |

Task 20 superseded the earlier no-driver condition: `mizar-driver` now exists
and was verified because it consumes the IR-owned dispatch input bundle. The
staged diff check passed immediately before the task-20 commit.

## Human Review Surface

Primary human review should inspect:

- [00.crate_plan.md](./00.crate_plan.md)
- [todo.md](./todo.md)
- [identity.md](./identity.md)
- [storage.md](./storage.md)
- [publisher.md](./publisher.md)
- [cache_adapter.md](./cache_adapter.md)
- [projection.md](./projection.md)
- [source_spec_correspondence.md](./source_spec_correspondence.md)
- [bilingual_documentation_sync.md](./bilingual_documentation_sync.md)
- [architecture_22_follow_up_audit.md](./architecture_22_follow_up_audit.md)
- [module_boundary_refactor_gate.md](./module_boundary_refactor_gate.md)
- `crates/mizar-ir/src/`
- `crates/mizar-ir/tests/`

## Next-Phase Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Start the next integration phase after the mizar-ir closeout commit exists.
Keep `mizar-ir` as the owner of compiler-internal IR storage, deterministic
snapshot-scoped handles, sealed typed `PhaseOutputRef<T>` values, phase output
publication, cache-adapter rehydration boundaries, artifact projection drafts,
and snapshot replacement. Do not move proof acceptance, trusted status,
verifier-policy selection, kernel acceptance, `mizar-cache` `CacheKey`
construction, dependency fingerprints, or proof-reuse validation into
`mizar-ir`.

The best next task is a real downstream integration phase: producer dispatch
inputs through the driver front door, driver-owned cache lookup over real
records, cache scheduling integration, diagnostics registry integration,
producer projection payloads, or artifact publication tokens. Preserve the
existing `external_dependency_gap` classifications until the owning crate
provides the real seam. Do not add placeholder crates, stub APIs, fake
publication tokens, or a `mizar-driver` dependency to `mizar-ir`.

Cache hit rehydration remains optimization-only: incomplete, unknown,
uncacheable, incompatible, corrupt, stale, or unvalidated records must miss
before any handle is reconstructed, and a rehydrated handle never upgrades
proof or trusted status.
```

Raise reasoning above `xhigh` only for simultaneous driver/cache/artifact/proof
integration with broad API migration. Lower to `high` for a narrow docs-only
follow-up, such as backfilling a committed task hash or updating one paired
documentation row.
