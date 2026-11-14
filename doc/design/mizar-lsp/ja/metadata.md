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

task12 は reader の仕様だけを定める。task7 の registry/atomic activation、task9 の診断公開、task10 の build bridge、task13 の hover/type 実装は別単位のままである。Rust API、transport response、機能ごとの lookup algorithm はここで選択しない。

read は公開済み入力と snapshot owner の既存 association を使う。この仕様は OpenBuffer provenance carrier、未保存 artifact の保存/公開、保留中の build scheduling policy を選択しない。

## Test design

この設計 task では test を変えない。task13 の artifact-backed hover fixture と stale data のマークに関する義務を維持する。この文書は module 実装や §23.6/SCA-003 の完了 credit を与えない。
