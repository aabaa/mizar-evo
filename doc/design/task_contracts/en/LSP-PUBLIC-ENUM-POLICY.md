# Task LSP-PUBLIC-ENUM-POLICY: public enum compatibility
Canonical language: English; [Japanese pointer](../ja/LSP-PUBLIC-ENUM-POLICY.md).
Status: specified. Tier: full; crate-wide public-enum policy decision.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), [task21](../../mizar-lsp/en/todo.md); consumers: Rust callers of mizar-lsp.
Purpose: apply the [frontend task25 procedure](../../mizar-frontend/en/todo.md) to every LSP-owned public enum.

## Authority and readiness

Dependency: task11 is complete. Authority: all [LSP module specs](../../mizar-lsp/en/todo.md#module-implementation); enum owners are [range](../../mizar-lsp/en/range.md#public-enum-policy), [snapshot](../../mizar-lsp/en/snapshot.md#public-enum-policy) and [build bridge](../../mizar-lsp/en/build_bridge.md#public-enum-policy).
The missing explicit per-enum decisions and regression guard are design_drift and test_gap. Existing attributes and variants already implement the selected policies; no runtime or language behavior changes are required.

## Scope and ownership

Record each decision and rationale only in its paired owning module spec. Keep RangeMapError non-exhaustive; deliberately retain exhaustive LspSnapshotFreshness and BuildBridgeError. Preserve exhaustive internal matches. Public reexports count toward the inventory even when their defining module is private.
Use existing lint-policy tests to guard the complete source inventory, chosen attributes and paired owner documentation; compile external matches against the existing public routes. Reuse current mapper, snapshot and bridge behavior tests. No new correspondence document: owner sections provide the policy correspondence; task22 retains the broader source/spec audit.
Writes: this contract pair, paired owner sections, LSP plan/TODO links and existing LSP tests. No new variants, public seams, fallbacks, protocol/wire changes, producer behavior, protected specifications/tests/trace, AGENTS/protocol edits, task15 remainder or later tasks. Coverage and SCA-003 status remain unchanged.

## Test matrix and exit

Cap: three grouped cases. (1) Scan all LSP source files and compare public enum declarations with the complete three-enum inventory, including the reexported bridge error. (2) Check the non-exhaustive attribute on RangeMapError and its deliberate absence on the other two; check each paired owner policy. (3) Compile downstream wildcard handling for RangeMapError and exhaustive matches for the freshness states and capture/submission error stages, using public paths.
Independent specification/test-intent review before freeze; sufficiency, precision, volume/scope, consistency and final full-tier assessment after implementation.
Freeze: both lint_policy suites, including link/fragment/pair checks. Implementation: fmt, workspace all-target/all-feature Clippy with warnings denied, both lint_policy suites and cargo test -p mizar-lsp. Existing stdio tests provide the short E2E check; no executable behavior changes. Full cargo test follows cadence; no push or tenth-task trigger applies.
Exit: all existing public enums have documented policies and regression guards; task21 is checked in both TODOs. Freeze and implementation commits count as one changed task. Push: no.
