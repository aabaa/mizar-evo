# Module: proof_reuse

> 正本は英語です。英語版: [../en/proof_reuse.md](../en/proof_reuse.md)。

Status: task 10 で仕様化し、task 11 で実装した。

## Purpose

`proof_reuse` は、proof-related cache record を現在の build の最適化として使えるかを
検証する。

この module は `mizar-proof` が export する validation metadata を消費する。proof
policy、deterministic winner selection、proof status projection、witness publication、
kernel checking、ATP 実行、artifact manifest commit は所有しない。validation 成功が言えるのは、
cached proof-related phase output が現在の入力と互換であることだけである。proof を受理せず、
evidence を kernel-verified と mark せず、trusted `used_axioms` を作らない。

trusted proof acceptance は、proof/status owner layer が消費する `mizar-kernel`
`KernelCheckResult` だけに由来する。

正式な `mizar-cache` scaffold より前に書かれた historical な `mizar-proof` closeout/audit
document は、cache support を absent または design-only と記述している場合がある。それらは proof
crate の closeout 時点の状態を記録したものである。現在の cache ownership と proof-reuse
validation については、本 document と `mizar-cache` crate plan を権威とする。

## Inputs

proof reuse validation は、現在の cache key と `mizar-proof` が export した proof metadata から
導かれる immutable request を消費する:

- `ObligationAnchor`;
- obligation fingerprint;
- canonical VC fingerprint;
- canonical local-context fingerprint;
- dependency-slice fingerprint;
- active verifier policy fingerprint;
- cache-side proof-reuse schema version と、`mizar-proof` が export する
  dependency/proof-reuse schema compatibility;
- `mizar-proof` が export した selected proof class と proof-evidence identity;
- `KernelVerified` selection で利用可能な selected proof witness hash;
- `DischargedBuiltin` selection の deterministic discharge hash;
- status projection が export する proof-reuse validation hash;
- selected evidence hash、selected candidate provenance hash、selection reason、
  tie-break key hash;
- trusted class について `mizar-proof` が export する場合の trusted used-axioms reference hash;
- artifact/cache boundary が供給する dependency artifact availability/hash metadata;
- miss 説明専用の diagnostic/explanation ref。

validation request は `CacheRecord` を参照してよいが、record は proof authority ではない。
record は request と比較する bytes と metadata を供給するだけであり、trusted status を定義しない。

## 公開 enum policy

この module が所有する exhaustive public enum exception はない。すべての public
enum は `#[non_exhaustive]` とする。downstream match は wildcard arm を持たなければならず、
新しい variant は proof owner と cache spec が reuse behavior を定義するまで miss または
diagnostic のままでなければならない。

| Public enum | 前方互換性の決定 |
|---|---|
| `ProofReuseValidationOutcome` | `#[non_exhaustive]`; unknown outcome を proof-reuse hit として扱ってはならない。 |
| `ProofReuseMissReason` | `#[non_exhaustive]`; 新しい miss reason は diagnostic-only であり proof authority ではない。 |

## Reusable Classes

validation predicate は class-aware である:

| Exported proof class | Reuse requirement | cache が trust するか |
|---|---|---:|
| `KernelVerified` | selected proof witness hash、selected evidence hash、proof-reuse validation hash、`ObligationAnchor`、canonical VC/local-context/dependency-slice fingerprint、policy fingerprint、schema version、dependency artifact がすべて一致する。 | no |
| `DischargedBuiltin` | deterministic discharge hash、selected evidence hash、proof-reuse validation hash、`ObligationAnchor`、canonical VC/local-context/dependency-slice fingerprint、policy fingerprint、schema version、dependency artifact がすべて一致する。 | no |
| `PolicyPermittedExternal` | `mizar-proof` が現在 policy の selected class として export した場合だけ、non-trusted validation metadata として比較または記録してよい。complete proof-reuse hit ではない。 | no |
| `PolicyAssumed` | policy metadata としてだけ比較または記録してよい。complete proof-reuse hit ではない。 | no |
| `PolicyOpen`, `Rejected`, `NoSelectableEvidence` | diagnostic 用にだけ比較または記録してよい。complete proof-reuse hit ではない。 | no |

`proof_reuse` は `mizar-proof` が export する completeness predicate を尊重しなければならない。
現在の upstream contract では、proof recomputation を skip する complete proof-reuse hit を
生成できるのは `KernelVerified` と `DischargedBuiltin` だけである。non-trusted class は
diagnostic または status-owner recomputation 用 metadata にとどまり、proof-reuse hit 目的では
miss として扱わなければならない。`proof_reuse` は export された class を変えずに保存しなければ
ならず、non-trusted class を accepted と再解釈してはならず、別の winner を選んではならず、policy
が publication を許可するかを決めてはならない。

より広い VC、artifact、witness、checker schema compatibility は、owning producer が structured
cache-adapter field を公開するまで、exact `CacheKey`/record-store lookup と `mizar-proof`
validation hash の一部に残る。したがって task 11 は、現在 `mizar-proof` が structured に export
する schema field と cache-side proof-reuse schema guard だけを比較し、placeholder producer-adapter
schema API を発明しない。

## Validation Predicate

validation は、必要な field がすべて厳密に一致する場合だけ成功する:

1. cache key schema と proof-reuse schema version が supported;
2. `ObligationAnchor` と obligation fingerprint が一致する;
3. canonical VC fingerprint が一致する;
4. canonical local-context fingerprint が一致する;
5. dependency-slice fingerprint が一致し complete である;
6. verifier policy fingerprint と policy-compatibility field が一致する;
7. selected proof class と proof-evidence identity が一致する;
8. `KernelVerified` では selected proof witness hash が一致する;
9. `DischargedBuiltin` では deterministic discharge hash が一致する;
10. `mizar-proof` が供給する場合、selected evidence hash、selected provenance hash、
    selection reason、tie-break key hash が一致する;
11. `mizar-proof` が export する場合、trusted used-axioms reference hash が一致する;
12. proof-reuse validation hash が一致する;
13. dependency artifact availability と記録された domain/digest check が成功する;
14. `uncacheable`、incomplete-footprint、unsupported-schema、unknown toolchain、
    incompatible policy marker が存在しない。

trusted used-axioms reference hash は validation field でしかない。proof-reuse validation hash に
含まれてよいが、cache validation は cache data から trusted `used_axioms` を expose してはならず、
それを合成しようとする record を reject しなければならない。

必要な入力が missing、malformed、unknown、unsupported、mismatched の場合はすべて cache miss である。
miss は再計算へ退化する。proof acceptance へは決して退化しない。

## Determinism

validation predicate は以下に依存してはならない:

- cache hit/miss timing;
- record arrival order または write order;
- file modification time;
- worker id、thread id、process id、temporary path、scheduler priority;
- backend runtime duration、stdout/stderr order、backend log wording;
- stable diagnostic/explanation ref を除く diagnostic ordering;
- witness staging time または artifact manifest commit timing。

複数の cache record や proof metadata candidate が存在する場合でも、`proof_reuse` は winner を
選ばない。upstream `mizar-proof` metadata が選んだ candidate を検証し、ambiguity または conflict は
miss として扱う。

## Failure Semantics

validation は以下で miss を返す:

- missing `ObligationAnchor`;
- stale または mismatched obligation、VC、local-context、dependency-slice fingerprint;
- `KernelVerified` の selected proof witness hash 欠落;
- `DischargedBuiltin` の deterministic discharge hash 欠落;
- witness/discharge hash mismatch;
- selected evidence または proof-evidence identity mismatch;
- export される場合の trusted used-axioms reference hash の欠落または不一致;
- proof-reuse validation hash mismatch;
- unsupported proof-reuse schema;
- unknown toolchain/schema compatibility;
- incompatible verifier policy;
- incomplete dependency footprint;
- missing または mismatched dependency artifact;
- 明示的な uncacheable marker;
- externally attested、assumed、open、rejected、no-selectable metadata が trusted
  acceptance として提示されること。

miss reason は cache behavior の diagnostic だけである。published diagnostic を並べ替えず、
proof status に影響しない。

## Output Contract

validation 成功は以下を expose してよい:

- `mizar-proof` が export した validated proof-reuse class そのもの;
- 一致した witness hash または deterministic discharge hash;
- 一致した validation hash と schema version;
- reuse decision を説明する diagnostic ref。

以下を expose してはならない:

- 新しい `KernelCheckResult`;
- cache data から作った kernel-verified status;
- cache data から作った trusted `used_axioms`;
- `mizar-proof` が選んでいない selected winner;
- committed witness publication reference;
- artifact publication eligibility。

downstream consumer は proof status projection と publication decision について、引き続き
proof/status/artifact owner に問い合わせなければならない。

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `PROOFREUSE-G001` | `external_dependency_gap` | `mizar-build` は現在 scheduler integration seam を所有するが、end-to-end proof-reuse scheduling はここでは未接続である。task 11 は metadata を local に検証し、placeholder scheduling は追加しない。 |
| `PROOFREUSE-G002` | `external_dependency_gap` | `mizar-ir` は現在 cache-adapter validation boundary を所有する。proof reuse validation は IR adapter API や rehydration shortcut を発明しない。 |
| `PROOFREUSE-G003` | `external_dependency_gap` | artifact committed witness publication token は artifact owner のまま。`proof_reuse` は selected witness hash を比較してよいが、committed publication ref を合成してはならない。 |
| `PROOFREUSE-G004` | `external_dependency_gap` | artifact witness schema が distinct trusted class を持つまで、`DischargedBuiltin` artifact witness publication は unsupported のまま。reuse は deterministic discharge hash だけを使う。 |
| `PROOFREUSE-G005` | `external_dependency_gap` | task 20 は crate-owned cache lookup と proof-reuse validation contract を cover する。cross-crate clean/incremental equivalence は `mizar-build` scheduler と artifact publication integration に依存して残る。 |

## Tests For Task 11

task 11 は少なくとも以下を cover しなければならない:

- 一致する `KernelVerified` metadata が validate されること;
- 一致する `DischargedBuiltin` metadata が deterministic discharge hash により validate されること;
- 必要な各 component の欠落または mismatch が reuse を block すること:
  `ObligationAnchor`、obligation fingerprint、canonical VC fingerprint、local-context
  fingerprint、dependency-slice fingerprint、selected witness hash、deterministic
  discharge hash、selected proof class、selected proof-evidence identity、selected evidence hash、
  selected candidate provenance hash、selection reason、tie-break key hash、export される場合の
  trusted used-axioms reference hash、proof-reuse validation hash、policy、schema version、
  dependency artifact;
- incomplete dependency footprint、unsupported schema、unknown toolchain、明示的な
  uncacheable marker が miss になること;
- externally attested、policy-assumed、open、rejected、no-selectable metadata が
  kernel-verified または trusted `used_axioms` にならず、trusted `used_axioms` を合成しようとする
  record や trusted used-axioms reference hash を合成しようとする record が reject されること;
- upstream proof-reuse completeness が尊重されること: current non-trusted class を含む、
  exported completeness predicate が false の class は、metadata が一致しても proof-reuse hit
  目的では miss になること;
- cache record arrival order、write order、cache hit/miss timing が validation に影響しないこと;
- lint または source-surface guard により、task 11 が `mizar-build` scheduler stub、
  `mizar-ir` adapter stub、artifact committed publication placeholder API、witness publication
  shortcut を追加しないことを示すこと。

## Non-Goals

この module は ATP 実行、kernel 呼び出し、proof policy evaluation、proof winner selection、
proof status projection、witness publication、artifact commit、cluster-db update、build scheduling、
IR payload adaptation を行わない。
