# Task DOC-RR-HISTORY: retain historical AST specification references
Canonical language: English; [Japanese pointer](../ja/DOC-RR-HISTORY.md).
Status: complete (documentation preparation only). Tier: full gates. Owner: [kernel plan](../../mizar-kernel/en/00.crate_plan.md).
Dependencies: the available local discussion record and frozen AST-SPEC references.
Authority: [protocol migration and ownership rules](../../autonomous_crate_development.md#migration-policy); [compaction rules](../../documentation_compaction_rules.md).
Gap: design_drift in non-reproducible historical reference targets.
Scope: preserve the referenced agreed tables, examples, reflection plan, decisions, acceptance conditions and appendices in one [frozen quotation](../../archive/ast_spec_decision_source.md); replace only the AST-SPEC-00 through AST-SPEC-14 source hrefs.
Boundary: retain current local text verbatim with honest provenance; the exact original task source version is not established. Current language authority remains doc/spec/en/.
Forbidden: changing old contract content/status, implementation, .miz/expectations, specification, configuration, coverage/trace, user files, lint or ledger baselines.
Tests: equivalence of retained local text and href-only changes; tracked-target resolution and EN/JA pairing, without treating old completion as implementation evidence.
Migration: no tracked source section is removed or redirected, so this link repair is not a whole-section legacy-compaction ledger batch.
Reviews: independent specification/documentation, equivalence/test sufficiency, implementation-preparation, volume/scope, consistency and EN/JA; full gates for this document scope.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test; quotation/href equivalence and protected-scope checks.
Exit: historical references resolve to a retained tracked quotation without promoting its text into current requirements.
