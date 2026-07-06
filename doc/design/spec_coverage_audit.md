# Specification Coverage Audit

> Canonical language: English. This top-level design audit has no Japanese
> companion because the surrounding top-level design index documents are
> English-only.

Status: docs-only audit, 2026-07-02.

This document records whether each canonical specification chapter under
`doc/spec/en/` has implementation-facing coverage in `doc/design/`. It does
not change language behavior, `doc/spec`, `.miz` tests, expectation metadata,
or Rust source. It is a synchronization ledger for design and TODO work.

## Status Legend

- `covered` - design docs describe the implementation boundary at usable
  detail for the current crate milestone.
- `partial` - design docs cover the chapter, but some end-to-end behavior is
  deferred to later owner crates or producer/consumer seams.
- `todo` - the required design exists only as a planned module spec or
  follow-up task.
- `reference` - the document is a reference, example, or glossary input rather
  than a direct implementation surface.

## Coverage Matrix

| Spec chapter | Design coverage | Status | Follow-up |
|---|---|---|---|
| `00.index.md` | Index only. Crate TODOs and this audit provide design-side navigation. | reference | Keep links current when spec chapters are added or renamed. |
| `01.introduction.md` | Pipeline, AI-agent, and architecture overview documents cover the implementation posture. | covered | None. |
| `02.lexical_structure.md` | `mizar-lexer`, `mizar-frontend`, `mizar-parser`, and `mizar-syntax` specs cover tokenization, source mapping, context-sensitive lexing, and grammar handoff. | covered | None for the current milestone. |
| `03.type_system.md` | Architecture 04/06 plus `mizar-checker` and `mizar-core` specs cover normalized soft types, erasure, and checker/core handoff. | partial | Source-derived checker payload extraction remains external to the current checker/core milestones. |
| `04.variables_and_constants.md` | Parser grammar covers `reserve`, `let`, `set`, `take`, `given`, `consider`, `reconsider`, `deffunc`, and `defpred`; core binder normalization covers closures, free variables, alpha-equivalence, and substitution. | partial | No single design page traces the whole chapter. Future source-to-checker extraction audits should add a variables/constants trace once real payload extraction exists. |
| `05.structures.md` | Parser/syntax covers structure declarations and inheritance surfaces. Checker tasks 35-36 record the fields-only constructor/property-value source decision plus the root+path/view inheritance identity, exact coverage, and acyclicity decisions with inactive semantic corpus and traceability. Checker docs still record selector and structure-field payload gaps. | partial | Resolver/checker payload work must provide real structure identity/path-view payloads, selector facts, broader constructor coverage, field visibility, and proof-obligation inputs before downstream semantics claim full coverage. |
| `06.attributes.md` | Parser/syntax covers attribute definitions and tests; checker covers normalized attributes, contradiction checks, and fact queries. | partial | Accepted registration/proof status and artifact-fed activated summaries remain external. |
| `07.modes.md` | Parser/syntax and checker type-normalization docs cover mode syntax and unfolding boundaries. Checker task 35 pins constructor arguments as not being a property-value source with inactive semantic corpus and traceability. | partial | Resolver/checker signature payloads, positive property implementation payloads, and source-to-checker extraction remain required for full source coverage. |
| `08.type_inference.md` | Checker type-checker and overload-resolution docs cover declaration checking, facts, coercion candidates, `qua`, and recovery. | partial | Active checker-stage `.miz` coverage and source extraction are still tracked as external gaps in checker docs. |
| `09.predicates.md` | Parser/syntax covers predicate definitions and applications; checker/core/VC cover semantic handoff at a higher level. | partial | Predicate definition correctness and accepted proof evidence remain downstream/external until source payloads and proof consumers are wired. |
| `10.functors.md` | Parser/syntax covers functor definitions/applications; checker overload docs cover candidates and viability. | partial | Functor definition correctness, reductions, and accepted registration/redefinition payloads remain deferred to checker/VC/proof handoffs. |
| `11.symbol_management.md` | Lexer lexical environment, parser syntax, resolver env/symbol/name docs, and artifact summaries cover current symbol surfaces. | covered | Continue R-024 summary-backed reuse without resolver-local artifact formats. |
| `12.modules_and_namespaces.md` | Architecture 03, build module-index docs, resolver imports/env/name docs, and artifact module-summary docs cover module graph and namespace boundaries. | covered | Resolver R-024 remains the immediate reuse integration task. |
| `13.term_expression.md` | Parser/syntax covers terms; checker/core cover typed terms, inserted views, and lowering. | partial | Some source-derived payloads and semantic selector/constructor facts remain owner-gated. |
| `14.formulas.md` | Parser/syntax covers formulas; checker/core/VC cover typed formulas, erasure, proof goals, and generated obligations. | partial | Complete source-derived formula payloads remain external for proof/VC integration. |
| `15.statements.md` | Parser/syntax covers statement surfaces and recovery; core/proof/VC docs consume proof and algorithm statements through explicit payloads. | partial | Proof-verification source runner and full source-to-core extraction remain deferred. |
| `16.theorems_and_proofs.md` | Core, VC, ATP, kernel, proof, cache, artifact, and diagnostics docs cover the current proof pipeline boundaries. | partial | End-to-end proof/cache/artifact consumer integration is still split across evidence-pipeline follow-ups. |
| `17.clusters_and_registrations.md` | Architecture 04/17, checker registration/cluster trace docs, artifact registration summaries, and cache cluster-db docs cover the current data layers. | partial | Accepted status production, artifact publication, and persistent cluster-db materialization remain deferred/external in owner TODOs. |
| `18.templates.md` | Parser/syntax covers template syntax; checker overload docs cover explicit template expansion over supplied payloads. Core task 26 pins omitted func/pred template argument inference to mode-unfolded declared argument types, with inactive determinism corpus and traceability. | partial | Predicate, functor, scheme, and algorithm template roles remain deferred until parser/resolver/checker payloads expose concrete role mappings; active source-corpus execution for template inference remains deferred until runner/extraction support exists. |
| `19.overload_resolution.md` | Architecture 05 and checker overload docs cover candidate collection, template expansion, viability, specificity, root selection, refinement join, and `qua` insertion. Checker task 36 pins implicit upcast path uniqueness as syntactic over resolved `inherit` declaration paths. Checker task 37 pins specificity as a preorder, limits template tie-breaks to concrete-vector equivalence after expansion, and covers multiple-maximal-root ambiguity plus same-signature definition conflicts. | covered | Artifact projection and active source-corpus coverage remain external; ordinary/template-derived equivalent-root and same-return duplicate seeds stay inactive until runner/diagnostic payload support lands. |
| `20.algorithm_and_verification.md` | Parser/syntax, core control-flow, VC generation/discharge, and documentation/extraction docs cover the current algorithm pipeline. | partial | Branch/match/range/collection-loop payloads, term-only/partial termination, Pick non-emptiness, ghost-erasure trace families, MVM/code-extraction backend specs, and source-derived payloads remain TODO/deferred. |
| `21.source_code_annotation_and_atp.md` | Parser/syntax covers annotations; ATP/kernel/proof docs cover solver hints, backend evidence, portfolio, and policy. LSP/docs cover display and extraction consumers. | partial | `@show_*` and `@eval` need end-to-end diagnostic/display/evaluation projection specs before user-facing behavior is complete. |
| `22.error_handling_and_diagnostics.md` | `mizar-diagnostics` registry/failure/sink/render/explain docs cover shared diagnostics. | partial | Resolver name/import/label diagnostics still need a real user-facing adoption task and numeric-code mapping within the existing resolution family. Info/display diagnostics remain reserved until enumerated. |
| `23.package_management_and_build_system.md` | Build, artifact, cache, driver, diagnostics, LSP, and architecture docs cover manifest/build/artifact/cache/LSP/explanation slices. | partial | `mizar refine`, `mizar minimize`, and production `mizar semver-check` CLI ownership remain future driver/tooling tasks; LSP module specs are still planned. |
| `24.documentation_generation.md` | Architecture 13 and internal 05 define phase-16 boundaries; `mizar-doc` TODO schedules module specs. | todo | `mizar-doc` must write module specs for artifact reading, comments, links, math, render, extraction, backend, publisher, and a source/spec coverage closure audit. |
| `sample_codes.md` | Examples exercise intended language surfaces but are not direct implementation authority. | reference | Keep examples aligned through future source/spec audits. |
| `appendix_a.grammar_summary.md` | Parser/syntax grammar audits and parser TODO tasks cover the current grammar surface. | covered | Future grammar additions should update parser/syntax audits. |
| `appendix_b.operator_precedence.md` | Parser Pratt design and parser TODO cover precedence/associativity. | covered | Deferred operator-declaration follow-up remains parser task 46. |
| `appendix_c.glossary.md` | Terminology reference. | reference | Use during bilingual sync and user-facing docs. |
| `appendix_d.recommended_coding_rules.md` | Style/reference guidance, not an implementation phase. | reference | No crate task unless a formatter/linter is later specified. |
| `appendix_e.annotation_quick_reference.md` | Annotation reference mirrors chapter 21. | partial | Close together with chapter 21 annotation display/evaluation follow-ups. |

## Follow-Up Inventory

| ID | Class | Owner | Action |
|---|---|---|---|
| SCA-001 | `design_drift` | top-level design index | Keep `doc/design/README.md` crate status aligned with `doc/design/todo.md`; planned roots must not list existing workspace crates as merely planned. |
| SCA-002 | `todo` | `mizar-doc` | Complete phase-16 module specs and implementation tasks, then add a source/spec coverage closure audit for specs 20, 21, and 24. |
| SCA-003 | `todo` | `mizar-lsp` | Add an annotation display/evaluation projection audit so `@show_*` and `@eval` user-facing outputs have clear diagnostic, freshness, and artifact boundaries. |
| SCA-004 | `external_dependency_gap` | `mizar-resolve` + `mizar-diagnostics` | Map resolver name/import/label diagnostics into public diagnostic descriptors only when a real resolver producer adoption task starts; do not invent placeholder adapters. |
| SCA-005 | `external_dependency_gap` | `mizar-vc` + upstream producers | Keep missing algorithm payload families visible as deferred/no-candidate records until explicit source-derived payloads exist. |
| SCA-006 | `design_drift` | architecture/internal docs | Use current `mizar-doc` module names for phase-16 documentation and extraction; do not list the historical separate `mizar-extract` root as an active owner. |

## Verification

This audit is documentation-only. The expected verification is:

```text
git diff --check
```

Run broader Rust commands only if a later task edits source, tests, or
expectation metadata.
