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
| layout | [layout.md](./layout.md) | `src/layout.rs`, `src/path_rules.rs` | [~] |
| expectation_schema | [expectation_schema.md](./expectation_schema.md) | `src/expectation.rs` | [~] |
| staged_model | [staged_model.md](./staged_model.md) | `src/staged_model.rs` | [~] |
| traceability | [traceability.md](./traceability.md) | `src/traceability.rs` | [~] |
| harness | [harness.md](./harness.md) | `src/harness.rs`, `src/main.rs` | [~] discovery + `plan` mode |
| miz_corpus | [miz_corpus.md](./miz_corpus.md) | corpus tree under `tests/` | [~] |
| snapshot | [snapshot.md](./snapshot.md) | `src/snapshot.rs` | [ ] |
| fail_soundness | [fail_soundness.md](./fail_soundness.md) | harness rules + corpus cases | [ ] |
| minimal_crate | [minimal_crate.md](./minimal_crate.md) | crate boundary + CLI | [~] |

`mizar-test` is the corpus and harness crate: test discovery, `.expect.toml`
expectation parsing, the staged model, spec-coverage traceability, snapshot
comparison, and the fail/soundness contract. It is deliberately minimal
([minimal_crate.md](./minimal_crate.md)): it owns validation and planning,
not pipeline execution — stage runners live in consumer crates' integration
tests (precedent: the frontend lexical-corpus runner; `mizar-parser` task 3
decides the parse-only runner location).

Each task below is deliberately small — one behavior slice against an
existing spec — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The crate keeps its dependency set minimal per
[minimal_crate.md](./minimal_crate.md); it does not depend on pipeline
crates, and consumer crates depend on it for discovery, expectations, and
validation. Corpus and coverage growth is paced by the consumer-crate
runner tasks (`mizar-parser` task 3, `mizar-resolve` task 23,
`mizar-checker` tasks 12/29, `mizar-vc` task 15, `mizar-atp` task 20,
`mizar-kernel` task 17).

## Resolved And Open Decisions

- **No pipeline dependencies: resolved by [minimal_crate.md](./minimal_crate.md).**
  Stage execution lives in consumer crates; this crate validates, plans,
  compares, and reports.
- **Corpus runner location: owned by `mizar-parser` task 3** (and the
  corresponding tasks of later stages); this crate provides discovery and
  expectations either way.
- **Snapshot update mechanism: open, resolved by task 5.** Decide how
  baselines are (re)generated — explicit update mode versus environment
  flag — within the update policy of [snapshot.md](./snapshot.md), and
  record the decision there.

## Ordered Task List

Keep `cargo test -p mizar-test` green after each task (see
[Recommended Verification](#recommended-verification)).

### Foundation

1. **Lint-policy guard.** [ ]
   - Add `tests/lint_policy.rs` mirroring the `mizar-frontend` guard
     (workspace lint opt-in, deny baseline, rationale next to any `allow`).
   - Tests: lint-policy guard passes.
   - Deps: none. Spec: repository conventions.

2. **Source/spec gap audit and status sync.** [ ]
   - Trace every Public API and Tests promise of the nine module specs to
     the current implementation; record gaps as follow-up tasks in this
     TODO and set the module-table statuses accordingly.
   - Deps: 1. Spec: all module specs.

3. **Runner modes and CLI completion.** [ ]
   - Complete the CLI beyond `plan` per
     [minimal_crate.md](./minimal_crate.md) "CLI"/"Exit Codes" and
     [harness.md](./harness.md) "Runner Modes": validation mode over the
     corpus tree and coverage manifest with documented exit codes.
   - Tests: CLI fixtures per mode; exit codes match the spec table;
     deterministic output.
   - Deps: 2. Spec: `minimal_crate.md`, `harness.md`.

### Snapshot support

4. **Snapshot module: API and canonicalization.** [ ]
   - Add `src/snapshot.rs` implementing the snapshot kinds, public API, and
     canonicalization rules of [snapshot.md](./snapshot.md) (stable paths,
     normalized line endings, no nondeterministic fields).
   - Tests: canonicalization fixtures; comparison failures carry precise
     diffs.
   - Deps: 2. Spec: [snapshot.md](./snapshot.md) "Public
     API"/"Canonicalization".

5. **Snapshot update policy and determinism checks.** [ ]
   - Implement the baseline update flow (resolving the update-mechanism
     decision) and the determinism checks of
     [snapshot.md](./snapshot.md) (repeat-render comparison).
   - Tests: update flow round-trips; accidental-update protection;
     determinism check catches injected nondeterminism.
   - Deps: 4. Spec: [snapshot.md](./snapshot.md) "Update
     Policy"/"Determinism Checks".

### Coverage and soundness contracts

6. **Coverage and pass/fail-mix reporting.** [ ]
   - Report spec-trace coverage per stage and the corpus pass/fail mix
     against the 40/60 target of the test strategy, from the existing
     traceability and discovery data.
   - Tests: report fixtures over synthetic corpora; deterministic report
     bytes.
   - Deps: 3. Spec: [traceability.md](./traceability.md),
     [architecture/en/20.test_strategy.md](../../architecture/en/20.test_strategy.md).

7. **Stage-prerequisite validation.** [ ]
   - Enforce the staged-model rules: a case's stage prerequisites must be
     covered or declared built-in before coverage credit is granted.
   - Tests: prerequisite-violation fixtures fail validation with stable
     diagnostics.
   - Deps: 6. Spec: [staged_model.md](./staged_model.md) "Stage Rules".

8. **Fail/soundness contract support.** [ ]
   - Implement the expected-failure contract of
     [fail_soundness.md](./fail_soundness.md): required-case bookkeeping
     per domain, expected-failure assertions (diagnostic code and stage),
     and the regression rule that soundness cases never get deleted or
     weakened silently.
   - Tests: contract fixtures; weakening attempts flagged.
   - Deps: 6. Spec: [fail_soundness.md](./fail_soundness.md).

9. **Corpus size and review-rule validation.** [ ]
   - Validate the corpus-growth rules of [miz_corpus.md](./miz_corpus.md):
     file-size guidelines, naming, corpus-class placement, and
     generation-policy markers.
   - Tests: violation fixtures per rule; clean corpus passes.
   - Deps: 3. Spec: [miz_corpus.md](./miz_corpus.md).

### Consumer pacing and follow-ups

10. **Consumer-runner support.** [ ] — paced by consumer crates.
    - Keep discovery, expectations, stages, snapshot, and reporting in step
      with each consumer runner as it lands (`mizar-parser` task 3,
      `mizar-resolve` task 23, `mizar-checker` tasks 12/29, `mizar-vc`
      task 15, `mizar-atp` task 20, `mizar-kernel` task 17); one increment
      per consumer, in its own change. Checked off when the last runner
      lands.
    - Deps: 5, 8. Spec: [harness.md](./harness.md).

11. **Determinism suite.** [ ]
    - Property coverage that discovery order, plans, validation
      diagnostics, reports, and snapshot comparisons are byte-stable across
      runs and platforms.
    - Deps: 6. Spec: [harness.md](./harness.md) "Determinism Requirements".

12. **Public-enum forward-compatibility policy.** [ ]
    - Apply the `mizar-frontend` task-25 procedure to each public enum
      (`Stage`, `ExpectedOutcome`, `ValidationSeverity`, …); record
      decisions in the owning module specs.
    - Deps: 2. Spec: all module specs.

13. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-test/en/` with its Japanese companion and
      synchronize content.
    - Deps: 12. Spec: repository documentation policy.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-test
cargo clippy -p mizar-test --all-targets -- -D warnings
```

For tasks that change discovery, expectations, or stages, also run the
consumers that embed corpus runners (currently):

```text
cargo test -p mizar-frontend
```

Check the task off here once tests pass.

## Notes

- This crate stays minimal: validation, planning, comparison, reporting —
  never pipeline execution and never pipeline dependencies.
- Stage ids are canonical values shared with `.expect.toml`,
  `spec_trace.toml`, and consumer enums; display names may localize, ids
  may not.
- Fail/soundness coverage takes priority near the kernel; the 40/60
  pass/fail mix is a corpus-wide target, not per-directory.
- Snapshot baselines are the stability surface for internal renderings;
  the renderings themselves are not stable artifacts.
