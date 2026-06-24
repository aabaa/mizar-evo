# Module: discharge

> Canonical language: English. Japanese companion:
> [../ja/discharge.md](../ja/discharge.md).

## Purpose

This module specifies deterministic pre-ATP discharge for `mizar-vc` phase 12.
It consumes canonical `VcSet` data after generation, normalization, and status
policy projection, and it classifies each concrete VC as discharged by a
deterministic Mizar-side rule, left in an explicit policy status, or forwarded
unchanged as `NeedsAtp`.

Task 10 is specification-only. Rust discharge code is implemented by tasks 11
and 12. This document refines architecture 07 and 08; it does not change
language semantics, `.miz` fixtures, expectations, `doc/spec`, or traceability
metadata.

## Responsibility

Owned by this module:

- deterministic pre-ATP discharge rule selection;
- replayable, untrusted discharge evidence records;
- human-readable explanations for discharged and not-discharged VCs;
- computation-limit and definition-unfolding policy boundaries;
- preservation of ATP-bound VCs with full context and `NeedsAtp` status.

Out of scope:

- ATP translation, ATP portfolio scheduling, or backend configuration;
- kernel proof acceptance, certificate validation, or proof artifact
  publication;
- proof/cache reuse decisions and dependency-slice fingerprints;
- source syntax reconstruction, new VC generation, or new algorithm payload
  families;
- accepting registrations, clusters, reductions, or computation results when
  the explicit upstream trace data is absent.

## Gap Classification For This Spec

| ID | Class | Evidence | Handling |
|---|---|---|---|
| DIS-G001 | `spec_gap` | `discharge.md` did not exist before task 10, while tasks 11-12 need a phase-12 contract. | Task 10 adds the English/Japanese discharge spec only. |
| DIS-G002 | `source_drift` / `test_gap` | Before task 11, `src/discharge.rs`, `pub mod discharge`, lint-policy coverage, the task-11 discharge API, and focused engine tests did not exist. | Task 11 adds those source/module/test surfaces for explicit classes already represented in `VcIr`, with minimal stable `DischargeEvidenceRef` values. Task 12 expands replayable evidence and explanation serialization. |
| DIS-G003 | `external_dependency_gap` | `mizar-atp`, `mizar-kernel`, `mizar-proof`, `mizar-cache`, and active corpus-runner consumers are not wired to `mizar-vc`. | This spec records only prover-independent statuses, untrusted evidence, and deferred downstream integration points. |
| DIS-G004 | `external_dependency_gap` | Some type, cluster, registration, reduction, and computation traces may not yet be available as explicit upstream payloads for every VC. | Discharge uses only explicit facts, premise refs, proof hints, and policy inputs already present in `VcIr`; absent traces leave the VC `NeedsAtp` or deferred with an explanation, never silently discharged. |
| DIS-G005 | `deferred` | Artifact serialization, dependency-slice fingerprints, corpus fixtures, and kernel/proof/cache validation are owned by later tasks or crates. | Task 10 defines required shapes and invariants; task 11 records the engine default limit; task 12 adds in-memory replayable evidence/explanation records on `DischargeOutput` and leaves artifact/dependency/downstream consumer work deferred. |

## Inputs And Outputs

Required input:

- a validated `VcSet` whose concrete VCs have stable `VcId`s, seed accounting,
  local contexts, premises, goals, proof hints, anchors, and statuses;
- explicit local type, sethood, non-emptiness, cluster, registration, reduction,
  and checker facts already present as context entries or premise refs;
- definition unfold requests and computation hints from `ProofHint`;
- verifier policy inputs that control discharge, computation limits, and ATP
  dispatch.

Required output:

- the same VC ordering and seed accounting as the input `VcSet`;
- `Discharged` statuses only when a deterministic rule produced replayable
  evidence;
- `NeedsAtp` statuses for VCs requiring external search;
- preserved `PolicyOpen`, `AssumedByPolicy`, skipped, deferred, and error
  statuses;
- deterministic explanations for discharged VCs and for VCs that remain
  `NeedsAtp` or deferred because a rule, trace, or limit was unavailable.

Discharge must never remove a VC, weaken its goal, drop local context, rewrite
seed accounting, reorder VCs, or replace an ATP-bound obligation with a missing
record.

## Supported Discharge Classes

Tasks 11-12 may discharge only when all required facts and traces are explicit:

- syntactic tautology and contradiction checks over the canonical goal and
  local premises;
- reflexivity, direct equality normalization, and alpha-stable formula
  identity checks;
- type, sethood, and non-emptiness facts already present in the local context;
- cluster, registration, and reduction facts with explicit replayable trace or
  premise references and a goal-linked explicit generated or local fact;
- definitional reductions allowed by the VC's unfold requests and policy
  inputs, with a goal-linked explicit generated or local fact after the
  reduction boundary;
- bounded `by computation` or verification-time computation when the requested
  computation, its limit policy, and an explicit goal-linked computation result
  fact are present.

Unsupported or unavailable classes must not produce negative evidence. They
leave the VC in `NeedsAtp`, `PolicyOpen`, `AssumedByPolicy`, `DeferredExternal`,
or `Error` according to the existing status and the reason discovered.

## Limit Model

Every computation-based discharge is governed by a deterministic limit tuple:

- a policy key naming the limit source;
- a gas or step budget;
- an optional wall-clock-independent fuel class;
- a timeout label used only as a stable policy identifier, never as a source of
  nondeterministic timing;
- the definition-unfolding and reduction policy active for the VC.

Task 10 fixes this shape but does not choose numeric defaults. Task 11 sets the
engine default policy key to `task-11-computation-step-limit` with
`max_steps = 64`; callers may provide a different deterministic
`DischargePolicy`. A `LimitPolicy` computation hint must match the active policy
key, while `ByComputation` uses the active policy directly. Exceeding a limit is
not a failed proof: the VC remains `NeedsAtp` or deferred with a stable
explanation and no `Discharged` evidence.

## Evidence And Explanations

Discharge evidence is untrusted production evidence. It may justify a
pre-ATP status, diagnostics, and later proof/cache decisions, but it is not a
kernel-accepted proof.

Task 11 stores only the minimal status evidence that already exists in `VcIr`:
the deterministic rule name/version and a stable evidence hash in
`DischargeEvidenceRef`. The selected rule may rely only on preserved `VcIr`
inputs: local context entries, premise refs, proof-hint citations, unfold
requests, computation hints, policy inputs, generated formula refs, and the
unchanged goal. A trace, unfold, or computation marker alone is not discharge
evidence; it must be tied to an explicit generated or local fact for the same
goal.

Task 12 expands this into an in-memory replayable evidence record exposed by
`DischargeOutput`. It must be deterministic to render and clone, and it must be
paired with the stable not-discharged explanations produced by the same pass.
The record must include:

- the discharged `VcId`;
- the deterministic rule name and version;
- the input formula refs, local context refs, premise refs, and generated
  formula refs used by the rule;
- the policy keys, unfold requests, computation hints, and limit tuple when
  relevant;
- replay data or premise refs for every cluster, registration, reduction, or
  computation trace used by the rule;
- a stable evidence hash suitable for dependency-slice and artifact records.

Every `Discharged` VC in a `DischargeOutput` must have an evidence record whose
status evidence matches the VC status. VCs left `NeedsAtp`, `PolicyOpen`,
`AssumedByPolicy`, skipped, deferred, or error must have explanations but must
not be treated as discharged evidence. Artifact serialization, persistence, and
kernel-side replay validation remain outside task 12.

If the input `VcSet` already contains `VcStatus::Discharged`, task 12 preserves
that status and records a preserved-evidence record rather than fabricating
missing replay data. The preserved record must identify the VC, copy the
existing rule name and evidence hash, mark the evidence source as pre-existing
input status, and explain that detailed replay data was not reconstructed by
this pass. Such records are valid for status preservation and diagnostics only;
they are not newly produced proof evidence.

Task 20 makes newly produced deterministic evidence hashes cross-edit stable.
The hash input is the deterministic rule plus the canonical VC and local-context
fingerprints, not the snapshot-local `VcId`. Preserved/pre-existing discharged
status records do not synthesize proof evidence and are not eligible for
cross-edit proof reuse keys. If either canonical fingerprint is unavailable, the
evidence hash marker must be conservative unknown and the VC is not eligible for
the Task 20 proof-reuse candidate key.

Unavailable trace markers belong only to not-discharged explanations or to trace
classes that the selected rule did not use.

Each not-discharged VC must record an explanation category such as
`needs_atp`, `policy_open`, `assumed_by_policy`, `missing_trace`,
`limit_exceeded`, `unsupported_rule`, `deferred_external`, or `error`. These
categories are diagnostic data only; they must not erase the VC.

## Status Interaction

Discharge may change `Open` or `NeedsAtp` VCs to `Discharged` only when evidence
exists. It may leave them as `NeedsAtp` when no deterministic rule applies.
Policy statuses remain explicit:

- `PolicyOpen` is not discharge evidence and is not sent to ATP;
- `AssumedByPolicy` is an accepted assumption marker, not proof evidence;
- skipped, deferred, and error statuses remain visible and are not dispatched;
- `Discharged` evidence is never kernel proof acceptance.

Only canonical VCs with `NeedsAtp` status are eligible for ATP translation.
They must retain source refs, local context, premises, proof hints, anchors,
seed accounting, and the original goal.

## Determinism And Ordering

Discharge order is ascending `VcId`. Rule selection must not depend on hash-map
iteration, worker completion order, local absolute paths, backend availability,
wall-clock time, or nondeterministic resource measurements.

Discharged, policy-status, deferred/error, and `NeedsAtp` output lists preserve
input VC order. Diagnostics with the same source range are ordered by `VcId`,
then stable rule name, then stable diagnostic category.

## Planned Tests

Task 11 must add Rust coverage for:

- stable discharge of tautology, contradiction, reflexivity, and direct
  equality-normalization fixtures;
- explicit type/sethood/non-emptiness fact discharge without global inference;
- explicit cluster, registration, and reduction trace discharge when replayable
  trace or premise refs are present;
- definitional reduction discharge only when allowed by unfold policy;
- bounded computation discharge when computation input and limit policy are
  explicit and the limit is not exceeded;
- limit-exceeded computation remaining `NeedsAtp` or deferred with a stable
  explanation;
- no-erase ATP boundary: unsupported rules preserve full VC context and
  `NeedsAtp` status;
- deterministic output order across repeated runs.

Task 12 must add Rust coverage for:

- evidence records for every discharged VC;
- evidence hashes and explanation categories that render deterministically;
- policy statuses that are not treated as discharge evidence;
- missing trace data that fails closed instead of discharging;
- preservation of seed accounting, anchors, proof hints, and generated formula
  refs across discharge.

## Public Enum Policy

Task 17 classifies every `discharge` public enum as a downstream
forward-compatible API surface. Each enum must keep `#[non_exhaustive]` so later
evidence sources, replay modes, explanation categories, and deterministic
discharge rules can be added without breaking downstream exhaustive matches.

| public enum | decision |
|---|---|
| `DischargeEvidenceSource` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `DischargeEvidenceReplay` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `DischargeExplanationCategory` | `#[non_exhaustive]` downstream forward-compatible surface. |
| `DischargeRule` | `#[non_exhaustive]` downstream forward-compatible surface. |

No exhaustive public enum exceptions are owned by this module. Internal
`mizar-vc` matches that intentionally enumerate current variants may remain
exhaustive.
