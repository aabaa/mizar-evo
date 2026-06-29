# mizar-proof Status Projection

> 正本: [../en/status.md](../en/status.md)。

## 目的

`mizar-proof` は、決定的な proof selection を artifact 向けおよび diagnostics
向けの proof status record へ射影する責務を持つ。

status projection は proof acceptance ではない。trusted acceptance と trusted
`used_axioms` は、status が `KernelCheckStatus::Accepted` であり、`KernelVerified`
または `DischargedBuiltin` として選ばれた `mizar-kernel` の `KernelCheckResult`
だけに由来する。externally attested evidence、backend diagnostic、backend が報告した
axiom list、proof cache record、witness metadata、policy assumption は、
kernel-verified status や trusted `used_axioms` にならない。

## 入力

projection は次を消費する:

- deterministic selection merge が作る `ArtifactProofSelection` stream;
- 現在の build に対する active `VerifierPolicy` または同等の publication profile;
- VC/artifact producer が供給する stable obligation identity:
  `VcId`、obligation id、`ObligationAnchor`、obligation fingerprint、source range
  または diagnostic anchor、canonical VC fingerprint、canonical local-context
  fingerprint、dependency-slice fingerprint、policy fingerprint;
- selected trusted class に対し、selected accepted evidence hash に bind された
  optional trusted kernel result reference;
- policy、kernel、ATP、diagnostics owner が作った optional diagnostic または
  explanation reference。

projection は stale または不一致な入力を reject しなければならない。`VcId`、
`ObligationAnchor`、source range、arrival order、completion time、runtime duration、
backend profile runtime、cache hit だけは proof identity ではなく、status reuse に
十分ではない。

## 状態モデル

internal projection result は selection winner class と projected obligation status を
分離して保持する:

| Winner class | Projected obligation status | Trusted | Notes |
|---|---|---:|---|
| `KernelVerified` | `accepted` | yes | 対応する accepted kernel result が必要である。final artifact publication で accepted として publish するには、publish 可能な kernel witness reference も必要である。 |
| `DischargedBuiltin` | `accepted` | yes | 対応する accepted kernel result が必要である。artifact schema support が存在するまで final artifact witness publication は `external_dependency_gap` のままである。 |
| `PolicyPermittedExternal` | `externally_attested` | no | policy-controlled evidence のみ。trusted `used_axioms` を持たない。 |
| `PolicyAssumed` | `policy_assumed` internal status | no | accepted status や externally attested status と区別したままにする。現在の artifact schema にはこの public obligation status がないため、後続 schema が追加するまでは artifact publication は `external_dependency_gap` である。 |
| `PolicyOpen` | `open` | no | 利用可能なら explanation reference を持つ。 |
| `Rejected` | `rejected` | no | 選択された rejection または policy/kernel diagnostic reference を持つ。 |
| `NoSelectableEvidence` | `open` または `rejected` | no | active policy が `AllowPolicyOpen` で open obligation の publish を許す場合だけ `open`。`RecordDiagnostic` は diagnostics-only であり `rejected` に射影する。それ以外も `rejected`。no-selectable-evidence explanation を持つ。 |

`not_required` は proof selection を必要としない producer-owned obligation のために
予約される。`ArtifactProofSelection` からは emit しない。

projection は `DischargedBuiltin` を `KernelVerified` に潰してはならず、
`PolicyAssumed` を `externally_attested` に潰してはならず、non-trusted status を
`accepted` に昇格させてはならない。

## Trusted `used_axioms`

trusted `used_axioms` projection は、次をすべて満たす場合だけ許可される:

1. selected winner class が `KernelVerified` または `DischargedBuiltin` である;
2. selected proof selection が trusted `used_axioms` availability を報告している;
3. projection input が accepted kernel result、またはそこから導出された trusted
   kernel-owned reference を含む;
4. accepted kernel evidence hash が selection の selected evidence hash と一致する;
5. kernel result が policy-tainted ではなく、status が `Accepted` である。

これらが成り立つ場合、projection は kernel-owned accepted result が供給した
trusted used-axiom reference または ordered axiom list をそのまま公開してよい。
projection は backend-reported axiom list、externally attested citation、cache
dependency record、diagnostic hint、policy-assumption dependency を混ぜてはならない。

すべての non-trusted status では trusted `used_axioms` は absent である。untrusted
diagnostic suggestion は diagnostics-owned reference としてだけ持てるが、accepted
dependency fact ではない。

## Diagnostics And Explanations

すべての non-accepted projection は安定した explanation surface を持つ:

- `externally_attested` は external admission status と policy fingerprint を記録する;
- `policy_assumed` は policy assumption reason と policy fingerprint を記録する;
- `open` は open-obligation reason と、利用可能なら best diagnostic reference を記録する;
- `rejected` は policy rejection、evidence rejection、kernel rejection、backend
  exhaustion、invalid selection input の failure layer を記録する;
- `NoSelectableEvidence` は生成された no-selectable-evidence diagnostic result id を記録する。

diagnostic ordering は architecture 19 に従う: source identity、source range、
phase order、severity、diagnostic code、stable detail key である。parallel completion
order、backend runtime、cache lookup timing は diagnostic ordering に参加しない。

## Artifact Projection

projection は stable projection data からだけ artifact obligation field を埋めてよい:

- `status`;
- trusted accepted witness が publish 可能な場合の `accepted_witness_obligation_id`;
- non-accepted outcome の `diagnostic_ref` または explanation reference;
- artifact consistency check に使う policy fingerprint と obligation fingerprint。

現在の `mizar-artifact` `ProofWitnessRef` schema version `2.0` は、
`kernel_verified` formula/substitution evidence に対する trusted `ProofWitnessRef`
だけを受理する。
そのため:

- `KernelVerified` は対応する witness reference が利用可能な場合だけ accepted
  artifact status を publish できる;
- `DischargedBuiltin` は accepted internal projection のままだが、まだ trusted
  artifact witness ref を publish できない; その deterministic discharge hash は
  現行 artifact schema では internal projection および proof-reuse metadata に留まり、
  accepted artifact obligation field として書いてはならない;
- `PolicyAssumed` は現在の `ObligationStatus` では lossless に表現できない。

これらは integration gap であり、placeholder witness を emit したり status 名を
置き換えたりする許可ではない。

## Proof Reuse Metadata

status projection は proof reuse の validation metadata を export する:

- selected winner class;
- projected obligation status;
- `ObligationAnchor`;
- obligation fingerprint;
- canonical VC fingerprint;
- canonical local-context fingerprint;
- dependency-slice fingerprint;
- policy fingerprint;
- selected evidence hash;
- publish 可能な場合の selected proof witness payload artifact hash
  （`witness_artifact_hash`）;
- 存在する場合の deterministic discharge hash;
- 存在する場合の trusted used-axiom reference hash;
- 存在する場合の external admission status;
- selected candidate id、selected-candidate provenance hash、selection reason、
  tie-break key hash を含む matching proof-evidence identity;
- artifact/cache boundary が供給する場合の dependency artifact fingerprint と
  compatible な dependency/proof-reuse schema version;
- diagnostic または explanation reference hash。

この metadata は cache validation predicate である。proof authority ではない。
reuse にはさらに、matching `ObligationAnchor`、canonical VC fingerprint、
canonical local-context fingerprint、dependency-slice fingerprint、compatible verifier
policy、matching proof evidence、compatible referenced dependency artifact と schema が
必要である。cache record は externally attested、assumed、open、rejected、
no-selectable outcome を trusted status に昇格させられず、trusted `used_axioms` も
合成できない。

status projection は上記すべての field に対する安定した proof-reuse validation hash も
export する。将来の `mizar-cache` は、その hash と structured field を reuse
predicate として比較してよいが、一致は cache validation 後に再計算を省けることを
示すだけである。dependency artifact/schema compatibility の欠落、policy
incompatibility、witness hash mismatch、deterministic discharge mismatch、
proof-evidence identity mismatch は miss である。一致しても `ExternallyAttested`、
`PolicyAssumed`、`Open`、`Rejected`、`NoSelectableEvidence` が `Accepted` に
昇格することはなく、trusted `used_axioms` も作られない。

complete な proof-reuse predicate は class-aware である。`KernelVerified` には
selected proof witness hash が必要であり、`DischargedBuiltin` には deterministic
discharge hash が必要である。non-trusted class は exported metadata のままで、
complete proof-reuse hit にはならない。

## Deferred And External Dependency Gaps

| Gap | Classification | Handling |
|---|---|---|
| `STATUS8-G001` | `external_dependency_gap` | 現在の artifact obligation status には distinct な public `policy_assumed` value がない。projection は内部で区別を保ち、collapse せず artifact publication を defer する。 |
| `STATUS8-G002` | `external_dependency_gap` | 現在の `ProofWitnessRef` trusted reader は `DischargedBuiltin` witness publication を reject する。projection は trusted status と deterministic discharge hash を記録してよいが、witness publication は deferred のままである。 |
| `STATUS8-G003` | `deferred` | diagnostics、artifact emission、manifest commit、cache lookup、ATP early-stop integration は後続 task がこの projection を消費する。この spec は stable metadata だけを定義する。 |

## Public Enum Policy

task 14 は public-enum forward-compatibility procedure をこの module に適用する。
すべての public status-projection enum は downstream-facing API surface であり、
`#[non_exhaustive]` を維持しなければならない。downstream consumer は wildcard match
arm を保つ。artifact-facing status enum はさらに、新しい variant を publish したり
現在の artifact field に map したりする前に artifact schema compatibility review を必要とする。

| Enum | Compatibility decision |
|---|---|
| `TrustedUsedAxiomsError` | forward-compatible |
| `ProjectedProofStatus` | forward-compatible |
| `CurrentArtifactObligationStatus` | forward-compatible with artifact compatibility review |
| `ArtifactPublicationGap` | forward-compatible with artifact compatibility review |
| `ArtifactStatusPublication` | forward-compatible with artifact compatibility review |
| `StatusProjectionError` | forward-compatible |

No exhaustive public enum exceptions are owned by this module.

## Non-Goals

status projection は ATP backend 実行、SAT solving、kernel 呼び出し、substitution
invention、premise selection、proof cache query、witness の stage/publish、artifact
manifest write、proof acceptance を行わない。
