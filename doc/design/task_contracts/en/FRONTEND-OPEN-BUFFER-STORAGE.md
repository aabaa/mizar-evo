# Task FRONTEND-OPEN-BUFFER-STORAGE: native editor source storage
Canonical language: English; [Japanese pointer](../ja/FRONTEND-OPEN-BUFFER-STORAGE.md).
Status: complete. Tier: full. Owners: [frontend plan](../../mizar-frontend/en/00.crate_plan.md), [session plan](../../mizar-session/en/00.crate_plan.md).
Scope: frontend task 30 and session task 34; prerequisite for LSP 6–7.
Consumers: Driver storage decoders; genuine editor SourceLoad/Frontend integration follows separately.
Authority: [source codec](../../mizar-frontend/en/source.md#canonical-openbuffer-payload), [aggregate codec](../../mizar-frontend/en/orchestration.md#openbuffer-frontendoutput-storage), [native normalization](../../mizar-session/en/source.md#pure-source-text-normalization); architecture 02/11/12, internal 03/06.
Dependencies: native loader, SESSION-SOURCE-CAPTURE, existing disk/nested codecs and real frontend.
Gap: real OpenBuffer inputs run through the frontend but have no lossless retained payload format.
No language-specification gap or changed diagnostic, parser, acceptance or cache policy is authorized.

Add to SourceUnit and FrontendOutput<SurfaceAst>:
`canonical_open_buffer_bytes(&self, input: &SourceInput) -> Option<Vec<u8>>`;
`from_canonical_open_buffer_bytes(bytes: &[u8], source_id: SourceId, input: &SourceInput) -> Option<Self>`;
`from_canonical_storage_bytes(bytes: &[u8], source_id: SourceId, input: &SourceInput) -> Option<Self>`.
Keep disk API signatures, wire bytes, origin restrictions and rejection behavior.
Evolve existing encoding bodies into shared origin-aware routines; their real callers
are Disk encoding, OpenBuffer encoding and exact decoder re-encoding. Shared public
storage decoding serves origin-specific wrappers and the actual aggregate/Driver consumer.
Change only the existing Driver Frontend blob decoder call to storage decoding;
its genuine Disk input binding, guards, error and publication behavior remain unchanged.
Expose/re-export existing `normalize_source_text(&str) -> (String, Option<Vec<LoadingMapSegment>>)`;
remove its obsolete private result struct. Native Disk/Open acquisition and codecs
share this pure operation; generated loading, normalization and validation policy stay unchanged.

Use separate OpenBuffer v1 domains and exact owner-defined field order/framing.
Preserve nonnegative matching Native i64 versions, including values above i32::MAX.
Require captured caller metadata/raw text, exact URI/version and normalized text/map/hash;
SourceInput is not a validation token. No filesystem/loader/parser/provider runs on decode.
Rebind all typed identities to the supplied SourceId; do not allocate provisional ids.
No fake Disk projection, duplicate normalizer, new DTO/adapter/Rust file, or new dependency.
No Driver OpenBuffer transport/service, InternalOnly publication, LSP snapshot/currentness,
artifact, semantic producer, acceptance, diagnostic adoption, cache hit or milestone completion.

Tests extend existing source/orchestration fixtures with genuinely loaded buffers and real frontend outputs:
lossless maps/text/metadata/diagnostics/AST, identity rebinding, untransformed/Unicode/BOM/CRLF/lone-CR/empty
inputs, i64 boundaries, stale/negative versions and altered URI/raw input/context/maps/hash;
strict domains/framing/truncation/trailing/noncanonical nested bytes and aggregate cap.
Retain every existing Disk and unsupported-origin test; decoding survives changed/deleted disk files.
Affected source: session source/lib, frontend source/orchestration, one Driver decoder call;
paired owner plans/TODOs/source/orchestration, driver adapter and audit ownership links.
Coverage impact: infrastructure owner addition only; language trace and active coverage unchanged.
Require independent specification, test-sufficiency, implementation, volume/scope and consistency reviews;
parent full cargo fmt --check, cargo clippy --all-targets --all-features -- -D warnings, cargo test.
Exit: strict native retained codecs; whole LSP 6–7, Steps 6–8 and Goal remain open.
