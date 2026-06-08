# mizar-parser: Recovery

> 正本は英語です。英語版: [../en/recovery.md](../en/recovery.md)。

状態: task 12 の最小回復は実装済み。完全な文法回復は計画中。

## 目的

このモジュールは、パーサーの同期点と回復方針を定義する。

## 責務

- `;`、`end`、トップレベル項目のキーワード、EOF などの安定した境界で同期する。
- 回復可能な構文構造を保持しながら、構文診断を出力する。
- 意味論的な事実を捏造せず、`mizar-syntax` の回復ノードを生成する。

現在の最小挙動:

- `end` token が存在しない場合、block 風キーワードに対する `end` 欠落を EOF で診断し、明示的な recovered `MissingEnd` node を作る。
- 合成の文字列必須 parser context で文字列リテラルが欠落した場合に診断し、明示的な recovered `MissingStringLiteral` node を作る。
- 1 トークンの裸の `end` ストリームは、構文診断とともに `ast = None` を返す。
