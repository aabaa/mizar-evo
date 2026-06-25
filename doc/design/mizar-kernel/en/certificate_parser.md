# Module: certificate_parser

> Canonical language: English. Japanese companion:
> [../ja/certificate_parser.md](../ja/certificate_parser.md).

## Purpose

The `certificate_parser` module owns parsing and structural validation for the
normalized kernel certificate consumed by phase 14. It refines the certificate
top level in
[architecture 15](../../architecture/en/15.kernel_certificate_format.md).

Parsing is not proof acceptance. A parsed certificate is still untrusted
evidence until later kernel modules validate imported facts, substitutions,
resolution replay, cluster traces, and the final goal.

## Schema Ownership

`mizar-kernel` owns the normalized certificate schema types, schema-version
table, section tags, and byte grammar. Evidence producers such as future
`mizar-atp` code may construct this schema, but the kernel must not depend on
producer crates. Until those producer/consumer crates exist, their integration
is an `external_dependency_gap`; task 5 implements only the parser-owned schema
and structural validation described here.

The schema is versioned. Task 5 implements schema version `1` and encoding
version `1` only.

## Trust Statement

This module is trusted kernel code. It must be small, deterministic, total over
bounded inputs, and fail closed for malformed bytes or unsupported profiles.

The module must not perform proof search, premise selection, overload
resolution, cluster search, ATP search, implicit coercion insertion, fallback
inference, backend-specific proof translation, imported-fact availability
lookup, proof-policy projection, cache lookup, artifact lookup, wall-clock or
random-state reads, unordered iteration, or hidden reads of mutable
compiler-global state.

## Owned Behavior

The module owns:

- decoding the normalized certificate byte envelope;
- checking schema, encoding, kernel-profile, and hash-input algorithm
  compatibility;
- validating section presence, ordering, uniqueness, offsets, byte lengths, and
  item counts;
- checking top-level resource limits before allocating section payloads;
- validating stable id namespaces and duplicate ids;
- validating target VC binding against an explicit parse context;
- validating certificate-local symbol and variable manifests used to construct
  clause validation contexts;
- decoding generated clauses through public clause data shapes and invoking the
  `clause` module for structural validation;
- checking reference shapes and order constraints that are purely structural,
  including resolution-step self/forward references;
- retaining deterministic byte/section/item/field locations for parser errors;
- producing deterministic parsed data for later checkers.

The module does not own:

- imported axiom/theorem availability or proof-status policy;
- substitution capture avoidance, alpha-conversion, freshness, or
  free-variable checks;
- resolution pivot polarity checks or resolvent recomputation;
- cluster trace replay or cluster-search reconstruction;
- final-goal derivation;
- ATP backend parsing, MiniSAT trace normalization, witness storage, proof
  artifact publication, or cache reuse validation.

## Input And Context

Task 5 should implement parsing from an explicit byte slice and an explicit
parse context:

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

The context is caller-supplied immutable data. The parser must not populate it
from resolver state, checker state, ATP output, cache state, artifact state, or
global compiler state.

## Canonical Envelope

The normalized certificate is a domain-separated, versioned binary envelope.
It is not a backend-specific proof format.

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

`KernelProfileRecord` is parser-owned schema metadata:

```text
KernelProfileRecord
  u16 profile_id
  u16 clause_schema_version
  u16 clause_encoding_version
  u8 clause_tautology_policy        # 1 = reject, 2 = marker
  u8 certificate_hash_input_algorithm
```

Task 5 supports `certificate_hash_input_algorithm = 1`, named
`canonical-envelope-v1`. This is an identifier for canonical hash input bytes,
not a cryptographic digest provider. The parser returns or stores canonical
hash input bytes; it must not add a digest dependency merely to hash them.
Unknown hash-input algorithm ids are `unsupported_certificate_format`.

`Fingerprint` is length-prefixed stable bytes:

```text
Fingerprint
  u8 algorithm_id
  u32 byte_length
  bytes digest
```

Task 5 validates only the shape and context equality of fingerprints. It does
not compute digests.

## Section Directory

Schema version `1` requires every section below exactly once, in numeric tag
order. Sections may contain zero items unless a per-section rule says
otherwise.

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

Directory entries are encoded as:

```text
DirectoryEntry
  u8 section_tag
  u32 item_count
  u32 payload_offset
  u32 payload_length
```

`payload_offset` is relative to the first byte after the directory. Entries
must be sorted by `section_tag`, offsets must be sorted and non-overlapping,
and schema `1` requires contiguous section payloads with no gaps or trailing
bytes. The parser must reject duplicate sections, missing sections, unknown
sections, out-of-order sections, overlapping sections, non-contiguous ranges,
`u32` overflow, and any `payload_length > max_section_bytes` before allocating.

## Item Frames

Every item in a section is a frame:

```text
ItemFrame
  u8 section_tag
  u8 item_tag
  u32 length
  bytes payload
```

The frame `section_tag` must match the containing directory entry. The frame
`item_tag` must match the table above. The number of item frames in a payload
must equal `DirectoryEntry.item_count`. The parser must reject truncated frames,
trailing bytes, length overflow, item-tag mismatch, and section-tag mismatch.

All integers are unsigned big-endian values. Strings and opaque bytes use:

```text
Bytes
  u32 byte_length
  bytes payload
```

## Item Payloads

The parser owns these concrete payload layouts for schema `1`.

Symbol kind tags in this certificate schema are owned here:

| Tag | Symbol kind |
|---|---|
| `0x01` | predicate |
| `0x02` | functor-as-predicate |
| `0x03` | equality |
| `0x04` | built-in relation |

Task 5 maps these tags into public `clause::SymbolKind` values. It must not
call or duplicate private clause encoder tags.

```text
SymbolManifestEntry
  u8 symbol_kind                 # certificate schema tag above
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

Unknown term tags, including malformed-placeholder tags, are malformed
certificates. Recursion depth is bounded by `max_term_recursion_depth`.

Remaining structural payloads:

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

Generated clauses are decoded from `GeneratedClause` into public `clause`
module data shapes and validated by `Clause::from_canonical_parts` using a
`ClauseValidationContext` derived only from the manifest and parse context.
Task 5 must not duplicate or depend on private clause byte tags.

## Structural Validation Rules

A certificate is structurally valid only when:

- the envelope domain separator, schema version, encoding version, kernel
  profile, and hash-input algorithm are accepted by the parse context;
- `target_vc` exactly matches `expected_target_vc`;
- every required section appears exactly once in canonical order;
- every section count and payload size is within the parse context limits;
- stable ids are unique within their namespace and sorted by id;
- imported fact references have non-empty package, module, item, statement
  fingerprint, and required-proof-status fields;
- imported facts are sorted by `imported_fact_id`, ids are unique, stable
  reference keys are unique, and duplicate keys are rejected even when every
  payload byte is identical;
- symbol and variable manifests are sorted, unique, and complete for generated
  clause validation;
- generated clauses are sorted by `clause_id`, unique, and accepted by the
  `clause` module;
- substitution entries are sorted by `substitution_id`, unique, structurally
  decoded, and have sorted/unique freshness and free-variable reference lists;
- resolution steps have unique step ids in canonical order;
- resolution parent references point only to existing imported fact ids,
  generated clause ids, or earlier resolution-step ids; self references and
  forward resolution-step references are malformed;
- pivot literals are structurally valid;
- `ResolutionStep.generated_clause` must use the `generated_clause` namespace
  and point to an existing generated clause id;
- `DerivedFact.source` may use the imported axiom, imported theorem, generated
  clause, or resolution-step namespace; imported and generated ids must exist,
  and resolution-step ids must refer to an earlier step;
- pivot polarity and resolvent equality are left to `resolution_trace`;
- derived fact ids are sorted and unique;
- final-goal references are syntactically valid and point to an existing
  generated clause, resolution step, or derived fact.

The parser must not synthesize missing references, infer missing symbols,
insert coercions, expand definitions, search for alternate parents, or accept a
certificate because a backend said it was proved.

## Ordering And Hashing

All parsed collections use deterministic order:

1. section tag;
2. stable namespace id;
3. stable reference key within the namespace;
4. canonical child bytes as a final tie breaker when the section defines child
   encodings.

The parser exposes canonical certificate hash input bytes. It does not compute
a cryptographic digest. Hash input bytes include the domain separator, schema
version, encoding version, the full `KernelProfileRecord` including
`certificate_hash_input_algorithm`, `target_vc`, section directory, and
canonical section payload bytes.

Hash input bytes must not include file paths, source ranges, display names,
backend stdout/stderr, timestamps, elapsed time, allocation addresses, map/set
iteration order, worker completion order, cache keys, artifact paths, or policy
projection outcomes.

## Failure Classes And Locations

Task 5 should report module-local structural errors with:

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

`detail` maps parser-owned cases to the stable rejection reasons that
`rejection.md` will finalize:

| Parser-owned case | Stable detail |
|---|---|
| unsupported schema, encoding, kernel profile, hash-input algorithm, or unknown section tag | `unsupported_certificate_format` |
| target VC mismatch | `context_mismatch` |
| malformed envelope, directory, frame, field, id ordering, duplicate id, malformed reference, or noncanonical generated clause | `malformed_certificate` |
| generated clause rejected by the clause module | `malformed_certificate` |
| count, byte, `u32` length/offset, or recursion-depth exhaustion | `resource_exhaustion` |

Pure parsing never emits `timeout`; a later deterministic budget wrapper may
turn a budget stop into timeout, but task 5 parser tests must assert that pure
parser failures are never timeout. All parser errors are non-acceptance
outcomes.

## Gap Classification

- `spec_gap`: architecture 15 names the logical certificate top level but does
  not define concrete bytes, section tags, or manifest fields. This module
  spec closes that gap for task 5 by defining schema `1`, encoding `1`, the
  normalized kernel envelope, section directory, item payloads, parser-owned
  manifests, and parser-owned failure locations.
- `test_gap`: no Rust tests currently cover normalized certificate parsing,
  section canonicality, reference validation, profile rejection, stable
  failure detail/location, hash input coverage, or parser resource limits.
- `external_dependency_gap`: normalized certificate producers in `mizar-atp`
  and proof-witness consumers in `mizar-proof`, `mizar-cache`, and
  `mizar-artifact` are absent or incomplete and must not be mocked here.
- `deferred`: backend-specific MiniSAT proof payload translation, artifact
  witness storage, cache reuse validation, and source-derived `.miz`
  certificate corpora remain outside task 5.

## Planned Tests

Task 5 must add Rust tests for:

- a minimal valid normalized certificate;
- unsupported schema, encoding, profile, hash-input algorithm, and unknown
  section rejection as `certificate_rejection` with
  `unsupported_certificate_format`;
- target VC mismatch rejection as `certificate_rejection` with
  `context_mismatch`;
- missing, duplicate, unknown, out-of-order, overlapping, non-contiguous,
  truncated, and trailing section data rejection with deterministic byte and
  section locations;
- item count mismatch, item-tag mismatch, section-tag mismatch, malformed
  field, and trailing item payload rejection with deterministic item and field
  locations;
- `max_certificate_bytes`, `max_section_bytes`, per-section count limits,
  `u32` length/offset overflow, and term-recursion-depth resource exhaustion
  before large allocation;
- imported fact id ordering, duplicate-id rejection, duplicate-key rejection,
  malformed package/module/item fields, malformed fingerprint, and malformed
  required-proof-status rejection;
- symbol and variable manifest ordering, duplicate, unsupported kind, and
  completeness checks;
- generated clause validation through the `clause` module, including a
  noncanonical clause failure mapped to `malformed_certificate`;
- substitution entry structural validation, malformed freshness/free-variable
  reference lists, and no capture/freshness acceptance;
- resolution-step id ordering plus malformed parent, self parent, forward
  parent, malformed pivot literal, malformed generated-clause namespace, and
  missing generated-clause reference rejection;
- final-goal and derived-fact source namespace/reference validation;
- deterministic parsed-output ordering for symbol manifests, variable
  manifests, imported axioms, imported theorems, generated clauses,
  substitutions, resolution steps, derived facts, and canonical child-byte tie
  breakers;
- hash-input stability under shuffled but equivalent source data;
- positive hash-input coverage for the domain separator, schema version,
  encoding version, full kernel profile including hash-input algorithm,
  `target_vc`, section directory, and canonical payload bytes;
- hash exclusion for source paths, source ranges, display names, backend logs,
  timestamps, elapsed time, allocation addresses, allocation order, map/set
  iteration order, worker completion order, cache keys, artifact paths, and
  policy projection outcomes;
- stable failure-category/detail assertions for every negative case, including
  that pure parser failures never emit `timeout`;
- lint coverage showing the parser does not import or call ATP/proof/cache/
  artifact crates, resolver/checker global state, wall-clock/random APIs, or
  unordered-map/set iteration, and does not perform search.

No `.miz` fixture, expectation sidecar, or `doc/spec` change is required for
this module-spec task.
