# Task REFACTOR-R1-OPENBUFFER: publication-route consolidation
Canonical language: English; [Japanese pointer](../ja/REFACTOR-R1-OPENBUFFER.md).
Status: frozen scope; implementation follows freeze. Tier: full (publisher authorization API consolidation).
Owners: [Driver plan](../../mizar-driver/en/00.crate_plan.md), [IR plan](../../mizar-ir/en/00.crate_plan.md). Consumer: existing LSP snapshot registry.
Purpose: consolidate audit H2/M2 under the already-approved D1 serialized provenance and D2 isolated ephemeral staging plus OpenBuffer/InternalOnly publication.

## Authority and readiness

Authority: owner dispatch R1; [native capture](../../mizar-driver/en/request.md#native-source-capture), [captured sources](../../mizar-driver/en/frontend_adapter.md#native-captured-sources), [genuine draft](../../mizar-driver/en/frontend_adapter.md#source-theorem-unpublished-module-draft), [publisher authorization](../../mizar-ir/en/publisher.md#published-output-authorization), [published parents](../../mizar-ir/en/dispatch_input.md#published-parents), [projection eligibility](../../mizar-ir/en/projection.md#required-output-publication-eligibility), [snapshot retention](../../mizar-lsp/en/snapshot.md#currentness-and-retention).
Dependencies: genuine producer d3b51a2d6 and registry activation cee89c158. D1/D2 are settled; no open semantic decision.
Gap: design_drift/source_drift; repeated SourceOrigin conversion and parallel publisher validation bodies with a boolean dispatch selector. Existing behavior is the preservation oracle, not new language intent.

## Scope and ownership

Driver owns one private SourceOrigin-to-(OutputOrigin, PublicationTarget) function used by every existing native publication conversion. Preserve OpenBuffer/InternalOnly and the existing non-OpenBuffer PackageSource/CurrentPackage fallback, including Generated; existing producer eligibility gates remain authoritative.
IR owns one validate_output function with an explicit OutputAuthorization enum (CurrentPackage or Published), replacing both old validation methods. Preserve snapshot checks, exact output/generation roots, error classes and precedence, publication records and storage-validity separation. Dispatch input carries this enum instead of a boolean; existing parent constructors keep their contracts.
Migrate all existing validation callers, including IR cache/projection and Driver/LSP production/tests, mechanically to their prior mode. Do not retain parallel compatibility validators or add a route representation.
Writes: existing mizar-driver, mizar-ir and mechanically affected mizar-lsp source/tests; paired Driver/IR owner documents only for the changed API or route; this contract pair and one index row per owning plan.
Forbidden: language/spec/.miz/expectation/trace/coverage changes; Artifact schema or crate changes; Session identities/snapshots/retention; lower compiler crates; scheduling, cache-key, projection-service, Kernel evidence or ATP redesign; dependency changes; R2–R5 and LSP task9. No AGENTS.md/protocol edits.
Semantic deferral: future origin classes and new acceptance/storage policy remain owner decisions. Existing D1/D2 provenance, ephemeral cleanup and LSP current/stale retention remain unchanged.

## Test matrix

Cap: six categories, using existing genuine fixtures and assertions; add cases only for a demonstrated gap.
1. Disk/current-package publication passes CurrentPackage and Published authorization; OpenBuffer/InternalOnly passes Published and fails CurrentPackage with its existing error.
2. Unsupported internal origins, direct sealing, foreign publishers, failed seals and unrooted/resealed generations never gain authorization; physical collection remains independently rejected.
3. Unknown/mismatched/obsolete/replaced snapshots preserve rejection precedence; retained old readers do not gain current authority.
4. Genuine native Disk/OpenBuffer pipeline outputs preserve parent lineage, original source/version/hash provenance and route, including independent clean edit/replay controls.
5. Genuine theorem draft/main/types/statements/manifest paths preserve both origins, refusal results and ephemeral cleanup; coherent snapshot publication remains complete.
6. Existing LSP registry activation/current-stale/replacement/retention tests and IR cache/projection regressions pass with the same authorization selection.
Existing anchors: IR publisher tests published_authority_tracks_only_current_and_successful_open_internal_outputs and collected_direct_resealed_and_failed_open_sealing_do_not_gain_authority; Driver source_load native_open/native_theorem_artifact and snapshot publication tests; LSP server registry tests.

## Verification and exit

Freeze: independent specification/test-intent review; both lint_policy suites (including contract links/fragments/pairs/caps).
Implementation: independent sufficiency, precision, volume/scope, consistency and final assessment; cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; both lint_policy suites; cargo test -p mizar-ir -p mizar-driver -p mizar-lsp and a genuine snapshot-publication end-to-end case. CARGO_BUILD_JOBS=4.
Affected identity/currentness/retention and replay obligations run through these owner suites. Unaffected heavy proof/evidence/resolver/cache matrices defer to the normal periodic/integration gate. Full cargo test only on a named cadence trigger; none for this local first outstanding task, push disabled.
Exit: one route conversion, one enum-argument validator, no boolean selector or old validator callers; preservation matrix passes, no unresolved review finding, freeze then implementation committed locally with a clean tree. Freeze plus implementation counts as one changed task.
