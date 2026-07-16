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
| layout | [layout.md](./layout.md) | `src/layout.rs`, `src/path_rules.rs` | [x] discovery, missing-sidecar diagnostics, and unknown-root inventory implemented; public API/ownership wording synchronized by task 238, unreachable sidecar-name diagnostic removed by task 239, and direct raw-order/missing-root/unknown-root coverage added by task 240 |
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

- `layout`: tasks 238-240 synchronize the documented discovery API and
  harness/expectation ownership, remove the unreachable sidecar-name
  diagnostic, and close MT-AUDIT-020 with direct raw-order, missing-root, and
  multiple-unknown-root coverage. Keep that coverage synchronized as new roots land.
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
    - The historical selected task-10 ledger records `mizar-parser` task 3
      (`parse-only`),
      `mizar-resolve` task 23 (`declaration-symbol`), `mizar-checker` task 12
      (`type-elaboration` external-gap runner), task 16
      (source-derived builtin type-expression normalization), task 17
      (source-derived builtin type-expression projection to `ResolvedTypedAst`),
      task 18 (source-derived reserve declaration semantic bridge), task 19
      (reserve bridge `ResolvedTypedAstSummary::from_ast` readiness plus next
      builtin declaration inventory), task 20 (reserve bridge binder-only
      `CoreContext` readiness), and the post-task-20 resolver R-G007
      parser-backed same-signature/different-return functor conflict active
      declaration-symbol seed plus exact SymbolEnv-derived declaration-symbol
      pass payload assertions, checker task 50's same-module attributed
      reserve evidence-query active fail slice, and checker task 51's
      same-module local mode reserve missing-expansion active fail slice, and
      checker task 52's same-module local structure reserve evidence-query
      active fail slice, and checker task 53's attributed local structure
      reserve evidence-query active fail slice, and checker task 54's
      attributed local mode reserve missing-expansion active fail slice, and
      checker task 55's bare same-module local mode expansion active pass
      slice, and checker task 56's one-edge same-module local-mode expansion
      chain active pass/gap slice, and checker task 57's same-module local-mode
      structure-RHS evidence-query active fail slice, and checker task 58's
      same-module local-mode attributed-builtin-RHS evidence-query active fail
      slice, and checker task 59's same-module attributed local-mode reserve
      evidence-query active fail slice, and checker task 60's same-module
      attributed local-mode structure-RHS evidence-query active fail slice, and
      checker task 61's same-module attributed local-mode attributed-builtin-RHS
      evidence-query active fail slice, and checker task 62's same-module
      local-mode structure-RHS chain evidence-query active fail slice, and
      checker task 63's same-module local-mode attributed-RHS chain
      evidence-query active fail slice, and checker task 64's same-module
      attributed local-mode bare-builtin chain evidence-query active fail slice,
      and checker task 65's same-module attributed local-mode structure-RHS chain
      evidence-query active fail slice, and checker task 66's same-module
      attributed local-mode attributed-builtin-RHS chain evidence-query active
      fail slice, and checker task 67's structure-qualified attribute
      extraction-gap active boundary slice, and checker task 68's
      argument-bearing local-mode reserve extraction-gap active boundary slice,
      and checker task 69's argument-bearing local-structure reserve
      extraction-gap active boundary slice, and checker task 70's bracket-form
      local-mode reserve extraction-gap active boundary slice, checker task 71's bracket-form
      local-structure reserve extraction-gap active boundary slice, checker
      task 72's two-edge bare local-mode chain active pass slice, checker task
      73's three-edge pass slice, checker task 74's structural bare
      local-mode chain active pass slice, checker task 75's lower-stage
      forward local-mode active-range boundary, checker task 76's lower-stage
      forward local-structure active-range boundary, checker task 77's
      lower-stage forward local-attribute active-range boundary, checker task
      78's imported structure reserve extraction-gap boundary, checker
      task 79's imported mode reserve extraction-gap boundary, checker
      task 80's imported attribute reserve extraction-gap boundary, checker
      task 81's argument-bearing local attribute reserve extraction-gap
      boundary plus declaration-symbol suffix projection, checker task
      82's imported mode reserve provenance bridge, checker task 83's
      imported structure reserve provenance bridge, checker task 84's
      imported attribute reserve provenance bridge, checker task 85's
      imported non-empty attribute reserve provenance bridge, checker task
      116's imported positive empty attribute reserve provenance bridge, and
      checker task 86's theorem formula extraction-gap boundary, checker task 106's
      builtin equality theorem term/formula checker bridge, checker task 110's imported predicate/functor
      theorem checker bridge, checker task 108's builtin
      membership theorem checker bridge, checker task 107's builtin
      inequality theorem checker bridge, checker task 109's builtin
      type assertion theorem term/formula/type checker bridge, checker task 113's imported
      attribute assertion theorem checker bridge, checker task 114's exact
      attribute-level non-empty imported attribute assertion theorem checker
      bridge, checker task 111's exact set-enumeration theorem
      checker bridge, checker task 112's exact formula connective/quantifier
      shell checker bridge, checker task 117's exact formula constant kind
      checker bridge, checker task 118's builtin-binary exact-token guard,
      checker task 119's exact reserved-variable equality active pass bridge,
      checker task 120's exact reserved-variable membership active pass bridge,
      checker task 121's exact reserved-variable inequality active pass bridge,
      checker task 122's reflexive type-assertion gate and exact
      reserved-variable type-assertion active pass bridge,
      checker task 123's exact distinct reserved-variable equality active pass
      bridge,
      checker task 124's exact multiple-reserve-declaration equality active pass
      bridge with distinct pre-normalization source ranges and one semantic
      normalized type,
      checker task 125's exact heterogeneous-reserve membership active pass
      bridge with left `object`, right/expected `set`, and two normalized
      semantic identities,
      checker task 126's exact direct-local-mode reserved-variable equality
      active pass bridge with four raw local-mode result/expected inputs and
      one builtin-`set` identity normalized from the real expansion RHS,
      checker task 127's exact one-edge local-mode-chain reserved-variable
      equality active pass bridge with four raw outer-mode inputs, two real
      expansion links, and terminal-RHS normalized provenance,
      checker task 128's exact direct local-object-mode reserved-variable
      equality active pass bridge with four raw object-mode inputs and one
      builtin-`object` identity normalized from the real expansion RHS,
      checker task 129's exact one-edge local-object-mode-chain equality active
      pass bridge with four raw outer-mode inputs, two real expansions, and
      terminal object-RHS normalized provenance,
      checker task 130's exact direct-local-mode inequality active pass bridge
      with four raw mode inputs, one real expansion, terminal set-RHS
      provenance, and a fact-free pre-desugaring checked inequality,
      checker task 131's exact direct-local-object-mode inequality active pass
      bridge with four raw object-mode inputs, one real expansion, terminal
      object-RHS provenance, and a fact-free pre-desugaring checked inequality,
      checker task 132's exact one-edge local-mode-chain inequality active pass
      bridge with four raw outer-mode inputs, two real expansions, terminal
      set-RHS provenance, and a fact-free pre-desugaring checked inequality,
      checker task 133's exact one-edge local-object-mode-chain inequality
      active pass bridge with four raw outer-mode inputs, two real expansions,
      terminal object-RHS provenance, and a fact-free pre-desugaring checked
      inequality,
      checker task 134's exact two-edge local-mode-chain equality active pass
      bridge with four raw outer-mode inputs, three real expansions, terminal
      set-RHS provenance, and a fact-free checked equality,
      checker task 135's exact two-edge local-object-mode-chain equality active
      pass bridge with four raw outer-mode inputs, three real expansions,
      terminal object-RHS provenance, and a fact-free checked equality,
      checker task 136's exact two-edge local-mode-chain inequality active pass
      bridge with four raw outer-mode inputs, three real expansions, terminal
      set-RHS provenance, and a fact-free pre-desugaring checked inequality,
      checker task 137's exact two-edge local-object-mode-chain inequality
      active pass bridge with four raw outer-mode inputs, three real expansions,
      terminal object-RHS provenance, and a fact-free pre-desugaring checked
      inequality,
      checker task 138's exact direct local-mode reserved-variable type-
      assertion active pass bridge with a raw local-mode subject, an independent
      builtin-set asserted type, one real expansion, terminal set-RHS
      provenance, and a fact-free checked type assertion,
      checker task 88's proof skeleton
      extraction-gap boundary, and checker task 89's statement proof
      extraction-gap boundary, and checker task 90's predicate/functor
      definition extraction-gap boundary, and checker task 91's attribute
      definition extraction-gap boundary, and checker task 92's mode/structure
      definition extraction-gap boundary, and checker task 93's proof-local
      declaration extraction-gap boundary, checker task 94's proof-local
      inline definition extraction-gap boundary, and checker task 95's
      registration block extraction-gap boundary, and checker task 96's
      redefinition/notation extraction-gap boundary as
      prepared/implemented increments.
      This historical inline selection has checker task 138 as its
      latest-numbered entry. Detailed lifecycle for checker tasks 139-236 is
      maintained in the paired [crate plan](./00.crate_plan.md),
      [harness](./harness.md), and [traceability](./traceability.md). The
      active Task 233 corpus contains 180 type-elaboration cases within 395
      cases / 359 requirements, type-elaboration coverage 227/215, and
      pass/fail 211/184; Step 5 is active and Steps 6/7 are deferred. Checker
      task 233 supplies the latest active exact parenthesized builtin-object
      equality row without rebaselining an existing expectation.
      Checker task 234 supplies the latest active exact seven-expansion
      set-terminal full-distance six-hop asserted-head row. Its fixture and six
      backlinks account for 396 cases / 360 requirements, type-elaboration
      228/216, pass/fail 212/184, and active runner 181 without rebaselining an
      existing expectation.
      Checker task 236 supplies the latest active exact object-terminal
      full-distance six-hop sibling with six directly validated links, one
      terminal-only object edge, six backlinks, and all 57 prior owners. The
      route accounts for 397 cases / 361 requirements, type-elaboration
      229/217, pass/fail 213/184, and active runner 182 without rebaselining an
      existing expectation.
      Checker task 29, `mizar-vc` task 15,
      `mizar-atp` task 20, and `mizar-kernel` task 17 are recorded as
      `paced/open`; no placeholder runner or fake active fixture is created for
      them.
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

16. **Source-derived builtin type-expression bridge.** [x]
    - Completed: adds the first real source-to-checker extraction slice for active
      `type_elaboration`: after frontend parsing and resolver symbol
      collection pass, extracts reserve-only unrecovered builtin `set`/`object`
      `TypeExpression` nodes into checker-owned `TypeExpressionInput` payloads,
      normalizes them through `mizar-checker`, and assembles a minimal `TypedAst`
      shell.
    - Keep unsupported declaration, term, formula, coercion, attribute,
      mode/structure, overload, fact, CoreIr, ControlFlowIr, VC, and proof seed
      payloads on explicit external gaps. Do not rebaseline existing `.miz` or
      expectation semantics, and do not promote Architecture-22 rows without
      prepared consumer execution.
    - Deps: 10, `mizar-checker` task 12. Spec: [harness.md](./harness.md),
      [expectation_schema.md](./expectation_schema.md),
      [traceability.md](./traceability.md), checker MC-G020.

17. **Source-derived builtin `ResolvedTypedAst` bridge.** [x]
    - Completed: extends the task-16 active `type_elaboration` source bridge
      so the normalized builtin `set`/`object` type-expression payloads are
      assembled into `TypedAst` and then projected through
      `ResolvedTypedAst::assemble` with real checker-owned expression metadata,
      source-preserved node hints, and empty cluster/overload predecessor
      outputs. The runner verifies that every supported source type site
      reaches a resolved node, expression metadata, and a final type without
      diagnostics.
    - Keep declaration extraction, non-builtin type heads, attributes, terms,
      formulas, overload candidates, cluster facts, proof evidence, CoreIr,
      ControlFlowIr, VC seeds, and `proof_verification` rows deferred until
      their producer/consumer seams are executable. Do not add fake active
      fixtures, public checker diagnostic codes, or CoreIr/ControlFlowIr/VC
      payloads.
    - Deps: 16, `mizar-checker` task 28. Spec: [harness.md](./harness.md),
      checker `resolved_typed_ast.md`, checker MC-G020/MC-G027.

18. **Source-derived reserve declaration semantic bridge.** [x]
    - Completed: extends the active `type_elaboration` source bridge from
      builtin type-expression sites to reserve-only builtin declaration
      payloads. The runner extracts unrecovered top-level `reserve` items with
      bare builtin `set`/`object` heads into syntax-free source reserve
      payloads. Checker task 48 owns the producer seam that turns those payloads
      into a checker-owned module `BindingEnv`, one `DeclarationInput` per
      binding, binding-specific `TypeExpressionInput` sites, and
      `DeclarationChecker` output; the runner continues that handoff into
      `TypedAst` and `ResolvedTypedAst`. Shared source type ranges such as
      `reserve x, y for set` keep distinct typed sites for each binding.
    - Unsupported non-builtin declarations beyond task 96's redefinition/notation
      extraction-gap boundary, task 95's registration block extraction-gap boundary, task 94's proof-local inline definition boundary,
      task 93's proof-local declaration boundary, and task 92's mode/structure
      definition boundary, imported attribute provenance beyond
      the task-84 `TypeCaseAttr` bridge, task-85 negative
      `empty`/builtin-`set` bridge, task-116 positive `empty`/builtin-`set`
      bridge, and task-80 boundary, imported structure provenance beyond the task-83
      `R` bridge, task-97 `TypeCaseStruct` bridge, and task-78 boundary, imported mode expansion payloads beyond
      task 82's provenance bridge, attribute argument payloads beyond the task-81 boundary,
      attributed or argument-bearing
      mode/structure heads, structure base-shape payloads, definition payloads beyond the task-92 extraction-gap boundary, proof-local declaration payloads beyond the task-93 extraction-gap boundary, inline definition payloads beyond the task-94 extraction-gap boundary, registration payloads and activation/correctness payloads beyond the task-95 extraction-gap boundary, redefinition/notation payloads beyond the task-96 extraction-gap boundary, imported predicate/functor semantic payloads, quantifier binder/context payloads, terms and
      membership operand expected-type construction/checking, inequality desugaring or
      equality semantic checking, broader type-assertion type payload extraction,
      type-assertion semantic checking, imported attribute assertion
      attribute-chain/provenance payload extraction, imported attribute-level
      non-empty assertion attribute-chain/provenance payload extraction, set-enumeration
      term payload extraction, negated
      attribute admissibility/semantic checking, attribute admissibility/semantic
      checking, formula/theorem/proof payloads beyond the
      task-106 builtin equality theorem checker bridge, task-107 builtin inequality theorem checker bridge, task-108 builtin membership theorem checker bridge, task-109 builtin type assertion theorem checker bridge, task-110 imported predicate/functor theorem checker bridge, task-111 set-enumeration theorem checker bridge, task-112 formula connective/quantifier shell checker bridge, task-113 imported attribute assertion theorem checker bridge, task-114 exact attribute-level non-empty imported attribute assertion theorem checker bridge, and task-86/task-105/task-88/task-89/task-93/task-94/task-95/task-96
      extraction-gap boundaries,
      coercions, overload payloads, facts, CoreIr,
      ControlFlowIr, VC payloads, and proof evidence remain on the explicit
      `type_elaboration.external_dependency.ast_payload_extraction` gap. The
      CoreIr/ControlFlowIr/VC/proof rows are not promoted because no real
      source-derived payload is lowered into those consumers yet.
    - Deps: 16, 17, checker MC-G011/MC-G016/MC-G020. Spec:
      [harness.md](./harness.md), [expectation_schema.md](./expectation_schema.md),
      [traceability.md](./traceability.md).

19. **Reserve bridge core summary readiness and builtin declaration
    inventory.** [x]
    - Completed: extends the active reserve-only builtin declaration bridge by
      passing the real checker-owned `ResolvedTypedAst` payload to
      `mizar-core`'s `ResolvedTypedAstSummary::from_ast`. The runner verifies
      that the summary preserves source/module identity and has no checker
      recovery/diagnostic sites for successful active reserve pass cases.
    - Inventory result: no next builtin declaration family is promoted in this
      task. `let`, `given`, `consider`, and quantified declarations require
      local scope, assumption, formula, or constraint-discharge payloads;
      `set` requires RHS term inference payloads; `reconsider` requires
      coercion/obligation evidence; `deffunc`/`defpred` require body/formal
      payloads. Those families remain on the source-to-checker extraction gap
      until a prepared active runner seam can execute them without raw
      reconstruction or fake evidence.
    - The `ResolvedTypedAstSummary` read is summary-only; it does not build or
      publish `CoreIr`, `ControlFlowIr`, VC seeds, proof rows, or public
      checker diagnostic codes.
    - Deps: 18, `mizar-core` elaborator summary API. Spec:
      [harness.md](./harness.md), [expectation_schema.md](./expectation_schema.md),
      [traceability.md](./traceability.md), core `elaborator.md`.

20. **Reserve bridge core context readiness.** [x]
    - Completed: extends the active reserve-only builtin declaration bridge by
      feeding the same real checker-owned `BindingEnv` and `ResolvedTypedAst`
      handoff into `mizar-core` `CoreContextInput` with one
      `CoreVariableSeed` and one `CoreBinderSeed` per extracted reserve
      binding and no `CoreItemSeed`. The runner verifies source/module
      identity, binder source ranges, checker provenance, empty item registry,
      empty core diagnostics, and an empty core worklist for successful active
      reserve pass cases.
    - This is a binder/context readiness check only. Reserve declarations still
      provide no owner item, term, formula, proof, algorithm, or obligation
      payload, so this task does not construct or publish `CoreIr`,
      `ControlFlowIr`, VC seeds, proof rows, public checker diagnostic codes,
      new active fixtures, or expectation semantic changes.
    - Deps: 19, `mizar-core` `prepare_core_context`. Spec:
      [harness.md](./harness.md), [expectation_schema.md](./expectation_schema.md),
      [traceability.md](./traceability.md), core `elaborator.md`.

### Kernel soundness-audit follow-ups (2026-07-03)

The kernel acceptance-boundary audit
([soundness_argument.md](../../mizar-kernel/en/soundness_argument.md))
reported two harness-owned findings, F7 and F8. These are minimal
audit-driven additions; broader runner growth remains task 10 pacing.

21. **Corrected-path soundness vocabulary in the required-case registry (kernel F7).** [x]
    - Extend `REQUIRED_SOUNDNESS_CASES` and the layout/expectation docs with
      the corrected kernel rejection vocabulary: `invalid_sat_refutation`,
      `context_mismatch`, `missing_provenance`, and an
      unsupported-legacy-certificate-under-normal-policy case, per
      architecture 20's required coverage. Re-key the certificate-corpus
      sidecars that currently use non-`soundness.` stable keys for these
      reasons onto the new `soundness.certificate.*` keys in the same
      change, without changing any rejection behavior.
    - Acceptance: registry rejects unknown `soundness.*` keys as before;
      the 23-case audit corpus satisfies the extended registry;
      `mizar-test` plan errors stay 0; the fail-soundness bookkeeping
      reports the corrected cases as covered.
    - Completed: task 21 adds the corrected `soundness.certificate.*`
      required-case keys for `invalid_sat_refutation`, `context_mismatch`,
      `missing_provenance`, and unsupported legacy certificates under normal
      policy while retaining legacy `invalid_sat_proof`. Existing certificate
      sidecars for the corrected reasons now use `domain = "certificate"` and
      soundness stable keys without changing payloads or rejection behavior.
    - Verify: `cargo test -p mizar-test`.
    - Deps: 8; corpus from mizar-kernel audit (`f75af877`). Spec:
      architecture 20; soundness_argument.md F7.

22. **Certificate-corpus root naming reconciliation (kernel F8).** [x]
    - Reconcile architecture 20's `tests/kernel_evidence/` directory list
      with the implemented `tests/certificates/` layout: rename one side or
      cross-reference both (docs-only if cross-referencing). Update
      architecture 20 (en+ja) and the corpus README in the same change.
    - Completed by task 22: architecture 20 (EN/JA), the certificate corpus
      README, the crate plan, and the kernel soundness argument now identify
      `tests/certificates/` as the canonical certificate/kernel-evidence
      corpus root. Remaining `tests/kernel_evidence/` mentions are historical
      retired-name notes, not normative corpus roots.
    - Verify: `cargo test -p mizar-test`; `git diff --check`.
    - Deps: none. Spec: architecture 20; soundness_argument.md F8.

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

## Task 241 Active Addendum

- [x] Add the exact test-first `(x) <> x` builtin-set fixture, a new expectation
  derived from Chapters 04/13/14/16, four shared backlinks, and one dedicated
  checker row. Do not modify or rebaseline existing fixtures or expectations.
- [x] Add active metadata/CLI assertions for runner 183 and protect the real
  frontend/resolver/checker payload with exact/negative/corruption/immutable/
  focused-regression and all-54-prior-owner bidirectional tests.
- [x] Synchronize the 398 cases / 362 requirements, type-elaboration 230/218,
  and pass/fail 214/184 counts. Parenthesized membership, imported/other
  parenthesized variants, proof/IR/VC, and broader semantics receive no Task 241
  credit. Step 5 remains active; Steps 6/7 remain deferred.

## Task 242 Active Addendum

- [x] Add the exact test-first builtin-object `(x) <> x` fixture, a Chapter
  03/04/13/14/16-derived expectation, five shared backlinks, and one dedicated
  checker row without modifying or rebaselining existing expectations.
- [x] Add active metadata/CLI assertions for runner 184 and protect the real
  frontend/resolver/checker payload with exact/negative/corruption/immutable/
  focused-regression and all-55-prior-owner bidirectional tests.
- [x] Synchronize 399 cases / 363 requirements, type-elaboration 231/219, and
  pass/fail 215/184. Parenthesized membership and active imported provenance
  receive no Task 242 credit; missing imported expansion/evidence/signature
  payloads and proof/CoreIr/ControlFlowIr/VC remain deferred. Step 5 remains
  active; Steps 6/7 remain deferred.

## Task 243 Active Addendum

- [x] Add the exact test-first builtin-set `(x) in x` fixture, a Chapter 04/13/
  14/16-derived expectation, four shared backlinks, and one dedicated checker
  row without modifying or rebaselining existing expectations.
- [x] Add active metadata/CLI assertions for runner 185 and protect the real
  frontend/resolver/checker payload with exact/negative/corruption/immutable/
  focused-regression and all-56-prior-owner bidirectional tests. Exercise no
  left expected input and unexpected-left/wrong-right/missing-right corruptions.
- [x] Synchronize 400 cases / 364 requirements, type-elaboration 232/220, and
  pass/fail 216/184. Discharge the extraction gap only for the exact source.
  Object-left/set-right parenthesized membership and active imported provenance
  receive no Task 243 credit; missing imported expansion/evidence/signature
  payloads and proof/CoreIr/ControlFlowIr/VC remain deferred. Step 5 remains
  active; Steps 6/7 remain deferred.

## Task 244 Active Addendum

- [x] Add the test-first `.miz`/expectation pair for the exact two-reserve source
  `reserve x for object; reserve y for set; theorem
  ParenthesizedHeterogeneousReserveMembershipPayloadBoundary: (x) in y;`.
- [x] Activate the real frontend/resolver runner route with ordered distinct
  bindings, ordinals 2/3, `BindingId(0/1)`, two written-range-anchored object/set
  identities, two inferred terms, five type entries, right-only expected-set
  input, and a checked membership without wrapper semantics or coercion.
- [x] Cover finite exact/near-miss/provenance/corruption behavior, immutable
  output, all 57 prior binary owners, Tasks 120/125/223/233/241/242/243, the real
  imported-mode-gap diagnostic fixture, and the real active sidecar.
- [x] Add five shared backlinks plus one dedicated requirement and synchronize
  active runner 186, cases/requirements 401/365, type 233/221, and pass/fail
  217/184.
- [x] Limit extraction-gap discharge to the exact source. Other parenthesized
  shapes and imported-positive provenance receive no Task 244 credit; missing
  imported expansion/evidence/signature payloads and proof/CoreIr/ControlFlowIr/
  VC remain deferred. Step 5 remains active; Steps 6/7 remain deferred.

## Task 245 Active Addendum

- [x] Add the exact test-first `x in (x)` fixture/expectation from Chapters 04/
  13/14/16, four shared backlinks, and one dedicated checker requirement.
- [x] Activate explicit `Right` wrapper-side metadata and a Task-245-only key/
  config/role namespace while retaining the six earlier `Left` routes.
- [x] Verify the real frontend/resolver/checker payload, right-inner expected-set
  ownership, side/config/range/constraint corruptions, Task-243 cross-route
  rejection, immutable/module boundaries, and all 58 prior owners in both
  directions.
- [x] Synchronize active runner 187, cases/requirements 402/366, type 234/222,
  and pass/fail 218/184. Other shapes and imported-positive provenance receive
  no credit; missing imported expansion/evidence/signature and proof/CoreIr/
  ControlFlowIr/VC remain deferred. Step 5 remains active; Steps 6/7 remain
  deferred.

## Task 246 Active Addendum

- [x] Add the exact three-mode set-terminal `(z) = z` fixture and six trace
  references without changing existing expectations.
- [x] Require conditional mode-node admission, three expansions, four raw Outer
  inputs, ordinal 1/2 `BindingId(0)`, one terminal set identity, two terms, six
  entries, two constraints, one checked equality, and no wrapper ownership.
- [x] Cover all five nonidentity orders, finite structure/provenance/corruption,
  Tasks 134/223, immutable/module behavior, 59 prior owners, and a real sidecar.
- [x] Synchronize runner 188, plan 403/367, type 235/223, pass/fail 219/184.
  Step 5 remains active; Steps 6/7 remain deferred.

## Runner Module-Boundary Refactor Backlog

Priority: complete this maintenance series before adding the next Step 5
semantic bridge. Classify it as behavior-preserving `design_drift` in source
layout and reviewability, not as new language or runner coverage. At Task 246
closeout, `src/runner.rs` has 111,262 lines: a 17,142-line pre-test-module
prefix containing 137 `#[cfg(test)]` helpers, followed by a single
approximately 94,120-line test module containing 272 `#[test]` attributes.

- [x] Audit the runner boundary and add paired EN/JA module-boundary documents
  in Task 248.
  Inventory orchestration, parse-only, declaration-symbol, type-elaboration,
  source-extraction, payload-validation, fixture-builder, and corruption-test
  ownership; record the dependency map, target source layout, move order, and
  exit criteria. Before any source move, update the paired `00.crate_plan.md`
  files with task IDs, affected files/tests, coverage-audit impact, completion
  conditions, and forbidden behavior. Keep this an audit/docs-only task and
  commit.
- [x] Task 249 mechanically moved the monolithic private `mod tests` out of
  `runner.rs` into `src/runner/tests.rs`.
  Preserve module privacy, test names, test discovery, helper behavior, and all
  public APIs. Do not combine the move with renaming, deduplication,
  generalization, or semantic cleanup. Commit the move as one task.
- [x] Split the private tests into shared support plus parse-only,
  declaration-symbol, and type-elaboration owners. Split type-elaboration
  further by cohesive source-bridge family when needed; use one bounded
  move-only task/commit per family and keep cross-owner isolation tests intact.
  Tasks 250-252, 253A, 254, and 253B completed the shared-support, parse-only,
  baseline type-elaboration source-extraction/handoff, leading reserved/binary,
  non-long-chain mode, and direct reserved fragments. Tasks 253/253B are now
  complete. Tasks 255A-255E completed the leading, four-edge, three-edge
  object, two-edge object, and final type-assertion asserted-head fragments;
  parent Task 255 and Task 256 are complete. Task 257A completed the leading
  binary-route fixture/isolation family. Fresh authority review isolates the
  Task 180 formula-constant fixture. Tasks 257A-257H and parent Task 257 are
  complete; the private test layout is stable.
- [ ] After the test layout is stable, split production helpers along the
  audited phase and ownership boundaries. Leave `runner.rs` as the public
  facade and top-level orchestration owner. Keep internal visibility minimal
  and do not change detail keys, diagnostics, payload contracts, fixture
  ownership, ordering, or fail-closed behavior. Tasks 258-259 completed the
  private shared frontend and parse-only owners, Tasks 260A-260B moved the
  shared resolver leaf and declaration-symbol owner, and Task 261 moved the
  fixture/import-summary owner. Tasks 262A-262B moved the common source-AST leaf,
  Task 262C moved the reserve type-expression/symbol-projection leaf, Task 262D
  moved the shared exact fixture-import AST projection, and Task 262E moved the
  reserve declaration/local-mode extraction family, and Task 262F moved only
  the standalone formula-constant source leaf, and Task 262G moved the shared
  exact numeral AST-projection prerequisite for the remaining formula
  extractors. Task 262H0 completed the test-only preservation prerequisite for
  the bounded builtin equality, inequality, and membership family, and Task
  262H completed the subsequent move. Task 262I0 completed the test-only
  preservation prerequisite for the bounded builtin type-assertion family;
  Task 262I moved only that family after I0. Task 262J0 completed the test-only
  preservation prerequisite for the imported predicate/functor family. The
  shared symbol projection moved in Task 262J1, and Task 262J2 moved the exact
  imported predicate/functor family. Fresh inventory decomposes the exact
  imported attribute assertion family into test-only preservation Task 262K0,
  followed by move-only Task 262K; both are now complete. Fresh inventory now
  decomposes the set-enumeration family into test-only preservation Task 262L0,
  followed by move-only Task 262L; both are now complete. Fresh inventory now
  decomposes the connective/quantifier family into test-only
  preservation Task 262M0, followed by move-only Task 262M; both are now
  complete. Fresh inventory decomposes the remaining reserved-variable
  formula work into test-only preservation Task 262N0, shared source-substrate
  Task 262N, direct-binary Task 262O, parenthesized-binary Task 262P, and
  type-assertion Task 262Q. Tasks 262N0, 262N, 262O, and 262P are complete.
  Fresh review inserted test-only preservation Task 262Q0 before move-only Task
  262Q; both and parent Task 262 are complete. Fresh dependency inventory
  decomposes Task 263 and selects bounded checker-handoff substrate Task 263A
  first; Task 263A is complete. Fresh inventory selected common frontend
  diagnostic projection Task 263B, which is also complete. Fresh inventory
  selected expected-result/failure-projection Task 263C, which is complete with
  exact-body and byte-stability preservation. Fresh Task 263 inventory now
  selected the exact 50-line type active-admission gate Task 263D, which is
  complete with exact-body and byte-stability preservation. Fresh Task 263
  inventory selected the exact 33-line checker-output transport substrate Task
  263E, which is complete with exact-body and byte-stability preservation.
  Fresh Task 263 inventory selected the exact 277-line checker-output builder
  family Task 263F, which is complete with exact-body and byte-stability
  preservation. Fresh inventory now selects exact 229-line type-assertion
  validator/shared normalized-type predicate family Task 263G, which is
  complete with exact-body and byte-stability preservation. Fresh inventory
  selected exact 380-line binary-formula validator/helper family Task 263H,
  which is complete with exact-body and byte-stability preservation. Fresh
  inventory now selects exact 67-line config-independent parenthesized-
  validator core Task 263I, which is complete with exact-body and byte-
  stability preservation. Fresh inventory now selects exact 46-line type-
  assertion result/detail core Task 263J, which is complete with exact-body and
  byte-stability preservation. Fresh inventory selected the exact 36-line
  binary-formula result/detail core as Task 263K, which is complete with exact-
  body and byte-stability preservation. Fresh inventory selected the exact
  16-line parenthesized-binary output-detail core as Task 263L, which is
  complete with exact-body and byte-stability preservation. Fresh inventory
  selected the exact 17-line parenthesized-binary payload-detail wrapper as
  Task 263M, which is complete with exact-body and byte-stability preservation.
  Fresh inventory now selects the remaining config/named-wrapper family, so
  Task 263 keeps this parent item open.
- [ ] Close out the series by synchronizing the paired source-layout inventory,
  crate plan, todo, harness/source-path tables, and ownership guards. Confirm
  that active runner 188, plan 403/367, type-elaboration 235/223, pass/fail
  219/184, all 272 discovered unit tests, expectation/trace credit, and all
  existing `.miz` intent remain unchanged before fresh inventory resumes
  Step 5.

For every source-moving task, require review-only checks for visibility drift,
test-discovery drift, owner-boundary drift, source/docs inconsistency, and
accidental behavior changes. Run focused tests, `cargo test -p mizar-test`,
`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
workspace `cargo test`, and `git diff --check`; repair and rerun failures until
all commands pass. A test or verification failure is not itself a reason to
stop this series.
