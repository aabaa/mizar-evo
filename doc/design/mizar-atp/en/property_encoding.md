# mizar-atp Property Encoding

> Canonical language: English. Japanese companion:
> [../ja/property_encoding.md](../ja/property_encoding.md).

## Purpose

This document specifies how `mizar-atp` represents resolved Mizar-side
properties as backend-neutral `EncodedProperty` rows. It refines the
"Property Encoding" section of
[architecture 09](../../architecture/en/09.atp_interface_protocol.md).

Property encoding is candidate-production input only. It does not prove a VC,
does not run a backend, and does not create trusted acceptance material. Kernel
acceptance remains based on formula/substitution/provenance/target evidence
checked by `mizar-kernel`.

## Scope

Task 7 is specification-only. It authorizes the future task-8 source module to
encode explicit, already available property facts into `AtpProblem.properties`.
It does not add Rust source, concrete TPTP/SMT-LIB text, backend process
execution, portfolio policy, or artifact witness publication.

The property encoder may consume only structured property inputs produced by
earlier Mizar-owned phases. It must not infer properties from symbol names,
backend declarations, library naming conventions, proof hints, traces, logs, or
backend responses.

## Inputs And Identity

Each encoded property must have:

- a stable source property identity supplied by the VC/core side;
- a property family from the supported-family table below;
- the backend-visible target symbol already declared in `AtpProblem`;
- the target symbol kind and arity required by the family;
- source provenance that can be represented as `AtpSourceRef::EncodedProperty`;
- the selected `LogicProfile` and its property/native-extension capability.

The deterministic property identity is based on the source property identity,
property family, target symbol source identity, target arity, and selected
encoding strategy. It must not use traversal order, map iteration, display
spelling alone, source ranges alone, backend output order, process ids, random
state, or wall-clock time.

Duplicate source property identities or duplicate encoded identities are
fail-closed producer input errors. Distinct property families may apply to the
same target symbol only when each has its own explicit source property fact.

## Supported Families

The task-8 implementation may encode only these property families until a later
spec expands the table:

| Family | Target | Formula Shape | Default Encoding |
|---|---|---|---|
| `commutativity` | binary function | `forall a b. F(a, b) = F(b, a)` | axiom formula |
| `symmetry` | binary predicate | `forall a b. P(a, b) -> P(b, a)` | axiom formula |
| `reflexivity` | binary predicate | `forall a. P(a, a)` | axiom formula |
| `idempotence` | binary function | `forall a. F(a, a) = a` | axiom formula |
| `involutiveness` | unary function | `forall a. F(F(a)) = a` | axiom formula |
| `projectivity` | unary function | `forall a. F(F(a)) = F(a)` | axiom formula |
| `asymmetry` | binary predicate | `forall a b. P(a, b) -> not P(b, a)` | axiom formula |
| `connectedness` | binary predicate | `forall a b. a != b -> P(a, b) or P(b, a)` | axiom formula |
| `irreflexivity` | binary predicate | `forall a. not P(a, a)` | axiom formula |

Any family not listed here is `deferred`. Wrong target kind, wrong arity,
missing declaration, missing symbol-map row, missing provenance, malformed
source identity, or unsupported formula shape fails closed.

## Encoding Strategies

### Axiom Formula

The default strategy is `EncodedProperty::axiom`. The generated formula must be
a structured `AtpFormulaTree`, not backend text. It must be universal over the
required formal variables and must use the same backend-visible target symbol
that appears in the `target_symbol` field.

The selected `LogicProfile` must support every construct used by the formula:
quantifiers for the universally quantified families, equality for function
properties and connectedness disequality, implication, disjunction, and
negation where they appear, and first-order terms for nested unary-function
properties. The current backend-neutral `LogicProfile` treats Boolean
connectives as baseline first-order formula-tree constructs; therefore
`connectedness` is permitted to use `AtpFormulaTree::Or` when quantifiers and
equality are supported. If a future profile adds explicit connective gates,
`connectedness` must require disjunction support and fail closed when it is
absent. If the profile does not support a required construct, the property is
unsupported for that profile; the encoder must not approximate the property,
drop it silently, or replace it with `true`.

### Generated Binders

Every formal variable introduced by a property axiom must be represented as a
generated binder symbol before the formula is constructed. For each binder, the
encoder must create:

- an `AtpDeclaration` with kind `AtpDeclarationKind::GeneratedBinder` and arity
  `0`;
- an `AtpSymbolMapEntry` with `AtpSymbolSource::GeneratedBinder`;
- provenance that explains the generated binder identity.

Binder identities, backend-neutral symbol names, declarations, symbol-map
rows, and provenance payloads are derived from canonical property identity,
target symbol source identity, binder position, and optional sort identity.
They must not depend on display spelling alone, source range alone, traversal
order, map iteration, random state, or backend output. The binder order inside
the quantified formula is canonical by family definition: unary families use
the single position `0`; binary families use positions `0` then `1`.

Missing or duplicate generated-binder declarations, symbol-map rows, or
provenance rows fail closed before an `AtpProblem` is constructed. A property
axiom must not reuse caller-supplied instantiated variables as trusted payload;
the property encoder owns the generated binder identities for the
backend-neutral candidate problem.

### Native Declaration

`EncodedProperty::native_declaration` may be used only when:

- `LogicProfile::native_properties()` is supported;
- the selected concrete encoder spec defines a backend-native declaration whose
  semantics are no stronger and no weaker than the explicit property facts
  available in the VC context;
- the native declaration has its own `AtpDeclaration`, `symbol_map` row, and
  `AtpProvenance`;
- the encoding decision is recorded in `EncodedProperty` so candidate evidence
  and used-property reporting can refer to it.

Backend-native support is not a proof method and is not trusted acceptance
material. Backend-reported use of a native declaration remains advisory until
kernel evidence checking validates the corresponding formula/substitution
evidence.

Native AC-style declarations are allowed only when the native backend construct
matches the available property facts exactly. If a backend construct combines
associativity and commutativity, both facts must already be explicit and
available; commutativity alone must not be upgraded to AC.

Task 8 must not emit native declarations yet. At task-8 time the concrete
TPTP/SMT-LIB encoder specs that would define exact native semantics are still
later tasks, so native-property requests are classified as `deferred` or
unsupported/open-status outcomes even if `LogicProfile::native_properties()` is
supported. A later task may enable `EncodedProperty::native_declaration` only
after the relevant concrete encoder spec and tests define the exact semantics,
declaration shape, symbol-map row, and provenance.

## Provenance

Every encoded property row must have an `AtpProvenance` entry with
`AtpSourceRef::EncodedProperty`. The source binding must name the resolved
property fact and the target symbol identity. The provenance payload must be a
deterministic producer-side anchor derived from the resolved property fact; it
must not be backend text, a backend proof method, a backend log, a resolution
trace, an SMT proof object, an instantiated formula, or a SAT problem.

For native declarations, provenance belongs to both the `EncodedProperty` row
and the backend-visible declaration. The declaration provenance explains the
generated declaration; the property provenance explains the Mizar-side fact that
authorized the property.

## Determinism

Equivalent validated inputs must produce byte-identical `AtpProblem` debug
rendering and the same problem id. Property rows are ordered by deterministic
dense ids derived from canonical property identity. When task 8 builds property
rows, it must sort inputs before assigning dense ids and reject duplicate keys
before constructing an `AtpProblem`.

Diagnostics may mention unsupported property families or profiles, but
diagnostics do not participate in proof acceptance.

## Public Enum Forward Compatibility

Task 22 applies the frontend task-25 policy to the `property_encoding` module.
Public enums owned here are `#[non_exhaustive]` for downstream crates:
`AtpPropertyTargetKind`, `AtpPropertyEncodingStrategy`, and
`AtpPropertyEncodingError`.

Public enum inventory: `AtpPropertyTargetKind`, `AtpPropertyEncodingStrategy`, `AtpPropertyEncodingError`.

Future property families, strategies, or error variants must be specified
before source uses them. Inside `mizar-atp`, matches that affect generated
formulas, target declarations, provenance, backend-visible syntax, or proof
status must be explicit and fail closed unless a paired spec documents an
intentional fallback.

## Failure Semantics

- `source_drift`: if source later encodes a property family not specified here,
  add a paired English/Japanese spec update before implementation.
- `deferred`: associativity, transitivity, antisymmetry, compatibility,
  monotonicity, and domain-specific algebraic packages remain unspecified until
  a later task expands the family table.
- `external_dependency_gap`: property inputs from downstream proof/cache/artifact
  crates are unavailable until those crates define stable producer contracts.
- `boundary_violation`: treating backend-native property declarations, backend
  proof methods, logs, traces, or used-axiom reports as trusted acceptance
  material is forbidden.

Task-8 implementation must fail closed on malformed, duplicate, missing, or
unavailable property inputs. Unsupported-but-well-formed property/profile
combinations leave the VC open for another explicit profile or later pipeline
step; they do not create accepted proof status.

## Task-8 Test Expectations

Task 8 must add focused Rust coverage for:

- every supported family in axiom form;
- generated-binder declaration, symbol-map, provenance, canonical identity, and
  binder order for unary and binary property axioms;
- wrong target kind and arity for each function/predicate family group;
- missing declaration, symbol-map row, and provenance;
- profile rejection for missing quantifier/equality support;
- connectedness coverage that exercises the `Or` formula-tree branch, and
  connective-gate rejection if a future profile adds explicit connective
  capability flags;
- native declaration requests are deferred or fail closed until concrete
  encoder specs define exact native semantics;
- deterministic row order and problem identity under shuffled property inputs;
- duplicate source property identity and duplicate encoded identity rejection;
- absence of backend proof methods, backend logs, resolution traces,
  instantiated formulas, SAT problems, and accepted-proof status fields from the
  property API.
