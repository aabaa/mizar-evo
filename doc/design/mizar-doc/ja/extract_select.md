# mizar-doc 抽出選択 — 定義提案
正本: 英語; [English companion](../en/extract_select.md)。
状態: 非規範の提案のみ。ES-Q1–ES-Q7 は未選択であり、選択 policy、API、診断コード、実行可能部分集合の拡張を採用しない。

## 根拠と固定境界
Doc16 は spec20 [code extraction](../../../spec/ja/20.algorithm_and_verification.md#2010-コードの抽出) の下で、architecture13 の [verified subset](../../architecture/en/13.documentation_and_extraction.md#extraction-uses-verified-executable-subset) と [Step 1](../../architecture/en/13.documentation_and_extraction.md#step-1-select-extractable-items) を具体化する。抽出は検証済み `terminating` algorithm を消費し、export signature や数学的妥当性だけでは不十分である。target 言語、数値表現、host-library binding は inline annotation ではなく extractor configuration に属する。最初の target は Doc21 の判断である。
phase16 は phase14 の proof acceptance と phase15 の artifact integrity を消費する。意味解析・証拠受理を再実行せず、proof validity や公開入力を変更しない。[Doc2 reader](./artifact_reader.md#published-input-loading) が保証するのは publication consistency であり、現在の extraction-policy acceptance や実行準備ではない。spec23 の [exports](../../../spec/en/23.package_management_and_build_system.md#2351-mizirjson-artifact-format) は signature/contract を含むが algorithm body は含まない。source の再解析から意味を推定して補ってはならない。
各選択 item には source/dependency artifact の一致、active extraction policy による必要 contract/termination obligation の受理、ghost/proof state の消去可能性、全 runtime operation の target lowering が必要である。external attestation は存在しないか policy が許可していなければならない。保存された proof-status 名や witness の到達可能性だけでは受理を示さない。本提案は policy の既定値を推定しない。
[internal05 reader](../../internal/en/05.documentation_extraction.md#artifact-documentation-reader) と [extraction pipeline](../../internal/en/05.documentation_extraction.md#extraction-pipeline) は検証済み executable projection と dependency summary を要求する。必要 source/map の欠落は該当 item を阻止し、source が存在するだけでは抽出準備を示さない。implementation cache は architecture13 の [consumer validation](../../architecture/en/13.documentation_and_extraction.md#phase-16-is-a-consumer-phase) に従い、未検証 compiler IR を別の権威にしない。Doc2 はこれら抽出固有の前提をまだ供給しない。

## 選択と分類
既存 root 起点は exported algorithm、明示 extraction configuration、CLI 選択 item である。候補の特定は抽出許可ではない。非 algorithm と非 terminating algorithm は spec20.10 の executable root にできない。root channel の合成、private visibility、runtime dependency closure は ES-Q1–ES-Q3 に残し、暗黙の優先順位や到達性 algorithm を採用しない。
既存分類は architecture13 の意味を保持する。

| Status | 既存の意味 / 拒否境界 |
|---|---|
| `Extractable` | verification、computability、target の全制約を満たす。出力できるのはこの分類だけ。 |
| `SpecOnly` | 数学的には妥当だが選択 target で実行できない。stub や推測 runtime 値として出力しない。 |
| `BlockedByProof` | 必要 obligation が open または policy が不許可。選択側で再検証・再解釈しない。 |
| `BlockedByTarget` | runtime construct の target lowering がない。その item の部分コードを出力しない。 |
| `BlockedByDependency` | dependency artifact または runtime mapping がない。必要 runtime dependency を黙って省略しない。 |

必要 runtime call 経由で到達したものも含め、未対応 runtime construct を明示的に拒否する。必要 callee/mapping が利用できない item は全抽出制約を満たさない。依存を自動追加するか要求に列挙させるか、理由をどう伝播するかは ES-Q3/ES-Q4 が必要である。proof-only dependency がそれによって runtime code になることはない。
広い `SpecOnly` と target-blocked の説明は重なりうる。表の行順や最初の失敗は優先順位を定義しない。必要 executable projection/source-map evidence の欠落は拒否の前提であり、第六の status の発明、lower-reader error の付け替え、欠落を空 body とみなす許可ではない。実装前に ES-Q4/ES-Q5 で表現・分類の不足を閉じる必要がある。

## 実行可能部分集合と明示的拒否
spec20 の [ghost isolation](../../../spec/en/20.algorithm_and_verification.md#2013-variable-declarations-var-const) は ghost-to-runtime flow を禁止する。ghost 宣言・代入、specification-only snapshot、proof-only structure は既存境界でのみ消去できる。target を対応済みに見せるため runtime operation や必要 runtime check を消去してはならない。ghost escape は不正 projection または前段 checker bug を示す extraction error であり、値の挿入や proof の再評価で修復しない。
spec20 の [Pick](../../../spec/en/20.algorithm_and_verification.md#203-non-determinism-pick) は非 ghost runtime site、ghost-only site、specification formula の stable choice を区別する。非 ghost Pick は選択 target で実行可能な対応が必要で、なければ明示的に拒否する。deterministic witness、有限列挙、host choice routine をここで選ばない。ghost-only Pick は ghost state とともに消去されるが、消去前の必要 non-emptiness obligation は残る。specification formula の choice を runtime Pick site にしない。既存 partial/terminating verification 規則を維持し、partial algorithm を抽出対象に追加しない。
通常の代入、制御、call、数値・data operation も検証済み executable data と target mapping がある場合だけ対象になりうる。これは制約であり、現在の backend の対応を主張しない。新しい有限型規則、数値表現、dynamic-check policy、capability catalog は定義しない。RuntimeIr projection/erasure は Doc18–20、backend 対応と runtime binding は Doc21–22 に属する。
未対応 item は degraded code、placeholder、文の省略、runtime の意味変更ではなく明示的に拒否する。internal05 に従い未対応 construct は該当 item を阻止し、選択 profile が全 requested root を要求する場合はその境界に従う。適切な profile で独立した適格 item を出力する余地はあるが、拒否 item の部分出力は認めない。profile 既定値、request failure、staged-output retention は ES-Q7 と後続 publisher owner の定義を要する。

## Identity・診断・採用
artifact-bound module/FQN/item identity、利用可能な source range、dependency provenance を保持する。同じ verified input、target、policy、configuration に対して選択と診断は決定的でなければならない。canonical module/FQN 順は architecture13 に従う。root 重複・衝突、reason multiplicity、location 欠落の表現は明示的定義を要し、到着順で決めない。target name mangling/reverse map と output manifest は後続 owner に残す。
integrity/staleness/schema error は lower-reader failure のままとする。抽出拒否は独立した Extraction diagnostic family と [internal05](../../internal/en/05.documentation_extraction.md#diagnostics) の既存 ordering/profile 境界に従う。code、severity table、descriptor、location をここで割り当てない。欠落 range を捏造しない。documentation warning promotion と承認済み comment Q1–Q8 option ① は extraction fallback や proof policy 変更を許可しない。Doc7 は parked、Doc8 は dependent のままである。
Doc17 実装前に owner 回答を別途許可された所有定義へ反映し、executable-projection producer/schema、current policy evidence、root/dependency identity、status/reason/location 規則、target capability、extraction 診断を定義する必要がある。後続 fixture は適格・拒否 root、未対応 runtime construct ごとの拒否、dependency refusal、ghost/Pick 分離、deterministic/profile 挙動を扱う。既存 reader test は入力検証の根拠に留まり、本提案は executable coverage、allocation、audit credit を追加しない。

## Owner Q-Decisions — すべて未選択
各案は上記固定境界に制約される tool 定義候補であり、新しい言語規則や許可ではない。owner は別の限定定義を求めてもよい。採用には別途許可された仕様更新が必要であり、本提案は Doc16 の規範定義や Doc17 readiness を完了扱いにしない。

| Question / 対象定義 | 提案 ① | 提案 ② |
|---|---|---|
| ES-Q1 — root channel と重複 | 有効な export/config/CLI root の和集合を取り、同一 full artifact-bound identity を統合し、binding 衝突は拒否する。 | 明示 config/CLI root がある場合 default export を置換する。同一 bound identity を統合し、binding 衝突を拒否する。config/CLI の合成は採用更新で定義する。 |
| ES-Q2 — private 明示 root | 検証済み executable metadata と identity がある場合にのみ明示 private algorithm を許可する。export discovery は public のまま。 | 全 root を exported algorithm に限定し、明示 private root を拒否する。どちらも言語 visibility を変更せず raw-source lookup を許可しない。 |
| ES-Q3 — runtime dependency closure | 検証済み dependency metadata から必要 executable callee を推移追加する。採用前に cycle、policy propagation、refusal path を定義する。 | request に runtime callee または設定済み external mapping の列挙を要求し、欠落は caller を阻止する。採用前に closure validation と cycle を定義する。どちらも必要 call を消去しない。 |
| ES-Q4 — 分類と reason 報告 | 該当 typed refusal reason を決定的順序ですべて保持し、別途承認する優先表から五 status の一つを導く。 | 相互排他的分類 predicate と一つの primary reason、決定的 related dependency reason を定義する。両案とも SpecOnly/target refusal を区別し projection/map 欠落を分類する construct catalog を要し、lower error を付け替えない。 |
| ES-Q5 — executable/policy evidence | lower-owned accepted-obligation/policy result 参照と必要 source-map availability を伴う artifact-bound per-item executable projection を定義する。 | 別々に検証した executable view と policy view を full publication/item identity で結合する。両案とも producer/schema/currentness contract を要し、ここで proof acceptance、欠落 body、新 public API を発明しない。 |
| ES-Q6 — 必要 source/map availability | 選択 item と runtime operation ごとに必要 data を定義し、必要 projection/map data が揃う場合のみ source-less extraction を許容する。 | executable projection に加えて各選択 module の完全な source/map coverage を要求する。両案とも必要 data と refusal reason の正確な定義を要し、source availability は verified executable data の代わりにならない。 |
| ES-Q7 — profile 既定値と request outcome | 該当 item 拒否と明示 failed-root result を既定とし、名前付き strict profile が全 requested root を要求する。 | 全 requested root の成功を既定とし、名前付き permissive profile が明示拒否付きで独立した適格 item を許可する。両案とも拒否 item の code を出さず、request exit/output retention と publisher atomicity は後続定義を要する。 |
