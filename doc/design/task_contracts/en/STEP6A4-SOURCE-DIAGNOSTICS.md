# Task STEP6A4-SOURCE-DIAGNOSTICS: source-loading diagnostics

Canonical language: English; [Japanese pointer](../ja/STEP6A4-SOURCE-DIAGNOSTICS.md).
Status: implemented. Tier: full. Owner: [diagnostics plan](../../mizar-diagnostics/en/00.crate_plan.md).
Consumer: real driver SourceLoad service in Step 6. Dependencies: STEP6A1–A3; user-authorized source diagnostic specification.

## Scope and authority

[Spec 22.1.1–2 and 22.7](../../../spec/en/22.error_handling_and_diagnostics.md) allocates source-loading codes and non-range locations. This task specifies and
implements that approved public diagnostic change; source acceptance is unchanged.
Classified gap: shared drafts require a SourceId-backed primary span, while frontend source-load failures precede source allocation and forbid fake spans.

- Register E0600–E0603 in the source-loading family. E0600 reasons use the existing stable detail key; real producer code selection follows separately.
- Require an existing `SourceInput` package/path identity; pre-request discovery failures are deferred. Do not fabricate identities or use absolute origin paths.
- Introduce a primary-location sum: existing validated span or source request
  `(PackageId, NormalizedPath)` without SourceId, offsets, or fake range.
- Source-load records require the source-loading phase/category and non-range
  location; other codes retain existing primary-span validation.
- Preserve sink transport, snapshot suppression, deduplication, deterministic aggregation, and rendering with the new location through their real owners.
- Mechanically migrate existing draft constructors without changing outcomes.
- Do not implement producer adapters, frontend code remapping, artifact task 17,
  LSP conversion, Step 7, MVM, source acceptance, or existing `.miz` expectations.
  Frontend mapping and real source producer integration follow this prerequisite.

## Owner contracts and affected artifacts

[Registry](../../mizar-diagnostics/en/registry.md),
[records](../../mizar-diagnostics/en/failure_record.md),
[aggregation](../../mizar-diagnostics/en/aggregator.md), and
[rendering](../../mizar-diagnostics/en/render.md) own APIs and invariants.
Modify their paired docs and diagnostics source/tests; architecture failure
taxonomy adds source-loading provenance/category. Architecture/internal diagnostic models reference the location owner.
Coverage audit remains partial with real source producer adoption outstanding.

## Tests and exit

Rust boundary tests cover all allocated codes, location/code/phase/category mismatches, distinct reason keys, primary/secondary validation, package/path identity, duplicate and
reversed batch ordering, stale snapshots, and CLI rendering without source lookup.
Existing span diagnostics must retain output and ordering. `.miz` cannot encode
unreadable files or invalid UTF-8; source-loading integration tests follow with
its real producer. No language corpus activation is claimed here.

Independent specification, test-sufficiency, implementation, volume/scope, and source/document consistency reviews are required. Run targeted diagnostics and
driver tests, `cargo fmt --check`, all-target/all-feature clippy with `-D warnings`, and `cargo test`. Exit: validated non-range records can traverse the real shared
pipeline and all preexisting diagnostic tests retain their intended behavior.
