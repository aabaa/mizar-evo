# モジュール: declarations

> 正本は英語です。英語版: [../en/declarations.md](../en/declarations.md)。

状態: task R-011 は architecture 03 Step 3 の source-shaped declaration-shell
collector slice を実装する。これは表現済みの declaration-like `SurfaceAst` item、
visibility wrapper、recovered-shell state、export projection shell を記録する。
Step 3 全体はここでは完了しない。preliminary `SymbolId` assignment、label scope、
duplicate-label diagnostic、illegal-declaration diagnostic、kind-specific signature、
final export validation は、後続の name、label、symbol task に残る。

## 目的

declaration collector は、parser、checker、build crate に semantic decision を求めず、
後続 resolver phase に、表現済み source declaration の決定的 inventory を渡す。
collector は次を消費する:

- `SurfaceAst`。
- current canonical `ModuleId`。
- syntax-owned node range と recovery marker。

これは次を生成する:

- source-order の `DeclarationShell` entry を持つ `DeclarationShellSet`。
- 表現済み `ExportItem` node に対する `ExportProjectionShell` entry。
- declaration provenance（`SurfaceNodeId`、`SyntaxKind`、range）、export
  projection provenance（`SurfaceNodeId`、range）、resolver collection ordinal。

## 範囲

R-011 は意図的に source-shaped である。parser node kind を分類し、wrapper provenance
を保持してよいが、次は行わない:

- final `SymbolId`、overload slot、fully qualified name の割り当て。
- 型、本体、proof validity、registration semantics、algorithm contract の推論や検査。
- signature や theorem statement 内の name resolution。
- duplicate label や illegal declaration combination の検証。
- export target や private re-export legality の検証。

これらの decision は、後続 task の `names.md`、label resolution、signature collection、
または checker-owned fact を必要とする。

## shell を作る node

collector は、次の表現済み parser node kind に対して declaration shell を作る:

| Surface node kind | Shell kind | Notes |
|---|---|---|
| `PlaceholderItem` | `Placeholder` | 表現済み unresolved / recovered top-level item placeholder を保持する。 |
| `ReserveItem` | `Reserve` | source declaration inventory のみ。local-variable/type semantics は持たない。 |
| `TheoremItem` | `Theorem` | label extraction は後続 label work。 |
| `LemmaItem` | `Lemma` | label extraction は後続 label work。 |
| `DefinitionBlockItem` | `DefinitionBlock` | definition-local content の parent shell。 |
| `RegistrationBlockItem` | `RegistrationBlock` | registration-local content の parent shell。 |
| `ClaimBlockItem` | `ClaimBlock` | claim content は source-shaped のまま。 |
| `AttributeDefinition` | `AttributeDefinition` | type/signature checking は行わない。 |
| `PredicateDefinition` | `PredicateDefinition` | type/signature checking は行わない。 |
| `FunctorDefinition` | `FunctorDefinition` | type/signature checking は行わない。 |
| `ModeDefinition` | `ModeDefinition` | type/signature checking は行わない。 |
| `StructureDefinition` | `StructureDefinition` | field typing や inheritance validation は行わない。 |
| `AlgorithmDefinition` | `AlgorithmDefinition` | contract/body lowering は行わない。 |
| `AttributeRedefinition` | `AttributeRedefinition` | compatibility checking は行わない。 |
| `PredicateRedefinition` | `PredicateRedefinition` | compatibility checking は行わない。 |
| `FunctorRedefinition` | `FunctorRedefinition` | compatibility checking は行わない。 |
| `NotationAlias` | `NotationAlias` | original/alternate symbol resolution は行わない。 |
| `PropertyClause` | `PropertyClause` | proof validation は行わない。 |
| `StructureField` | `StructureField` | source shell のみ。 |
| `StructureProperty` | `StructureProperty` | source shell のみ。 |
| `InheritanceDefinition` | `InheritanceDefinition` | inheritance target validation は行わない。 |
| `FieldRedefinition` | `FieldRedefinition` | compatibility checking は行わない。 |
| `PropertyRedefinition` | `PropertyRedefinition` | compatibility checking は行わない。 |
| `ExistentialRegistration` | `ExistentialRegistration` | registration semantic checking は行わない。 |
| `ConditionalRegistration` | `ConditionalRegistration` | registration semantic checking は行わない。 |
| `FunctorialRegistration` | `FunctorialRegistration` | registration semantic checking は行わない。 |
| `ReductionRegistration` | `ReductionRegistration` | registration semantic checking は行わない。 |
| recovered `VisibleItem` without a represented target | `VisibilityWrapper` | malformed visibility syntax を捨てずに見える形で保持する。 |

`ExportItem` node は declaration shell ではなく `ExportProjectionShell` entry を作る。
各 projection は表現済み child `ModulePath` の spelling と range を保持するが、
resolve や validate はしない。

## 透明 wrapper と除外 node

次の node は shell collection では透明 wrapper である:

| Surface node kind | Collector behavior |
|---|---|
| `VisibleItem` | 明示的な `public` / `private` marker metadata を表現済み target shell に適用する。 |
| `AnnotatedDefinitionContent` | 自分自身の shell は作らず、含まれる definition shell を転送する。 |
| `AnnotatedRegistrationContent` | 自分自身の shell は作らず、含まれる registration shell を転送する。 |
| `AnnotatedStatement` / `AnnotatedAlgorithmStatement` | declaration-like node を含む場合は透明である。 |

次の node は R-011 では declaration shell を作らない:

| Category | Examples | Owner |
|---|---|---|
| import syntax | `ImportItem`, `ImportAliasDecl`, `ModuleBranchImport` | import resolution task |
| path syntax | `ModulePath`, `NamespacePath`, `QualifiedSymbol`, `PathSegment`, `RelativePrefix` | syntax/name task |
| context parameters | `DefinitionParameter`, `TemplateParameter`, `RegistrationParameter`, `AlgorithmParameters`, `TypedParameter` | 後続 signature/name task |
| patterns and bodies | `PredicatePattern`, `FunctorPattern`, `ModePattern`, `StructurePattern`, `NotationPattern`, definiens/case node | 後続 signature/checker task |
| correctness/proof details | `CorrectnessCondition`, `CoherenceCondition`, `ProofBlock`, justification/reference node | proof/checker task |
| statement-local inline definitions | `InlineFunctorDefinition`, `InlinePredicateDefinition` | 後続 local-name/signature/checker task |
| algorithm statements | `AlgorithmBody`, `AlgorithmStatementList`, `VariableDeclaration`, assignment/control statement node | algorithm/checker task |
| annotation-only nodes | `Annotation`, `LibraryAnnotation`, `StandaloneDiagnosticAnnotation` | syntax/trivia と後続 diagnostics |
| expressions and tokens | term/formula/type node、token、raw recovery node | syntax/name/checker task |

## visibility

visibility は R-011 では source-shaped のままである:

- 明示的な `public` / `private` marker は `DeclarationShellVisibility` になる。
- marker がない場合は `Unspecified` であり、semantic default はまだ適用しない。
- malformed / dangling visibility wrapper は recovered shell になる。
- default public/private behavior は、後続 phase が module interface を構築するときに決める。

## recovery

shell は次のいずれかを満たすと recovered として印付けされる:

- shell node 自体が recovered である。
- descendant に `ErrorRecovery` node または recovered token/node がある。
- shell の周囲の transparent wrapper に recovery がある。

この規則により、recoverable declaration は symbol identity を創作せず、後続 diagnostics
と navigation から見えるままになる。

## identity と provenance

`DeclarationShellId` と `ExportProjectionShellId` は、1 回の resolver run における
決定的な source-order collection id である。artifact-stable semantic identity ではない。
`SurfaceNodeId` は AST-local provenance としてのみ保持する。syntax documentation は、
これを永続 semantic anchor とはしていない。

後続 task は、関連仕様が揃ってから、current `ModuleId`、source order / structural
position、declaration kind、signature/name information から stable semantic identity を
導出する。

## 公開 enum の前方互換性

task R-026 は frontend task 25 の public-enum decision procedure をこの module に適用する。
`declarations` が所有する公開 resolver enum はすべて forward-compatible API surface
であり、`#[non_exhaustive]` を維持しなければならない:

- `DeclarationShellKind`
- `DeclarationShellVisibilityState`

この module は exhaustive な公開 enum 例外を所有しない。下流 consumer は wildcard
または fallback arm を持たなければならない。resolver 内部の match は、仕様化済みの
挙動を実装する範囲で、現在表現されている variant に対して exhaustive でよい。
