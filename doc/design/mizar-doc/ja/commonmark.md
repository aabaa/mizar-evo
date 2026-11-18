# Doc8 CommonMark・Tag-Field 定義提案
正本は英語: [English companion](../en/commonmark.md)。
状態: 過去の提案。2026-11-18 17:50 JST に Q9–Q18 のすべて①が承認され、[spec24.1/3](../../../spec/ja/24.documentation_generation.md) と [spec22 診断](../../../spec/ja/22.error_handling_and_diagnostics.md#2269-documentation-markdown-and-tag-warnings) に採用済み。以下の未選択・候補表記と②は履歴であり現在の gate ではない。parser 実装・適合性は別作業、Q1–Q8 は引き続き有効。

## 権威と固定入力
spec [2.9.2](../../../spec/en/02.lexical_structure.md#292-annotations)、[24.1–3](../../../spec/en/24.documentation_generation.md)、[comments 所有文書](./comments.md#markdown-と構造化タグ) に従う。Doc7 は対応付け済みの別々の source 順 block、raw line/map、完全な検証済み target provenance を供給する。Q3 は `:::` と直後の ASCII space 最大1個のみ除き、全体 trim/deindent や source の再正規化を許さない。
heading、thematic break、list、quote、fenced/indented code、reference definition、標準 link/image、autolink、escape/entity、raw HTML を含む CommonMark 全体を保つ。未知 tag は原文通過し、comment は statement annotation や formal contract ではない。metadata 照合は [残余検証](./comments.md#タグ検証の残余定義) に従い、利用可能な権威情報のみ使う。

## Parser 候補
Q9 の提案 pin は [CommonMark 0.31.2](https://spec.commonmark.org/0.31.2/) と [pulldown-cmark =0.13.6](https://docs.rs/crate/pulldown-cmark/0.13.6)、`default-features = false`、`Options::empty()` とする。これは候補であり、Cargo metadata の採用や適合性実証ではない。実装受入れ前に固定 CommonMark の例と下記 Mizar 相互作用 case を要求し、parser の差異は言語を縮小せず所有者へ戻す。
optional table、footnote、task list、strikethrough、smart punctuation、heading attribute、metadata block、native math option は無効とする。Mizar math は Q17 の別の提案 layer であり、無関係な拡張を有効にする許可ではない。raw HTML は CommonMark 構文として解析し、escape/sanitize と出力判断は Doc13–15 が所有する。offset event は派生 view であり、decode/正規化済み event と独立に raw spelling を保ち、新 AST/schema を直列化しない。

## Field Model 候補
以下は Q10–Q18 の①を一体として定義し、所有者 review のみに供する。`W` は ASCII space・tab・CR・LF であり、この候補では非 ASCII whitespace は field separator ではない。認識時のみ CommonMark container/paragraph indentation を除き map を保つ。paragraph の先頭 inline source 位置で、literal `@` と最長 non-W token が tag を開始し、既知名は完全一致の小文字のみ認識する。escape/entity decode や inline markup 除去で tag 名を作らない。
marker 後の W を同じ paragraph 内で読み飛ばし、slot がある tag は次の最長 non-W source token を name/ref/version とする。markup decode や構文制約を加えない。body は slot 後の separator W を除いた source suffix、slot なしなら marker 後の separator W を除いた全体とする。意味上の境界は separator W を含めないが、元の全 payload は保持する。欠落 slot を別 paragraph から補わない。soft/hard break は paragraph 内の field/body を継続し、次の paragraph/block で tag を終える。
field 抽出は container prefix を除く logical paragraph stream で display block 認識後・inline math 認識前に行い、必須 slot 内の math は解釈しない。各派生 segment を container 除去・Q3・保持済み Artifact/loading map 経由で実 source へ写し、除去 text をまたぐ連続 range を捏造せず segment provenance を保つ。raw 重複比較は元の Q3 抽出後 slice を使い、継続行 container prefix を含め、最後の寄与行の終端改行の直前までとする。内部改行と末尾 space/tab は有意のまま。

| 既知 tag | 明示 slot | 承認済み body 要件 |
|---|---|---|
| `@param` | name | 説明は任意。同一 parameter name に一つ。 |
| `@returns`、`@requires`、`@ensures` | なし | Q8 により空でない説明が必須。 |
| `@see` | ref | 説明 body は不要。未解決 ref は Doc9/10 へ渡す。 |
| `@since` | version | 説明 body は不要。新 version 文法なし。 |
| `@deprecated` | version | Q8 により後続 message は空でないことが必須。 |

候補①の nonempty は body に non-W source character が一つ以上あることとし、markup だけでも数え、renderer を参照しない。backtick で囲む `x` のような slot は raw token 全体であり、parameter name を推測しない。slot 欠落・承認済み重複・確立した metadata 不一致は全 text を保ち warning とする。完全/部分/不在の parameter/return metadata は comments 所有文書の制限を維持する。未知 tag・未文書化 parameter・未解決 ref・metadata 不在だけでは warning を追加しない。
候補①の重複検査は、同じ完全 provenance の module/item target に対応する全別 block を source 順で扱う。`@param` は raw name token 完全一致、singleton は payload に関係なく tag 名で比較する。Q7 の複数可 tag は tag 名と marker 直後から paragraph 終端までの全 payload を完全一致比較し、separator/末尾 W と改行を含め、Q3 抽出以外の加工をしない。Unicode・whitespace・entity・markup・reference の正規化なし。後続の各一致 occurrence に warning を出し、置換や block 結合をせず全 occurrence を保持する。

## 未選択の所有者 Q-Decisions
各行は Doc8 の該当部分を止める。①は具体候補であり、承認済み推奨ではない。異なる組合せの選択時は採用前に整合した定義へ改訂する。Q7 の singleton/反復規則と Q8 の必須 body は緩めない。

| 問い / 該当 gate | 未選択① | 未選択② |
|---|---|---|
| Q9 — 版付き CommonMark/parser gate | 適合性検査を条件に、上記の正確な baseline/pin/option を承認する。 | 採用前に別の正確な CommonMark/parser/version 提案を要求する。spec24.1 全体の surface は必須のまま。 |
| Q10 — paragraph/tag 境界 | CommonMark paragraph 先頭のみ tag を開始し、隣接する tag 状の行は継続 text とする。spec24.1 の連続 `@param a` / `@param b` / `@returns` 例は三つでなく一 field となるため、所有者の明確化が必要。 | 明示的 tag-field 拡張を加える。同じ paragraph の soft/hard break でも、条件を満たす行頭 marker が新 field を開始して前 field を終える。採用前に所有者が「paragraph の first token」と整合させる。 |
| Q11 — container paragraph | quote と tight/loose list item 内の paragraph も対象とし、event 出力が wrapper を省略しても CommonMark の構造上の paragraph を使う。heading/code/HTML/reference definition は除外。 | document-root paragraph のみ対象とし、入れ子 paragraph は tag 検証なしの通常 Markdown とする。 |
| Q12 — marker spelling | 上述の literal source token を使う。escape/entity の `@`、code/emphasis/link で囲む名前、大文字名、`@param,` は既知 tag にせず原文保持。 | paragraph の decode 済み先頭 text token を escape/entity も含め認識する。ただし code/inline-container 内は除外し、小文字完全一致と W/終端境界は維持。 |
| Q13 — slot/separator 抽出 | 上述の W 区切り raw token を paragraph の soft/hard break 越しにも使う。backtick/link/entity は raw spelling とし、特別な quoting なし。 | CommonMark Unicode whitespace を separator とし、最初の decode 済み inline text/code atom を slot にする。必須 slot は改行越しに取らない。採用前に slot provenance と拒否する inline form の明示 map が必要。 |
| Q14 — 説明 body の範囲 | 当該 paragraph の suffix のみ tag に属し、後続 paragraph/list/quote/code block は通常文書とする。 | 次の tag または container 終端まで同じ container 内の後続 block を tag body に含め、入れ子 Markdown 構造を保つ。 |
| Q15 — Q8 の nonempty | non-W source character があればよく、markup/entity/HTML spelling も数える。whitespace のみなら warning。 | 空白以外の表示 text、または code/math/image content を要求し、空 markup/HTML のみなら warning。将来の HTML renderer と独立した定義を所有者が確定する必要がある。 |
| Q16 — 重複 identity/scope | 上述の厳密 raw key/payload と、別 block 越しの同一 target scope を使う。 | raw 比較は同じだが各 block 内のみとし、別 block の同じ tag は衝突しない。 |
| Q17 — math/Markdown 優先順位 | 下記の限定 delimiter 候補を使い、CommonMark code/HTML/link destination を保護し通常 prose 内の link を保持する。 | 固定 parser の native math option は delimiter・escape・block 挙動を spec24.3 と明示的に整合させてから使用する。native 挙動を暗黙承認しない。 |
| Q18 — 表示失敗の境界 | CommonMark の literal/recovery form だけでは warning にせず、適格な未閉じ math delimiter または W のみの display payload は text を保って warning。実 conversion/map 失敗も該当 block を warning/劣化とする。 | 未閉じ code fence と不対 emphasis/backtick にも、有効な CommonMark parse を保持して warning を出す。この追加表示 lint は所有者の明示承認が必要。 |

## 拡張相互作用の候補
Q17①では CommonMark fenced/indented code、HTML block、reference definition を先に特定し、次に display math を特定する。これらの内部では構造化 tag を認識しない。残る paragraph から field を導出し、inline math より前に code span、inline HTML、link/image/autolink 構造を認識して label 以外の内容を保護する。display math 外の標準 link 構造は保つ。backtick 内容は Mizar highlighting と完全 label/FQN 照合の後続 owner へそのまま渡し、式を label として解決しない。Doc9/10 へ link destination と `@see` slot の raw spelling/range を渡し、ここでは参照解決しない。
残る paragraph text では、escape されない単一 `$` を同一 paragraph 内の次の escape されない単一 `$` と対にする。内部は空でなく、隣接 `$` はなく、通貨 heuristic は導入しない。backslash の偶奇で escape を判定する。math は emphasis に優先するが、code/HTML または link/image/autolink 全体の境界をまたがない。link label 内では label 内だけで対にできる。したがって `$[text](ref)$` は CommonMark link と literal の不対 dollar を保ち（Q18①なら warning）、`[$x$](ref)` は label 内 math と link を保持する。受理した math 内容は不透明として raw text/range を Doc11/12 へ渡す。この link 優先は未選択 Q17①であり、spec24.3 の制限を採用したものではない。
display math は container marker と最大3個の indentation space を除いた logical line に、前後の space/tab を除き `$$空でない内容$$` または `$$` のみがある場合とする。後者は同一 container stack/list item の後続 `$$` のみの logical line を探し、保護された code/HTML/reference-definition block または container 終端の直前で探索を止める。両 delimiter 行自体も保護 block の外でなければならない。中断時は opener を未閉じとし、間の text を display block に取り込まず CommonMark parse を保つ。したがって `$$` / fenced code / `$$` は code をまたいだり取り込んだりせず、code 内 `$$` は math を閉じない。受理 display block を paragraph/inline 処理より先に認識し、内部に non-W character を要求する。内部は不透明とし、内容/改行/map を保持して Doc11/12 へ渡す。他の dollar run は literal とし、適格な未閉じ opener は Q18①で warning とする。

対になった display delimiter の内部が W のみなら math node を作らず、綴り全体を literal text として保持して Q18①で warning とする。単一行 `$$…$$` 候補は最外側の対だけを使い、内部に escape されない `$$` を許さず、それ以外は行を literal のまま保つ。これら失敗規則も delimiter 文法と同様に未選択である。

## 劣化と採用の境界
CommonMark は全入力文字列を受け入れ、不対 emphasis/backtick と未閉じ fence にも定義された parse がある。「不正 Markdown」を理由に縮小 validator へ置換できない。Q18①は提案拡張の失敗または view 導出/map の実際の失敗だけを warning にし、該当 raw block を fallback として保持して既定文書 build を中断しない。下位 reader の integrity 拒否を表示成功に変えない。欠落 map に range を捏造せず、適用可能なら既存 provenance location 規則を使う。
採用前に Documentation 診断 owner が Markdown/tag warning の意味・descriptor・location/order/profile 挙動を定義/採用する必要がある。既存 W0400–W0404 は attachment warning であり流用しない。本提案は code を割り当てず新診断方針を確定しない。既存昇格 profile で表示 warning が文書を失敗させても、proof validity/extraction acceptance は変わらない。
所有者 review の witness は、空行区切り/連続 `@param` 行、`> @returns text` と tight-list `- @requires text`、escape/entity/code/wrapped `@param`、backtick/link/NBSP/改行を含む slot、空/markup のみ/複数行 body、一つ/二つの block の完全一致/whitespace 差のある複数可 tag、`$x$`・`$$x$$`・複数行 `$$`・code/HTML/link 内 dollar・不対 delimiter とする。期待結果は承認後に選択列から導く。新しい実行 fixture/spec test ではなく提案 case である。
Q9–Q18 の回答、整合した定義の採用、診断前提の充足まで Doc8 parsing は未完了に保つ。link/UsedBy、math/HTML 描画、抽出/公開、crate 全体の完了は後続 owner に残し、coverage/trace credit は変更しない。
