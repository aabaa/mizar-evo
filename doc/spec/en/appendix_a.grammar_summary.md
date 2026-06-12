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
- `"token"` is a reserved word or reserved symbol token.
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
by_refs            ::= "by" references ";" ;
proof_or_refs      ::= proof | by_refs ;

symbol_name        ::= identifier | user_symbol ;
def_symbol         ::= identifier | user_symbol ;
field_name         ::= identifier ;
selector           ::= field_name { "." field_name } ;
inline_func_name   ::= identifier ;
inline_pred_name   ::= identifier ;
deffunc_identifier ::= identifier ;
defpred_identifier ::= identifier ;
scheme_name        ::= label_identifier ;
```

`term_expression` and `formula` are precedence-driven grammars. The productions
below remove direct left recursion where possible, but term-level user operators
are still parsed by the active operator table described in Appendix B and
Chapters 10 and 13.

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

numeral          ::= digit { digit } ;

string_literal   ::= dq_string | sq_string ;
dq_string        ::= '"' { dq_char | escape_seq } '"' ;
sq_string        ::= "'" { sq_char | escape_seq } "'" ;
dq_char          ::= ? any character except '"' or '\' ? ;
sq_char          ::= ? any character except "'" or '\' ? ;
escape_seq       ::= "\" ( '"' | "'" | "\" ) ;

symbol_char      ::= ? any ASCII graphic character except "@" and whitespace ? ;
user_symbol      ::= symbol_char { symbol_char } ;

line_comment     ::= "::"  { character - newline } newline ;
block_comment    ::= "::=" { character } "=::" ;
doc_comment      ::= ":::" { character - newline } newline ;
```

Reserved words are case-sensitive and cannot be identifiers or user symbols.

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
set sethood snapshot st struct such suppose symmetry synonym
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
the parser and resolver classify it as selector access/update or as a namespace
separator according to the context described in Chapter 2.

## A.3 Type Expressions

Normative reference: [Chapter 3 (Type System)](./03.type_system.md).

```ebnf
type_expression   ::= attribute_chain type_head ;
type_head         ::= radix_type | mode_type ;

attribute_chain   ::= { [ "non" ] attribute_ref } ;
attribute_ref     ::= [ param_prefix ] [ struct_name "." ] attribute_name
                      [ "(" argument_list ")" ] ;
param_prefix      ::= parameter "-" | "(" parameter_list ")" "-" ;

radix_type        ::= builtin_type | struct_name [ type_args ] ;
mode_type         ::= mode_name [ type_args ] ;
type_args         ::= ( "of" | "over" ) argument_list ;
argument_list     ::= term_expression { "," term_expression } ;

builtin_type      ::= "object" | "set" ;
attribute_name    ::= qualified_symbol ;
mode_name         ::= qualified_symbol ;
struct_name       ::= qualified_symbol ;

parameter_list    ::= parameter { "," parameter } ;
parameter         ::= identifier | numeral ;
qualified_symbol  ::= { namespace_segment "." } user_symbol ;
namespace_segment ::= identifier ;
```

The final token of `qualified_symbol` is a user symbol in the active lexicon.
Identifier-shaped symbols are therefore parsed as symbols only when they are
available in that lexicon.

## A.4 Variables and Constants

Normative reference: [Chapter 4 (Variables and Constants)](./04.variables_and_constants.md).

```ebnf
reserve_decl       ::= "reserve" reserve_segment ";" ;
reserve_segment    ::= identifier_list "for" type_expression ;

identifier_list    ::= identifier { "," identifier } ;

let_decl           ::= "let" qualified_vars [ "such" conditions ] ";" ;
qualified_vars     ::= identifier_list [ "be" type_expression ] ;

set_decl           ::= "set" identifier "=" term_expression ";" ;

reconsider_decl    ::= "reconsider" reconsider_target "as" type_expression
                       [ simple_justification ] ";" ;
reconsider_target  ::= identifier "=" term_expression | term_expression ;

quantified_var     ::= identifier_list [ "being" type_expression ] ;
```

Statement-level forms are normalized in A.15.

## A.5 Structures

Normative reference: [Chapter 5 (Structures)](./05.structures.md).

```ebnf
struct_def       ::= "struct" struct_name [ type_params ] "where"
                     struct_member { struct_member } "end" ";" ;
struct_member    ::= field_decl | property_decl ;
field_decl       ::= "field" identifier "->" type_expression
                     [ ":=" term_expression ] ";" ;
property_decl    ::= "property" identifier "->" type_expression ";" ;
type_params      ::= ( "of" | "over" ) type_parameter_list ;
type_parameter_list ::= identifier { "," identifier } ;

inherit_def      ::= "inherit" struct_name "extends" parent_type
                     [ "where" inherit_member { inherit_member }
                       [ coherence_block ] "end" ] ";" ;
parent_type      ::= struct_name | "set" ;
inherit_member   ::= field_redef | property_redef ;
field_redef      ::= "field" identifier [ "->" type_expression ]
                     "from" ( identifier | "it" ) ";" ;
property_redef   ::= "property" identifier [ "->" type_expression ]
                     "from" identifier ";" ;

struct_constructor ::= struct_name "(" [ named_arg { "," named_arg } ] ")" ;
named_arg          ::= identifier ":" term_expression ;
field_access       ::= term_expression "." field_name ;
```

## A.6 Attributes

Normative reference: [Chapter 6 (Attributes)](./06.attributes.md).

```ebnf
attr_def       ::= "attr" label ":" subject "is" attr_pattern
                   "means" formula ";" ;
subject        ::= identifier ;
attr_pattern   ::= [ param_prefix ] attribute_name ;

redefine_attr  ::= "redefine" "attr" label ":" subject "is" attr_pattern
                   "means" formula ";"
                   "coherence" [ "with" label ] proof_or_refs ;
```

Attribute use inside type expressions is summarized in A.3.

## A.7 Modes

Normative reference: [Chapter 7 (Modes)](./07.modes.md).

```ebnf
mode_def       ::= "mode" label ":" mode_name [ type_params ] "is"
                   attribute_chain radix_type ";"
                   [ mode_property ] ;
mode_property  ::= "sethood" justification ";" ;

property_impl        ::= "definition"
                           "let" identifier "be" mode_name ";"
                           ( property_means_impl | property_equals_impl )
                         "end" ";" ;
property_means_impl  ::= "property" identifier "." identifier "means" formula ";"
                         existence_block uniqueness_block ;
property_equals_impl ::= "property" identifier "." identifier
                         "equals" term_expression ";" ;

mode_application     ::= mode_name [ type_args ] ;
```

## A.9 Predicates

Normative reference: [Chapter 9 (Predicates)](./09.predicates.md).

```ebnf
pred_def         ::= "pred" label ":" pred_pattern "means" formula ";" ;
pred_pattern     ::= [ loci ] predicate_symbol [ loci ] ;
loci             ::= locus | "(" locus_list ")" ;
locus_list       ::= locus { "," locus } ;
locus            ::= identifier ;
predicate_symbol ::= def_symbol ;

redefine_pred    ::= "redefine" "pred" pred_pattern "means" formula ";"
                     "coherence" [ "with" label ] proof_or_refs ;

pred_property    ::= ( "symmetry" | "asymmetry" | "connectedness"
                     | "reflexivity" | "irreflexivity" ) justification ";" ;

predicate_application ::= [ term_list ] [ negation ] predicate_symbol [ term_list ] ;
negation              ::= "does" "not" | "do" "not" ;
term_list             ::= term_expression { "," term_expression } ;
builtin_pred          ::= "in" | "=" | "<>" ;
```

## A.10 Functors and Operator Declarations

Normative reference: [Chapter 10 (Functors)](./10.functors.md).

```ebnf
func_def         ::= "func" label ":" func_pattern "->" type_expression
                     ( "means" formula | "equals" term_expression ) ";"
                     [ correctness_conditions ] ;
func_pattern     ::= [ loci ] functor_symbol [ loci ] ;
functor_symbol   ::= def_symbol ;

func_property    ::= ( "commutativity" | "idempotence"
                     | "involutiveness" | "projectivity" )
                     justification ";" ;

redefine_func    ::= "redefine" "func" label ":" func_pattern
                     "->" type_expression
                     ( "means" formula | "equals" term_expression ) ";"
                     "coherence" [ "with" label ] proof_or_refs ;

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

alt_pattern      ::= pred_pattern | func_pattern | mode_pattern | attr_pattern ;
original_pattern ::= pred_pattern | func_pattern | mode_pattern | attr_pattern ;
mode_pattern     ::= mode_name [ type_params ] ;
```

Import syntax is normalized in A.12. Visibility is represented by
`visibility ::= "private" | "public"` in this appendix; older chapter-local
forms that model visibility as optional `"private"` are compatibility wording.

## A.12 Modules and Namespaces

Normative reference: [Chapter 12 (Modules and Namespaces)](./12.modules_and_namespaces.md).

```ebnf
compilation_unit   ::= import_prelude export_prelude { declaration } ;

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

declaration        ::= definition_block
                     | reserve_decl
                     | template_def
                     | registration_block
                     | claim_block
                     | [ visibility ] theorem_item
                     | [ visibility ] notation_decl ;

visibility         ::= "private" | "public" ;

definition_block   ::= "definition" { definition_content } "end" ";" ;
definition_content ::= parameter_decl
                     | assumption
                     | correctness_condition
                     | property_item
                     | [ visibility ] definitional_item ;

parameter_decl     ::= "let" variable_list [ qualification ]
                       [ "such" conditions ] ";" ;

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

operator_expression  ::= term_primary { term_postfix } ;
term_primary         ::= variable_identifier
                       | "it"
                       | numeral
                       | "(" term_expression ")"
                       | struct_constructor
                       | set_expression
                       | choice_expression
                       | inline_functor_application
                       | bracket_functor_application ;

term_postfix         ::= "." field_name
                       | "with" "(" field_update_list ")" ;

variable_identifier ::= identifier ;

inline_functor_application ::= inline_func_name "(" [ term_list ] ")" ;

functor_application  ::= operator_expression ;
functor_loci         ::= locus | "(" term_list ")" ;
bracket_functor_application ::= user_symbol term_list user_symbol ;

field_update_list    ::= field_update { "," field_update } ;
field_update         ::= selector ":" term_expression ;

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

iff_formula          ::= implies_formula [ "iff" implies_formula ] ;
implies_formula      ::= or_formula [ "implies" implies_formula ] ;
or_formula           ::= and_formula
                         { "or" and_formula
                         | "or" "..." "or" and_formula } ;
and_formula          ::= not_formula
                         { "&" not_formula
                         | "&" "..." "&" not_formula } ;
not_formula          ::= "not" not_formula
                       | atomic_formula
                       | "(" formula ")"
                       | "contradiction"
                       | "thesis" ;

atomic_formula       ::= predicate_application
                       | inline_predicate_application
                       | type_assertion
                       | attribute_assertion ;

inline_predicate_application ::= inline_pred_name "(" [ term_list ] ")" ;
type_assertion               ::= term_expression "is" type_expression ;
attribute_assertion          ::= term_expression "is" [ "not" ] attribute_chain ;

quantified_vars      ::= explicit_vars [ "," implicit_vars ] | implicit_vars ;
explicit_vars        ::= quantified_segment { "," quantified_segment } ;
quantified_segment   ::= var_list ( "being" | "be" ) type_expression ;
implicit_vars        ::= var_list ;
var_list             ::= identifier { "," identifier } ;
```

`iff` is non-associative. Chaining `iff` without parentheses is a syntax error.

## A.15 Statements, Proofs, and References

Normative references: [Chapter 15 (Statements)](./15.statements.md) and
[Chapter 16 (Theorems and Proofs)](./16.theorems_and_proofs.md).

```ebnf
reasoning           ::= { statement } ;
proof               ::= "proof" reasoning "end" ;

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
                       | exemplification ;

generalization               ::= "let" variable_list [ qualification ]
                                 [ "such" conditions ] ";" ;
constant_definition          ::= "set" identifier "=" term_expression ";" ;
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
existential_assumption       ::= "given" variable_list [ qualification ]
                                 [ "such" conditions ] ";" ;
choice_statement             ::= "consider" identifier qualification "such"
                                 conditions simple_justification ";" ;

conclusion                   ::= ( "thus" | "hence" ) proposition
                                 justification ";" ;
diffuse_conclusion           ::= "hereby" reasoning "end" ;
exemplification              ::= "take" example ";" ;
example                      ::= term_expression | identifier "=" term_expression ;

type_changing_statement      ::= "reconsider" reconsider_item "as"
                                 type_expression simple_justification ";" ;
reconsider_item              ::= identifier | identifier "=" term_expression ;

compact_statement            ::= proposition justification ";" ;
iterative_equality           ::= [ label_identifier ":" ] term_expression
                                 "=" term_expression simple_justification
                                 { ".=" term_expression simple_justification } ";" ;
diffuse_statement            ::= [ label_identifier ":" ] "now" reasoning "end" ;

case_reasoning               ::= "per" "cases" simple_justification ";"
                                 ( case_list | suppose_list ) ;
case_list                    ::= case_item { case_item } ;
suppose_list                 ::= suppose_item { suppose_item } ;
case_item                    ::= "case" ( proposition | conditions ) ";"
                                 reasoning "end" ;
suppose_item                 ::= "suppose" ( proposition | conditions ) ";"
                                 reasoning "end" ;

proposition                  ::= [ label_identifier ":" ] formula ;
conditions                   ::= "that" proposition { "and" proposition } ;
variable_list                ::= identifier { "," identifier } ;
qualification                ::= ( "being" | "be" ) type_expression ;

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
                               | coercion_arg
                               | defpred_identifier
                               | deffunc_identifier ;
coercion_arg                 ::= identifier "as" radix_type ;
```

## A.16 Theorems and Correctness Blocks

Normative reference: [Chapter 16 (Theorems and Proofs)](./16.theorems_and_proofs.md).

```ebnf
theorem_item     ::= [ theorem_status ] theorem_role label_identifier ":"
                     formula [ justification ] ";" ;
theorem_status   ::= "open" | "assumed" | "conditional" ;
theorem_role     ::= "theorem" | "lemma" ;

correctness_conditions ::= { correctness_condition } ;
existence_block        ::= "existence" proof_or_refs ;
uniqueness_block       ::= "uniqueness" proof_or_refs ;
coherence_block        ::= "coherence" proof_or_refs ;
compatibility_block    ::= "compatibility" proof_or_refs ;
consistency_block      ::= "consistency" proof_or_refs ;
reducibility_block     ::= "reducibility" proof_or_refs ;
```

## A.17 Clusters and Registrations

Normative reference: [Chapter 17 (Clusters and Registrations)](./17.clusters_and_registrations.md).

```ebnf
registration_block ::= "registration" { registration_content } "end" ";" ;
registration_content ::= parameter_decl | registration_item ;

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
template_def    ::= "definition" { let_param } { template_item } "end" ";" ;

let_param       ::= "let" identifier_list "be" let_type
                    [ "such" "that" formula
                      ( "by" references | proof ) ] ";" ;
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

param_name      ::= identifier [ "[" type_arg_list "]" ] ;
type_arg_list   ::= type_arg { "," type_arg } ;
type_arg        ::= type_expression | qua_arg ;
qua_arg         ::= identifier { "qua" radix_type } ;

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
algo_statement_list ::= { algo_statement } ;

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
                     | claim_block ;

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

claim_block        ::= "claim" identifier "do" { theorem_item } "end" ";" ;

computation_proof  ::= "by" "computation"
                       [ "(" computation_option
                         { "," computation_option } ")" ] ;
computation_option ::= "steps" ":" nat_literal
                     | "timeout" ":" nat_literal
                     | "nest" ":" nat_literal ;

pick_expr          ::= "the" type_expression ;
```

## A.21 Annotations

Normative reference: [Chapter 21 (Source Code Annotation and ATP Integration)](./21.source_code_annotation_and_atp.md).

```ebnf
annotation_name     ::= "@" identifier ;
annotation          ::= library_annotation
                      | statement_annotation
                      | latex_annotation
                      | proof_hint_annotation
                      | show_thesis_annotation
                      | show_resolution_annotation
                      | show_type_annotation
                      | eval_annotation
                      | suppress_annotation ;

statement_annotation ::= annotation_name [ "(" annotation_args ")" ] ;

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
```

## A.22 Chapters Without Additional Surface Grammar

Chapters 8, 19, 22, 23, and 24 primarily define inference, overload
resolution, diagnostics, package/build behavior, and documentation generation.
They rely on the syntax summarized above and do not add parser-owned surface
forms beyond the annotation and reference forms already listed.
