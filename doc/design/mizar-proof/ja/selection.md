# mizar-proof Selection Spec

> 正本は英語です。英語版:
> [../en/selection.md](../en/selection.md)。

## 目的

selection module は、1 つの obligation に対する正規化済みかつ policy 分類済みの
candidate set から 1 つの proof evidence winner を選ぶ。決定的な winner ordering と、
selection が export する安定した reuse metadata を所有する。

この module は proof を accept しない。ATP backend、kernel check、SAT solving、
proof search、premise selection、substitution invention、status projection、
witness staging、cache lookup、artifact commit を実行しない。

## 入力

selection は、すでに前段が生成した immutable record を消費する:

- 正規化された obligation identity と encoded problem hash;
- active `VerifierPolicy` と `PolicyFingerprint`;
- `ProofPolicyEvaluator` から来る policy decision;
- 明示的な kernel evidence origin と accepted kernel evidence hash に対になった
  accepted kernel result;
- deterministic built-in discharge hash;
- externally attested admission record;
- open-obligation explanation;
- portfolio owner が渡す backend profile metadata と evidence-format metadata。

selection 開始前に入力は complete でなければならない。arrival order、backend
completion order、runtime duration、wall-clock time、worker id、process id、
temporary path は winner identity や tie-break に入らない。

## Winner Classes

selection は tie-break key を適用する前に winner class で candidate を比較する。

| Winner class | Eligible candidate | Trusted? | Notes |
|---|---|---:|---|
| `KernelVerified` | active policy を満たす formula/substitution evidence の accepted kernel result。 | yes | 最上位 class。trusted `used_axioms` は accepted kernel result からだけ copy してよい。 |
| `DischargedBuiltin` | kernel が accept した kernel-owned primitive evidence を含む、deterministic built-in evidence に対する accepted `KernelCheckResult`。 | yes | ATP `KernelVerified` とは別である。reuse validation のため deterministic discharge hash を export する。 |
| `PolicyPermittedExternal` | `may_win_selection = true` かつ `require_kernel_certificates = false` の external admission。 | no | trusted ではなく、trusted `used_axioms` を供給しない。 |
| `PolicyAssumed` | active policy が許可し、`require_kernel_certificates` に阻止されていない明示的な policy assumption。 | no | non-trusted policy status のみ。trusted witness material や `used_axioms` を供給しない。 |
| `PolicyOpen` | よりよい candidate を生成できない後に active policy が許す open obligation。 | no | 安定した explanation ref だけを持つ。 |
| `Rejected` | kernel rejection、evidence-format rejection、policy rejection、invalid selection input。 | no | proof winner ではない。eligible な non-rejected winner が存在しない場合だけ primary diagnostic result として選ばれる。 |
| `DiagnosticOnly` | backend diagnostic、backend proof payload、cache record、timing、counterexample、unsupported payload。 | no | winner にはならない。 |

`require_kernel_certificates = true` のとき、external record が存在していても
`PolicyPermittedExternal` は eligible winner ではない。external evidence は
`ExternalEvidenceAdmission` に従って記録されてよいが、winner selection はそれを
すべての kernel-checkable path と release publication acceptance より下位に扱う。
`PolicyAssumed` も `require_kernel_certificates` の下では勝てない。

## Deterministic Ordering

winner ordering は次の順である:

1. active policy を満たす `KernelVerified`。
2. active policy を満たす `DischargedBuiltin`。
3. external admission が winning を許し、kernel certificate が不要な場合だけ
   `PolicyPermittedExternal`。
4. policy assumption recording が有効で、kernel certificate が不要な場合だけ
   `PolicyAssumed`。
5. policy が open output を許し、よりよい candidate をこれ以上生成できない場合だけ
   `PolicyOpen`。
6. 最良の diagnostic rejection。

`DiagnosticOnly` candidate は winner selection から除外されるが、その diagnostic は
selected result に添付されてよい。

## Tie-Break Keys

同じ winner class 内では、candidate は次の安定 key tuple で sort される:

1. winner-class rank;
2. backend profile priority（小さい数値を優先）;
3. certificate/evidence-format priority（小さい数値を優先）;
4. encoded problem hash bytes;
5. policy profile id;
6. evidence payload hash または deterministic discharge hash;
7. provenance hash;
8. 安定した candidate source id。

tuple は lexicographic に比較する。selectable または rejected candidate はすべて、
安定した candidate source id を持たなければならない。producer がそれを渡せない
場合、selection input は invalid であり、iteration order に silent fallback せず
policy diagnostic material として報告しなければならない。backend 由来でない
candidate は backend-profile priority の最大値 sentinel を使う。
certificate/evidence format を持たない candidate は evidence-format priority の最大値
sentinel を使う。missing optional hash は tagged optional field
（`present(hash)` が `missing` より前）として present hash より後に sort する。
key 内の collection は hash 前に安定 byte ordering で sort する。同じ安定 id と
同一の canonical payload を持つ duplicate record は coalesce してよい。同じ id と
異なる canonical payload の組み合わせは invalid selection input である。

次の値は tie-break tuple に入れてはならない:

- candidate arrival order;
- backend completion order;
- backend runtime duration;
- wall-clock time;
- process id、worker id、thread id、temporary path、scheduler priority;
- backend stdout/stderr bytes または diagnostic wording;
- cache hit/miss state または cache lookup time。

## Rejected Candidate Ordering

eligible な non-rejected winner が存在しない場合、selection は primary diagnostic
result を提供するためだけに rejected candidate を 1 つ選ぶ。rejected candidate は
別の deterministic diagnostic tuple を使う:

1. rejection source rank: kernel rejection、evidence-format rejection、policy
   rejection、invalid selection input;
2. 利用可能な場合は architecture-19 failure category rank;
3. 安定した severity rank;
4. 安定した reason code;
5. encoded problem hash bytes;
6. evidence payload hash または deterministic discharge hash;
7. provenance hash;
8. 安定した candidate source id。

backend diagnostic、cache record、timing、unsupported payload は secondary
diagnostic ref として添付してよい。その場合は安定した diagnostic ref hash で
sort する。ただし、それらは kernel rejection や policy rejection record より上位に
ならず、proof winner にも決してならない。

candidate set が空、またはすべての input が `DiagnosticOnly` の場合、selection は
proof winner を合成したり input order で選んだりせず、`NoSelectableEvidence`
diagnostic outcome を返す。この outcome は selected candidate id を持たず、
trusted `used_axioms` も持たない。obligation identity、encoded problem hash、
policy fingerprint、`no_selectable_evidence` reason code から導出される安定した
diagnostic result id を持つ。diagnostic ref は安定した diagnostic ref hash で sort する。

## Selected Reuse Metadata

selection は、proof reuse と後段の witness publication が使う安定 metadata を export する:

| Field | Meaning |
|---|---|
| `selected_class` | deterministic ordering が選んだ winner class。 |
| `policy_fingerprint` | active `PolicyFingerprint`。 |
| `encoded_problem_hash` | encoded obligation の安定 hash。 |
| `selected_evidence_hash` | class に応じた kernel evidence payload hash、external evidence hash、policy-assumption source hash、または open explanation hash。 |
| `selected_proof_witness_hash` | artifact witness publication が利用可能な場合だけの proof witness ref hash。 |
| `deterministic_discharge_hash` | `DischargedBuiltin` の deterministic built-in discharge hash。 |
| `external_admission_status` | `PolicyPermittedExternal` の external publication status。 |
| `proof_witness_publication` | selected class に対する `available`、`external_dependency_gap`、または `not_applicable`。 |
| `tie_break_key_hash` | 実際の tie-break tuple の安定 hash。 |

`KernelVerified` では、proof reuse は accepted kernel result と trusted `used_axioms` に
依存してよい。`DischargedBuiltin` では、reuse は deterministic discharge hash と
accepted built-in/kernel evidence path に依存する。`PolicyPermittedExternal` と
`PolicyAssumed`、`PolicyOpen` では、reuse metadata は validation predicate にすぎず
trusted acceptance にならない。

現在の artifact witness reference は `discharged_builtin` publication をまだ support しない。
artifact schema gap が閉じるまで、selection は deterministic discharge hash を export してよいが、
その class では selected proof witness hash を export してはならない。proof-witness
publication は `external_dependency_gap` として mark しなければならない。

## Result Shape

task 6 implementation は次を持つ安定 result を expose しなければならない:

- winner または primary rejected diagnostic candidate が存在する場合の selected
  candidate id;
- selected winner class、または selectable/rejected candidate がない場合の
  `NoSelectableEvidence`;
- selected reuse metadata;
- non-winning candidate の ordered diagnostic ref;
- trusted `used_axioms` が accepted kernel result から利用可能かどうかの flag。

selected result は trusted `used_axioms` を合成してはならない。`KernelVerified` または
`DischargedBuiltin` として選ばれた accepted kernel result を参照する場合だけ、
trusted `used_axioms` を保持してよい。selection が使う trusted marker は accepted
kernel evidence hash に bind され、selected candidate の evidence hash と一致しなければならない。

## Artifact Proof Selection Merge

artifact merge は、`VcId` を key にした portfolio selection と phase-12 built-in
discharge selection を消費する。canonical `VcId` order で `VcId` ごとに 1 つの
`ArtifactProofSelection` を emit する。

source/class compatibility は merge validation の一部である。Portfolio input は
`KernelVerified`、policy-permitted external、policy-assumed、policy-open、rejected、
`NoSelectableEvidence` outcome を publish してよいが、`DischargedBuiltin` を publish
してはならない。Built-in discharge input は `DischargedBuiltin`、または built-in
discharge evidence の失敗・不在を表す diagnostic outcome（`Rejected` と
`NoSelectableEvidence`）を publish してよいが、`KernelVerified`、policy-permitted
external、policy-assumed、policy-open outcome を publish してはならない。invalid
source/class pair は artifact status projection の前に reject される。

merge ordering は通常の selection と同じ winner-class rank を使う:
`KernelVerified`、`DischargedBuiltin`、policy-permitted external、policy
assumption、policy-open、rejected、`NoSelectableEvidence` の順である。同じ class
内では、selected result の `tie_break_key_hash`、次に安定した source rank
（`Portfolio` が `BuiltinDischarge` より前）を比較する。同じ source から同じ
`VcId` へ duplicate input がある場合は invalid merge input である。

merge layer は final artifact status projection、witness staging、artifact manifest
write、ATP、kernel check、cache record の trust を行わない。`KernelVerified` と
`DischargedBuiltin` を distinct な trusted class として保ち、policy-permitted external、
policy-assumed、open、rejected、no-selectable outcome を status projection のための
non-trusted class として保持する。

## Deferred Integrations

- `DischargedBuiltin` の artifact witness publication は、`mizar-artifact` がその witness
  class を support するまで `external_dependency_gap` のままである。
- cache lookup と proof reuse validation は downstream integration work のままである。
  selection は安定 metadata だけを export し、cache を query したり trust したりしない。
- ATP early-stop は将来 winner-class information を消費してよいが、selection 自体は
  backend process を stop しない。
