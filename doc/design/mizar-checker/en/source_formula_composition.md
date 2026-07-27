# Source Formula Composition Transport

> Canonical language: English. Japanese companion:
> [../ja/source_formula_composition.md](../ja/source_formula_composition.md).

## Responsibility And Authority

Checker Task 257B1 owns the first cross-family composition step for quantified
formulas: one explicit universal binder, one atomic equality body, and the two
primary-term references captured by that binder's lexical scope. Canonical
authority is Chapter 4 §§4.1, 4.5, and 4.6 and Chapter 14 §§14.4.1, 14.4.4,
14.5.2, and 14.7.5. Task 252 owns the two variable-reference occurrences,
Task 256 owns the equality occurrence and its operand edges, and Task 257A
continues to own the universal occurrence, binder, body context, and written
binder type.

This transport does not evaluate equality or quantification, relativize the
binder type, publish a formula fact, accept a theorem, or create proof,
CoreIr, ControlFlowIr, or VC output. The `BindingEntry::captured` field remains
empty: it records free-variable capture by closure-like binders, whereas this
task records ordinary bound occurrences selected by lexical lookup.

## Exact Real Consumer

Implementation adds one specification-derived type-elaboration fixture:

```mizar
theorem FormulaQuantifierBoundUsePayloadBoundary: for x being set holds x = x;
```

The source is 79 bytes including its trailing newline and has SHA-256
`757872ac21c2a924c7c47f23328f5d76a8504255c195c17f113041c81bae5f3c`.
Its frozen half-open source ranges are universal `50..77`, binder segment
`54..65`, binder identifier `54..55`, binder type/head `62..65`, equality
`72..77`, left occurrence `72..73`, and right occurrence `76..77`.
Implementation preflight must remeasure these ranges through the real parser
before creating the fixture and treat any mismatch as documentation
`design_drift`, not as authority to change parser behavior.

The sidecar is a type-elaboration pass whose only positive claim is successful
source-to-checker transport. It grants no equality truth, quantified formula
truth, theorem status, accepted fact, or proof credit.

## Task-257A Profile Extension

`SourceCompositeFormulaProducer` gains a second exact profile without changing
the Task-257A profile or its debug bytes. The new profile has
formula/wrapper/root/binder/type-site/composite-edge/request counts
`1/0/1/1/1/0/2`: one universal formula in context 0, one unassigned root, one
explicit `x` binder and bare-`set` type site, no same-family child edge, and
quantifier-semantics plus binder-type requests. Its extended binding
environment remains `2/1/4`.

The exact real Task-257A `5/0/1/1/1/4/6` transaction, its validation/debug
output, installation, and existing consumer remain byte-identical. This
preservation excludes the former synthetic nonempty-wrapper admission, which
the exact profile partition retires and defers to Task 257B2. No existing
public input field or row meaning is repurposed. A profile discriminator is
derived from the validated table shape; the source does not provide a
caller-selected mode.
Validation accepts only the two exact profiles. Task-257A cardinalities with
Task-257B1 formulas, requests, binders, or edges, the inverse mixtures, and
any otherwise well-formed third shape fail atomically.

## Cross-Family Transaction

Task 257B1 adds a public syntax-free `source_formula_composition` module with
two dense tables:

```rust
pub struct SourceFormulaCompositionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub atomic_edges: Vec<SourceFormulaAtomicEdgeInput>,
    pub bound_uses: Vec<SourceQuantifierBoundUseInput>,
}

pub struct SourceFormulaAtomicEdgeInput {
    pub formula: SourceCompositeFormulaId,
    pub ordinal: usize,
    pub role: SourceFormulaAtomicEdgeRole,
    pub child: SourceAtomicFormulaId,
}

pub struct SourceQuantifierBoundUseInput {
    pub binder: SourceQuantifierBinderId,
    pub ordinal: usize,
    pub body_edge: SourceFormulaAtomicEdgeId,
    pub term: SourcePrimaryTermId,
    pub reference: SourcePrimaryTermReferenceId,
}
```

The dense ids are `SourceFormulaAtomicEdgeId` and
`SourceQuantifierBoundUseId`. Each exposes private storage, `new`, and
`index`. Immutable rows expose read-only accessors and tables expose only
`get`, source-ordered `iter`, `len`, and `is_empty`.
`SourceFormulaAtomicEdgeRole` has only `UniversalBody` in this slice.
`SourceFormulaCompositionError` and every public enum are
`#[non_exhaustive]`.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceFormulaAtomicEdgeRole` | `#[non_exhaustive]`; callers must tolerate later frozen cross-family body roles. |
| `SourceFormulaCompositionError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

The exact producer and output surface is:

```rust
impl SourceFormulaCompositionProducer {
    pub fn build(
        input: SourceFormulaCompositionHandoffInput,
        primary_terms: &SourcePrimaryTermHandoff,
        atomic_formulas: &SourceAtomicFormulaHandoff,
        composite_formulas: &SourceCompositeFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<SourceFormulaCompositionHandoff, SourceFormulaCompositionError>;
}

pub struct SourceFormulaCompositionHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    primary_term_fingerprint: String,
    atomic_formula_fingerprint: String,
    composite_formula_fingerprint: String,
    atomic_edges: SourceFormulaAtomicEdgeTable,
    bound_uses: SourceQuantifierBoundUseTable,
}
```

The three fingerprints are exact owned copies of the corresponding dependency
`debug_text()` strings, in Task-252/256/257 dependency order. They are
nonempty and are exposed read-only as `primary_term_fingerprint()`,
`atomic_formula_fingerprint()`, and `composite_formula_fingerprint()`.
The handoff also exposes `source_id()`, `module_id()`, `atomic_edges()`, and
`bound_uses()`. No mutable or unchecked publication surface exists.

`debug_text()` is deterministic and starts
`source-formula-composition-debug-v1\n`. It renders module identity, then the
three fingerprints with Rust debug-string escaping in primary/atomic/
composite order, then `atomic-edges: N` and every edge row, then
`bound-uses: N` and every use row. Edge fields are
id/formula/ordinal/role/child; use fields are
id/binder/ordinal/body-edge/term/reference. Role spelling is
`universal-body`. The positive test freezes one full literal rendering rather
than substring checks or comparison of two equally incomplete outputs.

The producer publishes one immutable handoff only after the complete
transaction validates.

The exact real aggregate is atomic-edge/bound-use `1/2`:

| Row | Association |
|---:|---|
| atomic edge 0 | composite universal 0, ordinal 0, universal-body, atomic equality 0 |
| bound use 0 | binder 0, ordinal 0, edge 0, primary term 0, reference 0 |
| bound use 1 | binder 0, ordinal 1, edge 0, primary term 1, reference 1 |

The dependent Task-252 transaction is terms/references/numeric requests
`2/2/0`. Both terms are normal `VariableReference`/`Value` rows in body
context 1, spell `x`, and resolve through `BindingEnv::lookup` to binding 0
with lexical scope `[0]` and use ordinal 1. The dependent Task-256 transaction
is formula/wrapper/head/candidate/type/attribute/edge/request
`1/0/0/0/0/0/2/2`: one equality in context 1, two primary-term operand edges,
and two unresolved operand-expected-type requests.

## Validation And Final Ownership

Validation authenticates source/module identity, all dependency fingerprints,
dense row order, the exact universal/binder/body-context relation, the atomic
equality context and range, both equality operand edges, both primary-term
references, binding lookup winner, source order, and containment. Every
bound-use term must be a direct operand of the edge's atomic child, every
reference must select the composition binder's binding, and no term or
reference may be omitted, duplicated, reordered, cross-context, out of range,
or associated with another binder or formula.

The existing `TypedAst::with_source_composite_formula` remains restricted to
the complete Task-257A profile. The new
`TypedAst::with_source_formula_composition(self, composite, composition)` is
a combined one-shot installer: Task-252 and Task-256 must already be
installed, but the second composite profile and its composition handoff are
validated and published together. The second profile can never appear in a
public `TypedAst` without its atomic body edge and bound-use rows.
`source_context()` must remain absent: a preinstalled Task-248 source-context
handoff is rejected atomically, preserving Task 257A's sole ownership of the
embedded source-derived `2/1/4` binding environment.
The legacy installer must reject a valid uninstalled Task-257B1 composite
profile. The combined installer must reject an AST that already owns a
Task-257A composite handoff. Both failures preserve every pre-existing
handoff and debug byte and publish neither the second profile nor the
composition handoff.
`TypedAst::source_formula_composition()` and
`ResolvedTypedAst::source_formula_composition()` expose the optional immutable
handoff. Typed and resolved debug output render the existing Task-252 term,
Task-256 atomic formula, Task-257 composite formula, then Task-257B1
composition in that order. Absence preserves legacy bytes exactly.
`ResolvedTypedAst::assemble` revalidates all fingerprints and clone-preserves
the same composition handoff without rebuilding rows. Installation or final
assembly fails atomically on missing, stale, substituted, or reordered
dependencies. The Task-257A-only AST remains valid with no composition
handoff.

## Tests And Exit Boundary

Checker tests cover the second composite profile, exact `1/2` composition,
dependency fingerprints, bound-use lookup and order, corruption of every
field and association, deterministic replay, full literal debug output,
combined one-shot installation, missing/wrong dependency rejection,
preinstalled Task-248 source-context rejection, Task-257A debug byte
preservation, legacy-installer B1 rejection, combined-installer Task-257A
rejection, cross-profile/hybrid/third-shape rejection, rollback byte
preservation, and final clone ownership. Runner tests cover exact parser
ranges, the `2/2/0`, `1/0/0/0/0/0/2/2`, `1/0/1/1/1/0/2`, and `1/2`
aggregates, same-arena composition, selector isolation, and no semantic table
or accepted-fact output.

Implementation may add one covered trace requirement,
`spec.en.checker.type_elaboration.source_quantifier_bound_use_payload`, mapped
only to the new pass sidecar. It may add reciprocal transport notes to the
existing Chapter-4, Chapter-14, Task-252, Task-256, and Task-257A rows without
changing their status. The projected counts are plan `415/381`,
type-elaboration `247/235`, pass/fail `225/190`, and active
parse/declaration/type/proof `101/5/194/1`.

Task 257B2 retains conjunction, disjunction, `iff`, repetition, and executable
formula grouping. Task 257B3 retains existential, restricted and nested
quantification, implicit reserved binders, and their additional scoped uses.
Task 257C retains predicate-chain and conditioned-comprehension composition.

## Implementation Result

Task 257B1 now implements this frozen boundary. The exact 79-byte pass
consumer builds the Task-252 `2/2/0`, Task-256 `1/0/0/0/0/0/2/2`,
second Task-257 `1/0/1/1/1/0/2`, and formula-composition `1/2`
transactions in one arena. Both direct `x` references resolve to binding 0 in
body context 1, while Task 252 remains their occurrence owner and
`BindingEntry::captured` remains empty.

The combined installer, legacy-profile partition, dependency fingerprints,
full literal debug rendering, corruption matrix, Task-248/Task-257A
exclusion, and resolved clone ownership are executable. The covered trace
requirement is
`spec.en.checker.type_elaboration.source_quantifier_bound_use_payload`, mapped
only to the new pass sidecar. Counts are plan `415/381`,
type-elaboration `247/235`, pass/fail `225/190`, active
parse/declaration/type/proof `101/5/194/1`, and warnings/errors `23/0`.
Task 257B2 is the next dependency-ready formula slice.

## Task 257B2 Frozen Connective/Grouping Addendum

Task 257B2 extends this transport only for the exact 166-byte source frozen in
the crate plan. The source retains Task-257B1's explicit `x being set` binder
and body context but deliberately contains no `x` occurrence. Its body is one
`iff` whose grouped left side contains repeated conjunction/disjunction and
whose grouped right side contains fixed conjunction/disjunction. This is a
Chapter-14 source-transport slice, not connective evaluation.

The third exact composite profile is
formula/wrapper/root/binder/type-site/same-family-edge/request
`8/6/1/1/1/7/9`, with binding environment `2/1/4`. It adds
`Conjunction`, `RepeatedConjunction`, `Disjunction`,
`RepeatedDisjunction`, and `Biconditional` formula kinds. The exact
same-family tree adds only disjunction-left/right and
biconditional-left/right roles; conjunction and repeated nodes reach atomic
children through the composition table. Repeated kinds remain distinguishable
by their kind and canonical spelling.

The six wrapper rows are real `ParenthesizedFormula` occurrences with ranges
`72..122`, `73..94`, `98..121`, `127..164`, `128..143`, and `147..163`.
They are grouped by formula/ordinal, preserve independent typed sites, use
context 1 and normal recovery, are associated with exactly one owner and
strictly contain that owner's range, and never become formulas, child edges,
requests, or semantic results. Descendant ranges may be nested inside an outer
wrapper, but an unrelated sibling may not overlap or be contained by it.

The lower profiles are Task 252 `16/0/16` and Task 256
`8/0/0/0/0/0/16/16`. Every primary term is a numeral, so the binder has no
reference and `BindingEntry::captured` remains empty. The composition profile
is atomic-edge/bound-use `8/0`; new `ConjunctionLeft`,
`ConjunctionRight`, `DisjunctionLeft`, and `DisjunctionRight` atomic-edge
roles associate the eight Task-256 equalities with their nearest composite
parent. No row copies a Task-252 or Task-256 occurrence.

Validation authenticates the exact dependency profiles and fingerprints,
formula tree, contexts, fixed/repeated kinds, direct repetition tokens,
wrappers, ranges, source order, parent containment, atomic associations, and
the absence of references/bound uses. It rejects A/B1/B2 hybrids, a fourth
otherwise valid profile, wrapper crossing or substitution, fixed/repeated
substitution, dependency replacement, and any omitted, duplicated, reordered,
or cross-source association.

The existing combined installer publishes the B2 composite and composition
atomically after exact Task-252/256 installation. The legacy composite
installer remains A-only; preinstalled Task-248 source context or an existing
A/B1 composite/composition prevents B2 publication without changing any
prior byte. Final assembly revalidates and clone-preserves the exact handoff.
The `source-formula-composition-debug-v1` header and every old A/B1 rendering
remain byte-identical.

Implementation may add only
`pass_type_elaboration_formula_connective_grouping_payload_001` and the
covered row
`spec.en.checker.type_elaboration.source_connective_grouping_payload`, plus
reciprocal unchanged-status transport notes. Projected counts are plan
`416/382`, type `248/236`, pass/fail `226/190`, and active
`101/5/195/1`. Connective truth, general repetition validation or expansion,
theorem acceptance, facts, proof/IR/VC, Task 257B3, Task 257C, and Steps 6/7
remain deferred.

This addendum is a documentation prerequisite only. It leaves production,
fixtures, sidecars, trace metadata/counts, and executable coverage unchanged
at plan `415/381`, type `247/235`, pass/fail `225/190`, active
`101/5/194/1`, and warnings/errors `23/0`.

## Task 257B2 Implemented Connective/Grouping Composition

The frozen third profile is now implemented. Eight ordered atomic edges map
the repeated/fixed conjunction and disjunction rows to their eight Task-256
equalities, while bound-use remains empty because the explicit binder is
unused. Exact dependency spellings, contexts, numeric requests, fingerprints,
wrapper/tree ownership, and empty capture are revalidated fail-closed.
Combined `TypedAst` publication and `ResolvedTypedAst` cloning are atomic;
connective truth, repetition expansion, theorem status, facts, proof, and IR
remain deferred.

## Task 257B3 Frozen Nested-Quantifier Composition

The fourth composition profile is exactly `3/6`. Its atomic rows associate
the outer restriction and inner restriction through new
`UniversalRestriction` roles and the innermost equality through
`UniversalBody`. The six source-ordered bound-use rows point to Task-252
references: three `x` uses select outer binder 1, one `y` use selects binder
2, and two `r` uses select inner binder 3 rather than reserved binding 0.
Every association names the atomic edge that encloses its term.
For source compatibility, public
`SourceQuantifierBoundUseInput::body_edge`, immutable
`SourceQuantifierBoundUse::body_edge()`, and the `body-edge` debug key remain
named as in B1 but are generalized to the owning atomic edge for B3
restriction uses. Exact owning-edge ids are `0,0,1,1,2,2`;
binder-row ids are `0,0,2,1,0,2`; per-binder ordinals are
`0,1,0,0,2,1`.

Validation authenticates Task-48 reserve-default provenance, Task-252
`6/6/0`, Task-256 `3/0/0/0/0/0/6/6`, Task-257B3
`3/0/1/3/3/2/6`, context ancestry, lexical lookup replay, shadowing,
nearest-parent roles, fingerprints, and final ownership. Direct nested
quantified uses do not become `CapturedFreeVariables`. No quantified truth,
restriction discharge, witness, theorem closure, fact, acceptance, proof, or
IR is produced.
