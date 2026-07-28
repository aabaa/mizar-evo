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

- block boundaries such as `definition`, `proof`, `now`, `case`, `suppose`, `hereby`, `algorithm`, algorithm `for ... do`, match `otherwise` branches, nested `struct`, explicit `inherit ... where`, and `end`;
- binder-introducing forms such as `let`, `for`, `ex`, `reserve`, `given`, `consider`, `set`, `reconsider`, `take`, `deffunc`, `defpred`, and algorithm `var` / `const` forms;
- comma-separated binding lists in recognized binder positions;
- local names whose binding range can be approximated without parsing expressions.

It is intentionally not a parser. It may under-approximate bindings when source is malformed or when a binding form is not yet implemented.

The skeleton tracks identifier bindings only. Inline `deffunc`, `defpred`, and
algorithm names are identifier-only local declarations and do not add
user-symbol dictionary entries. Current-module predicate/functor notation,
constructor names, aliases, and operator metadata are owned by the
range-aware lexical declaration prepass described in
`lexical_environment.md`.

The skeleton is suitable as a pre-parser handoff object: the parser may consume final tokens together with block and statement ranges from `ScopeSkeleton`. It must not treat the skeleton as the authoritative AST. In particular, expression grammar, type checking, semantic name resolution, and syntax acceptance remain parser/resolver responsibilities.

Lexical lifetimes are conservative:

- `reserve` is top-level/article scoped from the declaration point onward and is ignored with a recoverable diagnostic inside nested blocks;
- `let`, `consider`, `set`, `reconsider type_change_list as ...`, named
  `take name = ...` examples, `deffunc`, `defpred`, and algorithm `var` /
  `const` bind in the current lexical block, or fall back to a statement range
  when no block is open;
- an unnamed `take term_expression` example is a witness term use, not a
  lexical binding declaration, and therefore contributes no binding to a
  scope frame and no binder statement range;
- `for`, `ex`, and `given` bind only for the recovered statement range;
- `algorithm ... do ... end` is one lexical algorithm block; the header `do` does not open a separate `Do` frame.
- algorithm `for ... do` binders, including optional `processed name`, bind in the following `Do` block. Other non-header `do` tokens also open a conservative `Do` block.
- In an open algorithm block, an `otherwise` immediately after `end` or `end;` uses a conservative match-branch heuristic and opens a `Do` block so branch-local `end;` and the final match `end;` can both pair. Definition-side `otherwise` clauses and non-algorithm `end; otherwise` shapes do not open blocks.

The skeleton pre-scan must not require raw scan to split punctuation in advance. It may inspect inside `LexemeRun` spans to recognize delimiters such as `,`, `;`, parentheses, brackets, braces, and block-closing punctuation needed for binding-list and item-tail recovery.

## Implemented Algorithm Flow

The implementation is a conservative single pass over a reduced token stream.

1. Convert `RawTokenStream` into scope-skeleton tokens. Layout is ignored. `LexemeRun` values are split into identifier-shaped `Word` pieces, comma, semicolon, parentheses, brackets, braces, and `Other` runs. Other raw token kinds become `Other`.
2. Initialize a synthetic root frame starting at byte `0`, an empty block stack, and an empty `pending_do_bindings` buffer used by algorithm `for ... do` forms.
3. Walk tokens from left to right. Recognized block-opening words (`algorithm`, `definition`, `proof`, `now`, `suppose`, `hereby`, and `struct`) push an open frame. A `do` token opens a `Do` frame unless it is the header `do` that begins an open algorithm body without pending loop bindings; that header `do` attaches to the `Algorithm` frame instead. `inherit` pushes a frame only when a `where` appears before the statement semicolon or a block `end`, matching the explicit inheritance-block surface while leaving shorthand `inherit ...;` as a statement-shaped declaration. `case` opens a frame only when the rest of the statement does not contain `do`, so algorithm `case ... do` does not look like a proof branch. `otherwise` opens a conservative `Do` frame only when it follows a completed algorithm match case (`end; otherwise`), not for definition-side conditional definiens. `end` pops one frame and records both a block range and a lexical scope frame.
4. Recognized binder words delegate to shape-specific parsers. Plain binder
   lists such as `let x, y be ...` accept identifier-shaped names until a
   comma, semicolon, or stop word. `set x = ...` requires the `name =` shape.
   For `take`, only an initial identifier immediately followed by `=` is
   eligible for named-witness binding; any nonempty initial shape that can be
   an unnamed term is recovered to the statement end without inventing a
   binding or emitting a scope-skeleton diagnostic. Authoritative term syntax
   remains parser-owned. `reconsider` scans the `type_change_list`
   conservatively, records each item-head identifier, and skips optional
   equated right-hand sides until a top-level comma or `as` while tracking
   parenthesis, bracket, and brace depth. Algorithm `var` and `const` binders
   scan comma-separated declaration heads while tracking parenthesis depth so
   initializer tuples do not create extra binders.
5. `ghost var` and `ghost const` are treated as algorithm binders. `ghost target := term;` is treated as a non-binding assignment and skipped without a scope diagnostic. Other `ghost` forms produce a recoverable diagnostic and do not invent bindings.
6. Binder lifetimes are assigned by shape. `reserve` contributes to the root frame only outside nested blocks. `for`, `ex`, and `given` create statement-local frames. `consider`, `reconsider`, `let` inside a block, named-equals binders, `deffunc`, `defpred`, `var`, `const`, and `processed` extend the current block frame when one exists, otherwise fall back to a statement-local frame. Algorithm `for ... do` moves its binders, plus optional `processed name`, into the following `do` block via `pending_do_bindings`.
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

## Lexer Task 258B3M2P1 Frozen Contract

Fresh Checker Task 258B3M2 preflight exposed a lower-stage disagreement in the
real frontend consumer. The canonical Chapter 15 §15.4.4 grammar defines
`example ::= term_expression | identifier "=" term_expression` and gives
`take 101;` as its unnamed example. Chapter 13 §§13.1 and 13.1.4 define a
numeral as a primary term expression. The parser produces an unrecovered AST
for that exact shape, while the current scope skeleton reports
`UnsupportedBinderShape` because it routes every `take` through the
named-equals binder parser. `mizar-frontend::lexing::token_stream_from_raw`
maps that recoverable scope diagnostic into the real frontend stream, making
the disagreement observable before checker statement transport.

The frozen implementation slice is:

| Surface | Scope-skeleton result |
|---|---|
| Exact authority witness | The final-LF-terminated `proof\ntake 101;\nend;\n` is 21 bytes with SHA-256 `60cb34c7ca79ec289319c61198965a4d0a9918b5aaca34957ee1df9f8a2c3648`. Recover its `take` statement without a `Take` binding contribution, binder statement range, or scope diagnostic; the enclosing proof frame is unchanged. |
| Unnamed identifier witness `take x;` | Apply the same non-binding recovery; `x` is a term use and must not become a lexical override. |
| Existing named witness `take k = 101;` | Preserve the existing single initial `BindingShapeKind::Take` binding and its current lexical lifetime. |
| Empty or separator-led malformed shape such as `take;` or `take = 101;` | Continue to under-approximate without inventing a binding and retain the recoverable unsupported-shape diagnostic; the parser remains authoritative for rejection. |

This is a bounded `source_drift` in
`crates/mizar-lexer/src/scope_skeleton.rs`, a `design_drift` in the prior
named-only wording above, and a `test_gap` in the lexer/frontend unit suites.
The `take 42;` negative row in
`tests/lexical/fail/fail_lexical_scope_skeleton_complete_003.src` is
`test_expectation_drift`: its expectation contradicts the higher-authority
Chapter 15 rule and the existing named/unnamed parser pass fixture. The
implementation task may replace only that derived negative source row with a
genuinely malformed `take = 42;` row while preserving the expectation file and
its diagnostic count/order. There is no `spec_gap`, `boundary_violation`,
`repo_metadata_conflict`, or authority for changing canonical specification,
existing `.miz` sources, or expectations.

Implementation ownership is restricted to:

- dispatch/recovery for `take` in
  `crates/mizar-lexer/src/scope_skeleton.rs`;
- exactly one compound scope-skeleton library test,
  `scope_skeleton_distinguishes_unnamed_and_named_take_shapes`, for unnamed
  numeral/identifier, named-witness, and malformed-shape controls;
- exactly one compound frontend library test,
  `scope_skeleton_unnamed_take_term_is_not_a_frontend_diagnostic`, proving the
  exact 21-byte numeral witness produces no mapped scope-skeleton diagnostic;
- the single derived lexical fail-fixture source correction described above.

The task must not parse term expressions in the lexer, resolve witness terms,
create semantic witness bindings, change parser ASTs or resolver provenance,
implement later named examples in a mixed witness list, or change checker
statement transport and proof semantics. Later named-example lexical
under-approximation and every Task 258B3M2 checker transport concern remain
separate follow-ups.

The documentation prerequisite changes no production source, fixture,
sidecar, trace row/status/count, test list, or CLI/count/hash baseline. The
implementation is expected to leave active manifest counts and CLI hashes
unchanged; only the owned lexer/frontend source and unit-test inventories plus
the derived lexical fixture content may change.

The fresh library baseline is lexer/frontend tests `146/132`, projected by the
exact matrix to `147/133`. Sorted raw test-entry hashes are
`cef872d7c7597f09dea32163b3c1f27d7cf5f4bf34e250bae019941af956869e`
and
`749cc61010d94a45fe9d5fddff306e419fa245463205769f848539826958169c`;
normalized test-name hashes are
`d9e6e8960d9f1be2d23b5b546f7a3390dc156ae8437946f6eac22f47438eef55`
and
`143e2385e210b356da817b2662b80caa7515fe8dfa0c5c114171745b78ce4d52`.
Current module sizes are `1294/400/2452` lines for lexer scope production,
lexer scope tests, and frontend lexing. Post-implementation hashes and sizes
must be measured rather than treated as pre-authorized targets.

Exit requires the exact frontend source to be free of
`ScopeSkeleton(UnsupportedBinderShape)`, all frozen compatibility assertions
to pass, parser acceptance to remain unchanged, EN/JA documentation to agree,
the source/expectation correction to follow the authority order, relevant
crate/workspace/fmt/Clippy/CLI verification to pass, and final read-only
quality review to score at least 90/100 with every protocol hard gate passing.

## Lexer Task 258B3M2P1 Implementation Result

The implementation closes the frozen `source_drift`, `design_drift`,
`test_gap`, and `test_expectation_drift`. `take` dispatch now preserves the
existing initial `name =` binding path, treats plausible unnamed term starts as
non-binding statement recovery without a scope diagnostic, and retains
recoverable diagnostics for empty/separator-led shapes. The lexer still does
not parse term expressions.

The exact two-test matrix landed as specified. Library counts are
lexer/frontend `147/133`; sorted raw test-entry hashes are
`d55916e3165613154b586d00d44a29d893d8e902e03ae3ff1975361bb61f27c9`
and
`d9ed6e8c151187eeaa6a1969b05619f75108f33482d49c0b56d6830f468d1623`;
normalized test-name hashes are
`0cb403b4c9390daecfe6f7c5bf44c2fadaa76f6fc8c5f05cba04bbab898b96aa`
and
`a309083b7fbdd769f8bd59860a8772e67ad69935658d56beb7c6cee53dea2034`.
Post-implementation module sizes are `1330/485/2489` lines for lexer scope
production, lexer scope tests, and frontend lexing.

The derived lexical fail source is still 16 lines and now has SHA-256
`d661a81f1d79f760af43aab0c904a7c5400a90e003435d80fc298145ec56d1e5`;
its expectation file, diagnostic count/order, and line-based probes are
unchanged. A real 107-byte theorem preflight keeps 49 unrecovered parser nodes
while frontend diagnostics change from the false single scope diagnostic to
zero. Active manifest counts and all five CLI hashes remain unchanged.

## Tests

Tests should cover:

- empty skeleton;
- simple `let x`-style binding;
- comma-separated binders;
- nested block ranges;
- statement ranges for statement-local binders;
- proof branches (`case`, `suppose`, `hereby`), algorithm blocks, algorithm `for ... do` ranges, algorithm match `otherwise` branches, and nested `struct` / explicit `inherit ... where` ranges;
- local names from `take`, `deffunc`, `defpred`, and algorithm binders;
- unnamed `take` terms produce no binding or scope diagnostic while named
  initial witnesses retain their binding and malformed separator-led shapes
  retain recoverable under-approximation;
- malformed binders under-approximate rather than inventing names;
- `ScopeLexView` returns true only inside the binding range;
- deterministic output for repeated runs.
