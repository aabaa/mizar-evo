# 付録 A. 文法概要

> Canonical language: English. English canonical version: [../en/appendix_a.grammar_summary.md](../en/appendix_a.grammar_summary.md).

この付録は、parser と surface AST の作業で参照するための、章横断の正本文法サマリです。意味論、well-formedness、可視性、型検査、検証の振る舞いについては本文の各章が引き続き規範です。章本文に古い綴りや局所的な補助名が残っている場合、この付録は parser が従うべき正規化済みの文法向け形式を示します。

## A.1 EBNF 記法と共有 alias

この文法では次の記法を使います。

- `::=` は生成規則を定義します。
- `"token"` は literal token spelling です。予約語と予約記号は A.2 に列挙し、
  文脈限定の spelling は使用箇所で明示します。
- `[ item ]` は省略可能です。
- `{ item }` は 0 回以上の繰り返しです。
- `( a | b )` は選択肢のグループです。
- `? ... ?` は字句上の文字クラスを記述します。

章ごとの局所用語をそろえるための共通 alias は次の通りです。

```ebnf
label              ::= label_identifier ;
nat_literal        ::= numeral ;
term               ::= term_expression ;
expr               ::= term_expression ;
proof_block        ::= proof ;

symbol_name        ::= identifier | user_symbol ;
def_symbol         ::= identifier | user_symbol ;
attribute_def_name ::= def_symbol ;
mode_def_name      ::= def_symbol ;
struct_def_name    ::= def_symbol ;
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

`term_expression` と `formula` は優先順位に基づく文法です。以下の生成規則では可能な範囲で直接左再帰を除いています。term レベルのユーザー演算子と template functor application は parser-facing な表層形式であり、その結合の振る舞いは付録 B と第 10 章・第 13 章で定義される active operator table によって解決されます。

## A.2 語彙構造

規範参照: [第 2 章 (語彙構造)](./02.lexical_structure.md)。

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

line_comment     ::= "::"  { character - newline } newline ;
block_comment    ::= "::=" { character } "=::" ;
doc_comment      ::= ":::" { character - newline } newline ;
```

予約語は大文字小文字を区別し、識別子またはユーザーシンボルとして使えません。

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

予約特殊記号は次の通りです。

```text
,   .   ..   ;   :   :=   (   )   [   ]   {   }   .{
=   <>   &   ->   .=   .*   @[   ...
```

文字列リテラルは、文字列引数を要求する文法位置でのみ認識されます。現在は演算子宣言と文字列値 annotation が該当します。それ以外の位置では、引用符は通常の識別子またはユーザーシンボルの字句解析に参加します。

`.` が複合予約 token または active user symbol として消費されない場合、parser と resolver は、第 2 章で説明される文脈に従って selector access/update または namespace separator として分類します。

固定 annotation name と option name は文脈限定 spelling であり、その文法位置の
外では予約識別子ではありません。現在の文脈限定 spelling は `auto`, `cvc5`,
`e`, `max_axioms`, `solver`, `steps`, `timeout`, `vampire`, `z3`, `result`,
`term_size` と、A.21 で `@` の後に現れる固定 annotation-name spelling です。
`@` marker は annotation のために語彙的に予約されていますが、単独の予約記号
token ではありません。

予約 bracket token `[` と `]` は user symbol ではありませんが、term primary が
期待される位置では、組み込み bracket functor の区切り対として文法上認められます。
template-capable な type、functor、algorithm、definition、reference 名の後では、
同じ token は template 引数を開始します。複合 token `@[` は library annotation を
開始します。

`n-dimensional` や `(m,n)-ary` のような parameterized attribute spelling
に現れるハイフンは、予約 special symbol ではなく、文脈依存の
`param_prefix` separator です。属性位置に限り、lexer/parser はまず、
prefix が scope 内の parameter name、numeral、または parenthesized
parameter list で、suffix が active attribute name として解決できる場合に
`param_prefix` 分割を試みます。この判定に失敗した場合、spelling 全体を
1 つの attribute name として active lexicon に照合します。両方が有効な場合は
`param_prefix` 分割が優先されます。

## A.3 型式

規範参照: [第 3 章 (型システム)](./03.type_system.md)。

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
attribute_ref_name ::= qualified_symbol ;
mode_ref_name     ::= qualified_symbol ;
struct_ref_name   ::= qualified_symbol ;

parameter_list    ::= parameter { "," parameter } ;
parameter         ::= identifier | numeral ;
qualified_symbol  ::= { namespace_segment "." } user_symbol ;
namespace_segment ::= identifier ;
```

`qualified_symbol` の最後の token は active lexicon にある user symbol です。そのため、識別子形の symbol は、その lexicon で利用可能な場合に限って symbol として解析されます。

## A.4 変数と定数

規範参照: [第 4 章 (変数と定数)](./04.variables_and_constants.md)。

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

statement レベルの形式は A.15 で正規化します。

## A.5 構造体

規範参照: [第 5 章 (構造体)](./05.structures.md)。

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

## A.6 属性

規範参照: [第 6 章 (属性)](./06.attributes.md)。

```ebnf
attr_def       ::= "attr" label ":" subject "is" attr_pattern
                   "means" formula_definiens ";" ;
subject        ::= identifier ;
attr_pattern   ::= [ param_prefix ] attribute_def_name ;

redefine_attr  ::= "redefine" "attr" label ":" subject "is" attr_pattern
                   "means" formula_definiens ";"
                   "coherence" [ "with" label ] justification ";" ;
```

型式内での属性利用は A.3 にまとめています。

## A.7 モード

規範参照: [第 7 章 (モード)](./07.modes.md)。

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

## A.9 述語

規範参照: [第 9 章 (述語)](./09.predicates.md)。

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

## A.10 functor と演算子宣言

規範参照: [第 10 章 (functor)](./10.functors.md)。

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

functor application の構文は term expression とともに A.13 にまとめています。

## A.11 シンボル管理

規範参照: [第 11 章 (シンボル管理)](./11.symbol_management.md)。

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

import 構文は A.12 で正規化します。この付録では可視性を `visibility ::= "private" | "public"` として表します。可視性を省略可能な `"private"` として扱う古い章内表現は互換用の説明です。

## A.12 モジュールと namespace

規範参照: [第 12 章 (モジュールと namespace)](./12.modules_and_namespaces.md)。

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

## A.13 項式

規範参照: [第 13 章 (項式)](./13.term_expression.md)。

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

`operator_expression` は、active な prefix、infix、postfix、bracket functor 宣言によって拡張されます。parser 実装では、これらの演算子を直接左再帰 EBNF として展開せず、付録 B の優先順位・結合性 table を使います。

## A.14 論理式

規範参照: [第 14 章 (論理式)](./14.formulas.md)。

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

`iff` は非結合です。括弧なしで `iff` を連鎖させると構文エラーです。
`is_assertion_body` の選択肢は、type head と attribute がいずれも
active-lexicon symbol を使うため、構文上重なりえます。parser-facing AST は、
name / type resolution が type assertion または attribute assertion に分類するまで、
generic な `is_assertion` を保持するべきです。

## A.15 文、証明、参照

規範参照: [第 15 章 (文)](./15.statements.md) および [第 16 章 (定理と証明)](./16.theorems_and_proofs.md)。

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

## A.16 定理と correctness block

規範参照: [第 16 章 (定理と証明)](./16.theorems_and_proofs.md)。

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

## A.17 cluster と registration

規範参照: [第 17 章 (cluster と registration)](./17.clusters_and_registrations.md)。

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

## A.18 template

規範参照: [第 18 章 (template)](./18.templates.md)。

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

## A.20 algorithm と computation

規範参照: [第 20 章 (algorithm と検証)](./20.algorithm_and_verification.md)。

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

## A.21 annotation

規範参照: [第 21 章 (ソースコード annotation と ATP 連携)](./21.source_code_annotation_and_atp.md)。

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

`@show_type(...)` と `@eval(...)` は意図的に `annotation` から除外されています。
これらは `standalone_diagnostic_annotation` としてのみ解析され、後続の宣言や文には
付与されません。固定 annotation 名は `statement_annotation` からも除外されるため、
generic annotation として再解釈されることはありません。

## A.22 追加の表層文法を持たない章

第 8 章、第 19 章、第 22 章、第 23 章、第 24 章は、主に推論、overload resolution、診断、package/build、documentation generation を定義します。これらは上でまとめた構文に依存し、すでに列挙した annotation と reference 形式を超える parser-owned surface form は追加しません。
