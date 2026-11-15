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

source-load 診断は package/path identity と non-range location を保つ。EOF、挿入点、source span を捏造せず、span-only converter に渡さない。新たな non-range wire encoding はここでは定めない。

## Unsaved-text overlays

[軽量 frontend 診断](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lightweight-open-buffer-diagnostics) は現在 editor text と version を使い、`current` とし、検証済み semantic status を主張しない。version を key とする overlay 診断として公開 build index から分離し、CLI 出力にも `VerifiedArtifact` を含む artifact にも決して渡さない。

overlay の置換は internal03 に従い、current 配送と retained read は [snapshot invalidation](snapshot.md#freshness-and-replacement) と [retention](snapshot.md#currentness-and-retention) に従う。

## Ordering

source の診断を選択するときは aggregator の [canonical record order](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#build-aggregation)（`BuildDiagnosticIndex::records`）を保ち、message text、locale、非同期完了順で並べ直さない。

既存の明示的 native frontend path は ordered map の URI 順で通知をまとめ、各通知内で index record order を保つ。この経路は overlay/semantic の統合 merge 順序や coherent な完全 snapshot を確立しない。

## Implementation boundary

task8 は公開設計だけを定める。task7 は保持された完全 registry と atomic activation を提供する。task9 の配送実装と task10 の scheduling は未完了のままである。既存 native frontend path は syntax-only のままとする。新 transport payload、診断割当、producer 動作を導入しない。

## Test design

この設計 task では test を変えない。task9 は既存の公開 fixture、stale index 抑止、決定的順序の検証義務を維持し、この文書で実装や §23.6/SCA-003 の完了 credit を与えない。
