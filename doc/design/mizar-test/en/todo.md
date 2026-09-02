# mizar-test TODO

> Compacted 2026-09-02 (batch CPT-10, rules in
> [../../documentation_compaction_rules.md](../../documentation_compaction_rules.md)):
> completed task bodies and closed addenda-section bodies moved verbatim to
> [../../archive/test_todo_sections.md](../../archive/test_todo_sections.md).
> Every heading, every registered ledger redirect line, and every section
> with open work remains below.

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
2. **Source/spec gap audit and status sync.** [x]
3. **Runner modes and CLI completion.** [x]

### Snapshot support

4. **Snapshot module: API and canonicalization.** [x]
5. **Snapshot update policy and determinism checks.** [x]

### Coverage and soundness contracts

6. **Coverage and pass/fail-mix reporting.** [x]
7. **Stage-prerequisite validation.** [x]
8. **Fail/soundness contract support.** [x]
9. **Corpus size and review-rule validation.** [x]

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
12. **Public-enum forward-compatibility policy.** [x]
13. **Bilingual documentation sync audit.** [x]
14. **Incremental/parallel verification regression matrix.** [x]
15. **Architecture-22 follow-up audit.** [x]
16. **Source-derived builtin type-expression bridge.** [x]
17. **Source-derived builtin `ResolvedTypedAst` bridge.** [x]
18. **Source-derived reserve declaration semantic bridge.** [x]
19. **Reserve bridge core summary readiness and builtin declaration
    inventory.** [x]
20. **Reserve bridge core context readiness.** [x]

### Kernel soundness-audit follow-ups (2026-07-03)

The kernel acceptance-boundary audit
([soundness_argument.md](../../mizar-kernel/en/soundness_argument.md))
reported two harness-owned findings, F7 and F8. These are minimal
audit-driven additions; broader runner growth remains task 10 pacing.

21. **Corrected-path soundness vocabulary in the required-case registry (kernel F7).** [x]
22. **Certificate-corpus root naming reconciliation (kernel F8).** [x]

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

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Task 242 Active Addendum

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Task 243 Active Addendum

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Task 244 Active Addendum

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Task 245 Active Addendum

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Task 246 Active Addendum

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Runner Module-Boundary Refactor Backlog

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## VC Task 30 / Task-10 Consumer Ownership

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## VC Task 31 / Task-10 Consumer Completion

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Resolver Task 31 / Declaration-Symbol Completion

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Parser Task 47 / Parse-Only Completion

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Parser Task 48 / Property-Implementation Parse-Only Completion

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 248 / Task-10 Consumer Completion

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 249 Frozen Consumer Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 250 Frozen Consumer Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 251 Frozen Consumer Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 252 Frozen Consumer Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 253 Frozen Consumer Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 254 Frozen Consumer Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 255 Frozen Consumer Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 256 Frozen Consumer Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 257A Frozen Consumer Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 257B1 Frozen Consumer Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 257B2 Frozen Runner Checklist

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 257B3 Frozen Runner Checklist

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 257C1 Frozen Runner Checklist

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 255C1 Frozen Runner Checklist

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 257C2 Frozen Runner Checklist

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 256C1 Frozen Runner Checklist

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 257C2 Implementation Checklist

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 257C3 Frozen Runner Checklist

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 257C3 Implementation Checklist

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258A Frozen Consumer Checklist

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B1 Frozen Consumer Checklist

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B2 Frozen Consumer Checklist

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

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

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2A Runner Documentation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2A Runner Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B1 Runner Prerequisite Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B1 Runner Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2A Runner Prerequisite Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2A Runner Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B1P Runner Prerequisite Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B1P Runner Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B1A Runner Prerequisite Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B1A Runner Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B1B1P Runner Prerequisite Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B1B1P Runner Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B1B1 Runner Prerequisite Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B1B1 Runner Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2P Runner Prerequisite Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2P Runner Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2A Runner Frozen-Contract Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2A Runner Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2BP Runner Frozen-Contract Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2BP Runner Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2B Runner Frozen-Contract Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2B Runner Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2CP Runner Frozen-Prerequisite Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B2C Runner Frozen-Contract Ledger

Completion evidence: [central Task-258B3M2B2B2C historical contract](../../task_contracts/en/258B3M2B2B2C.md#completion-evidence).
Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3P Runner Frozen-Contract Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3P Runner Implementation-Closure Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3A Runner Frozen-Contract Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3A Runner Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3B Runner Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3B Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3C Documentation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3C Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3D Documentation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

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

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B3M2B2B3E Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B4A Documentation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B4A Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B4B Documentation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B4B Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B4C Documentation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Task 257B3 Private Double-LF Selector Prerequisite Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B4C Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B5A Frozen-Contract Documentation Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B5A Implementation Ledger

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B5B Frozen-Contract Documentation Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B5B Lower-Stage Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B5B Upper Implementation

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 258B5C Frozen-Contract Documentation Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## B5C R-032A Preflight Overlay

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

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

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 248 Two-Parameter Runner Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 259 Frozen-Consumer Correction Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 259 Active Consumer Implementation

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 260 Frozen Consumer Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 260 Active Consumer

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 261 Frozen Attribute-Definition Consumer

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 262 Frozen Mode-Definition Consumer

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 263 Preflight Resolver Gate

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 249S No-Runner Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 263 Frozen Runner Consumer

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 264R No-Runner Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 248P No-Runner Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 249PI No-Runner Prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 269A Dormant Consumer

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 269B Dormant B3M1 Increment

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 269CP Dormant Proof-`let` Lower Projection

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 269C Dormant Binding-Only Consumer

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 269CT Dormant Runner Increment

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 269GP Dormant Lower Increment

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 269GS Canonical Scope Reconciliation

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 269G Dormant Binding Consumer

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 269GT Dormant Source-Type Consumer

Completion evidence: [central Task-269GT historical contract](../../task_contracts/en/269GT.md#completion-evidence).
Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 269GUP Dormant Use-profile Binding Prerequisite

Completion evidence: [central Task-269GUP historical contract](../../task_contracts/en/269GUP.md#completion-evidence).
Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 269GUPT Dormant Source-Type Consumer

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 269GU Dormant Term/Reference Consumer

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Task 269GCP Given-condition Lower Route

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Task 269GC Given-condition Binding Route

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Task 269GCT Given-condition Source-Type Route

Completion evidence: [central Task-269GCT historical contract](../../task_contracts/en/269GCT.md#completion-evidence).
Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Task 269GCU Given-condition Term/reference Route

Completion evidence: [central Task-269GCU historical contract](../../task_contracts/en/269GCU.md#completion-evidence).
Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 269SDP Dormant Runner Handoff

- [x] Freeze exact source/Surface/shell/resolver/debug identity and the
  lower-only zero-credit boundary in synchronized EN/JA documentation.
- [x] After docs commit `f468b0163bb00726dca9b356f48790c73bb1fe98`, add only the private lower selector/facades and
  four runner tests.
- [x] Keep every existing `.miz`, sidecar, expectation, trace row/status,
  metadata count, dispatcher, diagnostic, and active result unchanged.
- [x] Reproduce runner library `592`, production `37/79025`, and raw/normalized
  test-list hashes; focused, crate, and implementation reviews pass.
- [x] Complete final source/docs and quality gates at uncapped `100/100`.
- [ ] Commit and hand off to the separate descendant context/binding consumer;
  occurrence and capture remain later.

Completion evidence: [central Task-269SDP historical contract](../../task_contracts/en/269SDP.md#completion-evidence).

## Checker Task 269SDC Dormant Consumer Handoff

Completion evidence: [central Task-269SDC historical contract](../../task_contracts/en/269SDC.md#completion-evidence).
Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Task 269SDT

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Task 269SDU Private Runner

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 277A Direct Parser-Origin Template Transport

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Resolver Task 277R1 Test-Only Fixture Probe

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 277B-L Private Association Probe

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 257C4A private binding-context probe

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 257C4B private bound-use probe

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Resolver Task 277R2 Test-Only Fixture Probe

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 277C Private Structural Composition Probe

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Task 257C4C0 inactive capture oracle

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Task 257C4C1 lexical-admission prerequisite

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Task 257C4C7 two-capture inactive oracle

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## Checker Task 257C4C8 normalized capture graph

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

## September 2026 Audit-1 Semantic-Bridge Oracle Corpus Increment

Details archived: [test_todo_sections.md](../../archive/test_todo_sections.md).

