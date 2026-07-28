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
| `SourceStatementKind` | `#[non_exhaustive]`; Task 258A accepts only `TheoremProposition`. |
| `SourceStatementRecovery` | `#[non_exhaustive]`; callers tolerate `Degraded`, while this exact route accepts only `Normal`. |
| `SourceStatementFormulaTarget` | `#[non_exhaustive]`; Task 258A accepts only one Task-256 `Atomic` target. |
| `SourceStatementInputFactKind` | `#[non_exhaustive]`; Task 258A accepts only `ReservedTypeGuard`. |
| `SourceStatementCandidateFactKind` | `#[non_exhaustive]`; Task 258A accepts only `UnverifiedProposition`. |
| `SourceStatementError` | `#[non_exhaustive]`; callers must not exhaustively match producer/installation failures. |

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
