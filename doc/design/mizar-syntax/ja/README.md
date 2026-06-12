# mizar-syntax

> 正本は英語です。英語版: [../en/README.md](../en/README.md)。

`mizar-syntax` は、構文解析済みの Mizar Evo ソースを表す `SurfaceAst` 境界を定義する。

この crate は、パーサー、リゾルバ、LSP、フォーマッタ、テストが共有できる構文データ構造を提供する。ただし、安定した公開アーティファクトスキーマではなく、コンパイラ内部データである。

状態: 表現基盤は、frontend parser-seam 統合に必要な rowan-backed `SurfaceAst`、typed
互換 view、決定的な snapshot rendering、構文診断、syntax-owned trivia side table
を所有している。frontend parser-seam 統合に十分な回復ノードと拡張済みの
recovery 語彙は利用可能であり、parser producer は段階的に追加される。

初期のモジュール仕様:

- `ast.md`
- `trivia.md`
- `recovery.md`

文法ゲートの計画ノート:

- `grammar_audit.md`
- `parse_only_acceptance_matrix.md`
- `parse_only_fixture_seed.md`

実装ロードマップ: [todo.md](./todo.md)。
