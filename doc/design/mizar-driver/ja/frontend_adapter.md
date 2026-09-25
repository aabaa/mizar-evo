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
publication 表現は [frontend owner](../../mizar-frontend/ja/source.md#sourceunit-publication) に従う。decoder は current request metadata、SourceId、expected source hash だけを捕捉し、bytes を復号する。完成 payload の保持や再読込・再parse はしない。
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
