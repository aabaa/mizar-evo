# mizar-parser: Pratt Parsing

Status: task-12 term Pratt parsing is implemented for active-lexicon prefix,
postfix, and infix operators; task-13 atomic formulas use the term Pratt
boundary; task-14 fixed formula connective precedence is implemented; task-15
set-comprehension primaries are implemented.

## Purpose

This module defines the precedence parser for term and formula expressions.

## Responsibilities

- use active-lexicon operator metadata for term-level prefix, postfix, and infix forms;
- use the fixed formula connective table for formula-level precedence;
- parse syntactic shape without performing overload resolution;
- report non-associative chaining and precedence surprises with source-local diagnostics.

## Term Pratt Contract

`ParseRequest::operator_fixity` carries parser-facing operator metadata derived
from `ParserInputs`: spelling, fixity kind, precedence, source-coordinate
`active_from`, and, for infix operators, associativity. The parser consumes
this metadata only as grammar configuration. It does not choose overload roots,
infer result types, evaluate cluster facts, or resolve selector-versus-namespace
roles.

`infix_operator`, `prefix_operator`, and `postfix_operator` declarations affect
only later tokens, so Pratt lookup filters operator metadata by the operator
token's source span. Entries with `active_from > token.span.start` are ignored.
Operator metadata is spelling-level. For a token spelling, Pratt lookup first
selects the newest active metadata entry, regardless of fixity, and then checks
whether that selected entry is valid for the current prefix/postfix/infix
context. A newer prefix declaration for a spelling therefore shadows an older
infix declaration for later tokens instead of letting the older infix remain
available. Same-activation ties are kept deterministic and are assumed to have
been checked for link-time conflicts before this parser stage.

Term parsing uses the following order:

1. Prefix operators are null denotations. Their operand is parsed at the
   operator precedence.
2. Primary terms and the fixed selector/update postfix chain are parsed next.
   Selector/update/application syntax therefore binds tighter than user
   operators. Set comprehensions added by task 15 are primary terms; their
   mapper child is parsed through the same term Pratt boundary, and their
   optional condition child is parsed through the fixed formula Pratt boundary.
3. User postfix and infix operators are folded by Pratt binding power.
4. `qua` is parsed by the module grammar after Pratt as the fixed lowest
   term-level operator and remains left-associative.

Atomic-formula parsing starts after the term Pratt boundary. Task 13 consumes
built-in predicates, `is` assertions, inline predicate calls, and syntax-only
user predicate segments around already parsed term operands.

For infix term operators, the binding powers match Appendix B:

| Associativity | Left binding power | Right minimum binding power |
|---|---:|---:|
| Left | `N` | `N + 1` |
| Right | `N` | `N` |
| None | `N` | `N + 1`, plus same-operator chain diagnostics |

Prefix and postfix operators use the supplied precedence as binding power; the
supplied metadata may come from an explicit declaration or from the
spec-defaulted summary-side metadata. After the newest active spelling entry is
selected, the parser uses it only in contexts where its fixity is syntactically
eligible: prefix entries where a term operand may start, and postfix / infix
entries after a left operand has been parsed. A same-spelling operator conflict
with incompatible metadata is a lexical-environment or link stage error; this
parser stage assumes `ParserInputs` has already selected a deterministic visible
entry order for the token position being parsed.

## Formula Pratt Contract

Task 14 uses a fixed formula parser, not import-dependent operator metadata.
Atomic formulas, parenthesized formulas, `thesis`, and `contradiction` are
primary formula operands. Prefix `not` parses one formula operand at the
prefix binding level. Binary connectives use the fixed hierarchy from Appendix
B:

| Operator | Associativity | Relative binding |
|---|---|---:|
| `&` | left | 50 |
| `or` | left | 40 |
| `implies` | right | 30 |
| `iff` | none | 20 |

The numeric binding powers are local parser constants only; the relative order
is the contract. `iff` emits `NonAssociativeOperatorChain` if another top-level
`iff` continues the same unparenthesized chain. Repetition forms `& ... &` and
`or ... or` keep the same precedence and associativity as their
non-repetition connective and are represented by the same binary formula node
with the additional `...` token preserved.

Quantifiers are not user operators. `for` and `ex` parse as outermost formula
forms before the Pratt binary table. A quantified formula may appear as the
right operand where Chapter 14 permits `( ... | quantified_formula )`, so
`P implies for x being T holds Q` groups as `P implies (for x being T holds
Q)`. Quantifier variable typing is syntactic only; implicit variable typing
from `reserve` belongs to later resolution.

## Public Enum Compatibility

`OperatorAssociativity` remains deliberately exhaustive. It is a closed
parser-facing fixity property with the same three meanings as
`mizar-syntax::SurfaceOperatorAssociativity`: left-associative,
right-associative, and non-associative. A future operator-model change that
needs another associativity category must update this design note, parser
matches, syntax payload mapping, and lint-policy expectations in the same
change.

`OperatorFixity` is also deliberately exhaustive for the current term operator
model: prefix, infix, and postfix. Bracket delimiter-pair functors are not
represented by this enum until parser inputs grow bracket-pair metadata.
