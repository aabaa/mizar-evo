# mizar-parser

> 正本は英語です。英語版: [../en/README.md](../en/README.md)。

`mizar-parser` は Mizar Evo の構文文法を実装する。

依存境界は狭く保つべきである。入力は `TokenStream`、出力は `SurfaceAst` と構文診断とする。パーサー補助付き字句解析は、文字列が必要な位置やシンボル種別フィルタのような、明示的なコンテキストオブジェクトを通じてのみ許可する。

初期のモジュール仕様:

- `grammar.md`
- `pratt.md`
- `recovery.md`

