# Module: vc_ir

> 正本は英語です。英語版:
> [../en/vc_ir.md](../en/vc_ir.md)。

## 目的

この module は `mizar-vc` が所有する prover 非依存の verification-condition IR を
仕様化する。

`VcIr` は phase 11 の出力であり phase 12 の入力である。local context、symbolic
premise、goal、proof hint、source/core provenance、policy-visible status、編集横断
anchor ingredient を記録するが、TPTP、SMT-LIB、backend process setting、proof
witness、kernel acceptance、artifact publication にはコミットしない。

Task 2 は仕様のみである。Rust data shape は task 3、seed intake は task 4、
generation、normalization、`VcId` assignment は generator task 6-8、status と
policy projection は task 9 が実装する。

## 責務

この module が所有するもの:

- `VcId`: 1 つの build snapshot 内で決定的な id;
- `VcKind`: obligation の verifier-facing category;
- `LocalContext`: obligation site で利用可能な explicit formula、binder、path
  condition、policy input;
- symbolic `PremiseRef`: ATP translation が encode する参照。予算に応じた選択は
  下記の VC-owned pretranslation assembly が所有する;
- backend text ではなく core または VC-local formula table への `VcFormulaRef` としての
  goal formula;
- ATP backend を選ばずに user hint、citation、unfold request、computation request
  を保持する `ProofHint`;
- `NeedsAtp` と policy status を含む `VcStatus`;
- seed intake と seed-to-VC accounting record;
- architecture 22 が要求する `ObligationAnchor` ingredient。

範囲外:

- source syntax の parse や missing checker/core payload の復元;
- name resolution、type inference、registration activation、overload resolution、
  elaboration、control-flow construction;
- concrete ATP translation、backend configuration、certificate validation、
  kernel replay、proof acceptance、cache lookup、artifact publication;
- seed、VC、anchor が存在するという理由だけで unverified registration や proof
  result を受理すること。

## この仕様の gap 分類

| ID | 分類 | 証拠 | 扱い |
|---|---|---|---|
| VCIR-G001 | `design_drift` | architecture 07 は一部 seed が 0 個または複数 VC を生成し得るとしているが、task 0 は各 seed が常に 1 個の concrete VC になるように読める古い計画文言を分類した。 | この仕様は seed intake と VC expansion を分離する。すべての handoff entry を正確に 1 回 intake-tracked にし、concrete VC generation は explicit seed-to-VC mapping を記録する。zero-VC の skipped/deferred/error case と将来の multi-VC expansion は可視化する。 |
| VCIR-G002 | `external_dependency_gap` | `mizar-core` は現在、registration/redefinition/reduction correctness を dedicated VC seed kind ではなく `DefinitionCorrectness`、`CheckerInitial`、provenance、non-exhaustive seed kind を通じて運ぶ。 | 利用可能な explicit seed/provenance data を保持し、利用不能な payload は external/deferred と分類し、`VcKind` を forward-compatible に保つ。 |
| VCIR-G003 | `external_dependency_gap` / `deferred` | Task 2 は当初 `mizar-atp`、`mizar-kernel`、`mizar-proof`、`mizar-cache` を inactive consumer として扱っていた。`mizar-kernel` task 23-31 は checker-side formula/substitution evidence path、goal-polarity binding、context-identity verification を提供し、post-closeout VC task 24-28 は producer-side handoff builder、explicit goal-polarity declaration/rejection、reuse identity integration、context-identity payload を提供するが、ATP candidate production、proof/cache consumer、artifact witness consumer は後続 work のままである。 | `VcIr` は prover-independent に保ち、untrusted deterministic evidence reference、status state、reuse-candidate data、task-24/28 handoff rule だけを記録する。SAT clause、backend trace、trusted acceptance field を追加しない。 |
| VCIR-G004 | `test_gap` | task 3 より前には `vc_ir` Rust data shape や tests がない。 | 下記の planned tests を task 3 と task 8 の義務にする。 |
| VCIR-G005 | `source_drift` / `test_gap` | task 8 後には final `VcSet` normalization が存在するが、verifier policy を `NeedsAtp`、`PolicyOpen`、`AssumedByPolicy` status に投影する deterministic API や focused test suite はない。 | Task 9 は immutable `VcSet` data 上の status-policy projection と、status change が context、proof hint、anchor、seed accounting、ATP-bound obligation を保持することの test を追加する。 |
| VCIR-G006 | `deferred` | Discharge evidence と dependency slice は後続 VC task で実装済みであり、post-closeout VC task 24-28 は producer-side kernel-evidence handoff、explicit goal-polarity declaration/rejection、reuse identity integration、context-identity payload を実装済みであり、`mizar-kernel` tasks 30-31 は trusted goal-polarity と F2 membership checks を実装済みである。ATP translation、downstream proof/cache/artifact または corpus consumer、source-derived corpus runner activation は後続 task または external seam のままである。 | Task 9 は discharge evidence 生成、dependency slice 計算、ATP problem translation、kernel/proof/cache/corpus path 呼び出し、downstream integration record の fabricate をしてはならない。 |

Task 2 では `doc/spec` や `.miz` tests を変更しない。この仕様は既存 architecture と
core handoff contract を精緻化するものであり、新しい言語 semantics を導入しない。

## 概念的な package shape

実装は正確な Rust field 名を選べるが、次の semantic shape を保存しなければならない。

```rust
struct VcSet {
    schema_version: VcSchemaVersion,
    snapshot: BuildSnapshotId,
    source: SourceId,
    module: VcModuleRef,
    generated_formulas: Vec<VcGeneratedFormula>,
    vcs: Vec<VcIr>,
    seed_accounting: SeedAccountingTable,
}

struct VcIr {
    id: VcId,
    kind: VcKind,
    source: VcSourceRef,
    seed: SeedVcRef,
    anchor: ObligationAnchor,
    local_context: LocalContext,
    premises: Vec<PremiseRef>,
    goal: VcFormulaRef,
    proof_hint: Option<ProofHint>,
    status: VcStatus,
    provenance: Vec<VcProvenance>,
}
```

`VcSet` は immutable snapshot である。後続 phase は side table や status projection を
生成できるが、`CoreIr`、`ControlFlowIr`、seed handoff を mutate してはならない。

`VcModuleRef` は、task 3 で `mizar-resolve` への新規依存を追加せずに、生成元 core
snapshot が与える正準 module identity を保持する。後続の boundary task は、workspace
dependency boundary と lint guard を同じ task 内で更新する場合に限り、これを直接の
resolved module id に置き換えてよい。

`VcFormulaRef` は borrowed core formula または VC-local generated formula を識別する:

```rust
enum VcFormulaRef {
    Core(CoreFormulaId),
    Generated(VcGeneratedFormulaId),
}

struct VcGeneratedFormula {
    kind: VcGeneratedFormulaKind,
    formula: CoreFormulaShape,
    provenance: Vec<VcProvenance>,
}
```

実装は既存の `mizar-core` formula constructor や normalization helper を再利用してよいが、
VC-local generated formula は upstream `CoreIr` ではなく `VcSet` 内に置く。そのため
generated formula は独自の snapshot-local id を持ち、seed、expansion index、生成した
generator rule を復元できる十分な provenance を持たなければならない。

## VcId

`VcId` は:

- 1 つの `BuildSnapshot` 内で決定的;
- canonical VC order において dense;
- snapshot 内の diagnostic、artifact ordering、result collation、stable debug
  rendering に使う;
- 編集横断 proof-reuse identity ではない。

`VcId` assignment は generator task 8 の責務である。より前の task は、concrete
`VcIr` として外へ出ない場合に限って placeholder / builder record を構築してよい。

編集をまたぐ reuse では、次の一致だけでは proof reuse を許可しない:

- 一致する `VcId`;
- 一致する source range;
- 一致する surface/syntax/arena id;
- 一致する `ObligationAnchor`。

編集横断 reuse にはさらに architecture-22 validation key が必要である: canonical VC
fingerprint、local-context fingerprint、dependency-slice fingerprint、互換 verifier
policy、一致する proof witness または deterministic discharge hash。

## VcKind

`VcKind` は ordering、diagnostic、policy、後続 consumer filtering のために obligation を
分類する。必須 category:

- theorem / lemma proof step;
- terminal proof goal;
- definition correctness;
- explicit core/checker seed が registration、redefinition、reduction correctness を表す場合の registration-style correctness;
- checker-initial carried obligation;
- generated non-emptiness / sethood obligation;
- generated Fraenkel membership axiom;
- algorithm precondition / postcondition;
- call precondition;
- algorithm assertion;
- loop invariant entry、preservation、break、continue obligation。existing exit shape は
  normal exit が存在するだけでは source-generate しない;
- range-loop positive-step obligation。existing range-bound/hidden-index shape は
  explicit canonical formula なしに source-generate しない;
- collection-loop finiteness と order-independence obligation;
- termination obligation。existing partial-termination shape は current canonical spec
  から source-generate しない;
- existing ghost-erasure-safety shape は validation/sort shape のみ。ghost isolation は
  static check で source-generated VC ではない;
- policy / deferred traceability record。

Rust enum は downstream forward-compatible である。Task 17 が最終的な public enum policy を
記録するが、implementation task は owning spec update なしに exhaustive public enum を
公開してはならない。

kind による ordering は task 8 で安定化・文書化する。ordering は hash-map iteration、
worker completion order、backend availability に依存してはならない。

## Seed accounting

入力は `mizar_core::control_flow::ObligationSeedHandoff` であり、既存 core seed と
flow-derived seed を含む。`mizar-vc` は missing seed data を復元するために raw syntax を
見てはならない。

すべての handoff entry は正確に 1 回 intake-tracked される。

```rust
struct SeedAccounting {
    handoff: ObligationHandoffId,
    origin: SeedOriginRef,
    seed_status: ObligationSeedStatus,
    mapping: SeedVcMapping,
}

enum SeedVcMapping {
    NoConcreteVc { reason: SeedNoVcReason },
    One { vc: VcId },
    Expanded { vcs: Vec<ExpandedVcRef>, expansion_schema: ExpansionSchemaVersion },
}

struct ExpandedVcRef {
    expansion_index: usize,
    vc: VcId,
}
```

規則:

- seed accounting row なしの concrete `VcIr` は存在しない;
- goal を持つ `Active` seed は concrete VC generation の対象になる;
- goal と supported explicit `ControlFlowObligationSite` metadata を持つ `Deferred`
  flow-derived `AlgorithmContract` seed も concrete VC generation の対象になる。
  `mizar-core` は task-7 algorithm schema を `mizar-vc` が適用するまでこれらの row を
  deferred として印付けるためである。元の deferred `seed_status` は task-8 accounting 用に
  記録され続ける;
- `Skipped`、`Deferred`、`Error` seed は diagnostic または provenance reason を持つ場合、
  concrete VC を 0 個にできる;
- disabled seed は visible no-VC mapping を使ってよいが、policy-open obligation は
  `PolicyOpen` status を持つ concrete VC のままにする。どちらの場合も silent omission
  にしてはならない;
- multi-VC expansion は、generated VC ごとの stable zero-based dense
  `expansion_index` を記録する explicit `Expanded` mapping を通じてのみ許される;
- ordinary theorem、definition、checker-initial、generated type、eligible な goal-bearing
  flow-derived milestone seed は、owning generator spec が特定の expansion schema を記録しない限り
  `One` を使う;
- 同じ canonical seed key と origin を持つ duplicate handoff entry は、`VcId` assignment 前に
  拒否するか deterministic diagnostic として表す。

これにより task-0 drift を解消する。「exactly once」は各 seed が正確に 1 回 accounted される
ことを意味し、concrete VC cardinality は explicit かつ audit 可能である。

### Task 4 の intake table

Task 4 は `VcId` 割り当て前の intake table を実装する:

```rust
struct SeedIntakeTable {
    rows: Vec<SeedIntakeRow>,
}

struct SeedIntakeRow {
    handoff: ObligationHandoffId,
    origin: SeedOriginRef,
    seed_status: ObligationSeedStatus,
    canonical_key: ObligationSeedCanonicalKey,
    source: CoreSourceRef,
    mapping: SeedIntakeMapping,
}

enum SeedIntakeMapping {
    EligibleOneVc { goal: CoreFormulaId },
    NoConcreteVc { reason: SeedNoVcReason },
}
```

この table は `VcSet` に格納される最終的な `SeedAccountingTable` ではない。`VcId`
を割り当てず、concrete `VcIr` も構築しない。handoff order と seed origin を保持し、
duplicate `(canonical_key, origin)` row を決定的に拒否し、後続の `VcId` assignment
より前に matching `source_map` entry を欠く handoff row も拒否する。
skipped/deferred/error/missing-goal row は、task-7-eligible な goal-bearing deferred
`FlowDerived` `AlgorithmContract` row が supported explicit flow-site metadata を持つ場合を
除き、visible no-VC mapping として記録する。Task 8 は deterministic `VcId` を割り当てる
ときに eligible row を消費する。

## LocalContext

`LocalContext` は self-contained である。ATP translation は semantic context を source text から
再構築してはならない。

必須 context component:

- binder declaration と normalized role;
- obligation site で利用可能な type predicate と sethood/non-emptiness fact;
- proof assumption、current thesis fact、scope 内 local label;
- `by` justification 由来の cited premise を symbolic reference として記録したもの;
- `ControlFlowIr` から準備された algorithm path condition、loop invariant availability、post-havoc fact;
- explicit に利用可能な checker fact、registration/cluster/reduction trace reference、inserted `qua` evidence;
- definition unfolding policy と selected local unfold request;
- status、dispatch、computation limit に影響する verifier policy input。

context entry は stable source/core provenance と canonical sort key を持つ。fingerprint や
debug rendering の前に決定的に sort しなければならない。

## PremiseRef

`PremiseRef` は symbolic である。prover syntax、selected ATP encoding、backend-local axiom name は
保存しない。

必須 premise reference class:

- local context formula;
- local proof label / citation;
- accepted dependency artifact から imported された theorem / lemma fact;
- definition boundary または permitted unfolding;
- checker fact または type predicate;
- registration、cluster、reduction trace reference;
- core elaboration 由来の generated fact;
- active verifier policy が許す場合の policy-provided assumption marker。

unknown、不完全、利用不能な premise data は empty premise set と解釈してはならない。dependency
slice を保守的にするか、diagnostic つき deferred/error status に VC を保つ必要がある。

## Planned Pretranslation Selection Assembly

VC は ATP の予算に応じた順序付き選択を受け取る immutable input assembly を所有する
（§§21.4.1,22.4.4）。producer は真正な source obligation、適格な局所・引用 fact と
formula projection、visible-fact index、有効な package/step budget、名前付き ranking
configuration を供給する。選択した translation input の局所・引用 axiom の総数は
有効な `max_axioms` を超えてはならない。元の VC、goal、source context 全体は変えない。選択した行は
formula・source/citation・context の binding を保持し、選択した problem は元の target と
budget/configuration identity に対応する。高順位の suggestion だけでは source proof の
適格な前提にならない。import 行は既存の [handoff authority](kernel_evidence_handoff.md#boundary-rules) を保つ。

予算超過の選択、非member行、同じ assembled row の重複選択、formula/provenance/context の欠落・不一致、
target/problem/configuration identity の古さは translation input の生成前に拒否する。
部分的・空の fallback や display/backend data からの再構成は行わない。translator と既存
handoff builder は前提の選択・変更をしない。現行 consumer API はこの selected-input 関係を
まだ提供せず、統合は上記の真正な producer と assembly に依存する。

将来の source test は予算内・ちょうど予算の入力で適格な局所・引用行をすべて保持し、
予算超過では cutoff tie と入力順の変更を含めて決定的な上位 budget 件の axiom subset を保持する。
package budget と step override を試し、元の VC/goal/context の不変性と全選択 binding を
比較する。破損ケースは真正な行だけの予算超過選択も含め、target、membership/重複、formula payload、provenance、context、
import authority、problem/configuration identity を一つずつ変え、translation 前の拒否を
確認する。pruning 失敗の診断と診断用 retry は ATP 所有で、この assembly は証明を受理しない。

## Goal formula

goal は `VcFormulaRef` である。producing `CoreIr` snapshot への `CoreFormulaId`、または
explicit generated-origin provenance を持つ VC-local generated formula を指す。

goal は次であってはならない:

- source string;
- parser/syntax id;
- backend text;
- source からコピーした unchecked theorem statement;
- missing payload を通すために作った formula。

Generated conjunction または split goal は `VcSet` の generated formula table に保存し、
task 8 と task 20 が `CoreIr` を mutate せず決定的に fingerprint できるよう、
generated-origin と seed expansion provenance を記録しなければならない。

## ProofHint

`ProofHint` は author input と policy input を保持する。backend dispatch configuration ではない。

必須 hint data:

- explicit `by` citation と local label;
- local unfold request と definition opacity override;
- `@proof_hint` 形式の premise restriction または backend-abstract intent;
- `@proof_hint` の `solver`、`max_axioms`、`timeout` option を symbolic policy input として保存したもの;
- `by computation` または verification-time computation request;
- task 10 が limit model を固定した後の computation-limit policy reference;
- すべての hint の source/core provenance。

未対応 hint は diagnostic または deferred status record として表す。context を silently drop したり
ATP dispatch を強制したりしてはならない。

## VcStatus

`VcStatus` は後続 consumer から見える phase-11/12 verifier state を記録する。

必須 state:

- `Open`: 生成済みで、まだ discharge または ATP 分類されていない;
- `NeedsAtp`: canonical VC を `mizar-atp` が translate しなければならない;
- `Discharged`: phase 12 が deterministic、replayable、untrusted evidence を生成した;
- `PolicyOpen`: active policy が意図的に VC を open にし、ATP dispatch しない;
- `AssumedByPolicy`: active policy が assumption marker を許すが proof evidence ではない;
- `SkippedDueToInvalidInput`: seed または owner が invalid/skipped であり concrete proof obligation を dispatch しない;
- `DeferredExternal`: 必要な upstream/downstream seam が利用できない;
- `Error`: deterministic VC generation が失敗した。

status 規則:

- `NeedsAtp` VC は full local context、premises、source、anchor、proof hint を保持しなければならない;
- policy state は underlying seed accounting や source provenance を消さない;
- `Discharged` evidence は kernel-verified proof status ではない;
- status change は deterministic かつ auditable でなければならない;
- status transition は task 9 と discharge task が所有し、raw data-shape constructor は所有しない。

## Task 9 Implementation Slice

Task 9 は immutable `VcSet` 上に deterministic status-policy projection を追加する。
projection は新しい `VcSet` を返し、入力 set を mutate してはならない。

Task 9 が support するもの:

- 現在の status の保持。
- concrete VC を `NeedsAtp` として mark すること。
- explicit policy key 付きで concrete VC を `PolicyOpen` として mark すること。
- explicit policy key と premise marker 付きで concrete VC を `AssumedByPolicy` として
  mark すること。

Policy override は `VcId` 順に sorted され、同じ `VcId` を duplicate せず、既存 VC を
target にしなければならない。missing target、duplicate override、unsorted override は
deterministic error である。`AssumedByPolicy` marker は既存の `VcSet` validation path に
よって検証されるため、invalid context、premise、generated formula reference は fail closed する。

Status change は変更された VC に `StatusPolicy` provenance を追加しなければならない。
preserve/no-op action は provenance を追加してはならない。Projection は `VcId`、order、
kind、source ref、seed ref、anchor、local context、premise、goal、proof hint、
generated formula table、seed accounting、既存の non-status provenance を保持しなければならない。
`Discharged` evidence を作成してはならない。evidence creation は deterministic discharge task
が所有する。dependency slice 計算、ATP 呼び出し、corpus fixture activation、
kernel/proof/cache result の受理、新しい generator payload family の追加も行ってはならない。

## ObligationAnchor

`ObligationAnchor` は best-effort な編集横断 candidate identity である。proof evidence ではなく、
kernel から信頼されない。

必須 ingredient:

- normalized owner identity: theorem、definition、registration、generated symbol、algorithm、proof block、checker-origin row;
- `VcKind`;
- seed handoff 由来の anchor-ready `LocalProofOrProgramPath`;
- 利用可能な label role と optional label hint;
- normalized semantic origin;
- source range が利用可能な場合を含む source/core provenance;
- source-shape hash または conservative unavailable marker;
- canonical goal hash;
- canonical local-context hash;
- generation schema version。

`VcId`、`SourceRange`、`SurfaceNodeId`、rowan identity、arena id、parser order、handoff-local id は
snapshot-local evidence として現れてよいが、編集横断 proof-reuse identity として使ってはならない。

必須 anchor ingredient が利用不能な場合、anchor は incomplete と印付けし、downstream proof/cache
reuse は fail closed しなければならない。

## Task 20 fingerprints

Task 20 は generated obligation 向けの deterministic cross-edit fingerprint helper を追加する:

- `CanonicalVcFingerprint` は VC kind、canonical goal payload、symbolic premise、
  proof hint、owning `VcSet` の generated-formula table から解決した generated formula
  payload を対象にする。
- `LocalContextFingerprint` は stable sort key、kind、解決済み formula payload、
  provenance、explicit verifier-policy input によって local-context entry を対象にする。

これらの fingerprint は `VcId`、source range、`SourceId`、handoff id、candidate sort key、
単一 build snapshot に局所的な row id を除外する。Generated formula reference は hash 前に
formula kind / shape / provenance payload へ解決される。invalid set に含まれる未解決 generated
formula reference は、reuse input になる前に validation で失敗しなければならない。
`CoreFormulaId`、`CoreDefinitionId`、dense owner id のような opaque upstream row identifier は
編集横断 payload ではない。stable な formula、definition、owner、context payload が `mizar-vc`
から利用できない場合、fingerprint helper は fingerprint を返さず、downstream reuse は fail
closed しなければならない。
Task 20 では quantified generated formula も、stable な binder-entry payload が利用可能でない限り
fail closed である。binder count や `ContextEntryId` だけでは canonical VC fingerprint を構成しない。

Task 20 は generated `ObligationAnchor` に source-shape、canonical-goal、canonical-context
hash marker も接続する。source-shape hash は source-shaped provenance が利用可能な場合に
available になり、owner class、`VcKind`、local proof/program path、label、semantic origin、
source/core provenance marker のような stable ingredient から導出する。`VcId`、source range、
`SourceId`、handoff id、candidate sort key、dense owner row id から導出してはならない。
Canonical goal/context hash marker は stable payload が利用可能な場合だけ available になる。
現在の CoreFormulaId-only goal と context entry は incomplete / conservative-unknown reuse input
のままである。

## 決定的 rendering

Task 3 は `VcIr` と関連 table の deterministic debug rendering を実装する。rendering は次を含む:

- schema version;
- `VcId` 順に sort した VC row;
- seed accounting row;
- source/core provenance summary;
- canonical order の local context entry と symbolic premise;
- status と policy reference;
- anchor ingredient と incomplete-anchor marker。

rendering は nondeterministic address、timing、worker id、absolute local path、hash-map iteration
order、backend runtime data を含めてはならない。

## planned tests

Task 3 は Rust coverage として次を追加しなければならない:

- explicit context、premise ref、goal、hint、anchor、status、seed accounting を持つ minimal
  `VcIr` の構築;
- complete set を validate する constructor では duplicate `VcId` や unsorted/invalid accounting を拒否すること;
- backend text なしで symbolic premise ref を保持すること;
- `NeedsAtp` と policy status が context を落とさないこと;
- 同じ fixture が run をまたいで byte-identical に render されること;
- incomplete anchor / dependency marker を cache-miss data として保持すること。
- generated / split goal を `CoreIr` を mutate せず VC-local generated formula table に保存すること。

Task 4 と task 8 は deterministic seed intake、duplicate seed rejection、seed-to-VC mapping、
`VcId` assignment の coverage を拡張する。Task 9 は deterministic status-policy projection、
`NeedsAtp` classification、policy-open / policy-assumed status、invalid override の拒否、
status change をまたいだ context、proof hint、anchor、generated formula、seed accounting の保持に
coverage を拡張する。

Task 2 では active `.miz` fixture を追加しない。`proof_verification` runner と source-derived payload
seam が external gap として残るためである。

## Task 30 source-derived identity contract

[source_vc_decomposition.md](./source_vc_decomposition.md) は exact Task-180 phase-11
identity を固定する。Task 31 は kind `TerminalProofGoal`、status `Open`、Core formula 0、
empty context/premise、exact theorem owner/path/source/origin、Active/ExistingCore seed mapping
1件を持つ snapshot-local `VcId(0)` 1件を生成する。current VC payload は Core formula id だけを
持つため canonical-goal hash は unavailable のままである。source-shape/empty-context hash は
available にできるが、anchor は明示的 incomplete のままで proof reuse を許可しない。

Tasks 32-55 は honest canonical representation を最初に必要とする nonempty real-source task
内でだけ `VcKind`、generated-formula shape、context role を拡張できる。algorithm type
correctness や explicit match exhaustiveness を unrelated existing kind に encode せず、dependent
binder/path order を flatten せず、label から formula identity を合成せず、Deferred input を
general に Open へ promote しない。各 extension は `#[non_exhaustive]` policy を維持し、validation、
deterministic rendering、corruption test、real `MT10-VC-PV/VC<n>` baseline と同時に実装する。

Existing `PartialTermination`、`GhostErasureSafety`、loop-exit、range-bound、hidden-index
variant 自体は source generation authority を与えない。

## public enum policy

task 17 は `vc_ir` の public enum をすべて downstream forward-compatible API surface
として分類する。後続の VC、policy、evidence、dependency、diagnostic category を downstream
の exhaustive match を壊さず追加できるよう、各 enum は `#[non_exhaustive]` を維持しなければならない。

| public enum | decision |
|---|---|
| `VcStatusAction` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `VcGeneratedFormulaKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `VcGeneratedFormulaShape` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `QuantifierKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `VcFormulaRef` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `VcKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `RegistrationCorrectnessKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `LoopInvariantPhase` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `RangeLoopObligation` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CollectionLoopObligation` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ContextEntryKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `PremiseRef` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `DefinitionOpacityOverride` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `PremiseRestriction` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ComputationHint` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `VcStatus` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SeedIntakeMapping` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SeedOriginRef` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SeedVcMapping` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `SeedNoVcReason` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `AnchorOwner` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `AnchorLabelRole` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `AnchorCompleteness` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `AnchorIngredient` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `VcProvenancePhase` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `HashMarker` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `VcIrError` | `#[non_exhaustive]` downstream forward-compatible surface。 |

この module が所有する exhaustive public enum exception はない。現在の variant を意図的に
列挙する `mizar-vc` 内部 match は exhaustive のままでよい。

## Concrete return equality

`VcGeneratedFormulaKind::AlgorithmPostcondition` と `VcGeneratedFormulaShape::Equals { left: CoreTermId, right: CoreTermId }` は既存の不変 Core 項上の実際の置換済み等式を保持する。generator が両参照と所有者を Core に照合し、別の項表現や文字列式を導入しない。debug bytes は両オペランドの順序を保持する。Core payload がなければ canonical identity・依存・再利用解析は未解決とし、discharge は Unknown を返す。数値 ID の一致だけでは証明しない。

`VcProgramValue { var: CoreVarId, definition: Option<CoreAlgorithmStmtId> }` は None で不変仮引数、Some で実際の宣言・書込み・runtime Pick 直後の値を識別する。Some(ループ文) は認証済み MayWrite ローカルの任意 cutpoint 値も識別し、代入等式を意味しない。generator が所有者・代入先・ソース順序・定義後使用を Core に照合する。
`ProgramEquals` と `ProgramTypePredicate` はこの値と実際の検査済み述語を保持し、AlgorithmStateFact が宣言・書込み文脈を AlgorithmAssertion 目標と区別する。Core は不変とし、Core payload が不明なら真偽・fingerprint・再利用を保守的に扱う。
`ContextEntryKind::PendingAlgorithmAssertion { handoff }` は One seed accounting を通じて後続の仮定を厳密な先行 assertion 目標へ結び付け、所有者・目標の不一致、自己依存、循環を拒否する。未解決の証明依存であり、直接・偽の承認済み前提や再利用可能な証拠にはしない。

## Builtin choice accounting

`SeedNoVcReason::BuiltinSetInhabitation { origin: GeneratedOriginId }` は [source
generator](./generator.md#source-existential-registration-proof) の accounting であり skipped/deferred や
Discharged evidence ではない。VcSet 構造検査は active な ExistingCore seed origin を要求し、参照 generated origin と実 seed goal
の認証は全 Core 再生成が行う。caller が構築できる値から汎用 consumer が真偽・受理を導いてはならない。

## Source registration canonical VC fingerprints

`VcSet::source_registration_canonical_fingerprints(&self, core: &CoreIr, formulas: &BTreeMap<CoreFormulaId, Vec<u8>>) -> Result<BTreeMap<VcId, CanonicalVcFingerprint>, String>` は既存 single/three existential-registration source generator だけを対象とする。自身の snapshot・VC schema・先頭 leaf の generation schema で VcSet 全体を再生成し、Open の完全一致か default try_discharge VcSet の完全一致だけを許す。上流 error を保ち、それ以外の未対応・不一致・空入力は内部 error unsupported source registration VC fingerprint として部分 map を返さない。既存 generic fingerprint・生成・status・入力は変えない。
caller は実際の checked-lowered Core と、全 goal/generated-fact premise が参照する Core root をちょうど覆う、変更しない同一 source の [Core formula payload](../../mizar-core/ja/elaborator.md#source-registration-formula-payloads) を渡す。value は非空、key は完全一致とし、異なる root の同じ value は許す。検査と generator replay は opaque bytes の由来、checker seal、source currentness、実行由来を認証しない。trusted consumer の前には将来の fresh source factory が実 producer を一つの呼出し内で結び付ける必要がある。
実 table の generated 参照を再帰解決し、欠落・循環・未対応は全体を拒否する。kind は GeneratedTypeObligation・NegatedPremise・Conjunction、shape は Ref・Not・And・Or のみ。完全な source 再生成で既存 dense CoreHandoff provenance を照合し、debug node/row 参照の代わりに固定 rule source-existential-registration-v1 と kind・順序付き shape・実 Core 内容へ正規化する。child 順を保ち、local entry/policy input は空、hint は無し、premise は GeneratedFact のみ許す。
式 bytes は tag 0 と frame 付き opaque Core bytes、または tag 1、kind byte 1 GeneratedTypeObligation / 2 NegatedPremise / 3 Conjunction、UTF-8 source-existential-registration-v1 の frame、shape byte 1 Ref / 2 Not / 3 And / 4 Or。Ref/Not は再帰 child 一つ、And/Or は checked big-endian u32 child 数と順序付き child を続ける。frame は全て checked big-endian u32 byte 長と bytes。dense/source/snapshot id、range、binder 表示名、debug provenance は含めない。
VC payload は UTF-8 mizar-vc/source-registration-canonical-vc/v1 と zero で始め、VC schema・generation schema の frame、RegistrationStyleCorrectness/Registration の kind byte 1、goal bytes の frame、checked big-endian u32 premise 数、辞書順で並べた premise bytes を個別 frame で続ける。premise bytes は GeneratedFact の byte 1 と式 bytes。重複数を保ち、参照 ID でなく解決後 bytes を並べる。空 context entry・空 policy input・hint 無しを表す zero 三つを末尾に置く。既存 stable_fingerprint_hash に domain mizar-vc-source-registration-canonical-vc を渡し、既存 CanonicalVcFingerprint に格納する。key は現在の VcId。
明示した入力責任の下で限定 VC 内容を識別する。status、anchor、source-shaped identity、完全な依存合成、deterministic discharge evidence hash、policy/witness identity、proof/cache/publication eligibility は別責務。既存の不完全 marker・generic method を変更せず、reuse key・Complete result・受理済み witness を生成しない。
テストは実 single/aggregate source と現在の owner/leaf 対応、固定 format から独立に求めた hash、Open/default-discharge 一致と不変入力、neutral edit・実 allocator relocation・宣言/registration 並べ替えと解決済み premise 順、body/owner/module/schema の参加、実 foreign graph、欠落・空・余分な実 root payload、構造 VC 改変と空 set の拒否を扱う。既存 Core/owner profile matrix 全体は複写せず再利用する。

## Source registration canonical goal and context

`VcSet::source_registration_canonical_goal_context(&self, vc: VcId, core: &CoreIr, formulas: &BTreeMap<CoreFormulaId, Vec<u8>>) -> Result<(Hash, Hash), String>` は現在の選択 leaf の goal hash、context hash の順で返す。
まず集合全体と完全な root map に [source canonical VC validation](#source-registration-canonical-vc-fingerprints) を適用し、全上流 error を保つ。その後に選択 VcId の存在を要求し、無ければ同じ内部 error unsupported source registration VC fingerprint を返す。両 hash は不可分に返し、入力を変更しない。
リンク先 profile の実 checked Core、変更しない同一 source の opaque Core55 bytes、対応 source graph、Open/default-discharge 対応、caller が負う provenance/currentness の責任を緩めない。既存 frame/formula encoder を共有し、旧 bytes と generic API を変えない。

選択 goal と全可視 GeneratedFact premise を同じ式 grammar で符号化する。context は実 witness-type・展開済み definition premise と重複数、空 local entry/policy input、hint 無しを含む。source 専用 context profile とし、既存 generic LocalContextFingerprint は local-row profile を保つ。
goal payload は UTF-8 mizar-vc/source-registration-canonical-goal/v1 と zero、VC schema・generation schema・goal bytes の frame の順。既存 stable_fingerprint_hash に domain mizar-vc-source-registration-canonical-goal を渡す。
context payload は UTF-8 mizar-vc/source-registration-canonical-context/v1 と zero、VC schema・generation schema の frame、checked big-endian u32 premise 数、辞書順の premise vector を個別 frame にして続ける。vector は GeneratedFact の byte 1 と式 bytes。空 local entry・空 policy input・hint 無しの zero 三つを末尾に置く。domain は mizar-vc-source-registration-canonical-context。frame 長は checked big-endian u32 byte 長を保つ。
両 component は VC 全体の fingerprint、Kernel/Artifact hash、source-shaped identity、完全な合成 dependency footprint、受理 evidence を代替しない。既存 anchor marker・依存完全性・status・policy・freshness・reuse eligibility・publication は変えず、trusted consumer の前には将来の source assembly が実 producer を結び付ける。

三つのテスト群で既存の実 source setup と現在の owner/leaf 対応を再利用する。独立した固定 format の goal/context 期待値と Open/default-discharge equality、neutral relocation/並替え・解決後 premise 内容順・schema/owner/module の影響と goal を保って該当 context を変える definition-body edit、全入力/選択 leaf の拒否・実上流 error・API 入力不変性を確認する。既存 canonical VC 期待値を保ち、旧 producer/profile の全 matrix は複製しない。

## Source registration source-shape hashes

`VcSet::source_registration_source_shape_hashes(&self, core: &CoreIr, formulas: &BTreeMap<CoreFormulaId, Vec<u8>>, syntax: &BTreeMap<CoreItemId, Vec<u8>>) -> Result<BTreeMap<VcId, Hash>, String>` は最初に既存の全 source canonical VC 検証を行い、上流エラーを保持する。呼出側は、現在の各 Core Registration 項目の実 SymbolId に対応する変更していない [Resolver 構文ペイロード](../../mizar-resolve/ja/symbols.md#source-registration-syntax-payloads)を渡す。現在の全 Registration キーとの完全一致、非空で相互に異なる値を要求する。キーは同じグラフ内の対応付け専用で、ID／SymbolId 表現を直列化しない。Resolver の production 依存や構文解析を追加しない。
実 SeedAccounting の Expanded 行を走査し、NoConcreteVc は無視、他 mapping は拒否する。ExistingCore origin、実 seed と Registration owner、CheckerInitial 種別、local_path `registration/existence`、展開 schema `source-existential-registration-v1` を要求する。実 ExpandedVcRef の現在の target と expansion_index を使い、全現在 VC を重複なく返す。既存の全 source replay が対応する展開構造を検証する。以後の profile／対応／framing 失敗は `unsupported source registration source shape` とし、部分結果を返さない。
各 leaf の内容は UTF-8 `mizar-vc/source-registration-source-shape/v1` とゼロで始め、VC schema、leaf の generation schema、実 expansion schema を順に checked big-endian u32 バイト長と内容で frame 化する。この検証済み登録 correctness 種別の byte 1、実際に検証した seed local_path の frame、checked big-endian u32 expansion_index、変更しない構文バイト列の frame を続ける。既存 stable_fingerprint_hash の domain `mizar-vc-source-registration-source-shape` を使う。既存 source_registration_frame を再利用し、その拒否は本操作の profile エラーへ変換する。他フィールドはハッシュしない。
この限定的なローカル構文 profile は、Resolver 内容内の実名付き所有者、記述トークン／形状、source seed path、展開役割を含む。trivia・割当・宣言順や、その宣言外の属性定義本体などの編集では同じ所有者の値を保持する。所有者／module、ローカルトークン／形状、schema、leaf 役割の変更は値に関与する。意味上の goal／context は別責任のままで、外部本体の変更時も再利用検証に必要となる。
opaque バイトの来歴と source／snapshot の現在性は呼出側責任である。構造 replay、キー完全一致、値の非空／相違は新しい Resolver 出力であることの認証ではない。将来の Proof 結合は実生成を同じ呼出内で対応付ける。返すのは内容ハッシュのみで、実行証跡、完全な正規化 anchor／semantic origin、依存関係／Proof identity、受理／再利用可能性や公開ではない。VcSet／Core、汎用 source-shape marker、anchor completeness、status、既存 fingerprint 振舞いを変更しない。
テストは実単一／集約 source pipeline と Resolver 構文生成を使い、実 Core symbol を現在の item キーへテスト内でのみ対応付ける。実装前に独立した手動文法と固定 schema／path／leaf／hash framing から期待値を定める。三系列で正確な現在 leaf／default discharge、中立／関与編集・snapshot・実 owner ID 再配置・外部本体不変・leaf 区別、全入力／上流拒否と不足／余分／空／重複 syntax map、入力／元 anchor 不変を検証する。構造チェックを認証とせず、架空の正常 source 対応を作らない。

## Source registration candidate anchor hashes

`VcSet::source_registration_candidate_anchor_hashes(&self, core: &CoreIr, formulas: &BTreeMap<CoreFormulaId, Vec<u8>>, syntax: &BTreeMap<CoreItemId, Vec<u8>>, owners: &BTreeMap<CoreItemId, Vec<u8>>) -> Result<BTreeMap<VcId, Hash>, String>` は最初に全 [source-shape map](#source-registration-source-shape-hashes) を得て上流エラーを保持する。owner キーは検証済み syntax キーと完全一致し、値は非空で相互に異なるものとする。呼出側は各実 Core item の SymbolId に対する変更していない Resolver registration owner bytes を渡す。ID は対応付けだけに使い、構造検証から opaque owner／syntax／formula の来歴・現在性を認証しない。Resolver bytes の解釈・再符号化は行わない。
各実 VC について replay 済み Registration owner と anchor local_path、未変更の syntax 由来 shape、実 [canonical goal/context](#source-registration-canonical-goal-and-context) を使う。全現在 VcId を返すか全体拒否し、以後の owner／framing／profile エラーは `unsupported source registration candidate anchor` とする。既存 source_registration_frame と stable_fingerprint_hash を再利用し、新 encoder／production 依存は作らない。
内容は UTF-8 `mizar-vc/source-registration-candidate-anchor/v1` とゼロ、VC schema・leaf generation schema・未変更 registration owner bytes・実 anchor local_path の順の checked big-endian u32 バイト長 frame、検証済み registration correctness 種別の byte 1、source-shape・canonical goal・canonical context 各 hash の raw 32 bytes の順とする。domain は `mizar-vc-source-registration-candidate-anchor` で、他の値は加えない。owner bytes は実 package／module／登録 label の意味的 origin を含み、shape は実展開役割により同一 seed path の leaf を区別する。元 anchor にない label hint を捏造しない。
この正規化候補内容は dense owner id・VcId・SourceId・snapshot・range・provenance 表現を含まない。対応済み移動／並替／trivia／外側 binder 編集では同じ値を保つ。ローカル構文／owner／schema は関与し、外部定義本体は local shape を保って context と候補を変え得る。raw 候補 hash であり、構造化 Complete anchor、VC 全体や依存 fingerprint、trusted receipt、再利用許可・受理状態ではない。既存 graph／anchor／marker／汎用 method を保持する。
Proof と共有する三つの実 integration test 系列で、独立した単一／集約 source の固定候補値・現在 leaf・下位 producer 一致、中立／関与編集・実 ID 再配置・展開 leaf 区別・外部 shape/context 分離、全入力上流／profile 拒否・不足／余分／空／重複 owner map・不在選択・実 foreign/malformed/unsupported source・入力不変を確認する。既存 component/profile matrix は複製しない。

## Source theorem canonical components

`VcSet::source_theorem_canonical_components(&self, vc: VcId, core: &mizar_core::core_ir::CoreIr, formulas: &BTreeMap<CoreFormulaId, Vec<u8>>) -> Result<(CanonicalVcFingerprint, Hash, LocalContextFingerprint), String>` は限定した VC・goal・local-context 内容をこの順に返し、`crates/mizar-vc/src/vc_ir.rs` の既存 wrapper を使う。
最初に未変更の [build_source_theorem_kernel_handoff](./kernel_evidence_handoff.md#source-theorem-local-handoff)(core, self, vc) を呼び、全上流 error を保持して untrusted handoff を破棄する。元の Open、単一 uncited Valid Theorem／一つの proof-let set／reflexive local-goal profile と検証済み構造だけを再利用し、discharged/projected graph、広い第二 validator、Kernel/ATP 実行、handoff hash 再解釈は許さない。以後の profile/map/framing 失敗は厳密に内部 `unsupported source theorem canonical components` とし、変更・部分結果・公開診断なしに不可分に返す。
caller は真正 checked-lowered Core と、異なる実 local guard/goal root 二つだけに対応する未変更の同一 source [Core theorem formula payload](../../mizar-core/ja/elaborator.md#source-theorem-formula-payloads) を渡し、値はともに非空とする。同じ opaque value は許し、byte provenance は caller の責任とする。replay が認証するのは同じ Core と VC の構造的対応であり、Checker seal・供給 bytes・現在性ではない。内容の解析・再構成・Debug 符号化は行わない。
実 local entry は一つの ProofAssumption/Core guard、実 Core goal、ちょうどその一つの LocalContext premise、空 policy inputs、hint 無しを要求する。実 sort_key は `core-context-formula-{guard.index():08}-source-ordinal-00000000`、provenance は key `context-formula-{guard.index()}` の CoreHandoff 一つ、その Some CoreProvenance は Checker phase と実 sole item の `checker/theorem/{actual item.symbol.fqn()}/skeleton/terminal` key に限る。検証した ordinal zero と provenance だけを下記 role に正規化し、他 provenance は拒否する。raw FQN は contribution/structural-path/slot を含むため、対応の検証だけに使い canonical owner 内容にはしない。
未変更の `source_registration_frame` と `stable_fingerprint_hash` を再利用し、framing 拒否を本 profile error へ変換する。F(b) は checked BEu32 byte 長と未変更 bytes、以下の u32 はすべて BEu32 とする。x = vc, goal, context の payload は UTF-8 `mizar-vc/source-theorem-canonical-{x}/v1` と zero、F(実 VC schema UTF-8)、F(実 generation schema UTF-8) で始める。domain は順に `mizar-vc-source-theorem-canonical-vc`、`mizar-vc-source-theorem-canonical-goal`、`mizar-vc-source-theorem-canonical-context`。
entry 内容 E は正規化 source ordinal の u32(0)、ProofAssumption の byte1、F(実 guard bytes)、provenance 数 u32(1)、CoreHandoff phase の byte1、検証済み key role を正規化した F(UTF-8 `context-formula`)、Some core provenance の byte1、Checker phase の byte1、検証済み Checker role を正規化した F(UTF-8 `source-theorem-local-proof/terminal`) の順。E は content frame だけで、新表現ではない。
goal suffix は F(実 goal bytes)。context suffix は entry 数 u32(1)、F(E)、policy-input 数 u32(0)。VC suffix は TheoremProofStep kind の byte1、F(実 goal bytes)、premise 数 u32(1)、LocalContext premise の byte1、実解決 entry の F(E)、policy-input 数 u32(0)、hint 無しの byte0。数と role は実検証内容を符号化し、非空 policy inputs は hashing 前に拒否する。
参加するのは以上の field だけで、schema は三つすべてに影響する。goal は Core payload の module/role/formula bytes 以外の owner/path/hint/context を含めず、context は検証した source-profile phase/role と正規化 guard ordinal を保つ。dense/source/snapshot/handoff/row/arena ID、range、binder 表示名、status、Debug 表現を除く。source syntax/anchor/dependency や Kernel/Artifact hash で代用しない。同 profile の owner 改名・binder 名・trivia・snapshot 変更は三つを保ち、source-shaped owner/syntax は別途未提供のまま。
既存 `crates/mizar-test/src/runner/proof_verification.rs` の `term_proof_tests` 内へ `source_theorem_canonical_components_*` family を厳密に二つ末尾追加し、全既存 bytes と最後の module 閉じ括弧を保つ。`with_step5c14_theorem_check`、`lower_source_theorem_skeletons`、`source_theorem_formula_payloads`、`generate_core_vcs` を再利用し helper/extraction は変えない。producer 正例は真正 source pipeline・実 Core guard/goal map・元 VC graph に限り、実入力由来の malformed container/map は拒否例専用とする。
第一 family は本 grammar・実 module/schema・未変更 hash primitive から Rust 前に作成し review した独立 frozen-format literal VC/goal/context 期待値を扱い、artifact は doc/ 外に置く。元 T、同 owner の異なる bound/local binder 名、trivia、真正に改名した owner U を比較し、決定性・入力 Core/map/VcSet 不変・旧 generic fingerprint の None・元の不完全 anchor の保持を要求する。
真正 snapshot 変更は三つを保ち、別 VC/generation schema は既存 CoreGenerationCandidateSet/intake/normalization を通して真正に再生成し三つすべてを変える。必要な既存 setup は inline とし、旧 container の relabel は producer 正例にしない。allocator relocation は実 source producer が提供する場合だけ主張し、架空の Accepted/Hit/Complete や proofless Open の成功 lowering は作らない。
第二 family は実 root による空/missing/empty-value/extra map と original-bound-root 置換、存在しない VcId、真正 foreign 同 profile Core/VC 対応、graph 全体の status/context/provenance/sort/policy/premise/hint drift を拒否する。明示的に malformed/source-unbacked な Core 拒否例を一つ用意し、valid source entry を保って sole seed の Checker provenance key だけを変え、generic VC graph を実際に再生成する。VC69 replay の成功と本 API 後段の profile 拒否を要求する。元 Core56 bytes は拒否例の operand 専用で変更 Core に認証済みとはせず、source seal・真正 producer・受理を主張せず lowerer/helper も変えない。absent/foreign 入力かつ空 map で最初の error を実 handoff replay と直接比較し、wrapper・検証順変更を検出する。全 case は指定 API/oracle に到達し、Core56/VC69 の広い既存 matrix は複製せず再利用する。
既存 generic fingerprint/reuse、元 anchor/incomplete marker、conservative dependency、source owner/syntax producer、同一呼出し Proof identity/classification/witness、freshness/currentness/cache/publication、広い Core/Theorem/ATP profile、Steps 6–8 は未完了のまま。この raw component は完全 identity や trusted producer receipt ではなく、Checker/Core/Kernel/Proof の所有境界と proof acceptance は変えない。

## Source theorem candidate anchor hash

`source_theorem_candidate_anchor_hash(owner: &[u8], syntax: &[u8], vc: CanonicalVcFingerprint, goal: Hash, context: LocalContextFingerprint) -> Result<Hash, String>` は `crates/mizar-vc/src/vc_ir.rs` に追加する free function で、opaque な raw ingredient の hashing だけを所有する。
caller は真正な同一 profile の変更しない owner/syntax bytes と canonical components を渡す。本関数は graph replay、owner/syntax grammar 検証、producer/profile/bytes 来歴/現在性の認証を行わず、opaque bytes を解析・再符号化しない。
空 owner/syntax と checked framing overflow を厳密に内部 `unsupported source theorem candidate anchor` で拒否し、部分結果や変更を返さない。未変更 source_registration_frame と stable_fingerprint_hash を使い、新 hash primitive は作らない。
F(b) は checked big-endian u32 byte 長と未変更 bytes。payload は UTF-8 `mizar-vc/source-theorem-candidate-anchor/v1` と NUL、TheoremProofStep の byte1、F(UTF-8 `source-theorem-local-proof/terminal`)、F(owner)、F(syntax)、vc.hash()・goal・context.hash() の各 raw 32 bytes をこの順に続ける。
domain は `mizar-vc-source-theorem-candidate-anchor`。固定 kind/terminal はこの版で caller が供給する ordinary singleton profile の記述であり、その成立を認証しない。別 role/profile は別途 review した版を要する。
schema は既に canonical components に参加する。独立した dense id・raw FQN・range・snapshot・VC status・policy・evidence・dependency field は追加せず、宣言 status は opaque owner bytes 内に保つ。元の構造 anchor・不完全 marker・汎用 fingerprint・source/dependency completeness・cache-miss status を保つ。
結果に trust・completeness・acceptance・reuse・publication 権限は無い。[Proof assembly](../../mizar-proof/ja/status.md#source-theorem-original-obligation-identity) が真正 source 対応、owner JSON、obligation 合成、共有 test design を所有する。

## Source algorithm return canonical components

`VcSet::source_algorithm_return_canonical_components(&self, vc: VcId, core: &mizar_core::core_ir::CoreIr, generation_schema: &GenerationSchemaVersion, formulas: &BTreeMap<CoreFormulaId, Vec<u8>>) -> Result<(CanonicalVcFingerprint, Hash, LocalContextFingerprint), String>` は未信頼の VC、goal、local-context content をこの順で返す。既存 canonical_vc_fingerprint の完全な関数 marker の前へ inline method 一つを挿入し、表現や export は加えない。
最初に未変更 [generate_source_algorithm_postconditions](./generator.md#bounded-source-return-postconditions)(core, self.snapshot(), generation_schema, self.schema_version()) を呼び、target/map 検査より先に全 String を保持し、再生成 VcSet 全体の一致を要求する。後段 profile/対応/正規化/framing 拒否は内部 `unsupported source algorithm return canonical components` とし、変更・diagnostic・部分結果を返さない。
検証済み Core を実 parameter/result 一組、Return(Some term) 文一つ、ensures 一つに限定する。指定 id が選ぶ Open AlgorithmPostcondition VC 一つ、generated AlgorithmPostcondition Equals goal 一つ、実 parameter-guard の Core を持つ CheckerFact context entry 一つ、その LocalContext premise 一つ、hint なしを要求する。zero-VC/state/assertion/loop/Pick/他 profile は正例ではない。既存検証と accounting は全体 replay が所有し、別 lowerer/general validator を加えない。
context sort_key `algorithm-parameter-type-0000`、key `algorithm-parameter-type` の CoreHandoff provenance 一つ、その Some Core provenance が実 parameter-guard source provenance と一致することを要求する。上流 generator が唯一の Checker phase と数値 algorithm/source-node key を検査する。caller は当該 nonempty guard root だけの真正な未変更 [Core algorithm formula bytes](../../mizar-core/ja/elaborator.md#source-algorithm-formula-payloads) を渡し、bytes 出自は caller が所有する。構造 replay は source/check seal、供給 bytes の認証、所有者/currentness/lifetime 権限ではない。
検証済み role と Term sort で実 parameter を Free、result を Generated とした BinderContext を作る。実置換済み goal の両 operand に既存 [normalize_core_term/canonical_term](../../mizar-core/ja/binder_normalization.md) を実 Core row/source で呼び、唯一 parameter の CanonicalVar::Free を要求する。goal tree G は Equals の byte1、続けて実 operand ごとに FreeParameter の byte2 と BEu32(0)。元 ensures bytes、数値 term/variable id、syntax、Debug、whole-Core/hash placeholder で goal を代用しない。
未変更 source_registration_frame と stable_fingerprint_hash を使う。F(b) は checked BEu32 byte 長と未変更 bytes で、以下の count は全て BEu32。x = vc, goal, context の payload は UTF-8 `mizar-vc/source-algorithm-return-canonical-{x}/v1` と NUL、F(実 VC-schema UTF-8)、F(指定 generation-schema UTF-8) から始め、domain は `mizar-vc-source-algorithm-return-canonical-{x}`。framing error は当該 profile 拒否へ写す。
entry E は検証済み唯一 parameter ordinal の u32(0)、CheckerFact の byte1、F(実 guard bytes)、provenance count u32(1)、CoreHandoff の byte1、F(UTF-8 `algorithm-parameter-type`)、Some core provenance の byte1、Checker の byte1、検証済み Checker role を正規化した F(UTF-8 `source-algorithm-return/parameter-type`)。局所 byte frame であり新 typed representation ではない。
policy input は順に flow, site-kind, ordinal, statement, block, exit の六件だけで、sort_key は `{index:04}-algorithm-site-{name}`、key は `algorithm-site-{name}`、value は 0, Ensures, 0, 0, 0, 0。この検証済み零座標は唯一 algorithm/return/site の role を示す。P は u32(6) と各未変更 input の F(実 sort_key)、F(実 key)、F(実 value)。別座標へ一般化したり policy を捨てたりしない。
goal suffix は F(G)。context suffix は entry count u32(1)、F(E)、P。VC suffix は AlgorithmPostcondition の byte1、F(G)、premise count u32(1)、LocalContext の byte1、解決済み実 entry の F(E)、P、hint なしの byte0。参加するのはこれらのみで両 schema は三成分全てへ参加する。裸の Core/VC/source/snapshot id、range、Debug を除き、実 opaque Core owner/binder bytes と検証済み policy 文字列を保つ。
同一 owner の binder 表記、trivia、真正に再生成した snapshot の変更は成分を保つ。Core58 guard bytes には module/item owner data が含まれるため未変更で保ち、owner 変更は context/VC に参加する。この profile の正規化 parameter 等式 goal は同じまま。owner prefix の解析、theorem と同じ owner 中立性、source-shape/dependency identity は約束しない。汎用 fingerprint None、既存不完全 anchor、metadata-only termination accounting と全 proof/cache policy を保つ。
完全 source/context/provenance/footprints、広い algorithm profile、E2 mixed group、header terminating/promotion、fresh acceptance、Artifact/witness、cache/reuse/Hit、publication/service/LSP/Steps6–8 は別途。Kernel/ATP 実行、架空の正 Core/check/VC/receipt/Accepted/Complete、新 lower producer/type/field/helper/alias/dependency/policy は加えない。

### Source algorithm return component tests

既存 mizar-test runner/proof_verification.rs の最終 term_proof_tests brace 前へ `source_algorithm_return_canonical_components_preserve_genuine_content` と `source_algorithm_return_canonical_components_preserve_refusals` のみ追記する。source_registration_inputs、step5c8_test_frontend、check_source_algorithm_types、lower_source_algorithms、Core58 guard bytes、実 generator を使う。未変更 [return/ensures](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) source/sidecar/snapshot、neutral binder/trivia と owner edit を使い、新 fn/corpus 変更/架空の正入力を加えない。
method より前に実 checked Core/guard map/raw graph を得る。Rust 前に doc 外へ置く独立 grammar vector は default、各 schema 変更、異常/空文字の literal goal hash を持ち、真正な未変更 guard bytes から VC/context を手動 frame してtest 内の独立した既存 hash primitive closure 一つを VC/goal/context の実三か所で呼ぶ。全 tuple、決定性、入力純粋性、正確な context/policy/Open/Deferred accounting、neutral edit、owner 参加、真正 snapshot/schema 再生成を検査する。古い container の relabel 正例や不変基準値だけの重複 assertion は不要。
存在しない target、missing/empty/extra/wrong-root map、代表的な全 graph context/provenance drift、実再生成 graph が異なる真正 foreign Core、真正 zero-VC と広い raw profile 一つを拒否する。比較前に実 old generator 結果を得る。真正 two-return Core の generator String に absent target/empty map を併せても上流 error を最初に保つ。map bytes は opaque/未信頼のまま。旧 lower grammar/decoder matrix を再利用し、到達不能 framing 幅 overflow は架空 producer/巨大 payload を作らず静的検査する。

## Source algorithm return candidate anchor hash

`source_algorithm_return_candidate_anchor_hash(owner: &[u8], syntax: &[u8], core: &[u8], vc: CanonicalVcFingerprint, goal: Hash, context: LocalContextFingerprint) -> Result<Hash, String>` は vc_ir.rs の既存 theorem-candidate 全 doccomment/function の前に置く単一 inline free function とする。opaque な呼出側データを hash するだけで source/受理を認証しない。
owner/syntax/core が空、長さ変換・checked 加算が不能、payload 全体が32MiB超の場合は内部 unsupported source algorithm return candidate anchor を返す。prefix/discriminator/四 u32 frame/末尾 hash96byte を含む全長を extension 前に検査し、変更や診断なしに atomic に拒否する。
payload は UTF-8 mizar-vc/source-algorithm-return-candidate-anchor/v1 と NUL、限定 return-postcondition profile の byte1、続いて四 F をこの順に置く: UTF-8 source-algorithm-return/postcondition、owner bytes、syntax bytes、core bytes。F は既存 source_registration_frame の u32 big-endian byte 長と元 bytes。最後に vc.hash()/goal/context.hash() の全32byte をこの順に追加する。
既存 stable_fingerprint_hash を domain mizar-vc-source-algorithm-return-candidate-anchor と payload 全体で呼ぶ。helper/type/representation/codec/dependency、source 検証、JSON parse、owner 正規化、下位 profile 変更を追加しない。candidate 構築は VC、実 source operands は同一 task の [Proof consumer](../../mizar-proof/ja/status.md#source-algorithm-return-obligation-identity) が所有する。
raw 結果は完全な SourceShapeHash/ObligationAnchor、provenance/currentness、proof/E4/termination、完全 dependencies、trusted witness/Artifact/publication/cache/reuse/Hit/services/LSP/Steps6–8 の権限ではない。呼出側 bytes や hash 一致だけでは何も認証しない。
既存 runner/proof_verification.rs の実 source identity 三 family が Rust 前に独立固定した candidate 全 grammar/hash、実 header/parameter のみの syntax 変化、owner/module の参加、empty operands、反復性・入力不変性を検査する。既存 framing-width/static review と下位 matrix を再利用し fixture や架空 accepted producer を追加しない。
