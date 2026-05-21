# Module: scope_skeleton

> Canonical language: English. English canonical version: [../en/scope_skeleton.md](../en/scope_skeleton.md).

## Purpose

この module は、full parsing の前に lexical scope skeleton を構築します。

Token disambiguation は、scoped identifier binding が active user symbol を override するかを知る必要があります。一方で full parsing は token disambiguation に依存します。その循環を避けるため、この module は raw lexer output に対する restricted pre-scan を行い、lexical override decision に必要な binding range だけを記録します。

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

Skeleton pre-scan は、lexical scope を近似するために必要な reserved-keyword-shaped structure だけを認識します。

- `definition`, `proof`, `now`, `case`, `suppose`, `hereby`, `algorithm`, `do`, `end` などの block boundary;
- `let`, `for`, `ex`, `reserve`, `given`, `consider`, `set`, `reconsider`, `take`, `deffunc`, `defpred`, algorithm `var` / `const` などの binder-introducing forms;
- recognized binder positions 内の comma-separated binding lists;
- full expression parsing なしに binding range を近似できる local names.

これは意図的に parser ではありません。source が malformed な場合や binding form が未実装の場合、binding を under-approximate してよいです。

この skeleton は pre-parser handoff object として扱えます。Parser は final token とともに `ScopeSkeleton` の block range / statement range を参照してよいですが、skeleton を authoritative AST として扱ってはいけません。Expression grammar、type checking、semantic name resolution、syntax acceptance は parser/resolver の責務として残します。

Lexical lifetime は保守的に扱います。

- `reserve` は top-level/article scoped で、declaration point 以降のみ有効です。nested block 内の `reserve` は recoverable diagnostic として under-approximate します。
- `let`, `consider`, `set`, `reconsider name = ...`, `take name = ...`, `deffunc`, `defpred`, algorithm `var` / `const` は current lexical block に bind します。open block がない場合は statement range に fallback し、file scope へ漏らしません。
- `for`, `ex`, `given` は recovered statement range にだけ bind します。
- algorithm `for ... do` の binder と optional `processed name` は後続の `do` block に bind します。

Skeleton pre-scan は raw scan が punctuation を事前に分割することを要求してはいけません。binding-list recovery に必要な `,`, `;`, block-closing punctuation などを認識するために、`LexemeRun` span の内部を調べてよいです。

## 実装上のアルゴリズムの流れ

実装は、縮約した token stream に対する保守的な single pass です。

1. `RawTokenStream` を scope skeleton 専用 token に変換します。layout は無視します。`LexemeRun` は identifier-shaped な `Word`、comma、semicolon、parentheses、`Other` run に分割します。それ以外の raw token kind は `Other` として扱います。
2. byte `0` から始まる synthetic root frame、空の block stack、空の `pending_do_bindings` を初期化します。`pending_do_bindings` は algorithm `for ... do` forms の binder を後続 `do` block に渡すための一時 buffer です。
3. token を左から右へ走査します。`algorithm`, `definition`, `proof`, `now`, `suppose`, `hereby`, `do` は block-opening word として open frame を push します。`case` は、その statement の残りに `do` が含まれない場合だけ proof branch の frame を開きます。これにより algorithm `case ... do` を proof branch と誤認しません。`end` は frame を pop し、block range と lexical scope frame の両方を記録します。
4. binder word は shape-specific parser に委譲します。`let x, y be ...` のような plain binder list は、comma、semicolon、stop word まで identifier-shaped name を読みます。`set x = ...`, `reconsider x = ...`, `take x = ...` のような named-equals binder は `name =` shape を要求します。algorithm `var` / `const` は parentheses depth を追跡しながら comma-separated declaration head を読むため、initializer tuple が余計な binder を作ることはありません。
5. `ghost var` と `ghost const` は algorithm binders として扱います。それ以外の `ghost` form は recoverable diagnostic を出し、binding を捏造しません。
6. binding lifetime は shape ごとに決めます。`reserve` は nested block 外でのみ root frame に入ります。`for`, `ex`, `given` は statement-local frame を作ります。`consider`、block 内の `let`、named-equals binders、`deffunc`、`defpred`、`var`、`const`、`processed` は open block があれば current block frame を拡張し、なければ statement-local frame に fallback します。algorithm `for ... do` は binder と optional `processed name` を `pending_do_bindings` 経由で次の `do` block に移します。
7. binding を frame に入れる前に、同じ lexical scope 内の既存 name と重複しないか確認します。duplicate は diagnostic を出して無視します。これにより、同じ spelling/range に対して競合する override が skeleton 内に作られません。
8. EOF に到達した時点で stack に残っている block は `source_end` で閉じ、`MissingEnd` diagnostic を出します。root frame は binding を持つ場合だけ出力します。最後に frames、blocks、statements、diagnostics を source span 順に sort して返します。

`ScopeLexView::binding_overrides_symbol` は、意図的に狭い質問だけに答えます。position `p` が frame 内にあり、spelling が一致し、かつ binding 自身の introduction span がすでに終わっている場合にのみ override を true にします。最後の条件により、binder occurrence そのものが早すぎる段階で identifier に再分類されることを防ぎます。

## Override Semantics

`ScopeLexView` は、ある spelling を特定位置で scoped identifier として扱ってよいか、という lexical disambiguation 用の質問だけに答えます。

以下には答えません。

- identifier が semantically defined か;
- identifier がどの declaration に resolve されるか;
- identifier の type;
- symbol use が valid か;
- selected overload.

## Determinism

同じ raw token stream に対して、skeleton は deterministic でなければなりません。

Recovery が必要な場合、diagnostic と recovered frame は source span 順に並べます。誤って disambiguation を変える binding を作るより、under-approximation を優先します。

## Error Handling

Diagnostic は structural で recoverable です。

- unmatched or missing `end`;
- malformed binder list;
- binder keyword followed by unsupported raw shape;
- same lexical scope 内の duplicate binding name;
- reliable に pair できない block nesting.

これらの diagnostic は program を semantic に accept/reject しません。後続の parser と resolver が authoritative syntax/name diagnostic を生成します。

## Tests

テストでは以下を確認します。

- empty skeleton;
- simple `let x`-style binding;
- comma-separated binder;
- nested block range;
- statement-local binder の statement range;
- `case`, `suppose`, `hereby` と algorithm `do` range;
- `take`, `deffunc`, `defpred`, algorithm binder 由来の local name;
- malformed binder では名前を捏造せず under-approximate すること;
- `ScopeLexView` が binding range 内でだけ true を返すこと;
- repeated run で output が deterministic であること。
