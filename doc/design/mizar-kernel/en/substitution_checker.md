# Module: substitution_checker

> Canonical language: English. Japanese companion:
> [../ja/substitution_checker.md](../ja/substitution_checker.md).

## Purpose

The `substitution_checker` module owns deterministic replay of substitution,
alpha-conversion, freshness, and free-variable evidence carried by a normalized
kernel certificate. It refines
[architecture 15](../../architecture/en/15.kernel_certificate_format.md)
"Substitution Rule" and
[architecture 16](../../architecture/en/16.substitution_and_binding.md).

Substitution replay is evidence checking, not inference. A successful replay
proves only that the listed substitution entry is well justified by its
explicit source term, target term, binder context, and side-condition evidence.
Final proof acceptance remains owned by the later `checker` module.

## Trust Statement

This module is trusted kernel code. It must replay explicit substitution and
binder evidence, compare normalized results deterministically, and fail closed
when a claimed result or side condition does not match.

The module must not perform proof search, ATP search, premise selection,
overload resolution, cluster search, implicit coercion insertion, fallback
inference, hidden binder repair, source-name lookup, cache lookup, artifact
lookup, wall-clock or random-state reads, unordered iteration, or hidden reads
of mutable compiler-global state. It must not accept a substitution because a
resolver, checker, ATP backend, cache, or artifact previously accepted it.

Task 20 audits this trust boundary as including no proof search, no SAT
solving, no ATP search or backend invocation, no premise selection, no overload
resolution, no cluster search, no implicit coercion insertion, no fallback
inference, no acceptance from backend-reported success alone, no source
loading, no cache lookup, no artifact lookup, no wall-clock or random-state
reads, no unordered iteration dependence, and no hidden reads of mutable
compiler-global state.

## Owned Behavior

The module owns:

- deriving an explicit substitution replay context from parsed certificate data
  and caller-supplied replay limits;
- decoding and validating the parser-owned `binder_context_encoding` for each
  substitution entry;
- replaying capture-avoiding substitution from `source_term` to `target_term`;
- validating deterministic alpha-conversion and freshness witnesses;
- validating free-variable side conditions;
- recording checked substitution entries for later checker orchestration;
- mapping replay failures to stable `rejection` records.

The module does not own:

- normalized certificate byte parsing or structural sorting of substitution
  ids;
- imported fact availability or proof-status validation;
- resolution trace replay;
- cluster or reduction trace replay;
- final proof acceptance, proof-policy projection, witness storage, cache
  reuse, or artifact emission;
- computing or selecting substitutions not explicitly recorded in the
  certificate.

## Input And Context

Task 11 and task 12 should implement replay from explicit immutable inputs:

```text
SubstitutionCheckInput
  target_vc_fingerprint
  parsed_certificate
  substitution_context
  replay_limits
```

`target_vc_fingerprint` is caller-owned and is copied only into stable
rejection records or private report binding checks. It is not derived from
backend output, cache state, artifact state, or mutable compiler-global state.

`parsed_certificate` is a `certificate_parser::ParsedCertificate`. The parser
has already checked section order, stable substitution id uniqueness,
substitution id ordering, term byte shape, sorted/unique freshness witness
reference lists, and sorted/unique free-variable constraint reference lists. The
substitution checker may assert those invariants in defensive tests, but it
must not duplicate byte parsing.

`substitution_context` is caller-supplied immutable data:

```text
SubstitutionContext
  substitution_payloads: sorted map substitution_id -> SubstitutionPayload
  freshness_witnesses: sorted map witness_id -> FreshnessWitness
  free_variable_constraints: sorted map constraint_id -> FreeVariableConstraint
  provenance_fingerprint
```

The concrete Rust type may avoid map dependencies by using sorted vectors, as
long as lookup and iteration are deterministic. The context is not populated
from resolver state, checker state, ATP output, cache state, artifact state, or
global compiler state. Missing substitution context, missing context
provenance, missing substitution payloads, missing referenced witness ids, or
missing referenced constraint ids is `missing_provenance`. Duplicate
substitution payload ids, witness ids, or constraint ids are invalid context
shape; the context constructor must reject them deterministically before replay
with a module-local context error. If an unchecked context shape reaches replay,
the ambiguous id is treated as `missing_provenance` at the failed reference.

The checker validates context entries at first use in deterministic
substitution-id order. It must not scan and reject unused context entries.
Unavailable imported facts, source definitions, or proof-status mismatches are
owned by later `checker` imported-fact tasks and remain `external_dependency_gap`
for this module.

The normalized certificate schema currently carries `source_term`,
`target_term`, binder-context bytes, and side-condition references for each
substitution entry. It does not yet inline a formal-to-actual map or rewrite
payload. Until a later certificate-schema task moves that payload into the
parsed certificate, the replay payload is explicit caller-supplied context
evidence keyed by `substitution_id`. Task 11 must reject a referenced
substitution without a payload as `missing_provenance`; it must not infer a
payload by diffing `source_term` and `target_term`.

Decoded binder contexts are not caller-supplied authority. They are internal
per-entry values derived only from each certificate entry's
`binder_context_encoding`; caller context data must never override, repair, or
cache them as proof evidence.

## Binder Evidence Model

`binder_context_encoding` is parser-owned opaque bytes until this module
decodes it. Task 11 should specify and implement the first kernel-owned binder
context grammar before accepting any substitution:

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

Accepted `binder_role` tags are:

| Tag | Role |
|---|---|
| `0x01` | universal binder |
| `0x02` | existential binder |
| `0x03` | definition formal binder |
| `0x04` | generated fresh binder |
| `0x05` | schematic binder |

The grammar inherits certificate encoding v1 scalar rules: all integers are
unsigned big-endian values, lists carry exact counts, byte ranges must be
consumed exactly, and trailing bytes are invalid. Unknown binder-context schema
versions, unknown binder roles, truncated fields, length overflows, duplicate
frames, and noncanonical ordering are `invalid_substitution` unless they exceed
a deterministic resource limit first.

Frames must be sorted by `canonical_index`, unique by `binder_id` and
`variable_id`, manifest-compatible, and compatible with the normalized term
encodings used by `source_term`, `target_term`, and payload `actual_term`
records. A normalized term may not reuse a `binder_id` for two binder nodes.
Free and schematic variable lists must be manifest-compatible, sorted, and
unique. Display names and source ranges are not encoded and never participate
in semantic equality.

The checker may consume stable data shapes and contracts from `mizar-core`, but
it must independently re-check binder evidence inside `mizar-kernel`. Calling a
`mizar-core` API that simply reports "already valid" is not sufficient proof.
The kernel-owned check must compare stable ids, binder depth, alpha-normalized
structure, and side-condition evidence explicitly.

Substitution payloads are concrete evidence records:

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

The only accepted `payload_kind` tag in task 11 is
`0x01 = formal_to_actual_map`. Tag `0x02 = local_abbreviation_expansion` is
reserved and deferred until definition-site closure and type-guard evidence are
specified; a present `0x02` payload is rejected as `invalid_substitution` rather
than accepted without guard evidence. Accepted `replacement_role` tags are
`0x01 = term_argument` and `0x02 = predicate_argument`. Role
`0x03 = captured_free_variable` is reserved and deferred with local
abbreviation closure evidence; a present `0x03` role is rejected as
`invalid_substitution` in task 11. Replacements must be sorted and unique by
`formal_variable_id`, every `actual_term` must validate against the certificate
symbol and variable manifests and term limits, `rewrite_path` must address an
existing node in `source_term`, and `owner_substitution_id` must match the
entry currently being checked. Malformed present payload records are
`invalid_substitution`; absent referenced payload records are
`missing_provenance`.

Side-condition context entries are concrete evidence records:

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

Accepted `constraint_kind` tags are `0x01 = must_remain_free_at_path` and
`0x02 = must_be_absent_from_capture_set`. Accepted `edge_kind` tags are
`0x01 = application_argument` and `0x02 = binder_body`. Variable lists must be
sorted and unique. `owner_substitution_id` must match the entry currently being
checked. `binder_path` and `term_path` must address existing normalized term
nodes. `deterministic_counter` must match the freshness counter produced by the
architecture 16 strategy for that path and role.

Task 11 only checks that `deterministic_counter` is present in the record whose
owner, path, and list bounds are otherwise valid. Task 12 owns verification that
the counter value matches the architecture 16 freshness strategy.

Task 12 uses the certificate-visible instantiation of the architecture 16
strategy. A freshness witness is semantically valid only when all of the
following hold:

- `binder_path` selects a binder node in `source_term`;
- the same normalized path selects a binder node in `target_term`;
- the source binder frame supplies the original bound variable and role;
- the target binder frame has role `generated fresh binder` and
  `variable_id == generated_variable_id`;
- `avoided_variables` exactly equals the recomputed sorted set containing the
  free variables of the source binder body except the original bound variable,
  plus the free variables of replacement actual terms inserted below that
  binder;
- `generated_variable_id` is absent from that recomputed avoided set;
- `deterministic_counter` is the zero-based position of
  `generated_variable_id` in the certificate variable manifest sorted by stable
  id after filtering out the recomputed avoided set.

The witness owner and binder path are part of the replay envelope for the
source or certificate owner and role data required by architecture 16. This
module must not consult producer state to fill in missing owner, role, or
source identity. If the available normalized evidence is insufficient to
recompute the candidate stream above, the witness is `invalid_substitution`.

Malformed present witness or constraint records are `invalid_substitution`;
absent referenced witness or constraint records are `missing_provenance`.

## Replay Limits

Replay limits are deterministic:

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

Exceeding a limit is `kernel_rejection` with `resource_exhaustion`. A budget
must be checked before allocating a large temporary context, term, payload,
replacement list, witness list, path segment list, avoided-variable set,
capture set, or free-variable set.

Term validation must use explicit certificate symbol and variable manifests,
the same stable term encoding rules as `clause`, and the configured term-size
and term-depth limits. The module must not recursively walk caller-supplied
terms without a deterministic depth budget.

## Substitution Replay

For each `SubstitutionEntry` in parsed certificate order:

1. Check `max_substitutions`.
2. Decode the entry's `binder_context_encoding` within
   `max_binder_context_bytes` and `max_binder_frames`.
3. Validate that `source_term` and `target_term` are structurally compatible
   with the decoded binder context, symbol manifest, variable manifest, and
   replay limits.
4. Resolve the `substitution_id` payload and each `freshness_witness_refs` id and
   `free_variable_constraint_refs` id from the explicit
   `substitution_context`.
5. Replay the explicit payload as capture-avoiding substitution using only
   stable ids, normalized binder positions, and recorded replacement entries.
6. Apply deterministic alpha-renaming only when the recorded binder context and
   freshness witnesses require it; generated fresh ids must follow the
   deterministic strategy from architecture 16.
7. Compare the replayed normalized result structurally with `target_term`.
8. Record the checked substitution id and normalized target for later checker
   orchestration.

Task 11 owns steps 1-5 and result comparison for direct substitution, including
payload shape, owner, rewrite-path, replacement ordering, resource, and
manifest validation. Task 11 validates referenced freshness and free-variable
records only for presence, shape, owner, path bounds, and deterministic
first-use behavior; semantic freshness and free-variable replay remain task 12.
Task 11 also checks the replayed target size and depth before allocating the
replayed term. A replacement of a variable occurrence beneath an active binder
is rejected in task 11 unless it is first rejected as a direct capture violation;
semantic acceptance of such under-binder substitutions is deferred to task 12.
Task 12 extends the same module with alpha-conversion, freshness, and
free-variable side-condition replay. Both tasks must use one coherent evidence
model.

The checker must not synthesize missing freshness witnesses, guess a binder
renaming, consult display names, infer omitted template arguments, insert
implicit coercions, or search for an alternate substitution that makes the
target term match.

## Alpha-Conversion And Freshness

Alpha-equivalence is decided by normalized binder structure and stable ids, not
source names or rendered text. A claimed alpha-conversion is valid only when:

- the two normalized terms differ only by a consistent renaming of bound
  variables within the decoded binder context;
- every renamed binder is paired by identical normalized source and target
  `binder_path` positions and by one referenced freshness witness;
- the source binder's frame supplies the original variable id, and every bound
  occurrence of that variable in the source binder scope is rewritten to the
  target generated variable id while replacement actual terms remain unchanged;
- the renaming is injective within each binder scope;
- no free variable becomes bound and no bound variable escapes its scope;
- every generated fresh id is justified by a referenced freshness witness;
- the deterministic freshness strategy from architecture 16 produces the same
  ids in the same order.

Freshness witnesses are evidence, not suggestions. A malformed,
out-of-context, stale, inconsistent, or unused-where-required present witness is
`invalid_substitution`. A referenced witness absent from the supplied context is
`missing_provenance`. Duplicate witness ids in the supplied context follow the
context-shape rule above.

## Free-Variable Conditions

Free-variable constraints are validated over normalized binder structure:

- entering a binder removes that binder variable from the free-variable set of
  its body;
- shadowing is represented by distinct stable ids;
- source display names never decide free-variable identity;
- a variable required to remain free must remain free at the recorded term path
  after replay;
- the checker recomputes the capture set at the recorded target path from the
  active target binder stack, and the recorded `capture_set` must exactly match
  the recomputed sorted set before any constraint predicate is accepted;
- a variable required to be absent from a capture set must not occur in that
  recomputed set;
- free-variable sets and constraint ids are sorted deterministically before any
  comparison or report emission.

A violated free-variable condition is `invalid_substitution`. A referenced
constraint id absent from `substitution_context` is `missing_provenance`.

## Report And Checker Interaction

The success report should expose deterministic checked-entry data only:

```text
SubstitutionCheckReport
  checked_substitutions: sorted Vec<CheckedSubstitution>

CheckedSubstitution
  substitution_id
  source_term
  target_term
```

The report is evidence-replay output for later checker orchestration. It must
not contain accepted proof status, policy outcome, used-axiom projection, or
artifact-facing witness decisions. The implementation may carry private replay
binding data, such as the caller-owned target fingerprint, certificate hash
input, and `substitution_context.provenance_fingerprint`, solely to reject
accidental pairing of a report with a different replay input or different
payload context. Accessors must still expose only checked substitution data.

`derived_fact` validation and final-goal acceptance remain outside this module
until `checker.md` specifies how checked substitutions, resolution steps,
cluster traces, and imported facts compose into a trusted proof result.

## Rejection Mapping

Substitution failures produce `kernel_rejection` records:

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

Every rejection location must be deterministic and must use the shared
`RejectionLocation` fields from `rejection.md`, especially `substitution_id`
and field paths. Human diagnostics may include extra text, but extra text must
not affect acceptance, ordering, or stable detail keys.

## Determinism And Cost

Replay order is the parsed substitution order, which the parser keeps sorted by
`substitution_id`. Context constructors must canonicalize input order or reject
duplicates deterministically before replay. The checker must use sorted vectors
or other deterministic data structures for binder frames, witness ids,
constraint ids, free-variable sets, and checked reports.

The result for identical parsed certificates, substitution contexts, and replay
limits must be byte-for-byte stable across platforms and worker counts. Worker
completion order, allocation addresses, source display names, backend logs,
cache keys, artifact paths, wall-clock time, and random state must not affect
any accepted/rejected result or rejection ordering.

Replay cost is linear in the size of checked substitution entries plus the size
of explicitly referenced binder, payload, replacement, witness, path, and
free-variable evidence, within the configured limits. The checker must not
perform transitive proof search or scan unrelated context entries.

## Gap Classification

- `spec_gap`: architecture 15 and 16 define substitution and binder principles
  but not the concrete `mizar-kernel` module contract, binder-context grammar,
  report shape, rejection locations, or task-11/task-12 split. This task closes
  that gap for the planned implementation tasks.
- `test_gap`: task 11 closed direct-substitution replay coverage for valid
  replay, explicit payload validation, target mismatch, capture violation,
  malformed binder context, missing provenance, resource limits, and
  deterministic reports. Task 12 closes alpha-equivalence, deterministic
  freshness, free-variable/capture-set, alpha-resource, and shuffled-context
  coverage for this module. Source-derived corpus fixtures remain deferred to
  later checker/source integration tasks.
- `external_dependency_gap`: source-derived substitution certificates and
  downstream proof/cache/artifact consumers are not active integration points
  for this module. Inline certificate encoding of substitution payloads is also
  deferred until a future schema task; task 11 consumes explicit immutable
  context payload evidence and rejects missing payloads rather than guessing.
  Local-abbreviation expansion payloads are deferred until definition-site
  closure and type-guard evidence are specified; captured-free-variable
  replacement roles are deferred with that same closure evidence.
- `deferred`: imported fact checking, derived-fact assembly, final proof
  acceptance, and policy projection remain owned by later `checker` and
  downstream proof tasks.

## Planned Tests

Task 11 must add Rust tests for:

- valid direct substitution accepted with deterministic checked-entry output;
- positive payload coverage for the accepted `formal_to_actual_map`
  `payload_kind` and for each accepted `replacement_role`, including
  `term_argument` and `predicate_argument`;
- missing substitution payload rejected as `missing_provenance`, and malformed
  payload kind, replacement role, rewrite path, owner id, replacement ordering,
  duplicate replacement, and byte-valid but manifest-incompatible actual term
  rejected as `invalid_substitution`;
- deferred `local_abbreviation_expansion` payloads rejected as
  `invalid_substitution` until definition-site closure and type-guard evidence
  are specified;
- deferred `captured_free_variable` replacement roles rejected as
  `invalid_substitution` until definition-site closure evidence is specified;
- no payload inferred by diffing source and target terms;
- a positive rewrite-path case where the same formal appears at multiple source
  locations and only the recorded `rewrite_path` occurrence is rewritten;
- source/target mismatch rejected as `invalid_substitution` with stable
  `substitution_id` and field path;
- byte-valid source or target terms that are incompatible with the symbol or
  variable manifests rejected as `invalid_substitution`;
- capture violation rejected without alpha repair when no recorded witness
  justifies the repair;
- under-binder replacement that is not itself a direct capture violation
  rejected in task 11 with semantic acceptance deferred to task 12;
- malformed or noncanonical binder context rejected as `invalid_substitution`;
- binder-context decoding cases for unknown schema version, unknown binder role,
  truncated field, length overflow, duplicate frame, noncanonical frame order,
  noncanonical free/schematic variable lists, exact-consumption/trailing-byte
  failures, and frame/term incompatibility;
- missing substitution context, missing provenance, missing freshness witness,
  and missing free-variable constraint rejected as `missing_provenance`;
- malformed present freshness witness, free-variable constraint, term path,
  binder path, owner id, binder role, constraint kind, term-path edge tags,
  binder-path edge cases, unsorted or duplicate `avoided_variables`, and
  unsorted or duplicate `capture_set` rejected as `invalid_substitution`;
- substitution count, binder-context byte/frame, term-size, and term-depth
  limits, freshness-witness count, and free-variable-constraint count rejected
  as `resource_exhaustion` before large allocation or deep recursion;
- payload replacement count, term-path segment count, avoided-variable count,
  and capture-set count limits rejected as `resource_exhaustion` before
  cloning, sorting, walking, or allocating the used entries;
- over-budget `Replacement.actual_term` size and depth rejected as
  `resource_exhaustion` before walking or allocating payload actual terms;
- over-budget replayed target size and depth rejected as `resource_exhaustion`
  before cloning or allocating the replayed term;
- context construction canonicalizing input order and rejecting duplicate
  payload, witness, or constraint ids deterministically before replay, plus the
  replay fallback mapping for any unchecked ambiguous context shape;
- first-use context validation ignoring unused malformed or stale extra
  payloads, witnesses, and constraints;
- every rejection class asserting stable category/detail keys, caller-owned
  target fingerprint propagation, and precise deterministic `RejectionLocation`
  fields;
- report/input private binding preventing accidental report reuse for both
  target-fingerprint mismatch, certificate/hash-input mismatch, and
  substitution-context provenance mismatch;
- lint coverage showing no proof search, ATP/proof/cache/artifact coupling,
  overload resolution, cluster search, implicit coercion insertion, fallback
  inference, hidden binder repair, source-name/display-name lookup, omitted
  template-argument inference, unordered iteration, wall-clock/random read, or
  global mutable-state read.

Task 12 adds Rust tests for:

- alpha-equivalent terms accepted by normalized binder structure, not display
  names;
- inconsistent alpha renaming rejected as `invalid_substitution`;
- deterministic freshness witnesses accepted only when they match the
  architecture 16 freshness strategy;
- missing, out-of-context, stale, or inconsistent freshness witnesses rejected,
  unused-where-required present witnesses rejected, with duplicate context ids
  covered by task-11 context-construction tests;
- free-variable constraints accepted for valid paths and rejected for capture,
  escaping binders, shadowing confusion, or missing variables;
- alpha-rename and free-variable resource limits rejected as
  `resource_exhaustion`;
- freshness-witness and free-variable-constraint count limits asserted again
  when alpha/freshness/FV replay introduces additional side-condition fixtures;
- deterministic report and rejection ordering under shuffled context fixture
  construction.
