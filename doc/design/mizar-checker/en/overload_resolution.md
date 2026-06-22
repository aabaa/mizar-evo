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
explicit payloads. Tasks 23-26 implement the remaining named sections below.
Task 28 later assembles the final `ResolvedTypedAst` data shape specified by
`resolved_typed_ast.md`.

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
2. If arguments are omitted, infer template parameters from exact argument
   types and mode-hierarchy constraints only.
3. Check constrained template parameters against explicit constraint evidence
   supplied by the caller and facts visible at the site.
4. Reject inference that is missing, contradictory, ambiguous, unsupported, or
   missing required constraint evidence.
5. Instantiate successful templates into concrete candidates and retain the
   template origin for tie-breaking and diagnostics.

Cluster expansion is not used during omitted-argument template inference.
Source-written template `qua_arg` is accepted only when it is a proof-free
inheritance widening exposed by the typed/coercion payloads. Predicate,
functor, scheme, and algorithm template roles remain deferred unless parser,
resolver, and checker-owned payloads expose the concrete role and parameter
mapping. Non-template candidates beat template-derived candidates only when
their concrete parameter vectors are otherwise equivalent.

Template expansion diagnostics preserve the skipped template candidate and the
reason. A failed template expansion does not become an ordinary candidate.
When a `such that` template constraint lacks visible accepted evidence, the
candidate is blocked or rejected with a constraint-evidence reason; the checker
must not create a new proof obligation or assume the constraint during overload
resolution. If parser/resolver payloads do not expose the constraint or label
mapping, the candidate is deferred under MC-G027 instead of being expanded.

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

## Specificity Partial Order

Specificity is represented as a graph per overload site, not as a global DAG.
Nodes are viable concrete candidates grouped by ordinary root. An edge
`A -> B` means root `A` is at least as specific as root `B` for every argument
position.

Comparison uses normalized parameter types and recorded closure facts:

- radix or structure compatibility follows the known mode/structure hierarchy;
- attribute subsumption uses already recorded closure facts;
- all argument positions must be at least as specific;
- strictness exists only when the reverse edge does not exist;
- incomparable pairs remain incomparable.

Allowed tie-breakers are limited to those in architecture 05 and spec 19:

- non-template beats template-derived when parameter vectors are equivalent;
- a source-local declaration may shadow an imported declaration only when
  signature collection has already accepted that shadowing;
- return type is never a tie-breaker;
- same-root redefinitions are not tie-breakers.

Specificity graph rendering is deterministic. Edges carry reason keys so
diagnostics and `@show_resolution` can explain why comparisons succeeded,
failed, or were blocked.

## Root Selection And Refinement Joins

The selected ordinary root is the unique maximal root in the per-site
specificity graph after allowed tie-breakers. If no root is viable, the site is
`NoMatch`. If several unrelated roots remain maximal, the site is `Ambiguous`.

After selecting a root, the checker gathers accepted viable redefinitions whose
`ordinary_root` equals the selected root:

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

## Deferred And External Gaps

Task 22 deliberately keeps the following deferred:

- Rust implementation for tasks 23-26;
- AST-wide source-to-checker extraction of overload sites, candidates,
  template payloads, source `qua` paths, and scheme/theorem roles;
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
- non-template priority over equivalent template-derived candidates;
- unsupported template/scheme roles produce deferred exclusions.

Task 24 viability:

- exact match, unique inheritance upcast, attribute weakening, source `qua`;
- known/context-visible/consumable evidence versus pending, degraded,
  rejected, and out-of-scope evidence;
- missing facts, invalid narrowing, ambiguous inheritance path;
- rejection reasons and view plans are stable.

Task 25 specificity:

- comparable, equivalent, and incomparable candidate pairs;
- per-site graph rendering;
- allowed tie-breakers only;
- return type never participates in root ordering.

Task 26 selection, refinement, and views:

- unique root selection, no-match, ambiguity, blocked path;
- active accepted same-root refinements;
- strongest result type, same-radix attribute union, and incompatible
  refinement joins;
- inserted widening views;
- failed sites remain explicit and cannot seed successful output.
