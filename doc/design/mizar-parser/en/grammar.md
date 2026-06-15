# mizar-parser: Grammar

Status: module skeleton and top-level placeholder dispatch implemented through
task 5; concrete import/export/item grammars planned.

## Purpose

This module defines parser entry points and the module/item grammar for Mizar Evo.

## Responsibilities

- consume parser-facing token transfer objects and produce `mizar-syntax::SurfaceAst`;
- parse modules, imports, definitions, registrations, statements, proofs, algorithms, annotations, terms, and formulas;
- keep parsing semantic-free: no name resolution, type inference, overload selection, or proof-obligation generation.

Current behavior:

- the crate-root public API (`parse`, `ParseRequest`, `ParserToken`,
  `ParseOutput`, and related transfer enums/entries) remains reachable at the
  original `mizar_parser::...` paths;
- `grammar` owns the current parser orchestration and syntax-event sink handoff,
  while Pratt expression parsing and recovery policy live in sibling
  implementation modules until later tasks grow the full grammar;
- grammar code emits tokens, ordinary nodes, and recovery nodes through the
  private syntax-event sink and documented `mizar-syntax` builder/accessor API,
  not by depending on rowan storage layout or dense arena indices.

## Task 4: Shared Paths

Production inventory:

```ebnf
module_path       ::= [ relative_prefix ] module_identifier
                      { "." module_identifier } ;
relative_prefix   ::= "." | ".." ;
module_identifier ::= identifier ;

namespace_path    ::= identifier { "." identifier } ;

qualified_symbol  ::= { namespace_segment "." } user_symbol ;
namespace_segment ::= identifier ;
```

`module_path` is the import/export path shape from Chapter 12. It is the only
shared path helper that accepts `relative_prefix`; `namespace_path` is reserved
for citation/reference prefixes and must not accept relative import prefixes.
`qualified_symbol` ends in a parser-facing `user_symbol` token supplied by the
active lexicon, with any preceding namespace segments represented as identifier
segments.

Parser task 4 provides shared helper methods and unit coverage only. It does
not introduce a standalone corpus position because these path forms become
frontend-reachable through later consuming grammar tasks: import items (task 6),
type heads (task 8), terms/formulas, and citations (task 17). The helper emits
`mizar-syntax` task-S-009 path nodes through the syntax-event sink and preserves
dot separators syntactically. It performs no module resolution, namespace
shadowing, symbol identity assignment, citation lookup, or validity checking.

## Task 5: Module Skeleton And Top-Level Dispatch

Production inventory:

```ebnf
compilation_unit   ::= import_prelude export_prelude { annotated_declaration } ;
import_prelude     ::= { import_stmt } ;
export_prelude     ::= { export_stmt } ;
declaration        ::= definition_block
                     | reserve_decl
                     | registration_block
                     | claim_block
                     | [ visibility ] theorem_item
                     | [ visibility ] notation_decl ;
visibility         ::= "private" | "public" ;
theorem_status     ::= "open" | "assumed" | "conditional" ;
theorem_role       ::= "theorem" | "lemma" ;
notation_decl      ::= operator_decl | synonym_def | antonym_def ;
```

Task 5 builds the stable surface skeleton that later item parsers replace with
concrete nodes. The parser emits a `CompilationUnit` node with one `ItemList`
child. The `ItemList` contains source-ordered `PlaceholderItem` nodes for
recognized top-level starts and `SkippedToken` recovery nodes for skipped
top-level input. Recognized starts are `import`, `export`, `definition`,
`reserve`, `registration`, `claim`, `theorem`, `lemma`, theorem-status prefixes
`open` / `assumed` / `conditional`, visibility prefixes `private` / `public`,
and notation starts `infix_operator`, `prefix_operator`, `postfix_operator`,
`synonym`, and `antonym`.

Consecutive library annotation prefixes beginning with `@[` are retained in the
same placeholder when they are followed by a recognized top-level start.
Malformed annotation parsing and concrete annotation nodes remain deferred to
the annotation grammar tasks. Semicolon-style placeholders scan through nested
`proof ... end` and contextual algorithm/proof blocks, so semicolons inside a
proof body do not split a theorem or lemma item. Contextual formula keywords
such as expression-level `if` and `otherwise` do not affect placeholder block
depth.

This task does not parse import aliases, export paths, theorem formulas,
visibility semantics, item validity, or symbol identities. `import` and
`export` are placeholders until tasks 6 and 7 replace them with concrete item
nodes. Token streams that contain no recognizable top-level item start keep the
task-3 compatibility behavior for the module skeleton: tokens are preserved and
the item list is empty. Such streams remain diagnostic-free only when the
earlier recovery pass also has no findings, as in the legacy minimal
token-stream corpus case.
Synthetic block-recovery streams whose first recognized item keyword is nested
under an earlier recovery block opener also keep this compatibility behavior;
ordinary malformed prefixes such as a stray reserved word before a theorem item
still produce `UnexpectedTopLevelToken` recovery.

## Public Enum Compatibility

`ParserTokenKind` is `#[non_exhaustive]` for downstream crates. The parser token
transfer vocabulary can grow as parser-facing lexing contexts gain additional
token classes, and downstream consumers should keep wildcard fallback arms.
Matches inside `mizar-parser` remain exhaustive so newly added token kinds force
local parser updates.
