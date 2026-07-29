# Source Structure-Term Transport

> Canonical language: English. Japanese companion:
> [../ja/source_structure.md](../ja/source_structure.md).

## Scope

Checker Task 254 owns a syntax-free, immutable description of source
structure construction, selector access, and functional update occurrences.
It transports source shape, resolver-authenticated constructor roots, written
members, `FieldUpdate` associations, ordered children, and unresolved
requests only. It does not decide field/property identity, inheritance views,
constructor coverage or defaults, selector or update results, value types,
facts, acceptance, proofs, or downstream IR.

The canonical language requirements are Chapter 5 Sections 5.5 and 5.7 and
Chapter 13 Section 13.3. Task 252 owns primary-term children. Task 253 owns
functor-application children. Task 254 links to their dense root IDs without
copying rows. Task 263 retains structure-definition, member, inheritance-view,
and constructor semantic payloads.

## Public Transaction

`SourceStructureProducer::build` consumes `SourceStructureHandoffInput`,
`SymbolEnv`, `BindingEnv`, `SourcePrimaryTermHandoff`, an optional
`SourceFunctorApplicationHandoff`, and `TypedArena`. The input has seven
source-ordered vectors:

- structure-family terms;
- transparent structure wrappers;
- resolver-authenticated constructor roots;
- written constructor, selector, and update-path members;
- parser `FieldUpdate` association containers;
- ordered child edges;
- unresolved constructor-signature, member-identity, inheritance-path, and
  result-type requests.

The producer publishes seven dense immutable tables only after the entire
transaction validates. Each public ID is a zero-based row index with `new`
and `index`; each table exposes `get`, source-ordered `iter`, `len`, and
`is_empty`. Rows expose read-only validated fields.

Term kinds are `Constructor`, `SelectorAccess`, and `FunctionalUpdate`.
Recovery is `Normal` or `Degraded`. Member roles are
`ConstructorAssignment`, `Selector`, and `UpdatePathSegment`. Edge roles are
`ConstructorValue`, `SelectorBase`, `SelectorArgument`, `UpdateBase`, and
`UpdateValue`. Targets are a Task-252 `Primary`, a Task-253 root
`Application`, or a later Task-254 `Structure` row.

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceStructureTermKind` | `#[non_exhaustive]`; callers must tolerate later structure-family source kinds. |
| `SourceStructureRecovery` | `#[non_exhaustive]`; callers must tolerate later recovery classes. |
| `SourceStructureMemberRole` | `#[non_exhaustive]`; callers must tolerate later written-member roles. |
| `SourceStructureEdgeRole` | `#[non_exhaustive]`; callers must tolerate later child-edge roles. |
| `SourceStructureTarget` | `#[non_exhaustive]`; callers must tolerate later frozen cross-family targets. |
| `SourceStructureRequestKind` | `#[non_exhaustive]`; callers must tolerate later unresolved request kinds. |
| `SourceStructureError` | `#[non_exhaustive]`; callers must not exhaustively match validation failures. |

No exhaustive public enum exceptions are owned by this module.

## Validation And Ownership

The producer authenticates source/module identity, dense source preorder,
context, range, canonical spelling, recovery, exact arena anchors, grouping,
ordinals, and single ownership. Term sites use
`source.term.structure.constructor`, `.selector`, or `.update`. Written
member sites use the exact role-specific keys
`source.term.structure.member.constructor-assignment`,
`source.term.structure.member.selector`, and
`source.term.structure.member.update-path-segment`. Whole update containers
use `source.term.structure.field-update`; transparent wrappers use
`source.term.structure.parenthesized`.

Each constructor has exactly one resolver-authenticated `Structure` root.
Local roots require a normal, conflict-free, source-preceding definition and
exact symbol/definition/contribution cross-index agreement. Imported roots
require public exported or re-exported provenance and an authenticated import
effect. Missing, pending, and opaque signature shells remain unresolved;
malformed or recovered provenance fails closed.

Constructor assignments and update values retain their final member.
Selector and update bases have no member. Update-path segments form
source-ordered parent chains; a `FieldUpdate` owns exactly one nonempty path
and replacement edge but is not a term. Repeated labels and paths remain
distinct ordered rows for Task 263.

Primary children must be same-context Task-252 roots. Application children
must be same-context Task-253 roots, meaning no Task-253 argument edge targets
them; nested Task-253 applications are rejected rather than multiply owned.
Structure children are later same-context Task-254 rows with one incoming
edge. Reverse Task-253 applications containing structure children and all
other frozen subtree exclusions fail closed without detached descendants.

For every structure term, the edge list is exactly the source-ordered set of
direct written children: parentless Task-252 roots not already owned as
Task-253 arguments, Task-253 root applications, and later Task-254 terms.
Candidates contained by another candidate belong to that nearer child and are
not detached into the outer term. No direct child may be omitted, duplicated,
or retargeted. Constructor values occur after their assignment label and
before the next label; selector bases end before the selector member and
arguments begin after it; update bases end before the first `FieldUpdate`;
each replacement is strictly inside its owning `FieldUpdate` and begins after
the final path member. A `FieldUpdate` spelling is exactly the written path
joined by ` . `, followed by ` := ` and the effective replacement spelling.

## Derived Dependency Fingerprints

The output derives `primary_term_fingerprint` from the exact Task-252
`debug_text()`. `application_fingerprint` is `Some` exactly when an
application edge exists and is derived from the exact Task-253 `debug_text()`;
otherwise it is `None`. An unrelated installed Task-253 handoff may coexist
with `None`.

`TypedAst::with_source_structure` is one-shot, requires Task 252 and the
targeted Task 253 dependency first, preserves the producer-validated written
partitions, and revalidates fingerprints, targets, cross-family ownership,
and arena sites. Conversely,
`with_source_application` revalidates an already installed Task-254 handoff,
so a Task-253 argument cannot claim a Task-254 primary target and a Task-253
application cannot contain, partially overlap, or bypass ownership of a
Task-254 term in either installation order. An unrelated Task-253 handoff is
valid with a `None` fingerprint only when its ranges and targets are disjoint
from Task 254. `ResolvedTypedAst` revalidates and clone-preserves the same
association without rebuilding or retargeting dense IDs. Both debug
renderings include the handoff only when present.

When Task 255 is already installed, `with_source_structure` also revalidates
its structure fingerprint, root-only target, and nearest-family range
partition before publishing Task 254. A later structure handoff therefore
cannot contain, overlap, or retarget an installed Task-255 occurrence.

## Private Source Consumer

Raw `SurfaceAst`, source node IDs, and syntax kinds remain in
`mizar-test::runner::type_elaboration::source_structure`. Production selects
only the three functor definientia in
`fail_type_elaboration_local_structure_term_gap_001`.
The leaf consumes the real declaration shells and reuses Task 248's
`SourceBindingContextProducer`; it does not fabricate a generated definition
context.

The exact term/wrapper/root/member/field-update/edge/request oracle is
5/0/3/9/2/10/26. The one shared arena also contains the Task-252
primary/reference/numeric-request slice 8/0/8. The real route has no Task-253
row or fingerprint. After transport validation it retains the Task-263
`type_elaboration.external_dependency.ast_payload_extraction` boundary with
no public diagnostic.

## Verification Boundary

Checker tests cover dense tables, all five arena keys and wrong-key
substitution, member/path and `FieldUpdate` ownership, wrapper nesting,
local/imported root provenance, all request cardinalities, Task-252/253/254
children, Task-253 root-only ownership, the full conditional fingerprint
matrix, corruption, determinism, installation, clone preservation, and atomic
failure. Runner tests cover the exact consumer and oracle, lower-stage shape,
synthetic child families, recovery, exclusions, mutation isolation,
deterministic replay, final ownership, and exclusion of every other active
type-elaboration case.

The bounded trace row is
`spec.en.checker.type_elaboration.source_structure_term_payload`. Task 254
changes MC-G017/MC-G018 executable coverage but leaves semantic
structure/member/view behavior, later term families, accepted facts/proofs,
and Steps 6/7 unimplemented.

## Task 258B3M2B2B2P Frozen Proof-Context Reuse Seam

B2P freezes only the future runner-private reuse of the existing public
Task-254 constructor producer for the exact 172-byte/76-node proof source in
the crate plan. Its owned-kind map is exactly constructor node 59 as
`source.term.structure.constructor` and member nodes 20/24 as
`source.term.structure.member.constructor-assignment`. Qualified root node
52 stays `source.surface.unowned` and participates only in authenticated
resolver-provenance traversal. Task 252 uses nodes 54/57 only as private
extraction roots and publishes numeral rows at nodes 53/56, so 53/56 are
`source.term.numeral` while 54/57 stay `source.surface.unowned`; no other
node becomes Task-254-owned.

The handoff uses existing `BindingContextId(1)` and shared
`SourceTermParts`, preserving Task-48 `2/1/0`, Task-252 `6/4/2`, and the
exact Task-254 `1/0/1/2/0/2/6` constructor/root/member/edge/request profile.
Root provenance is the imported public/exported, signature-free
`parser.type_fixtures::TypeCaseStruct#5`; edges connect members 0/1 only to
primaries 2/3, and the application fingerprint is absent.

The later implementation is confined to the mizar-test source-structure
leaf, publishes no checker or statement/witness API, and keeps the legacy
Task-254 route/debug output byte-compatible. Chapter 5 §5.7 selector
authority is explicitly excluded here and remains future B2B work, not
current constructor semantics. `FieldUpdate` and functional update remain
B2C. The two frozen runner tests exhaust bytes/nodes, ownership/provenance,
corruption precedence, stale/clean replay, legacy output, and empty upper
families; there is no checker test.

## Task 258B3M2B2B2P Private Reuse Result

The runner now implements the frozen exact-source owned-kind selector and
the existing-context/shared-Task-252 call into the unchanged public Task-254
producer. The two exact runner tests pass, including lower-profile,
ownership, resolver-provenance, corruption-precedence, stale-replay, and
legacy-output checks. No checker source or API changed, no selector/update
semantics were introduced, and B2A remains the next consumer.

The exact result preserves Task-48 `2/1/0`, Task-252 `6/4/2`, and Task-254
`1/0/1/2/0/2/6`; owned kinds 59/20/24; numeral sites 53/56; and unowned
52/54/57. It authenticates the imported public/exported, signature-free
`TypeCaseStruct#5` contribution 2 and current-source origin `7..27/[5]`.
The malformed recovery near miss is `diagnostics=1, nodes=74, root=73,
recovered=[52]`. Existing Task-254 source-structure/typed/final debug hashes
remain
`0d6af57b89e6156d8e5de6831568c81ec110880bebf1e4aeb4ab00563f4da6c8`,
`8264d1574faf67e19b6b84d6e11fa7ab6435335238b398fa0966bbfbc63d0599`,
and
`118a998bc5edb770c7818be1d74cbece0f566353bf9d3e6aabb817d994a3db40`.

## Task 258B3M2B2B2A Frozen Structure Consumer

B2A consumes, but does not broaden, the exact completed B2P Task-254
handoff. Structure term 0 remains constructor 59 in proof context 1 with
root 0, members 20/24, edges to `Primary(2/3)`, six unresolved requests,
and no application fingerprint. The new statement witness targets term 0;
it does not target the resolver root, either member, or a field value.

The existing Task-254 public producer, tables, validation, debug bytes, and
legacy route remain unchanged. Field/property identity, coverage/defaults,
value/result typing, inheritance, selector, update, and `FieldUpdate`
semantics remain Task 263/B2B/B2C work.

## Task 258B3M2B2B2A Implemented Structure Consumer

The statement route consumes the completed B2P seam and exact Task-254
handoff without changing its public producer or rows. Structure term 0
remains constructor 59 in proof context 1 with root 0, members 20/24, two
`Primary(2/3)` value edges, six requests, and no application fingerprint.
Only the statement witness adds the directed target to that term.

The B2P seam is now live, so its private dead-code allowance was removed;
visibility, extraction, ownership, validation, and debug bytes are
unchanged; `source_structure.rs` remains 5,036 lines. Selector, update,
`FieldUpdate`, field identity, typing,
inheritance, and all semantic behavior remain deferred.

## Task 258B3M2B2B2BP Frozen Proof-Context Selector Reuse

B2A post-commit inventory found that the generic Task-254 extractor already
models `SelectorAccess`, while the proof-context private seam accepts the
constructor-only profile. B2BP freezes runner-private selector siblings
before any Task-258 B2B consumer: `ImportedStructureSelectorSite`,
`imported_structure_selector_owned_node_kinds`, and
`imported_structure_selector_handoff_in_context`.

For the exact 171-byte source, Task 254 produces
`2/0/1/3/0/3/9`. Selector term 0 at node 62 points by `SelectorBase` to
constructor term 1 at node 61; constructor member values point to
`Primary(2/3)`. Owned nodes are exactly `62/61/29/20/24`. The seam reuses
the existing extractor/producer, binding context 1, shared Task-252 roots,
imported `TypeCaseStruct#5` provenance, and current debug grammar.
Constructor B2P/B2A bytes remain unchanged.

No checker API, Task-256/258 row, TypedAst statement installation, public
runner route, selector identity/type result, or semantic behavior is owned
here. Exactly two future runner tests freeze the private seam before B2B.

## Task 258B3M2B2B2BP Implementation Result

The three frozen production-private siblings are implemented in the runner
source-structure leaf and reuse the existing generic Task-254 extractor and
Task-252 proof-context roots. They publish only the exact
`2/0/1/3/0/3/9` lower table after full source, arena, provenance, ownership,
and fingerprint authentication. The two frozen tests pass and preserve B2P,
B2A, legacy Task-254, and empty upper-family bytes.

No checker source or public API changed. Selector identity, typing, result,
inheritance, proof, goal, and theorem behavior remain outside this transport
seam.

## Task 258B3M2B2B2B Frozen Consumer Boundary

B2BP is now the sole production-private lower seam consumed by B2B. For the
same 171-byte source, Task 254 must remain byte-for-byte
`2/0/1/3/0/3/9`: selector term `0` at node `62` targets constructor
`Structure(1)` by `SelectorBase` and owns selector member identity node
`29`; constructor term `1` at node `61` owns member identities at nodes
`20/24` and value edges to `Primary(2/3)`. Imported root
`TypeCaseStruct#5` retains
contribution `2`, origin `7..27`, and path `[5]`.

B2B may consume this authenticated table only to attach witness node `64` to
`Structure(0)`. Task-254 ownership and bytes do not move to Task 258.
Implementation may consume the seam in runner `source_statement.rs` and
remove only obsolete B2BP `dead_code` allowances from
`source_structure.rs`; it may not change Task-254 extraction, its public
surface, or existing tests. Selector identity/type/result, inheritance,
functional update, `FieldUpdate`, and all semantic behavior remain deferred.

## Task 258B3M2B2B2B Implemented Consumer Result

B2B now consumes only the frozen B2BP private selector owned-kind and
proof-context handoff seams. The authenticated Task-254 table remains
`2/0/1/3/0/3/9`: selector `Structure(0)` at node 62 points to constructor
`Structure(1)` at node 61, with members `29/20/24` and value edges to
`Primary(2/3)`. Task 258 adds only the witness-to-selector edge.

The B2BP extractor, lower rows, provenance, public surface, and existing
tests are unchanged; only obsolete consumer-use `dead_code` allowances were
removed. Checker `source_structure.rs` remains 5,036 lines; the runner
source-structure leaf is 4,506 lines after that cleanup. Selector
identity/type/result, inheritance, update/`FieldUpdate`, proof, goal, and all
semantic behavior remain deferred.

## Task 258B3M2B2B2CP Frozen Proof-Context Update Reuse

Fresh post-B2B inventory finds that the generic Task-254 extractor models
functional updates, while the production-private proof-context reuse surface
has only constructor and selector profiles. B2CP freezes runner-private
`ImportedStructureUpdateSite`, owned-kind, and in-context handoff siblings
before any B2C statement consumer.

For the exact 181-byte/86-node source, Task 254 must publish
`2/0/1/3/1/4/9`. Functional update `Structure(0)` is node 69; constructor
`Structure(1)` is node 65. Members are update path 30 and constructor
assignments 20/24. `FieldUpdate(0)` is node/range `68/153..159`, spelling
`x := 3`, and owns member 0. Edges are update base to `Structure(1)`,
update value/member 0 to `Primary(4)`, and constructor values/members 1/2
to `Primary(2/3)`. Imported root `TypeCaseStruct#5` retains contribution 2,
origin `7..27`, and path `[5]`; no application fingerprint exists.

B2CP owns no Task-258 witness or statement row. It reuses the unchanged
Task-254 public producer and freezes only exact-source private selection,
owned-kind authentication, existing proof context, and shared Task-252
parts. The two tests cover every byte/node, all lower rows and corruptions,
the exact missing-value recovery, replay, exact B2P constructor/B2BP
selector compatibility, and empty upper families. Functional-copy
semantics, member identity,
replacement/result typing, proof/goal/theorem behavior, and B2C witness
ownership remain deferred.

Task 256 later owns only nodes `55/77` and excludes the full update subtree;
containers `56/78` remain unowned. B2C alone may later own take/witness
nodes `72/71` and attach its witness to functional-update `Structure(0)`.
B2CP owns none of these upper rows or edges.
