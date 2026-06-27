# Module: backend

> 正本は英語です。英語版:
> [../en/backend.md](../en/backend.md)。

## 目的

`backend` module は、すでに encode された 1 つの `AtpProblem` に対して、1 つの外部 ATP/SMT
backend を呼び出す方法を仕様化する。child-process execution、resource limit、captured-output
metadata、backend version recording、timeout/cancellation の graceful handling、policy-neutral な
backend-result classification を所有する。

backend module は引き続き evidence producer boundary である。proof acceptance、
`mizar-kernel` 呼び出し、SAT checking、witness publication、portfolio winner selection、
cache update、backend proof method、log、unsat core、SMT proof object、TSTP trace、
resolution trace を trusted acceptance material に変えることは行わない。

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
