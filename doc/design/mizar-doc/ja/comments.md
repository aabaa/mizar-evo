# mizar-doc コメント定義
正本は英語: [English companion](../en/comments.md)。
状態: 定義提案。以下の Q-decision はすべて所有者の承認待ちであり、供給経路・依存・実装は選択しない。

## 権威と責務
spec [2.9/2.10](../../../spec/en/02.lexical_structure.md#29-comments-and-annotations) と [24.1](../../../spec/en/24.documentation_generation.md#241-documentation-comments-syntax) がコメント構文・対応付け・CommonMark・タグを所有する。architecture [02](../../architecture/en/02.source_and_frontend.md#comments-and-doc-comments-are-source-metadata) は構文上の候補を保持し、architecture [13](../../architecture/en/13.documentation_and_extraction.md#step-2-attach-doc-comments) と internal [05](../../internal/en/05.documentation_extraction.md#documentation-inputs) は検証済み identity とソース隣接性の両方を要求する。本定義はこの consumer 境界の詳細化であり、言語意味論を変更しない。
comments module は block 化、attachment 候補、Markdown/tag 解釈と表示診断を所有する。Doc7 は承認と入力定義の完成後に attachment を実装し、Doc8 は Doc7 後に構文解析を実装する。link/UsedBy 解決、数式描画、HTML/search 出力、抽出、公開は後続 owner に残す。

## 入力と来歴
検証済み [Doc5 plan](./doc_build.md#full-build-and-index-inputs) を消費し、publication、完全な ModuleSummaryIdentity、全 export、optional summary を変更しない。item は所有 publication/module と元の検証済み export identity（origin/FQN）で識別する。label query は候補であり、それだけで一意に結合できない。FQN 分割・signature 解析・近い名前・意味解析の再実行によって identity や label を捏造しない。
[optional native lexical reader](./artifact_reader.md#optional-native-lexical-source) は正規化 source text、Disk loading map、identity を持たない Lexer 前処理全体（raw comment kind/lexeme/span と診断）を返す。block、Markdown、tag、検証済み attachment、frontend 依存は供給しない。Doc4 Q4 は以下の Q1 として未決のまま保持し、この既存経路はいずれの transport も選択しない。
選択される transport は、ソース由来コメントを同じ選択 publication/module/source hash に結び、raw text、順序付き source span、Markdown offset から元の `:::` 行へ戻るための map を保持する必要がある。正規化 source、lexical text、元の Disk の座標は異なり、有効な map なしに offset を比較しない。map の欠落・不整合は attachment の根拠にならない。
コメント入力が利用不能な状態、利用可能だが doc comment がない状態、空の `:::` 本文は区別する。source hash/path/UTF8/schema の拒否は下位 reader の error を保持し、不在・空コメント・無関係な source への fallback にしない。optional source の欠落は有効な意味 metadata を消さず、コメントを補わない。artifact 投影には producer/schema/provenance 定義が、source 再取得には binding 済み input/API 定義が別途必要であり、本提案だけで存在するものとはしない。

## Block と Attachment 対象
spec24.1 の block は named definition/theorem/registration/algorithm の直前に連続する `:::` 行であり、物理的な空行が item との対応を切る。空の `:::` 行は block 内の Markdown 段落区切りである。通常の `::` と `::=…=::` は documentation text ではなく、途中の source 内容を越えて別 block を連結しない。
import/definition より前のファイル先頭 doc comment は module を文書化する。spec2.9.2 の doc block → item に付く `@proof_hint` → theorem の例を維持し、付属 annotation でこの対応を切らない。先頭 block の重なりと、それ以外の未規定な通常 trivia/standalone annotation 境界は Q2 とし、空行規則の暗黙の例外にしない。module は検証済み module identity に結合し、item はソース上隣接する候補と verifier の item identity/range が同一 item を示す必要がある。range 一致だけでは足りない。
definition/theorem/registration は保存された export label metadata を、algorithm は [24.2](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks) に従い識別子を使う。label 欠落・複数候補・range 衝突・未対応の対象情報は未解決として診断し、列挙順では選ばない。package/module/version を跨ぐ同名候補の完全な来歴を保持する。
公開文書は検証済み export 対象を使う。非 export コメントは private-doc 要求がなければ省略できるが、その要求は verifier metadata や検証済み状態を新設しない。invalid/unverified item のコメントを検証済み API 文書として表示しない。nested/unnamed/未対応対象に近接性だけで文書化資格を与えず、拡張には owner 定義を要する。未結合 block は黙って捨てず診断可能に保つ。

## Markdown と構造化タグ
Markdown 本文の導出でも raw block と各行の map を保持する。marker/空白変換は Q3 の承認待ちである。全体 trim/deindent、Unicode/tab/改行の再正規化は CommonMark の字下げ・code・診断位置を変え得るため行わない。
必要な surface は独自縮小言語ではなく CommonMark であり、段落・見出し・強調・list・fenced/indented code・block quote・標準 link を利用できる。空の `:::` 行で段落を区切れる。Doc8 は parser/version と拡張間の扱いを固定し、spec24.1 を黙って縮小しない。観測可能な曖昧さは owner の明確化へ戻す。
Mizar inline code と code block の内容は後続 highlighting 用に保つ。正確な label/FQN の code 参照、明示的 `[text](ref)`、`@see` は Doc9/10 へ渡し、式は code のままにする。inline `$…$` と単独行の display `$$…$$` math は [24.3](../../../spec/en/24.documentation_generation.md#243-mathematical-formula-rendering) に従い range とともに Doc11/12 へ渡す。comments は link/anchor/UsedBy を解決せず、数式・code example・verifier annotation を描画/評価しない。HTML 出力方針は Doc13–15 の責務である。
タグは Markdown 段落の最初の token の場合だけ認識し、名前・raw 引数/本文・range を保つ。spec24.1 の7種は `@param name`（parameter ごとに1個）、`@returns`（戻り値、`result`）、`@requires`/`@ensures`（説明の補足）、`@see ref`、`@since version`、`@deprecated version`（続く非推奨説明）である。この意味を保ち、formal contract の追加・検証変更・code 評価・新 version 文法の強制を行わない。
未知タグは spec2.9.2 に従い verbatim で通し、未知の statement annotation にしない。引数欠落・重複・parameter 不一致は Q4 とし、text の黙示破棄・tag 上書き・不在 parameter metadata の推測をしない。Markdown 解析結果は派生 view であり、新 serialized schema や意味 AST の代替ではない。

## 診断と決定性
[internal05](../../internal/en/05.documentation_extraction.md#diagnostics) の Documentation 診断 family と既存の順序/profile 規則を使う。利用可能な range を保ち、不在 source の range を捏造しない。source 不在は既定 warning、コメント欠落は optional lint である。不正な Markdown は warning と劣化表示にし、既定の文書 build を中断しない。未結合/曖昧 block と不正 tag は実装前に Q4 方針を必要とする。
integrity/staleness/schema error は下位 reader の error のままで、表示 warning と区別する。profile は文書 warning をその文書 build の error に昇格できるが、証明の有効性・公開された検証入力・抽出方針は変えない。前処理診断の意味/range を保持し、Doc 診断採用によって失敗した read を入力成功にしない。
同じ検証済み publication・binding 済みコメント入力・option に対し、source block/line/tag 順と semantic import 順を保持する。完全な publication/module/item identity と map 済み source 位置で、worker/入力列挙順に依存しない attachment/診断を行う。storage 順を参照優先順位にしない。劣化出力でも問題 text/range を保ち、文書再生成で source/artifact を変更しない。

## Owner Q-Decisions
以下は選択済み default ではなく提案である。実装契約が依拠する前に、選択肢と明確化をこの owner に承認として記録する必要がある。

| 問い | 選択肢と gate |
|---|---|
| Q1 — Doc4 Q4 を継承: コメント供給元 | ① emission 時に source 由来コメント/map を公開 artifact へ投影する。producer/schema の別作業で availability と source/identity binding を定義する。② Doc4 の PreprocessedSource/frontend transport として binding 済み source から取得する。既存の identity-free Lexer 結果で足りるかを含め API/依存を定義する。いずれも意味解析の再実行や未認証 fallback を認めない。Doc7 は選択 transport の定義・前提の完成を待ち、本提案は frontend 依存や artifact payload を追加しない。 |
| Q2 — 対応付け境界 | 最初の named item にも隣接する先頭 block は ① module のみ、または ② module と item の双方とし、spec24.1 の owner による明確化を要する。それ以外の未規定な通常 trivia/standalone annotation は ① item 隣接性を切る、または ② specification owner の明確化後に通過形式を列挙し、物理空行を越えず通過可能にする。両方とも spec2.9.2 の item に付く annotation の例を維持する。Doc7 前に module 先頭 trivia/複数 block の扱いを owner が定義し、named target の種類を自動拡張しない。 |
| Q3 — Markdown 本文抽出 | ① 各行の `:::` と続く ASCII space を最大1個除き残りを保持する、または ② `:::` だけ除き後続空白をすべて保持する。両方とも raw lexeme と正確な行/offset map を保持し、全体 deindent は認めない。CommonMark の code/list 字下げが変わるため Doc8 が暗黙に選べない。 |
| Q4 — tag と attachment 診断 | ① orphan/曖昧 block と不正/重複/利用可能 metadata と不一致の既知 tag に既定の表示 warning を出し自動修復せず text を保持する、または ② 未解決 attachment は診断し、既知 tag の検証は別途定義まで prose として保持する。owner は選択肢の warning severity・必須引数/重複規則・parameter 不明時の挙動を定義する。両方とも未知 tag を verbatim に保ち、既定の不正 Markdown 劣化を維持する。新診断 code は定めない。Doc7/8 はそれぞれ必要な方針定義を待つ。 |

## 引き渡し境界
Doc6 は本英日提案のみを供給する。Q 承認と選択 source/association 定義の完成が Doc7 の gate となり、Doc8 はさらに body/tag/Markdown 定義を必要とする。広範な Doc4/5、link/UsedBy、描画、抽出、公開、crate/Steps6–8 の完了は未達のままとする。
