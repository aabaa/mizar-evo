# Task LSP-DETERMINISM-FRESHNESS: determinism and freshness suite
Canonical language: English; [Japanese pointer](../ja/LSP-DETERMINISM-FRESHNESS.md).
Status: frozen scope. Tier: full; independent hardening suite, no reviewed pattern predecessor.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), [task20](../../mizar-lsp/en/todo.md). Consumers: existing diagnostic and feature handlers.
Purpose: add bounded generated properties for reproducible responses and explicit stale retained reads.

## Authority and readiness

Authority: architecture20 [incremental properties](../../architecture/en/20.test_strategy.md#incremental-and-parallel-properties) and [property tests](../../architecture/en/20.test_strategy.md#property-based-tests); architecture12 [stable snapshots](../../architecture/en/12.diagnostics_and_lsp.md#lsp-reads-stable-snapshots) and [freshness](../../architecture/en/12.diagnostics_and_lsp.md#open-buffers-override-disk-source).
Dependencies: completed tasks11 and13. Existing owners: [snapshot association](../../mizar-lsp/en/snapshot.md#source-association), [currentness](../../mizar-lsp/en/snapshot.md#currentness-and-retention), [diagnostic ordering](../../mizar-lsp/en/diagnostics.md#ordering), [hover](../../mizar-lsp/en/metadata.md#implementation-boundary), [syntax tokens](../../mizar-lsp/en/navigation.md#implementation-boundary).
Gap: test_gap; existing example tests cover individual freshness transitions and ordering, but not generated replay/permutation properties across publication and feature boundaries.
Existing tests: diagnostics.rs conversion tests; server.rs full_snapshot_registry_a and task9_publication_api_tests; hover_tests.rs and semantic_token_tests.rs genuine publication cases.

## Scope and ownership

Use existing fixtures, native captures, genuine coherent publications, aggregator and public handlers. Add no dependency, production API, artifact shape, semantic rule or producer behavior.
Identical snapshot means identical captured build inputs and source association; compare complete wire values/bytes with identical request ids. Rebuild generations may differ internally; do not strip response fields to force equality.
Nonempty diagnostic conversion uses explicitly typed draft fixtures, not fabricated coherent publications. Genuine successful publications exercise clean diagnostic clearing and real hover metadata. Native fix producers are absent: code-action replay covers graceful empty results only, with no positive fix-delivery claim.
Stale retained reads preserve the original source association and are marked StaleArtifact/stale_artifact. Current diagnostic delivery is suppressed, including writer bytes; syntax responses continue to use live buffers. Missing data is not pending work.
Writes: tests within existing diagnostics.rs and server.rs, this contract pair, task20 plan-index/TODO links and completion markers in the existing EN/JA trees. No other owner-document boundary or spec-coverage credit changes.
Forbidden: doc/spec, .miz/expectation/trace changes; Task15 parked unterminated-string/newline issue; Task19/later work; scheduling, retention, producer or feature behavior changes. If tests expose a production defect, classify it against this scope before repair; parked defects are reported, never repaired here.

## Fixed test matrix

Cap: three grouped tests, deterministic enumeration without a random-library dependency; additional groups require an authority-backed sufficiency finding.
1. Nonempty diagnostic permutation: two captured sources with Unicode and LF/CRLF raw text; three distinct drafts spanning sources plus a duplicate. Enumerate all draft arrival permutations. Expect identical nonempty notification values/serialized bytes, canonical record handles/order, raw UTF-16 ranges and unchanged source/index inputs.
2. Genuine equal-input replay: six source layouts (LF/CRLF crossed with zero, one and two leading spaces), both source URIs, two genuine builds per layout with reversed source/VC submission order and reversed query order. Expect equal build ids and complete diagnostic notifications/framed bytes, nonempty hover and syntax-token responses, and empty code-action responses; each publication is Current.
3. Freshness sequences: enumerate all six orders of version-only edit, normalization-equivalent BOM edit and identical-tuple reopen, plus separate recapture and publisher-obsolete sequences. Start with genuine Current data; after every invalidator expect the retained association to remain readable as StaleArtifact, stale hover naming original URI/version without a live range, no diagnostic notifications/writer bytes, and repeatable syntax responses from live text. A genuine rebuild restores Current responses for live non-obsolete sequences.

## Verification and exit

Reviews: specification/test intent before freeze; independent sufficiency, precision, volume/scope, source/document consistency and full-tier assessment over the unit.
Freeze: both lint_policy suites, including recursive link/fragment/pair/volume checks. Implementation: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; both lint_policy suites; cargo test -p mizar-lsp (includes framed stdio); full cargo test on inherited outstanding-task cadence.
No production executable change: existing framed stdio tests provide the end-to-end check. Unchanged proof/kernel, resolver edit-clean, evidence and cache obligations defer to the cadence full run; no new test selector or policy.
Exit: freeze and implementation commits count as one changed task; task20 is checked only after the matrix and required reviews/checks pass. No push. Missing producer-backed features remain unavailable; no crate-exit, §23.6 or SCA-003 completion claim.
