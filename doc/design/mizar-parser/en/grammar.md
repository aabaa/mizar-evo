# mizar-parser: Grammar

Status: module skeleton, top-level placeholder dispatch, concrete import
items, export items, visibility wrappers, reserve-hosted type expressions, and
reserve-hosted primary terms implemented through task 9; remaining concrete
non-module item grammars planned.

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
- top-level `reserve` items are concrete enough to host syntax-only
  `TypeExpression` trees with attribute chains, generic type heads,
  `of`/`over` `TermExpression` arguments, bracket nested type arguments, and bracket
  `qua_arg` placeholders. Other non-module item grammars remain placeholders.

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
child. The `ItemList` contains source-ordered concrete item nodes,
`PlaceholderItem` nodes for not-yet-concrete recognized top-level starts, and
`SkippedToken` recovery nodes for skipped top-level input. Recognized starts
are `import`, `export`, `definition`,
`reserve`, `registration`, `claim`, `theorem`, `lemma`, theorem-status prefixes
`open` / `assumed` / `conditional`, visibility prefixes `private` / `public`,
and notation starts `infix_operator`, `prefix_operator`, `postfix_operator`,
`synonym`, and `antonym`. After task 6, `import` is a concrete item only while
the import prelude is still open; later `import` tokens are recovered as
misplaced top-level input.

Consecutive library annotation prefixes beginning with `@[` are retained in the
same placeholder when they are followed by a recognized annotated-declaration
start. They do not make `import` or `export` eligible for annotation; an
annotation prefix before an import/export prelude item is recovered as
unexpected top-level input together with that statement. Malformed annotation
parsing and concrete annotation nodes remain deferred to the annotation grammar
tasks. Semicolon-style placeholders scan through nested `proof ... end` and
contextual algorithm/proof blocks, so semicolons inside a proof body do not
split a theorem or lemma item. Contextual formula keywords such as
expression-level `if` and `otherwise` do not affect placeholder block depth.

This task does not parse theorem formulas, visibility semantics, item validity,
or symbol identities. After task 7, `export` and visibility prefixes are
concrete syntax wrappers; non-module declarations remain placeholder items
until their owning grammar tasks land. Token streams that
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

## Task 7: Export And Visibility Items

Production inventory:

```ebnf
export_stmt ::= "export" module_path { "," module_path } ";" ;
visibility  ::= "private" | "public" ;
```

The parser emits one `ExportItem` per `export_stmt` while the export prelude is
open. The import prelude still comes first; once the first non-import item is
seen, imports are closed. Contiguous `export` statements immediately after the
import prelude form the export prelude. The first ordinary declaration closes
the export prelude, and later `export` tokens recover as unexpected top-level
input with `UnexpectedTopLevelToken`, `SkippedToken` recovery, and skipped-range
trivia. Later `import` tokens remain late-import recovery.

For well-formed exports, `ExportItem` children are the `export` token, one or
more `ModulePath` nodes separated by comma tokens, and the terminating
semicolon token. Export paths use the task-4 `ModulePath`, `RelativePrefix`,
and `PathSegment` nodes. The parser preserves relative prefixes and comma
lists syntactically, but it does not resolve modules, inspect imported exports,
build facade summaries, or validate visibility.

Malformed export-internal syntax that can continue at the current statement
boundary uses `MalformedExport`. Examples include a missing path after
`export` or after a comma. Malformed source before the semicolon is owned by a
nested `SkippedToken` recovery node plus skipped-range trivia inside the
`ExportItem`. Missing export semicolons use `MissingSemicolon`.

Top-level visibility is represented only for the Chapter 12 forms that accept
it: theorem items and notation declarations. While those concrete item grammars
are still deferred, the parser emits a `VisibleItem` wrapper whose children are
source ordered: any already-skipped library annotation prefix tokens, one
`VisibilityMarker` wrapping the `private` or `public` token, and the following
target `PlaceholderItem`. Legal target starts are `theorem`, `lemma`, theorem
status plus theorem role (`open`, `assumed`, or `conditional` followed by
`theorem` or `lemma`), and notation starts `infix_operator`, `prefix_operator`,
`postfix_operator`, `synonym`, and `antonym`. Visibility on other top-level
declarations, duplicate visibility markers, and a dangling marker use
`MalformedVisibility`; any malformed tail tokens before the statement
semicolon are skipped inside the single `VisibleItem`. A semicolon-only
dangling marker keeps the semicolon as a direct `VisibleItem` child rather than
creating an empty recovery node. If the invalid target is a block-like
top-level declaration (`definition`, `registration`, or `claim`), the same
recovery owns the malformed target through its matching `end`; the following
semicolon remains the wrapper's statement terminator when present, which keeps
the wrapper from cascading into additional top-level recovery nodes.

## Task 8: Type Expressions

Production inventory:

```ebnf
reserve_decl      ::= "reserve" reserve_segment ";" ;
reserve_segment   ::= identifier_list "for" type_expression ;
identifier_list   ::= identifier { "," identifier } ;

type_expression   ::= attribute_chain type_head ;
type_head         ::= radix_type | mode_type ;

attribute_chain   ::= { [ "non" ] attribute_ref } ;
attribute_ref     ::= [ param_prefix ] [ struct_ref_name "." ] attribute_ref_name
                      [ "(" argument_list ")" ] ;
param_prefix      ::= parameter "-" | "(" parameter_list ")" "-" ;

radix_type        ::= builtin_type | struct_ref_name [ type_args ] ;
mode_type         ::= mode_ref_name [ type_args ] ;
type_args         ::= ( "of" | "over" ) argument_list
                    | "[" type_arg_list "]" ;
type_arg_list     ::= type_arg { "," type_arg } ;
type_arg          ::= type_expression | qua_arg ;
qua_arg           ::= identifier { "qua" radix_type } ;
argument_list     ::= term_expression { "," term_expression } ;

builtin_type      ::= "object" | "set" ;
attribute_ref_name ::= qualified_symbol ;
mode_ref_name     ::= qualified_symbol ;
struct_ref_name   ::= qualified_symbol ;
```

Task 8 makes type expressions executable through top-level `reserve`
declarations. The parser emits `ReserveItem` and `ReserveSegment` only as the
current host for `TypeExpression`; local statement-level `reserve` behavior is
owned by later statement tasks. A well-formed `ReserveItem` owns the `reserve`
token, one `ReserveSegment`, and the terminating semicolon. `ReserveSegment`
owns source-ordered identifiers and comma tokens, the `for` token, and a
`TypeExpression`.

The parser emits `TypeExpression` with an optional non-empty `AttributeChain`
and a required generic `TypeHead`. It does not decide whether a syntactic head
is a radix type, structure, or mode, and it does not decide whether a dotted
attribute spelling contains a structure qualifier or only namespace segments.
When a sequence of user-symbol-shaped references could be split multiple ways,
the parser keeps the rightmost available syntactic type-head candidate as the
`TypeHead` and treats preceding references as attributes. This is a syntax-only
boundary rule, not semantic classification.

`AttributeRef` preserves optional `non`, optional parameter prefixes, a
`QualifiedSymbol`, and optional parenthesized term arguments. `TypeHead`
preserves builtin `object` / `set` tokens or `QualifiedSymbol` heads plus
optional `TypeArguments`. Task 8 preserves `ParameterPrefix` only when the
incoming tokens already expose a local prefix split before an attribute
reference: identifier or numeral plus `-`, or a parenthesized identifier/numeral
list plus `-`. It does not validate template-parameter scope and does not split
one whole user-symbol spelling such as `n-dimensional`; that is classified as
source drift until a later parser/lexer task can consume parameter-scope facts
and active attribute suffixes.

`TypeArguments` preserves `of` / `over` argument lists. After task 9 those
lists own `TermExpression` children for primary terms rather than temporary
term-entry placeholders. Bracket type arguments recursively parse nested
`TypeExpression` children when possible. When a bracket argument instead matches `qua_arg`, task 8 stores it in a
temporary `TermPlaceholder` child that owns the identifier and any `qua`
radix-type tail tokens. Missing `]` is kept as `MalformedTypeExpression` plus
`UnmatchedOpeningDelimiter` recovery. The task-8 `TermPlaceholder` node is a
shallow token owner for one bracket `qua_arg` argument only after task 9; it
must not encode term classification, operator facts, name resolution, or
overload selection.

Malformed type syntax that can continue at the current reserve statement
boundary uses `MalformedTypeExpression`. Missing pure type expressions after
`reserve ... for` or inside bracket `type_arg_list` may insert
`MissingTypeExpression` recovery. Missing `of` / `over` term arguments are
task-9 term recovery (`MalformedTermExpression` plus `MissingTerm`) and must
not be reported as missing type expressions. Malformed tails before `;`, `,`,
`]`, or `)` may use nested `SkippedToken` recovery and skipped-range trivia
owned by the nearest reserve/type node.

Active parse-only corpus imports the syntax-only `parser.type_fixtures` module
to make identifier-shaped mode/attribute/structure symbols visible. The
`mizar-test` parse-only provider exports a small fixed set of task-8 fixture
symbols only for that fixture module; those symbols are test harness inputs only
and do not imply resolver semantics or built-in library content. Task-8 tests
pin at least: the rightmost attribute/type-head split for consecutive fixture
symbols, positive `non` attribute chains, `of` / `over` argument lists,
bracket nested `TypeExpression` arguments, bracket `qua_arg` placeholders,
local `ParameterPrefix` preservation in parser unit tests where tokens expose
the split, missing `]` diagnostics, and malformed type-expression insertion
after `reserve ... for`.

## Task 9: Primary Terms

Production inventory:

```ebnf
term_expression      ::= operator_expression { "qua" type_expression } ;
operator_expression  ::= postfix_expression | functor_application ;
postfix_expression   ::= term_primary { term_postfix } ;

term_primary         ::= variable_identifier
                       | "it"
                       | numeral
                       | "(" term_expression ")"
                       | struct_constructor
                       | set_enumeration
                       | choice_expression
                       | inline_functor_application
                       | bracket_functor_application ;
variable_identifier ::= identifier ;
numeral             ::= digit+ ;

choice_expression   ::= "the" type_expression ;
struct_constructor  ::= struct_ref_name [ type_args ]
                         "(" [ named_arg { "," named_arg } ] ")" ;
named_arg           ::= identifier ":" term_expression ;
set_enumeration     ::= "{" [ term_list ] "}" ;
term_list           ::= term_expression { "," term_expression } ;
inline_functor_application ::= inline_func_name "(" [ term_list ] ")" ;
bracket_functor_application ::= "[" term_list "]" ;
```

Task 9 introduces syntax-only primary term nodes and wires them into the
task-8 type parser wherever the grammar says `argument_list` or `term_list`.
`TypeArguments` for `of` / `over` and parenthesized `AttributeRef` arguments
therefore own `TermExpression` children rather than task-8 `TermPlaceholder`
children. Bracket `type_arg_list` behavior remains deterministic from task 8:
arguments that parse as `type_expression` stay nested `TypeExpression` nodes,
and entries that match `qua_arg` stay `TermPlaceholder` until task 11.
Task 9 does not reinterpret bracket type arguments as term expressions.
`template_functor_application` is a normative term primary, but it requires the
template argument surface owned by parser task 31 / mizar-syntax S-016; task 9
records that as deferred `source_drift` and does not parse template functor
applications.

The parser emits `TermExpression` as the current term wrapper. For task 9 it
contains exactly one primary-term child because selector/update postfixes,
`qua`, and active operator parsing are later parser tasks. `TermReference`
wraps either an identifier token or a shared `QualifiedSymbol` in term
position without deciding whether the symbol is a variable, inline functor,
structure name, or other semantic entity. `NumeralTerm`, `ItTerm`,
`ParenthesizedTerm`, `ChoiceTerm`, `ApplicationTerm`, `StructureConstructor`,
`FieldArgument`, and `SetEnumeration` preserve the corresponding source
delimiters and source-ordered children.

Parenthesized application syntax is parsed as `ApplicationTerm` unless the
argument list contains visible `identifier ":" term_expression` field
assignments, in which case task 9 preserves it as `StructureConstructor`.
This is a syntax-only split: zero-field constructors such as `S()` remain
generic applications until a later semantic boundary supplies structure facts.
Built-in bracket functor notation with reserved `[` and `]` is preserved as
`ApplicationTerm`. Active user-symbol delimiter pairs from
`user_symbol term_list user_symbol` require bracket-pair metadata that is not
part of `ParserInputs` yet; parser task 12 owns that active-operator extension.

Task 9 produces `MalformedTermExpression` diagnostics for missing or malformed
primary terms inside term lists and may insert `MissingTerm` recovery at pure
insertion points. Missing `)` / `}` / `]` delimiters use
`MalformedTermExpression` with `UnmatchedOpeningDelimiter` recovery under the
nearest term node. Malformed tails that can synchronize at `,`, `;`, `)`, `]`,
`}`, or a top-level item boundary may use `SkippedToken` recovery with skipped
range trivia.

Active parse-only corpus should reach task-9 terms through reserve-hosted type
argument lists and attribute argument lists until statement/formula hosts land.
Tests should pin at least: identifiers and numerals in term position,
parenthesized terms, `it`, choice terms using `the type_expression`, ordinary
parenthesized application, named-field structure-constructor syntax, set
enumeration literals, reserved bracket functor application, missing term
arguments, and missing term delimiters.

Result: task 9 is implemented. `of` / `over` and parenthesized `AttributeRef`
argument lists now own syntax-only `TermExpression` children for primary
terms; bracket `type_arg_list` still keeps nested `TypeExpression` children and
`qua_arg` `TermPlaceholder` children. Parser unit tests and active parse-only
pass/fail corpus cases cover the primary-term forms and recovery behavior
listed above.

Task 10 resolves the parser/syntax dot-role surface shape without adding
semantic lookup. A dotted qualified-name head remains a `QualifiedSymbol` when
the incoming token kinds already expose a qualified symbol. A reserved `.`
after an already parsed term is a postfix selector surface. The parser does
not decide whether a spelling is a namespace segment or a selected field using
scope; resolver phases own that classification.

For task 10, `TermExpression` still owns one current term-shape child, but that
child may now be a primary term or a nested postfix chain. `SelectorAccess`
preserves the base term-shape child, the `.` token, an identifier field token,
and an optional parenthesized argument list. Chained selectors nest
left-associatively, so `line.finish.y` is a selector whose base is the selector
`line.finish`. `StructureUpdate` preserves a base term-shape child, the `with`
token, delimiters, source-ordered `FieldUpdate` children, and comma tokens.
`FieldUpdate` owns an identifier selector path, the `:=` token, and a
`TermExpression` or `MissingTerm` recovery for the value. The selector path
uses identifiers only, matching `field_name`; examples and tests avoid
reserved-word field names such as `end`.

Task 10 parses functional update terms such as `p with (x := y)` wherever the
task-9 term parser is currently reachable. It does not parse standalone
in-place selector assignments such as `p.x := t`, because statement and
algorithm hosts are owned by later parser tasks. A leading `with (...)` remains
malformed because `with` is a postfix, not a `term_primary`. Malformed selector
or update syntax uses `MalformedTermExpression`; missing update values insert
`MissingTerm`; missing `)` delimiters use `UnmatchedOpeningDelimiter` under
the nearest selector/update term node.

Task 10 result: selector/update postfix parsing is implemented in the module
grammar. Unit coverage pins selector chains and calls, functional update lists,
missing update values, missing update delimiters, and the structure-constructor
field-list boundary after selector arguments. Active parse-only pass/fail
fixtures cover the frontend seam and trace back to §2.5.3 and §13.3.2-13.3.3.

## Public Enum Compatibility

`ParserTokenKind` is `#[non_exhaustive]` for downstream crates. The parser token
transfer vocabulary can grow as parser-facing lexing contexts gain additional
token classes, and downstream consumers should keep wildcard fallback arms.
Matches inside `mizar-parser` remain exhaustive so newly added token kinds force
local parser updates.
