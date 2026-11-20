# mizar-proof Policy Spec

source の open/assumed 登録は既存 development policy の `OpenObligation` と
`PolicyAssumption` で観測する。両者は別の非 kernel-verification class であり、
strict release の拒否規則は変えない。

> 正本は英語です。英語版:
> [../en/policy.md](../en/policy.md)。

## 目的

policy module は、evidence production の上位、artifact publication の下位にある
verifier-policy decision を定義する。proof evidence の分類、kernel checking を
schedule してよいかの判定、外部認証 evidence を policy evidence として許可または
拒否する規則、安定した `PolicyFingerprint`、policy-driven early-stop query を所有する。

この module は proof acceptor ではない。trusted acceptance と trusted
`used_axioms` は、status が `Accepted` である `mizar-kernel`
`KernelCheckResult` のうち、kernel evidence check kind が `ProofObligation` のもの
だけから来る。

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
| `instance_search_limits` | 明示的な finder work-count 上限の任意設定。[設定 carrier](#instance-search-limits) に留める。 |

### Instance Search Limits

`VerifierPolicy::instance_search_limits(&self) -> Option<[u64; 4]>` と `with_instance_search_limits(self, Option<[u64; 4]>) -> Self` は既存型で一つの private field を公開する。どちらも const とし、getter は値を copy、builder はこの field だけを変更する。Release/development/interactive/Default は None を保持し、None は設定解除・探索上限未設定を意味し、無制限探索ではない。zero/u64::MAX はそのまま保持し、変換や数値既定値を導入しない。
array の順は、有限な入力/生成 pool の異なる ground term 数、前提 instantiation candidate 数、名前付き equality instance candidate 数、untrusted SAT search check 数の最大値とする。将来 finder の決定的 work-count 上限であり、timeout・前提刈込み・backend run・Kernel parse/encode 上限と区別する。計数/制限適用・項生成・上限到達時の evidence なし・finder 起動は ATP の別作業で、設定の保存では行わない。
None の policy_fingerprint は従来 field/byte をそのまま出す。Some は既存 checker_schema_version field の後・finalize 前に field_u16 `instance_search_limits_version` = 1、続いて四要素の field_u64 `max_ground_terms`、`max_premise_instances`、`max_equality_instances`、`max_sat_checks` を追加する。既存 hash/domain/framing を再利用し、schema version・旧 field は変更しない。Some zero と None は異なり、解除で旧 fingerprint に戻る。既存 classification/scheduling/admission/selection/acceptance は不変で、設定 identity は検証用 data に留まる。
追加 unit 2系列で変更前の実 legacy fingerprint/default/解除と、明示 zero/max/各要素の区別/決定性を確認する。追加の実 source 1系列は元/偽定義の aggregate registration・両 format・既存 policy を再利用し、fresh Kernel result/PolicyDecision tuple 全体を未設定 baseline と比較して実入力不変性を保持する。この既存生成候補は finder 実行ではなく、test-only Accepted constructor や既存 policy/producer matrix の複製を追加しない。
この carrier は ATP30 の完了、term-wire 対応、探索/上限 evidence、runtime currentness、witness/publication/reuse、Step6–8 gate の成立を供給しない。

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
| `KernelVerified` | ATP formula/substitution evidence に対する accepted proof-obligation kernel result。 | yes | kernel result から trusted `used_axioms` を伝播してよい。Accepted consistency check は diagnostic-only であり、この class には入らない。 |
| `DischargedBuiltin` | kernel が accept した kernel-owned primitive evidence を含む、built-in discharge evidence に対する accepted proof-obligation kernel result。 | yes | `KernelVerified` とは別に射影する。ATP kernel verification へ潰してはならない。Accepted consistency check は diagnostic-only であり、この class には入らない。 |
| `KernelRejected` | rejected kernel result。 | no | diagnostics のために structured rejection reason を持つ。 |
| `KernelCheckable` | policy が kernel へ送ってよい unchecked formula/substitution candidate または built-in discharge evidence。 | no | schedule 可能な evidence にすぎない。kernel が accept するまで winner ではない。安定した kernel representation を持たない built-in discharge evidence はこの class ではない。 |
| `ExternallyAttested` | policy が許可した external attestation。 | no | `ExternalEvidenceAdmission` に従って recordable になりうる。policy-selectable になるのは許可され、`require_kernel_certificates` が false のときだけ。 |
| `OpenAllowed` | build mode が許す open obligation。 | no | diagnostic/status projection のみ。 |
| `AssumedByPolicy` | active policy が許す明示的な assumption。 | no | non-trusted policy status のみ。`require_kernel_certificates` を満たさず、trusted dependency を合成しない。 |
| `RejectedByPolicy` | active policy が reject した evidence または open status。 | no | 安定した policy rejection diagnostic を要求する。 |
| `DiagnosticOnly` | backend log、backend diagnostic、counterexample、cache record、timing、unsupported proof payload。 | no | proof winner にはならず、trusted material にもならない。 |

forward-compatible 実装は、新しい schema version の下でのみ class を追加してよい。
non-kernel material が trusted acceptance になれない規則は維持しなければならない。

訂正済み kernel proof-obligation rejection は terminal な policy input のままである。
`invalid_sat_refutation`、kernel-evidence goal-polarity の
`context_mismatch`、`missing_provenance`、real legacy unsupported-certificate
gate record は `KernelRejected` diagnostic のままにし、受理へ retry したり、
policy-open へ再分類したり、external admission で upgrade したりしてはならない。
selection は otherwise-open fallback よりこれらの rejection を優先して、status
projection が failed proof result を記録できるようにしてよい。ただし accepted
trusted kernel evidence は rejected diagnostic より上位のままである。proof crate は
この gate を test するために legacy certificate payload を fabricate してはならない。
real-payload coverage は public kernel legacy-certificate fixture または runner を待つ。

accepted proof-obligation kernel input について、`KernelPolicyInput` は
proof-reuse metadata が使う kernel-validated accepted goal polarity も公開する。
現在の public value は `AcceptedGoalPolarity::AssertFalseForRefutation` である。
Accepted consistency check と non-accepted または policy-tainted input は accepted
goal polarity を公開しない。

rejected kernel-evidence input について、kernel result が提供する場合、
`PolicyDecision` は kernel evidence check kind を保持する。selection はその明示 kind を
使って proof-obligation goal-polarity failure と consistency-check diagnostic を区別する。
rejection detail text だけからその区別を推測してはならない。

## Public Enum Policy

task 14 は public-enum forward-compatibility procedure をこの module に適用する。
すべての public policy enum は downstream-facing API surface であり、
`#[non_exhaustive]` を維持しなければならない。downstream consumer は wildcard
match arm を保ち、新しい variant を evidence 昇格として silent に扱わず、
policy/spec review を要求するものとして扱う。

| Enum | Compatibility decision |
|---|---|
| `BuildMode` | forward-compatible |
| `ExternalEvidenceMode` | forward-compatible |
| `OpenObligationMode` | forward-compatible |
| `PolicyAssumptionMode` | forward-compatible |
| `KernelEvidenceFormat` | forward-compatible |
| `CandidatePolicyClass` | forward-compatible |
| `PortfolioEarlyStopClass` | forward-compatible |
| `PortfolioEarlyStopReason` | forward-compatible |
| `KernelEvidenceOrigin` | forward-compatible |
| `AcceptedGoalPolarity` | forward-compatible |
| `PolicyCandidate` | forward-compatible |
| `BackendProofPayloadKind` | forward-compatible |
| `ExternalEvidencePublicationStatus` | forward-compatible |
| `PolicyDiagnosticCategory` | forward-compatible |
| `PolicyReasonCode` | forward-compatible |

No exhaustive public enum exceptions are owned by this module.

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

## Bounded native positive-result policy

crate-private `consume_object_inhabitation` は実際の Kernel `ObjectInhabitationCheck` を消費する。不透明型が正の存在導入と rule origin を固定する。`BuiltinKernelEvidence` が有効で、Kernel 所有の checker schema と指定 policy の要求 schema が完全一致する場合だけ、その policy の fingerprint を返す。それ以外は拒否する。この検査後の primitive 適格性経路は汎用 `KernelPolicyInput`、選択可能 class、accepted-evidence hash、refutation polarity を作らない。既存の汎用分類と trusted selection は変更しない。

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

`ExternalEvidenceAdmission` は、この matrix の task-owned API surface である。次を持つ:

- `record_as_development_evidence`;
- `may_win_selection`;
- `publication_status`;
- `diagnostic`;
- `trusted_used_axioms_allowed`。常に `false`。

`publication_status` は次のいずれかである:

- `RejectedByPolicy`;
- `ExternallyAttestedDevelopment`;
- `ExternallyAttestedOpenDiagnostic`;
- `ExternallyAttestedPolicyPermitted`。

完全な対応は次である:

| `external_evidence` | `require_kernel_certificates` | `build_mode` | Record? | May win? | Publication status | `PolicyDecision.class` | Diagnostic |
|---|---:|---|---:|---:|---|---|---|
| `Reject` | any | any | no | no | `RejectedByPolicy` | `RejectedByPolicy` | `policy_rejection/external_evidence_rejected` |
| `RecordDevelopment` | true | `Interactive` | yes | no | `ExternallyAttestedOpenDiagnostic` | `ExternallyAttested` | `policy_open/external_evidence_recorded` |
| `RecordDevelopment` | true | `Release` または `Development` | yes | no | `RejectedByPolicy` | `RejectedByPolicy` | `policy_rejection/external_evidence_requires_kernel_certificate` |
| `RecordDevelopment` | false | `Interactive` | yes | no | `ExternallyAttestedOpenDiagnostic` | `ExternallyAttested` | `policy_open/external_evidence_recorded` |
| `RecordDevelopment` | false | `Release` または `Development` | yes | no | `ExternallyAttestedDevelopment` | `ExternallyAttested` | `policy_open/external_evidence_recorded` |
| `PermitNonTrustedWinner` | true | `Interactive` | yes | no | `ExternallyAttestedOpenDiagnostic` | `ExternallyAttested` | `policy_open/external_evidence_recorded` |
| `PermitNonTrustedWinner` | true | `Release` または `Development` | yes | no | `RejectedByPolicy` | `RejectedByPolicy` | `policy_rejection/external_evidence_requires_kernel_certificate` |
| `PermitNonTrustedWinner` | false | any | yes | yes | `ExternallyAttestedPolicyPermitted` | `ExternallyAttested` | `policy_open/external_evidence_policy_permitted` |

policy-tainted accepted proof-obligation kernel input は同じ external admission mapping を使う。
元の kernel result は kernel status の唯一の由来のままだが、policy-tainted result は
trusted `KernelVerified` または `DischargedBuiltin` として射影されない。Accepted
consistency check は external admission mapping の前に diagnostic-only のままである。

external decision では、`PolicyDecision.diagnostic` は
`ExternalEvidenceAdmission.diagnostic` を mirror する。両方とも absent であるか、
構造的に等しくなければならない。kernel rejection record は external admission に保存しない。

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

- kernel-verified winner は、別の `KernelVerified` candidate を含む pending selectable
  class が active policy の下でそれを置き換えられない場合にだけ、残りの backend work を
  stop してよい;
- `require_kernel_certificates` が true のとき external evidence は stop の根拠にならない;
- diagnostic-only candidate は semantic early stop の根拠にならない;
- open result は、schedule 可能または running の evidence がよりよい allowed class を
  生めないと policy が証明できる場合だけ stop してよい。

runtime duration と first-completion order は early-stop authority では決してない。

task 12 はこれを ATP portfolio 向けの class-level finality query として公開する:

```text
PortfolioEarlyStopInput
  observed_best_class?
  pending_best_possible_classes[]

PortfolioEarlyStopDecision
  may_stop
  reason
  observed_best_class?
  blocking_pending_class?
```

`observed_best_class` は、必要な kernel result が利用可能になった後に観測済みの最良の
policy-normalized class である。`pending_best_possible_classes` は、pending または
running work がまだ生みうる class について、portfolio owner が保守的にまとめたもので
ある。これらの class は semantic possibility であり、backend completion event ではない。

decision が `may_stop = true` になるのは次のすべてを満たす場合だけである:

- observed class が存在し、active policy の下で selectable である;
- すべての pending class が active policy の下で selectable でないか、deterministic
  winner order において observed class より厳密に低い;
- observed class と同じ pending class が存在しない。同じ class の candidate は、
  後続 API が明示的な dominance proof を供給しない限り、deterministic tie-break key により
  observed candidate を置き換えうるためである。

`reason` は free-form string ではなく stable enum である:

| Reason | Meaning |
|---|---|
| `NoObservedCandidate` | observed class がまだ存在しない。 |
| `ObservedClassNotSelectable` | observed class は active policy の下で selectable winner になれない。 |
| `BlockedByHigherClass` | pending class がより上位の selectable winner を生みうる。 |
| `BlockedByEqualClass` | pending class が observed class と同じで、deterministic tie-break によりまだ勝ちうる。 |
| `NoDisplacingPendingClass` | observed class を置き換えうる pending selectable class が存在しない。 |

query は `PolicyCandidate` を best possible early-stop class に正規化する helper も提供する:

- accepted proof-obligation ATP formula/substitution kernel result と、まだ kernel check
  可能な formula/substitution evidence は `KernelVerified` に map する;
- accepted proof-obligation built-in/kernel-primitive evidence と、まだ kernel check 可能な
  built-in evidence は `DischargedBuiltin` に map する;
- accepted consistency-check kernel result は non-selectable な `DiagnosticOnly`
  evidence であり、early-stop class を持たない;
- externally attested evidence は、active policy がそれを winner として許可し、kernel
  certificate を要求しない場合にだけ `PolicyPermittedExternal` に map する;
- policy assumption と open obligation は、active policy がその class を winner として
  許可する場合にだけ、それぞれの non-trusted class に map する;
- rejected material と diagnostic-only material は stop の根拠にならない。

blocking pending class が存在する場合、それは class rank により選ばれ、input order では
選ばれない。この query は ATP backend を実行せず、process を cancel せず、kernel check を
schedule せず、backend timing を調べず、class 内の final winner を選ばず、proof status を
publish しない。portfolio/scheduler owner が消費する advisory finality predicate である。

task 12 は owner contract を `mizar-proof` に定義する。downstream `mizar-atp` によるこの
contract の消費、process cancellation wiring、runtime portfolio integration は、ATP 側が API を
採用するまで deferred / external-dependency work のままである。`mizar-proof` は ATP adapter や
scheduler stub を追加しない。

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
- trusted `used_axioms` は accepted proof-obligation kernel result からだけ copy する。
- built-in discharge は、kernel が accept した kernel-owned primitive evidence を含む
  accepted proof-obligation kernel result の後でだけ trusted `discharged_builtin` に
  なる。それ以外は deterministic policy evidence のままである。
- 現在の artifact witness reference は formula/substitution の `kernel_verified`
  publication を support しているが、`discharged_builtin` publication は support していない。
  その artifact schema gap が閉じるまで、policy と selection は `discharged_builtin` を
  区別して保ってよいが、その class の witness publication は `external_dependency_gap` であり、
  `kernel_verified` へ潰してはならない。

## Computation Publication Boundary

検証済み E5 入力/結果、有限の認証変換、候補全体検査は §§20.9.2/20.13.7 に従う。
実際の MVM/artifact/contract/proof 記録を消費し、term/truth、UID 置換、ガード、Binder 構築の後で残存を全体拒否する。
import 失敗は内部復帰を保ち ensures の一部を公開しない。生成側欠落や未完了は未検査の事実を作らない。
テストは term/truth/update/opaque 変換と終端 UID 置換を扱う。ensures/等式全体、量化型、annotation、container の残存、循環 requires、更新前置換の誤りを拒否し、実際の未完了段階をそれぞれ区別する。
明示 reference の replay は既定 false の manifest computation_replay より優先し、全呼出に適用する。既定 computed、全呼出 equation/SAT replay、厳密 cache-mode 同一性、strict の推移的 computed 依存を区別する。
テストは逆の manifest default に対する明示 replay true/false を直接/推移呼出しで扱い、全呼出し replay の拒否、strict computed 依存、弱い mode の cache 破損を確認する。

boolean conversion test は certified two-valuedness の下で B=true/B≠false、B=false/B≠true、B=C/B≠C の極性、guarded val_at、再正規化しない terminal ordinary b=true を点検する。truth equivalence は boolean term-argument/Binder-body slot を埋めず、未認証 term は拒否する。

## Formation Evidence Boundary

真正な pinned library/per-use formation evidence を正確な conditional premise と request identity とともに検証する。綴り、Open goal、算法 contract evidence は accepted status を与えない。古い version/content/UID/context/C と誤った premise substitution を拒否する。受理した instance は Checker へ戻し、有限 KΓ/Adm equivalence と Binder all-input construction obligation を E1/E2/E4/execution と区別する。

## Imported Promotion Authority Boundary

移送可能な promoted premise は、algorithm/contracts/equations と dependency/version/policy provenance に結び付いた真正な適用対象 E1/E2/E4 と complete-group evidence の検証を必要とする。module-local status と runtime/hash metadata はこの権威を与えない。

### Accepted evidence and validators

| 既存 producer/validator | 正確な権威と限界 |
|---|---|
| [KernelPolicyInput::from_kernel_result と accepted_evidence_hash_from_kernel_result](../../../../crates/mizar-proof/src/policy.rs)、[TrustedKernelEvidence::from_policy_input](../../../../crates/mizar-proof/src/selection.rs) | 実際の Accepted、untainted ProofObligation kernel result と target/checked evidence。external/open/audit/cache metadata はこの権威を得ない。個別義務の local acceptance に限る。 |
| [ProofObligationIdentity と project_status](../../../../crates/mizar-proof/src/status.rs) | obligation/VC/context/dependency identity と active selection-policy の一致。通常の kernel-verified publication は witness を必要とする。algorithm return-reflexivity/state-transitivity、theorem、registration check は真正な限定 producer だが、集約 E1/E2/E4/group acceptance ではない。 |
| [TrustedKernelWitnessMetadata と CommittedWitnessPublicationProof::from_source_theorem_manifest_commit](../../../../crates/mizar-proof/src/witness_store.rs) | private な accepted-result binding と committed source-theorem publication の検証。witness-file reader は untrusted wire evidence を復元する。それも theorem publication proof も imported promotion を検証しない。 |
| [FormulaImportedFactEvidence、validate_formula_import_projection、validate_formula_import_status](../../../../crates/mizar-kernel/src/checker.rs) | caller が渡す immutable imported context と package/module/export/statement identity、正確な statement-to-formula projection、許可 status を照合する。legacy identity/status check も生成元について同じ限界を持ち、exporting producer や promotion group を認証しない。 |

必要な論理内容は [§20.13.1–6](../../../spec/ja/20.algorithm_and_verification.md#2013-論理エンコーディング) に従う。E1 は result typing、single-valued execution、主張する全 typed input 上の value existence を正当化し、E2 は body partial correctness を検証し、E4 は正確な guard の下で normal return を独立に検証する。宣言、単なる停止、continuation の仮定 return、一つの ground call の E5 は E4 を代替しない。完全な input、type/requires guard、適用対象 syntax environment/InputOK/context/C binding を保持する。
通常の promotion は guarded totality/contract を公開し、definitional algorithm だけが §20.13.6 の guarded return equation を textual-order path guard と repeated const substitution とともに公開する。新たな coverage/consistency obligation は加えない。mutual promotion は全適用 prerequisite と complete membership の受理まで全 member を保留し、equation はさらに全 member が definitional であることを必要とする。
現在の source に、これらの義務の aggregate promotion producer や portable acceptance validator はない。VC57 の module-local acceptance dependency も未実装で、その契約は transferable receipt を禁止する。[Artifact binding](../../mizar-artifact/ja/manifest.md#imported-promotion-authority-boundary) は既存の transport foundation であり、代替 proof path ではない。

### Trust-transfer rule

信頼元は **importer が承認した Proof execution authority** による、実際の aggregate acceptance result の committed publication とする。importer は imported artifact から独立した自身の管理下の trust configuration により、その authority、承認済み Proof/kernel implementation と version、許可する policy/profile/checker-schema scope を確立する。authority がそれらの check を忠実に実行し、真正な結果だけを公開することを信頼する。artifact、caller、package registry、dependency は自己承認や trust scope の拡張を行えず、独立に確立した authority がなければ promoted premise を一つも与えない。

authority が promotion acceptance を公開できるのは、真正な accepted かつ untainted な kernel/Proof evidence が、完全な recursive group の全 member について上記の適用対象 prerequisite をすべて確立した後に限る。publication はその受理済み aggregate result から authority 管理下の execution と commit を経て生成され、caller が accepted result を捏造・置換できてはならない。この規則は既存の acceptance decision を移送し、Open、assumed、externally attested、failed、unchecked evidence を kernel acceptance に格上げしない。

Artifact transport は承認済み execution と committed publication から、正確な package/module/export と algorithm identity、contract、適格な equation、完全な group membership、member ごとの prerequisite までの authenticated binding を保持する。binding は上記の完全な guarded input/context identity、source と pinned library content、dependency version/lockfile、実際の proof-policy/profile/checker-schema provenance を含む。integrity hash は結び付いた content を識別するが、その受理者を確立しない。新 receipt/schema、public summary、caller が渡す accepted status だけでは authority を与えない。

Proof import validation は authenticated origin と完全な binding を確立し、importer が解決した object/dependency と現在の trust configuration に照合し、export 側 policy/profile/checker schema が importer の active acceptance requirement を満たすことを要求する。同じ policy label だけでは不十分で、弱い・不明な policy を import により強い status にしない。authority/binding/policy data が missing、unauthenticated、forged、stale、partial、mismatched、unsupported なら、どの member premise も公開しない。この provenance/projection validation は exporting module の再証明も opaque body の load も要求しない。

validation 成功時に公開できるのは上記の ordinary/definitional premise だけであり、imported provenance を受理に保持し、新たな local kernel result にはしない。export された一 member の公開にも complete-group acceptance が先行する。真正な non-computable/contract-only promotion は runtime artifact なしで同じ logical trust path を使い、computation は別の artifact/correspondence check を保つ。

これが承認済み trust-transfer 仕様決定である。[Artifact producer transport design](../../mizar-artifact/ja/manifest.md#producer-transport-design) が authentication と representation を選定し、以下の [import-validation design](#proof-import-validation-design) が acceptance handoff を所有する。実行可能 transport と Proof import validator は後続 slice に残る。この規則は言語規則や kernel inference を変更しない。

### Proof import-validation design

将来の Proof validator は、Artifact が構造検証した未信頼の immutable publication closure、正確な signed bytes/signature、選択した module/summary object を受け取る。独立した importer input は、解決済み logical object/dependency snapshot、現在の trust configuration、active acceptance requirement とする。完全な一 group を以下の段階で非公開に検証し、途中の成功ではどの member premise も公開しない。

1. **認証と承認。** Artifact 設計の canonical かつ supported な descriptor/version/suite を要求し、保持した正確な byte 列に対する framed Ed25519 signature を検証する。現在の importer 管理下の trust で key を解決し、authority に承認された Proof/kernel implementation/version と許可 policy/profile/checker-schema scope を認証済み descriptor に照合する。同梱 key、key identifier、registry origin、caller status は hint/data であり trust root ではない。承認済み signature が restricted producer path を通じた committed aggregate acceptance を証明し、local manifest の可視性や unsigned descriptor だけでは commitment を確立しない。transport が約束しない、認証済みの第二 exporter commit は要求しない。
2. **解決済み binding の照合。** descriptor と必須 object closure から trust-transfer rule が要求する完全な authenticated tuple を復元する。importer が選んだ正確な module/summary bytes、解決済み package/module/export/algorithm、logical statement、guarded input/context、source/pinned-library identity、dependency version/lockfile、完全な group と照合する。名前/version だけでなく content commitment も比較し、semantic-version compatibility や interface hash で異なる byte 列を承認しない。private source/proof content は認証済み identity で比較し、opaque body を load しない。binding が absent、substituted、mixed、duplicate、partial、mismatched なら拒否し、publication に合う別 object へ解決し直さない。
3. **acceptance と projection の検証。** supported checker schema と結び付いた promotion class/input/context の下で、認証済み committed acceptance が全 group member と全適用 prerequisite を網羅することを要求する。その完全な網羅性を表現できない schema は unsupported とする。missing、Open、failed、tainted、assumed、externally attested record や、authority 管理下の aggregate publication から切り離された accepted flag では満たさない。exporter proof を replay せず、認証済み acceptance record を検証する。実際の export 側 policy/profile/checker-schema semantics が全 active importer acceptance requirement を満たすことを、承認 scope 内の明示的で supported な importer 所有 compatibility check だけで要求する。label、fingerprint、version 番号の増加だけでは互換性を確立せず、不明・弱い組合せは fail closed とする。各 public statement の imported formula への正確な projection と、上記規則による ordinary/definitional eligibility を確認し、余分な equation や弱めた guard を拒否する。一 export だけを要求した場合も、未選択 member を complete-group validation に含める。
4. **imported provenance を保持した公開。** 全段階の成功後だけ、その検証済み group の適格な選択済み projection をまとめて利用可能にする。認証済み descriptor/signature、exporter authority/implementation、完全な binding、検証に使った importer resolution/trust/acceptance snapshot との対応を保持する。これは imported acceptance であり、新たな local kernel result、local accepted witness、強い status を生成しない。既存 kernel identity/projection/status check は consumer check として必要だが、この gate の代替にはならない。

**currentness と reuse。** 検証と公開は同じ immutable publication/importer snapshot を使う。公開前に resolution、trust、active requirement が変われば pending result を破棄し、新 snapshot で検証する。再利用にも、全 input が現在の importer state と一致する確認を要求し、authorization、implementation scope、policy/schema requirement、object、dependency の変更は以前の判断を無効にする。cached success や依然有効な signature でもこの確認を省略しない。ここでの currentness は wall-clock expiry や独立した trust-bootstrap protocol を追加しない。

**拒否と限界。** structural/content/transaction refusal は Artifact に残り、signature、authority、resolved-binding、acceptance/policy、projection refusal は Proof が所有する。どの失敗でも該当 group の全 promoted premise を保留し、status/hash/cache fallback や部分公開を行わない。legacy opaque signature は既存経路で利用できるが promotion authority は持たない。diagnostic naming/API shape は実装時に決め、この設計で public diagnostic code を追加しない。真正な non-computable/contract-only acceptance も runtime load なしで同じ logical gate を通り、computation は別の artifact/correspondence check を保つ。

### VC59 connection design

将来の VC59 経路が受け取るのは、[Proof import validation](#proof-import-validation-design) が完了した group validation のみであり、その検証に使った正確な importer 解決済み export と snapshot に対応させる。Artifact load、public summary、local VC57 acceptance、caller が渡す status は、検証済み imported acceptance としてこの経路に入れない。Proof は authentication/acceptance を、VC は検証済み projection の選択と premise への組込みを所有する。この handoff に新たな public receipt や serialized schema は要求しない。

- **適格な premise の選択。** 既存 import/name visibility で imported definition label を解決し、VC57 と同じ definition-expansion policy を適用する。後続 VC がその label を引用するか policy が利用可能にするとき、選択した algorithm の promotion axiom を渡す。algorithm 名や call は引用ではない（[§16.5.1](../../../spec/ja/16.theorems_and_proofs.md#1651-単純な引用)）。item order を保ち、遡及的に fact を追加しない。ordinary promotion は検証済みの guarded totality/contract のみを、definitional promotion はさらに上記の適格な equation のみを与え得る。正確な検証済み formula と完全な guarded input/context binding を使い、consumer は余分な equation や guard を外した強い statement を合成しない。
- **消費まで provenance を保持。** 各 premise を正確な解決済み export/algorithm と statement projection、および完了した validation の authenticated imported provenance に対応させ、complete-group と importer-snapshot の対応も保つ。VC 構築と kernel imported-context/projection/status check を通じてその対応を保持する。これらの check は引き続き必要だが Proof validation の代替にはならない。単なる formula/status に平坦化せず、local accepted witness/kernel result へ変換せず、別 export/group/context の authority にしない。再利用は validator の currentness 規則に従う。
- **不可分な公開と利用。** 選択した export が一つでも全 group を検証し、完全な validation と consumer mapping の成功後にのみ、選択した premise をまとめて利用可能にする。premise の構築から消費まで、同じ immutable publication と現在の importer snapshot を使う。resolution、authorization、acceptance requirement の変更は pending/reused premise を無効にするため破棄し、現在の input に対する validation 後にのみ再構築する。projection または provenance 対応の欠落・不一致は該当 group の全 promoted premise を保留し、部分的に組み立てた集合を VC に渡さない。import visibility、cached premise、以前の check 成功でもこの条件を省略しない。

[六行の portable validation](#portable-validation-design-obligations) は、この接続の将来の consumer check にも適用する。成功する ordinary/definitional と logical-only import は、正確な適格 formula と imported provenance を消費まで保ち、すべての拒否は group premise を一つも与えない。実際の接続は、真正な local aggregate producer、Artifact transport と Proof import-validation の実装、真正な VC57 local prerequisite/group acceptance、六行すべての実行可能な成功/拒否を待つ。この経路の文書化でいずれの gate も満たさない。

### Portable validation design obligations

以下の六行は将来の producer/consumer check であり、実行可能な portable-promotion coverage ではない。成功には上記 trust-transfer rule に従って認証した exporter acceptance が必要である。

| 入力形状 | 必要な結果と欠けた経路 |
|---|---|
| 承認された authority が真正に公開した、正確な algorithm/contracts、適用 E1/E2/E4、complete-group evidence を持つ ordinary/definitional accepted export | 検証済み totality/contract と適格な defining equation だけが VC59 へ届く。aggregate producer、transport、import acceptance validator が未実装。 |
| 適用 E1/E2/E4 または任意 member evidence が missing、Open、failed、forged、invalid | どの member premise も公開しない。認証済み prerequisite/group validation が未実装。 |
| algorithm/contract/equations、source/library binding、dependency/version、group membership が stale | premise を公開しない。正確な portable binding/currentness validation が未実装。 |
| 非互換 proof policy/profile/checker schema 下の accepted evidence | premise を公開しない。既存 kernel status check は portable policy provenance を認証しない。 |
| 真正な accepted non-computable/contract-only export | runtime artifact を load せず logical import が成功する。認証された logical-only path が未実装。computation は別に execution artifact の欠落/不正を拒否する。 |
| 未信頼・自己承認 origin、hash/status/public-summary-only または runtime-only input | promoted premise を公開しない。integrity、visibility、normal-return metadata は promotion acceptance を確立しない。 |

既存 [status test](../../../../crates/mizar-proof/src/status/tests.rs) は local class、witness publication、policy mismatch、reuse invalidation を、[kernel test](../../../../crates/mizar-kernel/src/checker/tests.rs) は imported context/projection/policy check を扱う。portable 成功行の実装ではない。
実装順序: 真正な VC57 local prerequisite/group acceptance は引き続き前提であり、この規則に従う Artifact producer transport design、Proof import validation、VC59 connection の順序を保つ。真正な producer/validator と六行すべての成功/拒否を実装してから、認証済み output のみに VC59 を接続する。opaque import は exporting module を再証明しない（[§12.6.1](../../../spec/ja/12.modules_and_namespaces.md#1261-不透明なインポート)）。runtime correspondence は execution owner に残る。
`external_dependency_gap`: 真正な local aggregate E1/E2/E4/group output、Artifact producer transport 実装、Proof import-validation 実装、VC59 connection は依然として存在しない。trust-transfer 定義が解消するのは従来の authority-definition gap のみであり、実行可能な acceptance/coverage credit を与えず、VC59 を unblock しない。

## Computation Admission Boundary

MVM evaluation による obligation discharge の前に、完全な真正 computation closure と各必須 ensures uniqueness proof/正確な syntactic exemption を検証する。E1 execution single-valuedness と要求された all-call equation/SAT replay は別 gate。Open/failed/stale/forged/不足 requirement は computation fact を与えない。真正 producer/diagnostic integration が存在するとき、Proof が仕様の E0330 projection を所有する。テストは真正 admission、raw VM return、replay-only evidence、不完全 imported closure、誤った call/contract/exemption provenance を区別し、strict/default/replay policy は維持する。
