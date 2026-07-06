# mizar-checker cluster trace design

> 正本は英語です。英語版: [../en/cluster_trace.md](../en/cluster_trace.md)。

## 目的

この文書は、[architecture 17](../../architecture/ja/17.cluster_trace_format.md)
の正準 `ResolutionTrace` schema を、`mizar-checker` の phase-7 実装用に精緻化する。
artifact schema は fork しない。checker は architecture document が定義するものと同じ
cluster step / reduction step concept を emit し、この文書は tasks 16-18 の
checker-local な ownership、ordering、replay、diagnostic、planned test boundary を固定する。

task 15 は documentation-only である。source behavior、artifact writer、parser /
resolver payload extraction は導入しない。

## authority と scope

primary authority:

- [spec 17](../../../spec/ja/17.clusters_and_registrations.md): cluster rule、
  reduction termination、deterministic reduction strategy、reduction traceability;
- [spec 23.7.7](../../../spec/ja/23.package_management_and_build_system.md#2377-ストレージフォーマットcluster-dbresolution-tracediagnostic-explanation):
  `resolution-trace/` storage と minimum-kernel replay requirement;
- [architecture 01](../../architecture/ja/01.ir_layers.md): immutable source-shaped
  IR layer としての `ResolutionTrace` ownership;
- [architecture 04](../../architecture/ja/04.type_and_registration_resolution.md):
  phase-7 registration resolution input/output;
- [architecture 17](../../architecture/ja/17.cluster_trace_format.md): replayable trace の
  正準 schema。

scope に含むもの:

- derived cluster fact と automatic reduction application に対する
  `ResolutionTrace` record の checker-local construction;
- deterministic step id、fact reference、traversal profile、debug rendering;
- diagnostic、artifact validator、minimum kernel 用の replay input contract;
- loop、bounded saturation、invalid substitution、mismatched strategy-audit key、
  invisible registration の failure classification;
- tasks 16-18 と後続 determinism work の planned Rust / corpus coverage。

scope 外:

- task 19 と後続 proof crate が所有する pending registration activation、proof
  acceptance、verifier policy decision;
- MC-G021 が開いている間の registration pattern、reduction term、guard evidence の
  source-to-checker payload extraction;
- canonical schema を使う build/artifact task が所有する JSON artifact emission、
  cache storage、artifact reader compatibility;
- ATP proof search、新しい theorem fact、overload candidate selection、hidden
  coercion insertion。

## trace model

`ResolutionTrace` は source file scoped である。trace は次から構築する。

- checked file の source / module identity;
- phase-6 `TypedAst` と `TypeFactTable`;
- task-14 registration database の activated registration に限定した view;
- 後続 task が供給する checker-ready cluster / reduction payload;
- cluster expansion depth や maximum generated fact count などの deterministic
  traversal setting。

trace output は次を含む。

- ordered cluster / reduction step;
- type fact table に追加された derived cluster fact;
- configured bound と ordering version を含む traversal profile metadata。

checker は trace construction / validation 中に diagnostic を emit してよく、
traversal profile は diagnostic count や stable diagnostic reference を記録してよい。
これらの diagnostic は追加の `ResolutionTrace` schema field ではない。詳細な
diagnostic payload は、canonical artifact model が定義する diagnostics / explanation
channel に置く。

pending、rejected、recovered、malformed、unaccepted registration は trace input ではない。
後続 operation がそれらを trace builder に使わせようとした場合、active step を捏造せず、
deterministic diagnostic で却下する。

### task 16: cluster closure data layer

task 16 はこの model の cluster 側を `src/cluster_trace.rs` として実装する。

最初の実装は `ClusterTraceBuilder`、`ClusterRuleInput`、`ClusterFactInput`、
`ClusterClosureOutput`、`ResolutionTrace`、cluster step、closure fact table、
traversal profile、replay report、checker-local diagnostic を公開する。builder は
task-14 `RegistrationDatabase` と explicit checker-owned rule / fact payload を消費する。
resolver registration kind が `Cluster` で、activation trigger と activation
fingerprint、または fingerprint が存在しない場合の accepted pattern fallback が
checker-owned rule payload と一致する activated registration だけを発火させる。

derived fact は checker-owned `ClusterFactTable` に記録し、canonical
`ClusterFactFingerprint` で dedup する。すべての derived fact は
`ClusterFactProvenance::TraceStep` を持つ。この task は phase-6 `TypeFactTable` を直接
mutate しない。traced cluster fact を共有 type fact table に写す処理は、source-to-checker
および registration payload seam が typed subject / predicate を捏造なしで供給できるまで
deferred のままである。

task 16 は pending、rejected、malformed、recovered、unknown、non-cluster、
existential-gating、trigger mismatch、fingerprint mismatch の rule input を checker-local
diagnostic で拒否する。diagnostic は `ResolutionTrace` schema の外側に保存する。emit される
trace は cluster step、derived fact、reduction step count が zero の traversal profile を
含む。
replay は active な task-14 registration database を受け取り、derived fact を replay
する前に accepted cluster identity、resolver id、fingerprint / pattern payload、audit
key を再検証する。

task 16 は reduction step、loop detection、bounded-saturation failure、contradiction
failure、artifact JSON emission、cache reader、existential gating、proof acceptance、
opaque resolver-shell parsing を実装しない。それらは tasks 17-20 と artifact/build
integration が所有する。

### task 17: cluster loop / bound data layer

task 17 は canonical artifact schema を変えずに、task 16 の cluster closure data
layer を拡張する。explicit `ClusterRuleInput` / `ClusterFactInput` payload 上で
checker-local な loop、bound、contradiction failure handling を実装する。

builder は canonical `ClusterFactFingerprint` ごとに derivation ancestry を追跡する。
適用可能な rule が antecedent の active ancestry path 上に既にある fact を導出しようとした
場合、その rule は deterministic な `cluster_loop` diagnostic で拒否する。active ancestry
path 上にない already-derived fact の重複は通常の duplicate closure fact とし、
fingerprint equality を確認した後にだけ無視する。

`ClusterTraversalConfig` の bound は closure 中に強制する。candidate の derived depth が
`max_cluster_depth` を超える場合、または insertion が `max_generated_facts` を超える場合、
deterministic な `cluster_bound_exceeded` diagnostic で拒否する。traversal profile は
configured bound、bounded saturation に到達したか、ordering version と bound setting から
導いた stable cache-key material を記録する。拒否された candidate は `closure_facts`、
`derived_facts`、trace step へ挿入しない。
depth は explicit fact-dependency hypergraph 上で測る。input fact の depth は `0`、
antecedent を持つ derived fact の depth は `1 + max(antecedent depths)`、antecedent を
持たない cluster-generated fact の depth は `1` とする。
spec §17.7.1 は、制限された no-argument cluster adjective grammar を language-level
の停止性根拠にする。task-17 の saturation bound は防御的な implementation
diagnostic であり、成功した truncated semantics ではない。
loop、bound、contradiction failure は checker-local な `ClusterClosureOutput` status を
incomplete にする。incomplete output は fatal candidate より前に導出された fact を保持して
よいが、それらを verified closure result として export してはならない。

contradiction handling はこの seam では checker-owned のままであり、explicit payload では
spec §17.7.3 の fatal closure rule を実装する。task 17 では explicit rule payload が、
generated fact と conflict する already-visible fact fingerprint を列挙
できる。rule が発火しようとした時点で列挙された fact が存在する場合、builder は
`cluster_contradiction` を emit し、その contradictory generated fact を verified または
degraded closure fact として export しない。contradiction は verified export に対する fatal
closure result であり、checker はその closure から truncated または degraded verified fact
set を publish してはならない。shared `TypeFactTable` に対する source-derived
incompatibility check は、source-to-checker payload extraction と registration payload が
利用可能になるまで deferred のままである。

task 17 は reduction step、artifact JSON emission、cache reader、existential gating、
proof acceptance、opaque resolver-shell parsing を実装しない。それらは later task と
external integration work が所有する。

## cluster step

cluster step は architecture の `ClusterStep` field を精緻化する。

```text
ClusterStep
  source_type
  applied_cluster
  generated_attribute
  generated_type
  dependency
  source_range
```

checker は `dependency` を、ordered antecedent fact reference と applied active
registration identity として記録する。antecedent reference は step より前に visible
な fact を指さなければならない。すなわち phase-6 input fact、先行 trace-derived fact、
phase 7 に明示的に公開された local assumption、または accepted cited fact である。
antecedent がない cluster は、明示的な空 antecedent list を記録する。

各 cluster step は次を保持しなければならない。

- active checker registration id と resolver registration provenance;
- activation payload が供給する checker-owned registration fingerprint または pattern
  fingerprint;
- generated attribute / generated type fingerprint;
- multi-consequent cluster を分割した後の single-consequent rule view;
- rule とそれを trigger した fact site の source provenance;
- source type id、cluster origin module path、declaration source order、
  generated attribute id、registration fingerprint から作る audit key。

cluster step は順に replay する。replay は input fact set から開始し、すべての
antecedent が既に存在すること、active cluster registration が accepted かつ visible
であることを確認し、generated fact を追加する。replay は search を実行せず、不足
antecedent を推論せず、transitive chain を圧縮しない。

## reduction step

reduction step は architecture の `ReductionStep` field を意味変更なしに使う。

```text
ReductionStep
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
  source_range
```

kernel-replay layer は `applied_reduction`、`rule_fqn`、`source_redex`、
`target_term`、`substitution`、`discharged_guards` である。strategy-audit layer は
`enclosing_term_before`、`redex_path`、`rule_view`、`selection_key` である。これらの
name と meaning は architecture 17 と spec 23.7.7 に合わせ続ける。

reduction replay は、すでに accepted された `reducibility` registration に対する局所
rewrite instance だけを検査する。redex が rule `LHS` と match すること、target が対応する
`RHS` instance であること、各 pattern binding が valid であること、すべての type /
attribute / `such` guard が stable evidence を持つことを確認する。`such` evidence は
applicability side condition であり、rule をより specific にはしない。これは discharged
side-condition set を trace に保持することで spec §17.6.4 の normalization signature と
一致する。minimum kernel は matching rule の探索や reduction の再選択を行わない。

strategy-audit key は leftmost-innermost redex path、active rule-view fingerprint、
spec 17.6.4 が要求する specificity/FQN selection key を記録する。local rewrite を replay
できる場合でも audit key が合わなければ trace validation failure である。

### task 18: reduction step data layer

task 18 は explicit `ReductionInput` payload 上の checker-owned data layer として、
`ResolutionTrace` の reduction 側を実装する。parser rewrite、source walk、raw syntax
に対する term matching、rule search、opaque resolver-shell parsing は実行しない。
source-derived reduction payload extraction は MC-G020 と MC-G021 により deferred のままである。

reduction builder は task-14 `RegistrationDatabase` を消費し、resolver registration kind が
`Reduction`、activation trigger が input trigger と一致し、activation fingerprint または
fingerprint が存在しない場合の accepted pattern fallback が input の active rule-view
fingerprint と一致する activated resolver registration だけを記録する。pending、rejected、
recovered、malformed、unknown、unaccepted、non-reduction registration は rewrite step を
生成しない。

valid input は architecture-17 field を持つ `ReductionStep` を記録する。すなわち applied
reduction fingerprint、rule FQN、normalization 前の enclosing term、redex path、source
redex fingerprint、target term fingerprint、valid substitution binding、discharged guard
evidence、rule-view fingerprint、selection key、resolver/checker registration provenance、
source range である。substitution と guard evidence は explicit checker-owned payload である。
invalid substitution binding、type / attribute / `such` guard evidence の欠落、または
deterministic strategy-audit material と合わない selection key は diagnostic となり、
reduction step を emit しない。

in-memory checker step はさらに `required_guards` を checker-local replay-only refinement
として保持する。これにより task 18 は、opaque resolver shell から rule payload を復元せずに、
すべての required type / attribute / `such` guard に discharged evidence があることを検査できる。
これは canonical artifact schema fork ではない。artifact projection はこれを省略するか、
MC-G021 が解消された後の canonical rule payload に置き換えなければならない。

task 18 は `such` guard を applicability side condition としてだけ扱う。rewrite step を
記録する前に stable evidence が必要であり、discharged side-condition evidence は explicit
reduction trace identity の一部である。strategy-audit selection key には寄与せず、rule を
より specific にすることはできない。task 46 はこの explicit-payload trace identity の Rust
coverage を追加する。利用可能な `such` evidence による source-derived normalization result
dependence は、MC-G020/MC-G021/MC-G023 extraction と runner support が存在するまで
deferred のままである。

task-18 reduction step の replay は、active reduction registration、resolver id、rule-view
fingerprint / pattern fallback、deterministic selection key、valid substitution binding、
discharged guard evidence を検査する。replay は matching reduction を探索せず、rule を再選択
せず、trace にない追加 normalization step を適用しない。

## determinism

cluster traversal order は architecture 17 の order である。

1. source type canonical id;
2. cluster origin module path;
3. declaration source order;
4. generated attribute canonical id;
5. registration fingerprint。

checker はこの order を worklist、per-trigger candidate list、trace step id、diagnostic に
使う。worker completion order、hash-map iteration、import order、cache insertion order、
activation input order は emitted trace を変えてはならない。

cluster closure はすべての intermediate step を記録する。`A -> B -> C` の chain は、
後続 artifact が original step を content-addressed reference として保持しない限り、
2 つの step として保存しなければならない。derived fact deduplication は canonical fact
fingerprint が一致した後にだけ許される。

reduction normalization は spec §17.6.4 に従う。固定された typed term、スコープ内の
activated reduction rule set、discharged side-condition set に対して、strategy は
left-to-right rewriting、leftmost-innermost redex traversal、alpha-equivalence と binding を
考慮した matching、most-specific rule selection、残る match に対する FQN tie-break である。
reduction order は cluster expansion depth で制限しない。停止性は registration-time
simplification-order validation によって得る。

## bounds と failure

traversal profile は少なくとも次を記録する。

- schema / order version;
- maximum cluster expansion depth;
- maximum generated cluster fact count;
- bounded saturation に到達したか;
- input fact、derived fact、cluster step、reduction step、diagnostic の count。

cluster depth または generated-fact bound の超過は bounded failure である。checker は
deterministic diagnostic を emit し、degraded fact を verified closure result として
export しない。必要な closure を黙って truncate してはならない。

tasks 16-18 用に予約する failure class:

| class | meaning |
|---|---|
| `cluster_loop` | rule が active expansion stack fingerprint を再訪しようとした |
| `cluster_bound_exceeded` | configured cluster depth または generated-fact count を超過した |
| `cluster_contradiction` | derived fact が type fact table で受理できない形で conflict した |
| `invisible_registration` | pending、rejected、recovered、malformed、unaccepted registration が要求された |
| `invalid_reduction_substitution` | recorded substitution が rule pattern を instantiate しない |
| `missing_guard_evidence` | type、attribute、`such` guard に stable evidence がない |
| `strategy_audit_mismatch` | emitted reduction step が deterministic strategy audit と合わない |

これらの name は public diagnostics code space が割り当てられるまでは checker-local class
である。test では stable detail key を使ってよい。

## replay cost

trace replay は trace size に対して linear または near-linear でなければならない。
implementation は replay 前に fact fingerprint、step id、rule fingerprint、term
fingerprint から local map を作ってよいが、replay 中に global proof search、overload
resolution、cluster search を行ってはならない。

replay consumer が行ってよいこと:

- artifact と kernel certificate を validate する;
- diagnostic と `@show_resolution` で derived fact / reduction を説明する;
- incremental build 用の dependency fingerprint を計算する;
- replay した fact を VC input に含める。

replay consumer は追加の cluster fact を推論したり、追加 reduction を適用したり、不足
step を修復したりしてはならない。

## Public Enum Policy

task 31 は frontend task-25 の public-enum decision procedure をこの module に適用する。
`cluster_trace` の public checker-owned enum はすべて forward-compatible API surface であり、
`#[non_exhaustive]` を維持しなければならない。downstream consumer は wildcard または
fallback arm を保持する。checker 内部の match は、仕様化済み behavior を実装するために
現在表現されている variant へ exhaustive のままにしてよい。

| enum | decision |
|---|---|
| `ClusterClosureStatus` | 前方互換; closure outcome は artifact と export handling とともに増える可能性がある。 |
| `ResolutionTraceStep` | 前方互換; trace step family は追加の checker-owned replay event とともに増える可能性がある。 |
| `ClusterReplayStatus` | 前方互換; replay outcome は artifact/cache validation とともに増える可能性がある。 |
| `ReductionGuardKind` | 前方互換; guard category は reduction proof payload とともに増える可能性がある。 |
| `ClusterRuleKind` | 前方互換; rule family は registration semantics とともに増える可能性がある。 |
| `ClusterFactProvenance` | 前方互換; fact provenance は registration、proof、artifact source とともに増える可能性がある。 |
| `ClusterDiagnosticClass` | 前方互換; diagnostic class は public checker diagnostic code が割り当てられる前に増える可能性がある。 |
| `ClusterDiagnosticSeverity` | 前方互換; diagnostic severity policy は IDE/artifact consumer とともに増える可能性がある。 |
| `ClusterDiagnosticRecovery` | 前方互換; diagnostic recovery state は partial trace policy とともに増える可能性がある。 |

この module が所有する exhaustive public enum exception はない。

## external / deferred input

task 15 は以下を open として分類する。

- `external_dependency_gap` / `deferred`: MC-G021 はまだ real cluster / reduction step を
  block している。resolver または artifact input から、checker-ready registration
  pattern、parameter payload、accepted correctness payload、reduction `LHS` / `RHS`
  term、guard evidence、active dependency summary が得られないためである。
- `deferred`: task 15 は `src/cluster_trace.rs` を実装しない。source behavior と Rust
  test は task 16 から開始する。
- `deferred`: artifact JSON emission と cache compatibility は architecture 17 を消費する
  build/artifact task が所有し、checker-local schema fork ではない。
- `test_gap`: active `.miz` cluster/reduction semantic fixture は、tasks 16-18 が
  checker-owned payload seam を持つまで deferred のままである。より広い corpus coverage は
  task 29 が所有する。

## planned tests

task 16:

- closure fixture が replayable derived fact を生成する;
- pending、rejected、recovered、malformed、unaccepted registration は発火しない;
- registration/input permutation をまたいで同じ input が同じ trace と diagnostic を emit する;
- subtype-compatible conditional cluster がすべての antecedent fact reference を記録する;
- transitive chain がすべての intermediate step を保持する。

task 17:

- direct / indirect loop が `cluster_loop` で停止する;
- depth と generated-fact bound が traversal profile と cache key material で可視である;
- bounded saturation は degraded verified fact を export しない;
- incompatible derived fact は stable `cluster_contradiction` diagnostic を emit し、
  contradiction に対する verified / degraded closure fact を export しない;
- duplicate derived fact は fingerprint equality の後にだけ無視される。

task 18:

- reduction step が enclosing-term fingerprint、redex path、source redex、target term、
  substitution、guard evidence、rule FQN、rule-view fingerprint、selection key、
  source provenance を記録する;
- invalid substitution、missing guard evidence、mismatched strategy-audit key を診断する;
- `such` side condition は applicability-only である;
- pending、rejected、recovered、malformed、unaccepted reduction は term を rewrite しない;
- replay は successful reduction step id を報告し、追加 rewrite を探索・追加しない。

後続 task:

- task 29 は checker-owned cluster/reduction payload extraction と active runner が
  存在するまで、`.miz` cluster/reduction coverage obligation を deferred として記録する;
- task 30 は crate-level determinism suite の一部として、cluster trace ordering の
  cross-checker determinism regression coverage を追加する;
- task 32 は source、docs、tests がまだ一致していることを audit する。
