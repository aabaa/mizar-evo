# mizar-syntax Task 6: Grammar Consistency Audit

> Canonical language: English. Japanese companion: [../ja/grammar_audit.md](../ja/grammar_audit.md).

Status: completed for the Task 6 grammar consistency gate on 2026-06-12.

## Scope

This audit treats [Appendix A](../../../spec/en/appendix_a.grammar_summary.md)
as the parser-facing canonical grammar summary and checks it against the
chapter-local syntax in Chapters 2-21. It deliberately stops before AST node
design. Findings are classified so Task 7 can build a parse-only acceptance
matrix and fixture plan without freezing unresolved grammar drift into
`SurfaceAst` node kinds.

## Method

- Extracted Appendix A EBNF productions and checked undefined nonterminals,
  duplicate definitions, reachability from `compilation_unit`, and direct left
  recursion outside the documented precedence parsers.
- Compared Appendix A production names and right-hand sides with chapter-local
  EBNF in Chapters 2-21. Chapters 8 and 19 are semantic companion chapters;
  their EBNF restates forms owned by Chapters 3, 13, 15, and 18 and is treated
  as compatibility wording unless called out below.
- Compared quoted grammar terminals with the reserved word and reserved special
  symbol lists in Appendix A and Chapter 2.
- Classified each finding as "fix before AST design", "semantic-only", or
  "defer to Task 7 / parser task".

## Mechanical Results

After the Task 6 spec fixes in Appendix A:

| Check | Result |
|---|---|
| Undefined nonterminals | No findings. `character` is now defined for comment productions. |
| Duplicate production definitions | No findings. |
| Direct left recursion | No findings outside the documented `term_expression` and `formula` precedence parsers. |
| Reserved word drift | `step` was missing while used by `for_range_stmt`; it is now reserved in Appendix A and Chapter 2. |
| Reserved special symbol drift | Chapter 2 now includes `..` for relative imports and dot-compound priority. `@` is clarified as an annotation marker rather than a standalone reserved-symbol token. |
| Contextual spelling drift | Appendix A now states that solver/annotation option spellings such as `max_axioms`, `steps`, and `timeout` are contextual, not reserved identifiers. `nest` remains globally reserved by the current lexical contract. |
| Semantic companion EBNF drift | Chapters 8 and 19 restate `qua`, `reconsider`, and overload-resolution helper forms. No parser-facing production change was needed; Appendix A remains the normalized source. |

The remaining productions that are not reached from `compilation_unit` are
intentional helpers or tracked follow-ups:

- Lexical/trivia helpers: `whitespace`, `tab`, `newline`, `line_comment`,
  `block_comment`, `doc_comment`, `character`.
- Compatibility aliases: `term`, `expr`, `proof_block`, `symbol_name`,
  `let_decl`, `set_decl`, `reconsider_decl`, `reconsider_target`,
  `qualified_vars`.
- Parser-normalized helper names whose concrete syntax is reached through
  another production: `field_access` through `term_postfix`,
  `qua_expression` through `term_expression`, `pick_expr` through
  `choice_expression`, and `mode_application` through `type_expression`.
- Open follow-ups recorded below: `scheme_app`, `scheme_name`,
  `functor_loci`, `param_name`, `type_arg_list`, `type_arg`, and `qua_arg`.

## Fix Before AST Design

These issues either were fixed in Task 6 or must be resolved before the
corresponding AST node vocabulary is frozen.

| ID | Finding | Status |
|---|---|---|
| G-AUD-001 | `builtin_pred ::= "in" | "=" | "<>"` existed but was not connected to formula parsing, so built-in membership/equality/inequality formulas were not reachable. | Fixed in Appendix A and Chapter 14 with a separate `builtin_predicate_application`, while `pred_pattern` still uses `def_predicate_symbol` so primitive predicates cannot be defined or redefined by grammar alone. Chapter 9 examples now avoid defining `in` or `=` as user predicates. Task 7 should include `x in X`, `x = y`, and `x <> y` formula fixtures plus negative fixtures for `pred`/`redefine pred` attempts using `in`, `=`, and `<>`. |
| G-AUD-002 | Annotation productions were unreachable from module items, proof statements, and algorithm statements. | Fixed in Appendix A with `annotated_declaration`, `annotated_statement`, `annotated_algo_statement`, annotated registration/template content, and annotated theorem items inside `claim_block`. Context validity remains semantic/diagnostic. |
| G-AUD-003 | Token lists drifted: `step` was used by range loops but not reserved; Chapter 2 lacked `..` while imports use it; standalone `@` needed clearer treatment. | Fixed in Appendix A, Chapter 2, and the lexer implementation track, including the dot-compound priority list. `@` is an annotation marker, not a standalone reserved-symbol token. The `mizar-lexer`/`mizar-session` tables, lexical fixtures, and traceability entries have been synchronized for `step` and `..`, so the affected lexical table and dot-disambiguation requirements in `tests/coverage/spec_trace.toml` are covered. |
| G-AUD-003a | Option and solver spellings such as `max_axioms`, `steps`, and `timeout` appear as fixed literal spellings but should not become globally reserved words. | Fixed in Appendix A by documenting them as contextual spellings. `nest` remains reserved intentionally under the current lexical contract and can be revisited only with a lexer-facing language change. |
| G-AUD-004 | `claim_block` appeared both as a top-level declaration and as an `algo_statement`, while Chapter 20 describes and exemplifies it as a top-level block about an algorithm's snapshots. | Fixed in Appendix A by keeping top-level `claim_block` and removing it from `algo_statement`. |
| G-AUD-005 | `attribute_assertion` used nullable `attribute_chain`, which allowed an empty `x is` shape. | Fixed in Appendix A with non-empty `attribute_test_chain`. |
| G-AUD-006 | `definition_block` and `template_def` both start with `definition ... end;`, and their contents overlap. Parser dispatch cannot rely on the opening keyword alone. | Open before definition/template AST nodes. Task 7 should include ordinary definition blocks, template definitions with `let T be type`, theorem-bearing templates, and ambiguous `let x be T` cases. Owner: parser definition/template tasks and mizar-syntax node vocabulary tasks 15-16. |
| G-AUD-007 | `type_head ::= radix_type | mode_type` is syntactically ambiguous because both are `qualified_symbol [ type_args ]`; the category distinction depends on the active symbol table. | Open before type AST nodes. Either normalize to a generic syntactic type head and defer category resolution, or document a parser lookup boundary. Owner: parser type-expression task and mizar-syntax task 10. |
| G-AUD-008 | Dot roles remain intentionally unresolved at parse time: selector access/update, namespace separation, and active user symbols can share the same surface token. | Open but already tracked in the mizar-syntax todo. Task 7 must include dot-chain fixtures for imports, references, qualified symbols, selector access, lvalues, and active `.` user-symbol cases. |
| G-AUD-009 | `type_assertion` and `attribute_assertion` shared the `term_expression "is" ...` boundary and both tails can start with active-lexicon symbols, so separate parse-time AST shapes would force semantic knowledge into the parser. | Fixed in Appendix A, Chapter 14, and parser/syntax TODOs by replacing the split atomic alternatives with generic `is_assertion`. Resolution later classifies it as type assertion or attribute assertion. Task 7 should include ambiguous `x is T`, `x is non empty`, and qualified attribute/type examples. |
| G-AUD-010 | `compact_statement` and zero-step `iterative_equality` overlap for a justified equality such as `x = y by A;`, because it can be parsed as a justified formula or as an iterative equality with no `.=` continuation. | Open before statement AST nodes. Parser design should either dispatch to `compact_statement` unless a `.=` continuation follows, or preserve a generic justified-equality surface until statement AST design decides. Task 7 should include `x = y by A;`, `x = y by A .= z by B;`, and label/`then` variants. |

## Semantic-Only Accepted

The parser should preserve source shape and avoid deciding these issues:

- Whether a `qualified_symbol` names a mode, struct, predicate, functor,
  attribute, or theorem label, except where the surrounding keyword fixes the
  syntactic role.
- Whether an annotation name is known, whether it is allowed at the current
  attachment site, and whether its arguments satisfy registry-specific rules.
- Visibility defaults and public/private export effects.
- Type well-formedness, sethood, registration closure, proof obligations,
  template guard satisfaction, and algorithm termination obligations.
- Import resolution, cyclic import rejection, and fully qualified symbol
  identity.
- Chapter 8 and Chapter 19 companion EBNF that restates `qua`, `reconsider`,
  and overload-resolution helper forms for semantic exposition. Parser ownership
  stays with the normalized productions in Appendix A and their owning chapters.

## Deferred To Task 7

Task 7 should turn the following into parse-only acceptance matrix rows before
AST snapshots are designed:

- Positive and negative fixtures for built-in predicates, including negative
  `pred`/`redefine pred` attempts for `in`, `=`, and `<>`; positive and
  negative fixtures for generic `is_assertion` forms that later resolve as type
  or attribute assertions, including attribute arguments.
- Annotation attachment fixtures at module level, inside definitions,
  registrations, proofs, algorithms, and claim blocks.
- Definition/template ambiguity fixtures, including predicate and functor
  template parameters.
- `scheme_app` surface examples. The current grammar reaches schema use through
  `simple_justification` plus `reference [ template_args ]`; the separate
  `scheme_app` helper is a compatibility alias and should either be removed or
  repurposed during fixture planning. Treat the unreachable `functor_loci`
  helper the same way: either remove it as dead chapter-compatibility wording or
  repurpose it with concrete fixtures.
- Parameterized name fixtures for `param_name`, `type_arg_list`, `type_arg`,
  and `qua_arg`.
- Dot-chain fixtures that distinguish namespace paths, selector access/update,
  lvalues, grouped/bulk references, and active user-symbol uses.
- `the type_expression` / `pick_expr`, `qua`, and `with (...)` term boundary
  fixtures.
- Compact statement versus iterative equality boundary fixtures, especially the
  zero-`.=` overlap for `x = y by A;` and the continued form
  `x = y by A .= z by B;`, with label and `then` variants.
- Chapter 8/19 duplicate-EBNF coverage through fixtures rather than separate
  parser productions: parenthesized and unparenthesized `qua`, chained `qua`,
  and `reconsider` forms with the current Appendix A justification boundary.
- Recovery fixtures for missing `end`, missing delimiters, malformed
  annotations, and misplaced `claim` or `import` forms.

## Review-Only Agent Findings

The first review-only pass found six issues. All were addressed in this audit
pass:

- Token drift was overstated as fully resolved. The note now distinguishes
  specification normalization from deferred `mizar-lexer` table/test sync.
- Built-in predicates leaked into predicate definitions. Appendix A now uses
  `def_predicate_symbol` for definitions and `builtin_predicate_application`
  for applications.
- `type_assertion` versus `attribute_assertion` was still an AST-shape
  ambiguity. Appendix A now preserves generic `is_assertion` until resolution.
- `claim` was top-level in Appendix A but still grouped with statement/local
  definition TODOs. The mizar-syntax and mizar-parser TODOs now assign it to
  template/algorithm/annotation nodes and parser task 32.
- `nest` was omitted from the contextual-keyword discussion. The audit now
  records that it remains globally reserved under the current lexical contract.
- `open`/`inherit` were listed as module-item work even though Appendix A does
  not define module-level forms for them. The TODOs now remove that wording and
  keep `open` as theorem status and `inherit` as structure-definition work.

The follow-up review-only pass found three remaining issues. All were addressed:

- Lexer token drift was deferred but not concretely owned. The `mizar-lexer`
  TODO owned and completed `step`/`..` table and fixture synchronization, and
  the affected lexical table coverage entries are now covered.
- Chapter 9 still showed `in` and `=` as predicate definition examples. Those
  examples now avoid primitive built-ins and state that `in`, `=`, and `<>` are
  built-in applications, not `pred`/`redefine pred` symbols.
- Chapter 14 and parser task 13 still exposed split type/attribute formula
  shapes. They now use generic `is_assertion` and keep type/attribute
  classification for resolution.

A later external review found five additional issues. All were addressed or
classified:

- The `compact_statement` / zero-step `iterative_equality` overlap is now
  recorded as G-AUD-010 and added to Task 7 fixture inputs.
- Chapter 2's dot-compound priority list now includes `..`; lexer fixtures and
  dot-disambiguation coverage have caught up in the lexer implementation track.
- The unreachable-production explanation now includes `functor_loci`,
  `scheme_name`, and `reconsider_target`, and no longer lists reachable
  `dq_char`.
- Chapter 14's `attribute_ref` now includes attribute argument lists to match
  Appendix A.
- Chapter 14's quantified-variable helper is renamed to `quantified_vars` to
  avoid colliding with Appendix A's compatibility alias `qualified_vars`.

## Chapter Drift Notes

Many chapter-local EBNF blocks still contain older helper names or narrower
forms, for example chapter-local `identifier` names where Appendix A now uses
`qualified_symbol`, older visibility as optional `private`, Chapter 8's
parenthesized-only `qua_expression` and narrower `type_expression` summary, and
older `justification ::= simple_justification | proof` before `by computation`
was added. Chapter 19 intentionally restates `qua_expression` and
`redefine_item` for overload-resolution exposition. These are compatibility
wording unless they are listed above. Appendix A is the parser-facing normalized
form; future chapter edits should avoid renormalizing parser behavior away from
Appendix A.
