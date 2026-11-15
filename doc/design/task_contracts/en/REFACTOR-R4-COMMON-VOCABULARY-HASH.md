# Task REFACTOR-R4-COMMON-VOCABULARY-HASH: common labels and hash utilities
Canonical language: English; [Japanese pointer](../ja/REFACTOR-R4-COMMON-VOCABULARY-HASH.md).

## Contract

- Status: frozen; full tier. Primary owner: mizar-session; consumers: Build, Cache, IR, Driver, Proof, VC and Kernel.
- Purpose: centralize phase/work-unit label conversion and duplicated hash utilities without changing any label, hash, cache key or output lineage.
- Owners: [Session plan](../../mizar-session/en/00.crate_plan.md), [Build task graph](../../mizar-build/en/task_graph.md), [Cache keys](../../mizar-cache/en/cache_key.md), [IR identity](../../mizar-ir/en/identity.md).
- Authority: [reproducible builds](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility), [snapshot contract](../../architecture/en/22.incremental_verification_contract.md), [bridge hash](../../architecture/en/15.kernel_certificate_format.md).
- Readiness: R1/R2/R3 landed; their decisions and run-28 D1/D2 remain binding. Audit 2026-11-15 section 7 R4 and H4 identify repeated label bridges, hex encoders and domain-hasher constructors.
- Classification: bounded design/source drift; no language, diagnostic, coverage-credit, trace or acceptance-policy change.

## Scope and decisions

- Session owns the existing Build PipelinePhase enum and its exact identity/CLI spellings; Build re-exports the type at its existing path. Cache/IR retain their open string-label wrappers and gain typed conversions. Diagnostic attribution and mizar-test expectation phases remain distinct vocabularies.
- Share the repeated module work-unit label formatter, preserving Rust string-debug escaping and separators. Keep Build's structured work units and task identities, all arbitrary string-label inputs and specialized layouts unchanged.
- Add Session Hash lower-hex encoding/strict decoding and length-framed BLAKE3 domain initialization. Migrate duplicate consumers in Build, Cache, IR, Driver, Proof, VC and Kernel where present; preserve framing, domain order, endian rules and each caller's errors.
- Extract the byte-identical VC/Kernel bridge mixer into Session, retaining its exact algorithm and callers. Other multiply-rotate schemes remain distinct and unchanged. Audit section 8 item 4, applicability of collision-resistance requirements, remains an unresolved owner decision; this task neither resolves it nor changes outputs.
- Allowed changes: Session shared utility modules/export plumbing and utility-only ID codec delegation; Build phase re-export; affected consumer conversions/hash helpers, regression tests, paired live API/module inventories and this contract.
- Forbidden: AGENTS/protocol/spec/.miz/expectation edits, dependency prohibitions or new dependencies, Task 9/R5, reopening earlier R units, identity/snapshot/retention restructuring, protected lexer/syntax/parser/frontend/resolve/Artifact changes, scheduling split, cache-adapter key model, IR projection service, Kernel evidence flow or ATP process/encoder changes.
- No semantic-service completion, hash-scheme migration, cache/schema invalidation or broad documentation cleanup. Spec coverage ownership and deferrals remain unchanged.

## Tests and exit

- Existing tests: [Build](../../../../crates/mizar-build/src/task_graph/tests.rs), [Cache](../../../../crates/mizar-cache/src/cache_key/tests.rs), [IR](../../../../crates/mizar-ir/src/identity.rs), [Kernel bridge](../../../../crates/mizar-kernel/src/checker.rs), [Driver](../../../../crates/mizar-driver/tests/source_load), [LSP](../../../../crates/mizar-lsp/src/server.rs). New utility vectors belong in Session tests. See [Build test design](../../mizar-build/en/task_graph.md#tests), [Cache test design](../../mizar-cache/en/cache_key.md#tests), [IR test design](../../mizar-ir/en/identity.md#tests).
- Matrix (cap six groups; expected values derive from the predecessor, never from the new helpers):
  1. Every scheduler phase retains its exact identity and snake-case CLI strings; Cache/IR typed conversions retain those strings and arbitrary labels remain supported.
  2. Module labels with ordinary, empty and escaped/non-ASCII package/path strings retain exact quoted-string formatting; specialized work-unit layouts and Build task IDs remain unchanged.
  3. Zero, full-range and mixed Hash bytes retain exact lower hex; decode round-trips and rejects wrong length, uppercase, non-hex and non-ASCII inputs with existing caller error behavior.
  4. Empty/nonempty/Unicode domains and payloads retain independently encoded BLAKE3 digests, including two-domain-prefix consumers; equal concatenations with different framing remain distinct.
  5. Empty/nonempty/long bridge domain/payload inputs retain predecessor golden hashes; VC and Kernel use the same utility and retain producer/replay agreement and refusal checks.
  6. Existing Build/Cache/IR/Driver/Proof/VC/Kernel owner suites retain task IDs, cache hits/misses, lineage, hash renderings and evidence outcomes; genuine coherent LSP snapshot E2E retains publication/currentness/retention.
- Reviews: independent specification/test intent, sufficiency, implementation precision, volume/scope, source/document consistency and final hard-gate/quality assessment.
- Freeze: both lint_policy suites including links/fragments/pairing/volume. Implementation: fmt, Clippy per AGENTS, both lint_policy suites; `cargo test -p mizar-session -p mizar-build -p mizar-cache -p mizar-ir -p mizar-driver -p mizar-proof -p mizar-vc -p mizar-kernel`; LSP `server::tests::genuine_complete_payload_manifest_first_activates_one_coherent_snapshot --exact`. Changed hash/cache/evidence obligations run immediately; unrelated resolver/proof-acceptance heavy matrices defer to periodic/integration cadence.
- Full cargo test follows named cadence triggers. Exit: focused gates, no unresolved findings and local implementation commit; no push.
