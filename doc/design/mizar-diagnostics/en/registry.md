# Diagnostic Registry

> Canonical language: English. Japanese companion:
> [../ja/registry.md](../ja/registry.md).

## Purpose

`mizar-diagnostics` owns the `DiagnosticCode` registry. The registry gives every
diagnostic published through the shared diagnostics model a stable machine
identity, validates that codes are not reused for different meanings, and
exposes compact metadata for rendering, aggregation, fixes, explanations, and
external consumers.

This task defines the initial stable code space for the diagnostics enumerated by
`doc/spec/en/22.error_handling_and_diagnostics.md`. Other architecture surfaces
that mention diagnostics, such as build failures, cache/artifact compatibility,
documentation extraction, and editor-only overlays, remain code-space gaps until
their normative ranges and descriptors are added here. `mizar-diagnostics`
remains the owner for those future allocations, but task 3 must not invent
placeholder ranges or descriptors for them.

The registry does not own phase semantics, proof acceptance, trusted status,
kernel acceptance, artifact mutation, LSP protocol conversion, or driver session
orchestration. Those services may consume registry metadata, but they must key on
`DiagnosticCode` rather than message text, localized text, ordering, or semantic
name.

## Code Shape

A `DiagnosticCode` has the form `Xnnnn`:

- `X` is the severity prefix: `E`, `W`, or `I`.
- `nnnn` is a four-digit decimal number.
- The prefix and numeric value are part of the stable identity.
- Numbers are assigned permanently. A retired number is never reissued.

Code ranges follow the language specification. The `PhaseFamily` labels in this
table are the canonical registry vocabulary used by descriptors and tests.

| Range | `PhaseFamily` | Scope | Default severity |
|---|---|---|---|
| `E0001`-`E0099` | `Syntax` | Lexical and syntax diagnostics | Error |
| `E0100`-`E0199` | `Type` | Type diagnostics | Error |
| `E0200`-`E0299` | `Resolution` | Resolution, overload, and template diagnostics | Error |
| `E0300`-`E0399` | `Proof` | Proof and ATP diagnostics | Error |
| `E0400`-`E0499` | `Logic` | Logical consistency and verification-condition diagnostics | Error |
| `E0500`-`E0599` | `Algorithm` | Algorithm verification diagnostics | Error |
| `W0001`-`W0099` | `StructuralWarning` | Structural warnings | Warning |
| `W0100`-`W0199` | `ProofWarning` | Proof and ATP warnings | Warning |
| `W0200`-`W0299` | `AlgorithmWarning` | Algorithm and contract warnings | Warning |
| `W0300`-`W0399` | `CompatibilityWarning` | Compatibility and packaging warnings | Warning |
| `I0001`-`I0099` | `Info` | Informational display diagnostics | Info |

An implementation diagnostic outside these ranges is a registry error, not a
normal user diagnostic.

## Descriptor Metadata

Each active or retired code has one descriptor:

| Field | Required | Compatibility role |
|---|---:|---|
| `code` | yes | Stable public identity. Immutable after allocation. |
| `meaning_key` | yes | Stable, non-localized registry key for compatibility validation. It is not displayed as the user-facing identity. |
| `semantic_name` | yes | Dot-separated human-readable name. It may be renamed if `meaning_key` stays the same and the old name is recorded as an alias. |
| `default_severity` | yes | Must agree with the code prefix. Changing the descriptor's default severity requires a new code unless the language specification changes the code range. Policy layers may still promote warnings to build failures or choose consumer-specific presentation outside registry identity. |
| `phase_family` | yes | Must match the assigned range. Moving a code to another family requires a new code. |
| `summary` | yes | Short English summary for registry audits. It may be clarified without changing identity. |
| `doc_url` | yes | Canonical documentation URL or repository-relative documentation target. The target may move if the descriptor keeps a stable redirect or replacement link. |
| `status` | yes | `active` or `retired`. |
| `since` | yes | Version or design task that first allocated the code. |
| `retired_since` | when retired | Version or design task that retired the code. |
| `replacement_codes` | optional | Codes that supersede a retired diagnostic. These are suggestions only and do not transfer identity. |
| `aliases` | optional | Previous semantic names for the same `meaning_key`. |

Message text, localized text, renderer layout, CLI ordering, LSP severity mapping,
fix suggestion wording, and explanation text are projections from a diagnostic
record. They are not registry identity.

For the initial registry, unless a row explicitly overrides it:

- `meaning_key` equals `semantic_name`;
- `status` is `active`;
- `since` is `spec-22.7-v1`;
- `doc_url` is
  `doc/spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference`;
- `aliases`, `retired_since`, and `replacement_codes` are empty;
- `summary` is the row's `Summary` value in the initial-allocation table.

## Deferred Code-Space Gaps

The following surfaces are explicitly outside the task-2 initial allocation:

| Surface | Classification | Task-2 disposition |
|---|---|---|
| Build, cache, artifact, and documentation-extraction diagnostics described by architecture/internal documents | `external_dependency_gap` / `spec_gap` | Do not allocate codes or publish current records until a later spec update adds canonical ranges and descriptors. |
| Existing lexer, frontend, parser, and resolver diagnostics that predate this crate | `external_dependency_gap` / `deferred` | Keep local diagnostics untouched until task 16 records a real adoption trigger; do not infer codes from message text. |
| Spec-21 display annotation names such as `info.thesis`, `info.resolution`, and `info.type` | `spec_gap` | Reserve `I0001`-`I0099`; do not allocate concrete info codes until the language specification maps those semantic names to numeric `DiagnosticCode` values. |

Unknown codes or descriptors outside the defined initial ranges are validation
failures for the current registry. That failure means the code space is not yet
specified; it does not authorize ad hoc local ranges.

## Allocation Rules

1. Choose the phase family from the normative language or architecture
   requirement that creates the diagnostic and from a range already defined in
   this document. If no range is defined, record a code-space gap instead of
   allocating a placeholder.
2. Allocate an unused number in that range. Gaps are intentional and may be kept
   for related future diagnostics.
3. Add a descriptor with a new `meaning_key`, semantic name, default severity,
   summary, and documentation target.
4. Add or update registry tests that lock the allocated descriptor set.
5. Do not allocate a code for an external adoption placeholder, a future LSP
   conversion shape, or a driver event that has no real producer/consumer seam.

If an existing lexer, frontend, parser, or resolver diagnostic is migrated later,
the migration must map the real producer output to an allocated code. It must not
infer identity from the old message string.

## Retirement Rules

A code is retired when the verifier no longer emits that diagnostic meaning.
Retirement preserves the descriptor with `status = retired`, `retired_since`, and
optional `replacement_codes`.

Retired codes remain valid for historical artifacts, caches, logs, and
explanation handles. New current diagnostics must not emit retired codes unless a
compatibility mode explicitly reads historical data; even then the record must be
marked as historical or stale by the consuming layer.

Retirement does not grant the number back to the allocator.

## Compatibility Validation

The registry implementation must validate descriptor compatibility against the
locked built-in registry:

- A code may not disappear; it must become `retired` instead.
- A retired code may not become active again. If the same diagnostic meaning is
  restored after retirement, allocate a new code and list it in
  `replacement_codes`.
- A code may not change prefix, numeric value, phase family, default severity, or
  `meaning_key`.
- A semantic-name rename is allowed only when the old name is retained in
  `aliases`.
- A documentation target may move only when the descriptor still points to the
  same diagnostic meaning.
- A descriptor may clarify `summary` text without changing identity.
- Two active descriptors may not share the same current
  `(phase_family, semantic_name)`. `aliases` preserve previous names for the same
  code only; they do not authorize duplicate active semantic names and are not a
  consumer identity path.
- Within one `phase_family`, every active `semantic_name` and every active alias
  string must be unique across the combined lookup domain. A current
  `semantic_name` may not reuse another descriptor's alias, and an alias may not
  reuse another descriptor's current name or alias.

Invalid or reused diagnostic codes are compiler implementation errors. They are
reported by registry validation tests or developer-mode internal errors; they are
not normal user diagnostics.

## Initial Allocations

The initial active registry mirrors the canonical code reference in
`doc/spec/en/22.error_handling_and_diagnostics.md#227-error-code-reference`.
Info codes remain reserved until display diagnostics are normatively enumerated
with numeric `DiagnosticCode` values.

| Code | Semantic name | `PhaseFamily` | Default severity | Summary |
|---|---|---|---|---|
| `E0001` | `syntax.unexpected_token` | `Syntax` | Error | Unexpected token in current syntactic context |
| `E0002` | `syntax.malformed_literal` | `Syntax` | Error | Numeric or string literal does not conform to lexical rules |
| `E0003` | `syntax.unexpected_end_of_file` | `Syntax` | Error | File ended with open construct pending |
| `E0010` | `syntax.missing_end` | `Syntax` | Error | Block opened without matching `end` |
| `E0011` | `syntax.unmatched_delimiter` | `Syntax` | Error | Parenthesis, bracket, or `do` without matching close |
| `E0012` | `syntax.reserved_keyword_as_identifier` | `Syntax` | Error | Reserved keyword used as identifier |
| `E0101` | `type.mismatch` | `Type` | Error | Expression type incompatible with required type |
| `E0102` | `type.narrowing_requires_proof` | `Type` | Error | Narrowing coercion without justification |
| `E0103` | `type.sethood.missing` | `Type` | Error | Fraenkel comprehension for type without `sethood` |
| `E0110` | `type.inference_conflict` | `Type` | Error | Conflicting type constraints within an expression |
| `E0120` | `type.mode_mismatch` | `Type` | Error | Mode incompatibility without registered widening |
| `E0121` | `type.attribute_required` | `Type` | Error | Required attribute not registered for the type |
| `E0122` | `type.attribute_contradiction` | `Type` | Error | Attribute combination rejected: mutually exclusive attributes or missing existential cluster |
| `E0201` | `resolve.ambiguous_symbol` | `Resolution` | Error | Two or more equally-ranked overload candidates |
| `E0202` | `resolve.no_viable_overload` | `Resolution` | Error | No candidate survives type-checking |
| `E0203` | `template.argument_omitted_not_inferable` | `Resolution` | Error | Template schema parameter cannot be inferred |
| `E0204` | `resolve.incompatible_refinement_join` | `Resolution` | Error | Same-root redefinitions expose incompatible joined facts |
| `E0301` | `proof.by.search_exhausted` | `Proof` | Error | `by` step: ATP exhausted resource budget |
| `E0302` | `proof.by.missing_fact` | `Proof` | Error | Goal likely provable but required lemma not in scope |
| `E0303` | `proof.obligation.open` | `Proof` | Error | Proof block closed with goals remaining |
| `E0310` | `proof.counterexample.found` | `Proof` | Error | Counterexample model found for the goal |
| `E0320` | `proof.atp.timeout` | `Proof` | Error | All ATP backends timed out |
| `E0321` | `proof.atp.axiom_budget_exceeded` | `Proof` | Error | Axiom set exceeds `max_axioms` limit for the obligation |
| `E0350` | `proof.kernel.unsupported_evidence` | `Proof` | Error | Legacy or backend proof material is unsupported under normal proof policy |
| `E0351` | `proof.kernel.missing_provenance` | `Proof` | Error | Kernel evidence is missing required provenance or context binding |
| `E0352` | `proof.kernel.invalid_substitution` | `Proof` | Error | Explicit substitution evidence failed kernel side conditions |
| `E0353` | `proof.kernel.invalid_sat_refutation` | `Proof` | Error | Kernel-derived SAT refutation check failed |
| `E0401` | `logic.contradictory_axioms` | `Logic` | Error | ATP derived `False` from declared axioms |
| `E0410` | `logic.circular_definition` | `Logic` | Error | Non-recursive definition refers to itself outside an algorithm block |
| `E0411` | `logic.circular_cluster` | `Logic` | Error | Cluster registration creates attribute inheritance cycle |
| `E0420` | `vc.postcondition.return` | `Logic` | Error | `ensures` not provable at `return` site |
| `E0421` | `vc.assert.failed` | `Logic` | Error | `assert` in algorithm body not provable |
| `E0422` | `vc.precondition.call_site` | `Logic` | Error | Callee `requires` clause not provable at call site |
| `E0423` | `vc.loop.establish` | `Logic` | Error | Loop invariant not provable before first iteration |
| `E0424` | `vc.loop.maintain` | `Logic` | Error | Loop invariant not provable to be preserved |
| `E0425` | `vc.loop.decrease` | `Logic` | Error | Termination measure not provably decreasing |
| `E0426` | `vc.recursion.decrease` | `Logic` | Error | Termination measure not provably decreasing at recursive call |
| `E0430` | `logic.cluster.inconsistency` | `Logic` | Error | Cluster registration creates contradiction |
| `W0001` | `warn.unused_variable` | `StructuralWarning` | Warning | Variable declared but never read |
| `W0002` | `warn.unused_definition` | `StructuralWarning` | Warning | Definition never referenced in package |
| `W0003` | `warn.unused_hypothesis` | `StructuralWarning` | Warning | Proof hypothesis never referenced in subsequent steps |
| `W0010` | `warn.deprecated_syntax` | `StructuralWarning` | Warning | Deprecated construct; replacement provided |
| `W0101` | `warn.redundant_hypothesis` | `ProofWarning` | Warning | `by` clause contains fact not used by accepted evidence |
| `W0102` | `warn.externally_attested_proof` | `ProofWarning` | Warning | External backend success without kernel-accepted evidence |
| `W0103` | `proof.citation.unused` | `ProofWarning` | Warning | Explicit citation absent from kernel-accepted `used_axioms` |
| `W0201` | `warn.unreachable_code` | `AlgorithmWarning` | Warning | Statement unreachable under static control-flow analysis |
| `W0202` | `warn.loop_may_not_terminate` | `AlgorithmWarning` | Warning | `terminating` algorithm with unverified loop measure |
| `W0210` | `warn.weakened_postcondition` | `AlgorithmWarning` | Warning | `ensures` weaker than what the verifier can prove |
| `W0301` | `compat.breaking_change` | `CompatibilityWarning` | Warning | Public API change requires a MAJOR bump |
| `W0302` | `compat.feature_addition` | `CompatibilityWarning` | Warning | Backward-compatible API addition requires a MINOR bump |
| `W0303` | `compat.overload_resolution_shift` | `CompatibilityWarning` | Warning | Registration, redefinition, or conditional-cluster change may shift overload/refinement resolution (heuristic MAJOR) |
| `W0304` | `compat.version_bump_insufficient` | `CompatibilityWarning` | Warning | Declared version bump smaller than required |
| `W0305` | `compat.edition_increase` | `CompatibilityWarning` | Warning | Package edition raised; MAJOR by default, review recommended |

## Lookup Contract

Registry lookup by `DiagnosticCode` returns descriptor metadata for active and
retired codes. Lookup by current semantic name is allowed only for human-facing
tooling and tests; it must never be the identity path for build tools or
consumers. Alias lookup is an audit aid only and must return the unique owning
`DiagnosticCode`, not a second descriptor. Because compatibility validation
rejects alias/current-name collisions, alias lookup must be deterministic; if a
caller supplies a registry that has not passed validation, alias lookup must fail
rather than choose a code.

Unknown codes, malformed code strings, or descriptors that fail compatibility
validation are registry failures. Aggregation and rendering must not invent a
descriptor for an unknown code.

## Public Enum Compatibility

Task 18 marks registry-owned public enums as `#[non_exhaustive]` for downstream
forward compatibility:

- `DiagnosticSeverity`;
- `PhaseFamily`;
- `DiagnosticCodeError`;
- `DiagnosticStatus`;
- `RegistryValidationError`.

`DiagnosticSeverity` follows architecture 19. The existing machine-readable
meaning and sort order of `Error`, `Warning`, and `Info` are compatibility
surfaces, and adding or reclassifying a variant requires compatibility review
and test updates. Consumers must not confuse severity with message text,
localized rendering, or phase success.

`PhaseFamily` is registry ownership metadata; it does not decide phase success,
driver orchestration, proof acceptance, or kernel acceptance. `DiagnosticStatus`
is registry lifecycle metadata; it does not decide artifact mutation or
publication. The `#[non_exhaustive]` marker is for external matching
compatibility only; internal validation may keep exhaustive matches where
deliberate review is required.
