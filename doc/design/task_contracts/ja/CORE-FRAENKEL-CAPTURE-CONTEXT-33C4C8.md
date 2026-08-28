# Task CORE-FRAENKEL-CAPTURE-CONTEXT-33C4C8: Core capture-context association

> 正本は英語です。canonical English:
> [../en/CORE-FRAENKEL-CAPTURE-CONTEXT-33C4C8.md](../en/CORE-FRAENKEL-CAPTURE-CONTEXT-33C4C8.md)。

状態: precommit implementation complete。exact stagingとtask-only commitが残る。
本taskはchecker Task33Cに続くdependency-minimalなzero-semantic Core-33 successorである。利用者は
[CORE-SOURCE-CONTEXT-33P](CORE-SOURCE-CONTEXT-33P.md)が記録した未決定choiceの
review後に本contractを採択した。

## identity、authority、ownership

| field | freeze値 |
|---|---|
| task | `CORE-FRAENKEL-CAPTURE-CONTEXT-33C4C8` |
| primary owner | `mizar-core::elaborator`、Core Task 33 |
| owning plan | [`mizar-core` crate plan](../../mizar-core/ja/00.crate_plan.md)、private consumerの[`mizar-test` crate plan](../../mizar-test/ja/00.crate_plan.md) |
| sole checker admission | checker Task33Cの既存immutable `SourceNestedFraenkelCaptureGraphOwnerHandoff`。complete validatorはchecker-privateのまま |
| Core destination | standalone immutable `SourceNestedFraenkelCaptureCoreContextHandoff` 1個。`CoreContextInput`、Typed、Resolved fieldは追加しない |
| owner join | retained Task33Rの完全な`SymbolId`から既存`CoreItemRegistry` entryへ。display name、range-only、FQN-only、numeric joinは禁止 |
| capture | C4C8 capture tableにあるouter `x,y`だけ。inner generator `z`は除外 |
| order | C4C8が認証したprivate capture-table order。deterministic transportだけに使用 |
| allocation | 既存Core variable identityのchecked max + 1からfresh snapshot-local `CoreVarId`をcapture orderで連続割当。empty contextは0開始 |
| coverage | semantic/execution creditはzero。Task277Bはnot ready / zero creditのまま |

Authority orderは`doc/spec/en/`、exact C4C7 `.miz`、trace rowとexpectation、
C4C8/Task33R/Task33C contract、Core design/sourceの順を維持する。Chapter 13はresolved
binding identityによるcaptureを固定するがparameter orderをobservableにしない。このため
checker-authenticated `x,y` orderは新しいlanguage semanticsではなくprivateな
alpha-invariant canonical transport ruleとして扱う。

## frozen public API

`crates/mizar-core/src/elaborator.rs`へ以下を追加する。

- immutable `SourceNestedFraenkelCaptureCoreVariable`: `capture()`、
  `generator()`、`resolver_binding()`、`core_var()` getter。
- immutable `SourceNestedFraenkelCaptureCoreVariableTable`。Direct keyは
  `SourceNestedFraenkelCaptureGraphCaptureId`であり、
  `get(id) -> Option<&SourceNestedFraenkelCaptureCoreVariable>`、
  `iter() -> impl Iterator<Item = (SourceNestedFraenkelCaptureGraphCaptureId,
  &SourceNestedFraenkelCaptureCoreVariable)>`、`len() -> usize`、
  `is_empty() -> bool`を持つ。
- immutable `SourceNestedFraenkelCaptureCoreContextHandoff`: updated
  `CoreContext`、checker Task33C receipt、exact owner `CoreItemId`、capture-variable
  tableをretainedし、`source_id()`、`module_id()`、`context()`、
  `checker_receipt()`、`owner_item()`、`captured_variables()`、non-authoritative
  `debug_text()` getterを持つ。
- non-exhaustive `SourceNestedFraenkelCaptureCoreContextError`: precedence順に
  `EnvironmentMismatch`、`InvalidCoreContext`、`InvalidOwnerAssociation`、
  `CoreVariableAllocationOverflow`、`CoreVariableCollision { var }`、
  `InvalidCaptureAssociation`。
- `SourceNestedFraenkelCaptureCoreContextProducer::build(
  context: CoreContext,
  checker_receipt: SourceNestedFraenkelCaptureGraphOwnerHandoff,
  ) -> Result<SourceNestedFraenkelCaptureCoreContextHandoff,
  SourceNestedFraenkelCaptureCoreContextError>`。

Collision payloadは`var: CoreVarId`。Errorは`Debug`、`Clone`、`Copy`、`PartialEq`、
`Eq`をderiveし、`std::error::Error`をimplementする。Exact display stringはvariant順に
`nested Fraenkel capture Core context environment is invalid`、
`nested Fraenkel capture Core context is invalid`、
`nested Fraenkel capture Core owner association is invalid`、
`nested Fraenkel capture Core variable allocation overflowed`、
`nested Fraenkel capture Core variable <index> collides`、
`nested Fraenkel capture Core association is invalid`である。

Producerは両inputをby-valueでconsumeし、complete handoff validationが通るまで何も
publishしない。Row/table/handoff fieldとconstructorはprivate。public installer/adapter/
mutable field/unchecked constructor/numeric conversion/parameter-argument vectorは公開しない。

## installationとvalidation

既存Task33C valueをproof-carrying checker capabilityとする。Coreはそのprivate graph
validatorを複製・公開せず、Core側boundaryだけを次の順で検証する。

1. checker receiptとCore contextの`SourceId`/`ModuleId` exact equality。
2. existing Core variable metadataとused-ID inventoryのcoherence。
3. Task33R whole-`SymbolId`をexact lookupしたvalid `Functor` Core item 1件と、local
   semantic-originのsource/module/source-range anchor一致。
4. 全existing Core variable identityより上のchecked allocation。overflow/collision拒否。
5. checker capture tableとのexact 2-row positional association。capture id、generator id、
   resolver binding、generator binder sourceを照合し、captured `z`、missing、extra、
   duplicate、reorderedを拒否。

Declared-variable setは`BinderContext.free_variables`と`variable_classes`、
`variable_roles`、`variable_sorts`、`binder_type_facts`各key setのexact equalityである。
全`BinderSourceRegistry` keyと全`BinderFrame.original_var`はこのsetに属さなければならない。
全existing `GeneratedOrigin.params` entryもdeclaredでなければcontext invalidである。
Allocatorのused-ID unionはこのdeclared set、binder-source key、frame original variable、
generated-origin paramである。Term/formulaやresolver/checker numeric fieldは参加しない。

Accepted `x,y` rowはretained contextへ`NormalizedVarClass::Free`、
`NormalizedVarSort::Term`、role `fraenkel-captured-parameter`、exact generator-binder
`CoreSourceRef`、checker provenance、empty type-fact vectorとしてinstallする。型/evidence
ownerはCore 34のまま。Handoffはこれらのinvariant、retained non-capture variableに対する
consecutive allocation、reserved roleを持つextra variable不在をrevalidateする。

Capture id `n`について唯一のnew provenance keyはexact
`source-nested-fraenkel-capture-core-variable-v1.capture.<n>`、phaseは
`CoreProvenancePhase::Checker`である。Generator `binder_range()`はexact 1件の同provenanceを
持つ`CoreSourceRef::direct(range)`となり、matching `BinderSourceRecord`も同じphase/keyの
`CheckerOwnedProvenance` 1件だけを持つ。Resolver-id text、spelling、FQN、rangeをkeyへ
encodeしない。

Coherent max-plus-one allocationはcollisionしないため、public producerでcollision variantは
future allocator change後のdefensive pathだけである。Current private postvalidation/helper testが
variant/orderをcoverし、overflowはexisting `CoreVarId(usize::MAX)`でpublicに到達できる。

Rangeはlocal provenanceをauthenticateするだけでresolver binding/owner identityを置換しない。
Resolver/checker/Coreのnumeric ID domainを相互reinterpretしない。sort、repair、inference、
recovery、partial publication、unchecked admissionは禁止する。

## scopeとdefer

Implementation/test pathは次に限定する。

- `crates/mizar-core/src/elaborator.rs`。
- `crates/mizar-test/src/runner/tests/type_elaboration/fraenkel_nested_capture_identity.rs`。
- generic public-API guardがtask-neutral調整を必要とする場合だけ
  `crates/mizar-core/tests/lint_policy.rs`。

Owned documentation deltaは本contract pair、paired Core plan/TODO、paired
`source_family_decomposition.md`、paired `elaborator.md`、paired source/spec・
bilingual・module-boundary audit、paired mizar-test plan/harness/bilingual audit、central
coverage auditである。
`doc/spec`、existing `.miz`、expectation、trace metadata、checker source、C4C4 captured
state、manifest、diagnostic、active runner route、legacy-compaction ledgerは保護する。

本taskはgenerated parameter/application argument、term、formula、functor、generated key、
`GeneratedOrigin`、sethood result、type evidence、semantic result、snapshot、coverage creditを
作らない。Core 35は該当するCore-33 local-binder/Core-34 type-evidence prerequisite後にだけ
本handoffをconsumeできる。captured parameter/argument subvectorの双方でcapture orderを
保存し、allocate/infer/reorder/repairしてはならない。Domain operandはseparateのまま。
Exact Core35/GeneratedOrigin semanticsとactive routeはdeferする。

## test、baseline、exit

Rust testはexact real C4C7 receipt、empty context（resolver idが`1,2`でも`x,y ->
CoreVarId(0), CoreVarId(1)`）、populated contextのchecked max-plus-one、deterministic
replay、exact owner/source association、retained zero-semantic context state、environment
mismatch、missing/wrong owner、allocator overflow、public error textをcoverする。Private
helper testはmalformed local metadata/capture rowを扱ってよい。Checker corruptionは既存
C4C8/Task33C testがownerのまま。

Entryはclean `1bf83e3b9275283cf7bd2f40915fc98b057fc693`で`origin/main`と同一、
divergence `0/0`、protected stashは
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`。Contract treeは`108/108`から
`109/109`になる。Baseline `mizar-core` lib inventoryはlisted test 155件、raw-list
SHA-256は`dd8c3a3d78413f2dae4f10019bf84e8966ebd3539d6854ef994e4825e01712c6`。
Baseline sourceは`elaborator.rs`が`17132/631992`、SHA-256
`55a74c67e1d1a1dc79134d3835f7aa9c7a1ed70c040848abb1f03f0fb6d421a7`、private
mizar-test leafが`1008/40967`、SHA-256
`c38e7c2c99d3b81fb8906edaf90244c7d08eca913c6758bc5f7064a10bfcbcd8`。

Protected C4C7 source/expectation/trace hashはそれぞれ
`b2c9583acf176f32e538c895a3029fe344a90353c47bd6231c5d1e72bd935fbc`、
`277749efd4c149c2a7b85a07d7aa4243e7a7f402ccf976b28d68b16396ff0b1e`、
`17bba212e5216256b5883ce641048de263cd045a12adf060ed354973a6ae6728`を維持する。

Exitにはindependent specification/equivalence、bilingual/boundary、test-sufficiency、
implementation、source/documentation/API reviewのno findings、focused Core/mizar-test
test、library/lint/metadata、fmt、warnings-denied workspace Clippy、full workspace test、
protected count/hash/link check、autonomous hard gate `9/9`、quality `90/100`以上、exact
task-only commit、clean postcommit proof、fresh successor inventoryを要求する。

## completion evidence

Standalone Core association、private Core helper/postvalidation test 4件、private real-receipt
consumer test 5件はcomplete。Final Rust measurementは次のとおり。

| path | line / byte | SHA-256 |
|---|---:|---|
| `crates/mizar-core/src/elaborator.rs` | `18124 / 669642` | `65ee229c9d490f2838c4ca28864acf7b48a8fbf30e2c9b08b53dc3f7288d368d` |
| private mizar-test leaf | `1408 / 56492` | `e131f7bfdf015c820026061865d3a052542e37396de46e700571b3a97dc604ee` |

Contract treeはexact `109/109`。Final raw library-test inventoryは`mizar-core` `159` /
SHA-256 `aff91928e457018533af9bd8712b81aa1e58e58ec098fa12348fcab73d45a336`、
`mizar-test` `632` /
`a9464d8d30aed8fafc5ed0b066903ce30140bcef82fb97a500cdffed88e2b9e1`、unchanged
checker `580` /
`269021fdb0a7b7d1f30bb4a82ffc4fa544d6224ed7ecfcd8bf27186eef254d7c`。

Independent pre-source specification/equivalence、bilingual/boundary reviewと、post-source
test-sufficiency、implementation、source/documentation/API reviewはfinding-specific repair後
すべて**NO FINDINGS**。Focused Core `4/4`、private real-receipt `5/5`、Task33C `4/4`と
lint、Task33R `7/7`、Core `159/159`、mizar-test `632/632`、lint `15/15`、metadata
`137/137`、formatting、offline Cargo metadata、warnings-denied all-target/all-feature
workspace Clippy、full all-feature workspace test/doctest、`git diff --check`はpass。

Protected C4C7 source/expectation/trace hashは上記freeze値とexact一致。Checker Task33C
source/lintはそれぞれ
`dcff2322170389f17d4ed01e00e47ea70a07008906d9ad4358dfeca2e232a7a8`、
`3c726af3c41a0a28faf0c8ca0770a815293624ee1424ce31bd8575b97f299d30`、Core lint sourceは
`4aea1816db81c1625b7353f4e7829528020ec2d69f054360004234ea28201103`のまま。
Specification、existing `.miz`/expectation/trace、checker、C4C4 capture、manifest、
Typed/Resolved、diagnostic、semantic route、generated origin、coverage credit、Task277B stateは
変更していない。Exact staging、cached review、parent final hard-gate scoring、task-only commit、
clean postcommit proof、fresh successor inventoryがexit operationとして残る。
