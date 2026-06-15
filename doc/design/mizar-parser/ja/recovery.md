# mizar-parser: Recovery

> 正本は英語です。英語版: [../en/recovery.md](../en/recovery.md)。

状態: task 12 の最小回復と task 28 の入れ子 block-end 回復は実装済みで、task 1
の module split と task 2 の cursor / diagnostic / synchronization helper が内部
`recovery` module に配線済み。完全な文法回復は計画中。

## 目的

このモジュールは、パーサーの同期点と回復方針を定義する。

## 責務

- `;`、`end`、トップレベル項目のキーワード、EOF などの安定した境界で同期する。
- 回復可能な構文構造を保持しながら、構文診断を出力する。
- 意味論的な事実を捏造せず、`mizar-syntax` の回復ノードを生成する。

現在の挙動:

- parser は private な有界先読み token cursor、既存の `SyntaxDiagnosticCode`
  variant を再利用する期待トークン診断 helper、同期集合、recovery node 送出
  helper を持つ。これらは内部基盤であり、crate root の公開 API を変更しない。
- 初期同期集合は `;`、`end`、EOF、および task 2 の top-level item keyword
  placeholder で停止する。この placeholder は `theorem`、`definition`、
  `registration`、`notation`、`scheme`、`reserve`、`begin`、`environ`、
  `vocabularies`、`constructors`、`requirements` である。後続の item 文法タスクが
  実際の top-level dispatch を追加するときに、この集合を拡張または絞り込む。
- 利用可能な `end` token を対応付けた後も parser の block stack が開いている場合、block 風キーワードに対する `end` 欠落を EOF で診断し、各欠落 close に明示的な recovered `MissingEnd` node を作る。現在の stack は top-level block と、それ自身の `end` を持つ algorithm control block を含む。`for` は formula quantifier が block end を消費しないように、`for <identifier> = ...` / `for <identifier> in ...` の loop 風 token shape の場合だけ開く。`else if` は nested block opener ではなく、1 つの conditional chain として扱う。
- 合成の文字列必須 parser context で文字列リテラルが欠落した場合に診断し、明示的な recovered `MissingStringLiteral` node を作る。
- 対応する block opener を持たない裸の `end` は、構文診断とともに `ast = None` を返す。

## 公開 enum の互換性

`StringRequiredContext` は downstream crate 向けに `#[non_exhaustive]` とする。現在の
parser behavior は `None` と合成の `UniformForTest` context だけを区別するが、実際の
grammar growth では operator declaration と annotation argument の parser-facing
string-required position が追加される。downstream match は wildcard fallback arm を
持たなければならない。一方、`mizar-parser` 内部の match は exhaustive のままにし、
新しい context が追加されたときに recovery と token adaptation の更新がローカルに
強制されるようにする。
