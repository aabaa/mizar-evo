# Source Property-Implementation Transport

> Canonical language: English. Japanese companion:
> [../ja/source_property_implementation.md](../ja/source_property_implementation.md).

## Task 264 Scope And Authority

Checker Task 264 owns one syntax-free, immutable source-to-checker intake for
struct-property implementations and their initial correctness obligations. Its
canonical authority is Chapter 5's virtual-property/value-source boundary;
Chapter 7 §§7.4.1, 7.8.2, and 7.10; Chapter 13 §§13.1.2 and 13.8.2; Chapter 16
§§16.6.1, 16.6.2, and 16.7.2; the Parser Task-48 pass/recovery fixtures; the
inactive overlap-without-coherence seed; and committed Tasks 248--255, 259,
263, 264R, and 248P public transports. The existing active mixed
predicate/functor gap remains a read-only isolation oracle.

This task closes only the checker `source_drift`, derived `design_drift`, and
canonical-derived `test_gap` for the bounded profiles below. It does not prove
existence, uniqueness, or coherence; accept an implementation; publish a
property value, fact, or axiom; construct a FOL goal or parameter/domain guard;
discharge a proof; or lower Core, CFG, or VC IR. There is no blocking
`spec_gap`. Treating the context-only Task-264R shell as a property semantic
identity, or deriving proof semantics from justification syntax, would be a
`boundary_violation`.

## Frozen Exact Sources

The future active means source is exactly, including its final LF:

```mizar
definition
  struct Task264Carrier where
    field carrier -> set;
    property marker -> set;
  end;
end;

definition
  let M be Task264Carrier;
  property M.marker means it = it;
  existence by computation(steps: 1);
  uniqueness by computation(steps: 1);
end;
```

It is 263 bytes, 13 lines, and has SHA-256
`cc90659f10cae4ef68890624df9b8b9d3f0e830dae5e20cc195dc8b263c5fa2b`.
The future active equals source is exactly:

```mizar
definition
  struct Task264Carrier where
    field carrier -> set;
    property marker -> set;
  end;
end;

definition
  let M be Task264Carrier;
  property M.marker equals M.carrier;
end;
```

It is 189 bytes, 11 lines, and has SHA-256
`175135aaf40b9eab1a28e73ca1aae9f250e66278410d50575cdd279f6d7a2784`.

Each source has one normal structure declaration with field `carrier -> set`
and virtual property `marker -> set`, followed by one top-level property
implementation with one `M: Task264Carrier` parameter. The means profile has
one equality-formula definiens with two `it` occurrences and explicit
existence/uniqueness clauses. The equals profile has one structure-selector
term definiens and no correctness clause. Neither source contains `assume`,
coherence, import, reserve, mode definition, attribute, predicate, functor,
theorem, proof block, conditional/case/otherwise definiens, recovery, or a
property-owned return-type declaration.

The parameter type supplies the implementation domain. The return type is
looked up from the referenced `marker` declaration; it is never inferred from
the definiens and never attached to the Task-264R shell. The two sources are
separate transactions, so they do not overlap and do not authorize a
coherence obligation.

## Frozen Surface And Resolver Profiles

Both sources parse with zero diagnostics, a normal root, and no expression
root. The means profile has exactly 85 dense Surface rows and root 84 at
`0..262`. Its semantic structural rows are:

| Node | Surface kind | Range | Task-264 role |
| ---: | --- | --- | --- |
| 53 | `StructurePattern` | `20..34` | structure name |
| 54/55/56 | `TypeHead` / `TypeExpression` / `StructureField` | `62..65` / `45..66` | field return declaration |
| 57/58/59 | `TypeHead` / `TypeExpression` / `StructureProperty` | `90..93` / `71..94` | property return declaration |
| 60/61 | `StructureDefinition` / `DefinitionBlockItem` | `13..101` / `0..106` | declaration owner only |
| 62/63/64 | `PathSegment` / `QualifiedSymbol` / `TypeHead` | `130..144` | parameter type |
| 65 | `DefinitionParameter` | `121..145` | implementation parameter |
| 66/67 and 68/69 | `ItTerm` / `TermExpression` | `172..174`, `177..179` | property-value occurrences |
| 70/71/72 | `BuiltinPredicateApplication` / `FormulaExpression` / `FormulaDefiniens` | `172..179` | means definiens |
| 73--76 | computation/justification/correctness | `183..218` | existence clause; proof subtree excluded |
| 77--80 | computation/justification/correctness | `221..257` | uniqueness clause; proof subtree excluded |
| 81 | `PropertyImplementation` | `108..262` | context and payload owner |
| 82/83/84 | `ItemList` / `CompilationUnit` / `Root` | `0..262` | module framing |

The equals profile has exactly 56 rows and root 55 at `0..188`. Its common
structure rows are 35--43; parameter path/head/parameter are 44--47; term
reference 48 is `M` at `173..174`; selector 49 and term/definiens 50/51 cover
`M.carrier` at `173..182`; property implementation 52 covers `108..188`; and
item/compilation/root are 53/54/55. Tokens 24--34 are exactly
`property M . marker equals M . carrier ; end ;` at `148..188`.

The private selector authenticates every loaded byte, final LF, every row
kind/range/recovery/ordered child, root identity, direct item order, and these
subtree partitions. Pattern/name tokens, parameter type tokens, structure
declaration descendants, correctness keywords, and computation-justification
descendants are excluded from definiens discovery.

Each resolver result is exactly five shells, three symbols, three definitions,
one local-source contribution, and zero resolver diagnostics:

- shells 0--3 are definition block `0..106`, structure definition `13..101`,
  field `45..66`, and property `71..94`;
- shell 4 is the parentless `PropertyImplementation`, node/range `81/108..262`
  for means and `52/108..188` for equals;
- definition 0 is structure `Task264Carrier`, definition 1 is selector
  `carrier`, and definition 2 is selector `marker`;
- definition 0 and its symbol share the normal local structure origin
  `13..101/[4,0,11,0]`, and the Task-249PI parameter head must name that
  exact resolver identity rather than a task-local hard-coded FQN;
- target `marker` is `SymbolKind::Selector` / `DefinitionKind::Selector`,
  contribution 0, normal exported local origin `71..94` with structural path
  `[4,0,11,0,19,1]`; and
- `carrier` has the normal local sibling selector origin
  `45..66/[4,0,11,0,18,0]`, shared by its symbol and definition; it is
  consumed only by the equals lower structure-selector handoff.

The property-implementation shell has no signature projection, symbol,
definition, contribution, or semantic origin. Task 264 validates shell 4 only
through the Task-248P context handoff and validates the referenced property
only through definition 2. Opaque signature text is not parsed to recover a
return type or property identity.

### Carrier identity transport

The representation-only
[Task264C contract](../../task_contracts/en/CHECKER-SOURCE-PROPERTY-CARRIER-IDENTITY-264C.md)
extends the existing handoff with one immutable
`SourcePropertyCarrierIdentity`. Its private fields retain the exact
structure/field/property resolver tuples described above; twelve role-specific
getters expose each whole symbol, definition, contribution, and semantic
origin, and `SourcePropertyImplementationHandoff::carrier_identity()` exposes
the aggregate. The producer derives the value from its existing `SymbolEnv`;
its signature and all Typed/Resolved installation APIs remain unchanged.

Construction validates all three exact resolver rows and their sole
contribution effects. Replay validates the retained normal origins, common
module/contribution, parameter type head, and equality between the retained
property and target row 0. All failures use the existing
`InvalidResolverTarget { index: 0 }`. The aggregate is authenticated transport
only: it is not the property-implementation shell's identity, a Core item, a
property value, or an accepted semantic fact. Its deterministic debug format
is `source-property-implementation-debug-v2` with structure, field, and
property identity rows before the existing payload rows.

## Frozen Lower Bundle And Mandatory Type Prerequisite

The exact profiles consume these lower owners:

| Owner | Means | Equals | Ownership |
| --- | --- | --- | --- |
| Task 248P | Profile C `1/1/1/2/2/2/0` | same | shell 4, context 1, binding 0 |
| Task 249 + Task 249PI | applications/expressions/members `1/3/2` | same | parameter type plus field/property written returns |
| Task 252 | `2/0/0` | `1/1/0` | two `it` terms; or selector base `M` |
| Task 253 | absent | absent | no functor application root |
| Task 254 | absent | `1/0/0/1/0/1/3` | `M.carrier` selector term/member/base edge and three requests |
| Task 255 | absent | absent | no set/choice/qua root |
| Task 256 | `1/0/0/0/0/0/0/2/2` | absent | equality formula and two primary edges |
| Tasks 259/263 | absent and isolated | absent and isolated | no sibling definition handoff |

Fresh inventory finds one lower `source_drift`: current Task-249S accepts only
its standalone four-row Task-263 structure-member profile and forbids a
binding-linked parameter application in the same handoff. Task 264 requires
one application root for `Task264Carrier` plus two independently owned member
return rows for `carrier -> set` and `marker -> set`. This is not safe to
fabricate in Task 264. After the Task-264 docs commit, fresh preflight must
select separate checker Task 249PI, with its own docs and implementation
commits, to add exactly this combined lower profile. Task 249PI must preserve
all existing Task-249/249R/249M/249S bytes and tests. Task 264 implementation
cannot begin until that prerequisite is committed and fresh inventory passes.

In the combined lower profile, application 0 belongs to binding 0 and has root
expression 0 for `Task264Carrier`; structure-member rows 0/1 have roots 1/2
and ranges `45..66`/`71..94`; property target `marker` consumes
`SourceTypeStructureMemberId(1)`. The exact expression/head site pairs are
means parameter `63/64`, field `55/54`, property `58/57`, and equals parameter
`45/46`, field `37/36`, property `40/39`. All three expressions in either
transaction are normal, argument-free, current-module source types.

Required fingerprints are the complete Task-248P, Task-249PI, and Task-252
`debug_text()` strings. Task-253/254/255/256 fingerprints are optional and
present exactly when a definiens target uses that family. Means has
`None/None/None/Some`; equals has `None/Some/None/None`. Callers cannot supply
fingerprint strings.

The exact Profile-C context uses module site/root 84 or 55, item shell/ordinal
`4/4`, item site 81 or 52, item range `108..262` or `108..188`, local scope
`[4]`, binding context/local context `1/1`, and module context/link `0/0`.
Binding 0 is the normal `DefinitionParameter` `M`, transaction ordinal and
visible-after ordinal zero, declaration `125..126`, written-type `130..144`,
site 65 or 47, and has no shadow/predecessor. Both local contexts have empty
assumptions and visible facts. The complete cardinality is
`1/1/1/2/2/2/0`; no context row is inferred from the implementation body.

## Exact Public Syntax-Free Input

Implementation adds `source_property_implementation.rs` with five dense ID
families:

```rust
pub struct SourcePropertyImplementationId(usize);
pub struct SourcePropertyParameterId(usize);
pub struct SourcePropertyTargetId(usize);
pub struct SourcePropertyDefiniensId(usize);
pub struct SourcePropertyCorrectnessId(usize);
```

Each ID is `Copy + Eq + Ord + Hash`, exposes only `new` and `index`, and is
allocated by vector order. The exact input surface is:

```rust
pub struct SourcePropertyImplementationHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub implementations: Vec<SourcePropertyImplementationInput>,
    pub parameters: Vec<SourcePropertyParameterInput>,
    pub targets: Vec<SourcePropertyTargetInput>,
    pub definientia: Vec<SourcePropertyDefiniensInput>,
    pub correctness: Vec<SourcePropertyCorrectnessInput>,
}

pub struct SourcePropertyImplementationInput {
    pub shell: DeclarationShellId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub context: BindingContextId,
    pub recovery: SourcePropertyImplementationRecovery,
    pub spelling: String,
    pub style: SourcePropertyImplementationStyle,
    pub parameter: SourcePropertyParameterId,
    pub target: SourcePropertyTargetId,
    pub definiens: SourcePropertyDefiniensId,
}

pub struct SourcePropertyParameterInput {
    pub owner: SourcePropertyImplementationId,
    pub ordinal: usize,
    pub binding: BindingId,
    pub written_type: SourceTypeApplicationId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub declaration_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourcePropertyImplementationRecovery,
    pub spelling: String,
}

pub struct SourcePropertyTargetInput {
    pub owner: SourcePropertyImplementationId,
    pub ordinal: usize,
    pub subject: BindingId,
    pub symbol: SymbolId,
    pub definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub subject_range: SourceRange,
    pub name_range: SourceRange,
    pub spelling: String,
    pub return_type: SourceTypeStructureMemberId,
}

pub struct SourcePropertyDefiniensInput {
    pub owner: SourcePropertyImplementationId,
    pub ordinal: usize,
    pub target: SourcePropertyDefiniensTarget,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub context: BindingContextId,
    pub recovery: SourcePropertyImplementationRecovery,
    pub spelling: String,
}

pub struct SourcePropertyCorrectnessInput {
    pub owner: SourcePropertyImplementationId,
    pub ordinal: usize,
    pub kind: SourcePropertyCorrectnessKind,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub justification: SourceAnchor,
    pub recovery: SourcePropertyImplementationRecovery,
    pub spelling: String,
}

#[non_exhaustive]
pub enum SourcePropertyImplementationStyle { Equals, Means }

#[non_exhaustive]
pub enum SourcePropertyDefiniensTarget {
    Primary(SourcePrimaryTermId),
    Application(SourceFunctorApplicationId),
    Structure(SourceStructureTermId),
    SetTerm(SourceSetTermId),
    AtomicFormula(SourceAtomicFormulaId),
}

#[non_exhaustive]
pub enum SourcePropertyCorrectnessKind { Existence, Uniqueness }

#[non_exhaustive]
pub enum SourcePropertyImplementationRecovery { Normal, Degraded }
```

The target `site` is exactly `TypedSiteRef::Role { node: 81,
role: TypeRole::new("source.property-implementation.target") }` for means and
the same role at node 52 for equals, with exact range `157..165`;
subject/name ranges are `157..158` / `159..165`. This avoids inventing a
Surface selector node that the grammar does not provide in the property
header. All input structs derive `Debug + Clone + PartialEq + Eq`. IDs and
public enums follow the repository non-exhaustive/forward-compatible policy;
the style, target, correctness-kind, and recovery enums are also `Copy`.
Raw Surface nodes and proof descendants never cross this seam.

## Immutable Output, Debug, And Producer API

The immutable row type names and stored fields, in API order, are exact:

| Row type | Stored fields, in API order |
| --- | --- |
| `SourcePropertyImplementation` | `id`, `shell`, `site`, `source_range`, `source_ordinal`, `context`, `recovery`, `spelling`, `style`, `parameter`, `target`, `definiens` |
| `SourcePropertyParameter` | `id`, `owner`, `ordinal`, `binding`, `written_type`, `site`, `source_range`, `declaration_range`, `context`, `recovery`, `spelling` |
| `SourcePropertyTarget` | `id`, `owner`, `ordinal`, `subject`, `symbol`, `definition`, `contribution`, `site`, `source_range`, `subject_range`, `name_range`, `spelling`, `return_type`, derived `origin` |
| `SourcePropertyDefiniens` | `id`, `owner`, `ordinal`, `target`, `site`, `source_range`, `context`, `recovery`, `spelling` |
| `SourcePropertyCorrectness` | `id`, `owner`, `ordinal`, `kind`, `site`, `source_range`, `justification`, `recovery`, `spelling`, derived `obligation` |

The getter names and signatures are exact. Every omitted implementation body
is a direct immutable field return:

```rust
impl SourcePropertyImplementation {
    pub const fn id(&self) -> SourcePropertyImplementationId;
    pub const fn shell(&self) -> DeclarationShellId;
    pub const fn site(&self) -> &TypedSiteRef;
    pub const fn source_range(&self) -> SourceRange;
    pub const fn source_ordinal(&self) -> usize;
    pub const fn context(&self) -> BindingContextId;
    pub const fn recovery(&self) -> SourcePropertyImplementationRecovery;
    pub fn spelling(&self) -> &str;
    pub const fn style(&self) -> SourcePropertyImplementationStyle;
    pub const fn parameter(&self) -> SourcePropertyParameterId;
    pub const fn target(&self) -> SourcePropertyTargetId;
    pub const fn definiens(&self) -> SourcePropertyDefiniensId;
}

impl SourcePropertyParameter {
    pub const fn id(&self) -> SourcePropertyParameterId;
    pub const fn owner(&self) -> SourcePropertyImplementationId;
    pub const fn ordinal(&self) -> usize;
    pub const fn binding(&self) -> BindingId;
    pub const fn written_type(&self) -> SourceTypeApplicationId;
    pub const fn site(&self) -> &TypedSiteRef;
    pub const fn source_range(&self) -> SourceRange;
    pub const fn declaration_range(&self) -> SourceRange;
    pub const fn context(&self) -> BindingContextId;
    pub const fn recovery(&self) -> SourcePropertyImplementationRecovery;
    pub fn spelling(&self) -> &str;
}

impl SourcePropertyTarget {
    pub const fn id(&self) -> SourcePropertyTargetId;
    pub const fn owner(&self) -> SourcePropertyImplementationId;
    pub const fn ordinal(&self) -> usize;
    pub const fn subject(&self) -> BindingId;
    pub const fn symbol(&self) -> &SymbolId;
    pub const fn definition(&self) -> DefinitionId;
    pub const fn contribution(&self) -> SourceContributionId;
    pub const fn site(&self) -> &TypedSiteRef;
    pub const fn source_range(&self) -> SourceRange;
    pub const fn subject_range(&self) -> SourceRange;
    pub const fn name_range(&self) -> SourceRange;
    pub fn spelling(&self) -> &str;
    pub const fn return_type(&self) -> SourceTypeStructureMemberId;
    pub const fn origin(&self) -> &SemanticOrigin;
}

impl SourcePropertyDefiniens {
    pub const fn id(&self) -> SourcePropertyDefiniensId;
    pub const fn owner(&self) -> SourcePropertyImplementationId;
    pub const fn ordinal(&self) -> usize;
    pub const fn target(&self) -> SourcePropertyDefiniensTarget;
    pub const fn site(&self) -> &TypedSiteRef;
    pub const fn source_range(&self) -> SourceRange;
    pub const fn context(&self) -> BindingContextId;
    pub const fn recovery(&self) -> SourcePropertyImplementationRecovery;
    pub fn spelling(&self) -> &str;
}

impl SourcePropertyCorrectness {
    pub const fn id(&self) -> SourcePropertyCorrectnessId;
    pub const fn owner(&self) -> SourcePropertyImplementationId;
    pub const fn ordinal(&self) -> usize;
    pub const fn kind(&self) -> SourcePropertyCorrectnessKind;
    pub const fn site(&self) -> &TypedSiteRef;
    pub const fn source_range(&self) -> SourceRange;
    pub const fn justification(&self) -> &SourceAnchor;
    pub const fn recovery(&self) -> SourcePropertyImplementationRecovery;
    pub fn spelling(&self) -> &str;
    pub const fn obligation(&self) -> InitialObligationId;
}
```

The table names are exactly `SourcePropertyImplementationTable`,
`SourcePropertyParameterTable`, `SourcePropertyTargetTable`,
`SourcePropertyDefiniensTable`, and `SourcePropertyCorrectnessTable`. Each
exposes only `get(id) -> Option<&Row>`,
`iter() -> impl Iterator<Item = (Id, &Row)>`, `const len() -> usize`, and
`const is_empty() -> bool`. `iter` is dense source order and never sorts.

The handoff surface is exact:

```rust
pub struct SourcePropertyCarrierIdentity { /* private fields */ }

impl SourcePropertyCarrierIdentity {
    pub fn structure_symbol(&self) -> &SymbolId;
    pub const fn structure_definition(&self) -> DefinitionId;
    pub const fn structure_contribution(&self) -> SourceContributionId;
    pub const fn structure_origin(&self) -> &SemanticOrigin;
    pub fn field_symbol(&self) -> &SymbolId;
    pub const fn field_definition(&self) -> DefinitionId;
    pub const fn field_contribution(&self) -> SourceContributionId;
    pub const fn field_origin(&self) -> &SemanticOrigin;
    pub fn property_symbol(&self) -> &SymbolId;
    pub const fn property_definition(&self) -> DefinitionId;
    pub const fn property_contribution(&self) -> SourceContributionId;
    pub const fn property_origin(&self) -> &SemanticOrigin;
}

pub struct SourcePropertyImplementationHandoff { /* private fields */ }

impl SourcePropertyImplementationHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub const fn carrier_identity(&self) -> &SourcePropertyCarrierIdentity;
    pub fn source_context_fingerprint(&self) -> &str;
    pub fn source_type_fingerprint(&self) -> &str;
    pub fn source_term_fingerprint(&self) -> &str;
    pub fn source_functor_application_fingerprint(&self) -> Option<&str>;
    pub fn source_structure_fingerprint(&self) -> Option<&str>;
    pub fn source_set_term_fingerprint(&self) -> Option<&str>;
    pub fn source_atomic_formula_fingerprint(&self) -> Option<&str>;
    pub const fn implementations(&self)
        -> &SourcePropertyImplementationTable;
    pub const fn parameters(&self) -> &SourcePropertyParameterTable;
    pub const fn targets(&self) -> &SourcePropertyTargetTable;
    pub const fn definientia(&self) -> &SourcePropertyDefiniensTable;
    pub const fn correctness(&self) -> &SourcePropertyCorrectnessTable;
    pub fn debug_text(&self) -> String;
}
```

The first three fingerprints are complete lower `debug_text()` strings. The
four optional getters return `None` exactly when that lower handoff is absent,
not an empty string. No fingerprint is caller supplied.

The exact no-blank-line debug grammar is below. `Rust-debug String` means
standard escaped `{:?}` rendering; `<definiens-target>` is exactly one of
`primary#<id>`, `application#<id>`, `structure#<id>`, `set-term#<id>`, or
`atomic-formula#<id>`. Optional fingerprints render the unquoted token `none`
when absent and `some(<Rust-debug String>)` when present. Role sites use the
literal role frozen above.

```text
source-property-implementation-debug-v2
module: <ModuleId.path>
source-context-fingerprint: <Rust-debug String>
source-type-fingerprint: <Rust-debug String>
source-term-fingerprint: <Rust-debug String>
source-functor-application-fingerprint: <none|some(Rust-debug String)>
source-structure-fingerprint: <none|some(Rust-debug String)>
source-set-term-fingerprint: <none|some(Rust-debug String)>
source-atomic-formula-fingerprint: <none|some(Rust-debug String)>
carrier-identity#0 role=structure symbol=<Rust-debug FQN string> definition=<id> contribution=<id> origin_range=<start>..<end> origin_path=<Rust-debug [u32]>
carrier-identity#1 role=field symbol=<Rust-debug FQN string> definition=<id> contribution=<id> origin_range=<start>..<end> origin_path=<Rust-debug [u32]>
carrier-identity#2 role=property symbol=<Rust-debug FQN string> definition=<id> contribution=<id> origin_range=<start>..<end> origin_path=<Rust-debug [u32]>
implementation#<id> shell=<id> range=<start>..<end> site=node#<id> ordinal=<n> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String> style=<equals|means> parameter=<id> target=<id> definiens=<id>
parameter#<id> owner=<id> ordinal=<n> binding=<id> written_type=<id> range=<start>..<end> declaration_range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
target#<id> owner=<id> ordinal=<n> subject=<id> symbol=<Rust-debug FQN string> definition=<id> contribution=<id> range=<start>..<end> subject_range=<start>..<end> name_range=<start>..<end> site=role#<node>:source.property-implementation.target spelling=<Rust-debug String> return_type=<id> origin_range=<start>..<end> origin_path=<Rust-debug [u32]>
definiens#<id> owner=<id> ordinal=<n> target=<definiens-target> range=<start>..<end> site=node#<id> context=<id> recovery=<normal|degraded> spelling=<Rust-debug String>
correctness#<id> owner=<id> ordinal=<n> kind=<existence|uniqueness> range=<start>..<end> site=node#<id> justification=range:<start>..<end> recovery=<normal|degraded> spelling=<Rust-debug String> obligation=<id>
```

There is exactly one final LF. Row families are emitted in the shown order
and each table uses dense order. The exact profile admits only `Node` sites
except the target's specified `Role` site, `SourceAnchor::Range`
justifications, and normal local non-recovered resolver origin; other site,
anchor, imported, generated, or recovered forms fail closed. Typed and final
debug contain this complete block exactly once.

The exact API is:

```rust
pub struct SourcePropertyImplementationProjection { /* private */ }

impl SourcePropertyImplementationProjection {
    pub const fn base_initial_obligations(&self) -> &InitialObligationTable;
    pub const fn handoff(&self) -> &SourcePropertyImplementationHandoff;
    pub const fn initial_obligations(&self) -> &InitialObligationTable;
    pub fn into_parts(self) -> (
        InitialObligationTable,
        SourcePropertyImplementationHandoff,
        InitialObligationTable,
    );
}

#[non_exhaustive]
pub enum SourcePropertyImplementationError {
    SourceIdentityMismatch,
    DependencyMismatch,
    InvalidResolverTarget { index: usize },
    InvalidImplementation { index: usize },
    InvalidParameter { index: usize },
    InvalidTarget { index: usize },
    InvalidDefiniens { index: usize },
    InvalidCorrectness { index: usize },
    InvalidObligation,
    InvalidArenaOwnership,
    UnsupportedTaskShape,
}

pub struct SourcePropertyImplementationProducer;

impl SourcePropertyImplementationProducer {
    pub fn build(
        input: SourcePropertyImplementationHandoffInput,
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
    ) -> Result<SourcePropertyImplementationProjection,
                SourcePropertyImplementationError>;
}
```

`Display` and `Error` are implemented without `Default` or blanket
conversion. Failure is atomic. No mutable getter, public row constructor,
replacement method, raw syntax dependency, or property-value result exists.

## Active Row And Style Oracle

Both transactions have implementation/parameter/target/definiens cardinality
`1/1/1/1`. Means has correctness 2; equals has correctness 0.

Every admitted input/output row has recovery `Normal`; any `Degraded` row is
rejected by that row family's `Invalid* { index }` error. The exact typed-arena
kind literals at all Task-264-owned or directly consumed sites are:

| Profile | Node(s) | Exact `TypedNodeKind` string |
| --- | ---: | --- |
| means | 54, 57, 64 | `source.type.head` |
| means | 55, 58, 63 | `source.type.expression` |
| means | 56, 59 | `source.definition.structure.member` |
| means | 60 | `source.definition.structure` |
| means | 65 | `source.definition.property-implementation.parameter` |
| means | 66, 68 | `source.term.it` |
| means | 70 | `source.formula.atomic.equality` |
| means | 72 | `source.definition.property-implementation.definiens` |
| means | 76, 80 | `source.definition.property-implementation.correctness` |
| means | 81 | `source.definition.property-implementation` |
| means | 84 | `source.module` |
| equals | 36, 39, 46 | `source.type.head` |
| equals | 37, 40, 45 | `source.type.expression` |
| equals | 38, 41 | `source.definition.structure.member` |
| equals | 42 | `source.definition.structure` |
| equals | 47 | `source.definition.property-implementation.parameter` |
| equals | 31 | `source.term.structure.member.selector` |
| equals | 48 | `source.term.variable-reference` |
| equals | 49 | `source.term.structure.selector` |
| equals | 51 | `source.definition.property-implementation.definiens` |
| equals | 52 | `source.definition.property-implementation` |
| equals | 55 | `source.module` |

All unlisted nodes remain `source.surface.unowned`. Every node except the
parameter ownership nodes uses its exact Surface range. Nodes 65/47 use the
parameter declaration anchor `125..126` required by the frozen Task-248P
context contract while the parameter payload row retains source range
`121..145` and declaration range `125..126`. Every node has
`TypingState::Unknown`, `NodeRecoveryState::Normal`, and the
lower-authenticated context link; role site validation also authenticates the
base node's kind. Any wrong kind returns `InvalidArenaOwnership` before row
publication.

- means implementation 0 uses shell 4, site 81, range `108..262`, ordinal 0,
  context 1, style Means, owner IDs 0/0/0, and exact spelling
  `definition\n  let M be Task264Carrier;\n  property M.marker means it = it;\n  existence by computation(steps: 1);\n  uniqueness by computation(steps: 1);\nend;`;
- equals implementation 0 uses shell 4, site 52, range `108..188`, ordinal 0,
  context 1, style Equals, the same dense owner IDs, and exact spelling
  `definition\n  let M be Task264Carrier;\n  property M.marker equals M.carrier;\nend;`;
- parameter 0 uses binding/type application `0/0`, sites 65 or 47, range
  `121..145`, declaration range `125..126`, context 1, ordinal 0, spelling
  `let M be Task264Carrier;`;
- target 0 uses subject binding 0, resolver symbol/definition/contribution for
  `marker` (`definition 2`, contribution 0), role site at owner 81 or 52 with
  exact role `source.property-implementation.target`,
  range `157..165`, subject/name ranges `157..158` / `159..165`, spelling
  `M.marker`, and return row `SourceTypeStructureMemberId(1)`;
- means definiens 0 is `AtomicFormula(0)`, site 72, range `172..179`, context
  1, spelling `it = it`; equals definiens 0 is `Structure(0)`, site 51, range
  `173..182`, context 1, spelling `M.carrier`;
- correctness 0/1 exist only for means, use sites 76/80, ranges `183..218` /
  `221..257`, justification anchors `193..217` / `232..256`, spellings
  `existence by computation(steps: 1);` /
  `uniqueness by computation(steps: 1);`, and obligations at baseline `b` /
  `b+1`.

A Means row requires an atomic-formula target and exactly two lower
`SourcePrimaryTermKind::It` rows with role `CurrentDefinitionResult`, both
wholly inside its definiens. Every `it` in the selected transaction must
belong to that means definiens. A one-`it`, three-`it`, zero-`it`, relocated,
wrong-role, or wrong-spelling means profile returns exactly
`InvalidDefiniens { index: 0 }`. Equals requires a non-formula term target,
zero correctness rows, and zero `it` occurrences. Injecting `it` into equals
or pairing Equals with an atomic formula also fails closed. Task 264 does not
replace `it` with a value term or create a definitional equation.

There is no guard table, guard ID, guard getter, assumption-statement input, or
guard fingerprint. The mode/structure type of `M` authenticates the domain
context; Task 264 does not invent the parameter/domain FOL guard.

## Initial Obligations And Task-259 Separation

`InitialObligationKind` gains exactly
`PropertyImplementationExistence` and
`PropertyImplementationUniqueness`. All three exhaustive serializers map them
to `property_implementation_existence` and
`property_implementation_uniqueness`. Means appends two pending rows in clause
order; equals appends none.

For baseline length `b`, means obligations are:

| Field | Existence | Uniqueness |
| --- | --- | --- |
| id/kind | `b` / `PropertyImplementationExistence` | `b+1` / `PropertyImplementationUniqueness` |
| owner/range | correctness site 76 / `183..218` | correctness site 80 / `221..257` |
| assumptions | empty | empty |
| goal | `source.definition.property-implementation.correctness:implementation=0:existence` | `source.definition.property-implementation.correctness:implementation=0:uniqueness` |
| provenance | `source.definition.property-implementation:implementation=0:correctness=0` | `source.definition.property-implementation:implementation=0:correctness=1` |
| status | `Pending` | `Pending` |

Empty assumptions mean only that Task 264 does not invent Chapter-7/16 guard,
return-type, relation, or FOL composition. The authenticated parameter,
property return row, means formula, and explicit correctness clauses remain
separate transport for a later VC owner.

Build/installation reject a baseline containing either new property kind,
`PredicatePropertyCorrectness`, `FunctorExistence`, or `FunctorUniqueness`.
Typed and final assembly reject a Task-259 predicate-definition handoff beside
Task 264, orphan property kinds without a handoff, extra property kinds, or a
means/equals obligation mismatch. Task-259 code, its obligation ordering, and
the mixed predicate/functor gap fixture remain byte-unchanged. No
predicate/property coexistence or install-order promise is made.

## Typed And Resolved Ownership

The one-shot owner surface is:

```rust
impl TypedAst {
    pub fn with_source_property_implementation(
        self,
        projection: SourcePropertyImplementationProjection,
    ) -> Result<Self, TypedAstError>;
    pub const fn source_property_implementation(
        &self,
    ) -> Option<&SourcePropertyImplementationHandoff>;
}

TypedAstError::InvalidSourcePropertyImplementation

impl ResolvedTypedAst {
    pub const fn source_property_implementation(
        &self,
    ) -> Option<&SourcePropertyImplementationHandoff>;
}

ResolvedTypedAstError::InvalidSourcePropertyImplementation
```

`TypedAstParts` and `ResolvedTypedAstInputs` gain no public construction field.
The one-shot method installs handoff and obligations together against the exact
retained baseline. Final assembly revalidates every lower fingerprint and
obligation link, clone-preserves the handoff, and publishes no checked formula,
expression metadata, fact, proof, diagnostic, or acceptance row.

## Dedicated Consumers, Tests, And Write Scope

Implementation adds two pass pairs:

- `pass_type_elaboration_property_implementation_means_payload_001.{miz,expect.toml}`;
- `pass_type_elaboration_property_implementation_equals_payload_001.{miz,expect.toml}`.

Both are `pass` / `type_elaboration` / `type_check`, have empty public
diagnostics/payloads, and cite only
`spec.en.checker.type_elaboration.source_property_implementation_payload`.
One covered trace row backlinks both sidecars. Credit is source transport plus
pending-obligation intake only. Parser Task-48 fixtures, the inactive overlap
seed, existing mixed definition gaps, and all existing expectations remain
byte-identical.

Five checker tests are frozen:

1. exact means/equals tables, getters, fingerprints, debug, and obligations;
2. independent row/style/`it`/correctness corruption fail-closed coverage;
3. resolver target, return-row, lower fingerprint, arena, and obligation
   corruption coverage;
4. transactional typed installation with a nonempty unrelated baseline; and
5. final clone/debug determinism, orphan/extra rejection, Task-259 isolation,
   and no semantic publication.

Four runner tests are frozen:

1. exact bytes/hash/Surface/resolver/lower/output for both sources;
2. byte/final-LF/AST/subtree/resolver/lower/`it` mutations fail at their owner;
3. exact two-case selection, reciprocal trace/count metadata, inactive
   coherence seed, and mixed-definition route isolation; and
4. proof-subtree preservation plus absence of goal composition, discharge,
   acceptance, facts, diagnostics, Core/CFG/VC output.

After Task 249PI, Task 264 may change only the new checker module, checker
crate export, typed/final one-shot owners and exhaustive serializers, lint
policy, the new private runner route and parent facades, one runner test leaf,
mechanical active-count assertions, the two new fixture/sidecar pairs, one
trace row, one `#[cfg(test)]`-only generic raw-term corruption seam required to
exercise the frozen impossible-state `it` validation, and synchronized derived
EN/JA records. The seam adds no production behavior or public API. Parser,
resolver, Cargo,
canonical specs, existing `.miz`, existing sidecars/expectations, existing
trace rows, Task-259 validation, and unrelated lower producers are forbidden.

Task 249PI is frozen to add exactly four checker-local lower tests and no
runner test, so it first rebaselines checker `469 -> 473`. Task 264 then
projects checker `473 -> 478` (+5) and runner `528 -> 532` (+4), with
resolver/syntax `148/59` unchanged; corpus/requirements
`426/394 -> 428/395`; pass/fail `233/193 -> 235/193`; active
parse/declaration/type/proof `101/7/203/1 -> 101/7/205/1`; and type
requirements `258 = 246 covered + 12 deferred -> 259 = 247 + 12`.
Warnings/errors stay `23/0`. Checker production paths project `28 -> 29` and
runner production `35 -> 36`; final line counts and path/content hashes are
remeasured after implementation. The docs prerequisite changes no executable
count, fixture, trace status, CLI output, production byte, or test-list hash.

## Semantic Deferrals And Exit Criteria

Deferred and forbidden are: parameter/domain/return-type/definiens goal or
guard composition; substitution of `it`; FOL existence/uniqueness/coherence;
overlap detection and the inactive coherence seed; proof parsing/verification,
discharge, acceptance, activation, facts/axioms, property-value lookup at use
sites, selector/result typing, calls, conditional/case/otherwise profiles,
dependent/attributed modes or return types, imported properties, multiple
parameters/implementations, inheritance/refinement, redefinition, recursion,
Core/CFG/VC, and every later proof/semantic family.

The documentation prerequisite exits only with synchronized EN/JA, repeated
review-only **NO FINDINGS**, unchanged executable artifacts/counts/hashes, all
nine hard gates PASS, uncapped quality at least 90/100, exact docs-only
staging, one commit, clean worktree, unchanged protected stash, and fresh
inventory selecting Task 249PI. Task 249PI then commits docs and implementation
separately and returns automatically to Task 264. Task 264 implementation has
the same reviews and hard gates plus the projected executable counts, exact
count/hash verification, and one dedicated logical-task commit.

## Public Enum Policy

| Public enum | Compatibility policy |
| --- | --- |
| `SourcePropertyImplementationStyle` | `#[non_exhaustive]`; callers must tolerate later explicitly frozen implementation styles. |
| `SourcePropertyDefiniensTarget` | `#[non_exhaustive]`; callers must tolerate later explicitly frozen lower-root targets. |
| `SourcePropertyCorrectnessKind` | `#[non_exhaustive]`; callers must tolerate later explicitly frozen correctness kinds. |
| `SourcePropertyImplementationRecovery` | `#[non_exhaustive]`; callers must tolerate later recovery classes. |
| `SourcePropertyImplementationError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Implemented Task 264 Result

The frozen contract is implemented without changing canonical specifications,
pre-existing fixtures, or pre-existing expectations. Checker production is
`29` paths / `162347` lines, with path/content hashes
`37b91c2c419b83fa63150fe65d09b56c474dfa3d61134ba84056009dcdb923c1` /
`450abc3b7407f206c27b04613737716cf2192fb46c8960c8e167fcf0900fa143`.
Checker library tests are `478`, with raw/normalized hashes
`b3d0b2e398899adac6b94c7bbaba93d89fdc2067452e6b3c16efb60783401b8d` /
`4d9c7f9821182f08aa37686c7fecc1374d3857fdb7fdd64c83520dd05988d500`.

Runner production is `36` paths / `69417` lines, with path/content hashes
`38a20909d1f89aa2a4c325fb47126cc911bb943b7fe1190dc668713f64ad49e2` /
`72cc9036654639dff5933dced07e79ec6132696b5f92eca5e0149085f4651d91`.
Runner library tests are `532`, with raw/normalized hashes
`8122a53fddb8ee98cf1225f43c4f6966f3f7b5718673f55218601ca3464ca293` /
`fbd9e691357c14cd413df7ffd46677e34914bbacffca4dc2fe25a856d3b9434a`.

The two source/sidecar hashes are equals
`175135aaf40b9eab1a28e73ca1aae9f250e66278410d50575cdd279f6d7a2784` /
`c491d7ea65e1c096d869af4666a06a053a5a0b213d9e79483d13e5ec91b75b6e`
and means
`cc90659f10cae4ef68890624df9b8b9d3f0e830dae5e20cc195dc8b263c5fa2b` /
`bced77302602f43f3237424aa2963e5522c1458e879e606c68d1a516cd737c3a`.
Trace hash is
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`.
Metadata is `428/395`, pass/fail `235/193`, active stages
`101/7/205/1`, type coverage `259 = 247 + 12`, and warnings/errors `23/0`.
Plan/parse/declaration/type/proof stdout hashes are
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718`,
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56`,
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74`,
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f`,
and `ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`.
