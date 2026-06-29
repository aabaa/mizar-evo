# mizar-proof Policy Spec

> 正本は英語です。英語版:
> [../en/policy.md](../en/policy.md)。

## 目的

policy module は、evidence production の上位、artifact publication の下位にある
verifier-policy decision を定義する。proof evidence の分類、kernel checking を
schedule してよいかの判定、外部認証 evidence を policy evidence として許可または
拒否する規則、安定した `PolicyFingerprint`、policy-driven early-stop query を所有する。

この module は proof acceptor ではない。trusted acceptance と trusted
`used_axioms` は、status が `Accepted` である `mizar-kernel`
`KernelCheckResult` だけから来る。

## 入力

policy evaluator は、他 crate から来る正規化された immutable record を消費する:

- この crate が所有する正規化された evidence-origin wrapper と対になった
  accepted または rejected `mizar-kernel` result;
- untrusted な `mizar-atp` portfolio candidate と backend diagnostic;
- deterministic な `mizar-vc` built-in discharge record と discharge hash;
- externally attested evidence record;
- open-obligation explanation;
- explicit policy-assumption record と source ref;
- active verifier policy setting。

cache metadata は cache owner が渡す validation input としてのみ観測してよい。
cache hit は proof authority ではなく、candidate class を変えない。

`KernelCheckResult` は、それ単体では checked evidence が ATP formula/substitution
evidence、built-in discharge evidence、kernel primitive のどれに由来するかを
識別しない。policy module はその kernel-checkable origin を明示的な normalized
input として受け取らなければならず、arrival order、backend identity、checked import
contents、diagnostics、used-axiom list から推測してはならない。externally attested
evidence は separate policy input のままであり、kernel-result origin ではない。

## Verifier Policy Settings

active policy は、次の policy-relevant field を持つ安定 record である:

| Field | Meaning |
|---|---|
| `schema_version` | fingerprint と diagnostic に含める policy schema version。 |
| `profile_id` | `release`、`development`、`interactive` などの安定 policy profile id。 |
| `build_mode` | release、development、interactive/open-editor mode。 |
| `require_kernel_certificates` | accepted publication に kernel-accepted formula/substitution evidence または accepted built-in kernel evidence を要求する。 |
| `external_evidence` | externally attested evidence を reject するか、development evidence として記録するか、non-trusted winner として policy が許可するか。 |
| `open_obligation` | open obligation を reject するか、diagnostics only として記録するか、現在の build mode で policy-open output として許すか。 |
| `policy_assumption` | 明示的な policy assumption を reject するか、non-trusted policy status として記録可能にするか。 |
| `kernel_evidence_formats` | schedule 可能な kernel-checkable evidence format。現在は formula/substitution evidence と built-in kernel evidence representation。 |
| `checker_schema_version` | policy が要求する kernel/proof checker schema version。 |

### Build Modes

release mode は default で strict である:

- `require_kernel_certificates = true`;
- externally attested evidence は winner にならない;
- open obligation は publication では reject される。

development mode は externally attested evidence と open obligation を
policy/development evidence として記録してよいが、それらの status は non-trusted のままである。

interactive mode は diagnostics と LSP feedback のために obligation を open のまま保持してよい。
それでも open や external evidence を trusted proof status として射影してはならない。

## Candidate Policy Classes

`CandidatePolicyClass` は winner selection 前に使う policy-facing class である。
class の意味的な順序は selection task が定めるが、policy classification 自体は
決定的で、arrival order に依存してはならない。

| Class | Source | Trusted? | Notes |
|---|---|---:|---|
| `KernelVerified` | ATP formula/substitution evidence に対する accepted kernel result。 | yes | kernel result から trusted `used_axioms` を伝播してよい。 |
| `DischargedBuiltin` | built-in discharge evidence または allowed kernel primitive evidence に対する accepted kernel result。 | yes | `KernelVerified` とは別に射影する。ATP kernel verification へ潰してはならない。 |
| `KernelRejected` | rejected kernel result。 | no | diagnostics のために structured rejection reason を持つ。 |
| `KernelCheckable` | policy が kernel へ送ってよい unchecked formula/substitution candidate または built-in discharge evidence。 | no | schedule 可能な evidence にすぎない。kernel が accept するまで winner ではない。安定した kernel representation を持たない built-in discharge evidence はこの class ではない。 |
| `ExternallyAttested` | policy が許可した external attestation。 | no | 許可され、`require_kernel_certificates` が false のときだけ recordable または policy-selectable。 |
| `OpenAllowed` | build mode が許す open obligation。 | no | diagnostic/status projection のみ。 |
| `AssumedByPolicy` | active policy が許す明示的な assumption。 | no | non-trusted policy status のみ。`require_kernel_certificates` を満たさず、trusted dependency を合成しない。 |
| `RejectedByPolicy` | active policy が reject した evidence または open status。 | no | 安定した policy rejection diagnostic を要求する。 |
| `DiagnosticOnly` | backend log、backend diagnostic、counterexample、cache record、timing、unsupported proof payload。 | no | proof winner にはならず、trusted material にもならない。 |

forward-compatible 実装は、新しい schema version の下でのみ class を追加してよい。
non-kernel material が trusted acceptance になれない規則は維持しなければならない。

## Kernel Scheduling Policy

`can_schedule_kernel_check` は、kernel evidence boundary 向けの evidence に対してだけ true を返す:

- target binding、encoded problem hash、formula label、symbol binding、provenance hash が
  encoded problem と一致する ATP formula/substitution evidence payload または ref;
- 安定した kernel evidence representation をすでに持つ deterministic built-in evidence;
- 明示的に許可された kernel primitive evidence。

次に対しては false を返す:

- externally attested evidence;
- backend proof method、backend log、SMT proof object、TSTP trace、resolution trace、
  unsat core、backend-reported used axiom、backend diagnostic;
- cache record と cache hit metadata;
- open obligation;
- 別の migration/audit policy が kernel の audit-only rejection surface に明示的に
  route しない限り legacy certificate/replay material。audit replay もこの crate で
  trusted acceptance を作れない。

scheduling permission は acceptance ではない。scheduled candidate は kernel が
`Accepted` を返した場合だけ trusted になる。

## Externally Attested Evidence

externally attested evidence は policy evidence である。kernel-verified evidence では
決してなく、trusted used-axiom material でもない。

Admission matrix:

| Policy state | Record as development evidence | May win selection | Publication status |
|---|---:|---:|---|
| `require_kernel_certificates = true` | `external_evidence` が recording を許す場合のみ | no | rejected-by-policy または open。accepted では決してない |
| release mode without external permission | no | no | rejected-by-policy |
| development mode with record permission | yes | kernel certificate が不要で明示的に policy-permitted の場合以外 no | externally attested / development evidence |
| interactive mode with record permission | yes | kernel certificate が不要で明示的に policy-permitted の場合以外 no | externally attested / open diagnostic |

rejection diagnostic は policy rejection と kernel rejection を区別しなければならない。
external record は `used_axioms` を合成してはならない。backend-reported axiom list は、
kernel が独立に evidence を accept して trusted `used_axioms` を生むまでは
diagnostic-only のままである。

## Open Obligations

open obligation は build mode と active policy によって分類される:

- release publication は open obligation を reject する;
- development mode は open obligation を policy-open result として記録してよい;
- interactive mode は diagnostics と LSP feedback のために open obligation を current に保ってよい。

open obligation は trusted proof status では決してない。status projection と diagnostics が
使う安定した explanation ref を持ってよい。

## Policy Assumptions

`AssumedByPolicy` は policy status であり、trusted acceptance ではない。明示的な
assumption source と、assumption の記録を許す active policy field を要求する。
release publication は、将来の non-trusted publication profile が明示的に許可しない
限り policy assumption を reject する。

policy assumption は `require_kernel_certificates` を満たせず、trusted witness material、
trusted `used_axioms`、proof reuse のための accepted dependency fact を作れない。
status projection は external attested、open、kernel-verified evidence と区別して保たなければならない。

## Policy Fingerprint

`PolicyFingerprint` は policy-relevant setting だけに対する canonical hash である:

- policy schema version;
- profile id;
- build mode;
- `require_kernel_certificates`;
- externally attested admission mode;
- open-obligation mode;
- policy-assumption mode;
- schedule 可能な kernel evidence representation set;
- checker schema version;
- classification、scheduling、selection、status projection、witness publication、
  proof reuse に影響する将来の forward-compatible policy field。

fingerprint に含めてはならないもの:

- candidate arrival order;
- backend completion time、runtime duration、process id、worker id、temporary path、
  local wall-clock time、scheduling priority;
- backend stdout/stderr bytes または diagnostic wording;
- cache hit/miss timing または cache record presence;
- stable published reference field 以外の artifact output path。

fingerprint serialization は安定 field name、sorted collection、explicit schema
version、決定的 byte encoding を使う。cache は fingerprint を validation predicate としてだけ
使ってよい。proof authority ではない。

## Early-Stop Policy Queries

policy module は、ATP portfolio でこれ以上よい acceptable class が不可能かどうかを
答えてよい。この query は policy-driven である:

- active policy でより上位 class が存在しない場合、kernel-verified winner は残りの
  backend work を stop してよい;
- `require_kernel_certificates` が true のとき external evidence は stop の根拠にならない;
- diagnostic-only candidate は semantic early stop の根拠にならない;
- open result は、schedule 可能または running の evidence がよりよい allowed class を
  生めないと policy が証明できる場合だけ stop してよい。

runtime duration と first-completion order は early-stop authority では決してない。

## Diagnostics

policy diagnostic は次を持つ安定 record である:

- `policy_rejection` または `policy_open` category;
- policy profile id と fingerprint;
- candidate class;
- stable reason code;
- optional explanation または evidence ref。

policy diagnostic は policy failure に kernel rejection code を再利用してはならない。
kernel failure と policy failure の両方が存在するとき、status projection は両方の layer を
区別可能に保つ。

## Boundary Rules

- policy module は ATP backend を呼ばず、SAT solving、proof search、premise selection、
  substitution invention、overload resolution、cluster search、coercion insertion、
  mutable global compiler state の参照を行わない。
- backend success、backend diagnostic、externally attested evidence、cache metadata、
  open obligation は trusted proof status を生まない。
- trusted `used_axioms` は accepted kernel result からだけ copy する。
- built-in discharge は accepted kernel checking または明示的に許可された kernel primitive
  result の後でだけ trusted `discharged_builtin` になる。それ以外は deterministic policy
  evidence のままである。
- 現在の artifact witness reference は formula/substitution の `kernel_verified`
  publication を support しているが、`discharged_builtin` publication は support していない。
  その artifact schema gap が閉じるまで、policy と selection は `discharged_builtin` を
  区別して保ってよいが、その class の witness publication は `external_dependency_gap` であり、
  `kernel_verified` へ潰してはならない。
