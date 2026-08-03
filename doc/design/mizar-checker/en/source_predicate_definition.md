# Source Predicate-Definition Intake

> Canonical language: English. Japanese companion:
> [../ja/source_predicate_definition.md](../ja/source_predicate_definition.md).

Status: Checker Task 259 implemented contract plus post-Task-248 contract
correction. The frozen boundary below is now implemented by the syntax-free
producer, its exact real consumer, and one covered trace backlink. The
separately committed Task-248 Profile-B dependency remains completed current
state.

## Authority, Scope, And Findings

The canonical authority is:

- Chapter 9 §§9.1 and 9.3-9.5 for ordinary predicate declarations, patterns,
  ordered typed parameters, definition-local `assume` guards, `means`
  definiens, and predicate properties;
- Chapter 9 §§9.9.3-9.9.5 for the definition biconditional, typed guards, and
  domain restrictions;
- Chapter 16 §16.6 for definition-time correctness obligations;
- the existing parser predicate-definition and property-clause fixtures and
  tests, the existing mixed predicate/functor type-elaboration boundary, and
  resolver declaration-shell/signature tests; and
- the public APIs completed by Checker Tasks 248-258.

The missing exact contract is nonblocking `design_drift`; the absent
source-to-checker producer is `source_drift`; and the absent dedicated real
consumer is a `test_gap`. There is no blocking `spec_gap`. Chapter 9 states
both that an `assume` guard restricts the definition domain and that a symmetry
property creates an obligation, but it does not specify the exact
guard-to-symmetry VC formula or quantifier/antecedent construction. Task 259
therefore transports a deterministic opaque pending obligation identity and
explicitly defers semantic FOL goal construction. It does not infer that
construction from current source behavior.

This task owns only predicate-definition intake: definition identity,
parameters, guards, the already produced atomic definiens, property identity,
and one initial-obligation link. It does not own a property proof, recursive
unfolding, truth, acceptance, or axiom publication.

## Exact Future Source

The dedicated future pass fixture is exactly these 165 UTF-8 bytes, including
the final LF:

```mizar
definition
  let x be set;
  let y be set;
  assume x = x;
  pred Task259PredicateDefinition: x task259_rel y means x = y;
  symmetry by computation(steps: 1);
end;
```

Its SHA-256 is
`91bdb5f51c0ea5f07bdd831700cb9803f2aa57e005921c7e4e1798ecbbf2bd9f`.
The source contains one ordinary `means` predicate, two separately written
`set` parameters, one equality guard, one equality definiens, and one
`symmetry` property with an explicit computation justification. It contains no
functor, theorem, proof block, import, correctness keyword, or recovery.

## Frozen Surface Profile

The exact parser result is 71 stored Surface rows, root node 70, root range
`0..164`, and no recovery. The relevant rows and ranges are:

| Node | Surface kind | Range | Direct role |
| ---: | --- | --- | --- |
| 38 | `TypeHead` | `22..25` | first builtin `set` head |
| 39 | `TypeExpression` | `22..25` | first written parameter type |
| 41 | `DefinitionParameter` | `13..26` | first `let x be set;` |
| 42 | `TypeHead` | `38..41` | second builtin `set` head |
| 43 | `TypeExpression` | `38..41` | second written parameter type |
| 45 | `DefinitionParameter` | `29..42` | second `let y be set;` |
| 50 | `BuiltinPredicateApplication` | `52..57` | guard equality |
| 51 | `FormulaExpression` | `52..57` | guard formula |
| 53 | `AssumptionStatement` | `45..58` | definition-local guard owner |
| 54 | `PredicatePattern` | `94..109` | `x task259_rel y` |
| 59 | `BuiltinPredicateApplication` | `116..121` | definiens equality |
| 60 | `FormulaExpression` | `116..121` | definiens formula |
| 61 | `FormulaDefiniens` | `116..121` | predicate definiens owner |
| 62 | `PredicateDefinition` | `61..122` | predicate declaration |
| 64 | `ComputationJustification` | `137..158` | proof-content subtree |
| 65 | `JustificationClause` | `134..158` | explicit justification subtree |
| 66 | `PropertyClause` | `125..159` | symmetry property |
| 67 | `DefinitionBlockItem` | `0..164` | common direct owner |
| 70 | `Root` | `0..164` | complete Surface root |

The binding declaration ranges are `x 17..18` and `y 33..34`; the written
type ranges are `22..25` and `38..41`. The predicate declaration label is
`66..92`, the pattern is `94..109`, and its symbol token is `96..107`.

Nodes 41, 45, 53, 62, and 66 are direct structural siblings of node 67 in
that order. Parameters and the assumption are not children of node 62.
Consequently the private runner selector, not the checker and not the
resolver, authenticates their common normal block, order, containment, and
association with the sole predicate. Lower-family traversal of the guard and
definiens must select only the two equality subtrees and must not treat pattern
loci, the declaration label, or the property-justification subtree as term or
formula occurrences.

## Frozen Raw Resolver Profile

Before type-elaboration enrichment, the raw resolver profile is exactly:

- three declaration shells, two signature projections, zero symbol
  diagnostics, two symbols, two definitions, and one local-source
  contribution;
- shell 0: `DefinitionBlock`, node 67, ordinal 0, no parent;
- shell 1: `PredicateDefinition`, node 62, ordinal 1, parent shell 0;
- shell 2: `PropertyClause`, node 66, ordinal 2, parent shell 0;
- predicate `DefinitionId(0)`: `SymbolKind::Predicate`,
  `DefinitionKind::Predicate`, spelling and notation
  `x task259_rel y`, origin anchor `61..122`, structural path
  `[4,0,8,0]`, normal, conflict-free, local and exported; and
- generic property projection: `SymbolKind::Attribute`,
  `DefinitionKind::Attribute`, origin anchor `125..159`, structural path
  `[4,0,17,1]`.

The generic property projection is resolver collection scaffolding only. Task
259 MUST NOT reinterpret or consume it as semantic evidence that the property
is a predicate property, is symmetric, is proved, or is accepted.

`DefinitionParameter` and `AssumptionStatement` have no declaration shells.
The predicate resolver definition has empty parameter and binder collections
and no syntactic arity. Task 259 must not infer arity two, parameter identity,
or guard ownership from resolver fields. Those facts come only from the exact
private source selector and the separately extended Task-248 handoff.

## Mandatory Separate Lower Prerequisite — Completed

At the time the original Task-259 contract was frozen, the Task-248 producer
accepted only its reserve-plus-one-shadowing-parameter Profile A. It rejected
this source's one normal definition block with two separately typed
parameters. The contract therefore required, and still requires, Task 259 not
to reconstruct a `BindingEnv`, fabricate `BindingId`s, or broaden Task 248
inside the Task-259 implementation commit.

The required separate logical tasks are now complete:

1. `f9b47375acc18acebf56a69f5d8a7edec539c2be` freezes Profile B as exactly one
   normal `DefinitionBlock`, two ordered separately written
   `DefinitionParameter` bindings, and no reserve item; and
2. `ca54135f36c9fecfc02c2b8120ec4e63e8c6ca36` implements that profile and the
   private exact runner selector while preserving the existing
   `SourceBindingContextHandoff`.

Fresh post-`ca54135f` inventory confirms the exact 165-byte source now yields
two checker-authenticated `BindingId`s, one definition-local
`BindingContextId`, exact parameter sites, and no shadow/capture edge. Profile
A remains unchanged. Task 259 implementation is therefore the next
dependency-ready logical task; it consumes this handoff and must not duplicate
its ownership.

## Frozen Lower Consumer Bundle

After the separate Task-248 extension, the exact lower bundle is:

| Owner | Exact profile |
| --- | --- |
| Task 248 | existing `SourceBindingContextHandoff`, extended only for the exact one-block/two-parameter profile |
| Task 249 | 2 applications / 2 expressions / 0 arguments |
| Task 252 | 4 `VariableReference` terms / 4 binding references / 0 numeric requests |
| Task 256 | 2 `Equality` formulas / 0 wrappers / 0 segments / 0 heads / 0 candidates / 0 type sites / 0 attributes / 4 edges / 4 requests |

In public Task-256
formula/wrapper/segment/head/candidate/type-site/attribute/edge/request order,
the last profile is exactly `2/0/0/0/0/0/0/4/4`.

Task-252 source order is guard `x`, guard `x`, body `x`, body `y`. The first
two references select the first parameter binding and the latter two select
the first and second parameter bindings. Task-256 formula 0 is guard `x = x`
and formula 1 is definiens `x = y`; each owns two
`BuiltinLeftOperand`/`BuiltinRightOperand` edges and two
`OperandExpectedType` requests.

The Task-259 handoff fingerprints the exact Task-248, Task-249, Task-252, and
Task-256 handoffs. Tasks 253-255, 257, and 258 are absent. In particular, the
definition-local `AssumptionStatement` is a Task-259 guard and is not a
`SourceStatement` row or Task-258 assumption/fact.

## Public Syntax-Free Contract

Implementation adds a checker-owned source module with five dense ID families:

```rust
pub struct SourcePredicateDefinitionId(usize);
pub struct SourcePredicateParameterId(usize);
pub struct SourcePredicateGuardId(usize);
pub struct SourcePredicatePropertyId(usize);
pub struct SourcePredicateCorrectnessId(usize);
```

Each ID is `Copy + Eq + Ord + Hash`, has the existing dense-ID `new` and
`index` API, and is allocated by vector order. The public input and row
families are frozen as follows; raw `SurfaceAst`, `SurfaceNodeId`,
`SyntaxKind`, and parser nodes never cross this seam:

```rust
pub struct SourcePredicateDefinitionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub definitions: Vec<SourcePredicateDefinitionInput>,
    pub parameters: Vec<SourcePredicateParameterInput>,
    pub guards: Vec<SourcePredicateGuardInput>,
    pub properties: Vec<SourcePredicatePropertyInput>,
    pub correctness: Vec<SourcePredicateCorrectnessInput>,
}

pub struct SourcePredicateDefinitionInput {
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourcePredicateDefinitionRecovery,
    pub spelling: String,
    pub definiens: SourceAtomicFormulaId,
}

pub struct SourcePredicateParameterInput {
    pub owner: SourcePredicateDefinitionId,
    pub ordinal: usize,
    pub binding: BindingId,
    pub written_type: SourceTypeApplicationId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub declaration_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourcePredicateDefinitionRecovery,
    pub spelling: String,
}

pub struct SourcePredicateGuardInput {
    pub owner: SourcePredicateDefinitionId,
    pub ordinal: usize,
    pub formula: SourceAtomicFormulaId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourcePredicateDefinitionRecovery,
    pub spelling: String,
}

pub struct SourcePredicatePropertyInput {
    pub owner: SourcePredicateDefinitionId,
    pub ordinal: usize,
    pub kind: SourcePredicatePropertyKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub justification: SourceAnchor,
    pub recovery: SourcePredicateDefinitionRecovery,
    pub spelling: String,
}

pub struct SourcePredicateCorrectnessInput {
    pub owner: SourcePredicateDefinitionId,
    pub property: SourcePredicatePropertyId,
    pub ordinal: usize,
    pub source_anchor: SourceAnchor,
}

#[non_exhaustive]
pub enum SourcePredicatePropertyKind {
    Symmetry,
}

#[non_exhaustive]
pub enum SourcePredicateDefinitionRecovery {
    Normal,
    Degraded,
}
```

## Public Immutable Output API

The immutable row type names and stored fields are exact:

| Row type | Stored fields, in API order |
| --- | --- |
| `SourcePredicateDefinition` | `id`, `symbol`, `definition`, `contribution`, `site`, `source_range`, `source_ordinal`, `context`, `recovery`, `spelling`, `definiens`, derived `origin` |
| `SourcePredicateParameter` | `id`, `owner`, `ordinal`, `binding`, `written_type`, `site`, `source_range`, `declaration_range`, `context`, `recovery`, `spelling` |
| `SourcePredicateGuard` | `id`, `owner`, `ordinal`, `formula`, `site`, `source_range`, `context`, `recovery`, `spelling` |
| `SourcePredicateProperty` | `id`, `owner`, `ordinal`, `kind`, `site`, `source_range`, `justification`, `recovery`, `spelling` |
| `SourcePredicateCorrectness` | `id`, `owner`, `property`, `ordinal`, `source_anchor`, derived `obligation` |

Every stored field has one same-named read-only getter. Dense IDs, resolver
IDs, ordinals, ranges, contexts, recovery/kind enums, and the obligation are
returned by value. `symbol`, `site`, `origin`, `justification`, and
`source_anchor` are returned by shared reference; `spelling()` returns
`&str`. There are no setters, mutable getters, public constructors, or row
replacement APIs.

The table names and signatures are exact:

```rust
pub struct SourcePredicateDefinitionTable { /* private rows */ }
pub struct SourcePredicateParameterTable { /* private rows */ }
pub struct SourcePredicateGuardTable { /* private rows */ }
pub struct SourcePredicatePropertyTable { /* private rows */ }
pub struct SourcePredicateCorrectnessTable { /* private rows */ }
```

Each table exposes only `get(id) -> Option<&Row>`,
`iter() -> impl Iterator<Item = (Id, &Row)>`, `const len() -> usize`, and
`const is_empty() -> bool`. `iter` is dense source order and never sorts.

The handoff and producer surface is exact:

```rust
pub struct SourcePredicateDefinitionHandoff { /* private fields */ }

impl SourcePredicateDefinitionHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn source_context_fingerprint(&self) -> &str;
    pub fn source_type_fingerprint(&self) -> &str;
    pub fn source_term_fingerprint(&self) -> &str;
    pub fn source_atomic_formula_fingerprint(&self) -> &str;
    pub const fn definitions(&self) -> &SourcePredicateDefinitionTable;
    pub const fn parameters(&self) -> &SourcePredicateParameterTable;
    pub const fn guards(&self) -> &SourcePredicateGuardTable;
    pub const fn properties(&self) -> &SourcePredicatePropertyTable;
    pub const fn correctness(&self) -> &SourcePredicateCorrectnessTable;
    pub fn debug_text(&self) -> String;
}

pub struct SourcePredicateDefinitionProducer;
```

The four fingerprints are the complete corresponding lower-handoff
`debug_text()` strings, not caller-supplied digests and not optional values.
The producer derives them after validating that all four lower handoffs share
the exact source/module identity and install correctly against the same typed
arena.

## Public Enum Policy

| Public enum | Compatibility policy |
| --- | --- |
| `SourcePredicatePropertyKind` | `#[non_exhaustive]`; callers must tolerate later explicitly frozen predicate-property kinds. |
| `SourcePredicateDefinitionRecovery` | `#[non_exhaustive]`; callers must tolerate later recovery classes. |
| `SourcePredicateDefinitionError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

The transactional build result and error surface is also frozen:

```rust
pub struct SourcePredicateDefinitionProjection {
    base_initial_obligations: InitialObligationTable,
    handoff: SourcePredicateDefinitionHandoff,
    initial_obligations: InitialObligationTable,
}

impl SourcePredicateDefinitionProjection {
    pub const fn base_initial_obligations(
        &self,
    ) -> &InitialObligationTable;
    pub const fn handoff(&self) -> &SourcePredicateDefinitionHandoff;
    pub const fn initial_obligations(&self) -> &InitialObligationTable;
    pub fn into_parts(
        self,
    ) -> (
        InitialObligationTable,
        SourcePredicateDefinitionHandoff,
        InitialObligationTable,
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePredicateDefinitionError {
    SourceIdentityMismatch,
    DependencyMismatch,
    InvalidResolverDefinition,
    InvalidDefinition { index: usize },
    InvalidParameter { index: usize },
    InvalidGuard { index: usize },
    InvalidProperty { index: usize },
    InvalidCorrectness { index: usize },
    InvalidObligation,
    InvalidArenaOwnership,
    UnsupportedTaskShape,
}

impl std::fmt::Display for SourcePredicateDefinitionError;
impl std::error::Error for SourcePredicateDefinitionError;

impl SourcePredicateDefinitionProducer {
    pub fn build(
        input: SourcePredicateDefinitionHandoffInput,
        env: &SymbolEnv,
        source_context: &SourceBindingContextHandoff,
        source_type: &SourceTypeApplicationHandoff,
        source_term: &SourcePrimaryTermHandoff,
        source_atomic_formula: &SourceAtomicFormulaHandoff,
        base_initial_obligations: &InitialObligationTable,
        arena: &TypedArena,
    ) -> Result<
        SourcePredicateDefinitionProjection,
        SourcePredicateDefinitionError,
    >;
}
```

The producer clones the authenticated baseline table, preserves every
existing row and ID byte-for-byte, and appends exactly one row at
`InitialObligationId(base_initial_obligations.len())`. The projection retains
both the baseline clone and completed table so installation can reject stale
or colliding state. The exact future fixture has an empty baseline, so its
new row is `InitialObligationId(0)` and the completed table has one row.

`SourcePredicateDefinitionError` has no `Default` or blanket conversion. Its
variants are fail-closed aggregate categories for source/provenance,
dependencies, each dense row family, obligation construction, arena
ownership, and the exact admitted profile. Callers retain a wildcard because
the enum is non-exhaustive.

The immutable output rows add their corresponding dense `id`; the definition
row additionally stores the resolver-derived `SemanticOrigin`, and the
correctness row stores the producer-allocated `InitialObligationId`. Neither
derived value is caller-supplied. The five public immutable tables are
`SourcePredicateDefinitionTable`, `SourcePredicateParameterTable`,
`SourcePredicateGuardTable`, `SourcePredicatePropertyTable`, and
`SourcePredicateCorrectnessTable`. Each exposes only `get`, source-ordered
`iter`, `len`, and `is_empty`; it exposes no insertion or replacement API.

`SourcePredicateDefinitionHandoff` owns the source/module identities, the
five tables, and all four dependency fingerprints derived inside
`SourcePredicateDefinitionProducer::build` from the exact lower-handoff
`debug_text()` values. The caller cannot supply a fingerprint. Its getters
are borrowed and read-only. Stable row-family/debug keys are exactly:

- `source.definition.predicate`;
- `source.definition.predicate.parameter`;
- `source.definition.predicate.guard`;
- `source.definition.predicate.property`; and
- `source.definition.predicate.correctness`.

All enums are `#[non_exhaustive]`. The exact source accepts only normal rows;
`Degraded` exists as a fail-closed extension boundary and is rejected by this
profile.

## Frozen Debug Grammar

`SourcePredicateDefinitionHandoff::debug_text()` is the sole dependency
fingerprint and begins exactly with
`source-predicate-definition-debug-v1\n`. It then emits these line families
in this exact order, with one final LF and no blank line:

```text
source-predicate-definition-debug-v1
module: <ModuleId.path>
source-context-fingerprint: <Rust-debug String>
source-type-fingerprint: <Rust-debug String>
source-term-fingerprint: <Rust-debug String>
source-atomic-formula-fingerprint: <Rust-debug String>
definition#<id> symbol=<Rust-debug FQN string> definition=<id> contribution=<id> ordinal=<n> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> origin_range=<start>..<end> origin_path=<Rust-debug [u32]> spelling=<Rust-debug String> definiens=<id>
parameter#<id> owner=<id> ordinal=<n> binding=<id> written_type=<id> range=<start>..<end> declaration_range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
guard#<id> owner=<id> ordinal=<n> formula=<id> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
property#<id> owner=<id> ordinal=<n> kind=<symmetry> range=<start>..<end> site=node#<id> justification=range:<start>..<end> recovery=<normal|degraded> spelling=<Rust-debug String>
correctness#<id> owner=<id> property=<id> ordinal=<n> anchor=range:<start>..<end> obligation=<id>
```

`Rust-debug String` means the standard escaped `{:?}` rendering of the owned
string. The exact admitted profile accepts only `TypedSiteRef::Node` and
`SourceAnchor::Range`; role sites, point/generated anchors, imported or
recovered origins, and any extra grammar branch fail closed. The definition
origin line omits source/module/import/recovery fields because validation has
already fixed them to the handoff source/module, `import_edge = None`, and
`recovered = false`. Row lines are emitted in dense table order:
definition, parameter, guard, property, correctness. Typed and final debug
rendering include this complete text exactly once and do not render a second
Task-259 summary.

## Exact Five-Table And Obligation Oracle

The table cardinality is exactly `1/2/1/1/1` in
definition/parameter/guard/property/correctness order.

- Definition 0 authenticates resolver `DefinitionId(0)` and its predicate
  symbol/contribution/origin, source ordinal 0, range `61..122`, the
  definition-local context, spelling
  `pred Task259PredicateDefinition: x task259_rel y means x = y;`, and
  Task-256 `SourceAtomicFormulaId(1)` as the definiens.
- Parameters 0 and 1 belong to definition 0. They preserve ordinals 0 and 1,
  the Task-248 `x` and `y` bindings, Task-249 applications 0 and 1, ranges
  `13..26` and `29..42`, declaration ranges `17..18` and `33..34`, the same
  definition-local context, and spellings `let x be set;` and
  `let y be set;`.
- Guard 0 belongs to definition 0, uses Task-256
  `SourceAtomicFormulaId(0)`, range `45..58`, the definition-local context,
  and spelling `assume x = x;`.
- Property 0 belongs to definition 0, has ordinal 0, kind `Symmetry`, range
  `125..159`, spelling `symmetry by computation(steps: 1);`, and only the
  explicit justification anchor `SourceAnchor::Range(134..158)`.
- Correctness 0 links definition 0, property 0, ordinal 0, obligation 0, and
  `SourceAnchor::Range(125..159)`.

Task 259 appends `InitialObligationKind::PredicatePropertyCorrectness`. The
producer returns exactly one complete `InitialObligationTable` row:

| Field | Exact value |
| --- | --- |
| id | `InitialObligationId(0)` |
| kind | `PredicatePropertyCorrectness` |
| owner | property 0's authenticated typed site |
| range | `125..159` |
| assumptions | empty |
| goal | opaque key `source.definition.predicate.correctness:property=0` |
| provenance | opaque key `source.definition.predicate:definition=0:property=0` |
| status | `Pending` |

The goal and provenance strings are deterministic transport identities, not a
claim about a FOL formula. The exact ordinary `means` predicate adds no
existence or uniqueness obligation. Task 259 creates no assumption facts from
the guard; therefore the obligation's `assumptions` vector is empty.

## Authentication And Validation

`SourcePredicateDefinitionProducer::build` is transactional. It receives the
syntax-free input, `SymbolEnv`, the existing extended
`SourceBindingContextHandoff`, Task-249 type handoff, Task-252 primary-term
handoff, Task-256 atomic-formula handoff, the authenticated current
`InitialObligationTable` baseline, and the typed arena. It returns a
projection containing the baseline clone, complete immutable predicate
handoff, and completed obligation table, or returns an error without
publishing any of them.

The predicate resolver `SymbolEntry`, `DefinitionEntry`, and contribution must
match the source/module, local-source contribution, normal origin, range
`61..122`, predicate symbol/definition kinds, exact spelling/notation, public
local declaration visibility/export state, and conflict-free definition.
Every lower ID, site, context, range, ordinal, spelling, and four dependency
fingerprints must match its owning handoff and typed-arena source key.

The property is authenticated only by the private runner proving that normal
Surface node 66 is a direct later sibling of predicate node 62 in the same
normal block 67, by source order and non-overlapping ranges, and by matching
the supplied typed site to the typed-arena source key. The checker validates
that syntax-free relationship. It never derives property meaning from the
resolver's generic Attribute projection.

Validation rejects missing, duplicate, reordered, dangling, cross-owner,
cross-module, recovered/degraded, stale-site, stale-context, stale-range,
stale-origin, stale-contribution, stale-symbol/definition, stale-binding,
stale-lower-ID, stale-fingerprint, wrong-kind, wrong-spelling, wrong-ordinal,
partial, or extra rows. It also rejects a wrong obligation owner, range, kind,
status, goal, provenance, assumptions, property link, correctness anchor, or
cardinality. Input order is validated, never sorted or repaired.

## Justification And Semantic Boundary

Task 259 stores only the explicit `134..158` justification anchor. It does not
consume, copy, lower, interpret, or validate node 64
`ComputationJustification` or node 65 `JustificationClause` proof content.
Task 258 remains absent from this exact route. Future Task 272 ownership of
proof skeleton and justification content is preserved; Task 259 neither
excludes that subtree from Task 272 nor pre-accepts it.

No property proof, proof search, discharge result, `VcId`, accepted obligation,
accepted predicate definition, type fact, theorem fact, biconditional axiom,
or ATP premise is produced.

## Typed And Final Installation

`TypedAst` installs the Task-259 handoff and its complete obligation table
atomically through one installation API. Installation requires all four lower
handoffs, reproduces their exact fingerprints, validates all typed sites, and
rejects prior/partial Task-259 occupancy. It also requires its current
`InitialObligationTable` to equal the projection's retained baseline exactly;
otherwise it returns the dedicated typed error without changing either
field. On success every baseline row/ID is preserved and only the one
producer-created row is appended. Neither installation order nor an error
may expose only one of the two outputs.

The exact public installation surface is:

```rust
impl TypedAst {
    pub fn with_source_predicate_definition(
        self,
        projection: SourcePredicateDefinitionProjection,
    ) -> Result<Self, TypedAstError>;

    pub const fn source_predicate_definition(
        &self,
    ) -> Option<&SourcePredicateDefinitionHandoff>;
}

// Added to the existing non-exhaustive enum.
TypedAstError::InvalidSourcePredicateDefinition
```

`TypedAstParts` does not gain a Task-259 field and is not a second install
path. It continues to establish the authenticated baseline obligation table;
the one-shot method above is the sole Task-259 allocator/publication path.

`ResolvedTypedAst` accepts no separately replaceable runner input. It
revalidates the typed-owned handoff, lower fingerprints, five tables, and
obligation row, then clone-preserves them. Empty debug rendering remains
byte-stable; nonempty rendering is deterministic and uses the five frozen
keys. Clone, rerun, and equivalent-input results must be byte-identical.

`ResolvedTypedAst::assemble(ResolvedTypedAstInputs<'_>)` keeps its existing
signature and obtains Task 259 only from `inputs.typed_ast`. It adds:

```rust
impl ResolvedTypedAst {
    pub const fn source_predicate_definition(
        &self,
    ) -> Option<&SourcePredicateDefinitionHandoff>;
}

// Added to the existing non-exhaustive enum.
ResolvedTypedAstError::InvalidSourcePredicateDefinition
```

No separate Task-259 field is added to `ResolvedTypedAstInputs`. Handoff
debug text begins with `source-predicate-definition-debug-v1`; typed/final
debug output includes that exact validated text only when the optional
handoff is present.

This boundary commits to no fact, VC, proof status, accepted definition,
artifact, Core IR, or Control-Flow IR.

## Dedicated Consumer And Trace Intent

The future implementation adds exactly:

- `tests/miz/pass/types/pass_type_elaboration_predicate_definition_payload_001.miz`;
- `tests/miz/pass/types/pass_type_elaboration_predicate_definition_payload_001.expect.toml`;
  and
- one new covered trace requirement
`spec.en.checker.type_elaboration.source_predicate_definition_payload`.

That trace row is frozen with:

```toml
source = "doc/design/mizar-checker/en/source_predicate_definition.md"
section = "Dedicated Consumer And Trace Intent"
stage = "type_elaboration"
status = "covered"
required = true
coverage = "pass"
```

The sidecar is frozen to `schema_version = 1`, id
`pass_type_elaboration_predicate_definition_payload_001`, kind `pass`,
stage `type_elaboration`, domain `checker.type_elaboration`, the source path
above, expected outcome `pass`, expected phase `type_check`, empty
`diagnostic_codes` and `diagnostic_payloads`, tag `active_type_elaboration`,
and exactly the future trace id in `spec_refs`. It has no failure category,
rejection reason, stable detail key, or failure payload.

The trace row is added only when the implementation and real consumer are
executable. The existing
`fail_type_elaboration_predicate_functor_definition_gap_001.miz`, its sidecar,
and all current trace rows remain byte-identical. That mixed predicate/functor
boundary remains gated on Task 260 and is not selected, promoted, or
reinterpreted by Task 259.

## Frozen Tests

The Task-259 checker implementation freezes these five focused tests:

1. `task_259_exact_predicate_definition_payload_and_pending_obligation`;
2. `task_259_independent_row_and_field_corruption_fails_closed`;
3. `task_259_dependency_and_obligation_corruption_fails_closed`;
4. `task_259_typed_installation_is_transactional`;
5. `task_259_final_clone_debug_determinism_and_family_isolation`.

They cover the exact `1/2/1/1/1` payload; every independent field, row, lower
dependency, and fingerprint; obligation status, goal, provenance, assumptions,
owner, range, and link; partial/duplicate/reordered occupancy; atomic
installation; immutable final clone; deterministic debug; and isolation from
Tasks 253-255, 257, 258, 260+, facts, proofs, and accepted status.

The transactional test also uses a valid nonempty baseline. It proves every
baseline row/ID is preserved byte-for-byte, the new ID equals
`baseline.len()`, all projection getters and `into_parts` preserve both
tables, and exact-baseline installation succeeds. Independent stale, missing,
extra, reordered, and colliding current-baseline mutations must return
`InvalidSourcePredicateDefinition` with no Task-259 row, obligation
replacement, or partial replay. Final clone/debug tests preserve both the
baseline and appended rows.

The runner implementation freezes these four focused tests:

1. `task259_real_source_surface_resolver_and_lower_bundle_is_exact`;
2. `task259_source_ast_resolver_and_lower_mutations_fail_at_the_owner`;
3. `task259_expectation_selection_and_mixed_definition_route_stay_isolated`;
4. `task259_route_publishes_no_property_proof_fact_or_acceptance`.

They authenticate all 165 source bytes, 71 Surface rows, raw resolver profile,
lower counts and associations, subtree exclusions, exact pass sidecar
selection, mixed-route preservation, replay, mutation ownership, and absence
of proof acceptance.

## Exact Implementation Consumers And Write Scope

The implementation logical task has these exact code/test consumers:

- `crates/mizar-checker/src/source_predicate_definition.rs` for the complete
  producer, five rows/tables, handoff/projection, validation, debug grammar,
  and the five focused checker tests;
- `crates/mizar-checker/src/lib.rs`,
  `crates/mizar-checker/src/typed_ast.rs`, and
  `crates/mizar-checker/src/resolved_typed_ast.rs` for the public module,
  one-shot typed installation, final clone/revalidation, getters, dedicated
  errors, and exact debug inclusion;
- `crates/mizar-checker/src/type_checker.rs` and
  `crates/mizar-checker/src/registration_resolution.rs` only for their
  existing exhaustive debug serializers of the new
  `InitialObligationKind`;
- `crates/mizar-checker/tests/lint_policy.rs` for the documented-module,
  public-enum, and source/spec-audit allowlists; its syntax-boundary test
  automatically scans every checker `.rs` file and needs no task-specific
  allowlist entry;
- `crates/mizar-test/src/runner.rs`,
  `crates/mizar-test/src/runner/type_elaboration.rs`,
  new private
  `crates/mizar-test/src/runner/type_elaboration/source_predicate_definition.rs`,
  `crates/mizar-test/src/runner/tests.rs`, and new
  `crates/mizar-test/src/runner/tests/type_elaboration/source_predicate_definition.rs`
  for exact selection, syntax-free input construction, lower-stage
  composition, routing, and the four focused runner tests;
- `crates/mizar-test/tests/metadata.rs` only for the four existing mechanical
  active-type count assertions;
- the one new `.miz`, same-stem sidecar, and one trace row named under
  **Dedicated Consumer And Trace Intent**; and
- synchronized Task-259 implementation/closure records in checker and
  mizar-test EN/JA crate plan, todo, module-boundary audit, the checker EN/JA
  source/spec audit, this EN/JA module spec, and
  `doc/design/spec_coverage_audit.md`.

`ResolvedTypedAst` privately clone-preserves the typed-owned complete
`InitialObligationTable` so it can revalidate the Task-259 correctness link;
it adds no public table getter and no second input. No Cargo manifest is
expected to change. Existing specifications, existing `.miz` files, existing
sidecars/expectations/trace rows, the mixed Task-260 case, and every Task-260+
implementation owner are outside the write scope. An independently discovered
mechanical consumer may be added only after classification and review; it may
not broaden language or test intent.

## Deferrals And Forbidden Scope

Task 259 forbids:

- recursive predicate unfolding or evaluation;
- constructing the guard-to-property FOL VC;
- consuming or proving the property justification, proof search, or discharge;
- `VcId`, facts, axioms, ATP premises, or theorem publication;
- accepted obligation, accepted definition, or semantic truth;
- overload candidate collection or winner selection;
- inferring signature parameters, binder identity, or arity from the resolver;
- reconstructing `BindingEnv` or widening Task 248 inside Task 259;
- Core IR, Control-Flow IR, VC lowering, artifacts, or public diagnostics; and
- Task 260 or any later definition, property-proof, overload, redefinition, or
  semantic owner.

Tasks 260-264, 269-279, the later advanced-semantics runner, and all
downstream semantic stages retain their existing ownership.

## Baseline, Audit Impact, And Exit

This documentation prerequisite changes no production source, tests, `.miz`
fixtures, sidecars, trace row, trace status, count, backlink, coverage credit,
or runner selection. Its frozen baseline is:

- plan/requirements `421/389`;
- pass/fail `228/193`;
- active parse/declaration/type/proof `101/7/198/1`;
- declaration coverage: 12 requirements = 7 covered + 5 partial;
- type coverage: 253 requirements = 241 covered + 12 deferred; and
- warnings/errors `23/0`.

After the future single pass sidecar and single covered trace row, the expected
oracle is plan/requirements `422/390`, pass/fail `229/193`, active
parse/declaration/type/proof `101/7/199/1`, and type coverage 254 requirements
/ 242 covered. These are expected deltas, not substitutes for fresh measured
counts and hashes.

The original documentation task exited after canonical EN/JA synchronization,
review, verification, and its dedicated commit. The separate Task-248
documentation and implementation prerequisites are now complete at
`f9b47375` and `ca54135f`. This contract-correction prerequisite exits only
after EN/JA synchronization, repeated review with no findings, docs-only
verification, exact staging, one dedicated documentation commit, and clean
post-commit inventory with the protected stash unchanged. Its next task is
Task-259 production implementation.

## Task 259 Implementation Result

The implementation realizes the frozen API without widening it. The public
module exposes the five dense ids; six input aggregates; two data enums; five
immutable rows and five source-ordered tables; the handoff, projection,
non-exhaustive error, and producer. The exact output is `1/2/1/1/1`, and the
projection retains the authenticated baseline while appending exactly one
`Pending` `PredicatePropertyCorrectness` obligation with empty assumptions and
the frozen opaque goal/provenance.

`TypedAst::with_source_predicate_definition` installs the handoff and complete
obligation table atomically. Final assembly accepts no second Task-259 input,
privately clone-preserves the complete table, revalidates the five tables and
Task-248/249/252/256 fingerprints, and renders the handoff once. No fact,
proof, VC, axiom, accepted definition, or public diagnostic is produced.

The private runner authenticates the exact 165-byte/final-LF artifact whose
SHA-256 is
`91bdb5f51c0ea5f07bdd831700cb9803f2aa57e005921c7e4e1798ecbbf2bd9f`,
then composes Task 248, Task 249 `2/2/0`, Task 252 `4/4/0`, Task 256
`2/0/0/0/0/0/0/4/4`, and Task 259. The five checker tests and four runner
tests named under **Frozen Tests** pass. The test-only checker body resides in
`tests/support/source_predicate_definition_unit.rs`; it remains a private
child module using the existing dev-dependency, so physical production source
is syntax-free and no lint exception or public resolver test API was added.
Two existing source-statement isolation tests received only the reviewed
mechanical active-count correction `198 -> 199`; their empty-selection
assertions remain unchanged.

Measured executable metadata is cases/requirements `422/390`, pass/fail
`229/193`, active parse/declaration/type/proof `101/7/199/1`, type coverage
`254/242`, and warnings/errors `23/0`. The exact new trace requirement has the
single same-stem sidecar backlink. Guard-to-property FOL construction,
justification proof, discharge, acceptance, facts/axioms, and the mixed
Task-260 route remain deferred.

Completed verification is focused checker `5/5`, focused runner `4/4`,
checker library `435/435`, runner library `512/512`, resolver library
`144/144`, syntax library `59/59`, metadata `137/137`, both lint policies,
`cargo fmt --all --check`, workspace warnings-denied Clippy, workspace
`cargo test`, and all five CLI/count/hash gates. The four independent reviews
ended with no findings, and all nine hard gates pass with an uncapped
`100/100`. The task commit and post-commit fresh inventory remain pending.
