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

Approved hash, provenance and command contracts are owned by the [validator contracts](code_action_validators.md); activation requires their authenticated producer registrations and validators.

`src/diagnostics.rs` converts published fixes through the existing metadata/build host. The ordinary stdio path without a publication host returns no actions. Requests use the live document version and original source association; primary diagnostic ranges select fixes by half-open overlap, with empty ranges treated as points inside a nonempty interval or coincident points.

The server advertises quickfix resolution. Clients must support code-action data and resolving edits; initial actions contain title/kind/data only. The locator binds an opaque process-local publication identity that is never reused, even when build ID, raw text and document version repeat. `codeAction/resolve` treats data as a locator and reconstructs the action from the current index, returning versioned `edit.documentChanges` only after revalidation. Unavailable or replaced actions fail with ContentModified. Client-supplied edits, commands and diagnostic contents grant no authority.

Local and snapshot-bound suggestions with matching explicit expected text can be offered at all three edit-bearing applicability levels. Informational suggestions, unknown classes and suggestions requiring hash/artifact/command validation unavailable at this boundary are filtered. The opaque hash is not interpreted as a source or artifact hash without an owning validator. No command wire encoding, explanation-query behavior, new action class or public diagnostic code/name/alias is defined here; tasks18/19 remain separate.

These rules consume existing suggestions and snapshot associations independently of deferred OpenBuffer artifact provenance/storage and Session SourceId encoding. They neither presume nor decide those matters and change no producer, diagnostics, artifact or snapshot behavior.

## Test design

Task17 tests cover existing fix safety/applicability classes, deferred resolution, versioned multi-source edits, independent raw UTF-16 edit round trips and invalid/stale refusal. Native publication tests cover currentness independently: existing production diagnostics do not yet attach fixes. Framed stdio covers the unavailable-publication path. Unvalidated artifact/command consumers remain deferred; §23.6/SCA-003 credit does not change.
