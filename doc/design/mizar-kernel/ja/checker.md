# Module: checker

> 正本は英語です。英語版:
> [../en/checker.md](../en/checker.md)。

## 目的

`checker` module は、normalized kernel certificate に対する phase 14 orchestration を
所有する。Parser、imported-fact validation、substitution checking、resolution replay、
explicit cluster/reduction trace replay、derived fact validation、final-goal acceptance を
組み合わせ、policy-independent な kernel result を生成する。

この module は
[architecture 15](../../architecture/ja/15.kernel_certificate_format.md) の
「Imported Facts」と「Kernel Rejection Semantics」、
[architecture 17](../../architecture/ja/17.cluster_trace_format.md)、および
[internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md) の
「Kernel Check Service」を精緻化する。

## Trust statement

この module は trusted kernel code である。必要な evidence がすべて explicit immutable
input から replay または check された後でなければ、proof を受理してはならない。

この module は proof search、ATP search、SAT solving、premise selection、overload
resolution、cluster search、registration activation、implicit coercion insertion、
fallback inference、source loading、cache lookup、artifact lookup、wall-clock /
random-state read、unordered iteration、mutable compiler-global state の hidden read を
行ってはならない。Backend-reported success、backend-reported used axioms、resolver
output、cache hit、artifact metadata、policy permission は kernel replay の代替にならない。

## Owned behavior

この module が所有するもの:

- parsed 済み `ParsedCertificate` 上の deterministic check pipeline を構成すること;
- certificate target、profile、immutable kernel context binding を validate すること;
- imported axiom / theorem を stable identity、statement fingerprint、required proof
  status で validate すること;
- `resolution_trace` へ imported-clause evidence を供給すること;
- `substitution_checker` と `resolution_trace` を呼び出し、checked output を使う前に
  private report binding を verify すること;
- evidence schema が実装された後の explicit cluster/reduction trace validation;
- `derived_facts` と `final_goal` を validate すること;
- checked imported fact reference だけから trusted `used_axioms` を抽出すること;
- deterministic accepted/rejected `KernelCheckResult` を返すこと。

この module が所有しないもの:

- certificate byte parsing;
- clause normalization internals;
- MiniSAT resolution replay internals;
- substitution、alpha、freshness、free-variable replay internals;
- cluster / reduction search;
- source-derived certificate production;
- proof-policy projection、witness publication、cache reuse、artifact emission;
- 複数 backend candidate の winner selection。

## Input and context

Task 13 は check-service contract を仕様化し、tasks 14-16 が slice ごとに実装する。

```text
KernelCheckInput
  target_vc_fingerprint
  parsed_certificate
  imported_fact_context
  substitution_context
  cluster_trace_context
  checker_limits

ImportedFactContext
  imported_axioms: sorted map imported_fact_id -> ImportedFactEvidence
  imported_theorems: sorted map imported_fact_id -> ImportedFactEvidence
  provenance_fingerprint

ImportedFactEvidence
  imported_fact_id
  package_id
  module_path
  exported_item_id
  statement_fingerprint
  accepted_proof_status
  normalized_clause_fingerprint
  clause

ClusterTraceContext
  cluster_steps: sorted map cluster_trace_step_id -> ClusterStepEvidence
  reduction_steps: sorted map reduction_step_id -> ReductionStepEvidence
  provenance_fingerprint
```

Concrete Rust type は sorted vector を使ってよい。ただし constructor は input order を
canonicalize するか、duplicate id を replay 前に deterministic に reject しなければならない。
Context entry は certificate order の first use で validate し、unused context entry は無視する。

`ImportedFactEvidence` は caller-supplied immutable evidence である。Resolver、checker、
ATP backend、cache、artifact、package index、global compiler table から query しない。
Imported-fact context または provenance の欠如は `missing_provenance` である。Parsed
imported fact に対応する evidence がない、identity が一致しない、fingerprint が一致しない、
または `RequiredProofStatus` より弱い status でしか受理されていない場合は
`unresolved_symbol` である。

`ImportedFactEvidence.clause` は resolution replay へ渡す normalized clause である。
`resolution_trace` に渡す前に、parsed certificate の kernel profile、symbol manifest、
variable manifest、checker limits に対して validate する。Checker は parsed certificate の
hash と clause profile の下で normalized clause の canonical fingerprint を再計算し、それが
`normalized_clause_fingerprint` と等しいこと、さらに再計算した fingerprint と evidence の
`statement_fingerprint` が parsed `ImportedFactRef.statement_fingerprint` と等しいことを要求する。
提供された immutable evidence の clause shape または profile mismatch は
`missing_provenance` であり、imported identity、clause-content fingerprint、proof-status
mismatch は `unresolved_symbol` である。

`KernelCheckInput` には caller-supplied `imported_clause_context` は存在しない。
`resolution_trace` へ渡す imported-clause context は、identity、fingerprint、proof-status、
clause-validation checks を通過した imported facts から checker が構築したものだけである。
これにより、unchecked clause が imported-fact validation を迂回することを防ぐ。

`cluster_trace_context` は explicit evidence input のままである。Task 15 が
cluster/reduction replay を実装するまでは、それを必要とする certificate evidence は開発計画上
`external_dependency_gap` / `deferred` として扱い、runtime では placeholder によって受理せず、
missing または invalid evidence として fail closed する。

Cluster / reduction evidence records は architecture 17 の replay fields を保持しなければ
ならない:

```text
ClusterStepEvidence
  cluster_trace_step_id
  source_type
  applied_cluster
  generated_attribute
  generated_type
  dependency

ReductionStepEvidence
  reduction_step_id
  applied_reduction
  rule_fqn
  enclosing_term_before
  redex_path
  source_redex
  target_term
  substitution
  discharged_guards
  rule_view
  selection_key

GuardEvidence
  guard_id
  source_fact_ref
  checked_dependency_ref
```

`enclosing_term_before`、`redex_path`、`rule_view`、`selection_key` のような
strategy-audit fields は、recorded evidence として check する。Kernel は explicit
active-rule data と recorded enclosing term に対して selected redex と rule を audit
するだけであり、代替 redex または rule を search しない。

## Result shape

Success / failure surface は policy-independent である:

```text
KernelCheckResult
  target_vc_fingerprint
  status
  checked_imports
  checked_substitutions
  checked_resolution_steps
  checked_cluster_steps
  checked_derived_facts
  final_goal
  used_axioms
  rejections

KernelCheckStatus
  accepted
  rejected

CheckedImportedFact
  namespace
  imported_fact_id
  statement_fingerprint
  accepted_proof_status
  policy_taint

CheckedDerivedFact
  derived_fact_id
  source_clause_ref
  payload_fingerprint
```

`accepted` は、この crate が normalized certificate の final goal を check したことだけを
意味する。Artifact-facing proof status は含まない。`mizar-proof` または後続 policy layer は
accepted ATP certificate を `kernel_verified`、accepted built-in certificate を
`discharged_builtin` として project してよいが、その projection はこの crate の外である。

`used_axioms` は、accepted certificate が実際に使用した checked imported axiom/theorem
reference だけから導出する。Backend-reported used-axiom list は、normalized certificate と
imported-fact context が同じ fact を check 可能にしない限り無視する。

Checked import のいずれかが `accepted_proof_status =
externally_attested_policy_permitted` を持つ場合、accepted kernel result はその import と
aggregate result に policy taint を持つ。Policy layer はそのような result を無条件の
`kernel_verified` として project してはならない。Policy-controlled な externally attested
または mixed-status result としてのみ emit してよい。Active release policy がその taint を
禁止する場合、immutable imported-fact context は external status を requirement を満たすもの
として提示してはならない。

Batch checking は single-certificate checks の deterministic wrapper である:

```text
KernelCheckBatchInput
  checks: sorted Vec<KernelCheckInput>

KernelCheckBatchResult
  results: sorted Vec<KernelCheckResult>
```

Batch results は target VC fingerprint、存在する場合は evidence id、canonical input order で
sort される。Worker completion order、cancellation arrival order、parallel scheduling は
result order に影響してはならない。

## Check pipeline

Checker は次の手順を deterministic order で実行する:

1. Parsed certificate target と kernel profile が caller の expected target / checker
   configuration と一致することを確認する。
2. Parsed certificate data と `checker_limits` から manifest-derived clause / term validation
   contexts を構築する。
3. Imported axiom / theorem reference を `ImportedFactContext` に対して first use で
   validate する。
4. Checked imported fact evidence だけから `resolution_trace` 用 imported-clause context を
   構築する。
5. `substitution_checker` で substitutions を replay し、その checked report だけを保持する。
6. `resolution_trace` で MiniSAT-compatible resolution trace を replay し、その checked report
   だけを保持する。
7. Task 15 evidence が存在する場合、explicit cluster/reduction traces を replay する。
8. 各 `derived_facts` の source clause reference が checked generated / imported /
   resolution / substitution-derived / cluster-derived fact を指すことを payload schema に従って
   validate する。
9. `final_goal` が参照する generated clause、resolution step、derived fact を解決し、
   target VC が要求する empty obligation または canonical final fact であることを check する。
10. Accepted result を 1 つ emit するか、earliest stable rejection records を含む rejected
    result を deterministic に emit する。

Checker は failed sub-checker report を repair したり、alternate pipeline を試してはならない。
Sub-checker が evidence を reject した場合、checker も certificate を reject する。

## Imported fact checking

Task 14 は resolution replay 前の imported-fact validation を実装する。

Parsed `ImportedFactRef` ごとに、checker は次を比較する:

- `imported_fact_id`;
- `package_id`;
- `module_path`;
- `exported_item_id`;
- `statement_fingerprint`;
- `required_proof_status`。

Proof-status strength は次の順序である:

```text
kernel_verified > discharged_builtin > externally_attested_policy_permitted
```

Evidence status は、parsed requirement 以上の強さを持ち、active kernel profile に許可される場合
だけ requirement を満たす。Externally attested fact は kernel-verified ではない。Parsed
certificate がその requirement を明示的に許可し、immutable context が policy-permitted status
を記録している場合だけ受理する。External attestation を禁止する release policy はこの module
の外だが、その decision は immutable imported-fact context を構築する前に反映されていなければ
ならない。

Imported proof-status、identity、fingerprint failure は `imported_fact_id` 付きの
`unresolved_symbol` である。Context または context provenance の欠如は
`missing_provenance` である。

## Cluster and reduction trace boundary

Task 15 は explicit cluster/reduction trace replay を実装する。Checker spec は次を要求する:

- cluster search または registration activation を行わない;
- hidden transitive expansion を行わない;
- generated type fact、reduction result、guard discharge はすべて explicit trace evidence に
  よって裏付けられる;
- trace が参照する dependency fact は、imported fact、generated fact、または earlier trace
  step として既に checked である;
- invalid cluster/reduction evidence は `invalid_cluster_trace` へ map する;
- cluster trace context または provenance の欠如は `missing_provenance` へ map する。

Task 15 開始時点で upstream `mizar-checker` cluster trace payload が未準備なら、その task は
gap を `external_dependency_gap` / `deferred` として記録し、runtime behavior は fail-closed
のままにしなければならない。

## Derived facts and final goal

`ParsedCertificate.derived_facts` は certificate-owned assembly records である。Task 16 は、
imported facts と cluster/reduction traces に concrete evidence contract ができた後で payload
schema を validate する。それまでは unknown derived-fact payload を受理しない。

`ClusterTraceContext` には caller-supplied derived-fact payload map は存在しない。唯一の
payload authority は parsed normalized certificate である。Checked derived fact は parsed
`derived_fact_id`、`source`、payload bytes に bind し、その payload を既に checked された
imported facts、generated clauses、resolution steps、または cluster/reduction steps に対して
validate しなければならない。External trace evidence は dependency を justify してよいが、
certificate-owned derived-fact payload を置換または補足してはならない。

`final_goal` acceptance は deterministic である:

- `generated_clause` と `resolution_step` goal は checked clause に解決され、後続 spec が別の
  final-fact schema を追加しない限り canonical empty clause でなければならない;
- `derived_fact` goal は checked derived fact に解決され、その payload schema が target VC を
  close すると明示していなければならない;
- `generated_clause` final goal は、checked resolution final-goal helper または checked
  derived-fact payload など、successful checked replay path によってその generated clause が
  消費された場合だけ accepted される。`ParsedCertificate.generated_clauses` に存在するだけでは
  proof acceptance ではない;
- missing、unchecked、forward、mismatched final-goal reference は、failed evidence family に
  応じて `invalid_sat_proof` または `invalid_cluster_trace` である;
- target mismatch は `context_mismatch` である。

## Limits

`CheckerLimits` は deterministic budgets をまとめ、該当 subset を sub-checkers へ渡す:

```text
CheckerLimits
  parser limits
  resolution replay limits
  substitution replay limits
  imported fact count
  imported clause validation limits
  cluster trace step count
  reduction trace step count
  derived fact count
  final report record count
```

Checker-owned budget を超えた場合は `resource_exhaustion` である。Large temporary vector の
allocation、unbounded context entry の sorting、imported clause の clone、report
materialization の前に budget check を行う。

## Rejection mapping

| Failure | Detail | Location |
|---|---|---|
| Missing imported fact context, cluster trace context, substitution context, derived imported-clause context, or provenance | `missing_provenance` | field path plus imported fact, substitution, cluster, reduction, or final-goal id when known |
| Malformed service witness envelope before parsing or before normalized evidence can be selected | `malformed_witness_data` | service evidence field path |
| Imported fact identity, statement fingerprint, unavailable theorem/axiom, or proof-status strength mismatch | `unresolved_symbol` | `imported_fact_id` |
| Substitution replay failure | forwarded `invalid_substitution`, `missing_provenance`, or `resource_exhaustion` | forwarded substitution location |
| Resolution replay failure | forwarded `invalid_sat_proof`, `missing_provenance`, or `resource_exhaustion` | forwarded clause or resolution-step location |
| Cluster/reduction trace replay failure | `invalid_cluster_trace` | cluster or reduction step id |
| Derived fact payload mismatch or unchecked dependency | `invalid_sat_proof` or `invalid_cluster_trace` | `derived_fact_id` |
| Final goal mismatch or unchecked final reference | `invalid_sat_proof` | `final_goal` plus referenced id when known |
| Target VC or context binding mismatch | `context_mismatch` | target/context field path |
| Unsupported checker or certificate profile | `unsupported_certificate_format` | profile field path |
| Checker-owned deterministic resource budget exhausted | `resource_exhaustion` | checker budget field path |
| Cancellation or timeout budget exhausted after parsing | `timeout` | cancellation or timeout field path |

複数の checks が失敗する場合、deterministic ordering は `rejection.md` に従う。Human diagnostic
text は context を追加してよいが、stable detail key と location は display name、file path、
backend log、cache key、worker completion order、allocation address、wall-clock time、
random state に依存してはならない。

## Determinism and cost

Checker は parsed certificate vectors を parser-validated order で処理する。Context
constructor は check 開始前に caller-supplied evidence を canonicalize する。Reports は stable
id と parser order だけで sort する。

Cost は checked certificate records と explicit に参照された context evidence に対して、設定
limits 内で線形である。Checker は unrelated dependency artifact を scan せず、alternate fact、
trace、substitution、proof を search してはならない。

Cancellation は cooperative かつ deterministic である。Checker は `CheckerLimits` で数えられる
定義済み step-boundary でのみ停止してよい。Stopped check は `timeout` を返し、partial
acceptance を返してはならない。Parser-owned malformed bytes は `malformed_certificate` のまま
であり、certificate または explicit kernel evidence に normalize できない service-envelope
evidence は `malformed_witness_data` である。

## Gap classification

- `spec_gap`: task 13 以前は、sub-checker report、imported facts、explicit cluster traces、
  derived facts、final-goal acceptance をどう合成するかを定義する local `checker` module
  contract がなかった。この spec は tasks 14-16 のためにその contract を閉じる。
- `test_gap`: task 14 は imported-fact validation tests を必要とする。Task 15 は explicit
  cluster/reduction trace replay tests、または記録済み `external_dependency_gap` を必要とする。
  Task 16 は end-to-end check-service と final-goal tests を必要とする。
- `external_dependency_gap`: source-derived certificate、ATP proof translation、`mizar-checker`
  による cluster trace payload production、proof/cache/artifact consumers は、この crate の
  active input ではない。Missing producer / consumer integration はここで mock しない。
- `deferred`: proof-policy projection、witness storage、cache reuse、artifact emission、
  backend-candidate selection は `mizar-kernel` の外に残る。

## Planned tests

Task 14 は以下の Rust tests を追加しなければならない:

- imported axiom / theorem evidence は identity、fingerprint、proof status が parsed
  requirement を満たす場合だけ accepted される;
- imported-fact context / provenance 欠如は `missing_provenance`;
- unavailable または mismatched imported facts は `unresolved_symbol`;
- imported clause evidence は resolution replay 前に certificate profile、symbol manifest、
  variable manifest、resource limits に対して validate される;
- mismatched `normalized_clause_fingerprint` と recomputed clause-content fingerprint
  mismatch は imported clause が resolution replay に入る前に rejected される;
- unused malformed imported context entries は無視される。

Task 15 は以下の Rust tests を追加しなければならない:

- explicit cluster / reduction traces は recorded evidence からだけ accepted される;
- hidden transitive expansion、invalid reduction substitution、missing guard evidence、
  dependency mismatch、strategy-audit mismatch は `invalid_cluster_trace`;
- cluster trace context / provenance 欠如は `missing_provenance`;
- upstream trace payload が未準備なら、fail-closed `external_dependency_gap` behavior を
  assert する。

Task 16 は以下の Rust tests を追加しなければならない:

- checked imports、substitutions、resolution trace、optional cluster trace、derived facts、
  final goal から full pipeline が accepted される;
- final-goal mismatch と unchecked final reference が deterministic に rejected される;
- duplicate context id、duplicate evidence id、simultaneous imported/cluster context
  failures、multiple rejection records が stable location で sort される;
- sub-checker report の accidental reuse を report/input binding が防ぐ;
- shuffled context construction と shuffled parallel batch completion の下でも result ordering
  が deterministic である;
- externally attested imported facts の policy taint propagation;
- certificate-owned derived-fact payload を external に置換または補足しようとする入力は
  final-goal acceptance 前に rejected される;
- malformed witness envelopes は `malformed_witness_data`、deterministic
  timeout/cancellation budgets は `timeout`、checker-owned deterministic resource limits は
  `resource_exhaustion` として rejected される;
- trusted-boundary lint/test set が trust statement を mirror すること: proof search、
  ATP search、SAT solving、premise selection、overload resolution、cluster search、
  registration activation、implicit coercion insertion、fallback inference、source loading、
  hidden dependency-artifact reads、ATP/proof/cache/artifact coupling、unordered iteration、
  wall-clock/random read、global mutable-state read がないこと。
