# Module: scope_skeleton

> Canonical language: English. English canonical version: [../en/scope_skeleton.md](../en/scope_skeleton.md).

## Purpose

This module builds a lexical scope skeleton before full parsing.

Token disambiguation は、scoped identifier binding が active user symbol を override するかを知る必要があります。Full parsing は token disambiguation に依存するため、この module は raw lexer output に対する restricted pre-scan を行い、lexical override decisions に必要な binding ranges だけを記録します。

## Public API

Expected API direction:

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

- `definition`, `proof`, `now`, `case`, `suppose`, `hereby`, `algorithm`, `do`, `end` などの block boundaries;
- `let`, `for`, `ex`, `reserve`, `given`, `consider`, `set`, `reconsider`, `take`, `deffunc`, `defpred`, algorithm `var` / `const` などの binder-introducing forms;
- recognized binder positions 内の comma-separated binding lists;
- full expression parsing なしに binding range を近似できる labels or local names.

It is intentionally not a parser. Source が malformed な場合や binding form が未実装の場合、bindings を under-approximate してよいです。

この skeleton は pre-parser handoff object として扱えます。Parser は final tokens とともに `ScopeSkeleton` の block ranges / statement ranges を参照してよいですが、skeleton を authoritative AST として扱ってはいけません。Expression grammar、type checking、semantic name resolution、syntax acceptance は parser/resolver の責務として残します。

Lexical lifetime は保守的に扱います:

- `reserve` は top-level/article scoped で、declaration point 以降のみ有効です。nested block 内の `reserve` は recoverable diagnostic として under-approximate します。
- `let`, `consider`, `set`, `reconsider name = ...`, `take name = ...`, `deffunc`, `defpred`, algorithm `var` / `const` は current lexical block に bind します。open block がない場合は statement range に fallback して file scope へ漏らしません。
- `for`, `ex`, `given` は recovered statement range にだけ bind します。
- algorithm `for ... do` の binder と optional `processed name` は後続の `do` block に bind します。

Skeleton pre-scan は raw scan が punctuation を事前に分割することを要求してはいけません。binding-list recovery に必要な `,`, `;`, block-closing punctuation などを認識するために、`LexemeRun` spans の内部を inspect してよいです。

## Override Semantics

`ScopeLexView` answers only whether a spelling may be treated as a scoped identifier for lexical disambiguation at a given position.

It must not answer:

- identifier が semantically defined か;
- identifier がどの declaration に resolve されるか;
- identifier の type;
- symbol use が valid か;
- selected overload.

## Determinism

The skeleton must be deterministic for the same raw token stream.

Recovery が必要な場合、diagnostics and recovered frames は source span で order されます。誤って disambiguation を変える binding を作るより、under-approximation を優先します。

## Error Handling

Diagnostics are structural and recoverable:

- unmatched or missing `end`;
- malformed binder list;
- binder keyword followed by unsupported raw shape;
- same lexical scope 内の duplicate binding name;
- block nesting that cannot be paired reliably.

These diagnostics do not accept or reject the program semantically; the parser and resolver later produce authoritative syntax/name diagnostics.

## Tests

Tests should cover:

- empty skeleton;
- simple `let x`-style binding;
- comma-separated binders;
- nested block ranges;
- statement-local binder の statement ranges;
- `case`, `suppose`, `hereby` と algorithm `do` ranges;
- `take`, `deffunc`, `defpred`, algorithm binders 由来の local names;
- malformed binders under-approximate rather than inventing names;
- `ScopeLexView` returns true only inside the binding range;
- deterministic output for repeated runs.
