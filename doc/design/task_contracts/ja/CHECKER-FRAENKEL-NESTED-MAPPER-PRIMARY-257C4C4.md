# Task CHECKER-FRAENKEL-NESTED-MAPPER-PRIMARY-257C4C4: nested Fraenkel mapper primary

> canonical English:
> [../en/CHECKER-FRAENKEL-NESTED-MAPPER-PRIMARY-257C4C4.md](../en/CHECKER-FRAENKEL-NESTED-MAPPER-PRIMARY-257C4C4.md)。
> 本書はlogical synchronized Japanese companionである。

Owning planは[mizar-checker](../../mizar-checker/ja/00.crate_plan.md#task-index)と
[mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。durable ownerは
[source term](../../mizar-checker/ja/source_term.md#task-257c4c4-nested-fraenkel-mapper-primary)と
[harness](../../mizar-test/ja/harness.md#checker-task-257c4c4-private-nested-mapper-primary-probe)。

## Status、目的、readiness

**Status:** documentation prerequisite freeze済み、implementation pending。

Clean `HEAD 5578f7e51f5acfb60494dbacb41640b976c9c55c`のfresh read-only
inventoryはcompleted C4C3のdependency-minimal successorとして本taskをselectする。
Chapter 13と人間のsemantic confirmationはinner mapper `x@94..95`をdistinctなouter
generator `x@136..137`のuseとfixする。C4C2はresolver relation、C4C3はexact checker
typed sitesまでauthenticating済み。本taskはC4C3-gated transactionでmissing Task-252
primary occurrenceとbinding referenceだけを追加する。

`spec_gap`はない。missing specialized forward-written Fraenkel profileは
`design_drift`、checker/imported-fixture regression不在は`test_gap`、freeze後のsource
不在は`source_drift`。exact-F5 C4A/C4B reuse、generic Task252 declaration-order lookupの
relax、resolver binding ID `1`をchecker `BindingId(1)`とみなすこと、capture/semantic
state追加は`boundary_violation`。

選択するintermediate splitはdependency-minimal、zero-semantic、default-deny。
両generator binder投影、outer bindingをinner owner contextへ置く案、別public binding
handoff、global Task252 rule変更は、unused surface/誤owner/global behavior変更を増やすため
rejectする。

## Authority、dependency、protected artifact

Authority orderはcanonical Chapter 13 §§13.4.2/13.4.4/13.8.6、既存
`pass_types_nested_comprehension_outer_generator_capture_001.miz`、sole trace row、inactive
sidecar、completed C4C2/C4C3、derived design/source inventory。

Source、inactive sidecar、trace SHA-256は順に
`c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431`、
`9ed000a30c1d519bd665f338c636fb9e529e9848a285209bebe6728f19961b92`、
`d4d817e83aac78d19e729702b26c62604fc57581eec18672a5c26ec44efe7a81`のまま。
Sidecarはinactive `advanced_semantics`、`pass/type_check`、diagnosticなし。traceは
test-intent-onlyでexecution/semantic creditを付与しない。

Sole dependencyはcomplete internal-valid C4C3 handoff: row 1件、resolver use `0`、
resolver binding `1`、outer typed binder `x@136..137`、inner mapper use `x@94..95`、
source ordinal `0`。Producerはcrate-private C4C3 validationでretained resolver/typed
snapshotを再認証してからTask252をprojectする。C4A/C4Bはnegative compatibility
profileでdependencyではない。

## Exact lower projection

`BindingEnv`はexact 3 context / 1 binding / 0 diagnostics:

| Context | Owner | Parent/layer | Bindings | Visible |
|---:|---|---|---|---|
| `0` | `Module` | none / `Module` | `[]` | `[]` |
| `1` | `SourceComprehension { 90..157 }` | `0` / `Expression` | `[0]` | `[0]` |
| `2` | `SourceComprehension { 92..123 }` | `1` / `Expression` | `[]` | `[0]` |

全contextはlexical scopeなし/Normal。Checker binding `0`は`x`、
`QuantifierBinder`、`SourceBound { context: 1, ordinal: 0 }`、owner `1`、declaration
`136..137`、visible-after `0`、type `Source(141..155)`、`Active`、Normal、capturedと
diagnosticsはempty。これはprojection-local IDでresolver binding ID `1`ではない。

Mapper queryはcontext `2`、scope/resolver fallbackなし、`x`、special logical ordinal
`1`。Ordinal `0`ではexact `ForwardReference([0])`、ordinal `1`では`Local(0)`。
これはgenerator visibility用profile coordinateでsource byte/C4C2 use/capture ordinal
ではない。

Private Task252 projection arenaはroot node `0`だけ: kind
`source.term.variable-reference`、`94..95`、resolved node/childrenなし、Unknown、Normal、
empty links。C4C3 real typed `Identifier`に対して認証されたsyntax-free projectionであり、
replacement `TypedAst`/installation/node-ID同一性ではない。

Existing `SourcePrimaryTermHandoff`はexact `1/1/0`: term 0 = node0、`94..95`、ordinal0、
context2、Normal、`x`、`VariableReference`/`Value`、parentなし。reference0 = term0から
checker binding0、`Variable`、scopeなし、use ordinal1。numeric requestはempty。
Inner `y`はC4C3でdistinct resolver binding0として認証するだけで本projectionへ複製しない。

## Frozen API、validation、default deny

Sole production ownerは`crates/mizar-checker/src/source_term.rs`。Exact public family:

```rust
SourceNestedFraenkelMapperPrimaryHandoff
#[non_exhaustive] SourceNestedFraenkelMapperPrimaryError
SourceNestedFraenkelMapperPrimaryProducer
```

Handoff getterはexact `source_id`、`module_id`、`dependency`、
`dependency_fingerprint`、`binding_env`、`binding_fingerprint`、
`projection_arena`、`source_term`、`source_term_fingerprint`、`debug_text`。
Producerはexact:

```rust
SourceNestedFraenkelMapperPrimaryProducer::build(
    dependency: SourceNestedFraenkelBinderUseHandoff,
) -> Result<SourceNestedFraenkelMapperPrimaryHandoff,
          SourceNestedFraenkelMapperPrimaryError>
```

Dependencyはby-valueでretainする。Unchecked/mutable/alternate input/profile selector/
installation/`Default`は追加しない。C4C3はcrate-private complete-validation entryだけを
追加し、retained resolver/TypedAst getterは公開しない。

Exact debug grammar:

```text
source-nested-fraenkel-mapper-primary-debug-v1
module: <package>::<path>
dependency-fingerprint: <Debug quoted complete C4C3 debug text>
binding-fingerprint: <Debug quoted complete BindingEnv debug text>
projection: nodes=1 root=0
source-term-fingerprint: <Debug quoted complete source-primary-term debug text>
```

Non-exhaustive errorはexact 3 variants/precedence: `InvalidDependency`
(`nested Fraenkel mapper-primary dependency is invalid`) →
`InvalidBindingEnvironment` (`nested Fraenkel mapper-primary binding environment is invalid`) →
`InvalidSourceTerm` (`nested Fraenkel mapper-primary source term is invalid`)。
Dependency corruption、context/binding/lookup corruption、arena/term/ref/request corruptionの
順でfail closedし、partial handoffをpublishしない。

Private `SourcePrimaryTermBindingProfile`にexact nested caseだけを加えlogical ordinal1と
authenticated textual-forward outer binderを許可する。Public generic build、他profile、
generic declaration-derived ordinal/forward rejection/role/error/installed transactionは不変。

## Scope、tests、audit impact

Rust scopeは`source_formula_composition.rs`のcrate-private validation seam、
`source_term.rs`のowner/tests、existing private
`fraenkel_nested_capture_identity.rs`のsole consumerだけ。

Checker testsはexact 4件:

1. `task257c4c4_builds_exact_nested_mapper_primary_handoff`;
2. `task257c4c4_rejects_dependency_and_binding_projection_corruption`;
3. `task257c4c4_rejects_arena_term_reference_and_precedence_corruption`;
4. `task257c4c4_replays_deterministically_and_preserves_generic_task252_rejection`。

Private testはexact
`task257c4c4_real_imported_fixture_builds_mapper_primary_handoff`。Dependency、context/
binding/query、arena、term/reference/request全field/cardinality/order、fingerprint/error
precedence、generic forward/raw Identifier rejection、deterministic replayをcoverする。

Baselineは`source_term.rs` `6451` lines /
`e6f96b3fd83c77c06689d53e7efc6ddae27c744d5ffed79019ced2d2104d4602`、
`source_formula_composition.rs` `9358` /
`eed8c480a2ddeceafd529ee4c37c333f6e36f8f23e62f4b53f782bc9df651b6c`、private leaf
`248` / `46bb3e63199d4b9794a9d56c214d76864a073cc35b0643ec64a8a1e412d5bb0a`。
Raw test listはchecker `558 -> 562`、mizar-test `620 -> 621`、baseline hashは
`aa1eccf5bd93c9574082f7c888918ccb2bbc76167aa5ef0c672a6db931e42d8f`と
`95ff9e007bd474cad657e626f61424db408ec343f6f1a6c1b84d6fff50ee9a75`。
Contract treeは`93/93 -> 94/94`。Final source measurementはimplementation後ここで1回ownする。

`spec_coverage_audit.md`はdurable zero-credit Task252 mapping/consumerを1件追加する。
Spec、`.miz`、expectation、trace、active route、diagnostic、semantic result、coverage creditは不変。

## 禁止境界、review、exit

Captured identity、semantic capture、generated-core parameter、type/sethood answer、request/
result、verdict、diagnostic、proof/fact、Typed/Resolved installation、production dispatch、
runner/registry/sidecar/trace activation、coverage credit、Task277B stateを追加しない。Binding
fieldのcapturedは構造上exact empty。Task277Bはnot-ready/zero-credit。

Spec/contract、test sufficiency、implementation、source/docs/API、bilingual/boundary、final
qualityの独立reviewを全てNO FINDINGSまで反復し、focused/package/lint/metadata/fmt/workspace
Clippy/full tests/unchanged CLI/protected hash/scope/staging/commit/postcommitを通す。Exitは
`9/9` hard gates、quality `>=90/100`、task-only commit、clean/stash-invariant proof、fresh
inventory。本runはuser指示により本task完了後にstopし、successorがreadyでもimplementしない。

Routingはparent GPT-5.6 Sol `xhigh`、frozen implementation/reviewはTerra `high/xhigh`。

## Completion evidence

Independent specification/APIとbilingual/owner/boundary reviewは**NO FINDINGS**。
`git diff --check`、checker lint-policy `15/15`、mizar-test lint-policy `15/15`、metadata
`137/137`がpass。Contract treesは`94/94`、protected/baseline hashは全てexactにreproduce。
Documentation-prerequisite staging/commitはpendingで、implementationとそのreview/hard
gateは未開始。
