# Doc8 Markdown/Tag 表示診断の定義提案
英語版を正本とする: [English](../en/documentation_markdown_tags.md)。
状態: 非規範の定義提案。以下の MT-Q1–MT-Q7 はすべて未選択。この文書はコードを割当・予約・発行しない。採用と実装には別途の権限が必要。

## 固定入力と所有権
Documentation 診断所有者が spec [22.1.1–2](../../../spec/en/22.error_handling_and_diagnostics.md#2211-error-code-system-e-series)、[22.6.8](../../../spec/en/22.error_handling_and_diagnostics.md#2268-documentation-attachment-warnings)、[24.1–3](../../../spec/en/24.documentation_generation.md) に基づきこの範囲を定義する。[comments Q1–Q8①](../../mizar-doc/ja/comments.md#owner-q-decisions)、[tag validation](../../mizar-doc/ja/comments.md#タグ検証の残余定義) と完全なソース・publication・target provenance を維持する。[Diagnostics plan](./00.crate_plan.md) が前提定義を、[Doc plan](../../mizar-doc/ja/00.crate_plan.md) が後続解析を所有する。
entry 206 が伝える owner 承認 2026-11-18 17:50 JST は entry 202 の [Doc8 Q9–Q18](../../mizar-doc/ja/commonmark.md#未選択の所有者-q-decisions) をすべて①として選択した。同提案の過去の未選択表記は判断を再開しない。正確な parser 基準、段落・tag・raw 比較規則、保護された math 境界を固定入力とし、②や新文法は提案しない。承認と整合的な採用は別である。
特に Q18① では CommonMark の literal/recovery 形だけでは絶対に警告しない。未対応の emphasis/backtick や未閉鎖 fence も同様。以下の継承 tag 条件を別として、この表示範囲では、適格な math delimiter の未対応、W のみの display payload、実際の conversion/map 失敗だけが警告になる。拡張失敗は math node を作らず文字列を保持する。実際の conversion/map 失敗は対象 block を保持済み raw text に縮退し、既定の documentation build を中断しない。受理した math は不透明とし、LaTeX 検証、link 解決、sanitization、後段 rendering 診断を追加しない。

## 候補の意味と descriptor
この節は MT-Q1① の具体化であり、採用済み registry ではない。全七条件は問題の文字列を保持し、tag 条件には承認済み field model を使い renderer 出力を使わない。name/ref/version slot の欠如は不在であって token 文法の誤りではない。不透明な backtick、link、entity、非 ASCII whitespace は追加の構文警告を正当化しない。必須 body は承認済み W/non-W のソース非空性を使い、markup のみでも非空とする。

| 候補 semantic name / meaning key (MT-Q2①) | 英語 summary に対応する正確な条件 |
|---|---|
| `documentation.tag.missing_slot` | 既知 tag に必須の name/ref/version slot がない。対象は順に `@param`、`@see`、`@since`・`@deprecated`。 |
| `documentation.tag.missing_body` | `@returns`、`@requires`、`@ensures`、または `@deprecated version` の後の必須説明 body が空。 |
| `documentation.tag.duplicate` | 後続の既知 tag が承認済み same-target raw 重複規則に違反する。同じ `@param` name、singleton (`@returns`、`@since`、`@deprecated`) の反復、repeatable `@requires`・`@ensures`・`@see` の同一 raw tag/payload。 |
| `documentation.tag.metadata_mismatch` | 利用可能な権威ある metadata により、完全な parameter list に名前が存在しない、または `@returns` の対象に戻り値がないと確定する。 |
| `documentation.markdown.unmatched_math_delimiter` | 承認済み保護境界規則の下で、適格 math opener に有効な closer がない。literal spelling/CommonMark parse を保持する。 |
| `documentation.markdown.empty_display_math` | 対応する display math 候補の内部が W のみ。全 literal spelling を保持し math node を作らない。 |
| `documentation.markdown.conversion_failure` | Markdown/tag 表示 view の導出または mapping が実際にできず、対象 block の raw fallback が必要。 |

重複範囲は同じ full-provenance target に付く別々の source 順 block を含む。出現を併合・上書きしない。未知・類似 tag、未記載 parameter、不在・部分的 parameter membership metadata、利用不能 return metadata、未解決 `@see` ref はこの警告を作らない。部分 metadata では不在を確定できず、必須 slot・重複検査は引き続き適用する。prose を formal formula と比較せず、signature、version 文法、ref 妥当性を推測しない。
Descriptor 方針: family `Documentation`、既定 severity `Warning`、phase `Documentation`、category `DocumentationWarning` は継承制約であり新たな選択ではない。将来の各 descriptor には未割当の個別 numeric code、選択した semantic name、不変 meaning key、表の summary、MT-Q7 provenance metadata が必要。初期 lifecycle 候補は active、aliases・retirement marker・replacement codes は空。実際の有効化は採用時のみ。[Registry metadata](./registry.md#descriptor-metadata)、[allocation](./registry.md#allocation-rules)、[retirement](./registry.md#retirement-rules)、compatibility に従う。改名時は永久 meaning key と alias を維持し、message は identity ではない。W0400–W0404 は attachment 専用で、他の slot も予約しない。

## 位置・発行・profile
各候補は検証済み publication、完全な module identity、利用可能な verified target、block ordinal、元の raw text/maps を保持する。実在の source-bound mapped range のみを使う。非連続の container/Q3 map から架空の連続 range を作らず、MT-Q3 に primary/related-range の選択を公開する。利用可能な source-bound primary がなければ、MT-Q3 は既存の [Documentation provenance location](./failure_record.md#documentation-primary-locations) の再利用を提案する。現在の規範適用は attachment 警告であり、後続採用で適用範囲を明示拡張する必要がある。path、label、artifact offset は SourceId や loaded-source binding ではない。span/EOF、行列、candidate identity、新しい location variant を捏造しない。
実際の conversion/map 失敗では、独立に保持された有効な raw-block mapping/provenance を fallback と位置に使い、失敗した view で自分自身を位置付けない。下位 reader の integrity/staleness/schema 拒否は元の error のままで、表示 warning や成功した read に変えない。MT-Q4–MT-Q6 は occurrence identity、重複条件、block failure 優先順位の候補を定義する。列挙順 fallback や出現の黙示的消失は許可しない。
導出物と重複比較では source block/line/tag 順を保持する。共有 [deduplication](./aggregator.md#deduplication-identity)、[canonical order](./aggregator.md#ordering-and-handles)、[Documentation ordering](./aggregator.md#documentation-locations) は完全 identity と block ordinal の数値順を含め変更しない。診断 record はこの比較規則に従い、別の source-order sort を追加しない。同位置では tag traversal と順序が異なり得る。割当前に numeric code 順を推測しない。新しい出現区別は MT-Q4 の machine details に置き、message、storage 順、worker 順に依存しない。共有 schema/API/dedup 変更は採用しない。
既定では文字列保持と fallback を伴う表示 warning とする。既存 documentation-profile promotion はその documentation build のみを失敗させ、code/default severity、proof validity、published verification input、extraction acceptance を変更しない。profile flag、suppression 構文、automatic fix、追加の escalation を設けない。将来の割当 code は該当する spec22 warning control に従う。missing source/comment、attachment、links/UsedBy、後段 math/HTML rendering 診断は各所有者に残す。Doc9 Q1–Q10 と Doc11 Q1–Q12 は未選択のままで、ここで決める前提ではない。

## 未選択の Owner Q-Decisions
各行は後続の診断採用を制約する。どちらも未選択。両候補とも固定警告条件と文字列保持を維持する。組合せ選択には実装前に整合的な descriptor/location/emission 定義が必要。

| 質問・対象 gate | 未選択① | 未選択② |
|---|---|---|
| MT-Q1 — descriptor 粒度 | 表の七 descriptor。duplicate/mismatch の subreason は structured details に保持。 | 四 descriptor: `documentation.tag.malformed` (missing-slot/body reason)、`documentation.tag.duplicate`、`documentation.tag.metadata_mismatch`、`documentation.markdown.presentation_failure` (unmatched/empty-display/conversion reason)。表の条件はすべて区別でき、conversion failure だけが block を縮退。 |
| MT-Q2 — 永久 name/key | 選択した各 semantic name を不変 meaning key とする。 | 不変 key を `documentation.tag.v1.<suffix>` / `documentation.markdown.v1.<suffix>` とし semantic name は version なし。いずれも code number/identity を割り当てない。 |
| MT-Q3 — primary/related range | 問題の marker/slot/delimiter の最初の完全な連続 mapped segment を優先。欠落 field は tag marker、conversion failure は有効な raw-block range。他の mapped segment は順序付き related range とする。使えるものがなければ既存 block provenance。 | 全 warning で含有 raw-block range を使い、利用可能な問題の mapped segment を順序付き related range にする。それもなければ既存 block provenance。両候補とも規範適用拡張が必要で、非連続 map を架空の橋渡しなく保持する。 |
| MT-Q4 — occurrence identity/detail | 利用可能な完全 target identity、block ordinal、reason、Q3 抽出 logical start/end offset で各出現を区別する。失敗 view は block identity と conversion stage/reason を使う。raw tag/duplicate key と mismatch/delimiter subreason を structured details に保持。 | identity の logical offset の代わりに block 内の source encounter ordinal を使う。同じ reason/target/raw 区別を維持し利用可能 offset は details に残す。両候補とも provenance-only 位置を含め複数の適格 delimiter や同一後続 tag を区別。正確な detail encoding は後続採用の前提。 |
| MT-Q5 — 同時条件 | 一出現で独立に確定した条件ごとに warning を発行し、全 reason details を保持。 | 一出現につき全確定 reason を含む一 warning を発行。descriptor 優先順位は conversion、missing slot、missing body、duplicate、metadata mismatch、unmatched delimiter、empty display。明示順位は worker 順に依存せず、reason や必要な warning occurrence を捨てない。 |
| MT-Q6 — conversion failure 優先 | view/map が完了した block からだけ tag/math warning を確定。失敗 block は conversion warning を発行し、独立に確定した reason とその occurrence identity を details に保持。他 block に影響しない。 | 同じ block の独立に検証できた warning を conversion warning と併せて保持。有効な出現を捨てず、失敗 view に依存する warning は公開しない。両候補とも raw fallback を維持し、実装前に再現可能な stage/reason 境界が必要。 |
| MT-Q7 — descriptor provenance | `since` は採用する規範 spec revision、`doc_url` はその正本 Markdown/tag 診断節。 | `since` は実際の割当 registry task、`doc_url` は引き続き規範 spec22 意味節。正確な revision/task token と実在 target は採用時に指定し、この提案を使わない。 |

## 採用 gate
Owner 決定後、別途許可された整合的採用で paired spec22 の意味・位置、registry descriptor、影響する record/aggregation/Doc 所有仕様を同期し、新規 code を割り当て、選択 detail encoding を定義し、descriptor/location/order/profile の test を揃えてから Doc8 を実装する。この提案も entry 202 の承認も採用を実行しない。coverage/trace/owner credit、Doc8 完了、Doc9/Doc11 決定、より広い phase 完了を主張しない。
