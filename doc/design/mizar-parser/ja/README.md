# mizar-parser

> 正本は英語です。英語版: [../en/README.md](../en/README.md)。

`mizar-parser` は Mizar Evo の構文文法を実装する。

依存境界は狭く保つべきである。入力は parser-facing token transfer object、出力は `SurfaceAst` と構文診断とする。パーサー補助付き字句解析は、文字列が必要な位置やシンボル種別フィルタのような、明示的なコンテキストオブジェクトを通じてのみ許可する。

状態: この crate は、session `SourceRange` を持つ frontend 適合済み token transfer object を消費し、`mizar_syntax::SurfaceAst` と構文診断を返す parser entry point を公開している。task 2 の parser 基盤は、private な cursor、構文 event、期待トークン診断、同期、recovery 送出 helper として配置済みである。grammar coverage は parser task 35 まで成長し、module/import/export、type/term/formula、statement/proof、definition、structure、registration、template、algorithm、verification-clause、annotation surface を含む。task 36 の predicate redefinition label repair と、hardening/audit task 37〜45 は引き続き計画中である。

初期のモジュール仕様:

- `grammar.md`
- `pratt.md`
- `recovery.md`

実装ロードマップ: [todo.md](./todo.md)。
