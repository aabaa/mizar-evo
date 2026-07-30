# Module: labels

> Canonical language: English. Japanese companion: [../ja/labels.md](../ja/labels.md).

Status: task R-017 specified the resolver-owned label-resolution contract, and
task R-018 implemented the executable theorem/lemma and proof-step projection
resolver in `src/labels.rs`. The implementation covers the dedicated label
scope family, proof-block nesting keys, forward-reference rejection, simple /
qualified citation candidates, lowered grouped-item candidates, `LabelIndex`
population, `LabelRefTable` outcomes, and crate-local/internal conflict
diagnostics. R-023 added declaration-symbol corpus collection only; it did not
add production `SurfaceAst` proof-label declaration/reference lowering. The
bounded normal-source proof-step/simple-unqualified collector is planned as
R-032 before Checker Task 258B5C active confinement coverage. Grouped
shared-prefix container diagnostics and definition/registration label
extraction remain outside R-032.

## References

This design derives the resolver-owned label contract from:

- architecture 03 "Label Resolution Is Scoped Separately from Item Resolution";
- spec chapter 15 statement labels, proof organization, justification forms,
  and scoping rules;
- spec chapter 16 theorem labels, proof-block visibility, and citation forms;
- spec chapter 22 diagnostic payload requirements and the current resolver-code
  `spec_gap`;
- architecture 22 `ObligationAnchor` provenance requirements;
- resolver-local `resolved_ast.md`, `env.md`, `imports.md`, `names.md`, and
  `declarations.md`.

## Purpose

The labels phase resolves label declarations and citation use sites after
imports, declaration shells, and namespace lookup are available, but before
proof checking, type checking, ATP dispatch, template instantiation, or
obligation generation. It consumes source-shaped syntax and resolver-owned
indexes, then records explicit label outcomes in `ResolvedAst` and visible
label projections in `SymbolEnv`.

Inputs:

- `SurfaceAst` for the current module;
- resolved imports and namespace lookup behavior from `imports.md` and
  `names.md`;
- declaration shells from `declarations.md`;
- module and dependency label projections from source-backed fixtures or
  summaries when available;
- syntax recovery markers and source ranges owned by `mizar-syntax`.

Outputs:

- label declaration records for represented theorem, definition, proof-step,
  and registration labels;
- `LabelIndex` entries and visible label projections;
- `LabelRefTable` entries for resolver-attempted citation use sites;
- explicit unresolved and ambiguous label records;
- crate-local/internal label diagnostic records with deterministic ordering.

## Boundary

The labels phase may:

- classify label declarations by label scope family and source role;
- resolve simple, qualified, and grouped citation labels;
- decide label visibility, duplicate-label conflicts, and forward-reference
  failures;
- preserve normalized provenance for downstream `ObligationAnchor` label
  hints and dependency slices.

It must not:

- prove a theorem, proof step, definition correctness condition, or
  registration condition;
- generate `ObligationAnchor` values or verification conditions;
- run ATP, select premises, or expand template arguments semantically;
- type-check definition bodies, registrations, or proof statements;
- choose an overload winner for ordinary names;
- invent public user-facing resolver diagnostic codes.

## Label Scope Families

Labels are not ordinary symbols. A label declaration belongs to one
resolver-owned family:

| Family | Sources | Visibility surface | Downstream consumers |
|---|---|---|---|
| theorem / lemma result | `theorem` and `lemma` items | current module after declaration; exported table when public | citations, artifacts, ATP premise selection |
| definition | definition and redefinition labels | defining item and source correctness-role provenance | checker, VC generation, diagnostics |
| proof step | labeled propositions, assumptions, conclusions, cases, `now` blocks, iterative equality chains | enclosing reasoning block and nested child blocks after declaration | proof justification and local context |
| registration | registration and reduction labels | registration item and registration trace | checker, kernel replay, diagnostics |

The expected label family comes from the use-site syntax. A `by` citation may
refer to a local proof-step label or a module theorem/lemma label. Definition
and registration label references are resolved only in syntax positions that
expect those families, such as correctness or registration trace sites. If a
use site can legally accept multiple families and more than one visible
candidate remains, the resolver records deterministic ambiguity instead of
choosing one by source order.

## Proof-Block Scope

Proof labels are scoped to reasoning blocks, not to the ordinary symbol
namespace.

- A label attached to a statement becomes visible only after the statement is
  complete.
- A label attached to `now ... end` belongs to the enclosing block and becomes
  visible only after that block closes.
- Labels declared inside a nested proof, case, suppose, or diffuse reasoning
  block are visible to that block and nested child blocks, but not to the
  enclosing block after the child block closes.
- Enclosing proof labels are visible inside nested child blocks unless the
  nested construct starts a separate module-level item.
- Inner-scope label shadowing is forbidden by spec chapter 15. A new label that
  repeats any label visible from the current label scope is a duplicate or
  conflict, not a shadowing declaration.
- Same-scope duplicate labels are duplicate-label conflicts.

The resolver must keep resolving the rest of the module after a duplicate or
conflicting label. It records the conflict as crate-local/internal diagnostic
data and keeps enough candidate provenance for later diagnostics and editor
navigation.

## Declaration Point And Forward References

Label lookup is declaration-point sensitive.

- A label is visible only after its declaring statement, item, or block is
  complete.
- A citation to a later label in the same proof block is unresolved.
- A theorem or lemma label is visible to later module items only after the
  theorem or lemma item is complete. A citation to a later theorem or lemma in
  the same module is unresolved.
- Definition and registration labels are visible at resolver-visible
  correctness-role and trace-provenance positions according to the enclosing
  item structure, but not before their declaring syntax has been collected.
- A self-reference from a label's own declaration body is unresolved unless a
  later proof/checker phase defines a separate recursive rule. R-017 defines no
  such rule.

Forward-reference failures are represented as explicit
`UnresolvedLabelRef`-style outcomes with the attempted spelling, use-site
range, and expected label family. They do not fabricate a label origin path.

## Citation Lookup

Simple unqualified citation lookup is label-family specific:

1. visible proof-step labels in the current proof block chain;
2. current-module theorem/lemma labels visible at the use site;
3. imported public theorem/lemma labels made visible through resolved imports
   and exports.

Because inner proof-label shadowing is forbidden, more than one proof-step
candidate for the same spelling is a conflict record. If an unqualified
citation still has multiple legal candidates after family and visibility
filtering, the resolver records `AmbiguousLabelRef` with candidates sorted by
normalized origin path, kind, and source range.

Qualified citations split namespace and label lookup:

1. Resolve the module prefix through the namespace rules in `names.md`.
2. Resolve the final label spelling in the target module's exported label
   table.

Citation prefixes are namespace paths only. The R-016 dot-chain finalization
rules for local-term shadowing, selectors, and `DeferredSelector` records do
not apply to simple, qualified, grouped, or bulk citation prefixes.

Grouped citations use the same resolved module prefix for each grouped label
and produce one label-resolution outcome per concrete grouped item. R-018
accepts the lowered per-item candidates after the shared prefix has already
been resolved or failed. Full `SurfaceAst` lowering records a shared-prefix
failure once and then attaches dependent unresolved label outcomes to each
grouped item. R-023 did not implement that container walk, and R-032 is limited
to simple unqualified proof-label citations, so grouped shared-prefix lowering
remains a later separately authorized task.

Bulk citations (`module_path.*`) are not permission to fabricate individual
label entries. If the target module's exported theorem/lemma label table is
available, the resolver may expand the bulk citation into the deterministic
public theorem/lemma label set required by spec chapter 16. If that table is
not available, the resolver records an unresolved module-label-set dependency
for the citation container; it does not invent synthetic `LabelRef` entries.

Template arguments attached to citations are carried as use-site provenance for
later template/proof phases. R-017 and R-018 do not validate, instantiate, or
type-check those arguments.

## Label Origin Paths

`LabelOriginPath` is the resolver-owned stable identity used in `LabelRef`,
`LabelIndex`, dependency edges, and later `ObligationAnchor` label hints. It is
not proof evidence and must not replace proof/checker-owned identities.

A canonical label-origin serialization contains enough structure to be stable
under formatting and unrelated local edits. "Canonical" applies to framing and
field order; identifier spellings remain exact parser token bytes and are not
case-folded or Unicode-normalized:

- canonical `ModuleId` or module path;
- label family and primary spelling;
- defining item kind and source contribution;
- source-shaped structural path to the declaring statement, proof block,
  definition clause, or registration clause;
- for proof labels, the enclosing theorem or proof owner plus proof-block and
  local statement path;
- for definition and registration labels, the source correctness-role or trace
  provenance when available without checker-owned semantics.

Source ranges and `SurfaceNodeId`s remain provenance for diagnostics and editor
navigation. They are not canonical label identity by themselves.

## Recovery And Diagnostics

Recovered or malformed label syntax is retained as unresolved or recovered
label records when the surrounding source shape is still represented. The
resolver must not panic on recovered proof or declaration subtrees.
Recovered label projections remain available as degraded label-index facts, but
they are excluded from label-reference candidate sets and from
duplicate/conflicting-label diagnostics so parser recovery does not cascade
into semantic ambiguity or conflict reports.

Diagnostic records remain crate-local/internal while R-G001 is open. Label
diagnostics must preserve:

- primary use-site or declaration range;
- duplicate/conflicting declaration ranges;
- expected label family;
- failed namespace or unresolved import dependency for qualified citations;
- deterministic candidate lists for ambiguity.

No public numeric resolver diagnostic code is assigned by this module spec.

## Determinism

Label collection and resolution are deterministic:

- declaration traversal follows stable source order;
- table ids are insertion-order ids from deterministic traversal;
- candidate lists are sorted by `LabelOriginPath`, label kind, and source
  range;
- diagnostics are sorted by primary source range, diagnostic class, and stable
  origin path;
- debug rendering uses normalized origin paths and never raw hash-map order.

## Public Enum Forward-Compatibility

Task R-026 applies the frontend task-25 public-enum decision procedure to this
module. All public resolver-owned enums in `labels` are forward-compatible API
surfaces and must remain `#[non_exhaustive]`:

- `LabelProjectionSource`
- `LabelReferenceScope`
- `LabelDiagnosticKind`
- planned `ProofLabelSourceCollectionError`

No exhaustive public enum exceptions are owned by this module. Downstream
consumers must keep wildcard or fallback arms; resolver-internal matches may
remain exhaustive over the currently represented variants when implementing the
specified behavior.

## Test Obligations

R-017 added no executable tests because it was documentation-only. R-018 adds
unit tests for:

- proof-block visibility and nested-block confinement;
- duplicate/conflicting labels across visible scopes, including the
  spec-forbidden inner-scope shadowing case;
- rejection of forward references to later labels;
- simple, qualified, and lowered grouped-item citation lookup where the parser
  already produces the relevant syntax;
- deterministic `LabelRefTable`, `LabelIndex`, and diagnostic ordering.

R-023 introduced active declaration-symbol corpus coverage, but not
label-reference corpus coverage or production proof-label source projection.
The remaining active label-reference cases are an R-G007 `test_gap`. R-032 is
the separate lower prerequisite for the first bounded Checker Task 258B5C
inner-to-outer and sibling confinement increment.

## R-032B Frozen Normal-Source Projection Contract

### Authority And Finding

Canonical Chapter 15 §15.10 scopes statement labels to the enclosing reasoning
block and forbids same-scope duplicates and inner-scope shadowing. Canonical
Chapter 16 §16.4.2 makes proof labels local to their proof block, and §16.5.1
allows a local label to cite an earlier proposition in the same proof.

The existing `LabelResolver` implements the correct prefix rule over explicit
inputs: declaration scope `D` is visible from reference scope `R` exactly when
`D` is a prefix of `R`, subject to the existing completion-boundary check.
The missing production `SurfaceAst` projection/reference path is Medium
`source_drift`; assigning that path to R-023 was `design_drift`. The former
bare mapping callback would have crossed the validated structural-lowering
boundary and was a `boundary_violation`; R-032A repairs that prerequisite
first. Missing active Checker Task 258B5C cases remain the R-G007 `test_gap`.
The absent public resolver code is the existing Low deferred R-G001
`spec_gap`. No other disagreement is frozen here.

### Exact Lowering Contract

R-032B adds one resolver-owned collector for represented, normal,
unrecovered source. A candidate or traversal is accepted only when every
required node and edge is direct, normal/unrecovered, and exact-shaped. The
default for every unlisted node kind or edge is skip: no row, no ordinal, and
no descent. Semantic descendants are never collector inputs.

The exhaustive default-deny Surface edge table is:

| Parent | Allowed direct child or inspection | Effect | All-other action |
|---|---|---|---|
| `Root` | Exactly one direct normal `CompilationUnit` structural child | Descend into that compilation unit; skip direct token children | Any other, additional, or missing structural child makes the root unsupported |
| `CompilationUnit` | Exactly one direct normal `ItemList` structural child | Descend into that item list | Any other direct child, or any additional/missing structural child, makes the compilation unit unsupported |
| `ItemList` | Direct normal `TheoremItem` children only | Scan supported theorem owners in source order | Skip and do not descend into every other item child, including `LemmaItem`, `VisibleItem`, `StatementItem`, definitions, annotations, and recovered items |
| `TheoremItem` | Inspect direct role, theorem-label, and colon tokens; require an exact normal label and exactly one direct `ProofBlock` justification | Allocate the theorem owner/root scope and descend only into that `ProofBlock` | Formula, every other token/wrapper, additional/missing proof, and every other child are not descended; a failed required shape makes the theorem owner unsupported |
| `ProofBlock` | Validate direct `proof`/`end` boundary with no recovered or malformed direct child; among ordered direct children accept only `CompactStatement` and `ConclusionStatement` | Each accepted statement consumes its module-global ordinal and is visited in direct-child order | Every other statement, wrapper, and token kind gets no descent and no ordinal; a malformed/recovered boundary makes this proof owner unsupported |
| `CompactStatement` | Direct `Proposition` only, solely to inspect its exact first identifier token followed by colon | Emit one proof-step projection only when that exact label shape exists | Never descend into `FormulaExpression` or its tokens; any other proposition child/shape emits no projection |
| `CompactStatement` or `ConclusionStatement` | Direct `ProofBlock`; direct `JustificationClause` | A proof block creates the next nested child scope and is descended; a justification permits the citation walk | Proposition/formula/token and every other child are not descended. `ConclusionStatement` proposition labels are excluded |
| `JustificationClause` | Direct `ReferenceList`, only when the exact first token is `by` | Descend into that reference list | Computation and every other child/shape get no descent |
| `ReferenceList` | Direct `Reference` children in source order | Visit each exact simple-reference sibling; comma tokens are skipped | `QualifiedReference`, `GroupedReference`, `BulkReference`, recovered nodes, and every other child get no descent and no row |
| `Reference` | Exactly one direct identifier token and no `TemplateArguments` or other child | Emit one `LabelReferenceCandidate` | Any additional, missing, template, qualified, malformed, or recovered child shape emits no row |

Any recovery, missing/error node, malformed boundary, or non-direct edge on a
required shape or owner chain makes that owner/edge unsupported; the collector
does not descend through it. Only successfully supported direct
`CompactStatement` and `ConclusionStatement` rows consume statement ordinals.

The exact public lower seam is:

```rust
pub struct ProofLabelSourceCollector<'a> {
    // Private fields.
}

impl<'a> ProofLabelSourceCollector<'a> {
    pub fn new(
        ast: &'a SurfaceAst,
        module: &ModuleId,
        namespace: NamespacePath,
        contribution: SourceContributionId,
        resolved: &'a SurfaceResolvedArena,
    ) -> Result<Self, ProofLabelSourceCollectionError>;

    pub fn collect(
        &self,
    ) -> Result<ProofLabelSourceCollection, ProofLabelSourceCollectionError>;
}
```

`ProofLabelSourceCollection` exposes
`projections() -> &[LabelProjection]`,
and `references() -> &[LabelReferenceCandidate]`. Construction validates the
exact map with `resolved.validate_against(ast, module)` and fails for a wrong,
stale, incomplete, or fabricated map. The collector obtains every
`ResolvedNodeId` through the validated `SurfaceResolvedArena`; no callback or
unmapped-reference side channel exists. It returns existing resolver inputs
and does not duplicate `LabelResolver` visibility, completion, ambiguity, or
unresolved-outcome logic.

The collector stores only the `ast` and `resolved` borrows under `'a`, owns
`namespace` and `contribution`, and neither borrows nor stores `module`.
`new` uses `module` only to validate the arena. Each `collect` re-runs
`resolved.validate_against(ast, resolved.module())`, using the arena's
validated canonical identity rather than a stored constructor argument.

`ProofLabelSourceCollection` derives `Debug`, `Clone`, `PartialEq`, and `Eq`;
it is not required to be `Copy`. `ProofLabelSourceCollectionError` derives
`Debug`, implements `Display` and `std::error::Error`, and is not required to
be `Clone`, `Eq`, or `Copy`.

The exact public error declaration is:

```rust
#[derive(Debug)]
#[non_exhaustive]
pub enum ProofLabelSourceCollectionError {
    SurfaceArena(SurfaceResolvedArenaError),
    ScopeComponentOverflow { node: SurfaceNodeId },
    StructuralPathComponentOverflow { node: SurfaceNodeId },
}
```

Downstream matches retain a wildcard arm. Both `new` and `collect` return this
error. Scope and structural-path conversion to `u32` is checked; no unwrap,
saturation, truncation, or panic is allowed.

Stable structure and provenance:

- traverse the exact `Root` -> `CompilationUnit` -> `ItemList` upper chain,
  then scan supported direct normal `TheoremItem` / direct `ProofBlock` roots
  in item-list source order; root scopes receive `[0]`, `[1]`, ... and the
  theorem owner consumes no statement ordinal;
- one shared module-global one-based statement counter is never reset per
  theorem. Within each supported root, visit normal supported
  `CompactStatement` and `ConclusionStatement` in depth-first preorder,
  descending only into direct nested `ProofBlock`s owned by those forms and in
  their ordered child order;
- every supported statement consumes one ordinal even when it has no label and
  no citation. A reference uses its owning supported statement's ordinal.
  Every unlisted wrapper/container and every excluded, unsupported, recovered,
  or malformed statement/subtree consumes none and is not descended;
- nested supported proof scopes append immediate proof-child components
  relative to their theorem owner, for example `[0, 0]`, `[0, 1]`, and
  `[0, 0, 0]`;
- `LabelProjection.visible_after_ordinal` is the maximum consumed ordinal in
  the whole labelled `CompactStatement` subtree, including the labelled
  statement itself and its own proof;
- the exact B5C inner-to-outer example consumes ordinals `1..5`: A declares at
  `2`, its own-proof statement is `3`, completion/visible-after is `3`, the
  following same-block statement is `4`, and the negative reference is `5`.
  The sibling example consumes `1..6` and its negative reference uses `6`.
  Multiple-theorem tests preserve the global counter so an earlier theorem
  label is ordinal-eligible from a later theorem and fails only confinement;
- `LabelOriginPath` uses this exact one-line serialized grammar with no spaces
  or newline:

```text
proof-step-v1|package=<n>:<package>|module=<n>:<module-path>|contribution=<u>|owner-kind=theorem|owner=<n>:<owner-label>|owner-occurrence=<u>|proof-path=<k>:<c0>,...|label=<n>:<label>|label-occurrence=<u>
```

  Here `<n>` is the following spelling's UTF-8 byte length. `<u>`, `<k>`, and
  every `<ci>` are canonical unsigned decimal with no leading zero except
  `0`; `<k>` is the number of comma-separated path components and every
  `<ci>` is checked `u32`. An empty relative proof path is exactly
  `proof-path=0:`. Length framing requires no escaping. Package/module
  fields come from canonical `ModuleId`. Owner and label spellings are exact
  parser identifier token text byte-for-byte: identity performs no case fold
  or Unicode transformation. `owner-occurrence` is zero-based among
  earlier/current supported normal top-level theorem owners with the same
  exact spelling. `label-occurrence` is zero-based among earlier/current
  same-spelling supported labelled compact statements in its declaring proof
  scope. `proof-path` is relative to the owner root, whose path is empty. The
  root visibility index is not serialized;
- projection `SemanticOrigin` anchors the exact label token and has structural
  path `[theorem item, compact statement, label token]`; reference origin
  anchors the exact reference and has path
  `[theorem item, owning CompactStatement or ConclusionStatement, reference]`;
- exact B5C paths are projection `[57, 42, 8]`, reference `[57, 55, 52]` for
  inner-to-outer, and projection `[67, 47, 8]`, reference `[67, 63, 60]` for
  sibling confinement;
- those richer table origins are intentionally distinct from, and validated
  in addition to, the R-032A arena node's minimal `[surface_id]` origin;
- output remains deterministic under unrelated formatting; different-spelling
  owner mutations do not perturb unrelated canonical identities.

R-032B excludes lemma/claim/definition/registration owners; top-level theorem
labels; assumption, given, take, set, consider, reconsider, case, suppose, now,
hereby, and iterative-equality forms; all other statement-label forms;
qualified, grouped, bulk, or template citation forms; recovered or malformed
shapes; and all semantic descendants. Those forms emit no collector rows.

### Ownership And Consumers

R-032B production/test ownership is exactly:

- `crates/mizar-resolve/src/labels.rs`;
- `crates/mizar-resolve/src/labels/tests.rs`; and
- synchronized resolver design records.

R-032A owns only `resolved_ast.rs`, `resolved_ast/tests.rs`, and paired design
records in the preceding commit. The later active consumer belongs to the private
`mizar-test` `declaration_symbol` route and uses the exact internal detail key
`declaration_symbol.label.proof_scope_confinement`. The public checker
`SourceStatementReferenceHandoff` is not a consumer because its boundary
rejects unresolved references; it must not be widened to transport this
negative outcome.

### Test And Exit Contract

The R-032B implementation must assert:

- enclosing-to-child success `[0] -> [0, 1]`;
- inner-to-outer unresolved `[0, 0] -> [0]`;
- sibling unresolved `[0, 0] -> [0, 1]`;
- cross-theorem same-spelling labels do not conflict, and a label declared in
  earlier theorem root `[0]` remains unresolved from later theorem root `[1]`
  with ordinals otherwise visibility-eligible;
- deterministic top-level theorem-root allocation;
- a citation from A's own proof is unresolved until the labelled compact
  statement completes, while a same-block citation after completion resolves;
- exact B5C ordinals, completion boundaries, ranges, anchors, structural paths,
  and `LabelOriginPath` uniqueness;
- exact `proof-step-v1` construction and byte equality, including UTF-8 byte
  lengths, empty/nonempty proof paths, zero-based occurrence counters, no
  escaping/normalization, and focused package/module/contribution/owner/
  owner-occurrence/path/label/label-occurrence mutations;
- module-global ordinal continuity across theorem roots, ordinal consumption by
  supported unlabeled/no-citation statements, and non-consumption/no-descent
  for unlisted wrappers or excluded subtrees;
- one positive test for every permitted table edge, including separate
  root-to-compilation-unit, compilation-unit-to-item-list, and
  item-list-to-theorem tests, plus theorem-to-proof, proof-to-each statement,
  compact-to-proposition-label inspection, each statement-to-nested-proof and
  justification, justification-to-reference-list, list-to-reference, and
  reference-to-identifier;
- the root upper-edge positive includes direct token siblings and proves they
  are skipped without disturbing its sole structural `CompilationUnit`;
- missing/additional/wrong upper structural children and relocated/wrapped
  alternatives are rejected: a theorem directly under `Root` or
  `CompilationUnit`, or beneath `VisibleItem`, is never reached;
- negative parent-relocation, wrapper insertion, formula-token, computation,
  qualified, grouped, bulk, template-argument, unsupported-proof-owner, and
  recovered/malformed mutations, each proving no row/ordinal/descent beyond
  the rejected edge;
- a mixed `ReferenceList` that collects only its exact simple `Reference`
  siblings while skipping commas and every unsupported sibling;
- an exhaustive representative default-deny matrix exercising the all-other
  action of every table row;
- exact inclusion of the supported forms and exclusion of every listed
  unsupported, semantic, malformed, or recovered form without panic;
- wrong source/module/arena/node and stale shape/recovery maps are rejected
  through R-032A validation;
- checked scope/path overflow returns `ProofLabelSourceCollectionError`;
- deterministic collection/candidate order plus focused spelling,
  proof-topology, unrelated-formatting, owner-spelling, and owner-order
  mutations.

The documentation prerequisite changes no production source, `.miz` fixture,
expectation sidecar, trace status/count, or public API. R-032A and R-032B must
be later separate commits and must not change parser/frontend production,
Cargo/workspace metadata, checker/type/proof/Core/CFG/VC behavior, public
diagnostic codes, or the active runner. In the historical pre-S-026 record,
the downstream B5C active consumer was the fourth logical task after fresh
inventory; the effective order below supersedes that execution count.

R-032A preflight subsequently inserted the separate mizar-syntax S-026
documentation and implementation prerequisites before R-032A because complete
dense Surface ids cannot otherwise include a valid disconnected node. This
does not change any R-032B label contract or active B5C test intent. The
effective order is S-026 docs, S-026 implementation, R-032A, R-032B, active
B5C, with fresh inventory between commits.
