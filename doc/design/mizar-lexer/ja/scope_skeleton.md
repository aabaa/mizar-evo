# Module: scope_skeleton

> Canonical language: English. English canonical version: [../en/scope_skeleton.md](../en/scope_skeleton.md).

## Purpose

このモジュールは、本格的なパース(full parsing)の前に、字句スコープスケルトン(lexical scope skeleton)を構築します。

トークンの曖昧性解消(disambiguation)は、スコープ付き識別子束縛(scoped identifier binding)がアクティブなユーザーシンボルを上書き(override)するかを知る必要があります。一方で、本格的なパースはトークンの曖昧性解消に依存します。この循環を避けるため、このモジュールは生の字句解析器(lexer)の出力に対して制限付きの事前スキャン(pre-scan)を行い、字句上の上書き判断に必要な束縛範囲だけを記録します。

## Public API

実装済み API:

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

スケルトンの事前スキャンは、字句スコープを近似するために必要な、予約キーワードの形をした構造だけを認識します。

- `definition`, `proof`, `now`, `case`, `suppose`, `hereby`, `algorithm`, `do`, nested `struct`, explicit `inherit ... where`, `end` などのブロック境界;
- `let`, `for`, `ex`, `reserve`, `given`, `consider`, `set`, `reconsider`, `take`, `deffunc`, `defpred`、アルゴリズムの `var` / `const` などの、束縛子(binder)を導入する形式;
- 認識された束縛子の位置にある、カンマ区切りの束縛リスト;
- 式をパースせずに束縛範囲を近似できる局所名.

これは意図的にパーサーではありません。ソースが不正な場合や束縛の形式が未実装の場合は、束縛を過小近似(under-approximate)してよいです。

このスケルトンは、パーサー前段の受け渡しオブジェクトとして扱えます。パーサーは、最終トークンとともに `ScopeSkeleton` のブロック範囲・文範囲を参照してよいですが、スケルトンを正規の AST として扱ってはいけません。特に、式の文法、型検査、意味論的な名前解決、構文の受理は、引き続きパーサー/リゾルバの責務として残します。

字句上の有効範囲(lifetime)は保守的に扱います。

- `reserve` は記事(article)全体のトップレベルスコープを持ち、宣言地点以降でのみ有効です。入れ子のブロック内の `reserve` は、回復可能な診断とともに過小近似します。
- `let`, `consider`, `set`, `reconsider type_change_list as ...`, `take name = ...`, `deffunc`, `defpred`、アルゴリズムの `var` / `const` は、現在の字句ブロックに束縛します。開いているブロックがない場合は文範囲にフォールバックします。
- `for`, `ex`, `given` は、復元した文範囲にだけ束縛します。
- アルゴリズムの `for ... do` の束縛子と、省略可能な `processed name` は、後続の `do` ブロックに束縛します。

スケルトンの事前スキャンは、生スキャンに句読点(punctuation)を事前分割させることを要求してはいけません。束縛リストと item tail の復元に必要な `,`, `;`、括弧、角括弧、波括弧、ブロックを閉じる句読点などを認識するために、`LexemeRun` の内部を調べてよいです。

## 実装上のアルゴリズムの流れ

実装は、簡約したトークンストリームに対する保守的な単一パスです。

1. `RawTokenStream` を、スコープスケルトン専用のトークンに変換します。レイアウト(空白類)は無視します。`LexemeRun` は、識別子の形をした `Word`、カンマ、セミコロン、括弧、角括弧、波括弧、`Other` のランに分割します。それ以外の生トークンの種別は `Other` として扱います。
2. バイト `0` から始まる合成のルートフレーム、空のブロックスタック、空の `pending_do_bindings` を初期化します。`pending_do_bindings` は、アルゴリズムの `for ... do` 形式の束縛子を後続の `do` ブロックに渡すための一時バッファです。
3. トークンを左から右へ走査します。`algorithm`, `definition`, `proof`, `now`, `suppose`, `hereby`, `struct`, `do` は、ブロックを開く語として開いたフレームをプッシュします。`inherit` は、statement semicolon または block `end` より前に `where` が現れる場合だけフレームをプッシュし、explicit inheritance block を表しつつ shorthand `inherit ...;` は statement 形の declaration として扱います。`case` は、その文の残りに `do` が含まれない場合だけ証明の分岐としてフレームを開きます。これにより、アルゴリズムの `case ... do` を証明の分岐と誤認しません。`end` はフレームを 1 つポップし、ブロック範囲と字句スコープフレームの両方を記録します。
4. 束縛子の語は、形状ごとのパーサーに委譲します。`let x, y be ...` のような単純な束縛子リストは、カンマ・セミコロン・停止語までの識別子の形をした名前を読みます。`set x = ...` と `take x = ...` のような名前付き等号の束縛子は、`name =` の形状を要求します。`reconsider` は `type_change_list` を保守的に走査し、各 item 先頭の識別子を記録し、任意の等号右辺を括弧・角括弧・波括弧の深さを追跡しながらトップレベルのカンマまたは `as` まで読み飛ばします。アルゴリズムの `var` / `const` は、括弧の深さを追跡しながらカンマ区切りの宣言ヘッドを読むため、初期化子のタプルが余計な束縛子を作ることはありません。
5. `ghost var` と `ghost const` はアルゴリズムの束縛子として扱います。それ以外の `ghost` 形式は、回復可能な診断を出し、束縛を捏造しません。
6. 束縛の有効範囲は形状ごとに決めます。`reserve` は、入れ子のブロックの外でのみルートフレームに入ります。`for`, `ex`, `given` は文単位のフレームを作ります。`consider`、`reconsider`、ブロック内の `let`、名前付き等号の束縛子、`deffunc`、`defpred`、`var`、`const`、`processed` は、開いているブロックがあれば現在のブロックフレームを拡張し、なければ文単位のフレームにフォールバックします。アルゴリズムの `for ... do` は、束縛子と省略可能な `processed name` を、`pending_do_bindings` 経由で次の `do` ブロックに移します。
7. 束縛をフレームに入れる前に、同じ字句スコープ内の既存の名前と重複しないか確認します。重複は診断を出して無視します。これにより、同じ綴り/範囲に対して競合する上書きがスケルトン内に作られないようにします。
8. EOF に到達した時点でスタックに残っているブロックは `source_end` で閉じ、`MissingEnd` 診断を出します。ルートフレームは束縛を持つ場合だけ出力します。最後に、フレーム・ブロック・文・診断をソーススパン順に整列して返します。

`ScopeLexView::binding_overrides_symbol` は、意図的に狭い問いだけに答えます。すなわち、位置 `p` がフレーム内にあり、綴りが一致し、かつ束縛自身の導入スパンがすでに終わっている場合にのみ、上書きを真とします。この最後の条件により、束縛子の出現そのものが早すぎる段階で識別子に再分類されることを防ぎます。

## Override Semantics

`ScopeLexView` は、ある綴りを特定位置でスコープ付き識別子として扱ってよいか、という字句の曖昧性解消向けの問いにだけ答えます。

以下には答えません。

- 識別子が意味論的に定義済みか;
- 識別子がどの宣言に解決されるか;
- 識別子の型;
- シンボルの使用が有効か;
- どのオーバーロードが選択されるか。

## Determinism

同じ生トークンストリームに対して、スケルトンは決定的でなければなりません。

回復が必要な場合、診断と復元したフレームはソーススパン順に並べます。誤って曖昧性解消を変えてしまう束縛を作るより、過小近似を優先します。

## Error Handling

診断は構造的かつ回復可能です。

- 対応しない、または欠落した `end`;
- 不正な束縛子リスト;
- 束縛子キーワードの後に未対応の生形状が続く;
- 同じ字句スコープ内の重複した束縛名;
- 確実に対応付けられないブロックの入れ子.

これらの診断はプログラムを意味論的に受理/拒否しません。後続のパーサーとリゾルバが、正規の構文/名前の診断を生成します。

## Tests

テストでは以下を確認します。

- 空のスケルトン;
- 単純な `let x` 形式の束縛;
- カンマ区切りの束縛子;
- 入れ子のブロック範囲;
- 文単位の束縛子に対する文範囲;
- `case`, `suppose`, `hereby`、アルゴリズムの `do` 範囲、nested `struct` / explicit `inherit ... where` 範囲;
- `take`, `deffunc`, `defpred`、アルゴリズムの束縛子に由来する局所名;
- 不正な束縛子では名前を捏造せず過小近似すること;
- `ScopeLexView` が束縛範囲の内側でだけ真を返すこと;
- 繰り返し実行で出力が決定的であること。
