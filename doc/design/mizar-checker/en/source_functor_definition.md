# Source Functor-Definition Transport

> Canonical language: English. Japanese companion:
> [../ja/source_functor_definition.md](../ja/source_functor_definition.md).

## Task 260 Scope And Authority

Checker Task 260 owns one syntax-free, immutable source-to-checker intake for
ordinary `func` definitions and their initial correctness obligations. Its
canonical authority is Chapter 10 Sections 10.1--10.6, especially the
`equals`/`means`, `assume`, return-type, existence, and uniqueness rules;
Chapter 16 Sections 16.4 and 16.6.1 plus the corresponding 16.7.2 obligation
rows; the existing parser functor-definition pass/recovery fixtures; the
active predicate/functor definition gap fixture; and the committed public
Tasks 248--256 and 259 transports.

This task closes only `source_drift` for the missing checker producer and the
spec-derived `test_gap` for one exact consumer. It does not accept a
definition, verify a correctness proof, create a fact or axiom, build a FOL
goal, discharge an obligation, publish a VC, or lower Core/ControlFlow IR.

## Frozen Exact Source

The future active source is exactly, including the final LF:

```mizar
definition
  let x be set;
  let y be set;
  assume x = x;
  func Task260EqualsDef: task260_equals(x) -> set equals x;
  func Task260MeansDef: task260_means(y) -> set means x = y;
  existence by computation(steps: 1);
  uniqueness by computation(steps: 1);
end;
```

It is 262 bytes, nine lines, and has SHA-256
`9bbf50016c72faf8b86342a9a65f8d59bf7747b85b43b6c5bc3c624c7212416a`.
It contains one normal definition block, two separately written builtin-set
parameters, one source guard, one `equals` functor, one `means` functor, two
builtin-set return types, one primary-term definiens, one equality-formula
definiens, and explicit `existence` and `uniqueness` correctness clauses.
There is no import, reserve, predicate, property, theorem, proof block,
conditional definiens, `otherwise`, redefinition, notation declaration, or
recovery.

## Frozen Surface Profile

The frontend produces zero diagnostics and exactly 108 dense Surface rows.
The root is node 107, range `0..261`, normal. The structural rows needed by
the private runner are:

| Node | Surface kind | Range | Task-260 role |
| ---: | --- | --- | --- |
| 62/63 | `TypeHead` / `TypeExpression` | `22..25` | parameter `x` written type |
| 65 | `DefinitionParameter` | `13..26` | first context parameter |
| 66/67 | `TypeHead` / `TypeExpression` | `38..41` | parameter `y` written type |
| 69 | `DefinitionParameter` | `29..42` | second context parameter |
| 70--75 | primary terms plus equality/formula | `52..57` | exact guard `x = x` |
| 77 | `AssumptionStatement` | `45..58` | context-level guard owner |
| 78 | `FunctorPattern` | `84..101` | `task260_equals ( x )` |
| 79/80 | `TypeHead` / `TypeExpression` | `105..108` | equals return type |
| 81--83 | primary/term/`TermDefiniens` | `116..117` | equals body `x` |
| 84 | `FunctorDefinition` | `61..118` | equals definition |
| 85 | `FunctorPattern` | `143..159` | `task260_means ( y )` |
| 86/87 | `TypeHead` / `TypeExpression` | `163..166` | means return type |
| 88--94 | primary terms plus equality/formula definiens | `173..178` | means body `x = y` |
| 95 | `FunctorDefinition` | `121..179` | means definition |
| 97/98/99 | computation/justification/correctness | `195..216` / `192..216` / `182..217` | existence clause and excluded proof subtree |
| 101/102/103 | computation/justification/correctness | `234..255` / `231..255` / `220..256` | uniqueness clause and excluded proof subtree |
| 104 | `DefinitionBlockItem` | `0..261` | common source/context owner |

The parameter declaration ranges are `17..18` and `33..34`. The definition
label ranges are `66..82` and `126..141`; patterns are `84..101` and
`143..159`; the resolver-owned definition ranges are `61..118` and
`121..179`. Nodes 65, 69, 77, 84, 95, 99, and 103 are normal direct siblings
of node 104 in source order.

The checker never receives these raw node numbers or syntax kinds. The
private `mizar-test` selector authenticates every loaded byte, final LF, all
108 row kinds/ranges/recovery/ordered children, root identity, direct sibling
order, and subtree partition before constructing syntax-free inputs.

## Frozen Raw Resolver Provenance

The resolver result is exactly three shells, two signature projections, zero
symbol diagnostics, two functor symbols, two functor definitions, and one
local-source contribution:

- shell 0 is the `DefinitionBlock` at node/range `104/0..261`;
- shell 1 is `FunctorDefinition` node/range `84/61..118`, parent shell 0;
- shell 2 is `FunctorDefinition` node/range `95/121..179`, parent shell 0;
- definition 0 is `SymbolKind::Functor` / `DefinitionKind::Functor`, notation
  `task260_equals ( x )`, structural path `[4,0,9,0]`;
- definition 1 is `SymbolKind::Functor` / `DefinitionKind::Functor`, notation
  `task260_means ( y )`, structural path `[4,0,9,1]`; and
- both are normal, local, exported, overloadable, conflict-free, and belong
  to the single local contribution.

Resolver `parameters`, `binders`, and syntactic arity are empty. Task 260
must not infer definition parameters, guard ownership, `equals`/`means`
style, return-type identity, definiens ownership, or correctness association
from those empty fields or from opaque signature payload text. Those values
come only from the authenticated Surface structure and lower handoffs. The
resolver does not prove overload uniqueness or definition correctness.

## Frozen Lower Bundle And Ownership

The exact source consumes the committed lower transports in this order:

| Owner | Exact active profile | Ownership |
| --- | --- | --- |
| Task 248 | Profile B `1/2/2/2/2/2/0` | definition-block context and two ordered bindings |
| Task 249 | `4/4/0` | two parameter types and two return types |
| Task 252 | `5/5/0` | guard operands, equals body, and means operands |
| Task 253 | absent | no application-root definiens |
| Task 254 | absent | no structure-root definiens |
| Task 255 | absent | no set/choice/qua-root definiens |
| Task 256 | `2/0/0/0/0/0/0/4/4` | guard equality and means equality |
| Task 259 | absent and independent | no predicate-definition input or fingerprint |

Task-252 source order is guard `x`, guard `x`, equals body `x`, means body
`x`, means body `y`. Task-256 formula 0 is the guard, and formula 1 is the
means definiens. Pattern-locus identifiers, definition labels, return-type
tokens, correctness keywords, and computation-justification descendants are
excluded from direct Task-252/256 discovery.

The public Task-260 definiens target supports exactly the already frozen
lower root families: Task-252 primary term, Task-253 application, Task-254
structure, Task-255 set term, or Task-256 atomic formula. The active source
uses `Primary(2)` for `equals` and `AtomicFormula(1)` for `means`. Optional
application/structure/set fingerprints remain absent for this source. No
composite formula, conditioned/case/otherwise definiens, or nested unsupported
root is admitted in Task 260.

## Exact Public Syntax-Free Input

Implementation adds `source_functor_definition.rs` with these five dense ID
families. Every ID is `Copy + Eq + Ord + Hash`, exposes only `new` and
`index`, and is allocated by vector order:

```rust
pub struct SourceFunctorDefinitionId(usize);
pub struct SourceFunctorParameterId(usize);
pub struct SourceFunctorGuardId(usize);
pub struct SourceFunctorDefiniensId(usize);
pub struct SourceFunctorCorrectnessId(usize);
```

The exact public input types are:

```rust
pub struct SourceFunctorDefinitionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub definitions: Vec<SourceFunctorDefinitionInput>,
    pub parameters: Vec<SourceFunctorParameterInput>,
    pub guards: Vec<SourceFunctorGuardInput>,
    pub definientia: Vec<SourceFunctorDefiniensInput>,
    pub correctness: Vec<SourceFunctorCorrectnessInput>,
}

pub struct SourceFunctorDefinitionInput {
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourceFunctorDefinitionRecovery,
    pub spelling: String,
    pub style: SourceFunctorDefinitionStyle,
    pub return_type: SourceTypeApplicationId,
    pub definiens: SourceFunctorDefiniensId,
}

pub struct SourceFunctorParameterInput {
    pub ordinal: usize,
    pub binding: BindingId,
    pub written_type: SourceTypeApplicationId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub declaration_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceFunctorDefinitionRecovery,
    pub spelling: String,
}

pub struct SourceFunctorGuardInput {
    pub ordinal: usize,
    pub formula: SourceAtomicFormulaId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceFunctorDefinitionRecovery,
    pub spelling: String,
}

pub struct SourceFunctorDefiniensInput {
    pub owner: SourceFunctorDefinitionId,
    pub ordinal: usize,
    pub target: SourceFunctorDefiniensTarget,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceFunctorDefinitionRecovery,
    pub spelling: String,
}

pub struct SourceFunctorCorrectnessInput {
    pub owner: SourceFunctorDefinitionId,
    pub ordinal: usize,
    pub kind: SourceFunctorCorrectnessKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub justification: SourceAnchor,
    pub recovery: SourceFunctorDefinitionRecovery,
    pub spelling: String,
}

#[non_exhaustive]
pub enum SourceFunctorDefinitionStyle { Equals, Means }

#[non_exhaustive]
pub enum SourceFunctorDefiniensTarget {
    Primary(SourcePrimaryTermId),
    Application(SourceFunctorApplicationId),
    Structure(SourceStructureTermId),
    SetTerm(SourceSetTermId),
    AtomicFormula(SourceAtomicFormulaId),
}

#[non_exhaustive]
pub enum SourceFunctorCorrectnessKind { Existence, Uniqueness }

#[non_exhaustive]
pub enum SourceFunctorDefinitionRecovery { Normal, Degraded }
```

Parameters and the guard belong to the shared Task-248 definition context;
they intentionally have no definition owner. Raw Surface/resolver types and
correctness proof nodes never cross this seam.

All input structs derive `Debug + Clone + PartialEq + Eq`. Style,
correctness-kind, recovery, and target enums derive
`Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash`. Immutable
rows, tables, the handoff, projection, and error derive
`Debug + Clone + PartialEq + Eq`; the producer is a unit struct.

## Exact Immutable Output And Producer API

The immutable row type names and stored fields, in API order, are:

| Row | Stored fields |
| --- | --- |
| `SourceFunctorDefinition` | `id`, `symbol`, `definition`, `contribution`, `site`, `source_range`, `source_ordinal`, `context`, `recovery`, `spelling`, `style`, `return_type`, `definiens`, derived `origin` |
| `SourceFunctorParameter` | `id`, `ordinal`, `binding`, `written_type`, `site`, `source_range`, `declaration_range`, `context`, `recovery`, `spelling` |
| `SourceFunctorGuard` | `id`, `ordinal`, `formula`, `site`, `source_range`, `context`, `recovery`, `spelling` |
| `SourceFunctorDefiniens` | `id`, `owner`, `ordinal`, `target`, `site`, `source_range`, `context`, `recovery`, `spelling` |
| `SourceFunctorCorrectness` | `id`, `owner`, `ordinal`, `kind`, `site`, `source_range`, `justification`, `recovery`, `spelling`, derived `obligation` |

Every stored field has one same-named read-only getter. Copy IDs/enums/ranges/
ordinals/contexts, including the definiens target, return by value; symbol,
site, origin, and justification return by shared reference; `spelling()`
returns `&str`. There
are no public row constructors, setters, mutable getters, or replacement APIs.

The exact table and handoff surface is:

```rust
pub struct SourceFunctorDefinitionTable { /* private rows */ }
pub struct SourceFunctorParameterTable { /* private rows */ }
pub struct SourceFunctorGuardTable { /* private rows */ }
pub struct SourceFunctorDefiniensTable { /* private rows */ }
pub struct SourceFunctorCorrectnessTable { /* private rows */ }

pub struct SourceFunctorDefinitionHandoff { /* private fields */ }

impl SourceFunctorDefinitionHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn source_context_fingerprint(&self) -> &str;
    pub fn source_type_fingerprint(&self) -> &str;
    pub fn source_term_fingerprint(&self) -> &str;
    pub fn application_fingerprint(&self) -> Option<&str>;
    pub fn structure_fingerprint(&self) -> Option<&str>;
    pub fn set_term_fingerprint(&self) -> Option<&str>;
    pub fn atomic_formula_fingerprint(&self) -> Option<&str>;
    pub const fn definitions(&self) -> &SourceFunctorDefinitionTable;
    pub const fn parameters(&self) -> &SourceFunctorParameterTable;
    pub const fn guards(&self) -> &SourceFunctorGuardTable;
    pub const fn definientia(&self) -> &SourceFunctorDefiniensTable;
    pub const fn correctness(&self) -> &SourceFunctorCorrectnessTable;
    pub fn debug_text(&self) -> String;
}
```

Each table exposes only `get(id) -> Option<&Row>`, source-ordered
`iter() -> impl Iterator<Item = (Id, &Row)>`, `const len() -> usize`, and
`const is_empty() -> bool`. Required fingerprints are the complete
Task-248/249/252 lower `debug_text()` strings. Optional fingerprints are the
complete Task-253/254/255/256 strings and are `Some` exactly when a target or
guard uses that family. The active profile has only the atomic-formula
fingerprint present. The caller cannot supply any fingerprint.

The build/projection/error ABI is exact:

```rust
pub struct SourceFunctorDefinitionProjection {
    base_initial_obligations: InitialObligationTable,
    handoff: SourceFunctorDefinitionHandoff,
    initial_obligations: InitialObligationTable,
}

impl SourceFunctorDefinitionProjection {
    pub const fn base_initial_obligations(&self) -> &InitialObligationTable;
    pub const fn handoff(&self) -> &SourceFunctorDefinitionHandoff;
    pub const fn initial_obligations(&self) -> &InitialObligationTable;
    pub fn into_parts(
        self,
    ) -> (
        InitialObligationTable,
        SourceFunctorDefinitionHandoff,
        InitialObligationTable,
    );
}

#[non_exhaustive]
pub enum SourceFunctorDefinitionError {
    SourceIdentityMismatch,
    DependencyMismatch,
    InvalidResolverDefinition { index: usize },
    InvalidDefinition { index: usize },
    InvalidParameter { index: usize },
    InvalidGuard { index: usize },
    InvalidDefiniens { index: usize },
    InvalidCorrectness { index: usize },
    InvalidObligation,
    InvalidArenaOwnership,
    UnsupportedTaskShape,
}

pub struct SourceFunctorDefinitionProducer;

impl SourceFunctorDefinitionProducer {
    pub fn build(
        input: SourceFunctorDefinitionHandoffInput,
        env: &SymbolEnv,
        source_context: &SourceBindingContextHandoff,
        source_type: &SourceTypeApplicationHandoff,
        source_term: &SourcePrimaryTermHandoff,
        applications: Option<&SourceFunctorApplicationHandoff>,
        structures: Option<&SourceStructureHandoff>,
        set_terms: Option<&SourceSetTermHandoff>,
        atomic_formulas: Option<&SourceAtomicFormulaHandoff>,
        base_initial_obligations: &InitialObligationTable,
        arena: &TypedArena,
    ) -> Result<
        SourceFunctorDefinitionProjection,
        SourceFunctorDefinitionError,
    >;
}
```

The error implements `Display` and `Error`, has no `Default` or blanket
conversion, and every public enum above is `#[non_exhaustive]`.

## Public Enum Policy

| Public enum | Compatibility policy |
| --- | --- |
| `SourceFunctorDefinitionStyle` | `#[non_exhaustive]`; callers must tolerate later explicitly frozen definition styles. |
| `SourceFunctorDefiniensTarget` | `#[non_exhaustive]`; callers must tolerate later explicitly frozen lower-root targets. |
| `SourceFunctorCorrectnessKind` | `#[non_exhaustive]`; callers must tolerate later explicitly frozen correctness kinds. |
| `SourceFunctorDefinitionRecovery` | `#[non_exhaustive]`; callers must tolerate later recovery classes. |
| `SourceFunctorDefinitionError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Frozen Debug Grammar And Active Row Oracle

`SourceFunctorDefinitionHandoff::debug_text()` is the dependency fingerprint.
It emits the following families in exactly this order, with a final LF and no
blank line. `Rust-debug` means standard escaped `{:?}` output; an absent
optional fingerprint is exactly `none`, while a present one is its Rust-debug
string:

```text
source-functor-definition-debug-v1
module: <ModuleId.path>
source-context-fingerprint: <Rust-debug String>
source-type-fingerprint: <Rust-debug String>
source-term-fingerprint: <Rust-debug String>
application-fingerprint: <none|Rust-debug String>
structure-fingerprint: <none|Rust-debug String>
set-term-fingerprint: <none|Rust-debug String>
atomic-formula-fingerprint: <none|Rust-debug String>
definition#<id> symbol=<Rust-debug FQN string> definition=<id> contribution=<id> ordinal=<n> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> origin_range=<start>..<end> origin_path=<Rust-debug [u32]> spelling=<Rust-debug String> style=<equals|means> return_type=<id> definiens=<id>
parameter#<id> ordinal=<n> binding=<id> written_type=<id> range=<start>..<end> declaration_range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
guard#<id> ordinal=<n> formula=<id> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
definiens#<id> owner=<id> ordinal=<n> target=<primary|application|structure|set-term|atomic-formula>:<id> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
correctness#<id> owner=<id> ordinal=<n> kind=<existence|uniqueness> range=<start>..<end> site=node#<id> justification=range:<start>..<end> recovery=<normal|degraded> spelling=<Rust-debug String> obligation=<id>
```

Only node sites, range anchors, normal local origins, and `Normal` rows are
admitted. Definition origins are exact resolver ranges/paths
`61..118/[4,0,9,0]` and `121..179/[4,0,9,1]`. The active immutable oracle is:

- definitions 0/1 use sites 84/95, ranges `61..118`/`121..179`, source
  ordinals 0/1, context 1, spellings
  `func Task260EqualsDef: task260_equals(x) -> set equals x;` and
  `func Task260MeansDef: task260_means(y) -> set means x = y;`, styles
  Equals/Means, return-type applications 2/3, and definientia 0/1;
- shared parameters 0/1 use bindings 0/1, type applications 0/1, sites 65/69,
  ranges `13..26`/`29..42`, declaration ranges `17..18`/`33..34`, context 1,
  ordinals 0/1, and spellings `let x be set;` / `let y be set;`;
- guard 0 uses atomic formula 0, site 77, range `45..58`, context 1, ordinal 0,
  and spelling `assume x = x;`;
- definiens 0 uses owner 0, ordinal 0, `Primary(SourcePrimaryTermId(2))`, site
  83, range `116..117`, context 1, spelling `x`; definiens 1 uses owner 1,
  ordinal 1, `AtomicFormula(SourceAtomicFormulaId(1))`, site 94, range
  `173..178`, context 1, spelling `x = y`;
- correctness 0/1 use owner 1, ordinals 0/1, kinds Existence/Uniqueness, sites
  99/103, ranges `182..217`/`220..256`, justification anchors
  `192..216`/`231..255`, spellings
  `existence by computation(steps: 1);` and
  `uniqueness by computation(steps: 1);`, and obligations at baseline length
  plus 0/1.

The five cardinalities are exactly `2/2/1/2/2`. Definitions, definientia,
and correctness are dense source order. Context rows are shared, not
duplicated. No Task-259 input, fingerprint, table, or getter exists.

## Initial-Obligation Contract

`InitialObligationKind` gains exactly `FunctorExistence` and
`FunctorUniqueness`. An `Equals` definition appends no existence or uniqueness
obligation. A `Means` definition with the exact two explicit correctness
clauses appends exactly two rows, in clause order:

- `FunctorExistence`, owner/source range from correctness row 0, status
  `Pending`;
- `FunctorUniqueness`, owner/source range from correctness row 1, status
  `Pending`.

Both have empty `TypeFactId` assumptions and deterministic opaque goal and
provenance identities. For a baseline length `b`, the exact rows are:

| Field | Existence row | Uniqueness row |
| --- | --- | --- |
| id | `InitialObligationId(b)` | `InitialObligationId(b + 1)` |
| kind | `FunctorExistence` | `FunctorUniqueness` |
| owner | correctness 0 site 99 | correctness 1 site 103 |
| range | `182..217` | `220..256` |
| assumptions | empty | empty |
| goal | `source.definition.functor.correctness:definition=1:existence` | `source.definition.functor.correctness:definition=1:uniqueness` |
| provenance | `source.definition.functor:definition=1:correctness=0` | `source.definition.functor:definition=1:correctness=1` |
| status | `Pending` | `Pending` |

Empty assumptions mean only that Task 260 does not
invent the Chapter-10/16 typed-parameter/guard/return-type/FOL composition.
They do not claim that the semantic obligations are unguarded. The guard,
parameters, return type, and means formula remain available as separate
authenticated source transport for later VC construction.

The computation justification subtrees are preserved in the Surface AST for
future proof/justification ownership. Task 260 does not read their options,
mark either obligation discharged, validate a proof, or accept/register the
functor. No fact, equality axiom, uniqueness fact, symbol activation, theorem,
proof evidence, Core IR, CFG, or VC row is produced.

The input baseline may contain unrelated existing obligation kinds, but it
must contain no `FunctorExistence`, `FunctorUniqueness`, or
`PredicatePropertyCorrectness` row. Build and typed installation reject any
such pre-existing row. With the Task-260 handoff present, the completed table
contains exactly the two linked Task-260 rows above and no other row of either
functor kind. With the handoff absent, final assembly rejects every orphan
`FunctorExistence` or `FunctorUniqueness` row.

## Validation, Typed Ownership, And Task 259 Separation

`SourceFunctorDefinitionProducer::build` validates complete source/module
identity, lower dependency fingerprints, arena anchors, dense ordinals,
contexts, resolver provenance, style/target consistency, return types,
source ranges, grouping, exact correctness cardinality and order, and baseline
obligation preservation before publishing. It rejects partial `means`
correctness, correctness on `equals`, wrong target family, stale fingerprint,
copied site, reordered row, cross-source/context data, overlapping ownership,
and any extra or missing row.

The producer returns the exact projection specified above. Failure is atomic.
The public installation surface is:

```rust
impl TypedAst {
    pub fn with_source_functor_definition(
        self,
        projection: SourceFunctorDefinitionProjection,
    ) -> Result<Self, TypedAstError>;

    pub const fn source_functor_definition(
        &self,
    ) -> Option<&SourceFunctorDefinitionHandoff>;
}

TypedAstError::InvalidSourceFunctorDefinition

impl ResolvedTypedAst {
    pub const fn source_functor_definition(
        &self,
    ) -> Option<&SourceFunctorDefinitionHandoff>;
}

ResolvedTypedAstError::InvalidSourceFunctorDefinition
```

`TypedAstParts` and `ResolvedTypedAstInputs` gain no Task-260 field. The
one-shot typed method is the only installation path. It requires the current
obligation table to equal the retained baseline, installs the handoff and
baseline-plus-two table together, and rejects prior Task-260 occupancy.
Final assembly revalidates the exact handoff, lower fingerprints, and the two
final appended obligation rows, then clone-preserves them. Typed/final debug
include the complete handoff text exactly once.

Task 259 and Task 260 are mutually isolated transactions in this task, not a
coexistence contract. Task-260 build and installation reject a baseline that
contains `PredicatePropertyCorrectness` or a `TypedAst` that already contains
a Task-259 handoff; final assembly rejects both handoffs appearing together.
Task 259 code and its final-row invariant remain unchanged. There is no
predicate-to-functor or functor-to-predicate install-order promise. A future
mixed-definition owner would need its own canonical authority, frozen
obligation ordering, compatibility edit, tests, and commit.

The existing mixed predicate-plus-functor fail fixture, sidecar, expectation,
and trace rows remain byte-unchanged in the documentation prerequisite. The
future implementation may use it only for isolation/authentication tests; it
does not rebaseline that existing expectation or claim mixed acceptance.

## Dedicated Consumer And Trace Intent

Implementation adds exactly one new active pass pair:

- `tests/miz/pass/types/pass_type_elaboration_functor_definition_payload_001.miz`;
- `tests/miz/pass/types/pass_type_elaboration_functor_definition_payload_001.expect.toml`.

The sidecar is `pass` / `type_elaboration` / `type_check`, has empty public
diagnostics and payloads, and cites only future requirement
`spec.en.checker.type_elaboration.source_functor_definition_payload`. Passing
credits successful source transport and pending-obligation intake only, not
definition correctness or acceptance. One covered trace row backlinks only
this sidecar.

The private runner route authenticates the exact loaded source, normal AST,
resolver profile, Task-248/249/252/256 bundle, and `2/2/1/2/2` Task-260
result. It runs before the generic definition extraction gap but matches no
other active case. Existing parser fixtures, mixed predicate/functor gap,
Task-253/255 functor-definiens gap cases, their sidecars, and their trace
intent remain unchanged.

## Frozen Tests And Write Scope

Five checker tests are frozen:

1. `task_260_exact_functor_definition_payload_and_pending_obligations`;
2. `task_260_independent_row_and_field_corruption_fails_closed`;
3. `task_260_dependency_and_obligation_corruption_fails_closed`;
4. `task_260_typed_installation_is_transactional`;
5. `task_260_final_clone_debug_determinism_and_predicate_isolation`.

Their assertion ownership is exact:

1. the exact-payload test covers all five `2/2/1/2/2` rows, every public
   getter, all active fields/ranges/sites/spellings/origins, style/target and
   correctness association, required fingerprints, optional fingerprint
   `None/None/None/Some`, the complete debug grammar, both exact serializer
   strings, and every field of the two obligations at IDs 0/1;
2. the row/field-corruption test independently mutates every input row field,
   dense ID/cardinality/order/grouping, shared parameter/guard context,
   Equals/Means and target compatibility, return type, recovery, spelling,
   resolver identity, correctness kind/owner/site/range/anchor, and rejects
   extra/missing/partial correctness without sorting or repair;
3. the dependency/obligation test corrupts each required and optional lower
   fingerprint/ID/arena owner plus every obligation ID/kind/owner/range/
   assumption/goal/provenance/status/link. It rejects pre-existing functor or
   predicate kinds, orphan functor kinds without a handoff, and extra functor
   kinds with a handoff;
4. the transactional test uses a valid nonempty unrelated baseline and proves
   byte-for-byte row/ID preservation, new IDs `b`/`b+1`, every projection
   getter and `into_parts`, one-shot install, exact-current-baseline success,
   and atomic rollback for stale/reordered/colliding baselines, replacement,
   prior occupancy, and half publication; and
5. the final test covers no-handoff legacy debug, immutable clone/replay,
   exact one-copy debug output, final revalidation, absent/orphan/extra rows,
   Task-259 handoff/predicate-baseline mutual exclusion, unchanged Task-259
   behavior, and absence of fact/proof/acceptance/VC output.

Four runner tests are frozen:

1. `task260_real_source_surface_resolver_and_lower_bundle_is_exact`;
2. `task260_source_ast_resolver_and_lower_mutations_fail_at_the_owner`;
3. `task260_expectation_selection_and_predicate_route_stay_isolated`;
4. `task260_route_publishes_no_proof_fact_acceptance_or_vc`.

Their assertion ownership is exact:

1. the real-source test authenticates all 262 bytes/final LF/hash, all 108
   Surface row kinds/ranges/recovery/ordered children, root/direct-sibling/
   subtree partition, all three resolver shells/two projections/symbols/
   definitions/local contribution, the exact Task-248/249/252/256 bundle,
   and every final Task-260 row/obligation/debug field;
2. the mutation test independently changes loaded bytes, final LF, root,
   row kind/range/recovery/child/order/relocation, excluded pattern/label/
   return-token/correctness/computation descendants, resolver environment/
   projection/symbol/definition/contribution, and every lower association so
   each corruption stops at its owning stage without a Task-260 result;
3. the selection test proves source bytes plus the complete structural/
   resolver profile are the only selector, expectation outcome/stage/tag/
   diagnostics/payload cannot select it, the Task-259 route remains isolated,
   and the existing mixed fixture remains on its unchanged gap. The existing
   metadata suite additionally proves the future sidecar/trace requirement is
   reciprocal, sole-backlinked, sorted, and counted exactly once; and
4. the non-publication test proves both computation/justification subtrees
   remain unconsumed and no composed goal, proof/discharge, accepted functor,
   symbol activation, fact/axiom, VC/IR, or public diagnostic is published.
   It also covers the six mechanical active-count consumers and projected
   CLI totals.

The future implementation may change only the new checker producer and
private test support; checker `lib.rs`, typed/final owners including the
`typed_ast.rs` exhaustive obligation-kind serializer, both external
serializers in `type_checker.rs` and `registration_resolution.rs`, and lint
policy; the new private runner route,
parent facades, test include/leaf, six mechanical active-type count
assertions; the new fixture/sidecar/trace row; and synchronized derived EN/JA
records. Parser, resolver, Cargo metadata, canonical specifications, existing
`.miz`, existing expectations/sidecars, and unrelated lower producers are
forbidden.

The `source_spec_audit.md` module-spec, crate-export, and public-surface
inventories must enumerate the exact API above. `tests/lint_policy.rs` must
add `source_functor_definition.rs`/`.md` to all three corresponding
allowlists: documented public module, public enum policy, and source/spec
audit coverage. The existing syntax-dependency scan remains unchanged and
receives no exception.

All three exhaustive serializers add exactly these byte mappings:

```text
FunctorExistence => "functor_existence"
FunctorUniqueness => "functor_uniqueness"
```

Checker library count projects `435 -> 440`; runner `512 -> 516`; resolver
and syntax stay `144/59`. Corpus/requirements project `422/390 -> 423/391`,
pass/fail `229/193 -> 230/193`, active parse/declaration/type/proof
`101/7/199/1 -> 101/7/200/1`, and type requirements `254/242 -> 255/243`.
Warnings/errors remain `23/0`.

## Semantic Deferrals And Exit Criteria

Deferred and forbidden in Task 260 are: parameter/guard/return-type goal
composition; FOL existence/uniqueness construction; proof parsing or
verification beyond subtree preservation; discharge, acceptance, activation,
facts/axioms, equality/uniqueness reasoning, overload selection, calls to the
new functors, conditional/case/otherwise consistency or coverage, dependent
or attributed return semantics, recursion checks, redefinition, notation,
properties, composite formula definientia, imported functors, mixed
predicate/functor acceptance, Core/CFG/VC, and every Task 261+ family.

The documentation prerequisite exits only with synchronized EN/JA, repeated
review-only **NO FINDINGS**, unchanged executable artifacts/counts/hashes, all
nine hard gates PASS, valid quality at least 90/100, exact staging, one docs
commit, clean worktree, unchanged protected stash, and fresh post-commit
inventory selecting the implementation. Implementation has the same review
and hard-gate requirements plus the projected executable counts and one
dedicated logical-task commit.
