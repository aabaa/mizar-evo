# Appendix E. Annotation Quick Reference

> Canonical language: English. Japanese companion: [../ja/appendix_e.annotation_quick_reference.md](../ja/appendix_e.annotation_quick_reference.md).

This appendix is a non-normative quick reference for annotation forms used in Mizar Evolution source files. The normative syntax and semantics are defined in [Chapter 2, §2.9](./02.lexical_structure.md#29-comments-and-annotations), [Chapter 21](./21.source_code_annotation_and_atp.md), [Chapter 22](./22.error_handling_and_diagnostics.md), and [Chapter 24, §24.1](./24.documentation_generation.md#241-documentation-comments-syntax).

* [E. Annotation Quick Reference](#appendix-e-annotation-quick-reference)
  * [E.1 Annotation Contexts](#e1-annotation-contexts)
  * [E.2 Statement and Item Annotations](#e2-statement-and-item-annotations)
  * [E.3 Library Annotations](#e3-library-annotations)
  * [E.4 Documentation Tags](#e4-documentation-tags)
  * [E.5 Development Guidance](#e5-development-guidance)

## E.1 Annotation Contexts

| Context | Surface Form | Typical Placement | Primary Use | See |
|---|---|---|---|---|
| Statement / item annotation | `@name` or `@name(...)` | Immediately before the item or source location it annotates | Proof hints, rendering hints, diagnostics, evaluation | Ch.2, Ch.21, Ch.22 |
| Library annotation | `@[label, ...]` | Immediately before a definition, theorem, or registration | Stable metadata labels for proof search | Ch.21 |
| Documentation tag | `@name ...` inside `:::` comments | First token of a documentation-comment paragraph | Structured generated documentation | Ch.24 |

Annotation names are fixed by the language registry for statement and item annotations. Documentation tags inside `:::` comments are handled by the documentation generator; unrecognized documentation tags are passed through rather than rejected by the verifier.

## E.2 Statement and Item Annotations

| Annotation | Form | Applies To | Effect |
|---|---|---|---|
| LaTeX rendering | `@latex("...")` | Immediately before a `func`, `pred`, `mode`, or `attr` definition | Supplies preferred mathematical rendering for documentation and IDE display. |
| Proof hint | `@proof_hint(...)` | Immediately before a `by` step, `thus` statement, or `proof ... end` block | Restricts or configures ATP proof search for the following proof obligation. |
| Show thesis | `@show_thesis` | Proof position | Emits the current thesis as an informational diagnostic. |
| Show resolution | `@show_resolution` | Immediately before an expression | Emits overload-resolution details for the following expression. |
| Show type | `@show_type(expr)` | Development position | Emits the inferred type of `expr` without binding it. |
| Evaluate expression | `@eval(expr)` | Top level, proof block, or algorithm body | Runs verification-time evaluation and emits the computed result when possible. |
| Suppress warning | `@suppress(Wnnnn)` | Item or smallest supported warning scope | Suppresses a warning such as `W0102` for the annotated scope. |

`@proof_hint` accepts options separated by commas:

| Option | Example | Meaning |
|---|---|---|
| `max_axioms` | `@proof_hint(max_axioms: 32)` | Caps the number of axioms sent to the ATP for the annotated step. |
| `timeout` | `@proof_hint(timeout: 60)` | Sets the per-step ATP time limit in seconds. |
| `solver` | `@proof_hint(solver: vampire)` | Selects a backend solver for the step. `auto` uses the default portfolio policy. |

Examples:

```mizar
@latex("\\gcd(a,b)")
func GcdDef: gcd(a, b) -> Nat means ...;

@proof_hint(max_axioms: 32, solver: vampire)
thus thesis by GroupAssoc, GroupIdentity;

@show_type(total + x)
@eval(factorial(10))
```

## E.3 Library Annotations

Library annotations attach stable labels to definitions, theorems, or registrations for use by proof search.

```ebnf
library_annotation ::= "@[" label_list "]" ;
label_list         ::= label_name { "," label_name } ;
label_name         ::= label_identifier [ "(" annotation_args ")" ] ;
annotation_args    ::= annotation_arg { "," annotation_arg } ;
annotation_arg     ::= identifier | nat_literal | string_literal ;
```

Examples:

```mizar
@[label, category("algebra")]
theorem Union_empty_right:
  X \/ {} = X by ...;

@[category("set")]
registration
  let X be set;
  reduce UnionEmpty: X \/ {} to X;
  reducibility proof ... end;
end;
```

The annotations above are illustrative. Built-in verifier behavior is defined only for annotations recognized by the language or implementation. Additional annotations may be registered by libraries and ignored by the built-in verifier. Reduction priority is not controlled by annotations; automatic rule selection is defined by §17.6.4.

## E.4 Documentation Tags

Documentation tags appear inside `:::` documentation comments. They are interpreted by `mizar doc`, not by the proof checker.

| Tag | Form | Meaning |
|---|---|---|
| Parameter | `@param name` | Documents one parameter named `name`. |
| Return value | `@returns` | Documents the return value; `result` refers to the return value. |
| Precondition prose | `@requires` | Describes a precondition in prose, supplementing formal `requires`. |
| Postcondition prose | `@ensures` | Describes a postcondition in prose, supplementing formal `ensures`. |
| See also | `@see ref` | Adds a cross-reference to another item, section, or URL. |
| Introduced version | `@since version` | Records the version in which the item was introduced. |
| Deprecation | `@deprecated version` | Marks the item as deprecated since `version`; the following text gives the message. |

Example:

```mizar
::: Divides `a` by `b` and returns the quotient.
:::
::: @param a  the dividend
::: @param b  the divisor; must be non-zero
::: @returns  the real number `a / b`
::: @requires `b <> 0`
::: @see mml.real.Real_div_mul
algorithm divide(a, b) -> Real
  requires b <> 0
  ensures result * b = a
do
  return a / b;
end;
```

## E.5 Development Guidance

| Goal | Prefer |
|---|---|
| Disambiguate overloads | Qualified names, explicit template arguments, or `qua`; do not rely on annotations to change semantics. |
| Explore a proof state | `@show_thesis`, `@show_type`, and `@show_resolution`. |
| Inspect computable values | `@eval(expr)`, only where the expression is expected to be computable. |
| Tune proof search | `@proof_hint`, with the smallest practical scope. |
| Improve rendered notation | `@latex` on exported symbols. |
| Document public APIs | `:::` comments with structured documentation tags. |
| Silence noisy diagnostics | `@suppress`, narrowly scoped and preferably temporary. |

Annotations are semantically neutral: removing them may affect diagnostics, rendering, proof-search performance, or documentation, but it must not change the logical meaning of the source.
