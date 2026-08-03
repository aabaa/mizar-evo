# Source Mode-Definition Transport

> Canonical language: English. Japanese companion:
> [../ja/source_mode_definition.md](../ja/source_mode_definition.md).

## Task 262 Scope And Authority

Checker Task 262 owns one syntax-free, immutable source-to-checker intake for
an ordinary parameterized `mode` definition, its normalized-right-hand-side
inhabitation request, and its explicit `sethood` correctness clause. Canonical
authority is Chapter 7 Sections 7.1--7.10, especially Sections 7.2, 7.7, and
7.8; Chapter 16 Sections 16.6 and 16.7.2 only where they identify definition
correctness as an obligation boundary; the existing mode-definition parser
pass/recovery fixtures; the active mixed mode/structure definition gap and its
sidecar/trace row; and the committed public Tasks 248--261 transports.

This task closes the missing exact producer `source_drift` and one
spec-derived executable-consumer `test_gap`. It does not decide whether the
RHS is inhabited, accept the mode, discharge `sethood`, compose a quantified
goal, publish a sethood/interface fact or concrete witness, activate an
expansion or registration, or lower proof/Core/ControlFlow/VC payloads.

Specification review found one mandatory lower `source_drift`: committed Task
249 requires every `SourceTypeApplicationInput` to be binding-linked at the
same ordinal, so two definition parameters cannot own a third RHS application.
Fabricating a third binding would be a `boundary_violation`. Chapter 7's
explicit mode RHS supplies sufficient authority for a separate checker-only
Task 249M standalone mode-RHS table. Task 249M documentation and implementation
must be separate commits before Task 262 implementation; this contract does
not authorize folding that lower change into Task 262.

## Frozen Exact Source

The future active source is exactly, including the final LF:

```mizar
definition
  let x be set;
  let y be set;
  mode Task262ModeDefinition: Task262Mode [x, y] is set;
  sethood by computation(steps: 1);
end;
```

It is 141 bytes, six lines, and has SHA-256
`3271f243670bd781c7167ff0d3bf463263a318abbe261aabdde1842c532a725e`.
It contains one normal definition block, two direct builtin-`set` context
parameters, one bracket-form mode application with ordered parameter
occurrences `x, y`, one bare builtin-`set` RHS/expansion, and one explicit
`sethood` clause with a computation justification.

The source has no `assume` statement, `equals`/`means` style, `->` return
type, term/formula definiens, attribute chain, imported/qualified name,
structure, property implementation, theorem, proof block, redefinition,
notation declaration, or recovery. `is` is the only mode-definition separator.
The RHS `set` is the mode's definiens/expansion; it is not a functor-style
return type and creates no Task-249R definition-return row.

## Frozen Literal Surface Oracle

The frontend produces zero diagnostics and exactly 54 dense Surface rows. The
root is node 53, range `0..140`, normal. Rows 0--33 are leaf tokens with no
children:

| Node | Token | Range | Node | Token | Range |
| ---: | --- | --- | ---: | --- | --- |
| 0 | `definition` | `0..10` | 17 | `,` | `87..88` |
| 1 | `let` | `13..16` | 18 | `y` | `89..90` |
| 2 | `x` | `17..18` | 19 | `]` | `90..91` |
| 3 | `be` | `19..21` | 20 | `is` | `92..94` |
| 4 | `set` | `22..25` | 21 | `set` | `95..98` |
| 5 | `;` | `25..26` | 22 | `;` | `98..99` |
| 6 | `let` | `29..32` | 23 | `sethood` | `102..109` |
| 7 | `y` | `33..34` | 24 | `by` | `110..112` |
| 8 | `be` | `35..37` | 25 | `computation` | `113..124` |
| 9 | `set` | `38..41` | 26 | `(` | `124..125` |
| 10 | `;` | `41..42` | 27 | `steps` | `125..130` |
| 11 | `mode` | `45..49` | 28 | `:` | `130..131` |
| 12 | `Task262ModeDefinition` | `50..71` | 29 | `1` | `132..133` |
| 13 | `:` | `71..72` | 30 | `)` | `133..134` |
| 14 | `Task262Mode` | `73..84` | 31 | `;` | `134..135` |
| 15 | `[` | `85..86` | 32 | `end` | `136..139` |
| 16 | `x` | `86..87` | 33 | `;` | `139..140` |

Rows 34--53 are exactly:

| Node | Surface kind | Range | Ordered children |
| ---: | --- | --- | --- |
| 34 | `TypeHead` | `22..25` | `[4]` |
| 35 | `TypeExpression` | `22..25` | `[34]` |
| 36 | `QualifiedVariableSegment` | `17..25` | `[2,3,35]` |
| 37 | `DefinitionParameter` | `13..26` | `[1,36,5]` |
| 38 | `TypeHead` | `38..41` | `[9]` |
| 39 | `TypeExpression` | `38..41` | `[38]` |
| 40 | `QualifiedVariableSegment` | `33..41` | `[7,8,39]` |
| 41 | `DefinitionParameter` | `29..42` | `[6,40,10]` |
| 42 | `ModePattern` | `73..91` | `[14,15,16,17,18,19]` |
| 43 | `TypeHead` | `95..98` | `[21]` |
| 44 | `TypeExpression` | `95..98` | `[43]` |
| 45 | `ComputationOption` | `125..133` | `[27,28,29]` |
| 46 | `ComputationJustification` | `113..134` | `[25,26,45,30]` |
| 47 | `JustificationClause` | `110..134` | `[24,46]` |
| 48 | `ModeProperty` | `102..135` | `[23,47,31]` |
| 49 | `ModeDefinition` | `45..135` | `[11,12,13,42,20,44,22,48]` |
| 50 | `DefinitionBlockItem` | `0..140` | `[0,37,41,49,32,33]` |
| 51 | `ItemList` | `0..140` | `[50]` |
| 52 | `CompilationUnit` | `0..140` | `[51]` |
| 53 | `Root` | `0..140` | tokens 0--33 followed by `[52]` |

The private runner authenticates every loaded byte, final LF, all row kinds,
ranges, recovery states, ordered children, root identity, and direct sibling
order before constructing syntax-free input. Checker production receives no
raw node kind, token, node number, parser type, or source text.

## Frozen Resolver Provenance

The resolver result is exactly two shells, one signature projection, zero
symbol diagnostics, one mode symbol, one mode definition, and one local-source
contribution:

- shell 0 is `DefinitionBlock`, node/range `50/0..140`, ordinal 0, no parent;
- shell 1 is `ModeDefinition`, node/range `49/45..135`, ordinal 1, parent 0;
- projection primary and notation spelling are exactly
  `Task262Mode [ x , y ]`, with `SymbolKind::Mode`,
  `DefinitionKind::Mode`, no syntactic arity, and overloadable status;
- definition 0 is normal, local, public/exported, conflict-free, and has
  structural origin path `[4,0,10,0]`; and
- the single contribution owns that symbol and definition.

The opaque parser signature roles are `ModePattern`, `TypeExpression`, and
`ModeProperty`. Resolver `parameters`, `binders`, and arity are empty. Task 262
must not reconstruct parameter declarations or application arguments from
those empty fields, infer RHS/inhabitation/sethood semantics from opaque
signature text, or treat resolver success as definition acceptance. The
authenticated Surface structure and lower handoffs are the only owners of
those associations.

## Frozen Lower Bundle And Ownership

After the separate Task-249M prerequisite, the exact source consumes only
these lower profiles:

| Owner | Exact profile | Task-262 ownership |
| --- | --- | --- |
| Task 248 | Profile B `1/2/2/2/2/2/0` | definition-block context and ordered bindings `x`, `y` |
| Task 249 | base applications/expressions/arguments `2/3/0` | parameter written types remain binding-linked applications 0/1; expression root 2 is standalone |
| Task 249M | mode-RHS rows `1` | independently owns expression root 2 and the definition/RHS source identity without a fabricated binding |
| Task 249R | absent | a mode RHS is not a definition return |
| Tasks 250--261 | absent | no attributes, terms, structures, sets, formulas, predicate/functor/attribute handoff |

All Task-248 rows belong to the authenticated block at node 50. Parameters
use `BindingId(0/1)`, `SourceTypeApplicationId(0/1)`, and the shared
definition context `BindingContextId(1)`. The bracket application owns the
ordered parameter-row vector `[0,1]`; those entries authenticate the pattern
occurrences at `86..87` and `89..90` without creating term rows. The expansion
and its inhabitation request both link to `SourceTypeModeRhsId(0)`, whose
canonical expression root 2 is node/range `44/95..98`.

Task 262 fingerprints exactly Task 248 `source_context` and the Task 249 plus
249M `source_type` handoff. It has no source-term, application-term, structure-term,
set-term, atomic-formula, composite-formula, return-type, attribute, or
evidence-response fingerprint. The runner may compose existing lower inputs;
it may not modify a lower producer or synthesize a missing lower row.

The separate Task-249M contract must freeze `SourceTypeModeRhsId`, one
`SourceTypeModeRhsInput`/immutable row/table, an exact-one-row extension
producer, `SourceTypeApplicationHandoff::mode_rhs()`, and a mode-RHS line in
the handoff's deterministic `debug_text()`. The row owns definition site/range,
source ordinal, and expression root 2, mirroring the independent ownership
shape of Task 249R without reusing its return semantics. Task 262 still accepts
one `&SourceTypeApplicationHandoff`; it validates `mode_rhs().get(0)` and its
root 2, and its complete source-type fingerprint therefore covers the Task-
249M row. Exact names/fields/errors/debug grammar and four lower tests belong
to the separate Task-249M frozen contract and commit.

## Exact Public Syntax-Free Input

Implementation adds `source_mode_definition.rs` with six dense ID families.
Each ID is `Copy + Eq + Ord + Hash`, exposes only `new` and `index`, and is
allocated by vector order:

```rust
pub struct SourceModeDefinitionId(usize);
pub struct SourceModeParameterId(usize);
pub struct SourceModeApplicationId(usize);
pub struct SourceModeExpansionId(usize);
pub struct SourceModeInhabitationRequestId(usize);
pub struct SourceModePropertyId(usize);
```

The exact public input is:

```rust
pub struct SourceModeDefinitionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub definitions: Vec<SourceModeDefinitionInput>,
    pub parameters: Vec<SourceModeParameterInput>,
    pub applications: Vec<SourceModeApplicationInput>,
    pub expansions: Vec<SourceModeExpansionInput>,
    pub inhabitation_requests: Vec<SourceModeInhabitationRequestInput>,
    pub properties: Vec<SourceModePropertyInput>,
}

pub struct SourceModeDefinitionInput {
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
    pub application: SourceModeApplicationId,
    pub expansion: SourceModeExpansionId,
    pub inhabitation_request: SourceModeInhabitationRequestId,
    pub property: Option<SourceModePropertyId>,
}

pub struct SourceModeParameterInput {
    pub owner: SourceModeDefinitionId,
    pub ordinal: usize,
    pub binding: BindingId,
    pub written_type: SourceTypeApplicationId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub declaration_range: SourceRange,
    pub pattern_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
}

pub struct SourceModeApplicationInput {
    pub owner: SourceModeDefinitionId,
    pub ordinal: usize,
    pub parameters: Vec<SourceModeParameterId>,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
}

pub struct SourceModeExpansionInput {
    pub owner: SourceModeDefinitionId,
    pub ordinal: usize,
    pub rhs: SourceTypeModeRhsId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
}

pub struct SourceModeInhabitationRequestInput {
    pub owner: SourceModeDefinitionId,
    pub ordinal: usize,
    pub expansion: SourceModeExpansionId,
    pub kind: SourceModeInhabitationRequestKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
}

pub struct SourceModePropertyInput {
    pub owner: SourceModeDefinitionId,
    pub ordinal: usize,
    pub kind: SourceModePropertyKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub justification: SourceAnchor,
    pub recovery: SourceModeDefinitionRecovery,
    pub spelling: String,
}

#[non_exhaustive]
pub enum SourceModeInhabitationRequestKind { Rhs }

#[non_exhaustive]
pub enum SourceModePropertyKind { Sethood }

#[non_exhaustive]
pub enum SourceModeDefinitionRecovery { Normal, Degraded }
```

All input structs derive `Debug + Clone + PartialEq + Eq`. Recovery, request,
and property enums derive `Debug + Clone + Copy + PartialEq + Eq + PartialOrd
+ Ord + Hash`. No public input accepts a `SemanticOrigin`, fingerprint,
allocated dense ID, `InitialObligationId`, result/evidence status, accepted
fact, formula, proof, or VC.

## Exact Immutable Output And Public API

The immutable row type names and stored fields, in API order, are:

| Row | Stored fields |
| --- | --- |
| `SourceModeDefinition` | `id`, `symbol`, `definition`, `contribution`, `site`, `source_range`, `source_ordinal`, `context`, `recovery`, `spelling`, `application`, `expansion`, `inhabitation_request`, `property`, derived `origin` |
| `SourceModeParameter` | `id`, `owner`, `ordinal`, `binding`, `written_type`, `site`, `source_range`, `declaration_range`, `pattern_range`, `context`, `recovery`, `spelling` |
| `SourceModeApplication` | `id`, `owner`, `ordinal`, `parameters`, `site`, `source_range`, `context`, `recovery`, `spelling` |
| `SourceModeExpansion` | `id`, `owner`, `ordinal`, `rhs`, `site`, `source_range`, `context`, `recovery`, `spelling` |
| `SourceModeInhabitationRequest` | `id`, `owner`, `ordinal`, `expansion`, `kind`, `site`, `source_range`, `context`, `recovery`, `spelling` |
| `SourceModeProperty` | `id`, `owner`, `ordinal`, `kind`, `site`, `source_range`, `justification`, `recovery`, `spelling`, derived `obligation` |

Every stored field has one same-named read-only getter. Copy IDs/enums/ranges/
ordinals/contexts and the optional property return by value; `parameters()`
returns `&[SourceModeParameterId]`; symbol, site, origin, and justification
return shared references; `spelling()` returns `&str`. There are no public row
constructors, setters, mutable getters, or replacement APIs.

The exact table and handoff surface is:

```rust
pub struct SourceModeDefinitionTable { /* private rows */ }
pub struct SourceModeParameterTable { /* private rows */ }
pub struct SourceModeApplicationTable { /* private rows */ }
pub struct SourceModeExpansionTable { /* private rows */ }
pub struct SourceModeInhabitationRequestTable { /* private rows */ }
pub struct SourceModePropertyTable { /* private rows */ }

pub struct SourceModeDefinitionHandoff { /* private fields */ }

impl SourceModeDefinitionHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn source_context_fingerprint(&self) -> &str;
    pub fn source_type_fingerprint(&self) -> &str;
    pub const fn base_initial_obligation_count(&self) -> usize;
    pub const fn definitions(&self) -> &SourceModeDefinitionTable;
    pub const fn parameters(&self) -> &SourceModeParameterTable;
    pub const fn applications(&self) -> &SourceModeApplicationTable;
    pub const fn expansions(&self) -> &SourceModeExpansionTable;
    pub const fn inhabitation_requests(
        &self,
    ) -> &SourceModeInhabitationRequestTable;
    pub const fn properties(&self) -> &SourceModePropertyTable;
    pub fn debug_text(&self) -> String;
}
```

Each table exposes only `get(id) -> Option<&Row>`, source-ordered
`iter() -> impl Iterator<Item = (Id, &Row)>`, `const len() -> usize`, and
`const is_empty() -> bool`. Both fingerprints are complete lower
`debug_text()` strings and cannot be caller-supplied.

The exact projection, error, and producer ABI is:

```rust
pub struct SourceModeDefinitionProjection {
    base_initial_obligations: InitialObligationTable,
    handoff: SourceModeDefinitionHandoff,
    initial_obligations: InitialObligationTable,
}

impl SourceModeDefinitionProjection {
    pub const fn base_initial_obligations(&self) -> &InitialObligationTable;
    pub const fn handoff(&self) -> &SourceModeDefinitionHandoff;
    pub const fn initial_obligations(&self) -> &InitialObligationTable;
    pub fn into_parts(self) -> (
        InitialObligationTable,
        SourceModeDefinitionHandoff,
        InitialObligationTable,
    );
}

#[non_exhaustive]
pub enum SourceModeDefinitionError {
    SourceIdentityMismatch,
    DependencyMismatch,
    InvalidResolverDefinition { index: usize },
    InvalidDefinition { index: usize },
    InvalidParameter { index: usize },
    InvalidApplication { index: usize },
    InvalidExpansion { index: usize },
    InvalidInhabitationRequest { index: usize },
    InvalidProperty { index: usize },
    InvalidObligation,
    InvalidArenaOwnership,
    UnsupportedTaskShape,
}

pub struct SourceModeDefinitionProducer;

impl SourceModeDefinitionProducer {
    pub fn build(
        input: SourceModeDefinitionHandoffInput,
        env: &SymbolEnv,
        source_context: &SourceBindingContextHandoff,
        source_type: &SourceTypeApplicationHandoff,
        base_initial_obligations: &InitialObligationTable,
        arena: &TypedArena,
    ) -> Result<SourceModeDefinitionProjection, SourceModeDefinitionError>;
}
```

The output rows, tables, handoff, projection, and error derive
`Debug + Clone + PartialEq + Eq`; the error implements `Display` and
`std::error::Error` and has no `Default` or blanket conversion. The producer is
a unit struct.

## Public Enum Policy

| Public enum | Compatibility policy |
| --- | --- |
| `SourceModeInhabitationRequestKind` | `#[non_exhaustive]`; later request kinds require a separately frozen owner. |
| `SourceModePropertyKind` | `#[non_exhaustive]`; later mode properties require canonical authority and tests. |
| `SourceModeDefinitionRecovery` | `#[non_exhaustive]`; callers tolerate later recovery classes. |
| `SourceModeDefinitionError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Exact Active Rows And Cardinalities

The active transaction is exactly `1/2/1/1/1/1`:

| Table | Exact active row |
| --- | --- |
| definition 0 | resolver symbol/definition/contribution 0; site node 49; range `45..135`; source ordinal 0; context 1; spelling bytes `mode Task262ModeDefinition: Task262Mode [x, y] is set;\n  sethood by computation(steps: 1);`; links application/expansion/request 0 and `Some(property 0)` |
| parameters 0/1 | owners 0, ordinals 0/1, bindings 0/1, written types 0/1, sites 37/41, ranges `13..26`/`29..42`, declaration ranges `17..18`/`33..34`, pattern ranges `86..87`/`89..90`, context 1, spellings `let x be set;`/`let y be set;` |
| application 0 | owner 0, ordinal 0, parameter IDs `[0,1]`, site 42, range `73..91`, context 1, spelling `Task262Mode [ x , y ]` |
| expansion 0 | owner 0, ordinal 0, RHS mode-type row 0 / expression root 2, site 44, range `95..98`, context 1, spelling `set` |
| request 0 | owner 0, ordinal 0, expansion 0, kind `Rhs`, site 44, range `95..98`, context 1, spelling `set` |
| property 0 | owner 0, ordinal 0, kind `Sethood`, site 48, range `102..135`, justification node/range `46/113..134`, spelling `sethood by computation(steps: 1);` |

Immutable output rows copy these fields, add dense IDs, and add the resolver-
derived `SemanticOrigin` only to the definition. The property output also owns
the derived `InitialObligationId`. Tables expose only `len`, `is_empty`, `get`,
and ordered `iter`; rows expose read-only accessors. The handoff stores source
and module identity, resolver identity, the two lower fingerprints, and the
six tables. Input order is canonical and is never sorted or repaired.

The projection retains an exact clone of the baseline, the handoff, and the
updated obligation table so TypedAst can authenticate compare-and-swap.

## Inhabitation Request And Initial Obligation Boundary

Chapter 7 makes RHS inhabitation mandatory, but this task has no accepted
evidence response or base-shape evaluator. Request 0 therefore records only
that expansion 0 requires RHS inhabitation evidence. It has no result,
availability, witness, diagnostic, or acceptance field. In particular, the
source spelling `set` does not authorize Task 262 to claim that the Chapter-17
base-shape table has been consulted or that the definition is accepted.

Let `b` be the exact baseline length. The projection stores a byte-identical
clone of that baseline, preserves every row and ID in `[0,b)` in order, and
appends exactly one row at `InitialObligationId(b)`. Arbitrary unrelated
baseline obligations, including ordinary existing-kind `Sethood` rows, are
allowed and preserved. Baseline rows of the sibling-only kinds
`PredicatePropertyCorrectness`, `FunctorExistence`, or `FunctorUniqueness` are
rejected because Task 262 cannot coexist with Tasks 259/260. Property 0 links
only to the appended ID:

| Field | Exact value |
| --- | --- |
| `id` | `InitialObligationId(b)` |
| `kind` | existing `InitialObligationKind::Sethood` |
| `owner` | property site node 48 |
| `source_range` | `102..135` |
| `assumptions` | empty |
| `goal` | `source.definition.mode.correctness:definition=0:sethood` |
| `provenance` | `source.definition.mode:definition=0:property=0` |
| `status` | `InitialObligationStatus::Pending` |

The empty assumption vector is representation-only. It does not state the
unguarded FOL obligation. Chapter 7's `ParamGuard` construction, quantified
goal, witness-set dependence, proof checking, computation execution,
discharge, acceptance, and exported/private semantic-fact behavior are all
deferred. The justification anchor is provenance only. No separate
mode-existence `InitialObligationKind` is invented: the mandatory existence
check is represented solely by request 0 until an authorized evidence consumer
exists.

Updated-table length must be exactly `b + 1`. Any suffix row beyond ID `b`,
any property link outside ID `b`, or any second Task-262 goal/provenance row is
invalid even if its kind is `Sethood`. Final validation is link- and prefix-
based rather than kind-wide: with a handoff it validates exactly property 0 ->
ID `b` and the single exact suffix while preserving unrelated baseline
`Sethood`; without a handoff it rejects an orphan row whose goal or provenance
uses the `source.definition.mode` domain. Thus existing general `Sethood`
obligations are neither claimed nor rejected by kind alone.

## Validation, Determinism, And Failure Atomicity

Validation rejects a wrong source/module/arena, non-dense or wrong
cardinality, missing/duplicate/reordered/dangling/cross-owner/cross-context
row, wrong binding/type/application/expansion/request/property association,
wrong site/range/ordinal/spelling/kind, recovered/degraded row, stale resolver
symbol/definition/contribution/origin, stale lower fingerprint, wrong baseline,
pre-existing appended row, wrong obligation owner/range/kind/text/status,
partial result, or extra row. It also rejects any parameter vector other than
`[0,1]` and any RHS other than Task-249M mode-RHS row 0 / expression root 2
for this exact shape.

Production authenticates the complete input before allocating output. Errors
return no partial handoff and do not mutate the input baseline, lower handoffs,
environment, or arena. `debug_text()` starts with
`source-mode-definition-debug-v1`, renders both fingerprints and all six
tables in ID order, includes the linked obligation ID, and ends with one LF.
Repeated build, clone, typed install, resolved assembly, and rendering are
byte-deterministic. Empty legacy debug output remains byte-identical.

The exact no-blank-line debug grammar is below. `Rust-debug` means standard
escaped `{:?}` output; all active sites are node sites and all rows are normal:

```text
source-mode-definition-debug-v1
module: <ModuleId.path>
source-context-fingerprint: <Rust-debug String>
source-type-fingerprint: <Rust-debug String>
base-initial-obligation-count: <n>
definition#<id> symbol=<Rust-debug FQN string> definition=<id> contribution=<id> ordinal=<n> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> origin_range=<start>..<end> origin_path=<Rust-debug [u32]> spelling=<Rust-debug String> application=<id> expansion=<id> inhabitation_request=<id> property=<none|id>
parameter#<id> owner=<id> ordinal=<n> binding=<id> written_type=<id> range=<start>..<end> declaration_range=<start>..<end> pattern_range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
application#<id> owner=<id> ordinal=<n> parameters=<Rust-debug [usize]> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
expansion#<id> owner=<id> ordinal=<n> rhs=<id> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
inhabitation-request#<id> owner=<id> ordinal=<n> expansion=<id> kind=<rhs> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
property#<id> owner=<id> ordinal=<n> kind=<sethood> range=<start>..<end> site=node#<id> justification=range:<start>..<end> recovery=<normal|degraded> spelling=<Rust-debug String> obligation=<id>
```

## Typed And Final Ownership

`TypedAst` adds one optional field and one one-shot installer:

```rust
pub fn with_source_mode_definition(
    self,
    projection: SourceModeDefinitionProjection,
) -> Result<Self, TypedAstError>;

pub const fn source_mode_definition(
    &self,
) -> Option<&SourceModeDefinitionHandoff>;

TypedAstError::InvalidSourceModeDefinition
```

The installer requires source context and source type, compares its current
obligation table with the projection baseline, validates the handoff and
single appended row, rejects prior Task-262 occupancy, and publishes the
handoff plus updated obligations only after all checks pass. It rejects an AST
already carrying Task 259, 260, or 261. `TypedAstParts` gains no Task-262 field
or alternate installation path.

`ResolvedTypedAst::assemble` obtains Task 262 only from the typed owner,
clone-preserves and revalidates it against the final lower handoffs and
obligations, and adds only:

```rust
pub const fn source_mode_definition(
    &self,
) -> Option<&SourceModeDefinitionHandoff>;

ResolvedTypedAstError::InvalidSourceModeDefinition
```

`ResolvedTypedAstInputs` gains no replaceable field. Final assembly rejects
every mixed Task-259/260/261/262 definition-family state, including the reverse
install order that an older sibling installer cannot know. Task 262 does not
weaken, merge, or reinterpret Task 259's predicate correctness, Task 260's
functor existence/uniqueness, or Task 261's no-obligation boundary.

## Dedicated Runner Consumer And Trace Intent

Implementation adds exactly one active pass pair:

- `tests/miz/pass/types/pass_type_elaboration_mode_definition_payload_001.miz`;
- `tests/miz/pass/types/pass_type_elaboration_mode_definition_payload_001.expect.toml`.

The sidecar is `pass` / `type_elaboration` / `type_check`, has empty public
diagnostics and payloads, and cites only
`spec.en.checker.type_elaboration.source_mode_definition_payload`. One new
required covered trace row reciprocally cites only that sidecar. This is a
transport pass: it means exact payload production completed without a public
diagnostic, not that the mode, RHS evidence, computation, sethood proof, or
interface fact was accepted.

The private runner route must authenticate the exact source/hash, all 54
Surface rows, both shells, resolver projection/symbol/definition/contribution,
Tasks-248/249 profiles and fingerprints, then call the checker. It must be
selected before the generic mixed mode/structure gap. The existing
`fail_type_elaboration_mode_structure_definition_gap_001.miz`, sidecar, and
trace row remain byte-identical because its structure half belongs to Task 263.
Parser pass/recovery fixtures also remain unchanged.

The frozen checker tests are exactly:

1. `task_262_mode_definition_exact_payload_and_obligations_are_deterministic`;
2. `task_262_mode_definition_row_field_corruption_fails_closed`;
3. `task_262_mode_definition_dependency_and_obligation_corruption_fails_closed`;
4. `task_262_mode_definition_typed_installation_is_transactional`; and
5. `task_262_mode_definition_final_clone_debug_determinism_and_family_isolation`.

The frozen runner tests are exactly:

1. `task262_mode_definition_source_consumer_is_exact`;
2. `task262_mode_definition_surface_resolver_lower_and_payload_corruption_fail_closed`;
3. `task262_mode_definition_selection_and_family_isolation_are_exact`; and
4. `task262_mode_definition_justification_and_semantic_subtrees_are_not_published`.

Test mutations must independently cover literal bytes/final LF, every
structural table, resolver origin/identity, parameter application order,
lower fingerprints, request/property associations, obligation baseline and
appended row, unrelated-baseline-`Sethood` preservation, sibling-kind rejection,
linked-suffix/orphan rejection, typed/final transactional ownership, immutable clones, debug
determinism, sibling-family isolation, route isolation, and non-publication.

## Count, Hash, Audit, And Write Scope

The documentation prerequisite changes no Rust, fixture, sidecar,
expectation, trace row/status/backlink/count, test-list entry, production path,
Cargo metadata, CLI output, or recorded hash. Current executable baselines
remain checker/runner/resolver/syntax `449/520/144/59`, metadata
cases/requirements `424/392`, pass/fail `231/193`, active
parse/declaration/type/proof `101/7/201/1`, type requirements `256/244`, and
warnings/errors `23/0`.

The separate Task-249M implementation projects checker `449 -> 453` with four
checker tests and no corpus/runner delta. Task-262 implementation then projects
checker `453 -> 458`, runner `520 -> 524`, one new
case/requirement/pass/active-type/covered-type row, metadata `425/393`,
`232/193`, `101/7/202/1`, and type `257/245`; resolver/syntax remain
`144/59`. Production manifests, test-list hashes, five CLI hashes, fixture,
sidecar, and trace hashes must be fresh-measured rather than predicted.

The later implementation write scope is limited to:

- checker mode-definition module, exports, TypedAst/final ownership, lint and
  source-spec inventories, and the five frozen checker tests;
- one private runner leaf and test leaf plus bounded facade/root registration;
- the one new pass source/sidecar and one reciprocal covered trace row; and
- synchronized EN/JA plan/TODO/ledger/module/source/trace/spec-coverage audits.

No `doc/spec`, existing `.miz`, existing expectation/sidecar, parser/resolver
production, lower checker producer, Task-249M behavior, Task-259/260/261
semantic behavior, Core, VC, kernel, or unrelated metadata may change.

## Explicit Semantic Deferrals And Exit Criteria

Deferred semantics are RHS evidence lookup/response, base-shape-table result,
attribute-chain inhabitation, registration order/activation, definition
acceptance/symbol activation, mode-application checking at use sites,
expansion facts and normalization, `ParamGuard` construction, quantified
existence/sethood FOL, computation/proof/discharge, witness handling,
exported/private sethood facts, property implementation/coherence, facts,
axioms, CoreIr, ControlFlowIr, VC, and every mixed definition-family meaning.
Task 263 owns structure definitions; Task 264 owns property implementations.

Task 262 is complete only when:

- the prerequisite contract and all synchronized audits pass repeated
  review-only specification review with no findings and are committed alone;
- separate Task-249M documentation and implementation commits pass their own
  reviews, hard gates, verification, staging, and post-commit inventory;
- fresh post-Task-249M inventory confirms the exact Task-262 source is then
  dependency-ready without any further lower-stage change;
- implementation matches every row, association, request, obligation,
  fingerprint, ownership, isolation, consumer, mutation, and exclusion above;
- separate test-sufficiency, implementation, and source/documentation reviews
  end with no findings;
- all nine protocol hard gates pass with no score cap and final read-only
  quality at least 90/100;
- focused/crate/library/metadata/lint/fmt/Clippy/workspace/CLI/count/hash/
  whitespace verification passes; and
- only Task-262 files are staged and committed, then clean HEAD/origin/stash
  inventory returns directly to dependency-ordered Task 263.

## Task 249M Lower-Contract Link

The upper contract is committed as `8c3fa20acef42477d38a66ddddec42dacced0863`.
The exact lower ABI, error precedence, debug grammar, `2/3/0/0/1` profile, and
four-test matrix are now frozen canonically in
[`source_type.md`](./source_type.md), section “Task 249M Frozen Standalone
Mode-RHS Extension.” This document grants no Task-262 implementation authority
until that separate docs prerequisite and implementation both commit and a
fresh inventory confirms the fingerprint seam.

## Task 249M Implemented Lower-Contract Link

The lower standalone mode-RHS ABI and exact fingerprint are now implemented
and checker-tested. Task 262 may consume only `mode_rhs().get(0)` and expression
root 2 after fresh inventory; this completion grants no authority to invent
goal/guard composition, discharge, acceptance, fact, proof, IR, or VC behavior.

## Task 262 Active Implementation Result

The frozen six-table producer is active at `1/2/1/1/1/1`. It authenticates
the exact source/resolver identity and Task-248/249/249M handoffs, retains
their fingerprints, leaves the RHS request unresolved, and appends exactly
one Pending `Sethood` obligation to the authenticated baseline. Typed and
resolved owners publish only immutable clones and reject mixed Tasks
259--262, stale dependencies, invalid links, and orphan mode-domain rows.

The exact pass pair and sole reciprocal covered trace row are active. The five
checker and four runner tests preserve the frozen corruption, transactional,
final-assembly, isolation, and non-publication matrix. Active counts are
checker/runner/resolver/syntax `458/524/144/59`, metadata `425/393`, pass/fail
`232/193`, active stages `101/7/202/1`, type coverage `257/245`, and
warnings/errors `23/0`. All semantic deferrals above remain unchanged; Task
263 is selected only after the dedicated commit and fresh inventory.
