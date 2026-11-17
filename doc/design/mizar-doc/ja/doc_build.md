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
## Published Label Export Candidates
`read_published_label_export_candidates(artifact_root: impl AsRef<Path>, expected_manifest_hash: Hash, module: &ModuleSummaryIdentity, label: &ExportedLabelSummary) -> Result<(PublishedManifestRead, Option<Vec<VerifiedExport>>), ManifestError>` は実際の公開 label record 一件の owner 候補を既存型だけで問い合わせる。
同じ root と必須 expected publication hash で read_published_export_index、その後 read_published_module_summary_index を呼ぶ。query record が不在・偽造でも完全検証と元の error を lookup より先に行う。最初の manifest/path/hash 全体を保持し、部分結果・追加 read/codec/hash/default/cache/helper・下位変更を作らない。両 reader が検証した完全 module 集合は同じ選択 publication に属する。
完全な ModuleSummaryIdentity が同じで、元の exported_labels record の全 field が supplied label と等しいことを要求する。module 不在・summary 不在・label 空・完全一致 record なしは None。この record query は不在理由を区別せず、完全 metadata は既存 summary reader が保持する。
実 record では、その module の保存 fully_qualified_name が保存 owner_fully_qualified_name と等しい export 全件だけを Some で返す。全 field と既存 FQN/origin 順を保持し、owner 対応なしは Some(empty)、重複 owner FQN は全候補とする。module identity を跨ぐ結合・勝者選択・反復 label/export の除去・name/signature からの label 導出・label field からの export origin/range/kind/visibility 推定は行わない。
production 関数一つだけを末尾追加し、元 Rust/Cargo/fixture/expectation/trace/ledger byte は保持する。doc_label_export_candidates_* の構造的公開物テストを二系列だけ追加し、新 helper は作らず既存 Doc publication fixture と Artifact writer を使う。unique/multiple/unmatched 候補、全 typed field/順序/replay/読取前後 byte、module path が同じで完全 identity が異なる場合、同じ label text、全 record の偽造、不在 record、lookup 前の代表的な元 error layer を扱う。下位 matrix 全体は再実行しない。
正確な保存 metadata 候補だけであり、visible reference の解決・anchor/page/link・warning・source attachment・semantic export producer・完全 DocIndex ではない。保存 proof status はデータのままとし、source/package/current-workspace/受理/witness/reuse/cache 権限、import 順/dependency index、完全 tasks4/5/rendering・Steps6–8 は別課題に保つ。
## Full Build And Index Inputs
上記 reader は `build_doc_plans` と未解決 `DocIndex` 入力の前提である。Doc4 提案は以下の Q1–Q3 の①で承認済み、Q4 は Doc6 に残す。この API は rendering や広い Doc4/5 統合の完了を意味しない。

### Phase-16 Requests
[internal05](../../internal/ja/05.documentation_extraction.md#phase-16-request) の request の役割を用いる。選択 package/root、artifact-manifest reference、optional source bundle、documentation/extraction configuration、profile、build snapshot は統合入力であり、現 reader が追加認証するものではない。
`render_docs` と `extract_code` は独立する。両方無効なら phase16 処理はなく、documentation のみ・extraction のみ・両方の場合は各 configuration と warning policy を保つ。consumer の無効化は phases0–15 の検証を変えない。extraction の選択・runtime readiness・lowering は tasks16–22 が所有する。
phase15 公開後の scheduling と snapshot binding は driver/service が所有する。doc_build は明示的に選ばれた publication を消費し、新しい manifest を探したり caller の path を現在性の証明にしたりしない。各 manifest は caller 指定の expected publication hash を既存 reader に渡す。scheduler・service・cache・source-to-artifact producer はここでは定義しない。

### Package Planning
選択 package publication ごとに独立して計画し、完全な manifest/path/hash と元の module identity を保つ。複数 package の request は package plan の集合であり、namespace を統合しない。package metadata は module identity と分け、planning は Q3 に従い選択 package tuple を manifest と各 module tuple に照合する。
semantic documentation 構築前に [published-input reader](./artifact_reader.md#published-input-loading) で必要な artifact 検証を全て行う。拒否された publication の使用可能な plan は返さず、既存 error layer を保つ。source なしの artifact 読取成功は source の存在や完全な rendering/extraction readiness を意味しない。optional native source/lexical read は独立した hash/path/map/error 契約を保つ。
documentation option は上位設計の output destination と private-documentation/profile 選択を含み、新たな option 優先順位は選ばない。output path は出力先であり、変更すべき artifact input path ではない。dependency documentation index は package/publication identity に対応する明示的 reference であり、後続 link に使うが proof・import resolution・dependency 選択の権限にはしない。
package-wide link resolution の前に各 package の module 入力を集める。各入力は完全な ModuleSummaryIdentity、verified artifact reference/hash、全 export、optional summary metadata を保つ。native source の有無と comment は Q2 の必須保存 source_file/range と別である。空 module を保ち、summary 不在・空 summary・native source 利用不可を区別して欠落データを捏造しない。

`DocBuildRequest` は独立した `Phase16Profile` の consumer switch、warning/private-documentation policy を保つ。各 `DocPublicationRequest` は expected package/hash、artifact root、output destination、optional source bundle、optional dependency-index reference、module ごとの optional な意味上の import vector を保つ。documentation 無効時は publication を読まず plan を返さず、extraction は独立の flag/policy により後続へ委譲する。
`build_doc_plans` は SnapshotRegistry、request/snapshot id、expected workspace root を取る。registry に保持された request/root の current snapshot を要求する。各 publication 内では既存 reader が expected hash を検証した後に package/source binding を行う。選択 package/manifest/module の package tuple の完全一致と snapshot source 内の package 所属、必須 package/module version、snapshot source の module/path/edition/source path/hash の一致、artifact/manifest の toolchain・lock・verifier configuration の一致を要求する。不一致は `DocBuildError::Binding`、reader error は `Publication(ManifestError)` として保ち、部分結果や source fallback を返さない。
各 `DocBuildPlan` は選択 request/profile、registry snapshot/request id、完全な PublishedManifestRead を保ち、必須 package_version と完全 identity module map を持つ `DocIndex` を内包する。各 `DocModuleInput` は元の ModuleArtifactEntry（必須 source_file）、完全な FQN/origin 順の VerifiedExport（必須 range）、optional な完全 ModuleSummary を保つ。index の完全 record query `label_export_candidates` は None・Some(empty)・保存 export の全候補を返し、resolved-link 権限を与えない。
package plan は完全 package tuple/publication hash、dependency reference は package tuple/publication hash/path で canonical 順にし、意味上の import vector 順と metadata 不在を保つ。重複 package/publication request は destination/option の勝者を選ばず拒否する。serialized schema・comment・citation・page・anchor・link・UsedBy の完成を導入しない。

### Index Inputs And Later Completion
[internal05 の index model](../../internal/ja/05.documentation_extraction.md#documentation-index) は package/publication 来歴、schema、item、link、UsedBy を表し、後続 consumer の目標であって raw export map の形ではない。module/package/version 間の名前衝突を区別するため、完全 module identity と元 export の origin/FQN を保つ。FQN のみの map に潰したり重複の勝者を選んだりしない。
[保存 label query](#published-label-export-candidates) は候補のみを返す。None・Some(empty)・複数候補は一意な documented item を確立しない。label は実際の exported label metadata から取り、signature や名前解析から作らない。欠落する対応や曖昧な候補は後続 index/link consumer への未解決入力として残し、anchor を補作しない。
spec [24.2](../../../spec/ja/24.documentation_generation.md#242-cross-referencing-and-hyperlinks) が label/FQN reference、現在 module の後に import 順で探索する規則、曖昧性 warning、exported-label の安定 anchor を所有する。canonical 格納順とは別に意味上の import 順を保つ。Doc9/10 が resolution と診断導入を所有し、import 順や dependency metadata の不在を解決済み graph として扱わない。
Doc6/7/8 は comment 取得・検証済み identity への attachment・Markdown summary、Doc11/12 は math metadata、Doc9/10 は link resolution、Doc13–15 は page layout/rendering/search emission、Doc23 は output publication を所有し、Doc24 は決定性を検証する。kind・range・proof summary・signature・LaTeX は存在する場合の保存 projection とする。UsedBy 完成は citation metadata と後続 index/render consumer を待ち、raw export のみで完成を主張しない。serialized DocIndex schema や page/anchor 衝突方針は追加しない。

### Determinism And Consumer Invariants
選択 publication、binding 済み source 入力、lock context、dependency index、option、target、toolchain、schema version が同一なら、planning/index 入力は列挙順や worker scheduling に依存しない。既存の完全 ModuleSummaryIdentity Ord と module 内の正確な FQN/origin 順を使い、重複・optional 不在・意味上の vector 順を保つ。version string を解析して別の順序にしない。
後続 module/item と search/cross-reference index は安定した module/FQN key を使う。page/anchor identity は spec24 に従い、衝突処理はその所有者に残す。再現可能な出力から timestamp を除外するか local metadata として隔離し、診断順は [architecture12](../../architecture/ja/12.diagnostics_and_lsp.md) に従う。完全 identity の tie-breaker は候補保持のためであり、visible reference の優先順位を決めない。
plan/index は派生 consumer product であり、意味解析・証拠受理を再実行せず、proof status や artifact/witness を変更しない。保存 proof status はデータのままとする。integrity/schema/staleness の失敗は error、presentation の問題は後続 documentation 診断と graceful degradation で扱い、warning 昇格は選択 profile のみが行う。documentation の品質は proof validity を変えない。
cache reuse・compiler IR access・一般 artifact emission は既存 producer/validation gate を維持し、本提案では権限を与えない。documentation 出力の再生成・削除は検証結果を変えない。Doc 全体・Artifact17・Steps6–8 は未完了である。

### Owner Q-Decisions
所有者は Q1–Q3 を①で承認した。この決定は新表現だけに適用し、後続 task の決定はその task に残す。

| Question | 矛盾または未定義の権威と求める判断 |
|---|---|
| Q1 — version optionality | Architecture13 に従い新 plan/index の Version は必須。保存 version 欠落を拒否し、保存 optional identity は変えず version を合成しない。 |
| Q2 — source optionality | Architecture13 に従い保存 source_file と item source range は必須。placeholder path/range を作らず、optional native source の有無で必須 metadata を補わない。 |
| Q3 — package/current-build binding | caller の expected publication hash と current build snapshot から owning-package/current-workspace binding を認証し、不一致は plan を返さない。raw reader だけでは引き続きこの binding を与えない。 |
| Q4 — Doc6 コメント供給元 | [Doc6 Q1](./comments.md#owner-q-decisions) で emission 時 Artifact 投影を承認済み。定義と実装前提が Doc7 を gate し、optional native Lexer input はコメント fallback にしない。 |

### Doc5 Acceptance Matrix
planning/index test は六つの構造的 publication case を上限とし、下位 reader の検証 matrix は元の所有者に残す。

| Input shape | 必要な観察 |
|---|---|
| documentation/extraction switch の全四組合せ | 有効 consumer のみ、検証は変えず extraction は委譲する。 |
| module path/FQN が衝突する二 package、空 module、dependency index | 全 metadata と dependency reference の完全 package/publication identity を保ち、勝者選択や namespace 統合をしない。 |
| 同一入力で列挙順を反転し worker scheduling を変更 | canonical plan/index 入力は同一、意味上の import 順を保つ。 |
| version/source 欠落、summary の不在と空 | Q1/Q2 の必須 metadata 欠落は拒否し、optional summary/source 利用可否の区別を保つ。version/path/comment/label を補作しない。 |
| 不正 manifest/hash/schema/path、古い artifact、package/root/snapshot 不一致 | 既存検証の拒否と Q3 の不一致拒否を保つ。拒否 publication の使用可能な plan、source fallback、部分 semantic index を返さない。 |
| label/dependency/import/citation/comment metadata の欠落・曖昧性 | None・Some(empty)・複数候補と候補の全 field を planning 入力まで保つ。欠落を残し、link/UsedBy/attachment や proof credit を捏造しない。 |
