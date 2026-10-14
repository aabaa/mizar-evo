# Source Services

> 正本は英語です。[英語版](../en/frontend_adapter.md)。

## Disk SourceLoad service

`PhaseRegistryBuilder::register_source_load()` は `mizar-frontend` 所有の実 disk-only service `SourceLoad` を `PipelinePhase::SourceLoad` に登録する。引数なしの登録メソッドは単一 phase の descriptor/catalog identity を固定し、別の public service type を公開しない。Frontendは [Disk Frontend service](#disk-frontend-service) として別登録し、SourceLoadは後続phaseを実行しない。
`SourceLoadInputs<'a>` は capture 済み `BuildSnapshot`、`BuildPlan`、`ModuleIndex`、呼出元 `SessionIdAllocator` を借用する。`PhaseExecutionResources<'a>` と `PhaseExecutionContext<'a>` が optional source inputs を運び、実 submission が scheduler dispatcher に渡す。後続 service がない default graph ではこの経路は未実行となるため、A5 は新しい graph/profile を作らず registry 直接実行を検証する。登録 service は allocator や payload を保持しない。

## Binding and execution

module work unit に対応する disk SourceVersion、workspace package plan、package index entry、workspace module entry がそれぞれ一意であることを要求する。package/module/path/edition、package root/source root/manifest、snapshot/workspace root を照合し、重複や矛盾を拒否する。module の source-relative path は package-relative path と source root に整合させる。安全でない workspace-relative plan root は拒否する。
canonical package root は canonical captured workspace 内に限定し、確定した canonical root から load する。root が利用できない場合は loader の実診断を通すが、publication には root 包含の確認を要求する。
capture 済み typed normalized path と metadata から Disk SourceInput を作り、workspace と package plan から package root を得る。filesystem 正規化と source 受理は session loader が所有する。
dispatch input hash は capture した version の SourceUnitCacheKey と一致させる。SourceLoad は output parent と dependency hash を消費しない。不足した dispatch identity は blocking のままとし planner output handle を捏造しない。
current publisher と空かつ未 seal で同一 snapshot/SourceLoad scope の diagnostic sink を要求する。不足・不整合 resource、binding/identity 不正、publication 失敗、capture 後の source 変更は output なしの Blocking。成功した load の hash 不一致は architecture 22 の obsolete dispatch result として拒否し、loader failure や新しい E0600 原因とは扱わない。一致する cancellation token は Cancelled、異なる snapshot の token は Blocking とする。
呼出元 allocator で `FrontendSourceLoader<DiskSourceLoader>::load_source_unit` を呼ぶ。loaded metadata、Disk origin、normalized text hash を照合し、disk codec で実 payload/map を capture 済み SourceId に結び直して publisher に渡す。snapshot の登録・復活や work-unit 許可は service が行わない。
publication 表現は [frontend owner](../../mizar-frontend/ja/source.md#sourceunit-publication) に従う。decoder は current request metadata、SourceId、[canonical storage comparison](../../mizar-frontend/ja/source.md#current-source-storage-comparison) 用の束縛済み SourceUnit を捕捉し、再読込・再parse なしで bytes を復号する。
publication 成功時だけ実 sealed output を持つ Complete を返す。[scheduled prefix](./driver.md#scheduled-sourceloadfrontend-prefix) は後続 service 不足でも実行できるが、full build は完了しない。

## Diagnostics and cache boundary

実 loader error は request の package/path、SourceLoad phase/category、current snapshot を持つ共有 draft を emit し、output なしの Fatal を返す。偽 range は使わない。InvalidUtf8 は E0601、UnreadableSourceFile は E0602、SourcePathOutsidePackageRoot は E0603。E0601–E0603 の detail key は割当済み semantic name とする。その他は E0600 とし、detail key は `source.` と variant 名の snake_case、未知の将来 variant は未対応の integration gap として診断を捏造せず Blocking とする。message は表示専用。draft 作成・emit 失敗は Blocking。
`cache_key` は NoKey。normalized hash だけでは current file の raw loading map と cache の一致を証明できないため、毎回実 source を load する。codec は storage 用であり cache-hit scheduling や snapshot 間再利用を保証しない。

## Tests and remaining ownership

実 temporary file と planner/index/snapshot の出力を使い、resident/blob publication、current SourceId/map、同じ normalized text の異なる raw map、capture 後の text 変更、binding/resource 不正、cancel、stale publisher、work-unit 不許可を検証する。
実 invalid UTF-8、削除・読取不可 file、対応環境での symlink escape、allocator failure を shared sink 経由で検証する。後続 service 不足の blocking と既存 registry test を維持する。
下記の実Frontendサービスがfrontend所有のpreprocess/lex/parse/recoveryと集約storageを利用する。cache compatibility、LSP、artifact と後続 semantic/proof phase は既存 owner に残す。

## Dependency lexical provider

provider は `mizar-resolve` を利用し、供給された source leaf の canonical identity と lexical payload API 用に `mizar-artifact` を本体依存とする。依存境界 lint はこれらの所有者接続だけを許可し、artifact publication 権限は driver の外に維持する。

`SourceLoadInputs::dependency_lexical_provider(artifact_roots)` は実 frontend
`LexicalSummaryProvider` の非公開実装を返す。root は呼出し側が渡す
`(PackageId, PathBuf)` の借用であり、registry の配置先を推測しない。
公開 resource に field を追加しない。呼出し側は build 検証済み module index と、
request の SourceId が表す source version からの stub を供給する。request に source hash はない。
SourceId に一致する captured source は一つで、request・index の edition と
index の package/module metadata が一致することを要求する。
resolver の provisional frontend mapper と既存 import resolver で path を解決する。
これは AST の import 検証ではない。依存 target の module/reference の artifact path・hash と
一意な明示 root を確認し、build の indexed reader と resolver の lexical summary consumer を呼ぶ。
成功時は実 summary の完全な artifact module identity と元の stub ordinal/span を保持する。
成功した `ResolvedImportEntry.import.module_id` は A15 の `ModuleLexicalSummary.module_id` と完全一致させる。
重複 import の provenance は残し、active environment の重複排除は frontend に任せる。
ファイル読込み失敗時は summary のない resolved entry を残し、既存 `MissingSummary` 回復を使う。
この診断専用 module ID は index の文字列成分に対する
`format!("{:?}:{:?}", package_id, module_path)` で、既知の package/path だけを表し lexer に渡らない。
未解決 path は既存 `UnresolvedImport` 回復に委ねる。
不正な request/index/root 対応、未対応 source-backed target、lexical payload の拒否は
部分出力なしの `ProviderUnavailable` とする。空の source summary を捏造しない。
競合回復は既存frontend、共有診断変換とIR公開は下記の実サービスが所有する。完全なsource export producer、current-build lock/cache/proof acceptanceは本providerの範囲外とする。

## Disk Frontend service

`register_frontend(artifact_roots: Vec<(PackageId, PathBuf)>)` は明示的な依存rootを所有する実Frontend専用サービスを登録する。pathを推測せず、execution-resource型を追加しない。既存SourceLoadInputsを借用し、frontend所有の [公開表現](../../mizar-frontend/ja/orchestration.md#frontendoutput-publication) を使う。後段サービス不足による全体graphの停止は維持する。
SourceLoadと同様にcurrent snapshot/workspace、一意なdisk source/version、workspace package/index/module metadataの一致を要求する。同じwork unitのcurrentな自身のSourceLoad/SourceUnit/schema-1 sealed親1個と、下記の[供給済みleaf親](#supplied-workspace-leaf-summaries)だけを許可し、publisher/storage検証後に型付きSourceUnitを復元する。identity・package/module/path/edition/hashと正準disk metadataを照合し、再読込やSourceId再割当はしない。
実行前に各indexed DependencySummaryが、そのmoduleの一意なdependency_summaries項目と、同じ `(artifact文字列, content_hash)` の一意なsnapshot artifact refに一致することを要求する。各dependency_summaries項目にも対応するindex moduleと捕捉refを要求する。欠落・同一pair重複・不一致はBlocking。同じ相対名で異なるhashは許し、無関係なsnapshot refは権限根拠にしない。呼出元はmodule-summary pathの正確な綴りを捕捉し、サービスはpackage名前空間を捏造しない。
dispatch input hashは捕捉versionのSourceUnitCacheKey、dependency hashesは捕捉module indexの全DependencySummary content hashを整列した多重集合とする。親hashはsealed bundleから得る。cache_keyはNoKeyであり、query identityはcache再利用や依存fileの可用性を保証しない。
同snapshotの空・未sealなFrontend sinkとcurrent publisherを要求し、schedulerはFrontend用sinkを作成する。一致するcancelはCancelled、不一致cancel・不正resource/binding/identity・provider/span/変換/公開失敗は出力なしBlocking。resourceを捏造せずpublisher snapshotを復活させない。
実dependency lexical providerとMizarParserSeamでrun_loadedする。診断列全体の変換後にsinkを変更する。診断ありSome ASTはRecoverable、診断ありASTなしはFatalで、どちらも出力を公開しない。診断なしASTなし・ASTがあるのに対応keyがない場合・診断を伴わないrecovery nodeはE0052を捏造せずBlocking。ASTありかつ診断・recoveryなしのみComplete出力を公開する。
変換には仕様22.2.3のfrontend所有のcode/class対応を使い、messageを分類根拠にしない。共有phaseは実coordinatorのFrontend、categoryはParseError、stable_detail_keyはdescriptorのsemantic name。構造化frontend.classはlocal class名のsnake_case、frontend.codeはtyped enum pathの各階層をsnake_caseでドット連結する（Syntaxはsyntax.と正確なkey）。これらの区別をidentityへ反映する。既存共有集約まで生成元の順序を維持する。
class対応はsource preconditionがlexical-precondition（未終端commentのみcomment-structure）、import/raw-import scanがimport-prescan、environmentがlexical-environment、scopeがscope-skeleton、raw-scan/lexerがtokenization、既知parser keyがsyntaxまたはannotation-syntaxとする。SourceLoad項目、未知code/class/parser文字列、不整合code/class組合せは変換全体を失敗させる。全主範囲・副アンカー射影を所有loaded sourceのidentity/本文に照合し、順序・長さ・UTF-8端点を検証する。共有アンカーは意図未指定で欠落なく保持し、message/生成理由原文、副位置順序・重複を維持する。recovery Noneはnoteなし、Someは空文字列を含め1個のNoteとする。保存復元の成功だけからソース結合を推測しない。
実clean/imported/失敗経路、blob往復、current親/lineage/hash分離、全mapping、未知・不正入力、アンカー/意図・空note保持、変換失敗の原子性を検査する。実際に生成する分類は実生成元、予約・pass-through分類はtyped retained fixtureを使い、後者で生成可能性を主張しない。全体buildと後段の受理は後続とする。

## 診断専用 import 継続の設計

resolver の [source-bound import producer](../../mizar-resolve/ja/imports.md#構文木からの-import-候補) が supplied-parent・診断専用・clean-importer 経路の parsed/prescan/source 対応と path 解決を担当する。driver の admission、封印出力の現行性、publication と診断変換は維持する。

限定経路は [仕様 22.3.5](../../../spec/ja/22.error_handling_and_diagnostics.md) に従う。Frontend は元の実 aggregate と診断変換、IR は sealed storage/lineage、driver は currentness・取消し・限定呼出し、resolver は AST 候補検証と型付き path 失敗を所有する。通常の成功 Frontend phase 出力や workspace summary provider とはしない。
限定経路は `run_loaded` 後の実 `FrontendOutput<SurfaceAst>` を `FrontendService::execute` 内で保持し、失敗出力の公開や後からの復号をしない。既存の SourceLoad 親・捕捉 source version/index・dispatch identity・現行 publisher を同じ loaded source に照合する。frontend 診断列全体の変換後、空でない正確な E0022/code-class 列、AST と生成元 key、全域の recovery/error node と flag の不在、`ImportPathCandidate::from_surface_ast` による完全な top-level import prelude のみを許す。候補は空でないことを要求し、実 preprocessing stub と全順序の個数・prefix/成分・alias の綴り/span・branch base/member span を照合する。直接候補の span は stub span と一致させ、branch は AST 宣言 span が prescan の member span を含むことを要求する。全 range と表現済み import token 本文（構文枠を含む）を同じ loaded source に照合する。stub は完全性の照合にだけ使い、単独で意味的 path を許可しない。欠落・余剰・綴り/span 不一致は拒む。捕捉 index の `ModuleIndexInput` を使う `ImportPathResolver` で path/alias 事実だけを解決する。graph や summary 入力を推測しない。
型付き path 失敗列全体を driver の Frontend service 内で非公開に変換する。E0220–E0224 に対応する class だけを Resolver phase draft にし、未割当 class があれば意味的 batch 全体を拒む。source 結合・range・UTF-8 境界・anchor・class/code 対応・安定 detail を全件検証してから別の Resolver sink に出す。意味的変換の失敗はその batch だけを捨て、検証済み Frontend batch は残す。呼出し時の取消しを処理前に確認し、その token は呼出し中は不変とする。解析前と返却前に publisher/親の現行性を再確認し、後段の scheduler 取消しと lane guard は cancelled/superseded 報告を抑える。出力 ref が空の `Recoverable` とし、既存 `PhaseResult.diagnostics` に Frontend batch と任意の Resolver batch を載せる。registry は従来どおり query observation を付けるが、`NoKey` は cache 再利用や成功 credit を与えない。scheduler は task を失敗にし、通常の後続を skip する。新しい出力型・registry resource・失敗出力 handle・Recoverable から Complete への変更は不要。
実 Frontend service はこの限定経路でソース結合済み E0220–E0224 変換を行う。[scheduled prefix](./driver.md#scheduled-sourceloadfrontend-prefix) により `CompilerDriver::submit` からこの経路を実行できるが、session の失敗と通常の後続の遮断を維持する。E0225 と完全な workspace summary は後続とする。

import draft の detail key は `import.source_module`（String list `[package, path]`）、`import.ordinal`（Integer）、`import.path`（resolver spelling String）、`import.branch_member`（branch のみ Source range）とする。E0223 は `import.alias`（String）と `import.target`（String list `[package, path]`）を追加する。stable detail key は registry semantic name、phase は Resolver、category は ResolveError。primary/secondary range は仕様22.3.5に従い anchor intent は未指定とし、この schema で集約時の candidate identity を保持する。

## Supplied workspace leaf summaries

Frontend は自身の SourceLoad/SourceUnit 親1件に加え、捕捉済み workspace package 内の import を持たない workspace leaf または[後述の中間モジュール](#workspace-intermediate-summaries)の current Frontend/FrontendOutput 親を受け取れる。bundle の位置でなく phase/kind/work-unit identity で選び、重複・無関係・未使用 workspace 親 を拒否する。既存 artifact summary 経路と公開 provider constructor は維持し、service が検証済み workspace 出力を渡さない場合の source-summary slice は空とする。
各 leaf 自身の一意な workspace PackagePlan と PackageIndexEntry を使い、version/edition と安全な root/source-root/manifest path を source entry に結び付ける。importer の metadata で target の metadata を代用しない。各 leaf は一意な捕捉 disk SourceVersion と workspace index/package（source id/hash、package/module、path、edition）に一致し、正しい canonical source metadata と current schema-1 sealed Frontend payload を持つ。登録 lineage は同じ snapshot と leaf work unit の SourceLoad/SourceUnit 親1件でなければならず、親が保持する source input key と leaf の source cache key は捕捉 SourceUnitCacheKey に一致しなければならない。disk を再読込せず埋込 source を検証する。AST/key があり、診断・AST 全体の recovery/error node・parsed/prescan import・reexport がないことを要求する。AST から symbol を収集し、完全な A13 source 対応と対応済み lexical export を検証する。未対応 operator/alias は拒否する。build の package/module/edition identity と package_version Some(一意な workspace PackagePlan.version.to_string()) を使い、その target の PackageIndexEntry.version の一致を要求する（canonical な捕捉 lockfile identity API がないため lockfile_identity は None）と既存 lexical contribution export、canonical lexer constructor を使い、完全な artifact ModuleSummary は合成しない。
共有する source/import 照合は診断専用継続の exact E0022/class・非空 candidate 条件と元の診断 batch を維持する。既存の private provider は一致する捕捉済み workspace package の target にだけこの summary を使う。成功 publication 前に importer の parsed import を preprocessing/source と照合し、全 workspace 親が実際の解決済み workspace target であることを確認する。不足・余分な workspace 親は停止する。消費した全 sealed parent を importer lineage に含め、診断返却・publication 前にも全親の現行性を再確認する。dependency hash は従来どおり捕捉 dependency artifact のみを表す。これは暫定的な import 字句処理であり、意味的 export/import の受理ではない。[scheduled prefix](driver.md#scheduled-sourceloadfrontend-prefix) が呼出し元の complete import overlay に従って workspace 親を渡す。overlay 自動発見、reexport、cycle、E0225 は後続とする。

## Workspace intermediate summaries

供給する clean な workspace Frontend は、他の捕捉済み workspace module または捕捉した DependencySummary target を import していてよいが、reexport は認めない。leaf の source/index/codec と A13 local export の検証は維持する。parsed/prescan import を照合し、全 target が未解決なく解決されることを要求し、self-import は拒否する。artifact target は捕捉した named input であり Frontend 親ではない。直接の登録済み lineage は、結び付けた自身の SourceLoad/SourceUnit 親1件と、異なる workspace target ごとの Frontend/FrontendOutput 親1件を含み、snapshot/work-unit が一致し、不足・余分な親がないことを要求する。import がなければ従来の leaf 経路となる。
named-input 全体を IR の name/domain/digest 正規順序で比較する。active-lexical-environment と tokens は供給 payload の cache key、dependency.ordinal は無関係な summary も含む捕捉 index summary hash 全体のソート済み列に一致させる。SourceLoad の named source-key 結合も維持する。祖先 payload の消費時検証は current な sealed 中間 publication に依存し、直接の登録 metadata から祖先 payload の再検証や独立した推移的 graph 検証を主張しない。A13 の直接 local public contribution だけを export し、import 記号を暗黙に summary へ含めない。中間モジュールの artifact payload は既存の artifact provider/reader が生成時に検証する。consumer は current な sealed publication と捕捉した結合に依存し、artifact 再読込や新たな lockfile 認証は主張しない。

Frontend blob decoder は既に束縛済みの Disk input で共有 frontend storage
復元を使う。Disk 限定の実行と publication の規則を維持する。

実 Native 取得と source 束縛が供給されるまで親の current/package 承認を維持する。
OpenBuffer/InternalOnly の来歴だけでは承認しない。

## Native captured sources

既存の登録 service と discovery は[request](request.md#native-source-capture)の実 Native 捕捉も消費する。Disk 節は従来の summary-only 経路を示し、その Open 結果拒否は維持する。任意借用 CapturedSnapshot は保護された loader/LoadedSource/SourceInput triple を供給し、caller 作成 payload 添付ではない。
discovery/SourceLoad/Frontend は保護 snapshot 全体、一意な実 metadata/ID/hash/origin/raw URI/version/map、保持 loader package root と既存 plan/index/workspace/source-root guard に束縛する。canonical ID 一致だけでは不十分。Generated と実捕捉のない Open summary は拒否する。 全 native consumer で指定 BuildPlan.toolchain と保護 snapshot toolchain の一致を要求し、不一致は既存 no-source/Blocking または no-overlay 結果とする。opaque identity 一致は released/running 版・制約適合・真正性を示さない。no-capture の従来経路・cache 準備 error・実 NoKey は不変。
既存 frontend 表示 path 方針で捕捉 Disk/Open を投影し、text 再読込や代替 source ID 取得は行わない。従来 Disk の load/drift/error 診断は不変。Native 取得失敗は型付き capture error であり、scheduled 診断を捏造しない。
SourceLoad/Frontend は owner の Disk/Open codec を使い、own/供給済み親の source payload 全体と raw loading map を実捕捉へ照合する。provider、lexical handoff/import 対応、診断変換、cancel と publication guard は維持する。discovery は出力/診断を公開せず部分推定 overlay を拒否する。
Open は OpenBuffer/InternalOnly、捕捉 Disk は PackageSource/CurrentPackage。実 Native dispatch の四 consumer だけが記録済み published 承認を使い、summary-only の current/package 検証は維持する。実 cache intent はともに NoKey。後続 service、LSP current 診断、proof/cache/artifact 権限と全 build 完了は未対応。

## Native lock content association

`SourceLoadInputs::validate_native_lock_content(self) -> Result<(), String>` は保護された native capture と供給 snapshot 全体の一致、plan/snapshot toolchain の一致を明示的に要求し、snapshot.lockfile_hash を既存 Build [lockfile_content_hash](../../mizar-build/ja/planner.md#lock-content-fingerprint) と比較する。既存 content profile を用い、capture 不足・非 native・foreign・不一致を拒否する。file read と mutation は行わない。caller 限定の前提であり、untagged Hash から profile を選ばず、generic native/cache/submit 経路を変更しない。file freshness、pin/config/toolchain 真正性、retention、scheduler/host 権限は与えない。

## Native Disk cache preparation

`SourceLoadInputs::prepare_native_disk_cache_request(self, &PhaseInput, &VerifierPolicy) -> Result<CacheKeyRequest, String>` は保護された native Disk capture から Cache owner の通常 request data を準備する。同一 snapshot、一意の束縛 module/version、実 SourceUnitCacheKey dispatch hash、parent/dependency hash なし、plan/snapshot toolchain 一致を要求する。既存 native_source guard を使い、root canonicalization は純粋 registry query の外の filesystem metadata I/O として明記し、source body は再取得しない。capture 不足・曖昧・foreign、Open/Generated、空 policy profile、POLICY_SCHEMA_VERSION 以外の policy schema、DEFAULT_CHECKER_SCHEMA_VERSION 以外の checker schema は拒否する。
本体 mizar-cache/mizar-proof 依存は既存 metadata API と後述の Cache owner の Store lookup を消費する。明示 policy の policy_fingerprint().hash() をコピーし、config identity と分離する。互換性値は供給された識別を運ぶだけで、active-run 選択・現 lock file 認証・toolchain 互換性を証明しない。
| Field | Exact projection |
|---|---|
| phase/work_unit/source_identity | SourceLoad、既存 IR package/module 表記、実 capture の package/module/path/hash/edition。allocator ID・診断 path を含めない |
| input_hashes | source / SOURCE_UNIT_CACHE_KEY_VERSION / 実 source-key digest; source-storage / mizar-frontend/source-unit-storage-side-table/v1 / 実 canonical Disk storage derive-key digest; captured-lockfile / mizar-driver/captured-lockfile-identity/v1 / 実 snapshot.lockfile_hash |
| config_hash / policy_fingerprint | 実 snapshot.verifier_config_hash / Cache PolicyFingerprint::new(policy.policy_fingerprint().hash()) |
| schema_versions | phase-output/SourceUnit = 実 SourceLoad descriptor schema; source-key/SourceUnit = SOURCE_UNIT_CACHE_KEY_VERSION; source-storage/Disk = mizar-frontend-source-disk-v1 |
| cache schema / compatibility | key schema と cache_schema_compatibility は共に CACHE_KEY_SCHEMA_VERSION |
| toolchain_compatibility | family mizar-session/toolchain、field identity、実 snapshot.toolchain.identity() |
| policy_compatibility | family mizar-proof/verifier-policy: schema_version・profile_id・checker_schema_version は実 public getter、fingerprint は実 policy fingerprint lower hex |
| footprint / optional inputs | 上の source-only 検査後 Complete、uncacheable=false。消費 dependency/slice/artifact・VC/context/witness/reuse・diagnostic input はなし |
captured-lockfile domain は保持 digest の派生 identity-carrier namespace であり、上流 digest algorithm や現 lock producer を主張しない。既存 source/storage owner の digest と bytes は保つ。Complete はこの SourceLoad の入力を全て束縛した footprint のみを表し、同 phase は output parent・lexical provider・dependency artifact を読まない。
lease/currentness・active policy 対応・互換性検証・lookup・保持 output・scheduling は caller/runtime と Cache が所有する。request data/key 構築は hit/publication/proof receipt ではない。両登録 cache intent は NoKey のままとし、summary-only 動作・Open/editor uncacheability・Frontend key producer と Steps 6–8 全体は保留する。

## Native Disk record encoding

`SourceLoadInputs::encode_native_disk_cache_record(self, &PhaseInput, &VerifierPolicy, &IrCacheAdapter, PhaseOutputRef<SourceUnit>) -> Result<EncodeCacheRecordOutcome, String>` は実封印済み SourceLoad output と同じ保護 native Disk input を消費する。上の request 準備と既存 CacheKeyBuilder を呼び、Cacheable 結果だけを encoding へ渡す。既存 root metadata I/O は純粋 cache query の外に置き、source body は再取得しない。
adapter の publisher/storage を使う。input snapshot に対する physical handle 有効性と既存 `validate_current_output` 承認を要求し、snapshot 登録や work 許可は行わない。SourceLoad phase、request と同じ IR work-unit 表記、SourceUnit kind、IR schema 1、lineage parent なし、named input は `source / SOURCE_UNIT_CACHE_KEY_VERSION / input.identities().input_hash()` 一件だけを要求する。
storage.get で実 typed payload を取得する。その SourceId が一意の native captured version と等しく、既存比較による canonical storage が取得済み SourceUnit と一致することを要求する。side table は source map 一件 `source-storage / normalized source path / request source-storage digest` と完全一致し、diagnostics・explanation refs・documentation attachments は空とする。
既存 IR encoder に実封印 handle、構築 key、実 request toolchain_compatibility を produced_by として、実 source-key digest の semantic bytes、束縛 typed payload の canonical Disk bytes、この全入力を検査済み zero-dependency SourceLoad footprint の Cacheable 分類を渡す。IR owner が semantic/storage/side-table hash と physical lineage を検証し、hash/codec/schema 実装を重複しない。
既存 encoding outcome を返す。binding/key/storage error は record なしの Err とし、caller は失敗を optimization skip とする。store insert・lookup・rehydration・新規 publication・scheduler decision は行わない。encoding は collection 越しに handle を保持せず、active-run policy/lease/fresh-lock receipt も作らない。caller/runtime がその対応と並行 retention/currentness を保ち、policy 選択と Cache 互換性は既存 owner に残る。
試験は実 scheduled SourceLoad resident/blob output、その実 side table と保護 capture を使う。既存 CacheStore insert/lookup はこの encoder の record だけを検証し、record transport を調べる。scheduler cache-hit equivalence の達成ではない。replay/local-ID relocation、normalized-equal で異なる raw map、foreign/collected/obsolete または誤束縛 output は fail closed を維持する。両登録 NoKey 理由、Open/internal publication、後続 producer/rehydration/retained-scheduling の保留は不変。

## Native Disk current lookup

`SourceLoadInputs::rehydrate_native_disk_cache(self, &PhaseInput, &VerifierPolicy, &CacheStoreRoot, &IrCacheAdapter) -> Result<CacheRehydrateOutcome<SourceUnit>, String>` は上の実 native Disk request と Cacheable builder key だけを準備する。input snapshot に対する供給 publisher の既存 validate_current_snapshot を要求し、snapshot 登録・復活・work 許可は行わない。同じ一意 captured version と private native_source binding を使い、純粋 cache query 外の既存 root metadata I/O を保ち、source body は再取得しない。
その完全な current key で Store.lookup を呼ぶ。実 Hit の header.produced_by が current request の完全な toolchain_compatibility projection と等しいことを要求し、不一致は IR を呼ばず binding Err とする。これは producer の完全な対応確認であり、新しい semantic compatibility registry ではない。全 non-Hit は IR adapter を通して既存 miss を保つ。
IR 期待値は実束縛 source-key digest の semantic bytes と完全な side table とする。normalized path の source-storage map 一件に request の実 storage digest を使い、他 vector は空。SourceLoad/current request work-unit/SourceUnit/IR schema 1、parent なし、IR named input は source/SOURCE_UNIT_CACHE_KEY_VERSION/実 source key 一件とする。三つの Cache request input を IR lineage へ写さない。
所有 BlobDecoder は current captured SourceId と実 SourceInput で from_canonical_storage_bytes を使い、取得済み native SourceUnit に対して matches_canonical_storage を行う。rehydrate_with_expected_payload に渡し、semantic/side 期待値と元の hash/lineage/decoder 検証、既存 current/package publication 許可を、返す handle より前に実行する。公開後に不一致 handle を捨てる方法は使わない。
binding/request/key/current-snapshot/producer error は current handle なしの Err とし、caller は失敗を optimization miss とする。他は既存 IR outcome を返す。cache data は読むが Store record は書かず、scheduler decision・retain owner・active policy/lease/lane/fresh-lock receipt・proof acceptance を作らない。configured cache root、runtime 対応、並行 lifetime は caller が所有し、publisher は publication 時も許可を確認する。
三つの異なる実 consumer は束縛 encoder の record と実 lookup を使い、current resident/blob/Frontend の clean output/map、実 input/context/policy 変更の miss、current/producer binding 拒否を調べる。負の Store 試験では実 record の metadata を敵対的に変更してもよいが、実 producer 主張や架空 Hit の根拠にはしない。両登録 NoKey 理由と Open/internal 規則は不変。後続 runtime/retained-scheduler/Frontend-key/semantic/artifact 統合は保留。
