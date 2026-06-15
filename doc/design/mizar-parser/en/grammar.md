# mizar-parser: Grammar

Status: module skeleton, top-level placeholder dispatch, and concrete import
items implemented through task 6; concrete export/other item grammars planned.

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
`synonym`, and `antonym`. After task 6, `import` is a concrete item only while
the import prelude is still open; later `import` tokens are recovered as
misplaced top-level input.

Consecutive library annotation prefixes beginning with `@[` are retained in the
same placeholder when they are followed by a recognized top-level start.
Malformed annotation parsing and concrete annotation nodes remain deferred to
the annotation grammar tasks. Semicolon-style placeholders scan through nested
`proof ... end` and contextual algorithm/proof blocks, so semicolons inside a
proof body do not split a theorem or lemma item. Contextual formula keywords
such as expression-level `if` and `otherwise` do not affect placeholder block
depth.

This task does not parse export paths, theorem formulas, visibility semantics,
item validity, or symbol identities. `export` remains a placeholder until task
7 replaces it with a concrete item node; non-import declarations remain
placeholder items until their owning grammar tasks land. Token streams that
contain no recognizable top-level item start keep the task-3 compatibility
behavior for the module skeleton: tokens are preserved and the item list is
empty. Such streams remain diagnostic-free only when the earlier recovery pass
also has no findings, as in the legacy minimal token-stream corpus case.
Synthetic block-recovery streams whose first recognized item keyword is nested
under an earlier recovery block opener also keep this compatibility behavior;
ordinary malformed prefixes such as a stray reserved word before a theorem item
still produce `UnexpectedTopLevelToken` recovery.

## Task 6: Import Items

Production inventory:

```ebnf
import_stmt          ::= "import" module_alias_decl
                         { "," module_alias_decl } ";" ;
module_alias_decl    ::= module_path [ "as" module_identifier ]
                       | module_branch_import ;
module_branch_import ::= module_path ".{"
                         module_identifier { "," module_identifier } "}" ;
```

The parser emits one `ImportItem` per `import_stmt` while the import prelude is
open. For well-formed imports, the item children are the `import` token, one or
more import declaration nodes separated by comma tokens, and the terminating
semicolon token. Simple imports and aliases emit `ImportAliasDecl` with a
`ModulePath` child, an optional `as` token, and an optional alias `PathSegment`.
Branch imports emit `ModuleBranchImport` with the base `ModulePath`, the `.{`
token, branch identifier `PathSegment` children separated by comma tokens, and
`}`.

Import paths use the task-4 shared `ModulePath`, `RelativePrefix`, and
`PathSegment` nodes. The parser preserves relative prefixes and branch
components syntactically, but it does not resolve modules, check alias
collisions, inspect exports, assign symbol identities, or decide visibility.

Once a non-import top-level item has been parsed, the import prelude closes.
Later `import` tokens are recovered with `UnexpectedTopLevelToken`,
`SkippedToken` recovery, and skipped-range trivia through the semicolon or next
top-level boundary. Missing import semicolons use `MissingSemicolon`.
Malformed import-internal syntax that can continue at the current statement
boundary, such as `as` without an alias or a branch import missing `}`, uses
`MalformedImport`. When malformed source before the semicolon is consumed, the
parser owns it with a `SkippedToken` recovery node and skipped-range trivia
inside the import item or its malformed declaration. Recovery shapes may
therefore include an `ImportItem` with no declaration after `import`, a trailing
comma without a following declaration, an `ImportAliasDecl` without an alias
segment, or a `ModuleBranchImport` without branch segments or `}`.

## Public Enum Compatibility

`ParserTokenKind` is `#[non_exhaustive]` for downstream crates. The parser token
transfer vocabulary can grow as parser-facing lexing contexts gain additional
token classes, and downstream consumers should keep wildcard fallback arms.
Matches inside `mizar-parser` remain exhaustive so newly added token kinds force
local parser updates.
