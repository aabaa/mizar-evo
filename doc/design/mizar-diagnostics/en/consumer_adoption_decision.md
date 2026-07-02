# Consumer Adoption Decision

> Canonical language: English. Japanese companion:
> [../ja/consumer_adoption_decision.md](../ja/consumer_adoption_decision.md).

## Purpose

This note records the task-16 adoption decision for `mizar-diagnostics`. The
crate now owns the diagnostic-code registry, structured failure records,
producer sink, deterministic aggregation, CLI rendering, structured fix
suggestions, and lazy explanation handles. Consumer migration is allowed only
when a real owning-consumer seam exists.

## Decision

Task 16 is documentation-only. No lexer, frontend, parser, resolver, driver,
LSP, build, or artifact consumer is migrated in this task. The current
workspace does not yet provide a real consumer adoption seam that can accept
`mizar-diagnostics` records without inventing placeholder adapters or moving
authority out of the owning crates.

Existing diagnostics remain in their owning crates until the triggers below are
met. Tools and future consumers must key on `DiagnosticCode` and structured
fields when adoption happens; message text remains presentation.

## Dispositions

| Area | Evidence | Disposition | Trigger to revisit |
|---|---|---|---|
| `mizar-resolve` | The resolver closeout records R-G001: the shared registry reserves a broad `Resolution` family, but resolver name/import/label descriptors and migration/adoption behavior are absent, and task R-013/R-015 keep those diagnostics crate-local/internal. | `external_dependency_gap` / `deferred` | `mizar-resolve` R-030 or an equivalent real producer-side adoption task defines concrete descriptors, numeric codes or aliases, migration behavior, and corpus or `.miz` coverage. |
| Lexer/frontend/parser diagnostics | These crates already expose crate-local diagnostics and merge ordering, but no cross-crate migration plan or consumer adoption seam exists for `mizar-diagnostics`. | `external_dependency_gap` / `deferred` | The owning crate opens a migration task that preserves existing corpus expectations, source ranges, and deterministic ordering through a real sink/aggregation path. |
| `mizar-lsp` | LSP diagnostic-publication spec and implementation tasks are still open; range conversion, document versions, overlay diagnostics, and protocol shaping belong to `mizar-lsp`. | `external_dependency_gap` | `mizar-lsp` tasks for snapshot publication and diagnostics conversion land and consume `BuildDiagnosticIndex` as an owned LSP projection. |
| `mizar-driver` | `crates/mizar-driver` now has a scaffold and may depend on diagnostics, but driver request/session/event-stream/publication tasks are still future work. | `external_dependency_gap` | `mizar-driver` lands real session orchestration and the publication boundary that can collect phase batches without diagnostics owning the driver bridge. |
| Artifact projection | `mizar-artifact` owns artifact mutation, manifest publication, and durable projected artifact schemas. | `external_dependency_gap` | Artifact-facing emission is driven by real producer outputs and an artifact-owned projection task. |

## Boundary Rules

- Do not add placeholder adapters, stub APIs, fake resolver adoption, or
  provisional conversion layers.
- Do not add `mizar-driver`, `mizar-lsp`, resolver, lexer, parser, or frontend
  dependencies from `mizar-diagnostics`.
- Do not migrate existing diagnostics merely to make this crate look adopted.
- Do not key tools on diagnostic message text during any future migration.
- Do not move LSP protocol conversion, driver session orchestration, artifact
  mutation, proof acceptance, kernel acceptance, or phase success authority into
  `mizar-diagnostics`.

## Verification

Task 16 verification is documentation-only:

- the workspace reverse-dependency lint remains the guard against accidental
  `mizar-diagnostics` consumer adoption;
- `git diff --check` and `git diff --cached --check` are sufficient for this
  task unless source files change.
