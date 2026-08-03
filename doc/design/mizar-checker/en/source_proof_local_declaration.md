# Source Proof-Local Declaration Transport

> Canonical language: English. Japanese companion:
> [../ja/source_proof_local_declaration.md](../ja/source_proof_local_declaration.md).

## Status and authority

This document freezes **Checker Task 269A**, the first dependency-minimal
slice of queue Task 269. English is canonical. The matching Japanese document
must remain synchronized in the same logical task.

The normative authority is:

1. `doc/spec/en/04.variables_and_constants.md` §§4.1, 4.4.3, and 4.6;
2. `doc/spec/en/15.statements.md` §15.4.4;
3. `doc/spec/en/16.theorems_and_proofs.md` §16.4;
4. the exact already-implemented Task-258B3N source/statement/witness/term
   transport and its parser/resolver provenance;
5. Tasks 248--258 public APIs, especially `LocalTermBinding`, `BindingEnv`,
   `SourcePrimaryTermHandoff`, `SourceStatementHandoff`, and
   `SourceStatementWitnessHandoff`.

The broad proof-local declaration gap fixture
`fail_type_elaboration_proof_local_declaration_gap_001.miz`, its sidecar, and
its existing covered diagnostic-gap trace rows remain read-only. Those rows do
not credit positive proof-local binding semantics. The fixture mixes `let`,
`given`, `consider`, `set`, and `reconsider`, so it cannot safely represent
this named-witness-only slice. No blocking `spec_gap` exists for the frozen
slice.

Selection inventory is HEAD
`52cf07be3c77d3aa2a797a7681ed9cbabf88295b` on `main`, clean before this docs
edit, `origin/main...HEAD = 0/19`, with protected `stash@{0}` fixed at
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`. The origin divergence is a
report-only `repo_metadata_conflict`; it does not obscure the task-only target
and is not repaired.

## Classification and task selection

Fresh inventory classifies the absence of a checker-owned binding transaction
for an already authenticated named witness as `source_drift`, the absent
contract as `design_drift`, and the missing exact producer/ownership/consumer
tests as a canonical-derived `test_gap`. Treating the name token as a public
symbol, reconstructing it from checker-side syntax, assigning witness typing,
or publishing proof results would be a `boundary_violation`.

Task 269A is dependency-ready after Tasks 248--258. It is deliberately smaller
than Task 269: it creates and links one named-witness local binding but does
not implement a later use of that binding. A later Task-269 slice retains
`let`/`set`/`given`/`consider`, multiple introductions, later-use replay, and
capture-by-resolved-binding coverage. Task 270 retains `deffunc`/`defpred`,
Task 271 retains `reconsider`, and Task 272 retains existential witness
matching, witness-type obligations, and goal substitution.

## Exact source and lower profile

The only admitted source is the existing private Task-258B3N text:

```mizar
reserve x for set;
theorem FormulaStatementNamedWitnessSmoke: x = x proof
  take y = x;
  thus x = x;
end;
```

It is exactly 107 bytes including the final newline and has SHA-256
`a57022c4b75991dd4308943477e03819f5bfe2c0d23ea1030730256252d7d329`.
The normal Surface AST has 51 nodes and root 50 with no recovery or diagnostic.
The exact name, witness, and take sites are node/range `13/81..82`,
`36/81..86`, and `37/76..87`. The RHS is
`SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId(2))`, site 34,
range `85..86`, spelling `x`, source ordinal 2, proof context 1, and its
existing reference resolves to reserved `BindingId(0)` at use ordinal 1.

The required lower cardinalities remain:

- binding contexts/bindings/diagnostics `2/1/0` before Task 269A;
- primary terms/references/numeric requests `5/5/0`;
- atomic formula rows `2/0/0/0/0/0/0/4/4`;
- theorem owners/statements/contexts/input facts/candidate facts `1/2/2/2/2`;
- witnesses/names `1/1`.

The proof context is `BindingContextId(1)`, parent 0, layer `Proof`, lexical
scope `[0]`, normal recovery, local `bindings=[]`, and
`visible_bindings=[BindingId(0)]` before the transaction. Task 269A must
validate all existing lower handoffs and all 51 arena nodes without changing
any node kind, anchor, child list, typing state, recovery state, or link.

## Resolver-local provenance

The private runner supplies exactly one resolver-owned `LocalTermBinding` only
after authenticating the complete source and Surface profile:

| field | exact value |
|---|---|
| spelling | `y` |
| lexical scope | `[0]` |
| declaration range | `81..82` |
| visible-after ordinal | `1` |

The checker consumes this value; it must not parse `"y = x"`, scan tokens, or
manufacture a `SymbolId`, declaration shell, contribution, name reference, or
module symbol. The exact resolver environment continues to contain no visible
module symbol named `y`. `BinderIdentity::ResolverLocal` preserves the four
fields above as the binding provenance.

## Public checker API

Implementation adds syntax-free module `source_proof_local_declaration`. The
following Rust declarations freeze its complete exported family, field
visibility, derives, and signatures; the implementation must not add a second
constructor or mutable table/handoff access:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceProofLocalDeclarationId(usize);

impl SourceProofLocalDeclarationId {
    pub const fn new(index: usize) -> Self;
    pub const fn index(self) -> usize;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalDeclarationHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub declarations: Vec<SourceProofLocalDeclarationInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalDeclarationInput {
    pub witness: SourceStatementWitnessId,
    pub name: SourceStatementWitnessNameId,
    pub rhs: SourceStatementWitnessTermTarget,
    pub binding_context: BindingContextId,
    pub source_ordinal: usize,
    pub local: LocalTermBinding,
    pub kind: SourceProofLocalDeclarationKind,
    pub recovery: SourceProofLocalDeclarationRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceProofLocalDeclarationKind {
    NamedWitness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceProofLocalDeclarationRecovery {
    Normal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalDeclaration {
    witness: SourceStatementWitnessId,
    name: SourceStatementWitnessNameId,
    rhs: SourceStatementWitnessTermTarget,
    binding: BindingId,
    binding_context: BindingContextId,
    source_ordinal: usize,
    visible_after_ordinal: usize,
    kind: SourceProofLocalDeclarationKind,
    recovery: SourceProofLocalDeclarationRecovery,
}

impl SourceProofLocalDeclaration {
    pub const fn witness(&self) -> SourceStatementWitnessId;
    pub const fn name(&self) -> SourceStatementWitnessNameId;
    pub const fn rhs(&self) -> SourceStatementWitnessTermTarget;
    pub const fn binding(&self) -> BindingId;
    pub const fn binding_context(&self) -> BindingContextId;
    pub const fn source_ordinal(&self) -> usize;
    pub const fn visible_after_ordinal(&self) -> usize;
    pub const fn kind(&self) -> SourceProofLocalDeclarationKind;
    pub const fn recovery(&self) -> SourceProofLocalDeclarationRecovery;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalDeclarationTable {
    rows: Vec<SourceProofLocalDeclaration>,
}

impl SourceProofLocalDeclarationTable {
    pub fn get(
        &self,
        id: SourceProofLocalDeclarationId,
    ) -> Option<&SourceProofLocalDeclaration>;
    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (SourceProofLocalDeclarationId, &SourceProofLocalDeclaration),
    >;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalDeclarationHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    base_binding_fingerprint: String,
    statement_fingerprint: String,
    witness_fingerprint: String,
    primary_term_fingerprint: String,
    binding_env: BindingEnv,
    final_binding_fingerprint: String,
    declarations: SourceProofLocalDeclarationTable,
}

impl SourceProofLocalDeclarationHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn base_binding_fingerprint(&self) -> &str;
    pub fn statement_fingerprint(&self) -> &str;
    pub fn witness_fingerprint(&self) -> &str;
    pub fn primary_term_fingerprint(&self) -> &str;
    pub const fn binding_env(&self) -> &BindingEnv;
    pub fn final_binding_fingerprint(&self) -> &str;
    pub const fn declarations(&self) -> &SourceProofLocalDeclarationTable;
    pub fn debug_text(&self) -> String;

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        statements: &SourceStatementHandoff,
        witnesses: &SourceStatementWitnessHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
    ) -> Result<(), SourceProofLocalDeclarationError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SourceProofLocalDeclarationProducer;

impl SourceProofLocalDeclarationProducer {
    pub fn build(
        input: SourceProofLocalDeclarationHandoffInput,
        statements: &SourceStatementHandoff,
        witnesses: &SourceStatementWitnessHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
    ) -> Result<
        SourceProofLocalDeclarationHandoff,
        SourceProofLocalDeclarationError,
    >;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalDeclarationError {
    InvalidTransaction,
    DependencyMismatch,
    InvalidAggregate,
    InvalidDeclaration {
        declaration: SourceProofLocalDeclarationId,
    },
    InvalidArena,
    InvalidBindingEnvironment,
    InvalidInstallation,
}
```

`SourceId`, `ModuleId`, `LocalTermBinding`, `BindingContextId`, `BindingId`,
`BindingEnv`, the five `SourceStatement*` types, `SourcePrimaryTermHandoff`,
and `TypedArena` above are the existing types from their current owner modules;
the new module defines no aliases or replacements for them.

No parser or syntax type crosses this API. No caller can supply a final
`BindingId`; dense identity is assigned transactionally by the checker.

## Exact output transaction

The exact input has one declaration row:

```text
declaration#0 kind=named-witness witness=0 name=0 rhs=primary#2
context=1 source_ordinal=1 local=("y", scope=[0], range=81..82,
visible_after=1) recovery=normal
```

The producer clone-preserves the complete Task-258B3N base environment and
constructs a replacement proof context plus one appended binding. The exact
post-transaction profile is contexts/bindings/diagnostics `2/2/0`:

- `BindingId(0)` is byte-identical to the reserved `x` row;
- `BindingId(1)` has spelling `y`, kind `LocalAbbreviation`, owner context 1,
  declaration range `81..82`, visible-after ordinal 1,
  `BindingTypeSite::Missing`, `BindingStatus::Active`, empty captured
  variables, empty diagnostics, and normal recovery;
- its identity is exact `ResolverLocal(scope=[0], ordinal=1,
  declaration_range=81..82)`;
- context 0 is byte-identical;
- context 1 becomes `bindings=[1]`, `visible_bindings=[0,1]`, with every other
  field byte-identical.

The declaration row links witness 0, name 0, RHS primary term 2, and binding
1. At source ordinal 1 a lookup of `y` is still a forward reference; at a
later ordinal in the same scope it resolves to binding 1. This ordering
prevents the RHS from capturing the binding being defined. Task 269A records
the definition-site link only; it does not rewrite or expand a later term.

## Fingerprints and validation order

The handoff retains exact byte fingerprints of:

1. `statements.binding_env().debug_text()` before extension;
2. `statements.debug_text()`;
3. `witnesses.debug_text()`;
4. `primary_terms.debug_text()`;
5. the final extended `BindingEnv::debug_text()`.

Installation and final assembly recompute all five. Equal cardinality is not
sufficient. Validation order is stable:

1. source/module transaction identity;
2. lower source/module/fingerprint equality and exact Task-258B3N profile;
3. exact one-row aggregate and dense IDs;
4. resolver-local spelling/scope/range/ordinal and row links;
5. all-51-node arena/subtree replay, including unchanged Task-258B3N owner
   kinds;
6. base-to-final binding-environment reconstruction and lookup behavior;
7. typed or final one-shot installation invariants.

The first failing class is returned. Phases 1--7 map respectively to
`InvalidTransaction`, `DependencyMismatch`, `InvalidAggregate`,
`InvalidDeclaration { declaration: SourceProofLocalDeclarationId(0) }`,
`InvalidArena`, `InvalidBindingEnvironment`, and `InvalidInstallation`.
The exact `Display` texts are:

| variant | exact text |
|---|---|
| `InvalidTransaction` | `source proof-local declaration transaction is invalid` |
| `DependencyMismatch` | `source proof-local declaration dependency mismatch` |
| `InvalidAggregate` | `source proof-local declaration aggregate is invalid` |
| `InvalidDeclaration { declaration }` | `source proof-local declaration <declaration.index()> is invalid` |
| `InvalidArena` | `source proof-local declaration arena is invalid` |
| `InvalidBindingEnvironment` | `source proof-local declaration binding environment is invalid` |
| `InvalidInstallation` | `source proof-local declaration installation is invalid` |

The error implements `std::error::Error`. Failure publishes no partial
binding, context, handoff, debug suffix, or final owner.

## Stable debug grammar

The new block is appended after the existing statement-witness block and does
not change any legacy debug byte:

```text
source-proof-local-declaration-debug-v1
module: <package>::<path>
base-binding-fingerprint: <quoted debug bytes>
statement-fingerprint: <quoted debug bytes>
witness-fingerprint: <quoted debug bytes>
primary-term-fingerprint: <quoted debug bytes>
declaration#0 kind=named-witness witness=0 name=0 rhs=primary#2 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal
final-binding-fingerprint: <quoted debug bytes>
```

Rows are dense-ID ordered. Enum spellings are exactly `named-witness` and
`normal`; RHS spelling is `primary#<id>`. Each fingerprint value is formatted
with Rust string `Debug` (`{:?}`): it is double-quoted and uses the standard
backslash escapes for embedded newlines, quotes, backslashes, and control
characters. The lines occur in exactly the shown order, there are no blank
lines, and the block has exactly one final LF. Empty legacy profiles emit no
new block.

## Typed and final ownership

`TypedAst` adds one private optional
`SourceProofLocalDeclarationHandoff` and exactly these two methods:

```rust
pub const fn source_proof_local_declaration(
    &self,
) -> Option<&SourceProofLocalDeclarationHandoff>;

pub fn with_source_proof_local_declaration(
    self,
    handoff: SourceProofLocalDeclarationHandoff,
) -> Result<Self, TypedAstError>;
```

`TypedAstParts` receives no replacement field. Installation is permitted only
over the exact already installed Task-258B3N source term, atomic formula,
statement, and witness bundle with otherwise empty semantic tables. It
revalidates every fingerprint, row, arena node, and binding transition before
atomic publication. `TypedAstError` gains exactly the unit variant
`InvalidSourceProofLocalDeclaration`.

`ResolvedTypedAst` clone-preserves the optional handoff from `TypedAst`, adds
only the same exact `source_proof_local_declaration` getter signature, and
replays the same validation. There is no `ResolvedTypedAstInputs` replacement
path. `ResolvedTypedAstError` gains exactly the unit variant
`InvalidSourceProofLocalDeclaration`; orphan, duplicate, stale,
same-length-corrupt, or half-installed values fail with it. Existing
Task-258B3N ownership and node hints remain unchanged; Task 269A adds no arena
node or node role.

## Private runner consumer and exclusions

`mizar-test` adds one private dormant Task-269A leaf. It selects only the exact
107-byte B3N source, calls the existing Task-258B3N producer first, constructs
the exact resolver `LocalTermBinding`, calls the checker producer, installs the
handoff, and reassembles the final AST. It is not wired into public corpus
dispatch, expectation selection, diagnostic serialization, or any CLI route.
The existing Task-258B3N route and its debug bytes remain unchanged.

The selector excludes every near miss, unnamed or multiple witness, different
RHS, recovery node, different lexical scope/range/ordinal, `let`, `set`,
`given`, `consider`, `reconsider`, `deffunc`, `defpred`, imported symbol, and
the broad proof-local gap fixture.

## Semantic deferrals

Task 269A publishes no inferred witness type, type table entry, coercion,
diagnostic, initial obligation, equality fact, existential match, goal or
guard composition, goal substitution, proof node, discharge, acceptance,
theorem fact, Core IR, control-flow IR, or VC. `BindingTypeSite::Missing` is a
representation boundary, not successful type inference.

Task 269B+ retains later-use and capture replay plus the other proof-local
declaration forms. Task 270 retains proof-local functional/predicative
abbreviations, Task 271 retains reconsideration, and Task 272 retains named
witness typing and existential-goal effects. No semantic behavior may be
derived from current source behavior in those tasks.

## Tests and count impact

The checker implementation adds exactly four library tests:

1. exact construction, binding row/context transition, definition-site links,
   stable debug, and ordinal lookup behavior;
2. transaction/dependency/aggregate/resolver/row/arena/fingerprint corruption
   precedence and rollback;
3. typed one-shot ownership, missing/orphan/duplicate/same-length corruption,
   unchanged arena, and legacy Task-258B3N debug compatibility;
4. final clone/replay, typed/final equality, sibling isolation, and proof/type/
   fact/diagnostic/IR/VC non-publication.

The private runner adds exactly four tests for the exact frontend consumer,
resolver/local and lower mutation matrices, near-miss/route isolation, and
typed/final replay with empty semantics. Projected library counts are checker
`478 -> 482` and runner `532 -> 536`; resolver/syntax counts remain `148/59`.
Checker and runner production manifests each gain one source path; exact line,
path-hash, content-hash, raw-test-list, and normalized-test-list values are
remeasured after implementation.

The docs prerequisite and implementation add no `.miz` fixture, sidecar,
expectation, trace row/backlink/status, metadata case, active outcome,
diagnostic code/key, or CLI output. Corpus/requirements remain `428/395`,
pass/fail `235/193`, stages `101/7/205/1`, type coverage `259=247+12`,
warnings/errors `23/0`, and trace SHA-256 remains
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceProofLocalDeclarationKind` | `#[non_exhaustive]`; callers must tolerate later explicitly frozen proof-local declaration forms. |
| `SourceProofLocalDeclarationRecovery` | `#[non_exhaustive]`; callers must tolerate later recovery classes. |
| `SourceProofLocalDeclarationError` | `#[non_exhaustive]`; callers must not exhaustively match validation or installation failures. |

No exhaustive public enum exceptions are owned by this module.

Implementation also updates the existing checker lint-policy module/doc/public-
surface inventories for the new exported module. Those guard edits add no test
case and are part of the frozen implementation scope; the source/spec export
inventory changes only in the implementation commit, when `lib.rs` actually
exports the module.

## Exit criteria

Task 269A is complete only when:

1. the exact producer and post-binding environment match this contract;
2. typed and final ownership are atomic and all legacy Task-258B3N bytes stay
   stable;
3. the private consumer and exactly eight tests pass without corpus/trace
   activation;
4. focused, crate, lint-policy, metadata, formatting, warnings-denied Clippy,
   workspace, all CLI, count/hash, and whitespace gates pass;
5. test-sufficiency, implementation, source/documentation, and final quality
   reviews end **NO FINDINGS**, all nine hard gates PASS without a score cap,
   and quality is at least 90/100;
6. one implementation commit contains only the frozen Task-269A scope, then
   fresh inventory continues automatically to the next dependency-ready
   Task-269 slice.
