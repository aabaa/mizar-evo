# Module: translator

> Canonical language: English. Japanese companion:
> [../ja/translator.md](../ja/translator.md).

## Purpose

The `translator` module owns the deterministic translation from validated
`mizar-vc` `VcIr` obligations to backend-neutral `AtpProblem` values.
Translation is still candidate-production setup: it materializes already
available VC formulas, declarations, soft-type facts, provenance, and target
binding into the problem layer before any concrete TPTP/SMT-LIB encoder,
backend process, portfolio policy, or kernel check runs.

The translator does not prove a VC. It does not call ATP backends, SAT solvers,
or `mizar-kernel`, and it does not produce trusted acceptance material.

## Boundary Rules

The translator may:

- consume a validated `VcSet`, one `VcIr` with `VcStatus::NeedsAtp`, and the
  matching VC kernel-evidence handoff target binding;
- choose an explicit backend-neutral `LogicProfile` from caller-supplied
  profile policy and VC shape;
- translate VC formula references into structured `AtpFormulaTree` payloads;
- materialize only already available local context facts, explicit `PremiseRef`
  entries, generated VC facts, imported fact payloads, type facts, and encoded
  property inputs;
- derive canonical dense ids, declarations, symbol-map rows, provenance rows,
  and non-semantic diagnostics.

It must not:

- select additional premises outside the `VcIr` / handoff inputs;
- invent substitutions, repair binders, perform overload resolution, search
  clusters, insert implicit coercions, unfold definitions by fallback policy, or
  approximate unsupported formulas;
- treat proof hints, backend logs, backend proof traces, backend-reported
  `used_axioms`, resolution traces, SMT proof objects, instantiated formulas, or
  SAT problems as trusted problem fields;
- classify backend output as accepted proof status or publish proof witnesses,
  cache entries, or artifact status.

## Inputs

Translator construction consumes an immutable input bundle conceptually shaped
as:

```text
AtpTranslationInput
  vc_set
  vc
  kernel_handoff
  logic_profile
  formula_projection_table
  declaration_projection_table
  soft_type_projection_table
  imported_formula_payloads?
  property_inputs?
  diagnostics_policy?
```

`vc_set` and `vc` identify the ATP-eligible `NeedsAtp` obligation.
`kernel_handoff` supplies the validated target fingerprint and
formula/provenance material that later kernel evidence candidates must agree
with. `logic_profile` is explicit; the translator may reject the profile as
unsupported for the VC shape, but it must not silently change the profile to
make translation succeed.

The structured formula, declaration, and soft-type projection tables are the
only sources for `AtpFormulaTree` payloads, declaration kind/arity,
binder/sort relationships, and soft-type guards. They must be produced from
`mizar-core` / `mizar-vc` owned data by an earlier deterministic phase and
keyed by `VcFormulaRef`, context identity, declaration identity, or provenance
identity. Kernel handoff formula fingerprints and payload bytes are agreement
checks and provenance anchors; the translator must not parse them to recover
ATP formula trees or declarations. For materialized formula rows, the
`AtpProblem` provenance payload is a deterministic handoff-derived anchor, not
the caller-supplied projection payload bytes.

Missing structured projections are fail-closed producer input errors for the
implemented translator tasks. The translator must not reconstruct formulas,
declarations, or type guards from opaque kernel formula bytes, debug rendering,
backend text, display names, source text, or by running name resolution,
overload resolution, implicit-coercion insertion, cluster search, or
substitution invention.

The source `VcIr` must be status `NeedsAtp`. `Open`, `Discharged`,
`AssumedByPolicy`, `PolicyOpen`, `SkippedDueToInvalidInput`,
`DeferredExternal`, and `Error` obligations are not translated to ATP problems
by task-5/6 translator source. They remain owned by deterministic discharge,
policy, diagnostics, or later external integration.

## Target Binding

The translator must require the supplied VC handoff to target the same `VcId`
in the same `VcSet`. A mismatch fails closed before any `AtpProblem` is
constructed.

The resulting `AtpTargetBinding` is derived from the handoff target
fingerprint and a stable producer binding that names the VC/handoff source. The
snapshot-local `VcId` may participate in the `AtpProblem` semantic identity for
collation, but it is not by itself a stable target binding or proof-reuse key.

## Deterministic Translation Order

Equivalent validated inputs must produce byte-identical `AtpProblem` debug
rendering and the same problem id. Deterministic ordering applies before
constructing `AtpProblem`:

- profile selection is caller-explicit and has a stable profile name;
- declarations are keyed by canonical source/generated identity, then by
  declaration kind and arity;
- generated binder declarations are keyed by canonical binder context, not
  display spelling or traversal order;
- premise formulas follow the canonical sorted `VcIr` premise order after
  validating source class and stable source binding; duplicate premise
  references, duplicate source/formula identities, or repeated imported source
  tuples fail closed before problem construction;
- type guards and soft-type facts follow canonical context/source identity;
- encoded properties follow stable property identity and target symbol;
- symbol-map and provenance rows follow their canonical keys;
- diagnostics are sorted only for debug output and do not participate in
  semantic identity.

The translator must never use map iteration, pointer addresses, backend output
order, source range alone, display spelling alone, wall-clock time, random
state, or process ids as ordering keys.

## Formula Materialization

`VcFormulaRef::Core` and `VcFormulaRef::Generated` references must be lowered
only from the structured formula projection table to structured
`AtpFormulaTree` values without backend text. The translator may lower:

- constants, atoms, equality, connectives, and quantifiers supported by the
  selected `LogicProfile`;
- generated formula shapes such as conjunction, split goal, negation,
  implication, and quantified wrappers when all referenced payloads exist;
- diagnostic/error formula references only as fail-closed translation errors,
  not as axioms that prove anything.

If a referenced formula payload is missing, malformed, unsupported by the
selected profile, or would require alpha-repair/substitution invention, the
translator returns an unsupported/open translation result or fail-closed error
according to the failure class. It must not drop the formula, replace it with
`true`, approximate it, or move the obligation to accepted status.

## Premise Materialization

`vc.premises` is the authoritative premise list for task-6 axiom translation.
Each premise becomes an axiom only when the referenced formula payload and
provenance are available and valid for the selected profile.

The translator rejects duplicate premise references, duplicate source/formula
identities, and repeated imported source tuples instead of silently coalescing
them. Duplicate premises would make provenance and later backend `used_axioms`
identity ambiguous, so they are fail-closed producer input errors.

Allowed premise sources are:

- `PremiseRef::LocalContext` entries with explicit formula payloads;
- generated VC facts referenced by `PremiseRef::GeneratedFact`;
- imported facts only when the imported payload, statement fingerprint,
  required proof status, and formula context requirements are supplied by the
  VC/handoff inputs. The ATP projection symbol is not a substitute for the
  imported source tuple; repeated package/module/item/status/statement tuples
  are rejected even when they arrive under different imported symbols.

For the task-29 post-audit kernel contract, non-imported premise projections
must preserve the VC handoff source identity. A local hypothesis, cited
premise, or generated VC fact whose projection source binding is relabeled by
the ATP producer is not repaired into a new identity; it fails closed before an
`AtpProblem` or backend candidate is constructed. Imported fact projections
continue to rely on the kernel-validated statement projection payload supplied
through the VC handoff; the ATP translator does not replace that payload with a
symbol label, backend axiom label, or backend `used_axioms` record.

Checker-owned facts and type predicates are future conditional premise
families. Task-6 translator source keeps them fail-closed unless the VC handoff
exposes matching explicit source classes/projections. A generic policy-builtin
bucket is not enough for the ATP translator to claim checker-owned or
type-predicate provenance.

Unsupported premise references, conservative unknown markers, policy
assumptions, trace-only records, cluster/reduction trace labels, definition
unfolding policy markers, and proof-hint citations are not premises by
themselves. They may become axioms only after an earlier VC phase has already
materialized explicit formula/provenance payloads and placed those payloads in
`vc.premises` or the handoff data.

Proof hints may constrain later backend/profile/portfolio choices, but they do
not authorize the translator to add, drop, or prune `vc.premises`. `Only` and
`Exclude` restrictions remain diagnostic/profile/portfolio metadata unless an
earlier immutable VC phase has already reflected them in `vc.premises`. If a
task-5/6 translator observes that proof-hint restrictions cannot be reconciled
with the immutable premise list, it may emit diagnostics or an unsupported/open
translation outcome, but it must not mutate the premise set.

## Goal And Polarity

The VC goal is translated to the `AtpProblem.conjecture` formula. The
translator records `ExpectedBackendResult::Unsat` for every task-5/6 problem.
Task-6 source also requires the matching VC kernel handoff final-goal polarity
to be `AssertFalseForRefutation`; any other handoff polarity fails closed
before constructing the `AtpProblem`. The goal projection provenance binding
and projection source identity must both use the final-goal source binding
`goal:1`; premise-style generated, checker-owned, or local bindings cannot be
used for the conjecture.
Task 29 treats a proof-obligation handoff requesting
`AssertTrueForConsistency` as a non-candidate condition at this boundary; ATP
does not reinterpret proof obligations as consistency checks.
Concrete encoders may later present the goal as a TPTP conjecture, a negated
TPTP conjecture, or an SMT assertion of the negated goal, but the
backend-neutral problem contract remains that a successful backend result must
correspond to unsatisfiability of premises plus the negated goal.

The goal must not be copied into `axioms`, backend-reported `used_axioms`, or
trusted acceptance material.

## Declarations And Symbol Map

Every backend-visible symbol used by formulas, properties, generated binders,
type guards, or native property declarations must have:

- a unique `AtpDeclaration`;
- kind and arity matching every use;
- an `AtpSymbolMapEntry` linking the backend-safe symbol to canonical
  Mizar/core/generated identity;
- provenance for generated declarations or facts that can affect candidate
  evidence.

Name mangling for concrete encoders is deferred to TPTP/SMT-LIB encoder specs.
The translator owns only backend-neutral canonical symbols and the mapping
needed to make later diagnostics and candidate-evidence extraction traceable.
Declaration kind, arity, binder/sort relationships, and canonical source
identity come from the structured declaration projection table, not from
display names or backend text.

Duplicate declarations, missing declarations, missing symbol-map rows,
kind/arity mismatches, and noncanonical dense id derivation fail closed before
problem construction.

## Soft-Type Preservation

Soft type information from local context, generated type obligations, type
predicates, non-emptiness facts, sethood facts, subtype/coercion facts, guards,
and intersection-like facts must not be erased by selecting a sort-only
encoding.

The translator may use backend sorts only when the selected `LogicProfile`
records that the relevant type information is represented losslessly. Any
soft-type fact not represented by the sort must remain as a guard predicate,
explicit axiom, or type-context entry with provenance. If preservation cannot
be expressed in the selected profile, translation is unsupported/open for that
profile.

Soft-type guards and preservation requirements come from the structured
soft-type projection table. The translator must not infer them from symbol
spelling, backend sort conventions, or kernel handoff bytes.

## Provenance

Every declaration, axiom, conjecture, type guard, encoded property, and native
property declaration that can affect candidate evidence must have provenance
derived from VC/handoff inputs. Provenance records carry source bindings and
stable projection payloads; later producer tasks may refine those payloads to
fingerprints when their owner exposes a fingerprint field. They are not
backend logs or trace explanations.

Imported facts require package/module/item identity, statement fingerprint,
required proof status, and formula context requirements. For task-6 source the
formula-context requirement is bound to the VC handoff formula-context
fingerprint; missing or mismatched imported context fails closed.

## Failure Semantics

Translator failures are producer-side open-status or fail-closed diagnostics,
not proof results.

- malformed or missing required VC/handoff payloads fail closed;
- missing structured formula/declaration/soft-type projections fail closed;
- stale target binding, wrong VC, duplicate ids, duplicate symbols, duplicate
  premise identities, or provenance gaps fail closed;
- unsupported formula/profile features produce unsupported/open translation
  outcomes for that profile;
- profile unavailability, disabled ATP policy, or absent backend configuration
  are recorded for later backend/portfolio tasks and do not construct accepted
  proof status.

## Public Enum Forward Compatibility

Task 22 applies the frontend task-25 policy to the `translator` module. Public
enums owned here are `#[non_exhaustive]` for downstream crates:
`AtpSymbolSourceProjection`, `AtpSoftTypeRepresentation`,
`AtpFormulaProjectionTarget`, and `AtpTranslationError`.

Public enum inventory: `AtpSymbolSourceProjection`, `AtpSoftTypeRepresentation`, `AtpFormulaProjectionTarget`, `AtpTranslationError`.

Future variants must be specified before source relies on them. Inside
`mizar-atp`, matches that affect premise selection, formula provenance,
handoff agreement, backend-visible formulas, or proof status must be explicit
and fail closed unless a paired spec documents an intentional fallback.

## Gap Classification

- resolved `deferred`: task 4 specifies translator ownership and boundaries.
- resolved `source_drift`: task 5 defines Rust projection input structs for
  declaration and soft-type payloads, implements target/status handoff gates,
  deterministic declaration/symbol-map/type-context translation, and validates
  type-guard signatures without constructing a final `AtpProblem`.
- resolved `source_drift`: task 6 defines structured Rust formula projection
  targets for VC formula refs and imported facts, implements deterministic
  axiom/conjecture materialization, records `ExpectedBackendResult::Unsat`,
  checks final-goal handoff polarity for `AssertFalseForRefutation`,
  rejects duplicate premise refs and duplicate resolved formula/source
  identities, and checks projection fingerprints/provenance payloads against
  the matching VC kernel handoff without parsing handoff formula bytes.
- `deferred`: checker-owned and type-predicate premise materialization remains
  fail-closed unless the VC handoff exposes a matching explicit source
  class/projection. `mizar-atp` must not invent a placeholder source class.
- `deferred`: property encoding, concrete encoders, backend runner, portfolio,
  and candidate evidence extraction remain in their own module specs/tasks.
- `external_dependency_gap`: proof-policy winner selection, witness
  publication, and cache promotion remain outside `mizar-atp` until their owner
  crates define stable contracts.

## Planned Tests

Task 5/6 implementation together adds Rust coverage for:

- rejecting non-`NeedsAtp` VCs and stale/mismatched target handoffs;
- missing structured formula/declaration/soft-type projections fail closed;
- malformed formula/declaration/soft-type projections fail closed when they
  violate required shape, declaration kind/arity, or provenance invariants;
- unsupported formula/profile features and formulas requiring alpha-repair or
  substitution invention produce unsupported/open outcomes for that explicit
  profile, without silently switching profiles;
- deterministic declaration and symbol-map output under shuffled equivalent
  local context and generated formula inputs;
- duplicate, missing, or kind/arity-mismatched declarations fail closed;
- duplicate premise references or duplicate source/formula identities fail
  closed;
- proof hints and premise restrictions do not add, drop, or prune premises by
  themselves;
- imported facts with missing required proof status, statement fingerprint, or
  formula-context requirements fail closed;
- premise-order determinism and provenance completeness for local context,
  generated, and imported facts; checker-owned and type facts stay
  fail-closed until the VC handoff exposes matching explicit source classes;
- soft-type facts remain represented as sorts only when lossless, otherwise as
  guards/axioms/type-context entries or unsupported profile results;
- goal/conjecture polarity always records `ExpectedBackendResult::Unsat`;
- prohibited backend/kernel/SAT/proof-acceptance material is absent from the
  translator public API and debug rendering.
