# Module: scope_skeleton

> Canonical language: English. Japanese companion: [../ja/scope_skeleton.md](../ja/scope_skeleton.md).

## Purpose

This module builds a lexical scope skeleton before full parsing.

Token disambiguation may need to know whether a scoped identifier binding overrides an active user symbol. Full parsing depends on token disambiguation, so this module performs a restricted pre-scan over raw lexer output and records only the binding ranges needed for lexical override decisions.

## Public API

Implemented API:

```rust
pub struct ScopeSkeleton {
    pub frames: Vec<LexicalScopeFrame>,
    pub blocks: Vec<LexicalBlockRange>,
    pub statements: Vec<LexicalStatementRange>,
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

pub struct LexicalBlockRange {
    pub kind: LexicalBlockKind,
    pub range: SourceRange,
}

pub struct LexicalStatementRange {
    pub kind: LexicalStatementKind,
    pub range: SourceRange,
}

pub trait ScopeLexView {
    fn binding_overrides_symbol(&self, spelling: &str, position: SourcePos) -> bool;
}

pub fn build_scope_skeleton(raw: &RawTokenStream) -> ScopeSkeleton;
```

## Recognized Structure

The skeleton pre-scan recognizes only reserved-keyword-shaped structure needed to approximate lexical scopes:

- block boundaries such as `definition`, `proof`, `now`, `case`, `suppose`, `hereby`, `algorithm`, `do`, and `end`;
- binder-introducing forms such as `let`, `for`, `ex`, `reserve`, `given`, `consider`, `set`, `reconsider`, `take`, `deffunc`, `defpred`, and algorithm `var` / `const` forms;
- comma-separated binding lists in recognized binder positions;
- local names whose binding range can be approximated without parsing expressions.

It is intentionally not a parser. It may under-approximate bindings when source is malformed or when a binding form is not yet implemented.

The skeleton is suitable as a pre-parser handoff object: the parser may consume final tokens together with block and statement ranges from `ScopeSkeleton`. It must not treat the skeleton as the authoritative AST. In particular, expression grammar, type checking, semantic name resolution, and syntax acceptance remain parser/resolver responsibilities.

Lexical lifetimes are conservative:

- `reserve` is top-level/article scoped from the declaration point onward and is ignored with a recoverable diagnostic inside nested blocks;
- `let`, `consider`, `set`, `reconsider name = ...`, `take name = ...`, `deffunc`, `defpred`, and algorithm `var` / `const` bind in the current lexical block, or fall back to a statement range when no block is open;
- `for`, `ex`, and `given` bind only for the recovered statement range;
- algorithm `for ... do` binders, including optional `processed name`, bind in the following `do` block.

The skeleton pre-scan must not require raw scan to split punctuation in advance. It may inspect inside `LexemeRun` spans to recognize delimiters such as `,`, `;`, and block-closing punctuation needed for binding-list recovery.

## Implemented Algorithm Flow

The implementation is a conservative single pass over a reduced token stream.

1. Convert `RawTokenStream` into scope-skeleton tokens. Layout is ignored. `LexemeRun` values are split into identifier-shaped `Word` pieces, comma, semicolon, parentheses, and `Other` runs. Other raw token kinds become `Other`.
2. Initialize a synthetic root frame starting at byte `0`, an empty block stack, and an empty `pending_do_bindings` buffer used by algorithm `for ... do` forms.
3. Walk tokens from left to right. Recognized block-opening words (`algorithm`, `definition`, `proof`, `now`, `suppose`, `hereby`, and `do`) push an open frame. `case` opens a frame only when the rest of the statement does not contain `do`, so algorithm `case ... do` does not look like a proof branch. `end` pops one frame and records both a block range and a lexical scope frame.
4. Recognized binder words delegate to shape-specific parsers. Plain binder lists such as `let x, y be ...` accept identifier-shaped names until a comma, semicolon, or stop word. Named-equals binders such as `set x = ...`, `reconsider x = ...`, and `take x = ...` require the `name =` shape. Algorithm `var` and `const` binders scan comma-separated declaration heads while tracking parenthesis depth so initializer tuples do not create extra binders.
5. `ghost var` and `ghost const` are treated as algorithm binders; unsupported `ghost` forms produce a recoverable diagnostic and do not invent bindings.
6. Binder lifetimes are assigned by shape. `reserve` contributes to the root frame only outside nested blocks. `for`, `ex`, and `given` create statement-local frames. `consider`, `let` inside a block, named-equals binders, `deffunc`, `defpred`, `var`, `const`, and `processed` extend the current block frame when one exists, otherwise fall back to a statement-local frame. Algorithm `for ... do` moves its binders, plus optional `processed name`, into the following `do` block via `pending_do_bindings`.
7. Before bindings enter a frame, names are deduplicated against existing names in that same lexical scope. Duplicates are ignored with a diagnostic so the skeleton cannot create two competing overrides for the same spelling and range.
8. At EOF, any still-open block is closed at `source_end` and reported as `MissingEnd`. The root frame is emitted only if it contains bindings. Frames, blocks, statements, and diagnostics are sorted by source span before returning.

`ScopeLexView::binding_overrides_symbol` then answers a narrow question: a binding overrides a spelling at position `p` only when `p` lies inside the frame, the spelling matches, and the binding's own introduction span has already ended. This last condition prevents the binder occurrence itself from being reclassified as an identifier too early.

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
- duplicate binding name in the same lexical scope;
- block nesting that cannot be paired reliably.

These diagnostics do not accept or reject the program semantically; the parser and resolver later produce authoritative syntax/name diagnostics.

## Tests

Tests should cover:

- empty skeleton;
- simple `let x`-style binding;
- comma-separated binders;
- nested block ranges;
- statement ranges for statement-local binders;
- proof branches (`case`, `suppose`, `hereby`) and algorithm `do` ranges;
- local names from `take`, `deffunc`, `defpred`, and algorithm binders;
- malformed binders under-approximate rather than inventing names;
- `ScopeLexView` returns true only inside the binding range;
- deterministic output for repeated runs.
