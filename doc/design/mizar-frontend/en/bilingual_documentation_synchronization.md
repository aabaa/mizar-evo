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
- Module and task statuses are synchronized: tasks 1-21 are complete, and
  follow-up tasks 22-24 remain open.
- Terminology is synchronized for `SourceUnit`, `PreprocessedSource`,
  `ImportStub`, `ActiveLexicalEnvironment`, `TokenStream`, parser seam,
  `FrontendOutput`, frontend content cache keys, recoverable diagnostics, hard
  `FrontendError`s, and parser-assisted lexing plans.
- Japanese companion links now prefer Japanese companion architecture/spec/module
  links when the companion exists, while preserving links to English canonical
  files where the referenced canonical source is intentionally English.
- Behavior commitments are synchronized for loading-map preservation, coarse
  raw-scan recovery, provider provenance validation, bounded conflict retry,
  structured lexing payload preservation, `ast = None` parser recovery, stable
  diagnostic merge order, source-load locations without fabricated ranges, and
  the resident-set `ModuleLexicalSummary` boundary, task-19 cache-key
  storage/computation boundaries, task-20 parser-assisted lexing-plan
  boundaries, and task-21 durable lint-policy guards.
- No unsynchronized Japanese companion gap remains.

## Pair Checklist

| English canonical | Japanese companion | Synchronization status |
|---|---|---|
| [README.md](./README.md) | [../ja/README.md](../ja/README.md) | Module index, crate boundary, status labels, and context links are synchronized. |
| [span_bridge.md](./span_bridge.md) | [../ja/span_bridge.md](../ja/span_bridge.md) | Public API, identity-loading behavior, composite/degraded mappings, registry invariants, and error surfaces are synchronized. |
| [source.md](./source.md) | [../ja/source.md](../ja/source.md) | Public API, diagnostic display path policy, loading-map preservation, error propagation, and constraints are synchronized. |
| [preprocess.md](./preprocess.md) | [../ja/preprocess.md](../ja/preprocess.md) | Public API, comment/doc-comment handling, import stubs, coarse raw import recovery, diagnostics, and parser-assisted string-argument handling are synchronized. |
| [lexical_env.md](./lexical_env.md) | [../ja/lexical_env.md](../ja/lexical_env.md) | Provider seam API, provenance validation, import canonicalization, conflict retry, malformed-summary boundary, cache fingerprint, and resident-set links are synchronized. |
| [lexing.md](./lexing.md) | [../ja/lexing.md](../ja/lexing.md) | Token stream API, parser lexing-plan API, scope view API, payload variants, two-pass contextual skeleton behavior, and raw-scan recovery are synchronized. |
| [parsing.md](./parsing.md) | [../ja/parsing.md](../ja/parsing.md) | Parser input API, position-sensitive string context API, seam API, parser cache-key version API, stub/real parser behavior, Pratt/fixity coverage, and parser recovery are synchronized. |
| [cache_key.md](./cache_key.md) | [../ja/cache_key.md](../ja/cache_key.md) | Frontend content cache-key APIs, parser lexing-plan content keys, storage boundary, invalidation rules, and tests are synchronized. |
| [orchestration.md](./orchestration.md) | [../ja/orchestration.md](../ja/orchestration.md) | Frontend API, `FrontendOutput.cache_keys`, diagnostic classes, source-load locations, merge order, hard-error boundaries, syntax pass-through, and output constraints are synchronized. |
| [source_spec_correspondence.md](./source_spec_correspondence.md) | [../ja/source_spec_correspondence.md](../ja/source_spec_correspondence.md) | Task-16 audit text now records task-19 cache-key wiring, task-20 parser-assisted lexing, task-21 durable lint enforcement, and remaining tasks 22-24. |
| [todo.md](./todo.md) | [../ja/todo.md](../ja/todo.md) | Task statuses and follow-up records are synchronized; task 21 is checked off with this result. |

## Link Policy

English canonical files link to English canonical architecture/spec/module
documents and to their Japanese companion at the top of each file. Japanese
companion files link back to the English canonical mizar-frontend file at the top
of each file, and otherwise prefer Japanese companion links when those companion
documents exist. Links that intentionally point at English canonical documents
are kept when the referenced source of truth is English-only or the text is about
the English canonical decision itself.

## Follow-up Records

No new follow-up task was added by this audit. Tasks 18, 19, 20, and 21 have
since been completed; existing open follow-ups are:

- Task 22: precise raw-scan recovery contract.
- Task 23: resident-set contract guard for the lexical environment.
- Task 24: reserved frontend diagnostic surface coverage.
