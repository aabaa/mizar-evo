# 証明ローカル宣言 source transport

> 正本は英語です。英語版:
> [../en/source_proof_local_declaration.md](../en/source_proof_local_declaration.md)。

## 状態と authority

この文書はqueue Task 269の最初の2つのdependency-minimal sliceである
**Checker Tasks 269A--269B**をfreezeする。英語版がcanonicalであり、同じlogical
task内で本JA companionを同期する。

normative authorityは次の順である。

1. `doc/spec/en/04.variables_and_constants.md` §§4.1、4.4.3、4.6。
2. `doc/spec/en/15.statements.md` §15.4.4。
3. `doc/spec/en/16.theorems_and_proofs.md` §16.4。
4. 実装済みTask-258B3Nのexact source/statement/witness/term transportと
   parser/resolver provenance。
5. Tasks 248--258の公開API、特に`LocalTermBinding`、`BindingEnv`、
   `SourcePrimaryTermHandoff`、`SourceStatementHandoff`、
   `SourceStatementWitnessHandoff`。

広いproof-local declaration gap fixture
`fail_type_elaboration_proof_local_declaration_gap_001.miz`、sidecar、existing
covered diagnostic-gap trace rowはread-onlyのままにする。これらのrowは
positive proof-local binding semanticsをcreditしない。このfixtureは`let`、
`given`、`consider`、`set`、`reconsider`を混在させるためnamed-witness-only
sliceの安全な表現には使えない。frozen sliceをblockする`spec_gap`はない。

selection inventoryはHEAD
`52cf07be3c77d3aa2a797a7681ed9cbabf88295b`、`main`、docs edit前clean、
`origin/main...HEAD = 0/19`、protected `stash@{0}`
`f65cf4a13752ec380710814a9ac6392ccb9d75d4`。origin divergenceはreport-only
`repo_metadata_conflict`で、task-only targetを曖昧にせず修復しない。

## 分類とtask選択

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
