# mizar-artifact ModuleSummary Schema

> Canonical language: English. Japanese companion:
> [../ja/module_summary.md](../ja/module_summary.md).

## Purpose

`ModuleSummary` is the dependency-facing artifact projection for resolver and
incremental-build consumers. It lets a downstream module load exported names,
labels, lexical contributions, re-exports, and interface fingerprints without
re-reading the dependency source file or loading compiler-internal IR.

This document refines architecture 03 "Module Summary" and architecture 18
interface fingerprint requirements. It depends on the canonical store rules in
[store.md](./store.md).

## Scope

The `module_summary` schema owns:

- stable module identity and schema version fields for module summaries;
- exported symbol, label, lexical, and re-export projections visible to
  importers;
- the `interface_hash` used as the dependency-facing reuse key and fingerprint
  input;
- canonical ordering rules for all summary collections;
- compatibility and reader validation requirements for summary artifacts.

The schema does not own:

- resolver algorithms, import legality decisions, or name lookup execution;
- type inference, overload selection, proof search, proof acceptance, or kernel
  validation;
- proof bodies, algorithm bodies, expression metadata, diagnostics, raw
  `SymbolEnv`, raw `ResolvedAst`, or cache records;
- manifest transactions or artifact-store I/O beyond using the store canonical
  rules.

## Top-Level Shape

The schema family is `mizar-artifact/module-summary`. Version `1.0` is the
initial version for task 5.

Conceptual shape:

```rust
struct ModuleSummary {
    schema_version: String,
    module: ModuleSummaryIdentity,
    source_hash: Hash,
    interface_hash: Hash,
    exported_symbols: Vec<ExportedSymbolSummary>,
    exported_labels: Vec<ExportedLabelSummary>,
    lexical_summary: ModuleLexicalSummary,
    reexports: Vec<ModuleReexportSummary>,
    dependency_interfaces: Vec<DependencyInterfaceRef>,
}
```

`source_hash` records the exact source file content used to produce the summary
so readers can diagnose stale artifacts. It is not part of `interface_hash`,
because body-only, proof-body-only, diagnostic-only, and comment-only source
changes must not invalidate importers when the exported interface is unchanged.

## Module Identity

`ModuleSummaryIdentity` contains the stable package and module identity visible
to downstream tools:

- package id;
- package version or lockfile identity when available;
- module path;
- language edition.

Normalized source path and `source_hash` are source metadata for stale-artifact
diagnostics and manifest consistency checks. They are not module identity and
are not part of `interface_hash` unless a schema-specific rule makes a source
path visible to importers. Local aliases and filesystem traversal details are
not part of module identity. Readers reject a summary when the manifest entry
or requested import path identifies a different package or module.

## Exported Symbols

`exported_symbols` contains dependency-facing declarations only. Each entry
records:

- stable origin id for pairing the same exported surface element across builds;
- fully-qualified exported name;
- exported namespace path and visibility;
- declaration kind, such as definition, theorem, mode, predicate, functor,
  attribute, struct, registration-facing declaration, notation, or algorithm
  signature;
- source range suitable for diagnostics and navigation;
- rendered surface signature or statement needed by importers;
- canonical interface fingerprint for that exported element;
- proof acceptance status only when importer visibility or interface validity
  depends on that status.

The summary excludes implementation bodies:

- theorem proof bodies;
- algorithm bodies;
- local definitions and private declarations not exported or re-exported;
- expression metadata;
- ATP logs, proof witness payloads, and kernel-internal proof state.

Proof acceptance status is projected data supplied by proof-producing phases.
`ModuleSummary` records the status but does not validate proofs or decide
whether a proof is accepted.

## Exported Labels

`exported_labels` records labels that downstream modules may cite. Each entry
records:

- stable label origin id;
- label text;
- fully-qualified owner item;
- exported visibility;
- source range;
- target kind, such as theorem, definition, scheme, registration, or proof
  obligation label when such labels are exported by a later schema.

Private or module-local proof-step labels are excluded. Ambiguous or duplicate
labels are not normalized away by this schema; resolver diagnostics are owned by
`mizar-resolve`.

## Lexical Summary

`lexical_summary` contains only exported lexical contributions needed to build a
candidate active lexical environment for importers:

- exported notation declarations and parse effects;
- exported reserved/user symbol contributions;
- exported vocabulary or namespace prefixes needed by lexical disambiguation;
- lexical schema version or fingerprint fields that affect token
  classification.

The lexical summary is not proof authority and does not decide whether an import
is legal. Active lexical environment construction may use candidate summaries,
but semantic import resolution validates imports later.

## Re-exports And Dependencies

`reexports` records exported forwarding relationships by stable module identity
and, when item-level re-exports are supported, by exported item identity. It
preserves provenance so importers can report diagnostics against the original
module and stable origin.

`dependency_interfaces` records the dependency `ModuleSummary` interface hashes
that affected this summary. Missing dependency data is never interpreted as "no
dependency"; incomplete dependency interface information makes the summary
uncacheable for reuse decisions owned by `mizar-cache`.

## Interface Hash

`interface_hash` is the canonical dependency-facing key for the importer-visible
projection in a `ModuleSummary`. It is not the byte identity of the summary file.
The manifest path and store-level `artifact_hash` identify and validate the
published file bytes.

`interface_hash` is computed with the task-3 `HashClass::Interface` domain over
the canonical interface projection.

The hash includes:

- schema family and schema version;
- module identity fields that affect importer interpretation;
- language edition;
- exported symbols, labels, lexical contributions, and re-exports;
- exported theorem statement and accepted proof status when visible to
  importers;
- exported algorithm signatures and `requires` / `ensures` contracts;
- dependency-facing notation, overload summaries, and coherent-refinement
  metadata when present in the summary.

The hash excludes:

- `source_hash` of the whole file;
- comments and formatting outside syntax-sensitive notation;
- proof bodies and algorithm bodies;
- local diagnostics and diagnostic wording;
- timestamps, absolute local paths, worker ids, backend timings, and other
  hash-excluded local metadata.

Two summaries with byte-different `source_hash` values but identical exported
interface projection have the same `interface_hash`, while their manifest
entries or store-level `artifact_hash` values may differ.

## Canonical Ordering

All collections are serialized deterministically:

- exported symbols sort by fully-qualified name, stable origin id, and source
  range;
- exported labels sort by label text, owner fully-qualified name, stable origin
  id, and source range;
- lexical contributions sort by contribution kind and canonical lexical key;
- re-exports sort by target module identity and item identity;
- dependency interface references sort by package id, module path, and
  interface hash.

No insertion order, source traversal order, filesystem order, or worker
completion order may affect serialized bytes or hashes.

## Reader And Writer Requirements

Writers use the canonical UTF-8 JSON rules from `store.md` and emit the current
schema version. Readers:

- check schema-version compatibility before interpreting summary fields;
- verify that the manifest entry, requested module, and summary module identity
  agree;
- verify `interface_hash` when requested by the consuming command or manifest
  entry;
- reject raw-IR-shaped payloads and unknown cache-record encodings;
- reject summaries that claim accepted proof status without a stable projected
  status field defined by this schema or a later compatible schema.

Reader failures are artifact diagnostics. They do not establish proof authority
and do not silently fall back to internal cache records.

## Deferred Implementation

Task 4 adds this specification only. Source implementation is deferred to task
5, which adds the `ModuleSummary` schema, writer, validating reader, and tests
for round-trips, interface-hash stability under body-only changes, and
incompatible-version reads.
