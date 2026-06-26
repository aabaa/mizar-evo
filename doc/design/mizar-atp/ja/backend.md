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

task-14 source は、ここで記述する generic process runner と mock backend fixture を実装してよい。
real backend adapter、backend-specific output parser、candidate evidence extraction、
backend-available integration test は task 15 に残る。広い classification fixture と polarity
validation は task 16 に残る。ただし task 14 は invalid な `Proved` result を表現不能にする
constructor と invariant を公開してよい。

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

Task 14 はこれらの invariant check と mock classification を公開してよい。task 15 は最初の real backend
extractor を追加する。task 16 は real-backend-style output に対する full outcome と polarity
classification fixture を追加する。

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

payload は kernel-owned schema と互換な formula/substitution evidence でなければならない。backend proof
object と log は extractor の diagnostic input になってよいが、後続 kernel checking に渡す candidate
payload は backend proof method、SMT proof object、unsat core、resolution trace、legacy certificate
であってはならない。kernel acceptance、trusted `used_axioms`、proof witness draft、artifact status、
cache promotion は downstream task/crate に属する。

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

## gap classification

- resolved `deferred` spec gap: task 13 は source が存在する前に backend runner と result
  classification contract を定義する。
- `deferred`: task 14 generic process runner source、mock backend fixture、
  platform-specific resource enforcement、no-zombie test。
- `deferred`: task 15 first concrete backend adapter と formula/substitution candidate extraction。
- `deferred`: task 16 full real-output result classification と polarity fixture。
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
