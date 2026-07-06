# mizar-checker: Overload Resolution

> Canonical language: English. Japanese companion:
> [../ja/overload_resolution.md](../ja/overload_resolution.md).

## Purpose

`overload_resolution` specifies phase-8 checker behavior after type checking
and registration resolution have produced typed sites, candidate groups,
recorded facts, coercion candidates, and replayable registration traces. It
refines:

- [architecture 05](../../architecture/en/05.overload_resolution.md);
- [spec chapter 18](../../../spec/en/18.templates.md) templates and template
  inference;
- [spec chapter 19](../../../spec/en/19.overload_resolution.md) ordinary
  overload selection, `qua`, and `redefine`;
- [`typed_ast.md`](./typed_ast.md) typed sites, type facts, and coercion
  candidates;
- [`registration_resolution.md`](./registration_resolution.md) activated
  registration and gate boundaries;
- [`cluster_trace.md`](./cluster_trace.md) recorded closure and reduction trace
  facts.

Task 21 is documentation-only. It introduces no Rust source, no new language
behavior, no source extraction, no artifact writer, and no proof acceptance.
Task 22 implements the checker-local site/candidate collection data layer for
explicit payloads. Task 23 implements the checker-local template expansion
data layer over those payloads. Task 24 implements the checker-local viability
filtering data layer over explicit recorded-evidence payloads. Tasks 25-26
implement the remaining named sections below. Task 28 later assembles the
final `ResolvedTypedAst` data shape specified by `resolved_typed_ast.md`.

## Boundary

`overload_resolution` owns:

- collection of final overload sites from checker-owned typed payloads;
- concrete candidate records with declaration, root, template, and provenance
  metadata;
- template expansion before ordinary candidate ordering;
- viability filtering over already recorded type facts, coercion candidates,
  and source-written `qua`;
- per-site specificity graphs over viable ordinary roots;
- unique ordinary root selection and same-root refinement joins;
- insertion records for proof-free widening views that must be explicit to
  later phases;
- failed-site preservation with stable diagnostics and candidate lists.

`overload_resolution` does not own:

- name lookup, import visibility, declaration signature collection, or global
  `SymbolEnv` scanning;
- type-expression normalization, term/formula inference, or new candidate
  discovery from raw syntax;
- cluster closure, registration activation, reduction replay, or new fact
  derivation;
- `VcId`, proof discharge, accepted verifier status, or kernel replay;
- return-type-based ordinary overload selection;
- stable public diagnostic-code allocation while checker diagnostic code-space
  remains external.

## Inputs And Outputs

The phase consumes checker-owned payloads:

```rust
struct OverloadResolutionInput {
    typed_ast: TypedAst,
    sites: Vec<OverloadSiteInput>,
    candidates: Vec<OverloadCandidateInput>,
    facts: TypeFactTable,
    coercions: CoercionTable,
    trace: ResolutionTrace,
}
```

The input must already be scope-filtered. Current-module candidates must be
visible before the use site, imported candidates must be visible and exported
according to the resolver, and same-signature declaration conflicts must already
be represented as diagnostics or blocked candidate groups. The checker may
validate provenance for diagnostics, but it must not widen candidate sets by
searching resolver globals.

The phase produces:

```rust
struct OverloadResolutionOutput {
    sites: OverloadSiteTable,
    candidates: OverloadCandidateTable,
    template_expansions: TemplateExpansionTable,
    viability: ViabilityTable,
    specificity: SpecificityGraphTable,
    results: OverloadResultTable,
    inserted_views: InsertedViewTable,
    diagnostics: OverloadDiagnosticTable,
}
```

The output is checker-local until task 28 projects it into `ResolvedTypedAst`.
Ids are deterministic within the output but are not proof ids, artifact ids, or
cross-edit cache keys.

## Site And Candidate Collection

An overload site is a typed use whose final semantic definition is not yet
selected:

```rust
struct OverloadSiteInput {
    key: OverloadSiteKey,
    owner: TypedSiteRef,
    source_range: SourceRange,
    kind: OverloadSiteKind,
    name: OverloadNameKey,
    arguments: Vec<TypedSiteRef>,
    expected: Option<NormalizedTypeId>,
    source_qua: Vec<SourceQuaView>,
    recovery: OverloadSiteRecovery,
}
```

Site kinds include functor, predicate, attribute, mode, selector, structure
field, template name, and scheme/theorem application only when the resolver and
parser expose the required payloads. Unsupported or missing source roles are
classified as `external_dependency_gap` or `deferred`; they are not guessed
from raw syntax.

Each candidate records the already-resolved declaration identity:

```rust
struct OverloadCandidateInput {
    site: OverloadSiteKey,
    symbol: SymbolId,
    ordinary_root: SymbolId,
    declaration_kind: CandidateDeclarationKind,
    parameters: Vec<NormalizedTypeId>,
    result: Option<NormalizedTypeId>,
    origin: CandidateOrigin,
    template: Option<TemplateCandidatePayload>,
    coherence: Option<CoherenceStatus>,
    provenance: CandidateProvenance,
}

enum CoherenceStatus {
    Accepted,
    Pending,
    Rejected,
}
```

`OverloadSiteInput.key` is a caller-supplied stable key for the collection
payload. `OverloadCandidateInput.site` references that key; output
`OverloadSiteId`s are assigned only after canonical site sorting and are local
to the collection output. A candidate whose site key is unknown is diagnosed and
not inserted.

Task 22 treats site provenance as the combination of `owner`, `source_range`,
`name`, `arguments`, `source_qua`, and `recovery`. Candidate provenance is the
explicit `CandidateProvenance` payload. Both are retained for deterministic
debug rendering and diagnostics; the collector does not scan raw syntax or
resolver globals to invent missing provenance.

`result` is retained for metadata and same-root refinement joins. It never
participates in ordinary root selection. A `redefine` candidate must point to
the ordinary root it refines; same-root redefinitions are not competing roots.
Pending or rejected coherence may be retained for diagnostics, but only
accepted coherence can make a redefinition active.

Canonical ordering for sites and candidates uses source id, source range,
typed owner key, declaration kind, ordinary root, symbol id, template
instantiation key, import/source provenance, explicit declaration order, and
candidate-local id as the last duplicate tie-break. Within a site, candidates
sort by declaration kind, ordinary root, symbol id, template instantiation key,
provenance, explicit declaration order, and candidate-local id. Hash-map
iteration, worker order, import read order, and diagnostic insertion order must
not affect rendered output.

### Task 22 Collection Data Layer

`src/overload_resolution.rs` implements `OverloadCollectionOutput::collect`
for explicit `OverloadSiteInput` and `OverloadCandidateInput` vectors. It:

- assigns deterministic local `OverloadSiteId` and `OverloadCandidateId`
  values after canonical sorting;
- preserves site provenance, candidate provenance, source-written `qua`
  metadata, template payload metadata, and coherence metadata;
- validates duplicate site keys and candidate links to unknown site keys;
- emits stable checker-local diagnostics for duplicate keys, unknown links,
  unsupported roles, and recovery states while retaining the offending or
  relevant site/candidate input provenance;
- marks unsupported site and candidate roles as deferred;
- preserves already scope/visibility-filtered candidate sets as supplied.

Task 22 deliberately does not walk `TypedAst`, scan `SymbolEnv`, parse opaque
resolver shells, expand templates, run viability checks, build specificity
graphs, select roots, insert views, or project `ResolvedTypedAst`. Those remain
tasks 23-28 or MC-G027 external/deferred work.

## Template Expansion

Template expansion runs before ordinary viability and specificity.

For each candidate tagged with a template payload:

1. Classify explicit template arguments against the template parameter list.
2. If arguments are omitted, validate the caller-supplied omitted-argument
   inference payload, whose evidence must be exact argument type or
   mode-hierarchy constraint metadata already exposed before task 23.
3. Check constrained template parameters against explicit constraint evidence
   status supplied by the caller and facts visible at the site.
4. Reject inference that is missing, contradictory, ambiguous, unsupported, or
   missing required constraint evidence.
5. Instantiate successful templates into concrete candidates and retain the
   template origin for tie-breaking and diagnostics.

Task 23 does not compute omitted-argument inference from `TypedAst`, fact
tables, cluster closure, or resolver globals. It validates explicit payload
fields: template parameter list, template arguments, omitted-argument
inference payloads, source-`qua` argument status, and constraint-evidence
status. Cluster expansion is not used during omitted-argument template
inference.
Source-written template `qua_arg` is accepted only when it is a proof-free
inheritance widening exposed by the typed/coercion payloads. Predicate,
functor, scheme, and algorithm template roles remain deferred unless parser,
resolver, and checker-owned payloads expose the concrete role and parameter
mapping. Non-template candidates beat template-derived candidates only when
their concrete parameter vectors are otherwise equivalent. Template declared
constraint strictness is not a tie-breaker after expansion; unresolved
equivalent template-derived roots remain ambiguous. This records the task-37
Phase B overload-selection decision only. Mizar-core task 26 / template-audit
F7 now records the separate Phase A omitted-template-argument inference
determinism rule.

Template expansion diagnostics preserve the skipped template candidate and the
reason. A failed template expansion does not become an ordinary candidate.
When a `such that` template constraint lacks visible accepted evidence, the
candidate is rejected or deferred with a constraint-evidence reason; the checker
must not create a new proof obligation or assume the constraint during overload
resolution. If parser/resolver payloads do not expose the constraint or label
mapping, the candidate is deferred under MC-G027 instead of being expanded.

### Task 23 Template Expansion Data Layer

`TemplateExpansionOutput::expand` consumes an `OverloadCollectionOutput` and
validates only the explicit `TemplateCandidatePayload` metadata retained by
task 22. Successful template candidates are copied into a concrete
`OverloadCandidateTable` with the template payload removed and
`CandidateOrigin::TemplateDerived` retained. Non-template candidates are copied
unchanged. Rejected or deferred template candidates are omitted from the
concrete candidate table and preserved in `TemplateExpansionTable` with
diagnostics, source candidate ids, successful substitution lists, and stable
failure reasons. Task 23 does not rewrite normalized signature ids; the
candidate's normalized parameter/result metadata is the caller-provided
concrete signature, and the expansion table records the template argument
mapping that justified exposing it as concrete.

Task 23 deliberately does not infer omitted parameters from fresh site/fact
state, run cluster expansion, check viability, compare specificity, select
ordinary roots, insert views, or synthesize unsupported parser/resolver
template roles.

## Viability Over Recorded Facts

A concrete candidate is viable when every argument can be viewed as the
corresponding parameter by proof-free widening already recorded before phase 8:

- identical normalized type;
- unique structure or mode inheritance upcast;
- attribute weakening justified by existing `TypeFactId`s and phase-7 closure;
- source-written `qua` validated as widening;
- explicit template argument viability after successful template expansion.

The viability checker may read `TypeFactTable`, `CoercionTable`, and
`ResolutionTrace`. It must not run new type inference, derive new cluster
facts, activate registrations, create obligations, or accept proofs. Missing
facts reject or block the candidate with a stable explanation.

Only consumable evidence may make a candidate viable:

- known facts are usable when they are not degraded, rejected, contradicted, or
  obligation-pending;
- assumed facts are usable only through a local context that makes the
  assumption visible at the overload site;
- coercion records are usable only when they describe available widening or
  source-written `qua` views, not narrowing, missing evidence, or blocked
  obligations;
- trace-derived facts are usable only when the trace step is accepted and
  visible to the current typed site.

Pending obligations, degraded recovery facts, rejected facts, out-of-scope
assumptions, or unaccepted registration evidence cannot make a candidate viable.

```rust
enum CandidateViabilityStatus {
    Viable { views: Vec<ArgumentViewPlan> },
    Rejected { reasons: Vec<CandidateRejection> },
    Blocked { reason: CandidateBlockedReason },
}
```

Ambiguous inheritance paths block the affected candidate unless source `qua`
selects a path. Invalid narrowing is rejected; narrowing belongs to
`reconsider` and task-10 obligation handling, not overload resolution.

### Task 24 Viability Data Layer

`CandidateViabilityOutput::filter` consumes `TemplateExpansionOutput` plus
explicit checker-owned viability payloads keyed by concrete candidate id. The
payload for each argument states the already-recorded evidence kind and status:
exact type match, consumable fact widening, accepted widening/source-`qua`
coercion, rejected narrowing, missing evidence, pending/degraded/rejected fact
state, out-of-scope assumption, ambiguous inheritance, or an external
dependency deferral.

Task 24 validates those payloads without deriving facts from `TypeFactTable`,
running cluster closure, activating registrations, discharging obligations,
walking `TypedAst`, or inserting views. A candidate is emitted to the viable
candidate table only when all arguments have accepted proof-free evidence.
Rejected or blocked candidates are omitted from that table and retained in a
viability decision table with stable per-argument reasons and diagnostics.
The viability decision table contains one row for every concrete source
candidate, including accepted candidates, so accepted view plans and
non-viable reasons render from the same stable table. Existing candidate
diagnostics are remapped only for candidates that remain in the viable
candidate table. Duplicate viability payloads for the same concrete candidate
block that candidate with a stable reason, and payloads keyed to unknown
candidate ids are diagnosed instead of being silently consumed.

## Specificity Preorder

Specificity is represented as a graph per overload site, not as a global DAG.
Nodes are viable concrete candidates grouped by ordinary root. An edge
`A -> B` means root `A` is at least as specific as root `B` for every argument
position.

Comparison uses normalized parameter types and recorded closure facts:

- radix or structure compatibility follows the known mode/structure hierarchy;
- attribute subsumption uses already recorded closure facts;
- all argument positions must be at least as specific;
- strictness exists only when the reverse edge does not exist;
- mutual specificity means the concrete normalized parameter vectors are
  closure-equivalent, not that the roots are interchangeable;
- incomparable pairs remain incomparable.

Allowed tie-breakers are limited to those in architecture 05 and spec 19:

- non-template beats template-derived when parameter vectors are equivalent;
- a source-local declaration may shadow an imported declaration only when
  signature collection has already accepted that shadowing;
- template declared constraint strictness is not a tie-breaker after expansion;
- return type is never a tie-breaker;
- same-root redefinitions are not tie-breakers.

Specificity graph rendering is deterministic. Edges carry reason keys so
diagnostics and `@show_resolution` can explain why comparisons succeeded,
failed, or were blocked.

### Task 25 Specificity Graph Data Layer

`SpecificityGraphOutput::build` consumes `CandidateViabilityOutput` plus
explicit checker-owned pairwise comparison payloads keyed by viable candidate
ids. Each payload records whether the left candidate is at least as specific,
the right candidate is at least as specific, both are equivalent, the pair is
incomparable, or the comparison is blocked/deferred. Task 25 records one graph
per overload site, one node per viable concrete candidate, comparison rows for
observed or missing same-site pairs, and directed edges only for accepted
at-least-as-specific relationships.

Task 25 does not derive closure facts, inspect return types, select roots,
apply tie-breakers, join refinements, or insert views. Missing comparison
payloads, duplicate pair payloads, cross-site pair payloads, and unknown
candidate ids are diagnosed with stable reasons instead of being silently
consumed. Incomparable pairs remain explicit comparison rows and do not create
edges.

## Root Selection And Refinement Joins

The selected ordinary root is represented by a unique maximal
non-redefinition candidate in the per-site specificity graph after allowed
tie-breakers. If no root is viable, the site is `NoMatch`. If several distinct
ordinary roots remain maximal, whether unrelated/incomparable or equivalent by
mutual specificity, the site is `Ambiguous`.
At the checker data-layer boundary, "after allowed tie-breakers" means the
pairwise comparison payloads have already encoded only spec-allowed
non-template/template or accepted local-shadow decisions as graph edges or
equivalence. Root selection must not apply new tie-breakers, must not inspect
return types to order roots, and must leave an unresolved tie as `Ambiguous`.
Redefinition candidates never serve as the selected root candidate; if a graph
leaves only redefinitions maximal for an ordinary root, the site is blocked as
a malformed/missing ordinary-root payload and the redefinitions remain
refinement-only metadata. If multiple same-root non-redefinition candidates
remain maximal, for example because a non-template/template tie-breaker was not
encoded as an edge, the site is blocked as an ambiguous ordinary-root payload.

After selecting a root, the checker gathers accepted viable redefinitions whose
`ordinary_root` equals the selected root and whose coherence metadata is
`Accepted`:

- no active refinement exposes the root metadata;
- one active refinement exposes that refinement metadata;
- several active refinements are joined when result facts are compatible;
- incompatible joined facts produce `IncompatibleRefinementJoin`.

Refinement joins may use return/result metadata, because the ordinary root has
already been selected. They must not use result metadata to select among
unrelated ordinary roots. Pending or failed coherence evidence keeps the
redefinition diagnostic-visible but unavailable to the join.

Join compatibility follows spec 19:

- for functors, if one active result type is at least as specific as every
  other active result type, that strongest type is exposed;
- otherwise, functor result types with the same radix may expose the union of
  mutually consistent attributes, closed under already recorded active cluster
  facts;
- incompatible result radices, contradictory attributes, or no unique joined
  type produce `IncompatibleRefinementJoin`;
- predicate and attribute refinements join logical bookkeeping only; coherence
  prevents truth-value disagreement, but incompatible structure/attribute
  metadata is still diagnosed rather than ordered by source position.

## `qua` View Insertion

Source-written `qua` views are preserved. The checker may also record inserted
views when the selected candidate requires a proof-free widening that later
phases must observe explicitly:

- unique inheritance path upcasts;
- attribute weakening justified by existing `TypeFactId`s;
- explicit template-argument widening after template expansion.

Inserted views are semantic metadata, not source edits. They record the source
argument, target normalized type, reason, evidence facts or inheritance path,
selected candidate, and whether the view was source-written or inserted.

Inserted views are forbidden for narrowing, missing facts, ambiguous paths, or
post-hoc overload disambiguation after ambiguity was detected. If multiple
inheritance paths exist and no source `qua` selects one, the site or candidate
remains blocked.

### Task 26 Selection And View Data Layer

`OverloadSelectionOutput::resolve` consumes `SpecificityGraphOutput` plus
explicit checker-owned refinement-join and inserted-view payloads keyed by
overload site. It computes candidate maximality from the per-site specificity
graph without applying additional tie-breakers, selects the unique maximal
non-redefinition ordinary root candidate when one exists, preserves `NoMatch`
and `Ambiguous` failed-site records when selection cannot succeed, and
validates that caller-supplied refinement/view payloads refer only to selected-
root redefinitions with accepted coherence and proof-free widening evidence.

Task 26 may use result metadata only after the ordinary root has been selected:
strongest result type and same-radix attribute-union payloads are accepted when
the caller supplies compatible evidence, while incompatible joins become
`IncompatibleRefinementJoin` failed-site records. Inserted views are recorded
only for accepted widening/source-`qua` evidence. Narrowing, ambiguous paths,
missing facts, missing selection payloads, or payloads for non-selected roots
produce stable failed records and diagnostics instead of fabricated success.

## Failed-Site Preservation

Recoverable failure produces explicit failed overload-site records:

```rust
enum OverloadResult {
    Resolved {
        site: OverloadSiteId,
        root: CandidateId,
        refinements: Vec<CandidateId>,
        exposed_result: Option<ExposedResultPayload>,
        inserted_views: Vec<InsertedViewId>,
    },
    NoMatch { site: OverloadSiteId, rejected: Vec<CandidateId> },
    Ambiguous { site: OverloadSiteId, candidates: Vec<CandidateId>, graph: SpecificityGraphId },
    IncompatibleRefinementJoin {
        site: OverloadSiteId,
        root: CandidateId,
        refinements: Vec<CandidateId>,
        reason: RefinementJoinFailure,
    },
    Blocked { site: OverloadSiteId, reason: OverloadBlockedReason },
}
```

Failed sites keep candidate lists, rejection reasons, source spans, and graph
ids where available. They must never be rewritten into fabricated successful
definitions. Elaboration and proof phases skip or degrade failed sites rather
than lowering them into core logic.

## Diagnostics And Determinism

Diagnostics are checker-local until public diagnostic code allocation exists.
They must include stable detail keys for:

- no viable candidate;
- overload ambiguity;
- template inference failure or ambiguity;
- ambiguous inheritance path;
- invalid or narrowing `qua`;
- incompatible refinement join;
- pre-existing declaration conflict;
- missing checker-owned payloads.

Every diagnostic preserves primary source range, candidate order, declaration
and import provenance, expected/actual type summaries, comparison reasons, and
suggested `qua` targets when a unique widening would disambiguate the site.

Deterministic rendering requirements:

1. Sites sort by source id, range, owner, and kind.
2. Candidates sort within each site by declaration kind, ordinary root,
   symbol id, template key, provenance, declaration source order, and the final
   duplicate tie-break.
3. Template exclusions, viability rejections, specificity edges, active
   refinements, inserted views, and diagnostics all render in canonical order.
4. Equivalent inputs produce byte-identical debug rendering.

## Public Enum Policy

Task 31 applies the frontend task-25 public-enum decision procedure to this
module. All public checker-owned enums in `overload_resolution` are
forward-compatible API surfaces and must remain `#[non_exhaustive]`;
downstream consumers must keep wildcard or fallback arms. Checker-internal
matches may remain exhaustive over the currently represented variants when
implementing the specified behavior.

| enum | decision |
|---|---|
| `OverloadSiteKind` | Forward-compatible; overload site roles may grow as source extraction expands. |
| `UnsupportedOverloadRole` | Forward-compatible; unsupported role categories may grow with parser/checker surfaces. |
| `OverloadSiteRecovery` | Forward-compatible; site recovery states may grow with source recovery integration. |
| `CandidateDeclarationKind` | Forward-compatible; candidate declaration families may grow with Mizar declarations. |
| `CandidateOrigin` | Forward-compatible; candidate origins may grow with summaries, artifacts, and recovery sources. |
| `CoherenceStatus` | Forward-compatible; coherence states may grow when proof/artifact status is connected. |
| `TemplateArgument` | Forward-compatible; template argument forms may grow with parser/template semantics. |
| `TemplateQuaStatus` | Forward-compatible; template `qua` states may grow with view evidence policy. |
| `TemplateConstraintEvidenceStatus` | Forward-compatible; constraint evidence states may grow with proof/artifact inputs. |
| `CandidateScope` | Forward-compatible; candidate scopes may grow with dependency and summary sources. |
| `OverloadSiteStatus` | Forward-compatible; site states may grow with recovery and deferred inputs. |
| `OverloadCandidateStatus` | Forward-compatible; candidate states may grow across collection and filtering. |
| `TemplateSubstitutionSource` | Forward-compatible; substitution sources may grow with inference payloads. |
| `TemplateExpansionStatus` | Forward-compatible; expansion outcomes may grow with template semantics. |
| `TemplateExpansionFailure` | Forward-compatible; expansion failures may grow with new template checks. |
| `ArgumentViabilityEvidence` | Forward-compatible; viability evidence may grow with facts, views, and proofs. |
| `ViabilityFactStatus` | Forward-compatible; fact evidence states may grow with fact-query policy. |
| `ViabilityCoercionKind` | Forward-compatible; viability coercion categories may grow with view insertion. |
| `ViabilityCoercionStatus` | Forward-compatible; viability coercion states may grow with evidence handling. |
| `CandidateViabilityStatus` | Forward-compatible; candidate viability states may grow with rejection/blocking policy. |
| `ArgumentViewKind` | Forward-compatible; argument view kinds may grow with additional coercion forms. |
| `CandidateRejectionReason` | Forward-compatible; rejection reasons may grow with semantic checks. |
| `CandidateBlockedReasonKind` | Forward-compatible; blocked reasons may grow with external dependencies. |
| `SpecificityComparisonStatus` | Forward-compatible; comparison input statuses may grow with specificity evidence. |
| `SpecificityBlockedReasonKind` | Forward-compatible; blocked comparison reasons may grow with payload validation. |
| `SpecificityComparisonOutcome` | Forward-compatible; comparison outcomes may grow with ordering policy. |
| `SpecificityFailureReason` | Forward-compatible; specificity failure reasons may grow with graph validation. |
| `RefinementJoinStatus` | Forward-compatible; refinement join states may grow with result-type policy. |
| `RefinementJoinFailure` | Forward-compatible; refinement join failures may grow with compatibility checks. |
| `ExposedResultSource` | Forward-compatible; exposed-result sources may grow with selection policy. |
| `InsertedViewKind` | Forward-compatible; inserted view kinds may grow with accepted coercion forms. |
| `InsertedViewStatus` | Forward-compatible; inserted view states may grow with validation policy. |
| `OverloadResultStatus` | Forward-compatible; overload result states may grow with failed-site handling. |
| `OverloadBlockedReason` | Forward-compatible; blocked overload reasons may grow with selection validation. |
| `OverloadDiagnosticProvenance` | Forward-compatible; diagnostic provenance may grow with additional payload stages. |
| `OverloadDiagnosticClass` | Forward-compatible; diagnostic classes may grow before public checker diagnostic codes are allocated. |
| `OverloadDiagnosticSeverity` | Forward-compatible; diagnostic severity policy may grow with IDE/artifact consumers. |
| `OverloadDiagnosticRecovery` | Forward-compatible; diagnostic recovery states may grow with partial overload policy. |

No exhaustive public enum exceptions are owned by this module.

## Deferred And External Gaps

Task 26 deliberately keeps the following deferred:

- AST-wide source-to-checker extraction of overload sites, candidates,
  template payloads, source `qua` paths, viability evidence, specificity
  comparison evidence, selection/refinement payloads, inserted-view evidence, and
  scheme/theorem roles;
- parser/resolver exposure for unsupported template and scheme roles noted by
  MC-G006;
- public diagnostic-code allocation;
- artifact emission/reuse and stable `ResolvedTypedAst` schema projection;
- active `.miz` overload semantic fixtures.

These gaps must be recorded as `test_gap`, `external_dependency_gap`, or
`deferred` by implementation tasks. They are not permission to scan raw syntax,
parse opaque resolver shells, fabricate candidates, or treat failed sites as
successful.

## Planned Tests

Task 22 candidate-site collection:

- one fixture per supported site kind;
- provenance and source range retention;
- already-filtered scope/visibility inputs are preserved without global scans;
- deterministic candidate order;
- duplicate site keys, missing candidate site links, and deferred unsupported
  roles.

Task 23 template expansion:

- explicit and omitted template argument cases;
- no cluster expansion during inference;
- constrained template evidence accepted, missing, and deferred cases;
- source-`qua` accepted widening and rejected narrowing cases;
- missing omitted-inference payload cases;
- non-template priority over equivalent template-derived candidates;
- unsupported template/scheme roles produce deferred exclusions;
- deterministic template expansion rendering.

Task 24 viability:

- exact match, unique inheritance upcast, attribute weakening, source `qua`;
- known/context-visible/consumable evidence versus pending, degraded,
  rejected, and out-of-scope evidence;
- missing facts, invalid narrowing, ambiguous inheritance path;
- missing viability payloads and deferred external dependencies;
- rejection reasons, blocked reasons, and view plans are stable;
- deterministic viability rendering.

Task 25 specificity:

- comparable, equivalent, and incomparable candidate pairs;
- blocked and deferred comparison rows without edges;
- per-site graph rendering;
- missing, duplicate, unknown, and cross-site comparison payload diagnostics;
- no root selection, tie-breaker application, or return-type-based ordering.

Task 26 selection, refinement, and views:

- unique root selection, no-match, ambiguity, blocked path;
- missing, duplicate, and unknown selection payload diagnostics;
- missing or ambiguous ordinary-root candidate when the maximal set has zero
  or multiple non-redefinition roots;
- blocked specificity graph preservation;
- no additional root-selection tie-breaker when graph maximal roots remain
  unrelated;
- active accepted same-root refinements;
- rejected/pending coherence redefinitions rejected as inactive refinements;
- strongest result type, same-radix attribute union, and incompatible
  refinement joins;
- inserted widening views;
- rejected narrowing or missing-evidence inserted views;
- non-selected-root refinement/view payload diagnostics;
- deterministic rendering for equivalent selection/refinement/view payload
  orderings;
- failed sites remain explicit and cannot seed successful output.
