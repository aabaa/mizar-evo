# Task STEP6A2-PACKAGE-SOURCE-CODEC: package source publication payload

Canonical language: English; [Japanese pointer](../ja/STEP6A2-PACKAGE-SOURCE-CODEC.md).

Status: implemented; full tier. Primary owner: mizar-frontend.
Purpose: provide reconstructible canonical SourceUnit storage bytes.
Consumers: the forthcoming SourceLoad phase service and IR blob storage.
Dependencies: session source loader/maps/identity and IR canonical storage; ready.
Owner plan: [frontend](../../mizar-frontend/en/00.crate_plan.md).

## Authority and inventory

- [Source loading](../../architecture/en/02.source_and_frontend.md#step-1-load-sourceunit),
  [source identity](../../mizar-session/en/ids.md),
  [source owner](../../mizar-frontend/en/source.md),
  [IR storage](../../mizar-ir/en/storage.md).
- `external_dependency_gap`: SourceUnit has no reconstructible canonical payload;
  existing real session loading and frontend projection remain authoritative.
- Existing tests: inline source loader/map tests in frontend `src/source.rs`.
- No semantic, diagnostic, or coverage-audit status changes are authorized.

## Frozen scope

- Add encode/decode methods on existing SourceUnit for disk/package sources only,
  with a versioned, length-framed canonical payload; no new public wrapper type.
- Preserve package/module/path/edition, normalized text/hash and optional loading
  map segments. Exclude allocator-local SourceId and diagnostic filesystem path.
- Decode with a supplied current SourceId and caller-validated disk SourceInput; compare
  stable metadata, use the request's diagnostic path and session map constructors.
- Require Disk origin, no anchor, and DiskBytes map origin matching source path.
- Reject unsupported origins/anchors, invalid format, inconsistent source/map
  metadata, truncation, invalid UTF-8, trailing data and incompatible requests.
- These are storage bytes, not semantic hash input: publication must keep maps
  in the side-table hash; that IR integration is a subsequent dependency.
- Never replay loading/parsing or capture the original payload in a decoder.
- Codec errors are local rejection, not new public language diagnostics or cache
  acceptance. Session owns normalization, hashing and identity allocation.
- Update only frontend source, inline tests, paired source/API owner documents,
  plan indexes, this contract pair and the Step 6 sequencing entry.
- No phase adapter, parser/AST codec, resolver/checker changes, artifact task 17,
  Step 7, MVM execution, new dependencies or existing .miz/expectation edits.

## Acceptance

Test real disk loads with Unicode, identity and BOM/CRLF normalization; roundtrip
maps and text, rebind identities/paths, and demonstrate stable bytes across them.
Test malformed/incompatible inputs and unsupported origins fail closed.
Independent specification, test-sufficiency, implementation, volume/scope and
source/documentation reviews; protocol quality evaluation before commit.
Run frontend tests, `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`,
and `git diff --check`. Exit: disk SourceUnit reconstructs without source replay;
producer readiness and end-to-end equivalence remain separate follow-up work.
