# 二言語ドキュメント同期監査

> 正本は英語です。英語版:
> [../en/bilingual_documentation_synchronization.md](../en/bilingual_documentation_synchronization.md)。

状態: parser task 44 で完了し、parser task 45 後に refresh 済み。

## 範囲

この監査は、`doc/design/mizar-parser/en/` の各英語正本ドキュメントと、
`doc/design/mizar-parser/ja/` の日本語 companion を比較する。

対象は、source/spec correspondence が成立した後でも drift しうる
ドキュメント面である。

- public API list、parser transfer type、forward-compatibility policy note。
- module と task の状態。completed、pending、deferred、follow-up record を含む。
- grammar surface、Pratt parsing、recovery、frontend seam、syntax-event output、
  active parse-only corpus evidence、reserved-word coverage の用語。
- 英語正本ドキュメントと日本語 companion ドキュメントへのリンク。
- syntax-only parsing、recovery ownership、parser/frontend boundary と
  parser/syntax boundary、determinism、fuzz robustness、module-boundary cleanup、
  task 43 の source/spec correspondence に関する挙動の約束。

この監査は task 43 の
[source/spec 対応監査](./source_spec_audit.md) を置き換えるものではない。
task 43 は source、spec、test、reserved-word coverage の traceability を確認した。
この task は、英語と日本語の読者に同じ implementation-facing commitment が
提示されていることを確認する。

## 結果

- 英語正本の parser docs と日本語 companion の間に、public API、parser transfer
  type、enum-policy、behavior-promise の drift は残っていない。
- module と task の状態は同期済みである。parser task 1-45 は完了済みであり、
  task 46 は concrete operator declaration と operator 予約語 corpus coverage の
  ために明示的に deferred のままである。
- `ParseRequest`、`ParserToken`、`ParseOutput`、`OperatorFixityEntry`、
  `StringRequiredContext`、`SurfaceAst`、syntax-event output、Pratt metadata、
  recovery node、`ReservedWord` token coverage、parser-deferred reserved word、
  parser-owned module boundary の用語は同期済みである。
- link policy は同期済みである。英語正本ファイルは英語正本ドキュメントへリンクする。
  日本語 companion ファイルは英語正本の parser file へ戻り、それ以外では日本語
  companion target が存在する場合にそちらを優先する。新しい cross-cutting audit docs
  は companion link を双方向に明示する。
- syntax-only parsing、resolver / build-system 依存を持たないこと、source-order
  preservation、active metadata に基づく Pratt lookup、formula precedence、
  recovery synchronization、unrecoverable stray `end`、frontend passthrough、
  parser/syntax boundary ownership、deterministic output、fuzz robustness、task 43
  の reserved-word coverage について、挙動の約束は同期済みである。
- この監査で見つけて閉じたのは documentation `design_drift` のみである。parser
  status と index text は task 44 を pending として扱ったままであり、parser audit
  list はこの bilingual audit を含んでおらず、英語 task 43 TODO には重複した
  "recorded as" 句が残っていた。source、test、specification、expectation、
  `spec_gap`、`test_gap`、`test_expectation_drift`、`boundary_violation`、
  `repo_metadata_conflict` の finding は新たに発生していない。
- 未同期の日本語 companion gap は残っていない。

## ペア別チェックリスト

| 英語正本 | 日本語 companion | 同期状態 |
|---|---|---|
| [README.md](../en/README.md) | [README.md](./README.md) | crate boundary、task 45 までの parser status、audit list、task 46 deferred state を同期済み。 |
| [grammar.md](../en/grammar.md) | [grammar.md](./grammar.md) | grammar inventory、syntax-only responsibility、`ParserTokenKind` public enum policy note、current grammar-surface status、deferred operator-declaration wording を同期済み。 |
| [pratt.md](../en/pratt.md) | [pratt.md](./pratt.md) | term Pratt、formula Pratt、active metadata、associativity、cache-key boundary、public enum compatibility promise を同期済み。 |
| [recovery.md](../en/recovery.md) | [recovery.md](./recovery.md) | recovery responsibility、synchronization policy、diagnostic ownership、task 37 consolidation status、public enum compatibility promise を同期済み。 |
| [source_spec_audit.md](../en/source_spec_audit.md) | [source_spec_audit.md](./source_spec_audit.md) | task 43 の public API trace、behavior trace、reserved-word guard、parser-deferred reserved-word list、task 46 follow-up classification を同期済み。 |
| [bilingual_documentation_synchronization.md](../en/bilingual_documentation_synchronization.md) | [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md) | この task 44 監査が二言語同期の結果を両言語で記録する。 |
| [todo.md](../en/todo.md) | [todo.md](./todo.md) | task status と follow-up record は task 45 まで同期済み。task 46 は deferred のまま両言語で残る。 |

## リンク方針

英語正本ファイルは、英語正本の spec / design / test ドキュメントへリンクする。
日本語 companion ファイルは、各ファイル冒頭で英語正本の `mizar-parser` ファイルへ
戻り、それ以外では companion が存在する場合に日本語 companion リンクを優先する。
新しい cross-cutting audit docs は companion link を双方向に明示する。参照先の
source of truth が英語のみである箇所、または英語正本の authority そのものを述べる
箇所では、意図的に英語正本へのリンクを維持する。

directory-level parser README、top-level design README、top-level roadmap は、
file-by-file の日本語 companion ではなく英語 index document である。この task は、
それらの parser status と audit link を更新し、paired English/Japanese parser docs
の summary と一致させた。

## Follow-up 記録

task 44 は新しい implementation、test、specification follow-up を作らなかった。
変更は上記の documentation `design_drift` を閉じることだけである。

task 45 はその後、public enum forward-compatibility policy follow-up を完了した。
新しい implementation、test、specification follow-up は作らず、既存の parser
lint-policy guard がすべての public parser enum を分類していることを確認した。
task 46 は concrete operator declaration と operator 予約語 corpus coverage の
deferred task として残る。
