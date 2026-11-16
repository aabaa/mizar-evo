# Bilingual sync audit

This document owns language parity boundaries for the `mizar-lsp` design tree.

## Language ownership

- The [non-range diagnostic presentation proposal](non_range_diagnostics.md) is paired; Q1–Q5 await owner approval.

- The [checkpoint observation proposal](observation_seam.md) is paired; Q1–Q5 await owner approval.
- The approved [nonterminal execution seam](execution_seam.md) is paired; Q1–Q5 use option A.
- The approved [native payload producers](native_payload_producers.md) and [ranged diagnostic host input](ranged_diagnostic_host.md) are paired.

- The [producer metadata publication proposal](producer_metadata_publication.md) is paired; Q1–Q5 await owner approval.
- The nine module designs remain EN/JA translations: [range](range.md), [server](server.md), [snapshot](snapshot.md), [diagnostics](diagnostics.md), [build bridge](build_bridge.md), [metadata](metadata.md), [navigation](navigation.md), [code actions](code_action.md), and [explanations](explain.md).
- The [validator contract proposal](code_action_validators.md) is also paired; its options await owner approval.
- Status, roadmap, and audit owners are [crate plan](00.crate_plan.md), [TODO](todo.md), [source/spec correspondence](source_spec_correspondence.md), and this audit. Their Japanese companion paths are pointers to English. The plan pointer retains its Task Index contract links; the TODO pointer retains the `順序付きタスク一覧` inbound anchor.

## Retained boundaries

No translation-only parity gap is intentionally deferred. The [code action](code_action.md) and [navigation](navigation.md) pairs retain the OpenBuffer provenance/storage and Session SourceId encoding deferrals; the [server](server.md) pair retains executable host/build integration as deferred. These remain implementation boundaries, and parity grants no specification or coverage credit.
