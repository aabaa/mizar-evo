# Module: traceability

> Canonical language: English. English canonical version: [../en/traceability.md](../en/traceability.md).

## Purpose

この module は、`doc/spec/` requirements と committed tests を接続する traceability manifest を定義する。Specification text 自体には test links を追加しない。

Specification は読みやすい language reference として維持する。Test coverage は `mizar-test` が所有する machine-readable manifest で管理する。

## Design Decision

Specification-to-test links は `doc/spec/` の外に置く。

Traceability model は bidirectional である。

- manifest は spec requirements から、それを cover する tests へ map する
- each test expectation sidecar は test から one or more spec requirement ids へ map する

Harness は両方向を validate する。

```text
doc/spec/...                    pure specification text
tests/coverage/spec_trace.toml  spec requirement -> tests
*.expect.toml                   test -> spec requirement ids
```

## Manifest Location

Canonical manifest:

```text
tests/coverage/spec_trace.toml
```

Additional generated reports は `tests/coverage/reports/` に出力してよいが、それらは derived artifacts である。Manifest が source of truth である。

## Requirement Record

各 requirement record は specification の checkable unit を表す。

```toml
[[requirement]]
id = "spec.en.02.lexical.identifiers.basic"
source = "doc/spec/en/02.lexical_structure.md"
section = "2.6 Identifiers"
stage = "lexical"
status = "planned"
required = true
coverage = "pass_and_fail"
tests = []
```

Fields:

| Field | Meaning |
|---|---|
| `id` | Stable requirement id。Unrelated semantics に再利用してはならない。 |
| `source` | Requirement を所有する specification file。 |
| `section` | Human-readable section heading or section number。 |
| `stage` | Executable coverage を最初に所有する staged model stage。 |
| `status` | `planned`, `covered`, `partial`, `deferred`, or `obsolete`。 |
| `required` | Release coverage がこの item を要求するか。 |
| `coverage` | Expected coverage shape。 |
| `tests` | Expectation sidecars または fixture metadata への canonical relative paths。 |

Optional fields:

| Field | Meaning |
|---|---|
| `anchors` | Available な stable heading anchors。 |
| `notes` | Short human review notes。 |
| `depends_on` | 先に cover されるべき lower-stage requirement ids。 |
| `built_in` | `true` の場合、その requirement は built-ins によって提供され、executable coverage なしで他 requirement の `depends_on` を満たせる。 |
| `deferred_reason` | `status = "deferred"` の場合に必須。 |
| `issue` | Tracking issue or design discussion reference。 |

## Coverage Shapes

`coverage` は期待される test evidence の種類を記録する。

Allowed values:

| Value | Meaning |
|---|---|
| `none` | Executable test は不要。Explanatory text に使う。 |
| `pass` | 少なくとも one accepting test が必要。 |
| `fail` | 少なくとも one rejecting test が必要。 |
| `pass_and_fail` | Accepting and rejecting tests の両方が必要。 |
| `diagnostic` | Stable diagnostic or failure category を check する必要がある。 |
| `snapshot` | Deterministic snapshot を check する必要がある。 |
| `property` | Property or generated test family が cover する。 |
| `manual_review` | Human review が必要。Executable coverage だけでは不十分。 |

複数の shapes が必要な場合、specification section を複数の requirement records に分割する。

## Public Enum Forward Compatibility

task 12 は `mizar-frontend` task 25 の手続きを traceability enum に適用する。
これらの enum は `spec_trace.toml` に保存され、harness が report し、downstream
tooling が消費するため、`#[non_exhaustive]` を維持しなければならない。downstream
caller は wildcard match arm を保つ必要がある。crate 内部の match は現在知られている
variant に対して exhaustive のままでよい。

| Public enum | Decision |
|---|---|
| `RequirementStatus` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoverageShape` | `#[non_exhaustive]` downstream forward-compatible surface。 |

この module が所有する exhaustive public enum exception はない。

## Test Sidecar Reference

Each expectation sidecar は cover する spec requirements を記録する。

```toml
schema_version = 1
id = "pass_lexical_identifier_basic_001"
stage = "lexical"
spec_refs = [
  "spec.en.02.lexical.identifiers.basic",
]
```

Executable sidecar の `stage` は requirement の `stage` と一致しなければならない。
ただし requirement が満たされた `depends_on` chain によって later stage からの
coverage を明示的に許す場合を除く。Earlier-stage executable sidecar は later-stage
requirement を credit できない。`manual_review` entry は executable coverage ではなく
metadata anchor なので、declared prerequisites が満たされている限り cross-stage handoff
note を記録してよい。

## Validation

Harness は次を validate する。

1. Every manifest `source` exists.
2. Every `id` is unique.
3. Every listed test path exists.
4. Every listed test sidecar points back to the requirement id.
5. Every sidecar `spec_refs` entry exists in the manifest.
6. Stage names match the staged model.
7. Required coverage shapes are satisfied when the validation mode requires coverage completeness.
8. Deferred required items include a `deferred_reason`.
9. Obsolete items are not referenced by active tests.
10. Manifest records are sorted deterministically by `id`.
11. `depends_on` ids が存在し、自分自身を指さず、lower-stage requirement のみを指す。
12. Declared stage prerequisites が満たされていない、または executable stage が credited requirement と合わない linked sidecar は coverage credit を受けない。

Validation は referenced files が存在すること以外に `doc/spec/` prose を parse してはならない。Requirement granularity は manifest が所有する。

## Validation Modes

Traceability validation は modes を持つ。

| Mode | Purpose | Coverage Completeness |
|---|---|---|
| `metadata` | Minimal crate and local editing. | Required ではない。Tests なしの planned items は最大でも warnings。 |
| `development` | Implementation 中の normal CI. | `status = "covered"` or `partial` の requirements にのみ required。 |
| `release` | Release readiness gate. | `status = "deferred"` with reason を除き、every `required = true` requirement に required。 |

All modes は manifest syntax、unique ids、source file existence、known stage ids、known sidecar references、sidecar back-references を validate する。

`required = true` であることだけを理由に missing coverage を error にするのは
`release` mode だけである。`development` mode は `metadata` より厳しく、
manifest 上 `covered` または `partial` の item には gate を適用するが、compiler
pipeline が存在する前の planned coverage map entries は引き続き許容する。

現在の mode behavior:

- `metadata` は stored-status drift を warning として emit し、missing
  coverage shape では fail しない。
- `development` は manifest 上 `covered` または `partial` の requirement
  に対する missing coverage と status drift を fail させる。
- `release` は `deferred` かつ reason 付きでない限り、すべての
  `required = true` requirement の missing coverage と status drift を
  fail させる。

## Coverage Status

`status` は reporting 時に derive されるが、review workflow のため manifest にも保存する。

Rules:

- `planned` means the requirement is known but lacks sufficient tests.
- `partial` means some required coverage exists but not all coverage shapes are satisfied.
- `covered` means all required coverage shapes are satisfied by active tests.
- `deferred` means coverage is intentionally postponed.
- `obsolete` means the requirement no longer applies and active tests must not claim it.

Report は stored status と computed status が一致しない場合に flag する。その severity は validation mode によって決まる。

実装済み coverage report は valid bidirectional link だけから evidence を
derive する。Manifest が sidecar path を list し、parse 済み sidecar が
requirement id を backref している必要がある。Invalid sidecar、one-way link、
unsatisfied prerequisites、invalid executable stage mismatch には coverage credit を
与えない。Coverage evidence は次のように計算する。

- `pass`: `expected_outcome = "pass"` の linked sidecar が少なくとも 1 件ある。
- `fail`: `expected_outcome = "fail"` の linked sidecar が少なくとも 1 件ある。
- `pass_and_fail`: pass evidence と fail evidence の両方が存在する。
- `diagnostic`: linked fail sidecar が diagnostic/failure identity metadata
  を持つ。
- `snapshot`: linked sidecar が snapshot output、snapshot outcome、または
  snapshot kind を持つ。
- `property`: linked sidecar が `kind = "property_seed"` を持つ。
- `manual_review`: linked sidecar が存在する。ただし executable evidence
  だけでは十分ではないため、stored status は human-reviewed status のまま扱う。

`none` と `manual_review` の status は linked metadata がある場合、manifest
から推論で変更しない。これらは review workflow state のままである。
`manual_review` に linked metadata がない場合は `planned` として計算する。

## Stage Interaction

Traceability は [staged_model.md](./staged_model.md) の staged model を使う。

Coverage credit は lower-stage prerequisites が既に covered、built-ins として declared、または acceptable status を持つ `depends_on` に listed されている場合にのみ与える。

task 7 は declared stage-prerequisite と `depends_on` credit rules を enforce する。
Harness は `doc/spec/` prose や source contents から prerequisites を推測しない。
明示的な lower-stage credit が必要な requirements は `depends_on` に列挙し、built-in
prerequisites は `built_in = true` を使う。

例えば parser fixture は cluster declaration の syntax を cover できるが、cluster expansion semantics は cover しない。Semantic requirement は advanced semantic tests が存在するまで planned のままである。

Declaration-symbol stage では、active runner gate（`active_declaration_symbol`、
`stage = "declaration_symbol"`、`expected_phase = "resolve"`、pass/fail outcome）
を満たす `.miz` sidecar だけが executable coverage になる。public resolver
diagnostic code が未仕様の間、fail coverage は resolver internal detail key を
`diagnostic_payloads` または `stable_detail_key` で assert してよい。この range が
存在するまで、active sidecar は `diagnostic_codes` を空にしなければならない。

type-elaboration stage では、active runner gate（`active_type_elaboration`、
`stage = "type_elaboration"`、`expected_phase = "type_check"`、pass/fail outcome）
を満たす `.miz` sidecar だけが executable coverage になる。task 16-20 bridge
continuation が credit してよいのは狭い reserve-only builtin declaration pass slice である。
つまり unrecovered な top-level reserve item のうち、segment が 1 個以上の identifier と
exactly one bare builtin `set` / `object` type-expression を持ち、attribute、argument、
parameter prefix、non-builtin symbol head を持たないものに限る。task 55 はさらに、
un-attributed / argument-free の same-module local mode を reserve type head とし、
runner が unique / unrecovered / preceding / same-module / no-argument `ModeDefinition`
から、RHS が bare builtin `set` / `object` で enclosing definition block が
definition-local context を持たない real `ModeExpansion` を導ける場合だけ、狭い bare
local-mode expansion pass slice を credit してよい。task 56 はさらに、reserve type head が
accepted task-55 bare builtin RHS expansion を持つ preceding same-module no-argument
local mode へ expand し、runner が両方の real source-derived expansion を checker-owned
reserve seam の前に挿入する場合だけ、狭い one-edge local-mode expansion chain pass slice を
credit してよい。task 57 は diagnostic-only fail slice として、reserve head が real
same-module local-mode expansion を通じて same-module local structure head へ expand する場合を
credit してよい。runner は real expansion を checker-owned seam に渡すが、checker は
missing base-shape / constructor-witness evidence query を報告する。task 58 は parallel な
diagnostic-only fail slice として、reserve head が real same-module local-mode expansion を
通じて attributed builtin head へ expand する場合を credit してよい。runner は real
expansion を checker-owned seam に渡すが、checker は missing attributed-type
existential evidence query を報告する。task 60 は direct attributed-root
structure-RHS diagnostic-only fail slice として、reserve head が attributed
argument-free same-module local mode で、runner が task-57 由来の unique /
preceding / no-context 条件の下で real direct local-structure RHS expansion を導出し、
checker が missing base-shape / constructor-witness と full attributed-type evidence
query を報告する場合を credit してよい。task 61 は direct attributed-root
attributed-builtin-RHS diagnostic-only fail slice として、reserve head が attributed
argument-free same-module local mode で、runner が task-58 由来の unique /
preceding / no-context 条件の下で real direct attributed-builtin RHS expansion を導出し、
checker が missing full attributed-type existential evidence query を報告する場合を
credit してよい。task 62 は one-edge bare local-mode structure-RHS chain
diagnostic-only fail slice として、reserve head が un-attributed argument-free
same-module local mode で、runner が source order 上の unique / unrecovered /
preceding same-module definitions から real `A -> B` と `B -> LocalStruct` expansion
の両方を導出し、checker が missing base-shape / constructor-witness evidence query を
報告する場合を credit してよい。task 63 は one-edge bare local-mode
attributed-builtin-RHS chain diagnostic-only fail slice として、reserve head が
un-attributed argument-free same-module local mode で、runner が source order 上の
unique / unrecovered / preceding same-module definitions と argument-free same-module
RHS attributes から real `A -> B` と `B -> marked set` expansion の両方を導出し、
checker が missing attributed-type existential evidence query を報告する場合を credit
してよい。task 65 は one-edge attributed-root structure-RHS chain diagnostic-only
fail slice として、reserve head が attributed argument-free same-module local mode で、
root が bare reserve use と mixed でなく dependency も attributed ではない場合に、
runner が source order 上の unique / unrecovered / preceding same-module definitions
から real `A -> B` と `B -> LocalStruct` expansion の両方を導出し、checker が
missing base-shape / constructor-witness と full attributed-type evidence query を
報告する場合を credit してよい。task 66 は one-edge attributed-root
attributed-builtin-RHS chain diagnostic-only fail slice として、reserve head が
attributed argument-free same-module local mode で、root が bare reserve use と mixed でなく
dependency も attributed ではない場合に、runner が source order 上の unique /
unrecovered / preceding same-module definitions と argument-free same-module RHS
attributes から real `A -> B` と `B -> marked set` expansion の両方を導出し、checker が
missing full attributed-type evidence query を
報告する場合を credit してよい。task 67 は structure-qualified attribute
extraction-gap boundary slice として、same-module structure-qualified attribute
reference が parser/resolver executable である一方、checker payload が real qualifier と
attribute-owner provenance を保持するまで runner が
`type_elaboration.external_dependency.ast_payload_extraction` を assert する場合を
credit してよい。task 68 は argument-bearing mode reserve extraction-gap boundary
slice として、`Element of a` のような same-module argument-bearing local mode surface と
reserve use が parser/resolver executable である一方、checker payload が real
type-argument と term-argument provenance を保持するまで runner が
`type_elaboration.external_dependency.ast_payload_extraction` を assert する場合を
credit してよい。task 69 は argument-bearing structure reserve extraction-gap boundary
slice として、`of` parameter surface を持つ same-module structure declaration と
`LocalStruct of a` のような reserve use が parser/resolver executable である一方、
checker payload が real type-argument と term-argument provenance を保持するまで
runner が `type_elaboration.external_dependency.ast_payload_extraction` を assert する場合を
credit してよい。task 70 は bracket-form local mode reserve extraction-gap boundary
slice として、same-module bracket-parameter mode declaration と `Family[set]` のような
reserve use を含む source が parser/resolver まで到達する一方、bracket type-argument
payload extraction や mode-head resolution の前に runner が
`type_elaboration.external_dependency.ast_payload_extraction` を assert する場合を
credit してよい。task 71 は bracket-form local structure reserve extraction-gap boundary
slice として、same-module bracket-parameter structure declaration と `LocalStruct[set]` のような
reserve use を含む source が parser/resolver まで到達する一方、bracket type-argument
payload extraction や structure-head resolution の前に runner が
`type_elaboration.external_dependency.ast_payload_extraction` を assert する場合を
credit してよい。task 72 は two-edge bare local-mode chain pass slice、task 73 は
three-edge bare local-mode chain pass slice、task 74 は structural bare local-mode
chain pass slice として、unique / unrecovered / preceding / same-module /
no-argument な definition から chain の各 real expansion を抽出し、temporary
depth cap を AST-bounded structural traversal budget に置き換える場合を credit
してよい。task 75 は forward local-mode reserve head について、checker handoff 前の
`type_elaboration.lower_stage.frontend:malformed_type_expression` だけを credit
してよい。task 76 は forward local-structure reserve head について同じ lower-stage
detail だけを credit し、checker structure type-head、base-shape、
constructor-witness payload extraction を credit しない。task 77 は forward
local-attribute reserve type expression について同じ lower-stage detail だけを
credit し、checker `AttributeInput` payload extraction や attributed-type
evidence query を credit しない。task 78 は、task 83 がその `R` 部分を supersede する前の documented `R`
imported-structure reserve-head external-gap boundary だけを historical に credit
する。task 97 は documented `TypeCaseStruct` 部分を supersede する。task-83
`R` bridge と task-97 `TypeCaseStruct` bridge 外の broader imported structure は
deferred とする。将来の active case は
`type_elaboration.external_dependency.ast_payload_extraction` を観測し、real
imported structure provenance、structure type-head payload extraction、
base-shape / constructor-witness evidence、positive structure elaboration、
CoreIr、ControlFlowIr、VC、proof payload を credit してはならない。task 83 と task
97 は imported-structure reserve-head provenance bridge だけを credit してよい。
runner は documented `parser.type_fixtures` imported structure `R` と
`TypeCaseStruct` について real imported `SymbolKind::Structure` symbol を checker
type head として渡した後に
`type_elaboration.checker.checker.declaration.deferred.evidence_query` を観測する。
これは imported structure provenance と type-head payload extraction を credit する
が、imported module AST extraction、base-shape / constructor-witness evidence、
positive structure elaboration、CoreIr、ControlFlowIr、VC、proof payload は credit
しない。task 79 は task-82
`TypeCaseMode` bridge 外の imported-mode reserve-head external-gap boundary
だけを credit する。これらの case は documented `parser.type_fixtures`
imported mode summary について
`type_elaboration.external_dependency.ast_payload_extraction` を観測し、
imported mode provenance、mode type-head payload extraction、`ModeExpansion`
payload、positive mode elaboration、CoreIr、ControlFlowIr、VC、proof payload を
credit しない。task 82 は imported-mode reserve-head provenance bridge だけを
credit してよい。runner は同じ documented `parser.type_fixtures` imported mode
summary について real imported `SymbolKind::Mode` symbol を checker type head
として渡した後に
`type_elaboration.checker.checker.type.external.mode_expansion_payload` を観測する。
これは imported mode provenance と type-head payload extraction を credit するが、
imported module AST extraction、`ModeExpansion` payload、positive mode
elaboration、CoreIr、ControlFlowIr、VC、proof payload は credit しない。task 80 は
historically imported-attribute reserve external-gap boundary を credit したが、
task-84 `TypeCaseAttr`、task-85 negative `empty`/builtin-`set`、task-116 positive
`empty`/builtin-`set`、task-171 negative `empty`/builtin-`object` bridge がその row
の全 active fixture を supersede した。positive `empty object`、symbol head 上の
imported attribute、broader source shape は active fixture credit のない deferred
extraction gap に残り、real source-derived payload producer が存在するまで将来の
test も deferred とする。task 84 は imported-attribute
provenance / `AttributeInput` bridge だけを credit してよい。runner は documented
`parser.type_fixtures` imported attribute `TypeCaseAttr` を builtin `set` 上の
real imported `SymbolKind::Attribute` checker `AttributeInput` として渡した後に
`type_elaboration.checker.checker.declaration.deferred.evidence_query` を観測する。
これは imported attribute provenance と argument-free `AttributeInput` payload
extraction を credit するが、imported module AST extraction、attributed-type
existential/evidence payload、positive imported attributed type elaboration、
`empty` のような generic imported attribute、structure-qualified attribute owner
provenance、attribute argument、CoreIr、ControlFlowIr、VC、proof payload は credit
しない。task 85 は imported negative `empty` attribute provenance /
`AttributeInput` bridge だけを credit してよい。runner は既存 `non empty set`
fixture について real imported `SymbolKind::Attribute` symbol `empty` を builtin
`set` 上の negative checker `AttributeInput` として渡した後に
`type_elaboration.checker.checker.declaration.deferred.evidence_query` を観測する。
これはその fixture の imported attribute provenance と argument-free negative
`AttributeInput` payload extraction だけを credit し、imported module AST extraction、
attributed-type existential/evidence payload、broader imported attribute、
structure-qualified attribute owner provenance、attribute
argument、CoreIr、ControlFlowIr、VC、proof payload は credit しない。task 116 は
existing `empty set` fixture について、matching positive `empty`/builtin-`set`
provenance / `AttributeInput` bridge と同じ evidence-query diagnostic だけを credit
してよい。task 171 は existing `non empty object` fixture について exact negative
`empty`/builtin-`object` provenance / `AttributeInput` bridge と同じ evidence-query
diagnostic だけを credit してよい。attribute admissibility/evidence、positive
`empty object`、symbol head 上の imported attribute、accepted attributed type は
credit しない。task 81 は
argument-bearing local-attribute extraction-gap boundary だけを credit する。
runner は `param_prefix` 構文で宣言され `attribute_name(args)` として使われる
same-module parameterized attribute について
`type_elaboration.external_dependency.ast_payload_extraction` を観測し、real
term-argument provenance、checker `AttributeInput` argument payload、
attributed-type evidence、positive attributed type elaboration、CoreIr、
ControlFlowIr、VC、proof payload を credit しない。
declaration-symbol runner は別途、この declaration の resolver suffix-primary
projection と suffix による imported-lexicon visibility を credit してよいが、
これは checker argument payload extraction ではない。
task 67、task 68、
task 69、task 70、task 71 の external-gap boundary case、task 75/task 76/task
77 の lower-stage boundary case、task 78 の historical imported-structure external-gap
case（task 83 が `R` 部分を supersede し、broader non-`R` case は deferred）、task 79 の imported-mode external-gap case、task 84 / task 85 / task 116 外の task 80 imported-attribute
external-gap case、task 81 の argument-bearing local-attribute external-gap case、
task 86 の formula-only theorem external-gap case、task 87 の historical
term-bearing theorem formula external-gap case（task 106 が exact
`TermFormulaPayloadBoundary: 1 = 1` portion を checker term/formula payload
bridge と numeric-type / partial-formula diagnostic へ supersede）、task 110 の imported predicate/functor theorem checker bridge、task 108 の builtin membership theorem checker bridge、task 107 の builtin inequality theorem checker bridge、task 109 の builtin type-assertion theorem checker bridge、task 103 の imported attribute assertion theorem formula external-gap case（exact task 113 bridge 外）、task 113 の imported attribute assertion theorem checker bridge、task 114 の exact attribute-level non-empty imported attribute assertion theorem checker bridge、task 111 の exact set-enumeration theorem checker bridge、task 112 の connective/quantifier formula shell checker bridge、task 88 の proof-block theorem external-gap case、
task 89 の statement-proof external-gap case、task 90 の predicate/functor
definition external-gap case、task 91 の attribute definition external-gap case、
task 92 の mode/structure definition external-gap case、task 93 の
proof-local declaration external-gap case、および task 94 の proof-local inline
definition external-gap case、task 95 の registration block external-gap case、
task 96 の redefinition/notation external-gap case を除き、task 85 の
imported negative `empty`/builtin-`set` provenance slice と task 116 の
imported positive `empty`/builtin-`set` provenance slice を含む上記の supported reserve slices を syntax-free checker source
reserve payload へ変換し、checker-owned seam が module `BindingEnv`、binding ごとの
`DeclarationInput`、binding 固有の `TypeExpressionInput` site、
`DeclarationChecker` output を構築する。runner はその handoff を checker-owned
`TypedAst`、checker-owned `ResolvedTypedAst` へ継続し、その後 `mizar-core` の
`ResolvedTypedAstSummary::from_ast` と binder-only `CoreContext` preparation で
readiness として読む。
active pass test は、listed source が少なくとも 1 個の抽出済み reserve binding を持ち、
runner regression evidence が checker handoff construction、最小 `TypedAst`、
`ResolvedTypedAst`、summary-readiness、binder-only core context path の実行を確認する場合だけ、
この slice を cover してよい。pass slice は diagnostic external-gap row から credit せず、
専用の traceability row/test を持たなければならない。

case が task 96 の redefinition/notation extraction-gap boundary、task 95 の registration block extraction-gap boundary、task 94 の
proof-local inline definition boundary、task 93 の proof-local declaration
boundary、task 92 の mode/structure definition boundary を超える未対応の
non-builtin declaration、imported symbol、unresolved / ambiguous symbol、
attribute / mode / structure argument、structure-qualified attribute provenance、
type-argument / term-argument provenance、proof-local declaration payload、
inline definition payload、registration payload、activation / correctness
payload、term、formula、coercion、overload payload、fact、CoreIr、ControlFlowIr、
VC payload、proof payload extraction を必要とする場合、
covered active fail test は引き続き external-gap detail key
`type_elaboration.external_dependency.ast_payload_extraction` を assert してよい。
対応済み checker-owned fail slice は、same-module attributed builtin reserve head の
missing-evidence diagnostic、same-module local structure reserve head の missing
base-shape evidence diagnostic、full normalized attributed type の existential evidence を欠く
attributed local structure、task-57 の local structure RHS を持つ same-module local-mode
expansion における missing base-shape evidence、task-58 の attributed builtin RHS を持つ
same-module local-mode expansion における missing attributed-type existential evidence、
task-59 の real direct bare-builtin expansion を持つ attributed local-mode reserve head
における missing attributed-type existential evidence、
task-60 の real direct local-structure RHS expansion を持つ attributed local-mode
reserve head における missing base-shape / constructor-witness と full attributed-type
evidence、
task-61 の real direct attributed-builtin RHS expansion を持つ attributed local-mode
reserve head における missing full attributed-type existential evidence、
task-62 の local structure RHS で終端する one-edge bare local-mode chain における
missing base-shape / constructor-witness evidence、
task-63 の attributed builtin RHS で終端する one-edge bare local-mode chain における
missing attributed-type existential evidence、
task-64 の bare-builtin chain を持つ attributed local-mode reserve head における
missing attributed-type existential evidence、
task-65 の structure-RHS chain を持つ attributed local-mode reserve head における
missing base-shape / constructor-witness evidence と full attributed-type existential evidence、
task-66 の attributed-builtin-RHS chain を持つ attributed local-mode reserve head における
missing full attributed-type existential evidence、または same-module local mode reserve head の
missing mode-expansion payload diagnostic（mixed attributed/bare
local-mode source や task-74 structural guard を満たさない chain を含む）の checker detail key を代わりに assert してよい。task 56 の
attributed-chain-dependency fail case は同じ missing mode-expansion payload family に属し、
partial chain expansion は credit しない。attributed-RHS chain も task 58 / task 61 の
direct slice、task 63 の bare one-edge chain slice、task 66 の attributed-root
one-edge chain slice の外では credit せず、structure-RHS chain も task 60 の direct attributed-root
slice、task 62 の bare one-edge chain slice、task 65 の attributed-root chain slice の外では
credit しない。task 67 の structure-qualified attribute case は extraction-gap
boundary coverage としてだけ credit し、real qualified attribute payload coverage とは
扱わない。task 68 の argument-bearing mode case は extraction-gap boundary coverage
としてだけ credit し、real mode-argument payload、arity matching、mode expansion、
positive type-elaboration coverage とは扱わない。task 69 の argument-bearing
structure case は extraction-gap boundary coverage としてだけ credit し、real
structure-argument payload、arity matching、base-shape evidence、positive structure
type-elaboration coverage とは扱わない。task 70 の bracket-form mode case は
extraction-gap boundary coverage としてだけ credit し、real bracket type-argument
payload、`qua`-argument payload、mode-head resolution、arity matching、mode expansion、
positive type-elaboration coverage とは扱わない。task 71 の bracket-form structure case は
extraction-gap boundary coverage としてだけ credit し、real bracket type-argument
payload、`qua`-argument payload、structure-head resolution、arity matching、base-shape /
constructor-witness evidence、positive structure type-elaboration coverage とは扱わない。
task 72 の pass case は source-derived two-edge bare local-mode chain bridge だけを
credit し、task 73 の pass case は source-derived three-edge bare local-mode chain
bridge だけを credit し、task 74 の pass case は AST-bounded structural bare
local-mode chain bridge だけを credit する。task-74 unsupported chain case は
broader mode expansion、既存 one-edge diagnostic を超える structure /
attributed-builtin terminal、CoreIr / ControlFlowIr / VC / proof promotion を
credit しない。task 75/76/77 の fail case は forward local-mode /
local-structure / local-attribute reference の lower-stage active-range boundary だけを
credit し、checker `ModeExpansion`、structure type-head、base-shape、
constructor-witness、`AttributeInput`、attributed-type evidence production は credit しない。
task-83 `R` bridge と task-97 `TypeCaseStruct` bridge 外の broader imported
structure case は deferred とする。将来の
fail case は imported structure extraction-gap boundary だけを credit し、real
imported structure provenance や structure evidence は credit してはならない。task
83 / task 97 の fail case は real imported structure provenance と checker type-head
payload extraction だけを credit し、imported module AST extraction、base-shape /
constructor-witness evidence、positive imported structure elaboration、downstream
payload promotion は credit しない。task 79 の fail case は task-82 `TypeCaseMode` bridge 外の imported
mode extraction-gap boundary だけを credit する。task 82 の fail case は real
imported mode provenance と checker type-head payload extraction だけを credit
し、imported module AST extraction、imported mode expansion、arity checking、
positive imported mode elaboration、downstream payload promotion は credit しない。
task 80 の fail case は task-84 `TypeCaseAttr` bridge、task-85 negative
`empty`/builtin-`set` bridge、task-116 positive `empty`/builtin-`set` bridge 外の
imported attribute extraction-gap boundary だけを credit し、real imported
attribute provenance や attributed-type evidence は credit
しない。task 84 の fail case は real imported
attribute provenance と checker `AttributeInput` payload extraction だけを credit
し、imported module AST extraction、attributed-type existential/evidence payload、
positive imported attributed type elaboration、generic imported attribute、
qualified owner provenance、attribute argument、downstream payload promotion は
credit しない。task 85 の fail case は builtin `set` について real imported
negative `empty` provenance と checker `AttributeInput` payload extraction だけを
credit し、task 116 は matching positive `empty`/builtin-`set` payload、task 171
は exact negative `empty`/builtin-`object` payload を credit する。positive
`empty object`、symbol head 上の imported attribute、broader imported attribute、
imported module AST extraction、attributed-type evidence、owner provenance、
attribute argument、downstream payload は extraction/deferred gap に残す。
task 81 の fail case は argument-bearing local-attribute extraction-gap boundary
だけを credit し、real term-argument provenance、checker `AttributeInput`
argument payload、attributed-type evidence、positive attributed type elaboration、
CoreIr、ControlFlowIr、VC、proof payload は credit しない。
task 86 の fail case は parser / resolver 実行後の formula-only theorem
extraction-gap boundary だけを credit し、checker theorem/formula payload
extraction、recorded fact、theorem acceptance、proof skeleton、
`formula_statement` runner support、CoreIr、ControlFlowIr、VC、proof payload は
credit しない。
task 87 の fail case は当初、parser / resolver 実行後の term-bearing theorem
formula extraction-gap boundary だけを credit した。task 106 は exact
`TermFormulaPayloadBoundary: 1 = 1` sidecar について real checker term/formula
payload extraction を credit するが、numeric type payload、equality checking、
recorded fact、theorem acceptance、proof skeleton、`formula_statement` runner
support、CoreIr、ControlFlowIr、VC、proof payload は credit しない。
task 98 の fail case は parser / resolver 実行後の imported predicate/functor
theorem formula extraction-gap boundary だけを credit していた。task 110 は exact
`ImportedPredicateFunctorPayloadBoundary: 1 divides (1 ++ 2)` sidecar について
real checker numeral、imported functor-application、predicate-application
payload extraction を credit するが、imported module AST extraction、semantic
predicate/functor signature、term inference、formula checking、recorded fact、
theorem acceptance、proof skeleton、`formula_statement` runner support、CoreIr、
ControlFlowIr、VC、proof payload は credit しない。
task 100 の fail case は当初、parser / resolver 実行後の builtin membership
theorem formula extraction-gap boundary だけを credit した。task 108 は exact
`BuiltinMembershipPayloadBoundary: 1 in 1` sidecar について real checker
term/formula payload extraction を credit するが、numeric type payload、
membership operand expected-type construction/checking、recorded fact、theorem
acceptance、`formula_statement` runner support、CoreIr、ControlFlowIr、VC、proof
payload は credit しない。
task 101 の fail case は当初、parser / resolver 実行後の builtin inequality
theorem formula extraction-gap boundary だけを credit した。task 107 は exact
`BuiltinInequalityPayloadBoundary: 1 <> 2` sidecar について real checker
term/formula payload extraction を credit するが、numeric type payload、
inequality desugaring または equality semantic checking、recorded fact、theorem
acceptance、`formula_statement` runner support、CoreIr、ControlFlowIr、VC、proof
payload は credit しない。task 119 は separate exact pass row
`reserve x for set; theorem ReservedVariableEqualityPayloadBoundary: x = x;`
を追加する。2 つの identifier term は real reserve `BindingEnv` で解決され、
result type と equality expected type は記述された builtin `set` reserve から
導かれ、checker type/well-formedness は diagnostic/fact なしで完了する。implicit
universal closure、equality truth、theorem acceptance、`formula_statement`、proof、
CoreIr、ControlFlowIr、VC は credit しない。task 123 は distinct-binding exact
pass row
`reserve x, y for set; theorem DistinctReservedVariableEqualityPayloadBoundary: x = y;`
を追加する。real multi-reserve producer は 2 binding identity と共有された記述上の
builtin `set` type range 1 個を保存し、independent source-order lookup と operand
ごとの result/expected role は fact-free `Checked` equality へ到達する。implicit
closure/order、equality truth/fact、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は credit しない。task 124 は separate exact pass row
`reserve x for set; reserve y for set; theorem MultipleReserveDeclarationEqualityPayloadBoundary: x = y;`
を追加する。この row は 2 declaration 固有の written type range を 4
pre-normalization result/expected input に保持し、checker は同一の builtin `set`
semantics を deterministic に 1 normalized type へ intern する。exact
multiple-declaration type/well-formedness だけを credit し、implicit closure/order、
equality truth/fact、theorem acceptance、proof、CoreIr、ControlFlowIr、VC は credit
しない。task 125 は heterogeneous exact pass row
`reserve x for object; reserve y for set; theorem HeterogeneousReserveMembershipPayloadBoundary: x in y;`
を追加する。左 result は `object`、右 result と唯一の expected input は `set` を
保持する。checker は 2 normalized identity を記録し、右の両 role で `set`
identity を共有する。exact heterogeneous membership type/well-formedness だけを
credit し、membership truth/fact、object/set coercion、implicit closure/order、
theorem acceptance、proof、CoreIr、ControlFlowIr、VC は credit しない。task 126 は
exact direct-local-mode pass row
`definition mode LocalModeFormulaDef: LocalModeFormula is set; end; reserve x for LocalModeFormula; theorem LocalModeReservedVariableEqualityPayloadBoundary: x = x;`
を追加する。4 個の raw result/expected input はすべて記述された local-mode symbol
と reserve range を保持する。checker は real AST-derived bare-set mode expansion を
消費し、1 個の normalized builtin-`set` identity をその expansion RHS に anchor
して、2 個の `Inferred` variable と 1 個の fact-free `Checked` equality を記録する。
これは exact direct local-mode type/well-formedness handoff だけを credit し、mode
definition declaration checking/acceptance、inhabitation evidence、implicit
closure/order、equality truth/fact、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は credit しない。task 127 は exact one-edge local-mode-chain pass row
`definition mode BaseModeFormulaDef: BaseModeFormula is set; end; definition mode ChainModeFormulaDef: ChainModeFormula is BaseModeFormula; end; reserve x for ChainModeFormula; theorem ChainedLocalModeReservedVariableEqualityPayloadBoundary: x = x;`
を追加する。4 raw result/expected input はすべて記述された outer-mode symbol と
reserve range を保持する。checker は real AST-derived expansion link 2 個を消費し、
1 normalized builtin-`set` identity を terminal `set` RHS に anchor して、2
`Inferred` variable と 1 fact-free `Checked` equality を記録する。これは exact
one-edge-chain type/well-formedness handoff だけを credit し、mode-definition
declaration checking/acceptance、inhabitation evidence、object terminal、
longer-chain formula、closure/order、equality truth/fact、theorem acceptance、proof、
CoreIr、ControlFlowIr、VC は credit しない。task 128 は exact direct
local-object-mode pass row
`definition mode LocalObjectModeDef: LocalObjectMode is object; end; reserve x for LocalObjectMode; theorem LocalObjectModeReservedVariableEqualityPayloadBoundary: x = x;`
を追加する。4 raw result/expected input は written object-mode symbol と reserve range
を保持する。checker は real AST-derived expansion を消費し、real `object` RHS に 1
normalized builtin-`object` identity を anchor し、2 `Inferred` variable と 1
fact-free `Checked` equality を記録する。これは exact direct local-object-mode
type/well-formedness handoff だけを credit し、mode-definition declaration
checking/acceptance、inhabitation evidence、broader object-mode formula、
closure/order、equality truth/fact、theorem acceptance、proof、CoreIr、
ControlFlowIr、VC は credit しない。
task 129 は exact one-edge local-object-mode-chain pass row
`definition mode BaseObjectModeDef: BaseObjectMode is object; end; definition mode ChainObjectModeDef: ChainObjectMode is BaseObjectMode; end; reserve z for ChainObjectMode; theorem ChainedLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;`
を追加する。4 raw outer-mode role を保持し、両 real expansion は terminal RHS に
anchor された 1 builtin-object identity に normalize される。exact
type/well-formedness だけを credit し、declaration acceptance/inhabitation、longer
chain、closure/order、truth/fact、theorem acceptance、proof、CoreIr、ControlFlowIr、
VC は credit しない。
task 130 は exact direct local-mode inequality pass row
`definition mode LocalModeInequalityDef: LocalModeInequality is set; end; reserve x for LocalModeInequality; theorem LocalModeReservedVariableInequalityPayloadBoundary: x <> x;`
を追加する。4 raw role は real RHS expansion 1 本を通じて builtin-set identity
1 個に normalize され、fact-free pre-desugaring `Checked` inequality は exact
type/well-formedness だけを credit する。declaration acceptance、truth、proof、
Core、VC は credit しない。
task 131 は exact direct local-object-mode inequality pass row
`definition mode LocalObjectModeInequalityDef: LocalObjectModeInequality is object; end; reserve x for LocalObjectModeInequality; theorem LocalObjectModeReservedVariableInequalityPayloadBoundary: x <> x;`
を追加する。4 raw object-mode role は real RHS expansion 1 本を通じて
builtin-object identity 1 個に normalize され、fact-free pre-desugaring
`Checked` inequality は exact type/well-formedness だけを credit する。mode
declaration acceptance/inhabitation、truth、proof、Core、VC は credit しない。
task 132 は exact one-edge local-mode-chain inequality pass row
`definition mode BaseModeInequalityDef: BaseModeInequality is set; end; definition mode ChainModeInequalityDef: ChainModeInequality is BaseModeInequality; end; reserve x for ChainModeInequality; theorem ChainedLocalModeReservedVariableInequalityPayloadBoundary: x <> x;`
を追加する。4 raw outer-mode role は real expansion link 2 本を通じて
terminal-RHS builtin-set identity 1 個に normalize され、fact-free
pre-desugaring `Checked` inequality は exact type/well-formedness だけを credit
する。declaration acceptance/inhabitation、desugaring、truth、proof、Core、VC は
credit しない。
task 133 は exact one-edge local-object-mode-chain inequality pass row
`definition mode BaseObjectModeInequalityDef: BaseObjectModeInequality is object; end; definition mode ChainObjectModeInequalityDef: ChainObjectModeInequality is BaseObjectModeInequality; end; reserve z for ChainObjectModeInequality; theorem ChainedLocalObjectModeReservedVariableInequalityPayloadBoundary: z <> z;`
を追加する。4 raw outer-mode role は real expansion link 2 本を通じて
terminal-RHS builtin-object identity 1 個に normalize され、fact-free
pre-desugaring `Checked` inequality は exact type/well-formedness だけを credit
する。declaration acceptance/inhabitation、desugaring、truth、proof、Core、VC は
credit しない。
task 134 は exact two-edge local-mode-chain equality pass row
`definition mode BaseTwoEdgeModeEqualityDef: BaseTwoEdgeModeEquality is set; end; definition mode MiddleTwoEdgeModeEqualityDef: MiddleTwoEdgeModeEquality is BaseTwoEdgeModeEquality; end; definition mode OuterTwoEdgeModeEqualityDef: OuterTwoEdgeModeEquality is MiddleTwoEdgeModeEquality; end; reserve z for OuterTwoEdgeModeEquality; theorem TwoEdgeLocalModeReservedVariableEqualityPayloadBoundary: z = z;`
を追加する。4 raw outer-mode role は real expansion link 3 本を通じて
terminal-RHS builtin-set identity 1 個に normalize され、fact-free `Checked`
equality は exact type/well-formedness だけを credit する。declaration
acceptance/inhabitation、implicit closure/order、truth、proof、Core、VC は credit
しない。
task 135 は exact two-edge local-object-mode-chain equality pass row
`definition mode BaseTwoEdgeObjectModeEqualityDef: BaseTwoEdgeObjectModeEquality is object; end; definition mode MiddleTwoEdgeObjectModeEqualityDef: MiddleTwoEdgeObjectModeEquality is BaseTwoEdgeObjectModeEquality; end; definition mode OuterTwoEdgeObjectModeEqualityDef: OuterTwoEdgeObjectModeEquality is MiddleTwoEdgeObjectModeEquality; end; reserve z for OuterTwoEdgeObjectModeEquality; theorem TwoEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;`
を追加する。4 raw outer-mode role は real expansion link 3 本を通じて
terminal-RHS builtin-object identity 1 個に normalize され、fact-free `Checked`
equality は exact type/well-formedness だけを credit する。declaration
acceptance/inhabitation、implicit closure/order、truth、proof、Core、VC は credit
しない。
task 136 は exact two-edge local-mode-chain inequality pass row
`definition mode BaseTwoEdgeModeInequalityDef: BaseTwoEdgeModeInequality is set; end; definition mode MiddleTwoEdgeModeInequalityDef: MiddleTwoEdgeModeInequality is BaseTwoEdgeModeInequality; end; definition mode OuterTwoEdgeModeInequalityDef: OuterTwoEdgeModeInequality is MiddleTwoEdgeModeInequality; end; reserve z for OuterTwoEdgeModeInequality; theorem TwoEdgeLocalModeReservedVariableInequalityPayloadBoundary: z <> z;`
を追加する。4 raw outer-mode role は real expansion link 3 本を通じて
terminal-RHS builtin-set identity 1 個に normalize され、fact-free
pre-desugaring `Checked` inequality は exact type/well-formedness だけを credit
する。mode declaration acceptance/inhabitation、inequality desugaring、implicit
closure/order、truth、proof、Core、VC は credit しない。
task 137 は exact two-edge local-object-mode-chain inequality pass row
`definition mode BaseTwoEdgeObjectModeInequalityDef: BaseTwoEdgeObjectModeInequality is object; end; definition mode MiddleTwoEdgeObjectModeInequalityDef: MiddleTwoEdgeObjectModeInequality is BaseTwoEdgeObjectModeInequality; end; definition mode OuterTwoEdgeObjectModeInequalityDef: OuterTwoEdgeObjectModeInequality is MiddleTwoEdgeObjectModeInequality; end; reserve z for OuterTwoEdgeObjectModeInequality; theorem TwoEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary: z <> z;`
を追加する。4 raw outer-mode role は real expansion link 3 本を通じて
terminal-RHS builtin-object identity 1 個に normalize され、fact-free
pre-desugaring `Checked` inequality は exact type/well-formedness だけを credit
する。declaration acceptance/inhabitation、inequality desugaring、implicit
closure/order、truth、proof、Core、VC は credit しない。
task 138 は exact direct local-mode reserved-variable type-assertion pass row
`definition mode LocalModeTypeAssertionDef: LocalModeTypeAssertion is set; end; reserve x for LocalModeTypeAssertion; theorem LocalModeReservedVariableTypeAssertionPayloadBoundary: x is set;`
を追加する。raw subject は記述された local-mode provenance、asserted builtin
`set` は独立した formula source を保持し、real expansion 1 本が両者を
terminal-RHS builtin-set identity 1 個へ normalize する。1 `Inferred` term と 1
fact-free `Checked` type assertion は exact normalized-reflexive
type/well-formedness だけを credit し、mode declaration
acceptance/inhabitation、formula-side local-mode asserted head、general
reachability/widening/`qua`、truth、proof、Core、VC は credit しない。
task 139 は exact direct local-mode left reserved-variable membership pass row
`definition mode LocalModeMembershipDef: LocalModeMembership is set; end; reserve x for LocalModeMembership; reserve y for set; theorem LocalModeReservedVariableMembershipPayloadBoundary: x in y;`
を追加する。raw left result は記述された local-mode provenance、right result と
sole expected-set input は独立した explicit reserve provenance を保持する。real
expansion 1 本が left role を normalize し、right builtin-set role は直接
normalize され、3 role すべてが terminal-RHS builtin-set identity 1 個へ intern
する。2 `Inferred` term と 1 fact-free `Checked` membership は exact
type/well-formedness だけを credit し、mode declaration
acceptance/inhabitation、membership truth/fact、implicit closure/order、theorem
acceptance、proof、Core、VC は credit しない。
task 140 は exact direct local-object-mode left reserved-variable membership
pass row
`definition mode LocalObjectModeMembershipDef: LocalObjectModeMembership is object; end; reserve x for LocalObjectModeMembership; reserve y for set; theorem LocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`
を追加する。raw left result は記述された local object-mode provenance、right
result と sole expected-set input は独立した explicit reserve provenance を保持する。
real expansion 1 本が left を terminal-RHS builtin-object identity へ normalize
し、right role は distinct explicit-reserve-anchored builtin-set identity へ直接
normalize する。2 `Inferred` term と 1 fact-free `Checked` membership は exact
type/well-formedness だけを credit し、mode declaration
acceptance/inhabitation、membership truth/fact、object/set coercion、implicit
closure/order、theorem acceptance、proof、Core、VC は credit しない。
task 141 は exact one-edge local-mode-chain left reserved-variable membership
pass row
`definition mode BaseModeMembershipDef: BaseModeMembership is set; end; definition mode ChainModeMembershipDef: ChainModeMembership is BaseModeMembership; end; reserve x for ChainModeMembership; reserve y for set; theorem ChainedLocalModeReservedVariableMembershipPayloadBoundary: x in y;`
を追加する。raw left result は written outer-mode provenance、right result と sole
expected-set input は独立した explicit reserve provenance を保持する。real
expansion 2 本が left を terminal set RHS へ recursive に normalize し、right
role は直接 normalize され、3 role すべてが terminal-RHS builtin-set identity 1
個へ intern する。2 `Inferred` term と 1 fact-free `Checked` membership は exact
type/well-formedness だけを credit し、mode declaration
acceptance/inhabitation、membership truth/fact、implicit closure/order、theorem
acceptance、proof、Core、VC は credit しない。
task 142 は exact one-edge local-object-mode-chain left reserved-variable
membership pass row
`definition mode BaseObjectModeMembershipDef: BaseObjectModeMembership is object; end; definition mode ChainObjectModeMembershipDef: ChainObjectModeMembership is BaseObjectModeMembership; end; reserve x for ChainObjectModeMembership; reserve y for set; theorem ChainedLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`
を追加する。raw left result は written outer-mode provenance、right result と sole
expected-set input は独立した explicit reserve provenance を保持する。real
expansion 2 本が left を terminal object RHS へ recursive に normalize し、right
role は直接 distinct explicit-reserve-anchored builtin-set identity 1 個へ
normalize される。2 `Inferred` term と 1 fact-free `Checked` membership は exact
type/well-formedness だけを credit し、mode declaration
acceptance/inhabitation、membership truth/fact、object/set coercion、implicit
closure/order、theorem acceptance、proof、Core、VC は credit しない。
task 143 は exact two-edge local-mode-chain left reserved-variable membership
pass row
`definition mode BaseTwoEdgeModeMembershipDef: BaseTwoEdgeModeMembership is set; end; definition mode MiddleTwoEdgeModeMembershipDef: MiddleTwoEdgeModeMembership is BaseTwoEdgeModeMembership; end; definition mode OuterTwoEdgeModeMembershipDef: OuterTwoEdgeModeMembership is MiddleTwoEdgeModeMembership; end; reserve x for OuterTwoEdgeModeMembership; reserve y for set; theorem TwoEdgeLocalModeReservedVariableMembershipPayloadBoundary: x in y;`
を追加する。raw left result は written outer-mode provenance、right result と sole
expected-set input は独立した explicit reserve provenance を保持する。real
expansion 3 本が left を terminal set RHS へ再帰的に normalize し、right role は
直接 normalize され、3 role すべてが terminal-RHS builtin-set identity 1 個へ
intern する。2 `Inferred` term と 1 fact-free `Checked` membership は exact
type/well-formedness だけを credit し、mode declaration acceptance/inhabitation、
membership truth/fact、implicit closure/order、theorem acceptance、proof、Core、
VC は credit しない。
task 144 は exact two-edge local-object-mode-chain left reserved-variable
membership pass row
`definition mode BaseTwoEdgeObjectModeMembershipDef: BaseTwoEdgeObjectModeMembership is object; end; definition mode MiddleTwoEdgeObjectModeMembershipDef: MiddleTwoEdgeObjectModeMembership is BaseTwoEdgeObjectModeMembership; end; definition mode OuterTwoEdgeObjectModeMembershipDef: OuterTwoEdgeObjectModeMembership is MiddleTwoEdgeObjectModeMembership; end; reserve x for OuterTwoEdgeObjectModeMembership; reserve y for set; theorem TwoEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`
を追加する。raw left result は written outer-mode provenance、right result と sole
expected-set input は独立した explicit reserve provenance を保持する。real
expansion 3 本が left を terminal object RHS へ再帰的に normalize し、right
role は distinct explicit-reserve-anchored builtin-set identity へ直接 normalize
される。2 `Inferred` term と 1 fact-free `Checked` membership は exact
type/well-formedness だけを credit し、mode declaration acceptance/
inhabitation、membership truth/fact、object/set coercion、implicit closure/
order、theorem acceptance、proof、Core、VC は credit しない。
task 145 は exact direct local-object-mode reserved-variable type assertion
pass row
`definition mode LocalObjectModeTypeAssertionDef: LocalObjectModeTypeAssertion is object; end; reserve x for LocalObjectModeTypeAssertion; theorem LocalObjectModeReservedVariableTypeAssertionPayloadBoundary: x is object;`
を追加する。raw subject result は written local-mode provenance、asserted
builtin `object` は独立した formula-anchored source node を保持する。real
expansion 1 本が両 input を terminal-RHS builtin-object identity 1 個へ normalize
してから、1 `Inferred` term と 1 fact-free `Checked` type assertion が exact
normalized-reflexive type/well-formedness だけを credit する。mode declaration
acceptance/inhabitation、formula-side local-mode asserted head、general
reachability/widening/`qua`、object/set coercion、truth/fact、closure/order、
theorem acceptance、proof、Core、VC は credit しない。
task 146 は exact one-edge local-mode-chain reserved-variable type assertion
pass row
`definition mode BaseModeTypeAssertionDef: BaseModeTypeAssertion is set; end; definition mode ChainModeTypeAssertionDef: ChainModeTypeAssertion is BaseModeTypeAssertion; end; reserve x for ChainModeTypeAssertion; theorem ChainedLocalModeReservedVariableTypeAssertionPayloadBoundary: x is set;`
を追加する。raw subject result は written outer-mode provenance、asserted
builtin `set` は独立した formula-anchored source node を保持する。real expansion
2 本が両 input を terminal-RHS builtin-set identity 1 個へ再帰的に normalize
してから、1 `Inferred` term と 1 fact-free `Checked` type assertion が exact
normalized-reflexive type/well-formedness だけを credit する。mode declaration
acceptance/inhabitation、formula-side local-mode asserted head、general
reachability/widening/`qua`、truth/fact、closure/order、theorem acceptance、proof、
Core、VC は credit しない。
task 147 は exact one-edge local-object-mode-chain reserved-variable type
assertion pass row
`definition mode BaseObjectModeTypeAssertionDef: BaseObjectModeTypeAssertion is object; end; definition mode ChainObjectModeTypeAssertionDef: ChainObjectModeTypeAssertion is BaseObjectModeTypeAssertion; end; reserve x for ChainObjectModeTypeAssertion; theorem ChainedLocalObjectModeReservedVariableTypeAssertionPayloadBoundary: x is object;`
を追加する。raw subject result は written outer-mode provenance、asserted
builtin `object` は独立した formula-anchored source node を保持する。real
expansion 2 本が両 input を terminal-RHS builtin-object identity 1 個へ
再帰的に normalize してから、1 `Inferred` term と 1 fact-free `Checked` type
assertion が exact normalized-reflexive type/well-formedness だけを credit する。
mode declaration acceptance/inhabitation、formula-side local-mode asserted
head、general reachability/widening/`qua`、object/set coercion、truth/fact、
closure/order、theorem acceptance、proof、Core、VC は credit しない。
task 148 は exact two-edge local-mode-chain reserved-variable type assertion
pass row
`definition mode BaseTwoEdgeModeTypeAssertionDef: BaseTwoEdgeModeTypeAssertion is set; end; definition mode MiddleTwoEdgeModeTypeAssertionDef: MiddleTwoEdgeModeTypeAssertion is BaseTwoEdgeModeTypeAssertion; end; definition mode OuterTwoEdgeModeTypeAssertionDef: OuterTwoEdgeModeTypeAssertion is MiddleTwoEdgeModeTypeAssertion; end; reserve x for OuterTwoEdgeModeTypeAssertion; theorem TwoEdgeLocalModeReservedVariableTypeAssertionPayloadBoundary: x is set;`
を追加する。raw subject result は written outer-mode provenance、asserted
builtin `set` は独立した formula-anchored source node を保持する。
real expansion 3 本が両 input を terminal-RHS builtin-set identity 1 個へ再帰的に
normalize してから、1 `Inferred` term と 1 fact-free `Checked` type assertion
が exact normalized-reflexive type/well-formedness だけを credit する。mode
declaration acceptance/inhabitation、formula-side local-mode asserted head、
general reachability/widening/`qua`、truth/fact、closure/order、theorem
acceptance、proof、Core、VC は credit しない。
task 149 は次の exact two-edge local-object-mode-chain reserved-variable type
assertion pass row を追加する:
`definition mode BaseTwoEdgeObjectModeTypeAssertionDef: BaseTwoEdgeObjectModeTypeAssertion is object; end; definition mode MiddleTwoEdgeObjectModeTypeAssertionDef: MiddleTwoEdgeObjectModeTypeAssertion is BaseTwoEdgeObjectModeTypeAssertion; end; definition mode OuterTwoEdgeObjectModeTypeAssertionDef: OuterTwoEdgeObjectModeTypeAssertion is MiddleTwoEdgeObjectModeTypeAssertion; end; reserve x for OuterTwoEdgeObjectModeTypeAssertion; theorem TwoEdgeLocalObjectModeReservedVariableTypeAssertionPayloadBoundary: x is object;`
raw subject result は written outer-mode provenance、asserted
builtin `object` は独立した formula-anchored source node を保持する。real
expansion 3 本が両 input を terminal-RHS builtin-object identity 1 個へ再帰的に
normalize してから、1 `Inferred` term と 1 fact-free `Checked` type assertion が
exact normalized-reflexive type/well-formedness だけを credit する。mode
declaration acceptance/inhabitation、formula-side local-mode asserted head、
general reachability/widening/`qua`、object/set coercion、truth/fact、closure/
order、theorem acceptance、proof、Core、VC は credit しない。exact source
guard、独立した definition/three-link corruption、real frontend/resolver
sidecar が active row を保護する。
task 150 は次の exact three-edge local-mode-chain reserved-variable type
assertion pass row を追加する:
`definition mode BaseThreeEdgeModeTypeAssertionDef: BaseThreeEdgeModeTypeAssertion is set; end; definition mode InnerThreeEdgeModeTypeAssertionDef: InnerThreeEdgeModeTypeAssertion is BaseThreeEdgeModeTypeAssertion; end; definition mode MiddleThreeEdgeModeTypeAssertionDef: MiddleThreeEdgeModeTypeAssertion is InnerThreeEdgeModeTypeAssertion; end; definition mode OuterThreeEdgeModeTypeAssertionDef: OuterThreeEdgeModeTypeAssertion is MiddleThreeEdgeModeTypeAssertion; end; reserve x for OuterThreeEdgeModeTypeAssertion; theorem ThreeEdgeLocalModeReservedVariableTypeAssertionPayloadBoundary: x is set;`
raw subject result は written outer-mode provenance、asserted builtin `set` は
独立した formula-anchored source node を保持する。real expansion 4 本が両
input を terminal-RHS builtin-set identity 1 個へ再帰的に normalize してから、
1 `Inferred` term と 1 fact-free `Checked` type assertion が exact normalized-
reflexive type/well-formedness だけを credit する。mode declaration acceptance/
inhabitation、formula-side local-mode asserted head、general reachability/
widening/`qua`、truth/fact、closure/order、theorem acceptance、proof、Core、VC
は credit しない。exact source guard、独立した definition/four-link
corruption、real frontend/resolver sidecar で active row を保護する。
task 151 は次の exact three-edge local-object-mode-chain reserved-variable type
assertion pass row を追加する:
`definition mode BaseThreeEdgeObjectModeTypeAssertionDef: BaseThreeEdgeObjectModeTypeAssertion is object; end; definition mode InnerThreeEdgeObjectModeTypeAssertionDef: InnerThreeEdgeObjectModeTypeAssertion is BaseThreeEdgeObjectModeTypeAssertion; end; definition mode MiddleThreeEdgeObjectModeTypeAssertionDef: MiddleThreeEdgeObjectModeTypeAssertion is InnerThreeEdgeObjectModeTypeAssertion; end; definition mode OuterThreeEdgeObjectModeTypeAssertionDef: OuterThreeEdgeObjectModeTypeAssertion is MiddleThreeEdgeObjectModeTypeAssertion; end; reserve x for OuterThreeEdgeObjectModeTypeAssertion; theorem ThreeEdgeLocalObjectModeReservedVariableTypeAssertionPayloadBoundary: x is object;`
raw subject result は written outer-mode provenance、asserted builtin `object` は
独立した formula-anchored source node を保持する。real expansion 4 本が両 input
を terminal-RHS builtin-object identity 1 個へ再帰的に normalize してから、1
`Inferred` term と 1 fact-free `Checked` type assertion が exact normalized-
reflexive type/well-formedness だけを credit する。mode declaration acceptance/
inhabitation、formula-side local-mode asserted head、general reachability/
widening/`qua`、object/set coercion、truth/fact、closure/order、theorem
acceptance、proof、Core、VC は credit しない。exact source guard、独立した
definition/four-link corruption、real frontend/resolver sidecar で active row
を保護する。
task 152 は次の exact four-edge local-mode-chain reserved-variable type
assertion pass row を追加する:
`definition mode BaseFourEdgeModeTypeAssertionDef: BaseFourEdgeModeTypeAssertion is set; end; definition mode InnerFourEdgeModeTypeAssertionDef: InnerFourEdgeModeTypeAssertion is BaseFourEdgeModeTypeAssertion; end; definition mode MiddleFourEdgeModeTypeAssertionDef: MiddleFourEdgeModeTypeAssertion is InnerFourEdgeModeTypeAssertion; end; definition mode OuterFourEdgeModeTypeAssertionDef: OuterFourEdgeModeTypeAssertion is MiddleFourEdgeModeTypeAssertion; end; definition mode TooDeepFourEdgeModeTypeAssertionDef: TooDeepFourEdgeModeTypeAssertion is OuterFourEdgeModeTypeAssertion; end; reserve x for TooDeepFourEdgeModeTypeAssertion; theorem FourEdgeLocalModeReservedVariableTypeAssertionPayloadBoundary: x is set;`
raw subject result は written outermost-mode provenance、asserted builtin `set`
は独立した formula-anchored source node を保持する。real expansion 5 本が両
input を terminal-RHS builtin-set identity 1 個へ再帰的に normalize してから、
1 `Inferred` term と 1 fact-free `Checked` type assertion が exact normalized-
reflexive type/well-formedness だけを credit する。mode declaration acceptance/
inhabitation、formula-side local-mode asserted head、general reachability/
widening/`qua`、truth/fact、closure/order、theorem acceptance、proof、Core、VC
は credit しない。exact source guard、独立した definition/five-link
corruption、real frontend/resolver sidecar で active row を保護する。
task 153 は次の exact four-edge local-object-mode-chain reserved-variable type
assertion pass row を追加する:
`definition mode BaseFourEdgeObjectModeTypeAssertionDef: BaseFourEdgeObjectModeTypeAssertion is object; end; definition mode InnerFourEdgeObjectModeTypeAssertionDef: InnerFourEdgeObjectModeTypeAssertion is BaseFourEdgeObjectModeTypeAssertion; end; definition mode MiddleFourEdgeObjectModeTypeAssertionDef: MiddleFourEdgeObjectModeTypeAssertion is InnerFourEdgeObjectModeTypeAssertion; end; definition mode OuterFourEdgeObjectModeTypeAssertionDef: OuterFourEdgeObjectModeTypeAssertion is MiddleFourEdgeObjectModeTypeAssertion; end; definition mode TooDeepFourEdgeObjectModeTypeAssertionDef: TooDeepFourEdgeObjectModeTypeAssertion is OuterFourEdgeObjectModeTypeAssertion; end; reserve x for TooDeepFourEdgeObjectModeTypeAssertion; theorem FourEdgeLocalObjectModeReservedVariableTypeAssertionPayloadBoundary: x is object;`
raw subject result は written outermost-mode provenance、asserted builtin
`object` は独立した formula-anchored source node を保持する。real expansion 5
本が両 input を terminal-RHS builtin-object identity 1 個へ再帰的に normalize
してから、1 `Inferred` term と 1 fact-free `Checked` type assertion が exact
normalized-reflexive type/well-formedness だけを credit する。declaration
acceptance/inhabitation、formula-side local asserted head、general reachability/
widening/`qua`、object/set coercion、truth/fact、closure/order、theorem
acceptance、proof/Core/VC は credit しない。exact source guard、独立した
definition/five-link corruption、real frontend/resolver sidecar で active row
を保護する。
task 154 は次の test-first exact three-edge local-mode-chain reserved-variable
equality pass row を追加する:
`definition mode BaseThreeEdgeModeEqualityDef: BaseThreeEdgeModeEquality is set; end; definition mode InnerThreeEdgeModeEqualityDef: InnerThreeEdgeModeEquality is BaseThreeEdgeModeEquality; end; definition mode MiddleThreeEdgeModeEqualityDef: MiddleThreeEdgeModeEquality is InnerThreeEdgeModeEquality; end; definition mode OuterThreeEdgeModeEqualityDef: OuterThreeEdgeModeEquality is MiddleThreeEdgeModeEquality; end; reserve z for OuterThreeEdgeModeEquality; theorem ThreeEdgeLocalModeReservedVariableEqualityPayloadBoundary: z = z;`
raw result/expected input 4 個は written outer-mode provenance を保持し、両
operand は ordinal 1、2 で `BindingId(0)` へ解決し、real expansion 4 本は全
role を terminal-RHS builtin-set identity 1 個へ normalize してから 2
`Inferred` term と 1 fact/deferred-free `Checked` equality が exact type/well-
formedness だけを credit する。mode declaration acceptance/inhabitation、
equality truth/fact、closure/order、theorem acceptance、proof/Core/VC は credit
しない。trace row、production route、独立した corruption matrix、real
frontend/resolver sidecar が active row を保護する。
task 155 は次の test-first exact three-edge local-object-mode-chain reserved-
variable equality pass row を追加する:
`definition mode BaseThreeEdgeObjectModeEqualityDef: BaseThreeEdgeObjectModeEquality is object; end; definition mode InnerThreeEdgeObjectModeEqualityDef: InnerThreeEdgeObjectModeEquality is BaseThreeEdgeObjectModeEquality; end; definition mode MiddleThreeEdgeObjectModeEqualityDef: MiddleThreeEdgeObjectModeEquality is InnerThreeEdgeObjectModeEquality; end; definition mode OuterThreeEdgeObjectModeEqualityDef: OuterThreeEdgeObjectModeEquality is MiddleThreeEdgeObjectModeEquality; end; reserve z for OuterThreeEdgeObjectModeEquality; theorem ThreeEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;`
raw result/expected input 4 個は written outer-mode provenance を保持し、両
operand は ordinal 1、2 で `BindingId(0)` へ解決し、real expansion 4 本は全
role を terminal-RHS builtin-object identity 1 個へ normalize してから 2
`Inferred` term と 1 fact/deferred-free `Checked` equality が exact type/well-
formedness だけを credit する。mode declaration acceptance/inhabitation、
object/set coercion、equality truth/fact、closure/order、theorem acceptance、
proof/Core/VC は credit しない。trace row、production route、独立した corruption
matrix、real frontend/resolver sidecar が active row を保護する。
task 156 は次の test-first exact three-edge local-mode-chain reserved-variable
inequality pass row を追加する:
`definition mode BaseThreeEdgeModeInequalityDef: BaseThreeEdgeModeInequality is set; end; definition mode InnerThreeEdgeModeInequalityDef: InnerThreeEdgeModeInequality is BaseThreeEdgeModeInequality; end; definition mode MiddleThreeEdgeModeInequalityDef: MiddleThreeEdgeModeInequality is InnerThreeEdgeModeInequality; end; definition mode OuterThreeEdgeModeInequalityDef: OuterThreeEdgeModeInequality is MiddleThreeEdgeModeInequality; end; reserve z for OuterThreeEdgeModeInequality; theorem ThreeEdgeLocalModeReservedVariableInequalityPayloadBoundary: z <> z;`
raw result/expected input 4 個は written outer-mode provenance を保持し、両
operand は ordinal 1、2 で `BindingId(0)` へ解決し、real expansion 4 本は全
role を terminal-RHS builtin-set identity 1 個へ normalize してから 2
`Inferred` term と 1 fact/deferred-free pre-desugaring `Checked` inequality が
exact type/well-formedness だけを credit する。mode declaration acceptance/
inhabitation、inequality desugaring、truth/fact、closure/order、theorem
acceptance、proof/Core/VC は credit しない。trace row、production route、独立
した corruption matrix、real frontend/resolver sidecar が active row を保護する。
task 157 は次の exact three-edge local-object-mode-chain reserved-
variable inequality pass row を追加する:
`definition mode BaseThreeEdgeObjectModeInequalityDef: BaseThreeEdgeObjectModeInequality is object; end; definition mode InnerThreeEdgeObjectModeInequalityDef: InnerThreeEdgeObjectModeInequality is BaseThreeEdgeObjectModeInequality; end; definition mode MiddleThreeEdgeObjectModeInequalityDef: MiddleThreeEdgeObjectModeInequality is InnerThreeEdgeObjectModeInequality; end; definition mode OuterThreeEdgeObjectModeInequalityDef: OuterThreeEdgeObjectModeInequality is MiddleThreeEdgeObjectModeInequality; end; reserve z for OuterThreeEdgeObjectModeInequality; theorem ThreeEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary: z <> z;`
raw result/expected input 4 個は written outer-mode provenance を保持し、両
operand は ordinal 1、2 で `BindingId(0)` へ解決し、real expansion 4 本は全
role を terminal-RHS builtin-object identity 1 個へ normalize してから 2
`Inferred` term と 1 fact/deferred-free pre-desugaring `Checked` inequality が
exact type/well-formedness だけを credit する。mode declaration acceptance/
inhabitation、object/set coercion、inequality desugaring、truth/fact、closure/
order、theorem acceptance、proof/Core/VC は credit しない。trace row、
production route、独立した corruption matrix、real frontend/resolver sidecar
が active row を保護する。
task 158 は次の exact active three-edge local-mode-chain left reserved-
variable membership pass row を追加する:
`definition mode BaseThreeEdgeModeMembershipDef: BaseThreeEdgeModeMembership is set; end; definition mode InnerThreeEdgeModeMembershipDef: InnerThreeEdgeModeMembership is BaseThreeEdgeModeMembership; end; definition mode MiddleThreeEdgeModeMembershipDef: MiddleThreeEdgeModeMembership is InnerThreeEdgeModeMembership; end; definition mode OuterThreeEdgeModeMembershipDef: OuterThreeEdgeModeMembership is MiddleThreeEdgeModeMembership; end; reserve x for OuterThreeEdgeModeMembership; reserve y for set; theorem ThreeEdgeLocalModeReservedVariableMembershipPayloadBoundary: x in y;`
raw left result は written outer-mode provenance、right result と sole expected-
set input は独立した explicit reserve provenance を保持し、left expected type
は持たない。operand を ordinal 2/3 で `BindingId(0/1)` へ解決し、続いて real
expansion 4 本が 3 role を terminal-RHS builtin-set identity 1 個へ normalize
してから、2 `Inferred` term、1 fact/deferred-free `Checked` membership、exactly
one right-owned constraint が exact type/well-formedness だけを credit する。
mode declaration acceptance/
inhabitation、membership truth/fact、closure/order、theorem acceptance、proof/
Core/VC、object-terminal behavior、broader chain depth は credit しない。trace
row、production route、独立した corruption matrix、real frontend/resolver
sidecar が active row を保護する。
task 159 は exact
`reserve x, y for set; theorem DistinctReservedVariableMembershipPayloadBoundary: x in y;`
の active row を追加する。credit するのは Chapters 4、13、14.5.3、16
だけである。reserve item 1 個は shared written set range 1 個上に distinct
binding を作り、identifier use 2 個は ordinal 2/3 で resolve され、left result
と right result/sole expected-set input は shared-source-anchored builtin-set
identity 1 個へ intern される前にその range を保持しなければならない。意図する
checker result は 2 `Inferred` variable と 1 fact/deferred-free `Checked`
membership、exactly one right-owned constraint、no left expected type である。
production routing、corruption/near-miss coverage、real frontend/resolver
sidecar が active row を保護する。Chapter 3、
truth/fact、closure/order、theorem acceptance、proof/Core/VC、separate
declaration、broader source shape は credit しない。
task 160 は exact `reserve x, y for set; theorem
DistinctReservedVariableInequalityPayloadBoundary: x <> y;` の active row
を追加する。credit は Chapter 4、13、14.5.2、16 だけである。reserve item 1 個は
shared written set range 1 個を持つ distinct binding を作り、identifier use 2 個は
ordinal 2/3 で解決され、result/expected role pair 2 組は shared-source-anchored
builtin-set identity 1 個へ intern する前にその range を保持しなければならない。
intended checker result は 2 `Inferred` variable と 2 ordered operand-owned constraint
を持つ 1 fact/deferred-free pre-desugaring `Checked` inequality である。production
route、corruption/near-miss coverage、real frontend/resolver sidecar が active row を
保護する。Chapter 3、desugaring/truth/fact、closure/order、
theorem acceptance、proof/Core/VC、separate declaration、broader source shape は
credit しない。
task 161 は exact `reserve x for set; reserve y for set; theorem
MultipleReserveDeclarationInequalityPayloadBoundary: x <> y;` の active row を追加する。
credit は Chapter 4、13、14.5.2、16 だけである。reserve item 2
個は distinct binding/written range を作り、use は ordinal 2/3 で解決され、result/
expected role pair 2 組は earlier `x` range に canonical anchor された builtin-set
identity 1 個へ intern する前に対応する range を保持しなければならない。intended
result は 2 `Inferred` variable と 2 ordered constraint を持つ 1 fact/deferred-free
pre-desugaring `Checked` inequality である。production route、corruption/near-miss
coverage、real sidecar が active row を保護する。
Chapter 3、shared-range behavior、desugaring/truth/fact、closure/order、theorem
acceptance、proof/Core/VC、broader shape は credit しない。
task 162 は exact `reserve x for set; reserve y for set; theorem
MultipleReserveDeclarationMembershipPayloadBoundary: x in y;` の active row
を追加する。参照は Chapter 4、13、14.5.3、16 だけである。reserve item 2 個は
distinct binding/written range を作り、use は ordinal 2/3 で解決され、left result
は first range、right result と sole right expected input は second range を保持し、
left expected input は持たない。intended result は earlier `x` range に canonical
anchor された builtin-set identity 1 個、2 `Inferred` variable、exactly one right-
owned constraint を持つ 1 fact/deferred-free `Checked` membership である。
production routing、corruption/near-miss coverage、real sidecar が active row を
保護するため active credit は 113 cases である。Chapter 3、shared-range
behavior、membership truth/fact、closure/order、theorem acceptance、proof/Core/
VC、broader shape は credit しない。
task 163 は `object` に終端する definition chain 4 本、
`reserve x for OuterThreeEdgeObjectModeMembership;`、`reserve y for set;`、
`ThreeEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`
だけの active row を追加する。Chapter 3、4、7、13、14.5.3、16 を既存
Chapter 4/13/14/16 requirement row、three-edge-chain row、dedicated checker row
を介して参照する。intended credit は real expansion 4 本、raw left / explicit-
set right provenance、ordinal 2/3 の `BindingId(0/1)`、distinct object/set
identity、no left expected input、2 inferred variable、exactly one right-owned
constraint を持つ 1 fact-free checked membership だけである。production route、
corruption/near-miss coverage、real sidecar、active count 114 がこれを保護し、
coercion、truth/fact、closure/order、theorem/proof/Core/VC、他の depth、broader
shape は credit しない。
task 164 は `set` に終端する definition chain 5 本、
`reserve x for TooDeepFourEdgeModeMembership;`、`reserve y for set;`、
`FourEdgeLocalModeReservedVariableMembershipPayloadBoundary: x in y;` だけの
active row を追加する。Chapter 4、7、13、14.5.3、16 を既存 Chapter
4/13/14/16 requirement row、structural-chain row、dedicated checker row を介して
参照する。intended credit は real expansion 5 本、raw left / explicit-set
right provenance、ordinal 2/3 の `BindingId(0/1)`、terminal-set-RHS identity
1 個、no left expected input、2 inferred variable、exactly one right-owned
constraint を持つ 1 fact-free checked membership だけである。6 trace backlink、
production route、corruption/near-miss coverage、real sidecar が active count
115 を保護する。truth/fact、closure/order、theorem/proof/
Core/VC、object-terminal behavior、他 depth、broader shape は credit しない。
task 165 は `object` に終端する definition chain 5 本、
`reserve x for TooDeepFourEdgeObjectModeMembership;`、`reserve y for set;`、
`FourEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;`
だけの active row を追加する。Chapter 3、4、7、13、14.5.3、16 を既存
Chapter 4/13/14/16 requirement row、structural-chain row、dedicated checker row
を介して参照する。intended credit は real expansion 5 本、raw left /
explicit-set right provenance、ordinal 2/3 の `BindingId(0/1)`、distinct
terminal-object-RHS / explicit-set identity、no left expected input、2 inferred
variable、exactly one right-owned constraint を持つ 1 fact-free checked
membership だけである。trace backlink 6 件、production route、corruption/
near-miss coverage、real sidecar が active count 116 を保護する。truth/fact、
object/set coercion、closure/order、theorem/proof/Core/VC、
他 depth、broader shape は credit しない。
task 166 は `set` に終端する definition chain 5 本、
`reserve z for TooDeepFourEdgeModeEquality;`、
`FourEdgeLocalModeReservedVariableEqualityPayloadBoundary: z = z;` だけの
active row を追加する。Chapter 4、7、13、14.5.2、16 を既存 Chapter
4/13/14/16 requirement row、structural-chain row、dedicated checker row を介して
参照する。intended credit は real expansion 5 本、raw outermost-mode result/
expected input 4 個、ordinal 1/2 の `BindingId(0)`、terminal-set-RHS identity
1 個、2 inferred variable、1 fact/deferred-free checked equality、ordered
operand-owned expected constraint 2 個だけである。trace backlink 6 件、
production routing、corruption/near-miss coverage、real sidecar が active count
117 を保護する。declaration acceptance/
inhabitation、truth/fact、closure/order、theorem/proof/Core/VC、object-terminal
behavior、他 depth、broader shape は credit しない。
task 167 は `object` に終端する definition chain 5 本、
`reserve z for TooDeepFourEdgeObjectModeEquality;`、
`FourEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;`
だけの active row を追加する。Chapter 3、4、7、13、14.5.2、16 を既存
Chapter 4/13/14/16 requirement row、structural-chain row、dedicated checker row
を介して参照する。intended credit は real expansion 5 本、raw outermost-mode
result/expected input 4 個、ordinal 1/2 の `BindingId(0)`、terminal-object-RHS
identity 1 個、2 inferred variable、1 fact/deferred-free checked equality、
ordered operand-owned expected constraint 2 個を object/set coercion なしで
保持する slice だけである。trace backlink 6 件、production routing、
corruption/near-miss coverage、real sidecar が active count 118 を保護する。
declaration acceptance/inhabitation、truth/fact、closure/
order、theorem/proof/Core/VC、set-terminal behavior、他 depth、broader shape は
credit しない。
task 168 は `set` に終端する definition chain 5 本、
`reserve z for TooDeepFourEdgeModeInequality;`、
`FourEdgeLocalModeReservedVariableInequalityPayloadBoundary: z <> z;` だけの
active row を追加する。Chapter 4、7、13、14.5.2、16 を既存 Chapter
4/13/14/16 requirement row、structural-chain row、dedicated checker row を介して
参照する。intended credit は real expansion 5 本、raw outermost-mode result/
expected input 4 個、ordinal 1/2 の `BindingId(0)`、terminal-set-RHS identity
1 個、2 inferred variable、1 fact/deferred-free pre-desugaring checked
inequality、ordered operand-owned expected constraint 2 個だけである。trace
backlink 6 件、production routing、corruption/near-miss coverage、real sidecar
が active count 119 を保護する。declaration acceptance/inhabitation、
inequality desugaring/truth/fact、closure/order、theorem/proof/Core/VC、object-
terminal behavior、他 depth、broader shape は credit しない。
task 169 は `object` に終端する definition chain 5 本、
`reserve z for TooDeepFourEdgeObjectModeInequality;`、
`FourEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary: z <> z;`
だけの active row を追加する。Chapter 3、4、7、13、14.5.2、16 を既存
Chapter 4/13/14/16 requirement row、structural-chain row、dedicated checker row
を介して参照する。intended credit は real expansion 5 本、raw outermost-
mode result/expected input 4 個、ordinal 1/2 の `BindingId(0)`、terminal-
object-RHS identity 1 個、2 inferred variable、1 fact/deferred-free pre-
desugaring checked inequality、ordered operand-owned expected constraint 2 個
を object/set coercion なしで保持する slice だけである。trace backlink 6 件、
production routing、corruption/near-miss coverage、real sidecar が active
count 120 を保護する。declaration acceptance/inhabitation、inequality desugaring/
truth/fact、closure/order、theorem/proof/Core/VC、set-terminal behavior、他
depth、broader shape は credit しない。
task 172 は `BaseMode -> set` で終端する definition chain 7 本、
`reserve z for ChainMode6;`、
`LongLocalModeReservedVariableEqualityPayloadBoundary: z = z;` だけの active
row を追加する。Chapter 4、7、13、14.5.2、16 を既存 Chapter 4/13/14/16
requirement row、structural-chain row、dedicated checker row を介して参照する。
intended credit は real AST-derived expansion 7 本、raw `ChainMode6` result/
expected input 4 個、ordinal 1/2 の `BindingId(0)`、terminal `BaseMode` RHS の
builtin-set identity 1 個、2 inferred variable、1 fact/deferred-free checked
equality、ordered operand-owned expected constraint 2 個だけである。exact
routing、corruption/near-miss coverage、real frontend/resolver sidecar が active
count 121 を保護する。declaration acceptance/inhabitation、truth/fact、closure/
order、theorem/proof/Core/ControlFlow/VC、imported/attributed/argument-bearing または別 chain
shape、general unbounded semantics は credit しない。
task 173 は exact sibling active row
`LongLocalModeReservedVariableInequalityPayloadBoundary: z <> z;` を追加し、
同じ Chapter 4/7/13/14.5.2/16、structural-chain link、dedicated checker row を
参照する。intended credit は real expansion 7 本、raw `ChainMode6` role 4 個、
ordinal 1/2 の `BindingId(0)`、terminal `BaseMode` RHS identity 1 個、2 inferred
variable、ordered constraint 2 個、1 fact/deferred-free pre-desugaring checked
inequality だけである。backlink 6 件、full guard、real sidecar が active count
122 を保護し、desugaring/truth/fact と broader semantics は credit しない。
task 174 は exact test-first sibling row
`LongLocalModeReservedVariableMembershipPayloadBoundary: x in y;` を追加し、
Chapter 4/7/13/14.5.3/16、structural-chain link、dedicated checker row を参照
する。intended credit は real expansion 7 本、raw `ChainMode6` left result、
独立した explicit-set right result と sole right expected input、ordinal 2/3 の
`BindingId(0/1)`、terminal `BaseMode` RHS identity 1 個、left expected input
なし、2 inferred variable、right-owned constraint 1 個、1 fact/deferred-free
checked membership だけである。backlink 6 件、production routing、full guard、
real sidecar が active count 123 を保護する。membership truth/fact と broader
semantics は credit しない。
task 175 は exact test-first sibling row
`LongLocalModeReservedVariableTypeAssertionPayloadBoundary: x is set;` を追加し、
Chapter 3/4/7/13/14.2.3/16、structural-chain link、dedicated checker row を参照
する。intended credit は real expansion 7 本、raw `ChainMode6` subject result、
独立した formula-side builtin-set asserted input、ordinal 1 の `BindingId(0)`、
terminal `BaseMode` RHS identity 1 個、1 inferred variable、general reachability
を用いない 1 fact/deferred-free normalized-reflexive checked type assertion だけ
である。backlink 7 件は存在し、production routing、full guard、real sidecar、
active count 124 を保護する。widening/`qua`、truth/fact、broader
semantics は credit しない。
task 176 は exact test-first sibling row
`LongLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;` を追加し、
Chapter 4/13/14.5.2/16、structural-chain link、dedicated checker row を参照する。
intended credit は real expansion 7 本、raw `ChainObjectMode6` result/expected
input 4 個、ordinal 1/2 の `BindingId(0)`、terminal `BaseObjectMode` RHS
identity 1 個、2 inferred term、ordered operand-owned constraint 2 個、object/set
coercion を用いない 1 fact/deferred-free checked equality だけである。backlink
6 件、production routing、full guard、real sidecar が active count 125 を保護する。
truth/fact と broader semantics は credit しない。
task 177 は matching exact test-first sibling row
`LongLocalObjectModeReservedVariableInequalityPayloadBoundary: z <> z;` を追加し、
Chapter 4/13/14.5.2/16、structural-chain link、dedicated checker row を参照する。
intended credit は real expansion 7 本、raw `ChainObjectMode6` result/expected
input 4 個、ordinal 1/2 の `BindingId(0)`、terminal `BaseObjectMode` RHS
identity 1 個、2 inferred term、ordered operand-owned constraint 2 個、object/set
coercion を用いない 1 fact/deferred-free pre-desugaring checked inequality
だけである。backlink 6 件、production routing、full guard、real sidecar が
active count 126 を保護する。desugaring、truth/fact、broader semantics は
credit しない。
task 178 は matching exact active sibling row
`LongLocalObjectModeReservedVariableMembershipPayloadBoundary: x in y;` を追加し、
Chapter 4/13/14.5.3/16、structural-chain link、dedicated checker row を参照する。
intended credit は real expansion 7 本、raw `ChainObjectMode6` left result、独立
した explicit-set right result/sole expected input、ordinal 2/3 の
`BindingId(0/1)`、distinct terminal-object-RHS と explicit-set identity、left
expected input なし、2 inferred term、right-owned constraint 1 個、object/set
coercion を用いない 1 fact/deferred-free checked membership だけである。
backlink 6 件、production routing、full guard、real sidecar が active count 127 を
保護する。
truth/fact と broader semantics は credit しない。
task 179 は matching exact active sibling row
`LongLocalObjectModeReservedVariableTypeAssertionPayloadBoundary: x is object;`
を追加し、Chapter 3/4/13/14.2.3/16、structural-chain link、dedicated checker row
を参照する。intended credit は real expansion 7 本、raw `ChainObjectMode6`
subject result、独立した formula-side builtin-object asserted input、ordinal 1
の `BindingId(0)`、terminal-object-RHS identity 1 個、1 inferred term、general
reachability と object/set coercion を用いない 1 fact/deferred-free normalized-
reflexive checked type assertion だけである。shared backlink 6 件、dedicated
row、production routing、full guard、real sidecar が active count 128 を保護する。truth/
fact、acceptance、broader semantics は credit しない。
task 180 は exact standalone pass row
`SourceDerivedContradictionConstantBoundary: contradiction` を Chapter 14/16
と dedicated checker bridge reference とともに追加する。intended credit は
real formula-leaf site/range、module-root context、1 checked
`FormulaKind::Contradiction`、空の term/type/constraint/candidate/fact/deferred/
diagnostic payload である。dedicated row、production route、exact/near-miss/
corruption guard、real sidecar が active count 129 を保護する。falsehood/fact
publication、theorem acceptance、proof-goal closure、implicit closure/child
graph、`formula_statement`、proof、CoreIr、ControlFlowIr、VC は credit しない。
task 182 は exact active formula-side local-mode asserted-head row
`LocalModeAssertedHeadPayloadBoundary: x is LocalModeAssertedHead;` を Chapter
3/4/7/13/14.2.3/16 と dedicated checker bridge reference とともに追加する。
current credit は direct set-terminal real expansion 1 本、同じ local-mode symbol
へ resolve する独立した raw reserve-subject/formula-side asserted input、ordinal 1
の `BindingId(0)`、terminal-definition-RHS builtin-set identity 1 個へ intern する
known type entry 3 個、1 inferred term、general reachability を用いない 1 fact/
deferred-free normalized-reflexive checked type assertion だけに限定する。shared
backlink 5 件と dedicated row、production routing、exact/near-miss/corruption
guard、real frontend/resolver sidecar が active count 130 を保護する。declaration
acceptance/inhabitation、widening/`qua`、truth/fact、theorem/proof/CoreIr/
ControlFlowIr/VC、他 asserted-head family、general semantics は credit しない。
task 183 は exact active object-terminal formula-side local-mode asserted-head
row `LocalObjectModeAssertedHeadPayloadBoundary: x is LocalObjectModeAssertedHead;`
を Chapter 3/4/7/13/14.2.3/16 と dedicated checker bridge reference とともに
追加する。current credit は direct object-terminal real expansion 1 本、同じ
resolved mode symbol 向けの独立した raw reserve-subject/formula-side asserted
input、ordinal 1 の `BindingId(0)`、terminal-definition-RHS builtin-object
identity 1 個へ intern する known type entry 3 個、1 inferred term、general
reachability と object/set coercion を用いない 1 fact/deferred-free normalized-
reflexive checked type assertion だけに限定する。shared backlink 5 件と dedicated
row、production routing、exact/near-miss/corruption guard、real frontend/resolver
sidecar が active count 131 を保護する。declaration acceptance/inhabitation、
truth/fact、theorem/proof/CoreIr/ControlFlowIr/VC、他 asserted-head family、
general semantics は credit しない。
task 184 は exact active one-edge set-terminal same-outer-mode formula-side
asserted-head row `ChainedLocalModeAssertedHeadPayloadBoundary: x is
ChainModeAssertedHead;` を Chapter 3/4/7/13/14.2.3/16 と dedicated checker bridge
reference とともに追加する。current credit は real expansion 2 本、同じ resolved
outer symbol 向けの独立した raw reserve-subject/formula-side asserted input、
ordinal 1 の `BindingId(0)`、terminal base-definition-RHS builtin-set identity
1 個へ intern する known type entry 3 個、1 inferred term、general reachability を
用いない 1 fact/deferred-free normalized-reflexive checked type assertion だけに
限定する。shared backlink 5 件と dedicated row、production routing、exact/near-
miss/corruption guard、real frontend/resolver sidecar が active count 132 を保護する。
declaration acceptance/inhabitation、widening/`qua`、truth/fact、closure/order、
theorem/proof/CoreIr/ControlFlowIr/VC、object/deeper/他 asserted-head chain、
general chain semantics は credit しない。
task 185 は exact active one-edge object-terminal same-outer-mode formula-side
asserted-head row `ChainedLocalObjectModeAssertedHeadPayloadBoundary: x is
ChainObjectModeAssertedHead;` を Chapter 3/4/7/13/14.2.3/16 と dedicated checker
bridge reference とともに追加する。current credit は real expansion 2 本、同じ
resolved outer symbol 向けの独立した raw reserve-subject/formula-side asserted
input、ordinal 1 の `BindingId(0)`、terminal base-definition-RHS builtin-object
identity 1 個へ intern する known type entry 3 個、1 inferred term、general
reachability、widening/`qua`、object/set coercion を用いない 1 fact/deferred-free
normalized-reflexive checked type assertion だけに限定する。shared backlink 5 件
と dedicated row、production routing、exact/near-miss/corruption guard、real
frontend/resolver sidecar が active count 133 を保護する。declaration/attribute
acceptance、truth/fact、closure/order、theorem/proof/CoreIr/ControlFlowIr/VC、
set-terminal/deeper/他 asserted-head chain、general chain semantics は credit
しない。
task 186 は exact active two-edge set-terminal same-outer-mode formula-side
asserted-head row `TwoEdgeLocalModeAssertedHeadPayloadBoundary: x is
OuterTwoEdgeModeAssertedHead;` を Chapter 3/4/7/13/14.2.3/16 と dedicated checker
bridge reference とともに追加する。current credit は real expansion 3 本、同じ
resolved outer symbol 向けの独立した raw reserve-subject/formula-side asserted
input、ordinal 1 の `BindingId(0)`、terminal base-definition-RHS builtin-set
identity 1 個へ intern する known type entry 3 個、1 inferred term、reachability、
widening、`qua` を用いない 1 fact/deferred-free normalized-reflexive checked type
assertion だけに限定する。shared backlink 5 件と dedicated row、production
routing、exact/near-miss/corruption guard、real frontend/resolver sidecar が active
count 134 を保護する。declaration/attribute acceptance、truth/fact、closure/order、
theorem/proof/CoreIr/ControlFlowIr/VC、object-terminal/deeper/imported/他 asserted-
head chain、general chain semantics は credit しない。
task 187 は exact active two-edge object-terminal same-outer-mode formula-side
asserted-head row `TwoEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
OuterTwoEdgeObjectModeAssertedHead;` を Chapter 3/4/7/13/14.2.3/16 と dedicated
checker bridge reference とともに追加する。current credit は real expansion 3
本、同じ resolved local outer symbol 向けの独立した raw reserve-subject/
formula-side asserted site/range、ordinal 1 の `BindingId(0)`、terminal base-
definition-RHS builtin-object identity 1 個へ intern する known type entry 3
個、1 inferred term、reachability、widening、`qua`、object/set coercion を用いない
1 fact/deferred-free normalized-reflexive checked type assertion だけに限定する。
shared backlink 5 件と dedicated row、production routing、exact/near-miss/
corruption guard、real frontend/resolver sidecar が active count 135 を保護する。
positive imported semantics、declaration/attribute acceptance、truth/fact、
closure/order、theorem/proof/CoreIr/ControlFlowIr/VC、set-terminal/deeper/他
asserted-head chain、general chain semantics、downstream payload は credit
しない。
task 188 は exact active builtin-object reserved-variable equality row
`ReservedObjectVariableEqualityPayloadBoundary: x = x;` を shared な Chapter
4/13/14.5.2/16/checker bridge reference 5 件と dedicated checker bridge
reference 1 件とともに追加する。current credit は source-order ordinal 1 と 2
から解決する `BindingId(0)`、記述された 1 個の `object` type range を保持する
distinct な result/expected role site 4 個、その reserve を anchor とする canonical
normalized builtin-object identity 1 個、inferred variable term 2 個、ordered
expected constraint 2 個、object/set coercion を用いない fact/deferred-free な
`Checked` equality 1 個だけに限定する。exact-route、provenance near-miss、
mutable payload-corruption、shared immutable-output validation、real frontend/
resolver sidecar guard が active count 136 を保護する。general/non-reflexive
object equality、truth/fact、closure/order、declaration/theorem acceptance、
`formula_statement`、proof、CoreIr、ControlFlowIr、VC、downstream payload は
credit しない。
task 189 は exact active builtin-object reserved-variable type-assertion row
`ReservedObjectVariableTypeAssertionPayloadBoundary: x is object;` を shared な
Chapter 3/4/13/14.2.3/16 reference 5 件と dedicated checker bridge reference 1 件
とともに追加する。current credit は source-order ordinal 1 から解決する
`BindingId(0)`、distinct reserve-result/formula-side asserted object site/range、
argument/attribute-free な raw `BuiltinObject` input 2 個、written reserve type を
anchor とする canonical normalized identity 1 個、inferred variable 1 個、known
type entry 3 個、expected constraint 0 個、fact/deferred-free な `Checked` assertion
1 個だけに限定する。exact-route、provenance near-miss、mutable payload-
corruption、shared immutable-output validation、real frontend/resolver sidecar
guard が active count 137 を保護する。reachability/widening/`qua`、object/set
coercion、truth/fact、closure/order、declaration/theorem acceptance、
`formula_statement`、proof、CoreIr、ControlFlowIr、VC、downstream payload は
credit しない。
task 190 は exact active builtin-object reserved-variable inequality row
`ReservedObjectVariableInequalityPayloadBoundary: x <> x;` を shared な Chapter
4/13/14.5.2/16/checker bridge reference 5 件と dedicated checker bridge
reference 1 件とともに追加する。current credit は source-order ordinal 1 と 2
から解決する `BindingId(0)`、記述された 1 個の `object` type range を保持する
distinct result/expected role site 4 個、argument/attribute-free な raw
`BuiltinObject` input 4 個、その reserve を anchor とする canonical normalized
builtin-object identity 1 個、inferred variable term 2 個、known type entry 6 個、
ordered expected constraint 2 個、fact/candidate/diagnostic/deferred-free な pre-
desugaring `Checked` inequality 1 個だけに限定する。exact-route、structural/
provenance near-miss、mutable payload-corruption、positive immutable-output
validation、real frontend/resolver sidecar guard が active count 138 を保護する。
inequality desugaring/equality truth、object/set coercion、fact、closure/order、
declaration/theorem acceptance、`formula_statement`、proof、CoreIr、
ControlFlowIr、VC、downstream payload は credit しない。
task 191 は exact active distinct-binding shared-builtin-object equality row
`DistinctReservedObjectVariableEqualityPayloadBoundary: x = y;` を shared な
Chapter 4/13/14.5.2/16/builtin-type bridge reference 5 件と dedicated checker
bridge reference 1 件とともに追加する。credit は source-order ordinal 2 と 3
から解決する `BindingId(0/1)`、両 binding と distinct result/expected role
site 4 個にまたがる shared written `object` range 1 個、argument/attribute-free
な raw `BuiltinObject` input 4 個、reserve range を anchor とする canonical
normalized builtin-object identity 1 個、inferred variable term 2 個、known type
entry 6 個、ordered expected constraint 2 個、fact/candidate/diagnostic/deferred-
free な `Checked` equality 1 個だけに限定する。exact-route、structural/
provenance near-miss、mutable payload-corruption、positive immutable-output
validation、real frontend/resolver sidecar は active count 139 を保護する。
equality truth、object/set coercion、fact、closure/
order、declaration/theorem acceptance、`formula_statement`、proof、CoreIr、
ControlFlowIr、VC、downstream payload は credit しない。
task 192 は exact active distinct-binding shared-builtin-object inequality row
`DistinctReservedObjectVariableInequalityPayloadBoundary: x <> y;` を shared な
Chapter 4/13/14.5.2/16/builtin-type bridge reference 5 件と dedicated checker
bridge reference 1 件とともに追加する。credit は source-order ordinal 2 と 3
から解決する `BindingId(0/1)`、両 binding と distinct result/expected role
site 4 個にまたがる shared written `object` range 1 個、argument/attribute-free
な raw `BuiltinObject` input 4 個、reserve range を anchor とする canonical
normalized builtin-object identity 1 個、inferred variable term 2 個、known type
entry 6 個、ordered expected constraint 2 個、fact/candidate/diagnostic/deferred-
free な `Checked` inequality 1 個だけに限定する。exact-route、structural/
provenance near-miss、mutable payload-corruption、positive immutable-output
validation、real frontend/resolver sidecar は active count 140 を保護する。
inequality desugaring/equality truth、object/set
coercion、fact、closure/order、declaration/theorem acceptance、
`formula_statement`、proof、CoreIr、ControlFlowIr、VC、downstream payload は
credit しない。
task 193 は exact active multiple-reserve-declaration builtin-object equality
row `reserve x for object; reserve y for object; theorem
MultipleObjectReserveDeclarationEqualityPayloadBoundary: x = y;` を shared な
Chapter 4/13/14.5.2/16/builtin-type bridge reference 5 件と dedicated checker
bridge reference 1 件とともに追加する。credit は source-order ordinal 2 と 3
から解決する `BindingId(0/1)`、distinct result/expected role site 4 個に
またがる binding ごとの distinct written `object` range 2 個、argument/
attribute-free な raw `BuiltinObject` input 4 個、先行する `x` range を anchor
とする canonical normalized builtin-object identity 1 個、inferred variable
term 2 個、known type entry 6 個、ordered expected constraint 2 個、fact/
candidate/diagnostic/deferred-free な `Checked` equality 1 個だけに限定する。
exact-route、structural/provenance near-miss、mutable payload-corruption、
positive immutable-output validation、real frontend/resolver sidecar は active
count 141 を保護する。equality truth、object/set coercion、fact、closure/order、
declaration/theorem acceptance、`formula_statement`、proof、CoreIr、
ControlFlowIr、VC、shared-range shape、downstream payload は credit しない。
task 194 は exact active multiple-reserve-declaration builtin-object inequality
row `reserve x for object; reserve y for object; theorem
MultipleObjectReserveDeclarationInequalityPayloadBoundary: x <> y;` を shared な
Chapter 4/13/14.5.2/16/builtin-type bridge reference 5 件と dedicated checker
bridge reference 1 件とともに追加する。credit は source-order ordinal 2 と 3
から解決する `BindingId(0/1)`、distinct raw result/expected role 4 個に
またがる binding ごとの ordered distinct written `object` range 2 個、
argument/attribute-free な `BuiltinObject` input 4 個、先行する `x` range を
anchor とする canonical normalized builtin-object identity 1 個、inferred
variable term 2 個、known type entry 6 個、ordered expected constraint 2 個、
fact/candidate/diagnostic/deferred-free な pre-desugaring `Checked` inequality 1
個だけに限定する。exact-route、structural/provenance near-miss、mutable
payload-corruption、immutable-output validation、real frontend/resolver sidecar
は active count 142 を保護する。inequality desugaring/equality truth、object/set
coercion、fact、closure/order、declaration/theorem acceptance、
`formula_statement`、proof、CoreIr、ControlFlowIr、VC、shared-range shape、
downstream payload は credit しない。

task 195 は ordered local definition 4 個 `Outer -> Middle -> Inner -> Base ->
set`、outer-mode reserve 1 個、`ThreeEdgeLocalModeAssertedHeadPayloadBoundary: x
is OuterThreeEdgeModeAssertedHead;` を持つ exact active three-edge set-terminal
same-outer-mode asserted-head row を、shared Chapter 4/7/13/14.2.3/16 reference
5 件と dedicated checker bridge reference 1 件とともに追加する。credit は real
AST-derived expansion 4 個、ordinal 1 から解決する `BindingId(0)`、同じ outer
symbol の distinct raw reserve-subject/formula asserted-type site/range、base-
definition-RHS anchor の `BuiltinSet` identity 1 個へ normalize する known type
entry 3 個、inferred variable 1 個、expected constraint/candidate/fact/
diagnostic/deferred reason 0 個、normalized-reflexive `Checked` type assertion 1
個だけに限定する。exact-route、unrelated local/imported/ambiguous asserted head
を含む structural/provenance near-miss、mutable corruption、immutable-output
validation、real frontend/resolver sidecar は active count 143 を保護する。
reachability/widening/`qua`、declaration/theorem acceptance、truth/fact、closure/
order、`formula_statement`、proof、CoreIr、ControlFlowIr、VC、broader formula/
child-graph semantics、general chain は credit しない。

task 196 は ordered local definition 4 個 `Outer -> Middle -> Inner -> Base ->
object`、outer-mode reserve 1 個、
`ThreeEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
OuterThreeEdgeObjectModeAssertedHead;` を持つ exact active three-edge object-
terminal same-outer-mode asserted-head row を、shared Chapter 4/7/13/14.2.3/16
reference 5 件と dedicated checker bridge reference 1 件とともに追加する。
credit は real AST-derived expansion 4 個、ordinal 1 から解決する
`BindingId(0)`、同じ outer symbol の distinct raw reserve-subject/formula
asserted-type site/range、base-definition-RHS anchor の `BuiltinObject` identity
1 個へ normalize する known type entry 3 個、inferred variable 1 個、expected
constraint/candidate/fact/diagnostic/deferred reason 0 個、normalized-reflexive
`Checked` type assertion 1 個だけに限定し、object/set coercion はない。exact-
route、unrelated local/imported/ambiguous asserted head を含む structural/
provenance near-miss、`BuiltinSet`/canonical mutable corruption、immutable-
output validation、real frontend/resolver sidecar は active count 144 を保護
する。reachability/widening/`qua`、declaration/theorem acceptance、truth/fact、
closure/order、`formula_statement`、proof、CoreIr、ControlFlowIr、VC、broader
formula/child-graph semantics、general chain は credit しない。

task 197 は ordered local definition 5 個 `TooDeep -> Outer -> Middle -> Inner
-> Base -> set`、outermost-mode reserve 1 個、
`FourEdgeLocalModeAssertedHeadPayloadBoundary: x is
TooDeepFourEdgeModeAssertedHead;` を持つ exact active four-edge set-terminal
same-outermost-mode asserted-head row を、shared Chapter 4/13/14.2.3/16
reference 4 件、shared Task 74 structural-chain reference 1 件、dedicated
checker bridge reference 1 件とともに追加する。credit は real AST-derived
expansion 5 個、ordinal 1 から解決する `BindingId(0)`、同じ outermost symbol
の distinct raw reserve-subject/formula asserted-type site/range、base-
definition-RHS anchor の `BuiltinSet` identity 1 個へ normalize する known
type entry 3 個、inferred variable 1 個、expected constraint/candidate/fact/
diagnostic/deferred reason 0 個、normalized-reflexive `Checked` type assertion
1 個だけに限定する。exact-route、full-reorder、connected-deeper、unrelated
local/imported/ambiguous asserted head を含む structural/provenance near-miss、
`BuiltinObject`/canonical mutable corruption、immutable-output validation、
real frontend/resolver sidecar は active count 145 を保護する。reachability/
widening/`qua`、declaration/theorem acceptance、truth/fact、closure/order、
`formula_statement`、proof、CoreIr、ControlFlowIr、VC、broader formula/child-
graph semantics、general chain は credit しない。

task 198 は ordered local definition 5 個 `TooDeep -> Outer -> Middle -> Inner
-> Base -> object`、outermost-mode reserve 1 個、
`FourEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
TooDeepFourEdgeObjectModeAssertedHead;` を持つ exact active four-edge object-
terminal same-outermost-mode asserted-head row を、shared Chapter 4/13/14.2.3/
16 reference 4 件、shared Task 74 structural-chain reference 1 件、dedicated
checker bridge reference 1 件とともに追加する。credit は real AST-derived
expansion 5 個、ordinal 1 から解決する `BindingId(0)`、同じ outermost symbol
の distinct raw reserve-subject/formula asserted-type site/range、base-
definition-RHS anchor の `BuiltinObject` identity 1 個へ normalize する known
type entry 3 個、inferred variable 1 個、expected constraint/candidate/fact/
diagnostic/deferred reason 0 個、normalized-reflexive `Checked` type assertion
1 個だけに限定し、object/set coercion はない。exact-route、full-reorder、
connected-deeper、unrelated local/imported/ambiguous asserted head を含む
structural/provenance near-miss、`BuiltinSet`/canonical mutable corruption、
immutable-output validation、real frontend/resolver sidecar は active count
146 を保護する。reachability/widening/`qua`、declaration/theorem acceptance、
truth/fact、closure/order、`formula_statement`、proof、CoreIr、ControlFlowIr、
VC、broader formula/child-graph semantics、general chain は credit しない。

task 199 は `BaseMode -> set`、`ChainMode6 -> ChainMode5` までの ordered local
link 6 個、`ChainMode6` reserve 1 個、
`LongLocalModeAssertedHeadPayloadBoundary: x is ChainMode6;` を持つ exact
active seven-expansion set-terminal same-`ChainMode6` asserted-head row を、
shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 74 structural-chain
reference 1 件、dedicated checker bridge reference 1 件とともに追加する。credit
は real AST-derived expansion 7 個、ordinal 1 から解決する `BindingId(0)`、同じ
symbol の distinct raw reserve-subject/formula asserted-type site/range、
`BaseModeDef` RHS anchor の `BuiltinSet` identity 1 個へ normalize する known
type entry 3 個、inferred variable 1 個、expected constraint/candidate/fact/
diagnostic/deferred reason 0 個、normalized-reflexive `Checked` type assertion 1
個だけに限定する。exact-route、per-link removal/reorder、complete-reverse、
connected-eighth、unrelated local/imported/ambiguous asserted head を含む
structural/provenance near-miss、`BuiltinObject`/canonical mutable corruption、
immutable-output validation、real frontend/resolver sidecar は active count 147
を保護する。object-terminal/other-depth/imported/attributed/argument-bearing/
other asserted head、reachability/widening/`qua`、declaration/theorem acceptance、
truth/fact、closure/order、`formula_statement`、proof、CoreIr、ControlFlowIr、
VC、broader formula/child-graph semantics、general unbounded chain は credit
しない。

task 200 は `BaseObjectMode -> object`、`ChainObjectMode6 ->
ChainObjectMode5` までの ordered local link 6 個、`ChainObjectMode6` reserve
1 個、`LongLocalObjectModeAssertedHeadPayloadBoundary: x is ChainObjectMode6;`
を持つ exact active seven-expansion object-terminal same-`ChainObjectMode6`
asserted-head row を、shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task
74 structural-chain reference 1 件、dedicated checker bridge reference 1 件と
ともに追加する。credit は real AST-derived expansion 7 個、ordinal 1 から
解決する `BindingId(0)`、同じ symbol の distinct raw reserve-subject/formula
asserted-type site/range、`BaseObjectModeDef` RHS anchor の `BuiltinObject`
identity 1 個へ normalize する known type entry 3 個、inferred variable 1 個、
expected constraint/candidate/fact/diagnostic/deferred reason 0 個、object/set
coercion のない normalized-reflexive `Checked` type assertion 1 個だけに限定
する。exact-route、per-link removal/reorder、complete-reverse、connected-
eighth、unrelated local/imported/ambiguous asserted head を含む structural/
provenance near-miss、`BuiltinSet`/canonical mutable corruption、immutable-
output validation、real frontend/resolver sidecar は active count 148 を保護
する。set-terminal/other-depth/imported/attributed/argument-bearing/other
asserted head、reachability/widening/`qua`、declaration/theorem acceptance、
truth/fact、closure/order、`formula_statement`、proof、CoreIr、ControlFlowIr、
VC、broader formula/child-graph semantics、general unbounded chain は credit
しない。

task 120 は
matching exact pass row
`reserve x for set; theorem ReservedVariableMembershipPayloadBoundary: x in x;`
を追加する。両 identifier result と右 membership expected type は記述された
`set` reserve から導かれ、no-fact `Checked` membership は type/well-formedness
だけを記録する。membership truth/fact、implicit closure、theorem acceptance、
proof、CoreIr、ControlFlowIr、VC は credit しない。
task 121 は exact pass row
`reserve x for set; theorem ReservedVariableInequalityPayloadBoundary: x <> x;`
を追加する。checker-owned inequality API が 2 つの expected-type slot を提供し、
task 119 が real reserve binding/use producer を提供するため、runner は 2 組の
linked result/expected role pair と 1 つの fact-free pre-desugaring `Checked`
inequality を記録する。task 107 の numeral inequality bridge は expected type
なしの partial のままである。inequality desugaring/truth/fact、implicit closure、
theorem acceptance、proof、CoreIr、ControlFlowIr、VC は credit しない。
task 122 は exact pass row
`reserve x for set; theorem ReservedVariableTypeAssertionPayloadBoundary: x is set;`
を追加する。task 119 が real reserve lookup/result input、task 109 が independently
source-anchored な formula asserted-type input を供給する。checker は normalized
reflexive identity だけを受理し、1 fact-free `Checked` type assertion を記録する。
known non-identical type は external reachability payload gap 上の partial に残る。
general reachability/widening/`qua`、attribute、truth/fact、implicit closure、
theorem acceptance、proof、CoreIr、ControlFlowIr、VC は credit しない。
task 109 は task 102 の exact builtin type-assertion sidecar を supersede し、
real checker term/formula payload extraction と asserted builtin `set`
`TypeExpressionInput` を credit するが、numeric type payload、より広い
asserted type payload、type-assertion semantic checking、recorded fact、
theorem acceptance、`formula_statement` runner support、CoreIr、ControlFlowIr、
VC、proof payload は credit しない。
task 113 は exact imported attribute assertion theorem formula について task 103 を
supersede し、imported `empty` provenance validation と real checker term/formula
payload handoff を credit するが、numeric type payload、attribute-chain semantic
payload extraction、theorem formula 向け checker `AttributeInput` payload extraction、
term inference、attribute admissibility/semantic checking、formula checking、
recorded fact、theorem acceptance、imported module AST extraction、
`formula_statement` runner support、CoreIr、ControlFlowIr、VC、proof payload は
credit しない。
task 114 の fail case は exact attribute-level non-empty imported attribute
assertion theorem formula について task 104 だけを supersede する。parser /
resolver 実行後に direct `non` surface と imported `empty` provenance を検証し、
real checker term/formula payload を渡してから、numeric type payload、negated
attribute-chain semantic payload extraction、theorem formula 向け checker
`AttributeInput` payload extraction、term inference、negated attribute
admissibility/semantic checking、formula checking、recorded fact、theorem
acceptance、imported module AST extraction、`formula_statement` runner support、
CoreIr、ControlFlowIr、VC、proof payload の手前で fail closed する。
task 111 の fail case は parser / resolver 実行後の exact
`SetEnumerationPayloadBoundary: {1, 2} = {1, 2}` checker handoff だけを credit
する。つまり、4 つの numeral item term、2 つの set-enumeration term、1 つの
equality formula の real checker payload だけである。broader set-enumeration
term extraction、result-type payload、term inference、equality/formula
checking、recorded fact、theorem acceptance、`formula_statement` runner support、
CoreIr、ControlFlowIr、VC、proof payload は credit しない。
task 112 / task 117 の fail case は parser / resolver 実行後の exact
connective/quantifier theorem formula checker shell handoff についてだけ task 99 を
supersede する。implication、universal quantification、negation の real checker
`FormulaInput` shell と、2 つの source constant に対応する exact
`FormulaKind::Contradiction` payload だけを credit し、formula constant semantic
truth value、child-formula graph payload、quantifier binder/context payload、
formula checking、recorded fact、theorem acceptance、`formula_statement` runner
support、CoreIr、ControlFlowIr、VC、proof payload は credit しない。
task 88 の fail case は parser / resolver 実行後の proof-block / proof-skeleton
extraction-gap boundary だけを credit し、checker proof skeleton payload extraction、
local proof context、formula payload extraction、recorded fact、theorem acceptance、
`formula_statement` runner support、CoreIr、ControlFlowIr、VC、proof payload は
credit しない。
task 89 の fail case は parser / resolver 実行後の statement-level proof-justification
extraction-gap boundary だけを credit し、checker statement proof payload extraction、
nested proof skeleton payload、local proof context、formula payload extraction、
label-reference semantic checking、recorded fact、theorem acceptance、
`formula_statement` runner support、CoreIr、ControlFlowIr、VC、proof payload は
credit しない。
task 90 の fail case は parser / resolver 実行後の predicate/functor definition
extraction-gap boundary だけを credit し、checker definition declaration payload
extraction、definition-local context、definiens formula/term payload extraction、
overload payload、recorded fact、`formula_statement` runner support、CoreIr、
ControlFlowIr、VC、proof payload は credit しない。
task 91 の fail case は parser / resolver 実行後の attribute definition
extraction-gap boundary だけを credit し、checker attribute definition declaration
payload extraction、definition-local context、formula-definiens payload extraction、
attributed-type evidence、recorded fact、`formula_statement` runner support、
CoreIr、ControlFlowIr、VC、proof payload は credit しない。
task 92 の fail case は parser / resolver 実行後の mode/structure definition
extraction-gap boundary だけを credit し、checker mode/structure definition
declaration payload extraction、mode expansion、structure base-shape /
constructor / selector evidence、definition-local context、recorded fact、
`formula_statement` runner support、CoreIr、ControlFlowIr、VC、proof payload は
credit しない。
task 93 の fail case は parser / resolver 実行後の proof-local declaration
statement extraction-gap boundary だけを credit し、checker proof-local
declaration payload extraction、local proof context、formula/term payload
extraction、RHS term inference、reconsider coercion / obligation evidence、
recorded fact、theorem acceptance、`formula_statement` runner support、CoreIr、
ControlFlowIr、VC、proof payload は credit しない。
task 94 の fail case は parser / resolver 実行後の proof-local inline definition
extraction-gap boundary だけを credit し、checker inline definition formal/body
payload extraction、local abbreviation expansion、term/formula body payload
extraction、guard evidence、recorded fact、theorem acceptance、`formula_statement` runner support、
CoreIr、ControlFlowIr、VC、proof payload は credit しない。
task 95 の fail case は parser / resolver 実行後の registration-block
extraction-gap boundary だけを credit し、checker registration-item payload
extraction、correctness-condition / proof-obligation payload、accepted
activation / evidence status、cluster / reduction semantics、Chapter 17 semantic
row、fact、`formula_statement` または `advanced_semantics` runner support、
CoreIr、ControlFlowIr、VC、proof payload は credit しない。
task 96 の fail case は parser / resolver 実行後の redefinition / notation
extraction-gap boundary だけを credit し、checker redefinition payload
extraction、notation alias relation payload、redefinition target inference、
coherence proof-obligation payload、overload candidate payload、Chapter 11
alias semantic resolution、Chapter 19 overload / redefinition semantics、fact、
`formula_statement` または `advanced_semantics` runner support、CoreIr、
ControlFlowIr、VC、proof payload は credit しない。
これらの gap test はより広い task 7-11 semantic pass/fail coverage を満たさず、
prepared consumer execution が存在するまで `CoreIr`、`ControlFlowIr`、
`proof_verification` row は deferred のままにする。summary/context readiness read は
CoreIr / ControlFlowIr / VC / proof の昇格ではない。

## Reporting

Default report は次で group する。

- spec file
- stage
- status
- missing coverage shape
- tests with unknown spec refs
- tests that cover obsolete requirements

Reports は deterministic で CI output に適していなければならない。

現在の `plan` CLI report は deterministic totals、stage ごとの coverage status
count、missing-shape count、architecture test strategy の 40% pass / 60% fail
target に対する corpus-wide pass/fail mix を出力する。Pass/fail mix は unique
valid sidecar を数えるため、複数 requirement を cover する sidecar も重複計上
しない。

task 14 は report に architecture-22 matrix summary を追加する。この summary は
consumer crate を実行せず、validated sidecar metadata から作られる。
[expectation_schema.md](./expectation_schema.md) の registry にある required scenario id
ごとに、registry equivalence class、planned metadata count、active execution count、
missing かどうかを記録する。task 14 で commit する anchor は次である。

```text
tests/property/architecture22_matrix_001.expect.toml
```

これは `stage = "advanced_semantics"`、`domain = "incremental_verification"` の
`property_seed` metadata-only sidecar であり、manual-review requirement
`spec.en.architecture_22.regression_matrix.metadata` に link する。これにより、将来の
consumer-specific runner または integration test が real execution を所有するまで、
全 matrix row を planned metadata として visible に保ち、inactive のままにできる。

## Constraints and Assumptions

- `doc/spec/` は per-test links を持たない。
- Requirement ids は test corpus の stable public identifiers である。
- Manifest は manually edit してよいが、validation は automated である。
- Generated tests は committed expectation metadata を通じてのみ coverage に寄与できる。
- Coverage は semantic evidence であり、line or branch coverage ではない。


## task 201 traceability

task 201 は shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 56 chain-producer reference 1 件、dedicated checker bridge reference 1 件を持つ exact active immediate-radix asserted-head row 1 件を追加する。credit は real AST-derived expansion 2 個、distinct Outer reserve-subject / Base formula asserted-type symbol/site/range、ordinal 1 から解決する `BindingId(0)`、Base-definition-RHS `BuiltinSet` 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の `Checked` type assertion 1 個だけに限定する。exact route、provenance/corruption、Task 146/184 isolation、immutable-output、real sidecar test が active count 149 を保護する。plan は 364 cases / 328 requirements を持つ。broader asserted-head、declaration、theorem、formula-child、proof、CoreIr、ControlFlowIr、VC semantics は credit しない。


## task 202 traceability

task 202 は shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 56 chain-producer reference 1 件、dedicated checker reference 1 件を持つ exact object-terminal immediate-radix active row 1 件を追加する。credit は real expansion 2 個、distinct Outer/Base symbol/site/range、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinObject` 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。exact/corruption、real Tasks 147/185/201 isolation、immutable-output、sidecar test が active count 150 を保護する。plan は 365 cases / 329 requirements を持ち、broader semantics は credit しない。


## task 203 traceability

task 203 は shared Chapter 4/7/13/14.2.3/16 reference 5 件と dedicated checker reference 1 件を持つ exact two-edge set-terminal immediate-radix active row 1 件を追加する。credit は real expansion 3 個、distinct Outer/Middle symbol/site/range、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinSet` 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。exact/corruption/order/duplicate/spelling/imported/ambiguous/deeper coverage、real Tasks 122/148/149/186/187/201/202 isolation、immutable output、real sidecar が active count 151 を保護する。plan は既存 expectation を rebaseline せず 366 cases / 330 requirements を持ち、two-hop Base assertion と broader semantics は credit しない。


## task 204 traceability

task 204 は shared Chapter 4/7/13/14.2.3/16 reference 5 件と dedicated checker reference 1 件を持つ exact two-edge object-terminal immediate-radix active row 1 件を追加する。credit は real object expansion 3 個、distinct Outer/Middle symbol/site/range、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinObject` 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定し、object/set coercion は行わない。exact/corruption/order/duplicate/spelling/imported/ambiguous/deeper coverage、real Tasks 189/145/147/149/187/202 および set Tasks 148/186/203 isolation、immutable output、real sidecar が active count 152 を保護する。plan は既存 expectation を rebaseline せず 367 cases / 331 requirements を持ち、two-hop Base assertion と broader semantics は credit しない。

## task 205 traceability

task 205 は shared Chapter 4/7/13/14.2.3/16 reference 5 件と dedicated checker reference 1 件を持つ exact three-edge set-terminal immediate-radix active row 1 件を追加する。credit は real set-terminal expansion 4 個、distinct Outer/Middle symbol/site/range、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinSet` 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。exact/corruption/all-23-orders/missing/duplicate/label/spelling/radix/imported/ambiguous/deeper/multi-hop coverage、全 16 declared owner route との bidirectional isolation、immutable output、real sidecar が active count 153 を保護する。plan は既存 expectation を rebaseline せず 368 cases / 332 requirements を持ち、multi-hop Inner/Base assertion、matching object sibling、broader semantics は credit しない。

## task 206 traceability

task 206 は shared Chapter 4/7/13/14.2.3/16 reference 5 件と dedicated checker reference 1 件を持つ exact three-edge object-terminal immediate-radix active row 1 件を追加する。credit は real object-terminal expansion 4 個、distinct Outer/Middle symbol/site/range、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinObject` 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定し、object/set coercion は行わない。exact/corruption/all-23-orders/per-definition/imported/ambiguous/deeper/multi-hop/local-other coverage、全 17 declared owner route との bidirectional isolation、immutable output、real sidecar が active count 154 を保護する。plan は既存 expectation を rebaseline せず 369 cases / 333 requirements を持ち、multi-hop Inner/Base assertion と broader semantics は credit しない。

## task 207 traceability

task 207 は shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 74 producer reference 1 件、dedicated checker reference 1 件を持つ exact four-edge set-terminal immediate-radix active row 1 件を追加する。credit は real set-terminal expansion 5 個、distinct TooDeep/Outer symbol/site/range、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinSet` 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。exact/corruption/all-119-orders/per-definition/imported/ambiguous/deeper/multi-hop/local-other coverage、全 20 declared owner route との bidirectional isolation、immutable output、real sidecar が active count 155 を保護する。plan は既存 expectation を rebaseline せず 370 cases / 334 requirements を持ち、multi-hop Middle/Inner/Base assertion、matching object sibling、broader semantics は credit しない。

## task 208 traceability

task 208 は shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 74 producer reference 1 件、dedicated checker reference 1 件を持つ exact four-edge object-terminal immediate-radix active row 1 件を追加する。credit は real object-terminal expansion 5 個、distinct TooDeep/Outer symbol/site/range、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinObject` 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定し、object/set coercion は行わない。exhaustive source/provenance/corruption coverage、全 21 declared owner route との bidirectional isolation、immutable output、real sidecar が active count 156 を保護する。plan は既存 expectation を rebaseline せず 371 cases / 335 requirements を持ち、multi-hop Middle/Inner/Base assertion と broader semantics は credit しない。

## task 209 traceability

task 209 は shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 74 structural-producer reference 1 件、dedicated checker reference 1 件を持つ exact seven-expansion set-terminal immediate-radix active row 1 件を追加する。credit は real expansion 7 個、distinct ChainMode6/ChainMode5 symbol/site/range、ordinal 1 / `BindingId(0)`、BaseModeDef-RHS `BuiltinSet` 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。全 5,039 nonidentity order、宣言済み finite source/provenance/corruption matrix、Task 209 実装前の owner route 34 件、immutable output、real sidecar が active count 157 を保護する。plan は既存 expectation を rebaseline せず 372 cases / 336 requirements を持ち、multi-hop/broader semantics は credit しない。

## task 210 traceability

task 210 は shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 74 structural-producer reference 1 件、dedicated checker reference 1 件を持つ exact seven-expansion object-terminal immediate-radix active row 1 件を追加する。credit は real object-terminal expansion 7 個、distinct ChainObjectMode6/ChainObjectMode5 symbol/site/range、ordinal 1 / `BindingId(0)`、BaseObjectModeDef-RHS `BuiltinObject` 1 個へ normalize する known entry 3 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。全 5,039 nonidentity order、宣言済み finite source/provenance/corruption matrix、Task 210 実装前の owner route 35 件、immutable output、real sidecar が active count 158 を保護する。plan は既存 expectation を rebaseline せず 373 cases / 337 requirements を持ち、multi-hop/broader semantics は credit しない。

## task 211 traceability

task 211 は shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 72 two-edge structural-producer reference 1 件、dedicated checker reference 1 件を持つ exact two-edge set-terminal two-hop asserted-head active row 1 件を追加する。credit は real expansion 3 個、distinct Outer/Base symbol/site/range、actual bare link 2 本、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinSet` 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。全5 nonidentity definition order、finite structural/provenance/corruption matrix、既存 owner route 36 件、immutable output、real sidecar が active count 159 を保護する。plan は既存 expectation を rebaseline せず 374 cases / 338 requirements を持ち、他 distance、generic reachability、broader semantics は credit しない。

## task 212 traceability

task 212 は shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 72 two-edge structural-producer reference 1 件、dedicated checker reference 1 件を持つ exact two-edge object-terminal two-hop asserted-head active row 1 件を追加する。credit は real object expansion 3 個、distinct Outer/Base symbol/site/range、actual bare link 2 本、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinObject` 1 個へ normalize する known entry 3 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。全5 nonidentity definition order、finite structural/provenance/corruption matrix、既存 owner route 37 件、immutable output、real sidecar が active count 160 を保護する。plan は既存 expectation を rebaseline せず 375 cases / 339 requirements を持ち、他 distance、generic reachability、object/set coercion、broader semantics は credit しない。

## task 213 traceability

task 213 は shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 73 structural-producer reference 1 件、dedicated checker reference 1 件を持つ exact three-edge set-terminal two-hop asserted-head active row 1 件を追加する。credit は real expansion 4 個、distinct Outer/Inner symbol/site/range、明示検証する relation link 2 本、terminal-only の Inner-to-Base-to-set tail、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinSet` 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。全23 nonidentity definition order、finite structural/provenance/corruption matrix、Task 211/212 focused regression、既存 owner route 38 件、immutable output、real sidecar が active count 161 を保護する。plan は既存 expectation を rebaseline せず 376 cases / 340 requirements と type-elaboration coverage 208/196 を持ち、object sibling、他 distance、generic reachability、broader semantics は credit しない。

## task 214 traceability

task 214 は shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 73 structural-producer reference 1 件、dedicated checker reference 1 件を持つ exact three-edge object-terminal two-hop asserted-head active row 1 件を追加する。credit は real object expansion 4 個、distinct Outer/Inner symbol/site/range、明示検証する relation link 2 本、terminal-only の Inner-to-Base-to-object tail、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinObject` 1 個へ normalize する known entry 3 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。全23 nonidentity definition order、finite structural/provenance/corruption matrix、Task 211/212/213 focused regression、既存 owner route 39 件、immutable output、real sidecar が active count 162 を保護する。plan は既存 expectation を rebaseline せず 377 cases / 341 requirements、type-elaboration coverage 209/197、pass/fail 193/184 を持ち、他 distance、generic reachability、object/set coercion、broader semantics は credit しない。

## task 215 traceability

task 215 は shared Chapter 4/13/14.2.3/16 と Task 74 structural-producer reference 5 件、dedicated checker reference 1 件を持つ exact four-edge set-terminal two-hop asserted-head active row 1 件を追加する。credit は real set expansion 5 個、distinct TooDeep/Middle symbol/site/range、明示検証する relation link 2 本、terminal-only の Middle-to-Inner-to-Base-to-set tail、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinSet` 1 個へ normalize する known entry 3 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。全119 nonidentity definition order、finite structural/provenance/corruption matrix、Tasks 211-214 focused regression、既存 owner route 40 件、immutable output、real sidecar が active count 163 を保護する。plan は既存 expectation を rebaseline せず 378 cases / 342 requirements、type-elaboration coverage 210/198、pass/fail 194/184 を持ち、object sibling、他 distance、generic reachability、object/set coercion、broader semantics は credit しない。

## task 216 traceability

task 216 は shared Chapter 4/13/14.2.3/16 と Task 74 structural-producer reference 5 件、dedicated checker reference 1 件を持つ exact four-edge object-terminal two-hop asserted-head active row 1 件を追加する。credit は real object expansion 5 個、distinct TooDeep/Middle symbol/site/range、明示検証する relation link 2 本、terminal-only の Middle-to-Inner-to-Base-to-object tail、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinObject` 1 個へ normalize する known entry 3 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。全119 nonidentity definition order、finite structural/provenance/corruption matrix、Tasks 211-215 focused regression、既存 owner route 41 件、immutable output、real sidecar が active count 164 を保護する。plan は既存 expectation を rebaseline せず 379 cases / 343 requirements、type-elaboration coverage 211/199、pass/fail 195/184 を持ち、他 distance、generic reachability、object/set coercion、broader semantics は credit しない。

## task 217 traceability

task 217 は shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 73 structural-producer reference 1 件、dedicated checker reference 1 件を持つ exact three-edge set-terminal three-hop asserted-head active row 1 件を追加する。credit は real set expansion 4 個、distinct Outer/Base symbol/site/range、明示検証する relation link 3 本、terminal-only Base-to-set normalization、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinSet` 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。全23 nonidentity definition order、finite structural/provenance/corruption matrix、Tasks 211-216 focused regression、既存 owner route 42 件、immutable output、real sidecar が active count 165 を保護する。plan は既存 expectation を rebaseline せず 380 cases / 344 requirements、type-elaboration coverage 212/200、pass/fail 196/184 を持ち、object sibling、他 depth、generic reachability、object/set coercion、broader semantics は credit しない。

## task 218 traceability

task 218 は shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 73 structural-producer reference 1 件、dedicated checker reference 1 件を持つ exact three-edge object-terminal three-hop asserted-head active row 1 件を追加する。credit は real object expansion 4 個、distinct Outer/Base symbol/site/range、明示検証する relation link 3 本、terminal-only Base-to-object normalization、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinObject` 1 個へ normalize する known entry 3 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。全23 nonidentity definition order、finite structural/provenance/corruption matrix、Tasks 211-217 focused regression、既存 owner route 43 件、immutable output、real sidecar が active count 166 を保護する。plan は既存 expectation を rebaseline せず 381 cases / 345 requirements、type-elaboration coverage 213/201、pass/fail 197/184 を持ち、他 depth、generic reachability、object/set coercion、broader semantics は credit しない。

## task 219 traceability

task 219 は shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 74 structural-producer reference 1 件、dedicated checker reference 1 件を持つ exact four-edge set-terminal three-hop asserted-head active row 1 件を追加する。credit は real set expansion 5 個、distinct TooDeep/Inner symbol/site/range、明示検証する relation link 3 本、terminal-only Inner-to-Base-to-set normalization、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinSet` 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。全119 nonidentity definition order、unconnected unsupported deeper asserted head と actual connected sixth-definition/sixth-edge asserted head の独立 guard を含む finite structural/provenance/corruption matrix、Task 207 と Tasks 211-218 focused regression、既存 owner route 44 件、immutable output、real sidecar が active count 167 を保護する。plan は既存 expectation を rebaseline せず 382 cases / 346 requirements、type-elaboration coverage 214/202、pass/fail 198/184 を持ち、object sibling、Base full-distance assertion、generic reachability、object/set coercion、broader semantics は credit しない。

## task 220 traceability

task 220 は shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 74 structural-producer reference 1 件、dedicated checker reference 1 件を持つ exact four-edge object-terminal three-hop asserted-head active row 1 件を追加する。credit は real object expansion 5 個、distinct TooDeep/Inner symbol/site/range、明示検証する relation link 3 本、terminal-only Inner-to-Base-to-object normalization、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinObject` 1 個へ normalize する known entry 3 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。全119 nonidentity definition order、unconnected unsupported deeper asserted head と actual connected sixth-definition/sixth-edge asserted head の独立 guard を含む finite structural/provenance/corruption matrix、Tasks 208 と 211-219 focused regression、既存 owner route 45 件、immutable output、real sidecar は active count 168 を保護する。plan は既存 expectation を rebaseline せず 383 cases / 347 requirements、type-elaboration coverage 215/203、pass/fail 199/184 を持ち、Base full-distance assertion、generic reachability、object/set coercion、broader semantics は credit しない。

## task 221 traceability

task 221 は shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 74 structural-producer reference 1 件、dedicated checker reference 1 件を持つ exact four-edge set-terminal full-distance four-hop asserted-head active row 1 件を追加する。credit は real set expansion 5 個、distinct TooDeep/Base symbol/site/range、明示検証する relation link 4 本、terminal-only Base-to-set normalization、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinSet` 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定する。全119 nonidentity definition order、unconnected-deeper と actual connected fifth-link の独立 guard を含む exhaustive finite structural/provenance/corruption coverage、Task 207 と Tasks 211-220 focused regression、既存 owner route 46 件、immutable output、real sidecar は active count 169 を保護する。plan は既存 expectation を rebaseline せず 384 cases / 348 requirements、type-elaboration coverage 216/204、pass/fail 200/184 を持ち、object sibling、longer chain、imported-positive definition、attributed/argument-bearing behavior、generic reachability、broader semantics は credit しない。

## task 222 traceability

task 222 は shared Chapter 4/13/14.2.3/16 reference 4 件、shared Task 74 structural-producer reference 1 件、dedicated checker reference 1 件を持つ exact four-edge object-terminal full-distance four-hop asserted-head active row 1 件を追加する。credit は real object expansion 5 個、distinct TooDeep/Base symbol/site/range、明示検証する relation link 4 本、terminal-only Base-to-object normalization、ordinal 1 / `BindingId(0)`、Base-definition-RHS `BuiltinObject` 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個だけに限定し、object/set coercion は credit しない。全119 nonidentity definition order、unconnected-deeper と actual connected fifth-link の独立 guard を含む exhaustive finite structural/provenance/corruption coverage、Task 208 と Tasks 211-221 focused regression、既存 owner route 47 件、immutable output、real sidecar は active count 170 を保護する。active corpus は既存 expectation を rebaseline せず 385 cases / 349 requirements、type-elaboration coverage 217/205、pass/fail 201/184 を持ち、longer chain、imported-positive definition、attributed/argument-bearing behavior、generic reachability、object/set coercion、broader semantics は credit しない。

## task 223 traceability

task 223 は shared Chapter 4/13/14.5.2/16 reference 4 件と dedicated checker reference 1 件を持つ exact single-left-parenthesized reserved-variable equality active row 1 件を追加する。Chapter 13 shared row は先行 credit を変えず section label に §§13.1.3/13.8.8 を追加する。新しい credit は real `ParenthesizedTerm` wrapper 1 個、inner/direct-right `x` reference 各1個、独立 wrapper/inner/right source metadata、ordinal 1/2 `BindingId(0)` lookup、inner reference の real reserve-derived builtin-set value/type を既存 equality consumer へ透明に再利用することだけに限定する。parenthesis 独自 type/axiom/fact/FOL node/child-graph は credit しない。finite exact/near-miss/corruption matrix、先行 reserved-variable binary-formula owner 52 件、immutable output、real sidecar は active count 171 を保護する。active corpus は既存 expectation を rebaseline せず 386 cases / 350 requirements、type-elaboration coverage 218/206、pass/fail 202/184 を持つ。focused、relevant-crate、workspace verification は成功し、arbitrary parenthesis/precedence、formula grouping、closure/order、truth/fact、acceptance、proof/IR/VC、broader semantics は credit しない。

## task 224 traceability

task 224 は shared Chapter 4/13/14/16 と Task 74 structural-producer reference 5 件、dedicated checker reference 1 件を持つ active seven-expansion set-terminal two-hop asserted-head row 1 件を追加する。credit は real expansion 7 個、distinct `ChainMode6`/`ChainMode4` provenance、直接検証する bare link 2 本、terminal-only tail normalization、ordinal 1 / `BindingId(0)`、BaseModeDef-RHS `BuiltinSet` 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個に限定する。finite matrix、先行 owner 48 件、immutable output、focused sibling、real sidecar は active count 172 を保護する。active corpus は既存 expectation を rebaseline せず 387 cases / 351 requirements、type-elaboration 219/207、pass/fail 203/184 を持つ。focused、relevant-crate、workspace verification は成功し、broader semantics は credit しない。

## task 225 traceability

task 225 は shared Chapter 4/13/14/16 と Task 74 structural-producer reference 5 件、dedicated checker reference 1 件を持つ active seven-expansion object-terminal two-hop asserted-head row 1 件を追加する。credit は real object expansion 7 個、distinct `ChainObjectMode6`/`ChainObjectMode4` provenance、直接検証する bare link 2 本、terminal-only tail normalization、ordinal 1 / `BindingId(0)`、BaseObjectModeDef-RHS `BuiltinObject` 1 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個に限定する。finite matrix、先行 owner 49 件、immutable output、focused sibling、real sidecar は active count 173 を保護する。active corpus は既存 expectation を rebaseline せず 388 cases / 352 requirements、type-elaboration 220/208、pass/fail 204/184 を持ち、focused、relevant-crate、workspace verification は成功した。broader semantics は credit しない。

## task 226 traceability

task 226 は shared Chapter 4/13/14/16 と Task 74 structural-producer reference 5 件、dedicated checker reference 1 件を持つ active seven-expansion set-terminal three-hop asserted-head row 1 件を追加する。credit は real set expansion 7 個、distinct `ChainMode6`/`ChainMode3` provenance、直接検証する bare link 3 本、terminal-only tail normalization、ordinal 1 / `BindingId(0)`、BaseModeDef-RHS `BuiltinSet` 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個に限定する。finite matrix、先行 owner 50 件、immutable output、focused sibling、real sidecar は active count 174 を保護する。active corpus は既存 expectation を rebaseline せず 389 cases / 353 requirements、type-elaboration 221/209、pass/fail 205/184 を持ち、focused、relevant-crate、workspace verification は成功した。broader semantics は credit しない。

## task 227 active traceability

task 227 は shared Chapter 4/13/14/16 と Task 74 structural-producer reference 5 件、dedicated checker reference 1 件を持つ active seven-expansion object-terminal three-hop asserted-head row 1 件を追加する。credit は real object expansion 7 個、distinct `ChainObjectMode6`/`ChainObjectMode3` provenance、直接検証する bare link 3 本、terminal-only tail normalization、ordinal 1 / `BindingId(0)`、BaseObjectModeDef-RHS `BuiltinObject` 1 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個に限定する。finite matrix、先行 owner 51 件、immutable output、focused sibling、real sidecar は active count 175 を保護する。active corpus は既存 expectation を rebaseline せず 390 cases / 354 requirements、type-elaboration 222/210、pass/fail 206/184 を持ち、focused、relevant-crate、workspace verification は成功した。broader semantics は credit しない。
