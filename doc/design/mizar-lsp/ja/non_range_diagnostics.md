# 非 range diagnostic 表示案
正本: 英語。[English companion](../en/non_range_diagnostics.md)。

## Status and authority

**2026-11-16 に owner が Q1–Q5 の A を全承認した。** [crate plan](00.crate_plan.md#task-index) の実装契約が選択を固定する。以下の選択肢は当初の判断対象を記録し、未選択の B は実装禁止。Task41 Audit G6 の配送証跡は別途扱う。
権限: [spec §22.1.2](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display)、[source-loading primary locations](../../mizar-diagnostics/en/failure_record.md#source-loading-primary-locations)、[aggregation](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#build-aggregation)、[diagnostic conversion](diagnostics.md#range-conversion)、[snapshot currentness](snapshot.md#freshness-and-replacement)。既存 code・request identity・正規化/deduplication・順序は Diagnostics が所有し、LSP は表示と配信だけを所有する。
LSP 3.17 の [Diagnostic](https://raw.githubusercontent.com/microsoft/language-server-protocol/gh-pages/_specifications/lsp/3.17/types/diagnostic.md) は `range` が必須で、[publishDiagnostics](https://raw.githubusercontent.com/microsoft/language-server-protocol/gh-pages/_specifications/lsp/3.17/language/publishDiagnostics.md) はその object を運ぶ。range の省略/null 化や path のみの record を `relatedInformation` に置く方法では標準 non-range encoding にならない。これは以下の拡張案の理由であり、承認ではない。

## Required identity and ownership boundary

SourceId・読める text・editor document がなくても、source-load record は要求された package と正規化済み package-relative path を識別する。non-range variant、既存 code/severity/message、canonical snapshot-scoped handle を保持する。同じ path でも別 package は区別し、message text や filesystem-local absolute path を identity としない。失敗した request の代わりに EOF・insertion point・source span・line/column・underline・excerpt・架空の document version を与えない。
non-range primary を span-only converter に渡さない。実在する secondary anchor は元の source による厳密な検証が必要であり、順序/重複数・shape・provenance・generated reason を retained record 経由で保持する。未知 variant・stale record・表現不能な retained anchor は明示的に失敗し、破棄・再正規化・新 deduplication・provenance の推測を行わない。
どの表示案も一つの genuine immutable diagnostic selection と元の association を参照する。handle/wire token は currentness・edit・proof・artifact・acceptance 権限を与えない。retained read は古い位置を live text に再対応付けしない。既存 CLI/artifact projection・overlay isolation・ranged delivery・他 unit の未決事項は本定義の範囲外とする。

## Open owner decisions

以下の問いはすべて A で解決済み。提案値や推奨という表現は、変更なく承認された当初案の記録である。

**Q1 — wire carrier は何か。** A: 専用の server-to-client JSON-RPC notification（仮称 `mizar/publishNonRangeDiagnostics`）で structured non-range replacement set を送り、標準 ranged notification を併用する。B: 同じ structured set を返す専用 client pull request とし、unavailable/stale result と client polling/refresh lifecycle を実装前に明記する。**A を推奨**: 既存 push delivery に合い、能動的な消去が可能。B は unsolicited payload を避けるが refresh/availability 契約が別途必要。以下の Q4/Q5 は A を記述するため、B を選ぶ場合は対応する pull lifecycle を coding 前に owner が承認する。標準 push/pull の Diagnostic array、偽 URI と zero range、情報 popup はこの structured non-range carrier を提供しない。

**Q2 — support をどう交渉するか。** A: initialized connection ごとに `capabilities.experimental.mizarNonRangeDiagnostics: {version: 1}` で opt in し、server も `capabilities.experimental` に同じ supported version を広告する。欠落・不正・未対応 version では拡張を無効にし、non-range を含む batch は frame を書く前に明示的に失敗する。ranged-only の既存動作は維持する。黙った省略・popup fallback・non-range 配信成功の主張はしない。B: initialization で拡張を必須とし、非互換 client を明示的に拒否する。その client は通常の ranged-only session も利用できない。互換性のため **A を推奨**。未知の custom notification を受信するだけでは acknowledgment/support にならない。field 名/version は提案であり、現 initialization の変更ではない。

**Q3 — structured payload はどこまでか。** A: 下記の compact v1 envelope とし、厳密な package/path・primary kind・既存 display field/handle・実在する secondary location を含め、豊富な payload は元の record に保持する。B: 全 diagnostic field/anchor/note/detail/fix/explanation を lossless に wire projection し、codec と未対応値の規則を実装前に別 freeze する。公開 record codec を新設せず既存 ranged handle model に従う **A を推奨**。handle は semantic name・stable detail key/reason・note（欠落と空を区別）・intent・anchor shape/provenance・attachment を保持するが、新 query/action route を約束しない。B は schema/互換性の責務が増える。

Q1-A/Q3-A の条件付き envelope（field 名と値は提案）:
```text
params = {version: 1, selection: <opaque connection-local publication token> | null,
          records: [<record>, ...]}
record = {location: {kind: "sourceLoad", package: <PackageId string>,
                    path: <NormalizedPath string>}, code: <existing public code>,
          severity: <existing LSP severity mapping>, message: <exact message>,
          data: {snapshot: <existing published BuildSnapshotId string>, id: <existing ordinal>},
          relatedInformation: [<validated real secondary locations/messages>, ...]}
```
array は `BuildDiagnosticIndex::records` の non-range subsequence の順序を保つ。推測した URI で group 化したり text で sort しない。string は JSON escape と owner の package/path の underlying value を用い、debug format・filesystem resolution・path 連結を使わない。`data` は現 ranged diagnostic encoding を使う。secondary conversion と note-span validation は既存 original-map 規則を再利用し、secondary が空なら primary の text は不要。token は同じ snapshot の別 publication を区別する live connection 内限定の値で、永続 identity や handle の代わりではない。null selection は Q4-A の空 array による消去専用とし、record があれば非 null selection と一致する handle snapshot を要求する。

**Q4 — 何を置換し、いつ消去するか。** A: 各拡張 notification が、その connection の選択済み diagnostic publication の non-range set 全体（含まれる全 package）を置換する。genuine clean replacement は空 set を送り、受理した invalidation/selection 喪失は配信可能な connection に `{selection: null, records: []}` を送る。元の selection identity/currentness check を維持し、同じ snapshot id でも古い record は復活しない。Shutdown/exit は local selection/delivery state を破棄し、terminal closure 後の write は約束しない。client は connection 終了時にその display を破棄する。B: package ごとの replacement set と明示的 tombstone により、他 package を独立して保持する。別 publication 間の merge state を追加しない **A を推奨**。B は package removal・overlapping publication・tombstone 規則の owner 定義が先に必要。どちらも古い retained record を current とせず、snapshot admission を変更しない。消去は除去を伝えるだけで、新しい current diagnostic ではない。

**Q5 — mixed batch と失敗の保証は何か。** A: 書込み前に capability・selection・全 ranged/non-range record/anchor・JSON frame を preflight し、未対応/不正 record が一つでもあれば batch 全体を拒否する。有効かつ対応済みの selection では、既存 URI 順の ranged notification の後に拡張 replacement frame を一つ（空の場合も）送り、各 carrier 内で canonical order を保つ。nonempty/current frame ごとに同じ selection と currentness を再検証する。消去 frame は、その除去より新しい replacement selection がないことを検証する。conversion/I/O error を伝播し、currentness 喪失で以後の write を止める。既送信 frame は rollback できず、retry/acknowledgment/atomic client display を保証しない。B: ranged/non-range を一つの交渉済み拡張 frame にまとめ、新 ranged carrier と client adoption を導入する。stream I/O はその一 frame の途中でも失敗し得る。既存 ranged interoperability と batch 全体の conversion failure を維持する **A を推奨**。B は複数 frame 間の display skew を減らすが wire surface が広がり、schema は別承認が必要。どの案も外部 write の原子性を保証しない。

## Conditional validation and implementation handoff

owner 判断後、選ばれた carrier/schema/capability/lifecycle を presentation 実装契約で freeze する。推奨を承認と解釈しない。非 A 案に従属する規則が未定なら実装前に owner へ戻す。従来の [ranged host admission](../en/ranged_diagnostic_host.md#admission-and-ownership) は SourceLoad を除外し、現 converter は captured native source を必要とする。wire shape の承認だけでは両 gate を緩めない。別途の [source-load 前提](../../mizar-driver/ja/request.md#native-source-capture) が genuine capture/retention/host admission を定義し、package/path と元の native currentness guard を保持し、失敗 source の text を要求したり coherent full snapshot を合成しない。前提の後に表示実装、次に Task41 を進める。caller が任意に作った index は positive host-delivery evidence ではない。
以下の document-review case は、対応する owner 承認後にのみ executable obligation となる。上限六件。追加には authority clause または .miz test が必要。

| Case | Input と必要な evidence |
|---|---|
| 1 | SourceId/text がない missing/unreadable/invalid-text source を含む genuine source-load E0600–E0603 shape。選んだ carrier は package/path/non-range kind と元の handle/code/message を保持し、座標/version を捏造しない。typed conversion fixture だけでは host delivery の証明にならない。 |
| 2 | 別 package の同じ path・同じ message・複数 record・実在する反復 secondary anchor。identity・canonical subsequence・順序/重複数・retained payload fidelity を保持し、不正な実 anchor は明示的に失敗する。 |
| 3 | 対応済み・欠落・不正・未知の capability version。選んだ handshake/refusal outcome を満たし、未対応 frame・黙った破棄・成功の主張をしない。ranged-only 互換性は Q2 に従う。 |
| 4 | nonempty から clean・package removal・edit/invalidation・同 snapshot replacement・Shutdown/exit。選んだ replacement/clear lifecycle を守り、obsolete record を current と再分類せず、post-terminal write を約束しない。 |
| 5 | valid mixed input・primary text のない non-range-only input・不正/未知 record/anchor・stale selection。選んだ preflight outcome に従い、conversion/capability failure は zero frame。source-load primary を span-only conversion しない。 |
| 6 | real native host provenance による実際の framed delivery。batch 途中の replacement と writer failure で厳密な payload/currentness check・明示 error を確認し、rollback/atomic-display を主張しない。既存 ranged frame・CLI/artifact/overlay・retained-read control を維持する。 |

`crates/mizar-lsp/src/diagnostics.rs` の `conversion_fails_the_whole_batch_for_source_load_unknown_and_invalid_spans` は現 refusal regression として維持する。承認後の実装では recognized source-load branch のみ変更可能とし、invalid/unknown refusal intent を保つ。`server.rs` の native wire framing と writer-currentness/I/O control を再利用する。Task41 は schema/converter fixture に加えて実際の承認済み non-range delivery evidence を必要とし、本提案は runtime や §23.6/SCA-003 credit を与えない。
