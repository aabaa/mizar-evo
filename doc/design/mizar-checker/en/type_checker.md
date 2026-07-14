# mizar-checker: Type Checker

> Canonical language: English. Japanese companion:
> [../ja/type_checker.md](../ja/type_checker.md).

## Purpose

`type_checker` specifies phase-6 type checking and inference after resolver name
resolution and binding-environment construction. It refines:

- [architecture 04](../../architecture/en/04.type_and_registration_resolution.md)
  Step 2, "Normalize Type Expressions";
- [architecture 04](../../architecture/en/04.type_and_registration_resolution.md)
  Step 3, "Check Declarations and Local Bindings";
- [architecture 04](../../architecture/en/04.type_and_registration_resolution.md)
  Step 4, "Infer Terms and Formulas";
- [spec chapter 03](../../../spec/en/03.type_system.md) soft types, radix
  types, attributes, and widening;
- [spec chapter 08](../../../spec/en/08.type_inference.md) conversion,
  `qua`, and `reconsider`;
- [spec chapter 13](../../../spec/en/13.term_expression.md) term expressions,
  set enumeration, set comprehension, and choice terms;
- [`typed_ast.md`](./typed_ast.md) type, fact, coercion, obligation, and
  diagnostic tables;
- [`binding_env.md`](./binding_env.md) local binding and context construction.

Task 6 is a specification-only task. It adds no Rust source, no active checker
runner, no language behavior change, no registration closure, no final overload
root selection, no VC generation, and no proof acceptance. Tasks 7-11 implement
the named sections below.

## Boundary

`type_checker` owns:

- normalization of source type expressions into deterministic normalized type
  keys;
- declaration, local-binding, and binding-type checks over `BindingEnv`;
- term and formula type inference before final overload root selection;
- expected-type constraints and unresolved typed candidate groups;
- widening, source-written `qua`, and narrowing coercion candidates;
- checker-local `InitialObligation`s for sethood, non-emptiness, and
  narrowing claims;
- type facts, local assumptions, and deterministic fact queries;
- partial typing and diagnostic recovery for recoverable semantic errors.

`type_checker` does not own:

- resolver name lookup, label lookup, import/export validation, or symbol
  allocation;
- binding-context construction or binder identity rules already owned by
  `binding_env`;
- cluster saturation, registration activation, reduction normalization, or the
  canonical `ResolutionTrace` schema;
- final ordinary overload root selection, active refinement joining, or inserted
  overload-disambiguating `qua` views;
- expansion replay for `set`, `deffunc`, or `defpred` bodies beyond preserving
  the definition-time closure metadata supplied by `binding_env`;
- `VcId`, `ObligationAnchor`, proof witness, prover result, accepted verifier
  status, or kernel replay;
- public diagnostic-code allocation while the checker diagnostic code-space
  remains an external planning gate.

## Inputs And Outputs

The type checker consumes:

- one resolver `ResolvedAst`;
- the matching resolver `SymbolEnv`;
- a validated `BindingEnv`;
- checker-local configuration controlling recovery and feature gates;
- dependency summaries and activated registration summaries when later tasks
  expose them through a task-scoped seam.

The type checker produces a checker-local `TypedAst` snapshot:

```rust
struct TypeCheckOutput {
    typed_ast: TypedAst,
    diagnostics: TypeDiagnosticTable,
}
```

The logical implementation may use a mutable `TypeCheckState` while checking,
but accepted output is expressed through `TypedAst` tables:

- `LocalTypeContextTable`;
- `TypeTable`;
- `TypeFactTable`;
- `CoercionTable`;
- `InitialObligationTable`;
- `TypeDiagnosticTable`.

No phase-6 output is a stable artifact schema. Later artifact tasks may project
stable summaries from `TypedAst` only through their own specifications.

## Normalized Type Model

Type checking treats Mizar types as soft predicates over untyped objects. A
normalized type is a radix/type-head key plus a canonical attribute set:

```rust
struct NormalizedType {
    id: NormalizedTypeId,
    head: TypeHeadRef,
    args: Vec<NormalizedTypeId>,
    attributes: AttributeSet,
    source: TypeSource,
    status: NormalizedTypeStatus,
}

enum TypeHeadRef {
    BuiltinObject,
    BuiltinSet,
    Mode(SymbolId),
    Structure(SymbolId),
    Error(TypeHeadErrorKind),
}

enum TypeHeadErrorKind {
    Unknown,
    WrongKind,
    Ambiguous,
    Unsupported,
    Recovery,
}

struct AttributeSet {
    positive: Vec<AttributeInstance>,
    negative: Vec<AttributeInstance>,
}

struct AttributeInstance {
    symbol: SymbolId,
    args: Vec<NormalizedTypeId>,
    source_range: SourceRangeKey,
    spelling: String,
}
```

Canonical type keys are ordered by head kind, canonical `SymbolId`, normalized
argument keys, then attribute keys. Source ranges are retained for diagnostics;
source spelling and ranges are retained on normalized records for debug
rendering, but neither defines semantic equality.

Required invariants:

- built-in `object` and `set` heads are canonical singleton heads;
- mode heads unfold to their declared radix and attribute chain when the resolver
  exposes enough signature payload;
- structure heads remain radix heads and are not silently treated as `set`;
- duplicate attributes with identical polarity and arguments collapse to one
  canonical instance;
- positive and negative occurrences of the same attribute key create a
  diagnostic and degraded normalized type rather than being silently removed;
- type arguments are normalized before they participate in the parent type key;
- missing signature, mode-definition, structure, or attribute payload is an
  `external_dependency_gap`, not an invitation to inspect raw parser syntax.

## Task 7: Type-Expression Normalization

Task 7 implements this section.

Inputs:

- checker-owned resolved type-expression payloads that identify typed sites,
  source ranges, type-head symbols, type arguments, and attribute occurrences;
- type-head, attribute, mode, and structure identities validated through
  `SymbolEnv`;
- an explicit mode-expansion provider for radix/attribute payloads when later
  resolver or artifact tasks expose those payloads.

Outputs:

- a task-local `TypeNormalizationOutput` owning `NormalizedTypeTable` entries in
  deterministic key order;
- `TypeEntry`s for type-expression sites;
- diagnostics for unknown heads, wrong arity, wrong symbol kind,
  contradictory attributes, unsupported checker-owned payloads, and missing
  explicit mode-expansion provider payloads.

Task 7 does not walk `ResolvedAst` directly because the current resolver does
not expose a typed site table for type expressions. The integration seam is the
checker-owned payload listed above. A later resolver/source-walk task may fill
that payload from `ResolvedAst`, but task 7 must not infer it from raw surface
node kinds.

Rules:

1. Resolve the type head only through resolver/symbol payloads. The checker must
   not redo name lookup or inspect raw syntax node kinds.
2. Normalize type arguments left-to-right, then use canonical type keys in the
   parent type.
3. Normalize attribute polarity by storing `non A` in the negative set and `A`
   in the positive set.
4. Sort attributes by canonical symbol id, normalized arguments, polarity, and
   source range; this order is for storage and rendering only.
5. Unfold mode definitions only through the explicit mode-expansion provider
   when it supplies defining radix and attribute payload. If that payload is
   absent, record an `external_dependency_gap` and keep a degraded type headed
   by the mode symbol.
6. Preserve source ranges for every diagnostic. Preserve source spelling and
   range on normalized type and attribute records for debug rendering, using a
   deterministic representative source for semantically equivalent type keys.
7. Never use cluster closure to "fix" a normalized type during task 7. Cluster
   closure is phase 7 and belongs to later registration tasks.

## Task 8: Declaration And Local-Binding Checking

Task 8 implements this section.

Declaration checking attaches normalized types to bindings and introduces local
facts into the appropriate `LocalTypeContext` snapshot.

Task 8 uses a checker-owned declaration payload. The payload identifies the
`BindingEnv` binding/context ids, typed declaration sites, optional explicit
type-expression payloads, and source ranges to check. The current resolver does
not expose an AST-wide declaration/type-site table, reserve default payload,
right-hand-side term payload, or definition body payload, so task 8 must not
walk raw syntax to reconstruct them.

Current source-derived producer seam: the `type_checker` module may accept a
syntax-free reserve-only payload extracted by an upstream runner and build the
checker-owned `BindingEnv` plus `DeclarationCheckingOutput` for builtin
`set` / `object` reserve declarations. For the successful pass slice the
payload is bare builtin `set` / `object`. Task 50 additionally permits
source-derived attribute payloads on those builtin heads only when the
attribute symbol is already present in resolver `SymbolEnv`; attributed
reserve declarations are marked with `MissingEvidenceQuery` and remain active
fail cases until a real existential/evidence-query seam exists. Task 51
additionally permits un-attributed reserve type heads that resolve to a unique
same-module `LocalSource` `SymbolKind::Mode` entry with no type arguments.
Those local-mode reserve declarations reach type normalization and fail closed
with `checker.type.external.mode_expansion_payload` until a real
mode-expansion provider/extraction seam exists. Task 54 permits that
local-mode slice to carry same-module source-derived attributes, which still
fail closed with `checker.type.external.mode_expansion_payload` when no
supported real mode-expansion payload is available or the same local mode is
mixed with a bare reserve use; the bridge does not claim existential evidence
for the fully expanded attributed type. Task 55 permits one real mode-expansion
producer slice for bare local-mode reserve uses only: the runner may provide a
`ModeExpansion` for a unique unrecovered same-module no-argument
`ModeDefinition` that precedes the reserve type use, has no definition-local
parameter/assumption context, and has a bare builtin `set` / `object` RHS with
no attributes or arguments. The runner withholds that expansion for sources
where any reserve binding uses attributes on the same local mode head, so task
54's attributed local-mode fail-closed behavior is preserved until task 59.
Task 59 then permits that same real direct bare-builtin RHS expansion for an
attributed local-mode reserve head only when the same local mode is not also
used as a bare reserve head in the same bridge input. The expanded attributed
declaration reaches `checker.declaration.deferred.evidence_query` because
real attributed-type existential evidence is still absent; mixed bare/
attributed uses stay on the missing-expansion path. Task 56 extends
that producer by one source-derived chain edge: a bare local-mode reserve head
may expand through a unique preceding same-module no-argument mode definition
whose RHS is another bare same-module no-argument local mode, but only when
that dependency mode has its own accepted task-55 bare builtin `set` /
`object` expansion, the dependency definition precedes the chain definition,
both definition nodes are uniquely paired in the AST, and no reserve binding
uses attributes on either chain symbol. Forward, ambiguous, partial, imported,
argument-bearing, parameterized, contextual, cyclic, attributed-structure RHS
outside the task-62 bare one-edge chain slice, attributed-RHS chains outside
the task-58/task-61 direct slices and task-63 bare one-edge chain slice,
attributed-root bare-builtin chains outside task 64, attributed-root
structure-RHS chains outside task 65, or broader attributed-builtin RHS mode
definitions remain on the missing-expansion /
extraction-gap path, and the checker-owned seam must not fabricate expansion
or existential evidence. Task 72 extends only the bare builtin terminal pass
slice one more source-derived dependency edge: a bare local-mode reserve head
may expand through `Outer -> Middle -> Base -> set` / `object` when all three
mode definitions are unique, unrecovered, same-module, no-argument,
definition-local-context-free, and source-preceding. The active pass fixtures
continue through the existing `TypedAst`, `ResolvedTypedAst`, summary-readiness,
and binder-only `CoreContext` preparation path, but do not promote new
CoreIr/ControlFlowIr/VC/proof payloads. Three-edge local-mode dependency
chains originally remained on the missing mode-expansion diagnostic for task
72. Task 73 promotes the same source-derived seam one more dependency edge:
`Outer -> Middle -> Inner -> Base -> set` / `object` may pass when all four
mode definitions satisfy the same uniqueness, same-module, no-argument,
definition-local-context-free, and source-preceding constraints. Task 74 then
replaces the temporary depth cap with a structural bare builtin-terminal rule:
any AST-bounded acyclic chain of same-module no-argument local modes may pass
when every mode definition is unique, unrecovered,
definition-local-context-free, source-preceding, argument-free, and
attribute-free, and the terminal RHS is exactly builtin `set` / `object`. The
producer carries an AST-derived traversal budget equal to the number of source
mode definitions; that budget is a resource guard, not a semantic chain-length
limit. Chains that violate those structural guards remain on the
missing-expansion / extraction-gap path. Task 75 fixes the forward-reference
boundary for this family: if a reserve head names a local mode before that
mode declaration item is active, lower-stage frontend/resolver processing
rejects the type expression with
`type_elaboration.lower_stage.frontend:malformed_type_expression` before any
checker handoff. The source-derived runner must not fabricate a later
`ModeExpansion` payload from the future declaration.
Task 76 mirrors that active-range boundary for same-module local structures:
if a reserve head names a local structure before the structure declaration item
is active, lower-stage frontend/resolver processing rejects the type expression
with `type_elaboration.lower_stage.frontend:malformed_type_expression` before
checker handoff. The runner must not fabricate a later structure type-head
payload, successful reserve declaration, base-shape/constructor-witness
evidence query, or downstream CoreIr/ControlFlowIr/VC/proof payload from the
future declaration.
Task 77 applies the same boundary to same-module local attributes: if a reserve
type expression such as `marked set` uses an attribute before the attribute
declaration item is active, lower-stage frontend/resolver processing rejects the
type expression with the same
`type_elaboration.lower_stage.frontend:malformed_type_expression` detail before
checker handoff. The runner must not fabricate an `AttributeInput`,
attributed-type evidence query, successful reserve declaration, or downstream
CoreIr/ControlFlowIr/VC/proof payload from the future attribute declaration.
Task 78 historically recorded the imported-structure analogue as an external
extraction-gap boundary rather than a checker payload. Task 83 supersedes that
boundary only for the documented `parser.type_fixtures` `R` reserve head by
passing the real imported `SymbolKind::Structure` as a checker type head. Task
97 applies the same narrow bridge to the documented `TypeCaseStruct` fixture.
Broader imported structures outside those task-83/task-97 fixtures remain
deferred. The bridge must not treat that imported summary as real imported
module AST extraction, must not fabricate base-shape/constructor-witness
evidence, positive structure elaboration, or downstream
CoreIr/ControlFlowIr/VC/proof payloads.
Tasks 83 and 97 record the imported-structure provenance bridge: `R` and
`TypeCaseStruct` from the documented `parser.type_fixtures` import summary
reach declaration checking as imported structure type heads, then fail closed
on `type_elaboration.checker.checker.declaration.deferred.evidence_query`
because base-shape/constructor-witness evidence is still absent. This does not
credit positive imported structure elaboration or imported module AST
extraction.
Task 79 originally recorded the imported-mode analogue as the same external
extraction-gap boundary. Task 82 supersedes that boundary only for the
documented `parser.type_fixtures` `TypeCaseMode` reserve head by passing the
real imported `SymbolKind::Mode` as a checker type head; imported modes outside
that Task82 bridge remain on
`type_elaboration.external_dependency.ast_payload_extraction`. The bridge must
not treat that imported summary as real imported module AST extraction, must
not fabricate `ModeExpansion` payloads, positive mode elaboration, or
downstream CoreIr/ControlFlowIr/VC/proof payloads.
Task 80 historically recorded the imported-attribute analogue as the same
external extraction-gap boundary. Task 84 supersedes that boundary only for the
documented `parser.type_fixtures` `TypeCaseAttr` reserve attribute by passing
the real imported `SymbolKind::Attribute` as a checker `AttributeInput`, and
task 85 supersedes it only for the existing negative `empty`/builtin-`set`
fixture by passing the real imported `empty` attribute as a negative checker
`AttributeInput`; task 116 supersedes it for the existing positive
`empty`/builtin-`set` fixture by passing the same imported attribute as a
positive checker `AttributeInput`; task 171 supersedes it for the existing
negative `empty`/builtin-`object` fixture by passing the real imported attribute
as a negative checker `AttributeInput`; broader imported attributes outside
those bridges remain deferred until source-derived fixtures and payload
producers exist. The bridge
must not treat that imported summary as real imported module AST extraction,
must not fabricate attributed-type evidence, positive attributed type
elaboration, positive `empty` on builtin `object`, imported `empty` on symbol
heads, or
downstream CoreIr/ControlFlowIr/VC/proof payloads.
Task 84 records the imported-attribute provenance bridge: `TypeCaseAttr` from
the documented `parser.type_fixtures` import summary reaches declaration
checking as an imported attribute payload on builtin `set`, then fails closed on
`type_elaboration.checker.checker.declaration.deferred.evidence_query` because
attributed-type existential/evidence payloads are still absent. This does not
credit positive imported attributed type elaboration, imported module AST
extraction, generic imported attributes such as `empty`, structure-qualified
attribute owner provenance, or attribute arguments.
Task 85 records the next imported-attribute provenance slice: the existing
`non empty set` fixture may pass the documented `parser.type_fixtures` imported
attribute `empty` as a negative checker `AttributeInput` on builtin `set`, then
fail closed on the same evidence-query diagnostic. This supersedes the broader
task-80 payload gap only for that negative `empty`/builtin-`set` source shape.
Task 116 records the matching positive imported-attribute provenance slice:
the existing `empty set` fixture may pass the documented imported `empty`
attribute as a positive checker `AttributeInput` on builtin `set`, then fail
closed on the same evidence-query diagnostic. This supersedes the broader
task-80 payload gap only for that positive `empty`/builtin-`set` source shape.
Task 171 specifies the matching negative builtin-object provenance slice: the
existing `non empty object` fixture must pass the documented imported `empty`
attribute as a negative checker `AttributeInput` on builtin `object`, retain
`ImportedSource` provenance and written polarity, and then fail closed on the
same evidence-query diagnostic. This supersedes the task-80 payload gap only
for that exact source shape. Positive `empty object` and imported attributes on
symbol heads remain extraction gaps. Tasks 85, 116, and 171 do not credit
attribute admissibility, attributed-type evidence or acceptance, imported
module AST extraction, structure-qualified owner provenance, attribute
arguments, or downstream payloads.
Task 81 records the same extraction-gap boundary for a same-module
argument-bearing local attribute surface: a declaration-site attribute written
with Chapter 6 `param_prefix` syntax, such as `attr RankedDef: x is 2-ranked`,
and a Chapter 3/6 use-site application such as `ranked(2) set`, are carried by
the real lexer/parser/frontend producer seam to the active type-elaboration
runner, with resolver declaration-symbol projection recording the suffix as the
primary spelling and lexer-visible lexical summary while preserving the
prefixed surface as notation. The runner reports
`type_elaboration.external_dependency.ast_payload_extraction` because checker
payload extraction still does not preserve real term-argument provenance or
checker-owned `AttributeInput` argument payloads. The bridge must not fabricate
attribute arguments, attributed-type evidence, positive parameterized
attribute elaboration, or downstream CoreIr/ControlFlowIr/VC/proof payloads.
Task 86 records a theorem/formula extraction-gap boundary: a source containing
only a theorem formula, for example `theorem FormulaPayloadBoundary: thesis;`,
can reach the active type-elaboration runner. Task 115 supersedes only this
exact source by passing the source-derived `thesis` formula constant site/range
to the checker as a recovery `FormulaInput`. Task 117 supersedes that recovery
marker by using a real `FormulaKind::Thesis` checker payload while keeping the
bridge fail-closed on missing formula payload. This boundary still does not
credit formula constant semantics, child-formula graph payloads, theorem
acceptance, formula facts, proof skeletons, local proof contexts,
`formula_statement`, CoreIr, ControlFlowIr, VC, or proof payloads.
Task 106 supersedes the task 87 generic boundary for the narrow builtin
equality theorem formula `theorem TermFormulaPayloadBoundary: 1 = 1;`. The
active runner now extracts real source-derived checker `TermInput` payloads for
the two Chapter 13 numeral operands and a real checker `FormulaInput` payload
for the Chapter 14 equality formula under the module binding context, then
fails closed on `type_elaboration.checker.checker.term.external.numeric_type_payload`
and `type_elaboration.checker.checker.formula.term.partial` because numeric
type payloads, equality checking, recorded facts, theorem acceptance, the
dedicated `formula_statement` runner, CoreIr, ControlFlowIr, VC, and proof
payloads are still absent.
Task 98 originally recorded the imported predicate/functor variant of that same
boundary:
`theorem ImportedPredicateFunctorPayloadBoundary: 1 divides (1 ++ 2);` reaches
parser and resolver execution through the documented `parser.type_fixtures`
surface. Task 110 supersedes that exact source by extracting real
source-derived numeral `TermInput`s, a functor-application `TermInput` carrying
the imported `++` symbol reference, and a predicate-application `FormulaInput`.
The runner validates `divides`/`++` imported provenance, then fails closed on
missing numeric type payload, missing functor signature payload, missing
predicate signature payload, and partial formula checking. This does not credit
imported module AST extraction, semantic predicate/functor signature payloads,
term inference, formula checking, recorded facts, theorem acceptance, the
dedicated `formula_statement` runner, CoreIr, ControlFlowIr, VC, or proof
payloads.
Task 108 supersedes the task 100 builtin membership generic boundary for the
exact formula `theorem BuiltinMembershipPayloadBoundary: 1 in 1;`. The active
runner now extracts real source-derived checker `TermInput` payloads for the
two Chapter 13 numeral operands and a real checker `FormulaInput` payload for
the Chapter 14 membership formula under the module binding context, then fails
closed on `type_elaboration.checker.checker.term.external.numeric_type_payload`
and `type_elaboration.checker.checker.formula.term.partial`. Numeric type
payloads, membership operand expected-type construction/checking, recorded
facts, theorem acceptance, the dedicated `formula_statement` runner, CoreIr,
ControlFlowIr, VC, and proof payloads are still absent.
Task 107 supersedes the task 101 generic boundary for the exact builtin
inequality theorem formula `theorem BuiltinInequalityPayloadBoundary: 1 <> 2;`.
The active runner now extracts real source-derived checker `TermInput` payloads
for the two Chapter 13 numeral operands and a real checker `FormulaInput`
payload for the Chapter 14 inequality formula under the module binding context,
then fails closed on
`type_elaboration.checker.checker.term.external.numeric_type_payload` and
`type_elaboration.checker.checker.formula.term.partial` because numeric type
payloads, inequality desugaring or equality semantic checking, recorded facts,
theorem acceptance, the dedicated `formula_statement` runner, CoreIr,
ControlFlowIr, VC, and proof payloads are still absent.
Task 118 tightens the shared task 106/107/108 builtin-binary theorem producer:
equality, membership, and inequality configs are selected only when the direct
theorem tokens are exactly `theorem <label> : ;`. Status-prefixed or otherwise
extra-token theorem shapes remain on
`type_elaboration.external_dependency.ast_payload_extraction`. This guard-only
repair adds no new `.miz` sidecar coverage or spec coverage credit.
Task 119 adds the first no-diagnostic source-derived identifier-term/equality
slice for the exact source
`reserve x for set; theorem ReservedVariableEqualityPayloadBoundary: x = x;`.
The runner reuses the real reserve `BindingEnv`, resolves both identifier-term
sites through independent `BindingEnv::lookup` calls. Their lookup ordinals
are assigned by sorting the source binding and the two use ranges as one
binding/use event stream, so the exact fixture derives ordinals 1 and 2 after
the reserve binding at ordinal 0 instead of supplying a shared synthetic use
ordinal. The runner projects the written reserve type's
range, spelling, and builtin `set` head into four distinct checker role sites:
the two term result types and the two equality expected-type constraints.
`TermFormulaChecker` records both variable terms as `Inferred` and the equality
formula as `Checked` with no diagnostics or facts. The active producer validates
the complete checker payload before reporting a pass: declaration/binding
identity, both lookup results, term/formula sites and statuses, expected-type
ranges, all four role owners, normalized source spelling/range/head, and empty
candidate/fact/deferred/diagnostic tables must agree. Any mismatch reports the
stable `type_elaboration.checker.reserved_variable_equality.invalid_payload`
detail key. Here `Checked` means only
that the source-derived term/type/formula payload is well formed; task 119 does
not materialize implicit universal-closure nodes, prove or accept the theorem,
record equality facts, activate `formula_statement`, or produce proof,
CoreIr, ControlFlowIr, or VC payloads. Non-exact labels, operands, reserve
bindings/types, attributed types, operators, status/extra tokens, additional
reserve or theorem items, source-order reversal, recovery, or numeral-term shapes
remain on `type_elaboration.external_dependency.ast_payload_extraction`.
Task 123 adds the exact distinct-binding sibling
`reserve x, y for set; theorem DistinctReservedVariableEqualityPayloadBoundary: x = y;`.
The real multi-reserve handoff owns two checker bindings even though both
source bindings point to the same written builtin `set` type range. Source
binding/use ordering derives lookup ordinals 2 and 3 after binding ordinals 0
and 1, and independent `BindingEnv::lookup` calls resolve the operands to
`BindingId(0)` and `BindingId(1)` rather than collapsing them. Operand-specific
result and expected roles retain the corresponding source binding provenance;
the checker records two `Inferred` variable terms and one fact-free `Checked`
equality. Production invariants validate the distinct identities, shared type
range, exact source shape, role ownership, and empty
candidate/fact/deferred/diagnostic output. Drift reports
`type_elaboration.checker.distinct_reserved_variable_equality.invalid_payload`,
and a near-miss matrix plus real frontend/resolver sidecar keep separate
reserve items, reversed/same operands, wrong labels/operators/types, extra
bindings/items, status/recovery, and numeral operands on the extraction gap.
This is type/well-formedness only: implicit universal closure and quantifier
order, equality truth/facts, theorem acceptance, `formula_statement`, proof,
CoreIr, ControlFlowIr, and VC remain deferred.
Task 124 adds the exact multiple-declaration sibling
`reserve x for set; reserve y for set; theorem MultipleReserveDeclarationEqualityPayloadBoundary: x = y;`.
The reserve producer retains two binding identities and two distinct written
type ranges. Four operand-specific pre-normalization result/expected inputs
retain those respective ranges before the checker interns their identical
builtin `set` semantics to one normalized type with the earliest deterministic
source representative. Validation checks the original inputs independently,
so normalized interning neither fabricates a duplicate type nor erases written
provenance. Exact shape guards, a task-specific invalid-payload key, near misses,
and a real frontend/resolver sidecar cover the seam. Implicit closure/order,
truth/facts, theorem acceptance, `formula_statement`, proof, CoreIr,
ControlFlowIr, and VC remain deferred.
Task 125 adds the exact heterogeneous membership sibling
`reserve x for object; reserve y for set; theorem HeterogeneousReserveMembershipPayloadBoundary: x in y;`.
The generalized binary bridge retains binding-specific builtin shapes rather
than assuming every reserved operand is `set`: the left result input is
source-derived `object`, while the right result and sole expected input are
source-derived `set`. Production validation requires two normalized identities,
with the right result/expected roles sharing the `set` identity and the left
`object` identity remaining distinct; each identity keeps the deterministic
canonical source from its written declaration. Exact guards, a task-specific
invalid-payload key, near misses, and a real frontend/resolver sidecar cover the
seam. This is type/well-formedness only: membership truth/facts, object/set
coercion evidence, implicit closure/order, theorem acceptance,
`formula_statement`, proof, CoreIr, ControlFlowIr, and VC remain deferred.
Task 126 adds the exact direct-local-mode equality sibling. Four raw
result/expected inputs retain the reserve's `LocalModeFormula` symbol and range,
while `TermFormulaChecker` receives the exact AST-derived direct bare-`set`
`ModeExpansion` and normalizes all roles to one builtin-set identity. The
normalized canonical source is the expansion RHS; original mode provenance
remains reviewable in the raw inputs. Exact guards, an invalid key, withheld-
family near misses, and a real sidecar guard the bridge. Mode declaration
checking/acceptance, inhabitation evidence, broader modes, closure/order,
truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr, and VC remain
deferred.
Task 127 adds the exact one-edge bare local-mode-chain equality sibling. The
active source has two separate definition blocks for
`BaseModeFormula -> set` and `ChainModeFormula -> BaseModeFormula`, then one
outer-mode reserve and `ChainedLocalModeReservedVariableEqualityPayloadBoundary:
x = x;`. The runner preserves four raw outer-mode inputs while the existing
recursive `TypeNormalizer` consumes both real task-56 expansions and keeps the
terminal `set` RHS as the one normalized identity's canonical source. Exact
labels/block structure/chain links, invalid-link corruption, and withheld-family
near misses guard the route. Mode declaration acceptance/inhabitation, object terminals, longer
chain formulas, closure/order, truth/facts, theorem acceptance, proof, CoreIr,
ControlFlowIr, and VC remain deferred.
Task 128 is limited to the exact direct local-object-mode equality sibling.
The source reuses the task-55 `LocalObjectMode -> object` definition,
adds one reserve and `LocalObjectModeReservedVariableEqualityPayloadBoundary:
x = x;`, preserves four raw object-mode inputs, and requires the existing
`TypeNormalizer` to anchor one builtin-object identity at the real expansion
RHS. Exact block/label guards, an invalid key, withheld-family near misses, and
a real sidecar protect the route. Mode declaration acceptance/inhabitation,
closure/order, truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr,
and VC remain deferred.
Task 129 is limited to the exact one-edge local-object-mode-chain equality
sibling. The source reuses task 56's
`ChainObjectMode -> BaseObjectMode -> object` producer, adds the exact outer-mode
reserve and `ChainedLocalObjectModeReservedVariableEqualityPayloadBoundary:
z = z;`, preserves four raw outer-mode inputs, and requires recursive
`TypeNormalizer` consumption of both real expansions with terminal object-RHS
provenance. Exact labels/block order/chain links, invalid-link corruption,
withheld-family near misses, and a real sidecar protect the route. Mode
declaration acceptance/inhabitation, longer-chain formulas, closure/order,
truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr, and VC remain
deferred.
Task 130 is limited to exact direct bare-set local-mode inequality. It preserves
four raw `LocalModeInequality` inputs, consumes the real expansion, and anchors
one builtin-set identity at the RHS before recording a fact-free
pre-desugaring `Checked` inequality. Non-exact shapes fail closed; declaration
acceptance, desugaring, truth/facts, theorem/proof/Core/VC remain deferred.
Task 131 applies that exact inequality consumer to the direct bare-object
`LocalObjectModeInequality -> object` producer. Four raw object-mode inputs
retain their written provenance while one real expansion normalizes them to a
single RHS-anchored builtin-object identity before two `Inferred` variable terms
and one fact-free pre-desugaring `Checked` inequality are recorded. Non-exact
shapes fail closed; mode declaration acceptance/inhabitation, desugaring,
closure/order, truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr,
and VC remain deferred.
Task 132 applies the same pre-desugaring inequality consumer to the exact
one-edge bare-set chain `ChainModeInequality -> BaseModeInequality -> set`.
Four raw outer-mode inputs retain their written provenance while both real
AST-derived expansions normalize them to one terminal-RHS builtin-set identity
before two `Inferred` variable terms and one fact-free `Checked` inequality are
recorded. Missing or non-exact links and object-terminal, direct, or longer
shapes fail closed; mode declaration acceptance/inhabitation, desugaring,
closure/order, truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr,
and VC remain deferred.
Task 133 applies that consumer to the exact one-edge bare-object chain
`ChainObjectModeInequality -> BaseObjectModeInequality -> object`. Four raw
outer-mode inputs retain written provenance while both real AST-derived
expansions normalize them to one terminal-RHS builtin-object identity before
two `Inferred` variable terms and one fact-free `Checked` inequality are
recorded. Missing/non-exact links and set-terminal, direct, or longer shapes
fail closed; declaration acceptance/inhabitation, desugaring, closure/order,
truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr, and VC remain
deferred.
Task 134 applies the equality consumer to the exact two-edge bare-set chain
`OuterTwoEdgeModeEquality -> MiddleTwoEdgeModeEquality -> BaseTwoEdgeModeEquality -> set`.
Four raw outer-mode inputs retain written provenance while all three real
AST-derived expansions normalize them to one terminal-RHS builtin-set identity
before two `Inferred` variable terms and one fact-free `Checked` equality are
recorded. Missing/non-exact links and object-terminal, direct, one-edge, or
longer shapes fail closed; declaration acceptance/inhabitation, implicit
closure/order, truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr,
and VC remain deferred.
Task 135 applies the same equality consumer to the exact two-edge bare-object
chain
`OuterTwoEdgeObjectModeEquality -> MiddleTwoEdgeObjectModeEquality -> BaseTwoEdgeObjectModeEquality -> object`.
Four raw outer-mode inputs retain written provenance while all three real
AST-derived expansions normalize them to one terminal-RHS builtin-object
identity before two `Inferred` variable terms and one fact-free `Checked`
equality are recorded. Missing/non-exact links and set-terminal, direct,
one-edge, or longer shapes fail closed; declaration acceptance/inhabitation,
implicit closure/order, truth/facts, theorem acceptance, proof, CoreIr,
ControlFlowIr, and VC remain deferred.
Task 136 applies the pre-desugaring inequality consumer to the exact two-edge
bare-set chain
`OuterTwoEdgeModeInequality -> MiddleTwoEdgeModeInequality -> BaseTwoEdgeModeInequality -> set`.
Four raw outer-mode inputs retain written provenance while all three real
AST-derived expansions normalize them to one terminal-RHS builtin-set identity
before two `Inferred` variable terms and one fact-free pre-desugaring `Checked`
inequality are recorded. Missing/non-exact links and object-terminal, direct,
one-edge, or longer shapes fail closed; mode declaration
acceptance/inhabitation, inequality desugaring, implicit closure/order,
truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr, and VC remain
deferred.
Task 137 applies the builtin-object pre-desugaring inequality consumer to the
exact two-edge bare-object chain
`OuterTwoEdgeObjectModeInequality -> MiddleTwoEdgeObjectModeInequality -> BaseTwoEdgeObjectModeInequality -> object`.
Four raw outer-mode inputs retain written provenance while all three real
AST-derived expansions normalize them to one terminal-RHS builtin-object
identity before two `Inferred` variable terms and one fact-free pre-desugaring
`Checked` inequality are recorded. Missing/non-exact links and set-terminal,
direct, one-edge, or longer shapes fail closed; declaration
acceptance/inhabitation, inequality desugaring, implicit closure/order,
truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr, and VC remain
deferred.
Task 138 applies the normalized-reflexive type-assertion consumer to the exact
direct bare-set mode source
`LocalModeTypeAssertion -> set; reserve x for LocalModeTypeAssertion; theorem ...: x is set;`.
The raw subject result retains its written local-mode symbol and range, the
asserted builtin `set` retains its independent formula source node, and the one
real AST-derived expansion reaches `TermFormulaChecker`. Both inputs normalize
to one builtin-set identity canonically anchored at the definition RHS before
one `Inferred` variable term and one fact-free `Checked` type assertion are
recorded. Missing/non-exact expansions and formula-side local-mode asserted
heads fail closed; mode declaration acceptance/inhabitation, general
reachability/widening/`qua`, truth/facts, theorem acceptance, proof, CoreIr,
ControlFlowIr, and VC remain deferred.
Task 139 exact direct-local-mode left reserved-variable membership checker
bridge applies the right-only expected-`set` membership consumer to
`LocalModeMembership -> set; reserve x for LocalModeMembership; reserve y for set; theorem ...: x in y;`.
The raw left result retains its written local-mode symbol/range, while the right
result and sole expected-`set` input retain their independent explicit reserve
range. The one real AST-derived expansion normalizes the left result; the two
right builtin-set roles normalize directly, and all three intern to one
builtin-set identity canonically anchored at the earlier definition RHS.
Production validation requires `BindingId(0/1)`, two `Inferred` variables, one
fact-free `Checked` membership, exactly one right-owned expected constraint,
and no left expected input. Independent expansion and right expected-`set`
corruption tests report the task-specific invalid-payload key. Missing/non-
exact modes, reserves, or formulas fail closed; mode declaration
acceptance/inhabitation, membership truth/facts, implicit closure/order,
theorem acceptance, proof, CoreIr, ControlFlowIr, and VC remain deferred.
Task 140 exact direct local-object-mode left reserved-variable membership
checker bridge applies Task 125's right-only expected-`set`, two-binding
membership consumer to
`LocalObjectModeMembership -> object; reserve x for LocalObjectModeMembership; reserve y for set; theorem ...: x in y;`.
The raw left result retains its written local object-mode symbol/range, while
the right result and sole expected-`set` input retain their independent
explicit reserve range. The one real AST-derived expansion normalizes the left
result to a builtin-object identity canonically anchored at the definition
RHS; the two right roles normalize directly to one distinct builtin-set
identity anchored at their explicit reserve. Production validation requires
`BindingId(0/1)`, two `Inferred` variables, one fact-free `Checked` membership,
exactly one right-owned expected constraint, no left expected input, and no
object/set coercion. Independent expansion and right expected-`set` corruption
tests report the task-specific invalid-payload key. Missing/non-exact modes,
reserves, or formulas fail closed; mode declaration acceptance/inhabitation,
membership truth/facts, implicit closure/order, theorem acceptance, proof,
CoreIr, ControlFlowIr, and VC remain deferred.
Task 141 exact one-edge local-mode-chain left reserved-variable membership
checker bridge applies Task 139's right-only expected-`set`, two-binding
membership consumer to
`ChainModeMembership -> BaseModeMembership -> set; reserve x for ChainModeMembership; reserve y for set; theorem ...: x in y;`.
The raw left result retains its written outer-mode symbol/range, while the
right result and sole expected-`set` input retain their independent explicit
reserve range. Both real AST-derived expansions recursively normalize the left
result to one builtin-set identity canonically anchored at the terminal `set`
RHS; the two right roles normalize directly and intern to that identity.
Production validation requires `BindingId(0/1)`, two `Inferred` variables, one
fact-free `Checked` membership, exactly one right-owned expected constraint,
and no left expected input. Independent corruption of either chain link and
the right expected-`set` projection reports the task-specific invalid-payload
key. Missing/non-exact modes, reserves, or formulas fail closed; mode
declaration acceptance/inhabitation, membership truth/facts, implicit
closure/order, theorem acceptance, proof, CoreIr, ControlFlowIr, and VC remain
deferred.
Task 142 exact one-edge local-object-mode-chain left reserved-variable
membership checker bridge applies the same right-only expected-`set`,
two-binding membership consumer to
`ChainObjectModeMembership -> BaseObjectModeMembership -> object; reserve x for ChainObjectModeMembership; reserve y for set; theorem ...: x in y;`.
The raw left result retains its written outer-mode symbol/range, while the
right result and sole expected-`set` input retain their independent explicit
reserve range. Both real AST-derived expansions recursively normalize the left
result to one builtin-object identity canonically anchored at the terminal
`object` RHS; the two right roles normalize directly to one distinct
explicit-reserve-anchored builtin-set identity. Production validation requires
`BindingId(0/1)`, two `Inferred` variables, one fact-free `Checked` membership,
exactly one right-owned expected constraint, no left expected input, and no
object/set coercion. Independent corruption of either chain link and the right
expected-`set` projection must report the task-specific invalid-payload key.
Missing/non-exact modes, reserves, or formulas fail closed; mode declaration
acceptance/inhabitation, membership truth/facts, implicit closure/order,
theorem acceptance, proof, CoreIr, ControlFlowIr, and VC remain deferred.
Task 143 specifies the exact two-edge set-terminal local-mode-chain left
reserved-variable membership bridge. The source contains three unique,
unrecovered, source-preceding, no-argument/attribute definitions
`OuterTwoEdgeModeMembership -> MiddleTwoEdgeModeMembership -> BaseTwoEdgeModeMembership -> set`,
then ordered reserves `x` for the outer mode and `y` for explicit `set`, and
`TwoEdgeLocalModeReservedVariableMembershipPayloadBoundary: x in y;`. The raw
left result retains the outer symbol/range, while the right result and sole
expected-set input retain the independent explicit reserve range. All three
real Task 72 expansions must recursively normalize the left role; the right
builtin-set roles normalize directly, and all three intern to one
terminal-RHS-anchored builtin-set identity. The exact contract requires
`BindingId(0/1)`, two `Inferred` terms, one fact-free `Checked` membership,
exactly one right-owned expected constraint, and no left expected type. Each
definition label, both chain radices, all three expansion entries, and the
right expected-set projection are guarded independently. Missing or non-exact
definitions, reserves, or formulas fail closed. Mode declaration
acceptance/inhabitation, membership truth/facts, implicit closure/order,
theorem acceptance, proof, CoreIr, ControlFlowIr, and VC remain deferred.
Task 144 specifies the exact two-edge object-terminal local-mode-chain left
reserved-variable membership bridge. The source contains three unique,
unrecovered, source-preceding, no-argument/attribute definitions
`OuterTwoEdgeObjectModeMembership -> MiddleTwoEdgeObjectModeMembership -> BaseTwoEdgeObjectModeMembership -> object`,
then ordered reserves `x` for the outer mode and `y` for explicit `set`, and
`TwoEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`.
The raw left result retains the outer symbol/range, while the right result and
sole expected-set input retain the independent explicit reserve range. All
three real Task 72 expansions must recursively normalize the left role to one
terminal-RHS-anchored builtin-object identity; the right builtin-set roles
normalize directly to one distinct explicit-reserve-anchored identity. The
exact contract requires `BindingId(0/1)`, two `Inferred` terms, one fact-free
`Checked` membership, exactly one right-owned expected constraint, no left
expected type, and no object/set coercion. Each definition label, both chain
radices, all three expansion entries, and the right expected-set projection
are guarded independently. Missing or non-exact definitions, reserves, or
formulas fail closed. Mode declaration acceptance/inhabitation, membership
truth/facts, implicit closure/order, theorem acceptance, proof, CoreIr,
ControlFlowIr, and VC remain deferred.
Task 145 specifies the exact direct bare-object local-mode reserved-variable
normalized-reflexive type assertion bridge
`LocalObjectModeTypeAssertion -> object; reserve x for LocalObjectModeTypeAssertion; theorem ...: x is object;`.
The raw subject result retains its written local-mode symbol/range, while the
asserted builtin `object` retains its independent formula source node. The one
real Task 55 expansion must reach `TermFormulaChecker`; both inputs normalize
to one builtin-object identity canonically anchored at the definition RHS
before one `Inferred` variable term and one fact-free `Checked` type assertion
are recorded. The exact contract requires `BindingId(0)` and source-order use
ordinal 1. The definition label and expansion entry are guarded independently;
missing or non-exact definitions, reserves, and formulas fail closed. Mode
declaration acceptance/inhabitation, formula-side local-mode asserted-head
extraction, general reachability/widening/`qua`, object/set coercion,
truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr, and VC remain
deferred.
Task 146 specifies the exact one-edge bare-set local-mode-chain reserved-
variable normalized-reflexive type assertion bridge
`BaseModeTypeAssertion -> set; ChainModeTypeAssertion -> BaseModeTypeAssertion; reserve x for ChainModeTypeAssertion; theorem ...: x is set;`.
The raw subject result retains its written outer-mode symbol/range, while the
asserted builtin `set` retains its independent formula source node. Both real
Task 56 expansions must reach `TermFormulaChecker`; the inputs recursively
normalize to one builtin-set identity canonically anchored at the terminal
definition RHS before one `Inferred` variable term and one fact-free `Checked`
type assertion are recorded. The exact contract requires `BindingId(0)` and
source-order use ordinal 1. Both definition labels, the chain radix, and both
expansion entries are guarded independently; missing or non-exact definitions,
reserves, and formulas fail closed. Mode declaration acceptance/inhabitation,
formula-side local-mode asserted-head extraction, general reachability/
widening/`qua`, truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr,
and VC remain deferred.
Task 147 specifies the exact one-edge bare-object local-mode-chain reserved-
variable normalized-reflexive type assertion bridge
`BaseObjectModeTypeAssertion -> object; ChainObjectModeTypeAssertion -> BaseObjectModeTypeAssertion; reserve x for ChainObjectModeTypeAssertion; theorem ...: x is object;`.
The raw subject result retains its written outer-mode symbol/range, while the
asserted builtin `object` retains its independent formula source node. Both
real Task 56 expansions must reach `TermFormulaChecker`; the inputs recursively
normalize to one builtin-object identity canonically anchored at the terminal
definition RHS before one `Inferred` variable term and one fact-free `Checked`
type assertion are recorded. The exact contract requires `BindingId(0)` and
source-order use ordinal 1. Both definition labels, the chain radix, and both
expansion entries are guarded independently; missing or non-exact definitions,
reserves, and formulas fail closed. Mode declaration acceptance/inhabitation,
formula-side local-mode asserted-head extraction, general reachability/
widening/`qua`, object/set coercion, truth/facts, theorem acceptance, proof,
CoreIr, ControlFlowIr, and VC remain deferred.
Task 148 specifies only the exact two-edge bare-set
local-mode-chain reserved-variable normalized-reflexive type assertion bridge
`BaseTwoEdgeModeTypeAssertion -> set; MiddleTwoEdgeModeTypeAssertion -> BaseTwoEdgeModeTypeAssertion; OuterTwoEdgeModeTypeAssertion -> MiddleTwoEdgeModeTypeAssertion; reserve x for OuterTwoEdgeModeTypeAssertion; theorem ...: x is set;`.
The raw subject result retains its written outer-mode symbol/range, while
the asserted builtin `set` retains its independent formula source node. All
three real Task 72 expansions reach `TermFormulaChecker`; the inputs
recursively normalize to one builtin-set identity canonically anchored at the
terminal definition RHS before one `Inferred` variable term and one fact-free
`Checked` type assertion are recorded. The exact contract requires
`BindingId(0)` and source-order use ordinal 1. All three definition labels,
both chain radices, and all three expansion entries are guarded independently;
missing or non-exact definitions, reserves, and formulas fail closed. Mode
declaration acceptance/inhabitation, formula-side local-mode asserted-head
extraction, general reachability/widening/`qua`, truth/facts, theorem
acceptance, proof, CoreIr, ControlFlowIr, and VC remain deferred.
Task 149 specifies only the exact two-edge bare-object local-mode-chain
reserved-variable normalized-reflexive type assertion bridge
`BaseTwoEdgeObjectModeTypeAssertion -> object; MiddleTwoEdgeObjectModeTypeAssertion -> BaseTwoEdgeObjectModeTypeAssertion; OuterTwoEdgeObjectModeTypeAssertion -> MiddleTwoEdgeObjectModeTypeAssertion; reserve x for OuterTwoEdgeObjectModeTypeAssertion; theorem ...: x is object;`.
The raw subject result must retain its written outer-mode symbol/range, while
the asserted builtin `object` retains its independent formula source node. All
three real Task 72 expansions must reach `TermFormulaChecker`; the inputs must
recursively normalize to one builtin-object identity canonically anchored at
the terminal definition RHS before one `Inferred` variable term and one fact-
free `Checked` type assertion are recorded. The exact contract requires
`BindingId(0)` and source-order use ordinal 1. All three definition labels,
both chain radices, and all three expansion entries require independent guards;
missing or non-exact definitions, reserves, and formulas fail closed. Mode
declaration acceptance/inhabitation, formula-side local-mode asserted-head
extraction, general reachability/widening/`qua`, object/set coercion, truth/
facts, theorem acceptance, proof, CoreIr, ControlFlowIr, and VC remain
deferred. Exact source guards, independent definition/three-link corruption,
and the real frontend/resolver sidecar protect the production route.
Task 150 adds only the exact three-edge bare-set local-mode-chain
reserved-variable normalized-reflexive type assertion bridge
`BaseThreeEdgeModeTypeAssertion -> set; InnerThreeEdgeModeTypeAssertion -> BaseThreeEdgeModeTypeAssertion; MiddleThreeEdgeModeTypeAssertion -> InnerThreeEdgeModeTypeAssertion; OuterThreeEdgeModeTypeAssertion -> MiddleThreeEdgeModeTypeAssertion; reserve x for OuterThreeEdgeModeTypeAssertion; theorem ...: x is set;`.
The raw subject result must retain its written outer-mode symbol/range, while
the asserted builtin `set` retains its independent formula source node. All
four real Task 73 expansions must reach `TermFormulaChecker`; both inputs must
recursively normalize to one builtin-set identity canonically anchored at the
terminal definition RHS before one `Inferred` variable term and one fact-free
`Checked` type assertion are recorded. The exact contract requires
`BindingId(0)` and source-order use ordinal 1. All four definition labels,
all three chain radices, and all four expansion entries require independent
guards; missing or non-exact definitions, reserves, and formulas fail closed.
Mode declaration acceptance/inhabitation, formula-side local-mode asserted-
head extraction, general reachability/widening/`qua`, truth/facts, theorem
acceptance, proof, CoreIr, ControlFlowIr, and VC remain deferred. Exact source
guards, independent definition/four-link corruption, and the real frontend/
resolver sidecar protect the active production route.
Task 151 adds only the exact three-edge bare-object local-mode-chain
reserved-variable normalized-reflexive type assertion bridge
`BaseThreeEdgeObjectModeTypeAssertion -> object; InnerThreeEdgeObjectModeTypeAssertion -> BaseThreeEdgeObjectModeTypeAssertion; MiddleThreeEdgeObjectModeTypeAssertion -> InnerThreeEdgeObjectModeTypeAssertion; OuterThreeEdgeObjectModeTypeAssertion -> MiddleThreeEdgeObjectModeTypeAssertion; reserve x for OuterThreeEdgeObjectModeTypeAssertion; theorem ...: x is object;`.
The raw subject result must retain its written outer-mode symbol/range, while
the asserted builtin `object` retains its independent formula source node. All
four real Task 73 expansions must reach `TermFormulaChecker`; both inputs must
recursively normalize to one builtin-object identity canonically anchored at
the terminal definition RHS before one `Inferred` variable term and one fact-
free `Checked` type assertion are recorded. The exact contract requires
`BindingId(0)` and source-order use ordinal 1. All four definition labels, all
three chain radices, and all four expansion entries require independent guards;
missing or non-exact definitions, reserves, and formulas fail closed. Mode
declaration acceptance/inhabitation, formula-side local-mode asserted-head
extraction, general reachability/widening/`qua`, object/set coercion, truth/
facts, theorem acceptance, proof, CoreIr, ControlFlowIr, and VC remain deferred.
Exact source guards, independent definition/four-link corruption, and the real
frontend/resolver sidecar protect the active production route.
Task 152 adds only the exact four-edge bare-set local-mode-chain reserved-
variable normalized-reflexive type assertion bridge
`BaseFourEdgeModeTypeAssertion -> set; InnerFourEdgeModeTypeAssertion -> BaseFourEdgeModeTypeAssertion; MiddleFourEdgeModeTypeAssertion -> InnerFourEdgeModeTypeAssertion; OuterFourEdgeModeTypeAssertion -> MiddleFourEdgeModeTypeAssertion; TooDeepFourEdgeModeTypeAssertion -> OuterFourEdgeModeTypeAssertion; reserve x for TooDeepFourEdgeModeTypeAssertion; theorem ...: x is set;`.
The raw subject result must retain its written outermost-mode symbol/range,
while the asserted builtin `set` retains its independent formula source node.
All five real Task 74 expansions must reach `TermFormulaChecker`; both inputs
must recursively normalize to one builtin-set identity canonically anchored at
the terminal definition RHS before one `Inferred` variable term and one fact-
free `Checked` type assertion are recorded. The exact contract requires
`BindingId(0)` and source-order use ordinal 1. All five definition labels, all
four chain radices, and all five expansion entries require independent guards;
missing or non-exact definitions, reserves, and formulas fail closed. Mode
declaration acceptance/inhabitation, formula-side local-mode asserted-head
extraction, general reachability/widening/`qua`, truth/facts, theorem
acceptance, proof, CoreIr, ControlFlowIr, and VC remain deferred. Exact source
guards, independent definition/five-link corruption, and the real frontend/
resolver sidecar protect the active production route.
Task 153 adds only the exact four-edge bare-object local-mode-chain reserved-
variable normalized-reflexive type assertion bridge
`BaseFourEdgeObjectModeTypeAssertion -> object; InnerFourEdgeObjectModeTypeAssertion -> BaseFourEdgeObjectModeTypeAssertion; MiddleFourEdgeObjectModeTypeAssertion -> InnerFourEdgeObjectModeTypeAssertion; OuterFourEdgeObjectModeTypeAssertion -> MiddleFourEdgeObjectModeTypeAssertion; TooDeepFourEdgeObjectModeTypeAssertion -> OuterFourEdgeObjectModeTypeAssertion; reserve x for TooDeepFourEdgeObjectModeTypeAssertion; theorem ...: x is object;`.
The raw subject result must retain its written outermost-mode symbol/range,
while the asserted builtin `object` retains its independent formula source
node. All five real Task 74 expansions must reach `TermFormulaChecker`; both
inputs must recursively normalize to one builtin-object identity canonically
anchored at the terminal definition RHS before one `Inferred` variable term
and one fact-free `Checked` type assertion are recorded. The exact contract
requires `BindingId(0)` and source-order use ordinal 1. All five definition
labels, all four chain radices, and all five expansion entries require
independent guards; missing or non-exact definitions, reserves, and formulas
fail closed. Mode declaration acceptance/inhabitation, formula-side local-mode
asserted-head extraction, general reachability/widening/`qua`, object/set
coercion, truth/facts, theorem acceptance, proof, CoreIr, ControlFlowIr, and VC
remain deferred. Exact source guards, independent definition/five-link
corruption, and the real frontend/resolver sidecar protect the active route.
Task 154 adds only the exact three-edge bare-set local-mode-chain reserved-
variable equality bridge
`BaseThreeEdgeModeEquality -> set; InnerThreeEdgeModeEquality -> BaseThreeEdgeModeEquality; MiddleThreeEdgeModeEquality -> InnerThreeEdgeModeEquality; OuterThreeEdgeModeEquality -> MiddleThreeEdgeModeEquality; reserve z for OuterThreeEdgeModeEquality; theorem ...: z = z;`.
All four raw result/expected inputs retain their written outer-mode source;
both identifier uses resolve independently to `BindingId(0)` at source-order
ordinals 1 and 2. The four real Task 73 expansions recursively normalize all
roles to one builtin-set identity canonically anchored at the terminal RHS
before two `Inferred` variable terms and one fact/deferred-free `Checked`
equality are recorded. Every definition label, every chain radix, and every
expansion entry requires an independent guard; non-exact definition, reserve,
formula, or withheld chain/terminal shapes fail closed. This is equality type/
well-formedness only: mode declaration acceptance/inhabitation, equality truth
or facts, implicit closure/order, theorem acceptance, proof, CoreIr,
ControlFlowIr, and VC remain deferred. The exact source guards, independent
definition/radix/expansion corruption matrix, production route, and real
frontend/resolver sidecar now protect the active bridge.
Task 155 adds only the exact three-edge bare-object local-mode-chain reserved-
variable equality bridge
`BaseThreeEdgeObjectModeEquality -> object; InnerThreeEdgeObjectModeEquality -> BaseThreeEdgeObjectModeEquality; MiddleThreeEdgeObjectModeEquality -> InnerThreeEdgeObjectModeEquality; OuterThreeEdgeObjectModeEquality -> MiddleThreeEdgeObjectModeEquality; reserve z for OuterThreeEdgeObjectModeEquality; theorem ...: z = z;`.
All four raw result/expected inputs retain their written outer-mode source;
both identifier uses resolve independently to `BindingId(0)` at source-order
ordinals 1 and 2. The four real Task 73 expansions recursively normalize all
roles to one builtin-object identity canonically anchored at the terminal RHS
before two `Inferred` variable terms and one fact/deferred-free `Checked`
equality are recorded. Every definition label, every chain radix, and every
expansion entry requires an independent guard; non-exact definition, reserve,
formula, or withheld chain/terminal shapes fail closed. This is equality type/
well-formedness only: mode declaration acceptance/inhabitation, object/set
coercion, equality truth or facts, implicit closure/order, theorem acceptance,
proof, CoreIr, ControlFlowIr, and VC remain deferred. Exact source guards, the
independent definition/radix/expansion corruption matrix, production route,
and real frontend/resolver sidecar now protect the active bridge.
Task 156 adds only the exact three-edge bare-set local-mode-chain reserved-
variable inequality bridge
`BaseThreeEdgeModeInequality -> set; InnerThreeEdgeModeInequality -> BaseThreeEdgeModeInequality; MiddleThreeEdgeModeInequality -> InnerThreeEdgeModeInequality; OuterThreeEdgeModeInequality -> MiddleThreeEdgeModeInequality; reserve z for OuterThreeEdgeModeInequality; theorem ...: z <> z;`.
All four raw result/expected inputs must retain their written outer-mode source;
both identifier uses must resolve independently to `BindingId(0)` at source-
order ordinals 1 and 2. The four real Task 73 expansions must recursively
normalize all roles to one builtin-set identity canonically anchored at the
terminal RHS before two `Inferred` variable terms and one fact/deferred-free
pre-desugaring `Checked` inequality are recorded. Every definition label,
every chain radix, and every expansion entry requires an independent guard;
non-exact definition, reserve, formula, or withheld chain/terminal shapes must
fail closed. This is inequality type/well-formedness only: mode declaration
acceptance/inhabitation, inequality desugaring, truth or facts, implicit
closure/order, theorem acceptance, proof, CoreIr, ControlFlowIr, and VC remain
deferred. Exact source guards, the independent definition/radix/expansion
corruption matrix, production route, and real frontend/resolver sidecar now
protect the active bridge.
Task 157 adds only the exact three-edge bare-object local-mode-chain reserved-
variable inequality bridge
`BaseThreeEdgeObjectModeInequality -> object; InnerThreeEdgeObjectModeInequality -> BaseThreeEdgeObjectModeInequality; MiddleThreeEdgeObjectModeInequality -> InnerThreeEdgeObjectModeInequality; OuterThreeEdgeObjectModeInequality -> MiddleThreeEdgeObjectModeInequality; reserve z for OuterThreeEdgeObjectModeInequality; theorem ...: z <> z;`.
All four raw result/expected inputs must retain their written outer-mode source;
both identifier uses must resolve independently to `BindingId(0)` at source-
order ordinals 1 and 2. The four real Task 73 expansions must recursively
normalize all roles to one builtin-object identity canonically anchored at the
terminal RHS before two `Inferred` variable terms and one fact/deferred-free
pre-desugaring `Checked` inequality are recorded. Every definition label,
every chain radix, and every expansion entry requires an independent guard;
non-exact definition, reserve, formula, or withheld chain/terminal shapes must
fail closed. This is inequality type/well-formedness only: mode declaration
acceptance/inhabitation, object/set coercion, inequality desugaring, truth or
facts, implicit closure/order, theorem acceptance, proof, CoreIr,
ControlFlowIr, and VC remain deferred. The source and trace contract,
production route, independent corruption matrix, and real frontend/resolver
sidecar now protect the active bridge.
Task 158 specifies only the exact three-edge bare-set local-mode-chain left
reserved-variable membership bridge
`BaseThreeEdgeModeMembership -> set; InnerThreeEdgeModeMembership -> BaseThreeEdgeModeMembership; MiddleThreeEdgeModeMembership -> InnerThreeEdgeModeMembership; OuterThreeEdgeModeMembership -> MiddleThreeEdgeModeMembership; reserve x for OuterThreeEdgeModeMembership; reserve y for set; theorem ...: x in y;`.
The raw left result retains the written outer-mode source, while the right
result and sole expected input retain the independent explicit-set reserve
source; no left expected input exists. The checker resolves `x/y` independently
to `BindingId(0/1)` at source-order ordinals 2/3. All four real task-73
expansions normalize those three roles to one builtin-set identity canonically
anchored at the terminal RHS before two `Inferred` variable terms and one fact/
deferred-free `Checked` membership with exactly one right-owned expected-type
constraint are recorded. Every definition label/radix and expansion entry
requires an independent guard; non-exact chain, terminal, reserve, formula, or
withheld shapes fail closed. Mode declaration acceptance/inhabitation,
membership truth/facts, implicit closure/order, theorem acceptance, proof,
CoreIr, ControlFlowIr, VC, object-terminal behavior, and broader chain depths
remain deferred. The source/trace contract, production route, independent
corruption matrix, and real frontend/resolver sidecar now protect the active
bridge.
Task 159 specifies only
`reserve x, y for set; theorem DistinctReservedVariableMembershipPayloadBoundary: x in y;`.
One reserve item creates distinct `BindingId(0/1)` values with one shared
written set range; independent lookups use ordinals 2/3. The left/right result
and sole right expected roles retain that range, no left expected role exists,
and the checker records one normalized builtin-set identity, two `Inferred`
variables, and one fact/deferred-free `Checked` membership with one right-owned
constraint. Exact source and matched-output corruption guards are required.
Membership truth/facts, closure/order, theorem acceptance, proof/Core/VC,
separate declarations, and broader shapes remain deferred. The source/trace
contract, production route, independent corruption matrix, and real frontend/
resolver sidecar now protect the active bridge.
Task 160 specifies only the source
`reserve x, y for set; theorem DistinctReservedVariableInequalityPayloadBoundary: x <> y;`.
One reserve item must create distinct `BindingId(0/1)` values with one shared
written set range and independent lookups at ordinals 2/3. Both operand results
and both operand-owned expected roles must retain that range, normalize to one
shared-source builtin-set identity, and produce two `Inferred` variables plus
one fact/deferred-free pre-desugaring `Checked` inequality with two ordered
constraints. Exact source, route-order, matched-output, and corruption guards
are required. Inequality desugaring/truth/facts, closure/order, theorem
acceptance, proof/Core/VC, separate declarations, and broader shapes remain
deferred. The source/trace contract, production route, independent corruption
coverage, and real frontend/resolver sidecar now protect the active bridge.
Task 161 specifies only the source `reserve x for set; reserve y for
set; theorem MultipleReserveDeclarationInequalityPayloadBoundary: x <> y;`.
Two reserve items must create distinct `BindingId(0/1)` values with distinct
written set ranges and independent lookups at ordinals 2/3. Each operand result
and expected role must retain its binding's range, while all four roles
normalize to one canonical builtin-set identity anchored at the earlier `x`
range before two `Inferred` variables and one fact/deferred-free pre-desugaring
`Checked` inequality with two ordered constraints. Exact source, route-order,
matched-output, and corruption guards are required. Desugaring/truth/facts,
closure/order, theorem acceptance, proof/Core/VC, shared ranges, and broader
shapes remain deferred. The source/trace contract, production route,
corruption coverage, and real sidecar now protect the active bridge.
Task 162 specifies only the active source `reserve x for set; reserve y for
set; theorem MultipleReserveDeclarationMembershipPayloadBoundary: x in y;`.
Two reserve items must create distinct `BindingId(0/1)` values with distinct
written set ranges and independent lookups at ordinals 2/3. The left result
retains the first range; the right result and sole right expected role retain
the second; no left expected role exists. All three roles normalize to one
canonical builtin-set identity anchored at the earlier `x` range before two
`Inferred` variables and one fact/deferred-free `Checked` membership with
exactly one right-owned constraint. Exact source, route-order, matched-output,
and corruption guards are required. Membership truth/facts, closure/order,
theorem acceptance, proof/Core/VC, shared ranges, and broader shapes remain
deferred. The production route, independent corruption/near-miss coverage, and
real frontend/resolver sidecar now implement and guard the contract, so the
active count is 113.
Task 163 specifies only the active source whose four unique, unrecovered,
same-module, argument-free, source-preceding mode definitions form
`OuterThreeEdgeObjectModeMembership -> MiddleThreeEdgeObjectModeMembership ->
InnerThreeEdgeObjectModeMembership -> BaseThreeEdgeObjectModeMembership ->
object`, followed by `reserve x` for the outer mode, `reserve y for set`, and
`ThreeEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`.
The implementation must resolve `x/y` to `BindingId(0/1)` at ordinals 2/3,
preserve the raw outer-mode left result and independent explicit-set right
result/sole expected input, expose no left expected input, consume all four
real AST-derived expansions, and normalize to distinct terminal-object-RHS and
explicit-set identities. It must then require two `Inferred` variables and one
fact/deferred-free `Checked` membership with exactly one right-owned
constraint. Exact route-order, near-miss, matched-output, expansion corruption,
and real frontend/resolver sidecar guards protect the active route.
Membership truth/facts, object/set coercion, closure/order, theorem acceptance,
proof/Core/VC, other chain depths, and broader shapes remain deferred. The
active runner contains 114 cases.
Task 164 specifies only the active source whose five unique, unrecovered,
same-module, argument-free, source-preceding mode definitions form
`TooDeepFourEdgeModeMembership -> OuterFourEdgeModeMembership ->
MiddleFourEdgeModeMembership -> InnerFourEdgeModeMembership ->
BaseFourEdgeModeMembership -> set`, followed by `reserve x` for the outermost
mode, `reserve y for set`, and
`FourEdgeLocalModeReservedVariableMembershipPayloadBoundary: x in y;`.
The implementation must resolve `x/y` to `BindingId(0/1)` at ordinals 2/3,
preserve raw outermost-mode left provenance and independent explicit-set right
result/sole expected provenance, expose no left expected input, consume all
five real AST-derived expansions, and normalize all three type roles to one
terminal-set-RHS identity. It must require two `Inferred` variables and one
fact/deferred-free `Checked` membership with exactly one right-owned
constraint. Exact route-order, near-miss, matched-output, expansion-corruption,
and real frontend/resolver sidecar guards are required. Membership truth/facts,
closure/order, theorem acceptance, proof/Core/VC, object-terminal behavior,
other chain depths, and broader shapes remain deferred. The production route,
full corruption/near-miss coverage, and real sidecar now guard active runner
115.
Task 165 specifies only the active source whose five unique, unrecovered,
same-module, argument-free, source-preceding mode definitions form
`TooDeepFourEdgeObjectModeMembership -> OuterFourEdgeObjectModeMembership ->
MiddleFourEdgeObjectModeMembership -> InnerFourEdgeObjectModeMembership ->
BaseFourEdgeObjectModeMembership -> object`, followed by `reserve x` for the
outermost mode, `reserve y for set`, and
`FourEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`.
The implementation must resolve `x/y` to `BindingId(0/1)` at ordinals 2/3,
preserve raw outermost-mode left provenance and independent explicit-set right
result/sole expected provenance, expose no left expected input, consume all
five real AST-derived expansions, and normalize to distinct terminal-object-
RHS and explicit-set identities. It must require two `Inferred` variables and
one fact/deferred-free `Checked` membership with exactly one right-owned
constraint. Membership truth/facts, object/set coercion, closure/order, theorem
acceptance, proof/Core/VC, other chain depths, and broader shapes remain
deferred. The fixture, expectation, six trace backlinks, exact route-order,
near-miss, matched-output, expansion-corruption, and real frontend/resolver
sidecar guards now protect active runner 116.
Task 166 specifies only the active source whose five unique, unrecovered,
same-module, argument-free, source-preceding mode definitions form
`TooDeepFourEdgeModeEquality -> OuterFourEdgeModeEquality ->
MiddleFourEdgeModeEquality -> InnerFourEdgeModeEquality ->
BaseFourEdgeModeEquality -> set`, followed by one outermost-mode `reserve z`
and `FourEdgeLocalModeReservedVariableEqualityPayloadBoundary: z = z;`.
The implementation must resolve both uses to `BindingId(0)` at ordinals 1/2,
preserve all four raw result/expected inputs, consume all five real expansions,
and normalize every role to one terminal-set-RHS identity. It requires two
`Inferred` variables, one fact/deferred-free `Checked` equality, and two ordered
operand-owned expected constraints. Exact route-
order, per-definition/link/depth near misses, matched-output/expansion
corruption, and a real frontend/resolver sidecar are mandatory. Declaration
acceptance/inhabitation, equality truth/facts, closure/order, theorem acceptance,
proof/Core/VC, object-terminal behavior, other depths, and broader shapes stay
deferred. The fixture, expectation, six trace backlinks, exact routing, full
near-miss/corruption matrix, and real frontend/resolver sidecar now protect
active runner 117.
Task 167 specifies only the active source whose five unique, unrecovered,
same-module, argument-free, source-preceding mode definitions form
`TooDeepFourEdgeObjectModeEquality -> OuterFourEdgeObjectModeEquality ->
MiddleFourEdgeObjectModeEquality -> InnerFourEdgeObjectModeEquality ->
BaseFourEdgeObjectModeEquality -> object`, followed by one outermost-mode
`reserve z` and
`FourEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;`.
The implementation must resolve both uses to `BindingId(0)` at ordinals 1/2,
preserve all four raw result/expected inputs, consume all five real expansions,
and normalize every role to one terminal-object-RHS identity. It must require
two `Inferred` variables, one fact/deferred-free `Checked` equality, and two
ordered operand-owned expected constraints without object/set coercion. Exact
route-order, per-definition/link/depth near misses, matched-output/expansion
corruption, and a real frontend/resolver sidecar are mandatory. Declaration
acceptance/inhabitation, equality truth/facts, closure/order, theorem
acceptance, proof/Core/VC, set-terminal behavior, other depths, and broader
shapes stay deferred. The fixture, expectation, six trace backlinks, exact
routing, full near-miss/corruption matrix, and real frontend/resolver sidecar
now protect active runner 118.
Task 168 specifies the test-first source whose five unique, unrecovered,
same-module, argument-free, source-preceding mode definitions form
`TooDeepFourEdgeModeInequality -> OuterFourEdgeModeInequality ->
MiddleFourEdgeModeInequality -> InnerFourEdgeModeInequality ->
BaseFourEdgeModeInequality -> set`, followed by one outermost-mode `reserve z`
and `FourEdgeLocalModeReservedVariableInequalityPayloadBoundary: z <> z;`.
The implementation must resolve both uses to `BindingId(0)` at ordinals 1/2,
preserve four raw result/expected inputs, consume five real expansions, and
normalize every role to one terminal-set-RHS identity before two `Inferred`
variables, one fact/deferred-free pre-desugaring `Checked` inequality, and two
ordered operand-owned expected constraints. Exact route-order, near-miss,
matched-output, expansion-corruption, and real sidecar guards are mandatory.
Declaration acceptance/inhabitation, inequality desugaring/truth/facts,
closure/order, theorem acceptance, proof/Core/VC, object-terminal behavior,
other depths, and broader shapes stay deferred. Fixture/expectation, six trace
backlinks, exact routing, full near-miss/corruption coverage, and the real
frontend/resolver sidecar now protect active runner 119.
Task 169 specifies the test-first source whose five unique, unrecovered,
same-module, argument-free, source-preceding mode definitions form
`TooDeepFourEdgeObjectModeInequality -> OuterFourEdgeObjectModeInequality ->
MiddleFourEdgeObjectModeInequality -> InnerFourEdgeObjectModeInequality ->
BaseFourEdgeObjectModeInequality -> object`, followed by one outermost-mode
`reserve z` and
`FourEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary: z <> z;`.
The implementation must resolve both uses to `BindingId(0)` at ordinals 1/2,
preserve four raw result/expected inputs, consume five real expansions, and
normalize every role to one terminal-object-RHS identity before two `Inferred`
variables, one fact/deferred-free pre-desugaring `Checked` inequality, and two
ordered operand-owned expected constraints without object/set coercion. Exact
route-order, near-miss, matched-output, expansion-corruption, and real sidecar
guards are mandatory. Declaration acceptance/inhabitation, inequality
desugaring/truth/facts, closure/order, theorem acceptance, proof/Core/VC, set-
terminal behavior, other depths, and broader shapes stay deferred. Fixture/
expectation, six trace backlinks, exact routing, full near-miss/corruption
coverage, and the real frontend/resolver sidecar now protect active runner 120.
Task 172 specifies only the test-first source whose seven unique, unrecovered,
same-module, argument-free, source-preceding mode definitions form
`ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3 -> ChainMode2 ->
ChainMode1 -> BaseMode -> set`, followed by `reserve z for ChainMode6;` and
`LongLocalModeReservedVariableEqualityPayloadBoundary: z = z;`. The
implementation must consume all seven real AST-derived expansions, resolve
both uses to `BindingId(0)` at ordinals 1/2, retain four raw `ChainMode6`
result/expected inputs, and normalize every role to one identity anchored at
the terminal `BaseMode` builtin-set RHS. It must require two `Inferred`
variables, one fact/deferred-free `Checked` equality, and two ordered operand-
owned expected constraints. Exact route order, per-definition/link/order/depth/
recovery/context/parameterization near misses, matched-output and expansion
corruption, and a real frontend/resolver sidecar are mandatory. Declaration
acceptance/inhabitation, equality truth/facts, closure/order, theorem
acceptance, formula-statement/proof/Core/ControlFlow/VC payloads, imported/
attributed/argument-bearing or other chain shapes, and general unbounded
semantics remain deferred. The fixture, expectation, six trace backlinks,
production route, full near-miss/corruption matrix, and real frontend/resolver
sidecar now protect active runner 121.
Task 173 specifies the sibling exact source with the same seven definitions and
`reserve z for ChainMode6;`, followed only by
`LongLocalModeReservedVariableInequalityPayloadBoundary: z <> z;`. It must use
all seven real expansions and retain the same raw provenance, ordinal 1/2
`BindingId(0)`, terminal-`BaseMode`-RHS identity, two `Inferred` variables, and
two ordered operand-owned expected constraints before one fact/deferred-free
pre-desugaring `Checked` inequality. The Task 172 exact/near-miss/corruption/
real-sidecar guard breadth is mandatory. Inequality desugaring/truth/facts,
acceptance, closure/order, theorem/proof/Core/ControlFlow/VC, other chain shapes,
and general unbounded semantics remain deferred.
The fixture, expectation, six backlinks, exact routing, shared full guard
matrix, Task 173 corruption tests, and real sidecar now protect active runner
122.
Task 174 specifies the exact membership sibling with the same seven
definitions, followed by ordered `reserve x for ChainMode6;` and
`reserve y for set;` declarations and only
`LongLocalModeReservedVariableMembershipPayloadBoundary: x in y;`. The
production route must consume all seven real expansions while preserving the
raw `ChainMode6` left result and independent explicit-set right result and sole
right expected input. It must resolve ordinal 2/3 `BindingId(0/1)`, normalize
all three roles to one terminal-`BaseMode`-RHS builtin-set identity, leave the
left expected input absent, and record two `Inferred` variables plus one fact/
deferred-free `Checked` membership with exactly one right-owned constraint.
The Task 172 full structural guard matrix plus Task 174 membership-specific
matched-output corruption and real-sidecar tests are mandatory. Membership
truth/facts, acceptance, closure/order, theorem/proof/Core/ControlFlow/VC,
other chain shapes, and general unbounded semantics remain deferred. The
fixture, expectation, six backlinks, exact routing, shared full structural
guards, membership-specific corruption tests, and real sidecar now protect
active runner 123.
Task 175 specifies the exact normalized-reflexive type-assertion sibling with
the same seven definitions, one `reserve x for ChainMode6;`, and only
`LongLocalModeReservedVariableTypeAssertionPayloadBoundary: x is set;`. The
production route must consume all seven real expansions while preserving the
raw `ChainMode6` subject result and independent formula-side builtin-set
asserted input. It must resolve ordinal 1 `BindingId(0)`, normalize both roles
to one terminal-`BaseMode`-RHS builtin-set identity, and record one `Inferred`
variable plus one fact/deferred-free `Checked` type assertion without invoking
general reachability. The Task 172 full structural guard matrix plus Task 175
type-assertion-specific matched-output corruption and real-sidecar tests are
mandatory. Widening/`qua`, assertion truth/facts, acceptance, closure/order,
theorem/proof/Core/ControlFlow/VC, other chain shapes, and general unbounded
semantics remain deferred. The test-first fixture, expectation, and seven
backlinks plus production routing, type-assertion-specific corruption coverage,
and the real sidecar now protect active runner 124.
Task 176 specifies only the exact seven-definition builtin-object-terminal
long-chain sibling ending in
`LongLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;`. The
production route composes Task 74's AST-bounded source producer
with Task 167's equality consumer, consumes all seven real expansions, preserves
four raw `ChainObjectMode6` result/expected inputs, resolves ordinal 1/2
`BindingId(0)`, and normalizes all roles to one terminal-`BaseObjectMode`-RHS
builtin-object identity. It must record two `Inferred` terms, two ordered
operand-owned constraints, and one fact/deferred-free `Checked` equality
without object/set coercion. Task 172's shared seven-definition structural
guard pattern plus Task 176 object-terminal/matched-output corruption and real-
sidecar tests are mandatory. Truth/facts, acceptance, closure/order, theorem/
proof/Core/ControlFlow/VC, other chain shapes, and general semantics remain
deferred. The test-first fixture, expectation, and six backlinks are present;
production routing, object-specific corruption coverage, and the real sidecar
now protect active runner 125.
Task 177 specifies only the matching seven-definition builtin-object-terminal
long-chain sibling ending in
`LongLocalObjectModeReservedVariableInequalityPayloadBoundary: z <> z;`. The
production route composes Task 74's AST-bounded source producer with Task 169's
pre-desugaring inequality consumer, consumes all seven real expansions,
preserves four raw `ChainObjectMode6` result/expected inputs, resolves ordinal
1/2 `BindingId(0)`, and normalizes all roles to one terminal-
`BaseObjectMode`-RHS builtin-object identity. It records two `Inferred` terms,
two ordered operand-owned constraints, and one fact/deferred-free pre-desugaring `Checked`
inequality without object/set coercion. Task 172's shared seven-definition
structural guard pattern plus Task 177 object-terminal/matched-output corruption
and real-sidecar tests are mandatory. Inequality desugaring, truth/facts,
acceptance, closure/order, theorem/proof/Core/ControlFlow/VC, other chain shapes,
and general semantics remain deferred. The test-first fixture, expectation, and
six backlinks plus production routing, object-specific corruption coverage, and
the real sidecar now protect active runner 126.
Task 178 specifies only the matching seven-definition builtin-object-terminal
left-membership sibling ending in
`LongLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`. The
production route must compose Task 74's AST-bounded chain producer with Task
165's object-left/set-right membership consumer, consume all seven real
expansions, preserve the raw `ChainObjectMode6` left result and independent
explicit-set right result/sole right expected input, resolve ordinal 2/3
`BindingId(0/1)`, and normalize them to distinct terminal-`BaseObjectMode`-RHS
builtin-object and explicit-set identities. It must record no left expected
input, two `Inferred` terms, exactly one right-owned constraint, and one fact/
deferred-free `Checked` membership without object/set coercion. Task 172's
shared seven-definition structural guard pattern plus Task 178 membership/
object-specific matched-output corruption and real-sidecar tests protect the route.
Truth/facts, acceptance, closure/order, theorem/proof/Core/ControlFlow/VC, other
chain shapes, and general semantics remain deferred. The fixture, expectation,
six backlinks, production routing, full guards, and real sidecar protect active
runner 127.
Task 179 specifies only the matching normalized-reflexive type-assertion sibling
ending in
`LongLocalObjectModeReservedVariableTypeAssertionPayloadBoundary: x is object;`.
The production route composes Task 74's AST-bounded chain producer with
Task 153's object-normalizing type-assertion consumer, consumes all seven real
expansions, preserves the raw `ChainObjectMode6` subject result and independent
formula-side builtin-object asserted input, resolves ordinal 1 `BindingId(0)`,
and normalizes both roles to one terminal-`BaseObjectMode`-RHS builtin-object
identity. It records one `Inferred` term and one fact/deferred-free `Checked`
type assertion without general reachability or object/set coercion. Task 172's
shared seven-definition structural guard pattern and Task 153's real object
consumer/source near misses are reused. Task 175's matched-output guards are
adapted to reject a builtin-set asserted head and corrupted raw
`ChainObjectMode6` subject provenance; a real-sidecar test protects the route.
Widening/`qua`, truth/facts, acceptance, closure/order, theorem/
proof/Core/ControlFlow/VC, other chain shapes, and general semantics remain
deferred. The fixture, expectation, seven trace references, production routing,
full guards, and real sidecar protect active runner 128.
Task 180 specifies the exact formula-leaf sibling
`theorem SourceDerivedContradictionConstantBoundary: contradiction;`. The
source route adds a standalone exact leaf extractor by reusing Task 117's
contradiction-kind mapping and Task 115's standalone theorem-shape validation
pattern, then passes this leaf to the existing `FormulaKind::Contradiction`
consumer without a synthetic missing-payload reason. The result must preserve
the exact source site/range and module-root
context and contain one `Checked` formula with no terms, asserted type,
expected constraints, candidates, facts, deferred reasons, or diagnostics.
This status means formula type/well-formedness checking only. It does not
publish falsehood or facts, accept the theorem, close a proof goal, extract an
implicit closure or child graph, activate `formula_statement`, or create
proof/CoreIr/ControlFlowIr/VC payloads. Exact-source, near-miss/corruption, and
real parser/resolver-sidecar guards must protect the route.
Task 181 is an exactness repair, not a semantic promotion. When the reserve
source bridge observes a real imported `parser.type_fixtures` attribute, it
must accept only the five already credited sources: the four single-binding
shapes positive `TypeCaseAttr set`, positive or negative `empty set`, and
negative `empty object`, plus the ordered mixed source `reserve x for set;
reserve y for non empty set;`. The exact-source gate must require one
argument-free attribute on each attributed binding and no unrelated top-level
item. Duplicate or mixed attributes, wrong polarity/head, and multiple
bindings/items outside that exact mixed source remain on the source extraction
gap. The five existing fail-closed fixtures retain their evidence-query
expectations; this repair does
not add a `.miz` case, accept attributed types, provide evidence, or promote
positive `empty object`.
Task 182 adds the first formula-side local-mode asserted head. The exact
source contains a `definition` block with `mode LocalModeAssertedHeadDef:
LocalModeAssertedHead is set;`, one `reserve x for LocalModeAssertedHead;`, and only
`LocalModeAssertedHeadPayloadBoundary: x is LocalModeAssertedHead;`. The
producer preserves two independent raw type-expression inputs: the reserve-
owned subject result and the formula-owned asserted type have distinct sites
and ranges while resolving to the same real local-mode symbol. One real AST-
derived mode expansion normalizes all three known type entries to one
builtin-set identity canonically anchored at the definition RHS. The prepared
type-assertion consumer resolves ordinal 1 to `BindingId(0)`, records one
`Inferred` variable and one fact/deferred-free normalized-reflexive `Checked`
formula, and uses no general reachability payload. Exact-source, near-miss,
corruption, production-route, and real parser/resolver-sidecar guards
reject builtin or other-mode asserted heads, attributed/argument-bearing heads,
object terminals, chains, recovery, extra items, and collapsed provenance.
This is type/well-formedness only; mode declaration acceptance/inhabitation,
widening/`qua`, truth/facts, theorem acceptance, proof/CoreIr/ControlFlowIr/VC,
other asserted-head families, and general semantics remain deferred.
Task 183 adds the direct object-terminal sibling. The exact source contains
one definition block with `mode LocalObjectModeAssertedHeadDef:
LocalObjectModeAssertedHead is object;`, one matching reserve, and only
`LocalObjectModeAssertedHeadPayloadBoundary: x is LocalObjectModeAssertedHead;`.
The producer retains independent raw reserve-subject and formula-side
asserted type-expression inputs with distinct sites/ranges and the same resolved
mode symbol. One real AST-derived expansion normalizes all three known type
entries to one builtin-object identity canonically anchored at the definition
RHS. The prepared consumer resolves ordinal 1 to `BindingId(0)`, records one
`Inferred` variable and one fact/deferred-free normalized-reflexive `Checked`
formula, and uses neither general reachability nor object/set coercion. Exact,
near-miss, corruption, route-order, and real frontend/resolver-sidecar guards
reject set terminals, builtin/other asserted heads, chains, attributes,
arguments, recovery, extra items, and collapsed provenance. Declaration
acceptance/inhabitation, truth/facts, theorem/proof/CoreIr/ControlFlowIr/VC,
other asserted-head families, and general semantics remain deferred.
Task 184 adds only the one-edge set-
terminal same-outer-mode asserted-head sibling. The exact source contains two
ordered definition blocks with `mode BaseModeAssertedHeadDef:
BaseModeAssertedHead is set;` and `mode ChainModeAssertedHeadDef:
ChainModeAssertedHead is BaseModeAssertedHead;`, one outer-mode reserve, and
only `ChainedLocalModeAssertedHeadPayloadBoundary: x is
ChainModeAssertedHead;`. The producer retains independent raw reserve-
subject and formula-side asserted type-expression inputs with distinct sites/
ranges and the same resolved outer-mode symbol. Both real AST-derived
expansions normalize all three known type entries to one builtin-set
identity canonically anchored at the base definition RHS. The prepared
consumer resolves ordinal 1 to `BindingId(0)`, records one `Inferred`
variable and one fact/deferred-free normalized-reflexive `Checked` formula, and
does not invoke general reachability, widening, or `qua`. Exact, near-miss,
corruption, route-order, and real frontend/resolver-sidecar guards reject
wrong links/terminals/order/depth, builtin/base/other asserted heads,
attributes, arguments, recovery, extra items, and collapsed provenance. This
task does not credit declaration acceptance/inhabitation, truth/facts,
closure/order, theorem/proof/CoreIr/ControlFlowIr/VC, object-terminal or deeper
asserted-head chains, or general chain semantics.
Task 120 extends that real identifier-term seam only for the exact source
`reserve x for set; theorem ReservedVariableMembershipPayloadBoundary: x in x;`.
The same source-range event ordering derives distinct lookup ordinals 1 and 2,
and two independent `BindingEnv::lookup` calls resolve both identifier terms to
the written reserve binding. The shared producer projects builtin `set` to two
result roles and, following the existing checker membership contract, one
expected-type role owned by the right operand. Production validation requires
two `Inferred` variable terms, one `Checked` `FormulaKind::Membership`, exactly
that single right expected-type constraint, the three exact role owners with
source range/spelling/head intact, and empty candidate/fact/deferred/diagnostic
output. Any matched-source construction or invariant failure reports
`type_elaboration.checker.reserved_variable_membership.invalid_payload`.
`Checked` remains type/well-formedness only: task 120 does not record membership
truth/facts, materialize implicit closure, accept the theorem, activate
`formula_statement`, or produce proof, CoreIr, ControlFlowIr, or VC payloads.
Non-exact labels, operators, operands, reserve shapes, item counts/order,
recovery/status tokens, and numeral operands remain on the extraction gap.
Task 121 adds the exact sibling
`reserve x for set; theorem ReservedVariableInequalityPayloadBoundary: x <> x;`.
The shared producer supplies two independently resolved known-`set` variables,
two result roles, and two expected roles linked to the checker outputs. One
fact-free `Checked` `FormulaKind::Inequality` records pre-desugaring
type/well-formedness only. Construction/invariant drift reports
`type_elaboration.checker.reserved_variable_inequality.invalid_payload`, and a
real frontend/resolver active-sidecar test guards the payload. The source of the
two-expected-type contract is checker-owned API coverage plus task 119's real
role producer; task 107 remains the partial numeral bridge without expected
types. Task 121 does not implement `not equality` desugaring, truth/facts,
implicit closure, theorem acceptance, proof, CoreIr, ControlFlowIr, or VC.
Task 109 supersedes the exact builtin `set` portion of task 102:
`theorem BuiltinTypeAssertionPayloadBoundary: 1 is set;` reaches parser and
resolver execution with a Chapter 13 numeral term and the Chapter 14 builtin
type-assertion form, then passes source-derived checker `TermInput`,
`FormulaInput`, and asserted builtin `set` `TypeExpressionInput` payloads before
failing on missing numeric type payloads and partial formula checking. Broader
asserted type payload extraction, term inference, type-assertion semantic
checking, formula checking beyond the partial-term diagnostic, recorded facts,
theorem acceptance, the dedicated `formula_statement` runner, CoreIr,
ControlFlowIr, VC, and proof payloads are still absent.
Task 122 combines that formula-side asserted-type producer with task 119's real
reserved-variable producer for the exact source
`reserve x for set; theorem ReservedVariableTypeAssertionPayloadBoundary: x is set;`.
It also repairs the checker-owned type-assertion admissibility gate: exactly one
ready subject and one normalized asserted type are required, semantic normalized
identity preserves `Checked`, and a known non-identical pair becomes `Partial`
with `FormulaDeferredReason::MissingTypeAssertionReachabilityPayload` and
`checker.formula.external.type_assertion_reachability_payload` instead of
inventing widening evidence. Missing asserted payload and invalid subject arity
use `checker.formula.missing_asserted_type` (`Partial`) and
`checker.formula.type_assertion.subject_arity` (`Error`) respectively. The
source runner independently retains the reserve-derived subject result input
and formula-node-derived asserted input before normalization, validates their
distinct source anchors and empty arguments/attributes, and then requires the
checker to link both to the same reflexive builtin-`set` semantic id. It records
one `Inferred` variable and one fact-free `Checked` type assertion. Task 109's
numeric subject remains partial with its exact existing two diagnostics.
General reachability/widening/`qua`, broader asserted types or attributes,
truth/facts, implicit closure, theorem acceptance, `formula_statement`, proof,
CoreIr, ControlFlowIr, and VC remain deferred.
Task 113 supersedes task 103 for the exact positive imported attribute
assertion variant of that same term/formula boundary:
`import parser.type_fixtures; theorem ImportedAttributeAssertionPayloadBoundary: 1 is empty;`
reaches parser and resolver execution with a Chapter 13 numeral term, the
documented imported `parser.type_fixtures` `empty` attribute, and the Chapter
14 attribute-assertion form, then validates imported `empty` provenance and
passes source-derived checker `TermInput` and `FormulaInput` payloads before
failing on missing numeric type payloads, missing formula/attribute semantic
payload, and partial formula checking. This does not credit imported module AST
extraction, imported attribute assertion attribute-chain semantic payload
extraction, checker `AttributeInput` payload extraction for theorem formulas,
term inference, attribute admissibility/semantic checking, formula checking,
recorded facts, theorem acceptance, the dedicated `formula_statement` runner,
CoreIr, ControlFlowIr, VC, or proof payloads.
Task 114 supersedes task 104 for the exact attribute-level `non empty`
imported attribute assertion variant of that same term/formula boundary:
`import parser.type_fixtures; theorem ImportedNonEmptyAttributeAssertionPayloadBoundary: 1 is non empty;`
reaches parser and resolver execution with a Chapter 13 numeral term, the
documented imported `parser.type_fixtures` `empty` attribute, Chapter 6
attribute negation/composition, and the Chapter 14 attribute-assertion form,
then validates the direct `non` surface and imported `empty` provenance and
passes source-derived checker `TermInput` and `FormulaInput` payloads before
failing closed on missing numeric type payload, missing formula/attribute
semantic payload, and partial formula checking. This does not credit imported
module AST extraction, negated attribute-chain semantic payload extraction,
checker `AttributeInput` payload extraction for theorem formulas, term
inference, negated attribute admissibility/semantic checking, formula checking,
recorded facts, theorem acceptance, the dedicated `formula_statement` runner,
CoreIr, ControlFlowIr, VC, or proof payloads.
Task 111 supersedes task 105 only for the exact set-enumeration theorem bridge:
`theorem SetEnumerationPayloadBoundary: {1, 2} = {1, 2};` reaches parser and
resolver execution with Chapter 13 set-enumeration term operands and Chapter 14
builtin equality, then passes real source-derived checker payloads for four
numeral item terms, two set-enumeration terms, and the equality formula before
failing closed on missing numeric type payloads, missing set-enumeration
result-type/sethood payloads, and partial formula checking. This does not claim
broader set-enumeration term extraction, term inference, equality/formula
checking, recorded facts, theorem acceptance, the dedicated `formula_statement`
runner, CoreIr, ControlFlowIr, VC, or proof payloads.
Task 112 supersedes task 99 only for the exact connective/quantifier theorem
formula bridge:
`theorem FormulaConnectiveQuantifierPayloadBoundary: contradiction implies for x being set holds not contradiction;`
reaches parser and resolver execution through the Chapter 14 implication,
universal-quantifier, and negation surfaces, then passes real source-derived
checker `FormulaInput` shells for the implication, quantified formula, and
negation. The checker must fail closed with
`FormulaDeferredReason::MissingFormulaPayload` for the implication and negation
shells and `FormulaDeferredReason::MissingQuantifierPayload` for the quantified
shell because child-formula graph payloads, quantifier binder/context payloads,
formula checking, recorded facts, theorem acceptance, the dedicated
`formula_statement` runner, CoreIr, ControlFlowIr, VC, and proof payloads are
still absent. Task 117 extends only this exact source by additionally passing
both source-derived `contradiction` constant sites/ranges as
`FormulaKind::Contradiction` checker payloads before the same missing formula
payload diagnostic. This does not credit formula constant semantic truth values,
broader formula extraction, or any accepted formula fact.
Task 88 records the matching proof-block boundary: a theorem such as
`theorem ProofSkeletonPayloadBoundary: thesis proof thus thesis; end;` reaches
parser and resolver execution with a Chapter 16 proof block and Chapter 15
conclusion statement, then stays on
`type_elaboration.external_dependency.ast_payload_extraction` because real proof
skeleton payload extraction, local proof context, formula payload extraction,
recorded facts, theorem acceptance, the dedicated `formula_statement` runner,
CoreIr, ControlFlowIr, VC, and proof payloads are still absent.
Task 89 records the statement-level proof-justification boundary: a theorem
proof containing a labeled `A: thesis proof ... end;` statement and a final
`thus thesis proof ... end;` conclusion reaches parser and resolver execution,
then stays on `type_elaboration.external_dependency.ast_payload_extraction`
because real statement proof payload extraction, nested proof skeleton payloads,
local proof context, formula payload extraction, label-reference semantic
checking, recorded facts, theorem acceptance, the dedicated
`formula_statement` runner, CoreIr, ControlFlowIr, VC, and proof payloads are
still absent.
Task 90 records the predicate/functor definition boundary: a definition block
with `pred DefinitionPredicatePayloadBoundary: x boundary_rel y means thesis;`
and `func DefinitionFunctorPayloadBoundary: boundary_func x -> set equals x;`
reaches parser and resolver execution, then stays on
`type_elaboration.external_dependency.ast_payload_extraction` because real
predicate/functor definition declaration payload extraction, definition-local
context, definiens formula/term payloads, overload payloads, recorded facts,
the dedicated `formula_statement` runner, CoreIr, ControlFlowIr, VC, and proof
payloads are still absent.
Task 91 records the attribute definition boundary: a definition block with
`attr AttributePayloadBoundary: x is marked means thesis;` reaches parser and
resolver execution, then stays on
`type_elaboration.external_dependency.ast_payload_extraction` because real
attribute definition declaration payload extraction, definition-local context,
formula-definiens payloads, attributed-type evidence, recorded facts, the
dedicated `formula_statement` runner, CoreIr, ControlFlowIr, VC, and proof
payloads are still absent.
Task 92 records the mode/structure definition boundary: a definition block
with `struct DefinitionStructPayloadBoundary where ... end;` and
`mode DefinitionModePayloadBoundaryDef: DefinitionModePayloadBoundary is set;`
reaches parser and resolver execution, then stays on
`type_elaboration.external_dependency.ast_payload_extraction` because real
mode/structure definition declaration payload extraction, mode expansion,
structure base-shape/constructor/selector evidence, definition-local context,
recorded facts, the dedicated `formula_statement` runner, CoreIr, ControlFlowIr,
VC, and proof payloads are still absent.
Task 93 records the proof-local declaration statement boundary: a theorem proof
with `let`, `given`, `consider`, `set`, and `reconsider` statements reaches
parser and resolver execution, then stays on
`type_elaboration.external_dependency.ast_payload_extraction` because real
proof-local declaration payload extraction, local proof context, formula and
term payloads, RHS term inference, reconsider coercion/obligation evidence,
recorded facts, the dedicated `formula_statement` runner, CoreIr, ControlFlowIr,
VC, and proof payloads are still absent.
Task 94 records the proof-local inline definition boundary: a theorem proof
with `deffunc` and `defpred` statements reaches parser and resolver execution,
then stays on `type_elaboration.external_dependency.ast_payload_extraction`
because real inline definition formal/body payload extraction, local
abbreviation expansion, term and formula body payloads, guard evidence,
recorded facts, theorem acceptance, the dedicated `formula_statement` runner, CoreIr,
ControlFlowIr, VC, and proof payloads are still absent.
Task 95 records the registration block boundary: a top-level registration
block with existential and conditional clusters reaches parser and resolver
execution, then stays on
`type_elaboration.external_dependency.ast_payload_extraction` because real
registration-item payload extraction, correctness-condition/proof-obligation
payloads, accepted activation/evidence status, cluster/reduction semantics,
recorded facts, the dedicated `formula_statement` or `advanced_semantics`
runners, CoreIr, ControlFlowIr, VC, and proof payloads are still absent. This
does not credit Chapter 17 semantic cluster/reduction rows.
Task 96 records the redefinition/notation boundary: top-level and
definition-local synonym/antonym aliases plus attribute, predicate, and functor
redefinition declarations reach parser and resolver execution using documented
`parser.type_fixtures` symbols, then stay on
`type_elaboration.external_dependency.ast_payload_extraction` because real
redefinition payload extraction, notation alias relation payloads, redefinition
target inference, coherence proof-obligation payloads, overload candidate
payloads, recorded facts, the dedicated `formula_statement` or
`advanced_semantics` runners, CoreIr, ControlFlowIr, VC, and proof payloads are
still absent. This does not credit Chapter 11 alias semantic resolution or
Chapter 19 overload/redefinition semantics.
Task 82 promotes only the imported-mode provenance portion of task 79: a
reserve head such as `TypeCaseMode` from the documented
`parser.type_fixtures` import summary may be passed as a checker-owned
`TypeHeadInput::Symbol` when the visible resolver symbol has
`SymbolKind::Mode` and an `ImportedSource` contribution. The runner must still
withhold `ModeExpansion` payloads because no real imported mode definition or
module-summary expansion payload is available, so the active case reaches
`checker.type.external.mode_expansion_payload` instead of the generic AST
payload extraction gap. This does not claim imported module AST extraction,
arity checking, positive imported mode elaboration, imported structures,
imported attributes, CoreIr, ControlFlowIr, VC, or proof payloads.
Task 57 additionally permits a bare same-module
no-argument local mode expansion whose RHS is a same-module local structure
head with no type arguments. The real `ModeExpansion` is consumed, so the case
must not report the missing mode-expansion payload diagnostic; however, the
expanded structure radix still fails closed with
`checker.declaration.deferred.evidence_query` until real
base-shape/constructor-witness evidence extraction exists. Task 62 additionally
permits the same structure-RHS diagnostic path through one bare local-mode
dependency edge: the unique unrecovered same-module no-argument terminal mode
definition `B is LocalStruct` and the unique unrecovered same-module
no-argument chain definition `A is B` must both precede the reserve use, the
unique unrecovered same-module structure definition must precede `B`, both mode
definitions must be free of definition-local context, and the runner must
extract both real `B -> LocalStruct` and `A -> B` expansions from the same
`SurfaceAst`. The expanded chain still fails closed with
`checker.declaration.deferred.evidence_query` until real
base-shape/constructor-witness evidence extraction exists; attributed roots,
attributed/deeper chains, imported or argument-bearing symbols, and contextual
or parameterized definitions remain outside this slice. Task 63 additionally
permits the same attributed-builtin RHS diagnostic path through one bare
local-mode dependency edge: the unique unrecovered same-module no-argument
terminal attributed-builtin mode definition (`B is marked set` or `B is marked object`) and the unique
unrecovered same-module no-argument chain definition `A is B` must both
precede the reserve use, `B` must precede `A`, both mode definitions must be
free of definition-local context, and the runner must extract both real
`B -> marked set` and `A -> B` expansions from the same `SurfaceAst`. The
expanded chain still fails closed with
`checker.declaration.deferred.evidence_query` until real attributed-type
existential evidence extraction exists; attributed roots, attributed/deeper
chains, imported or argument-bearing attributes or modes, and contextual or
parameterized definitions remain outside this slice.

Task 64 permits one attributed-root variant of the task-56 chain: an
attributed local-mode reserve head `marked A` may consume the real one-edge
chain `A -> B -> set` / `object` only when `A` is not also used as a bare
reserve head, `B` is not used as an attributed reserve head, both mode
definitions are unique, unrecovered, same-module, no-argument, and free of
definition-local context, and the source order is `B -> A -> reserve`. The
checker consumes both real expansions and the reserve-head attributes, then
fails closed with `checker.declaration.deferred.evidence_query` until real
attributed-type existential evidence extraction exists; attributed roots whose
one-edge dependency terminates in a local structure RHS remain outside task 64
but are admitted by task 65, and attributed-builtin RHS terminals remain
outside task 64 but are admitted by task 66, while deeper chains, mixed
bare/attributed uses, imports, arguments, contextual or parameterized
definitions, and positive attributed-type acceptance remain outside this
slice.

Task 65 permits the structure-RHS counterpart of the task-64 attributed-root
chain: an attributed local-mode reserve head `marked A` may consume the real
one-edge chain `A -> B -> LocalStruct` only when `A` is not also used as a bare
reserve head, `B` is not used as an attributed reserve head, `B is LocalStruct`
and `A is B` are unique, unrecovered, same-module, no-argument, and free of
definition-local context, the same-module structure definition is unique,
unrecovered, and precedes `B`, and the source order is
`LocalStruct -> B -> A -> reserve`. The checker consumes both real expansions
and the reserve-head attributes, then fails closed with
`checker.declaration.deferred.evidence_query` until real structure
base-shape/constructor-witness evidence and full attributed-type existential
evidence extraction exist; attributed-builtin RHS terminals, deeper chains,
mixed bare/attributed uses, attributed dependencies, imports, arguments,
ambiguous symbols, contextual or parameterized definitions, positive
structure/attributed-type acceptance, CoreIr, ControlFlowIr, VC, and proof
payloads remain outside this slice, except that task 66 separately admits
one-edge attributed-builtin RHS terminals.

Task 66 permits the attributed-builtin-RHS counterpart of the task-64/task-65
attributed-root chains: an attributed local-mode reserve head `marked A` may
consume the real one-edge chain `A -> B -> marked set` or
`A -> B -> marked object` only when `A` is not also used as a bare reserve
head, `B` is not used as an attributed reserve head, both mode definitions are
unique, unrecovered, same-module, no-argument, and free of definition-local
context, RHS attributes resolve to argument-free same-module attribute symbols,
and the source order is `B -> A -> reserve`. The checker consumes both real
expansions, the reserve-head attributes, and the terminal RHS attributes, then
fails closed with `checker.declaration.deferred.evidence_query` until real
full attributed-type existential evidence extraction exists; deeper chains,
mixed bare/attributed uses, attributed dependencies, imports, ambiguous
symbols, attribute or mode arguments, contextual or parameterized definitions,
positive attributed-type acceptance, CoreIr, ControlFlowIr, VC, and proof
payloads remain outside this slice.

Task 67 records the boundary for structure-qualified attribute references in
reserve type expressions. A source expression such as
`LocalStruct.marked LocalStruct` is valid type-expression syntax by Chapters 3
and 6 over a local structure declared by Chapter 5, but the current
checker-owned `AttributeInput` payload carries only the resolved attribute
symbol, polarity, arguments, range, and spelling; it has no structure-qualifier
or attribute-owner provenance. The active runner therefore
must leave structure-qualified attribute references on
`type_elaboration.external_dependency.ast_payload_extraction` instead of
rewriting them to an unqualified same-module attribute payload. This is
diagnostic boundary coverage only: it does not promote a real qualified
attribute payload, does not change the same-module no-argument unqualified
attribute slices, and does not fabricate existential evidence, CoreIr,
ControlFlowIr, VC, or proof payloads.

Task 68 records the boundary for argument-bearing mode heads in reserve type
expressions. Source such as `Element of a` is valid type-expression syntax by
Chapter 3 and can appear in a same-module mode surface, but the current
reserve source bridge admits only argument-free local mode or structure heads
and has no real term/type-argument provenance payload. The active runner
therefore must leave this source family on
`type_elaboration.external_dependency.ast_payload_extraction` before checker
mode expansion, arity matching, or positive type elaboration. This is
diagnostic boundary coverage only: it does not promote mode arguments into
`TypeExpressionInput`, does not fabricate term payloads, and does not promote
CoreIr, ControlFlowIr, VC, or proof payloads.

Task 69 records the matching boundary for argument-bearing structure heads in
reserve type expressions. Source such as `LocalStruct of a` is valid
type-expression syntax by Chapters 3 and 5 and can name a same-module
structure declaration with an `of` parameter surface, but the current reserve
source bridge still admits only argument-free local mode or structure heads and
has no real term/type-argument provenance payload. The active runner therefore
must leave this source family on
`type_elaboration.external_dependency.ast_payload_extraction` before
structure argument payload extraction, arity matching, base-shape or
constructor-witness evidence, or positive structure type elaboration. This is
diagnostic boundary coverage only: it does not promote structure arguments into
`TypeExpressionInput`, does not fabricate term payloads, and does not promote
CoreIr, ControlFlowIr, VC, or proof payloads.

Task 70 records the bracket-form counterpart for local mode heads in reserve
type expressions. Source such as `Family[set]` is valid bracket
type-argument syntax by Chapters 3 and 7 and can appear beside a same-module
bracket-parameter mode declaration, but the current reserve source bridge
still admits only argument-free local mode or structure heads and has no real
bracket `type_arg_list` or `qua`-argument provenance payload. The active runner
therefore must leave this source family on
`type_elaboration.external_dependency.ast_payload_extraction` before bracket
type-argument payload extraction, mode-head resolution, arity matching, mode
expansion, or positive type elaboration. This is diagnostic boundary coverage
only: it does not promote bracket arguments into `TypeExpressionInput`, does
not fabricate `qua` or term payloads, and does not promote CoreIr,
ControlFlowIr, VC, or proof payloads.

Task 71 records the bracket-form counterpart for local structure heads in
reserve type expressions. Source such as `LocalStruct[set]` is valid bracket
type-argument syntax by Chapters 3 and 5 and can appear beside a same-module
bracket-parameter structure declaration, but the current reserve source bridge
still admits only argument-free local mode or structure heads and has no real
bracket `type_arg_list`, `qua`-argument, or structure argument provenance
payload. The active runner therefore must leave this source family on
`type_elaboration.external_dependency.ast_payload_extraction` before bracket
type-argument payload extraction, structure-head resolution, arity matching,
base-shape or constructor-witness evidence, or positive structure type
elaboration. This is diagnostic boundary coverage only: it does not promote
bracket arguments into `TypeExpressionInput`, does not fabricate `qua`, term,
base-shape, or constructor evidence payloads, and does not promote CoreIr,
ControlFlowIr, VC, or proof payloads.

Task 60 additionally
permits that direct structure-RHS expansion for an attributed local-mode
reserve head only when the mode definition is unique, unrecovered, preceding,
no-argument, and free of definition-local context, the structure definition is
unique, unrecovered, same-module, and precedes the mode definition, and the
same mode is not also used as a bare reserve head in the same bridge input.
The expanded attributed structure type fails closed with
`checker.declaration.deferred.evidence_query` until real base-shape/
constructor-witness evidence and full attributed-type existential evidence
exist; mixed bare/attributed uses, dependencies/chains, imported or
argument-bearing symbols, and attributed structure RHSs remain outside this
slice. Task 58 additionally
permits a bare same-module no-argument local mode expansion whose RHS is an
attributed builtin `set` / `object` type. The real `ModeExpansion` is
consumed, so the case must not report the missing mode-expansion payload
diagnostic; however, the expanded attributed type still fails closed with
`checker.declaration.deferred.evidence_query` until real attributed-type
existential evidence extraction exists. Task 61 additionally permits that
direct attributed-builtin RHS expansion for an attributed local-mode reserve
head only when the mode definition is unique, unrecovered, preceding,
no-argument, and free of definition-local context, and the same mode is not
also used as a bare reserve head in the same bridge input. The expanded
attributed type fails closed with
`checker.declaration.deferred.evidence_query` until real attributed-type
existential evidence extraction exists; mixed bare/attributed uses,
dependencies/chains, imported or argument-bearing symbols, structure RHSs, and
attributed structure RHSs remain outside this slice. Task 52 additionally permits
un-attributed reserve type heads that resolve to a unique same-module
`LocalSource` `SymbolKind::Structure` entry with no type arguments. Those
local-structure reserve declarations reach declaration checking and fail
closed with `checker.declaration.deferred.evidence_query` until real
base-shape/constructor-witness evidence extraction exists. Task 53 permits
that local-structure slice to carry same-module source-derived attributes,
which still fail closed with `checker.declaration.deferred.evidence_query`
because Chapter 17 requires existential evidence for the full normalized
attributed type; bare-structure base-shape evidence would not be sufficient for
positive acceptance. The payload must include source/module identity, the
reserve item source range, each binding spelling and declaration range, and
each supported type-expression spelling/range/head plus any supported
same-module attribute symbol/range/polarity and supported same-module
local-mode expansion payloads. The seam
exposes deterministic typed-site ids for the runner to assemble the existing
`TypedAst` / `ResolvedTypedAst` readiness checks for the successful
bare-builtin and supported local-mode expansion slices, but it does not
authorize `mizar-checker` to import
`mizar-syntax`, scan raw syntax, accept non-reserve declarations, invent
imported symbols, fabricate mode expansions or existential/base-shape
evidence, attach imported or argument-bearing source attributes to local mode
heads, accept argument-bearing symbol heads, or claim CoreIr / ControlFlowIr /
VC / proof execution.

Required behavior:

- `let`, definition parameters, quantified variables, `given`, `consider`, and
  `take` binders receive `TypeEntry`s linked to their `BindingId` and
  `TypedSiteRef`;
- reserved variables use explicit default type-site payloads only when that
  payload says the occurrence is not shadowed by a local binding;
- `set` declarations attach an explicit normalized type when supplied, or record
  a deferred diagnostic for right-hand-side inference when that payload is not
  available yet;
- `deffunc` and `defpred` formals are checked as local definition parameters
  when explicit formal payloads are supplied; body checking is deferred to term
  and formula inference;
- `reconsider x as T` updates the type view of an existing binding at the
  current site, while `reconsider y = t as T` introduces a new local binding;
- declarations with attributed or constrained types are marked as requiring
  later sethood/existence handling; task 10 emits the corresponding
  `InitialObligation`s once coercion and evidence checks exist;
- `such that`, `given`, and assumption-like clauses on checked declarations add
  `Assumed` facts only to the context that introduced them. If the introducing
  declaration is partial or erroneous, task 8 drops those assumption payloads
  with an explicit diagnostic rather than publishing active evidence. Task 11
  owns the full fact query API.

Invalid declarations must produce explicit diagnostics and partial entries. They
must not fabricate known facts, silently activate registrations, or drop the
source-shaped typed site.

## Task 9: Term And Formula Type Inference

Task 9 implements this section.

Term inference records a `TypeEntry` for each typed term site. Formula inference
records well-formedness and task-9-local facts introduced by formula structure.
Task 11 keeps responsibility for the complete deterministic fact query API and
for expanding fact recording beyond the minimal inference records needed by
task 9.

Task 9 uses a checker-owned term/formula payload. The current resolver does not
yet expose an AST-wide typed term/formula table, built-in numeric type payload,
functor/predicate candidate signatures, selector and structure-field payloads,
source `qua` coercion evidence, or sethood/non-emptiness evidence queries. Task
9 must therefore not walk raw syntax to reconstruct those inputs. It consumes
explicit payloads supplied by the caller: term/formula sites, binding ids or
resolver symbols for references, explicit result/expected type expressions,
current-result type payloads for `it`, and unresolved candidate groups when a
later overload phase must finish root selection. Missing resolver/source
payloads are classified as `external_dependency_gap` or deferred diagnostics.

The implementation surface is `TermFormulaChecker`. It produces a
`TermFormulaInferenceOutput` containing normalized type expressions, checked
term and formula records, a task-local `OpenCandidateSetTable`, `TypeTable`,
`TypeFactTable`, and diagnostics. `TypedAst` already reserves
`TypeEntryActual::CandidateSet(OpenCandidateSetId)`, but task 9 owns the first
candidate-set payload table in `type_checker`; later overload and
`ResolvedTypedAst` tasks consume or project that table rather than treating the
candidate id alone as a final overload decision.

Formula well-formedness is recorded in checked-formula status and linked facts,
not as a fabricated successful term type. Source-written `qua`, sethood checks,
and non-emptiness checks may record deferred diagnostics and open type views in
task 9, but `CoercionTable` entries and `InitialObligation`s are emitted only by
task 10.

Task 9 classification:

- `external_dependency_gap`: missing resolver/source payloads for AST-wide
  term/formula extraction, numeric built-ins, functor/predicate signatures,
  selector/structure signatures, or source `qua` evidence. These are recorded
  as degraded diagnostics and partial/skipped checked terms or formulas.
- `deferred`: sethood, non-emptiness, and source-`qua` coercion/obligation
  emission that is intentionally owned by task 10. These are recorded as
  deferred requirement markers or diagnostics, not as `CoercionTable` or
  `InitialObligationTable` entries.

Term rules:

- variable references consume `BindingEnv` lookup results and attach the
  selected binding or resolver symbol to the typed site;
- `it` is valid only in definition/property contexts that provide a current
  result type;
- numerals receive the built-in numeric type payload exposed by resolver or a
  degraded external-gap type when that payload is absent;
- functor applications may keep candidate groups when final overload root
  selection is not phase-6-deterministic;
- selector access records supplied result or candidate payloads and degrades
  missing selector/signature payloads as MC-G017; field/property visibility
  validation waits for resolver-exposed selector signatures;
- structure constructors record supplied result payloads and degrade missing
  structure-field payloads as MC-G017; field coverage and value-type checking
  wait for resolver-exposed structure signatures;
- set enumeration and set comprehension produce set-like types and record
  deferred sethood requirements for generator domains when spec chapter 13
  requires them;
- `the T` records a choice-like typed term and a deferred non-emptiness
  requirement for `T` without assigning a proof-owned id;
- source-written `qua` records the source view needed by later checking and a
  deferred source-`qua` requirement; task 10 creates the `SourceQua` coercion
  candidate.

Formula rules:

- predicate applications check candidate argument types but keep unresolved
  candidate groups for phase 8 when final root selection is ambiguous;
- built-in `=`, `<>`, and `in` forms check term well-formedness and add
  appropriate expected-type constraints;
- type assertions require exactly one ready subject and a normalized asserted
  type; normalized identity is reflexively admissible, while non-identical
  known types remain partial on the explicit external reachability payload gap
  until widening/`qua` evidence exists; attribute-assertion admissibility remains
  deferred until real radix/parameter and attribute-chain semantic payloads
  exist;
- logical connectives preserve formula type/well-formedness state and combine
  facts only through explicit assumption/conclusion rules owned by statements;
- quantified formulas create binder contexts through `BindingEnv` and check the
  body under those contexts.

If enough local payload is present to know that a site has no matching typed
candidate, the checker records `Unknown` or `Error` status instead of inventing a
successful type.

## Task 10: Coercion Candidates And Initial Obligations

Task 10 implements this section.

Coercion entries are checker-discovered candidates, not final inserted views.

Task 10 uses checker-owned coercion and initial-obligation payloads. The current
resolver/checker boundary still does not expose an AST-wide coercion request
table, active dependency-summary fact database, structure inheritance graph,
cluster-closure evidence, sethood evidence, non-emptiness evidence, or
proof-query results. Task 10 consumes explicit source sites, normalized source
and target type payloads, supporting fact ids, and obligation request payloads.
Missing resolver/dependency payloads are `external_dependency_gap`s and produce
blocked or rejected entries with diagnostics; they must not be repaired by raw
syntax walking, registration closure, proof search, or invented facts.

The implementation surface is `CoercionObligationChecker`. It produces a
`CoercionCheckingOutput` containing normalized type expressions, `TypeEntry`s,
`CoercionTable`, `InitialObligationTable`, optional obligation-backed facts,
and diagnostics. Supporting facts are accepted only when explicitly supplied
and present with a consumable status in the input fact table. The output
preserves the input facts and appends local obligation facts so coercion support
ids remain valid for handoff. Built-in radix widening may append a
checker-local builtin fact; structure-inheritance and activated-summary
evidence require supplied consumable supporting fact ids and are not accepted
from an enum marker alone. Task 11 still owns the full fact query API.

Required behavior:

- widening candidates are proof-free only when supported by known type facts,
  built-in radix widening recorded as a local builtin fact, structure
  inheritance payload represented by supplied facts, or already activated
  dependency summaries represented by supplied facts through a task-scoped seam;
- source-written `qua` is valid only for statically checkable upcasts or
  compatible views; it must not be used as narrowing proof;
- explicit narrowing to a more specific type creates an `InitialObligation`
  unless the task-10 input supplies `KnownFacts` evidence with consumable
  supporting fact ids that already support the target type;
- task 47 distinguishes explicit and omitted `reconsider` payloads.
  `CoercionJustification::Explicit` preserves the task-10 obligation path.
  `CoercionJustification::Omitted` is accepted only when a consumable fact
  backs proof-free evidence from known local facts, built-in radix widening,
  structure inheritance, activated summaries/cluster closure, static upcast, or
  compatible view. Evidence markers alone do not discharge the omitted form;
- omitted `reconsider` with missing or non-consumable proof-free evidence emits
  `type.narrowing_requires_proof`, records a rejected/degraded coercion, and
  creates no implicit obligation, proof search, or hidden `by`;
- sethood and non-emptiness requirements create `InitialObligation`s with source
  assumptions and deterministic local ids;
- failed or unsupported coercions remain as `Blocked` or `Rejected` entries with
  diagnostics.

`InitialObligationId` is the phase-6 boundary. Task 10 must not assign `VcId`,
`ObligationAnchor`, prover status, proof witness, or accepted verifier status.
Context-sensitive `Assumed` facts are not queried directly by the omitted
`reconsider` helper; an upstream producer must first use the fact-query/context
boundary to supply a consumable supporting fact id. Source-derived
reconsider/coercion extraction remains deferred under MC-G019/MC-G020.

## Task 11: Type Facts And Queries

Task 11 implements this section.

Type facts are the local currency shared by declaration checking, inference,
coercion checking, and later registration/overload phases.

Task 11 uses the facts already recorded by declaration checking, term/formula
inference, and coercion checking. It does not add a source walker or infer new
registration facts. The current resolver/checker boundary still does not expose
an AST-wide statement/proof assumption table, theorem acceptance payload, or
phase-7 `ResolutionTrace` facts; those remain `external_dependency_gap`s for
later tasks.

The implementation surface is `TypeFactQueryEngine`. It consumes a
`TypeFactTable` plus an optional `LocalTypeContextTable` and answers
deterministic point queries through `TypeFactQuery` /
`TypeFactQueryOutput`. Query output carries explicit `Satisfied`, `Missing`, or
`Contradicted` status, matched fact ids in canonical order, and query-local
diagnostics for contradictions. It may list active visible facts for a context,
but it must not derive new facts or rewrite table entries.

`TypeFactQuery` matches by subject, predicate, polarity, and an optional local
context id. Provenance does not participate in point-query matching; it is kept
for canonical output ordering, traceability, and later explanation. A positive
query is `Satisfied` when at least one active positive fact for the
subject/predicate is visible and no active negative fact for the same
subject/predicate is visible. A negative query is symmetric. A query is
`Missing` when no active matching-polarity fact is visible and no active
opposite-polarity fact is visible.

Contradiction means active visible facts for the same subject and predicate with
opposite polarity, ignoring provenance after visibility/status filtering. A
contradicted query returns all active same subject/predicate fact ids for both
polarities in canonical order and emits a query-local diagnostic; it does not
mutate the underlying fact table.

Assumed facts are visible only when the query supplies a context id and the
engine has a `LocalTypeContextTable` where that context can consume the fact.
If the engine has no context table, the query omits a context, the context id is
missing, or the fact is outside that context's visibility chain, `Assumed` facts
are inactive for that query. This prevents assumption leakage into
registration, overload, or dependency-summary consumers that have not selected a
local context.

Fact sources:

- `Declared`: binding declarations and type-expression sites;
- `Assumed`: local assumptions, `such that`, `given`, and proof-context
  assumptions;
- `Inferred`: direct checker rules such as built-in widening or selector result
  typing;
- `Obligation`: facts whose claim is represented by an `InitialObligationId`;
- `Builtin`: built-in facts about `object`, `set`, equality, and membership;
- `Registration`: reserved for later phase-7 closure with a `ResolutionTrace`
  step.

Query rules:

- only `Known` facts are unconditionally consumable;
- `Assumed` facts are consumable only in the introducing context or visible
  descendants recorded in `LocalTypeContextTable`;
- `PendingObligation`, `Degraded`, and `Rejected` facts are not active evidence;
- `TypeFactTable` semantic keys include subject, predicate, polarity, and
  provenance. Assumption visibility is controlled by `LocalTypeContextTable`
  `introduced_assumptions` / `visible_facts`, not by insertion order;
- contradictory active facts produce query diagnostics and explicit
  `Contradicted` status instead of being resolved by insertion order.

Phase 6 may record facts needed by later registration resolution, but it must not
create `Registration` provenance or trace steps before phase 7 owns the
corresponding derivation.

## Partial Typing And Recovery

Recoverable errors must leave explicit partial state:

- unresolved or ambiguous type heads create degraded `NormalizedType`s and
  `TypeEntry` diagnostics;
- unresolved terms, missing binding payloads, missing signature payloads, and
  impossible candidate groups produce `Unknown`, `Error`, or `Skipped` entries;
- facts and coercions derived from degraded sites carry `Degraded`, `Blocked`,
  or `Rejected` status;
- diagnostics retain primary source ranges and stable secondary keys;
- later phases must check status predicates before consuming type, fact,
  coercion, or obligation entries.

Recovery is an under-approximation policy. The checker may omit facts and emit
diagnostics, but it must not invent verified facts, activate registrations, or
mark an obligation as accepted.

## Diagnostics

Task-local diagnostics use stable message keys until the public checker
diagnostic code-space is allocated.

Required diagnostic classes include:

- unknown or ambiguous type head;
- unsupported or missing resolver signature payload;
- wrong type-argument arity or kind;
- contradictory attributes;
- uninhabited or unsupported attributed declaration;
- illegal declaration or local-binding type;
- invalid `qua` target or narrowing without an obligation;
- failed sethood or non-emptiness requirement;
- term/formula kind mismatch;
- ambiguous or impossible candidate group;
- partial-typing recovery boundary.

Diagnostics are not proof evidence. They may explain degraded table entries, but
they must not be used as supporting facts.

## Determinism

Task 7-11 implementations must preserve deterministic output:

- normalized type ids are allocated by canonical type key;
- declaration, term, formula, coercion, obligation, fact, and diagnostic
  iteration order is independent of hash-map iteration;
- candidate groups are sorted by resolver candidate identity, mandatory type
  constraints, source range, then stable local id;
- debug rendering extends `typed-ast-debug-v1` without host paths, memory
  addresses, or nondeterministic map order;
- equivalent `ResolvedAst`, `SymbolEnv`, `BindingEnv`, dependency summaries, and
  checker configuration produce equivalent `TypedAst` tables.

## Public Enum Policy

Task 31 applies the frontend task-25 public-enum decision procedure to this
module. All public checker-owned enums in `type_checker` are forward-compatible
API surfaces and must remain `#[non_exhaustive]`; downstream consumers must
keep wildcard or fallback arms. Checker-internal matches may remain exhaustive
over the currently represented variants when implementing the specified
behavior.

| enum | decision |
|---|---|
| `CoercionRequestKind` | Forward-compatible; coercion request categories may grow with later view and obligation forms. |
| `CoercionJustification` | Forward-compatible; justification classes may grow as proof-block and artifact-backed evidence payloads land. |
| `CoercionEvidence` | Forward-compatible; coercion evidence may grow with proof, registration, and artifact sources. |
| `CoercionDeferredReason` | Forward-compatible; deferred coercion reasons may grow as external payload gaps close. |
| `InitialRequirementKind` | Forward-compatible; initial requirement categories may grow with VC/proof integration. |
| `TypeFactQueryStatus` | Forward-compatible; fact query outcomes may grow as contradiction and evidence policy matures. |
| `TermKind` | Forward-compatible; term categories may grow with source-to-checker extraction. |
| `TermReference` | Forward-compatible; term references may gain additional checker-owned identity anchors. |
| `TermDeferredReason` | Forward-compatible; deferred term reasons may grow as source payloads land. |
| `FormulaKind` | Forward-compatible; formula categories may grow with statement/proof extraction. |
| `FormulaDeferredReason` | Forward-compatible; deferred formula reasons may grow as source payloads land. |
| `CandidateIdentity` | Forward-compatible; open candidate identities may grow with richer overload extraction. |
| `CandidateSetKind` | Forward-compatible; candidate-set categories may grow with later overload phases. |
| `CandidateSetStatus` | Forward-compatible; candidate-set states may grow with deferred and failed-site handling. |
| `CandidateStatus` | Forward-compatible; candidate states may grow with evidence and recovery handling. |
| `TermStatus` | Forward-compatible; checked-term states may grow with partial inference policy. |
| `FormulaStatus` | Forward-compatible; checked-formula states may grow with partial inference policy. |
| `DeclarationKind` | Forward-compatible; declaration kinds may grow with more Mizar binding forms. |
| `DeclarationDeferredReason` | Forward-compatible; deferred declaration reasons may grow as extraction gaps close. |
| `DeclarationStatus` | Forward-compatible; declaration states may grow with local recovery and handoff policy. |
| `TypeHeadInput` | Forward-compatible; input type heads may grow with resolver and built-in payloads. |
| `AttributePolarity` | Forward-compatible; attribute polarity may grow if type predicates gain richer qualifiers. |
| `TypeHeadRef` | Forward-compatible; normalized type heads may grow with structures, modes, and built-ins. |
| `TypeHeadErrorKind` | Forward-compatible; type-head error categories may grow with resolver diagnostics. |
| `NormalizedTypeStatus` | Forward-compatible; normalized type states may grow with recovery and artifact handoff policy. |

No exhaustive public enum exceptions are owned by this module.

## Planned Tests For Tasks 7-11

Task 7 must add Rust tests for:

- attribute sorting, deduplication, polarity, and contradiction diagnostics;
- built-in singleton heads, structure heads that remain radix heads, and
  recursive type-argument normalization;
- mode unfolding idempotence when the explicit mode-expansion provider supplies
  payload;
- degraded mode/type entries when signature payload is missing;
- unknown or ambiguous heads, wrong arity/kind diagnostics, and source-range
  preservation;
- deterministic normalized type ids;
- the guard that type normalization does not use cluster closure to repair
  degraded types.

Task 8 must add Rust tests for:

- binding type attachment for `let`, quantified binders, definition parameters,
  reserved variables, `set`, `deffunc`, `defpred`, and `reconsider`;
- explicit reserved-variable default payload handling and deferred diagnostics
  when reserve/source payload is missing;
- `set`, `deffunc`, and `defpred` deferred diagnostics when RHS or body payloads
  are not yet available;
- both `reconsider` forms and deferred obligation diagnostics for constrained
  declarations;
- local assumption visibility and context snapshot updates;
- partial entries after invalid declarations;
- deterministic local-context, type-entry, diagnostic, and debug-rendering
  order.

Task 9 must add Rust tests for:

- variable, numeral, selector, structure, set-expression, choice, `qua`, and
  parenthesized term sites;
- `it` validity and built-in `=`, `<>`, and `in` expected-type constraints;
- predicate applications, type assertions, attribute assertions, connectives,
  and quantified formulas;
- candidate groups that remain open for overload resolution;
- sorted candidate groups and deterministic term/formula/diagnostic rendering;
- unknown/error/skipped partial typing, including the rule that recovery does
  not fabricate successful types.

Task 10 must add Rust tests for:

- widening, source `qua`, and narrowing coercion candidates;
- invalid `qua` targets that do not become narrowing proof;
- failed or unsupported coercions that remain `Blocked` or `Rejected`;
- sethood and non-emptiness initial obligations;
- `reconsider` obligation source ranges and assumption lists;
- deterministic coercion, obligation, diagnostic, and debug-rendering order;
- boundary guards that no `VcId` or proof-owned status is assigned.

Task 11 must add Rust tests for:

- fact deduplication and canonical query order;
- consumability rules for `Known`, `Assumed`, `PendingObligation`, `Degraded`,
  and `Rejected`;
- contradiction diagnostics;
- absence of `Registration` provenance before phase 7 trace ownership.

No `.miz` checker-stage fixtures are required by task 6 because it is
documentation-only. Task 12 still owns the first active `type_elaboration`
corpus runner and traceability metadata.

## Task 6 Classification

| Class | Finding | Action |
|---|---|---|
| `spec_gap` | No task-6 blocking spec gap remains for the named phase-6 sections. Chapters 03, 08, and 13 plus architecture 04 provide enough authority for normalization, declaration checking, term-expression inference, coercion candidates, facts, and recovery. | Continue to task 7 after task-6 review, verification, and commit. |
| `test_gap` | Active checker-stage `.miz` coverage and `type_elaboration` runner remain absent. | Tasks 7-11 add task-local Rust tests. Task 12 owns active corpus coverage and traceability metadata. |
| `design_drift` | Architecture 04 examples use broad `TypeContext` and `CoercionCandidateTable` names, while existing checker docs use `BindingEnv`, immutable `LocalTypeContextTable`, and `CoercionTable`. | This spec preserves the refined checker module split and treats `CoercionTable` entries as candidates until later phases resolve them. |
| `source_drift` | At task 6 time `src/type_checker.rs` did not exist yet. | Resolved by task 7, which creates the module and exports it from `lib.rs`; no source repair belonged to task 6. |
| `external_dependency_gap` | Several implementation seams depend on resolver-exposed signature payloads for mode unfolding, structure fields, attributes, functor/predicate candidates, built-ins, and dependency activated summaries. Public checker diagnostic codes are also not allocated. | Implementation tasks must consume only exposed resolver/artifact payloads. Missing payloads become external dependency gaps or degraded diagnostics; do not add direct raw-syntax reconstruction. |
| `deferred` | Registration closure, reduction normalization, final overload selection, inserted overload-disambiguating `qua` views, VC generation, proof acceptance, kernel replay, and artifact publication remain outside task 6 and phase 6. | Later checker and downstream crate tasks own these boundaries. |
