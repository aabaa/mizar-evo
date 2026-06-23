# Core IR

> 正本は英語です。英語版:
> [../en/core_ir.md](../en/core_ir.md)。

## 目的

`CoreIr` は `mizar-core` phase 9 が生成する backend-neutral な論理表現である。
`ResolvedTypedAst` の後に現れる最初の non-source-shaped semantic layer であり、
term、formula、proof skeleton、definition、algorithm shell、provenance、
obligation seed を、後続の control-flow preparation、VC generation、
deterministic discharge、ATP translation、kernel checking に向けて正規化する。

この task-2 document は data shape と invariant のみを仕様化する。lowering、
`ControlFlowIr`、VC generation、artifact schema、proof acceptance、kernel replay は
実装しない。

## references

- [architecture 01](../../architecture/ja/01.ir_layers.md) は `CoreIr` と
  `ControlFlowIr` を immutable internal IR layer として定義する。
- [architecture 06](../../architecture/ja/06.elaboration_and_core_ir.md) は
  phase-9 elaboration の責務と初期 `CoreIr` interface shape を定義する。
- [architecture 07](../../architecture/ja/07.vc_generation.md) は obligation seed が
  後続で concrete `VcId` になる方法を定義する。
- [architecture 16](../../architecture/ja/16.substitution_and_binding.md) は binder
  identity、alpha-equivalence、substitution replay を定義する。
- [architecture 22](../../architecture/ja/22.incremental_verification_contract.md) は
  snapshot-local `VcId` と cross-edit `ObligationAnchor` candidate の分離を定義する。
- [checker resolved typed AST](../../mizar-checker/ja/resolved_typed_ast.md) は
  elaboration への source-shaped semantic input を定義する。
- [spec 03](../../../spec/ja/03.type_system.md)、[spec 13](../../../spec/ja/13.term_expression.md)、
  [spec 14](../../../spec/ja/14.formulas.md)、[spec 16](../../../spec/ja/16.theorems_and_proofs.md)、
  [spec 20](../../../spec/ja/20.algorithm_and_verification.md) は core node が表現する
  language behavior を定義する。

## 責務

`core_ir` が所有するもの:

- core item、term、formula、definition、proof tree、algorithm shell、
  generated-origin record、obligation seed の dense id と table。
- name、type、registration、overload、inserted-view decision が final になった後の
  backend-neutral な term/formula node shape。
- eager inline を伴わない stable definition expansion boundary。
- proof search を行わず、thesis transition、assumption、citation、terminal goal を
  記録する proof-skeleton data。
- CFG を構築せず、lowered contract、ghost/runtime classification、source-shaped
  statement order、phase-10 handoff material を保持する algorithm-shell data。
- diagnostic、obligation、artifact-facing node すべての source/core provenance。
- task-local test、snapshot、audit が使う deterministic debug rendering。

`core_ir` が所有しないもの:

- source walking や source-to-checker payload extraction。
- name/type/registration/cluster/overload/proof search。
- capture-avoiding substitution algorithm や alpha-equivalence decision。
  それらは `binder_normalization.md` が仕様化する。
- `ControlFlowIr` construction、use-before-assignment analysis、unreachable diagnostic。
- concrete `VcId` assignment、`VcIr`、ATP encoding、proof certificate、verified
  artifact schema、cache record、public diagnostic code-space。

## data model

field が upstream/downstream id を明示しない限り、すべての id は 1 つの `CoreIr`
snapshot 内で dense である。dense id は、この document と lowering spec が定義する
insertion order によって決定的である。public enum は task 21 が explicit exhaustive
exception を記録しない限り forward-compatible とする。

以下の Rust 名は概念と関係について normative だが、task 3 は invariant を保つ範囲で
実装しやすい concrete field を選んでよい。

```rust
struct CoreIr {
    module_id: ModuleId,
    source_id: SourceId,
    items: CoreItemTable,
    terms: CoreTermTable,
    formulas: CoreFormulaTable,
    definitions: CoreDefinitionTable,
    proofs: CoreProofTable,
    proof_nodes: CoreProofNodeTable,
    algorithms: CoreAlgorithmTable,
    algorithm_statements: CoreAlgorithmStmtTable,
    generated: GeneratedOriginTable,
    obligation_seeds: ObligationSeedTable,
    source_map: CoreSourceMap,
    diagnostics: CoreDiagnosticTable,
}
```

root `CoreIr` は構築後 immutable である。builder は存在してよいが、immutable value を
生成する前に reference を validate しなければならない。

### core item

`CoreItem` は module-level logical boundary である。後続 phase が参照し得る accepted
または partially lowered declaration は、それぞれ item row を 1 つだけ持つ。

```rust
struct CoreItem {
    id: CoreItemId,
    symbol: SymbolId,
    kind: CoreItemKind,
    visibility: CoreVisibility,
    status: CoreItemStatus,
    dependencies: Vec<CoreItemId>,
    source: CoreSourceRef,
    diagnostics: Vec<CoreDiagnosticId>,
}
```

`CoreItemKind` は少なくとも structure、mode、attribute、predicate、functor、
theorem、lemma、instantiation metadata が concrete になった scheme/template、
registration、reduction、local generated definition、algorithm を含む。

`CoreItemStatus` は valid、skipped、partial、error-preserving item を区別する。
skipped / partial item は source map と diagnostic を保持してよいが、verified premise を
downstream phase に提供してはならない。

ordering:

- module item は dependency summary 読み込み後の canonical source order に従う。
- generated item は owning item、generated-origin kind、local path、normalized key で
  並べる。
- skipped/error item は diagnostic を安定させるため traversal order に残る。

### core term

`CoreTerm` は overload と inserted-view decision が final になった後の logical term を
表す。

```rust
struct CoreTermNode {
    id: CoreTermId,
    kind: CoreTermKind,
    source: CoreSourceRef,
}

enum CoreTermKind {
    Var(CoreVarId),
    Const(SymbolId),
    Apply { functor: SymbolId, args: Vec<CoreTermId> },
    Select { selector: SymbolId, base: CoreTermId },
    Tuple(Vec<CoreTermId>),
    SetEnum(Vec<CoreTermId>),
    Generated { origin: GeneratedOriginId, args: Vec<CoreTermId> },
    Error(CoreDiagnosticId),
}
```

規則:

- `Var` は display name ではなく canonical core variable id を使う。
- `Apply` の functor と `Const` の symbol は canonical `SymbolId`。
- `the T` のような stable choice term は、functor が generated `choice_T` symbol、
  argument が captured free parameter である通常の `Apply` node として表現する。
  対応する `GeneratedOrigin` record が generated symbol key と evidence を所有する。
  core には magic choice-node semantics はなく、stable choice term に
  `CoreTermKind::Generated` を使ってはならない。
- Fraenkel comprehension は captured free parameter、generated-origin record 内の
  sethood evidence、生成された集合に対する definitional membership-axiom obligation seed
  を持つ generated set-valued term へ lower される。
- source-written / inserted `qua` view は underlying term を変えない。その evidence は
  provenance、explicit predicate、obligation seed で表現する。
- `Error` term は first-class recovery node であり、valid logical term として
  受理してはならない。

### core formula

`CoreFormula` は logical proposition と type predicate を表す。

```rust
struct CoreFormulaNode {
    id: CoreFormulaId,
    kind: CoreFormulaKind,
    source: CoreSourceRef,
}

enum CoreFormulaKind {
    True,
    False,
    Atom { predicate: SymbolId, args: Vec<CoreTermId> },
    Equals { left: CoreTermId, right: CoreTermId },
    TypePred { subject: CoreTermId, ty: CoreTypePredicate },
    Not(CoreFormulaId),
    And(Vec<CoreFormulaId>),
    Or(Vec<CoreFormulaId>),
    Implies { premise: CoreFormulaId, conclusion: CoreFormulaId },
    Iff { left: CoreFormulaId, right: CoreFormulaId },
    Forall { binders: Vec<CoreBinder>, body: CoreFormulaId },
    Exists { binders: Vec<CoreBinder>, body: CoreFormulaId },
    Error(CoreDiagnosticId),
}
```

規則:

- type erasure は必ず explicit `TypePred` formula、local assumption、view
  provenance、diagnostic、または obligation seed を残す。
- type erasure により生成される conjunction は stable predicate ordering を使う。
- quantifier binder は `binder_normalization.md` と互換な `CoreBinder` row で表現する。
- formula node は surface precedence、parentheses、notation spelling を保持しない。

### binder と variable

```rust
struct CoreBinder {
    var: CoreVarId,
    role: CoreVarRole,
    ty_guard: Option<CoreFormulaId>,
    source_name: Option<String>,
    source: CoreSourceRef,
}
```

`source_name` は diagnostic-only である。semantic equality、hashing、substitution、
normalization は `CoreVarId` と task 4 が選ぶ canonical binder representation を使う。
free-variable set と substitution side condition は binder module が表現するが、core
node はその module が source syntax を見ずに実行できるだけの binder/source provenance を
保存しなければならない。

### definition と expansion boundary

`CoreDefinitionTable` は後続 phase が unfold できる各 definition の semantic boundary を
記録する。

```rust
struct CoreDefinition {
    id: CoreDefinitionId,
    item: CoreItemId,
    symbol: SymbolId,
    params: Vec<CoreBinder>,
    body: DefinitionBody,
    expansion: ExpansionPolicy,
    correctness: Vec<ObligationSeedId>,
    generated_dependencies: Vec<GeneratedOriginId>,
    source: CoreSourceRef,
}
```

`DefinitionBody` は term definiens、formula equivalence、guarded definiens branch、
algorithm-backed computable body、unavailable/error body を区別する。

`ExpansionPolicy` は少なくとも opaque、transparent、reducible、computable policy を
含む。この policy は後続 phase に unfold/reduce を許可するが、elaboration 中の eager
inline を強制しない。

guarded definition は branch order を保存する。overlap、coverage、compatibility、
coherence、existence、uniqueness、reducibility check は obligation seed として表現し、
accepted proof result として扱わない。

### proof table

`CoreProofTable` は proof skeleton を記録し、proof acceptance は記録しない。

```rust
struct CoreProof {
    id: CoreProofId,
    item: CoreItemId,
    proposition: CoreFormulaId,
    root: CoreProofNodeId,
    status: CoreProofStatus,
    source: CoreSourceRef,
}

enum CoreProofStatus {
    Open,
    Assumed,
    Conditional,
    Error,
}

enum CoreProofNodeKind {
    IntroduceBinder { binder: CoreBinder, child: CoreProofNodeId },
    Assume { label: Option<CoreLabelRef>, formula: CoreFormulaId, child: CoreProofNodeId },
    Step { label: Option<CoreLabelRef>, formula: CoreFormulaId, justification: CoreJustification },
    CurrentGoal { thesis: CoreFormulaId, child: CoreProofNodeId },
    Sequence { children: Vec<CoreProofNodeId> },
    Branch { kind: ProofBranchKind, children: Vec<CoreProofNodeId> },
    TerminalGoal { obligation: ObligationSeedId, citations: Vec<CoreCitation> },
    Error(CoreDiagnosticId),
}
```

規則:

- `thesis` は現在の `CoreFormulaId` へ解決され、magic identifier としては保持しない。
  明示的な current-goal transition は `CurrentGoal` node で表す。
- `Sequence` node は順序付き proof block を保存し、1 つの proof path 上で label を伝播する。
  `Branch` child は sibling scope を分離する。
- citation は label、current module または dependency summary にある proof-like symbol
  （`Theorem`、`Lemma`、`Scheme`）、または generated origin への symbolic reference のまま
  保持する。core は ATP premise selection を決めず、functor や mode など proof でない local symbol は
  proof citation として valid ではない。raw `CoreIr` validation は item table に存在する local symbol の
  kind を検証する。external dependency-symbol citation の kind は Core IR 構築前の elaborator/context
  validation が保証し、この table set では symbolic なまま保持する。
- terminal goal は生成された theorem-proof obligation seed を参照し、durable な citation list を
  terminal proof node にも保存する。
- `open`、`assumed`、`conditional` status は policy input として保持する。core は proof
  を accept/reject しない。
- `error` は recovery status に限る。malformed proof skeleton input を記録するが、
  proof を accept/reject しない。
- terminal proof obligation はすべて `ObligationSeedId` を参照する。

### algorithm shell

`CoreAlgorithmTable` は phase-9 algorithm shell を保持する。これは `ControlFlowIr` では
ない。

```rust
struct CoreAlgorithm {
    id: CoreAlgorithmId,
    item: CoreItemId,
    symbol: SymbolId,
    params: Vec<CoreBinder>,
    result: Option<CoreBinder>,
    contracts: CoreContractSet,
    statements: Vec<CoreAlgorithmStmtId>,
    ghost_effects: Vec<GhostEffectKey>,
    source: CoreSourceRef,
    diagnostics: Vec<CoreDiagnosticId>,
}
```

algorithm shell が保持するもの:

- parameter / result binder。
- lowered `requires`、`ensures`、`assert`、`invariant`、`decreasing`
  formula/term。
- executable `the` occurrence から生成された `Pick` site。
- ghost/runtime classification。
- source statement order と phase 10 に必要な local path information。

basic block、control-flow edge、use-before-assignment fact、reachability diagnostic、
generated VC は含めない。

`CoreAlgorithmStmtTable` は `CoreAlgorithm.statements` が参照する source-ordered
statement-shell row を所有する。
直接列挙された statement と `If`、`While`、`Match` arm からネストして参照される
statement は、すべて containing `CoreAlgorithmId` と同じ `owner` を持たなければならない。
phase 10 は `ControlFlowIr` 構築時にこの owner 関係を信頼してよい。

```rust
struct CoreAlgorithmStmt {
    id: CoreAlgorithmStmtId,
    owner: CoreAlgorithmId,
    kind: CoreAlgorithmStmtKind,
    source: CoreSourceRef,
    diagnostics: Vec<CoreDiagnosticId>,
}

enum CoreAlgorithmStmtKind {
    Let { binder: CoreBinder, value: Option<CoreTermId>, ghost: bool },
    Assign { target: CorePlace, value: CoreTermId },
    Assert { formula: CoreFormulaId },
    If { condition: CoreFormulaId, then_body: Vec<CoreAlgorithmStmtId>, else_body: Vec<CoreAlgorithmStmtId> },
    While { condition: CoreFormulaId, invariants: Vec<CoreFormulaId>, decreasing: Vec<CoreTermId>, body: Vec<CoreAlgorithmStmtId> },
    Match { scrutinee: CoreTermId, arms: Vec<CoreAlgorithmMatchArm> },
    Return(Option<CoreTermId>),
    Break,
    Continue,
    Pick { binder: CoreBinder, witness_ty: Option<CoreFormulaId>, ghost: bool },
    Error(CoreDiagnosticId),
}
```

task 3 は一部 variant を task 13 / task 15 が behavior を追加するまで minimal にしてよいが、
owning table、source ref、deterministic ordering、statement reference validation は
提供しなければならない。`CoreAlgorithmStmt` row は shell であり、CFG block id は
エンコードしない。

## generated origin

generated term と internal symbol は `GeneratedOriginTable` で追跡する。

```rust
struct GeneratedOrigin {
    id: GeneratedOriginId,
    owner: CoreItemId,
    kind: GeneratedOriginKind,
    key: GeneratedOriginKey,
    functor: Option<SymbolId>,
    params: Vec<CoreVarId>,
    evidence: Vec<CoreProvenance>,
    source: CoreSourceRef,
}
```

kind には stable choice、Fraenkel comprehension、local abbreviation expansion entry、
generated type predicate、skipped/error placeholder、その他の generated bookkeeping record を含める。
実行可能 algorithm の `Pick` binding は generated origin ではなく、statement-local な
`CoreAlgorithmStmtKind::Pick` row である。
`GeneratedOriginKind::AlgorithmPick` は将来の非実行 algorithm bookkeeping 用の予約 variant であり、
task 13 の shell lowering は emit しない。

generated key は normalized semantic origin と alpha-normalized payload を使う。
source display name を identity に使ってはならない。owning module spec が stable
artifact projection を後で定めない限り、generated name は diagnostic-only である。
internal symbol に対応する generated origin は generated functor symbol を `functor` に記録する。
term 以外の bookkeeping を表す origin では省略してよい。

## obligation seed

`ObligationSeed` は `mizar-vc` が消費する phase-9/10 handoff unit である。これは
`VcId` でも proof evidence でもなく、それ自体では `ObligationAnchor` でもない。

```rust
struct ObligationSeed {
    id: ObligationSeedId,
    owner: CoreItemId,
    kind: ObligationSeedKind,
    goal: Option<CoreFormulaId>,
    context: Vec<CoreFormulaId>,
    local_path: LocalProofOrProgramPath,
    label: Option<CoreLabelRef>,
    semantic_origin: NormalizedSemanticOrigin,
    provenance: Vec<CoreProvenance>,
    source: CoreSourceRef,
    core_refs: Vec<CoreNodeRef>,
    status: ObligationSeedStatus,
    diagnostics: Vec<CoreDiagnosticId>,
}
```

goal invariant:

- `status = Active` は `goal = Some(_)` を要求する。
- `status = Skipped`、`Deferred`、`Error` は、invalid/skipped item、external
  dependency gap、failed lowering site の traceability のために seed を保存する場合に限り
  `goal = None` を使ってよい。
- `goal = None` の seed は diagnostic または provenance reason を持たなければならず、
  concrete VC に変換してはならない。
- seed kind が将来複数の VC に展開される場合でも、seed は aggregate normalized goal
  または generated conjunction goal を記録する。分割は `mizar-vc` の責務である。

seed kind は少なくとも以下を含む:

- theorem / lemma proof terminal goal。
- definition existence、uniqueness、coherence、compatibility、coverage、
  overlap consistency、reducibility correctness。
- type/coercion checking から持ち越す checker-initial obligation。
- generated choice/comprehension term の non-emptiness / sethood obligation。
- generated Fraenkel comprehension set の definitional membership axiom。
- algorithm precondition、postcondition、assertion、invariant、termination measure、
  ghost-erasure safety、および task 18 後の phase-10 flow-derived check。

`local_path` は anchor-ready でなければならない:

- proof path は proof block、branch、step、terminal-goal position を記録する。
- program path は algorithm statement、branch、loop、contract、generated obligation
  position を記録する。
- generated path は owner item、generated-origin kind、normalized key を記録する。

`semantic_origin` は normalized であり、source display spelling に依存しない。
seed を生んだ theorem、definition、registration、generated symbol、algorithm、
checker-origin row を識別する。

`provenance` は `mizar-vc` が消費する source/core 情報を含まなければならない:

- 利用可能な source range と resolved/checker id。
- seed に関わる core item/term/formula/proof/algorithm reference。
- 利用可能な label / citation hint。
- generated material 由来なら generated-origin id。
- applicable な erasure/view/template/proof-skeleton provenance。

seed は owner item、source range、local path、kind、label、normalized semantic origin、
dense id tie-breaker により決定的に並ぶ。

### obligation seed handoff

Task 18 は、`mizar-vc` の seed intake が消費する `ObligationSeedHandoff` view を公開する。
handoff は引き続き core が所有する phase-9/10 metadata である。`VcId` の割り当て、
`ObligationAnchor` の計算、`VcIr` の構築、context fingerprint、proof acceptance の判定は行わない。

handoff は snapshot-local な独立 id 空間を持つ。

```rust
struct ObligationSeedHandoff {
    entries: ObligationHandoffTable,
    source_map: Map<ObligationHandoffId, CoreSourceRef>,
}

struct ObligationHandoffEntry {
    seed: ObligationSeed,
    origin: ObligationHandoffOrigin,
    flow_site: Option<ControlFlowObligationSite>,
}

enum ObligationHandoffOrigin {
    ExistingCore { seed: ObligationSeedId },
    FlowDerived { flow: ControlFlowId, algorithm: CoreAlgorithmId },
}
```

`ObligationHandoffId` はこの handoff value に局所的である。`ObligationSeedId` でも
`VcId` でもない。handoff source map は `ObligationHandoffId` を key にする。既存 core seed は、
`ObligationHandoffOrigin::ExistingCore` を通じて元の `CoreSourceMap.obligation_sources` entry も保持する。

`ControlFlowObligationSite` は、control-flow id を `CoreNodeRef` に埋め込まずに CFG-local site を
識別する。site class と、関連する flow-local index を記録する。たとえば contract-site ordinal、
assertion-site ordinal、loop-invariant-site ordinal、termination-measure-site ordinal、
partial-termination-site ordinal、`LocalId`、`AssignmentEffectId`、`LoopId`、`BasicBlockId`、
`ControlFlowExitId`、該当する場合の statement id である。

handoff は次を含む。

- 既存の `CoreIr.obligation_seeds` row すべて。canonical seed order で sort し、元の
  `ObligationSeedId` へ link する。
- `ControlFlowIr` の contract、termination、ghost-erasure site から導出した追加の
  phase-10 seed row。origin となる `ControlFlowId`、`CoreAlgorithmId`、局所 CFG site へ link する。
- handoff seed id 用の source map。これにより `mizar-vc` は raw syntax を見ずに各 seed を
  source へ trace できる。

既存 core seed は、元の `kind`、`status`、goal、context、local path、label、normalized
semantic origin、provenance、source、diagnostic、`CoreNodeRef` を保存する。これにより
theorem / lemma terminal goal、definition correctness、checker-initial obligation、
generated choice / comprehension obligation、elaboration 中に作られた deferred/error traceability row
を覆う。

flow 由来 seed は、明示的な `ControlFlowIr` metadata からのみ生成する。対象は entry
`requires`、return `ensures`、algorithm / statement assertion、loop invariant、
decreasing / partial termination site、ghost-only local / assignment site である。seed は
`CoreNodeRef::Item`、`CoreNodeRef::Algorithm`、存在する場合は formula または term、statement-owned
site では statement ref を含む。CFG-local id は handoff entry の flow-site metadata に残し、
`CoreNodeRef` には入れない。`CoreIr` が `ControlFlowIr` table id から独立である必要があるためである。

task 18 の flow 由来 seed は、formula goal を持つ場合でも `Deferred` とする。assertion /
invariant の具体的な VC context、`requires` の caller-side substitution、`ensures` の result
substitution、termination well-foundedness schema、ghost-erasure proof shape は `mizar-vc` の責務である。
deferred seed は、その obligation がすでに VC generation 可能であると見せかけずに、anchor-ready な
program path、source/core/CFG provenance、status を保存する。

handoff order は core seed と flow-derived seed を合わせて決定的である。各 seed の canonical key を
比較し、同順位の場合は origin class、flow id、site kind、local site index で tie-break する。
handoff id は handoff snapshot に局所的であり、`VcId` ではない。

## source map と provenance

diagnostic、obligation seed、snapshot line、artifact projection、後続 source-mapped
metadata を生成し得る core item と term/formula/proof/algorithm node は、必ず source
map entry を持つ。

```rust
struct CoreSourceMap {
    item_sources: Map<CoreItemId, CoreSourceRef>,
    term_sources: Map<CoreTermId, CoreSourceRef>,
    formula_sources: Map<CoreFormulaId, CoreSourceRef>,
    definition_sources: Map<CoreDefinitionId, CoreSourceRef>,
    proof_sources: Map<CoreProofNodeId, CoreSourceRef>,
    algorithm_sources: Map<CoreAlgorithmStmtId, CoreSourceRef>,
    generated_sources: Map<GeneratedOriginId, CoreSourceRef>,
    obligation_sources: Map<ObligationSeedId, CoreSourceRef>,
}
```

`CoreSourceRef` が含むもの:

- 利用可能な `SourceId` と source range。
- 利用可能な upstream `ResolvedTypedAst` node/expression/metadata id。
- 利用可能な originating symbol、label、checker row id。
- generated node 用の `GeneratedFrom` marker。
- sort 済み `CoreProvenance` entry list。

node が direct source range を持たない場合、`GeneratedFrom` が必須である。これは owning
source/core node、generated-origin kind、normalized key、reason を記録する。
`GeneratedFrom` marker が item owner を名指す場合、その `(owner, kind, key)` は
ちょうど 1 つの `GeneratedOrigin` row に対応しなければならない。`GeneratedOrigin` row は
owner item、kind、normalized key により一意である。stable choice term は引き続き通常の
`Apply(choice_T(...))` term へ lower し、`GeneratedOrigin` row は magic term node ではなく
generated symbol と evidence を記録する。

source map は debug extra ではなく必須 data である。task 3 の test は `CoreItem` から
到達可能なすべての node が direct source または `GeneratedFrom` を持つことを確認する。

## diagnostic と error node

core diagnostic は次のような boundary failure を分類する:

- `ResolvedTypedAst` からの unresolved / blocked semantic input。
- invalid type erasure または view evidence 欠落。
- source construct に対する unsupported lowering。
- malformed proof skeleton。
- malformed generated-origin / source-map data。
- algorithm shell lowering failure。

diagnostic は local structured record である。public diagnostic code allocation は
`mizar-diagnostics` に deferred。

local diagnostic table は最小限の deterministic shape を持つ:

```rust
struct CoreDiagnostic {
    id: CoreDiagnosticId,
    class: CoreDiagnosticClass,
    severity: CoreDiagnosticSeverity,
    recovery: CoreDiagnosticRecovery,
    message_key: CoreDiagnosticMessageKey,
    primary_source: CoreSourceRef,
    related: Vec<CoreSourceRef>,
    owner: Option<CoreNodeRef>,
}
```

`message_key` は test と debug rendering 用の crate-local stable key であり、public
diagnostic code ではない。diagnostic row は primary source range、owner node、class、
message key、dense id tie-breaker により並ぶ。related source ref は phase、source
range、provenance で sort する。error node は failed lowering site を説明する owner
node、primary source、または `GeneratedFrom` marker を持つ diagnostic row を参照
しなければならない。

Error node は invalid lowering site を保持するが、それを valid logical term/formula に
変えてはならない。downstream phase は `Error` node と skipped/partial item を
non-verified input として扱わなければならない。

## deterministic debug rendering

task 3 はこの document の data shape に対する deterministic debug renderer を実装する。
rendering は internal / test-facing であり、stable published artifact schema ではない。

rendering rules:

- table は dense id order で render する。
- symbol id、source id、label、local path、generated key は canonical textual form を
  使う。
- map は sorted key で render する。
- provenance list は phase、source range、semantic origin、dense id で render する。
- error / skipped node は明示的に render する。
- source display name は diagnostic 用に出してよいが、semantic equality や generated key
  はそれに依存してはならない。

## validation and test obligations

task 3 は Rust test で以下を追加する:

- すべての core table を持つ minimal `CoreIr` の構築。
- stable dense id と deterministic debug rendering。
- item、term、formula、definition、proof、algorithm、generated origin、obligation seed、
  source map 間の invalid reference の rejection。algorithm statement と diagnostic
  reference も含む。
- item から到達可能なすべての node が source または `GeneratedFrom` に mapping されること。
- `ObligationSeed` の ordering と local path、label、normalized semantic origin、
  source/core provenance、status の保持。
- active obligation seed は goal を要求し、goal を持たない skipped/deferred/error seed は
  diagnostic/provenance reason を持ち、VC に変換されないこと。
- error node が explicit かつ non-verified のまま残ること。

task 2 では `.miz` fixture は不要である。source-derived pass coverage は checker payload
extraction と mizar-test stage support が捏造なしで core lowering に入力できるように
なるまで deferred。

## public enum policy

task 21 は `core_ir` の public enum をすべて downstream forward-compatible API surface
として分類する。将来の semantic category を下流 crate の exhaustive match を壊さずに
追加できるよう、各 enum は `#[non_exhaustive]` を維持しなければならない。

| public enum | decision |
|---|---|
| `CoreItemKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoreItemStatus` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoreTermKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoreFormulaKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `DefinitionBody` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `DefinitionBranchBody` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ExpansionPolicy` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoreProofStatus` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoreProofNodeKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoreCitation` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ProofBranchKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoreAlgorithmStmtKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `GeneratedOriginKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ObligationSeedKind` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `ObligationSeedStatus` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoreSourceAnchor` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoreProvenancePhase` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoreDiagnosticClass` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoreDiagnosticSeverity` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoreDiagnosticRecovery` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoreNodeRef` | `#[non_exhaustive]` downstream forward-compatible surface。 |
| `CoreIrError` | `#[non_exhaustive]` downstream forward-compatible surface。 |

この module が所有する exhaustive public enum exception はない。現在の variant を意図的に
列挙する `mizar-core` 内部 match は exhaustive のままでよい。

## drift and gap classification

| ID | class | evidence | action |
|---|---|---|---|
| COREIR-G001 | `spec_gap` | task 2 前に `core_ir.md` は存在しなかった。 | 本 document が task 3 用 module-spec gap を閉じる。 |
| COREIR-G002 | `test_gap` | task 3 前に `core_ir` source と test は存在しない。 | task 3 が上記 data shape と Rust test を実装する。 |
| COREIR-G003 | `external_dependency_gap` | checker closeout は source-to-checker extraction と source-derived semantic pass fixture を deferred としている。 | task 3 は explicit Rust fixture を使う。source-derived `.miz` core fixture は payload seam まで deferred。 |
| COREIR-G004 | `external_dependency_gap` | `mizar-vc`、`mizar-kernel`、`mizar-proof` crate は workspace member ではない。 | seed / provenance shape のみを仕様化し、downstream consumer は実装しない。 |
| COREIR-G005 | `deferred` | published artifact schema と public diagnostic code allocation は後続 crate の責務。 | debug rendering は internal、diagnostic は local に保つ。 |

## forbidden behavior

`core_ir` implementation は以下をしてはならない:

- raw syntax を見ること、source-to-checker extraction を行うこと。
- name/type/registration/overload/proof search を実行すること。
- registration や cluster rule を activate すること。
- explicit expansion policy 外で definition を eager inline すること。
- `VcId` や cross-edit proof reuse identity を割り当てること。
- artifact schema、ATP encoding、proof certificate、cache record、public diagnostic code を
  emit すること。
- generated display name や source spelling を semantic identity として扱うこと。
