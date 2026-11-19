# Task DOC13-USEDBY-SAFE-FLOW-PREREQUISITES: UsedBy fragments
Canonical language: English; [Japanese companion](../ja/DOC13-USEDBY-SAFE-FLOW-PREREQUISITES.md).

## Status And Ownership
Frozen bounded implementation, full tier: unlike comment composition, this consumes Doc9 reverse citations and adds a public presentation seam. Freeze and implementation count as one changed task; push:no.
[Doc Render](../../mizar-doc/en/00.crate_plan.md) owns safe fragment output; Doc9 retains reverse-citation identities, ordering, provenance and completeness, Artifact direct-by production and Diagnostics warning authority. Complete rendering remains gated.

## Authorities And Gaps
Authorities: [spec24.2 direct back-references](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks), [spec24.4 output](../../../spec/en/24.documentation_generation.md#244-html-output), [Doc9 inputs](../../mizar-doc/en/link_prerequisites.md#artifact-owned-direct-by-producer), [owner slots](../../mizar-doc/en/render.md#owner-slots-and-dom-hooks), [readiness](../../mizar-doc/en/render_prerequisites.md#ordered-readiness-gates).
Source/test gap: genuine Doc9 reverse-citation results exist but have no isolated safe-flow producer. Existing Doc10 published-input and native Driver fixtures provide lower-stage witnesses. No coverage/audit/trace or credit change.

## Frozen Scope And Decisions
Produce one safe flow fragment per selected theorem in the attached module, aligned with the original Doc9 UsedBy result order; non-theorem exports create no entries. Obtain fresh results through the existing resolver using authenticated imports, direct-by products, allocations and specification binding, with no reference occurrences. Return the unchanged result alongside fragments so all coverage, occurrence provenance, parsed/input warnings and documentation-warning promotion survive.
Never accept caller-forged UsedBy records as authority, reconstruct citations, classify references or require available comments as citation evidence. Reuse existing binding refusals for foreign module/publication, direct-by inventory, output root and specification version.
Display each distinct citing theorem once in Doc9 order, including recorded self/cross-package citations. Use its exact full FQN as escaped text and the existing validated allocation/href encoder for a link; missing destination/label readiness stays safe unlinked text, independently of citation coverage. No invented diagnostic or completeness change follows from a presentation destination failure.
Complete-empty coverage produces a visibly empty UsedBy list. Incomplete/unavailable coverage is visibly unavailable even when known citing entries remain displayed; never imply exhaustiveness. Preserve repeated direct occurrences and both independent coverage states in the returned result, without serializing or duplicating them into HTML. Completeness is relative to the selected finite closure.
Internal API naming and presentation markup are autonomous design choices. No authored HTML becomes active; no supplied warning is dropped, reinterpreted or duplicated.
Forbidden: spec/AGENTS/protocol edits, language-intent changes, supplied-fragment template expansion, discovery, new diagnostics, complete-page/search claims, Q12, Doc13/G4/G5 closure, Doc14–15, schemas/publication, Doc23/24 or credit. New semantic choices stop for Muse.

## Artifacts And Reviews
Write set: this pair, paired Doc plan navigation and render/render_prerequisites boundary updates; existing mizar-doc render.rs, mizar-test Doc tests and native mizar-driver/tests/request.rs witness. Lower-stage producers remain unchanged.
Independent specification review before freeze; test sufficiency, precision, volume/scope and final source/EN-JA consistency over the whole unit. Do not repeat passed specification review unless this contract changes.

## Acceptance Matrix
Six families, cap six; extras require a cited authority clause:
1. Complete genuine direct edges: deterministic theorem/identity order, distinct citing entries, repeated/self/cross-package occurrences retained; non-theorem, transitive, See and used_axioms-only inputs do not create citations.
2. Complete-empty versus incomplete/unavailable coverage: visibly distinct output, known edges retained, both coverage states unchanged and no exhaustive claim.
3. Allocated links use exact escaped FQN and encoded href; missing page/anchor/label readiness stays safe unlinked text without mutating coverage or warnings.
4. Existing parsed/input-scope warnings, provenance and fail-on-documentation-warnings behavior match the lower resolver exactly; repeated rendering is deterministic and input products remain unchanged.
5. Foreign module/publication, imports/direct-by/allocations/specification/output-root bindings retain existing refusals; missing comment metadata alone does not invent missing citation data.
6. Genuine native Driver publication reaches the fragment API with authoritative direct-by identities and retained source provenance.
Verification: CARGO_BUILD_JOBS=4. Freeze: both lint_policy suites (ledger/local links/fragments), git diff --check. Implementation: fmt --check; all-target/all-feature Clippy -D warnings; both policies; Doc owner tests and published_inputs corpus; short native Driver publication E2E. Full cargo test runs for this tenth changed task; no push is authorized.
Exit: commit freeze first, then reviewed implementation locally; excluded gates remain open.
