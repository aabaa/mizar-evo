# Module: certificate_parser

> 正本は英語です。英語版:
> [../en/certificate_parser.md](../en/certificate_parser.md)。

## 目的

`certificate_parser` module は、phase 14 が消費する normalized kernel
certificate の parse と structural validation を所有する。これは
[architecture 15](../../architecture/ja/15.kernel_certificate_format.md) の
certificate top level を精緻化する。

Parse は proof acceptance ではない。parsed certificate は、後続の kernel
module が imported fact、substitution、resolution replay、cluster trace、final
goal を検証するまで、まだ信頼されない evidence である。

## Schema Ownership

`mizar-kernel` は normalized certificate schema type、schema-version table、
section tag、byte grammar を所有する。将来の `mizar-atp` などの evidence producer
はこの schema を構築してよいが、kernel は producer crate に依存してはならない。
producer / consumer crate が存在するまで、その統合は `external_dependency_gap` で
ある。task 5 はここで定義する parser-owned schema と structural validation だけを
実装する。

schema は versioned である。task 5 は schema version `1` と encoding version `1`
だけを実装する。

## Trust Statement

この module は trusted kernel code である。bounded input に対して小さく、
deterministic で、total であり、malformed bytes や unsupported profile に対して
fail closed しなければならない。

この module は、proof search、premise selection、overload resolution、cluster
search、ATP search、implicit coercion insertion、fallback inference、
backend-specific proof translation、imported-fact availability lookup、
proof-policy projection、cache lookup、artifact lookup、wall-clock / random-state
read、unordered iteration、mutable compiler-global state の hidden read を行っては
ならない。

## Owned Behavior

この module が所有するもの:

- normalized certificate byte envelope の decode;
- schema、encoding、kernel-profile、hash-input algorithm compatibility の検査;
- section の presence、ordering、uniqueness、offset、byte length、item count の
  検証;
- section payload を allocate する前の top-level resource limit 検査;
- stable id namespace と duplicate id の検証;
- 明示的な parse context に対する target VC binding の検証;
- clause validation context を構築する certificate-local symbol / variable
  manifest の検証;
- public clause data shape へ generated clause を decode し、structural validation
  のために `clause` module を呼び出すこと;
- resolution-step self / forward reference を含む、純粋に structural な reference
  shape と order constraint の検査;
- parser error の deterministic byte/section/item/field location の保持;
- 後続 checker のための deterministic parsed data の生成。

この module が所有しないもの:

- imported axiom/theorem availability または proof-status policy;
- substitution の capture avoidance、alpha-conversion、freshness、free-variable
  checks;
- resolution pivot polarity checks または resolvent recomputation;
- cluster trace replay または cluster-search reconstruction;
- final-goal derivation;
- ATP backend parsing、MiniSAT trace normalization、witness storage、proof artifact
  publication、cache reuse validation。

## Input And Context

Task 5 は、明示的な byte slice と明示的な parse context から parse する実装にする。

```text
CertificateParseContext
  accepted_schema_versions
  accepted_encoding_versions
  accepted_kernel_profiles
  expected_target_vc
  clause_validation_policy
  max_certificate_bytes
  max_section_bytes
  max_imported_facts
  max_generated_clauses
  max_substitutions
  max_resolution_steps
  max_derived_facts
  max_symbol_manifest_entries
  max_variable_manifest_entries
  max_term_recursion_depth
```

context は caller supplied の immutable data である。parser は resolver state、
checker state、ATP output、cache state、artifact state、global compiler state から
context を populate してはならない。

## Canonical Envelope

normalized certificate は domain-separated, versioned binary envelope である。
backend-specific proof format ではない。

```text
CertificateEnvelopeV1
  bytes domain_separator = "MIZAR_KERNEL_CERT\0"
  u16 schema_version = 1
  u16 encoding_version = 1
  KernelProfileRecord kernel_profile
  Fingerprint target_vc
  u32 directory_entry_count
  DirectoryEntry[directory_entry_count] section_directory
  bytes section_payloads
```

`KernelProfileRecord` は parser-owned schema metadata である。

```text
KernelProfileRecord
  u16 profile_id
  u16 clause_schema_version
  u16 clause_encoding_version
  u8 clause_tautology_policy        # 1 = reject, 2 = marker
  u8 certificate_hash_input_algorithm
```

Task 5 は `certificate_hash_input_algorithm = 1` をサポートする。名前は
`canonical-envelope-v1` である。これは canonical hash input bytes の識別子であり、
cryptographic digest provider ではない。parser は canonical hash input bytes を返す
または保持する。hash するためだけに digest dependency を追加してはならない。
未知の hash-input algorithm id は `unsupported_certificate_format` である。

`Fingerprint` は length-prefixed stable bytes である。

```text
Fingerprint
  u8 algorithm_id
  u32 byte_length
  bytes digest
```

Task 5 は fingerprint の shape と context equality だけを検証する。digest は計算し
ない。

## Section Directory

Schema version `1` は、下記の section が numeric tag order でちょうど一度ずつ現れる
ことを要求する。per-section rule が別に定めない限り、section は 0 item を含んでよい。

| Tag | Section | Item tag | Count rule |
|---|---|---|---|
| `0x01` | `symbol_manifest` | `0x01` symbol entry | `<= max_symbol_manifest_entries` |
| `0x02` | `variable_manifest` | `0x01` variable entry | `<= max_variable_manifest_entries` |
| `0x03` | `imported_axioms` | `0x01` imported fact ref | `<= max_imported_facts` |
| `0x04` | `imported_theorems` | `0x01` imported fact ref | `<= max_imported_facts` |
| `0x05` | `generated_clauses` | `0x01` generated clause | `<= max_generated_clauses` |
| `0x06` | `substitutions` | `0x01` substitution entry | `<= max_substitutions` |
| `0x07` | `resolution_trace` | `0x01` resolution step | `<= max_resolution_steps` |
| `0x08` | `derived_facts` | `0x01` derived fact | `<= max_derived_facts` |
| `0x09` | `final_goal` | `0x01` final goal ref | exactly `1` |

Directory entry は以下で encode する。

```text
DirectoryEntry
  u8 section_tag
  u32 item_count
  u32 payload_offset
  u32 payload_length
```

`payload_offset` は directory の直後の最初の byte からの相対位置である。entry は
`section_tag` で sort され、offset は sort 済みで overlap してはならない。schema
`1` では gap や trailing bytes のない contiguous section payload を要求する。parser
は duplicate section、missing section、unknown section、out-of-order section、
overlapping section、non-contiguous range、`u32` overflow、`payload_length >
max_section_bytes` を allocate 前に拒否しなければならない。

## Item Frames

section 内の各 item は frame である。

```text
ItemFrame
  u8 section_tag
  u8 item_tag
  u32 length
  bytes payload
```

frame `section_tag` は containing directory entry と一致しなければならない。frame
`item_tag` は上の table と一致しなければならない。payload 内の item frame 数は
`DirectoryEntry.item_count` と一致しなければならない。parser は truncated frame、
trailing bytes、length overflow、item-tag mismatch、section-tag mismatch を拒否する。

すべての integer は unsigned big-endian value である。string と opaque bytes は以下を
使う。

```text
Bytes
  u32 byte_length
  bytes payload
```

## Item Payloads

parser は schema `1` の具体的な payload layout を所有する。

この certificate schema における symbol kind tag はここで所有する。

| Tag | Symbol kind |
|---|---|
| `0x01` | predicate |
| `0x02` | functor-as-predicate |
| `0x03` | equality |
| `0x04` | built-in relation |

Task 5 はこれらの tag を public `clause::SymbolKind` value に map する。private
clause encoder tag を call または複製してはならない。

```text
SymbolManifestEntry
  u8 symbol_kind                 # 上記の certificate schema tag
  u32 symbol_id

VariableManifestEntry
  u32 variable_id

ImportedFactRef
  u32 imported_fact_id
  Bytes package_id
  Bytes module_path
  Bytes exported_item_id
  Fingerprint statement_fingerprint
  u8 required_proof_status       # 1 kernel_verified, 2 discharged_builtin,
                                 # 3 externally_attested_policy_permitted

GeneratedClause
  u32 clause_id
  u8 clause_form                 # 1 ordinary, 2 empty, 3 tautology
  u32 literal_count
  LiteralRecord[literal_count] literals

LiteralRecord
  u8 polarity                    # 1 negative, 2 positive
  AtomRecord atom

AtomRecord
  u8 symbol_kind
  u32 symbol_id
  u32 arity
  u32 term_count
  TermRecord[term_count] arguments

TermRecord
  u8 term_tag
  ... payload
```

`TermRecord` tags:

| Tag | Term payload |
|---|---|
| `0x01` | `u32 variable_id` |
| `0x02` | `u8 symbol_kind, u32 symbol_id, u32 term_count, TermRecord[term_count] arguments` |
| `0x03` | `u32 binder_id, TermRecord body` |

unknown term tag（malformed-placeholder tag を含む）は malformed certificate である。
recursion depth は `max_term_recursion_depth` で bound される。

残りの structural payload:

```text
RefList
  u32 count
  u32[count] ids                 # sorted and unique

SubstitutionEntry
  u32 substitution_id
  TermRecord source_term
  TermRecord target_term
  Bytes binder_context_encoding
  RefList freshness_witness_refs
  RefList free_variable_constraint_refs

ClauseRef
  u8 namespace                   # 1 generated_clause, 2 resolution_step,
                                 # 3 imported_axiom, 4 imported_theorem
  u32 id

ResolutionStep
  u32 step_id
  ClauseRef parent_a
  ClauseRef parent_b
  LiteralRecord pivot_literal
  ClauseRef generated_clause

DerivedFact
  u32 derived_fact_id
  ClauseRef source
  Bytes payload

FinalGoalRef
  u8 namespace                   # 1 generated_clause, 2 resolution_step,
                                 # 3 derived_fact
  u32 id
```

Generated clause は `GeneratedClause` から public `clause` module data shape へ
decode され、manifest と parse context だけから導出された
`ClauseValidationContext` を使って `Clause::from_canonical_parts` で検証される。
Task 5 は private clause byte tag を複製したり依存したりしてはならない。

## Structural Validation Rules

certificate は以下の場合だけ structurally valid である。

- envelope domain separator、schema version、encoding version、kernel profile、
  hash-input algorithm が parse context に受理される;
- `target_vc` が `expected_target_vc` と完全一致する;
- すべての required section が canonical order でちょうど一度ずつ現れる;
- すべての section count と payload size が parse context limit 内である;
- stable id が namespace 内で一意で、id で sort されている;
- imported fact reference が non-empty package、module、item、statement
  fingerprint、required-proof-status fields を持つ;
- imported facts が `imported_fact_id` で sort され、id が一意で、stable reference
  key が一意である。duplicate key は payload byte が完全一致しても拒否される;
- symbol / variable manifest が sort 済み、一意で、generated clause validation に
  対して complete である;
- generated clauses が `clause_id` で sort 済み、一意で、`clause` module に受理される;
- substitution entries が `substitution_id` で sort 済み、一意で、structurally
  decode され、freshness / free-variable reference list が sort 済みで一意である;
- resolution step が canonical order の一意な step id を持つ;
- resolution parent reference が existing imported fact id、generated clause id、
  earlier resolution-step id だけを指す。self reference と forward resolution-step
  reference は malformed;
- pivot literal は structurally valid である;
- `ResolutionStep.generated_clause` は `generated_clause` namespace を使い、existing
  generated clause id を指さなければならない;
- `DerivedFact.source` は imported axiom、imported theorem、generated clause、
  resolution-step namespace を使ってよい。imported / generated id は存在しなければ
  ならず、resolution-step id は earlier step を指さなければならない;
- pivot polarity と resolvent equality は `resolution_trace` に残す;
- derived fact id が sort 済みで一意である;
- final-goal reference が syntactically valid で、existing generated clause、
  resolution step、derived fact を指す。

parser は missing reference を synthesize したり、missing symbol を infer したり、
coercion を挿入したり、definition を展開したり、alternate parent を探索したり、
backend が proved と言ったことを理由に certificate を受理したりしてはならない。

## Ordering And Hashing

すべての parsed collection は deterministic order を使う。

1. section tag;
2. stable namespace id;
3. namespace 内の stable reference key;
4. section が child encoding を定義する場合は canonical child bytes を final tie
   breaker とする。

parser は canonical certificate hash input bytes を公開する。cryptographic digest は
計算しない。Hash input bytes は domain separator、schema version、encoding
version、`certificate_hash_input_algorithm` を含む完全な `KernelProfileRecord`、
`target_vc`、section directory、canonical section payload bytes を含む。

Hash input bytes は file paths、source ranges、display names、backend stdout/stderr、
timestamps、elapsed time、allocation addresses、map/set iteration order、worker
completion order、cache keys、artifact paths、policy projection outcomes を含んでは
ならない。

## Failure Classes And Locations

Task 5 は module-local structural errors を以下の形で報告する。

```text
CertificateParseError
  category = certificate_rejection
  detail
  location

CertificateParseLocation
  byte_offset
  section_tag?
  item_index?
  field_path?
```

`detail` は parser-owned case を `rejection.md` が最終化する stable rejection reason に
map する。

| Parser-owned case | Stable detail |
|---|---|
| unsupported schema, encoding, kernel profile, hash-input algorithm, or unknown section tag | `unsupported_certificate_format` |
| target VC mismatch | `context_mismatch` |
| malformed envelope, directory, frame, field, id ordering, duplicate id, malformed reference, or noncanonical generated clause | `malformed_certificate` |
| generated clause rejected by the clause module | `malformed_certificate` |
| count, byte, `u32` length/offset, or recursion-depth exhaustion | `resource_exhaustion` |

pure parsing は `timeout` を決して emit しない。後続の deterministic budget wrapper が
budget stop を timeout に変換することはあり得るが、task 5 parser tests は pure parser
failure が timeout にならないことを assert しなければならない。すべての parser error
は non-acceptance outcome である。

## Gap Classification

- `spec_gap`: architecture 15 は logical certificate top level を名付けているが、
  concrete bytes、section tags、manifest fields を定義していない。この module spec
  は task 5 のために schema `1`、encoding `1`、normalized kernel envelope、section
  directory、item payload、parser-owned manifest、parser-owned failure location を
  定義してその gap を閉じる。
- `test_gap`: normalized certificate parsing、section canonicality、reference
  validation、profile rejection、stable failure detail/location、hash input coverage、
  parser resource limits を覆う Rust tests はまだない。
- `external_dependency_gap`: `mizar-atp` の normalized certificate producer と
  `mizar-proof`、`mizar-cache`、`mizar-artifact` の proof-witness consumer は未存在
  または未完成であり、ここで mock してはならない。
- `deferred`: backend-specific MiniSAT proof payload translation、artifact witness
  storage、cache reuse validation、source-derived `.miz` certificate corpus は task 5
  の外に残す。

## Planned Tests

Task 5 は Rust tests を追加しなければならない。

- minimal valid normalized certificate;
- unsupported schema、encoding、profile、hash-input algorithm、unknown section を
  `certificate_rejection` + `unsupported_certificate_format` として拒否;
- target VC mismatch を `certificate_rejection` + `context_mismatch` として拒否;
- missing、duplicate、unknown、out-of-order、overlapping、non-contiguous、truncated、
  trailing section data を deterministic byte / section location とともに拒否;
- item count mismatch、item-tag mismatch、section-tag mismatch、malformed field、
  trailing item payload を deterministic item / field location とともに拒否;
- `max_certificate_bytes`、`max_section_bytes`、per-section count limits、`u32`
  length/offset overflow、term-recursion-depth resource exhaustion を大きな allocation
  前に拒否;
- imported fact id ordering、duplicate-id rejection、duplicate-key rejection、
  malformed package/module/item fields、malformed fingerprint、malformed
  required-proof-status rejection;
- symbol / variable manifest ordering、duplicate、unsupported kind、completeness
  checks;
- `clause` module 経由の generated clause validation（`malformed_certificate` に
  map される noncanonical clause failure を含む）;
- substitution entry structural validation、malformed freshness/free-variable
  reference lists、capture/freshness acceptance を行わないこと;
- resolution-step id ordering と malformed parent、self parent、forward parent、
  malformed pivot literal、malformed generated-clause namespace、missing
  generated-clause reference rejection;
- final-goal と derived-fact source namespace/reference validation;
- symbol manifests、variable manifests、imported axioms、imported theorems、
  generated clauses、substitutions、resolution steps、derived facts、canonical
  child-byte tie breakers の deterministic parsed-output ordering;
- shuffled だが equivalent な source data に対する hash-input stability;
- domain separator、schema version、encoding version、hash-input algorithm を含む
  full kernel profile、`target_vc`、section directory、canonical payload bytes の
  positive hash-input coverage;
- source paths、source ranges、display names、backend logs、timestamps、elapsed
  time、allocation addresses、allocation order、map/set iteration order、worker
  completion order、cache keys、artifact paths、policy projection outcomes の hash
  exclusion;
- すべての negative case に対する stable failure category/detail assertions。pure
  parser failure が `timeout` を emit しないことを含む;
- parser が ATP/proof/cache/artifact crate、resolver/checker global state、
  wall-clock/random API、unordered-map/set iteration を import/call せず、search を
  行わないことを示す lint coverage。

この module-spec task では、`.miz` fixture、expectation sidecar、`doc/spec` change は
不要である。
