# Explanation queries
正本: [英語版](../en/explain.md)。英語版と同期した日本語訳。

## Responsibility

LSP は internal03 の [Explanation Queries](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#explanation-queries) に従い、既存の explanation handle をオンデマンドに解決する。explain command は [explanation artifact と上限付き query service](../../architecture/en/12.diagnostics_and_lsp.md#request-sources) を読み、compiler state を一括抽出しない。

既存の [ExplanationRef](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#explanation-reference) を使う。これは埋め込み trace ではなく handle である。[診断公開](diagnostics.md#build-index-publication) はこの handle を保持し、[code-action command](code_action.md#request-sources-and-conversion) は snapshot と diagnostic handle を捕捉して実行時に再検証する。explanation 種別、handle field、payload schema はここでは追加しない。

## On-demand resolution

internal03 の 4 段階の flow に従う:

1. 要求は `DiagnosticId`、`ExprId`、`VcId` のいずれかと snapshot id を渡す。
2. `ExplanationStore` は artifact hash または cache key に照らして handle を解決する。
3. query service はサイズと深さの上限を強制する。
4. 応答は freshness、source range、上限付きの構造化 explanation data を含む。

[既定の診断 record](../../architecture/en/12.diagnostics_and_lsp.md#explanation-data-is-lazy-and-bounded) は explanation 種別、関連 id、出力された場合の artifact path、上限付き preview text を保持する。詳細 data は [Explanation Store](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#explanation-store) の裏付けとなる artifact、cache-backed query data、専用 explanation file に遅延保持する。cache record は任意の裏付け/高速化 data であり、診断はそれがなくても解釈可能なままとし、cache-backed explanation は証明状態を決定しない。

## Latency and size limits

上位文書はレイテンシ、サイズ、深さの数値上限を定めていないため、この仕様も追加せず、新しい timeout や切り詰め policy を定義しない。

## Freshness and stale handles

handle は作成時の snapshot または artifact hash の範囲内だけで有効である。[incremental update flow](../../architecture/en/12.diagnostics_and_lsp.md#incremental-update-flow) は旧 snapshot の stale な explanation handle を無効化する。旧 snapshot の handle には stale-handle 応答を返す。一致する code/span があれば、bridge は現在の診断に対する再問い合わせを提案してよい。

既存の [snapshot association](snapshot.md#source-association) と [range 変換](range.md) に従い、応答の freshness と source range を元の source snapshot に対応付ける。古い range を現在の buffer の range として付け替えない。

## Missing payload and cancellation

[LSP Snapshot Errors](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-snapshot-errors) に従い、explanation payload の欠落時は compact diagnostic preview を表示し、explanation が利用不能であると報告する。explanation 内容を捏造しない。

[explanation query はキャンセル可能](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#cancellation) である。公開済み snapshot は不変のままであり、公開後はキャンセルしない。

## Implementation boundary

task18 は query の挙動だけを定める。task9 の公開、task17 の code action、task19 の explanation-query 実装、mizar-diagnostics task15 の ExplanationStore 実装は別単位のままである。producer、diagnostics、artifact、snapshot の挙動を変更しない。

## Test design

この設計単位はコードもテストも変更しない。task19 は tasks17/18 と mizar-diagnostics task15 を依存として、query fixture、上限付き payload、裏付け data 欠落時の劣化という既存の義務を保持する。task19 と explain module は未完のままで、§23.6/SCA-003 の credit は変更しない。
