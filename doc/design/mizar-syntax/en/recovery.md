# mizar-syntax: Recovery Nodes

Status: minimal task-12 recovery nodes implemented; full recovery vocabulary planned.

## Purpose

This module defines the syntax representation of parser recovery.

## Responsibilities

- represent missing constructs, skipped tokens, unmatched delimiters, and malformed annotations;
- mark recovered nodes so resolver and checker phases can skip or reject them explicitly;
- preserve original source spans for diagnostics.

Current minimal vocabulary includes recovered token nodes for lexer error tokens
and explicit recovered nodes for missing `end` and missing string literals.
Broader skipped-token, unmatched delimiter, and malformed-annotation recovery
remains planned.

## Public API

### Syntax Diagnostics

`SyntaxDiagnostic` is the parser-facing diagnostic record carried alongside an
optional `SurfaceAst`.

| Field | Contract |
|---|---|
| `code` | stable syntax diagnostic category for orchestration and tests |
| `message` | human-readable parser diagnostic text; not a stable machine key |
| `primary` | source range that should receive the main diagnostic highlight |
| `secondary` | optional source anchors for opener/context/candidate spans |
| `recovery_note` | optional short explanation of the recovery action that let parsing continue |

`SyntaxDiagnostic::new` creates a diagnostic with no secondary anchors and no
recovery note. `with_secondary` appends secondary anchors without replacing
existing ones. `with_recovery_note` records parser recovery advice or action
text; diagnostics that abort parsing rather than recover may leave it unset.

Current `SyntaxDiagnosticCode` values are:

| Code | Producer condition | Recovery note expectation |
|---|---|---|
| `UnexpectedErrorToken` | parser receives a lexer-owned error-recovery token | optional; the recovered token itself preserves the lexer input |
| `DanglingOperator` | Pratt expression parsing finds an operator without the required operand | optional; no recovery node is required |
| `NonAssociativeOperatorChain` | parser sees a chain that violates a non-associative operator contract | optional; no recovery node is required |
| `MissingEnd` | parser inserts a missing `end` at a synchronization point | set when parsing continues after insertion |
| `MissingStringLiteral` | parser inserts a missing string literal in a string-required context | set when parsing continues after insertion |
| `UnrecoverableInput` | parser cannot construct a trustworthy `SurfaceAst` for the input | optional; set when the parser can suggest a source edit, and the parse result may have `ast = None` |

The diagnostic code vocabulary is syntax-level only. It must not encode name
resolution, type checking, proof obligations, or semantic facts.

### Current Recovery Vocabulary

`SyntaxRecoveryKind` currently has the task-12 minimum vocabulary:

| Kind | Producer meaning | Node shape | Diagnostic code |
|---|---|---|---|
| `ErrorToken` | a lexer error-recovery token was preserved in the syntax stream | recovered token node with `SurfaceTokenKind::ErrorRecovery`, or a recovery node when a parser task needs an explicit wrapper | `SyntaxDiagnosticCode::UnexpectedErrorToken` |
| `MissingEnd` | parser inserted a missing `end` at a synchronization point | `SurfaceNodeKind::ErrorRecovery(MissingEnd)` with a zero-width insertion range and optional opener/context child | `SyntaxDiagnosticCode::MissingEnd` |
| `MissingStringLiteral` | parser inserted a missing string literal in a string-required context | `SurfaceNodeKind::ErrorRecovery(MissingStringLiteral)` with a zero-width insertion range and no required children | `SyntaxDiagnosticCode::MissingStringLiteral` |

Recovered nodes must have `recovered = true`. A recovered token preserves the
original token text and source range so diagnostics, formatter recovery, and LSP
features can show the user's input instead of invented text.

### Range And Child Rules

Recovery ranges are source-local byte ranges in the same source as the
`SurfaceAst`.

- Inserted missing constructs use a zero-width range at the insertion point.
- Recovered lexer tokens use the original token range.
- Recovery nodes may keep context children outside their own range. The
  compatibility view retains those children for diagnostics and parser tests,
  while the rowan green tree omits out-of-range recovery children to remain
  source-shaped.
- Ordinary non-recovery nodes should still contain all child ranges. Any future
  recovery exception must be documented next to the recovery kind that needs it.

### Recovery And Trivia Split

Recovery nodes describe the syntactic placeholder or marker that parser
consumers must notice. `SurfaceTrivia::skipped_token_ranges` describes skipped
source spans and optional owners for diagnostics, formatting, and code actions.
When a recovery strategy both inserts a placeholder and skips source text, the
placeholder belongs in `SurfaceNodeKind::ErrorRecovery`, while the skipped span
belongs in trivia. Do not encode raw skipped text in the recovery node.

### Expansion Contract

Expanding recovery vocabulary is task 5 in [todo.md](./todo.md). Each new
`SyntaxRecoveryKind` must specify:

- the parser condition that produces it;
- whether the node is an inserted placeholder, a wrapper around source text, or
  a marker for skipped input;
- range and child-role rules, including whether out-of-range context children
  are allowed;
- the corresponding `SyntaxDiagnosticCode` or reason for sharing an existing
  code;
- whether skipped source spans are also recorded as `SkippedTokenRange`;
- snapshot rendering name and at least one constructible test fixture.

Planned categories are:

| Category | Required specification before implementation |
|---|---|
| missing constructs | one kind per construct family that downstream phases need to distinguish; insertion range and context-child role |
| skipped tokens | owner selection, skipped range ownership, and interaction with `SkippedTokenReason::Recovery` |
| unmatched delimiters | opener/closer context roles and primary diagnostic anchor |
| malformed annotations | annotation range ownership and interaction with `SkippedTokenReason::MalformedAnnotation` |

### Public Enum Compatibility

`SyntaxRecoveryKind` and `SyntaxDiagnosticCode` promise future variants as
grammar recovery grows. The pre-consumer gate in [todo.md](./todo.md) should mark
them `#[non_exhaustive]` for downstream crates unless a later task records a
deliberate exhaustive decision. Matches inside `mizar-syntax` and
`mizar-parser` should remain exhaustive so adding a recovery kind forces local
snapshot and diagnostic updates.
