# Documentation 診断の定義提案
正本は英語: [English companion](../en/documentation_definition.md)。
状態: 過去の提案。2026-11-18 に DD-Q1–DD-Q7 の全①を所有者が承認。現行規範と採用は [spec22](../../../spec/en/22.error_handling_and_diagnostics.md#2268-documentation-attachment-warnings) と[registry](../en/registry.md#documentation-attachment-allocation)が所有する。以下は採用前の提案記録であり、②は未承認。

## 権威と固定境界
[spec22](../../../spec/en/22.error_handling_and_diagnostics.md#2211-error-code-system-e-series) が severity prefix 付き恒久 code を所有し、[registry の gap](../en/registry.md#deferred-code-space-gaps) は後続の仕様更新まで Documentation の割当・公開を禁止する。[internal05](../../internal/en/05.documentation_extraction.md#diagnostics) の phase family は `Documentation` で、`Extraction` とは別。本提案は [Doc7 attachment outcomes](../../mizar-doc/en/comments.md#residual-association-definitions) のみを扱う。
承認済み comments Q1–Q8 の ① は維持する。attachment 失敗は既定の presentation warning として text/provenance を保持し、修復や verified identity の推測をしない。Warning は既承認要件であり、新しい severity 選択ではない。E/I descriptor は提案しない。profile による昇格は documentation build を失敗させ得るが、code identity、proof validity、extraction policy を変えない。下位 reader の integrity/staleness/schema 失敗と preprocessing 診断は元の意味を維持する。
割当と互換性検証は registry の責務。現在の [PhaseFamily](../../../../crates/mizar-diagnostics/src/registry.rs) は family ごとに一つの既定 severity を持ち、prefix と番号から family を求める。`Documentation` と `Extraction` は存在しない。DD-Q1 は後続更新の表現を問うもので、既存の別 family への流用を認めない。

## 候補範囲と attachment descriptor
候補 A は `W0400`–`W0499` で初期行は下表。候補 B は `W0700`–`W0799` で、各行を +300 する。どちらも既存 warning 範囲・割当と重ならない。中間の番号、未使用番号、E/I 範囲、Extraction 範囲を予約しない。採用範囲内の未使用番号も未割当のままとなる。
各行は DD-Q1–DD-Q6 に依存する候補 descriptor。`phase_family` は `Documentation`、`default_severity` は Warning の案とする。suffix に `documentation.attachment.` を付けたものが `semantic_name` 案で、別の安定 `meaning_key` は DD-Q4 に従う。

| 候補 A code | name suffix | 条件 / 英語 `summary` 案 |
|---|---|---|
| `W0400` | `orphan` | No eligible module or adjacent named-item candidate for the documentation block. |
| `W0401` | `unresolved_target` | Adjacent documentation target lacks supported verified identity, label or target boundary. |
| `W0402` | `ambiguous` | Multiple identities or contradictory candidate/range associations remain for the documentation block. |
| `W0403` | `unverified_target` | Documentation target is invalid or unverified. |
| `W0404` | `unresolved_coordinates` | Required documentation association coordinates are absent or cannot be related to the verified target. |

DD-Q3 の別案は unresolved-target、unverified-target、unresolved-coordinates を一つの descriptor にまとめる。既存の五条件と presentation outcome はすべて維持する。どちらも条件が重なる場合の優先度・多重度・重複排除を定義しない。既存の deterministic ordering に従い、追加の emission policy が必要なら前提定義とし、列挙順 fallback を使わない。
将来の初回割当用 metadata 案は `status = active`、alias・retirement marker・replacement code なし。`since` と `doc_url` は DD-Q5 の対象であり、最終値は実際の割当を指し、未承認の本提案を指さない。[割当](../en/registry.md#allocation-rules)・[retirement](../en/registry.md#retirement-rules)・[互換性](../en/registry.md#compatibility-validation) 規則を維持する。code/prefix/family/default severity/meaning key は恒久的で、retired descriptor は残し番号を再発行しない。semantic name の変更には alias が必要で、message text は identity ではない。

## 位置と採用の前提
利用可能な保存 block range を主位置とし、利用可能な candidate range を関連位置にできる。所有 publication/module/item の provenance と mapped source position を維持し、candidate location を捏造しない。association 座標の欠落は block range の欠落とは限らず、表現可能な block range があれば `unresolved_coordinates` の位置にできる。private-doc の意図的省略だけでは orphan や missing-comment warning にしない。
利用可能な source-bound 主位置がない場合、[spec22 locations](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display) と現在の [record model](../../../../crates/mizar-diagnostics/src/failure_record.rs) は SourceLoad code だけに非範囲位置を認める。artifact offset の保存だけでは loaded-source binding は成立しない。span/EOF を捏造せず、Documentation warning を SourceLoad に偽装しない。DD-Q7 はこの別の表現 gap を示す。範囲と descriptor の選択だけでは解消しない。
missing source、任意の missing-comment lint、Markdown/tag、link/UsedBy、math/rendering、全 Extraction 診断は後続定義に残し、それらの catch-all descriptor を提案しない。source-integrity refusal を `unresolved_coordinates` に格下げしない。既存の共有 ordering/profile 規則の下で source order と完全な identity を保持し、warning text を key にしない。
owner 選択後、別途承認された更新で DD-Q6 の対訳規範 owner を同期し、registry/record 仕様、実装、descriptor/互換性/位置テストを整えてから実際の Doc7 採用に進む。本提案は実装 coverage や code emission の許可を与えない。Doc7 はこれらの前提と自身の契約が整うまで保留、Doc8 も依存したまま。本提案の承認で両 task や phase16 全体が完了することはない。

## Owner Q-Decisions — すべて未選択
以下は本提案固有の質問であり、承認済み comments Q1–Q8 とは別。どの選択肢も提案であって許可ではなく、既定の選択はない。

| 質問 / 影響 gate | 提案 ① | 提案 ② |
|---|---|---|
| DD-Q1 — canonical family / PhaseFamily shape | `Documentation` を追加し、family の唯一の既定 severity を Warning とする。現行の severity-per-family model を維持し、将来の error/info は新しい定義を要する。 | canonical `Documentation` を severity 非依存とし、既定 severity の検証を code range/descriptor に移す。承認された共有 API/model 仕様変更が必要であり、error/info 範囲は割り当てない。 |
| DD-Q2 — 番号範囲 | 候補 A `W0400`–`W0499`、五行案では `W0400`–`W0404`。 | 候補 B `W0700`–`W0799`、五行案では `W0700`–`W0704`。どちらも未使用 slot や他の診断 class を割り当てない。 |
| DD-Q3 — descriptor 集合と名前 | 既存 attachment 条件ごとの表の五 descriptor。 | 選択範囲の offset 0, 1, 2 に `orphan`、`unresolved`、`ambiguous` の三 descriptor。`unresolved` は三つの unresolved/unverified 条件を統合し、structured detail の typed reason key `unresolved_target`、`unverified_target`、`unresolved_coordinates` で区別する。summary は “Documentation target or association cannot be established.”。採用前に detail schema の仕様化が必要。 |
| DD-Q4 — 安定 meaning key | 選んだ `documentation.attachment.<suffix>` semantic name を不変 meaning key とする。 | 不変 key は `documentation.attachment.v1.<suffix>` とし、選んだ semantic name は version なしを維持する。後の rename で key を変えたり意味を移したりしない。 |
| DD-Q5 — descriptor provenance metadata | `since` は採用する規範仕様 revision、`doc_url` はその canonical attachment-code section を指す。 | `since` は実際に割り当てる registry task、`doc_url` は canonical registry attachment-descriptor section を指す。両案とも採用更新で具体 token/target を用意し、上記 status/lifecycle 既定は維持する。 |
| DD-Q6 — 規範所有 / 更新 gate | 対訳 spec22 が family/range、code-to-meaning mapping、summary を所有し、対訳 registry が完全な metadata を導出する。 | 対訳 spec22 が family/range を所有し、canonical attachment descriptor の意味を対訳 registry に明示委譲する。仕様での明示委譲を要し、registry だけの編集では現行禁止を解除できない。 |
| DD-Q7 — 主位置が利用不能 | publication/module/block provenance に基づく Documentation 専用の非範囲位置を対訳 spec22 と共有 record owner で定義し、実装・テスト後に採用する。loaded-source position は捏造しない。 | 現行の span-only 採用を維持し、表現不能 warning と Doc7 全体の完了を別途 owner 承認の位置定義まで明示保留する。warning の破棄や完全 attachment coverage の主張は認めない。 |
