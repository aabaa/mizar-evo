# Structured Fix Suggestions

> Canonical language: English. Japanese companion:
> [../ja/fix.md](../ja/fix.md).

## Purpose

This document specifies structured fix suggestions owned by
`mizar-diagnostics`. A fix suggestion is a bounded, machine-readable projection
attached to a `DiagnosticRecord`. It can describe a possible source edit or an
opaque command reference, but it never applies edits by itself.

Fix suggestions are keyed by stable suggestion identity and `DiagnosticCode`.
Human titles, `help:` text, localized wording, and CLI rendering are not
identity.

## Scope

The fix module owns:

- stable `FixSuggestionId` values referenced by diagnostic records;
- structured fix payloads with title, applicability, safety classification, and
  edit list;
- compiler-native source ranges for text edits;
- optional expected text and snapshot/hash preconditions for safe validation;
- deterministic ordering and debug snapshots for fix payloads;
- projection data that CLI rendering and LSP code-action conversion may read.

The fix module does not own:

- automatic edit application or workspace mutation;
- LSP `CodeAction` or `WorkspaceEdit` protocol objects;
- current-buffer validation or command execution;
- source loading, line maps, path normalization, or UTF-16 conversion;
- proof acceptance, phase status, kernel acceptance, driver orchestration, or
  artifact mutation.

## Data Model

Task 13 stores structured fix payloads on diagnostic records in the shape
equivalent to:

```rust
struct FixSuggestion {
    id: FixSuggestionId,
    producer_key: Option<String>,
    title: String,
    applicability: FixApplicability,
    safety: FixSafety,
    edits: Vec<FixEdit>,
    command: Option<FixCommandRef>,
    required_snapshot: Option<BuildSnapshotId>,
    required_text_hash: Option<Hash>,
}

struct FixEdit {
    range: SourceRange,
    replacement: String,
    expected_text: Option<String>,
}
```

`FixSuggestionId` is stable within the diagnostic draft or record and is
assigned before aggregation from an explicit producer key or deterministic
producer-local ordinal. It must not depend on `DiagnosticHandle`, because
handles are assigned only after aggregation and deduplication. Published
records may project the containing `DiagnosticHandle` next to the fix payload
for convenience, but that back-reference is not part of pre-publication fix
identity.

`producer_key` is an optional structured identity string supplied by the
producer when a deterministic ordinal is not descriptive enough. It is not
human-facing text. Durable consumers must key on `DiagnosticCode`, structured
diagnostic detail fields, and fix identity strings, not on title or rendered
wording.

`title` is human-facing. It may change and must not be used as identity.
`command` is an opaque reference for a later owner; it must not execute inside
`mizar-diagnostics`.

## Applicability And Safety

Initial applicability levels:

| Applicability | Meaning |
|---|---|
| `MachineApplicable` | The edit is expected to be mechanically correct when all preconditions still hold. |
| `MaybeIncorrect` | The edit is plausible but should be reviewed. |
| `HasPlaceholders` | The replacement contains placeholders the user must fill. |
| `Informational` | The suggestion has no direct edit and is rendered as help text. |

Initial safety classes:

| Safety | Rule |
|---|---|
| `LocalTextEdit` | All edits target explicit source ranges and expected text may be validated against current source text. |
| `SnapshotBound` | The suggestion is valid only for `required_snapshot`. |
| `ArtifactAssisted` | The suggestion depends on artifact/source hashes checked by an external owner. |
| `CommandOnly` | No text edit is provided; a later command owner may interpret the command ref. |

No applicability level authorizes automatic application by this crate. Even
`MachineApplicable` means "safe to offer when preconditions hold," not "apply
without confirmation."

## Edit Rules

Each `FixEdit` must:

- use compiler-native `SourceRange` with `SourceId`, start, and end byte
  offsets;
- validate `start <= end`;
- carry replacement text as UTF-8;
- optionally carry `expected_text` for current-buffer validation;
- avoid overlapping another edit in the same suggestion unless a later spec
  defines a deterministic merge rule.

Task 13 should reject a suggestion whose edit ranges overlap within one
`SourceId`. Multiple edits are ordered by source key, start, end, and
replacement text for deterministic snapshots.

The fix module does not convert byte ranges to line/column or LSP UTF-16
coordinates. CLI rendering may derive `help:` text from the payload; LSP
conversion remains owned by `mizar-lsp`.

## Attachment To Records

Fix suggestions attach to `DiagnosticRecord` values as compact structured
payloads or stable handles. They must preserve:

- the diagnostic code and handle they belong to;
- the stable suggestion id;
- applicability and safety;
- edit ranges and replacement text;
- expected text and snapshot/hash preconditions.

Aggregation deduplication must include the canonical fix payload in diagnostic
identity. The canonical fix payload consists of `FixSuggestionId`,
`producer_key`, applicability, safety, ordered edits including `expected_text`,
the optional command reference, and snapshot/hash preconditions. Diagnostics
must remain distinct when any canonical fix payload field differs. Human titles,
message text, rendered `help:` lines, and localized wording must not be the
deduplication key.

## Debug Snapshot

Task 13 should expose deterministic fix debug snapshots with:

1. `kind=fix`.
2. `id`.
3. `producer_key`.
4. `diagnostic`.
5. `title`, escaped with Rust debug-string escaping.
6. `applicability`.
7. `safety`.
8. ordered edits.
9. optional command.
10. snapshot/hash preconditions.

Snapshots are test/debug data, not CLI rendering and not LSP code actions. They
must not include memory addresses, map iteration order, localized field names,
or process-local ordering.

## Public Enum Compatibility

Task 18 marks fix-owned public enums as `#[non_exhaustive]` for downstream
forward compatibility:

- `FixApplicability`;
- `FixSafety`;
- `FixSuggestionError`.

Future applicability or safety variants must keep the no-auto-apply boundary and
must document their preconditions before consumers expose them.

## Boundary Rules

- A fix suggestion is advisory. It never mutates source text or artifacts.
- `mizar-diagnostics` may validate payload shape and deterministic ordering, but
  it does not validate the current editor buffer.
- CLI rendering may show help text derived from fixes; it does not apply them.
- LSP code-action conversion, current-buffer revalidation, and command
  execution belong to `mizar-lsp` or the driver layer.
- Proof/kernel/trusted acceptance cannot depend on whether a fix suggestion is
  present or accepted by a user.
