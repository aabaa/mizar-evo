# 付録 A. 文法概要

> Canonical language: English. English canonical version: [../en/appendix_a.grammar_summary.md](../en/appendix_a.grammar_summary.md).

この付録には、主要仕様全体で導入されている EBNF 生成と語彙規則が統合されています。セクション番号は、ルールが規範的に定義されている章と一致します。ここのテキストは参照の概要であり、再定義ではありません。

* [A. 文法要約](#appendix-a-grammar-summary)
  * [A.2 語彙構造](#a2-lexical-structure)
    * [A.2.1 文字セット](#a21-character-set)
    * [A.2.2 空白](#a22-whitespace)
    * [A.2.3 トークンのカテゴリ](#a23-token-categories)
    * [A.2.4 予約​​語](#a24-reserved-words)
    * [A.2.5 特殊記号](#a25-special-symbols)
    * [A.2.6 識別子](#a26-identifiers)
    * [A.2.7 数値と文字列リテラル](#a27-numerals-and-string-literals)
    * [A.2.8 ファイルとモジュールの命名](#a28-file-and-module-naming)
    * [A.2.9 コメントと注釈](#a29-comments-and-annotations)
    * [A.2.10 レクサー/パーサーの責任分割](#a210-lexer--parser-responsibility-split)
  * [A.3 タイプシステム](#a3-type-system)
    * [A.3.1 タイプカテゴリ](#a31-type-categories)
    * [A.3.2 型式の文法](#a32-type-expression-grammar)
    * [A.3.3 組み込み型](#a33-built-in-types)
    * [A.3.4 サブ型の意味論](#a34-subtyping-semantics)


## A.2 語彙構造

規格参照: [第 2 章 (語彙構造)](./02.lexical_structure.md)。

### A.2.1 文字セット

* source fileは **UTF-8** でエンコードされます。
* **コード領域**は ASCII のみを使用します。
* **コメントと注釈**には完全な Unicode が含まれる場合があります。
* バックスラッシュ `\` は、文字列リテラル内のエスケープ文字です (§A.2.7)。

### A.2.2 空白

```ebnf
whitespace    = " " | tab | newline ;
tab           = ? ASCII 0x09 ? ;
newline       = ? ASCII 0x0A | 0x0D 0x0A ? ;
```

空白はtokenを区切るものであり、文字列リテラル内を除いて重要ではありません。

### A.2.3 tokenのカテゴリ

コメントの除去 (§A.2.9) とマクロレベルのインポート解決後の Mizar source fileは、次の 5 つのカテゴリから抽出された一連のtokenになります。

1. **予約語** (§A.2.4) — 固定キーワード。
2. **特殊記号** (§A.2.5) — 予約された句読点およびユーザー定義の記号名。
3. **識別子** (§A.2.6) — ユーザーが選択した英数字の名前。
4. **数値** (§A.2.7) — 符号なし整数リテラル。
5. **文字列リテラル** (§A.2.7) — 引用符で囲まれた文字シーケンス。文法上の必要な位置でのみ認識されます。

token化は、file-scoped active lexicon に対する **最長一致ルール** によって管理されます。この lexicon は final tokenization の前に top-of-file import prelude から一度だけ構築されます (§A.2.10)。

### A.2.4 予約​​語

予約語は大文字と小文字が区別され、識別子またはユーザー シンボルとして使用することはできません。

```
algorithm and antonym as assert assume assumed asymmetry attr
be being break by
case cases claim cluster coherence commutativity compatibility computation
conditional connectedness const consider consistency continue contradiction
decreasing deffunc definition defpred do does downto
else end ensures equals ex exhaustive existence export extends
field for from func
ghost given
hence hereby holds
idempotence if iff implies import in infix_operator inherit invariant
involutiveness irreflexivity is it
left lemma let
match means mode
nest non none not now
object of open or otherwise over
per postfix_operator pred prefix_operator private processed
projectivity proof property public
qua
reconsider reduce reducibility redefine reflexivity registration requires
reserve return right
set sethood snapshot st struct such suppose symmetry synonym
take terminating that the then theorem thesis thus to transitivity type
uniqueness
var
where while with
```

### A.2.5 特殊記号

#### 予約された特殊シンボル

```
,   .   ;   :   :=   (   )   [   ]   {   }   .{
=   <>   &   ->   .=   .*   @[   ...
```

|token |役割 |
|---|---|
| `,` `;` `:` |区切り文字 |
| `:=` |割り当て — フィールド更新 (§13.3.3)、変数初期化 (§20.3.1)、アルゴリズム割り当て (§20.4) |
| `.` |複合token オープナー、セレクター アクセス / 更新、namespaceセパレーター、またはユーザー登録のバイナリ 関手 (§A.2.5 ドット曖昧さ回避) - ユーザーによって再定義することもできる唯一の予約記号。
| `.*` |一括引用 (§16.5) |
| `.{` … `}` |グループ化された引用 (§16.5); `}` はセット/フランケルクローザーとしても機能します |
| `@[` … `]` |library の注釈 (§21.2)。 `]` はリスト/インデックスクローザーとしても機能します。
| `=` `<>` `.=` |等式、組み込み不等式、段階的変換 |
| `->` |関数矢印 |
| `...` |省略記号 |
| `(` `)` `[` `]` `{` `}` `&` |標準的なグループ化と結合 |

#### ユーザー定義のシンボル名

```ebnf
symbol_char = ? any ASCII graphic character except "@" and whitespace ? ;
user_symbol = symbol_char { symbol_char } ;
```

`@` を除く任意の ASCII グラフィックが許容されます。ユーザーシンボルは、予約語または `.` 以外の予約された特殊シンボルと正確に一致してはなりません。最長一致により曖昧さが解決されます。後のインポートは、以前のインポートを同点でシャドウします。

#### ドットの曖昧さ回避 (parser側)

§A.2.10 で採用されたレクサー/parserの分割では、`.` は単一の `DOT` tokenとして発行され、parserは優先順位に従ってその役割を解決します。

1. **複合予約token** — `.{`、`.*`、`.=`、`...` は、必要な後続文字が存在する場合、レクサーによって直接認識されます。
2. **`.` を含むユーザー定義シンボル** — 例: `|.`、`.|`、`|. .|`、または古典的な Mizar 形式 `f.x` を与えるバイナリ関手 `.`。レクサーは、アクティブなレキシコンに対する最長一致により、登録されたシンボルを認識します。
3. **セレクター アクセス / 更新** — 用語式の直後にある `DOT` は、フィールド アクセス (`p.x`、`line.end.y`; §5.7、§13.3.2)、または `:=` の左側としてフィールド更新 (§13.3.3) を示します。
4. **namespace区切り文字** — module参照として使用されるパス内の識別子間の `DOT` (`import`、引用 `by` 句、`@[...]`、または用語コンテキストが確立される前の修飾名内) は、namespaceコンポーネント (§A.2.8) を分離します。

変数としてもスコープ内にあるnamespaceコンポーネントは、変数として解決されます。次の `.` は、namespace区切り文字ではなく、セレクター アクセスになります。

### A.2.6 識別子

```ebnf
identifier = ( letter | "_" ) { letter | digit | "_" | "'" } ;
label_identifier = identifier ;
letter     = "a"..."z" | "A"..."Z" ;
digit      = "0"..."9" ;
```

識別子は予約語と一致してはなりません。識別子では大文字と小文字が区別されます。形状が識別子の文法と登録ユーザーのシンボルの両方に一致するtokenは、アクティブな辞書に表示される場合に限り、シンボルとして分類されます。

ラベルは`label_identifier`を使用します。これらは識別子tokenの形状を共有していますが、§16.4.2 および項目固有の章で説明されているラベルnamespaceを占有します。

### A.2.7 数値と文字列リテラル

#### 数字

```ebnf
numeral = digit { digit } ;
```

浮動小数点、ブール値、またはその他のリテラル型は組み込まれていません。非整数値はlibrary用語としてエンコードされます。

#### 文字列リテラル

```ebnf
string_literal  = dq_string | sq_string ;
dq_string       = '"' { dq_char | escape_seq } '"' ;
sq_string       = "'" { sq_char | escape_seq } "'" ;
dq_char         = ? any character except '"' or '\' ? ;
sq_char         = ? any character except "'" or '\' ? ;
escape_seq      = "\" ( '"' | "'" | "\" ) ;
```

**コンテキスト認識**: `"` および `'` は、**文字列リテラル引数を必要とする文法の位置でのみ**、文字列区切り文字としてtoken化されます。ドキュメント コメントの外側では、すべての文字列リテラルは、指定されたフォーム内の `(` または `,` の直後に表示されます。他のすべての位置では、識別子またはユーザー定義シンボルの一部として通常の字句解析に参加します。特に、後置逆演算子 `f"` (§11) は、文字列区切り文字ではなく、`"` のユーザー シンボルの使用です。

現在文字列リテラルが必要な文法位置:

|ポジション |参考資料 |
|---|---|
| `infix_operator(STRING, ...)`、`prefix_operator(STRING, ...)`、`postfix_operator(STRING, ...)` — 最初の引数 | §10.7、§13 |
| `@latex(STRING)` およびその他の文字列値の注釈引数 | §21 |

### A.2.8 fileとmoduleの命名

* fileは `.miz` で終わります。
* 各fileは 1 つのmoduleを定義します。module名は拡張子なしのfile名と同じです。
* namespaceは、パッケージの `src/` ルートを基準としたfileのパスから派生します。パッケージ名 (`mizar.pkg` から) はnamespaceのルートです。 `src/` の下の各中間ディレクトリは、1 つの点線コンポーネントを提供します。 [§23.3](./23.package_management_and_build_system.md#233-workspace-layout)を参照してください。

例 — パッケージ `algebra`、file `algebra/src/groups/basic.miz`:

```
Module name:  basic
Namespace:    algebra.groups.basic
```

### A.2.9 コメントと注釈

```ebnf
line_comment   = "::"  { character - newline } newline ;
block_comment  = "::=" { character } "=::" ;
doc_comment    = ":::" { character - newline } newline ;

annotation_name  = "@" identifier ;
library_annot    = "@[" label_name { "," label_name } "]" ;
label_name       = label_identifier [ "(" annotation_args ")" ] ;
annotation_args  = annotation_arg { "," annotation_arg } ;
annotation_arg   = identifier | numeral | string_literal ;
```

* コメントは解析前に削除されます (§A.2.10)。
* `@` の直後には識別子が続く必要があります (空白は不可)。
* 注釈名には `snake_case` が使用されます。レジストリは固定されており、インポートによって拡張できません。
* 3 つの注釈コンテキスト: ステートメント レベル、`:::` コメント内のドキュメント タグ、および括弧形式のlibrary参照 `@[...]`。

### A.2.10 レクサー/parserの責任分割

字句解析と解析は次のように分割されます (§2 の説明を参照)。

|懸念事項 |レイヤー |
|---|---|
|コメントの削除 |前処理 |
|import prelude scan and import resolution |前処理 (final tokenization 前) |
|予約語 / 予約された特殊記号 |レクサー |
|ユーザー記号認識 (アクティブな辞書、最長一致) |レクサー |
|数字認識 |レクサー |
| `"` / `'` 文字列リテラル認識 |parser支援 (文字列が必要な位置でのみ有効) |
| `.` ロールの割り当て (セレクター vs namespace vs 複合 vs ユーザー関手) |parser + ネームリゾルバー |
|namespaceパスの変数シャドウイング |ネームリゾルバー |

レクサーは均一なtoken ストリームを発行します。 `"`、`'`、および `.` のコンテキスト依存の解釈は、アクティブなレキシコンと現在のスコープに対してparserとネーム リゾルバーによって実行されます。


## A.3 タイプシステム

規格参照: [第3章 (型システム)](./03.type_system.md)。 Mizar は、型なし集合理論を層にした **ソフト型システム** を使用します。型はチェックと可読性をガイドしますが、論理コアについては消去されます。

### A.3.1 タイプカテゴリ

|カテゴリー |役割 | | で定義
|---|---|---|
|基数型 |型階層のルート — 組み込み (`object`、`set`) およびユーザー定義構造体 | §A.3.3、第 5 章 |
|モードの種類 |名前付きタイプ。それぞれは `attribute_chain radix_type` に展開します。第7章 |
|属性 |型を絞り込む述語 |第6章 |
|クラスター |型推論の登録メカニズム |第17章 |

基数型とモード型は 2 つの素な構文カテゴリを形成し、どちらも型式の先頭で使用できます。

### A.3.2 型式の文法

```ebnf
type_expression   = attribute_chain type_head ;
type_head         = radix_type | mode_type ;
attribute_chain   = { [ "non" ] attribute_ref } ;
attribute_ref     = [ param_prefix ] [ struct_name "." ] attribute_name ;
param_prefix      = parameter "-" | "(" parameter_list ")" "-" ;

radix_type        = builtin_type | struct_name [ type_args ] ;
mode_type         = mode_name [ type_args ] ;

type_args         = ( "of" | "over" ) argument_list ;
argument_list     = term_expression { "," term_expression } ;

builtin_type      = "object" | "set" ;
attribute_name    = qualified_symbol ;   (* registered by Ch.6 *)
mode_name         = qualified_symbol ;   (* registered by Ch.7 *)
struct_name       = qualified_symbol ;   (* registered by Ch.5 *)
```

* `qualified_symbol` は、第 12 章 §12.7 で `{ namespace_segment "." } user_symbol` として定義されており、`namespace_segment` は `identifier` です。 `Group` や `std.algebra.Group` など、裸の形式とnamespace修飾された形式の両方をサポートします。
* `struct_name "." attribute_name` は、構造体修飾された属性参照です。チェーン内のすべての `.` は、§A.2.5 (ドットの曖昧さ回避、規則 4) のnamespace区切り規則に従い、スコープ内の変数によってシャドウされたセグメントはセレクター アクセスとして再解釈されます。
* `qualified_symbol` (§A.2.5.2) の末尾の `user_symbol` は、定義章 (Ch.5 構造体 / Ch.6 属性 / Ch.7 モード) によって登録された、アクティブな辞書内に存在する必要があります。
* `parameter`、`parameter_list`は18章で定義されたtemplateパラメータです。
* `type_args` と `param_prefix` には、Ch.18 で導入された代替ブラケット形式のプロダクションがあります。これらの形式は上記の形式を拡張しますが、置き換えるものではありません。

### A.3.3 組み込み型

|タイプ |説明 |
|---|---|
| `object` |ユニバーサル タイプ — Mizar ユニバースのすべての値 (構造体を含む) |
| `set` | ZFC スタイルの数学セット。 `object` のサブタイプ |

`struct` で定義された型は、`object` のサブタイプですが、`set` のサブタイプではありません。

### A.3.4 サブ型の意味論

「`S` は `T` のサブタイプです」は、`S` のすべてのメンバーが `T` のメンバーであり、FOL では `∀x. is_S(x) ⇒ is_T(x)` としてエンコードされることを意味します。 (スーパータイプへの) 拡張は自動的に行われます。 (サブタイプに) 絞り込むには、証明義務のある `reconsider` (Ch.15) が必要です。

ATP にエクスポートするときに型は消去されます。変数宣言は型なしになり、型の仮定は仮説になり、属性チェーンは結合になります。
