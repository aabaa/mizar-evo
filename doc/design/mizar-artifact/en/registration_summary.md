# RegistrationSummary

> Canonical language: English. Japanese companion: [../ja/registration_summary.md](../ja/registration_summary.md).

## Purpose

`RegistrationSummary` is the checker-facing projection of activated
registrations and their replay trace references. It supports cross-module reuse
without loading checker-internal registration databases.

## Schema

The schema version is `mizar-artifact/registration-summary/v1`.

Fields:

- `schema_version`;
- `package_id`;
- `module_path`;
- `registration_hash`;
- `registrations`;
- `cluster_trace_refs`;
- `local_metadata`.

Only registrations whose coherence, existence, or reducibility obligations have
been accepted by the configured verification pipeline may appear as active
registrations. Pending, rejected, or open registrations may be diagnostic
metadata elsewhere but are not importer facts.

## Registration Entries

Each registration entry records:

- stable registration id and label;
- registration kind;
- trigger key used by checker lookup;
- canonical source pattern;
- generated fact or reduction result summary;
- accepted proof status summary;
- optional witness reference id.

The entry is a projection. It must not include raw `TypedAst`,
`ResolutionTrace`, checker indexes, or implicit trace steps.

## Trace References

Cluster and reduction traces stay in separate artifacts. A trace reference
records:

- portable path;
- content hash;
- trace kind;
- trace schema version.

When trace bytes are available, readers validate the content hash before using
the reference.

## Ordering And Hashes

Registration entries are sorted by kind, trigger key, registration id, and
label. Trace references are sorted by path and hash.

`registration_hash` covers the active registration entries and trace reference
identity. It excludes `local_metadata`.

## Reader Rules

Readers reject unsupported versions, hash mismatches, nondeterministic order,
missing trace hashes, and forbidden raw checker payloads.
