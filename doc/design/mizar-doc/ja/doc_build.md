# mizar-doc Build Inputs
正本は英語: [English companion](../en/doc_build.md)。
## Published Export Metadata
`read_published_export_index(artifact_root: impl AsRef<Path>, expected_manifest_hash: Hash) -> Result<(PublishedManifestRead, BTreeMap<ModuleSummaryIdentity, Vec<VerifiedExport>>), ManifestError>` は後続 build/index planning のための raw 公開 metadata を既存型だけで返す。
必須の expected publication hash を既存 [published reader](./artifact_reader.md#published-input-loading) に渡し、manifest/path/hash 全体を変更せず保持する。既存検証・error layer を部分結果なしで伝播し、追加 read/codec/hash domain/cache/callback/production helper を作らない。
各 VerifiedExport 全体を元の完全な ModuleSummaryIdentity を key とする map に移す。既存 Ord の package id・optional version・optional lock identity・module path・edition 順を使い、module path だけや version 解析順にしない。既存 Manifest 検証が完全 key の一意性を保証し、export が空の module も保持する。
module 内では保存された fully_qualified_name、origin_id の正確な文字列順に並べる。Artifact の元の origin 優先順とは異なり、module ごとの origin 一意性で全順序となる。重複 FQN、Public/Reexported visibility、全 typed field/optional value を保持し、重複除去・勝者選択・name/signature 解析・label 導出・欠落 metadata 合成を行わない。
外側 Manifest package と各 entry の package tuple の一致は別途保証されていない。元の identity を保持し、所有 package 一致・source 来歴・現在性・受理・link resolution の権限を作らない。保存済み proof status はデータのままとする。
追加する source-backed 構造的三系列は normal/explicit metadata と FQN 順序、完全 identity 衝突・重複 FQN・optional 欠落、空 module と委譲先の拒否・byte 保持を扱う。元の実 published-reader fixture と変更しない下位 matrix を再利用し、共有 private publisher は実際の三系列から呼ぶ場合に限る。
## Full Build And Index Inputs
これは tasks4/5 の最初の前提であり、完全な DocBuildPlan/DocIndex ではない。package request/options/current-build binding、source の利用可能性・comment、sidecar label、page/anchor identity、LaTeX、dependency index、link/UsedBy、rendering/extraction、service/診断統合は後続 TODO に残し、欠落を埋めない。新 type/field/profile/dependency や producer/Kernel/proof/reuse/cache 権限を追加せず、Doc 全体・Artifact17・Steps6–8 は未完了に保つ。
