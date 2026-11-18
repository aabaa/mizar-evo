# Doc9 Link-Model 定義提案
正本は英語: [English companion](../en/links.md)。
状態: 歴史的提案。Doc9 Q1–Q10 全①は [spec24.2](../../../spec/ja/24.documentation_generation.md#242-相互参照とハイパーリンク) に採用済み。本状態と規範仕様は、保存した以下の提案本文・表の候補/未選択表記と②に優先し、それらは現行の採用 gate ではない。[Doc4](./doc_build.md#owner-q-decisions)、[comments](./comments.md#owner-q-decisions)、採用済み [Doc8](./commonmark.md) の所有境界を保つ。

## 権威と固定境界
現行 handoff: 採用済み link semantics は spec24.2 が所有し、Q9 は Artifact 所有の publication-bound direct-by record を要求する。export も `used_axioms` も citation fallback にはしない。[Doc13 Q3](../en/render.md#candidate-destinations-and-page-content) は採用済み href encoding interface だけを供給し、filesystem/source routing は選ばない。
spec [24.1–2](../../../spec/en/24.documentation_generation.md#242-cross-referencing-and-hyperlinks) が label 参照、探索優先順位、曖昧性警告、UsedBy、export label anchor を規定する。architecture13 [共有識別](../../architecture/en/13.documentation_and_extraction.md#documentation-and-extraction-share-symbol-identity)・[Step3](../../architecture/en/13.documentation_and_extraction.md#step-3-resolve-links-and-render-math)、internal05 [index](../../internal/en/05.documentation_extraction.md#documentation-index)・[diagnostics](../../internal/en/05.documentation_extraction.md#diagnostics) が消費側所有を制限する。提案の依存は承認範囲内の [Doc4 入力](./doc_build.md#index-inputs-and-later-completion) であり、Doc8 の採用や解析ではない。
Doc4 は package/version/lock/module/edition の完全識別、選択 publication hash、格納済み export origin/FQN、optional label 候補を渡す。canonical 順序は保存順序であり import の意味上の優先順位ではない。None・利用可能な空・曖昧な候補を区別し、reader の integrity/schema/binding 拒否を維持する。source fallback、label 捏造、package/version 選択、証明受理、直列化 index schema は認めない。
固定対象は exact label/FQN の inline code、明示 `[text](ref)`、`@see ref`。明示リンクと @see は label/FQN・仕様 section・外部 URL を受け取る。code 内の式・定理断片・overload 表記は code のまま。非修飾参照は現 module の export label、次に意味上の import 順で探索し、曖昧なら警告して非リンク表示とする。FQN は overload 解決を行わない。
各 documented item は export label に基づく完全 FQN anchor（例 `#algebra.ring.AddAssoc`）を持ち、page 内で label が一意なら `#AddAssoc` も利用可能。section は `#sec-20-5-1`。definition/theorem/registration の label と algorithm identifier が仕様どおりの label を供給する。権威ある metadata 不在を FQN 分割や signature 解析で埋めない。

## 解決モデル候補
以下は Q1–Q10 の候補①をまとめた所有者レビュー用の記述である。Rust API・parser 文法・artifact field は採用しない。他の組合せには採用前の整合的定義が必要。
parser 中立な occurrence は form（code/明示/@see）、正確な raw spelling、optional 解釈済み destination、source segment map、付着先 module/item 識別、publication 来歴を保持する。認識・decode・field 範囲は Doc8 所有であり Q9–Q18 を選択せず将来の occurrence を記述する。候補①は渡された解釈済み参照を厳密比較し、追加 trim・大小文字変換・Unicode 正規化・式解析をしない。raw は診断用に保持し、code 式全体を複数参照に分解しない。
候補①では認識済み仕様 section を明示的な生成仕様 index に送り、外部 URL はそのまま渡す。他の適格参照は権威ある export label/FQN を照会する。未対応・欠落した destination 分類は未解決 handoff であり symbol spelling を捏造しない。network 照会は行わない。page/URL 安全性と HTML escape は render 所有であり、scheme allowlist の追加や仕様の passthrough 規則の変更はしない。
target は完全な元 module 識別、publication hash、export origin、FQN、実 label、visibility を保持する。reexport の格納識別を保ち origin の同一性を捏造しない。候補①は複数経路から到達した同一の完全 target のみ重複除去し、同じ label/FQN の異なる record は別に保つ。選択した documentation visibility 方針を適用し、格納 metadata を検証権威に昇格させない。現 module 候補が全 import 候補に優先し、一つなら非修飾探索は成功、複数なら曖昧として終える。現 module 候補がない場合だけ意味上の順序で全 import を調べ候補・経路を保持する。異なる二 target の一致は import 位置が違っても曖昧であり列挙順で選ばない。
候補①の FQN 探索範囲は明示的に結合した package/dependency documentation closure。version/package 間の FQN 衝突でも完全識別を残す。探索 namespace と target page/anchor が完全と判明した場合だけ一意 target をリンク可能とする。完全探索でゼロなら未解決、異なる複数なら曖昧。不足 metadata は不完全であり negative lookup や完全な空 graph ではない。一意な現 module hit は後続 import 探索を要さず、それがない場合は import 順序・完全性の不在が非修飾解決成功を妨げる。
候補①の import 入力は source module ごとの権威ある順序付き module 識別と、利用不可・明示的に完全な空・不完全 vector の区別を持つ。dependency 参照は package tuple・publication hash・index location を保持し、index 消費側は利用前に schema/publication/完全 target 識別を検証済み入力に対して認証する。path 単独や canonical sort は import edge を認証せず version を選ばない。cycle があっても source を再帰探索せず、供給済み有限 closure だけを使う。完全性・検証 producer はここでは承認しない。

## Page・Anchor・UsedBy 入力候補
候補①は完全 module/publication 識別を key とする別 allocation table から module-page destination を得る。path は衝突する package/version を区別し、選択 output root に相対とする。table は href 作成前に path 衝突を拒否するが directory spelling や output traversal 方針は選ばない。完全 anchor id は権威ある FQN そのものであり、表記に基づく slug・suffix・改名はしない。同じ page に同じ完全 id を要する異なる item があれば所有者解決までその page の link readiness を止める。権威ある label 不在も item-anchor readiness を止める。
候補①の短縮 alias は page 内でその label を持つ異なる documented target が一つで、完全 id・section id・他 alias と衝突しない場合だけ出す。完全 id を優先し、optional alias の抑制は完全 anchor を変えない。render 境界で DOM 属性 escape と href component encode を行い論理 anchor 識別を維持するが、正確な fragment encoding 契約は Q6 に残す。section 探索には結合済み仕様 version・section id・page destination が必要で、不在・未知 section を local module anchor に変えない。
spec24.2 は定理から定理 T への直接 `by` 引用をすべて要求する。architecture13 は `used_axioms` と citation metadata に言及するが、spec [23.5.1](../../../spec/en/23.package_management_and_build_system.md#2351-mizirjson-artifact-format) の `used_axioms` は特定 status の accepted-evidence fact であり、export 識別/range だけでは構文上の全引用を供給できない。raw export や推移的 accepted-evidence dependency は必要な直接引用 graph を確立しない。この不一致は入力定義 gate であり言語・証明意図の変更や UsedBy 完了宣言を許可しない。
候補①は publication に結合された権威ある直接引用 record を必要とし、引用元定理・引用先定理・`by` occurrence 来歴・完全性 scope を同じ完全 export 識別へ写像する。将来 producer は対象 theorem body/proof の coverage を示す必要があり、producer 不在や endpoint 未写像は利用不可/不完全であり引用ゼロではない。source/name/proof evidence から消費側が再計算しない。Doc4 入力と既存 artifact schema は維持し、producer/schema 作業は別途承認・task を要する。
直接写像済み theorem-to-theorem edge だけを反転し UsedBy とする。候補①は occurrence 来歴を保持し、target ごとに引用元定理を一度表示する。権威ある self-citation は含め、cross-package edge は結合済み両 endpoint を要する。@see・Markdown link・推移的 dependency・used_axioms 単独 entry は direct-by evidence ではない。表示は既存の完全 module 識別順、正確な FQN、origin、最後に publication hash で整列し、反復 occurrence の順は別に保持する。不完全 citation coverage は網羅的な空 UsedBy とせず明示する。UI と不完全入力警告方針は Q10 に残す。

## 未選択の所有者 Q 決定
歴史的決定表: ①列は上記状態に従い採用済み。候補の前提と②列は来歴として保存し、現行の選択肢や採用 gate にはしない。

| 歴史的質問 / gate | 採用済み①（上記状態優先） | 歴史的未選択② |
|---|---|---|
| Q1 — occurrence 比較 / syntax handoff | 上述の将来 parser が渡す解釈済み destination を厳密比較し raw map は別保持。 | raw 参照 spelling のみ比較。escape/entity/code-span 正規化と完全 CommonMark の整合を採用前に所有者が定義する。いずれも Doc8 認識は決めない。 |
| Q2 — 重複経路 / import 曖昧性 | 経路間の同一完全 target をまとめ、複数の異なる import target 一致を曖昧とする。 | 一つの完全 target でも経路の複数性を曖昧とし、警告境界を採用前に定義する。異なる target を列挙順で選ぶことはどちらも許可しない。 |
| Q3 — import 完全性 / 不足入力 | 確定的な現 module hit は許し、他探索は権威ある完全順序 vector まで不完全とする。 | 明示的な完全空 vector を含む完全 import metadata まで publication 全体の解決を gate。どちらも不在を空にしない。 |
| Q4 — dependency-index 検証 / closure | 後続 index 消費側が上述の明示 closure 参照を認証し dependency discovery はしない。 | 所有 producer の事前検証済み dependency-index 入力を要求し、消費前に検証/完全性契約を定義。 |
| Q5 — FQN namespace / version 衝突 | 結合 closure を探索し、完全識別衝突を新構文なしで曖昧とする。 | FQN 解決前に衝突のない documentation namespace を要求。version 選択/統合や FQN 構文変更なしで拒否範囲を定義する。 |
| Q6 — page/full-anchor 割当と encoding | 上述の識別 key table と正確な FQN id を使い、衝突は page readiness を止める。採用前に render と canonical href encoding を定義。 | render 所有の事前割当・検証済み page/anchor registry を要求し、同じ spec24 完全 id を保つ。衝突は gate のままで黙って suffix を付けない。 |
| Q7 — optional 短縮 alias | 上述の衝突しない unique-target alias のみ出す。 | 短縮 alias は出さない。完全 export-label/FQN anchor は必須のまま。 |
| Q8 — 仕様 section / routing | 明示的に version 結合した生成仕様 index を要求し、未知 section は未解決。 | render 所有の仕様 destination registry を section link の gate とし、正確な section 文法/version 結合を採用前に要求。どちらも URL はそのまま。 |
| Q9 — 完全 direct-by citation producer | 上述の occurrence/coverage 来歴を持つ明示 publication-bound citation record を Artifact 所有者へ提案。 | 指定所有者の別途検証済み publication-bound citation sidecar を提案し認証・完全性を採用前に定義。どちらも used_axioms 単独を使わず、ここで artifact field を加えない。 |
| Q10 — 表示診断 / 不完全 graph | 未解決・曖昧参照を安全な非リンク text と決定的 Documentation warning にし、不完全 link/UsedBy 入力を利用不可と完全性警告案で示す。 | 必須の未解決・曖昧警告を保ち、不完全入力が影響する link/UsedBy output を gate。診断 severity と拒否範囲は所有者定義を要する。 |

## 採用とレビュー用 witness
採用済みの [Doc10 前提 interface](./link_prerequisites.md) は producer/allocation handoff と readiness gate の順序を定義する。採用は producer、認証、coverage、allocation、診断、実装の gate のどれも充足しない。正確な共有診断は別途の未採用の提案のままとする。
候補①は既存の diagnostic stage/file/range 規則と安定 target/publication 識別の tie-breaker で表示 draft を整列する。source occurrence は attachment/loading 変換を通る segment map を保持し、不在 map には既存の provenance location を使い offset を捏造しない。Documentation 診断所有者は実装前に未解決・曖昧・不完全入力の意味、位置、順序、profile 挙動を定義する必要がある。ここで新たな共有診断 code は割り当てず、W0400–W0404 は attachment 警告のまま。警告昇格は documentation のみ失敗させ verification/extraction acceptance を変えない。
review witness は exact code label と式、明示/@see の label/FQN/section/URL、import 不在時の local hit、完全空と import 不在、重複経路と異なる一致 import、package/version FQN 衝突、dependency index の誤 hash、full/short/section anchor 衝突、section registry 不在、直接反復/self/cross-package by record と used_axioms 単独、不在と完全空 citation coverage を含む。候補の期待結果は上の段落と Q 選択肢に従い、実行 fixture や採用済み test intent ではない。
Doc10 producer handoff は [前提所有者](./link_prerequisites.md) で供給済みとし、検証済み allocation、診断採用/割当、resolver 統合は別途 readiness gate として残す。Doc9/Doc8 の採用はこれらの入力や診断定義を供給しない。link 解決、schema/診断割当、math/HTML render、extraction/publication、coverage/owner credit、広い Doc4/5/crate 完了は主張しない。
