# mizar-parser

> 正本は英語です。英語版: [../en/README.md](../en/README.md)。

`mizar-parser` は Mizar Evo の構文文法を実装する。

依存境界は狭く保つべきである。入力は parser-facing token transfer object、出力は `SurfaceAst` と構文診断とする。パーサー補助付き字句解析は、文字列が必要な位置やシンボル種別フィルタのような、明示的なコンテキストオブジェクトを通じてのみ許可する。

状態: この crate は、session `SourceRange` を持つ frontend 適合済み token transfer object を消費し、`mizar_syntax::SurfaceAst` と構文診断を返す parser entry point を公開している。task 2 の parser 基盤は、private な cursor、構文 event、期待トークン診断、同期、recovery 送出 helper として配置済みである。parser workstream は parser task 42 まで成長し、module/import/export、type/term/formula、statement/proof、definition、structure、registration、template、algorithm、verification-clause、annotation surface、predicate redefinition label repair、recovery consolidation と fail-corpus expansion、parse-only `SurfaceAst` snapshot baseline、parser determinism coverage、parser-owned valid-UTF-8 fuzz target、frontend passthrough follow-through audit、private annotation/test module-boundary split を含む。task 43 の source/spec correspondence と reserved-word coverage audit、task 44 の bilingual documentation synchronization、task 45 の public enum policy、canonical な3つの `reconsider` tail formを整合させるtask 47は完了済みである。task 48が次のauthorized nonempty Step-5 parser taskであり、concrete operator declaration用task 46はdeferredのままである。

モジュール仕様と監査:

- `00.crate_plan.md`
- `grammar.md`
- `pratt.md`
- `recovery.md`
- `source_spec_audit.md`
- `bilingual_documentation_synchronization.md`

実装ロードマップ: [todo.md](./todo.md)。
