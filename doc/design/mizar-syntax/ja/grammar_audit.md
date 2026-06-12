# mizar-syntax Task 6: 正本文法の整合性監査

> 正本は英語です。英語版: [../en/grammar_audit.md](../en/grammar_audit.md)。

状態: 2026-06-12 の Task 6 文法整合性ゲートとして完了。

## 対象範囲

この監査では [Appendix A](../../../spec/ja/appendix_a.grammar_summary.md) を
parser-facing な正本文法サマリとして扱い、第 2-21 章の章別構文と照合する。
AST ノード設計には踏み込まない。Task 7 が parse-only acceptance matrix と
fixture 計画を作るときに、未解決の文法 drift を `SurfaceAst` の node kind に
固定してしまわないよう、指摘を分類して記録する。

## 方法

- Appendix A の EBNF production を抽出し、未定義 nonterminal、重複定義、
  `compilation_unit` からの到達性、文書化済み precedence parser の外にある
  直接左再帰を確認した。
- 第 2-21 章の章別 EBNF と、Appendix A の production 名および右辺を比較した。
  第 8 章と第 19 章は semantic companion chapter であり、その EBNF は第 3、
  13、15、18 章が所有する形式を再掲するものとして扱い、下で明示するものを除き
  互換用の説明とした。
- Appendix A と第 2 章の予約語・予約特殊記号リストを、文法内の quoted terminal
  と照合した。
- 各指摘を「AST 設計前に修正」「semantic-only」「Task 7 / parser task へ
  defer」に分類した。

## 機械チェック結果

Task 6 の Appendix A 仕様修正後の結果:

| 確認項目 | 結果 |
|---|---|
| 未定義 nonterminal | 指摘なし。コメント production 用の `character` を定義した。 |
| 重複 production 定義 | 指摘なし。 |
| 直接左再帰 | 文書化済みの `term_expression` / `formula` precedence parser の外では指摘なし。 |
| 予約語 drift | range loop で使う `step` が未予約だったため、Appendix A と第 2 章で予約語に追加した。 |
| 予約特殊記号 drift | 第 2 章に relative import 用 `..` を追加し、dot-compound priority にも反映した。`@` は単独の予約記号 token ではなく annotation marker として明確化した。 |
| 文脈限定 spelling drift | Appendix A で、`max_axioms`、`steps`、`timeout` などの solver / annotation option spelling は予約識別子ではなく文脈限定であると明示した。`nest` は現行の lexical contract に従い global reserved のままとする。 |
| semantic companion EBNF drift | 第 8 章と第 19 章は `qua`、`reconsider`、overload-resolution helper form を再掲する。parser-facing production の変更は不要で、Appendix A を正規化済み正本とする。 |

`compilation_unit` から未到達のまま残る production は、意図的な helper または
下記の follow-up である。

- lexical / trivia helper: `whitespace`, `tab`, `newline`, `line_comment`,
  `block_comment`, `doc_comment`, `character`。
- 互換 alias: `term`, `expr`, `proof_block`, `symbol_name`, `let_decl`,
  `set_decl`, `reconsider_decl`, `reconsider_target`, `qualified_vars`。
- 実際の構文は別 production から到達する parser-normalized helper:
  `field_access` は `term_postfix`、`qua_expression` は `term_expression`、
  `pick_expr` は `choice_expression`、`mode_application` は `type_expression`
  から到達する。
- 下記で記録した未解決 follow-up: `scheme_app`, `scheme_name`,
  `functor_loci`, `param_name`, `type_arg_list`, `type_arg`, `qua_arg`。

## AST 設計前に修正

これらは Task 6 で修正済み、または対応する AST node vocabulary を固定する前に
解決が必要な項目である。

| ID | 指摘 | 状態 |
|---|---|---|
| G-AUD-001 | `builtin_pred ::= "in" | "=" | "<>"` は存在していたが formula parsing に接続されず、組み込み membership / equality / inequality formula が到達不能だった。 | Appendix A と第 14 章に独立した `builtin_predicate_application` を追加して修正済み。一方で `pred_pattern` は `def_predicate_symbol` を使うため、primitive predicate は文法だけでは定義・再定義できない。第 9 章の例も `in` や `=` を user predicate として定義しない形に直した。Task 7 では `x in X`、`x = y`、`x <> y` の formula fixture と、`in`、`=`、`<>` を使う `pred` / `redefine pred` 試行の negative fixture を含める。 |
| G-AUD-002 | annotation production が module item、proof statement、algorithm statement から到達不能だった。 | Appendix A に `annotated_declaration`、`annotated_statement`、`annotated_algo_statement`、annotation 付き registration/template content、`claim_block` 内の annotation 付き theorem item を追加して修正済み。文脈上の妥当性は semantic / diagnostic 側に残す。 |
| G-AUD-003 | token list drift があった。`step` は range loop 文法で使うが未予約、第 2 章は import の `..` を欠き、単独 `@` は扱いを明確化する必要があった。 | Appendix A と第 2 章で仕様レベルでは修正済みであり、dot-compound priority list にも反映した。`@` は annotation marker であり、単独の予約記号 token ではない。`step` と `..` に対する `mizar-lexer` / `mizar-session` table と lexical fixture の同期は lexer 実装トラックへ defer し、`doc/design/mizar-lexer/ja/todo.md` で明示的に所有する。該当する lexical table と dot-disambiguation requirement は実装同期まで `tests/coverage/spec_trace.toml` で `partial` とする。 |
| G-AUD-003a | `max_axioms`、`steps`、`timeout` などの option / solver spelling は固定 literal spelling として現れるが、global な予約語にするべきではない。 | Appendix A で文脈限定 spelling として文書化して修正済み。`nest` は現行 lexical contract の下で意図的に reserved のままとし、lexer-facing な言語変更を行う場合だけ再検討する。 |
| G-AUD-004 | `claim_block` が top-level declaration と `algo_statement` の両方にあり、第 20 章では algorithm の snapshot に関する top-level block として説明・例示されていた。 | Appendix A では top-level `claim_block` を維持し、`algo_statement` から削除して修正済み。 |
| G-AUD-005 | `attribute_assertion` が nullable な `attribute_chain` を使っており、空の `x is` 形を許していた。 | Appendix A に非空の `attribute_test_chain` を追加して修正済み。 |
| G-AUD-006 | `definition_block` と `template_def` はどちらも `definition ... end;` で始まり、内容も重なる。parser dispatch は開始 keyword だけに依存できない。 | definition/template AST node の前に未解決。Task 7 では通常の definition block、`let T be type` を持つ template definition、theorem を含む template、曖昧な `let x be T` case を含める。owner: parser の definition/template task と mizar-syntax node vocabulary task 15-16。 |
| G-AUD-007 | `type_head ::= radix_type | mode_type` はどちらも `qualified_symbol [ type_args ]` であり、構文上は曖昧で、区別は active symbol table に依存する。 | type AST node の前に未解決。generic な syntactic type head に正規化して category resolution を後段へ送るか、parser lookup 境界を文書化する。owner: parser type-expression task と mizar-syntax task 10。 |
| G-AUD-008 | dot の役割は parse 時点では意図的に未解決である。selector access/update、namespace separation、active user symbol が同じ表層 token を共有しうる。 | 未解決だが mizar-syntax todo で追跡済み。Task 7 では import、reference、qualified symbol、selector access、lvalue、active `.` user-symbol の dot-chain fixture を含める。 |
| G-AUD-009 | `type_assertion` と `attribute_assertion` は `term_expression "is" ...` 境界を共有し、どちらの tail も active-lexicon symbol から始まりうるため、parse 時点で別 AST 形状に分けると parser に意味論知識を持ち込む。 | Appendix A、第 14 章、parser / syntax TODO で 2 つの atomic alternative を generic な `is_assertion` に置き換えて修正済み。後段の resolution が type assertion または attribute assertion に分類する。Task 7 では曖昧な `x is T`、`x is non empty`、修飾 attribute / type の例を含める。 |
| G-AUD-010 | `compact_statement` と zero-step `iterative_equality` は、`x = y by A;` のような正当化付き等式で重なる。justified formula としても、`.=` continuation を持たない iterative equality としても読めるためである。 | statement AST node の前に未解決。parser design は `.=` continuation が続かない限り `compact_statement` に dispatch するか、statement AST 設計が決めるまで generic な justified-equality surface を保持する必要がある。Task 7 では `x = y by A;`、`x = y by A .= z by B;`、label / `then` variant を含める。 |

## Semantic-Only として受容

parser は source shape を保ち、次の判断を抱え込まない。

- `qualified_symbol` が mode、struct、predicate、functor、attribute、theorem
  label のどれを指すか。ただし周囲の keyword が構文上の役割を固定する場合を除く。
- annotation name が既知か、現在の attachment site で許可されるか、引数が
  registry 固有の規則を満たすか。
- visibility default と public/private export 効果。
- type well-formedness、sethood、registration closure、proof obligation、
  template guard の充足、algorithm termination obligation。
- import resolution、循環 import の拒否、fully qualified symbol identity。
- 第 8 章と第 19 章の companion EBNF が、意味論説明のために `qua`、
  `reconsider`、overload-resolution helper form を再掲すること。parser 所有権は
  Appendix A と所有章の正規化済み production に置く。

## Task 7 へ defer

AST snapshot を設計する前に、Task 7 で次を parse-only acceptance matrix の行にする。

- built-in predicate の positive / negative fixture。`in`、`=`、`<>` を使う
  `pred` / `redefine pred` 試行の negative fixture、および後で type assertion
  または attribute assertion に解決される generic `is_assertion` form の
  positive / negative fixture を含める。attribute argument も含める。
- module level、definition 内、registration 内、proof 内、algorithm 内、claim block
  内の annotation attachment fixture。
- definition/template ambiguity fixture。predicate / functor template parameter を含む。
- `scheme_app` の表層例。現在の文法では schema use は
  `simple_justification` と `reference [ template_args ]` から到達する。独立した
  `scheme_app` helper は互換 alias であり、fixture 計画時に削除または再用途化する。
  到達不能な `functor_loci` helper も同様に、dead な章互換 wording として削除するか、
  concrete fixture 付きで再用途化する。
- `param_name`、`type_arg_list`、`type_arg`、`qua_arg` の parameterized name fixture。
- namespace path、selector access/update、lvalue、grouped/bulk reference、active
  user-symbol use を区別する dot-chain fixture。
- `the type_expression` / `pick_expr`、`qua`、`with (...)` の term 境界 fixture。
- compact statement と iterative equality の境界 fixture。特に `x = y by A;`
  の zero-`.=` overlap と、`x = y by A .= z by B;` の continuation form、
  label / `then` variant を含める。
- 第 8/19 章の重複 EBNF は別 parser production ではなく fixture で覆う:
  parenthesized / unparenthesized `qua`、chained `qua`、現行 Appendix A の
  justification 境界に従う `reconsider` form。
- missing `end`、missing delimiter、malformed annotation、誤配置された `claim` /
  `import` form の recovery fixture。

## Review-Only Agent の指摘

最初の review-only pass は 6 件を指摘した。すべてこの監査 pass で対応済みである。

- token drift を完全解消と書きすぎていた。仕様正規化と、defer した
  `mizar-lexer` table / test sync を分けて記録した。
- built-in predicate が predicate definition に漏れていた。Appendix A は定義用に
  `def_predicate_symbol`、適用用に `builtin_predicate_application` を使う。
- `type_assertion` と `attribute_assertion` の AST 形状曖昧性が残っていた。
  Appendix A は resolution まで generic `is_assertion` を保持する。
- `claim` は Appendix A では top-level だが TODO では statement/local definition
  に残っていた。mizar-syntax と mizar-parser の TODO で、template / algorithm /
  annotation node および parser task 32 の担当に移した。
- `nest` が contextual keyword discussion から漏れていた。現行 lexical contract
  では global reserved のままと記録した。
- `open` / `inherit` が module-item work として残っていたが、Appendix A は
  module-level form を定義していない。TODO からその表現を削除し、`open` は
  theorem status、`inherit` は structure definition の作業に残す。

follow-up の review-only pass は 3 件を指摘した。すべて対応済みである。

- lexer token drift を defer したが、具体的な owner が弱かった。`mizar-lexer`
  TODO が `step` / `..` table と fixture 同期を所有し、該当する lexical table
  coverage entry は実装同期まで `partial` とした。
- 第 9 章に `in` と `=` を predicate definition として示す例が残っていた。
  primitive built-in を定義しない例に直し、`in`、`=`、`<>` は `pred` /
  `redefine pred` symbol ではなく built-in application であると明記した。
- 第 14 章と parser task 13 に type / attribute formula を分割する表現が
  残っていた。generic `is_assertion` に揃え、type / attribute 分類は
  resolution 側に残した。

その後の外部レビューは 5 件を指摘した。すべて対応または分類済みである。

- `compact_statement` / zero-step `iterative_equality` の overlap を G-AUD-010
  として記録し、Task 7 fixture input に追加した。
- 第 2 章の dot-compound priority list に `..` を追加した。lexer fixture が
  追いつくまで dot-disambiguation coverage entry は `partial` とした。
- unreachable production の説明に `functor_loci`、`scheme_name`、
  `reconsider_target` を追加し、到達可能な `dq_char` を削除した。
- 第 14 章の `attribute_ref` に Appendix A と同じ attribute argument list を
  追加した。
- 第 14 章の quantified-variable helper を `quantified_vars` に改名し、
  Appendix A の互換 alias `qualified_vars` との衝突を避けた。

## 章別 drift メモ

章別 EBNF には、Appendix A が `qualified_symbol` に正規化した箇所でまだ
`identifier` を使うもの、visibility を省略可能な `private` として扱う古い形、
第 8 章の parenthesized-only な `qua_expression` と狭い `type_expression` 要約、
`by computation` 追加前の `justification ::= simple_justification | proof` など、
古い helper 名や狭い形式が残っている。第 19 章は overload-resolution の説明のために
`qua_expression` と `redefine_item` を意図的に再掲している。上で列挙したものを
除き、これらは互換用の説明である。parser-facing な正規化済み形式は Appendix A
であり、今後の章本文編集で Appendix A から parser behavior を逆方向にずらさないこと。
