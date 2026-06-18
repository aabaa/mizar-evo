# mizar-resolve TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs do not exist yet; each is written by its own spec task (English
and Japanese in the same change) before the implementation tasks that cite it,
following
[architecture/en/03.module_and_symbol_resolution.md](../../architecture/en/03.module_and_symbol_resolution.md).

| Module | Spec | Source | Status |
|---|---|---|---|
| resolved_ast | `resolved_ast.md` (task 2) | `src/resolved_ast.rs` | [ ] |
| env | `env.md` (task 3) | `src/env.rs` | [ ] |
| imports | `imports.md` (task 8) | `src/imports.rs` | [ ] |
| names | `names.md` (task 12) | `src/names.rs` | [ ] |
| labels | `labels.md` (task 17) | `src/labels.rs` | [ ] |
| symbols | `symbols.md` (task 19) | `src/symbols.rs` | [ ] |

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

The crate depends on `mizar-session` and `mizar-syntax`. It consumes
`SurfaceAst` produced through the frontend seam, so meaningful input exists
only after `mizar-parser` tasks 5-7 (module skeleton, imports, exports) land;
resolution breadth then grows with parser grammar tasks. Later, the
ModuleSummary-reuse task adds a dependency on `mizar-artifact` (schema wave).
Architecture: [03.module_and_symbol_resolution.md](../../architecture/en/03.module_and_symbol_resolution.md)
(must also refine architecture 18 and 19 per
[internal 07](../../internal/en/07.crate_module_layout.md));
IR ownership: [01.ir_layers.md](../../architecture/en/01.ir_layers.md).

## Resolved And Open Decisions

- **Dot-role finalization: open, resolved by task 16.** The parser leaves
  selector-versus-namespace separation syntactic (`mizar-parser` task 10,
  `mizar-syntax` task 8); the resolver finishes the decision using variable
  scope. Registered at the top level
  ([../../todo.md](../../todo.md) "Resolved And Open Decisions").
- **Interim orchestration seam: open, resolved by task 7.** Pipeline
  orchestration is owned by `mizar-driver`
  ([internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md));
  the resolver is a phase service, not a driver. The open part is the interim
  module-index input the resolver consumes until the `mizar-build` module-index
  provider and the `mizar-driver` phase registry are integrated
  (workspace-stub provider is the default candidate).
- **`mizar-diagnostics` adoption timing: open, decided before task 13.**
  `mizar-diagnostics` is part of the target crate layout
  ([internal 07](../../internal/en/07.crate_module_layout.md),
  [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md));
  the resolver is the first crate whose diagnostics span multiple files.
  Decide whether to introduce the shared diagnostic record now or keep
  per-crate diagnostics one more layer. Registered at the top level.
- **ModuleSummary reuse timing: open, resolved by task 24.** Architecture 03
  allows dependency modules to be consumed as `ModuleSummary` artifacts
  instead of re-read sources. The first iteration resolves the in-memory
  dependency closure; the artifact-backed path needs `mizar-artifact`
  module-summary schema first. Registered at the top level.

## Ordered Task List

Keep `cargo test -p mizar-resolve` green after each task (see
[Recommended Verification](#recommended-verification)).

### Foundation

1. **Crate scaffold and lint-policy guard.** [ ]
   - Add the `mizar-resolve` workspace member depending on `mizar-session` and
     `mizar-syntax`; add `tests/lint_policy.rs` mirroring the `mizar-frontend`
     guard (workspace lint opt-in, deny baseline, rationale next to any
     `allow`).
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: none. Spec: architecture 03.

2. **Spec: `resolved_ast.md`.** [ ]
   - Write the `ResolvedAst` data-shape spec (English and Japanese, no code):
     `ModuleId`, `SymbolId` (stable, fully qualified), node arena,
     `NameRefTable`, `LabelRefTable`, `ResolvedImports`, explicit
     unresolved/ambiguous node representation, recovered-shell rules, and
     normalized origin/provenance fields consumed by downstream obligation
     anchors.
   - Deps: 1. Spec: architecture 03 "Interface Definitions",
     [01.ir_layers.md](../../architecture/en/01.ir_layers.md).

3. **Spec: `env.md`.** [ ]
   - Write the `SymbolEnv` spec (English and Japanese, no code): the index
     family (`SymbolIndex`, `DefinitionIndex`, `OverloadIndex`,
     `RegistrationIndex`, `NamespaceGraph`), per-source contribution
     tracking, and invalidation notes.
   - Deps: 1. Spec: architecture 03 "Symbol Environment".

4. **Implement `resolved_ast` data shapes.** [ ]
   - Implement the arena, tables, and id invariants exactly as task 2
     specified, plus typed accessor helpers.
   - Tests: id determinism; arena invariants (valid child ids, no cycles);
     table round-trips.
   - Deps: 2. Spec: `resolved_ast.md`.

5. **Implement `env` data shapes.** [ ]
   - Implement the `SymbolEnv` index family and per-source contribution
     tracking as task 3 specified.
   - Tests: index insert/lookup round-trips; contribution tracking per source
     unit.
   - Deps: 3. Spec: `env.md`.

6. **Deterministic debug rendering.** [ ]
   - Add stable, human-readable renderings of `ResolvedAst` and `SymbolEnv`
     for corpus snapshot baselines; byte-identical across runs and platforms.
   - Tests: identical output across repeated renders; fixture covering every
     current node/table kind.
   - Deps: 4, 5. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md)
     "Snapshot Tests".

7. **Module-index input contract and interim orchestration seam.** [ ]
   - Resolve the interim-seam decision: define the package/module index input
     (architecture 03 Step 1) the resolver consumes as a phase service
     ([internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)),
     with a workspace-stub provider for tests until the `mizar-build`
     module-index provider and `mizar-driver` registry are integrated. Record
     the decision here and at the top level.
   - Tests: stub provider feeds a multi-module fixture; module identity is
     alias-independent.
   - Deps: 4. Spec: architecture 03 "Step 1", `mizar-build` todo tasks 5-6
     and `module_index.md`.

### Imports

8. **Spec: `imports.md`.** [ ]
   - Write the import-resolution spec (English and Japanese, no code): the
     two-pass contract (frontend candidate prescan versus semantic
     validation), alias and relative-prefix rules, cycle policy, and
     unresolved-import representation.
   - Deps: 2. Spec: architecture 03 "Step 2",
     [12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md).

9. **Import graph construction and cycle rejection.** [ ]
   - Build the semantic import graph over the module index and reject cycles
     with deterministic diagnostics.
   - Tests: cycle fixtures rejected deterministically; acyclic fixtures
     produce the expected graph.
   - Deps: 7, 8, `mizar-parser` task 6. Spec: `imports.md`.

10. **Import aliases, relative prefixes, and unresolved-import recovery.** [ ]
    - Resolve aliases and `.`/`..` prefixes to canonical module identity;
      represent unresolved imports explicitly and keep resolving the rest of
      the module.
    - Tests: aliases do not change canonical identity; unresolved imports do
      not abort module resolution.
    - Deps: 9. Spec: `imports.md`.

### Names

11. **Declaration shells.** [ ]
    - Build local declaration shells from `SurfaceAst` items (architecture 03
      Step 3): item identity, visibility markers, export projections — no
      typing, no body resolution.
    - Tests: shell per item kind the parser produces; recovered subtrees
      yield shells flagged recovered, not silent drops.
    - Deps: 7, `mizar-parser` tasks 5 and 7. Spec: architecture 03 "Step 3".

12. **Spec: `names.md`.** [ ]
    - Write the name-resolution spec (English and Japanese, no code): scope
      model, namespace-before-symbol ordering, visibility and shadowing
      rules, ambiguity representation, and the dot-chain finalization
      contract (decision recorded by task 16).
    - Deps: 2. Spec: architecture 03 "Step 4",
      [11.symbol_management.md](../../../spec/en/11.symbol_management.md).

13. **Namespace resolution.** [ ]
    - Resolve namespace segments before symbols (architecture 03 "Namespaces
      Resolve Before Symbols") over the import graph and declaration shells.
    - Tests: nested namespace fixtures; missing-namespace diagnostics carry
      the failing segment range.
    - Deps: 10, 11, 12. Spec: `names.md`.

14. **Qualified names, visibility, and shadowing.** [ ]
    - Resolve qualified and unqualified symbol references with visibility and
      shadowing per the spec scope rules; record results in `NameRefTable` as
      `SymbolId`s.
    - Tests: qualification, shadowing, private-symbol visibility fixtures.
    - Deps: 13. Spec: `names.md`,
      [12.modules_and_namespaces.md](../../../spec/en/12.modules_and_namespaces.md).

15. **Unresolved and ambiguous reference diagnostics.** [ ]
    - Represent unresolved/ambiguous references as explicit nodes with
      deterministic candidate lists; no diagnostic cascades from a single
      unresolved root.
    - Tests: ambiguity fixtures with stable candidate order; one unresolved
      import produces one primary diagnostic.
    - Deps: 14. Spec: `names.md`,
      [22.error_handling_and_diagnostics.md](../../../spec/en/22.error_handling_and_diagnostics.md).

16. **Dot-chain finalization.** [ ]
    - Finish the unresolved dot chains the parser left syntactic: selector
      access versus namespace separation by variable scope. Record the
      decision in `names.md` and close the top-level open decision.
    - Tests: selector/namespace splits from spec §A.2.5 examples; chains that
      fit neither role are diagnosed.
    - Deps: 14, `mizar-parser` task 10. Spec:
      [§A.2.5](../../../spec/en/appendix_a.grammar_summary.md).

### Labels

17. **Spec: `labels.md`.** [ ]
    - Write the label-resolution spec (English and Japanese, no code): the
      separate label scope family, proof-block nesting, forward-reference
      policy, and normalized label-origin paths used by downstream
      `ObligationAnchor` construction.
    - Deps: 2. Spec: architecture 03 "Label Resolution Is Scoped Separately".

18. **Label resolution.** [ ]
    - Resolve statement/theorem labels per task 17, including proof-block
      nesting.
    - Tests: label shadowing across nested proofs; references to later labels
      rejected; `LabelRefTable` determinism.
    - Deps: 11, 17, `mizar-parser` task 22. Spec: `labels.md`,
      [16.theorems_and_proofs.md](../../../spec/en/16.theorems_and_proofs.md).

### Signature collection

19. **Spec: `symbols.md`.** [ ]
    - Write the signature-collection spec (English and Japanese, no code):
      declaration-pass contract (no type checking), per-kind signature
      shapes, duplicate/illegal-declaration policy, and normalized semantic
      origins that remain stable under formatting and unrelated local edits.
    - Deps: 3. Spec: architecture 03 "Step 5",
      [11.symbol_management.md](../../../spec/en/11.symbol_management.md).

20. **Collection skeleton and duplicate detection.** [ ]
    - Populate `SymbolEnv` from declaration shells: registration of names per
      kind, duplicate and conflict diagnostics, overload candidate grouping —
      signatures still opaque.
    - Tests: duplicate detection per kind; candidate grouping; deterministic
      diagnostic order.
    - Deps: 5, 11, 19. Spec: `symbols.md`.

21. **Per-kind signature extraction.** [ ] — paced by `mizar-parser` tasks 23-31.
    - Extract concrete signatures (structs, modes, attributes, predicates,
      functors, theorems, registrations, templates) incrementally: each
      increment lands after the parser grammar task that produces the
      declaration kind, in its own change. Checked off when the last paired
      increment lands.
    - Tests per increment: signature shape fixtures and `SymbolEnv` lookups
      for that kind.
    - Deps: 20; pairs with `mizar-parser` tasks 23-31. Spec: `symbols.md`.

### Hardening and cross-cutting follow-ups

22. **Recovered-syntax policy.** [ ]
    - Define and implement how each resolver stage treats recovered
      `SurfaceAst` subtrees (skip, shell-only, or diagnose), keeping the
      `recovered` flag contract from `mizar-syntax`.
    - Tests: recovered input never panics resolution; diagnostics do not
      cascade from recovered regions.
    - Deps: 13. Spec: [mizar-syntax recovery.md](../../mizar-syntax/en/recovery.md).

23. **Corpus runner at stage `declaration_symbol`.** [ ]
    - Wire `tests/miz/{pass,fail}/` cases at stage `declaration_symbol`
      through the harness, with `spec_trace.toml` coverage entries; seed pass
      and fail cases for tasks 9-20; grow toward the 40/60 pass/fail mix.
    - Deps: 20. Spec: [staged_model.md](../../mizar-test/en/staged_model.md),
      [traceability.md](../../mizar-test/en/traceability.md).

24. **ModuleSummary reuse.** [ ]
    - Consume dependency modules as `ModuleSummary` artifacts (schema-version
      checked) instead of re-reading sources; fall back to source resolution
      when summaries are absent or incompatible.
    - Tests: summary-backed and source-backed resolution agree on a shared
      fixture; incompatible schema falls back with a diagnostic.
    - Deps: 20, `mizar-artifact` task 5. Spec: architecture 03 "Module
      Summary", [18.dependency_fingerprint.md](../../architecture/en/18.dependency_fingerprint.md).

25. **Determinism suite.** [ ]
    - Property coverage that identical inputs produce identical ids, tables,
      diagnostic order, and debug renderings, mirroring the frontend suite.
    - Deps: 21. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).

26. **Public-enum forward-compatibility policy.** [ ]
    - Apply the `mizar-frontend` task-25 decision procedure to each public
      enum; record each decision next to the enum in the owning module spec.
    - Deps: 21. Spec: all module specs.

27. **Source/spec correspondence audit.** [ ]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 26. Spec: all module specs and this TODO.

28. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-resolve/en/` with its Japanese companion and
      synchronize API lists, statuses, terminology, links, and behavior
      promises.
    - Deps: 27. Spec: repository documentation policy.

29. **Module-boundary refactor gate.** [ ]
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
