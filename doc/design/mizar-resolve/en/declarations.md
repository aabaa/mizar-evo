# Module: declarations

> Canonical language: English. Japanese companion: [../ja/declarations.md](../ja/declarations.md).

Status: task R-011 implements the source-shaped declaration-shell collector
slice of architecture 03 Step 3. It records represented declaration-like
`SurfaceAst` items, visibility wrappers, recovered-shell state, and export
projection shells. It does not complete all of Step 3: preliminary `SymbolId`
assignment, label scopes, duplicate-label diagnostics, illegal-declaration
diagnostics, kind-specific signatures, and final export validation remain later
name, label, and symbol tasks.

## Purpose

The declaration collector gives later resolver phases a deterministic inventory
of represented source declarations without asking the parser, checker, or build
crates to make semantic decisions. The collector consumes:

- a `SurfaceAst`;
- the current canonical `ModuleId`;
- syntax-owned node ranges and recovery markers.

It produces:

- `DeclarationShellSet` with source-order `DeclarationShell` entries;
- `ExportProjectionShell` entries for represented `ExportItem` nodes;
- declaration provenance (`SurfaceNodeId`, `SyntaxKind`, ranges), export
  projection provenance (`SurfaceNodeId`, ranges), and resolver collection
  ordinals.

## Scope

R-011 is intentionally source-shaped. It may classify parser node kinds and
carry wrapper provenance, but it must not:

- assign final `SymbolId`s, overload slots, or fully qualified names;
- infer or check types, bodies, proof validity, registration semantics, or
  algorithm contracts;
- resolve names inside signatures or theorem statements;
- validate duplicate labels or illegal declaration combinations;
- validate export targets or private re-export legality.

Those decisions require `names.md`, label resolution, signature collection, or
checker-owned facts from later tasks.

## Included Shells

The collector creates declaration shells for these represented parser node
kinds:

| Surface node kind | Shell kind | Notes |
|---|---|---|
| `PlaceholderItem` | `Placeholder` | Preserves represented unresolved or recovered top-level item placeholders. |
| `ReserveItem` | `Reserve` | Source declaration inventory only; no local-variable/type semantics. |
| `TheoremItem` | `Theorem` | Label extraction is later label work. |
| `LemmaItem` | `Lemma` | Label extraction is later label work. |
| `DefinitionBlockItem` | `DefinitionBlock` | Parent shell for definition-local content. |
| `RegistrationBlockItem` | `RegistrationBlock` | Parent shell for registration-local content. |
| `ClaimBlockItem` | `ClaimBlock` | Claim contents remain source-shaped. |
| `AttributeDefinition` | `AttributeDefinition` | No type/signature checking. |
| `PredicateDefinition` | `PredicateDefinition` | No type/signature checking. |
| `FunctorDefinition` | `FunctorDefinition` | No type/signature checking. |
| `ModeDefinition` | `ModeDefinition` | No type/signature checking. |
| `StructureDefinition` | `StructureDefinition` | No field typing or inheritance validation. |
| `AlgorithmDefinition` | `AlgorithmDefinition` | No contract/body lowering. |
| `AttributeRedefinition` | `AttributeRedefinition` | No compatibility checking. |
| `PredicateRedefinition` | `PredicateRedefinition` | No compatibility checking. |
| `FunctorRedefinition` | `FunctorRedefinition` | No compatibility checking. |
| `NotationAlias` | `NotationAlias` | No original/alternate symbol resolution. |
| `PropertyClause` | `PropertyClause` | No proof validation. |
| `StructureField` | `StructureField` | Source shell only. |
| `StructureProperty` | `StructureProperty` | Source shell only. |
| `InheritanceDefinition` | `InheritanceDefinition` | No inheritance target validation. |
| `FieldRedefinition` | `FieldRedefinition` | No compatibility checking. |
| `PropertyRedefinition` | `PropertyRedefinition` | No compatibility checking. |
| `ExistentialRegistration` | `ExistentialRegistration` | No registration semantic checking. |
| `ConditionalRegistration` | `ConditionalRegistration` | No registration semantic checking. |
| `FunctorialRegistration` | `FunctorialRegistration` | No registration semantic checking. |
| `ReductionRegistration` | `ReductionRegistration` | No registration semantic checking. |
| recovered `VisibleItem` without a represented target | `VisibilityWrapper` | Keeps malformed visibility syntax visible instead of dropping it. |

`ExportItem` nodes produce `ExportProjectionShell` entries rather than
declaration shells. Each projection keeps represented child `ModulePath`
spellings and ranges, but does not resolve or validate them.

## Transparent And Excluded Nodes

These nodes are transparent wrappers for shell collection:

| Surface node kind | Collector behavior |
|---|---|
| `VisibleItem` | Applies explicit `public` / `private` marker metadata to the represented target shell. |
| `AnnotatedDefinitionContent` | Does not create its own shell; forwards contained definition shell(s). |
| `AnnotatedRegistrationContent` | Does not create its own shell; forwards contained registration shell(s). |
| `AnnotatedStatement` / `AnnotatedAlgorithmStatement` | Transparent when they contain declaration-like nodes. |

These nodes do not create declaration shells in R-011:

| Category | Examples | Owner |
|---|---|---|
| import syntax | `ImportItem`, `ImportAliasDecl`, `ModuleBranchImport` | import resolution tasks |
| path syntax | `ModulePath`, `NamespacePath`, `QualifiedSymbol`, `PathSegment`, `RelativePrefix` | syntax/name tasks |
| context parameters | `DefinitionParameter`, `TemplateParameter`, `RegistrationParameter`, `AlgorithmParameters`, `TypedParameter` | later signature/name tasks |
| patterns and bodies | `PredicatePattern`, `FunctorPattern`, `ModePattern`, `StructurePattern`, `NotationPattern`, definiens/case nodes | later signature/checker tasks |
| correctness/proof details | `CorrectnessCondition`, `CoherenceCondition`, `ProofBlock`, justification/reference nodes | proof/checker tasks |
| statement-local inline definitions | `InlineFunctorDefinition`, `InlinePredicateDefinition` | later local-name/signature/checker tasks |
| algorithm statements | `AlgorithmBody`, `AlgorithmStatementList`, `VariableDeclaration`, assignment/control statement nodes | algorithm/checker tasks |
| annotation-only nodes | `Annotation`, `LibraryAnnotation`, `StandaloneDiagnosticAnnotation` | syntax/trivia and later diagnostics |
| expressions and tokens | term/formula/type nodes, tokens, raw recovery nodes | syntax/name/checker tasks |

## Visibility

Visibility remains source-shaped in R-011:

- explicit `public` / `private` markers become `DeclarationShellVisibility`;
- missing markers are `Unspecified`, not semantically defaulted;
- malformed or dangling visibility wrappers become recovered shells;
- default public/private behavior is decided when later phases build the
  module interface.

## Recovery

A shell is marked recovered when any of the following is true:

- the shell node itself is recovered;
- any descendant contains an `ErrorRecovery` node or recovered token/node;
- a transparent wrapper around the shell contains recovery.

This rule keeps recoverable declarations visible to later diagnostics and
navigation without fabricating symbol identities.

## Identity And Provenance

`DeclarationShellId` and `ExportProjectionShellId` are deterministic
source-order collection ids for one resolver run. They are not artifact-stable
semantic identities. `SurfaceNodeId` is retained only as AST-local provenance;
syntax documentation explicitly does not make it a persistent semantic anchor.

Later tasks derive stable semantic identity from the current `ModuleId`,
source order/structural position, declaration kind, and signature/name
information once the relevant specs are in place.
