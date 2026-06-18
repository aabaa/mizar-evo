# 二言語ドキュメント同期監査

> 正本は英語です。英語版:
> [../en/bilingual_documentation_synchronization.md](../en/bilingual_documentation_synchronization.md)。

状態: predicate redefinition label follow-up audit の後、S-023 まで完了。

## 範囲

この監査は、`doc/design/mizar-syntax/en/` の各英語正本ドキュメントと、
`doc/design/mizar-syntax/ja/` の日本語 companion を比較する。task 22 が
parser/syntax predicate redefinition label contract を変更した後、S-023 として
再実行した。

対象は、source/spec correspondence が成立した後でも drift しうる
ドキュメント面である。

- public API 名、builder / accessor method、enum 名、diagnostic variant。
- module と task の状態。completed、deferred、follow-up record を含む。
- rowan-backed `SurfaceAst`、typed compatibility view、trivia side table、
  recovery vocabulary、deterministic snapshot、parser/syntax responsibility
  boundary の用語。
- 英語正本ドキュメントと日本語 companion ドキュメントへのリンク。
- syntax-only representation、recovery / trivia ownership、raw-kind
  compatibility、identity / reuse rule、parser task pairing、source/spec
  correspondence、rustdoc deferral に関する挙動の約束。

この監査は S-019 の
[source/spec 対応監査](./source_spec_correspondence.md) を置き換えるものではない。
S-019 は source、spec、test traceability を確認した。この task は、英語と
日本語の読者に同じ implementation-facing commitment が提示されていることを
確認する。

## 結果

- 英語正本の module spec と日本語 companion の間に、public API、enum、
  diagnostic、method-level の drift は残っていない。
- module と task の状態は同期済みである。S-001 から S-020 と S-022 から
  S-023 は完了済み、S-021 は明示的に deferred のままであり、
  `mizar-syntax` と pair された parser task 4-36 は完了済みで、tasks 24-25 は
  pending follow-up のまま残る。既存 follow-up の `MSYN-GAP-001`、
  `MSYN-GAP-003`、`MSYN-GAP-013` は分類済みのまま残る。
- `SurfaceAst`、`SurfaceAstBuilder`、`SurfaceNodeView`、`SyntaxKind`、
  `SurfaceNodeKind`、`SurfaceTokenKind`、`SurfaceTrivia`、
  `SurfaceTriviaBuilder`、`SyntaxRecoveryKind`、`SyntaxDiagnostic`、
  rowan-backed green-tree storage、deterministic snapshot、typed
  compatibility view、parser task pairing、syntax-only semantic boundary の
  用語は同期済みである。
- 日本語 companion のリンクは、companion が存在する spec / design / test
  ドキュメントについて日本語側を優先するよう同期した。各ファイル冒頭の英語
  正本への戻りリンクと、`doc/spec/en/` への authority reference は意図的に
  英語のまま維持する。
- syntax-only data structure、生 storage を semantic identity contract としない
  こと、source ownership と sorted trivia rendering、recovery-node vocabulary と
  active producer status、この phase の append-only raw-kind numbering、
  persistent identity ではない `SurfaceNodeId`、range / snapshot または green-node
  equality による reuse validation、parser/syntax task pairing、source/spec
  correspondence、S-021 rustdoc deferral について、挙動の約束は同期済みである。
- S-020 で見つかった drift は documentation `design_drift` のみである。二言語
  文書セットがまだ S-020 を pending と記述しており、日本語 companion の一部リンクが
  日本語 target の存在する箇所でも英語 companion target を指していた。この task は
  source を変更せずにその drift を閉じた。
- source/test mismatch、新しい `spec_gap`、新しい `test_gap`、
  `test_expectation_drift`、`boundary_violation`、`repo_metadata_conflict` は
  見つからなかった。
- 未同期の日本語 companion gap は残っていない。
- S-023 の再監査では、task 22 後に残っていた documentation `design_drift` を
  見つけて閉じた。parser README と top-level roadmap の status text はまだ
  parser task 36 / syntax task 22 を pending として扱っており、syntax README /
  audit record も S-019/S-020 を最新の対応結果として提示し、historical
  crate-exit report には現在の follow-up status note が必要だった。英語と日本語
  companion は、parser task 36 と syntax task 22-23 が完了済み、S-021 が
  deferred、syntax tasks 24-25 が active refactor follow-up として pending で
  あることについて同期済みである。

## ペア別チェックリスト

| 英語正本 | 日本語 companion | 同期状態 |
|---|---|---|
| [README.md](../en/README.md) | [README.md](./README.md) | module index、crate boundary、S-023 までの status label、cross-cutting audit link を同期済み。 |
| [00.crate_plan.md](../en/00.crate_plan.md) | [00.crate_plan.md](./00.crate_plan.md) | crate responsibility、specification/test reference、parser task pairing、gap classification、task decomposition、S-020 result、exit criteria を同期済み。 |
| [ast.md](../en/ast.md) | [ast.md](./ast.md) | public API、rowan storage boundary、task 35 までの syntax vocabulary と task 22 の predicate redefinition label follow-through、compatibility view policy、raw-kind policy、identity/reuse rule、task status を同期済み。 |
| [trivia.md](../en/trivia.md) | [trivia.md](./trivia.md) | public API、trivia side-table ownership、sorting、attachment、snapshot behavior、parser/frontend responsibility boundary を同期済み。 |
| [recovery.md](../en/recovery.md) | [recovery.md](./recovery.md) | public API、recovery kind、diagnostic code、active / vocabulary-only producer status、malformed annotation recovery、source/test evidence を同期済み。 |
| [grammar_audit.md](../en/grammar_audit.md) | [grammar_audit.md](./grammar_audit.md) | grammar gate finding、parser task map、gap classification、close-out status を同期済み。 |
| [parse_only_acceptance_matrix.md](../en/parse_only_acceptance_matrix.md) | [parse_only_acceptance_matrix.md](./parse_only_acceptance_matrix.md) | acceptance category、active/deferred status、grammar-position reference、parser-facing ownership note を同期済み。 |
| [parse_only_fixture_seed.md](../en/parse_only_fixture_seed.md) | [parse_only_fixture_seed.md](./parse_only_fixture_seed.md) | seed fixture intent、activation rule、deferred row、parser ownership reference を同期済み。 |
| [source_spec_correspondence.md](../en/source_spec_correspondence.md) | [source_spec_correspondence.md](./source_spec_correspondence.md) | S-019 と S-023 の source/spec/test correspondence、public API と method traceability、follow-up record、この bilingual audit への handoff を同期済み。 |
| [bilingual_documentation_synchronization.md](../en/bilingual_documentation_synchronization.md) | [bilingual_documentation_synchronization.md](./bilingual_documentation_synchronization.md) | この S-020 / S-023 監査が二言語同期の結果を両言語で記録する。 |
| [crate_exit_report.md](../en/crate_exit_report.md) | [crate_exit_report.md](./crate_exit_report.md) | historical task-35 exit status、quality score、deferred item、verification result、task 24-25 が final close-out を refresh するという note を同期済み。 |
| [todo.md](../en/todo.md) | [todo.md](./todo.md) | task status と follow-up record は S-023 まで同期済み。S-021 は両言語で deferred のままであり、tasks 24-25 は pending のまま。 |

## リンク方針

英語正本ファイルは、英語正本の spec / design / test ドキュメントへリンクし、
各ファイル冒頭で日本語 companion へリンクする。日本語 companion ファイルは、
各ファイル冒頭で英語正本の `mizar-syntax` ファイルへ戻り、それ以外では
companion が存在する場合に日本語 companion リンクを優先する。参照先の source
of truth が英語のみである箇所、または英語正本の authority そのものを述べる
箇所では、意図的に英語正本へのリンクを維持する。

## Follow-up 記録

S-020 は新しい implementation、test、specification follow-up を作らなかった。
変更は上記の documentation `design_drift` を閉じることだけである。

S-021 は、フロントエンドパイプライン外の長命な consumer が `mizar-syntax` に
対してコーディングを始めるか、workspace が rustdoc policy を採用するまで、明示的に
rustdoc summary の deferred task として残る。どちらか早い方を再着手 trigger とする。

最終 crate exit task は、S-020 audit result を変更せず、同期済みの
[crate_exit_report.md](./crate_exit_report.md) companion を追加した。

S-023 は新しい implementation、test、specification follow-up を作らなかった。
task 22 の predicate-label repair 後に status text が遅れていた documentation
`design_drift` のみを閉じた。
