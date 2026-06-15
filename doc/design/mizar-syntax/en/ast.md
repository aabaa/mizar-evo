# mizar-syntax: Surface AST

Status: rowan-backed storage boundary and task-12 compatibility views implemented; full AST coverage planned.

## Purpose

This module defines the source-shaped `SurfaceAst` produced by `mizar-parser`.
`SurfaceAst` is backed by an immutable rowan green tree. The current
`SurfaceNode`/`SurfaceNodeId` surface remains as a compatibility view while the
parser and frontend migrate from the task-12 minimal tree shape.

## Responsibilities

- define `SurfaceAst`, rowan syntax kinds, compatibility syntax node ids, and
  parser-facing construction APIs;
- preserve source order, source ranges, and recovery nodes;
- represent modules, items, terms, formulas, statements, proofs, algorithms, and annotations;
- avoid resolved symbol ids, inferred types, overload resolution results, cluster facts, and proof obligations.

## Public API

### Storage Boundary

`SurfaceAst` owns a rowan green tree. Rowan is the storage backend for syntax
shape and deterministic sharing; it is not the semantic identity surface of the
compiler. Consumers should use the typed accessors on `SurfaceAst` and
`SurfaceNodeView` unless they are explicitly testing the storage boundary.
The raw rowan root is available through `SurfaceAst::rowan_root`, and the green
node through `SurfaceAst::green_node`, for infrastructure tests and carefully
documented integrations.

The task-12 compatibility data (`SurfaceNode`, `SurfaceNodeId`, `token_nodes`,
`root`, and `expression_root`) is backed by private fields inside `SurfaceAst`,
but parts of that surface remain public during migration: compatibility types,
read-only accessors, and `SurfaceNode` constructors/fields are still exported so
`mizar-parser`, `mizar-frontend`, and existing tests can assert the current
minimal shapes. This is a public compatibility API, not the storage backend and
not a stable artifact schema. New consumers should prefer `SurfaceNodeView` and
typed accessors. Compatibility ids and nodes must not be serialized as
cross-run identities, and consumers cannot mutate them independently of the
green tree.

### Syntax Kind Mapping

`SyntaxKind` is the raw rowan kind vocabulary. Node kinds currently map as:

| Surface role | Raw kind |
|---|---|
| root node | `SyntaxKind::Root` |
| compatibility token node | `SyntaxKind::Token` |
| compilation unit node | `SyntaxKind::CompilationUnit` |
| top-level item list node | `SyntaxKind::ItemList` |
| parser task-5 placeholder item node | `SyntaxKind::PlaceholderItem` |
| module path node | `SyntaxKind::ModulePath` |
| namespace path node | `SyntaxKind::NamespacePath` |
| qualified symbol node | `SyntaxKind::QualifiedSymbol` |
| path segment node | `SyntaxKind::PathSegment` |
| relative import prefix node | `SyntaxKind::RelativePrefix` |
| infix expression node | `SyntaxKind::InfixExpression` |
| recovery node | `SyntaxKind::ErrorRecovery` |

Token roles are separate raw kinds: identifier, reserved word, reserved symbol,
numeral, lexeme run, user symbol, string literal, error-recovery token, and
unknown token. The rowan tree is source-shaped: each token appears once as a
rowan token leaf in source order. Compatibility side tables may retain token
payloads for the task-12 API, but they must not cause duplicated token leaves
or repeated text in the rowan tree.

The current raw discriminants are part of the rowan boundary for this phase:

| Raw value | `SyntaxKind` | Role |
|---:|---|---|
| 0 | `Unknown` | fallback for unrecognized raw rowan kinds |
| 1 | `Root` | root node |
| 2 | `Token` | compatibility token wrapper node |
| 3 | `InfixExpression` | infix expression node |
| 4 | `ErrorRecovery` | recovery node |
| 5 | `ModulePath` | module import/export path node |
| 6 | `NamespacePath` | citation/reference namespace path node |
| 7 | `QualifiedSymbol` | namespace-qualified active user symbol node |
| 8 | `PathSegment` | single identifier or user-symbol segment wrapper |
| 9 | `RelativePrefix` | `.` / `..` import-relative prefix wrapper |
| 10 | `CompilationUnit` | module file skeleton node |
| 11 | `ItemList` | top-level item list node |
| 12 | `PlaceholderItem` | task-5 keyword-dispatched placeholder item node |
| 100 | `TokenIdentifier` | identifier token leaf |
| 101 | `TokenReservedWord` | reserved-word token leaf |
| 102 | `TokenReservedSymbol` | reserved-symbol token leaf |
| 103 | `TokenNumeral` | numeral token leaf |
| 104 | `TokenLexemeRun` | lexeme-run token leaf |
| 105 | `TokenUserSymbol` | user-symbol token leaf |
| 106 | `TokenStringLiteral` | string-literal token leaf |
| 107 | `TokenErrorRecovery` | lexer recovery token leaf |
| 108 | `TokenUnknown` | unknown token leaf |

`SyntaxKind::from_raw` maps any unknown raw value to `Unknown`.
`SyntaxKind::is_node_kind` is true only for `Root`, `Token`,
`InfixExpression`, `ErrorRecovery`, the task-S-009 shared path node kinds, and
the task-5 module skeleton node kinds listed above; `is_token_kind` is true
only for the token leaf kinds. Future raw values should be appended or assigned
into a documented reserved range so existing snapshots and rowan tests fail
loudly when the raw vocabulary changes.

### Current Surface Vocabulary

The current implemented surface node vocabulary is deliberately small:

| Public surface kind | Payload | Raw rowan node kind | Notes |
|---|---|---|---|
| `SurfaceNodeKind::Root` | none | `SyntaxKind::Root` | top-level compatibility root |
| `SurfaceNodeKind::Token(SurfaceToken)` | token kind and interned text | `SyntaxKind::Token` with one token leaf of the token raw kind | compatibility wrapper around a rowan token leaf |
| `SurfaceNodeKind::CompilationUnit` | none | `SyntaxKind::CompilationUnit` | parser task-5 module file skeleton; one `ItemList` child and no semantic module identity |
| `SurfaceNodeKind::ItemList` | none | `SyntaxKind::ItemList` | source-order list of top-level item placeholders and item-level recovery markers |
| `SurfaceNodeKind::PlaceholderItem` | none | `SyntaxKind::PlaceholderItem` | keyword-dispatched top-level item placeholder used until later tasks replace it with concrete item nodes |
| `SurfaceNodeKind::ModulePath` | none | `SyntaxKind::ModulePath` | `module_path`; optional `RelativePrefix`, first `PathSegment`, then repeated `.` token plus `PathSegment`; only this path shape may contain `RelativePrefix` |
| `SurfaceNodeKind::NamespacePath` | none | `SyntaxKind::NamespacePath` | `namespace_path`; first `PathSegment`, then repeated `.` token plus identifier `PathSegment`; relative prefixes are not allowed |
| `SurfaceNodeKind::QualifiedSymbol` | none | `SyntaxKind::QualifiedSymbol` | `qualified_symbol`; zero or more namespace identifier `PathSegment` + `.` token pairs followed by final user-symbol `PathSegment` |
| `SurfaceNodeKind::PathSegment` | none | `SyntaxKind::PathSegment` | wraps exactly one identifier or user-symbol token; role is determined by parent and token kind |
| `SurfaceNodeKind::RelativePrefix` | none | `SyntaxKind::RelativePrefix` | wraps exactly one `.` or `..` token at the start of a `ModulePath` |
| `SurfaceNodeKind::InfixExpression(SurfaceInfixOperator)` | spelling, precedence, associativity | `SyntaxKind::InfixExpression` | task-12 Pratt expression shape |
| `SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind)` | recovery kind | `SyntaxKind::ErrorRecovery` | builder-created recovery nodes are recovered |

`SurfaceTokenKind` currently maps to the token raw kinds listed above:
`Identifier`, `ReservedWord`, `ReservedSymbol`, `Numeral`, `LexemeRun`,
`UserSymbol`, `StringLiteral`, `ErrorRecovery`, and `Unknown`.
`SurfaceOperatorAssociativity` currently has `Left`, `Right`, and
`NonAssociative`.

Shared path nodes added for `mizar-parser` task 4 are syntax-only shapes. Their
node ranges run from the first token owned by the path or wrapper through the
last token owned by that node. Parent path nodes list children in source order.
Separator `.` tokens between path segments are direct children of the parent
path node rather than wrapped as `PathSegment`. These nodes do not produce
recovery nodes or trivia entries by themselves; consuming grammar tasks own
missing-path diagnostics, skipped-token trivia, and doc-comment attachment.
`SurfaceNodeView` exposes typed `as_module_path`, `as_namespace_path`,
`as_qualified_symbol`, `as_path_segment`, and `as_relative_prefix` helpers so
consumers do not need raw rowan traversal for these shared path shapes.

Module skeleton nodes added for `mizar-parser` task 5 are syntax-only shapes.
`CompilationUnit` represents the source file surface and owns exactly one
`ItemList` child. `ItemList` children are source-ordered `PlaceholderItem`
nodes and item-level recovery nodes such as `SkippedToken`. `PlaceholderItem`
wraps the source tokens consumed for one top-level item boundary, including
annotation prefixes and recovered items that are missing their terminating
semicolon. The parser must not encode import resolution, visibility semantics,
theorem validity, or symbol identity in these nodes. `SurfaceNodeView` exposes typed
`as_compilation_unit`, `as_item_list`, and `as_placeholder_item` helpers.
Leading doc-comment attachment to the following item is represented through
`SurfaceTrivia`, not by copying comment text into item nodes.

### Vocabulary Increment Contract

Node vocabulary grows only in the same change as the `mizar-parser` grammar task
that constructs the new shape. Before or with each increment, this spec must add
the implementation-facing contract for every new public syntax kind:

- the `SurfaceNodeKind` variant name and its raw `SyntaxKind` mapping;
- payload fields, if any, and whether they are parser facts or compatibility
  data;
- child roles and child order, including optional or repeated roles;
- range rules for the node and for its children, including any documented
  recovery exceptions;
- typed accessor or view helpers that consumers should use instead of raw rowan
  traversal;
- snapshot rendering text for the new kind and any escaping or sorting rules;
- recovery/trivia interaction, if the node owns skipped tokens, missing
  constructs, doc-comment attachment, or whitespace-sensitive hints.

The language grammar under `doc/spec/en/` defines what constructs exist. This
module spec defines how those constructs are represented in `SurfaceAst`.

### Builder Boundary

`SurfaceAstBuilder` is the parser-facing construction boundary. Parser code
adds tokens, ordinary nodes, and recovery nodes through builder methods, then
finishes with the root and optional expression root. Parser grammar code must
not push into a private arena, allocate rowan nodes directly, or rely on raw
rowan traversal. If grammar growth needs another tree operation, add it here
as a typed builder or accessor first.

Builder ids are local to one builder instance. A child, root, or expression-root
id from another builder is invalid. `add_node` creates ordinary structural nodes
only; token nodes must be created with `add_token` or `add_recovered_token`, and
recovery nodes with `add_recovery`. `finish` verifies that the optional root and
expression root exist and that non-root structural parents do not share child
subtrees.

During construction, parser infrastructure may inspect already-emitted builder
nodes through typed builder accessors such as `node_kind` and `node_range`.
Those accessors expose only the surface kind and source range needed for parser
composition; they do not expose the private builder arena as a storage contract.

The compatibility root may list both source-order token nodes and structural
nodes that contain those tokens, because task-12 consumers still inspect both
views. The rowan green tree remains source-shaped: when a structural child owns
the source tokens, the builder must emit those tokens once under the structural
rowan node rather than duplicating token leaves from the compatibility root
listing. Recovery nodes may keep context children outside their own insertion
range in compatibility views; those out-of-range context children are not
emitted under the recovery rowan node.

Current rowan construction deduplicates root-listed token nodes only when they
are also descendants of non-recovery structural root children. It does not
deduplicate a token that is both listed at the compatibility root and included
as an in-range child of a recovery node. Until a future builder check or rowan
emission rule covers that case, parser producers must not create recovery nodes
that wrap in-range token children also present in the root token listing. Use
out-of-range context children for missing-construct recovery, or record skipped
source spans in trivia instead of wrapping duplicated token leaves.

### Accessor Conventions

`SurfaceAst::node_view`, `root_view`, `expression_view`, and `token_views`
return typed views that expose kind, range, recovered flag, children, token
payload, infix payload, and recovery kind without requiring rowan traversal.
The compatibility `SurfaceAst::node` accessor remains available for existing
tests and migration code.

### Snapshot Rendering

`SurfaceAst::snapshot_text` returns the deterministic, human-readable surface
snapshot format used by syntax tests and later parser corpus baselines. The
format is versioned with the `surface-ast-snapshot-v1` header and renders the
root view, optional expression root, and token compatibility view in stable
stored order. Each node line includes the surface kind, source-local byte range,
`recovered` flag, and kind-specific payload needed to distinguish the current
syntax vocabulary: token kind/text, infix spelling/precedence/associativity,
or recovery kind.

Snapshot text deliberately avoids rowan pointer identity, builder ids,
`SurfaceNodeId` values, raw `SourceId` debug output, absolute paths, timings,
hash-map iteration order, and other nondeterministic data. Ranges are rendered
as byte offsets within the `SurfaceAst` source; source identity belongs to the
outer snapshot/profile record owned by `mizar-test`.

`SurfaceAst::snapshot_text_with_trivia` appends the deterministic trivia side
table described in [trivia.md](./trivia.md). The default syntax snapshot omits
that section so existing syntax-only baselines remain stable.

The current syntax snapshot format is:

```text
surface-ast-snapshot-v1
root:
  <node-or-none>
expression_root:
  <node-or-none>
token_nodes:
  <node-or-none>
```

Node lines are indented by two spaces per depth and use these current forms:

```text
Root range=<start>..<end> recovered=<bool>
Token kind=<SurfaceTokenKind> text="<escaped-text>" range=<start>..<end> recovered=<bool>
CompilationUnit range=<start>..<end> recovered=<bool>
ItemList range=<start>..<end> recovered=<bool>
PlaceholderItem range=<start>..<end> recovered=<bool>
ModulePath range=<start>..<end> recovered=<bool>
NamespacePath range=<start>..<end> recovered=<bool>
QualifiedSymbol range=<start>..<end> recovered=<bool>
PathSegment range=<start>..<end> recovered=<bool>
RelativePrefix range=<start>..<end> recovered=<bool>
InfixExpression spelling="<escaped-text>" precedence=<u8> associativity=<SurfaceOperatorAssociativity> range=<start>..<end> recovered=<bool>
ErrorRecovery kind=<SyntaxRecoveryKind> range=<start>..<end> recovered=<bool>
```

`<escaped-text>` uses Rust default character escaping so control characters,
quotes, backslashes, and non-printing characters render deterministically. Snapshot
format changes require a new header version plus updates to this spec, the
Japanese companion, and affected baseline snapshots. Update `mizar-test`
snapshot documentation only when the outer snapshot envelope or update policy
changes.

### Range Attachment

Every surface node carries a `SourceRange` from `mizar-session`. For ordinary
nodes, parent ranges contain all child ranges. Recovery nodes may violate that
containment when a zero-width insertion node keeps an opener or skipped token
as context; for example, a missing-`end` recovery node is attached at the EOF
insertion range while its child points back to the block opener.

### Identity Rules

Rowan green-node identity, rowan text ranges, and dense `SurfaceNodeId` values
are internal cache and compatibility details. They are deterministic within a
constructed `SurfaceAst`, but they are not stable artifact ids and must not be
serialized as cross-run identities. Stable consumers should key on deterministic
snapshots, content cache keys, source ids/ranges, and later semantic ids owned
by resolver/checker layers.

### Public Enum Compatibility

The current public syntax enums are not yet the long-lived resolver/LSP surface.
Before parser tasks 5-7 make them plausible downstream inputs, apply the
pre-consumer gate in [todo.md](./todo.md): enums that promise future vocabulary
growth (`SyntaxKind`, `SurfaceNodeKind`, and `SurfaceTokenKind`) are marked
`#[non_exhaustive]` for downstream crates, and the lint-policy gate keeps those
attributes present. `MizarLanguage` remains deliberately exhaustive because it
is an empty rowan marker enum rather than a downstream syntax category.
`SurfaceOperatorAssociativity` is currently a closed three-way operator property
(`Left`, `Right`, `NonAssociative`) and remains deliberately exhaustive unless a
later operator-model task designs a new associativity category. Matches inside
this crate should remain exhaustive so new variants cause local compile-time
updates; downstream crates must include wildcard fallback arms where
`#[non_exhaustive]` requires them.
