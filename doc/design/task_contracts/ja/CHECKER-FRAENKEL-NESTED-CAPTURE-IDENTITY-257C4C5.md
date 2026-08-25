# Task CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-257C4C5: nested Fraenkel capture-identity receipt

> canonical English:
> [../en/CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-257C4C5.md](../en/CHECKER-FRAENKEL-NESTED-CAPTURE-IDENTITY-257C4C5.md).

Owning plan: [mizar-checker](../../mizar-checker/ja/00.crate_plan.md#task-index)と
[mizar-test](../../mizar-test/ja/00.crate_plan.md#task-index)。Durable ownerはchecker
[source formula composition](../../mizar-checker/ja/source_formula_composition.md#task-257c4c5-nested-fraenkel-capture-identity-receipt)とtest
[harness](../../mizar-test/ja/harness.md#checker-task-257c4c5-private-capture-identity-receipt-probe)。

## Status、決定、目的

**Status:** complete。

Independent specification/equivalence、bilingual/boundary、implementation、
test-sufficiency、source/documentation/API、final-quality reviewはrecorded repair後
no findings。

人間の決定により、completed C4C4後の最初のcapture-identity receiptのsole ownerを
existing Task257C `source_formula_composition` familyに固定する。本taskはexact resolved
identity association 1件だけを運び、captureの意味論判断もinstallationも行わない。

- complete C4C4 handoffをby-value consume/retainする。
- exact inner comprehension、mapper primary/reference、projection-local checker binding、
  resolver use/bindingをimmutable row 1件でassociateする。
- exact C4C4 profileだけをacceptし、ordering claimはexact source ordinalだけ。
- destinationはstandalone Task257C handoffであり、`TypedAst`、`ResolvedTypedAst`、
  `CoreIr`、`GeneratedOrigin`ではない。
- corruptionはsort/inference/repair/caller-selected profileなしでatomicにrejectする。

`spec_gap`はない。未確定だったunique owner/API/oracleは`design_drift`、exact regressionの
欠落は`test_gap`、capture installation/semantic result/Core loweringは
`boundary_violation`である。

## Authorityと保護する意味

Authority orderは次のまま。

1. canonical [Chapter 13 §§13.4.2, 13.4.4, 13.8.6](../../../spec/en/13.term_expression.md#134-set-expressions)
2. existing [`pass_types_nested_comprehension_outer_generator_capture_001.miz`](../../../../tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.miz)
3. sole [trace row](../../../../tests/coverage/spec_trace.toml)
4. inactive [expectation](../../../../tests/miz/pass/types/pass_types_nested_comprehension_outer_generator_capture_001.expect.toml)
5. completed [C4C2](RESOLVE-FRAENKEL-NESTED-CAPTURE-257C4C2.md)、
   [C4C3](CHECKER-FRAENKEL-NESTED-BINDER-USE-257C4C3.md)、
   [C4C4](CHECKER-FRAENKEL-NESTED-MAPPER-PRIMARY-257C4C4.md)、その後にderived owner docs/source inventory

Frozen meaning:

- inner mapper `x@94..95`はouter generator `x@136..137`のresolved binding identityを参照する。
- inner generator `y@102..103`はinner comprehension localでcaptureしない。
- associationはdisplay spellingや異なるdomainのnumeric ID coincidenceでなくresolved binding identityに基づく。
- C4C4 outer-x projectionはby-valueのままで、sole `BindingEntry::captured`はempty。
- Task277Bはnot-ready/zero execution and semantic creditのまま。

Protected `.miz`/expectation/trace SHA-256は順に
`c3b8bd62c16406ccedee2e64a71ef62a5c4b329d2319be33ad3834a9541af431`、
`9ed000a30c1d519bd665f338c636fb9e529e9848a285209bebe6728f19961b92`、
`d4d817e83aac78d19e729702b26c62604fc57581eec18672a5c26ec44efe7a81`で、byte-identicalを維持する。

## Sole ownerとdependency boundary

Sole production ownerは`crates/mizar-checker/src/source_formula_composition.rs`。
これはTask257C cross-family associationであり、新Task252 occurrence ownerでもTask255 set-term ownerでもない。

Sole lower dependencyはcomplete internally-valid
`SourceNestedFraenkelMapperPrimaryHandoff` 1件。C4C4はexisting complete validatorへ
delegateするcrate-private `validate_complete()` seam 1件だけを追加し、mutable stateやnew public getterは公開しない。
C4C5 producerはdependencyをby-value consumeし、associationを読む前にvalidateし、immutable retainする。

Task255はdependencyではない。Current admitted comprehension profileはnested
`Element of NAT` outer-to-inner resolved identity relationをownしない。Task255 admissionをwidenせず、Task255 termをsynthesizeしない。

## Exact associationとordering

Tableはexact row ID `0` 1件。

| Field | Exact value | Authenticated source |
|---|---:|---|
| owner context | checker `BindingContextId(2)` | C4C4 inner-comprehension context |
| owner range | `92..123` | C4C4 `SourceComprehension` owner |
| mapper term | `SourcePrimaryTermId(0)` | C4C4 mapper `x@94..95` |
| mapper reference | `SourcePrimaryTermReferenceId(0)` | C4C4 outer-x reference |
| projected binding | checker `BindingId(0)` | C4C4 outer-x by-value projection |
| resolver use index | `0` | retained C4C3 mapper use |
| resolver binding | `FraenkelGeneratorVariableBindingId(1)` | retained C4C3 outer generator identity |
| source ordinal | `0` | exact C4C3 association order |

Identity ID/checker binding IDはlocal dense IDでresolver binding ID `1`と同一ではない。
Cross-domain evidenceはresolver binding objectであり、spelling `x`はjoin keyではない。

Exactly-oneとはrow `0`あり、row `1`なし、len 1、iterationが`(0,row0)`だけをyieldすること。
missing/extra/duplicate/reorderedはinvalid。`source_ordinal == 0`だけがordering claimで、
general capture order/Core generated-parameter orderは定義しない。

## Frozen public APIとdestination

Existing public moduleへの追加はexactly:

```rust
SourceNestedFraenkelCaptureIdentityId
SourceNestedFraenkelCaptureIdentity
SourceNestedFraenkelCaptureIdentityTable
SourceNestedFraenkelCaptureIdentityHandoff
#[non_exhaustive] SourceNestedFraenkelCaptureIdentityError
SourceNestedFraenkelCaptureIdentityProducer
```

Dense IDはprivate storageで`Debug`/`Clone`/`Copy`/`PartialEq`/`Eq`/
`PartialOrd`/`Ord`/`Hash`をderiveし、exact
`new(index: usize) -> Self`と`index(self) -> usize`だけを公開する。

Row getterはexact:

```rust
owner_context() -> BindingContextId
owner_range() -> SourceRange
mapper_term() -> SourcePrimaryTermId
mapper_reference() -> SourcePrimaryTermReferenceId
projected_binding() -> BindingId
resolver_use_index() -> usize
resolver_binding() -> FraenkelGeneratorVariableBindingId
source_ordinal() -> usize
```

Tableはexact:

```rust
get(
    &self,
    id: SourceNestedFraenkelCaptureIdentityId,
) -> Option<&SourceNestedFraenkelCaptureIdentity>
iter(
    &self,
) -> impl Iterator<
    Item = (
        SourceNestedFraenkelCaptureIdentityId,
        &SourceNestedFraenkelCaptureIdentity,
    ),
>
len(&self) -> usize
is_empty(&self) -> bool
```

Iterationはdense/source-ordinal order。Handoff getterはexact:

```rust
source_id() -> SourceId
module_id() -> &ModuleId
dependency() -> &SourceNestedFraenkelMapperPrimaryHandoff
dependency_fingerprint() -> &str
identities() -> &SourceNestedFraenkelCaptureIdentityTable
debug_text() -> String
```

Producer signature:

```rust
SourceNestedFraenkelCaptureIdentityProducer::build(
    dependency: SourceNestedFraenkelMapperPrimaryHandoff,
) -> Result<SourceNestedFraenkelCaptureIdentityHandoff,
          SourceNestedFraenkelCaptureIdentityError>
```

Frozen dense ID `new`を除き、public/raw/unchecked row/table/handoff constructor、mutable
accessor、caller DTO、profile selector、`Default`、installer、adapter、conversionは追加しない。
このhandoffがcomplete current destination。
`TypedAst`/`ResolvedTypedAst`にfield/method/installation surfaceを追加しない。

Exact debug grammar:

```text
source-nested-fraenkel-capture-identity-v1|module=<package>.<path>|identities=1|dependency-fingerprint=<Debug quoted complete C4C4 debug text>
```

## Default-deny oracle

Handoffはprivate complete validatorと、later owner用crate-private `validate_complete()`
boundary 1件を持つ。後者はstateをexposeせずcrate外からaccessできない。

Non-exhaustive errorはexactly:

```rust
InvalidDependency
InvalidCaptureIdentity {
    capture_identity: SourceNestedFraenkelCaptureIdentityId,
}
```

Displayは順に
`nested Fraenkel capture-identity dependency is invalid`、
`nested Fraenkel capture identity <id> is invalid`。

最初にcomplete retained C4C4 dependency、source/module/fingerprintをreauthenticateし、
失敗は`InvalidDependency`。次にexact table cardinality/dense ID layoutを要求し、wrong total
count/ID layoutは常に`InvalidCaptureIdentity { capture_identity: ...Id::new(0) }`。
最後にexact inner owner context/range/parent/layer/scope/recovery/visibility associationを含む
全row fieldをC4C4 term/reference/projected binding/retained C4C3 use/bindingへ照合する。
Lowest invalid rowをreportし、exactly-one profileでは常にID `0`。Projected bindingのcaptured
identitiesはempty必須。Missing/extra/duplicate/reordered/stale/mismatch/recovered/partial stateは
このprecedenceでfailし、handoffをpublishしない。
Sort/deduplicate/spelling inference/C4C4 mutation/repairは禁止。

## Typed/ResolvedとCore/GeneratedOrigin boundary

C4C5はchecker association receiptだけをownする。capture set、binding visibility、
`TypedAst`/`ResolvedTypedAst` installationを決定しない。Later checker ownerはseparate
human decisionとfrozen contractなしにcapture semanticsを作れない。

Core Task33はlater Core context/binder/source identity owner、Core Task35はterm/formula
lowering、binder links、generated comprehension origin、source identity ownerのまま。
C4C5はfuture explicit free/generated-parameter transportのowner/orderを割り当てない。その
Core33/Core35 surface間joinはseparate human decisionを必要とする。
C4C5はCore adapter、`CoreVar`、parameter、`GeneratedOrigin`、sethood/membership evidence、ordering ruleを追加しない。
Future Core boundaryはown separate contractのもとcomplete checker inputをconsumeし、checker dense IDをCore identityとしてreinterpretしない。

## Scope、tests、audit impact

Production変更はexact 3 existing Rust files:

1. `source_term.rs`: crate-private C4C4 complete-validation seam
2. `source_formula_composition.rs`: sole owner/oracle/producer/checker tests
3. private `fraenkel_nested_capture_identity.rs`: sole current consumer

Checker test exact 4件:

1. `task257c4c5_builds_exact_capture_identity_handoff`
2. `task257c4c5_rejects_dependency_owner_and_precedence_corruption`
3. `task257c4c5_rejects_identity_cardinality_order_and_field_corruption`
4. `task257c4c5_replays_deterministically_and_preserves_empty_capture_and_installation`

Private leaf testはexact
`task257c4c5_real_imported_fixture_builds_capture_identity_handoff`。全testはexact C4C4/C4C5
routeだけをcallする。Active runner/registry consumerは追加しない。

Clean `HEAD 17b9af203fefe65d48ed88758d356ff8cdfcd0a3` baseline:

- `source_formula_composition.rs`: `9411` lines / `2b982a6ab418e63ee6996c428aea2f8d5a4b3fc6bb55c9e830043f07fec73e56`
- `source_term.rs`: `7574` / `2ef60bd40d0ff147f1615d20bd3a9fff3980e916868da90f998b00c3b4d369fe`
- private leaf: `416` / `7760e98cb9b6fb3ea26f232b34551119d6d084c0f4785cd11b3af7cf829be1f1`
- raw library tests: checker `562 -> 566`、mizar-test `621 -> 622`
- paired contracts: `99/99 -> 100/100`

Implementation後`doc/design/spec_coverage_audit.md`へzero-credit mapping 1件を追加する。
Trace row/status、test intent、active route、diagnostic、semantic result、coverage creditは不変。

## Completion evidence

Final source measurements:

| Path | Lines | SHA-256 |
|---|---:|---|
| `crates/mizar-checker/src/source_formula_composition.rs` | `9940` | `1b4efce50a86f36357478f1dcf98f64bda96a710de6ed1b8caa79e056cc3a515` |
| `crates/mizar-checker/src/source_term.rs` | `7583` | `f7703a170781fe0a2bd2840589ecab79ca56c2cd25006ba469abdebeac7012c0` |
| `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs` | `519` | `4c403bdc7b060e52b5ba6585b82d5f34485813a49d4d035ac7214239206b72cf` |

Checker productionは`32` paths / `196872` lines。Path hashは
`9dc5b02f26679677e593ea755394d68533173d2be988b7ef1ddcfd84a41b9787`のまま、
final content-manifest hashは
`47be280901c7feb00ce3454dc8d59d15fed71e741183b2f2201b034ef0e117a3`。
Paired contract treeは`100/100`。

Focused testはchecker `4/4`、mizar-test `1/1`、full libraryは`566/566`と
`622/622`、両lint-policyは`15/15`、metadataは`137/137`でPASS。
`cargo fmt --check`、workspace Clippy `-D warnings`、full `cargo test`、
`git diff --check`もPASS。Protected fixture/expectation/trace hashは上記frozen値と
exact一致し、`doc/spec`とprotected artifactにdiffはない。

全required independent reviewは**NO FINDINGS**。Final read-only quality reviewは
score capなしで全`9/9` hard gateをPASSし、valid uncapped `100/100`
（`20/20/15/15/10/10/5/5`）。Sole owner、exact by-value identity receipt、
default-deny oracle、empty captured state、zero-credit audit、Typed/Resolved/Core
installation不在を確認した。Task277Bはnot-ready/zero-creditのまま。

Reviewed task-only implementationはbaseline
`17b9af203fefe65d48ed88758d356ff8cdfcd0a3`上の
`72662d38dc42df7943e1d1db9f187fe1eced0da6`としてcommitした。
`git show --check`はPASS、immediate worktreeはclean、`origin/main...HEAD`は`0/1`。
Protected stash `f65cf4a13752ec380710814a9ac6392ccb9d75d4`、authority hash 3件、
source measurement、contract countは不変。Commitは自己hashを含められないため、
closure-record commit hashとその後のclean proofはfinal handoffで報告する。

## Forbidden behavior、reviews、exit

`doc/spec`、existing `.miz`、expectation、trace、diagnostic、active behavior、semantic
result、coverage credit、C4C4 captured state、Task277B readinessを変更しない。
Capture set/semantic capture decision/type/sethood/membership/generated parameter/Core origin/
installation/production dispatch/new fixtureを追加しない。

Required independent reviewはspecification/equivalence、bilingual/boundary、test
sufficiency、implementation、source/documentation/API consistency。Multiple owner/API/
installation/oracle ambiguityが再発したら`design_drift`/`test_gap`/`boundary_violation`として
停止し、replacementを推測しない。

Exitには全review no findings、focused/library test、checker/mizar-test lint/metadata、
`cargo fmt --check`、workspace Clippy `-D warnings`、full workspace tests、protected hash/
Task277B checks、exact diff/staging review、task-only commit、clean postcommit proofが必要。
Final measured count/hashはこのcontractへ一度だけrecordする。
