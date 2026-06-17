# Appendix A. Grammar Summary

> Canonical language: English. Japanese companion: [../ja/appendix_a.grammar_summary.md](../ja/appendix_a.grammar_summary.md).

This appendix is the canonical cross-chapter grammar summary for parser and
surface-AST work. The main chapters remain the normative source for semantics,
well-formedness, visibility, type checking, and verification behavior. When a
chapter uses an older spelling or a local helper name, this appendix gives the
normalized grammar-facing form that the parser should follow.

## A.1 EBNF Conventions and Shared Aliases

The grammar uses this notation:

- `::=` defines a production.
- `"token"` is a literal token spelling. Reserved words and reserved symbols
  are listed in A.2; contextual spellings are called out where they are used.
- `[ item ]` is optional.
- `{ item }` repeats zero or more times.
- `( a | b )` groups alternatives.
- `? ... ?` describes a lexical character class.

Common aliases used to reconcile chapter-local wording:

```ebnf
label              ::= label_identifier ;
nat_literal        ::= numeral ;
term               ::= term_expression ;
expr               ::= term_expression ;
proof_block        ::= proof ;

symbol_name        ::= identifier | user_symbol ;
def_symbol         ::= identifier | user_symbol ;
constructor_name   ::= identifier | readable_constructor_name ;
attribute_def_name ::= constructor_name ;
mode_def_name      ::= constructor_name ;
struct_def_name    ::= constructor_name ;
field_name         ::= identifier ;
selector           ::= field_name { "." field_name } ;
inline_func_name   ::= identifier ;
inline_pred_name   ::= identifier ;
deffunc_identifier ::= identifier ;
defpred_identifier ::= identifier ;
scheme_name        ::= label_identifier ;
formula_definiens ::= formula
                    | formula_case { "," formula_case } [ "otherwise" formula ] ;
formula_case      ::= formula "if" formula ;
term_definiens    ::= term_expression
                    | term_case { "," term_case } [ "otherwise" term_expression ] ;
term_case         ::= term_expression "if" formula ;
```

`term_expression` and `formula` are precedence-driven grammars. The productions
below remove direct left recursion where possible. Term-level user operators
and template functor applications are still parser-facing surface forms whose
binding behavior is resolved by the active operator table described in
Appendix B and Chapters 10 and 13.

## A.2 Lexical Structure

Normative reference: [Chapter 2 (Lexical Structure)](./02.lexical_structure.md).

```ebnf
whitespace    ::= " " | tab | newline ;
tab           ::= ? ASCII 0x09 ? ;
newline       ::= ? ASCII 0x0A | 0x0D 0x0A ? ;

identifier       ::= ( letter | "_" ) { letter | digit | "_" | "'" } ;
label_identifier ::= identifier ;
letter           ::= "a"..."z" | "A"..."Z" ;
digit            ::= "0"..."9" ;
character        ::= ? any Unicode scalar value ? ;

numeral          ::= digit { digit } ;

string_literal   ::= dq_string | sq_string ;
dq_string        ::= '"' { dq_char | escape_seq } '"' ;
sq_string        ::= "'" { sq_char | escape_seq } "'" ;
dq_char          ::= ? any character except '"' or '\' ? ;
sq_char          ::= ? any character except "'" or '\' ? ;
escape_seq       ::= "\" ( '"' | "'" | "\" ) ;

symbol_char      ::= ? any ASCII graphic character except "@" and whitespace ? ;
user_symbol      ::= symbol_char { symbol_char } ;
constructor_segment ::= ( letter | digit | "_" | "'" )
                        { letter | digit | "_" | "'" } ;
readable_constructor_name ::= constructor_segment "-"
                              constructor_segment
                              { "-" constructor_segment } ;
constructor_name ::= identifier | readable_constructor_name ;

line_comment     ::= "::"  { character - newline } newline ;
block_comment    ::= "::=" { character } "=::" ;
doc_comment      ::= ":::" { character - newline } newline ;
```

Reserved words are case-sensitive and cannot be identifiers, user symbols, or
constructor names.

```text
algorithm and antonym as assert assume assumed asymmetry attr
be being break by
case cases claim cluster coherence commutativity compatibility computation
conditional connectedness const consider consistency continue contradiction
decreasing deffunc definition defpred do does downto
else end ensures equals ex exhaustive existence export extends
field for from func
ghost given
hence hereby holds
idempotence if iff implies import in infix_operator inherit invariant
involutiveness irreflexivity is it
left lemma let
match means mode
nest non none not now
object of open or otherwise over
per postfix_operator pred prefix_operator private processed
projectivity proof property public
qua
reconsider reduce reducibility redefine reflexivity registration requires
reserve return right
set sethood snapshot st step struct such suppose symmetry synonym
take terminating that the then theorem thesis thus to transitivity type
uniqueness
var
where while with
```

Reserved special symbols are:

```text
,   .   ..   ;   :   :=   (   )   [   ]   {   }   .{
=   <>   &   ->   .=   .*   @[   ...
```

String literals are recognized only at grammar positions that require string
arguments, currently operator declarations and string-valued annotations.
Outside those positions, quote characters participate in ordinary identifier or
user-symbol lexing.

When `.` is not consumed as a compound reserved token or an active user symbol,
the parser preserves the syntax-only surface that is visible at the current
grammar position: a `.` after an already parsed term is a selector/update
postfix, while a `.` inside a qualified-name head stays part of that qualified
surface. The resolver later applies scope-dependent selector-versus-namespace
classification as described in Chapter 2.

The hyphen used in parameterized attribute spellings such as `n-dimensional`
or `(m,n)-ary` is a contextual `param_prefix` separator, not a reserved special
symbol. In attribute positions only, the lexer/parser first attempts a
`param_prefix` split when the prefix is an in-scope parameter name, a numeral,
or a parenthesized parameter list, and the suffix resolves to an active
attribute name. If that check fails, the entire spelling is matched against the
active lexicon as one attribute name. When both interpretations are valid, the
`param_prefix` split wins.

Fixed annotation names and option names are contextual spellings, not reserved
identifiers outside their grammar positions. Current contextual spellings are
`auto`, `cvc5`, `e`, `max_axioms`, `solver`, `steps`, `timeout`, `vampire`,
`z3`, `result`, and `term_size`, plus the fixed annotation-name spellings
following `@` in A.21.
The `@` marker is lexically reserved for annotations, but it is not a
standalone reserved-symbol token.

The reserved bracket tokens `[` and `]` are not user symbols, but they are
admitted by the grammar as a built-in bracket-functor delimiter pair when a
term primary is expected. After a template-capable type, functor, algorithm,
definition, or reference name, the same tokens start template arguments. The
compound token `@[` starts a library annotation.

## A.3 Type Expressions

Normative reference: [Chapter 3 (Type System)](./03.type_system.md).

```ebnf
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
attribute_name    ::= attribute_ref_name ;
mode_name         ::= mode_ref_name ;
struct_name       ::= struct_ref_name ;
attribute_ref_name ::= qualified_constructor_name ;
mode_ref_name     ::= qualified_constructor_name ;
struct_ref_name   ::= qualified_constructor_name ;

parameter_list    ::= parameter { "," parameter } ;
parameter         ::= identifier | numeral ;
qualified_symbol            ::= { namespace_segment "." } user_symbol ;
qualified_constructor_name  ::= { namespace_segment "." } constructor_name ;
namespace_segment           ::= identifier ;
```

The final token of `qualified_symbol` is a functor/predicate notation symbol
in the active lexicon. The final token of `qualified_constructor_name` is a
mode, structure, or attribute constructor name. Identifier-shaped symbols are
therefore parsed as symbols only when they are available in the active lexicon
and admitted by the current grammar position.

## A.4 Variables and Constants

Normative reference: [Chapter 4 (Variables and Constants)](./04.variables_and_constants.md).

```ebnf
reserve_decl       ::= "reserve" reserve_segment ";" ;
reserve_segment    ::= identifier_list "for" type_expression ;

identifier_list    ::= identifier { "," identifier } ;

let_decl           ::= "let" qualified_vars [ "such" conditions ] [ "by" references ] ";" ;
qualified_vars     ::= explicit_qualified_vars [ "," implicit_qualified_vars ]
                     | implicit_qualified_vars ;
explicit_qualified_vars ::= qualified_segment { "," qualified_segment } ;
qualified_segment  ::= identifier_list ( "being" | "be" ) type_expression ;
implicit_qualified_vars ::= identifier_list ;

set_decl           ::= "set" equating_list ";" ;
equating_list      ::= equating { "," equating } ;
equating           ::= identifier "=" term_expression ;

reconsider_decl    ::= "reconsider" type_change_list "as" type_expression
                       [ simple_justification ] ";" ;
type_change_list   ::= reconsider_item { "," reconsider_item } ;
reconsider_item    ::= identifier | identifier "=" term_expression ;

```

Statement-level forms are normalized in A.15.

## A.5 Structures

Normative reference: [Chapter 5 (Structures)](./05.structures.md).

```ebnf
struct_def       ::= "struct" struct_def_name [ type_params ] "where"
                     struct_member { struct_member } "end" ";" ;
struct_member    ::= field_decl | property_decl ;
field_decl       ::= "field" identifier "->" type_expression
                     [ ":=" term_expression ] ";" ;
property_decl    ::= "property" identifier "->" type_expression ";" ;
type_params      ::= ( "of" | "over" ) type_parameter_list
                   | "[" type_parameter_list "]" ;
type_parameter_list ::= identifier { "," identifier } ;

inherit_def      ::= "inherit" inherit_child "extends" parent_type
                     [ "where" inherit_member { inherit_member }
                       [ coherence_block ] "end" ] ";" ;
inherit_child    ::= struct_ref_name [ type_args ] ;
parent_type      ::= struct_ref_name [ type_args ] | "set" ;
inherit_member   ::= field_redef | property_redef ;
field_redef      ::= "field" identifier [ "->" type_expression ]
                     "from" ( identifier | "it" ) ";" ;
property_redef   ::= "property" identifier [ "->" type_expression ]
                     "from" identifier ";" ;

struct_constructor ::= struct_ref_name [ type_args ]
                       "(" [ named_arg { "," named_arg } ] ")" ;
named_arg          ::= identifier ":" term_expression ;
field_access       ::= term_expression "." field_name
                       [ "(" [ term_list ] ")" ] ;
```

## A.6 Attributes

Normative reference: [Chapter 6 (Attributes)](./06.attributes.md).

```ebnf
attr_def       ::= "attr" label ":" subject "is" attr_pattern
                   "means" formula_definiens ";" ;
subject        ::= identifier ;
attr_pattern   ::= [ param_prefix ] attribute_def_name ;

redefine_attr  ::= "redefine" "attr" label ":" subject "is" attr_pattern
                   "means" formula_definiens ";"
                   "coherence" [ "with" label ] justification ";" ;
```

Attribute use inside type expressions is summarized in A.3.

## A.7 Modes

Normative reference: [Chapter 7 (Modes)](./07.modes.md).

```ebnf
mode_def       ::= "mode" label ":" mode_def_name [ type_params ] "is"
                   attribute_chain radix_type ";"
                   [ mode_property ] ;
mode_property  ::= "sethood" justification ";" ;

property_impl        ::= "definition"
                           "let" identifier "be" mode_application ";"
                           ( property_means_impl | property_equals_impl )
                         "end" ";" ;
property_means_impl  ::= "property" identifier "." identifier
                         "means" formula_definiens ";"
                         existence_block uniqueness_block ;
property_equals_impl ::= "property" identifier "." identifier
                         "equals" term_definiens ";" ;

mode_application     ::= mode_ref_name [ type_args ] ;
```

## A.9 Predicates

Normative reference: [Chapter 9 (Predicates)](./09.predicates.md).

```ebnf
pred_def         ::= "pred" label ":" pred_pattern "means" formula_definiens ";" ;
pred_pattern     ::= [ loci ] def_predicate_symbol [ template_loci ] [ loci ] ;
loci             ::= locus_list | "(" locus_list ")" ;
locus_list       ::= locus { "," locus } ;
locus            ::= identifier ;
template_loci    ::= "[" locus_list "]" ;
def_predicate_symbol ::= def_symbol ;
predicate_symbol ::= def_symbol ;

redefine_pred    ::= "redefine" "pred" pred_pattern "means" formula_definiens ";"
                     "coherence" [ "with" label ] justification ";" ;

pred_property    ::= ( "symmetry" | "asymmetry" | "connectedness"
                     | "reflexivity" | "irreflexivity" ) justification ";" ;

predicate_application ::= user_predicate_application
                        | builtin_predicate_application ;
user_predicate_application ::= predicate_segment { predicate_chain_segment } ;
predicate_segment ::= [ term_list ] [ negation ] predicate_head [ term_list ] ;
predicate_chain_segment ::= [ negation ] predicate_head term_list ;
predicate_head ::= predicate_symbol [ template_args ] ;
builtin_predicate_application ::= term_expression builtin_pred term_expression ;
negation              ::= "does" "not" | "do" "not" ;
term_list             ::= term_expression { "," term_expression } ;
builtin_pred          ::= "in" | "=" | "<>" ;
```

## A.10 Functors and Operator Declarations

Normative reference: [Chapter 10 (Functors)](./10.functors.md).

```ebnf
func_def         ::= "func" label ":" func_pattern "->" type_expression
                     ( "means" formula_definiens | "equals" term_definiens ) ";"
                     [ correctness_conditions ] ;
func_pattern     ::= [ loci ] functor_symbol [ template_loci ] [ loci ] ;
functor_symbol   ::= def_symbol ;

func_property    ::= ( "commutativity" | "idempotence"
                     | "involutiveness" | "projectivity" )
                     justification ";" ;

redefine_func    ::= "redefine" "func" label ":" func_pattern
                     "->" type_expression
                     ( "means" formula_definiens | "equals" term_definiens ) ";"
                     "coherence" [ "with" label ] justification ";" ;

operator_decl    ::= infix_operator_decl
                   | prefix_operator_decl
                   | postfix_operator_decl ;
infix_operator_decl   ::= "infix_operator" "(" string_literal ","
                          infix_assoc "," nat_literal ")" ";" ;
prefix_operator_decl  ::= "prefix_operator" "(" string_literal ","
                          nat_literal ")" ";" ;
postfix_operator_decl ::= "postfix_operator" "(" string_literal ","
                          nat_literal ")" ";" ;
infix_assoc           ::= "left" | "right" | "none" ;
```

Functor application syntax is summarized with term expressions in A.13.

## A.11 Symbol Management

Normative reference: [Chapter 11 (Symbol Management)](./11.symbol_management.md).

```ebnf
synonym_def    ::= "synonym" alt_pattern "for" original_pattern ";" ;
antonym_def    ::= "antonym" alt_pattern "for" original_pattern ";" ;

alt_pattern      ::= pred_pattern | func_pattern
                   | alt_mode_pattern | alt_attr_pattern ;
original_pattern ::= pred_pattern | func_pattern
                   | original_mode_pattern | original_attr_pattern ;
alt_mode_pattern      ::= mode_def_name [ type_params ] ;
original_mode_pattern ::= mode_ref_name [ type_params ] ;
alt_attr_pattern      ::= [ param_prefix ] attribute_def_name ;
original_attr_pattern ::= [ param_prefix ] attribute_ref_name ;
```

Import syntax is normalized in A.12. Visibility is represented by
`visibility ::= "private" | "public"` in this appendix; older chapter-local
forms that model visibility as optional `"private"` are compatibility wording.

## A.12 Modules and Namespaces

Normative reference: [Chapter 12 (Modules and Namespaces)](./12.modules_and_namespaces.md).

```ebnf
compilation_unit   ::= import_prelude export_prelude { annotated_declaration } ;

import_prelude     ::= { import_stmt } ;
export_prelude     ::= { export_stmt } ;

import_stmt        ::= "import" module_alias_decl
                       { "," module_alias_decl } ";" ;
export_stmt        ::= "export" module_path { "," module_path } ";" ;

module_alias_decl  ::= module_path [ "as" module_identifier ]
                     | module_branch_import ;
module_branch_import ::= module_path ".{" module_identifier
                       { "," module_identifier } "}" ;
module_path        ::= [ relative_prefix ] module_identifier
                       { "." module_identifier } ;
relative_prefix    ::= "." | ".." ;
module_identifier  ::= identifier ;

annotated_declaration ::= { annotation } declaration
                        | standalone_diagnostic_annotation ;

declaration        ::= definition_block
                     | reserve_decl
                     | registration_block
                     | claim_block
                     | [ visibility ] theorem_item
                     | [ visibility ] notation_decl ;

visibility         ::= "private" | "public" ;

definition_block   ::= "definition" { definition_content } "end" ";" ;
definition_content ::= { annotation }
                       ( definition_parameter_decl
                       | assumption
                       | correctness_condition
                       | property_item
                       | [ visibility ] definitional_item
                       | [ visibility ] theorem_item
                       | [ visibility ] registration_item ) ;

parameter_decl     ::= "let" qualified_vars [ "such" conditions ] [ "by" references ] ";" ;
definition_parameter_decl ::= "let" definition_parameter_binding
                              [ definition_parameter_constraint ] ";" ;
definition_parameter_binding ::= definition_qualified_vars
                               | pred_param
                               | func_param ;
definition_qualified_vars ::= definition_explicit_vars [ "," definition_implicit_vars ]
                            | definition_implicit_vars ;
definition_explicit_vars ::= definition_qualified_segment
                             { "," definition_qualified_segment } ;
definition_qualified_segment ::= identifier_list parameter_qualification ;
definition_implicit_vars ::= identifier_list ;
parameter_qualification ::= ( "being" | "be" ) let_type ;
definition_parameter_constraint ::= "such" conditions
                                  | "such" "that" formula
                                    ( "by" references | proof )
                                  | "by" references ;

definitional_item  ::= struct_def
                     | inherit_def
                     | attr_def
                     | redefine_attr
                     | mode_def
                     | property_impl
                     | pred_def
                     | redefine_pred
                     | func_def
                     | redefine_func
                     | algorithm_def
                     | notation_decl ;

notation_decl      ::= operator_decl | synonym_def | antonym_def ;
property_item      ::= pred_property | func_property | mode_property ;
correctness_condition ::= existence_block
                        | uniqueness_block
                        | coherence_block
                        | compatibility_block
                        | consistency_block
                        | reducibility_block ;
```

## A.13 Term Expressions

Normative reference: [Chapter 13 (Term Expressions)](./13.term_expression.md).

```ebnf
term_expression      ::= operator_expression { "qua" type_expression } ;

operator_expression  ::= postfix_expression | functor_application ;
postfix_expression   ::= term_primary { term_postfix } ;
term_primary         ::= variable_identifier
                       | "it"
                       | numeral
                       | "(" term_expression ")"
                       | struct_constructor
                       | set_expression
                       | choice_expression
                       | inline_functor_application
                       | template_functor_application
                       | bracket_functor_application ;

term_postfix         ::= "." field_name
                         [ "(" [ term_list ] ")" ]
                       | "with" "(" field_update_list ")" ;

variable_identifier ::= identifier ;

inline_functor_application ::= inline_func_name "(" [ term_list ] ")" ;

template_functor_application ::= functor_symbol template_args
                                 [ "(" [ term_list ] ")" ] ;

functor_application  ::= [ functor_loci ] functor_symbol [ functor_loci ] ;
functor_loci         ::= term_expression | "(" term_list ")" ;
bracket_functor_application ::= user_symbol term_list user_symbol
                              | "[" term_list "]" ;

field_update_list    ::= field_update { "," field_update } ;
field_update         ::= selector ":=" term_expression ;

set_expression       ::= set_enumeration | set_comprehension ;
set_enumeration      ::= "{" [ term_list ] "}" ;
set_comprehension    ::= "{" term_expression "where" typed_var_list
                         [ ":" formula ] "}" ;
typed_var_list       ::= typed_var { "," typed_var } ;
typed_var            ::= identifier "is" type_expression ;

choice_expression    ::= "the" type_expression ;
qua_expression       ::= operator_expression "qua" type_expression
                         { "qua" type_expression } ;
```

`operator_expression` is extended by active prefix, infix, postfix, and bracket
functor declarations. Parser implementations should use the precedence and
associativity table from Appendix B instead of expanding those operators as
directly left-recursive EBNF.

## A.14 Formulas

Normative reference: [Chapter 14 (Formulas)](./14.formulas.md).

```ebnf
formula              ::= quantified_formula | iff_formula ;

quantified_formula   ::= universal_formula | existential_formula ;
universal_formula    ::= "for" quantified_vars [ "st" formula ]
                         ( "holds" formula | quantified_formula ) ;
existential_formula  ::= "ex" quantified_vars "st" formula ;

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

atomic_formula       ::= predicate_application
                       | inline_predicate_application
                       | is_assertion ;

inline_predicate_application ::= inline_pred_name "(" [ term_list ] ")" ;
is_assertion                 ::= term_expression "is" [ "not" ]
                                  is_assertion_body ;
is_assertion_body            ::= type_expression | attribute_test_chain ;
attribute_test_chain         ::= [ "non" ] attribute_ref
                                  { [ "non" ] attribute_ref } ;

quantified_vars      ::= explicit_vars [ "," implicit_vars ] | implicit_vars ;
explicit_vars        ::= quantified_segment { "," quantified_segment } ;
quantified_segment   ::= var_list ( "being" | "be" ) type_expression ;
implicit_vars        ::= var_list ;
var_list             ::= identifier { "," identifier } ;
```

`iff` is non-associative. Chaining `iff` without parentheses is a syntax error.
The `is_assertion_body` alternatives can overlap syntactically because type
heads and attributes both use active-lexicon symbols. Parser-facing ASTs should
preserve a generic `is_assertion` until name/type resolution classifies it as a
type assertion or an attribute assertion.

## A.15 Statements, Proofs, and References

Normative references: [Chapter 15 (Statements)](./15.statements.md) and
[Chapter 16 (Theorems and Proofs)](./16.theorems_and_proofs.md).

```ebnf
reasoning           ::= { annotated_statement } ;
proof               ::= "proof" reasoning "end" ;

annotated_statement ::= { annotation } statement ;
statement           ::= [ "then" ] linkable_statement
                      | standalone_statement ;

linkable_statement  ::= compact_statement
                      | conclusion
                      | choice_statement
                      | type_changing_statement
                      | iterative_equality
                      | case_reasoning ;

standalone_statement ::= generalization
                       | constant_definition
                       | inline_functor_definition
                       | inline_predicate_definition
                       | assumption
                       | diffuse_statement
                       | diffuse_conclusion
                       | exemplification
                       | standalone_diagnostic_annotation ;

generalization               ::= "let" qualified_vars [ "such" conditions ] [ "by" references ] ";" ;
constant_definition          ::= "set" equating_list ";" ;
inline_functor_definition    ::= "deffunc" identifier "(" [ typed_params ] ")"
                                 "->" type_expression "equals"
                                 term_expression ";" ;
inline_predicate_definition  ::= "defpred" identifier "(" [ typed_params ] ")"
                                 "means" formula ";" ;
typed_params                 ::= typed_param { "," typed_param } ;
typed_param                  ::= identifier ( "being" | "be" ) type_expression ;

assumption                   ::= single_assumption
                               | collective_assumption
                               | existential_assumption ;
single_assumption            ::= "assume" proposition ";" ;
collective_assumption        ::= "assume" conditions ";" ;
existential_assumption       ::= "given" qualified_vars [ "such" conditions ] ";" ;
choice_statement             ::= "consider" qualified_vars "such"
                                 conditions simple_justification ";" ;

conclusion                   ::= ( "thus" | "hence" ) proposition
                                 justification ";" ;
diffuse_conclusion           ::= "hereby" reasoning "end" ";" ;
exemplification              ::= "take" example_list ";" ;
example_list                 ::= example { "," example } ;
example                      ::= term_expression | identifier "=" term_expression ;

type_changing_statement      ::= "reconsider" type_change_list "as"
                                 type_expression simple_justification ";" ;
(* type_change_list and reconsider_item are defined in A.4. *)

compact_statement            ::= proposition justification ";" ;
iterative_equality           ::= [ label_identifier ":" ] term_expression
                                 "=" term_expression simple_justification
                                 ".=" term_expression simple_justification
                                 { ".=" term_expression simple_justification } ";" ;
diffuse_statement            ::= [ label_identifier ":" ] "now" reasoning "end" ";" ;

case_reasoning               ::= "per" "cases" simple_justification ";"
                                 ( case_list | suppose_list ) ;
case_list                    ::= case_item { case_item } ;
suppose_list                 ::= suppose_item { suppose_item } ;
case_item                    ::= "case" ( proposition | conditions ) ";"
                                 reasoning "end" ";" ;
suppose_item                 ::= "suppose" ( proposition | conditions ) ";"
                                 reasoning "end" ";" ;

proposition                  ::= [ label_identifier ":" ] formula ;
conditions                   ::= "that" proposition { "and" proposition } ;

justification                ::= simple_justification | proof | computation_proof ;
simple_justification         ::= [ "by" references ] ;
references                   ::= reference { "," reference } ;
reference                    ::= label_identifier [ template_args ]
                               | qualified_reference [ template_args ]
                               | grouped_reference
                               | bulk_reference ;
qualified_reference          ::= namespace_path "." label_identifier ;
grouped_reference            ::= namespace_path ".{" grouped_item
                                 { "," grouped_item } "}" ;
grouped_item                 ::= label_identifier [ template_args ] ;
bulk_reference               ::= namespace_path ".*" ;
namespace_path               ::= identifier { "." identifier } ;
template_args                ::= "[" template_arg { "," template_arg } "]" ;
template_arg                 ::= type_expression
                               | term_expression
                               | qua_arg
                               | defpred_identifier
                               | deffunc_identifier ;
(* qua_arg is defined in A.3. *)
```

## A.16 Theorems and Correctness Blocks

Normative reference: [Chapter 16 (Theorems and Proofs)](./16.theorems_and_proofs.md).

```ebnf
theorem_item     ::= [ theorem_status ] theorem_role label_identifier ":"
                     formula [ justification ] ";" ;
theorem_status   ::= "open" | "assumed" | "conditional" ;
theorem_role     ::= "theorem" | "lemma" ;

correctness_conditions ::= { correctness_condition } ;
existence_block        ::= "existence" justification ";" ;
uniqueness_block       ::= "uniqueness" justification ";" ;
coherence_block        ::= "coherence" justification ";" ;
compatibility_block    ::= "compatibility" justification ";" ;
consistency_block      ::= "consistency" justification ";" ;
reducibility_block     ::= "reducibility" justification ";" ;
```

## A.17 Clusters and Registrations

Normative reference: [Chapter 17 (Clusters and Registrations)](./17.clusters_and_registrations.md).

```ebnf
registration_block ::= "registration" { registration_content } "end" ";" ;
registration_content ::= { annotation } ( parameter_decl | registration_item ) ;

registration_item  ::= existential_registration
                     | conditional_registration
                     | functorial_registration
                     | reduction_registration ;

existential_registration ::= "cluster" label ":" adjective_list
                             type_expression ";"
                             "existence" justification ";" ;
conditional_registration ::= "cluster" label ":" antecedent_adjectives
                             "->" consequent_adjectives
                             "for" type_expression ";"
                             "coherence" justification ";" ;
functorial_registration  ::= "cluster" label ":" functor_term
                             "->" consequent_adjectives
                             "for" type_expression ";"
                             "coherence" justification ";" ;
reduction_registration   ::= "reduce" label ":" term_expression
                             "to" term_expression ";"
                             "reducibility" justification ";" ;

adjective_list        ::= adjective { adjective } ;
adjective             ::= [ "non" ] [ param_prefix ] attribute_name ;
antecedent_adjectives ::= adjective_list ;
consequent_adjectives ::= adjective_list ;
functor_term          ::= functor_application ;
```

## A.18 Templates

Normative reference: [Chapter 18 (Templates)](./18.templates.md).

```ebnf
template_definition ::= definition_block ;

template_parameter_decl ::= definition_parameter_decl ;
let_type        ::= "type" [ "extends" bound_type ] | type_expression ;
bound_type      ::= [ attribute_chain ] radix_type ;

template_item   ::= attr_def
                  | mode_def
                  | struct_def
                  | pred_def
                  | func_def
                  | algorithm_def
                  | theorem_item
                  | registration_item ;

pred_param      ::= identifier "be" "pred" "(" type_list ")" ;
func_param      ::= identifier "be" "func" "(" type_list ")" "->"
                    type_expression ;
type_list       ::= type_expression { "," type_expression } ;

param_name      ::= qualified_symbol [ template_args ] ;

scheme_app      ::= "by" scheme_name template_args
                    { "," label_identifier } ";" ;
```

## A.20 Algorithms and Computation

Normative reference: [Chapter 20 (Algorithm and Verification)](./20.algorithm_and_verification.md).

```ebnf
algorithm_def ::= [ "terminating" ] "algorithm" identifier
                  [ "[" schema_params "]" ]
                  "(" [ identifier_list ] ")"
                  [ "->" type_expression ]
                  [ "requires" formula ]
                  [ "ensures" formula ]
                  [ "decreasing" term_list ]
                  algorithm_body ";" ;

schema_params      ::= identifier { "," identifier } ;
algorithm_body     ::= "do" algo_statement_list "end" ;
algo_statement_list ::= { annotated_algo_statement } ;

annotated_algo_statement ::= { annotation } algo_statement ;
algo_statement     ::= var_decl
                     | ghost_var_decl
                     | const_decl
                     | ghost_const_decl
                     | assignment
                     | ghost_assignment
                     | if_stmt
                     | while_stmt
                     | for_range_stmt
                     | for_collection_stmt
                     | match_stmt
                     | return_stmt
                     | break_stmt
                     | continue_stmt
                     | assert_stmt
                     | snapshot_stmt
                     | standalone_diagnostic_annotation ;

var_decl           ::= "var" var_binding { "," var_binding }
                       [ "as" type_expression [ justification ] ] ";" ;
ghost_var_decl     ::= "ghost" "var" var_binding { "," var_binding }
                       [ "as" type_expression [ justification ] ] ";" ;
const_decl         ::= "const" const_binding { "," const_binding }
                       [ "as" type_expression [ justification ] ] ";" ;
ghost_const_decl   ::= "ghost" "const" const_binding { "," const_binding }
                       [ "as" type_expression [ justification ] ] ";" ;
var_binding        ::= identifier [ ":=" term_expression ] ;
const_binding      ::= identifier ":=" term_expression ;

assignment         ::= lvalue ":=" term_expression ";" ;
ghost_assignment   ::= "ghost" lvalue ":=" term_expression ";" ;
lvalue             ::= identifier { "." identifier } ;

if_stmt            ::= "if" formula "do" algo_statement_list if_tail ;
if_tail            ::= "end" ";"
                     | "else" if_stmt
                     | "else" algo_statement_list "end" ";" ;

while_stmt         ::= "while" formula "do"
                       { while_annotation ";" }
                       algo_statement_list "end" ";" ;
while_annotation   ::= "invariant" formula [ justification ]
                     | "decreasing" term_list [ justification ] ;

for_range_stmt     ::= "for" identifier "=" term_expression
                       ( "to" | "downto" ) term_expression
                       [ "step" term_expression ]
                       "do" { for_annotation ";" }
                       algo_statement_list "end" ";" ;
for_collection_stmt ::= "for" identifier "in" term_expression
                        [ "processed" identifier ] "do"
                        { for_annotation ";" }
                        algo_statement_list "end" ";" ;
for_annotation     ::= "invariant" formula [ justification ] ;

match_stmt         ::= "match" term_expression "do"
                       match_case { match_case }
                       ( "otherwise" algo_statement_list "end" ";"
                       | exhaustiveness_proof )
                       "end" ";" ;
match_case         ::= "case" term_pattern "do"
                       algo_statement_list "end" ";" ;
term_pattern       ::= term_expression ;
exhaustiveness_proof ::= "exhaustive" [ justification ] ";" ;

return_stmt        ::= "return" [ term_expression [ justification ] ] ";" ;
break_stmt         ::= "break" ";" ;
continue_stmt      ::= "continue" ";" ;
assert_stmt        ::= "assert" formula [ justification ] ";" ;
snapshot_stmt      ::= "snapshot" identifier ";" ;

claim_block        ::= "claim" identifier "do" { annotated_theorem_item }
                       "end" ";" ;
annotated_theorem_item ::= { annotation } theorem_item ;

computation_proof  ::= "by" "computation"
                       [ "(" computation_option
                         { "," computation_option } ")" ] ;
computation_option ::= "steps" ":" nat_literal
                     | "timeout" ":" nat_literal
                     | "nest" ":" nat_literal ;

pick_expr          ::= "the" type_expression ;
term_size_expr     ::= "term_size" "(" term_expression ")" ;
```

## A.21 Annotations

Normative reference: [Chapter 21 (Source Code Annotation and ATP Integration)](./21.source_code_annotation_and_atp.md).

```ebnf
annotation          ::= library_annotation
                      | statement_annotation
                      | latex_annotation
                      | proof_hint_annotation
                      | show_thesis_annotation
                      | show_resolution_annotation
                      | suppress_annotation ;

statement_annotation ::= generic_annotation_name [ "(" annotation_args ")" ] ;
generic_annotation_name ::= "@" generic_annotation_identifier ;
generic_annotation_identifier ::= ? identifier other than fixed annotation names
                                    latex, proof_hint, show_thesis,
                                    show_resolution, show_type, eval,
                                    suppress ? ;

library_annotation  ::= "@[" label_list "]" ;
label_list          ::= label_name { "," label_name } ;
label_name          ::= label_identifier [ "(" annotation_args ")" ] ;
annotation_args     ::= annotation_arg { "," annotation_arg } ;
annotation_arg      ::= identifier | nat_literal | string_literal ;

latex_annotation    ::= "@latex" "(" string_literal ")" ;

proof_hint_annotation ::= "@proof_hint" "(" proof_hint_options ")" ;
proof_hint_options    ::= proof_hint_option { "," proof_hint_option } ;
proof_hint_option     ::= "max_axioms" ":" nat_literal
                        | "timeout" ":" nat_literal
                        | "solver" ":" solver_name ;
solver_name           ::= "vampire" | "e" | "cvc5" | "z3" | "auto" ;

show_thesis_annotation     ::= "@show_thesis" ;
show_resolution_annotation ::= "@show_resolution" ;
show_type_annotation       ::= "@show_type" "(" term_expression ")" ;
eval_annotation            ::= "@eval" "(" term_expression ")" ;
suppress_annotation        ::= "@suppress" "(" identifier ")" ;
standalone_diagnostic_annotation ::= show_type_annotation | eval_annotation ;
```

`@show_type(...)` and `@eval(...)` are intentionally excluded from
`annotation`; they are parsed only as `standalone_diagnostic_annotation` and do
not attach to a following declaration or statement. Fixed annotation names are
also excluded from `statement_annotation`, so they cannot be reinterpreted as
generic annotations.

## A.22 Chapters Without Additional Surface Grammar

Chapters 8, 19, 22, 23, and 24 primarily define inference, overload
resolution, diagnostics, package/build behavior, and documentation generation.
They rely on the syntax summarized above and do not add parser-owned surface
forms beyond the annotation and reference forms already listed.
