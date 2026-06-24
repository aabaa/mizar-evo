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
| DIS-G002 | `source_drift` / `test_gap` | `src/discharge.rs`, discharge evidence data, and discharge tests do not exist yet. | Tasks 11-12 implement the source and tests against this spec. |
| DIS-G003 | `external_dependency_gap` | `mizar-atp`, `mizar-kernel`, `mizar-proof`, `mizar-cache`, and active corpus-runner consumers are not wired to `mizar-vc`. | This spec records only prover-independent statuses, untrusted evidence, and deferred downstream integration points. |
| DIS-G004 | `external_dependency_gap` | Some type, cluster, registration, reduction, and computation traces may not yet be available as explicit upstream payloads for every VC. | Discharge uses only explicit facts, premise refs, proof hints, and policy inputs already present in `VcIr`; absent traces leave the VC `NeedsAtp` or deferred with an explanation, never silently discharged. |
| DIS-G005 | `deferred` | Concrete numeric computation limits, evidence serialization, dependency-slice fingerprints, corpus fixtures, and kernel/proof/cache validation are owned by later tasks or crates. | Task 10 defines required shapes and invariants; tasks 11-12 and later dependency/consumer tasks fill implementation details. |

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
  premise references;
- definitional reductions allowed by the VC's unfold requests and policy
  inputs;
- bounded `by computation` or verification-time computation when the requested
  computation and its limit policy are explicit.

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

Task 10 fixes this shape but does not choose numeric defaults. Task 11 records
the concrete default and per-policy limit interpretation. Exceeding a limit is
not a failed proof: the VC remains `NeedsAtp` or deferred with a stable
explanation and no `Discharged` evidence.

## Evidence And Explanations

Discharge evidence is untrusted production evidence. It may justify a
pre-ATP status, diagnostics, and later proof/cache decisions, but it is not a
kernel-accepted proof.

Each discharged VC must record:

- the discharged `VcId`;
- the deterministic rule name and version;
- the input formula refs, local context refs, premise refs, and generated
  formula refs used by the rule;
- the policy keys, unfold requests, computation hints, and limit tuple when
  relevant;
- replay data or premise refs for every cluster, registration, reduction, or
  computation trace used by the rule;
- a stable evidence hash suitable for dependency-slice and artifact records.

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
