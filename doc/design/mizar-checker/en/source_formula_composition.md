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

The Task-257A `5/0/1/1/1/4/6` input, validation, debug output, installation,
and existing consumer remain byte-identical. No existing public input field or
row meaning is repurposed. A profile discriminator is derived from the
validated table shape; the source does not provide a caller-selected mode.
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
