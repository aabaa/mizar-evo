# 証明ローカル宣言 source transport

> 正本は英語です。英語版:
> [../en/source_proof_local_declaration.md](../en/source_proof_local_declaration.md)。

## 状態と authority

このliving owner文書はChecker Task-269 proof-local declaration lineageの
Tasks 269A/B/CP/C/CTからGP/GS/G/GT/GUP/GUPT/GU、GCP/GC/GCT/GCUまでの
completed sliceと、current lower-only **Task 269SDP** contractをfreezeする。
英語版がcanonicalであり、同じlogical task内で本JA companionを同期する。

normative authorityは次の順である。

1. `doc/spec/en/03.type_system.md` §§3.1--3.4。
2. `doc/spec/en/04.variables_and_constants.md` §§4.1、4.2、4.4（特に
   4.4.1/4.4.3）、4.6（特に4.6.1/4.6.2）。
3. `doc/spec/en/08.type_inference.md` §§8.1、8.3。
4. `doc/spec/en/13.term_expression.md` §§13.1.1、13.8.1。
5. `doc/spec/en/15.statements.md` §§15.2.1--15.2.2、15.3.3、15.4.4、
   15.6.1、15.10、15.11.1--15.11.2、15.11.4。
6. `doc/spec/en/16.theorems_and_proofs.md` §§16.3.3、16.4.1--16.4.3、
   16.5。§16.5はhistorical syntax/justification boundaryだけを保持し、proof
   justification ownershipを意味しない。
7. 実装済みTask-258B3Nのexact source/statement/witness/term transportと
   parser/resolver provenance。
8. parser simple/block statement fixture、broad proof-local declaration
   fixture、mixed predicate/functor boundary fixtureとunchanged sidecar/trace。
9. Tasks 248--259の公開API、特に`LocalTermBinding`、`BindingEnv`、
   `SourcePrimaryTermHandoff`、`SourceStatementHandoff`、
   `SourceStatementWitnessHandoff`。

広いproof-local declaration gap fixture
`fail_type_elaboration_proof_local_declaration_gap_001.miz`、sidecar、existing
covered diagnostic-gap trace rowはread-onlyのままにする。これらのrowは
positive proof-local binding semanticsをcreditしない。このfixtureは`let`、
`given`、`consider`、`set`、`reconsider`を混在させるためindividual
Task-269 sliceの安全な表現には単独で使えない。completed sliceとlower-only
SDPをblockする`spec_gap`はない。Ch.4/15のlater `set` effect矛盾はSDP syntax
transportにはnonblocking、capture/closure consumerにはblockingである。

historical Task-269A selection inventoryはHEAD
`52cf07be3c77d3aa2a797a7681ed9cbabf88295b`、`main`、docs edit前clean、
`origin/main...HEAD = 0/19`、protected `stash@{0}`
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`。origin divergenceはreport-only
`repo_metadata_conflict`で、task-only targetを曖昧にせず修復しない。

current Task-269SDP selection inventoryはHEAD
`f984ae683419944493c07723e9950a9101a46502`、`main`、SDP docs edit前clean、
同じreport-only origin `0/19`、同じprotected stash identityである。

## Historical Task 269A 分類とtask選択

認証済みnamed witnessにchecker-owned binding transactionがない差を
`source_drift`、contract欠落を`design_drift`、exact producer/ownership/
consumer test欠落をcanonical-derived `test_gap`と分類する。name tokenを
public symbolにする、checkerでsyntaxから再構成する、witness typingやproof
resultを公開することは`boundary_violation`である。

Task 269AはTasks 248--258後にdependency-readyである。これはTask 269全体
より狭く、named-witness local bindingを1件作りdefinition-site linkを記録
するが、後続useは実装しない。後続Task-269 sliceが`let`/`set`/`given`/
`consider`、複数introductions、later-use replay、capture-by-resolved-binding
coverageを保持する。Task 270は`deffunc`/`defpred`、Task 271は
`reconsider`、Task 272はexistential witness matching、witness-type
obligation、goal substitutionを保持する。

## exact sourceとlower profile

許可するのは既存private Task-258B3N textだけである。

```mizar
reserve x for set;
theorem FormulaStatementNamedWitnessSmoke: x = x proof
  take y = x;
  thus x = x;
end;
```

final newlineを含むexact 107 bytes、SHA-256は
`a57022c4b75991dd4308943477e03819f5bfe2c0d23ea1030730256252d7d329`。
normal Surface ASTは51 nodes/root 50でrecovery/diagnosticは0。name、
witness、take site/rangeは`13/81..82`、`36/81..86`、`37/76..87`。
RHSは`SourceStatementWitnessTermTarget::Primary(SourcePrimaryTermId(2))`、
site 34、range `85..86`、spelling `x`、source ordinal 2、proof context 1で、
既存referenceはuse ordinal 1にreserved `BindingId(0)`へresolveする。

lower cardinalityは不変である。

- transaction前binding contexts/bindings/diagnostics `2/1/0`。
- primary terms/references/numeric requests `5/5/0`。
- atomic formula rows `2/0/0/0/0/0/0/4/4`。
- theorem owners/statements/contexts/input facts/candidate facts `1/2/2/2/2`。
- witnesses/names `1/1`。

proof contextは`BindingContextId(1)`、parent 0、layer `Proof`、lexical scope
`[0]`、normal recovery、transaction前`bindings=[]`、
`visible_bindings=[BindingId(0)]`。Task 269Aはarena nodeのkind、anchor、
children、typing、recovery、linkを変更せず、全lower handoffと51 nodesを
validateする。

## resolver-local provenance

private runnerは完全なsource/Surface profileを認証した後だけ、exact 1件
のresolver-owned `LocalTermBinding`を渡す。

| field | exact value |
|---|---|
| spelling | `y` |
| lexical scope | `[0]` |
| declaration range | `81..82` |
| visible-after ordinal | `1` |

checkerはこの値をconsumeし、`"y = x"`をparseしたりtoken scanしたり、
`SymbolId`、declaration shell、contribution、name reference、module symbolを
捏造しない。resolver environmentにはvisible module symbol `y`が存在しない。
`BinderIdentity::ResolverLocal`が上記4 fieldをprovenanceとして保持する。

## 公開checker API

syntax-free module `source_proof_local_declaration`を追加する。次のRust宣言が
export family全体、field visibility、derive、signatureをfreezeする。
implementationはsecond constructorやmutable table/handoff accessを追加しない。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceProofLocalDeclarationId(usize);

impl SourceProofLocalDeclarationId {
    pub const fn new(index: usize) -> Self;
    pub const fn index(self) -> usize;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalDeclarationHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub declarations: Vec<SourceProofLocalDeclarationInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalDeclarationInput {
    pub witness: SourceStatementWitnessId,
    pub name: SourceStatementWitnessNameId,
    pub rhs: SourceStatementWitnessTermTarget,
    pub binding_context: BindingContextId,
    pub source_ordinal: usize,
    pub local: LocalTermBinding,
    pub kind: SourceProofLocalDeclarationKind,
    pub recovery: SourceProofLocalDeclarationRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceProofLocalDeclarationKind {
    NamedWitness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceProofLocalDeclarationRecovery {
    Normal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalDeclaration {
    witness: SourceStatementWitnessId,
    name: SourceStatementWitnessNameId,
    rhs: SourceStatementWitnessTermTarget,
    binding: BindingId,
    binding_context: BindingContextId,
    source_ordinal: usize,
    visible_after_ordinal: usize,
    kind: SourceProofLocalDeclarationKind,
    recovery: SourceProofLocalDeclarationRecovery,
}

impl SourceProofLocalDeclaration {
    pub const fn witness(&self) -> SourceStatementWitnessId;
    pub const fn name(&self) -> SourceStatementWitnessNameId;
    pub const fn rhs(&self) -> SourceStatementWitnessTermTarget;
    pub const fn binding(&self) -> BindingId;
    pub const fn binding_context(&self) -> BindingContextId;
    pub const fn source_ordinal(&self) -> usize;
    pub const fn visible_after_ordinal(&self) -> usize;
    pub const fn kind(&self) -> SourceProofLocalDeclarationKind;
    pub const fn recovery(&self) -> SourceProofLocalDeclarationRecovery;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalDeclarationTable {
    rows: Vec<SourceProofLocalDeclaration>,
}

impl SourceProofLocalDeclarationTable {
    pub fn get(
        &self,
        id: SourceProofLocalDeclarationId,
    ) -> Option<&SourceProofLocalDeclaration>;
    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (SourceProofLocalDeclarationId, &SourceProofLocalDeclaration),
    >;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalDeclarationHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    base_binding_fingerprint: String,
    statement_fingerprint: String,
    witness_fingerprint: String,
    primary_term_fingerprint: String,
    binding_env: BindingEnv,
    final_binding_fingerprint: String,
    declarations: SourceProofLocalDeclarationTable,
}

impl SourceProofLocalDeclarationHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn base_binding_fingerprint(&self) -> &str;
    pub fn statement_fingerprint(&self) -> &str;
    pub fn witness_fingerprint(&self) -> &str;
    pub fn primary_term_fingerprint(&self) -> &str;
    pub const fn binding_env(&self) -> &BindingEnv;
    pub fn final_binding_fingerprint(&self) -> &str;
    pub const fn declarations(&self) -> &SourceProofLocalDeclarationTable;
    pub fn debug_text(&self) -> String;

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        statements: &SourceStatementHandoff,
        witnesses: &SourceStatementWitnessHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
    ) -> Result<(), SourceProofLocalDeclarationError>;

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        statements: &SourceStatementHandoff,
        witnesses: &SourceStatementWitnessHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalDeclarationError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SourceProofLocalDeclarationProducer;

impl SourceProofLocalDeclarationProducer {
    pub fn build(
        input: SourceProofLocalDeclarationHandoffInput,
        statements: &SourceStatementHandoff,
        witnesses: &SourceStatementWitnessHandoff,
        primary_terms: &SourcePrimaryTermHandoff,
        arena: &TypedArena,
    ) -> Result<
        SourceProofLocalDeclarationHandoff,
        SourceProofLocalDeclarationError,
    >;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalDeclarationError {
    InvalidTransaction,
    DependencyMismatch,
    InvalidAggregate,
    InvalidDeclaration {
        declaration: SourceProofLocalDeclarationId,
    },
    InvalidArena,
    InvalidBindingEnvironment,
    InvalidInstallation,
}
```

上記の`SourceId`、`ModuleId`、`LocalTermBinding`、`BindingContextId`、
`BindingId`、`BindingEnv`、5件の`SourceStatement*` type、
`SourcePrimaryTermHandoff`、`TypedArena`は現owner moduleの既存typeであり、
new moduleはalias/replacementを定義しない。

parser/syntax typeはAPIを越えない。callerはfinal `BindingId`を指定できず、
dense identityはcheckerがtransactionally割り当てる。
`validate_complete_installation`はcrate-private integration surfaceである。
`validate_installation`でphase 1--6をreplayした後、owner availabilityがfalseなら
phase-7 `InvalidInstallation`へmapする。Typed/final ownerはこのinternal errorを
dedicated AST errorへmapし、何もpublishしない。

## exact output transaction

exact inputは次の1 rowである。

```text
declaration#0 kind=named-witness witness=0 name=0 rhs=primary#2
context=1 source_ordinal=1 local=("y", scope=[0], range=81..82,
visible_after=1) recovery=normal
```

producerはTask-258B3N base environmentを完全clone-preserveし、proof context
を置換してbindingを1件appendする。post profileはcontexts/bindings/
diagnostics `2/2/0`。

- `BindingId(0)`はreserved `x` rowとbyte-identical。
- `BindingId(1)`はspelling `y`、kind `LocalAbbreviation`、owner context 1、
  declaration range `81..82`、visible-after 1、`BindingTypeSite::Missing`、
  `BindingStatus::Active`、empty capture/diagnostics、normal recovery。
- identityはexact `ResolverLocal(scope=[0], ordinal=1,
  declaration_range=81..82)`。
- context 0はbyte-identical。
- context 1は`bindings=[1]`、`visible_bindings=[0,1]`となり、他fieldは
  byte-identical。

declaration rowはwitness 0、name 0、RHS primary term 2、binding 1をlinkする。
source ordinal 1の`y` lookupはforward referenceで、同じscopeのlater
ordinalではbinding 1になる。これによりRHSが自己bindingをcaptureしない。
Task 269Aはdefinition-site linkだけを記録し、later termをrewrite/expand
しない。

## fingerprintとvalidation order

handoffは次のexact byte fingerprintを保持する。

1. extension前`statements.binding_env().debug_text()`。
2. `statements.debug_text()`。
3. `witnesses.debug_text()`。
4. `primary_terms.debug_text()`。
5. final extended `BindingEnv::debug_text()`。

installation/final assemblyは5件すべてを再計算する。同じcardinalityだけ
では不十分。validation orderは次の通り。

1. source/module transaction identity。
2. lower source/module/fingerprint equalityとexact Task-258B3N profile。
3. exact one-row aggregateとdense IDs。
4. resolver-local spelling/scope/range/ordinalとrow links。
5. Task-258B3N owner kind不変を含む全51-node arena/subtree replay。
6. base-to-final binding-environment reconstructionとlookup behavior。
7. typed/final one-shot installation invariant。

最初のfailure classを返す。phase 1--7はそれぞれ
`InvalidTransaction`、`DependencyMismatch`、`InvalidAggregate`、
`InvalidDeclaration { declaration: SourceProofLocalDeclarationId(0) }`、
`InvalidArena`、`InvalidBindingEnvironment`、`InvalidInstallation`へmapする。
exact `Display` textは次である。

| variant | exact text |
|---|---|
| `InvalidTransaction` | `source proof-local declaration transaction is invalid` |
| `DependencyMismatch` | `source proof-local declaration dependency mismatch` |
| `InvalidAggregate` | `source proof-local declaration aggregate is invalid` |
| `InvalidDeclaration { declaration }` | `source proof-local declaration <declaration.index()> is invalid` |
| `InvalidArena` | `source proof-local declaration arena is invalid` |
| `InvalidBindingEnvironment` | `source proof-local declaration binding environment is invalid` |
| `InvalidInstallation` | `source proof-local declaration installation is invalid` |

errorは`std::error::Error`を実装する。failure時はpartial binding/context/
handoff/debug suffix/final ownerを公開しない。

## stable debug grammar

新blockは既存statement-witness block後にappendし、legacy debug byteを変更
しない。

```text
source-proof-local-declaration-debug-v1
module: <package>::<path>
base-binding-fingerprint: <quoted debug bytes>
statement-fingerprint: <quoted debug bytes>
witness-fingerprint: <quoted debug bytes>
primary-term-fingerprint: <quoted debug bytes>
declaration#0 kind=named-witness witness=0 name=0 rhs=primary#2 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal
final-binding-fingerprint: <quoted debug bytes>
```

rowはdense ID順。enum spellingはexact `named-witness`/`normal`、RHSは
`primary#<id>`。各fingerprint valueはRust string `Debug` (`{:?}`)でformat
するためdouble quoteされ、embedded newline、quote、backslash、control
characterにはstandard backslash escapeを用いる。line順は表示どおり、blank
lineはなく、block末尾はexactly 1 LF。empty legacy profileは新blockを出さない。

## Typed/final ownership

`TypedAst`はprivate optional `SourceProofLocalDeclarationHandoff`とexactly
次の2 methodを追加する。

```rust
pub const fn source_proof_local_declaration(
    &self,
) -> Option<&SourceProofLocalDeclarationHandoff>;

pub fn with_source_proof_local_declaration(
    self,
    handoff: SourceProofLocalDeclarationHandoff,
) -> Result<Self, TypedAstError>;
```

`TypedAstParts`にreplacement fieldは追加しない。exact Task-258B3N source
term/atomic formula/statement/witness bundleが既にinstallされ、他semantic
tableがemptyの場合だけ全fingerprint/row/arena/binding transitionを再検証し
atomic publishする。`TypedAstError`はexact unit variant
`InvalidSourceProofLocalDeclaration`を1件追加する。

`ResolvedTypedAst`は`TypedAst`からhandoffをclone-preserveし、matching
read-only getterだけを同じexact `source_proof_local_declaration` signatureで
追加して同じvalidationをreplayする。`ResolvedTypedAstInputs`にreplacement
pathはない。`ResolvedTypedAstError`はexact unit variant
`InvalidSourceProofLocalDeclaration`を1件追加し、orphan、duplicate、stale、
same-length corruption、half-installはこれでfailする。既存Task-258B3N node
ownership/hintは不変で、Task 269Aはarena node/roleを追加しない。

## private runner consumerと除外

`mizar-test`はprivate dormant Task-269A leafを1件追加する。exact 107-byte
B3N sourceだけを選択し、既存Task-258B3N producer、exact resolver
`LocalTermBinding`構築、checker producer、typed install、final reassemblyを
順に行う。public corpus dispatch、expectation selection、diagnostic
serialization、CLI routeへwireしない。既存Task-258B3N route/debug byteは
不変である。

すべてのnear miss、unnamed/multiple witness、異なるRHS、recovery node、
異なるscope/range/ordinal、`let`、`set`、`given`、`consider`、`reconsider`、
`deffunc`、`defpred`、imported symbol、広いgap fixtureを除外する。

## semantic deferral

Task 269Aはinferred witness type、type row、coercion、diagnostic、initial
obligation、equality fact、existential match、goal/guard composition、goal
substitution、proof node、discharge、acceptance、theorem fact、Core IR、CFG、
VCを公開しない。`BindingTypeSite::Missing`はrepresentation boundaryであり、
type inference成功ではない。

Task 269B+がlater-use/capture replayと他のproof-local declaration form、
Task 270がfunctional/predicative abbreviation、Task 271がreconsideration、
Task 272がnamed-witness typing/existential-goal effectを保持する。現実装から
これらのsemantic behaviorを推測しない。

## testとcount impact

checkerはexact construction/lookup、corruption precedence/rollback、typed
ownership/legacy compatibility、final replay/sibling isolation/empty semantics
のexact 4 library testsを追加する。private runnerはexact frontend、resolver/
lower mutation、near-miss/route isolation、typed/final replayのexact 4 testsを
追加する。projected library countはchecker `478 -> 482`、runner
`532 -> 536`、resolver/syntax `148/59`不変。checker/runner production
manifestはそれぞれ1 source path増え、line/path/content/test-list hashは実装後
再測定する。

docs prerequisite/implementationとも`.miz`、sidecar、expectation、trace
row/backlink/status、metadata case、active outcome、diagnostic code/key、CLI
outputを変更しない。corpus/requirements `428/395`、pass/fail `235/193`、
stages `101/7/205/1`、type `259=247+12`、warnings/errors `23/0`、trace
SHA-256
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`
は不変。

## Public Enum Policy

| Public enum | Compatibility policy |
|---|---|
| `SourceProofLocalDeclarationKind` | `#[non_exhaustive]`。callerはlater explicitly frozen proof-local declaration formを許容する。 |
| `SourceProofLocalDeclarationRecovery` | `#[non_exhaustive]`。callerはlater recovery classを許容する。 |
| `SourceProofLocalDeclarationError` | `#[non_exhaustive]`。callerはvalidation/installation failureをexhaustive matchしない。 |
| `SourceProofLocalLetBindingRecovery` | `#[non_exhaustive]`。callerはlater explicitly frozen proof-`let` recovery classを許容する。 |
| `SourceProofLocalLetBindingError` | `#[non_exhaustive]`。callerはproof-`let` validation/installation failureをexhaustive matchしない。 |
| `SourceProofLocalGivenBindingRecovery` | `#[non_exhaustive]`。callerはlater explicitly frozen proof-`given` recovery classを許容する。 |
| `SourceProofLocalGivenBindingError` | `#[non_exhaustive]`。callerはproof-`given` validation/installation failureをexhaustive matchしない。 |
| `SourceProofLocalGivenUseBindingError` | `#[non_exhaustive]`。callerはproof-`given` later-use-profile validation failureをexhaustive matchしない。 |
| `SourceProofLocalGivenConditionBindingError` | `#[non_exhaustive]`。callerはproof-`given` declaration-condition binding validation/installation failureをexhaustive matchしない。 |
| `SourceProofLocalGivenDescendantBindingError` | `#[non_exhaustive]`。callerはproof-`given` descendant binding/context validation/installation failureをexhaustive matchしない。 |

この module が所有する exhaustive public enum exception はない。

implementationはnew exported moduleに対するexisting checker lint-policy
module/doc/public-surface inventoryも更新する。このguard editはtest caseを
追加せずfrozen implementation scopeに含む。source/spec export inventoryは
`lib.rs`が実際にexportするimplementation commitでだけ変更する。

## exit criteria

Task 269Aは次をすべて満たした場合だけ完了する。

1. exact producer/post binding environmentがcontractに一致する。
2. typed/final ownershipがatomicでlegacy Task-258B3N byteが不変。
3. private consumerとexact 8 testsがcorpus/trace activationなしでpass。
4. focused/crate/lint/metadata/fmt/Clippy/workspace/all CLI/count/hash/
   whitespace gateがpass。
5. test、implementation、source/docs、final quality reviewが**NO FINDINGS**、
   全9 hard gatesがscore capなしでPASSし90/100以上。
6. frozen Task-269A scopeだけを1 implementation commitにし、fresh inventory
   から次のdependency-ready Task-269 sliceへ自動継続する。

## implementation result

frozen module/API/producer、fingerprint 5件、`2/1/0 -> 2/2/0` transition、
ordinal lookup replay、Typed/final ownership、dormant runner leaf、exact compound
test 8件を実装した。checker/runner libraryは`482/536`、production inventoryは
`30/164419`、`37/69729`。exact fixture/corpus/trace/metadata/CLI no-opと全semantic
deferralを保存する。independent review、full verification、exact commit、fresh
Task-269B+ inventoryがcompletion gateとして残る。

その後independent reviewは全て**NO FINDINGS**、hard gate 9件はscore capなし
`100/100`、full verificationはPASSし、implementation commit
`f548ceb9f1acbeca72919809f2a1db84da213982`後worktree clean、origin divergence
`21/0` report-only、protected stash不変を確認した。fresh inventoryは以下の
Task 269Bを選択した。

## Task 269B frozen mixed-witness binding increment

### selection、authority、classification

complete Task-258B3M1 lower transportが既にpublic/verifiedなため、Task 269Bは
次のdependency-ready slice。canonical authorityはChapter 4 §§4.4.3/4.6、
Chapter 15 §15.4.4のleft-to-right syntax-order note、Chapter 16 §16.3.3 item
5/§16.4、existing `pass_parser_simple_statements_001.miz`、frozen
Task-258B3M1 contract、committed Task-269A API。broad gap fixture/expectation/
traceはread-only diagnostic authorityでpositive creditを与えない。

missing B3M1 transactionは`source_drift`、open-ended ownershipは
`design_drift`、exact-profile test不足はcanonical-derived `test_gap`。
blocking `spec_gap`/lower defectはなく、parser diagnostic 0、structural resolver
lowering、private theorem owner、全lower handoffはcomplete。unnamed witness1を
bindingにする、goal effect/typing/proofを加えるのは`boundary_violation`。

fresh inventoryはHEAD `f548ceb9f1acbeca72919809f2a1db84da213982`、`main`、
clean、`origin/main...HEAD=0/21`、protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`。origin divergenceはreport-only
`repo_metadata_conflict`で修復しない。

### exact source/lower transaction

admitted source 2件目はexisting final-LF 113-byte Task-258B3M1、SHA-256
`412a6a7f8fddebd67418f3482855ea89a1e7da922b42ebb93463971d8e49c186`：

```mizar
reserve x for set;
theorem FormulaStatementMultipleWitnessSmoke: x = x proof
  take y = x, x;
  thus x = x;
end;
```

parser diagnostic 0、56 unrecovered nodes/root55、one current-module theorem
owner、import/citation/proof-step label/witness-name symbolなし。exact siteはname
`13/84..85`、named witness `38/84..89`、take `42/79..93`、RHS primary2
`36/88..89`。lower profileはbinding `2/1/0`、primary `6/6/0`、atomic
`2/0/0/0/0/0/0/4/4`、statement `1/2/2/2/2`、witness/name `2/1`。

witness0は`Named`、name0/`Primary(2)`、ordinals `1/0`、`y = x`。witness1は
`Unnamed`、nameなし、`Primary(3)`、`1/1`、`x`。Task269Bはwitness0の
declarationだけを作りwitness1へchecker bindingを与えない。

runnerはfull authentication後resolver-local `y`、scope `[0]`、range
`84..85`、visible-after1を渡す。transactionはdeclaration0、witness/name/RHS
`0/0/2`、context1、source ordinal1、`NamedWitness`、normal。base `x`を保存し
`2/1/0 -> 2/2/0`だけを行う。binding1は`LocalAbbreviation`、
`ResolverLocal([0],1,84..85)`、missing type、active、uncaptured、diagnosticなし。
context1はbindings `[1]`/visible `[0,1]`。ordinal1はforward、ordinal2はbinding1。

### API/fingerprint/ownership/validation

public type/field/variant/error/method/installer/debug line/module/source pathを追加しない。
Task-269A APIとbase BindingEnv/statement/witness/primary/final BindingEnvの5 exact
fingerprintをreuseしprofile tagは追加しない。exact cardinality/fingerprint/range/
all-node replayでB3N/B3M1を識別する。

phase 7件は不変。phase2はexact B3N/B3M1、phase3はdeclaration exactly1、phase4は
`0/0/2`/resolver provenance、phase5は56 nodes、phase6はfinal env、phase7は
Typed/final one-shot。cross-profile hybridはexisting precedenceでatomic reject。
B3N/public/debug byteは不変。private leafはexact B3M1 branchだけを追加し、public
dispatch/corpus/metadata/diagnostic/CLIを触らない。

### exclusion、test、impact、exit criteria

later-use/capture、witness1 binding、additional named witness、他B3M2、`let`/
`given`/`consider`/`set`/`deffunc`/`defpred`/`reconsider`、imported spelling、
type/coercion/obligation、existential/goal/guard/substitution、fact/proof/
acceptance、Core/CFG/VCをexclude。left-to-rightはsyntax orderだけでgoal effectは
Task272。

existing checker4+runner4 compound testsをexpandしfunctionは増やさない。B3M1、
witness2件中row0だけbinding、fingerprint5件、56 nodes、mutation、B3N compatibility、
Typed/final、route isolation、empty semanticsをcover。countsは`482/536`、
resolver/syntax `148/59`、production paths `30/37`。line/hashはremeasure。

`doc/spec`、`.miz`、sidecar、expectation、trace/coverage、metadata、diagnostic、
Cargo、public route、active outcome、CLIは不変。よって
`doc/design/spec_coverage_audit.md`はno-op。

docs review **NO FINDINGS**/dedicated commit/fresh preflight後に実装し、全review
**NO FINDINGS**、hard gate9件uncapped 90/100以上、verification/count/hash/staging、
implementation commit、fresh next Task-269 selectionでcomplete。

Completion evidence: [central Task-269B historical contract](../../task_contracts/ja/269B.md#completion-evidence)。

## Checker Task 269CP frozen isolated proof-`let` lower prerequisite

### 選択、authority、分類

Task-269B implementation commit
`afd54a37ce4022929bdaf60be519ac4adbdd9b8e`直後のfresh inventoryはTask
269CPだけをselectする。canonical authorityはChapter 4 Sections 4.2/4.6、
Chapter 15 Sections 15.2.1/15.10/15.11.1、Chapter 16 Sections 16.3.3/16.4
である。existing parser simple-statement fixture/testsはnormal
`LetStatement -> QualifiedVariableSegment -> TypeExpression` shapeを与える。
mixed active fixture
`fail_type_elaboration_proof_local_declaration_gap_001.miz`、sidecar、trace
backlinkはread-only boundary evidenceのままである。

isolated lower contract不在は`design_drift`、private extractor不在はbounded
`source_drift`、mutation/isolation tests不在はcanonical-derived `test_gap`で、
blocking `spec_gap`はない。resolverにAST-wide local-use/capture tableがないため、
later-use/captureはdependency-readyでない。checker-side syntax reconstructionは
`boundary_violation`になる。origin/main差はreport-only
`repo_metadata_conflict`でありexact targetを隠さない。

Task 269CPはfuture Task 269Cのrunner-private lower prerequisiteである。上で
freezeしたchecker ABIをextendせず、checker bindingやTyped/Resolved proof-local
handoffをinstallしない。Task-269C直接選択は`BindingTypeSite::Missing`を維持する
binding-only transactionに限り、`SourceTypeProducer`をcall/extendしてはならない。

### Exact source、Surface profile、fingerprint

sole admitted sourceは次のprivate final-LF textである。

```mizar
reserve x for set;
theorem FormulaStatementLetSmoke: x = x proof
  let y be set;
  thus x = x;
end;
```

final LF 1個込みでexactly 100 bytes、source SHA-256は
`7860a3fe5af89063ac6a2b9a4465cac36d26f6d64e892ba6e2c89bcbaaf9763a`。
normal Surface snapshot SHA-256は
`1fc35ec18db82efc0968b2f42b08cfaae678184983210cd26f060d45354c7f68`、
51 nodes/root 50/root range `0..99`でrecovery/frontend diagnosticは0である。

| node | kind | range | children |
| ---: | --- | --- | --- |
| 27 | `ReserveItem` | `0..18` | `[0,26,4]` |
| 34 | `TypeHead` | `76..79` | `[15]` |
| 35 | `TypeExpression` | `76..79` | `[34]` |
| 36 | `QualifiedVariableSegment` | `71..79` | `[13,14,35]` |
| 37 | `LetStatement` | `67..80` | `[12,36,16]` |
| 45 | `ConclusionStatement` | `83..94` | `[17,44,21]` |
| 46 | `ProofBlock` | `59..98` | `[11,37,45,22]` |
| 47 | `TheoremItem` | `19..99` | `[5,6,7,33,46,23]` |
| 48/49/50 | `ItemList` / `CompilationUnit` / `Root` | `0..99` | `[27,47]` / `[48]` / tokens `0..23` plus `[49]` |

name token 13は`y@71..72`、token 14は`be@73..75`、type-head token 15は
`set@76..79`。declaration ordinalはtheorem 0とconclusion 2の間の1、proof
lexical scopeは`[0]`である。

### Resolver provenanceとprivate lower output

resolverはnormal shell 2件だけを持つ: reserve shell0/node27/`0..18`と
theorem shell1/node47/`19..99`。public/exported theorem projection/symbol 1件、
definition0、`LocalSource` contribution0、origin path `[2,1]`を生成し、import、
label、overload、registration、visible module symbol `y`はない。private extractorは
complete provenance認証後だけ
`LocalTermBinding::new("y", LocalTermScope::new(vec![0]), 71..72, 1)`を構成する。

implementationはcrate-private syntax-free
`SourceProofLocalLetLowerOutput` 1件だけをownする。source/module identity、theorem
symbol/definition/contribution、theorem/proof/let/segment/name/type/type-headの
role-specific range、ordinal1、local binding、deterministic debug textをretainedする。
raw `SurfaceAst`/node id/kind/token/source textはexisting private
`mizar-test::runner::type_elaboration::source_statement` leaf内に留める。
source/snapshot hashはselector fingerprintで、checkerのindependent typed fieldではない。
Task 269Cがcopyできるのはcomplete byte-exact `debug_text()` string 1件だけで、opaque
syntax-free authentication fingerprintとして扱う。embedded source/snapshot hashとtype
rangeはselector evidenceのまま、checkerはtyped siteへparseせずindependent fieldも
acceptしない。特に上のSurface tableのnode番号はrunner boundaryをcrossせず、
`TypedSiteRef`へlaunderせずtyped ownershipとしてもpublishしない。

private data shapeはexact fields `source_id`、`module_id`、
`source_fingerprint`、`surface_fingerprint`、`theorem_symbol`、
`theorem_definition`、`contribution`、`theorem_range`、`proof_range`、
`let_range`、`segment_range`、`name_range`、`type_range`、`type_head_range`、
`source_ordinal`、`local`を持つ。field name自体がsource roleを表し、generic site
idはない。deriveは`Debug`、`Clone`、`PartialEq`、`Eq`。read-only crate-private
getterと`debug_text()`だけを公開し、leaf外constructorはない。

complete debug grammarは次である。

```text
source-proof-local-let-lower-debug-v1
module: <package>::<module>
source-fingerprint: "7860a3fe5af89063ac6a2b9a4465cac36d26f6d64e892ba6e2c89bcbaaf9763a"
surface-fingerprint: "1fc35ec18db82efc0968b2f42b08cfaae678184983210cd26f060d45354c7f68"
theorem symbol=<quoted-fqn> definition=0 contribution=0 range=19..99 proof=59..98
let range=67..80 segment=71..79 source_ordinal=1
name range=71..72 spelling="y" scope=[0] visible_after=1
type range=76..79 head=76..79 spelling="set" form=bare
```

`<package>`、`<module>`、`<quoted-fqn>`だけがvalidated runtime valueで、残りの
全byte/line orderとtrailing LF 1個はliteralである。

### Ownership、exclusion、semantic deferral

269CPはexact extraction/provenance authenticationだけをownする。
`SourceStatementHandoff`、`SourceTypeApplicationHandoff`、`BindingEnv` mutation、
`LetBinding`、source proof-local handoff、Typed/Resolved owner、type result、assumption、
fact、obligation、diagnostic、goal、theorem status、proof、Core/CFG/VCをpublishしない。
future Task 269Cはchecker let-binding ABIを別途freezeし、binding type siteをmissing
のままにする。`LetBinding` source-type admissionの不在はこのbinding-only transaction
をblockしないが、later typed-source ownerは別prerequisiteとしてselect/freezeする。
Task 269CP/269Cのどちらにも混在させない。

selectorはbyte差、node/root/range/child/token/recovery差、shell/symbol/definition/
contribution/module/namespace/origin/scope/ordinal/local-field差、multiple/implicit
variable、multiple segment、attribute、`such that`、trailing `by`、nested proof、
later `y` use、`given`/`consider`/`take`/`set`/`reconsider`/`deffunc`/`defpred`
substitutionをrejectする。Task-269A/B sourceとmixed active gap fixtureはexact
no-match familyである。

Chapter-15 universal encoding、type guard、well-formedness discharge、goal/thesis
transformation、universal closure、単一definition siteを超えるshadow behavior、
later-use、capture、typing、proof acceptance、全semantic effectはdeferredである。
現実装からruleを推測しない。

### Tests、impact、audit、exit

implementation scopeはexisting runner production leaf、existing test-only facade
2段、existing proof-local runner test file、すなわち
`crates/mizar-test/src/runner/type_elaboration/source_statement.rs`、
`crates/mizar-test/src/runner/type_elaboration.rs`、
`crates/mizar-test/src/runner.rs`、
`crates/mizar-test/src/runner/tests/type_elaboration/source_proof_local_declaration.rs`
だけ。最初のfacadeはnew crate-private test seamの再exportだけ、次の
facadeはexisting included runner test moduleへのimportだけをownし、production
dispatchやpublic APIは変更しない。new runner tests exact 4件がfull output/debug、
parser/resolver/local/all-node mutation、near-missとB3N/B3M1/mixed isolation、
checker/active semantic effect 0をcoverする。checker testsは`482`、runnerは
`536 -> 540` projected。runner production pathsは37のままline/test-list/
content hashを再測定する。

`.miz`、sidecar、expectation、trace row/status/backlink、metadata、Cargo、public
diagnostic、case、requirement、pass/fail、active-stage、type-coverage、CLI outputは
変更しない。coverage auditは`269CP -> 269C` ownershipだけを記録しcreditは0、
Chapters 15/16はpartial、existing trace hashは不変。

exitはsynchronized EN/JA、specification review **NO FINDINGS**、docs-only
verification/commit、fresh preflight、exact private implementation、independent
test/implementation/source-doc review **NO FINDINGS**、uncapped 90/100以上でhard
gate 9件PASS、task-only staging/commit、clean post-commit、protected stash不変、
上のbinding-only contractに限定したTask-269C自動選択を要求する。

### Implementation closure

runner-private producerはこのprerequisiteだけを実装し、later semantic ownerを
activateしない。frozen node/range/child/recovery全rowに加え、expression root absent、
token side table `0..23`、reserve/theorem shell全field、exact
`parser-signature-v1` theorem payload、definition/contribution provenance、visible
module `y` absenceをauthenticateする。outputはfrozen syntax-free identity、range、
ordinal、local row、fingerprint、debug grammarだけをretainする。

adjacent tests 4件はexact success、全node/side-table/shell/resolver/output/local
guard、exact rejection precedence、near-miss/family/fixture isolation、checker/active
semantic effect 0をcoverする。test-sufficiency/implementation再reviewは
**NO FINDINGS**。checker ABI、source type、Typed/Resolved owner、binding transaction、
goal/fact/proof/acceptance/discharge/downstream IRはactivateしない。

## Checker Task 269C frozen binding-only proof-`let` transaction

### 選択、authority、分類

Task-269CP implementation commit
`4431211d64e0030180852a5d8055edc202a629ba`後のfresh inventoryはTask 269Cだけを
selectする。Chapter 4 Sections 4.1/4.2/4.6はproof-local `let`がenclosing proof
scopeにfresh free-variable binding 1件をintroduceすることを要求する。Chapter 15
Sections 15.2.1/15.10はproof-block localityを要求しsame-scope duplicateを禁止する。
Chapter 16 Sections 16.3.3/16.4.1/16.4.2はproof-block ownerとlocal visibilityを
定める。このauthorityがauthorizeするのはbinding row/scopeだけで、type guard構築、
`thesis`変更、obligation discharge、proof acceptanceではない。

Task 269CPはexact runner-private source/Surface/resolver/local projectionを供給する。
existing reserve bridgeはexact module-level base `BindingEnv`をindependentにprepareでき、
checkerには`BindingDraft::from_local_term`、`BindingKind::LetBinding`、
`BindingTypeSite::Missing`、lexical lookup、one-shot Typed/final ownership patternがある。
missing binding transactionはbounded `source_drift`、focused fail-closed coverageはbounded
`test_gap`。欠けている`LetBinding` source-type admissionとresolver-wide use/capture payloadは
separate `source_drift`で、269Cへmergeすれば`boundary_violation`。worktree clean/protected
stash不変。`origin/main`差はexact commit targetを隠さないreport-only
`repo_metadata_conflict`。

### Exact syntax-free checker ABI

existing checker module `source_proof_local_declaration`へnamed-witness transactionの
extensionではない次のpublic sibling contractを追加する。private fieldにはunchecked
constructor/mutable accessorを置かない。

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalLetBindingHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub lower_fingerprint: String,
    pub theorem_symbol: SymbolId,
    pub theorem_definition: DefinitionId,
    pub contribution: SourceContributionId,
    pub theorem_range: SourceRange,
    pub proof_range: SourceRange,
    pub let_range: SourceRange,
    pub segment_range: SourceRange,
    pub name_range: SourceRange,
    pub source_ordinal: usize,
    pub local: LocalTermBinding,
    pub recovery: SourceProofLocalLetBindingRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceProofLocalLetBindingId(usize);

impl SourceProofLocalLetBindingId {
    pub const fn new(index: usize) -> Self;
    pub const fn index(self) -> usize;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceProofLocalLetBindingRecovery {
    Normal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalLetBinding {
    binding: BindingId,
    binding_context: BindingContextId,
    source_ordinal: usize,
    visible_after_ordinal: usize,
    recovery: SourceProofLocalLetBindingRecovery,
}

impl SourceProofLocalLetBinding {
    pub const fn binding(&self) -> BindingId;
    pub const fn binding_context(&self) -> BindingContextId;
    pub const fn source_ordinal(&self) -> usize;
    pub const fn visible_after_ordinal(&self) -> usize;
    pub const fn recovery(&self) -> SourceProofLocalLetBindingRecovery;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalLetBindingTable {
    rows: Vec<SourceProofLocalLetBinding>,
}

impl SourceProofLocalLetBindingTable {
    pub fn get(
        &self,
        id: SourceProofLocalLetBindingId,
    ) -> Option<&SourceProofLocalLetBinding>;
    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (SourceProofLocalLetBindingId, &SourceProofLocalLetBinding),
    >;
    pub const fn len(&self) -> usize;
    pub const fn is_empty(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceProofLocalLetBindingHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    lower_fingerprint: String,
    theorem_symbol: SymbolId,
    theorem_definition: DefinitionId,
    contribution: SourceContributionId,
    theorem_range: SourceRange,
    proof_range: SourceRange,
    let_range: SourceRange,
    segment_range: SourceRange,
    name_range: SourceRange,
    base_binding_env: BindingEnv,
    base_binding_fingerprint: String,
    binding_env: BindingEnv,
    final_binding_fingerprint: String,
    bindings: SourceProofLocalLetBindingTable,
}

impl SourceProofLocalLetBindingHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn lower_fingerprint(&self) -> &str;
    pub const fn theorem_symbol(&self) -> &SymbolId;
    pub const fn theorem_definition(&self) -> DefinitionId;
    pub const fn contribution(&self) -> SourceContributionId;
    pub const fn theorem_range(&self) -> SourceRange;
    pub const fn proof_range(&self) -> SourceRange;
    pub const fn let_range(&self) -> SourceRange;
    pub const fn segment_range(&self) -> SourceRange;
    pub const fn name_range(&self) -> SourceRange;
    pub const fn base_binding_env(&self) -> &BindingEnv;
    pub fn base_binding_fingerprint(&self) -> &str;
    pub const fn binding_env(&self) -> &BindingEnv;
    pub fn final_binding_fingerprint(&self) -> &str;
    pub const fn bindings(&self) -> &SourceProofLocalLetBindingTable;
    pub fn debug_text(&self) -> String;

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
    ) -> Result<(), SourceProofLocalLetBindingError>;

    pub(crate) fn validate_complete_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        installation_available: bool,
    ) -> Result<(), SourceProofLocalLetBindingError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SourceProofLocalLetBindingProducer;

impl SourceProofLocalLetBindingProducer {
    pub fn build(
        input: SourceProofLocalLetBindingHandoffInput,
        base_binding_env: &BindingEnv,
    ) -> Result<
        SourceProofLocalLetBindingHandoff,
        SourceProofLocalLetBindingError,
    >;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProofLocalLetBindingError {
    InvalidTransaction,
    DependencyMismatch,
    InvalidBaseBindingEnvironment,
    InvalidAggregate,
    InvalidDeclaration {
        binding: SourceProofLocalLetBindingId,
    },
    InvalidBindingEnvironment,
    InvalidInstallation,
}
```

`SourceId`、`SourceRange`、`ModuleId`、`SymbolId`、`DefinitionId`、
`SourceContributionId`、`LocalTermBinding`、`BindingId`、`BindingContextId`、
`BindingEnv`はexisting owner-module typeで、siblingはaliasを定義しない。raw
`SurfaceAst`、syntax node、declaration shell、`SymbolEnv`、source text、type-expression
row、formula、goal、fact、proof、obligationはABIをcrossしない。checker inputには
independent source/Surface fingerprint、type range、type-head fieldを意図的に置かない。
唯一のlower tokenはTask 269CPでfreezeしたcomplete byte-exact
`source-proof-local-let-lower-debug-v1` stringである。このopaque stringはsource/Surface
SHA-256
`7860a3fe5af89063ac6a2b9a4465cac36d26f6d64e892ba6e2c89bcbaaf9763a` /
`1fc35ec18db82efc0968b2f42b08cfaae678184983210cd26f060d45354c7f68`と
`76..79` type evidenceをembedするが、いずれもseparate admitted checker ownershipには
ならない。

exact fail-closed provenanceはtheorem `19..99`、proof `59..98`、`let` `67..80`、
segment `71..79`、name `71..72`、source ordinal1、theorem definition/contribution
index `0/0`、local `y`、scope `[0]`、declaration `71..72`、visible-after1。theorem
symbolはsupplied module所属でTask-269CP identityをretainする。sibling theorem、second
declaration/segment/name、implicit declaration、`such that`、trailing `by`、nested
proof、recovered node、adjacent 269A/B profileをacceptしない。

### Base、transition、lookup、output

runnerはexisting `extract_builtin_source_reserve_declarations_after_node_guard`と
`SourceReserveDeclarationBridge::prepare_binding_env`からbaseを得て、checker bindingを
source textからfabricate/rescanしない。checkerはexact normal base `1/1/0`を
authenticateする。context0はmodule context、parent/scopeなし、reserved binding0を
own/exposeする。binding0は`x`、`ReservedVariable`、module-owned、declaration/identity
range `8..9`、visible-after0、source type site `14..17`、reserved、uncaptured、
diagnostic-free、normal。source/module identityは一致しdiagnosticはempty。

atomic transitionはexact `1/1/0 -> 2/2/0`。proof context1をowner
`SourceStatement(59..98)`、parent0、proof layer、scope `[0]`、owned `[1]`、visible
`[0,1]`、normalでappendする。binding1は`y`、kind `LetBinding`、
`ResolverLocal([0], ordinal=1, range=71..72)`、owner1、declaration `71..72`、
visible-after1、`BindingTypeSite::Missing`、active、capture/diagnostic empty、normal。
single handoff rowはrow0 -> binding1/context1/source ordinal1/visible-after1/normal。
context0/binding0はbyte-identical。

synthetic same-scope lookup ordinal1はbinding1のexisting forward-reference result、ordinal2は
local binding1を返す。これはtable visibilityだけをvalidateし、source use ordinal2や
use-site/capture rowをclaimしない。handoffはexact base/final binding debug fingerprintを
retainする。deterministic `source-proof-local-let-binding-debug-v1`はcomplete frozen
transactionをprintし、全fieldをpublish前validationへ含める。

validationはtransactionalで、first failure classは次のstable orderで決まる。(1)
source/module transaction identity、(2) byte-exact lower tokenとtheorem
symbol/module/FQN、definition/contribution `0/0`、全5 range、(3) exact base
`BindingEnv`、(4) dense output row 1件、(5) local spelling/scope/ordinal/range/recoveryと
row link、(6) reconstructed final `BindingEnv`、binding fingerprint 2件、lookup result
2件、(7) Typed/final owner availability。errorは順に`InvalidTransaction`、
`DependencyMismatch`、`InvalidBaseBindingEnvironment`、`InvalidAggregate`、
`InvalidDeclaration { binding: SourceProofLocalLetBindingId(0) }`、
`InvalidBindingEnvironment`、`InvalidInstallation`。`build`はphase 1--6を実行する。
public inputはsingularなのでaggregate corruptionはreplay時だけ可能。

exact `Display` textは次の通り。

| variant | exact text |
|---|---|
| `InvalidTransaction` | `source proof-local let-binding transaction is invalid` |
| `DependencyMismatch` | `source proof-local let-binding dependency mismatch` |
| `InvalidBaseBindingEnvironment` | `source proof-local let-binding base binding environment is invalid` |
| `InvalidAggregate` | `source proof-local let-binding aggregate is invalid` |
| `InvalidDeclaration { binding }` | `source proof-local let-binding <binding.index()> is invalid` |
| `InvalidBindingEnvironment` | `source proof-local let-binding binding environment is invalid` |
| `InvalidInstallation` | `source proof-local let-binding installation is invalid` |

errorは`std::error::Error`をimplementする。failureはpartial context/binding/table row/
fingerprint/Typed owner/final ownerをpublishしない。

exact stable debug grammarは次の通り。

```text
source-proof-local-let-binding-debug-v1
module: <package>::<path>
lower-fingerprint: <quoted Task-269CP debug bytes>
theorem symbol=<quoted-fqn> definition=0 contribution=0 range=19..99 proof=59..98
let range=67..80 segment=71..79 name=71..72 source_ordinal=1
base-binding-fingerprint: <quoted BindingEnv debug bytes>
binding#0 binding=1 context=1 source_ordinal=1 visible_after=1 recovery=normal
final-binding-fingerprint: <quoted BindingEnv debug bytes>
```

rowはdense-ID order、recovery spellingは`normal`だけ。lower/binding fingerprintはRust
string `Debug`（`{:?}`）を使い、double quoteとLF/quote/backslash/control characterの
standard escapeを含む。symbol FQNも`{:?}`を使う。lineはshown order、blank lineなし、
final LF 1件。existing Task-269A/B/empty debug byteは不変。

### Typed/final ownershipとexclusion

`TypedAst`はprivate optional `source_proof_local_let_binding` fieldと次のmethodsだけを
追加し、`TypedAstParts`にはreplacement fieldを追加しない。

```rust
pub const fn source_proof_local_let_binding(
    &self,
) -> Option<&SourceProofLocalLetBindingHandoff>;

pub fn with_source_proof_local_let_binding(
    self,
    handoff: SourceProofLocalLetBindingHandoff,
) -> Result<Self, TypedAstError>;
```

`TypedAstError`はunit variant `InvalidSourceProofLocalLetBinding`だけを追加し、exact textは
`typed AST source proof-local let-binding handoff is inconsistent`。admitted baseは
otherwise-empty `TypedAst`: resolved root/typed node/existing source handoff全family
（Task-269A/Bを含む）なし、context/type/fact/coercion/initial-obligation/diagnostic table
empty。installationはphase 1--7をreplayし、node/link/source type/fact/coercion/
obligation/diagnosticを追加せず1回だけpublishする。duplicate/orphan/stale/cross-family/
partial/semantic-coexistenceはnew AST errorへmapしinput valueは不変。

`ResolvedTypedAst`はsame exact read-only getter signatureを追加し、same empty semantic/node
profileへのvalidation replay後だけhandoffをclone-preserveする。`ResolvedTypedAstInputs`に
replacement pathを追加しない。`ResolvedTypedAstError`はunit variant
`InvalidSourceProofLocalLetBinding`だけを追加し、exact textは
`resolved typed AST source proof-local let-binding handoff is inconsistent`。deterministic
debugはexisting Task-269A/B slotの後のproof-local handoff slotにnew blockをappendし、mutual
exclusionによりnonemptyは1件だけ。expression metadata/candidate/overload/cluster/formula/
statement semantic/proof/terminal goal/initial obligation/diagnostic rowは追加しない。
existing Task-269A/B/empty debug byteは不変。

269Cは`LetBinding` source-type admission、bare-`set` type checking、type guard/FOL
relativization、`such that`/`by`、same-scope duplicate diagnostic、actual later-use/capture
extraction、formula/thesis/goal transition、fact、proof/discharge/acceptance、Core/CFG/VC/ATP、
active corpus dispatchをexcludeする。type siteを`Missing`から変更する前にseparate
documentation prerequisiteでmissing source-type ownerをfreezeする。

### Implementation/test scope、measurement、audit impact、exit

later implementationが変更できるexisting Rust fileはexactly 7件。

1. `crates/mizar-checker/src/source_proof_local_declaration.rs`
2. `crates/mizar-checker/src/typed_ast.rs`
3. `crates/mizar-checker/src/resolved_typed_ast.rs`
4. `crates/mizar-test/src/runner/type_elaboration/source_proof_local_declaration.rs`
5. `crates/mizar-test/src/runner/type_elaboration.rs`（test-only facade）
6. `crates/mizar-test/src/runner.rs`（test-only root facade）
7. `crates/mizar-test/src/runner/tests/type_elaboration/source_proof_local_declaration.rs`

new module/path、active dispatch、public runner route、checker syntax dependency、parser/resolver
source、Cargo、fixture、sidecar、expectation、trace metadata、diagnostic mappingは禁止。
checker tests 4件はexact producer/output/debug/lookup、complete input/base/output corruption/
error precedence、Typed/final one-shot/cross-family/rollback/replay、missing-type/empty-semantic
preservationをcover。runner tests 4件はexact Task-269CP-to-checker transaction、lower/base/
checker corruption、near-miss/family/public-route isolation、active/semantic effect 0をcover。
Task-269CP testsとTask-269A/B byteは不変。

docs baselineはchecker/runner library `482/540`。raw/normalized test-list SHA-256はchecker
`c89028b747ba4a551d74a2f6cc9c79e3520cc79ad0f019e18a2a4c123d52288c` /
`da1022d491be404da68e41c77b800f7d0ca65765e397d28489e40d961ab453a2`、runner
`8b9a2b9ea4aad3c6ed0b6eae32a0285d6a9fe1b5389dcc31ebc7adb872317522` /
`a8955748da86930f3e2165637e170d68c77756cbc03f3ff38b3f8de0d21cbc50`。
implementationは`486/544`をproject。checker productionは30 paths/165,219 lines、
path/content hash
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`1fb5ea739c810ff66ed551b359ffa7cbb26265c0057fa18f5128ee5966bad958`。
runnerは37 paths/71,194 lines、
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d` /
`4dcfc69a867dea5c12457d94825493a8a48e4fd5ac7b91d86412371ac25f6b03`。
implementation後lines/content hashを再測定しpath hashは不変でなければならない。

broad `.miz`/expectation/covered diagnostic rowはprivate dormant sliceがbroad caseを
execute/acceptしないため不変。`spec_coverage_audit.md`はTask 269Cをzero-credit binding
ownerとして記録しseparate source-type prerequisiteをnameする。trace manifestはbyte-
identical。exitはEN/JA sync、spec review **NO FINDINGS**、docs-only hard gate 9件とuncapped
`>=90/100`、exact docs-only stage/commit、fresh preflight、exact 7-file implementation、
separate test/implementation/source-doc review **NO FINDINGS**、full verification/count/hash、
final hard gate 9件とuncapped `>=90/100`、task-only commit、clean post-commit、protected
stash不変後のnext dependency-ready task選択を要求する。

### Documentation prerequisite review / verification

first read-only specification reviewはhigh `design_drift` 2件を報告した。claimed exact
checker APIがunderspecifiedで、Task-269CP selector-fingerprint boundaryとproposed checker
fieldが矛盾していた。contractは全Rust field/signature、validation/error/debug byte、
Typed/final owner APIをfreezeし、runner boundaryをcrossできるのをopaque complete lower
debug fingerprint 1件だけへ限定した。re-reviewは**NO FINDINGS**、blocking `spec_gap`
なし、remaining `boundary_violation`なし。

diffは`doc/design` 38 filesだけ。checker/runner lint policy `15/15` / `14/14`、metadata
`137/137`、`cargo fmt --all --check`、warnings-denied workspace Clippy、full `cargo test`、
Cargo metadata、`git diff --check`はPASS。libraryはchecker/runner `482/540`、raw/
normalized test-list hashは
`c89028b747ba4a551d74a2f6cc9c79e3520cc79ad0f019e18a2a4c123d52288c` /
`da1022d491be404da68e41c77b800f7d0ca65765e397d28489e40d961ab453a2`と
`8b9a2b9ea4aad3c6ed0b6eae32a0285d6a9fe1b5389dcc31ebc7adb872317522` /
`a8955748da86930f3e2165637e170d68c77756cbc03f3ff38b3f8de0d21cbc50`のまま。
productionはfrozen path/content hashのchecker `30/165219`、runner `37/71194`不変。

CLI 5件のplan/parse/declaration/type/proof stdout hashは
`700f4bf503783742cefd8004fa095675b7476d46e9a3a6dd439916d237eb6718` /
`a8a7aa639d2ebc65eddc923c7e9369ea5637d50e935f808600f446da1bfbda56` /
`71e83ba0b20d4015e07b3bd2c0c4db2837b6151d1251812caed7954530d53c74` /
`4b2c7bd5ec3cc56e5672fb351126126230ec84fba9bd2bd9049a516d378fab7f` /
`ccf3d2d4d0a3755e00989d97af369a7c560302f76798d0a185d57ec3891e8450`を再現。
corpus/requirements `428/395`、pass/fail `235/193`、active `101/7/205/1`、type
coverage `259=247+12`、warnings/errors `23/0`、trace SHA-256
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`は不変。

narrowed contractのrepeated specification reviewは**NO FINDINGS**。docs-only
preflightはfocused 269CP/C/CT、lint `15/14`、metadata137、Cargo metadata、fmt、
warnings-denied Clippy、workspace全test、CLI 5件、whitespaceをPASS。libraries
`490/548`、production `30/168322`/`37/71647`、EN canonicalに記録したproduction/
test-list/CLI/fixture/trace hashは全てexactに再現した。

repeated source/docs consistency/final-quality reviewは**NO FINDINGS**。hard gate 9件は
score capなし`100/100`で全PASS。exact stage/docs commitはparent-owned。
independent final quality reviewは**NO FINDINGS**、hard gate 9件PASS、score capなし、valid
`100/100`（`20/20/15/15/10/10/5/5`）。exact staging/docs-only commitはparent-owned
next step。

### Task-269C implementation result

commit済みdocumentation prerequisiteを、freeze済み7 source file境界を拡張せず
実装した。checker producerはopaqueなTask-269CP lower fingerprint、exact theorem/
range/resolver provenance、reserve-only `1/1/0` base、1-row declaration table、
exact `2/2/0` final environmentをfreeze済み7-phase順で検証する。missing-type、
active、uncapturedな`LetBinding`を1件だけpublishし、definition-site ordinal 1は
forward、synthetic ordinal 2はbinding 1をresolveする。Typed/final installは
one-shotで全既存sibling familyと排他的、transaction全体をreplayし、nodeまたは
semantic payloadを追加しない。

private dormant runnerは不変のTask-269CP projectionと既存reserve bridgeだけを
consumeする。checker 4件とrunner 4件がexact output、corruption/precedence、
cross-family/rollback/replay、near-miss、semantic emptinessのtest gapを閉じる。
libraryはchecker/runner `486/544`。raw/normalized test-list SHA-256はchecker
`0a4d39c5cad8ee81ee1a9b52fa437a6203202cc783100c275adb1a717fb749f7` /
`2bece131be70bdfd0a3128faa1b83852b774692353c4926f069bafa61d2d7e28`、runner
`fa69bfaa53fb75a2a6ec62b1ac7faf8fc5e5a12693a3840e0e31439eafa156db` /
`717a16f30326b9878949c7158be81eff5f7769c32ceeb19e23de0e569eb7ab4c`。
productionはchecker `30/167058`、runner `37/71412`。path hashは
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`
のまま、content hashは
`d5d6c3bf41176422ffe78b9c612db02ef8eb8550ea080d0c11e90c16d320cb49` /
`bf8c5a242bdc3e8a6809583ef1813138afbb246e41612413d7a7783631bc3cd6`。

parser/resolver、source-type、active dispatch、public runner、fixture、sidecar、
expectation、trace、metadata、Cargo、diagnostic、goal、guard、proof、discharge、
acceptance、fact、Core、CFG、VC ownerは変えていない。corpus/count/CLI/traceは
freeze済みTask-264 baselineを維持する。coverage auditはzero-credit binding
transportだけをcloseし、separate source-type prerequisiteはfresh dependency
selectionへ残す。

Typed/final ownerは比較的大きいhandoffをprivate boxed storageで保持し、freeze済み
by-value installerと`Option<&Handoff>` getter signatureを維持する。この
representation-only choiceはlegacy cross-family testがdefault Rust test-thread
stackをexhaustすることを防ぎ、ownership/validation/debug byte/public semanticsを
変更しない。

### Task-269C implementation review and verification

legacy cross-family crate testがdefault-thread stack exhaustionを検出後、上のbounded
private boxing correctionによりpublic contractを変えず当該testをrestoreした。
repeated test-sufficiency/implementation/source-documentation reviewはすべて
**NO FINDINGS**。focused checker Task-269Cは`4/4`、checker/runner libraryは
`486/486` / `544/544`、lint policyは`15/15` / `14/14`、metadataは`137/137`。
Cargo metadata、`cargo fmt --all --check`、warnings-denied all-target/all-feature
workspace Clippy、full `cargo test --no-fail-fast`、`git diff --check`はPASS。

metadata CLI 5件はexit zeroでfrozen plan/parse/declaration/type/proof stdout hashを
再現し、cases/requirements `428/395`、pass/fail `235/193`、active stage
`101/7/205/1`、type coverage `259=247+12`、warnings/errors `23/0`を報告する。
final production/test-list/trace hashも上記recordを再現。independent final qualityは
**NO FINDINGS**、全9 hard gate PASS、score capなし、valid `100/100`
（`20/20/15/15/10/10/5/5`）。task-only stage/commitとclean post-commit fresh
inventoryだけがparent-owned gateとして残る。

## Task 269CT immutable dependency boundary

269CTは`SourceProofLocalLetBindingHandoff`をby-value consumeし、missing-type `2/2/0`
snapshotとdependency fingerprintを不変にpreserveする。separate source-type compositeがtyped
overlay/type handoffをownし、このmoduleはAPI/source変更なし。syntax rescan、later use/capture、
assumption、goal、fact、proof behavior、active routeはscope外。

## Task 269CT implemented consumer boundary

`source_type.rs`はTask-269C handoffをby-value consumeし、unchanged dependencyとbyte-exact
fingerprintをcompositeへstoreする。Task-269C direct owner/missing type siteはunchangedで、
composite Typed/final profileではempty。このmoduleのsource/API/test変更はなく、later
use/captureと全proof semanticsはdefer。

## Checker Task 269GP frozen isolated proof-`given` syntax-lower prerequisite

### Selection、authority、classification

`c60361977f6c4d832cf4217b85bd9b458c902848`でのfresh inventoryは269GPだけを
selectする。Task 269は未完了で、269A/Bはnamed `take`、269CP/C/CTはisolated
`let` definition-siteとwritten typeだけを実装済み。Task 270はTask 269依存のため
未readyである。`set`はRHSのresolved local-binding captureが必要で、`consider`は
justification subtreeも持つ。source-order上の最小ready formは`given`
syntax-only definition-site projectionである。

authorityはcanonical Chapter 4 §§4.2/4.4/4.6、Chapter 15
§§15.3.3/15.10/15.11.4、Chapter 16 §§16.3.3/16.4/16.5、parser simple-statement
fixture、unchanged broad proof-local gap fixture。broad fixture/sidecar/
expectation/trace backlinkはread-onlyでpositive creditを与えない。

specification reviewはbinding consumerをblockする`spec_gap`を確認した。Chapter 4
§4.6.1は`given` binderを導入statement/formulaへ限定する一方、Chapter 16
§§16.3.3/16.4.2はwitnessをlocal subproof/enclosing blockでavailableとする。Chapter 15
§15.10は`let` scopeだけを定め、矛盾を解消しない。このため269GPはsyntax/range/
provenance transportだけへstrictly narrowし、`LocalTermBinding`、scope path、
visible-after ordinal、condition availability、later-use promiseをpublishしない。
269G/269GTはcanonical scope intentがhuman decisionで整合されるまでblocked。

narrowed task内のmissing contractは`design_drift`、private lower projectionはbounded
`source_drift`、4-test guardはcanonical-derived `test_gap`。binding visibility、
existential fact、Skolem result、label identity、goal change、use/captureの捏造は
`boundary_violation`。
`origin/main...HEAD=0/8`はreport-only `repo_metadata_conflict`、protected stash
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`は範囲外。

### Exact source / Surface / resolver

sole private final-LF sourceは次である。

```mizar
reserve x for set;
theorem FormulaStatementGivenSmoke: thesis proof
  given y being set such that G: thesis;
  thus thesis;
end;
```

129 bytes、SHA-256
`04e54b8ada9af54fde9f937e1bb0f96bd8cf85002b2b57f4d348b11c8eb72a2f`。
normal Surfaceは48 nodes/root47/range `0..128`、expression rootなし、recovery/
diagnosticなし、token `0..24`、snapshot SHA-256
`58ac16a3c75860180a8bec5dc8e87ec8b269fe75715a6d8363f7ef064e3deea8`。
selectorは全node kind/source/range/ordered children/recovery/tokenをauthenticateする。
主要rowはreserve `28/0..18/[0,27,4]`、type head/expression
`31/32/84..87`、segment `33/76..87/[11,12,32]`、condition subtree
`34..37/93..107`、given `38/70..108/[10,33,14,37,19]`、conclusion
`42/111..123`、proof `43/62..127`、theorem `44/19..128`、root47。condition/
label token16/17/conclusion subtreeはselector-onlyでhandoffへ渡さない。

shellはnormal rootのreserve `0/28/0..18`とtheorem `1/44/19..128`だけ。resolverは
public/exported theorem symbol/definition/local-source contribution各1件を持ち、originは
`19..128`、path `[2,1]`、importなし。signatureは
`node=TheoremItem;symbol=theorem;definition=theorem;primary_tokens=theorem FormulaStatementGivenSmoke : thesis proof given y being set such that G : thesis ; thus thesis ; end ;;notation=_;arity=_;roles=FormulaExpression,ProofBlock`
である。parameters/binders/notation/doc/conflict/dependenciesと全other indexesはempty。
resolverは`y`も`G`もsymbol/labelとして公開しないため269GPも捏造しない。

theorem symbolはprimary spelling `FormulaStatementGivenSmoke`、requested
namespace/module、`Public`/`Exported`、contribution0、notationなし、relations empty、上記
exact source/range/path/no-import origin。definition0は同symbol/origin、kind `Theorem`、
`Public`、contribution0、arity/signature deviationなし、全optional/list field empty。
contribution0はrequested module、`LocalSource { source_id }`、anchor `0..18`、symbol
effect `[theorem]`、definition effect `[0]`、その他全effect empty。

private `SourceProofLocalGivenLowerOutput`はsource/module identity/fingerprint、theorem
resolver identity、theorem/proof/given/segment/name/type range、exact token spelling
`y`/`set`、source statement ordinal1だけをretainし、binding-shaped fieldを持たない。
source/Surface/shell/resolver/output/debug fingerprintは独立にfail-closed authenticateする。

### Complete runner-private Rust contract

production leafはcrate-private familyだけを追加し、fieldはprivate、accessorは全て
`pub(in crate::runner)`。`SOURCE_PROOF_LOCAL_GIVEN_TEXT`、`Debug/Clone/PartialEq/Eq`
の`SourceProofLocalGivenLowerOutput`をfreezeする。fieldsは`source_id/module_id`、source/
Surface fingerprint `String`、`theorem_symbol: SymbolId`、definition/contribution、theorem/
proof/given/segment/name/type/type-head ranges、`name_spelling: String`、
`type_spelling: String`、source ordinal。binding fieldはない。
production functionは`(&SurfaceAst, ModuleId, &DeclarationShellSet, &SymbolEnv, &str) ->
Option<Result<SourceProofLocalGivenLowerOutput, String>>`。

read-only implは各field accessorに加え`name_spelling() -> &str`、
`type_spelling() -> &str`、`debug_text() -> String`を`pub(in crate::runner)`で公開する。
exact definition/contribution/ordinalは`0/0/1`、theorem `19..128`、proof `62..127`、
given `70..108`、segment `76..87`、name `76..77`/`"y"`、type/head
`84..87`/`"set"`。debugはmodule、quoted source/Surface hash、theorem
fqn/definition/contribution/range/proof、given/segment/ordinal、name、type/head/spelling/
formの順で、exactly one terminal LFを持つ。name行にscope/visibilityは含めない。

test-only exact enumsは`SourceProofLocalGivenSurfaceMutation`、
`SourceProofLocalGivenLowerMutation`、`SourceProofLocalGivenShellMutation`、
`SourceProofLocalGivenResolverProfileMutation`。全て
`pub(in crate::runner)`かつ`Debug/Clone/Copy/PartialEq/Eq`で、EN canonicalに列挙した
Task269CP-mirroring variantsをexactに持つ。lower variantsはbinding mutationを持たず、
`NameSpelling`と`TypeSpelling`を含む。

5 seamsは`source_proof_local_given_lower_output_with_surface_mutation`、
`source_proof_local_given_lower_output_with_mutation`、
`source_proof_local_given_lower_output_with_shell_mutation`、
`source_proof_local_given_lower_output_with_resolver_profile_mutation`、
`source_proof_local_given_lower_output_with_resolver_mutation`。最後だけ
`impl FnOnce(SymbolEnv) -> SymbolEnv`を取る。selector/source mismatchは`None`、selected
source後は`Some(Err(String))`。precedenceはexact Surface identity、shell count/export/
ordinal profile、resolver module/empty indexes、theorem symbol/definition/contribution、
lower row、debug bytes。

error grammarはEN canonicalの16 exact stringsに固定する。prefixは全て`Task269GP`、
shell field failureだけ`Task269GP declaration shell {ordinal} mismatch`でordinalを
substituteする。whole-environment seamはneutral reconstructionを保持し、missing/
duplicate/wrong-module/cross-profileをrejectする。

### Ownership、tests、impact、exit

269GPはrunner-privateでchecker/public API、`LocalTermBinding`、BindingEnv/source type/arena、Typed/final
owner、statement/formula/fact/diagnostic、active dispatch、fixture/sidecar/expectation/
trace/metadata/Cargo/creditを追加しない。canonical `given` scope contradictionはdirect
binding/type consumerの269G/269GTだけをblockする。condition availability、Skolem/existential、
label/fact、escape、goal/thesis、proof/discharge/acceptance、Core/CFG/VCはTask258/272
またはlater workへdeferする。`set` capture、`consider`、other forms、real later-use、
Task270は新しいblockerを付与せずseparately deferred。

implementation scopeはexisting runner source-statement leaf、test-only facade 2件、
proof-local test fileの4 files。exact test functionは
`source_proof_local_given_lower_projection_is_exact_and_private`、
`source_proof_local_given_lower_rejects_every_corruption_with_frozen_precedence`、
`source_proof_local_given_lower_excludes_near_misses_and_adjacent_families`、
`source_proof_local_given_lower_has_zero_checker_or_semantic_effect`。全variant/token/
node/shell、whole-env、precedence、near miss、adjacent family、zero effect、269CP/C/CT
不変をcoverする。runner `548 -> 552`、checker490、paths `30/37`。
corpus/requirements `428/395`、pass/fail `235/193`、active `101/7/205/1`、type
`259=247+12`、warnings/errors `23/0`、trace SHA-256
`55b754c8c4d0d293a1c44e2ba4b0090f407bba1d429b461b6cb4d6ddca9ca2b3`は不変。

EN/JA sync、repeated spec review **NO FINDINGS**、uncapped 90/100以上のhard gate 9件、
full verification/count/hash、docs-only staging、dedicated commit、fresh preflight後だけ
4-file implementationを開始する。implementationも4種review **NO FINDINGS**、全gate、
exact commit後のfresh inventoryは269Gを開始せず、human-owned canonical scope
contradictionをblockerとしてreportする。

Completion evidence: [central Task-269GP historical contract](../../task_contracts/ja/269GP.md#completion-evidence)。

## Checker Task 269GS canonical proof-`given` scope reconciliation

explicit human authorityが旧contradictionをresolveする。`given`導入変数はdeclarationの
`such that` condition内の出現をbindし、後続statementでは最内のenclosing proof/reasoning
block末尾まで可視。nested child blockはshadowしない限りbindingを継承し、変数はblock
終了後やsibling blockでは可視ではない。本ruleがcoverするのは
witness variableだけ。existing reasoning-block label ruleが`such that` labelを引き続き
governし、condition/fact、existential/Skolem、goal、proof、discharge、acceptance、IR、VC
behaviorは推測しない。

Task269GSはpaired Chapter 4/15/16 specificationとderived recordだけのdocumentation-only
prerequisite。production、fixture、sidecar、expectation、trace、count/status、metadata、
Cargo、public API artifactを変更しない。existing 269GP lower rowはsyntax/range/provenance-
onlyのまま。resolved ruleによりbinding-only Task269Gはdependency-readyとなり、exact scope
ID、visibility ordinal、nested inheritance/shadowing、block restore、spec-derived testsを
後続contractでfreezeする。Task269GTは269G後にseparate ordering、全semantic exclusion維持。

## Checker Task 269G frozen binding-only proof-`given` transaction

Task269GS commit `10bdd041517eb0334df982484b540e2799b106ca`直後のfresh
inventoryはTask269Gだけをselectする。canonical Chapter 4/15/16により、`given`
witnessはdeclarationの`such that` condition内をbindし、後続statementでは最内の
enclosing proof/reasoning block末尾まで可視。nested childはshadowしない限り継承し、
parent/siblingでは不可視、child終了後はouter bindingへrestoreする。本authorityが許す
のはlexical binding/lookupだけである。

Task269GP exact lowerとreserve baseをconsumeする。missing transactionは
`source_drift`、focused binding coverageはTask269GS `test_gap`。condition/label fact、
existential/Skolem fact、goal、proof、discharge、acceptance、IR、VCは作らない。source typeは
separate Task269GTであり、本taskへの混在は`boundary_violation`。lower-stage変更は不要。
source 129 byte/Surface 48 node、source/surface hash
`04e54b8ada9af54fde9f937e1bb0f96bd8cf85002b2b57f4d348b11c8eb72a2f` /
`58ac16a3c75860180a8bec5dc8e87ec8b269fe75715a6d8363f7ef064e3deea8`、
resolver/lower debugはbyte-identical。`.miz`/sidecar/expectation/traceはread-onlyで、
spec-derived Rust scope matrixだけを追加する。

### exact ABI、transaction、scope

`BindingKind`へpublic `GivenWitness`を`LetBinding`直後、`Generated`直前に1 variant追加する。
existing enumのderiveと`#[non_exhaustive]`を維持し、`binding_kind_name`のstable keyは
exact `given_witness`。このkeyはfinal `BindingEnv::debug_text()` fingerprintと、それを
quoteするouter handoff debugのfinal fieldだけへ入る。reserve-only base fingerprintは
byte-identicalでこのkeyを含まない。EN canonical sectionに列挙したexact sibling family、すなわち
`SourceProofLocalGivenBindingHandoffInput`、dense ID/recovery/row/table、immutable handoff、
producer、7 error variantを追加する。EN code blockのderive/attribute、全getter/build/
validator signature、constness、return typeをexact契約とする。read-only getter、dense
iteration、`debug_text`、crate-private installation replayだけを公開し、unchecked
constructor/mutable accessorは持たない。`SurfaceAst`、syntax/shell、`SymbolEnv`、
source text、type expression、condition/formula/goal/fact/proof/obligationはABIをcrossせず、
sole lower tokenはbyte-exact `source-proof-local-given-lower-debug-v1` string。

exact provenanceはtheorem `19..128`、proof `62..127`、given `70..108`、segment
`76..87`、name `76..77`、source ordinal 1、definition/contribution `0/0`、`y`。
runner localはscope `[0]`、declaration `76..77`、visible-after 1。`set@84..87`はopaque
lower内だけでtype siteはTask269GTまで`Missing`。

base `1/1/0`からatomic `2/2/0`へ遷移する。context 1は
`SourceStatement(62..127)`、parent 0、proof、scope `[0]`、owned `[1]`、visible
`[0,1]`、normal。binding 1は`y`/`GivenWitness`、resolver-local
`([0],1,76..77)`、owner 1、visible-after 1、missing type、active、capture/diagnosticなし、
normal。row 0はbinding/context `1/1`、source/visible-after `1/1`。

lookup matrixはdeclaration前 `1/[0]/1`がforward、same-statement `such that`とfirst
later statement `1/[0]/2`がlocal 1、test-derived child `2/[0,0]/2`がlocal 1、parent
`0/[]/2`とtest-derived sibling `4/[1]/2`がunresolved。ordinal-2のproof行2件は別test
intentでcondition/later-use source rowは作らない。test-derived context 2はowner
`Generated("task269g-unshadowed-child")`、parent 1、layer `Block`、scope `[0,0]`、owned
`[]`、visible `[0,1]`、normal。context 3はowner
`Generated("task269g-shadow-child")`、parent 1、layer `Block`、scope `[0,1]`、owned `[2]`、
visible `[0,1,2]`、normal。context 4はowner `Generated("task269g-sibling")`、parent 0、
layer `Block`、scope `[1]`、owned `[]`、visible `[0]`、normal。test-only binding 2はspelling
`y`、`GivenWitness`、`ResolverLocal([0,1], ordinal=2, range=109..110)`、owner 3、declaration
`109..110`、visible-after 2、type `Missing`、status `Active`、capture/diagnostics empty、
normal。synthetic `109..110`はdeterministic table keyだけで第2 source declarationを主張
しない。context 3 ordinal 3はinner 2、child exit後のcontext 1 ordinal 3はouter 1へ
restore。このmatrixはproductionへpublishしない。

failure precedenceはtransaction、lower/theorem/ranges、base、aggregate、local/row、
final env/fingerprint/lookup、Typed/final availability。exact error textはEN canonicalの
`source proof-local given-binding ...` tableどおり、failureはatomic。debugは
`source-proof-local-given-binding-debug-v1`、module、quoted lower、theorem、given/
segment/name/ordinal、quoted base、dense row、quoted finalの順、blankなしfinal LF 1件。

### Typed/final、exclusion、files/tests

`TypedAst`はprivate optional fieldと次のexact methodsだけを追加し、`TypedAstParts`に
replacement fieldは追加しない。

```rust
pub const fn source_proof_local_given_binding(
    &self,
) -> Option<&SourceProofLocalGivenBindingHandoff>;

pub fn with_source_proof_local_given_binding(
    self,
    handoff: SourceProofLocalGivenBindingHandoff,
) -> Result<Self, TypedAstError>;
```

installerはhandoffをby-value consumeするone-shot。
`InvalidSourceProofLocalGivenBinding` textは
`typed AST source proof-local given-binding handoff is inconsistent`。
otherwise-empty profileだけにinstallし、semantic table/node/other handoffを追加しない。
`ResolvedTypedAst`は次のexact read-only getterを追加し、validation replay後だけcloneする。

```rust
pub const fn source_proof_local_given_binding(
    &self,
) -> Option<&SourceProofLocalGivenBindingHandoff>;
```

error textは
`resolved typed AST source proof-local given-binding handoff is inconsistent`。debugは
existing let binding/type後のproof-local slotでcross-family mutually exclusive。

condition/label/fact、type guard/source type、goal/thesis、proof/discharge/acceptance、
Core/CFG/VC/ATP、diagnostic mapping、active dispatch/corpus、source use/captureは禁止。
Task269GTだけがhandoffをby-value consumeして`set@84..87`をadmitできる。multi-segment
`given`、`consider`、free-witness export、Task270はseparate。

implementation write scopeはexact 8 Rust file:

1. `crates/mizar-checker/src/binding_env.rs`;
2. `crates/mizar-checker/src/source_proof_local_declaration.rs`;
3. `crates/mizar-checker/src/typed_ast.rs`;
4. `crates/mizar-checker/src/resolved_typed_ast.rs`;
5. `crates/mizar-test/src/runner/type_elaboration/source_proof_local_declaration.rs`;
6. `crates/mizar-test/src/runner/type_elaboration.rs`;
7. `crates/mizar-test/src/runner.rs`;
8. `crates/mizar-test/src/runner/tests/type_elaboration/source_proof_local_declaration.rs`。

checker exact testsは
`source_proof_local_given_binding_builds_exact_scope_transaction`、
`source_proof_local_given_binding_rejects_corruption_with_stable_precedence`、
`source_proof_local_given_binding_typed_and_resolved_ownership_is_atomic`、
`source_proof_local_given_binding_scope_matrix_is_lexical_and_semantically_empty`。
runner exact testsは
`task269g_exact_given_binding_transaction_debug_and_lookup_are_stable`、
`task269g_lower_base_and_checker_corruption_fail_closed`、
`task269g_typed_and_resolved_owners_are_one_shot_and_semantically_empty`、
`task269g_near_miss_neighbor_and_active_routes_remain_isolated`。

checker corruption seamはwrong source/module/cross-family、wrong lower/theorem/range、wrong
base、truncated aggregate、mutated local/row、wrong final fingerprint/lookup、unavailable/
duplicate/cross-family/rollback replayを分離し、7 error variantとcombined-corruptionの
first-error precedenceを全assertする。runner corruption enumはexact `None`、
`WrongLowerFingerprint`、`EmptyBase`、`WrongTheoremRange`、`WrongProofRange`、
`WrongGivenRange`、`WrongSegmentRange`、`WrongNameRange`、`WrongLocalSpelling`、
`WrongLocalScope`、`WrongLocalRange`、`WrongLocalVisibleAfter`、`WrongSourceOrdinal`。
Typed/Resolvedはinitial/duplicate/cross-family/rollback/post-build mutation replayをcoverする。
libraryは
`490/552 -> 494/556`、production pathは`30/37`不変。docs baseline line
`168322/72916`、path/content hashはchecker
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`4d0c793a47dac672e5f395c9c2b9e7c9274b5d776b54870888ba5c918f751dc2`、runner
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d` /
`532d96defde8f63fa821a4f619c21699069eed19c8f48d50be1f1516be0dac63`。
implementation後はchecker/runner双方のraw/normalized test-list hashとproduction
line/contentを再計測し、path hashは固定。上記値はdocs prerequisite baselineでは不変。
corpus/trace/CLI/countは不変、audit creditはprivate lexical-binding Rust coverageだけ。

docs prerequisiteはEN/JA sync、spec review **NO FINDINGS**、hard gate 9件uncapped
`>=90/100`、exact docs commitを要求。fresh lower preflight後exact 8-file implementation、
test/implementation/source-doc review **NO FINDINGS**、full verification/count/hash、final
gate/score、task-only commit、clean inventory、stash不変を満たしてTask269GTを選ぶ。

### Task-269G implementation closure

exact checker transaction、boxed Typed/final ownership、private dormant runner consumerをfrozen
Rust 8 filesへ実装した。producerはunchanged Task269GP lowerとreserve-only baseをauthenticateし、
dense `GivenWitness` row 1件、`1/1/0 -> 2/2/0` environment、
`BindingTypeSite::Missing`を保持。checker scope matrixはblock inheritance、shadow、restore、
parent/sibling exclusionを証明し、validation/cross-family failureはpublication前にrollback。

checker/runner exact 4+4 testsでlibraryは`494/556`。raw/normalized list hashはchecker
`ce299dfafb8db5d5c27cb9e271dd77d08a09b45a7323d0efc17790e0d104a984` /
`6d8f1938b05118e129f8d0942bd7af77914435b6b45282bd46e636132891d4cb`、runner
`194b2884a9d933823e0d06b24460cd510fd9d16fbd6823b9e13584779acd1f03` /
`728a5b688c19acc42d66a9c2f5c13ad67d795949ec88a2d877b917c9607d80e8`。
productionはchecker `30/169847`、runner `37/73118`、path hashは
`c89f43f6abebf7ebeb3ac9394ecd8ea3186ad28934c75526d2cc0b85a66ebad5` /
`1f9e2c9c6589412d832eb92015d913c1b2e0f1309cba9c5c991e08b04d67a73d`、
content hashは`e47862eebdb59b576160d4b64ab390549d91daecd69fd34f8bcfbc2952d6ca96` /
`2cae769737fdee4560ab1d1bca81f10d900ff8a1d9824aba720806f84e802711`。

`.miz`/sidecar/expectation/trace/metadata/Cargo/parser/resolver/active dispatch/diagnostic/
condition/fact/source type/goal/proof/discharge/acceptance/initial obligation/Core/CFG/VC ownerは
不変。corpus/count/CLI/traceはfrozen Task264 baseline。test-sufficiency/implementation/source-
docs/final-quality reviewは**NO FINDINGS**。focused/crate/workspace/lint/metadata/fmt/Clippy/
CLI/count/hash/whitespaceはPASS、hard gate 9件はcapなし`100/100`
(`20/20/15/15/10/10/5/5`)。parent-owned staging/commit/fresh inventory後にseparate
source-type-only Task269GTへ進む。

## Checker Task 269GT frozen Given-type consumer

Task269GTは`SourceProofLocalGivenBindingHandoff`をby-value consumeしlower/binding fingerprintを
preserve、new `source_type.rs` composite内でbinding 1へexact `set@84..87`だけoverlay。
Given binding ABI/scope matrix/debug bytesは不変。condition/fact/use/capture/free-export/proof
semanticsはconsumer外。

Completion evidence: [central Task-269GT historical contract](../../task_contracts/ja/269GT.md#completion-evidence)。

## Checker Task 269GUP frozen use-profile binding prerequisite

Task269GUPはlater-use前のbinding profileだけをauthenticateする。authorityはSpec 4.6.1、
15.3.3、15.10、16.3.3、16.4.2とhuman-confirmed enclosing-block rule。128-byte siblingは
distinct source transactionで、自身の`BindingEnv`内にnew checker-local `BindingId(1)`をderiveし、
old Task269G bindingとのobject identityをclaimしない。provenanceは
`BinderIdentity::ResolverLocal([0],1,76..77)`、scope ruleはchecker lookupがauthenticate。
resolver API追加なし。

accepted sourceは`thus thesis;`だけを`thus y = y;`へ変えたexact 128 bytes、final LF 1個、
source SHA-256 `ec15ded78ae96022840a8419a85d74643de3b37337e9a202cbda77ee97aa7c01`、
54-node Surface SHA-256
`c64297ce72e380a2e4146276966e085d780f8b38f2528d5abaa440a50c67db6d`。
root 53、token `0..26`、expression root/diagnostic/recoveryなし。reserve `0..18`、theorem
`19..127`、formula `55..61`、proof `62..126`、Given `70..108`、name `76..77`、type
`84..87`、conclusion `111..122`。両later `y`、equality/conclusion/condition/label/formula/proofは
selector-only exclusionで、GUPはterm/later-use rowをpublishしない。

### Exact lower / public binding ABI

旧Task269GP/G/GTはexactのままnew sourceをreject。runner-private
`SOURCE_PROOF_LOCAL_GIVEN_USE_TEXT`、`SourceProofLocalGivenUseLowerOutput`、lower functionを追加。
output fields/getter/constness/signatureはEN canonical exact blockどおり。unique debug headerは
`source-proof-local-given-use-lower-debug-v1`、source/Surface SHA、theorem `19..127`、proof
`62..126`、Given `70..108`、segment/name/type `76..87`/`76..77`/`84..87`をbyte-exact render。
全54 node/token identityをSHAとrole rows 30..53でauthenticateし、term/predicate/conclusionは
selector-only。test-only Surface/Lower/Shell/ResolverProfile mutation enumと5 lower seamsはENの
exact variant/name/signature、binding route mutationは`None`、lower/base/range/local/ordinalの全
G-style variants。literal route signature/return、None/Some semantics、GUPT by-value seam、lower/
route/producer/handoff別precedenceはEN canonicalどおり。private errorはEN complete listとbyte-identical。

checker `source_proof_local_declaration.rs`はexact public
`SourceProofLocalGivenUseBindingHandoffInput/Handoff/Producer/Error`だけを追加。input field/type/order、
producer signature、handoff private field/getter/constness/validatorはEN Rust blockどおり。existing
`SourceProofLocalGivenBindingTable`/row/recoveryをreuse。errorはnon-exhaustive standard traitsで
`InvalidTransaction`、`DependencyMismatch`、`InvalidBaseBindingEnvironment`、`InvalidAggregate`、
`InvalidDeclaration { binding }`、`InvalidBindingEnvironment`のprecedence。Typed ownerがないため
installation variantなし。debug headerは`source-proof-local-given-use-binding-debug-v1`で
Task269G grammar/new ranges/unique lower fingerprintをexact使用。
exact DisplayはENに列挙したtransaction/dependency/base/aggregate/indexed declaration/final-env
6 strings。debug grammarもEN code blockのlabel/order/quote/final LFをbyte-exact使用。

### Exact binding / lookup

base exact `1/1/0`からfinal exact `2/2/0`。proof context 1 owner `62..126`、parent 0、scope
`[0]`、bindings `[1]`、visible `[0,1]`。binding 1はactive normal `GivenWitness`、owner 1、
declaration `76..77`、visible-after 1、type `Missing`、identity `([0],1,76..77)`、capture/
diagnostic empty。ordinal 1 forward、ordinal 2 local B1。child inheritance、inner shadow、restore、
parent/sibling exclusionもtestするがsource occurrenceはpublishしない。

### Scope / tests / impact / exit

Rust scopeはchecker `source_proof_local_declaration.rs`; runner `source_statement.rs`、
`source_proof_local_declaration.rs`、`type_elaboration.rs`、`runner.rs`、existing testのexact 6 files。
docs stageはpaired EN/JA 40 files + global ledger 2 files。EN canonicalに列挙したchecker/runner各4 exact testsで全input/
fingerprint/corruption/precedence/lookup/isolationをcoverし`498 -> 502` / `560 -> 564`。
source type、term/reference、Typed/final、arena、Task252 allowlist、fixture/trace/semanticsは不変。
全review NO FINDINGS、9 gates uncapped 90/100以上、exact staging、separate docs/implementation
commit後GUPTをfresh inventory。GU/capture/Task270はdefer。
Completion evidence: [central Task-269GUP historical contract](../../task_contracts/ja/269GUP.md#completion-evidence)。

## Task 269GUPT frozen dependency consumer

public GUP binding handoffは`SourceProofLocalGivenUseTypeProducer`だけがby-value consumeする。unchanged lower seamはauthenticated `84..87`だけに使い、128-byte selector、54-node Surface、lower fingerprint、resolver provenance、binding rows/lookup/public ABIは変更しない。complete dependency debugをfingerprint化し、later identifierの最初のconsumerは269GUのまま。

Completion evidence: [central Task-269GUPT historical contract](../../task_contracts/ja/269GUPT.md#completion-evidence)。

## Task 269GU binding dependency凍結

GUはcommitted GUPT compositeをby valueでconsumeし、immutable typed BindingEnv
だけを使う。GUP lower/source/shell/resolver fingerprint、declaration row、context、
scope `[0]`、visibility ordinal 1、public APIは不変。later use 2件のordinal 2は
`source_term.rs`がderiveし、本moduleはoccurrence/reference/capture/fact/proof
ownerを追加しない。

Completion evidence: [central Task-269GU historical contract](../../task_contracts/ja/269GU.md#completion-evidence)。

## Task 269GCP frozen Given-condition lower prerequisite

cleanなpost-GU HEAD `998dc104957d47e2707f4a8292d2002f1c5beb2d`のfresh
inventoryは、`given` witnessを同じdeclaration condition内で使うrunner-private
lower prerequisiteだけを選択する。canonical Chapters 4 §4.6.1、15
§§15.3.3/15.10、16 §§16.3.3/16.4.2がこのoccurrenceを明示的にbindする。
existing parser/broad proof-local fixtureはsyntax reachabilityのみで該当useなし。
missing exact profileは`source_drift`/`test_gap`、本contractが`design_drift`を
修復し、blocking `spec_gap`はない。

exact final-LF sourceは134 bytes:

```mizar
reserve x for set;
theorem ProofLocalGivenConditionUseSmoke: thesis proof
  given y being set such that G: y = y;
  thus thesis;
end;
```

source SHA-256は
`2c2d767a0654670412b377bdcc6c5970ecec05b41c02aa754766320927bc6aad`。
read-only frontendはdiagnostic 0、54-node/root 53、root range `0..133`、
Surface snapshot SHA-256
`49d46d5f24338772e6e968f12c2216a8957b35242474132690db843b510b430f`。
token node 0--26、reserve structural 27--30、theorem thesis 31--32、Given type/
segment 33--35、condition term/reference 36--39 (`107..108`/`111..112`)、
equality/formula/proposition/condition 40--43、Given 44、final thesis/conclusion
45--48、proof 49、theorem 50、item-list/compilation/root 51--53。retained rangeは
theorem `19..133`、proof `68..132`、Given `76..113`、segment `82..93`、name
`82..83`、bare builtin `set` type/head `90..93`、source ordinal 1。condition
reference 2件はselector-authenticatedだがGCPはpublishしない。

resolverはreserve shell 0/node30/`0..18`とtheorem shell 1/node50/`19..133`の
exact 2件、export shell 0をfreeze。public/exported local theorem symbol 1、theorem
definition 1、`0..18` contribution 1、origin `19..133` path `[2,1]`。import、
export、label、overload、registration、lexical summary、namespace edge、dependency、
module summary、relation、parameter、binder、diagnostic、他effectはempty。
opaque signature schemaはexact `parser-signature-v1`、payloadは次の1-line byte
stringである。

```text
node=TheoremItem;symbol=theorem;definition=theorem;primary_tokens=theorem ProofLocalGivenConditionUseSmoke : thesis proof given y being set such that G : y = y ; thus thesis ; end ;;notation=_;arity=_;roles=FormulaExpression,ProofBlock
```

`y`をmodule `SymbolId`へ昇格しない。

implementation scopeはexisting runner 4 files (`runner.rs`、
`runner/type_elaboration.rs`、`runner/type_elaboration/source_statement.rs`、
`runner/tests/type_elaboration/source_proof_local_declaration.rs`)だけ。private
`SourceProofLocalGivenConditionLowerOutput`、exact Surface/lower/shell/resolver
mutation、dormant production-private base 1件と`#[cfg(test)]` mutation seam 5件、
runner test 4件を追加する。validation orderはSurface、
shell、resolver inventory、theorem symbol/definition/contribution、lower row、
exact debug。selector mismatchは`None`、selected failureは`Some(Err(_))`。

checker public API、BindingEnv、type/term/reference、condition/formula/fact、label
lifetime、existential/Skolem、guard/assume、capture/export result、goal、initial
obligation、proof/discharge/acceptance、Core/CFG/VC、Typed/Resolved owner、dispatch、
diagnostic、fixture/sidecar/expectation/trace/metadata/active creditは全禁止。
GCP後はdistinct by-value binding Task 269GC、type GCT、condition use GCUの順。
existing GUP/GUPT/GU validatorを緩めずhigher ownerでbindingを再構築しない。
descendant use、Task-272 export enforcement、Task-269 `set` capture、Task270
resolver-local inline identityはseparate follow-up。

private rowのfield型・順序・非`pub` field、
`#[cfg_attr(not(test), allow(dead_code))]`、
`#[derive(Debug, Clone, PartialEq, Eq)]`、`pub(in crate::runner)` visibilityは
ENのexact Rust blockとbyte-for-byte同一である。field/accessor順は`source_id`、
`module_id`、source/Surface fingerprint、theorem symbol/definition/contribution、
theorem/proof/Given/segment/name range、name spelling、type/head range、type
spelling、source ordinal、`debug_text()`。getterの型・visibility・constnessもEN
blockとexactに共有し、4 String getterと`debug_text`だけがnon-constである。
debug headerは
`source-proof-local-given-condition-lower-debug-v1`で、ENのexact 7 data linesと
final LFをbyte-for-byte共有する。

Surface/Lower/Shell/ResolverProfile mutationのexact enum名、全variant順、
`Debug, Clone, Copy, PartialEq, Eq` derive、`pub(in crate::runner)` visibility、
`cfg_attr`はENの4 Rust blocksとbyte-for-byte共有する。base
`source_proof_local_given_condition_lower_output`は
`#[cfg_attr(not(test), allow(dead_code))]`、5引数、return
`Option<Result<SourceProofLocalGivenConditionLowerOutput, String>>`。5つの
`#[cfg(test)]` seamはENのexact signaturesを共有し、最後だけ
`impl FnOnce(SymbolEnv) -> SymbolEnv`を追加する。

lower-only private error ABIは次のexact 16 stringsであり、GUPのbinding/base
errors 2件と追加dump/diagnosticを含まない。

```text
Task269GCP exact Surface identity changed after selection
Task269GCP requires exactly two declaration shells
Task269GCP resolver shells unexpectedly export a path
Task269GCP declaration shell {ordinal} mismatch
Task269GCP raw resolver module mismatch
Task269GCP local y already resolves as a module symbol
Task269GCP raw resolver inventory mismatch
Task269GCP requires one exact theorem owner
Task269GCP exact theorem owner provenance mismatch
Task269GCP requires one exact theorem definition
Task269GCP theorem contribution is missing
Task269GCP theorem symbol provenance mismatch
Task269GCP theorem definition provenance mismatch
Task269GCP theorem contribution provenance mismatch
Task269GCP private lower output mismatch
Task269GCP private lower debug grammar mismatch
```

testは全field/node/token/shell/
empty resolver index/effect/symbol-definition-contribution/debug/combined
precedenceをcover。near missはold GP/GUP、`G: thesis`、unlabelled condition、
later-use-only、theorem/witness rename、type/form、recovery、extra item、missing LF、
active route全件。

baselineはlibrary `510/572`、parser/resolver/syntax `226/148/59`、production
`30/176258`/`37/75339`、cases/requirements `428/395`、pass/fail `235/193`、
warnings/errors `23/0`、stages `101/7/205/1`、type `259=247+12`。implementationは
runner `576`、checker test不変。path hashは不変、changed runner count/content/
test-list hashはremeasure。parser/broad fixture/sidecar/trace SHAはEN記載値を
byte-for-byte共有する。

docs prerequisiteはexact 42 Markdown（checker paired record 28、mizar-test
paired record 12、global ledger 2）だけをown。Rust/Cargo/canonical artifact/
expectation/trace TOML/metadataは変更しない。

exitはEN/JA sync、spec review **NO FINDINGS**、docs-only gate 9件uncapped
`>=90/100`、exact Markdown commit。その後fresh preflight、exact 4-file/4-test、
test/implementation/source-doc review **NO FINDINGS**、full verification、全gate、
separate commit、Task269GC fresh inventoryを必須とする。

Completion evidence: [central Task-269GCP historical contract](../../task_contracts/ja/269GCP.md#completion-evidence)。

## Checker Task 269GC frozen Given-condition binding consumer

clean post-GCP HEAD `59eb7de68d83901375883a2a6249796afc6a0de3`のfresh inventoryは
Task269GCだけをselectする。canonical Chapters 4 §4.6.1、15 §§15.3.3/15.10、
16 §§16.3.3/16.4.2とhuman semantic decisionが定めるexact ruleは、`given`
variableが自分のdeclarationの`such that` condition内をbindし、後続statement
では対応する最内proof/reasoning blockの残りとshadowされないdescendantで可視、
parent/sibling/block終了後では不可視、というもの。labelはordinary label scopeを
維持し、condition/fact/proof/discharge/acceptance/goal/guard/obligationの新lifetimeは
作らない。

implemented GCPのexact 134-byte/54-node source、source/Surface SHA、shell/
resolver provenance、rangeはEN contractとbyte-for-byte共有する。GC checker ABIは
complete `source-proof-local-given-condition-lower-debug-v1`とunchanged
reserve-only `BindingEnv`だけをconsumeし、Surface/syntax/shell/SymbolEnv/source
text/type/condition/occurrence IDを受け取らない。missing producer/testは
`source_drift`/`test_gap`、本contractは`design_drift`を修復し、blocking
`spec_gap`なし。origin `0/13`はreport-only `repo_metadata_conflict`。

public sibling familyは
`SourceProofLocalGivenConditionBindingHandoffInput`、
`SourceProofLocalGivenConditionBindingHandoff`、
`SourceProofLocalGivenConditionBindingProducer`、non-exhaustive
`SourceProofLocalGivenConditionBindingError`。field/getter/signature/derive/
constness/error variant/displayはENのexact Rust blocks/tableを共有する。existing
`BindingKind::GivenWitness`とGiven binding ID/recovery/row/tableだけをcommon row
vocabularyとしてreuseし、G/GUP/GC handoffはdistinct。unchecked constructor/
mutable public accessorは追加しない。

dependencyはtheorem `19..133`、proof `68..132`、Given `76..113`、segment
`82..93`、name `82..83`、ordinal 1、definition/contribution `0/0`、spelling
`y`、complete GCP lower fingerprint。runner localはscope `[0]`、declaration
`82..83`、visible-after 1、normal。`set@90..93`、condition reference
`107..108`/`111..112`、condition/formula/labelはopaque lower内だけでGCはpublish
しない。

checkerはsupplied symbolとlower bytesを相互にself-validatingとは扱わず、exact GCP
theorem identityを独立認証する。symbol moduleはtransaction `module_id`、namespaceは
requested module path、primary spellingはexact
`ProofLocalGivenConditionUseSmoke`。module pathへ順に `\\` -> `\\\\`、`:` ->
`\\c`、`|` -> `\\p`、`/` -> `\\s`を適用した値を
`escaped_module_path`とし、required local IDはexactに次のbytesである。

```text
contribution=0:namespace={escaped_module_path}:owner=theorem#1:shell=theorem:kind=theorem:name=ProofLocalGivenConditionUseSmoke:notation=_:arity=_:definition=theorem:registration=_:policy=non-overloadable:slot=non-overloadable:_:theorem:_
```

required FQNはexact
`{module_id.package}::{module_id.path}::{required-local-id}`。dependency validatorは
supplied symbolのmodule/local/FQNをこの独立構築値と照合し、expected complete GCP
lower fingerprintもsupplied FQNでなくrequired FQNから構築する。このため
`theorem_symbol`と`lower_fingerprint`のcoherent mutationも
`DependencyMismatch`であり、checker corruption testはこのoracleを含む。

atomic transitionはexact `1/1/0 -> 2/2/0`。context 1は
`SourceStatement(68..132)`、parent 0、Proof、scope `[0]`、owned `[1]`、visible
`[0,1]`、normal。binding 1は`y`/`GivenWitness`、resolver-local
`([0],1,82..83)`、owner 1、visible-after 1、type Missing、Active、uncaptured、
diagnostic-free、normal。row 0はbinding/context `1/1`、source/visible-after
`1/1`。base context/bindingはbyte-identicalで、`binding_env.rs`やkindは変更しない。

lookup matrixはEN tableをexactに共有する。own `such that`とfirst subsequent
statementは同じcontext/scope/ordinal `1/[0]/2`を使うdistinct test intentであり、
condition/later-use rowは作らない。unshadowed child、shadow child、restore、
parent/sibling exclusionをtest-derived context 2--4/binding 2で検証し、synthetic
`114..115` rowはhandoff/runner/production/source claimへ入れない。validation
precedenceとexact debug grammar/header
`source-proof-local-given-condition-binding-debug-v1`はENと共有する。

TypedAstはold Given-use term slot後にboxed optional
`source_proof_local_given_condition_binding`、getter、consuming one-shot
installerだけを追加し、error
`InvalidSourceProofLocalGivenConditionBinding` / `typed AST source proof-local
given-condition binding handoff is inconsistent`を追加。ResolvedTypedAstもboxed
clone-preserved owner/getterと同名error / `resolved typed AST source proof-local
given-condition binding handoff is inconsistent`だけを追加する。otherwise-empty
profile、all-owner mutual exclusion、complete replayを必須にし、Typed/final node、
semantic table、parts/input pathは追加しない。

Resolved getterのexact signatureはEN Rust blockを共有する。

private runnerは`source_proof_local_given_condition_binding_output`とcfg-test
`_with_mutation`だけ。GCP selector mismatch=`None`、selected failure=
`Some(Err(_))`、successはone-shot Typed/Resolved owner。exact mutationはEN記載の
13 variants、private base errorは`Task269GC exact reserve base extraction
failed` / `Task269GC exact reserve base failed: {error}`。GCP Surface/shell/
resolver validationはduplicateしない。

private output struct/getter、13-variant mutation、base/cfg-test functionのexact
derive/attribute/visibility/signatureはENのRust blockを共有する。output fieldは
privateで`typed_ast()`/`resolved()` const getterだけ。production functionは
`#[allow(dead_code)]`、mutation seamだけ`#[cfg(test)]`である。

implementationはEN記載exact 7 existing Rust files、checker/runner各4 named
tests。checker corruptionはtheorem symbolとlower fingerprintのcoherent mutation
oracleを含む。GCはbinding-onlyでtype/term/reference/condition/formula/fact/label/
Skolem/assume guard/goal/proof/discharge/acceptance/initial obligation/
diagnostic/Core/CFG/VC/ATP/dispatch/corpus/creditをpublishしない。next GCTだけが
bindingをby-value consumeして`set@90..93`をoverlayし、その後GCUだけがcondition
occurrenceをtransport可。descendant/export/capture/Task270はseparate。

docs baselineはlibrary `510/576`、parser/resolver/syntax `226/148/59`、
production `30/176258`/`37/76642`、case/requirement `428/395`、pass/fail
`235/193`、warning/error `23/0`、stage `101/7/205/1`、type `259=247+12`、
projected library `514/580`。全hashはEN baseline/protected listを共有する。
docs prerequisiteはsame exact 42 MarkdownだけでRust/Cargo/spec/.miz/sidecar/
expectation/trace/metadata/count/status/hashを変更しない。

exitはEN/JA sync、spec review **NO FINDINGS**、docs-only hard gates 9件
uncapped `>=90/100`、exact docs commit、fresh preflight、7-file/8-test
implementation、test/implementation/source-doc review **NO FINDINGS**、full
verification、all gates/score、separate commit、clean/stash確認、Task269GCTの
automatic fresh inventory。

Completion evidence: [central Task-269GC historical contract](../../task_contracts/ja/269GC.md#completion-evidence)。

## Task 269GCT frozen Given-condition type consumer

GC implementation commit `8181ae8fc8af0c7028254ad30147b417fbf84611`後の
fresh inventoryはGCTだけをselect。immutable dependencyはcomplete GC handoff：
134-byte/54-node/root-53 identity、range `19..133`/`68..132`/`76..113`/
`82..93`/`82..83`、independent resolver provenance、complete GCP lower
fingerprint、reserve base、final `2/2/0` env、dense Given row 1件、全fingerprint。
GCTは再構築/relax不可。

runnerが追加で読むのはunchanged GCPのtype/head `90..93`、spelling `set`、
`Bare`だけ。frozen two-row input/three-node arenaを構築しGCをby value checkerへ
transferする。new published declaration siteは`set@90..93`だけ。label、condition
`y@107..108`/`111..112`、equality/formula、`thus`、proof closeはstrict subtree
exclusion。

missing producer/owner/testは`source_drift`/`test_gap`、missing contract/stale GC
statusは`design_drift`。canonical artifact変更、condition/fact/proof/obligation、
old GT/GUPT reuse、runner-side ownershipは`boundary_violation`。origin `0/15`は
report-only `repo_metadata_conflict`、blocking `spec_gap`なし。

checker exact testsは
`task269gct_exact_condition_type_composition_is_stable`、
`task269gct_dependency_binding_input_and_arena_corruption_fail_closed`、
`task269gct_typed_and_resolved_ownership_is_atomic`、
`task269gct_generic_neighbor_and_condition_use_routes_remain_isolated`。
runner exact testsは
`task269gct_exact_condition_type_route_is_stable`、
`task269gct_dependency_input_and_arena_corruption_fail_closed`、
`task269gct_typed_and_resolved_owners_are_one_shot_and_semantically_empty`、
`task269gct_near_miss_neighbor_and_active_routes_remain_isolated`。

corruption matrixはwrong source/module、stale dependency fingerprint、全GC error
class/coherent nested mutation、binding wrong type/stale fingerprint/both-row non-
type field、input application count/binding/ordinal/root、expression count/source/
module/site/range/spelling、head site/range/spelling、form/head/recovery/non-empty
argument、arena wrong rootと各nodeのkind/resolved/anchor/children/typing/recovery/
links、post-build source-type shape/fingerprint、4-tier precedenceをcover。
ownershipはduplicate/全sibling both order/rollback/clone/exact final role/public
semantic table empty/node hint・expression metadata rejection。isolationはwrong
label/name/type、missing final LF、old G/GT/GUP/GUPT/GU、proof-let、generic type、
GCP/GC、全active runner routeをcover。

checker 3 filesとproof-local runner leaf/facade 2/test leafのexact 7 filesだけ。
lower selector、fixture/sidecar/expectation/trace/metadata/Cargo/diagnostic/dispatch/
CLI/result/creditは不変。GCTはwritten typeで終了、condition occurrenceはGCU、
semantic/descendant/capture/export/Task270はdefer。

Completion evidence: [central Task-269GCT historical contract](../../task_contracts/ja/269GCT.md#completion-evidence)。

## Task 269GCU frozen given-condition occurrence consumer

private runnerはunchanged GCT routeのinstalled handoffをby-value consumeし、
GCP-authenticated `y@107..108`/`y@111..112`からEN `source_term.md`のexact
2-term/2-reference/0-request inputと6-node arenaだけをconstructする。private
output ABIはnewest GCT patternへexact freezeする：

```rust
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: Task 269GCU is a private dormant runner consumer until activation.
pub(in crate::runner) struct SourceProofLocalGivenConditionUseTermRouteOutput {
    typed_ast: TypedAst,
    resolved: ResolvedTypedAst,
}

impl SourceProofLocalGivenConditionUseTermRouteOutput {
    pub(in crate::runner) const fn typed_ast(&self) -> &TypedAst;
    pub(in crate::runner) const fn resolved(&self) -> &ResolvedTypedAst;
}
```

fieldはprivateでmutable/consuming accessorなし。mutation enumは以下のattribute
とrationaleをexactに持つ。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; other variants are private Task-269GCU corruption seams.
pub(in crate::runner) enum SourceProofLocalGivenConditionUseTermRouteMutation {
    None,
    WrongDependencyModule,
    WrongTermRange,
    WrongReferenceBinding,
    WrongArenaRoot,
    WrongArenaKind,
}
```

production/test seamは
`source_proof_local_given_condition_use_term_output` /
`..._output_with_mutation`、returnは
`Option<Result<SourceProofLocalGivenConditionUseTermRouteOutput, String>>`。
exact checker/runner各4 testsと7-file scopeはENと同じ。parser/resolver/lower、
fixture/sidecar/expectation/trace/metadata/Cargo/diagnostic/dispatchは変更しない。
label/equality/formula/condition/Given/thus/proof、semantic/descendant/export/
capture/Task270はexcluded。

Completion evidence: [central Task-269GCU historical contract](../../task_contracts/ja/269GCU.md#completion-evidence)。

## Task 269SDP descendant/set lower境界

exact 180-byte sourceとparser/resolver provenanceだけを
`SourceProofLocalGivenDescendantSetLowerOutput`へ保持する。Given、now、
`set z = y`、`set q = z`はrange/spelling syntaxであり、binding/context/
term/capture/fact/proof semanticsではない。`CaptureSmoke`もcreditではない。

Ch.4/15の`set`意味論矛盾はlower-only SDPには影響しないが、closure/
capture実装をblockする。SDP後はGiven-plus-child-contextだけを先に分離し、
occurrenceとcaptureを混在させない。実装scopeは
`runner/type_elaboration/source_statement.rs`、`runner/type_elaboration.rs`、
`runner.rs`、`runner/tests/type_elaboration/source_proof_local_declaration.rs`
のprivate runner 4 filesだけ。exact testsは
`task269sdp_exact_descendant_set_lower_projection_is_stable`、
`task269sdp_surface_lower_and_subtree_corruption_fail_closed`、
`task269sdp_resolver_shell_and_precedence_corruption_fail_closed`、
`task269sdp_near_miss_and_active_routes_remain_isolated`の4件である。
full range/resolver signatureはcrate plan、literal debugとtype-for-type ABIは
以下の本owner文書がcanonicalに保持する。

### Task 269SDP exact private lower ABI

EN canonicalのexact Rust blockと同一に、runner-private rowは
`SourceProofLocalGivenDescendantSetLowerRow` 2件を固定長arrayとして持つ
`SourceProofLocalGivenDescendantSetLowerOutput`である。row field順はstatement/
equating/name range、name spelling、RHS range/spelling、source ordinal。output
field順はsource/module、source/Surface fingerprint、theorem symbol/definition/
contribution、theorem/proof、Given/segment/name/type、Given ordinal、Now、Set
rows 2件、inner/outer conclusionである。全fieldはnon-`pub`、型visibilityは
`pub(in crate::runner)`、mutable/consuming accessorはない。attributes/derive、
getterの型・順序・constnessもEN blockとexactに共有する。

row値はGiven `81..99`、segment `87..98`、name `87..88`/`"y"`、type/head
`95..98`/`"set"`、ordinal 1、Now `102..159`。Set 0はstatement
`110..120`、Equating `114..119`、name `114..115`/`"z"`、RHS
`118..119`/`"y"`、ordinal 0。Set 1は`125..135`、`129..134`、
`129..130`/`"q"`、`133..134`/`"z"`、ordinal 1。conclusionはinner
`140..152`、outer `162..174`である。

debugはheader
`source-proof-local-given-descendant-set-lower-debug-v1`に続くEN canonicalの
module、2 fingerprint、theorem、Given 3行、Now、各Set 2行、conclusion行を
exact separator/orderで出力し、final LFはexactly one。`{package}`、
`{module}`、`{fqn}`だけが入力identityから置換される。

Surface/Lower/Shell/ResolverProfile mutation enumのexact type名、全variant順、
indexed cardinality、attribute/derive/visibilityはENの4 Rust blockと共有する。
lower mutationは全output fieldとSet index 0/1を覆い、resolver mutationは
nonempty contribution symbol/definition effectも個別に覆う。dormant base 1件と
`#[cfg(test)]` seam 5件の名前、5 common arguments、mutation argument、return
`Option<Result<SourceProofLocalGivenDescendantSetLowerOutput, String>>`もENの
exact signaturesを共有する。最後のseamだけ
`impl FnOnce(SymbolEnv) -> SymbolEnv`を取る。

selector mismatch（missing final LFを含む）は`None`、selected validation
failureは`Some(Err(_))`、successはimmutable row。precedenceはSurface、shell
count/export、shell 0/1、resolver module/local `y/z/q`/indexes、theorem owner、
definition、symbol/definition各1 effectを持つcontribution、lower row、debug。
private error ABIは次のexact 16 stringsだけである。

```text
Task269SDP exact Surface identity changed after selection
Task269SDP requires exactly two declaration shells
Task269SDP resolver shells unexpectedly export a path
Task269SDP declaration shell {ordinal} mismatch
Task269SDP raw resolver module mismatch
Task269SDP local y/z/q already resolves as a module symbol
Task269SDP raw resolver inventory mismatch
Task269SDP requires one exact theorem owner
Task269SDP exact theorem owner provenance mismatch
Task269SDP requires one exact theorem definition
Task269SDP theorem contribution is missing
Task269SDP theorem symbol provenance mismatch
Task269SDP theorem definition provenance mismatch
Task269SDP theorem contribution provenance mismatch
Task269SDP private lower output mismatch
Task269SDP private lower debug grammar mismatch
```

Surface oracleは68件すべてのkind/source/range/recovery/children、root 67、
expression rootなし、token index `[0,36)`を検査する。structural partitionは
checker plan記載の36--67をexactに共有する。これらはsyntax/corruption
contractだけであり、binding/reference/capture/closure/fact/proof semanticsを
一切publishしない。

Completion evidence: [central Task-269SDP historical contract](../../task_contracts/ja/269SDP.md#completion-evidence)。

## Task 269SDC frozen descendant binding consumer

Task 269SDCはimmutable Task-269SDP lower debugをconsumeし、外側Given
bindingとexact descendant context relationshipだけをinstallする。authority、
range、分類、7 primary implementation files + 1 `cfg(test)`-only
predecessor-ownership support file、8 tests、zero-credit境界、exitはcrate
planに凍結し、本ownerはcomplete public ABI/replayを凍結する。

public familyは
`SourceProofLocalGivenDescendantBindingHandoffInput`、
`SourceProofLocalGivenDescendantBindingHandoff`、
`SourceProofLocalGivenDescendantBindingProducer`、non-exhaustive
`SourceProofLocalGivenDescendantBindingError`。既存
`SourceProofLocalGivenBinding{Id,Table,Recovery}`をreuseする。input field
orderはsource/module/lower fingerprint/theorem symbol/definition/
contribution/theorem/proof/Given/segment/name/descendant ranges/source ordinal/
`LocalTermBinding`/descendant `LocalTermScope`/recovery。handoffは同じ
dependency fieldsの後にbase env/fingerprint、final env/fingerprint、one
binding table、`BindingContextId` descendant contextをprivateに保持する。

全read-only getterはEN owner記載field順・constnessをcanonicalとする。
producerは`build(input, &BindingEnv)`だけを公開し、handoffまたはexact
errorを返す。error variantsは`InvalidTransaction`、
`DependencyMismatch`、`InvalidBaseBindingEnvironment`、`InvalidAggregate`、
`InvalidDeclaration { binding: SourceProofLocalGivenBindingId }`、`InvalidDescendantContext`、
`InvalidBindingEnvironment`、`InvalidInstallation`で、exact English
Display bytesはEN ownerをcanonicalとする。errorは`Display`/`Error`を実装。
`pub(crate)`な`validate_installation(&self, SourceId, &ModuleId)`と
`validate_complete_installation(&self, SourceId, &ModuleId, bool)`のexact
Result signatureもEN ownerをcanonicalとする。

dependencyはprimary name `ProofLocalGivenDescendantCaptureSmoke`、
definition/contribution `0/0`、range `19..179`/`73..178`/`81..99`/
`87..98`/`87..88`/`102..159`とcomplete SDP lower debugを独立再構成する。
coherent symbol+fingerprint corruptionもreject。inputはordinal 1、`y`、
scope `[0]`、declaration `87..88`、visible-after 1、descendant `[0,0]`、
normalのみ。

base `1/1/0`からfinal `3/2/0`へのatomic replayはcrate planのexact profile。
rowはbinding/context `1/1`、source/visible `1/1`、normal、descendant context
は2。validation/error mappingはsource/module identity ->
`InvalidTransaction`、lower/theorem/theorem・proof・Given・segment・name ranges
-> `DependencyMismatch`、base env/
base fingerprint -> `InvalidBaseBindingEnvironment`、aggregate ->
`InvalidAggregate`、local/row -> `InvalidDeclaration`、descendant scalar/
scope/context -> `InvalidDescendantContext`、final env/fingerprint/lookup ->
`InvalidBindingEnvironment`、Typed/final availability ->
`InvalidInstallation`の順。全failureはpublication前。

checkerは全alterable public input/injected handoff fieldをcorruptする。runner 15-variant
seamはrepresentably corruptibleなroute inputとcombined precedenceだけをcover
する。recovery enumは`Normal` 1 variantだけなのでrecovery mutationは作らず、
success pathでexact `Normal`を検証する。scope matrix、test-only binding 2、
contexts 3--5のexact identity/parent/scope/visible/resultはEN crate planを
canonicalとし、handoff/source claimには入れない。

debug headerは
`source-proof-local-given-descendant-binding-debug-v1`。続くmodule、quoted
SDP lower、theorem、Given、quoted base env、binding row、
`descendant range=102..159 context=2 parent=1 scope=[0,0] recovery=normal`、
quoted final envの順、blankなし・final LF 1個。exact literalはEN ownerを
canonicalとする。

`TypedAst`/`ResolvedTypedAst`はboxed optional
`source_proof_local_given_descendant_binding`を1件だけownし、full replay後
のみpublishする。Typed getter/installerとResolved getterのexact signature、
Typed/Resolved error display bytesはEN ownerをcanonicalとする。debug slotは
GCU直後かつ`source_statement_references`/node/table前で、absent bytes不変、
present 1回。duplicateと既存proof-local owner 10件を両順序でrejectし、
さらに`resolved_root`と全current source-owner slotをrejectする。全existing
source-owner installerはreciprocal SDC availability checkを追加し、generic
`source_term`をnon-proof rollback sentinelとする。rollback/debug不変を
検証する。semantic tablesは全empty。

private runner outputは`SourceProofLocalGivenDescendantBindingRouteOutput`、
`#[derive(Debug, PartialEq, Eq)]`、dormant rationaleの
`#[allow(dead_code)]`、`pub(in crate::runner)`で、private `typed_ast`/
`resolved` fieldsと同visibilityの`const fn` borrowed gettersだけを持つ。
mutation enumは`SourceProofLocalGivenDescendantBindingRouteMutation`、
`Debug, Clone, Copy, PartialEq, Eq`、private seam rationaleの
`#[allow(dead_code)]`、`pub(in crate::runner)`で、EN owner記載の15 variants
exact order。production functionはdormant `allow(dead_code)`、mutation
functionは`cfg(test)`。routeは
`source_proof_local_given_descendant_binding_output`とcfg-test
`..._with_mutation`で、全parameter/return exact signaturesはEN ownerを
canonicalとする。selector mismatchだけ`None`。reserve errorは
`Task269SDC exact reserve base extraction failed`と
`Task269SDC exact reserve base failed: {error}`。SDP lower errorは不変に
propagateする。

runnerはlower `source_id`/`module_id`、theorem symbol/definition/
contribution、theorem/proof/Given/segment/name ranges、Given name spelling/
source ordinal、descendant-`now` range、`lower_fingerprint`となるcomplete
`debug_text()`を直接consumeする。Given type getters、Set row/RHS、conclusion
getterはcomplete lower fingerprint外では読まず、`y@118..119` occurrenceを作らない。
test corruptionは全representable route input/fingerprint/row/context-2
fields/lookup/installation/precedenceをcoverする。残るexact tests/
exclusions/count/exitは
EN crate planをcanonicalとする。

Completion evidence: [central Task-269SDC historical contract](../../task_contracts/ja/269SDC.md#completion-evidence)。
