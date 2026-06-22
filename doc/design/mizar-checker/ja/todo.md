# mizar-checker TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様はまだ存在しない。各仕様は、それを引用する実装タスクより前に、
専用の仕様タスクが（英語と日本語を同じ変更で）執筆する。モジュール名は
[internal 07](../../internal/ja/07.crate_module_layout.md) の最小分割に従う。
この crate はアーキテクチャ 04、05、16、17、18、19 を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| typed_ast | `typed_ast.md`（task 2） | `src/typed_ast.rs` | [x] |
| binding_env | `binding_env.md`（task 4） | `src/binding_env.rs` | [x] |
| type_checker | `type_checker.md`（task 6） | `src/type_checker.rs` | [~] |
| registration_resolution | `registration_resolution.md`（task 13） | `src/registration_resolution.rs` | [~] |
| cluster_trace | `cluster_trace.md`（task 15） | `src/cluster_trace.rs` | [~] |
| overload_resolution | `overload_resolution.md`（task 21） | `src/overload_resolution.rs` | [~] |
| resolved_typed_ast | `resolved_typed_ast.md`（task 27） | `src/resolved_typed_ast.rs` | [ ] |

`mizar-checker` はパイプライン phase 6-8 を実装する。入力は `ResolvedAst` と
`SymbolEnv`、出力は `TypedAst`、`ResolutionTrace`、`ResolvedTypedAst` で
ある。phase に対応する 3 つの波で構築する: 型検査（phase 6）、再生可能な
trace を伴う cluster/registration 解決（phase 7）、オーバーロード解決
（phase 8）。soft type は意味論的メタデータであり、すべての事実は論理述語
または registration 由来の事実として説明可能でなければならず、どの波も
証明探索を行わない。

依存順序: `typed_ast` データ → `binding_env` / `type_checker`（第 1 波）→
`registration_resolution` / `cluster_trace`（第 2 波）→
`overload_resolution` / `resolved_typed_ast`（第 3 波）。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は `mizar-session` と `mizar-resolve` に依存する（`mizar-syntax`
には推移的に依存）。第 1 波は `mizar-resolve` task 14 と 20（名前解決、
`SymbolEnv` 骨格）を必要とし、以後の波は `mizar-resolve` task 21 の
シグネチャ増分と、対応する `mizar-parser` の定義文法タスク（23-31）と
ともに成長する。アーキテクチャ:
[04.type_and_registration_resolution.md](../../architecture/ja/04.type_and_registration_resolution.md)、
[05.overload_resolution.md](../../architecture/ja/05.overload_resolution.md)、
[16.substitution_and_binding.md](../../architecture/ja/16.substitution_and_binding.md)、
[17.cluster_trace_format.md](../../architecture/ja/17.cluster_trace_format.md)。
crate 所有権: [internal 07](../../internal/ja/07.crate_module_layout.md)。

## 解決済みおよび保留中の決定

- **TypedAst の arena 表現: task 3 で解決済み。** `TypedAst` は dense local
  id を持つ同質な `TypedNodeKind` arena を使い、現在の `mizar-syntax`
  compatibility view と `mizar-resolve` arena style を鏡映する。task 3 は
  node-kind storage のための direct `mizar-syntax` dependency を追加せず、
  checker-local な source-shape projection を使う。`ResolvedTypedAst` は task 28
  で同じ決定を再訪する。
- **registration の活性化ゲート: task 19 で解決済み。** ローカル
  registration は、その証明義務が設定済み verifier ポリシーに受理される
  まで自動推論に影響してはならない（アーキテクチャ 04 の制約）。phase
  11-14 がまだ存在しないため、task 19 は暫定ポリシーを実装する。
  生成された義務は pending / unverified status として記録し、explicit な
  accepted verifier/artifact status input が利用可能になるまで registration は
  active database に入らない。
  トップレベルに登録済み。`mizar-vc`/`mizar-proof` 着地時に再訪する。
- **trace スキーマ準拠: 解決済み。**
  [17.cluster_trace_format.md](../../architecture/ja/17.cluster_trace_format.md)
  が `ResolutionTrace` の正準スキーマである。`cluster_trace.md` はそれを
  精緻化するのであって、分岐させない。
- **diagnostics レコード: `mizar-resolve` の決定に従う**（`mizar-diagnostics`
  採用時期）。resolver が採用したレコードを checker も使う。トップレベルに
  登録済み。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-checker` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### 第 1 波: 型検査（phase 6）

1. **crate の足場と lint 方針のガード。** [x]
   - `mizar-session` と `mizar-resolve` に依存する workspace メンバー
     `mizar-checker` を追加し、`mizar-frontend` のガードに倣った
     `tests/lint_policy.rs` を追加する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: `mizar-resolve` task 5。仕様: アーキテクチャ 04。
   - task 1 で完了: crate scaffold、最小 crate root、dependency boundary、
     lint-policy guard を追加した。crate boundary を超える checker semantics や
     public API は導入していない。

2. **仕様: `typed_ast.md`。** [x]
   - `TypedAst` のデータ形状仕様を執筆する（英語と日本語、コードなし）:
     ノード arena、`TypeTable`、`TypeFactTable`、`CoercionTable`、
     `InitialObligationId` を持つ `InitialObligation`（`VcId` は決して
     使わない）、エラー後の部分型付け契約。
   - 依存: 1。仕様: アーキテクチャ 04「Typed AST」、
     [01.ir_layers.md](../../architecture/ja/01.ir_layers.md)。
   - task 2 で完了: `typed_ast.md` が論理データ形状、local context snapshot、
     table/status invariant、`InitialObligationId` boundary、partial-typing
     recovery、task 3 のテスト義務、deferred arena-representation decision を
     定義した。

3. **`typed_ast` データ形状の実装。** [x]
   - task 2 に従って arena とテーブルを実装し、arena 表現の決定を解決し、
     決定的 debug レンダリングを加える。
   - テスト: id の決定性。テーブルのラウンドトリップ。レンダリングの
     安定性。
   - 依存: 2。仕様: `typed_ast.md`。
   - task 3 で完了: `src/typed_ast.rs` が dense id、同質な `TypedNodeKind`
     arena、local context snapshot、type/fact/coercion/obligation/diagnostic
     table、validation、`typed-ast-debug-v1` rendering を実装した。unit test は
     determinism、table round-trip、context/status invariant、proof-boundary
     guard、stable rendering を覆う。

4. **仕様: `binding_env.md`。** [x]
   - 束縛/コンテキストの仕様を執筆する（英語と日本語、コードなし）:
     `SymbolEnv` 上の階層化ローカル型コンテキスト（モジュール層、ブロック
     層、束縛層。アーキテクチャ 04 Step 1）と、アーキテクチャ 16 と整合
     する checker 側の束縛変数の扱い（binder の識別、捕獲なし）。
   - 依存: 1。仕様: アーキテクチャ 04「Step 1」、
     [16.substitution_and_binding.md](../../architecture/ja/16.substitution_and_binding.md)。
   - task 4 で完了: `binding_env.md` が checker-owned binding/context boundary、
     layered context graph、binding identity、lookup order、reserved-variable
     handling、closure metadata expectation、diagnostic、deterministic rendering、
     task 5 の test obligation、external dependency gap を定義した。

5. **束縛環境とコンテキストの構築。** [x]
   - task 4 に従って `SymbolEnv` と `ResolvedAst` の束縛上にコンテキスト
     構築を実装する。
   - テスト: 層をまたぐ参照順序。reserve された変数のコンテキスト。binder
     スコープのフィクスチャ。決定的な反復順。
   - 依存: 3、4、`mizar-resolve` task 20。仕様: `binding_env.md`。
   - task 5 で完了: `src/binding_env.rs` が checker-owned binding-env data
     layer、validation、`ResolvedAst` と `SymbolEnv` 上の module-shell
     construction、明示 binding payload 上の local lookup、resolver
     `NameRefEntry::resolution()` fallback、決定的な `binding-env-debug-v1`
     rendering、現時点で未公開の resolver/source-walk payload に対する
     external-gap diagnostic を実装した。

6. **仕様: `type_checker.md`。** [x]
   - 検査/推論の仕様を、実装タスクが引用する名前付き節とともに執筆する
     （英語と日本語、コードなし）: 型式正規化（正規化述語としての型、
     Step 2）、宣言とローカル束縛の検査（Step 3）、項/論理式の推論
     （Step 4）、coercion 候補と initial obligation、型事実、部分型付けの
     回復。
   - 依存: 4。仕様: アーキテクチャ 04「Step 2」〜「Step 4」、
     [03.type_system.md](../../../spec/ja/03.type_system.md)、
     [08.type_inference.md](../../../spec/ja/08.type_inference.md)、
     [13.term_expression.md](../../../spec/ja/13.term_expression.md)。
   - task 6 で完了: `type_checker.md` が phase-6 boundary、normalized type
     model、task 7 normalization、task 8 declaration/local binding checking、
     task 9 term/formula inference、task 10 coercion and initial-obligation
     behavior、task 11 fact query、partial recovery、deterministic rendering
     expectation、external/deferred gate を定義した。

7. **型式の正規化。** [x]
   - surface の型式を正準述語形へ正規化する処理を実装する（attribute
     順序、`non`、radix 型の扱い）。
   - テスト: attribute 順序の正準化。正規化の冪等性。
   - 依存: 5、6。仕様: `type_checker.md`（正規化の節）。
   - task 7 で完了: `src/type_checker.rs` が task-local
     `NormalizedTypeTable` を持つ `TypeNormalizationOutput`、checker-owned
     type-expression payload normalization、deterministic type id/debug
     rendering、explicit mode-expansion provider support、`TypeEntry`
     emission、explicit mode-expansion provider payload 欠落時の degraded
     diagnostic、unsupported-payload recovery を実装した。resolver/source-walk
     site extraction と完全な signature payload は external dependency として残る。

8. **宣言とローカル束縛の検査。** [x]
   - 宣言とローカル束縛（`let`、`reserve`、`set`、…）を正規化された型に
     対して検査し、不正な宣言を診断し、エラー後も部分出力を保つ。
   - テスト: 束縛ごとのフィクスチャ。診断が束縛範囲を保持する。
   - 依存: 7。仕様: `type_checker.md`（宣言の節）。
   - task 8 で完了: `DeclarationChecker` が `BindingEnv` 上の checker-owned
     declaration/context payload を受け取り、binding declaration site に normalized
     type を attach し、local type-context snapshot を構築し、checked declaration の
     assumption fact を記録し、invalid / degraded assumption payload は diagnostic とともに
     drop し、illegal declaration 後も partial output を保持し、不足 RHS / body / reserve / evidence
     payload を raw syntax walk や task-10 obligation の捏造なしに deferred diagnostic として発行する。

9. **項と論理式の型推論。** [x]
   - 項と論理式の型を `TypeTable` へ推論する。候補が残る箇所では
     オーバーロード根を未確定のまま残す（アーキテクチャ 04「Overload
     Candidate Filtering Is Allowed, Root Selection Is Deferred」）。
   - テスト: パーサーが生成する項/論理式種別ごとの推論フィクスチャ。
     型エラー時の部分推論結果。
   - 依存: 8。仕様: `type_checker.md`（推論の節）。
   - task 9 で完了: `TermFormulaChecker` が checker-owned term/formula
     payload を受け取り、term ごとの `TypeEntry`、checked-formula
     well-formedness、task-local inference fact、決定的な open candidate set、
     expected-type constraint、partial/error/skipped recovery を記録する。final
     overload selection、raw syntax walk、`CoercionTable` 発行、
     `InitialObligation` の捏造は行わない。

10. **coercion 候補、sethood、non-emptiness、narrowing 義務。** [x]
    - widening/narrowing/`qua` の coercion 候補を `CoercionTable` に記録し、
      sethood/non-emptiness/narrowing の `InitialObligation` を発行する。
    - テスト: coercion 種別ごとの候補集合。義務が `InitialObligationId` と
      ソース範囲を保持する。sethood/non-emptiness evidence 欠落と不正な
      `qua` narrowing の fail fixture も含める。
    - 依存: 9。仕様: `type_checker.md`（coercion/義務の節）。
    - task 10 で完了: `CoercionObligationChecker` は checker-owned な
      coercion / initial-obligation payload を受け取り、widening/source-`qua`/
      narrowing candidate を記録し、deterministic local id と source range を持つ
      sethood/non-emptiness/narrowing `InitialObligation` を作る。supporting fact
      のため input fact id を保持し、obligation-backed fact を追加する。不足する
      inheritance / summary / cluster / sethood / non-emptiness / proof-query
      input は `VcId` 割り当て、obligation discharge、inserted view 捏造ではなく
      external dependency gap として残す。

11. **型事実の記録とクエリ。** [x]
    - 推論中の事実記録と、registration/overload の波が後で使う決定的
      クエリ API を実装する。
    - テスト: 事実の来歴。クエリの決定性。事実の重複なし。
    - 依存: 9, 10。仕様: `type_checker.md`（型事実の節）。
    - task 11 で完了: `TypeFactQueryEngine` は既存 checker fact table 上で
      deterministic point query に答え、optional `LocalTypeContextTable` を通じて
      local assumption visibility を尊重し、explicit な `Satisfied` / `Missing` /
      `Contradicted` status を返す。contradiction diagnostic は fact を mutate せず
      報告し、provenance は point-query matching ではなく ordering / explanation のために
      保持する。statement/proof assumption、theorem acceptance、phase-7 trace fact は
      MC-G019 external dependency gap として残す。

12. **stage `type_elaboration` のコーパスランナー。** [x]
    - stage `type_elaboration` の external-gap fail case を
      `spec_trace.toml` 項目付きでハーネスに接続する。real task 7-11 semantic
      pass/fail seed は source-to-checker payload extraction が存在するまで deferred にする。
    - 依存: 10、11。仕様: [staged_model.md](../../mizar-test/ja/staged_model.md)。
    - task 12 で boundary-preserving runner として完了:
      active `type-elaboration` harness command は `.miz` case を frontend parsing と
      resolver symbol collection まで通し、AST 全体の source-to-checker payload
      extraction API が存在するまで MC-G020
      `type_elaboration.external_dependency.ast_payload_extraction` を report する。
      real task 7-11 semantic pass/fail `.miz` assertion は checker payload を捏造して
      accepted にせず deferred のままにする。

### 第 2 波: cluster と registration の解決（phase 7）

13. **仕様: `registration_resolution.md`。** [x]
    - registration の仕様を執筆する（英語と日本語、コードなし）: pending
      と activated のデータベース分離、existential ゲート、来歴付き
      reduction 書き換え、検証義務（アーキテクチャ 04 Step 5-6）。
    - 依存: 2。仕様: アーキテクチャ 04「Registration Databases」、
      [17.clusters_and_registrations.md](../../../spec/ja/17.clusters_and_registrations.md)。
    - task 13 で完了: `registration_resolution.md` は phase-7 境界、
      pending/activated registration database 分離、validation と
      `InitialObligationId` rule、existential gating、cluster closure、
      reduction provenance、deterministic diagnostic/recovery、task 14 と
      16-20 の planned test、MC-G021 external/deferred payload gap を、source
      behavior を追加せず定義する。

14. **registration 索引。** [x]
    - `SymbolEnv` の registration 宣言上に pending/activated データベースを
      実装する。
    - テスト: pending 項目は発火しない。活性化が項目を決定的に移動する。
      ソース単位の寄与追跡。
    - 依存: 11、13、`mizar-resolve` task 21（registration 増分）。仕様:
      `registration_resolution.md`。
    - task 14 で完了: `registration_resolution` module は resolver registration
      identity/origin metadata の pending / activated / rejected database を構築し、
      semantic payload を完全な explicit `ActivationInput` として受け取った場合だけ
      active record を作成する。pending external gap と invalid activation は
      deterministic diagnostic/debug output に残り、MC-G021 semantic payload gap は
      deferred のまま。

15. **仕様: `cluster_trace.md`。** [x]
    - 正準スキーマの精緻化として `ResolutionTrace` の仕様を執筆する
      （英語と日本語、コードなし）: cluster step、reduction step、前件
      事実、監査キー、決定的トラバーサル、再生コスト上限。
    - 依存: 13。仕様:
      [17.cluster_trace_format.md](../../architecture/ja/17.cluster_trace_format.md)。
    - task 15 で完了: `cluster_trace.md` は architecture 17 の正準 schema を
      fork せず精緻化し、checker-local な cluster / reduction step ownership、
      antecedent fact reference、audit key、deterministic traversal、replay-cost bound、
      diagnostic、tasks 16-18 の planned test を固定する。source behavior は task 16
      まで deferred のままで、real semantic payload は MC-G021 によって gate される。

16. **trace 記録付き cluster 解決閉包。** [x]
    - 決定的トラバーサルで attribute 伝播の閉包を実装し（アーキテクチャ 04
      Step 5）、すべての適用を `ResolutionTrace` に記録する。
    - テスト: 閉包フィクスチャ。trace の再生が同じ導出事実に到達する。
      決定的な適用順。subtype-compatible conditional cluster。
      pending/rejected/unaccepted registration は発火しない。
    - 依存: 14、15。仕様: `cluster_trace.md`、`registration_resolution.md`。
    - task 16 で完了: `cluster_trace` は explicit `ClusterRuleInput` /
      `ClusterFactInput` payload と task-14 activated registration 上の checker-owned
      cluster closure data layer を公開する。replayable cluster step、trace provenance
      付き derived closure fact、deterministic traversal profile、checker-local diagnostic
      を記録し、reduction、artifact emission、`TypeFactTable` mutation、resolver shell
      semantic の捏造は行わない。

17. **cluster ループ検出と有界飽和。** [x]
    - cluster ループを検出し、発散する代わりに有界飽和診断を発行する
      （アーキテクチャ 17「Cluster Loop Detection」）。
    - テスト: ループフィクスチャが安定した診断で停止する。上限が設定として
      可視である。矛盾する導出は fatal であり、degraded verified fact を export しない。
    - 依存: 16。仕様: [17.cluster_trace_format.md](../../architecture/ja/17.cluster_trace_format.md)。
    - task 17 で完了: cluster closure は fact ancestry/depth を追跡し、
      direct / indirect loop を診断し、traversal profile / cache-key visibility 付きで
      depth / generated-fact bound を強制し、explicit conflict-fingerprint contradiction を
      incomplete closure result として報告し、拒否された degraded fact は挿入しない。
      source-derived `TypeFactTable` contradiction check と artifact/cache integration は
      deferred のまま。

18. **reduction の適用。** [x]
    - reduction 書き換え（redex パス、置換、ガード証拠）を、完全な来歴を
      `ResolutionTrace` に記録しつつ実装する。
    - テスト: redex パスの正しさ。ガード証拠の必須化。source redex、
      target term、rule FQN、rule-view fingerprint、selection key、
      enclosing-term fingerprint、source provenance の記録。`such` side condition は
      applicability-only。pending/rejected/unaccepted reduction は rewrite しない。
      invalid substitution と mismatched strategy-audit key を診断する。再生可能な
      trace。
    - 依存: 16。仕様: `registration_resolution.md`（reduction の節）、
      アーキテクチャ 17「Reduction Step」。
    - task 18 で完了: `ReductionTraceBuilder` は explicit payload 上で replayable
      reduction step を記録し、architecture-17 provenance field を保持し、active reduction
      registration、rule-view fingerprint、substitution、guard evidence、strategy-audit
      key を検証し、`such` guard を applicability-only evidence として扱う。raw syntax
      matching、resolver-shell parsing、artifact/cache integration、source-derived
      reduction extraction は deferred のまま。

19. **pending registration の検証と活性化ゲート。** [x]
    - pending registration 宣言を検証し（アーキテクチャ 04 Step 6）、その
      義務を発行し、暫定の活性化ゲートポリシーを実装する。決定をここと
      トップレベルに記録する。
    - テスト: 不正な registration の診断。kind-specific validation は existential、
      conditional、functorial、reduction pattern を cover し、reduction の
      free-variable / occurrence / orientation / source-provenance check を含む。
      未検証 registration は推論に影響せず、policy-admitted activation には後続
      proof/artifact input からの accepted verifier status を要求する。
    - 依存: 17、18。仕様: `registration_resolution.md`。
    - task 19 で完了: `RegistrationValidationInput` は explicit な checker-ready
      pending payload を検証し、checker-local `InitialObligationId` を発行し、validated
      record を `inference=false` の pending として保持し、recovered origin と malformed
      kind-specific payload を rejected にし、spec 17.6.4 固定の reduction size /
      variable rule を強制し、verifier/artifact status が missing または rejected の
      activation input を rejected にする。source extraction、accepted-status
      production/import、artifact reuse、active `.miz` semantic fixture は deferred のまま。

20. **attribute 付き型使用の existential ゲート。** [x]
    - attribute 付き型は existential registration が非空性を正当化する
      箇所でのみ使用可能であることを強制する（アーキテクチャ 04
      「Existential Registrations Gate Attributed Type Use」）。
    - テスト: existential 欠落フィクスチャが安定した診断で失敗する。
      pending/rejected/unaccepted existential registration は gate を満たさない。
      activated gate は visible guard を要求し、degraded recovery 後に verified fact を
      seed しない。
    - 依存: 19。仕様: `registration_resolution.md`、
      [17.clusters_and_registrations.md](../../../spec/ja/17.clusters_and_registrations.md)。
    - task 20 で完了: `ExistentialGateOutput` は explicit な checker-owned gate
      payload を activated existential registration に照らして評価し、candidate を
      accepted validation kind と pattern / correctness / evidence / fingerprint record に
      bind し、visible consumable guard fact evidence を要求し、full accepted
      attributed-type pattern を match し、deterministic result precedence を適用し、
      satisfied normal gate だけが verified fact を seed できることを保証する。source
      extraction、artifact reuse、accepted-status production、active `.miz` gate fixture は
      deferred のまま。

### 第 3 波: オーバーロード解決（phase 8）

21. **仕様: `overload_resolution.md`。** [x]
    - オーバーロードの仕様を、名前付き節とともに執筆する（英語と日本語、
      コードなし）: 来歴付きサイト/候補収集、template 展開、記録済み事実に
      よる viability、specificity 半順序（サイトごとのグラフ、グローバル
      DAG なし）、根の選択と refinement 結合、`qua` view 挿入（widening
      限定、多重継承の曖昧性）、失敗サイトの保存（アーキテクチャ 05）。
    - 依存: 2。仕様: アーキテクチャ 05、
      [19.overload_resolution.md](../../../spec/ja/19.overload_resolution.md)、
      [18.templates.md](../../../spec/ja/18.templates.md)。
    - task 21 で完了: `overload_resolution.md` は checker-local phase-8 boundary、
      explicit site/candidate payload、template expansion、recorded fact 上の
      viability、per-site specificity graph、root selection、refinement join、
      widening-only inserted `qua` view、failed-site preservation、diagnostic、
      determinism、tasks 22-26 の planned task coverage、MC-G027 test/deferred/external
      gap を code なしで定義する。

22. **候補サイトの収集。** [x]
    - スコープ/可視性でフィルタ済みの後、`TypedAst` site ref と resolver symbol id を
      持つ explicit overload site / candidate payload を収集する。
    - テスト: 適用形ごとのサイトカバレッジ。来歴の保持。決定的な候補順。
    - 依存: 11、21。仕様: `overload_resolution.md`（サイトの節）。
    - task 22 で完了: `src/overload_resolution.rs` は explicit site / candidate
      payload 上の checker-owned `OverloadCollectionOutput::collect` を公開する。
      deterministic local site/candidate id を割り当て、site / candidate provenance、
      source-written `qua`、template、coherence metadata を保持し、duplicate site key と
      missing candidate-site link を rejected input provenance を残しながら診断し、
      unsupported role を stable diagnostic 付きで deferred にし、`SymbolEnv` scan、raw syntax walk、template expansion、viability
      check、root selection、`ResolvedTypedAst` projection なしに、scope/visibility
      filter 済み candidate set を供給された通りに保存する。

23. **template 展開。** [ ]
    - 通常の候補順序付けに先立って template 候補を具体候補へ展開する。
      展開不能な template は理由を記録して除外する。
    - テスト: 展開フィクスチャ。constrained-template evidence case。
      除外が理由を保持する。
    - 依存: 22、`mizar-parser` task 31。仕様: `overload_resolution.md`
      （template の節）。

24. **viability フィルタリング。** [ ]
    - 記録済みの型事実のみを用いて候補を viability でフィルタする — 新しい
      推論は行わない（アーキテクチャ 05「Viability Uses Type Facts, Not
      New Inference」）。
    - テスト: viability フィクスチャ。consumable な fact evidence と
      pending/degraded/rejected fact evidence の対比。診断のための棄却理由の保持。
    - 依存: 23。仕様: `overload_resolution.md`（viability の節）。

25. **specificity グラフの構築。** [ ]
    - viable 候補上にサイトごとの specificity グラフを構築する。
    - テスト: 順序フィクスチャ。比較不能な組は比較不能のまま。決定的な
      グラフレンダリング。
    - 依存: 24。仕様: `overload_resolution.md`（specificity の節）。

26. **根の選択、refinement の結合、view の挿入。** [ ]
    - オーバーロード根を選択し、整合する refinement グループを結合し、
      `qua` view を挿入し、失敗サイトを明示的に保存する（アーキテクチャ
      05 Step 5）。
    - テスト: strongest-type、attribute-union、incompatible refinement join を
      含む選択フィクスチャ。候補リスト付きの曖昧性診断。失敗サイトは
      決して有効な出力にならない。
    - 依存: 25。仕様: `overload_resolution.md`（選択/view の節）。

27. **仕様: `resolved_typed_ast.md`。** [ ]
    - `ResolvedTypedAst` のデータ形状仕様を執筆する（英語と日本語、コード
      なし）: 最終型、`OverloadResolutionTable`、`CoercionInsertionTable`、
      `ClusterFactTable`、式メタデータ。
    - 依存: 21。仕様: [01.ir_layers.md](../../architecture/ja/01.ir_layers.md)、
      アーキテクチャ 05「Step 6」。

28. **`ResolvedTypedAst` の組み立て。** [ ]
    - LSP と artifact のための式メタデータを備えた最終の source 形状
      意味論 AST を組み立て、決定的 debug レンダリングを加える。
    - テスト: 組み立てフィクスチャ。`ExprId` によるメタデータ参照。
      レンダリングの安定性。
    - 依存: 26、27。仕様: `resolved_typed_ast.md`。

### 強化と横断フォローアップ

29. **stage `formula_statement` / `advanced_semantics` のコーパス拡大。** [ ]
    - registration/overload のコーパスケース（cluster、reduction、曖昧性、
      refinement 結合）を `spec_trace.toml` 項目付きで追加し、40/60 の
      pass/fail 比率へ向けて拡大する。
    - レビュー監査由来の advanced-semantics negative case として、
      `now`/`proof` block からの witness leakage、未充足の `deffunc`/`defpred`
      guard、comprehension の sethood 欠落、不正な `qua` narrowing を含める。
    - 依存: 20、28。仕様: [staged_model.md](../../mizar-test/ja/staged_model.md)。

30. **決定性スイート。** [ ]
    - 同一入力が同一の型、事実、trace、候補順、診断を生むことの
      プロパティ的検証。
    - 依存: 28。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

31. **公開 enum の前方互換性ポリシー。** [ ]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用し、所有
      モジュール仕様に決定を記録する。
    - 依存: 28。仕様: 全モジュール仕様。

32. **ソース/仕様対応監査。** [ ]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - 依存: 31。仕様: 全モジュール仕様と本 TODO。

33. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-checker/en/` の各英語正本と日本語版を比較し、
      内容を同期する。
    - 依存: 32。仕様: リポジトリのドキュメント方針。

34. **module 境界リファクタリング gate。** [ ]
    - crate を下流 consumer 向けに完了扱いにする前に、source layout を監査し、
      oversized file、混在した責務、module table と module spec 境界に沿って
      分割すべき private helper を洗い出す。review bottleneck になった実装
      ファイルは、公開 API、診断、決定的 rendering、artifact-facing schema、
      consumer-visible behavior を変えずに private module へ分割する。
    - 分割後は必要に応じて本 module table / source path を更新し、移動した
      API について source/spec 対応監査と二言語ドキュメント同期監査の範囲を
      再実行する。挙動 cleanup や API 公開を移動と混ぜない。それらは独立した
      spec task を要求する。
    - 依存: 33。仕様: 本 TODO、
      [internal 07](../../internal/ja/07.crate_module_layout.md)、全モジュール仕様。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-checker
cargo clippy -p mizar-checker --all-targets -- -D warnings
```

resolver 境界やコーパスに触れるタスクでは追加で実行する:

```text
cargo test -p mizar-resolve
cargo test -p mizar-test
```

テストが通ったらここでタスクにチェックを付ける。

## 備考

- checker が所有するのは soft type の事実、再生可能な registration 効果、
  オーバーロードの最終決定のみ: 証明探索、ATP の前提選択、任意の一階推論は
  行わない。
- ここで `VcId` を割り当てることは決してない。phase 6-8 は
  `InitialObligationId` を発行し、`mizar-vc` が後で正確に 1 回変換する。
- 各波の網羅性は `mizar-resolve` のシグネチャ増分とパーサーの定義文法
  タスクが律速する。resolver がまだ収集できない宣言種別を検査しない。
- 依存スライスと fingerprint の統合（アーキテクチャ 18）は `mizar-cache`
  とともに到来する。checker はスライスが計算可能であり続けるよう、
  ソース単位の寄与追跡を正確に保つだけでよい。
