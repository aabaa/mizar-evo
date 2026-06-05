# mizar-parser

> 正本は英語です。英語版: [../en/README.md](../en/README.md)。

`mizar-parser` は Mizar Evo の構文文法を実装する。

依存境界は狭く保つべきである。入力は parser-facing token transfer object、出力は `SurfaceAst` と構文診断とする。パーサー補助付き字句解析は、文字列が必要な位置やシンボル種別フィルタのような、明示的なコンテキストオブジェクトを通じてのみ許可する。

状態: task 11 の最小 crate は、session `SourceRange` を持つ frontend 適合済み token transfer object を消費する parser entry point を公開し、`mizar_syntax::SurfaceAst` と構文診断を返す。token 順と範囲を保持し、小さな Pratt parser によって明示的な演算子 fixity を検証する。完全な文法範囲と回復は引き続き計画中である。

初期のモジュール仕様:

- `grammar.md`
- `pratt.md`
- `recovery.md`
