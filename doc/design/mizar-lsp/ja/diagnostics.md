# Diagnostic publication
正本: [英語版](../en/diagnostics.md)。英語版と同期した日本語訳。

## Responsibility

LSP は [spec §22.1.2](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display) と [internal03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#diagnostic-aggregator) に従い、既存診断を変換・配送する。正規化・identity・重複排除・順序は aggregator の責務であり、LSP は producer を再実行せず、公開 code/name/alias を割り当てない。registry と consumer-adoption gate は上流に残る。

## Build-index publication

選択した `LspSnapshot` に対応する immutable な `BuildDiagnosticIndex` を読み、record と snapshot に属する handle を保持する。CLI、artifact projection と LSP semantic 診断はこの index を共有し、LSP 変換は index を変更しない。obsolete な診断を current として配送しない。

元の [snapshot association](snapshot.md#source-association) により current 配送を検証する。旧 index と新 map を混ぜず、旧 range を current-buffer range に付け替えない。

完全 snapshot の公開と stale read は [snapshot freshness 境界](snapshot.md#freshness-and-replacement) に従う。真正な coherent manifest-backed producer は `27e55711e` で実装され、後続 registry/IR-retention/atomic activation は `LSP-SNAPSHOT-REGISTRY-ACTIVATION` が凍結する。`ArtifactCommitted`、frontend 完了、index 単独ではこの境界を確立しない。

## Range conversion

正確な source snapshot と既存 [range mapper](range.md) を使い、LSP 境界だけで native source range を LSP UTF-16 range へ変換する。mapper の raw OpenBuffer mapping と検証規則にも従う。

§22.1.2 に従い、primary range、保持された secondary anchor とその順序・多重度、Range と Point の区別、与えられた provenance と generated reason を保持する。invalid/unknown/unrepresentable な診断や anchor は明示的な変換失敗とし、黙って捨てたり range を捏造したり公開成功としない。既存 producer の重複排除と代表選択は変えない。

source-load 診断は package/path identity と non-range location を保つ。EOF、挿入点、source span を捏造せず、span-only converter に渡さない。

[承認済み Q1–Q5 A 表示](non_range_diagnostics.md#open-owner-decisions) が compact v1 schema と接続単位の opt-in を所有する。対応接続には ranged frame の後に全件置換の `mizar/publishNonRangeDiagnostics` を送り、未対応接続の non-range batch は送信前に全体を拒否する。詳細 payload は元 record に保持し、Task41 配送証跡は別途扱う。

## Unsaved-text overlays

[軽量 frontend 診断](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lightweight-open-buffer-diagnostics) は現在 editor text と version を使い、`current` とし、検証済み semantic status を主張しない。version を key とする overlay 診断として公開 build index から分離し、CLI 出力にも `VerifiedArtifact` を含む artifact にも決して渡さない。

overlay の置換は internal03 に従い、current 配送と retained read は [snapshot invalidation](snapshot.md#freshness-and-replacement) と [retention](snapshot.md#currentness-and-retention) に従う。

`Server::submit_originating_diagnostic_overlay` は元の native capture を消費し、既存の guard 付き Frontend producer を明示的に一度呼ぶ。通知集合全体が適格な場合だけ、元の immutable index と保持 source association を `LspDiagnosticOverlay` として exact URI/document version を key に保存する。呼出側の index/診断は受け取らず、通常の submission/error と既存 notification/writer API は維持する。未取得結果は保存を維持し、適格な clean 結果は当該 source の空 index へ置換する。

`Server::diagnostic_overlay` は元 cohort の raw tuple/incarnation と live Driver guard が current の間だけ URI/version で immutable な `Arc<LspDiagnosticOverlay>` を選択する。current は editor-current Frontend 出力であり semantic verification を意味しない。read は当該 document の元 record と native source tuple だけを公開し、cohort index/association は identity/retention のため内部で共有する。無効化後は元 source の参照だけを許す。新しい適格結果は重なる key を置換し、受理した編集/close は該当 cohort の選択を除去し、shutdown/exit は全選択を除去する。無視/不正/無関係 traffic は除去しない。完全 snapshot の activation 成功は一致する URI/version key を除去し、失敗/partial work は消去しない。reader は最後の参照解放まで元 OpenBufferOverlay lease を保持する。保存は Server 内で semantic publication、CLI rendering、artifact projection から分離し、自動 host integration は task26/DRIVER-G-012 に残す。

## Ordering

source の診断を選択するときは aggregator の [canonical record order](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#build-aggregation)（`BuildDiagnosticIndex::records`）を保ち、message text、locale、非同期完了順で並べ直さない。

既存の明示的 native frontend path は ordered map の URI 順で通知をまとめ、各通知内で index record order を保つ。この経路は overlay/semantic の統合 merge 順序や coherent な完全 snapshot を確立しない。

## Implementation boundary

`src/diagnostics.rs` が task9 と承認済み non-range 表示を実装する。`Server::snapshot_diagnostic_notifications` は current な保持診断 publication を選択し、変換エラーは batch 全体を拒否する。opt-in 済みの live 接続で既存 selection を失うか stale になれば null-selection の空置換を返す。受理した編集無効化は serving loop または明示 diagnostic writer 用に消去をキューへ入れ、新 publication が古い消去を置換する。terminal 接続はキューを破棄する。

通知は URI 順、capture 時の document version を使い、正常 source の空リストも含む。primary range と順序を保つ secondary `relatedInformation` は元の raw map で変換する。診断の `data` は既存 handle を `{snapshot: <published BuildSnapshotId>, id: <ordinal>}` として保持 index 内の未変更 record を参照する。anchor の形、generated reason、provenance、note、intent は新しい表現に複製せず元の record に保持する。この handle は currentness や編集権限を与えない。

`Server::publish_snapshot_diagnostics_to_writer` は変換と書込みの間も選択 snapshot を保持し、各 frame 前に currentness と選択 identity を検証する。変換・I/O エラーは伝播し、外部書込みに atomic-currentness、rollback、retry の保証はない。build scheduling/event consumption は task11 に残り、既存 native frontend path は syntax-only のままとする。

## Test design

task9 の test は真正な coherent 公開、stale index 抑止、決定的順序、正確な source 変換、保持 record identity、明示的 batch 失敗、frame 配送を検証する。task11 や §23.6/SCA-003 の完了 credit は与えない。

`server.rs::diagnostic_overlay_tests` は真正な native Frontend work と coherent publication により、document/version key、source filtering、置換/clean clearing、元 source の currentness、最後の reader までの lease 保持、完全公開時の選択的除去を検証する。非空 overlay の前後で元 semantic index、実際の CLI 出力、直列化した VerifiedArtifact を比較し、host integration の credit は与えない。

別の [ranged diagnostic host input](ranged_diagnostic_host.md) は full semantic snapshot なしで実 terminal source-error record を受け入れます。診断 notification/writer は選択されたその publication を使い、canonical data handle、元 map、batch 全体の変換検証を保持します。
