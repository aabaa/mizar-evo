# Task LSP-CODE-ACTION-DESIGN: code-action design
Canonical language: English; [Japanese pointer](../ja/LSP-CODE-ACTION-DESIGN.md).
Status: specified (design only). Tier: full; no frozen predecessor contract for this code-action surface.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), task16. Consumer: task17.
Purpose: specify conversion of existing structured fix suggestions into editor code actions, applicability filtering and no automatic application.

## Authority and readiness

Authority: [LSP task16](../../mizar-lsp/en/todo.md), [roadmap Step8 item2](../../todo.md#step-8--user-facing-consumer-wave-); architecture12 [Fix Suggestions](../../architecture/en/12.diagnostics_and_lsp.md#fix-suggestions), [Request Sources](../../architecture/en/12.diagnostics_and_lsp.md#request-sources), [Incremental Update Flow](../../architecture/en/12.diagnostics_and_lsp.md#incremental-update-flow) and [Code Actions](../../architecture/en/12.diagnostics_and_lsp.md#code-actions); internal03 [Fix Suggestion](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#fix-suggestion) and [Code Actions](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#code-actions).
Dependency: completed task8 [diagnostic publication](../../mizar-lsp/en/diagnostics.md#build-index-publication). Task17 implementation additionally depends on task9 and mizar-diagnostics task13.
Gap: absent code-action owner document is design_drift. Authorities determine the design independently of parked OpenBuffer provenance/storage, Session SourceId encoding and registry activation decisions; none is decided here.

## Scope and ownership

Author paired mizar-lsp/{en,ja}/code_action.md, each at most 80 lines. Convert the existing FixSuggestion structure using FixSuggestion, missing_facts.json and explanation refs; define no new payload or field.
Cover local text edits, artifact-assisted edits and commands under their existing safety rules. Offer only applicable suggestions validated for the current snapshot; text-dependent edits validate range/expected text, artifact-assisted edits also validate artifact/source hashes and relevant handles, commands capture/revalidate snapshot and diagnostic handles. Invalidate stale actions on snapshot replacement.
Never auto-apply edits: explicit editor application requires the class validation. Prefer small explicit edits; distinguish backend-derived proof candidates from kernel-verified used_axioms.
LSP owns conversion; diagnostics owns fix production, snapshot owns source association/freshness, Artifact owns stored inputs. Reuse [range conversion](../../mizar-lsp/en/range.md); no producer, diagnostic, artifact or snapshot behavior changes.
Forbidden: new fix kinds/fields/action classes or public diagnostic codes/names/aliases; protected spec/test/trace changes; task9 publication, task17 implementation or tasks18/19 explanation-query design.
Writes: contract pair and one task16 Task Index row per LSP plan first; then specification pair and task16-only TODO pointers; new-pair audit rows only if required. Outside dispatch paths, only NT2 mechanically required registration lines are permitted.
Coverage: design only; no module completion or §23.6/SCA-003 credit. Task17 stays open.

## Verification and exit

Reviews: independent specification/test-intent before freeze; sufficiency, precision, volume/scope and source/document consistency over the whole unit, then full-tier assessment.
Fixed matrix, no test/code additions/changes/deletions: before freeze and after specification commit, run both `cargo test -p mizar-checker --test lint_policy` and `cargo test -p mizar-test --test lint_policy`, plus local link/fragment checks.
Specification headings/anchors match after heading-text normalization; contract companion is a pointer. English contract cap: 60 lines/660 words. `cargo test --locked -p mizar-lsp` passes.
Exactly two local documentation commits, one changed task; combined diff limited to dispatch paths/NT2 registration lines; final `git status --porcelain` empty. No fmt/Clippy/full workspace test: no cadence trigger, push disabled.
Exit: freeze first, specification/task16 completion second. Stop for required undetermined behavior (classified gap/exact decision), size-cap failure, or required lint/link fixes beyond dispatch/NT2 scope.
