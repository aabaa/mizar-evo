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
| `SourceStatementKind` | `#[non_exhaustive]`; Task 258A accepts `TheoremProposition`, while Task 258B1 additionally accepts exact `ProofStepProposition` and `Conclusion` rows. |
| `SourceStatementRecovery` | `#[non_exhaustive]`; callers tolerate `Degraded`, while this exact route accepts only `Normal`. |
| `SourceStatementFormulaTarget` | `#[non_exhaustive]`; Task 258A accepts only one Task-256 `Atomic` target. |
| `SourceStatementInputFactKind` | `#[non_exhaustive]`; Task 258A accepts only `ReservedTypeGuard`. |
| `SourceStatementCandidateFactKind` | `#[non_exhaustive]`; Task 258A accepts only `UnverifiedProposition`. |
| `SourceStatementLabelKind` | `#[non_exhaustive]`; Task 258B1 accepts only one resolver-authenticated `ProofStep` label. |
| `SourceStatementCitationKind` | `#[non_exhaustive]`; Task 258B1 accepts only one `SimpleLocal` backward citation. |
| `SourceStatementError` | `#[non_exhaustive]`; callers must not exhaustively match producer/installation failures. |
| `SourceStatementReferenceError` | `#[non_exhaustive]`; callers must not exhaustively match reference dependency, aggregate, label, or citation failures. |

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
    pub label: SourceStatementLabelId,
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
    pub const fn label(&self) -> SourceStatementLabelId;
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
pub enum SourceStatementCitationKind {
    SimpleLocal,
}
```

Both IDs have the existing dense-ID derives and `new`/`index` accessors.
Inputs, immutable rows, tables, and handoff derive `Debug, Clone, PartialEq,
Eq`; the two enums use the existing public data-enum derives and are
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
