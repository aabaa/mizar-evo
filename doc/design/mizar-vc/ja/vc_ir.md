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
- symbolic `PremiseRef`: 後続 ATP translation が encode または prune できる参照;
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
| VCIR-G003 | `external_dependency_gap` | `mizar-atp`、`mizar-kernel`、`mizar-proof`、`mizar-cache` は active workspace consumer ではない。 | prover-independent IR、untrusted deterministic evidence reference、status state、reuse-candidate data のみを仕様化する。 |
| VCIR-G004 | `test_gap` | task 3 より前には `vc_ir` Rust data shape や tests がない。 | 下記の planned tests を task 3 と task 8 の義務にする。 |
| VCIR-G005 | `source_drift` / `test_gap` | task 8 後には final `VcSet` normalization が存在するが、verifier policy を `NeedsAtp`、`PolicyOpen`、`AssumedByPolicy` status に投影する deterministic API や focused test suite はない。 | Task 9 は immutable `VcSet` data 上の status-policy projection と、status change が context、proof hint、anchor、seed accounting、ATP-bound obligation を保持することの test を追加する。 |
| VCIR-G006 | `deferred` | Discharge evidence、dependency slice、ATP translation、kernel/proof/cache/corpus consumer、source-derived corpus runner activation は後続 task または external seam である。 | Task 9 は discharge evidence 生成、dependency slice 計算、ATP problem translation、kernel/proof/cache/corpus path 呼び出し、downstream integration record の fabricate をしてはならない。 |

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
- loop invariant entry、preservation、break、continue、exit obligation;
- range-loop positive-step、range-bound、hidden-index obligation;
- collection-loop finiteness と order-independence obligation;
- termination と partial-termination obligation;
- ghost-erasure safety;
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
