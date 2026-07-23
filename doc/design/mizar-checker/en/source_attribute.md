# mizar-checker: Source Attribute Projection

> Canonical language: English. Japanese companion:
> [../ja/source_attribute.md](../ja/source_attribute.md).

## Purpose And Authority

`source_attribute` implements the Task 250 raw
attribute-chain/qualification/provenance producer frozen in
[`00.crate_plan.md`](./00.crate_plan.md). Its canonical authority is Chapters
03 §§3.2-3.3, 06 §§6.2/6.6/6.9, 11 §§11.2-11.3, 12
§§12.3/12.5/12.6.1/12.7, and the Chapter-17 §§17.3/17.10
restricted-adjective boundary. The bounded audit owners are MC-G014 and
MC-G020, with Tasks 248-249 as input dependencies.

## Boundary And Model

The module accepts no `SurfaceAst`, `SurfaceNodeId`, or syntax kind. One
syntax-free `SourceAttributeHandoffInput` owns dense chain, attribute,
qualifier, argument-group, and argument tables. A nonempty chain links exactly
one authenticated Task-249 `SourceTypeExpressionId`. Attribute rows preserve
source order, polarity, full occurrence and target-name sites/ranges/spellings,
recovery, and resolver-authenticated attribute symbol/contribution identity.
Negative rows preserve the written `non` occurrence independently.

An optional qualifier row preserves only a written structure disambiguator and
its authenticated structure symbol/contribution. It is not owner-compatibility
or admissibility evidence. Prefix and parenthesized-argument-list groups remain
independent written forms. Groups preserve exact delimiter, comma, and hyphen
provenance; actual rows preserve `PrefixIdentifier`, `PrefixNumeral`, or
`TermSite` typed sites and `SemanticOrigin` without a selected `BindingId`,
type result, or normalized term.

## Validation And Atomicity

`SourceAttributeProducer` authenticates its parent
`SourceTypeApplicationHandoff`, `BindingEnv`, `SymbolEnv`, and `TypedArena`.
It rejects cross-source/module input, missing or duplicate parents, empty or
reordered chains, dangling or multiply owned rows, stale sites/ranges/
spellings/recovery, overlapping source elements, polarity/`non` mismatch,
wrong symbol roles or contributions, local declarations not active before
use, invisible or out-of-closure imported symbols, unauthenticated qualifiers,
invalid group form/punctuation/cardinality, and invalid actual order/kind/
origin. Spelling authentication is compositional from actuals through groups
and attributes, and the complete source-ordered attribute spelling must be a
prefix of the already authenticated Task-249 expression spelling.

Dense public vectors are validated in their supplied order. Validation never
sorts, repairs, normalizes, or publishes a partial handoff. Every typed site is
revalidated during `TypedAst` installation. Failure leaves the typed and
resolved ASTs without a Task-250 payload.

## Ownership, Consumers, And Exclusions

`TypedAst` owns the optional immutable handoff. `ResolvedTypedAst` can only
clone it from that typed AST; no separately replaceable resolved input exists.
Conditional debug rendering keeps legacy bytes unchanged when the handoff is
absent.

The real runner consumers are exactly Tasks 81, 67, 84, and 85. Together they
publish Task-249 `4/4/0` and Task-250
`4 chains / 4 attributes / 1 qualifier / 1 group / 1 actual`, with three
positive and one negative attribute, two local and two imported attributes,
and three builtin-`set` heads plus one local structure head. The synthetic
private extractor probe `p-ranked (q,2)-graded set` covers two ordered
attributes, single and parenthesized prefixes, exact punctuation, and three
prefix actuals through the public producer.

Task 250 preserves prefix and argument-list spellings for a later semantic
owner. It performs no arity or term typing, normalized-instance or prefix/list
equivalence decision, admissibility, structure-owner compatibility, evidence
request/result, fact/truth/closure, accepted declaration/proof production, or
Core/CFG/VC construction. The legacy normalized `AttributeInput` remains a
separate bridge.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceAttributePolarityInput` | `#[non_exhaustive]`; callers must tolerate later polarity source forms. |
| `SourceAttributeArgumentGroupKind` | `#[non_exhaustive]`; callers must tolerate later syntax-free group forms. |
| `SourceAttributePrefixForm` | `#[non_exhaustive]`; callers must tolerate later prefix spellings. |
| `SourceAttributeActualKind` | `#[non_exhaustive]`; callers must tolerate later syntax-free actual kinds. |
| `SourceAttributeError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Task 250 Classification

| Class | Result |
|---|---|
| `test_gap` | Closed only for the exact four real handoffs and synthetic extractor/corruption matrices. |
| `source_drift` | Repaired for complete raw chain, polarity, qualifier, argument, provenance, and final-handoff transport. |
| `design_drift` | Repaired by the frozen contract and paired component/plan/todo/audit/runner documents. |
| `boundary_violation` | None; raw syntax remains runner-owned and later semantic decisions remain deferred. |
| `spec_gap` | None for this bounded raw input-handoff slice. |
| `test_expectation_drift` | None; only the authorized Task-67/81 pending progression changes, while Task-84/85 preserve their outcomes. |
| `repo_metadata_conflict` | None observed. |
