# Active-run context producer と native Disk cache binding 案
canonical English: [English companion](../en/active_run_context_cache_binding.md)。

## 状態と根拠

**owner は Q1–Q6 を A として承認済み。** [Driver plan](00.crate_plan.md#task-index) が索引する歴史的定義案であり、inventory は実装前の境界を記す。現在の native Disk SourceLoad behavior は [active-run binding](driver.md#active-run-sourceload-binding) が所有する。現在の限定 cold Frontend behavior は [cold Frontend binding](driver.md#active-run-cold-frontend-binding) が所有し、full STEP6C は別途とする。
根拠は英語正本がリンクする仕様 §23.4、architecture 22 の equivalence、fallback、freshness、complete footprint。全選択肢で既存言語・proof behavior を保持する。

## 既存 producer と欠けている対応付け

[Native capture](request.md#native-source-capture) は実 source/loader/input tuple と snapshot identity を保持する。[Build planning](../../mizar-build/ja/planner.md) は typed package configuration、lock content、module/dependency index を供給する。明示 [config/lock check](frontend_adapter.md#native-config-content-association) は content hash と capture の対応であり active runtime selection の認証ではない。記録 toolchain version/constraint や opaque ToolchainInfo の一致は実行 compiler、選択 backend、resolved pin を特定しない。
[Native Disk preparation](frontend_adapter.md#native-disk-cache-preparation) は SourceLoadInputs と明示 VerifierPolicy を受け、compatibility field は supplied identity を運ぶ。NativeCacheOptions の policy/Store/mode は configured service に保持される policy から独立しており、この run の実効設定・policy との対応 producer はない。genuine SourceLoad persistence、leaf persistence、retained Frontend lookup は既存の限定明示経路であり、完全な runtime provenance や cold Frontend expectation ではない。
本案の producer/binding は `design_gap`。genuine running-toolchain/component identity と active configuration/policy association producer は `external_dependency_gap` で、cold Frontend expectation、後段 dependency/proof producer も未存在。承認後も source/test 作業は別契約を要する。caller 作成 digest、descriptor、成功 flag、generic published handle では埋めない。

## 提案経路と所有権

Q1–Q6 を前提として Driver bootstrap が実 protected capture、produced plan/index、選択 registry/service、実効 invocation input から submission-local association を組み立て、native cache に該当する検証済みデータのみ渡す。下位 semantic state を再構成せず transferable receipt としない。表現、public type、constructor、hash/schema 拡張はここでは freeze しない。
Build は config/lock projection、plan validation、graph、cache scheduling、Proof は policy 解釈/fingerprint・selection/status・exported reuse metadata、Kernel は evidence check、Cache は key・compatibility・proof-reuse/footprint/Store 検証、Frontend は独立 expectation/phase footprint、IR は sealing・physical retention・publication permission を所有する。Driver は同一 run の対応と live session eligibility を確認し、下位判断を再定義せず profile を拡張しない。

| 必須の対応 | genuine source と拒否境界 |
|---|---|
| Capture/task | 元 native Disk source、package/module/path/edition/raw-map/source-key と実 snapshot/task/work-unit/plan/index/sealed parent の一意で厳密な対応。local ID や normalized hash の一致だけでは足りない。 |
| Config/lock | bootstrap が実際に選んだ package verifier/build setting と typed lock input。既存 content profile/capture 比較と package 別 Store-root 選択を保持。supplied aggregate/hash 一致は current-file/resolved-dependency provenance ではない。 |
| 実効 policy/invocation | 各 configured producer と cache 検証に実際に渡す Proof policy、該当する実行設定由来の resolved backend profile/priority、evidence priority、記録 seed/resource budget。policy/config identity を分離し、active selection 不明や対応矛盾では最適化しない。 |
| Runtime/schema | 実選択 service implementation と登録/実 descriptor、owner output/key/storage/checker schema、実行 compiler/component と該当 selected backend の genuine identity。TypeId は process 内 service の識別だけであり copied descriptor/version/env claim/opaque ToolchainInfo は portable identity/compatibility ではない。Q2 の producer が必要。 |
| Dependency/expectation | 実 artifact/index/summary identity、owner-complete phase footprint、IR が必要とする独立 current payload byte/side table。SourceLoad の完全な zero-artifact footprint は Frontend/import/proof の zero-dependency 根拠ではなく、unknown は empty ではない。 |

最低要件は admitted phase の該当 identity closure 全体であり全 future service の field ではない。compatibility 判断は Cache に残し runtime version/schema の同値性を新設しない。session/request/lane/generation、cancel、publisher、retention は利用認可であり canonical key に含めない。[Key purity](registry.md#cache-key-purity-contract) に従い worker/order/timing、未宣言 environment/filesystem metadata、cache-root routing を semantic key input としない。宣言 seed/budget/config identity は owner の規則を保持し acquisition/root I/O は pure query 外で行う。

## 寿命・currentness・拒否

invocation 中は元の immutable owner value を使う。別 capture/plan/registry/policy selection の context を hash/snapshot id 一致だけで転用しない。frozen capture から disk-file freshness を推論せず実 source load/dispatch 検証を保持。設定/registry/policy selection 変更は retained association の書換えでなく新 association を要する。
[Lane/session と request-snapshot guard](request.md#current-session-publication-boundary)、cancel、publisher authorization、physical validity を、lookup/materialization/persistence/handoff/final reporting の既存 checkpoint で保持し、既存経路同様 lower work 後に再検査。snapshot lease は snapshot resource、IR owner は slot を保持するだけで currentness/publication 権限ではない。cached parent は実 scheduler consumer まで scoped retain し、success/refusal/error/cancel/supersession の全 return で解放。retained read は実行・公開権限を復活させない。
Store I/O と lane/publisher 変更をまたぐ atomic lifetime は保証せず、先行する valid complete record の rollback を要求しない。将来 reuse は新 run に通常 Cache 検証を行う。publication/currentness suppression と corruption/semantic rejection を分離する。
context 欠落・非対応・曖昧・不一致は Q6 に従い cache preparation を控える。Store/lookup/decode/retention failure は既存 miss/error-as-miss、invalid submit/dispatch/graph input と実 producer error は既存順序・結果を保持。policy 弱化、入力修復、proof reroute、metadata 由来 hit を認めない。validated skip を受理するのは Build で、実 CacheHit task のみ prepared output を公開する。context/hit は acceptance、complete proof slice、witness/Artifact publication、verified session ではない。

## Owner Q-decisions

**Q1 — producer/binding 境界。** A: 実 Driver bootstrap が実際に install/use する value から private association を作り native cache が同じ association を使う。legacy API の限定 claim は維持。B: host 発行 reusable context を後の submission でも受ける。新権限と cross-run 検証を最小化する **A を推奨**。B は issuer 認証、厳密 capture/registry/config binding、revocation、consumer 規則の freeze が必要。caller 作成 context を genuine producer としない。

**Q2 — 実行 producer identity。** A: 実 construction/selection site で実行 compiled component と selected service/backend implementation の owner-produced deterministic identity を供給し、release/pin 認証は別とする。B: genuine distribution/toolchain-selection attestation を必須とする。lock metadata を release 認証と誤認せず実行を特定できる **限定 native 実装の A を推奨**。producer は未存在で、authoritative build/selection source、再現可能な versioned encoding、Cache 所有 exact compatibility rule を前提契約で freeze する。process-local TypeId/caller label/自己申告 digest では不足。B は外部 selection/attestation owner も必要。unknown provenance は双方とも unavailable で、新 compatibility policy をここで承認しない。

**Q3 — 実効設定/policy 対応。** A: bootstrap で実効 input を一度 resolve し、同じ Proof-owned policy value を configured service/cache 検証へ渡し phase 別 role を明示。B: 独立構築のまま owner 提供の完全な対応検証を admission 時に追加。事後的 equality を selection 証明としない **A を推奨**。後続契約で実 config/invocation override、既存 precedence、対象 service と policy role を特定する。未定義 config-to-policy mapping/new precedence は Q-decision に戻す。policy/config fingerprint を統合しない。

**Q4 — 寿命。** A: submission-local borrowed/owned immutable association、既存 point check/scoped IR retain。新 submission ごとに再構築。B: session/registry ごとに reusable context を保持し invalidation/per-use revalidation。stale authority と解放経路を減らす **A を推奨**。B は replacement/cancel/reconfiguration invalidation が必要。volatile lifetime ID を key に含めず atomic Store/currentness を約束せず retention を currentness にしない。

**Q5 — 実装の段階。** A: 必須 runtime identity 前提を含め既存 native Disk SourceLoad profile の genuine binding を先行し、cold Frontend producer/consumer は別 freeze。B: binding と genuine 独立 cold Frontend expectation/lookup を一括実装。小さい oracle で対応 gap を検証する **A を推奨**。B は Frontend footprint/expectation matrix 全体と owner scope が必要。双方とも不完全 context を認めず、本定義で registered NoKey behavior を変更せず full STEP6C としない。proof reuse/dependency-binding/他 service gap は別。

**Q6 — context unavailable。** A: 必須対応が完全でない task/profile の最適化だけを控える。B: 必須対応が一つでも unavailable なら当 submission 全体の任意 cache preparation を控える。独立した適格最適化を保ち architecture 22 の performance-only fallback に従う **A を推奨**。B は単純だが無関係な work も再実行し得る。双方とも ordinary execution、既存 error/publication suppression、cache 無効、invalid-input/producer-error 順序を保持し新 public diagnostic なし。cache miss で valid run を拒否せず semantic failure を成功にしない。

## 条件付き検証と引き継ぎ

frozen 定義契約の六件は選択肢承認後のみ実装義務となる。otherwise-viable genuine positive と independent clean oracle を用い、実 capture/bootstrap/登録 execution/Store record から cold/warm を観測し、test 作成 context/Hit/Complete を根拠としない。source/module/package/snapshot/plan/config/lock/policy 対応、genuine runtime selection、cross-run swap、service/schema/provenance、complete/incomplete footprint を独立に変える。equal label/hash で対応欠落を隠さず refusal control/unknown compatibility では最適化を控え、実効 policy を弱めず ordinary output/canonical diagnostic を保持。
cancel/supersession、publisher replacement、collected slot、全 owner-release return を deterministic checkpoint で検証し retained read の current use を拒否。cache 無効、非対応 origin/profile、producer 欠落、実 Store/lookup/decode failure と viable ordinary execution を比較。既存 config/lock/toolchain、SourceLoad/retained-Frontend、Cache/IR matrix を再利用し複製しない。Frontend/proof/import は別 admission まで拒否 control とし context で解禁しない。
owner が Q1–Q6 を決定してから Muse が実装を dispatch。後続 freeze は genuine producer API/identity source、該当 phase closure、config/policy mapping/role、purity/compatibility/検証順序、owner write scope、lifetime/release、test case/focused/E2E gate を特定する。欠けた前提は明示 unavailable 境界とし捏造しない。新 semantic/compatibility choice は owner へ戻す。本定義は source/test/spec/trace/coverage status を変更しない。
