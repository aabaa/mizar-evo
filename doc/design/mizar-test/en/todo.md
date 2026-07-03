# mizar-test TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Unlike the pipeline crates, the module specs of this crate already exist;
tasks below implement against them and close gaps. The crate refines
[architecture/en/20.test_strategy.md](../../architecture/en/20.test_strategy.md)
per [internal 07](../../internal/en/07.crate_module_layout.md).

| Module | Spec | Source | Status |
|---|---|---|---|
| layout | [layout.md](./layout.md) | `src/layout.rs`, `src/path_rules.rs` | [~] discovery/pairing and validation-mode unknown-root policy implemented; public API sync pending |
| expectation_schema | [expectation_schema.md](./expectation_schema.md) | `src/expectation.rs` | [~] core schema, profile/provenance metadata retention, and fail/soundness rejection gates implemented; general snapshot hardening pending |
| staged_model | [staged_model.md](./staged_model.md) | `src/staged_model.rs` | [~] stage ids and declared prerequisite validation implemented; richer admission policy pending |
| traceability | [traceability.md](./traceability.md) | `src/traceability.rs` | [~] syntax/backrefs, coverage report/status gates, manifest ordering, obsolete-ref checks, prerequisite credit gates, and architecture-22 matrix summary implemented |
| harness | [harness.md](./harness.md) | `src/harness.rs`, `src/main.rs`, `src/runner.rs` | [~] metadata plan, validation-mode CLI, profile filtering, coverage/pass-fail/matrix report, and active parse/declaration/type runners |
| miz_corpus | [miz_corpus.md](./miz_corpus.md) | corpus tree under `tests/` | [~] roots discovered, pass/fail mix reported, provenance/profile policy rules validated; future corpus classes pending |
| snapshot | [snapshot.md](./snapshot.md) | `src/snapshot.rs`, `src/expectation.rs`, `src/runner.rs` | [~] general snapshot record API/hash/update/determinism helpers implemented; sidecar/runner integration pending |
| fail_soundness | [fail_soundness.md](./fail_soundness.md) | `src/expectation.rs`, `src/harness.rs`, future runner cases | [~] metadata contract gates implemented; active proof/certificate/kernel execution paced by future runners |
| minimal_crate | [minimal_crate.md](./minimal_crate.md) | crate boundary + CLI | [~] metadata plan, validation modes, CLI fixtures, coverage gates, and prerequisite gates implemented |

`mizar-test` is the corpus and harness crate: test discovery, `.expect.toml`
expectation parsing, the staged model, spec-coverage traceability, snapshot
comparison, and the fail/soundness contract. It is deliberately minimal
([minimal_crate.md](./minimal_crate.md)): the metadata `plan` mode owns
validation and planning without executing payloads, while explicit active
runner subcommands may depend on the narrow pipeline seams needed for their
stage. The parse-only runner location was settled by `mizar-parser` task 3;
the declaration-symbol runner follows the same active-subcommand model for
`mizar-resolve` task 23.

Each task below is deliberately small — one behavior slice against an
existing spec — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The crate keeps its dependency set minimal per
[minimal_crate.md](./minimal_crate.md). Its metadata APIs remain payload-free;
active runner subcommands add only the pipeline dependencies needed by the
stage they execute. Corpus and coverage growth is paced by the consumer-crate
runner tasks (`mizar-parser` task 3, `mizar-resolve` task 23,
`mizar-checker` tasks 12/29, `mizar-vc` task 15, `mizar-atp` task 20,
`mizar-kernel` task 17).

## Resolved And Open Decisions

- **No pipeline dependencies: resolved by [minimal_crate.md](./minimal_crate.md).**
  The metadata `plan` path has no payload execution. Explicit active runner
  subcommands may depend on the narrow pipeline seams they exercise; those
  dependencies are not used by metadata validation.
- **Corpus runner location: owned by `mizar-parser` task 3** (and the
  corresponding tasks of later stages); `mizar-resolve` task 23 extends this
  precedent with the declaration-symbol runner in `mizar-test`.
- **Snapshot update mechanism: open, resolved by task 5.** Decide how
  baselines are (re)generated — explicit update mode versus environment
  flag — within the update policy of [snapshot.md](./snapshot.md), and
  record the decision there.

## Task 2 Audit Baseline

Task 2 recorded the crate-wide source/spec audit in
[00.crate_plan.md](./00.crate_plan.md). The audit did not identify a blocking
`spec_gap`, accepted `repo_metadata_conflict`, or required language behavior
change. The prior trace manifest ordering conflict was repaired by
`897d549`; task 6 added the manifest-order validator and regression test.

Follow-up ownership from the audit:

- `layout`: sync the documented Public API with `DiscoveredLayout` and the
  harness-owned `TestCase`; keep unknown-root policy covered as new roots land.
- `expectation_schema`: validate generated origin tables, certificate/kernel
  `rejection_reason`, diagnostic ordering, and the future general
  `[[snapshots]]` hash registry.
- `traceability`: keep coverage/status reporting synchronized as new evidence
  kinds land. Manifest order validation, mode-aware coverage/status
  computation, obsolete-reference checks, declared prerequisite gates, and
  existing link-validator error fixtures are implemented.
- `harness`: keep runner-specific report docs synchronized with exported APIs
  as later generic outcome/reporting surfaces land.
- `miz_corpus`: add enforceable generated/fuzz/stress metadata,
  corpus-policy profile constraints, and stress exclusion checks. Corpus-wide
  pass/fail mix reporting is implemented.
- `snapshot`: implement the general snapshot module, canonical hashing,
  explicit update flow, and determinism checks beyond the transitional
  parse-only `SurfaceAst` baseline path.
- `fail_soundness`: task 8 implements fail/soundness metadata bookkeeping,
  case-level required checks, false-arithmetic stable-key gating, and
  weakening/deletion diagnostics. Active proof/certificate/kernel execution
  remains paced by future consumer runners.

## Ordered Task List

Keep `cargo test -p mizar-test` green after each task (see
[Recommended Verification](#recommended-verification)).

### Foundation

1. **Lint-policy guard.** [x]
   - Add `tests/lint_policy.rs` mirroring the `mizar-frontend` guard
     (workspace lint opt-in, deny baseline, rationale next to any `allow`).
   - Tests: lint-policy guard passes.
   - Deps: none. Spec: repository conventions.

2. **Source/spec gap audit and status sync.** [x]
   - Trace every Public API and Tests promise of the nine module specs to
     the current implementation; record gaps as follow-up tasks in this
     TODO and set the module-table statuses accordingly.
   - Audit record: [00.crate_plan.md](./00.crate_plan.md) "Known Gaps And
     Drift" and [Task 2 Audit Baseline](#task-2-audit-baseline).
   - Deps: 1. Spec: all module specs.

3. **Runner modes and CLI completion.** [x]
   - Complete the CLI beyond `plan` per
     [minimal_crate.md](./minimal_crate.md) "CLI"/"Exit Codes" and
     [harness.md](./harness.md) "Runner Modes": validation mode over the
     corpus tree and coverage manifest with documented exit codes.
   - Close task-2 gaps for `ValidationMode` use, strict/permissive
     unknown-root policy, plan-mode CLI output/exit-code fixtures, and the
     documented/public reporting API shape.
   - Retain optional sidecar metadata that is currently type-checked and
     discarded (`profiles`, `notes`, `ast_profile`, `snapshot_profiles`) and
     apply profile filtering to plan construction.
   - Reconcile the `parser.type_fixtures` import-summary exception with
     [harness.md](./harness.md): document it explicitly or remove the fixture
     symbol injection.
   - Add focused expectation-schema regression fixtures for unsupported schema
     versions, id/source-stem mismatches, invalid enum/outcome pairs, and
     duplicate sidecar `spec_refs`.
   - Tests: CLI fixtures per mode; exit codes match the spec table;
     deterministic output.
   - Deps: 2. Spec: `minimal_crate.md`, `harness.md`.

### Snapshot support

4. **Snapshot module: API and canonicalization.** [x]
   - Add `src/snapshot.rs` implementing the snapshot kinds, public API, and
     canonicalization rules of [snapshot.md](./snapshot.md) (stable paths,
     normalized line endings, no nondeterministic fields).
   - Tests: canonicalization fixtures; comparison failures carry precise
     diffs.
   - Deps: 2. Spec: [snapshot.md](./snapshot.md) "Public
     API"/"Canonicalization".

5. **Snapshot update policy and determinism checks.** [x]
   - Implement the baseline update flow (resolving the update-mechanism
     decision) and the determinism checks of
     [snapshot.md](./snapshot.md) (repeat-render comparison).
   - Tests: update flow round-trips; accidental-update protection;
     determinism check catches injected nondeterminism.
   - Deps: 4. Spec: [snapshot.md](./snapshot.md) "Update
     Policy"/"Determinism Checks".

### Coverage and soundness contracts

6. **Coverage and pass/fail-mix reporting.** [x]
   - Report spec-trace coverage per stage and the corpus pass/fail mix
     against the 40/60 target of the test strategy, from the existing
     traceability and discovery data.
   - Close task-2 traceability gaps for coverage-shape computation,
     manifest stored-status comparison, manifest order validation, obsolete
     references, missing manifest source files, missing listed tests, and
     existing link-validator error-path tests, including duplicate manifest
     test paths, missing backrefs, unparsed listed tests, deferred required
     reasons, and planned-without-tests warnings.
   - Tests: report fixtures over synthetic corpora; deterministic report
     bytes.
   - Deps: 3. Spec: [traceability.md](./traceability.md),
     [architecture/en/20.test_strategy.md](../../architecture/en/20.test_strategy.md).

7. **Stage-prerequisite validation.** [x]
   - Enforce the staged-model rules: a case's stage prerequisites must be
     covered or declared built-in before coverage credit is granted.
   - Close task-2 gaps for `depends_on` handling, built-in declarations,
     stage mismatch diagnostics, and higher-stage coverage not being credited
     before prerequisites are satisfied.
   - Tests: prerequisite-violation fixtures fail validation with stable
     diagnostics.
   - Deps: 6. Spec: [staged_model.md](./staged_model.md) "Stage Rules".

8. **Fail/soundness contract support.** [x]
   - Implement the expected-failure contract of
     [fail_soundness.md](./fail_soundness.md): required-case bookkeeping
     per domain, expected-failure assertions (diagnostic code and stage),
     and the regression rule that soundness cases never get deleted or
     weakened silently.
   - Close task-2 gaps for certificate/kernel `rejection_reason`, typed fail
     identity or equivalent validation, false-arithmetic coverage, and
     domain-required case bookkeeping.
   - Tests: contract fixtures; weakening attempts flagged.
   - Completed: certificate/kernel `rejection_reason` validation, recognized
     `soundness.*` case shape/profile/phase gates, mode-aware missing-case
     diagnostics, and false-arithmetic stable-key gating. Real
     proof/certificate/kernel execution is not fabricated before the owning
     consumer runners exist.
   - Deps: 6. Spec: [fail_soundness.md](./fail_soundness.md).

9. **Corpus size and review-rule validation.** [x]
   - Validate the corpus-growth rules of [miz_corpus.md](./miz_corpus.md):
     file-size guidelines, naming, corpus-class placement, and
     generation-policy markers.
   - Close task-2 gaps for generated/fuzz/property origin metadata,
     reproducibility metadata, optional metadata retention that belongs to
     corpus policy, corpus-policy profile constraints, stress exclusion, and
     fuzz-category preservation.
   - Tests: violation fixtures per rule; clean corpus passes.
   - Completion: task 9 implements `[origin]` provenance parsing/retention,
     corpus placement/profile gates, stress exclusion, fuzz-category
     preservation, upper-bound `.miz` size diagnostics, naming diagnostics, and
     metadata fixtures for clean and violating corpora.
   - Deps: 3. Spec: [miz_corpus.md](./miz_corpus.md).

### Consumer pacing and follow-ups

10. **Consumer-runner support.** [ ] — paced by consumer crates.
    - Keep discovery, expectations, stages, snapshot, and reporting in step
      with each consumer runner as it lands (`mizar-parser` task 3,
      `mizar-resolve` task 23, `mizar-checker` tasks 12/29, `mizar-vc`
      task 15, `mizar-atp` task 20, `mizar-kernel` task 17); one increment
      per consumer, in its own change. Checked off when the last runner
      lands.
    - Support explicit active/planned gating for consumer runners when
      traceability seed cases are committed before the owning pipeline stage
      can execute them. The default metadata plan may discover such cases, but
      a consumer runner must not silently count a planned seed as executed
      coverage.
    - R-023 paired work adds the `declaration-symbol` active runner command for
      `mizar-resolve` task 23, including active-tag validation, public-code
      gating while resolver diagnostic ranges are unspecified, summary
      reporting, and two traceable seed fixtures. This task stays open until
      all planned consumer runners land.
    - Current task-10 ledger records `mizar-parser` task 3 (`parse-only`),
      `mizar-resolve` task 23 (`declaration-symbol`), and `mizar-checker` task
      12 (`type-elaboration`) as prepared/implemented increments. Checker task
      29, `mizar-vc` task 15, `mizar-atp` task 20, and `mizar-kernel` task 17
      are recorded as `paced/open`; no placeholder runner or fake active
      fixture is created for them.
    - Deps: 5, 8. Spec: [harness.md](./harness.md).

11. **Determinism suite.** [x]
    - Property coverage that discovery order, plans, validation
      diagnostics, reports, and snapshot comparisons are byte-stable across
      runs and platforms.
    - Close task-2 gaps for general snapshot hash determinism,
      parallel-equivalence modes, and nondeterminism diagnostics outside the
      transitional parse-only `SurfaceAst` path.
    - Completion: task 11 adds canonical-byte stability tests for metadata
      plans and active runner reports, generic snapshot nondeterminism
      diagnostics outside `SurfaceAst`, and snapshot-level
      `verify_snapshot_parallel_equivalence`.
    - Deps: 6. Spec: [harness.md](./harness.md) "Determinism Requirements".

12. **Public-enum forward-compatibility policy.** [x]
    - Apply the `mizar-frontend` task-25 procedure to each public enum
      (`Stage`, `ExpectedOutcome`, `ValidationSeverity`, …); record
      decisions in the owning module specs.
    - Completion: all public enums in `crates/mizar-test/src` are
      downstream `#[non_exhaustive]`, owning EN/JA module specs record the
      inventory and decision, and lint coverage guards source attributes plus
      EN/JA inventory entries.
    - Deps: 2. Spec: all module specs.

13. **Bilingual documentation sync audit.** [x]
    - Compare each English canonical document under
      `doc/design/mizar-test/en/` with its Japanese companion and
      synchronize content.
    - Completion: [bilingual_sync_audit.md](./bilingual_sync_audit.md)
      records the task-13 paired-file audit; task 14 completion is recorded
      below.
    - Deps: 12. Spec: repository documentation policy.

14. **Incremental/parallel verification regression matrix.** [x]
    - Add corpus/harness metadata and reporting support for the architecture-22
      regression matrix, while keeping this crate pipeline-free. Consumer
      crates execute the cases, but `mizar-test` owns the scenario ids,
      expected equivalence classes, active/planned gating, and traceability
      records.
    - Matrix rows must cover: clean sequential == clean parallel; clean build
      == incremental build for externally visible artifacts; sequential
      incremental == parallel incremental; randomized ready-task scheduling;
      randomized ATP backend completion order; cache hit/miss timing;
      `VcId` reordering with reuse only on matching `ObligationAnchor`,
      fingerprints, policy, and witness/discharge hashes; missing dependency
      slice forcing cache miss; stale snapshot diagnostics and obsolete-result
      non-publication; proof witness mismatch; externally attested evidence
      non-upgrade; cache-key races; artifact manifest atomicity; registration
      and cluster invalidation; theorem proof-body and theorem-status
      invalidation; notation/operator invalidation.
    - Deps: 10, 11. Spec:
      [20.test_strategy.md](../../architecture/en/20.test_strategy.md),
      [22.incremental_verification_contract.md](../../architecture/en/22.incremental_verification_contract.md).
    - Completion: task 14 adds the architecture-22 scenario registry,
      sidecar metadata validation, deterministic plan/report summary, and the
      metadata-only `tests/property/architecture22_matrix_001` anchor covering
      all 18 required scenario ids as `planned`. All rows remain inactive
      because no scenario-specific clean/incremental/parallel/cache-race
      consumer runner is prepared; `active` gates are rejected rather than
      fabricating execution.

15. **Architecture-22 follow-up audit.** [x]
    - Re-run the source/spec gap and bilingual documentation sync audits, and
      review the task-14 scenario ids, equivalence classes, active/planned
      gating, and traceability records against architecture 22; record any
      remaining matrix gaps as follow-up tasks.
    - Completion: task 15 updates
      [bilingual_sync_audit.md](./bilingual_sync_audit.md) and
      [00.crate_plan.md](./00.crate_plan.md) with the post-task-14 audit.
      The 18 scenario ids/classes and the metadata-only trace anchor match
      architecture 20/22; every row remains `planned` because no prepared
      consumer runner increment was newly confirmed. Remaining active matrix
      execution is recorded as MT-AUDIT-014, a consumer-paced `test_gap`. No
      `spec_gap`, `repo_metadata_conflict`, language behavior change, or
      existing expectation semantic change is required.
    - Deps: 14. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md),
      [22.incremental_verification_contract.md](../../architecture/en/22.incremental_verification_contract.md),
      repository documentation policy.

## Recommended Verification

Run after each task:

```text
cargo fmt --check
cargo test -p mizar-test
cargo clippy -p mizar-test --all-targets -- -D warnings
```

For tasks that change discovery, expectations, or stages, also run the
consumers that embed corpus runners (currently):

```text
cargo test -p mizar-frontend
cargo test -p mizar-resolve
```

For the architecture-22 regression matrix, also run the active consumer
crates for the rows being added:

```text
cargo test -p mizar-build
cargo test -p mizar-driver
cargo test -p mizar-cache
cargo test -p mizar-vc
cargo test -p mizar-atp
cargo test -p mizar-proof
```

Check the task off here once tests pass.

## Notes

- This crate stays minimal: metadata validation, planning, comparison, and
  reporting stay payload-free. Explicit active runner subcommands are the only
  paths that execute pipeline seams, and those seams are scoped to the stage
  being run.
- Stage ids are canonical values shared with `.expect.toml`,
  `spec_trace.toml`, and consumer enums; display names may localize, ids
  may not.
- Fail/soundness coverage takes priority near the kernel; the 40/60
  pass/fail mix is a corpus-wide target, not per-directory.
- Snapshot baselines are the stability surface for internal renderings;
  the renderings themselves are not stable artifacts.
