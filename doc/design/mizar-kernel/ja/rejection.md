# Module: rejection

> 正本は英語です。英語版:
> [../en/rejection.md](../en/rejection.md)。

## 目的

`rejection` module は `mizar-kernel` の安定した rejection vocabulary を所有する。
[architecture 15](../../architecture/ja/15.kernel_certificate_format.md) の
「Kernel Rejection Semantics」と
[architecture 19](../../architecture/ja/19.failure_semantics.md) を精緻化する。

Rejection は非受理の結果である。決定的で、machine-readable で、diagnostic 文言
の変更に対して安定していなければならない。backend success report、cache hit、
artifact witness、policy projection が kernel rejection を acceptance に変える
ことは決してない。

## Trust Statement

この module は trusted kernel code である。明示的で構造化された failure record
と deterministic ordering rule だけを定義しなければならない。

この module は proof search、premise selection、overload resolution、cluster
search、ATP search、implicit coercion insertion、fallback inference、imported-fact
lookup、cache lookup、artifact lookup、wall-clock / random-state read、unordered
iteration、mutable compiler-global state の hidden read を行ってはならない。

Task 20 の audit では、この trust boundary は no proof search, no SAT solving,
no ATP search or backend invocation, no premise selection, no overload
resolution, no cluster search, no implicit coercion insertion, no fallback
inference, no acceptance from backend-reported success alone, no source
loading, no cache lookup, no artifact lookup, no wall-clock or random-state
reads, no unordered iteration dependence, no hidden reads of mutable
compiler-global state を含むものとして検査する。

## Owned Behavior

この module が所有するもの:

- kernel が使う phase-level rejection category;
- parser と checker module が使う stable structured detail key;
- certificate bytes と checked evidence object の deterministic location;
- 複数の rejection record の deterministic ordering;
- public rejection value の追加または rename に関する compatibility rule;
- parser-owned error から shared rejection record への conversion policy。

この module が所有しないもの:

- 人間向け diagnostic wording;
- source-level parse、resolve、type、overload、cluster-loop failure;
- normalized certificate が存在する前の ATP backend timeout classification;
- proof-policy projection または user-facing proof status;
- cache、artifact、proof-witness publication。

## Categories

`mizar-kernel` は以下の phase-level category を使う。

| Category | Owner | Meaning |
|---|---|---|
| `certificate_rejection` | `certificate_parser` and checker-service witness normalization | normalized certificate envelope、service-level witness envelope、schema、context binding、structural bytes、canonical ordering、parser resource bound が不正。 |
| `kernel_rejection` | later checker modules | parsed certificate は構造的に読めるが、replay または evidence checking が失敗する。 |

Architecture 19 は `parse_error`、`resolve_error`、`type_error`、
`overload_ambiguity`、`cluster_loop`、`atp_timeout` などのより広い pipeline
category も定義する。これらは `mizar-kernel` の所有ではない。分類が曖昧な場合、
soundly に検出できる最も早い phase を選び、fail closed する。

## Stable Details

Detail key は安定した API value である。Diagnostic text はこれらの key を変更せず
に改善してよい。

| Detail key | Category | Meaning |
|---|---|---|
| `unsupported_certificate_format` | `certificate_rejection` | domain separator、schema version、encoding version、kernel profile、hash-input algorithm、unknown required section tag が非対応。 |
| `context_mismatch` | `certificate_rejection` | certificate が caller-supplied kernel context と一致しない target VC または explicit context identity に bound されている。Unsupported profile は `unsupported_certificate_format` のままである。 |
| `malformed_certificate` | `certificate_rejection` | envelope、directory、item frame、field encoding、canonical order、duplicate id、structural reference、generated-clause payload が不正。 |
| `malformed_witness_data` | `certificate_rejection` | kernel check service witness data が、normalized certificate または checked evidence object として扱える前に構造的に malformed である。 |
| `missing_provenance` | `kernel_rejection` | 必要な candidate provenance、premise provenance、immutable context provenance が欠落しており、kernel が evidence の由来を検証できない。 |
| `resource_exhaustion` | `certificate_rejection` or `kernel_rejection` | deterministic size、count、recursion、memory、trace-length、replay-cost limit を超えた。 |
| `invalid_substitution` | `kernel_rejection` | capture avoidance、alpha-conversion、freshness、free-variable side-condition replay が失敗した。 |
| `invalid_sat_proof` | `kernel_rejection` | clause または MiniSAT-compatible resolution replay が失敗した。pivot、parent、resolvent、final-goal derivation mismatch を含む。 |
| `invalid_cluster_trace` | `kernel_rejection` | explicit cluster または reduction trace replay が失敗した。hidden transitive expansion、invalid reduction substitution、strategy-audit mismatch を含む。 |
| `unresolved_symbol` | `kernel_rejection` | referenced symbol、imported theorem、imported axiom、VC、content fingerprint、required imported proof status が利用不能、不一致、または current kernel profile が許すより弱い。 |
| `timeout` | `kernel_rejection` | deterministic checker budget が replay を停止した。pure parser failure はこの detail を emit しない。 |

`timeout` と `resource_exhaustion` は proof acceptance ではない。obligation は未検証の
まま残る。

## Rejection Record

Task 7 は次の形の shared record を実装する。

```text
RejectionRecord
  target_vc_fingerprint
  category
  detail
  location
  stable_detail_key
```

`target_vc_fingerprint` は kernel check input が供給する deterministic sort key で
ある。Parser conversion は常に caller-supplied expected target VC を付与し、それで
並べる。certificate が主張する `target_vc` は、読める場合でも diagnostic context
に限られ、`context_mismatch` の ordering owner になってはならない。

`stable_detail_key` は `detail` の canonical な snake-case spelling である。snapshot、
diagnostic ordering、compatibility review に使う。record は後で diagnostic-facing
context を持ってよいが、その context が acceptance や ordering に影響しては
ならない。

Public rejection enum は、public-enum forward-compatibility audit が正当化された
例外を記録しない限り `#[non_exhaustive]` とする。

## Locations

Rejection location は deterministic で evidence-owned である。source path、
source range、display name、backend log、allocation address、wall-clock time、
worker completion order、cache key、artifact path、map/set iteration order に依存
しない。

```text
RejectionLocation
  certificate_byte_offset?
  section_tag?
  item_index?
  field_path?
  clause_ref?
  resolution_step_id?
  substitution_id?
  imported_fact_id?
  cluster_trace_step_id?
  reduction_step_id?
  derived_fact_id?
  final_goal?
```

規則:

- parser failure は byte offset と、分かっている section、item、field path を保持
  しなければならない;
- resolution failure は failed step と、利用可能な最も精密な parent、pivot、
  generated clause、final-goal reference を特定しなければならない;
- substitution failure は substitution entry と、失敗した source、target、
  freshness、alpha、free-variable field を特定しなければならない;
- imported-fact failure は imported fact namespace と stable imported fact id を
  特定しなければならない;
- cluster / reduction trace failure は、利用可能な場合に explicit trace step と
  失敗した source type、rule、substitution、guard、strategy-audit field を
  特定しなければならない;
- 早い段階の malformed bytes がより精密な reference を妨げる場合、location は
  partial でよいが、deterministic でなければならない。

## Parser Mapping

`certificate_parser` task 5 は module-local parse error をすでに公開している。
Task 7 は情報を失わずにそれを map しなければならない。

| Parser detail | Shared detail |
|---|---|
| `UnsupportedCertificateFormat` | `unsupported_certificate_format` |
| `ContextMismatch` | `context_mismatch` |
| `MalformedCertificate` | `malformed_certificate` |
| `ResourceExhaustion` | `resource_exhaustion` |

すべての parser error は category `certificate_rejection` を維持する。Parser error を
`kernel_rejection`、`timeout`、ATP/backend failure に remap してはならない。

## Checker Mapping

後続 checker module は failure を次のように map する。

| Checker owner | Failure | Detail |
|---|---|---|
| `resolution_trace` | pivot polarity mismatch、parent lookup mismatch、resolvent mismatch、generated-clause mismatch、non-final derivation | `invalid_sat_proof` |
| `substitution_checker` | capture、alpha-conversion、freshness、free-variable violation | `invalid_substitution` |
| `checker` cluster replay | hidden transitive expansion、cyclic explicit trace、invalid reduction substitution、guard mismatch、strategy-audit mismatch | `invalid_cluster_trace` |
| `checker` imported facts | missing imported theorem or axiom、fingerprint mismatch、unavailable VC or symbol、current kernel profile が許すより弱い status で accepted された imported theorem | `unresolved_symbol` |
| `checker` service | missing candidate、premise、immutable context provenance | `missing_provenance` |
| `checker` service | normalized certificate replay 前の malformed service-level witness data | `malformed_witness_data` |
| `checker` service | deterministic replay budget stop | budget が time-step 型なら `timeout`、size/memory/count 型なら `resource_exhaustion` |

kernel は missing premise を synthesize したり、alternate parent を search したり、
coercion を insert したり、cluster を expand したり、backend が proved と報告した
ことを理由に受理してはならない。

## Deterministic Ordering

1 つの batch で複数の rejection を報告する場合、次の順に並べる。

1. target VC fingerprint bytes;
2. category order: `certificate_rejection`、次に `kernel_rejection`;
3. 存在する場合は certificate byte offset;
4. 存在する場合は section tag、item index、field path;
5. stable evidence id: imported fact id、imported axiom/theorem clause ref、
   generated-clause ref、resolution step id、resolution-step clause ref、
   substitution id、cluster trace step id、reduction step id、derived fact id、
   final-goal marker;
6. stable detail key。

Parallel checking や worker completion order はこの順序に影響してはならない。

## Compatibility Policy

Stable category と detail key は snapshot-facing API である。spelling、meaning、
phase ownership、ordering の変更には compatibility review と public-enum policy
update が必要である。新しい detail key を追加できるのは次の場合だけである。

- 既存 key が rejection を正確に表さない;
- owning phase が文書化されている;
- test が category、detail key、deterministic location を assert する;
- source/documentation consistency が同じ task で review される。

Human diagnostic message は、stable category、detail、ordering、location semantics が
変わらない限り compatibility review なしで改善してよい。

## Gap Classification

- `spec_gap`: architecture 15 は rejection reason を命名するが、shared
  `mizar-kernel` record shape、parser mapping、checker mapping、deterministic
  location rule は定義していない。この module spec は task 6 でその gap を閉じる。
- `test_gap`: task 7 では stable category/detail key、parser error conversion、
  deterministic ordering、`#[non_exhaustive]` public enum、pure parser failure が
  `timeout` にならない保証の Rust tests が必要である。
- `external_dependency_gap`: `mizar-proof`、`mizar-cache`、`mizar-artifact` は
  kernel rejection record の active consumer ではない。placeholder policy/cache/
  artifact coupling をここに追加してはならない。
- `deferred`: source-derived `.miz` rejection snapshot と downstream proof policy
  projection は task 6 の外に残す。

## Planned Tests

Task 7 は Rust tests を追加しなければならない。

- `FailureCategory` と rejection detail ごとに 1 つの stable key;
- allowed category/detail pair と invalid mapping の拒否。parser detail が
  `kernel_rejection` にならず、`timeout` が `certificate_rejection` にならない
  ことを含む;
- parser-error conversion が `target_vc_fingerprint` を保持すること。malformed
  bytes のため `target_vc` を読めない場合の caller-supplied fallback と、
  category、detail、byte offset、section、item index、field path の保持を含む;
- checker-side location が `resolution_step_id`、`substitution_id`、
  `clause_ref`、`imported_fact_id`、`cluster_trace_step_id`、`reduction_step_id`、
  `derived_fact_id`、`final_goal` を保持すること;
- `missing_provenance` を `missing_provenance`、`malformed_witness_data` を
  `malformed_witness_data`、imported proof-status mismatch を `unresolved_symbol`
  とする checker-service mapping;
- resolution failure を `invalid_sat_proof`、substitution failure を
  `invalid_substitution`、cluster/reduction failure を `invalid_cluster_trace`、
  missing または fingerprint-mismatched imported fact を `unresolved_symbol` とする
  checker-owner mapping;
- insertion order や worker completion order に依存しない deterministic ordering。
  `target_vc_fingerprint`、category order、byte offset、section/item/field、
  checker evidence id、stable detail key の tie-breaker case を明示すること;
- pure parser failure が `timeout` に map されないこと;
- `timeout` と `resource_exhaustion` が非受理結果のままであること;
- planned `#[non_exhaustive]` policy に従って public rejection enum が forward
  compatible であること;
- rejection code が ATP/proof/cache/artifact crate を import せず、imported fact を
  global state から読まず、global mutable state、wall-clock/random API を読まず、
  proof/premise/cluster/ATP search を行わず、overload を resolve せず、implicit
  coercion を insert せず、fallback inference を行わず、unordered map/set
  iteration に依存しないことを示す lint coverage。

この module-spec task では、`.miz` fixture、expectation sidecar、`doc/spec`、
Rust source change は不要である。
