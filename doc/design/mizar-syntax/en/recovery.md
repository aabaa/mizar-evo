# mizar-syntax: Recovery Nodes

Status: recovery vocabulary for missing constructs, skipped tokens, unmatched delimiters, and malformed annotations implemented in `mizar-syntax`; parser production remains incremental.

## Purpose

This module defines the syntax representation of parser recovery.

## Responsibilities

- represent missing constructs, skipped tokens, unmatched delimiters, and malformed annotations;
- mark recovered nodes so resolver and checker phases can skip or reject them explicitly;
- preserve original source spans for diagnostics.

The parser currently produces recovered token nodes for lexer error tokens and
explicit recovered nodes for missing `end`, missing string literals, and task-5
top-level skipped tokens. The remaining recovery kinds are constructible in
`mizar-syntax` so future parser grammar tasks can add producers without
changing the syntax snapshot vocabulary.

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
| `MissingSemicolon` | parser reaches a top-level item boundary or EOF where `;` is required | set when parsing continues with the next item or EOF |
| `MissingStringLiteral` | parser inserts a missing string literal in a string-required context | set when parsing continues after insertion |
| `MalformedImport` | parser task 6 finds import-internal syntax that can continue at the current import statement boundary | set when the import item remains represented, for example missing alias after `as` or missing `}` in a branch import |
| `MalformedExport` | parser task 7 finds export-internal syntax that can continue at the current export statement boundary | set when the export item remains represented, for example missing module path after `export` or after `,` |
| `MalformedVisibility` | parser task 7 finds a duplicate, dangling, or invalid top-level visibility marker | set when the visible item wrapper remains represented; malformed tail tokens are skipped inside it when present |
| `UnexpectedTopLevelToken` | parser task 5 skips source tokens that cannot start a top-level item | set when a `SkippedToken` recovery node and skipped trivia range are emitted |
| `UnrecoverableInput` | parser cannot construct a trustworthy `SurfaceAst` for the input | optional; set when the parser can suggest a source edit, and the parse result may have `ast = None` |

The diagnostic code vocabulary is syntax-level only. It must not encode name
resolution, type checking, proof obligations, or semantic facts.

### Recovery Vocabulary

`SyntaxRecoveryKind` covers the recovery categories promised for the
pre-consumer syntax phase. Kinds marked "not produced yet" are mizar-syntax
vocabulary only until the paired parser grammar task documents and implements a
producer.

| Kind | Producer condition | Node shape | Range and child rule | Diagnostic/trivia split | Snapshot name |
|---|---|---|---|---|---|
| `ErrorToken` | parser receives a lexer-owned error-recovery token | recovered token node with `SurfaceTokenKind::ErrorRecovery`, or `SurfaceNodeKind::ErrorRecovery(ErrorToken)` when a parser task needs an explicit wrapper | token form uses the original token range; wrapper form uses the same source range and no required children | `SyntaxDiagnosticCode::UnexpectedErrorToken`; raw token text remains on the recovered token, not in trivia | `ErrorToken` |
| `MissingEnd` | parser inserts a missing `end` at a block synchronization point | `SurfaceNodeKind::ErrorRecovery(MissingEnd)` inserted placeholder | zero-width range at insertion point; may keep the block opener/context child outside the insertion range | `SyntaxDiagnosticCode::MissingEnd`; no skipped range unless the same recovery also skips source text | `MissingEnd` |
| `MissingStringLiteral` | parser inserts a missing string literal in a string-required context | inserted placeholder | zero-width range at insertion point; no required children | `SyntaxDiagnosticCode::MissingStringLiteral`; no skipped range | `MissingStringLiteral` |
| `MissingItem` | not produced yet; future module/item parser expects a top-level item and synchronizes before the next item boundary or EOF | inserted placeholder | zero-width range at insertion point; optional context child for the synchronization token or containing item list, allowed outside the insertion range | no dedicated diagnostic code yet; the producer task must add or explicitly share a code before emitting user-facing diagnostics; skipped source belongs to `SkippedTokenRange` when present | `MissingItem` |
| `MissingTypeExpression` | not produced yet; future type parser expects a type expression after a keyword such as `of`, `over`, `mode`, or a declaration binder | inserted placeholder | zero-width range at insertion point; optional keyword/binder context child, allowed outside the insertion range | no dedicated diagnostic code yet; no skipped range for pure insertion | `MissingTypeExpression` |
| `MissingTerm` | not produced yet; future term parser expects a term operand, argument, selector base, or constructor field value | inserted placeholder | zero-width range at insertion point; optional operator/call/context child, allowed outside the insertion range | no dedicated diagnostic code yet; no skipped range for pure insertion | `MissingTerm` |
| `MissingFormula` | not produced yet; future formula parser expects a formula after logical syntax such as `st`, `holds`, a connective, or a theorem/proof keyword | inserted placeholder | zero-width range at insertion point; optional keyword/operator context child, allowed outside the insertion range | no dedicated diagnostic code yet; no skipped range for pure insertion | `MissingFormula` |
| `MissingStatement` | not produced yet; future statement parser expects a proof, algorithm, or block statement and synchronizes at the next statement boundary | inserted placeholder | zero-width range at insertion point; optional preceding keyword or block context child, allowed outside the insertion range | no dedicated diagnostic code yet; skipped source belongs to `SkippedTokenRange` when present | `MissingStatement` |
| `MissingProofStep` | not produced yet; future proof parser expects a justification, inference step, case branch, or proof-closing step | inserted placeholder | zero-width range at insertion point; optional proof/block context child, allowed outside the insertion range | no dedicated diagnostic code yet; skipped source belongs to `SkippedTokenRange` when present | `MissingProofStep` |
| `MissingAnnotationArgument` | not produced yet; future annotation parser expects an annotation argument such as a string literal or bracket argument | inserted placeholder | zero-width range at insertion point; optional annotation marker/list context child, allowed outside the insertion range | no dedicated diagnostic code yet; malformed or skipped source belongs to `SkippedTokenRange` with `MalformedAnnotation` or `Recovery` as appropriate | `MissingAnnotationArgument` |
| `SkippedToken` | parser task 5 skips one or more top-level tokens to reach an item boundary; future parser tasks may use the same marker at narrower grammar boundaries | marker for skipped input | range covers the skipped source span; no required children; optional synchronization owner child may be attached when it does not duplicate root-listed token leaves | `SyntaxDiagnosticCode::UnexpectedTopLevelToken` for task-5 top-level skips; the skipped span must also be recorded in `SurfaceTrivia::skipped_token_ranges` with `SkippedTokenReason::Recovery` | `SkippedToken` |
| `UnmatchedOpeningDelimiter` | not produced yet; future parser sees an opener with no matching closer before synchronization or EOF | marker, usually paired with an inserted missing close | primary marker range is zero-width at the expected closer or synchronization point; opener/context child is expected and may be outside the marker range | no dedicated diagnostic code yet; opener span should be a secondary diagnostic anchor; skipped text, if any, belongs to trivia | `UnmatchedOpeningDelimiter` |
| `UnmatchedClosingDelimiter` | not produced yet; future parser sees a closing delimiter with no matching opener | marker around source text | range covers the unmatched closer token; no required children | no dedicated diagnostic code yet; the closer token remains in the token stream, and skipped tokens beyond it belong to trivia | `UnmatchedClosingDelimiter` |
| `MalformedAnnotation` | not produced yet; future annotation parser recognizes an annotation marker or body that cannot be parsed as a valid annotation | marker around source text | range covers the malformed annotation marker/body span; optional annotation owner child may be attached when available | no dedicated diagnostic code yet; malformed source must also be recorded in `SurfaceTrivia::skipped_token_ranges` with `SkippedTokenReason::MalformedAnnotation` | `MalformedAnnotation` |

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
belongs in trivia. Do not encode raw skipped text as a string payload in the
recovery node. A grammar task may additionally attach the skipped token nodes
as in-range recovery children when it documents that ownership and nests the
recovery under a non-recovery structural owner, so rowan rendering can keep the
source tokens emitted once while trivia still records the skipped span.

Parser tasks that start producing a currently unproduced recovery kind must
update this table if they refine the producer condition, add a dedicated
`SyntaxDiagnosticCode`, or require a more specific trivia ownership rule.

### Public Enum Compatibility

`SyntaxRecoveryKind` and `SyntaxDiagnosticCode` promise future variants as
grammar recovery grows. The pre-consumer gate in [todo.md](./todo.md) marks them
`#[non_exhaustive]` for downstream crates, and the lint-policy gate keeps those
attributes present unless a later task records a deliberate exhaustive decision.
Matches inside `mizar-syntax` should remain exhaustive so adding a recovery kind
forces local snapshot and diagnostic updates; downstream crates, including
`mizar-parser`, must include wildcard fallback arms where `#[non_exhaustive]`
requires them.
