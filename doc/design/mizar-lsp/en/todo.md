# mizar-lsp TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs do not exist yet; each is written by its own spec task (English
and Japanese in the same change) before the implementation tasks that cite
it. The exception is `range`, where code predates the spec and task 2
retrofits one. The crate refines architecture 12 and 19 and internal 03 per
[internal 07](../../internal/en/07.crate_module_layout.md).

| Module | Spec | Source | Status |
|---|---|---|---|
| range | `range.md` (task 2) | `src/range_mapper.rs` | [~] conversions exist, spec pending |
| server | `server.md` (task 4) | `src/server.rs` | [ ] |
| snapshot | `snapshot.md` (task 6) | `src/snapshot.rs` | [ ] |
| diagnostics | `diagnostics.md` (task 8) | `src/diagnostics.rs` | [ ] |
| build_bridge | `build_bridge.md` (task 10) | `src/build_bridge.rs` | [ ] |
| metadata | `metadata.md` (task 12) | `src/metadata.rs` | [ ] |
| navigation | `navigation.md` (task 14) | `src/navigation.rs` | [ ] |
| code_action | `code_action.md` (task 16) | `src/code_action.rs` | [ ] |
| explain | `explain.md` (task 18) | `src/explain.rs` | [ ] |

`mizar-lsp` owns the editor-facing protocol bridge: open-buffer source
snapshots and document versions, `SourceRange` → LSP UTF-16 conversion,
diagnostic publication from the current `LspSnapshot`, hover/navigation/
semantic features served from published artifacts and snapshot metadata,
code actions from structured fix suggestions, explanation queries, and
build scheduling through the compiler driver. It owns no proof acceptance,
no type checking, no ATP reasoning, and no artifact mutation. It is built in
three waves: **wave A** (range spec retrofit) is independent; **wave B**
(server, snapshots, diagnostics, build bridge) lands with
`mizar-diagnostics`, `mizar-ir`, and `mizar-driver`; **wave C** (metadata
features) grows with the semantic layers that produce the metadata.

Each task below is deliberately small — one module spec, or one behavior
slice of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The crate currently depends on `mizar-session` and `mizar-lexer` (range
conversion). Wave B adds `mizar-diagnostics` (records and indexes),
`mizar-ir` (snapshot handles), and `mizar-driver` (build requests and
events); wave C adds `mizar-artifact` (metadata readers). Architecture:
[12.diagnostics_and_lsp.md](../../architecture/en/12.diagnostics_and_lsp.md),
[19.failure_semantics.md](../../architecture/en/19.failure_semantics.md);
internal: [03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md).

## Resolved And Open Decisions

- **LSP transport/protocol library: open, resolved by task 4.** Choose
  between an existing protocol crate and a minimal in-house JSON-RPC layer,
  weighing the workspace's external-dependency policy; record the decision
  and its trust implications in `server.md`.
- **Overlay diagnostics: resolved by internal 03.** Open-buffer diagnostics
  for unsaved text reuse the shared record shape but are never published to
  CLI output or `VerifiedArtifact`; `diagnostics.md` restates this.
- **Metadata source: resolved by internal 03.** Hover/navigation features
  read published artifacts and `LspSnapshot` metadata, never raw compiler
  IR; freshness is tracked per snapshot and stale data is marked, not
  hidden.

## Ordered Task List

Keep `cargo test -p mizar-lsp` green after each task (see
[Recommended Verification](#recommended-verification)).

### Wave A: range foundation

1. **Lint-policy guard.** [ ]
   - Add `tests/lint_policy.rs` mirroring the `mizar-frontend` guard.
   - Tests: lint-policy guard passes.
   - Deps: none. Spec: repository conventions.

2. **Spec: `range.md` (retrofit).** [ ]
   - Write the range-conversion spec for the existing `range_mapper`
     (English and Japanese): UTF-16 code-unit rules, surrogate pairs, line
     endings, line-map interaction, and error cases; record any
     code-versus-spec gaps as follow-ups.
   - Deps: 1. Spec: [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md)
     "Range Conversion".

3. **Range-conversion hardening.** [ ]
   - Close the gaps task 2 found and add property tests (round-trips where
     defined, surrogate-pair boundaries, CRLF/LF).
   - Tests: property suite; conversion totality over fixture corpora.
   - Deps: 2. Spec: `range.md`.

### Wave B: server, snapshots, diagnostics, build bridge

4. **Spec: `server.md`.** [ ]
   - Write the server-lifecycle spec (English and Japanese, no code):
     initialize/shutdown, capability negotiation, document
     synchronization, request routing, and the transport-library decision
     with its rationale.
   - Deps: 2. Spec: [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md)
     "LSP Bridge".

5. **Server lifecycle.** [ ]
   - Implement initialize/shutdown, capability negotiation, and document
     synchronization over the chosen transport.
   - Tests: lifecycle fixtures over an in-memory transport; malformed
     requests answered per protocol, never crashing.
   - Deps: 4. Spec: `server.md`.

6. **Spec: `snapshot.md`.** [ ]
   - Write the snapshot/freshness spec (English and Japanese, no code):
     open-buffer snapshots, document versions, the current/stale
     `LspSnapshot` model over `mizar-ir` handles, and staleness marking
     rules.
   - Deps: 4. Spec: [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md)
     "LSP Snapshot".

7. **Open-buffer snapshots and versions.** [ ]
   - Implement buffer snapshots with version tracking and the
     current/stale snapshot registry.
   - Tests: edit sequences keep versions monotonic; stale snapshots are
     marked, never silently served as current.
   - Deps: 5, 6, `mizar-ir` task 13. Spec: `snapshot.md`.

8. **Spec: `diagnostics.md`.** [ ]
   - Write the diagnostic-publication spec (English and Japanese, no
     code): publication from `BuildDiagnosticIndex`, range conversion,
     overlay diagnostics for unsaved text (restating the
     never-to-CLI/artifact rule), and ordering guarantees.
   - Deps: 6. Spec: [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md)
     "Diagnostic Aggregator"/"Lightweight Open-Buffer Diagnostics".

9. **Diagnostic publication.** [ ]
   - Publish diagnostics from the current snapshot's index with converted
     ranges; obsolete-snapshot diagnostics are never published as current.
   - Tests: publication fixtures; stale-index suppression; deterministic
     order.
   - Deps: 7, 8, `mizar-diagnostics` task 9. Spec: `diagnostics.md`.

10. **Spec: `build_bridge.md`.** [ ]
    - Write the build-bridge spec (English and Japanese, no code):
      scheduling builds through the `CompilerDriver` API, consuming build
      events, debouncing/superseding on edits, and watch-mode interplay.
    - Deps: 6. Spec: [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
      "Watch and LSP Build".

11. **Build bridge.** [ ]
    - Implement build scheduling and event consumption through
      `mizar-driver`; superseded sessions cancel cleanly.
    - Tests: edit-burst fixtures debounce; events map to the right
      snapshot; cancellation on supersede.
    - Deps: 9, 10, `mizar-driver` tasks 8 and 11. Spec: `build_bridge.md`.

### Wave C: metadata features

12. **Spec: `metadata.md`.** [ ]
    - Write the metadata-reader spec (English and Japanese, no code):
      reading `VerifiedArtifact` expression metadata and module summaries
      for IDE features, freshness rules, and the no-raw-IR boundary.
    - Deps: 6. Spec: [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md),
      [internal 06](../../internal/en/06.ir_storage_and_snapshot_handles.md)
      "Artifact Projection Boundary".

13. **Hover and type metadata.** [ ]
    - Serve hover (inferred types, resolved symbols, `@show_*` data) from
      expression metadata.
    - Tests: hover fixtures over fixture artifacts; stale data marked.
    - Deps: 12, `mizar-artifact` task 11. Spec: `metadata.md`.

14. **Spec: `navigation.md`.** [ ]
    - Write the navigation spec (English and Japanese, no code):
      definitions, references, document symbols, and semantic tokens over
      artifact metadata and snapshot data.
    - Deps: 12. Spec: architecture 12,
      [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md).

15. **Navigation features.** [ ] — paced by metadata producers.
    - Implement definitions, references, document symbols, and semantic
      tokens incrementally as the producing layers land (resolver
      metadata, checker expression metadata); one feature increment per
      change. Checked off when the last increment lands.
    - Deps: 13, 14. Spec: `navigation.md`.

16. **Spec: `code_action.md`.** [ ]
    - Write the code-action spec (English and Japanese, no code):
      conversion of structured fix suggestions into LSP code actions,
      applicability filtering, and the no-auto-apply rule.
    - Deps: 8. Spec: architecture 12 "Fix Suggestion",
      [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md)
      "Code Actions".

17. **Code actions.** [ ]
    - Implement code actions from `mizar-diagnostics` fix payloads.
    - Tests: action fixtures per fix kind; edits round-trip through range
      conversion.
    - Deps: 9, 16, `mizar-diagnostics` task 13. Spec: `code_action.md`.

18. **Spec: `explain.md`.** [ ]
    - Write the explanation-query spec (English and Japanese, no code):
      resolving explanation handles to bounded payloads on demand, and
      latency/size limits.
    - Deps: 8. Spec: [internal 03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md)
      "Explanation Queries".

19. **Explanation queries.** [ ]
    - Implement explanation requests over the `mizar-diagnostics`
      explanation store.
    - Tests: query fixtures; bounded payloads; missing backing data
      degrades cleanly.
    - Deps: 17, 18, `mizar-diagnostics` task 15. Spec: `explain.md`.

### Hardening and cross-cutting follow-ups

20. **Determinism and freshness suite.** [ ]
    - Property coverage that identical snapshots produce identical
      published diagnostics and feature responses, and that stale data is
      always marked.
    - Deps: 11, 13. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).

21. **Public-enum forward-compatibility policy.** [ ]
    - Apply the `mizar-frontend` task-25 procedure to each public enum.
    - Deps: 11. Spec: all module specs.

22. **Source/spec correspondence audit.** [ ]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 21. Spec: all module specs and this TODO.

23. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-lsp/en/` with its Japanese companion and
      synchronize content.
    - Deps: 22. Spec: repository documentation policy.

24. **Annotation display and evaluation projection audit.** [ ]
    - Before claiming chapter-21 display/evaluation coverage, audit how
      `@show_type`, `@show_resolution`, `@show_thesis`, and `@eval` flow from
      parser/syntax nodes through diagnostics, metadata artifacts, and LSP
      presentation. Record whether each output is a diagnostic, inline hint,
      explicit information response, or unavailable/deferred result; preserve
      snapshot freshness and never evaluate terms inside the LSP server.
    - Deps: 13, 18, 23. Spec:
      [21.source_code_annotation_and_atp.md](../../../spec/en/21.source_code_annotation_and_atp.md),
      [12.diagnostics_and_lsp.md](../../architecture/en/12.diagnostics_and_lsp.md),
      [spec_coverage_audit.md](../../spec_coverage_audit.md).

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-lsp
cargo clippy -p mizar-lsp --all-targets -- -D warnings
```

For wave B/C tasks, also run the integrated crates:

```text
cargo test -p mizar-diagnostics
cargo test -p mizar-driver
cargo test -p mizar-artifact
```

Check the task off here once tests pass.

## Notes

- The bridge owns protocol conversion and freshness, never semantics: no
  proof acceptance, no type checking, no ATP reasoning, no artifact
  mutation.
- Overlay diagnostics for unsaved text never reach CLI output or
  `VerifiedArtifact`.
- Features read published artifacts and snapshot metadata, never raw
  compiler IR; stale data is marked, not hidden.
- LSP workflows may prioritize latency, but batch verification remains the
  semantic baseline.
