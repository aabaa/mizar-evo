# Module: portfolio

> 正本は英語です。英語版:
> [../en/portfolio.md](../en/portfolio.md)。

## 目的

`portfolio` module は、すでに `VcStatus::NeedsAtp` に到達した 1 つの VC に対する
phase-13 ATP portfolio planning と candidate collection を仕様化する。

この module は evidence producer と handoff boundary である。backend run の計画、
backend result の収集、candidate ordering の正規化、再現性 metadata の記録、後続の
kernel / proof-policy stage への candidate evidence handoff を行ってよい。ただし、
proof を受理したり、artifact-facing winner を選択したり、`mizar-kernel` を呼び出したり、
witness を publish したり、cache を更新したり、backend proof method、log、unsat core、
SMT proof object、TSTP trace、resolution trace、completion order、instantiated formula を
trusted acceptance material に変えたりしてはならない。

## 範囲

task 17 は specification-only である。この仕様が存在した後、将来の `src/portfolio.rs`
module が policy-neutral な portfolio planning と candidate collection を実装することを
許可する。Rust source、新しい backend adapter の spawn、real backend output の parse、
formula/substitution evidence の発明、kernel 呼び出し、proof policy 評価、artifact witness
publication、proof-cache promotion は追加しない。

task 18 は、early stop しない collection path を実装してよい。early-stop 機構は、stable な
外部 proof-policy finality contract が存在した後に限って実装してよい。それまでは no early stop
だけが source implementation path である。ただし `mizar-proof` policy を local に実装してはならない。
現在 `mizar-proof` は workspace crate ではないため、policy evaluation、witness publication、
proof cache promotion は `external_dependency_gap` のままである。

## 入力と出力

概念上の portfolio API は次を消費する:

```text
PortfolioInput
  portfolio_id
  vc_id
  vc_hash
  atp_problem
  backend_profiles
  encoded_problem_set
  obligation_budget
  scheduler_budget
  proof_hint?
  build_snapshot
  policy_constraints
  cancellation
```

そして次を生成する:

```text
PortfolioEvidenceSet
  portfolio_id
  vc_id
  vc_hash
  plan_hash
  backend_results
  candidates
  pending_capabilities
  stop_summary
  diagnostics
  metadata
```

`PortfolioEvidenceSet` は accepted proof result ではない。後続の kernel checking と
proof-policy selection に渡す deterministic evidence set である。

## 境界規則

portfolio layer は次を行ってよい:

- policy constraint、source hint、backend availability、logic profile、concrete encoder、
  obligation budget で許可された configured backend profile を選択する;
- 同じ `AtpProblem` から既に構築された concrete TPTP / SMT-LIB encoding を要求または消費する;
- `BackendRunInput` record を backend runner に dispatch する;
- `BackendRunResult` value、candidate evidence ref または payload bytes、counterexample
  diagnostic、stdout/stderr hash、timing summary、resource observation、cancellation record を収集する;
- deterministic candidate id と candidate ordering key を構築する;
- cancellation が要求された場合、または外部 proof-policy finality decision が pending candidate は
  selected policy class を覆せないと述べた場合に限り、残りの backend run を停止する。

次を行ってはならない:

- proof policy を評価する、canonical proof winner を選ぶ、artifact proof status へ project する;
- `mizar-kernel` を呼び出す、SAT checking を実行する、instantiated formula を導出する;
- premise を追加選択する、substitution を発明する、binder を修復する、overload を解決する、
  cluster を探索する、implicit coercion を挿入する、fallback inference を行う;
- externally attested backend output を kernel-verified proof status と同等に分類する;
- backend-reported `used_axioms`、backend proof method、proof log、unsat core、SMT proof object、
  TSTP trace、MiniSAT-compatible resolution trace、legacy certificate を信頼する;
- raw completion order、wall-clock timing、process id、temporary path、backend output order を
  semantic candidate identity に含める;
- proof witness、cache entry、artifact proof status を publish する。

## portfolio planning

planning は等価な入力に対して deterministic である。plan は、選択された backend profile と
concrete encoding ごとに 1 つの `BackendRunInput` を含む。選択には stable input だけを使う:

- `AtpProblem.problem_id`、`vc_id`、`vc_hash`、target binding、logic profile、expected result;
- profile id、backend kind、concrete format、supported observed result、evidence format、
  deterministic priority、required resource limit;
- portfolio 開始前に materialize 済みの proof hint と policy constraint;
- backend availability record と configured executable identity;
- 明示的な obligation budget と scheduler budget。

plan は selected logic profile または concrete encoding を消費できない profile を reject または skip
しなければならない。externally attested output または diagnostic output しか生成できない profile は、
policy constraint がその evidence の記録を許可する場合に限り schedule してよい。それでも
`mizar-atp` 内で kernel-verified にはならない。

schedulable な profile が 1 つもない場合、portfolio は stable diagnostic reason を持つ open
evidence set を返す。backend result や proof candidate を fabricate してはならない。

## candidate model

portfolio candidate は 1 つの backend result から導出される normalized record である:

```text
PortfolioCandidate
  candidate_id
  source_run_id
  backend_profile_id
  encoded_problem_hash
  target_binding
  candidate_kind
  evidence_format
  evidence_payload_or_ref?
  counterexample_ref?
  observed_result?
  provenance_hash
  candidate_hash
  diagnostics
```

candidate kind は policy-neutral である:

- `FormulaSubstitution`: evidence-extraction route が存在した後の、kernel-owned schema と互換な
  formula/substitution evidence candidate;
- `ExternallyAttested`: policy が記録してよいが kernel acceptance ではない backend evidence;
- `Counterexample`: diagnostic model または counterexample evidence;
- `Unknown` / `Error`: proof candidate ではなく diagnostic のみ。

`FormulaSubstitution` candidate も、`mizar-kernel` が検査するまで untrusted である。
`ExternallyAttested` candidate を黙って kernel-verified status に昇格してはならない。backend
proof log は diagnostic または将来の extractor input として保持してよいが、後続へ handoff する
candidate evidence は formula/substitution evidence bytes または ref と target binding /
provenance でなければならない。

## deterministic ordering and identity

portfolio は private な length-prefixed canonical field と明示的 domain tag により stable hash を構築する:

| Hash | Domain | 必須 field |
|---|---|---|
| `plan_hash` | `mizar-atp/portfolio-plan/v1` | `vc_hash`、`AtpProblem.problem_id`、selected profile id、concrete input hash、policy-constraint fingerprint、budget record |
| `candidate_hash` | `mizar-atp/portfolio-candidate/v1` | candidate kind、evidence format、candidate payload/ref hash、target binding、provenance hash、encoded problem hash、backend profile id、observed result |
| `evidence_set_hash` | `mizar-atp/portfolio-evidence-set/v1` | `plan_hash`、sort 済み backend-result metadata hash、sort 済み candidate hash、stop summary、non-semantic diagnostic hash |

candidate ordering は raw completion order から独立している。canonical order は次の通り:

1. candidate kind tag（`FormulaSubstitution`、`ExternallyAttested`、`Counterexample`、
   `Unknown`、`Error`）を handoff grouping として使う。proof policy は評価しない;
2. backend profile deterministic priority;
3. evidence format priority;
4. encoded problem hash;
5. candidate hash;
6. backend profile id;
7. source run id。

この順序は reproducible candidate handoff のためだけのものである。artifact-facing winner order
ではなく、`mizar-proof` policy を上書きしてはならない。

## early stop and cancellation

portfolio が残りの backend process を停止してよいのは次の場合だけである:

- caller cancellation が build snapshot を置き換えた;
- obligation budget または scheduler budget が尽きた;
- 外部 proof-policy finality decision が、active policy の下で pending candidate は selected
  policy class を覆せないと述べている。

外部 finality decision がない場合、安全な既定動作は、scheduled backend result が terminal status
に到達するか cancellation が要求されるまで全て収集することである。`mizar-atp` は backend
completion order、backend priority 単体、externally attested success、まだ kernel checked でない
candidate の存在から policy finality を推論してはならない。

cancellation は in-process portfolio work では cooperative である。child backend process は backend
runner を通じて終了する。cancelled run は diagnostic metadata を残してよいが、partial accepted
proof state は決して残さない。

## resource budget

portfolio は obligation-level budget と各 run に割り当てた per-backend budget の両方を記録する。
budget assignment は deterministic であり、stable profile configuration と明示的 input budget に
基づく。timeout、memory、process-count、stdout/stderr、temporary-file limit は required または
best-effort limit として backend runner に渡す。

unsupported required limit は、`Proved` candidate が構築される前に対象 run を error にする。
budget exhaustion は obligation を open または diagnostic にするだけで、trusted proof status を
作らない。

## kernel and policy handoff

portfolio は次を handoff する:

- candidate が存在し、外部 policy が kernel-checked evidence を有用と判断する場合、
  formula/substitution candidate を kernel check scheduler へ渡す;
- externally attested record または diagnostic record を、accepted proof material として扱わずに
  proof policy と diagnostics へ渡す;
- reproducibility metadata、hash、stdout/stderr ref を、それぞれの所有 crate が stable contract を
  定義した後に artifact / cache layer が消費できるよう渡す。

portfolio は `KernelCheckResult`、`ProofWitnessDraft`、trusted `used_axioms`、artifact proof
selection、proof-cache entry を構築しない。backend-reported used axiom は kernel に検証されるまで
advisory のままである。

## failure semantics

backend failure は各 run に局所化される:

| Condition | Portfolio handling |
|---|---|
| schedulable profile なし | deterministic diagnostic を持つ open evidence set |
| timeout または budget exhaustion | terminal run status と diagnostic。有用な他の run は継続してよい |
| process crash または spawn failure | backend error diagnostic。verifier は crash しない |
| malformed backend output | unknown または error。extraction が成功しない限り `FormulaSubstitution` candidate はない |
| candidate metadata mismatch | handoff candidate として reject し、diagnostic を保持する |
| 後続 kernel rejection | candidate-specific proof error。portfolio evidence set は reproducible のまま |
| 後続 policy rejection | backend rejection と kernel rejection から区別された policy error |

全 run が失敗した portfolio は open proof obligation であり、accepted proof ではない。

## gap classification

- resolved `deferred` spec gap: task 17 は `src/portfolio.rs` が存在する前に、portfolio planning、
  candidate collection、deterministic ordering、early-stop constraint、budget、handoff boundary を
  仕様化する。
- `external_dependency_gap`: `mizar-proof` は workspace crate ではないため、proof policy finality、
  artifact-facing winner selection、witness publication はここでは実装できない。
- `external_dependency_gap` / `deferred`: first real-backend formula/substitution extraction は
  ATP-G-015 により blocked のままである。task 18 は既存 mock candidate または既に仕様化された
  candidate input を使わなければならず、fake real-output schema を発明してはならない。
- `external_dependency_gap`: proof witness storage、artifact projection、proof-cache promotion は
  `mizar-atp` の外に残る。

## task-17 test coverage

task 17 は documentation-only であり、Rust test を追加しない。task 18 は次の source coverage を
追加するべきである:

- backend availability と profile input order を shuffle しても deterministic portfolio planning になること;
- backend completion order を shuffle しても candidate ordering が同一であること;
- 外部 policy finality decision がない場合は early stop しない collection になること;
- 明示的な cancellation を尊重し、partial accepted proof state を残さないこと;
- stable な外部 policy finality contract が存在する場合、その finality decision を尊重し、
  partial accepted proof state を残さないこと。存在しない場合は、implementation が early-stop
  oracle を fabricate しないこと;
- timeout、crash、malformed output、metadata mismatch、unsupported-limit が backend result から伝播すること;
- portfolio API から kernel call、proof policy evaluation、witness/cache publication、accepted proof
  status、trusted backend proof material、caller-supplied instantiated formula、SAT problem が排除されること。
