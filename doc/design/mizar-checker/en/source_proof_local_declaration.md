# Source Proof-Local Declaration Transport

> Canonical language: English. Japanese companion:
> [../ja/source_proof_local_declaration.md](../ja/source_proof_local_declaration.md).

## Status and authority

This living owner document freezes the completed Checker Task-269 proof-local
declaration lineage from Tasks 269A/B/CP/C/CT through GP/GS/G/GT/GUP/GUPT/GU,
GCP/GC/GCT/GCU, plus the current lower-only **Task 269SDP** contract. English
is canonical. The matching Japanese document must remain synchronized in the
same logical task.

The normative authority is:

1. `doc/spec/en/03.type_system.md` §§3.1--3.4;
2. `doc/spec/en/04.variables_and_constants.md` §§4.1, 4.2, 4.4 (especially
   4.4.1 and 4.4.3), and 4.6 (especially 4.6.1 and 4.6.2);
3. `doc/spec/en/08.type_inference.md` §§8.1 and 8.3;
4. `doc/spec/en/13.term_expression.md` §§13.1.1 and 13.8.1;
5. `doc/spec/en/15.statements.md` §§15.2.1--15.2.2, 15.3.3, 15.4.4,
   15.6.1, 15.10, 15.11.1--15.11.2, and 15.11.4;
6. `doc/spec/en/16.theorems_and_proofs.md` §§16.3.3, 16.4.1--16.4.3,
   and 16.5, with §16.5 retained only for the historical syntax/justification
   boundary and not as proof-justification ownership;
7. the exact already-implemented Task-258B3N source/statement/witness/term
   transport and its parser/resolver provenance;
8. the parser simple/block statement fixtures, the broad proof-local
   declaration fixture, the mixed predicate/functor boundary fixture, and
   their unchanged sidecars/trace metadata; and
9. Tasks 248--259 public APIs, especially `LocalTermBinding`, `BindingEnv`,
   `SourcePrimaryTermHandoff`, `SourceStatementHandoff`, and
   `SourceStatementWitnessHandoff`.

The broad proof-local declaration gap fixture
`fail_type_elaboration_proof_local_declaration_gap_001.miz`, its sidecar, and
its existing covered diagnostic-gap trace rows remain read-only. Those rows do
not credit positive proof-local binding semantics. The fixture mixes `let`,
`given`, `consider`, `set`, and `reconsider`, so it cannot safely represent
any exact Task-269 slice by itself. No blocking `spec_gap` exists for the
completed slices or lower-only Task 269SDP. The Chapter-4/15 conflict over
later `set` effects remains nonblocking for SDP syntax transport and blocking
for every capture/closure consumer.

The historical Task-269A selection inventory was HEAD
`52cf07be3c77d3aa2a797a7681ed9cbabf88295b` on `main`, clean before this docs
edit, `origin/main...HEAD = 0/19`, with protected `stash@{0}` fixed at
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`. The origin divergence is a
report-only `repo_metadata_conflict`; it does not obscure the task-only target
and is not repaired.

The current Task-269SDP selection inventory is HEAD
`f984ae683419944493c07723e9950a9101a46502` on `main`, clean before the SDP
documentation edit, with the same report-only origin divergence `0/19` and
the same protected stash identity.

## Historical Task 269A classification and task selection

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

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        statements: &SourceStatementHandoff,
        witnesses: &SourceStatementWitnessHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
        installation_available: bool,
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
`validate_complete_installation` is crate-private integration surface: it
replays phases 1--6 through `validate_installation`, then maps a false owner
availability flag to phase-7 `InvalidInstallation`. Typed/final owners map
that internal error to their dedicated AST error and publish nothing.

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

Repeated specification review of the narrowed contract reports **NO
FINDINGS**. Docs-only preflight passes focused Task-269CP/C/CT tests,
checker/runner lint `15/14`, metadata `137`, Cargo metadata, format,
warnings-denied workspace Clippy, full workspace tests, all five CLIs, and
whitespace. Libraries remain `490/548`; production remains
`30/168322` and `37/71647`, with path/content hashes
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`4d0c793a47dac672e5f395c9c2b9e7c9274b5d776b54870888ba5c918f751dc2`
and
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d` /
`0f8f5926b9bee23c92d1f05e9cc9e85b4c0561b543e9e0a1e4c825f43b6c5798`.
Raw/normalized test-list hashes remain checker
`10e1f56783a472b63a0473893196d68b54a7a7aa3a3aff4f66e74ac42b4a2ad2` /
`21d65f467319e2e7ac463344902b10dfce5716a96c41a87e879326c293ff36e0`
and runner
`cd47be81d6e0987a4461191b700c442c3182fb9f35fe6ab6e2d216ba122fd841` /
`e24bc08e3c8207ba96b6df3de995a3b489e333f8599233c1eded9f81fe696a77`.
Plan/parse/declaration/type/proof hashes remain
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.

Repeated source/docs consistency and final-quality reviews report **NO
FINDINGS**. All nine hard gates PASS without a score cap at `100/100`
(`20/20/15/15/10/10/5/5`). Exact staging and the dedicated docs commit remain
parent-owned.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceProofLocalDeclarationKind` | `#[non_exhaustive]`; callers must tolerate later explicitly frozen proof-local declaration forms. |
| `SourceProofLocalDeclarationRecovery` | `#[non_exhaustive]`; callers must tolerate later recovery classes. |
| `SourceProofLocalDeclarationError` | `#[non_exhaustive]`; callers must not exhaustively match validation or installation failures. |
| `SourceProofLocalLetBindingRecovery` | `#[non_exhaustive]`; callers must tolerate later explicitly frozen proof-`let` recovery classes. |
| `SourceProofLocalLetBindingError` | `#[non_exhaustive]`; callers must not exhaustively match proof-`let` validation or installation failures. |
| `SourceProofLocalGivenBindingRecovery` | `#[non_exhaustive]`; callers must tolerate later explicitly frozen proof-`given` recovery classes. |
| `SourceProofLocalGivenBindingError` | `#[non_exhaustive]`; callers must not exhaustively match proof-`given` validation or installation failures. |
| `SourceProofLocalGivenUseBindingError` | `#[non_exhaustive]`; callers must not exhaustively match proof-`given` later-use-profile validation failures. |
| `SourceProofLocalGivenConditionBindingError` | `#[non_exhaustive]`; callers must not exhaustively match proof-`given` declaration-condition binding validation or installation failures. |
| `SourceProofLocalGivenDescendantBindingError` | `#[non_exhaustive]`; callers must not exhaustively match proof-`given` descendant binding/context validation or installation failures. |

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

## Implementation result

The frozen module, API, producer, five fingerprints, `2/1/0 -> 2/2/0`
transition, ordinal lookup replay, Typed/final ownership, dormant runner leaf,
and exact eight compound tests are implemented. Checker/runner libraries are
`482/536`; production inventories are `30/164419` and `37/69729`. The exact
fixture/corpus/trace/metadata/CLI no-op and all semantic deferrals are
preserved. Independent reviews ended **NO FINDINGS**, all nine hard gates
passed without a score cap at `100/100`, full verification passed, and
implementation commit `f548ceb9f1acbeca72919809f2a1db84da213982` preserved a
clean worktree, report-only origin divergence `21/0`, and the protected stash.
Fresh inventory selected Task 269B below.

## Task 269B Frozen Mixed-Witness Binding Increment

### Selection, authority, and classification

Task 269B is the next dependency-ready Task-269 slice because the complete
Task-258B3M1 lower transport is already public and verified. Canonical
authority is Chapter 4 §§4.4.3 and 4.6, Chapter 15 §15.4.4 including its
left-to-right syntax-order note, Chapter 16 §16.3.3 item 5 and §16.4, the
existing `pass_parser_simple_statements_001.miz` parser fixture, the frozen
Task-258B3M1 source/AST/statement/witness contract, and the committed Task-269A
API. The broad proof-local gap fixture, expectation, and trace rows remain
read-only diagnostic authority and grant no positive coverage credit.

The missing B3M1 binding transaction is `source_drift`; the previously
open-ended Task-269B+ ownership is `design_drift`; absent exact-profile tests
are a canonical-derived `test_gap`. There is no blocking `spec_gap` or lower-
stage defect: parser diagnostics are zero, structural resolver lowering and
private theorem-owner enrichment are stable, and Task-258B3M1 publishes every
required lower handoff. Treating the unnamed second witness as a binding,
assigning left-to-right existential goal effects, or adding typing/proof
results would be a `boundary_violation`.

Fresh selection inventory is HEAD
`f548ceb9f1acbeca72919809f2a1db84da213982`, branch `main`, clean worktree,
`origin/main...HEAD = 0/21`, and protected `stash@{0}`
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`. Origin divergence remains a
report-only `repo_metadata_conflict`; it is not repaired.

### Exact admitted source and lower transaction

The second admitted source is exactly the existing 113-byte final-LF
Task-258B3M1 text, SHA-256
`412a6a7f8fddebd67418f3482855ea89a1e7da922b42ebb93463971d8e49c186`:

```mizar
reserve x for set;
theorem FormulaStatementMultipleWitnessSmoke: x = x proof
  take y = x, x;
  thus x = x;
end;
```

It has zero parser diagnostics, 56 unrecovered Surface/Typed nodes, root 55,
one current-module theorem owner, and no import, citation, proof-step label,
or witness-name symbol. Exact sites are name `13/84..85`, named witness
`38/84..89`, take `42/79..93`, and RHS primary term 2 at `36/88..89`. Lower
profiles are binding `2/1/0`, primary `6/6/0`, atomic
`2/0/0/0/0/0/0/4/4`, statement `1/2/2/2/2`, and witness/name `2/1`.

Witness 0 is `Named`, links name 0 and `Primary(2)`, has source/within-take
ordinals `1/0`, and spelling `y = x`. Witness 1 is `Unnamed`, has no name,
links `Primary(3)`, has ordinals `1/1`, and spelling `x`. Task 269B creates
only the declaration for witness 0. Witness 1 remains immutable lower syntax
and receives no checker binding.

The runner supplies exact resolver-local `y`, scope `[0]`, declaration range
`84..85`, and visible-after ordinal 1 after authenticating all source and lower
bytes. The checker transaction remains declaration 0, witness/name/RHS
`0/0/2`, proof context 1, source ordinal 1, `NamedWitness`, normal recovery.
It changes only contexts/bindings/diagnostics `2/1/0 -> 2/2/0`: binding 1 is
`LocalAbbreviation`, `ResolverLocal([0],1,84..85)`, missing type site, active,
uncaptured, diagnostic-free, and context 1 becomes bindings `[1]`, visible
bindings `[0,1]`. Definition-site ordinal 1 remains a forward reference and
same-scope ordinal 2 resolves binding 1.

### API, fingerprints, ownership, and validation

Task 269B adds no public type, field, enum variant, error, method, installer,
debug line, module, or source path. It reuses the complete Task-269A API and
its five fingerprints: base `BindingEnv`, statement, witness, primary-term,
and final `BindingEnv` debug bytes. The handoff carries no profile tag. Exact
lower cardinalities, fingerprints, declaration range, and all-node replay
jointly distinguish B3N from B3M1.

The seven validation phases remain unchanged. Phase 2 accepts only a complete
exact B3N or B3M1 lower profile; phase 3 still requires one proof-local
declaration although B3M1 has two witness rows; phase 4 requires declaration 0
to link only B3M1 `0/0/2` and exact resolver provenance; phase 5 replays all
56 nodes; phase 6 reconstructs the exact final environment; phase 7 preserves
Typed/final one-shot installation. Cross-profile source, arena, statement,
witness, primary, or fingerprint mixtures fail atomically in the existing
precedence. B3N output and all public/debug bytes remain unchanged.

The existing private dormant runner leaf gains only an exact B3M1 selector
branch after B3N. It calls the Task-258B3M1 lower route, constructs the
authenticated local value, installs the same handoff, and reassembles empty
semantics. Public dispatch, active corpus selection, metadata, diagnostics,
and all CLI paths remain untouched.

### Exclusions, tests, impact, and exit criteria

Task 269B excludes later-use/capture replay, binding witness 1, additional
named witnesses, other Task-258B3M2 profiles, `let`, `given`, `consider`,
`set`, `deffunc`, `defpred`, `reconsider`, imported spellings, type inference,
coercions, initial obligations, existential matching, goal/guard composition,
substitution, facts, proof/discharge/acceptance, theorem facts, Core IR, CFG,
and VC. The left-to-right rule authenticates dense witness syntax order only;
Task 272 retains all goal effects.

The existing four checker and four runner compound tests are expanded; no new
test function is added. They cover exact B3M1 construction/debug/lookup, both
witness rows with only row 0 bound, all five fingerprints, all 56 nodes,
resolver/local/lower/cross-profile mutations, B3N compatibility, Typed/final
replay, route isolation, and empty semantics. Library counts stay checker/
runner `482/536`; resolver/syntax stay `148/59`. Production path counts stay
`30/37`; lines and path/content/test-list hashes are remeasured after
implementation.

The docs prerequisite and implementation change no `doc/spec`, `.miz`,
sidecar, expectation, trace row/status/count, coverage credit, metadata,
diagnostic, Cargo manifest, public route, active outcome, or CLI output.
Therefore `doc/design/spec_coverage_audit.md` remains unchanged. Corpus and
CLI baselines remain the Task-269A values until remeasurement proves otherwise.

Task 269B is complete only after the docs prerequisite is independently
reviewed to **NO FINDINGS**, committed alone, and followed by fresh preflight;
the exact increment is then implemented, all test/implementation/source-doc
reviews end **NO FINDINGS**, all nine hard gates pass uncapped at 90/100 or
better, all verification/count/hash/staging gates pass, one implementation
commit is created, and fresh inventory selects the next dependency-ready
Task-269 slice.

Completion evidence: [central Task-269B historical contract](../../task_contracts/en/269B.md#completion-evidence).

## Checker Task 269CP Frozen Isolated Proof-`let` Lower Prerequisite

### Selection, authority, and classification

Fresh inventory after Task-269B implementation commit
`afd54a37ce4022929bdaf60be519ac4adbdd9b8e` selects only Task 269CP. The
canonical authority is Chapter 4 Sections 4.2 and 4.6, Chapter 15 Sections
15.2.1, 15.10, and 15.11.1, and Chapter 16 Sections 16.3.3 and 16.4. The
existing parser simple-statement fixture and tests establish the normal
`LetStatement -> QualifiedVariableSegment -> TypeExpression` shape. The
mixed active fixture
`fail_type_elaboration_proof_local_declaration_gap_001.miz`, its sidecar, and
its trace backlinks remain read-only boundary evidence.

The absent isolated lower contract is `design_drift`; the absent private
extractor is bounded `source_drift`; and its missing mutation/isolation tests
are a canonical-derived `test_gap`. There is no blocking `spec_gap`.
Later-use/capture is not dependency-ready because the resolver publishes no
AST-wide local-use/capture table; reconstructing it from checker-side syntax
would be a `boundary_violation`. The local origin/main difference remains a
report-only `repo_metadata_conflict` and does not obscure the exact target.

Task 269CP is a runner-private lower prerequisite for future Task 269C. It
does not extend the checker ABI frozen above, create a checker binding, or
install any Typed/Resolved proof-local handoff. Direct Task-269C selection is
authorized only for a binding-only transaction that retains
`BindingTypeSite::Missing`; it cannot call or extend `SourceTypeProducer`.

### Exact source, Surface profile, and fingerprints

The sole accepted source is the following private final-LF text:

```mizar
reserve x for set;
theorem FormulaStatementLetSmoke: x = x proof
  let y be set;
  thus x = x;
end;
```

It is exactly 100 bytes including one final LF. Its source SHA-256 is
`7860a3fe5af89063ac6a2b9a4465cac36d26f6d64e892ba6e2c89bcbaaf9763a`.
The normal Surface snapshot has SHA-256
`1fc35ec18db82efc0968b2f42b08cfaae678184983210cd26f060d45354c7f68`,
51 nodes, root 50, root range `0..99`, and no recovery or frontend
diagnostic. Exact structural rows are:

| node | kind | range | children |
| ---: | --- | --- | --- |
| 27 | `ReserveItem` | `0..18` | `[0,26,4]` |
| 34 | `TypeHead` | `76..79` | `[15]` |
| 35 | `TypeExpression` | `76..79` | `[34]` |
| 36 | `QualifiedVariableSegment` | `71..79` | `[13,14,35]` |
| 37 | `LetStatement` | `67..80` | `[12,36,16]` |
| 45 | `ConclusionStatement` | `83..94` | `[17,44,21]` |
| 46 | `ProofBlock` | `59..98` | `[11,37,45,22]` |
| 47 | `TheoremItem` | `19..99` | `[5,6,7,33,46,23]` |
| 48/49/50 | `ItemList` / `CompilationUnit` / `Root` | `0..99` | `[27,47]` / `[48]` / tokens `0..23` plus `[49]` |

Name token 13 is exactly `y@71..72`; token 14 is `be@73..75`; type-head
token 15 is `set@76..79`. The declaration ordinal is 1 between theorem
ordinal 0 and conclusion ordinal 2. The proof lexical scope is `[0]`.

### Resolver provenance and private lower output

Resolver preflight has exactly two normal shells: reserve shell 0 at node
27/range `0..18` and theorem shell 1 at node 47/range `19..99`. It produces
one public/exported theorem projection and symbol, definition 0, contribution
0 of kind `LocalSource`, origin path `[2,1]`, and no import, label, overload,
registration, or visible module symbol named `y`. The private extractor may
construct exactly
`LocalTermBinding::new("y", LocalTermScope::new(vec![0]), 71..72, 1)` only
after authenticating that complete provenance.

The implementation owns one crate-private, syntax-free
`SourceProofLocalLetLowerOutput`. It retains source/module identity; the
theorem symbol, definition, and contribution; role-specific ranges for the
theorem, proof, let, segment, name, type expression, and type head;
declaration ordinal 1; the local binding; and deterministic debug text. Raw
`SurfaceAst`, `SurfaceNodeId`, node kinds, tokens, and source text remain
inside the existing private
`mizar-test::runner::type_elaboration::source_statement` leaf. Source and
snapshot hashes are selector fingerprints, not separately typed checker
fields. Task 269C may copy only the complete byte-exact `debug_text()` string
as one opaque syntax-free authentication fingerprint. Its embedded source/
snapshot hashes and type ranges remain selector evidence: the checker neither
parses them into typed sites nor accepts independent fields for them. In
particular, the node numbers in the Surface table above never cross the runner
boundary, are never laundered into `TypedSiteRef` values, and are never
published as typed ownership.

The private data shape is frozen as these fields: `source_id`, `module_id`,
`source_fingerprint`, `surface_fingerprint`, `theorem_symbol`,
`theorem_definition`, `contribution`, `theorem_range`, `proof_range`,
`let_range`, `segment_range`, `name_range`, `type_range`, `type_head_range`,
`source_ordinal`, and `local`. The field names themselves provide the seven
source roles; there is no generic site id. It derives `Debug`, `Clone`,
`PartialEq`, and `Eq`; read-only crate-private getters and `debug_text()` are
the only access. No constructor is exposed outside the leaf.

The complete debug grammar is:

```text
source-proof-local-let-lower-debug-v1
module: <package>::<module>
source-fingerprint: "7860a3fe5af89063ac6a2b9a4465cac36d26f6d64e892ba6e2c89bcbaaf9763a"
surface-fingerprint: "1fc35ec18db82efc0968b2f42b08cfaae678184983210cd26f060d45354c7f68"
theorem symbol=<quoted-fqn> definition=0 contribution=0 range=19..99 proof=59..98
let range=67..80 segment=71..79 source_ordinal=1
name range=71..72 spelling="y" scope=[0] visible_after=1
type range=76..79 head=76..79 spelling="set" form=bare
```

Only `<package>`, `<module>`, and `<quoted-fqn>` are validated runtime values;
every other byte and line order is literal, including one trailing LF.

### Ownership, exclusions, and semantic deferrals

Task 269CP owns only exact extraction and provenance authentication. It
publishes no `SourceStatementHandoff`, `SourceTypeApplicationHandoff`,
`BindingEnv` mutation, `LetBinding`, source proof-local handoff, TypedAst or
ResolvedTypedAst owner, type result, assumption, fact, obligation, diagnostic,
goal, theorem status, proof, Core, CFG, or VC row. Future Task 269C must freeze
its checker let-binding ABI separately and keep the binding type site missing.
The already observed absence of source-type admission for
`BindingKind::LetBinding` does not block that binding-only transaction, but a
later typed-source owner must be selected and frozen as a separate
prerequisite; neither Task 269CP nor Task 269C may absorb it.

The selector rejects every byte change; absent/duplicate/reordered/recovered
nodes; wrong root, range, child, token, shell, symbol, definition,
contribution, module, namespace, origin, scope, ordinal, or local field;
multiple or implicit variables; multiple typed segments; attributes;
`such that`; trailing `by`; nested proof shapes; later use of `y`; and
`given`, `consider`, `take`, `set`, `reconsider`, `deffunc`, or `defpred`
substitution. Task-269A/B private sources and the mixed active gap fixture are
explicit no-match families.

The Chapter-15 universal encoding, type guard, well-formedness discharge,
goal/thesis transformation, universal closure, shadow behavior beyond this
single definition site, later-use resolution, capture, typing, proof
acceptance, and all semantic effects remain deferred. No rule is inferred
from current implementation behavior.

### Tests, impact, audit, and exit

Implementation is limited to the existing runner production leaf, both
existing test-only facade hops, and the existing proof-local runner test file:
`crates/mizar-test/src/runner/type_elaboration/source_statement.rs`,
`crates/mizar-test/src/runner/type_elaboration.rs`,
`crates/mizar-test/src/runner.rs`, and
`crates/mizar-test/src/runner/tests/type_elaboration/source_proof_local_declaration.rs`.
The first facade hop only re-exports the new crate-private test seam and the
second only imports it into the already included runner test module; neither
is a production dispatch or public API change.
Exactly four new runner tests
cover the full exact output/debug oracle, parser/resolver/local/all-node
mutation matrix, near-miss and B3N/B3M1/mixed-family isolation, and zero
checker/active semantic effect. Checker tests remain `482`; runner tests are
projected `536 -> 540`. Runner production paths remain 37; line count and
raw/normalized test-list plus production-content hashes are remeasured.

No `.miz`, sidecar, expectation, trace row/status/backlink, metadata, Cargo,
public diagnostic, corpus case, requirement, pass/fail, active-stage count,
type-coverage count, or CLI output may change. The coverage audit records the
explicit `269CP -> 269C` follow-up but grants no executable coverage; Chapters
15/16 remain partial and the existing trace hash remains unchanged.

Exit requires synchronized EN/JA records, repeated specification review with
**NO FINDINGS**, docs-only verification and commit, fresh parser/resolver/
count/hash preflight, the exact runner-private implementation, independent
test/implementation/source-doc reviews with **NO FINDINGS**, all nine final
hard gates PASS at uncapped 90/100 or better, task-only staging/commit, clean
post-commit inventory, unchanged protected stash, and automatic Task-269C
selection limited to the binding-only contract above.

### Implementation closure

The runner-private producer now realizes exactly this prerequisite and no
later semantic owner. In addition to every frozen node/range/child/recovery
row, selection authenticates the absent expression root, token side table
`0..23`, exact reserve/theorem shell fields, exact
`parser-signature-v1` theorem payload, definition/contribution provenance,
and absent visible module `y`. The output retains only the frozen syntax-free
identities, ranges, ordinal, local row, fingerprints, and debug grammar.

Four adjacent tests cover exact success; every node, side-table, shell,
resolver, output, and local guard; exact rejection precedence; near misses
and family/fixture isolation; and zero checker/active semantic effect.
Test-sufficiency and implementation re-reviews report **NO FINDINGS**. No
checker ABI, source type, Typed/Resolved owner, binding transaction, goal,
fact, proof, acceptance, discharge, or downstream IR is activated.

## Checker Task 269C Frozen Binding-Only Proof-`let` Transaction

### Selection, authority, and classification

Fresh inventory after Task-269CP implementation commit
`4431211d64e0030180852a5d8055edc202a629ba` selects only Task 269C. Chapter 4
Sections 4.1, 4.2, and 4.6 require a proof-local `let` to introduce one fresh
free-variable binding in the enclosing proof scope. Chapter 15 Sections
15.2.1 and 15.10 require proof-block locality and prohibit same-scope
duplicates. Chapter 16 Sections 16.3.3, 16.4.1, and 16.4.2 establish the
proof-block owner and local visibility. These authorities justify only the
binding row and scope; they do not authorize this task to construct the type
guard, change `thesis`, discharge an obligation, or accept the proof.

Task 269CP supplies the exact runner-private source/Surface/resolver/local
projection. The existing reserve bridge can independently prepare the exact
module-level base `BindingEnv`, and the checker already exposes
`BindingDraft::from_local_term`, `BindingKind::LetBinding`,
`BindingTypeSite::Missing`, lexical lookup, and one-shot Typed/final ownership
patterns. The missing binding transaction is bounded `source_drift`; focused
fail-closed coverage is a bounded `test_gap`. Missing `LetBinding` source-type
admission and resolver-wide use/capture payload remain separate
`source_drift`; merging either into 269C would be a `boundary_violation`.
The worktree is clean and the protected stash is unchanged. `origin/main`
divergence is a report-only `repo_metadata_conflict` that does not obscure the
exact commit target.

### Exact syntax-free checker ABI

The existing checker module `source_proof_local_declaration` adds the following
public sibling contract, not an extension of the named-witness transaction.
Private fields have no unchecked constructor or mutable accessor.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalLetBindingHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub lower_fingerprint: String,
    pub theorem_symbol: SymbolId,
    pub theorem_definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub theorem_range: SourceRange,
    pub proof_range: SourceRange,
    pub let_range: SourceRange,
    pub segment_range: SourceRange,
    pub name_range: SourceRange,
    pub source_ordinal: usize,
    pub local: LocalTermBinding,
    pub recovery: SourceProofLocalLetBindingRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceProofLocalLetBindingId(usize);

impl SourceProofLocalLetBindingId {
    pub const fn new(index: usize) -> Self;
    pub const fn index(self) -> usize;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceProofLocalLetBindingRecovery {
    Normal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalLetBinding {
    binding: BindingId,
    binding_context: BindingContextId,
    source_ordinal: usize,
    visible_after_ordinal: usize,
    recovery: SourceProofLocalLetBindingRecovery,
}

impl SourceProofLocalLetBinding {
    pub const fn binding(&self) -> BindingId;
    pub const fn binding_context(&self) -> BindingContextId;
    pub const fn source_ordinal(&self) -> usize;
    pub const fn visible_after_ordinal(&self) -> usize;
    pub const fn recovery(&self) -> SourceProofLocalLetBindingRecovery;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalLetBindingTable {
    rows: Vec<SourceProofLocalLetBinding>,
}

impl SourceProofLocalLetBindingTable {
    pub fn get(
        &self,
        id: SourceProofLocalLetBindingId,
    ) -> Option<&SourceProofLocalLetBinding>;
    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (SourceProofLocalLetBindingId, &SourceProofLocalLetBinding),
    >;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalLetBindingHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    lower_fingerprint: String,
    theorem_symbol: SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    let_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
    base_binding_env: BindingEnv,
    base_binding_fingerprint: String,
    binding_env: BindingEnv,
    final_binding_fingerprint: String,
    bindings: SourceProofLocalLetBindingTable,
}

impl SourceProofLocalLetBindingHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn lower_fingerprint(&self) -> &str;
    pub const fn theorem_symbol(&self) -> &SymbolId;
    pub const fn theorem_definition(&self) -> DefinitionId;
    pub const fn contribution(&self) -> SourceContributionId;
    pub const fn theorem_range(&self) -> SourceRange;
    pub const fn proof_range(&self) -> SourceRange;
    pub const fn let_range(&self) -> SourceRange;
    pub const fn segment_range(&self) -> SourceRange;
    pub const fn name_range(&self) -> SourceRange;
    pub const fn base_binding_env(&self) -> &BindingEnv;
    pub fn base_binding_fingerprint(&self) -> &str;
    pub const fn binding_env(&self) -> &BindingEnv;
    pub fn final_binding_fingerprint(&self) -> &str;
    pub const fn bindings(&self) -> &SourceProofLocalLetBindingTable;
    pub fn debug_text(&self) -> String;

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
    ) -> Result<(), SourceProofLocalLetBindingError>;

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalLetBindingError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SourceProofLocalLetBindingProducer;

impl SourceProofLocalLetBindingProducer {
    pub fn build(
        input: SourceProofLocalLetBindingHandoffInput,
        base_binding_env: &BindingEnv,
    ) -> Result<
        SourceProofLocalLetBindingHandoff,
        SourceProofLocalLetBindingError,
    >;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalLetBindingError {
    InvalidTransaction,
    DependencyMismatch,
    InvalidBaseBindingEnvironment,
    InvalidAggregate,
    InvalidDeclaration {
        binding: SourceProofLocalLetBindingId,
    },
    InvalidBindingEnvironment,
    InvalidInstallation,
}
```

`SourceId`, `SourceRange`, `ModuleId`, `SymbolId`, `DefinitionId`,
`SourceContributionId`, `LocalTermBinding`, `BindingId`, `BindingContextId`,
and `BindingEnv` are their existing owner-module types; the sibling defines no
aliases. No raw `SurfaceAst`, syntax node, declaration shell, `SymbolEnv`,
source text, type-expression row, formula, goal, fact, proof, or obligation
crosses this ABI. The checker input deliberately has no independent source or
Surface fingerprint, type range, or type-head field. Its sole lower token is
the complete byte-exact `source-proof-local-let-lower-debug-v1` string frozen
by Task 269CP. That opaque string embeds source/Surface SHA-256
`7860a3fe5af89063ac6a2b9a4465cac36d26f6d64e892ba6e2c89bcbaaf9763a` /
`1fc35ec18db82efc0968b2f42b08cfaae678184983210cd26f060d45354c7f68`
and the `76..79` type evidence, but none becomes separately admitted checker
ownership.

The exact fail-closed provenance is theorem range `19..99`, proof range
`59..98`, `let` range `67..80`, segment range `71..79`, name range `71..72`,
source ordinal 1, theorem definition/contribution indices `0/0`, and local
`y`, scope `[0]`, declaration `71..72`, visible-after ordinal 1. The theorem
symbol must belong to the supplied module and retain the Task-269CP identity.
No sibling theorem, second declaration/segment/name, implicit declaration,
`such that`, trailing `by`, nested proof, recovered node, or adjacent 269A/B
profile is accepted.

### Base, transition, lookup, and output

The runner obtains the base through existing
`extract_builtin_source_reserve_declarations_after_node_guard` and
`SourceReserveDeclarationBridge::prepare_binding_env`; it does not fabricate
or rescan a checker binding from source text. The checker authenticates exact
normal base profile `1/1/0`: context 0 is the module context, has no parent or
scope, and owns/exposes reserved binding 0. Binding 0 is `x`,
`ReservedVariable`, module-owned, declaration/identity range `8..9`,
visible-after 0, source type site `14..17`, reserved, uncaptured,
diagnostic-free, and normal. Source/module identities match and diagnostics
are empty.

The atomic transition is exactly `1/1/0 -> 2/2/0`. It appends proof context 1
with owner `SourceStatement(59..98)`, parent 0, proof layer, scope `[0]`, owned
bindings `[1]`, visible bindings `[0,1]`, and normal recovery. It appends
binding 1 with spelling `y`, kind `LetBinding`,
`ResolverLocal([0], ordinal=1, range=71..72)`, owner context 1, declaration
range `71..72`, visible-after 1, `BindingTypeSite::Missing`, active status,
empty captures/diagnostics, and normal recovery. The single handoff row is row
0 -> binding 1/context 1/source ordinal 1/visible-after 1/normal. Context 0 and
binding 0 remain byte-identical.

A synthetic same-scope lookup at ordinal 1 returns the existing forward-
reference result for binding 1; ordinal 2 returns local binding 1. This
validates table visibility only: Task 269C claims no source use at ordinal 2
and creates no use-site or capture row. The handoff retains exact base/final
binding debug fingerprints. Deterministic
`source-proof-local-let-binding-debug-v1` prints the complete frozen transaction.
Every field participates in validation before publication.

Validation is transactional and the first failing class wins in this stable
order: (1) source/module transaction identity; (2) byte-exact lower token plus
theorem symbol/module/FQN, definition/contribution `0/0`, and all five ranges;
(3) exact base `BindingEnv`; (4) one dense output row; (5) local spelling,
scope, ordinal, range, recovery, and row links; (6) reconstructed final
`BindingEnv`, both binding fingerprints, and the two lookup results; and (7)
Typed/final owner availability. The respective errors are
`InvalidTransaction`, `DependencyMismatch`, `InvalidBaseBindingEnvironment`,
`InvalidAggregate`, `InvalidDeclaration { binding:
SourceProofLocalLetBindingId(0) }`, `InvalidBindingEnvironment`, and
`InvalidInstallation`. `build` performs phases 1--6; aggregate corruption is
possible only during replay because the public input is singular.

Exact `Display` text is:

| variant | exact text |
|---|---|
| `InvalidTransaction` | `source proof-local let-binding transaction is invalid` |
| `DependencyMismatch` | `source proof-local let-binding dependency mismatch` |
| `InvalidBaseBindingEnvironment` | `source proof-local let-binding base binding environment is invalid` |
| `InvalidAggregate` | `source proof-local let-binding aggregate is invalid` |
| `InvalidDeclaration { binding }` | `source proof-local let-binding <binding.index()> is invalid` |
| `InvalidBindingEnvironment` | `source proof-local let-binding binding environment is invalid` |
| `InvalidInstallation` | `source proof-local let-binding installation is invalid` |

The error implements `std::error::Error`. Failure publishes no partial
context, binding, table row, fingerprint, Typed owner, or final owner.

The exact stable debug grammar is:

```text
source-proof-local-let-binding-debug-v1
module: <package>::<path>
lower-fingerprint: <quoted Task-269CP debug bytes>
theorem symbol=<quoted-fqn> definition=0 contribution=0 range=19..99 proof=59..98
let range=67..80 segment=71..79 name=71..72 source_ordinal=1
base-binding-fingerprint: <quoted BindingEnv debug bytes>
binding#0 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal
final-binding-fingerprint: <quoted BindingEnv debug bytes>
```

Rows are dense-ID ordered. `normal` is the only recovery spelling. The lower
and binding fingerprint values use Rust string `Debug` (`{:?}`), including
double quotes and standard escapes for LF, quotes, backslashes, and control
characters. The symbol FQN also uses `{:?}`. Lines occur exactly in the shown
order, there are no blank lines, and the block has one final LF. Existing
Task-269A/B and empty debug bytes are unchanged.

### Typed/final ownership and exclusions

`TypedAst` adds one private optional `source_proof_local_let_binding` field and
exactly these methods; `TypedAstParts` receives no replacement field:

```rust
pub const fn source_proof_local_let_binding(
    &self,
) -> Option<&SourceProofLocalLetBindingHandoff>;

pub fn with_source_proof_local_let_binding(
    self,
    handoff: SourceProofLocalLetBindingHandoff,
) -> Result<Self, TypedAstError>;
```

`TypedAstError` adds exactly the unit variant
`InvalidSourceProofLocalLetBinding`, whose exact text is
`typed AST source proof-local let-binding handoff is inconsistent`. The
admitted base is an otherwise-empty `TypedAst`: no resolved root, typed node,
or existing source handoff of any family (including Task-269A/B), and empty
context/type/fact/coercion/initial-obligation/diagnostic tables. Installation
replays phases 1--7 and publishes once without adding a node, link, source
type, fact, coercion, obligation, or diagnostic. Duplicate, orphan, stale,
cross-family, partial, or semantic-coexisting installation maps to the new
AST error and leaves the input value unchanged.

`ResolvedTypedAst` adds the same exact read-only getter signature and clone-
preserves the handoff only after replaying validation against the same empty
semantic/node profile. `ResolvedTypedAstInputs` receives no replacement path.
`ResolvedTypedAstError` adds exactly the unit variant
`InvalidSourceProofLocalLetBinding`, with exact text
`resolved typed AST source proof-local let-binding handoff is inconsistent`.
Its deterministic debug appends the new block in the proof-local handoff slot,
after the existing Task-269A/B slot; mutual exclusion makes only one nonempty.
It adds no expression metadata, candidate, overload, cluster, formula,
statement-semantic, proof, terminal-goal, initial-obligation, or diagnostic
row. Existing Task-269A/B and empty debug bytes remain unchanged.

Task 269C excludes source-type admission for `LetBinding`, bare-`set` type
checking, type guards or FOL relativization, `such that`/`by`, same-scope
duplicate diagnostics, actual later-use/capture extraction, formula/thesis or
goal transitions, facts, proof/discharge/acceptance, Core/CFG/VC/ATP, and
active corpus dispatch. A separate documentation prerequisite must freeze the
missing source-type owner before any type site changes from `Missing`.

### Implementation/test scope, measurements, audit impact, and exit

The later implementation may change exactly seven existing Rust files:

1. `crates/mizar-checker/src/source_proof_local_declaration.rs`;
2. `crates/mizar-checker/src/typed_ast.rs`;
3. `crates/mizar-checker/src/resolved_typed_ast.rs`;
4. `crates/mizar-test/src/runner/type_elaboration/source_proof_local_declaration.rs`;
5. `crates/mizar-test/src/runner/type_elaboration.rs` (test-only facade);
6. `crates/mizar-test/src/runner.rs` (test-only root facade);
7. `crates/mizar-test/src/runner/tests/type_elaboration/source_proof_local_declaration.rs`.

No new module/path, active dispatch, public runner route, checker syntax
dependency, parser/resolver source, Cargo file, fixture, sidecar, expectation,
trace metadata, or diagnostic mapping is permitted. Four checker unit tests
cover exact producer/output/debug/lookup, complete input/base/output
corruption and error precedence, Typed/final one-shot/cross-family/rollback/
replay, and missing-type/empty-semantic preservation. Four runner tests cover
the exact Task-269CP-to-checker transaction, lower/base/checker corruption,
near-miss/family/public-route isolation, and zero active/semantic effect.
Existing Task-269CP tests and every Task-269A/B byte remain unchanged.

The docs baseline is checker/runner libraries `482/540`. Raw/normalized
test-list SHA-256 values are checker
`c89028b747ba4a551d74a2f6cc9c79e3520cc79ad0f019e18a2a4c123d52288c` /
`da1022d491be404da68e41c77b800f7d0ca65765e397d28489e40d961ab453a2`
and runner
`8b9a2b9ea4aad3c6ed0b6eae32a0285d6a9fe1b5389dcc31ebc7adb872317522` /
`a8955748da86930f3e2165637e170d68c77756cbc03f3ff38b3f8de0d21cbc50`.
Implementation projects `486/544`. Checker production remains 30 paths /
165,219 lines with path/content hashes
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`1fb5ea739c810ff66ed551b359ffa7cbb26265c0057fa18f5128ee5966bad958`;
runner remains 37 paths / 71,194 lines with
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d` /
`4dcfc69a867dea5c12457d94825493a8a48e4fd5ac7b91d86412371ac25f6b03`.
Lines and content hashes are remeasured after implementation; path hashes must
remain unchanged.

The broad `.miz`, expectation, and covered diagnostic rows stay unchanged
because this private dormant slice neither executes nor accepts that broad
case. `spec_coverage_audit.md` records Task 269C as a zero-credit binding owner
and names the separate source-type prerequisite; the trace manifest remains
byte-identical. Exit requires synchronized EN/JA, repeated specification
review with **NO FINDINGS**, all nine docs-only hard gates and uncapped quality
`>=90/100`, exact docs-only staging/commit, fresh preflight, the exact seven-
file implementation, separate no-findings test/implementation/source-doc
reviews, full verification/count/hash reproduction, all nine final gates and
uncapped `>=90/100`, task-only commit, clean post-commit inventory, and
unchanged protected stash before selecting the next dependency-ready task.

### Documentation-prerequisite review and verification

The first read-only specification review found two high `design_drift`
issues: the claimed exact checker API was underspecified, and the Task-269CP
selector-fingerprint boundary contradicted the proposed checker fields. The
contract now freezes every Rust field/signature, validation/error/debug byte,
and Typed/final owner API, while permitting only one opaque complete lower
debug fingerprint across the runner boundary. Re-review reports **NO
FINDINGS**, no blocking `spec_gap`, and no remaining `boundary_violation`.

The diff remains 38 `doc/design` files only. Checker/runner lint policies pass
`15/15` and `14/14`, metadata passes `137/137`, and `cargo fmt --all --check`,
warnings-denied workspace Clippy, full `cargo test`, Cargo metadata, and
`git diff --check` pass. Libraries remain checker/runner `482/540`; their
raw/normalized test-list hashes remain
`c89028b747ba4a551d74a2f6cc9c79e3520cc79ad0f019e18a2a4c123d52288c` /
`da1022d491be404da68e41c77b800f7d0ca65765e397d28489e40d961ab453a2`
and
`8b9a2b9ea4aad3c6ed0b6eae32a0285d6a9fe1b5389dcc31ebc7adb872317522` /
`a8955748da86930f3e2165637e170d68c77756cbc03f3ff38b3f8de0d21cbc50`.
Production remains checker `30/165219` and runner `37/71194` with the frozen
path/content hashes above.

All five CLI commands reproduce unchanged plan/parse/declaration/type/proof
stdout hashes
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718` /
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56` /
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74` /
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f` /
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.
Corpus/requirements stay `428/395`, pass/fail `235/193`, active stages
`101/7/205/1`, type coverage `259=247+12`, warnings/errors `23/0`, and trace
SHA-256 remains
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.
Independent final quality review reports **NO FINDINGS**: all nine hard gates
PASS, no score cap applies, and the valid score is `100/100`
(`20/20/15/15/10/10/5/5`). Exact staging and the docs-only commit remain
parent-owned next steps.

### Task-269C implementation result

The committed documentation prerequisite is implemented without widening the
frozen seven-source-file boundary. The checker producer validates the opaque
Task-269CP lower fingerprint, exact theorem/range/resolver provenance, the
reserve-only `1/1/0` base environment, the one-row declaration table, and the
exact `2/2/0` final environment in the frozen seven-phase order. It publishes
one missing-type, active, uncaptured `LetBinding`; definition-site lookup at
ordinal 1 remains forward and synthetic ordinal 2 resolves binding 1. Typed
and final installation are one-shot, mutually exclusive with every existing
sibling family, replay the complete transaction, and add no node or semantic
payload.

The private dormant runner consumes the unchanged Task-269CP projection and
existing reserve bridge only. Four checker and four runner tests close exact
output, corruption/precedence, cross-family/rollback/replay, near-miss, and
semantic-emptiness gaps. Libraries are now checker/runner `486/544`. Raw and
normalized test-list SHA-256 values are checker
`0a4d39c5cad8ee81ee1a9b52fa437a6203202cc783100c275adb1a717fb749f7` /
`2bece131be70bdfd0a3128faa1b83852b774692353c4926f069bafa61d2d7e28`
and runner
`fa69bfaa53fb75a2a6ec62b1ac7faf8fc5e5a12693a3840e0e31439eafa156db` /
`717a16f30326b9878949c7158be81eff5f7769c32ceeb19e23de0e569eb7ab4c`.
Production is checker `30/167058` and runner `37/71412`; path hashes remain
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`,
and content hashes are
`d5d6c3bf41176422ffe78b9c612db02ef8eb8550ea080d0c11e90c16d320cb49` /
`bf8c5a242bdc3e8a6809583ef1813138afbb246e41612413d7a7783631bc3cd6`.

No parser/resolver, source-type, active dispatch, public runner, fixture,
sidecar, expectation, trace, metadata, Cargo, diagnostic, goal, guard, proof,
discharge, acceptance, fact, Core, CFG, or VC owner changed. Corpus/count/CLI
and trace values remain the frozen Task-264 baseline. The coverage audit
therefore closes only the zero-credit binding transport and keeps the separate
source-type prerequisite for fresh dependency selection.

Typed and final owners use private boxed storage for this comparatively large
handoff while preserving the frozen by-value installer and `Option<&Handoff>`
getter signatures. This representation-only choice prevents legacy
cross-family tests from exhausting the default Rust test-thread stack and
does not alter ownership, validation, debug bytes, or public semantics.

### Task-269C implementation review and verification

After a legacy cross-family crate test exposed default-thread stack exhaustion,
the bounded private boxing correction above restored that test without changing
the public contract. Repeated test-sufficiency, implementation, and source/
documentation reviews all end **NO FINDINGS**. Focused checker Task-269C tests
pass `4/4`; checker and runner libraries pass `486/486` and `544/544`; lint
policies pass `15/15` and `14/14`; metadata passes `137/137`. Cargo metadata,
`cargo fmt --all --check`, warnings-denied all-target/all-feature workspace
Clippy, full `cargo test --no-fail-fast`, and `git diff --check` pass.

All five metadata CLIs exit zero and reproduce the frozen plan/parse/
declaration/type/proof stdout hashes. They report cases/requirements `428/395`,
pass/fail `235/193`, active stages `101/7/205/1`, type coverage
`259=247+12`, and warnings/errors `23/0`. Final production, test-list, and trace
hashes reproduce the values recorded above. Independent final quality reports
**NO FINDINGS**: all nine hard gates PASS, no score cap applies, and the valid
score is `100/100` (`20/20/15/15/10/10/5/5`). Only task-only staging/commit
and clean post-commit fresh inventory remain parent-owned gates.

## Task 269CT Immutable Dependency Boundary

Task 269CT consumes `SourceProofLocalLetBindingHandoff` by value and preserves
its missing-type `2/2/0` snapshot and all dependency fingerprints unchanged.
The separate source-type composite owns the typed overlay and type handoff;
this module gains no API or source change. Syntax rescanning, later-use/
capture, assumptions, goals, facts, proof behavior, and active routing remain
outside the boundary.

## Task 269CT Implemented Consumer Boundary

`source_type.rs` now consumes the Task-269C handoff by value and stores an
unchanged dependency plus its byte-exact fingerprint inside the composite.
Task-269C's direct owner and missing type site remain unchanged and empty in
the composite Typed/final profiles. This module has no source/API/test change;
later use/capture and all proof semantics remain deferred.

## Checker Task 269GP Frozen Isolated Proof-`given` Syntax-Lower Prerequisite

### Selection, authority, and disagreement classification

Fresh inventory at `c60361977f6c4d832cf4217b85bd9b458c902848`
selects only Task 269GP. Task 269 is still open: Tasks 269A/B implement named
`take`, and Tasks 269CP/C/CT implement only one isolated `let` definition-site
and its written type. Task 270 depends on Task 269 and is therefore not yet
ready. A `set` slice would require the RHS to be authenticated as a resolved
local-binding capture, while `consider` adds a justification subtree; the
source-order-minimal independently implementable form is a syntax-only
`given` definition-site projection.

Canonical authority is Chapter 4 Sections 4.2, 4.4, and 4.6; Chapter 15
Sections 15.3.3, 15.10, and 15.11.4; and Chapter 16 Sections 16.3.3, 16.4, and
16.5. The parser simple-statement fixture and unchanged broad
`fail_type_elaboration_proof_local_declaration_gap_001` fixture establish the
surface family. That broad fixture, sidecar, expectation, and trace backlinks
remain read-only and cannot credit this isolated positive slice.

Specification review found a blocking `spec_gap` for any binding consumer:
Chapter 4 Section 4.6.1 limits `given` binders to the introducing statement or
formula, while Chapter 16 Sections 16.3.3 and 16.4.2 make the witness available
in the local subproof/enclosing block. Chapter 15 Section 15.10 specifies only
`let` variable scope and does not resolve the conflict. Therefore 269GP is
strictly narrowed to syntax/range/provenance transport. It publishes no
`LocalTermBinding`, scope path, visible-after ordinal, condition availability,
or later-use promise. Task 269G and 269GT are human-blocked until canonical
scope intent is reconciled; this task does not choose either interpretation.

Within the narrowed task, the missing exact lower contract is `design_drift`,
its absent private source/Surface/resolver projection is bounded
`source_drift`, and the missing four-test guard matrix is a canonical-derived
`test_gap`. Synthesizing binding visibility, an existential fact, Skolem
result, label identity, goal change, or local-use/capture row would be a
`boundary_violation`. The local
`origin/main...HEAD = 0/8` is a report-only `repo_metadata_conflict`; protected
stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4` is outside the task.

### Exact source and complete Surface identity

The sole admitted source is this private final-LF text:

```mizar
reserve x for set;
theorem FormulaStatementGivenSmoke: thesis proof
  given y being set such that G: thesis;
  thus thesis;
end;
```

It is exactly 129 bytes with SHA-256
`04e54b8ada9af54fde9f937e1bb0f96bd8cf85002b2b57f4d348b11c8eb72a2f`.
The normal Surface AST has 48 nodes, root 47, root range `0..128`, no
expression root, recovery, or frontend diagnostic, token rows `0..24`, and
snapshot SHA-256
`58ac16a3c75860180a8bec5dc8e87ec8b269fe75715a6d8363f7ef064e3deea8`.
The selector authenticates every node kind, source id/range, ordered child
list, recovery state, and token text/kind. Role-defining rows are:

| node | kind | range | children |
| ---: | --- | --- | --- |
| 28 | `ReserveItem` | `0..18` | `[0,27,4]` |
| 31/32 | `TypeHead` / `TypeExpression` | `84..87` | `[13]` / `[31]` |
| 33 | `QualifiedVariableSegment` | `76..87` | `[11,12,32]` |
| 34/35 | `FormulaConstant(Thesis)` / `FormulaExpression` | `101..107` | `[18]` / `[34]` |
| 36 | `Proposition` | `98..107` | `[16,17,35]` |
| 37 | `ConditionList` | `93..107` | `[15,36]` |
| 38 | `GivenStatement` | `70..108` | `[10,33,14,37,19]` |
| 42 | `ConclusionStatement` | `111..123` | `[20,41,22]` |
| 43 | `ProofBlock` | `62..127` | `[9,38,42,23]` |
| 44 | `TheoremItem` | `19..128` | `[5,6,7,30,43,24]` |
| 47 | `Root` | `0..128` | tokens `0..24`, then `[46]` |

Condition subtree `34..37`, label tokens `16/17`, and conclusion subtree
`39..42` are selector-only exclusions and do not cross the lower handoff.

### Resolver provenance and private lower output

The declaration-shell profile is exactly two normal root shells: reserve
ordinal/node/range `0/28/0..18` and theorem `1/44/19..128`. The symbol
environment contains exactly one public/exported theorem symbol, definition,
and local-source contribution. The theorem origin is range `19..128`, path
`[2,1]`, and no import. Its opaque parser signature is exactly:

```text
node=TheoremItem;symbol=theorem;definition=theorem;primary_tokens=theorem FormulaStatementGivenSmoke : thesis proof given y being set such that G : thesis ; thus thesis ; end ;;notation=_;arity=_;roles=FormulaExpression,ProofBlock
```

The definition has no parameters, binders, notation, document, conflict, or
dependencies. Import/export/label/overload/registration/lexical-summary/
namespace/declaration-dependency/module-summary indexes are empty. Resolver
publishes neither `y` nor `G` as a module symbol or label; 269GP must not invent
either identity.

The theorem symbol's primary spelling is `FormulaStatementGivenSmoke`,
namespace/module are the requested module, visibility/export status are
`Public`/`Exported`, contribution is 0, notation is absent, relations are
empty, and origin is the exact source/range/path/no-import row above. Definition
0 references that symbol and origin, has kind `Theorem`, visibility `Public`,
contribution 0, no arity or signature deviation, and all listed optional/list
fields empty. Contribution 0 has the requested module, kind
`LocalSource { source_id }`, anchor `0..18`, symbol effect `[theorem]`,
definition effect `[0]`, and empty label/overload/registration/lexical/
namespace/declaration-dependency/import/export/diagnostic effects.

Runner-private `SourceProofLocalGivenLowerOutput` retains only source/module
identity and fingerprints, theorem resolver identity, theorem/proof/
`GivenStatement`/segment/name/type ranges, exact token spellings `y` and `set`,
and source statement ordinal 1. It deliberately has no binding-shaped field.
Complete source SHA-256, Surface snapshot SHA-256, shell profile, resolver
theorem profile, output row, and exact debug bytes are independent fail-closed
fingerprints.

### Complete runner-private Rust contract

The production leaf adds only this crate-private family; field visibility is
private and all accessors are `pub(in crate::runner)`:

```rust
pub(in crate::runner) const SOURCE_PROOF_LOCAL_GIVEN_TEXT: &str;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::runner) struct SourceProofLocalGivenLowerOutput {
    source_id: SourceId,
    module_id: ModuleId,
    source_fingerprint: String,
    surface_fingerprint: String,
    theorem_symbol: SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    given_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
    name_spelling: String,
    type_range: SourceRange,
    type_head_range: SourceRange,
    type_spelling: String,
    source_ordinal: usize,
}

pub(in crate::runner) fn source_proof_local_given_lower_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalGivenLowerOutput, String>>;
```

The read-only impl exposes exactly `source_id() -> SourceId`,
`module_id() -> &ModuleId`, `source_fingerprint() -> &str`,
`surface_fingerprint() -> &str`, `theorem_symbol() -> &SymbolId`,
`theorem_definition() -> DefinitionId`,
`contribution() -> SourceContributionId`, `theorem_range() -> SourceRange`,
`proof_range() -> SourceRange`, `given_range() -> SourceRange`,
`segment_range() -> SourceRange`, `name_range() -> SourceRange`,
`name_spelling() -> &str`, `type_range() -> SourceRange`,
`type_head_range() -> SourceRange`, `type_spelling() -> &str`,
`source_ordinal() -> usize`, and `debug_text() -> String`, all
`pub(in crate::runner)`. Exact values are definition/contribution/ordinal
`0/0/1`, theorem `19..128`, proof `62..127`, given `70..108`, segment
`76..87`, name `76..77`/`"y"`, and type/head `84..87`/`"set"`.
`debug_text()` is byte-exact in this field order and ends with exactly one LF:

```text
source-proof-local-given-lower-debug-v1
module: {package}::{module}
source-fingerprint: "04e54b8ada9af54fde9f937e1bb0f96bd8cf85002b2b57f4d348b11c8eb72a2f"
surface-fingerprint: "58ac16a3c75860180a8bec5dc8e87ec8b269fe75715a6d8363f7ef064e3deea8"
theorem symbol="{fqn}" definition=0 contribution=0 range=19..128 proof=62..127
given range=70..108 segment=76..87 source_ordinal=1
name range=76..77 spelling="y"
type range=84..87 head=84..87 spelling="set" form=bare
```

Four test-only enums are `pub(in crate::runner)`, derive
`Debug, Clone, Copy, PartialEq, Eq`, and have exactly these variants:

```rust
enum SourceProofLocalGivenSurfaceMutation {
    None, ExpressionRoot, TokenNode(usize), TokenNodeCount,
    NodeKind(usize), NodeSourceId(usize), NodeRange(usize),
    NodeRecovery(usize), NodeChildren(usize),
    MissingRootIdentity, WrongRootIdentity,
}
enum SourceProofLocalGivenLowerMutation {
    None, SourceId, Module, SourceFingerprint, SurfaceFingerprint,
    TheoremSymbol, TheoremDefinition, Contribution, TheoremRange, ProofRange,
    GivenRange, SegmentRange, NameRange, NameSpelling, TypeRange,
    TypeHeadRange, TypeSpelling, SourceOrdinal,
}
enum SourceProofLocalGivenShellMutation {
    None, Id(usize), Ordinal(usize), Kind(usize), Module(usize), Node(usize),
    Syntax(usize), Range(usize), Parent(usize), VisibilityState(usize),
    VisibilityMarker(usize), VisibilitySpelling(usize), Recovery(usize),
}
enum SourceProofLocalGivenResolverProfileMutation {
    None, ResolverModule, ImportIndex, ExportIndex, LabelIndex, OverloadIndex,
    RegistrationIndex, LexicalSummaryIndex, NamespaceGraph,
    DeclarationDependencyIndex, ModuleSummaryIndex, SymbolModule,
    SymbolNotation, SymbolContribution, SymbolRelations, SymbolOriginSource,
    SymbolOriginImport, DefinitionId, DefinitionParameters,
    DefinitionBinders, DefinitionNotation, DefinitionDoc,
    DefinitionContribution, DefinitionConflict, DefinitionDependencies,
    ContributionLabelEffect, ContributionOverloadEffect,
    ContributionRegistrationEffect, ContributionLexicalEffect,
    ContributionNamespaceEffect, ContributionDeclarationDependencyEffect,
    ContributionImportEffect, ContributionExportEffect,
    ContributionDiagnosticEffect,
}
```

The five `#[cfg(test)] pub(in crate::runner)` seams are the production
signature plus respectively a final `SourceProofLocalGivenSurfaceMutation`,
`SourceProofLocalGivenLowerMutation`, `SourceProofLocalGivenShellMutation`, or
`SourceProofLocalGivenResolverProfileMutation` argument, named:

```text
source_proof_local_given_lower_output_with_surface_mutation
source_proof_local_given_lower_output_with_mutation
source_proof_local_given_lower_output_with_shell_mutation
source_proof_local_given_lower_output_with_resolver_profile_mutation
source_proof_local_given_lower_output_with_resolver_mutation
```

The last seam instead takes final
`mutate: impl FnOnce(SymbolEnv) -> SymbolEnv`. Selector/source mismatches return
`None`; once selected, all failures return `Some(Err(String))`. Validation
precedence is exact Surface identity after selection; shell count/export/
profile in ordinal order; resolver module and empty indexes; theorem symbol,
definition, and contribution; lower row; then debug bytes. Error strings are:

```text
Task269GP exact Surface identity changed after selection
Task269GP requires exactly two declaration shells
Task269GP resolver shells unexpectedly export a path
Task269GP declaration shell {ordinal} mismatch
Task269GP raw resolver module mismatch
Task269GP local y already resolves as a module symbol
Task269GP raw resolver inventory mismatch
Task269GP requires one exact theorem owner
Task269GP exact theorem owner provenance mismatch
Task269GP requires one exact theorem definition
Task269GP theorem contribution is missing
Task269GP theorem symbol provenance mismatch
Task269GP theorem definition provenance mismatch
Task269GP theorem contribution provenance mismatch
Task269GP private lower output mismatch
Task269GP private lower debug grammar mismatch
```

The whole-environment seam preserves neutral reconstruction and rejects
missing, duplicated, wrong-module, and cross-profile environments. No error
string may expose parser/debug internals beyond this list.

### Ownership, exclusions, tests, impact, and exit

Task 269GP is runner-private. It adds no checker/public API,
`LocalTermBinding`, `BindingEnv`,
source-type table, typed arena, `TypedAst`/`ResolvedTypedAst` owner, statement
or formula row, fact, diagnostic, active dispatch, fixture, sidecar,
expectation, trace row/status/backlink, metadata case, Cargo edge, or coverage
credit. The canonical `given` scope contradiction blocks only the direct
binding/type consumers 269G/269GT. Given-condition availability,
Skolem/existential meaning, label or
candidate-fact publication, escape checking, goal/thesis composition, proof/
discharge/acceptance, and Core/CFG/VC remain Task 258/272 or later semantic
work. `set` capture, `consider`, other local forms, real later-use replay, and
Task 270 remain separately deferred; 269GP does not assign them a new blocker.

Implementation is restricted to the existing runner source-statement leaf,
two existing test-only facade hops, and existing proof-local test file. The
four exact test functions are:

```text
source_proof_local_given_lower_projection_is_exact_and_private
source_proof_local_given_lower_rejects_every_corruption_with_frozen_precedence
source_proof_local_given_lower_excludes_near_misses_and_adjacent_families
source_proof_local_given_lower_has_zero_checker_or_semantic_effect
```

They cover exact source/Surface/resolver/output/debug; every enum variant,
every token/node/shell ordinal, whole-environment corruption and precedence;
source/header/body/trailing-LF near misses, broad/parser fixtures, adjacent
`let`/`take`/`set`/`consider`/`reconsider`/inline families; and zero checker or
semantic effect plus unchanged Tasks 269CP/C/CT.
Runner library projects `548 -> 552`; checker stays `490`; production paths
remain checker/runner `30/37`. Baselines remain corpus/requirements `428/395`,
pass/fail `235/193`, active stages `101/7/205/1`, type coverage `259=247+12`,
warnings/errors `23/0`, and trace SHA-256
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.

The prerequisite exits only after synchronized EN/JA records, repeated
specification review with **NO FINDINGS** for the narrowed contract, all nine
hard gates uncapped at
90/100 or better, full verification/count/hash reproduction, exact docs-only
staging, its dedicated commit, and fresh preflight. The four-file
implementation then requires separate test-sufficiency, implementation,
source/docs, and final-quality reviews with **NO FINDINGS**, the same hard and
verification gates and one exact commit. Fresh inventory after that commit
must report the human-owned canonical scope contradiction as the blocker
rather than automatically starting 269G.

Completion evidence: [central Task-269GP historical contract](../../task_contracts/en/269GP.md#completion-evidence).

## Checker Task 269GS Canonical Proof-`given` Scope Reconciliation

Explicit human authority resolves the previous contradiction. A variable
introduced by `given` binds its occurrences in that statement's `such that`
conditions and remains visible to subsequent statements through the end of the
innermost enclosing proof or reasoning block. Nested child blocks inherit the
binding unless they shadow it; the variable is not visible after the block or
in a sibling block. This rule covers the witness variable only. Existing
reasoning-block label rules continue to govern `such that` labels, and no
condition/fact, existential/Skolem, goal, proof, discharge, acceptance, IR, or
VC behavior is inferred.

Task 269GS is a documentation-only prerequisite over the paired Chapter 4/15/16
specifications and synchronized derived records. It changes no production,
fixture, sidecar, expectation, trace, count/status, metadata, Cargo, or public
API artifact. The existing 269GP lower row stays syntax/range/provenance-only.
The resolved rule makes binding-only Task 269G dependency-ready, with exact
scope IDs, visibility ordinals, nested inheritance/shadowing, block restoration,
and spec-derived tests still to freeze. Task 269GT remains separately ordered
after 269G, and all semantic exclusions above remain in force.

## Checker Task 269G Frozen Binding-Only Proof-`given` Transaction

### Selection, authority, and classified boundary

Fresh inventory after Task-269GS commit
`10bdd041517eb0334df982484b540e2799b106ca` selects only Task 269G. Canonical
Chapters 4, 15, and 16 require a `given` witness to bind its occurrences in
the declaration's `such that` conditions and to remain visible to later
statements through the innermost enclosing proof or reasoning block. A nested
child inherits the binding unless it shadows it; parent and sibling blocks do
not see it, and leaving a child restores the outer binding. This authority is
sufficient for lexical binding and lookup only.

Task 269GP supplies the immutable exact source/Surface/resolver/lower
projection, and the reserve bridge supplies the module base `BindingEnv`. The
absent transaction is `source_drift`; focused binding/lookup coverage is the
Task-269GS `test_gap`. Task 269G closes both only at the private binding
boundary. It does not interpret or publish a condition, label fact,
existential/Skolem fact, goal, proof step, discharge, acceptance, IR, or VC.
Source-type admission is separate Task 269GT; merging it here is a
`boundary_violation`.

No lower-stage change is required. Task-269GP's 129-byte source, 48-node
Surface arena, resolver profile, lower fields, source SHA-256
`04e54b8ada9af54fde9f937e1bb0f96bd8cf85002b2b57f4d348b11c8eb72a2f`,
Surface SHA-256
`58ac16a3c75860180a8bec5dc8e87ec8b269fe75715a6d8363f7ef064e3deea8`,
and debug grammar remain byte-identical. Existing `.miz`, sidecars,
expectations, and trace metadata are read-only. Spec-derived Rust scope tests
are sufficient for this dormant binding slice; active corpus/type execution
remains deferred.

### Exact checker ABI and public surface

`binding_env::BindingKind` adds exactly one forward-compatible public variant,
`GivenWitness`, immediately after `LetBinding` and before `Generated` in the
existing `#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]`
and `#[non_exhaustive]` enum. Its exhaustive internal `binding_kind_name`
mapping is exactly `GivenWitness => "given_witness"`; this stable key enters
the final `BindingEnv::debug_text()` fingerprint and consequently the outer
handoff debug's quoted final-fingerprint field. The reserve-only base
fingerprint stays byte-identical and does not contain this key. The existing checker module
`source_proof_local_declaration` adds the following sibling family; private
fields have no unchecked public constructor or mutable accessor:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenBindingHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub lower_fingerprint: String,
    pub theorem_symbol: SymbolId,
    pub theorem_definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub theorem_range: SourceRange,
    pub proof_range: SourceRange,
    pub given_range: SourceRange,
    pub segment_range: SourceRange,
    pub name_range: SourceRange,
    pub source_ordinal: usize,
    pub local: LocalTermBinding,
    pub recovery: SourceProofLocalGivenBindingRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceProofLocalGivenBindingId(usize);

impl SourceProofLocalGivenBindingId {
    pub const fn new(index: usize) -> Self;
    pub const fn index(self) -> usize;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceProofLocalGivenBindingRecovery {
    Normal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenBinding {
    binding: BindingId,
    binding_context: BindingContextId,
    source_ordinal: usize,
    visible_after_ordinal: usize,
    recovery: SourceProofLocalGivenBindingRecovery,
}

impl SourceProofLocalGivenBinding {
    pub const fn binding(&self) -> BindingId;
    pub const fn binding_context(&self) -> BindingContextId;
    pub const fn source_ordinal(&self) -> usize;
    pub const fn visible_after_ordinal(&self) -> usize;
    pub const fn recovery(&self) -> SourceProofLocalGivenBindingRecovery;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenBindingTable {
    rows: Vec<SourceProofLocalGivenBinding>,
}

impl SourceProofLocalGivenBindingTable {
    pub fn get(
        &self,
        id: SourceProofLocalGivenBindingId,
    ) -> Option<&SourceProofLocalGivenBinding>;
    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (SourceProofLocalGivenBindingId, &SourceProofLocalGivenBinding),
    >;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenBindingHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    lower_fingerprint: String,
    theorem_symbol: SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    given_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
    base_binding_env: BindingEnv,
    base_binding_fingerprint: String,
    binding_env: BindingEnv,
    final_binding_fingerprint: String,
    bindings: SourceProofLocalGivenBindingTable,
}

impl SourceProofLocalGivenBindingHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn lower_fingerprint(&self) -> &str;
    pub const fn theorem_symbol(&self) -> &SymbolId;
    pub const fn theorem_definition(&self) -> DefinitionId;
    pub const fn contribution(&self) -> SourceContributionId;
    pub const fn theorem_range(&self) -> SourceRange;
    pub const fn proof_range(&self) -> SourceRange;
    pub const fn given_range(&self) -> SourceRange;
    pub const fn segment_range(&self) -> SourceRange;
    pub const fn name_range(&self) -> SourceRange;
    pub const fn base_binding_env(&self) -> &BindingEnv;
    pub fn base_binding_fingerprint(&self) -> &str;
    pub const fn binding_env(&self) -> &BindingEnv;
    pub fn final_binding_fingerprint(&self) -> &str;
    pub const fn bindings(&self) -> &SourceProofLocalGivenBindingTable;
    pub fn debug_text(&self) -> String;

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
    ) -> Result<(), SourceProofLocalGivenBindingError>;

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalGivenBindingError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SourceProofLocalGivenBindingProducer;

impl SourceProofLocalGivenBindingProducer {
    pub fn build(
        input: SourceProofLocalGivenBindingHandoffInput,
        base_binding_env: &BindingEnv,
    ) -> Result<
        SourceProofLocalGivenBindingHandoff,
        SourceProofLocalGivenBindingError,
    >;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalGivenBindingError {
    InvalidTransaction,
    DependencyMismatch,
    InvalidBaseBindingEnvironment,
    InvalidAggregate,
    InvalidDeclaration { binding: SourceProofLocalGivenBindingId },
    InvalidBindingEnvironment,
    InvalidInstallation,
}
```

The error implements `std::error::Error`; the signatures above are complete,
including constness, return types, derives, and non-exhaustive attributes.

All referenced values are existing owner-module types. No `SurfaceAst`, syntax
node/shell, `SymbolEnv`, source text, type expression, condition, formula,
goal, fact, proof, or obligation crosses the checker ABI. The sole lower token
is the complete byte-exact `source-proof-local-given-lower-debug-v1` string.

### Exact transaction and lexical scope matrix

Dependency validation fixes theorem `19..128`, proof `62..127`, `given`
`70..108`, segment `76..87`, name `76..77`, source ordinal `1`, definition/
contribution `0/0`, and spelling `y`. The runner creates the syntax-free local
identity at scope `[0]`, declaration `76..77`, visible-after ordinal `1`.
`set@84..87` stays inside the opaque lower fingerprint and the binding type
site remains `Missing` until Task 269GT.

The exact normal reserve base is `1/1/0`. The atomic transition is
`1/1/0 -> 2/2/0`: context 1 is `SourceStatement(62..127)`, parent 0, proof
layer, scope `[0]`, owned `[1]`, visible `[0,1]`, normal; binding 1 is `y`,
`GivenWitness`, resolver-local `([0], ordinal=1, range=76..77)`, owner context
1, visible-after 1, missing type, active, uncaptured, diagnostic-free, normal;
row 0 records binding/context `1/1`, source/visible-after `1/1`, normal. Context
0 and binding 0 remain byte-identical.

The installed environment must reproduce:

| intent | context / scope / ordinal | result |
|---|---|---|
| before declaration | `1 / [0] / 1` | forward binding 1 |
| same-statement `such that` | `1 / [0] / 2` | local binding 1 |
| first later statement | `1 / [0] / 2` | local binding 1 |
| inherited child | `2 / [0,0] / 2` | local binding 1 |
| parent | `0 / [] / 2` | unresolved |
| sibling | `4 / [1] / 2` | unresolved |

The two proof-context ordinal-2 rows are distinct test intents over one lexical
table; this task creates no condition-use or later-use source row. The three
non-proof contexts and one shadow row exist only in the checker test-derived
environment, with every `BindingEnv` field frozen as follows:

- context 2: owner `Generated("task269g-unshadowed-child")`, parent 1, layer
  `Block`, lexical scope `[0,0]`, owned `[]`, visible `[0,1]`, recovery
  `Normal`;
- context 3: owner `Generated("task269g-shadow-child")`, parent 1, layer
  `Block`, lexical scope `[0,1]`, owned `[2]`, visible `[0,1,2]`, recovery
  `Normal`;
- context 4: owner `Generated("task269g-sibling")`, parent 0, layer `Block`,
  lexical scope `[1]`, owned `[]`, visible `[0]`, recovery `Normal`;
- binding 2: spelling `y`, kind `GivenWitness`, identity
  `ResolverLocal(scope=[0,1], ordinal=2, declaration_range=109..110)`, owner
  context 3, declaration `109..110`, visible-after 2, type `Missing`, status
  `Active`, empty captured variables and diagnostics, recovery `Normal`.

The synthetic `109..110` test-only range is only a deterministic table key and
does not assert a second source declaration. Lookup in context 3 at ordinal 3
selects binding 2, while lookup after child exit in context 1 at ordinal 3
restores binding 1. This matrix never enters the handoff, runner output, or
production context table.

Validation precedence is transaction identity; lower/theorem/ranges; base;
dense aggregate; local/row fields; final environment/fingerprints/lookups;
Typed/final availability. Failure is atomic. Exact display text is:

| variant | exact text |
|---|---|
| `InvalidTransaction` | `source proof-local given-binding transaction is invalid` |
| `DependencyMismatch` | `source proof-local given-binding dependency mismatch` |
| `InvalidBaseBindingEnvironment` | `source proof-local given-binding base binding environment is invalid` |
| `InvalidAggregate` | `source proof-local given-binding aggregate is invalid` |
| `InvalidDeclaration { binding }` | `source proof-local given-binding <binding.index()> is invalid` |
| `InvalidBindingEnvironment` | `source proof-local given-binding binding environment is invalid` |
| `InvalidInstallation` | `source proof-local given-binding installation is invalid` |

Stable debug is exactly:

```text
source-proof-local-given-binding-debug-v1
module: <package>::<path>
lower-fingerprint: <quoted Task-269GP debug bytes>
theorem symbol=<quoted-fqn> definition=0 contribution=0 range=19..128 proof=62..127
given range=70..108 segment=76..87 name=76..77 source_ordinal=1
base-binding-fingerprint: <quoted BindingEnv debug bytes>
binding#0 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal
final-binding-fingerprint: <quoted BindingEnv debug bytes>
```

Quoted fields use Rust `Debug`; order is fixed, with no blank line and one
final LF.

### Typed/final ownership and exclusions

`TypedAst` adds private optional `source_proof_local_given_binding` and exactly
these methods; `TypedAstParts` receives no replacement field:

```rust
pub const fn source_proof_local_given_binding(
    &self,
) -> Option<&SourceProofLocalGivenBindingHandoff>;

pub fn with_source_proof_local_given_binding(
    self,
    handoff: SourceProofLocalGivenBindingHandoff,
) -> Result<Self, TypedAstError>;
```

The installer consumes the handoff by value and is one-shot. `TypedAstError` adds
`InvalidSourceProofLocalGivenBinding`, text
`typed AST source proof-local given-binding handoff is inconsistent`.
Installation accepts only an otherwise-empty profile and publishes no node,
context/type/fact/coercion/initial-obligation/diagnostic row or other handoff.

`ResolvedTypedAst` adds exactly this read-only getter:

```rust
pub const fn source_proof_local_given_binding(
    &self,
) -> Option<&SourceProofLocalGivenBindingHandoff>;
```

It clone-preserves the handoff only after replay. `ResolvedTypedAstInputs` has
no replacement path.
`ResolvedTypedAstError` adds `InvalidSourceProofLocalGivenBinding`, text
`resolved typed AST source proof-local given-binding handoff is inconsistent`.
Debug uses the proof-local slot after the existing `let` binding/type slot;
cross-family ownership is mutually exclusive.

Task 269G creates no condition/label/fact, theorem fact, type guard/source
type, goal/thesis transition, proof/discharge/acceptance, Core/CFG/VC/ATP,
diagnostic mapping, active dispatch/corpus outcome, or source use/capture row.
Only Task 269GT may consume the handoff by value and admit `set@84..87`.
Multi-segment `given`, `consider`, free-witness export, and Task 270 remain
separate.

### Frozen files, tests, measurements, audit, and exit

Implementation may change exactly eight existing Rust files:

1. `crates/mizar-checker/src/binding_env.rs`;
2. `crates/mizar-checker/src/source_proof_local_declaration.rs`;
3. `crates/mizar-checker/src/typed_ast.rs`;
4. `crates/mizar-checker/src/resolved_typed_ast.rs`;
5. `crates/mizar-test/src/runner/type_elaboration/source_proof_local_declaration.rs`;
6. `crates/mizar-test/src/runner/type_elaboration.rs`;
7. `crates/mizar-test/src/runner.rs`;
8. `crates/mizar-test/src/runner/tests/type_elaboration/source_proof_local_declaration.rs`.

No path/module, parser/resolver/lower source, `.miz`, sidecar, expectation,
trace row/status/backlink, metadata assertion, Cargo file, public runner
route, active dispatch, or diagnostic key changes. The exact checker tests are:

1. `source_proof_local_given_binding_builds_exact_scope_transaction`;
2. `source_proof_local_given_binding_rejects_corruption_with_stable_precedence`;
3. `source_proof_local_given_binding_typed_and_resolved_ownership_is_atomic`;
4. `source_proof_local_given_binding_scope_matrix_is_lexical_and_semantically_empty`.

The exact runner tests are:

1. `task269g_exact_given_binding_transaction_debug_and_lookup_are_stable`;
2. `task269g_lower_base_and_checker_corruption_fail_closed`;
3. `task269g_typed_and_resolved_owners_are_one_shot_and_semantically_empty`;
4. `task269g_near_miss_neighbor_and_active_routes_remain_isolated`.

The checker corruption test fixes one seam per validation tier: wrong
input/handoff source or module gives `InvalidTransaction`; wrong lower,
theorem, or range gives `DependencyMismatch`; empty/wrong reserve base gives
`InvalidBaseBindingEnvironment`; truncated dense rows gives
`InvalidAggregate`; a mutated row/local gives `InvalidDeclaration`; a wrong
final fingerprint or lookup environment gives `InvalidBindingEnvironment`;
unavailable, duplicate, cross-family, or rollback-broken Typed/Resolved replay
gives `InvalidInstallation`. Each of the seven variants is asserted, and the
documented first-error precedence is asserted on combined corruptions.

The exact checker-only mutation methods are
`set_lower_fingerprint_for_test`,
`set_base_binding_fingerprint_for_task269g_test`,
`truncate_task269g_bindings_for_test`,
`corrupt_task269g_binding_row_for_test`, and
`set_final_binding_fingerprint_for_task269g_test`. Direct same-module test
mutation of the private final environment supplies the lookup-environment
case; none of these seams is a public or runner API.

The runner corruption enum is exactly `None`, `WrongLowerFingerprint`,
`EmptyBase`, `WrongTheoremRange`, `WrongProofRange`, `WrongGivenRange`,
`WrongSegmentRange`, `WrongNameRange`, `WrongLocalSpelling`,
`WrongLocalScope`, `WrongLocalRange`, `WrongLocalVisibleAfter`, and
`WrongSourceOrdinal`; all non-`None` routes fail closed before publication.
Typed/Resolved tests exercise initial install, duplicate replay, cross-family
replay, rollback, and post-build mutation without publishing semantic rows.

Docs baseline checker/runner libraries is `490/552`, projected `494/556`.
Raw/normalized hashes are checker
`10e1f56783a472b63a0473893196d68b54a7a7aa3a3aff4f66e74ac42b4a2ad2` /
`21d65f467319e2e7ac463344902b10dfce5716a96c41a87e879326c293ff36e0`
and runner
`9dff9057edba19fe41f71bfa2936f6708438f4a9c969b4b87f9da40641710cd0` /
`fb55cd699daaf5beb28077eb36385cf16eedae43e38fe4385244f632ea4e54e2`.
Production paths stay `30/37`, currently `168322/72916` lines. Path/content
hashes are checker
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`4d0c793a47dac672e5f395c9c2b9e7c9274b5d776b54870888ba5c918f751dc2`
and runner
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d` /
`532d96defde8f63fa821a4f619c21699069eed19c8f48d50be1f1516be0dac63`.
After implementation, both libraries' raw/normalized test-list hashes and
both production line/content measurements are remeasured; path hashes stay
fixed. The values above remain the unchanged documentation-prerequisite
baseline.

Corpus/requirements `428/395`, pass/fail `235/193`, warnings/errors `23/0`,
active stages `101/7/205/1`, type coverage `259=247+12`, trace, and all five
CLI hashes remain unchanged. The coverage audit may credit only focused
private lexical-binding tests/ownership, never active `.miz`, trace, type,
condition/fact, proof, or downstream coverage.

Exit requires synchronized EN/JA, repeated spec review **NO FINDINGS**, all
nine docs-only gates and uncapped `>=90/100`, exact docs commit, fresh
lower-stage preflight, exact eight-file implementation, separate no-findings
test/implementation/source-doc reviews, full verification/count/hash, final
nine gates and score, task-only commit, clean inventory, and protected-stash
identity before selecting Task 269GT.

### Task-269G implementation closure

The exact checker transaction, boxed Typed/final ownership, and private dormant
runner consumer are implemented in the frozen eight Rust files. The producer
authenticates the unchanged Task-269GP lower row and reserve-only base, emits
one dense `GivenWitness` row, preserves the `1/1/0 -> 2/2/0` environment
transition, and retains `BindingTypeSite::Missing`. The checker scope matrix
proves block inheritance, shadowing, restoration, and parent/sibling exclusion.
All validation and cross-family failures roll back before publication.

The exact four checker and four runner tests raise library inventories to
`494/556`. Raw/normalized test-list hashes are checker
`ce299dfafb8db5d5c27cb9e271dd77d08a09b45a7323d0efc17790e0d104a984` /
`6d8f1938b05118e129f8d0942bd7af77914435b6b45282bd46e636132891d4cb`
and runner
`194b2884a9d933823e0d06b24460cd510fd9d16fbd6823b9e13584779acd1f03` /
`728a5b688c19acc42d66a9c2f5c13ad67d795949ec88a2d877b917c9607d80e8`.
Production is checker `30/169847` and runner `37/73118`; path hashes remain
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`
and content hashes are
`e47862eebdb59b576160d4b64ab390549d91daecd69fd34f8bcfbc2952d6ca96` /
`2cae769737fdee4560ab1d1bca81f10d900ff8a1d9824aba720806f84e802711`.

No `.miz`, sidecar, expectation, trace, metadata, Cargo, parser, resolver,
active dispatch, diagnostic key, condition/fact, source-type, goal, proof,
discharge, acceptance, initial obligation, Core, CFG, or VC owner changes.
Corpus/count/CLI and trace values remain the frozen Task-264 baseline. Test-
sufficiency, implementation, source/docs, and final-quality reviews end **NO
FINDINGS**. Focused/crate/workspace/lint/metadata/fmt/Clippy/CLI/count/hash/
whitespace verification passes; all nine hard gates PASS uncapped at
`100/100` (`20/20/15/15/10/10/5/5`). Only parent-owned staging, commit, and
fresh inventory remain before the separate source-type-only Task 269GT.

## Checker Task 269GT Frozen Given-Type Consumer

Task 269GT consumes `SourceProofLocalGivenBindingHandoff` by value, preserves
its lower and binding fingerprints, and overlays only binding 1's exact
`set@84..87` source type in a new `source_type.rs` composite. This document's
Given binding ABI, lexical scope matrix, and debug bytes remain unchanged.
Condition/fact/use/capture/free-export and every proof semantic stay outside
the consumer.

Completion evidence: [central Task-269GT historical contract](../../task_contracts/en/269GT.md#completion-evidence).

## Checker Task 269GUP Frozen Use-profile Binding Prerequisite

Task 269GUP authenticates only the binding profile required before later-use
transport. Its canonical authority is Spec 4.6.1, 15.3.3, 15.10, 16.3.3, and
16.4.2 plus the human-confirmed rule: a `given` variable is visible through
the remainder of the corresponding block and descendants unless shadowed,
but not in parent or sibling blocks. The 128-byte sibling is a distinct source
transaction. GUP therefore derives a new checker-local `BindingId(1)` in that
source's own `BindingEnv`; it does not claim object identity with Task-269G's
old binding. `BinderIdentity::ResolverLocal(scope=[0], ordinal=1,
range=76..77)` supplies provenance, while checker `BindingEnv::lookup`
authenticates the scope rule. No resolver API is added.

The only accepted source is:

```mizar
reserve x for set;
theorem FormulaStatementGivenSmoke: thesis proof
  given y being set such that G: thesis;
  thus y = y;
end;
```

It is exactly 128 bytes, ends in one LF, has source SHA-256
`ec15ded78ae96022840a8419a85d74643de3b37337e9a202cbda77ee97aa7c01`,
and has the exact 54-node Surface fingerprint
`c64297ce72e380a2e4146276966e085d780f8b38f2528d5abaa440a50c67db6d`.
The parser profile is root 53 with token nodes `0..26`, no expression root,
diagnostic, or recovery. The reserve shell is `0..18`; the theorem shell is
`19..127`, its formula is `55..61`, proof is `62..126`, Given is `70..108`,
declaration/name is `76..77`, written type is `84..87`, and the conclusion is
`111..122`. The two `TermReference` leaves `y@116..117` and `y@120..121`,
equality, conclusion wrapper, condition/label, theorem formula, proof shell,
and every other non-declaration subtree are selector-only exclusions. GUP
publishes no term or later-use row.

### Exact runner-private lower ABI

The old 129-byte Task-269GP/G/GT selectors and validators remain byte/profile
exact and reject this source. GUP adds a distinct runner-private
`SOURCE_PROOF_LOCAL_GIVEN_USE_TEXT`, `SourceProofLocalGivenUseLowerOutput`, and
`source_proof_local_given_use_lower_output(...)`. The output derives
`Debug, Clone, PartialEq, Eq`; its private fields, in order, are `source_id:
SourceId`, `module_id: ModuleId`, `source_fingerprint: String`,
`surface_fingerprint: String`, `theorem_symbol: SymbolId`,
`theorem_definition: DefinitionId`, `contribution: SourceContributionId`,
`theorem_range`, `proof_range`, `given_range`, `segment_range`, and
`name_range: SourceRange`, `name_spelling: String`, `type_range` and
`type_head_range: SourceRange`, `type_spelling: String`, and
`source_ordinal: usize`.

Every field has the corresponding read-only `pub(in crate::runner)` getter;
copy fields are `const fn`, string/symbol/module getters return references, and
`debug_text() -> String` is non-const. The function signature takes
`&SurfaceAst`, `ModuleId`, `&DeclarationShellSet`, `&SymbolEnv`, and `&str`,
returning `Option<Result<SourceProofLocalGivenUseLowerOutput, String>>`.
Selector mismatch is `None`; selected validation failure is `Some(Err(_))`.

The getter names, in field order, are `source_id`, `module_id`,
`source_fingerprint`, `surface_fingerprint`, `theorem_symbol`,
`theorem_definition`, `contribution`, `theorem_range`, `proof_range`,
`given_range`, `segment_range`, `name_range`, `name_spelling`, `type_range`,
`type_head_range`, `type_spelling`, and `source_ordinal`, followed by
`debug_text`.

The exact lower debug bytes end in one LF:

```text
source-proof-local-given-use-lower-debug-v1
module: {package}::{module}
source-fingerprint: "ec15ded78ae96022840a8419a85d74643de3b37337e9a202cbda77ee97aa7c01"
surface-fingerprint: "c64297ce72e380a2e4146276966e085d780f8b38f2528d5abaa440a50c67db6d"
theorem symbol="{fqn}" definition=0 contribution=0 range=19..127 proof=62..126
given range=70..108 segment=76..87 source_ordinal=1
name range=76..77 spelling="y"
type range=84..87 head=84..87 spelling="set" form=bare
```

The selector authenticates every node kind/source/range/children/recovery and
every token kind/text. Role rows are reserve `30@0..18`; theorem thesis
`31/32@55..61`; Given type `33/34@84..87`, segment `35@76..87`, condition
`36..39@93..107`, Given `40@70..108`; excluded term pairs
`41/42@116..117` and `43/44@120..121`; excluded predicate/formula/proposition
`45..47@116..121`; conclusion `48@111..122`; proof `49@62..126`; theorem
`50@19..127`; item-list/compilation/root `51/52/53@0..127`. The exact ordered
children are the measured 54-node snapshot represented by the frozen Surface
SHA, not inferred during production.

Four test-only mutation enums use the GUP prefix and exactly the Task-269GP
variant sets: `Surface` has `None`, `ExpressionRoot`, `TokenNode(usize)`,
`TokenNodeCount`, `NodeKind/NodeSourceId/NodeRange/NodeRecovery/NodeChildren
(usize)`, `MissingRootIdentity`, `WrongRootIdentity`; `Lower` has `None`,
`SourceId`, `Module`, both fingerprints, theorem symbol/definition,
contribution, every retained range/spelling, and source ordinal; `Shell` has
`None` plus `Id/Ordinal/Kind/Module/Node/Syntax/Range/Parent/VisibilityState/
VisibilityMarker/VisibilitySpelling/Recovery(usize)`; `ResolverProfile` has
`None`, resolver module, every normally empty index, every theorem
symbol/definition provenance field, and every contribution-effect field. The
five test-only lower seams have the base name plus `_with_surface_mutation`,
`_with_mutation`, `_with_shell_mutation`, `_with_resolver_profile_mutation`,
and `_with_resolver_mutation`; the last takes
`impl FnOnce(SymbolEnv) -> SymbolEnv`.

The private binding route mutation enum is
`SourceProofLocalGivenUseBindingRouteMutation` with exact variants `None`,
`WrongLowerFingerprint`, `EmptyBase`, `WrongTheoremRange`, `WrongProofRange`,
`WrongGivenRange`, `WrongSegmentRange`, `WrongNameRange`,
`WrongLocalSpelling`, `WrongLocalScope`, `WrongLocalRange`,
`WrongLocalVisibleAfter`, and `WrongSourceOrdinal`. The cfg-test seam appends
that mutation to the production route signature. The literal handoff seam is:

```rust
pub(in crate::runner) fn source_proof_local_given_use_binding_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalGivenUseBindingHandoff, String>>;

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_use_binding_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenUseBindingRouteMutation,
) -> Option<Result<SourceProofLocalGivenUseBindingHandoff, String>>;
```

Both routes are dormant and `pub(in crate::runner)`. A selector mismatch is
`None`; all selected lower/base/producer failures are `Some(Err(_))`; success
is the public binding handoff itself, which Task 269GUPT must consume by value.
No temporary Typed/Resolved wrapper is introduced.

Validation precedence is path-specific. The private lower path is Surface,
shells, resolver module/empty indexes, theorem symbol/definition/contribution,
lower row, then lower debug. The binding route runs that path, reserve
extraction/base construction, exact input construction, then producer build.
Producer `build` checks transaction, dependency, base environment, input
declaration, constructed environment/lookup, and final fingerprint; its own
one-row aggregate cannot fail cardinality. Handoff `validate_installation`
checks transaction, dependency, base environment/fingerprint, aggregate
cardinality, row/declaration, then reconstructed final environment/lookup/
fingerprint. Combined-failure tests target each path independently. Exact
private error strings are
the following complete list; no parser debug dump or additional diagnostic is
exposed:

```text
Task269GUP exact Surface identity changed after selection
Task269GUP requires exactly two declaration shells
Task269GUP resolver shells unexpectedly export a path
Task269GUP declaration shell {ordinal} mismatch
Task269GUP raw resolver module mismatch
Task269GUP local y already resolves as a module symbol
Task269GUP raw resolver inventory mismatch
Task269GUP requires one exact theorem owner
Task269GUP exact theorem owner provenance mismatch
Task269GUP requires one exact theorem definition
Task269GUP theorem contribution is missing
Task269GUP theorem symbol provenance mismatch
Task269GUP theorem definition provenance mismatch
Task269GUP theorem contribution provenance mismatch
Task269GUP private lower output mismatch
Task269GUP private lower debug grammar mismatch
Task269GUP exact reserve base extraction failed
Task269GUP exact reserve base failed: {error}
```

### Exact public binding ABI

The checker adds only this public family in
`source_proof_local_declaration.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenUseBindingHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub lower_fingerprint: String,
    pub theorem_symbol: SymbolId,
    pub theorem_definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub theorem_range: SourceRange,
    pub proof_range: SourceRange,
    pub given_range: SourceRange,
    pub segment_range: SourceRange,
    pub name_range: SourceRange,
    pub source_ordinal: usize,
    pub local: LocalTermBinding,
    pub recovery: SourceProofLocalGivenBindingRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenUseBindingHandoff { /* frozen fields below */ }

#[derive(Debug, Clone, Copy, Default)]
pub struct SourceProofLocalGivenUseBindingProducer;

impl SourceProofLocalGivenUseBindingProducer {
    pub fn build(
        input: SourceProofLocalGivenUseBindingHandoffInput,
        base_binding_env: &BindingEnv,
    ) -> Result<SourceProofLocalGivenUseBindingHandoff,
                SourceProofLocalGivenUseBindingError>;
}
```

The handoff's exact private layout and read-only API are:

```rust
pub struct SourceProofLocalGivenUseBindingHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    lower_fingerprint: String,
    theorem_symbol: SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    given_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
    base_binding_env: BindingEnv,
    base_binding_fingerprint: String,
    binding_env: BindingEnv,
    final_binding_fingerprint: String,
    bindings: SourceProofLocalGivenBindingTable,
}

impl SourceProofLocalGivenUseBindingHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn lower_fingerprint(&self) -> &str;
    pub const fn theorem_symbol(&self) -> &SymbolId;
    pub const fn theorem_definition(&self) -> DefinitionId;
    pub const fn contribution(&self) -> SourceContributionId;
    pub const fn theorem_range(&self) -> SourceRange;
    pub const fn proof_range(&self) -> SourceRange;
    pub const fn given_range(&self) -> SourceRange;
    pub const fn segment_range(&self) -> SourceRange;
    pub const fn name_range(&self) -> SourceRange;
    pub const fn base_binding_env(&self) -> &BindingEnv;
    pub fn base_binding_fingerprint(&self) -> &str;
    pub const fn binding_env(&self) -> &BindingEnv;
    pub fn final_binding_fingerprint(&self) -> &str;
    pub const fn bindings(&self) -> &SourceProofLocalGivenBindingTable;
    pub fn debug_text(&self) -> String;
    pub(crate) fn validate_installation(
        &self, source_id: SourceId, module_id: &ModuleId,
    ) -> Result<(), SourceProofLocalGivenUseBindingError>;
}
```

The old dense row/table/recovery types are reused; no duplicate binding-row
ABI is created. All public structs/enums remain ordinary syntax-free checker
payloads and expose no Surface node or parser type.

`SourceProofLocalGivenUseBindingError` derives `Debug, Clone, PartialEq, Eq`,
is `#[non_exhaustive]`, implements `Display` and `Error`, and has variants
`InvalidTransaction`, `DependencyMismatch`,
`InvalidBaseBindingEnvironment`, `InvalidAggregate`,
`InvalidDeclaration { binding: SourceProofLocalGivenBindingId }`, and
`InvalidBindingEnvironment`. Display strings replace the old family prefix
with `source proof-local given-use binding`; the declaration string inserts
the dense binding index. There is no installation variant because GUP has no
Typed/final owner.

The exact `Display` strings are `source proof-local given-use binding
transaction is invalid`, `source proof-local given-use binding dependency
mismatch`, `source proof-local given-use binding base binding environment is
invalid`, `source proof-local given-use binding aggregate is invalid`, `source
proof-local given-use binding {binding.index()} is invalid`, and `source
proof-local given-use binding binding environment is invalid`.

The handoff debug header is
`source-proof-local-given-use-binding-debug-v1`. The remaining labels, field
order, quoting, row rendering, fingerprint rendering, and final LF are exactly
the Task-269G binding debug grammar, with theorem/proof ranges
`19..127`/`62..126` and the unique GUP lower fingerprint above.

```text
source-proof-local-given-use-binding-debug-v1
module: {package}::{module}
lower-fingerprint: {quoted exact lower debug}
theorem symbol={quoted fqn} definition=0 contribution=0 range=19..127 proof=62..126
given range=70..108 segment=76..87 name=76..77 source_ordinal=1
base-binding-fingerprint: {quoted exact base BindingEnv debug}
binding#0 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal
final-binding-fingerprint: {quoted exact final BindingEnv debug}
```

### Exact binding profile and lookup matrix

The base remains the exact reserve-only `1 context / 1 binding / 0
diagnostics` environment. The output is exactly `2/2/0`; context 1 is a normal
proof context owned by `SourceStatement(62..126)`, parent 0, scope `[0]`,
bindings `[1]`, visible `[0,1]`. Binding 1 is active normal `GivenWitness`,
owned by context 1, declaration `76..77`, visible after ordinal 1, type
`Missing`, resolver-local identity `([0],1,76..77)`, empty capture and
diagnostics. No sort, repair, or mutation of binding/context 0 is permitted.

Lookup at definition ordinal 1 is the exact forward-reference result and
lookup at ordinal 2 is `Local(BindingId(1))`. Tests also freeze inherited-child
selection, inner-shadow selection, restoration to binding 1, and parent and
sibling exclusion. These tests authenticate the user-approved block lifetime;
GUP still publishes no source occurrence.

### Scope, tests, impact, and exit

The exact Rust write set is checker
`source_proof_local_declaration.rs`; runner `source_statement.rs`,
`source_proof_local_declaration.rs`, `type_elaboration.rs`, `runner.rs`, and
the existing proof-local test leaf. The docs stage set is exactly 40 paired
EN/JA plan/todo/audit/owner/trace files plus the two global ledgers
`doc/design/spec_coverage_audit.md` and `doc/design/todo.md`. Checker/runner libraries project
`498 -> 502` and `560 -> 564`.

The four checker tests are
`source_proof_local_given_use_binding_is_exact_and_new_source_local`,
`source_proof_local_given_use_binding_rejects_every_corruption_in_precedence`,
`source_proof_local_given_use_binding_inherits_shadows_restores_and_excludes`,
and `source_proof_local_given_use_binding_has_no_type_term_or_semantic_owner`.
The four runner tests are
`source_proof_local_given_use_binding_profile_is_exact_and_private`,
`source_proof_local_given_use_binding_profile_rejects_every_corruption`,
`source_proof_local_given_use_binding_profile_isolates_old_and_near_miss_sources`,
and `source_proof_local_given_use_binding_profile_has_zero_semantic_effect`.
They cover every lower Surface/shell/resolver/output mutation, every public
input field, both fingerprints, aggregate/row/environment corruption, combined
failure precedence, old/new GP/G/GT rejection, deterministic debug replay, and
the full lookup matrix.

No source type, term/reference/request, Typed/final owner, arena, Task-252
allowlist, `.miz`, sidecar, expectation, trace, metadata, Cargo, public
dispatch, CLI, diagnostic, or active result may change. No condition/label
fact, existential/Skolem interpretation, assumption/guard, equality/formula
truth, goal, obligation, proof/acceptance, export, capture/closure/substitution,
Core, CFG, or VC row is created. This task exits only after all reviews end NO
FINDINGS, all nine hard gates pass uncapped at least 90/100, exact staging is
audited, and docs/implementation are separate commits. Fresh inventory then
selects Task 269GUPT; Task 269GU, capture, and Task 270 remain deferred.
Completion evidence: [central Task-269GUP historical contract](../../task_contracts/en/269GUP.md#completion-evidence).

## Task 269GUPT Frozen Dependency Consumer

The public GUP binding handoff is consumed by value only by `SourceProofLocalGivenUseTypeProducer`. GUPT uses the unchanged private GUP lower seam solely to recover authenticated `84..87`; it does not alter the 128-byte selector, 54-node Surface profile, lower fingerprint, resolver provenance, binding rows, lookup lifetime, or GUP public ABI. The new type composite preserves the complete dependency debug text as its dependency fingerprint. Task 269GU remains the first permitted later-identifier consumer.

Completion evidence: [central Task-269GUPT historical contract](../../task_contracts/en/269GUPT.md#completion-evidence).

## Task 269GU Frozen Binding Dependency

GU consumes the committed GUPT composite by value and uses only its immutable
typed binding environment. GUP lower/source/shell/resolver fingerprints,
declaration row, contexts, scope `[0]`, visibility ordinal 1, and public API
remain unchanged. The two later use lookups derive ordinal 2 in
`source_term.rs`; this module gains no occurrence, reference, capture, fact,
or proof owner.

Completion evidence: [central Task-269GU historical contract](../../task_contracts/en/269GU.md#completion-evidence).

## Task 269GCP Frozen Given-condition Lower Prerequisite

Fresh clean post-GU inventory at
`998dc104957d47e2707f4a8292d2002f1c5beb2d` selects only the runner-private
lower prerequisite for a `given` witness used in its own declaration
condition. Canonical Chapters 4 §4.6.1, 15 §§15.3.3/15.10, and 16
§§16.3.3/16.4.2 explicitly bind these occurrences. Existing parser and broad
proof-local fixtures prove syntax reachability but contain no such occurrence.
The missing exact profile is `source_drift` plus `test_gap`; this frozen record
repairs `design_drift`. There is no blocking `spec_gap`.

The exact dormant final-LF source is 134 bytes:

```mizar
reserve x for set;
theorem ProofLocalGivenConditionUseSmoke: thesis proof
  given y being set such that G: y = y;
  thus thesis;
end;
```

Its source SHA-256 is
`2c2d767a0654670412b377bdcc6c5970ecec05b41c02aa754766320927bc6aad`.
Read-only frontend preflight reports no diagnostics and freezes a 54-node
Surface arena with root 53, root range `0..133`, and snapshot SHA-256
`49d46d5f24338772e6e968f12c2216a8957b35242474132690db843b510b430f`.
Token nodes are 0--26. Structural nodes are reserve type head/expression/
segment/item 27--30; theorem thesis constant/formula 31--32; Given type
head/expression/segment 33--35; condition terms/references 36--39 at
`107..108` and `111..112`; equality 40, formula 41, proposition 42,
condition list 43, Given statement 44; final thesis constant/formula/
proposition/conclusion 45--48; proof 49; theorem 50; item list/compilation/root
51--53. The exact retained ranges are theorem `19..133`, proof `68..132`,
Given `76..113`, segment `82..93`, name `82..83`, and written bare builtin
`set` type/head `90..93`, with source ordinal 1. The two condition references
remain selector-authenticated but are not published by GCP.

Resolver preflight freezes exactly two declaration shells: reserve shell
0/node 30/range `0..18` and theorem shell 1/node 50/range `19..133`, both
normal, parentless, and visibility-unspecified, with no export shell. The
symbol environment has one public/exported local theorem symbol, one theorem
definition, and one contribution anchored at `0..18`; the theorem origin is
`19..133` with structural path `[2,1]`. Imports, exports, labels, overloads,
registrations, lexical summaries, namespace edges, declaration dependencies,
module summaries, relations, parameters, binders, diagnostics, and all other
contribution effects are empty. The opaque signature schema is exactly
`parser-signature-v1` and its payload is the following one-line byte string:

```text
node=TheoremItem;symbol=theorem;definition=theorem;primary_tokens=theorem ProofLocalGivenConditionUseSmoke : thesis proof given y being set such that G : y = y ; thus thesis ; end ;;notation=_;arity=_;roles=FormulaExpression,ProofBlock
```

Neither `y` occurrence is promoted to a module `SymbolId`.

Implementation may change exactly four existing runner files:
`runner.rs`, `runner/type_elaboration.rs`,
`runner/type_elaboration/source_statement.rs`, and
`runner/tests/type_elaboration/source_proof_local_declaration.rs`. It adds a
private `SourceProofLocalGivenConditionLowerOutput`, the exact GCP Surface,
lower, shell, and resolver-profile mutation enums, one dormant
production-private base function, and five `#[cfg(test)]` mutation seams,
and four runner tests named
`task269gcp_exact_condition_lower_projection_is_stable`,
`task269gcp_surface_and_lower_corruption_fail_closed`,
`task269gcp_resolver_shell_and_symbol_corruption_fail_closed`, and
`task269gcp_near_miss_and_active_routes_remain_isolated`. Selection mismatch
is `None`; selected validation failure is `Some(Err(_))`; success is the
private immutable lower row. Validation order is Surface, shells, resolver
inventory, theorem symbol/definition/contribution, lower row, then exact debug
bytes.

The output and debug grammar retain only source/module identity, both SHA-256
fingerprints, theorem symbol/definition/contribution, the six declaration
ranges/spellings above, source ordinal, and a final LF. No checker public API,
`BindingEnv`, type/term/reference row, condition/formula/fact, label lifetime,
existential/Skolem state, guard/assumption, capture/export result, goal,
initial obligation, proof/discharge/acceptance, Core/CFG/VC row, Typed owner,
Resolved owner, runner dispatch, diagnostic, fixture, sidecar, expectation,
trace row/status/backlink, metadata case, or active coverage is permitted.

The private row is frozen type-for-type as follows; no field is `pub`, and the
row itself is visible only within `crate::runner`:

```rust
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::runner) struct SourceProofLocalGivenConditionLowerOutput {
    source_id: SourceId,
    module_id: ModuleId,
    source_fingerprint: String,
    surface_fingerprint: String,
    theorem_symbol: SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    given_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
    name_spelling: String,
    type_range: SourceRange,
    type_head_range: SourceRange,
    type_spelling: String,
    source_ordinal: usize,
}
```

Its same-named read-only getter signatures are frozen in field order, followed
by `debug_text`; only the four `String` getters and `debug_text` are
non-const:

```rust
pub(in crate::runner) const fn source_id(&self) -> SourceId;
pub(in crate::runner) const fn module_id(&self) -> &ModuleId;
pub(in crate::runner) fn source_fingerprint(&self) -> &str;
pub(in crate::runner) fn surface_fingerprint(&self) -> &str;
pub(in crate::runner) const fn theorem_symbol(&self) -> &SymbolId;
pub(in crate::runner) const fn theorem_definition(&self) -> DefinitionId;
pub(in crate::runner) const fn contribution(&self) -> SourceContributionId;
pub(in crate::runner) const fn theorem_range(&self) -> SourceRange;
pub(in crate::runner) const fn proof_range(&self) -> SourceRange;
pub(in crate::runner) const fn given_range(&self) -> SourceRange;
pub(in crate::runner) const fn segment_range(&self) -> SourceRange;
pub(in crate::runner) const fn name_range(&self) -> SourceRange;
pub(in crate::runner) fn name_spelling(&self) -> &str;
pub(in crate::runner) const fn type_range(&self) -> SourceRange;
pub(in crate::runner) const fn type_head_range(&self) -> SourceRange;
pub(in crate::runner) fn type_spelling(&self) -> &str;
pub(in crate::runner) const fn source_ordinal(&self) -> usize;
pub(in crate::runner) fn debug_text(&self) -> String;
```

The exact debug bytes are:

```text
source-proof-local-given-condition-lower-debug-v1
module: {package}::{module}
source-fingerprint: "2c2d767a0654670412b377bdcc6c5970ecec05b41c02aa754766320927bc6aad"
surface-fingerprint: "49d46d5f24338772e6e968f12c2216a8957b35242474132690db843b510b430f"
theorem symbol="{fqn}" definition=0 contribution=0 range=19..133 proof=68..132
given range=76..113 segment=82..93 source_ordinal=1
name range=82..83 spelling="y"
type range=90..93 head=90..93 spelling="set" form=bare
```

The four mutation enums derive `Debug, Clone, Copy, PartialEq, Eq`, are
`pub(in crate::runner)`, and have these literal variant sets:

```rust
enum SourceProofLocalGivenConditionSurfaceMutation {
    None,
    ExpressionRoot,
    TokenNode(usize),
    TokenNodeCount,
    NodeKind(usize),
    NodeSourceId(usize),
    NodeRange(usize),
    NodeRecovery(usize),
    NodeChildren(usize),
    MissingRootIdentity,
    WrongRootIdentity,
}

enum SourceProofLocalGivenConditionLowerMutation {
    None,
    SourceId,
    Module,
    SourceFingerprint,
    SurfaceFingerprint,
    TheoremSymbol,
    TheoremDefinition,
    Contribution,
    TheoremRange,
    ProofRange,
    GivenRange,
    SegmentRange,
    NameRange,
    NameSpelling,
    TypeRange,
    TypeHeadRange,
    TypeSpelling,
    SourceOrdinal,
}

enum SourceProofLocalGivenConditionShellMutation {
    None,
    Id(usize),
    Ordinal(usize),
    Kind(usize),
    Module(usize),
    Node(usize),
    Syntax(usize),
    Range(usize),
    Parent(usize),
    VisibilityState(usize),
    VisibilityMarker(usize),
    VisibilitySpelling(usize),
    Recovery(usize),
}

enum SourceProofLocalGivenConditionResolverProfileMutation {
    None,
    ResolverModule,
    ImportIndex,
    ExportIndex,
    LabelIndex,
    OverloadIndex,
    RegistrationIndex,
    LexicalSummaryIndex,
    NamespaceGraph,
    DeclarationDependencyIndex,
    ModuleSummaryIndex,
    SymbolModule,
    SymbolNotation,
    SymbolContribution,
    SymbolRelations,
    SymbolOriginSource,
    SymbolOriginImport,
    DefinitionId,
    DefinitionParameters,
    DefinitionBinders,
    DefinitionNotation,
    DefinitionDoc,
    DefinitionContribution,
    DefinitionConflict,
    DefinitionDependencies,
    ContributionLabelEffect,
    ContributionOverloadEffect,
    ContributionRegistrationEffect,
    ContributionLexicalEffect,
    ContributionNamespaceEffect,
    ContributionDeclarationDependencyEffect,
    ContributionImportEffect,
    ContributionExportEffect,
    ContributionDiagnosticEffect,
}
```

Each declaration above also has the exact attributes
`#[cfg_attr(not(test), allow(dead_code))]` and
`#[derive(Debug, Clone, Copy, PartialEq, Eq)]`. The one dormant base and five
test-only mutation seams are frozen to the same first five parameters and
return type:

```rust
pub(in crate::runner) fn source_proof_local_given_condition_lower_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalGivenConditionLowerOutput, String>>;

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_condition_lower_output_with_surface_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenConditionSurfaceMutation,
) -> Option<Result<SourceProofLocalGivenConditionLowerOutput, String>>;

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_condition_lower_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenConditionLowerMutation,
) -> Option<Result<SourceProofLocalGivenConditionLowerOutput, String>>;

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_condition_lower_output_with_shell_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenConditionShellMutation,
) -> Option<Result<SourceProofLocalGivenConditionLowerOutput, String>>;

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_condition_lower_output_with_resolver_profile_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenConditionResolverProfileMutation,
) -> Option<Result<SourceProofLocalGivenConditionLowerOutput, String>>;

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_condition_lower_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceProofLocalGivenConditionLowerOutput, String>>;
```

The base carries `#[cfg_attr(not(test), allow(dead_code))]`; each mutation seam
carries `#[cfg(test)]`. The complete stable lower-only private error ABI is
exactly these 16 strings; the two GUP binding/base errors do not belong to GCP,
and no additional parser dump or diagnostic is exposed:

```text
Task269GCP exact Surface identity changed after selection
Task269GCP requires exactly two declaration shells
Task269GCP resolver shells unexpectedly export a path
Task269GCP declaration shell {ordinal} mismatch
Task269GCP raw resolver module mismatch
Task269GCP local y already resolves as a module symbol
Task269GCP raw resolver inventory mismatch
Task269GCP requires one exact theorem owner
Task269GCP exact theorem owner provenance mismatch
Task269GCP requires one exact theorem definition
Task269GCP theorem contribution is missing
Task269GCP theorem symbol provenance mismatch
Task269GCP theorem definition provenance mismatch
Task269GCP theorem contribution provenance mismatch
Task269GCP private lower output mismatch
Task269GCP private lower debug grammar mismatch
```

Tests must cover all fields and combined
precedence, every node/token mutation, both shells, every normally empty
resolver index/effect, all theorem symbol/definition/contribution fields, and
exact debug replay. Near misses include the old GP and GUP sources, condition
`G: thesis`, an unlabelled condition, a later-use-only `y = y`, theorem or
witness renaming, altered type/form, recovery, extra item, missing final LF,
and every active corpus route.

GCP is a lower-stage prerequisite only. Task 269GC must next build a distinct
by-value binding profile for this exact source; later GCT/GCU slices retain the
written type and condition occurrence/reference transport. They may not loosen
the exact GUP/GUPT/GU validators or reconstruct bindings in a higher owner.
Descendant occurrence transport follows separately. Free-witness export
enforcement remains gated by the Task-272 block-result/proof owner. Task 269's
`set` capture replay and Task 270's resolver-local inline-definition identity
also remain separate, later prerequisites.

Docs-only baselines are checker/runner libraries `510/572`, parser/resolver/
syntax `226/148/59`, checker production `30/176258`, runner production
`37/75339`, cases/requirements `428/395`, pass/fail `235/193`, warnings/errors
`23/0`, stages `101/7/205/1`, and type coverage `259=247+12`. Implementation
projects runner library `576` and changes no checker test. Production path
hashes remain
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`;
changed runner line/content/test-list hashes must be remeasured. Existing
parser fixture, sidecar, broad fixture, broad sidecar, and trace hashes remain
`bd9a2d473fa84012afb36dab8d0f9c11063dd5618df5a31791d57cba2c027234`,
`7361b50bc564d900e1852deaeaaf804544ad9c8ad0a3321a67c1e31bbaa80f17`,
`5fc4849a77eced7a93d65e0cae000c87b1730070c74aef116d6ca62be896ecd9`,
`8e2c73b1661a37c35887b08af01b42fc886199e7a3fb07db8c1412c69f62fa43`,
and `55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.

The documentation prerequisite owns exactly 42 Markdown files: 28 paired
checker plan/todo/audit/owner records, 12 paired mizar-test plan/todo/audit
records, and the two global ledgers. It changes no Rust, Cargo, canonical
artifact, expectation, trace TOML, or metadata file.

Exit requires synchronized EN/JA frozen records, review-only specification
audit ending **NO FINDINGS**, all nine docs-only hard gates at uncapped
`>=90/100`, exact Markdown staging and a dedicated prerequisite commit. The
implementation then requires fresh parser/resolver/lower/count/hash preflight,
the exact four-file/four-test transaction, separate test/implementation/
source-doc reviews ending **NO FINDINGS**, full verification, all nine hard
gates, task-only staging, a separate commit, and automatic fresh inventory of
Task 269GC.

Completion evidence: [central Task-269GCP historical contract](../../task_contracts/en/269GCP.md#completion-evidence).

## Checker Task 269GC Frozen Given-condition Binding Consumer

Fresh clean post-GCP inventory at
`59eb7de68d83901375883a2a6249796afc6a0de3` selects only Task 269GC. The
canonical rule in Chapters 4 §4.6.1, 15 §§15.3.3/15.10, and 16
§§16.3.3/16.4.2, confirmed by the human semantic decision, is exact: a
`given` variable binds occurrences in its own declaration's `such that`
conditions; for subsequent statements it remains visible through the rest of
the corresponding innermost proof or reasoning block and descendants unless
shadowed; it is absent from the parent, siblings, and every site after block
exit. Labels keep ordinary label scope, and this lexical rule creates no new
condition, fact, proof, discharge, acceptance, goal, guard, or obligation
lifetime.

The implemented GCP lower row authenticates the exact final-LF 134-byte source,
54-node/root-53 Surface tree, two declaration shells, one theorem provenance,
and source/Surface SHA-256 values
`2c2d767a0654670412b377bdcc6c5970ecec05b41c02aa754766320927bc6aad` /
`49d46d5f24338772e6e968f12c2216a8957b35242474132690db843b510b430f`.
Task 269GC consumes only its complete byte-exact
`source-proof-local-given-condition-lower-debug-v1` string plus the unchanged
reserve-only base environment. It does not accept `SurfaceAst`, syntax nodes,
shells, `SymbolEnv`, source text, type syntax, condition syntax, or occurrence
IDs at the checker ABI. Missing binding production/tests are
`source_drift`/`test_gap`; this contract repairs `design_drift`. There is no
blocking `spec_gap`. Origin divergence `0/13` is report-only
`repo_metadata_conflict` and is not repaired.

### Exact public checker ABI

The existing `BindingKind::GivenWitness`,
`SourceProofLocalGivenBindingId`, `SourceProofLocalGivenBindingRecovery`,
`SourceProofLocalGivenBinding`, and `SourceProofLocalGivenBindingTable` remain
byte-for-byte unchanged and are reused only as common row vocabulary. G, GUP,
and GC handoff identities are distinct. The existing checker module
`source_proof_local_declaration` adds exactly this sibling family; fields are
private except for the input transaction, and no unchecked constructor or
mutable public accessor exists:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenConditionBindingHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub lower_fingerprint: String,
    pub theorem_symbol: SymbolId,
    pub theorem_definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub theorem_range: SourceRange,
    pub proof_range: SourceRange,
    pub given_range: SourceRange,
    pub segment_range: SourceRange,
    pub name_range: SourceRange,
    pub source_ordinal: usize,
    pub local: LocalTermBinding,
    pub recovery: SourceProofLocalGivenBindingRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalGivenConditionBindingHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    lower_fingerprint: String,
    theorem_symbol: SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    given_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
    base_binding_env: BindingEnv,
    base_binding_fingerprint: String,
    binding_env: BindingEnv,
    final_binding_fingerprint: String,
    bindings: SourceProofLocalGivenBindingTable,
}

impl SourceProofLocalGivenConditionBindingHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn lower_fingerprint(&self) -> &str;
    pub const fn theorem_symbol(&self) -> &SymbolId;
    pub const fn theorem_definition(&self) -> DefinitionId;
    pub const fn contribution(&self) -> SourceContributionId;
    pub const fn theorem_range(&self) -> SourceRange;
    pub const fn proof_range(&self) -> SourceRange;
    pub const fn given_range(&self) -> SourceRange;
    pub const fn segment_range(&self) -> SourceRange;
    pub const fn name_range(&self) -> SourceRange;
    pub const fn base_binding_env(&self) -> &BindingEnv;
    pub fn base_binding_fingerprint(&self) -> &str;
    pub const fn binding_env(&self) -> &BindingEnv;
    pub fn final_binding_fingerprint(&self) -> &str;
    pub const fn bindings(&self) -> &SourceProofLocalGivenBindingTable;
    pub fn debug_text(&self) -> String;

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
    ) -> Result<(), SourceProofLocalGivenConditionBindingError>;

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalGivenConditionBindingError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SourceProofLocalGivenConditionBindingProducer;

impl SourceProofLocalGivenConditionBindingProducer {
    pub fn build(
        input: SourceProofLocalGivenConditionBindingHandoffInput,
        base_binding_env: &BindingEnv,
    ) -> Result<
        SourceProofLocalGivenConditionBindingHandoff,
        SourceProofLocalGivenConditionBindingError,
    >;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalGivenConditionBindingError {
    InvalidTransaction,
    DependencyMismatch,
    InvalidBaseBindingEnvironment,
    InvalidAggregate,
    InvalidDeclaration { binding: SourceProofLocalGivenBindingId },
    InvalidBindingEnvironment,
    InvalidInstallation,
}
```

The error implements `std::error::Error`. All signatures, derives,
non-exhaustive attributes, constness, return types, and field order above are
complete. Exact display text is:

| variant | exact text |
|---|---|
| `InvalidTransaction` | `source proof-local given-condition binding transaction is invalid` |
| `DependencyMismatch` | `source proof-local given-condition binding dependency mismatch` |
| `InvalidBaseBindingEnvironment` | `source proof-local given-condition binding base binding environment is invalid` |
| `InvalidAggregate` | `source proof-local given-condition binding aggregate is invalid` |
| `InvalidDeclaration { binding }` | `source proof-local given-condition binding <binding.index()> is invalid` |
| `InvalidBindingEnvironment` | `source proof-local given-condition binding binding environment is invalid` |
| `InvalidInstallation` | `source proof-local given-condition binding installation is invalid` |

### Exact dependency, transaction, and scope matrix

Dependency replay fixes theorem `19..133`, proof `68..132`, Given `76..113`,
segment `82..93`, name `82..83`, source ordinal 1, definition/contribution
`0/0`, spelling `y`, and the complete GCP lower fingerprint. The runner creates
one syntax-free `LocalTermBinding` at lexical scope `[0]`, declaration
`82..83`, visible-after ordinal 1, recovery `Normal`. The written bare
`set@90..93`, both condition references at `107..108` and `111..112`, their
equality/formula/condition structure, and label `G` remain authenticated only
inside the opaque lower bytes; GC publishes none of them.

The checker independently authenticates the exact GCP theorem identity rather
than treating the supplied symbol and lower bytes as mutually self-validating.
The symbol module is exactly the transaction `module_id`; its namespace is the
requested module path; and its primary spelling is exactly
`ProofLocalGivenConditionUseSmoke`. Let `escaped_module_path` be the module path
after replacing `\\` with `\\\\`, `:` with `\\c`, `|` with `\\p`, and `/` with
`\\s`, in that order. The required local ID is exactly:

```text
contribution=0:namespace={escaped_module_path}:owner=theorem#1:shell=theorem:kind=theorem:name=ProofLocalGivenConditionUseSmoke:notation=_:arity=_:definition=theorem:registration=_:policy=non-overloadable:slot=non-overloadable:_:theorem:_
```

The required FQN is exactly
`{module_id.package}::{module_id.path}::{required-local-id}`. Dependency
validation first requires equality of the supplied symbol's module, local ID,
and FQN with those independently constructed values. It also constructs the
expected complete GCP lower fingerprint with the independently constructed
required FQN, not with the supplied symbol's FQN. Consequently a coherent
mutation of both `theorem_symbol` and `lower_fingerprint` is a
`DependencyMismatch`; checker corruption tests must include that oracle.

The exact normal reserve base is `1/1/0`. The atomic binding transition is
`1/1/0 -> 2/2/0`: context 1 is `SourceStatement(68..132)`, parent 0, layer
`Proof`, lexical scope `[0]`, owned `[1]`, visible `[0,1]`, recovery `Normal`;
binding 1 is spelling `y`, kind `GivenWitness`, resolver-local identity
`([0], ordinal=1, declaration=82..83)`, owner context 1, visible-after 1,
`BindingTypeSite::Missing`, `Active`, uncaptured, diagnostic-free, recovery
`Normal`; row 0 records binding/context `1/1`, source/visible-after `1/1`, and
normal recovery. Context 0 and reserve binding 0 are byte-identical to the GCP
base. GC does not change `binding_env.rs` or add a binding kind.

The installed environment must reproduce this canonical lookup matrix:

| intent | context / lexical scope / ordinal | result |
|---|---|---|
| prior source position | `1 / [0] / 1` | forward binding 1 |
| declaration's own `such that` | `1 / [0] / 2` | local binding 1 |
| subsequent statement in block | `1 / [0] / 2` | local binding 1 |
| unshadowed descendant | `2 / [0,0] / 2` | local binding 1 |
| shadowed descendant | `3 / [0,1] / 3` | local binding 2 |
| after shadowed descendant | `1 / [0] / 3` | local binding 1 |
| parent block | `0 / [] / 2` | unresolved |
| sibling block | `4 / [1] / 2` | unresolved |

The two context-1 ordinal-2 rows are intentionally distinct test intents over
one lexical table: one represents the declaration condition and one the first
subsequent statement. GC creates no condition-use or later-use source row.
Test-only context 2 is generated `task269gc-unshadowed-child`, parent 1, block
layer, scope `[0,0]`, owned `[]`, visible `[0,1]`; context 3 is generated
`task269gc-shadow-child`, parent 1, block layer, scope `[0,1]`, owned `[2]`,
visible `[0,1,2]`; context 4 is generated `task269gc-sibling`, parent 0, block
layer, scope `[1]`, owned `[]`, visible `[0]`. All are normal. Test-only binding
2 is a normal active uncaptured diagnostic-free `GivenWitness` named `y`,
resolver-local at scope `[0,1]`, ordinal 2, deterministic synthetic declaration
range `114..115`, owner context 3, missing type. This row never enters the
handoff, runner, production table, or a source claim.

Validation precedence is transaction identity; lower/theorem/ranges; base;
dense aggregate; local/row fields; final environment/fingerprints/lookups;
Typed/final availability. Every failure is atomic. Stable debug is exactly:

```text
source-proof-local-given-condition-binding-debug-v1
module: <package>::<path>
lower-fingerprint: <quoted complete Task-269GCP debug bytes>
theorem symbol=<quoted-fqn> definition=0 contribution=0 range=19..133 proof=68..132
given range=76..113 segment=82..93 name=82..83 source_ordinal=1
base-binding-fingerprint: <quoted BindingEnv debug bytes>
binding#0 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal
final-binding-fingerprint: <quoted BindingEnv debug bytes>
```

Quoted fields use Rust `Debug`; order is fixed, with no blank line and one
final LF.

### Typed, Resolved, and runner ownership

`TypedAst` adds one boxed optional
`source_proof_local_given_condition_binding` after the existing Given-use term
slot, a read-only getter, and a consuming one-shot installer:

```rust
pub const fn source_proof_local_given_condition_binding(
    &self,
) -> Option<&SourceProofLocalGivenConditionBindingHandoff>;

pub fn with_source_proof_local_given_condition_binding(
    self,
    handoff: SourceProofLocalGivenConditionBindingHandoff,
) -> Result<Self, TypedAstError>;
```

`TypedAstError` adds `InvalidSourceProofLocalGivenConditionBinding`, rendered
`typed AST source proof-local given-condition binding handoff is inconsistent`.
Installation accepts only an otherwise-empty profile, rejects duplicate and
every old/new source owner in both orders, and publishes no Typed node,
context, type, fact, coercion, initial obligation, diagnostic, or
`TypedAstParts` field.

`ResolvedTypedAst` adds the corresponding boxed optional clone-preserved owner
and this exact read-only getter:

```rust
pub const fn source_proof_local_given_condition_binding(
    &self,
) -> Option<&SourceProofLocalGivenConditionBindingHandoff>;
```

Assembly revalidates the
complete handoff and the empty Typed profile before cloning it. Its error adds
`InvalidSourceProofLocalGivenConditionBinding`, rendered `resolved typed AST
source proof-local given-condition binding handoff is inconsistent`. It adds no
node hint, node role, final node, checked formula, semantic table, or
`ResolvedTypedAstInputs` path. Debug emits this slot after the old Given-use
term slot. All old debug bytes are unchanged when the slot is absent.

The private runner consumer is exactly
`source_proof_local_given_condition_binding_output` plus cfg-test
`_with_mutation`, returning an optional immutable output with read-only
`typed_ast()` and `resolved()` getters. Selection is delegated unchanged to
GCP; mismatch is `None`, selected lower/checker failure is `Some(Err(_))`, and
success contains the one-shot Typed/final owners. The mutation enum derives
`Debug, Clone, Copy, PartialEq, Eq`, is visible only in `crate::runner`, and is
exactly `None`, `WrongLowerFingerprint`, `EmptyBase`, `WrongTheoremRange`,
`WrongProofRange`, `WrongGivenRange`, `WrongSegmentRange`, `WrongNameRange`,
`WrongLocalSpelling`, `WrongLocalScope`, `WrongLocalRange`,
`WrongLocalVisibleAfter`, and `WrongSourceOrdinal`. Its validation errors are
the checker display strings plus exact private base messages `Task269GC exact
reserve base extraction failed` and `Task269GC exact reserve base failed:
{error}`. The runner does not duplicate GCP Surface/shell/resolver validation.

Its complete private type/function surface is:

```rust
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub(in crate::runner) struct SourceProofLocalGivenConditionBindingRouteOutput {
    typed_ast: TypedAst,
    resolved: ResolvedTypedAst,
}

impl SourceProofLocalGivenConditionBindingRouteOutput {
    pub(in crate::runner) const fn typed_ast(&self) -> &TypedAst;
    pub(in crate::runner) const fn resolved(&self) -> &ResolvedTypedAst;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(in crate::runner) enum SourceProofLocalGivenConditionBindingRouteMutation {
    None,
    WrongLowerFingerprint,
    EmptyBase,
    WrongTheoremRange,
    WrongProofRange,
    WrongGivenRange,
    WrongSegmentRange,
    WrongNameRange,
    WrongLocalSpelling,
    WrongLocalScope,
    WrongLocalRange,
    WrongLocalVisibleAfter,
    WrongSourceOrdinal,
}

#[allow(dead_code)]
pub(in crate::runner) fn source_proof_local_given_condition_binding_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalGivenConditionBindingRouteOutput, String>>;

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_condition_binding_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenConditionBindingRouteMutation,
) -> Option<Result<SourceProofLocalGivenConditionBindingRouteOutput, String>>;
```

The two output getters are the only access to the private fields. Attribute
rationale comments may describe dormancy but may not change these attributes
or visibility. The non-test function selects `None`; the test seam injects
only the listed mutation before checker production.

### Files, tests, deferrals, measurements, and exit

Implementation may change exactly seven existing Rust files:

1. `crates/mizar-checker/src/source_proof_local_declaration.rs`;
2. `crates/mizar-checker/src/typed_ast.rs`;
3. `crates/mizar-checker/src/resolved_typed_ast.rs`;
4. `crates/mizar-test/src/runner/type_elaboration/source_proof_local_declaration.rs`;
5. `crates/mizar-test/src/runner/type_elaboration.rs`;
6. `crates/mizar-test/src/runner.rs`;
7. `crates/mizar-test/src/runner/tests/type_elaboration/source_proof_local_declaration.rs`.

Exact checker tests are
`source_proof_local_given_condition_binding_builds_exact_scope_transaction`,
`source_proof_local_given_condition_binding_rejects_corruption_with_stable_precedence`,
`source_proof_local_given_condition_binding_typed_and_resolved_ownership_is_atomic`,
and
`source_proof_local_given_condition_binding_scope_matrix_is_lexical_and_semantically_empty`.
Exact runner tests are
`task269gc_exact_condition_binding_transaction_debug_and_lookup_are_stable`,
`task269gc_lower_base_and_checker_corruption_fail_closed`,
`task269gc_typed_and_resolved_owners_are_one_shot_and_semantically_empty`, and
`task269gc_near_miss_neighbor_and_active_routes_remain_isolated`. Corruption
tests cover every error variant, one seam per validation tier, combined
first-error precedence, coherent theorem-symbol plus lower-fingerprint
corruption, post-build mutation, duplicate/cross-family install in
both orders, rollback, near misses, G/GUP/GCP isolation, and positive legacy
active-route replay.

GC is binding-only. It does not publish written type, term/reference,
condition/formula/fact, label, existential/Skolem state, assumption guard,
goal/thesis transition, proof/discharge/acceptance, initial obligation,
diagnostic, Core/CFG/VC/ATP row, active dispatch, corpus result, or coverage
credit. Task 269GCT is the sole next by-value type consumer and may overlay only
`set@90..93`; Task 269GCU may transport the two declaration-condition term/
reference occurrences only after the exact GCT dependency. Descendant
occurrence transport, free-witness export enforcement, `set` capture, and Task
270 remain separate. Existing G/GUP/GUPT/GU APIs and bytes are immutable.

Docs-only baselines are checker/runner libraries `510/576`, parser/resolver/
syntax `226/148/59`, checker production `30/176258`, runner production
`37/76642`, cases/requirements `428/395`, pass/fail `235/193`, warnings/errors
`23/0`, stages `101/7/205/1`, and type coverage `259=247+12`. Implementation
projects `514/580`. Path hashes remain
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`;
baseline content hashes are
`5390a4ddc5516d3550fc2c6f5b010c629a79a66e098766a29d814b5dad40ee66` /
`adaeaad8bf2943e05e402f1bc565b5bb0f9a509fb74ffdcd9bbb05eab4d86b22`.
Raw/normalized test-list hashes are checker
`f6b6a3e76b9ae3207aae24434c5291986330755f18b8397b4bf62132fde2ed74` /
`51cdc82064023eaa77a9415de2c63f77940df92102086f7eeb71b48be771be34`
and runner
`b0f551ee94ca2e0ba0f294a80fc517e109832d3d1039659ad407fa4452f6bf86` /
`625307a34a88434ab27f03643a76311ac6f1d8c1f02bdb70ea8af77ced4fcede`.
All changed implementation measurements must be remeasured.

Protected parser fixture/sidecar, broad proof-local fixture/sidecar,
mixed-boundary fixture/sidecar, and trace SHA-256 values remain respectively
`bd9a2d473fa84012afb36dab8d0f9c11063dd5618df5a31791d57cba2c027234`,
`7361b50bc564d900e1852deaeaaf804544ad9c8ad0a3321a67c1e31bbaa80f17`,
`5fc4849a77eced7a93d65e0cae000c87b1730070c74aef116d6ca62be896ecd9`,
`8e2c73b1661a37c35887b08af01b42fc886199e7a3fb07db8c1412c69f62fa43`,
`d330fcb2a8196aaf2bf653cc604b7cd660f56dcef5331a55bd1b35b84e2732ef`,
`f318b97a75b119bf32b3d130f84df829e93a4a87007c4ada7e890fa30010f46c`,
and `55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.

The documentation prerequisite owns exactly the same 42 Markdown records as
GCP: 28 paired checker records, 12 paired mizar-test records, and two global
ledgers. It changes no Rust, Cargo, canonical spec, `.miz`, sidecar,
expectation, trace TOML, metadata, count, status, or production/test-list hash.
Exit requires synchronized EN/JA, repeated review-only specification audit
ending **NO FINDINGS**, all nine docs-only hard gates uncapped at `>=90/100`,
exact docs staging/commit, fresh parser/resolver/GCP/count/hash preflight, exact
seven-file/eight-test implementation, separate test-sufficiency,
implementation, and source/docs reviews ending **NO FINDINGS**, full
verification, all nine implementation gates and `>=90/100`, task-only commit,
clean post-commit inventory, protected stash identity, and automatic selection
of Task 269GCT.

Completion evidence: [central Task-269GC historical contract](../../task_contracts/en/269GC.md#completion-evidence).

## Task 269GCT Frozen Given-condition Type Consumer

Fresh inventory selects GCT only after GC implementation commit
`8181ae8fc8af0c7028254ad30147b417fbf84611`. The immutable dependency is the
complete `SourceProofLocalGivenConditionBindingHandoff`: exact 134-byte source,
54-node/root-53 Surface identity, theorem/proof/Given/segment/name ranges
`19..133`, `68..132`, `76..113`, `82..93`, `82..83`, independently validated
resolver provenance, complete GCP lower fingerprint, reserve base, final
`2/2/0` binding environment, one dense Given row, and all dependency/debug
fingerprints. GCT may not reconstruct or weaken any of these fields.

The runner additionally reads only the unchanged GCP `type_range` and
`type_head_range`, both exactly `90..93`, spelling `set`, `Bare`. It constructs
the two-row source-type input and three-node arena frozen in `source_type.md`,
then transfers the GC handoff by value to the checker-owned GCT producer.
`set@90..93` is the only newly published declaration site. The condition label,
the two `y` occurrences at `107..108` and `111..112`, equality/formula tree,
`thus` subtree, and proof close are strict subtree exclusions.

Missing distinct type producer/owners/tests are `source_drift` and `test_gap`;
the absent GCT contract and stale GC handoff wording are `design_drift`.
Changing canonical artifacts or expectations toward current source, adding
condition/fact/proof/obligation behavior, reusing the older G/GT or GUP/GUPT
composite, or moving binding/type construction into the runner is a
`boundary_violation`. Origin `0/15` is report-only
`repo_metadata_conflict`. No blocking `spec_gap` remains.

The exact four checker tests are
`task269gct_exact_condition_type_composition_is_stable`,
`task269gct_dependency_binding_input_and_arena_corruption_fail_closed`,
`task269gct_typed_and_resolved_ownership_is_atomic`, and
`task269gct_generic_neighbor_and_condition_use_routes_remain_isolated`. The
exact four runner tests use the corresponding names
`task269gct_exact_condition_type_route_is_stable`,
`task269gct_dependency_input_and_arena_corruption_fail_closed`,
`task269gct_typed_and_resolved_owners_are_one_shot_and_semantically_empty`, and
`task269gct_near_miss_neighbor_and_active_routes_remain_isolated`.

The corruption matrix is exhaustive at the frozen boundary. Dependency tests
cover wrong source/module, stale dependency fingerprint, every GC dependency
validation class, and coherent nested mutation. Binding tests cover wrong
type site, stale fingerprint, and a non-type field on both rows. Common input
tests cover application count/binding/ordinal/root; expression count/source/
module/site/range/spelling; head site/range/spelling; form/head/recovery; and a
non-empty argument. Arena tests cover wrong root and, for every node, kind,
resolved node, anchor, children, typing, recovery, and links. Post-build tests
cover source-type shape/fingerprint and the four-tier error precedence.
Ownership tests cover duplicate and every sibling in both orders, rollback,
clone replay, exact final role, all public semantic tables empty, and rejection
of node hints/expression metadata. Isolation covers wrong label/name/type,
missing final LF, old G/GT and GUP/GUPT/GU, proof-`let`, generic source type,
GCP/GC replay, and all existing active runner routes.

The checker files are exactly `source_type.rs`, `typed_ast.rs`, and
`resolved_typed_ast.rs`; the runner files are exactly
`runner/type_elaboration/source_proof_local_declaration.rs`,
`runner/type_elaboration.rs`, `runner.rs`, and the proof-local runner test leaf.
No lower selector, fixture, sidecar, expectation, trace, metadata, Cargo,
diagnostic, dispatch, CLI, corpus result, or active coverage changes. GCT ends
at written source-type transport; GCU retains the declaration-condition
occurrences, and all semantic, descendant, capture/export, and Task-270 work is
deferred.

Completion evidence: [central Task-269GCT historical contract](../../task_contracts/en/269GCT.md#completion-evidence).

## Task 269GCU Frozen Given-condition Occurrence Consumer

Fresh inventory selects GCU only after GCT implementation commit
`d6fb0ed28ced4d4706a1793b3aedd2a20eea0749`. The private runner must call the
unchanged GCT route and consume the installed
`SourceProofLocalGivenConditionTypeHandoff` by value. That dependency retains
the exact 134-byte source, 54-node/root-53 Surface fingerprint, authenticated
GCP theorem/resolver provenance, GC `2/2/0` environment, witness binding 1,
and the two-row/three-node GCT type transaction. GCU may not reconstruct,
weaken, or separately select any dependency field.

The unchanged frontend exposes exactly two condition `TermReference` leaves:
`y@107..108` and `y@111..112`. The runner constructs the exact two-term/two-
reference/zero-request input and six-node arena frozen in `source_term.md`.
It transfers the GCT handoff to
`SourceProofLocalGivenConditionUseTermProducer`, installs the returned owner on
an otherwise empty `TypedAst`, and assembles an otherwise empty
`ResolvedTypedAst`. Its private output has only `typed_ast` and `resolved`
fields. The non-test function is dormant and always selects mutation `None`.

The output ABI, field privacy, derives, and read-only accessors are frozen
exactly to the newest GCT pattern:

```rust
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: Task 269GCU is a private dormant runner consumer until activation.
pub(in crate::runner) struct SourceProofLocalGivenConditionUseTermRouteOutput {
    typed_ast: TypedAst,
    resolved: ResolvedTypedAst,
}

impl SourceProofLocalGivenConditionUseTermRouteOutput {
    pub(in crate::runner) const fn typed_ast(&self) -> &TypedAst;
    pub(in crate::runner) const fn resolved(&self) -> &ResolvedTypedAst;
}
```

No field is `pub(in crate::runner)` and no mutable or consuming accessor
exists.

The private mutation enum is exactly:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; other variants are private Task-269GCU corruption seams.
pub(in crate::runner) enum SourceProofLocalGivenConditionUseTermRouteMutation {
    None,
    WrongDependencyModule,
    WrongTermRange,
    WrongReferenceBinding,
    WrongArenaRoot,
    WrongArenaKind,
}
```

The private production and test-seam functions are respectively
`source_proof_local_given_condition_use_term_output` and
`source_proof_local_given_condition_use_term_output_with_mutation`; both return
`Option<Result<SourceProofLocalGivenConditionUseTermRouteOutput, String>>`.
Only the test seam accepts the mutation argument. Visibility, dead-code
rationale, and absence from active dispatch match the newer GCT private route.

Exact checker tests are
`task269gcu_exact_occurrence_references_and_fingerprints_are_stable`,
`task269gcu_dependency_term_input_and_arena_corruption_fail_closed`,
`task269gcu_typed_and_resolved_ownership_is_atomic`, and
`task269gcu_generic_and_neighbor_routes_remain_isolated`. Exact runner tests
are `task269gcu_exact_term_reference_composition_and_replay_are_stable`,
`task269gcu_dependency_input_and_arena_corruption_fail_closed`,
`task269gcu_typed_and_resolved_owners_are_one_shot_and_semantically_empty`, and
`task269gcu_near_miss_gct_and_active_routes_remain_isolated`.

Implementation changes exactly checker `source_term.rs`, `typed_ast.rs`, and
`resolved_typed_ast.rs`, plus runner proof-local production leaf, facade,
`runner.rs`, and proof-local test leaf. It adds no parser/resolver/lower code,
fixture, sidecar, expectation, trace, metadata, Cargo, diagnostic, dispatcher,
or active route. `G@104..105`, punctuation, equality, formula/proposition/
condition containers, Given statement semantics, `thus`, proof close, later/
descendant/sibling occurrences, label/fact/guard/goal/proof/discharge/
acceptance/initial-obligation/capture/export/IR behavior, and Task 270 are
strict exclusions.

Completion evidence: [central Task-269GCU historical contract](../../task_contracts/en/269GCU.md#completion-evidence).

## Task 269SDP Frozen Descendant/Set Lower Boundary

SDP authenticates only the exact 180-byte
`ProofLocalGivenDescendantCaptureSmoke` source and its parser/resolver
provenance. The immutable runner-private
`SourceProofLocalGivenDescendantSetLowerOutput` records theorem/proof/Given,
`now`, and the ordered `set z = y` / `set q = z` syntax sites plus source and
Surface fingerprints. It is built in `source_statement.rs` and re-exported
only through private runner/test facades.

The source is canonical-derived from Given descendant visibility and `set`
syntax, but Chapter 4 and Chapter 15 disagree on later `set` effects. The
resulting `spec_gap` does not affect syntax-only SDP; the lower output assigns
no binding or closure meaning. It adds no `BindingEnv`, context transition, Given or
LocalAbbreviation row, type, term/reference, captured `BinderIdentity`,
condition/fact, result/export, proof, obligation, semantic table, diagnostic,
or active route. The first consumer after SDP must freeze descendant
context/binding only; occurrence transport and `z`/`q` capture remain
separate later consumers. `CaptureSmoke` is a source spelling, not credit.
Combining those layers would be a `boundary_violation`; implementing closure
or capture before canonical reconciliation would also violate authority.

Exact source/Surface hashes are
`efa21af05a15f611815a4eb573577d0a368a3134693b225bdb56177f3637c2a8` /
`cbeae821434b0db13d77d7dac9984d8d6bf8012de9e7c680be12e8371e87ceaa`.
Surface is `68/root=67/tokens=[0,36)` (token indices 0--35), shells are
reserve node 39 and theorem node 64. The full ranges and resolver signature
are summarized in the crate plan; the literal debug grammar and type-for-type
ABI are frozen below in this owner document. The four existing runner files are
`crates/mizar-test/src/runner/type_elaboration/source_statement.rs`,
`crates/mizar-test/src/runner/type_elaboration.rs`,
`crates/mizar-test/src/runner.rs`, and
`crates/mizar-test/src/runner/tests/type_elaboration/source_proof_local_declaration.rs`.
The exact tests are
`task269sdp_exact_descendant_set_lower_projection_is_stable`,
`task269sdp_surface_lower_and_subtree_corruption_fail_closed`,
`task269sdp_resolver_shell_and_precedence_corruption_fail_closed`, and
`task269sdp_near_miss_and_active_routes_remain_isolated`; these four files and
four tests are the entire later implementation scope.

### Exact Task 269SDP private lower ABI

The 68-node Surface oracle is exact, not digest-only. In the following compact
table, `RW(s)`, `Id(s)`, and `RS(s)` mean the exact debug kinds
`Token(SurfaceToken { kind: ReservedWord, text: s })`,
`Token(SurfaceToken { kind: Identifier, text: s })`, and
`Token(SurfaceToken { kind: ReservedSymbol, text: s })`. Every token has no
children. Every row has the selected `SourceId` and `recovered=false`:

```text
 0 RW("reserve") 0..7       1 Id("x") 8..9
 2 RW("for") 10..13         3 RW("set") 14..17
 4 RS(";") 17..18           5 RW("theorem") 19..26
 6 Id("ProofLocalGivenDescendantCaptureSmoke") 27..64
 7 RS(":") 64..65           8 RW("thesis") 66..72
 9 RW("proof") 73..78      10 RW("given") 81..86
11 Id("y") 87..88          12 RW("being") 89..94
13 RW("set") 95..98        14 RS(";") 98..99
15 RW("now") 102..105      16 RW("set") 110..113
17 Id("z") 114..115        18 RS("=") 116..117
19 Id("y") 118..119        20 RS(";") 119..120
21 RW("set") 125..128      22 Id("q") 129..130
23 RS("=") 131..132        24 Id("z") 133..134
25 RS(";") 134..135        26 RW("thus") 140..144
27 RW("thesis") 145..151   28 RS(";") 151..152
29 RW("end") 155..158      30 RS(";") 158..159
31 RW("thus") 162..166     32 RW("thesis") 167..173
33 RS(";") 173..174        34 RW("end") 175..178
35 RS(";") 178..179
```

Structural rows use `index kind range children`:

```text
36 TypeHead 14..17 [3]
37 TypeExpression 14..17 [36]
38 ReserveSegment 8..17 [1,2,37]
39 ReserveItem 0..18 [0,38,4]
40 FormulaConstant(Thesis) 66..72 [8]
41 FormulaExpression 66..72 [40]
42 TypeHead 95..98 [13]
43 TypeExpression 95..98 [42]
44 QualifiedVariableSegment 87..98 [11,12,43]
45 GivenStatement 81..99 [10,44,14]
46 TermReference 118..119 [19]
47 TermExpression 118..119 [46]
48 Equating 114..119 [17,18,47]
49 SetStatement 110..120 [16,48,20]
50 TermReference 133..134 [24]
51 TermExpression 133..134 [50]
52 Equating 129..134 [22,23,51]
53 SetStatement 125..135 [21,52,25]
54 FormulaConstant(Thesis) 145..151 [27]
55 FormulaExpression 145..151 [54]
56 Proposition 145..151 [55]
57 ConclusionStatement 140..152 [26,56,28]
58 NowStatement 102..159 [15,49,53,57,29,30]
59 FormulaConstant(Thesis) 167..173 [32]
60 FormulaExpression 167..173 [59]
61 Proposition 167..173 [60]
62 ConclusionStatement 162..174 [31,61,33]
63 ProofBlock 73..178 [9,45,58,62,34]
64 TheoremItem 19..179 [5,6,7,41,63,35]
65 ItemList 0..179 [39,64]
66 CompilationUnit 0..179 [65]
67 Root 0..179 [0,1,...,35,66]
```

Root is 67, the expression root is absent, and token-node identity is the
half-open sequence `0..36`. This table serializes to the already frozen
Surface snapshot digest; any kind, source, range, recovery, children, root,
expression-root, token identity, or token-count difference fails Surface
validation.

The lower row is a syntax projection only. These two structs, field order,
field types, cardinality, visibility, attributes, and derives are exact; no
field is `pub`, and neither type is visible outside `crate::runner`:

```rust
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::runner) struct SourceProofLocalGivenDescendantSetLowerRow {
    statement_range: SourceRange,
    equating_range: SourceRange,
    name_range: SourceRange,
    name_spelling: String,
    rhs_range: SourceRange,
    rhs_spelling: String,
    source_ordinal: usize,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::runner) struct SourceProofLocalGivenDescendantSetLowerOutput {
    source_id: SourceId,
    module_id: ModuleId,
    source_fingerprint: String,
    surface_fingerprint: String,
    theorem_symbol: SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    given_range: SourceRange,
    given_segment_range: SourceRange,
    given_name_range: SourceRange,
    given_name_spelling: String,
    given_type_range: SourceRange,
    given_type_head_range: SourceRange,
    given_type_spelling: String,
    given_source_ordinal: usize,
    descendant_now_range: SourceRange,
    set_rows: [SourceProofLocalGivenDescendantSetLowerRow; 2],
    inner_conclusion_range: SourceRange,
    outer_conclusion_range: SourceRange,
}
```

The same-named read-only getters are exact in this order. Only spelling,
fingerprint, and `debug_text` getters are non-const; there is no mutable or
consuming accessor:

```rust
pub(in crate::runner) const fn statement_range(&self) -> SourceRange;
pub(in crate::runner) const fn equating_range(&self) -> SourceRange;
pub(in crate::runner) const fn name_range(&self) -> SourceRange;
pub(in crate::runner) fn name_spelling(&self) -> &str;
pub(in crate::runner) const fn rhs_range(&self) -> SourceRange;
pub(in crate::runner) fn rhs_spelling(&self) -> &str;
pub(in crate::runner) const fn source_ordinal(&self) -> usize;

pub(in crate::runner) const fn source_id(&self) -> SourceId;
pub(in crate::runner) const fn module_id(&self) -> &ModuleId;
pub(in crate::runner) fn source_fingerprint(&self) -> &str;
pub(in crate::runner) fn surface_fingerprint(&self) -> &str;
pub(in crate::runner) const fn theorem_symbol(&self) -> &SymbolId;
pub(in crate::runner) const fn theorem_definition(&self) -> DefinitionId;
pub(in crate::runner) const fn contribution(&self) -> SourceContributionId;
pub(in crate::runner) const fn theorem_range(&self) -> SourceRange;
pub(in crate::runner) const fn proof_range(&self) -> SourceRange;
pub(in crate::runner) const fn given_range(&self) -> SourceRange;
pub(in crate::runner) const fn given_segment_range(&self) -> SourceRange;
pub(in crate::runner) const fn given_name_range(&self) -> SourceRange;
pub(in crate::runner) fn given_name_spelling(&self) -> &str;
pub(in crate::runner) const fn given_type_range(&self) -> SourceRange;
pub(in crate::runner) const fn given_type_head_range(&self) -> SourceRange;
pub(in crate::runner) fn given_type_spelling(&self) -> &str;
pub(in crate::runner) const fn given_source_ordinal(&self) -> usize;
pub(in crate::runner) const fn descendant_now_range(&self) -> SourceRange;
pub(in crate::runner) const fn set_rows(
    &self,
) -> &[SourceProofLocalGivenDescendantSetLowerRow; 2];
pub(in crate::runner) const fn inner_conclusion_range(&self) -> SourceRange;
pub(in crate::runner) const fn outer_conclusion_range(&self) -> SourceRange;
pub(in crate::runner) fn debug_text(&self) -> String;
```

The exact row values are Given `81..99`, segment `87..98`, name
`87..88`/`"y"`, type/head `95..98`/`"set"`, source ordinal 1, descendant
Now `102..159`, Set row 0 `110..120`/Equating `114..119`/name
`114..115`/`"z"`/RHS `118..119`/`"y"`/ordinal 0, Set row 1
`125..135`/Equating `129..134`/name `129..130`/`"q"`/RHS
`133..134`/`"z"`/ordinal 1, inner conclusion `140..152`, and outer
conclusion `162..174`. The exact debug bytes, including the dynamic module and
theorem FQN placeholders and exactly one final LF, are:

```text
source-proof-local-given-descendant-set-lower-debug-v1
module: {package}::{module}
source-fingerprint: "efa21af05a15f611815a4eb573577d0a368a3134693b225bdb56177f3637c2a8"
surface-fingerprint: "cbeae821434b0db13d77d7dac9984d8d6bf8012de9e7c680be12e8371e87ceaa"
theorem symbol="{fqn}" definition=0 contribution=0 range=19..179 proof=73..178
given range=81..99 segment=87..98 source_ordinal=1
given-name range=87..88 spelling="y"
given-type range=95..98 head=95..98 spelling="set" form=bare
descendant-now range=102..159
set#0 statement=110..120 equating=114..119 source_ordinal=0
set#0 name range=114..115 spelling="z" rhs range=118..119 spelling="y"
set#1 statement=125..135 equating=129..134 source_ordinal=1
set#1 name range=129..130 spelling="q" rhs range=133..134 spelling="z"
conclusions inner=140..152 outer=162..174
```

All four mutation enums are `pub(in crate::runner)`, carry
`#[cfg_attr(not(test), allow(dead_code))]`, and derive
`Debug, Clone, Copy, PartialEq, Eq`. Their literal variant sets and order are:

```rust
enum SourceProofLocalGivenDescendantSetSurfaceMutation {
    None,
    ExpressionRoot,
    TokenNode(usize),
    TokenNodeCount,
    NodeKind(usize),
    NodeSourceId(usize),
    NodeRange(usize),
    NodeRecovery(usize),
    NodeChildren(usize),
    MissingRootIdentity,
    WrongRootIdentity,
}

enum SourceProofLocalGivenDescendantSetLowerMutation {
    None,
    SourceId,
    Module,
    SourceFingerprint,
    SurfaceFingerprint,
    TheoremSymbol,
    TheoremDefinition,
    Contribution,
    TheoremRange,
    ProofRange,
    GivenRange,
    GivenSegmentRange,
    GivenNameRange,
    GivenNameSpelling,
    GivenTypeRange,
    GivenTypeHeadRange,
    GivenTypeSpelling,
    GivenSourceOrdinal,
    DescendantNowRange,
    SetStatementRange(usize),
    SetEquatingRange(usize),
    SetNameRange(usize),
    SetNameSpelling(usize),
    SetRhsRange(usize),
    SetRhsSpelling(usize),
    SetSourceOrdinal(usize),
    InnerConclusionRange,
    OuterConclusionRange,
}

enum SourceProofLocalGivenDescendantSetShellMutation {
    None,
    Id(usize),
    Ordinal(usize),
    Kind(usize),
    Module(usize),
    Node(usize),
    Syntax(usize),
    Range(usize),
    Parent(usize),
    VisibilityState(usize),
    VisibilityMarker(usize),
    VisibilitySpelling(usize),
    Recovery(usize),
}

enum SourceProofLocalGivenDescendantSetResolverProfileMutation {
    None,
    ResolverModule,
    ImportIndex,
    ExportIndex,
    LabelIndex,
    OverloadIndex,
    RegistrationIndex,
    LexicalSummaryIndex,
    NamespaceGraph,
    DeclarationDependencyIndex,
    ModuleSummaryIndex,
    SymbolModule,
    SymbolNotation,
    SymbolContribution,
    SymbolRelations,
    SymbolOriginSource,
    SymbolOriginImport,
    DefinitionId,
    DefinitionParameters,
    DefinitionBinders,
    DefinitionNotation,
    DefinitionDoc,
    DefinitionContribution,
    DefinitionConflict,
    DefinitionDependencies,
    ContributionSymbolEffect,
    ContributionDefinitionEffect,
    ContributionLabelEffect,
    ContributionOverloadEffect,
    ContributionRegistrationEffect,
    ContributionLexicalEffect,
    ContributionNamespaceEffect,
    ContributionDeclarationDependencyEffect,
    ContributionImportEffect,
    ContributionExportEffect,
    ContributionDiagnosticEffect,
}
```

The dormant base and five test-only seams have these exact signatures. The
base alone carries `#[cfg_attr(not(test), allow(dead_code))]`; every mutation
seam carries `#[cfg(test)]`:

```rust
pub(in crate::runner) fn source_proof_local_given_descendant_set_lower_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalGivenDescendantSetLowerOutput, String>>;

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_descendant_set_lower_output_with_surface_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenDescendantSetSurfaceMutation,
) -> Option<Result<SourceProofLocalGivenDescendantSetLowerOutput, String>>;

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_descendant_set_lower_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenDescendantSetLowerMutation,
) -> Option<Result<SourceProofLocalGivenDescendantSetLowerOutput, String>>;

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_descendant_set_lower_output_with_shell_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenDescendantSetShellMutation,
) -> Option<Result<SourceProofLocalGivenDescendantSetLowerOutput, String>>;

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_descendant_set_lower_output_with_resolver_profile_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenDescendantSetResolverProfileMutation,
) -> Option<Result<SourceProofLocalGivenDescendantSetLowerOutput, String>>;

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_descendant_set_lower_output_with_resolver_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(SymbolEnv) -> SymbolEnv,
) -> Option<Result<SourceProofLocalGivenDescendantSetLowerOutput, String>>;
```

Source mismatch, including a missing final LF, returns `None`. A selected
validation failure returns `Some(Err(_))`; exact success returns the immutable
row. Validation precedence is exact Surface identity first; declaration-shell
count/export and shell ordinals 0 then 1; resolver module, absence of module
symbols `y`/`z`/`q`, and top-level indexes; theorem owner; theorem definition;
local contribution and its one-symbol/one-definition effects; lower row; then
exact debug bytes. The complete private error ABI is exactly these 16 strings:

```text
Task269SDP exact Surface identity changed after selection
Task269SDP requires exactly two declaration shells
Task269SDP resolver shells unexpectedly export a path
Task269SDP declaration shell {ordinal} mismatch
Task269SDP raw resolver module mismatch
Task269SDP local y/z/q already resolves as a module symbol
Task269SDP raw resolver inventory mismatch
Task269SDP requires one exact theorem owner
Task269SDP exact theorem owner provenance mismatch
Task269SDP requires one exact theorem definition
Task269SDP theorem contribution is missing
Task269SDP theorem symbol provenance mismatch
Task269SDP theorem definition provenance mismatch
Task269SDP theorem contribution provenance mismatch
Task269SDP private lower output mismatch
Task269SDP private lower debug grammar mismatch
```

The Surface oracle validates all 68 kind/source/range/recovery/children rows,
root 67, no expression root, and the exact half-open token-node sequence
`0..36`; its structural partition is the one frozen in the checker plan.
Tests enumerate both row indices for every indexed lower mutation, all nodes
and tokens, both shells, every resolver-profile variant, direct resolver-field
corruption, debug replay, and the stated validation precedence. None of these
private syntax or corruption records is a binding, reference, capture,
closure, fact, proof, or semantic payload.

Completion evidence: [central Task-269SDP historical contract](../../task_contracts/en/269SDP.md#completion-evidence).

## Task 269SDC Frozen Descendant Binding Consumer

Task 269SDC consumes the immutable Task-269SDP lower debug text and installs
only the outer Given binding plus the exact descendant context relationship.
The authority, ranges, classification, seven primary implementation files plus
one `cfg(test)`-only predecessor-ownership support file, eight tests,
zero-credit boundary, and exit gates are frozen in the crate plan. This owner
document freezes the complete public ABI and replay contract.

### Exact public ABI

The new structs derive `Debug, Clone, PartialEq, Eq`. Fields of the handoff are
private and appear in exactly this order:

```rust
pub struct SourceProofLocalGivenDescendantBindingHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub lower_fingerprint: String,
    pub theorem_symbol: SymbolId,
    pub theorem_definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub theorem_range: SourceRange,
    pub proof_range: SourceRange,
    pub given_range: SourceRange,
    pub segment_range: SourceRange,
    pub name_range: SourceRange,
    pub descendant_range: SourceRange,
    pub source_ordinal: usize,
    pub local: LocalTermBinding,
    pub descendant_scope: LocalTermScope,
    pub recovery: SourceProofLocalGivenBindingRecovery,
}

pub struct SourceProofLocalGivenDescendantBindingHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    lower_fingerprint: String,
    theorem_symbol: SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    given_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
    descendant_range: SourceRange,
    base_binding_env: BindingEnv,
    base_binding_fingerprint: String,
    binding_env: BindingEnv,
    final_binding_fingerprint: String,
    bindings: SourceProofLocalGivenBindingTable,
    descendant_context: BindingContextId,
}
```

The read-only getters follow field order. Copy values are `const fn`; borrowed
module/symbol/environment/table values are `const fn`; strings and
`debug_text()` are ordinary functions. There is no mutable or consuming
public accessor:

```rust
pub const fn source_id(&self) -> SourceId;
pub const fn module_id(&self) -> &ModuleId;
pub fn lower_fingerprint(&self) -> &str;
pub const fn theorem_symbol(&self) -> &SymbolId;
pub const fn theorem_definition(&self) -> DefinitionId;
pub const fn contribution(&self) -> SourceContributionId;
pub const fn theorem_range(&self) -> SourceRange;
pub const fn proof_range(&self) -> SourceRange;
pub const fn given_range(&self) -> SourceRange;
pub const fn segment_range(&self) -> SourceRange;
pub const fn name_range(&self) -> SourceRange;
pub const fn descendant_range(&self) -> SourceRange;
pub const fn base_binding_env(&self) -> &BindingEnv;
pub fn base_binding_fingerprint(&self) -> &str;
pub const fn binding_env(&self) -> &BindingEnv;
pub fn final_binding_fingerprint(&self) -> &str;
pub const fn bindings(&self) -> &SourceProofLocalGivenBindingTable;
pub const fn descendant_context(&self) -> BindingContextId;
pub fn debug_text(&self) -> String;
```

The producer derives `Debug, Clone, Copy, Default` and has only this build API:

```rust
pub struct SourceProofLocalGivenDescendantBindingProducer;

impl SourceProofLocalGivenDescendantBindingProducer {
    pub fn build(
        input: SourceProofLocalGivenDescendantBindingHandoffInput,
        base_binding_env: &BindingEnv,
    ) -> Result<
        SourceProofLocalGivenDescendantBindingHandoff,
        SourceProofLocalGivenDescendantBindingError,
    >;
}
```

The error derives `Debug, Clone, PartialEq, Eq`, is `#[non_exhaustive]`, and
implements `Display` plus `std::error::Error`. Its exact variants and display
strings are:

| variant | exact display |
|---|---|
| `InvalidTransaction` | `source proof-local given-descendant binding transaction is invalid` |
| `DependencyMismatch` | `source proof-local given-descendant binding dependency mismatch` |
| `InvalidBaseBindingEnvironment` | `source proof-local given-descendant binding base binding environment is invalid` |
| `InvalidAggregate` | `source proof-local given-descendant binding aggregate is invalid` |
| `InvalidDeclaration { binding }` | `source proof-local given-descendant binding <index> is invalid` |
| `InvalidDescendantContext` | `source proof-local given-descendant binding descendant context is invalid` |
| `InvalidBindingEnvironment` | `source proof-local given-descendant binding binding environment is invalid` |
| `InvalidInstallation` | `source proof-local given-descendant binding installation is invalid` |

The exact enum declaration uses
`InvalidDeclaration { binding: SourceProofLocalGivenBindingId }`. Validation
is not public; its exact signatures are:

```rust
pub(crate) fn validate_installation(
    &self,
    source_id: SourceId,
    module_id: &ModuleId,
) -> Result<(), SourceProofLocalGivenDescendantBindingError>;
pub(crate) fn validate_complete_installation(
    &self,
    source_id: SourceId,
    module_id: &ModuleId,
    installation_available: bool,
) -> Result<(), SourceProofLocalGivenDescendantBindingError>;
```

Complete installation appends only the one-shot owner check after full replay.

### Exact dependency and environment replay

Dependency validation independently reconstructs the theorem symbol local ID
using the existing escaped-module-path grammar and primary name
`ProofLocalGivenDescendantCaptureSmoke`; it does not trust a supplied symbol
or matching lower string. Definition/contribution are `0/0`; theorem, proof,
Given, segment, name, and descendant ranges are exactly `19..179`, `73..178`,
`81..99`, `87..98`, `87..88`, and `102..159`. `lower_fingerprint` is the
complete Task-269SDP lower debug text, including exact source/Surface SHA-256,
resolver theorem FQN, both Set rows, conclusions, and final LF.

The input declaration must be source ordinal 1, spelling `y`, scope `[0]`,
declaration `87..88`, visible-after 1, normal recovery, and descendant scope
`[0,0]`. The base is the exact reserve-only `1/1/0` environment. The final
environment is exactly `3 contexts / 2 bindings / 0 diagnostics` with the
profile in the crate plan. The reused public row is binding/context `1/1`,
source/visible-after `1/1`, normal recovery; `descendant_context` is exactly 2.
Binding 1 remains `BindingTypeSite::Missing`, active, uncaptured, and
diagnostic-free. Context 2 owns no binding and can see only `[0,1]`.

Validation precedence and exact error projection are frozen as follows:

1. source/module transaction identity -> `InvalidTransaction`;
2. lower fingerprint, theorem identity, and theorem/proof/Given/segment/name
   ranges -> `DependencyMismatch`;
3. base environment or base fingerprint ->
   `InvalidBaseBindingEnvironment`;
4. one-row aggregate shape -> `InvalidAggregate`;
5. local declaration or reused binding-row field ->
   `InvalidDeclaration { binding }`;
6. descendant range/scope or context-2 identity/parent/layer/scope/owned/
   visible/recovery field -> `InvalidDescendantContext`;
7. reconstructed final environment, final fingerprint, or any lookup result ->
   `InvalidBindingEnvironment`; and
8. Typed/final availability -> `InvalidInstallation`.

Every failure is atomic. Checker tests corrupt every alterable public input
field and injected handoff/environment field. The private runner seam covers
every *representably corruptible* route-input field and the combined-failure
precedence. `SourceProofLocalGivenBindingRecovery` currently has only the
single `Normal` value, so neither layer invents a synthetic recovery mutation;
both exact success paths explicitly check `Normal`.

The exact scope oracle is the crate-plan matrix. In particular, test-only
binding 2 is a normal active missing-type `GivenWitness y` in context 3 with
`ResolverLocal([0,1], ordinal=2, declaration=106..107)` and visible-after 2.
Context 3 is the shadow child `[0,1]`, context 4 is the same-proof sibling
child `[0,2]` that still inherits binding 1, and context 5 is the parent-0
proof sibling `[1]` where `y` is unresolved. Test-only contexts 3--5 and
binding 2 never enter the handoff or source provenance.

The exact debug grammar has no blank line and exactly one final LF:

```text
source-proof-local-given-descendant-binding-debug-v1
module: {package}::{module}
lower-fingerprint: {quoted complete Task-269SDP lower debug}
theorem symbol={quoted fqn} definition=0 contribution=0 range=19..179 proof=73..178
given range=81..99 segment=87..98 name=87..88 source_ordinal=1
base-binding-fingerprint: {quoted exact base BindingEnv debug}
binding#0 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal
descendant range=102..159 context=2 parent=1 scope=[0,0] recovery=normal
final-binding-fingerprint: {quoted exact final BindingEnv debug}
```

### Typed/Resolved and private runner boundary

`TypedAst` and `ResolvedTypedAst` each own one boxed optional handoff named
`source_proof_local_given_descendant_binding`. The exact Typed API is:

```rust
pub const fn source_proof_local_given_descendant_binding(
    &self,
) -> Option<&SourceProofLocalGivenDescendantBindingHandoff>;
pub fn with_source_proof_local_given_descendant_binding(
    self,
    handoff: SourceProofLocalGivenDescendantBindingHandoff,
) -> Result<Self, TypedAstError>;
```

The Typed error variant is
`InvalidSourceProofLocalGivenDescendantBinding` and renders exactly
`typed AST source proof-local given-descendant binding handoff is inconsistent`.
The Resolved borrowed getter is exactly:

```rust
pub const fn source_proof_local_given_descendant_binding(
    &self,
) -> Option<&SourceProofLocalGivenDescendantBindingHandoff>;
```

Its error variant has the same name and renders exactly
`resolved typed AST source proof-local given-descendant binding handoff is inconsistent`.
In both debug renderers, the optional SDC chunk appears immediately after the
current GCU `source_proof_local_given_condition_use_term` chunk and before
`source_statement_references` and node/table rendering. An absent SDC slot
preserves every current debug byte; a present slot appends the handoff debug
exactly once.

The Typed one-shot installer and final assembly reject a duplicate and all
ten predecessor proof-local owners in both orders: declaration, Let binding,
Let type, Given binding, Given type, Given-use type, Given-use term,
Given-condition binding, Given-condition type, and Given-condition-use term.
They also reject `resolved_root` and every other current source-owner slot;
every existing source-owner installer adds the reciprocal SDC availability
check, so no undocumented hybrid can be installed in either order. The ten
proof-local owners remain the explicit per-owner sentinel matrix, with generic
`source_term` as the representative non-proof source-owner rollback oracle.
Every replay happens before publication; failed reverse-order installation
preserves the pre-failure owner and debug bytes. Every semantic table remains
empty. Neither layer adds a node, type, term, occurrence, fact, obligation, or
diagnostic.

The private runner ABI is exactly:

```rust
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: Task 269SDC is a private dormant runner consumer until activation.
pub(in crate::runner) struct SourceProofLocalGivenDescendantBindingRouteOutput {
    typed_ast: TypedAst,
    resolved: ResolvedTypedAst,
}

impl SourceProofLocalGivenDescendantBindingRouteOutput {
    pub(in crate::runner) const fn typed_ast(&self) -> &TypedAst;
    pub(in crate::runner) const fn resolved(&self) -> &ResolvedTypedAst;
}
```

Neither field is visible and there is no mutable or consuming getter. The
private mutation enum is named
`SourceProofLocalGivenDescendantBindingRouteMutation`, derives
`Debug, Clone, Copy, PartialEq, Eq`, is `pub(in crate::runner)`, and carries
`#[allow(dead_code)]` with the rationale that production selects `None` while
the remaining variants are private Task-269SDC corruption seams. Its exact
declaration is:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; other variants are private Task-269SDC corruption seams.
pub(in crate::runner) enum SourceProofLocalGivenDescendantBindingRouteMutation {
    None,
    WrongLowerFingerprint,
    EmptyBase,
    WrongTheoremRange,
    WrongProofRange,
    WrongGivenRange,
    WrongSegmentRange,
    WrongNameRange,
    WrongDescendantRange,
    WrongLocalSpelling,
    WrongLocalScope,
    WrongLocalRange,
    WrongLocalVisibleAfter,
    WrongDescendantScope,
    WrongSourceOrdinal,
}
```

The exact production and cfg-test route signatures are:

```rust
#[allow(dead_code)] // Rationale: Task 269SDC is dormant until an active dispatcher is separately frozen.
pub(in crate::runner) fn source_proof_local_given_descendant_binding_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceProofLocalGivenDescendantBindingRouteOutput, String>>;

#[cfg(test)]
pub(in crate::runner) fn source_proof_local_given_descendant_binding_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceProofLocalGivenDescendantBindingRouteMutation,
) -> Option<Result<SourceProofLocalGivenDescendantBindingRouteOutput, String>>;
```

Only exact-source mismatch returns `None`. Selected lower/base/producer/
installation failures return `Some(Err(_))`. Extra runner errors are exactly
`Task269SDC exact reserve base extraction failed` and
`Task269SDC exact reserve base failed: {error}`; SDP lower errors propagate
unchanged.

The production function carries `#[allow(dead_code)]` with the dormant-route
rationale; the mutation-taking function is `#[cfg(test)]`. The production
function always selects `None`, and neither function is added to public or
active dispatch.

The route reads lower `source_id`, `module_id`, theorem symbol/definition/
contribution, theorem/proof/Given/segment/name ranges, Given name spelling and
source ordinal, descendant-`now` range, and the complete `debug_text()` used as
`lower_fingerprint`. It must not read the Given type getters, either Set row,
RHS, or conclusion getter except indirectly through that immutable complete
lower fingerprint, and it must not turn `y@118..119` into an occurrence. The
four checker and four runner test names, complete exclusions, projected counts,
and exit criteria in the crate plan are normative for this owner.

Completion evidence: [central Task-269SDC historical contract](../../task_contracts/en/269SDC.md#completion-evidence).
