# mizar-checker: source type application projection

> canonical languageはEnglish。English canonical:
> [../en/source_type.md](../en/source_type.md)。

## Purpose And Authority

`source_type`は[`00.crate_plan.md`](./00.crate_plan.md)でfreezeしたTask 249
type head/application/argument producerを実装する。canonical authorityは
Chapters 03 §§3.2-3.3、05 §§5.2/5.6、07 §§7.2-7.3/7.6、08 §§8.1/8.3、
12 §§12.3/12.5/12.6.1/12.7、18 §§18.1/18.2.2、Appendix Aである。
bounded audit ownerはMC-G014、MC-G016、MC-G020である。

## Boundary And Model

moduleは`SurfaceAst`、`SurfaceNodeId`、`SyntaxKind`を受け取らない。
syntax-free `SourceTypeHandoffInput`はdense outer-application、
expression/head、ordered argument vectorを持つ。applicationはauthenticated
reserve/definition bindingをroot expressionへlinkする。expressionはwritten/
head site、range、spelling、recovery、form、builtinまたはresolver-authenticated
mode/structure headを保持する。argumentは`TermSite`、recursive `TypeSite`、
`QuaSite`だけであり、term/`qua` siteは`SemanticOrigin`を持つがselected
`BindingId`を持たない。

`SourceTypeProducer`はactual `BindingEnv`、`SymbolEnv`、`TypedArena`に対して
inputをauthenticateしてから`SourceTypeApplicationHandoff`をpublishする。
legacy reserve bridgeの`prepare_binding_env`はinput-only pathであり、symbol
headをvalidateしてreal binding environmentを構築するが、declaration checkや
type normalizationは実行しない。definition-parameter applicationはactual
resolver `DeclarationShell` ownerを要求し、generated contextをdeclarationとして
authenticateしない。

## Validation And Atomicity

cross-source/module、stale binding identity/order/type site、unsupported head
kind、stale contribution provenance、siteより後のlocal head、invisible imported
head、missing/out-of-closure import edge/target、invalid/duplicate typed site、
empty spelling、range/recovery mismatch、invalid `SemanticOrigin`をrejectする。
Term/`qua` provenanceはexact identifier range、current source/module、import
edgeなし、matching recovery、deterministic
`[parent-expression, argument-ordinal]` structural pathを要求する。

flat graphはdangling、cycle、multiple parent、forward parent、duplicate child、
wrong form、unreachable、non-contained、overlapping sibling/top-level relationを
rejectする。cycle/reachability checkはiterative worklistを使い、public flat
inputでcall stackを消費しない。inputをsort/repairせず、failure時はpartial
handoffをpublishしない。

全expression/head/term/`qua` siteはproducer時と`TypedAst` install時の両方で
actual typed-arena nodeへ照合する。owning nodeはsame-source rangeでnarrower row
rangeをcontainし、recoveryが完全一致しなければならない。これにより既存Task-248
item node上のdistinct role siteをarena変更なしで利用できる。

## Ownership, Consumers, And Exclusions

optional immutable handoffは`TypedAst`が所有する。`ResolvedTypedAst`はtyped AST
からcloneするだけで、separately replaceable resolved inputを持たない。handoff
absent時はconditional debug renderingによりlegacy byteを維持する。

broad real consumerはexact ten reserve written typeをtraverseし、application
10、expression/head 13、argument 6をpublishする。Task-248 routeは別にactual
checker-owned binding environmentを使い、`Bare`/builtin-`set` 2 rowとargument
0をco-installする。expansion、normalization、inhabitation、subtyping、
evidence、term/`qua` binding selection、fact、declaration/proof acceptance、
Core/CFG/VCはTask 249外である。

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceTypeApplicationForm` | `#[non_exhaustive]`。callerはlater source-written formを許容する。 |
| `SourceTypeHead` | `#[non_exhaustive]`。callerはlater authenticated head kindを許容する。 |
| `SourceTypeArgument` | `#[non_exhaustive]`。callerはlater syntax-free argument shapeを許容する。 |
| `SourceTypeError` | `#[non_exhaustive]`。callerはvalidation failureをexhaustive matchしない。 |

この module が所有する exhaustive public enum exception はない。

## Task 249 Classification

| Class | Result |
|---|---|
| `test_gap` | exact Task-249 handoffとTask-248 dependency consumerについてのみclose。 |
| `source_drift` | complete type-head/application/argument、final-handoff transport、import-closure authentication、real `DeclarationShell` ownershipをrepair。 |
| `design_drift` | paired component/plan/todo/audit/runner docsでrepair。 |
| `boundary_violation` | implementation reviewで検出したrecursive public-input graph traversalをiterative worklistへ置換。syntaxはrunner-owned、semantic result fabricationは禁止。 |
| `spec_gap` | bounded input-handoff sliceにはなし。 |
| `repo_metadata_conflict` | 観測なし。 |
