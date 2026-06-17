# mizar-syntax: Trivia

Status: task-4 trivia model implemented; syntax-level item attachment fixture
landed with the first task-S-009 item-node increment.

## Purpose

This module defines syntax-adjacent trivia retained for diagnostics,
formatting, documentation, and LSP features.

`mizar-syntax` owns the syntax-facing trivia model, not comment extraction.
`mizar-frontend::PreprocessedSource` remains the owner of comment and
doc-comment extraction, raw doc-comment bodies, lexical text, and preprocess
maps. `SurfaceAst` stores only source-range-based syntactic hints: comment
ranges and kinds, doc-comment attachment targets, skipped-token ranges, and
whitespace-sensitive hints.

## Responsibilities

- store comment ranges and syntactic attachment hints without copying raw
  comment bodies;
- represent doc-comment attachment targets syntactically, with no semantic
  interpretation;
- preserve skipped token ranges and recovery ownership hints;
- preserve whitespace-sensitive hints needed by formatter and LSP consumers;
- render trivia deterministically when requested by syntax snapshots.

## Public API

### Storage Boundary

`SurfaceTrivia` is the immutable trivia side table carried by `SurfaceAst`.
It contains:

| Field | Meaning |
|---|---|
| `comments` | non-documentation or documentation comment ranges with `CommentKind`; raw text remains frontend-owned |
| `doc_comment_attachments` | syntactic attachment hints from a doc-comment source range to a node, token, or detached source anchor |
| `skipped_token_ranges` | source ranges skipped during recovery, with an optional syntactic owner and reason |
| `whitespace_hints` | source ranges where whitespace affects formatting, code actions, or token separation |

`SurfaceTriviaBuilder` builds the side table and sorts entries by source-local
range and kind before `finish`, so snapshots do not depend on construction
order. All ranges must belong to the same `SourceId` as the trivia table.

Sorting is deterministic within each table:

- comments sort by range start, range end, then `CommentKind` order
  (`SingleLine`, `MultiLine`, `Documentation`, unknown kinds last);
- doc-comment attachments sort by range start, range end, placement
  (`Leading`, then `Trailing`), then target key;
- skipped ranges sort by range start, range end, reason (`Recovery`,
  `MalformedAnnotation`, `UnexpectedToken`), then optional owner key; present
  owners sort before `None`;
- whitespace hints sort by range start, range end, then hint kind
  (`RequiresSeparation`, `LineBreakBefore`, `LineBreakAfter`,
  `SyntheticBoundary`).

Target keys sort node targets before token targets before detached anchors. Node
and token targets then sort by compatibility id index, range start, and range
end. Detached anchors sort by their rendered key prefix and local data:
generated anchors, points, ranges, then unknown anchors; generated anchors
include their generated-origin reason after the anchor range or point. Target
keys must not include raw `SourceId` debug output.

### Ownership Split

The frontend preprocessing layer extracts comments and doc comments from loaded
source, stores doc-comment bodies, removes comments from `lexical_text`, and
retains source maps. Syntax trivia references that data by `SourceRange` and
attachment target only. This keeps comment-only edit cache behavior available:
an unchanged token stream may reuse parser output while a later attachment
step can rebuild trivia hints from the frontend-owned `PreprocessedSource`.

### Incremental Reuse Boundary

The S-018 audit treats `SurfaceTrivia` as a range-attached side table, not as a
rowan identity surface. Later query layers may reuse a `SurfaceAst` green tree
for an unchanged token stream and rebuild or reattach trivia from
frontend-owned preprocessing output, provided `SurfaceAst::with_trivia`
validates that every node or token target still exists with the recorded range.
Trivia target keys, snapshot rendering, and sorting rules therefore avoid raw
rowan identity, file paths, and `SourceId` debug output. If a localized edit
changes the token stream, trivia owners must be recomputed against the new
syntax tree rather than carried forward by `SurfaceNodeId` alone.

### Attachment Targets

`TriviaAttachmentTarget` has three forms:

- `Node(TriviaNodeTarget)` for a non-token syntactic node attachment;
- `Token(TriviaNodeTarget)` for token-local leading or trailing attachment;
- `Detached(SourceAnchor)` when trivia is retained but no syntax node owns it.

`TriviaNodeTarget` stores both the compatibility `SurfaceNodeId` and the
target node's `SourceRange`. When trivia is attached to a `SurfaceAst`,
`SurfaceAst::with_trivia` verifies that each node/token target exists in that
AST, that the stored range matches the target node, that node targets do not
refer to token nodes, and that token targets refer to token nodes. Detached
source anchors must belong to the same source, including generated anchors
whose origin points back to a source range or point.

`TriviaPlacement` records whether the attachment is leading or trailing. A doc
comment attached to the following item node is a syntactic relationship; the
documentation generator may interpret the comment body later, but that meaning
does not enter `SurfaceAst`. The first item-node fixture attaches a leading doc
comment to a task-5 `PlaceholderItem`; frontend-produced attachment hints remain
a later integration step.

### Skipped Ranges

`SkippedTokenRange` records the skipped source range, an optional owner target,
and a `SkippedTokenReason` (`Recovery`, `MalformedAnnotation`, or
`UnexpectedToken`). Recovery nodes still carry their own `recovered` flag; the
trivia side table preserves the skipped source span for diagnostics,
formatters, and LSP code actions.

### Whitespace Hints

`WhitespaceHint` records range-based hints such as required token separation,
line breaks before or after a syntax element, and synthetic boundaries
introduced by preprocessing. These hints preserve formatting-sensitive facts
without making whitespace a semantic input.

### Snapshot Rendering

`SurfaceAst::snapshot_text` intentionally remains the task-3 syntax-only
baseline format. `SurfaceAst::snapshot_text_with_trivia` appends a deterministic
`trivia:` section when tests or corpus baselines need to assert trivia
ownership and attachment. The trivia snapshot renders source-local byte ranges,
kind names, attachment targets, skipped reasons, and whitespace hint kinds; it
does not render raw comment text, file paths, source-id debug output, or rowan
identity.

The current trivia snapshot section is:

```text
trivia:
  <entry-or-none>
```

Entry lines use these forms:

```text
Comment kind=<CommentKind> range=<start>..<end>
DocComment range=<start>..<end> placement=<TriviaPlacement> target=<target>
SkippedTokens reason=<SkippedTokenReason> range=<start>..<end> owner=<target-or-none>
WhitespaceHint kind=<WhitespaceHintKind> range=<start>..<end>
```

Targets render as `node:range:<start>..<end>`, `token:range:<start>..<end>`,
`detached:range:<start>..<end>`, `detached:point:<offset>`,
`detached:generated`, or `detached:unknown`. A missing node/token target renders
as `<missing>` only for defensive snapshot rendering; `SurfaceAst::with_trivia`
must reject such targets before normal snapshots are produced.

### Public Enum Compatibility

`TriviaAttachmentTarget`, `TriviaPlacement`, `SkippedTokenReason`, and
`WhitespaceHintKind` are public because parser, frontend, formatter, and LSP
layers will share trivia ownership. The S-017 final public-enum audit marks the
enums that can grow
(`TriviaAttachmentTarget`, `SkippedTokenReason`, and `WhitespaceHintKind`) as
`#[non_exhaustive]` for downstream crates, and the lint-policy gate keeps those
attributes present. `TriviaPlacement` remains deliberately exhaustive because
leading/trailing placement is a closed two-way syntactic relation; revisit that
decision only if a concrete middle/detached placement is designed. Internal
matches remain exhaustive. Any future public enum in this module must be added
to exactly one lint-policy classification before it lands.
