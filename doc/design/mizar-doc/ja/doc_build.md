# mizar-doc Build Inputs
正本は英語: [English companion](../en/doc_build.md)。
## Published Export Metadata
`read_published_export_index(artifact_root: impl AsRef<Path>, expected_manifest_hash: Hash) -> Result<(PublishedManifestRead, BTreeMap<ModuleSummaryIdentity, Vec<VerifiedExport>>), ManifestError>` は後続 build/index planning のための raw 公開 metadata を既存型だけで返す。
必須の expected publication hash を既存 [published reader](./artifact_reader.md#published-input-loading) に渡し、manifest/path/hash 全体を変更せず保持する。既存検証・error layer を部分結果なしで伝播し、追加 read/codec/hash domain/cache/callback/production helper を作らない。
各 VerifiedExport 全体を元の完全な ModuleSummaryIdentity を key とする map に移す。既存 Ord の package id・optional version・optional lock identity・module path・edition 順を使い、module path だけや version 解析順にしない。既存 Manifest 検証が完全 key の一意性を保証し、export が空の module も保持する。
module 内では保存された fully_qualified_name、origin_id の正確な文字列順に並べる。Artifact の元の origin 優先順とは異なり、module ごとの origin 一意性で全順序となる。重複 FQN、Public/Reexported visibility、全 typed field/optional value を保持し、重複除去・勝者選択・name/signature 解析・label 導出・欠落 metadata 合成を行わない。
外側 Manifest package と各 entry の package tuple の一致は別途保証されていない。元の identity を保持し、所有 package 一致・source 来歴・現在性・受理・link resolution の権限を作らない。保存済み proof status はデータのままとする。
追加する source-backed 構造的三系列は normal/explicit metadata と FQN 順序、完全 identity 衝突・重複 FQN・optional 欠落、空 module と委譲先の拒否・byte 保持を扱う。元の実 published-reader fixture と変更しない下位 matrix を再利用し、共有 private publisher は実際の三系列から呼ぶ場合に限る。
## Optional Published Module Summaries
`read_published_module_summary_index(artifact_root: impl AsRef<Path>, expected_manifest_hash: Hash) -> Result<(PublishedManifestRead, BTreeMap<ModuleSummaryIdentity, Option<ModuleSummary>>), ManifestError>` は後続 label/index 消費側へ既存型の sidecar を返し、新表現を作らない。
必須 expected hash を既存 published-artifact reader に渡し、manifest/path/hash 全体と元の拒否を保持し、部分結果を返さない。optional triple の完全性、schema/source/module/interface binding、witness 到達性は既存 reference validation が所有する。
存在する triple ごとに元の portable path を既存 PublishedArtifactPath/read_published_artifact で再読し、MODULE_SUMMARY_SCHEMA_FAMILY・宣言 schema version・除外なし・宣言 expected store digest を使う。既存 read_module_summary に実 module/interface hash を渡し、StoreIo と ModuleSummary { path, boxed error } を保持する。同じ完全 content hash が初回の schema/source binding を保持するため、private validator・codec/helper/domain/cache を複製しない。
全 manifest module を既存 ModuleSummaryIdentity の完全 Ord で返す。triple 不在は None、label 空の summary が存在すれば Some とし、全 typed field と元の vector 順を保持する。FQN/signature から label を推測せず、export との結合・衝突の重複除去・他 metadata の破棄を行わない。
追加する source-backed 構造的三系列で全 typed field/replay/読取前後 byte、完全 identity・不在/空 summary/module、代表的な hash/path/module/interface/source 拒否を固定する。元の実 published-artifact fixture と下位検証 matrix を再利用し、追加 private helper は実三系列から呼ぶ場合だけとする。構造的 label record は caller-owned metadata であり、producer/受理証拠を捏造しない。
公開データ整合性だけを扱い、source/package/current-workspace 認証・証明受理・link resolution・source 隣接性・trusted reuse・完全 DocIndex/rendering の権限を与えない。label 対応/衝突・page/anchor・import 順序と後続統合は未完了。
## Full Build And Index Inputs
これは tasks4/5 の最初の前提であり、完全な DocBuildPlan/DocIndex ではない。package request/options/current-build binding、source の利用可能性・comment、label 対応、page/anchor identity、LaTeX、dependency index、link/UsedBy、rendering/extraction、service/診断統合は後続 TODO に残し、欠落を埋めない。新 type/field/profile/dependency や producer/Kernel/proof/reuse/cache 権限を追加せず、Doc 全体・Artifact17・Steps6–8 は未完了に保つ。
