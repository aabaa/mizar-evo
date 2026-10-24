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
元 handoff の local Formula v1 を実 target と kernel profile/tautology policy で parse し、元 formula/provenance data を得る。[限定構造抽出](#bounded-structured-source-substitutions)で置換を導出して書き、新 handoff の v1 を parse して closed-choice name writer に渡し、新 handoff の v2 を frame 化する。parse は構造検査であり受理ではない。元 source profile の完全な wire・canonical/context identity・formula・provenance・refutation goal を保ち、元 substitution record は新 recipe や最終 wire の入力にしない。
既存 FormulaSubstitutionBytes candidate に正確な v2 byte と fresh encoded container の完全な target binding・input hash・provenance hash・全 label・全 symbol mapping を入れる。別の candidate byte・hash・identity・encoded output は入力しない。未対応 source/encoding/framing/parsing/name は部分結果なしで中断する。
candidate_id は既存 private StableHasher の domain `mizar-atp/source-registration-candidate/v1` から、byte field `encoded-input-hash`、`encoded-metadata-hash`、u64 field `target-fingerprint-algorithm`、byte field `formula-substitution-bytes` の順で作る。実 container hash・target algorithm ID・正確な payload byte を使い、既存小文字 hex と prefix `source-registration-v1:` で表す。metadata は target digest/producer・label・mapping・source provenance を結び、旧 hash を変えず algorithm を明示的に加える。input hash は実 text/format/profile、payload は通常の evidence field を結ぶ。Debug・runtime data・report・caller receipt を hash に入れない。
pair は未検査の試行であり、fresh Kernel check が拒否する実際の偽の source も含む。backend result や探索成功ではない。caller が現 source/context 認証と独立した通常 [Kernel 検査](../../mizar-kernel/ja/checker.md)、[Proof](../../mizar-proof/ja/selection.md) が受理を所有する。既存 original-Open built-in facade は変えない。探索上限・汎用 finder・advisory 抽出・backend 実行・publication・reuse は保留する。
異なる実 source 回帰 caller 3箇所で direct concrete encoding/metadata、独立した通常 wire/fresh Kernel outcome、source/context 全体の破損を検査する。両 format、元・改名した正/負の single/aggregate source、決定的 replay/固定 identity、元 Open・選択した Discharged の拒否、破損 output の未信頼性を扱う。既存 manual/producer oracle を維持する。

## Bounded structured source substitutions

既存 source candidate factory の公開 signature を保ち、実 source translator の ATP tree、元 parsed Kernel formula、[完全な ground 対応](#source-ground-term-correspondence)を使う。既存 whole-source/default-discharge/profile/encoding gate と error を先に適用する。Core/VC は caller-owned のままとし、新 public operation・recipe type・field・dependency・下位 API・鮮度/受理 token は導入しない。
各 quantified axiom の実 GeneratedVcFact provenance marker を選択 VC の実 VcFormulaRef に解決する。その producer ref と generated source class に一致する元 envelope entry を一意に求め、元 formula_id で parsed formula を選ぶ。ground join と同じく完全な source identity/class・fingerprint・provenance ID・target・payload と ATP provenance anchor の一致を検査する。独立 row ID・manifest 順・綴り・hash から Kernel 数値 identity を作らない。実 quantified premise 全体を過不足・重複なく対応させる。
Forall は sort のない binder 一つだけを許す。body を対応する Kernel Atom/Not/And/Or tree と整列し、Boolean constructor・子の順序・個数を保持する。ATP Equality は Kernel Equality arity 2、builtin:set は BuiltinRelation arity 1、source attribute atom は実 Predicate kind/arity に対応する。数値 predicate identity は元 formula に属する。Kernel formula に Forall node はない。nested quantifier・True/False/Implies/Exists・未対応 term/formula shape は拒否する。
整列した term path で ATP binder 変数の各出現を実 Kernel VariableId に結ぶ。少なくとも一出現を要求し、body と source profile 全体で一貫した formal 対応を要求する。別 ATP 変数や Kernel ID の矛盾は拒否する。非変数 ground 出現は既存完全対応 map に存在し、その実 Kernel term と等しくなければならない。観測した nullary choice だけを扱い、一般関数・推測 symbol・manifest 個数では対応を確立しない。
実 ground type fact の後の VC premise 順に進み、実 source ref で導出済み quantified record を結ぶ。検査した零始まり substitution ID をその順に付け、ATP row index は使わない。既存 KernelSubstitutionPayload に実 ref・対応 VariableId・観測 nullary FunctorPredicate term・解決した元 **formula** provenance payload を入れる。元 parsed/envelope substitution は recipe oracle にしない。
既存 [binder/payload grammar](../../mizar-kernel/ja/substitution_checker.md#binder-evidence-model) に限定する。schema 1・frame 無し・実 ID の free variable 一つ・schematic variable 無しとし、payload は実 substitution ID・kind 1・空 root path・role 1 の formal-to-choice replacement 一つ・空 side condition を持つ。nullary term の限定 wire（Application/FunctorPredicate tag・実 u32 symbol ID・引数0）を inline で書き、canonical term byte を使わない。汎用 codec・新 schema/hash domain・Kernel policy は作らない。
元 formula evidence getter と解決 provenance から既存 KernelFormulaPayload/Projection を再構築し、別の final-goal projection も含める。同じ実 VcSet/VcId・profile・polarity・manifest と、空 imported payload・formula_context None・discharge_output None で既存 build_kernel_evidence_handoff を呼ぶ。独立導出した substitution vector だけを渡し、VC set や source context は再構成しない。
新 canonical/context identity の整合性と元 source profile との一致を要求する。不一致は実 error とし、旧 candidate fallback や expected hash/context の書き換えはしない。**返された新 handoff** の v1 を parse し、新 parsed substitution から既存 reflexivity name を導出して新 v2 を frame 化する。旧完全 v1/v2 wire・candidate ID/hash tail・encoded metadata・source outcome を保つ。構造固有の拒否は内部 String `registration.atp.invalid_structured_substitution` とし、下位 producer/parser/writer error と public diagnostic は維持する。
private な基本 formula/term walker は実 recursive Boolean/argument path で三つ以上の実 call site がある場合だけ許し、inline work stack も選べる。一回用 facade・別 AST・test observer・writer 注入・実装を写した lint は追加しない。走査に必要な正確な pure Kernel data-path allowance だけを広げ、public inventory と trusted/downstream 禁止を全て保つ。
既存 mizar-test runner の実 source helper と source_candidate_check_bytes を再利用し、旧 test body/oracle を保つ。旧 factory の完全 wire/container/canonical/context 互換性、両 format、改名/single/aggregate/false source、新 writer と通常 fresh Kernel Accepted/Rejected outcome、下位拒否、入力不変性を扱う。Proof prebuilt の None/zero と SAT screening の None/zero/exact/MAX count を保ち、十分な既存 matrix は複製しない。private pure-walker relation test は既存未信頼 ATP/Kernel 型で出現の一致/矛盾・未対応 shape・ground mapping を扱えるが、架空 Core・source capture・accepted result は作らない。
旧公開出力の完全一致により、独立抽出と検査後の旧置換 copy は観測上等しくなる。runtime byte 一致だけで内部由来は証明できない。独立実装/dataflow review で、抽出 vector が既存 builder に入り、返された新 handoff が name と wire の両方に使われ、元 substitution の read/copy や fallback がそれらを供給しないことを確認する。test は観測可能な互換性と walker relation を扱い、test-only 観測 seam を作らない。
これは実際に偽の source の未検査候補も返す、限定 ATP30 前提である。通常 fresh Proof/Core/VC/context/policy/Kernel check、metadata/selection/witness projection、source SAT screening は維持する。全 ATP30/31、一般抽出、代替列挙/探索/枯渇、backend/advisory 統合、finder policy 適用、runtime/footprint、受理、witness/publication/reuse/cache、Steps 6–8 は未完了に残す。

## Source candidate SAT screening
`screen_source_registration_candidate(&CoreIr, &VcSet, VcId, LogicProfile, ConcreteFormat, Option<[u64; 4]>) -> Result<Option<(EncodedBackendProblem, BackendCandidateEvidence)>, String>` は既存 source factory の一候補だけを検査し、screening 自体では backend 実行・項/置換生成・証明受理を行わない。
既存 source candidate factory、その後 source ground-term correspondence を同じ immutable tuple で呼び、元の whole-input/profile/encoding 拒否を limits より先に保持する。optional setting は caller（既存 Proof policy getter など）から渡し、新依存や default を作らない。None は未設定で None を返し、literal zero/MAX を保持する。既存 Proof facade、policy fingerprint、prebuilt candidate の挙動は変えない。
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

## Local-variable reflexivity names

`encode_local_variable_reflexivity_names(&mizar_kernel::formula_evidence::ParsedKernelEvidence) -> Result<Vec<Vec<u8>>, String>` は既存 backend module の決定的な untrusted name encoder とする。
schema 1、空の substitutions、`AssertFalseForRefutation`、plain final-goal Equality atom の両引数が同じ実際の `Term::Variable` である場合だけ扱う。goal を直接読み、通常 parse 済みの manifest/arity/term 検証を重複せず利用する。他の context/formula entry の消費は要求しない。
他の profile はすべて `Err("unsupported local-variable reflexivity input")` とし、partial names や空の成功へ fallback しない。
既存 Formula-v2 schema-1 name を一つ返す。順に big-endian u32 instance ID 1、u8 reflexivity tag 1、実際の equality-symbol u32 ID、u8 Variable tag 1、実際の variable u32 ID（14 bytes）。source/hash/role 番号から ID を推定せず、変数を ground constant にしない。
既存 VC `encode_local_evidence(Some(&names))` が v2 を frame し、Kernel だけが instance を構成・検査する。encoder は探索、SAT query、backend 実行、result 分類、policy、受理、公開の authority を持たない。
既存 closed-choice encoder の全 byte と動作を保ち、substitution 不在の `Ok(empty)` と全拒否も変えない。source Proof consumer、汎用 finder/探索/limit enforcement、Steps 6–8 は別作業とする。
既存 private runner `term_proof_tests` の旧 prefix を保って二系列を追記する。真正な元/alpha-renamed Checker/Core/generic-VC/theorem-handoff source から通常 v1 parse、正確で決定的かつ pure な names、既存 v2 writer を経て元 target/context への fresh normal Kernel run を行う。実際の Accepted、ProofObligation、policy taint/imported used axioms 不在、既存 default policy/limits、names 不在 v1 の拒否、foreign/modified original binding の fail closed、source status 維持を要求する。
補助系列では通常 parse 済み untrusted input の実際の非零 ID と、非対応 schema/polarity/substitutions/non-equality/Boolean shape/異なる変数/ground term を扱う。encoder 入力だけであり、偽 source/backend producer や Accepted control にしない。既存 parser の malformed-manifest/arity/budget coverage を再利用する。

## Explicit local-variable transitivity names

`encode_local_variable_transitivity_names(parsed: &mizar_kernel::formula_evidence::ParsedKernelEvidence, middle: mizar_kernel::clause::VariableId) -> Result<Vec<Vec<u8>>, String>` を既存 reflexivity encoder の隣に inline の untrusted encoder として追加する。
schema1、空 substitutions、AssertFalseForRefutation、引数がちょうど二つの Variable left/right である plain final Equality atom だけを扱う。middle は呼出側が明示する。実際の plain premise Equality(left,middle) と Equality(middle,right) を要求し、各引数は二つの Variable、symbol は goal と完全一致させる。operand 順序を保ち、無関係の formula を創作・反転・書換えせず無視する。parse 済み manifest/term 検証を再利用する。
既存 v2 section7 schema1 name を一つ返す。BEu32 instance1、u8 transitivity tag3、実際の equality-symbol BEu32 ID、left,middle,right それぞれの Variable tag1/BEu32 ID の順（24 bytes）。他は Err("unsupported local-variable transitivity input") とし、partial names や空成功へ fallback しない。source 名・role 番号・hash・ground-term 代理から ID を作らない。
明示された一つの chain の検証であり、middle の選択や代替の列挙をしない。finder 探索と policy counter は別作業とし、free Kernel variable を ATP ground term にしない。既存 parser/SAT resource limit は下位 owner が保持し、verifier policy と旧 encoder を変更しない。
VC の encode_local_evidence(Some(&names)) だけが framing を行い、Kernel だけが法則を導出して元 target/context を検査する。type/field/helper/adapter/codec/hash/dependency、production Kernel/SAT call、source-check seal、backend 実行、status/受理、witness/公開/reuse/Hit、汎用 finder/Steps6–8 の達成を追加しない。source freshness と policy consumer は Proof の別作業とする。

### Explicit transitivity name tests
既存 runner/tests.rs に source_algorithm_state_transitivity_names_reach_normal_kernel と source_algorithm_state_transitivity_names_reject_untrusted_profiles だけを追記する。既存 step5c14 source/Core/VC helper と全旧 byte を保つ。ATP public-API lint inventory には新関数の正確な一項目だけを加え、trust exclusion を保つ。
assertion を除いた元 var/const/assignment source、真正 rename、独立 constant の各 variant を用いる。committed state handoff と新 encoder の前に、旧 source check/Core/write version/raw state equality/goal の実関係を確認する。Rust の前に既存 Kernel grammar から独立した literal24byte name を固定する。決定性と全入力の不変を比較し、既存 v2 writer から元 target/context への fresh normal ProofObligation check を行う。実際の untainted Accepted/imported used axioms 不在を要求し、names 不在 v1、coherent な誤 middle name、真正 foreign target/context は拒否させる。raw Open/Deferred/incomplete と default policy/limit を保つ。
補助 parse 済み untrusted data では非零 symbol/variable ID、片方の chain leg の欠落・反転、異なる equality symbol、誤・未知 middle、non-equality/Boolean/ground/arity goal、substitutions、schema2、誤 polarity を扱う。encoder 入力だけであり source producer や Accepted control にしない。malformed manifest/term/budget は既存 parser coverage を再利用する。

## Bounded local-variable transitivity selection

`select_local_variable_transitivity_names(parsed: &mizar_kernel::formula_evidence::ParsedKernelEvidence, max_equality_candidates: Option<u64>) -> Option<Vec<Vec<u8>>>` を不変の明示 encoder の隣に一つ inline で加える。parser が厳密な数値昇順・一意を検証済みの実 variable manifest をそのまま走査し、各 middle を encode_local_variable_transitivity_names で試し、最初の成功 vector をそのまま返す。source 綴り/role番号/fingerprint から middle を推測せず、leg反転、term生成、instance-ID1 name の結合をしない。
明示上限は既存 finder limits の named-equality-candidate 座標であり ground-term/SAT 上限ではない。chain 不成立も含め、訪れた manifest middle ごとの (left,middle,right) 候補を一回と数え、literal u64 上限を超える試行の前に止める。None は未設定で None、Some(0)/上限到達/chain無し/非対応 profile も None とし、部分 name や数値 default を返さない。u64::MAX を狭い型に変換しない。旧明示 encoder の byte/error と下位 validation を全て保つ。
有限の候補選択であり SAT/Kernel/backend 実行、一般 substitution search、Proof policy/acceptance を行わない。VC の既存 writer だけが name を frame し、Kernel だけが元 target/context に対して law を生成・検査する。変数を ATP ground term にしない。schema/hash/type/field/helper/adapter/dependency、current-source seal、witness/publication/reuse/cache、ATP30/Steps6–8 全体の完了を加えない。
既存 runner/tests.rs に source_algorithm_state_transitivity_selection_reaches_normal_kernel と source_algorithm_state_transitivity_selection_respects_order_and_limits の二 family だけを追記し helper を保つ。selector Rust 前に元/rename/独立 constant の実 Core/state/goal と literal name を旧明示 encoder、通常 v2 writer、fresh Kernel で基準検証する。その後、自動 name、未設定/zero/不足/一致/MAX 上限、決定性、全入力不変を比較し、元 target/context の fresh normal check で実 Accepted と names 無し v1 の拒否、raw Open/incomplete accounting/default policy 保持を要求する。補助の通常 parse 済み untrusted data は通常の昇順 manifest、位置を変えた formula、二つの実 directed-chain middle で最小 ID 選択、失敗試行の計数、chain無し/非対応の None を確認し、source/Accepted 権限を持たせない。旧 encoder/parser/foreign-context matrix を再利用し、public-function inventory は正確な新関数一項目だけを加える。

## Source theorem candidate inputs

build_source_theorem_candidate_inputs(core: &CoreIr, original: &VcSet, vc_id: VcId, discharge: &DischargeOutput, policy: &DischargePolicy, logic_profile: LogicProfile, concrete_format: ConcreteFormat) -> Result<(EncodedBackendProblem, BackendCandidateEvidence), String> を旧 source-registration factory の完全な doccomment 前に一つ追加する。実際の非信頼 ATP 候補を生成し、過去の Builtin result の付け替えはしない。
最初に [scoped source translation](./translator.md#source-theorem-scoped-sequent)、次に明示 format で既存 EncodedBackendProblem::encode を使い、error と caller profile を保つ。同じ元 graph/全 discharge/明示 policy から [processed source handoff](../../mizar-vc/ja/kernel_evidence_handoff.md#source-theorem-configured-discharge-handoff) を独立に再生成し、その error を保つ。schema/policy/format の default や fallback は加えない。
encoded/problem target binding は、新鮮な handoff の完全 target fingerprint と mizar-vc-kernel-evidence-handoff:<hex canonical hash> 生成 binding に一致させる。実 guard の proof-let CoreVar を読み、唯一の ATP Forall binder の GeneratedBinder(proof-let:<実 index>) symbol row と対応付ける。全 source profile は既存 owner が検証するため複製せず、opaque provenance から式を復元しない。

新鮮な target/kernel profile から normal v1 parse context を作り、対応 tautology policy を明示変換する。その handoff の実 v1 bytes を parse し、唯一の実 manifest variable が実 local guard と final equality に共有され、実 local-hypothesis/context・新鮮な producer refs と対応することを要求する。variable/symbol ID を推測せず、元 quantified variable を使わない。
実 parsed evidence に既存 encode_local_variable_reflexivity_names を適用し、同じ handoff の encode_local_evidence(Some(names)) で実 v2 bytes を作る。元 local Kernel formula を閉じた ATP conjecture で置換しない。parser error は Debug、他の下位 error は String/Display を保ち、新関連付け拒否は内部 theorem.candidate.invalid_projection とする。
既存 FormulaSubstitutionBytes candidate に実 v2 bytes と encoded target/input hash/provenance hash/formula labels/symbol bindings を渡し、encoded problem と返す。厳密な opaque ID は source-theorem-candidate-v1:<encoded.metadata_hash() bytes の小文字 hex>。固定手続きの命名であり、実 encoded metadata が problem/profile/format/canonical handoff を束縛し、手続きがそこから wire を決定的に作る。candidate-wire digest、生成元 seal、canonical VC fingerprint、再利用/currentness key ではない。consumer は全 field 比較と独立 wire 検査を要する。

真正な ATP 所有の合成として実 scoped problem と primitive 対応を消費してから新 evidence を作る。Builtin Kernel result の分類変更や結果からの Proof origin 推論はしない。旧 registration/Builtin API/status を保ち、後段 Proof が FormulaSubstitution scheduling と新鮮な検査を担う。本体に Kernel/SAT/Proof policy/backend process/Proved observation/受理/witness/公開/cache/service/一般 finder/search-limit enforcement は加えない。
既存 runner/proof_verification.rs private term_proof_tests に真正 source 二 family を追記する。新 API 前に実 scoped problem/native encoding/processed handoff/v1/ATP43 names/v2 baseline を独立構築し、encoded 全体と candidate 全 field・厳密 ID/wire、source/target/context/variable 対応、反復・入力不変、実 original/owner/alpha/trivia/SourceId/snapshot/schema/policy variant と両対応 format を検査する。format 別 ID が異なっても実 local evidence bytes の一致は許す。
実返却 candidate を独立再構築した processed target/profile/context で parse し、実 candidate/container 全比較から encoded_problem_matches を導く。既存 FormulaSubstitution scheduling 後に新鮮な normal Kernel check と AtpFormulaSubstitution policy evaluation を使う。実結果だけを要求し、現在の reflexivity の Accepted/KernelVerified は test の consumer 検証であり本体の受理や source activation ではない。旧 Open/Builtin result は異なる対象・origin なので期待値に付け替えない。
因果的拒否は実 source translation error を format/後段 error より先に保つこと、真正 foreign output の全不一致、absent/unsupported/cross-original/discharge/profile error を検査する。異なる policy の真正な等出力は値の関連付けとして許す。下位 profile/framing/metadata matrix を再利用し、injected-pair seam や架空 source/Proved/Accepted control は加えない。
