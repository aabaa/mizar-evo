# mizar-parser

> 正本は英語です。英語版: [../en/README.md](../en/README.md)。

`mizar-parser` は Mizar Evo の構文文法を実装する。

現在のTask-46状態: concreteなinfix/prefix/postfix operator declarationを、annotation付き、
visibility付きtop-level、およびdefinition-local notation positionでdedicated
`OperatorDeclaration` nodeとしてparseする。完了済みfrontend Task 20がnamed
position-sensitive string contextとlocal operator metadata handoffをすでに提供していた。
Task 46はsyntax-onlyであり、operatorをactivateせず、`ParseRequest::operator_fixity`を
変更しない。post-Task-46 parser milestoneは9 hard gateをすべて満たし、freshな
independent read-only scoreは99/100である。

依存境界は狭く保つべきである。入力は parser-facing token transfer object、出力は `SurfaceAst` と構文診断とする。パーサー補助付き字句解析は、文字列が必要な位置やシンボル種別フィルタのような、明示的なコンテキストオブジェクトを通じてのみ許可する。

状態: この crate は、session `SourceRange` を持つ frontend 適合済み token transfer object を消費し、`mizar_syntax::SurfaceAst` と構文診断を返す parser entry point を公開している。Tasks 1-48は実装済みである。P-043-01/P-046はTask 46によりclosed、P-265-47Dはnonblocking human-owned wording gapのままである。独立にclassifiedされたoverbroad frontend string-position heuristicはparser scope外に残る。Task 46はglobal Step 5を閉じず、Task 49やSteps 6/7を許可しない。

モジュール仕様と監査:

- `00.crate_plan.md`
- `grammar.md`
- `pratt.md`
- `recovery.md`
- `source_spec_audit.md`
- `bilingual_documentation_synchronization.md`
- `crate_exit_report.md`

実装ロードマップ: [todo.md](./todo.md)。
