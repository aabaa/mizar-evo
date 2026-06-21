# Module: symbols

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
`OverloadIndex` failure metadata. The resolver does not select, rank, or
rewrite overload candidates.

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
or inline proof abbreviations. Artifact-backed summary reuse is task R-024.
Until then, symbols may consume source-backed or in-memory dependency
projections, but must not define a resolver-local artifact schema.

R-020 does not add a dedicated lexical-summary or artifact-summary data shape.
R-021 adds the resolver-local `ModuleLexicalSummaryIndex`; the parser-backed
extractor marks lexer-visible notation tokens (`UserSymbol` / `LexemeRun`) as
eligible, and collection seeds only export-visible, non-recovered projections.
Property keywords, algorithms, theorem labels, structure constructors, and
selectors do not seed active lexical summaries in R-021. Artifact-backed
`ModuleSummary` reuse remains R-024 work.

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
