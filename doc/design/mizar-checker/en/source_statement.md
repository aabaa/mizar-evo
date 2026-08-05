# Source Statement Transport

> Canonical language: English. Japanese companion:
> [../ja/source_statement.md](../ja/source_statement.md).

This component is the syntax-free checker boundary for source theorem owners,
statement shells, visibility-scoped inputs, and unverified candidate
propositions. It does not parse statements, resolve labels, prove formulas, or
publish theorem facts.

## Task 258A Frozen Reserved-Variable Theorem Slice

Task 258A is the first bounded slice of Checker Task 258. Its exact future
`MT10-FS` source is:

```mizar
reserve x for set;
theorem FormulaStatementReservedVariableEqualitySmoke: x = x;
```

The source is exactly 81 UTF-8 bytes including the final LF and has SHA-256
`341aad596ef6891dfa33c189895df2350d357ac8edaf3747f160bbad7a2ddd96`.
The normal source ranges are:

| Occurrence | Half-open byte range |
|---|---:|
| reserve declaration | `0..18` |
| written reserve type `set` | `14..17` |
| theorem item / owner | `19..80` |
| theorem label | `27..72` |
| equality / formula | `74..79` |
| left `x` | `74..75` |
| `=` | `76..77` |
| right `x` | `78..79` |

Canonical authority is Chapter 4 §§4.3 and 4.7.1, Chapter 14 §14.5.2,
Chapter 15 §§15.8 and 15.10, and Chapter 16 §§16.1, 16.2, 16.7.1, and
16.9. Chapter 4 makes a free reserved theorem identifier an implicitly
universally closed variable with its reserved type. Chapter 16 requires a
named theorem owner and permits omitted justification for an unmodified item,
but automatic proof and publication occur only after verification. Task 258A
therefore transports the reserved type guard as a visible input and the
equality as an unverified candidate; it does not assert either equality truth
or theorem acceptance.

### Exact lower inputs and resolver provenance

The private extractor must consume a real final-LF frontend AST and the real
resolver `SymbolEnv`. It selects exactly one normal reserve item followed by
one normal, unmodified `theorem` with the exact label and direct,
unparenthesized reserved-variable equality above. It reuses:

- one Task-48-derived normal module `BindingEnv` with context 0 and active,
  visible `ReservedVariable` binding 0 for `x`, identifier declaration range
  `8..9` within reserve item `0..18`, source type site `14..17`, and first-use
  ordinal 1;
- Task 252 profile `2/2/0`: two `VariableReference` / `Value` primaries and
  two independently authenticated references to binding 0, each with
  Task-252 stored use ordinal 1;
- Task 256 profile `1/0/0/0/0/0/2/2`: one normal `Equality`, two built-in
  operand edges to Task-252 primaries 0 and 1, and two unresolved operand
  expected-type requests;
- the shared typed arena in which the theorem node contains the direct
  `FormulaExpression` wrapper, that wrapper contains the atomic formula
  occurrence, and the formula subtree contains both term occurrences in
  left-to-right order.

Task 248 supplies the canonical binding/context model but its current exact
`SourceBindingContextHandoff` profiles do not admit a reserve-plus-theorem
transaction and are not fabricated or extended here. Task 249 and Tasks
253–255 are absent. Task 257 formula-owner handoffs are absent because this
exact formula is atomic.

The owner is authenticated against all resolver views. There must be exactly
one local source theorem symbol for the current source/module.
`CheckedStatementOwner::validate_exact_local_theorem` must accept it.
`SymbolEntry`, `DefinitionEntry`, `LabelEntry`, and the checked owner agree
on source/module, contribution, normal theorem origin `19..80`, spelling,
visibility, and export status. The runner's exact label selector/projection
range is `27..72`; the published `LabelEntry` carries the shared theorem
origin rather than a separate declaration-range field.
The module-wide `SourceContribution` retains the real resolver anchor at the
first declaration shell, reserve `0..18`, rather than fabricating a
theorem-local contribution. The exact kinds are `SymbolKind::Theorem`,
`DefinitionKind::Theorem`, and `LabelKind::Theorem`; visibility/export are
`Public` / `Exported`; the contribution is `LocalSource`, contains the
theorem symbol, definition, and label effects, and has no import edge.
Recovered, imported, summary, missing, duplicated, stale, cross-contribution,
wrong-kind, private, local-only, or source-inconsistent provenance fails
before publication.

### Frozen syntax-free API

The later implementation adds one public `source_statement` module with five
dense tables:

```rust
pub struct SourceTheoremOwnerId(usize);
pub struct SourceStatementId(usize);
pub struct SourceStatementContextId(usize);
pub struct SourceStatementInputFactId(usize);
pub struct SourceStatementCandidateFactId(usize);

pub struct SourceStatementHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub owners: Vec<SourceTheoremOwnerInput>,
    pub statements: Vec<SourceStatementInput>,
    pub contexts: Vec<SourceStatementContextInput>,
    pub input_facts: Vec<SourceStatementInputFactInput>,
    pub candidate_facts: Vec<SourceStatementCandidateFactInput>,
}

pub struct SourceTheoremOwnerInput {
    pub symbol: SymbolId,
    pub contribution: SourceContributionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub role: SourceTheoremRole,
    pub status: SourceTheoremStatus,
    pub recovery: SourceStatementRecovery,
}

pub struct SourceStatementInput {
    pub owner: SourceTheoremOwnerId,
    pub context: SourceStatementContextId,
    pub formula: SourceStatementFormulaTarget,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub spelling: String,
    pub kind: SourceStatementKind,
    pub recovery: SourceStatementRecovery,
}

pub struct SourceStatementContextInput {
    pub statement: SourceStatementId,
    pub binding_context: BindingContextId,
    pub source_range: SourceRange,
    pub visible_bindings: Vec<BindingId>,
}

pub struct SourceStatementInputFactInput {
    pub statement: SourceStatementId,
    pub context: SourceStatementContextId,
    pub ordinal: usize,
    pub kind: SourceStatementInputFactKind,
    pub binding: BindingId,
    pub uses: Vec<SourcePrimaryTermReferenceId>,
}

pub struct SourceStatementCandidateFactInput {
    pub statement: SourceStatementId,
    pub context: SourceStatementContextId,
    pub ordinal: usize,
    pub kind: SourceStatementCandidateFactKind,
    pub formula: SourceStatementFormulaTarget,
}

#[non_exhaustive]
pub enum SourceTheoremRole {
    Theorem,
}

#[non_exhaustive]
pub enum SourceTheoremStatus {
    Unmodified,
}

#[non_exhaustive]
pub enum SourceStatementKind {
    TheoremProposition,
}

#[non_exhaustive]
pub enum SourceStatementRecovery {
    Normal,
    Degraded,
}

#[non_exhaustive]
pub enum SourceStatementFormulaTarget {
    Atomic(SourceAtomicFormulaId),
}

#[non_exhaustive]
pub enum SourceStatementInputFactKind {
    ReservedTypeGuard,
}

#[non_exhaustive]
pub enum SourceStatementCandidateFactKind {
    UnverifiedProposition,
}
```

Each ID is a dense zero-based row index with public `new(index)` and
`index()` accessors. The immutable row types expose read-only accessors.
Every table exposes only `get(id)`, source-ordered `iter()`, `len()`, and
`is_empty()`. `SourceStatementHandoff` owns an exact clone of the
producer-validated `BindingEnv` and exposes it through
`binding_env()`. It also exposes source/module, the quoted
`binding_env.debug_text()` fingerprint, Task-252 and Task-256 fingerprints,
the checked owner, all five tables, and deterministic `debug_text()`. The
owned environment is part of equality and clone preservation; there are no
mutable or unchecked public constructors.

```rust
pub struct SourceTheoremOwner { /* immutable validated owner fields */ }
impl SourceTheoremOwner {
    pub const fn symbol(&self) -> &SymbolId;
    pub const fn contribution(&self) -> SourceContributionId;
    pub const fn site(&self) -> &TypedSiteRef;
    pub const fn source_range(&self) -> SourceRange;
    pub fn spelling(&self) -> &str;
    pub const fn role(&self) -> SourceTheoremRole;
    pub const fn status(&self) -> SourceTheoremStatus;
    pub const fn recovery(&self) -> SourceStatementRecovery;
}

pub struct SourceStatement { /* immutable validated statement fields */ }
impl SourceStatement {
    pub const fn owner(&self) -> SourceTheoremOwnerId;
    pub const fn context(&self) -> SourceStatementContextId;
    pub const fn formula(&self) -> SourceStatementFormulaTarget;
    pub const fn site(&self) -> &TypedSiteRef;
    pub const fn source_range(&self) -> SourceRange;
    pub const fn source_ordinal(&self) -> usize;
    pub fn spelling(&self) -> &str;
    pub const fn kind(&self) -> SourceStatementKind;
    pub const fn recovery(&self) -> SourceStatementRecovery;
}

pub struct SourceStatementContext { /* immutable validated context fields */ }
impl SourceStatementContext {
    pub const fn statement(&self) -> SourceStatementId;
    pub const fn binding_context(&self) -> BindingContextId;
    pub const fn source_range(&self) -> SourceRange;
    pub fn visible_bindings(&self) -> &[BindingId];
}

pub struct SourceStatementInputFact { /* immutable validated input fields */ }
impl SourceStatementInputFact {
    pub const fn statement(&self) -> SourceStatementId;
    pub const fn context(&self) -> SourceStatementContextId;
    pub const fn ordinal(&self) -> usize;
    pub const fn kind(&self) -> SourceStatementInputFactKind;
    pub const fn binding(&self) -> BindingId;
    pub fn uses(&self) -> &[SourcePrimaryTermReferenceId];
}

pub struct SourceStatementCandidateFact {
    /* immutable validated candidate fields */
}
impl SourceStatementCandidateFact {
    pub const fn statement(&self) -> SourceStatementId;
    pub const fn context(&self) -> SourceStatementContextId;
    pub const fn ordinal(&self) -> usize;
    pub const fn kind(&self) -> SourceStatementCandidateFactKind;
    pub const fn formula(&self) -> SourceStatementFormulaTarget;
}

impl SourceTheoremOwnerTable {
    pub fn get(&self, id: SourceTheoremOwnerId) -> Option<&SourceTheoremOwner>;
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceTheoremOwnerId, &SourceTheoremOwner)>;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}
impl SourceStatementTable {
    pub fn get(&self, id: SourceStatementId) -> Option<&SourceStatement>;
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceStatementId, &SourceStatement)>;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}
impl SourceStatementContextTable {
    pub fn get(
        &self,
        id: SourceStatementContextId,
    ) -> Option<&SourceStatementContext>;
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceStatementContextId, &SourceStatementContext)>;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}
impl SourceStatementInputFactTable {
    pub fn get(
        &self,
        id: SourceStatementInputFactId,
    ) -> Option<&SourceStatementInputFact>;
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceStatementInputFactId, &SourceStatementInputFact)>;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}
impl SourceStatementCandidateFactTable {
    pub fn get(
        &self,
        id: SourceStatementCandidateFactId,
    ) -> Option<&SourceStatementCandidateFact>;
    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            SourceStatementCandidateFactId,
            &SourceStatementCandidateFact,
        ),
    >;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}

impl SourceStatementHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub const fn binding_env(&self) -> &BindingEnv;
    pub fn binding_fingerprint(&self) -> &str;
    pub fn primary_term_fingerprint(&self) -> &str;
    pub fn atomic_formula_fingerprint(&self) -> &str;
    pub const fn checked_owner(&self) -> &CheckedStatementOwner;
    pub const fn owners(&self) -> &SourceTheoremOwnerTable;
    pub const fn statements(&self) -> &SourceStatementTable;
    pub const fn contexts(&self) -> &SourceStatementContextTable;
    pub const fn input_facts(&self) -> &SourceStatementInputFactTable;
    pub const fn candidate_facts(&self) -> &SourceStatementCandidateFactTable;
    pub fn debug_text(&self) -> String;

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        primary_terms: &SourcePrimaryTermHandoff,
        atomic_formulas: &SourceAtomicFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<(), SourceStatementError>;
}
```

The five IDs derive `Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
Hash`. Inputs, immutable rows, tables, and the handoff derive
`Debug, Clone, PartialEq, Eq`. The seven data enums derive `Debug, Clone,
Copy, PartialEq, Eq, PartialOrd, Ord, Hash`. `SourceStatementError` derives
`Debug, Clone, PartialEq, Eq` and implements `Display` and `Error`.
`SourceStatementProducer` derives `Debug, Clone, Copy, Default`. No other
public trait implementation or constructor is part of Task 258A.

`SourceStatementProducer::build` takes the complete input plus
`&SymbolEnv`, `&BindingEnv`, `&SourcePrimaryTermHandoff`,
`&SourceAtomicFormulaHandoff`, and `&TypedArena`. It validates the whole
transaction before returning an immutable handoff. Its public signature is:

```rust
pub fn build(
    input: SourceStatementHandoffInput,
    symbols: &SymbolEnv,
    bindings: &BindingEnv,
    primary_terms: &SourcePrimaryTermHandoff,
    atomic_formulas: &SourceAtomicFormulaHandoff,
    arena: &TypedArena,
) -> Result<SourceStatementHandoff, SourceStatementError>;
```

The exact non-exhaustive error is:

```rust
#[non_exhaustive]
pub enum SourceStatementError {
    DependencyMismatch,
    InvalidOwner { owner: SourceTheoremOwnerId },
    InvalidStatement { statement: SourceStatementId },
    InvalidContext { context: SourceStatementContextId },
    InvalidInputFact { fact: SourceStatementInputFactId },
    InvalidCandidateFact { fact: SourceStatementCandidateFactId },
    InvalidAggregate,
}
```

### Exact `1/1/1/1/1` transaction

All five vectors contain exactly one row and every row has ordinal/id 0.
Owner 0 is the authenticated theorem label. Statement 0 spans `19..80`,
uses the theorem site, normal recovery, exact single-space token spelling,
kind `TheoremProposition`, owner 0, context 0, and
`Atomic(SourceAtomicFormulaId::new(0))`. The statement formula is the exact
Task-256 equality at `74..79`; the theorem arena node contains the direct
formula-expression wrapper, which contains the atomic occurrence and its
ordered term descendants, without another statement, proof, or justification
subtree.

Context 0 spans the theorem, uses binding context 0, and has exact visible
bindings `[BindingId::new(0)]`. Its binding environment must be normal,
source/module-identical, and contain no diagnostic. Input fact 0 is
`ReservedTypeGuard`, points to binding 0 and exact reference uses `[0, 1]`,
and is visible in context 0. Both references independently select active
normal reserved binding 0 through Task 252, and the binding declaration/type
site precede the theorem and both uses.

Candidate fact 0 is `UnverifiedProposition`, belongs to statement/context 0,
and points to the same atomic equality. It is deliberately not an input fact,
not visible to the statement itself or any later statement, and not a
`TypeFactId`, checked formula, theorem result, axiom, accepted premise, or
discharged goal.

The stable debug schema is:

```text
source-statement-debug-v1
module: <package>::<module>
binding-env-fingerprint: <quoted BindingEnv debug>
primary-term-fingerprint: <quoted Task-252 debug>
atomic-formula-fingerprint: <quoted Task-256 debug>
owner#0 symbol=<resolver id> contribution=<id> role=theorem status=unmodified range=19..80 site=<node> recovery=normal spelling="FormulaStatementReservedVariableEqualitySmoke"
statement#0 ordinal=0 owner=0 context=0 formula=atomic:0 kind=theorem-proposition range=19..80 site=<node> recovery=normal spelling="theorem FormulaStatementReservedVariableEqualitySmoke : x = x ;"
context#0 statement=0 binding_context=0 range=19..80 visible_bindings=[0]
input-fact#0 statement=0 context=0 ordinal=0 kind=reserved-type-guard binding=0 uses=[0, 1]
candidate-fact#0 statement=0 context=0 ordinal=0 kind=unverified-proposition formula=atomic:0
```

Missing, extra, duplicated, reordered, copied, substituted, stale, recovered,
wrong-source/module/range/spelling/site/ordinal/kind/status, wrong owner or
contribution, mismatched fingerprints, non-visible binding, wrong binding
kind/type site/status/recovery, missing or swapped term references, wrong
formula target, or input/candidate aliasing fails before publication.

Every validation entry point uses one total precedence order:

1. authenticate the source/module, owned `BindingEnv`, Task-252 handoff,
   Task-256 handoff, their exact profiles and stored fingerprints, the shared
   arena, and required lower installation available to that entry point; any
   failure in this dependency tier is `DependencyMismatch`;
2. check all five input/table cardinalities and dense aggregate order before
   inspecting any row; any missing, extra, duplicated, or reordered
   aggregate is `InvalidAggregate`; and
3. validate the first invalid row in owner, statement, context, input-fact,
   candidate-fact order and return its strongly typed row-local error.

Only `SourceStatementProducer::build` receives the live `SymbolEnv`. In its
owner-row tier, resolver theorem identity, contribution membership, label
effects, range, spelling, visibility, export, kind, origin, and
`CheckedStatementOwner::validate_exact_local_theorem` are properties of
owner row 0; their failure is `InvalidOwner { owner: 0 }`, not
`DependencyMismatch`. `validate_installation`, typed-AST installation, and
final assembly do not re-query resolver views. After their dependency and
aggregate tiers, they validate the immutable stored `CheckedStatementOwner`
against owner row 0 and then validate the remaining rows in the same order.
Mixed dependency/cardinality corruption therefore always yields
`DependencyMismatch`, and mixed aggregate/row corruption always yields
`InvalidAggregate`. No later entry point claims stronger resolver
reauthentication than its frozen inputs permit.

### Typed/final ownership and exclusions

`TypedAst` and `ResolvedTypedAst` gain one optional
`SourceStatementHandoff`, accessor, deterministic debug projection,
revalidation, and dedicated `InvalidSourceStatement` errors. Installation is
one-shot and requires the exact Task-252 and Task-256 handoffs first.
`ResolvedTypedAst::assemble` revalidates and clone-preserves the same object;
it does not reconstruct any row.

The handoff-owned `BindingEnv` is the exact object cloned from the producer
input. Installation and final assembly revalidate its source/module,
`binding-env-debug-v1` fingerprint, normal module context, exact active
reserved binding, visibility, declaration/type ranges, first-use ordinal,
absence of diagnostics, Task-252 reference winners, and statement-context
use. This makes stale or substituted binding provenance observable without
adding a `BindingEnv` parameter to either public installer or final
assembly.

Task 248 and Task 258A are mutually exclusive owners. The only production
ordering is a constructor-supplied Task-248 `SourceBindingContextHandoff`
followed by `with_source_statement`; it fails with
`TypedAstError::InvalidSourceStatement`. Task 248 has no post-construction
installer, and Task 258A does not add one. Checker tests exercise the reverse
logical attempt through the specifically named `#[cfg(test)]`
`with_source_context_for_test`, which runs the same private typed-AST
validation and fails with `TypedAstError::InvalidSourceContext`. Final
assembly rejects a state prepared only by the specifically named
`#[cfg(test)]` `inject_source_statement_for_test` bypass with
`ResolvedTypedAstError::InvalidSourceStatement`. All three failures publish
neither second owner nor final output, preserve byte-identical prior debug
state, and allow replay with the valid owner. This exact profile never
installs Task 248.

```rust
#[cfg(test)]
pub(crate) fn with_source_context_for_test(
    self,
    source_context: SourceBindingContextHandoff,
) -> Result<Self, TypedAstError>;

#[cfg(test)]
pub(crate) fn inject_source_statement_for_test(
    &mut self,
    statement: SourceStatementHandoff,
);
```

The source-statement debug chunk follows all Task-257 formula-owner chunks and
precedes the existing node/table section. Installing it changes no existing
lower fingerprint or semantic table. Failure publishes no field, preserves
byte-identical state, and permits valid replay.

Task 258A must leave `TypedAst::facts`, `ResolvedTypedAst::checked_formulas`,
the existing Task-266 `statement_semantics`, Task-268 `checked_proofs`,
checked proof nodes/terminal goals, cluster facts, diagnostics, CoreIr,
ControlFlowIr, VC, cache, and artifact output empty. The Task-266/268 exact
standalone-contradiction route retains sole ownership of its existing checked
statement/proof tables. No new `StatementSemanticInput`,
`StatementProofIntentInput`, `CheckedStatementOwner` constructor, or
accepted-status path is added.

The exact selector excludes lemmas, role aliases, status modifiers,
justifications and proof blocks, statements inside proofs, assumptions,
conclusions, witnesses, citations, local labels, composite or other atomic
formulas, parentheses, multiple reserves/theorems, shadowing, imports,
definitions, comments that alter exact bytes, missing final LF, recovered
syntax, synthetic-only ASTs, and every named near miss. Those broader
statement families remain Task 258B or Tasks 269–272.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceTheoremRole` | `#[non_exhaustive]`; Task 258A accepts only `Theorem`. |
| `SourceTheoremStatus` | `#[non_exhaustive]`; Task 258A accepts only `Unmodified`. |
| `SourceStatementKind` | `#[non_exhaustive]`; Task 258A accepts `TheoremProposition`, Task 258B1 additionally accepts exact `ProofStepProposition` and `Conclusion` rows, and Task 258B2 additionally accepts one exact unlabeled `Assumption` row. |
| `SourceStatementRecovery` | `#[non_exhaustive]`; callers tolerate `Degraded`, while this exact route accepts only `Normal`. |
| `SourceStatementFormulaTarget` | `#[non_exhaustive]`; Task 258A accepts only one Task-256 `Atomic` target. |
| `SourceStatementInputFactKind` | `#[non_exhaustive]`; Task 258A accepts only `ReservedTypeGuard`. |
| `SourceStatementCandidateFactKind` | `#[non_exhaustive]`; Task 258A accepts only `UnverifiedProposition`. |
| `SourceStatementWitnessTermTarget` | `#[non_exhaustive]`; Task 258B3 accepts only exact Task-252 `Primary` term 2. |
| `SourceStatementWitnessKind` | `#[non_exhaustive]`; Task 258B3 accepts only one `Unnamed` witness. |
| `SourceStatementLabelKind` | `#[non_exhaustive]`; Task 258B1 accepts only one resolver-authenticated `ProofStep` label. |
| `SourceStatementCitationTarget` | `#[non_exhaustive]`; Tasks 258B1/B5A use `Local(SourceStatementLabelId)`, while Task 258B5B uses `Imported` without fabricating a local label row. |
| `SourceStatementCitationKind` | `#[non_exhaustive]`; Tasks 258B1/B5A accept `SimpleLocal`, while Task 258B5B accepts `SimpleImported`. |
| `SourceStatementError` | `#[non_exhaustive]`; callers must not exhaustively match producer/installation failures. |
| `SourceStatementReferenceError` | `#[non_exhaustive]`; callers must not exhaustively match reference dependency, aggregate, label, or citation failures. |
| `SourceStatementWitnessError` | `#[non_exhaustive]`; callers must not exhaustively match witness dependency, aggregate, or row failures. |

No exhaustive public enum exceptions are owned by this module.

### Tests, traceability, and exit

Checker tests cover the complete API/debug oracle, exact resolver and
lower-profile validation, Task-252 reference IDs 0/1 independently from the
two stored use ordinals 1/1, and every row-local error and aggregate
precedence. Mutating either stored ordinal away from 1 is a dependency
mutation that yields `DependencyMismatch`, publishes nothing, preserves
prior debug bytes, and permits valid replay. Tests also cover owned-binding
substitution, the production Task-248-first rejection, the named reverse
test-only validation seam, injected final coexistence,
installation/revalidation/clone, rollback/replay, and
proof that all semantic tables remain empty. Runner tests parse and resolve
the exact 81-byte source
through the real frontend, exercise the private dormant `MT10-FS` route,
assert exact ranges/provenance/profiles, the measured left/right Task-252 use
ordinal sequence 1/1 independently from the upstream binding/use
source-event lookup sequence 1/2, and final ownership, reject loaded-source,
final-LF, named, recovered, subtree, and lower-dependency near misses, and
preserve active type-elaboration route isolation.

The implementation target is three checker tests and four mizar-test library
tests. It does not add the future `.miz` or sidecar, does not reclassify or
modify `pass_type_elaboration_reserved_variable_equality_001`, and does not
add or cover a trace row. The existing deferred
`spec.en.checker.formula_statement.source_payloads` row remains deferred with
an empty test list until Task 258, Tasks 269–272, and `MT10-FS` are complete.

Task 258A is complete only when the syntax-free transaction, real parser/
resolver test route, typed/resolved ownership, exact empty-semantic boundary,
reviews, and verification pass. It does not complete the Task 258 umbrella:
explicit assumption/conclusion/witness statements, local label/citation
inputs, composite theorem roots, nested statement contexts, and broader
visibility remain for a separately frozen Task 258B.

## Task 258A Implementation Result

The frozen transaction is implemented without extending its language or test
intent. `SourceStatementProducer` publishes only the five exact dense rows,
owns the validated binding environment, and authenticates the theorem across
the current module namespace, symbol, definition, label, contribution, and
checked-owner views. Installation rechecks the Task-252/256 debug
fingerprints, both stored reference-use ordinals, arena topology, direct
formula-expression wrapper, and excluded descendants.

Typed installation rejects every existing semantic table. Final assembly
also rejects cluster, overload, expression, statement-semantic, proof, and
diagnostic coexistence before building output. It accepts either no node
hints or the complete dense set of source-preserved hints whose sole role is
`source.statement.transport`; those hints preserve syntax nodes and do not
create semantic facts. Task-248-first, named reverse test-only, and injected
final coexistence paths all fail atomically and retain valid replay.

The checker has exactly three compound Task-258A tests and the dormant real
runner route has exactly four. No fixture, sidecar, expectation, trace row,
trace status, or active count changed. Broader statement shapes and every
semantic acceptance/proof decision remain Task 258B or Tasks 269–272.

## Task 258B Decomposition

Fresh post-Task-258A inventory shows that the original Task 258B umbrella is
not one safe logical task. Explicit assumptions and witnesses require new
statement payloads and later proof-local binding ownership; composite theorem
roots require Task-257 cross-family composition; imported and nested
visibility add distinct resolver profiles. Combining all of them would mix
source transport with Tasks 269–272 proof semantics.

Task 258B is therefore decomposed as follows:

1. **Task 258B1**, frozen below, transports one exact theorem with nested
   proof contexts, a labeled local proposition, two explicit conclusion
   shells, and one resolver-authenticated local citation. It publishes no
   accepted fact and performs no proof or justification semantics.
2. **Task 258B2+** retains explicit assumptions and witnesses, composite
   theorem roots, and broader imported/outer/inner visibility profiles. Each
   profile requires a later separate frozen contract.
3. Tasks 269–272 continue to own proof-local declarations and bindings,
   closure/capture/substitution, `reconsider` intent, proof skeleton
   decomposition, justification selection, and proof results.

This decomposition is `design_drift` closure, not a language change.

## Task 258B1 Frozen Nested-Conclusion/Local-Citation Slice

### Authority, exact source, and lower-stage evidence

Canonical authority is Chapter 4 §§4.3, 4.6, and 4.7.1 for the reserved
variable and its type guard; Chapter 14 §14.5.2 for equality; Chapter 15
§§15.4.1, 15.8.1–15.8.2, 15.10, and 15.12 for direct conclusions, full
statement proofs, labels, citations, and scope; and Chapter 16 §§16.1–16.2,
16.4.1–16.4.2, 16.5.1, 16.7.1–16.7.3, 16.8, and 16.9 for the theorem owner,
proof blocks, proof-step visibility, and later proof semantics. The existing
`pass_parser_theorems_proofs_001.miz`,
`fail_type_elaboration_statement_proof_gap_001.miz`, parser tests, and
resolver `labels` tests are unchanged lower-stage oracles.

The private dormant consumer accepts exactly this 139-byte final-LF source:

```mizar
reserve x for set;
theorem FormulaStatementNestedContextSmoke: x = x proof
  A: x = x proof
    thus x = x;
  end;
  thus x = x by A;
end;
```

Its SHA-256 is
`e5b87121e97e4ec4160b0189eff598d05f3ed5193698238226461f00593a907b`.
Fresh real-frontend inventory measured a normal root `0..138` and these
half-open ranges:

| Occurrence | Range |
|---|---:|
| reserve item / type | `0..18` / `14..17` |
| theorem owner / label / theorem equality | `19..138` / `27..61` / `63..68` |
| outer proof block | `69..137` |
| labeled compact statement / label / equality | `77..114` / `77..78` / `80..85` |
| nested proof block | `86..113` |
| nested conclusion / equality | `96..107` / `101..106` |
| outer conclusion / equality | `117..133` / `122..127` |
| simple justification / citation `A` | `128..132` / `131..132` |
| outer `end;` | `134..138` |

The four equality operand pairs are `63..64`/`67..68`,
`80..81`/`84..85`, `101..102`/`105..106`, and
`122..123`/`126..127`. The parser shape is exactly one reserve and one
unmodified theorem, one direct theorem `FormulaExpression`, one outer
`ProofBlock`, one labeled `CompactStatement` with one direct formula and one
nested `ProofBlock`, one nested `ConclusionStatement`, and one later outer
`ConclusionStatement` with one `JustificationClause` and one simple
`Reference`. Every node is normal.

The ordinary declaration/symbol collection supplies one public/exported
local theorem symbol, definition, and `LocalSource` contribution whose source
anchor remains the reserve `0..18`; as in Task 258A, the private runner
constructs and authenticates the theorem label projection. The normal symbol
environment intentionally contains no proof-step label index. Task 258B1
uses the existing public resolver boundary instead: one exact parser-backed
`ResolvedAst`, one
`LabelProjection::proof_step` for `A`, one
`LabelReferenceCandidate::unqualified_citation`, and
`LabelResolver::resolve`. The label is `Private`/`LocalOnly`,
`LabelKind::ProofStep`, visible only after statement ordinal 1 in outer proof
scope `[0]`; the citation is statement ordinal 3 in the same scope and
resolves to that origin. The nested proof scope is `[0, 0]`. Missing,
ambiguous, forward, sibling, inner-to-outer substitution, theorem-kind,
imported, recovered, cross-source/module/contribution, wrong-range, or
wrong-spelling provenance is rejected.

### Frozen lower profiles and base statement transaction

The lower dependency graph is:

```text
Task 48 reserve x:set base 1 context / 1 binding / 0 diagnostics
  -> Task 258B1 proof-context extension 3 contexts / 1 binding / 0 diagnostics
  -> Task 252 primary/reference/numeric 8/8/0
  -> Task 256 atomic/wrapper/segment/head/candidate/type/attribute/edge/request
     4/0/0/0/0/0/0/8/8
  -> Task 258 base owner/statement/context/input/candidate 1/4/4/4/4
  -> Task 258B1 label/citation composition 1/1
```

`BindingContextOwner` gains one non-exhaustive
`SourceStatement { source_range: SourceRange }` variant. Context 0 is the
unchanged module context with reserved binding 0. Context 1 is
`SourceStatement { 69..137 }`, parent 0, layer `Proof`; context 2 is
`SourceStatement { 86..113 }`, parent 1, layer `Proof`. Both own no binding
and preserve visible bindings `[0]`. Context 1 has lexical scope `[0]`;
context 2 has lexical scope `[0,0]`; module context 0 retains `None`. All
three have normal recovery. The environment is exactly `3/1/0`; this source
adds no proof-local variable, capture, or diagnostic.
`BindingEnv::try_new` requires each `SourceStatement` range to be nonempty and
from the environment source, and renders it exactly as
`source-statement(<start>..<end>)`. Every pre-B1 binding-environment debug
byte remains unchanged.

Task 252 owns eight `VariableReference`/`Value` rows and eight references to
reserved binding 0 in source order. Their binding contexts are
`0,0,1,1,2,2,1,1`, and every producer-stored reference use ordinal is 1.
Task 256 owns four normal `Equality` rows in contexts `0,1,2,1`, with two
built-in operand edges and two unresolved operand-expected-type requests per
row. No Task-248 context handoff or Task-249/253–257 handoff is installed.

The existing five input vectors and public API remain source-compatible.
`SourceStatementProducer::build` recognizes the Task-258A
`1/1/1/1/1` profile and the Task-258B1 `1/4/4/4/4` profile from validated
rows and dependencies; there is no caller-supplied profile flag. Task 258B1
adds only these public enum variants:

```rust
pub enum BindingContextOwner {
    SourceStatement { source_range: SourceRange },
    // existing variants remain unchanged
}

pub enum SourceStatementKind {
    TheoremProposition,
    ProofStepProposition,
    Conclusion,
}
```

The existing base-producer and installation precedence is unchanged for
both profiles: source/module, binding environment, Task-252/256
fingerprints, shared arena, and required lower installation fail first as
`DependencyMismatch`; profile cardinality and dense aggregate order fail
second as `InvalidAggregate`; then the first invalid owner, statement,
context, input-fact, or candidate-fact row fails in that order. Mixed
dependency/cardinality corruption is dependency-first, and mixed
aggregate/row corruption is aggregate-first.

The four statement rows are source preorder. `Spelling` is the exact
single-space token rendering stored, validated, and printed by the base
handoff:

| Row | Kind | Context row / binding context | Range | Formula | Spelling |
|---:|---|---:|---:|---:|---|
| 0 | `TheoremProposition` | `0 / 0` | `19..138` | atomic 0 | `theorem FormulaStatementNestedContextSmoke : x = x proof A : x = x proof thus x = x ; end ; thus x = x by A ; end ;` |
| 1 | `ProofStepProposition` | `1 / 1` | `77..114` | atomic 1 | `A : x = x proof thus x = x ; end ;` |
| 2 | `Conclusion` | `2 / 2` | `96..107` | atomic 2 | `thus x = x ;` |
| 3 | `Conclusion` | `3 / 1` | `117..133` | atomic 3 | `thus x = x by A ;` |

All four use owner 0, normal recovery, exact source ordinal `0..3`, and
direct parser formula paths. Context rows use the same statement ranges and
visible binding `[0]`. Input fact row `i` is a
`ReservedTypeGuard` for statement/context `i`, binding 0, ordinal 0, and
Task-252 reference pair `[2i, 2i+1]`. Candidate row `i` is an
`UnverifiedProposition` for statement/context `i`, ordinal 0, and atomic
formula `i`. None of the candidates is visible or accepted merely because a
source label or citation exists.

Owner 0 is exactly the theorem site/range `19..138`, label range `27..61`,
spelling `FormulaStatementNestedContextSmoke`, role `Theorem`, status
`Unmodified`, and normal recovery. Its symbol and contribution are the sole
authenticated current-module public/exported theorem declaration from the
ordinary Task-258A owner path; the B1 branch may not invent or substitute a
second owner, contribution, theorem label, or source anchor.

### Frozen local-label/citation composition API

Task 258B1 adds two dense tables in the existing public
`source_statement` module without changing Task-258A debug bytes:

```rust
pub struct SourceStatementLabelId(usize);
pub struct SourceStatementCitationId(usize);

pub struct SourceStatementReferenceHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub labels: Vec<SourceStatementLabelInput>,
    pub citations: Vec<SourceStatementCitationInput>,
}

pub struct SourceStatementLabelInput {
    pub statement: SourceStatementId,
    pub context: SourceStatementContextId,
    pub candidate: SourceStatementCandidateFactId,
    pub origin_path: LabelOriginPath,
    pub proof_scope: LabelScopePath,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub visible_after_ordinal: usize,
    pub spelling: String,
    pub kind: SourceStatementLabelKind,
    pub recovery: SourceStatementRecovery,
}

pub struct SourceStatementCitationInput {
    pub statement: SourceStatementId,
    pub context: SourceStatementContextId,
    pub target: SourceStatementCitationTarget,
    pub label_ref: LabelRefId,
    pub proof_scope: LabelScopePath,
    pub source_range: SourceRange,
    pub ordinal: usize,
    pub kind: SourceStatementCitationKind,
    pub recovery: SourceStatementRecovery,
}

pub struct SourceStatementLabel { /* immutable validated label fields */ }
impl SourceStatementLabel {
    pub const fn statement(&self) -> SourceStatementId;
    pub const fn context(&self) -> SourceStatementContextId;
    pub const fn candidate(&self) -> SourceStatementCandidateFactId;
    pub const fn origin_path(&self) -> &LabelOriginPath;
    pub const fn proof_scope(&self) -> &LabelScopePath;
    pub const fn source_range(&self) -> SourceRange;
    pub const fn source_ordinal(&self) -> usize;
    pub const fn visible_after_ordinal(&self) -> usize;
    pub fn spelling(&self) -> &str;
    pub const fn kind(&self) -> SourceStatementLabelKind;
    pub const fn recovery(&self) -> SourceStatementRecovery;
}

pub struct SourceStatementCitation { /* immutable validated citation fields */ }
impl SourceStatementCitation {
    pub const fn statement(&self) -> SourceStatementId;
    pub const fn context(&self) -> SourceStatementContextId;
    pub const fn target(&self) -> SourceStatementCitationTarget;
    pub const fn label_ref(&self) -> LabelRefId;
    pub const fn proof_scope(&self) -> &LabelScopePath;
    pub const fn source_range(&self) -> SourceRange;
    pub const fn ordinal(&self) -> usize;
    pub const fn kind(&self) -> SourceStatementCitationKind;
    pub const fn recovery(&self) -> SourceStatementRecovery;
}

pub struct SourceStatementLabelTable { /* dense source-order rows */ }
impl SourceStatementLabelTable {
    pub fn get(&self, id: SourceStatementLabelId) -> Option<&SourceStatementLabel>;
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceStatementLabelId, &SourceStatementLabel)>;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}

pub struct SourceStatementCitationTable { /* dense source-order rows */ }
impl SourceStatementCitationTable {
    pub fn get(
        &self,
        id: SourceStatementCitationId,
    ) -> Option<&SourceStatementCitation>;
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceStatementCitationId, &SourceStatementCitation)>;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}

#[non_exhaustive]
pub enum SourceStatementLabelKind {
    ProofStep,
}

#[non_exhaustive]
pub enum SourceStatementCitationTarget {
    Local(SourceStatementLabelId),
    Imported,
}

#[non_exhaustive]
pub enum SourceStatementCitationKind {
    SimpleLocal,
    SimpleImported,
}
```

Both IDs have the existing dense-ID derives and `new`/`index` accessors.
Inputs, immutable rows, tables, and handoff derive `Debug, Clone, PartialEq,
Eq`; the three enums use the existing public data-enum derives and are
`#[non_exhaustive]`. `SourceStatementLabel` and
`SourceStatementCitation` expose read-only accessors for every corresponding
input field. `SourceStatementLabelTable` and
`SourceStatementCitationTable` expose only typed `get`, source-ordered
`iter`, `len`, and `is_empty`.

`SourceStatementReferenceProducer::build` has the exact syntax-free
signature:

```rust
pub fn build(
    input: SourceStatementReferenceHandoffInput,
    statements: &SourceStatementHandoff,
    resolver_ast: &ResolvedAst,
    projection: &LabelProjection,
    reference: &LabelReferenceCandidate,
    resolution: &LabelResolutionResult,
    arena: &TypedArena,
) -> Result<SourceStatementReferenceHandoff, SourceStatementReferenceError>;
```

The immutable handoff owns exact clones of the validated `ResolvedAst`,
`LabelProjection`, `LabelReferenceCandidate`, and `LabelResolutionResult`,
stores the Task-258 base debug fingerprint, and has no mutable or unchecked
constructor. Its exact table/arena accessors are:

```rust
pub const fn source_id(&self) -> SourceId;
pub const fn module_id(&self) -> &ModuleId;
pub fn statement_fingerprint(&self) -> &str;
pub const fn resolver_ast(&self) -> &ResolvedAst;
pub const fn label_projection(&self) -> &LabelProjection;
pub const fn reference_candidate(&self) -> &LabelReferenceCandidate;
pub const fn label_resolution(&self) -> &LabelResolutionResult;
pub const fn labels(&self) -> &SourceStatementLabelTable;
pub const fn citations(&self) -> &SourceStatementCitationTable;
pub fn debug_text(&self) -> String;
```

Its error is:

```rust
#[non_exhaustive]
pub enum SourceStatementReferenceError {
    DependencyMismatch,
    InvalidLabel { label: SourceStatementLabelId },
    InvalidCitation { citation: SourceStatementCitationId },
    InvalidAggregate,
}
```

Reference production, typed installation, and final assembly use one total
precedence. Source/module, statement fingerprint, resolver-AST identity,
resolver replay result, and shared typed arena fail first as
`DependencyMismatch` in the producer. Typed/final installation then applies
the same dependency-first class to the statement handoff's binding/lower
fingerprints and the actually installed Task-252/256 values. Exact `1/1`
cardinality, dense IDs, and source-order aggregate structure fail second as
`InvalidAggregate`. The first invalid projection/label pair then fails as
`InvalidLabel`, followed by the first invalid reference/citation pair as
`InvalidCitation`. Mixed dependency/cardinality corruption is
dependency-first; mixed aggregate/row corruption is aggregate-first. Later
entry points revalidate all four stored resolver objects and do not construct
replacement provenance.

The only admitted profile is `1/1`. Label 0 belongs to statement/context 1,
candidate 1, range `77..78`, ordinal 0, visible-after statement ordinal 1,
scope `[0]`, spelling `A`, kind `ProofStep`, and normal recovery. Its exact
origin path is `<package>::<module>::proof::A`. Citation 0 belongs to
statement/context 3, label 0, resolver reference 0, range `131..132`,
ordinal 0, scope `[0]`, kind `SimpleLocal`, and normal recovery.

The projection must be a current-module proof-step projection with the same
origin path, spelling/range, owner-0 source/module and contribution,
`Private`/`LocalOnly`, visible-after ordinal 1, and proof scope `[0]`.
The trusted namespace is derived only as
`NamespacePath::new(statements.module_id().path().as_str())`; the projection
module and namespace must equal the authenticated statement module and that
derived namespace. More exactly, the proof-step semantic origin is normal,
non-imported, anchored at `77..78`, and has structural path `[12]`; it is
exactly the origin of real label-token resolver node 12 and
shares owner 0's source/module and contribution, not owner 0's anchor/path.
The reference candidate must be an unqualified
`ProofOrTheorem` citation whose site spelling/range, current-source semantic
origin, source ordinal 3, and scope `[0]` match citation 0. Its semantic
origin is normal, non-imported, anchored at `131..132`, with structural path
`[68]`; it is exactly the origin of its `ReferenceSite` node, resolver node
68, the real `SurfaceNodeKind::Reference` at `131..132`.

The runner uses a deterministic two-pass `ResolvedArenaBuilder` adapter,
not an exposed or invented `ResolvedNodeId`. Each pass inserts all 77 real
surface nodes exactly once in parser arena order, requires every returned
resolver id to equal the corresponding surface index, preserves every node
kind, range, child list, and recovery state, uses current source/module range
origins with structural path `[index]`, and finishes with real root 76. No
node is omitted, generated, reordered, or used only to mint an id. The
preliminary pass keeps all nodes `NotApplicable` with no reference key and
supplies the genuine node-68 id needed to build the candidate. After
`LabelResolver::resolve`, the final pass changes only node 68 to
`NodeResolutionState::Resolved` with
`NodeReferenceKey::Label(resolution.ids()[0])`; every other node remains
`NotApplicable` with no reference key.

The checker requires 77 resolver and typed nodes, root 76, all-index
anchor/child/recovery parity, current source/module origins with no import
edge, node 68 as the sole resolved/keyed node, resolver node 12 to match the
proof-step projection origin, and resolver node 68 to match both the
candidate origin and typed citation node at the same range. Exact
`SurfaceNodeKind` parity, including node 68 as `Reference`, is enforced only
by the runner's real-parser selector and tests. The checker production
boundary does not name, match, stringify, or otherwise interpret
`SurfaceNodeKind`, so it adds no normal/runtime `mizar-syntax` dependency;
the frozen checker test matrix may add only the test-only dev-dependency
specified below. The runner then calls
`ResolvedAst::try_new` with the final arena, empty name-reference and
import/export tables, and an exact clone of the resolver-produced
label-reference table. The resulting `ResolvedAst` must have the current
source/module, root 76, 77 nodes, zero name references, exactly one label
reference, and zero imports/exports; its label table must equal
`resolution.table()`, and the node-68 label key must point to that same
table entry.
This is the bounded B1 parser-to-public-resolver adapter, not full driver
label lowering or a synthetic arena.

The producer
replays exactly
`LabelResolver::new(&[projection.clone()]).resolve(statements.module_id(),
&derived_namespace, &[reference.clone()])` and requires structural equality
with the supplied resolution. The runner must not populate or mutate a
`LabelRefTable`, result, or replacement reference outcome; it may only clone
the resolver-produced table into `ResolvedAst::try_new`. The producer never
derives the trusted namespace from the projection. The result and index
contain only proof-step `A`—not the theorem owner projection—plus one resolved
reference, ids `[0]`, no diagnostic, and no unresolved or ambiguous entry.
Thus mutually altered scope or ordinal inputs cannot be hidden by a lossy
result table.

The stable new debug schema is:

```text
source-statement-reference-debug-v1
module: <package>::<module>
statement-fingerprint: <quoted source-statement debug>
resolver-ast root=76 nodes=77 name_refs=0 label_refs=1 imports=0 exports=0 label_node=12 reference_node=68 reference_state=resolved reference_key=label#0
resolver-projection origin=<package>::<module>::proof::A namespace=<module> range=77..78 visible_after=1 scope=[0] kind=proof-step visibility=private export=local-only spelling="A"
resolver-reference node=68 range=131..132 source_ordinal=3 scope=[0] expectation=proof-or-theorem spelling="A"
resolver-result index=1 references=1 ids=[0] diagnostics=0
label#0 statement=1 context=1 candidate=1 origin=<package>::<module>::proof::A scope=[0] range=77..78 source_ordinal=0 visible_after=1 kind=proof-step recovery=normal spelling="A"
citation#0 statement=3 context=3 label=0 label_ref=0 scope=[0] range=131..132 ordinal=0 kind=simple-local recovery=normal
```

### Installation, exclusions, semantics, tests, and audit

`TypedAst` adds the exact field/accessor and combined one-shot installer:

```rust
source_statement_references: Option<SourceStatementReferenceHandoff>,

pub const fn source_statement_references(
    &self,
) -> Option<&SourceStatementReferenceHandoff>;

pub fn with_source_statement_references(
    self,
    statements: SourceStatementHandoff,
    references: SourceStatementReferenceHandoff,
) -> Result<Self, TypedAstError>;
```

The installer requires a fresh statement slot, exact Task-252/256
dependencies, the B1 base/reference pair, the `3/1/0` binding environment,
and one shared arena. Every failure is
`TypedAstError::InvalidSourceStatement`; it publishes both handoffs only
after complete validation. The existing
`with_source_statement(self, statement) -> Result<Self, TypedAstError>`
remains the Task-258A-only installer, and its debug bytes and validation are
unchanged.

`ResolvedTypedAst` adds the same optional field and exact
`source_statement_references(&self) ->
Option<&SourceStatementReferenceHandoff>` accessor. `assemble` revalidates
and clones the B1 pair together, with every failure reported as
`ResolvedTypedAstError::InvalidSourceStatement`. In both typed and resolved
debug text, the base `source_statement.debug_text()` is followed immediately
by `source_statement_references.debug_text()`, before node/table output.
Task-258A has no reference chunk and remains byte-identical.

Task-248 source context, every Task-257 family, either preinstalled statement
profile, a B1 base without references, references without the matching base,
mixed A/B1 rows, and both install orders with any other source owner fail
without partial mutation.

The typed arena preserves the exact theorem, proof-block, compact-statement,
conclusion, proposition/formula-wrapper, equality, term, justification, and
reference topology. Owned statement sites are theorem `19..138`, compact
statement `77..114`, and conclusions `96..107`/`117..133`; formula targets
must be their direct structural descendants. The compact statement may
contain only the frozen nested proof/conclusion subtree. The exact admitted
statement containment tree is row 0 containing rows 1 and 3, with row 1
containing row 2; all four statement sites and formula targets remain
distinct. Any other ancestor/descendant or sibling crossing, duplicate site
reuse, moving the citation into the nested proof, adding an
assumption/witness/second label/reference, or substituting a
proof/justification node fails closed.

The exact selector rejects a missing final LF, byte/name/status/role/reserve
change, omitted or extra item, direct/parenthesized/composite/non-equality
formula, missing/extra/reordered proof block or statement, `hence`, `then`,
assumption, witness, `given`, `consider`, `now`, `hereby`, case/suppose,
iterative equality, theorem citation, imported label, forward citation,
local-label shadowing, recovery, comments changing bytes, and every active
corpus or Task-258A near miss.

Task 258B1 leaves `TypedAst::facts`, checked formulas, statement semantics,
checked proofs/nodes/goals, cluster/overload/expression outputs,
diagnostics, CoreIr, ControlFlowIr, VC, cache, artifacts, proof acceptance,
and theorem publication empty. A `ProofStepProposition` is still an
unverified candidate; a `SimpleLocal` citation is resolver-resolved but
semantically unaccepted proof intent, not an input fact or accepted premise.
Tasks 269–272 retain all semantic interpretation.

Four checker tests freeze the B1 base/reference API and debug, complete
dependency/aggregate/row/subtree error precedence, resolver scope/ordinal/
origin corruption—including independently stale results as
`DependencyMismatch` and coherently replayed projection/reference mutations
as `InvalidLabel`/`InvalidCitation`—full resolver/typed-arena parity and
node/origin corruption, exact binding-owner range/debug validation, Task-258A
byte compatibility, typed/final ownership, semantic coexistence rejection,
rollback, clone, and replay. Five
mizar-test library tests freeze the real frontend ranges, theorem and local
label resolver provenance, two-pass/final-keyed 77-node resolver AST, and
replay bundle, binding contexts, eight references/four atomics, the two
handoffs, selector isolation, mutations, and empty final output.

The checker test module needs to construct the frozen resolver AST, while
production checker code must not name `SurfaceNodeKind`. The implementation
therefore adds `mizar-syntax` only under
`crates/mizar-checker/Cargo.toml` `[dev-dependencies]`; the corresponding
`Cargo.lock` dependency edge is mechanical. No production dependency or
runtime syntax access is added. This bounded test-construction gap is part of
the B1 `source_drift`/`test_gap`, not an independent resolver change.

This documentation prerequisite changes no production source, fixture,
sidecar, expectation, trace TOML/status/count, executable route, test list,
or hash. Baselines remain plan `419/387`, type `253/241`, pass/fail
`228/191`, active parse/declaration/type/proof `101/5/198/1`,
warnings/errors `23/0`, checker/runner libraries `338/369`, and runner
production 30 paths / 34,955 lines with path/content hashes
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`dd399648aecadf2e7a63f685ad87577b7ebae9a9064fbfaba429a07d25ed9912`.
All test-list and five CLI hashes remain the Task-258A completion values.

The missing contract was `design_drift`; the absent B1 profile, reference
handoff, installers, dormant route, and matrices are bounded
`source_drift`/`test_gap`. There is no blocking `spec_gap`,
`test_expectation_drift`, `source_undocumented_behavior`, or
`boundary_violation`, and no unresolved `repo_metadata_conflict`.
`spec.en.checker.formula_statement.source_payloads`
stays deferred with `tests = []`; the coverage audit changes only follow-up
ownership. Documentation exit requires synchronized EN/JA records,
independent no-findings review, unchanged measured artifacts, all hard gates,
quality at least 90/100, task-only staging, and one dedicated documentation
commit. Only after that commit and fresh preflight may Task-258B1
implementation begin.

### Task 258B1 Implementation Status

The frozen base and reference transactions are implemented exactly. The base
producer selects the Task-258A or Task-258B1 profile from authenticated
dependencies rather than a caller flag. The reference producer publishes one
proof-step label and one simple local citation only after replaying the exact
resolver projection/reference/result and comparing every resolver node to
the same-index typed node. Dependency, aggregate, label, and citation error
precedence is fail-closed and replay-safe.

The combined `TypedAst`/`ResolvedTypedAst` owner publishes the base/reference
pair atomically. Four checker tests cover the complete API/debug surface,
dependency/row/provenance corruptions, owner exclusion/rollback, final
revalidation/clone, and the empty semantic boundary. Task-258A installer and
debug bytes remain unchanged. Broader statement shapes and every proof
semantic remain outside this implementation.

## Task 258B2 Frozen Single-Assumption Slice

Task 258B2 is the next dependency-ready transport slice after Task 258B1.
Its authority is `doc/spec/en/15.statements.md` §§15.3.1, 15.4.1, 15.8.2,
and 15.10, the equality formula/term rules in Chapters 13–14, the reserve
visibility rules in Chapter 4, existing
`pass_parser_simple_statements_001.miz`, the Task-88/89 parser/resolver
fixtures, and the public Task-48/252/256/258A/258B1 APIs. Those authorities
support an unlabeled single assumption as source intent; they do not
authorize accepting it as a fact or interpreting its proof effect.

The exact future corpus-dormant consumer is this 113-byte final-LF source,
SHA-256
`c9d77d864ab899865bac77c29c57ff5785d553f8b119ef2274e4e9caf031a125`:

```mizar
reserve x for set;
theorem FormulaStatementSingleAssumptionSmoke: x = x proof
  assume x = x;
  thus x = x;
end;
```

Fresh parser/resolver inventory freezes the following identity:

| Object | Exact identity |
| --- | --- |
| surface arena | 55 nodes, root 54, all unrecovered |
| reserve/theorem | reserve `0..18`; theorem item node 51, `19..112`; label `27..64` |
| theorem owner | one local public/exported theorem, contribution 0, origin path `[2,1]` |
| proof | node 50, `72..111` |
| statement rows | theorem node 51 `19..112`; assumption node 41 `80..93`; conclusion node 49 `96..107` |
| atomic targets | nodes 32/38/46 at `66..71`, `87..92`, `101..106` |
| primary terms | nodes 28/30/34/36/42/44 at `66..67`, `70..71`, `87..88`, `91..92`, `101..102`, `105..106` |
| resolver labels | no proof-step label, citation, or label-reference key |

The syntax-free lower composition is exactly Task-48 `2/1/0`: module context
0 and one `BindingContextOwner::SourceStatement` proof context 1 over
`72..111`, both exposing reserved binding 0. Task-252 is `6/6/0` with context
sequence `0,0,1,1,1,1` and stored use ordinal 1. Task-256 is
`3/0/0/0/0/0/0/6/6` with formula contexts `0,1,1`. The source-statement base
is `1/3/3/3/3`; no `SourceStatementReferenceHandoff` is installed.

`SourceStatementKind` adds only `Assumption`. The exact statement kinds are
`TheoremProposition`, `Assumption`, and `Conclusion`; their normalized
spellings are respectively
`theorem FormulaStatementSingleAssumptionSmoke : x = x proof assume x = x ;
thus x = x ; end ;`, `assume x = x ;`, and `thus x = x ;`.
Every row has one direct atomic formula target, one context, one
`ReservedTypeGuard` input over its two Task-252 references, and one
`UnverifiedProposition` candidate. The assumption kind records source intent
only. `SourceStatementProducer::build` keeps its signature and selects this
profile from exact authenticated rows and dependencies, never from a caller
flag.

The existing base-only `TypedAst::with_source_statement` installer is widened
from Task-258A-only to the exact Task-258A or Task-258B2 base profiles.
Task-258B1 still requires the paired reference installer. Installation and
final `ResolvedTypedAst` assembly revalidate the same shared arena and clone
the base atomically. Task-248, every Task-257 family, Task-258A/B1/B2
cross-profile hybrids, preinstalled statement/reference payloads, all
semantic tables, and both ownership orders with any foreign source owner
remain rejected without partial mutation. Task-258A and Task-258B1 debug
bytes remain unchanged.

The statement containment graph is theorem row 0 containing sibling rows 1
and 2; neither proof statement contains the other. Formula targets must be
the direct proposition/formula descendants of their owning statement and
must stay distinct. Proof-block, proposition, formula-wrapper, punctuation,
and unrelated surface nodes remain unowned validation context. Duplicate
sites, row crossing, ancestor/descendant substitution, a formula from the
other statement, recovered/degraded nodes, extra labels/citations,
assumption labels, collective `assume that`, `given`, `consider`, `take`,
`then`, `hence`, `now`, `hereby`, case/suppose, iterative equality,
composite/non-equality formulas, extra/reordered statements, or any source
byte change fail closed.

Task 258B2 publishes no accepted premise, fact, checked formula, statement
semantic, proof node/goal, diagnostic, theorem status, IR, VC, cache, or
artifact. In particular, `Assumption` plus
`UnverifiedProposition` is not authorization to add the formula to a proof
context. Task 272 retains assumption/proof-skeleton/justification meaning;
Tasks 269–271 retain local declarations, closure/capture/substitution, and
reconsider intent. Task 258B3 retains witness transport, Task 258B4
composite theorem roots, and Task 258B5 broader imported/outer/inner
visibility.

The future implementation is limited to the existing checker
`source_statement.rs`, typed/final profile validation, lint-policy tests, the
existing private runner statement leaf/facades, four checker tests, and five
runner tests. It must reuse the existing
`BindingContextOwner::SourceStatement` contract; a `binding_env.rs` source
change is forbidden and any newly discovered lower-stage defect requires a
separate prerequisite. The tests must cover
every exact row/accessor/debug field, Task-48/252/256 fingerprints, parser and
resolver provenance, all-index typed/surface parity, profile and semantic
ownership in both orders, complete mutation/replay, exact selector near
misses, active-corpus isolation, clone, rollback, and empty semantics. No
production syntax dependency, fixture, sidecar, expectation, trace edit,
active route, or corpus credit is permitted.

The missing Task-258B2 contract is `design_drift`. The absent exact profile
and dormant route are bounded `source_drift`, and the absent four/five test
matrices are `test_gap`. There is no blocking `spec_gap`,
`source_undocumented_behavior`, `test_expectation_drift`,
`boundary_violation`, or unresolved `repo_metadata_conflict`.
`spec.en.checker.formula_statement.source_payloads` remains deferred with
`tests = []`; the coverage audit changes follow-up ownership only.

This documentation prerequisite changes no source, fixture, sidecar,
expectation, trace row/status/count, route, test list, or hash. Baselines
remain plan/type `419/387` and `253/241`, pass/fail `228/191`, active
parse/declaration/type/proof `101/5/198/1`, warnings/errors `23/0`,
checker/runner libraries `342/374`, and runner production 30 paths /
35,854 lines with path/content hashes
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`f2d133e6fc42bd62058e95c274944aa03d80e9f8f2b5a0394a4d11e58ec3a66e`.
All four test-list hashes and five CLI hashes remain the Task-258B1 values.
Exit requires synchronized EN/JA documentation, independent no-findings
reviews, every hard gate, read-only quality at least 90/100, task-only
staging, and one dedicated documentation commit. Implementation may begin
only after that commit and a fresh parser/resolver/lower-API/count/hash
preflight.

## Task 258B3M1 Implementation Result

The frozen mixed row is implemented without widening semantics or API.
The private profile authenticates all 56 raw parser tuples and typed nodes,
the exact resolver owner with no resolver-owned `y`, six primary terms,
two atomic formulas, two base statements, two dense witnesses, and one
name. Validation preserves dependency/fingerprint, aggregate, witness 0,
witness 1, then name precedence; B3/B3N v1 bytes remain unchanged.

Exactly four checker and five runner compound tests pass, including every
node mutation, exhaustive base/witness/name/resolver replay, all ownership
orders, all final coexistence stages, near misses, and active isolation.
Libraries are `358/394`; module sizes are `14045/4659/7201/3156`, runner
sizes `3724/688/2501/7246`, and runner production is 30 paths / 38,103
lines. Binding and semantic ownership remain deferred to Tasks 269/272;
B3M2 remains next before B4.

## Task 258B2 Implementation Closure

The frozen 113-byte profile is implemented without widening its language
meaning. `SourceStatementKind::Assumption`, the exact syntax-free
`1/3/3/3/3` producer profile, base-only typed/final installation, and the
dormant runner leaf now transport the theorem, assumption, and conclusion
over Task-48 `2/1/0`, Task-252 `6/6/0`, and Task-256
`3/0/0/0/0/0/0/6/6`. Resolver authentication requires the one exact local
public/exported theorem label, contribution 0, origin path `[2,1]`, and no
import, citation, or reference handoff.

Four checker and five runner tests close the bounded `source_drift` and
`test_gap`, including all-index arena parity, lower/resolver mutation replay,
subtree exclusion, Task-248/257/258 cross-family ownership, clone/debug, and
empty semantics. `Assumption` remains paired only with
`UnverifiedProposition`; it creates no premise, fact, checked formula,
statement semantic, proof, goal, diagnostic, or accepted theorem. Task
258B3 retains witnesses, Task 258B4 composite roots, Task 258B5 broader
visibility, and Tasks 269–272 proof semantics.

## Task 258B3 Frozen Single-Witness Slice

Task 258B3 is the next dependency-ready transport slice after Task 258B2.
Its canonical authority is `doc/spec/en/15.statements.md` §§15.4.4 and
15.11.5, Chapters 4, 13, and 14 for the reserved variable, term, and equality
shells, existing `pass_parser_simple_statements_001.miz` named/unnamed
`take` syntax, the parser/resolver fixtures, and the public
Task-48/252/256/258A/258B1/258B2 APIs. The grammar authorizes the unnamed
`take x;` source shape and left-to-right witness order. Section 15.11.5
assigns existential-goal matching, type obligations, substitution, and
named-abbreviation effects to later semantics; none is executed here.

The exact future corpus-dormant consumer is this 104-byte final-LF source,
SHA-256
`76fb48354fc0dfb17047900a047a5b28b806df60d139a3133e606f0ef12a3f82`:

```mizar
reserve x for set;
theorem FormulaStatementSingleWitnessSmoke: x = x proof
  take x;
  thus x = x;
end;
```

The equality theorem root intentionally isolates witness transport from the
Task-258B4 composite-root slice. Consequently this dormant source is not
claimed to be a semantically valid proof: `take` would require an
existential goal. It cannot become an active corpus case or an accepted
theorem in Task 258B3.

Fresh parser/resolver inventory freezes the following identity:

| Object | Exact identity |
| --- | --- |
| surface arena | 49 nodes, root 48, all unrecovered |
| reserve/theorem | reserve node 25 `0..18`; theorem node 45 `19..103`; label `27..61` |
| theorem owner | one local public/exported theorem, contribution 0, range `19..103`, origin path `[2,1]`; no import |
| proof | node 44, `69..102`, lexical scope `[0]` |
| formula statements | theorem node 45 with transparent `FormulaExpression` wrapper 31 and Task-256 atomic site 30 at `63..68`; conclusion node 43 with wrapper 41 and atomic site 40 at `92..97` |
| witness | `TakeStatement` node 35 `77..84`; `Witness` node 34 `82..83`; transparent `TermExpression` wrapper 33 and Task-252 term/reference site 32 at `82..83` |
| formula terms | transparent wrappers 27/29 and Task-252 sites 26/28 at `63..64`/`67..68`; wrappers 37/39 and sites 36/38 at `92..93`/`96..97` |
| resolver labels | no proof-step label, citation, label-reference key, or resolver companion |

The syntax-free lower composition is exact. Task 48 is `2/1/0`: module
context 0 and proof context 1 owned by
`BindingContextOwner::SourceStatement { source_range: 69..102 }`, with
parent 0, proof layer, scope `[0]`, no local binding, visible reserved
binding 0, and normal recovery. Binding 0 remains reserved `x` at `8..9`,
typed by `set` at `14..17`.

Task 252 is `5/5/0`. Dense term/reference IDs 0–4 have actual owned sites
26/28/32/36/38 under transparent wrappers 27/29/33/37/39 and cover ranges
`63..64`, `67..68`, `82..83`, `92..93`, and `96..97`; their binding contexts
are `0,0,1,1,1`, scopes are `[],[],[0],[0],[0]`, source ordinals are 0–4,
and every stored use ordinal is 1. Every term is the normal spelling `x`,
kind `VariableReference`, role `Value`, no parent, and every reference is
the normal `Variable` reference to binding 0. Task 256 is
`2/0/0/0/0/0/0/4/4`: equality formulas 0/1 at `63..68`/`92..97`, contexts
0/1, each with ordered left/right primary targets. Formula 0 targets terms
0/1 and formula 1 targets terms 3/4. Primary term 2 is excluded from every
atomic edge/request and is owned only by the witness transaction. Task-256
formula IDs 0/1 own `BuiltinPredicateApplication` sites 30/40 under
transparent `FormulaExpression` wrappers 31/41. All application, structure,
and set-term fingerprints remain absent.

The base `SourceStatementHandoff` remains formula-only and has exact
cardinality `1/2/2/2/2`:

| Table row | Exact contract |
| --- | --- |
| owner 0 | authenticated theorem symbol/contribution; node 45; `19..103`; spelling `FormulaStatementSingleWitnessSmoke`; `Theorem` / `Unmodified` / normal |
| statement 0 | owner/context 0; atomic formula 0; node 45; `19..103`; source ordinal 0; `TheoremProposition`; normalized complete-theorem spelling |
| statement 1 | owner 0/context 1; atomic formula 1; node 43; `87..98`; source ordinal 2; `Conclusion`; spelling `thus x = x ;` |
| context 0/1 | statement 0/1; binding context 0/1; ranges `19..103`/`87..98`; visible bindings `[0]` |
| input fact 0/1 | statement/context 0/1; ordinal 0; `ReservedTypeGuard`; binding 0; uses `[0,1]`/`[3,4]` |
| candidate 0/1 | statement/context 0/1; ordinal 0; `UnverifiedProposition`; atomic formula 0/1 |

The normalized theorem spelling is
`theorem FormulaStatementSingleWitnessSmoke : x = x proof take x ; thus x
= x ; end ;`. Dense base table IDs do not erase global proof-source order:
base source ordinals are exactly 0 and 2, while the companion witness source
ordinal is 1. The combined partition must be exactly `[0,1,2]`, with no
duplicate, gap, or reorder.

Witnesses cannot be added to `SourceStatementKind`: every base row requires
a formula, formula-statement context, input fact, and candidate fact, while
`take x;` contains a term and no proposition. Task 258B3 therefore adds one
separate syntax-free transaction:

- dense `SourceStatementWitnessId`;
- `SourceStatementWitnessHandoffInput` and
  `SourceStatementWitnessInput`;
- non-exhaustive `SourceStatementWitnessKind::Unnamed` and
  `SourceStatementWitnessTermTarget::Primary`;
- immutable `SourceStatementWitnessHandoff`, `SourceStatementWitness`,
  and `SourceStatementWitnessTable`;
- `SourceStatementWitnessProducer` and non-exhaustive
  `SourceStatementWitnessError`.

The exact public construction surface is:

```rust
pub struct SourceStatementWitnessHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub witnesses: Vec<SourceStatementWitnessInput>,
}

pub struct SourceStatementWitnessInput {
    pub owner: SourceTheoremOwnerId,
    pub binding_context: BindingContextId,
    pub term: SourceStatementWitnessTermTarget,
    pub take_site: TypedSiteRef,
    pub take_range: SourceRange,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub ordinal: usize,
    pub spelling: String,
    pub kind: SourceStatementWitnessKind,
    pub recovery: SourceStatementRecovery,
}

#[non_exhaustive]
pub enum SourceStatementWitnessTermTarget {
    Primary(SourcePrimaryTermId),
}

#[non_exhaustive]
pub enum SourceStatementWitnessKind {
    Unnamed,
}

pub struct SourceStatementWitnessHandoff { /* private validated fields */ }
pub struct SourceStatementWitness { /* private validated fields */ }
pub struct SourceStatementWitnessTable { /* private dense rows */ }
pub struct SourceStatementWitnessProducer;

#[non_exhaustive]
pub enum SourceStatementWitnessError {
    DependencyMismatch,
    InvalidWitness { witness: SourceStatementWitnessId },
    InvalidAggregate,
}

impl SourceStatementWitnessProducer {
    pub fn build(
        input: SourceStatementWitnessHandoffInput,
        statements: &SourceStatementHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
    ) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError>;
}
```

`SourceStatementWitnessId` is
`Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash`. Inputs,
rows, tables, and handoffs are `Debug + Clone + PartialEq + Eq`.
Kinds/targets are
`Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash`; the
producer is `Debug + Clone + Copy + Default`; the error is
`Debug + Clone + PartialEq + Eq` and implements `Display` and `Error`.
`SourceStatementWitnessHandoffInput` contains `source_id`,
`module_id`, and `witnesses`. The immutable handoff contains source/module
identity, derived exact `statement_fingerprint` and
`primary_term_fingerprint`, and exactly one witness row. Handoff accessors
expose `source_id`, `module_id`, both fingerprints, `witnesses`, and
deterministic `debug_text`; table accessors expose `get`, `iter`, `len`, and
`is_empty`.

Witness row 0 has owner 0, direct `BindingContextId(1)`, primary target 2,
take site/range node 35/`77..84`, witness site/range node 34/`82..83`,
source ordinal 1, within-`take` ordinal 0, spelling `x`, kind `Unnamed`, and
normal recovery. Its accessors expose every field without syntax types. The
typed arena assigns only `source.statement-witness.take` to node 35 and
`source.statement-witness.item` to node 34; transparent
`TermExpression` wrapper 33 stays `source.surface.unowned`, and Task 252
owns `TermReference` node 32. The companion validates ordered containment
35 → 34 → 33 → 32, the exact term/reference range, binding 0, context 1,
scope `[0]`, use ordinal 1, and absence from Task-256 edges. It neither
copies tokens nor invents a formula, binding, resolver node, projection,
reference, or result.

`SourceStatementWitness` exposes `owner`, `binding_context`, `term`,
`take_site`, `take_range`, `site`, `source_range`, `source_ordinal`,
`ordinal`, `spelling`, `kind`, and `recovery` accessors with the matching
borrowed/value return style used by existing statement rows.

`SourceStatementWitnessProducer::build(input, statements, primary_terms,
arena)` authenticates the exact Task-258B3 base profile and stores both
debug fingerprints. Failure precedence is source/module/base/lower/
fingerprint/shared-arena dependency first as `DependencyMismatch`, exact
one-row cardinality second as `InvalidAggregate`, then the first field,
ordinal, site, containment, target, context, binding, scope, or recovery
failure as `InvalidWitness { witness }`. Revalidation applies the same
precedence. Resolver provenance is sufficient without a new resolver
bundle: the base transaction retains the `SymbolEnv`-authenticated theorem
owner, while Task 252 retains the authenticated reserved-variable reference
used by the witness.

The exact debug grammar is:

```text
source-statement-witness-debug-v1
module: <package>::<module>
statement-fingerprint: <quoted source-statement debug>
primary-term-fingerprint: <quoted source-primary-term debug>
witness#0 owner=0 binding_context=1 term=primary#2 take_range=77..84 take_site=35 range=82..83 site=34 source_ordinal=1 ordinal=0 kind=unnamed recovery=normal spelling="x"
```

`SourceStatementWitnessError` has exactly `DependencyMismatch`,
`InvalidWitness { witness: SourceStatementWitnessId }`, and
`InvalidAggregate`. Its text names dependency mismatch, the invalid dense
witness ID, or invalid witness aggregate respectively.

`TypedAst` adds optional field/accessor
`source_statement_witnesses: Option<SourceStatementWitnessHandoff>` /
`source_statement_witnesses()`, and only
`with_source_statement_witnesses(statements, witnesses)` may publish the B3
pair. The existing base-only installer remains Task-258A/258B2-only and the
reference-paired installer remains Task-258B1-only. Final
`ResolvedTypedAst` adds the same field/accessor, revalidates, and
clone-preserves the same pair. An orphan half, standalone B3 base,
stale/cross-profile fingerprint, B1 references plus witnesses, Task-248,
any Task-257 family, any other source family, any semantic table, or either
ownership order fails atomically. Debug order is lower handoffs, base
`source-statement-debug-v1`, witness
`source-statement-witness-debug-v1`, then nodes; all earlier debug bytes stay
unchanged. B3 production may build the formula-only base independently for
producer validation, but no typed or final owner may install it without the
matching witness handoff.

The exact containment graph has theorem row 0 containing the conclusion,
take, witness, and their lower descendants; conclusion row 1 contains only
its own formula/terms. The base owns only its two formula rows, and the
companion owns only the take/witness wrappers. Duplicate sites, crossing
rows, a formula or term substituted from another row, witness term attached
to an atomic edge, recovered/degraded nodes, wrong child order, named or
multiple witnesses, any other term, missing/extra/reordered statements,
assumption, citation/label, composite theorem root, broader visibility, or
any byte change fails closed.

The future checker matrix is exactly four compound tests: complete
API/debug/lower-profile publication; exhaustive dependency/aggregate/base/
witness/all-index/provenance mutation with replay; typed ownership and all
Task-248/257/258 cross-family orders with rollback; and final
clone/orphan/stale-half rejection plus empty semantics. Mutations explicitly
cover base ordinals `0/2`, witness source/within-take ordinals `1/0`, the
combined partition, term-2 substitution by 0/1/3/4, binding context 0 or a
foreign proof scope, swapped take/witness sites, wrapper/reference
substitution, every range/spelling/kind/recovery field, independently stale
statement/primary fingerprints, and coherent replay. The complete API test
also freezes at the Rust type/public-surface level that witness input exposes
`BindingContextId` and no `SourceStatementContextId` field; this is not a
runtime mutation. The runner matrix is exactly five
compound tests: real frontend/resolver/lower identity; complete mutation/
replay; selector/subtree/byte near misses including named/multiple/missing/
extra witnesses, `take y`, reordered/extra statements, and composite/
existential roots; active-route and every A/B1/B2 family isolation in both
orders; and typed/final debug clone with empty semantic output. Tests may
use the existing syntax dev-dependency; production checker code remains
syntax-free.

Task 258B3 publishes no accepted witness, existential match, type obligation,
substitution, local abbreviation, fact, premise, checked formula, statement
semantic, proof node/goal, diagnostic, theorem status, IR, VC, cache, or
artifact. Tasks 258B3N/M explicitly retain named, multiple, and other
witness-term transport and must be separately frozen after B3 and before B4.
Task 258B4 retains composite theorem roots, Task 258B5 broader
imported/outer/inner visibility, and Tasks 269–272 binding,
closure/substitution, reconsider, proof-skeleton, justification, and goal
semantics.

The missing B3 contract is resolved `design_drift`. The absent exact
producer/paired ownership/dormant route is bounded `source_drift`; the absent
four/five matrices are `test_gap`. There is no blocking `spec_gap`,
`source_undocumented_behavior`, `test_expectation_drift`,
`boundary_violation`, or unresolved `repo_metadata_conflict`.
`spec.en.checker.formula_statement.source_payloads` remains deferred with
`tests = []`; the coverage audit records ownership only and awards no credit.

This documentation prerequisite changes no source, fixture, sidecar,
expectation, trace row/status/count, active route, test list, or hash.
Current baselines remain plan/type `419/387` and `253/241`, pass/fail
`228/191`, active parse/declaration/type/proof `101/5/198/1`,
warnings/errors `23/0`, checker/runner libraries `346/379`, and runner
production 30 paths / 36,479 lines. Checker test-list hashes are
`83fbd231030ff57c3c2c152c9374ca10579eb50797bd0b455a22a576b9f6edd5` /
`aa34d2780713de5b89ff75e24cc152797260daefdac064410120358980555119`;
runner hashes are
`3642d5057d7dc2f47c1b739b61f9c4272b823fe200bc72270e9345386df59586` /
`467fe747add608900943eaee02e333c8d672a3a4a433f9a4efa3fea4f4b21e5a`.
Runner path/content hashes remain
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`a0553883c61b5a113b3af509f58296cc97a7b1dfd31b6f82b1d71b95ff0f8bcb`.
Current checker module sizes are `source_statement.rs` 7,334 lines,
`typed_ast.rs` 4,550, `resolved_typed_ast.rs` 7,172, and unchanged
`binding_env.rs` 3,156. The five CLI hashes remain
`4cc13ea6bee6c1a6458d4a7d027a7eea685b711eda8410edafad8faa01809d54`,
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
`210055108c257ff65c6f45fb654c82e506653ec4617b68d111893bb3aa1da5a8`,
`f87a743b914d2d51d6b9a8dbcf3c8d93bbc1403b44907fd85123a1865f84edd5`,
and `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.
Implementation projects libraries `350/384` and keeps 30 production paths;
its exact line counts and changed test/content hashes must be measured, not
predicted.

Exit requires synchronized EN/JA documentation, independent no-findings
reviews, every hard gate, read-only quality at least 90/100, task-only
staging, and one dedicated documentation commit. Implementation may begin
only after that commit and a fresh parser/resolver/lower-API/count/hash
preflight.

## Task 258B3 Implementation Result

The frozen producer, row/table/handoff/error API, exact B3 base profile,
fingerprints, containment checks, combined `[0,1,2]` order, and deterministic
debug text are implemented. Four checker tests exercise publication,
dependency/aggregate/row/provenance corruption, paired ownership, final
revalidation, replay, and empty semantics. Bounded `source_drift` and
`test_gap` are closed; all semantic deferrals and the deferred trace row
remain unchanged.

## Task 258B3N Frozen Named-Witness Slice

Task 258B3N is the next dependency-ready slice after the Task 258B3
implementation. It is narrower than the former B3N/M umbrella: B3N owns one
named witness whose right-hand side is the already supported
reserved-variable term, while Task 258B3M retains multiple witnesses and
all other witness-term shapes. Task 258B4 remains blocked behind both.

Canonical authority is `doc/spec/en/15.statements.md` §§15.4.4 and 15.11.5,
`doc/spec/en/04.variables_and_constants.md` §4.4.3, Chapters 13 and 14 for
the reserved-variable/equality shells, the existing
`pass_parser_simple_statements_001.miz` parser fixture, the parser/resolver
fixtures, and the public Tasks 48/252/256/258A/B1/B2/B3 APIs. The grammar
authorizes `take y = x;` and Chapter 4 classifies `y` as a local name.
Section 15.11.5 assigns local-name use and existential-witness effects to
later semantics. B3N records the exact name occurrence but creates no
`BindingId`, local abbreviation, substitution, fact, obligation, proof
result, or accepted theorem. Task 269 owns the future local `BindingId`,
RHS link, capture-by-resolved-binding abbreviation replay, and context
transition for named `take`; Task 272 owns ordered existential-binder
matching, witness type-obligation requests, capture-avoiding goal
substitution, and the remaining goal. Task 270 remains only the
`deffunc`/`defpred` closure owner, and Task 271 remains only the
`reconsider` owner.

The exact future corpus-dormant consumer is this 107-byte final-LF source,
SHA-256
`a57022c4b75991dd4308943477e03819f5bfe2c0d23ea1030730256252d7d329`:

```mizar
reserve x for set;
theorem FormulaStatementNamedWitnessSmoke: x = x proof
  take y = x;
  thus x = x;
end;
```

The equality root keeps composite-root behavior in Task 258B4. Because
`take` requires an existential goal, this source is not a valid proof and
must not become an active corpus case.

Fresh real parser/resolver inventory freezes:

| Object | Exact identity |
| --- | --- |
| surface arena | 51 nodes, root 50, one source, all unrecovered |
| reserve/theorem | reserve node 27 `0..18`; theorem node 47 `19..106`; label token 6 `27..60` |
| theorem owner | one local public/exported theorem, contribution 0, range `19..106`, origin path `[2,1]`; no import |
| proof | node 46 `68..105`, lexical scope `[0]` |
| formula statements | theorem node 47 with wrapper 33 and atomic node 32 `62..67`; conclusion node 45 with wrapper 43 and atomic node 42 `95..100` |
| named witness | `TakeStatement` node 37 `76..87`; `Witness` node 36 `81..86`; name token 13 `81..82` spelling `y`; `=` token 14 `83..84`; RHS wrapper 35 and term/reference node 34 `85..86` spelling `x` |
| formula terms | wrappers 29/31 and Task-252 nodes 28/30 `62..63`/`66..67`; wrappers 39/41 and nodes 38/40 `95..96`/`99..100` |
| resolver labels | theorem owner projection only; no proof-step label, citation, label-reference key, or new resolver companion |

The exact syntax-free lower composition remains Task-48 `2/1/0`: module
context 0, proof context 1 owned by
`BindingContextOwner::SourceStatement { source_range: 68..105 }`, one
reserved binding 0, no diagnostic, empty proof-context binding list, and
visible binding `[0]`. The name token `y` is not a Task-48 binding in B3N.
Task 252 is `5/5/0`, with term/reference nodes
`28/30/34/38/40`, ranges `62..63`, `66..67`, `85..86`, `95..96`,
`99..100`, source ordinals `0..4`, contexts `0/0/1/1/1`, and use ordinal
1. The name token is not a primary term. Task 256 remains
`2/0/0/0/0/0/0/4/4`; its equality formulas are nodes 32/42 and its edges
target primary terms `[0,1,3,4]`, excluding witness RHS term 2.

The base statement profile remains `1/2/2/2/2`: owner 47, theorem and
conclusion statements at source ordinals 0/2, contexts 0/1, reserved guards,
and unverified candidates. The witness companion becomes `1 witness /
1 name`. Witness 0 has owner 0, proof binding context 1, primary term 2,
take node/range `37`/`76..87`, witness node/range `36`/`81..86`, spelling
`y = x`, source ordinal 1, within-take ordinal 0, kind `Named`, normal
recovery, and `name = Some(name#0)`. Name row 0 links to witness 0 and owns
token node/range `13`/`81..82`, spelling `y`, and normal recovery. The base
plus witness partition is exactly `[0,1,2]`.

The frozen public-table extension is:

- `SourceStatementWitnessNameId` has the existing dense-ID contract:
  `Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash`, with
  public `const fn new(usize) -> Self` and `const fn index(self) -> usize`;
- public `SourceStatementWitnessNameInput` derives
  `Debug + Clone + PartialEq + Eq` and has exactly the public fields
  `witness: SourceStatementWitnessId`, `site: TypedSiteRef`,
  `source_range: SourceRange`, `spelling: String`, and
  `recovery: SourceStatementRecovery`;
- immutable public row `SourceStatementWitnessName` derives
  `Debug + Clone + PartialEq + Eq`, keeps the same five fields private, and
  exposes `pub const fn witness(&self) -> SourceStatementWitnessId`,
  `pub const fn site(&self) -> &TypedSiteRef`,
  `pub const fn source_range(&self) -> SourceRange`,
  `pub fn spelling(&self) -> &str`, and
  `pub const fn recovery(&self) -> SourceStatementRecovery`;
- immutable public `SourceStatementWitnessNameTable` derives
  `Debug + Clone + PartialEq + Eq` and has the standard dense
  `pub fn get(&self, SourceStatementWitnessNameId)
  -> Option<&SourceStatementWitnessName>`,
  `pub fn iter(&self)
  -> impl Iterator<Item = (SourceStatementWitnessNameId,
  &SourceStatementWitnessName)>`, `pub const fn len(&self) -> usize`, and
  `pub const fn is_empty(&self) -> bool` accessors;
- `SourceStatementWitnessHandoffInput` adds exactly
  `pub names: Vec<SourceStatementWitnessNameInput>`;
  `SourceStatementWitnessHandoff` stores the table and exposes
  `pub const fn names(&self) -> &SourceStatementWitnessNameTable`;
- `SourceStatementWitnessInput` adds
  `pub name: Option<SourceStatementWitnessNameId>`;
  `SourceStatementWitness` stores it and exposes
  `pub const fn name(&self) -> Option<SourceStatementWitnessNameId>`;
  `SourceStatementWitnessKind` adds only `Named`;
- a name row contains no resolver symbol, `BindingId`, type, substitution,
  or semantic status. Task 258B3 remains one `Unnamed` witness with
  `name = None` and an empty name table;
- only exact B3 `(1 witness, 0 names)` or B3N `(1 witness, 1 name)`
  aggregates are valid. Dependencies and the complete shared arena are
  authenticated first, then aggregate cardinality, witness rows, and name
  rows in that order. A bad profile/count is `InvalidAggregate`; a wrong
  witness kind/name option or forward link is
  `InvalidWitness { witness }`; a bad name row or reverse witness link is
  the new `InvalidName { name: SourceStatementWitnessNameId }`. Its display
  text is `source statement witness name {index} is invalid`.

`debug_text()` retains the `source-statement-witness-debug-v1` header and
keeps every Task-258B3 byte unchanged. A named witness appends
` name={name.index()}` to its existing witness line. Dense name rows follow
all witness rows, each exactly
`witness-name#{id} witness={witness} range={start}..{end} site={site} recovery={recovery} spelling={spelling:?}`.
Thus empty B3 names emit no new bytes, while B3N name identity and ordering
are deterministic. Every hybrid, orphan, duplicate, sparse, reordered,
stale-fingerprint, or cross-profile table fails under the precedence above.

`SourceStatementWitnessProducer`, `TypedAst`, and `ResolvedTypedAst` retain
the paired base/witness installation API. B3N may install only the
authenticated B3N base/witness/name bundle; standalone halves, B3/B3N
hybrids, reference hybrids, Task-248/257/other-258 ownership, and semantic
coexistence fail atomically. All 51 nodes must match the frozen range, kind,
normal recovery, and ordered child list.

The checker test contract is exactly four compound tests:

1. complete API/debug, B3 compatibility, B3N lower/base/witness/name
   publication, resolver owner, all accessors, and empty semantics;
2. exhaustive dependency/aggregate/row/name/fingerprint/provenance and
   all-51-node range/kind/Recovered/Degraded/child mutation with replay;
3. paired typed ownership, B3/B3N hybrid rejection, and every existing
   Task-248/257/258 order with rollback;
4. final clone/revalidation, orphan/stale-half/reference-hybrid and every
   semantic-table/proof/goal coexistence rejection.

The runner test contract is exactly five compound tests:

1. real bytes/hash, parser/resolver identity, Task-48/252/256/base,
   witness/name rows, combined ordinals, arena parity, and paired output;
2. exhaustive lower/base/witness/name/fingerprint/resolver/all-index mutation
   with deterministic replay;
3. selector and byte/subtree near misses including unnamed, changed/missing
   name, missing `=`, multiple witnesses, non-primary RHS, reordered/extra
   statements, composite/existential roots, and recovery;
4. B3N/B3/B2/B1/A and active-route isolation in both ownership orders;
5. typed/final clone/debug, rollback, and empty semantic output.

This prerequisite changes no production or test source, `doc/spec`, `.miz`,
fixture, sidecar, expectation, trace row/status/count, active route, test
list, count, or hash. Current baselines are checker/runner libraries
`350/384`, checker modules `9812/4644/7195/3156`, runner
production leaf/facade/root/test leaf `2806/681/2495/4291`, production 30
paths / 37,172 lines, plan/type `419/387` and `253/241`, pass/fail
`228/191`, active parse/declaration/type/proof `101/5/198/1`, and
warnings/errors `23/0`. Test-list hashes remain
`67b97e6594a4208aa0e0413c072b7f21809e9f88c7ab97671d6a9dea16c831a7` /
`cef91e5ce85dde5101147206de5c066b229651b7d4d4a99a3543c09e618e4651`
and
`4a077d6ab1fa4d881ae4d8d46afd003e785be573d8438772e9fbffe37374cd2f` /
`9d0c11fe6e48f136525ef4b0ca61235d8b4d0a16b703b12ba2c378d1f947b2ae`.
Production path/content hashes remain
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`adfc81c21e69a91b194161525856aa40eb0e3ea76facfc2146dcb00b473ab3c2`;
the five CLI hashes remain the Task-258B3 values.

Implementation projects exactly four checker and five runner compound tests,
hence libraries `354/389`; changed module sizes and content/test hashes must
be measured, not predicted. The missing exact named-witness contract is
resolved `design_drift`; future producer/route work is bounded
`source_drift`, and the future matrices are `test_gap`. There is no blocking
`spec_gap`, `source_undocumented_behavior`, `test_expectation_drift`,
`boundary_violation`, or `repo_metadata_conflict`.

`spec.en.checker.formula_statement.source_payloads` remains deferred with
`tests = []`. The coverage audit records only frozen B3N ownership; no
semantic credit is awarded. Exit requires synchronized EN/JA documents,
independent no-findings reviews, all hard gates, read-only quality at least
90/100, task-only staging, and one dedicated documentation commit. Only
after that commit and a fresh parser/resolver/lower/count/hash preflight may
B3N implementation begin. Task 258B3M remains the next documentation
prerequisite; Task 258B4 remains blocked behind B3M.

## Task 258B3N Implementation Result

The syntax-only named witness is implemented exactly as frozen: one
`Named` witness row points to one dense name row for token `y`; B3 remains
`Unnamed` with no name rows and byte-identical v1 debug output. Validation
authenticates the exact base/lower fingerprints, 51-node arena, forward and
reverse name links, subtree boundaries, and
dependency/aggregate/witness/name error precedence. It creates no binding,
abbreviation, obligation, fact, proof result, goal transition, or accepted
theorem. Four checker and five runner compound tests close the bounded
`source_drift`/`test_gap`; Task 258B3M is next.

## Task 258B3M1 Frozen Mixed Multiple-Witness Slice

Fresh inventory decomposes the former open-ended Task 258B3M into two
dependency-ordered slices. Task 258B3M1 owns only two reserved-variable
witness rows in one `take`: a named row followed by an unnamed row. Task
258B3M2 retains every non-reserved-variable and other witness-term shape.
Task 258B4 remains blocked behind B3M2.

Canonical authority is `doc/spec/en/15.statements.md` §§15.4.4 and 15.11.5,
`doc/spec/en/16.theorems_and_proofs.md` §16.3.3 item 5, and
`doc/spec/en/04.variables_and_constants.md` §4.4.3. The existing
`pass_parser_simple_statements_001.miz` fixture already contains the mixed
shape `take a = x, y;`; its parser test requires one `TakeStatement` and two
`Witness` nodes. The parser consumes comma-separated witnesses in source
order and accepts either `identifier = term_expression` or
`term_expression`. This authority freezes syntax transport only. Task 269
still owns a future binding for `y`, its RHS link, abbreviation replay, and
context transition. Task 272 still owns ordered existential-binder
matching, witness type obligations, capture-avoiding substitution, and the
remaining goal. Tasks 270/271 remain limited to `deffunc`/`defpred` closure
and `reconsider`.

The exact future corpus-dormant consumer is this 113-byte final-LF source,
SHA-256
`412a6a7f8fddebd67418f3482855ea89a1e7da922b42ebb93463971d8e49c186`:

```mizar
reserve x for set;
theorem FormulaStatementMultipleWitnessSmoke: x = x proof
  take y = x, x;
  thus x = x;
end;
```

Its equality goal is not an existential claim, so the source is not a valid
proof and must never become an active accepted corpus case.

Fresh real frontend inventory freezes 56 unrecovered nodes with root 55 and
one source. Token nodes are exactly
`0:reserve@0..7, 1:x@8..9, 2:for@10..13, 3:set@14..17,
4:;@17..18, 5:theorem@19..26,
6:FormulaStatementMultipleWitnessSmoke@27..63, 7::@63..64,
8:x@65..66, 9:=@67..68, 10:x@69..70, 11:proof@71..76,
12:take@79..83, 13:y@84..85, 14:=@86..87, 15:x@88..89,
16:,@89..90, 17:x@91..92, 18:;@92..93, 19:thus@96..100,
20:x@101..102, 21:=@103..104, 22:x@105..106, 23:;@106..107,
24:end@108..111, 25:;@111..112`; each has no child. Structural nodes are:

| IDs | Exact kind, range, ordered children |
| --- | --- |
| 26–29 | `TypeHead 14..17 [3]`; `TypeExpression 14..17 [26]`; `ReserveSegment 8..17 [1,2,27]`; `ReserveItem 0..18 [0,28,4]` |
| 30–35 | `TermReference 65..66 [8]`; `TermExpression 65..66 [30]`; `TermReference 69..70 [10]`; `TermExpression 69..70 [32]`; `BuiltinPredicateApplication 65..70 [31,9,33]`; `FormulaExpression 65..70 [34]` |
| 36–42 | `TermReference 88..89 [15]`; `TermExpression 88..89 [36]`; `Witness 84..89 [13,14,37]`; `TermReference 91..92 [17]`; `TermExpression 91..92 [39]`; `Witness 91..92 [40]`; `TakeStatement 79..93 [12,38,16,41,18]` |
| 43–50 | `TermReference 101..102 [20]`; `TermExpression 101..102 [43]`; `TermReference 105..106 [22]`; `TermExpression 105..106 [45]`; `BuiltinPredicateApplication 101..106 [44,21,46]`; `FormulaExpression 101..106 [47]`; `Proposition 101..106 [48]`; `ConclusionStatement 96..107 [19,49,23]` |
| 51–55 | `ProofBlock 71..111 [11,42,50,24]`; `TheoremItem 19..112 [5,6,7,35,51,25]`; `ItemList 0..112 [29,52]`; `CompilationUnit 0..112 [53]`; `Root 0..112 [0..25,54]` |

Resolver provenance is exactly one local public/exported theorem owner and
label, contribution 0, owner range `19..112`, label range `27..63`,
structural origin `[2,1]`, normal recovery, and no import, proof-step label,
citation, label-reference key, witness-name symbol, or companion resolver
bundle.

The syntax-free lower composition is Task-48 `2/1/0`: module context 0,
proof context 1 owned by source range `71..111`, one reserved binding 0,
visible binding `[0]`, empty proof-owned binding list, and no diagnostic.
Token `y` is not a `BindingId`. Task 252 is `6/6/0`, with term/reference
nodes `30/32/36/39/43/45`, ranges `65..66`, `69..70`, `88..89`,
`91..92`, `101..102`, `105..106`, source ordinals `0..5`, contexts
`0/0/1/1/1/1`, binding 0, scope `[0]`, and use ordinal 1. Task 256 remains
`2/0/0/0/0/0/0/4/4`; its equality nodes 34/47 target primary terms
`[0,1,4,5]` and exclude both witness terms 2/3.

The base statement profile remains `1/2/2/2/2`: owner/theorem node 52 and
conclusion node 50 have source ordinals 0/2 and contexts 0/1. The witness
companion becomes exactly `2 witnesses / 1 name`:

| Row | Frozen syntax-only identity |
| --- | --- |
| witness 0 | owner 0; context 1; primary term 2; take node/range `42`/`79..93`; item node/range `38`/`84..89`; source ordinal 1; within-`take` ordinal 0; spelling `y = x`; `Named`; normal; `Some(name#0)` |
| witness 1 | owner 0; context 1; primary term 3; take node/range `42`/`79..93`; item node/range `41`/`91..92`; source ordinal 1; within-`take` ordinal 1; spelling `x`; `Unnamed`; normal; no name |
| name 0 | links only witness 0; token node/range `13`/`84..85`; spelling `y`; normal |

The two rows share source ordinal 1 because they belong to one source
`take` item; their dense within-`take` ordinals preserve syntax order only.
The combined source-item order is theorem 0, both witness rows at 1, and
conclusion 2. It does not assert left-to-right goal effect.

No public type, enum variant, field, or installer is added. The existing
dense witness/name tables, `Named`/`Unnamed` kinds, primary-term target,
`SourceStatementWitnessProducer`,
`TypedAst::with_source_statement_witnesses`, and final
`ResolvedTypedAst` ownership are sufficient. The private validator adds
only an exact B3M1 profile. Dependency/fingerprint/complete shared-arena
validation precedes aggregate cardinality; witness rows are then validated
in dense order before name rows. Reordered kind/name/term/ordinal links,
orphan or duplicate names, B3/B3N/B3M1 hybrids, sparse rows, and copied
dependencies fail atomically under the existing
`DependencyMismatch` / `InvalidAggregate` /
`InvalidWitness { witness }` / `InvalidName { name }` precedence.

The typed arena assigns `source.statement-witness.take` to node 42,
`source.statement-witness.item` to nodes 38/41, and
`source.statement-witness.name` only to token 13. Term-expression wrappers
37/40 remain unowned and Task 252 owns references 36/39. The take contains
exactly named witness 0, comma, unnamed witness 1 in order. The name is a
descendant only of witness 0; the two RHS wrapper/reference subtrees are
distinct siblings. Both witnesses are descendants of theorem/proof/take
and excluded from the conclusion subtree; Task 256 excludes both.

`source-statement-witness-debug-v1` stays unchanged. B3 and B3N debug bytes
remain byte-identical. B3M1 emits witness rows 0 then 1 using the existing
line grammar, followed by name row 0. The paired typed/final owner rejects
standalone halves, reference hybrids, every Task-248/257/other-258 family
in either order, and any nonempty semantic/proof/goal table. Successful
assembly clone-preserves the pair and leaves every semantic output empty.

The checker test contract is exactly four compound tests:

1. complete API/debug, B3/B3N compatibility, exact B3M1 lower/base/
   witness/name publication, resolver provenance, and empty semantics;
2. dependency/cardinality, statement/primary fingerprints, each
   witness/name/order/link/provenance field, mixed-fault precedence
   `DependencyMismatch` before `InvalidAggregate`, then witness 0 before
   witness 1 before the name row, and every one of the 56 nodes with
   range/kind/child corruption plus both `NodeRecoveryState::Recovered`
   and `Degraded`, all with deterministic replay;
3. paired typed ownership, B3M1/B3N/B3 hybrids, and every existing
   Task-248/257/258 ownership order with rollback;
4. final clone/revalidation plus orphan, independently stale private
   statement and primary fingerprints, reference-hybrid, and every
   semantic/proof/goal coexistence rejection.

The runner test contract is exactly five compound tests:

1. exact bytes/hash, parser/resolver identity, Task-48/252/256/base,
   both witness rows, name row, ordinals, shared arena, and paired output;
2. exhaustive lower/base/witness/name/resolver/all-index mutation,
   positive public statement/primary fingerprint equality, copied
   cross-profile handoff and aggregate/cardinality corruption, mixed-fault
   dependency/aggregate/witness-0/witness-1/name precedence, and
   deterministic replay;
3. selector/byte/subtree near misses including reversed named/unnamed,
   both named, both unnamed, missing/extra/reordered witnesses, changed
   comma/name/`=`, non-primary RHS, recovery, and composite/existential roots;
4. B3M1/B3N/B3/B2/B1/A and active-route isolation in both ownership orders;
5. typed/final debug clone, rollback, and empty semantic output.

This documentation prerequisite changes no production or test source,
`doc/spec`, existing `.miz`, fixture, sidecar, expectation, trace
row/status/count, active route, test list, count, or hash. Current baselines
remain checker/runner libraries `354/389`, checker modules
`12114/4644/7200/3156`, runner statement leaf/facade/root/test leaf
`3183/684/2498/5799`, production 30 paths / 37,555 lines, plan/type
`419/387` and `253/241`, pass/fail `228/191`, active
parse/declaration/type/proof `101/5/198/1`, and warnings/errors `23/0`.
Test-list hashes remain
`3b4eb710711061fed2c008e7e7f10e3c433398c5ddca050464d8e0d2dc9fc3af` /
`3be45d9cbe826df9fc4562feda0350c751fbcfeb776296ffba676f8cc0d54cae`
and
`bb6cbbad01b281ac0e55b2944ddc83bee73903ededa2501f4343a4b4ffb645ce` /
`65e097ba6f86648b45cf3b7bcf5a888a7e3b0498ea30ee88277960d49af60ccf`.
Runner production path/content hashes remain
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`2289634cb6126854e1382093f1adedd6d0608d0e1d241ff33b1eedd48a4716eb`;
the five CLI hashes remain the Task-258B3N values.

Implementation projects exactly four checker and five runner tests, hence
libraries `358/394`; changed sizes and hashes must be measured afterward.
Decomposing the broad B3M wording and freezing B3M1 resolves
`design_drift`; future code is bounded `source_drift` and future tests are
`test_gap`. There is no blocking `spec_gap`,
`source_undocumented_behavior`, `test_expectation_drift`,
`boundary_violation`, or `repo_metadata_conflict`, and no lower-stage
prerequisite. The coverage audit changes follow-up ownership only;
`spec.en.checker.formula_statement.source_payloads` remains `deferred` with
`tests = []` and gains no credit.

Exit requires synchronized EN/JA documentation, independent no-findings
reviews, all protocol hard gates, read-only quality at least 90/100,
task-only staging, and one dedicated documentation commit. Implementation
may begin only after that commit and a fresh parser/resolver/lower/count/hash
preflight.

## Task 258B3M2A Frozen Numeral-Witness Slice

Fresh post-Lexer-Task-258B3M2P1 inventory first decomposed the broad B3M2
“other witness-term shapes” umbrella into B3M2A and B3M2B. B3M2A owns only
one unnamed numeral witness. The later B3M2B1 contract owns a parenthesized
wrapper with its reserved-variable child, while B3M2B2 retains compound,
application, selector, update, set, choice, and other authority-valid terms.
It retains `it` only in a Chapter-13-valid `means` context. Task 258B4
remains blocked behind B3M2B2.

Canonical authority is `doc/spec/en/15.statements.md` §15.4.4, which defines
an unnamed example as `term_expression` and gives `take 101;` verbatim;
`doc/spec/en/13.term_expression.md` §§13.1, 13.1.4, and 13.9, which make a
numeral a primary term; `doc/spec/en/04.variables_and_constants.md` §4.4.3,
which states that an unnamed witness introduces no local name; and
`doc/spec/en/16.theorems_and_proofs.md` §16.3.3 item 5. Chapter 15 §15.11.5
owns the later witness type obligation and existential substitution. This
task freezes syntax transport only. Task 252 retains the numeral occurrence
and unresolved numeric-type request; Task 272 retains type inference,
existential matching, substitution, remaining-goal construction, and proof
acceptance. Task 269 receives no binding work because the witness is unnamed.

The exact future corpus-dormant consumer is this final-LF 107-byte source,
SHA-256
`7b424949e98761b0179758065db5d164ad7d0a640f082801986683a54c43a2d1`:

```mizar
reserve x for set;
theorem FormulaStatementNumeralWitnessSmoke: x = x proof
  take 101;
  thus x = x;
end;
```

Its equality goal is not existential, so it is not a valid proof and must
never become an active accepted corpus case. The dedicated lexer
prerequisite is complete: a fresh real frontend run has zero diagnostics.

The fresh frontend inventory freezes one source with 49 unrecovered nodes
and root 48. Token nodes are exactly
`0:reserve@0..7, 1:x@8..9, 2:for@10..13, 3:set@14..17,
4:;@17..18, 5:theorem@19..26,
6:FormulaStatementNumeralWitnessSmoke@27..62, 7::@62..63,
8:x@64..65, 9:=@66..67, 10:x@68..69, 11:proof@70..75,
12:take@78..82, 13:101@83..86, 14:;@86..87, 15:thus@90..94,
16:x@95..96, 17:=@97..98, 18:x@99..100, 19:;@100..101,
20:end@102..105, 21:;@105..106`; each has no child. Structural nodes are:

| IDs | Exact kind, range, ordered children |
| --- | --- |
| 22–25 | `TypeHead 14..17 [3]`; `TypeExpression 14..17 [22]`; `ReserveSegment 8..17 [1,2,23]`; `ReserveItem 0..18 [0,24,4]` |
| 26–31 | `TermReference 64..65 [8]`; `TermExpression 64..65 [26]`; `TermReference 68..69 [10]`; `TermExpression 68..69 [28]`; `BuiltinPredicateApplication 64..69 [27,9,29]`; `FormulaExpression 64..69 [30]` |
| 32–35 | `NumeralTerm 83..86 [13]`; `TermExpression 83..86 [32]`; `Witness 83..86 [33]`; `TakeStatement 78..87 [12,34,14]` |
| 36–43 | `TermReference 95..96 [16]`; `TermExpression 95..96 [36]`; `TermReference 99..100 [18]`; `TermExpression 99..100 [38]`; `BuiltinPredicateApplication 95..100 [37,17,39]`; `FormulaExpression 95..100 [40]`; `Proposition 95..100 [41]`; `ConclusionStatement 90..101 [15,42,19]` |
| 44–48 | `ProofBlock 70..105 [11,35,43,20]`; `TheoremItem 19..106 [5,6,7,31,44,21]`; `ItemList 0..106 [25,45]`; `CompilationUnit 0..106 [46]`; `Root 0..106 [0..21,47]` |

Resolver provenance is exactly one local public/exported theorem owner and
label, contribution 0, owner range `19..106`, label range `27..62`,
structural origin `[2,1]`, and normal recovery. There is no import,
proof-step label, citation, label-reference key, witness-name symbol, or
companion resolver handoff. The private runner may reuse the existing exact
theorem-owner enrichment; it must not publish a new resolver API.

The syntax-free lower composition is:

- Task 48 `2/1/0`: module context 0, proof context 1 owned by `70..105`,
  reserved binding 0 visible as `[0]`, no proof-owned binding, and no
  diagnostic;
- Task 252 `5/4/1`: terms at nodes `26/28/32/36/38`, ranges `64..65`,
  `68..69`, `83..86`, `95..96`, `99..100`, source ordinals `0..4`,
  contexts `0/0/1/1/1`; dense reference IDs `0/1/2/3` target terms
  `0/1/3/4` respectively, resolve binding 0 with exact lexical-scope vector
  `[]/[]/[0]/[0]`, and have use ordinal 1; numeral term 2 has kind
  `Numeral`, spelling `101`, normal `Value` role, no reference, and numeric
  request 0 owned by node/range `32`/`83..86` with request ordinal 0;
- Task 256 `2/0/0/0/0/0/0/4/4`: equality nodes 30 and 40 target primary
  pairs `[0,1]` and `[3,4]`; numeral witness term 2 is excluded from every
  atomic edge and request;
- base statement `1/2/2/2/2`: theorem node 45 and conclusion node 43 have
  source ordinals 0 and 2 and contexts 0 and 1. The theorem input-fact row
  uses dense references `[0,1]`, which target terms `[0,1]`; the conclusion
  row uses references `[2,3]`, which target terms `[3,4]`. Term IDs must
  never be reused as reference IDs across the numeral hole.

The witness companion is exactly `1 witness / 0 names`. Witness 0 owns
owner 0, binding context 1, primary term 2, take node/range `35`/`78..87`,
item node/range `34`/`83..86`, source ordinal 1, within-`take` ordinal 0,
spelling `101`, kind `Unnamed`, normal recovery, and no name. The combined
source-item partition is exactly `[0,1,2]`. This is syntax order only and
does not claim an existential goal effect.

No public type, variant, field, error, table, accessor, producer, or
installer is added. Existing `SourcePrimaryTermKind::Numeral`,
`SourceNumericTypeRequestTable`,
`SourceStatementWitnessTermTarget::Primary`, witness/name tables,
`SourceStatementWitnessProducer`,
`TypedAst::with_source_statement_witnesses`, and final ownership are
sufficient. Private base/witness profile selectors add only B3M2A.
Validation precedence remains dependency/fingerprint and complete-arena
authentication, aggregate cardinality, witness row 0, then name rows
(empty): `DependencyMismatch` precedes `InvalidAggregate`, which precedes
`InvalidWitness { witness: 0 }`; `InvalidName` is unreachable for the
empty name table. The numeral row and numeric request remain Task-252-owned inputs;
the witness table does not duplicate their type semantics.

The typed arena assigns `source.term.numeral` to node 32,
`source.statement-witness.item` to node 34,
`source.statement-witness.take` to node 35,
`source.formula.atomic.equality` to nodes 30/40,
`source.statement.conclusion` to node 43, and
`source.statement.theorem` to node 45. Wrapper node 33 is unowned. The
witness subtree is a theorem/proof/take descendant and is disjoint from
both equality subtrees and the conclusion. Task 256 must continue excluding
primary term 2.

`source-statement-witness-debug-v1` remains unchanged, and B3/B3N/B3M1
debug bytes remain byte-identical. The paired typed/final owner rejects
base-only or witness-only installation, B3/B3N/B3M1/B3M2A hybrids,
reference hybrids, every Task-248/257/other-258 family in either order,
stale fingerprints, numeric-request corruption, and any nonempty
semantic/proof/goal table. Successful assembly clone-preserves the pair and
leaves every semantic output empty.

The checker test contract is exactly four compound tests:

1. `task258b3m2a_exact_numeral_witness_api_debug_and_compatibility_are_stable`;
2. `task258b3m2a_dependencies_numeric_request_witness_precedence_and_all_nodes_fail_closed`;
3. `task258b3m2a_paired_ownership_hybrids_and_all_family_orders_are_atomic`;
4. `task258b3m2a_final_clone_revalidation_and_semantic_deferrals_are_stable`.

Together they freeze the exact lower/base/witness publication, public API
no-op and prior debug bytes; dense reference-to-term mapping, both exact
input-fact `uses` rows, dependency/aggregate/witness precedence, numeric
request, and all 49 node mutations with recovered/degraded replay; paired
ownership and all family orders; and final clone plus empty semantics.

The runner test contract is exactly five compound tests:

1. `task258b3m2a_real_frontend_freezes_numeral_witness_contract`;
2. `task258b3m2a_validation_precedence_mutation_and_replay_fail_closed`;
3. `task258b3m2a_selector_and_byte_subtree_near_misses_are_exact`;
4. `task258b3m2a_family_and_active_route_isolation_is_atomic_in_both_orders`;
5. `task258b3m2a_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`.

Near misses include another numeral, missing/extra sign or token, named
numeral, multiple witnesses, identifier/`it`/parenthesized/application/
selector/update/set/choice RHS, recovery, changed theorem shape, and
existential/composite roots. The exact dormant selector runs before
B3M1/B3N/B3/B2/B1/A without adding a public harness route or detail key.
For the exact authenticated B3M2A output, the existing private detail
projection returns `Some(Vec::new())`: it requires the paired witness
handoff, two base statements, lookup ordinals `1/1`, and exactly four
reference use ordinals `[1; 4]`. `None` remains a selector miss, and an
owned but invalid output retains the existing
`type_elaboration.checker.typed_ast_invalid` detail.
The real-frontend and final compound tests assert the dense
reference-to-term mapping, both input-fact `uses` rows, and this exact
detail projection.

This documentation prerequisite changes no production/test source,
canonical specification, existing `.miz`, fixture, expectation, sidecar,
trace row/status/count, active route, test list, count, or hash. Fresh
baselines are checker/runner libraries `358/394`, checker module sizes
`14045/4659/7201/3156`, runner statement leaf/facade/root/test sizes
`3724/688/2501/7246`, and runner production 30 paths / 38,103 lines.
Checker raw/normalized test-list hashes are
`39c9d84a4fe990f3a74d69554aeb5be6d41349bd8dfe40d0bc269eacab5355d5` /
`cd4e902f325c08226c10deeec64c3b8de1d11f346d82a81f1008687f009c372f`;
runner hashes are
`e729eaf60f00a53a9767375d8718ea8179c27bf3c660c5a936eaeeea2ef8d00a` /
`af7e5ed68cec3e3feda6fb2264471b359443e849cf0f67ed4d111207e008bb12`.
Runner production path/content hashes are
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`b5b3523f13bd5b0ef5da10bd003db75fc89fd98d9d23300071f468ec22746c19`.
Plan/type remain `419/387` and `253/241`; pass/fail `228/191`; active
parse/declaration/type/proof `101/5/198/1`; warnings/errors `23/0`; and the
five CLI hashes remain exactly
`4cc13ea6bee6c1a6458d4a7d027a7eea685b711eda8410edafad8faa01809d54`
(plan),
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`
(parse),
`210055108c257ff65c6f45fb654c82e506653ec4617b68d111893bb3aa1da5a8`
(declaration),
`f87a743b914d2d51d6b9a8dbcf3c8d93bbc1403b44907fd85123a1865f84edd5`
(type), and
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`
(proof).

Implementation projects exactly four checker and five runner tests, hence
libraries `362/399`; every changed size and hash is a measured result, not a
target. Decomposing B3M2 and freezing B3M2A resolves `design_drift`; future
private code is bounded `source_drift`, and future tests are `test_gap`.
There is no blocking `spec_gap`, `source_undocumented_behavior`,
`test_expectation_drift`, or `boundary_violation`. A report-only
`repo_metadata_conflict` occurred after
the contract was drafted: while local HEAD remained
`076555b7ba61788e30c4266c0a9fd0375004c4de`, the remote-tracking
`origin/main` moved externally from `1e81db7a` to that same commit at
2026-07-28 19:46:29 +0900, changing the measured ahead count from 12 to 0.
The task-owned paths, clean committed base, and untouched `stash@{0}` remain
unambiguous, so the conflict is nonblocking and is not repaired here. The
completed lexer prerequisite closes the only lower-stage defect.

The coverage audit changes follow-up ownership only.
`spec.en.checker.formula_statement.source_payloads` remains `deferred` with
`tests = []` and receives no credit. Exit requires synchronized EN/JA
documents, independent no-findings reviews, all protocol hard gates,
read-only quality at least 90/100, task-only staging, and one dedicated
documentation commit. Implementation may begin only after that commit and a
fresh parser/resolver/lower/count/hash preflight.

## Task 258B3M2A Implementation Result

The checker now recognizes one private `Task258B3M2A` syntax-free profile.
The runner authenticates the final-LF 107-byte source before dispatch; the
checker fail-closed authenticates its 49-node arena projection/root 48, exact
module/proof binding contexts, five primary terms,
dense references `0/1/2/3 -> 0/1/3/4` with scopes `[]/[]/[0]/[0]`, numeric
request 0 on numeral term 2, both Task-256 equalities over only
`[0,1,3,4]`, and the `1/2/2/2/2` base. The witness validator then publishes
exactly one unnamed witness targeting term 2, no names, and source partition
`[0,1,2]`; dependency, aggregate, and witness validation precedence remains
explicit.

The paired typed/final consumers accept only this base/witness pair.
Standalone, stale, reordered, cross-family, subtree, resolver, lower-table,
numeric-request, and every node/byte mutation fail without partial
ownership. Four checker and five runner compound tests pass. The private
detail projection remains `Some(Vec::new())` with lookup ordinals `1/1` and
reference use ordinals `[1;4]`; all semantic/proof/goal tables remain empty.

Libraries are `362/399`. Checker module sizes are
`15746/4660/7202/3156`; runner statement leaf/facade/root/test sizes are
`4185/691/2505/8611`, and runner production is 30 paths / 38,571 lines.
No canonical specification, `.miz`, fixture, expectation, sidecar, trace
row/status/count, active route, public API, binding, or semantic owner
changed. This closes the bounded `source_drift`/`test_gap`; B3M2B remains
next before B4.

## Task 258B3M2B1 Frozen Parenthesized-Witness Slice

Fresh post-B3M2A inventory decomposes the remaining B3M2B umbrella into
dependency-ordered B3M2B1 and B3M2B2. B3M2B1 owns only one unnamed
parenthesized reserved-variable witness, `take (x);`. B3M2B2 retains
application, structure, selector, update, set, choice, compound, and other
authority-valid witness terms. It also retains `it` only where Chapter 13
§13.1.2 permits `it` in a valid `means` definition or property context;
this theorem-proof slice does not authorize `take it;`. B4 remains blocked
behind B3M2B2.

Canonical authority is `doc/spec/en/15.statements.md` §15.4.4, where an
unnamed example is any `term_expression`, and
`doc/spec/en/13.term_expression.md` §§13.1, 13.1.3, 13.8.8, and 13.9,
where `( term_expression )` is a type-preserving primary term. Existing
`tests/miz/pass/parser/pass_parser_simple_statements_001.miz` and its
expectation authenticate unnamed `take` syntax. The active
`tests/miz/pass/types/pass_type_elaboration_parenthesized_reserved_variable_equality_001.miz`,
its existing expectation, and the covered
`spec.en.checker.type_elaboration.source_primary_term_payload` trace row
authenticate the real wrapper/child and reserved-binding provenance.
The earlier B3/B3M1 contracts authenticate `x` in proof scope. Chapter 15
§15.11.5 and Task 272 retain type obligations, existential matching,
substitution, remaining-goal construction, and proof acceptance. None of
these existing consumers or trace artifacts changes.

The exact future corpus-dormant consumer is this final-LF 113-byte source,
SHA-256
`f09815b49d1b4598218f656a366ef73ec0dffd1f581a1018f07aa2ebcf410bf2`:

```mizar
reserve x for set;
theorem FormulaStatementParenthesizedWitnessSmoke: x = x proof
  take (x);
  thus x = x;
end;
```

Its equality goal is deliberately not existential, so it is not a valid
proof and must not become an active accepted corpus case. A fresh frontend
run yields zero diagnostics, 53 unrecovered nodes, and root 52. Token nodes
are exactly
`0:reserve@0..7, 1:x@8..9, 2:for@10..13, 3:set@14..17,
4:;@17..18, 5:theorem@19..26,
6:FormulaStatementParenthesizedWitnessSmoke@27..68, 7::@68..69,
8:x@70..71, 9:=@72..73, 10:x@74..75, 11:proof@76..81,
12:take@84..88, 13:(@89..90, 14:x@90..91, 15:)@91..92,
16:;@92..93, 17:thus@96..100, 18:x@101..102, 19:=@103..104,
20:x@105..106, 21:;@106..107, 22:end@108..111,
23:;@111..112`; each has no child. Structural nodes are:

| IDs | Exact kind, range, ordered children |
| --- | --- |
| 24–27 | `TypeHead 14..17 [3]`; `TypeExpression 14..17 [24]`; `ReserveSegment 8..17 [1,2,25]`; `ReserveItem 0..18 [0,26,4]` |
| 28–33 | `TermReference 70..71 [8]`; `TermExpression 70..71 [28]`; `TermReference 74..75 [10]`; `TermExpression 74..75 [30]`; `BuiltinPredicateApplication 70..75 [29,9,31]`; `FormulaExpression 70..75 [32]` |
| 34–39 | `TermReference 90..91 [14]`; `TermExpression 90..91 [34]`; `ParenthesizedTerm 89..92 [13,35,15]`; `TermExpression 89..92 [36]`; `Witness 89..92 [37]`; `TakeStatement 84..93 [12,38,16]` |
| 40–47 | `TermReference 101..102 [18]`; `TermExpression 101..102 [40]`; `TermReference 105..106 [20]`; `TermExpression 105..106 [42]`; `BuiltinPredicateApplication 101..106 [41,19,43]`; `FormulaExpression 101..106 [44]`; `Proposition 101..106 [45]`; `ConclusionStatement 96..107 [17,46,21]` |
| 48–52 | `ProofBlock 76..111 [11,39,47,22]`; `TheoremItem 19..112 [5,6,7,33,48,23]`; `ItemList 0..112 [27,49]`; `CompilationUnit 0..112 [50]`; `Root 0..112 [0..23,51]` |

Resolver provenance is exactly one local public/exported theorem owner and
label, contribution 0, owner range `19..112`, label range `27..68`,
structural origin `[2,1]`, and normal recovery. There is no import,
proof-step label, citation, witness-name symbol, or added resolver handoff.
The existing private theorem-owner enrichment is sufficient.

The syntax-free lower composition is:

- Task 48 `2/1/0`: module context 0 and proof context 1, proof owner
  `76..111`, reserved binding 0 visible in lexical scope `[0]`, no
  proof-owned binding, and no diagnostic;
- Task 252 `6/5/0`: five surface roots at nodes `28/30/36/40/42` expand to
  six dense primary rows. Terms 0/1 are variable `x` at `28/70..71` and
  `30/74..75` in context 0. Term 2 is the parenthesized wrapper at
  `36/89..92`, spelling `( x )`, context 1, and no parent. Term 3 is its
  variable child at `34/90..91`, context 1, parent term 2. Terms 4/5 are
  variable `x` at `40/101..102` and `42/105..106` in context 1. Dense
  reference IDs `0/1/2/3/4` target terms `0/1/3/4/5`; all select binding 0
  with use ordinal 1 and lexical scopes `[]/[]/[0]/[0]/[0]`. Term 2 has no
  reference, and there is no numeric request;
- Task 256 `2/0/0/0/0/0/0/4/4`: equality nodes 32 and 44 target primary
  pairs `[0,1]` and `[4,5]`. Both witness wrapper term 2 and child term 3
  are excluded from every atomic edge and request;
- base statement `1/2/2/2/2`: theorem node/range `49/19..112` and
  conclusion `47/96..107` have source ordinals 0 and 2 and contexts 0 and
  1. Their input facts use references `[0,1]` and `[3,4]`.

The runner must represent the five extraction roots separately from the six
expected primary rows. It must not reuse root count as primary count or
derive the conclusion atomic start as `root_count - 2`; the frozen atomic
starts are `[0,4]`, and the input-fact reference starts are `[0,3]`. This is
a private Task-258 consumer adjustment, not a Task-252 or Task-256 defect.

The witness companion is exactly `1 witness / 0 names`. Witness 0 owns
owner 0, binding context 1, `Primary(2)`, take node/range `39/84..93`,
item node/range `38/89..92`, source ordinal 1, within-`take` ordinal 0,
token-normalized spelling `( x )`, kind `Unnamed`, normal recovery, and no
name. Combined source order is `[0,1,2]`. The wrapper is the witness target;
the inner variable remains only the Task-252 child/reference.

Typed ownership assigns `source.term.parenthesized` to node 36,
`source.term.variable-reference` to node 34,
`source.statement-witness.item` to node 38,
`source.statement-witness.take` to node 39,
`source.formula.atomic.equality` to nodes 32/44,
`source.statement.conclusion` to node 47, and
`source.statement.theorem` to node 49. Nodes 35 and 37 remain unowned
surface wrappers. The wrapper/child subtree is inside take/proof/theorem and
disjoint from both equality subtrees and the conclusion. Tasks 253–255
receive no application, structure, selector, update, set, choice, wrapper,
or cross-family edge.

No public type, variant, field, error, table, accessor, producer, installer,
route, or detail key is added. Existing
`SourceBindingContextHandoff`, `SourceTypeApplicationHandoff`,
`SourceAttributeHandoff`, `SourceEvidenceHandoff`,
`SourcePrimaryTermHandoff`, and `SourceFunctorApplicationHandoff` are the
public Task-248–253 lower families. Task 254's
`SourceStructureHandoff` is the next excluded family. B3M2B1 reuses the
Task-248 context and Task-252 projection; Tasks 249–251 and 253–254 remain
empty or excluded. Existing
`SourcePrimaryTermKind::Parenthesized`, `SourcePrimaryTermInput::parent`,
reference tables, `SourceStatementWitnessTermTarget::Primary`, witness/name
tables, `SourceStatementWitnessProducer`,
`TypedAst::with_source_statement_witnesses`, and final ownership are
sufficient. Private base/witness selectors add only B3M2B1. Validation
precedence remains complete dependency/fingerprint/arena authentication,
aggregate cardinality, witness row 0, then empty name rows:
`DependencyMismatch` before `InvalidAggregate` before
`InvalidWitness { witness: 0 }`; `InvalidName` is unreachable.

The paired typed/final owner rejects base-only or witness-only installation,
B3/B3N/B3M1/B3M2A/B3M2B1 hybrids, Task-252 parent/child corruption,
reference remapping, every Task-248/253–257/other-258 family in either
order, stale fingerprints, and nonempty semantic/proof/goal tables.
Successful assembly clone-preserves the pair and leaves all semantic outputs
empty. B3M2A remains isolated by exact bytes, 49-versus-53-node arena,
term count/kind, numeric-request presence, fingerprints, and final
revalidation.

The checker test contract is exactly four compound tests:

1. `task258b3m2b1_exact_parenthesized_witness_api_debug_and_compatibility_are_stable`;
2. `task258b3m2b1_dependencies_parent_child_witness_precedence_and_all_nodes_fail_closed`;
3. `task258b3m2b1_paired_ownership_hybrids_and_all_family_orders_are_atomic`;
4. `task258b3m2b1_final_clone_revalidation_and_semantic_deferrals_are_stable`.

Checker test 2 must independently reject a new reference row on wrapper
term 2 while the child reference remains, removal/remapping/duplication of
child reference 2, and Task-256 edge/request contamination by term 2 and by
term 3. These are separate mutations; no one rejection stands in for
another.

The runner test contract is exactly five compound tests:

1. `task258b3m2b1_real_frontend_freezes_parenthesized_witness_contract`;
2. `task258b3m2b1_validation_precedence_mutation_and_replay_fail_closed`;
3. `task258b3m2b1_selector_and_byte_subtree_near_misses_are_exact`;
4. `task258b3m2b1_family_and_active_route_isolation_is_atomic_in_both_orders`;
5. `task258b3m2b1_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`.

Runner test 2 attempts each wrapper-reference, child-reference, and
term-2/term-3 Task-256 contamination mutation through real route
construction. When a malformed row cannot form a valid Task-252 or Task-256
handoff, that owning public producer rejects first; the paired statement
consumer is exercised whenever lower construction succeeds. Runner test 3
additionally proves that selector and subtree near misses cannot publish
partial wrapper/child ownership or a detached child reference.

Together they freeze exact source/resolver/lower/base/witness identity,
five-root/six-primary separation, parent/child and reference ownership,
dependency/aggregate/witness precedence, all 53 node and byte/subtree
mutations, B3M2A and Tasks 253–255 isolation, family/active order,
rollback/replay, debug compatibility, final cloning, and empty semantics.
Near misses include `x`, `101`, `(101)`, `((x))`, named/multiple
parenthesized witnesses, application/structure/selector/update/set/choice
terms, recovery, changed theorem shape, and existential/composite roots.
An authority-invalid `take it;` in this theorem proof is a near miss, not a
future promise.

The exact private detail projection remains `Some(Vec::new())`, requiring
the paired witness handoff, two base statements, lookup ordinals `1/1`, and
five reference-use ordinals `[1; 5]`. `None` remains a selector miss, and an
owned invalid output retains
`type_elaboration.checker.typed_ast_invalid`.

This documentation prerequisite changes no production/test source,
canonical specification, existing `.miz`, fixture, expectation, sidecar,
trace row/status/count, active route, test list, count, or hash. Fresh
baselines are checker/runner libraries `362/399`, checker module sizes
`15746/4660/7202/3156`, runner statement leaf/facade/root/test sizes
`4185/691/2505/8611`, and runner production 30 paths / 38,571 lines.
Checker raw/normalized test-list hashes are
`af5f3c7030167087367ebbf534b9ebde03fcfcb3b406dcacbd4eccd1841a25e7` /
`4b95f5557e65e4d9ec4e9df90f3f61e77318570a0498996a7929b29500f127d7`;
runner hashes are
`a9557e877ad59d5d5da47861f41beaecd2e6a28b9a7a090381bb966096ecea13` /
`88a2d2e70f04c7606a78630c42ae66b7506a15df2f0cd91b4dbb3945181ad847`.
Runner path/content hashes remain
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`30640df237d236a980cd0daf013996e3d37dc36fbabd0e9badadac8a0e57c4c2`.
Plan/type remain `419/387` and `253/241`; pass/fail `228/191`; active
parse/declaration/type/proof `101/5/198/1`; warnings/errors `23/0`; and
plan/parse/declaration/type/proof CLI hashes remain
`4cc13ea6bee6c1a6458d4a7d027a7eea685b711eda8410edafad8faa01809d54`,
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
`210055108c257ff65c6f45fb654c82e506653ec4617b68d111893bb3aa1da5a8`,
`f87a743b914d2d51d6b9a8dbcf3c8d93bbc1403b44907fd85123a1865f84edd5`,
and `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

Implementation projects exactly four checker and five runner tests, hence
libraries `366/404`; changed sizes and hashes must be measured rather than
treated as targets. This prerequisite resolves the two bounded
`design_drift` findings: the broad umbrella and the root/primary conflation.
Future private code is bounded `source_drift`, and future tests are
`test_gap`. There is no blocking `spec_gap`, unsafe test intent,
lower-stage defect, `source_undocumented_behavior`,
`test_expectation_drift`, or language/crate `boundary_violation`.
A review-only writer concurrently duplicating the B3M2B1 documentation was
an operational `boundary_violation`, not a `repo_metadata_conflict`; the
parent reconciled the task-owned documentation without changing repository
metadata.

The coverage audit changes follow-up ownership only.
`spec.en.checker.formula_statement.source_payloads` remains `deferred` with
`tests = []` and receives no credit. Exit requires synchronized EN/JA
documents, independent no-findings reviews, all protocol hard gates,
read-only quality at least 90/100, task-only staging, and one dedicated
documentation commit. Implementation may begin only after that commit and
a fresh parser/resolver/lower/count/hash preflight.

## Task 258B3M2B1 Implementation Result

The checker now recognizes one private syntax-free `Task258B3M2B1`
profile. It authenticates the complete 53-node arena, exact module/proof
binding contexts, Task-48 `2/1/0`, Task-252 `6/5/0` with parenthesized term
2 / child term 3 and dense references `0/1/2/3/4 -> 0/1/3/4/5`,
Task-256 pairs `[0,1]` / `[4,5]`, base `1/2/2/2/2`, and one unnamed
outer-term witness/no names with source partition `[0,1,2]`. The paired
typed/final path publishes both halves atomically; every byte, node,
parent/reference, resolver, dependency, subtree, family, replay, and
semantic-coexistence near miss fails without partial ownership.

All four checker and five runner compound tests pass. Libraries are
`366/404`; checker module sizes are `17569/4661/7203/3156`; runner
statement leaf/facade/root/test sizes are `4676/695/2508/9902`; runner
production is 30 paths / 39,069 lines. Checker raw/normalized test-list
hashes are
`0e43763c92ee171b18b5a2f80b92cd278b49ac9895d95410ca52ca787bcac3c8` /
`7685e21bc0d76bb8d824dd821e800707d251e8c025682ef69b2db798d6888e5d`;
runner hashes are
`a28c33e517d8efdd635e23e6f2273c29b966aa6102efb321eed73335ab11483c` /
`f8e8dc6ef605cbd8f8ad722983793434339b3cad21bf53703ab6c21f0b8742a5`.
Runner path/content hashes are
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`04bf563fcc99ccbc3b789a8596d953ade05453b2267639ef0ce3d8d54cbd6b45`.
All five CLI counts and hashes remain unchanged.

No canonical artifact, fixture, expectation, sidecar, trace status/count,
active route, public API, binding, or semantic/proof/goal owner changed.
The formula-statement row remains `deferred`, `tests = []`, with no credit.
The bounded B3M2B1 `source_drift` and `test_gap` are closed; B3M2B2 is next
before B4. Public Task-252/256 fail-close remains stronger than the
statement boundary: malformed lower rows reject before paired consumption,
and no test-only seam or public API was added to bypass that invariant.

## Task 258B3M2B2A Frozen Nested-Parenthesized-Witness Slice

Fresh post-B3M2B1 inventory decomposes broad B3M2B2 into dependency-ordered
B3M2B2A and B3M2B2B. B3M2B2A owns only one unnamed, exactly two-level
parenthesized reserved-variable witness, `take ((x));`. It depends only on
the existing Task-252 nested-primary graph. B3M2B2B retains triple/deeper
parentheses, application, structure constructor/selector/update, set,
choice, compound, and every other authority-valid witness term. Its future
cross-family work must be decomposed in lower-owner order Task 253, then
Task 254, then Task 255 before B4/B5.

Canonical authority is `doc/spec/en/15.statements.md` §15.4.4, where an
unnamed example is any `term_expression`;
`doc/spec/en/13.term_expression.md` §§13.1, 13.1.3, 13.8.8, and 13.9,
where parenthesization is arbitrarily nestable, type preserving, and
FOL-transparent; and `doc/spec/en/16.theorems_and_proofs.md` §§16.3.3 and
16.7.3, which retain the later existential-introduction effect. Existing
`pass_parser_simple_statements_001.miz` authenticates unnamed `take`.
`pass_parser_primary_terms_001.miz`, Task-252's
`task252_nested_parentheses_exclude_mixed_subtrees_and_keep_siblings`, and
the covered `source_primary_term_payload` trace row authenticate nested
primary transport. No existing source, expectation, or trace metadata
changes or receives formula-statement credit.

The exact future corpus-dormant consumer is this final-LF 121-byte source,
SHA-256
`35396db1f7e22abfbe94861709b2ab9bca38d4464712dfbce114533d2ab4d71d`:

```mizar
reserve x for set;
theorem FormulaStatementNestedParenthesizedWitnessSmoke: x = x proof
  take ((x));
  thus x = x;
end;
```

The equality goal deliberately does not authorize existential introduction,
so this source must not become an active accepted corpus case. A fresh
frontend run yields zero diagnostics, 57 unrecovered nodes, root 56, and 26
tokens:
`0:reserve@0..7, 1:x@8..9, 2:for@10..13, 3:set@14..17,
4:;@17..18, 5:theorem@19..26,
6:FormulaStatementNestedParenthesizedWitnessSmoke@27..74,
7::@74..75, 8:x@76..77, 9:=@78..79, 10:x@80..81,
11:proof@82..87, 12:take@90..94, 13:(@95..96, 14:(@96..97,
15:x@97..98, 16:)@98..99, 17:)@99..100, 18:;@100..101,
19:thus@104..108, 20:x@109..110, 21:=@111..112,
22:x@113..114, 23:;@114..115, 24:end@116..119,
25:;@119..120`; every token has no child. Structural nodes are:

| IDs | Exact kind, range, ordered children |
| --- | --- |
| 26–29 | `TypeHead 14..17 [3]`; `TypeExpression 14..17 [26]`; `ReserveSegment 8..17 [1,2,27]`; `ReserveItem 0..18 [0,28,4]` |
| 30–35 | `TermReference 76..77 [8]`; `TermExpression 76..77 [30]`; `TermReference 80..81 [10]`; `TermExpression 80..81 [32]`; `BuiltinPredicateApplication 76..81 [31,9,33]`; `FormulaExpression 76..81 [34]` |
| 36–43 | `TermReference 97..98 [15]`; `TermExpression 97..98 [36]`; `ParenthesizedTerm 96..99 [14,37,16]`; `TermExpression 96..99 [38]`; `ParenthesizedTerm 95..100 [13,39,17]`; `TermExpression 95..100 [40]`; `Witness 95..100 [41]`; `TakeStatement 90..101 [12,42,18]` |
| 44–51 | `TermReference 109..110 [20]`; `TermExpression 109..110 [44]`; `TermReference 113..114 [22]`; `TermExpression 113..114 [46]`; `BuiltinPredicateApplication 109..114 [45,21,47]`; `FormulaExpression 109..114 [48]`; `Proposition 109..114 [49]`; `ConclusionStatement 104..115 [19,50,23]` |
| 52–56 | `ProofBlock 82..119 [11,43,51,24]`; `TheoremItem 19..120 [5,6,7,35,52,25]`; `ItemList 0..120 [29,53]`; `CompilationUnit 0..120 [54]`; `Root 0..120 [0..25,55]` |

Resolver provenance is exactly one local public/exported theorem owner and
label, contribution 0, owner range `19..120`, label range `27..74`,
structural origin `[2,1]`, and normal recovery. There is no import,
proof-step label, citation, witness-name symbol, or new resolver handoff.
The distinct theorem label is mandatory: the existing B3M2B1 same-label
mutation from `(x)` to `((x))` remains a selector near miss.

The syntax-free composition is:

- Task 48 `2/1/0`: module context 0 and proof context 1, proof owner
  `82..119`, reserved binding 0 visible in lexical scope `[0]`, no
  proof-owned binding, and no diagnostic;
- Task 252 `7/5/0`: five surface roots at nodes `30/32/40/44/46` expand
  to seven dense primary rows. Terms 0/1 are variable `x` at
  `30/76..77` and `32/80..81` in context 0. Term 2 is outer
  parenthesized `40/95..100`, spelling `( ( x ) )`, with no parent.
  Term 3 is inner parenthesized `38/96..99`, spelling `( x )`, parent
  term 2. Term 4 is variable `36/97..98`, parent term 3. Terms 5/6 are
  variable `44/109..110` and `46/113..114`. Terms 2–6 are in context 1.
  Dense references
  `0/1/2/3/4` target terms `0/1/4/5/6`; all select binding 0 at use
  ordinal 1, with lexical scopes `[]/[]/[0]/[0]/[0]`. Terms 2/3 have no
  reference, and there is no numeric request;
- Task 256 `2/0/0/0/0/0/0/4/4`: equality nodes 34 and 48 target
  primary pairs `[0,1]` and `[5,6]`. The complete witness chain
  `2 -> 3 -> 4` is excluded from every atomic edge and request;
- base statement `1/2/2/2/2`: theorem node/range `53/19..120` and
  conclusion `51/104..115` have source ordinals 0 and 2 and contexts 0
  and 1. Their input facts use references `[0,1]` and `[3,4]`.

The five extraction roots must remain distinct from seven primary rows.
Frozen atomic starts are `[0,5]`; input-fact reference starts are `[0,3]`.
The witness companion is exactly `1 witness / 0 names`: witness 0 owns
owner 0, context 1, `Primary(2)`, take `43/90..101`, item
`42/95..100`, source ordinal 1, witness ordinal 0, normalized spelling
`( ( x ) )`, normal recovery, and no name. Combined source order is
`[0,1,2]`.

Typed ownership assigns `source.term.variable-reference` to node 36,
`source.term.parenthesized` to nodes 38/40,
`source.statement-witness.item` to node 42,
`source.statement-witness.take` to node 43,
`source.formula.atomic.equality` to nodes 34/48,
`source.statement.conclusion` to node 51, and
`source.statement.theorem` to node 53. `TermExpression` and
`FormulaExpression` surface wrappers remain unowned. Tasks 249–251,
253–255, and 257 receive no row or ownership.

No public type, variant, field, error, table, accessor, producer, installer,
route, detail key, or debug grammar is added. Existing Task-248/252/256/base
and witness/name tables, `SourcePrimaryTermKind::Parenthesized`, parent
links, `Primary(2)` witness targets, and paired typed/final ownership are
sufficient. Private profiles and the exact selector alone add B3M2B2A.
Dependency/fingerprint/arena validation precedes aggregate cardinality,
witness row 0, and empty name rows. The paired owner rejects standalone,
hybrid, stale, parent-chain-corrupt, reference-corrupt, cross-family, or
semantic-coexisting states without partial publication.

The checker test contract is exactly four compound tests:

1. `task258b3m2b2a_exact_nested_parenthesized_witness_api_debug_and_compatibility_are_stable`;
2. `task258b3m2b2a_dependencies_parent_chain_witness_precedence_and_all_nodes_fail_closed`;
3. `task258b3m2b2a_paired_ownership_hybrids_and_all_family_orders_are_atomic`;
4. `task258b3m2b2a_final_clone_revalidation_and_semantic_deferrals_are_stable`.

Checker test 2 independently adds a new reference row on outer wrapper term
2 while the valid leaf reference 2 still targets term 4, then separately
adds a new reference row on inner wrapper term 3 while that valid leaf row
still remains. It also independently removes, remaps, duplicates, or
detaches each `2 -> 3 -> 4` parent/reference association and contaminates
Task-256 edge/request ownership separately with terms 2, 3, and 4. Checker
test 3 covers prior statement profiles and Tasks 253–255 in both
installation orders without weakening their own public fail-close.

The runner test contract is exactly five compound tests:

1. `task258b3m2b2a_real_frontend_freezes_nested_parenthesized_witness_contract`;
2. `task258b3m2b2a_validation_precedence_mutation_and_replay_fail_closed`;
3. `task258b3m2b2a_selector_and_byte_subtree_near_misses_are_exact`;
4. `task258b3m2b2a_family_and_active_route_isolation_is_atomic_in_both_orders`;
5. `task258b3m2b2a_typed_final_clone_debug_rollback_and_empty_semantics_are_stable`.

Together they freeze all 121 bytes and 57 nodes, resolver provenance,
five-root/seven-primary separation, the complete two-parent chain,
reference ownership, subtree exclusion, all independent corruptions,
B3M2B1 same-label and exact-source isolation, every earlier family and
active order, rollback/replay, debug stability, final cloning, and empty
semantics. Near misses include `x`, `(x)`, `(((x)))`, `((101))`, named or
multiple nested witnesses, application/structure/selector/update/set/choice
terms, recovery, changed theorem or label, and composite/existential roots.
The theorem-proof `take it;` remains authority-invalid.

Runner test 2 attempts both extra-wrapper-reference mutations while the
valid term-4 leaf reference remains, plus every parent/leaf-reference and
term-2/3/4 Task-256 mutation, through real route construction. When an
invalid row cannot form a Task-252/256 handoff, that owning lower producer
rejects first; every constructible handoff reaches the paired statement
consumer.

The private detail projection remains `Some(Vec::new())`, requiring paired
base/witness ownership, two statements, lookup ordinals `1/1`, and
reference-use ordinals `[1; 5]`. `None` is a selector miss; an owned invalid
output retains `type_elaboration.checker.typed_ast_invalid`.
Task 269 remains a no-op. Task 272 retains witness typing, existential-goal
matching, substitution, remaining-goal construction, and proof acceptance;
all formula truth/fact, Core/ControlFlow/VC, and goal outputs remain empty.

This documentation prerequisite changes no production/test source,
canonical specification, existing `.miz`, fixture, expectation, sidecar,
trace row/status/count, active route, test list, count, or hash. Baselines
are checker/runner libraries `366/404`, checker sizes
`17569/4661/7203/3156`, runner statement leaf/facade/root/test sizes
`4676/695/2508/9902`, and runner production 30 paths / 39,069 lines.
Checker raw/normalized test-list hashes are
`0e43763c92ee171b18b5a2f80b92cd278b49ac9895d95410ca52ca787bcac3c8` /
`7685e21bc0d76bb8d824dd821e800707d251e8c025682ef69b2db798d6888e5d`;
runner hashes are
`a28c33e517d8efdd635e23e6f2273c29b966aa6102efb321eed73335ab11483c` /
`f8e8dc6ef605cbd8f8ad722983793434339b3cad21bf53703ab6c21f0b8742a5`.
Runner path/content hashes remain
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275` /
`04bf563fcc99ccbc3b789a8596d953ade05453b2267639ef0ce3d8d54cbd6b45`.
Plan/type remain `419/387` and `253/241`; pass/fail `228/191`; active
parse/declaration/type/proof `101/5/198/1`; warnings/errors `23/0`; and all
five CLI hashes remain unchanged.

Implementation projects exactly four checker and five runner tests, hence
libraries `370/409`; changed sizes and hashes must be measured. This
prerequisite closes the broad-umbrella `design_drift`; future code is
bounded `source_drift` and future tests are `test_gap`. There is no blocking
`spec_gap`, unsafe test intent, lower-stage defect,
`source_undocumented_behavior`, `test_expectation_drift`, or language/crate
`boundary_violation`. The historical external-origin movement remains a
report-only `repo_metadata_conflict`; the exact task and commit base remain
unambiguous.

The coverage audit changes follow-up ownership only.
`spec.en.checker.formula_statement.source_payloads` remains `deferred` with
`tests = []` and no credit. Exit requires synchronized EN/JA documents,
independent no-findings reviews, all hard gates, read-only quality at least
90/100, task-only staging, and one dedicated documentation commit.
Implementation may begin only after that commit and a fresh
parser/resolver/lower/count/hash preflight.

## Task 258B3M2B2A Implementation Result

The checker now recognizes one private `Task258B3M2B2A` profile. It
authenticates the exact 57-node arena, Task-48 `2/1/0`, Task-252 `7/5/0`
with parent chain `2 -> 3 -> 4` and references to `0/1/4/5/6`,
Task-256 equality pairs `[0,1]` / `[5,6]`, base `1/2/2/2/2`, and one
unnamed outer-term witness/no names with source partition `[0,1,2]`.
Dependency/fingerprint/arena validation precedes aggregate, witness 0, and
empty-name validation. Wrapper references and independent Task-256
contamination by terms 2, 3, or 4 fail closed.

All four checker and five runner compound tests pass. Libraries are
`370/409`; checker sizes are `19571/4662/7204/3156`; runner statement
sizes are `5188/699/2513/11234`; production is 30 paths / 39,590 lines.
Raw/normalized test-list hashes are
`18cae89ddf8a5a21cca3741fd2c3e19a6d23b53c9ffe8e482dca63310445245c` /
`a1c328b0a1fef79df97b3fc5cb353dac8ac1ecc7a8477f27c11124de9f390d84`
and
`7e76d1de5b01b7a6fbe7fa8c88a8bffc3f957ec35a7d8a27cd456031d70d9299` /
`8eae5a5a084f0feeaba678c3b0aa11f47956c7f98946d7205b82984a8b5eb23a`.
The production path hash remains
`98f3b264a59fed5b08c3e8f20e7ca58ff54efaa154eab16a7572a69ce923f275`;
content is
`291da8a26e90f75e7f54e221314c1fcb9ebba375c238a07b02a161f7af6dfe66`.

No canonical artifact, fixture, expectation, sidecar, trace status/count,
active route, public API, binding, or semantic/proof/goal owner changed.
The formula-statement row remains `deferred`, `tests = []`, with no credit.
B3M2B2B remains the next witness-term slice before B4.

## Task 258B3M2B2B Lower-Owner Decomposition

Broad B3M2B2B first depends on Task 258B3M2B2B1P, a private Task-253
proof-context reuse seam. The seam is a separate lower-stage logical task:
it does not add `Application` to the witness target, install statement
tables, or permit application/statement coexistence. After B1P is committed
and freshly inventoried, B3M2B2B1A may freeze the exact imported-infix
application-witness contract. Other Task-253 forms, Task-254 structure
constructor/selector/update forms, Task-255 set/choice/qualification forms,
and remaining compound terms stay deferred.

## Task 258B3M2B2B1P Dependency Completion

The private Task-253 proof-context prerequisite is complete and verified,
but no statement or witness table is installed. Task 258 behavior and the
application-to-witness ownership edge remain absent. Fresh inventory may
now freeze B3M2B2B1A as a separate documentation task; all other
Task-253/254/255 and compound shapes remain deferred.

## Task 258B3M2B2B1A Application-Witness Ownership

B1A consumes exactly the B1P `take 1 ++ 2;` source. The syntax-free witness
row is:

| Field | Value |
|---|---|
| owner / binding context | theorem owner 0 / proof context 1 |
| source ordering | witness source ordinal 1, witness ordinal 0 |
| take occurrence | node 49, range `111..123` |
| witness occurrence | witness-container node 48, range/spelling `116..122` / `1 ++ 2` |
| target | `SourceStatementWitnessTermTarget::Application(SourceFunctorApplicationId(0))` |
| kind / recovery / name | `Unnamed` / `Normal` / `None` |

Parser containment is node 49 -> 48 -> 47 -> 46, but ownership crosses only
from witness row 0 to Task-253 application row 0. Node 48 is the owned witness
site, node 47 is an unowned transparent traversal node, and node 46 is the
application target. Node 47 does not create a Task-253 wrapper or Task-252 primary.
Task 252 owns the two numeral arguments as primaries 2/3; Task 253 owns the
application, head, candidate, arguments, and requests; Task 256 excludes the
whole subtree. The witness consumer must not copy or retarget any lower row.

`SourceStatementWitnessTermTarget`, already `#[non_exhaustive]`, adds the
`Application(SourceFunctorApplicationId)` variant. The immutable handoff adds
`application_fingerprint: Option<String>` and the read-only
`application_fingerprint()` accessor. Legacy primary-target handoffs retain
`None`. `SourceStatementWitnessProducer::build(...)` remains unchanged for
legacy callers and rejects an application target without the new dependency.
`build_with_application(...)` takes the same input and base/primary/arena
dependencies plus the exact `SourceFunctorApplicationHandoff`; only B1A may
produce `Some(application.debug_text())`.

The additive public signatures are frozen as:

```rust
pub fn application_fingerprint(&self) -> Option<&str>;

pub fn build_with_application(
    input: SourceStatementWitnessHandoffInput,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    application: &SourceFunctorApplicationHandoff,
    arena: &TypedArena,
) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError>;

pub fn with_source_application_statement_witnesses(
    self,
    application: SourceFunctorApplicationHandoff,
    statements: SourceStatementHandoff,
    witnesses: SourceStatementWitnessHandoff,
) -> Result<Self, TypedAstError>;
```

Installation accepts exactly two dependency shapes:

1. legacy statement-witness profiles with no source application and no
   application fingerprint;
2. B1A with the exact source application, a matching fingerprint, and witness
   target `Application(0)`.

Every `Some/None`, missing/orphan, wrong-ID, wrong-context, wrong-range,
stale-fingerprint, substituted-candidate, or cross-profile hybrid rejects.
Dependency/source/module/fingerprint and all lower handoffs validate before
aggregate `1/0`, witness row 0, and the empty name table. The existing
`source-statement-witness-debug-v1` bytes remain identical for every legacy
profile. A B1A rendering alone inserts
`application-fingerprint: Some(...)` after the primary fingerprint and uses
`term=application#0`.

The sole TypedAst entry point is
`with_source_application_statement_witnesses(application, statements,
witnesses)`. It publishes all three handoffs only after complete validation.
`with_source_application` and `with_source_statement_witnesses` continue to
reject the opposite family, including partial B1A installation.
`ResolvedTypedAst` revalidates and clone-preserves the same exact bundle.
Although Task 256 sees the Task-253 handoff during combined revalidation, its
two equality formulas use only primaries `[0,1]` and `[4,5]` and retain no
application fingerprint. Only the witness handoff consumes Task 253.

The contract ends at source provenance and ownership. Witness type checking,
goal matching, existential substitution, remaining-goal construction,
formula truth, proof acceptance, Core/ControlFlow/VC, and all diagnostic or
active-route behavior remain Task 272 or later. Other application forms,
parentheses around the application, Tasks 254/255 terms, named/multiple
witnesses, and broader proof shapes remain later B1B+ slices.

The exact checker tests are:

1. `task258b3m2b2b1a_exact_application_witness_api_debug_and_legacy_compatibility_are_stable`;
2. `task258b3m2b2b1a_dependencies_application_witness_precedence_and_all_nodes_fail_closed`;
3. `task258b3m2b2b1a_combined_ownership_hybrids_and_all_family_orders_are_atomic`;
4. `task258b3m2b2b1a_final_clone_revalidation_and_semantic_deferrals_are_stable`.

## Task 258B3M2B2B1A Implementation Result

`SourceStatementWitnessTermTarget::Application(SourceFunctorApplicationId)`
and the optional application fingerprint are implemented additively.
`SourceStatementWitnessProducer::build_with_application` accepts only the
exact B1A statement/application pair; the legacy builder still accepts only
application-free profiles and preserves their debug bytes. The exact
143-byte/63-node profile maps witness node 48 through traversal node 47 to
the Task-253 application at node 46 without adding a wrapper or primary
duplicate. The producer authenticates the real imported symbol, local/FQN
lookups and complete contribution/path/export provenance, Task-252 arguments
and requests, Task-256 equality-only exclusion, base `1/2/2/2/2`, one
unnamed witness/no names, and both dependency fingerprints.

The four named checker tests above and five corresponding runner tests pass
their exhaustive byte/subtree/provenance/dependency/precedence/family/
rollback/replay/clone matrices. Libraries are `374/416`; checker modules are
`21664/4742/7224/3156`. No semantic, proof, type, substitution, or goal
meaning is inferred, and no canonical, fixture, active, expectation,
sidecar, or trace artifact changed.

## Task 258B3M2B2B1B1P Lower-Owner Deferral

Fresh inventory selects `take (1 ++ 2);` as the next B1B1 statement shape,
but B1B1P freezes only its missing runner-private wrapped Task-253 reuse seam.
The 158-byte/67-node motivating source has Task-252 `6/4/2` and Task-253
`1/1/1/2/2`; wrapper node 50 owns `129..137`, inner application node 48 owns
`130..136`, and the future witness will continue to target
`Application(0)`. No new statement/witness profile, checker API, atomic
installer, test, or semantic behavior is authorized by B1B1P.

After the documentation and lower-seam implementation commits, B1B1 must be
fresh-inventoried and frozen separately. Parenthesized applications with
other operators/operands, nested wrappers/applications, named or multiple
witnesses, Task-254/255 witness terms, goal matching, type obligations,
substitution, and proof acceptance remain deferred.

## Task 258B3M2B2B1B1P Dependency Completion

The runner-private wrapped Task-253 prerequisite is complete and passes its
two exact tests without publishing a statement or witness. The future B1B1
consumer may now be fresh-inventoried against application 0 and wrapper
containment, but no B1B1 selector, checker row, installer, or semantic behavior
is inferred by this completion.

## Task 258B3M2B2B1B1 Wrapped Application-Witness Ownership

B1B1 consumes only the final-LF 158-byte/67-node source containing
`take (1 ++ 2);`. It reuses the B1B1P Task-253 application/wrapper handoff
and the existing B1A public application-witness schema. It adds no public
type, method, table, or fingerprint grammar.

The exact containment path is `take 53 -> witness 52 -> unowned 51 ->
wrapper 50 -> unowned 49 -> application 48`. Task 258 owns take/witness
nodes 53/52 and the directed `Witness(0) -> Application(0)` edge. Task 253
continues to own wrapper/application 50/48; wrapper 0 is containment
metadata, never the target. Task 252 owns numeral primaries 2/3, and Task
256 excludes the entire subtree.

Owner 0 is the exact local theorem
`FormulaStatementParenthesizedApplicationWitnessSmoke`: site/range
`63/48..157`, label `56..108`, contribution 0, `LocalSource` anchor
`29..47`, checked origin `48..157` with structural path `[2,1]`,
public/exported/normal. Resolver symbol, definition, label, contribution,
and checked owner must agree. The resolver has one import and no
witness-name symbol.

Base rows are exact:

| Row | Frozen value |
| --- | --- |
| statement 0 | owner/context `0/0`; `Atomic(0)`; site/range `63/48..157`; ordinal 0; `TheoremProposition`; normalized complete-theorem spelling |
| statement 1 | owner/context `0/1`; `Atomic(1)`; site/range `61/141..152`; ordinal 2; `Conclusion`; `thus x = x ;` |
| context 0/1 | statements 0/1; binding contexts 0/1; ranges `48..157` / `141..152`; visible binding `[0]` |
| input fact 0/1 | corresponding statement/context; ordinal 0; `ReservedTypeGuard`; binding 0; refs `[0,1] -> Primary(0/1)` / `[2,3] -> Primary(4/5)` |
| candidate fact 0/1 | corresponding statement/context; ordinal 0; `UnverifiedProposition`; `Atomic(0/1)` |

Witness 0 is owner/context `0/1`, source/witness ordinal `1/0`,
normal/unnamed/no name. Take is site/range `53/124..138`, children
`[17,52,23]`. The witness is site/range `52/129..137`, normalized spelling
`( 1 ++ 2 )`, child `[51]`, and target `Application(0)`. The theorem
contains take/witness, the conclusion does not, and only nodes 53/52 use
statement-witness ownership kinds.

Lower tables remain Task-48 `2/1/0`, Task-252 `6/4/2`, wrapped Task-253
`1/1/1/2/2`, and equality-only Task-256 `2/0/0/0/0/0/0/4/4`. Application
0 remains node/range/context `48/130..136/1`; wrapper 0 remains
`50/129..137/1`; head is `20/132..134/++`; arguments are `Primary(2/3)`;
candidate provenance is the exact imported
`parser.type_fixtures::++#12`, contribution 2, origin `7..27`, path `[12]`,
public/exported/no signature.

The implementation adds one explicit crate-private B1B1 statement and
witness profile. It reuses
`SourceStatementWitnessTermTarget::Application`,
`SourceStatementWitnessProducer::build_with_application`, and
`TypedAst::with_source_application_statement_witnesses`; B1A is not
broadened. Validation proceeds through selector/owner, lower dependencies
and fingerprints, aggregate, all base rows, witness, empty names, atomic
typed install, and final revalidation. Failure is atomic and clean replay
must remain byte-identical.

The four exact checker and five exact runner tests named in the crate plan
cover all 158 bytes, all fields of 67 nodes plus root, five resolver
substitutions, the eight-entry reparse matrix, B1A compatibility, family and
active-route isolation, validation precedence, rollback/replay/clone, and
empty downstream semantics. Type checking, goal matching, substitution,
proof acceptance, Task-254/255 forms, and other application/witness shapes
remain deferred.

## Task 258B3M2B2B3A Implemented Source-Statement Witness Closure

The source-statement owner now provides exactly `SetTerm(SourceSetTermId)`,
the optional set fingerprint/getter, `build_with_set_term`, and the
crate-private set-aware installation seam. The exact one-witness/zero-name
profile authenticates the resolver label, Tasks 48/252/255/256/258, all
`57` nodes/root, the ownership partition, and the sole witness-to-set edge.
The four checker and five runner tests cover all frozen mutations,
fingerprint tuples, near misses, family orders, rollback/replay, final clone,
and empty semantics. Specification, test-sufficiency, and implementation
reviews report **NO FINDINGS**. All semantic deferrals remain. The second
source/documentation consistency repeat and final documentation/boundary
reread also report **NO FINDINGS**; parent final verification listed in the
crate plans passes, including exact `39`-file scope. Independent final
read-only quality review reports **NO FINDINGS**. All nine hard gates PASS
with no score cap; the valid score is `98/100`
(`20/20/15/14/10/10/5/4`). The stated semantic and coverage deferrals
remain unchanged as residual risk. Only the dedicated implementation
commit, post-commit invariant verification, and fresh next-task inventory
remain pending.

## Task 258B3M2B2B1B1 Implementation Result

The private B1B1 profile now authenticates the exact 158-byte/67-node owner,
base statements, one unnamed application witness, lower fingerprints, and
wrapper containment. All four checker and five runner tests pass, including
mutation precedence, B1A/family/active isolation, rollback/replay, and final
clone checks. This closes the bounded `source_drift` and `test_gap`; no
semantic, proof, goal, or type-substitution behavior was added.

## Task 258B3M2B2B2P Statement-Owner Deferral

B2P publishes no Task-258 owner, statement, context, input/candidate fact,
witness, witness name, typed coexistence row, or final-statement profile.
The future B2A contract alone may add
`SourceStatementWitness(0) -> SourceStructureTerm(0)` after this lower seam
is implemented and fresh-inventoried. Take node 62 and witness node 61 are
therefore unowned by B2P, while transparent node 60 remains excluded and
constructor node 59 stays Task-254-owned. Witness obligation, substitution,
proof/fact acceptance, and goal discharge remain deferred.

## Task 258B3M2B2B2P Statement-Owner Result

B2P is implemented without publishing a Task-258 owner, statement, context,
fact, witness, witness name, coexistence row, or final-statement profile.
Take 62, witness 61, and transparent term 60 remain unowned by B2P.
Fresh inventory therefore keeps B2A as the sole next owner of the directed
`SourceStatementWitness(0) -> SourceStructureTerm(0)` edge.

## Task 258B3M2B2B2A Frozen Structure-Witness Ownership

B2A composes the exact Task-258 theorem owner/base rows and one unnamed
witness for the 172-byte source. Base counts are `1/2/2/2/2`; witness/name
counts are `1/0`. Owner row 0 is theorem node 72 at `48..171`, spelling
`FormulaStatementStructureConstructorWitnessSmoke`, role/status
`Theorem/Unmodified`, and normal recovery. Statement 0 uses owner/context
`0/0`, `Atomic(0)`, node/range `72/48..171`, ordinal 0,
`TheoremProposition`, normal recovery, and literal normalized spelling
`theorem FormulaStatementStructureConstructorWitnessSmoke : x = x proof take TypeCaseStruct ( x : 1 , y : 2 ) ; thus x = x ; end ;`.
Statement 1 uses owner/context `0/1`, `Atomic(1)`, node/range
`70/155..166`, ordinal 2, `Conclusion`, normal recovery, and literal
spelling `thus x = x ;`. Contexts 0/1 use statements/binding contexts 0/1,
the corresponding statement ranges, and visible binding `[0]`. Input facts
0/1 use the corresponding statement/context, ordinal 0,
`ReservedTypeGuard`, binding 0, and exact reference uses
`[0,1]` / `[2,3]`; candidate facts use ordinal 0,
`UnverifiedProposition`, and target `Atomic(0/1)`.

Witness 0 is context 1, ordinal `1/0`, take `62/120..152`, item
`61/125..151`, normalized spelling
`TypeCaseStruct ( x : 1 , y : 2 )`, unnamed/no name, and targets only
`Structure(0)`. The base transaction owns theorem/conclusion rows 72/70;
the B2A extension owns take/witness 62/61 and the directed edge.

The frozen additive schema is
`SourceStatementWitnessTermTarget::Structure(SourceStructureTermId)`,
`structure_fingerprint(&self) -> Option<&str>`, and the exact public
signatures:

```rust
pub fn build_with_structure(
    input: SourceStatementWitnessHandoffInput,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    structure: &SourceStructureHandoff,
    arena: &TypedArena,
) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError>;

pub fn with_source_structure_statement_witnesses(
    self,
    structure: SourceStructureHandoff,
    statements: SourceStatementHandoff,
    witnesses: SourceStatementWitnessHandoff,
) -> Result<Self, TypedAstError>;
```

Legacy primary profiles use fingerprint pair `(None, None)`, application
profiles `(Some, None)`, B2A `(None, Some)`, and `(Some, Some)` is invalid.
The conditional structure fingerprint debug line follows the existing
conditional application position; only B2A renders `term=structure#0`.
Legacy primary/application builders, fingerprints, debug bytes, and
installers remain exact. Task 256 has no `Structure` edge and retains
`structure_fingerprint() == None`; only the combined typed/final paths
reauthenticate it with `Some(&structure)`.

The contract ends at source provenance/ownership. No existential matching,
type obligation, substitution, remaining goal, formula truth, proof fact,
or theorem acceptance is published.

## Task 258B3M2B2B2A Structure-Witness Result

The syntax-free producer now enumerates the exact B2A profile and publishes
one unnamed `Structure(0)` witness over the authenticated Task-258 base.
`build_with_structure` stores the exact structure fingerprint while
legacy/application pairs remain `(None, None)` / `(Some, None)` and hybrids
reject. Debug rendering emits the conditional structure fingerprint and
`term=structure#0` only for B2A.

Four checker tests validate the public API, base/witness rows, dependency
substitutions, fingerprint isolation, atomic installation, and final clone.
`source_statement.rs` is 27,194 lines.
No existential matching, obligation, goal, proof fact, theorem acceptance,
active route, or coverage credit is introduced.

## Task 258B3M2B2B2BP Lower-Prerequisite Exclusion

B2BP is not a statement-witness profile. Its 171-byte selector source
motivates only a private Task-254 proof-context reuse seam, and Task 258 owns
no theorem, statement, context, fact, witness, or name row. No
`SourceStatementWitnessTermTarget`, fingerprint, builder, TypedAst
installer, final clone rule, public API, or debug grammar changes.

The later B2B consumer may own the exact witness-to-selector edge only after
B2BP is implemented and committed separately. B2C
functional-update/`FieldUpdate` and all selector identity/type, proof, goal,
and theorem semantics remain deferred.

The B2BP private lower seam is now implemented after its frozen prerequisite.
This does not install a statement witness or alter any checker API. Fresh
dependency inventory may therefore freeze B2B as the next separate logical
task; B2C and all semantic/proof/goal behavior remain deferred.

## Task 258B3M2B2B2B Frozen Structure-Selector Witness Contract

The exact 171-byte, final-LF source adds one syntax/provenance-only witness
edge to the existing Task-258 base table. The base remains
`1/2/2/2/2`: theorem site node `75`, conclusion statement node `73`,
context `1` seeing context `0`, input-fact references `[0,1]` and `[2,3]`,
and candidates at atomic statements `0/1`. The witness table is exactly
`1/0`: `take` node `65` owns witness expression node `64`, preserves the
selector spelling, has no name, and targets Task-254 `Structure(0)` only.
The selector base `Structure(1)` is a lower-stage child, not a second witness
target.

`SourceStatementProducer`,
`SourceStatementWitnessProducer::build_with_structure`, the
combined TypedAst installer, and final-AST clone path are the exact
consumers. The implementation may change only `source_statement.rs`,
`typed_ast.rs`, and `resolved_typed_ast.rs` in the checker. It adds no public
API or debug grammar. Task 252 owns nodes `47/49/55/58/66/68`; Task 254 owns
`62/61/29/20/24`; Task 256 owns `51/70`; Task 258 owns base nodes `75/73`;
B2B owns only nodes `65/64` and the witness-to-`Structure(0)` edge. Formula
containers `52/71` and private numeric roots `56/59` remain unowned.

The four required checker tests are:

- `task258b3m2b2b2b_exact_structure_selector_witness_api_debug_and_legacy_compatibility_are_stable`
- `task258b3m2b2b2b_dependencies_structure_selector_witness_precedence_and_all_nodes_fail_closed`
- `task258b3m2b2b2b_combined_ownership_hybrids_and_all_family_orders_are_atomic`
- `task258b3m2b2b2b_final_clone_revalidation_and_semantic_deferrals_are_stable`

Canonical `take` semantics would require an existential goal, but this smoke
source has conclusion `x = x`; therefore this task authenticates syntax and
provenance only. Existential matching, proof facts, obligations, goals,
theorem acceptance, selector identity/type/result, B2C functional update,
and `FieldUpdate` remain deferred.

## Task 258B3M2B2B2B Implementation Result

The source-statement producer now authenticates the exact 171-byte/79-node
B2B profile and publishes base `1/2/2/2/2` plus one unnamed witness/no
names. Witness 0 owns take/item nodes `65/64` and targets only Task-254
selector `Structure(0)`; constructor `Structure(1)`, members, roots,
primaries, applications, and transparent containers are not witness
targets. Task-256 nodes `51/70` remain owned and containers `52/71` remain
unowned.

The existing structure-aware builder, fingerprint, and atomic installer are
reused without public API growth. The four frozen checker tests pass,
including exact dependency precedence, all-node failure, B2A/B2B hybrid
rollback, final-clone revalidation, and empty semantic deferrals.
`source_statement.rs` is 29,941 lines. No selector meaning, proof/goal
effect, theorem acceptance, B2C update/`FieldUpdate`, corpus active-route
status, or trace credit was added.

## Task 258B3M2B2B2CP Statement-Owner Deferral

B2CP is not a statement-witness profile. Its 181-byte functional-update
source motivates only a private Task-254 proof-context reuse seam. Task 258
owns no theorem, statement, context, fact, take, witness, name, or directed
witness target during this prerequisite or its implementation. No
`SourceStatementWitnessTermTarget`, fingerprint, producer, TypedAst
installer, final-clone rule, public API, or debug grammar changes.

After B2CP implementation is separately committed, B2C must fresh-inventory
and freeze its complete Task-258 base transaction and provenance, including
the theorem/statement/context/fact rows for theorem 82 and conclusion 80
and the local owner/label; their exact counts are not fixed by B2CP. The
B2C witness extension may then own only take/witness nodes `72/71` and the
exact witness-to-functional-update `Structure(0)` edge. Task 256 later owns
only nodes `55/77`; formula containers `56/78` remain unowned and its
formula table excludes the update subtree. Update/member/`FieldUpdate`
semantics, replacement/result typing, functional-copy meaning, existential
obligations/substitution, proof, goal, and theorem acceptance remain
deferred. Because the smoke theorem's goal is `x = x`, its `take`
occurrence supplies no semantic-acceptance claim.

## Task 258B3M2B2B2CP Implementation Result: Statement Surface Unchanged

CPC1 correction commit `ee267d9c` is complete and the B2CP private lower
reuse seam is implemented. The two frozen runner tests pass, closing the
prerequisite `design_drift`, bounded `source_drift`, and `test_gap`.
Final test-sufficiency and implementation re-reviews have no findings.
Source-statement production and tests remain unchanged: B2CP still
publishes no Task-258 statement, witness, name, target edge, fingerprint,
TypedAst/final row, public API, or active route.

No specification, `.miz`, fixture, expectation, sidecar, trace
status/count/backlink/credit, or semantic behavior changed. The formula row
remains `deferred`, `tests = []`; coverage impact is narrative-only.
Functional-copy/update meaning, type/result identity, B2C ownership,
proof/goal/theorem acceptance, and IR remain deferred. Concurrent ownership
is report-only `repo_metadata_conflict` with no metadata repair.
Broad formatting, Clippy, tests, and all count/hash gates pass. The final
source/documentation re-review has no findings. Independent final quality
has no findings, all nine hard gates PASS, and valid `98/100`. B2CP
implementation commit `b146f0f72dceac2233c9d679b7820e264974b227` is
complete.

## Task 258B3M2B2B2C Frozen Statement and Witness Contract

The exact 181-byte/86-node B2C source publishes Task-258 base
`1 owner / 2 statements / 2 contexts / 2 input facts / 2 candidate facts`
plus `1 witness / 0 names`. Local contribution 0 has `LocalSource` reserve
anchor `29..47`; checked owner origin is `48..180/[2,1]`; label `56..99`
is public/exported/normal. Owner 0 is theorem site 82, spelling
`FormulaStatementStructureUpdateWitnessSmoke`, role/status
`Theorem/Unmodified`, and normal recovery.

Statement 0 is owner/context `0/0`, `Atomic(0)`, site 82/range `48..180`,
ordinal 0, kind `TheoremProposition`, normal, and spells
`theorem FormulaStatementStructureUpdateWitnessSmoke : x = x proof take TypeCaseStruct ( x : 1 , y : 2 ) with ( x := 3 ) ; thus x = x ; end ;`.
Statement 1 is owner/context `0/1`, `Atomic(1)`, conclusion site 80/range
`164..175`, ordinal 2, kind `Conclusion`, normal, and spells
`thus x = x ;`. Context rows 0/1 name statements 0/1, use binding contexts
0/1, copy ranges `48..180` / `164..175`, and expose `[0]`. Input facts 0/1
are ordinal-0 `ReservedTypeGuard` rows for binding 0 using references
`[0,1]` / `[2,3]`; candidate facts 0/1 are ordinal-0
`UnverifiedProposition` rows targeting `Atomic(0/1)`.

Witness 0 is owner 0/context 1, source/within-take ordinals `1/0`, unnamed,
normal, and nameless. Take is `72/115..161`; item is `71/120..160`;
spelling is
`TypeCaseStruct ( x : 1 , y : 2 ) with ( x := 3 )`; its sole target is
functional-update `Structure(0)`. It records the exact B2CP structure debug
fingerprint, has no application fingerprint, and renders
`term=structure#0`. Transparent 70, constructor `Structure(1)`,
root/member/`FieldUpdate`, primary, application, formula, and container rows
are not witness targets.

Task-256 formula 0 is node/range `55/101..106`, ordinal/context `0/0`;
formula 1 is `77/169..174`, ordinal/context `1/1`. Both are normal
equalities spelling `x = x`; their edges/requests target exactly
`Primary(0/1)` / `Primary(5/6)`, with no direct structure target or
structure fingerprint and with the update subtree excluded.

The existing public `Structure` target, structure fingerprint,
structure-aware producer, combined TypedAst installer, and final clone are
reused unchanged. Exactly four future checker tests freeze API/debug/legacy
compatibility, dependency precedence/all-node failure, combined ownership/
family-order atomicity, and final-clone/semantic deferrals. Exactly five
future runner tests freeze the real frontend, corruption/replay,
update/byte/subtree near misses, family/active-route isolation, and
typed/final/debug/empty-semantics behavior. Validation proceeds through
exact source/arena, local and imported provenance, Tasks 48/252/254/256,
Task-258 base, witness, atomic publication, and final clone. The `x = x`
goal makes this source transport only; all statement semantics remain
deferred.

## Task 258B3M2B2B2C Implemented Statement and Witness Contract

The statement producer now recognizes the exact B2C Task-48/252/256/base
profile and the witness producer additionally authenticates the existing
Task-254 functional-update handoff. It publishes one unnamed proof-context
witness whose sole term target is `Structure(0)` and whose debug spelling is
`term=structure#0`. No name row, public table/API, reverse edge, fact,
obligation, proof result, goal progress, or theorem acceptance is added.

All four frozen checker tests and five frozen runner tests pass. Final
test-sufficiency and implementation reviews have no findings; final
source/documentation and quality reviews remain pending.

## Task 258B3M2B2B2C Broad Statement Verification

The broad format, Clippy, crate, and workspace gates, focused `4/4` and
`5/5`, and sibling `12/12` and `21/21` suites pass with unchanged counts and
hashes. The statement/witness transport above therefore requires no source-
contract change and gains no semantic credit. Independent final source/
documentation and quality reviews, commit, and post-commit inventory remain
pending.

## Task 258B3M2B2B2C Final Statement Review Status

Independent final source/documentation consistency and final quality report
**NO FINDINGS**; all nine hard gates PASS with a valid `98/100`. The exact
statement/witness evidence and semantic deferrals remain unchanged. Only
cached-diff/staging audit, implementation commit, and post-commit inventory/
fresh-next-task gates remain pending.

## Task 258B3M2B2B3P Statement-Owner Deferral

B2C implementation is committed as
`e8373c683448e524cb98edde83fdf8de83a125cd` with clean post-commit
invariants. The next lower prerequisite B3P authenticates only Task-255
enumeration term 0 at node/range `40/90..96` in proof context 1 for the
117-byte set-enumeration source. It owns no `SourceStatement`,
`SourceStatementWitness`, statement-to-term edge, checker API, or checker
test.

The theorem statement, `take` witness, proof, and all containers remain
unowned by B3P. Upper B3A is a separate future logical task that may freeze
and implement `SourceStatementWitness -> SetTerm(0)` plus public witness
schema/installers and four checker/five runner tests. B3P neither anticipates
that edge nor claims witness, existential, substitution, type, goal, proof,
or theorem semantics.

## Task 258B3M2B2B3P Documentation Review Status

All four documentation-phase review tracks report **NO FINDINGS**, and all
recorded source/count/hash/scope/trace-no-op verification passes. This
confirms the B3P statement-owner exclusion without closing the later B3A
consumer. Future B3P implementation `source_drift`/`test_gap` remains
planned; final quality, commit, post-commit, and fresh inventory are pending.

## Task 258B3M2B2B3P Final Quality Status

Final quality has **NO FINDINGS**, all nine hard gates PASS, and valid
`98/100` (`20/20/15/14/10/10/5/4`). Only stage/commit, post-commit, and
fresh implementation inventory remain pending.

## Task 258B3M2B2B3P Implemented Statement-Owner Exclusion

The private lower B3P implementation from prerequisite commit
`285a1f11c310bb313c4c6b4feae914eb11f74754` publishes Task-252 and
Task-255 set-enumeration transport only. Statement, witness, proof, theorem,
and term-expression containers remain unowned, and no
`SourceStatementWitness -> SetTerm(0)` edge or statement semantic row exists.
The two passing tests explicitly preserve that exclusion and isolate all
active and adjacent statement profiles.

This closes lower B3P `source_drift`/`test_gap` without consuming upper B3A
ownership. B3A is the next dependency-authorized task. Test-sufficiency and
implementation, source/documentation consistency repeat, and documentation/
boundary repeat reviews are **NO FINDINGS**. Lint-policy `15/14`, metadata
`137`, focused/library/fmt, workspace Clippy/tests, CLI/manifests/test-list
hashes, diff check, and exact 30-file scope PASS. Independent final quality
reports **NO FINDINGS**; all nine hard gates PASS with valid `98/100`
(`20/20/15/14/10/10/5/4`). Only commit/post-commit and fresh B3A inventory
remain pending.

## Task 258B3M2B2B3A Frozen Source-Statement Witness Contract

The source owner is node/range `53/19..116`, local-only and
public/exported, with label spelling `FormulaStatementSetEnumerationWitnessSmoke`
at `27..69`, reserve anchor `0..18`, origin `19..116/[2,1]`,
`LocalSource` contribution `0`, and no import/recovery.
B3A freshly authenticates the resolver label and `CheckedStatementOwner`;
B3P's empty-label resolver oracle cannot substitute for this check.

Task 258 base remains one owner, two statements/contexts/input facts/
candidates. Statement `0` is owner/context `0/0`, `Atomic(0)`,
`53/19..116`, ordinal `0`, `TheoremProposition`, normalized
`theorem FormulaStatementSetEnumerationWitnessSmoke : x = x proof take { 1 , 2 } ; thus x = x ; end ;`.
Statement `1` is `0/1`, `Atomic(1)`, `51/100..111`, ordinal `2`,
`Conclusion`, normalized `thus x = x ;`. Binding contexts `0/1` see `[0]`;
statement context ranges are `19..116` and `100..111`;
`ReservedTypeGuard` uses refs `[0,1]` / `[2,3]`; candidates are `Atomic(0/1)`.

B3A adds exactly witness `0`/zero names: owner/context `0/1`, source/take
ordinals `1/0`, take `43/85..97`, item `42/90..96`, spelling
`{ 1 , 2 }`, unnamed `Normal`, target `SetTerm(0)`. API is exactly:

```rust
SourceStatementWitnessTermTarget::SetTerm(SourceSetTermId)

pub fn set_term_fingerprint(&self) -> Option<&str>;

pub fn build_with_set_term(
    input: SourceStatementWitnessHandoffInput,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    set_terms: &SourceSetTermHandoff,
    arena: &TypedArena,
) -> Result<SourceStatementWitnessHandoff, SourceStatementWitnessError>;

pub(crate) fn validate_installation_with_set_term(
    &self,
    source_id: SourceId,
    module_id: &ModuleId,
    statements: &SourceStatementHandoff,
    primary_terms: &SourcePrimaryTermHandoff,
    set_terms: Option<&SourceSetTermHandoff>,
    arena: &TypedArena,
) -> Result<(), SourceStatementWitnessError>;
```

Accepted application/structure/set fingerprint tuples are
`None/None/None`, `Some/None/None`, `None/Some/None`, and B3A
`None/None/Some`; every other tuple is invalid. Debug appends set fingerprint
after existing optional fields and renders `set-term#0`. Producer/handoff
validation retains `SourceStatementWitnessError::DependencyMismatch`;
typed installation and final statement/witness revalidation retain their
existing `InvalidSourceStatement` variants. Earlier final lower-stage
failures retain their owning variants, including `InvalidSourceSetTerm` and
`InvalidSourceAtomicFormula`. No error variant/display or legacy byte
changes.

The frozen four checker plus five runner tests exhaust bytes/LF, `57` AST
nodes/root, resolver including label, Tasks 48/252/255/256/258, partition/
graph, fingerprints, set/label near misses, family hybrids/orders,
rollback/replay, final clone, and empty semantics. Precedence is source/AST,
resolver plus label, Tasks 48, 252, 255, 256, 258 base, witness, atomic
publication, final clone.

This is transport only. Result/numeric/set/element typing, existential goal
matching, witness guards/obligations, substitution, goal progress/discharge,
proof/theorem acceptance, facts, overload/coercion, Core/CFG/VC, imported
sets, broader set forms, B4/B5, and active-route/corpus/diagnostic credit
remain deferred.

## Task 258B3M2B2B3B Frozen Empty-Enumeration Statement Contract

B3A closed at `a147bad88f1963c504f796051ba0b855eca71d07`; its generic SetTerm
carrier does not make its exact `{ 1 , 2 }` statement profile accept `{}`.
B3B therefore freezes the 118-byte
`FormulaStatementEmptySetEnumerationWitnessSmoke` source with zero
diagnostics, 50 nodes/root 49, local public/exported theorem
`46/19..117`, label `27..74`, and proof context `1` at `82..116`.

Task 252 owns reference roots `{27,29,37,39}`. Task 255 owns only empty
enumeration `33/95..97`, spelling `{ }`, profile
`1/0/0/0/0/0/1`, no child edge, and one `ResultType` request. Task 256 owns
formula roots `{31,41}`; Task-258 base owns `{44,46}`. B3B owns witness/take
`{35,36}` and the sole `Witness(0) -> SetTerm(0)` edge. All other nodes are
unowned as frozen in the crate plan.

B3B reuses the B3A SetTerm API and set-only fingerprint tuple. Its future
private exact profile changes no public schema or debug grammar. The
four-checker/five-runner matrix freezes all bytes/nodes/resolver/lower rows,
zero-edge nonvacuity, precedence, family isolation, replay/rollback, final
clone, and empty semantics. Singleton/nonempty enumeration, choice,
comprehension, `qua`, named/multiple witnesses, semantic typing,
existential/proof behavior, B4/B5, and active/trace credit remain deferred.

## Task 258B3M2B2B3B Implemented Empty-Enumeration Witness Closure

The private exact profile now authenticates the 118-byte source, all 50
nodes/root 49, local theorem/label provenance, Tasks 48/252/255/256/258,
the full ownership partition, and one unnamed
`SetTerm(SourceSetTermId::new(0))` witness. The Task-255 term remains the
exact zero-edge `{ }` enumeration with one `ResultType` request. The four
checker plus five runner tests cover byte/node mutation, both resolver
matrices, frozen lower and upper values, bidirectional family orders,
non-vacuous zero-edge rejection, rollback/replay, final clone, and empty
semantics.

No public schema, error, debug grammar, dependency, active route, or
semantic result changed. The three initial medium test gaps and the
repeat's additional currently mutable Task-48/252/255 mutation/replay gap
are remediated; the latter has exact `32/55/23` matrices. Post-auth
injection and stage-prefix/non-generic-guard assertions complete their
authentication. All test-sufficiency repeats and the final implementation
repeat report **NO FINDINGS**. Source/documentation consistency repeat also
reports **NO FINDINGS**. Final documentation/boundary and independent
quality reviews report **NO FINDINGS**, all hard gates PASS, valid
`98/100`.

## Task 258B3M2B2B3C Frozen Statement Profile

The exact `110`-byte/`52`-node choice source contributes one theorem owner,
two statement/context/guard/candidate rows, and one unnamed witness/no name.
Task 258 owns `{48,46}` and B3C owns `{38,37}`; the witness at
`37/82..89` targets `SetTerm(0)` in proof context `1`. The complete
cross-family graph and owner partition are frozen in the crate plan.
Implementation must reuse the existing SetTerm target/fingerprint/install/
clone APIs and add no public/error/debug/semantic surface. Exact four checker
and five runner tests plus `32/55/39/72/62/21` mutation matrices are frozen.

## Task 258B3M2B2B3C Implemented Choice Statement

The source-statement producer now accepts only the exact 110-byte,
52-node/root-51 `take the set;` profile and installs one unnamed witness
whose target is `SetTerm(0)`. It validates the complete Task-48/252/255/256/
258 tables, exact owner partition, local resolver provenance, zero
Task-255 edges, set fingerprint, and choice/witness subtree exclusion before
publishing the syntax-free handoff.

All bytes/final LF, `52 x 4` node surfaces/root, resolver mutations,
`32/55/39/72/62/21` fields, family orders, immediate replay, clone/
rollback/debug, and empty semantics are exercised by the frozen four checker
plus five runner tests. Resolver replay and exact upper stage-prefix/
non-generic rejection close two initial medium `test_gap` findings. A
B3A-hard-coded branch was restricted to B3C while retaining both enumeration
siblings, closing `source_drift`/`test_gap`; repeated reviews report
**NO FINDINGS**. Public APIs/errors/debug grammar and semantics remain
unchanged. The new private dormant exact selector branch is not selected by
active corpus sources, so existing active-corpus routing and outcomes remain
unchanged.

## Task 258B3M2B2B3D Frozen Qua Statement Profile

The exact 109-byte/54-node qua source contributes one theorem owner, two
statement/context/guard/candidate rows, and one unnamed witness/no name.
Task 258 owns `{50,48}` and B3D owns `{40,39}`; the witness at
`39/79..88` targets `SetTerm(0)` in proof context `1`. Task 255 owns
`{35,36,37}` and its `QuaBase -> Primary(2)` edge; the complete owner/
unowned graph is frozen in the crate plan. Existing SetTerm fingerprint,
producer, typed install, final replay, error, and debug APIs are reused.
Four checker/five runner tests and `32/70/44/72/62/21` matrices are frozen.

## Task 258B3M2B2B3D Implemented Qua Statement Inventory

The source-statement producer now recognizes only the exact dormant
109-byte/54-node/root-53 qua source and installs one unnamed witness
targeting `SetTerm(0)`. It authenticates Task-48/252/255/256/258, the
complete owner/unowned partition, local resolver provenance, the
`QuaBase`/ordered-request graph, set fingerprint, and witness/subtree
exclusions before publication.

The frozen four checker and five runner tests exhaust bytes/final LF,
`54 x 4` node surfaces/root, resolver provenance, exact
`32/70/44/72/62/21` fields with replay and owning prefixes, all 24 B3
family orders, clone/rollback/debug behavior, and empty semantics.
Test-sufficiency review reports **NO FINDINGS**. Public APIs/errors/debug
grammar, both Task-255 source owners, active corpus behavior, and semantic
tables remain unchanged. Independent implementation review also reports
**NO FINDINGS**. Repeated source/documentation and boundary review reports
**NO FINDINGS** after correcting the Medium stale-review state and the two
Low 24-order/qua-edge descriptions. Both packages, formatting, full Clippy,
workspace tests, five CLIs, and count/hash reruns PASS. Independent final
read-only quality review reports **NO FINDINGS**; all nine hard gates PASS
with no cap at valid `100/100` (`20/20/15/15/10/10/5/5`). Only exact
staging/cached-diff review, implementation commit, and
post-commit/fresh-next-task gates remain pending.

## Task 258B3M2B2B3E Frozen Comprehension Statement Profile

The exact 139-byte source owns theorem/conclusion nodes `{56,54}` and B3E
take/witness nodes `{46,45}`. The unnamed witness at `45/89..118` targets
Task-255 `SetTerm(0)` at `43/89..118`; the take is `46/84..119`.
Task-252 owns `{32,34,38,47,49}`, Task-255 `{16,40,41,43}`, Task-256
`{36,51}`, and generator segment `42` remains unowned. The complete graph is
the two equality pairs, comprehension mapper to `Primary(2)`, and witness to
the set term. No generator binding, semantic witness, proof acceptance, or
fact edge is introduced.

## Task 258B3M2B2B3E Implemented Comprehension Statement Inventory

The private selector authenticates the exact final-LF 139-byte/60-node
source and theorem/label provenance before accepting the exact
Task-48/252/255/256/258 tuple. B3E publishes one unnamed
`SourceStatementWitnessInput` at witness/take nodes `{45,46}`, targeting
`SetTerm(0)`. Existing public witness DTOs and producer APIs are reused; only
private profile cases and validators are added.

The four frozen checker tests cover exact profile/legacy compatibility,
owning-stage precedence/all nodes, combined ownership/120 family orders, and
final clone/semantic deferrals. Exact matrices, coherent post-auth Task-255
near misses, non-generic guards, repeated failure, and clean replay pass. No
witness matching, substitution, goal progress, proof acceptance, facts,
Core/CFG/VC, B4/B5, or active-route behavior is added.

Final source/documentation consistency reports **NO FINDINGS** after the
bounded design corrections. Full verification PASSes; independent final
quality reports **NO FINDINGS**, all nine hard gates PASS, valid `100/100`.
Staging and post-commit gates subsequently closed in implementation commit
`e4479691db3b0a8785bb16e94d386bd71a394274`; fresh inventory selected
Task 258B4A.

## Task 258B4A Frozen Composite Statement Root

B4A consumes the private 80-byte/double-LF explicit-universal theorem and
the already authenticated Task-257B1 `1/0/1/1/1/0/2` composite plus `1/2`
composition. It publishes one theorem owner, one theorem statement, one
context, zero input facts, and one unverified candidate. Both statement and
candidate target `SourceStatementFormulaTarget::Composite(
SourceCompositeFormulaId::new(0))`; context 0 names binding context 0 with
an empty visible-binding set.

Owner 0 is the checked local theorem symbol at Surface site 22/range
`0..78`, contribution 0, label spelling
`FormulaQuantifierBoundUsePayloadBoundary`, `Theorem`/`Unmodified`/`Normal`.
Statement 0 is owner/context 0, site 22/range `0..78`, ordinal 0,
`TheoremProposition`/`Normal`, with normalized spelling
`theorem FormulaQuantifierBoundUsePayloadBoundary : for x being set holds x = x ;`.
Context 0 is statement 0, binding context 0, range `0..78`, visible `[]`.
Candidate 0 is statement/context/ordinal 0,
`UnverifiedProposition`, `Composite(0)`.

`SourceStatementFormulaTarget` gains the `Composite` variant.
`SourceStatementHandoff` stores optional composite-formula and
formula-composition fingerprints, exposed by
`composite_formula_fingerprint(&self) -> Option<&str>` and
`formula_composition_fingerprint(&self) -> Option<&str>`. Existing atomic
routes keep both absent and retain byte-identical debug text. Present values
equal the corresponding lower handoff `debug_text()` bytes. The dedicated
`SourceStatementProducer::build_with_formula_composition` argument order is
input, symbols, bindings, primary terms, atomic formulas, composite
formulas, formula composition, and arena. It validates Task-252/256/257/B1,
resolver owner contribution 0/origin `[2,0]`, the complete `1/1/1/0/1`
table profile, exact `Composite(0)` links, and subtree exclusion before
publication.

For a B4A handoff only, `debug_text()` inserts the Rust-Debug-quoted
`composite-formula-fingerprint: {:?}` followed by
`formula-composition-fingerprint: {:?}` after the existing atomic-formula
fingerprint and before owner 0. Atomic handoffs omit both lines, so their
complete debug bytes do not change.

The lower `UnassignedStatement` root ownership stays unchanged. No binder
fact, truth, proof acceptance, publication, fact, goal, justification,
diagnostic, or semantic result is inferred. The active one-final-LF
79-byte Task-257B1 fixture is an upper-route negative; only the private
double-LF 80-byte source can select B4A.

Repeated read-only documentation review reports **NO FINDINGS**. Independent
final quality passes all nine hard gates with no cap at valid `100/100`;
only staging, commit, and post-commit inventory remain.

## Task 258B4A Implemented Composite Statement Root

`SourceStatementFormulaTarget` now admits `Composite(0)`, and statement
handoffs optionally fingerprint both the Task-257 composite and composition
dependencies. The dedicated producer authenticates the frozen 80-byte
route's syntax-free input, resolver owner contribution 0/origin `[2,0]`,
lower profiles, exact owned lower sites/ranges, and the complete
`1/1/1/0/1` upper tables before publication. The runner selector separately
authenticates the source bytes and all 26 Surface rows/root 25. Atomic
statement routes retain absent optional fingerprints and byte-identical
debug text.

The B4A debug text adds only the two frozen quoted fingerprint lines.
Thirty-eight upper-input mutations, coherent rooted-arena and relocated-term
lower near misses, nineteen final statement corruptions, missing-lower
tuples, route isolation, and replay prove the failure boundary. The lower
typed arena remains rootless while Surface root 25 remains authenticated;
`UnassignedStatement` is not rewritten. No truth, fact, theorem acceptance,
proof, goal, justification, diagnostic, or semantic result is added.

## Task 258B4B Frozen Connective/Grouping Statement Root

The exact private source is 167 bytes, ends in two LFs, hashes to
`3145e60413841ae005977400f1acd21f0974c7bad635f37fe3df6eeae7700748`,
and parses with zero diagnostics as 124 Surface nodes/root 123. The theorem
owner is node 120/range `0..165`, label node 1/range `8..48`, and universal
root node 118/range `50..164`. Raw real-frontend resolver ownership is
public/exported local theorem contribution 0, origin `[2,0]`, with no label
projection, import, or recovery. The runner enriches it before handoff with
exactly one public/exported theorem `LabelProjection` for contribution 0
whose spelling, namespace, origin, range anchor, contribution, normal
recovery, and contribution label effect all match the owner; exact enriched
resolver cardinalities are `1/1/1/1/0`.

The statement handoff is exactly one owner, one statement, one context, zero
input facts, and one candidate. Owner spelling is
`FormulaConnectiveGroupingPayloadBoundary`. Statement and candidate target
`Composite(0)`; the statement spelling is
`theorem FormulaConnectiveGroupingPayloadBoundary : for x being set holds ( ( 0 = 0 & ... & 0 = 3 ) or ( 0 = 0 or ... or 0 = 3 ) ) iff ( ( 0 = 0 & 0 = 0 ) or ( 0 = 0 or 0 = 0 ) ) ;`.
Context 0 references binding context 0/range `0..165` with visible `[]`.

No public DTO, producer, accessor, installer, error, or debug grammar is
added. `build_with_formula_composition` and the two existing optional lower
fingerprints are extended only to the exact matched Task-257B2 profile.
The crate-private cardinality-only `is_task_258b4a_profile` is narrowed to
B4A's exact owner spelling/range, and a symmetric exact
`is_task_258b4b_profile` is added. Their call sites must pair B4A only with
Task-257B1 and B4B only with Task-257B2.
The 42 lower-owned nodes and `UnassignedStatement` root remain unchanged;
Task 258 owns only theorem node 120 and the two upper root links. The active
166-byte source, B4A, atomic statement families, rooted/relocated lower
near misses, and every profile hybrid are mandatory fail-closed negatives.
The runner mutation matrix authenticates and independently corrupts every
field of that enriched theorem label projection and its contribution label
effect; describing the raw preflight as label-free does not waive the
checker-consumed label contract.
The runner-private route output uses lookup telemetry `0/0/[]` for this
zero-reference profile. Those zeros are sentinels, not reference ordinals;
the transport-detail guard must accept them only for the exact matched
Task-257B2/B4B profile and must keep B4A at `1/1/[1,1]`. No public checker
DTO or statement semantic is changed by this dormant runner convention.

## Task 258B4B Implemented Connective/Grouping Statement Root

The private 167-byte/double-LF source now selects the frozen route only after
raw label-free resolver provenance is enriched to exact `1/1/1/1/0`. The
producer reuses the Task-257B2 lower transaction in a rootless 124-node
arena, retains the `42/1/81` ownership split, and installs exactly one owner,
one statement, one context, zero input facts, and one candidate
(`1/1/1/0/1`). Both statement associations target `Composite(0)`.

The exact profile predicates and their call sites pair Task-257B1 only with
B4A and Task-257B2 only with B4B. Runner telemetry `0/0/[]` is accepted only
for the matched B2/B4B profile; B1/B4A remains `1/1/[1,1]`. The active
166-byte route and all coherent profile hybrids fail closed. Four checker
and five runner focused tests pass, and separate test-sufficiency and
implementation reviews report **NO FINDINGS**. No public DTO, debug grammar,
lower owner, semantic table, corpus artifact, or trace state changes.

Final source/documentation, bilingual, and boundary consistency reviews now
report **NO FINDINGS**. Focused checker `4/4` and runner `5/5`, full
`cargo test --offline`, `cargo fmt --all -- --check`, full offline Clippy
with warnings denied, five CLI, all count/hash, exact-scope, audit-no-op,
forbidden-artifact, and unchanged-stash gates PASS. Independent final
quality reports **NO FINDINGS**; all nine hard gates PASS with no cap at
valid `100/100` (`20/20/15/15/10/10/5/5`). Staging/cached-diff review,
the implementation commit, post-commit inventory, and B4C remain pending.

## Task 258B4C Frozen Nested-Quantifier Statement Root

The exact private source is 139 bytes with two final LFs and SHA-256
`36e5a68a92451590644951838a9af8926212bd78f88d1f90563f12b650b161c1`.
It parses with zero diagnostics as 66 normal Surface nodes/root 65. Reserve
item 35 is `0..18`, theorem item 62 is `19..137`, label token 6 is
`27..65`, and outer composite root 60 is `67..136`. Raw resolver
cardinalities are `1/0/1/1/0`: public/exported theorem owner range
`19..137`, origin `[2,1]`, contribution 0 anchored to reserve `0..18`.
The runner adds one exact theorem label projection and contribution effect,
producing `1/1/1/1/0`.

The active 138-byte source remains Task-257B3 lower-only. The current exact
source guard does not admit the private double-LF source, so a separate
lower-stage prerequisite must first extend only the private Task-257B3
selector and its runner tests to exact 138-or-139-byte acceptance while
rejecting zero/triple LF. B4C upper implementation must not contain that
repair, and production `source_formula_composition.rs` remains unchanged.

The matched lower profile is binding `4/4/0`, primary `6/6/0`, atomic
`3/0/0/0/0/0/0/6/6`, composite `3/0/1/3/3/2/6`, and composition
`3/6`. The 66-node rootless arena retains exact lower ownership
`{9,17,22,32,33,36,37,38,39,41,43,44,45,46,47,48,50,52,53,55,57,58,59,60}`;
Task 258 owns only theorem node 62, leaving 41 nodes unowned. The lower root
remains `UnassignedStatement`.

The upper publication is exactly `1/1/1/0/1`. Owner and statement use site
62/range `19..137`; statement and candidate target `Composite(0)`. Context
0 uses binding context 0 with visible `[0]`. There is no input fact because
no Task-252 reference selects reserved binding 0; the six references select
bindings `1,1,3,2,1,3` with ordinals `2,2,4,4,4,4`. Private route telemetry
is `2/2/[2,2,4,4,4,4]`. Exact profile pairing is B4A/B1, B4B/B2, and
B4C/B3 only.

No public API, error, or debug grammar changes. The same seven eventual upper
consumers as B4B are frozen, with four checker and five runner tests. Truth,
restriction discharge, existential witnesses, implicit closure, facts,
theorem acceptance/publication, proofs, and downstream IR remain deferred.

## Task 258B4C Implemented Nested-Quantifier Statement Root

The separately committed selector prerequisite
`42356f38ed0e679d7b878caf0e647c6aa8148d82` supplies the exact private
139-byte lower transaction. The producer authenticates all 66 Surface rows,
raw `1/0/1/1/0` provenance, enriched `1/1/1/1/0`, and matched Task-257B3
handoffs before publishing upper `1/1/1/0/1`. Statement and candidate both
target `Composite(0)`; context 0 exposes exactly `[0]`, with no input fact.

Checker revalidation covers all lower fingerprints, exact B1/A-B2/B-B3/C
pairing, every rootless-arena anchor and normal recovery state, and exact
`24/1/41` ownership. Resolver, row, arena, family/Task-248, telemetry,
rollback/replay, clone/debug, and empty-semantic matrices are covered by
exactly four checker and five runner tests. Test-sufficiency and
implementation reviews report **NO FINDINGS**.

This remains syntax/provenance transport only. Public APIs, debug/error
grammar, active artifacts, trace/coverage state, truth, restriction or
witness semantics, facts, theorem acceptance, proof, and IR are unchanged.

## Task 258B5A Frozen Ancestor/Descendant Statement Transaction

The private 185-byte source publishes exactly one theorem owner and five
syntax-free rows: theorem node 89/context 0/`Atomic(0)`, labeled proof-step
node 67/context 1/`Atomic(1)`, conclusion node 65/context 2/`Atomic(2)`,
outer conclusion node 87/context 1/`Atomic(3)`, and descendant conclusion
node 85/context 3/`Atomic(4)`. Source ordinals are `0..4`; each context sees
reserved binding `[0]`, each input consumes Task-252 references
`[2i,2i+1]`, and each candidate is unverified.

Reference handoff label row 0 names statement/context/candidate 1, range
`95..96`, origin `<package>::<module>::proof::A`, visible-after ordinal 1,
scope `[0]`, and private/local-only SemanticOrigin `[12]`. Citation row 0
names statement/context 4, range `170..171`, `LabelRefId(0)`, scope `[0,1]`,
simple-local kind, and resolver node 82/SemanticOrigin `[82]`. Exact prefix
visibility is provenance only: the task creates no fact, acceptance, proof,
goal, or other semantic output.

## Task 258B5A Implemented Ancestor/Descendant Statement Transaction

The private producer now authenticates the exact 185-byte source, all 93
normal Surface rows/root 92, raw and enriched resolver provenance, and the
unchanged BindingEnv, Task-252, and Task-256 handoffs before constructing the
frozen base `1/5/5/5/5` and reference `1/1` profiles. It owns only the ten
term, five formula, and five statement nodes; the label, citation,
proof-block, wrapper, and other 73 nodes remain arena provenance.

The label remains statement/context/candidate 1 at `95..96`, scope `[0]`,
visible-after ordinal 1, private/local-only contribution 0. The citation
remains statement/context 4 at `170..171`, scope `[0,1]`, resolver node 82,
and `LabelRefId(0)`. Reference validation also authenticates every resolver
node kind so a coherent arena-kind mutation cannot bypass the frozen
Surface/resolver identity. B1/B5A cross-pairing, row, scope, ownership,
fingerprint, relocation/recovery, and replay mismatches fail atomically.
No fact, acceptance, proof, goal, diagnostic, or public API is added.

## Task 258B5B Frozen Imported Citation Transaction

The private 146-byte/final-LF B5B source has 57 normal Surface rows/root 56.
After the mandatory separate import-summary prerequisite, the producer must
authenticate raw resolver `1/0/1/1/0`, opt-in augmented resolver
`8/1/1/3/1`, BindingEnv `2/1/0`, Task-252 `4/4/0`, Task-256
`2/0/0/0/0/0/0/4/4`, Task-258 base `1/2/2/2/2`, and reference
local-label/citation `0/1`.

Owner 0 is theorem node 53/range `48..145`. Statement 0 is that theorem in
context 0 over `Atomic(0)` and references `[0,1]`; statement 1 is conclusion
node 51/range `122..140` in proof context 1/scope `[0]` over `Atomic(1)` and
references `[2,3]`. Each has one reserved-type-guard input and one
unverified candidate. The transaction owns only terms `35,37,41,43`,
formulas `40,46`, and statements `51,53`, preserving exact `8/49`
ownership.

Primary term/ref ids 0..3 are node/range/context/source-ordinal
`35/108..109/0/0`, `37/112..113/0/1`, `41/127..128/1/2`, and
`43/131..132/1/3`; all spell `x`, select binding 0 at stored use ordinal 1,
and are normal `VariableReference`/`Value` rows with matching reference ids,
role `Variable`, and scopes none/none/`[0]`/`[0]`. Atomic formulas 0/1 are
normal equality nodes
`40/108..113` and `46/127..132`, context/source ordinal `0/0` and `1/1`,
spelling `x = x`, with paired left/right primary edges and exact request
triples `(0,0,0), (0,1,1), (1,0,2), (1,1,3)` exactly as frozen in the
crate plan.

The owner is the current-module public/exported theorem symbol at
contribution 0 with current-source/current-module origin anchor `48..145`,
path `[2,1]`, no import edge, and normal recovery. Statement source ordinals
are 0/1; their normalized spellings are
`theorem FormulaStatementImportedPublicTheoremCitationSmoke : x = x proof thus x = x by Ref ; end ;`
and `thus x = x by Ref ;`. Input-fact and candidate table ids are 0/1, but
each row's own ordinal field is 0; inputs use `[0,1]`/`[2,3]`, and
candidates target `Atomic(0)`/`Atomic(1)`.

There is no local label row. Citation id 0 belongs to statement/context 1,
node/range `48 / 136..139`, scope `[0]`, and dense citation-row ordinal 0.
The resolver reference candidate independently has source-statement ordinal
1. The citation uses `LabelRefId(0)`, `ProofOrTheorem`, spelling `Ref`, and
normal recovery. Its
singular resolver projection is imported/public/exported theorem provenance
from `parser.type_fixtures`, with the exact opt-in origin path
`summary:parser.type_fixtures::Ref:label:Ref`, structural path `[1,0]`,
current-module namespace, imported contribution 2, and anchor `7..27`.

Resolved import id 0 is owned by resolver node 29 (`ImportAliasDecl`), has
range `7..27`, exact spelling `import parser.type_fixtures;`, no alias, and
resolution `Resolved(<package>::parser.type_fixtures)`. Its origin is the
current source/current module, range anchor `7..27`, path `[0]`, no import
edge, and normal recovery. Nodes 28/29/30 retain their own
`ModulePath`/`ImportAliasDecl`/import-item identities, ranges
`7..27`/`7..27`/`0..28`, paths `[28]`/`[29]`/`[30]`, normal recovery,
`NotApplicable`, and no reference key. Node 48 remains the sole keyed node.

The imported projection declaration range is `7..27`; its semantic origin
uses the current source, declaring module
`<package>::parser.type_fixtures`, range anchor `7..27`, path `[1,0]`, no
import edge, and normal recovery. The reference candidate origin uses the
current source/current module, range anchor `136..139`, path `[48]`, no
import edge, and normal recovery. Producer and final-clone tests mutate
every import-row and both origin tuples independently.

Because an imported citation has no local label id, the later upper task
adds non-exhaustive public
`SourceStatementCitationTarget::{Local(SourceStatementLabelId), Imported}`.
Citation input, immutable row, and getter use `target`/`target()` in place
of mandatory `label`/`label()`; B1/B5A use `Local` unchanged.
`SourceStatementCitationKind::SimpleImported` is added. B5B debug prints the
imported projection and `target=imported`, represents the absent local label
node, and emits no `label#0` line; B1/B5A debug bytes remain exact.

The complete B5B public debug schema is:

```text
source-statement-reference-debug-v1
module: <package>::<module>
statement-fingerprint: <quoted source-statement debug>
resolver-ast root=56 nodes=57 name_refs=0 label_refs=1 imports=1 exports=0 label_node=absent reference_node=48 reference_state=resolved reference_key=label#0
resolver-projection source=imported origin=summary:parser.type_fixtures::Ref:label:Ref module=parser.type_fixtures namespace=<module> range=7..27 contribution=2 path=[1,0] kind=theorem visibility=public export=exported spelling="Ref"
resolver-reference node=48 range=136..139 source_ordinal=1 scope=[0] expectation=proof-or-theorem spelling="Ref"
resolver-result index=1 references=1 ids=[0] diagnostics=0
citation#0 statement=1 context=1 target=imported label_ref=0 scope=[0] range=136..139 ordinal=0 kind=simple-imported recovery=normal
```

The placeholders denote validated runtime module/fingerprint values; every
other token, field order, and line order is literal. No `label#0` line is
present. Checker test 1, runner test 1, and final-clone coverage assert the
whole schema while retaining byte-identical B1/B5A output.

The producer rejects absent, duplicate, private/local-only, re-exported or
otherwise wrong export status, wrong-kind,
wrong-module/namespace/contribution/origin/range/path, recovered, stale,
relocated, wrong dense citation-row ordinal, wrong resolver source-statement
ordinal, cross-profile, partial, and wrongly keyed rows atomically. Checker
test 2, runner test 2, and final-clone coverage independently mutate
`Exported` to `ReExported`.
B5C and every semantic result remain deferred. This prerequisite changes no
source, fixture, expectation, trace row, or public runner schema.

## Task 258B5B Implemented Imported Citation Transaction

After documentation commit
`141dc44a757555e8d4837756515e1577f672348b` and isolated lower commit
`46dd9db56ced2fcc57799420de9d5fed06f284f5`, the upper transaction
implements the frozen 146-byte route in the three checker and four runner
consumers only. It publishes exact Task-258 base `1/2/2/2/2`, reference
`0/1`, and root-preserving `8/49` ownership from the 57-node/root-56
resolver arena.

Citation row 0 uses `target=Imported`, `SimpleImported`, statement/context
1, `LabelRefId(0)`, scope `[0]`, range `136..139`, and dense ordinal 0.
There is no local label row. The producer authenticates resolved import 0,
the imported/public/exported theorem projection, reference node 48,
resolution key 0, source-statement ordinal 1, and the independent
source/module/range/anchor/path/recovery provenance before publication.
Its debug output includes `label_node=absent` and `source=imported`, emits
no `label#0` row, and leaves B1/B5A local debug bytes unchanged.

The primary API sketch and Public Enum Policy above now reflect the actual
non-exhaustive target enum, `target` field/accessor, and
`SimpleImported` variant. Dependency, aggregate, import, projection,
reference, row, cross-profile, installation, and final-clone mutations fail
atomically and preserve valid replay. Four checker tests and five upper
runner tests cover the exact route; the separate lower commit retains its
two tests. B5B alone compares the full nested operand child paths because
both operands share the formula wrapper's sole immediate child; every
pre-existing statement profile retains its immediate-child ordering rule,
and the exact-profile test records this distinction. Facts, acceptance,
proofs, goals, diagnostics, and downstream IR remain empty, while B5C and
active corpus/trace coverage remain deferred.

## Task 258B5C Frozen Unresolved-Reference Exclusion

R-032A supplies the validated structural arena; R-032B supplies one
private/local-only proof-step projection for `A` at scope `[0,0]`, visible
after completion ordinal 3, and one simple unqualified reference candidate
at either enclosing scope `[0]` or sibling scope `[0,1]`. Both resolution results have
`has_unresolved = true` and exactly one `UnresolvedLabelRef`.

`SourceStatementReferenceHandoff` intentionally rejects that state and
requires the reference node to have a keyed `Resolved` result. Therefore
the two B5C negatives publish no `SourceStatementLabelInput`,
`SourceStatementCitationInput`, immutable label/citation row, resolver
projection replay, statement/reference profile, owned-node partition, or
debug output. The declaration-symbol runner observes the resolver failure
directly after R-032A structural validation and R-032B collection; it must
not manufacture a local label id, resolved node, scope, imported target,
citation ordinal, or statement context.

All Surface nodes remain syntax-owned. Structure constructors, selectors,
functional/field updates, Task-252 terms, Task-253 formulas, B1/B5A/B5B
profiles, facts, proof progress, acceptance, and downstream semantics have
no B5C edge.

The resolver source form is specifically a normal
`ConclusionStatement -> JustificationClause -> ReferenceList -> Reference`
path. Its module-global owning-statement ordinal and canonical
`proof-step-v1` provenance remain below this handoff. Source-byte runner
selection and `proof_scope_input` failures publish no source-statement row.

The complete lower allowlist starts at exact
`Root -> CompilationUnit -> ItemList -> direct TheoremItem -> direct
ProofBlock`, and admits only direct normal compact/
conclusion statements, inspects only a compact proposition label, and
descends from a supported statement only to direct proof/justification
children. Its exact simple-reference chain is the one shown above. All
Root and CompilationUnit each require exactly one normal structural child,
and ItemList scans only direct normal theorem children. Missing/additional/
wrong upper children, direct Root/Compilation theorem relocation,
`VisibleItem` wrapping, and all other excluded and mixed forms are
no-row/no-ordinal/no-descent.

The runner independently validates env/module, module-derived namespace,
exact one id-0 LocalSource contribution record/public `ast.source_id`, and
each projection's module/namespace/contribution. Every field mutation is
input-only and therefore cannot publish this DTO.

## Task 269A Frozen Upper Consumer Boundary

Task 269A consumes, but does not modify, the exact Task-258B3N
`SourceStatementHandoff` and `SourceStatementWitnessHandoff`. Witness 0, name
0, and RHS `Primary(2)` are immutable lower identities; their complete debug
bytes are retained as upper fingerprints. The new handoff owns only the
definition-site association and extended binding environment.

The existing name node 13 remains `source.statement-witness.name`, witness
node 36 remains `source.statement-witness.item`, and take node 37 remains
`source.statement-witness.take`. No `source.proof-local.*` node is created.
The existing Task-258B3N route/debug result remains a stable lower oracle.

## Task 269A Active Upper Consumer Boundary

The separate proof-local producer now fingerprints and consumes the unchanged
Task-258B3N statement/witness/primary bundle. Four adjacent checker tests reuse
the private B3N oracle without changing the lower public API, nodes, rows, or
legacy debug bytes. All new binding ownership remains in
`source_proof_local_declaration`.

## Task 269B B3M1 lower-consumer boundary

Task 269B consumes the already frozen Task-258B3M1 `2 witnesses / 1 name`
profile byte-for-byte. Its complete 56-node authentication and replay preserve
the existing distributed node ownership: source-statement owns the take,
witness-item, witness-name, dense within-take order, and its witness/statement
handoffs; Task 252 retains the RHS-reference nodes, and Task 256 retains the
formula nodes. The fifth fingerprint authenticates the final binding
environment rather than a lower source-statement input. The new binding
increment may link only witness/name/RHS `0/0/2`; it must not alter or bind
unnamed witness 1, change any lower API/debug byte, or assign left-to-right
goal semantics.

## Task 269B active B3M1 lower-consumer boundary

The implemented upper consumer leaves every B3M1 lower row, node, range,
ordinal, and debug byte unchanged. It fingerprints the two-row witness handoff,
associates only named row 0/name 0/RHS primary 2, and explicitly verifies that
unnamed row 1 allocates no checker binding. All-node and isolated cross-profile
tests remain adjacent private tests and add no lower API or semantic meaning.

## Task 269CP lower statement boundary

The runner-private exact source contains theorem ordinal 0, proof-local let
ordinal 1, and conclusion ordinal 2, but 269CP publishes no new
`SourceStatementKind`, statement/generalization table, or statement handoff.
It authenticates nodes 47/46/37/36/13/35/34 and resolver theorem provenance
as role anchors within the complete 51-node normal Surface snapshot. The
runner also authenticates root node 50, absent expression root, token nodes
0 through 23, and every node's source identity, range, recovery, and ordered
children. Existing Task-258 and Task-269A/B debug bytes and profiles remain
unchanged; a checker statement edge requires a later frozen contract.

## Task 269C no-statement binding boundary

The binding-only transaction consumes the Task-269CP theorem/proof/let ranges
but adds no `SourceStatementKind`, statement context/fact/candidate row,
formula edge, or statement semantic. `SourceStatement(59..98)` is solely the
binding-context owner tag. Goal/thesis and conclusion ownership remain
deferred.

The implemented transaction preserves this boundary: its context owner tag is
validated as provenance only and no statement, formula, thesis, conclusion,
fact, or proof row is emitted.

## Task 269CT No-Statement Boundary

The source-type prerequisite reuses Task-269CP theorem/proof/let provenance
and Task-269C binding ownership without publishing a statement row, current
goal, thesis transition, proof-skeleton node, conclusion, fact, or acceptance.
All statement APIs, fingerprints, and tests remain unchanged.

## Task 269CT Implemented No-Statement Boundary

The final composite maps only its three exact source-type nodes. A regression
proves that even a complete three-row `source.statement.transport` hint set is
rejected with `InvalidSourceProofLocalLetType`; it cannot be silently consumed
or overridden. Statement handoffs, semantics, proofs, and fingerprints remain
empty/unchanged.

## Task 269GP No-Statement Lower Boundary

The runner authenticates `GivenStatement(70..108)` as a syntax owner range
only and publishes no binding scope or visibility.
Condition, proposition, label, thesis, conclusion, fact, statement context,
and proof rows are excluded from `SourceProofLocalGivenLowerOutput`. No source-
statement API or checker fingerprint changes.

The implemented runner-only projection retains exactly this exclusion; no
checker source-statement file or API changed.

## Task 269GS No-Statement-Owner Reconciliation

The canonical block-lifetime rule does not change source-statement lowering or
the existing 269GP private row. No condition, label, formula, or statement
payload is added. Binding-only consumption remains Task 269G.

## Task 269G Statement Boundary

The existing exact `given` lower output is consumed byte-for-byte; this module
adds no statement, condition, label, formula, or use-site row. The checker
binding handoff identifies the enclosing proof only. Statement semantics and
all proof effects remain deferred.

The implemented transaction preserves this boundary. Its proof context is
authenticated only as provenance; no statement, condition, label, formula,
fact, thesis, conclusion, or proof row is emitted.

## Task 269GT Statement Boundary

The type composite authenticates the existing proof statement context only as
binding provenance. It adds no statement, condition, label, formula, fact,
thesis, conclusion, or proof row and does not reinterpret the `such that`
condition.

### Task 269GT implemented statement boundary

No statement owner or statement/proof hint is added. The dormant consumer remains outside dispatch, and final assembly accepts the Given-type composite only with empty statement, condition, fact, proof, and semantic inputs.

## Task 269GUP Statement Boundary

The entire `thus y = y;` conclusion is selector-only. GUP adds no statement,
term, conclusion, formula, equality, condition, label, fact, proof hint/state,
or acceptance owner. The private lower/binding route remains outside dispatch.
### Task 269GUP implemented binding profile

The frozen six-file transaction and its exact four checker/four runner tests are implemented. Libraries measure `502/564`; checker/runner production is `30/172531` and `37/74826`, with unchanged path hashes and content hashes `e0342952a01a0b379cf7b06ad243cd40a1656e940480196323cf43fbe7d8f7c5` / `8fe7c8c0b7e855e5113f3830873e133f42c8048a3272055e2fddd5ebd9cbb1bc`.

This closes only dormant private lexical-binding evidence and grants zero active corpus, trace, type, term/use, condition/fact, goal/proof, obligation, diagnostic, or CLI credit. Task 269GUPT is next; Task 269GU, capture, and Task 270 remain deferred.

## Task 269GUPT Statement Boundary

GUPT owns only the written type on the already authenticated `given` declaration. The `such that G: thesis` condition, label, conclusion equality, later `y` leaves, proof goal, and acceptance remain selector-only with no statement/fact/semantic row. Existing statement APIs and production dispatch are byte-identical.

### Task 269GUPT implemented statement boundary

Only the authenticated written source type is transported. Statement APIs,
the condition/label/equality/later-use subtrees, proof state, acceptance, and
production dispatch remain unchanged.

## Task 269GU Statement Boundary

GU consumes only the two `TermReference` leaves inside `thus y = y;`.
`TermExpression` wrappers, `BuiltinPredicateApplication`, equality formula,
proposition, `ConclusionStatement`, `such that` condition/label, proof block,
goal, facts, and acceptance remain selector-only. Existing statement APIs and
production dispatch stay unchanged.

### Task 269GU implemented statement boundary

Only the two primary-term/reference leaves are transported. Statement APIs,
the condition/label/equality/formula shells, facts, proof state, acceptance,
and production dispatch remain unchanged.

## Task 269GCP Frozen Statement Exclusion

The selector authenticates the Given statement, condition list, labeled
proposition, equality subtree, and final conclusion only to prove exact source
identity. GCP publishes none of them through `source_statement`; condition and
label facts, statement semantics, proof state, and acceptance remain absent.

### Task 269GCP implemented statement exclusion

The runner now authenticates the exact Given/condition statement subtrees only
as selector evidence. No `source_statement` payload, condition/label fact,
assumption, proof state, or acceptance result is published.

## Task 269GC Frozen Statement Exclusion

GC uses theorem/proof/Given/segment/name ranges only to authenticate and scope
the lexical binding. It adds no `source_statement` handoff, proposition,
condition list, label, fact, assumption, conclusion, proof-state, or acceptance
row. The two condition occurrences remain GCU-owned after GCT.

### Task 269GC implemented statement exclusion

The binding handoff is implemented using only authenticated ranges. No
`source_statement` payload, condition/label fact, assumption, conclusion,
proof state, or acceptance row changed. The condition occurrences remain
opaque and GCU-owned after GCT.

## Task 269GCT Frozen Statement Exclusion

GCT authenticates the GCP/GC statement only as a source/type dependency. It
adds no statement handoff, proposition, condition list, label, fact,
assumption, conclusion, proof state, or acceptance row. The condition equality
and both witness occurrences remain opaque and GCU-owned. No change to
`source_statement.rs` is permitted.

### Task 269GCT implementation status

After documentation prerequisite `b43081161b31fcc4bc23ac2fd42c5c42e772ab78`,
the exact seven-file implementation and four checker/four private runner tests
are present. The new public checker family is
`SourceProofLocalGivenConditionType{Handoff,Producer,Error}`; Typed and
Resolved own the same boxed composite atomically. Libraries are `518/584`.
Checker production is `30/179612`, with unchanged path hash
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5`
and content hash
`8078ee6235c8ca52ce8cdba0be9a347231260d3421c54625a3fc96cf395c9718`.
Runner production is `37/77159`, with unchanged path hash
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`
and content hash
`5b0e68f35d37fcf843f7cb64885f09bfa9dd5423c17506713e096811a5ddf689`.
Raw/normalized test-list hashes are checker
`6d10b524115a209f198bc5085a726bc1fcc6f92dc3e25a8056e29975b708b656` /
`502f7535a34b9d2224c67e6db15f4eaf45f05eec2a2fe4c914704ecf162d89b2`
and runner
`d599bd69654d000f44858942cec771742d8c3c9e0d2ca459d7fecc84d76752c9` /
`bc3cdabbc6424b0f01d817ed323dd823ff57d1d8d4261220dc3d9c37d9004a61`.

The implementation changes no canonical specification, `.miz`, fixture,
sidecar, expectation, trace row/status/backlink, metadata, diagnostic, public
dispatch, CLI byte, active result, or semantic credit. GCU still owns both
condition occurrences and every wider semantic effect. Independent test-sufficiency, implementation, source/documentation, and
final-quality reviews report **NO FINDINGS**. All nine hard gates PASS with no
score cap at `100/100`; focused and crate suites, lint policies, formatting,
Clippy, workspace tests, metadata, all five CLIs, count/hash oracles, and diff
checks pass. Dedicated implementation commit
`d6fb0ed28ced4d4706a1793b3aedd2a20eea0749` is complete.

## Task 269GCU Frozen Statement Deferral

The unchanged 54-node Surface tree supplies exact selector evidence for two
`TermReference` leaves at `107..108` and `111..112`. GCU publishes no
statement owner, proposition, condition list, equality/formula target, label,
fact candidate, proof state, conclusion, or acceptance row. The Given
statement and all enclosing/subsequent statement structure remain opaque to
this term/reference consumer.

### Task 269GCU implementation status

After documentation prerequisite `15f47a837bc2f52d4cd30e8a4dcb86c16f2961d3`,
the seven frozen implementation files, one `cfg(test)`-only predecessor
ownership-sentinel support file, and four checker/four private runner tests are
present. The support seam closes the review-discovered Task-269A both-order
`test_gap` without changing production API or behavior. The public family is
`SourceProofLocalGivenConditionUseTerm{Handoff,Producer,Error}`; Typed and
Resolved own the same boxed composite atomically. Libraries are `522/588`.
Checker production is `30/181154`, with unchanged path hash
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5`
and content hash
`f9901821c2242bfe66321c57982b54b78425c7940c5a7c47c93c43a8c2c035dc`.
Runner production is `37/77435`, with unchanged path hash
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`
and content hash
`0651af8339c147d04f88be237f8f49fc716b7da3ff90238be50a9527e89992b7`.
Raw/normalized test-list hashes are checker
`d453ca1e8a7cf9870f14a0f933451ca201c19cc8c8367d51767c40a941766f82` /
`7cd84f6cd8e6d1070b39be9e5f1031512cc2c1b664829f10d337f1b67bcb74b3`
and runner
`7a99bcbb35838b6c1df31dec7b7c70d9c569df86bdc6f5c68d72f41578be2a9e` /
`e49dac17564f330ad5c73018538bf5736720e47f4833709c1b9d36622208888a`.

The implementation closes only the two frozen own-condition `y` term/reference
occurrences. The authoritative block-scope decision makes a `given` binding
visible through the remainder of its innermost block and descendant blocks,
subject to inner shadowing, but descendant-use/capture implementation remains
a separate successor. No canonical specification, `.miz`, fixture, sidecar,
expectation, trace row/status/backlink, metadata, diagnostic, public dispatch,
CLI byte, active result, or semantic credit changed. Equality/formula/fact,
guard, goal, proof/obligation/acceptance, export/capture enforcement,
downstream IR, and Task 270 remain deferred. Independent test-sufficiency,
implementation, and source/documentation reviews report **NO FINDINGS**.
Final read-only quality reports **NO FINDINGS**: all nine hard gates PASS
without a score cap at `100/100`. Focused and full measured gates pass.
Exact staging and implementation commit f984ae683419944493c07723e9950a9101a46502 are complete.

## Task 269SDP Source-statement Ownership

`source_statement.rs` is the sole production owner of the later SDP lower
selector. It may validate exact source bytes, all 68 Surface nodes/token
membership, two declaration shells, theorem symbol/definition/contribution,
the Given/now/two-Set subtree ranges, and deterministic debug bytes. It may
not create a statement semantic row, proof context, label/fact, block result,
binding context, term/reference, capture, or diagnostic. All existing
statement producers and active routes remain byte-for-byte isolated.

## Task 269SDP Implementation Status

Documentation prerequisite `f468b0163bb00726dca9b356f48790c73bb1fe98` is
complete. The frozen four Rust files and four exact tests now implement only
the dormant lower projection. Focused `4/4` and runner-library `592/592`
tests pass; independent test-sufficiency and implementation reviews report
**NO FINDINGS**.

Checker/runner libraries are `522/592`. Checker production remains
`30/181154`, path/content SHA-256
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`f9901821c2242bfe66321c57982b54b78425c7940c5a7c47c93c43a8c2c035dc`.
Runner production is `37/79025`, path/content
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d` /
`313843b1f4f2e210588410de2e1440f1263711fc6cad4085a943d467d5c6ba5a`.
Runner module sizes are source-statement `23936`, facade `959`, root
`2792`, and proof-local test leaf `9648`. Raw/normalized test-list hashes are
checker `d453ca1e8a7cf9870f14a0f933451ca201c19cc8c8367d51767c40a941766f82` /
`7cd84f6cd8e6d1070b39be9e5f1031512cc2c1b664829f10d337f1b67bcb74b3`
and runner `40f4271712d7fed6ed238a2e03b61511fc26914af52333b12732824e740ead4a` /
`e9e4f359a571a1aa383168ff6950568788ecffcea2c4eb5d85934fd4ee15e147`.

Corpus/requirements `428/395`, pass/fail `235/193`, warnings/errors `23/0`,
stages `101/7/205/1`, type `259=247+12`, trace hash
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`,
fixtures, sidecars, expectations, metadata, five CLI bytes, diagnostics,
dispatch, and active results remain unchanged. SDP publishes zero checker,
`BindingEnv`, type, term/reference, capture/closure, fact, proof, obligation,
or coverage credit. The next task is the separate Given-plus-descendant
context/binding consumer; occurrence remains later, and `z`/`q` capture stays
blocked by the Chapter-4/15 `set` `spec_gap`. The implementation self-hash is
pending its task-only commit.
