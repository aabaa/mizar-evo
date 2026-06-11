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
  parser-assisted lexing-plan boundary。

この監査は task 16 の
[source/spec 対応監査](./source_spec_correspondence.md) を置き換えるものではない。
task 16 はソース、仕様、テストの traceability を確認した。この task は、英語と
日本語の読者に同じ約束が提示されていることを確認する。

## 結果

- 英語正本のモジュール仕様と日本語 companion の間に、公開 API または
  エラー／診断 variant の drift は残っていない。
- モジュールとタスク状態は同期済みである。task 1-29 は完了済みであり、
  task 29 は frontend-owned の real-parser fuzz follow-up を完了した。parser-owned
  側は `mizar-parser` task 39 で引き続き追跡する。
- `SourceUnit`、`PreprocessedSource`、`ImportStub`、
  `ActiveLexicalEnvironment`、`TokenStream`、parser seam、
  `FrontendOutput`、frontend content cache keys、回復可能診断、hard `FrontendError`、
  parser-assisted lexing plan の用語は同期済みである。
- 日本語 companion のリンクは、companion が存在する architecture/spec/module
  ドキュメントについて日本語側を優先するよう同期した。一方、参照先の正本が
  意図的に英語である箇所は英語正本へのリンクを維持した。
- loading-map preservation、精密な recoverable raw-scan recovery、provider provenance
  validation、有界な conflict retry、構造化 lexing payload の保持、
  `ast = None` parser recovery、安定した診断統合順序、捏造 range を使わない
  source-load location、resident-set の `ModuleLexicalSummary` 境界と task 23
  の guard、task 19 の cache-key storage/computation boundary、task 20 の
  parser-assisted lexing-plan boundary、task 21 の durable lint-policy guard、
  task 22 の raw-scan recovery boundary、task 24 の予約 diagnostic surface 方針、
  task 26 の公開 API rustdoc summary 方針、task 28 の parser-growth follow-through、
  task 29 の real-parser frontend fuzz coverage について、挙動の約束は同期済みである。
- 未同期の日本語 companion gap は残っていない。

## ペア別チェックリスト

| 英語正本 | 日本語 companion | 同期状態 |
|---|---|---|
| [README.md](../en/README.md) | [README.md](./README.md) | モジュール索引、crate 境界、状態ラベル、コンテキストリンクを同期済み。 |
| [span_bridge.md](../en/span_bridge.md) | [span_bridge.md](./span_bridge.md) | 公開 API、identity-loading behavior、composite/degraded mapping、registry invariant、エラー surface を同期済み。 |
| [source.md](../en/source.md) | [source.md](./source.md) | 公開 API、診断表示パス方針、loading-map preservation、エラー伝播、制約を同期済み。 |
| [preprocess.md](../en/preprocess.md) | [preprocess.md](./preprocess.md) | 公開 API、comment/doc-comment handling、import stub、精密な recoverable raw import recovery、診断、parser-assisted string-argument handling を同期済み。 |
| [lexical_env.md](../en/lexical_env.md) | [lexical_env.md](./lexical_env.md) | provider seam API、provenance validation、import canonicalization、conflict retry、malformed-summary boundary、cache fingerprint、resident-set links を同期済み。 |
| [lexing.md](../en/lexing.md) | [lexing.md](./lexing.md) | token stream API、parser lexing-plan API、scope view API、payload variant、two-pass contextual skeleton behavior、raw-scan recovery を同期済み。 |
| [parsing.md](../en/parsing.md) | [parsing.md](./parsing.md) | parser input API、position-sensitive string context API、seam API、parser cache-key version API、stub/real parser behavior、Pratt/fixity coverage、parser recovery を同期済み。 |
| [cache_key.md](../en/cache_key.md) | [cache_key.md](./cache_key.md) | frontend content cache-key API、parser lexing-plan content key、storage boundary、invalidation rules、tests を同期済み。 |
| [orchestration.md](../en/orchestration.md) | [orchestration.md](./orchestration.md) | frontend API、`FrontendOutput.cache_keys`、diagnostic class、source-load location、merge order、hard-error boundary、syntax pass-through、output constraints を同期済み。 |
| [source_spec_correspondence.md](../en/source_spec_correspondence.md) | [source_spec_correspondence.md](./source_spec_correspondence.md) | task 16 の監査文は、task 19 の cache-key wiring、task 20 の parser-assisted lexing、task 21 の durable lint enforcement、task 22 の raw-scan recovery、task 23 の resident-set guard status、task 24 の reserved diagnostic surface coverage、task 26 の rustdoc summary coverage、task 28 の parser-growth follow-through、task 29 の real-parser frontend fuzz coverage を記録する。 |
| [todo.md](../en/todo.md) | [todo.md](./todo.md) | タスク状態と follow-up 記録は task 29 まで同期済み。 |

## リンク方針

英語正本ファイルは、英語正本の architecture/spec/module ドキュメントへリンクし、
各ファイル冒頭で日本語 companion へリンクする。日本語 companion ファイルは、
各ファイル冒頭で英語正本の mizar-frontend ファイルへ戻り、それ以外では
companion が存在する場合に日本語 companion リンクを優先する。英語正本の判断
そのものを参照している箇所、または参照先が英語のみの正本である箇所では、
意図的に英語正本へのリンクを維持する。

## Follow-up 記録

task 18、task 19、task 20、task 21、task 22、task 23、task 24、task 25、
task 26、task 27、task 28、task 29 はその後完了した。task 29 は frontend-owned の
real-parser fuzz follow-up を完了した。parser-owned 側は `mizar-parser` task 39 で
引き続き追跡する。現在予約されている fallback variant に対する具体的 producer を
将来の non-exhaustive lexer/session/parser contract が公開した場合は、producer-backed
tests を追加する。
