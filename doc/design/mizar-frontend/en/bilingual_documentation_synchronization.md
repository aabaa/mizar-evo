# Bilingual documentation synchronization audit

> Canonical language: English. Japanese companion: [../ja/bilingual_documentation_synchronization.md](../ja/bilingual_documentation_synchronization.md).

Status: completed for task 17.

## Scope

This audit compares every English canonical document under
`doc/design/mizar-frontend/en/` with its Japanese companion under
`doc/design/mizar-frontend/ja/`.

It focuses on the documentation surfaces that can drift independently after the
implementation is already source/spec traceable:

- public API lists and re-exported type names;
- error and diagnostic variants;
- task statuses and follow-up records;
- terminology for frontend phases, seams, and recovery boundaries;
- links to canonical English documents and Japanese companion documents;
- behavior commitments, especially recovery-vs-hard-failure boundaries and
  parser-assisted lexing-plan boundaries.

This audit does not replace the task-16
[source/spec correspondence audit](./source_spec_correspondence.md). Task 16
checked source, spec, and test traceability; this task checks that the bilingual
documentation set now presents the same commitments to English and Japanese
readers.

## Result

- No remaining public API or error/diagnostic variant drift was found between the
  English canonical module specs and the Japanese companions.
- Module and task statuses are synchronized: tasks 1-29 are complete; task 29
  completed the frontend-owned real-parser fuzz follow-up, while the
  parser-owned counterpart remains tracked by `mizar-parser` task 39.
- Terminology is synchronized for `SourceUnit`, `PreprocessedSource`,
  `ImportStub`, `ActiveLexicalEnvironment`, `TokenStream`, parser seam,
  `FrontendOutput`, frontend content cache keys, recoverable diagnostics, hard
  `FrontendError`s, and parser-assisted lexing plans.
- Japanese companion links now prefer Japanese companion architecture/spec/module
  links when the companion exists, while preserving links to English canonical
  files where the referenced canonical source is intentionally English.
- Behavior commitments are synchronized for loading-map preservation, precise
  recoverable raw-scan recovery, provider provenance validation, bounded conflict retry,
  structured lexing payload preservation, `ast = None` parser recovery, stable
  diagnostic merge order, source-load locations without fabricated ranges, and
  the resident-set `ModuleLexicalSummary` boundary and task-23 guard, task-19
  cache-key storage/computation boundaries, task-20 parser-assisted lexing-plan
  boundaries, task-21 durable lint-policy guards, task-22 raw-scan recovery
  boundaries, task-24 reserved diagnostic surface policy, task-26 public API
  rustdoc summary policy, task-28 parser-growth follow-through, and task-29
  real-parser frontend fuzz coverage, plus the retrospective autonomous
  crate-development plan and exit report.
- No unsynchronized Japanese companion gap remains.

## Pair Checklist

| English canonical | Japanese companion | Synchronization status |
|---|---|---|
| [README.md](./README.md) | [../ja/README.md](../ja/README.md) | Module index, crate boundary, status labels, and context links are synchronized. |
| [00.crate_plan.md](./00.crate_plan.md) | [../ja/00.crate_plan.md](../ja/00.crate_plan.md) | Retrospective crate responsibility, specification/test references, gap classification, task decomposition, and exit criteria are synchronized. |
| [span_bridge.md](./span_bridge.md) | [../ja/span_bridge.md](../ja/span_bridge.md) | Public API, identity-loading behavior, composite/degraded mappings, registry invariants, and error surfaces are synchronized. |
| [source.md](./source.md) | [../ja/source.md](../ja/source.md) | Public API, diagnostic display path policy, loading-map preservation, error propagation, and constraints are synchronized. |
| [preprocess.md](./preprocess.md) | [../ja/preprocess.md](../ja/preprocess.md) | Public API, comment/doc-comment handling, import stubs, precise recoverable raw import recovery, diagnostics, and parser-assisted string-argument handling are synchronized. |
| [lexical_env.md](./lexical_env.md) | [../ja/lexical_env.md](../ja/lexical_env.md) | Provider seam API, provenance validation, import canonicalization, conflict retry, malformed-summary boundary, cache fingerprint, and resident-set links are synchronized. |
| [lexing.md](./lexing.md) | [../ja/lexing.md](../ja/lexing.md) | Token stream API, parser lexing-plan API, scope view API, payload variants, two-pass contextual skeleton behavior, and raw-scan recovery are synchronized. |
| [parsing.md](./parsing.md) | [../ja/parsing.md](../ja/parsing.md) | Parser input API, position-sensitive string context API, seam API, parser cache-key version API, stub/real parser behavior, Pratt/fixity coverage, and parser recovery are synchronized. |
| [cache_key.md](./cache_key.md) | [../ja/cache_key.md](../ja/cache_key.md) | Frontend content cache-key APIs, parser lexing-plan content keys, storage boundary, invalidation rules, and tests are synchronized. |
| [orchestration.md](./orchestration.md) | [../ja/orchestration.md](../ja/orchestration.md) | Frontend API, `FrontendOutput.cache_keys`, diagnostic classes, source-load locations, merge order, hard-error boundaries, syntax pass-through, and output constraints are synchronized. |
| [source_spec_correspondence.md](./source_spec_correspondence.md) | [../ja/source_spec_correspondence.md](../ja/source_spec_correspondence.md) | Task-16 audit text records task-19 cache-key wiring, task-20 parser-assisted lexing, task-21 durable lint enforcement, task-22 raw-scan recovery, task-23 resident-set guard status, task-24 reserved diagnostic surface coverage, task-26 rustdoc summary coverage, task-28 parser-growth follow-through, and task-29 real-parser frontend fuzz coverage. |
| [crate_exit_report.md](./crate_exit_report.md) | [../ja/crate_exit_report.md](../ja/crate_exit_report.md) | Hard-gate status, quality score, deferred items, verification results, and next-task handoff are synchronized. |
| [todo.md](./todo.md) | [../ja/todo.md](../ja/todo.md) | Task statuses and follow-up records are synchronized through task 29. |

## Link Policy

English canonical files link to English canonical architecture/spec/module
documents and to their Japanese companion at the top of each file. Japanese
companion files link back to the English canonical mizar-frontend file at the top
of each file, and otherwise prefer Japanese companion links when those companion
documents exist. Links that intentionally point at English canonical documents
are kept when the referenced source of truth is English-only or the text is about
the English canonical decision itself.

## Follow-up Records

Tasks 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, and 29 have since been
completed. Task 29 completed the frontend-owned real-parser fuzz follow-up; the
parser-owned counterpart remains tracked by `mizar-parser` task 39. Future
producer-backed tests should be added when non-exhaustive lexer/session/parser
contracts expose concrete producers for the currently reserved fallback
variants.
