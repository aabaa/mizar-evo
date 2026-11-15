# Published metadata reader
正本: [英語版](../en/metadata.md)。英語版と同期した日本語訳。

## Responsibility

metadata reader は [internal03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-bridge) に従い、IDE 機能のために公開 artifact metadata を読む。hover/navigation は公開 artifact と `LspSnapshot` metadata を使い、生の compiler IR は読まない。この reader は型検査、名前解決、証明受理、artifact の変更を行わない。

## Published inputs

[snapshot freshness](snapshot.md#freshness-and-replacement) に従い、選択した snapshot の manifest が解決した出力を使う。`VerifiedArtifact.expressions` を既存の [ExpressionMetadata](../../mizar-artifact/en/verified_artifact.md#expression-metadata) として読み、producer が所有する式 identity、native source range、描画済み summary を保つ。任意 metadata は任意のままとし、不足する型を推論したり symbol を解決したりしない。

既存の [ModuleSummary](../../mizar-artifact/en/module_summary.md#scope) projection から module identity と importer 向けの公開 symbol、label、lexical、reexport 情報を読む。module summary は式 metadata を含まず、`VerifiedArtifact.expressions` の代用にも、公開されていない局所的 semantic 情報の供給元にもならない。

schema、互換性、canonical order、検証は Artifact が所有する。既存の [validating reader](../../mizar-artifact/en/verified_artifact.md#reader-and-writer-requirements) と [summary reader 規則](../../mizar-artifact/en/module_summary.md#reader-and-writer-requirements) を使う。この設計で field、summary の形、format、検証方針を追加しない。

## Snapshot association and freshness

一回の metadata read は一つの immutable な [source association](snapshot.md#source-association) と対応する公開 metadata を request 全体で保持する。古い式 row や summary と新しい snapshot の source/map を混ぜない。range 変換は既存の [range 境界](range.md) と対応する元 source を使う。

snapshot ごとの current/stale_artifact/pending 規則と [retention](snapshot.md#currentness-and-retention) をそのまま適用する。stale semantic data は古いという理由だけで隠さず、明示的に stale とする。保持された stale range は元 source に対応したままとし、current buffer への配送と安全でない編集には snapshot の検証規則を適用する。

artifact 入力が欠ける場合は unavailable のまま IDE 機能を段階的に縮退させ、空の成功 snapshot や予定済み作業を意味させない。syntax-only の open-buffer overlay は semantic metadata を供給しない。この reader は currentness を確立せず、等しい build id や保持 handle からも導かない。

## Projection boundary

sealed phase output からの projection は上流の [Artifact Projection Boundary](../../internal/en/06.ir_storage_and_snapshot_handles.md#artifact-projection-boundary) が所有する。公開後の [artifact reader](../../internal/en/06.ir_storage_and_snapshot_handles.md#artifact-projection) は internal IR storage の代わりに安定した projection を読む。

metadata reader は生の AST/IR を読み込まず、compiler storage reference をたどらず、compiler phase を通じて不足する semantic metadata を再構成しない。描画済み metadata や式 identity は証明権威を確立しない。

## Implementation boundary

`Server::snapshot_hover` は保持された一つの publication の式 metadata を、元の OpenBuffer URI と UTF-16 位置で読む。位置を含む最小の半開区間を選び、同じ区間なら式 identity で決定する。plaintext hover は公開済みの surface、identity、および存在する型、symbol、coercion、thesis、overload summary を含み、欠けた field は再構成しない。

current hover は元の map で変換した LSP range を含む。stale hover は metadata を保持し、`stale_artifact` と元 source の URI/version、検索座標が保持された元 source に属することを明記して、current-buffer の LSP range を省く。不正な source association/range は明示的に失敗し、metadata がなければ hover を返さない。

`Server::handle_message_with_metadata` が hover request に Driver/registry の freshness を供給し、build bridge はこの経路を使う。通常の handler は request を検証し、metadata host がなければ hover を返さない。実行ファイルの host/build 統合と navigation は別作業のままである。

read は公開済み入力と snapshot owner の既存 association を使う。この仕様は OpenBuffer provenance carrier と未保存 artifact の保存/公開を選択せず、build scheduling は [build bridge](build_bridge.md) が所有する。

## Test design

artifact fixture は任意 metadata の表示、式の選択、元の raw source の mapping を検証する。実際の coherent publication は request 配信、stale 表示、置換を検証する。この reader は §23.6 や SCA-003 の annotation projection audit を完了させない。
