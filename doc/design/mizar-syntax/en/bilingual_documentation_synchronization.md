# Bilingual documentation synchronization audit

> Canonical language: English. Japanese companion:
> [../ja/bilingual_documentation_synchronization.md](../ja/bilingual_documentation_synchronization.md).

Status: completed for S-020 after the S-019 source/spec correspondence audit.

## Scope

This audit compares every English canonical document under
`doc/design/mizar-syntax/en/` with its Japanese companion under
`doc/design/mizar-syntax/ja/`.

It focuses on documentation surfaces that can drift after source/spec
correspondence is already established:

- public API names, builder/accessor methods, enum names, and diagnostic
  variants;
- module and task statuses, including completed, deferred, and follow-up
  records;
- terminology for the rowan-backed `SurfaceAst`, typed compatibility views,
  trivia side tables, recovery vocabulary, deterministic snapshots, and
  parser/syntax responsibility boundary;
- links to canonical English documents and Japanese companion documents;
- behavior commitments for syntax-only representation, recovery/trivia
  ownership, raw-kind compatibility, identity/reuse rules, parser task pairing,
  source/spec correspondence, and rustdoc deferral.

This audit does not replace the S-019
[source/spec correspondence audit](./source_spec_correspondence.md). S-019
checked source, spec, and test traceability; this task checks that the bilingual
documentation set presents the same implementation-facing commitments to
English and Japanese readers.

## Result

- No remaining public API, enum, diagnostic, or method-level drift was found
  between the English canonical module specs and the Japanese companions.
- Module and task statuses are synchronized: S-001 through S-020 are complete,
  S-021 remains explicitly deferred, parser tasks 4-35 are complete where paired
  with `mizar-syntax`, and existing follow-ups `MSYN-GAP-001`, `MSYN-GAP-003`,
  and `MSYN-GAP-013` remain classified.
- Terminology is synchronized for `SurfaceAst`, `SurfaceAstBuilder`,
  `SurfaceNodeView`, `SyntaxKind`, `SurfaceNodeKind`, `SurfaceTokenKind`,
  `SurfaceTrivia`, `SurfaceTriviaBuilder`, `SyntaxRecoveryKind`,
  `SyntaxDiagnostic`, rowan-backed green-tree storage, deterministic snapshots,
  typed compatibility views, parser task pairing, and syntax-only semantic
  boundaries.
- Japanese companion links now prefer Japanese companion spec/design/test
  documents when the companion exists. Top-of-file links back to English
  canonical documents and authority references to `doc/spec/en/` remain
  intentionally English.
- Behavior commitments are synchronized for syntax-only data structures, raw
  storage not being a semantic identity contract, source ownership and sorted
  trivia rendering, recovery-node vocabulary and active producer status,
  append-only raw-kind numbering for this phase, non-persistent
  `SurfaceNodeId`, range/snapshot or green-node equality reuse validation,
  parser/syntax task pairing, source/spec correspondence, and S-021 rustdoc
  deferral.
- The only S-020 drift found was documentation `design_drift`: the bilingual
  set still described S-020 as pending and some Japanese companion links pointed
  at English companion targets even though Japanese targets existed. This task
  closes that drift without source changes.
- No source/test mismatch, new `spec_gap`, new `test_gap`,
  `test_expectation_drift`, `boundary_violation`, or `repo_metadata_conflict`
  was found.
- No unsynchronized Japanese companion gap remains.

## Pair Checklist

| English canonical | Japanese companion | Synchronization status |
|---|---|---|
| [README.md](./README.md) | [../ja/README.md](../ja/README.md) | Module index, crate boundary, status label, and cross-cutting audit links are synchronized. |
| [00.crate_plan.md](./00.crate_plan.md) | [../ja/00.crate_plan.md](../ja/00.crate_plan.md) | Crate responsibility, specification/test references, parser task pairing, gap classification, task decomposition, S-020 result, and exit criteria are synchronized. |
| [ast.md](./ast.md) | [../ja/ast.md](../ja/ast.md) | Public API, rowan storage boundary, syntax vocabulary through task 35, compatibility view policy, raw-kind policy, identity/reuse rules, and task status are synchronized. |
| [trivia.md](./trivia.md) | [../ja/trivia.md](../ja/trivia.md) | Public API, trivia side-table ownership, sorting, attachment, snapshot behavior, and parser/frontend responsibility boundary are synchronized. |
| [recovery.md](./recovery.md) | [../ja/recovery.md](../ja/recovery.md) | Public API, recovery kinds, diagnostic codes, active and vocabulary-only producer status, malformed annotation recovery, and source/test evidence are synchronized. |
| [grammar_audit.md](./grammar_audit.md) | [../ja/grammar_audit.md](../ja/grammar_audit.md) | Grammar gate findings, parser task map, gap classifications, and close-out status are synchronized. |
| [parse_only_acceptance_matrix.md](./parse_only_acceptance_matrix.md) | [../ja/parse_only_acceptance_matrix.md](../ja/parse_only_acceptance_matrix.md) | Acceptance categories, active/deferred status, grammar-position references, and parser-facing ownership notes are synchronized. |
| [parse_only_fixture_seed.md](./parse_only_fixture_seed.md) | [../ja/parse_only_fixture_seed.md](../ja/parse_only_fixture_seed.md) | Seed fixture intent, activation rules, deferred rows, and parser ownership references are synchronized. |
| [source_spec_correspondence.md](./source_spec_correspondence.md) | [../ja/source_spec_correspondence.md](../ja/source_spec_correspondence.md) | S-019 source/spec/test correspondence, public API and method traceability, follow-up records, and the handoff to this S-020 audit are synchronized. |
| [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md) | [../ja/bilingual_documentation_synchronization.md](../ja/bilingual_documentation_synchronization.md) | This S-020 audit records the bilingual synchronization result in both languages. |
| [crate_exit_report.md](./crate_exit_report.md) | [../ja/crate_exit_report.md](../ja/crate_exit_report.md) | Final hard-gate status, quality score, deferred items, verification results, and next-task handoff are synchronized. |
| [todo.md](./todo.md) | [../ja/todo.md](../ja/todo.md) | Task statuses and follow-up records are synchronized through S-020; S-021 remains deferred in both languages. |

## Link Policy

English canonical files link to English canonical spec/design/test documents and
to their Japanese companion at the top of each file. Japanese companion files
link back to the English canonical `mizar-syntax` file at the top of each file,
and otherwise prefer Japanese companion links when those companion documents
exist. Links that intentionally point at English canonical documents are kept
when the referenced source of truth is English-only or the text is about the
English canonical authority itself.

## Follow-up Records

S-020 did not create a new implementation, test, or specification follow-up.
The only change was to close the documentation `design_drift` recorded above.

S-021 remains explicitly deferred for rustdoc summaries until a long-lived
consumer outside the frontend pipeline starts coding against `mizar-syntax` or
the workspace adopts a rustdoc policy, whichever comes first.

The final crate exit task later added synchronized
[crate_exit_report.md](./crate_exit_report.md) companions without changing the
S-020 audit result.
