# Genuine reflexivity Kernel/status service handoff 案
canonical English: [English companion](../en/genuine_reflexivity_status_handoff.md)。

## 状態と根拠

**Owner は 2026-11-17 に Q1–Q5 の A を承認し、B は未承認のままとする。** この案は実装前の handoff 境界を記録する。[Driver plan](00.crate_plan.md#task-index) が定義契約を索引し、個別に freeze した契約が実装と readiness を規定する。この承認はより広い実装を許可しない。
根拠は英語正本がリンクする仕様 §16.5、§21.7.1/§21.7.5、§23.4/§23.5.1。built-in discharge は元の obligation に対して実 Kernel が受理した formula/substitution evidence を要する。言語・受理方針は変更しない。

## 既存 producer と不足する consumer

[Proof original-graph check](../../mizar-proof/en/status.md#source-theorem-reflexivity-original-graph) は実 Core/VC graph と Kernel result/policy decision を返す。genuine status、trusted used_axioms、source FILE は別 facade であり、統合 Driver receipt ではない。明示的 BuiltinDischarge origin、DischargedBuiltin class、不完全な dependency slice と mandatory MISS を維持する。
M1.3 の [FILE staging](../../mizar-proof/en/witness_store.md#explicit-source-theorem-file-staging)、[built-in main profile](../../mizar-artifact/en/verified_artifact.md#explicit-built-in-enclosing-profile)、committed reader、witness publication は既存 producer。native service 登録や currentness を証明しない。
Driver91 wrapper は [廃止済み](frontend_adapter.md#native-theorem-reflexivity-core-vc-caller)。[登録 candidate-check](frontend_adapter.md#scheduled-source-theorem-candidate-check) は別の通常 ATP 経路。`candidate.rs` の FILE は KernelVerified を要求し、`artifact.rs` は AtpFormulaSubstitution origin と KernelVerified projection を要求する。その guard 削除や DischargedBuiltin の再分類は既存境界を越える。
不足するのは genuine reflexivity evidence/status を実 KernelCheck service の新鮮な認可済み output として運ぶ境界。M1.4 と M1.5 は未完了で coverage/milestone credit は付与しない。

## 全選択肢の不変条件

Driver は capture association、登録、task input、sealed transport、live lane/snapshot/publisher 認可を所有する。Resolver/Checker/Core/VC は source check と元 graph、Kernel は evidence 検査、Proof は origin/policy/selection/status/trusted used_axioms/witness、Artifact は schema/store/manifest commit/read reachability を所有する。Driver が semantic acceptance を再構成してはならない。
実 captured source/module、authentic retained chain、選択した stored VC、明示 stored schema、実効 verifier policy を使う。数値 VcId/hash だけでなく元 Core/VC 全体を認証する。[raw Core/VC handoff](frontend_adapter.md#native-theorem-kernel-handoff-core-vc-caller) は source admission/lineage 制約であり受理ではない。genuine Proof reflexivity producer を使い、raw handoff/default discharge 成功で代用しない。
publish 前に一意の phase/kind parent role、snapshot/module/input/dependency identity、physical retention、publisher 認可を検証し、lower work 後に live guard を再検証する。status を実 goal/context/polarity/evidence/policy/trusted used_axioms に束縛する。BuiltinDischarge origin は検査前に選び成功から推論しない。上流 refusal/error 順序を維持し unsupported、rejected/open、resource/internal failure を新 public diagnostic なしで区別する。
実 Kernel と既存 policy の受理後のみ publish。元 Pending/Active/Open graph はそのままで、不完全 slice を complete にせず cache/reuse、他 obligation discharge、session verified を導かない。task Complete は transport 成功のみ。cancel/stale lane、後続 snapshot/lane association として受理した source change、collected parent、publisher replacement は既存 point check で新 publish を防ぐ。frozen capture は disk edit/removal を再検査しない。過去の retained payload/token は live authority を与えない。
draft、構造的に読める FILE、hash は committed witness ではない。後続 FILE/main/manifest consumer は M1.3 の class/identity/policy/evidence/staged-reference 一致と reachability 検査を使う。selected-witness-hash 不在、generic/state built-in gap、mandatory MISS、caller-expected-snapshot/retained-token 制限を保持し、cross-edit/transferable receipt を新設しない。

## Owner Q-decisions

**Q1 — 登録と graph 経路。** A: genuine Core/VC/source-check 前提を持つ opt-in genuine-reflexivity KernelCheck profile。架空の AtpSolve/BackendRun/通常 candidate parent は不要。B: candidate-check に明示 tag と分離 parent/input 検証で built-in 分岐を追加。通常 ATP を保ち混同を拒否する **A を推奨**。B は登録を減らすが accepted-input 境界を広げる。双方とも exact descriptor/output kind/graph/default-input のレビューが必要。既存 service 数で暗黙に許可しない。

**Q2 — Kernel result と status の結合。** A: Proof 所有の限定 composition が同じ genuine evaluation の元 graph、実 result/decision、対応する status/full slice/trusted used_axioms を既存 owned value で返す。B: Driver が別 facade を呼び元 graph/identity/policy/evidence 全体の対応を検証。semantic producer が same-result 保証を持つ **A を推奨**。統合 API は未存在で、新 public semantic model を作らず exact API/error を定める Proof 前提契約が必要。B は entry point を避けられるが replay/equality 契約が必要で hash だけでは不十分。双方とも caller 作成の acceptance や Driver 内 Kernel 実装を認めない。

**Q3 — 最初の実装範囲。** A: M1.4 初回は新鮮な sealed Kernel/status output まで。genuine FILE/Artifact consumer は M1.3 producer を使い別契約。B: Kernel/status と witness/Artifact service 適応を同じ実装契約に含める。境界を独立し genuine source E2E で確認する **A を推奨**。A だけでは committed-artifact M1 目標を満たさず B は大きい staging/commit/readback matrix が必要。通常 KernelVerified guard を緩和せず Artifact17 完了を主張しない。

**Q4 — refusal と ATP fallback。** A: 既存 unsupported/open/rejected/error を返し自動 ATP fallback なし。B: 独立した real candidate/check 前提と origin/status を保ち通常 ATP profile へ明示 fallback。backend-free 経路と refusal 因果を限定できる **A を推奨**。B の scheduling/failure composition は実装前の別承認が必要。双方とも実効 policy を変更せず disabled built-in を弱い policy で再試行しない。backend 成功は受理ではない。

**Q5 — status 公開。** A: internal typed sealed service data のみ、既存 task/event 意味を使い CLI/LSP/public status schema は延期。B: versioned public Driver result/event に distinct built-in proof status も公開。presentation 互換性契約なしで handoff を確立する **A を推奨**。B は event/schema/consumer/stale-publication 規則を実装契約に要する。task/session complete を proof acceptance、retained data を current publication としない。

## 条件付き検証と引き継ぎ

次の六件は承認済み実装範囲でのみ runtime obligation となる。既存 test intent/selector は Driver plan の定義契約が索引する。ここでは test/expectation を変更しない。

| Case | 入力と要求結果 |
|---|---|
| 1 | Disk/OpenBuffer の omitted-step singleton reflexivity と rename/trivia。実登録 service が authentic 元 graph/実 Kernel result と DischargedBuiltin/trusted used_axioms を結合。fake positive なし、元 graph/full slice 不変。 |
| 2 | genuine source/module/snapshot/VC/policy の別 product と schema/goal/context/polarity/evidence 不一致を publish 前に拒否。各 lower baseline を成立させ無関係な早期 guard を原因にしない。lower Kernel rejection matrix 再利用。 |
| 3 | disabled policy、cited/open/multiple/unsupported source、backend-only 成功は既存結果を保持し accepted status/FILE と policy 弱化なし。Q4-B なら実 fallback と独立 normal check を証明。 |
| 4 | 実 registration/profile/descriptor、厳密 parent role/lineage、旧 profile/通常 ATP control。承認経路のみ genuine input を受け cross-profile/不正 payload/missing 前提を拒否。sealed output と admitted input の厳密な束縛、および異なる束縛の run 間で genuine output を交換した際の拒否を検証。fake positive TypeId/direct-D 代用なし。 |
| 5 | cancel/replaced lane、stale/collected original parent、retained reader。古い結果を復活させず current publish を防ぐ。read は work を進めず sleep race でなく決定的 guard control を使う。 |
| 6 | status/FILE/main/manifest/readback/publication の一致と class/identity/reachability 不一致 control。DischargedBuiltin/trusted used_axioms/不完全/MISS/retained-token 制限を保持。Q3-A では後続 consumer 責務で初回 service 完了 evidence ではない。normal/generic/state 不変。 |

owner は Q1–Q5 の A を選択した。Muse が各限定実装契約を割り当て exact producer API、genuine source/profile oracle、registration/graph/input/output/schema、policy/refusal、owner write scope、該当 matrix、focused/E2E gate を freeze。Q2-A の Proof composition は前提であり transferable receipt ではない。Q3-A の FILE/Artifact consumer は別 dispatch を要する。この定義から readiness/実装承認を推論せず新 semantic/acceptance choice は owner に戻す。
