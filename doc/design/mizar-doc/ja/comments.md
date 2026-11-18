# mizar-doc コメント定義
正本は英語: [English companion](../en/comments.md)。
状態: Q1–Q4 の①は所有者承認済み。下記 Q2/Q4 残余提案は所有者判断待ち。Q1 と CommonMark の前提は別途残る。

## 権威と責務
spec [2.9/2.10](../../../spec/en/02.lexical_structure.md#29-comments-and-annotations) と [24.1](../../../spec/en/24.documentation_generation.md#241-documentation-comments-syntax) がコメント構文・対応付け・CommonMark・タグを所有する。architecture [02](../../architecture/en/02.source_and_frontend.md#comments-and-doc-comments-are-source-metadata) は構文上の候補を保持し、architecture [13](../../architecture/en/13.documentation_and_extraction.md#step-2-attach-doc-comments) と internal [05](../../internal/en/05.documentation_extraction.md#documentation-inputs) は検証済み identity とソース隣接性の両方を要求する。本定義はこの consumer 境界の詳細化であり、言語意味論を変更しない。
comments module は block 化、attachment 候補、Markdown/tag 解釈と表示診断を所有する。Doc7 は承認と入力定義の完成後に attachment を実装し、Doc8 は Doc7 後に構文解析を実装する。link/UsedBy 解決、数式描画、HTML/search 出力、抽出、公開は後続 owner に残す。

## 入力と来歴
検証済み [Doc5 plan](./doc_build.md#full-build-and-index-inputs) を消費し、publication、完全な ModuleSummaryIdentity、全 export、optional summary を変更しない。item は所有 publication/module と元の検証済み export identity（origin/FQN）で識別する。label query は候補であり、それだけで一意に結合できない。FQN 分割・signature 解析・近い名前・意味解析の再実行によって identity や label を捏造しない。
[optional native lexical reader](./artifact_reader.md#optional-native-lexical-source) は正規化 source text、Disk loading map、identity を持たない Lexer 前処理全体（raw comment kind/lexeme/span と診断）を返す。block、Markdown、tag、検証済み attachment、frontend 依存は供給しない。承認 Q1 は [Artifact 投影](../../mizar-artifact/ja/comment_projection.md) とし、この optional reader はコメント fallback にしない。
Artifact の [availability/identity](../../mizar-artifact/en/comment_projection.md#availability-and-identity)、[座標/map](../../mizar-artifact/en/comment_projection.md#text-coordinates-and-adjacency)、[integrity 規則](../../mizar-artifact/en/comment_projection.md#compatibility-hashes-and-refusals) に従う。map の欠落/不整合は Doc attachment の根拠にならず、optional source 不在でも有効な意味 metadata は利用できる。

## Block と Attachment 対象
spec24.1 の block は named definition/theorem/registration/algorithm の直前に連続する `:::` 行であり、物理的な空行が item との対応を切る。空の `:::` 行は block 内の Markdown 段落区切りである。通常の `::` と `::=…=::` は documentation text ではなく、途中の source 内容を越えて別 block を連結しない。
import/definition より前のファイル先頭 doc comment は module を文書化する。spec2.9.2 の doc block → item に付く `@proof_hint` → theorem の例を維持し、付属 annotation でこの対応を切らない。承認 Q2 により先頭 block は module のみとし、通常 trivia/standalone annotation は item 隣接性を切る。module-leading trivia/複数 block の適格性は下記 Q5/Q6 の判断待ちで、どちらの案も選択しない。module は検証済み module identity に結合し、item はソース上隣接する候補と verifier の item identity/range が同一 item を示す必要がある。range 一致だけでは足りない。
definition/theorem/registration は保存された export label metadata を、algorithm は [24.2](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks) に従い識別子を使う。label 欠落・複数候補・range 衝突・未対応の対象情報は未解決として診断し、列挙順では選ばない。package/module/version を跨ぐ同名候補の完全な来歴を保持する。
公開文書は検証済み export 対象を使う。非 export コメントは private-doc 要求がなければ省略できるが、その要求は verifier metadata や検証済み状態を新設しない。invalid/unverified item のコメントを検証済み API 文書として表示しない。nested/unnamed/未対応対象に近接性だけで文書化資格を与えず、拡張には owner 定義を要する。未結合 block は黙って捨てず診断可能に保つ。

### 対応付けの残余定義
block は連続する documentation-comment 行の最大列とする。物理的空行と通常の行/block comment は block を分け、各 block の raw text・map・source 順を別々に保つ。水平字下げと連続行間の通常改行は、途中の comment/空行ではない。同じ block を module と item の両方に付けない。
module-leading 適格性は import または source item より前の file prefix のみで評価する。standalone diagnostic annotation も source item である。item-attached annotation は次の item の境界に属し、通常 trivia ではない。同じ検証済み item に対し、空行/comment/standalone annotation を挟まない場合のみ隣接性を保つ。import・無関係な item・中断を飛び越えて後続 target を探さない。prefix 内の trivia/複数 block は Q5/Q6 の未承認事項とし、fallback を選ばず未解決に保つ。
以下は承認 Q4 の warning 条件の詳細化であり、machine code の付与や reader failure の変更ではない。

| 正常な入力読込み後の条件 | 表示上の扱い |
|---|---|
| 適格 module/隣接 named-item 候補なし（末尾/中断 block を含む） | orphan warning。text/range を保持し、別 target に付けない。 |
| 隣接構文はあるが検証済み identity/label/対象境界が不在または未対応 | unresolved-target warning。text/range/近い名前から identity を推測しない。 |
| 複数 identity または矛盾する候補/range 対応が残る | ambiguous-attachment warning。候補の来歴を保持し、選択しない。 |
| invalid/unverified target | unverified-target warning。検証済み API 文書として表示しない。 |
| 対応に必要な座標が不在または検証済み target と関連付け不能 | unresolved-attachment warning。map を捏造しない。下位 reader の integrity refusal はこの warning ではなく error のまま。 |

診断位置には該当 block の保存済み source range を使い、候補位置は存在する場合のみ related range にできる。検証済み非 export target は既存 private-doc 選択規則を保ち、公開出力からの意図的省略だけでは orphan/missing-comment warning にしない。これは診断条件であり、新 code registry や warning 重複排除方針ではない。

## Markdown と構造化タグ
Markdown 本文の導出でも raw block と各行の map を保持する。承認 Q3 に従い各行の `:::` と続く ASCII space 最大1個を除き、残りと逆 map を保持する。全体 trim/deindent、Unicode/tab/改行の再正規化は CommonMark の字下げ・code・診断位置を変え得るため行わない。
必要な surface は独自縮小言語ではなく CommonMark であり、段落・見出し・強調・list・fenced/indented code・block quote・標準 link を利用できる。空の `:::` 行で段落を区切れる。Doc8 は parser/version と拡張間の扱いを固定し、spec24.1 を黙って縮小しない。観測可能な曖昧さは owner の明確化へ戻す。
Mizar inline code と code block の内容は後続 highlighting 用に保つ。正確な label/FQN の code 参照、明示的 `[text](ref)`、`@see` は Doc9/10 へ渡し、式は code のままにする。inline `$…$` と単独行の display `$$…$$` math は [24.3](../../../spec/en/24.documentation_generation.md#243-mathematical-formula-rendering) に従い range とともに Doc11/12 へ渡す。comments は link/anchor/UsedBy を解決せず、数式・code example・verifier annotation を描画/評価しない。HTML 出力方針は Doc13–15 の責務である。
タグは Markdown 段落の最初の token の場合だけ認識し、名前・raw 引数/本文・range を保つ。spec24.1 の7種は `@param name`（parameter ごとに1個）、`@returns`（戻り値、`result`）、`@requires`/`@ensures`（説明の補足）、`@see ref`、`@since version`、`@deprecated version`（続く非推奨説明）である。この意味を保ち、formal contract の追加・検証変更・code 評価・新 version 文法の強制を行わない。
未知タグは spec2.9.2 に従い verbatim で通し、未知の statement annotation にしない。承認 Q4 は不正/重複/利用可能 metadata と不一致の既知 tag に text を保持する既定表示 warning を出すが、下記残余規則を適用し Q7/Q8 は所有者判断待ちとする。text の黙示破棄・tag 上書き・不在 parameter metadata の推測をしない。Markdown 解析結果は派生 view であり、新 serialized schema や意味 AST の代替ではない。

### タグ検証の残余定義
明示的な必須 slot は spec24.1 に従い、`@param` は `name`、`@see` は `ref`、`@since`/`@deprecated` は `version` とする。slot 不在は malformed warning とし、元 tag 全体を保持する。version 構文・参照解決・parameter 型文法を新設しない。Markdown から tag field を認識する正確な規則は Doc8 の CommonMark 定義に残し、code example 内 text を実行 annotation として検証しない。
同じ parameter 名の `@param` 複数出現は「parameter ごとに1個」に反する。繰返し側に warning を出し、先勝ち/後勝ちの置換をせず両方を source 順に保つ。他 tag の多重性と説明本文だけの空欄は Q7/Q8 の判断待ち。未知 tag は verbatim のままで、未知であることだけを理由に既知 tag 用 warning を出さない。
parameter 名の比較は同じ検証済み target の利用可能な authoritative parameter metadata に限定する。完全な一覧は未知名を確定できるが、部分一覧は不在を確定できない。一覧不在なら membership check を行わず、必須 slot/重複 check は行う。signature 解析や tag text からの一覧推測をしない。`@returns` も authoritative metadata が戻り値なしを明示する場合のみ mismatch warning とし、metadata 不在を不一致にしない。未記載 parameter の coverage warning は導入しない。`@requires`/`@ensures` の prose を形式式と比較せず、`@see` 解決は Doc9/10 に残す。
malformed・duplicate・確定 metadata mismatch は該当 tag の保存 range で既定表示 warning とし、全 raw argument/body と map 済み range を保持する。

## 診断と決定性
[internal05](../../internal/en/05.documentation_extraction.md#diagnostics) の Documentation 診断 family と既存の順序/profile 規則を使う。利用可能な range を保ち、不在 source の range を捏造しない。source 不在は既定 warning、コメント欠落は optional lint である。不正な Markdown は warning と劣化表示にし、既定の文書 build を中断しない。承認 Q4 は未結合/曖昧 block と不正 tag に text を保持する既定表示 warning を出すが、上記残余定義と未決 Q5–Q8 が実装前の規則となる。
integrity/staleness/schema error は下位 reader の error のままで、表示 warning と区別する。profile は文書 warning をその文書 build の error に昇格できるが、証明の有効性・公開された検証入力・抽出方針は変えない。前処理診断の意味/range を保持し、Doc 診断採用によって失敗した read を入力成功にしない。
同じ検証済み publication・binding 済みコメント入力・option に対し、source block/line/tag 順と semantic import 順を保持する。完全な publication/module/item identity と map 済み source 位置で、worker/入力列挙順に依存しない attachment/診断を行う。storage 順を参照優先順位にしない。劣化出力でも問題 text/range を保ち、文書再生成で source/artifact を変更しない。

## Owner Q-Decisions
所有者承認（2026-11-18、entry 164）は次の Q1–Q4 表の①すべてを選択した。②は許可しない。残る定義は前提条件であり、追加意味論の承認ではない。

| 問い | 選択肢と gate |
|---|---|
| Q1 — Doc4 Q4 を継承: コメント供給元 | ① emission 時に source 由来 comment/map を公開 artifact へ投影する。[Artifact 定義](../../mizar-artifact/ja/comment_projection.md) が availability・binding・map・互換性・handoff を所有し、実装前提は未完了とする。source 再取得 transport は選択しない。 |
| Q2 — 対応付け境界 | ① 先頭 block は module のみ、通常 trivia/standalone annotation は item 隣接性を切る。item 付属 annotation の例は保持する。attachment 結果は上記で定義し、module-leading trivia/複数 block は Doc7 前の Q5/Q6 判断待ち。 |
| Q3 — Markdown 本文抽出 | ① 各行の `:::` と続く ASCII space 最大1個を除き、残りと raw/逆 map を保持する。全体 deindent はしない。 |
| Q4 — tag と attachment 診断 | ① orphan/曖昧 block と不正/重複/利用可能 metadata と不一致の既知 tag に既定表示 warning を出し、自動修復せず text を保持する。必須 slot・parameter 重複・metadata availability は上記で定義し、他の多重性/本文要件は Q7/Q8 に残す。新診断 code は定めない。 |

### 追加の提案 Q-Decisions — 未選択
Q1–Q4 承認で確定していない残余意味論を問う。各組は提案のみで、どちらも許可されていない。承認済み①は再検討しない。

| 問い / gate | 提案① | 提案② |
|---|---|---|
| Q5 — module block 前 trivia / Doc7 | whitespace・空行・通常 comment は module prefix 適格性を保つ。その text は除外する。 | whitespace・空行のみ適格性を保ち、通常 comment は module prefix を終了する。 |
| Q6 — 分離した module-prefix block / Doc7 | 全適格 block を module のみに付け、結合/置換せず source 順で別々に保つ。 | 最初の適格 block のみ module に付け、後続は item 隣接性規則に従い、それ以外は orphan warning とする。 |
| Q7 — parameter 以外の重複 / Doc8 | `@returns`/`@since`/`@deprecated` は単一。`@requires`/`@ensures`/`@see` は複数可だが同一 raw tag/payload の反復は warning。 | `@param` の名前 key 以外は同一 raw tag/payload のみ重複とし、異なる payload は全 tag で複数可。 |
| Q8 — 説明本文の欠落 / Doc8 | `@returns`/`@requires`/`@ensures` と `@deprecated version` 後の message に空でない説明を要求し、不在なら warning。 | 明示 name/ref/version slot のみ必須とし、説明 prose が空なだけでは warning にしない。 |

## 引き渡し境界
Doc7 は Q1 定義と実装前提に加え所有者の Q5/Q6 判断を待つ。Doc8 はさらに Doc7・Q7/Q8・CommonMark/tag-field 定義を待つ。この残余提案は実装も選択肢の承認も供給しない。広範な Doc4/5、link/UsedBy、描画、抽出、公開、crate/Steps6–8 の完了は未達のままとする。
