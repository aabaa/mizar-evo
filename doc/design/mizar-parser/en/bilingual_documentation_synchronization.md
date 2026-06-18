# Bilingual Documentation Synchronization Audit

> Canonical language: English. Japanese companion:
> [../ja/bilingual_documentation_synchronization.md](../ja/bilingual_documentation_synchronization.md).

Status: completed for parser task 44 and refreshed after parser task 45.

## Scope

This audit compares every English canonical document under
`doc/design/mizar-parser/en/` with its Japanese companion under
`doc/design/mizar-parser/ja/`.

It focuses on documentation surfaces that can drift after source/spec
correspondence is already established:

- public API lists, parser transfer types, and forward-compatibility policy
  notes;
- module and task statuses, including completed, pending, deferred, and
  follow-up records;
- terminology for grammar surfaces, Pratt parsing, recovery, frontend seams,
  syntax-event output, active parse-only corpus evidence, and reserved-word
  coverage;
- links to canonical English documents and Japanese companion documents;
- behavior commitments for syntax-only parsing, recovery ownership,
  parser/frontend and parser/syntax boundaries, determinism, fuzz robustness,
  module-boundary cleanup, and task-43 source/spec correspondence.

This audit does not replace the task-43
[source/spec correspondence audit](./source_spec_audit.md). Task 43 checked
source, spec, test, and reserved-word coverage traceability; this task checks
that the bilingual documentation set presents the same implementation-facing
commitments to English and Japanese readers.

## Result

- No remaining public API, parser transfer type, enum-policy, or
  behavior-promise drift was found between the English canonical parser docs and
  the Japanese companions.
- Module and task statuses are synchronized: parser tasks 1-45 are complete,
  and task 46 remains explicitly deferred for concrete operator declarations
  and operator reserved-word corpus coverage.
- Terminology is synchronized for `ParseRequest`, `ParserToken`,
  `ParseOutput`, `OperatorFixityEntry`, `StringRequiredContext`,
  `SurfaceAst`, syntax-event output, Pratt metadata, recovery nodes,
  `ReservedWord` token coverage, parser-deferred reserved words, and
  parser-owned module boundaries.
- Link policy is synchronized. English canonical files link to English
  canonical documents; Japanese companion files link back to English canonical
  parser files and otherwise prefer Japanese companion targets when those
  targets exist. New cross-cutting audit docs include explicit companion links
  in both directions.
- Behavior commitments are synchronized for syntax-only parsing, no resolver or
  build-system dependency, source-order preservation, Pratt lookup over active
  metadata, formula precedence, recovery synchronization, unrecoverable stray
  `end`, frontend passthrough, parser/syntax boundary ownership, deterministic
  output, fuzz robustness, and task-43 reserved-word coverage.
- This audit found and closed only documentation `design_drift`: parser status
  and index text still described task 44 as pending, the parser audit lists did
  not include this bilingual audit, and the English task-43 TODO wording had a
  duplicated "recorded as" phrase. No source, test, specification, expectation,
  `spec_gap`, `test_gap`, `test_expectation_drift`, `boundary_violation`, or
  `repo_metadata_conflict` finding was introduced.
- No unsynchronized Japanese companion gap remains.

## Pair Checklist

| English canonical | Japanese companion | Synchronization status |
|---|---|---|
| [README.md](./README.md) | [../ja/README.md](../ja/README.md) | Crate boundary, parser status through task 45, audit list, and task 46 deferred state are synchronized. |
| [grammar.md](./grammar.md) | [../ja/grammar.md](../ja/grammar.md) | Grammar inventory, syntax-only responsibilities, `ParserTokenKind` public enum policy note, current grammar-surface status, and deferred operator-declaration wording are synchronized. |
| [pratt.md](./pratt.md) | [../ja/pratt.md](../ja/pratt.md) | Term Pratt, formula Pratt, active metadata, associativity, cache-key boundary, and public enum compatibility promises are synchronized. |
| [recovery.md](./recovery.md) | [../ja/recovery.md](../ja/recovery.md) | Recovery responsibilities, synchronization policy, diagnostic ownership, task-37 consolidation status, and public enum compatibility promises are synchronized. |
| [source_spec_audit.md](./source_spec_audit.md) | [../ja/source_spec_audit.md](../ja/source_spec_audit.md) | Task-43 public API trace, behavior trace, reserved-word guard, parser-deferred reserved-word list, and task-46 follow-up classification are synchronized. |
| [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md) | [../ja/bilingual_documentation_synchronization.md](../ja/bilingual_documentation_synchronization.md) | This task-44 audit records the bilingual synchronization result in both languages. |
| [todo.md](./todo.md) | [../ja/todo.md](../ja/todo.md) | Task statuses and follow-up records are synchronized through task 45; task 46 remains deferred in both languages. |

## Link Policy

English canonical files link to English canonical spec/design/test documents.
Japanese companion files link back to the English canonical `mizar-parser` file
at the top of each file, and otherwise prefer Japanese companion links when
those companion documents exist. New cross-cutting audit docs include explicit
companion links in both directions. Links that intentionally point at English
canonical documents are kept when the referenced source of truth is English-only
or the text is about the English canonical authority itself.

The directory-level parser README, top-level design README, and top-level
roadmap are English index documents rather than per-file Japanese companions.
This task refreshed their parser status and audit links so their summaries agree
with the paired English/Japanese parser docs.

## Follow-up Records

Task 44 did not create a new implementation, test, or specification follow-up.
It closed only the documentation `design_drift` recorded above.

Task 45 later completed the public enum forward-compatibility policy follow-up
without creating a new implementation, test, or specification follow-up. It
confirmed that the existing parser lint-policy guard classifies every public
parser enum. Task 46 remains deferred for concrete operator declarations and
operator reserved-word corpus coverage.
