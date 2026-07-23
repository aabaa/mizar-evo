# Source-Evidence Handoff

> Canonical language: English. Japanese companion:
> [../ja/source_evidence.md](../ja/source_evidence.md).

## Purpose And Authority

The public `source_evidence` module implements Checker Task 251. It transports
source-derived evidence requests and authenticated dependency references from
the syntax-owning runner into the checker without importing raw syntax. The
canonical authority slice is Chapters 03 §§3.2-3.6, 05 §§5.3-5.5,
06 §§6.2-6.5, 07 §§7.2-7.3 and 7.8-7.8.1, 08 §§8.1-8.3,
13 §§13.4.2, 13.5, and 13.6, 17 §§17.1, 17.3.4, and 17.7-17.8, and
19 §§19.2.1-19.2.2, 19.4.2, and 19.6.2. The source/spec audit tracks the
bounded implementation under MC-G016, MC-G018, and MC-G026.

The module is transport-only. `Requested`, `Missing`, `Rejected`, and
`Supplied` describe dependency-input state, not truth, acceptance,
consumability, satisfaction, or proof. Task 251 does not evaluate existential
gates, create facts, select inheritance paths, decide coercions, normalize
types, or publish accepted semantic results.

## Public Model

`SourceEvidenceHandoffInput` carries one `SourceId`, one `ModuleId`, ordered
`SourceEvidenceRequestInput` rows, and ordered
`SourceEvidenceResponseInput` references. `SourceEvidenceProducer::build`
authenticates the transaction against:

- the Task-249 `SourceTypeApplicationHandoff`;
- the optional Task-250 `SourceAttributeHandoff`;
- the resolver-authenticated `SymbolEnv`;
- the existing `TypeFactTable`; and
- a caller-supplied `SourceEvidenceDependencyCatalog`.

Successful construction publishes an immutable `SourceEvidenceHandoff` with a
dense `SourceEvidenceRequestTable` and `SourceEvidenceResponseTable`.
`SourceEvidenceRequestId` and `SourceEvidenceResponseId` are table-local dense
identities. `SourceEvidenceRequest` and `SourceEvidenceResponse` expose
read-only accessors, and `debug_text()` renders the transaction
deterministically.

Each request records:

- `owner: TypedSiteRef` and `site: TypedSiteRef`;
- `source_range: SourceRange` and application `source_ordinal`;
- `SourceEvidenceRecovery`;
- `SourceEvidenceRequestKind`;
- `SourceEvidenceInputState`; and
- `SourceEvidenceRequestOrigin`.

The public request kinds are `ModeExpansion`, `StructureInhabitation`,
`AttributedTypeInhabitation`, `Sethood`, `NonEmptiness`,
`InheritancePath`, and `CoercionViability`. Task 251 emits only the first
three. `SourceEvidenceRequestOrigin::SourceTypeApplication` authenticates the
Task-249 application/expression pair and an optional Task-250 attribute chain.

`SourceEvidenceResponseKey` is an opaque nonempty dependency identity.
`SourceEvidenceDependencyRecord` associates the key with its authenticated
parent request, `SourceEvidenceResponseDisposition`,
`SourceEvidenceResponseProvenance`, and optional
`SourceEvidenceResponsePayload`. `SourceEvidenceDependencyCatalog` exposes
construction, lookup, iteration, length, and emptiness checks, but is not
copied wholesale into the published handoff.

The response payload vocabulary is `ModeExpansion(ModeExpansion)`,
`StructureBaseEvidence(ExistentialGateBaseEvidence)`,
`ExistentialGate(ExistentialGateInput)`, and `TypeFact(TypeFactId)`.
`SourceEvidenceResponseDisposition` is `Rejected` or `Supplied`;
`SourceEvidenceResponseProvenance` is `ExplicitInput` or
`ExternalDependency`; and `SourceEvidenceRecovery` is `Normal` or
`Degraded`. These public enums and `SourceEvidenceError` are
`#[non_exhaustive]`.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceEvidenceRequestKind` | `#[non_exhaustive]`; callers must tolerate later evidence-request families. |
| `SourceEvidenceInputState` | `#[non_exhaustive]`; callers must tolerate later transport input states. |
| `SourceEvidenceRequestOrigin` | `#[non_exhaustive]`; callers must tolerate later authenticated source owners. |
| `SourceEvidenceResponseDisposition` | `#[non_exhaustive]`; callers must tolerate later dependency dispositions. |
| `SourceEvidenceResponsePayload` | `#[non_exhaustive]`; callers must tolerate later syntax-free dependency payloads. |
| `SourceEvidenceResponseProvenance` | `#[non_exhaustive]`; callers must tolerate later authenticated provenance classes. |
| `SourceEvidenceRecovery` | `#[non_exhaustive]`; callers must tolerate later recovery classes. |
| `SourceEvidenceError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Validation And Atomicity

Requests are dense and remain in Task-249 application order. An unattributed
application uses its root expression as origin, expression site as owner,
head site as request site, expression range, application ordinal, and mapped
expression recovery. A builtin set/object head emits no request. An
authenticated resolver `Mode` or `Structure` head emits exactly one
`ModeExpansion` or `StructureInhabitation` request.

An attributed application uses the application root expression, the chain
site and range, application ordinal, and mapped chain recovery. The chain must
name that expression and is independently authenticated by its own dense
Task-250 ordinal. Its request kind is always
`AttributedTypeInhabitation`, independent of the underlying head.
`NodeRecoveryState::Normal` maps to `Normal`; `Recovered` and `Degraded` map
to `Degraded`.

`Requested` and `Missing` have no response. `Rejected` has exactly one
authenticated response with no payload. `Supplied` has one or more
authenticated responses with payloads appropriate to the request kind. Every
catalog key is nonempty and unique, every response consumes one matching key
once, and every catalog record is consumed. Missing keys, duplicate reuse,
cross-request records, stale records, wrong disposition/payload combinations,
dangling fact references, and mismatched gate owner/range/recovery or guard
facts are `SourceEvidenceError`s.

Construction is atomic: table identity, parentage, order, source/module,
owner/site/range/ordinal/recovery, origin, symbol kind, response cardinality,
catalog association, payload kind, fact references, and gate associations are
validated before publication. Invalid input never becomes a partial handoff
or a transport-level `Rejected` state.

## Ownership And Consumers

`TypedAst` owns `Option<SourceEvidenceHandoff>` and
`TypedAst::with_source_evidence` validates installation against the typed
source/module and existing facts. `ResolvedTypedAst` only clone-preserves the
handoff and exposes it through `source_evidence()`.

Raw `SurfaceAst`, `SurfaceNodeId`, and `SyntaxKind` remain private
`mizar-test` concerns. The private
`runner::type_elaboration::source_evidence` leaf reuses the Task-249 and
Task-250 extractors and owns only exact request association and dispatch. It
does not duplicate their syntax traversal.

The real Task-251 selector contains only:

1. `fail_type_elaboration_source_type_application_payload_001`;
2. `fail_type_elaboration_imported_attribute_gap_001`; and
3. `fail_type_elaboration_attributed_reserve_gap_001`.

Together they publish ten `Missing` requests: five mode-expansion, three
structure-inhabitation, and two attributed-type-inhabitation requests, with no
responses. Only the broad Task-249 case advances to
`type_elaboration.checker.source_evidence.dependency_input_missing` /
`source_evidence_dependency_input_missing`; the two attributed cases preserve
their existing evidence-query outcome.

## Verification And Deferrals

Checker unit tests cover the four states, dense order, exact Task-249/250
association, distinct application/chain ordinals, state/response cardinality,
catalog and payload corruption, symbol authentication, gate/fact corruption,
deterministic rendering, and transactional typed-AST installation. Runner
tests use real `.miz` extraction to cover the exact selector, cardinalities,
final `TypedAst`/`ResolvedTypedAst` ownership, four-state injection through the
production consumer, corruption, and determinism.

No `.miz` source is added. One covered trace requirement,
`spec.en.checker.type_elaboration.source_evidence_request_payload`, owns the
three existing expectation sidecars. Broader source sites, semantic evidence
interpretation, fact creation, gate evaluation, accepted registration or
artifact status, Core/CFG/VC output, Tasks 252+, and Steps 6/7 remain deferred
to their explicit owners.
