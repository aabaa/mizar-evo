# ModuleSummary Reuse

> Canonical language: English. Japanese companion:
> [../ja/module_summary_reuse.md](../ja/module_summary_reuse.md).

## Purpose

Task R-024 consumes dependency-facing `ModuleSummary` artifacts published by
`mizar-artifact` so resolver import/name/lexical consumers can use exported
dependency projections without re-reading dependency source files.

This document refines the resolver side of architecture 03 "Module Summary" and
architecture 18 dependency fingerprints. The canonical artifact schema, writer,
reader, hash framing, and compatibility policy remain owned by
`mizar-artifact`.

## Scope

`mizar-resolve` owns:

- requesting a dependency summary by canonical `mizar-build::module_index::ModuleId`;
- validating a supplied canonical summary through `mizar-artifact` reader APIs;
- converting validated exported symbol, label, lexical, re-export, and
  dependency-interface projections into resolver-owned `SymbolEnv` summary
  contribution indexes;
- reporting deterministic crate-local fallback records when a summary is absent,
  incompatible, mismatched, or otherwise invalid;
- comparing source-backed and summary-backed exported projections in tests.

`mizar-resolve` does not own:

- artifact JSON syntax, schema version policy, reader/writer implementation, or
  hash framing;
- store or manifest I/O;
- source loading for dependency-summary-backed modules;
- proof acceptance, type checking, overload winner selection, cache reuse, or
  semver classification;
- public resolver diagnostic codes.

## Input Contract

The resolver receives two independent inputs:

1. the build-owned module index provider from
   `mizar_resolve::module_index::ModuleIndexInput`;
2. a canonical `mizar_artifact::module_summary::ModuleSummary` value or
   canonical JSON value read at the artifact boundary.

The resolver may ask the provider for a `DependencyModuleSummaryRef` to
determine whether a module is summary-backed and to retain the artifact path in
fallback diagnostics. The resolver must not interpret
`DependencyModuleSummaryRef::content_hash` as an interface hash unless a later
build/artifact contract explicitly says so. The optional expected interface hash
is a validation input to the artifact-owned reader. The module-index provider
does not expose the full artifact `ModuleSummaryIdentity` lockfile field, so the
resolver validates only the known identity fields after reading: package id,
package version, module path, and language edition. A summary with a
`lockfile_identity` is accepted when those known fields match.

## Summary-Backed Projection

A validated summary contributes one `ContributionKind::Summary` record and one
`ModuleSummaryIndex` entry keyed by the canonical dependency module.

Exported symbols are lowered to resolver-owned `SymbolEntry` records:

- `SymbolId.module` is the canonical dependency `ModuleId`;
- `SymbolId.local` is derived from the stable exported origin id;
- `SymbolId.fqn` is the summary fully qualified name;
- known public serialized visibility becomes public/exported, known private
  serialized visibility remains private/local-only, and unknown visibility
  values produce deterministic fallback records instead of widening access;
- declaration kind, rendered signature, and interface fingerprint remain opaque
  strings in resolver signature shells.

Exported labels are lowered to `LabelIndex` entries only when visibility is a
known public or private value. Known public labels receive public export status,
known private labels remain local-only, and unknown visibility values produce
  deterministic fallback records. Unknown serialized label target kinds also
  produce fallback records because the resolver must not fabricate proof or
  label semantics for artifact-only strings.

Lexical contributions are lowered into `ModuleLexicalSummaryIndex` entries only
when they can be paired with an exported summary symbol. Unpaired lexical
contributions remain represented by deterministic fallback records so the
resolver does not fabricate symbol identities.

Re-exports and dependency-interface references are stored as declaration
dependency edges with generated anchors. They are dependency-facing facts only;
they do not validate export legality, proof status, or cache reuse.

## Fallback And Diagnostics

Summary reuse is fail-closed. The resolver falls back to source-backed
resolution only when the caller supplies a source-backed resolver input for the
same canonical module. For dependency-summary-backed modules that have no
workspace or in-memory source representation, the resolver records an
unavailable/incompatible summary result and leaves source loading to the
build/session/driver owners.

Fallback is required when:

- the provider has no dependency summary for the module;
- the artifact reader rejects schema version, canonical order, hash, or shape;
- the resolver-known module identity fields do not match;
- the caller-supplied expected interface hash does not match;
- the summary cannot be mapped into resolver-owned projections without
  fabricating required identity.

Fallback records are crate-local/internal and deterministic. They include the
canonical module, artifact path when available, fallback reason, and stable
detail text. They do not allocate public diagnostic codes.

## Source-Backed Agreement

For a shared fixture, summary-backed and source-backed resolution agree when the
exported resolver-facing facts match:

- exported symbol fully qualified names, kinds, visibility/export status, and
  opaque rendered signatures;
- exported label names, owner paths, visibility/export status, and target kind
  payloads;
- exported lexical contribution keys that are paired with exported symbols;
- dependency module identity and interface hash references.

The comparison intentionally ignores source ranges, source ids, local private
symbols, proof bodies, algorithm bodies, diagnostic wording, and internal
artifact bytes.

## Determinism

Summary-backed projection order follows the canonical ordering already enforced
by `mizar-artifact`. Resolver indexes still sort by their own existing stable
keys, so repeated consumption of byte-equivalent summaries must produce
byte-identical `SymbolEnv` snapshots and fallback records.

## Public Enum Forward-Compatibility

Task R-024 applies the resolver public-enum decision procedure to this module.
`ModuleSummaryReuseReason` is a public crate-local/internal diagnostic surface
and must remain `#[non_exhaustive]`. Downstream consumers must include wildcard
or fallback handling for future summary-unavailable, artifact-rejected, or
unsupported-projection classes.

- `ModuleSummaryReuseReason`
