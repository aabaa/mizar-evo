# mizar-parser: Recovery

> 正本は英語です。英語版: [../en/recovery.md](../en/recovery.md)。

状態: task 12 の最小回復と task 28 の入れ子 block-end 回復は実装済みで、task 1
の module split により内部 `recovery` module として配置済み。完全な文法回復は
計画中。

## 目的

このモジュールは、パーサーの同期点と回復方針を定義する。

## 責務

- `;`、`end`、トップレベル項目のキーワード、EOF などの安定した境界で同期する。
- 回復可能な構文構造を保持しながら、構文診断を出力する。
- 意味論的な事実を捏造せず、`mizar-syntax` の回復ノードを生成する。

現在の挙動:

- 利用可能な `end` token を対応付けた後も parser の block stack が開いている場合、block 風キーワードに対する `end` 欠落を EOF で診断し、各欠落 close に明示的な recovered `MissingEnd` node を作る。現在の stack は top-level block と、それ自身の `end` を持つ algorithm control block を含む。`for` は formula quantifier が block end を消費しないように、`for <identifier> = ...` / `for <identifier> in ...` の loop 風 token shape の場合だけ開く。`else if` は nested block opener ではなく、1 つの conditional chain として扱う。
- 合成の文字列必須 parser context で文字列リテラルが欠落した場合に診断し、明示的な recovered `MissingStringLiteral` node を作る。
- 対応する block opener を持たない裸の `end` は、構文診断とともに `ast = None` を返す。
