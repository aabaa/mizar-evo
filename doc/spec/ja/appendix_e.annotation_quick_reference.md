# 付録 E. 注釈クイックリファレンス

> Canonical language: English. English canonical version: [../en/appendix_e.annotation_quick_reference.md](../en/appendix_e.annotation_quick_reference.md).

この付録は、Mizar Evolution source fileで使用される注釈フォームに関する非標準のクイック リファレンスです。標準的な構文と意味論は、[第 2 章、§2.9](./02.lexical_structure.md#29-コメントと注釈)、[第21章](./21.source_code_annotation_and_atp.md)、[第22章](./22.error_handling_and_diagnostics.md)、および [第 24 章、§24.1](./24.documentation_generation.md#241-ドキュメントのコメント構文) で定義されています。

* [E. 注釈のクイックリファレンス](#付録-e-注釈クイックリファレンス)
  * [E.1 注釈コンテキスト](#e1-注釈コンテキスト)
  * [E.2 ステートメントと項目の注釈](#e2-ステートメントと項目の注釈)
  * [E.3 ライブラリの注釈](#e3-library-の注釈)
  * [E.4 ドキュメントタグ](#e4-ドキュメントタグ)
  * [E.5 開発ガイダンス](#e5-開発ガイダンス)

## E.1 注釈コンテキスト

|コンテキスト |表面形状 |典型的な配置 |主な用途 |参照 |
|---|---|---|---|---|
|ステートメント/項目の注釈 | `@name` または `@name(...)` |注釈対象の項目または source 位置の直前 |証明ヒントとレンダリングヒント |第2章、第21章、第22章 |
|単独診断 annotation | `@show_type(expr)`, `@eval(expr)` |独立した source item として配置 |診断と検証時評価 |第21章 |
|library の注釈 | `@[label, ...]` |定義、定理、または登録の直前 |証明探索用の安定したメタデータ ラベル |第21章 |
|ドキュメントタグ | `:::` 内の `@name ...` コメント |ドキュメントコメント段落の最初のtoken |構造化された生成ドキュメント |第24章 |

注釈名は、ステートメントおよび項目の注釈の言語レジストリによって固定されます。 `:::` コメント内のドキュメント タグは、ドキュメント ジェネレーターによって処理されます。認識されないドキュメント タグは、検証器によって拒否されずにスルーされます。

## E.2 ステートメント、項目、診断形式

|注釈 |フォーム |適用対象 |効果 |
|---|---|---|---|
| LaTeX レンダリング | `@latex("...")` | `func`、`pred`、`mode`、または `attr` 定義の直前 |ドキュメントおよび IDE 表示に推奨される数学的レンダリングを提供します。 |
|証明のヒント | `@proof_hint(...)` | `by` ステップ、`thus` ステートメント、または `proof ... end` ブロックの直前 |次の証明義務の ATP 証明検索を制限または構成します。 |
|論文を表示 | `@show_thesis` |証明位置 |現在の理論を情報診断として出力します。 |
|解像度を表示 | `@show_resolution` |式の直前 |次の式のoverload解決の詳細を出力します。 |
|型を表示 | `@show_type(expr)` |単独診断 item | `expr` の推論された型をバインドせずに出力します。 |
|式を評価する | `@eval(expr)` |トップレベル、証明ブロック、またはアルゴリズム本体の単独診断 item |検証時の評価を実行し、可能な場合は計算結果を出力します。 |
|警告を抑制 | `@suppress(Wnnnn)` |項目またはサポートされている最小の警告範囲 |注釈付きスコープに対する `W0102` などの警告を抑制します。 |

`@show_type(expr)` と `@eval(expr)` は単独診断形式です。後続の宣言や文に
annotation として付与されることはありません。

`@proof_hint` は、カンマで区切られたオプションを受け入れます。

|オプション |例 |意味 |
|---|---|---|
| `max_axioms` | `@proof_hint(max_axioms: 32)` |注釈付きステップの ATP に送信される公理の数を制限します。 |
| `timeout` | `@proof_hint(timeout: 60)` |ステップごとの ATP 時間制限を秒単位で設定します。 |
| `solver` | `@proof_hint(solver: vampire)` |ステップのbackend ソルバーを選択します。 `auto` はデフォルトのポートフォリオ ポリシーを使用します。 |

例:

```mizar
@latex("\\gcd(a,b)")
func GcdDef: gcd(a, b) -> Nat means ...;

@proof_hint(max_axioms: 32, solver: vampire)
thus thesis by GroupAssoc, GroupIdentity;

@show_type(total + x)
@eval(factorial(10))
```

## E.3 library の注釈

library の注釈は、証明検索で使用するために、定義、定理、または登録に安定したラベルを付けます。

```ebnf
library_annotation ::= "@[" label_list "]" ;
label_list         ::= label_name { "," label_name } ;
label_name         ::= label_identifier [ "(" annotation_args ")" ] ;
annotation_args    ::= annotation_arg { "," annotation_arg } ;
annotation_arg     ::= identifier | nat_literal | string_literal ;
```

例:

```mizar
@[label, category("algebra")]
theorem Union_empty_right:
  X \/ {} = X by ...;

@[category("set")]
registration
  let X be set;
  reduce UnionEmpty: X \/ {} to X;
  reducibility proof ... end;
end;
```

上記のアノテーションは例示です。組み込み検証器の動作は、言語または実装によって認識されるアノテーションに対してのみ定義されます。追加のアノテーションはlibraryによって登録され、組み込み検証器によって無視される場合があります。リダクションの優先順位はアノテーションでは制御しません。自動規則選択は §17.6.4 で定義されます。

## E.4 ドキュメントタグ

ドキュメント タグは、`:::` ドキュメント コメント内に表示されます。これらは証明チェッカーではなく、`mizar doc` によって解釈されます。

|タグ |フォーム |意味 |
|---|---|---|
|パラメータ | `@param name` | `name` という名前の 1 つのパラメータを文書化します。 |
|戻り値 | `@returns` |戻り値を文書化します。 `result` は戻り値を指します。 |
|前提条件の散文 | `@requires` |前提条件を散文で説明し、形式的な `requires` を補足します。 |
|事後条件散文 | `@ensures` |散文で事後条件を説明し、形式的な `ensures` を補足します。 |
|も参照してください。 `@see ref` |別の項目、セクション、または URL への相互参照を追加します。 |
|導入バージョン | `@since version` |アイテムが導入されたバージョンを記録します。 |
|廃止 | `@deprecated version` | `version` 以降、項目を非推奨としてマークします。次のテキストがメッセージを示しています。 |

例：

```mizar
::: Divides `a` by `b` and returns the quotient.
:::
::: @param a  the dividend
::: @param b  the divisor; must be non-zero
::: @returns  the real number `a / b`
::: @requires `b <> 0`
::: @see mml.real.Real_div_mul
algorithm divide(a, b) -> Real
  requires b <> 0
  ensures result * b = a
do
  return a / b;
end;
```

## E.5 開発ガイダンス

|目標 |優先する |
|---|---|
|overloadの曖昧さを解消する |修飾名、明示的なtemplate引数、または `qua`。意味論を変更するためにアノテーションに依存しないでください。 |
|証明状態を探索する | `@show_thesis`、`@show_type`、`@show_resolution`。 |
|計算可能な値を検査する | `@eval(expr)`、式が計算可能であると予想される場合のみ。 |
|チューン証明探索 | `@proof_hint`、実用範囲が最小。 |
|レンダリングされた表記を改善する |エクスポートされたシンボルの `@latex`。 |
|パブリック API を文書化する |構造化ドキュメントタグを含む `:::` コメント。 |
|ノイズの多い診断をサイレントにする | `@suppress`、範囲が狭く、できれば一時的なもの。 |

注釈は意味的に中立です。注釈を削除すると、診断、レンダリング、証明探索のパフォーマンス、または文書化に影響する可能性がありますが、source の論理的な意味を変更してはなりません。
