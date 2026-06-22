# mizar-checker cluster trace design

> Canonical language: English. Japanese companion: [../ja/cluster_trace.md](../ja/cluster_trace.md).

## Purpose

This document refines the canonical `ResolutionTrace` schema from
[architecture 17](../../architecture/en/17.cluster_trace_format.md) for the
`mizar-checker` phase-7 implementation. It does not fork the artifact schema.
The checker emits the same cluster and reduction step concepts defined by the
architecture document, and this document fixes the checker-local ownership,
ordering, replay, diagnostics, and planned test boundaries for tasks 16-18.

Task 15 is documentation-only. It introduces no source behavior, no artifact
writer, and no parser or resolver payload extraction.

## Authority And Scope

Primary authorities:

- [spec 17](../../../spec/en/17.clusters_and_registrations.md): cluster rules,
  reduction termination, deterministic reduction strategy, and reduction
  traceability;
- [spec 23.7.7](../../../spec/en/23.package_management_and_build_system.md#2377-storage-format-cluster-db-resolution-trace-diagnostic-explanation):
  `resolution-trace/` storage and minimum-kernel replay requirements;
- [architecture 01](../../architecture/en/01.ir_layers.md): `ResolutionTrace`
  ownership as an immutable source-shaped IR layer;
- [architecture 04](../../architecture/en/04.type_and_registration_resolution.md):
  phase-7 registration resolution inputs and outputs;
- [architecture 17](../../architecture/en/17.cluster_trace_format.md): the
  canonical replayable trace schema.

In scope:

- checker-local construction of `ResolutionTrace` records for derived cluster
  facts and automatic reduction applications;
- deterministic step ids, fact references, traversal profiles, and debug
  rendering;
- replay input contracts for diagnostics, artifact validators, and the minimum
  kernel;
- failure classification for loops, bounded saturation, invalid substitutions,
  mismatched strategy-audit keys, and invisible registrations;
- planned Rust and corpus coverage for tasks 16-18 and later determinism work.

Out of scope:

- activation of pending registrations, proof acceptance, or verifier policy
  decisions owned by task 19 and later proof crates;
- source-to-checker payload extraction for registration patterns, reduction
  terms, or guard evidence while MC-G021 remains open;
- JSON artifact emission, cache storage, or artifact reader compatibility,
  which are owned by build/artifact tasks using the canonical schema;
- ATP proof search, new theorem facts, overload candidate selection, or hidden
  coercion insertion.

## Trace Model

`ResolutionTrace` is source-file scoped. A trace is constructed from:

- the source/module identity for the checked file;
- the phase-6 `TypedAst` and `TypeFactTable`;
- the task-14 registration database, restricted to activated registrations;
- checker-ready cluster and reduction payloads supplied by later tasks;
- deterministic traversal settings such as cluster expansion depth and maximum
  generated fact count.

The trace output contains:

- ordered cluster and reduction steps;
- derived cluster facts that were added to the type fact table;
- traversal profile metadata including configured bounds and the ordering
  version.

The checker may emit diagnostics while constructing or validating a trace, and
the traversal profile may record diagnostic counts or stable diagnostic
references. Those diagnostics are not additional `ResolutionTrace` schema
fields; detailed diagnostic payloads remain in the diagnostics/explanation
channel defined by the canonical artifact model.

Pending, rejected, recovered, malformed, or unaccepted registrations are not
trace inputs. If a later operation asks the trace builder to use one, the
operation is rejected with a deterministic diagnostic instead of fabricating an
active step.

### Task 16: Cluster Closure Data Layer

Task 16 implements the cluster side of this model as `src/cluster_trace.rs`.

The first implementation exposes `ClusterTraceBuilder`, `ClusterRuleInput`,
`ClusterFactInput`, `ClusterClosureOutput`, `ResolutionTrace`, cluster steps,
closure fact tables, traversal profiles, replay reports, and checker-local
diagnostics. The builder consumes a task-14 `RegistrationDatabase` plus
explicit checker-owned rule and fact payloads. It fires only activated resolver
registrations whose kind is `Cluster` and whose activation trigger plus
activation fingerprint, or accepted pattern fallback when no fingerprint is
present, matches the checker-owned rule payload.

Derived facts are recorded in a checker-owned `ClusterFactTable`, deduplicated
by canonical `ClusterFactFingerprint`, and every derived fact has
`ClusterFactProvenance::TraceStep`. This task does not mutate the phase-6
`TypeFactTable` directly; mapping traced cluster facts into the shared type
fact table remains deferred until the source-to-checker and registration
payload seams can provide typed subjects and predicates without fabrication.

Task 16 rejects pending, rejected, malformed, recovered, unknown, non-cluster,
existential-gating, trigger-mismatched, and fingerprint-mismatched rule inputs
with checker-local diagnostics. Diagnostics are stored outside the
`ResolutionTrace` schema. The emitted trace contains cluster steps, derived
facts, and a traversal profile with reduction step count fixed to zero.
Replay takes the active task-14 registration database and revalidates the
accepted cluster identity, resolver id, fingerprint/pattern payload, and audit
key before replaying derived facts.

Task 16 deliberately does not implement reduction steps, loop detection,
bounded-saturation failure, contradiction failure, artifact JSON emission,
cache readers, existential gating, proof acceptance, or opaque resolver-shell
parsing. Those remain owned by tasks 17-20 and artifact/build integration.

### Task 17: Cluster Loop And Bound Data Layer

Task 17 extends the task-16 cluster closure data layer without changing the
canonical artifact schema. It implements checker-local loop, bound, and
contradiction failure handling over explicit `ClusterRuleInput` and
`ClusterFactInput` payloads.

The builder tracks derivation ancestry by canonical `ClusterFactFingerprint`.
If an applicable rule would derive a fact that is already on an antecedent's
active ancestry path, the rule is rejected with a deterministic `cluster_loop`
diagnostic. Repeated already-derived facts that are not on the active ancestry
path remain ordinary duplicate closure facts and are ignored only after
fingerprint equality.

`ClusterTraversalConfig` bounds are enforced during closure. A candidate whose
derived depth would exceed `max_cluster_depth`, or whose insertion would exceed
`max_generated_facts`, is rejected with a deterministic
`cluster_bound_exceeded` diagnostic. The traversal profile records the
configured bounds, whether bounded saturation was reached, and stable cache-key
material derived from the ordering version and bound settings. The rejected
candidate is not inserted into `closure_facts`, `derived_facts`, or trace steps.
Depth is measured over the explicit fact-dependency hypergraph: input facts
have depth `0`; a derived fact with antecedents has depth
`1 + max(antecedent depths)`; and a zero-antecedent cluster-generated fact has
depth `1`.
Loop, bound, and contradiction failures set the checker-local
`ClusterClosureOutput` status to incomplete. An incomplete output may still
carry facts that were derived before the fatal candidate, but those facts must
not be exported as a verified closure result.

Contradiction handling remains checker-owned at this seam. Task 17 allows an
explicit rule payload to list already-visible fact fingerprints that conflict
with the generated fact. If any listed fact is present when the rule would fire,
the builder emits `cluster_contradiction` and does not export a verified or
degraded closure fact for the contradictory generated fact. A contradiction is
a fatal closure result for verified export: the checker must not publish a
truncated or degraded verified fact set from that closure. Source-derived
incompatibility checks against the shared `TypeFactTable` remain deferred until
source-to-checker payload extraction and registration payloads are available.

Task 17 does not implement reduction steps, artifact JSON emission, cache
readers, existential gating, proof acceptance, or opaque resolver-shell parsing.
Those remain owned by later tasks and external integration work.

## Cluster Steps

A cluster step refines the architecture `ClusterStep` fields:

```text
ClusterStep
  source_type
  applied_cluster
  generated_attribute
  generated_type
  dependency
  source_range
```

The checker records `dependency` as an ordered set of antecedent fact
references plus the applied active registration identity. Antecedent references
must point to facts that are visible before the step: phase-6 input facts,
earlier trace-derived facts, local assumptions explicitly exposed to phase 7,
or accepted cited facts. A zero-antecedent cluster records an explicit empty
antecedent list.

Each cluster step must preserve:

- the active checker registration id and resolver registration provenance;
- the checker-owned registration fingerprint or pattern fingerprint supplied by
  the activation payload;
- the generated attribute and generated type fingerprints;
- the single-consequent rule view after multi-consequent clusters are split;
- source provenance for the rule and for the fact site that triggered it;
- an audit key built from the source type id, cluster origin module path,
  declaration source order, generated attribute id, and registration
  fingerprint.

Cluster steps are replayed in order. Replay starts from the input fact set,
checks that all antecedents are already present, checks that the active cluster
registration is accepted and visible, then adds the generated fact. Replay does
not run search, infer missing antecedents, or collapse transitive chains.

## Reduction Steps

A reduction step uses the architecture `ReductionStep` fields without changing
their meaning:

```text
ReductionStep
  applied_reduction
  rule_fqn
  enclosing_term_before
  redex_path
  source_redex
  target_term
  substitution
  discharged_guards
  rule_view
  selection_key
  source_range
```

The kernel-replay layer is `applied_reduction`, `rule_fqn`, `source_redex`,
`target_term`, `substitution`, and `discharged_guards`. The strategy-audit layer
is `enclosing_term_before`, `redex_path`, `rule_view`, and `selection_key`.
These names and meanings stay aligned with architecture 17 and spec 23.7.7.

Reduction replay checks only the local rewrite instance against an already
accepted `reducibility` registration: the redex matches the rule `LHS`, the
target is the corresponding `RHS` instance, each pattern binding is valid, and
every type, attribute, and `such` guard has stable evidence. `such` evidence is
an applicability side condition only; it does not make a rule more specific.
The minimum kernel does not search for matching rules or reselect reductions.

The strategy-audit key records the leftmost-innermost redex path, the active
rule-view fingerprint, and the specificity/FQN selection key required by
spec 17.6.4. A mismatched audit key is a trace validation failure even when the
local rewrite could be replayed.

## Determinism

Cluster traversal order is the architecture-17 order:

1. source type canonical id;
2. cluster origin module path;
3. declaration source order;
4. generated attribute canonical id;
5. registration fingerprint.

The checker uses that order for the worklist, for per-trigger candidate lists,
for trace step ids, and for diagnostics. Worker completion order, hash-map
iteration, import order, cache insertion order, and activation input order must
not change the emitted trace.

Cluster closure records every intermediate step. A chain `A -> B -> C` must be
stored as two steps unless a later artifact also preserves the original steps by
content-addressed reference. Derived fact de-duplication is allowed only after
canonical fact fingerprints match.

Reduction normalization follows spec 17.6.4: left-to-right rewriting,
leftmost-innermost redex traversal, matching modulo alpha-equivalence and
binding, most-specific rule selection, and FQN tie-breaks for remaining
matches. Reduction order is not capped by cluster expansion depth; termination
comes from registration-time simplification-order validation.

## Bounds And Failures

The traversal profile records at least:

- schema/order version;
- maximum cluster expansion depth;
- maximum generated cluster fact count;
- whether bounded saturation was reached;
- counts for input facts, derived facts, cluster steps, reduction steps, and
  diagnostics.

Exceeding a cluster depth or generated-fact bound is a bounded failure. The
checker must emit a deterministic diagnostic and avoid exporting degraded facts
as verified closure results. It must not silently truncate a required closure.

Failure classes reserved for tasks 16-18:

| Class | Meaning |
|---|---|
| `cluster_loop` | a rule would revisit an active expansion stack fingerprint |
| `cluster_bound_exceeded` | configured cluster depth or generated-fact count was exceeded |
| `cluster_contradiction` | derived facts conflict in a way the type fact table cannot accept |
| `invisible_registration` | pending, rejected, recovered, malformed, or unaccepted registration was requested |
| `invalid_reduction_substitution` | recorded substitution does not instantiate the rule pattern |
| `missing_guard_evidence` | type, attribute, or `such` guard has no stable evidence |
| `strategy_audit_mismatch` | emitted reduction step disagrees with deterministic strategy audit |

These names are checker-local classes until a public diagnostics code space is
allocated. Stable detail keys may be used in tests.

## Replay Cost

Trace replay must be linear or near-linear in trace size. Implementations may
build local maps from fact fingerprints, step ids, rule fingerprints, and term
fingerprints before replay, but must not perform global proof search, overload
resolution, or cluster search during replay.

Replay consumers may:

- validate artifacts and kernel certificates;
- explain derived facts and reductions in diagnostics and `@show_resolution`;
- compute dependency fingerprints for incremental builds;
- include replayed facts in VC inputs.

Replay consumers must not infer additional cluster facts, apply additional
reductions, or repair missing steps.

## External And Deferred Inputs

Task 15 classifies the following as open:

- `external_dependency_gap` / `deferred`: MC-G021 still blocks real cluster and
  reduction steps because checker-ready registration patterns, parameter
  payloads, accepted correctness payloads, reduction `LHS`/`RHS` terms, guard
  evidence, and active dependency summaries are not available from resolver or
  artifact inputs.
- `deferred`: task 15 does not implement `src/cluster_trace.rs`; all source
  behavior and Rust tests start in task 16.
- `deferred`: artifact JSON emission and cache compatibility are owned by
  build/artifact tasks that must consume architecture 17 rather than a
  checker-local schema fork.
- `test_gap`: active `.miz` cluster/reduction semantic fixtures remain
  deferred until tasks 16-18 have checker-owned payload seams; broader corpus
  coverage is owned by task 29.

## Planned Tests

Task 16:

- closure fixtures produce replayable derived facts;
- pending, rejected, recovered, malformed, and unaccepted registrations do not
  fire;
- same input emits the same trace and diagnostics across registration/input
  permutations;
- subtype-compatible conditional clusters record all antecedent fact refs;
- transitive chains keep every intermediate step.

Task 17:

- direct and indirect loops terminate with `cluster_loop`;
- depth and generated-fact bounds are visible in the traversal profile and
  cache key material;
- bounded saturation does not export degraded verified facts;
- incompatible derived facts emit stable `cluster_contradiction` diagnostics
  and export no verified or degraded closure fact for the contradiction;
- duplicate derived facts are ignored only after fingerprint equality.

Task 18:

- reduction steps record enclosing-term fingerprint, redex path, source redex,
  target term, substitution, guard evidence, rule FQN, rule-view fingerprint,
  selection key, and source provenance;
- invalid substitutions, missing guard evidence, and mismatched strategy-audit
  keys are diagnosed;
- `such` side conditions are applicability-only;
- pending, rejected, recovered, malformed, or unaccepted reductions do not
  rewrite terms;
- replay reaches the same term fingerprints as the emitted trace.

Later tasks:

- task 29 adds active `.miz` coverage after checker-owned cluster/reduction
  payloads exist;
- task 30 adds determinism regression/property coverage for trace ordering and
  diagnostic ordering;
- task 32 audits that source, docs, and tests still agree.
