# Source Attribute-Definition Transport

> canonical languageはEnglishです。canonical companion:
> [../../en/source_attribute_definition.md](../../en/source_attribute_definition.md)。

Status: Checker Task 261 implemented source-transport boundary。frozen contractは
exactly one pass consumer/one covered trace rowとしてactiveです。

## Authority、classification、scope

Task 261はordinary unparameterized `attr ... means ...` definition 1件の
syntax-free immutable intakeだけをownします。authorityはChapter 6 §§6.1、6.2、
6.8.1、6.9のlabel/subject/optional pattern prefix/attribute name/formula
definiens/predicate-style identity、Chapter 16 §§16.6/16.7.2のdefinition
correctness boundary、existing parser attribute pass/recovery fixture、
`fail_type_elaboration_attribute_definition_gap_001.miz` sidecar/trace、resolver
declaration/signature tests、completed Tasks 248/249/252/256とTask-259/260
definition-family boundaryです。

missing contractはnonblocking `design_drift`、missing producerは`source_drift`、
missing exact consumerは`test_gap`です。blocking `spec_gap`はありません。
Chapter 16はordinary attribute用initial-obligation kind/goalを定義せず、
attribute-specific rowはredefinition `coherence`だけです。Task 261は
`InitialObligationTable` rowを追加せず、parser support/source behaviorから
発明しません。formula checking、definitional equivalence、acceptance、fact
publicationはdeferredです。

Task 261はdefinition identity、definition-local parameter 2件、subject binding、
already-produced equality formula definiens、resolver/lower provenanceだけを
transportします。prefix parameter、attribute application、redefinition/coherence、
case/otherwise、truth/proof/acceptance/cluster/fact/axiom/overload/IR/VCはownしません。

## Frozen exact source

pass sourceはfinal LF込みexact 116 UTF-8 bytesです。

```mizar
definition
  let x be set;
  let y be set;
  attr Task261AttributeDefinition: x is task261_marked means x = y;
end;
```

SHA-256は
`ffd4954aad628d7946aaf7afb1b472a6bdfca7bce5ba0cf09f5b284c9dda07bf`。
normal definition block 1件、separately written builtin-set parameter 2件、
ordinary unparameterized attribute、subject `x`、spelling `task261_marked`、
equality definiens 1件だけを含みます。`assume`、prefix/argument-list、`non`、
qualifier、redefinition、correctness、conditional/otherwise、import/reserve/
theorem/proof/recoveryはありません。

sourceはChapter 6からderiveしたexact `test_gap`です。既存lowerのtwo-parameter
profileをそのままconsumeします。existing 91-byte one-parameter `thesis` gapの
fixture/sidecar/expectation/traceはbyte-unchangedでbroader gapに残し、その再分類の
ためにunsupported Task-248/formula-constant shapeをretrofitしません。

## Frozen Surface profile

read-only parser probeはdiagnostic 0、dense Surface row exactly 45、root 44、
range `0..115`、expression rootなし、recoveryなしです。private runnerは全rowの
token/kind/range/recovery/ordered childrenをliteral oracleとしてauthenticateします。

| Node | Surface kind | Range | Task-261 role |
| ---: | --- | --- | --- |
| 24/25 | `TypeHead` / `TypeExpression` | `22..25` | parameter `x` written type |
| 27 | `DefinitionParameter` | `13..26` | first context parameter |
| 28/29 | `TypeHead` / `TypeExpression` | `38..41` | parameter `y` written type |
| 31 | `DefinitionParameter` | `29..42` | second context parameter |
| 32 | `AttributePattern` | `83..97` | `task261_marked` pattern |
| 33/34 | `TermReference` / `TermExpression` | `104..105` | body operand `x` |
| 35/36 | `TermReference` / `TermExpression` | `108..109` | body operand `y` |
| 37/38/39 | equality / formula / definiens | `104..109` | exact body |
| 40 | `AttributeDefinition` | `45..110` | definition/subject/pattern/body owner |
| 41 | `DefinitionBlockItem` | `0..115` | common owner |
| 44 | `Root` | `0..115` | complete root |

parameter declaration rangeは`17..18`/`33..34`、labelは`50..76`、subjectは
`78..79`、patternは`83..97`です。27/31/40はblock 41のnormal direct siblingで
orderも固定です。checkerへraw node number/AST/syntax kind/tokenは渡しません。
label/pattern subtreeはTask-252/256 discoveryからexcludeし、equality subtree 37
だけをdefiniens lower rootにします。

## Frozen resolver provenance

exact resolverはshell 2、projection 1、diagnostic 0、attribute symbol/definition
各1、local-source contribution 1です。

- shell 0: `DefinitionBlock` node/range `41/0..115`、ordinal 0、parentなし。
- shell 1: `AttributeDefinition` node/range `40/45..110`、ordinal 1、parent 0。
- definition 0: `SymbolKind::Attribute` / `DefinitionKind::Attribute`、spelling/
  notation `task261_marked`、path `[4,0,7,0]`、public/local/exported/
  overloadable/conflict-free、contribution 0。
- opaque `parser-signature-v1`はrepresentation provenanceだけでcheckerはparseしない。

resolver parameters/binders/arityはemptyです。context parameters、subject、prefix
arity、formula root、correctnessをempty field/opaque textから推測しません。

## Frozen lower bundle

| Owner | Exact profile | Task-261 use |
| --- | --- | --- |
| Task 248 | Profile B `1/2/2/2/2/2/0` | block context、ordered `x`/`y` binding |
| Task 249 | `2/2/0` | binding-linked builtin-set types |
| Task 252 | `2/2/0` | equality operands/binding refs |
| Task 256 | `1/0/0/0/0/0/0/2/2` | equality 1、edge/request各2 |
| Tasks 249R/250/251/253--255/257--258 | absent | return/use/evidence/other root/formulaなし |
| Tasks 259/260 | absent/isolation | predicate/functor transactionなし |

Task 261 routeはauthenticated row 27/31/shell 41からTask-248 Profile B inputを
local constructし、Task-259/260 helper/public producerをmodify/generalizeしません。
Task-249 application 0/1、Task-252 reference 0/1、Task-256 formula 0をexactに
associateし、lower debug fingerprint 4件すべてをidentityへ含めます。

## Frozen public syntax-free contract

new `source_attribute_definition.rs`はdense ID 4 familyを追加します。

```rust
pub struct SourceAttributeDefinitionId(usize);
pub struct SourceAttributeParameterId(usize);
pub struct SourceAttributeSubjectId(usize);
pub struct SourceAttributeDefiniensId(usize);
```

各IDは`Copy + Eq + Ord + Hash`、`new`/`index`だけを公開します。inputはcanonical
ENとexactに同期する以下の4 vectorです。

```rust
pub struct SourceAttributeDefinitionHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub definitions: Vec<SourceAttributeDefinitionInput>,
    pub parameters: Vec<SourceAttributeParameterInput>,
    pub subjects: Vec<SourceAttributeSubjectInput>,
    pub definientia: Vec<SourceAttributeDefiniensInput>,
}
```

definition inputはsymbol/definition/contribution/site/range/source ordinal/context/
recovery/spelling/subject/definiens、parameter inputはowner/ordinal/binding/written
type/site/owner+declaration range/context/recovery/spelling、subject inputはowner/
binding/site/range/context/recovery/spelling、definiens inputはowner/ordinal/
`SourceAtomicFormulaId`/site/range/context/recovery/spellingを持ちます。
`SourceAttributeDefinitionRecovery`は`#[non_exhaustive]`のNormal/Degradedです。

immutable row/tableは全fieldとdense IDをmirrorし、tableは`get`/`iter`/`len`/
`is_empty`だけを公開します。handoffはsource/module、4 table、lower fingerprint
4件、deterministic debug、typed/final validationをownします。public enum/errorは
`#[non_exhaustive]`です。

exact immutable rowは`SourceAttributeDefinition`、
`SourceAttributeParameter`、`SourceAttributeSubject`、
`SourceAttributeDefiniens`です。definition rowはinput field、dense `id`、derived
`SemanticOrigin`、parameter/subject/definiens rowは対応input fieldとdense `id`を
exact API orderでstoreします。各fieldはsame-named read-only getterだけを持ち、
row constructor/setter/mutable getter/replacement APIはありません。

exact table/handoff surfaceはcanonical ENと同じです。

```rust
pub struct SourceAttributeDefinitionTable { /* private rows */ }
pub struct SourceAttributeParameterTable { /* private rows */ }
pub struct SourceAttributeSubjectTable { /* private rows */ }
pub struct SourceAttributeDefiniensTable { /* private rows */ }

pub struct SourceAttributeDefinitionHandoff { /* private fields */ }

impl SourceAttributeDefinitionHandoff {
    pub const fn source_id(&self) -> SourceId;
    pub const fn module_id(&self) -> &ModuleId;
    pub fn source_context_fingerprint(&self) -> &str;
    pub fn source_type_fingerprint(&self) -> &str;
    pub fn source_term_fingerprint(&self) -> &str;
    pub fn source_atomic_formula_fingerprint(&self) -> &str;
    pub const fn definitions(&self) -> &SourceAttributeDefinitionTable;
    pub const fn parameters(&self) -> &SourceAttributeParameterTable;
    pub const fn subjects(&self) -> &SourceAttributeSubjectTable;
    pub const fn definientia(&self) -> &SourceAttributeDefiniensTable;
    pub fn debug_text(&self) -> String;
}
```

各tableはtyped dense `get`、source-ordered `iter`、`len`、`is_empty`だけを
exposeします。fingerprint 4件はproducerがauthenticated lower
`debug_text()`全体からderiveし、caller suppliedではありません。

exact producer/error surfaceは以下です。

```rust
#[non_exhaustive]
pub enum SourceAttributeDefinitionError {
    SourceIdentityMismatch,
    DependencyMismatch,
    InvalidResolverDefinition { index: usize },
    InvalidDefinition { index: usize },
    InvalidParameter { index: usize },
    InvalidSubject { index: usize },
    InvalidDefiniens { index: usize },
    InvalidArenaOwnership,
    UnsupportedTaskShape,
}

pub struct SourceAttributeDefinitionProducer;

impl SourceAttributeDefinitionProducer {
    pub fn build(
        input: SourceAttributeDefinitionHandoffInput,
        env: &SymbolEnv,
        source_context: &SourceBindingContextHandoff,
        source_type: &SourceTypeApplicationHandoff,
        source_term: &SourcePrimaryTermHandoff,
        source_atomic_formula: &SourceAtomicFormulaHandoff,
        arena: &TypedArena,
    ) -> Result<SourceAttributeDefinitionHandoff, SourceAttributeDefinitionError>;
}
```

errorは`Display + Error`、`Default`/blanket conversionなしです。row/table/
handoffは`Debug + Clone + PartialEq + Eq`、producerはunit structです。
## Public Enum Policy

| Public enum | compatibility policy |
| --- | --- |
| `SourceAttributeDefinitionRecovery` | `#[non_exhaustive]`。callerはlater explicitly-frozen recovery classを許容する。 |
| `SourceAttributeDefinitionError` | `#[non_exhaustive]`。callerはvalidation failureをexhaustive matchしない。 |

この module が所有する exhaustive public enum exception はない。

### Debug grammar

stable family keyは
`source.definition.attribute`、`.parameter`、`.subject`、`.definiens`です。

debugは`source-attribute-definition-debug-v1`、module、source-context/type/term/
atomic-formula fingerprint、definition/parameter/subject/definiens rowの順で、
canonical EN記載のexact field order/Rust-debug grammar、final LF 1件、blank lineなし
です。accepted siteは`TypedSiteRef::Node`だけ、originはlocal/unrecovered、rowは
`Normal`だけです。role/imported/recovered/extra branchをfail closedにrejectし、
typed/finalへcomplete stringをexactly once renderします。

## Exact four-table oracle

cardinalityはdefinition/parameter/subject/definiens orderで`1/2/1/1`です。

- definition 0: resolver definition/symbol/contribution 0、site 40、ordinal 0、
  range/local origin `45..110`、origin path `[4,0,7,0]`、context 1、Normal、
  spelling `attr Task261AttributeDefinition: x is task261_marked means x = y;`、
  subject 0、definiens 0。
- parameter 0/1: owner 0、binding 0/1、Task-249 application 0/1、site 27/31、
  ordinal 0/1、owner range `13..26`/`29..42`、declaration range
  `17..18`/`33..34`、context 1、spelling `let x be set;`/`let y be set;`。
- subject 0: owner 0、binding 0、definition site 40、token range `78..79`、
  context 1、spelling `x`。
- definiens 0: owner 0、ordinal 0、Task-256 formula 0、site 39、range
  `104..109`、context 1、spelling `x = y`。

guard/property/correctness/prefix/use/initial-obligation rowはありません。existing
`InitialObligationTable`にはstrict no-read/no-write contractを適用します。Task 261
input/producer/handoff/installerはtableをreceive/inspectせずobligation projectionを
公開しません。optional handoff installはそのfieldだけを変更するためpreexisting
row/IDはbyte-identicalで、errorは何もpublishしません。

## Validation とsemantic boundary

`SourceAttributeDefinitionProducer::build`はinput、`SymbolEnv`、Task-248 context、
Task-249 type、Task-252 primary、Task-256 atomic formula、typed arenaをatomicに
validateします。missing/duplicate/reordered/dangling/cross-owner/module、recovery、
stale site/range/context/origin/symbol/definition/contribution/binding/type/formula/
fingerprint、wrong spelling/ordinal/kind、partial/extra rowをrejectし、sort/repair
しません。

bodyは`SourceAtomicFormulaId(0)`へのoccurrence linkだけです。equality evaluation、
FOL biconditional、truth、subject admissibility、evidence/fact、obligationを作りません。
accepted attribute、cluster/type fact、axiom/theorem/proof/Core/CFG/VCもpublishしません。

parameterized/prefixed、case/otherwise、formula constant/composite/quantified、
redefinition/coherence、qualifier/inheritance/negative/application/clusterはdeferredで、
別authority/lower contract/test/commitが必要です。

## Typed/final ownership

`TypedAst`はoptional fieldとone-shot
`with_source_attribute_definition(SourceAttributeDefinitionHandoff)`、getter、
`TypedAstError::InvalidSourceAttributeDefinition`だけを追加します。lower fingerprint/
arenaをauthenticateし、obligation tableをread/compareせず、prior occupancyをreject
します。structural updateはexisting obligation tableをunchangedにretainします。
`TypedAstParts`にはfieldを追加しません。

`ResolvedTypedAst::assemble`はtyped ownerだけからclone/revalidateし、getterと
`ResolvedTypedAstError::InvalidSourceAttributeDefinition`だけを追加します。
`ResolvedTypedAstInputs`にreplaceable fieldは追加しません。debug headerは
`source-attribute-definition-debug-v1`、typed/finalにexactly once、legacy empty
bytesはunchangedです。

current exact Task-259/260 transactionとはmutually isolatedです。Task-261 installer/
finalは他definition handoffをrejectしますが、Task-259/260 validationをeditせず
install-order promiseを作りません。future mixed contractはsame-source lower ownership/
obligation orderingをseparately freezeします。

## Consumer、trace、counts

implementationはnew pass pair
`pass_type_elaboration_attribute_definition_payload_001.miz` / `.expect.toml`と
sole requirement
`spec.en.checker.type_elaboration.source_attribute_definition_payload` 1件だけを
追加します。sidecarはpass/type_elaboration/type_check、empty diagnostics/payload、
one covered reciprocal backlinkです。creditはexact transportだけです。

private routeはcomplete 116-byte/45-row/resolver/lower profileだけをgeneric
attribute gapより先にselectします。outcome/stage/tag/diagnostic/payload/filenameは
selectorではありません。existing one-parameter gap、parser cases、Task-259/260、
mixed gapはunchangedです。

implementation resultはchecker/runner `444 -> 449` / `516 -> 520`、active type
`200 -> 201`、plan case/requirement `423/391 -> 424/392`、pass/fail
`230/193 -> 231/193`、type requirement `255/243 -> 256/244`です。parse/
declaration/proofは`101/7/1`、warnings/errorsは`23/0`です。

## Frozen tests、write scope、exit

checker focused test exactly 5件はexact handoff/non-empty baselineに対するstrict
obligation no-read/no-write、input+resolver
corruption、lower+fingerprint corruption、atomic install/Task-259/260 isolation、final
determinism/no semantic publicationをcoverします。exact nameはcanonical ENです。

runner focused test exactly 4件はexact transport、literal 45-row plus source/resolver/
lower corruption、source-only selection+reciprocal trace、no semantic publicationを
coverします。excluded label/pattern subtreeと全mechanical count consumerもtestします。

implemented scopeはnew checker producer/support、checker lib/typed/final/
serializer/lint、private runner leaf/facade/test、new fixture/sidecar/trace、mechanical
count、derived EN/JAだけです。parser/resolver/Cargo/doc/spec/existing `.miz`/existing
expectation/sidecar/lower producer/Task-259/260 behavior/unrelated semanticsはunchangedです。

module/source audit、mizar-test traceability、spec coverage auditはactive partial
creditをrecordします。Chapter 6はTask 261後もpartialです。

implementation exitはEN/JA sync、repeated review-only **NO FINDINGS**、exact
count/hash、hard gate 9件PASS、valid quality 90+、task-only stage、one logical-task
commit、clean post-commit、Task 262+へのautomatic returnです。
