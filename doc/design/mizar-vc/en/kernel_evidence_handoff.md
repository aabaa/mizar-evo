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
- kernel profile;
- symbol and variable manifests needed by kernel formula validation;
- formula evidence entries for local hypotheses, cited premises, generated VC
  facts, accepted imported facts, and policy-bounded built-ins;
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
  formula_context_requirements?
  diagnostic_inputs?
```

`formula_context_requirements` is not a canonical evidence-envelope section.
It records the immutable imported-fact context that must be supplied to
`mizar-kernel` as `FormulaEvidenceContext` before imported axioms or theorems
can be treated as accepted. `mizar-vc` may carry candidate source bindings and
required proof-status requirements, but it does not certify imported facts as
accepted. Missing or mismatched imported-fact context is a fail-closed builder
error, kernel rejection, or `external_dependency_gap`.
The builder rejects an empty context provenance fingerprint and returns imported
axiom/theorem requirements in canonical sorted, duplicate-free order. Imported
formula payloads must bind the same fingerprint as their imported statement
requirement.

`diagnostic_inputs` are optional producer-side details for explainability. They
are excluded from the canonical kernel evidence bytes, hash inputs, and proof
reuse identity unless a later spec explicitly promotes a stable field.
Snapshot-local `VcId`, generated formula ids, context-entry ids, source ranges,
and handoff row ids may appear in diagnostics, but canonical evidence must bind
through stable formula fingerprints, target identifiers, source bindings, and
provenance records.

The task-25 target VC fingerprint is specific to the kernel handoff and excludes
`ProofHint` data. Proof hints, premise restrictions, solver preferences, and
diagnostic replay data can guide candidate production or explanations, but they
do not block target binding and do not enter canonical evidence hash input.

## Input Mapping

The builder input is a validated `VcSet`, a selected `VcIr`, and optional
producer-owned records already computed by prior VC phases:

| VC input | Kernel evidence mapping |
|---|---|
| `VcSet` schema, module, source, canonical VC fingerprint, and selected `VcIr` | `target_vc`, target provenance binding, and deterministic package identity. If a stable target binding cannot be computed, the builder fails closed. |
| `LocalContext` entries with formula refs | Formula evidence entries with local-hypothesis source bindings. Entries without stable formula payloads or provenance are recorded as missing payloads, not fabricated. |
| `PremiseRef::LocalContext` and `PremiseRef::GeneratedFact` | References to the corresponding local-hypothesis or generated-VC-fact formula evidence entries. |
| `PremiseRef::ImportedFact` | Candidate imported axiom/theorem formula entries only when package/module/exported item identity, statement fingerprint, required proof-status requirement, and matching `FormulaEvidenceContext` input are available. `mizar-vc` does not certify the imported fact as accepted; proof/kernel-owned context must do that. Otherwise the premise is an `external_dependency_gap` or fail-closed builder error. |
| `PremiseRef::CheckerFact`, `TypePredicate`, trace, registration, cluster, reduction, definition, policy, and conservative-unknown variants | Mapped only when an explicit formula payload, allowed source class, target binding, and provenance are already available. Marker-only or trace-only records do not become trusted evidence. |
| `VcGeneratedFormula` table | Generated VC fact entries when the formula tree can be projected into the kernel-supported formula grammar and provenance binds the selected target. |
| `VcIr.goal` | The standalone `final_goal` record. It is never a premise and never a source of `used_axioms`. |
| `ProofHint` and premise restrictions | Diagnostic or candidate-production metadata only. They do not select premises, add premises, drop premises, or authorize acceptance. The builder may reference only exact premise refs already materialized in immutable `VcIr` inputs; restrictions that are not already reflected in those inputs stay diagnostic. |
| `DischargeEvidenceRecord` | Task 25 carries replayable input references as diagnostics outside canonical evidence and canonical hash input. A discharge rule name or evidence hash is not trusted acceptance material. Promoting deterministic discharge data into canonical formula/substitution/provenance evidence requires a later spec-backed task. |
| `DependencySlice` and proof-reuse candidate data | Identity and invalidation inputs for task 26. They do not prove the VC and do not replace kernel checking. |

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
Formula and imported-statement fingerprints must use the kernel formula
fingerprint algorithm for this handoff version; another algorithm id is a
fail-closed builder error, not a cue to reinterpret bytes.

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
- `deferred` `VC-HANDOFF-G005`: task 26 owns dependency-slice and proof-reuse
  identity updates that include the kernel evidence hash.

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

Task 26 must add invalidation tests showing that proof-reuse identity changes
when the canonical kernel evidence hash changes and remains unavailable when
downstream proof/cache/artifact schemas are absent.

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
| `KernelGoalPolarity` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `KernelEvidenceHandoffError` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `KernelEvidenceRole` | `#[non_exhaustive]` downstream forward-compatible surface. |

No exhaustive public enum exceptions are owned by this module. Internal
`mizar-vc` matches that intentionally enumerate current variants may remain
exhaustive.

## Task 26 Handoff

Recommended reasoning: `xhigh`.

Prompt:

```text
Continue mizar-vc autonomous correction from completed task 25. Before editing,
verify a clean worktree, confirm the task 25 commit in git log, and re-read
doc/design/mizar-vc/en/kernel_evidence_handoff.md,
doc/design/mizar-vc/en/dependency_slice.md,
doc/design/architecture/en/22.incremental_verification_contract.md,
doc/design/architecture/en/18.dependency_fingerprint.md,
crates/mizar-vc/src/kernel_evidence_handoff.rs, and
crates/mizar-vc/src/dependency_slice.rs. Implement task 26 only: extend
dependency-slice and proof-reuse identity to include the canonical kernel
evidence hash produced by the task-25 builder. Keep downstream proof/cache/
artifact schemas external; do not promote a handoff package to proof acceptance
or add placeholder consumers. Add focused Rust tests for hash-driven
invalidation and fail-closed unavailable reuse when kernel evidence or
downstream consumers are absent. Run cargo fmt --check, cargo test -p mizar-vc,
cargo clippy -p mizar-vc --all-targets --all-features -- -D warnings,
git diff --check, and git diff --cached --check after explicit path staging.
Use review-only agents for the required AGENTS.md review phases.
```

Rationale: task 26 updates architecture-22 reuse identity at the kernel
evidence boundary. Keep `xhigh` because the hash can invalidate cached/reused
proof candidates but still must not become acceptance material. Lower reasoning
is appropriate only for typo-only documentation synchronization.
