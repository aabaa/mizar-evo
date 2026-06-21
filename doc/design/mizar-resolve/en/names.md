# Module: names

> Canonical language: English. Japanese companion: [../ja/names.md](../ja/names.md).

Status: task R-012 specifies the resolver-owned name-resolution contract for
tasks R-013 through R-016. Task R-013 implements the namespace lookup slice:
resolved and unresolved import-alias records, reserved namespace roots,
package-name bindings, current-package fallback, and internal namespace
unresolved/ambiguous records. Task R-014 implements ordinary symbol-name lookup
over a preliminary `NameSymbolProjection` seam: qualified and unqualified
visibility, declaration-point filtering, current-module shadowing,
overload-group placeholders, enabled builtins, and deterministic `NameRefTable`
outcomes. Task R-015 implements root-aware internal name diagnostic records:
`NameDiagnosticRootId`, primary/cascade roles, unresolved import-alias roots,
namespace/name dependent records, stable candidate payloads, and deterministic
record ordering. Task R-016 implements source-shaped dot-chain finalization with
`LocalTermScope`, `LocalTermBinding`, `DotChainCandidate`, and
`DotChainFinalizer`: visible local term bindings shadow namespace heads and
produce `DeferredSelector`, while non-shadowed chains resolve their leading path
as a namespace and their final segment as a qualified symbol. The diagnostic code
range for public resolver diagnostics is still a known `spec_gap`; this
document therefore specifies diagnostic classes, payloads, and ordering, but no
public numeric diagnostic codes.

## References

This design derives the resolver-owned name contract from:

- architecture 03 Step 4 and "Namespaces Resolve Before Symbols";
- spec chapter 4 scope and shadowing for local variables and binders;
- spec chapter 11 scope, visibility, imports, and conflict rules;
- spec chapter 12 module namespaces, import/export placement, and visibility;
- spec chapter 13 selector-access syntax and selector-name restrictions;
- spec chapter 19 overload candidate construction and checker-owned winner
  selection;
- spec chapter 22 diagnostic payload requirements and the current resolver-code
  `spec_gap`;
- appendix A dot-role handoff between parser/syntax and resolver;
- resolver-local `resolved_ast.md`, `env.md`, `imports.md`, and
  `declarations.md`.

## Purpose

The names phase resolves source name-use sites after imports and declaration
shells exist, but before type checking, proof checking, overload winner
selection, or selector type checking. It consumes source-shaped syntax and
resolver-owned indexes, then records explicit name outcomes in `ResolvedAst`.

Inputs:

- `SurfaceAst` for the current module;
- resolved import graph, resolved alias bindings, and unresolved import-alias
  dependency records from `imports.md`;
- declaration shells from `declarations.md`;
- preliminary shell-derived symbol identities for current-module declarations;
- dependency symbol/namespace projections from source-backed fixtures or
  summaries when available, refined by `symbols.md` later;
- label scopes only for namespace prefixes used by qualified citations. Label
  resolution itself is specified by `labels.md`.

Outputs:

- namespace lookup results used by qualified name resolution;
- `NameRefTable` entries for resolver-attempted ordinary name-use sites;
- explicit unresolved and ambiguous symbol-name records;
- internal namespace unresolved/ambiguous records used before final symbol
  lookup;
- `DeferredSelector` records for dotted syntax whose selector decision belongs
  to checker/type information;
- internal resolver diagnostic records with deterministic ordering.

## Boundary

The names phase may:

- resolve module namespaces, import aliases, namespace prefixes, constructor
  names, user-symbol spellings, and builtin prelude names;
- build deterministic candidate sets for later overload and checker phases;
- decide name-level visibility, shadowing, unresolved references, and
  name-level ambiguity.

It must not:

- select a type-directed overload winner;
- rank candidates by inferred argument types;
- fire clusters, insert coercions, or decide `qua` validity;
- type-check selectors or structure fields;
- validate proof labels beyond namespace prefixes;
- invent public user-facing diagnostic codes.

## Symbol Identity Handoff

R-011 declaration shells intentionally do not assign final `SymbolId`s. Name
tasks nevertheless must not populate `NameRefTable` with raw strings. Before
R-014 records resolved ordinary references, the resolver must derive a
preliminary symbol-identity projection from declaration shells and imported
summary entries:

- the declaring `ModuleId`;
- declaration kind family;
- deterministic declaration ordinal or structural path from the shell;
- source spelling or notation slot when available without type checking;
- an overload/relation placeholder slot when multiple same-spelling
  declarations can coexist but `symbols.md` has not yet assigned final slots.

This projection must produce the same `SymbolId` shape required by
`resolved_ast.md`, but it remains a resolver identity for name references, not a
complete `SymbolEnv` entry. R-014 exposes this seam as `NameSymbolProjection`.
R-019 through R-021 refine the same identities with kind-specific signatures,
overload families, relation links, and exported summary data. If a declaration
cannot be represented as a semantic declaration, the resolver records an
unresolved or ambiguous result instead of fabricating a `SymbolId`.

## Name-Use Sites

The resolver attempts name lookup for represented syntax nodes whose spelling
can denote a namespace, declaration, builtin, or selector boundary:

| Surface node or role | Name-phase behavior |
|---|---|
| `NamespacePath` | Resolve every segment to a module namespace, import alias, or namespace child. |
| `QualifiedSymbol` | Resolve leading segments as namespace path, then resolve the final symbol/constructor spelling inside the target namespace. |
| declaration references inside signatures or statements | Resolve only the referenced spelling and namespace; defer type meaning. |
| selector-looking dotted chains | Decide only the namespace-vs-local-term boundary; defer selector type checking. |
| builtin prelude spelling | Resolve as `ResolvedBuiltin` when enabled by the edition and not shadowed by an earlier semantic scope. |

Parser recovery nodes and malformed path segments are retained as unresolved
records instead of being dropped.

## Scope Model

Semantic lookup considers these tiers in order:

1. local proof, block, or statement bindings;
2. current definition/theorem/template parameters;
3. current-module declarations visible at the use site;
4. explicitly imported public symbols and namespaces;
5. re-exported public symbols made visible through imported facade modules;
6. builtin prelude symbols enabled by the active edition.

Current-module declarations become visible only after the declaration item is
complete. Forward references to later declarations are not permitted by the
language specification. Imported summaries seed the base lexical environment,
but semantic lookup still validates imports and visibility before a reference is
committed.

Local bindings shadow namespace components. If a leading segment is in scope as
a local variable or parameter, a following dot is treated as selector syntax,
not as a namespace separator. The resolver records `DeferredSelector` when it
can identify the term base but the selector field/property decision requires
type information.

Overloadable declaration spellings are collected as deterministic candidate
sets. The names phase may filter by namespace, declaration point, visibility,
symbol family, and syntactic arity when those facts are available without type
checking. It does not choose the overload winner and does not report
checker-owned overload ambiguity. A valid same-spelling overload set is ordered
and passed downstream as candidates or as one resolver-visible overload group.
Only namespace/import/visibility conflicts or candidate sets that cannot be
represented as one overload-capable group remain resolver-owned `Ambiguous`
results.

## Namespace-Before-Symbol Resolution

Qualified references are resolved in two layers:

1. Resolve the leading segments as a namespace path.
2. Resolve the final spelling inside the target namespace's exported symbol
   table or, for the current module, the current-module visible table.

The namespace layer never falls back to symbol lookup after a segment has
failed. A failed segment produces one unresolved namespace result anchored at
that segment range, and dependent symbol lookup records the failed namespace
root instead of fabricating a target.

Alias spelling is local provenance only. Resolved identities use canonical
`ModuleId` and `SymbolId` values, not import aliases.

## Namespace Lookup Precedence

Task R-013 starts after a leading namespace candidate has been collected. Local
term binding shadowing is still the R-016 dot-chain layer; when a future
dot-chain pass proves that the first segment is a local term, namespace lookup
is not attempted for that segment.

Namespace candidates are resolved in this order:

1. Resolved import aliases are considered first. Import resolution rejects
   aliases that collide with reserved roots, so a resolver-visible resolved
   alias is already alias-shaped provenance.
2. Unresolved import aliases are considered before reserved roots and package
   bindings. A namespace candidate that depends on such an alias records an
   internal `UnresolvedImportAlias` dependency, or `AmbiguousImportAlias` when
   duplicate import records retain multiple canonical targets; it does not fall
   through to root/package/current-package lookup.
3. Reserved namespace roots (`std`, `pub`, `pkg`, `dev`, `ext`) are matched
   through the longest root binding. If no binding exists, the first segment
   after the root, or the root itself for an empty suffix, is the failing
   segment.
4. Package-name namespace bindings use longest-prefix matching and take
   precedence over current-package fallback.
5. Current-package fallback is attempted only when no import alias, unresolved
   import alias, reserved root, or package-name binding matched the first
   segment.
6. After a package has been selected, the remaining segments must name an
   indexed module. Missing module lookup reports the earliest segment whose
   prefix has no module in the selected package. Stale namespace/package
   provider state is a crate-local `ProviderError`, not a user-facing module
   miss.

## Visibility And Shadowing

Visibility is interpreted at the use site:

- current-module code may see both public and private current-module
  declarations after their declaration points;
- importers may see only public declarations and legal public re-exports;
- private dependency symbols are inaccessible even if their spellings were
  present in a provisional lexical environment;
- a private symbol can participate in current-module lookup but must not be
  serialized into dependency-facing exported symbol tables.

Shadowing is source- and tier-sensitive:

- local variables and parameters shadow namespace prefixes and ordinary symbols
  at the same spelling;
- current-module declarations with the same visible spelling are considered
  before imported declarations, but overload candidate construction may still
  retain imported candidates when later overload rules allow them to
  participate;
- two incompatible imported candidates with no qualification or overload-family
  relationship become a deterministic ambiguity rather than an arbitrary pick;
- qualification by module path or alias restricts lookup to the qualified
  namespace and suppresses unrelated unqualified candidates.

## Unresolved And Ambiguous References

Every failed or ambiguous ordinary symbol lookup is explicit in `NameRefTable`.
Namespace failures that occur before a final symbol spelling is reached are
explicit internal namespace records linked from the dependent name reference or
diagnostic. Task R-015 keeps `NameRefTable` as the source of truth for name
outcomes and adds a crate-local diagnostic report that assigns deterministic
`NameDiagnosticRootId` values for unresolved/ambiguous name references,
unresolved namespace paths, unresolved import-alias dependencies, and ambiguous
namespace/import-alias roots. This matches the current `NameRefTable` shape,
whose ambiguous candidate payload is a deterministic list of `SymbolId`
candidates.

Required unresolved symbol-name classes:

- missing symbol in a resolved namespace;
- inaccessible private symbol;
- recovered or malformed final spelling;
- builtin disabled by edition after higher semantic lookup tiers fail.

A builtin shadowed by an earlier semantic scope resolves to that earlier
binding; it is not an unresolved builtin.

Required internal namespace unresolved classes:

- unknown namespace segment;
- unresolved import or alias dependency;
- stale module-index provider state;
- recovered or malformed namespace segment.

Required ambiguous classes:

- multiple visible symbols that cannot be represented as one overload-capable
  group;
- incompatible imported candidates that require qualification;
- recovered syntax whose surviving candidates cannot be ranked without
  inventing semantics.

Required internal namespace ambiguity classes:

- multiple namespace bindings for the same alias or segment;
- import alias collisions that prevent a unique namespace target;
- recovered namespace syntax whose surviving namespace candidates cannot be
  ranked without inventing semantics.

Checker-owned overload ambiguity is not a names-phase ambiguity. If all
candidates are valid members of one overload-capable set, the resolver records a
deterministic candidate set or preliminary group identity and leaves viability,
specificity, and best-root selection to later type/overload phases.

`NameRefTable` ambiguous symbol candidate lists follow `resolved_ast.md`:
canonical fully qualified name, then module id, then source range, then local
symbol id. Internal namespace candidate lists use canonical module id, namespace
path, source range, and stable variant name. The ordering must not depend on
hash-map iteration or filesystem order.

One unresolved root should produce one primary resolver diagnostic. Dependent
nodes link to the root unresolved key and should not emit cascaded primary
diagnostics unless they add a distinct cause. When a namespace failure is caused
by an unresolved import alias, the import dependency root is the primary record;
dependent namespace records and failed-namespace `NameRefTable` entries link to
that root as cascade records. Recovered namespace records and recovered
reference origins are retained in resolver tables but omitted from internal
name diagnostic roots and cascades.

## Dot-Chain Finalization

The parser and syntax crates keep dot roles source-shaped. Task R-016 finalizes
the resolver-owned semantic boundary for chains that can be either namespace
qualification or selector access.

Implementation contract:

- local term bindings carry a stable lexical `LocalTermScope`, declaration
  range, and visible-after ordinal. A binding is in scope when the use-site
  scope is equal to or nested under the binding scope and the binding is visible
  before the chain ordinal. When several bindings with the same spelling are in
  scope, the innermost scope wins, then latest visible-after ordinal, then
  declaration range;
- if the first segment resolves to an in-scope local term binding, the chain is
  selector syntax and namespace lookup is not attempted for that segment. The
  resolver records `DeferredSelector` with the use-site base term node from
  `DotChainCandidate`, not the binding declaration node;
- if the first segment resolves to an import alias, namespace root, package
  binding, or current-module namespace and no local binding shadows it, the
  resolver treats the leading chain as namespace-qualified until the final
  symbol segment;
- when the resolver can identify the term base but needs type information to
  validate the selector, it records `DeferredSelector`;
- when both namespace and selector interpretations are impossible, the failure
  is unresolved at the earliest decisive segment;
- malformed, recovered, or single-segment dotted-chain candidates are retained
  as unresolved internal records without inventing parser or checker semantics.

This finalizer records the handoff. It does not perform selector field lookup,
selector-call type checking, type-directed overload winner selection, or parser
recovery decisions.

## Diagnostics

Until the external diagnostic specification allocates resolver code ranges,
name diagnostics remain crate-local/internal records. They must include:

- primary source range at the failing use site or segment;
- secondary ranges for relevant imports, declarations, or candidates;
- failure class or ambiguity class;
- attempted spelling and normalized namespace prefix;
- deterministic candidate keys for ambiguous lookups;
- root unresolved key for cascade suppression;
- primary/cascade role for each internal record.

Diagnostic ordering uses root primary range, primary-before-cascade role,
failure/ambiguity class name, attempted spelling, stable candidate key, and
record-local range/name-reference tie-breakers. Public numeric codes must be
added only after `doc/spec/en/22.error_handling_and_diagnostics.md` assigns or
delegates resolver code ownership.

## Public Enum Forward-Compatibility

Task R-026 applies the frontend task-25 public-enum decision procedure to this
module. All public resolver-owned enums in `names` are forward-compatible API
surfaces and must remain `#[non_exhaustive]`:

- `NamespaceResolutionOrigin`
- `NamespaceFailureClass`
- `NamespacePartialOrigin`
- `NameProjectionSource`
- `NameReferenceScope`
- `NameDiagnosticRole`
- `NameDiagnosticKind`

No exhaustive public enum exceptions are owned by this module. Downstream
consumers must keep wildcard or fallback arms; resolver-internal matches may
remain exhaustive over the currently represented variants when implementing the
specified behavior.

## Tests Planned For Implementation Tasks

R-012 adds no tests because it changes documentation only. Follow-on tasks must
cover:

- namespace segment lookup and missing-segment ranges (R-013);
- qualified and unqualified visibility, shadowing, and private access (R-014);
- unresolved/ambiguous candidate ordering and cascade suppression (R-015);
- selector-vs-namespace dot-chain finalization (R-016).
