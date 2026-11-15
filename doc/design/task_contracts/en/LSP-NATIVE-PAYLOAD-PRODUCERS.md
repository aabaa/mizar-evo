# Task LSP-NATIVE-PAYLOAD-PRODUCERS: approved native payload implementation
Canonical language: English; [Japanese pointer](../ja/LSP-NATIVE-PAYLOAD-PRODUCERS.md).
Status: frozen implementation contract. Tier: full; new producer and ranged host boundary, no pattern predecessor.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md); lower stages: Syntax/Parser typed correction, Frontend preservation, Diagnostics canonical attachment, Driver original-source production.
Consumers: Tasks17/19/38; Task39 redispatch is separate. Dependencies: approved [producer package](../../mizar-lsp/en/native_payload_producers.md) Q1 A/Q2 A/Q3 B/Q4 A/Q5 A and [validators](../../mizar-lsp/en/code_action_validators.md#status-and-authority).

## Authority and scope

Authority: spec [export grammar](../../../spec/en/12.modules_and_namespaces.md#124-export-statements), [fix suggestions](../../../spec/en/22.error_handling_and_diagnostics.md#2213-fix-suggestions-and-help-annotations), [frontend adoption](../../../spec/en/22.error_handling_and_diagnostics.md#2223-frontend-diagnostic-adoption), [editor actions](../../../spec/en/23.package_management_and_build_system.md#2361-hover-code-actions-and-diagnostics).
Existing intent: `module/tests/parser_diagnoses.rs::parser_diagnoses_missing_export_semicolon_before_next_item`; Driver frontend conversion tests; LSP action, explanation and deferred-validator fixtures remain distinct from genuine production.
Gaps: `design_drift` and `test_gap`: genuine E0041 has no typed insertion/backing; success-only full snapshots cannot carry this source-error profile to consumers.
Implement only syntactically complete export path/list before a recognized next item: advisory empty-range `;` insertion with expected text `""`, preserving next-token diagnostic span; no EOF/malformed/other-site inference.
Retain canonical context JSON, CacheRecord key/encoding, role-framed hashes, exact subject/snapshot association, deterministic occurrence keys and independent payload refusal as specified by the approved producer owner.
Define Q3-B separately in [ranged diagnostic host input](../../mizar-lsp/en/ranged_diagnostic_host.md); implement terminal source-error ranged publication from the original native capture, with immutable store and canonical command association, without successful semantic/artifact publication.
Use existing action/explain/command validation, cancellation and error rules. Current edits require original editor association; retained explanations keep original coordinates/content as stale, and replacement invalidates old locators even for equal ids.
Writes: affected Syntax/Parser/Frontend/Driver/LSP source and tests; this contract pair, paired producer/adapter and boundary-owner docs, owning plan links. No serialized phase/cache codec or SourceId format.
Forbidden: language/spec/recovery/severity/acceptance changes, arbitrary command/effect owners, Task25/39/40/41 implementation, AGENTS/protocol edits, semantic coverage credit or SCA-003 status changes.

## Test matrix and exit

Cap six grouped cases; extra cases require a cited authority clause or .miz intent:
1. Real parser→Frontend→Driver supported export path/list before item yields exact E0041 insertion/context/command; EOF, malformed paths and other semicolon sites preserve diagnostics without guessed payloads.
2. Repeated sites, reordered aggregation and multiple records/fixes retain deterministic canonical payloads and exact occurrence-to-record/command/backing association; sibling/foreign ids grant no authority.
3. Genuine OpenBuffer source with Unicode in permitted regions, BOM/CRLF and preceding lines reaches ranged host: actual advisory action, StructuredText JSON and resolved/executed explain command match independent loaded-byte/raw UTF-16 facts; full semantic snapshot remains absent.
4. Same produced payload after edit/version-only change/close-reopen/equal-id replacement: stale actions/commands refuse; retained same-selected reads keep original URI/version/ranges/content as stale; obsolete requests and in-flight replacement return stale_handle.
5. Missing/wrong encoding/hash/source/handle/publication/dependency or command binding refuses only affected payload; diagnostic and independently valid edit survive; cancellation/missing-host behavior remains unchanged.
6. Non-range, nonterminal, obsolete or foreign capture cannot enter adapter and refusal preserves the prior selection; genuine eligible clean/replacement publications select atomically with original source lifetime retained, typed consumer fixtures provide no native evidence.
Reviews: independent specification/test-intent before freeze; test sufficiency, precision, volume/scope, source/doc consistency and final full-tier assessment over freeze plus implementation.
Verification: fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites (links/fragments included), Syntax/Parser/Frontend tests and parse-only/syntax-smoke corpus, affected Driver native publication/currentness tests, LSP library/policy/stdio and a short genuine end-to-end route.
Changed canonical context/hash/retention obligations run focused positive/refusal matrices; unrelated proof/kernel/evidence/cache replay heavy groups defer to cadence/integration. Full cargo test only on named cadence trigger.
Exit: freeze then implementation locally committed as one changed task; no push; producer prerequisites land without claiming Task39 completion.
