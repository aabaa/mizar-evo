# Module: substitution_checker

> 正本は英語です。英語版:
> [../en/substitution_checker.md](../en/substitution_checker.md)。

## 目的

`substitution_checker` module は、normalized kernel certificate に含まれる
substitution、alpha-conversion、freshness、free-variable evidence の決定的 replay
を所有する。これは
[architecture 15](../../architecture/ja/15.kernel_certificate_format.md)
「Substitution Rule」と
[architecture 16](../../architecture/ja/16.substitution_and_binding.md) を精緻化する。

Substitution replay は evidence checking であり、inference ではない。成功した replay
が示すのは、listed substitution entry が explicit source term、target term、
binder context、side-condition evidence によって正当化されていることだけである。
最終的な proof acceptance は後続の `checker` module が所有する。

## Trust statement

この module は trusted kernel code である。Explicit substitution と binder evidence を
replay し、normalized result を決定的に比較し、claimed result または side condition が
一致しない場合は fail closed しなければならない。

この module は proof search、ATP search、premise selection、overload resolution、
cluster search、implicit coercion insertion、fallback inference、hidden binder repair、
source-name lookup、cache lookup、artifact lookup、wall-clock / random-state read、
unordered iteration、mutable compiler-global state の hidden read を行ってはならない。
Resolver、checker、ATP backend、cache、artifact が以前に受理したことを理由に
substitution を受理してはならない。

## Owned behavior

この module が所有するもの:

- parsed certificate data と caller-supplied replay limits から explicit substitution
  replay context を導出すること;
- 各 substitution entry の parser-owned `binder_context_encoding` を decode / validate
  すること;
- `source_term` から `target_term` への capture-avoiding substitution を replay すること;
- deterministic alpha-conversion と freshness witness を validate すること;
- free-variable side condition を validate すること;
- 後続 checker orchestration のために checked substitution entry を記録すること;
- replay failure を stable `rejection` record へ map すること。

この module が所有しないもの:

- normalized certificate byte parsing や substitution id の structural sorting;
- imported fact availability または proof-status validation;
- resolution trace replay;
- cluster / reduction trace replay;
- final proof acceptance、proof-policy projection、witness storage、cache reuse、
  artifact emission;
- certificate に explicit に記録されていない substitution の計算または選択。

## Input and context

Task 11 と task 12 は explicit immutable input から replay を実装する:

```text
SubstitutionCheckInput
  target_vc_fingerprint
  parsed_certificate
  substitution_context
  replay_limits
```

`target_vc_fingerprint` は caller-owned であり、stable rejection record または
private report binding check にだけ copy される。Backend output、cache state、
artifact state、mutable compiler-global state から導出してはならない。

`parsed_certificate` は `certificate_parser::ParsedCertificate` である。Parser はすでに
section order、stable substitution id uniqueness、substitution id ordering、term byte
shape、sorted/unique freshness witness reference list、sorted/unique free-variable
constraint reference list を検査済みである。Substitution checker は defensive tests
でこれらの invariant を assert してよいが、byte parsing を重複実装してはならない。

`substitution_context` は caller-supplied immutable data である:

```text
SubstitutionContext
  substitution_payloads: sorted map substitution_id -> SubstitutionPayload
  freshness_witnesses: sorted map witness_id -> FreshnessWitness
  free_variable_constraints: sorted map constraint_id -> FreeVariableConstraint
  provenance_fingerprint
```

Concrete Rust type は map dependency を避けるため sorted vector を使ってよい。ただし
lookup と iteration は deterministic でなければならない。この context は resolver
state、checker state、ATP output、cache state、artifact state、global compiler state から
populate してはならない。Substitution context の欠如、context provenance の欠如、
substitution payload の欠如、referenced witness id の欠如、referenced constraint id の
欠如は `missing_provenance` である。Duplicate substitution payload id、witness id、
constraint id は invalid context shape である。Context constructor は replay 前に
module-local context error として決定的に拒否しなければならない。未検査の context shape が
replay に到達した場合、その ambiguous id は failed reference 上の `missing_provenance`
として扱う。

Checker は context entry を deterministic substitution-id order の first use で validate
する。未使用 context entry を scan して reject してはならない。Unavailable imported
fact、source definition、proof-status mismatch は後続の `checker` imported-fact task が
所有し、この module では `external_dependency_gap` のままである。

現在の normalized certificate schema は各 substitution entry について `source_term`、
`target_term`、binder-context bytes、side-condition reference を保持するが、
formal-to-actual map や rewrite payload をまだ inline していない。後続の certificate
schema task がその payload を parsed certificate へ移すまで、replay payload は
`substitution_id` で key 付けされた explicit caller-supplied context evidence とする。
Task 11 は payload のない referenced substitution を `missing_provenance` として拒否し、
`source_term` と `target_term` の diff から payload を推論してはならない。

Decoded binder context は caller-supplied authority ではない。各 certificate entry の
`binder_context_encoding` だけから導出される internal per-entry value であり、caller
context data がそれを proof evidence として override、repair、cache してはならない。

## Binder evidence model

`binder_context_encoding` は、この module が decode するまで parser-owned opaque bytes
である。Task 11 は substitution を受理する前に、最初の kernel-owned binder context
grammar を仕様化し実装しなければならない:

```text
BinderContextV1
  u16 schema_version = 1
  u32 frame_count
  BinderFrame[frame_count] frames
  u32 free_variable_count
  VariableId[free_variable_count] free_variables
  u32 schematic_variable_count
  VariableId[schematic_variable_count] schematic_variables

BinderFrame
  u32 binder_id
  u32 canonical_index
  u32 variable_id
  u8 binder_role
```

Accepted `binder_role` tags:

| Tag | Role |
|---|---|
| `0x01` | universal binder |
| `0x02` | existential binder |
| `0x03` | definition formal binder |
| `0x04` | generated fresh binder |
| `0x05` | schematic binder |

この grammar は certificate encoding v1 の scalar rule を継承する。すべての integer は
unsigned big-endian value であり、list は exact count を持ち、byte range は余りなく消費
されなければならず、trailing bytes は invalid である。Unknown binder-context schema
version、unknown binder role、truncated field、length overflow、duplicate frame、
noncanonical ordering は、先に deterministic resource limit を超えない限り
`invalid_substitution` である。

Frame は `canonical_index` で sorted、`binder_id` と `variable_id` で unique、
manifest-compatible、かつ `source_term` / `target_term` / payload `actual_term`
record が使う normalized term encoding と compatible でなければならない。Normalized
term は 2 つの binder node に同じ `binder_id` を再利用してはならない。Free /
schematic variable list は manifest-compatible、sorted、unique である。Display name と
source range は encode せず、semantic equality に参加しない。

Checker は `mizar-core` の stable data shape と contract を消費してよいが、binder
evidence は `mizar-kernel` 内で独立に再検査しなければならない。`mizar-core` API が
単に「すでに valid」と報告することは十分な proof ではない。Kernel-owned check は
stable id、binder depth、alpha-normalized structure、side-condition evidence を explicit
に比較しなければならない。

Substitution payload は concrete evidence record である:

```text
SubstitutionPayload
  u32 owner_substitution_id
  u8 payload_kind
  TermPath rewrite_path
  u32 replacement_count
  Replacement[replacement_count] replacements

Replacement
  u32 formal_variable_id
  TermRecord actual_term
  u8 replacement_role
```

Task 11 で受理する `payload_kind` tag は `0x01 = formal_to_actual_map` のみである。
`0x02 = local_abbreviation_expansion` は reserved であり、definition-site closure と
type-guard evidence が仕様化されるまで deferred とする。存在する `0x02` payload は
guard evidence なしに受理せず、`invalid_substitution` として拒否する。Accepted
`replacement_role` tags は `0x01 = term_argument`、`0x02 = predicate_argument` のみである。
`0x03 = captured_free_variable` は reserved であり、local abbreviation closure evidence と
ともに deferred とする。存在する `0x03` role は task 11 では `invalid_substitution` として
拒否する。Replacement は `formal_variable_id` で sorted unique でなければならない。各
`actual_term` は certificate symbol / variable manifest と term limit に対して validate され、
`rewrite_path` は `source_term` 内の既存 node を指さなければならず、
`owner_substitution_id` は現在 check 中の entry と一致しなければならない。存在するが
malformed な payload record は `invalid_substitution` であり、参照された payload record が
存在しない場合は `missing_provenance` である。

Side-condition context entry は concrete evidence record である:

```text
FreshnessWitness
  u32 witness_id
  u32 owner_substitution_id
  u32 generated_variable_id
  TermPath binder_path
  u32 avoided_variable_count
  VariableId[avoided_variable_count] avoided_variables
  u32 deterministic_counter

FreeVariableConstraint
  u32 constraint_id
  u32 owner_substitution_id
  u8 constraint_kind
  u32 variable_id
  TermPath term_path
  u32 capture_set_count
  VariableId[capture_set_count] capture_set

TermPath
  u32 segment_count
  TermPathSegment[segment_count] segments

TermPathSegment
  u8 edge_kind
  u32 child_index
```

Accepted `constraint_kind` tags は `0x01 = must_remain_free_at_path` と
`0x02 = must_be_absent_from_capture_set` である。Accepted `edge_kind` tags は
`0x01 = application_argument` と `0x02 = binder_body` である。Variable list は sorted
unique でなければならない。`owner_substitution_id` は現在 check 中の entry と一致しなければ
ならない。`binder_path` と `term_path` は既存の normalized term node を指さなければならない。
`deterministic_counter` は、その path と role に対する architecture 16 strategy が生成する
freshness counter と一致しなければならない。

Task 11 は、owner、path、list bound が otherwise valid な record に
`deterministic_counter` が存在することだけを検査する。Counter value が architecture 16 の
freshness strategy と一致するかの検証は task 12 が所有する。

存在するが malformed な witness または constraint record は `invalid_substitution` である。
参照された witness または constraint record が存在しない場合は `missing_provenance` である。

## Replay limits

Replay limits は deterministic である:

```text
SubstitutionReplayLimits
  max_substitutions
  max_binder_context_bytes
  max_binder_frames
  max_freshness_witnesses
  max_free_variable_constraints
  max_term_encoding_bytes
  max_term_recursion_depth
  max_alpha_renames
  max_payload_replacements
  max_term_path_segments
  max_avoided_variables
  max_capture_set_variables
```

Limit 超過は `kernel_rejection` / `resource_exhaustion` である。大きな temporary
context、term、payload、replacement list、witness list、path segment list、
avoided-variable set、capture set、free-variable set を allocate する前に budget を検査しなければ
ならない。

Term validation は explicit certificate symbol / variable manifest、`clause` と同じ stable
term encoding rule、設定された term-size / term-depth limit を使う。この module は
deterministic depth budget なしに caller-supplied term を再帰的に walk してはならない。

## Substitution replay

Parsed certificate order の各 `SubstitutionEntry` について:

1. `max_substitutions` を検査する。
2. `max_binder_context_bytes` と `max_binder_frames` の範囲内で entry の
   `binder_context_encoding` を decode する。
3. `source_term` と `target_term` が decoded binder context、symbol manifest、
   variable manifest、replay limits と構造的に compatible であることを validate する。
4. `substitution_id` payload と `freshness_witness_refs` id と
   `free_variable_constraint_refs` id を explicit `substitution_context` から解決する。
5. Stable id、normalized binder position、recorded replacement entry だけを使って、
   explicit payload を capture-avoiding substitution として replay する。
6. Recorded binder context と freshness witness が要求する場合に限り deterministic
   alpha-renaming を適用する。Generated fresh id は architecture 16 の deterministic
   strategy に従わなければならない。
7. Replayed normalized result を `target_term` と structural に比較する。
8. 後続 checker orchestration のために checked substitution id と normalized target を
   記録する。

Task 11 は direct substitution の step 1-5 と result comparison を所有し、payload の shape、
owner、rewrite-path、replacement ordering、resource、manifest validation を含めて検査する。
Task 11 は referenced freshness / free-variable record を presence、shape、owner、path
bound、deterministic first-use behavior についてだけ validate し、semantic freshness /
free-variable replay は task 12 に残す。Task 11 は replayed target の size / depth も
replayed term を allocate する前に検査する。Active binder の下にある variable occurrence
を置換する場合は、direct capture violation として先に reject される場合を除き task 11
では reject し、そのような under-binder substitution の semantic acceptance は task 12 に
defer する。Task 12 は同じ module に alpha-conversion、
freshness、free-variable side-condition replay を追加する。両 task は一貫した evidence
model を使わなければならない。

Checker は missing freshness witness を合成したり、binder renaming を推測したり、
display name を参照したり、omitted template argument を推論したり、implicit coercion を
挿入したり、target term に一致する alternate substitution を探索したりしてはならない。

## Alpha-conversion and freshness

Alpha-equivalence は normalized binder structure と stable id で判断し、source name や
rendered text では判断しない。Claimed alpha-conversion が valid である条件:

- 2つの normalized term の差分が decoded binder context 内の consistent bound-variable
  renaming だけである;
- renaming が各 binder scope 内で injective である;
- free variable が bound にならず、bound variable が scope から escape しない;
- generated fresh id がすべて referenced freshness witness によって正当化される;
- architecture 16 の deterministic freshness strategy が同じ id を同じ順序で生成する。

Freshness witness は evidence であり、suggestion ではない。Malformed、out-of-context、
stale、inconsistent、または必要箇所で未使用の present witness は `invalid_substitution`
である。参照された witness が supplied context に存在しない場合は
`missing_provenance` である。Supplied context 内の duplicate witness id は上記の
context-shape rule に従う。

## Free-variable conditions

Free-variable constraint は normalized binder structure 上で validate する:

- binder に入ると、その binder variable は body の free-variable set から除かれる;
- shadowing は distinct stable id で表現される;
- source display name は free-variable identity を決定しない;
- free のままでなければならない variable は replay 後の recorded term path で free の
  ままでなければならない;
- capture set から absent でなければならない variable はその set に出現してはならない;
- free-variable set と constraint id は比較または report emission の前に deterministic に
  sort される。

Free-variable condition 違反は `invalid_substitution` である。参照された constraint id が
`substitution_context` に存在しない場合は `missing_provenance` である。

## Report and checker interaction

Success report は deterministic checked-entry data だけを expose する:

```text
SubstitutionCheckReport
  checked_substitutions: sorted Vec<CheckedSubstitution>

CheckedSubstitution
  substitution_id
  source_term
  target_term
```

この report は後続 checker orchestration のための evidence-replay output である。
Accepted proof status、policy outcome、used-axiom projection、artifact-facing witness
decision を含んではならない。実装は、report を別の replay input と誤って組み合わせる
ことを拒否する目的に限り、caller-owned target fingerprint や certificate hash input
や `substitution_context.provenance_fingerprint` などの private replay binding data を保持
してよい。これにより、異なる replay input または異なる payload context との誤った組み合わせを
拒否する。Accessor は引き続き checked substitution data だけを expose しなければならない。

`derived_fact` validation と final-goal acceptance は、checked substitution、resolution
step、cluster trace、imported fact を trusted proof result へどう合成するかを `checker.md`
が仕様化するまで、この module の外に留まる。

## Rejection mapping

Substitution failure は `kernel_rejection` record を生成する:

| Failure | Detail | Location |
|---|---|---|
| Missing substitution context, missing context provenance, missing substitution payload, missing freshness witness id, or missing free-variable constraint id | `missing_provenance` | `substitution_id` plus the failed payload, witness, or constraint field when known |
| Duplicate substitution payload, freshness witness, or free-variable constraint ids in caller-supplied context, if not rejected by the context constructor before replay | `missing_provenance` | the first failed payload, witness, or constraint reference when known |
| Malformed present substitution payload, rewrite path, replacement role, replacement ordering, owner id, or actual term | `invalid_substitution` | `substitution_id` plus the failed payload, replacement, path, role, owner, or actual-term field |
| Malformed decoded binder context after parser byte decoding succeeded | `invalid_substitution` | `substitution_id` plus `binder_context` field |
| Malformed present freshness witness, free-variable constraint, term path, binder path, owner id, role tag, or side-condition tag | `invalid_substitution` | `substitution_id` plus the failed witness, constraint, path, or tag field |
| Source or target term incompatible with the replay context, symbol manifest, variable manifest, or binder context | `invalid_substitution` | `substitution_id` plus `source_term` or `target_term` field |
| Capture avoidance failure, target mismatch, invalid alpha-conversion, invalid freshness witness, or free-variable condition violation | `invalid_substitution` | `substitution_id` plus the most precise source, target, witness, alpha, or free-variable field |
| Replay count, binder-context byte/frame, payload replacement count, witness count, free-variable count, term-path segment count, avoided-variable count, capture-set count, term-size, term-depth, or alpha-rename limit exceeded | `resource_exhaustion` | most precise substitution or field location available |

すべての rejection location は deterministic で、`rejection.md` の shared
`RejectionLocation` fields、特に `substitution_id` と field path を使わなければならない。
Human diagnostic は追加 text を含んでよいが、その text は acceptance、ordering、stable
detail key に影響してはならない。

## Determinism and cost

Replay order は parsed substitution order である。Parser はこの order を
`substitution_id` sorted に保つ。Context constructor は input order を canonicalize するか、
duplicate を replay 前に deterministic に reject しなければならない。Checker は binder
frame、witness id、constraint id、free-variable set、checked report に sorted vector または
その他の deterministic data structure を使わなければならない。

同じ parsed certificate、substitution context、replay limits に対する result は、
platform や worker count に関係なく byte-for-byte stable でなければならない。Worker
completion order、allocation address、source display name、backend log、cache key、
artifact path、wall-clock time、random state は accepted/rejected result または rejection
ordering に影響してはならない。

Replay cost は checked substitution entry の size と、explicit に参照された binder、
payload、replacement、witness、path、free-variable evidence の size に対して、設定 limit
内で線形である。Checker は transitive proof search を行わず、unrelated context entry を
scan してはならない。

## Gap classification

- `spec_gap`: architecture 15 と 16 は substitution / binder principle を定義するが、
  concrete `mizar-kernel` module contract、binder-context grammar、report shape、
  rejection location、task-11/task-12 split は定義していない。この task は planned
  implementation task のためにその gap を閉じる。
- `test_gap`: task 11 は valid substitution replay、explicit payload validation、target
  mismatch、capture violation、malformed binder context、missing provenance、resource
  limits、deterministic report の Rust tests をまだ必要とする。Task 12 は
  alpha-equivalence、freshness、free-variable fail tests をまだ必要とする。
- `external_dependency_gap`: source-derived substitution certificate と downstream
  proof/cache/artifact consumer は、この module の active integration point ではない。
  Substitution payload の inline certificate encoding も将来の schema task まで deferred
  である。Task 11 は explicit immutable context payload evidence を消費し、missing payload
  は推測せずに拒否する。Local-abbreviation expansion payload は definition-site closure と
  type-guard evidence が仕様化されるまで deferred とする。Captured-free-variable
  replacement role も同じ closure evidence とともに deferred とする。
- `deferred`: imported fact checking、derived-fact assembly、final proof acceptance、
  policy projection は後続 `checker` と downstream proof tasks が所有する。

## Planned tests

Task 11 は以下の Rust tests を追加しなければならない:

- valid direct substitution が deterministic checked-entry output とともに accepted される;
- accepted `formal_to_actual_map` `payload_kind` と accepted `replacement_role`
  それぞれの positive payload coverage。`term_argument` と `predicate_argument` を含む;
- missing substitution payload が `missing_provenance` として rejected され、malformed
  payload kind、replacement role、rewrite path、owner id、replacement ordering、duplicate
  replacement、byte-valid だが manifest-incompatible な actual term が `invalid_substitution`
  として rejected される;
- deferred `local_abbreviation_expansion` payload は definition-site closure と
  type-guard evidence が仕様化されるまで `invalid_substitution` として rejected される;
- deferred `captured_free_variable` replacement role は definition-site closure evidence が
  仕様化されるまで `invalid_substitution` として rejected される;
- source と target term の diff から payload が推論されないこと;
- 同じ formal が複数の source location に現れる場合に、recorded `rewrite_path` occurrence
  だけが rewrite される positive rewrite-path case;
- source/target mismatch が stable `substitution_id` と field path を持つ
  `invalid_substitution` として rejected される;
- byte-valid だが symbol または variable manifest と incompatible な source / target term
  が `invalid_substitution` として rejected される;
- recorded witness が repair を正当化しない capture violation が alpha repair なしで
  rejected される;
- direct capture violation そのものではない under-binder replacement は task 11 で
  rejected され、semantic acceptance は task 12 に deferred される;
- malformed または noncanonical binder context が `invalid_substitution` として
  rejected される;
- unknown schema version、unknown binder role、truncated field、length overflow、
  duplicate frame、noncanonical frame order、noncanonical free/schematic
  variable list、exact-consumption / trailing-byte failure、frame/term
  incompatibility という binder-context decoding cases;
- missing substitution context、missing provenance、missing freshness witness、
  missing free-variable constraint が `missing_provenance` として rejected される;
- 存在するが malformed な freshness witness、free-variable constraint、term path、
  binder path、owner id、binder role、constraint kind、term-path edge tag、
  binder-path edge case、unsorted / duplicate `avoided_variables`、unsorted /
  duplicate `capture_set` が `invalid_substitution` として rejected される;
- substitution count、binder-context byte/frame、term-size、term-depth limits、
  freshness-witness count、free-variable-constraint count が大きな allocation または
  深い recursion の前に `resource_exhaustion` として rejected される;
- payload replacement count、term-path segment count、avoided-variable count、
  capture-set count limits が used entry の clone、sort、walk、allocation 前に
  `resource_exhaustion` として rejected される;
- over-budget `Replacement.actual_term` size / depth が payload actual term の walk または
  allocation 前に `resource_exhaustion` として rejected される;
- over-budget replayed target size / depth が replayed term の clone または allocation 前に
  `resource_exhaustion` として rejected される;
- context construction が input order を canonicalize し、duplicate witness / constraint
  ids と duplicate payload ids を replay 前に deterministic に reject すること、および
  未検査の ambiguous context shape に対する replay fallback mapping;
- first-use context validation が unused malformed / stale extra payload、witness、
  constraint を ignore すること;
- すべての rejection class で stable category/detail key、caller-owned target
  fingerprint propagation、precise deterministic `RejectionLocation` field を assert
  すること;
- report/input private binding が target-fingerprint mismatch と certificate/hash-input
  mismatch と substitution-context provenance mismatch について accidental report reuse を
  防ぐ;
- proof search、ATP/proof/cache/artifact coupling、overload resolution、cluster search、
  implicit coercion insertion、fallback inference、hidden binder repair、
  source-name/display-name lookup、omitted template-argument inference、
  unordered iteration、wall-clock/random read、global mutable-state read がないことの
  lint coverage。

Task 12 は以下の Rust tests を追加しなければならない:

- alpha-equivalent terms が display name ではなく normalized binder structure によって
  accepted される;
- inconsistent alpha renaming が `invalid_substitution` として rejected される;
- deterministic freshness witness は architecture 16 freshness strategy と一致する場合に
  限り accepted される;
- missing、out-of-context、stale、inconsistent freshness witness が rejected され、
  必要箇所で未使用の present witness が rejected され、duplicate context id は task-11
  context-construction tests で扱われる;
- free-variable constraint は valid path で accepted され、capture、escaping binder、
  shadowing confusion、missing variable で rejected される;
- alpha-rename と free-variable resource limits が `resource_exhaustion` として rejected
  される;
- alpha/freshness/FV replay が追加の side-condition fixture を導入する時点で、
  freshness-witness count と free-variable-constraint count limit も再度 assert される;
- shuffled context fixture construction の下でも report と rejection ordering が
  deterministic である。
