# STEP 5 ソース由来 VC family 分解

> 正本言語: 英語。英語正本:
> [../en/source_vc_decomposition.md](../en/source_vc_decomposition.md)。

本文書は mizar-vc Task 30 の確定成果である。Task 31 に対する Task-180
theorem obligation から VC への厳密な写像を固定し、その他すべてのソース由来
VC family を bounded follow-up task へ分解する。これは task、依存関係、consumer
の authority のみであり、言語意味論、Rust 動作、`.miz` source、expectation、
trace status、test list、coverage credit を変更しない。

## Authority と開始時 baseline

inventory は repository authority order に従う。

1. `doc/spec/en/`;
2. 既存 `.miz` source;
3. `tests/coverage/spec_trace.toml`;
4. 既存 expectation sidecar;
5. design document;
6. 非規範的 inventory evidence としての source。

各 task の行で引用する canonical Chapter 03-21 を適用する。確定済み upstream
task authority は checker Task 247 の
[payload-family decomposition](../../mizar-checker/en/payload_family_decomposition.md)
と Core Task 32 の
[source-family decomposition](../../mizar-core/en/source_family_decomposition.md)
である。Core Task 31 は唯一の厳密な Task-180 Core 例外である。Core Tasks
33-53、Gate A1/MC-G004、Gate S1 は利用可能 payload ではなく、未実装の依存関係
として維持する。

Task 30 は clean entry の behavioral oracle を維持する。active runner は parse
96、declaration 4、type elaboration 188、plan 403/368、type elaboration 236/224、
pass/fail 219/184、warnings/errors 23/0、`mizar-test` unit test 272、production
17 paths / 19,803 lines である。Core Task 32 が記録した CLI、test-list、
production path/content hash は不変でなければならない。Task 30 は trace
requirement を追加せず、既存 deferred ownership 文言だけを変更できる。

## Descendant task の共通 contract

以下の各行は 1 nonempty logical task / 1 commit である。編集前に exact canonical
section、source family、syntax-free Core/control-flow input、VC consumer、
visibility、generated formula/context 表現、禁止範囲、test、trace impact、exit
criteria を固定する。

各 source-derived VC task は、行で狭めない限り次を満たす。

- 検証済み `CoreIr`、`ControlFlowOutput`、obligation handoff を消費し、raw
  syntax を検査せず、checker/Core semantics を `mizar-vc` 内で再構築しない。
- source/module/owner identity、local path、semantic origin、seed status、source
  map、context order、versioned provenance を transactional に保存する。
- canonical obligation が既存 Core formula でない場合、完全で replayable な
  generated formula または substitution 表現を用いる。
- `VcId` は snapshot-local のまま、validation/normalization 後だけ dense かつ
  deterministic に割り当てる。
- anchor を正直に保つ。canonical formula、context、trace、dependency identity が
  unavailable なら明示的 incomplete かつ proof reuse 不可のままにする。
- 既存 intake policy に従い、各 input seed を zero/one/many VC のいずれかとして
  exactly once accounting し、不完全な deferred input を暗黙に open VC にしない。
- missing、duplicate、reordered、stale、recovered、diagnostic-bearing、cross-owner、
  cross-module、wrong kind/status/goal/path、partial context、corrupt provenance を
  fail closed にする。
- deterministic rerun、immutable whole-`VcSet` 比較、完全な
  `VcSet::debug_text()` snapshot、task-local corruption coverage を追加する。
- 新規に主張する VC family、kind、phase、semantic branch のすべてについて、実行可能な
  spec-derived real-source assertion を追加する。1つの最小 fixture で複数 claim を
  cover してよいのは、各 claim を個別に assert する場合だけである。
- 各行が主張する non-generation boundary ごとに最小 real-source zero-VC/near-miss
  case を、新規 diagnostic family ごとに最小 real-source negative case を追加する。
- producer、consumer、real baseline が同時に実装された時だけ narrow task-local
  trace row を active にする。broad proof-verification row は deferred のままにする。

すべての task で、proof search、別途認可されない deterministic discharge、
`NeedsAtp` policy 実行、ATP/kernel/proof/cache/artifact、theorem/algorithm acceptance、
fact publication、extraction/MVM、upstream evidence の捏造、expectation rebaseline、
Steps 6/7 promotion を禁止する。

## VC Task 31 の厳密な Task-180 mapping

Task 31 は narrow Core-aware adapter を所有する。厳密な Core Task-31 `CoreIr` を
borrow して検証し、Core data を変更せず、synthetic
`vc-proof-goal:terminal` provenance marker を挿入しない。

accepted input は public かつ structurally valid な theorem item 1件、`False`
formula 1件、`PendingAutomaticProof` 1件、direct `TerminalGoal` 1件、`proof/0`
の Active `TheoremProof` seed 1件だけを持つ。context、label、citation、diagnostic、
term、definition、algorithm、generated row は空である。source map、Core ref、
semantic origin、checker/proof-skeleton provenance は committed Core Task-31
contract と一致しなければならない。

Task 31 は空の `ControlFlowOutput` を構築・検証し、1件の
`ExistingCore { seed: 0 }` handoff/source-map row と、fresh に再計算した
`EligibleOneVc { goal: CoreFormulaId(0) }` intake row を検証する。marker text では
なく、認証済み direct proof-node-to-obligation-to-goal relation により candidate を
次のように分類する。

- seed handoff 0 を持つ dense `VcId(0)` ちょうど1件。
- `VcKind::TerminalProofGoal` と `VcStatus::Open`。
- owner `AnchorOwner::Theorem(CoreItemId(0))`。
- goal `VcFormulaRef::Core(CoreFormulaId(0))`。
- primary source、local path `proof/0`、semantic origin を保存。
- local context、premise、proof hint、related source list、generated formula table は空。
- 既存 Core-handoff、generator、normalization provenance chain。
- その VC に対応する Active/ExistingCore seed-accounting row 1件。

source-shape と empty-context anchor hash は available である。現在の VC payload
は Core formula id だけを transport するため canonical-goal hash は保守的に
unavailable のままで、anchor は
`Incomplete { missing: [CanonicalGoalHash] }` のままとする。この exact formula が
`False` と prevalidate 済みでも、hash 捏造や proof reuse の権限にはならない。

adapter は extra、missing、duplicate、reordered、stale、cross-owner、wrong
status/kind/goal/path/source/provenance/Core-reference、proof-link row をすべて reject
する。`PendingAutomaticProof` から `Open` は undischarged obligation の phase mapping
にすぎない。Task 31 は discharge、`NeedsAtp` transition、proof verification、
theorem acceptance、fact publication を行わない。

## 準備済み mizar-test consumer

これらは open mizar-test Task 10 内の increment であり、新規 top-level task では
ない。実 producer、source、sidecar、trace row、baseline が実行される前に coverage
を与えない。

| Increment | Stage、phase、artifact、ownership |
|---|---|
| `MT10-VC-T180` | VC Task 31 専用。distinct な `tests/miz/pass/theorems/pass_proof_verification_contradiction_formula_constant_001.miz` source と matching expectation sidecar を `proof_verification` / `active_proof_verification`、`expected_phase = "vc_generation"`、pipeline phase 11 で追加する。2つの complete `VcSet` と完全な `SnapshotKind::VcIr` / `VcSet::debug_text()` bytes を比較する。admission test は wrong stage、missing/duplicate/wrong active tag、wrong `expected_phase`、unchanged type-elaboration sidecar を reject する。既存 type-elaboration Task-180 source、sidecar、Core snapshot、outcome、trace backlink は不変。Task 31 は trace row をちょうど1件追加・active 化する。id `spec.en.mizar_vc.vc_ir.task180_proof_verification_snapshot`、source `doc/design/mizar-vc/en/source_vc_decomposition.md`、section `VC Task 31; exact Task-180 open VcIr proof-verification snapshot`、stage `proof_verification`、status `covered`、`required = true`、coverage `snapshot`、sole test backlink `tests/miz/pass/theorems/pass_proof_verification_contradiction_formula_constant_001.expect.toml` とする。Task 30 は row を追加しない。 |
| `MT10-VC-PV` | VC Tasks 32-55 の shared contract。各 task は最小 real source、sidecar、task-local trace row、full `VcIr` baseline（owned zero-VC boundary では empty `VcSet` を含む）、corruption/negative coverage を持つ distinct slice `MT10-VC-PV/VC<n>` を所有する。stage/tag は `proof_verification` / `active_proof_verification`、`expected_phase = "vc_generation"`、pipeline phase 11、artifact は complete deterministic `VcSet::debug_text()` bytes。VC 40 は VC 37/39 と Core 40/A1 の背後、VC 53 は bounded canonical-authority gap の背後で未実行のまま。missing scheme/theorem role は direct VC 41 の外で Gate S1 の背後に残る。 |

最初の runner/tag/guard 変更は Task 31 の最初の real baseline と同時に行う。empty
runner/snapshot infrastructure prerequisite は置かない。既存 parser-only theorem、
definition、registration、algorithm fixture を semantic VC baseline に再分類しない。

## 確定した definition/proof VC task graph

| VC task | Bounded family と canonical authority | 必須 Core dependency と consumer | Exit boundary |
|---|---|---|---|
| 32 | General theorem proof-step/terminal-goal VC、stable formula payload、quantified binder、ordered local context。Specs 04.5, 14, 15, 16.1-16.5/16.7。 | Core 33-35/37; `MT10-VC-PV/VC32`. | Task 180 は VC 31 のまま。proof search、closure、acceptance、verified premise publication なし。 |
| 33 | `equals` と `means` の両方に対する functor-definition correctness: result-type/type-correctness assertion、guarded-branch consistency、missing-`otherwise` coverage、および `means` だけの existence/uniqueness。Specs 10.3-10.6/10.12.2-10.12.6, 16.6.1/16.6.4。 | Core 33-36; `MT10-VC-PV/VC33`. | result-type assertion は accepted style 両方の VC。unconditional `equals` には existence、uniqueness、guarded-consistency、coverage VC はなく、ill-formed/type-invalid body は VC generation 前に reject する。predicate/attribute/mode/structure declaration に存在しない existence/uniqueness VC を付与せず、correctness acceptance なし。 |
| 34 | Predicate/functor の declared algebraic-property obligation。Specs 09.5, 10.6, 16.6。 | Core 33-36; `MT10-VC-PV/VC34`. | exact declared property/guard のみ。theorem fact や inferred property なし。 |
| 35 | Explicit mode-declaration `sethood`、structure inheritance/type-inclusion coherence、property-implementation `means` existence/uniqueness と overlap-coherence obligation。Specs 05.3/05.8, 07.8.1, 16.6, 19.2.2。 | Core 33-36; 必要なら parser 48; `MT10-VC-PV/VC35`. | Mode RHS inhabitation は mandatory checker evidence lookup/hard error のままで VC ではない。accepted constructor/inheritance/property/mode evidence なし。 |
| 36 | Term-derived choice non-emptiness、generated Fraenkel membership formula、per-occurrence non-template `qua` inheritance/cluster-widening validity obligation。Specs 13.4-13.6/13.8.6-13.8.7, 14。 | Core 34-36; `MT10-VC-PV/VC36`. | Template view actual は VC 41。Fraenkel sethood は VC generation 前に validate し、accepted evidence だけを context に入れる。use-site/generated sethood VC、implicit choice evidence、invented view、proof-free narrowing なし。 |
| 37 | Existential/conditional/functorial registration correctness。Specs 07.8, 16.6.3, 17.2-17.5/17.8.3/17.9。 | Core 39; `MT10-VC-PV/VC37`. | pending correctness VC のみ。activation/closure/registered fact なし。 |
| 38 | Predicate/functor/attribute redefinition compatibility/coherence。Specs 06.7, 09.6-9.7, 10.7-10.8, 11.1, 16.6, 19.5。 | Core 38; direct role のみ; `MT10-VC-PV/VC38`. | guessed root、accepted coherence、refinement winner、missing scheme role なし。 |
| 39 | Reduction `reducibility` universal-equality obligation。Specs 17.6, 17.9.4。 | Core 39; `MT10-VC-PV/VC39`. | simplification-order/size/variable check は structural registration-time rejection rule で、決して VC ではない。rewrite activation や label/order だけからの formula synthesis なし。 |
| 40 | **Blocked-reserved:** authenticated registration/cluster/reduction trace context/fingerprint。Specs 17.1/17.3.4/17.6-17.9。 | complete 済み VC 37/VC 39 output と Core 40、Gate A1/MC-G004; 未実行 `MT10-VC-PV/VC40`. | 全 dependency が揃った後、authenticated trace context/fingerprint を real VC-37 registration/cluster correctness VC と VC-39 reduction-equality VC、およびそれらの full snapshot に attach する。trace-derived goal や standalone trace-only candidate は決して作らない。 |
| 41 | Dependency-ready direct template use-site constraint、signature compatibility、view-actual validity obligation。Specs 18.2/18.10.2/18.10.4-18.10.5。 | Core 34-38; `MT10-VC-PV/VC41`. | authenticated direct role/substitution request だけを使用する。missing scheme/theorem role はこの executable slice の外で Core 41/Gate S1 により blocked のままとし、捏造しない。 |

definition/property task は existing kind が semantically honest な場合、exact
correctness-family provenance 付き `DefinitionCorrectness` を使う。Task 36 は個別の
generated non-emptiness/Fraenkel kind を使う。existing `GeneratedSethood` kind は以前の
generator contract に由来する explicit-handoff compatibility であり、Task 30 の
source-derived family ではない。explicit mode declaration sethood は VC 35 で
`DefinitionCorrectness` に属し、Fraenkel sethood は prerequisite evidence で sethood VC
を emit しない。将来の canonical authority と最初の real source なしに descendant が
`GeneratedSethood` を source-generate してはならない。registration/redefinition/
reduction task は exact family/style provenance 付き
`RegistrationStyleCorrectness` を使う。narrower kind の追加は canonical formula と
最初の real source を持つ将来 task だけが行え、Task 30 は empty enum expansion を
認可しない。

## 確定した algorithm VC task graph

各 algorithm task は diagnostic-bearing または incomplete な Core/CFG input を
reject する。branch と hidden-state fact は、explicit canonical formula により proof
obligation になる場合を除き、ordered context/provenance である。

| VC task | Bounded family と canonical authority | 必須 Core dependency と consumer | Exit boundary |
|---|---|---|---|
| 42 | Local `as T`/narrowing と field-update type obligation。Specs 05.7-05.8, 08.2, 13.3, 19.3, 20.1.3-20.1.4。 | Core 42/46/48/52-53; `MT10-VC-PV/VC42`. | 必要なら最初の real source と共に honest narrow kind を追加し、unrelated existing kind に押し込めない。 |
| 43 | callee-body context としての `requires`、return-result substitution、postcondition、assertion VC。Specs 20.4-20.5, 20.13.1/20.13.3。 | Core 42-43/46/48/52-53; `MT10-VC-PV/VC43`. | declared entry `requires` は `AlgorithmPrecondition` proof VC ではない。generated goal なしの return/postcondition fact なし。 |
| 44 | Call actual/result substitution、call-precondition VC、verified `terminating` callee の successor postcondition fact。Specs 20.4.1, 20.8, 20.13.1。 | Core 46/48/52-53; concrete substitution は VC-owned; `MT10-VC-PV/VC44`. | substitution は replayable、capture-safe、provenance-versioned。label text inference なし。Task 53 の bounded transport/authentication-authority gap が解消され exact evidence が利用可能になるまで partial-call postcondition を admit しない。 |
| 45 | nested real VC で実行する `if` path context。Specs 20.2.1, 20.13.3。 | Core 43/48/52-53; `MT10-VC-PV/VC45`. | `if` 単独では standalone VC を作らず、then/else condition は ordered context。 |
| 46 | ordered `match` capture/nonmatch context と explicit `exhaustive` proof のみ。Specs 20.2.5, 20.13.3。 | Core 45/50/52-53; `MT10-VC-PV/VC46`. | implicit exhaustiveness VC なし。honest kind は explicit canonical goal と共にだけ追加。 |
| 47 | old-state/havoc/alias context 付き while invariant establishment/preservation/break/continue。Specs 20.2.2, 20.5, 20.13.3。 | Core 43/46/48/52-53; `MT10-VC-PV/VC47`. | normal exit は successor context であり、exit の存在だけで `LoopInvariantPhase::Exit` にしない。 |
| 48 | range positive-step と invariant/break obligation。`to`/`downto` の evaluated-once `a0`/`b0`/`s0`/`i_exit` context。Specs 20.2.3, 20.5, 20.13.3。 | Core 44/46/49/52-53; `MT10-VC-PV/VC48`. | explicit canonical formula なしの `RangeBound`/`HiddenIndex` standalone VC なし。 |
| 49 | collection finiteness、invariant establishment/maintenance/break、processed-set context。Specs 20.2.4, 20.5, 20.13.3。 | Core 44/46/49/52-53; `MT10-VC-PV/VC49`. | standalone order independence は explicit formula が必要。それ以外は maintenance が意味を担う。 |
| 50 | runtime/ghost provenance を区別した set/type `Pick` non-emptiness。Specs 20.3, 20.13.3。 | Core 42/46/48/52-53; `MT10-VC-PV/VC50`. | exact Pick origin 付き non-emptiness kind を使用。chosen witness/execution result なし。 |
| 51 | term-derived Nat-valued/lexicographic loop/continue measure formula。Specs 20.5.2-20.5.3, 20.7, 20.13.3。 | Core 46/48-49/52-53; `MT10-VC-PV/VC51`. | explicit measure term/binder data だけから生成。text marker/automatic totality なし。 |
| 52 | recursive/mutually recursive component-decrease obligation。Specs 20.7-20.8, 20.13.4。 | Core 46/52-53; `MT10-VC-PV/VC52`. | `func` promotion、call-graph acceptance、termination fact なし。 |
| 53 | **Blocked-reserved non-VC admission boundary:** exact call に対する explicit cited/in-context verified termination theorem からの partial-call successor-postcondition admission。Specs 20.7-20.8, 20.13.1 は admission condition を定義するが、transport/authentication implementation は定義しない。 | Core 46/52-53; 未実行 `MT10-VC-PV/VC53`。current canonical authority は authenticated termination-evidence reference payload、producer、schema、authentication contract を命名していない。 | `PartialTermination` VC は存在しない。exact verified evidence がなければ call は type fact だけを提供し、successor-postcondition fact は0件。missing `decreasing` metadata や曖昧な caller request は obligation を作らない。VC 53 は将来の canonical authority が evidence producer、reference identity/schema、authentication rule、owning test を命名するまで blocked のまま。 |
| 54 | captured program context と universal valid-execution quantification を持つ snapshot/claim theorem obligation。Specs 20.6, 20.13。 | Core 47/51-53; `MT10-VC-PV/VC54`. | explicit captured payload なしの state theorem、old-state substitution、claim acceptance なし。 |
| 55 | Non-VC ghost-isolation integration と zero-VC accounting。Specs 20.1.3, 20.3, 20.13.5。 | Core 46/52-53; `MT10-VC-PV/VC55`. | diagnostic-free explicit isolation は `GhostErasureSafety` VC を emit せず、ghost leakage または corrupt/diagnostic-bearing flow は Core 53 を通じて reject する。extraction/runtime erasure/execution/accepted isolation fact なし。 |

現行 generated-formula/local-context builder は dependent binder、guarded schema、
concrete substitution、old-state/havoc、match nonmatch、term-derived goal のすべてを
まだ表現できない。各表現を最初に必要とする real task がこの source/design drift
を所有する。standalone empty formula-infrastructure task は置かない。deferred flow
row は owning task が exact spec-backed goal を供給した時だけ eligible になり、
general Deferred-to-Open promotion はない。

## Disagreement classification

| Class | Task-30 result |
|---|---|
| `spec_gap` | dependency-ready Tasks 31-52/54-55 を block する gap はない。widening の user-facing “no proof” 文言は Spec 13.8.7 の explicit automatically discharged FOL obligation により狭めて解釈できる。一方、bounded gap が VC 53 を block する。current canonical authority は exact verified termination evidence を要求するが、その producer、reference identity/schema、authentication rule、gate、owning test を命名していない。Task 30 は payload を捏造せずこの gap を報告し、VC 53 を reserved のままにする。 |
| `test_gap` | Task-180 VC runner/admission/baseline/corruption matrix、per-family positive assertion、owned zero-VC/near-miss/diagnostic negative は未実装で、Tasks 31-55 に割り当てた。 |
| `design_drift` | Task 30 が umbrella ownership drift を閉じる。structural terminal classification、honest kind/context/formula gap、non-template `qua`、direct-template use site、entry-`requires`、range/exit boundary を割り当てた。draft simplification-order、`PartialTermination`、`GhostErasureSafety` generation proposal は canonical authority 上 structural/evidence/static boundary で VC ではないため除去した。 |
| `source_drift` | exact Core terminal relation はまだ正しく消費されない。current entry-`requires` mapping と existing exit/range/partial/ghost data shape は canonical source generation を認可せず、broader formula/substitution/context route も未実装。 |
| `source_undocumented_behavior` | この inventory ではなし。 |
| `test_expectation_drift` | Task-30 scope ではなし。既存 type-elaboration/parser expectation は不変。 |
| `boundary_violation` | accepted contract には残らない。marker injection、raw-source reconstruction、fake anchor/status、acceptance、unrelated kind への押し込み、reject済み static/evidence family 3種のgenerationは違反になる。 |
| `repo_metadata_conflict` | なし。Task 30 は requirement row を追加せず count/hash oracle を維持する。 |

## Coverage と exit boundary

Task 30 は `spec_coverage_audit.md` について、umbrella follow-up ownership を accepted
VC 31-55 graph に置き換えるだけである。既存 trace row は deferred-reason ownership
text だけを変更でき、status、required flag、coverage class、tests は不変。chapter
rating、test count、runner count、behavioral hash はすべて不変である。

exact Task-31 mapping と2つの consumer contract が固定され、全 source-derived family
が bounded owner または explicit blocked authority gap を持ち、利用可能な Core dependency/canonical citation
が完全で、英日文書が同期し、全 preservation oracle が通った時だけ Task 30 は完了する。
次の sequential STEP 5 task は VC Task 31 である。Steps 6/7 は deferred のままで、
Tasks 31-55 の命名は proof-verification/acceptance credit を与えない。
