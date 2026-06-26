# Module: resolution_trace

> 正本は英語です。英語版:
> [../en/resolution_trace.md](../en/resolution_trace.md)。

## 目的

`resolution_trace` module は、normalized kernel certificate が持つ
MiniSAT-compatible resolution step の決定的 replay を所有する。この文書は
[architecture 15](../../architecture/ja/15.kernel_certificate_format.md)
「Resolution Trace」を精緻化し、`certificate_parser` が既に parse した
certificate data、`clause` が所有する normalized clause、`rejection` が所有する
shared rejection record だけを消費する。

Resolution replay は evidence checking であり solving ではない。成功した replay
が示すのは、列挙された step が列挙された parent から従うことだけである。
最終的な proof acceptance は後続の `checker` module が所有する。

## Closeout 後の修正

この module は formula/substitution evidence correction 後の legacy compatibility
surface である。通常 proof policy は replay 済み MiniSAT-compatible resolution trace を
trusted acceptance material として扱ってはならない。Task 29 は public service use を
`KernelCheckPolicy.allow_legacy_certificate_audit` の背後に gate し、通常 proof policy 下で
downstream crate が replay success を `kernel_verified` と誤認できないようにする。

Migration/audit mode は legacy trace を inspect し、debugging 用の deterministic checked
record を返してよいが、service-level result は `unsupported_certificate_format` audit record を
持つ `Rejected` のままであり、trusted `final_goal` や `used_axioms` は持たない。その audit data
は trusted proof witness、cache promotion、artifact proof status、`kernel_verified` acceptance へ
投影してはならない。新しい ATP work はこの trace format ではなく formula/substitution evidence
candidate を生成しなければならない。

## Trust Statement

この module は trusted kernel code である。explicit resolution step を
certificate order で replay し、主張された各 resolvent を再計算し、step が一致し
ない場合は fail closed しなければならない。

この module は SAT solving、ATP search、proof search、premise selection、
overload resolution、cluster search、implicit coercion insertion、fallback
inference、imported-fact discovery、cache lookup、artifact lookup、wall-clock または
random-state read、unordered iteration、mutable compiler-global state の hidden read
を行ってはならない。Alternate parent、alternate pivot、alternate generated clause、
backend-reported used axiom を試すことで trace を修復してはならない。

Task 20 の audit では、この trust boundary は no proof search, no SAT solving,
no ATP search or backend invocation, no premise selection, no overload
resolution, no cluster search, no implicit coercion insertion, no fallback
inference, no acceptance from backend-reported success alone, no source
loading, no cache lookup, no artifact lookup, no wall-clock or random-state
reads, no unordered iteration dependence, no hidden reads of mutable
compiler-global state を含むものとして検査する。

## 所有する挙動

この module が所有するもの:

- parsed certificate order に従う resolution step replay;
- explicit な generated/imported/previously checked step clause に対する parent
  reference 解決;
- pivot polarity と parent occurrence の検査;
- `clause` module を使った resolvent の再計算と canonicalization;
- 再計算した resolvent と referenced generated clause の比較;
- 後続 step と checker orchestration のための checked step clause の記録;
- deterministic replay-count / replay-size limit の適用;
- replay failure から stable `rejection` record への mapping。

この module が所有しないもの:

- normalized certificate byte parsing または structural reference validation;
- ATP proof format から backend-specific MiniSAT trace を構築すること;
- imported fact availability、content fingerprint、proof-status validation;
- substitution、alpha-conversion、free-variable、derived-fact、cluster trace
  checking;
- proof-policy projection、witness storage、cache reuse、artifact emission;
- 複数 backend result から winning proof candidate を選ぶこと。

## Input and context

Task 9 は explicit immutable input から replay を実装する:

```text
ResolutionTraceInput
  target_vc_fingerprint
  parsed_certificate
  imported_clause_context
  replay_limits
```

`target_vc_fingerprint` は caller-owned であり、stable rejection record または
private report binding check にだけ copy される。Backend output、cache state、
artifact state、mutable compiler-global state から導出してはならない。

`parsed_certificate` は `certificate_parser::ParsedCertificate` である。Parser は既に
section order、stable id uniqueness、parent reference shape、generated-clause
reference existence、self/forward resolution-step reference、final-goal reference
shape を検査している。Replay checker は debug-oriented tests でこれらを assert して
よいが、byte parsing を重複実装してはならない。

`imported_clause_context` は caller-supplied immutable data である:

```text
ImportedClauseContext
  imported_axiom_clauses: sorted map imported_fact_id -> Clause
  imported_theorem_clauses: sorted map imported_fact_id -> Clause
  provenance_fingerprint
```

具体的な Rust type は、lookup と iteration が deterministic である限り、map 依存を
避けて sorted vector を使ってよい。この context は resolver state、ATP output、
cache state、artifact state、global compiler state から populate してはならない。
Imported-clause context が欠けている場合は `missing_provenance` である。
実装は、各 imported namespace を sorted unique id として保持する constructor または
type invariant によって imported context ordering を deterministic にしなければならない。
Input order を canonicalize してよいが、duplicate id は invalid context shape であり、
replay 前に決定的に拒否しなければならない。
Context は存在するが、parent reference が必要とする parsed imported namespace/id を
含まない場合も、task 9 は `missing_provenance` を返さなければならない。Resolution
replay が step を検査するために必要な immutable clause evidence を欠いているためで
ある。Imported fact の unavailable や fingerprint mismatch は後続の `checker`
imported-fact task が所有し、replay orchestration の前後で `unresolved_symbol` へ map
する。

Parsed trace が使う supplied imported `Clause` value は、replay-derived context と同じ
profile、symbol manifest、variable manifest に対して既に normalized でなければならない。
Task 9 は imported clause を deterministic trace order で first use 時に validate し、
unused context entry を scan して拒否してはならない。実装は profile agreement を安価に
検査した後、used imported clause を generated parent や replay 済み step parent と同じ
bounded parent `ClauseValidationContext` で validate し、それが成功してから replay 用に
clone する。Non-resource の profile、symbol、variable、canonical-form incompatibility は
`missing_provenance` であり、parent を修復または再解釈する機会ではない。Parent literal
count、term-size、term-depth、canonical-byte budget failure は replay resource check の
ままであり、`resource_exhaustion` に map する。

`replay_limits` は deterministic である:

```text
ResolutionReplayLimits
  max_checked_steps
  max_parent_literals
  max_resolvent_literals
  max_resolvent_canonical_bytes
  max_term_encoding_bytes
  max_term_recursion_depth
```

Limit 超過は `resource_exhaustion` detail を持つ `kernel_rejection` である。大きな
temporary resolvent を allocate する前に budget を検査しなければならない。

Replay normalization は public parsed certificate data と replay limits だけから導いた
`ClauseValidationContext` を使わなければならない:

- `ClauseProfile` は `ParsedCertificate.kernel_profile` の
  `clause_schema_version`、`clause_encoding_version`、`clause_tautology_policy`
  から構築する;
- allowed / known symbols は `ParsedCertificate.symbol_manifest` から得る;
- canonical variables は `ParsedCertificate.variable_manifest` から得る;
- literal、term-size、term-recursion/depth の limit は `ResolutionReplayLimits` から
  得る。この depth limit は caller-supplied imported clause と parsed/generated clause の
  両方に適用する。

Task 9 がこの導出の helper を必要とする場合、`certificate_parser` に小さな public
または crate-private helper を追加してよい。ただし global lookup、downstream crate
dependency、second parser を追加してはならない。

Canonical-byte limit accounting は clause-owned non-allocating helper を使わなければ
ならない。現在の `clause` API が canonical byte vector を allocate せずに literal または
clause の canonical length を計算する十分な情報を公開していない場合、task 9 は
`clause` に小さな public または crate-private length/bounded-writer helper を追加しなけれ
ばならない。Resolution checker は clause encoder を重複実装したり、replay limit 超過を
知るためだけに canonical bytes を allocate したりしてはならない。

Imported-clause validation も depth-bounded でなければならない。現在の `clause` API が
明示的な recursion-depth budget で borrowed `Term` value を validate できない場合、
task 9 は clause-owned depth-bounded validation helper を追加するか、clause validation
context を拡張しなければならない。Resolution checker は caller-supplied imported term を
deterministic depth budget なしに recursive walk してはならない。

## Clause reference resolution

Checker は certificate から parse された同じ clause-reference namespace を受け付ける:

| Namespace | Replay source |
|---|---|
| `generated_clause` | Parsed certificate 内の generated clause。 |
| `resolution_step` | 以前に replay 済みの step が生成した checked clause。 |
| `imported_axiom` | `imported_clause_context` が与える normalized clause。 |
| `imported_theorem` | `imported_clause_context` が与える normalized clause。 |

Resolution-step parent は earlier checked step だけを参照できる。Parser は self と
forward reference を malformed certificate として拒否する。Task 9 tests でも、
replay が unchecked future step を参照しないことを確認する。

Checker は missing reference の clause を合成したり、alternate namespace を探したり、
display name で imported fact を調べたり、backend-provided used-axiom list を parent
table として受理したりしてはならない。

## Replay algorithm

Certificate order の各 `ResolutionStep` について:

1. step count と両 parent clause size の replay limit を検査する。
2. explicit reference source から `parent_a` と `parent_b` を lookup する。
3. `pivot_literal` が canonical literal identity で `parent_a` に出現することを
   検査する。
4. 同じ atom かつ opposite polarity の literal が `parent_b` に出現することを
   検査する。
5. Matched pivot を除いた後の parent literal count から、allocation-free に resolvent
   literal count の upper bound を計算する。その bound が `max_resolvent_literals` を
   超える場合は allocation 前に拒否する。
6. `parent_a` から pivot を除いた全 literal と、`parent_b` から opposite-polarity
   pivot を除いた全 literal から、bounded accumulator を通して raw resolvent を作る。
   Accumulator は各 push の前に literal count と canonical-byte total を検査し、
   replay limit を超えて grow する前に止める。
7. Bounded raw resolvent を導出済みの `ClauseValidationContext` で normalize する。
8. Normalized resolvent を `generated_clause` と比較する。
9. 後続 step のために checked step id と normalized resolvent を記録する。

Parent orientation は意味を持つ。`parent_a` は encode された pivot を含み、
`parent_b` は opposite-polarity literal を含まなければならない。反対向きを使いたい
producer は parent を入れ替えるか、pivot polarity を明示的に反転して encode する。
Checker は parent を黙って入れ替えてはならない。

比較は normalized clause value と canonical bytes の structural comparison である。
Rendered text、display name、source range、backend log、allocation address、
hash-map iteration order、worker completion order を使ってはならない。

## Final-goal interaction

Resolution checker は replay 済み各 step の checked clause を後続の `checker`
module に報告してよい。それ自体は trusted proof acceptance を生成しない。

Task 9 の success report は deterministic checked-step data だけを expose する:

```text
ResolutionReplayReport
  checked_steps: sorted Vec<CheckedResolutionStep>

CheckedResolutionStep
  step_id
  generated_clause_id
  clause
```

この report は後続 checker orchestration のための evidence-replay output である。
Accepted proof status、used-axiom projection、policy outcome、artifact-facing witness
decision を含んではならない。実装は、report を別の replay input と誤って組み合わせる
ことを拒否する目的に限り、caller-owned target fingerprint や certificate hash input
などの private replay binding data を保持してよい。Accessor は引き続き checked-step
data だけを expose しなければならない。

Task 9 が `generated_clause` または `resolution_step` namespace の final goal を検証
する helper を含む場合、その helper は、参照された clause が successful replay に
よって checked であることを要求しなければならない。`resolution_step` reference は
その step が成功裏に replay された後だけ checked である。`generated_clause` reference
は、その generated clause id が少なくとも 1 つの successfully replayed step の claimed
output である場合だけ checked である。Parsed `generated_clauses` section に存在するだけ
では足りない。Checked final-goal clause は、後続の `checker.md` spec が別の
final-goal rule を明示しない限り、profile の `empty` contradiction form でなければ
ならない。Unchecked または non-empty resolution final goal は `invalid_sat_proof` で
ある。

`derived_fact` final goal はこの module の外であり、後続の checker orchestration と
substitution/cluster-derived fact task に defer する。

## Rejection mapping

Replay failure は `kernel_rejection` record を生成する:

| Failure | Detail | Location |
|---|---|---|
| Imported-clause context、context provenance、supplied context 内の imported parent namespace/id、または replay profile/manifest context と compatible でない used imported clause | `missing_provenance` | 分かる場合は `resolution_step_id` と parent `clause_ref` |
| Parsing invariant が壊れた状態で generated または earlier-step parent が欠けている | `invalid_sat_proof` | `resolution_step_id` と parent `clause_ref` |
| Pivot が `parent_a` に存在しない | `invalid_sat_proof` | `resolution_step_id`、parent `clause_ref`、pivot field path |
| Opposite-polarity pivot が `parent_b` に存在しない | `invalid_sat_proof` | `resolution_step_id`、parent `clause_ref`、pivot field path |
| 再計算した resolvent が referenced generated clause と異なる | `invalid_sat_proof` | `resolution_step_id` と generated `clause_ref` |
| Resolution final goal が unchecked または non-empty checked clause を参照する | `invalid_sat_proof` | `resolution_step_id` または final-goal marker |
| Replay count、parent-size、term-size、term-depth、resolvent-size、canonical-byte limit 超過 | `resource_exhaustion` | 利用できる最も精密な step、parent、generated-clause location |

すべての rejection location は deterministic で、`rejection.md` の shared
`RejectionLocation` field を使わなければならない。Human diagnostics が追加の text
を含んでよいが、それは acceptance、ordering、stable detail key に影響してはならない。

## Determinism and cost

Replay cost は、明示的な per-step limit の範囲で、declared trace size と parent /
resolvent の total literal payload size に線形でなければならない。Checker が一時
index を使う場合は deterministic data structure または sorted vector を使う。

同一の parsed certificate、imported clause context、limit に対する結果は platform
と worker schedule に依存せず同一でなければならない。Parallel batch checking は
`checker` が所有するが、この module は worker completion order に依存しない ordering
の output を expose しなければならない。

## Gap classification

- `spec_gap`: architecture 15 は high-level resolution replay を説明するが、module が
  所有する input context、parent-reference ownership、final-goal helper boundary、
  rejection mapping、replay limit を定義していない。この module spec は task 8 の
  ためにその gap を閉じる。
- `test_gap`: task 9 では valid replay、各 single-step mutation class、imported parent
  context handling、final-goal helper behavior、deterministic output、replay-cost limit
  の Rust tests がまだ必要である。
- `external_dependency_gap`: backend-specific proof から normalized MiniSAT-compatible
  trace への translation は将来の `mizar-atp` 作業が所有する。Proof-policy projection
  と witness publication は将来の `mizar-proof`、`mizar-cache`、`mizar-artifact` 作業が
  所有する。`mizar-kernel` に placeholder integration を追加してはならない。
- `deferred`: imported fact availability、content-fingerprint validation、required
  proof-status validation は後続の `checker` imported-fact task に着地する。Source-derived
  `.miz` snapshot と expectation sidecar は後続の soundness corpus task に着地する。

## Planned tests

Task 9 は以下の Rust tests を追加しなければならない:

- explicit parent clause 2 個から empty clause を導出する valid single-step replay;
- explicit immutable context が与える generated-clause、imported-axiom、
  imported-theorem、earlier resolution-step parent を使う valid replay;
- pivot が `parent_a` に存在しない場合に `invalid_sat_proof` として拒否すること;
- opposite-polarity pivot が `parent_b` に存在しない場合に `invalid_sat_proof` として
  拒否すること;
- certificate が parent または pivot polarity を明示的に入れ替えない限り、swapped
  parent orientation を拒否すること;
- 再計算した resolvent に extra literal、missing literal、different polarity、
  different canonical literal bytes がある場合に generated-clause mismatch を拒否すること;
- active clause profile に従う tautology と empty-clause outcome;
- imported-clause context が欠けている場合に global state を参照せず
  `missing_provenance` として拒否すること;
- context は存在するが provenance が欠けている input を `missing_provenance` として
  拒否すること;
- supplied context 内に imported parent namespace/id が欠けている場合に
  `missing_provenance` として拒否すること;
- imported context construction が input order を canonicalize し、duplicate id を replay
  前に決定的に拒否すること;
- used imported context clause の profile、symbol、variable、canonical form が
  replay-derived context で validate できない場合に `missing_provenance` として
  拒否すること;
- unused または extra imported context entry は `resolution_trace` replay では無視し、
  exact imported-fact auditing は `checker` に defer すること;
- defensive constructor または test fixture で generated / earlier-step parent
  invariant が壊れている場合に `invalid_sat_proof` として拒否すること;
- `generated_clause` または `resolution_step` final goal について、resolution
  final-goal helper が checked empty clause だけを受理すること;
- `generated_clause` final goal が parsed section には存在するが successfully replayed
  step によって生成されていない場合に拒否すること;
- `generated_clause` final goal が successful replay によって生成されていても checked
  clause が non-empty の場合に拒否すること;
- `resolution_step` final goal について、step が unchecked または checked step clause が
  non-empty の場合に拒否すること;
- すべての replay rejection で `kernel_rejection`、stable detail key、caller-owned
  target fingerprint、rejection table が約束する最も精密で決定的な
  `RejectionLocation` field を assert すること;
- replay count、parent literal、term encoding、term recursion depth、resolvent literal、
  resolvent byte limit を、大きな allocation または deep recursion の前に
  `resource_exhaustion` として拒否すること;
- deeply nested imported context term を、stack overflow や panic ではなく、
  clause-owned depth-bounded validation path によって `resource_exhaustion` として
  拒否すること;
- parsed kernel profile、symbol manifest、variable manifest、literal limit、term
  encoding limit、term-depth limit から、public parsed data または明示的な
  crate-private helper だけを使って replay context を導出すること;
- canonical-byte accounting に clause-owned non-allocating canonical length または
  bounded-writer helper を使い、clause encoder を重複実装しないこと;
- oversized resolvent が unbounded raw literal vector を collect する前に拒否される
  bounded accumulator behavior;
- shuffled test fixture construction または simulated worker completion order の下でも
  checked-step output と rejection ordering が決定的であること;
- success report が checked step id、generated clause id、clause だけを expose し、
  proof-acceptance または policy-status field を含まないこと;
- resolution-trace module が SAT solver、ATP/proof/cache/artifact coupling、proof
  search、premise selection、overload resolution、cluster search、implicit coercion
  insertion、fallback inference、unordered iteration、wall-clock/random read、
  global mutable-state read を持たないことを示す lint coverage。Closeout 後の
  audit 済み SAT checker は `sat_checker` だけに属する。

この module-spec task では `.miz` fixture、expectation sidecar、`doc/spec`、Rust source
change は不要である。
