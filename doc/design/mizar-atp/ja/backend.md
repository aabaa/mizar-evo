# Module: backend

> 正本は英語です。英語版:
> [../en/backend.md](../en/backend.md)。

## 目的

`backend` module は、すでに encode された 1 つの `AtpProblem` に対して、1 つの外部 ATP/SMT
backend を呼び出す方法を仕様化する。child-process execution、resource limit、captured-output
metadata、backend version recording、timeout/cancellation の graceful handling、policy-neutral な
backend-result classification を所有する。

backend module は引き続き evidence producer boundary である。proof acceptance、
Kernel checking/acceptance API 呼び出し、witness publication、portfolio winner selection、
cache update、backend proof method、log、unsat core、SMT proof object、TSTP trace、
resolution trace を trusted acceptance material に変えることは行わない。generic process runner は SAT を実行せず、下記の別の限定 source query が既存 SAT service を untrusted 候補の検査にだけ用いる。

## 範囲

Task 13 は specification-only である。将来の source module が backend runner API、mock-backend
test harness、deterministic run metadata、fail-closed result classification を公開することを許可する。
Rust source、process spawn、real backend integration、backend proof language parsing、real backend
からの formula/substitution candidate extraction、kernel 呼び出し、artifact witness publication、
portfolio、proof policy は追加しない。

task-14 source は、ここで記述する generic process runner と mock backend fixture を実装する。
task 15 は first real-backend route を再評価し、`external_dependency_gap` / `deferred` として
記録した。`mizar-atp` が real backend output を kernel-owned formula/substitution candidate
payload に parse するには、paired evidence-extraction spec がまだ必要である。広い
classification fixture と polarity validation は task 16 に残る。ただし task 14 は invalid な
`Proved` result を表現不能にする constructor と invariant を公開してよい。

## 入力と出力

概念上の task-14 API は次を消費する:

```text
BackendRunInput
  run_id
  encoded_problem
  backend_profile
  command
  resource_limits
  io_mode
  cancellation
```

そして次を生成する:

```text
BackendRunResult
  run_id
  encoded_problem_hash
  backend_identity
  command_fingerprint
  status
  observed_result?
  candidate_evidence?
  counterexample?
  stdout_hash
  stderr_hash
  stdout_excerpt?
  stderr_excerpt?
  exit_status?
  termination
  timing
  resource_observations
  diagnostics
```

`encoded_problem` は concrete backend input と、実行後に必要な metadata を含む: source
`AtpProblem.problem_id`、target binding、expected result、logic profile、concrete format、
formula label / assertion label、symbol binding、provenance hash、semantic input hash である。
backend runner は caller-supplied instantiated formula、SAT clause、proof method、backend
`used_axioms` を trusted field として受け取ってはならない。

## encoded problem contract

runner は concrete encoder が生成した immutable encoded input を消費する:

| Field | Requirement |
|---|---|
| `problem_id` | source `AtpProblem` identity。backend metadata を backend-neutral problem に結び付けるが、proof acceptance ではない。 |
| `target_binding` | candidate evidence へ持ち越す stable target binding。mismatch は `Proved` result を構築する前に fail closed する。 |
| `expected_result` | backend success polarity。現在は `ExpectedBackendResult::Unsat`。 |
| `concrete_format` | encoder が選択した TPTP または SMT-LIB profile。 |
| `input_text` | backend input の exact bytes。 |
| `input_hash` | backend input bytes と encoding profile の hash。 |
| `symbol_bindings` / `formula_labels` | 後続 candidate extraction が使う encoder side metadata。 |
| `provenance_hash` | ATP provenance metadata に対する stable hash。 |

backend runner は `input_hash` と metadata hash を記録する。input を書き換えたり、backend text を
normalize したり、proof command を追加したり、unsat core を trusted data として要求したり、別の
expected result を推論してはならない。

## Genuine encoded input production

`EncodedBackendProblem::encode(&AtpProblem, ConcreteFormat) -> Result<Self, String>` は既存 TPTP FOF / SMT-LIB Uninterpreted encoder を実行する。未対応 profile の error をそのまま伝え、reprofile・本文修復を行わない。返却された UTF-8 byte・raw label、実 problem ID・target・Unsat 方針・profile name と明示的 fragment tag `fof` / `tff-like` / `smtlib-uninterpreted` を保持する。既存 constructor と hash domain は変更しない。
実 symbol row の ATP name・concrete name・型付き source を `a=<UTF-8 byte長>:<小文字hex>;b=<byte長>:<hex>;s=<source>` に正準化する。source は既存 MizarSymbol / GeneratedBinder / TypeGuard に対応する `mizar=<byte長>:<hex>` / `binder=<byte長>:<hex>` / `guard=<10進id>`。Debug、区切り曖昧性、並行 symbol model を導入しない。
既存 private StableHasher の domain `mizar-atp/backend-source-provenance/v1`、byte field `source-problem-id` に実 problem_id().hash() を入れて provenance_hash を作る。この保守的 binding は正準問題の全 typed provenance・formula・declaration・target・symbol map・profile を含み、無関係な意味変更でも無効化され得る。既存 identity は diagnostic を除外する。provenance 専用 reuse key や proof evidence ではなく、fixture 定数・入力 receipt を使わない。
別 encoded output・label・hash を入力せず、実 encoder が label 選択・mangling、backend が不変 input binding のみを所有する。全 source 型・固定 mapping byte・coherent metadata 変更・diagnostic neutrality・両 format の実 source pipeline を検査する。汎用 source/backend integration・instance search/limit・advisory extraction・execution/classification・proof acceptance は別依存として保留する。

## Closed-choice reflexivity names

`encode_closed_choice_reflexivity_names(&ParsedKernelEvidence) -> Result<Vec<Vec<u8>>, String>` は実際の Kernel-parsed Formula v1 data のみを読む。caller が source/context を認証し、名前は未信頼の輸送であって証明や完全な candidate ではない。ATP が決定的な instance 選択を所有し、Kernel だけが名前を decode して formula を導出・検査する。
schema 1 と一意の Equality manifest key を要求する。各 substitution は payload kind 1、空の rewrite path、role 1 の replacement 1件、その actual term は引数0の FunctorPredicate application でなければならない。未対応入力は部分結果なしで拒否する。実 SymbolKey の昇順で重複除去し、検査した昇順の非零 u32 instance ID を付ける。substitution がなければ名前もなく、source string/hash から数値 identity を導出しない。
各 item は既存 Formula-v2 grammar を保持する: big-endian instance ID、reflexivity tag 1、実 equality symbol ID、term/application tag 2/2、実 choice symbol ID、引数数0。通常の VC transport が item を frame 化する。instance formula の入力、探索、SAT 検査、backend 実行、candidate 分類、verifier budget を含まない。
[Proof の source facade](../../mizar-proof/ja/status.md#source-existential-registration-proof) が fresh v1 parse 後にこの処理を消費する。実 Open/NeedsAtp source caller で byte と未対応 parsed shape を検査し、完全な ATP 抽出と policy 所有の探索上限は保留する。

## Source registration candidate inputs

`build_source_registration_candidate_inputs(&CoreIr, &VcSet, VcId, LogicProfile, ConcreteFormat) -> Result<(EncodedBackendProblem, BackendCandidateEvidence), String>` は結び付けた未信頼輸送のみを作る。既存 [source translator](translator.md#source-registration-projections) と実 encoded factory を実行し、独立に [同じ source handoff](../../mizar-vc/ja/kernel_evidence_handoff.md#source-existential-registration-proof) を再構成する。generator/default-discharge 全体を再実行して選択した NeedsAtp graph を認証し、元の Open、選択した Discharged、未対応・整合的に偽装した入力では pair を返さず error とする。明示した profile/format を保ち、pruning・fallback・reprofile をしない。
実 handoff を local Formula v1 として frame 化し、実 target と kernel profile/tautology policy を使って既存 Kernel parser を呼ぶ。これは構造検査であって受理ではない。parsed 数値 key を closed-choice name writer に渡し、同じ handoff の Formula v2 に名前を frame 化する。ATP/backend 名から term を復元せず、instance formula を入力しない。元の全 source formula・置換・provenance・refutation goal を保つ。
既存 FormulaSubstitutionBytes candidate に正確な v2 byte と fresh encoded container の完全な target binding・input hash・provenance hash・全 label・全 symbol mapping を入れる。別の candidate byte・hash・identity・encoded output は入力しない。未対応 source/encoding/framing/parsing/name は部分結果なしで中断する。
candidate_id は既存 private StableHasher の domain `mizar-atp/source-registration-candidate/v1` から、byte field `encoded-input-hash`、`encoded-metadata-hash`、u64 field `target-fingerprint-algorithm`、byte field `formula-substitution-bytes` の順で作る。実 container hash・target algorithm ID・正確な payload byte を使い、既存小文字 hex と prefix `source-registration-v1:` で表す。metadata は target digest/producer・label・mapping・source provenance を結び、旧 hash を変えず algorithm を明示的に加える。input hash は実 text/format/profile、payload は通常の evidence field を結ぶ。Debug・runtime data・report・caller receipt を hash に入れない。
pair は未検査の試行であり、fresh Kernel check が拒否する実際の偽の source も含む。backend result や探索成功ではない。caller が現 source/context 認証と独立した通常 [Kernel 検査](../../mizar-kernel/ja/checker.md)、[Proof](../../mizar-proof/ja/selection.md) が受理を所有する。既存 original-Open built-in facade は変えない。探索上限・汎用 finder・advisory 抽出・backend 実行・publication・reuse は保留する。
異なる実 source 回帰 caller 3箇所で direct concrete encoding/metadata、独立した通常 wire/fresh Kernel outcome、source/context 全体の破損を検査する。両 format、元・改名した正/負の single/aggregate source、決定的 replay/固定 identity、元 Open・選択した Discharged の拒否、破損 output の未信頼性を扱う。既存 manual/producer oracle を維持する。

## Source candidate SAT screening
`screen_source_registration_candidate(&CoreIr, &VcSet, VcId, LogicProfile, ConcreteFormat, Option<[u64; 4]>) -> Result<Option<(EncodedBackendProblem, BackendCandidateEvidence)>, String>` は既存 source factory の一候補だけを検査し、backend 実行・項/置換生成・証明受理を行わない。
変更しない source candidate factory、その後 source ground-term correspondence を同じ immutable tuple で呼び、元の whole-input/profile/encoding 拒否を limits より先に保持する。optional setting は caller（既存 Proof policy getter など）から渡し、新依存や default を作らない。None は未設定で None を返し、literal zero/MAX を保持する。既存 Proof facade、policy fingerprint、prebuilt candidate の挙動は変えない。
完全な既存 observed correspondence pool、実 parsed candidate の substitution record 数、named equality record 数を最初の三座標と比較し、第四座標は SAT 一回を許可する必要がある。usize-to-u64 は checked conversion とし、切り捨てない。これは限定候補の admission count であり、先行 source load/build や一般探索の列挙を制限するものではない。超過は SAT encode/solve 前に None とし、部分 pair を返さない。
設定時は同じ tuple の元 handoff を実 target/profile/tautology parser context の取得だけに再構築し、schema2 を明示許可して元 candidate FormulaSubstitutionBytes を parse する。wire/name/provenance/candidateID/encoded binding/入力は保持し、parse error は内部 String で伝播する。private numeric name の再構築・canonical bytes の wire 流用・別 profile・candidate 由来 source context・供給 SAT clause は禁止する。
既存 `mizar_kernel::sat_encoding::SatEncodingContext::v1`/`mizar_kernel::sat_encoding::encode_formula_evidence` と `mizar_kernel::sat_checker::SatCheckContext::v1`/`mizar_kernel::sat_checker::check_sat_problem` だけを用い、内部 limits/replay は変えない。encoding error は内部 String で伝播し、is_unsat の場合だけ元 pair をそのまま返す。Sat/Rejected は None。一回までの query は KernelCheckResult/accepted status ではなく、独立した元 context による通常の fresh Kernel check と Proof policy を代替しない。
backend 関数一つの挿入と必要な module intro 表現だけを実装する。Rust 前に既存 public-function inventory と正確な fully-qualified SAT service/context 四経路だけを拡張する。元の禁止 token/assertion と他 inventory byte を保持し、generic runner/portfolio 本体は query-free、Kernel acceptance/witness/cache/proof-policy call の拒否を保つ。新 production type/helper/file/adapter/field/dependency/profile/hash/wire や下位編集は行わない。
source_candidate_sat_screen_* 三系列だけを追加し、元の実 source helper を再利用する。positive/false/single/aggregate/両 format の旧 factory pair と通常 fresh Kernel の独立 oracle・replay/入力保持、実 policy getter の None/zero/exact/各座標不足/MAX、設定/未設定の元 whole-input/profile 拒否を扱う。元の実 handoff に対する source_candidate_check_bytes と既存 corruption matrix/不変 fixture を使い、新 context producer や一回用 helper を作らない。
caller-owned Core/VC replay と Unsat は元 source/current-runtime 認証・受理・witness/reuse/cache 権限を与えない。代替列挙・新 instance 生成・advisory report・backend 実行・一般 finder search 制限・Proof scheduling・ATP30/31/Steps6–8 は未完了に残す。

## Source ground term correspondence

`source_registration_ground_term_correspondence(&CoreIr, &VcSet, VcId, LogicProfile) -> Result<BTreeMap<AtpTerm, mizar_kernel::clause::Term>, String>` は既存項の owned copy だけを返す。同一の immutable tuple から既存 source ATP problem と元 VC handoff を別々に再構築し、既存 whole-graph/default-discharge/profile gate を先に適用する。caller の Core/VC record とこの query は元 source の鮮度や証明権限を確立しない。
既存 source candidate factory と同じく、元 handoff の local Formula v1 を実 target/profile/tautology policy で parse し、lower error を伝播する。profile 変更・candidate 構築・v2/equality-name writer・checker・探索は呼ばない。限定 source route は観測 ground term 1個、閉じた generated builtin:set axiom 1個、その複製 type guard 1個を要求し、他の shape は部分 data なしで拒否する。
ATP axiom の唯一の引数は観測した nullary Function と一致させる。解決済み GeneratedVcFact provenance を既存 generated:N marker によって選択 VC の実 VcFormulaRef::Generated に結び、独立した ATP/Kernel ID を parse・同一視しない。producer_formula_ref と GeneratedVcFact source が一致する元 envelope entry を一意に求め、その実 formula_id で parsed entry を選ぶ。source ID/class・完全 formula fingerprint・provenance ID を比較し、元/parsed provenance の target・formula fingerprint・payload、および既存 handoff payload encoding による ATP anchor の一致を要求する。
対応 Kernel formula は arity 1 の BuiltinRelation atom、唯一の引数は元の nullary FunctorPredicate Application とする。既存 ATP 引数と実 Kernel 引数を組にし、owned return のためだけに clone する。type guard は axiom と同じ formula/解決済み ATP provenance source/payload を持ち、全観測項を過不足・矛盾なく対応させる。名前/hash・manifest 順・宣言・個数・候補 substitution から Kernel 数値 symbol を推測しない。
既存 producer/framing/parser error を先に伝播し、対応付け固有の拒否は内部 String `registration.atp.invalid_ground_term_correspondence` とする。public diagnostic・type/helper/alias/profile・wire/hash domain・dependency・lower-stage 変更は加えない。backend.rs に関数1個を挿入し、それ以外の旧 byte を保つ。既存 lint_policy.rs はこの関数と data-only Term/BuiltinRelation/Formula::Atom/FormulaSource::GeneratedVcFact の正確な path の inventory だけを更新し、Kernel import/検査/SAT/publication/reuse を含む trusted/downstream 禁止を全て保つ。
実 source test を2系列追加する。元/改名/single/全 aggregate/偽定義を両対応 profile で扱い、元 parsed goal/substitution 項を独立 oracle とした完全 pair・決定的 replay・実入力不変性を確認する系列、および既存 whole-input/profile/current-handle 拒否と実 Core/VC 不変性の系列とする。旧 helper/lower matrix を再利用し、架空 Accepted/Complete/context/evidence や単一用途の parser/producer helper を追加しない。
対応は既存の観測 ground choice に限定する。一般の関数/変数/量化子対応・Forall 除去・項/substitution/equality 生成・policy 計数/制限/SAT/探索・candidate 受理・runtime・witness/publication/reuse・全 ATP30/Steps6–8 は後続 owner に残す。

## stable hash と fingerprint

Task-14 source は backend metadata 用の private deterministic hash helper を実装しなければならない。
`std::hash`、`DefaultHasher`、raw `Debug` rendering、process id、temporary path、
wall-clock timestamp、completion order を semantic hash input にしてはならない。

helper は length-prefixed canonical field と明示的 domain tag を使う:

| Hash | Domain | 必須 field |
|---|---|---|
| `input_hash` | `mizar-atp/backend-input/v1` | `concrete_format`、logic-profile name/fragment、exact `input_text` bytes |
| `command_fingerprint` | `mizar-atp/backend-command/v1` | backend kind、profile id、concrete format、semantic executable id、ordered args、sorted allowlisted environment、working-directory policy kind、input-delivery mode、random seed、resource-limit record |
| `stdout_hash` / `stderr_hash` | `mizar-atp/backend-stream/v1` | stream kind とその pipe で観測された complete byte stream |
| `metadata_hash` field | `mizar-atp/backend-metadata/v1` | 必要に応じた sorted label、symbol binding、provenance bytes、target-binding fingerprint bytes |

semantic executable id は configured stable backend name または executable basename を含んでよいが、
machine-local absolute path を含めてはならない。explicit working directory は command fingerprint
内では policy kind で表現する。local directory path は diagnostic には現れてよいが semantic hash には
入らない。

## backend profile と command

`BackendProfile` は deterministic configuration record である:

```text
BackendProfile
  profile_id
  backend_kind
  concrete_format
  supported_observed_results
  candidate_evidence_formats
  version_probe
  default_args
  requires_candidate_evidence
  deterministic_priority
```

`BackendCommand` は shell string ではなく、単一 executable invocation である:

```text
BackendCommand
  executable
  args
  environment_policy
  working_directory_policy
  stdin_policy | problem_file_policy
  random_seed?
```

Task 14 は process-spawn API に argument を直接渡し、backend input や profile-provided command text
を解釈するために shell を起動してはならない。environment variable は allowlist され、sort され、
stable key で記録される。temporary directory は run ごとに private であり、process exit または kill
後に削除される。設定された absolute executable path は diagnostic metadata に記録してよいが、
machine-local path は semantic problem identity に参加してはならない。

## process model

runner は `BackendRunInput` ごとに 1 つの child process を起動する。

必要な挙動:

- `io_mode` に従って stdin または private temporary problem file で input を渡す。
- stdin mode では byte-exact input を private spool に stage し、path を backend に渡さず
  read handle を fd 0 として接続する。これにより input delivery は blocking writer
  thread に依存しない。
- stdout と stderr を configured byte limit まで capture する。
- 完全に capture した stdout/stderr を hash するか、capture limit により stream が truncated されたことを記録する。
- 利用可能なら exit code または platform signal / termination detail を記録する。
- start/end monotonic timing、elapsed time、timeout budget、kill grace duration を記録する。
- host platform が提供する CPU、wall-clock、memory、process-count、stdout/stderr、temporary-file limit を best-effort で適用する。
- timeout、cancellation、portfolio stop 時に child process を終了する。
- termination 後に process を wait/reap し、task-14 test が child process が残っていないことを assert できるようにする。
- spawn failure、version-probe failure、timeout、cancellation、non-zero crash、malformed output を verifier panic なしで分類する。

resource-limit mechanism は platform-dependent である。現在 platform で limit を enforce できない
場合、runner は unsupported-limit diagnostic を記録する。後続 policy は unsupported enforcement を
error にしてよいが、backend runner 自身は limit が利用できなかったことを理由に proof status を
fabricate してはならない。

Task-14 source は unsupported limit を `best_effort` または `required` として表現しなければならない。
unsupported best-effort limit は diagnostic である。unsupported required limit は、`Proved` result を
構築可能にする前に run を `Error` として分類する。

private problem-file mode では、runner は run ごとの private directory を race-resistant に作成し
（`create_new` / exclusive creation semantics）、preexisting path を再利用せずに problem file を書き、
platform が対応する場合は best-effort restrictive permission を適用し、normal exit、timeout、
cancellation、crash、spawn failure の後に file と directory を削除する。privacy または cleanup を
enforce できない場合は diagnostic を記録する。privacy failure は file-mode execution では `Error` である。

stdout/stderr reader は pipe backpressure deadlock を避けなければならない。configured retained-byte
limit に達した後も reader は EOF まで pipe を drain し続け、prefix だけを保持し truncation flag を
記録する。stream hash は retained prefix だけでなく complete observed stream に対して計算する。process
kill または pipe failure により stream を完全に drain できない場合、result は incomplete-stream diagnostic
を記録し、`Proved` に分類してはならない。

stdin mode は、backend descendant が fd 0 を open のまま保持することで verifier 側 writer thread が
blocked のまま残る構造を導入してはならない。private stdin staging を作成または読み取り用に open
できない場合、run は process spawn 前に `Error` となる。

## backend identity と reproducibility

runner は次を記録する:

- backend kind と profile id。
- executable identity と command fingerprint。
- version probe command、version stdout/stderr hash、可能な場合 parsed version。
- selected concrete format と encoded input hash。
- normalized argument、sort 済み allowlisted environment、working-directory policy、
  input-delivery mode、random seed、resource limit。
- exit status、termination class、stdout/stderr hash、timing、resource observation。

`command_fingerprint` は deterministic であり、process id、temporary path、wall-clock timestamp、
raw completion order、machine-local absolute path を除外する。ただし後続 config spec が
path-sensitive reproducibility を明示的に有効化した場合はその限りではない。diagnostic rendering は
有用な場合 local path を含めてよいが、semantic hash と proof-reuse identity はそれに依存してはならない。

## result classification

backend runner は process status、observed backend result、candidate evidence availability を分離する。

```text
BackendRunStatus
  Proved
  Counterexample
  Timeout
  Unknown
  Error
  Cancelled
```

`Proved` は candidate-evidence status だけを意味する。kernel acceptance ではなく、artifact proof status
へ直接 project してはならない。

`Proved` はすべての条件を満たす場合だけ構築できる:

1. process が timeout、cancellation、crash、または parsing を無効にする capture-limit corruption なしで完了した。
2. backend output が `encoded_problem.expected_result` に一致する observed result として parse された。
3. 現在の interface では、これは `ExpectedBackendResult::Unsat` に対応する unsatisfiable / refutation / theorem result を意味する。
4. supported candidate format の formula/substitution evidence が存在する。
5. candidate evidence の target binding、input hash、formula label または assertion label、symbol binding、provenance hash が encoded problem metadata と一致する。
6. backend proof method、backend log、unsat core、SMT proof object、resolution trace、TSTP trace、backend-reported `used_axioms` を trusted acceptance material として使っていない。

observed result が expected result に一致しても、supported candidate formula/substitution evidence が
存在しない場合、result は missing-evidence diagnostic 付きの `Unknown` または `Error` であり、
`Proved` ではない。backend が `sat`、`counter-satisfiable`、model data、counterexample を報告した場合、
provenance mapping が成功するときだけ `Counterexample` にしてよい。それ以外は `Unknown` または
`Error` である。`unknown`、timeout、cancellation、malformed output、parse failure、
unsupported observed status、polarity mismatch は決して `Proved` ではない。

Task 14 はこれらの invariant check と mock classification を公開してよい。task 15 は paired
extraction spec と guarded backend route が存在するまで、最初の real backend extractor を deferred
として記録する。task 16 もその route が存在するまで deferred のままであり、その後に
real-backend-style output に対する full outcome と polarity classification fixture を追加してよい。

## candidate evidence boundary

Candidate evidence record は untrusted extraction output である:

```text
BackendCandidateEvidence
  candidate_id
  schema_family
  payload_ref_or_bytes
  target_binding
  encoded_problem_hash
  provenance_hash
  formula_label_refs
  symbol_binding_refs
  extraction_diagnostics
```

payload は kernel-owned schema と互換な formula/substitution evidence でなければならず、task-14 mock
classification は明示的な payload bytes または明示的な payload reference を要求しなければならない。
label / symbol / provenance metadata だけでは candidate evidence ではない。backend proof object と log は
将来の extractor の diagnostic input になってよいが、後続 kernel checking に渡す candidate payload は
backend proof method、SMT proof object、unsat core、TSTP trace、resolution trace、backend log、
backend-reported `used_axioms`、legacy certificate であってはならない。kernel acceptance、trusted
`used_axioms`、proof witness draft、artifact status、cache promotion は downstream task/crate に属する。

Task 29 は post-audit F1/F2/F6 kernel contract を backend classification の上流に保つ。
backend classifier は candidate target binding、encoded-problem hash、provenance hash、
formula label、symbol binding を比較してよいが、backend output から goal polarity、
context-identity payload、imported-statement projection を発明または推測してはならない。
kernel contract が要求する translator/VC-handoff evidence を欠く candidate は、real extractor が
explicit formula/substitution payload を生成できるまで non-candidate のままである。

## failure semantics

- `Timeout`: timeout budget が経過し、child process が終了されたか、実行中でないことが確認された。VC は open のままか他 candidate へ進む。proof status は accepted されない。
- `Cancelled`: scheduler、portfolio、または user cancellation が run を停止した。proof status は accepted されない。
- `Error`: spawn failure、missing executable、permission failure、required resource limit が unsupported、crash、parsing が required な場合の non-UTF / parse failure、capture-limit corruption、temporary-file failure、malformed backend output。
- `Unknown`: backend は完了したが unknown/unsupported status を報告した、または hard process error なしに evidence が不十分だった。
- `Counterexample`: provenance を通じて対応付けられた diagnostic-only model/counterexample data。proof acceptance ではない。

すべての status は origin の `VcId` / problem identity に紐付き、deterministic diagnostic を生成する。
backend failure は unrelated VC を crash させたり既存 proof status を mutate したりしてはならない。

## determinism

同等の backend run input と同等の mock process behavior は、non-semantic timing を normalize するか
別途 non-semantic と mark した後、byte-identical deterministic run metadata を生成しなければならない。
determinism は次を含む:

- command fingerprint と profile id。
- input hash と concrete format。
- resource-limit record。
- stdout/stderr hash と truncation flag。
- exit status と termination class。
- result classification。
- candidate evidence metadata ordering。
- diagnostic key と ordering。

raw completion order、process id、temporary path、wall-clock timestamp、backend scheduling race、
host-specific absolute path は canonical candidate ordering や proof status を決定しない。

task 19 は artifact と reproducibility note のための stable run-metadata projection を追加する。
この projection は次を記録する:

- run id、problem id、backend kind、profile id、concrete format、encoded input hash、
  encoded metadata hash、command fingerprint。
- semantic executable id、正規化済み argument、sort 済み allowlisted environment、
  working-directory policy kind、input-delivery mode、random seed、timeout setting、
  capture limit、platform resource-limit record。
- version probe が設定されている場合の version-probe success、parsed version、version
  stdout/stderr hash、version diagnostic。
- terminal status、observed result、termination class、exit status、child-reaping flag、
  elapsed time、stdout/stderr hash、stream total byte count、truncation / incomplete flag、
  backend diagnostic。

この projection は reproducibility metadata だけである。candidate evidence、kernel
evidence、proof policy、artifact winner selection、witness publication、cache promotion ではない。
elapsed time、backend version output、stdout/stderr byte、diagnostic などの runtime observation は、
diagnostic-only であり、downstream candidate hash、trusted-acceptance hash、proof-acceptance
material の外に残さなければならない。

## public enum forward compatibility

task 22 は frontend task 25 の方針を `backend` module に適用する。この module が所有する
public enum は downstream crate 向けに `#[non_exhaustive]` とする:
`BackendWorkingDirectoryPolicy`、`BackendIoMode`、`BackendLimitRequirement`、
`BackendRunStatus`、`BackendObservedResult`、`BackendTermination`、
`BackendCandidatePayload`、`BackendConfigError`。

Public enum inventory: `BackendWorkingDirectoryPolicy`, `BackendIoMode`, `BackendLimitRequirement`, `BackendRunStatus`, `BackendObservedResult`, `BackendTermination`, `BackendCandidatePayload`, `BackendConfigError`.

将来の runner policy、status、observed result、payload class、error variant は、source が使う前に
仕様化しなければならない。`mizar-atp` 内部では、process execution、candidate evidence
classification、result metadata、proof status に影響する match は、paired spec が意図的 fallback を
記録しない限り、明示的に保ち fail closed しなければならない。新しい payload class は、paired
extraction spec が formula/substitution evidence へ mapping するまで diagnostic または untrusted のままである。

## gap classification

- resolved `deferred` spec gap: task 13 は source が存在する前に backend runner と result
  classification contract を定義する。
- resolved `source_drift`: task 14 は generic process runner、mock classification seam、
  deterministic run metadata、private input handling、drain-safe capture、fail-closed process
  status を実装する。
- `external_dependency_gap` / `deferred`: real backend output を kernel が parse できる
  formula/substitution candidate bytes/ref に変換する paired `mizar-atp` evidence-extraction
  spec または source module がなく、supported architecture-10 backend executable も
  verification environment に存在しないため、task 15 はまだ first concrete backend adapter を
  追加できない。
- `external_dependency_gap` / `deferred`: task 16 full real-output result classification と
  polarity fixture は task-15 extraction route に依存する。その route が存在するまで、task 14 の
  mock classification invariant が唯一の実装済み classifier surface である。
- `external_dependency_gap`: proof policy、winner selection、proof witness publication、cache
  promotion、artifact projection、backend availability は task 13 の外にある。

## Task-14 Test Expectations

Task 14 は次の focused Rust coverage を追加しなければならない:

- stdin mode と private problem-file mode による mock backend process invocation。
- 両 mode で `encoded_problem.input_text` が byte-exact に渡されること。shell
  metacharacter、proof command、unsat-core request、backend directive に見える byte も inert な
  problem data のままであり、runner が rewrite、normalize、append、interpret しないこと。
- shell interpretation を使わない direct executable/argument spawning。
- process id、temp path、timestamp、raw completion order、machine-local absolute executable /
  working-directory path を除外する deterministic command fingerprint。shuffled
  environment-policy input は sorted allowlist で記録され、同じ semantic fingerprint を生成すること。
- version probe の success/failure recording と stdout/stderr hash。
- timeout、cancellation、kill-grace、crash、non-zero exit、missing executable、spawn-permission fixture。
- stdout/stderr capture hashing、truncation flag、limit exceeded diagnostic。
- private temporary directory cleanup と、timeout/cancellation/crash 後に child process が残らないこと。
- resource-limit metadata recording。unsupported-limit diagnostic を含む。
- expected-result polarity mismatch、candidate formula/substitution evidence 不在、candidate の
  target binding、input hash、label、symbol、provenance の mismatch、または timeout /
  cancellation / crash / parsing を無効にする capture-limit corruption 後に otherwise matching
  candidate が到着した場合、`Proved` constructor/classification が reject されること。
- observed result が `ExpectedBackendResult::Unsat` と一致し、candidate formula/substitution
  evidence metadata が target binding、input hash、label、symbol、provenance と一致する場合だけ
  mock `Proved` classification になること。
- counterexample、unknown、timeout、cancelled、error status が accepted proof status を生成しないこと。
- kernel/SAT checking、proof policy、witness/cache publication、backend proof method trust、
  resolution-trace trust、unsat-core trust、SMT proof-object trust、trusted backend `used_axioms` が
  存在しないこと。
