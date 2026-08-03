# Source Structure-Definition Transport

> Canonical language: English. Japanese companion:
> [../ja/source_structure_definition.md](../ja/source_structure_definition.md).

## Task 263 Scope And Authority

Checker Task 263 owns one syntax-free immutable intake for two ordinary
zero-parameter structure declarations, their field/property selector
declarations, one direct inheritance edge, exact member coverage, fields-only
constructor order, and the resulting root/path/view member associations.
Canonical authority is `doc/spec/en/05.structures.md` Sections 5.1--5.8;
`doc/spec/en/13.term_expression.md` Sections 13.3--13.3.3 only for the
constructor/selector declaration boundary; `doc/spec/en/16.theorems_and_proofs.md`
Sections 16.6--16.6.4 only for the definition-correctness/initial-obligation
boundary; and `doc/spec/en/19.overload_resolution.md` Section 19.2.2 only for
the inheritance-path/upcast-viability boundary, with winner selection and
implicit conversion deferred. Declaration identity/provenance is grounded in
Chapter 5 plus the committed Task-263R resolver supporting authority. Existing
structure parser pass/recovery fixtures, the active mixed
mode/structure-definition gap and its
sidecar/trace rows, and the committed public Tasks 248--262, Task 263R, and
Task 249S transports are supporting authorities in repository authority order.

Fresh inventory classifies the absent upper structure producer and executable
consumer as `source_drift` and a canonical-derived `test_gap`; the missing
frozen contract is `design_drift`. There is no blocking `spec_gap`. Task 263R
already closed the false cross-structure selector-conflict `source_drift`, and
Task 249S already closed the standalone member-type `source_drift`. Their
separate committed public APIs are prerequisites, not Task-263 write scope.

Task 263 transports authenticated declaration shape. It does not accept a
structure, constructor, selector, inheritance edge, or redefinition as
semantically valid; choose an upcast; synthesize a property constructor
argument; infer a member name; compose a guard or logical goal; append an
obligation for identical member types; discharge a proof; publish a fact or
axiom; or lower Core, control-flow, or VC payloads.

## Frozen Exact Source

The future active source is exactly, including the final LF:

```mizar
definition
  struct Task263Base where
    field carrier -> set;
    property marker -> set;
  end;

  struct Task263Derived where
    field carrier -> set;
    property marker -> set;
  end;

  inherit Task263Derived extends Task263Base where
    field carrier from carrier;
    property marker from marker;
  end;
end;
```

It is 320 bytes, 16 lines, and has SHA-256
`078eaee4b17341c9d8ebeb8a1f631ca984873bd07eb4e5d9c1a9486b39ac6671`.
It contains one normal definition block, two zero-parameter structure
definitions, four independently written bare builtin-`set` member types, one
direct inheritance declaration, and two explicit same-spelling member
mappings. The source has no `let`, reserve/default parameter, binding context,
inherited parameter, renamed member, narrowed type, default field, second
parent, diamond, cycle, recovery, constructor term, selector application,
functional update, property implementation, correctness clause, theorem,
proof, or diagnostic.

Parameters and contexts are frozen as exactly absent. Task 263 receives no
`SourceBindingContextHandoff`, stores no `BindingId` or `BindingContextId`, and
publishes no positive parameter row. Any parameterized structure profile is a
separate future contract; fabricating a context for this source is a
`boundary_violation`.

## Frozen Literal Surface Oracle

The frontend emits zero diagnostics and exactly 75 dense Surface rows. Root
node 74 covers `0..319` and is normal. Rows 0--49 are leaf tokens:

| Node | Token | Range | Node | Token | Range |
| ---: | --- | --- | ---: | --- | --- |
| 0 | `definition` | `0..10` | 25 | `marker` | `169..175` |
| 1 | `struct` | `13..19` | 26 | `->` | `176..178` |
| 2 | `Task263Base` | `20..31` | 27 | `set` | `179..182` |
| 3 | `where` | `32..37` | 28 | `;` | `182..183` |
| 4 | `field` | `42..47` | 29 | `end` | `186..189` |
| 5 | `carrier` | `48..55` | 30 | `;` | `189..190` |
| 6 | `->` | `56..58` | 31 | `inherit` | `194..201` |
| 7 | `set` | `59..62` | 32 | `Task263Derived` | `202..216` |
| 8 | `;` | `62..63` | 33 | `extends` | `217..224` |
| 9 | `property` | `68..76` | 34 | `Task263Base` | `225..236` |
| 10 | `marker` | `77..83` | 35 | `where` | `237..242` |
| 11 | `->` | `84..86` | 36 | `field` | `247..252` |
| 12 | `set` | `87..90` | 37 | `carrier` | `253..260` |
| 13 | `;` | `90..91` | 38 | `from` | `261..265` |
| 14 | `end` | `94..97` | 39 | `carrier` | `266..273` |
| 15 | `;` | `97..98` | 40 | `;` | `273..274` |
| 16 | `struct` | `102..108` | 41 | `property` | `279..287` |
| 17 | `Task263Derived` | `109..123` | 42 | `marker` | `288..294` |
| 18 | `where` | `124..129` | 43 | `from` | `295..299` |
| 19 | `field` | `134..139` | 44 | `marker` | `300..306` |
| 20 | `carrier` | `140..147` | 45 | `;` | `306..307` |
| 21 | `->` | `148..150` | 46 | `end` | `310..313` |
| 22 | `set` | `151..154` | 47 | `;` | `313..314` |
| 23 | `;` | `154..155` | 48 | `end` | `315..318` |
| 24 | `property` | `160..168` | 49 | `;` | `318..319` |

Rows 50--74 are exactly:

| Node | Surface kind | Range | Ordered children |
| ---: | --- | --- | --- |
| 50 | `StructurePattern` | `20..31` | `[2]` |
| 51 | `TypeHead` | `59..62` | `[7]` |
| 52 | `TypeExpression` | `59..62` | `[51]` |
| 53 | `StructureField` | `42..63` | `[4,5,6,52,8]` |
| 54 | `TypeHead` | `87..90` | `[12]` |
| 55 | `TypeExpression` | `87..90` | `[54]` |
| 56 | `StructureProperty` | `68..91` | `[9,10,11,55,13]` |
| 57 | `StructureDefinition` | `13..98` | `[1,50,3,53,56,14,15]` |
| 58 | `StructurePattern` | `109..123` | `[17]` |
| 59 | `TypeHead` | `151..154` | `[22]` |
| 60 | `TypeExpression` | `151..154` | `[59]` |
| 61 | `StructureField` | `134..155` | `[19,20,21,60,23]` |
| 62 | `TypeHead` | `179..182` | `[27]` |
| 63 | `TypeExpression` | `179..182` | `[62]` |
| 64 | `StructureProperty` | `160..183` | `[24,25,26,63,28]` |
| 65 | `StructureDefinition` | `102..190` | `[16,58,18,61,64,29,30]` |
| 66 | `InheritanceTarget` | `202..216` | `[32]` |
| 67 | `InheritanceTarget` | `225..236` | `[34]` |
| 68 | `FieldRedefinition` | `247..274` | `[36,37,38,39,40]` |
| 69 | `PropertyRedefinition` | `279..307` | `[41,42,43,44,45]` |
| 70 | `InheritanceDefinition` | `194..314` | `[31,66,33,67,35,68,69,46,47]` |
| 71 | `DefinitionBlockItem` | `0..319` | `[0,57,65,70,48,49]` |
| 72 | `ItemList` | `0..319` | `[71]` |
| 73 | `CompilationUnit` | `0..319` | `[72]` |
| 74 | `Root` | `0..319` | tokens 0--49 followed by `[73]` |

The private runner authenticates every byte, final LF, row kind, range,
recovery state, ordered child list, root, and direct sibling order before it
constructs syntax-free input. Checker production receives no parser type,
source text, raw token, Surface kind, or Surface node number.

## Frozen Resolver Provenance

The corrected Task-263R resolver result is exactly ten shells, eight
projections, eight symbols, eight definitions, one local-source contribution,
and zero resolver diagnostics. Shells in order are block 71; structure 57;
field 53; property 56; structure 65; field 61; property 64; inheritance 70;
field redefinition 68; and property redefinition 69. Their source ordinals are
0--9, with both structures parented by the block, both declaration members by
their nearest structure, and both redefinitions by the inheritance shell.

The exact resolver definition IDs are:

| Role | Symbol kind / definition kind | Definition ID | Range | Structural path |
| --- | --- | ---: | --- | --- |
| base structure | `Structure` / `Structure` | 0 | `13..98` | `[4,0,11,0]` |
| base `carrier` | `Selector` / `Selector` | 1 | `42..63` | `[4,0,11,0,18,0]` |
| base `marker` | `Selector` / `Selector` | 2 | `68..91` | `[4,0,11,0,19,1]` |
| derived structure | `Structure` / `Structure` | 3 | `102..190` | `[4,0,11,1]` |
| derived `carrier` | `Selector` / `Selector` | 4 | `134..155` | `[4,0,11,1,18,0]` |
| derived `marker` | `Selector` / `Selector` | 5 | `160..183` | `[4,0,11,1,19,1]` |
| field mapping | `Redefinition` / `Redefinition` | 6 | `247..274` | `[4,0,20,2,21,0]` |
| property mapping | `Redefinition` / `Redefinition` | 7 | `279..307` | `[4,0,20,2,22,1]` |

Every row is normal, local, public/exported, non-overloadable,
conflict-free, and contribution 0 owns all eight symbols and definitions.
Structure spelling is `Task263Base`/`Task263Derived`; declaration selector and
redefinition spelling is `carrier`/`marker`. The checker authenticates exact
symbol/definition/contribution/origin pairs and never reconstructs owner or
member identity from FQN text or opaque resolver signatures.

## Frozen Lower Bundle

Only the Task-249 plus Task-249S `SourceTypeApplicationHandoff` is present. Its
applications/expressions/arguments/definition returns/mode RHS/structure
members profile is exactly `0/4/0/0/0/4`. Structure-member IDs 0--3 own nodes
53/56/61/64 and expression roots 0--3 at nodes 52/55/60/63; every expression
is argument-free, bare, normal builtin `set`. Task 263 fingerprints the entire
lower `debug_text()` and links declaration members only by
`SourceTypeStructureMemberId`.

Task 248 is exactly absent because the source has no parameters or context.
Tasks 249R and 249M are absent inside the same source-type handoff. Every lower
term/application/structure-term/set/formula/evidence and Tasks 259--262
definition-family handoff is absent. The runner may compose only the existing
Task-249S producer; it may not edit a lower producer or fabricate a binding,
context, application, return row, mode-RHS row, member type, resolver relation,
or semantic result.

## Exact Public Syntax-Free Input

Implementation adds `source_structure_definition.rs` with five dense ID
families. IDs derive `Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord
+ Hash`, expose only `new` and `index`, and are allocated by vector order:

```rust
pub struct SourceStructureDefinitionId(usize);
pub struct SourceStructureMemberId(usize);
pub struct SourceStructureInheritanceId(usize);
pub struct SourceStructureMappingId(usize);
pub struct SourceStructureCoherenceRequestId(usize);
```

The exact public caller input is:

```rust
pub struct SourceStructureDefinitionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub definitions: Vec<SourceStructureDefinitionInput>,
    pub members: Vec<SourceStructureMemberInput>,
    pub inheritances: Vec<SourceStructureInheritanceInput>,
    pub mappings: Vec<SourceStructureMappingInput>,
}

pub struct SourceStructureDefinitionInput {
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub recovery: SourceStructureDefinitionRecovery,
    pub spelling: String,
    pub members: Vec<SourceStructureMemberId>,
    pub constructor_fields: Vec<SourceStructureMemberId>,
}

pub struct SourceStructureMemberInput {
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub owner: SourceStructureDefinitionId,
    pub ordinal: usize,
    pub kind: SourceStructureMemberKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub recovery: SourceStructureDefinitionRecovery,
    pub spelling: String,
    pub written_type: SourceTypeStructureMemberId,
    pub constructor_ordinal: Option<usize>,
}

pub struct SourceStructureInheritanceInput {
    pub child: SourceStructureDefinitionId,
    pub parent: SourceStructureDefinitionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub recovery: SourceStructureDefinitionRecovery,
    pub spelling: String,
    pub mappings: Vec<SourceStructureMappingId>,
}

pub struct SourceStructureMappingInput {
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub inheritance: SourceStructureInheritanceId,
    pub ordinal: usize,
    pub kind: SourceStructureMemberKind,
    pub view_member: SourceStructureMemberId,
    pub parent_member: SourceStructureMemberId,
    pub root_member: SourceStructureMemberId,
    pub path: Vec<SourceStructureInheritanceId>,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub recovery: SourceStructureDefinitionRecovery,
    pub spelling: String,
}

#[non_exhaustive]
pub enum SourceStructureMemberKind { Field, Property }

#[non_exhaustive]
pub enum SourceStructureCoherenceRequestKind { MemberTypeInclusion }

#[non_exhaustive]
pub enum SourceStructureDefinitionRecovery { Normal, Degraded }
```

All input structs derive `Debug + Clone + PartialEq + Eq`. The enums derive
`Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + Hash`. The caller
does not supply parameter/context rows, a coherence request, an obligation ID,
type-relation verdict, chosen path, semantic origin, fingerprint, fact,
acceptance, proof, or diagnostic. `path` is authenticated source association,
not a resolver-selected upcast winner.

## Exact Immutable Output And Producer ABI

The four caller-backed immutable rows add dense IDs and resolver-derived
origins where applicable. Their stored fields otherwise match input names.
`SourceStructureDefinition` and `SourceStructureMember` add `origin`;
`SourceStructureMapping` adds `origin`; `SourceStructureInheritance` has no
resolver identity or origin. Each stored field has one same-named read-only
getter. Slices return shared slices, strings return `&str`, symbols/sites/
origins return references, and Copy values return by value.

The producer derives an immutable fifth table:

```rust
pub struct SourceStructureCoherenceRequest {
    /* private id, mapping, kind, site, source_range */
}
```

Its read-only getters are `id`, `mapping`, `kind`, `site`, and
`source_range`. `MemberTypeInclusion` records only the Chapter-5 request that a
mapped child member type be included in its parent/root member type. It has no
assumptions, goal, evidence, status, obligation ID, proof, or result. The exact
Task-263 profile derives zero rows because both mapped pairs have identical
authenticated bare builtin-`set` types.

Each of the five tables exposes only `get`, source-ordered `iter`, `len`, and
`is_empty`. The handoff surface is:

```rust
pub struct SourceStructureDefinitionHandoff { /* private fields */ }

impl SourceStructureDefinitionHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn source_type_fingerprint(&self) -> &str;
    pub const fn base_initial_obligation_count(&self) -> usize;
    pub const fn definitions(&self) -> &SourceStructureDefinitionTable;
    pub const fn members(&self) -> &SourceStructureMemberTable;
    pub const fn inheritances(&self) -> &SourceStructureInheritanceTable;
    pub const fn mappings(&self) -> &SourceStructureMappingTable;
    pub const fn coherence_requests(&self) -> &SourceStructureCoherenceRequestTable;
    pub fn debug_text(&self) -> String;
}
```

The handoff's private state also contains an immutable
`base_initial_obligations_snapshot: InitialObligationTable`. It has no public
getter and is not caller-supplied. The producer clones it from the authenticated
baseline before validation completes. The public
`base_initial_obligation_count()` is a compact cardinality invariant, not a
substitute for the snapshot.

The exact projection/error/producer ABI is:

```rust
pub struct SourceStructureDefinitionProjection {
    base_initial_obligations: InitialObligationTable,
    handoff: SourceStructureDefinitionHandoff,
    initial_obligations: InitialObligationTable,
}

impl SourceStructureDefinitionProjection {
    pub const fn base_initial_obligations(&self) -> &InitialObligationTable;
    pub const fn handoff(&self) -> &SourceStructureDefinitionHandoff;
    pub const fn initial_obligations(&self) -> &InitialObligationTable;
    pub fn into_parts(self) -> (
        InitialObligationTable,
        SourceStructureDefinitionHandoff,
        InitialObligationTable,
    );
}

#[non_exhaustive]
pub enum SourceStructureDefinitionError {
    SourceIdentityMismatch,
    DependencyMismatch,
    InvalidResolverDefinition { index: usize },
    InvalidDefinition { index: usize },
    InvalidMember { index: usize },
    InvalidInheritance { index: usize },
    InvalidMapping { index: usize },
    InvalidCoherenceRequest { index: usize },
    InvalidObligation,
    InvalidArenaOwnership,
    UnsupportedTaskShape,
}

pub struct SourceStructureDefinitionProducer;

impl SourceStructureDefinitionProducer {
    pub fn build(
        input: SourceStructureDefinitionHandoffInput,
        env: &SymbolEnv,
        source_type: &SourceTypeApplicationHandoff,
        base_initial_obligations: &InitialObligationTable,
        arena: &TypedArena,
    ) -> Result<SourceStructureDefinitionProjection, SourceStructureDefinitionError>;
}
```

Output rows/tables/handoff/projection/error derive `Debug + Clone + PartialEq
+ Eq`; error implements `Display` and `Error`. No new
`InitialObligationKind`, public diagnostic, public exhaustive-enum exception,
mutable getter, setter, row constructor, or replacement API is authorized.

## Public Enum Policy

| Public enum | Compatibility policy |
| --- | --- |
| `SourceStructureMemberKind` | `#[non_exhaustive]`; later member classes require separate canonical authority and tests. |
| `SourceStructureCoherenceRequestKind` | `#[non_exhaustive]`; later coherence-request classes require a separately frozen semantic owner. |
| `SourceStructureDefinitionRecovery` | `#[non_exhaustive]`; callers tolerate later recovery classes. |
| `SourceStructureDefinitionError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Exact Active Rows And Associations

The immutable profile is exactly `2/4/1/2/0`:

| Table | Exact row |
| --- | --- |
| definition 0 | base structure resolver definition 0; site 57, `13..98`; ordinal 0; members `[0,1]`; constructor fields `[0]`; exact source spelling |
| definition 1 | derived structure resolver definition 3; site 65, `102..190`; ordinal 1; members `[2,3]`; constructor fields `[2]`; exact source spelling |
| member 0 | base `carrier`, resolver definition 1; owner 0 ordinal 0; `Field`; site 53, `42..63`; spelling `field carrier -> set;`; written type 0; constructor ordinal `Some(0)` |
| member 1 | base `marker`, resolver definition 2; owner 0 ordinal 1; `Property`; site 56, `68..91`; spelling `property marker -> set;`; written type 1; constructor ordinal `None` |
| member 2 | derived `carrier`, resolver definition 4; owner 1 ordinal 0; `Field`; site 61, `134..155`; spelling `field carrier -> set;`; written type 2; constructor ordinal `Some(0)` |
| member 3 | derived `marker`, resolver definition 5; owner 1 ordinal 1; `Property`; site 64, `160..183`; spelling `property marker -> set;`; written type 3; constructor ordinal `None` |
| inheritance 0 | child 1, parent 0; site 70, `194..314`; ordinal 0; mappings `[0,1]`; exact source spelling |
| mapping 0 | resolver definition 6; inheritance 0 ordinal 0; `Field`; view/parent/root `2/0/0`; path `[0]`; site 68, `247..274`; spelling `field carrier from carrier;` |
| mapping 1 | resolver definition 7; inheritance 0 ordinal 1; `Property`; view/parent/root `3/1/1`; path `[0]`; site 69, `279..307`; spelling `property marker from marker;` |

The exact definition/inheritance spelling bytes, without surrounding source
indentation and without an added final LF, are:

```text
struct Task263Base where\n    field carrier -> set;\n    property marker -> set;\n  end;
struct Task263Derived where\n    field carrier -> set;\n    property marker -> set;\n  end;
inherit Task263Derived extends Task263Base where\n    field carrier from carrier;\n    property marker from marker;\n  end;
```

These are three separate Rust strings, one per line above. Member/mapping
spellings are the exact single statements shown in the active-row table.

The structure symbol is the default-constructor declaration identity. Its
ordered `constructor_fields` are fields only. Every member row is a selector
declaration identity, including properties; property rows never appear in a
constructor vector. The inheritance mapping covers every parent/base member
exactly once, preserves kind and spelling, identifies the unique root and
direct path, and refers to no member outside the two endpoint structures. In
this exact source each declared child view participates in one mapping, but
that is a bounded shape check rather than a replacement for the canonical
parent-coverage rule.

The lower written-type pairs `2 -> 0` and `3 -> 1` are independently written
rows but both sides are exact bare builtin `set`. The producer compares their
authenticated lower expression form/head/spelling/arguments; it does not infer
equality from member spelling. The exact relation is identical, so Chapter 5
requires no type-inclusion proof and the coherence table is empty.

## Initial-Obligation And Semantic Boundary

Let `b` be the caller baseline length. The projection retains a byte-identical
baseline clone, stores a second immutable byte-equal snapshot plus `b` inside
the handoff, and returns a byte-identical final table of the same length. Every
baseline row and ID `[0,b)` is preserved,
including unrelated existing obligation kinds, subject to their existing
owner validators. Task 263 neither claims nor rejects a kind globally. It
appends zero rows and introduces no new kind. A build, typed install, or final
assembly that changes any baseline row even at the same length, appends any
suffix, or associates an obligation with this zero-request profile fails
closed. Existing orphan
predicate/functor/attribute/mode definition domains remain invalid under their
own absence validators; preserving bytes does not bypass that rule.

The exact runner source starts with `b = 0`, so its projection snapshot,
handoff snapshot, and final table are all empty. Checker tests additionally
use nonempty unrelated baselines to prove composition and same-length mutation
rejection. Build requires projection baseline == private handoff snapshot ==
final table. Typed install requires current table == projection baseline ==
private snapshot == final table. Final assembly requires its current complete
table == the private snapshot and rechecks `len == b`; count-only replay cannot
hide a changed row.

This unchanged table is not a claim that coherence, construction, selectors,
or inheritance are accepted. For a future nonidentical mapped type, the only
currently frozen artifact is a `MemberTypeInclusion` request row. The exact
guard, quantified goal, assumptions, facts, proof/discharge policy, diagnostic,
acceptance rule, and obligation-kind ownership require separate canonical
authority and are explicitly not invented by Task 263.

## Validation, Determinism, And Failure Atomicity

Validation rejects a wrong source/module/arena; a non-dense or wrong
cardinality; reordered/dangling/cross-owner/cross-kind row; any exact-shape or
spelling drift that would require a parameter/context representation; wrong
site/range/ordinal/spelling/recovery;
duplicate or missing member coverage; property constructor participation;
wrong field constructor order; wrong child/parent/root/path/view link; cycle,
self-edge, second edge, renamed mapping, narrowed mapping, or extra mapping;
stale resolver symbol/definition/contribution/origin/conflict; stale lower
fingerprint/member root; a type relation that would derive nonzero coherence
for this bounded task or a test-injected coherence row; changed
obligation baseline; partial output; or an unsupported subtree.

Validation uses global category order: source identity, dependency identity,
exact cardinality and task shape, all resolver rows, all definition rows, all
member rows, inheritance, mappings and coverage, lower type relation, derived
coherence, obligations, then arena ownership. All input is authenticated
before output publication; errors mutate no environment, lower handoff,
baseline, or arena and return no partial handoff.

`debug_text()` has the following exact no-blank-line grammar and ends with one
LF. `<Rust-debug String>` is `{:?}` escaping on the complete string; list
fields are `{:?}` on a `Vec<usize>` and therefore use decimal elements with a
comma followed by one space. All active sites are exact node sites, all
origins are range anchors, and all rows are normal:

```text
source-structure-definition-debug-v1
module: <ModuleId.path>
source-type-fingerprint: <Rust-debug String>
base-initial-obligation-count: <n>
profile: definitions=<n> members=<n> inheritances=<n> mappings=<n> coherence_requests=<n>
definition#<id> symbol=<Rust-debug FQN string> definition=<id> contribution=<id> ordinal=<n> range=<start>..<end> site=node#<id> recovery=<normal|degraded> origin_range=<start>..<end> origin_path=<Rust-debug [u32]> spelling=<Rust-debug String> members=<Rust-debug [usize]> constructor_fields=<Rust-debug [usize]>
member#<id> symbol=<Rust-debug FQN string> definition=<id> contribution=<id> owner=<id> ordinal=<n> kind=<field|property> range=<start>..<end> site=node#<id> recovery=<normal|degraded> origin_range=<start>..<end> origin_path=<Rust-debug [u32]> spelling=<Rust-debug String> written_type=<id> constructor_ordinal=<none|n>
inheritance#<id> child=<id> parent=<id> ordinal=<n> range=<start>..<end> site=node#<id> recovery=<normal|degraded> spelling=<Rust-debug String> mappings=<Rust-debug [usize]>
mapping#<id> symbol=<Rust-debug FQN string> definition=<id> contribution=<id> inheritance=<id> ordinal=<n> kind=<field|property> view_member=<id> parent_member=<id> root_member=<id> path=<Rust-debug [usize]> range=<start>..<end> site=node#<id> recovery=<normal|degraded> origin_range=<start>..<end> origin_path=<Rust-debug [u32]> spelling=<Rust-debug String>
coherence-request#<id> mapping=<id> kind=<member-type-inclusion> range=<start>..<end> site=node#<id>
```

For the active fixture the module line is exactly
`module: tests.miz.pass.types.pass_type_elaboration_structure_definition_payload_001`,
the profile line is exactly `profile: definitions=2 members=4 inheritances=1
mappings=2 coherence_requests=0`, and no `coherence-request#` line follows.
The source-type fingerprint is the complete Task-249S `debug_text()` rendered
as one escaped Rust-debug string, not a hash or caller-provided substitute.
Definition/member/mapping rows carry the resolver FQNs whose structural roles
and exact origins are frozen above. Rows occur in table order and then dense ID
order exactly as the grammar lists them.

Repeat build, clone, typed install, final assembly, and rendering are
byte-deterministic. Empty legacy typed/final debug output is unchanged when
Task 263 is absent.

The private obligation snapshot is deliberately not rendered by
`debug_text()`: Task 263 owns equality/authentication, not a new public
obligation serialization or disclosure surface. Its count is rendered, and
same-length row corruption is detected by private table equality rather than
debug bytes. Derived `Debug`/`Eq` remain implementation diagnostics; the
canonical stable fingerprint is `debug_text()` plus the separately validated
private snapshot.

The handoff likewise retains one private ordered snapshot of the eight
resolver `(symbol, definition, contribution)` identities authenticated during
build. Typed/final replay compares every definition/member/mapping row with
that immutable snapshot before structural validation, so a same-module symbol
substitution cannot survive after the resolver environment is no longer in
scope. This snapshot has no getter and is intentionally absent from
`debug_text()`; the complete public row identities remain rendered by the
frozen grammar above.

## Typed And Final Ownership

`TypedAst` is the sole mutable transaction owner and adds exactly:

```rust
pub fn with_source_structure_definition(
    self,
    projection: SourceStructureDefinitionProjection,
) -> Result<Self, TypedAstError>;

pub const fn source_structure_definition(
    &self,
) -> Option<&SourceStructureDefinitionHandoff>;

TypedAstError::InvalidSourceStructureDefinition
```

The installer requires the exact Task-249S source-type handoff, compares its
current obligations with the projection baseline, private handoff snapshot,
and identical final table, validates the handoff, rejects prior occupancy, and
publishes only after every check passes.
It rejects an AST carrying Task 259 predicate, Task 260 functor, Task 261
attribute, or Task 262 mode definition handoffs. Task 259 remains an
independent predicate-definition transaction; its correctness row, facts,
and mixed predicate/functor boundary are neither consumed nor changed.
`TypedAstParts` gains no replaceable Task-263 field.

`ResolvedTypedAst::assemble` obtains Task 263 only by cloning the typed owner,
revalidates the clone against the final source type and the private byte-equal
obligation snapshot,
and adds the same getter plus
`ResolvedTypedAstError::InvalidSourceStructureDefinition`. Inputs gain no
replacement path. Reverse mixed-family states also fail. Task 263 does not
change `types`, `facts`, `coercions`, diagnostics, Task-259/260 obligations,
or existing definition-family semantic outputs. Its only bounded changes to
the four existing installers are the frozen reverse-order guards that reject a
preinstalled Task-263 handoff.

## Dedicated Runner Consumer And Trace Intent

Implementation adds exactly one canonical-derived active pass pair:

- `tests/miz/pass/types/pass_type_elaboration_structure_definition_payload_001.miz`;
- `tests/miz/pass/types/pass_type_elaboration_structure_definition_payload_001.expect.toml`.

The source is the frozen 320-byte text. The sidecar is `pass` /
`type_elaboration` / `type_check`, has empty public diagnostics and payloads,
and cites only
`spec.en.checker.type_elaboration.source_structure_definition_payload`. One
new required covered trace row reciprocally cites only that sidecar. This
credits exact transport without a public diagnostic; it does not credit
acceptance, execution, proof, fact, axiom, Core, CFG, or VC semantics.

The private runner route authenticates source/hash, all 75 Surface rows, all
10 shells, all eight resolver identities, the exact Task-249S handoff and
fingerprint, subtree exclusions, and the unchanged obligation baseline before
calling the checker. It is selected before the generic mixed
mode/structure-definition gap. The existing mixed gap, every existing `.miz`,
sidecar/expectation, and parser/resolver fixture remains byte-identical.

The frozen checker tests are exactly:

1. `task_263_structure_definition_exact_payload_and_debug_are_deterministic`;
2. `task_263_structure_definition_resolver_and_row_corruption_fail_closed`;
3. `task_263_structure_definition_coverage_constructor_and_type_corruption_fail_closed`;
4. `task_263_structure_definition_obligation_and_typed_installation_are_transactional`; and
5. `task_263_structure_definition_final_clone_and_family_isolation_are_exact`.

The frozen runner tests are exactly:

1. `task263_structure_definition_source_surface_and_resolver_are_exact`;
2. `task263_structure_definition_lower_payload_and_subtree_corruption_fail_closed`;
3. `task263_structure_definition_selection_trace_and_family_isolation_are_exact`; and
4. `task263_structure_definition_semantic_deferrals_are_not_published`.

Mutations cover final LF/source bytes; every structural row; resolver
symbol/definition/contribution/origin; lower fingerprint/member association;
member kind, constructor order, exact coverage, child/parent/root/path/view;
cycle/second edge/rename/narrowing; zero coherence; unchanged arbitrary
baseline obligations; arena ownership; one-shot typed/final ownership; debug
replay; sibling-family and route isolation; and all prohibited outputs.

Within checker tests 2 and 3, failure precedence is not single-fault-only. For
each adjacent pair in the frozen 12-category order, one compound input carries
both faults and asserts that the earlier category's exact error variant/index
wins. The matrix also includes later-row higher-priority faults paired with
earlier-row lower-priority faults for resolver-vs-definition,
definition-vs-member, member-vs-mapping/coverage, mapping-vs-lower-type, and
obligation-vs-arena boundaries. Row order must never override category order.
These assertions remain inside the existing five test names and do not change
the projected checker count.

## Count, Hash, Audit, And Write Scope

This documentation prerequisite changes no Rust, fixture, sidecar,
expectation, trace row/status/backlink/count, test-list entry, production path,
Cargo metadata, CLI output, or executable hash. Baselines remain checker /
runner / resolver / syntax `462/524/146/59`; metadata cases/requirements
`425/393`; pass/fail `232/193`; active parse/declaration/type/proof
`101/7/202/1`; type requirements `257 = 245 covered + 12 deferred`; and
warnings/errors `23/0`. Checker production remains `27/156019` with path /
content hashes
`180b090a167912f0b04f014180ec6755aa5bde54eecd49f0990cc87fb566667f` /
`37a7bb07a441086ee2915f601dedbca002f9a356b53a32050c29d467eb56b9f1`.

Implementation projects checker `462 -> 467`, runner `524 -> 528`, metadata
`426/394`, pass/fail `233/193`, active stages `101/7/203/1`, and type
requirements `258 = 246 covered + 12 deferred`; resolver/syntax remain
`146/59`. All production manifests, test-list hashes, five CLI hashes, corpus
and trace hashes are fresh-measured, never predicted as exit evidence.

The implementation scope is limited to the new checker module and five tests;
public export, Typed/final one-shot ownership, source-spec and lint inventories;
cfg(test)-only predicate/functor/mode projection fixtures and predicate/mode
test-module visibility for the frozen bidirectional isolation matrix; one
private runner route/test leaf plus bounded registration and count-oracle-only
updates in four sibling test leaves; the one new pass source/sidecar and
reciprocal trace row; and synchronized EN/JA
plan/TODO/ledger/module/source/trace/spec-coverage audits. No `doc/spec`,
existing `.miz`, existing sidecar/expectation, parser/resolver production,
Task-249S lower behavior, Task-259--262 semantics or outputs beyond the frozen
Task-263 mutual-exclusion guards, public diagnostic, fact, proof, Core, CFG,
VC, Cargo dependency, or unrelated metadata may change.

## Explicit Semantic Deferrals And Exit Criteria

Deferred are all parameterized structures; inherited parameter substitution;
defaults; multiple parents, diamonds and cycles; renamed or narrowed members;
nonidentical-type coherence goal/guard/obligation ownership; path selection at
use sites; constructor, selector, and update type checking or evaluation;
extensional equality; constructor axioms; upcasts; evidence lookup; accepted
definitions or inheritance; property implementations (Task 264); definition
facts; diagnostics; proof/discharge/acceptance; CoreIr, ControlFlowIr, and VC;
and every mixed definition-family meaning.

Task 263 exits only when:

- this EN/JA frozen contract and synchronized audits pass repeated review-only
  specification review with **NO FINDINGS**, all documentation gates, an
  uncapped nine-gate score of at least 90/100, a dedicated docs-only commit,
  and clean post-commit inventory;
- fresh inventory after that commit confirms Task 263 remains dependency-ready
  with the exact parser/resolver/lower profiles and no lower-stage defect;
- implementation matches every row, association, fingerprint, zero-request,
  unchanged-obligation, ownership, isolation, consumer, mutation, and
  exclusion above without adjacent semantics;
- separate test-sufficiency, implementation, source/documentation, and final
  read-only reviews end with **NO FINDINGS**, all nine hard gates PASS without
  a score cap, and quality is at least 90/100;
- focused/crate/library/lint/metadata/fmt/Clippy/workspace/CLI/count/hash/
  whitespace verification passes; and
- only Task-263 files are staged and committed, then clean HEAD/origin/stash
  inventory returns automatically to dependency-ordered Task 264+.

## Active Implementation Result

The exact API and consumer above are implemented without semantic expansion.
Five checker and four runner tests pass; all eleven adjacent pairs in the
12-category precedence order are exercised within the frozen test names.
Libraries/counts are `467/528/146/59`, metadata is `426/394`, active type is
`203`, and the sole new trace row is covered by the sole new pass sidecar.
Independent reviews report **NO FINDINGS**, all nine hard gates PASS without a
score cap at `100/100`, and full verification passes. Exact staging, commit,
and clean post-commit inventory remain pending.
