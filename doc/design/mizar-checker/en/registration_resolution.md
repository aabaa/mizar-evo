# mizar-checker: Registration Resolution

> Canonical language: English. Japanese companion:
> [../ja/registration_resolution.md](../ja/registration_resolution.md).

## Purpose

`registration_resolution` specifies phase-7 registration handling after
phase-6 type checking has produced `TypedAst`, normalized types, and type
facts. It refines:

- [architecture 04](../../architecture/en/04.type_and_registration_resolution.md)
  Step 5, "Resolve Registration Closure";
- [architecture 04](../../architecture/en/04.type_and_registration_resolution.md)
  Step 6, "Validate Pending Registration Declarations";
- [architecture 17](../../architecture/en/17.cluster_trace_format.md)
  replayable cluster and reduction trace requirements;
- [spec chapter 17](../../../spec/en/17.clusters_and_registrations.md)
  existential, conditional, functorial, and reduction registrations;
- [`typed_ast.md`](./typed_ast.md) fact, obligation, diagnostic, and recovery
  tables;
- [`type_checker.md`](./type_checker.md) normalized type, coercion, and
  fact-query contracts.

Task 13 is a specification-only task. It adds no Rust source, no active
registration inference, no `ResolutionTrace` implementation, no verifier
policy, no `VcId` assignment, and no proof acceptance. Tasks 14 and 16-20
implement the named sections below.

## Boundary

`registration_resolution` owns:

- the checker-side split between pending registration declarations and
  activated registrations;
- deterministic registration ids, indexes, status records, diagnostics, and
  source-contribution tracking inside the checker boundary;
- validation of checker-ready registration payloads for well-formed patterns,
  compatible referenced symbols, parameter typing, and stable provenance;
- emission of checker-local `InitialObligationId` records for `existence`,
  `coherence`, and `reducibility` conditions;
- cluster fact closure over activated conditional and functorial registrations;
- existential gating for attributed type use;
- reduction normalization by activated reductions, with provenance sufficient
  for the canonical `ResolutionTrace`;
- deterministic partial output and diagnostics after recoverable registration
  errors.

`registration_resolution` does not own:

- parser or resolver extraction of registration syntax;
- resolver symbol allocation, label lookup, visibility, import/export checks,
  or opaque signature-shell construction;
- public artifact schemas for activated registration summaries;
- `VcId`, `ObligationAnchor`, VC generation, ATP search, proof acceptance,
  verifier policy decisions, or kernel replay;
- final overload root selection, active refinement joining, or inserted
  overload-disambiguating views;
- public diagnostic-code allocation while the checker diagnostic code-space
  remains an external planning gate.

Only activated registrations may contribute automatic type facts, reduction
steps, or attributed-type existence gates. Pending, malformed, unverified,
failed, or externally blocked registrations may be recorded for diagnostics and
local metadata, but they must not fire or satisfy gates.

## Inputs And Outputs

The registration resolver consumes:

- one `TypedAst` from phase 6;
- normalized type entries and type facts from `type_checker`;
- deterministic fact queries from `TypeFactQueryEngine`;
- resolver `SymbolEnv` registration declarations as origins and identity
  anchors;
- checker-ready registration payloads when a resolver/source-extraction task
  supplies them;
- activated dependency registration summaries when artifact/reuse integration
  supplies accepted summaries;
- an accepted verifier-status input only when a later proof/artifact phase
  exposes one.

The registration resolver produces a checker-local phase-7 output:

```rust
struct RegistrationResolutionOutput {
    typed_ast: TypedAst,
    registrations: RegistrationDatabase,
    facts: TypeFactTable,
    trace: ResolutionTrace,
    diagnostics: TypeDiagnosticTable,
}
```

This shape is logical. Task 14 may implement the database before task 15
implements the concrete trace module. No phase-7 output in this document is a
stable published artifact schema.

## Registration Data Model

The checker maintains two distinct stores.

```rust
struct RegistrationDatabase {
    pending: PendingRegistrationTable,
    activated: ActivatedRegistrationIndex,
    rejected: RejectedRegistrationTable,
    diagnostics: TypeDiagnosticTable,
    initial_obligations: InitialObligationTable,
}

struct PendingRegistration {
    id: CheckerRegistrationId,
    resolver_registration: RegistrationId,
    symbol: Option<SymbolId>,
    label: LabelRef,
    kind: RegistrationKind,
    pattern: RegistrationPatternStatus,
    parameters: Vec<TypedRegistrationParameter>,
    correctness: CorrectnessCondition,
    source: RegistrationSource,
    contribution: SourceContributionId,
    status: PendingRegistrationStatus,
    obligations: Vec<InitialObligationId>,
}

struct ActivatedRegistration {
    id: CheckerRegistrationId,
    resolver_registration: RegistrationId,
    label: LabelRef,
    kind: RegistrationKind,
    validation_kind: Option<RegistrationValidationKind>,
    pattern: RegistrationPattern,
    parameters: Vec<TypedRegistrationParameter>,
    correctness: AcceptedCorrectness,
    source: RegistrationSource,
    contribution: SourceContributionId,
    activation: ActivationEvidence,
}
```

`RegistrationPatternStatus` distinguishes usable checker-ready payloads from
missing, malformed, unsupported, or external-gap payloads. `ActivationEvidence`
is not generated by the checker from obligation creation; it must reference an
accepted dependency summary or an accepted verifier status supplied by a later
phase.

The checker-side `ActivatedRegistrationIndex` is not the resolver-side
`RegistrationIndex`. The resolver index supplies declaration identities,
symbol links, kind, target shell, visibility/export metadata, dependency
mentions, recovery state, and source contributions before checker validation.
The checker index contains only accepted registrations with typed patterns and
accepted correctness evidence. From task 20 onward, activated records also
carry the accepted checker validation kind when the activation input or a valid
companion validation payload provides it; consumers that require a specific
cluster subkind, such as existential gates, must check this field rather than
trusting a caller-supplied candidate tag.

## Pending Versus Activated Databases

The pending table records registration declarations seen in the current module
or imported through a not-yet-accepted local summary. Pending records are
allowed to carry diagnostics, `InitialObligationId`s, source contribution
links, and enough stable identity to become active later. They are never used
for automatic inference.

The activated index contains:

- accepted dependency registrations imported through verified summaries;
- accepted local registrations from a prior accepted pass or explicit accepted
  verifier-status input;
- no registration whose well-formedness, correctness obligation, or activation
  evidence is missing.

Activation is item-ordered and deterministic. A registration may be active for
subsequent items only after its own correctness condition has been accepted and
the checker input for that pass includes the accepted status. Earlier items are
not rechecked retroactively by treating the registration as if it had always
been active. In a single pass without accepted proof/artifact input, a new
local registration remains pending even if the checker successfully emits its
initial obligation. That no-accepted-input state is an interim conservative
approximation of the spec 17.1 asynchronous-acceptance contract: it may leave
the module or dependent use sites pending, but it is not a final language rule
that a preceding local registration can never license later items. Once
`mizar-vc`, `mizar-proof`, and artifact integration supply accepted status in
source order, the completed checker pass must activate accepted local
registrations for later items while keeping earlier items non-retroactive.

Ordering requirements:

1. Resolver-origin ids and source contribution ids are preserved.
2. Activated trigger lists sort by canonical trigger key, then origin module
   path, declaration source order, label FQN, registration kind, and
   fingerprint.
3. Pending and rejected tables render by source contribution, declaration
   order, registration kind, and label.
4. Hash-map iteration, worker order, import order, and cache insertion order
   must not affect firing order or diagnostics.

### Task 14: Registration Index Data Layer

Task 14 implements this section as `src/registration_resolution.rs`.

The first implementation builds checker-owned pending, activated, rejected, and
diagnostic tables from resolver `SymbolEnv` registration declarations. Resolver
registrations are treated as identity and provenance records only: the checker
preserves resolver registration id, optional symbol id, resolver registration
kind, opaque target-shell classification, visibility, export status, normalized
origin, source contribution, dependencies, and recovery state.

Pending records created from resolver entries without accepted checker-owned
activation input are marked as `external_dependency_gap` and never contribute
automatic facts, reductions, or existential gates. Malformed resolver target
shells become rejected records. Activated records can be created only through
explicit caller-supplied activation input that includes the resolver kind,
trigger key, accepted checker-owned pattern key, accepted correctness key,
activation evidence key, and, for subkind-sensitive consumers added in later
tasks, the accepted checker validation kind. Activation evidence alone is not
sufficient.

Task 14 deliberately does not parse opaque resolver target shells, validate
semantic registration patterns, create `InitialObligationId`s, accept proofs,
read artifact summaries, compute cluster closure, apply reductions, satisfy
existential gates, or produce `ResolutionTrace` steps. Later tasks may consume
the task-14 data layer, but they must continue to treat MC-G021 payloads as
external until an explicit checker-owned payload seam is available.

Task 14 canonical ordering:

1. pending and rejected records sort by source contribution id, origin
   structural path, resolver registration id, resolver registration kind,
   label/symbol fallback key, and rejection reason when present;
2. activated trigger lists sort by trigger key, origin module path, origin
   structural path, resolver registration id, label/symbol fallback key,
   resolver registration kind, fingerprint or pattern fallback key, and
   checker registration id;
3. debug rendering uses the same checker-owned order, never resolver map or
   worker iteration order.

## Validation Obligations

Task 19 implements validation. Validation starts from a checker-ready
registration payload and the resolver declaration identity.

Required checks:

- the resolver declaration exists and has a registration kind compatible with
  the checker payload;
- the label resolves to a registration label and can be bound to a stable
  checker registration id;
- all referenced attributes, modes, structures, functors, terms, and type
  heads resolve to compatible symbols;
- surrounding registration parameters are well typed and their local facts are
  available only through context visibility rules;
- existential patterns contain an attributed normalized type with at least one
  attribute;
- conditional patterns contain antecedent and consequent attribute sets over a
  compatible normalized type;
- functorial patterns contain a typed functor result pattern and consequent
  attributes over the declared result type;
- reduction patterns contain typed `LHS` and `RHS` terms, every free pattern
  variable in `RHS` occurs free in `LHS`, variable occurrence counts do not
  increase, and `LHS` is strictly larger than `RHS` under the fixed
  simplification order from spec 17.6.4;
- source provenance and source contribution ids are present for diagnostics,
  trace replay, and dependency fingerprints.

Validation outputs a pending registration plus one or more
`InitialObligationId`s:

| Registration kind | Correctness condition | Checker-local obligation |
|---|---|---|
| Existential | `existence` | inhabited attributed type witness |
| Conditional | `coherence` | antecedent attributes imply consequent attributes |
| Functorial | `coherence` | matched functor result has consequent attributes |
| Reduction | `reducibility` | universally quantified equality `LHS = RHS` |

Creating an initial obligation does not discharge it. The checker must not
store `VcId`, prover output, kernel acceptance, or accepted verifier status in
place of the local `InitialObligationId`.

Validation failures produce rejected pending records or degraded pending records
with diagnostics. They do not produce activated registrations and do not affect
cluster closure, reduction normalization, or existential gating.

### Task 19: Pending Validation And Activation Gating

Task 19 implements this section as an explicit-payload data layer in
`src/registration_resolution.rs`. The checker consumes caller-supplied
`RegistrationValidationInput` payloads keyed by resolver registration id. It
does not parse resolver opaque target shells, walk raw syntax, read artifact
summaries, or infer missing checker-ready registration payloads.

Without accepted activation evidence, validated payloads create pending registrations with
`RegistrationPatternStatus::Validated(...)`, checker-local
`InitialObligationId`s in an `InitialObligationTable`, and
`PendingRegistrationStatus::AwaitingVerifierAcceptance`. These records are
still pending: they always report `inference=false` and cannot contribute
cluster facts, reduction steps, or existential gates. The emitted obligations
use `InitialObligationKind::RegistrationCorrectness` and remain
`InitialObligationStatus::Pending`; task 19 does not assign `VcId`s, verifier
results, proof witnesses, kernel acceptance, or public artifact ids.

The checker validates existential, conditional, functorial, and reduction
payload shapes through checker-owned fields. Existential, conditional, and
functorial payloads require resolver `Cluster` declarations; reduction payloads
require resolver `Reduction` declarations. Recovered resolver origins, missing
or incompatible referenced symbols, invalid parameters, missing correctness
conditions, malformed kind-specific patterns, and missing source provenance are
rejected with deterministic checker-local diagnostics. Reduction validation
checks the fixed spec-17.6.4 order: every `RHS` free variable must occur in
`LHS`, `RHS` occurrence counts must not exceed `LHS` counts, and the
alpha-normalized structural size supplied by the checker-ready term payload
must satisfy `size(LHS) > size(RHS)`. The caller does not choose or certify a
custom termination order.

Activation remains gated by accepted external evidence. `ActivationInput`
continues to be the only way task-14 and later closure/reduction code create
activated registrations, and task 19 requires that input to carry accepted
verifier or artifact status. Missing or rejected status is diagnosed as
unaccepted activation evidence and never creates an active record. A valid
pending registration plus a generated obligation is not proof acceptance.
When accepted activation evidence is supplied together with a checker-ready
validation payload for the same resolver registration, task 19 validates that
companion payload first. If it is invalid or duplicated, activation is rejected.
If an explicit accepted validation kind is also supplied, it must agree with the
validated companion kind and must be compatible with the resolver registration
kind.
If it is valid, the accepted activation record is created and the companion
payload does not emit a new pending obligation in that pass; the accepted
status is the external proof/artifact input, not an obligation generated by the
checker.

Task 19 deliberately keeps the following deferred:

- source-to-checker extraction of registration validation payloads from `.miz`
  syntax;
- production or import of accepted verifier/artifact status;
- active `.miz` semantic fixtures for registration validation;
- artifact emission/reuse of pending-validation or activation decisions.

## Existential Gating

Task 20 implements attributed-type existence checks. The checker consults
activated existential registrations when a source construct introduces or
requires a value of an attributed type:

- `let x be A T`;
- mode definitions whose definiens contains attributes;
- functor return types with attributes;
- `consider`, `given`, and `take` contexts that claim witnesses of attributed
  types;
- later checker-owned surfaces that explicitly require inhabited attributed
  types.

An existential gate succeeds only when an activated existential registration,
with all parameter and guard facts visible at the site, proves that the
attributed normalized type is inhabited. Pending registrations, generated but
unaccepted obligations, and missing proof status do not satisfy the gate.

If the gate is missing, the checker emits a type error and may keep degraded
typed output so later diagnostics remain useful. The degraded output must not
export the value as fully verified metadata and must not seed downstream facts
as if inhabitation had been proved.

Existential registrations are activation-checked registrations and may emit
`existence` obligations during validation, but they are not ordinary
attribute-propagation edges in cluster fact closure. Their automatic phase-7 use
is to satisfy attributed-type inhabitation gates.

### Task 20: Existential Gate Data Layer

Task 20 implements existential gating as an explicit-payload data layer in
`src/registration_resolution.rs`. The checker consumes `ExistentialGateInput`
records supplied by a later source-to-checker extraction task; it does not walk
raw syntax or parse resolver opaque shells.

Gate inputs identify the typed site, source range, full accepted attributed
type pattern key, activation trigger, requested attributes, required guard
keys, checker-produced guard evidence, and candidate existential evidence.
The pattern key is the full normalized attributed type/pattern fingerprint,
including parameter instantiation; matching by type head alone is insufficient.
This preserves parameterized existential registrations such as attributed
`Subset of X` gates.

Task 47 extends the same explicit-payload seam to unattributed base-shape
inhabitation from spec §17.3.4. An `ExistentialGateBaseEvidence` record may
satisfy only an unattributed gate whose normalized pattern exactly matches the
evidence pattern. Built-in `object` and `set` use `Builtin` coverage and no
guards. Accepted mode applications and bare schema type parameters use
`CompleteGuardSet` coverage and at least one visible consumable guard fact, so
the accepted mode obligation or schema-context assumption is explicit.
Structure constructor witnesses use either `ZeroFieldStructure` with no guards,
or `CompleteGuardSet` with one or more field-inhabitation guards; an empty
complete guard set is invalid for nonzero structures. Base evidence never
satisfies a gate with requested attributes; attributed types still require an
activated existential registration for the full attributed pattern.

A candidate can satisfy a gate only when it is bound to the same accepted
activation record:

- the registration is present in `RegistrationDatabase::activated()`;
- the active resolver kind is `Cluster`;
- the active record carries accepted validation kind `Existential`, derived
  from a valid companion validation payload or explicit activation input;
- the candidate is tagged `RegistrationValidationKind::Existential`;
- the accepted pattern, correctness key, activation evidence key, trigger, and
  optional activation fingerprint match the active record;
- the gate pattern matches the accepted pattern and the candidate covers every
  requested attribute;
- every required guard has visible and consumable checker-produced evidence,
  represented by `TypeFactId`s from the existing fact-query boundary.

Pending, rejected, external-gap, recovered, malformed, unaccepted, inactive,
non-existential, or mismatched candidates do not satisfy gates. Guard evidence
is not a caller assertion: it is a checker-owned proof that the relevant fact
query already found a visible consumable fact at the gate site. Later
integration may replace the explicit evidence payload with direct
`TypeFactQueryEngine` calls, but task 20 does not fabricate facts.

Gate result precedence is deterministic:

1. `DegradedRecovery` if the gate site is degraded;
2. `Satisfied` if a valid candidate has all required guard evidence, or valid
   base evidence has all required guard evidence;
3. `BlockedGuard` if a valid candidate or valid base evidence is present but a
   guard is missing, invisible, or non-consumable;
4. `InvalidCandidate` if candidates or base evidence are present but none are
   valid;
5. `MissingExistential` if no candidates or base evidence are available.

Only `Satisfied` gates may seed verified downstream facts. Satisfied
base-evidence gates have no checker registration id but record the base
evidence kind/coverage and any visible guard fact ids. Missing, blocked-guard,
invalid-candidate, and degraded-recovery gates emit checker-local diagnostics,
keep degraded output only for follow-on diagnostics, and must not export
inhabited metadata or seed verified facts.

Task 20 deliberately keeps the following deferred:

- source-to-checker extraction of attributed-type gate sites;
- production/import of accepted verifier or artifact status beyond the task-19
  activation input seam;
- artifact emission/reuse of gate decisions;
- active `.miz` semantic fixtures for existential gates.

Task 47 additionally keeps source-derived base-shape execution deferred until
source-to-checker extraction can prove the accepted mode tuple, complete
structure-field coverage, schema-context assumption, and activation item order
from real source payloads. The current implementation consumes only explicit
checker-owned base evidence and must not infer missing guards from type spelling
or resolver opaque shells.

## Cluster Closure

Task 16 implements closure over activated conditional and functorial
registrations, while task 17 implements deterministic loop and saturation
diagnostics.

Closure rules:

1. Normalize multi-consequent registrations into single-consequent internal
   rules before firing.
2. Initialize the fact set from the normalized type, explicit attributes, local
   assumptions visible through context rules, and already recorded consumable
   facts.
3. Fire only activated registrations whose antecedents and parameter guards are
   satisfied by consumable facts.
4. Apply conditional registrations registered for a type head `T` to compatible
   subtypes of `T`, using the same subtype relation and recorded facts consumed
   by phase-6 fact queries. Exact-head matching is insufficient.
5. Add each new derived fact with `FactProvenance::Registration` linked to the
   corresponding resolution step.
6. Record every step in `ResolutionTrace`; hidden transitive closure is
   forbidden.
7. Stop at a deterministic fixed point, a contradiction, a loop diagnostic, or
   a configured saturation bound.

The restricted no-argument cluster adjective grammar from spec §17.7.1 is the
language-level termination argument for closure. The configured task-17
saturation bound is a defensive diagnostic and cache-key input, not permission
to export a successful truncated fixed point. Contradictory derived attributes
are a registration-resolution diagnostic and must not be silently reconciled by
dropping one side. Spec §17.7.3 makes a closure-time contradiction a fatal,
non-recoverable soundness-boundary failure: the checker may report contextual
diagnostics, but it must not continue by degrading the affected facts into
exportable partial output. The current checker-local diagnostic class remains
`cluster_contradiction` until MC-G005 allocates public checker diagnostic codes;
the public diagnostic contract should map this family to the Chapter 22
cluster-inconsistency code rather than inventing a second semantic condition.

## Reduction Rewrites

Reduction normalization over activated reduction registrations is the eventual
phase-7 contract. Reductions are semantic rewrites; they are not parser
rewrites and they must preserve source provenance. Task 18 implements the
checker-local reduction trace data layer over explicit reduction payloads only;
full typed-term matching, traversal, rule search/selection from source terms,
and source-derived guard extraction remain deferred behind MC-G020 and MC-G021.
For a fixed rewrite site, normalization is deterministic over the typed term,
the in-scope activated reduction rule set, and the discharged side-condition
set that supplies stable local-fact, trace, or citation evidence for `such`
guards.

Required behavior:

- only activated reductions may match;
- matching is over typed terms and normalized guards, not raw syntax strings;
- each candidate must satisfy the type and attribute guards introduced by the
  registration parameters;
- each `such` side condition from surrounding registration parameters must
  already be available as recorded local-fact, trace, or citation evidence
  before the rule may apply; such side conditions are applicability guards and
  do not make the rule more specific;
- traversal is leftmost-innermost;
- rule selection discards non-applicable rules, compares `LHS` patterns by
  pattern subsumption first, uses position-wise §19.2.3 type/attribute guard
  comparison only when pattern comparison has no unique winner, treats missing
  correspondences and mixed guard winners as incomparable, excludes `such`
  guards from specificity, and then chooses the lexicographically smallest
  rule FQN as the stable tie breaker;
- each applied rewrite records source redex, target term, substitution,
  discharged guards, rule FQN, active rule-view fingerprint, selection key,
  redex path, enclosing-term fingerprint, and source provenance;
- the original source term remains available for diagnostics and LSP metadata.

The simplification-order check is part of registration validation. A reduction
whose `RHS` is not strictly smaller than its `LHS` is rejected before
activation. Runtime rewrite step limits must not be used to compensate for a
missing validation proof.

Task 46 aligns the checker-owned explicit-payload trace layer with this
signature: replay requires stable type, attribute, and `such` evidence; the
discharged side-condition set is part of the recorded reduction trace identity;
and `such` evidence remains excluded from the strategy-audit selection key.
Source-derived normalization results, rule search, and guard extraction stay
deferred under MC-G020/MC-G021/MC-G023/MC-G030 until real source payloads and an
active advanced-semantics runner exist.

## Diagnostics And Recovery

Diagnostics are deterministic and checker-local until a public checker
diagnostic code-space is allocated. Required classes include:

- missing checker-ready registration payload;
- incompatible resolver registration kind;
- malformed or unsupported registration pattern;
- missing or incompatible referenced symbol;
- invalid registration parameter;
- missing correctness condition;
- blocked obligation emission;
- unaccepted activation evidence;
- unavailable existential registration;
- cluster contradiction, loop, or saturation bound;
- invalid reduction orientation, substitution, guard evidence, or strategy
  audit key.

Recoverable diagnostics produce explicit pending, rejected, skipped, or degraded
records. They must not fabricate successful types, accepted obligations,
activated registrations, trace steps, or exported facts.

Cluster contradictions are excluded from recoverable diagnostic handling. They
are fatal soundness failures and stop export of the affected phase-7 output.

## Public Enum Policy

Task 31 applies the frontend task-25 public-enum decision procedure to this
module. All public checker-owned enums in `registration_resolution` are
forward-compatible API surfaces and must remain `#[non_exhaustive]`;
downstream consumers must keep wildcard or fallback arms. Checker-internal
matches may remain exhaustive over the currently represented variants when
implementing the specified behavior.

| enum | decision |
|---|---|
| `RegistrationPatternStatus` | Forward-compatible; pattern extraction states may grow as checker-ready payloads land. |
| `PendingRegistrationStatus` | Forward-compatible; pending states may grow with validation and artifact handoff policy. |
| `RejectedRegistrationReason` | Forward-compatible; rejection reasons may grow as validation surfaces expand. |
| `ResolverTargetShell` | Forward-compatible; resolver shell projections may grow with resolver payloads. |
| `RegistrationValidationKind` | Forward-compatible; validation categories may grow with additional registration families. |
| `RegistrationValidationPattern` | Forward-compatible; validation patterns may grow with richer semantic payloads. |
| `RegistrationReferencedSymbolRole` | Forward-compatible; referenced-symbol roles may grow as validation checks expand. |
| `ActivationVerifierStatus` | Forward-compatible; verifier/artifact status may grow when proof handoff is connected. |
| `ExistentialGateBaseEvidenceKind` | Forward-compatible; base-shape inhabitation evidence kinds may grow with additional source-extracted type shapes. |
| `ExistentialGateBaseEvidenceCoverage` | Forward-compatible; base-evidence coverage categories may grow as structure and mode payloads become richer. |
| `ExistentialGateRecovery` | Forward-compatible; existential-gate recovery states may grow with source extraction. |
| `ExistentialGateStatus` | Forward-compatible; existential-gate outcomes may grow with evidence and artifact reuse. |
| `RegistrationDiagnosticClass` | Forward-compatible; diagnostic classes may grow before public checker diagnostic codes are allocated. |
| `RegistrationDiagnosticSeverity` | Forward-compatible; diagnostic severity policy may grow with IDE/artifact consumers. |
| `RegistrationDiagnosticRecovery` | Forward-compatible; diagnostic recovery states may grow with partial registration policy. |

No exhaustive public enum exceptions are owned by this module.

## External Dependency Gaps And Deferrals

| ID | Class | Evidence | Required action |
|---|---|---|---|
| MC-G005 | `spec_gap` / `external_dependency_gap` | No public checker diagnostic code-space exists yet. | Keep task-local diagnostic classes and stable detail keys private until the owning spec/design allocates public codes. |
| MC-G019 | `external_dependency_gap` | Statement/proof assumptions, theorem acceptance payloads, and phase-7 trace fact payloads are not available to task 11 fact queries. | Registration tasks may query only existing checker fact tables and visible contexts. They must not fabricate accepted proof facts. |
| MC-G020 | `external_dependency_gap` / `deferred` | There is no AST-wide source-to-checker extraction API for the checker-owned payloads used by tasks 7-11. | Registration tasks must consume explicit checker-owned registration payloads when available and keep source `.miz` semantic coverage deferred until extraction exists. |
| MC-G021 | `external_dependency_gap` / `deferred` | The current resolver registration index exposes declaration identity, kind, opaque target shell, visibility/export metadata, dependencies, recovery state, and source contribution, but not checker-ready typed registration patterns, parameter type payloads, correctness-condition anchors, accepted verifier status, active dependency-summary consumption, reduction `LHS`/`RHS` term payloads, or guard-evidence payloads. Task 19 consumes explicit validation payloads and validates them, but still does not source-extract those payloads or create accepted status. | Task 14 may use resolver registrations as identity/origin records only. Tasks 16-20 must use explicit checker-owned payload seams or defer behavior rather than parsing opaque shells, inventing summaries, or treating emitted obligations as accepted. |
| MC-G025 | `external_dependency_gap` / `deferred` | Task 19 emits checker-local registration-correctness `InitialObligationId`s and gates activation on accepted verifier/artifact status, but the proof/artifact phase that creates or imports that accepted status is not wired to `mizar-checker`. This is an interim conservative approximation of spec 17.1's asynchronous acceptance contract, not a final rejection policy for later source items. | Keep valid local registrations pending until explicit accepted status input is supplied. Do not promote generated obligations to activated registrations. Lift the approximation when accepted status production/import is connected so accepted earlier registrations can license later items in source order. |
| MC-G026 | `test_gap` / `external_dependency_gap` / `deferred` | Task 20 implements existential gates over explicit payloads, and task 47 extends that explicit seam to base-shape inhabitation evidence. Source-to-checker extraction of attributed/base gate sites, accepted mode tuples, complete structure-field coverage, schema-context assumptions, accepted-status production/import beyond task-19 activation input, artifact emission/reuse, and active `.miz` existential gate fixtures are not wired. | Keep task-local Rust coverage over explicit gate and base-evidence payloads. Defer real source-derived gate cases and artifact reuse until the owning extraction/proof/artifact tasks provide inputs. |

## Planned Tests

Task 14:

- pending entries never fire in closure, reduction, or existential gates;
- activation moves accepted entries into deterministic trigger lists;
- rejected and external-gap entries remain visible only to diagnostics;
- source contribution ids round-trip through pending, activated, and rejected
  records.

Task 16:

- conditional and functorial closure reaches a deterministic fixed point;
- conditional clusters apply to compatible subtypes, not only exact type heads;
- pending, rejected, unaccepted, and external-gap registrations do not
  contribute cluster facts even when their antecedents would match;
- every derived fact has registration provenance and a trace step;
- multi-consequent registrations are normalized into single-consequent rules;
- repeated runs produce identical fact and trace order.

Task 17:

- direct and indirect cluster loops terminate with stable diagnostics;
- contradictory cluster derivations are fatal and do not export a truncated or
  degraded verified fact set;
- saturation bounds are configuration-visible and included in deterministic
  rendering;
- truncated closure is not exported as verified output.

Task 18:

- redex paths, substitutions, guard evidence, rule FQNs, selection keys,
  source redexes, target terms, active rule-view fingerprints,
  enclosing-term fingerprints, and source provenance are recorded for every
  reduction;
- `such` side conditions must have recorded local-fact, trace, or citation
  evidence before a rule applies and do not affect specificity ranking;
- pending, rejected, unaccepted, and external-gap reductions never rewrite
  terms even when their patterns would match;
- invalid reduction substitutions and mismatched strategy-audit keys are
  rejected with stable diagnostics;
- unguarded or unsupported matches are rejected;
- deterministic rule selection is independent of insertion or import order.

Task 19:

- malformed patterns and missing referenced symbols are rejected;
- kind-specific validation covers attributed existential patterns, compatible
  conditional type heads, functor result patterns, and reduction
  free-variable, occurrence-count, simplification-order, and source-provenance
  requirements;
- validation emits `InitialObligationId`s without assigning `VcId`s;
- generated but unaccepted obligations never activate registrations;
- accepted verifier-status inputs are required before local activation.

Task 20:

- missing existential registrations fail attributed type use with stable
  diagnostics;
- activated existential registrations satisfy gates only when guards are
  visible;
- pending, rejected, unaccepted, and external-gap existential registrations do
  not satisfy attributed-type gates;
- degraded output after missing existence does not seed verified downstream
  facts.

## Task Mapping

- Task 14 implements the pending/activated database and deterministic indexes.
- Task 15 specifies the concrete `ResolutionTrace` shape consumed by tasks
  16-18.
- Task 16 implements cluster closure and trace recording.
- Task 17 implements loop and saturation diagnostics.
- Task 18 implements reduction rewrites with provenance.
- Task 19 implements pending registration validation and activation gating.
- Task 20 implements existential gating of attributed type use.

The implementation tasks must not begin by changing `doc/spec` or existing
`.miz` expectation files to match current source behavior. If a required input
is absent, classify it as an external dependency gap or deferral and keep the
behavior inactive.

## Task 251 Existential-Gate Transport Addendum

Task 251 may carry an authenticated `ExistentialGateInput` only as a supplied
dependency payload. The source-evidence producer verifies the request owner,
range, recovery, and existing guard-fact references, then transports the
snapshot without calling the gate evaluator or changing registration status.
Supplied transport is not an activated registration, satisfied gate, accepted
fact, or artifact result. Task 274 and its external accepted-status authority
remain blocked-reserved.
