# 二言語ドキュメント同期監査

> 正本は英語です。英語版: [../en/bilingual_documentation_synchronization.md](../en/bilingual_documentation_synchronization.md)。

状態: task 17 で完了。

## 範囲

この監査は、`doc/design/mizar-frontend/en/` の各英語正本ドキュメントと、
`doc/design/mizar-frontend/ja/` の日本語 companion を比較する。

対象は、実装がすでに source/spec 対応済みになった後でも独立に drift しうる
ドキュメント面である。

- 公開 API 一覧と再エクスポートされた型名。
- エラー／診断 variant。
- タスク状態と follow-up 記録。
- フロントエンドフェーズ、seam、回復境界の用語。
- 英語正本ドキュメントと日本語 companion ドキュメントへのリンク。
- 挙動の約束。特に、回復可能診断と hard failure の境界、および
  parser-assisted lexing の gate。

この監査は task 16 の
[source/spec 対応監査](./source_spec_correspondence.md) を置き換えるものではない。
task 16 はソース、仕様、テストの traceability を確認した。この task は、英語と
日本語の読者に同じ約束が提示されていることを確認する。

## 結果

- 英語正本のモジュール仕様と日本語 companion の間に、公開 API または
  エラー／診断 variant の drift は残っていない。
- モジュールとタスク状態は同期済みである。task 1-17 はこの監査時点で完了し、
  task 18 は現在完了済みであり、follow-up task 19-24 は未着手のままである。
- `SourceUnit`、`PreprocessedSource`、`ImportStub`、
  `ActiveLexicalEnvironment`、`TokenStream`、parser seam、
  `FrontendOutput`、回復可能診断、hard `FrontendError`、
  parser-assisted lexing gate の用語は同期済みである。
- 日本語 companion のリンクは、companion が存在する architecture/spec/module
  ドキュメントについて日本語側を優先するよう同期した。一方、参照先の正本が
  意図的に英語である箇所は英語正本へのリンクを維持した。
- loading-map preservation、粗い raw-scan recovery、provider provenance
  validation、有界な conflict retry、構造化 lexing payload の保持、
  `ast = None` parser recovery、安定した診断統合順序、捏造 range を使わない
  source-load location、resident-set の `ModuleLexicalSummary` 境界について、
  挙動の約束は同期済みである。
- 未同期の日本語 companion gap は残っていない。

## ペア別チェックリスト

| 英語正本 | 日本語 companion | 同期状態 |
|---|---|---|
| [README.md](../en/README.md) | [README.md](./README.md) | モジュール索引、crate 境界、状態ラベル、コンテキストリンクを同期済み。 |
| [span_bridge.md](../en/span_bridge.md) | [span_bridge.md](./span_bridge.md) | 公開 API、identity-loading behavior、composite/degraded mapping、registry invariant、エラー surface を同期済み。 |
| [source.md](../en/source.md) | [source.md](./source.md) | 公開 API、診断表示パス方針、loading-map preservation、エラー伝播、制約を同期済み。 |
| [preprocess.md](../en/preprocess.md) | [preprocess.md](./preprocess.md) | 公開 API、comment/doc-comment handling、import stub、粗い raw import recovery、診断、annotation gate 用語を同期済み。 |
| [lexical_env.md](../en/lexical_env.md) | [lexical_env.md](./lexical_env.md) | provider seam API、provenance validation、import canonicalization、conflict retry、malformed-summary boundary、cache fingerprint、resident-set links を同期済み。 |
| [lexing.md](../en/lexing.md) | [lexing.md](./lexing.md) | token stream API、scope view API、payload variant、two-pass contextual skeleton behavior、raw-scan recovery、parser-context gate を同期済み。 |
| [parsing.md](../en/parsing.md) | [parsing.md](./parsing.md) | parser input API、seam API、stub/real parser behavior、Pratt/fixity coverage、parser recovery、task 20 gate を同期済み。 |
| [orchestration.md](../en/orchestration.md) | [orchestration.md](./orchestration.md) | frontend API、diagnostic class、source-load location、merge order、hard-error boundary、syntax pass-through、output constraints を同期済み。 |
| [source_spec_correspondence.md](../en/source_spec_correspondence.md) | [source_spec_correspondence.md](./source_spec_correspondence.md) | task 16 の監査文は、bilingual wording review を未完のまま残さず、この完了済み task 17 監査を指すよう同期済み。 |
| [todo.md](../en/todo.md) | [todo.md](./todo.md) | タスク状態と follow-up 記録を同期済み。task 17 はこの監査結果とともに完了済み。 |

## リンク方針

英語正本ファイルは、英語正本の architecture/spec/module ドキュメントへリンクし、
各ファイル冒頭で日本語 companion へリンクする。日本語 companion ファイルは、
各ファイル冒頭で英語正本の mizar-frontend ファイルへ戻り、それ以外では
companion が存在する場合に日本語 companion リンクを優先する。英語正本の判断
そのものを参照している箇所、または参照先が英語のみの正本である箇所では、
意図的に英語正本へのリンクを維持する。

## Follow-up 記録

この監査では新しい follow-up task は追加していない。task 18 はその後完了した。
既存の未着手 follow-up は次のとおり。

- Task 19: incremental cache-key wiring。
- Task 20: parser-assisted lexing contract finalization。
- Task 21: durable lint enforcement。
- Task 22: precise raw-scan recovery contract。
- Task 23: lexical environment の resident-set contract guard。
- Task 24: reserved frontend diagnostic surface coverage。
