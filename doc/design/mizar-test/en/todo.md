# mizar-test TODO

## Parser Task 46 / Operator-Declaration Parse-Only Completion

- [x] Admit the exact active pass/fail pair and pin its sidecars.
- [x] Mark `spec.en.10.operator_declarations.parser` covered with exact
  backlinks and remove only the six now-exercised operator words from the
  parse-only deferred-reserved-word set.
- [x] Keep existing corpus/expectations, production runner layout, semantic
  operator behavior, Task 49, and Steps 6/7 unchanged.

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
    - Core Task 31 is a completed consumer-paced increment within this open
      task, not a new mizar-test task id. The existing Task-180 active
      type-elaboration case validates the complete checker bundle, lowers the
      exact CoreIr twice, and verify-compares its committed full-byte baseline.
      This adds one covered requirement (plan 403/368, type 236/224) without
      changing 188 active cases, 219/184 pass/fail, the `.miz`, phase, or
      diagnostics. Broader CoreIr/ControlFlowIr/proof-verification remains
      paced by Core Task 32 and its prerequisites.
    - Checker Task 247 now names two future non-placeholder consumer increments
      inside this same open task. `MT10-FS` owns the `formula-statement`
      stage/tag/report path and adds the distinct
      `pass_formula_statement_reserved_variable_equality_smoke_001` source and
      singular formula-statement sidecar, with corruption of the same checker
      bundle as negative runner coverage. The existing type-elaboration fixture
      and its sole sidecar stay unchanged. `MT10-AS` owns the
      `advanced-semantics` path and must run a new spec-derived, non-Task-49,
      single-ordinary-functor/single-candidate reflexive-equality smoke through
      real definition, application, candidate, and ordinary-root producers,
      plus a distinct Task-270 definition-time capture smoke that preserves the
      outer resolved identity across display-name shadowing. It also owns the
      existing advanced-semantics omitted-`reconsider` case after parser Task
      47 and checker Tasks 251/271-272, including explicit non-accepting pending/
      blocked intent and no proof search. Neither increment
      may be an empty/placeholder runner or activate the 24-fixture Task-49
      reconciliation set early. Their complete dependencies and blocked gates are
      canonical in checker
      [payload_family_decomposition.md](../../mizar-checker/en/payload_family_decomposition.md).
    - Core Task 32 now names five more non-placeholder increments in this open
      task: `MT10-CIR-TE`, `MT10-CIR-FS`, `MT10-CIR-AS`, `MT10-CIR-ALG`, and
      `MT10-CFG-PV`. Their exact stage/tag/phase/artifact dependencies and
      corruption boundaries are canonical in Core
      [source_family_decomposition.md](../../mizar-core/en/source_family_decomposition.md).
      The first general Core snapshot integration and first
      `SnapshotKind::ControlFlowIr` change must each land with its first real
      baseline, never as empty infrastructure. Naming the consumers changes no
      current runner, sidecar, trace status, or coverage.
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
      The separately traced exact Task-180 CoreIr snapshot is now promoted by
      Core Task 31. Every broader CoreIr/ControlFlowIr/VC/proof row remains
      deferred because no corresponding real source-derived payload is lowered
      into those consumers yet.
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
- [x] After the test layout is stable, split production helpers along the
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
  Fresh inventory selected the exact seven-fragment/720-line cohesive
  parenthesized config/named-route owner as Task 263N, which is complete with
  exact-body and byte-stability preservation. Fresh inventory selected the
  exact eight-fragment/546-line leading direct-binary route owner as Task 263O,
  which is complete with token-identical body and byte-stability preservation.
  Fresh inventory selected the corrected exact five-fragment/313-line
  multiple-reserve declaration binary route family as Task 263P, which is
  complete with token-identical body and byte-stability preservation. Fresh
  inventory selected the exact five-fragment/116-line base reserved-variable
  membership/inequality route family as Task 263Q, which is complete with
  token-identical body and byte-stability preservation. Fresh inventory now
  selected the exact ten-fragment/183-line direct local-mode membership,
  equality, and inequality route family as Task 263R, which is complete with
  token-identical body and byte-stability preservation. Fresh inventory now
  selected the exact ten-fragment/190-line direct local-object-mode membership,
  equality, and inequality route family as Task 263S, which is complete with
  token-identical body and byte-stability preservation. Fresh inventory now
  selected the exact fourteen-fragment/207-line chained local-mode membership,
  equality, and inequality route family as Task 263T, which is complete with
  token-identical body and byte-stability preservation. Fresh inventory now
  selected the exact nine-fragment/229-line chained local-object-mode
  membership, equality, and inequality route family as Task 263U, which is
  complete with token-identical body and byte-stability preservation. Fresh
  inventory selected the exact fifteen-fragment/222-line two-edge local-mode
  membership, equality, and inequality route family as Task 263V, which is
  complete with token-identical body and byte-stability preservation. Fresh
  inventory selected the exact eleven-fragment/241-line two-edge local-object-
  mode membership, equality, and inequality route family as Task 263W, which
  is complete with token-identical body and byte-stability preservation. Fresh
  inventory selects the exact fifteen-fragment/242-line three-edge local-mode
  membership, equality, and inequality route family as Task 263X, which is
  complete with token-identical body and byte-stability preservation. Fresh
  inventory selects the exact eleven-fragment/258-line three-edge local-
  object-mode membership, equality, and inequality route family as Task 263Y,
  which is complete with token-identical body and byte-stability preservation.
  Fresh inventory selects the exact fifteen-fragment/252-line four-edge local-
  mode membership, equality, and inequality route family as Task 263Z, which
  is complete with token-identical body and byte-stability preservation. Fresh
  inventory selects the exact eleven-fragment/273-line four-edge local-object-
  mode membership, equality, and inequality route family as Task 263ZA, which
  is complete with token-identical body and byte-stability preservation. Fresh
  dependency inventory selects the exact two-fragment/74-line shared long-chain
  seven-expansion definition tables as Task 263ZB, which is complete with
  token-identical body and byte-stability preservation. Fresh inventory can now
  select the exact fifteen-fragment/176-line local-mode long-chain membership,
  equality, and inequality binary route family as Task 263ZC, which is complete
  with token-identical body and byte-stability preservation. Fresh inventory
  selects the exact fifteen-fragment/186-line local-object-mode long-chain
  membership, equality, and inequality binary route sibling as Task 263ZD;
  it is complete with token-identical body and byte-stability preservation.
  Fresh inventory selects the exact five-fragment/52-line local-mode long-chain
  reserved-variable type-assertion route as Task 263ZE and the first nonempty
  owner in private `type_assertion_routes.rs`; it is complete with token-
  identical body and byte-stability preservation. Fresh inventory selects the
  exact five-fragment/48-line local-mode long-chain same-mode asserted-head
  route as Task 263ZF in the same owner; it is complete with token-identical
  body and byte-stability preservation. Fresh inventory selects the exact
  five-fragment/50-line local-mode long-chain immediate-radix asserted-head
  route as Task 263ZG in the same owner; it is complete with token-identical
  body and byte-stability preservation. Fresh inventory selects the exact
  five-fragment/51-line local-mode long-chain two-hop asserted-head route as
  Task 263ZH in the same owner; it is complete with token-identical body and
  byte-stability preservation. Fresh inventory selects the exact five-
  fragment/54-line local-mode long-chain three-hop asserted-head route as Task
  263ZI in the same owner; it is complete with token-identical body and byte-
  stability preservation. Fresh inventory selects the exact five-fragment/55-
  line local-mode long-chain four-hop asserted-head route as Task 263ZJ in the
  same owner; it is complete with token-identical body and byte-stability
  preservation. Fresh inventory selects the exact five-fragment/56-line local-
  mode long-chain five-hop asserted-head route as Task 263ZK in the same owner;
  it is complete with token-identical body and byte-stability preservation.
  Fresh inventory selects the exact five-fragment/55-line local-mode long-chain
  six-hop asserted-head route as Task 263ZL in the same owner; it is complete
  with token-identical body, byte-stability, and stale local-table runner-
  exposure removal. Fresh inventory selects the exact five-fragment/58-line
  local-object-mode long-chain six-hop asserted-head route as Task 263ZM in the
  same owner; it is complete with token-identical body, byte-stability, and
  object-terminal fail-closed preservation. Fresh inventory returns to the
  exact five-fragment/57-line local-object-mode long-chain five-hop asserted-
  head route as Task 263ZN in the same owner; it is complete with token-
  identical body, byte-stability, and object-terminal fail-closed preservation.
  Fresh inventory returns to the exact five-fragment/56-line local-object-mode
  long-chain four-hop asserted-head route as Task 263ZO in the same owner; it is
  complete with token-identical body, byte-stability, and object-terminal fail-
  closed preservation.
  Fresh inventory returns to the exact five-fragment/55-line local-object-mode
  long-chain three-hop asserted-head route as Task 263ZP in the same owner; it
  is complete with token-identical body, byte-stability, and object-terminal
  fail-closed preservation.
  Fresh inventory returns to the exact five-fragment/54-line local-object-mode
  long-chain two-hop asserted-head route as Task 263ZQ in the same owner; it is
  complete with token-identical body, byte-stability, and object-terminal fail-
  closed preservation.
  Fresh inventory returns to the exact five-fragment/52-line local-object-mode
  long-chain immediate-radix asserted-head route as Task 263ZR in the same
  owner; it is complete with token-identical body, byte-stability, and object-
  terminal fail-closed preservation.
  Fresh inventory returns to the exact five-fragment/50-line local-object-mode
  long-chain same-mode asserted-head route as Task 263ZS in the same owner; it
  is complete with token-identical body, byte-stability, and object-terminal
  fail-closed preservation.
  Fresh inventory returns to the exact five-fragment/52-line local-object-mode
  long-chain reserved-variable builtin type-assertion route as Task 263ZT in
  the same owner; it is complete with token-identical body, byte-stability,
  direct sibling-table ownership, and object-terminal fail-closed
  preservation.
  Fresh inventory returns to the exact five-fragment/53-line direct local-
  object-mode reserved-variable builtin type-assertion route as Task 263ZU in
  the same owner; it is complete with token-identical body, byte-stability,
  and object-terminal fail-closed preservation.
  Fresh inventory selects the exact five-fragment/67-line chained local-object-
  mode reserved-variable builtin type-assertion route as Task 263ZV in the same
  owner; it is complete with token-identical body, byte-stability, two-
  expansion object-terminal chain, and fail-closed preservation. Fresh
  inventory returns to the remaining local-object-mode type-assertion/
  asserted-head routes and selects the exact five-fragment/71-line two-edge
  local-object-mode reserved-variable builtin type-assertion route as Task
  263ZW in the same owner. Preserve its token-identical body, byte-stability,
  three-expansion object-terminal chain, and fail-closed behavior; it is
  complete. Fresh inventory returns to the remaining local-object-mode type-
  assertion/asserted-head routes and selects the exact five-fragment/82-line
  three-edge local-object-mode reserved-variable builtin type-assertion route
  as Task 263ZX in the same owner. Preserve its token-identical body, byte-
  stability, four-expansion object-terminal chain, and fail-closed behavior;
  it is complete. Fresh inventory returns to the remaining local-object-mode
  type-assertion/asserted-head routes and selects the exact five-fragment/81-
  line four-edge local-object-mode reserved-variable builtin type-assertion
  route as Task 263ZY in the same owner. Preserve its token-identical body,
  byte-stability, five-expansion object-terminal chain, and fail-closed
  behavior; it is complete. Fresh inventory returns to the remaining local-
  object-mode asserted-head routes and selects the exact five-fragment/55-line
  direct local-object-mode same-mode asserted-head route as Task 263ZZ in the
  same owner. Preserve its token-identical body, byte-stability, one-expansion
  object-terminal same-mode behavior, and fail-closed behavior; it is complete.
  Fresh inventory returns to the remaining local-object-mode asserted-head
  routes and selects the exact five-fragment/63-line chained local-object-mode
  same-mode asserted-head route as Task 263ZZA in the same owner. Preserve its
  token-identical body, byte stability, two-expansion object-terminal same-mode
  behavior, and fail-closed behavior; it is complete. Fresh inventory returns
  to the remaining local-object-mode asserted-head routes and selects the exact
  five-fragment/65-line chained local-object-mode immediate-radix asserted-head
  route as Task 263ZZB in the same owner. Preserve its token-identical body,
  byte stability, two-expansion object-terminal immediate-radix behavior, and
  fail-closed behavior; it is complete. Fresh inventory returns to the
  remaining local-object-mode asserted-head routes and selects the exact five-
  fragment/68-line two-edge local-object-mode same-mode asserted-head route as
  Task 263ZZC in the same owner. Preserve its token-identical body, byte
  stability, three-expansion object-terminal same-mode behavior, and fail-
  closed behavior; it is complete. Fresh inventory returns to the remaining
  local-object-mode asserted-head routes and selects the exact five-fragment/
  72-line two-edge local-object-mode immediate-radix asserted-head route as
  Task 263ZZD in the same owner. Preserve its token-identical body, byte
  stability, three-expansion object-terminal immediate-radix behavior, and
  fail-closed behavior; it is complete. Fresh inventory returns to the
  remaining local-object-mode asserted-head routes and selects the exact five-
  fragment/71-line two-edge local-object-mode two-hop asserted-head route as
  Task 263ZZE in the same owner. Preserve its token-identical body, byte
  stability, three-expansion object-terminal two-hop behavior, and fail-closed
  behavior; it is complete. Fresh inventory returns to the remaining local-
  object-mode asserted-head routes and selects the exact five-fragment/83-line
  three-edge local-object-mode two-hop asserted-head route as Task 263ZZF in
  the same owner. Preserve its token-identical body, byte stability, four-
  expansion object-terminal two-hop behavior, and fail-closed behavior; it is
  complete. Fresh inventory returns to the remaining local-object-mode
  asserted-head routes and selects the exact five-fragment/89-line four-edge
  local-object-mode two-hop asserted-head route as Task 263ZZG in the same
  owner. Preserve its token-identical body, byte stability, five-expansion
  object-terminal two-hop behavior, and fail-closed behavior; it is complete.
  Fresh inventory selects the exact five-fragment/84-line three-edge local-
  object-mode three-hop asserted-head route as Task 263ZZH in the same owner.
  Preserve its token-identical body, byte stability, four-expansion object-
  terminal three-hop behavior, and fail-closed behavior; it is complete. Fresh
  inventory selects the exact five-fragment/91-line four-edge local-object-mode
  three-hop asserted-head route as Task 263ZZI in the same owner. Preserve its
  token-identical body, byte stability, five-expansion object-terminal three-
  hop behavior, and fail-closed behavior; it is complete. Fresh inventory
  returns to the remaining local-object-mode asserted-head routes and selects
  the exact five-fragment/92-line four-edge local-object-mode four-hop
  asserted-head route as Task 263ZZJ in the same owner. Preserve its token-
  identical body, byte stability, five-expansion object-terminal four-hop
  behavior, and fail-closed behavior; it is complete. Fresh inventory returns
  to the remaining local-object-mode asserted-head routes and selects the exact
  five-fragment/81-line three-edge local-object-mode immediate-radix asserted-
  head route as Task 263ZZK in the same owner. Preserve its token-identical
  body, byte stability, four-expansion object-terminal immediate-radix
  behavior, and fail-closed behavior; it is complete. Fresh inventory returns
  to the remaining local-object-mode asserted-head routes and selects the exact
  five-fragment/86-line four-edge local-object-mode immediate-radix asserted-
  head route as Task 263ZZL in the same owner. Preserve its token-identical
  body, byte stability, five-expansion object-terminal immediate-radix
  behavior, and fail-closed behavior; it is complete. Fresh inventory returns
  to the remaining local-object-mode asserted-head routes and selects the exact
  five-fragment/78-line four-edge local-object-mode same-mode asserted-head
  route as Task 263ZZM in the same owner. Preserve its token-identical body,
  byte stability, five-expansion object-terminal same-mode behavior, and fail-
  closed behavior; it is complete. Fresh inventory returns to the remaining
  local-object-mode asserted-head routes and selects the exact five-fragment/
  73-line three-edge local-object-mode same-mode asserted-head route as Task
  263ZZN in the same owner. Preserve its token-identical body, byte stability,
  four-expansion object-terminal same-mode behavior, and fail-closed behavior;
  it is complete. Fresh inventory finds no physical local-object-mode asserted-
  head route left in `runner.rs` and returns to the remaining production-helper
  families. It selects the exact five-fragment/53-line direct local-mode same-
  mode asserted-head route as Task 263ZZO in the same owner. Preserve its token-
  identical body, byte stability, one-expansion set-terminal same-mode behavior,
  and fail-closed behavior; it is complete. Fresh inventory returns to the
  remaining production-helper families and selects the exact five-fragment/62-
  line chained local-mode same-mode asserted-head route as Task 263ZZP in the
  same owner. Preserve its token-identical body, byte stability, two-expansion
  set-terminal same-mode behavior, and fail-closed behavior without moving its
  immediate-radix sibling; it is complete. Fresh inventory returns to the
  remaining production-helper families and selects the exact five-fragment/61-
  line chained local-mode immediate-radix asserted-head route as Task 263ZZQ in
  the same owner. Preserve its token-identical body, byte stability, two-
  expansion set-terminal immediate-radix behavior, and fail-closed behavior
  without moving its two-edge sibling; it is complete. Fresh inventory returns
  to the remaining production-helper families and selects the exact five-
  fragment/66-line two-edge local-mode immediate-radix asserted-head route as
  Task 263ZZR in the same owner. Preserve its token-identical body, byte
  stability, three-expansion set-terminal immediate-radix behavior, and fail-
  closed behavior without moving its two-hop sibling; it is complete. Fresh
  inventory returns to the remaining production-helper families and selects
  the exact five-fragment/67-line two-edge local-mode two-hop asserted-head
  route as Task 263ZZS in the same owner. Preserve its token-identical body,
  byte stability, three-expansion set-terminal two-hop behavior, and fail-
  closed behavior without moving its three-edge sibling; it is complete. Fresh
  inventory returns to the remaining production-helper families and selects
  the exact five-fragment/72-line three-edge local-mode two-hop asserted-head
  route as Task 263ZZT in the same owner. Preserve its token-identical body,
  byte stability, four-expansion set-terminal two-hop behavior, and fail-closed
  behavior without moving its four-edge sibling; it is complete. Fresh
  inventory returns to the remaining production-helper families and selects
  the exact five-fragment/77-line four-edge local-mode two-hop asserted-head
  route as Task 263ZZU in the same owner. Preserve its token-identical body,
  byte stability, five-expansion set-terminal two-hop behavior, and fail-closed
  behavior without moving a three-hop or other route; it is complete. Fresh
  inventory returns to the remaining production-helper families and selects
  the exact five-fragment/75-line three-edge local-mode three-hop asserted-head
  route as Task 263ZZV in the same owner. Preserve its token-identical body,
  byte stability, four-expansion set-terminal three-hop behavior, and fail-
  closed behavior without moving its four-edge or other siblings; it is
  complete. Fresh inventory returns to the remaining production-helper
  families and selects the exact five-fragment/80-line four-edge local-mode
  three-hop asserted-head route as Task 263ZZW in the same owner. Preserve its
  token-identical body, byte stability, five-expansion set-terminal three-hop
  behavior, and fail-closed behavior without moving its four-hop or other
  siblings; it is complete. Fresh inventory returns to the remaining
  production-helper families and selects the exact five-fragment/79-line four-
  edge local-mode four-hop asserted-head route as Task 263ZZX in the same owner.
  Preserve its token-identical body, byte stability, five-expansion set-terminal
  four-hop behavior, and fail-closed behavior without moving another route;
  it is complete. Fresh inventory returns to the remaining production-helper
  families and selects the exact five-fragment/47-line direct builtin-set
  reserved-variable type-assertion route as Task 263ZZY in the same owner.
  Preserve its token-identical body, byte stability, independent reserve and
  formula-side source provenance, normalized-reflexive builtin-set behavior,
  and fail-closed behavior without moving its builtin-object, local-mode, or
  other siblings; it is complete. Fresh inventory returns to the remaining
  production-helper families and selects the exact 10-line shared term/formula
  diagnostic-key projection as Task 263ZZZ in existing private
  `type_elaboration/output.rs`. Preserve its token-identical body, canonical
  diagnostic traversal, prefix, sorting, deduplication, byte stability, and
  nine existing parent consumers without moving a wrapper or changing any
  key, diagnostic, payload, or fail-closed behavior; it is complete. Fresh
  inventory returns to the remaining production-helper families and selects
  the exact five-fragment/47-line direct builtin-object reserved-variable type-
  assertion route as Task 263ZZZA in existing private
  `type_elaboration/type_assertion_routes.rs`. Preserve its token-identical
  body, byte stability, independent reserve and formula-side source provenance,
  normalized-reflexive builtin-object behavior, and fail-closed behavior
  without moving its builtin-set, local-mode, chained, or other siblings; it is
  complete. Fresh inventory returns to the remaining production-helper
  families and selects the exact two-fragment/28-line standalone contradiction
  formula output/detail family as Task 263ZZZB in existing private
  `type_elaboration/output.rs`. Preserve its token-identical bodies, byte
  stability, exact checked contradiction payload, empty diagnostics/deferred/
  facts, one normal detail consumer, and test-only output consumers without
  moving another formula family or route; it is complete. Fresh inventory
  returns to the remaining production-helper families and selects the exact
  two-fragment/30-line formula-statement output/detail family as Task 263ZZZC
  in existing private `type_elaboration/output.rs`. Preserve its token-
  identical bodies, byte stability, partial thesis payload, one missing-formula
  deferred reason and diagnostic, normal detail consumer, and test-only output/
  extractor consumers without moving another formula family or route; it is
  complete with all preservation gates passing. Task 263 keeps this parent item
  open. Fresh inventory selects the exact 35-line inline builtin-binary term/
  formula checker/detail producer as Task 263ZZZD in existing private
  `type_elaboration/output.rs`. Preserve its token-identical body, byte
  stability, two ordered numeral terms, source-selected equality/inequality/
  membership formula, ordered/deduplicated diagnostics, normal detail consumer,
  and test-only extractor consumers without moving another formula family or
  route; it is complete with all preservation gates passing. Task 263 remains
  open. Fresh inventory selects the exact two-fragment/43-line builtin type-
  assertion formula output/detail family as Task 263ZZZE in existing private
  `type_elaboration/output.rs`. Preserve token-identical bodies, byte stability,
  source-derived numeral/formula/asserted-type payloads, type-entry ownership,
  normalized builtin-set type, diagnostic ordering, normal detail consumer,
  and test-only output/extractor consumers; it is complete with all
  preservation gates passing. Task 263 remains open. Fresh inventory selects
  the exact five-fragment/52-line direct local-mode reserved-variable type-
  assertion route as Task 263ZZZF in existing private
  `type_elaboration/type_assertion_routes.rs`. Preserve token-identical bodies,
  key/test alias, one real expansion, normalized-reflexive Task138 output,
  normal detail, test-only config/output/extractor, and all fail-closed/isolation
  boundaries. It is complete with all preservation gates passing. Task 263
  remains open. Corrected fresh inventory selects the exact 29-line shared
  imported-attribute assertion checker-output core as Task 263ZZZG in existing
  private `type_elaboration/output.rs`. Preserve its token-identical body,
  shared Task113/114 numeral/attribute-assertion payload, context, deferred
  reason, diagnostics, both retained wrappers, normal parent-only visibility,
  and all fail-closed/isolation boundaries. It is complete with all
  preservation gates passing. Task 263 remains open pending fresh inventory.
  Fresh inventory selects the exact 8-line positive imported-attribute
  assertion output wrapper as Task 263ZZZH in existing private
  `type_elaboration/output.rs`. Preserve token-identical positive extractor
  selection and payload forwarding into the moved shared core, normal parent-
  only visibility, retained detail/non-empty wrappers, exact diagnostics, and
  all fail-closed/isolation boundaries. It is complete with all preservation
  gates passing. Task 263 remains open pending fresh inventory.
  Fresh inventory selects the exact 8-line positive imported-attribute
  assertion detail wrapper as Task 263ZZZI in existing private
  `type_elaboration/output.rs`. Preserve token-identical output-to-canonical-
  key projection, normal detail visibility, test-only output/extractor
  crossings, retained non-empty family, exact diagnostics, and all fail-closed/
  isolation boundaries. It is complete with all preservation gates passing.
  Task 263 remains open pending fresh inventory.
  Fresh inventory selects the exact 9-line attribute-level-negative imported-
  attribute assertion output wrapper as Task 263ZZZJ in existing private
  `type_elaboration/output.rs`. Preserve token-identical direct-`non` extractor
  selection and payload forwarding into the shared core, normal parent-only
  visibility, the retained detail wrapper, exact diagnostics, and all fail-
  closed/isolation boundaries. It is complete with all preservation gates
  passing. Task 263 remains open pending fresh inventory.
  Fresh inventory selects the exact 9-line attribute-level-negative imported-
  attribute assertion detail wrapper as Task 263ZZZK in existing private
  `type_elaboration/output.rs`. Preserve token-identical output-to-canonical-
  key projection, normal detail visibility, test-only output/extractor
  crossings, exact diagnostics, and all fail-closed/isolation boundaries. It is
  complete with all preservation gates passing. Task 263 remains open pending
  fresh inventory.
  Corrected fresh inventory selects the exact 43-line set-enumeration checker-
  output producer as Task 263ZZZL in existing private
  `type_elaboration/output.rs`. Preserve token-identical four ordered numeral
  items, two ordered set-enumeration terms, equality formula, context, payload/
  status/diagnostics,
  normal parent-only visibility, retained detail wrapper, and all fail-closed/
  isolation boundaries. It is complete with all preservation gates passing.
  Task 263 remains open pending fresh inventory.
  Fresh inventory selects the exact 8-line set-enumeration formula detail
  wrapper as Task 263ZZZM in existing private
  `type_elaboration/output.rs`. Preserve token-identical output-to-canonical-
  key projection, normal detail visibility, test-only output and extractor
  crossings, exact diagnostics, and every fail-closed/
  isolation boundary. It is complete with all preservation gates passing.
  Task 263 remains open pending fresh inventory.
  Fresh inventory selects the exact 49-line imported predicate/functor
  checker-output producer as Task 263ZZZN in existing private
  `type_elaboration/output.rs`. Preserve token-identical ordered inputs,
  imported functor reference and both symbol provenances, predicate formula,
  context, payload/status/diagnostics, normal producer visibility, test-only
  extractor crossing, retained detail/connective families, and every fail-
  closed/isolation boundary. It is complete with all preservation gates
  passing. Task 263 remains open pending fresh inventory.
  Fresh inventory selects the exact 8-line imported predicate/functor formula
  detail wrapper as Task 263ZZZO in existing private
  `type_elaboration/output.rs`. Preserve token-identical output-to-canonical-
  key projection, normal detail visibility, test-only output/extractor
  crossings, exact diagnostics, and every fail-closed/isolation boundary. It
  is complete with all preservation gates passing. Task 263 remains open
  pending fresh inventory.
  Fresh inventory selects the exact 52-line formula connective/quantifier
  checker-output producer as Task 263ZZZP in existing private
  `type_elaboration/output.rs`. Preserve token-identical five ordered formula
  shells, contexts, deferred reasons, payload/status/diagnostics, normal
  producer visibility, test-only extractor crossing, retained detail, and
  every fail-closed/isolation boundary. It is complete with all preservation
  gates passing. Task 263 remains open pending fresh inventory.
  Fresh inventory selects the exact 8-line formula connective/quantifier detail
  wrapper as Task 263ZZZQ in private `output.rs`. Preserve exact key projection,
  normal detail visibility, test-only output/extractor crossings, diagnostics,
  and fail-closed/isolation behavior. It is complete with all preservation
  gates passing; Task 263 remains open pending fresh inventory.
  Fresh inventory selects the exact five-fragment/62-line chained local-mode
  reserved-variable type-assertion route as Task 263ZZZR in existing private
  `type_elaboration/type_assertion_routes.rs`. Preserve the leaf-private key,
  config-derived test alias, normal detail route, test-only config/output/
  extractor crossings, exact Task 146 normalization and provenance, and every
  fail-closed/isolation boundary. It is complete with every preservation gate
  passing; Task 263 remains open pending fresh inventory.
  Fresh inventory selects the exact five-fragment/67-line two-edge local-mode
  reserved-variable builtin type-assertion route as Task 263ZZZS in private
  `type_elaboration/type_assertion_routes.rs`. Preserve the leaf-private key,
  normal detail, test-only config/output/extractor, exact Task 148 provenance/
  normalization, and all fail-closed/isolation boundaries. It is complete with
  every preservation gate passing; Task 263 remains open pending fresh
  inventory.
  Fresh inventory selects the exact five-fragment/67-line Task 186 two-edge
  local-mode same-mode asserted-head route as Task 263ZZZT in private
  `type_elaboration/type_assertion_routes.rs`. Preserve the leaf-private key,
  config-derived test alias, normal detail route, test-only config/output/
  extractor crossings, exact same-Outer relation and normalization/provenance,
  and every fail-closed/isolation boundary. It is complete with every
  preservation gate passing; Task 263 remains open pending fresh inventory.
  Fresh inventory selects the exact five-fragment/71-line Task 205 three-edge
  local-mode immediate-radix asserted-head route as Task 263ZZZU in private
  `type_elaboration/type_assertion_routes.rs`. Preserve the leaf-private key,
  config-derived test alias, normal detail route, test-only config/output/
  extractor crossings, exact immediate-radix relation and normalization/
  provenance, and every fail-closed/isolation boundary. It is complete with
  every preservation gate passing; Task 263 remains open pending fresh
  inventory.
  Fresh inventory selects the exact five-fragment/73-line Task 150 three-edge
  local-mode reserved-variable builtin type-assertion route as Task 263ZZZV in
  private `type_elaboration/type_assertion_routes.rs`; it wins the 73-line tie
  by the smaller consumer surface. Preserve the leaf-private key,
  config-derived test alias, normal detail route, test-only config/output/
  extractor crossings, exact builtin relation and normalization/provenance,
  and every fail-closed/isolation boundary. It is complete with every
  preservation gate passing; Task 263 remains open pending fresh inventory.
  Fresh inventory selects the exact five-fragment/73-line Task 195 three-edge
  local-mode same-mode asserted-head route as Task 263ZZZW in private
  `type_elaboration/type_assertion_routes.rs`. Preserve the leaf-private key,
  config-derived test alias, normal detail route, test-only config/output/
  extractor crossings, exact same-Outer relation and normalization/provenance,
  and every fail-closed/isolation boundary. It is complete with every
  preservation gate passing; Task 263 remains open pending fresh inventory.
  Fresh inventory selects the exact five-fragment/76-line Task 207 four-edge
  local-mode immediate-radix asserted-head route as Task 263ZZZX in private
  `type_elaboration/type_assertion_routes.rs`. Preserve the leaf-private key,
  config-derived test alias, normal detail route, test-only config/output/
  extractor crossings, exact immediate-radix relation and normalization/
  provenance, and every fail-closed/isolation boundary. It is complete with
  every preservation gate passing; Task 263 remains open pending fresh
  inventory.
  Corrected fresh inventory selects the exact five-fragment/76-line Task 152
  four-edge local-mode reserved-variable builtin type-assertion route as Task
  263ZZZY in private `type_elaboration/type_assertion_routes.rs`. Preserve the
  leaf-private key, config-derived test alias, normal detail route, test-only
  config/output/extractor crossings, exact builtin relation and five-expansion
  normalization/provenance, and every fail-closed/isolation boundary. It is
  complete with every preservation gate passing; Task 263 remains open pending
  fresh inventory.
  Fresh inventory finds the exact five-fragment/78-line Task 197 four-edge
  local-mode same-mode asserted-head route as the sole remaining production-
  helper family and selects it as Task 263ZZZZ in private
  `type_elaboration/type_assertion_routes.rs`. Preserve the leaf-private key,
  config-derived test alias, normal detail route, test-only config/output/
  extractor crossings, exact same-TooDeep relation and five-expansion
  normalization/provenance, and every fail-closed/isolation boundary. It is
  complete with every preservation gate passing. Fresh production-helper
  inventory finds only top-level dispatch/orchestration in `runner.rs`; Task
  263 is complete and the series advances to separate Task 264 closeout.
- [x] Close out the series by synchronizing the paired source-layout inventory,
  crate plan, todo, harness/source-path tables, and ownership guards. Confirm
  that active runner 188, plan 403/367, type-elaboration 235/223, pass/fail
  219/184, all 272 discovered unit tests, expectation/trace credit, and all
  existing `.miz` intent remain unchanged before fresh inventory resumes
  Step 5. Task 264 is complete: the paired final inventory records 17
  production runner paths/18,952 lines, path/content manifest hashes
  `b36d96fe...`/`62d30627...`, eleven private type-elaboration leaves, and a
  facade/top-level-orchestration-only `runner.rs`. All preservation counts,
  four CLI hashes, and raw/normalized test-list hashes remain unchanged;
  `spec_coverage_audit.md` remains unchanged. Fresh canonical Step 5 inventory
  finds no next nonempty unchecked task. Steps 6/7 remain deferred.
- [x] **Task 265: formalize Step 5 execution authority.** Perform a fresh
  canonical inventory and assign every remaining family either a concrete
  owner task or a nonempty owner-owned decomposition task. Synchronize the
  top-level roadmap, paired owner plans/TODOs and current-state audits,
  traceability deferred ownership, and the specification coverage audit.
  Change no source, language semantics, `.miz` fixture, expectation, trace
  status/test list, runner count, or coverage credit. The resulting dependency
  graph is Task 266 -> Task 267 -> Task 268; Tasks 266 + 268 -> mizar-core Task
  31; checker Task 247 -> core Task 32; Core Tasks 31 + 32 -> mizar-vc Task 30
  -> VC 31. Parser Tasks 47-48 and resolver Task 31
  are independently authorized checker-Task-49 prerequisites, not Task-266
  dependencies. Checker Task 247, core Task 32, and VC Task 30 exhaustively own
  the remaining checker, CoreIr/ControlFlowIr, and VC/obligation family
  decomposition without fabricating payloads. Steps 6/7 stay
  deferred. Inventory classifications: the missing executable decomposition is
  `design_drift`; the exact Task-180 final-handoff, property-implementation,
  same-return conflict, Core, and VC gaps are `source_drift` and `test_gap`;
  Task-47 recovery is `test_expectation_drift` plus `source_drift`; rebuilding
  another crate's raw syntax downstream would be a `boundary_violation` and is
  forbidden. Task 265 found no new or blocking `spec_gap` in its selected
  execution-authority slice; the pre-existing MC-G005 public-code allocation
  `spec_gap` remained explicit. No `source_undocumented_behavior` or
  `repo_metadata_conflict` was found.
  Checker Task 247 has now completed the authorized docs/traceability split:
  Tasks 248-264/269-279, Task-10 increments `MT10-FS`/`MT10-AS`, and existing
  Task 49 own the remaining families. Resolver Task 31 solely activates the
  same-return member through `declaration_symbol`; Task 49 activates the other
  23 and reconciles/deduplicates the exact 24-fixture set. Task 274 and external scheme/theorem-role
  Gate S1 are explicit blocked gates, so Task 49 is not yet executable. Core
  Task 32 is now docs-decomposition-authorized.
- [x] **Task 266: preserve the exact Task-180 checked contradiction in the
  final checker handoff.** Extend checker-owned, syntax-free
  `ResolvedTypedAst` data so one resolver theorem owner is linked to the one
  existing checked `FormulaKind::Contradiction` result for
  `SourceDerivedContradictionConstantBoundary`, preserving owner/formula
  identities, source ranges, state, and provenance. `mizar-test` continues to
  own real AST extraction and exact active-runner assertions; checker owns the
  final semantic identity and validation. Reject missing, duplicate,
  reordered, recovered, or mismatched owner/formula rows. Reuse the existing
  `.miz` and expectation unchanged; add checker and runner unit/corruption/
  determinism tests and keep four CLI outputs byte-stable. Do not publish
  falsehood/facts, accept the theorem, create proof status/skeleton/terminal
  goals, lower Core/CFG/VC payloads, broaden formula shapes, or promote a
  runner stage. Deps: Task 265 and checker Task 180. Specs: 14 and 16.
- [x] **Task 267: decide the omitted-justification theorem handoff contract.**
  In paired checker/core design documents, specify the checker-owned
  pending-auto-proof status, proof skeleton, explicit terminal-goal payload,
  source/provenance links, malformed/missing behavior, and exact mapping into
  core types for an ordinary theorem with no written justification. This is a
  docs-only task; it must not equate omitted justification with accepted proof,
  infer a terminal goal from raw syntax inside core, run proof search, or edit
  fixtures/expectations/trace status. Deps: Task 266. Specs: 15 and 16;
  architecture 06.
  Complete: explicit `Unmodified`/`Omitted` intent maps to one distinct
  `PendingAutomaticProof`, one direct terminal goal, and the exact future
  `False`/Active `TheoremProof` core seed at `proof/0`; corrupt input fails
  atomically and no acceptance credit is assigned.
- [x] **Task 268: implement the accepted Task-267 checker producer.** Extend
  only the exact Task-180 final handoff with the Task-267 proof status,
  skeleton, and terminal-goal payload. Add fail-closed checker/runner tests for
  missing, duplicate, reordered, corrupt, and owner/formula/proof mismatch;
  assert deterministic nonempty debug rendering for all three proof tables and
  byte-identical Task-266 debug output when they are empty;
  keep theorem acceptance, discharge, Core/VC generation, broader proof forms,
  existing expectation changes, and Steps 6/7 outside scope. Deps: Task 267.
  Complete: the exact extractor emits explicit intent only for the unannotated,
  unjustified, proof-block-free Task-180 theorem; checker/runner corruption and
  immutable output assertions pass. Existing fixture, expectation, runner
  admission, counts, and CLI bytes remain unchanged. Core Task 31 is next.

## VC Task 30 / Task-10 Consumer Ownership

VC Task 30 reserves `MT10-VC-T180` solely for VC Task 31. It uses a distinct
Task-180-shaped theorem source/sidecar at `proof_verification` /
`active_proof_verification`, `expected_phase = "vc_generation"`, phase 11, and
compares complete deterministic `SnapshotKind::VcIr` / `VcSet::debug_text()`
bytes. The existing type-elaboration Task-180 source, sidecar, and Core snapshot
must remain unchanged. The first proof-verification runner/tag/guard change
lands with this first real baseline, never as empty infrastructure.

VC Tasks 32-55 share `MT10-VC-PV`; each owns a distinct
`MT10-VC-PV/VC<n>` source/sidecar/trace/baseline slice. VC 40 remains
unexecuted behind completed VC 37/39 outputs plus Core 40/A1; VC 53 remains
unexecuted because canonical authority does not name its evidence producer/
reference schema/authentication contract/tests. Missing scheme/theorem-role slices
remain outside direct VC 41 behind S1. Task 30 changes no runner, case, expectation, trace
status/test, count, hash, or coverage.

## VC Task 31 / Task-10 Consumer Completion

The exact `MT10-VC-T180` increment is complete. The distinct
`pass_proof_verification_contradiction_formula_constant_001` sidecar is the
only active proof-verification case, uses phase `vc_generation`, and carries
the complete VcIr snapshot. The public runner/report/CLI executes the exact
source-to-checker-to-Core-to-VC path twice, fails closed on admission,
generation, or baseline errors, and reports one passing result. Plan counts are
404/369, proof-verification coverage is 4/1, and pass/fail is 220/184; the
parse/declaration/type active counts remain 96/4/188.

The existing type-elaboration Task-180 source/sidecar/Core snapshot remains
unchanged. Broad proof-verification, VC 32-55, discharge, ATP/kernel/proof
acceptance, facts, and Steps 6/7 remain deferred or dependency-paced.

## Resolver Task 31 / Declaration-Symbol Completion

The exact same-return increment is complete. The unchanged
`fail_resolve_same_signature_same_return_conflict_001.miz` source and activated
sidecar now execute through the existing real frontend/resolver runner and
observe `declaration_symbol.signature.same_signature_definition_conflict`.
Declaration-symbol admission is five cases; plan 404/369, parse 96, type 188,
proof 1, and pass/fail 220/184 remain unchanged. The different-return sidecar
and its existing detail key remain byte-identical. No other Task-49 member,
semantic overload behavior, public code, or Steps 6/7 status changed.

For every source-moving task, require review-only checks for visibility drift,
test-discovery drift, owner-boundary drift, source/docs inconsistency, and
accidental behavior changes. Run focused tests, `cargo test -p mizar-test`,
`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
workspace `cargo test`, and `git diff --check`; repair and rerun failures until
all commands pass. A test or verification failure is not itself a reason to
stop this series.

## Parser Task 47 / Parse-Only Completion

One spec-derived pass fixture activates only the omitted-justification and
proof-block `reconsider` rows. The existing mixed recovery `.miz` remains
unchanged; its sidecar drops only the obsolete omitted-tail parser diagnostic.
The active plan is 405/369, parse-only is 97, and pass/fail is 221/184.
Declaration/type/proof admissions remain 5/188/1. Semantic reconsider
acceptance and E0102 production remain deferred to their owning checker tasks;
Parser Task 48 and Steps 6/7 are not promoted.

## Parser Task 48 / Property-Implementation Parse-Only Completion

The authorized nonempty Task-48 slice is complete. The new pass/fail corpus
pair runs the dedicated top-level property-implementation grammar through the
real parse-only runner, and the exact
`spec.en.07.modes.property_implementation.parser` row is `covered` with
`pass_and_fail`. Active totals are plan 407/369, parse-only 99/99, pass/fail
222/185, and warnings/errors 23/0; declaration/type/proof admissions remain
5/188/1.

This completion grants parser/syntax-only credit. Property payload extraction,
semantic overlap/coherence handling, proof acceptance/discharge, and inactive
semantic Task 39 remain unchanged. No checker task or Steps 6/7 authority is
promoted by this increment.

## Checker Task 248 / Task-10 Consumer Completion

- [x] Add the exact active reserve-plus-definition-parameter shadowing fixture,
  its single-reference sidecar, and the bounded covered trace row.
- [x] Match the real resolver shells and source walk, pass only syntax-free
  item/declaration/context payloads into `mizar-checker`, and preserve the
  complete handoff from `TypedAst` to `ResolvedTypedAst`.
- [x] Assert distinct same-spelling binding identities, structural shadowing,
  deterministic debug output, and zero later semantic payloads; keep term-use
  selection and Tasks 249+/269+ outside this increment.
- [x] Update chapter-level coverage because this task changes exact executable
  coverage; keep the broad payload-extraction row and Steps 6/7 unchanged.

## Checker Task 249 Frozen Consumer Prerequisite

- [x] Freeze the future exact ten-reserve-root broad fail consumer and its
  10/13/6 handoff oracle without adding the fixture, sidecar, or trace row yet.
- [x] Freeze the existing Task-248 pass route as the dependency regression:
  unchanged source/sidecar/trace, exactly two `Bare`/builtin-`set` rows linked
  to its two existing bindings, and zero arguments.
- [x] Keep the sole pending key runner-owned and every checker semantic result
  table empty; preserve Tasks 68-71 byte-for-byte.
- [x] Implement Checker Task 249 as one logical task. The exact broad 10/13/6
  route and unchanged Task-248 2/2/0 co-consumer execute through the immutable
  checker handoff; plan 411/372, type 238/226, pass/fail 224/187, active type
  190, and fresh hashes are mandatory completion oracles. Tasks 250+, 269+,
  and Steps 6/7 remain unpromoted.

## Checker Task 250 Frozen Consumer Prerequisite

- [x] Freeze exactly the existing Task-81/67/84/85 active fail fixtures as the
  only real consumers, with one Task-249 application/root and no type argument
  per route.
- [x] Freeze the aggregate Task-250 oracle at four nonempty chains, four
  attributes, one qualifier, one parenthesized argument group, and one actual;
  preserve exact polarity and local/imported provenance.
- [x] Freeze Task-81/67 runner-only pending progression and Task-84/85 existing
  evidence-query preservation, with no new `.miz`, broad expectation
  rebaseline, semantic result, or public diagnostic.
- [x] Require private synthetic-`SurfaceAst` extractor coverage for
  multi-attribute order and single/parenthesized prefix projection, plus the
  checker corruption/determinism matrix.
- [x] Implement Checker Task 250 as one logical task: the private
  `source_attribute` leaf executes the exact four real consumers and the
  synthetic prefix probe through the public checker handoff; plan 411/373 and
  type 239/227 are reached with unchanged pass/fail and admissions. Tasks
  251+/269+ and Steps 6/7 remain unpromoted.

## Checker Task 251 Frozen Consumer Prerequisite

- [x] Freeze exactly the Task-249 broad fixture plus Task-84/85 as the
  representative real selector; add no `.miz` and keep all siblings
  byte-identical.
- [x] Freeze ten missing requests with histogram 5 mode-expansion /
  3 structure-inhabitation /
  2 attributed, combined Task-249 12/15/6, Task-250 2/2/0/0/0, and zero
  dependency references.
- [x] Freeze broad-only progression to the missing-dependency detail while
  preserving Task-84/85 evidence-query details and every outcome/public code.
- [x] Require requested/missing/rejected/supplied injection through the real
  AST and production Task-10 consumer to final `TypedAst`/`ResolvedTypedAst`;
  supplied input is not accepted evidence and corruption is atomic failure.
- [x] Implement Checker Task 251 as one separate logical task. Reach plan
  411/374 and type 240/228 with unchanged pass/fail/admissions/warnings,
  exact isolation, full hashes, reviews, and one dedicated commit.

## Checker Task 252 Frozen Consumer Prerequisite

- [x] Freeze exactly the existing builtin numeral equality, bare
  reserved-variable equality, and single-left-parenthesized reserved-variable
  equality routes, with no new `.miz` or outcome/detail change.
- [x] Freeze aggregate term/reference/numeric-request cardinalities 7/4/2 and
  the source-only parent edge; parentheses add no semantic type, term, fact,
  axiom, or FOL row.
- [x] Freeze synthetic same-producer coverage for a
  `LocalAbbreviation` constant, `it` current-result role, and nested
  parentheses without taking Task-269 local-binding or Tasks-260/264
  definition ownership.
- [x] Require transactional final ownership, full corruption/determinism and
  isolation checks, one bounded three-sidecar trace reference, and the
  no-new-case implementation oracle plan 411/375 and type 241/229.
- [x] Correct the post-freeze ordinal contract to count preceding completed
  binding rows, retain duplicate-priority groups for reachable `Ambiguous`
  rejection, and record `Resolver` as structurally unreachable.
- [x] Implement Checker Task 252 as one separate logical task. Preserve
  pass/fail 224/187, admissions 101/5/190/1, warnings/errors 23/0; reach 291
  library tests and the verified 23-path/24,120-line layout with fresh hashes.
  Tasks 253+/260/264/269 and Steps 6/7 remain unpromoted.

## Checker Task 253 Frozen Consumer Prerequisite

- [x] Distinguish Checker Task 253 from the already-completed `mizar-test`
  runner-refactor Tasks 253A/253B.
- [x] Freeze the existing imported `1 ++ 2` route plus one exact new
  spec-derived module-local `task253_local_source(x)` second-definiens fail
  route as the only real consumers.
- [x] Freeze the new source as reserve `x`, then one definition with two
  functor declarations sharing inner parameter `x`; reuse the Task-248
  two-item/two-binding shadow handoff and require the actual to reference
  `BindingId(1)` / `BindingContextId(1)` / `use_ordinal == 2`.
- [x] Freeze aggregate Task-253 application/wrapper/candidate/argument/request
  tables 2/1/2/3/4 and referenced Task-252 primary/reference/numeric slice
  3/1/2, without duplicate primary ownership.
- [x] Freeze the Task-253 transparent-wrapper origin for `(1 ++ 2)` and
  individually authenticated candidate references without claiming
  completeness, viability, ranking, or a winner.
- [x] Freeze synthetic ordinary/nested/parenthesized/candidate coverage;
  inline zero/one/two-actual source-schema coverage only; and template
  whole-subtree exclusion.
- [x] Assign inline identity/formals/capture/substitution to Task 270,
  template direct roles/actuals/guards/requests to Task 277, and
  ordinary/template candidate collection and selection to Task 278.
- [x] Preserve the imported outcome/detail and freeze the new local sidecar at
  `definition_declaration_payload_extraction_gap` /
  `type_elaboration.external_dependency.ast_payload_extraction`, with no
  public diagnostic.
- [x] Implement Checker Task 253 as one separate logical task, add exactly the
  new fixture/sidecar and bounded diagnostic trace row, preserve the imported
  outcome/detail, and reach measured 412/376, 242/230, 224/188, and
  101/5/191/1 oracles with 303 library tests. The paired completion documents
  record the 24-path/25,607-line manifest and exact five-CLI, test-list, and
  production hashes. Tasks 254+ and Steps 6/7 remain unpromoted.

## Checker Task 254 Frozen Consumer Prerequisite

- [x] Freeze exactly one new spec-derived local structure-term fail source
  with one `Task254Pair` declaration and three definientia for construction,
  selector access, and functional update.
- [x] Freeze the real Task-254
  term/wrapper/root/member/field-update/edge/request oracle
  5/0/3/9/2/10/26 and the composed Task-252
  primary/reference/numeric-request slice 8/0/8, with no real Task-253 row.
- [x] Confine raw constructor/selector/update, member, `FieldUpdate`, wrapper,
  and edge extraction to one private runner leaf; keep the checker handoff
  syntax-free.
- [x] Preserve repeated labels and paths in source order, represent nested
  paths as member chains, and assign no independent term/type/fact to a
  `FieldUpdate`.
- [x] Freeze one-way same-context Task-252 roots, Task-253 root applications
  not targeted by another Task-253 argument edge, and Task-254 child
  composition; reject nested Task-253 targets and keep whole-subtree exclusion
  for reverse Task-253 applications, Task-255 terms, templates, and initial
  type-argument-bearing constructors.
- [x] Freeze the sidecar at
  `definition_declaration_payload_extraction_gap` /
  `type_elaboration.external_dependency.ast_payload_extraction` with no
  public diagnostic, leaving structure definition/member/view and all
  semantic decisions to Task 263.
- [x] Keep this prerequisite documentation-only: no fixture, sidecar, trace
  row/status/count, runner route, test list, production source, or executable
  credit changes; preserve the 412/376, 242/230, 224/188,
  101/5/191/1, 303-test, and 24-path/25,607-line baselines.
- [x] Implement Checker Task 254 as a separate logical task. The exact
  fixture/sidecar, bounded requirement, Chapter-5/13 transport-only widening,
  four reciprocal backlinks, Task-248 context reuse, complete
  real/synthetic/exclusion/corruption/final-ownership matrix, and measured
  413/377, 243/231, 224/189, and 101/5/192/1 oracles are complete.

## Checker Task 255 Frozen Consumer Prerequisite

- [x] Freeze exactly one future local-definition fail source with enumeration,
  condition-free comprehension, choice, and `qua` definientia.
- [x] Freeze the private raw-syntax owner and public six-table syntax-free
  boundary at 4/0/1/3/4/7 plus Task-252 4/0/4, with no real Task-253/254
  target or fingerprint.
- [x] Preserve a written generator declaration without fabricating
  `BindingId`/capture; assign binding/capture to Task 257 and condition formulas
  to Tasks 256-257.
- [x] Admit only bare builtin `set`/`object` target sites and keep Task-249
  declaration-application ownership unchanged.
- [x] Keep this prerequisite documentation-only, preserving 413/377,
  243/231, 224/189, 101/5/192/1, 312 tests, 25 paths / 27,317 lines, and all
  hashes. The separate implementation projects 414/378, 244/232, 224/190,
  and active type 193 subject to fresh preflight.
- [x] Implement the separate Task-255 consumer with the exact fixture,
  sidecar and five reciprocal trace references, Task-248/252 composition,
  final six-table 4/0/1/3/4/7 plus 4/0/4 oracle, active-case isolation, and
  reviewed synthetic/exclusion/corruption/install-order coverage. Preserve
  the external dependency gap and leave binder/formula/semantic ownership
  deferred as frozen.

## Checker Task 256 Frozen Consumer Prerequisite

- [x] Freeze the exact eight existing active fail consumers without adding or
  changing a `.miz` source or any current outcome/detail field.
- [x] Freeze the private raw-syntax owner and public eight-table boundary at
  Task-256 `8/0/1/1/1/2/13/11`, Task-252 `16/0/16`,
  Task-253 `1/1/1/2/2`, and Task-255 `2/0/0/0/4/2`, with no real
  Task-254 target.
- [x] Freeze imported predicate/attribute provenance, bare formula-owned
  asserted types, source-anchored attribute polarity, nearest-family term
  ownership, conditional fingerprints, and eleven unresolved requests.
- [x] Freeze the Task-256-owned combined composition order: complete Task-252
  union first, Task-253/255 dependencies in the same handoff/arena next, with
  existing lower-family exact selectors and allowlists unchanged.
- [x] Require exact ordered positive vectors for all thirteen edges and
  eleven requests, including the Task-253 outer wrapper range and attribute
  target/`non` anchors, plus unchanged standalone selector-isolation oracles.
- [x] Keep predicate chains/negation, inline/templates, general asserted type
  graphs, qualified/argument-bearing attributes, semantic facts/truth, and
  conditioned comprehensions explicitly outside this bounded increment.
- [x] Keep this prerequisite documentation-only, preserving 414/378,
  244/232, 224/190, 101/5/193/1, 320 tests, 26 paths / 29,138 lines, and
  all hashes. The separate implementation projects 414/379 and 245/233 with
  unchanged case count, subject to fresh preflight.
- [x] Implement the separate Task-256 consumer, producer, final handoff,
  bounded reciprocal trace increment, and reviewed
  real/synthetic/exclusion/corruption/install-order matrix. The exact eight
  existing sources now exercise the syntax-free checker transaction while
  retaining every pre-existing semantic detail owner.

## Checker Task 257A Frozen Consumer Prerequisite

Checker Tasks 257A-C in this section are checker producer slices and are
unrelated to the completed mizar-test Tasks 257A-H test-layout series above.

- [x] Freeze the one unchanged connective/quantifier fail source, its exact
  five formula sites, binder segment/identifier/type site, and source ranges.
- [x] Freeze private ownership: extend the existing `source_formula.rs` raw
  extraction shape and use a dedicated private `source_composite_formula`
  assembler without widening lower-family selectors or allowlists.
- [x] Freeze the public seven-table `5/0/1/1/1/4/6` transaction and exact
  `2/1/4` binding environment, including the single context transition and
  resolver-shaped local binder identity.
- [x] Freeze the ordered formula/root/binder/type-site/edge/request oracle,
  unchanged two-key semantic detail vector, all-active isolation, and
  environment/table corruption/install/final-ownership coverage, including
  one full literal handoff debug snapshot, exact legacy debug bytes, and the
  executable preinstalled-source-context rejection.
- [x] Keep broader connectives/quantifiers, bound use/capture, predicate
  chains, conditioned comprehensions, theorem ownership, and all semantic
  answers outside Checker Task 257A.
- [x] Keep this prerequisite documentation-only, preserving 414/379,
  245/233, 224/190, 101/5/193/1, 287/328 checker/mizar-test tests,
  27 paths / 30,154 lines, and all hashes. The separate implementation
  projects 414/380 and 246/234 with unchanged case count.
- [x] Implement the separate Checker Task 257A selector extension, private assembler,
  public producer/binding prepass/final handoff, bounded reciprocal trace
  increment, and reviewed real/synthetic/exclusion/corruption/install matrix.
  The route preserves the unchanged two-key semantic detail vector and the
  corrected parser ranges `52..113`, `78..89`, `78..79`, and `86..89`.
  Checker Task 257B is the next dependency-ready slice.

## Checker Task 257B1 Frozen Consumer Prerequisite

- [x] Freeze the exact 79-byte specification-derived pass source and its
  universal/binder/type/equality/two-use ranges.
- [x] Freeze same-arena composition at Task-252 `2/2/0`, Task-256
  `1/0/0/0/0/0/2/2`, Task-257 `1/0/1/1/1/0/2`, and Task-257B1 `1/2`.
- [x] Preserve Task-252 reference ownership and require both references to
  select quantifier binding 0 in context 1; do not misuse captured-free-
  variable metadata.
- [x] Preserve Task-257A source-context exclusion: the combined installer
  keeps `source_context()` absent and atomically rejects a preinstalled
  Task-248 handoff.
- [x] Freeze ownership-partition tests: the legacy installer rejects B1, the
  combined installer rejects an AST with Task 257A, and both roll back
  byte-identically without partial publication.
- [x] Freeze profile-discriminator tests for A-cardinality/B-row hybrids, the
  inverse hybrids, and a third otherwise valid profile.
- [x] Keep semantic truth, theorem acceptance, broader connectives,
  existential/restricted/nested/implicit binders, predicate chains, and
  conditioned comprehensions outside Task 257B1.
- [x] Keep this prerequisite documentation-only at 414/380, 246/234,
  224/190, 101/5/193/1, 299/333 tests, and 28 paths / 30,654 lines.
- [x] Implement the exact consumer, second composite profile, lower-family
  composition, public `1/2` handoff, trace row, tests, and final ownership.
  The bounded pass route preserves all semantic deferrals; Checker Task 257B2
  is next.

## Checker Task 257B2 Frozen Runner Checklist

- [x] Freeze the exact 166-byte source, SHA-256, parser ranges, repeated flags,
  connective tokens, grouping wrappers, and exact private selector.
- [x] Freeze same-arena Task-252 `16/0/16`, Task-256
  `8/0/0/0/0/0/16/16`, Task-257B2 `8/6/1/1/1/7/9`, and
  composition `8/0`.
- [x] Freeze selector isolation, mutation/recovery, A/B1 preservation, final
  ownership, trace/count impact, and all semantic deferrals.
- [x] Keep this prerequisite documentation-only at 415/381, 247/235,
  225/190, active 101/5/194/1, 338 tests, and 29 paths / 31,374 lines.
- [x] Implement the exact route, sidecar, covered trace row, and tests after
  the documentation commit and fresh parser/resolver preflight.
- [x] Verify corpus `416/382`, type `248/236`, pass/fail `226/190`, active
  `101/5/195/1`, 343 library tests, and absence of semantic output.
- [x] Keep Task 257B3 unselected while its own EN/JA frozen contract is
  prepared; the contract is complete and implementation remains separate.

## Checker Task 257B3 Frozen Runner Checklist

- [x] Freeze the exact 138-byte source and hash, Task-48 reserve extraction,
  parser/resolver nodes/scopes/ranges as mandatory preflight facts, and the
  exact private selector boundary.
- [x] Freeze same-arena Task-252 `6/6/0`, Task-256
  `3/0/0/0/0/0/6/6`, Task-257B3 `3/0/1/3/3/2/6`, and composition
  `3/6`, including source-order lookup and owning-edge associations.
- [x] Keep Task-248 source context absent, validate the reserve-derived base,
  and freeze isolation, mutation, previous-route, final-ownership, and
  semantic-output tests.
- [x] Keep this prerequisite documentation-only at `416/382`, `248/236`,
  `226/190`, active `101/5/195/1`, 343 tests, and 29 paths /
  32,064 lines.
- [x] Implement the exact route, sidecar, covered trace row, and tests after
  the documentation commit and fresh preflight.
- [x] Verify corpus `417/383`, type `249/237`, active type `196`, 349
  library tests, exact selector isolation, and absence of semantic output.

## Checker Task 257C1 Frozen Runner Checklist

- [x] Freeze the exact 107-byte source/hash, two segment/head ranges, negative
  token ranges, same imported provenance, and loaded-source/final-LF guards.
- [x] Freeze same-arena Task-252 `3/0/3` and extended Task-256
  `1/0/2/2/2/0/0/3/2`, including one shared middle boundary edge.
- [x] Freeze exact source near misses, recovery/mixed-chain exclusion,
  corruption/isolation/install/final matrices, and empty semantic output.
- [x] Keep this prerequisite at `417/383`, `249/237`, `227/190`, active
  `101/5/196/1`, 349 tests, and 29 paths / 32,809 lines.
- [x] Implement the exact route, one fixture/sidecar, covered trace row, and
  tests after the documentation commit and fresh preflight.
- [x] Verify `418/384`, `250/238`, `228/190`, active `101/5/197/1`, 353
  tests, exact selector isolation, fail-closed corruption, and empty semantic
  output.

## Checker Task 255C1 Frozen Runner Checklist

- [x] Freeze the valid 191-byte source/hash, parser ranges, imported `++`
  provenance, loaded-source/final-LF selector, and future fail detail.
- [x] Freeze same-arena Task-252 `4/0/4`, Task-253 `1/0/1/2/2`, and
  Task-255 `1/0/1/1/1/1/2`, including the direct condition-wrapper anchor
  and untargeted condition operands.
- [x] Freeze reusable Task-253 ownership, near-miss/corruption/isolation,
  atomic install/clone, empty semantic output, and unchanged prior routes.
- [x] Keep this prerequisite documentation-only at `418/384`, `250/238`,
  `228/190`, active `101/5/197/1`, 353 tests, and 29 paths / 33,184 lines.
- [x] Implement the exact route, fail sidecar, covered trace row, and tests
  after the documentation commit and fresh preflight. The runner now measures
  `419/385`, `251/239`, `228/191`, active `101/5/198/1`, 357 library tests,
  and preserves empty semantic output and every prior route.

## Checker Task 257C2 Frozen Runner Checklist

- [x] Freeze reuse of the unchanged 191-byte fixture, hash, parser ranges,
  imported mapper provenance, and exact Task-252/253/255 profiles.
- [x] Freeze the reusable Task-256 equality builder, exact
  `1/0/0/0/0/0/0/2/2` profile, direct wrapper/equality ownership split, and
  same-arena association.
- [x] Freeze the dedicated one-edge Task-257C2 handoff, route order,
  mutation/near-miss/isolation/final-clone tests, bidirectional A/B/C2
  installer exclusion, and semantic exclusions.
- [x] Freeze reuse of the existing sidecar, one future trace row, unchanged
  419-case/pass-fail/active counts, projected plan `419/386` and type
  `252/240`, unchanged diagnostic intent, and zero executable artifact
  changes in this prerequisite.
- [x] Complete the separately documented Task-256C1 frozen-contract and
  checker-only implementation; both lower install orders now pass without a
  runner edit.
- [x] Implement only this frozen runner slice after Task-256C1 and fresh
  preflight, in the separate Task-257C2 implementation commit.

## Checker Task 256C1 Frozen Runner Checklist

- [x] Freeze runner non-ownership: no source, test, fixture, sidecar,
  expectation, trace, production manifest, or CLI change.
- [x] Preserve the 191-byte fixture only as authority and a future
  Task-257C2 consumer; checker-local tests own both install orders and
  corruption coverage.
- [x] Preserve `419/385`, `251/239`, `228/191`, active `101/5/198/1`,
  357 tests, 29 paths / 33,725 lines, and all runner hashes.
- [x] Keep this runner unchanged during Task 256C1 implementation and verify
  the checker-only lower gate; at the C1 exit, fresh preflight of the frozen
  Task-257C2 route was the next logical task.

## Checker Task 257C2 Implementation Checklist

- [x] Publish the exact five-profile same-arena route before lower
  diagnostic-only routes and retain the existing extraction-gap detail.
- [x] Add four runner tests for exact profiles/provenance/ownership,
  dependency and arena mutations, loaded-source/named near misses, active
  isolation, sidecar stability, replay, and final clone.
- [x] Add no fixture; update only the existing sidecar reference/note and the
  single covered trace row.
- [x] Measure `419/386`, `252/240`, `228/191`, active `101/5/198/1`,
  361 tests, and 29 production paths / 34,064 lines.

## Checker Task 257C3 Frozen Runner Checklist

- [x] Reuse the unchanged 107-byte Task-257C1 pass fixture/hash, exact
  ranges, final-LF guard, and imported `divides` provenance.
- [x] Freeze Task-252 `3/0/3`, Task-256
  `1/0/2/2/2/0/0/3/2`, and Task-257C3 `1/1` in one arena.
- [x] Freeze complete-route precedence over Task 257C1, named near misses,
  active isolation, corruption/arena rollback, replay, final clone, and empty
  semantic output.
- [x] Freeze reuse of the existing sidecar with one future reference/note and
  one future covered trace row; add no fixture or semantic expectation.
- [x] Keep this prerequisite at `419/386`, `252/240`, `228/191`, active
  `101/5/198/1`, 361 tests, and 29 paths / 34,064 lines.
- [x] Implement only the frozen route after the documentation commit and
  fresh parser/resolver/lower-stage/count/hash preflight.

## Checker Task 257C3 Implementation Checklist

- [x] Publish the exact complete route before the lower C1 route without
  changing the existing fixture or semantic detail.
- [x] Add exactly four runner tests for route/provenance/debug, near-miss
  isolation, all dependency/arena mutations, rollback/replay, and clone.
- [x] Update only the existing sidecar reference/note and one covered trace
  row.
- [x] Measure 365 tests and the 29-path / 34,290-line production manifest.

## Checker Task 258A Frozen Consumer Checklist

- [x] Freeze the exact 81-byte final-LF source/hash and real frontend/
  resolver library-test path without adding the future corpus fixture.
- [x] Freeze Task-48 binding, Task-252 `2/2/0`, Task-256
  `1/0/0/0/0/0/2/2`, and Task-258A `1/1/1/1/1` composition.
- [x] Freeze exact owner/label provenance, selector/subtree exclusions,
  owned BindingEnv/fingerprint, absent Task-248 owner, typed/resolved
  equality, empty semantic output, and active-route isolation.
- [x] Preserve all fixtures, sidecars, trace metadata/status/count, CLI
  counts/hashes, 365-test list, and 29-path / 34,290-line production manifest.
- [x] After the checker documentation commit and fresh preflight, add only
  the dormant production route and exactly four library tests; leave corpus
  activation to `MT10-FS`.
- [x] Measure 369 runner tests and the 30-path / 34,955-line production
  manifest; keep plan/type/active counts and trace metadata unchanged.

## Checker Task 258B1 Frozen Consumer Checklist

- [x] Decompose the old Task-258B runner umbrella and select only the
  139-byte nested equality/conclusion/local-citation source; freeze its final
  LF, hash, parser ranges, and resolver theorem/local-label provenance.
- [x] Freeze the exact shared Task-48 `3/1/0`, Task-252 `8/8/0`, Task-256
  `4/0/0/0/0/0/0/8/8`, Task-258B1 `1/4/4/4/4`, and reference `1/1`
  transaction without accepted facts or proof semantics.
- [x] Freeze private raw-syntax ownership, corpus-dormant precedence,
  the two-pass 77-node/root-76 resolver AST with sole resolved/keyed node 68,
  replayable resolver projection/reference/result, selector/subtree/
  provenance mutations, active-route isolation, final clone/replay, and
  exactly five future library tests.
- [x] Preserve every fixture, sidecar, expectation, trace row/status/count,
  active route, executable count, and Task-258A hash in this prerequisite.
- [x] After the checker documentation commit and fresh parser/resolver/lower
  preflight, implement only the dormant Task-258B1 route and five tests;
  leave Task 258B2+ and Tasks 269–272 deferred.
- [x] Measure 374 runner tests and the unchanged 30-path production topology
  at 35,854 lines; preserve all corpus/trace/CLI counts and hashes.

## Checker Task 258B2 Frozen Consumer Checklist

- [x] Decompose Task 258B2+ and freeze only the final-LF 113-byte
  single-assumption theorem, its hash, exact parser ranges, and theorem-only
  resolver provenance.
- [x] Freeze the Task-48 `2/1/0`, Task-252 `6/6/0`, Task-256
  `3/0/0/0/0/0/0/6/6`, and Task-258B2 `1/3/3/3/3` transaction with no
  reference association or semantic output.
- [x] Freeze private raw-syntax ownership, corpus-dormant precedence,
  selector/subtree and provenance mutations, Task-258A/B1 and active-route
  isolation, typed/final cloning, and exactly five future runner tests.
- [x] Preserve all fixture, sidecar, expectation, trace row/status/count,
  active route, source, 374-test list, 30-path / 35,854-line manifest, and
  existing count/hash baselines in this documentation prerequisite.
- [x] After the checker documentation commit and a fresh parser/resolver/
  lower/API/count/hash preflight, implemented only the dormant Task-258B2
  route and five tests. B3–B5 and Tasks 269–272 remain deferred.

## Checker Task 258B3 Frozen Consumer Checklist

- [x] Freeze the exact 104-byte final-LF source/hash, 49-node/root-48 tree,
  all ranges, theorem-only resolver provenance, and no label-reference
  bundle.
- [x] Freeze Task-48 `2/1/0`, Task-252 `5/5/0`, Task-256
  `2/0/0/0/0/0/0/4/4`, base `1/2/2/2/2`, one witness row, and `[0,1,2]`
  order in one arena.
- [x] Freeze private selector precedence, all-index parity, complete
  mutation/replay, named/multiple/missing/extra and subtree near misses, all
  A/B1/B2/active isolation orders, clone/debug, rollback, and empty
  semantics.
- [x] Preserve every corpus artifact, trace row/status/count, active route,
  source, 379-test list, 30-path / 36,479-line manifest, and hash in this
  prerequisite.
- [x] After the documentation commit and fresh preflight, implemented only
  the dormant B3 route with exactly five runner tests. The runner has 384
  tests and production/test hashes are remeasured in the implementation
  result.
- [x] Freeze Task 258B3N's exact named-primary dormant runner consumer,
  51-node identity, witness/name `1/1`, five-test matrix, no semantics, and
  unchanged runner baselines.
- [x] Implement only the frozen B3N consumer after its documentation commit
  and fresh preflight. Five compound tests pass; the runner has 389 tests
  and 30 production paths / 37,555 lines, with no active-corpus change.
- [x] Decompose broad Task 258B3M into exact B3M1 reserved-variable mixed
  multiple-witness transport and B3M2 other witness-term shapes.
- [x] Freeze only the 113-byte/56-node B3M1 dormant consumer, lower/base
  profiles, witness/name `2/1`, shared/dense ordinals, exact five-test
  matrix, no semantics, and unchanged runner baselines.
- [x] Implement only B3M1 after its docs commit and fresh preflight,
  projecting 394 runner tests.
- [x] Decompose B3M2 into exact unnamed-numeral B3M2A and remaining
  other-term B3M2B.
- [x] Freeze only the 107-byte/49-node B3M2A dormant consumer, Task-252
  `5/4/1` with numeric request 0, witness/name `1/0`, exact five-test
  matrix, no public/active/semantic route, and unchanged runner baselines.
- [x] Implement only B3M2A after its docs commit and fresh preflight,
  projecting 399 runner tests.
- [x] Decompose B3M2B into exact parenthesized B3M2B1 and remaining
  authority-valid B3M2B2.
- [x] Freeze B3M2B1 only; keep implementation and B3M2B2 separate.
- [x] Implement frozen B3M2B1 after its documentation commit and fresh
  parser/resolver/lower/count/hash preflight.
- [x] Decompose B3M2B2 into exact nested-parenthesized B3M2B2A and
  remaining authority-valid B3M2B2B.
- [x] Freeze only B3M2B2A; keep implementation and B3M2B2B separate.
- [x] Implement B3M2B2A after its documentation commit and fresh
  parser/resolver/lower/count/hash preflight.
- [ ] Freeze/implement B3M2B2B before Task 258B4.

## Checker Task 258B3M1 Runner Implementation Ledger

- [x] Select only the exact 113-byte/56-node source before B3N/B3/B2/B1/A.
- [x] Assemble six terms, two atomic formulas, two statements, two witnesses,
  and one name without activating a corpus or semantic route.
- [x] Add exactly five compound tests for identity, mutation/replay,
  byte/subtree near misses, both ownership orders, active isolation, and
  empty final semantics.
- [x] Remeasure 394 tests, leaf/facade/root/test sizes
  `3724/688/2501/7246`, and 30 production paths / 38,103 lines.

## Checker Task 258B3M2A Runner Documentation Ledger

- [x] Freeze the exact 107-byte/hash selector, 49-node/root-48 arena,
  zero-diagnostic frontend, theorem-only resolver provenance, and
  B3M1/B3N/B3/B2/B1/A precedence.
- [x] Freeze Task-48/252/256/base plus witness/name `1/0`, numeric request
  ownership, complete mutation/replay, all near misses, both ownership
  orders, final clone/debug/rollback, and empty semantics.
- [x] Freeze exactly five future runner tests while preserving 394 tests,
  sizes `3724/688/2501/7246`, 30 production paths / 38,103 lines, and all
  corpus/expectation/sidecar/trace/active/list/hash baselines.

## Checker Task 258B3M2A Runner Implementation Ledger

- [x] Select only the exact 107-byte/49-node source before
  B3M1/B3N/B3/B2/B1/A and keep every near miss dormant.
- [x] Assemble the authenticated lower/base handoffs and one unnamed
  numeral witness/no names without adding public, active, binding, or
  semantic ownership.
- [x] Add exactly five compound tests for identity, precedence,
  mutation/replay, all near misses, both family/active orders, rollback,
  detail projection, and empty final semantics.
- [x] Measure 399 tests, sizes `4185/691/2505/8611`, and 30 production
  paths / 38,571 lines; retain B3M2B before B4.

## Checker Task 258B3M2B1 Runner Prerequisite Ledger

- [x] Freeze the final-LF 113-byte/hash exact source, 53 nodes/root 52,
  zero diagnostics, and one local exported theorem owner/label.
- [x] Freeze five roots versus Task-252 `6/5/0`, wrapper term 2 / child
  term 3, refs `0/1/2/3/4 -> 0/1/3/4/5`, atomic starts `[0,4]`, and
  input-fact starts `[0,3]`.
- [x] Freeze one unnamed outer-term witness/no names, no public route/key,
  binding, active case, fixture, trace credit, or semantic output.
- [x] Freeze exactly five compound tests, unchanged 399-test and
  30-path/38,571-line baselines, and B3M2B2-before-B4.
- [x] After the docs commit/fresh preflight, implement only B3M2B1 and
  measure 404 runner tests.

## Checker Task 258B3M2B1 Runner Implementation Ledger

- [x] Add only the exact dormant 113-byte/53-node selector and assert zero
  frontend diagnostics.
- [x] Separate five roots from six Task-252 primaries and keep wrapper term
  2 / child term 3 out of Task-256 equalities.
- [x] Publish only paired base plus `1 witness / 0 names`, with no detail
  key, active route, binding, or semantic output.
- [x] Keep exactly five tests and cover Tasks 253–255 in both ownership
  orders without weakening lower-producer fail-close.
- [x] Measure 404 tests, sizes `4676/695/2508/9902`, and 30 production
  paths / 39,069 lines; retain B3M2B2 before B4.

## Checker Task 258B3M2B2A Runner Prerequisite Ledger

- [x] Freeze only final-LF 121-byte `take ((x));`, SHA-256
  `35396db1f7e22abfbe94861709b2ab9bca38d4464712dfbce114533d2ab4d71d`,
  57 nodes/root 56, and zero frontend diagnostics.
- [x] Freeze five roots / seven primaries, wrapper chain `2 -> 3 -> 4`,
  refs to `0/1/4/5/6`, and equalities over `[0,1]` / `[5,6]`.
- [x] Freeze paired base plus `1 witness / 0 names`, no detail key,
  active route, binding, semantics, fixture, sidecar, or trace change.
- [x] Freeze exactly five future compound tests, unchanged 404-test and
  30-path/39,069-line baselines, and B3M2B2B-before-B4.

## Checker Task 258B3M2B2A Runner Implementation Ledger

- [x] Add only the exact dormant 121-byte/57-node selector and require zero
  frontend diagnostics.
- [x] Compose five roots/seven primaries, chain `2 -> 3 -> 4`, five
  references, two equalities, and paired base plus `1 witness / 0 names`.
- [x] Preserve lower-producer-first failure, `Some(Vec::new())`, lookups
  `1/1`, uses `[1; 5]`, and empty binding/semantic output.
- [x] Add exactly five compound tests covering identity, corruption,
  near-miss, family/active isolation, replay, rollback, and final clone.
- [x] Measure 409 tests, sizes `5188/699/2513/11234`, and 30 production
  paths / 39,590 lines; retain B3M2B2B before B4.

## Checker Task 258B3M2B2B1P Runner Prerequisite Ledger

- [x] Freeze the private explicit-context Task-253 unwrapped imported
  application seam and preserve the legacy context-0 entry point.
- [x] Freeze the 143-byte/63-node motivating source and proof-context-1
  Task-253 `1/0/1/2/2` profile without adding a statement consumer.
- [x] Freeze exactly two compound tests for identity, context/provenance/
  form corruption, replay, and legacy byte compatibility.
- [x] Preserve 409 tests, sizes `5188/699/2513/11234`, 30 production
  paths / 39,590 lines, every active/fixture/expectation/sidecar/trace
  artifact, and B1P-before-B1A order.

## Checker Task 258B3M2B2B1P Runner Implementation Ledger

- [x] Add the private explicit-context helper and retain context-0
  compatibility by delegation to it.
- [x] Reuse exact Task-252 roots and the public Task-253 producer in proof
  context 1 without a statement consumer or duplicated lower rows.
- [x] Pass exactly two compound tests for identity, complete fail-close,
  replay/rollback, fixed legacy bytes, and empty downstream ownership.
- [x] Measure 411 tests, Task-253 sizes `1782/701/2514/2799`, and 30
  production paths / 39,857 lines.
- [x] Fresh-inventory and freeze the exact B3M2B2B1A application-witness
  contract in a separate documentation commit.

## Checker Task 258B3M2B2B1A Runner Prerequisite Ledger

- [x] Freeze the final-LF 143-byte/hash source, diagnostics 0, 63 nodes/root
  62, theorem/import resolver provenance, and proof context 1.
- [x] Freeze reuse of Task-252 `6/4/2` and Task-253 `1/0/1/2/2`, equality
  exclusion, base `1/2/2/2/2`, and one unnamed `Application(0)` witness.
- [x] Freeze owned take/witness nodes 49/48, unowned traversal node 47, and
  Task-253 target node 46 without lower-row duplication; require the atomic
  checker three-handoff installer.
- [x] Freeze exactly five compound tests and empty semantic/proof/goal
  output with no active/fixture/expectation/sidecar/trace change.
- [x] Preserve 411 tests, sizes `5188/701/2514/11234`, and 30 production
  paths / 39,857 lines; project 416 tests for implementation.
- [x] After the docs commit and fresh preflight, implement only the exact
  B1A dormant consumer.

## Checker Task 258B3M2B2B1A Runner Implementation Ledger

- [x] Select only the exact 143-byte/63-node source and authenticate the
  theorem plus imported `parser.type_fixtures::++` local/FQN resolver
  provenance.
- [x] Reuse Task-252 `6/4/2`, Task-253 `1/0/1/2/2`, Task-256 equality
  exclusion, and publish base `1/2/2/2/2` plus one unnamed
  `Application(0)` witness through the atomic checker installer.
- [x] Pass exactly five compound tests covering every loaded-source byte
  mutation, reparsed near misses, provenance/dependency/precedence
  corruption, family and active-route isolation, rollback, replay, final
  clone, and empty semantic/proof/goal tables.
- [x] Measure 416 runner tests, sizes `5618/706/2520/11945`, and 30
  production paths / 40,298 lines; leave active cases, fixtures,
  expectations, sidecars, and trace metadata unchanged.

## Checker Task 258B3M2B2B1B1P Runner Prerequisite Ledger

- [x] Freeze the exact final-LF 158-byte/67-node parenthesized imported
  application source with zero diagnostics.
- [x] Freeze shared Task-252 `6/4/2`, Task-253 `1/1/1/2/2`, proof context
  1, wrapper node 50, application node 48, and imported `++` provenance.
- [x] Restrict implementation to one private wrapper-aware Task-253 reuse
  sibling and preserve all unwrapped context-0/context-1 bytes.
- [x] Freeze exactly two runner compound tests for wrapper/context/
  provenance corruption, stale replay, clean replay, and compatibility.
- [x] Freeze all 158 source-byte and 67-node mutations, every success field,
  dormant-route exclusion, lower-stage precedence, atomic rollback, and
  separate context-0/context-1 debug hashes.
- [x] Preserve 416 tests, Task-253 sizes `1782/706/2520/2799`, 30 paths /
  40,298 lines, and every active/fixture/expectation/sidecar/trace artifact.
- [x] After the docs commit and fresh preflight, implement only B1B1P;
  freeze the B1B1 statement consumer in a later logical task.

## Checker Task 258B3M2B2B1B1P Runner Implementation Ledger

- [x] Add only the private exact wrapped-imported-application reuse seam and
  preserve both legacy unwrapped contexts.
- [x] Authenticate the complete imported `++` resolver provenance and reject
  five same-source identity/path/signature/export/contribution substitutions.
- [x] Pass exactly two compound tests covering all source bytes, all AST
  fields, the exact eight-entry reparsed near-miss matrix, empty downstream
  tables, atomic failure, replay, and compatibility.
- [x] Measure 418 tests, sizes `2652/708/2523/3727`, and 30 production paths
  / 41,173 lines with the recorded production and test-list hashes.
- [x] Leave public/active/statement routes, fixtures, expectations, sidecars,
  trace status/count, and semantic/proof/goal ownership unchanged; select
  B1B1 documentation only after this implementation commit.

## Checker Task 258B3M2B2B1B1 Runner Prerequisite Ledger

- [x] Freeze the exact final-LF 158-byte/67-node selector and complete local
  theorem/imported `++` resolver provenance.
- [x] Freeze Task-48 `2/1/0`, Task-252 `6/4/2`, wrapped Task-253
  `1/1/1/2/2`, equality-only Task-256, base `1/2/2/2/2`, and one unnamed
  `Application(0)` witness/no names.
- [x] Reuse the B1B1P wrapped seam and existing B1A checker API/atomic
  installer through one explicit private B1B1 profile.
- [x] Freeze five exact runner tests for all bytes/nodes, resolver
  substitutions, near-miss matrix, precedence, B1A/family/active isolation,
  rollback/replay/clone, and empty upper tables.
- [x] Preserve tests `374/418`, all measured sizes/counts/hashes, public and
  active routes, fixtures, expectations, sidecars, trace status/count, and
  semantic/proof/goal ownership.
- [x] Commit documentation alone, fresh-inventory, then implement B1B1.

## Checker Task 258B3M2B2B1B1 Runner Implementation Ledger

- [x] Implement the exact private wrapped selector and B1B1 route without
  broadening B1A or active dispatch.
- [x] Pass all five frozen runner tests and four checker tests; libraries are
  `378/423`.
- [x] Close `source_drift`, `test_gap`, and completion `design_drift`; test
  sufficiency and implementation reviews report no findings.
- [x] Preserve fixtures, expectations, sidecars, trace status/count, public
  APIs, and semantic/proof/goal/type-substitution deferrals.
- [x] Pass every final read-only quality hard gate with a valid `98/100`
  score before commit.

## Checker Task 258B3M2B2B2P Runner Prerequisite Ledger

- [x] Select the exact final-LF 172-byte/hash, zero-diagnostic,
  76-node/root-75 imported structure-constructor source before B2A.
- [x] Freeze Task-48 `2/1/0`, Task-252 `6/4/2`, and Task-254
  `1/0/1/2/0/2/6` in proof context 1 without duplicated Task-252 rows.
- [x] Freeze exact imported `TypeCaseStruct#5` contribution/origin/export
  provenance and the only owned kinds at constructor 59 and members 20/24.
- [x] Freeze exactly two runner compound tests covering all bytes/nodes,
  lower rows, provenance, substitutions, precedence, atomic replay, and
  existing Task-254 output.
- [x] Preserve checker source/API/tests, statement consumers, active routes,
  fixtures, expectations, sidecars, trace status/count, and all semantic,
  proof, goal, and IR owners.
- [x] Preserve documentation baselines `378/423`, sizes
  `1689/713/2528/1716`, 30 paths / 41,513 lines, and all measured hashes.
- [x] After the dedicated documentation commit and fresh preflight,
  implement B2P only, project 425 runner tests, remeasure all changed
  counts/hashes, then fresh-inventory B2A documentation.

## Checker Task 258B3M2B2B2P Runner Implementation Ledger

- [x] Implement the exact private owned-kind selector and
  existing-context/shared-Task-252 Task-254 seam in the frozen four files.
- [x] Pass both frozen compound tests and the 425-test runner library.
- [x] Close `source_drift`, `test_gap`, and completion `design_drift`.
- [x] Preserve checker/public/active/fixture/expectation/sidecar/trace and
  semantic boundaries; publish no Task-258 row.
- [x] Remeasure sizes, production manifest, and test-list hashes; keep B2A
  next.
- [x] Complete the final read-only quality review with no findings, every
  hard gate passing, and a valid score of `98/100`.
- [x] Fresh-inventory and freeze B2A documentation separately after commit.

## Checker Task 258B3M2B2B2A Runner Frozen-Contract Ledger

- [x] Distinguish `258B3M2B2B2A` from historical `258B3M2B2A` and freeze
  only the exact final-LF 172-byte/76-node structure-constructor witness.
- [x] Freeze reuse of Task-48 `2/1/0`, Task-252 `6/4/2`, Task-254
  `1/0/1/2/0/2/6`, equality-only Task-256 `2/0/0/0/0/0/0/4/4` with no
  direct structure edge/fingerprint, Task-258 base `1/2/2/2/2`, and
  witness/name `1/0`.
- [x] Freeze complete current/imported provenance, ownership at 62/61 only,
  retained Task-254 59/20/24 and Task-252 45/47/53/56/63/65 ownership, and
  unowned 52/54/57/60.
- [x] Freeze the sole witness target `Structure(0)`, additive checker
  target/fingerprint/builder/atomic installer seams, and no duplicated lower
  rows or parser/resolver projections.
- [x] Freeze exactly five runner and four checker tests covering all bytes,
  nodes, dependencies, precedence, isolation, rollback/replay/final clone,
  compatibility, malformed recovery, and empty semantics.
- [x] Preserve source, fixtures, expectations, sidecars, trace status/count,
  active routes, APIs, diagnostics, test baselines `378/425`, runner sizes
  `5962/2857/715/2531/13381/2991`, counts, and exact hashes.
- [x] Complete no-findings specification review and every documentation hard
  gate with a valid final quality score of `98/100`.
- [x] After the dedicated commit, fresh-inventory and implement only B2A.

## Checker Task 258B3M2B2B2A Runner Implementation Ledger

- [x] Implement the exact private selector and reuse B2P plus
  Task-48/252/254/256/base without duplicating lower rows.
- [x] Publish one `Structure(0)` witness through the checker-owned APIs and
  pass exactly five runner plus four checker tests.
- [x] Close bounded B2A `source_drift` and `test_gap`; preserve active,
  fixture, expectation, sidecar, trace, and semantic boundaries.
- [x] Record runner tests 430, sizes/manifests, and test-list hashes.
- [x] Complete no-findings test, implementation, and docs consistency
  reviews.
- [x] Pass focused checker/runner `4/4` and `5/5`, full format,
  all-target/all-feature Clippy with warnings denied, and `cargo test -q`
  with libraries `382/430` and lint policies `15/14`.
- [x] Pass all five CLIs at exit zero with 23 warnings / zero errors and
  unchanged counts/hashes; preserve manifests/test lists/forbidden artifacts
  and leave `stash@{0}` untouched.
- [x] Complete final quality review with all nine hard gates passing and a
  valid score of `98/100`.
- [x] Commit as `7613d50d`, verify clean metadata/stash invariants, and
  fresh-inventory the next dependency.

## Checker Task 258B3M2B2B2BP Runner Frozen-Contract Ledger

- [x] Freeze B2BP as the private selector proof-context lower prerequisite,
  distinct from B2B, over the 171-byte/79-node exact source.
- [x] Freeze exact Task-48 `2/1/0`, Task-252 `6/4/2`, Task-254
  `2/0/1/3/0/3/9`, provenance, ownership, edge/request, malformed, and
  exclusion matrices.
- [x] Freeze private selector site/owned-kind/context-handoff siblings in
  existing source-structure owners; add no checker/public/active surface.
- [x] Freeze exactly two runner tests and zero checker tests, including
  corruption/replay and constructor-seam compatibility.
- [x] Preserve fixtures, expectations, sidecars, trace/credit, diagnostics,
  active cases, baseline `382/430`, sizes, manifests, and hashes.
- [x] Complete no-findings specification/source-documentation reviews and
  all verification.
- [x] Record external docs commit `6f84d4eb` as a report-only metadata
  conflict and freeze docs-only BPC1 imported-provenance correction.
- [x] Repeat test, implementation-boundary, and source/documentation reviews
  after BPC1 with no findings.
- [x] Pass BPC1 final quality with no findings, all nine hard gates, and a
  valid `98/100`.
- [x] Commit the correction only and fresh-inventory B2BP implementation
  only.
- [x] After separate implementation, return to B2B frozen consumer docs.

## Checker Task 258B3M2B2B2BP Runner Implementation Ledger

- [x] Implement the exact private selector site/owned-kind/context handoff
  in the frozen four files with no public or active surface.
- [x] Pass the two exact tests, all mutation/precedence/replay gates, the
  exact malformed diagnostic, and B2P/B2A/legacy compatibility.
- [x] Close bounded `source_drift` and `test_gap`; test-sufficiency and
  implementation reviews report no findings.
- [x] Preserve fixtures, expectations, sidecars, trace/credit, diagnostics,
  active cases, checker APIs, and semantic boundaries.
- [x] Record runner tests `432`, sizes, production/test-list hashes, and
  unchanged CLI counts/hashes.
- [x] Complete source/documentation consistency with no findings and pass
  final quality review with all nine hard gates and a valid `98/100`.
- [x] Commit once, then fresh-inventory B2B documentation.

## Checker Task 258B3M2B2B2B Runner Frozen-Contract Ledger

- [x] Freeze the exact source/parser/malformed profile and all
  Task-48/252/254/256/258 rows, provenance, ownership, and exclusions.
- [x] Freeze the production-private B2BP consumer, B2A/B2B family boundary,
  exact five runner implementation owners, and no-public/no-active route.
- [x] Freeze five exact runner tests, precedence and replay matrices,
  subtree near misses, final clone, rollback, and semantic emptiness.
- [x] Record `382/432` baseline, `386/437` projection, module/manifest/
  test-list/CLI hashes, and unchanged fixture/trace/coverage impact.
- [x] Complete all four no-findings reviews and pass all nine final quality
  hard gates with a valid `98/100`.
- [x] Commit the paired documentation prerequisite alone as `4d2fb2b6`.
- [x] Fresh-inventory and implement only B2B after the documentation commit.

## Checker Task 258B3M2B2B2B Runner Implementation Ledger

- [x] Implement the exact frozen eight-file transaction and consume only the
  private B2BP owned-kind/proof-context handoff seams.
- [x] Preserve exact source, lower/base/witness rows, ownership exclusions,
  transitive surface validation, and the no-public/no-active/no-semantic
  boundary.
- [x] Pass the five exact frozen runner tests and record checker/runner
  library counts `386/437`.
- [x] Record final runner sizes `6826/4506/728/2543/17120/4315`, the
  30-path / 45,224-line production manifest, and production/test-list
  hashes.
- [x] Close the bounded `design_drift`, `source_drift`, and `test_gap`.
- [x] Complete specification/dependency review with no findings.
- [x] Complete test-sufficiency review with no findings.
- [x] Complete implementation review with no findings.
- [x] Complete source/documentation consistency review with no findings.
- [x] Complete all focused/full verification, lint, count, and hash gates.
- [x] Pass final read-only quality review with all hard gates and a valid
  score of at least `90/100`.
- [x] Create implementation commit `8311502c`, verify clean worktree,
  ahead-three origin metadata and untouched stash, then fresh-inventory the
  B2CP prerequisite before B2C.

## Checker Task 258B3M2B2B2CP Runner Frozen-Prerequisite Ledger

- [x] Establish the private update/`FieldUpdate` reuse seam before B2C.
- [x] Freeze the 181-byte/86-node exact source, 180-byte malformed profile,
  Task-48/252/254 rows, provenance, ownership, edges, and exclusions.
- [x] Freeze the exact four-file runner boundary, two tests, zero checker
  tests, and no statement/public/active/semantic output.
- [x] Freeze private `ImportedStructureUpdateSite`, owned-kind, and
  context-handoff siblings plus B2P-constructor/B2BP-selector compatibility.
- [x] Use the exact second test name
  `task258b3m2b2b2cp_structure_update_corruption_replay_and_prior_sibling_compatibility_fail_closed`.
- [x] Keep functional-copy semantics, type/result identity, witness
  obligations, theorem/proof acceptance, goals, and IR deferred; make no
  semantic acceptance claim for the `take` inside the `x = x` goal.
- [x] Record libraries `386/437`, projection `386/439`, current module,
  manifest, test-list, count, and CLI hashes.
- [x] Classify the skipped prerequisite as `design_drift`, the future seam
  as bounded `source_drift`, and its tests as `test_gap`; no blocking or
  nonblocking `spec_gap` was found.
- [x] Complete specification/dependency, test-sufficiency,
  implementation-boundary, and source/documentation reviews with no
  findings; pass documentation verification and all hard gates.
- [x] Record concurrent docs commit `817bb92b` as report-only
  `repo_metadata_conflict`; its restored `spec_gap` label invalidated the
  hard-gate and `98/100` claims.
- [x] Complete CPC1 repeated no-findings reviews, pass all nine hard gates,
  and obtain valid final quality `98/100`; explicitly justify live broad
  reruns blocked by the unrelated incomplete source diff.
- [x] Commit docs-only correction `258B3M2B2B2CPC1` separately as
  `ee267d9c`.
- [x] Fresh-inventory and implement only the private dormant B2CP runner
  seam; pass exactly its two frozen tests and close `design_drift`,
  `source_drift`, and `test_gap`.
- [x] Complete final test-sufficiency and implementation re-reviews with no
  findings.
- [x] Pass focused and workspace formatting, Clippy, tests, and all
  count/hash gates.
- [x] Synchronize final runner metrics and narrative-only audit impact with
  no specification/corpus/trace-credit change.
- [x] Complete final source/documentation review with no findings.
- [x] Pass independent final quality with no findings, all nine hard gates,
  and a valid `98/100`.
- [x] Pass the staged-diff audit and create the dedicated B2CP
  implementation commit `b146f0f72dceac2233c9d679b7820e264974b227`.
- [x] Fresh-inventory B2C after the B2CP commit.

## Checker Task 258B3M2B2B2C Runner Frozen-Contract Ledger

- [x] Record B2CP commit `b146f0f72dceac2233c9d679b7820e264974b227`
  complete and select B2C from clean fresh inventory.
- [x] Freeze exact 181-byte/86-node source, 180-byte missing-value profile,
  and five valid-excluded byte/hash/node profiles.
- [x] Freeze Task-48 `2/1/0`, Task-252 `7/4/3`, Task-254
  `2/0/1/3/1/4/9`, Task-256 `2/0/0/0/0/0/0/4/4`, Task-258 base
  `1/2/2/2/2`, and witness `1/0`.
- [x] Freeze resolver provenance, cross-family edges, ownership/exclusions,
  and B2C ownership only of `72/71` plus witness-to-`Structure(0)`.
- [x] Freeze the eight existing implementation files, unchanged private
  B2CP seam, no-public/no-active boundary, and exact four checker/five
  runner test names.
- [x] Freeze `386/439` baseline, `390/444` projection, sizes/manifests/
  hashes, unchanged corpus/CLI gates, and narrative-only audit impact.
- [x] Classify stale task selection as `design_drift`, future code as
  `source_drift`, and nine missing tests as `test_gap`; record no normative
  `spec_gap`, expectation drift, or current boundary violation.
- [x] Preserve all executable/canonical artifacts and state that `take`
  under `x = x` is source transport only.
- [x] Complete specification review with no findings.
- [x] Complete test-sufficiency review with no findings.
- [x] Complete implementation-boundary review with no findings.
- [x] Complete source/documentation consistency review with no findings.
- [x] Pass documentation verification and all required count/hash gates.
- [x] Complete final read-only quality review with every hard gate passing
  and a valid score of `98/100`.
- [x] Pass the cached-diff audit and create the dedicated B2C frozen-contract
  documentation commit as `d6076cc757ce675d1b46a720b4f00805923d3c70`.
- [x] Fresh-inventory and implement only the scoped B2C eight-file
  transaction.

## Checker Task 258B3M2B2B2C Runner Implementation Ledger

- [x] Limit runner implementation to the frozen statement/structure/facade/
  root/test files and consume the private B2CP seam unchanged.
- [x] Authenticate the exact source, malformed profile, and five valid
  excluded profiles without adding an active fixture or public route.
- [x] Preserve every Task-48/252/254/256/base row and publish only the
  witness-to-`Structure(0)` edge.
- [x] Add and pass exactly five runner tests and the paired four checker tests.
- [x] Pass runner library `444` plus policy suites and checker library `390`.
- [x] Complete final test-sufficiency and implementation reviews with no
  findings.
- [x] Synchronize EN/JA plans, ledgers, harness/module audits, and
  narrative-only coverage audit with final sizes and hashes.
- [x] Keep specification, `.miz`, fixtures, expectations, sidecars, trace
  status/tests, coverage credit, active corpus, public API, and semantics
  unchanged.
- [x] Pass broad workspace format, Clippy, and tests, including focused
  `4/4` and `5/5` and sibling `12/12` and `21/21` suites.
- [x] Complete final source/documentation consistency re-review with
  **NO FINDINGS**.
- [x] Complete independent final read-only quality review with
  **NO FINDINGS**, all nine hard gates PASS, and a valid `98/100`.
- [x] Audit the cached implementation diff and commit B2C as
  `e8373c683448e524cb98edde83fdf8de83a125cd`.
- [x] Verify clean ahead-eight/behind-zero post-commit repository state,
  unchanged stash, and fresh-inventory B3P.

## Checker Task 258B3M2B2B3P Runner Frozen-Contract Ledger

- [x] Freeze exact 117-byte/hash and 57-node/root-56 parser profile.
- [x] Freeze proof context 1, the local resolver record, Task-48 `2/1/0`,
  Task-252 `6/4/2`, Task-255 `1/0/0/0/0/2/1`, ownership, and exclusions.
- [x] Freeze exactly four runner files and two compound tests while
  preserving existing context-0 helper bytes.
- [x] Require the same two tests to exhaust all source bytes/LF variants,
  node and lower-table fields, resolver substitutions, owner partitions,
  precedence/replay/rollback/clones, family/semantic emptiness, and literal
  Task-111 handoff/typed/resolved debug hashes.
- [x] Add no checker source/test/API or upper B3A statement-witness edge.
- [x] Preserve all canonical/executable/trace artifacts and semantic
  deferrals; record baseline `390/444`, projection `390/446`, and exact
  counts/hashes.
- [x] Complete specification review with no findings.
- [x] Complete documentation review/repeat with no findings.
- [x] Complete test-sufficiency review with no findings.
- [x] Complete implementation-boundary review with no findings.
- [x] Complete source/documentation consistency review with no findings.
- [x] Pass source/hash, `15/14` lint, `390/444` libraries, production/
  test-list, five CLI hash, exact-scope, diff, and trace-no-op verification.
- [x] Complete final quality with no findings, all nine hard gates PASS, and
  valid `98/100` (`20/20/15/14/10/10/5/4`).
- [x] Audit/stage the task-only docs and commit the frozen contract as
  `285a1f11c310bb313c4c6b4feae914eb11f74754`.
- [x] Verify clean post-commit invariants and unchanged stash, then fresh-
  inventory the private B3P runner seam.

## Checker Task 258B3M2B2B3P Runner Implementation-Closure Ledger

- [x] Record prerequisite commit
  `285a1f11c310bb313c4c6b4feae914eb11f74754`.
- [x] Implement exactly four existing runner files with one `pub(super)`
  explicit-context sibling, context-0 delegate, and no public/active change.
- [x] Add exactly two compound tests covering bytes/LF, 57 nodes, resolver
  `63`, binding `39`, Task-252/255, fingerprint-only absence, precedence,
  replay, clones, literal hashes, and isolation.
- [x] Complete test-sufficiency and implementation reviews with
  **NO FINDINGS**.
- [x] Pass focused `2/2`, runner library `446/446`, formatting, package
  Clippy with `-D warnings`, and diff check.
- [x] Record sizes `7240/4517/740/2557/19275/2528`, production
  `30/49472`, and current production/test-list hashes.
- [x] Keep canonical, fixture, expectation, sidecar, trace, checker/public,
  and active-route surfaces unchanged.
- [x] Complete repeated source/documentation consistency and
  documentation/boundary reviews with **NO FINDINGS**.
- [x] Pass lint-policy `15/14`, metadata `137`, focused `2/2`, runner
  library `446/446`, formatting, workspace-wide warnings-denied Clippy and
  tests, five CLI/count/hash, current manifest/test-list hash, exact-30-file
  scope, and diff-check gates.
- [x] Complete final read-only quality review with **NO FINDINGS**, all nine
  hard gates PASS, and valid `98/100`
  (`20/20/15/14/10/10/5/4`).
- [x] Audit/stage and commit the B3P implementation closure as
  `abbfedfc2cdbaa97d8294893859da8cd350ad9a8`.
- [x] Verify clean post-commit HEAD, ahead-10 origin metadata, untouched
  stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4`, and fresh-inventory
  upper B3A.

## Checker Task 258B3M2B2B3A Runner Frozen-Contract Ledger

- [x] Close B3P commit `abbfedfc2cdbaa97d8294893859da8cd350ad9a8`
  and fresh-inventory B3A with clean/ahead-10/untouched-stash evidence.
- [x] Freeze authority, source/resolver label facts, lower Tasks
  48/252/255/256/258, one witness/zero names, partition/graph, and
  non-existential source-only intent.
- [x] Freeze exact seven implementation files, unchanged B3P set-term
  consumer, additive API/debug, five runner plus four checker tests,
  matrices, precedence, and deferrals.
- [x] Record `design_drift`/`source_drift`/`test_gap`, no blocking
  disagreement, baselines/projections/hashes, trace no-op, and exact
  `32`-doc prerequisite scope.
- [x] Complete specification/documentation, test-sufficiency, and
  implementation/API boundary reviews with **NO FINDINGS**.
- [x] Pass source/count/hash, lint/library, CLI, exact-scope, diff, and
  trace-no-op verification.
- [x] Complete documentation/boundary and source/docs consistency
  reviews with **NO FINDINGS**.
- [x] Complete final quality with **NO FINDINGS**, all nine hard gates PASS,
  and valid `98/100` (`20/20/15/14/10/10/5/4`).
- [x] Create the dedicated documentation-only commit
  `f4ff45964d97b31b6c328381120ba8ede080a2b1`.
- [x] Verify clean ahead-11/behind-0 post-commit state, unchanged stash
  `f65cf4a13752ec380710814a9ac6392ccb9d75d4`, and fresh
  implementation inventory.

## Checker Task 258B3M2B2B3A Runner Implementation Ledger

- [x] Close the prerequisite commit/post-commit/fresh-inventory gates.
- [x] Implement only the exact four runner plus three checker source files,
  preserving both set-term source owners and all authority artifacts.
- [x] Add the exact five runner plus four checker tests, additive API,
  set-only tuple, atomic installation/final clone, and all frozen matrices.
- [x] Complete specification, test-sufficiency, and implementation reviews
  with **NO FINDINGS**.
- [x] Pass focused/package tests, formatting, targeted Clippy, five CLI,
  final count/hash manifests, and diff checks.
- [x] Complete the second source/documentation consistency repeat with
  **NO FINDINGS**.
- [x] Complete the final documentation/boundary reread with
  **NO FINDINGS**.
- [x] Pass parent final verification: focused checker `4` plus runner `5`;
  checker package `394` plus lint-policy `15`; mizar-test package `451`,
  layout `3`, lint-policy `14`, metadata `137`, public-enum `2`, snapshot
  `21`; format; workspace Clippy/tests; five CLI counts/hashes; production
  manifests/test lists; diff check; and exact `39`-file scope.
- [x] Complete independent final read-only quality review with
  **NO FINDINGS**: all nine hard gates PASS with no score cap, valid
  `98/100` (`20/20/15/14/10/10/5/4`), and the stated residual deferrals
  unchanged.
- [x] Create dedicated B3A implementation commit
  `a147bad88f1963c504f796051ba0b855eca71d07`.
- [x] Verify clean ahead-12/behind-0 post-commit state and unchanged stash.
- [x] Fresh-inventory and select B3B empty-enumeration documentation.

## Checker Task 258B3M2B2B3B Runner Ledger

- [x] Freeze exact 118-byte/hash source, zero diagnostics, 50 nodes/root 49,
  resolver label, and complete Task-48/252/255/256/258 handoffs.
- [x] Freeze one zero-edge Enumeration target and one unnamed witness with
  no new lower helper or public API.
- [x] Freeze exactly five exhaustive dormant runner tests and four matching
  checker tests, projecting libraries `398/456` from `394/451`.
- [x] Keep `.miz`, expectations, sidecars, trace status/count, active route,
  CLI output, and semantics unchanged.
- [x] Complete all repeated reviews and verification with **NO FINDINGS**.
- [x] Commit the documentation prerequisite as
  `080e6824d843655986079f5d5fc41abe06b0fbd6`, verify clean
  ahead-13/behind-0 state and unchanged stash
  `f65cf4a13752ec380710814a9ac6392ccb9d75d4`, and fresh-inventory the
  separate B3B implementation.

## Checker Task 258B3M2B2B3B Implementation Ledger

- [x] Close prerequisite commit/post-commit/fresh-inventory gates.
- [x] Implement the exact four runner owners plus paired three checker
  owners and preserve public API, diagnostics, dependencies, and routes.
- [x] Implement the frozen five runner and four checker tests.
- [x] Remediate the initial three test-sufficiency findings within the
  existing nine tests.
- [x] Remediate the additional B3B-specific Task-48/252/255 lower-field
  mutation finding with exact `32/55/23` matrices.
- [x] Complete the independent implementation repeat with
  **NO FINDINGS** before the bounded test-only follow-up.
- [x] Add post-auth injection plus stage-prefix/non-generic-guard
  assertions and complete all test-sufficiency repeats with
  **NO FINDINGS**.
- [x] Complete final implementation repeat with **NO FINDINGS**.
- [x] Rerun focused tests and format/diff checks after the follow-up.
- [x] Complete final runner count/hash measurements.
- [x] Rerun libraries `398/456`, workspace Clippy/tests, and five CLI
  invariants.
- [x] Complete source/documentation consistency repeat with
  **NO FINDINGS** after the two `design_drift` wording fixes.
- [x] Complete independent final documentation/boundary review with
  **NO FINDINGS**.
- [x] Complete independent final read-only quality review with
  **NO FINDINGS**, all nine hard gates PASS, no score cap, and valid
  `98/100` (`20/20/15/14/10/10/5/4`).
- [x] Stage exactly `39` synchronized task files and inspect cached diff.
- [x] Create implementation commit
  `dbbf5f6a2b0bd58d8434fb4687f7bfad398ca4bc`.
- [x] Verify clean ahead-14/behind-0 post-commit state and unchanged stash.
- [x] Fresh-inventory and select B3C choice witness.

## Checker Task 258B3M2B2B3C Documentation Ledger

- [x] Freeze exact `110`-byte/hash, 52-node/root-51 choice source and local
  resolver provenance.
- [x] Freeze lower profiles `2/1/0`, `4/4/0`, empty Tasks 253/254,
  `1/0/0/1/0/0/2`, `2/0/0/0/0/0/0/4/4`, base `1/2/2/2/2`, witness `1/0`.
- [x] Freeze owner/unowned graph and SetTerm witness edge with zero
  Task-255 child edges.
- [x] Freeze exact four checker/five runner names and
  `32/55/39/72/62/21` plus byte/node/resolver/family/replay matrices.
- [x] Keep exact future runner scope at four files and both
  `source_set_term.rs` owners unchanged.
- [x] Preserve fixtures, expectations, sidecars, trace status/count, active
  route, CLI output, semantics, and coverage credit.
- [x] Fix initial medium ownership/matrix findings; repeat specification
  review **NO FINDINGS**.
- [x] Complete consistency/boundary review with **NO FINDINGS**.
- [x] Complete independent final quality review with **NO FINDINGS**, all
  nine hard gates PASS, no score cap, and valid `98/100`.
- [x] Verify exact docs-only scope, crate/workspace checks, five CLIs, and
  all count/hash/no-op gates.
- [x] Create dedicated B3C documentation commit
  `ea48ffc4fa586ac6d0813cd23a6b1d9b571087b2` and verify clean
  post-commit/stash state.
- [x] Fresh-inventory B3C implementation.

## Checker Task 258B3M2B2B3C Implementation Ledger

- [x] Close the prerequisite at clean ahead-15/behind-0 with stash
  unchanged; confirm no lower-stage prerequisite.
- [x] Implement only the frozen runner four and checker three source files;
  preserve both `source_set_term.rs` owners.
- [x] Implement exact runner five/checker four tests and
  `32/55/39/72/62/21` field matrices.
- [x] Remediate resolver replay and upper-family prefix/non-generic
  `test_gap` findings.
- [x] Remediate the B3A-hard-coded B3C `source_drift`/`test_gap` without
  changing enumeration siblings.
- [x] Complete repeated test-sufficiency and implementation reviews with
  **NO FINDINGS**.
- [x] Pass focused `5/5 + 4/4`, runner package
  `461+3/14/137/2/21`, and formatting.
- [x] Record final sizes, production/test hashes, unchanged CLI hashes, and
  trace/authority no-op.
- [x] Complete workspace Clippy/tests and final measurements.
- [x] Complete final source/documentation consistency and quality reviews.
- [x] Stage exact 39 synchronized task files and create implementation commit
  `7988a50934656ff90b31e06b883225f86196103b`.
- [x] Verify clean ahead-1/behind-0 post-commit state, unchanged stash, and
  report-only external origin movement.
- [x] Fresh-inventory and select B3D qua witness.

## Checker Task 258B3M2B2B3D Documentation Ledger

- [x] Freeze exact 109-byte/hash, 24-token, 54-node/root-53 qua source and
  local resolver owner/label provenance.
- [x] Freeze lower profiles `2/1/0`, `5/4/1`, empty Tasks 253/254,
  `1/0/0/1/0/1/2`, `2/0/0/0/0/0/0/4/4`, base
  `1/2/2/2/2`, witness `1/0`.
- [x] Freeze owner/unowned graph, `QuaBase -> Primary(2)`, and
  witness-to-SetTerm edge.
- [x] Freeze exact four checker/five runner names and
  `32/70/44/72/62/21` plus byte/node/resolver/family/replay matrices.
- [x] Keep future runner scope at four files and both `source_set_term.rs`
  owners unchanged.
- [x] Preserve authority, fixtures, expectations, sidecars, trace
  status/count/tests, active behavior, semantics, and coverage credit.
- [x] Complete all repeated reviews with **NO FINDINGS**.
- [x] Pass exact docs-only scope, crate/workspace checks, five CLIs, and all
  count/hash/no-op gates.
- [x] Complete independent final quality with all nine hard gates and valid
  score `>=90/100`.
- [x] Create dedicated B3D documentation commit
  `43af562c2cb84e72658cee059abbe7543ee73fe7`.
- [x] Verify clean ahead-2/behind-0 post-commit state, unchanged stash
  fingerprint `f65cf4a13752ec...`, and fresh-inventory B3D implementation.

## Checker Task 258B3M2B2B3D Implementation Ledger

- [x] Implement only the frozen runner four and paired checker three source
  consumers; preserve both `source_set_term.rs` owners.
- [x] Implement exact runner five/checker four tests and
  `32/70/44/72/62/21` mutation matrices with replay and owning prefixes.
- [x] Complete independent test-sufficiency review with **NO FINDINGS**.
- [x] Pass focused `5/5 + 4/4`, runner package
  `466+3/14/137/2/21`, checker package `406+15`, formatting, and full
  Clippy.
- [x] Record final runner/checker sizes, production/test-list hashes,
  unchanged CLI hashes/counts, and authority/trace/active/semantic no-ops.
- [x] Complete repeated independent implementation review with
  **NO FINDINGS**.
- [x] Complete repeated source/documentation consistency, bilingual, and
  boundary review with **NO FINDINGS** after the Medium stale-review and two
  Low 24-order/qua-edge corrections.
- [x] Pass checker package `406+15`, runner package
  `466+3/14/137/2/21`, formatting, full Clippy, full workspace tests, five
  CLIs, and final count/hash reruns.
- [x] Complete independent final read-only quality review with
  **NO FINDINGS**, all nine hard gates PASS, no score cap, and valid
  `100/100` (`20/20/15/15/10/10/5/5`).
- [ ] Stage the exact synchronized implementation scope, inspect cached diff,
  and create one implementation commit.
- [ ] Verify post-commit/stash invariants and fresh-inventory the next task.

## Checker Task 258B3M2B2B3E Documentation Ledger

- [x] Close B3D implementation commit
  `08a7d1e3d8c4b3b439325a16e1e139df4a1c18ed` and fresh-inventory B3E.
- [x] Freeze the exact 139-byte/hash, 28-token, 60-node/root-59 source and
  local resolver provenance.
- [x] Freeze lower/upper profiles, corrected ownership, one generator/type
  site, no condition, mapper/witness edges, and explicit semantic deferrals.
- [x] Freeze four checker/five runner names, exact
  `32/70/53/72/62/21` matrices, and all 120 B3A-E family orders.
- [x] Restrict implementation to checker three plus runner four consumers;
  preserve both set-term owners and every authority/corpus/trace/active
  surface.
- [x] Complete repeated reviews with **NO FINDINGS**.
- [x] Pass documentation-only scope/count/hash/no-op verification.
- [x] Pass independent final quality with all nine hard gates and valid
  score `>=90/100`.
- [x] Create prerequisite commit
  `8075000bf79be3fdea6b22f366fb6d9e59781fe7` and fresh-inventory B3E.

## Checker Task 258B3M2B2B3E Implementation Ledger

- [x] Implement only the frozen runner four and checker three consumers.
- [x] Add exact five runner/four checker tests and frozen matrices/orders.
- [x] Use coherent same-provenance successful Task-255 post-auth negatives.
- [x] Complete test and implementation reviews with **NO FINDINGS**.
- [x] Pass focused `5/5 + 4/4` and libraries `471/410`.
- [x] Record final sizes/hashes, CLI pass, and all no-op boundaries.
- [x] Complete source/documentation, bilingual, and boundary consistency
  with **NO FINDINGS** after three `design_drift` corrections.
- [x] Complete independent quality with **NO FINDINGS**, all nine gates
  PASS, no cap, and valid `100/100`.
- [x] Pass focused/package tests, formatting, full Clippy, root workspace
  tests, five CLIs, and count/hash/scope/forbidden/stash gates.
- [x] Stage exact B3E scope and inspect cached diff.
- [x] Create B3E implementation commit
  `e4479691db3b0a8785bb16e94d386bd71a394274`.
- [x] Verify clean post-commit/stash invariants and fresh-inventory B4A.

## Checker Task 258B4A Documentation Ledger

- [x] Freeze the distinct 80-byte/double-LF source/hash, 26-node/root-25
  parser surface, and resolver owner provenance.
- [x] Freeze exact Task-252/256/257/B1/binding profiles and Task-258
  `1/1/1/0/1` `Composite(0)` association.
- [x] Preserve the active 79-byte case as lower-only route isolation and
  forbid every fixture/expectation/sidecar/trace edit.
- [x] Freeze the five future runner consumers, including the sole
  crate-private Task-257B1 helper visibility seam, and five exact tests with
  lower/upper mutation, replay, family-order, clone/debug, and empty-semantic
  coverage.
- [x] Preserve truth, acceptance, proof, facts, active behavior, public
  runner schemas, and formula-statement coverage credit.
- [x] Complete repeated documentation review with **NO FINDINGS**.
- [x] Pass docs-only verification and all no-op/count/hash/stash gates.
- [x] Complete final quality with all hard gates and score `>=90/100`.
- [x] Stage/inspect and create dedicated B4A documentation commit
  `9da1ac13e811c78359d8d64e740832b2a30dae24`.
- [x] Verify clean ahead-6/behind-0 post-commit state, unchanged stash, and
  fresh-inventory B4A implementation.

## Checker Task 258B4A Implementation Ledger

- [x] Implement only the frozen five runner and three checker consumers.
- [x] Authenticate all 80 bytes, 26 Surface rows/root 25, resolver
  provenance, lower profiles/owned sites, and upper `1/1/1/0/1`.
- [x] Add five runner/four checker tests with exact mutation, coherent-near-
  miss, route-isolation, family-order, rollback, clone, and semantic-empty
  coverage.
- [x] Complete test-sufficiency and implementation reviews with
  **NO FINDINGS**.
- [x] Pass focused runner `5/5` and checker `4/4`; measure libraries
  `476/414` and runner production 30 paths/55,109 lines.
- [x] Preserve corpus/expectation/sidecar/trace/active/public-runner/
  semantic no-op boundaries.
- [x] Complete final source/documentation and bilingual consistency with
  **NO FINDINGS** after three Low `design_drift` corrections.
- [x] Pass package/workspace/Clippy/fmt/CLI/count/hash/stash gates.
- [x] Pass independent final quality with **NO FINDINGS**, all nine hard
  gates PASS, no cap, and valid `100/100`.
- [x] Stage the exact B4A scope and inspect the cached diff.
- [x] Create the dedicated B4A implementation commit
  `662adbde71e665ab37504ac476e94c935c493535`.
- [x] Verify clean ahead-7/behind-0 post-commit state, unchanged stash, and
  fresh-inventory B4B.

## Checker Task 258B4B Documentation Ledger

- [x] Freeze private 167-byte/double-LF source/hash, 124 nodes/root 123, and
  exact local theorem resolver provenance.
- [x] Freeze lower Task-252/256/257/B2/binding profiles, rootless arena,
  42/1/81 ownership, and upper `1/1/1/0/1` `Composite(0)` links.
- [x] Preserve the active 166-byte fixture as lower-only and forbid every
  corpus/expectation/sidecar/trace/active-route edit.
- [x] Freeze exactly four runner consumers, five runner tests, complete
  mutation/isolation/replay/final matrices, and semantic/coverage deferrals.
- [x] Record baseline runner `476`, projection `481`, production
  `30/55109`, unchanged manifests/CLIs, narrative audit impact, and exit
  gates.
- [x] Complete repeated documentation review with **NO FINDINGS**.
- [x] Pass docs-only verification and all no-op/count/hash/stash gates.
- [x] Pass independent final quality with all hard gates and score
  `>=90/100`.
- [x] Stage/inspect and create the dedicated B4B documentation commit
  `b8a7b8257a682f7c88de943ceaa35b67c0585bc4`.
- [x] Verify clean ahead-8/behind-0 post-commit state, unchanged stash
  fingerprint, and fresh-inventory B4B implementation.

## Checker Task 258B4B Implementation Ledger

- [x] Change exactly four runner and three checker files; leave
  `source_formula_composition.rs=1,853` and every lower owner unchanged.
- [x] Authenticate the private 167-byte route, raw label-free and enriched
  `1/1/1/1/0` resolver profiles, Task-257B2 lower transaction, rootless
  124-node `42/1/81` arena, and upper `1/1/1/0/1` `Composite(0)` links.
- [x] Pair B1/A versus B2/B exactly; retain B4B `0/0/[]`, B4A
  `1/1/[1,1]`, and the active 166-byte lower-only negative.
- [x] Pass focused runner `5/5` and checker `4/4`.
- [x] Complete test-sufficiency and implementation reviews with
  **NO FINDINGS**.
- [x] Measure runner library `481`, production `30/56007`, exact owner
  sizes and path/content/test-list hashes; reproduce checker `418` and
  `23/140821`.
- [x] Preserve public runner schemas, active behavior, semantics, corpus,
  expectations, sidecars, trace status/count/backlinks, and specifications.
- [x] Repeat source/documentation, bilingual, and boundary consistency
  reviews to **NO FINDINGS**.
- [x] Run complete crate/workspace, fmt, Clippy, CLI, count/hash, scope, and
  stash verification.
- [x] Complete independent final quality with **NO FINDINGS**, all nine hard
  gates PASS, no score cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Stage/inspect and commit only the exact B4B implementation scope as
  `752c17ae7d552d5268d1028612b8174e480b6f3e`.
- [x] Verify clean ahead-1/behind-0 post-commit state, unchanged stash, and
  fresh-inventory B4C.

## Checker Task 258B4C Documentation Ledger

- [x] Freeze the private 139-byte/two-LF source hash
  `36e5a68a92451590644951838a9af8926212bd78f88d1f90563f12b650b161c1`
  and preserve the active 138-byte lower-only hash
  `cbfd7077713e8e9630900e349d5f579251c19fba55434acb62170ea1dd940237`.
- [x] Freeze Surface `66/root65`, theorem `62` `19..137`, label token `6`
  `27..65`, outer formula `60` `67..136`, raw resolver `1/0/1/1/0`,
  owner path `[2,1]`, contribution `0` anchor `0..18`, and enriched
  resolver `1/1/1/1/0`.
- [x] Freeze the mandatory independent lower prerequisite to only
  `source_formula.rs` plus the runner `source_formula_composition.rs` test
  leaf: accept exact 138/139-byte one-/two-LF variants, reject zero/triple
  LF, and leave production `source_formula_composition.rs` unchanged.
- [x] Require the lower prerequisite test count to be fresh-inventory
  measured, with its own review, verification, quality gate, commit, and
  post-commit inventory.
- [x] Freeze lower profiles binding `4/4/0`, primary `6/6/0`, atomic
  `3/0/0/0/0/0/0/6/6`, composite `3/0/1/3/3/2/6`, and
  composition `3/6`.
- [x] Freeze upper `1/1/1/0/1`, context visible `[0]`, no input facts, both
  `Composite(0)` links, ownership `24/1/41`, and telemetry
  `2/2/[2,2,4,4,4,4]`.
- [x] Freeze the same seven eventual upper consumers as B4B and project
  focused checker `4` / runner `5` tests with complete exact/mutation/
  isolation/order/replay/final-empty coverage.
- [x] Preserve all existing spec, fixture, expectation, sidecar, trace
  status/count, active route, public schema, semantic/proof outputs, and
  coverage-audit status; record narrative audit impact only and keep B5
  deferred.
- [x] Record baselines checker/runner `418/481`, checker production
  `23/140821`, runner production `30/56007`, exact production/test-list and
  five CLI hashes, and unchanged stash.
- [x] Complete repeated documentation review with **NO FINDINGS**.
- [x] Pass docs-only verification and all no-op/count/hash/stash gates.
- [x] Pass independent final quality with **NO FINDINGS**, all nine hard
  gates PASS, no cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Stage/inspect and create the dedicated B4C documentation commit
  `3c723316ae632a867d29e8f4fc36348be30df202`.
- [x] Verify clean post-commit/stash invariants and fresh-inventory the
  mandatory lower-stage prerequisite.

## Task 257B3 Private Double-LF Selector Prerequisite Ledger

- [x] Limit the prerequisite to `runner/type_elaboration/source_formula.rs`
  and the runner `source_formula_composition` test owner.
- [x] Admit exactly the active 138-byte and private 139-byte identities;
  reject zero/triple LF and source/AST identity spoofing.
- [x] Preserve identical Task-257B3 lower tables/fingerprints, production
  `source_formula_composition.rs`, active CLI/trace behavior, and every
  upper owner.
- [x] Complete independent reviews, focused and broad verification, and
  final quality with **NO FINDINGS** and all hard gates PASS.
- [x] Stage/inspect and create the dedicated lower-stage prerequisite
  commit `42356f38ed0e679d7b878caf0e647c6aa8148d82`.
- [x] Verify clean post-commit/stash invariants and fresh-inventory B4C
  implementation.

## Checker Task 258B4C Implementation Ledger

- [x] Change exactly four runner and three checker files; leave every lower
  production owner, fixture, sidecar, expectation, trace row, and
  specification unchanged.
- [x] Authenticate the private 139-byte route, exact Surface/raw/enriched
  resolver profiles, Task-257B3 lower transaction, rootless 66-node
  `24/1/41` arena, and upper `1/1/1/0/1` `Composite(0)` links.
- [x] Pair B1/A versus B2/B versus B3/C exactly; retain the active 138-byte
  route as lower-only and publish telemetry `2/2/[2,2,4,4,4,4]`.
- [x] Pass focused runner `5/5` and checker `4/4`.
- [x] Complete test-sufficiency and implementation reviews with
  **NO FINDINGS**.
- [x] Measure runner library `488`, production `30/56872`, checker library
  `422`, checker production `23/141952`, and exact production/test-list
  hashes.
- [x] Preserve public schemas, active behavior, semantics, corpus,
  expectations, sidecars, trace status/count/backlinks, and specifications.
- [x] Complete final source/documentation, bilingual, and boundary
  consistency reviews with **NO FINDINGS** after correcting one Medium
  `design_drift`.
- [x] Run complete crate/workspace, fmt, Clippy, CLI, count/hash, scope, and
  stash verification; reproduce every frozen count and hash.
- [x] Complete independent final quality with **NO FINDINGS**, all nine
  hard gates PASS, no cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Stage/inspect and commit only the exact B4C implementation scope as
  `50ab1ebc747e912fff1f0cf111832e3c2c81ba01`.
- [x] Verify clean post-commit state, unchanged protected stash, and
  fresh-inventory the next dependency-ready logical task.

## Checker Task 258B5A Frozen-Contract Documentation Prerequisite

- [x] Freeze exact source/hash, Surface/resolver/lower/base/reference rows,
  proof-label `[0]` to descendant-citation `[0,1]`, and 20/73 ownership.
- [x] Freeze private route telemetry
  `1/1/[1,1,1,1,1,1,1,1,1,1]`, five runner tests, exact consumers, and
  atomic B1/B5A pairing.
- [x] Exclude B5B imports, B5C active negatives, public/corpus/trace changes,
  and all semantic outputs.
- [x] Complete independent specification, test-sufficiency,
  source/documentation boundary, and bilingual reviews with **NO FINDINGS**.
- [x] Reproduce crate/workspace, fmt, Clippy, CLI, exact scope/count/hash,
  authority no-op, repository-state, and stash gates.
- [x] Complete repeated independent final quality with **NO FINDINGS**, all
  nine hard gates PASS, no cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Commit only synchronized documentation as
  `59021f764f146d669f84877042f0512882c9c5ff`, verify post-commit
  invariants, and fresh-inventory B5A implementation.

## Checker Task 258B5A Implementation Ledger

- [x] Change exactly four runner and three checker consumers; keep every
  parser/resolver/lower production owner and public harness schema unchanged.
- [x] Authenticate the 185-byte source, exact Surface/raw/enriched resolver
  profiles, lower handoffs, Task-258 base/reference rows, `20/73`
  ownership, label `[0]`, citation `[0,1]`, and resolver node 82.
- [x] Preserve exact telemetry
  `1/1/[1,1,1,1,1,1,1,1,1,1]`, B1/B5A atomic installation,
  selector isolation, replay, clone, and empty semantics.
- [x] Keep B5B/B5C, active fixtures, specifications, expectations, sidecars,
  trace status/count/backlinks and credit, public results, diagnostics, and
  semantic outputs unchanged.
- [x] Run the frozen focused runner `5/5`, checker `4/4`, and preserved B1
  runner `6/6` tests.
- [x] Complete separate test-sufficiency and implementation reviews with
  **NO FINDINGS**.
- [x] Complete final source/documentation consistency review with
  **NO FINDINGS**.
- [x] Pass checker `426/426`, runner `493/493`, full workspace tests,
  formatting, exact Clippy, five CLIs, count/hash, and diff gates.
- [x] Complete final scope/forbidden-artifact, repository-state, and stash
  gates.
- [x] Complete independent final quality with **NO FINDINGS**, all nine hard
  gates PASS, no cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Stage and inspect only B5A, create implementation commit
  `4a79116c1a6f71155e4f366950fee8335b4dc8f1`, verify post-commit
  invariants, and fresh-inventory the next dependency-ready task.

## Checker Task 258B5B Frozen-Contract Documentation Prerequisite

- [x] Record B5A commit
  `4a79116c1a6f71155e4f366950fee8335b4dc8f1` as superseding its historical
  pending line and fresh-inventory B5B.
- [x] Classify the unfrozen API as `design_drift`, opt-in imported-label
  population as separate lower `source_drift`, and active coverage as
  bounded `test_gap`, with no blocking gap.
- [x] Freeze the 146-byte source/hash, 57-node frontend/resolver, raw
  `1/0/1/1/0`, opt-in `8/1/1/3/1`, all lower profiles, Task-258
  `1/2/2/2/2 + 0/1`, and `8/49` ownership.
- [x] Freeze the two-file lower prerequisite and two tests, exact imported
  `Ref` provenance, citation target/API, telemetry, seven upper consumers,
  four checker/five runner tests, and B1/B5A debug stability.
- [x] Preserve all authority/test/trace/public/semantic boundaries and B5C.
- [x] Complete specification review with no blocking finding and pass
  crate/workspace tests, format, Clippy, and five CLIs.
- [x] Complete test-contract, source/documentation, and bilingual reviews
  with **NO FINDINGS**.
- [x] Complete final scope/repository/stash gates.
- [x] Complete final quality with **NO FINDINGS**, all nine hard gates PASS,
  no cap, and valid `100/100` (`20/20/15/15/10/10/5/5`).
- [x] Stage/commit only synchronized B5B documentation as `141dc44a`,
  verify invariants, and fresh-inventory the mandatory lower task.

## Checker Task 258B5B Lower-Stage Prerequisite

- [x] Change only `runner/import_fixtures.rs` and the statement test leaf;
  add the opt-in `Ref` label helper and two tests in separate commit
  `46dd9db5`, then fresh-inventory upper implementation.

## Checker Task 258B5B Upper Implementation

- [x] Change only the frozen three checker and four runner Rust consumers;
  keep synchronized design output in the same logical task without expanding
  that code boundary.
- [x] Authenticate exact source 146 bytes, 57 nodes/root 56, raw/enriched
  resolver `1/0/1/1/0` and `8/1/1/3/1`, Binding `2/1/0`, Task-252
  `4/4/0`, Task-256 two formulas/four edges/four requests, Task-258
  `1/2/2/2/2 + 0/1`, and `8/49` ownership.
- [x] Install the imported target/kind and exact import/projection/reference
  provenance only under exact-source opt-in; preserve B1/B5A debug bytes,
  pairing, replay, and empty semantics.
- [x] Pass focused checker `4/4`, runner `7/7` (upper five/lower two), full
  checker `430/430`, runner `500/500`, and record current production,
  test-list, CLI hashes, and owner counts.
- [x] Complete test-sufficiency and repeated implementation reviews with
  **NO FINDINGS**; classify the hard-gate documentation mismatch as
  `design_drift` and synchronize it.
- [x] Keep the spec-coverage change narrative-only and preserve requirement
  `tests = []`, trace status/count/backlink/owner/credit, corpus,
  expectations, sidecars, public runner schema, B5C, and semantic deferrals.
- [x] Complete final source/documentation consistency with **NO FINDINGS**,
  then pass workspace formatting, exact Clippy, full tests, five CLIs, and
  final count/hash/scope gates.
- [x] Complete independent final quality with **NO FINDINGS**, all nine hard
  gates PASS, no cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Stage and inspect only the B5B upper logical task, create implementation
  commit `f27d2c9169b08078f00b75c4a57f94e30fa28f59`, verify clean
  post-commit/stash invariants, and fresh-inventory the next dependency-ready
  task.

## Checker Task 258B5C Frozen-Contract Documentation Prerequisite

- [x] Select B5C from the clean post-B5B inventory and freeze Chapter 15
  §15.10 plus Chapter 16 §§16.4.2/16.5.1 as the controlling label-scope
  authority; keep Chapter 11 §11.2 contextual only.
- [x] Classify the absent normal-source proof-label projection path as Medium
  `source_drift` with potential `boundary_violation`, stale derived ownership
  as `design_drift`, missing active confinement cases as `test_gap`, and
  unspecified public resolver codes as a Low deferred, nonblocking `spec_gap`.
- [x] Freeze the exact 173-byte inner-to-outer and 197-byte sibling sources,
  hashes, normal frontend identities, proof scopes, statement ordinals,
  declaration/reference nodes and ranges, and expected unresolved resolver
  outcomes.
- [x] Record the structurally validated Surface-to-resolved provider as a
  known resolver prerequisite with sufficient architecture authority.
- [x] Freeze strict dependency order: this documentation-only commit,
  resolver R-032A structural arena/map, resolver R-032B proof-label source
  collector, then active B5C declaration-symbol fixtures/sidecars/trace/
  runner commit.
- [x] Freeze both exact `Result`-returning APIs/errors, narrow R-032B
  inclusion/exclusion, theorem-root paths, completion visibility ordinal 3,
  exact label/semantic origins, and lower positive/negative/provenance tests.
- [x] Freeze same-`'a` ast/resolved storage, validation-only module, owned
  namespace/contribution, `Self` return, exact `SurfaceNodeId` error payloads,
  global one-based ordinals, `ConclusionStatement`/reference chain, and
  canonical `proof-step-v1` identity.
- [x] Require the active runner to consume validated resolver-owned
  projections and candidates; forbid checker installation and
  runner-fabricated ids, semantic proof scopes, ordinals, or origins.
- [x] Freeze source-byte-plus-normal-AST-only route selection, shared
  env/module and exact local-source contribution-0 authentication, separate
  private input/confinement details, expectation-copy guards, and exact
  48-file documentation scope.
- [x] Freeze the two future fail fixtures, sidecar stage/domain/phase/category,
  empty public diagnostic codes, private detail key, two trace requirement
  IDs, projected count changes, exact consumers, tests, exclusions, audit
  impact, and exit criteria.
- [x] Complete repeated specification, test-contract, source/documentation,
  bilingual, and final-quality reviews with no findings and all hard gates
  PASS at valid quality at least `90/100`.
- [x] Reproduce current crate/workspace, format, Clippy, five-CLI,
  count/hash/scope, authority no-op, repository-state, and protected-stash
  gates without changing production, corpus, expectation, sidecar, or trace
  status/count.
- [x] Stage and commit only synchronized B5C frozen-contract documentation,
  verify clean post-commit/stash invariants, and fresh-inventory resolver
  R-032A.
- [x] Freeze the resolver's exact
  `Root -> CompilationUnit -> ItemList -> direct TheoremItem -> direct
  ProofBlock` upper chain, exact-one normal upper children, direct-normal
  theorem scan, remaining default-deny no-ordinal/no-descent forms, and
  positive-edge/missing/additional/wrong/direct-relocation/`VisibleItem`/
  mixed-list tests.
- [x] Freeze independent runner mutations for environment module; projection
  module/namespace/contribution; contribution zero/multiple cardinality, id,
  all non-local kinds, record module, and LocalSource source id, all mapping
  only to `proof_scope_input`.
- [x] Keep source-bytes-plus-normal-AST selection, expectation
  non-selection, empty public codes, and the exact 48-file scope unchanged.

## B5C R-032A Preflight Overlay

- [x] Complete the separate mizar-syntax S-026 frozen-documentation commit.
- [x] Complete the separate S-026 implementation and its review/verification
  gates.
- [x] Complete the dedicated S-026, R-032A, and R-032B prerequisite commits
  before adding active B5C artifacts; R-032B is
  `b3a7e79a6b60db2974e911c69bb56ff5f4609064`.
- [x] Preserve the exact source-only selector, provenance authentication,
  private details, empty public codes, and projected active count impact.

## Checker Task 258B5C Active Implementation

- [x] Add only the two frozen fail fixture/sidecar pairs and two covered trace
  rows; reproduce `421/389`, `228/193`, `101/7/198/1`, and `23/0`.
- [x] Implement only the private declaration-symbol consumer over unchanged
  R-032A/R-032B APIs, with no runner-derived semantic identity.
- [x] Cover exact dense profiles, every frozen provenance/result corruption,
  expectation non-selection, replay/order, and existing-case isolation.
- [x] Correct the omitted metadata count consumer in exactly four `5 -> 7`
  assertions; classify it as `test_expectation_drift` plus write-scope
  `design_drift`, without changing test intent.
- [x] Complete findings-free test, implementation, and source/documentation
  reviews plus focused/crate/workspace/count/hash verification gates.
- [x] Complete independent final quality with **NO FINDINGS**, all nine hard
  gates PASS, no score cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [ ] Stage only task files, create the dedicated commit, verify repository
  and stash invariants, then fresh-inventory the next executable task.

## Checker Task 259 Frozen Consumer Prerequisite

- [x] Record B5C commit
  `33ac57e96f048dc40559565f54369cac854409a7` and successful post-commit
  invariants before selecting Task 259.
- [x] Freeze the future exact pass fixture/sidecar name, source bytes/hash,
  pass/type-check/empty-diagnostic expectation, and one-row trace intent.
- [x] Freeze lower build order and exact profiles:
  extended Task 248, Task 249 `2/2/0`, Task 252 `4/4/0`, Task 256
  `2/0/0/0/0/0/0/4/4`, and Task 259 `1/2/1/1/1` plus one pending obligation.
- [x] Freeze exact-source/normal-AST selection, resolver predicate
  authentication, lower-handoff corruption, all-or-nothing install,
  deterministic replay/order, expectation non-selection, and family
  isolation tests.
- [x] Keep the mixed predicate-plus-functor fixture, sidecar, trace rows,
  generic gap detail, and all Task-260 ownership unchanged.
- [x] Keep this prerequisite documentation-only at `421/389`, `228/193`,
  `101/7/198/1`, type `253/241`, and warnings/errors `23/0`.
- [x] Complete all four findings-free reviews, full verification and
  count/hash reproduction, and independent final quality with all nine hard
  gates PASS at valid `100/100`.
- [x] Complete the task-only documentation commit
  `d5294b8f4be46a420bbdfa2fc4062384be983ce0` and post-commit fresh
  inventory.
- [x] Fresh-inventory the separate Task-248 extension documentation
  prerequisite.

## Checker Task 248 Two-Parameter Runner Prerequisite

- [x] Freeze exact shell/direct-parameter/range/type/source-order extraction
  from the future Task-259 source without selecting an active route.
- [x] Freeze caller-owned sites plus shared-arena anchor/context validation
  and return only the existing Task-248 projection.
- [x] Preserve Profile A and exclude every guard/predicate/property/
  justification descendant plus Task 249+/259 semantics.
- [x] Freeze the five-file Rust scope, four-test matrix, runner
  `504 -> 508` projection, and no fixture/sidecar/trace/count delta.
- [x] Complete findings-free reviews and full docs-only focused/crate/
  workspace/count/hash verification.
- [x] Complete independent final quality with **NO FINDINGS**, all nine hard
  gates PASS, no score cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Complete the dedicated documentation commit
  `f9b47375acc18acebf56a69f5d8a7edec539c2be` and clean post-commit
  inventory.
- [x] Implement the separate Task-248 extension commit
  `ca54135f36c9fecfc02c2b8120ec4e63e8c6ca36`, then return to the Checker
  Task-259 consumer.

## Checker Task 259 Frozen-Consumer Correction Prerequisite

- [x] Record completed Task-248 Profile B and current runner `508` baseline.
- [x] Freeze the new private route leaf, parent facade/re-export, test include
  and leaf, four mechanical metadata assertions, one pass fixture/sidecar,
  and one trace row as exact future consumers.
- [x] Keep `BindingEnv`, all lower handoffs, and Task-259 tables checker-owned;
  keep raw AST/sibling/subtree authentication private to the runner.
- [x] Repeat specification/consumer review to no findings and complete
  docs-only verification with every executable artifact byte-unchanged.
- [x] Commit only the correction documents as
  `e202dd70bf4e97ddb53c1275b49e667b6a77f7a0`, verify clean/stash-invariant
  state, and fresh-inventory Task-259 implementation.

## Checker Task 259 Active Consumer Implementation

- [x] Add the exact 165-byte pass fixture/sidecar and sole covered trace
  backlink without changing the existing mixed Task-260 fixture family.
- [x] Implement the private Task 248 -> 249 -> 252 -> 256 -> 259 route with
  exact lower profiles and a final `1/2/1/1/1` handoff plus one `Pending`
  `PredicatePropertyCorrectness` obligation.
- [x] Add the four frozen runner tests and keep raw AST, sibling, resolver,
  subtree-exclusion, and exact-source authentication runner-private.
- [x] Independently review and update only the two stale source-statement
  active-type count consumers from `198` to `199`; preserve their empty
  selection assertions and keep the other two mechanical metadata assertions
  aligned.
- [x] Reproduce executable metadata `422/390`, `229/193`,
  `101/7/199/1`, type `254/242`, warnings/errors `23/0`, metadata `137`,
  checker `435`, runner `512`, resolver `144`, and syntax `59`.
- [x] Complete final independent test-sufficiency, implementation, and
  source/documentation-consistency reviews with no findings; pass all nine
  hard gates with an uncapped final quality score of `100/100`.
- [x] Fresh-measure final affected production/test-support inventories:
  checker `24/147030`, runner `31/63248`, checker producer/support
  `1794/1974`, and runner production/test leaves `1233/517`.
- [x] Stage only Task-259 files, create implementation commit
  `b61be7e567b92d31b3544b86e5c7a68537625743`, verify repository/stash
  invariants, and fresh-inventory Task 260.

## Checker Task 260 Frozen Consumer Prerequisite

- [x] Freeze exact source/hash/AST/resolver and lower Task-248/249/252/256
  bundle before any active consumer edit.
- [x] Freeze `2/2/1/2/2`, two Pending obligations, exact-source selection,
  lower corruption, atomic install, deterministic replay, and Task-259/mixed
  family isolation.
- [x] Freeze one future pass pair/trace row, four runner tests, six mechanical
  active-count assertions, projected counts, and empty semantic output.
- [x] Keep all current fixtures, sidecars, expectations, trace rows/status,
  production source, and Cargo metadata byte-unchanged in this prerequisite.
- [x] Repeat all four reviews to no findings and complete docs-only
  verification with all nine gates PASS and uncapped quality `100/100`.
- [x] Complete exact staging, documentation commit
  `b587038f12f84a77720f6441a000ddb84c7b996f`, and post-commit gates.
- [x] Record Task 249R as checker-only: no runner source/library-count,
  fixture/sidecar/trace, or corpus/metadata/CLI-count change, and replace the
  impossible lower `4/4/0` profile with `2/4/0/2`.
- [x] Resume the frozen Task-260 consumer after the separate Task-249R
  documentation and implementation commits pass their gates; implementation
  commit is `c233bfdff8317a1f4ffdd5750e62a29ee6e69b2f`.

## Checker Task 260 Active Consumer

- [x] Add the private exact-source/resolver/lower route and keep all raw
  syntax/resolver mutation ownership inside `mizar-test`.
- [x] Add exactly four frozen runner tests, including a literal 108-row
  Surface oracle and independent environment/projection/symbol/definition/
  contribution plus every-lower-association corruption.
- [x] Add one pass pair and sole covered trace backlink, update six mechanical
  active-type assertions to `200`, and preserve all existing expectations.
- [x] Reproduce runner `516`, metadata `137`, `423/391`, `230/193`,
  `101/7/200/1`, type `255/243`, warnings/errors `23/0`, and exact CLI/
  test-list/production hashes.
- [x] End repeated test-sufficiency and full implementation reviews with
  **NO FINDINGS** while publishing no proof/fact/acceptance/VC payload.
- [x] Complete source/documentation consistency with **NO FINDINGS** and pass
  the full shared verification matrix.
- [x] Pass all nine final hard gates with quality `100/100` and no score cap.
- [x] Complete the shared Task-260 staging/commit/post-commit gates in
  `c83e424a485a24dd0f00ddea687903a235d85850`.

## Checker Task 261 Frozen Attribute-Definition Consumer

- [x] Freeze the exact 116-byte source/hash, 45-row Surface oracle, resolver
  provenance, lower Task-248/249/252/256 associations, and Task-259/260
  isolation before any implementation edit.
- [x] Freeze the private runner/public checker ownership split and checker
  table cardinalities `1/2/1/1`, with no ordinary initial obligation.
- [x] Freeze exactly one future pass pair/trace row, five checker tests, four
  runner tests, projected counts, hashes to remeasure, exclusions, and exit
  gates.
- [x] Keep production, fixtures, sidecars, expectations, trace rows/status/
  counts, and Cargo metadata byte-unchanged in this documentation prerequisite.
- [x] Complete findings-free reviews, all nine documentation gates, exact
  staging, prerequisite commit `209c32fc2ec547ceedd32f1052345ae2fc5b0451`,
  and clean post-commit inventory.
- [x] Fresh-inventory and implement only the frozen Task 261 source/runner/
  fixture/trace/count scope; focused and repeated test/implementation reviews
  end with **NO FINDINGS**.
- [x] Complete source/documentation consistency with **NO FINDINGS** and pass
  full verification, including exact count/hash reproduction.
- [x] Pass all nine final hard gates with **NO FINDINGS**, no score cap, and
  quality `100/100`.
- [x] Complete exact staging/commit/post-commit as
  `b1782bfc06388410229f07ee193a5febe0bf525e`, then select Task 262.

## Checker Task 262 Frozen Mode-Definition Consumer

- [x] Freeze the exact 141-byte source/hash, 54-row Surface oracle, two-shell
  resolver identity, lower Task-248/249 associations, and sibling isolation.
- [x] Freeze the private runner/public checker boundary, six
  `1/2/1/1/1/1` tables, unresolved RHS request, and one pending existing-kind
  `Sethood` row without semantic acceptance.
- [x] Freeze one future pass pair/sole trace row, five checker/four runner
  tests, projected counts, hashes to remeasure, exclusions, and exit gates.
- [x] Keep production, fixtures, sidecars, expectations, trace rows/status/
  counts, and Cargo metadata unchanged in this documentation prerequisite.
- [x] Complete findings-free reviews, all nine docs gates at uncapped
  `100/100`, exact staging, prerequisite commit
  `8c3fa20acef42477d38a66ddddec42dacced0863`, and clean post-commit inventory.
- [x] Fresh-inventory and freeze mandatory checker Task 249M with no runner or
  corpus change.
- [x] Review and separately commit Task-249M documentation; then implement its
  four checker tests and complete checker review/verification/separate commit.
- [x] Return to and implement only Task 262: exact consumer, four tests, pass
  pair, reciprocal trace row, active counts, and measured hashes are present.
- [x] Complete repeated reviews with **NO FINDINGS**, full verification, all
  nine hard gates at uncapped quality `100/100`, and exact Task-262 commit
  readiness; then fresh-inventory Task 263+ without broadening mixed semantics.

## Checker Task 263 Preflight Resolver Gate

- [x] Freeze the exact 320-byte future source as Chapter-5-derived test intent
  without adding it to the corpus in the lower prerequisite.
- [x] Preserve every runner route, fixture, sidecar, expectation, trace row/
  status/count, active case, metadata assertion, and CLI hash in Task 263R.
- [x] Complete the separate Task-263R documentation commit and fresh-inventory
  the lower implementation with no `mizar-test` or corpus delta.
- [x] Confirm the implemented two-file resolver correction changes only the
  resolver test inventory `144 -> 146` and passes its exact probes/reviews.
- [x] Complete all full/final gates with **NO FINDINGS**, all nine hard gates
  PASS, no score cap, and valid `100/100`.
- [x] After implementation commit
  `997457dd3189030aa3b137b568ce82fed456fe1e`, fresh-inventory the Task-263
  boundary; checker Task 249S supersedes the direct consumer freeze as the
  remaining lower prerequisite.

## Checker Task 249S No-Runner Prerequisite

- [x] Freeze exact checker-only `0/4/0/0/0/4` member-type intake and four local
  tests without changing runner, corpus, trace, metadata, or CLI artifacts.
- [x] Keep the no-op boundary through the separate implementation, then
  let Task 263 alone add its exact private consumer and pass/trace pair.

## Checker Task 263 Frozen Runner Consumer

- [x] Freeze exact source/hash, 75-row Surface oracle, `10/8/8/8/0` resolver
  oracle, `0/4/0/0/0/4` lower fingerprint, and subtree exclusions.
- [x] Assign one private pre-gap route, four exact runner tests, one new
  canonical-derived pass pair, and one reciprocal covered trace row.
- [x] Freeze transport-only credit, existing mixed-gap byte stability,
  projected runner/metadata/coverage counts, hash remeasurement, scope, and
  semantic non-publication.
- [x] Preserve the no-op boundary through docs-only commit readiness; repeated
  reviews are **NO FINDINGS** and all nine hard gates pass at uncapped
  `100/100`.
- [x] After the docs commit and fresh inventory, implement only the exact
  route/tests/pass/trace and synchronize active audits/counts/hashes.
- [x] Pass all reviews and verification with **NO FINDINGS**, all nine hard
  gates at uncapped `100/100`; complete Checker Task-263 implementation commit
  `f11a517e91433b461447522eff06cd85e6187063` and clean fresh inventory.

## Checker Task 264R No-Runner Prerequisite

- [x] Freeze zero runner/corpus/sidecar/expectation/trace/metadata/CLI impact.
- [x] Treat the two existing Parser Task 48 fixtures as read-only lower probes;
  keep the inactive coherence seed byte-identical and inactive.
- [x] Reconfirm all post-Task-263 counts and hashes through the docs and lower
  implementation; add no runner consumer before Checker Task 264. Dedicated
  lower implementation commit
  `db8c39e31678d6b8a1f0900a5368c3b95c7162b5` and clean post-commit inventory
  are complete.

## Checker Task 248P No-Runner Prerequisite

- [x] Freeze zero runner/corpus/sidecar/expectation/trace/metadata/CLI impact
  and preserve runner `528`, production `35/67939`, and all hashes.
- [ ] Complete the checker-only docs and implementation commits without adding
  a Profile-C runner helper; then fresh-inventory Task 264's consumer.
