# Vampire の evidence extraction

正本言語: English; [英語 companion](../en/evidence.md)。
状態: guarded Vampire 4.9 extraction を `backend::run_vampire_backend` に実装。candidate は既存の kernel-checked finder だけから生成する。凍結済み契約は設計記録として保持する。
根拠: [仕様 §21.7.1–§21.7.5](../../../spec/en/21.source_code_annotation_and_atp.md#217-atp-integration)、[architecture 10](../../architecture/ja/10.atp_backend_integration.md#具体例探索による根拠の生成)、[backend contract](backend.md)。

## Guarded route と入力

この route の backend は Vampire のみとする。既存の [generic runner](../en/backend.md#process-model)、[profile/command](../en/backend.md#backend-profile-and-command)、version probe、private stdin/file input、stream capture、policy timeout/resource limit、exit/stream fingerprint、private-temp cleanup を使う。daemon、library linking、shell interpretation、fallback backend は導入しない。
translator が encoded problem から生成する、宣言 profile 用の既存 deterministic [TPTP FOF encoding](tptp_encoder.md) だけを送る。source text、追加 premise、premise set の拡大、backend 固有の入力追加を禁止する。生成する全 record を encoded-problem hash に束縛し、元の target、formula-label、symbol-binding、provenance metadata を保持する。

guarded API は既存 run input、元 structured problem と VC/kernel handoff、変更しない optional finder limit を受け取る。引数は `--mode vampire --input_syntax tptp --proof tptp --output_axiom_names on`、environment allowlist は空、同一 executable への絶対 command/probe path と `--version` probe を要求する。

## Extraction と advisory mapping

extraction は、完了した一回の実行の captured output を既存の [result-classification](../en/backend.md#result-classification) 入力と任意の advisory used-premise report に写す。何も受理せず、trusted evidence を生成しない。stdout から取るのは SZS status、使用したと報告された encoded-problem formula name、backend identity/version と再現用 metadata のみとする。stderr とその他の output は診断専用である。
advisory report は、使用したと報告された input formula name の集合である。集合全体を encoded problem の formula label と symbol binding を通じて元の premise に対応付け、label 順を決定的にし重複を除く。finder には実際の premise label だけを渡す。conjecture label は premise ではない。report は探索を狭めるだけで、formula、substitution、追加 premise、trusted `used_axioms` を供給しない。
proof object、TSTP trace、inference step、proof 内の substitution は candidate payload に入れない。advisory set 用に input-name reference を読むことは trace の evidence への変換ではなく、proof 内容はすべて診断専用のままである。

SZS Proof block 内の axiom-role input record の `file(input,name)` annotation だけを advisory name とし、conjecture-role record は除外する。formula と inference annotation は opaque のまま扱う。

## Finder handoff と信頼境界

既存の [instance finder API と supported-input boundary](../en/instance_finder.md#api-and-boundary) だけを使い、元の structured problem、encoded problem、VC/kernel handoff を保持する。新 candidate schema は定義しない。`CandidateKernelEvidence` / `BackendCandidateEvidence` は、明示的な kernel-schema formula/substitution payload bytes または payload reference、target binding、encoded-problem hash、provenance と一致する `formula_label_refs` / `symbol_binding_refs` を含む。metadata だけでは evidence にならない（[candidate boundary](../en/backend.md#candidate-evidence-boundary)）。
advisory label は premise 探索だけを制限する。finder が substitution と named equality-instance candidate を生成し、kernel が equality formula を構成して candidate を独立検証する。[verifier-policy search limit](../en/instance_finder.md#search-and-limits)、認証済み入力、決定的順序を維持する。backend output から goal polarity、context-identity payload、imported-statement projection を供給せず、task-29 の upstream boundary を維持する。
finder candidate を伴わない backend success は kernel evidence を生まず、既存 classifier では `Proved` にできない。externally attested/open の扱いは downstream policy が所有する。extraction は受理、proof status、witness publication、cache promotion を与えない。

## 失敗と決定性

candidate handoff 前に [既存 failure semantics](../en/backend.md#failure-semantics) を適用する。timeout、cancellation、crash、nonzero exit、不完全/破損 capture、SZS status 欠落/不正、構文解析不能な advisory output は、その実行から candidate を生成しない。未対応または不一致の observed result は `Proved` にしない。
構文解析できた advisory report でも、未知 label や non-premise label などにより全体を対応付けできなければ、診断を付けて全体を破棄する。部分的に狭めず、report がない場合と同様に全 encoded premises で finder を実行する。この mapping fallback は process、status、version、元 handoff、policy の検査を免除しない。report がない場合も全 encoded premises を使う。
extraction は encoded problem、captured stdout bytes、pinned version record の純粋で決定的な関数とする。同じ入力は同じ advisory set と handoff を生み、元 handoff と policy limit も同じなら finder candidate id と hash も一致する。

## 実行ファイルの provisioning

outer orchestrator が凍結後、実装 dispatch 前に sandbox 外で [公式 release distribution](https://github.com/vprover/vampire/releases) から Vampire 4.9 を provision する。この checkout 内の `.tools/provers/vampire-4.9/vampire` に実行ファイルを配置し、tooling の `.tools/` は ignore する。この open-source route に購入、licence 取得、credential、private binary は含めない。この凍結では何も install しない。
provisioning 時に binary の SHA-256 を計算し、provisioning/implementation run record に記録する。live design document には記録しない。solve 前に既存 runner の version probe で Vampire と自己申告 version 4.9 を確認する。実行ファイル欠落、probe 失敗、version 不一致は backend unavailable とし、solve、代替 executable、fallback なしで route を拒否する。run metadata に backend identity/version と command fingerprint を保持する。

## 保留する所有範囲

portfolio execution、CVC5/Z3/E adapter、SMT-LIB extraction、TFF/typed path（ATP-G-010/ATP-G-012）、proof policy、witness/cache/artifact integration、externally attested handling は各 owner に deferred のままとする。CVC5 は theory-heavy goal の primary、Z3 は optional/diagnostic のままで、どちらもこの route に加えない。Step 7 item 1 は未完了とする。実装は凍結契約の real encoded-problem matrix と guarded executable を検証してから ATP-G-015/ATP-G-016 の解決を記録する。provisioning だけでは不十分である。
