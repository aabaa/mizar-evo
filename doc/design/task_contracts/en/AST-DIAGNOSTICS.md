# Task AST-DIAGNOSTICS: Reflection descriptor preparation
Canonical language: English; [Japanese pointer](../ja/AST-DIAGNOSTICS.md).
Status: frozen descriptor scope; producer adoption deferred. Tier: full gates.
Primary owner: [mizar-diagnostics](../../mizar-diagnostics/en/00.crate_plan.md).
Consumers: future parser/checker, VC/proof/MVM, mizar-test and tooling adoption.
Dependencies/readiness: existing registry infrastructure suffices for descriptors;
source-aware owning-stage failure records remain prerequisites for emission.
Authority: [§22.4.8](../../../spec/en/22.error_handling_and_diagnostics.md#2248-reflection-and-computation-diagnostics)
and [§22.7](../../../spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference).
Gap: source_drift/test_gap for the six reserved descriptors; source-derived reports deferred.
Scope: add only E0104/E0111/E0112/E0206/E0332/E0333 to the existing built-in registry,
with exact §22.7 semantic names and summaries, meaning_key equal to semantic_name,
Error severity, Type/Resolution/Proof families according to their assigned ranges,
and the [initial metadata defaults](../../mizar-diagnostics/en/registry.md#descriptor-metadata),
including Active status, since = spec-22.7-v1, the §22.7 doc_url and empty lifecycle extras.
Use existing descriptor constructors and lookup/validation infrastructure; no new API/type.
Affected artifacts: registry/builtin.rs, registry integration tests and the paired
[registry owner](../../mizar-diagnostics/en/registry.md#reflection-diagnostic-adoption).
[Failure records](../../mizar-diagnostics/en/failure_record.md) retain existing construction rules.
Forbidden: producer emission, executable diagnostic coverage/runner credit,
source-aware owning-stage failure records, new IDs/syntax, inferred codes from text,
silent candidate fragment publication or protected .miz/expectation intent changes.
B1-B8 remain record classes, never diagnostic codes. Existing descriptors remain unchanged.
Test matrix (cap six cases, one per reserved code): each exact code/name/summary/family
must round-trip through lookup with all metadata above and shared draft/record construction;
the existing allocation lock admits exactly those six additions and existing registry
validation/compatibility tests retain their infrastructure scope. These synthetic records
establish registry usability only; source-bound producer/limit-stage tests remain deferred.
Audit: no producer emission, semantic coverage, runner credit or follow-up ownership change;
leave the coverage audit and corpus/trace artifacts untouched.
Reviews: specification/test intent, sufficiency, implementation precision, volume/scope,
source/document consistency and bounded-task quality under the canonical protocol.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings;
both lint_policy suites; cargo test -p mizar-diagnostics; scoped local links/fragments.
Full cargo test is required on the pushed tree. Descriptor-only work adds no executable path.
Exit: all six descriptors match the frozen metadata and reviewed registry tests;
no completion or coverage claim for deferred adoption.
