# Module: symbols

`validate_source_symbol_env` reuses resolver collection to authenticate the complete local symbol environment.

> Canonical language: English. Japanese companion: [../ja/symbols.md](../ja/symbols.md).

Status: tasks R-019 to R-021 specify and implement the resolver-owned
signature-collection path. R-020 implements collection, duplicate/conflict
detection, registration indexing, and overload grouping over explicit
declaration projections. R-021 adds parser-backed per-kind projection
extraction, parser-owned opaque signature payloads, and a resolver-local module
lexical summary index for exported lexer-visible spellings. Semantic `.miz`
corpus coverage and traceability metadata remain task R-023 work.

## References

This design derives the resolver-owned symbol and signature contract from:

- architecture 03 Step 5, "Collect Signature Environment";
- architecture 03 "Signature Collection Is a Declaration Pass, Not Type
  Checking";
- architecture 01 `SymbolEnv` and `ResolvedAst` boundaries;
- spec chapters 5, 6, 7, 9, and 10 for structures, attributes, modes,
  predicates, and functors;
- spec chapter 11 for synonyms, antonyms, visibility, imports, and conflict
  behavior;
- spec chapter 12 for module interfaces, export visibility, and public
  algorithm signatures;
- spec chapters 16 and 18 for theorem/lemma, scheme, and template declaration
  surfaces;
- spec chapter 17 for registration labels and registration declaration
  surfaces;
- spec chapter 19 for overload candidate construction and checker-owned
  overload winner selection;
- spec chapter 20 for algorithm signatures, contracts, and the signature/body
  visibility split;
- spec chapter 22 diagnostic payload requirements and the current resolver-code
  `spec_gap`;
- resolver-local `env.md`, `resolved_ast.md`, `declarations.md`, `imports.md`,
  `names.md`, and `labels.md`.

## Purpose

The symbols phase collects resolver-visible declaration signatures after
imports, declaration shells, name lookup, and label lookup have produced their
source-shaped resolver facts. It builds the `SymbolEnv` indexes consumed by the
checker and by downstream module resolution, while preserving unresolved,
ambiguous, malformed, and recovered declaration facts explicitly.

Inputs:

- `ResolvedAst` for the current module;
- local `DeclarationShellSet` and `ExportProjectionShell` records from
  `declarations.md`;
- namespace, symbol-name, import, export, and label projections from the prior
  resolver stages;
- dependency module projections from source-backed fixtures or in-memory
  summaries;
- syntax recovery markers and source ranges owned by `mizar-syntax`.

Outputs:

- local and visible `SymbolIndex` entries with stable `SymbolId`s;
- `DefinitionIndex` records keyed by symbol identity;
- `OverloadIndex` groups for resolver-visible overload families;
- `RegistrationIndex` entries before checker activation;
- resolver-owned synonym, antonym, and redefinition relation records;
- `DeclarationDependencyIndex` edges discovered without type checking;
- dependency-facing `ModuleSummary` projections when the required in-memory
  summary shape is available;
- module lexical summary contributions for downstream active lexical
  environments.

## Boundary

Signature collection is a declaration pass. It may:

- assign stable symbol identities to representable semantic declarations;
- apply public/private defaults from the language specification;
- register source spellings, notation spellings, syntactic arity, declaration
  parameters, contracts, and source-level dependency mentions;
- reject duplicate declarations that cannot form an overload set;
- record illegal overload groups and name-level malformed relation targets;
- build deterministic exported projections for public symbols, labels, and
  lexical spellings.

It must not:

- infer or validate expression, term, or type correctness;
- select an overload winner, rank candidates by inferred types, or insert
  coercions;
- fire registrations, compute cluster closure, or decide registration
  applicability for a term;
- validate selector access by structure type;
- prove definitions, registrations, algorithms, theorems, lemmas, or schemes;
- lower algorithm bodies, compute verification conditions, or decide algorithm
  termination;
- invent parser syntax, build-owned module discovery, driver orchestration, or
  artifact schemas;
- invent public user-facing resolver diagnostic codes while R-G001 is open.

## Symbol-Bearing Shells

R-011 collects both semantic declarations and context-only shells. R-019
classifies symbol-bearing shells before R-020 assigns ids.

Symbol-bearing shells include represented declarations that contribute a module
semantic declaration: structures, modes, attributes, predicates, functors,
algorithms, theorem/lemma results, schemes/templates, registrations, synonyms,
antonyms, redefinitions, and structure members.

Proof-local inline `deffunc` / `defpred` abbreviations are local resolver
binding shells, not module symbols. They may be represented for proof-scope
lookup, but they do not receive exported or module `SymbolId`s, do not populate
the module `SymbolIndex`, and do not seed module lexical summaries unless a
later human-reviewed specification defines a separate local-binding identity
family.

Context-only shells do not receive `SymbolId`s by themselves:

- definition, registration, claim, and proof grouping blocks;
- visibility wrappers, annotation wrappers, and recovered wrappers;
- placeholder, reserve, import, and export container items;
- raw parameter, body, statement, expression, and recovery nodes.

A context-only shell may contribute a structural path, parameter context,
visibility marker, export projection, or recovery state to a symbol-bearing
child. The resolver must not fabricate a symbol identity for a grouping shell
unless a later human-reviewed specification explicitly makes that shell a
semantic declaration.

## Collection Order

Collection is deterministic and declaration-point aware:

1. Import and dependency projections are loaded before local declarations.
2. Local declaration shells are traversed in source order, including
   definition-block and registration-block structural paths.
3. Each representable declaration receives a provisional signature shell and a
   stable `SymbolId` before relation and dependency edges are finalized.
4. Duplicate, conflict, and overload-group checks run over the complete
   source-order declaration inventory for the current module.
5. Export projections, dependency-facing summaries, and lexical summaries are
   built from the public visible surface after local conflicts are recorded.
6. Indexes are sorted into canonical order before debug rendering, cache-key
   input, or module-summary projection.

Forward references to later declarations are not introduced by signature
collection. The phase may record a dependency mention for a syntactic target,
but name-use visibility remains governed by `names.md`.

## Stable Symbol Identity And Origins

`SymbolId` extends the preliminary identity projection from `names.md` into a
complete declaration identity. A normalized symbol origin contains:

- canonical `ModuleId`;
- declaration kind family;
- primary spelling or notation slot normalized by resolver spelling rules;
- definition-block or registration-block structural path;
- declaration ordinal inside the relevant structural owner;
- overload, relation, member, or contract slot when same-spelling declarations
  or declaration-owned subitems can legally coexist;
- source contribution id for invalidation and provenance grouping.

The canonical identity must be stable under formatting changes, trivia edits,
and unrelated local edits outside the structural owner. Source ranges,
`SurfaceNodeId`s, and session-local allocation counters are diagnostic
provenance only; they are not sufficient semantic identity by themselves.

## Signature Shells

Every collected declaration stores a resolver-level signature shell in
`DefinitionIndex`. A common shell records:

- `SymbolId`, declaration kind, visibility, export status, and recovery state;
- primary spelling and optional notation spelling;
- syntactic arity and represented parameter/locus slots;
- source structural path and normalized semantic origin;
- doc/comment attachment ids when available from syntax trivia;
- relation role when the declaration is a synonym, antonym, or redefinition;
- syntactic dependency mentions and unresolved target placeholders.

Signature shells are intentionally shallow. They preserve the syntax needed by
checker/type/proof phases, but they do not claim that the signature is
well-typed, semantically compatible, terminating, executable, or proof-valid.

For attribute patterns with a declaration-site `param_prefix`, the primary
spelling is the declared attribute suffix such as `ranked`, while the optional
notation spelling preserves the concrete pattern surface such as
`2 - ranked`. Resolver symbol collection does not lower the prefix into
checker `AttributeInput` arguments. Exported lexical summaries for attributes
use the suffix primary spelling, because imported attribute uses resolve the
attribute name and carry any arguments through later checker-owned payload
extraction.

## Per-Kind Signature Shapes

| Declaration family | Resolver-owned signature payload |
|---|---|
| structure | Constructor spelling, represented parent/inheritance mentions, field/member names, property names, selector/member structural paths, and recovery state. Field typing and inheritance validity are checker-owned. |
| mode | Mode constructor spelling, label when present, locus/template slots, represented type-parameter surface, radix or parent mode mentions, and source arity. Type expression validity is checker-owned. |
| attribute | Attribute constructor spelling, label when present, optional parameter prefix, locus/template slots, antonym/synonym role if present, and syntactic target mentions. Attribute consistency is checker-owned. |
| predicate | Predicate label when present, predicate pattern or notation spelling, locus/template slots, syntactic arity, and represented definiens/proposition anchors. Formula typing and logical equivalence are checker/proof-owned. |
| functor | Functor label when present, functor pattern or notation spelling, locus/template slots, syntactic arity, optional result-mode mention, and represented definiens anchors. Result typing and coherence are checker-owned. |
| algorithm | Algorithm identifier, terminating modifier, schema/template parameter slots, formal parameter names, optional result type mention, `requires` / `ensures` / `decreasing` contract anchors, and body anchor. Body lowering, execution, VC generation, and termination proof are checker/algorithm/proof-owned. Public summaries expose the signature and contracts, never the body. Algorithms do not seed operator-like lexical spellings. |
| theorem / lemma | Label origin when present, statement shell anchor, template/parameter slots when represented, and cited dependency mentions already resolved by label/name phases. Proof validity and obligation generation are proof-owned. |
| scheme / template | Declared name or label, template parameter slots, formula/term parameter shells, statement shell anchor, and dependency mentions. Instantiation validity is checker/proof-owned. |
| registration | Registration label, registration kind, syntactic target shell, parameter slots, dependency mentions, visibility/export metadata where applicable, and recovery state. Registration activation and cluster closure are checker-owned. |
| synonym / antonym | Alternate pattern shell, original target mention, relation polarity, arity/notation surface, and unresolved or ambiguous target payloads. Semantic equivalence or negation proof is checker/proof-owned. |
| redefinition | Redefined target mention, replacement signature shell, relation ordinal, and compatibility placeholders. Compatibility checking is checker-owned. |
| inline deffunc / defpred | Local proof abbreviation shell only when represented by resolver input. These are proof-scope bindings, not module `SymbolEnv` symbols; they do not receive exported or module `SymbolId`s, seed module lexical summaries, or become exported module symbols. |

R-020 uses explicit declaration projections and may collapse source shell
subitems into the closest resolver-owned symbol family until R-021 performs
parser-backed extraction. `StructureField` uses `selector`; property clauses
and structure properties use the projected `attribute` or `selector` family
chosen by the extractor; predicate/functor/attribute/field/property
redefinitions use `redefinition`; and represented inheritance contributes only
when the extractor provides a `structure` projection. The collapse is explicit
in `SymbolDeclarationProjection`, so the collector never infers a family from a
context shell alone.

If parser coverage does not yet expose a concrete source role, R-021 leaves the
payload opaque and records the shell as pending extraction rather than
inventing source structure.

R-021 extracts represented parser-backed source roles for theorem and lemma
labels, attribute/predicate/functor/mode/structure patterns, algorithms,
notation aliases, redefinitions, property clauses, structure selectors, and
labeled registrations. Template parameters that are direct declaration
children are preserved as named payload roles; template loci nested under
parser pattern nodes remain preserved in the flattened parser-owned signature
surface until syntax exposes a dedicated declaration-owned role. The
parser/syntax boundary currently has no module-level scheme declaration shell;
resolver extraction therefore treats scheme declarations as an external
source-role dependency gap and does not fabricate scheme/template module
symbols.

## Duplicates, Conflicts, And Overloads

Duplicate detection is name-level and kind-family specific:

- non-overloadable declarations with the same namespace, kind family, and
  spelling are duplicate/conflict records;
- private and public declarations still conflict inside the defining module if
  the language would not allow both declarations to coexist;
- same-spelling imports remain visible as candidates until semantic lookup can
  determine whether qualification or aliasing resolves the conflict;
- recovered declarations may retain degraded symbol/definition facts and
  `RecoveredShell` metadata when the spelling and kind family are represented,
  but they do not participate in duplicate or illegal-overload diagnostics.

An overload group may be formed only when the family is overloadable and the
available syntax provides a compatible resolver-owned grouping key:

- namespace or module visibility context;
- surface spelling or symbolic notation;
- kind family;
- syntactic arity or notation shape when available without type checking.

Illegal overload groups are recorded as crate-local/internal diagnostics and
`OverloadIndex` failure metadata. The exact ordinary-functor classifier admits
only parser-backed `SymbolKind::Functor` / `DefinitionKind::Functor` pairs (or
equivalent explicit test projections). It groups by namespace, primary
spelling, normalized notation pattern, normalized definition argument context,
and syntactic arity. Recovered declarations and other kind pairs are excluded.

The bounded ordinary-predicate path binds pattern loci to actual preceding block parameters
with explicit bare set/object types and unique unparenthesized loci. Its private key uses namespace, kind, canonical
notation/locus positions, ordered argument types and arity; labels, means bodies and bound
variable spelling are irrelevant. Public names/identities stay unchanged; unsupported types,
guards, grouped/template patterns and recovered shapes are unsupported. Predicates have no return surface and use
SameSignatureDefinitionConflict with every actual candidate retained in source order.

For a group of at least two candidates whose normalized return surface is
identical, the resolver emits exactly one internal
`SameSignatureDefinitionConflict` diagnostic and stores
`DeclarationConflictClass::SameSignatureDefinitionConflict` on every
candidate definition. For a group containing more than one distinct normalized
return surface, the existing `SameSignatureReturnConflict` diagnostic and
definition-conflict metadata win for the entire group; the resolver emits no
overlapping same-return diagnostic. Both classes retain all candidates in
canonical source order and anchor the diagnostic at the canonical first
candidate. The diagnostic Debug spelling is the Rust variant name
`SameSignatureDefinitionConflict`; the explicit `SymbolEnv` snapshot spelling
of the new definition-conflict metadata is exactly
`same_signature_definition_conflict`.

The functor profile remains a syntactic surface check. Predicate locus canonicalization
does not extend to general type equivalence, imported or template signatures; overload
ranking and winner selection remain checker-owned. The resolver does not
select, rank, or rewrite overload candidates.

## Checker Task 263R Frozen Selector-Owner Conflict Partition

Chapter 5 declares fields and properties inside a particular structure and
requires inherited members to retain both their root declaration and their
inheritance path/view. Consequently, equal selector spellings in different
structure declarations do not form a module-level duplicate. Equal spellings
inside one structure still collide because fields and properties share that
structure's member namespace.

Task 263R changes only non-overloadable `SymbolKind::Selector` conflict
partitioning. Its key is `(namespace, spelling, kind, nearest structure
declaration owner)`. The nearest ancestor shell whose kind is
`StructureDefinition` supplies the owner. A selector without such an ancestor
uses an explicit missing-owner partition and retains the previous conservative
name-level conflict behavior. All non-selector keys remain exactly
`(namespace, spelling, kind)`; redefinitions, overload groups, symbol ids,
definitions, visibility, exports, signatures, contribution effects, and
diagnostic ordering are unchanged.

The required evidence has two extractor-backed resolver cases. Separate structures may
each declare `field carrier` and `property marker` without a
`DuplicateDeclaration`. A field/property collision with the same spelling in
one structure must still produce exactly one canonical duplicate diagnostic
and conflict metadata on both definitions. Projection-order mutation must not
change either result. This task performs no type checking, inheritance
validation, selector lookup, constructor generation, checker intake, or proof
semantics.

## Visibility, Exports, Summaries, And Lexical Contributions

The symbols phase applies default visibility from the relevant language
specification chapters when `DeclarationShellVisibility` is unspecified. Spec
chapter 11 defines ordinary symbol defaults, chapter 12 places definitions and
algorithms in the module interface, chapter 17 defines registration labels and
interface behavior, and chapter 20 defines the algorithm signature/body split.
Explicit private declarations are available inside the defining module after
their declaration point, but are excluded from dependency-facing public
summaries.

Export projections are name-level interface projections:

- public local symbols and legal re-exports appear in the exported surface;
- private declarations are not exported or re-exported;
- an export that targets a private, unresolved, ambiguous, or malformed symbol
  records resolver-owned failure metadata;
- dependency summaries expose only exported symbol and label projections.

The dependency-facing `ModuleSummary` shape produced by this phase contains
only resolver-owned interface data:

- canonical module identity and summary version for the in-memory seam;
- exported symbol entries, label projections, relation links, and overload
  groups;
- declaration dependency edges needed by importers and invalidation;
- module lexical summary entries for public symbolic spellings.

The module lexical summary feeds downstream active lexical environments with
public predicate/functor notation and other lexer-visible user symbols. It
does not include private declarations, algorithm identifiers, theorem labels,
or inline proof abbreviations. R-024 adds artifact-backed summary reuse as a
separate adapter that consumes canonical `mizar-artifact` `ModuleSummary`
records and projects their validated public surface into these summary
contribution indexes without defining a resolver-local artifact schema.

R-020 does not add a dedicated lexical-summary or artifact-summary data shape.
R-021 adds the resolver-local `ModuleLexicalSummaryIndex`; the parser-backed
extractor marks lexer-visible notation tokens (`UserSymbol` / `LexemeRun`) as
eligible, and collection seeds only export-visible, non-recovered projections.
Property keywords, algorithms, theorem labels, structure constructors, and
selectors do not seed active lexical summaries in R-021. R-024 reuses only the
exported lexical entries already present in validated canonical summaries.

## Dependency Edges And Relations

`DeclarationDependencyIndex` records resolver-visible edges discovered during
signature collection:

- signature mentions from one declaration shell to another symbol, label, or
  unresolved target key;
- synonym, antonym, and redefinition target references;
- registration target and prerequisite mentions;
- algorithm contract mentions visible without body lowering;
- theorem/lemma statement mentions visible without proof checking;
- export and re-export edges needed for invalidation and diagnostics.

Edges carry dependency kind, source contribution id, use-site range or
recovered anchor, and deterministic source/target keys. They must not encode
type-derived dependencies, selected overload winners, cluster firing traces,
algorithm execution traces, or proof-obligation dependencies.

## Recovery And Diagnostics

Recovered or malformed declaration syntax is retained when the surrounding
source shape is represented:

- a recovered declaration with a usable spelling may receive a recovered
  signature shell and `RecoveredShell` conflict metadata;
- a recovered declaration without enough identity data remains a shell-only
  unresolved declaration fact;
- malformed relation targets record unresolved or ambiguous target payloads
  instead of fabricating links;
- duplicate and illegal-overload diagnostics ignore recovered declarations so
  parser recovery does not cascade into semantic conflict reports.

Diagnostic records remain crate-local/internal while R-G001 is open. They
preserve source ranges, declaration origins, conflict candidates, relation
targets, and recovery state, but assign no public numeric resolver codes.

`SignatureProjectionExtractor::collect(module)` reuses opaque symbol collection and adds
source-owned `SymbolDiagnosticClass::SynonymLociMismatch` only for ordinary local functor
synonyms with uniquely bound bare builtin loci. It authenticates both patterns and the unique
functor in an earlier completed definition block by namespace, notation shape and ordered locus types; binder names,
labels and bodies do not define constructor identity. Parenthesized argument lists are allowed;
repeated/unbound loci, recovery, ambiguous or unsupported targets cannot supply this diagnostic.
The diagnostic anchors the actual alias shell/range, identifies the original constructor SymbolId,
and belongs to the source contribution. Unequal synonym locus counts still reject the alias. A plain binary functor synonym or predicate antonym with the same bound-locus set and a unique same-source local target of the matching constructor kind and namespace in an earlier completed definition block may publish existing RelationMetadata(Synonym/Antonym, target); count equality alone is insufficient.
This records original-constructor identity and antonym inversion, not proof acceptance; checker replays ordered source loci at each application. Source collection/replay authenticate the relation; opaque collection, numeric codes, predicate synonyms, alias chains and unsupported targets retain their boundaries. Antonym mismatch supplies no synonym-loci diagnostic.

## Determinism

Signature collection must be byte-stable for equivalent inputs:

- declaration traversal follows source order and structural paths;
- symbol ids, overload slots, relation ordinals, and member slots are assigned
  from canonical grouping keys;
- candidate and diagnostic lists are sorted by normalized semantic origin,
  kind family, source range, and declaration ordinal;
- maps are rendered through sorted projections, never raw hash iteration;
- debug snapshots contain normalized origins and stable spellings rather than
  session-local addresses or allocation ids.

## Public Enum Forward-Compatibility

Task R-026 applies the frontend task-25 public-enum decision procedure to this
module. All public resolver-owned enums in `symbols` are forward-compatible API
surfaces and must remain `#[non_exhaustive]`:

- `SymbolOverloadPolicy`
- `SymbolDiagnosticClass`

No exhaustive public enum exceptions are owned by this module. Downstream
consumers must keep wildcard or fallback arms; resolver-internal matches may
remain exhaustive over the currently represented variants when implementing the
specified behavior.

## Test Obligations

R-019 is documentation-only and adds no executable tests. R-020 adds resolver
unit tests for:

- registration of opaque declaration signatures into `SymbolEnv`;
- duplicate/conflict detection per represented kind family;
- legal and illegal overload grouping;
- registration insertion into symbol, definition, and registration indexes;
- recovered and context-only shell policy;
- deterministic symbol, definition, overload, diagnostic, and contribution
  ordering.

R-021 adds resolver unit tests for parser-backed per-kind signature extraction
over represented source roles, template-role preservation in opaque payloads,
and parser-backed lexical-summary spelling fixtures where the syntax supports
them. R-023 adds semantic `.miz` corpus cases and traceability metadata for the
`declaration_symbol` stage. Existing `.miz`
cases and expectations must not be rebaselined to match resolver
implementation behavior.

## Checker Task 263R Implemented Selector Partition

The frozen correction is implemented without changing the public symbol
model. Each collected selector privately records the nearest ancestor
`StructureDefinition` shell, if one exists. Duplicate collection keys
selectors by `(namespace, spelling, kind, selector owner)`; every other symbol
kind continues to use the pre-existing `(namespace, spelling, kind)` key with
the owner coordinate fixed to `None`, and ownerless selectors all use that
same `None` owner partition. The owner is conflict-classification data only: it does not alter
`SymbolId`, definition/signature shells, module summaries, lookup, provenance,
overload/redefinition rules, or recovery.

The cross-structure extractor test produces six definition symbols and no
conflict in either projection order. The same-structure field/property test
produces one stable `DuplicateDeclaration` and two `DuplicateSpelling`
conflict marks in either order. The existing ownerless/non-selector duplicate
matrix remains unchanged. This closes the frozen `source_drift` and resolver
`test_gap`; structure inheritance, constructor selection, and checker
semantics remain deferred to Task 263.

## Checker Task 264R Context-Only Symbol Boundary

`DeclarationShellKind::PropertyImplementation` is context-only in Task 264R.
`SignatureProjectionExtractor` returns no projection for it, and normal symbol
collection must create no property-owned `SymbolId`, `DefinitionId`,
`RegistrationId`, overload/redefinition relation, lexical summary, dependency,
contribution effect, or diagnostic. The existing one-per-source `LocalSource`
contribution remains; its anchor ignores property-implementation shells and
falls back to `Point(0)` when no represented semantic shell exists. Context-only
property shells do not contribute to semantic sibling ordinals, so inserting
them cannot change an existing sibling's structural path, `SymbolId`, origin,
definition, or contribution effects. Existing enum order and kind codes/keys
remain stable; the new variant is append-only after `VisibilityWrapper`, with
code `28` and key `property-implementation`. A caller-fabricated projection
retains the existing `ContextOnlyShell` guard/diagnostic rather than receiving
an identity. It must not be represented as `Selector`, `PropertyClause`,
`PropertyRedefinition`, or `RegistrationKind::Property`. Existing siblings keep
their exact identities and order. The later checker source producer, not the
resolver symbol table, owns property reference/provenance, implementation
domain, return-type and definiens payloads.

## Checker Task 264R Implemented Symbol Boundary

Property-implementation shells now extract no projection and create no symbol,
definition, registration, overload, dependency, lexical summary, diagnostic,
or property-owned contribution effect. Anchoring skips leading property shells
and falls back to `Point(0)` for property-only sources. Semantic sibling
ordinals exclude them, preserving the recovery theorem's `theorem#1`,
`[2, 1]`, `304..334`, `DefinitionId(1)`, and contribution-zero fingerprints.
A fabricated projection still fails with `ContextOnlyShell`.

## Resolver Task 33R Exact Containing-Functor Owner Receipt

The canonical [Task 33R contract](../../task_contracts/en/RESOLVE-FRAENKEL-CONTAINING-OWNER-33R.md)
adds one specialized zero-semantic association at the existing symbol and
definition allocation boundary. `SourceNestedFraenkelFunctorOwnerProducer`
accepts only the surface AST, module, complete structural resolver arena, and
exact fingerprinted Fraenkel collection. It internally recollects declaration
shells, parser-backed projections, and the symbol result, then selects exactly
one unrecovered `FunctorDefinition` shell whose `node_id` maps through
`SurfaceResolvedArena::resolved_node_for` to the C4C8R common functor identity.

The opaque immutable `SourceNestedFraenkelFunctorOwnerHandoff` retains the
entire derivation and exposes only immutable source/module/fingerprint,
definition-block/functor resolved nodes, declaration shell, final `SymbolId`,
`DefinitionId`, contribution, and origin getters. The non-exhaustive
`SourceNestedFraenkelFunctorOwnerError` owns the seven contract-frozen
default-deny classes. The public complete oracle and exact resolver-collection
oracle rederive the row in fixed precedence. Exact return types are
`SourceId`, `&ModuleId`, `&str`, two `ResolvedNodeId`s,
`DeclarationShellId`, `&SymbolId`, `DefinitionId`, `SourceContributionId`,
`&SemanticOrigin`, and `String`; both validators return
`Result<(), SourceNestedFraenkelFunctorOwnerError>`. The task contract owns the
complete signatures and exact fieldless error/display ABI.

The existing public `SymbolCollector::collect()` remains unchanged and
delegates to a private no-target path. The specialized producer supplies the
already authenticated target shell to that same private path, which captures
the exact `CollectedProjection::new` symbol/origin/contribution and the
`DefinitionId` returned during insertion. This private in-flight allocation
row is returned with the ordinary symbol result and rederived by the complete
oracle; the association is never reconstructed by searching a finished
environment.

Association uses the existing `CollectedProjection::new` identity path and
the final definition index, never display spelling, ranges, or numeric-id
reinterpretation. Token text may remain inside the exact surface fingerprint
or debug output but is not an association/admission key. Missing, extra,
duplicate, reordered, stale, foreign,
recovered, partial, mismatched, or corrupt input is rejected without sorting,
repair, inference, or unchecked admission. This receipt is not installed in
checker, Typed/Resolved, or Core and adds no semantic result, parameter order,
generated origin, diagnostic, active route, coverage credit, or Task-277B
readiness.

The public-enum policy registration additionally includes:

- `SourceNestedFraenkelFunctorOwnerError`

## Source lexical export correspondence

`SymbolCollectionResult::pair_exported_lexical_declarations(locals, map_span)`
returns `Option<Vec<(&SymbolEntry, &LocalUserSymbolDeclaration)>>`. The caller
must supply locals from the same frontend source as the collection and map
lexical `declared_at` spans through that source's preprocess map. The mapper
returns `Option<MappedSourceRange>`; only `Exact` mappings are supported, using
their loaded `primary` range. Composite/degraded mappings reject the result. This operation checks correspondence, not source authenticity.
It does not produce a complete module summary or bind artifact identities.

Each nonempty mapped span must be contained by exactly one symbol origin range
before visibility filtering. Its source id and module must agree
with the origin and `ContributionKind::LocalSource { source_id }` contribution;
recovered origins are rejected. Local lexer module names must equal
the resolver module path (the existing frontend convention). Supported exact
kind pairs are predicate, functor, mode, attribute and structure. Only
`Visibility::Public` + `ExportStatus::Exported` pairs are retained;
`Visibility::Private` + `ExportStatus::LocalOnly` pairs are omitted.
`ReExported` and all other combinations,
unsupported or missing owners, ambiguity, invalid/missing mappings, malformed
lexical spans or collection diagnostics reject the entire result with `None`.
Borrowed declarations preserve every field and input order; several lexical
pieces may pair with the same symbol. Empty input yields an empty vector when
collection diagnostics are empty. No metadata is inferred from resolver notation.

Aliases, standalone operator declarations, absent prepass families, completeness,
source authentication and artifact origin serialization remain producer work.
This method considers `locals.user_symbols` only; it cannot authorize publication
of a summary that omits other lexical effects.

### Frontend-bound correspondence

`SymbolCollectionResult::pair_frontend_lexical_declarations(frontend)` accepts
`FrontendOutput<SurfaceAst>` from a trusted, unchanged frontend run and returns
the same borrowed pairs as the lower correspondence method. It binds the source
package/module, source ids, retained preprocessing and local declarations, and
the exact parser-backed resolver collection; it does not authenticate arbitrary
constructed frontend outputs or grant semantic acceptance.

Reject absent ASTs, any frontend diagnostics, mismatched source ids or module,
source text/hash/line-map inconsistency, and failed source-map registration.
Re-run preprocessing on the retained source and require equality with the retained
preprocessed output; re-scan that lexical text and require equality of collected
locals. This public raw rescan does not reproduce the frontend's private
parser-planned scanner; any resulting mismatch is unsupported and rejects.
Recollect declaration shells and signatures from the supplied AST and
require equality with this collection. No source loading, provider call or parser
replay occurs. Delegate through the reconstructed bridge's lexical mapping,
preserving mapping status and all existing correspondence exclusions.

Integration tests use the real disk frontend/parser and source-derived resolver
collection, with no imports or synthetic semantic projections. They exercise
comment-shifted coordinates, public/private declarations, retained metadata,
repeatability and mismatched or stale inputs. Test-only provider plumbing does
not implement a production lexical-summary provider.

### Source lexical contribution producer

`SymbolCollectionResult::export_frontend_lexical_contributions(frontend, identity)`
returns `Option<Vec<LexicalContributionSummary>>` for the frontend-bound direct
pairs. The caller supplies build-owned `ModuleSummaryIdentity`; package, module
path and edition must match the retained source. Artifact identity validation and
JSON normalization stay with `ModuleSummaryIdentity::canonical_json`. Version
and lockfile provenance remain the caller's responsibility.

Reject any standalone operator declarations or AST notation aliases, identity
mismatch, failed frontend correspondence or A8 encoding failure; never return a
partial vector. Each pair emits kind `exported-symbol` under producer schema
`mizar-resolve/exported-lexical/v1`. Its key is `entry.symbol().local().as_str()`;
the corresponding exported symbol producer must use that same `origin_id`.
Construct A8 shapes from unchanged local spelling, rank, kind, arity and operator,
with A10 envelope-bound module/symbol identity strings (including their canonical
newlines). Use the artifact-owned identity JSON and canonical string writer.
Preserve pair order, private-filter rank gaps and multiple spellings per origin;
do not deduplicate by key or infer metadata from normalized resolver notation.

This produces contributions, not exported symbol rows, a complete module/lexical
summary, lexical fingerprint, import provider or publication. Missing source
prepass families and export policy remain with their owners. Tests cover actual
frontend multi-piece/private/default-operator payloads, identity changes and
rejections, unsupported aliases/operators, codec bounds and deterministic output.

## Constructor Actual Roles

Future constructor-actual resolution consumes real mode/structure signatures and ordered type/value parameter roles, preserving declaration/scope and expression provenance for bracket/of/over actuals. It does not infer roles from spelling or choose typed overload applicability. Missing or stale declaration inputs block the semantic consumer.

## Planned Algorithm Call Dependencies

The existing signature index is not a source body-call/definiens graph. Future authentic call targets, complete definition-block recursive groups and source/range/edge provenance feed Checker cycle/applicability checks and Core lowering; Resolver never chooses typed overload winners or treats graph references as accepted calls.

## Source registration owner payloads

`source_registration_owner_payloads(ast: &SurfaceAst, source: &SurfaceResolvedArena, symbols: &SymbolEnv) -> Result<BTreeMap<SymbolId, Vec<u8>>, String>` derives named owner bytes while retaining the actual resolved SymbolId as the map key. Validate the exact AST/arena/module association first, then perform the existing whole-environment replay and preserve its errors. Recollect existing shells/projections from that same AST; no caller identity string, range-only match or debug text supplies owner content.
The profile supports only AttributeDefinition in a top-level DefinitionBlock and ExistentialRegistration in a top-level RegistrationBlock. An attribute pattern must have exactly one direct constructor token outside its optional ParameterPrefix, with spelling equal to the actual projection primary spelling; otherwise refuse the payload. Each selected declaration needs its actual mandatory label, actual represented projection and exactly one corresponding source SymbolEntry; reject duplicate labels within either owner kind, duplicate symbol association or any unsupported/unmapped Attribute/Registration entry. Other symbol kinds are ignored. Any failure returns Err without a partial map; a source with no target owners yields an empty map. This withholds optional identity data rather than changing name resolution, overload selection or public diagnostics.
Each value starts with UTF-8 `mizar-resolve/source-registration-owner/v1` and a zero byte, then kind byte 1 (attribute) or 2 (existential registration). Encode package, module path and exact label in that order as checked big-endian u32 UTF-8 byte length followed by bytes; append the actual constructor primary spelling in the same framing only for attributes. Keep parser spelling without case/Unicode normalization. Duplicate labels are conservative refusal, not a new language diagnostic. No SourceId, range, node/row/contribution id, structural ordinal, opaque signature, binder spelling, source body or debug provenance participates.
Module identity and unique named declaration content supply this bounded cross-edit payload. Formatting, trivia, raw offsets, bound-variable spelling and declaration/block order cannot change it. Changing module, kind, label or attribute constructor changes it. SymbolId continues to own resolution and is never replaced with a raw lookup string; values are unhashed canonical owner content only. Formula/context/definition bodies, complete anchors/dependencies, proof reuse and witness/Artifact publication require separate genuine producers; currentness and source-execution provenance remain caller responsibilities.
Tests use actual Frontend/resolver single/aggregate source inputs, independent exact framing and actual SymbolId association; genuine allocation relocation, edits/reordering and renamed/module variants; foreign AST/arena/env, recovery, missing/duplicate/unsupported owner refusal and old-result preservation. At most one new private genuine setup helper serves three actual families. Existing source replay, symbol identities, diagnostics, fixtures and proof/coverage expectations remain untouched.

## Arena registration owner payloads

`source_registration_owner_payloads_from_arena(source: &SurfaceResolvedArena, symbols: &SymbolEnv) -> Result<BTreeMap<SymbolId, Vec<u8>>, String>` uses the existing structural reconstruction and whole-environment replay, then delegates to the [AST-bound owner producer](#source-registration-owner-payloads). Return its exact current SymbolId associations and bytes, including empty results; any reconstruction/environment or owner-profile failure returns the existing error without a partial map. No raw source text, parser run or caller identity map supplies missing data.
The reconstruction uses actual stored kinds/token text, child order, ranges, root/source/module and existing SurfaceAstBuilder validation. It is a structural projection, not restoration of original trivia, expression-root or complete Frontend storage. The returned owner map retains the existing producer's content-only boundary: callers own source currentness; no execution receipt, formula/VC hash, complete anchor/dependency, proof acceptance/reuse or publication follows.
Tests use actual single/aggregate Frontend AST/arena/env, independent current-key bytes and equivalence with the AST-bound producer; genuine edits/reordering/allocation relocation and participating content controls; foreign/augmented environments, recovered inputs and unsupported owner-profile refusal with unchanged legacy validation/errors.

## Source registration syntax payloads

`source_registration_syntax_payloads_from_arena(source: &SurfaceResolvedArena, symbols: &SymbolEnv) -> Result<BTreeMap<SymbolId, Vec<u8>>, String>` returns declaration-local syntax content for current existential-registration symbols. First use existing structural AST/environment reconstruction, then the existing AST-bound named-owner producer, preserving upstream errors. Recollect shells from that AST and join each existential shell to exactly one Registration entry by full existing `shell_origin`/`shell_context`; require its genuine owner-map key. Return exactly all Registration entries, ignoring other kinds; empty/attribute-only sources yield an empty map after owner validation. Any later failure returns `unsupported source registration syntax payload`, with no partial result.
Each value begins with UTF-8 `mizar-resolve/source-registration-syntax/v1` and zero, then checked big-endian u32 byte length and the genuine existing named-owner bytes. Traverse that shell's actual subtree in ordered preorder. Each node emits its explicit Syntax-owned `SyntaxKind` u16 discriminator, big endian; a Token additionally emits its token-kind SyntaxKind u16 and checked big-endian u32 UTF-8 byte length plus exact text. Every node then emits checked big-endian u32 child count, followed by those children recursively in stored order. No ids, ranges, trivia, Debug or storage strings participate; token spelling/case/binder text inside the subtree is retained without semantic or alpha normalization.
The only supported non-token variants are unit kinds: ExistentialRegistration, TypeExpression, AttributeChain, AttributeRef, QualifiedSymbol, PathSegment, TypeHead, CorrectnessCondition, ProofBlock, TakeStatement, Witness, TermExpression, ChoiceTerm, ConclusionStatement, Proposition, FormulaExpression, IsAssertion, AttributeTestChain, JustificationClause, ReferenceList and Reference. Tokens support only Identifier, ReservedWord, ReservedSymbol and UserSymbol. Reject all other variants, including parameter-bearing operators, quantifiers, constants and recovery, rather than dropping their fields. This is an optional content profile, not a language acceptance restriction; Syntax/Parser retain ownership of the represented tree and its vocabulary.
The selected root excludes its enclosing block/bindings and referenced definition bodies. Independent context/formula producers must account for those inputs. In particular, editing an external attribute body leaves these local bytes unchanged. Formatting, allocation and declaration order leave corresponding owner values unchanged; package/module, owner label and any represented local token or shape change participate. Caller-owned currentness remains separate. These raw bytes do not construct complete SourceShapeHash/anchor/dependencies, proof identity, acceptance/reuse or publication.
Tests use real single/aggregate Frontend/resolver inputs and manually composed grammar bytes independent of the producer traversal, including the aggregate assertion's actual TypeExpression/TypeHead shape. Cover actual current-key association, neutral and participating edits/relocations, empty results, genuine upstream and unsupported-subtree refusal, and immutable prior inputs/results. No fabricated positive AST or accepted/cache state is supplied.

## Source Public Theorem Signature Candidates
`source_public_theorem_signature_candidates(&SurfaceAst, &SurfaceResolvedArena, &SymbolEnv) -> Result<Option<BTreeMap<SymbolId, SignatureShell>>, String>` supplies unpublished body-free surface-token statement shells for the actual public theorem/lemma owners. Reuse existing types; do not replace signatures in the original env.
First call the [original formula association](./labels.md#source-public-theorem-formula-candidates), preserving its complete AST/arena/unaugmented-env validation, source profile, errors and whole-call None. Return exactly its actual SymbolId key set; admitted all-private input yields Some(empty). Never substitute raw names, current allocation guesses or empty values for missing producers.
For each associated original formula, visit only its subtree in stored child order and retain every token once; a LIFO walk reverses children. Do not walk Root's compatibility token listing. Accept only Identifier, ReservedWord, ReservedSymbol, Numeral, LexemeRun and UserSymbol tokens with nonempty text and UTF-8 byte length representable as u32. Any other token, unrepresentable length or token-free selected formula refuses the entire map, including otherwise supported owners; never skip a token or return a partial map.
Each value is SignatureShell::Opaque with schema source-theorem-statement-tokens-v1. Its String payload concatenates one record per token: the unsigned decimal value of token.kind.syntax_kind() as u16, colon, checked u32 UTF-8 byte length in decimal, colon, exact token text. Decimal numbers have no leading zeroes. Length framing delimits each text even when it contains delimiters; there is no separator, normalization or Debug encoding between records.
Payloads include original token kinds, spelling, binder names and order only. They exclude owner label/role/status, module identity, IDs, ranges, trivia/comments, proof/justification bodies and non-token node kinds or variant fields. Thus proof-only, trivia, allocation, owner-renaming and declaration-order changes preserve corresponding values; participating token-kind/text/order changes affect them. This is a framed surface-token projection, not an AST codec, alpha-normalized or elaborated statement, semantic-equivalence decision, semantic/interface/imported-statement fingerprint, or proof/status authority.
Append exactly two source_public_theorem_signature_candidates_* families in labels/tests.rs using its unchanged genuine Frontend helper. Independently assert literal complete payloads/schema and actual original public SymbolId associations for private/public, lemma, open/assumed and numeral sources; private exclusion/all-private empty, replay and immutable inputs. Compare genuine proof-only/trivia/comment/relocation/rename/reorder controls and participating binder/operator/numeral edits without using the producer's traversal as the oracle. Code remains ASCII-only under spec02; Unicode comment edits supply no code tokens.
Source-derived unsupported-token negatives rebuild the genuine collectors/arena, first prove original formula association Some with a supported public owner retained, then require whole-map refusal. Reuse inherited validation/admission matrices, with representative real foreign AST/env and unsupported-owner/error-before-admission cases. Do not fabricate positive ASTs, source owners, scope, accepted states or cache hits.
Env signature replacement/composition and dependency joins, complete ModuleSummary/VerifiedExport and semantic fingerprints, source acquisition/current-runtime authentication, public diagnostics, service registration, phase15, type/proof acceptance, witnesses/footprints/reuse/cache/Doc and Steps6–8 completion remain separate work. Existing collectors, signature schemas/callers, source text producer and all protected language/test intent are unchanged.

## Source Theorem Module Candidates
`source_theorem_module_candidates(&SurfaceAst, &SurfaceResolvedArena, &SymbolEnv) -> Result<Option<(ResolvedAst, SymbolEnv)>, String>` composes an unpublished source-profile metadata pair using existing types. The caller retains the original unaugmented env; this derived env is not a replacement source-replay receipt or a complete ModuleResolver/Checker input contract.
First call source_theorem_resolved_ast, then source_public_theorem_signature_candidates, then source_theorem_label_reference_candidates on the same original AST/structural arena/unaugmented env. Preserve each actual lower error/None in that order. Return the actual complete semantic AST unchanged and use the real label result's complete index, never a reconstructed/empty substitute for a missing producer. All-private or citation-free inputs use genuine empty producer batches only after admission.
Further restrict this optional aggregate to actual LabelResolution::Resolved rows throughout the semantic AST. Any unresolved, ambiguous or other citation outcome refuses the entire pair, without changing the older APIs' outcomes, diagnostics or lookup policy. Never select a winner or drop a citation; no citation means this condition is vacuous.
Clone the original twelve index families, replacing the label index with the exact real result and recording every real label origin path under its actual contribution. Keep every original symbol/definition/contribution identity, origin, visibility/export flag and unaffected index/field/counter. Require actual ownership associations; do not rely on contribution mutators silently ignoring nonexistent IDs.
Use each actual public signature-map value unchanged in both the corresponding SymbolEntry and DefinitionEntry. Consume exactly the actual map key set once in each family; missing/extra/duplicate associations refuse the pair. Private signatures and all other fields remain original. Rebuild SymbolIndex rather than appending duplicates; rebuild definitions in their original numeric DefinitionId order, preserving every optional field, shell-id vector, existing dependency and allocated id. SymbolId-sorted iteration is not allocation order; never construct replacement IDs from names or counters.
For every real label-reference row, obtain its enclosing theorem from the existing checked hierarchical citation origin [theorem surface id, statement id, reference id]. Join the genuine declaration shell to exactly one original SymbolEntry using full existing shell_origin/shell_context and kind; range alone, target spelling or parsed FQN fragments are insufficient. Require the resolved target's actual label-index association and keep original table IDs/sites/origins.
In actual LabelRefId iteration order, allocate one DeclarationDependencyIndex edge per citation on a clone of the original dependency index: source Symbol(actual enclosing owner), target Label(actual resolved origin path), kind LabelCitation, actual reference-site range anchor, actual source-owner contribution. Preserve old edges/IDs; record each fresh real edge ID in that contribution's effects and that source definition's dependency list. Preserve repeated citation occurrences as separate edges. These are syntactic citation dependencies for resolver metadata, never proof-obligation or accepted-used-fact dependencies.
Return existing SymbolEnv::new for the same module with those composed indexes, alongside the unchanged actual semantic AST. Any failed whole-association check yields None, no partial pair. Do not mutate the original arena/env/AST, alter their provenance, relabel the derived env as unaugmented, add local SymbolIds, or replace any lower table/state with invented data.
Append exactly two source_theorem_module_candidates_* families to labels/tests.rs using its unchanged genuine Frontend helper. Independently check actual original keys/IDs, full literal public signature values in both views, private retention, complete label rows/contribution effects, per-citation source/target/kind/anchor/real edge IDs and source-definition associations, all retained definition fields and untouched index families, unchanged semantic AST, replay and inputs. Include private/public, lemma, open/assumed, citation-free/numeral and all-private sources; a genuine source with at least twelve theorems must first demonstrate non-allocation-ordered DefinitionIndex iteration, then exact ID preservation.
Negative tests must show actual source_theorem_resolved_ast Some with a real unresolved citation before aggregate None; reuse inherited profile/error matrices with representative refusals/foreign inputs and actual lower precedence. Exercise a source-derived negative where semantic AST remains Some but the genuine public-signature producer returns None, then require whole-pair refusal, never an empty signature substitute. Rebuild actual collectors/arena for such negatives; do not fabricate a positive AST, owner, label outcome, producer, evidence or accepted/cache state. Check the original replay receipt remains usable and the genuinely augmented fixture env does not pass as that original receipt.
This pair supplies bounded resolver metadata only. General ordinary names/imports, private VisibleItem Checker adoption, complete env/summary/VerifiedExport and semantic/interface/imported-statement fingerprints, public diagnostics, source/current-runtime authentication, storage codecs/services/phase15, type/proof acceptance, complete phase footprints/witnesses/reuse/cache/Doc and Steps6–8 remain separate. Original collectors, schemas, callers, diagnostics, recovery, proof policy and protected test intent do not change.

## Source Theorem Module Original Inputs
`source_theorem_module_original_inputs(ast: &SurfaceAst, module: &ModuleId) -> Result<Option<(SurfaceResolvedArena, SymbolEnv)>, String>` returns the actual original structural arena and unaugmented collector env, using existing types.
Use DeclarationShellCollector::new(ast, module).collect(), then SignatureProjectionExtractor::new(ast, &shells, NamespacePath::new(module.path().as_str())).collect(module). Actual collector diagnostics return Ok(None) before SurfaceResolvedArena::lower(ast, module); preserve its error.to_string(). Return Some((actual arena, collected.into_env())), moving the existing owned env unchanged without cloning or reconstructing indexes/IDs.
Some reports successful construction only, not admission to the later theorem metadata profile: unsupported/unresolved citations or proof variables can still yield the actual pair producer's None/String. Neither an empty substitute nor the augmented output env replaces this original receipt; AST/module and all retained data remain unchanged. Source/module provenance and currentness stay caller-owned.
The two native module consumers and storage encoder/decoder each call this factory directly and use BOTH returned values with the same [module-pair producer](#source-theorem-module-candidates). Native capture, typed-parent/whole-output equality and physical/publication prechecks/rechecks keep their order and errors. Storage framing/caps precede the factory; nested decoding retains the supplied genuine SourceId/module. Encoder/native factory None/String stay None/String; decoder converts actual factory/pair refusal or error to None.
Append only `source_theorem_module_original_inputs_match_real_original_context` and `source_theorem_module_original_inputs_preserve_constructor_and_admission_boundaries` in crates/mizar-resolve/src/labels/tests.rs. Keep parsed_private_theorem_fixture/NoImportedSummaries and every old test/import/fixture/expectation unchanged; no new helper.
Use genuine fixture AST/module and independent pre-factory old collectors/lowering as the whole-arena/all-env-field/ID/counter oracle, including other package/dotted module controls. Obtain the whole old pair before the factory and compare it using BOTH actual returned values; distinguish the genuinely augmented env. Cover private/public, all-private, lemma, open/assumed, numeral/mixed values, repeated citations, twelve-declaration nonallocation order, BOM/CRLF/Unicode trivia, repeatability and immutable AST/module inputs. Reuse existing signature/ID/citation/storage/native matrices.
Require actual factory Some before genuine unsupported/unresolved/foreign-qualified citation/reserve or unresolved-proof-variable pair None/String; compare to independent old assembly/outcome and unwrap the real variable error. Parse [duplicate theorem source](../../../../tests/miz/fail/resolve/fail_resolve_duplicate_theorem_symbol_001.miz) with inline existing Frontend setup because the unchanged helper rejects collector diagnostics; assert actual diagnostics before factory Ok(None). Preserve AST/module. No fabricated positive AST/error/producer/state; valid parser AST lower-error conversion retains static/existing lower coverage.
Gross production additions are at most 100 lines, the factory at most 32 and test additions at most 240. Production changes are confined to the new function and four context-assembly blocks in symbols.rs/frontend_adapter.rs; all other functions, guards, grammar, errors and producers stay byte exact. No new type/field/adapter/codec/hash/profile/dependency or parser/filesystem/id allocator. The original-input seam grants no semantic stream, fingerprint, complete env/ModuleResolver/summary/VerifiedExport, service/IR sealing, execution/currentness, proof/acceptance/cache/phase footprint or Steps6–8 completion.

## Source Theorem Module Storage
`source_theorem_module_candidates_canonical_bytes(&SurfaceAst, &ModuleId) -> Result<Option<Vec<u8>>, String>` encodes only the pair deterministically produced from that actual source AST and module by the original collectors and [source_theorem_module_candidates](#source-theorem-module-candidates). It is not a generic serializer for arbitrary ResolvedAst/SymbolEnv values.
First obtain the complete existing [SurfaceAst canonical storage](../../mizar-syntax/en/ast.md#publication-storage); storage-invalid/out-of-domain input yields None. The aggregate is limited to 32 MiB, including checked framing and module bytes; reject overflow/oversize before resolver replay. These are storage limits, not language acceptance limits.
Use [original module inputs](#source-theorem-module-original-inputs) on the same AST/module. Collector diagnostics yield None; lowering and actual pair errors remain the original String; actual pair None remains None. Encode only after the actual complete pair is Some, with no independent admission or substituted tables.
The exact envelope starts with UTF-8 `mizar-resolve/source-theorem-module-storage/v1` and zero. Append exactly three fields in order: literal UTF-8 module package, literal UTF-8 module path, and the unchanged complete Syntax canonical AST bytes. Each field has a checked u64 little-endian byte length followed by its exact bytes. No source ids, debug text, semantic-table copies, new syntax vocabulary or source-body reconstruction are stored. Body/trivia/ranges remain in the existing nested AST format; these bytes are not semantic/interface/imported-statement fingerprints.
`source_theorem_module_candidates_from_canonical_bytes(&[u8], SourceId, &ModuleId) -> Option<(ResolvedAst, SymbolEnv)>` requires the caller's real typed source id and expected module. Check aggregate cap/domain/three lengths/truncation/trailing bytes and byte-exact package/path equality to that expected module before nested decode. Never create a module or source identity from stored text/bits. Decode through existing SurfaceAst::from_canonical_bytes with that supplied source id, inheriting all canonical/topology/trivia/resource validation.
Use the same [original module inputs](#source-theorem-module-original-inputs) and actual pair producer on that decoded AST and expected module, returning its entire genuine pair. Any storage, collector, lowering or lower producer refusal/error yields None. The decoder loads no file, allocates no source id, parses no source and grants no source-execution/currentness/proof/publication authority. Successful rebinding authenticates storage structure and explicit module equality only; the caller owns source/capture/provenance/current-runtime validation.
For the same source id, decoded pair equality includes the complete semantic AST and all twelve env index families, original ids/fields/counters, actual signatures/labels/citation edges/effects. Rebinding must equal a genuine producer run over that same stored AST under the supplied real id; never patch ids in a previously resolved pair or relabel the derived env as the original collector receipt. Existing producers, schemas, lookup and error policy stay unchanged.
Append exactly two source_theorem_module_storage_* families using the unchanged real Frontend helper and existing codecs/collectors. Cover real private/public, all-private, lemma, open/assumed, repeated and distinct citation endpoints, unequal signatures and twelve-theorem non-allocation order, plus normalized BOM/CRLF/internal Unicode trivia. Compare complete original and independently rebound genuine producer pairs, fixed literal envelope prefix/nested AST bytes, repeatability and unchanged inputs. Allocate a genuinely distinct SourceId through the existing session allocator; decoded Syntax storage is not an execution receipt. Reuse lower matrices rather than duplicate them.
Negative controls use genuine unsupported/unresolved and unresolved-proof-variable lower outcomes; encoder preserves actual None/String, decoder rejects their source-derived envelopes. Check every strict truncation, wrong domain, length overflow, surplus fields/trailing data, malformed/noncanonical/unsupported nested Syntax bytes, expected-module mismatch and outer size limit. Source-inconsistent AST mutations are negative only. No fabricated positive AST/owner/frame/producer/accepted/cache state. No filesystem/parser or id allocator is introduced in production.
This source-derived storage prerequisite does not seal IR or register a service. General names/imports and arbitrary resolver values, full ModuleResolver/Checker, complete summaries/VerifiedExport, semantic fingerprints/public diagnostics/live runtime, phase15/type/proof acceptance, complete footprints/witness/reuse/cache/Doc and Steps6–8 remain separate.

## Source Theorem Syntax Payloads

`source_theorem_syntax_payloads_from_arena(source: &SurfaceResolvedArena, symbols: &SymbolEnv) -> Result<BTreeMap<SymbolId, Vec<u8>>, String>` supplies optional declaration-local raw syntax. First call the existing `validated_source_symbol_ast(source, symbols)?`, preserving complete actual AST reconstruction, whole-environment collector replay and upstream errors; retain that actual AST. No new named-owner producer is needed.
Recollect actual declaration shells from that AST and module. Select only parentless `DeclarationShellKind::Theorem` shells; the collector unwraps an outer VisibleItem without a shell parent. Join each shell to exactly one actual `SymbolKind::Theorem` entry by full existing `shell_origin(ast.source_id, source.module(), shell, &shell_context(...))`, never name/id fallback, and key its actual subtree bytes by that entry's current SymbolId. Ignore Lemma and other declaration kinds; empty/other-only input returns empty only after validation.
Each value starts with UTF-8 `mizar-resolve/source-theorem-syntax/v1` and zero. In stored-child ordered preorder, emit every actual node's `syntax_kind()` as big-endian u16; Token additionally emits token-kind SyntaxKind as big-endian u16, checked big-endian u32 UTF-8 byte length and exact text. Every node then emits checked big-endian u32 child count followed recursively by those children; a LIFO walk reverses children. No owner bytes, ids, ranges, structural paths/order indices, trivia, Debug/storage strings or module/visibility metadata enter the value.
The only supported unit non-token kinds are TheoremItem, FormulaExpression, QuantifierVariableSegment, TypeExpression, TypeHead, BuiltinPredicateApplication, TermExpression, TermReference, ProofBlock, LetStatement, QualifiedVariableSegment, ConclusionStatement and Proposition. Additionally admit only `QuantifiedFormula(SurfaceQuantifierKind::Universal)`: its parameter is constant in this versioned profile, so the fixed tag is unambiguous. Tokens admit only nonempty Identifier, ReservedWord and ReservedSymbol. Refuse every other node/token/recovery/unknown or parameter value; broader quantifiers require a separately versioned/reviewed profile, never an omitted variable payload.
Unsupported selection/subtree, missing child/node, unrepresentable length/count, duplicate key or missing theorem row returns whole-call `unsupported source theorem syntax payload`, without partial results. Require exactly one returned row for every actual Theorem entry. Existing earlier reconstruction/environment errors retain precedence; this optional profile changes no Parser/Checker/Core acceptance or diagnostics.
The root excludes an outer VisibleItem/private marker, other declarations and referenced definitions. Named package/module/namespace/visibility/role facts remain separate genuine checked-owner ingredients. Identical represented declarations across modules/private wrappers have identical value bytes and their own actual keys/owner facts; current SymbolId is association only. Local label/binder/proof-token spelling and topology participate without case/alpha/semantic normalization; formatting/comments/allocation/source ids/declaration placement do not.
These bytes establish neither semantic equivalence nor source freshness, complete SourceShapeHash/ObligationAnchor/dependencies, proof identity/acceptance, witness/reuse/cache/publication or Steps6–8 completion. Existing [registration syntax](#source-registration-syntax-payloads) and [public theorem signature](#source-public-theorem-signature-candidates) profiles remain separate and unchanged.
Append exactly `source_theorem_syntax_payloads_exact_current_keys_and_empty`, `source_theorem_syntax_payloads_neutral_relocation_order_and_local_edits` and `source_theorem_syntax_payloads_preserve_upstream_and_refuse_unsupported_subtrees` families in `crates/mizar-test/src/runner/tests.rs`, reusing unchanged `source_registration_owner_inputs`. The first uses genuine single/aggregate T/U inputs, exact actual current keys, repeatability, immutable inputs/results and validated empty/lemma-only sources. Before implementation independently derive complete literal expected grammar bytes from old parser/Syntax observations for `theorem T: for X being set holds X = X proof let X be set; thus X = X; end;`; U may use the one explicit framed identifier substitution, never producer traversal/output as oracle.
The second uses genuine whitespace/Unicode-comment, allocation relocation, declaration reordering, package/module and private-wrapper controls, comparing corresponding values by each actual named entry; local label/binder/proof-local token edits change bytes. Preserve passed source/env and prior results; never relabel containers or assert unrelated unpassed Frontend state.
The third compares genuine foreign/augmented/recovered source/env API errors to existing `validate_source_symbol_env`, reusing inherited matrices. A genuine existential-quantifier theorem must pass old whole-source validation yet refuse this profile; mix that existential theorem with supported T to prove whole-map refusal. A real parser-base AST changed to Existential is only a malformed negative, after real rebuilding and successful old validation. Every assertion reaches the API; no fabricated positive AST/owner/accepted/cache state or invented upstream outcome.
Implementation is one inline public function in existing `symbols.rs` (at most 130 added production lines) and the three appended families (at most 240 added test lines); preserve all old Rust/import/helper bytes. No new file/type/field/helper/adapter/parser/representation/dependency/hash/codec, owner producer, other Rust/spec/.miz/expectation/trace/ledger/lint/Cargo change or corpus activation.
