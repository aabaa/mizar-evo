# mizar-syntax

> 正本は英語です。英語版: [../en/README.md](../en/README.md)。

`mizar-syntax` は、構文解析済みの Mizar Evo ソースを表す `SurfaceAst` 境界を定義する。

この crate は、パーサー、リゾルバ、LSP、フォーマッタ、テストが共有できる構文データ構造を提供する。ただし、安定した公開アーティファクトスキーマではなく、コンパイラ内部データである。

状態: この crate は、rowan-backed `SurfaceAst`、typed 互換 view、決定的な
snapshot rendering、構文診断、syntax-owned trivia side table、task-35 surface
vocabulary、parser task 36 と pair した task 22 の predicate redefinition label
follow-through、task 24 の private AST source split を所有している。Parser Task 48 は
post-exit `PropertyImplementation` vocabulary increment として、append-only
`SyntaxKind` 192、対応する surface node/accessor/snapshot/raw-kind/rowan contract、
active parse-only pass/fail evidence を追加する。これは `SPEC-07-PI-PLACEMENT` に基づく
syntax-only coverage であり、Task-39 property semantics と S-021 rustdoc summary は
deferred のままである。

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
