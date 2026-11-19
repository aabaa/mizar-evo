# Task DOC13-SEARCH-COMPOSITION: authenticated canonical search payload
Canonical language: English; [Japanese companion](../ja/DOC13-SEARCH-COMPOSITION.md).

## Status And Ownership
Frozen bounded implementation, full tier; freeze and implementation are one changed task; push:no.
[Doc Render](../../mizar-doc/en/00.crate_plan.md) owns composition and serialization; G1/G2 and shared Q4/MT summaries retain their authority. Consumers are the supplied version-1 search reader and future emission.
Dependencies: adopted [identity/allocation handoffs](../../mizar-doc/en/render_prerequisites.md#identity-and-allocation-handoffs), [search wire interface](../../mizar-doc/en/render_prerequisites.md#proposed-search-encoding-and-compatibility), and existing authenticated publications, destinations and shared summaries. This isolated producer is ready under the [sequential plan](../../todo.md#sequential-execution-plan).

## Authorities And Gaps
Authorities: [spec24.4 search and version](../../../spec/en/24.documentation_generation.md#244-html-output), [spec22 MT recovery](../../../spec/en/22.error_handling_and_diagnostics.md#2269-documentation-markdown-and-tag-warnings), [Q4/MT selection](../../mizar-doc/en/render.md#joint-q4mt-summary-definition), [reader compatibility](../../mizar-doc/en/render_prerequisites.md#adopted-g4-supplied-search-reader), [readiness](../../mizar-doc/en/render_prerequisites.md#ordered-readiness-gates).
Source/test gap: shared export summaries and G1/G2 exist, but no complete authenticated selected-publication search payload or canonical serializer connects them. No language intent, audit/trace/coverage or completion-credit change.

## Frozen Scope And Decisions
Revalidate the current selected plan, explicit finite publication inventory and original G1 producers using existing authentication. Require matching G2 destinations and exactly one bound parsed-comment input per selected module, including zero-export modules; refuse missing, duplicate, foreign, stale or mismatched products as a whole.
Include every selected stored Public/Reexported export once per complete identity, in full module Ord then exact FQN/origin order, publication hash last. Dependency-only items never enter; do not merge distinct duplicate FQNs. Project exact owner version, structured module components, optional lock identity, publication hash, kind, FQN and signature.
Use shared search summaries without reparsing or inventing metadata: known absence is null, available empty is empty, unavailable refuses; approved raw-block conversion/map recovery retains its degraded summary and warnings. Construct root-relative dedicated-item hrefs with the existing Q3 encoder and full FQN fragment.
Return complete in-memory canonical JSON bytes/text only: declared envelope/record order, integer format_version 1, compact UTF-8, no BOM, one final LF; quote/backslash escaping and lowercase Unicode escapes for all controls, other Unicode and slash literal. No generic Artifact JSON ordering substitution.
API names, argument grouping and bounded internal placement are autonomous design under these adopted meanings. Avoid duplicated payload representations and new single-use public types.
Forbidden: spec/AGENTS/protocol edits, language choices, new diagnostics or Q8/Q12 decisions, file emission, package overview, complete render/search emission, auxiliary producers or encounter ordinals, schemas/publication, G4/G5, Doc23/24 or completion credit. New semantics stop for Muse.

## Artifacts And Reviews
Write set: this pair; paired Doc plan/TODO navigation and render/render_prerequisites boundary updates; existing mizar-doc render.rs and narrowly required handoffs; mizar-test runner/tests/doc.rs and native mizar-driver/tests/request.rs witness. Lower-stage meanings stay unchanged.
Independent specification/EN-JA review before freeze; then test sufficiency, implementation precision, volume/scope and final source/documentation consistency over the whole unit. No repeated specification review unless the contract changes.

## Acceptance Matrix
Six families, cap six; extra families require a cited authority clause:
1. Complete selected exports across modules, undocumented and reexported items, exact strings/structured identities and dedicated hrefs; dependency-only exports excluded; complete-empty modules/package retained and serialize empty items.
2. Permuted publication/producer/comment enumeration produces identical bytes in owner order; distinct complete identities sharing FQNs survive where G2 permits; collisions retain G2 refusal.
3. Missing/duplicate/foreign summary modules, unavailable projection, changed plan/options/publication/export/destination binding and stale snapshot refuse without partial output or input mutation.
4. Known absent, attached empty and all-failed/mixed conversion/map recovery yield shared null/empty/successful text with unchanged raw blocks and warnings; no new refusal for adopted recovery.
5. Exact golden bytes cover key order, identity null/string distinctions, quote/backslash/all control escapes, literal Unicode/slash, empty display strings and one final LF; repeat generation is identical.
6. Generated payload passes the real bundled reader and preserves order/text/hrefs; native Driver publication composes through genuine authenticated producers; incompatible supplied-reader preservation checks remain in the owner group.
Verification: documentation freezes run both lint_policy suites and local links/fragments. Implementation runs fmt, Clippy, both lint_policy suites, mizar-doc and Doc owner tests/published_inputs corpus, bundled reader checks and short native Driver E2E. Changed canonical-byte/currentness matrices run immediately; unaffected proof/evidence/cache/replay heavy groups defer to periodic/integration cadence. Full cargo test only on a named cadence trigger.
Exit: commit freeze then reviewed implementation locally; excluded gates remain open.
