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

- `definition`, `proof`, `now`, `case`, `suppose`, `hereby`, `algorithm`、algorithm `for ... do`、match `otherwise` branch、nested `struct`、explicit `inherit ... where`、`end` などのブロック境界;
- `let`, `for`, `ex`, `reserve`, `given`, `consider`, `set`, `reconsider`, `take`, `deffunc`, `defpred`、アルゴリズムの `var` / `const` などの、束縛子(binder)を導入する形式;
- 認識された束縛子の位置にある、カンマ区切りの束縛リスト;
- 式をパースせずに束縛範囲を近似できる局所名.

これは意図的にパーサーではありません。ソースが不正な場合や束縛の形式が未実装の場合は、束縛を過小近似(under-approximate)してよいです。

このスケルトンは識別子の束縛だけを追跡します。インライン `deffunc`、`defpred`、
アルゴリズム名は識別子のみのローカル宣言であり、ユーザーシンボルの辞書エントリを
追加しません。現在のモジュールの述語 / functor 記法、コンストラクタ名、別名、
演算子メタデータは、`lexical_environment.md` で説明する、範囲対応の字句宣言事前パスの
責務です。

このスケルトンは、パーサー前段の受け渡しオブジェクトとして扱えます。パーサーは、最終トークンとともに `ScopeSkeleton` のブロック範囲・文範囲を参照してよいですが、スケルトンを正規の AST として扱ってはいけません。特に、式の文法、型検査、意味論的な名前解決、構文の受理は、引き続きパーサー/リゾルバの責務として残します。

字句上の有効範囲(lifetime)は保守的に扱います。

- `reserve` は記事(article)全体のトップレベルスコープを持ち、宣言地点以降でのみ有効です。入れ子のブロック内の `reserve` は、回復可能な診断とともに過小近似します。
- `let`, `consider`, `set`, `reconsider type_change_list as ...`、名前付きの
  `take name = ...` example、`deffunc`, `defpred`、アルゴリズムの `var` /
  `const` は、現在の字句ブロックに束縛します。開いているブロックがない場合は
  文範囲にフォールバックします。
- 名前なしの `take term_expression` example は witness term の使用であり、字句上の
  束縛宣言ではありません。そのため scope frame への binding contribution と
  binder statement range のいずれも追加しません。
- `for`, `ex`, `given` は、復元した文範囲にだけ束縛します。
- `algorithm ... do ... end` は 1 つの algorithm lexical block です。header の `do` は別の `Do` frame を開きません。
- アルゴリズムの `for ... do` の束縛子と、省略可能な `processed name` は、後続の `Do` ブロックに束縛します。それ以外の header ではない `do` token も、保守的な `Do` ブロックを開きます。
- open algorithm block 内では、`end` または `end;` の直後に現れる `otherwise` が conservative な match-branch heuristic として `Do` block を開きます。これにより branch-local `end;` と final match `end;` の両方が対応できます。definition 側の `otherwise` clause と non-algorithm の `end; otherwise` 形は block を開きません。

スケルトンの事前スキャンは、生スキャンに句読点(punctuation)を事前分割させることを要求してはいけません。束縛リストと item tail の復元に必要な `,`, `;`、括弧、角括弧、波括弧、ブロックを閉じる句読点などを認識するために、`LexemeRun` の内部を調べてよいです。

## 実装上のアルゴリズムの流れ

実装は、簡約したトークンストリームに対する保守的な単一パスです。

1. `RawTokenStream` を、スコープスケルトン専用のトークンに変換します。レイアウト(空白類)は無視します。`LexemeRun` は、識別子の形をした `Word`、カンマ、セミコロン、括弧、角括弧、波括弧、`Other` のランに分割します。それ以外の生トークンの種別は `Other` として扱います。
2. バイト `0` から始まる合成のルートフレーム、空のブロックスタック、空の `pending_do_bindings` を初期化します。`pending_do_bindings` は、アルゴリズムの `for ... do` 形式の束縛子を後続の `do` ブロックに渡すための一時バッファです。
3. トークンを左から右へ走査します。`algorithm`, `definition`, `proof`, `now`, `suppose`, `hereby`, `struct` は、ブロックを開く語として開いたフレームをプッシュします。`do` token は、pending loop binding がなく開いている algorithm body を始める header `do` でない限り `Do` frame を開きます。その header `do` は `Algorithm` frame に付着します。`inherit` は、statement semicolon または block `end` より前に `where` が現れる場合だけフレームをプッシュし、explicit inheritance block を表しつつ shorthand `inherit ...;` は statement 形の declaration として扱います。`case` は、その文の残りに `do` が含まれない場合だけ証明の分岐としてフレームを開きます。これにより、アルゴリズムの `case ... do` を証明の分岐と誤認しません。`otherwise` は completed algorithm match case（`end; otherwise`）の後に現れる場合だけ conservative な `Do` frame を開き、definition 側の conditional definiens では開きません。`end` はフレームを 1 つポップし、ブロック範囲と字句スコープフレームの両方を記録します。
4. 束縛子の語は、形状ごとのパーサーに委譲します。`let x, y be ...` の
   ような単純な束縛子リストは、カンマ・セミコロン・停止語までの識別子の形をした
   名前を読みます。`set x = ...` は `name =` の形状を要求します。`take` では、
   先頭の識別子の直後に `=` がある場合だけ名前付き witness binding の候補とします。
   名前なし term になり得る空でない先頭形状は、束縛を捏造せず scope-skeleton
   diagnostic も出さずに文末まで回復します。authoritative term syntax は引き続き
   parser の所有です。`reconsider` は `type_change_list` を保守的に走査し、各 item
   先頭の識別子を記録し、任意の等号右辺を括弧・角括弧・波括弧の深さを追跡しながら
   トップレベルのカンマまたは `as` まで読み飛ばします。アルゴリズムの `var` /
   `const` は、括弧の深さを追跡しながらカンマ区切りの宣言ヘッドを読むため、
   初期化子のタプルが余計な束縛子を作ることはありません。
5. `ghost var` と `ghost const` はアルゴリズムの束縛子として扱います。`ghost target := term;` は束縛しない assignment として扱い、scope diagnostic なしで読み飛ばします。それ以外の `ghost` 形式は、回復可能な診断を出し、束縛を捏造しません。
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

## Lexer Task 258B3M2P1 Frozen Contract

Checker Task 258B3M2 の fresh preflight で、real frontend consumer の
lower-stage disagreement が見つかりました。canonical Chapter 15 §15.4.4 は
`example ::= term_expression | identifier "=" term_expression` と定義し、
名前なし example として `take 101;` を示します。Chapter 13 §§13.1 と 13.1.4 は
numeral を primary term expression と定義します。parser はこの exact shape から
unrecovered AST を生成しますが、現在の scope skeleton はすべての `take` を
named-equals binder parser に送るため `UnsupportedBinderShape` を報告します。
`mizar-frontend::lexing::token_stream_from_raw` はこの recoverable scope
diagnostic を real frontend stream に写像するため、checker statement transport
より前に disagreement が観測されます。

凍結する implementation slice は次のとおりです。

| Surface | Scope-skeleton result |
|---|---|
| Exact authority witness | final LF を持つ `proof\ntake 101;\nend;\n` は 21 bytes、SHA-256 `60cb34c7ca79ec289319c61198965a4d0a9918b5aaca34957ee1df9f8a2c3648`。`Take` binding contribution、binder statement range、scope diagnostic のいずれも作らず `take` statement を回復する。enclosing proof frame は不変。 |
| 名前なし identifier witness `take x;` | 同じ non-binding recovery を適用する。`x` は term use であり lexical override にしてはならない。 |
| 既存の名前付き witness `take k = 101;` | 既存の先頭 1 件の `BindingShapeKind::Take` binding と現在の lexical lifetime を維持する。 |
| `take;` や `take = 101;` のような空または separator-led の malformed shape | 束縛を捏造せず under-approximate し、recoverable unsupported-shape diagnostic を維持する。rejection の authority は parser に残る。 |

これは `crates/mizar-lexer/src/scope_skeleton.rs` の bounded
`source_drift`、従来の named-only 文言の `design_drift`、lexer/frontend
unit suite の `test_gap` です。
`tests/lexical/fail/fail_lexical_scope_skeleton_complete_003.src` の
negative `take 42;` row は `test_expectation_drift` です。その expectation は、
より上位 authority の Chapter 15 rule と、既存の named/unnamed parser pass
fixture に矛盾します。implementation task では、その derived negative source row
だけを実際に malformed な `take = 42;` row に置き換えてよく、expectation file と
diagnostic count/order は維持します。canonical specification、既存 `.miz`
source、expectation を変更する authority はなく、`spec_gap`,
`boundary_violation`, `repo_metadata_conflict` もありません。

implementation ownership は次に限定します。

- `crates/mizar-lexer/src/scope_skeleton.rs` の `take` dispatch/recovery;
- unnamed numeral/identifier、named-witness、malformed-shape controls に対する
  `scope_skeleton_distinguishes_unnamed_and_named_take_shapes` という exact 1 件の
  compound scope-skeleton library test;
- exact 21-byte numeral witness が mapped scope-skeleton diagnostic を生成しないことを
  示す `scope_skeleton_unnamed_take_term_is_not_a_frontend_diagnostic` という exact
  1 件の compound frontend library test;
- 上記の derived lexical fail-fixture source correction 1 件。

lexer で term expression を parse すること、witness term を resolve すること、
semantic witness binding を作ること、parser AST または resolver provenance を
変更すること、mixed witness list の後続 named example を実装すること、checker
statement transport または proof semantics を変更することは禁止します。
後続 named-example の lexical under-approximation と Task 258B3M2 checker
transport の全 concern は別 follow-up に残します。

documentation prerequisite は production source、fixture、sidecar、trace
row/status/count、test list、CLI/count/hash baseline を変更しません。
implementation でも active manifest counts と CLI hashes は不変の見込みであり、
変更してよいのは owned lexer/frontend source と unit-test inventories、および
derived lexical fixture content だけです。

fresh library baseline は lexer/frontend tests `146/132`、exact matrix による
projected counts は `147/133` です。sorted raw test-entry hashes は
`cef872d7c7597f09dea32163b3c1f27d7cf5f4bf34e250bae019941af956869e`
と
`749cc61010d94a45fe9d5fddff306e419fa245463205769f848539826958169c`、
normalized test-name hashes は
`d9e6e8960d9f1be2d23b5b546f7a3390dc156ae8437946f6eac22f47438eef55`
と
`143e2385e210b356da817b2662b80caa7515fe8dfa0c5c114171745b78ce4d52`
です。current module sizes は lexer scope production / lexer scope tests /
frontend lexing の順に `1294/400/2452` lines です。post-implementation
hashes/sizes は pre-authorized target にせず実測します。

exit には、exact frontend source に
`ScopeSkeleton(UnsupportedBinderShape)` がないこと、凍結した compatibility
assertion がすべて通ること、parser acceptance が不変であること、EN/JA docs が
一致すること、source/expectation correction が authority order に従うこと、
relevant crate/workspace/fmt/Clippy/CLI verification が通ること、final read-only
quality review が 90/100 以上かつ protocol hard gates がすべて PASS であることを
要求します。

## Lexer Task 258B3M2P1 Implementation Result

implementation は frozen `source_drift`, `design_drift`, `test_gap`,
`test_expectation_drift` を close します。`take` dispatch は既存の initial
`name =` binding path を維持し、plausible な unnamed term start を scope
diagnostic なしの non-binding statement recovery として扱い、empty/separator-led
shape には recoverable diagnostic を維持します。lexer は引き続き term
expression を parse しません。

exact two-test matrix は指定どおり追加されました。library counts は
lexer/frontend `147/133`、sorted raw test-entry hashes は
`d55916e3165613154b586d00d44a29d893d8e902e03ae3ff1975361bb61f27c9`
と
`d9ed6e8c151187eeaa6a1969b05619f75108f33482d49c0b56d6830f468d1623`、
normalized test-name hashes は
`0cb403b4c9390daecfe6f7c5bf44c2fadaa76f6fc8c5f05cba04bbab898b96aa`
と
`a309083b7fbdd769f8bd59860a8772e67ad69935658d56beb7c6cee53dea2034`
です。post-implementation module sizes は lexer scope production / lexer scope
tests / frontend lexing の順に `1330/485/2489` lines です。

derived lexical fail source は 16 lines のままで、SHA-256 は
`d661a81f1d79f760af43aab0c904a7c5400a90e003435d80fc298145ec56d1e5`
です。expectation file、diagnostic count/order、line-based probes は不変です。
real 107-byte theorem preflight は 49 unrecovered parser nodes を維持しつつ、
frontend diagnostics が false scope diagnostic 1 件から 0 件になりました。
active manifest counts と 5 個の CLI hashes はすべて不変です。

## Tests

テストでは以下を確認します。

- 空のスケルトン;
- 単純な `let x` 形式の束縛;
- カンマ区切りの束縛子;
- 入れ子のブロック範囲;
- 文単位の束縛子に対する文範囲;
- `case`, `suppose`, `hereby`、algorithm block、algorithm `for ... do` 範囲、algorithm match `otherwise` branch、nested `struct` / explicit `inherit ... where` 範囲;
- `take`, `deffunc`, `defpred`、アルゴリズムの束縛子に由来する局所名;
- unnamed `take` term は binding も scope diagnostic も生成せず、先頭の named
  witness は binding を維持し、malformed separator-led shape は recoverable
  under-approximation を維持すること;
- 不正な束縛子では名前を捏造せず過小近似すること;
- `ScopeLexView` が束縛範囲の内側でだけ真を返すこと;
- 繰り返し実行で出力が決定的であること。
