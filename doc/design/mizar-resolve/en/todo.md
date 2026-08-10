# mizar-resolve TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs are introduced by their own spec tasks (English and Japanese in
the same change) before the implementation tasks that cite them, following
[architecture/en/03.module_and_symbol_resolution.md](../../architecture/en/03.module_and_symbol_resolution.md).
Autonomous crate development preparation is tracked in
[00.crate_plan.md](./00.crate_plan.md).

| Module | Spec | Source | Status |
|---|---|---|---|
| resolved_ast | `resolved_ast.md` (task 2) | `src/resolved_ast.rs` | [x] |
| env | `env.md` (task 3) | `src/env.rs` | [x] |
| module_index | architecture 03 Step 1 / `mizar-build` `module_index.md` (task 7) | `src/module_index.rs` | [x] |
| imports | `imports.md` (task 8) | `src/imports.rs` | [~] |
| declarations | `declarations.md` (task 11) | `src/declarations.rs` | [x] |
| names | `names.md` (task 12) | `src/names.rs` | [~] |
| labels | `labels.md` (tasks 17 and 32) | `src/labels.rs` | [~] |
| symbols | `symbols.md` (task 19) | `src/symbols.rs` | [~] |
| recovered syntax policy | `recovery.md` (task 22) | `src/recovery.rs` helper plus stage-specific call sites | [x] |

`mizar-resolve` implements pipeline phases 4-5: `SurfaceAst` in, `ResolvedAst`
plus `SymbolEnv` out. It is the first semantic owner — namespaces, imports,
exports, labels, qualified names, and signature collection — and it is built in
two waves: first the data shapes and the import/name skeleton, then resolution
breadth growing in lockstep with `mizar-parser` grammar coverage (the resolver
cannot resolve what the parser cannot yet produce).

Dependency order: data shapes (`resolved_ast`, `env`) → `imports` → `names` /
`labels` → `symbols` (signature collection) → artifact-summary reuse.

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The crate depends on `mizar-session`, `mizar-syntax`, and the build-side
`ModuleIndexProvider` contract from `mizar-build`. It consumes `SurfaceAst`
produced through the frontend seam, so meaningful input exists only after
`mizar-parser` tasks 5-7 (module skeleton, imports, exports) land; resolution
breadth then grows with parser grammar tasks. Later, the ModuleSummary-reuse
task adds a dependency on `mizar-artifact` (schema wave).
Architecture: [03.module_and_symbol_resolution.md](../../architecture/en/03.module_and_symbol_resolution.md)
(must also refine architecture 18 and 19 per
[internal 07](../../internal/en/07.crate_module_layout.md));
IR ownership: [01.ir_layers.md](../../architecture/en/01.ir_layers.md).

## Resolved And Open Decisions

- **Dot-role finalization: resolved by task 16.** The parser leaves
  selector-versus-namespace separation syntactic (`mizar-parser` task 10,
  `mizar-syntax` task 8); `mizar_resolve::names::DotChainFinalizer` finishes
  the decision using lexical local-term scope and preserves selector validation
  for checker/type phases. Also recorded at the top level
  ([../../todo.md](../../todo.md) "Resolved And Open Decisions").
- **Interim orchestration seam: resolved by task 7.** Pipeline orchestration is
  owned by `mizar-driver`
  ([internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md));
  the resolver is a phase service, not a driver. The resolver consumes the
  build-side `ModuleIndexProvider` through
  `mizar_resolve::module_index::ModuleIndexInput`, and keeps only a
  resolver-local `WorkspaceStubModuleIndexProvider` for tests and fixtures until
  driver registry integration lands.
- **`mizar-diagnostics` adoption timing: deferred by task 13.**
  `mizar-diagnostics` remains part of the target crate layout
  ([internal 07](../../internal/en/07.crate_module_layout.md),
  [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md)).
  R-013 keeps namespace-resolution failures as crate-local/internal records,
  and R-015 keeps name diagnostics crate-local/internal, because R-G001 still
  lacks a resolver code range. Revisit before the first later user-facing
  resolver diagnostic integration.
- **ModuleSummary reuse timing: resolved by task 24.** Architecture 03 allows
  dependency modules to be consumed as `ModuleSummary` artifacts instead of
  re-read sources. The first iteration resolves the in-memory dependency
  closure. R-024 now consumes canonical `mizar-artifact` module summaries
  through the artifact-owned reader and maps validated projections into
  resolver-owned summary contribution indexes without inventing a
  resolver-local artifact format.
- **Nested proof label shadowing wording: resolved by task 17.** An earlier
  task-18 test note asked for "label shadowing across nested proofs", but
  spec chapter 15 forbids inner-scope label shadowing. R-017 classifies that
  note as `design_drift` in this derived TODO and repairs the target to
  duplicate/conflict rejection across visible label scopes.
- **Normal-source proof-label projection: planned as task 32.** R-018's
  `LabelResolver` correctly enforces scope-prefix visibility over explicit
  projections. R-023 added declaration-symbol corpus collection, not the
  production `SurfaceAst` proof-label declaration/reference collector. The
  missing lower path is Medium `source_drift`, and the older full-source-walk
  attribution to R-023 is `design_drift`.

## Ordered Task List

Keep `cargo test -p mizar-resolve` green after each task (see
[Recommended Verification](#recommended-verification)).

### Foundation

1. **Crate scaffold and lint-policy guard.** [x]
   - Add the `mizar-resolve` workspace member depending on `mizar-session` and
     `mizar-syntax`; add `tests/lint_policy.rs` mirroring the `mizar-frontend`
     guard (workspace lint opt-in, deny baseline, rationale next to any
     `allow`).
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: none. Spec: architecture 03.

2. **Spec: `resolved_ast.md`.** [x]
   - Write the `ResolvedAst` data-shape spec (English and Japanese, no code):
     `ModuleId`, `SymbolId` (stable, fully qualified), node arena,
     `NameRefTable`, `LabelRefTable`, `ResolvedImports`, explicit
     unresolved/ambiguous node representation, recovered-shell rules, and
     normalized origin/provenance fields consumed by downstream obligation
     anchors.
   - Deps: 1. Spec: architecture 03 "Interface Definitions",
     [01.ir_layers.md](../../architecture/en/01.ir_layers.md).

3. **Spec: `env.md`.** [x]
   - Write the `SymbolEnv` spec (English and Japanese, no code): the index
     family (`SymbolIndex`, `LabelIndex`, `DefinitionIndex`, `OverloadIndex`,
     `RegistrationIndex`, `NamespaceGraph`, `DeclarationDependencyIndex`),
     per-source contribution tracking, and invalidation notes.
   - Deps: 1. Spec: architecture 03 "Symbol Environment".

4. **Implement `resolved_ast` data shapes.** [x]
   - Implement the arena, tables, and id invariants exactly as task 2
     specified, plus typed accessor helpers.
   - Tests: id determinism; arena invariants (valid child ids, no cycles);
     table round-trips.
   - Deps: 2. Spec: `resolved_ast.md`.

5. **Implement `env` data shapes.** [x]
   - Implement the `SymbolEnv` index family and per-source contribution
     tracking as task 3 specified.
   - Tests: index insert/lookup round-trips; contribution tracking per source
     unit.
   - Deps: 3. Spec: `env.md`.

6. **Deterministic debug rendering.** [x]
   - Add stable, human-readable renderings of `ResolvedAst` and `SymbolEnv`
     for corpus snapshot baselines; byte-identical across runs and platforms.
   - Tests: identical output across repeated renders; fixture covering every
     current node/table kind.
   - Deps: 4, 5. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md)
     "Snapshot Tests".

7. **Module-index input contract and interim orchestration seam.** [x]
   - Resolve the interim-seam decision: define the package/module index input
     (architecture 03 Step 1) the resolver consumes as a phase service
     ([internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)),
     with a workspace-stub provider for tests until the `mizar-build`
     module-index provider and `mizar-driver` registry are integrated. Record
     the decision here and at the top level.
   - Decision: `mizar_resolve::module_index::ModuleIndexInput` borrows the
     build-side `ModuleIndexProvider` contract and forwards package,
     namespace, module, and dependency-summary lookups. The resolver converts
     build-owned module identities into resolver `ModuleId` values, but does
     not discover packages, load sources, construct module indexes, or parse
     dependency-summary artifacts. `WorkspaceStubModuleIndexProvider` is
     resolver-local test infrastructure only.
   - Tests: stub provider feeds a multi-module fixture; module identity is
     alias-independent; provider errors are deterministic.
   - Deps: 4. Spec: architecture 03 "Step 1", `mizar-build` todo tasks 5-6
     and `module_index.md`.

### Imports

8. **Spec: `imports.md`.** [x]
   - Write the import-resolution spec (English and Japanese, no code): the
     two-pass contract (frontend candidate prescan versus semantic
     validation), alias and relative-prefix rules, cycle policy, and
     unresolved-import representation.
   - Completed by [imports.md](./imports.md), paired with
     [../ja/imports.md](../ja/imports.md). The spec keeps public diagnostic
     codes out of scope until R-G001 is resolved.
   - Deps: 2. Spec: architecture 03 "Step 2",
     [12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md).

9. **Import graph construction and cycle rejection.** [x]
   - Build the semantic import graph over the module index and reject cycles
     with deterministic cycle records. Public/user-facing diagnostics remain
     gated by R-G001.
   - Implemented `src/imports.rs` canonical graph construction over
     `ModuleIndexInput`. R-010 extends the same module with alias binding,
     relative-prefix interpretation, and unresolved-import recovery from
     source-shaped paths.
   - Tests: cycle fixtures rejected deterministically; acyclic fixtures
     produce the expected graph.
   - Deps: 7, 8, `mizar-parser` task 6. Spec: `imports.md`.

10. **Import aliases, relative prefixes, and unresolved-import recovery.** [x]
    - Resolve aliases and `.`/`..` prefixes to canonical module identity;
      represent unresolved imports explicitly and keep resolving the rest of
      the module.
    - Implemented `ImportPathResolver` and source-shaped
      `ImportPathResolution` records in `src/imports.rs`, preserving alias
      spans, branch provenance, normalized path components, matched namespace
      or package candidates, and crate-local failure classes without public
      diagnostic codes. `ResolvedAst` source-walk integration remains paired
      with later import/name tasks.
    - Tests: aliases do not change canonical identity; `.`/`..` use
      dot-separated `ModulePath` directories; namespace/package bindings win
      over package-local fallback; unresolved imports do not abort module
      resolution; duplicate aliases and reserved-root aliases are explicit
      unresolved records.
    - Deps: 9. Spec: `imports.md`.

### Names

11. **Declaration shells.** [x]
    - Build local declaration shells from `SurfaceAst` items (architecture 03
      Step 3): item identity, visibility markers, export projections — no
      typing, no body resolution.
    - Implemented the source-shaped collector slice in `src/declarations.rs`
      and specified it in [declarations.md](./declarations.md). This records
      represented declaration-like items, visibility wrappers, recovered-shell
      state, transparent annotation wrappers, and export projection shells.
      Preliminary `SymbolId`s, label scopes, duplicate/illegal-declaration
      diagnostics, final export validation, and kind-specific signature
      extraction remain later name, label, and symbol work.
    - Tests: parser-produced declaration shell include/exclude inventory,
      visibility wrapper propagation, transparent annotation wrappers,
      recovered subtrees retained and flagged recovered, export projection
      shells retained without target validation.
    - Deps: 7, `mizar-parser` tasks 5 and 7. Spec: architecture 03 "Step 3".

12. **Spec: `names.md`.** [x]
    - Write the name-resolution spec (English and Japanese, no code): scope
      model, namespace-before-symbol ordering, visibility and shadowing
      rules, ambiguity representation, and the dot-chain finalization
      contract (decision recorded by task 16).
    - Added [names.md](./names.md) as the English canonical design and
      synchronized the Japanese companion. The spec keeps type-directed
      overload winner selection, selector type checking, cluster firing, and
      public resolver diagnostic-code allocation outside R-012.
    - Deps: 2. Spec: architecture 03 "Step 4",
      [11.symbol_management.md](../../../spec/en/11.symbol_management.md).

13. **Namespace resolution.** [x]
    - Resolve namespace segments before symbols (architecture 03 "Namespaces
      Resolve Before Symbols") over the import graph and declaration shells.
    - Implemented the R-013 namespace lookup slice in `src/names.rs`. It
      resolves source-shaped namespace path candidates through import aliases,
      reserved namespace roots, package-name bindings, and current-package
      fallback to canonical module namespaces. It retains internal namespace
      unresolved/ambiguous records, unresolved import-alias dependencies,
      deterministic ambiguous alias target payloads, and provider-error
      classifications. It does not look up final symbols, selector fields, or
      overload winners.
    - Tests: nested namespace fixtures, import aliases, package/current-package
      fallback, longest-prefix bindings, all reserved roots, recovered and
      malformed paths, unresolved import aliases, ambiguous aliases, provider
      errors, deterministic ordering, and missing-namespace records carrying
      the earliest failing segment range.
    - Deps: 10, 11, 12. Spec: `names.md`.

14. **Qualified names, visibility, and shadowing.** [x]
    - Resolve qualified and unqualified symbol references with visibility and
      shadowing per the spec scope rules; record results in `NameRefTable` as
      `SymbolId`s.
    - Implemented the R-014 symbol-name lookup slice in `src/names.rs`. It uses
      preliminary `NameSymbolProjection` records, declaration-point filtering,
      current-module shadowing, qualified namespace restriction, imported
      public visibility, enabled builtin fallback, failed-namespace propagation,
      and overload-group placeholder collapse without checker-owned winner
      selection.
    - Tests: qualification, current-module shadowing, declaration-point
      visibility, private dependency rejection, builtin shadowing/fallback,
      overload-group collapse, incompatible ambiguity, failed namespace,
      recovered/malformed final spellings, and deterministic table order.
    - Deps: 13. Spec: `names.md`,
      [12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md).

15. **Unresolved and ambiguous reference diagnostics.** [x]
    - Represent unresolved/ambiguous references as explicit nodes with
      deterministic candidate lists; no diagnostic cascades from a single
      unresolved root.
    - Implemented R-015 as crate-local/internal `NameDiagnosticReport` records
      in `src/names.rs`: deterministic `NameDiagnosticRootId` allocation,
      primary/cascade roles, unresolved import-alias roots, namespace/name
      dependent records, stable symbol/namespace candidate payloads, and
      record ordering without public numeric diagnostic codes.
    - Tests: ambiguity fixtures with stable candidate order; one unresolved
      import produces one primary diagnostic; mixed import-root, namespace,
      name, and symbol-ambiguity diagnostics preserve deterministic ordering.
    - Deps: 14. Spec: `names.md`,
      [22.error_handling_and_diagnostics.md](../../../spec/en/22.error_handling_and_diagnostics.md).

16. **Dot-chain finalization.** [x]
    - Finish the unresolved dot chains the parser left syntactic: selector
      access versus namespace separation by variable scope. Record the
      decision in `names.md` and close the top-level open decision.
    - Implemented R-016 in `src/names.rs` with `LocalTermScope`,
      `LocalTermBinding`, `DotChainCandidate`, `DotChainFinalizer`, and
      `DotChainResolution`. In-scope local terms shadow namespace heads and
      produce `DeferredSelector` records using the use-site base node; otherwise
      the leading path resolves through `NamespaceResolver` and the final
      segment through qualified `SymbolNameResolver`.
    - Tests: selector/namespace splits from spec §A.2.5 examples; chains that
      fit neither role are diagnosed; out-of-scope locals do not shadow
      namespaces; innermost local binding wins; output order is deterministic.
    - Deps: 14, `mizar-parser` task 10. Spec:
      [§A.2.5](../../../spec/en/appendix_a.grammar_summary.md).

### Labels

17. **Spec: `labels.md`.** [x]
    - Write the label-resolution spec (English and Japanese, no code): the
      separate label scope family, proof-block nesting, forward-reference
      policy, and normalized label-origin paths used by downstream
      `ObligationAnchor` construction.
    - Completed by [labels.md](./labels.md), paired with
      [../ja/labels.md](../ja/labels.md). The spec keeps proof validity,
      template instantiation, ATP premise selection, `ObligationAnchor`
      construction, and public resolver diagnostic-code allocation outside
      R-017.
    - Deps: 2. Spec: architecture 03 "Label Resolution Is Scoped Separately".

18. **Label resolution.** [x]
    - Resolve statement/theorem labels per task 17, including proof-block
      nesting.
    - Implemented R-018 in `src/labels.rs` with `LabelScopePath`,
      `LabelProjection`, `LabelReferenceCandidate`, `LabelResolver`,
      `LabelResolutionResult`, and crate-local/internal `LabelDiagnostic`
      records. The executable slice resolves theorem/lemma and proof-step
      label projections, rejects forward references, handles qualified/grouped
      item candidates through already resolved namespace/module projections,
      and populates deterministic `LabelIndex` / `LabelRefTable` outputs
      without public diagnostic codes or proof/VC semantics.
    - Tests: proof-block visibility; duplicate/conflicting labels across visible
      nested proof scopes rejected; references to later labels rejected; simple,
      qualified, and lowered grouped-item citation lookup where parser coverage exists;
      deterministic `LabelIndex` / `LabelRefTable` / diagnostic ordering.
    - Deps: 11, 17, `mizar-parser` task 22. Spec: `labels.md`,
      [16.theorems_and_proofs.md](../../../spec/en/16.theorems_and_proofs.md).

### Signature collection

19. **Spec: `symbols.md`.** [x]
    - Write the signature-collection spec (English and Japanese, no code):
      declaration-pass contract (no type checking), per-kind signature
      shapes, duplicate/illegal-declaration policy, and normalized semantic
      origins that remain stable under formatting and unrelated local edits.
    - Deps: 3. Spec: architecture 03 "Step 5",
      [11.symbol_management.md](../../../spec/en/11.symbol_management.md).
    - Completed by R-019: `symbols.md` specifies the resolver-owned
      signature-collection contract, stable symbol origins, symbol-bearing
      shell classification, per-kind opaque / structural payloads including
      algorithms, duplicate/conflict and overload policy, exported summary and
      lexical-summary projections, recovery, relation/dependency edges, and the
      R-020/R-021/R-023 test handoff.

20. **Collection skeleton and duplicate detection.** [x]
    - Populate `SymbolEnv` from declaration shells: registration of names per
      kind, duplicate and conflict diagnostics, overload candidate grouping —
      signatures still opaque.
    - Tests: duplicate detection per kind; candidate grouping; deterministic
      diagnostic order.
    - Deps: 5, 11, 19. Spec: `symbols.md`.
    - Completed by R-020: `src/symbols.rs` adds the explicit
      `DeclarationShellId`-keyed projection seam, opaque symbol collection into
      `SymbolIndex` / `DefinitionIndex` / `RegistrationIndex` / `OverloadIndex`,
      internal duplicate and illegal-overload diagnostics, recovered and
      context-only shell policy, contribution tracking, and deterministic unit
      tests. Dedicated lexical-summary data shapes are completed by R-021.
      Artifact-backed summary consumption is implemented by R-024 as a
      canonical `mizar-artifact` `ModuleSummary` consumer.

21. **Per-kind signature extraction.** [x] — paced by `mizar-parser` tasks 23-31.
    - Extract concrete signatures (structs, modes, attributes, predicates,
      functors, algorithms, theorems, registrations, templates, and relation
      declarations such as synonyms, antonyms, and redefinitions)
      incrementally: each increment lands after the parser grammar task that
      produces the declaration kind, in its own change. Checked off when the
      last paired increment lands.
    - Tests per increment: signature shape fixtures and `SymbolEnv` lookups
      for that kind.
    - Deps: 20; pairs with `mizar-parser` tasks 23-31. Spec: `symbols.md`.
    - Completed by R-021: `SignatureProjectionExtractor` lowers represented
      parser-backed declaration shells into `SymbolDeclarationProjection`
      records with parser-owned opaque signature payloads, preserves template
      roles in the owning declaration payload, and seeds
      `ModuleLexicalSummaryIndex` entries for exported lexer-visible spellings.
      Module-level scheme declarations remain an external source-role gap until
      parser/syntax exposes a scheme declaration shell.

### Hardening and cross-cutting follow-ups

22. **Recovered-syntax policy.** [x]
    - Define and implement how each resolver stage treats recovered
      `SurfaceAst` subtrees (skip, shell-only, or diagnose), keeping the
      `recovered` flag contract from `mizar-syntax`.
    - Tests: recovered input never panics resolution; diagnostics do not
      cascade from recovered regions.
    - Deps: 13. Spec: [mizar-syntax recovery.md](../../mizar-syntax/en/recovery.md).
    - Completed by R-022: added [recovery.md](./recovery.md), centralized
      resolver-local recovered-subtree detection, and made name, label, and
      symbol diagnostics suppress dependent semantic diagnostics from recovered
      origins or shells while retaining degraded table/env facts.

23. **Corpus runner at stage `declaration_symbol`.** [x]
    - Wire `tests/miz/{pass,fail}/` cases at stage `declaration_symbol`
      through the harness, with `spec_trace.toml` coverage entries; seed an
      initial spec-derived pass/fail set for the declaration-symbol path and
      record broader semantic corpus growth toward the 40/60 pass/fail mix as
      explicit follow-up coverage.
    - Deps: 20. Spec: [staged_model.md](../../mizar-test/en/staged_model.md),
      [traceability.md](../../mizar-test/en/traceability.md).
    - Completed by R-023: `mizar-test declaration-symbol` now discovers
      active `.miz` expectations tagged `active_declaration_symbol` at
      `stage = "declaration_symbol"` / `expected_phase = "resolve"`, runs the
      frontend plus resolver declaration-shell, signature-projection, and
      symbol-collection path, and compares fail cases against crate-local
      internal detail keys in `diagnostic_payloads` / `stable_detail_key`
      without inventing public resolver diagnostic codes. Seed corpus coverage
      includes one pass smoke fixture for parser-backed declarations,
      visibility, and theorem/lemma symbols, plus one duplicate-theorem fail
      fixture derived from same-scope label uniqueness, with `spec_trace.toml`
      requirements.
    - Post-task-20 R-G007 increment: the same active runner now also executes
      the parser-backed functor signature-conflict seed for same
      argument-signature definitions with different return signatures, using
      the resolver-owned internal `SameSignatureReturnConflict` class and the
      `declaration_symbol.signature.same_signature_return_conflict` detail
      key. Broader semantic import/name/label corpus growth for tasks 9-19 is
      still recorded as R-G007 test-gap follow-up for future runner assertion
      expansions, but the executable declaration-symbol runner now has three
      traceable active cases.
    - Post-task-20 R-G007 pass-payload increment: the active pass smoke
      fixture exact-compares SymbolEnv-derived symbol and definition fact keys
      for represented kind, visibility, and export status. It does not add
      import graph, namespace/name resolution, dot-chain, label-reference,
      checker, CoreIr, VC, or proof assertions.

24. **ModuleSummary reuse.** [x]
    - Consume dependency modules as `ModuleSummary` artifacts (schema-version
      checked) instead of re-reading sources; fall back to source resolution
      when summaries are absent or incompatible.
    - Tests: summary-backed and source-backed resolution agree on a shared
      fixture; incompatible schema falls back with a diagnostic.
    - Deps: 20, `mizar-artifact` task 5. Spec: architecture 03 "Module
      Summary", [18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md).
    - Completed by R-024: `src/module_summary_reuse.rs` consumes canonical
      `mizar-artifact` `ModuleSummary` JSON through the artifact-owned reader,
      projects validated exported symbols, labels, lexical entries, re-exports,
      and dependency interface references into `SymbolEnv` summary
      contribution indexes, and produces deterministic crate-local fallback
      records for missing, incompatible, or unsupported summaries. The resolver
      does not define artifact schemas, readers, writers, hash framing, public
      diagnostics, or source loading for artifact-only dependency modules.

25. **Determinism suite.** [x]
    - Property coverage that identical inputs produce identical ids, tables,
      diagnostic order, and debug renderings, mirroring the frontend suite.
    - Deps: 21. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).
    - Completed by R-025: added a crate-root determinism regression that
      builds equivalent public-seam inputs twice and compares import graph
      resolution, name diagnostic order, `ResolvedAst` debug rendering, and
      `SymbolEnv` debug rendering. Existing module-local determinism tests
      continue to cover detailed id, table, candidate, and diagnostic ordering
      within `resolved_ast`, `env`, `imports`, `names`, `labels`, and `symbols`.

26. **Public-enum forward-compatibility policy.** [x]
    - Apply the `mizar-frontend` task-25 decision procedure to each public
      enum; record each decision next to the enum in the owning module spec.
    - Deps: 21. Spec: all module specs.
    - Completed by R-026: audited resolver-owned public enums in
      `resolved_ast`, `env`, `imports`, `declarations`, `names`, `labels`, and
      `symbols`; every public enum remains `#[non_exhaustive]` with no
      exhaustive exception. The owning module specs now list the decision and
      downstream wildcard/fallback requirement, and `mizar-resolve` lint tests
      guard future public-enum additions in those spec-owned modules.

27. **Source/spec correspondence audit.** [x]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 26. Spec: all module specs and this TODO.
    - Completed by R-027: [source_spec_correspondence.md](./source_spec_correspondence.md)
      records public API family, behavior-boundary, task-requirement, and
      follow-up traceability. The audit found no unclassified blocking/high
      `spec_gap`, `test_gap`, `source_drift`, `source_undocumented_behavior`,
      `test_expectation_drift`, `boundary_violation`, or
      `repo_metadata_conflict`. Existing classified records remain: R-G001
      public resolver diagnostic code-space `spec_gap`, R-G002 historical
      semantic corpus coverage `test_gap` refined by R-G007, and R-G006
      parser/syntax scheme-role dependency. The post-task-20 R-G007
      signature-conflict slice closes one symbol assertion increment but leaves
      import/name/dot-chain/label active assertions open. R-G003 deferred
      `ModuleSummary` reuse is resolved by R-024.

28. **Bilingual documentation sync audit.** [x]
    - Compare each English canonical document under
      `doc/design/mizar-resolve/en/` with its Japanese companion and
      synchronize API lists, statuses, terminology, links, and behavior
      promises.
    - Deps: 27. Spec: repository documentation policy.
    - Completed by R-028: [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md)
      records the paired English/Japanese design-document checklist. The audit
      found no remaining mismatch in public API families, enum policy, task
      states, deferred/external-dependency records, behavior promises, boundary
      statements, terminology, or resolver task handoff wording. No `doc/spec`,
      `.miz`, expectation, or source files changed.

29. **Module-boundary refactor gate.** [x]
    - Before treating the crate as ready for downstream consumers, audit the
      source layout for oversized files, mixed responsibilities, and private
      helpers that should be split along the module table and spec boundaries.
      Split any review-bottleneck implementation files into private modules
      without changing public APIs, diagnostics, deterministic renderings,
      artifact-facing schemas, or consumer-visible behavior.
    - After any split, update this module table/source paths as needed and
      re-run the source/spec and bilingual documentation audit scopes for the
      moved APIs. Do not mix behavior cleanup or API exposure into the move;
      those require their own spec tasks.
    - Deps: 28. Spec: this TODO,
      [internal 07](../../internal/en/07.crate_module_layout.md), all module
      specs.
    - Completed by R-029: [module_boundary_refactor.md](./module_boundary_refactor.md)
      records the source-layout audit and behavior-preserving split. Public
      module paths and APIs stayed unchanged; inline unit tests moved to
      per-module private `tests.rs` files; deterministic snapshot helpers moved
      to `env/snapshot.rs` and `resolved_ast/snapshot.rs`; resolved-AST
      validation moved to `resolved_ast/validation.rs`; and crate-local name
      diagnostic assembly moved to `names/diagnostics.rs`. Source/spec and
      bilingual documentation audit scopes were re-run for the moved APIs with
      no new drift.

30. **Public resolver diagnostic adoption gate.** [ ]
    - Map resolver name/import/label diagnostics into public
      `mizar-diagnostics` descriptors only when a real user-facing producer
      adoption task starts. The shared registry already reserves the
      `Resolution` family, but this task must define concrete semantic names,
      numeric codes or aliases, migration behavior for crate-local diagnostics,
      corpus/expectation coverage, and LSP/artifact projection boundaries.
    - Deps: R-024 and the first downstream consumer that needs user-facing
      resolver diagnostics. Spec:
      [22.error_handling_and_diagnostics.md](../../../spec/en/22.error_handling_and_diagnostics.md),
      [mizar-diagnostics consumer adoption](../../mizar-diagnostics/en/consumer_adoption_decision.md),
      [spec_coverage_audit.md](../../spec_coverage_audit.md).
    - Forbidden behavior: no placeholder adapters, no invented public codes
      without registry/spec alignment, and no rebaselining existing expectation
      sidecars merely to match current crate-local diagnostics.

31. **Same-signature/same-return declaration conflict.** [x]
    - Extend only resolver-owned declaration/signature collection so two
      ordinary declarations with the same symbol kind, spelling, arity, and
      argument signature conflict even when their return signatures are also
      the same, as required by Chapter 19 and checker Task 37. Preserve source
      declaration identities and deterministic diagnostic ordering; do not run
      checker semantic type equivalence or overload winner selection.
    - Add the existing deferred same-return seed to the declaration-symbol
      runner only after exact resolver unit/corpus, near-miss, order, recovery,
      and sidecar tests pass. Update its trace row from deferred only in the
      implementation task; do not rebaseline the existing different-return
      expectation.
    - Exact contract: only ordinary functor definitions represented by the
      `Functor` symbol/definition kind pair participate. Group by namespace,
      primary spelling, normalized notation pattern, normalized definition
      argument context, and syntactic arity. An all-return-identical group uses
      new appended non-exhaustive `SameSignatureDefinitionConflict` diagnostic
      and definition-conflict variants and the exact
      `declaration_symbol.signature.same_signature_definition_conflict` key.
      A mixed/different-return group retains one existing
      `SameSignatureReturnConflict` diagnostic over every candidate and the
      existing detail key; that classification wins and no overlapping
      diagnostic is emitted.
    - Preserve canonical first shell/range and candidate order under input
      permutations. Assert parser-backed and explicit projections, exact and
      near-miss kind/spelling/pattern/context/arity/namespace cases, recovered
      suppression, mixed-return priority, the activated same-return sidecar,
      and the byte-identical different-return sidecar. Change only the exact
      same-return trace row to covered and update paired component/runner docs
      plus `spec_coverage_audit.md`.
    - Deps: R-021/R-023; independent Task-49 prerequisite. Spec: Chapter 19
      section 19.1; checker Task 37 and its deferred trace row.
    - Completed by R-031: ordinary-functor collection now emits the appended
      `SameSignatureDefinitionConflict` diagnostic/definition metadata for an
      all-return-identical exact argument-key group, preserves the existing
      different-return class for mixed groups, suppresses recovered/nonordinary
      near misses, serializes the new conflict as
      `same_signature_definition_conflict`, and executes the existing exact
      same-return seed through the active declaration-symbol runner. The sole
      trace row is covered; the different-return expectation is unchanged.

32A. **Validated Surface structural arena.** [x]
    - Implement the exact `SurfaceResolvedArena` / public non-exhaustive
      `SurfaceResolvedArenaError` contract in `resolved_ast.md`.
    - Own exactly `resolved_ast.rs`, `resolved_ast/tests.rs`,
      `tests/lint_policy.rs` for the sole R-026
      `SurfaceResolvedArenaError` owning-spec decision entry, and paired docs.
      Prove complete child-first same-index mapping, exact structural/origin
      preservation, every named mismatch/error variant including
      `ResolutionStateMismatch` / `ReferenceKeyMismatch`, checked overflow,
      and downstream wildcard compatibility.
    - This is one lower-prerequisite commit after the B5C documentation commit.
      It changes no labels, runner, fixtures, sidecars, trace, or other phase.
    - Implementation preflight classified the previous two-Rust-file scope as
      High `design_drift`: the mandatory R-026 public-enum guard scans the new
      enum. Complete the synchronized docs-only scope correction in a separate
      commit, then fresh-inventory before the exact three-Rust-file
      implementation.
    - Completed: `SurfaceResolvedArena` now lowers every dense surface node to
      its same-index structural node, validates the exact frozen precedence,
      exposes the frozen accessors/error surface, and carries no semantic
      resolution. The complete mismatch/precedence/helper/determinism matrix
      and the sole R-026 enum decision pass without changing labels, active
      artifacts, trace state, or coverage credit.

32B. **Normal-source proof-label confinement projections.** [x]
    - After fresh inventory of R-032A, implement the exact
      `ProofLabelSourceCollector` / public non-exhaustive
      `ProofLabelSourceCollectionError` contract in `labels.md`.
      Use the exact `'a` impl lifetime: store only AST/arena borrows, own
      namespace/contribution, and do not store the validated module argument.
    - Own exactly `labels.rs`, `labels/tests.rs`,
      `tests/lint_policy.rs` for the sole R-026
      `ProofLabelSourceCollectionError` owning-spec decision with
      `spec_name: "labels.md"`, and paired docs. Use only the validated R-032A
      map; no callback, unmapped side channel, fabricated id, unchecked
      conversion, or semantic descendant collection.
    - Prove exact supported-subtree/exclusion behavior, root `[0]`, `[1]`, ...
      and nested scopes, the shared module-global one-based ordinal walk,
      completion boundaries, exact `proof-step-v1` origin serialization,
      `[0] -> [0,1]` visibility, inner/sibling/cross-theorem confinement,
      own-proof pre-completion rejection, same-block post-completion success,
      overflow/recovery, and deterministic mutation matrices.
      Implement the exhaustive default-deny direct Surface edge table from
      `labels.md`: positive evidence per permitted edge, exact mixed-list
      filtering, relocation/wrapper/formula/computation/qualified/grouped/bulk/
      template/owner/recovery negatives, and every row's all-other action.
      Prove `Root` -> `CompilationUnit` -> `ItemList` -> `TheoremItem` one edge
      at a time and reject missing/additional/wrong/relocated/wrapped upper
      shapes, including theorem-under-root/compilation-unit/visible-item.
    - This is a second lower-prerequisite commit before private `mizar-test`
      active key `declaration_symbol.label.proof_scope_confinement`. Public
      checker handoff and active artifacts remain excluded.
    - Deps: R-018/R-023, B5C docs, R-032A; authority Chapter 15 §15.10 and
      Chapter 16 §§16.4.2/16.5.1.
    - Completed: the exact public collector/collection/error source and sole
      R-026 decision are present. Every specification/test/implementation/
      source-documentation review reports **NO FINDINGS**; focused/full/count/
      hash/scope gates and all nine hard gates pass without a score cap at
      valid `100/100` (`20/20/15/15/10/10/5/5`). Task-only staging, commit
      `b3a7e79a6b60db2974e911c69bb56ff5f4609064`, and post-commit fresh
      inventory are complete.

## Crate Close-Out

- Completed: [crate_exit_report.md](./crate_exit_report.md) records
  non-deferred task completion, the original R-024 deferral, the R-024
  follow-up implementation overlay, milestone gates, quality score 94/100,
  full verification, human-review surface, task commits, and next-task handoff.
  R-030 is a later integration follow-up opened by the spec-coverage audit; it
  does not reopen the completed R-001 through R-029 milestone. R-031 is an
  independently authorized Step-5 corpus increment and likewise does not
  rewrite that completed milestone. Planned R-032A/R-032B are separate lower
  prerequisites; their documentation and implementation commits do not reuse or
  reopen the original 94/100 score.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-resolve
cargo clippy -p mizar-resolve --all-targets -- -D warnings
```

For tasks that touch the frontend seam, corpus, or shared boundaries, also run:

```text
cargo test -p mizar-syntax
cargo test -p mizar-frontend
cargo test -p mizar-test
```

For normalized origin, label-origin, or symbol-origin fields consumed by
obligation anchors, also run:

```text
cargo test -p mizar-core
cargo test -p mizar-vc
```

Check the task off here once tests pass.

## Notes

- The resolver owns names, scopes, imports, exports, labels, and signature
  collection only: no type inference, no overload winner selection, no
  cluster facts, no proof semantics.
- Downstream phases use `SymbolId`, never raw strings, once a symbol
  resolves; unresolved/ambiguous nodes stay explicit for diagnostics.
- Resolution breadth is paced by `mizar-parser` grammar tasks; do not build
  resolution for syntax the parser cannot yet produce.

## R-032A Lower-Stage Prerequisite Overlay

- [x] Complete the separate mizar-syntax S-026 frozen-documentation commit,
  proving the planned dense-id contract for every stored compatibility node,
  including disconnected nodes.
- [x] Complete the separate S-026 implementation with the exact dense accessor,
  complete unit/rustdoc matrix, and passing review/verification gates.
- [x] After its dedicated commit, fresh-inventory R-032A; identify the
  mandatory R-026 enum-decision owner omission as High `design_drift`.
- [x] Complete the separate R-032A lint-policy docs correction through
  findings-free reviews, full verification, and valid `100/100` final quality.
- [x] After its dedicated commit, verify repository/stash invariants and
  fresh-inventory R-032A implementation.
- [x] Implement R-032A only after that correction, using
  `SurfaceAst::node_views()` and
  the exact validation precedence in [resolved_ast.md](./resolved_ast.md).
- [x] Reject unsafe ids, dummy-AST id minting, resolver-side syntax mutation,
  and any combined syntax/resolver commit as `boundary_violation`.

## R-032B Lint-Policy Frozen-Scope Prerequisite

- [x] Fresh-inventory the mandatory R-026 guard and classify the omitted
  `tests/lint_policy.rs` owner as High `design_drift`, with no semantic
  `spec_gap`, `test_gap`, or test-intent change.
- [x] Complete the current synchronized docs-only prerequisite across exactly
  31 design files: 16 resolver, eight checker, six `mizar-test`, and one
  global ledger.
- [x] Complete independent specification, test/scope, and
  source/documentation consistency reviews with **NO FINDINGS** and pass the
  docs-only verification/count/hash gates.
- [x] Complete independent final read-only quality with **NO FINDINGS**, all
  nine hard gates PASS, no score cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Complete task-only staging, correction commit
  `f1cf0a5d15f2db51176e9e91a4f5a6447a88ad7a`, post-commit invariants, and
  fresh implementation inventory.
- [x] Complete the current R-032B implementation in exactly
  `src/labels.rs`, `src/labels/tests.rs`, and the sole R-026
  `ProofLabelSourceCollectionError` / `labels.md` decision in
  `tests/lint_policy.rs`; fix every review finding, complete final fresh
  no-findings rereviews, and pass focused/full/count/hash/scope verification.
- [x] Complete independent final quality with **NO FINDINGS**, all nine hard
  gates PASS, no score cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Complete task-only restaging/cached-diff review, dedicated commit
  `b3a7e79a6b60db2974e911c69bb56ff5f4609064`, and post-commit
  invariant/fresh inventory.
- [x] Continue only afterward to the active B5C consumer. The effective order
  remains S-026 docs, S-026 implementation, R-032A lint docs, R-032A
  implementation, R-032B lint docs, R-032B implementation, active B5C, with
  fresh inventory between commits.
- [x] Keep specification, semantics, test intent, source, fixtures, sidecars,
  expectations, trace status/counts, and Cargo metadata unchanged in this
  prerequisite. `spec_coverage_audit.md` is a deliberate no-op because no
  chapter coverage, traceability, owner, deferral, or coverage credit changes.

## Checker Task 258B5C Active Consumer

- [x] Privately consume unchanged R-032A `SurfaceResolvedArena` and R-032B
  `ProofLabelSourceCollector` / `LabelResolver` APIs in `mizar-test`, with no
  resolver production or public API change.
- [x] Add exactly two fail fixtures, two expectation sidecars, and two covered
  trace rows for inner-to-outer and sibling confinement.
- [x] Update four frozen active-count/CLI assertions in
  `crates/mizar-test/tests/metadata.rs` from declaration stage `5` to `7`.
- [x] Record plan `421/389`, pass/fail `228/193`, active
  parse/declaration/type/proof `101/7/198/1`, and warnings/errors `23/0`.
- [x] Keep public diagnostic codes empty and authenticate only through private
  key `declaration_symbol.label.proof_scope_confinement`.
- [x] Close only these two R-G007 negatives; keep import, name, dot-chain, and
  other label-reference coverage open.
- [x] Complete findings-free test, implementation, and source/documentation
  reviews plus focused/crate/workspace/count/hash verification gates.
- [x] Complete independent final quality with **NO FINDINGS**, all nine hard
  gates PASS, no score cap, and valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [x] Complete task-only cached-diff review, dedicated B5C commit
  `33ac57e96f048dc40559565f54369cac854409a7`, and post-commit fresh inventory.

## Checker Task 263R Selector-Owner Prerequisite

- [x] Fresh-inventory Task 263 authority, parser/resolver behavior, resolver
  public/source boundaries, counts/hashes, origin delta, and protected stash.
- [x] Classify cross-structure selector duplication as `source_drift`, the
  name-level design rule as `design_drift`, missing regression coverage as
  `test_gap`, and origin divergence as report-only `repo_metadata_conflict`.
- [x] Freeze the exact 320-byte probe/hash and measured `75/10/8/8/2`
  AST/shell/projection/symbol/diagnostic profile.
- [x] Freeze selector-only nearest-structure ownership, conservative missing-
  owner behavior, same-owner collisions, all non-selector exclusions, two
  Rust owners/tests, unchanged public API, and exit criteria.
- [x] Keep production/tests, fixtures, sidecars, expectations, trace status/
  counts, CLIs, and Cargo metadata unchanged in this docs prerequisite.
- [x] Complete repeated specification, test-sufficiency, and source/
  documentation boundary reviews with **NO FINDINGS** and pass focused/full/
  metadata/lint/format/Clippy/CLI/count/hash/scope gates.
- [x] Complete findings-free final read-only quality with all nine hard gates
  PASS, no score cap, and valid `100/100` (`20/20/15/15/10/10/5/5`).
- [x] Complete exact staging and dedicated documentation commit
  `34692ee222d5465750f061da82fe878566a1557c`, then confirm clean post-commit
  inventory and the unchanged protected stash.
- [x] Fresh-inventory and implement the frozen selector-owner partition in
  exactly `src/symbols.rs` and `src/symbols/tests.rs`; add exactly the two
  frozen tests and preserve every public and non-selector contract.
- [x] Complete test-sufficiency and implementation reviews with
  **NO FINDINGS**; pass focused resolver tests, resolver Clippy, formatting,
  and both exact probes.
- [x] Complete source/documentation consistency with **NO FINDINGS**, all full
  verification gates, and independent final quality with all nine hard gates
  PASS, no score cap, and valid `100/100` (`20/20/15/15/10/10/5/5`).
- [x] Complete exact staging, the dedicated implementation commit
  `997457dd3189030aa3b137b568ce82fed456fe1e`, and post-commit fresh
  inventory; then return to Checker Task 263.

## Checker Task 264R Property-Implementation Shell Prerequisite

- [x] Fresh-inventory Chapter 7, Parser Task 48 pass/recovery fixtures,
  declaration/symbol public APIs, source behavior, counts/hashes, origin delta,
  and protected stash.
- [x] Classify dropped represented shells as `source_drift`, the missing
  context-only/no-symbol contract as `design_drift`, absent regressions as
  `test_gap`, identity reuse as `boundary_violation`, and origin divergence as
  report-only `repo_metadata_conflict`.
- [x] Freeze parentless source-ordered shell intake, range/recovery provenance,
  no projection/identity/property-owned contribution effect, preservation of
  the existing `LocalSource` contribution/anchor and sibling fingerprints,
  exact two tests, pass/recovery
  profiles and hashes, implementation scope, count impact, exclusions, and
  exit criteria in synchronized EN/JA documentation.
- [x] Repeat review-only specification and boundary audits to **NO FINDINGS**;
  pass docs-only verification and all nine final-quality gates at uncapped
  `100/100`.
- [x] Stage and commit only the synchronized documentation prerequisite as
  `b1ed8ea19f8845d8c54f795a7375d4add4af237d`; confirm clean inventory and
  protected-stash invariance.
- [x] Fresh-inventory and implement exactly the four frozen resolver files,
  adding exactly two tests and changing the resolver library count `146 ->
  148` with no corpus/checker/runner/trace/Cargo delta.
- [x] Complete findings-free test-sufficiency, implementation, and source/docs
  reviews plus all focused/full verification and count/hash gates.
- [ ] Complete findings-free final quality, exact staging, implementation
  commit, and clean fresh inventory; then select Checker Task 248P.

## Resolver Task 277R1 Frozen Documentation Prerequisite

- [x] Freeze the canonical [Task 277R1 contract](../../task_contracts/en/RESOLVE-TEMPLATE-TYPEPARAM-277R1.md),
  authority, exact parser profile, `source_drift`/`design_drift`/Rust
  `test_gap`, four-path Rust scope, five named tests, row ABI, counts, and
  protected hash/no-impact decisions.
- [x] Commit the documentation prerequisite as `2438cbb7d39c1844557293b270ef1784cfc31ece`,
  reproduce every frozen baseline, and implement only the four frozen Rust
  paths and five named tests. Preserve the private fixture boundary and every
  semantic/active/protected exclusion.
- [x] Repair test-discrimination, global link-order, and recovered-subtree
  findings; finish independent test/implementation reviews at **NO FINDINGS**
  and pass focused, crate, workspace, lint, metadata, formatting, Clippy, CLI,
  and count/hash verification.
- [x] Finish independent source/documentation and bilingual reviews at **NO
  FINDINGS**.
- [x] Finish independent final-quality review at **NO FINDINGS** with all nine
  hard gates passing, no score cap, and valid `100/100`.
- [x] Complete exact staging, task-only implementation commit
  `b22033c38249326e366ceb9e19b1a9100da2248e`, immediate post-implementation
  proof, and fresh successor inventory. Task 277B remains not ready.

## Resolver Task 277R2 Frozen Documentation Prerequisite

- [x] Freeze the canonical [Task 277R2 contract](../../task_contracts/en/RESOLVE-FRAENKEL-GENERATOR-VAR-277R2.md),
  authority, exact F5 binding/use profile, exhaustive public ABI, default-deny
  matrix, exact five-path Rust scope, named `4 + 1` tests, baselines, and
  protected no-impact decisions.
- [x] Implement only the five frozen Rust paths and five named tests, preserving
  the private fixture boundary and all semantic, active, trace, coverage, and
  production exclusions.
- [x] Complete independent test-sufficiency and implementation reviews at **NO
  FINDINGS**; pass focused `4 + 1`, resolver and mizar-test library/lint,
  package and workspace Clippy, workspace tests, metadata, formatting, diff,
  five CLI hashes, and protected path-hash verification.
- [x] Complete final source/documentation/API, bilingual, and boundary
  integration reviews at **NO FINDINGS**.
- [x] Complete final-quality review at **NO FINDINGS**, with all nine hard
  gates passing without a score cap at valid `100/100`
  (`20/20/15/15/10/10/5/5`).
- [ ] Complete exact staging/cached-diff review, task-only implementation
  commit, post-commit proof, and fresh successor inventory. Task 277B remains
  not ready with zero semantic credit.
