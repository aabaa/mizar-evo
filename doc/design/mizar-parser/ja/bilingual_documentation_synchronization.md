# 二言語ドキュメント同期監査

> 正本は英語です。英語版:
> [../en/bilingual_documentation_synchronization.md](../en/bilingual_documentation_synchronization.md)。

状態: parser task 44で完了し、post-Task-46 parser closeoutまでrefresh済み。
pre-Task-46 `PARSER-CRATE-CLOSEOUT` entryはhistorical checkpointとしてのみ保持する。

## Task 46 pair recheck

paired plan、README、grammar、recovery、source/spec audit、TODO、本auditはexact 3
declaration form、annotation/visibility付きtop-levelとdefinition-local placement、
append-only syntax kind 193、local recovery、active pass/fail pair 1組、syntax-only
credit、unchanged Pratt/semantic behaviorで一致する。両言語はP-043-01/P-046をclosed、
従来closeoutをsuperseded、post-Task-46 closeoutをcurrentとし、Task 49やSteps 6/7を
promoteしない。

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
- module/task statusは同期済みである。parser Tasks 1-48とpost-Task-46 closeoutは
  completeし、fresh independent read-only scoreは99/100である。successor parser taskは
  authorizeされていない。
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
| [00.crate_plan.md](../en/00.crate_plan.md) | [00.crate_plan.md](./00.crate_plan.md) | Task-46 completion、current oracle、historical checkpoint labeling、post-Task-46 closeout gateを同期済み。 |
| [README.md](../en/README.md) | [README.md](./README.md) | crate boundary、Tasks 1-48 completion、syntax-only credit、residual ownership、no-successor statusを同期済み。 |
| [grammar.md](../en/grammar.md) | [grammar.md](./grammar.md) | Task 46までのgrammar inventory、`reconsider_tail`、property/operator declaration、syntax-only responsibility、enum policyを同期済み。 |
| [pratt.md](../en/pratt.md) | [pratt.md](./pratt.md) | term Pratt、formula Pratt、active metadata、associativity、cache-key boundary、public enum compatibility promise を同期済み。 |
| [recovery.md](../en/recovery.md) | [recovery.md](./recovery.md) | Task-46/47/48 recovery ownership、nested-depth synchronization、diagnostic ownership、public enum compatibility promiseを同期済み。 |
| [source_spec_audit.md](../en/source_spec_audit.md) | [source_spec_audit.md](./source_spec_audit.md) | Task-43 audit、closed Task-46/47/48 classification、reserved-word guard、syntax-only credit、residual ownershipを同期済み。 |
| [bilingual_documentation_synchronization.md](../en/bilingual_documentation_synchronization.md) | [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md) | この task 44 監査が二言語同期の結果を両言語で記録する。 |
| [todo.md](../en/todo.md) | [todo.md](./todo.md) | Tasks 1-48、current closeout、residual ownership、no-successor statusを同期済み。 |
| [crate_exit_report.md](../en/crate_exit_report.md) | [crate_exit_report.md](./crate_exit_report.md) | Post-Task-46 scope、9 hard gates、fresh 99/100 score、current oracle、external frontend finding、no-successor handoffを同期済み。 |

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
Task 46はそのcheckpointではdeferredだったが、現在はcompleteしている。

## Task 47 pair recheck

paired plan、README、grammar、recovery、source/spec audit、TODO、本auditは3つの
`reconsider_tail` form、exact AST/recovery ownership、新しいactive pass fixture 1件、newly
covered trace row 2件、unchanged semantic gate、405/369および97/97 parse-only oracleで
一致する。両言語はnonblockingなChapter-8 list-wording `spec_gap`、当時の次taskである
Task 48、deferred Task 46、deferred Steps 6/7も維持する。
Task 47にbilingual driftは残らない。

## Task 48 pair recheck

paired plan、README、grammar、recovery、source/spec audit、TODO、本auditはtop-level
placement、exact means/equals ownership、specialized mode parameter shape、nested-depth
recovery、append-only syntax kind 192、active pass/fail sidecar 2件、parse-only 99/99、
syntax-only credit、unchanged Task-39 semantic gateで一致する。Task 46とSteps 6/7は
deferredのままである。Task 48にbilingual driftは残らない。

## Post-Task-46 parser crate closeout pair recheck

paired plan/README/TODO、本audit、global index、crate exit reportはTasks 1-48がcomplete、
P-043-01/P-046がclosedで一致する。両言語は同じ9 passing hard gate、fresh independent
99/100 score、verification count/hash、P-265-47D human ownership、external/uncredited
frontend heuristic、global Step-5 exclusion、authorized successor parser taskなしを記録する。
Task 49を推定せず、Steps 6/7をpromoteしない。
`PARSER-CRATE-POST-TASK46-CLOSEOUT`後にbilingual driftは残らない。
