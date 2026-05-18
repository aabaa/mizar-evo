# Module: scope_skeleton

> Canonical language: English. Japanese companion: [../ja/scope_skeleton.md](../ja/scope_skeleton.md).

## Purpose

This module builds a lexical scope skeleton before full parsing.

Token disambiguation may need to know whether a scoped identifier binding overrides an active user symbol. Full parsing depends on token disambiguation, so this module performs a restricted pre-scan over raw lexer output and records only the binding ranges needed for lexical override decisions.

## Public API

Expected API direction:

```rust
pub struct ScopeSkeleton {
    pub frames: Vec<LexicalScopeFrame>,
    pub diagnostics: Vec<ScopeSkeletonDiagnostic>,
}

pub struct LexicalScopeFrame {
    pub range: SourceRange,
    pub bindings: Vec<ScopedBindingShape>,
}

pub struct ScopedBindingShape {
    pub spelling: String,
    pub introduced_at: SourceRange,
    pub kind: BindingShapeKind,
}

pub trait ScopeLexView {
    fn binding_overrides_symbol(&self, spelling: &str, position: SourcePos) -> bool;
}

pub fn build_scope_skeleton(raw: &RawTokenStream) -> ScopeSkeleton;
```

## Recognized Structure

The skeleton pre-scan recognizes only reserved-keyword-shaped structure needed to approximate lexical scopes:

- block boundaries such as `definition`, `proof`, `now`, and `end`;
- binder-introducing forms such as `let`, `for`, `reserve`, and `given`;
- comma-separated binding lists in recognized binder positions;
- labels or local names whose binding range can be approximated without parsing expressions.

It is intentionally not a parser. It may under-approximate bindings when source is malformed or when a binding form is not yet implemented.

The skeleton pre-scan must not require raw scan to split punctuation in advance. It may inspect inside `LexemeRun` spans to recognize delimiters such as `,`, `;`, and block-closing punctuation needed for binding-list recovery.

## Override Semantics

`ScopeLexView` answers only whether a spelling may be treated as a scoped identifier for lexical disambiguation at a given position.

It must not answer:

- whether the identifier is semantically defined;
- what declaration the identifier resolves to;
- what type the identifier has;
- whether a symbol use is valid;
- which overload is selected.

## Determinism

The skeleton must be deterministic for the same raw token stream.

When recovery is needed, diagnostics and recovered frames are ordered by source span. Under-approximation is preferred over inventing bindings that could change disambiguation incorrectly.

## Error Handling

Diagnostics are structural and recoverable:

- unmatched or missing `end`;
- malformed binder list;
- binder keyword followed by unsupported raw shape;
- block nesting that cannot be paired reliably.

These diagnostics do not accept or reject the program semantically; the parser and resolver later produce authoritative syntax/name diagnostics.

## Tests

Tests should cover:

- empty skeleton;
- simple `let x`-style binding;
- comma-separated binders;
- nested block ranges;
- malformed binders under-approximate rather than inventing names;
- `ScopeLexView` returns true only inside the binding range;
- deterministic output for repeated runs.
