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
| `02.lexical_structure.md` | `mizar-lexer`, `mizar-frontend`, `mizar-parser`, and `mizar-syntax` specs cover tokenization, source mapping, context-sensitive lexing, and grammar handoff. Checker tasks 75/76/77 add active type-elaboration diagnostic coverage for the source active-range rule that a local mode, structure, or attribute spelling is not available to an earlier reserve type expression before its declaration item is complete. Checker task 81 adds lexer/frontend unit coverage for splitting the local attribute `param_prefix` hyphen only when an active or declaration-site attribute suffix is present, while preserving ordinary hyphenated constructor names as single user symbols; resolver unit coverage records the suffix as the declaration symbol primary spelling. | covered | None for the current milestone; forward-reference acceptance is forbidden by the Chapter 2/11 active-range rules and covered by task 75/76/77 lower-stage rejection. |
| `03.type_system.md` | Architecture 04/06 plus `mizar-checker` and `mizar-core` specs cover normalized soft types, erasure, and checker/core handoff. Checker task 41 defines `attribute_ref(args)` as ordinary use-site attribute application while excluding it from cluster adjectives. Checker task 50 adds active source-derived diagnostic coverage for same-module attributed builtin reserve type expressions reaching the checker evidence-query gap. Checker task 51 adds active source-derived diagnostic coverage for same-module local mode reserve heads reaching the missing mode-expansion payload gap. Checker task 52 adds active source-derived diagnostic coverage for same-module local structure reserve heads reaching the checker evidence-query gap. Checker task 53 adds active source-derived diagnostic coverage for same-module attributed local structure reserve type expressions reaching the checker evidence-query gap. Checker task 54 adds active source-derived diagnostic coverage for same-module attributed local mode reserve type expressions reaching the missing mode-expansion payload gap when no supported real expansion is available or the same mode is mixed with a bare reserve use. Checker task 55 adds active pass coverage for same-module no-argument local mode reserve heads whose real AST-derived mode expansion has a bare builtin RHS. Checker task 56 adds active pass coverage for one-edge same-module local-mode expansion chains whose dependency mode has that accepted bare builtin RHS expansion, plus an active attributed-dependency fail-closed diagnostic. Checker task 57 adds active diagnostic coverage for a real same-module local-mode expansion whose RHS is a local structure head, stopping at the checker evidence-query gap for missing base-shape/constructor-witness evidence instead of reporting a missing mode-expansion payload. Checker task 58 adds active diagnostic coverage for a real same-module local-mode expansion whose RHS is an attributed builtin head, stopping at the checker evidence-query gap for missing attributed-type existential evidence instead of reporting a missing mode-expansion payload. Checker task 59 adds active diagnostic coverage for a same-module attributed local mode reserve head whose real direct bare-builtin mode expansion is available, stopping at the checker evidence-query gap for missing attributed-type existential evidence instead of reporting a missing mode-expansion payload. Checker task 60 adds active diagnostic coverage for a same-module attributed local mode reserve head whose real direct local-structure RHS expansion is available, stopping at the checker evidence-query gap for missing base-shape/constructor-witness and full attributed-type evidence instead of reporting a missing mode-expansion payload. Checker task 61 adds active diagnostic coverage for a same-module attributed local mode reserve head whose real direct attributed-builtin RHS expansion is available, stopping at the checker evidence-query gap for missing full attributed-type evidence instead of reporting a missing mode-expansion payload. Checker task 62 adds active diagnostic coverage for a one-edge bare local-mode chain ending in a same-module local structure RHS, stopping at the checker evidence-query gap for missing base-shape/constructor-witness evidence instead of reporting a missing mode-expansion payload. Checker task 63 adds active diagnostic coverage for a one-edge bare local-mode chain ending in an attributed builtin RHS, stopping at the checker evidence-query gap for missing attributed-type existential evidence instead of reporting a missing mode-expansion payload. Checker task 72 adds active pass coverage for two-edge bare local-mode chains ending in builtin `set` / `object`; checker task 73 adds active pass coverage for three-edge bare local-mode chains; checker task 74 replaces the temporary chain-depth guard with AST-bounded structural pass coverage for bare same-module no-argument local-mode chains ending in builtin `set` / `object`. Checker task 81 confirms a same-module parameterized attribute declared with numeral `param_prefix` syntax and used through `attribute_name(args)` reaches the active runner before failing closed on the checker source-to-payload extraction gap. Checker task 82 confirms the documented `parser.type_fixtures` `TypeCaseMode` imported mode reserve head carries real imported mode provenance/type-head payloads to the checker before failing closed on the missing imported mode-expansion payload. Checker task 83 confirms the documented `parser.type_fixtures` imported structure `R` carries real imported structure provenance/type-head payloads to the checker before failing closed on the missing base-shape/constructor-witness evidence query. Checker task 97 confirms the documented imported structure `TypeCaseStruct` carries the same real imported structure provenance/type-head payloads to the checker before failing closed on that missing evidence query. Checker task 84 confirms the documented `parser.type_fixtures` imported attribute `TypeCaseAttr` carries real imported attribute provenance/`AttributeInput` payloads to the checker before failing closed on the missing attributed-type evidence query. Checker task 85 confirms the documented `parser.type_fixtures` imported attribute `empty` carries real imported negative `AttributeInput` payloads over builtin `set` for the existing `non empty set` fixture before failing closed on the same missing attributed-type evidence query. Checker task 116 confirms the matching positive `empty set` fixture carries real imported positive `AttributeInput` payloads over builtin `set` before failing closed on that evidence query. | partial | AST-wide source-derived checker payload extraction, imported attributes beyond the task-84 `TypeCaseAttr` provenance/`AttributeInput` bridge, task-85/task-116 `empty`/builtin-`set` bridges, and task-80 diagnostic boundary, imported structures beyond the task-83 `R` and task-97 `TypeCaseStruct` provenance/type-head bridges and task-78 diagnostic boundary, imported mode expansions beyond task 82's provenance/type-head bridge, attribute argument payloads beyond the task-81 diagnostic boundary, mode/structure arguments, broader/attributed/argument-bearing/parameterized/contextual/ambiguous/cyclic mode expansion, structure base-shape/full attributed-type existential evidence, and positive attributed or structure type acceptance remain external to the current checker/core milestones. |
| `04.variables_and_constants.md` | Parser grammar covers `reserve`, `let`, `set`, `take`, `given`, `consider`, `reconsider`, `deffunc`, and `defpred`; core binder normalization covers closures, free variables, alpha-equivalence, and substitution. Checker task 44 aligns `reconsider` optional-justification syntax with Chapter 8's semantic gate. Checker task 119 adds exact active type-elaboration pass coverage for `reserve x for set; theorem ReservedVariableEqualityPayloadBoundary: x = x;`: both identifier terms resolve through the real reserve `BindingEnv` and reuse the written builtin `set` type for result/expected payloads. Checker task 123 adds the exact distinct-binding sibling `reserve x, y for set; theorem DistinctReservedVariableEqualityPayloadBoundary: x = y;`: the two real checker bindings retain one shared written type range and resolve independently. | partial | Tasks 119 and 123 credit only exact same-binding and distinct-binding reserved-variable term/type/equality well-formedness; implicit universal-closure/order nodes, theorem acceptance, equality facts/truth, broader reserved-variable uses, and proof/Core/VC payloads remain deferred. Parser task 47 owns the current parser/source drift where omitted `reconsider` justification is still parsed as a recovery error and proof-block `reconsider` is not yet accepted. |
| `05.structures.md` | Parser/syntax covers structure declarations and inheritance surfaces. Checker tasks 35-36 record the fields-only constructor/property-value source decision plus the root+path/view inheritance identity, exact coverage, and acyclicity decisions with inactive semantic corpus and traceability. Core task 27 implements explicit-payload reduct-view lowering for renamed/multi-path `qua` views and preserves exact-instance guard formulas on reduct terms. Kernel task 35 re-audits the soundness argument against view terms and records no kernel invariant or corpus-sidecar change: view choices are part of normalized atom subject bytes. Checker task 52 confirms a same-module source-derived local structure symbol can reach reserve declaration checking and fail closed on the missing base-shape evidence query; task 53 confirms the same structure head can carry source-derived attributes while still failing closed on the full attributed-type evidence query. Checker task 57 confirms a same-module local mode expansion can reach a local structure RHS and then fail closed on missing base-shape/constructor-witness evidence. Checker task 60 confirms the same direct local-structure RHS expansion can be consumed through an attributed local-mode reserve head while still failing closed on missing base-shape/constructor-witness and full attributed-type evidence. Checker task 62 confirms a one-edge bare local-mode chain can consume a real terminal local-structure RHS expansion while still failing closed on missing base-shape/constructor-witness evidence. Checker task 76 confirms a forward same-module local-structure reserve head fails lower-stage active-range checking before any checker structure type-head payload, base-shape query, or constructor-witness query is produced. Checker task 83 confirms the documented imported structure `R` can reach reserve declaration checking and fail closed on the missing base-shape/constructor-witness evidence query. Checker task 97 confirms the documented imported structure `TypeCaseStruct` reaches the same reserve declaration checking boundary and fails closed on the same missing evidence query. Checker task 92 adds active type-elaboration boundary coverage for a structure definition inside a source `definition` block, but keeps structure definition declaration, field/selector, base-shape/constructor, and evidence payload extraction on the checker source-to-payload extraction gap. Checker docs still record selector and structure-field payload gaps. | partial | Resolver/checker payload work must provide real source-derived structure identity/path-view payloads, selector facts, broader constructor coverage, field visibility, base-shape/constructor-witness evidence, full attributed-type existential evidence, and proof-obligation inputs before downstream semantics claim full coverage. Task 76 credits only the structure syntax/type-head surface under active-range/no-forward-reference rejection, tasks 83 and 97 credit only imported `R`/`TypeCaseStruct` provenance/type-head extraction before the missing evidence query, and task 92 credits only the structure definition extraction-gap boundary, not structure definition payload extraction or downstream semantic payloads. |
| `06.attributes.md` | Parser/syntax covers attribute definitions and tests; checker covers normalized attributes, contradiction checks, and fact queries. Checker task 41 records that `attr_pattern` declares parameter slots and `attribute_name(args)` is only a use-site application form. Checker task 50 confirms same-module source-derived attribute symbols can reach declaration checking on builtin reserve heads as real payloads and fail closed on missing evidence. Checker task 53 confirms those same no-argument attribute payloads can be attached to same-module local structure reserve heads and still fail closed without existential evidence. Checker task 58 confirms the same no-argument attribute payloads can be carried through a real local-mode attributed-builtin RHS expansion while still failing closed without attributed-type existential evidence. Checker task 59 confirms the same no-argument attribute payloads can be attached to a same-module local-mode reserve head once a real direct bare-builtin mode expansion is available, still failing closed without attributed-type existential evidence. Checker task 60 confirms those attribute payloads can also be attached when the real direct local-mode expansion has a local-structure RHS, still failing closed without base-shape/constructor-witness and full attributed-type evidence. Checker task 61 confirms those attribute payloads can be present on both the same-module local-mode reserve head and the real direct attributed-builtin RHS expansion, still failing closed without full attributed-type evidence. Checker task 63 confirms the same no-argument attribute payloads can be carried through a one-edge bare local-mode chain ending in an attributed builtin RHS, still failing closed without attributed-type existential evidence. Checker task 77 confirms a forward same-module local-attribute reserve type expression fails lower-stage active-range checking before any checker `AttributeInput` payload or attributed-type evidence query is produced. Checker task 80 historically confirms imported attribute reserve types from the documented `parser.type_fixtures` import summary reach the active runner at the source-to-checker extraction gap; checker task 84 supersedes the documented `TypeCaseAttr` portion by carrying real imported attribute provenance/`AttributeInput` payloads to the checker evidence-query gap; checker task 85 supersedes the existing negative `empty`/builtin-`set` fixture by carrying real imported negative `AttributeInput` payloads to the same evidence-query gap; checker task 116 supersedes the matching positive `empty`/builtin-`set` fixture by carrying real imported positive `AttributeInput` payloads to that same evidence-query gap. Checker task 81 confirms a same-module parameterized attribute declared with `param_prefix` syntax and used through `attribute_name(args)` reaches the active runner but remains on the source-to-checker extraction gap until real term-argument provenance and checker `AttributeInput` argument payload extraction exist. Checker task 91 adds active type-elaboration boundary coverage for an attribute definition inside a source `definition` block, but keeps attribute definition declaration and formula-definiens payload extraction on the checker source-to-payload extraction gap. Checker task 113 supersedes task 103 for the exact imported `empty` attribute assertion theorem formula by validating imported attribute provenance and passing source-derived numeral and attribute-assertion checker payloads before failing closed on missing numeric type and formula/attribute semantic payloads; checker task 114 supersedes task 104 for the exact attribute-level `non empty` imported attribute assertion variant by validating imported attribute provenance and passing source-derived numeral and attribute-assertion checker payloads before failing closed on missing numeric type and formula/attribute semantic payloads. Tasks 113 and 114 still keep imported attribute assertion attribute-chain semantic payload extraction, theorem-formula `AttributeInput` payload extraction, attribute admissibility/semantic checking, formula checking, and theorem acceptance deferred; task 114 also keeps negated attribute admissibility/semantic checking deferred. | partial | Attribute definition correctness, definition-local context, formula body checking, broader attribute assertion payload extraction, imported attribute theorem-formula provenance beyond task 113 exact `empty` bridge and task 114 exact `non empty` bridge, imported attribute-level non-empty assertion semantic payload/provenance, negated attribute admissibility/semantic checking, attribute admissibility/semantic checking, attributed-type evidence, accepted facts, and proof evidence remain external. Imported attribute symbols beyond the task-84 `TypeCaseAttr` bridge, task-85/task-116 `empty`/builtin-`set` bridges, and task-80 diagnostic boundary, attribute argument payloads beyond the task-81 diagnostic boundary, accepted registration/proof status, existential evidence queries, and artifact-fed activated summaries remain external. Task 77 credits only the attribute syntax/use surface under active-range/no-forward-reference rejection; task 84 credits only imported attribute provenance/no-argument `AttributeInput`; task 85 credits only imported negative `empty` provenance/no-argument `AttributeInput` over builtin `set`; task 116 credits only imported positive `empty` provenance/no-argument `AttributeInput` over builtin `set`; task 91 credits only the attribute definition extraction-gap boundary, not attribute definition payload extraction or downstream semantic payloads; task 113 credits only exact imported `empty` provenance and theorem-formula checker handoff, not theorem-formula `AttributeInput`, attribute-chain semantic payloads, or attribute checking; task 114 credits only exact imported `non empty` provenance and theorem-formula checker handoff, not theorem-formula `AttributeInput`, negated attribute-chain semantic payloads, or negated attribute checking. |
| `07.modes.md` | Parser/syntax and checker type-normalization docs cover mode syntax and unfolding boundaries. Checker task 35 pins constructor arguments as not being a property-value source, task 39 pins overlapping property implementations as requiring coherence, task 43 pins guarded parameterized mode-existence/sethood obligations plus exported sethood status, and checker task 47 adds owner-crate explicit-payload coverage for accepted-mode base inhabitation evidence keyed to the same normalized argument tuple. Checker task 51 confirms a same-module source-derived local mode symbol can reach reserve type normalization and fail closed on the missing real mode-expansion payload; task 54 confirms the same source-derived local mode head can carry same-module attributes while still failing closed on the missing expansion payload when no supported real expansion is available or the same mode is mixed with a bare reserve use. Checker task 55 confirms a bare same-module local mode reserve head can consume a real AST-derived no-argument bare-builtin RHS expansion and pass the active type-elaboration bridge. Checker task 56 confirms the bridge can consume a real one-edge same-module local-mode expansion chain when the dependency mode has that accepted builtin RHS expansion, while attributed dependencies still fail closed. Checker task 57 confirms a real same-module local-mode expansion may have a local structure RHS, but still fails closed at the structure evidence query until base-shape evidence extraction exists. Checker task 58 confirms a real same-module local-mode expansion may have an attributed builtin RHS, but still fails closed at the attributed-type evidence query until existential evidence extraction exists. Checker task 59 confirms a same-module attributed local-mode reserve head may consume a real direct bare-builtin mode expansion, but still fails closed at the attributed-type evidence query until existential evidence extraction exists. Checker task 60 confirms a same-module attributed local-mode reserve head may consume a real direct local-structure RHS mode expansion, but still fails closed until structure base-shape/constructor-witness and full attributed-type evidence extraction exist. Checker task 61 confirms a same-module attributed local-mode reserve head may consume a real direct attributed-builtin RHS mode expansion, but still fails closed until full attributed-type evidence extraction exists. Checker task 62 confirms a one-edge bare local-mode chain may consume a real terminal local-structure RHS mode expansion, but still fails closed until structure base-shape/constructor-witness evidence extraction exists. Checker task 63 confirms a one-edge bare local-mode chain may consume a real terminal attributed-builtin RHS mode expansion, but still fails closed until attributed-type existential evidence extraction exists. Checker task 72 confirms a two-edge bare local-mode chain may consume real same-module local-mode expansions when the terminal RHS is builtin `set` / `object`; checker task 73 confirms the same for three-edge bare local-mode chains; checker task 74 removes the temporary depth cap for the narrow bare builtin-terminal family and confirms AST-bounded structural chains, including cached and long chains, pass under the same unique/unrecovered/same-module/no-argument/source-preceding guards; checker task 75 confirms forward local-mode reserve heads fail at lower-stage active-range checking before any checker mode-expansion payload is produced; checker task 79 confirms imported mode reserve heads from the documented `parser.type_fixtures` import summary reach the active runner, and checker task 82 confirms the same source can carry real imported mode provenance/type-head payload to the checker before failing closed on the missing imported mode-expansion payload, and checker task 92 adds active type-elaboration boundary coverage for a mode definition inside a source `definition` block while keeping mode definition declaration payload extraction and mode expansion on the checker source-to-payload extraction gap. | partial | Active property-implementation parser support/fixtures, broader/imported/attributed/argument-bearing/parameterized/contextual/ambiguous/cyclic resolver/checker mode-expansion payloads beyond task 82's imported-mode provenance bridge, mode arguments, positive property implementation payloads, accepted coherence status, source-derived sethood evidence, structure base-shape evidence, full attributed-mode existential evidence, mode definition declaration payloads beyond task 92's extraction-gap boundary, and broader source-to-checker extraction remain required for full source coverage. Task 92 does not credit mode definition payload extraction or downstream semantic payloads. |
| `08.type_inference.md` | Checker type-checker and overload-resolution docs cover declaration checking, facts, coercion candidates, `qua`, and recovery. Checker task 44 pins omitted `reconsider` justification to proof-free widening/inheritance/cluster-closure/local-fact discharge and names `type.narrowing_requires_proof` for the missing-proof case, with inactive semantic corpus. Checker task 47 adds owner-crate explicit-payload Rust coverage for `CoercionJustification::Omitted`, consumable proof-free evidence markers, and the no-implicit-obligation failure path. | partial | Active checker-stage `.miz` coverage and source extraction are still tracked as external gaps in checker docs. Parser task 47 must align parse-only recovery with the optional syntax and proof-block `reconsider` grammar before parser coverage can claim this surface. |
| `09.predicates.md` | Parser/syntax covers predicate definitions and applications; checker/core/VC cover semantic handoff at a higher level. Checker task 90 adds active type-elaboration boundary coverage for a predicate definition inside a source `definition` block, but keeps predicate definition declaration and definiens payload extraction on the checker source-to-payload extraction gap. | partial | Predicate definition correctness, definition-local context, formula body checking, overload payloads, accepted facts, and proof evidence remain downstream/external until source payloads and proof consumers are wired. Task 90 does not credit predicate definition payload extraction or downstream semantic payloads. |
| `10.functors.md` | Parser/syntax covers functor definitions/applications; checker overload docs cover candidates and viability. Checker task 90 adds active type-elaboration boundary coverage for a functor definition inside a source `definition` block, but keeps functor definition declaration and definiens payload extraction on the checker source-to-payload extraction gap. | partial | Functor definition correctness, definition-local context, term/formula body checking, overload payloads, reductions, accepted registration/redefinition payloads, and proof evidence remain deferred to checker/VC/proof handoffs. Task 90 does not credit functor definition payload extraction or downstream semantic payloads. |
| `11.symbol_management.md` | Lexer lexical environment, parser syntax, resolver env/symbol/name docs, and artifact summaries cover current symbol surfaces. Checker tasks 75/76/77 add active diagnostic coverage for the module-item ordering rule that later same-module local mode, structure, or attribute declarations do not make a symbol visible to earlier reserve type expressions. Checker task 78 originally covered the documented imported structure `R` extraction-gap boundary before task 83 superseded that `R` portion, checker task 79 adds the matching imported mode symbol boundary, checker task 80 adds the matching imported attribute symbol boundary before task 84 supersedes the documented `TypeCaseAttr` portion, task 85 supersedes the negative `empty`/builtin-`set` portion, and task 116 supersedes the positive `empty`/builtin-`set` portion, checker task 82 promotes the imported mode symbol to real checker type-head provenance while still failing on missing expansion, checker task 83 promotes imported structure `R` to real checker type-head provenance while still failing on missing structure evidence, checker task 97 promotes imported structure `TypeCaseStruct` to the same real checker type-head provenance while still failing on missing structure evidence, checker task 84 promotes imported attribute `TypeCaseAttr` to real checker `AttributeInput` provenance while still failing on missing attributed-type evidence, and checker task 85 promotes imported attribute `empty` to real negative checker `AttributeInput` provenance over builtin `set` while still failing on missing attributed-type evidence, and checker task 116 promotes the matching positive `empty`/builtin-`set` source to real positive checker `AttributeInput` provenance while failing on the same evidence gap. Broader imported structures outside task 83/task 97 and broader imported attributes outside task 84/task 85/task 116 remain deferred. Checker task 81 adds resolver declaration-symbol coverage for a parameterized local attribute whose suffix is the lexer-visible primary spelling while the prefixed surface remains notation/signature data. | covered | Continue R-024 summary-backed reuse without resolver-local artifact formats; forward-reference acceptance remains forbidden by active-range rules and covered by task 75/76/77 lower-stage rejection. Task 78 is historical for the `R` extraction-gap boundary now superseded by task 83 and the `TypeCaseStruct` boundary now superseded by task 97; broader imported structures remain deferred. Task 80 is historical for the `TypeCaseAttr`, negative `empty`, and positive `empty` extraction-gap boundaries now superseded by task 84/task 85/task 116; broader imported attributes remain deferred. Task 82 credits imported mode provenance/type-head extraction but not imported mode expansion, task 83 credits imported `R` structure provenance/type-head extraction and task 97 credits imported `TypeCaseStruct` provenance/type-head extraction, but neither credits imported module AST extraction or structure evidence, task 84 credits imported `TypeCaseAttr` attribute provenance/`AttributeInput` extraction, task 85 credits imported negative `empty` attribute provenance/`AttributeInput` extraction over builtin `set`, and task 116 credits imported positive `empty` attribute provenance/`AttributeInput` extraction over builtin `set`; none of these tasks credit imported module AST extraction, attributed-type evidence, positive attributed-type acceptance, non-`set` imported `empty`, owner provenance, or downstream evidence extraction. Task 81 credits only declaration-symbol suffix projection and the source-to-checker extraction-gap boundary, not real attribute argument payload extraction. Task 96 credits only the parser/resolver-executable redefinition/notation source boundary and source-to-checker extraction-gap diagnostic, not alias relation resolution, visibility/export semantics beyond declaration-symbol collection, semantic equivalence, redefinition target inference, overload payloads, or advanced_semantics runner support. Task 110 supersedes task 98 for the exact imported predicate/functor theorem formula by crediting real checker term/formula payload handoff before missing numeric/signature payload and partial-formula diagnostics, task 113 supersedes task 103 for the exact imported `empty` attribute assertion theorem formula by crediting imported attribute provenance plus checker term/formula handoff before missing numeric/formula semantic payload diagnostics, and task 114 supersedes task 104 for the exact attribute-level non-empty imported attribute assertion theorem formula by crediting imported attribute provenance plus checker term/formula handoff before missing numeric/formula semantic payload diagnostics; none credits imported semantic payloads, imported module AST extraction, broader term/formula payload extraction beyond the exact task-110/task-113/task-114 handoffs, attribute assertion payloads, checker `AttributeInput` extraction for theorem formulas, formula checking, theorem facts, or formula_statement runner support. |
| `12.modules_and_namespaces.md` | Architecture 03, build module-index docs, resolver imports/env/name docs, and artifact module-summary docs cover module graph and namespace boundaries. Checker task 78 is historical for the documented imported structure `R` extraction-gap boundary now superseded by task 83, checker task 80 is historical for the documented imported attribute extraction-gap boundary now superseded for `TypeCaseAttr` by task 84, for negative `empty`/builtin-`set` by task 85, and for positive `empty`/builtin-`set` by task 116, checker task 79 adds active diagnostic boundary coverage for mode reserve surfaces read through the documented import-summary fixture, checker task 82 promotes the imported mode surface to real imported symbol provenance/type-head extraction only, checker task 83 promotes the imported structure `R` surface to real imported symbol provenance/type-head extraction only, checker task 97 promotes the imported structure `TypeCaseStruct` surface to the same provenance/type-head extraction boundary, checker task 110 supersedes task 98 for the exact imported predicate/functor theorem formula by validating imported predicate/functor provenance and passing real checker term/formula payloads before missing numeric/signature payload and partial-formula diagnostics, checker task 113 supersedes task 103 for the exact imported `empty` attribute assertion theorem formula by validating imported attribute provenance and passing checker term/formula payloads before missing semantic payload diagnostics, checker task 114 supersedes task 104 for the exact attribute-level non-empty imported attribute assertion theorem formula by validating imported attribute provenance and passing checker term/formula payloads before missing semantic payload diagnostics, checker task 84 promotes the imported attribute `TypeCaseAttr` surface to real imported symbol provenance/`AttributeInput` extraction only, and checker task 85 promotes the imported attribute `empty` surface to real imported negative `AttributeInput` extraction only for builtin `set`, and checker task 116 promotes the matching positive `empty`/builtin-`set` source to real imported positive `AttributeInput` extraction. Broader imported structures outside task 83/task 97 and broader imported attributes outside task 84/task 85/task 116 remain deferred. | covered | Resolver R-024 remains the immediate reuse integration task. Task 78 is historical for the `R` extraction-gap boundary now superseded by task 83, and broader imported structures remain deferred. Task 80 is historical for the `TypeCaseAttr`, negative `empty`, and positive `empty` extraction-gap boundaries now superseded by task 84/task 85/task 116, and broader imported attributes remain deferred. Task 84, task 85, and task 116 do not claim real imported module AST extraction, attributed-type evidence, owner provenance, arguments, or positive imported attributed-type elaboration; task 116 also does not claim positive attributed-type acceptance, and neither empty bridge claims imported `empty` on non-`set` heads; task 82 does not claim imported module AST extraction or imported mode expansion; task 83 and task 97 do not claim imported module AST extraction, base-shape/constructor-witness evidence, or positive imported structure elaboration; task 110 does not claim imported module AST extraction, semantic predicate/functor signatures, term inference, formula checking, theorem facts, or formula_statement runner support. Task 113 and task 114 do not claim imported module AST extraction, imported attribute assertion semantic payloads, theorem-formula `AttributeInput` extraction, formula checking, theorem facts, or formula_statement runner support; task 114 also does not claim negated attribute-chain semantic payloads or negated attribute checking. |
| `13.term_expression.md` | Parser/syntax covers terms; checker/core cover typed terms, inserted views, and lowering. Checker task 43 pins Fraenkel sethood lookup to the resolved mode and normalized instantiated argument tuple. Core task 27 adds explicit-payload `qua` reduct term lowering with distinct renamed/multi-path view terms and no-reduct identity/cluster reuse. Kernel task 35 confirms those view terms remain ordinary normalized term subjects for kernel atom identity; the kernel does not infer or collapse `qua` paths. Core task 30 adds explicit-payload Fraenkel sethood gating for template type parameters by cross-referencing accepted bound/constraint sethood records and preserving bare parameters as missing sethood. Checker task 106 supersedes task 87 for the exact builtin equality theorem `1 = 1` slice by passing real source-derived numeral `TermInput`s to the checker before failing on missing numeric type payloads, checker task 110 supersedes task 98 for the exact imported predicate/functor term-application theorem formula by passing real checker term/formula payloads before failing closed, checker task 108 supersedes task 100 for the builtin membership variant `theorem BuiltinMembershipPayloadBoundary: 1 in 1;` by passing real checker term/formula payloads before failing closed with numeral operands, checker task 107 supersedes task 101 for the exact builtin inequality theorem `1 <> 2` slice by passing real source-derived numeral `TermInput`s to the checker before failing on missing numeric type payloads, checker task 109 supersedes task 102 for the exact builtin type-assertion theorem `1 is set` slice by passing a real source-derived numeral `TermInput` and asserted builtin `set` `TypeExpressionInput` before failing on missing numeric type payloads, checker task 113 supersedes task 103 for the exact imported attribute assertion theorem formula by passing a real source-derived numeral `TermInput` before failing on missing numeric and formula/attribute semantic payloads, checker task 114 supersedes task 104 for the exact attribute-level non-empty imported attribute assertion variant by passing a real source-derived numeral `TermInput` before failing on missing numeric and formula/attribute semantic payloads, and checker task 111 supersedes task 105 for the exact set-enumeration theorem by passing four source-derived numeral item `TermInput`s and two set-enumeration `TermInput`s before failing on missing numeric and result-type/sethood payloads. Checker tasks 119-123 add exact positive reserved-variable identifier-term inference for same-binding equality, membership, inequality, reflexive type assertion, and distinct-binding equality over one shared multi-reserve type range. | partial | Source-derived payloads and term inference beyond the exact Tasks 119-123 reserved-variable slices, positive source-derived sethood evidence flow, real checker view-functor/sethood extraction, and semantic selector/constructor facts remain owner-gated. Tasks 119-123 credit type/well-formedness only and do not credit implicit closure/order, truth/facts, theorem acceptance, or downstream payloads. Tasks 106, 107, 108, and 109 credit only narrow numeral term handoff and still lack numeric type payloads, successful term inference, and accepted equality/inequality/membership/type-assertion facts; task 109 also credits only the exact builtin `set` asserted-type handoff, not broader asserted type payloads or type-assertion semantic checking. Task 110 credits only the exact imported predicate/functor term/formula handoff and not semantic signatures or term inference; task 111 credits only the exact set-enumeration term handoff and not result-type/sethood payload extraction or term inference; task 113 credits only the exact imported attribute assertion numeral term handoff and not numeric type payloads or term inference; task 114 credits only the exact attribute-level non-empty imported attribute assertion numeral term handoff and not numeric type payloads or term inference; neither credits imported predicate/functor semantic payloads, membership operand expected-type construction/checking beyond task 120, inequality desugaring or equality semantic checking beyond tasks 119/121/123, broader type-assertion type payload extraction or reachability beyond task 122, imported attribute assertion attribute-chain/provenance payload extraction, imported attribute-level non-empty assertion attribute-chain/provenance semantic payload extraction, broader set-enumeration term payload extraction, negated attribute admissibility/semantic checking, attribute admissibility/semantic checking, quantifier binder/context payloads, formula payloads, or downstream semantic payloads. |
| `14.formulas.md` | Parser/syntax covers formulas; checker/core/VC cover typed formulas, erasure, proof goals, and generated obligations. Checker task 86 adds active type-elaboration boundary coverage for a formula-only theorem source that reaches parser/resolver execution; checker task 117 supersedes task 115 for the exact `FormulaPayloadBoundary: thesis` source by passing the source-derived `thesis` formula constant as a real `FormulaKind::Thesis` checker payload before failing closed on missing formula payload. Checker task 106 supersedes task 87 for the exact term-bearing builtin equality theorem formula by passing a real source-derived checker equality `FormulaInput` before failing on partial formula checking, task 110 supersedes task 98 for the exact imported predicate/functor theorem formula checker bridge, task 108 supersedes task 100 for the exact builtin membership theorem formula by passing a real source-derived checker membership `FormulaInput` before failing on partial formula checking, task 107 supersedes task 101 for the exact builtin inequality theorem formula by passing a real source-derived checker inequality `FormulaInput` before failing on partial formula checking, task 109 supersedes task 102 for the exact builtin type-assertion theorem formula by passing a real source-derived checker type-assertion `FormulaInput` and asserted builtin `set` `TypeExpressionInput` before failing on partial formula checking, task 113 supersedes task 103 for the exact imported attribute assertion theorem formula by passing a real checker `AttributeAssertion` `FormulaInput` before missing semantic payload diagnostics, task 114 supersedes task 104 for the exact attribute-level non-empty imported attribute assertion theorem formula by passing a real checker `AttributeAssertion` `FormulaInput` before missing semantic payload diagnostics, task 111 supersedes task 105 for the exact set-enumeration equality theorem by passing a real checker equality `FormulaInput` over two set-enumeration term sites before failing on partial formula checking, task 112 supersedes task 99 for the exact connective/quantifier theorem formula by passing real checker `FormulaInput` shells for implication, universal quantification, and negation before failing on missing formula/quantifier payloads, task 117 extends that exact source by passing both `contradiction` constants as real `FormulaKind::Contradiction` checker payloads before the same missing formula payload diagnostic, task 88 adds the proof-block theorem variant whose `thus thesis;` conclusion still depends on formula/proof payload extraction, and task 89 adds the statement-level proof-justification variant with nested proof blocks. Checker tasks 119-123 add exact positive formula type/well-formedness for same-binding equality, membership, inequality, reflexive type assertion, and distinct-binding equality. | partial | Complete source-derived formula payloads and formula checking beyond the exact Tasks 119-123 reserved-variable slices, formula constant semantic checking, child-formula graph payloads, term inference, membership operand expected-type construction/checking, inequality desugaring or equality semantic checking, broader type-assertion type payload extraction, type-assertion semantic checking, imported attribute assertion attribute-chain/provenance payload extraction, imported attribute-level non-empty assertion attribute-chain/provenance semantic payload extraction, set-enumeration result-type/sethood payload extraction, negated attribute admissibility/semantic checking, attribute admissibility/semantic checking, facts, proof contexts, `formula_statement` execution, proof skeleton extraction, and proof/VC integration remain external. Tasks 119-123 do not credit implicit closure/order, truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr, or VC. Tasks 106, 107, 108, and 109 credit only narrow equality/inequality/membership/type-assertion formula handoffs and still lack numeric type payloads, equality/inequality/membership/type-assertion semantic checking, and accepted formula facts; task 109 also credits only the exact builtin `set` asserted-type handoff. Task 110 credits only the exact imported predicate/functor formula handoff and not semantic signatures, formula checking, or accepted facts; task 111 credits only the exact set-enumeration equality handoff and not equality checking or sethood/result-type semantics; task 112 credits only exact formula shell handoff and not child-formula graph payloads, quantifier binder/context payloads, formula checking, or accepted facts; task 117 credits only exact `thesis`/`contradiction` formula constant kind payloads and not formula constant semantics, child graph payloads, formula checking, or accepted facts; tasks 105, 88, and 89 do not credit formula payload extraction, and tasks 113 and 114 credit only exact imported attribute assertion formula handoffs, not formula checking or semantic payloads; these tasks do not credit imported predicate/functor semantic signatures beyond task 110, builtin membership operand checking beyond task 120, builtin inequality desugaring/equality checking beyond task 121, broader builtin type-assertion payload/checking beyond tasks 109/122, imported attribute assertion payload/checking, imported attribute-level non-empty assertion payload/checking, broader set-enumeration term payload extraction, equality checking beyond tasks 119/123, quantifier binder/context payloads, formula payloads, or downstream semantic payloads. |
| `15.statements.md` | Parser/syntax covers statement surfaces and recovery; core/proof/VC docs consume proof and algorithm statements through explicit payloads. Checker task 44 updates `reconsider` statement grammar to optional simple justification or proof block plus the Chapter 8 semantic gate. Checker task 88 adds active type-elaboration diagnostic coverage for a source-derived `thus thesis;` conclusion inside a theorem proof block, task 89 adds the same diagnostic boundary for statement-level proof justifications, and task 93 adds proof-local `let`, `given`, `consider`, `set`, and `reconsider` statement coverage, and task 94 adds proof-local `deffunc` and `defpred` inline definition coverage, but all four keep proof-statement, proof-local declaration, and inline definition payload extraction on the checker source-to-payload extraction gap. | partial | Proof-verification source runner, proof-statement payload extraction, proof-local declaration payload extraction, inline definition formal/body payload extraction, local proof context, label-reference semantic checking, reconsider coercion/obligation evidence, local abbreviation expansion, theorem acceptance, and full source-to-core extraction remain deferred. Current parser omitted-justification recovery and proof-block `reconsider` rejection are `test_expectation_drift` / `source_drift` owned by deferred parser task 47. |
| `16.theorems_and_proofs.md` | Core, VC, ATP, kernel, proof, cache, artifact, and diagnostics docs cover the current proof pipeline boundaries. Checker task 86 adds active type-elaboration boundary coverage for the theorem formula slot using `theorem FormulaPayloadBoundary: thesis;`, checker task 117 supersedes task 115 for that exact source by passing the source-derived `thesis` formula constant as a real `FormulaKind::Thesis` checker payload before failing closed, checker task 106 supersedes task 87 for the term-bearing equality variant `theorem TermFormulaPayloadBoundary: 1 = 1;` by passing real checker term/formula payloads before failing closed, checker task 110 supersedes task 98 for the exact imported predicate/functor theorem checker bridge, checker task 108 supersedes task 100 for the builtin membership variant `theorem BuiltinMembershipPayloadBoundary: 1 in 1;` by passing real checker term/formula payloads before failing closed, checker task 107 supersedes task 101 for the builtin inequality variant `theorem BuiltinInequalityPayloadBoundary: 1 <> 2;` by passing real checker term/formula payloads before failing closed, checker task 109 supersedes task 102 for the exact builtin type-assertion theorem variant by passing real checker term/formula/asserted-type payloads before failing closed, checker task 113 supersedes task 103 for the exact imported attribute assertion theorem formula variant by passing real checker term/formula payloads before failing closed, checker task 114 supersedes task 104 for the exact attribute-level non-empty imported attribute assertion theorem formula variant by passing real checker term/formula payloads before failing closed, checker task 111 supersedes task 105 for the exact set-enumeration theorem variant by passing real checker term/formula payloads before failing closed, checker task 112 supersedes task 99 for the exact connective/quantifier theorem formula variant by passing real checker formula shell payloads before failing closed, checker task 117 also passes both exact `contradiction` constants in that source as real `FormulaKind::Contradiction` payloads, checker task 88 adds the proof-block variant `theorem ProofSkeletonPayloadBoundary: thesis proof thus thesis; end;`, checker task 89 adds a statement-proof theorem variant with labeled and final proof-justified statements, checker task 93 adds a theorem proof containing proof-local declarations, and checker task 94 adds a theorem proof containing proof-local inline definitions. Checker tasks 119-123 add exact positive theorem-slot term/formula type/well-formedness for same-binding equality, membership, inequality, reflexive type assertion, and distinct-binding equality, but all of these tasks still keep theorem acceptance, facts, proof-skeleton/statement, proof-local declaration, inline definition, imported predicate/functor semantic signatures beyond task 110, builtin membership operand checking beyond task 120, builtin inequality desugaring/equality checking beyond task 121, broader builtin type-assertion payload/checking beyond tasks 109/122, set-enumeration result-type/sethood payload extraction, equality checking beyond tasks 119/123, imported attribute assertion payload/checking, imported attribute-level non-empty assertion payload/checking, formula constant semantic checking beyond task 117's exact constant-kind handoff, child-formula graph payloads, connective/quantifier formula semantics beyond task 112/task 117, quantifier binder/context payloads, and proof semantics on the external gap. | partial | End-to-end theorem acceptance, proof/cache/artifact consumer integration, recorded facts, proof skeleton payloads, statement proof payloads, proof-local declaration payloads, inline definition formal/body payloads, numeric/signature/result-type payloads beyond tasks 106, 107, 108, 109, 110, 111, 113, and 114, formula child/binder payloads beyond task 112, formula constant semantics beyond task 117, term/formula checking beyond the exact Tasks 119-123 reserved-variable slices plus tasks 112, 113, 114, and 117, implicit closure/order representation, proof contexts, local abbreviation expansion, reconsider coercion/obligation evidence, and `formula_statement` execution are still split across evidence-pipeline follow-ups. |
| `17.clusters_and_registrations.md` | Architecture 04/17, checker registration/cluster trace docs, artifact registration summaries, and cache cluster-db docs cover the current data layers. Checker task 38 pins functorial-cluster `for` as a result-type applicability guard in bilingual spec text with inactive semantic corpus and traceability. Checker task 40 pins item-ordered activation with asynchronous acceptance and the non-retroactive ordering seed. Checker task 41 pins the restricted no-argument cluster adjective termination premise, closure-time fatal contradiction diagnostics, and parser rejection of argument-bearing registration adjectives. Checker task 42 pins reduction determinism as a function of term, in-scope activated rules, and discharged side-condition evidence with pattern-first/guard-second/FQN rule selection. Checker task 43 pins the inhabitation-evidence table for attributed existential registrations, built-in `object`/`set`, accepted modes, bare structure constructor witnesses, and bare schema type parameters in template bodies. Checker task 46 adds owner-crate explicit-payload Rust coverage for fatal closure contradiction diagnostics and reduction trace identity over discharged side-condition evidence while preserving `such` as applicability-only for strategy audit. Checker task 47 adds owner-crate explicit-payload Rust coverage for built-in, accepted-mode, structure-constructor, and schema-parameter base inhabitation evidence while documenting the task-40 activation contract as the target of the interim accepted-input policy. Checker task 50 adds active source-derived coverage that attributed reserve declarations without real existential/evidence-query inputs fail closed at the checker boundary. Checker task 51 adds active source-derived coverage that local mode reserve heads without real mode-expansion payloads fail closed before any accepted-mode/base-inhabitation claim. Checker task 52 adds active source-derived coverage that local structure reserve heads without real base-shape/constructor-witness evidence fail closed before any structure-inhabitation claim. Checker task 53 adds active source-derived coverage that attributed local structure reserve heads still require full attributed-type existential evidence and fail closed instead of using bare-structure base evidence. Checker task 54 adds active source-derived coverage that attributed local mode reserve heads still require real mode expansion before any full attributed-type evidence query or accepted-mode claim when no supported real expansion is available or the same mode is mixed with a bare reserve use. Checker task 55 adds active source-derived pass coverage for bare local mode reserve heads whose real AST-derived RHS expansion is builtin `set` / `object`, relying only on the Chapter 17 base-shape inhabitation table for that bare RHS. Checker task 56 extends that active pass coverage to one-edge local-mode chains whose dependency mode has the same accepted bare builtin RHS expansion, while attributed dependencies still fail closed before any attributed-type evidence claim. Checker task 57 adds active source-derived diagnostic coverage for a real local-mode expansion whose RHS is a local structure head and proves the bridge now reports the missing structure evidence query, not a missing expansion payload. Checker task 58 adds active source-derived diagnostic coverage for a real local-mode expansion whose RHS is an attributed builtin head and proves the bridge now reports the missing attributed-type evidence query, not a missing expansion payload. Checker task 59 adds active source-derived diagnostic coverage for an attributed local-mode reserve head whose real direct bare-builtin expansion is available and proves the bridge now reports the missing attributed-type evidence query, not a missing expansion payload. Checker task 60 adds active source-derived diagnostic coverage for an attributed local-mode reserve head whose real direct local-structure RHS expansion is available and proves the bridge now reports the missing full attributed structure-type evidence query, not a missing expansion payload. Checker task 61 adds active source-derived diagnostic coverage for an attributed local-mode reserve head whose real direct attributed-builtin RHS expansion is available and proves the bridge now reports the missing attributed-type evidence query, not a missing expansion payload. Checker task 62 adds active source-derived diagnostic coverage for a one-edge bare local-mode chain ending in a local structure RHS and proves the bridge now reports the missing base-shape evidence query, not a missing expansion payload. Checker task 63 adds active source-derived diagnostic coverage for a one-edge bare local-mode chain ending in an attributed builtin RHS and proves the bridge now reports the missing attributed-type evidence query, not a missing expansion payload. Checker task 72 adds active pass coverage for two-edge bare local-mode chains ending in builtin `set` / `object` using only the existing builtin base-shape table; checker task 73 adds the corresponding three-edge pass coverage; checker task 74 replaces the temporary depth cap with AST-bounded structural pass coverage for bare same-module no-argument local-mode chains ending in builtin `set` / `object`, still without claiming broader accepted-mode or attributed/structure evidence. Checker task 95 adds active source-derived boundary coverage for parser/resolver-executable registration blocks, but keeps registration-item payload extraction, correctness-condition/proof-obligation payloads, accepted activation/evidence status, cluster/reduction semantics, and advanced runner support deferred. Core task 28 consumes explicit checker existential-gate results for template type actuals and preserves accepted registration/base/fact evidence or missing-gate diagnostics without re-running registration activation. | partial | Registration-item payload extraction beyond task 95's boundary, correctness-condition/proof-obligation payloads, accepted status production/import, positive accepted-local activation in source-derived passes beyond task 55/56/74 bare builtin RHS slices, source-derived positive inhabitation table execution for attributed/structure/parameterized cases, broader/imported/attributed/argument-bearing/parameterized/contextual/ambiguous/cyclic real mode-expansion extraction, real structure base-shape evidence extraction, real attributed-type existential evidence extraction, artifact publication, persistent cluster-db materialization, active source-derived cluster closure/contradiction execution, active source-derived functorial-cluster execution, source-derived reduction rule selection execution, source-derived normalization-result dependence, and active source-derived template actual gate execution remain deferred/external in owner TODOs. |
| `18.templates.md` | Parser/syntax covers template syntax; checker overload docs cover explicit template expansion over supplied payloads. Core task 26 pins omitted func/pred template argument inference to mode-unfolded declared argument types, with inactive determinism corpus and traceability. Core task 27 lowers explicit bounded-template view actuals through reduct terms and keeps template-bound facts/field selections on the final view term. Kernel task 35 closes the F1/F3 soundness follow-up for those reduct-view terms without adding kernel semantics or corpus rewrites. Core task 28 lowers explicit schema type-parameter inhabitation assumptions and template type-actual gate records, preserving missing-existential rejection without actual-side existential axioms. Core task 29 preserves explicit scheme-actual validation rows for type/predicate/functor parameters, directional F4 widening evidence, skipped functor-guard obligation seeds, partial/void algorithm rejection diagnostics, and F6 enclosing-parameter substitution metadata without source-derived closure expansion. Core task 30 preserves explicit template type-parameter sethood records, accepts only bound-inherited or constraint-supplied sethood for Fraenkel generation, and keeps bare template type parameters diagnostic-only/missing. Checker task 43 aligns template type-actual inhabitation with Chapter 17's table and preserves F2/F5 negative seeds. Checker task 47 adds owner-crate explicit-payload coverage for schema type-parameter base evidence while keeping source-derived template actual execution deferred. | partial | Active source-corpus execution for template inference, view-actual extraction, type-actual inhabitation acceptance, scheme-actual compatibility, proof-local `defpred`/`deffunc` closure expansion, promoted-algorithm actual extraction, and source-derived sethood evidence flow remains deferred until runner/extraction support exists. |
| `19.overload_resolution.md` | Architecture 05 and checker overload docs cover candidate collection, template expansion, viability, specificity, root selection, refinement join, and `qua` insertion. Checker task 36 pins implicit upcast path uniqueness as syntactic over resolved `inherit` declaration paths. Checker task 37 pins specificity as a preorder, limits template tie-breaks to concrete-vector equivalence after expansion, and covers multiple-maximal-root ambiguity plus same-signature definition conflicts. Checker task 41 links cluster-closure finiteness back to Chapter 17's restricted adjective grammar. Checker task 44 pins omitted `coherence with` target inference to exactly one visible earlier root and names `resolve.ambiguous_redefinition_target` for multi-root cases. Checker task 45 adds owner-crate Rust explicit-payload regressions for equivalent template-derived ambiguity, encoded non-template/template priority, unencoded ordinary/template ties, and same-root redefinition metadata not breaking distinct-root ties, while keeping omitted-target diagnostic production upstream. | covered | Artifact projection and active source-corpus coverage remain external except for task 96's parser/resolver-executable redefinition/notation extraction-gap boundary. Ordinary/template-derived equivalent-root, same-return duplicate, ambiguous redefinition-target, alias semantic-resolution, target-inference, coherence-obligation, and overload-candidate seeds stay inactive until runner/diagnostic payload support lands. |
| `20.algorithm_and_verification.md` | Parser/syntax, core control-flow, VC generation/discharge, and documentation/extraction docs cover the current algorithm pipeline. | partial | Branch/match/range/collection-loop payloads, term-only/partial termination, Pick non-emptiness, ghost-erasure trace families, MVM/code-extraction backend specs, and source-derived payloads remain TODO/deferred. |
| `21.source_code_annotation_and_atp.md` | Parser/syntax covers annotations; ATP/kernel/proof docs cover solver hints, backend evidence, portfolio, and policy. LSP/docs cover display and extraction consumers. | partial | `@show_*` and `@eval` need end-to-end diagnostic/display/evaluation projection specs before user-facing behavior is complete. |
| `22.error_handling_and_diagnostics.md` | `mizar-diagnostics` registry/failure/sink/render/explain docs cover shared diagnostics. Checker task 44 refines E0102 for omitted `reconsider` justification and reserves E0205 `resolve.ambiguous_redefinition_target` in the spec. | partial | Resolver name/import/label diagnostics still need a real user-facing adoption task and numeric-code mapping within the existing resolution family. E0205 remains spec-reserved rather than an active Rust diagnostics registry row until a producer and source-derived payload support exist. Info/display diagnostics remain reserved until enumerated, and public diagnostic emission for the new semantic seeds remains deferred on source payload/runner support. |
| `23.package_management_and_build_system.md` | Build, artifact, cache, driver, diagnostics, LSP, and architecture docs cover manifest/build/artifact/cache/LSP/explanation slices. Chapter 23's functional-cluster registration-node discussion is synchronized with the Chapter 17 `for` result-guard contract. | partial | `mizar refine`, `mizar minimize`, and production `mizar semver-check` CLI ownership remain future driver/tooling tasks; LSP module specs are still planned. |
| `24.documentation_generation.md` | Architecture 13 and internal 05 define phase-16 boundaries; `mizar-doc` TODO schedules module specs. | todo | `mizar-doc` must write module specs for artifact reading, comments, links, math, render, extraction, backend, publisher, and a source/spec coverage closure audit. |
| `sample_codes.md` | Examples exercise intended language surfaces but are not direct implementation authority. | reference | Keep examples aligned through future source/spec audits. |
| `appendix_a.grammar_summary.md` | Parser/syntax grammar audits and parser TODO tasks cover most current grammar surfaces. Task 39 changes the A.7 `property_impl` grammar and records deferred parser coverage for that block surface. Checker task 44 updates type-changing statement grammar to allow omitted `reconsider` simple justification and proof-block `reconsider`, matching Chapters 4/8/15. | partial | Property implementation block parser support/fixtures remain deferred; parser task 47 owns omitted-`reconsider` justification and proof-block parser alignment; future grammar additions should update parser/syntax audits. |
| `appendix_b.operator_precedence.md` | Parser Pratt design and parser TODO cover precedence/associativity. | covered | Deferred operator-declaration follow-up remains parser task 46. |
| `appendix_c.glossary.md` | Terminology reference. | reference | Use during bilingual sync and user-facing docs. |
| `appendix_d.recommended_coding_rules.md` | Style/reference guidance, not an implementation phase. | reference | No crate task unless a formatter/linter is later specified. |
| `appendix_e.annotation_quick_reference.md` | Annotation reference mirrors chapter 21. | partial | Close together with chapter 21 annotation display/evaluation follow-ups. |

Task119 current-state override for chapters `04.variables_and_constants.md`,
`13.term_expression.md`, `14.formulas.md`, and
`16.theorems_and_proofs.md`: this paragraph is the authoritative qualification
of those chapters' earlier matrix rows. Generic statements there that identifier
terms, formulas, or theorem term/formula checking remain unavailable mean
coverage beyond the exact task-119 slice. Checker task 119 adds the exact active pass source
`reserve x for set; theorem ReservedVariableEqualityPayloadBoundary: x = x;`.
Both identifier terms resolve through independent real reserve `BindingEnv`
lookups at source-order-derived use ordinals; their
result types and the equality expected-type constraints are distinct
source-anchored projections of the written builtin `set` reserve type. The
checker records two `Inferred` variable terms and one type/well-formedness
`Checked` equality without candidates, diagnostics, deferred reasons, or facts.
Production runner validation enforces the complete payload invariants and a
real-frontend/resolver unit test observes the active sidecar payload. The corresponding Chapter 13,
14, and 16 traceability rows change from diagnostic-only to `pass_and_fail`,
and Chapter 4 gains its first exact type-elaboration pass row. This credit does
not include implicit universal-closure nodes, equality truth/facts, theorem
acceptance, `formula_statement`, proof skeletons, CoreIr, ControlFlowIr, VC, or
broader identifier/equality extraction; those remain deferred.

Task120 current-state override for chapters `04.variables_and_constants.md`,
`13.term_expression.md`, `14.formulas.md`, and
`16.theorems_and_proofs.md`: checker task 120 adds the exact active membership
pass source
`reserve x for set; theorem ReservedVariableMembershipPayloadBoundary: x in x;`.
Both identifier results and the right operand's expected `set` type are
source-anchored projections of the written reserve, and independent real
`BindingEnv` lookups resolve the two uses. The checker records two `Inferred`
variables and one no-fact type/well-formedness `Checked` membership. Production
invariants and the real frontend/resolver sidecar payload test guard the slice.
This credit does not include membership truth/facts, implicit closure, theorem
acceptance, proof, CoreIr, ControlFlowIr, VC, or broader term/formula extraction;
those remain deferred. The generic matrix gap wording is qualified by tasks
119 through 125 for exact same-binding equality, membership, inequality, type
assertion, distinct-binding equality, multiple-declaration equality, and
heterogeneous-reserve membership.

Task121 current-state override for chapters 04, 13, 14, and 16: the exact
reserved-variable `x <> x` pass adds two source-derived linked result/expected
role pairs and one fact-free pre-desugaring `Checked` inequality. The two
expected-type slots come from the checker-owned inequality API and the real
reserve binding/use producer comes from task 119; task 107 remains a partial
numeral inequality bridge without expected types. Production invariants and a
real sidecar test guard the slice. Inequality desugaring/truth/facts, implicit
closure, theorem acceptance, proof, CoreIr, ControlFlowIr, VC, and broader
extraction remain deferred.

Task122 current-state override for chapters 03, 04, 13, 14, and 16: the exact
`reserve x for set; theorem ReservedVariableTypeAssertionPayloadBoundary: x is set;`
pass combines task 119's real reserve lookup/result producer with task 109's
formula-side asserted builtin-`set` source node. `TermFormulaChecker` now
requires one ready subject and one asserted type, accepts normalized semantic
identity as the reflexive admissibility case, and keeps known non-identical
types `Partial` on the external reachability payload gap instead of inventing
widening. Production invariants independently preserve the two
pre-normalization source anchors and require one `Inferred` variable plus one
fact-free `Checked` type assertion. General reachability/widening/`qua`,
attributes, truth/facts, implicit closure, theorem acceptance, proof, CoreIr,
ControlFlowIr, VC, and broader extraction remain deferred.

Task123 current-state override for chapters 04, 13, 14, and 16: the exact
`reserve x, y for set; theorem DistinctReservedVariableEqualityPayloadBoundary: x = y;`
pass combines the real multi-reserve producer with task 119's equality
consumer. Source-order lookup preserves distinct checker binding identities
for `x` and `y` while both source bindings retain the shared written builtin
`set` range; operand-specific result/expected roles reach two `Inferred`
variables and one fact-free type/well-formedness `Checked` equality. Production
invariants, a near-miss matrix, and a real frontend/resolver sidecar guard the
exact slice. This closes the task's `test_gap`, `source_drift`, and
`design_drift` only for exact distinct-binding equality. Implicit
universal-closure/order nodes, truth/facts, theorem acceptance, broader
reserved-variable formulas, proof, CoreIr, ControlFlowIr, and VC remain
deferred.

Task124 current-state override for chapters 04, 13, 14, and 16: the exact
`reserve x for set; reserve y for set; theorem MultipleReserveDeclarationEqualityPayloadBoundary: x = y;`
pass reuses the real two-declaration reserve producer and task 119's equality
consumer. Source-order lookup preserves `BindingId(0)` / `BindingId(1)`, while
four operand-specific pre-normalization result/expected inputs retain the two
distinct written builtin `set` ranges. The checker deterministically interns
their identical semantics to one normalized type whose canonical source is the
earliest range; production validation checks both original provenances rather
than fabricating duplicate semantic nodes. An exact near-miss matrix and real
frontend/resolver sidecar guard this `test_gap`, `source_drift`, and
`design_drift` repair. This does not change the chapters' partial status.
Implicit universal-closure/order nodes, truth/facts, theorem acceptance,
broader reserved-variable formulas, proof, CoreIr, ControlFlowIr, and VC remain
deferred.

Task125 current-state override for chapters 03, 04, 13, 14, and 16: the exact
`reserve x for object; reserve y for set; theorem HeterogeneousReserveMembershipPayloadBoundary: x in y;`
pass combines the real mixed-builtin two-declaration reserve producer with task
120's membership consumer. The left result retains its written builtin `object`
range, while the right result and sole expected input retain the written builtin
`set` range. Production validation requires two normalized identities, with the
right roles sharing `set`, the left `object` remaining distinct, and both
identities keeping deterministic per-type source representatives. An exact
near-miss matrix and real frontend/resolver sidecar guard this `test_gap`,
`source_drift`, and `design_drift` repair. This does not change the chapters'
partial status. Membership truth/facts, object/set coercion evidence, implicit
closure/order, theorem acceptance, broader formulas, proof, CoreIr,
ControlFlowIr, and VC remain deferred.

Task126 current-state override for chapters 04, 07, 13, 14, and 16: the exact
`definition mode LocalModeFormulaDef: LocalModeFormula is set; end; reserve x for LocalModeFormula; theorem LocalModeReservedVariableEqualityPayloadBoundary: x = x;`
pass combines task 55's real AST-derived direct bare-set mode-expansion
producer with task 119's reserved-variable equality consumer. Four raw
result/expected inputs retain the written local-mode symbol and reserve range,
while normalization consumes the expansion table and interns one builtin-`set`
identity whose canonical source is the real expansion RHS. Production
validation, an exact near-miss matrix, and a real frontend/resolver sidecar
guard this `test_gap`, `source_drift`, and `design_drift` repair. This does not
change the chapters' partial status or credit mode-definition declaration
checking/acceptance, inhabitation evidence, implicit closure/order,
truth/facts, theorem acceptance, broader/chained/imported mode formulas, proof,
CoreIr, ControlFlowIr, or VC; those remain deferred.

Task127 current-state override for chapters 04, 07, 13, 14, and 16: the exact
`definition mode BaseModeFormulaDef: BaseModeFormula is set; end; definition mode ChainModeFormulaDef: ChainModeFormula is BaseModeFormula; end; reserve x for ChainModeFormula; theorem ChainedLocalModeReservedVariableEqualityPayloadBoundary: x = x;`
pass combines task 56's real AST-derived one-edge mode-expansion-chain producer
with task 126's equality consumer. Four raw result/expected inputs retain the
written outer-mode symbol and reserve range, while recursive normalization
consumes both real expansion entries and interns one builtin-`set` identity
whose canonical source is the terminal `set` RHS. Production validation,
invalid-link corruption, an exact near-miss matrix, and a real
frontend/resolver sidecar guard this `test_gap`, `source_drift`, and
`design_drift` repair. This does not change the chapters' partial status or
credit mode-definition declaration checking/acceptance, inhabitation evidence,
object terminals, longer-chain formulas, closure/order, truth/facts, theorem
acceptance, proof, CoreIr, ControlFlowIr, or VC; those remain deferred.

Task128 current-state override for chapters 03, 04, 07, 13, 14, and 16: the
exact
`definition mode LocalObjectModeDef: LocalObjectMode is object; end; reserve x for LocalObjectMode; theorem LocalObjectModeReservedVariableEqualityPayloadBoundary: x = x;`
pass combines task 55's real AST-derived direct bare-object mode-expansion
producer with task 126's equality consumer. Four raw result/expected inputs
retain the written local object-mode symbol and reserve range, while
normalization consumes the real expansion and interns one builtin-`object`
identity whose canonical source is the real expansion RHS. Production
validation, invalid-expansion corruption, an exact near-miss matrix, and a real
frontend/resolver sidecar guard this `test_gap`, `source_drift`, and
`design_drift` repair. This does not change the chapters' partial status or
credit mode-definition declaration checking/acceptance, inhabitation evidence,
broader object-mode formulas, implicit closure/order, truth/facts, theorem
acceptance, proof, CoreIr, ControlFlowIr, or VC; those remain deferred.

Task129 current-state override for chapters 03, 04, 07, 13, 14, and 16: the
exact `ChainObjectMode -> BaseObjectMode -> object` reserved-variable equality
pass combines task 56's real one-edge expansion producer with tasks 127/128's
recursive equality and builtin-object consumers. Four raw outer-mode inputs
survive while both expansions normalize to one builtin-object identity anchored
at the terminal RHS. Production invariants, invalid-link corruption, near
misses, and a real sidecar guard this `test_gap`, `source_drift`, and
`design_drift` repair. The chapters remain partial; declaration
acceptance/inhabitation, longer chains, closure/order, truth/facts, theorem
acceptance, proof, CoreIr, ControlFlowIr, and VC remain deferred.

Task64 addendum for chapters `03.type_system.md`, `06.attributes.md`,
`07.modes.md`, and `17.clusters_and_registrations.md`: checker task 64 adds
active source-derived diagnostic coverage for an attributed local-mode reserve
head whose one-edge dependency chain ends in a bare builtin RHS. The active
runner consumes the real expansion payloads and reserve-head attribute, then
stops at the missing attributed-type evidence query instead of reporting a
missing mode-expansion payload. This does not change the partial status of
those chapters; imported/argument-bearing/structure-RHS/attributed-RHS/deeper
chains, positive attributed-type acceptance, CoreIr, ControlFlowIr, VC, and
proof payloads remain deferred.

Task65 addendum for chapters `03.type_system.md`, `05.structures.md`,
`06.attributes.md`, `07.modes.md`, and `17.clusters_and_registrations.md`:
checker task 65 adds active source-derived diagnostic coverage for an
attributed local-mode reserve head whose one-edge dependency chain ends in a
same-module local structure RHS. The active runner consumes the real expansion
payloads and reserve-head attribute, then stops at the missing structure
base-shape/constructor-witness and full attributed-type evidence query instead
of reporting a missing mode-expansion payload. This does not change the
partial status of those chapters; attributed-builtin terminal dependencies,
mixed uses, attributed dependencies, imported/ambiguous/argument-bearing,
contextual/parameterized/recovered or deeper chains, positive
structure/attributed-type acceptance, CoreIr, ControlFlowIr, VC, and proof
payloads remain deferred.

Task66 addendum for chapters `03.type_system.md`, `06.attributes.md`,
`07.modes.md`, and `17.clusters_and_registrations.md`: checker task 66 adds
active source-derived diagnostic coverage for an attributed local-mode reserve
head whose one-edge dependency chain ends in an attributed builtin RHS. The
active runner consumes the real expansion payloads, reserve-head attribute,
and terminal RHS attributes, then stops at the missing full attributed-type
evidence query instead of reporting a missing mode-expansion payload. This
does not change the partial status of those chapters; deeper chains, mixed
uses, attributed dependencies, imported/ambiguous/argument-bearing/contextual/
parameterized/recovered definitions, positive attributed-type acceptance,
CoreIr, ControlFlowIr, VC, and proof payloads remain deferred.

Task67 addendum for chapters `03.type_system.md`, `05.structures.md`, and
`06.attributes.md`: checker task 67 adds active source-derived diagnostic
boundary coverage for a same-module structure-qualified attribute reference in
a reserve type expression. The active runner proves the real `.miz` path is
parser/resolver executable, but leaves `LocalStruct.marked LocalStruct` on the
source-to-checker payload extraction gap because checker-owned attribute
payloads do not yet preserve real structure-qualifier or attribute-owner
provenance. This does not change the partial status of those chapters;
qualified attribute payloads, positive attributed-structure acceptance,
existential evidence, CoreIr, ControlFlowIr, VC, and proof payloads remain
deferred.

Task68 addendum for chapter `03.type_system.md`: checker task 68 adds active
source-derived diagnostic boundary coverage for a reserve type expression whose
same-module local mode head carries `of` type arguments, such as
`Element of a`. Chapter 3 and parser coverage already define this syntax; this
addendum does not claim argument-bearing mode semantics. The active runner
proves the source is parser/resolver executable but leaves it on the
source-to-checker payload extraction gap because checker-owned reserve payloads
do not yet preserve real type-argument or term-argument provenance. This does
not change the partial status of the chapter; mode/structure arguments, arity
matching, term payloads, mode expansion, positive type elaboration, CoreIr,
ControlFlowIr, VC, and proof payloads remain deferred.

Task69 addendum for chapters `03.type_system.md` and `05.structures.md`:
checker task 69 adds active source-derived diagnostic boundary coverage for a
reserve type expression whose same-module local structure declaration uses an
`of` parameter surface and whose reserve head carries `of` type arguments, such
as `LocalStruct of a`. Chapters 3 and 5 plus parser coverage already define
this syntax; this addendum does not claim argument-bearing structure semantics.
The active runner proves the source is parser/resolver executable but leaves it
on the source-to-checker payload extraction gap because checker-owned reserve
payloads do not yet preserve real type-argument or term-argument provenance.
This does not change the partial status of those chapters; mode/structure
arguments, arity matching, term payloads, base-shape/constructor-witness
evidence, positive structure type elaboration, CoreIr, ControlFlowIr, VC, and
proof payloads remain deferred.

Task70 addendum for chapters `03.type_system.md` and `07.modes.md`: checker
task 70 adds active source-derived diagnostic boundary coverage for source
containing a same-module bracket-parameter local mode declaration and a
bracket-form reserve type head, such as `Family[set]`. Chapters 3 and 7 plus
parser coverage already define this syntax; this addendum does not claim
bracket-form mode application semantics. The active runner proves the source
is parser/resolver executable but leaves it on the source-to-checker payload
extraction gap before bracket type-argument payload extraction or mode-head
resolution because checker-owned reserve payloads do not yet preserve real
bracket type-argument or `qua`-argument provenance. This does not change the
partial status of either chapter; bracket `type_arg_list` payloads,
`qua`-argument lowering, arity matching, mode expansion, positive type
elaboration, CoreIr, ControlFlowIr, VC, and proof payloads remain deferred.

Task71 addendum for chapters `03.type_system.md` and `05.structures.md`:
checker task 71 adds active source-derived diagnostic boundary coverage for
source containing a same-module bracket-parameter local structure declaration
and a bracket-form reserve type head, such as `LocalStruct[set]`. Chapters 3
and 5 plus parser coverage already define this syntax; this addendum does not
claim bracket-form structure application semantics. The active runner proves
the source is parser/resolver executable but leaves it on the source-to-checker
payload extraction gap before bracket type-argument payload extraction or
structure-head resolution because checker-owned reserve payloads do not yet
preserve real bracket type-argument or `qua`-argument provenance. This does
not change the partial status of either chapter; bracket `type_arg_list`
payloads, `qua`-argument lowering, arity matching, base-shape or
constructor-witness evidence, positive structure type elaboration, CoreIr,
ControlFlowIr, VC, and proof payloads remain deferred.

Task72/73/74 addendum for chapters `03.type_system.md`, `07.modes.md`, and
`17.clusters_and_registrations.md`: checker task 72 adds active
source-derived pass coverage for same-module no-argument local-mode expansion
chains with two mode-to-mode edges whose terminal RHS is builtin `set` /
`object`, and task 73 promotes the same source-derived seam to three
mode-to-mode edges. Task 74 then replaces the temporary depth guard with
AST-bounded structural pass coverage for bare same-module no-argument
local-mode chains whose terminal RHS is exactly builtin `set` / `object`.
The active runner derives every expansion from the same `.miz` `SurfaceAst` and
continues only the supported bare builtin-terminal structural pass cases through
the existing `TypedAst`, `ResolvedTypedAst`, summary-readiness, and binder-only
context path. This does not change the partial status of these chapters;
imported/argument-bearing/attributed/parameterized/contextual/ambiguous/cyclic/
forward-reference acceptance forbidden by active-range rules, chains that violate the task-74 structural
guards, structure or attributed-builtin terminals beyond the existing one-edge
diagnostic slices, CoreIr, ControlFlowIr, VC, and proof payloads remain
deferred.

Task75 addendum for chapters `02.lexical_structure.md`, `07.modes.md`, and
`11.symbol_management.md`: checker task 75 adds active source-derived
diagnostic coverage for a reserve head that names a later same-module local
mode declaration. The active runner observes the lower-stage
`type_elaboration.lower_stage.frontend:malformed_type_expression` detail before
checker handoff, so this credits only the active-range/no-forward-reference
boundary and not checker `ModeExpansion` production.

Task76 addendum for chapters `02.lexical_structure.md`, `05.structures.md`, and
`11.symbol_management.md`: checker task 76 adds active source-derived
diagnostic coverage for a reserve head that names a later same-module local
structure declaration. The active runner observes the same lower-stage
`type_elaboration.lower_stage.frontend:malformed_type_expression` detail before
checker handoff, so this credits only the active-range/no-forward-reference
boundary and the structure syntax/type-head surface, not checker structure
type-head payload extraction, base-shape evidence, constructor-witness
evidence, CoreIr, ControlFlowIr, VC, or proof payload promotion.

Task77 addendum for chapters `02.lexical_structure.md`, `06.attributes.md`, and
`11.symbol_management.md`: checker task 77 adds active source-derived
diagnostic coverage for a reserve type expression that uses a later same-module
local attribute declaration. The active runner observes the same lower-stage
`type_elaboration.lower_stage.frontend:malformed_type_expression` detail before
checker handoff, so this credits only the active-range/no-forward-reference
boundary and the attribute syntax/use surface, not checker `AttributeInput`
payload extraction, attributed-type evidence queries, CoreIr, ControlFlowIr,
VC, or proof payload promotion.

Task78 addendum for chapters `03.type_system.md`, `05.structures.md`,
`11.symbol_management.md`, and `12.modules_and_namespaces.md`: checker task 78
originally added active source-derived diagnostic coverage for the documented
`parser.type_fixtures` imported structure `R` reserve head on
`type_elaboration.external_dependency.ast_payload_extraction`. Task83
supersedes that documented `R` portion and task97 supersedes the documented
`TypeCaseStruct` portion by carrying imported structure provenance/type-head
payloads to the checker evidence-query diagnostic. Broader imported-structure
reserve heads outside the task-83 `R` and task-97 `TypeCaseStruct` bridges
remain deferred. This credits no real
imported module AST extraction, base-shape evidence, constructor-witness
evidence, positive structure type elaboration, CoreIr, ControlFlowIr, VC, or
proof payload promotion.

Task83/task97 addendum for chapters `03.type_system.md`, `05.structures.md`,
`11.symbol_management.md`, and `12.modules_and_namespaces.md`: checker tasks 83
and 97 add active source-derived diagnostic coverage for reserve heads whose
imported structures `R` and `TypeCaseStruct` come from the documented
`parser.type_fixtures` import summary. The active runner observes
`type_elaboration.checker.checker.declaration.deferred.evidence_query`, so this
credits only real imported structure provenance and structure type-head payload
extraction for those fixtures, not imported module AST extraction, base-shape
evidence, constructor-witness evidence, positive structure type elaboration,
CoreIr, ControlFlowIr, VC, or proof payload promotion.

Task79 addendum for chapters `03.type_system.md`, `07.modes.md`,
`11.symbol_management.md`, and `12.modules_and_namespaces.md`: checker task 79
adds active source-derived diagnostic coverage for a reserve head whose mode
symbol comes from the documented `parser.type_fixtures` import summary. Task82
supersedes that statement only for the documented `TypeCaseMode` fixture by
crediting imported mode provenance/type-head extraction; task79 coverage outside
that bridge still observes `type_elaboration.external_dependency.ast_payload_extraction`.
This credits only the imported-mode reserve-head extraction-gap boundary, not
real imported module AST extraction, `ModeExpansion` payloads, positive mode
elaboration, CoreIr, ControlFlowIr, VC, or proof payload promotion.

Task80 addendum for chapters `03.type_system.md`, `06.attributes.md`,
`11.symbol_management.md`, and `12.modules_and_namespaces.md`: checker task 80
historically added active source-derived diagnostic coverage for reserve types
whose attribute symbols come from the documented `parser.type_fixtures` import
summary and observed `type_elaboration.external_dependency.ast_payload_extraction`.
Task84, Task85, and Task116 supersede that boundary only in narrow slices:
Task84 for `TypeCaseAttr` imported provenance and `AttributeInput` payload
extraction, Task85 for the negative `empty`/builtin-`set` fixture, and Task116
for the positive `empty`/builtin-`set` fixture. Broader imported attributes
outside these bridges still credit only the extraction-gap boundary until
source-derived fixtures and payload producers exist. Task80 keeps the active
boundary sidecar for `non empty object` on that extraction-gap row; it does not
credit the `empty`/builtin-`set` bridges. This does not treat the import summary as real
imported module AST extraction and does not credit attributed-type evidence,
positive attributed type elaboration, CoreIr, ControlFlowIr, VC, or proof
payload promotion.

Task84 addendum for chapters `03.type_system.md`, `06.attributes.md`,
`11.symbol_management.md`, and `12.modules_and_namespaces.md`: checker task 84
promotes the imported-attribute reserve boundary from task 80 just far enough
for the active runner to pass the documented `parser.type_fixtures`
`TypeCaseAttr` `ImportedSource` attribute symbol as a checker `AttributeInput`
on builtin `set`. The active runner observes
`type_elaboration.checker.checker.declaration.deferred.evidence_query`, so this
credits imported attribute provenance and no-argument `AttributeInput` payload
extraction only, not imported module AST extraction, attributed-type
existential/evidence payloads, positive imported attributed type elaboration,
generic imported attributes such as `empty`, structure-qualified attribute
owner provenance, attribute arguments, CoreIr, ControlFlowIr, VC, or proof
payload promotion.

Task85 addendum for chapters `03.type_system.md`, `06.attributes.md`,
`11.symbol_management.md`, and `12.modules_and_namespaces.md`: checker task 85
further narrows the imported-attribute reserve boundary from task 80 only for
the existing documented `non empty set` fixture. The active runner passes the
`parser.type_fixtures` imported `empty` `ImportedSource` attribute symbol as a
negative checker `AttributeInput` on builtin `set` and observes
`type_elaboration.checker.checker.declaration.deferred.evidence_query`, so this
credits imported attribute provenance and no-argument negative
`AttributeInput` payload extraction only for that fixture. It does not credit
imported module AST extraction, attributed-type existential/evidence payloads,
imported `empty` on non-`set` heads, broader imported
attributes, structure-qualified attribute owner provenance, attribute
arguments, CoreIr, ControlFlowIr, VC, or proof payload promotion.
Task116 addendum for chapters `03.type_system.md`, `06.attributes.md`,
`11.symbol_management.md`, and `12.modules_and_namespaces.md`: checker task 116
further narrows the imported-attribute reserve boundary from task 80 only for
the existing documented `empty set` fixture. The active runner passes the
`parser.type_fixtures` imported `empty` `ImportedSource` attribute symbol as a
positive checker `AttributeInput` on builtin `set` and observes
`type_elaboration.checker.checker.declaration.deferred.evidence_query`, so this
credits imported attribute provenance and no-argument positive
`AttributeInput` payload extraction only for that fixture. It does not credit
imported module AST extraction, attributed-type existential/evidence payloads,
positive attributed-type acceptance, imported `empty` on non-`set` heads,
broader imported attributes, structure-qualified attribute owner provenance,
attribute arguments, CoreIr, ControlFlowIr, VC, or proof payload promotion.
The accompanying `non empty object` sidecar remains external-gap coverage under
the Task80 row.

Task86 / Task115 / Task117 addendum for chapters `14.formulas.md` and
`16.theorems_and_proofs.md`: checker task 86 adds active source-derived
diagnostic coverage for a formula-only theorem source after parser and
resolver execution. Checker task 115 supersedes only the exact unrecovered
`theorem FormulaPayloadBoundary: thesis;` source by passing the source-derived
`thesis` formula constant site/range as a checker recovery `FormulaInput`.
Checker task 117 supersedes that recovery marker by passing a real
`FormulaKind::Thesis` checker payload for that exact fixture, so the active
runner now observes
`type_elaboration.checker.checker.formula.external.formula_payload`; non-exact
formula-only theorem shapes remain on
`type_elaboration.external_dependency.ast_payload_extraction`. This credits
only exact formula constant kind handoff, not formula constant semantics,
child-formula graph payloads, checker theorem/formula semantic
checking, local proof context, recorded facts, theorem acceptance,
`formula_statement` runner support, CoreIr, ControlFlowIr, VC, or proof
payload promotion.

Task106 addendum for chapters `13.term_expression.md`, `14.formulas.md`, and
`16.theorems_and_proofs.md`: checker task 106 supersedes the task-87 generic
boundary for the exact source-derived theorem equality formula
`theorem TermFormulaPayloadBoundary: 1 = 1;`. The active runner observes real
checker `TermInput` and equality `FormulaInput` payloads derived from the source
AST, then fails closed on missing numeric type payloads and partial formula
checking. This credits only the narrow builtin equality checker handoff. It does
not credit numeric type payload extraction, equality semantic checking, recorded
facts, theorem acceptance, `formula_statement` runner support, CoreIr,
ControlFlowIr, VC, or proof payload promotion.

Task88 addendum for chapters `14.formulas.md`, `15.statements.md`, and
`16.theorems_and_proofs.md`: checker task 88 adds active source-derived
diagnostic coverage for a theorem proof block with a `thus thesis;` conclusion
after parser and resolver execution. The active runner observes
`type_elaboration.external_dependency.ast_payload_extraction`, so this credits
only the proof-block/proof-skeleton extraction-gap boundary. It does not credit
checker proof skeleton payload extraction, local proof context, formula payload
extraction, recorded facts, theorem acceptance, `formula_statement` runner
support, CoreIr, ControlFlowIr, VC, or proof payload promotion.

Task89 addendum for chapters `14.formulas.md`, `15.statements.md`, and
`16.theorems_and_proofs.md`: checker task 89 adds active source-derived
diagnostic coverage for statement-level proof justifications inside a theorem
proof after parser and resolver execution. The active runner observes
`type_elaboration.external_dependency.ast_payload_extraction`, so this credits
only the statement-proof extraction-gap boundary. It does not credit checker
statement proof payload extraction, nested proof skeleton payload extraction,
local proof context, formula payload extraction, label-reference semantic
checking, recorded facts, theorem acceptance, `formula_statement` runner
support, CoreIr, ControlFlowIr, VC, or proof payload promotion.

Task90 addendum for chapters `09.predicates.md` and `10.functors.md`: checker
task 90 adds active source-derived diagnostic coverage for a definition block
containing predicate and functor definitions after parser and resolver
execution. The active runner observes
`type_elaboration.external_dependency.ast_payload_extraction`, so this credits
only the predicate/functor definition extraction-gap boundary. It does not
credit checker definition declaration payload extraction, definition-local
context, definiens formula/term payload extraction, overload payloads, recorded
facts, `formula_statement` runner support, CoreIr, ControlFlowIr, VC, or proof
payload promotion.

Task91 addendum for chapter `06.attributes.md`: checker task 91 adds active
source-derived diagnostic coverage for an attribute definition after parser and
resolver execution. The active runner observes
`type_elaboration.external_dependency.ast_payload_extraction`, so this credits
only the attribute definition extraction-gap boundary. It does not credit
checker attribute definition declaration payload extraction, definition-local
context, formula-definiens payload extraction, attributed-type evidence,
recorded facts, `formula_statement` runner support, CoreIr, ControlFlowIr, VC,
or proof payload promotion.

Task92 addendum for chapters `05.structures.md` and `07.modes.md`: checker task
92 adds active source-derived diagnostic coverage for structure and mode
definitions after parser and resolver execution. The active runner observes
`type_elaboration.external_dependency.ast_payload_extraction`, so this credits
only the mode/structure definition extraction-gap boundary. It does not credit
checker mode/structure definition declaration payload extraction, mode
expansion, structure base-shape/constructor/selector evidence, definition-local
context, recorded facts, `formula_statement` runner support, CoreIr,
ControlFlowIr, VC, or proof payload promotion.

Task93 addendum for chapters `15.statements.md` and
`16.theorems_and_proofs.md`: checker task 93 adds active source-derived
diagnostic coverage for proof-local `let`, `given`, `consider`, `set`, and
`reconsider` statements inside a theorem proof after parser and resolver
execution. The active runner observes
`type_elaboration.external_dependency.ast_payload_extraction`, so this credits
only the proof-local declaration extraction-gap boundary. It does not credit
checker proof-local declaration payload extraction, inline definition formal/body payload extraction, local proof context,
formula/term payload extraction, RHS term inference, reconsider
coercion/obligation evidence, recorded facts, theorem acceptance,
`formula_statement` runner support, CoreIr, ControlFlowIr, VC, or proof payload
promotion.

Task94 addendum for chapters `15.statements.md` and
`16.theorems_and_proofs.md`: checker task 94 adds active
source-derived diagnostic coverage for proof-local `deffunc` and `defpred`
inline definitions inside a theorem proof after parser and resolver execution.
The active runner observes
`type_elaboration.external_dependency.ast_payload_extraction`, so this credits
only the proof-local inline definition extraction-gap boundary. It does not
credit checker inline definition formal/body payload extraction, local
abbreviation expansion, term/formula body payload extraction, guard evidence,
recorded facts, theorem acceptance, `formula_statement` runner support, CoreIr,
ControlFlowIr, VC, or proof payload promotion.

Task95 addendum for chapter `17.clusters_and_registrations.md`: checker task 95
adds active source-derived diagnostic coverage for a top-level registration
block containing existential and conditional clusters after parser and resolver
execution. The active runner observes
`type_elaboration.external_dependency.ast_payload_extraction`, so this credits
only the registration-block extraction-gap boundary. It does not credit checker
registration-item payload extraction, correctness-condition/proof-obligation
payloads, accepted activation/evidence status, cluster/reduction semantics,
Chapter 17 semantic rows, `formula_statement` or `advanced_semantics` runner
support, CoreIr, ControlFlowIr, VC, or proof payload promotion.

Task96 addendum for chapters `11.symbol_management.md` and
`19.overload_resolution.md`: checker task 96 adds active source-derived
diagnostic coverage for top-level and definition-local synonym/antonym aliases
plus attribute, predicate, and functor redefinition declarations after parser
and resolver execution. The active runner observes
`type_elaboration.external_dependency.ast_payload_extraction`, so this credits
only the redefinition/notation extraction-gap boundary. It does not credit
checker redefinition payload extraction, notation alias relation payloads,
redefinition target inference, coherence proof-obligation payloads, overload
candidate payloads, Chapter 11 alias semantic resolution, Chapter 19
overload/redefinition semantics, `formula_statement` or `advanced_semantics`
runner support, CoreIr, ControlFlowIr, VC, or proof payload promotion.

Task81 addendum for chapters `02.lexical_structure.md`, `03.type_system.md`,
`06.attributes.md`, and `11.symbol_management.md`: checker task 81 adds active
source-derived diagnostic coverage for a same-module parameterized attribute
declared with `param_prefix` syntax and used through `attribute_name(args)` in
a reserve type expression.
The active runner observes
`type_elaboration.external_dependency.ast_payload_extraction`, so this credits
only the argument-bearing local-attribute reserve extraction-gap boundary and
the lexer/parser/resolver producer seam needed to carry that real source
surface to the checker boundary. Resolver coverage is limited to declaration
symbol suffix projection, suffix-based lexical summary export, and prefixed
notation preservation. It does not credit real term-argument provenance, checker
`AttributeInput` argument payload extraction, attributed-type evidence,
positive attributed type elaboration, CoreIr, ControlFlowIr, VC, or proof
payload promotion.

Task82 addendum for chapters `03.type_system.md`, `07.modes.md`,
`11.symbol_management.md`, and `12.modules_and_namespaces.md`: checker task 82
promotes the imported-mode reserve-head boundary from task 79 just far enough
for the active runner to pass the documented `parser.type_fixtures`
`TypeCaseMode` `ImportedSource` mode symbol as a checker type head. The active runner observes
`type_elaboration.checker.checker.type.external.mode_expansion_payload`, so
this credits imported mode provenance and type-head payload extraction only,
not imported module AST extraction, imported mode-definition/module-summary
expansion payloads, arity checking, positive imported mode elaboration, CoreIr,
ControlFlowIr, VC, or proof payload promotion.

Task130 addendum for chapters `04.variables_and_constants.md`, `07.modes.md`,
`13.term_expression.md`, `14.formulas.md`, and
`16.theorems_and_proofs.md`: checker task 130 adds only the exact active
type/well-formedness pass for a reserved identifier with a real direct bare-set
local-mode expansion used in pre-desugaring inequality. Four raw mode-headed
roles normalize through the one AST-derived expansion to a terminal-RHS
builtin-set identity, producing two inferred variables and one fact-free
checked inequality. Coverage remains partial: mode declaration
acceptance/inhabitation, inequality desugaring, implicit closure/order,
truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr, and VC are not
credited.

Task131 addendum for chapters `03.type_system.md`, `04.variables_and_constants.md`,
`07.modes.md`, `13.term_expression.md`, `14.formulas.md`, and
`16.theorems_and_proofs.md`: checker task 131 adds only the exact active
type/well-formedness pass for a reserved identifier with a real direct
bare-object local-object-mode expansion used in pre-desugaring inequality. Four
raw object-mode-headed roles normalize through the one AST-derived expansion to
a terminal-RHS builtin-object identity, producing two inferred variables and
one fact-free checked inequality. The classified changes are `test_gap`,
`source_drift`, and `design_drift`; no specification intent or existing
expectation changes. Coverage remains partial: mode declaration
acceptance/inhabitation, inequality desugaring, implicit closure/order,
truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr, and VC are not
credited.

Task132 addendum for chapters `04.variables_and_constants.md`, `07.modes.md`,
`13.term_expression.md`, `14.formulas.md`, and
`16.theorems_and_proofs.md`: checker task 132 adds only the exact active
type/well-formedness pass for a reserved identifier whose outer local mode
normalizes through two real AST-derived set-terminal expansion links in a
pre-desugaring inequality. Four raw outer-mode roles normalize to one
terminal-RHS builtin-set identity, producing two inferred variables and one
fact-free checked inequality. The classified changes are `test_gap`,
`source_drift`, and `design_drift`; no specification intent or existing
expectation changes. Coverage remains partial: mode declaration
acceptance/inhabitation, object-terminal, direct and longer chain formulas,
inequality desugaring, implicit closure/order, truth/facts, theorem acceptance,
proof, CoreIr, ControlFlowIr, and VC are not credited.

Task133 addendum for chapters `03.types.md`, `04.variables_and_constants.md`,
`07.modes.md`, `13.term_expression.md`, `14.formulas.md`, and
`16.theorems_and_proofs.md`: checker task 133 adds only the exact active
type/well-formedness pass for a reserved identifier whose outer local mode
normalizes through two real AST-derived object-terminal expansion links in a
pre-desugaring inequality. Four raw outer-mode roles normalize to one
terminal-RHS builtin-object identity, producing two inferred variables and one
fact-free checked inequality. The classified changes are `test_gap`,
`source_drift`, and `design_drift`; no specification intent or existing
expectation changes. Coverage remains partial: declaration
acceptance/inhabitation, set-terminal, direct and longer chain formulas,
inequality desugaring, implicit closure/order, truth/facts, theorem acceptance, proof, CoreIr,
ControlFlowIr, and VC are not credited.

Task134 addendum for chapters `04.variables_and_constants.md`, `07.modes.md`,
`13.term_expression.md`, `14.formulas.md`, and
`16.theorems_and_proofs.md`: checker task 134 adds only the exact active
type/well-formedness pass for a reserved identifier whose outer local mode
normalizes through three real AST-derived two-edge set-terminal expansion links
in an equality. Four raw outer-mode roles normalize to one terminal-RHS
builtin-set identity, producing two inferred variables and one fact-free
checked equality. The classified changes are `test_gap`, `source_drift`, and
`design_drift`; no specification intent or existing expectation changes.
Coverage remains partial: declaration acceptance/inhabitation, object-terminal,
direct, one-edge and longer chain formulas, implicit closure/order, truth/facts,
theorem acceptance, proof, CoreIr, ControlFlowIr, and VC are not credited.

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
