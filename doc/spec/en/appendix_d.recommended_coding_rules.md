# Appendix D. Recommended Coding Rules

> Canonical language: English. Japanese companion: [../ja/appendix_d.recommended_coding_rules.md](../ja/appendix_d.recommended_coding_rules.md).

This appendix gives non-normative coding recommendations for Mizar Evolution source files. These rules are intended to improve readability, proof stability, diagnostics, documentation quality, and long-term maintainability. They do not add syntax or verification requirements beyond the main specification.

* [D. Recommended Coding Rules](#appendix-d-recommended-coding-rules)
  * [D.1 General Principles](#d1-general-principles)
  * [D.2 Names, Labels, and Modules](#d2-names-labels-and-modules)
  * [D.3 Types, Definitions, and Notation](#d3-types-definitions-and-notation)
  * [D.4 Proofs and Citations](#d4-proofs-and-citations)
  * [D.5 Algorithms and Verification](#d5-algorithms-and-verification)
  * [D.6 Annotations and Documentation](#d6-annotations-and-documentation)
  * [D.7 Refactoring Checklist](#d7-refactoring-checklist)

## D.1 General Principles

Prefer code whose mathematical intent is visible before the verifier explains it. A proof or definition should read as a stable mathematical argument, not as a sequence of accidental choices that only works under the current import set.

| Rule | Recommendation |
|---|---|
| Make intent local | Put the type, qualification, citation, or annotation close to the expression it clarifies. |
| Keep proof dependencies explicit | Cite the facts actually needed by a proof step; avoid broad citations once the proof has stabilized. |
| Prefer stable source over clever source | Avoid depending on fragile overload choices, import order, or implicit cluster chains when a short explicit form is clearer. |
| Separate specification from execution | State mathematical contracts in `requires`, `ensures`, invariants, and theorems; state reusable inference facts in registrations; keep implementation policy in algorithms or tooling configuration. |
| Use parentheses for readers | Parentheses are recommended whenever precedence or quantifier scope is not obvious, even when the parser would accept the unparenthesized form. |

## D.2 Names, Labels, and Modules

Names should make exported APIs easy to cite, document, and diagnose.

| Area | Recommended Rule |
|---|---|
| Package names | Use lowercase `snake_case`, matching the package naming rule in Chapter 12. |
| Module paths | Keep directory paths aligned with mathematical domains, for example `algebra.groups.basic` rather than implementation-oriented names. |
| Definition labels | Use stable labels for exported definitions; changing a label is a public API change for citations and documentation links. |
| Theorem and lemma labels | Use names that describe the statement, such as `Union_empty_right` or `Subgroup_card_nonzero`, rather than proof-local numbering for exported results. |
| Local proof labels | Short labels such as `A1`, `A2`, or `Step1` are acceptable inside a small proof, but avoid exporting or documenting them as public references. |
| Constructor names | Use ordinary identifiers for modes, structures, and attributes unless a readable hyphenated constructor name such as `1-sorted`, `R-module`, or `C-star-algebra` is clearer. Keep operator-like notation for functors and predicates. |
| Namespace aliases | Use aliases to shorten long imports when they improve readability, but avoid aliases that hide the mathematical origin of a symbol. |
| Qualified names | Use qualified names when two imports provide similar notation or when a proof depends on a specific library result. |

Avoid reusing names that differ only by case, punctuation, or small abbreviation. Such names are legal but hard to search and easy to confuse in diagnostics.

## D.3 Types, Definitions, and Notation

Type information is part of the proof author's communication with both the verifier and later readers.

| Rule | Recommendation |
|---|---|
| Encode preconditions in types | When possible, prefer a refined type such as `non zero Real` over an `assume y <> 0` side condition. This reduces repeated proof obligations. |
| Choose attributes for reusable unary properties | Use attributes for properties that should refine types or participate in cluster inference; use predicates for general relations. |
| Prefer `equals` for direct definitions | Use `equals` when a functor has an explicit value. Use `means` when the value is characterized by a property and existence/uniqueness proofs are appropriate. |
| Register common consequences | If a property should be inferred repeatedly, add a cluster registration rather than reproving the same attribute implication at every use site. |
| Make ambiguous views explicit | Use `qua` when multiple inheritance paths, overload candidates, or attribute-compatible views could plausibly apply. |
| Keep operator declarations unsurprising | Use precedence values consistent with nearby mathematical notation. Declare precedence explicitly for exported symbolic functors. |
| Use phrase notation sparingly | Phrase predicates can improve readability, but symbolic notation is often easier to scan in dense algebraic expressions. |

For overloaded definitions, prefer adding a definition for the most specific common subtype when many callers would otherwise need the same `qua` disambiguation. Use `redefine` only when the new declaration is a true coherent refinement of an existing root; do not rely on redefinition to reconcile distinct ordinary overloads.

## D.4 Proofs and Citations

Proofs should be robust under library growth. A proof that relies on broad search today may become slower or less predictable as imports expand.

| Rule | Recommendation |
|---|---|
| Start broad, finish narrow | During exploration, grouped or bulk citations can be useful. Before committing library code, refine successful proofs to the specific labels used. |
| Keep `by` clauses focused | Cite the definitions and lemmas that explain the step. Long citation lists make later maintenance harder. |
| Use intermediate lemmas | Split large goals when the ATP needs too many facts or when the proof argument has a natural mathematical subclaim. |
| Prefer local assumptions over hidden context | When a proof depends on a condition, make the assumption or preceding labelled step visible near the use. |
| Parenthesize complex formulas | In long formulas, add parentheses around the intended scope of `not`, `&`, `or`, `implies`, `iff`, `for`, and `ex`. |
| Treat `open` and `assumed` items visibly | Use theorem statuses deliberately. Do not let ordinary theorems depend on unsettled material. |
| Keep automation hints advisory | Use `@proof_hint` to control resources or solver choice, not to encode mathematical content. |

When a proof step succeeds only with a large axiom budget, prefer adding a named lemma or a narrower citation before increasing the budget permanently.

## D.5 Algorithms and Verification

Algorithm code should make its logical contract and execution assumptions explicit.

| Rule | Recommendation |
|---|---|
| State contracts first | Use `requires` and `ensures` to describe the externally visible behavior before relying on body details. |
| Keep invariants semantic | Write loop invariants in terms of mathematical state, not incidental iteration order, unless the algorithm is explicitly order-dependent. |
| Use `for ... in` only for order-independent loops | If the result depends on traversal order, use an explicitly ordered construct instead. |
| Introduce proof-only state for clarity | Use `ghost var` and `ghost const` for proof-only values, and use `snapshot` to name program states at important points without changing runtime behavior. |
| Give termination measures meaningful names | A `decreasing` expression should be easy to relate to the recursive call or loop progress. |
| Prefer explicit search for existential computation | For executable witnesses, implement a search algorithm with clear failure behavior rather than relying on non-executable arbitrary choice. |
| Use `by computation` for concrete facts | Reserve computation proof steps for ground or effectively computable goals, and use ordinary proof for symbolic properties. |

Non-executable mathematical definitions are acceptable, but code intended for extraction should avoid operations that the MVM cannot evaluate.

## D.6 Annotations and Documentation

Annotations should clarify source code while remaining erasable metadata.

| Rule | Recommendation |
|---|---|
| Document exported items | Add `:::` documentation comments for public definitions, theorems, registrations, and algorithms. |
| Keep documentation close | Place documentation comments immediately before the item they document, with no blank line separating them. |
| Use structured tags consistently | Use `@param`, `@returns`, `@requires`, `@ensures`, `@see`, `@since`, and `@deprecated` when they add searchable structure. |
| Prefer `@latex` for exported notation | Add `@latex` to exported symbols whose source notation is not the best rendered mathematical notation. |
| Use development annotations temporarily | `@show_type`, `@show_resolution`, and `@show_thesis` expose verifier state, while `@eval` runs verification-time expression evaluation. Keep these annotations in committed code only when their output is intentionally part of the workflow. |
| Avoid semantic annotations | Do not use annotations to change the meaning of a construct. Disambiguate source with qualified names, `qua`, explicit template arguments, or clearer definitions. |
| Suppress warnings narrowly | Use `@suppress` only at the smallest reasonable scope and prefer fixing the underlying issue when possible. |

Documentation should describe the mathematical contract and intended use, not restate the syntax that is already visible in the source.

## D.7 Refactoring Checklist

Before moving or publishing a module, check the following:

| Check | Question |
|---|---|
| Imports | Are all imports still needed, and are ambiguous symbols qualified? |
| Labels | Have public labels remained stable, or has the change been treated as an API change? |
| Overloads | Do new overloads introduce ambiguity for existing callers? |
| Redefinitions | Do new same-root redefinitions have compatible joined result facts? |
| Clusters | Are repeated attribute proofs better expressed as registrations? |
| Citations | Can broad or bulk citations be refined to the facts actually used? |
| Annotations | Are development-only display annotations removed or intentionally retained? |
| Documentation | Do exported items have documentation comments and stable cross-references? |
| Algorithms | Are contracts, invariants, and termination measures still aligned with the implementation? |
