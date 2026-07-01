# SourceFrontend Adapter Readiness

> Canonical language: English. Japanese companion:
> [../ja/frontend_adapter.md](../ja/frontend_adapter.md).

Status: task D-006 classified as `external_dependency_gap`. No adapter source is
implemented in this task.

## Scope

Task D-006 checked whether `mizar-driver` can register a real `SourceFrontend`
`PhaseService` for `mizar-frontend` phases 1-3.

The adapter is allowed only when all of these owner seams exist:

- `mizar-frontend` provides the real source-to-syntax phase behavior;
- `mizar-ir` can seal the produced frontend payload through a real producer
  contract, including canonical payload bytes and a decoder for the stored type;
- `mizar-diagnostics` can accept frontend diagnostics as validated
  `DiagnosticDraft` values without using diagnostic message text as identity;
- the driver can translate a build-plan work slice into frontend source requests
  without duplicating planner or source-loading authority.

## Inventory Result

`mizar-frontend` exposes a real `Frontend<L, P, PS>::run` API that returns
`FrontendOutput<PS::Ast>`, including source, preprocessing, token, optional AST,
diagnostic, and frontend content-cache-key data.

`mizar-ir::publisher::PhaseOutputPublisher` is also real, but publishing requires
producer-supplied canonical payload bytes, a typed decoder, side tables, parent
handles, named input hashes, and explicit allowed work-unit context. The current
frontend surface does not define canonical serialization or a stable producer
payload schema for `FrontendOutput`.

`mizar-diagnostics::sink::DiagnosticSink` accepts already validated
`DiagnosticDraft` values. The current frontend surface does not provide a
driver-facing conversion from `FrontendDiagnostic` to diagnostic registry codes,
structured detail fields, freshness, and draft inputs. The driver must not use
frontend message text as diagnostic identity.

The driver core and scheduler submission specs are not yet in place, so there is
no stable driver-owned build-plan slice to `SourceUnitRequest` mapping for a
real adapter invocation.

## Classification

| ID | Class | Disposition | Evidence | Required owner seam |
|---|---|---|---|---|
| DRIVER-G-010 | `design_drift` | `external_dependency_gap` | `SourceFrontend` has real in-memory frontend output, but no canonical frontend producer payload, no diagnostics-draft bridge, and no driver build-plan input mapping. | Frontend/IR/diagnostics/driver integration must define the producer payload schema, diagnostic draft conversion, and request mapping. |

## Decision

Do not add a `SourceFrontend` adapter in D-006. The registry table continues to
classify `SourceFrontend` as `external_dependency_gap`.

When the missing owner seams exist, a future implementation may add a real
adapter with these minimum properties:

- it invokes `mizar-frontend` rather than reimplementing source, preprocessing,
  lexing, parsing, or parser recovery;
- it publishes only owner-defined canonical frontend payloads through
  `PhaseOutputPublisher`;
- it emits only validated diagnostics through `DiagnosticSink`;
- it reports missing payload/diagnostic/publisher context as a blocking adapter
  result rather than fabricating an output handle;
- it leaves LSP conversion, cache compatibility decisions, and artifact
  publication outside `mizar-driver`.

## Verification

D-006 is documentation-only. Use diff checks and review-only agents for the
classification. No Rust verification is required unless a later change adds
source code.
