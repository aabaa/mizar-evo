# mizar-syntax

> 正本は英語です。英語版: [../en/README.md](../en/README.md)。

`mizar-syntax` は、構文解析済みの Mizar Evo ソースを表す `SurfaceAst` 境界を定義する。

この crate は、パーサー、リゾルバ、LSP、フォーマッタ、テストが共有できる構文データ構造を提供する。ただし、安定した公開アーティファクトスキーマではなく、コンパイラ内部データである。

状態: この crate は、rowan-backed `SurfaceAst`、typed 互換 view、決定的な
snapshot rendering、構文診断、syntax-owned trivia side table、task-35 surface
vocabulary を所有している。S-019 source/spec audit では、module spec の public API
と behavior promise に対する欠落実装は見つからなかった。

自律 crate 開発の kickoff plan:

- [00.crate_plan.md](./00.crate_plan.md)

初期のモジュール仕様:

- `ast.md`
- `trivia.md`
- `recovery.md`

文法ゲートの計画ノート:

- `grammar_audit.md`
- `parse_only_acceptance_matrix.md`
- `parse_only_fixture_seed.md`

横断的な監査ノート:

- `source_spec_correspondence.md`
- `bilingual_documentation_synchronization.md`
- `crate_exit_report.md`

実装ロードマップ: [todo.md](./todo.md)。
