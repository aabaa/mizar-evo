# Code actions
Canonical language: English; [Japanese companion](../ja/code_action.md).

## Responsibility

LSP converts existing structured fix suggestions into editor code actions under architecture12 [Fix Suggestions](../../architecture/en/12.diagnostics_and_lsp.md#fix-suggestions) and [Code Actions](../../architecture/en/12.diagnostics_and_lsp.md#code-actions). Diagnostics owns suggestion production; LSP offers actions without changing producers, proof acceptance or artifacts.

## Request sources and conversion

Use the [request sources](../../architecture/en/12.diagnostics_and_lsp.md#request-sources): `FixSuggestion`, `missing_facts.json` and explanation refs. On a request carrying document version and range, look up diagnostics and their suggestions in the current snapshot under the [five-step flow](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#code-actions), attached to the existing [diagnostic publication](diagnostics.md#build-index-publication).

Consume the producer's existing `FixSuggestion` payload, not a parallel fix schema. Its title, applicability and supplied edits or optional command describe the offered action. Preserve existing preconditions, including supplied `required_snapshot` and `required_text_hash`, under [internal03's Fix Suggestion](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#fix-suggestion); this conversion adds no fix kind, field or payload shape.

| Action class | Conversion and safety |
|---|---|
| Local text edits | Offer the suggestion's explicit edits, such as inserting `qua` or an annotation or replacing a typo; require matching range and expected text. |
| Artifact-assisted edits | Use the suggestion with `missing_facts` or explanation refs for adding a lemma or refining a citation; require matching artifact and source hashes and validate the relevant handle. |
| Commands | Offer the supplied command, such as explain, show VC panel or re-run verification; capture snapshot and diagnostic handles and revalidate when executing. No source edit unless the command later validates it. |

Prefer small explicit edits over broad rewrites. Proof suggestions distinguish backend-derived candidates from kernel-verified `used_axioms`; a candidate is not a verified proof fact.

## Applicability and freshness

Offer an action only when the suggestion's applicability and its class validation hold for the current snapshot. All source-text-dependent edits validate the target range and expected text; artifact-assisted edits additionally validate artifact hash, source hash and the relevant explanation or `missing_facts` handle. These rules supply the filtering criteria without adding a new applicability policy.

Use the associated source snapshot and existing [range conversion](range.md) at the LSP UTF-16 boundary. Preserve [snapshot association](snapshot.md#source-association); never combine suggestion ranges from an old source with current-buffer text or maps.

Under the [incremental update flow](../../architecture/en/12.diagnostics_and_lsp.md#incremental-update-flow), replacement invalidates stale code actions from the previous snapshot. Do not offer them as current actions; retained diagnostics or explanation handles do not authorize stale edits.

## Explicit application

The server offers code actions and never applies suggestion edits on its own. Edits are applied only through the editor's explicit code-action application path after the applicable range/text and artifact/source validation passes. Earlier validation while offering an action does not permit applying it after its preconditions become stale. Commands retain their execution-time handle revalidation and source-edit validation rule.

## Implementation boundary

Task16 specifies conversion and safety only. Task9 diagnostic publication, task17 code-action implementation and tasks18/19 explanation queries remain separate. No command wire encoding, explanation-query behavior, new action class or public diagnostic code/name/alias is defined here.

These rules consume existing suggestions and snapshot associations independently of parked OpenBuffer artifact provenance/storage, Session SourceId encoding and registry activation decisions. They neither presume nor decide those matters and change no producer, diagnostics, artifact or snapshot behavior.

## Test design

This design unit changes no code or tests. Task17 retains its action-fixture-per-fix-kind and edit range-conversion round-trip obligations, with dependencies on task9 and `mizar-diagnostics` task13. Task17 and the module remain open; §23.6/SCA-003 credit does not change.
