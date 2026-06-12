# Appendix B. Operator Precedence

> Canonical language: English. Japanese companion: [../ja/appendix_b.operator_precedence.md](../ja/appendix_b.operator_precedence.md).

This appendix summarizes the precedence and associativity rules used when parsing term expressions and formulas. The normative definitions remain in the main chapters: symbolic functor declarations are specified in [Chapter 10, §10.9](./10.functors.md#109-operator-precedence-and-associativity) and [Chapter 13, §13.7](./13.term_expression.md#137-operator-precedence-and-associativity); formula precedence is specified in [Chapter 14, §14.6](./14.formulas.md#146-precedence-and-associativity).

* [B. Operator Precedence](#appendix-b-operator-precedence)
  * [B.1 Precedence Domains](#b1-precedence-domains)
  * [B.2 Term Operators](#b2-term-operators)
  * [B.3 Atomic Formula Boundary](#b3-atomic-formula-boundary)
  * [B.4 Formula Operators](#b4-formula-operators)
  * [B.5 Recommended Parsing Algorithm](#b5-recommended-parsing-algorithm)
  * [B.6 Diagnostics](#b6-diagnostics)

## B.1 Precedence Domains

Mizar Evolution uses two precedence domains:

| Domain | Applies To | Precedence Source |
|---|---|---|
| Term expressions | User-defined functors, prefix/postfix/infix notation, selector access, `qua`, parenthesized terms | Operator declarations and fixed term syntax |
| Formulas | Atomic formulas, `not`, `&`, `or`, `implies`, `iff`, quantifiers | Fixed language-defined hierarchy |

Term precedence is resolved before formula-level connectives are parsed. For example, `a + b = c + d` is parsed as the atomic formula `(a + b) = (c + d)`, and that atomic formula then participates in formula-level precedence.

Parentheses override precedence in both domains.

## B.2 Term Operators

Symbolic functors may declare parsing metadata using standalone operator declarations:

```mizar
infix_operator("+", left, 80);
infix_operator("*", left, 90);
infix_operator("^", right, 95);
prefix_operator("-", 85);
postfix_operator("!", 95);
```

| Declaration | Meaning |
|---|---|
| `infix_operator("sym", left, N);` | Binary infix operator, left-associative at precedence `N` |
| `infix_operator("sym", right, N);` | Binary infix operator, right-associative at precedence `N` |
| `infix_operator("sym", none, N);` | Binary infix operator, non-associative at precedence `N` |
| `prefix_operator("sym", N);` | Prefix operator at precedence `N` |
| `postfix_operator("sym", N);` | Postfix operator at precedence `N` |

Precedence values range from `0` to `255`; higher values bind more tightly. The default precedence for a symbolic functor with no declaration is `64`, non-associative.

Built-in predicate symbols such as `=`, `<>`, and `in` are declared in the implicitly imported core module. They cannot be overridden by user declarations.

The `qua` type-qualification operator is fixed by the language rather than declared by `infix_operator`. It has the lowest term-level precedence and is left-associative, so `R qua AddGroup qua Magma` parses as `(R qua AddGroup) qua Magma`. Use parentheses when a qualification should apply to only a subterm inside a larger term expression.

Examples:

```mizar
a + b * c       :: a + (b * c)
a * b + c       :: (a * b) + c
a ^ b ^ c       :: a ^ (b ^ c), if ^ is right-associative
a %% b %% c     :: error, if %% is a non-associative term operator
```

## B.3 Atomic Formula Boundary

Atomic formulas are the bridge between term parsing and formula parsing. The parser first consumes the term expressions required by a predicate, type assertion, attribute assertion, equality, inequality, or membership test; the resulting atomic formula then becomes one operand in the formula parser.

| Atomic Form | Example | Term Parsing Inside |
|---|---|---|
| Predicate application | `a divides b` | Predicate loci are parsed as term expressions |
| Equality / inequality | `a + b = c + d` | Both sides are parsed as term expressions |
| Membership | `x in dom f` | Both sides are parsed as term expressions |
| Type assertion | `x is Element of A` | Subject is parsed as a term expression |
| Attribute assertion | `n is positive even Integer` | Subject is parsed as a term expression |

Formula connectives do not bind inside terms. Thus `f(x & y)` is invalid unless a grammar position explicitly expects a formula. Likewise, `x + y & z` is invalid as written: `x + y` is a term, not an atomic formula, so the left operand of `&` is incomplete. The author must write a complete formula such as `x + y = 0 & z is Nat`.

Parentheses are interpreted by the grammar position that requested the parse.
When a term is expected, `( ... )` contains a `term_expression`. When a formula
is expected, `( ... )` contains a `formula`. At an atomic-formula boundary the
parser first attempts to complete a term-headed atomic formula; if the
parenthesized contents contain formula-only syntax such as `implies`, `iff`,
`&`, `or`, or a quantifier, the parenthesized group is classified as a formula
operand instead. The same reclassification is required when the contents cannot
be consumed as a complete `term_expression` because their parse reaches an
atomic formula boundary operator such as `=`, `<>`, `in`, or `is`. For example,
`(a + b) = c` starts from a parenthesized term, whereas `(P implies Q) & R` and
`(x = y) & R` start from parenthesized formulas.

Predicate-chain notation is resolved at the atomic-formula boundary, not as
term-operator associativity. For example, after predicate resolution,
`a < b < c` denotes the conjunction `a < b & b < c`; it is not rejected merely
because `<` would be non-associative as a term operator.
Built-in predicates (`=`, `<>`, and `in`) do not participate in user predicate
chains. Mixed chains such as `a < b = c` or `a in B < c` are syntax errors
unless the author inserts explicit formula connectives and complete atomic
formulas.

## B.4 Formula Operators

Formula operators use a fixed hierarchy. Lower priority numbers in the table below bind more tightly.

| Priority | Operator Form | Associativity | Example |
|---|---|---|---|
| 1 | Atomic formulas | - | `x is Nat`, `a divides b`, `x = y` |
| 2 | `not` | Prefix | `not x > 0` |
| 3 | `&` | Left | `a & b & c` parses as `(a & b) & c` |
| 4 | `or` | Left | `a or b or c` parses as `(a or b) or c` |
| 5 | `implies` | Right | `a implies b implies c` parses as `a implies (b implies c)` |
| 6 | `iff` | Non-associative | `a iff b iff c` is an error |
| 7 | `for`, `ex` | Outermost binding | `for x being Nat holds P(x) implies Q(x)` |

Examples:

```mizar
not x > 0 & y > 0
:: (not (x > 0)) & (y > 0)

a or b implies c
:: (a or b) implies c

for x being Nat holds x > 0 implies x >= 1
:: for x being Nat holds (x > 0 implies x >= 1)
```

## B.5 Recommended Parsing Algorithm

A Pratt parser, also called precedence climbing with null and left denotations, is recommended for term expressions and formula expressions. It fits the language well because the active lexicon is import-dependent and operator declarations are table-driven.

Recommended implementation outline:

1. Preprocess imports and build the active lexicon, including each visible operator's fixity, precedence, associativity, arity, and defining module.
2. Tokenize using the longest-match rule from Chapter 2. String-literal recognition remains enabled only in grammar positions that require a string argument.
3. Parse primary term forms: variables, numerals, parenthesized terms, structure constructors, set expressions, and `the` expressions.
4. Parse term-level prefix and postfix operators using their declared binding powers.
5. Parse term-level infix operators by comparing the next operator's left binding power against the current minimum binding power.
6. Parse `qua` as the lowest-precedence term-level type qualification.
7. Complete an atomic formula by parsing predicate notation, equality, membership, type assertions, or attribute assertions around the parsed term operands. If a parenthesized group at this point contains formula-only syntax, classify it as a parenthesized formula instead of a parenthesized term.
8. Parse formula-level prefix, infix, and quantifier forms using a separate fixed binding-power table.

For infix term operators, use the following binding-power convention:

| Associativity | Left Binding Power | Right Minimum Binding Power |
|---|---:|---:|
| Left | `N` | `N + 1` |
| Right | `N` | `N` |
| None | `N` | `N + 1`, plus a same-operator chaining check |

The same pattern applies to formula connectives using the fixed formula table. Non-associative operators should emit a syntax error when the same operator is chained without parentheses.

The parser should not perform overload resolution while deciding precedence. Parsing chooses the syntactic shape from declared operator metadata; overload resolution then selects the semantic definition according to Chapter 19. If two imported modules assign different precedence or associativity to the same visible user symbol, this is a link-time conflict as specified in §10.9.

## B.6 Diagnostics

Precedence-related diagnostics should prefer local, source-level explanations:

| Situation | Recommended Diagnostic |
|---|---|
| Chained non-associative operator | Report the operator and suggest adding parentheses |
| Conflicting imported precedence declarations | Report both defining modules and the conflicting declarations |
| Unexpected grouping caused by default operator precedence | Report the default precedence `64` and suggest adding an explicit declaration |
| Formula connective appears where a term is required | Report the expected term boundary and suggest parentheses or moving the connective outside the term |

The diagnostic should show the parser's actual grouping when that grouping is surprising but valid.
