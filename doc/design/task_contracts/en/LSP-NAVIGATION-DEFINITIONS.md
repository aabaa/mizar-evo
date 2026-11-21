# Task LSP-NAVIGATION-DEFINITIONS: published definition navigation (P26)
Canonical language: English; [Japanese pointer](../ja/LSP-NAVIGATION-DEFINITIONS.md).
Status: specified. Tier: full; first definition protocol surface.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), task15; consumer: editor clients.
Purpose: serve compiler-bound definition targets without building a reference index.

## Authority and readiness

Follow [architecture12 request sources](../../architecture/en/12.diagnostics_and_lsp.md#request-sources), [navigation sources](../../mizar-lsp/en/navigation.md#feature-sources), [freshness](../../mizar-lsp/en/navigation.md#snapshot-association-and-freshness), [published inputs](../../mizar-lsp/en/metadata.md#published-inputs), and [range conversion](../../mizar-lsp/en/range.md).
Existing [direct-by projection](../../mizar-artifact/en/verified_artifact.md#in-memory-direct-by-projection) and [Driver publication](../../mizar-driver/en/frontend_adapter.md#producer-metadata-publication) supply original occurrence ranges and compiler-bound theorem endpoints. ModuleSummary and PublishedImportView supply canonical export identities and imported visibility. Missing definition serving is implementation_gap/test_gap.
Current producers bind local theorem citations; imported, non-theorem and unsupported occurrences can remain unmapped. Do not reconstruct them from spellings, expression symbol strings, proof evidence or a new resolver/index.

## Scope and ownership

Advertise definitionProvider and serve textDocument/definition with standard Location or null. Validate the existing JSON-RPC lifecycle, nonempty URI and UTF-16 position; the ordinary route returns null without a metadata host.
Retain one immutable selected publication. Find one original OpenBuffer source/artifact/metadata association; select one half-open direct-by target range at the cursor. Only a compiler-bound Theorem endpoint may yield a location. Authenticate its full module identity, publication hash and complete VerifiedExport against the selected target artifact/manifest and its ModuleSummary row. Foreign targets additionally require that exact summary/export in the source PublishedImportView visible surface.
Bind the target to one captured OpenBuffer source in the same publication, with matching artifact source/hash and original maps; convert its published export range using the existing range boundary. Never synthesize URIs from module names or read target files from disk. Missing, ambiguous, malformed or unsupported associations yield null.
Definition locations cannot carry stale_artifact provenance. Suppress current delivery unless the retained snapshot is still Current, the source and target remain their captured open-buffer versions/text, and the selected snapshot has not changed. Retained metadata remains available to existing hover; no stale range is relabeled as current.
No work is scheduled; unknown/closed documents, invalid cursor boundaries, missing metadata, unmapped targets and unavailable target sources yield null. Shutdown/exit suppress feature delivery. Edits/reopen/replacement follow existing snapshot invalidation, never equal-build-id revival.
Writes: this contract pair; LSP server.rs/metadata.rs and existing tests; paired navigation/metadata/TODO/plan boundaries and source/spec correspondence where stale. No new public Rust API/type/field, index representation, producer/reader/schema/snapshot policy, raw IR read, spec/corpus/expectation/trace, AGENTS or protocol change. Reference index, arbitrary expression definitions, document symbols and resolved tokens remain later work; task15 and §23.6/SCA-003 completion credit stay open.

## Test matrix and exit

Cap: six grouped cases plus existing suites: (1) genuine local theorem citation, independent expected target location and exact half-open cursor behavior; establish this publication fixture before other cases; (2) repeated/self citations, non-theorem/unmapped references and absent semantic products do not fabricate targets; (3) BOM, Unicode comments and mixed raw newlines preserve original UTF-16 occurrence and export coordinates; (4) exact endpoint/summary/import/source validation refuses mismatches, ambiguous or unavailable targets, with foreign visibility checks through existing owner products; (5) edits/version changes, close/reopen, replacement and terminal lifecycle never deliver stale locations; (6) capability/params/notification/lifecycle behavior and framed executable request coverage without synthetic publication or pending work.
Independent specification/test-intent review before freeze; sufficiency, precision, volume/scope, source/document consistency and full-tier assessment after implementation. Use checked-tree logs; no repeated producer or unrelated heavy matrices.
Freeze runs both lint_policy suites including links/fragments/pairs. Implementation runs fmt, workspace all-target/all-feature Clippy with warnings denied, both lint_policy suites, all mizar-lsp tests and short framed stdio E2E. Existing publication/reader/snapshot policies are unchanged; their heavy matrices defer to integration cadence. Full workspace test runs only on a cadence trigger; push is disabled.
Exit: freeze then implementation commits count as one changed task. No reference index or new semantic credit.
