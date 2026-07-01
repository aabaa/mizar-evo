# Bilingual Documentation Sync Audit: mizar-diagnostics

> Canonical language: English. Japanese companion:
> [../ja/bilingual_documentation_sync.md](../ja/bilingual_documentation_sync.md).

## Scope

This task-20 audit compares every English canonical document under
`doc/design/mizar-diagnostics/en/` with its Japanese companion under
`doc/design/mizar-diagnostics/ja/` after the source/spec correspondence audit.

This is a documentation synchronization gate only. It does not change source
behavior, public APIs, diagnostic identity, registry allocation, aggregation,
rendering, fix/explanation payloads, or any downstream adoption boundary.

## File-Pair Inventory

| English canonical file | Japanese companion | Result |
|---|---|---|
| `00.crate_plan.md` | `00.crate_plan.md` | Present and synchronized in substance through task 20. |
| `aggregator.md` | `aggregator.md` | Present and synchronized in substance. |
| `bilingual_documentation_sync.md` | `bilingual_documentation_sync.md` | Present and synchronized in substance for this audit. |
| `consumer_adoption_decision.md` | `consumer_adoption_decision.md` | Present and synchronized in substance. |
| `explain.md` | `explain.md` | Present and synchronized in substance. |
| `failure_record.md` | `failure_record.md` | Present and synchronized in substance. |
| `fix.md` | `fix.md` | Present and synchronized in substance. |
| `module_boundary_refactor_gate.md` | `module_boundary_refactor_gate.md` | Present and synchronized in substance after task 21 scoped rerun. |
| `registry.md` | `registry.md` | Present and synchronized in substance. |
| `render.md` | `render.md` | Present and synchronized in substance. |
| `sink.md` | `sink.md` | Present and synchronized in substance. |
| `source_spec_correspondence.md` | `source_spec_correspondence.md` | Present and synchronized in substance. |
| `todo.md` | `todo.md` | Present and synchronized in substance through task 21 scoped rerun. |

No English-only diagnostics design document and no Japanese-only companion was
found in this directory pair.

## Checks Performed

- Compared file-pair names under the English and Japanese directories.
- Compared heading structure for every file pair.
- Checked task completion records, module implementation tables, known-gap
  tables, source/spec audit results, and boundary rules for matching substance.
- Checked that external/deferred consumer, LSP, driver, artifact, and metadata
  gaps are represented consistently.
- Checked that Japanese companions retain the same no-placeholder/no-authority
  boundaries as the English canonical files.

Headings are considered synchronized when they identify the same section even if
the Japanese companion localizes the heading text. Technical identifiers,
diagnostic code names, file paths, gap classes, and API names intentionally stay
unchanged.

## Sync Result

The task-20 audit found no substantive bilingual drift requiring edits to module
specs. Task 21 re-runs this inventory for the module-boundary gate report and
updates this file with the new paired document. No unsynchronized companion is
introduced.

## Remaining Boundaries

The following are synchronized as deferred or external, not as work owned by
`mizar-diagnostics`:

- existing lexer/frontend/parser/resolver diagnostic migration;
- `mizar-lsp` protocol conversion and publication;
- driver session orchestration;
- artifact mutation, manifest publication, and durable projection ownership;
- the missing `mizar-artifact` closeout report noted as a
  `repo_metadata_conflict`.

No `boundary_violation`, `source_undocumented_behavior`, or unsynchronized
Japanese companion was found for the crate-owned diagnostics surface.

## Verification

Docs-only verification for this task:

```text
git diff --check
git diff --cached --check
```
