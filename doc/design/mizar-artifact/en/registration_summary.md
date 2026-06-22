# mizar-artifact RegistrationSummary Schema

> Canonical language: English. Japanese companion:
> [../ja/registration_summary.md](../ja/registration_summary.md).

## Purpose

`RegistrationSummary` is the dependency-facing artifact projection for
activated registrations. It lets downstream checker phases load exported
existential, conditional, functorial, and reduction registration contributions
without loading checker-internal indexes, raw `TypedAst`, or internal
`cluster-db` records.

This document refines architecture 04 "Type and Registration Resolution" and
architecture 17 "Cluster and Registration Trace Format". It depends on the
canonical store rules in [store.md](./store.md).

## Scope

The `registration_summary` schema owns:

- stable module identity and schema version fields for registration summaries;
- activated, exported registration contributions visible to importers;
- references to published `ResolutionTrace` artifacts by hash;
- the dependency-facing `registration_interface_hash` used by checker reuse and
  dependency fingerprints;
- canonical ordering rules for registration contributions and trace references;
- compatibility and reader validation requirements for summary artifacts.

The schema does not own:

- type checking, cluster search, reduction search, or checker fixed-point
  algorithms;
- proof acceptance, kernel replay, ATP dispatch, or proof witness payloads;
- the concrete `ResolutionTrace` body schema from architecture 17;
- internal `RegistrationIndex`, `TypeFactTable`, `TypedAst`, `ResolutionTrace`,
  `cluster-db`, cache records, or cache-key lookup;
- manifest transactions or artifact-store I/O beyond using the store canonical
  rules.

## Top-Level Shape

The schema family is `mizar-artifact/registration-summary`. Version `1.0` is
the initial version for task 7.

Conceptual shape:

```rust
struct RegistrationSummary {
    schema_version: String,
    module: ModuleSummaryIdentity,
    source_hash: Hash,
    registration_interface_hash: Hash,
    activated_registrations: Vec<ActivatedRegistrationSummary>,
    trace_artifacts: Vec<RegistrationTraceArtifactRef>,
    dependency_registrations: Vec<DependencyRegistrationRef>,
}
```

Task 6 specifies this shape only. Task 7 serializes it as canonical JSON and
adds the validating reader/writer.

`source_hash` records the source text used to produce the summary so readers can
diagnose stale artifacts. It is not part of `registration_interface_hash`.
Comment-only, proof-body-only, diagnostic-only, and local source-range changes
must not invalidate importers when activated exported registrations are
unchanged.

The `module` field uses the same identity shape defined by
[module_summary.md](./module_summary.md). Normalized source paths and local
filesystem aliases are source metadata, not registration identity.

## Activated Registrations

`activated_registrations` contains only registrations whose well-formedness and
correctness obligations have been accepted by the configured verifier policy.
Pending, rejected, local-only, private, or unverified registrations are excluded
from this summary and must not contribute automatic type facts downstream.

Each activated registration records:

- stable registration origin id;
- source label or stable generated label;
- registration kind: `existential`, `conditional`, `functorial`, or
  `reduction`;
- exported visibility and namespace/module provenance;
- canonical trigger key used by downstream checker indexes;
- normalized pattern summary, including referenced type head, attribute,
  functor, term head, parameters, and guard fingerprints as applicable;
- generated contribution summary, such as produced existence fact, attribute
  fact, functorial result facts, or reduction `source -> target` fingerprint;
- accepted proof status and verifier-policy fingerprint that made the
  registration visible;
- trace references that explain cluster expansion or reduction strategy when
  such traces are required for replay or diagnostics;
- optional diagnostic/navigation source range metadata.

`RegistrationSummary` records projected accepted status but does not decide that
a proof is accepted. Proof-producing phases and kernel acceptance remain outside
this crate.

Task 7 uses this canonical JSON field shape:

```text
activated_registration = {
  "origin_id": string,
  "label": string | null,
  "registration_kind": "existential" | "conditional" | "functorial" | "reduction",
  "visibility": string,
  "namespace_path": [string, ...],
  "source_module": module,
  "trigger_key": string,
  "normalized_pattern": registration_pattern,
  "generated_contribution": registration_contribution,
  "accepted_status": "accepted",
  "verifier_policy_fingerprint": interface_hash_string,
  "trace_ids": [string, ...],
  "source_range": source_range | null
}

registration_pattern = {
  "fingerprint": interface_hash_string,
  "type_head": string | null,
  "attribute": string | null,
  "functor": string | null,
  "term_head": string | null,
  "parameters": [string, ...],
  "guards": [interface_hash_string, ...]
}

registration_contribution = {
  "kind": "existence_fact" | "attribute_fact" | "functorial_result" | "reduction_rule",
  "summary": string,
  "fingerprint": interface_hash_string
}

source_range = {
  "start_byte": non_negative_integer,
  "end_byte": non_negative_integer
}
```

Optional fields are present with JSON `null` when absent. Readers reject empty
strings in identity, label, kind, trigger, summary, parameter, and trace-id
fields; unknown enum values; ranges whose start is greater than end; duplicate
`trace_ids`; and any `accepted_status` other than `accepted`. Pending or
unaccepted registrations are represented by absence from this summary, not by
alternate status values.

## Trace Artifact References

`trace_artifacts` contains hash-addressed references to published
`ResolutionTrace` artifacts. The reference is stable enough for artifact
validators and diagnostics to locate the trace without embedding trace bodies in
the registration summary.

Conceptual shape:

```text
trace_artifact = {
  "trace_id": string,
  "trace_kind": "cluster" | "reduction",
  "artifact_path": string,
  "artifact_hash": artifact_hash_string,
  "trace_replay_hash": interface_hash_string,
  "diagnostic_hash": diagnostic_hash_string | null,
  "used_by_registration_origin_ids": [string, ...]
}
```

The trace body remains owned by the trace artifact schema from architecture 17.
This summary owns only the stable reference. Missing trace data is never
interpreted as "no trace"; if a registration requires a trace for replay or
diagnostics, the reference must be present and hash-validated by the reader.
Activation is not a `ResolutionTrace` kind in architecture 17. Registration
activation is projected through `accepted_status` and
`verifier_policy_fingerprint`; any future activation proof witness reference is
owned by the proof-witness schema, not by `ResolutionTrace`.

`trace_replay_hash` is the semantic hash that participates in registration
compatibility. It identifies the replay-relevant cluster or reduction trace
projection under the trace schema's own domain. `artifact_hash` validates the
published trace file bytes, and `diagnostic_hash` validates optional diagnostic
payloads; those byte/diagnostic hashes are not part of
`registration_interface_hash` unless their replay hash changes.

## Dependency Registration References

`dependency_registrations` records registration summaries from dependencies that
affected this module's activated registration projection. Missing dependency
data is never interpreted as "no dependency"; incomplete dependency
registration information makes the summary uncacheable for reuse decisions
owned by `mizar-cache`.

Conceptual shape:

```text
dependency_registration = {
  "module": module,
  "registration_interface_hash": interface_hash_string
}
```

## Registration Interface Hash

`registration_interface_hash` is the canonical dependency-facing key for the
importer-visible registration projection. It is not the byte identity of the
summary file. The manifest path and store-level `artifact_hash` identify and
validate the published file bytes.

The hash is computed with task-3 `HashClass::Interface`, schema family
`mizar-artifact/registration-summary`, and the current schema version.

The hash includes:

- schema family and schema version;
- module identity fields that affect importer interpretation;
- language edition;
- every activated exported registration contribution;
- registration kind, trigger key, normalized pattern, generated contribution,
  accepted proof status, verifier-policy fingerprint, and export visibility;
- required trace artifact references by trace id, kind, and semantic
  `trace_replay_hash`;
- dependency registration references and their registration interface hashes.

The hash excludes:

- `source_hash` of the whole file;
- diagnostic and navigation source ranges;
- proof bodies, proof witness payloads, and trace artifact bodies;
- trace artifact paths, trace artifact byte hashes, and diagnostic trace hashes;
- local diagnostics and diagnostic wording;
- timestamps, absolute local paths, worker ids, backend timings, and other
  hash-excluded local metadata.

Two summaries with byte-different source hashes or source ranges but identical
activated registration projections have the same `registration_interface_hash`,
while their manifest entries or store-level `artifact_hash` values may differ.

## Canonical Ordering

All collections are serialized deterministically:

- activated registrations sort by registration kind, trigger key, origin id,
  label, normalized pattern fingerprint, generated contribution fingerprint,
  and accepted proof status; readers reject duplicate origin ids;
- trace references sort by trace kind, trace id, artifact path, artifact hash,
  and `trace_replay_hash`; optional `diagnostic_hash` is not an interface-order
  tie-breaker; readers reject duplicate trace ids;
- `used_by_registration_origin_ids` sort by origin id and reject duplicates;
- dependency registration references sort by full module identity and
  registration interface hash; readers reject duplicate module identities.

No insertion order, source traversal order, filesystem order, cache insertion
order, or worker completion order may affect serialized bytes or hashes.

## Reader And Writer Requirements

Task 7 writers use the canonical UTF-8 JSON rules from `store.md` and emit the
current schema version. Task 7 readers operate over `CanonicalJson` values
produced at the store boundary; byte-level artifact parsing and duplicate-key
detection for files remain part of the later artifact-store I/O task. Readers:

- check schema-version compatibility before interpreting summary fields;
- verify that the manifest entry, requested module, and summary module identity
  agree;
- verify `registration_interface_hash` when requested by the consuming command
  or manifest entry;
- verify trace references by hash when the referenced trace artifact is supplied
  by the caller;
- reject unaccepted, pending, private, raw-checker-shaped, raw-trace-shaped, or
  cache-record-shaped payloads;
- reject summaries that claim accepted proof status without a stable projected
  status field and verifier-policy fingerprint defined by this schema or a
  later compatible schema.

Reader failures are artifact diagnostics. They do not establish proof authority,
do not rerun registration search, and do not silently fall back to internal
cache records.

## Deferred Implementation

Task 6 adds this specification only. Source implementation is deferred to task
7, which adds the `RegistrationSummary` schema, writer, validating reader, and
tests for round-trips, trace references resolving by hash, deterministic
ordering, incompatible-version reads, and registration/hash mismatch rejection.

Checker producer integration, concrete `ResolutionTrace` artifact production,
proof acceptance, and manifest/file I/O remain external or later-task work and
must not be stubbed in task 6.
