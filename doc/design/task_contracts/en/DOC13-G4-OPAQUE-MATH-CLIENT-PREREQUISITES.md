# Task DOC13-G4-OPAQUE-MATH-CLIENT-PREREQUISITES: bounded G4 client math
Canonical language: English; [Japanese companion](../ja/DOC13-G4-OPAQUE-MATH-CLIENT-PREREQUISITES.md).

## Status And Ownership
Frozen prerequisite, full tier: new client execution and asset-integrity boundary. Freeze plus implementation is one changed task.
Purpose: discharge only opaque-occurrence portions of Doc11 Q6/Q7/Q12 and Doc13 Q6/Q11 within [G4](../../mizar-doc/en/render_prerequisites.md#ordered-readiness-gates).
Primary owner: [mizar-doc](../../mizar-doc/en/00.crate_plan.md). Math owns runtime/configuration/errors/reproducibility; render owns transport/DOM/local assets. Diagnostics owns all shared build-diagnostic allocation and emission.
Dependencies: adopted [Doc8](../../mizar-doc/en/commonmark.md), [G3 coordinates](../../mizar-doc/en/render_prerequisites.md#adopted-g3-source-coordinates), local immutable upstream KaTeX distribution and pinned browser.

## Authority And Gaps
Authority: [spec24.3](../../../spec/en/24.documentation_generation.md#243-mathematical-formula-rendering), [spec24.4](../../../spec/en/24.documentation_generation.md#244-html-output), [Doc11](../../mizar-doc/en/math.md#adopted-opaque-client-runtime) and [Doc13](../../mizar-doc/en/render.md#adopted-opaque-math-transport).
Design/source/test gap: real Doc8 ordered segmented payloads exist, but safe client transport, runtime, fallback and reproducible local assets do not.
Existing tests: Doc8 math/provenance in `mizar-test/src/runner/tests/doc8_math.rs`; native commented publication in Driver `tests/request.rs`.

## Frozen Scope
Borrow real ParsedComments/MathRange and original attachments; preserve block/occurrence ordinals, classification, exact joined interior UTF-8 bytes and independent provenance segments. No reparse, normalization, delimiter insertion or fabricated coordinates.
Implement only isolated HTML fragments and local-asset handoff defined by the owner sections. No page renderer, placeholder site or publication transaction.
Pin one KaTeX release, matching JS/CSS/all referenced fonts, license, upstream provenance and digests; authenticate complete assets/configuration before use. No runtime CDN or mutable downloads.
Client execution uses fresh macro state and detached output for every occurrence, disabled trust, finite expansion/size/payload limits and accessible escaped-payload fallback; neighboring occurrences continue. No build-time TeX validation or invalid-TeX build diagnostic.
Internal API/names, error wording and validation order are autonomous presentation choices. Existing proof/extraction acceptance and lower-reader errors remain unchanged.
Forbidden: spec/test-intent changes, Q12 descriptors/locations/precedence/profiles, complete templates, symbolic math/annotations/numbering/references, remaining Doc11 dependencies, G5/coherent Doc13, schemas/publication/Doc23/24, audit/trace/coverage/owner/milestone credit, AGENTS/protocol edits.

## Artifacts And Reviews
Write set: this pair; paired math/render/render-prerequisite boundaries and crate-plan links; bounded Doc math/render modules, bundled assets/configuration/manifest; Doc and mizar-test owner tests and browser witness harness. No diagnostic-owner code changes.
Reviews: independent specification before freeze; test sufficiency, precision, volume/scope, final source/EN-JA consistency and scoped quality assessment. Frozen specification review repeats only after contract changes.

## Acceptance Matrix
Six families, cap six; extra families require cited authority:
1. Real Doc8 inline/display, repeated/empty and container-discontiguous occurrences retain payload, source order, target/publication and independent mapped provenance.
2. Hostile HTML/script/attribute spelling, entities, Unicode, controls and CR/LF round-trip exactly through transport; executable markup is never derived from payload.
3. Actual pinned KaTeX success, invalid/unsupported-TeX failure, accessible escaped-payload fallback and subsequent success; absent runtime retains safe fallback. Fake runtime alone is insufficient.
4. Trusted HTML/URL commands stay disabled; global macro definitions cannot affect neighbors; expansion, size and client payload limits apply without build-time validation.
5. Missing/corrupt/mismatched runtime/CSS/fonts/configuration refuse; assets load locally only, including nested-page use, without external requests.
6. Equal bound inputs yield identical transport/asset bytes and equivalent success/failure/DOM in the pinned Linux x86_64 Chrome environment; browser pixel equivalence is excluded.
Verification: CARGO_BUILD_JOBS=4; freeze runs both lint_policy suites and links/fragments. Implementation runs fmt, all-target/all-feature Clippy -D warnings, both policies, Doc owner/corpus, genuine Doc8 transport and real-browser witnesses plus native short E2E. Full cargo test is required for requested push and inherited outstanding range.
Exit: reviewed bounded handoff committed after freeze and pushed after full success; excluded gates remain open.
