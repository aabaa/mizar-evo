# Task LSP-PRODUCER-METADATA-PUBLICATION-DESIGN: completion producer proposal
Canonical language: English; [Japanese pointer](../ja/LSP-PRODUCER-METADATA-PUBLICATION-DESIGN.md).
Status: frozen spec-only unit; proposal awaits owner approval. Tier: full; new producer publication boundary, no pattern predecessor.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), Task25 prerequisite. Consumer: [completion](EDITOR-LSP-COMPLETION.md).
Purpose: define notation and import/alias metadata publication contracts; implement nothing and leave semantic choices to the owner.

## Authority and readiness

Authority: spec [§2.5.2](../../../spec/en/02.lexical_structure.md#252-user-defined-symbolic-names), [§11.1](../../../spec/en/11.symbol_management.md#111-synonyms-and-antonyms), [§12.3–12.5](../../../spec/en/12.modules_and_namespaces.md#123-import-statements), and [§23.6](../../../spec/en/23.package_management_and_build_system.md#236-lsp-integration-and-ide-support).
Existing owners: [lexical events/storage](../../mizar-lexer/en/lexical_environment.md#step-5a5-local-notation-alias-collection), [direct producer](../../mizar-resolve/en/symbols.md#source-lexical-contribution-producer), [import/alias resolution](../../mizar-resolve/en/imports.md#alias-binding), [summary schema](../../mizar-artifact/en/module_summary.md#scope), [metadata reads](../../mizar-lsp/en/metadata.md#published-inputs), [snapshot association](../../mizar-lsp/en/snapshot.md#source-association).
Gaps: `design_drift`/producer incompleteness: `symbols.rs::export_frontend_lexical_contributions` rejects operator declarations and `NotationAlias`; bounded summary candidates do not publish a complete imported/aliased surface. Lexical candidates alone do not authenticate semantic aliases or exports.
Readiness: owner authorized this definition unit; new semantic/publication choices are explicit questions with options, not approved requirements. Task25 stays parked pending approval, producer implementation and redispatch.

## Scope and boundaries

Author paired `doc/design/mizar-lsp/{en,ja}/producer_metadata_publication.md`: separate established requirements, conditional publication proposal and owner decisions, with alternatives, recommendation/tradeoff and approval dependencies.
Define source-owned spelling/arity/fixity/precedence and activation, alias origin/target/visibility, imports/namespace aliases/reexports/dependency identities, completeness/refusal and deterministic publication. Distinguish lexical candidates, semantic facts and proof authority.
Reuse existing owner schemas/codecs where sufficient; expose any missing consumable carrier as an option, without defining a production API/schema change here. Specify local versus dependency-facing data and a genuine producer-to-reader acceptance chain; never reconstruct facts in LSP.
Forbidden: Rust/test/fixture/spec/expectation/trace/ledger changes, producer or Task25 implementation, completion/coverage credit, AGENTS.md/protocol changes, Task39 native payloads, Task40 Driver execution, Task41 presentation, or parked provenance/storage/SourceId decisions.
Writes: this contract pair and one Task Index row per LSP plan first; then paired proposal, paired metadata-owner links, Task25-only TODO prerequisite link and bilingual-owner registration. No other fan-out.
Coverage: existing §23.6/SCA-003 credit and deferrals remain unchanged; this proposal grants no implementation readiness by itself.

## Review matrix and exit

Document-review matrix (cap six; positive outcomes conditional on approval): (1) direct notation plus later explicit operator and later overload: retain source spelling, activation and spelling-level metadata without a new symbol; (2) synonym/antonym, imported original and multi-spelling origin: distinguish lexical candidate from authenticated alias, preserve target/provenance or report unavailable; (3) private/recovered/unsupported declarations: no leakage or partial-success publication; (4) ordinary/aliased/duplicate/conflicting imports and reexports: canonical identity and visibility preserved, conflicts unresolved; (5) missing/wrong dependency identity/hash/schema or unpaired contribution: refuse complete publication, never infer an empty closure; (6) permutations/rebuild/edit/stale snapshots: deterministic owner canonicalization, changed interface invalidation and original-source freshness.
Existing intent: lexer `src/tests/lexical_environment.rs` operator activation/alias tests; resolver `src/symbols/tests.rs::source_lexical_pairing_rejects_incomplete_or_ambiguous_correspondence`, `src/imports/tests.rs::aliases_do_not_change_canonical_targets_or_graph_candidates`, `src/module_summary_reuse.rs::typed_lexical_shapes_bind_to_artifact_exports`; artifact `src/module_summary/tests.rs::module_summary_round_trips_through_canonical_json`. No test changes or new runtime claims.
Reviews: independent specification/test-intent before freeze; sufficiency, precision, volume/scope, consistency and final full-tier assessment over both commits; an extra case must cite authority or a .miz test.
Before each commit: both `cargo test -p mizar-checker --test lint_policy` and `cargo test -p mizar-test --test lint_policy`, local links/fragments, EN/JA section parity and `git diff --check`. Caps: contract 60 lines/660 words; combined documentation 200 added lines. Documentation only: no fmt/Clippy/runtime/E2E; full test only on a named cadence trigger.
Exit: two local documentation commits (freeze then paired proposal), one changed task; task-related tree clean, owner edits excluded; owner approval explicitly pending, followed by separate producer units and Task25 redispatch.
