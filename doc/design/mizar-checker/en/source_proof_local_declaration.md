# Source Proof-Local Declaration Transport

> Canonical language: English. Japanese companion:
> [../ja/source_proof_local_declaration.md](../ja/source_proof_local_declaration.md).

## Status and authority

This document freezes **Checker Tasks 269A--269CP**, the first three
dependency-minimal slices of queue Task 269. English is canonical. The matching Japanese document
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

## Task 269B active B3M1 binding contract

The existing API now accepts the exact frozen B3M1 profile. Construction and
installation authenticate all five fingerprints, the 56-node arena, both
witness rows, one name, and exact local provenance before producing only
declaration 0 and binding 1. Direct tests prove final `2/2/0`, context-1
`bindings=[1]`, `visible_bindings=[0,1]`, and absence of binding 2. Every node
kind plus representative cardinality/root/anchor/children/resolved/recovery/
typing/links corruption, and isolated B3N arena/statement/witness/primary
mixes, fail closed. No deferred semantic owner was activated.

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
snapshot hashes are selector fingerprints, not checker payloads. In
particular, the node numbers in the Surface table above are authenticated
selector-internal facts and are never laundered into `TypedSiteRef` values or
otherwise published as typed ownership.

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
