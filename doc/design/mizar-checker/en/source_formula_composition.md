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
| `SourceConditionFormulaCompositionError` | `#[non_exhaustive]`; callers must not exhaustively match condition/formula validation failures. |
| `SourceFormulaAtomicEdgeRole` | `#[non_exhaustive]`; callers must tolerate later frozen cross-family body roles. |
| `SourceFormulaCompositionError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |
| `SourceFraenkelGeneratorBindingContextError` | `#[non_exhaustive]`; callers must not exhaustively match Fraenkel generator binding-context validation failures. |
| `SourceFraenkelGeneratorBoundUseError` | `#[non_exhaustive]`; callers must not exhaustively match Fraenkel generator bound-use validation failures. |
| `SourceNestedFraenkelBinderUseError` | `#[non_exhaustive]`; callers must not exhaustively match nested Fraenkel binder/use validation failures. |
| `SourceNestedFraenkelCaptureIdentityError` | `#[non_exhaustive]`; callers must not exhaustively match nested Fraenkel capture-identity validation failures. |
| `SourcePredicateChainCompositionError` | `#[non_exhaustive]`; callers must not exhaustively match predicate-chain composition validation failures. |

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

## Task 257B3 Implementation Status

The fourth profile and `3/6` association transaction are now executable from
the exact source consumer. Checker and real-runner tests authenticate all
three parent roles, six lookup-selected uses, dependency fingerprints,
deterministic rendering, atomic installation, rollback, and resolved cloning.
Atomic-edge validation rejects an outer assignment whenever a deeper
descendant composite formula also contains the atom, preserving the frozen
nearest-parent/subtree exclusion. The frozen semantic deferrals remain
unchanged.

## Task 257C1 Prerequisite Boundary

Task 257C1 supplies only the lower Task-256 predicate-segment graph and shared
term boundary. This module receives no new row in that slice. Predicate-chain
implicit conjunction and segment-local semantic negation require a later,
separately frozen Task-257C composition contract after the C1 implementation.
Conditioned-comprehension composition also waits for its separate Task-255
condition-bearing prerequisite.

The implementation must nevertheless add an empty `predicate_segments`
vector to all three existing `SourceAtomicFormulaHandoffInput` literals in
this production file. The three matching mizar-test composition literals also
remain empty. These are mandatory compatibility edits for the extended input
shape; they add no composition row, selector admission, debug output, or
semantic behavior.

The compatibility edits are now installed and verified. Task 257C1 publishes
only the lower Task-256 handoff; this module still owns no predicate-chain
composition row or semantic conjunction/negation.

## Task 257C2 Frozen Condition-Formula Composition

Task 257C2 adds a dedicated second transaction in this module. It associates
the one Task-255C1 `SourceSetConditionId` with the one direct Task-256
`SourceAtomicFormulaId`; it does not create or require a synthetic
`SourceCompositeFormulaHandoff`. Canonical authority is Chapters 10 §10.1,
13 §§13.4/13.4.2, and 14 §§14.2/14.5.2/14.8. The exact source remains the
committed 191-byte conditioned-comprehension fixture with final-LF SHA-256
`8d9c3208d0e5a099e54c58f57642642046f0669c9b49e30d115549ba15a6eb3f`.

The lower graph is Task-252 `4/0/4`, Task-253 `1/0/1/2/2`, Task-255
`1/0/1/1/1/1/2`, and Task-256
formula/wrapper/segment/head/candidate/type/attribute/edge/request
`1/0/0/0/0/0/0/2/2`. Task 255 owns the `177..182`
`FormulaExpression` wrapper; Task 256 owns the distinct direct
`BuiltinPredicateApplication` equality site with the same range, spelling
`3 = 4`, context 0, and normal recovery. Its two operand edges point to
Task-252 primaries 2 and 3. The new association owns no site and copies no
lower row.

The new public surface is
`SourceConditionFormulaCompositionHandoffInput`,
`SourceConditionFormulaEdgeInput`, immutable
`SourceConditionFormulaCompositionHandoff`/`SourceConditionFormulaEdge`,
`SourceConditionFormulaEdgeTable`, dense `SourceConditionFormulaEdgeId`,
`SourceConditionFormulaCompositionProducer`, and non-exhaustive
`SourceConditionFormulaCompositionError`. The input has only source/module
identity and an `edges` vector. Each edge stores condition, dense ordinal, and
atomic formula. The sole exact row is `0/0/0`.

The producer consumes Task-252 primary, Task-253 application, Task-255 set
term, Task-256 atomic formula, and arena dependencies. The handoff retains
their four exact nonempty debug fingerprints in that order. IDs expose
`new`/`index`; tables expose `get`/`iter`/`len`/`is_empty`; rows and the
handoff expose only read-only accessors. The public producer and handoff
signatures are frozen as:

```rust
impl SourceConditionFormulaCompositionProducer {
    pub fn build(
        input: SourceConditionFormulaCompositionHandoffInput,
        primary_terms: &SourcePrimaryTermHandoff,
        applications: &SourceFunctorApplicationHandoff,
        set_terms: &SourceSetTermHandoff,
        atomic_formulas: &SourceAtomicFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<
        SourceConditionFormulaCompositionHandoff,
        SourceConditionFormulaCompositionError,
    >;
}

impl SourceConditionFormulaCompositionHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn primary_term_fingerprint(&self) -> &str;
    pub fn application_fingerprint(&self) -> &str;
    pub fn set_term_fingerprint(&self) -> &str;
    pub fn atomic_formula_fingerprint(&self) -> &str;
    pub const fn edges(&self) -> &SourceConditionFormulaEdgeTable;
    pub fn debug_text(&self) -> String;
}
```

The exact ID/table/row signatures are frozen in the canonical crate plan.
Errors are `DependencyMismatch`, `InvalidEdge { edge }`, and
`InvalidAggregate`.

Debug output has a separate
`source-condition-formula-composition-debug-v1` header, module identity,
primary/application/set/atomic fingerprints, and:

```text
edges: 1
  edge#0 condition=0 ordinal=0 formula=0
```

Validation requires exact lower profiles and fingerprints, equal
source/module, the direct wrapper-to-atomic arena relation, equal
condition/formula range/spelling/context/recovery, exact operand edges and
requests, and absence of duplicate Task-255 ownership. Missing, duplicated,
reordered, substituted, copied, stale, or wrong-profile inputs fail before
publication. The new typed/resolved optional handoff installs only after all
four lower handoffs and excludes any Task-257 composite/Task-257B composition
in this bounded profile. `TypedAstError` and `ResolvedTypedAstError` each add
the dedicated `InvalidSourceConditionFormulaComposition` variant. Final
assembly revalidates and clone-preserves it.

At the frozen pre-Task-256C1 baseline, this transaction was executable only
after the separate condition-container compatibility prerequisite made the
authenticated Task-255-encloses-Task-256 relation valid in both lower-handoff
installation orders without weakening arbitrary overlap rejection. Task
256C1 now passes both orders. No C2 production edit starts before fresh
post-commit preflight of that completed prerequisite.

All existing Task-257B input literals, producer calls, tables, installer
signature, successful legacy fingerprints, and debug bytes remain unchanged.
The legacy Task-257A and combined Task-257B installers add reciprocal checks:
if C2 is already installed, they reject atomically through their existing
`InvalidSourceCompositeFormula` and `InvalidSourceFormulaComposition`
variants. Conversely, the C2 installer rejects already installed A/B through
`InvalidSourceConditionFormulaComposition`; tests cover both installation
orders and byte-identical rollback. The exact private consumer
reuses the Task-255C1 selector, Task-253 imported-`++` seam, and a reusable
Task-256 equality builder in one surface-indexed arena. It adds no fixture:
the existing fail sidecar keeps the same definition-intake diagnostic and may
gain only the reciprocal Task-257C2 spec reference. One new covered trace row
may map only to that sidecar.

Equality truth, generator binding/reference/capture, predicate-chain
conjunction or segment negation, formula facts/results, sethood/result typing,
definition/theorem acceptance, proof/IR/VC, and broader comprehension
coverage remain deferred. This documentation prerequisite changed no
production, fixture, sidecar, trace, count, test list, or hash. The separate
Task-257C2 implementation commit has since completed the frozen transaction
after fresh post-Task-256C1 preflight. It adds the dedicated public handoff,
producer, table, dense ID, error surface, typed/resolved ownership, exact
private runner consumer, three checker tests, four runner tests, the single
covered trace row, and the reciprocal sidecar reference without changing any
fixture or semantic diagnostic. Measured exit is plan `419/386`, type
`252/240`, libraries `332/361`, and active
parse/declaration/type/proof `101/5/198/1`.

## Task 257C3 Frozen Predicate-Chain Composition

Task 257C3 is a third independent transaction in this module. It reuses the
existing 107-byte Task-257C1 pass consumer and authenticates only how its two
already validated predicate segments compose. It does not create
`SourceCompositeFormula` rows or semantic formula results.

The handoff input contains two dense tables. Exact profile `1/1` is:

```text
conjunction#0 formula=0 ordinal=0 left_segment=0 right_segment=1 boundary=1
negation#0 formula=0 ordinal=0 segment=1
```

The public families are
`SourcePredicateChainConjunction{Id,Input,Table}`,
`SourcePredicateChainNegation{Id,Input,Table}`,
`SourcePredicateChainCompositionHandoffInput`,
`SourcePredicateChainCompositionHandoff`,
`SourcePredicateChainCompositionProducer`, and non-exhaustive
`SourcePredicateChainCompositionError`. Rows expose exactly their input
fields; IDs and tables expose the standard dense accessors. The handoff
exposes source/module, exact Task-252 and Task-256 debug fingerprints, both
tables, and deterministic `debug_text()`.

The exact public ID, row, and table signatures are:

```rust
impl SourcePredicateChainConjunctionId {
    pub const fn new(index: usize) -> Self;
    pub const fn index(self) -> usize;
}
impl SourcePredicateChainNegationId {
    pub const fn new(index: usize) -> Self;
    pub const fn index(self) -> usize;
}
impl SourcePredicateChainConjunction {
    pub const fn formula(&self) -> SourceAtomicFormulaId;
    pub const fn ordinal(&self) -> usize;
    pub const fn left_segment(&self) -> SourcePredicateSegmentId;
    pub const fn right_segment(&self) -> SourcePredicateSegmentId;
    pub const fn boundary(&self) -> SourceAtomicEdgeId;
}
impl SourcePredicateChainNegation {
    pub const fn formula(&self) -> SourceAtomicFormulaId;
    pub const fn ordinal(&self) -> usize;
    pub const fn segment(&self) -> SourcePredicateSegmentId;
}
impl SourcePredicateChainConjunctionTable {
    pub fn get(
        &self,
        id: SourcePredicateChainConjunctionId,
    ) -> Option<&SourcePredicateChainConjunction>;
    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourcePredicateChainConjunctionId,
            &SourcePredicateChainConjunction,
        ),
    >;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}
impl SourcePredicateChainNegationTable {
    pub fn get(
        &self,
        id: SourcePredicateChainNegationId,
    ) -> Option<&SourcePredicateChainNegation>;
    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourcePredicateChainNegationId,
            &SourcePredicateChainNegation,
        ),
    >;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}
```

The producer receives the input, `SourcePrimaryTermHandoff`,
`SourceAtomicFormulaHandoff`, and the common `TypedArena`. It reauthenticates
Task-252 `3/0/3`, Task-256 `1/0/2/2/2/0/0/3/2`, the two same-symbol imported
predicate candidates, positive segment 0, exact negative `does not` segment
1, and the canonical root spelling. Conjunction 0 must reuse boundary edge 1
as segment 0's right and segment 1's left edge; that existing
`PredicateChainBoundary` targets primary 1. Negation 0 targets only segment
1. No lower row or resolver provenance is copied.

The stable header is
`source-predicate-chain-composition-debug-v1`, followed by module identity,
primary/atomic fingerprints, the conjunction count/row, and the negation
count/row in that order. The exact error signature is:

```rust
#[non_exhaustive]
pub enum SourcePredicateChainCompositionError {
    DependencyMismatch,
    InvalidConjunction {
        conjunction: SourcePredicateChainConjunctionId,
    },
    InvalidNegation {
        negation: SourcePredicateChainNegationId,
    },
    InvalidAggregate,
}
```

The producer checks both table cardinalities before either row validator.
Any conjunction or negation cardinality other than exactly one yields
`InvalidAggregate`. With exact `1/1` cardinality it validates conjunction 0
and then negation 0; any invalid field yields `InvalidConjunction` or
`InvalidNegation` with the corresponding strongly typed ID.

Typed/resolved ownership is optional, one-shot, revalidated, and
clone-preserved through `source_predicate_chain_composition()`. Dedicated
typed/resolved errors are `InvalidSourcePredicateChainComposition`. The
handoff is reciprocally exclusive with Task-257A composite, Task-257B
composition, and Task-257C2 condition composition in all installation
orders. Existing B/C2 successful fingerprints and debug bytes remain
unchanged when this optional handoff is absent.

C3-after-A/B/C2 fails with
`TypedAstError::InvalidSourcePredicateChainComposition`. The three reverse
orders fail with, respectively,
`TypedAstError::InvalidSourceCompositeFormula`,
`TypedAstError::InvalidSourceFormulaComposition`, and
`TypedAstError::InvalidSourceConditionFormulaComposition`. All six paths
publish nothing, preserve byte-identical state, and permit replay. In typed
and resolved debug output, the C3 chunk occupies the final mutually
exclusive formula-owner slot after Task-252 source-term, Task-256
source-atomic-formula, and the A/B/C2 slots, immediately before the existing
node/table section.

The later implementation reuses the existing Task-257C1 fixture and may
change only its sidecar reference/note plus one covered trace row
`spec.en.checker.type_elaboration.source_predicate_chain_composition`.
That row is required, has stage `type_elaboration`, status `covered`, and
coverage `pass`; its canonical source is
`doc/design/mizar-checker/en/source_formula_composition.md`, section
`Task 257C3 Frozen Predicate-Chain Composition`, and its sole mapped test is
the existing Task-257C1 sidecar. That sidecar's exact ordered spec-reference
set becomes the existing
`spec.en.checker.type_elaboration.source_predicate_chain_segment_payload`
followed by
`spec.en.checker.type_elaboration.source_predicate_chain_composition`.
The new row credits only the syntax-free association.
Predicate signature answers, overload selection, conjunction/negation truth,
formula facts/results, theorem acceptance, proof, IR/VC, and broader chains
remain deferred. This documentation prerequisite changes no executable
artifact; baseline remains plan `419/386`, type `252/240`, libraries
`332/361`, active `101/5/198/1`, and runner production 29 paths / 34,064
lines.

## Task 257C3 Implementation Result

The frozen third transaction is implemented with public dense conjunction
and negation IDs/tables, immutable input/handoff/producer/error surfaces,
two lower debug fingerprints, exact accessors, and stable debug text.
Validation reauthenticates both lower installations and their exact profiles
before cardinality, conjunction row 0, then negation row 0. Coherent but
wrong lower profiles, stale arenas/fingerprints, substituted rows, and every
cardinality/row precedence combination fail with the frozen typed error and
permit replay.

The runner publishes only the exact `1/1` handoff from the existing fixture.
No predicate token, candidate, resolver contribution, lower edge, truth,
fact, diagnostic, or semantic result is duplicated or inferred. Exactly
three checker and four runner tests cover the complete contract, and the
single covered trace row credits only this syntax-free association.

## Task 258B4A Downstream Statement Consumer

Task 258B4A is the first upper consumer of the Task-257B1
explicit-universal composite/composition pair. The lower tables, fingerprints,
`1/0/1/1/1/0/2` and `1/2` profiles, and
`SourceFormulaRootOwnership::UnassignedStatement` remain byte-for-byte
unchanged. The new statement constructor only reauthenticates those public
handoffs and targets `SourceCompositeFormulaId::new(0)`.

The paired typed installer is owned by Task 258, not by this module. It must
install the existing composite and formula-composition handoffs together
with the statement or publish nothing. No lower API, row, root ownership,
debug grammar, truth, binder semantics, or coverage credit is added here.

Repeated read-only documentation review reports **NO FINDINGS**. Independent
final quality passes all nine hard gates with no cap at valid `100/100`;
only staging, commit, and post-commit inventory remain.

## Task 258B4A Implemented Downstream Statement Consumer

The upper producer now consumes the already validated Task-257B1
composite/composition pair through its public checker handoffs. The lower
`1/0/1/1/1/0/2` and `1/2` rows, fingerprints, debug text, selector, and
`UnassignedStatement` ownership are unchanged. B4A adds only the upper
statement/candidate links to `Composite(0)` and the paired Task-258
transaction. The runner visibility edit exposes the existing validated
lower output only within `crate::runner`; it changes no public API or lower
behavior.

## Task 258B4B Downstream Statement Consumer

B4B consumes the existing Task-257B2 composite/composition pair unchanged:
`8/6/1/1/1/7/9` plus `8/0`, with Task-252 `16/0/16`,
Task-256 `8/0/0/0/0/0/0/16/16`, binding `2/1/4`, six
wrappers, seven same-family edges, eight atomic edges, no bound use, and
`UnassignedStatement` root ownership. Its private double-LF selector changes
no lower range, row, fingerprint, debug byte, or active 166-byte behavior.
The equivalent historical Task-257B2 shorthand is
`8/0/0/0/0/0/16/16`; the extra zero above is the current explicit,
empty predicate-segment slot.

The upper statement and candidate alone reference root `Composite(0)`.
Inner biconditional/disjunction/conjunction/repetition rows, wrappers,
equalities, and numerals acquire no upper ownership or semantic result. The
runner helper is already available through B4A's crate-private seam, so this
lower owner is a required no-op in B4B.

## Task 258B4C Downstream Statement Consumer

B4C consumes the existing Task-257B3 composite/composition handoffs:
binding `4/4/0`, Task 252 `6/6/0`, Task 256
`3/0/0/0/0/0/0/6/6`, Task 257 `3/0/1/3/3/2/6`, and composition
`3/6`. The lower composite roots at `67..136`, `92..136`, and
`110..136`, binder segments, three equality roots, six references, and six
bound-use edges remain unchanged. Composite root 0 remains
`UnassignedStatement`; the upper statement/candidate alone target it.

The active 138-byte source is frozen by its sidecar and trace row as
lower-only. B4C therefore uses the same source plus a second final LF,
exactly 139 bytes with SHA-256
`36e5a68a92451590644951838a9af8926212bd78f88d1f90563f12b650b161c1`.
Before upper work, a separate lower-stage prerequisite must update only
runner `type_elaboration/source_formula.rs` and
`runner/tests/type_elaboration/source_formula_composition.rs` so the
Task-257B3 selector accepts the exact active 138-byte hash
`cbfd7077713e8e9630900e349d5f579251c19fba55434acb62170ea1dd940237`
and private 139-byte hash, while rejecting the same text with zero or three
final LF bytes.

Production runner `source_formula_composition.rs` and both checker lower
owners remain unchanged in that prerequisite and in B4C. The change does
not broaden structural matching, alter a row/fingerprint/debug byte, or
credit active upper behavior. Truth, restriction discharge, existential
witnesses, capture, facts, theorem acceptance, proof, and IR remain
deferred.

## Task 257C4A Fraenkel generator binding context

Task 257C owns the completed standalone C4A handoff described by the canonical
[contract](../../task_contracts/en/CHECKER-FRAENKEL-GENERATOR-BINDING-257C4A.md).
This module exposes the binding-context ID/row/table, use-position
ID/row/table, handoff, producer, and error only. Its producer consumes exact
277C structural composition, R2 resolver collection, and `TypedAst`; opaque,
version/domain-tagged dependency snapshots revalidate their full fields and
the R2-to-277C-to-typed binder relation. The three use rows normalize actual
resolver positions only; they own no term, reference, or capture. C4A is
default-deny and produces no formula/type/sethood/evidence/verdict/diagnostic,
Typed/Resolved install, or production route. C4B must consume its handoff
before mapping a use or capture.
The implemented owner is 7303 lines, SHA-256
`f6da763061479e74e7b8f39169ecad311bb9bf879e91e93824d9899798017abc`.
All four exact checker tests, the 550-test package library, formatting,
workspace Clippy, full workspace tests, both lint suites, metadata, and
implementation/test reviews pass. Independent source/documentation and
bilingual/boundary reviews report **NO FINDINGS**. Final-quality is **NO
FINDINGS**, with `9/9` hard gates passing at valid uncapped `100/100`
(`20/20/15/15/10/10/5/5`). The historical pre-commit staging/cached review
passed. The task-only commit, immediate post-commit proof, and accepted
fresh-inventory disposition
are closed in the language-local [historical
checkpoint](../../task_contracts/en/CHECKER-FRAENKEL-GENERATOR-BINDING-257C4A.md#historical-immediate-post-implementation-checkpoint).
C4B remains unselected and requires a separately frozen post-closure
documentation prerequisite; Task 277B remains not ready with zero semantic
credit.

## Task 257C4B Fraenkel generator bound-use transport

The frozen [C4B contract](../../task_contracts/en/CHECKER-FRAENKEL-GENERATOR-BOUND-USE-257C4B.md)
adds one independent, syntax-free association transaction to this existing
module. Its producer consumes only the completed C4A handoff, revalidates the
whole opaque C4A dependency snapshot, and maps the three normalized positions
through the existing C4A `BindingEnv`. It creates no Task-252 term/reference,
role copy, capture, formula, type, sethood, request, verdict, diagnostic,
installation, or route.

The exact public family is:

```rust
SourceFraenkelGeneratorBoundUseId
SourceFraenkelGeneratorBoundUse
SourceFraenkelGeneratorBoundUseTable
SourceFraenkelGeneratorBoundUseHandoff
#[non_exhaustive] SourceFraenkelGeneratorBoundUseError
SourceFraenkelGeneratorBoundUseProducer
```

`SourceFraenkelGeneratorBoundUseId` exposes only:

```rust
new(index: usize) -> Self
index(self) -> usize
```

The immutable row exposes exactly:

```rust
use_position() -> SourceFraenkelGeneratorUsePositionId
binding_context() -> SourceFraenkelGeneratorBindingContextId
resolver_use_index() -> usize
source_ordinal() -> usize
lookup_ordinal() -> usize
context() -> BindingContextId
binding() -> BindingId
```

The table exposes exactly:

```rust
get(id: SourceFraenkelGeneratorBoundUseId) -> Option<&SourceFraenkelGeneratorBoundUse>
iter() -> impl Iterator<Item = (SourceFraenkelGeneratorBoundUseId, &SourceFraenkelGeneratorBoundUse)>
len() -> usize
is_empty() -> bool
```

The handoff exposes exactly:

```rust
source_id() -> SourceId
module_id() -> &ModuleId
dependency_summary() -> &str
bound_uses() -> &SourceFraenkelGeneratorBoundUseTable
debug_text() -> String
```

`dependency_summary()` is the exact C4A human summary
`source-fraenkel-generator-binding-context-v1|module=<package>.<path>|bindings=1|use-positions=3`.
It is non-authoritative and is never trusted without full snapshot validation.
The C4B debug text is exactly
`source-fraenkel-generator-bound-use-v1|module=<package>.<path>|bound-uses=3`.

The producer signature is exactly:

```rust
SourceFraenkelGeneratorBoundUseProducer::build(
    binding_context: &SourceFraenkelGeneratorBindingContextHandoff,
) -> Result<
    SourceFraenkelGeneratorBoundUseHandoff,
    SourceFraenkelGeneratorBoundUseError,
>
```

The handoff contains no public lower-dependency getter. Privately it retains a
clone in a snapshot whose exact version is
`source-fraenkel-generator-bound-use-dependency-v1` and exact domain is
`source-fraenkel-generator-bound-use`. Both `build` and handoff validation
check the wrapper environment, version/domain, C4A full validation, exact C4A
summary, then dense C4B rows. C4A full validation transitively authenticates
its R2, 277C, `TypedAst`, binding-table, use-position, and `BindingEnv` state;
C4B does not reproduce or weaken that validation.

The non-exhaustive error has exactly these variants and this precedence:

```rust
EnvironmentMismatch
InvalidBindingContextDependency
InvalidBoundUse { bound_use: SourceFraenkelGeneratorBoundUseId }
```

`EnvironmentMismatch` wins when the C4B wrapper source/module differs from its
retained C4A dependency. `InvalidBindingContextDependency` next covers a stale
version/domain, any C4A validation failure, or summary mismatch. Only after the
dependency is valid does the lowest invalid dense C4B row return
`InvalidBoundUse`; a wrong total count reports ID 0. No lower C4A error or raw
resolver identity is exposed.

For F5, bound-use IDs 0/1/2 map respectively from C4A use-position IDs 0/1/2,
all through binding-context ID 0 and context 1 to binding 0. Their
resolver-use/source ordinals are 0/1/2 and lookup ordinals are 1/2/3. Each
lookup must be `Local(binding0)`. Ordinal 0 remains C4A's separate
`ForwardReference` probe and is never a C4B row. Validation requires exact
getter equality against the retained C4A rows and exact `BindingEnv::lookup`
results; it does not infer an ordinal from a source range or spelling.

Default-deny accepts only the complete three-row normal profile. Missing,
extra, reordered, duplicated, recovered, stale, non-local, nested,
multiple-generator, or shadowed state fails atomically. The four exact checker
tests named by the contract cover the literal ABI/oracle, wrapper and complete
snapshot corruption, row/lookup corruption plus precedence, and deterministic
non-mutating replay. The raw checker list is frozen to change `550 -> 554`.

The implementation is complete in this existing owner at 7958 lines / SHA-256
`90b339d9707f9f8d847b678721e8db0ef6a00e4a15dbb41474a0cf6980f47168`.
It publishes exactly the family and getters above, validates the full retained
C4A snapshot before its three rows, and adds exactly the four named checker
tests. Focused `4/4`, checker `554/554`, formatting, package/full-workspace
Clippy, full workspace tests, metadata/public-enum suites, five unchanged CLI
replays, and diff checks pass. Independent implementation and test-sufficiency reviews are
**NO FINDINGS** after structural and `TypedAst` dependency corruption were
added to the existing precedence test. Final source/documentation review and
independent bilingual/boundary review are **NO FINDINGS** after the sole Low
baseline/current wording repair. Final-quality is **NO FINDINGS**; all `9/9`
hard gates pass at valid uncapped `100/100`
(`20/20/15/15/10/10/5/5`). The contract's exact 23-path staging/cached review
also passes. The task-only commit, immediate post-commit proof, and accepted
fresh semantic STOP are closed at the language-local [historical
checkpoint](../../task_contracts/en/CHECKER-FRAENKEL-GENERATOR-BOUND-USE-257C4B.md#historical-immediate-post-implementation-checkpoint).
No successor is selected.

## Task 257C4C3 nested Fraenkel binder/mapper-use transport

The frozen [C4C3 contract](../../task_contracts/en/CHECKER-FRAENKEL-NESTED-BINDER-USE-257C4C3.md)
adds an independent zero-semantic transaction to this existing Task-257C
owner. It consumes only the completed C4C2 resolver collection and an exact
normal `TypedAst`; C4A/C4B remain exact-F5 compatibility-negative profiles and
their APIs and validators are not reused or weakened.

The completed implementation exposes exactly
`SourceNestedFraenkelBinderUseId`,
`SourceNestedFraenkelBinderUse`, `SourceNestedFraenkelBinderUseTable`,
`SourceNestedFraenkelBinderUseHandoff`, non-exhaustive
`SourceNestedFraenkelBinderUseError`, and
`SourceNestedFraenkelBinderUseProducer`. The row exposes only resolver use
index, resolver binding ID, outer binder `TypedNodeId`, inner mapper-use
`TypedNodeId`, and source ordinal. Its table exposes dense read-only
`get`/`iter`/`len`/`is_empty`; the handoff exposes source, module, resolver
summary, the table, and deterministic debug text.

The resolver summary is exactly
`fraenkel-generator-variable-source-v1|module=<package>.<path>|bindings=2|uses=1`
and remains non-authoritative. The exact handoff debug grammar is
`source-nested-fraenkel-binder-use-v1|module=<package>.<path>|binder-uses=1`.
The non-exhaustive error contains exactly `EnvironmentMismatch`,
`InvalidResolverDependency`, `InvalidTypedDependency`, and
`InvalidBinderUse { binder_use: SourceNestedFraenkelBinderUseId }`, in that
validation precedence.

The producer authenticates the complete retained C4C2 two-binding/one-use
profile, unique normal resolved-to-typed mapping, exact typed structure and
ranges, then publishes exactly row 0 linking mapper use 0 at `x@94..95` to
outer binding 1 at `x@136..137`. C4C4 adds only a crate-private
`validate_complete()` reauthentication entry point; it exposes no retained
resolver or typed-AST getter. The separate Task-252 owner consumes the complete
C4C3 handoff by value and cannot bypass this validation. C4C3 itself creates no Task-252 occurrence,
`BindingEnv`, checker binding, capture, formula, type/sethood, request/result,
verdict, diagnostic, installation, route, or coverage credit. Exact validation
order, snapshot tags, default-deny matrix, tests, and completion evidence are
owned by the contract.

## Task 257C4C5 nested Fraenkel capture-identity receipt

The implemented [C4C5 contract](../../task_contracts/en/CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-257C4C5.md)
selects this existing Task-257C module as the sole owner of the first
post-C4C4 capture-identity receipt. Its exact public family is the dense
`SourceNestedFraenkelCaptureIdentityId`, immutable row/table/handoff,
non-exhaustive `SourceNestedFraenkelCaptureIdentityError`, and
`SourceNestedFraenkelCaptureIdentityProducer`.

The producer consumes and retains one complete C4C4 handoff by value
and publishes exactly one immutable association row. Row 0 ties inner owner
context `2` / range `92..123`, mapper term `0`, mapper reference `0`, checker
binding `0`, resolver use `0`, resolver binding `1`, and source ordinal `0`.
The resolver binding identity, not spelling `x` or equality between unrelated
numeric IDs, authenticates the association. Missing, extra, duplicate,
reordered, stale, recovered, or mismatched state fails closed.

The standalone handoff is the complete destination for this task. There is no
`TypedAst` or `ResolvedTypedAst` installation, generalized capture or Core
parameter order, Task-255 dependency, Core adapter, generated origin, semantic
capture, or change to C4C4's empty `BindingEntry::captured`. The exact API,
validation precedence, corruption oracle, tests, Core-33/Core-35 boundary, and
forbidden behavior are owned once by the paired contract.

Four checker-owner tests cover the exact public/debug surface, dependency
precedence and both error strings, owner corruption, cardinality and every row
field, deterministic replay, by-value retention, and empty captured state. The
sole current consumer is the existing private imported-fixture leaf; it calls
the complete C4C1-C4C5 route without registry or active dispatch.

## Task 257C4C6 Capture-Identity Installation Boundary

The frozen [C4C6 contract](../../task_contracts/en/CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-INSTALLATION-257C4C6.md)
keeps this module as sole validator of the complete C4C5 receipt and adds one
crate-private typed-owner validation seam. The seam replays the full C4C5
transaction and authenticates the destination against the exact retained C4C3
pre-install `TypedAst`; source/module equality alone is insufficient. No
retained snapshot or unchecked constructor becomes public.

C4C3 rejects an already installed C4C6 owner and any populated resolved root,
local context, type, fact, coercion, initial-obligation, or typed-diagnostic
table, so a later receipt cannot recursively retain another receipt or carry
semantic typed state. The seam installs no captured state, semantic result,
Core identity, generated parameter/origin, or ordering rule. Exact AST storage,
final clone, and mutual exclusion belong to the linked Typed/Resolved owners.

## Task 257C4C7 Multi-capture Projection Boundary

The accepted [C4C7 contract](../../task_contracts/en/TEST-FRAENKEL-NESTED-MULTI-CAPTURE-257C4C7.md)
keeps this existing Task-257C family as the sole future checker owner of a
complete immutable, syntax-free, Core-ID-free nested-Fraenkel projection. It
must authenticate association by resolved binder identity and cover the
complete generator/mapper/optional-predicate/capture-occurrence graph with
owner, context, and source provenance. Repeated occurrences of one identity
form one distinct capture. Authenticated outer declaration/source order may be
used only as a private deterministic transport convention.

C4C7 creates no API or implementation. Its inactive two-capture oracle asserts
only that both outer identities are captured and the inner generator is local;
it does not assert order. A later contract must freeze the exact API and
default-deny matrix. C4C6 remains the immutable Typed/Resolved receipt;
standalone projection work must not add a second AST slot, reinterpret IDs, or
move Core-33 allocation or Core-35 lowering/GeneratedOrigin ownership here.
