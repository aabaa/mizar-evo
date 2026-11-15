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

`Server::handle_message_with_explanations` は保持中の公開単位と host が渡す不変の `ExplanationStore` から `mizar/explain` を処理する。params は published schema 形式の `snapshot` と、`diagnosticId` ordinal、`exprId`、`vcId` subject string のいずれか一つを含む。subject query は既存の診断に付いた handle を選び、欠落・曖昧な一致は unavailable とする。store は登録済みの上限付き payload を読み、LSP は producer file や service を開かない。この host を持たない通常の stdio は unavailable を返す。

応答は status (`available`、`unavailable`、`stale_handle`)、freshness、元 snapshot、元 URI/version/range、任意の既存 preview と上限付き payload の format/text/truncation/summary hash を持つ。旧 snapshot の要求は `stale_handle`、編集後の保持中の読み取りは元座標の `stale_artifact` とする。host の cancellation predicate を処理前と応答前に確認し、公開状態を変えずに RequestCancelled を返す。

## Test design

query fixture は既存 subject の選択、store の結果、上限付き payload の厳密な保持と native UTF-16 変換を検証する。実際の公開 fixture は host 選択と freshness、型付き fixture は元の source location を確認する。explanation content と stale location を合わせた host 配信の検証は native explanation producer の接続を待つ。cancellation、不正要求、framed stdio の劣化も対象とする。§23.6 は partial、SCA-003 は変更しない。
