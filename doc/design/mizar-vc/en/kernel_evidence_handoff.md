# Module: kernel_evidence_handoff

> Canonical language: English. Japanese companion:
> [../ja/kernel_evidence_handoff.md](../ja/kernel_evidence_handoff.md).

## Purpose

Task 24 defines the producer-side handoff from `mizar-vc` data to the corrected
`mizar-kernel` formula/substitution evidence format. The handoff is an
untrusted, prover-independent package shape. It is not proof acceptance and it
does not make `mizar-vc` a SAT checker, ATP backend, proof-policy owner, or
kernel caller.

The handoff maps a validated immutable `VcSet` and a selected `VcIr` into the
material that the kernel checker can parse and check:

- target VC binding;
- explicit goal polarity bound to the selected obligation kind;
- kernel profile;
- symbol and variable manifests needed by kernel formula validation;
- formula evidence entries for local hypotheses, cited premises, generated VC
  facts, accepted imported facts with imported-statement projection payloads,
  and policy-bounded built-ins;
- explicit substitution evidence when upstream payloads already exist;
- provenance bindings for every formula, substitution, and final goal;
- a standalone final goal record.

The trusted acceptance boundary remains inside `mizar-kernel`. The kernel
re-derives instantiated formulas from formulas and substitutions, constructs
the deterministic SAT problem itself, and accepts only after trusted
in-process Rust SAT checking reports the required UNSAT result.

## Boundary Rules

`mizar-vc` must remain prover-independent. The handoff builder added by task 25
may inspect existing VC data, canonical formula payloads, context entries,
premise references, discharge records, dependency slices, and provenance, but it
must not:

- run SAT solving or call `mizar-kernel`;
- call ATP backends or parse backend logs;
- select premises, invent substitutions, repair binders, resolve overloads,
  search clusters, insert implicit coercions, or perform fallback inference;
- add TPTP, SMT-LIB, DIMACS, SAT clauses, resolution traces,
  MiniSAT-compatible certificates, solver proof methods, instance/inverse
  methods, SMT proof objects, backend stdout/stderr, backend success flags, or
  backend `used_axioms` to `VcIr` or to the canonical kernel evidence input;
- build legacy `Certificate`, `LegacyCertificate`, or `LegacyResolutionTrace`
  objects as trusted handoff material.

Instantiated formulas and SAT clauses are not handoff fields. They are
kernel-derived acceptance material only.

## Conceptual Handoff Shape

Task 25 implements an immutable builder equivalent to this conceptual shape,
using concrete Rust types chosen to match the existing `VcIr` and kernel parser
APIs. The canonical evidence section matches the kernel v1 envelope fields and
section names:

```text
VcKernelEvidenceHandoff
  canonical_evidence
    schema_version
    encoding_version
    target_vc
    kernel_profile
    symbol_manifest
    variable_manifest
    formula_evidence
    substitutions
    provenance
    final_goal
  context_identity
    schema_version
    target_vc
    canonical_evidence_hash
    non_imported_source_binding_rows
    context_identity_hash
  formula_context_requirements?
  diagnostic_inputs?
```

Task 28 adds `context_identity` as a stable, non-diagnostic handoff section.
It is intentionally outside `canonical_evidence`: `canonical_hash()` continues
to name the formula/substitution/provenance envelope hash, and the context
section has its own non-recursive `context_identity_hash()`. The context hash
input records the target VC fingerprint, the canonical evidence hash, and one
deterministic row for every local-hypothesis, cited-premise, or
generated-VC-fact formula evidence entry. Each row binds the source class/id to
the formula evidence row id, formula fingerprint, and producer `VcFormulaRef`.
Imported axiom/theorem entries are covered by `formula_context_requirements`
instead and do not appear in `context_identity`.

The v1 `context_identity_hash()` input is the UTF-8 line grammar shared with
`mizar-kernel` task 31:

```text
vc-kernel-context-identity-v1
schema-version=1
target-vc={algorithm_id}:{lower_hex_digest}
canonical-evidence-hash={lower_hex_32_byte_hash}
[entries]
source={source_debug}; formula-id={formula_id}; fingerprint={algorithm_id}:{lower_hex_digest}; producer={producer_debug}
```

Rows are sorted by source kind/id, formula id, formula fingerprint, and
producer formula ref. Source spellings use the Rust debug forms
`LocalHypothesis { local_context_id: N }`,
`CitedPremise { local_context_id: N }`, and
`GeneratedVcFact { vc_fact_id: N }`; producer refs use
`Core(CoreFormulaId(N))` or `Generated(VcGeneratedFormulaId(N))`. Kernel task
31 treats `canonical_evidence_hash` as the opaque `mizar-vc` canonical
formula-envelope handoff hash and recomputes only this context-identity hash.

`formula_context_requirements` is not a canonical evidence-envelope section.
It records the immutable imported-fact context that must be supplied to
`mizar-kernel` as `FormulaEvidenceContext` before imported axioms or theorems
can be treated as accepted. `mizar-vc` may carry candidate source bindings and
required proof-status requirements, but it does not certify imported facts as
accepted. Missing or mismatched imported-fact context is a fail-closed builder
error, kernel rejection, or `external_dependency_gap`.
The builder rejects an empty context provenance fingerprint and returns imported
axiom/theorem requirements in canonical sorted, duplicate-free order. Imported
formula payloads must carry an explicit imported-statement projection payload:
the requirement's imported-statement fingerprint uses the architecture-18
imported-statement algorithm, while the cited formula tree keeps the kernel
formula fingerprint algorithm. The projection binds those two fingerprints and
is checked fail-closed by the producer before any imported entry is emitted.

`diagnostic_inputs` are optional producer-side details for explainability. They
are excluded from the canonical kernel evidence bytes, stable handoff identity,
and proof reuse identity unless a later spec explicitly promotes a stable
field.
Snapshot-local `VcId`, generated formula ids, context-entry ids, source ranges,
and handoff row ids may appear in diagnostics, but canonical evidence must bind
through stable formula fingerprints, target identifiers, source bindings, and
provenance records.

The task-25 target VC fingerprint is specific to the kernel handoff and excludes
`ProofHint` data. Proof hints, premise restrictions, solver preferences, and
diagnostic replay data can guide candidate production or explanations, but they
do not block target binding and do not enter canonical evidence hash input.

Task 27 makes the goal polarity an explicit producer input. The builder accepts
only a polarity that matches the selected `VcIr` obligation kind before package
assembly and canonical hash input construction. All currently implemented
`VcKind` variants are proof obligations, so their required handoff polarity is
`AssertFalseForRefutation`. A caller request pairing any current proof
obligation with `AssertTrueForConsistency` is rejected fail-closed with
`GoalPolarityMismatch`.

## Input Mapping

The builder input is a validated `VcSet`, a selected `VcIr`, and optional
producer-owned records already computed by prior VC phases:

| VC input | Kernel evidence mapping |
|---|---|
| `VcSet` schema, module, source, canonical VC fingerprint, and selected `VcIr` | `target_vc`, target provenance binding, and deterministic package identity. If a stable target binding cannot be computed, the builder fails closed. |
| selected `VcIr.kind` and `KernelEvidenceHandoffInput.goal_polarity` | The builder enumerates every current VC kind and requires `AssertFalseForRefutation` for proof obligations. The validated explicit polarity is copied into `final_goal.polarity`; a consistency-polarity request fails before canonical evidence bytes or package hashes are built. Kernel-side acceptance binding is implemented by `mizar-kernel` task 30. |
| `LocalContext` entries with formula refs | Formula evidence entries with local-hypothesis or cited-premise source bindings plus `context_identity` rows that bind the context-entry id, formula evidence row id, formula fingerprint, target VC, and canonical evidence hash. Entries without stable formula payloads or provenance are recorded as missing payloads, not fabricated. |
| `PremiseRef::LocalContext` and `PremiseRef::GeneratedFact` | References to the corresponding local-hypothesis, cited-premise, or generated-VC-fact formula evidence entries, with matching `context_identity` rows for the non-imported source binding. |
| `PremiseRef::ImportedFact` | Candidate imported axiom/theorem formula entries only when package/module/exported item identity, imported-statement fingerprint, imported-statement-to-formula projection payload, required proof-status requirement, and matching `FormulaEvidenceContext` input are available. `mizar-vc` does not certify the imported fact as accepted; proof/kernel-owned context must do that. Otherwise the premise is an `external_dependency_gap` or fail-closed builder error. |
| `PremiseRef::CheckerFact`, `TypePredicate`, trace, registration, cluster, reduction, definition, policy, and conservative-unknown variants | Mapped only when an explicit formula payload, allowed source class, target binding, and provenance are already available. Marker-only or trace-only records do not become trusted evidence. |
| `VcGeneratedFormula` table | Generated VC fact entries when the formula tree can be projected into the kernel-supported formula grammar and provenance binds the selected target. |
| `VcIr.goal` | The standalone `final_goal` record. It is never a premise and never a source of `used_axioms`. |
| `ProofHint` and premise restrictions | Diagnostic or candidate-production metadata only. They do not select premises, add premises, drop premises, or authorize acceptance. The builder may reference only exact premise refs already materialized in immutable `VcIr` inputs; restrictions that are not already reflected in those inputs stay diagnostic. |
| `DischargeEvidenceRecord` | Task 25 carries replayable input references as diagnostics outside canonical evidence and canonical hash input. A discharge rule name or evidence hash is not trusted acceptance material. Promoting deterministic discharge data into canonical formula/substitution/provenance evidence requires a later spec-backed task. |
| `DependencySlice` and proof-reuse candidate data | Identity and invalidation inputs for tasks 26 and 28. They include both the canonical formula-envelope hash and context-identity hash, but they do not prove the VC and do not replace kernel checking. |

The builder must preserve deterministic ordering. Missing formula payloads,
missing imported-fact identity, missing provenance, non-projectable formulas,
or absent substitution payloads are fail-closed builder errors or classified
deferred rows; they are not silently dropped from a claimed complete evidence
package.

## Formula Projection

Kernel task 25 currently supports a propositional formula tree over normalized
kernel atoms. `mizar-vc` may project a VC formula into that grammar only when
the source formula payload already supplies all required normalized atom,
symbol, variable, binder, and provenance data.

`mizar-vc` must not reconstruct formulas from display text, source ranges,
debug renderings, backend encodings, trace names, local ids, or proof-method
metadata. When `CoreFormulaId`, `VcFormulaRef`, or generated formula shape
cannot be resolved to a stable kernel formula tree, the builder records an
`external_dependency_gap` and returns no trusted handoff package for that VC.
Formula-tree fingerprints must use the kernel formula fingerprint algorithm for
this handoff version. Imported-statement fingerprints are not formula-tree
fingerprints; they must use the imported-statement algorithm id defined for
architecture-18 dependency fingerprints and must be connected to the cited
formula-tree fingerprint by the canonical v1
`KernelImportedStatementProjection` payload. That payload is
`MIZAR_KERNEL_IMPORTED_STATEMENT_PROJECTION\0` followed by the canonical bytes
of the imported-statement fingerprint and the canonical bytes of the formula
fingerprint. Unsupported formula or imported-statement algorithm ids, empty or
noncanonical projection payloads, stale statement fingerprints, and
formula-projection mismatches are fail-closed builder errors, not cues to
reinterpret bytes.

## Substitutions

Substitution evidence is explicit. A substitution record may be included only
when an upstream or producer-owned payload already provides:

- source formula id;
- binder-context encoding;
- `substitution_checker` payload;
- freshness witnesses and free-variable constraints;
- provenance binding to the target VC and source formula fingerprint.

The handoff must not contain an instantiated formula or target formula field
inside the substitution record. The kernel applies checked substitutions and
derives instantiated formulas during checking. Missing, stale, duplicate, or
inconsistent substitution records are builder failures or kernel rejections,
not repair opportunities.
Freshness witnesses and free-variable constraints are opaque kernel-compatible
encoded records at this boundary. Task 25 sorts them deterministically and
rejects empty or duplicate side-condition records; a later kernel/proof task can
replace the opaque producer-side payload with a richer typed schema if needed.

## Legacy And Prohibited Material

Legacy resolution-trace certificates are migration/audit-only material under
the corrected evidence pipeline. Normal proof policy treats them as unsupported
and they cannot produce kernel-accepted status, proof witnesses, artifact
`kernel_verified` status, cache promotion, or trusted `used_axioms`.

The VC handoff must therefore exclude:

- TPTP or SMT-LIB problems;
- DIMACS or SAT clauses;
- instantiated formulas supplied by callers;
- resolution traces and MiniSAT-compatible certificates;
- backend proof methods, instance methods, inverse methods, SMT proof objects,
  and backend logs;
- backend `used_axioms`, success flags, timings, or stdout/stderr;
- legacy certificate parser outputs as accepted evidence.

## Gap Classification

Task 24 records the corrected handoff contract and updates the closeout-era
classification that treated `mizar-kernel` as absent. Kernel tasks 23-29 now
provide the formula/substitution evidence schema, deterministic instantiation
and SAT encoding, trusted SAT checker wrapper, SAT-backed check service, and
legacy-audit gating. The VC side is still producer-side only.

Remaining gaps:

- `external_dependency_gap` `VC-HANDOFF-G001`: source-derived stable full core
  formula payloads, definition payloads, quantified binder payloads, and some
  generated obligation payload families are still incomplete upstream.
- `external_dependency_gap` `VC-HANDOFF-G002`: imported fact package/module/item
  identity, required proof-status payloads, and immutable
  `FormulaEvidenceContext` inputs are not yet uniformly available for every
  `PremiseRef::ImportedFact`.
- `external_dependency_gap` `VC-HANDOFF-G003`: ATP candidate evidence
  production, proof witness policy, cache consumers, and artifact witness
  consumers remain downstream work.
- resolved `VC-HANDOFF-G004`: task 25 adds the immutable Rust handoff builder,
  canonical rendering/hash input, builder errors, lint-policy registration, and
  focused tests over explicit producer payloads.
- resolved `VC-HANDOFF-G005`: task 26 updates dependency-slice and
  proof-reuse identity so the current canonical kernel evidence hash
  participates in reuse invalidation without becoming proof acceptance
  material. Missing, duplicate, unknown-VC, or selected-VC-mismatched handoff
  inputs fail closed.
- resolved `VC-HANDOFF-G006`: task 27 adds explicit `goal_polarity` to the
  handoff input, records the validated value in `final_goal.polarity`, and
  rejects consistency polarity for every current proof-obligation VC kind before
  canonical package assembly. Kernel-side check-service enforcement is
  implemented by `mizar-kernel` task 30.
- resolved `VC-HANDOFF-G007`: task 28 adds the producer-side
  `context_identity` payload and hash for local-hypothesis, cited-premise, and
  generated-VC-fact formula evidence rows. The payload is bound to the target
  VC and canonical evidence hash and participates in dependency-slice /
  proof-reuse identity. Kernel-side membership verification is implemented by
  `mizar-kernel` task 31.
- resolved `VC-HANDOFF-G008`: task 29 adds the producer-side
  `KernelImportedStatementProjection` payload for imported axiom/theorem
  entries. The handoff now validates an architecture-18 imported-statement
  fingerprint separately from the kernel formula-tree fingerprint, records the
  projection in canonical evidence rendering/hash input, and exports the same
  payload to dependency slices. Kernel-side trusted validation and the pass
  fixture are implemented by paired `mizar-kernel` task 33.

## Planned Tests

Task 25 adds Rust coverage for:

- deterministic handoff rendering and canonical byte/hash input stability;
- local context, premise, generated formula, final goal, and provenance
  mapping;
- imported fact payload completeness and fail-closed missing identities;
- substitution payload inclusion without instantiated-formula fields;
- discharge records contributing only replayable diagnostics, not trusted rule
  names, evidence hashes, or canonical evidence fields;
- public API absence of backend text, SAT clauses, resolution traces, backend
  proof methods, and legacy certificate acceptance fields;
- missing formula/provenance/substitution payloads returning builder errors or
  classified deferred records.

Task 26 adds invalidation tests showing that proof-reuse identity changes when
the canonical kernel evidence hash changes, remains unavailable when no current
kernel evidence handoff is supplied, and rejects duplicate, unknown, or
selected-VC-mismatched kernel evidence handoff inputs. Downstream
proof/cache/artifact schemas remain external/deferred.

Task 27 adds Rust coverage showing that a normal proof-obligation handoff
declares `AssertFalseForRefutation` explicitly and that a caller request for
`AssertTrueForConsistency` fails closed with the stable
`GoalPolarityMismatch` diagnostic.

Task 28 adds Rust coverage showing that `context_identity` covers every
non-imported local-hypothesis, cited-premise, and generated-VC-fact source
binding, excludes imported premises, is stable and self-consistent, and becomes
stale if a canonical source binding is mutated.

Task 29 adds Rust coverage showing that imported facts can carry a distinct
imported-statement fingerprint and formula-tree fingerprint, that the
projection participates in canonical evidence/debug rendering and dependency
slices, and that unsupported imported-statement algorithms, stale statement
projections, formula-projection mismatches, conflicting same-requirement
projections, empty or noncanonical projection payloads, and missing
context/payload data fail closed.

## Public Enum Policy

Task 25 classifies every `kernel_evidence_handoff` public enum as a downstream
forward-compatible API surface. Each enum must keep `#[non_exhaustive]` so
later kernel profiles, imported-fact classes, proof-status requirements,
formula source variants, goal polarities, builder errors, and role diagnostics
can be added without breaking downstream exhaustive matches.

| public enum | decision |
|---|---|
| `KernelClauseTautologyPolicy` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `KernelCertificateHashInputAlgorithm` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `KernelImportedFormulaClass` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `KernelRequiredProofStatus` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `KernelFormulaSource` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `KernelContextIdentitySource` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `KernelGoalPolarity` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `KernelEvidenceHandoffError` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `KernelEvidenceRole` | `#[non_exhaustive]` downstream forward-compatible surface. |

No exhaustive public enum exceptions are owned by this module. Internal
`mizar-vc` matches that intentionally enumerate current variants may remain
exhaustive.

## Current Step-3 Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue Step 3 with `mizar-kernel` task 34 after the completed mizar-kernel
task 33 imported-statement projection validation. Before editing, verify a
clean worktree, confirm the task 33 commit, and re-read doc/design/todo.md,
doc/design/mizar-kernel/en/todo.md,
doc/design/mizar-kernel/en/soundness_argument.md,
doc/design/mizar-kernel/en/checker.md,
doc/design/mizar-kernel/en/resolution_trace.md,
doc/design/architecture/en/15.kernel_certificate_format.md, and this
kernel-evidence handoff spec. Pin down or retire the legacy resolution-trace
tautology marker without changing corrected formula-evidence acceptance,
activating unverified fixtures, fabricating runner payloads, or rebaselining
expectations to match current behavior.
```

Rationale: task 34 closes the remaining F9 legacy-audit-path ambiguity after
F6 is implemented on both producer and kernel-consumer sides. Keep `xhigh`
because the work touches the rejection taxonomy for a migration-only trusted
boundary; lower reasoning is appropriate only for typo-only documentation
synchronization.
