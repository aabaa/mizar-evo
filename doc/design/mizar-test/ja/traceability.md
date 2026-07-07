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
してよい。これらの source を syntax-free
checker source reserve payload へ変換し、checker-owned seam が module `BindingEnv`、
binding ごとの `DeclarationInput`、binding 固有の `TypeExpressionInput` site、
`DeclarationChecker` output を構築する。runner はその handoff を checker-owned
`TypedAst`、checker-owned `ResolvedTypedAst` へ継続し、その後 `mizar-core` の
`ResolvedTypedAstSummary::from_ast` と binder-only `CoreContext` preparation で
readiness として読む。
active pass test は、listed source が少なくとも 1 個の抽出済み reserve binding を持ち、
runner regression evidence が checker handoff construction、最小 `TypedAst`、
`ResolvedTypedAst`、summary-readiness、binder-only core context path の実行を確認する場合だけ、
この slice を cover してよい。pass slice は diagnostic external-gap row から credit せず、
専用の traceability row/test を持たなければならない。

case が未対応の non-builtin declaration、imported symbol、unresolved / ambiguous
symbol、attribute / mode / structure argument、term、formula、
coercion、overload payload、fact、CoreIr、ControlFlowIr、VC payload、proof payload
extraction を必要とする場合、
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
または same-module local mode reserve head の missing
mode-expansion payload diagnostic（mixed attributed/bare
local-mode source を含む）の checker detail key を代わりに assert してよい。task 56 の
attributed-chain-dependency fail case は同じ missing mode-expansion payload family に属し、
partial chain expansion は credit しない。attributed-RHS chain も task 58 / task 61 の
direct slice と task 63 の bare one-edge chain slice の外では credit せず、structure-RHS chain も task 60 の direct attributed-root
slice と task 62 の bare one-edge chain slice の外では credit しない。
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
