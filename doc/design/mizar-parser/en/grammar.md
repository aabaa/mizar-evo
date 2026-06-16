# mizar-parser: Grammar

Status: module skeleton, top-level placeholder dispatch, concrete import
items, export items, visibility wrappers, reserve-hosted type expressions,
task-15 term surfaces including set comprehensions, task-14 formula surfaces,
S-013 statement nodes, and task-22 theorem/proof items are implemented;
remaining concrete definition, registration, template, algorithm, annotation,
and package-oriented item grammars are planned.

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
  `of`/`over` `TermExpression` arguments, bracket nested type arguments, and
  bracket `qua_arg` entries parsed as `TermExpression` / `QuaExpression`
  surfaces. Task 12 extends those term arguments with active-lexicon
  prefix/postfix/infix operator expressions before `qua`. Other non-module
  item grammars remain placeholders.

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
it: theorem items and notation declarations. The parser emits a `VisibleItem`
wrapper whose children are source ordered: any already-skipped library
annotation prefix tokens, one `VisibilityMarker` wrapping the `private` or
`public` token, and the following target item node. Represented theorem and
lemma targets use concrete `TheoremItem` / `LemmaItem` nodes; notation targets,
short legacy theorem fragments, and theorem payloads that contain deferred
template predicate arguments remain `PlaceholderItem` targets. Legal target
starts are `theorem`, `lemma`, theorem status plus theorem role (`open`,
`assumed`, or `conditional` followed by `theorem` or `lemma`), and notation
starts `infix_operator`, `prefix_operator`, `postfix_operator`, `synonym`, and
`antonym`. Visibility on other top-level declarations, duplicate visibility
markers, and a dangling marker use `MalformedVisibility`; any malformed tail
tokens before the statement semicolon are skipped inside the single
`VisibleItem`. A semicolon-only dangling marker keeps the semicolon as a direct
`VisibleItem` child rather than creating an empty recovery node. If the invalid
target is a block-like top-level declaration (`definition`, `registration`, or
`claim`), the same recovery owns the malformed target through its matching
`end`; the following semicolon remains the wrapper's statement terminator when
present, which keeps the wrapper from cascading into additional top-level
recovery nodes.

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
not part of the language, and Chapter 4 classifies block-local
`reserve`-shaped statements as syntax errors. A well-formed `ReserveItem` owns
the `reserve` token, one `ReserveSegment`, and the terminating semicolon.
`ReserveSegment` owns source-ordered identifiers and comma tokens, the `for`
token, and a `TypeExpression`.

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
`TypeExpression` children when possible. From task 11 onward, when a bracket
argument instead matches Appendix-A `qua_arg`, the parser stores it as a
`TermExpression` child whose term-shape is an identifier `TermReference` or a
left-nested `QuaExpression` chain. This bracket fallback remains narrower than
ordinary term parsing: it starts from an identifier-shaped `qua_arg`, and each
`qua` target is parsed as a radix-type-shaped `TypeExpression`. Missing `]` is
kept as `MalformedTypeExpression` plus `UnmatchedOpeningDelimiter` recovery.
The task-8 `TermPlaceholder` node is retained only as legacy vocabulary after
task 11 and must not encode term classification, operator facts, name
resolution, or overload selection.

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
`user_symbol term_list user_symbol` require bracket-pair metadata beyond the
task-12 prefix/postfix/infix `OperatorFixity` entries and remain deferred until
that metadata exists.

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

## Task 11: `qua` Qualification

Production inventory:

```ebnf
term_expression      ::= operator_expression { "qua" type_expression } ;
qua_expression       ::= operator_expression "qua" type_expression
                         { "qua" type_expression } ;
```

Task 11 parses `qua` as the lowest-precedence term-level operator currently
implemented. The parser first forms primary terms and task-10 selector/update
postfix chains, then folds any `qua` suffixes into left-nested
`QuaExpression` nodes. `p.x qua T` therefore qualifies the selector result.
Selectors after a qualified term require parentheses because selector/update
postfixes bind before `qua`: `(p qua T).x` parses the selector on the
parenthesized qualified term, while `p qua T.x` leaves any dot inside the
target type only if the type parser can form that dotted type surface.

The target of ordinary term-level `qua` is a `TypeExpression`. If that type
contains `of` / `over` term arguments, those arguments are parsed with the full
term parser before an outer `qua` chain can continue. Consequently
`x qua Element of S qua Magma` is represented as `x qua Element of (S qua
Magma)`, while `(x qua Element of S) qua Magma` is required to qualify the
outer result again.

Bracket `type_arg_list` entries that match Appendix-A `qua_arg` are no longer
stored as task-8 `TermPlaceholder` nodes. They are parsed as `TermExpression`
children with an identifier `TermReference` base and optional left-nested
`QuaExpression` suffixes. This fallback is intentionally narrower than
ordinary term parsing: it starts from an identifier and each target uses
radix-type syntax, matching `qua_arg ::= identifier { "qua" radix_type }`.

Missing ordinary `qua` target types emit `MalformedTypeExpression` and insert a
`MissingTypeExpression` recovery child under the `QuaExpression`. Malformed
target tails synchronize with the type-expression recovery boundary before the
surrounding term parser continues. Bracket `qua_arg` recovery keeps using the
`TypeArguments` bracket diagnostics for missing `]`; a missing type after
bracket `qua` uses the same `MissingTypeExpression` child under the
`QuaExpression`.

Task 11 tests should pin at least: left-associative `qua` chains, selector and
application precedence (`p.x qua T`, `f(a) qua T`, `(p qua T).x`), the
`Element of S qua Magma` target-type argument binding, bracket `V qua R`
surface migration away from `TermPlaceholder`, missing and malformed target
diagnostics, and active parse-only pass/fail coverage traced to Chapter 13.

Task 11 result: `qua` qualification parsing is implemented in the module
grammar. Unit coverage pins left-associative chains, selector/application
precedence, parenthesized selector-after-`qua`, target-type argument binding,
bracket `qua_arg` migration away from `TermPlaceholder`, missing target
recovery, and malformed target-tail recovery. Active parse-only pass/fail
fixtures cover the frontend seam and trace back to §13.6.

## Task 12: Operator Expressions

Production inventory:

```ebnf
operator_expression ::= prefix_expression
                      | postfix_expression
                      | infix_expression
                      | selector_or_primary_term ;
prefix_expression   ::= prefix_operator operator_expression ;
postfix_expression  ::= operator_expression postfix_operator ;
infix_expression    ::= operator_expression infix_operator operator_expression ;
term_expression     ::= operator_expression { "qua" type_expression } ;
```

The concrete parser uses the Pratt contract in [pratt.md](./pratt.md) rather
than directly recursing through the schematic productions above. Operator
metadata reaches this crate as `ParseRequest::operator_fixity`, derived by the
frontend from `ParserInputs`: each entry records the source spelling, fixity
kind, precedence, and infix associativity when applicable. The parser uses this
table to group source tokens into `PrefixExpression`,
`PostfixExpression`, and `InfixExpression` syntax nodes. It does not resolve
overloads, validate result types, or invent default fixity for visible symbols
that are not present in the table. Default precedence and associativity from
Chapter 10 / Appendix B are expected to be materialized by the lexical-summary
producer before the frontend builds the parser `ParseRequest`.

Task 12 extends the module term parser rather than the legacy token-only Pratt
entry point. Each Pratt operand is the already implemented primary term plus
fixed selector/update postfix chain, so selectors, selector calls, ordinary
applications, structure updates, and parenthesized terms bind tighter than user
operators. `qua` remains outside Pratt as the fixed lowest-precedence
term-level operator. For example, `p.x ++ y qua T` groups the selector inside
the left operand and then qualifies the full infix expression, while
`(p qua T).x ++ y` requires parentheses for the selector after `qua`.
If the same source spelling is visible as both postfix and infix after a left
operand, the parser chooses the infix form when that infix entry is eligible
and the following token can start a right operand; otherwise it chooses the
postfix form when that postfix entry is eligible at the current binding power.

Postfix operators use a two-child node `[base, operator_token]`. Prefix
operators use `[operator_token, operand]`. Infix operators keep the existing
three-child order `[left, operator_token, right]` and preserve spelling,
precedence, and associativity payload. Non-associative chaining of the same
infix operator emits `NonAssociativeOperatorChain` at the second operator's
range. A missing infix right operand emits `DanglingOperator` at the dangling
operator range and leaves the partial left expression represented. A missing
prefix operand emits `DanglingOperator` at the prefix operator range and keeps
the represented `PrefixExpression` recoverable by inserting a `MissingTerm`
operand.

Task 12 tests pin active-lexicon fixity derivation from parse-only fixture
summaries, prefix/postfix/infix surface nodes, left/right/non-associative
grouping, dangling operator diagnostics, interaction with selector/update,
application, parentheses, and `qua`, plus active parse-only pass/fail corpus
coverage traced to Chapter 13 and Appendix B by
`spec.en.13.operator_precedence.parser`.

## Task 13: Atomic Formulas

Production inventory:

```ebnf
formula              ::= atomic_formula ;
atomic_formula       ::= predicate_application
                       | inline_predicate_application
                       | is_assertion ;

predicate_application        ::= user_predicate_application
                               | builtin_predicate_application ;
user_predicate_application   ::= predicate_segment { predicate_chain_segment } ;
predicate_segment            ::= [ term_list ] [ negation ] predicate_head
                                  [ term_list ] ;
predicate_chain_segment      ::= [ negation ] predicate_head term_list ;
predicate_head               ::= predicate_symbol ;
builtin_predicate_application ::= term_expression builtin_pred term_expression ;
inline_predicate_application ::= inline_pred_name "(" [ term_list ] ")" ;
is_assertion                 ::= term_expression "is" [ "not" ]
                                  is_assertion_body ;
is_assertion_body            ::= type_expression | attribute_test_chain ;
attribute_test_chain         ::= [ "non" ] attribute_ref
                                  { [ "non" ] attribute_ref } ;
negation                     ::= "does" "not" | "do" "not" ;
builtin_pred                 ::= "in" | "=" | "<>" ;
```

Task 13 implements the atomic-formula boundary only. Formula connectives,
quantifiers, parenthesized formulas, `thesis`, and `contradiction` stay with
task 14. Task 13 originally used theorem/lemma placeholder hosts for
`label: formula;` coverage; after task 22, theorem/lemma items with represented
formula payloads are concrete `TheoremItem` / `LemmaItem` nodes. Sources whose
formula payload still contains deferred template predicate arguments remain on
the legacy token-preserving `PlaceholderItem` path until task 31 / S-016.

`FormulaExpression` wraps one atomic formula child. Built-in predicate
applications preserve the left `TermExpression`, builtin predicate token, and
right `TermExpression`; missing right operands use term recovery rather than a
formula-specific diagnostic. `IsAssertion` preserves the subject
`TermExpression`, the `is` token, optional formula-level `not`, and a generic
body child. The body is either a `TypeExpression` or `AttributeTestChain`; the
parser does not decide whether the assertion is a type assertion or an
attribute assertion. `AttributeTestChain` reuses task-8 `AttributeRef`
surfaces and can represent attribute-only bodies such as `non empty` that do
not have a trailing type head. For task-13 active fixtures, bare lowercase
attribute-like bodies such as `empty` are also kept as `AttributeTestChain`
when no trailing type head can form a `TypeExpression`; uppercase or
type-argument-bearing bodies such as `T` remain `TypeExpression` surfaces.
This is a syntactic preservation rule, not resolver classification.

User predicate applications are syntax-only. `PredicateApplication` owns one
or more `PredicateSegment` children. A segment preserves optional left
`TermExpression` list children, optional `does not` / `do not` negation tokens,
one `PredicateHead`, and optional right term-list children. Predicate-chain
adjacency and overload validity are resolver-owned; the parser preserves the
written segments without proving that a chain such as `a < b < c` can resolve.
Built-in predicates are not predicate-chain heads: `in`, `=`, and `<>` form
single `BuiltinPredicateApplication` atoms only, so mixed chains such as
`a < b = c` remain syntax errors instead of being represented as user
predicate chains. Template predicate arguments remain deferred to task 31 /
S-016 because `template_args` is not represented yet.

The theorem/lemma formula host is exact for task-13 shapes: represented
`label: formula;` payloads emit `FormulaExpression` under the later concrete
theorem item. A missing right term in a predicate-chain segment reports
`MalformedTermExpression` and inserts `MissingTerm`.

Task 13 tests must pin built-in `in`, `=`, and `<>` atoms, generic `is`
assertions including an attribute-only `non` chain, inline predicate call
shape, active-lexicon user predicate segments, theorem formula
hosting, and malformed atomic formula recovery that does not require semantic
classification.

## Task 14: Connectives And Quantifiers

Production inventory:

```ebnf
formula              ::= quantified_formula | iff_formula ;
iff_formula          ::= implies_formula
                         [ "iff" ( implies_formula | quantified_formula ) ] ;
implies_formula      ::= or_formula
                         [ "implies" ( implies_formula | quantified_formula ) ] ;
or_formula           ::= and_formula
                         { "or" ( and_formula | quantified_formula )
                         | "or" "..." "or"
                           ( and_formula | quantified_formula ) } ;
and_formula          ::= not_formula
                         { "&" ( not_formula | quantified_formula )
                         | "&" "..." "&"
                           ( not_formula | quantified_formula ) } ;
not_formula          ::= "not" ( not_formula | quantified_formula )
                       | atomic_formula
                       | "(" formula ")"
                       | "contradiction"
                       | "thesis" ;

quantified_formula   ::= universal_formula | existential_formula ;
universal_formula    ::= "for" quantified_vars [ "st" formula ]
                         ( "holds" formula | quantified_formula ) ;
existential_formula  ::= "ex" quantified_vars "st" formula ;

quantified_vars      ::= explicit_vars [ "," implicit_vars ] | implicit_vars ;
explicit_vars        ::= qualified_segment { "," qualified_segment } ;
qualified_segment    ::= var_list ( "being" | "be" ) type_expression ;
implicit_vars        ::= var_list ;
var_list             ::= identifier { "," identifier } ;
```

Task 14 completes the S-012 formula surface by replacing the task-13
atomic-only formula parser with a fixed formula-precedence parser. Formula
precedence is separate from term operator fixity: `not` binds tighter than
`&`, then `or`, then right-associative `implies`, then non-associative `iff`,
and quantifiers bind as the outermost formula forms. `iff` chaining without
parentheses reports `NonAssociativeOperatorChain`. Term Pratt parsing still
owns user-defined functor precedence inside atomic formula operands.

`FormulaExpression` continues to wrap exactly one formula child. Task 14 adds
`PrefixFormula` for `not`, `BinaryFormula` for `&`, `or`, `implies`, `iff`,
`ParenthesizedFormula`, `QuantifiedFormula`, `QuantifierVariableSegment`, and
`FormulaConstant` for `thesis` and `contradiction`. Binary formula nodes
preserve the connective token; `& ... &` and `or ... or` preserve both
connective tokens plus the `...` token on the same binary node. Expansion and
alpha-equivalence checks for repetition forms remain semantic/checker work.

`QuantifiedFormula` preserves the quantifier token (`for` or `ex`), one or
more `QuantifierVariableSegment` children separated by comma tokens, optional
`st` condition formula for universal quantification, required `st` body
formula for existential quantification, and either a `holds` body formula or a
nested quantified-formula body for universal quantification. A
`QuantifierVariableSegment` preserves the written variable token list, optional
`be` / `being` token, and optional `TypeExpression`; it does not resolve
implicit variable types from `reserve`.

The theorem/lemma formula host expands from atomic formulas to all task-14
formulas. After task 22, prefixes followed by theorem justification or proof
tails such as `by` or `proof` are concrete theorem items, while template
predicate arguments remain deferred to task 31 / S-016. Task 15 owns Fraenkel
and set-builder terms that embed formulas inside term syntax.

Malformed formula operands after `not`, connectives, quantifier `st`, or
`holds` insert `MissingFormula` recovery and report
`MalformedFormulaExpression`. Quantifier headers are preserved when at least
one variable segment can be represented. A missing explicit type after `be` or
`being` reuses `MissingTypeExpression` recovery and
`MalformedTypeExpression`; malformed quantifier-header separators or tails
report `MalformedFormulaExpression`. Task 14 does not synthesize a missing
variable segment when the quantified variable list is entirely absent; that
input stays outside the concrete formula host until a later binder-recovery
task adds dedicated missing-binder vocabulary.

Task 14 tests must pin connective precedence and associativity, parenthesized
formula grouping, non-associative `iff` rejection, repetition-token
preservation, `thesis` / `contradiction`, universal and existential
quantifiers with explicit and implicit variables, nested universal
quantification without repeated `holds`, theorem formula hosting,
and missing-formula recovery.

## Task 15: Fraenkel And Set-Builder Terms

Production inventory:

```ebnf
set_expression       ::= set_enumeration | set_comprehension ;
set_enumeration      ::= "{" [ term_list ] "}" ;
set_comprehension    ::= "{" term_expression "where" typed_var_list
                          [ ":" formula ] "}" ;
typed_var_list       ::= typed_var { "," typed_var } ;
typed_var            ::= identifier "is" type_expression ;
```

Task 15 completes the S-011 term surface by adding the set-comprehension
primary term from Chapter 13. `SetEnumeration` remains the task-9 surface for
`{}` and `{ term_list }`; `SetComprehension` is selected only when the first
brace child parses as one mapper `TermExpression` followed by a top-level
`where` before the brace closes. The mapper term is the already implemented
task-12 term surface, so selector/update, `qua`, and active operator grouping
are preserved inside it. Nested comprehensions are ordinary nested
`SetComprehension` term children.

`SetComprehension` child order is source order: `{`, mapper
`TermExpression`, `where`, one or more `ComprehensionVariableSegment` children
separated by comma tokens, optional `:` plus `FormulaExpression`, then `}` or
delimiter recovery. `ComprehensionVariableSegment` owns the generator
identifier or a `MissingTerm` recovery in the identifier position, the `is`
token when present, and a `TypeExpression` or `MissingTypeExpression` recovery
when the `is` token is present. The parser does not resolve binder identity, implicit domains,
sethood, capture, mapper result type, or the elaborated Fraenkel symbol.

The optional condition after `:` uses the task-14 formula parser. Template
predicate arguments inside that formula remain deferred to task 31 / S-016;
when a theorem/lemma formula host contains a comprehension payload with such a
deferred predicate template surface, the host remains a legacy placeholder
rather than partially parsing task-15 syntax. Condition omission is represented
by the absence of both `:` and `FormulaExpression`, not by a synthetic `thesis`
or implicit true formula.

Missing mapper terms, missing generator identifiers, missing generator `is`,
and malformed generator separators use `MalformedTermExpression`; pure mapper
insertion points use `MissingTerm`, and generator segments may own `MissingTerm`
in the identifier position until a future binder-specific recovery vocabulary
exists. A missing generator type after `is` reuses `MalformedTypeExpression`
plus `MissingTypeExpression`. A colon
without a following condition formula reports `MalformedFormulaExpression` and
inserts `MissingFormula`. Missing `}` uses `MalformedTermExpression` with
`UnmatchedOpeningDelimiter` under the `SetComprehension` node.

Task 15 tests must pin: a comprehension with an omitted condition, a
conditioned comprehension, multiple generators with comma preservation, mapper
term precedence and nested comprehension structure, active parse-only
pass/fail fixtures, missing generator type recovery, missing condition formula
recovery, missing generator `is` recovery, missing closing-brace recovery, and
the distinction between set enumeration and set comprehension.

Task 15 result: `SetComprehension` and `ComprehensionVariableSegment` are
implemented as primary term surfaces. The parser selects comprehension syntax
only when a top-level `where` appears before the first top-level separator,
keeps enumeration syntax on the existing `SetEnumeration` path, reuses the
task-14 formula parser for optional conditions, and emits the documented
missing type, missing formula, missing term, and unmatched delimiter recovery
nodes. Active parse-only pass/fail fixtures and
`spec.en.13.set_expressions.parser` traceability cover the increment. The
lexer scope skeleton also treats expression-level `is set` type words as type
syntax rather than malformed `set name =` binder statements, so
set-comprehension fixtures can run in the active parse-only corpus.

## Task 16: Simple Statements

Task 16 starts S-013 statement syntax with the Chapter 15 statement forms that
carry no justification clause in this increment:

```ebnf
statement_item      ::= simple_statement ;
simple_statement   ::= let_statement
                     | assume_statement
                     | take_statement
                     | set_statement
                     | given_statement ;
let_statement      ::= "let" qualified_variable_segment
                       { "," qualified_variable_segment }
                       [ "such" condition_list ] ";" ;
assume_statement   ::= "assume" ( proposition | condition_list ) ";" ;
take_statement     ::= "take" witness { "," witness } ";" ;
set_statement      ::= "set" equating { "," equating } ";" ;
given_statement    ::= "given" qualified_variable_segment
                       { "," qualified_variable_segment }
                       [ "such" condition_list ] ";" ;
condition_list     ::= "that" proposition { "and" proposition } ;
proposition        ::= [ identifier ":" ] formula_expression ;
witness            ::= term_expression | identifier "=" term_expression ;
equating           ::= identifier "=" term_expression ;
```

`StatementItem` is a parser-owned temporary item host so active parse-only
fixtures can exercise module-level statement fragments. Task 22 proof blocks
own the same concrete statement nodes directly without the `StatementItem`
wrapper. Statement-level annotations are
not parsed in this task; annotated statement sources remain legacy placeholder
or recovery input until task 35 / S-016. The canonical Chapter 4 specification
classifies `reserve` as a top-level module declaration only, so task 16 treats
`reserve` coverage as a non-regression of the existing task-8 `ReserveItem`
path and does not add a block-local `ReserveStatement` node.

`QualifiedVariableSegment` preserves the written identifier list, optional
`be` / `being` token, and optional `TypeExpression` or `MissingTypeExpression`
recovery. It does not resolve implicit types from module-level `reserve`.
`ConditionList` preserves the statement-level `that` / `and` separators; `and`
is not formula conjunction. `Proposition` owns an optional label token plus
colon and one `FormulaExpression` or `MissingFormula` recovery. `Witness`
preserves either an ordinary term witness or the `identifier "=" term` named
witness shape. `Equating` preserves `identifier "=" term_expression` for `set`
abbreviations.

Task 16 originally deferred the justification surface: a `let` statement with
a top-level `by` tail before its semicolon stayed a legacy placeholder rather
than a partially concrete `LetStatement`. Task 17 now replaces that boundary
with a concrete justification-aware shape. The task-16 parser also does not
validate labels, references, witness leakage, type well-formedness, or proof
obligations.

Statement recovery reuses existing syntax-level diagnostics. Missing qualified
types use `MalformedTypeExpression` plus `MissingTypeExpression`.
Missing proposition formulas use `MalformedFormulaExpression` plus
`MissingFormula`. Missing `take` witnesses, `set` equating identifiers, or
`set` right-hand sides use `MalformedTermExpression` plus `MissingTerm` until a
binder-specific recovery kind exists. Malformed statement tails synchronize at
semicolon, EOF, or the next statement/item boundary and preserve skipped source
under `SkippedToken` recovery when tokens must be skipped.

Task 16 tests must pin: concrete `let`, `assume`, `assume that`, `given`,
`take`, and `set` statements; `StatementItem` wrapping for direct statement
heads; statement-level `and` condition splitting; named and unnamed take witnesses;
multiple `set` equatings; top-level `reserve` non-regression through the
existing `ReserveItem` path; deferral of `let ... by ...`; and recovery for
missing type, formula, term, equals, and
semicolon boundaries. Active parse-only corpus coverage should use top-level
statement hosts for non-`reserve` simple statements and keep top-level
`reserve` coverage on the existing `ReserveItem` path.

## Task 17: Justifications And Citations

Task 17 starts S-014 proof-support syntax with justification clauses that can
be consumed by already-concrete statement hosts. Canonical Chapter 15 and
Chapter 16 define simple justifications as `by references`; Chapter 20 adds
the computation proof form. The parser TODO's older `from` wording is derived
documentation drift for this increment and is not implemented because no
Chapter 15/16 justification production admits it.

```ebnf
justification_clause     ::= simple_justification | computation_proof ;
simple_justification     ::= "by" references ;
references               ::= reference { "," reference } ;
reference                ::= identifier
                           | qualified_reference
                           | grouped_reference
                           | bulk_reference ;
qualified_reference      ::= namespace_path "." identifier ;
grouped_reference        ::= namespace_path ".{" grouped_reference_item
                              { "," grouped_reference_item } "}" ;
grouped_reference_item   ::= identifier ;
bulk_reference           ::= namespace_path ".*" ;
computation_proof        ::= "by" "computation"
                              [ "(" computation_option
                                  { "," computation_option } ")" ] ;
computation_option       ::= ( "steps" | "timeout" | "nest" )
                              ":" numeral ;
let_statement            ::= "let" qualified_variable_segment
                              { "," qualified_variable_segment }
                              [ "such" condition_list ]
                              [ simple_justification ] ";" ;
compact_statement        ::= proposition justification_clause ";" ;
```

Template arguments on references and grouped items remain deferred to task 31 /
S-016, where template argument surfaces are introduced. In task 17, a reference
followed by `[` before the next citation separator is recoverable malformed
justification syntax rather than a partially represented template invocation.
Full `proof ... end` blocks, theorem/lemma item nodes, and proof-body nesting
land in task 22.

The parser-facing `Numeral` token is the token-level representation used for
the canonical Chapter 20 `nat_literal` in computation options. Numeric
well-formedness beyond token category stays outside this syntax increment.
The option names are matched by spelling: `steps` and `timeout` may arrive as
identifier tokens in the current lexer table, while `nest` arrives as a
reserved-word token.

`JustificationClause` owns the leading `by` token plus either a `ReferenceList`
or a `ComputationJustification`. `ReferenceList` owns source-ordered
`Reference`, `QualifiedReference`, `GroupedReference`, or `BulkReference`
children separated by comma tokens. `Reference` owns a local identifier token.
`QualifiedReference` owns a `NamespacePath`, the final dot token, and the final
identifier token; the earlier namespace path helper remains semantic-free.
`GroupedReference` owns a `NamespacePath`, the compound `.{` token, one or
more `GroupedReferenceItem` children separated by comma tokens, and `}` when
present. `BulkReference` owns a `NamespacePath` plus the compound `.*` token.
`ComputationJustification` owns the `computation` token and optional
parenthesized `ComputationOption` list; each option owns its option keyword,
colon token, and numeral token.

Task 17 consumes justifications only at canonical hosts that are small enough
for this increment. `let ... by refs;` is upgraded from task-16 placeholder
behavior to `LetStatement`; this host accepts only `simple_justification`
because Chapter 15 defines the generalization tail as `[ "by" references ]`.
Task 17 also adds a minimal explicit-justification `CompactStatement` host so
`proposition by refs;` and `proposition by computation(...);` can exercise the
shared justification nodes. Compact statements without an explicit `by` tail,
compact equality versus zero-step iterative equality dispatch, conclusions,
`consider`, and `reconsider` remain with later statement tasks. `assume`,
`given`, `take`, and `set` do not gain justification tails in this task because
the canonical Chapter 15 productions do not define such tails. The parser also
does not resolve references, validate computation options, select ATP engines,
or replay computation proofs.

Malformed justification syntax uses `SyntaxDiagnosticCode::MalformedJustification`.
Missing references, grouped items, or computation option operands are
represented with `MissingProofStep` recovery nodes under the relevant
justification node. Unexpected top-level tokens inside a justification recover
to comma, `}`, `)`, semicolon, the next statement/item boundary, or EOF, and
preserve skipped source with `SkippedToken` recovery plus skipped-range trivia.

Task 17 tests must pin: simple local references, qualified references,
grouped citations, bulk citations, comma-separated mixed reference lists,
`by computation` with and without options on explicit compact statements,
upgrade of `let ... by ...` from placeholder to concrete `LetStatement`,
rejection or recovery for non-canonical `assume` / `given` / `take` / `set`
justification tails, malformed leading/trailing commas, missing grouped `}`,
missing computation-option values, deferred template argument recovery, the
derived-documentation-drift `from` tail staying outside task-17 justification
nodes, and
active parse-only pass/fail corpus coverage with traceability to Chapter 15
§15.2.1/§15.8, Chapter 16 §16.5, and Chapter 20 §20.9.2.

## Task 18: `consider` And `reconsider`

Task 18 continues S-013 statement syntax with the Chapter 15 linkable
statement forms that carry mandatory simple justifications. The task uses the
task-17 `JustificationClause` and `ReferenceList` surfaces, but only in the
simple citation form; `by computation` remains accepted only by the explicit
task-17 compact-statement host until a later specification explicitly admits it
for more statement kinds.

Chapter 15 defines these statements with `simple_justification` while also
stating that both forms have mandatory justification. For this parser
increment, that prose and the crate plan are treated as the controlling
syntax intent: task 18 requires an explicit `by references` tail and recovers a
missing tail as malformed justification syntax instead of silently accepting an
empty justification.

```ebnf
statement_item       ::= ... | consider_statement | reconsider_statement ;
consider_statement  ::= "consider" qualified_vars
                         "such" condition_list simple_justification ";" ;
reconsider_statement::= "reconsider" reconsider_item
                         { "," reconsider_item }
                         "as" type_expression simple_justification ";" ;
reconsider_item     ::= identifier [ "=" term_expression ] ;
simple_justification::= "by" references ;
```

`ConsiderStatement` owns the `consider` token, source-ordered
`QualifiedVariableSegment` children and comma tokens, the `such` token, a
`ConditionList`, a simple `JustificationClause`, and the semicolon when
present. It reuses task-16 `qualified_vars` behavior, so a single
`QualifiedVariableSegment` may preserve a shared-type identifier list such as
`x, y being Real`, and multiple typed segments remain source ordered. It also
reuses task-16 condition-list behavior: the child `ConditionList` owns the
`that` token and statement-level `and` separators, and propositions may carry
labels.

`ReconsiderStatement` owns the `reconsider` token, source-ordered
`ReconsiderItem` children and comma tokens, the `as` token, one
`TypeExpression`, a simple `JustificationClause`, and the semicolon when
present. `ReconsiderItem` owns either an identifier token for an existing name
or an identifier, `=`, and a `TermExpression` for a newly introduced narrowed
name. The parser does not resolve whether an identifier is already bound, check
that all reconsidered terms have the target type, or generate proof
obligations.

Task 18 recovery reuses existing syntax diagnostics. Missing or malformed
qualified variables and target types use `MalformedTypeExpression` plus
`MissingTypeExpression` when an insertion is needed. Missing `consider`
conditions use `MalformedFormulaExpression` plus `MissingFormula` or malformed
condition-list recovery. Missing or malformed `ReconsiderItem` identifiers or
right-hand-side terms use `MalformedTermExpression` plus `MissingTerm`.
Missing or malformed mandatory `by references` tails use
`MalformedJustification` plus `MissingProofStep` or task-17 malformed
reference-list recovery. Malformed statement tails synchronize at semicolon,
EOF, or the next statement/item boundary and preserve skipped source under
`SkippedToken` recovery when tokens must be skipped.

Task 18 tests must pin: concrete `consider` with the canonical shared-type
shape `consider x, y being T ... by ...`, concrete `consider` with multiple
qualified-variable segments, statement-level `such that` / `and` conditions
with labels, concrete `reconsider` with bare and equated items, shared
target-type ownership, simple citation justification reuse, rejection/recovery
for missing `qualified_vars` / `such` / condition / `as` / target type /
justification / reconsider item pieces, recovery for `by computation` on
these statement hosts, top-level `StatementItem` wrapping, active parse-only
pass/fail corpus coverage, and traceability to Chapter 15 §15.3.4, §15.5.1,
and §15.8.

## Task 19: Conclusions, `then`, And Iterative Equality

Task 19 continues S-013 statement syntax with conclusion statements,
sequential `then` modifiers, and iterative equality chains. It also resolves
grammar-audit G-AUD-010 for the parser-owned boundary between explicit
compact equality statements and iterative equality chains.

```ebnf
statement_item              ::= ... | then_statement | conclusion_statement
                              | iterative_equality_statement ;
then_statement              ::= "then" linkable_statement ;
linkable_statement          ::= compact_statement
                              | conclusion_statement
                              | consider_statement
                              | reconsider_statement
                              | iterative_equality_statement
                              | case_reasoning ;
conclusion_statement        ::= ( "thus" | "hence" ) proposition
                                [ justification_clause ] ";" ;
iterative_equality_statement::= [ label_identifier ":" ]
                                term_expression "=" term_expression
                                [ simple_justification ]
                                iterative_equality_step
                                { iterative_equality_step } ";" ;
iterative_equality_step     ::= ".=" term_expression
                                [ simple_justification ] ;
simple_justification        ::= "by" references ;
```

`ThenStatement` is a syntax-only wrapper that owns the `then` token and one
linkable statement child. The parser does not desugar `then`, does not attach
the previous statement semantically, and does not rewrite `hence` into
`then thus`. Case reasoning is spec-valid linkable syntax but its concrete
statement nodes are owned by parser task 20, so task 19 leaves `then per cases`
on the deferred statement-placeholder path rather than rejecting it as an
invalid `then` modifier. A `then` before an implemented standalone statement
such as `let` is rejected with `MissingStatement` recovery under
`ThenStatement`; the following standalone statement remains available as the
next statement boundary.

`ConclusionStatement` owns `thus` or `hence`, one `Proposition`, an optional
explicit `JustificationClause`, optional recovery, and the semicolon when
present. Because Chapter 15 defines simple justifications as optional, a
conclusion without an explicit `by` tail remains syntactically accepted. If an
explicit `by` tail is present, the conclusion uses the task-17 justification
surface; computation justifications are accepted here because `conclusion`
uses the general `justification` production rather than `simple_justification`.
Full `proof ... end` justification blocks land in task 22.

`IterativeEqualityStatement` owns an optional label and colon, the first left
term, `=`, the first right term, an optional simple citation
`JustificationClause`, one or more `IterativeEqualityStep` children, optional
recovery, and the semicolon when present. Each `IterativeEqualityStep` owns
the `.=` token, one term expression, and an optional simple citation
`JustificationClause`. Computation justifications are not accepted in
iterative equality because the Chapter 15 production uses
`simple_justification` for every step.

G-AUD-010 dispatch is resolved as follows: the parser constructs
`IterativeEqualityStatement` only when a top-level `.=` continuation follows
the first equality. A justified equality with no `.=` continuation, such as
`x = y by A;`, remains a `CompactStatement`. The same rule applies to label
and `then` variants: `A1: x = y by A;` is compact, while
`A1: x = y by A .= z by B;` is iterative.

Task 19 recovery reuses existing diagnostics. Missing conclusion propositions
or invalid `then` linkable statements use `MalformedFormulaExpression` plus
`MissingFormula` or `MissingStatement` recovery. Missing or malformed equality
terms and `.=` step terms use `MalformedTermExpression` plus `MissingTerm`.
Malformed explicit citation tails use `MalformedJustification` plus task-17
justification recovery. Malformed statement tails synchronize at semicolon,
EOF, or the next statement/item boundary and preserve skipped source under
`SkippedToken` recovery when tokens must be skipped.

Task 19 tests must pin: `thus` with labels and explicit references, `hence`
without an explicit `by`, `then` wrapping compact/conclusion/current
introduction statements, rejection of `then` before standalone statements,
iterative equality with one and multiple `.=` steps, the compact-versus-
iterative boundary for `x = y by A;` versus `x = y by A .= z by B;`, label
and `then` variants of that boundary, malformed conclusion propositions,
missing iterative-equality terms, disallowed computation justifications inside
iterative equality, active parse-only pass/fail corpus coverage, and
traceability to Chapter 15 §15.4.1, §15.4.2, §15.7, §15.8, and §15.9.1.

## Task 20: Block Statements

Parser task 20 continues mizar-syntax S-013 with concrete syntax nodes for
Chapter 15 reasoning blocks. It upgrades the task-19 deferred `then per cases`
placeholder path to a `ThenStatement` that wraps `CaseReasoningStatement` when
the case-reasoning body is otherwise parseable.

```ebnf
statement_item              ::= ... | now_statement | hereby_statement
                              | case_reasoning_statement ;
linkable_statement          ::= ... | case_reasoning_statement ;
standalone_statement        ::= ... | now_statement | hereby_statement ;
now_statement               ::= [ label_identifier ":" ] "now"
                                reasoning_body "end" ";" ;
hereby_statement            ::= "hereby" reasoning_body "end" ";" ;
case_reasoning_statement    ::= "per" "cases"
                                [ simple_justification ] ";"
                                ( case_list | suppose_list | empty_branch_list ) ;
case_list                   ::= case_item { case_item } ;
suppose_list                ::= suppose_item { suppose_item } ;
empty_branch_list           ::= /* accepted only for fragment recovery */ ;
case_item                   ::= "case" ( proposition | conditions ) ";"
                                reasoning_body "end" ";" ;
suppose_item                ::= "suppose" ( proposition | conditions ) ";"
                                reasoning_body "end" ";" ;
reasoning_body              ::= { statement } ;
```

The parser keeps block reasoning syntax-only. `NowStatement` owns an optional
label and colon, the `now` token, zero or more nested statement nodes, the
closing `end`, optional recovery, and the closing semicolon when present.
`HerebyStatement` has the same block-body shape without a label. `CaseItem` and
`SupposeItem` own their branch keyword, either a `Proposition` or a
`ConditionList` (selected by a leading `that`), the header semicolon, zero or
more nested statement nodes, the branch-closing `end`, optional recovery, and
the closing semicolon when present. `CaseReasoningStatement` owns `per`,
`cases`, an optional simple citation `JustificationClause`, the header
semicolon, and source-ordered homogeneous `CaseItem` children or homogeneous
`SupposeItem` children. Once the first branch kind is visible, the other branch
keyword is a statement boundary outside the current case-reasoning node; the
parser must not silently mix `case` and `suppose` lists.

Chapter 15's prose and examples include `per cases;`, while the complete EBNF
summary prints an unbracketed `simple_justification`. Grammar audit
G-AUD-011 records that nonblocking inconsistency. The parser surface accepts
both `per cases;` and `per cases by A;`, and it does not diagnose a branchless
`per cases;` fragment because active parse-only fixtures may exercise statement
fragments outside a complete proof. The parser still preserves any following
`case` or `suppose` branches when they are present.

Task 20 recovery reuses existing diagnostics. Missing block `end` tokens use
`MissingEnd` recovery plus `MissingEnd` diagnostics with the block opener as a
secondary anchor. Missing semicolons after block `end` or after case headers
use `MissingSemicolon`. Missing case/suppose propositions use
`MalformedFormulaExpression` plus `MissingFormula`. Malformed block tails
synchronize at semicolon, `end`, EOF, or the next statement/item boundary and
preserve skipped source under `SkippedToken` recovery when tokens must be
skipped.

Task 20 tests must pin: labelled `now` blocks, `hereby` blocks, nested
statements inside block bodies, `per cases` with `case` branches, `per cases`
with `suppose` branches, rejection/recovery for mixed branch-list keywords,
`then per cases`, rejection of `then now` / `then hereby`, optional simple `by`
after `per cases`, rejection of `by computation(...)` after `per cases`, branch
headers with proposition and condition-list forms, missing branch/body `end`
recovery, missing branch-header semicolon recovery, active parse-only pass/fail
corpus coverage, and traceability to Chapter 15 §15.4.3, §15.6.1, §15.6.2,
§15.6.3, §15.8, and §15.9.1.

### Task 21: Local Definitions

Task 21 completes the S-013 statement-node bucket by making Chapter 15 inline
definitions concrete statement nodes. The parser accepts the standalone
statement forms only; `then deffunc` and `then defpred` remain invalid because
Chapter 15 keeps inline definitions out of `linkable_statement`.

```ebnf
standalone_statement        ::= ... | inline_functor_definition
                                   | inline_predicate_definition ;
inline_functor_definition   ::= "deffunc" identifier "(" [ typed_parameters ] ")"
                                "->" type_expression "equals"
                                term_expression ";" ;
inline_predicate_definition ::= "defpred" identifier "(" [ typed_parameters ] ")"
                                "means" formula ";" ;
typed_parameters            ::= typed_parameter { "," typed_parameter } ;
typed_parameter             ::= identifier ( "being" | "be" ) type_expression ;
```

`InlineFunctorDefinition` owns the `deffunc` keyword, a definition-name slot,
parameter parentheses, zero or more source-ordered `TypedParameter` children
separated by comma tokens, the `->` token, one return `TypeExpression` or
`MissingTypeExpression` recovery, the `equals` keyword, one `TermExpression` or
`MissingTerm` recovery, optional malformed-tail recovery, and the final
semicolon when present. `InlinePredicateDefinition` owns the same head shape
with the `defpred` keyword, the `means` keyword, and one `FormulaExpression` or
`MissingFormula` recovery. The definition-name slot is either the written
identifier token or `MissingTerm` recovery. `TypedParameter` owns one parameter
identifier token when present, optional `be` or `being` when written, and a
`TypeExpression` or `MissingTypeExpression` recovery. If the binder keyword is
missing but a type can be parsed before a parameter-list delimiter, the parser
keeps that type under the `TypedParameter` and diagnoses the missing binder;
otherwise it inserts `MissingTypeExpression` at the delimiter.

The parser treats `->`, `equals`, and `means` as inline-definition delimiters:
they stop type, term, and formula expression parsing/recovery at top level but
do not become expression operators. Inline definition parsing remains purely
syntactic. It does not expand definitions, validate captured variables, check
parameter guard satisfaction, introduce scope bindings, or classify later
applications of `deffunc` / `defpred` names.

Task 21 recovery reuses existing diagnostics. Missing definition names use
`MalformedTermExpression` plus `MissingTerm` recovery. Missing semicolons use
`MissingSemicolon`; missing `(`, `)`, `->`, `equals`, or `means` delimiters use
the closest existing malformed type/term/formula diagnostic while preserving
the inline-definition node when recovery can continue. Missing parameter and return types use
`MalformedTypeExpression` plus `MissingTypeExpression`; missing functor bodies
use `MalformedTermExpression` plus `MissingTerm`; missing predicate bodies use
`MalformedFormulaExpression` plus `MissingFormula`. Malformed parameter lists
and definition tails synchronize at `,`, `)`, `->`, `equals`, `means`,
semicolon, `end`, the next statement boundary, the next item boundary, or EOF.

Task 21 tests must pin: `deffunc` with typed parameters, zero-argument
`deffunc`, `defpred` with typed parameters, zero-argument `defpred`, use inside
reasoning bodies, rejection of `then deffunc` / `then defpred`, missing
definition names, missing parameter type binders or types, missing `)`, missing
`->`, missing return type, missing `equals`, missing functor body, missing
`means`, missing predicate body, missing semicolon, active parse-only pass/fail
corpus coverage, and traceability to Chapter 15 §15.2.3, §15.2.4, and §15.9.1.

### Task 22: Theorems And Proofs

Task 22 completes the S-014 theorem/proof increment by replacing represented
theorem/lemma formula and proof tails with concrete item nodes. The parser
accepts the canonical Chapter 16 theorem item shape and remains syntax-only:
status tokens are preserved but not validated, references are not resolved,
and proof obligations or theorem validity are not checked.

```ebnf
theorem_item     ::= [ theorem_status ] theorem_role label_identifier ":"
                     formula [ justification ] ";" ;
theorem_status   ::= "open" | "assumed" | "conditional" ;
theorem_role     ::= "theorem" | "lemma" ;
justification    ::= justification_clause | proof_block ;
proof_block      ::= "proof" reasoning "end" ;
reasoning        ::= { statement } ;
```

`TheoremItem` and `LemmaItem` own optional status tokens, the role token, a
label identifier or `MissingTerm`, the colon token when present, a
`FormulaExpression` or `MissingFormula`, an optional `JustificationClause` or
`ProofBlock`, optional recovery, and the final semicolon when present. A
visibility wrapper (`public` / `private`) owns its `VisibilityMarker` and wraps
the concrete theorem or lemma target; notation targets continue through the
existing placeholder path until their own item grammar lands.

`ProofBlock` owns `proof`, nested concrete statement nodes parsed by the
reasoning-body parser, optional recovery including `MissingEnd`, and `end` when
present. The following semicolon belongs to the enclosing theorem item or
statement. Task 22 admits `ProofBlock` as a full justification tail on
theorem/lemma items and on already-concrete statement hosts whose canonical
grammar uses `justification`: `ConclusionStatement` and `CompactStatement`.
Hosts whose grammar uses `simple_justification` (`let`, `consider`,
`reconsider`, iterative equality steps, and `per cases`) continue to accept
only task-17 `by` clauses.

The concrete theorem path intentionally keeps short legacy fragments such as
`theorem T;` as token-preserving placeholders, because earlier parser skeleton
tests use them as generic item boundaries. Represented theorem shapes begin
with either a colon, a label-colon pair, or a missing-colon form where a formula
start is visible after the label. Formula payloads containing deferred
predicate template arguments also remain placeholders until task 31 / S-016.

Task 22 recovery reuses existing diagnostics. Missing theorem labels use
`MalformedTermExpression` plus `MissingTerm`. Missing colons and formulas use
`MalformedFormulaExpression`; missing formulas insert `MissingFormula`. Missing
proof `end` tokens insert `MissingEnd` with `MissingEnd` diagnostics. The
parser synchronizes theorem/proof tails at semicolons, `end`, the next
statement or item boundary, case/suppose branch keywords, or EOF, and must not
swallow the following theorem item after a missing proof end.

Task 22 tests must pin: theorem and lemma items, status tokens, visibility
wrapping of theorem targets, `by` and `by computation` theorem justifications,
full theorem proof blocks, proof-body statement wiring, statement-level proof
justifications on conclusions and compact statements, missing label / colon /
formula / proof-end recovery, active parse-only pass/fail corpus coverage, and
traceability to Chapter 16 §16.2, §16.4.1, §16.5, and Chapter 20 §20.9.2.

## Public Enum Compatibility

`ParserTokenKind` is `#[non_exhaustive]` for downstream crates. The parser token
transfer vocabulary can grow as parser-facing lexing contexts gain additional
token classes, and downstream consumers should keep wildcard fallback arms.
Matches inside `mizar-parser` remain exhaustive so newly added token kinds force
local parser updates.
