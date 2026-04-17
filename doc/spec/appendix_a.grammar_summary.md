# Appendix A. Grammar Summary

This appendix consolidates the EBNF productions and lexical rules introduced throughout the main specification. Section numbers match the chapter in which the rule is normatively defined; the text here is a reference summary, not a redefinition.

* [A. Grammar Summary](#appendix-a-grammar-summary)
  * [A.2 Lexical Structure](#a2-lexical-structure)
    * [A.2.1 Character Set](#a21-character-set)
    * [A.2.2 Whitespace](#a22-whitespace)
    * [A.2.3 Token Categories](#a23-token-categories)
    * [A.2.4 Reserved Words](#a24-reserved-words)
    * [A.2.5 Special Symbols](#a25-special-symbols)
    * [A.2.6 Identifiers](#a26-identifiers)
    * [A.2.7 Numerals and String Literals](#a27-numerals-and-string-literals)
    * [A.2.8 File and Module Naming](#a28-file-and-module-naming)
    * [A.2.9 Comments and Annotations](#a29-comments-and-annotations)
    * [A.2.10 Lexer / Parser Responsibility Split](#a210-lexer--parser-responsibility-split)
  * [A.3 Type System](#a3-type-system)
    * [A.3.1 Type Categories](#a31-type-categories)
    * [A.3.2 Type Expression Grammar](#a32-type-expression-grammar)
    * [A.3.3 Built-in Types](#a33-built-in-types)
    * [A.3.4 Subtyping Semantics](#a34-subtyping-semantics)


## A.2 Lexical Structure

Normative reference: [Chapter 2 (Lexical Structure)](./02.lexical_structure.md).

### A.2.1 Character Set

* Source files are encoded in **UTF-8**.
* **Code regions** use ASCII only.
* **Comments and annotations** may contain full Unicode.
* Backslash `\` is the escape character inside string literals (§A.2.7).

### A.2.2 Whitespace

```ebnf
whitespace    = " " | tab | newline ;
tab           = ? ASCII 0x09 ? ;
newline       = ? ASCII 0x0A | 0x0D 0x0A ? ;
```

Whitespace separates tokens and is otherwise non-significant except inside string literals.

### A.2.3 Token Categories

A Mizar source file, after comment stripping (§A.2.9) and macro-level import resolution, is a sequence of tokens drawn from five categories:

1. **Reserved words** (§A.2.4) — fixed keywords.
2. **Special symbols** (§A.2.5) — reserved punctuation and user-defined symbolic names.
3. **Identifiers** (§A.2.6) — user-chosen alphanumeric names.
4. **Numerals** (§A.2.7) — unsigned integer literals.
5. **String literals** (§A.2.7) — quoted character sequences, recognised only at grammar positions that require them.

Tokenization is governed by the **longest-match rule** against the currently active lexicon. The active lexicon is extended dynamically by `import` statements (§A.2.10).

### A.2.4 Reserved Words

Reserved words are case-sensitive and cannot be used as identifiers or user symbols.

```
algorithm and antonym as assert assume asymmetry attr
be being break by
case cases claim cluster coherence commutativity compatibility computation
conjecture connectedness const consider consistency continue contradiction
decreasing deffunc definition defpred do does downto
else end ensures equals ex exhaustive existence export extends
field for from func
ghost given
hence hereby holds
idempotence if iff implies import in infix_operator inherit invariant
involutiveness irreflexivity is it
left let
match means mode
nest non none not now
object of or otherwise over
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

### A.2.5 Special Symbols

#### Reserved special symbols

```
,   .   ;   :   :=   (   )   [   ]   {   }   .{
=   <>   &   ->   .=   .*   @[   ...
```

| Token | Role(s) |
|---|---|
| `,` `;` `:` | Delimiters |
| `:=` | Assignment — field update (§13.3.3), variable initialization (§20.3.1), algorithm assignment (§20.4) |
| `.` | Compound-token opener, selector access / update, namespace separator, or user-registered binary functor (§A.2.5 Dot Disambiguation) — the sole reserved symbol that may also be redefined by users |
| `.*` | Bulk citation (§16.5) |
| `.{` … `}` | Grouped citation (§16.5); `}` doubles as the set/Fraenkel closer |
| `@[` … `]` | Library annotation (§21.2); `]` doubles as the list/index closer |
| `=` `<>` `.=` | Equality, built-in inequality, stepwise transformation |
| `->` | Function arrow |
| `...` | Ellipsis |
| `(` `)` `[` `]` `{` `}` `&` | Standard grouping and conjunction |

#### User-defined symbolic names

```ebnf
symbol_char = ? any ASCII graphic character except "@" and whitespace ? ;
user_symbol = symbol_char { symbol_char } ;
```

Any ASCII graphic except `@` is admissible. A user symbol must not coincide exactly with a reserved word or with a reserved special symbol other than `.`. Longest-match resolves ambiguities; later imports shadow earlier ones on ties.

#### Dot disambiguation (parser-side)

Under the lexer/parser split adopted in §A.2.10, `.` is emitted as a single `DOT` token and the parser resolves its role in priority order:

1. **Compound reserved tokens** — `.{`, `.*`, `.=`, `...` are recognised by the lexer directly when the required follow-character is present.
2. **User-defined symbols containing `.`** — e.g. `|.`, `.|`, `|. .|`, or a binary functor `.` giving the classical Mizar form `f.x`. The lexer recognises the registered symbol via longest-match against the active lexicon.
3. **Selector access / update** — `DOT` immediately following a term expression denotes field access (`p.x`, `line.end.y`; §5.7, §13.3.2) or, as the left-hand side of `:=`, a field update (§13.3.3).
4. **Namespace separator** — `DOT` between identifiers in a path used as a module reference (inside `import`, citation `by`-clauses, `@[...]`, or a qualified name before any term context is established) separates namespace components (§A.2.8).

A namespace component that is also in scope as a variable resolves as the variable; the following `.` is then selector access, not a namespace separator.

### A.2.6 Identifiers

```ebnf
identifier = ( letter | "_" ) { letter | digit | "_" | "'" } ;
letter     = "a"..."z" | "A"..."Z" ;
digit      = "0"..."9" ;
```

An identifier must not match a reserved word. Identifiers are case-sensitive. A token whose shape matches both the identifier grammar and a registered user symbol is classified as a symbol iff it appears in the active lexicon.

### A.2.7 Numerals and String Literals

#### Numerals

```ebnf
numeral = digit { digit } ;
```

No built-in floating-point, boolean, or other literal types; non-integer values are encoded as library terms.

#### String literals

```ebnf
string_literal  = dq_string | sq_string ;
dq_string       = '"' { dq_char | escape_seq } '"' ;
sq_string       = "'" { sq_char | escape_seq } "'" ;
dq_char         = ? any character except '"' or '\' ? ;
sq_char         = ? any character except "'" or '\' ? ;
escape_seq      = "\" ( '"' | "'" | "\" ) ;
```

**Contextual recognition**: `"` and `'` are tokenized as string delimiters **only at grammar positions that require a string literal**. At every other position they participate in ordinary lexing as parts of identifiers or user-defined symbols. In particular, the postfix inverse operator `f"` (§11) is a user-symbol use of `"`, not a string delimiter.

Grammar positions currently requiring a string literal:

| Position | Reference |
|---|---|
| `infix_operator STRING : ...`, `prefix_operator STRING : ...`, `postfix_operator STRING : ...` — first argument | §10.7, §13 |
| `@latex(STRING)` and other string-valued annotation arguments | §21 |

### A.2.8 File and Module Naming

* Files end with `.miz`.
* Each file defines one module; module name equals file name without extension.
* Namespace is derived from the file's path relative to the package's `src/` root. Package name (from `mizar.pkg`) is the namespace root; each intermediate directory under `src/` contributes one dotted component. See [§23.3](./23.package_management_and_build_system.md#233-workspace-layout).

Example — package `algebra`, file `algebra/src/groups/basic.miz`:

```
Module name:  basic
Namespace:    algebra.groups.basic
```

### A.2.9 Comments and Annotations

```ebnf
line_comment   = "::"  { character - newline } newline ;
block_comment  = "::=" { character } "=::" ;
doc_comment    = ":::" { character - newline } newline ;

annotation_name  = "@" identifier ;
library_annot    = "@[" label_list "]" ;
label_list       = label { "," label } ;
```

* Comments are removed before parsing (§A.2.10).
* `@` must be immediately followed by the identifier (no whitespace).
* Annotation names use `snake_case`; the registry is fixed and not extensible by import.
* Three annotation contexts: statement-level, documentation tags inside `:::` comments, and bracket-form library references `@[...]`.

### A.2.10 Lexer / Parser Responsibility Split

Lexing and parsing are split as follows (see §2 discussion):

| Concern | Layer |
|---|---|
| Comment stripping | Preprocessing |
| Import resolution | Preprocessing (before tokenization) |
| Reserved words / reserved special symbols | Lexer |
| User-symbol recognition (active lexicon, longest-match) | Lexer |
| Numeral recognition | Lexer |
| `"` / `'` string-literal recognition | Parser-assisted (enabled only at string-requiring positions) |
| `.` role assignment (selector vs namespace vs compound vs user functor) | Parser + name resolver |
| Variable shadowing of namespace paths | Name resolver |

The lexer emits a uniform token stream; context-dependent interpretation of `"`, `'`, and `.` is performed by the parser and name resolver against the active lexicon and the current scope.


## A.3 Type System

Normative reference: [Chapter 3 (Type System)](./03.type_system.md). Mizar uses a **soft type system** layered on untyped set theory: types guide checking and readability but are erased for the logical core.

### A.3.1 Type Categories

| Category | Role | Defined In |
|---|---|---|
| Radix-types | Root of the type hierarchy — built-ins (`object`, `set`) and user-defined structures | §A.3.3, Ch.5 |
| Mode-types | Named types; each unfolds to `attribute_chain radix_type` | Ch.7 |
| Attributes | Predicates that refine a type | Ch.6 |
| Clusters | Registration mechanism for type inference | Ch.17 |

Radix-types and mode-types form two disjoint syntactic categories, both usable at the head of a type expression.

### A.3.2 Type Expression Grammar

```ebnf
type_expression   = attribute_chain type_head ;
type_head         = radix_type | mode_type ;
attribute_chain   = { [ "non" ] attribute_ref } ;
attribute_ref     = [ param_prefix ] [ struct_name "." ] attribute_name ;
param_prefix      = parameter "-" | "(" parameter_list ")" "-" ;

radix_type        = builtin_type | struct_name [ type_args ] ;
mode_type         = mode_name [ type_args ] ;

type_args         = ( "of" | "over" ) argument_list ;
argument_list     = term_expression { "," term_expression } ;

builtin_type      = "object" | "set" ;
attribute_name    = user_symbol ;   (* registered by Ch.6 *)
mode_name         = user_symbol ;   (* registered by Ch.7 *)
struct_name       = user_symbol ;   (* registered by Ch.5 *)
```

* `struct_name "." attribute_name` is a struct-qualified attribute reference; the `.` follows the namespace-separator rule of §A.2.5 (Dot Disambiguation, rule 4) and is shadowed by any in-scope variable of the same name.
* `attribute_name`, `mode_name`, and `struct_name` are `user_symbol`s (§A.2.5.2) registered in the active lexicon by their defining chapters; a name matches these positions only when present in that lexicon.
* `parameter` and `parameter_list` are template parameters defined in Ch.18.
* `type_args` and `param_prefix` have alternative bracket-form productions introduced by Ch.18; those forms extend, but do not replace, the productions above.

### A.3.3 Built-in Types

| Type | Description |
|---|---|
| `object` | Universal type — every value in the Mizar universe, including structures |
| `set` | ZFC-style mathematical set; subtype of `object` |

`struct`-defined types are subtypes of `object` but not of `set`.

### A.3.4 Subtyping Semantics

"`S` is a subtype of `T`" means every member of `S` is a member of `T`, encoded in FOL as `∀x. is_S(x) ⇒ is_T(x)`. Widening (to a supertype) is automatic; narrowing (to a subtype) requires `reconsider` (Ch.15) with a proof obligation.

Types are erased when exporting to ATPs: variable declarations become untyped, type assumptions become hypotheses, and attribute chains become conjunctions.
