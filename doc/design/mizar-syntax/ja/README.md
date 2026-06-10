# mizar-syntax

> 正本は英語です。英語版: [../en/README.md](../en/README.md)。

`mizar-syntax` は、構文解析済みの Mizar Evo ソースを表す `SurfaceAst` 境界を定義する。

この crate は、パーサー、リゾルバ、LSP、フォーマッタ、テストが共有できる構文データ構造を提供する。ただし、安定した公開アーティファクトスキーマではなく、コンパイラ内部データである。

状態: task 12 の最小 crate は、frontend parser-seam 統合に必要な `SurfaceAst`、ソース順を保持するノード、構文診断、回復ノードを定義している。完全な trivia と回復モデルは引き続き計画中である。

初期のモジュール仕様:

- `ast.md`
- `trivia.md`
- `recovery.md`

実装ロードマップ: [todo.md](./todo.md)。
