# 複数直接 leaf の native Disk Frontend キャッシュ提案
Canonical language: English; [英語正本](../en/multiple_direct_leaf_frontend_caching.md)。

## 状態と権威

**Q1–Q5 は owner が全て option A を承認済み（2026-11-17）。** [Driver 計画](00.crate_plan.md#task-index) の本提案は[承認済み一 leaf 案の Q1-B](dependency_aware_frontend_caching.md#owner-q-decisions) が要求した複数 target の前提。実装には別の実装契約 freeze が必要。以下の代替案は判断の文脈として残し、B は未承認。
権威は spec の [字句前処理](../../../spec/en/02.lexical_structure.md#210-lexical-preprocessing)、[import conflict](../../../spec/en/11.symbol_management.md#1124-precedence-rules)、[import](../../../spec/en/12.modules_and_namespaces.md#123-import-statements)、[再現性](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility)、architecture 22 の [完全 footprint](../../architecture/en/22.incremental_verification_contract.md#dependency-slice-completeness)、[lexical/parse 入力](../../architecture/en/22.incremental_verification_contract.md#frontend-lexicalparse-footprints)、[clean 等価性](../../architecture/en/22.incremental_verification_contract.md#clean-build-equivalence)、[freshness](../../architecture/en/22.incremental_verification_contract.md#snapshot-scoped-results)、[fallback](../../architecture/en/22.incremental_verification_contract.md#cache-miss-fallback)、Cache の [key](../../mizar-cache/en/dependency_fingerprint.md#fingerprint-targets)/[Store](../../mizar-cache/en/cache_store.md#lookup)、IR の [期待値](../../mizar-ir/en/cache_adapter.md#producer-payload-expectations)。言語、conflict 解決、診断、acceptance、compatibility の新規則は提案しない。

## 既存 coverage と定義 gap

[Supplied summary 認証](frontend_adapter.md#supplied-workspace-leaf-summaries) は通常実行で複数 physical Frontend parent を訪問し target 対応 summary を返す。一方 cache の `registry.rs::native_frontend_cache_parents` は一 leaf まで、`frontend_adapter/source.rs::prepare_native_disk_frontend_cache_payload` は summary/leaf source 一つを要求し、全 resolved import がその target か検査する。private `mizar-driver/native-disk-frontend-dependencies/v1` は既に import 順の列、target 順 summary 列、phase/work-unit 順 physical parent 列を持つ。列の存在だけでは複数 target の安全性は証明されず、target/summary/Frontend/SourceLoad の厳密な対応と全要素の lifecycle/test 定義が不足する。
Driver は capture/runtime/provider 対応と orchestration、Build は graph/index/config/lock/readiness、Frontend は前処理/字句環境/key/parse/storage、Resolver/Artifact は resolution/A13 export/summary identity、Cache は canonical key/compatibility/Store、IR は再構築/physical lineage/retention、Proof は実 policy/acceptance/reuse を所有する。既存 owner/private 経路を使い、新 public context、lower-owner seam、並行 summary 表現は承認しない。

## 提案する認証済み closure

Q1-A は既存 zero-/one-leaf 経路を有限個の distinct direct captured workspace target に拡張し、各 target は clean AST-bearing import-free Disk leaf で reexport を持たず、importer も clean AST-bearing Disk で reexport を持たない。深さ一の closure で transitive traversal ではない。真正に認証した complete-empty public export は有効。不足/非対応 contribution、intermediate/transitive/artifact、self/cycle、reexport、Open/Generated、recovery/診断では最適化を見送る。同名 conflict と qualification/alias は既存 owner semantics に従い、診断を cache success に変えない。
認証した importer prescan/parsed import/実 resolution から distinct target を導き、target、実供給 immutable summary、受理済み leaf Frontend、captured target-owned source/metadata の全単射を要求する。importer の physical parent は自身の SourceLoad と distinct target ごとの Frontend 一つのみ。各 leaf Frontend の lineage は自身の current SourceLoad 一つのみ。各 target の native capture、snapshot/plan/index/task、package version/edition/root/path/source key/map、AST、A13 local export、summary identity を既存 zero-import leaf 検証で確認する。不足/余剰/重複/入替え/stale/foreign、正しい summary と別 leaf source の組合せも拒否する。同じ content/fingerprint でも target を統合せず identity の認証根拠にしない。source summary の `lockfile_identity: None` を保ち、capture の lock/config は release authenticity を与えない。
同一 target の重複 import は physical leaf parent を共有するが、ordinal/span/alias/branch provenance の全順序付き項目を保持する。字句 spelling/fingerprint で重複排除しない。実 parser/provider は期待値と実行で同じ検証済み summary 集合を使う。descriptor コピー/caller context/一致 hash は選択を認証しない。実 bootstrap descriptor/provider 対応と全参加 code の executing-image residency を保持し、不明なら caching を拒否する。

## Projection coverage と独立期待値

Q2-A は複数 target の全項目を真正 owner 値へ対応付けてから、既存完全 key bundle/private projection を再利用する。保持すべき既存 coverage:

| 必要入力 | 既存 owner coverage と複数 leaf の義務 |
|---|---|
| Lexical/parse 状態 | `FrontendCacheKeys::canonical_bytes`: source/edition、preprocess identity、active environment、token lexical/context/filter/位置依存 plan、AST parser schema/version/edition と parser-input hash。notation/synonym/antonym、predicate/functor metadata、位置依存 operator 効果を保持し、不明な coverage を拒否。 |
| 実前処理/token/import | `PreprocessedSource::canonical_bytes` と `TokenStream::canonical_bytes` が前処理/prescan/実 token/local declaration を commit。全 target の resolved 項目は target package/module、ordinal、source/alias/branch range を元 ordinal 順で保持。token-key hash は token 内容ではない。 |
| 全 summary/physical parent | package/module 順 summary は完全 canonical module identity、lexical fingerprint、owner 順 `ExportedSymbolShape::canonical_bytes` を保持。phase/work-unit 順 parent は own SourceLoad、全 leaf Frontend、全 leaf SourceLoad の content/side commitment。target ごとの zero-import 検証で source/key と summary/physical pair を encoding 前に結ぶ。 |
| その他 native context | 既存 source semantic/raw-storage/map、source-parent、完全 key-bundle、config/lock、実 policy/runtime/schema、canonical named-input を保持し、Cache 順序/compatibility 検証を維持。 |

Import provenance 順序を canonical target/parent 順序と分け、physical 重複は sort 前に拒否する。SourceId/session/lane/task/worker/completion/capture 順、retention、cache-root routing は semantic key に含めない。semantic proof DependencySlice、集約 fingerprint、zero 値で不足字句情報を代用しない。後続 freeze は厳密な field/codec/order/key 配置と Q2-A で zero-/one-leaf bytes が不変であることを示す。未対応入力や必要な schema/compatibility 変更は owner 判断へ戻す。
Q3-A は current bound SourceUnit と完全に同じ認証済み immutable summary 集合で真正な全 Frontend 解析を lookup 前に行い、Store/retained importer から独立させる。完全 owner key、semantic bytes、raw Disk storage/map/side table、named input/parent 期待値を生成。decode 前の全 raw storage 比較、IR semantic/side、Cache key/produced-by 検証、current SourceId rebinding/current lineage 再構築を保つ。cached/retained leaf は既存 current physical/owner 認証後のみ依存を供給できる。miss では再解析も許す。CacheHit と importer TaskStarted 不在は scheduler skip の証拠に限り、parsing-avoidance/full STEP6C credit を与えない。

## Readiness、retention、永続化

Q4-A は実 importer の [dependency-ready callback](../en/driver.md#dependency-ready-native-disk-frontend-binding) を再利用し、PackageResolve、自身の SourceLoad、必要な全 leaf Frontend の受理後に行う。親は実出力を持つ Completed/CacheHit のいずれでもよい。厳密な graph task/result/overlay 対応を結び、到着順/Store/cached string を readiness にしない。leaf の先行実行や合成 completion はせず、scheduler が受理した hit のみ importer 出力を渡す。failed/cancelled/unavailable parent は通常 scheduling に従い、部分 closure を準備しない。
Own SourceLoad、全 leaf Frontend と各 SourceLoad を消費前に scoped IR owner で保持し、全期待値/lookup/decode/scheduler 消費/最終検証まで維持する。再構築 importer も保持する。途中 acquisition failure は取得済み全 owner を解放する。lower work/scheduler handoff 前後で physical validity、lane/request/snapshot/cancellation/supersession/publisher を再検査する。success/miss/refusal/decode・Store failure/scheduler error/cancellation/supersession の全経路で解放する。reader/snapshot lease は current-use/publication 権を与えず、Store I/O を跨ぐ atomic lifetime や capture 後 disk freshness を約束しない。
Q5-A は同一完全 closure で dependency-ready lookup と、真正 clean importer/受理済み依存結果からの guarded PersistentPrefix write を組にする。全 parent/summary 対応と完全出力期待値を insert 前に検証する。一 leaf が不適格なら importer caching 全体を見送り、leaf を落として続けない。独立して適格な leaf write は残せる。guard loss は以後の write を抑え、以前の有効 write は rollback 不要。best-effort Store、package routing、invalid input/実 producer error 優先順位、pure registered NoKey、既存 direct/retained/zero-/one-leaf を保持する。不適格 context は最適化だけを見送り、新診断や semantic success を捏造しない。

## Owner Q-decisions

**Q1 — 個数。** A: duplicate import provenance を保持した有限の厳密な depth-one direct import-free workspace leaf 集合全て。B: distinct target 二つのみ先行し、それ以上は再定義を要求。**推奨 A:** 同じ target ごとの全単射と全親検証を適用でき、恣意的な個数 policy を加えない。transitive/intermediate/artifact admission や resource bound は与えない。
**Q2 — projection。** A: mapping が完全で zero-/one-leaf bytes が不変なら既存 private v1 list encoding/完全 key を保持。B: 複数 target admission 前に新 version の private projection を定義。**条件付き推奨 A:** 既存 list で全項目を表せるが、schema/compatibility 変更を仮定で承認しない。不足 coverage/encoding 曖昧性は実装前に owner 判断が必要で、部分 key は禁止。
**Q3 — 期待値。** A: 認証済み完全 summary 集合を使う全 importer 解析を lookup 前に実行。B: parsing を避ける独立期待値 producer を先に定義。**推奨 A:** 新 Frontend 権威を作らず承認済み oracle を拡張。B は別 owner seam/footprint 案と parsing-avoidance 基準が必要。
**Q4 — timing/lifetime。** A: 既存 importer-ready callback と scheduler 消費までの全 physical 依存 scoped ownership。B: 新 staging/retention 規則を持つ別 all-leaf 準備 stage。**推奨 A:** Build graph 実行を維持し投機的 producer を避ける。どちらも currentness/厳密な受理結果対応を迂回しない。
**Q5 — rollout。** A: guarded multi-leaf write/read を同時導入。B: read-only admission を先行、真正 write は後続。**推奨 A:** 自動 cold/warm 根拠と全 closure fallback を観測できる。B は positive lookup credit 前に別の真正 record producer が必要。Store compatibility、zero-/one-leaf、proof/publication policy は変えない。

## 条件付き検証と引継ぎ

Owner 承認後の実装 freeze は最大六 case family を定義:
1. Q1-A では distinct target 二つと三つの実行で真正 leaf/識別可能な export/片方の重複 import/真正 complete-empty export を用いる（Q1-B では三つ目 target の fallback を要求）。自動 cold write、warm/fresh capture、resident/blob/current SourceId rebinding を独立 clean の全 key/bytes/map/side/parent commitment/診断/outcome と比較。独立 workspace/capture registry/空 Store と逆 worker/completion/capture 条件を使い、期待値を検査対象 cache から得ない。
2. Importer bytes を固定し各 leaf を独立編集、importer token 内容で直接観測する字句変更も含める。target 追加/削除/置換、duplicate/alias/branch provenance 変更で viable 入力は invalidate/refuse。不変 local ID/scheduling 置換では canonical identity を維持。既存 notation/operator/parser/edition/config/lock/policy/runtime control を再利用し、複数対応が新義務を生む箇所を拡張。
3. 他条件が viable な実 record から不足/余剰/重複/入替え/foreign/stale/collected Frontend・SourceLoad pair、target/summary/provider 不一致、hash も整合させた payload corruption を独立攻撃。別 target の export fingerprint が等しくても unchecked publication 前に拒否。
4. Leaf を決定的に遅延し受理済み Completed/CacheHit を混在、親一つを fail/cancel、準備中に cancel/supersede/publisher 交換。早すぎる lookup がなく、全親を scheduler 消費まで保持し、部分 acquisition cleanup と全終端経路解放を確認。
5. 他条件が viable な multi-leaf closure で一 leaf のみ cache 不適格なら、importer hit/write がなく独立 clean 通常実行と等しく、その leaf を落とさないことを確認。同名 multi-leaf conflict は通常 error、qualification/alias 解決は通常 outcome を保持し、他条件が適格な時だけ caching。非対応/不完全/transitive/intermediate/artifact/cyclic/診断、disabled/failing Store は viable な通常実行と実 error 優先順位を保持。
6. 既存 zero-/one-leaf cold/warm、direct/retained/persistent/NoKey、canonical compatibility と later-service/proof/publication/reuse deferral は不変。広い cache/full STEP6C 完了を主張しない。
Driver `source_load/native_disk.rs` の imported cold/warm と独立 edit/clean、`supplied.rs`/`misc.rs` の親引継ぎ、`frontend.rs` の実 conflict、private source-adapter refusal、影響する Build/Frontend/Resolver/Cache/IR owner 群を再利用。positive 根拠は実 bootstrap/真正 producer/scheduler event/Store record。後続 freeze で厳密な write path/API/encoding/case outcome/focused check/短い E2E を指定し、変更 cache/lifecycle 義務はその時実行、全 regression は cadence trigger のみ。新 language/semantic/compatibility 判断は owner に戻す。本定義では code/test/spec/coverage/audit status を変えない。
