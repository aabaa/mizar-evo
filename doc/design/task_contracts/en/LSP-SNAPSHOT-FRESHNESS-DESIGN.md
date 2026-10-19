# Task LSP-SNAPSHOT-FRESHNESS-DESIGN: editor snapshot boundary
Canonical language: English; [Japanese pointer](../ja/LSP-SNAPSHOT-FRESHNESS-DESIGN.md).
Status: specified (design only). Tier: full. Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), task6; consumers: tasks7–11 and later metadata readers.
Authority: spec §§1/23.6, architecture12/19, internal03/06; [sole design owner](../../mizar-lsp/en/snapshot.md).
Readiness/gap: task4 and IR13 exist; missing owner-local design is design_drift, while absent coherent build publication/metadata and runtime host stay deferred.
Scope: paired snapshot/freshness owner describing immutable original editor association, freshness, equal normalized build ids versus editor versions/incarnations, coherent full-snapshot prerequisite and separate Session/IR retention. Select no new Rust API/type.
Writes: paired snapshot owner, paired contract/pointer, LSP plan/TODO references/module row and chapter23 design mapping only. No production, fixture or specification changes.
Tests/exit: design-only; freeze future genuine-message/capture/retention test relations at the owner and preserve all existing executable intent. No task7 implementation or full snapshot/ENTRY/Steps6–8 credit.
Forbidden: invented coherent event/producer/context/manifests/indexes/Hit/Accepted, copied source/metadata representations, language/proof/diagnostic/recovery policy changes or protected spec/.miz/expectation/trace/ledger changes.
Reviews: independent SPEC/TEST/CODE(document boundary)/VOLUME/CONSISTENCY and scoped full gates; parent required cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test plus links/parity.
