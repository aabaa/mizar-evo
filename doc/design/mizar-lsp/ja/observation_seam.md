# Driver 所有の native checkpoint 観測 seam
正本: 英語。[English companion](../en/observation_seam.md)。

## 状態と権限

**Q1–Q5 A は 2026-11-16 に owner 承認済み。** 以下の選択肢・推奨は提案時の根拠として残し、B 案は実装しない。観測実装と保留中 Task40 の修正は別 dispatch とする。Task41 の presentation 判断は変更しない。
権限: [承認済み実行 seam](execution_seam.md#implemented-a-package)、[bridge lifecycle](build_bridge.md#supersede-and-terminal-lifecycle)、Driver [execution](../../mizar-driver/en/driver.md#nonterminal-captured-execution)・[cancellation](../../mizar-driver/en/driver.md#cancellation)・[scheduler boundary](../../mizar-driver/en/driver.md#scheduler-boundary)・[session identity](../../mizar-driver/en/request.md#buildsession)・[events](../../mizar-driver/en/events.md)、internal01 [checkpoint policy](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md#cancellation)、spec [§23.6](../../../spec/en/23.package_management_and_build_system.md#236-lsp-integration-and-ide-support)。言語・proof・diagnostic・scheduler policy は提案しない。
`crates/mizar-driver/src/frontend_adapter/frontend.rs` の native `FrontendService::execute` checkpoint は、入力 snapshot と一致する token に対し `PhaseStatus::Cancelled` を返す。Driver registry dispatcher は結果を受けるが、finalization は `crates/mizar-driver/src/driver.rs:1777` で obsolete `phase_results` を消去する。そのため scheduler `TaskState::Cancelled` と空の result/output 集合だけでは checkpoint 受領を証明できず、service dispatch 前の cancellation でも成立する。これは証拠の欠落であり、実証された production defect ではない。

## 必要な証拠境界

観測は登録された genuine native Frontend 呼出しに由来し、その実 checkpoint 受領/result を scheduler が選んだ task と Driver session に結び付ける。request/ack、token を渡した事実だけ、scheduler cancellation、終端 outcome、公開 output の不在はいずれも受領の証拠ではない。既存 `pending_task` は呼出し前の admitted work を識別するが、その後の呼出しを証明しない。
native checkpoint 到達後、Driver の result 正規化・stale-result 消去・current handoff が provenance を失わせる前に記録する。return-witness 案では未変更の戻り値の事実を記録し、既存 result 検証に成功してから、かつ suppression 前にのみ positive receipt として公開する。記録済み証拠の後からの read-only 取得でよく、Driver の排他 borrow 中の callback は不要。suppression は観測を消去せず current output にも変えない。順序は時刻推測ではなく実 dispatch/return に従う。
必要な事実は authoritative session identity、実 graph TaskId と Frontend/service provenance、checkpoint kind、入力 snapshot、その呼出しへ実際に渡した厳密な cancellation token（Build cancellation generation/reason を含む）、主張を支える genuine result/receipt の事実である。request id・lane・request generation は元の session から解決し、request generation と cancellation generation を混同しない。同一 snapshot の実行間では snapshot や TaskId だけでは不十分。最小の immutable summary を保存し、phase payload・diagnostic・output・capture・publication token は複製しない。
現 service-entry checkpoint の return witness は native TypeId、input/token snapshot 一致、genuine Cancelled return、元の空 output/no-artifact result shape を確立し、既存検証を維持する。現 native code はこの matching-token branch からのみ Cancelled を返す。将来別の Cancelled return path を追加するなら、checkpoint 帰属の一意性を維持するか witness を強化する。同じ descriptor/status を名乗る非 native service は native 証拠にならない。未対応 path と記録不在は証拠なしであり、受領を推測しない。

## 所有権と不変条件

Driver が記録・対応付け・取得を所有する。Build は admission/checkpoint policy、native service は実行/受領を所有し続ける。同じ native concrete TypeId・registry service・dispatch resource・実行 route を保ち、service wrapper、producer 置換、別 scheduler、偽 result、観測のための再実行を認めない。観測は scheduling・cancellation reason・validation・canonical result/event・publication・acceptance に影響しない。厳密な API 名/型は承認後の実装契約が所有する。
承認済み first-reason・drain まで Cancelling・terminal replay 規則が権限を持つ。受領と drainage、単一 SessionFinished を区別する。観測は capture lease・artifact access・editor publication・currentness を付与しない。同一 snapshot の replacement と Watch/LSP session は別の証拠/control/publisher を保つ。cancel より先に勝った completion は completion のままで、受領を遡って捏造しない。checkpoint/progress/cleanup の latency 保証は追加しない。
obsolete session と Shutdown/exit 後の cleanup progress でも、終端結果が抑止された場合を含め読み取れること。反復 read/replay は観測生成・実行前進・cancellation 消費・終端 event 重複を起こさない。保持は明示し、保持中 execution ごとの実観測呼出しに限定して Q4 により owner と共に解放する。global log、capture の独立保持、新 session-eviction policy は導かない。

## 承認済み owner 判断（すべて A、提案時の選択肢を保持）

**Q1 — 証拠をどう公開するか。** A: session をキーにした専用 read-only Driver query とし、unknown session・既知で観測なし・観測ありを明示的に区別する。既存 submission/event API を維持する。B: 既存 replay に typed Driver observation event を追加し、同じ区別と suppression 前記録を保証する。**A を推奨**: canonical event schema/order や subscriber の変更を避ける。B は replay を再利用できるが event owner・互換性・stale-publication 分類の変更が必要で、観測 event は output/publication と数えられない。
**Q2 — native 受領を何で証明するか。** A: 既存 registry 境界で上記事実を持つ native-dispatch return witness を記録し、native matching-token Cancelled branch を provenance invariant とする。B: execution resource に Driver-owned receipt sink を加え、既存 native checkpoint 内で直接通知する。native service TypeId/route を維持し、受領と後続 result 検証/drainage を区別する。現 checkpoint では新 service/context callback が不要な **A を推奨**。B は Cancelled に別起源が加わった場合も checkpoint 帰属を明示できるが context/lifetime ownership が広がり、通知契約の別 freeze が必要。両案とも token や scheduler decision だけから受領を推測せず、B の observer 動作は実行を分岐・条件化しない。
**Q3 — identity/API の範囲はどこまでか。** A: genuine native Frontend service-entry checkpoint に限り、owner session と実 TaskId/native checkpoint discriminator を組み合わせ、既存 session identity/token 型を再利用する。B: 全 phase 向けの一般的な service-span checkpoint 観測 API とし、session/task と phase/service/checkpoint の区別を保持する。Task40 の不足前提には **A を推奨**。B は他 producer にも再利用できるが、それぞれの owner-defined provenance と大きな検証 matrix が必要であり、未計測 service に cancellation 証拠を付与しない。どちらも snapshot/task identity を共有する別 session を統合せず、caller 作成の receipt record を受け付けない。
**Q4 — いつまで保持し、どう解放するか。** A: captured execution の start 時に opt in し、immutable summary を既存 session execution/replay record と共に drainage/stale replay 後も保持し、同 record と共に解放する。read は非消費で、観測 disabled と enabled で受領なしを明示的に区別する。B: start 時に opt in し、取得後の明示的な consuming drain/ack を設け、consumed state と ack または session-record 解放までの保持を定義する。第二の cleanup protocol が不要な **A を推奨**。session ごとの量は実 native 呼出し数で制限するが、新たな global capacity 保証ではない。B は早く解放できるが reader 喪失/反復 read の意味が必要。両案とも約束した取得期間前に証拠を黙って捨てず、compiler resource を独立保持せず、checkpoint 後の遡及 enable を認めない。
**Q5 — 前提検証はどこまで必要か。** A: genuine captured OpenBuffer native work と既存 admitted-task progress 境界を使う。start 前に観測を enable し、実 pending Frontend を確認して Running 中に cancel を要求し、実呼出しを progress して suppression/drainage 後の witness を取得する。下記六つの control と既存 synchronous/native regression を含める。B: Disk/OpenBuffer と synchronous/nonterminal native route に同じ seam の matrix を広げ、追加 native checkpoint kind は owner 契約が存在してから含める。前提を限定する **A を推奨**。B は再利用の証拠を広げるが interior checkpoint の捏造や blocking call 中の cancellation access を暗黙に導かない。両案とも positive evidence に sleep/timeout race、live-work 証明に submission 前の cancel、synthetic acceptance、private state 改変、native wrapper を使わない。

## 条件付き検証と引継ぎ

| Case | 承認・実装後に必要な証拠 |
|---|---|
| 1 | 観測 enabled の genuine native 呼出し: session/task/checkpoint/token/result 対応を suppression 前に記録し、obsolete session の phase result 消去後も positive receipt を保つ。disabled query と呼出し前の既知 enabled session は受領を主張しない。 |
| 2 | 同一 snapshot の replacement 受理: old/replacement identity を区別し、old native receipt は受理済み Driver Superseded request 由来の SupersededSnapshot を持つ。obsolete output/publication はなく、replacement は自分の証拠で非 cancel 実行する。 |
| 3 | Shutdown/exit: genuine native receipt は supersession でなく Shutdown を持つ。progress 前の Cancelling、続く drainage と単一終端 event を確認し、protocol 終端/stale replay 後も証拠を取得できる。 |
| 4 | native dispatch なしの scheduler cancellation に native receipt はない。request なしの成功、無関係 failure、無効 result/token、cancel 前の completion は受領を捏造しない。反復 cancel/read/progress は勝った reason/result と観測数を維持する。 |
| 5 | 同一 snapshot の別 lane/Watch と replacement work は control/publisher/evidence を分離する。read は実行/publication に副作用を持たない。retention/disabled/unknown（Q4-B なら consumed）状態と owner cleanup は承認済み取得期間に従う。 |
| 6 | 非 cancel の enabled/disabled 実行で genuine canonical result/status/event/publication を保ち、同期互換性を含める。決定的 native test は receipt を除去したり suppression 後に移したりすると失敗する。前提実装は Task40 の combined bridge acceptance とは数えない。 |

既存 intent は Driver `tests/source_load/import.rs` の checkpoint test・lifecycle/replay control と LSP `build_bridge_event_tests.rs` / `build_bridge_timing_tests.rs` に残る。選択した API/provenance/retention 境界は Driver の [native checkpoint observations](../../mizar-driver/ja/driver.md#native-checkpoint-observations) が所有する。Muse は Task40 の test-only 修正を別 dispatch し、bridge 経由 Superseded/Shutdown 受領・drainage・identity・publication の組合せを検証する。本 seam は Task40 の combined bridge acceptance、semantic coverage、§23.6/SCA-003 credit を与えず、Task41 と他の owner-gated 判断を分離する。
