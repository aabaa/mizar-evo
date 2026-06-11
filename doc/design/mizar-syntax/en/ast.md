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

The task-12 `SurfaceNode` vector, `SurfaceNodeId`, `token_nodes`, `root`, and
`expression_root` are private compatibility side tables exposed through typed
accessors over the rowan-backed AST. They are kept so `mizar-parser` and
`mizar-frontend` can continue to assert the current minimal shapes while later
node vocabulary tasks move consumers to typed views. They must not be treated
as the storage backend or as stable artifact ids, and consumers cannot mutate
them independently of the green tree.

### Syntax Kind Mapping

`SyntaxKind` is the raw rowan kind vocabulary. Node kinds currently map as:

| Surface role | Raw kind |
|---|---|
| root node | `SyntaxKind::Root` |
| compatibility token node | `SyntaxKind::Token` |
| infix expression node | `SyntaxKind::InfixExpression` |
| recovery node | `SyntaxKind::ErrorRecovery` |

Token roles are separate raw kinds: identifier, reserved word, reserved symbol,
numeral, lexeme run, user symbol, string literal, error-recovery token, and
unknown token. The rowan tree is source-shaped: each token appears once as a
rowan token leaf in source order. Compatibility side tables may retain token
payloads for the task-12 API, but they must not cause duplicated token leaves
or repeated text in the rowan tree.

### Builder Boundary

`SurfaceAstBuilder` is the parser-facing construction boundary. Parser code
adds tokens, ordinary nodes, and recovery nodes through builder methods, then
finishes with the root and optional expression root. Parser grammar code must
not push into a private arena, allocate rowan nodes directly, or rely on raw
rowan traversal. If grammar growth needs another tree operation, add it here
as a typed builder or accessor first.

### Accessor Conventions

`SurfaceAst::node_view`, `root_view`, `expression_view`, and `token_views`
return typed views that expose kind, range, recovered flag, children, token
payload, infix payload, and recovery kind without requiring rowan traversal.
The compatibility `SurfaceAst::node` accessor remains available for existing
tests and migration code.

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
