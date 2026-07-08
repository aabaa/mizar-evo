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
`AttributeInput`; broader imported attributes outside those bridges remain
deferred until source-derived fixtures and payload producers exist. Active
runner sidecars now pin positive `empty set` and imported `empty` on builtin
`object` to the external extraction-gap boundary. The bridge
must not treat that imported summary as real imported module AST extraction,
must not fabricate attributed-type evidence, positive attributed type
elaboration, positive `empty set`, imported `empty` on non-`set` heads, or
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
Positive `empty set` and imported attributes on `object` remain active
external-gap boundary fixtures. It does not credit positive `empty set`,
imported attributes on `object` or
local symbol heads, imported module AST extraction, attributed-type evidence,
positive imported attributed type elaboration, structure-qualified owner
provenance, attribute arguments, or downstream payloads.
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
can reach the active type-elaboration runner, but it remains on
`type_elaboration.external_dependency.ast_payload_extraction` until checker-owned
theorem/formula payload extraction, local proof contexts, recorded facts, and
the dedicated `formula_statement` runner exist. This boundary does not credit
theorem acceptance, formula facts, proof skeletons, CoreIr, ControlFlowIr, VC,
or proof payloads.
Task 87 records the same boundary for a term-bearing theorem formula:
`theorem TermFormulaPayloadBoundary: 1 = 1;` reaches parser and resolver
execution with Chapter 13 numeral terms and the Chapter 14 builtin equality
surface, then stays on `type_elaboration.external_dependency.ast_payload_extraction`
because real term/formula payload extraction, term inference, formula checking,
recorded facts, theorem acceptance, the dedicated `formula_statement` runner,
CoreIr, ControlFlowIr, VC, and proof payloads are still absent.
Task 98 records the imported predicate/functor variant of that same boundary:
`theorem ImportedPredicateFunctorPayloadBoundary: 1 divides (1 ++ 2);` reaches
parser and resolver execution through the documented `parser.type_fixtures`
surface, then stays on `type_elaboration.external_dependency.ast_payload_extraction`
because imported predicate/functor semantic payloads, term/formula payload
extraction, term inference, formula checking, recorded facts, theorem
acceptance, the dedicated `formula_statement` runner, CoreIr, ControlFlowIr,
VC, and proof payloads are still absent. This does not credit imported module
AST extraction.
Task 100 records the builtin membership variant of that same term/formula
boundary: `theorem BuiltinMembershipPayloadBoundary: 1 in 1;` reaches parser
and resolver execution with Chapter 13 numeral terms and the Chapter 14
builtin membership predicate, then stays on
`type_elaboration.external_dependency.ast_payload_extraction` because real
term/formula payload extraction, membership operand type inference/checking,
formula checking, recorded facts, theorem acceptance, the dedicated
`formula_statement` runner, CoreIr, ControlFlowIr, VC, and proof payloads are
still absent.
Task 101 records the builtin inequality variant of that same term/formula
boundary: `theorem BuiltinInequalityPayloadBoundary: 1 <> 2;` reaches parser
and resolver execution with Chapter 13 numeral terms and the Chapter 14
builtin inequality predicate, then stays on
`type_elaboration.external_dependency.ast_payload_extraction` because real
term/formula payload extraction, inequality desugaring or equality semantic
checking, formula checking, recorded facts, theorem acceptance, the dedicated
`formula_statement` runner, CoreIr, ControlFlowIr, VC, and proof payloads are
still absent.
Task 102 records the builtin type-assertion variant of that same term/formula
boundary: `theorem BuiltinTypeAssertionPayloadBoundary: 1 is set;` reaches
parser and resolver execution with a Chapter 13 numeral term and the Chapter
14 builtin type-assertion form, then stays on
`type_elaboration.external_dependency.ast_payload_extraction` because real
term/formula payload extraction, type-assertion type payload extraction, term
inference, type-assertion semantic checking, formula checking, recorded facts,
theorem acceptance, the dedicated `formula_statement` runner, CoreIr,
ControlFlowIr, VC, and proof payloads are still absent.
Task 103 records the imported attribute assertion variant of that same
term/formula boundary:
`import parser.type_fixtures; theorem ImportedAttributeAssertionPayloadBoundary: 1 is empty;`
reaches parser and resolver execution with a Chapter 13 numeral term, the
documented imported `parser.type_fixtures` `empty` attribute, and the Chapter
14 attribute-assertion form, then stays on
`type_elaboration.external_dependency.ast_payload_extraction` because real
term/formula payload extraction, imported attribute assertion
attribute-chain/provenance payload extraction, term inference, attribute
admissibility/semantic checking, formula checking, recorded facts, theorem
acceptance, the dedicated `formula_statement` runner, CoreIr, ControlFlowIr,
VC, and proof payloads are still absent. This does not credit imported module
AST extraction or checker `AttributeInput` payload extraction for theorem
formulas.
Task 104 records the attribute-level `non empty` imported attribute assertion
variant of that same term/formula boundary:
`import parser.type_fixtures; theorem ImportedNonEmptyAttributeAssertionPayloadBoundary: 1 is non empty;`
reaches parser and resolver execution with a Chapter 13 numeral term, the
documented imported `parser.type_fixtures` `empty` attribute, Chapter 6
attribute negation/composition, and the Chapter 14 attribute-assertion form,
then stays on `type_elaboration.external_dependency.ast_payload_extraction`
because real term/formula payload extraction, imported attribute-level
non-empty assertion attribute-chain/provenance payload extraction, term
inference, negated attribute admissibility/semantic checking, formula checking,
recorded facts, theorem acceptance, the dedicated `formula_statement` runner,
CoreIr, ControlFlowIr, VC, and proof payloads are still absent. This does not
credit imported module AST extraction or checker `AttributeInput` payload
extraction for theorem formulas.
Task 105 records the set-enumeration variant of that same term/formula
boundary: `theorem SetEnumerationPayloadBoundary: {1, 2} = {1, 2};` reaches
parser and resolver execution with Chapter 13 set-enumeration term operands
and Chapter 14 builtin equality, then stays on
`type_elaboration.external_dependency.ast_payload_extraction` because real
set-enumeration term payload extraction, term/formula payload extraction, term
inference, equality/formula checking, recorded facts, theorem acceptance, the
dedicated `formula_statement` runner, CoreIr, ControlFlowIr, VC, and proof
payloads are still absent.
Task 99 records the connective/quantifier formula variant of that theorem
boundary:
`theorem FormulaConnectiveQuantifierPayloadBoundary: contradiction implies for x being set holds not contradiction;`
reaches parser and resolver execution through the Chapter 14 implication,
universal-quantifier, and negation surfaces, then stays on
`type_elaboration.external_dependency.ast_payload_extraction` because real
formula payload extraction, quantifier binder/context payloads, formula
checking, recorded facts, theorem acceptance, the dedicated
`formula_statement` runner, CoreIr, ControlFlowIr, VC, and proof payloads are
still absent.
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
- type and attribute assertions check admissibility of the subject term against
  the normalized asserted type or attribute chain;
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
