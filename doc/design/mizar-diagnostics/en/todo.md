# mizar-diagnostics TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs do not exist yet; each is written by its own spec task (English
and Japanese in the same change) before the implementation tasks that cite it.
Module names follow the minimum split of
[internal 07](../../internal/en/07.crate_module_layout.md) (`failure_record`,
`aggregator`) plus the registry/render/fix/explain modules of architecture 12
and internal 03; the crate refines architecture 12 and 19 and internal 03.

| Module | Spec | Source | Status |
|---|---|---|---|
| registry | `registry.md` (task 2) | `src/registry.rs` | [x] |
| failure_record | `failure_record.md` (task 4) | `src/failure_record.rs` | [x] |
| sink | `sink.md` (task 6) | `src/sink.rs` | [x] |
| aggregator | `aggregator.md` (task 8) | `src/aggregator.rs` | [x] |
| render | `render.md` (task 10) | `src/render.rs` | [x] |
| fix | `fix.md` (task 12) | `src/fix.rs` | [ ] |
| explain | `explain.md` (task 14) | `src/explain.rs` | [ ] |

`mizar-diagnostics` owns the canonical diagnostic record shared by every
phase: the stable diagnostic-code registry, structured failure records
(architecture 19), the producer sink API, build-level aggregation with
deterministic ordering, CLI rendering, structured fix suggestions, and lazy
explanation handles. Tools key on `DiagnosticCode`, never on message text;
messages may improve across versions, codes may not be reused.

Dependency order: `registry` → `failure_record` → `sink` → `aggregator` →
`render` / `fix` / `explain`.

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The crate depends on `mizar-session` only (source ranges and snapshot ids).
The first resolver adoption point was deferred by `mizar-resolve` task 13; the
next trigger is the first user-facing resolver diagnostic integration. The LSP
bridge in `mizar-lsp` is a target consumer of `mizar-diagnostics` records and
indexes, but real LSP adoption remains an `external_dependency_gap` until the
consumer seam is ready.
Architecture:
[12.diagnostics_and_lsp.md](../../architecture/en/12.diagnostics_and_lsp.md),
[19.failure_semantics.md](../../architecture/en/19.failure_semantics.md);
internal: [03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md);
spec: [22.error_handling_and_diagnostics.md](../../../spec/en/22.error_handling_and_diagnostics.md).

## Resolved And Open Decisions

- **Adoption timing: deferred by `mizar-resolve` task 13.** This crate remains
  in the target layout, but R-013 kept resolver failures as crate-local/internal
  records and R-015 kept name diagnostics crate-local/internal because resolver
  diagnostic code ownership is still a `spec_gap`. Revisit before the first
  later user-facing resolver diagnostic integration.
- **Migration of existing per-crate diagnostics: open, resolved by
  task 16.** `mizar-lexer`/`mizar-frontend`/`mizar-parser` diagnostics
  predate this crate. Decide whether they migrate to the shared record (and
  in what order) or keep local types behind conversion adapters; record the
  decision and its trigger here and at the top level.
- **Code-space allocation: resolved for the initial spec-22 registry by
  task 2.** `registry.md` fixes the current numeric ranges, canonical
  `PhaseFamily` vocabulary, descriptor defaults, and retirement finality.
  Architecture surfaces without normative ranges remain `external_dependency_gap`
  or `spec_gap` and must not receive placeholder allocations.

## Ordered Task List

Keep `cargo test -p mizar-diagnostics` green after each task (see
[Recommended Verification](#recommended-verification)).

### Records and registry

1. **Crate scaffold and lint-policy guard.** [x]
   - Add the `mizar-diagnostics` workspace member depending on
     `mizar-session`; add `tests/lint_policy.rs` mirroring the
     `mizar-frontend` guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: none. Spec: architecture 12.
   - Completed by task 1: the workspace member, crate manifest, empty boundary
     root, and lint-policy guard are in place without registry, record, sink,
     adapter, driver, LSP, or artifact behavior. The guard covers workspace
     membership, package metadata, `mizar-session`-only dependencies, workspace
     lint opt-in, the shared rustc/clippy lint baseline, documented allow
     exceptions, the empty initial public API, and no premature workspace reverse
     dependencies on `mizar-diagnostics`. Verification passed
     `cargo test -p mizar-diagnostics`,
     `cargo clippy -p mizar-diagnostics --all-targets -- -D warnings`, and
     `cargo fmt --check`.

2. **Spec: `registry.md`.** [x]
   - Write the registry spec (English and Japanese, no code): permanent
     `DiagnosticCode` allocation, code-space ranges per phase family,
     retirement rules, compatibility validation, and lookup metadata
     (semantic name, default severity, documentation URL).
   - Deps: 1. Spec: [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md)
     "Diagnostic Registry",
     [22.error_handling_and_diagnostics.md](../../../spec/en/22.error_handling_and_diagnostics.md).
   - Completed by task 2: `registry.md` now defines stable code identity,
     initial spec-22 phase-family ranges, canonical `PhaseFamily` names,
     descriptor defaults, allocation and final retirement rules, compatibility
     validation, initial allocations from spec 22.7, deferred code-space gaps,
     and lookup behavior. It explicitly keeps message text, localized text,
     rendering, ordering, LSP mapping, proof status, driver orchestration, and
     artifact mutation outside registry authority. Verification passed
     `git diff --check` and `git diff --cached --check`.

3. **Registry implementation.** [x]
   - Implement the code registry with compatibility validation (a code is
     never reused for a different meaning) and a registry-consistency test
     that locks allocated codes.
   - Tests: allocation/retirement fixtures; reuse attempts fail; lookup
     metadata round-trips.
   - Deps: 2. Spec: `registry.md`.
   - Completed by task 3: `src/registry.rs` now provides `DiagnosticCode`,
     canonical `PhaseFamily` and severity metadata, built-in spec-22
     descriptors, validated `DiagnosticRegistry` lookup by code/semantic name
     and alias, descriptor consistency validation, and baseline compatibility
     validation. Tests lock the initial allocation list, metadata round-trips,
     malformed/deferred range rejection, retirement finality, code-reuse
     rejection, semantic rename alias requirements, and alias-domain
     determinism. Verification passed `cargo test -p mizar-diagnostics`,
     `cargo clippy -p mizar-diagnostics --all-targets -- -D warnings`, and
     `cargo fmt --check`.

4. **Spec: `failure_record.md`.** [x]
   - Write the record spec (English and Japanese, no code):
     `DiagnosticDraft` and `DiagnosticRecord` shapes, stable failure
     categories per architecture 19, primary/secondary spans, structured
     details, and machine-readable payload rules.
   - Deps: 2. Spec: [19.failure_semantics.md](../../architecture/en/19.failure_semantics.md),
     [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md).
   - Completed by task 4: `failure_record.md` now defines the two-stage
     draft/record lifecycle, shared stable-code fields, `SourceRange`-backed
     primary and secondary spans, snapshot freshness states, stable failure
     categories, structured-detail payload rules, note payloads, attachment
     slots for later fix/explain tasks, deterministic debug rendering
     requirements, and boundary rules that keep proof, driver, LSP, cache, and
     artifact authority outside the record model. Verification passed
     `git diff --check` and `git diff --cached --check`.

5. **Record and draft implementation.** [x]
   - Implement drafts and records with span and detail tables and a
     deterministic debug rendering.
   - Tests: record round-trips; spans always reference a `SourceId`;
     rendering stability.
   - Deps: 3, 4. Spec: `failure_record.md`.
   - Completed by task 5: `src/failure_record.rs` now provides validated
     `DiagnosticDraft` and immutable `DiagnosticRecord` types, source
     snapshot/freshness state, snapshot-scoped handles, stable failure
     categories, span validation with zero-width intent, structured detail maps
     with deterministic key grammar and value ordering, note payloads, opaque
     fix/explanation attachment refs, descriptor projection from the registry,
     and deterministic debug snapshots. Tests cover structural draft-to-record
     round-trips, `SourceId`-backed span invariants, detail-key validation and
     sorted details, byte-stable debug output, stale/current freshness
     validation, retired-code rejection for current records, and related-handle
     snapshot boundaries. Verification passed
     `cargo test -p mizar-diagnostics`,
     `cargo clippy -p mizar-diagnostics --all-targets -- -D warnings`, and
     `cargo fmt --check`.

### Production and aggregation

6. **Spec: `sink.md`.** [x]
   - Write the producer-API spec (English and Japanese, no code):
     `DiagnosticSink`, phase-side draft emission rules, and what producers
     may not do (no CLI formatting, no LSP shapes).
   - Deps: 4. Spec: [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md)
     "Diagnostic Producer API".
   - Completed by task 6: `sink.md` now specifies producer scopes,
     `DiagnosticSink`/`DiagnosticBatch` behavior, sink-level phase and snapshot
     validation, immutable draft preservation, local production order,
     deterministic debug data, producer emission rules, sink errors, and
     boundaries that keep formatting, LSP protocol shapes, proof/phase status,
     driver orchestration, artifact mutation, and consumer migration outside
     the sink. Verification passed `git diff --check` and
     `git diff --cached --check`.

7. **Sink implementation.** [x]
   - Implement the sink with per-phase draft collection ready for
     aggregation.
   - Tests: sink fixtures across simulated phases; drafts preserved
     unmodified.
   - Deps: 5, 6. Spec: `sink.md`.
   - Completed by task 7: `src/sink.rs` now provides
     `DiagnosticProducerScope`, `DiagnosticSink`, immutable
     `DiagnosticBatch`, and `DiagnosticSinkError`. The sink accepts validated
     drafts whose phase and source snapshot match the scope, rejects sealed or
     mismatched emits without mutating previously collected drafts, preserves
     drafts in local order, and exposes byte-stable batch debug snapshots. Tests
     cover local-order preservation, non-mutating failed emits, sealed behavior,
     consumed-batch preservation, empty and non-empty debug snapshots, and the
     crate boundary guard. Verification passed
     `cargo test -p mizar-diagnostics`,
     `cargo clippy -p mizar-diagnostics --all-targets -- -D warnings`, and
     `cargo fmt --check`.

8. **Spec: `aggregator.md`.** [x]
   - Write the aggregator spec (English and Japanese, no code):
     normalization, identity assignment, deduplication, canonical sort
     order, `BuildDiagnosticIndex`, and the obsolete-snapshot rule
     (diagnostics from stale snapshots are never published as current).
   - Deps: 4, 6, 7. Spec: [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md)
     "Diagnostic Aggregator", architecture 19.
   - Completed by task 8: `aggregator.md` now specifies current-snapshot
     aggregation from sealed batches, immutable `BuildDiagnosticIndex`
     semantics, deterministic source keys, strict obsolete-snapshot filtering,
     stable structured deduplication identity, production-order-independent
     sorting and handle assignment, debug snapshots, and boundary rules. Phase
     status joins, workspace path normalization, driver/LSP/artifact/resolver
     adoption, and legacy diagnostic migration are recorded as outside task 9
     scope or `external_dependency_gap` rather than placeholder APIs.

9. **Aggregator implementation.** [x]
   - Implement aggregation into immutable `BuildDiagnosticIndex` values
     with deterministic ordering independent of production order.
   - Tests: shuffled input produces identical indexes; dedup fixtures;
     snapshot-scoped id determinism; stale-snapshot rejection; negative dedup
     cases where otherwise equal code/phase/primary-span records keep distinct
     structured details, fix edits, or explanation refs.
   - Deps: 7, 8. Spec: `aggregator.md`.
   - Completed by task 9: `src/aggregator.rs` now provides
     `DiagnosticAggregationInput`, deterministic `DiagnosticSourceKey`,
     obsolete-draft accounting, immutable `BuildDiagnosticIndex`, and
     `DiagnosticAggregationError`. Aggregation consumes sealed batches, filters
     non-publication snapshots out of current records, deduplicates by stable
     structured identity rather than message text, chooses representatives
     deterministically, assigns dense snapshot-local handles after canonical
     sorting, and exposes by-source/by-id lookup plus byte-stable debug
     snapshots. Tests cover shuffled input determinism, message-independent
     deduplication, negative dedup cases for details/fixes/explanations,
     obsolete snapshot withholding, snapshot-scoped id lookup, and debug output.
     Verification passed `cargo test -p mizar-diagnostics`,
     `cargo clippy -p mizar-diagnostics --all-targets -- -D warnings`, and
     `cargo fmt --check`.

### Presentation

10. **Spec: `render.md`.** [x]
    - Write the CLI rendering spec (English and Japanese, no code): message
      layout, span excerpts, severity styling, and the rule that rendering
      is keyed by code metadata.
    - Deps: 8. Spec: [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md),
      architecture 12.
    - Completed by task 10: `render.md` now specifies CLI rendering as a
      deterministic projection from `DiagnosticRecord` plus caller-supplied
      source context. It defines header layout, span/source-block layout,
      Unicode-scalar columns, missing-source fallbacks, notes/help projection,
      bounded fix/explanation references before their implementation tasks,
      plain/styled output options, and boundary rules that keep code identity,
      aggregation, source loading, LSP conversion, proof/phase status, driver
      orchestration, and artifact mutation outside rendering authority.

11. **CLI rendering.** [x]
    - Implement deterministic CLI rendering from records and line maps.
    - Tests: golden-file render fixtures; byte-identical output; coverage for
      workspace-relative paths, primary and secondary spans, multiline spans,
      Unicode-scalar column counts, notes, and fix/help projections.
    - Deps: 9, 10. Spec: `render.md`.
    - Completed by task 11: `src/render.rs` now provides
      `DiagnosticSourceContext`, `RenderOptions`, `RenderStyle`,
      `DiagnosticRenderInput`, and `render_diagnostics`. Rendering preserves
      input order, emits code/severity/semantic headers, reads caller-supplied
      paths/source keys/line-column data, renders primary and secondary source
      blocks plus note spans, projects notes/fix refs/explanation refs as
      bounded text, supports byte-stable plain output and ANSI header styling,
      and falls back deterministically when source context is missing. Tests
      cover byte-stable plain rendering, secondary and note spans, fix/help and
      explanation projections, missing-source fallback, multi-diagnostic
      separators, input ordering, and ANSI header styling. Verification passed
      `cargo test -p mizar-diagnostics`,
      `cargo clippy -p mizar-diagnostics --all-targets -- -D warnings`, and
      `cargo fmt --check`.

12. **Spec: `fix.md`.** [ ]
    - Write the fix-suggestion spec (English and Japanese, no code):
      structured edit suggestions, applicability levels, and safety rules
      (suggestions never auto-apply).
    - Deps: 4. Spec: architecture 12 "Fix Suggestion".

13. **Fix suggestions.** [ ]
    - Implement structured fix payloads attached to records.
    - Tests: fix round-trips; edits reference valid ranges.
    - Deps: 5, 12. Spec: `fix.md`.

14. **Spec: `explain.md`.** [ ]
    - Write the explanation spec (English and Japanese, no code): lazy
      explanation handles, bounded previews, and the rule that large traces
      stay in artifacts or dedicated files.
    - Deps: 4. Spec: [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md)
      "Explanation Store".

15. **Explanation store.** [ ]
    - Implement the explanation store with lazy resolution and bounded
      previews.
    - Tests: handle resolution fixtures; preview bounds enforced; missing
      backing data degrades cleanly.
    - Deps: 13, 14. Spec: `explain.md`.

### Adoption and follow-ups

16. **Consumer adoption and migration decision.** [ ]
    - Decide whether a real consumer adoption seam exists for the first
      consumer (`mizar-resolve`) and for pre-existing lexer/frontend/parser
      diagnostics. If the resolver/LSP/driver or frontend-family adoption
      seams are not ready, record `external_dependency_gap` or `deferred`
      dispositions here and at the top level.
    - If a real seam exists, wire only that consumer through the sink and
      aggregator and add the matching end-to-end tests. If no real seam exists,
      this task is documentation-only.
    - Do not add placeholder adapters, stub APIs, fake resolver adoption,
      provisional conversion layers, or `mizar-driver`/`mizar-lsp`
      dependencies from this crate.
    - Tests: documentation verification when adoption is deferred; otherwise
      end-to-end flow for every real adopted consumer, any real conversion
      adapter chosen by the owning consumer round-trips, and consumer corpus or
      `.miz` coverage is added when user-facing language diagnostics migrate.
    - Deps: 9, `mizar-resolve` task 15. Spec: `aggregator.md`.

17. **Determinism suite.** [ ]
    - Property coverage that identical inputs produce identical records,
      indexes, render output, and explanation previews.
    - Deps: 11, 15. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).

18. **Public-enum forward-compatibility policy.** [ ]
    - Apply the `mizar-frontend` task-25 procedure to each public enum;
      severity and category enums additionally follow the architecture 19
      compatibility policy.
    - Deps: 16. Spec: all module specs.

19. **Source/spec correspondence audit.** [ ]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 18. Spec: all module specs and this TODO.

20. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-diagnostics/en/` with its Japanese companion and
      synchronize content.
    - Deps: 19. Spec: repository documentation policy.

21. **Module-boundary refactor gate.** [ ]
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
    - Deps: 20. Spec: this TODO,
      [internal 07](../../internal/en/07.crate_module_layout.md), all module
      specs.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-diagnostics
cargo clippy -p mizar-diagnostics --all-targets -- -D warnings
```

For adoption/migration tasks, also run the consumers:

```text
cargo test -p mizar-resolve
cargo test -p mizar-frontend
cargo test -p mizar-parser
cargo test -p mizar-lexer
cargo test -p mizar-lsp
cargo test -p mizar-build
```

Run only the consumer commands whose implemented seams are touched by the
task. If a named consumer crate or `mizar-driver` integration seam is absent,
record it as an `external_dependency_gap` rather than adding a placeholder.

Check the task off here once tests pass.

## Notes

- Tools key on `DiagnosticCode`; messages may improve across versions,
  codes are permanent and never reused for a different meaning.
- Aggregation output is immutable and deterministically ordered; production
  order and parallelism never show through.
- Open-buffer (LSP overlay) diagnostics reuse the record shape but are
  never published to CLI output or `VerifiedArtifact`; the bridge logic
  lives in `mizar-lsp`.
- Large traces never live inline in diagnostics — compact references and
  bounded previews only.
