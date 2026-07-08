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
| resolved_typed_ast | `resolved_typed_ast.md`（task 27） | `src/resolved_typed_ast.rs` | [~] |

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
- **constructor の property 値供給源: task 35 で解決済み。** デフォルトの
  構造体 constructor は field のみを受け取り、`property` 値は第 7 章の
  property implementation からのみ来る。task 35 は spec 05/07 を英日で
  更新し、reject-first の inactive `advanced_semantics` seed と traceability を
  追加し、checker/core source semantics は変更しない。

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
      よる viability、specificity 前順序（サイトごとのグラフ、グローバル
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

23. **template 展開。** [x]
    - 通常の候補順序付けに先立って template 候補を具体候補へ展開する。
      展開不能な template は理由を記録して除外する。
    - テスト: 展開フィクスチャ。constrained-template evidence case。
      除外が理由を保持する。
    - 依存: 22、`mizar-parser` task 31。仕様: `overload_resolution.md`
      （template の節）。
    - task 23 で完了: `TemplateExpansionOutput::expand` は task 22 が保持した
      explicit `TemplateCandidatePayload` metadata だけを検証する。non-template candidate を
      copy し、成功した template を `CandidateOrigin::TemplateDerived` 付き concrete
      candidate に instantiate し、substitution と `TemplateExpansionTable` row を記録し、
      skipped template candidate を stable rejection/deferred diagnostic とともに保存する。
      explicit argument、omitted inference payload、accepted/missing/deferred constraint、
      source-`qua` widening/narrowing status、non-template priority、unsupported/deferred
      candidate、deterministic rendering を、cluster expansion、新しい fact inference、
      viability、specificity、root selection、view insertion なしで cover する。

24. **viability フィルタリング。** [x]
    - 記録済みの型事実のみを用いて候補を viability でフィルタする — 新しい
      推論は行わない（アーキテクチャ 05「Viability Uses Type Facts, Not
      New Inference」）。
    - テスト: viability フィクスチャ。consumable な fact evidence と
      pending/degraded/rejected fact evidence の対比。診断のための棄却理由の保持。
    - 依存: 23。仕様: `overload_resolution.md`（viability の節）。
    - task 24 で完了: `CandidateViabilityOutput::filter` は
      `TemplateExpansionOutput` と、concrete candidate id で key 付けされた
      explicit checker-owned viability payload を消費する。完全に viable な candidate
      だけを出力し、すべての candidate について decision row を記録し、accepted exact、
      consumable fact、widening、source-`qua` view plan を保持し、
      pending/degraded/rejected/out-of-scope/missing/narrowing evidence を stable
      diagnostic 付きで reject し、ambiguous または externally deferred payload を block
      する。新しい type inference、fact derivation、cluster firing、root selection、
      view insertion は行わない。

25. **specificity グラフの構築。** [x]
    - viable 候補上にサイトごとの specificity グラフを構築する。
    - テスト: 順序フィクスチャ。比較不能な組は比較不能のまま。決定的な
      グラフレンダリング。
    - 依存: 24。仕様: `overload_resolution.md`（specificity の節）。
    - task 25 で完了: `SpecificityGraphOutput::build` は
      `CandidateViabilityOutput` と、viable candidate id で key 付けされた explicit
      checker-owned pairwise comparison payload を消費する。site ごとに 1 graph、viable
      concrete candidate ごとに 1 node、same-site pair の comparison row、accepted
      at-least-as-specific relation だけの directed edge、edge を持たない explicit
      incomparable row、missing / duplicate / unknown / cross-site comparison payload 用の
      stable diagnostic を出力する。fact derivation、ordering 用の result type 参照、
      root-selection tie-breaker 適用、refinement join、view insertion は行わない。

26. **根の選択、refinement の結合、view の挿入。** [x]
    - オーバーロード根を選択し、整合する refinement グループを結合し、
      `qua` view を挿入し、失敗サイトを明示的に保存する（アーキテクチャ
      05 Step 5）。
    - テスト: strongest-type、attribute-union、incompatible refinement join を
      含む選択フィクスチャ。候補リスト付きの曖昧性診断。失敗サイトは
      決して有効な出力にならない。missing / duplicate / unknown / blocked
      payload diagnostic、missing / ambiguous ordinary-root candidate diagnostic、
      deterministic selection rendering。
    - 依存: 25。仕様: `overload_resolution.md`（選択/view の節）。
    - task 26 で完了: `OverloadSelectionOutput::resolve` は
      `SpecificityGraphOutput` と explicit checker-owned selection payload を
      消費する。site ごとの graph から unique maximal non-redefinition ordinary
      root candidate を選択し、`NoMatch`、`Ambiguous`、
      `IncompatibleRefinementJoin`、blocked site を failed output として記録し、
      accepted coherence を持つ same-root redefinition payload を検証し、root
      selection 後に限って strongest-result / attribute-union の exposed result
      metadata を受け入れ、accepted widening/source-`qua` inserted view を記録する。
      non-selected refinement、missing payload、narrowing / missing view evidence、
      blocked specificity graph は成功を捏造せず、additional root-selection
      tie-breaker も適用せず拒否する。

27. **仕様: `resolved_typed_ast.md`。** [x]
    - `ResolvedTypedAst` のデータ形状仕様を執筆する（英語と日本語、コード
      なし）: 最終型、`OverloadResolutionTable`、`CoercionInsertionTable`、
      `ClusterFactTable`、式メタデータ。
    - 依存: 21。仕様: [01.ir_layers.md](../../architecture/ja/01.ir_layers.md)、
      アーキテクチャ 05「Step 6」。
    - task 27 で完了: `resolved_typed_ast.md` は final source-shaped semantic
      AST boundary、node / expression metadata table、overload resolution
      projection、coercion insertion metadata、cluster fact reference / provenance preservation、
      failed-site preservation、deterministic rendering expectation、task 28 の
      planned tests、source-extraction / artifact gap を code なしで定義する。

28. **`ResolvedTypedAst` の組み立て。** [x]
    - LSP と artifact のための式メタデータを備えた最終の source 形状
      意味論 AST を組み立て、決定的 debug レンダリングを加える。
    - テスト: 組み立てフィクスチャ。`ExprId` によるメタデータ参照。
      レンダリングの安定性。
    - 依存: 26、27。仕様: `resolved_typed_ast.md`。
    - task 28 で完了: `ResolvedTypedAst::assemble` は explicit checker-owned
      typed AST、cluster fact、overload collection / template / viability /
      specificity、selection output を source-shaped resolved node、expression
      metadata、collection / expanded / viable candidate summary、template
      expansion summary、viability decision、specificity graph summary、overload
      record、inserted coercion、diagnostic、deterministic rendering へ射影する。
      failed site を保持し、source extraction、artifact、public diagnostic code、
      active `.miz` fixture は deferred のままにする。

### 強化と横断フォローアップ

29. **stage `formula_statement` / `advanced_semantics` の deferred corpus obligation。** [x]
    - registration/overload の deferred corpus obligation（cluster、reduction、
      曖昧性、refinement 結合）を `spec_trace.toml` 項目付きで記録する。
      active な 40/60 pass/fail 拡大は future work として残る。
    - レビュー監査由来の advanced-semantics negative obligation として、
      `now`/`proof` block からの witness leakage、未充足の
      `deffunc`/`defpred` guard、comprehension の sethood 欠落、不正な
      `qua` narrowing を deferred として記録する。
    - 依存: 20、28。仕様: [staged_model.md](../../mizar-test/ja/staged_model.md)。
    - task 29 では deferred corpus-record task として完了:
      `spec_trace.toml` は formula/statement、cluster/reduction、
      overload/refinement、review-audit negative obligation を、具体的な
      MC-G019/MC-G020/MC-G021/MC-G023/MC-G027 と runner blocker 付きで
      deferred として記録する。`mizar-test` に active `formula_statement` /
      `advanced_semantics` runner がなく、それらの case に必要な
      source-to-checker semantic payload extraction も mizar-checker にまだ
      存在しないため、active `.miz` fixture は追加していない。

30. **決定性スイート。** [x]
    - 同一入力が同一の型、事実、trace、候補順、診断を生むことの
      プロパティ的検証。
    - 依存: 28。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。
    - task 30 で完了: `crates/mizar-checker/src/determinism_suite.rs` は
      checker-owned Rust regression として、同一 input の rerun と
      canonicalized equivalent-input permutation を、type normalization、
      type-fact contradiction query、cluster closure trace、overload
      collection/template/viability/specificity/selection output、final
      `ResolvedTypedAst::assemble` projection に対して追加する。stage runner と
      source-to-checker payload extraction は既存 external gap の下で deferred
      のままなので、active `.miz` fixture は追加していない。

31. **公開 enum の前方互換性ポリシー。** [x]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用し、所有
      モジュール仕様に決定を記録する。
    - 依存: 28。仕様: 全モジュール仕様。
    - task 31 で完了: 現在の checker-owned public enum はすべて downstream
      forward-compatible API surface として分類し、`#[non_exhaustive]` を維持する。
      各 owning EN/JA module spec は `Public Enum Policy` table と
      no-exhaustive-exceptions statement を記録し、`tests/lint_policy.rs` は今後の
      public enum attribute と policy row の source/spec drift を guard する。

32. **ソース/仕様対応監査。** [x]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - 依存: 31。仕様: 全モジュール仕様と本 TODO。
    - task 32 で完了: [source_spec_audit.md](./source_spec_audit.md) は
      現在の checker `pub mod` export、top-level public item、public
      `dense_id!` / `string_key!` newtype をすべて inventory し、module behavior
      promise を implementation、Rust tests、または明示的な MC-G
      `external_dependency_gap` / `test_gap` / `deferred` row へ trace する。
      `tests/lint_policy.rs` はその inventory と gap reconciliation を guard する。
      この audit task では source/API behavior、`.miz` fixture、expectation を変更していない。

33. **二言語ドキュメント同期監査。** [x]
    - `doc/design/mizar-checker/en/` の各英語正本と日本語版を比較し、
      内容を同期する。
    - 依存: 32。仕様: リポジトリのドキュメント方針。
    - task 33 で完了: [bilingual_sync_audit.md](./bilingual_sync_audit.md) は
      現在の英日 checker design document pair をすべて inventory し、companion
      link と comparison basis を記録し、各 pair の sync debt を `none` と記録する。
      `tests/lint_policy.rs` は今後の pair inventory drift を guard する。

34. **module 境界リファクタリング gate。** [x]
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
    - task 34 で完了: [module_boundary_audit.md](./module_boundary_audit.md) は
      現在の checker Rust source / test-support file すべてを line count、boundary
      label、owning specification、split decision、hard-gate status とともに
      inventory する。必須の behavior-neutral split はない。大きい cohesive file は
      monitored ergonomics note のみであり、`tests/lint_policy.rs` が今後の
      source-layout audit drift を guard する。

### 第 4 波: 意味論監査フォローアップ（2026-07-03）

[semantic_spec_audit.md](./semantic_spec_audit.md) は checker が担当する仕様章
(03、05-08、13、14、17-19)を監査し、所見 SSA-001 から SSA-020 と 16 件の
adversarial rejection corpus を記録した。以下のタスクは全所見を、担当タスク
か明示的な処置(disposition)のいずれかに変換する。AGENTS.md が
`doc/spec/en/` を設計文書・コードより上位に置くため、spec 決定タスク
(35-44)が先行する: 仕様が決めていない挙動を checker が実装してはならない。
各 spec タスクは監査の提案解決策から選択し(またはより良い解決策を記録し)、
`doc/spec/en/` と `doc/spec/ja/` を同一変更で更新し、決定が新しい拒否を
生む場合は reject-first corpus seed を追加または活性化し、
`tests/coverage/spec_trace.toml` を更新する。

所見の処置(全 SSA id はタスクまたは記録済みの理由に対応する):

| 所見 | 処置 |
|---|---|
| SSA-001 | task 35 |
| SSA-002, SSA-011, SSA-012 | task 36 |
| SSA-003, SSA-010, SSA-016, SSA-019 | task 37 |
| SSA-004 | task 38 |
| SSA-005 | task 39 |
| SSA-006 | task 40 |
| SSA-007, SSA-008, SSA-020 | task 41 |
| SSA-009 | task 42 |
| SSA-013, SSA-014 | task 43 |
| SSA-015, SSA-017 | task 44 |
| SSA-018 | タスク化しない: greedy `of`/`over` parse は決定的かつ文書化済み(spec 19.6.4)。scope 感度 lint は将来の diagnostics 採用 wave に属し、そこで記録する |
| corpus seeds | task 49 が必要な runner、parser support、declaration-symbol support、source-to-checker payload extraction 到着時に、監査 fixture 16 件、task-35 constructor-property seed、task-36 duplicate-coverage seed、task-37 ordinary/template-derived equivalent-root seed と same-return signature-conflict seed、task-38 functorial-`for` guard seed、task-39 property-overlap coherence seed、task-44 omitted-`reconsider` / ambiguous-redefinition-target seed を活性化する |

35. **Spec 決定: constructor property 引数と extensionality(SSA-001)。** [x]
    - critical な §5.5.1/§5.8.4/§5.8.5 の不整合を解決する。推奨は解決策 1:
      constructor は field のみを受け取り、property 値は常に §7.4.1 の
      property implementation から来る。spec 05 と 07 を(英日同一変更で)
      更新し、テンプレート監査で導入済みの exact-instance extensionality
      本文(spec 05 §5.8.5、commit `cef7e109`)と整合させる。
    - 受け入れ条件: property 値の供給源が仕様上ちょうど 1 つになる。拒否
      される constructor-property 形を固定する reject-first `.miz` seed を
      sidecar と `spec_trace.toml` エントリ付きで追加する。§5.8 のいかなる
      公理族も相異なる property 引数から `b1 = b2` を導出できない。
    - 検証: `cargo test -p mizar-test`; corpus JSON/TOML の妥当性。
    - 依存: なし(spec wave の先頭)。参照: SSA-001;
      [template_encoding_audit.md](../../mizar-core/en/template_encoding_audit.md) F1。
    - task 35 で完了: spec 05 はデフォルト constructor を field-only とし、
      property projection axiom を削除した。spec 07 は property implementation が
      property 値の唯一の供給源であると明記した。
      `fail_structure_constructor_property_arg_001` を inactive
      `advanced_semantics` reject-first seed として追加し、traceability row
      `spec.en.05.structures.constructor_fields_only.semantic` と
      `spec.en.07.modes.property_implementation.not_constructor_source.semantic` を記録した。
      checker/core source semantics は変更していない。

36. **Spec 決定: structure member 同一性・upcast path・非循環性(SSA-002, SSA-011, SSA-012)。** [x]
    - diamond member 同一性を `from` chain で到達する root 宣言として定義
      する(またはより良い規則を記録する)。child member 型が全 parent の
      member 型に対して `⊑` であることを parent ごとの coherence 義務付きで
      要求する。§19.2.2 の path 一意性が syntactic か semantic かを明記する。
      §5.3 に明示的な継承非循環規則と診断を追加する。spec 05 と 19 を
      (英日同一変更で)更新し、§5.8.3/§13.8.7 に導入済みの reduct-view
      エンコード(`view_{D→B}`)と整合させる。
    - 受け入れ条件: 既存 seed
      `fail_structure_diamond_member_type_conflict_001`、
      `fail_structure_inherit_uncovered_member_001`、
      `fail_structure_inherit_cycle_001`、
      `fail_overload_inheritance_path_ambiguity_001` が決定後の規則の下でも
      有効(決定が根拠を変える場合は sidecar note のみ改訂)。member 改名
      時の同一性ケースに決定済みの結果がある。
    - 検証: `cargo test -p mizar-test`。
    - 依存: 35。参照: SSA-002, SSA-011, SSA-012; テンプレート監査 F1。
    - task 36 で完了: spec 05 は継承 member identity を root declaration と
      inheritance path/view の組として定義し、member coverage の exactness、
      既存 `coherence` block で discharge される parent ごとの type-inclusion
      obligation、renamed same-root path を distinct view として保持すること、
      acyclicity failure の `structures.inherit.cycle` を明記した。spec 19 は
      implicit upcast path uniqueness が resolved `inherit` declaration path 上の
      syntactic uniqueness であると明記した。
      `fail_structure_inherit_duplicate_member_coverage_001` を inactive
      duplicate-coverage seed として追加した。renamed-view exposure は有効な
      positive behavior のため renamed-view reject seed は追加せず、既存の
      structure/overload seed と template view-leak seed を他の guard として残す。
      checker/core source semantics は変更していない。

37. **Spec 決定: オーバーロード tie-break と tie の曖昧性(SSA-003, SSA-010, SSA-016, SSA-019)。** [x]
    - §19.6.1 Cases 2-3 を §19.4.3 と整合させる: constraint-strictness と
      non-template-beats-template 規則を明示的に追加するか、純粋な `⊑`
      選択を維持して case の期待結果を訂正するかを決める。§19.4.4 を
      「unique maximal root が無い」場合へ拡張する(equally specific な
      相異なる root を含む)。§19.1 の conflict 規則を return 型に関わらず
      同一シグネチャ宣言へ拡張する。§19.2.3 の antisymmetry 表現を
      closure-equivalence class 上のものへ修正し、§19.6.1 の三重重複文を
      削除する。architecture 05 の tie-breaker 一覧と
      `overload_resolution.md` を同一変更で更新する。
    - 受け入れ条件: §19.6.1 の例と §19.4.3/§19.4.4 の規則が一致する。
      tie-ambiguity の `.miz` seed が
      `fail_resolve_same_signature_return_conflict_001` に加わり、sidecar
      と trace エントリを持つ。
    - 検証: `cargo test -p mizar-test`。
    - 依存: なし。参照: SSA-003, SSA-010, SSA-016, SSA-019。
    - task 37 で完了: spec 19 は Phase B overload selection を、instantiated
      concrete parameter vector 上の通常の `⊑` preorder に保つことを明記した。
      template 宣言制約の厳しさは tie-breaker ではなく、non-template priority は
      concrete vector が相互同値な場合だけ適用される。return type は引き続き
      除外され、ambiguity は非空の maximal-root set が複数の相異なる root を
      含む場合と定義された。同一 argument signature を持つ ordinary definition
      は return type に関わらず declaration conflict であり、§19.6.1 の例も
      規則と一致した。
      Architecture 05 と `overload_resolution.md` も同期した。inactive seed
      `fail_overload_equivalent_roots_ambiguity_001`、
      `fail_overload_template_equivalent_roots_ambiguity_001`、
      `fail_resolve_same_signature_same_return_conflict_001` を追加した。最後の
      seed は resolver declaration-symbol support が現在の different-return
      diagnostic を超えて拡張されるまで inactive のまま。mizar-core task 26 /
      template-audit F7 は、別個の Phase A omitted-template inference determinism
      規則を記録する。checker/core/resolver source semantics は変更していない。

38. **Spec 決定: functorial cluster `for T` の意味論(SSA-004)。** [x]
    - applicability-guard 読み(registration は result の既知の正規化型が
      完全な `for` 型式そのもの、またはその subtype である場所で発火する。
      §17.7.2 の conditional cluster を鏡映)を仕様化し、coherence 義務に
      `is_T(F(args))` premise を追加する。§17.9.3 のエンコード表で
      `for T` が脱落しないよう更新する。spec 17 英日同一変更。
    - 受け入れ条件: `for T` を含む §17.9.3 の全行に guard が現れる。`for`
      型の外で適用された functorial registration を固定する reject-first
      seed を追加する。
    - 検証: `cargo test -p mizar-test`。
    - 依存: なし。参照: SSA-004。
    - task 38 で完了: spec 17 は functorial cluster 末尾の `for` 型式を、
      parameter と attribute を含む既知の正規化結果型全体に対する適用可能性
      guard として定義した。coherence 義務と §17.9.3/§17.9.6 のエンコードは、
      `for` guard を脱落させず結果型 guard を前提として含む。inactive
      `advanced_semantics` seed
      `fail_cluster_functorial_for_guard_001` と traceability row
      `spec.en.17.clusters.functorial_for_guard.semantic` を追加し、registration
      自体は valid のままでも、guarded attribute を欠く same-radix use site では
      後件属性が利用不能であることを固定した。spec 16 の proof-obligation
      summary と spec 23 の registration-node discussion も guarded functorial
      obligation に同期し、詳細 encoding を Chapter 17 に委ねる形にした。
      checker/core source semantics は変更していない。

39. **Spec 決定: property implementation の coherence(SSA-005)。** [x]
    - domain が重なる 2 つの `property S.p means/equals` 実装に coherence
      義務で関係付けることを要求するか、各 property を `inherit` 連結な
      mode family ごとに 1 実装へ制限する。spec 07 §7.4.1/§7.8.2 を英日で
      更新する。
    - 受け入れ条件: 選択した規則が義務の形または制限診断を命名する。
      未カバーの重なりを固定する reject-first seed を追加する。
    - 検証: `cargo test -p mizar-test`。
    - 依存: 35(property 値の供給源が先に確定していること)。参照: SSA-005。
    - task 39 で完了: spec 07 は同じ struct property の重なり合う
      implementation に、受理済み `coherence` correctness condition を要求する。
      grammar は property `means` の existence/uniqueness 後、および property
      `equals` 後に任意の `coherence` block を許すが、重なりがある場合は
      意味上必須である。spec 16 と Appendix A も同期した。inactive seed
      `fail_mode_property_overlap_missing_coherence_001` と traceability row
      `spec.en.07.modes.property_implementation.coherence.semantic`、および
      deferred parser row `spec.en.07.modes.property_implementation.parser` を
      追加した。checker/core source semantics は変更していない。

40. **Spec 契約: registration activation のタイミング(SSA-006)。** [x]
    - §17.1 の item-ordered activation を言語契約として維持し、correctness
      condition の受理が非同期でありうることを明記する: 実装は module を
      pending に保持してよいが、完了した検証 pass が受理するはずの use site
      を拒否してはならない。task-19 の暫定 policy が保守的近似であり
      `mizar-vc`/`mizar-proof` 統合到着時に解除されることを
      `registration_resolution.md` に記録する。
    - 受け入れ条件: spec 17(英日)が非同期受理契約を述べる。
      `registration_resolution.md`(英日)が暫定 policy をその名で記録する。
      `fail_mode_existential_after_declaration_001` がユーザー可視の順序
      エラーのままである。
    - 検証: `cargo test -p mizar-test`。
    - 依存: なし。参照: SSA-006、architecture 04。
    - task 40 で完了: spec 17.1 は item-ordered activation を言語契約として
      維持し、correctness acceptance が proof/kernel/artifact phase から非同期に
      到着してよいことを述べる。Architecture 04 と
      `registration_resolution.md` は、task 19 の accepted input 不在時の動作を、
      完了した pass なら受理する後続 source item への最終 rejection policy ではなく、
      暫定的な保守近似として明記した。既存 inactive seed
      `fail_mode_existential_after_declaration_001` は
      `spec.en.17.clusters.registration_activation_timing.semantic` を通じて
      negative non-retroactive slice を trace する。positive accepted-local
      activation は MC-G020/MC-G021/MC-G025/MC-G026 により deferred のままである。
      checker/core source semantics は変更していない。

41. **Spec 明確化: closure 停止性・矛盾検出サイト・`attr(args)`(SSA-007, SSA-008, SSA-020)。** [x]
    - closure の停止性が制限された adjective 文法に依存すること、adjective
      を term 引数へ拡張するには新しい停止性論証が必要であることを §17.7.1
      に明記する。矛盾する導出属性の closure 時検出を fatal な `cluster`
      診断として仕様化し、§17.7.3 の ATP 時の記述を書き分ける。
      §3.3/§6.2/§17.10 の `attr(args)` を解決する: 宣言・registration の
      story を定義するか、`attribute_ref` から除去する(除去を推奨 —
      cluster への導入は停止性論証を壊す)。spec 03、06、17 英日同一変更。
    - 受け入れ条件: 停止性論証が load-bearing として明記される。
      `fail_cluster_contradictory_consequent_001` が closure 時診断に対応
      する。`attribute_ref` 文法と宣言文法が一致する。
    - 検証: `cargo test -p mizar-test`。
    - 依存: なし。参照: SSA-007, SSA-008, SSA-020。
    - task 41 で完了: spec 17.7.1 と spec 19.2.1 は、引数なしの制限された
      cluster `adjective` 文法を load-bearing な停止性前提とし、architecture 04 は
      saturation bound を成功した truncated semantics ではなく防御的な failure
      diagnostic として扱う。spec 17.7.3 は static contradictory-consequent seed を
      含む矛盾する derived attribute について、closure 時の fatal `cluster` 診断を
      規定した。spec 03/06/Appendix A は `attribute_name(args)` を宣言済み
      parameterized attribute の use-site application として定義しつつ、cluster
      registration adjective から引数リストを除外する。traceability に
      `spec.en.17.clusters.restricted_adjective_grammar.parser` を追加した。
      checker/core source semantics は変更していない。

42. **Spec 明確化: reduction 決定性のシグネチャ(SSA-009)。** [x]
    - §17.6.4 の正規化決定性を(term、in-scope rules、discharged
      side-condition set)の関数として再記述する。複合 specificity を
      pattern subsumption 優先、次に position ごとの guard 比較、残る混合
      ケースは incomparable として FQN tie-break、と定義する。spec 17 英日。
      `registration_resolution.md`(reduction 節)へ反映する。
    - 受け入れ条件: 決定性言明の入力が matching 行の依存と一致する。
      task-18 の挙動(`such` guard は applicability のみ)が仕様本文から
      導出可能である。
    - 検証: `cargo test -p mizar-test`。
    - 依存: なし。参照: SSA-009。
    - task 42 で完了: spec 17 は reduction normalization を、項、スコープ内の
      activated reduction rule、解消済み side-condition 集合の決定的関数として
      定義する。`such` guard は applicability-only evidence であり、specificity
      の入力ではない。rule selection は pattern-first、guard-second で、
      §19.2.3 の位置ごとの比較と、同等、mixed、比較不能ケースの FQN tie-break を
      使う。`registration_resolution.md` も同じ規則を反映する。
      checker/core source semantics は変更していない。

43. **Spec 明確化: 依存 mode の sethood と built-in inhabitation(SSA-013, SSA-014)。** [x]
    - パラメータ化された sethood 義務形
      (`∀params. ∃S. ∀x. (is_T(x, params) → x ∈ S)`)を §7.8.1 に与え、
      §13.4.2 の comprehension gate が instantiated parameters で sethood
      を検査することを明記する。unattributed base について §7.8 と §17.3.4
      を調停し、built-in inhabitation 表(`object`、`set`、struct radix)を
      追加する。テンプレート監査が §17.3.4 に追加した template 実引数
      inhabitation gate と整合させる。spec 07、13、17 英日。gate への参照が
      ある箇所では Ch18 も同期してよい。
    - 受け入れ条件: checker の existential gate(task 20)が全 base-type
      形状に対し決定可能な規則を持つ。sethood の export 状態(module
      interface に含むか否か)が明記される。
    - 検証: `cargo test -p mizar-test`。
    - 依存: なし。参照: SSA-013, SSA-014; テンプレート監査 F2。
    - task 43 で完了: spec 07 は guarded parameterized existence /
      sethood obligation を与え、exported sethood は witness term を export
      しない module-interface semantic fact と明記した。spec 13 は sethood を
      resolved mode と正規化済み argument tuple で検査する。spec 17 は属性付き
      existential registration、built-in `object`/`set`、accepted mode、inhabited
      field 上の constructor witness による bare structure radix、§18.10.2 による
      template 本体内の bare schema type parameter の inhabitation table を追加した。
      spec 18 の type actual も同じ表を使う。既存 inactive
      sethood、existential、template seed は rejection intent を維持し、positive
      source-derived coverage は deferred のまま。checker/core source semantics は
      変更していない。

44. **Spec 明確化: `reconsider` の discharge と曖昧な redefinition target(SSA-015, SSA-017)。** [x]
    - justification を省略した `reconsider` は、narrowing 義務が proof-free
      widening、inheritance/view、cluster-closure、または既に記録済みの
      local type fact で discharge される場合に限り合法で、それ以外は
      justification を求める診断とすることを §8.2 に明記する。複数の元定義
      が該当する `coherence with` 省略 `redefine` に対する「ambiguous
      redefinition target」診断を §19.4.1 に命名する。spec 08 と 19 英日。
    - 受け入れ条件: 両挙動が命名済み診断を持ち、それぞれ 1 件の
      reject-first seed を持つ。
    - 検証: `cargo test -p mizar-test`。
    - 依存: 37(chapter 19 の編集を共有)。参照: SSA-015, SSA-017。
    - task 44 で完了: spec 04/08/15/Appendix A は、justification 省略
      `reconsider` が構文上受理可能である点で一致した。一方 spec 08/22 は、
      その省略形を proof-free widening / inheritance / cluster-closure /
      local-fact discharge に限定し、それ以外は `type.narrowing_requires_proof`
      を要求する。同じ grammar 更新で proof-block `reconsider` も明示した。
      spec 19/22 は、複数の可視先行 root を厳密に精密化する
      `coherence with` 省略に `resolve.ambiguous_redefinition_target` を命名し、
      宣言順、import 順、return type では選ばないことを明記した。2 件の inactive
      advanced-semantics seed が決定を固定する。既存 parser の justification
      省略および proof-block 挙動は parser task 47 の deferred `source_drift` /
      `test_expectation_drift` として残す。checker/core source semantics は
      変更していない。

45. **Checker 整合: オーバーロード tie-break の実装。** [x]
    - `overload_resolution.md` と第 3 波実装(tasks 23-26 の surface:
      template expansion priority、specificity 比較、root selection)を
      task-37 の決定に整合させ、決定された Case 2/3 の結果と tie-ambiguity
      規則の Rust regression を追加する。
    - task 44 の declaration-time `redefine` family target inference も整合
      させる。`coherence with` 省略時に target を推論できるのは、同じ
      symbol kind と arity の可視な先行 ordinary root がちょうど 1 つだけ
      redefinition signature によって厳密に精密化される場合に限る。複数
      root が該当する場合は、declaration order、import order、return type
      で選ばず、失敗 record/diagnostic を保持する。
    - 受け入れ条件: `cargo test -p mizar-checker` が決定済みの結果をカバー
      する。文書化されていない tie-breaker や redefinition target 省略時の
      chooser がコードに残らない。
    - 検証: `cargo test -p mizar-checker`、
      `cargo clippy -p mizar-checker --all-targets -- -D warnings`。
    - 依存: 37, 44。参照: SSA-003, SSA-010, SSA-017; architecture 05。
    - task 45 で完了: `overload_resolution.rs` に、task-37 Case 2/3 の
      explicit-payload regression を追加した。distinct な equivalent
      template-derived root は ambiguous のまま、encoded non-template priority と
      strictly-more-specific template edge は意図した root を選択し、未エンコードの
      ordinary / template-derived equivalence tie は ambiguous に残る。same-root の
      accepted redefinition metadata も distinct-root tie を解消できない。
      `overload_resolution.md`、checker plan/audit、top-level coverage audit は、
      `coherence with` 省略時 target diagnostic が declaration-checking /
      source-extraction producer 側の挙動であると記録した。この data layer は
      already-bound redefinition payload のみを受け取り、missing/deferred/rejected
      producer record を保持する。inactive `.miz` overload/redefinition seed と
      deferred traceability row は MC-G027/MC-G030 の下で変更していない。

46. **Checker 整合: closure の矛盾検出と停止性規則。** [x]
    - task-41/42 の決定を `cluster_trace.md` と `registration_resolution.md`
      (英日)にエンコードし、task 16-18 実装を整合させる: closure 時矛盾を
      fatal 診断(severity は §17.7.3 準拠)とし、防御的 saturation bound の
      傍らに文法ベース停止性 note を置き、訂正済み reduction 決定性
      シグネチャを反映する。
    - 受け入れ条件: module spec が新しい仕様本文を引用する。既存の
      determinism suite(task 30)を side-condition-set 依存へ拡張する。
    - 検証: `cargo test -p mizar-checker`、
      `cargo clippy -p mizar-checker --all-targets -- -D warnings`。
    - 依存: 41, 42。参照: SSA-007, SSA-008, SSA-009。
    - task 46 で完了: `cluster_trace.rs` は explicit closure contradiction が
      checker-local contradiction class、error severity、fatal recovery、incomplete
      closure status を持ち、contradictory generated fact を degraded export しない
      ことを assert する。determinism suite は explicit-payload reduction trace
      snapshot を追加し、discharged guard order の同値性が canonical であること、
      discharged `such` evidence の変更が trace identity を変えること、strategy-audit
      key は変わらず `such` specificity を含まないことを固定する。
      `cluster_trace.md`、`registration_resolution.md`、checker plan/audit、top-level
      coverage audit は task-41/42 の spec decision を引用する。source-derived
      normalization result、source-derived cluster contradiction extraction、artifact/cache
      replay、active `.miz` semantic fixture は MC-G020/MC-G021/MC-G023/MC-G030 の下で
      deferred のままである。

47. **Checker 整合: existential gate と activation 契約。** [x]
    - task-20 の existential gate を task-43 の built-in inhabitation 表と
      パラメータ化 sethood 形に整合させ、task-40 の activation 契約を暫定
      policy が近似する目標挙動として `registration_resolution.md` に記録
      する。
    - task 44 の justification 省略 `reconsider` handling も整合させる:
      proof search や暗黙の `by` は使わず、proof-free widening、一意な
      inheritance/view evidence、active cluster closure、または既に記録済みの
      local type fact が各 target obligation を discharge する場合だけ受理し、
      それ以外の failed site は `type.narrowing_requires_proof` を報告する。
    - 受け入れ条件: `mode M is set`、built-in、struct radix に対する gate
      挙動が決定済みの表と一致し、Rust regression を持つ。justification
      省略 `reconsider` は parser-only rejection ではなく semantic E0102 gate
      を維持する。
    - 検証: `cargo test -p mizar-checker`、
      `cargo clippy -p mizar-checker --all-targets -- -D warnings`。
    - 依存: 40, 43, 44。参照: SSA-006, SSA-013, SSA-014, SSA-015。
    - task 47 で完了: `registration_resolution.rs` は explicit
      base-shape inhabitation evidence を unattributed な exact pattern match に
      限って受理し、built-in `object`/`set`、accepted mode tuple、zero-field
      または fully guarded structure constructor、schema type parameter coverage
      を扱う。attributed gate は引き続き active existential candidate を要求し、
      hidden、non-consumable、不完全、または mismatched guard evidence は verified
      fact を seed せず gate を block または reject する。`type_checker.rs` は
      explicit narrowing request と omitted narrowing request を区別し、justification
      省略 `reconsider` は supplied consumable proof-free evidence が target を既に
      discharge する場合だけ受理し、それ以外では implicit obligation を作らず
      `type.narrowing_requires_proof` を報告する。`registration_resolution.md`、
      `type_checker.md`、checker plan/audit、top-level coverage audit は
      task-40/43/44 contract を記録する。source-derived base-shape extraction、
      positive accepted-local activation、source-derived omitted-`reconsider`
      parser/extraction coverage、artifact、active `.miz` fixture は
      MC-G018/MC-G020/MC-G021/MC-G025/MC-G026/MC-G030 の下で deferred のまま。

48. **Reserve source declaration producer seam。** [x]
    - 既存 reserve-only builtin declaration bridge を `type_checker` の
      checker-owned かつ syntax-free な producer seam へ昇格する。upstream が
      抽出した source/module identity、reserve source range、binding
      spelling/range、bare builtin `set` / `object` type-expression
      spelling/range/head を消費し、checker-owned `BindingEnv` と
      `DeclarationCheckingOutput` を構築する。active `mizar-test` runner が
      引き続き `TypedAst`、`ResolvedTypedAst`、summary-readiness、binder-only core
      context check を組み立てられるよう deterministic typed-site id を公開する。
    - 受け入れ条件: `mizar-checker` は direct `mizar-syntax` dependency を持たない。
      non-builtin declaration、attribute、promoted diagnostic slice 外の unsupported
      mode/structure payload、term、formula、coercion、overload evidence、fact、proof skeleton、
      CoreIr/ControlFlowIr/VC/proof payload、新しい active `.miz` coverage は
      MC-G020 の下で deferred のまま。active `type-elaboration` result は
      byte-stable のまま。
    - 検証: `cargo test -p mizar-checker`, `cargo test -p mizar-test`。
    - 依存: task 47; 外部 source family は MC-G020 のまま。参照:
      Step 5 source-derived semantic bridge; mizar-test task 10。
    - task 48 で完了: `type_checker.rs` は対応済み reserve-only builtin slice
      用の `SourceReserveDeclarationBridge`、`SourceReserveBindingInput`、
      `SourceReserveDeclarationHandoff` を公開する。`mizar-test` は引き続き real
      `.miz` AST extraction と lower-stage runner gating を所有し、その後この
      seam を通じて checker handoff production を委譲してから既存の
      `TypedAst` / `ResolvedTypedAst` / core readiness assertion を行う。
      `.miz` expectation、public diagnostic code、CoreIr/ControlFlowIr/VC/proof
      row、より広い semantic payload family は昇格していない。

49. **監査 corpus の活性化と task-29 record の改訂。** [ ]
    - `advanced_semantics`/`formula_statement` runner、property-implementation
      parser support、source-to-checker payload 抽出(mizar-test runner 成長 +
      MC-G020/MC-G021/MC-G023/MC-G027、および task-39 seed については
      MC-G030/property-implementation payload extraction)が到着したら、意味論監査 fixture
      16 件、task-35 constructor-property seed、task-36 duplicate-coverage
      seed、task-37 ordinary/template-derived equivalent-root ambiguity seed、
      task-38 functorial-`for` guard seed、task-39 property-overlap coherence
      seed、task-44 omitted-`reconsider` / ambiguous-redefinition-target seed を活性化する。
      declaration-symbol runner が該当 resolver diagnostic を support した時点で
      task-37 same-return signature-conflict seed も活性化する。task-29 の
      deferred corpus record を監査由来の requirement id を指す(または
      置き換えられる)よう改訂する。
    - 受け入れ条件: `mizar-test` plan が fixture を active と表示し plan
      error が 0 件。deferred record が二重計上されない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: 35-44 の決定; 外部: mizar-test の runner 対応。参照:
      semantic_spec_audit.md「Adversarial Corpus」。

50. **Source-derived attributed reserve evidence-gap bridge.** [x]
    - task 48 の reserve source declaration seam を、resolver `SymbolEnv` に
      すでに存在する attribute symbol に限り、builtin `set` / `object`
      reserve type-expression 上の source-derived attribute chain を受け取れる
      最小範囲で拡張する。
    - 受け入れ条件: same-module の source-derived attribute は checker-owned
      `TypeExpressionInput` に保存され、declaration checking で normalize される。
      attributed reserve declaration は real existential registration /
      evidence-query seam が存在するまで
      `checker.declaration.deferred.evidence_query` の active fail case に
      留める。imported attribute symbol、non-builtin head、promoted diagnostic slice 外の
      unsupported mode / structure payload、term、formula、proof skeleton、CoreIr / ControlFlowIr / VC /
      proof payload、successful attributed declaration は MC-G020 / MC-G021 /
      MC-G026 の下で deferred のままにする。
    - 検証: `cargo test -p mizar-checker`、`cargo test -p mizar-test`。
    - 依存: task 48。external evidence は MC-G021 / MC-G026 のまま。参照:
      Step 5 source-derived semantic bridge、mizar-test task 10、spec 03
      type expression、spec 17 existential gate。
    - task 50 で完了: `type_checker.rs` は syntax-free reserve bridge 上で
      source-derived attribute payload を受け取り、existential evidence を捏造せず
      declaration に `MissingEvidenceQuery` を付ける。`mizar-test` は checker
      diagnostic まで到達する same-module attributed reserve の active fail
      fixture を追加する一方、既存の import-backed attributed reserve fixture は
      imported symbol が active runner の `SymbolEnv` に入るまで、より広い
      extraction gap のまま保持する。

51. **Source-derived local mode reserve expansion-gap bridge.** [x]
    - task 48 の reserve source declaration seam を、type argument や source
      attribute を持たず、unique な same-module `LocalSource` mode symbol に
      解決される source-derived reserve type head だけを受け取るところまで拡張する。
    - 受け入れ条件: checker-owned bridge は symbol head が current module の
      local source 由来の exact `SymbolKind::Mode` entry であることを検証し、その後
      declaration checking は real mode-expansion payload extraction が未実装なので
      既存の `checker.type.external.mode_expansion_payload` diagnostic に到達する。
      imported mode、mode argument、unresolved/ambiguous head、mode
      expansion extraction、term、formula、CoreIr / ControlFlowIr / VC / proof
      payload、successful local-mode reserve declaration は MC-G020 の下で
      deferred のままにする。
    - 検証: `cargo test -p mizar-checker`、`cargo test -p mizar-test`。
    - 依存: task 48。external mode expansion は MC-G014 / MC-G020 のまま。参照:
      Step 5 source-derived semantic bridge、mizar-test task 10、spec 03 type
      expression、spec 07 mode、spec 17 accepted-mode inhabitation evidence。
    - task 51 で完了: `type_checker.rs` は syntax-free reserve bridge 上で
      local source-backed mode head を検証し、raw syntax から unfold せず既存の
      missing mode-expansion diagnostic を保持する。`mizar-test` は same-module
      local-mode reserve の active fail fixture を追加し、imported mode、
      argument-bearing mode head は広い extraction gap のままにする。

52. **Source-derived local structure reserve evidence-gap bridge.** [x]
    - task 48 の reserve source declaration seam を、type argument や source
      attribute を持たず、unique な same-module `LocalSource` structure symbol に
      解決される source-derived reserve type head だけを受け取るところまで拡張する。
    - 受け入れ条件: checker-owned bridge は symbol head が current module の
      local source 由来の exact `SymbolKind::Structure` entry であることを検証し、
      その reserved-variable declaration に `MissingEvidenceQuery` を付ける。
      real base-shape / constructor-witness evidence extraction が未実装なので
      declaration checking は `checker.declaration.deferred.evidence_query` に到達する。
      imported structure、structure argument、task 53 の diagnostic slice 外の attributed
      structure head、successful local-structure reserve declaration、structure field /
      default payload extraction、CoreIr / ControlFlowIr / VC / proof payload、より広い
      semantic pass coverage は MC-G020 / MC-G026 の下で deferred のままにする。
    - 検証: `cargo test -p mizar-checker`、`cargo test -p mizar-test`。
    - 依存: task 48。external base-shape evidence は MC-G020 / MC-G026 のまま。
      参照: Step 5 source-derived semantic bridge、mizar-test task 10、spec 03 type
      expression、spec 05 structure、spec 17 base-shape inhabitation evidence。
    - task 52 で完了: `type_checker.rs` は syntax-free reserve bridge 上で local
      source-backed structure head を検証し、symbol だけから structure inhabitation を
      推論せず missing evidence-query diagnostic を保持する。`mizar-test` は実 field を持つ
      local `struct` を使った same-module local-structure reserve の active fail fixture を追加し、
      imported structure と argument-bearing structure head は広い extraction gap のままにする。

53. **Source-derived attributed local structure reserve evidence-gap bridge.** [x]
    - task 48 の reserve source declaration seam を、type argument を持たない unique な
      same-module `LocalSource` structure reserve head に source-derived no-argument
      attribute payload を付けるところまで拡張する。
    - 受け入れ条件: checker-owned bridge は symbol head の exact local
      `SymbolKind::Structure` provenance を検証し、後続 task 54 の diagnostic slice 外の
      attributed local mode head は広い extraction gap のままにし、attributed
      local-structure reserved-variable declaration に
      `MissingEvidenceQuery` を付ける。full normalized attributed type の real existential
      evidence は未実装なので、`checker.declaration.deferred.evidence_query` に到達する。
      imported attribute / structure、attribute argument、qualified attribute disambiguation、
      structure argument、successful attributed structure reserve declaration、structure
      field/default/base-shape extraction、CoreIr / ControlFlowIr / VC / proof payload、
      より広い semantic pass coverage は MC-G020 / MC-G026 の下で deferred のままにする。
    - 検証: `cargo test -p mizar-checker`、`cargo test -p mizar-test`。
    - 依存: tasks 48、50、52。external full attributed-type existential evidence は
      MC-G020 / MC-G026 のまま。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 03 type expression、spec 05 structure、spec 17
      existential と base-shape inhabitation evidence。
    - task 53 で完了: `type_checker.rs` は syntax-free reserve bridge 上で local
      structure head に限って same-module source attribute を受け入れる。attributed
      local mode の diagnostic slice は後続 task 54 が所有する。`mizar-test` は same-module attributed
      local-structure reserve の active fail fixture を追加し、imported / argument-bearing
      form は広い extraction gap のままにする。

54. **Source-derived attributed local mode reserve expansion-gap bridge.** [x]
    - task 48 の reserve source declaration seam を、type argument を持たない unique な
      same-module `LocalSource` mode reserve head に source-derived no-argument
      attribute payload を付けるところまで拡張する。
    - 受け入れ条件: checker-owned bridge は symbol head の exact local
      `SymbolKind::Mode` provenance を検証し、same-module source-derived attribute を保持し、
      real mode expansion が存在するまでは `MissingEvidenceQuery` を付けず、real
      mode-expansion payload extraction が未実装なので
      `checker.type.external.mode_expansion_payload` に到達する。imported attribute / mode、
      attribute argument、qualified attribute disambiguation、mode argument、successful
      attributed mode reserve declaration、real mode expansion、accepted-mode / base evidence、
      fully expanded attributed type の existential evidence、CoreIr / ControlFlowIr / VC /
      proof payload、より広い semantic pass coverage は MC-G014 / MC-G020 / MC-G026 の下で
      deferred のままにする。
    - 検証: `cargo test -p mizar-checker`、`cargo test -p mizar-test`。
    - 依存: tasks 48、50、51。external mode-expansion と existential evidence は
      MC-G014 / MC-G020 / MC-G026 のまま。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 03 type expression、spec 07 mode、spec 17 existential と
      accepted-mode inhabitation evidence。
    - task 54 で完了: `type_checker.rs` は syntax-free reserve bridge 上で local mode
      head への same-module source attribute を受け入れ、missing existential evidence を
      evidence-query diagnostic として扱わない。`mizar-test` は same-module attributed
      local-mode reserve の active fail fixture を追加し、imported / argument-bearing form は
      広い extraction gap のままにする。

55. **Source-derived bare local mode expansion bridge.** [x]
    - active type-elaboration source bridge を、unique な same-module `LocalSource`
      no-argument mode definition の bare reserve use について real `ModeExpansion` を
      生成する最小範囲まで拡張する。対象 mode definition は unrecovered source definition
      として reserve use より前に現れ、definition-local parameter / assumption context を
      持たず、RHS が bare builtin `set` / `object` でなければならない。
    - 受け入れ条件: runner は expansion を `SurfaceAst` から抽出し、checker-owned
      syntax-free reserve seam に渡す。結果として bare local-mode reserve declaration は
      `BindingEnv`、`DeclarationChecker`、`TypedAst`、`ResolvedTypedAst`、
      summary-readiness、binder-only `CoreContext` まで active pass case になる。runner は
      attributed local-mode reserve use、mixed attributed/bare local-mode source、
      attributed mode RHS、imported / argument-bearing / parameterized / contextual mode、
      unresolved / ambiguous head、non-reserve declaration について mode expansion を渡さない。
      これらの family は既存の missing-expansion または broader extraction gap に残す。
    - 検証: `cargo test -p mizar-checker`、`cargo test -p mizar-test`。
    - 依存: tasks 48、51、54。より広い mode expansion と existential evidence は
      MC-G014 / MC-G020 / MC-G026 のまま。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 03 type expression、spec 07 mode、spec 17 base-shape
      inhabitation evidence。
    - task 55 で完了: `mizar-test` は narrow な bare local-mode reserve slice について
      real AST-derived `ModeExpansion` を抽出し、checker source reserve seam は evidence を
      捏造せず explicit mode-expansion payload を受け入れる。local mode expansion bridge の
      new active pass fixture を追加し、attributed / mixed / attributed-RHS case は
      missing expansion または evidence gap で fail closed のままにする。

56. **Source-derived local mode expansion chain bridge.** [x]
    - task-55 bridge を、same-module bare local-mode reserve head が、accepted bare
      builtin `set` / `object` RHS expansion を持つ preceding same-module no-argument
      local mode へ expand する場合の real chained `ModeExpansion` payload 生成まで
      最小限拡張する。
    - 受け入れ条件: runner は checker-owned reserve seam の前に source-derived
      expansion を両方挿入する。active pass fixture は `B -> A -> set` と
      `B -> A -> object` を cover し、active fail fixture は attributed dependency で
      chain 全体が withheld され missing mode-expansion diagnostic に到達することを示す。
      forward reference、ambiguous / imported / cyclic dependency、accepted dependency
      expansion を欠く partial chain、attributed use / RHS、argument、parameterized /
      contextual definition、CoreIr / ControlFlowIr / VC / proof payload、より広い
      semantic pass coverage は deferred のままにする。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、51、54、55。より広い mode expansion と existential evidence は
      MC-G014 / MC-G020 / MC-G026 のまま。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 03 type expression、spec 07 mode、spec 17 base-shape
      inhabitation evidence。
    - task 56 で完了: `mizar-test` は narrow な one-edge source-derived local-mode
      expansion chain を抽出し、`B -> A -> set` と `B -> A -> object` の active pass
      coverage を追加する。attributed dependency chain は CoreIr、ControlFlowIr、VC、
      proof payload へ昇格せず checker missing mode-expansion diagnostic で fail closed のままにする。

57. **Source-derived local mode structure-RHS evidence-gap bridge.** [x]
    - task-55 bridge を、same-module bare local-mode reserve head が preceding
      same-module no-argument local structure head へ expand する場合の real
      `ModeExpansion` payload 生成まで最小限拡張する。checker はその expansion を消費し、
      structure base-shape / constructor-witness evidence 欠落で fail closed しなければならない。
    - 受け入れ条件: checker unit coverage は `Mode -> LocalStruct` が real
      `ModeExpansion` を消費し、`checker.type.external.mode_expansion_payload` を出さず、
      declaration を `MissingEvidenceQuery` 付き partial にし、verified fact を export
      しないことを示す。runner unit coverage は same-module local structure RHS extraction
      が terminal expansion payload として受け入れられることを示す。active
      `type_elaboration` fail fixture は real `.miz` source path を
      `type_elaboration.checker.checker.declaration.deferred.evidence_query` で cover する。
      imported、argument-bearing、attributed、ambiguous、cyclic、forward-reference
      structure RHS はこの slice の外に残す。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、52、55。structure base-shape evidence とより広い mode expansion は
      MC-G020 / MC-G026 のまま。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 03 type expression、spec 05 structure、spec 07 mode、
      spec 17 base-shape inhabitation evidence。
    - task 57 で完了: `mizar-test` は RHS が same-module local structure head である
      real AST-derived local-mode expansion を抽出し、`mizar-checker` は expanded reserve
      declaration を missing expansion-payload diagnostic ではなく既存の missing
      evidence-query diagnostic に route する。positive structure acceptance、
      base-shape / constructor-witness extraction、imported / argument-bearing /
      attributed structure RHS、CoreIr、ControlFlowIr、VC、proof payload、より広い
      semantic pass coverage は deferred のままにする。

58. **Source-derived local mode attributed-builtin RHS evidence-gap bridge.** [x]
    - task-55 bridge を、same-module bare local-mode reserve head が RHS に
      attributed bare builtin `set` / `object` type を持つ preceding same-module
      no-argument local mode へ expand する場合の real `ModeExpansion` payload 生成まで
      最小限拡張する。
    - 受け入れ条件: checker unit coverage は `Mode -> marked set` が real
      `ModeExpansion` を消費し、`checker.type.external.mode_expansion_payload` を出さず、
      normalized attribute を保持し、declaration を `MissingEvidenceQuery` 付き partial
      にし、verified fact を export しないことを示す。runner unit coverage は direct
      attributed builtin RHS extraction が terminal expansion payload として受け入れられ、
      attributed RHS で終わる chain dependency は withheld のままであることを示す。
      既存の active `type_elaboration` attributed-RHS fail fixture は real `.miz`
      source path を `type_elaboration.checker.checker.declaration.deferred.evidence_query`
      で cover するよう更新する。attributed reserve head、mixed attributed/bare
      reserve use、imported / argument-bearing attribute/mode、attributed local
      structure RHS、attributed RHS 経由の chain promotion、successful attributed-mode
      declaration、existential evidence はこの slice の外に残す。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、50、54、55。full attributed-type existential evidence とより広い
      mode expansion は MC-G020 / MC-G026 のまま。参照: Step 5 source-derived
      semantic bridge、mizar-test task 10、spec 03 type expression、spec 07 mode、
      spec 17 attributed-type evidence。
    - task 58 で完了: `mizar-test` は RHS が attributed builtin head である real
      AST-derived local-mode expansion を抽出し、`mizar-checker` は expanded reserve
      declaration を missing expansion-payload diagnostic ではなく既存の missing
      evidence-query diagnostic に route する。positive attributed-type acceptance、
      existential evidence extraction、attributed reserve head、attributed-RHS chain、
      CoreIr、ControlFlowIr、VC、proof payload、より広い semantic pass coverage は
      deferred のままにする。

59. **Source-derived attributed local mode reserve evidence-gap bridge.** [x]
    - task-55 bridge を、same-module attributed local-mode reserve head について、
      unique な preceding same-module no-argument mode definition が direct bare builtin
      `set` / `object` RHS を持ち、同じ mode が同じ bridge input 内で bare reserve head
      としても使われていない場合の real `ModeExpansion` payload 生成まで最小限拡張する。
    - 受け入れ条件: checker unit coverage は real `Mode -> set` expansion を持つ
      `marked Mode` が `checker.type.external.mode_expansion_payload` を出さず、
      normalized attribute を保持し、declaration を `MissingEvidenceQuery` 付き partial
      にし、verified fact を export しないことを示す。runner unit coverage は single
      attributed local-mode reserve use が real direct bare-builtin expansion を受け取り、
      同じ mode の mixed bare/attributed use は引き続き expansion を withheld することを示す。
      既存の active `type_elaboration` attributed local-mode reserve fixture は real `.miz`
      source path を `type_elaboration.checker.checker.declaration.deferred.evidence_query`
      で cover するよう更新する。mixed bare/attributed reserve use、imported /
      argument-bearing attribute/mode、attributed dependency、chain、structure RHS、
      attributed RHS、successful attributed-mode declaration、existential evidence はこの
      slice の外に残す。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、50、54、55。full attributed-type existential evidence とより広い
      mode expansion は MC-G020 / MC-G026 のまま。参照: Step 5 source-derived
      semantic bridge、mizar-test task 10、spec 03 type expression、spec 07 mode、
      spec 17 attributed-type evidence。
    - task 59 で完了: `mizar-test` は同じ mode に mixed bare reserve use がない
      same-module attributed reserve head について real AST-derived direct bare-builtin
      local-mode expansion を抽出し、`mizar-checker` は expanded attributed reserve
      declaration を missing expansion-payload diagnostic ではなく既存の missing
      evidence-query diagnostic に route する。positive attributed-type acceptance、
      existential evidence extraction、mixed attributed/bare use、attributed dependency
      や chain、CoreIr、ControlFlowIr、VC、proof payload、より広い semantic pass coverage は
      deferred のままにする。

60. **Source-derived attributed local mode structure-RHS evidence-gap bridge.** [x]
    - task-57 structure-RHS bridge を、same-module attributed local-mode reserve head
      に対して real `ModeExpansion` payload を生成する最小範囲だけ拡張する。mode
      definition は unique / unrecovered / preceding / no-argument、definition-local
      context なしで、direct same-module local structure RHS を持ち、その structure
      definition は unique / unrecovered で mode definition より前に現れる必要がある。
      同じ bridge input 内で同じ mode が bare reserve head として使われていてはならない。
    - Acceptance: checker unit coverage は、real `Mode -> LocalStruct` expansion を持つ
      `marked Mode` が `checker.type.external.mode_expansion_payload` を出さず、
      normalized attribute を保持し、declaration を `MissingEvidenceQuery` 付き partial
      にし、verified fact を export しないことを証明する。runner unit coverage は、single
      attributed local-mode reserve use が real direct structure-RHS expansion を受け取り、
      mixed bare/attributed use、attributed structure-RHS chain、cached direct
      structure-RHS dependency は expansion を withheld し続けることを証明する。新しい active
      `type_elaboration` fail fixture は real `.miz` source path を
      `type_elaboration.checker.checker.declaration.deferred.evidence_query` で cover し、
      追加の active fail fixture は mixed structure-RHS と attributed structure-RHS chain
      exclusion を missing-expansion diagnostic で cover する。imported / argument-bearing attribute/mode/structure、dependency、chain、
      attributed structure RHS、attributed-builtin RHS、successful attributed / structure
      declaration、base-shape / existential evidence は slice 外に残す。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、50、52、53、57、59。structure base-shape evidence、full
      attributed-type existential evidence、より広い mode expansion は MC-G020 / MC-G026 のまま。
      参照: Step 5 source-derived semantic bridge、mizar-test task 10、spec 03 type
      expressions、spec 05 structures、spec 06 attributes、spec 07 modes、spec 17
      attributed-type evidence。
    - task 60 で完了: `mizar-test` は同じ mode に mixed bare reserve use がない
      same-module attributed reserve head について real AST-derived direct local-structure
      RHS expansion を抽出し、`mizar-checker` は expanded attributed reserve declaration を
      missing expansion-payload diagnostic ではなく既存の missing evidence-query diagnostic へ
      route する。positive attributed / structure acceptance、base-shape /
      constructor-witness extraction、existential evidence extraction、mixed attributed/bare
      use、dependency / chain、CoreIr、ControlFlowIr、VC、proof payload、より広い semantic
      pass coverage は deferred のままにする。

61. **Source-derived attributed local mode attributed-builtin-RHS evidence-gap bridge.** [x]
    - task-58 attributed-builtin RHS bridge を、same-module attributed local-mode reserve
      head に対して real `ModeExpansion` payload を生成する最小範囲だけ拡張する。mode
      definition は unique / unrecovered / preceding / no-argument、definition-local
      context なしで、direct attributed builtin `set` / `object` RHS を持つ必要がある。
      同じ bridge input 内で同じ mode が bare reserve head として使われていてはならない。
    - Acceptance: checker unit coverage は、real `Mode -> marked set` expansion を持つ
      `marked Mode` が `checker.type.external.mode_expansion_payload` を出さず、reserve
      head と RHS の normalized attribute を保持し、declaration を
      `MissingEvidenceQuery` 付き partial にし、verified fact を export しないことを証明する。
      runner unit coverage は、single attributed local-mode reserve use が real direct
      attributed-builtin RHS expansion を受け取り、mixed bare/attributed use と attributed
      RHS へ至る dependency chain は expansion を withheld し続けることを証明する。新しい
      active `type_elaboration` fail fixture は real `.miz` source path を
      `type_elaboration.checker.checker.declaration.deferred.evidence_query` で cover し、
      追加の active fail fixture は mixed attributed-RHS と attributed-RHS chain exclusion を
      missing-expansion diagnostic で cover する。imported / argument-bearing attribute/mode、
      dependency、chain、structure RHS、attributed structure RHS、successful attributed
      declaration、existential evidence extraction、CoreIr / ControlFlowIr / VC / proof payload は
      slice 外に残す。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、50、54、55、58、59。full attributed-type existential evidence と
      より広い mode expansion は MC-G020 / MC-G026 のまま。参照: Step 5 source-derived
      semantic bridge、mizar-test task 10、spec 03 type expressions、spec 06 attributes、
      spec 07 modes、spec 17 attributed-type evidence。
    - task 61 で完了: `mizar-test` は同じ mode に mixed bare reserve use がない
      same-module attributed reserve head について real AST-derived direct attributed-builtin
      RHS expansion を抽出し、`mizar-checker` は expanded attributed reserve declaration を
      missing expansion-payload diagnostic ではなく既存の missing evidence-query diagnostic へ
      route する。positive attributed acceptance、existential evidence extraction、mixed
      attributed/bare use、dependency / chain、CoreIr、ControlFlowIr、VC、proof payload、より広い
      semantic pass coverage は deferred のままにする。

62. **Source-derived local mode structure-RHS chain evidence-gap bridge を追加する。** [x]
    - task-56 chain producer を、bare same-module local-mode reserve head `A` に限って
      拡張する。`A` は unique / unrecovered / no-argument / preceding な `A is B`
      mode definition を持ち、`B` は unique / unrecovered / no-argument same-module
      local mode で、その preceding definition が `B is LocalStruct` でなければならない。
      unique / unrecovered / same-module local structure definition は `B` より前にあり、
      `B` は `A` より前、`A` は reserve use より前にある必要がある。両方の mode
      definition は definition-local context を持ってはならない。
    - Acceptance: runner unit coverage は同じ `SurfaceAst` から real source-derived
      `B -> LocalStruct` と `A -> B` expansion payload の両方が抽出されることを証明する。
      cached direct structure-RHS payload はこの one-edge chain を支えてよいが、deeper
      chain は withheld のままにする。新しい active `type_elaboration` fail fixture は
      real `.miz` source path を cover し、`checker.type.external.mode_expansion_payload`
      ではなく `type_elaboration.checker.checker.declaration.deferred.evidence_query` に到達する。
      checker は verified fact を emit せず、positive structure acceptance は deferred のまま。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、52、56、57。structure base-shape / constructor-witness evidence と
      より広い mode expansion は MC-G020 / MC-G026 のまま。参照: Step 5
      source-derived semantic bridge、mizar-test task 10、spec 03 type expressions、
      spec 05 structures、spec 07 modes、spec 17 evidence。
    - task 62 で完了: `mizar-test` は same-module local structure RHS で終端する
      one-edge bare local-mode chain を real AST-derived expansion として抽出し、
      `mizar-checker` は expanded reserve declaration を既存の missing evidence-query
      diagnostic へ route する。imported / ambiguous symbol、argument、contextual /
      parameterized definition、attributed root、attributed/deeper chain、positive
      structure acceptance、CoreIr、ControlFlowIr、VC、proof payload、より広い semantic
      pass coverage は deferred のまま。

63. **Source-derived local mode attributed-builtin-RHS chain evidence-gap bridge を追加する。** [x]
    - task-56 chain producer を、bare same-module local-mode reserve head `A` に限って
      拡張する。`A` は unique / unrecovered / no-argument / preceding な `A is B`
      mode definition を持ち、`B` は unique / unrecovered / no-argument same-module
      local mode で、その preceding definition が direct attributed builtin `set` /
      `object` RHS を持つ必要がある。`B` は `A` より前、`A` は reserve use より前に
      あり、両方の mode definition は definition-local context を持たず、RHS attributes は
      argument-free same-module attribute symbol に resolve しなければならない。
    - Acceptance: runner unit coverage は同じ `SurfaceAst` から real source-derived
      `B -> marked set` と `A -> B` expansion payload の両方が抽出されることを証明する。
      cached direct attributed-builtin-RHS payload はこの one-edge chain を支えてよいが、
      deeper chain と attributed root は withheld のままにする。新しい active
      `type_elaboration` fail fixture は real `.miz` source path を cover し、
      `checker.type.external.mode_expansion_payload` ではなく
      `type_elaboration.checker.checker.declaration.deferred.evidence_query` に到達する。
      checker は verified fact を emit せず、positive attributed-type acceptance は deferred のまま。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、50、56、58、61。full attributed-type existential evidence と
      より広い mode expansion は MC-G020 / MC-G026 のまま。参照: Step 5
      source-derived semantic bridge、mizar-test task 10、spec 03 type expressions、
      spec 06 attributes、spec 07 modes、spec 17 evidence。
    - task 63 で完了: `mizar-test` は attributed builtin RHS で終端する one-edge bare
      local-mode chain を real AST-derived expansion として抽出し、`mizar-checker` は
      expanded reserve declaration を既存の missing evidence-query diagnostic へ route する。
      imported / ambiguous symbol、attribute / mode argument、contextual /
      parameterized definition、attributed root、attributed/deeper chain、positive
      attributed-type acceptance、CoreIr、ControlFlowIr、VC、proof payload、より広い semantic
      pass coverage は deferred のまま。

64. **Source-derived attributed local mode bare-builtin chain evidence-gap bridge を追加する。** [x]
    - task-59 attributed-root producer を、`reserve z for marked A` に限定して拡張する。
      `A` は preceding definition が `A is B` である unique / unrecovered /
      no-argument / same-module mode、`B` は preceding definition が direct bare builtin
      `set` / `object` RHS を持つ unique / unrecovered / no-argument / same-module
      mode でなければならない。`B` は `A` より前に、`A` は reserve use より前に現れ、
      両方の mode definition は definition-local context を持たず、`A` は同じ bridge
      input 内で bare reserve head としても使われておらず、`B` は attributed reserve
      head として使われていないこと。
    - Acceptance: runner unit coverage は同じ `SurfaceAst` から real source-derived
      `B -> set` と `A -> B` expansion payload の両方、および attributed reserve head が
      抽出されることを証明する。cached direct bare-builtin dependency payload はこの
      one-edge attributed-root chain に使ってよいが、deeper chain、attributed dependency、
      `A` の mixed bare/attributed use、dependency が local structure RHS または
      attributed builtin RHS に終端する attributed root は引き続き withheld する。新しい
      active `type_elaboration` fail fixture は real `.miz` source path を cover し、
      `checker.type.external.mode_expansion_payload` ではなく
      `type_elaboration.checker.checker.declaration.deferred.evidence_query` に到達する。
      checker は verified fact を出さず、positive attributed-type acceptance は deferred のまま。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、50、55、56、59。full attributed-type existential evidence と
      broader mode expansion は MC-G020/MC-G026 のまま。参照: Step 5 source-derived
      semantic bridge、mizar-test task 10、spec 03 type expressions、spec 06 attributes、
      spec 07 modes、spec 17 evidence。

65. **Source-derived attributed local mode structure-RHS chain evidence-gap bridge を追加する。** [x]
    - task-64 attributed-root chain producer を、`reserve z for marked A` に限定して拡張する。
      `A is B`、`B is LocalStruct` であり、`LocalStruct` は `B` より前に現れる unique /
      unrecovered / same-module structure definition、両方の mode definition は unique /
      unrecovered / same-module / no-argument で definition-local context を持たず、source
      order は `LocalStruct -> B -> A -> reserve` でなければならない。`A` は同じ bridge
      input 内で bare reserve head としても使われておらず、`B` は attributed reserve head
      として使われていないこと。
    - Acceptance: runner unit coverage は同じ `SurfaceAst` から real source-derived
      `B -> LocalStruct` と `A -> B` expansion payload の両方、および attributed reserve
      head が抽出されることを証明する。cached direct structure-RHS dependency payload はこの
      one-edge attributed-root chain に使ってよいが、attributed-builtin terminal dependency、
      deeper chain、attributed dependency、`A` の mixed bare/attributed use、
      imported / ambiguous symbol、argument、contextual / parameterized / recovered
      definition は withheld のままにする。既存 active structure-RHS chain `.miz` fixture は
      `checker.type.external.mode_expansion_payload` から
      `type_elaboration.checker.checker.declaration.deferred.evidence_query` へ移る。
      checker は verified fact を出さず、positive structure / attributed-type acceptance は
      deferred のまま。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、50、56、60、62、64。structure base-shape /
      constructor-witness evidence、full attributed-type existential evidence、broader mode
      expansion は MC-G020/MC-G026 のまま。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 03 type expressions、spec 05 structures、spec 06 attributes、
      spec 07 modes、spec 17 evidence。

66. **Source-derived attributed local mode attributed-builtin-RHS chain evidence-gap bridge を追加する。** [x]
    - task-64/task-65 attributed-root chain producer を、`reserve z for marked A` に限定して
      拡張する。`A is B`、`B is marked set` または `B is marked object` であり、RHS
      attribute は argument-free same-module attribute symbol に resolve し、両方の mode
      definition は unique / unrecovered / same-module / no-argument で definition-local
      context を持たず、source order は `B -> A -> reserve` でなければならない。`A` は
      同じ bridge input 内で bare reserve head としても使われておらず、`B` は attributed
      reserve head として使われていないこと。
    - Acceptance: runner unit coverage は同じ `SurfaceAst` から real source-derived
      `B -> marked set/object` と `A -> B` expansion payload の両方、および attributed
      reserve head が抽出されることを証明する。mixed root、attributed dependency、
      deeper chain、imported / ambiguous symbol、attribute / mode argument、contextual /
      parameterized / recovered definition は withheld のままにする。既存 active
      attributed-RHS chain `.miz` fixture は `checker.type.external.mode_expansion_payload`
      から `type_elaboration.checker.checker.declaration.deferred.evidence_query` へ移る。
      checker は verified fact を出さず、positive attributed-type acceptance は deferred のまま。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、50、56、61、63、64。full attributed-type existential evidence と
      broader mode expansion は MC-G020/MC-G026 のまま。参照: Step 5 source-derived
      semantic bridge、mizar-test task 10、spec 03 type expressions、spec 06 attributes、
      spec 07 modes、spec 17 evidence。

67. **Source-derived structure-qualified attribute gap boundary を追加する。** [x]
    - same-module structure-qualified attribute reference を持つ reserve type
      expression、たとえば `LocalStruct.marked LocalStruct` に対する active
      `type_elaboration` boundary fixture を追加する。
    - Acceptance: active runner は real `.miz` source path が parser/resolver
      executable である一方、checker-owned attribute payload がまだ structure
      qualifier や attribute-owner provenance を持たないため
      `type_elaboration.external_dependency.ast_payload_extraction` に残ることを証明する。
      bridge はこの reference を unqualified attribute payload に書き換えず、
      positive attributed-structure acceptance、existential/evidence、CoreIr、
      ControlFlowIr、VC、proof payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、50、52、53。qualified-attribute provenance、
      attribute-owner resolution、full attributed-type existential evidence、
      broader attribute extraction は MC-G020/MC-G026 のまま。参照: Step 5
      source-derived semantic bridge、mizar-test task 10、spec 03 type expressions、
      spec 05 structures、spec 06 attributes。

68. **Source-derived argument-bearing mode reserve gap boundary を追加する。** [x]
    - same-module local mode head が `of` type arguments を持つ reserve type
      expression、たとえば `Element of a` に対する active `type_elaboration`
      boundary fixture を追加する。
    - Acceptance: active runner は real `.miz` source path が parser/resolver
      executable である一方、checker-owned reserve source bridge がまだ real
      type-argument / term-argument provenance を持たないため
      `type_elaboration.external_dependency.ast_payload_extraction` に残ることを証明する。
      この boundary は mode-argument payload extraction、arity matching、mode
      expansion、positive type elaboration、CoreIr/ControlFlowIr/VC/proof payload を
      実装済みとして扱ってはならない。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、51、55。type-argument / term-argument provenance、
      argument-bearing mode expansion、arity checking、positive acceptance、
      broader mode extraction は MC-G020/MC-G014 のまま。参照: Step 5
      source-derived semantic bridge、mizar-test task 10、spec 03 type expressions、
      spec 07 modes。

69. **Source-derived argument-bearing structure reserve gap boundary を追加する。** [x]
    - same-module local structure declaration が `of` parameter surface を持ち、
      reserve head が `of` type arguments を持つ reserve type expression、たとえば
      `LocalStruct of a` に対する active `type_elaboration` boundary fixture を追加する。
    - Acceptance: active runner は real `.miz` source path が parser/resolver
      executable である一方、checker-owned reserve source bridge がまだ real
      type-argument / term-argument provenance を持たないため
      `type_elaboration.external_dependency.ast_payload_extraction` に残ることを証明する。
      この boundary は structure-argument payload extraction、arity matching、
      base-shape / constructor-witness evidence、positive structure type elaboration、
      CoreIr/ControlFlowIr/VC/proof payload を実装済みとして扱ってはならない。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、52、68。type-argument / term-argument provenance、
      argument-bearing structure payload、base-shape evidence、arity checking、
      positive acceptance、broader structure extraction は MC-G020 のまま。参照:
      Step 5 source-derived semantic bridge、mizar-test task 10、spec 03 type
      expressions、spec 05 structures。

70. **Source-derived bracket-form local mode reserve gap boundary を追加する。** [x]
    - same-module bracket-parameter mode declaration と、たとえば
      `Family[set]` のような bracket-form reserve type head を含む source に対する
      active `type_elaboration` boundary fixture を追加する。
    - Acceptance: active runner は real `.miz` source path が parser/resolver
      executable である一方、checker-owned reserve source bridge がまだ real
      bracket type-argument / `qua`-argument provenance を持たないため、bracket
      type-argument payload extraction や mode-head resolution の前に
      `type_elaboration.external_dependency.ast_payload_extraction` に残ることを証明する。
      この boundary は bracket payload extraction、arity matching、mode expansion、
      positive type elaboration、CoreIr/ControlFlowIr/VC/proof payload を実装済みとして
      扱ってはならない。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、51、68。bracket `type_arg_list` provenance、
      `qua`-argument lowering、argument を持つ mode-head resolution、arity checking、
      positive acceptance、broader mode extraction は MC-G020/MC-G014 のまま。参照:
      Step 5 source-derived semantic bridge、mizar-test task 10、spec 03 type
      expressions、spec 07 modes。

71. **Source-derived bracket-form local structure reserve gap boundary を追加する。** [x]
    - same-module bracket-parameter structure declaration と、たとえば
      `LocalStruct[set]` のような bracket-form reserve type head を含む source に対する
      active `type_elaboration` boundary fixture を追加する。
    - Acceptance: active runner は real `.miz` source path が parser/resolver
      executable である一方、checker-owned reserve source bridge がまだ real
      bracket type-argument / `qua`-argument provenance を持たないため、bracket
      type-argument payload extraction や structure-head resolution の前に
      `type_elaboration.external_dependency.ast_payload_extraction` に残ることを証明する。
      この boundary は bracket payload extraction、arity matching、base-shape /
      constructor-witness evidence、positive structure type elaboration、
      CoreIr/ControlFlowIr/VC/proof payload を実装済みとして扱ってはならない。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、52、69。bracket `type_arg_list` provenance、
      `qua`-argument lowering、argument を持つ structure-head resolution、arity checking、
      positive structure acceptance、broader structure extraction は MC-G020/MC-G014 のまま。
      参照: Step 5 source-derived semantic bridge、mizar-test task 10、spec 03 type
      expressions、spec 05 structures。

72. **Source-derived two-edge bare local mode chain bridge を追加する。** [x]
    - task 56 の pass producer を、bare same-module no-argument local-mode chain
      `Outer -> Middle -> Base -> set` / `object` だけに拡張する。
    - Acceptance: active runner は unique / unrecovered / same-module な mode
      definition 3 個から、source order、definition-local context なし、
      attributes なし、arguments なしの条件で real `ModeExpansion` payload をすべて
      抽出し、reserve declaration は既存の `TypedAst`、`ResolvedTypedAst`、
      summary-readiness、binder-only `CoreContext` preparation path を通る。
      cold path と cached dependency reuse の three-edge local-mode chain は当時
      `type_elaboration.checker.checker.type.external.mode_expansion_payload` に残し、
      two-edge cap が暗黙に広がらないようにした。task 73 は同じ seam を
      three-edge へ昇格し、task 74 はその temporary depth guard を
      AST-bounded structural rule に置き換えた。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、55、56。attributed root / dependency、既存 one-edge
      diagnostic slice を超える structure / attributed-builtin terminal、
      imported / argument-bearing / parameterized / contextual / ambiguous /
      cyclic / forward-reference definition、task 74 の structural guard 外の chain、
      CoreIr、ControlFlowIr、VC、proof payload、broader mode extraction は
      MC-G020/MC-G014 のまま。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 03 type expressions、spec 07 modes。

73. **Source-derived three-edge bare local mode chain bridge を追加する。** [x]
    - task 72 の pass producer を、bare same-module no-argument local-mode chain
      `Outer -> Middle -> Inner -> Base -> set` / `object` だけに拡張する。
    - Acceptance: active runner は unique / unrecovered / same-module な mode
      definition 4 個から、source order、definition-local context なし、
      attributes なし、arguments なしの条件で real `ModeExpansion` payload をすべて
      抽出し、reserve declaration は既存の `TypedAst`、`ResolvedTypedAst`、
      summary-readiness、binder-only `CoreContext` preparation path を通る。
      cold path と cached dependency reuse の four-edge local-mode chain は task 73 時点で
      `type_elaboration.checker.checker.type.external.mode_expansion_payload` に残し、
      three-edge cap が暗黙に広がらないようにした。task 74 はその temporary depth
      guard を AST-bounded structural rule に置き換えた。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、55、56、72。attributed root / dependency、既存 one-edge
      diagnostic slice を超える structure / attributed-builtin terminal、
      imported / argument-bearing / parameterized / contextual / ambiguous /
      cyclic / forward-reference definition、task 74 の structural guard 外の chain、
      CoreIr、ControlFlowIr、VC、proof payload、broader mode extraction は
      MC-G020/MC-G014 のまま。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 03 type expressions、spec 07 modes。

74. **Source-derived structural bare local mode chain bridge を追加する。** [x]
    - task 73 の semantic chain-depth cap を、builtin `set` / `object` で終端する
      bare same-module no-argument local-mode chain 向けの structural rule に
      置き換える。
    - Acceptance: active runner は、各 mode definition が unique / unrecovered /
      same-module / no-argument / definition-local-context-free /
      source-preceding / argument-free / attribute-free で、terminal RHS が exactly
      builtin `set` / `object` である AST-bounded acyclic local-mode chain の各
      link について real `ModeExpansion` payload を抽出する。producer は source
      mode definition 数と等しい AST-derived traversal budget を持つため、resource
      safety は semantic chain-length limit ではなく structural guard である。
      four-edge、cached four-edge、object-terminal、long-chain active pass fixture
      は既存の `TypedAst`、`ResolvedTypedAst`、summary-readiness、binder-only
      `CoreContext` preparation path を通り、CoreIr、ControlFlowIr、VC、proof
      payload は昇格しない。structural guard を満たさない chain は fail closed の
      まま。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、55、56、72、73。attributed root / dependency、既存 one-edge
      diagnostic slice を超える structure / attributed-builtin terminal、
      imported / argument-bearing / parameterized / contextual / ambiguous /
      cyclic / forward-reference definition、structure / attributed evidence、
      CoreIr、ControlFlowIr、VC、proof payload、broader mode extraction は
      MC-G020/MC-G014 のまま。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 03 type expressions、spec 07 modes、spec 17
      base-shape inhabitation。

75. **Source-derived local mode forward-reference active-range boundary を追加する。** [x]
    - declaration item が active になる前に、後続 same-module local mode
      declaration を reserve head が名前参照する active fail coverage を追加する。
    - Acceptance: active type-elaboration runner は checker handoff 前に
      `type_elaboration.lower_stage.frontend:malformed_type_expression` を報告し、
      future declaration から `ModeExpansion` を捏造せず、successful reserve
      declaration、CoreIr、ControlFlowIr、VC、proof payload を昇格しない。
      forward reference acceptance は Chapter 2/11 active-range rule により
      forbidden のまま。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、55、74。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 02 active range、spec 07 modes、spec 11 symbol
      management。

76. **Source-derived local structure forward-reference active-range boundary を追加する。** [x]
    - declaration item が active になる前に、後続 same-module local structure
      declaration を reserve head が名前参照する active fail coverage を追加する。
    - Acceptance: active type-elaboration runner は checker handoff 前に
      `type_elaboration.lower_stage.frontend:malformed_type_expression` を報告し、
      future declaration から structure type-head payload を捏造せず、successful
      reserve declaration、base-shape / constructor-witness evidence query、
      CoreIr、ControlFlowIr、VC、proof payload を昇格しない。forward reference
      acceptance は Chapter 2/11 active-range rule により forbidden のまま。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、52、75。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 02 active range、spec 05 structures、spec 11 symbol
      management。

77. **Source-derived local attribute forward-reference active-range boundary を追加する。** [x]
    - declaration item が active になる前に、後続 same-module local attribute
      declaration を reserve type が使う active fail coverage を追加する。
    - Acceptance: active type-elaboration runner は checker handoff 前に
      `type_elaboration.lower_stage.frontend:malformed_type_expression` を報告し、
      future declaration から `AttributeInput` を捏造せず、successful reserve
      declaration、attributed-type evidence query、CoreIr、ControlFlowIr、VC、
      proof payload を昇格しない。forward reference acceptance は Chapter 2/11
      active-range rule により forbidden のまま。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、50、75、76。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 02 active range、spec 06 attributes、spec 11 symbol
      management。

78. **Source-derived imported structure reserve extraction-gap boundary を追加する。** [x]
    - 既存の `parser.type_fixtures` import summary が提供する imported
      structure symbol を head とする reserve type の active fail coverage を
      追加する。
    - Acceptance: active type-elaboration runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告し、
      imported structure provenance、structure type-head payload、base-shape /
      constructor-witness evidence、positive structure elaboration を捏造せず、
      CoreIr、ControlFlowIr、VC、proof payload へ昇格しない。この fixture は
      diagnostic boundary coverage のみである。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、52、69。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 03 type expressions、spec 05 structures、spec 11
      symbol management、spec 12 modules and namespaces。

79. **Source-derived imported mode reserve extraction-gap boundary を追加する。** [x]
    - 既存の `parser.type_fixtures` import summary が提供する imported mode
      symbol を head とする reserve type の active fail coverage を追加する。
    - Acceptance: active type-elaboration runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告し、
      imported mode provenance、mode type-head payload、`ModeExpansion` payload、
      positive mode elaboration、より広い imported mode semantics を捏造せず、
      CoreIr、ControlFlowIr、VC、proof payload へ昇格しない。この fixture は
      diagnostic boundary coverage のみであり、generic non-builtin imported-mode
      gap の traceability だけを精密化する。task 82 は documented
      `TypeCaseMode` provenance/type-head slice だけを上書きする。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、51、55、78。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 03 type expressions、spec 07 modes、spec 11
      symbol management、spec 12 modules and namespaces。

80. **Source-derived imported attribute reserve extraction-gap boundary を追加する。** [x]
    - 既存の `parser.type_fixtures` import summary が提供する imported
      attribute symbol を attribute として持つ reserve type の active fail coverage
      を追加する。
    - Acceptance: task 84 / task 85 より前は、active type-elaboration runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告した。
      task 84 が documented `TypeCaseAttr` 部分を、task 85 が negative
      `empty`/builtin-`set` 部分を supersede した後も、それらの bridge 外の
      broader imported attribute は imported attribute provenance、
      `AttributeInput` payload、attributed-type evidence、positive attributed type
      elaboration、より広い imported attribute semantics を捏造せず、CoreIr、
      ControlFlowIr、VC、proof payload へ昇格しない。この task は generic
      import-backed attributed reserve gap の historical diagnostic boundary
      coverage として残る。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、50、67、78、79。参照: Step 5 source-derived semantic
      bridge、mizar-test task 10、spec 03 type expressions、spec 06 attributes、
      spec 11 symbol management、spec 12 modules and namespaces。

81. **Source-derived argument-bearing local attribute reserve extraction-gap boundary を追加する。** [x]
    - `param_prefix` 構文で宣言された same-module parameterized attribute を、
      Chapter 3/6 の `attribute_name(args)` application form で reserve type
      expression に使う active fail coverage を追加する。
    - Acceptance: active type-elaboration runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告し、
      term-argument provenance、checker `AttributeInput` argument payload、
      attributed-type evidence、positive attributed type elaboration、より広い
      parameterized attribute semantics を捏造せず、CoreIr、ControlFlowIr、VC、
      proof payload へ昇格しない。この fixture は diagnostic boundary coverage
      のみであり、real source lexer/parser producer seam が parameterized local
      attribute surface を checker-owned extraction boundary まで運び、resolver
      declaration-symbol suffix projection がそれを保持することだけを確認する。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`、
      `cargo test -p mizar-lexer`、`cargo test -p mizar-frontend`、
      `cargo test -p mizar-parser`。
    - 依存: tasks 48、50、67、77。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 02 lexical structure、spec 03 type
      expressions、spec 06 attributes、spec 11 symbol management、
      mizar-lexer disambiguator design、
      mizar-resolve symbol projection design。

82. **Source-derived imported mode reserve provenance bridge を追加する。** [x]
    - task-79 の imported-mode reserve boundary を、active `type_elaboration`
      runner が real `parser.type_fixtures` import-summary 由来の
      `ImportedSource` mode symbol を checker `TypeHeadInput` として渡せる
      ところまでだけ昇格する。
    - Acceptance: checker reserve bridge は imported mode symbol が `SymbolEnv`
      で可視であり、`SymbolKind::Mode` を持ち、local source ではなく
      `ImportedSource` contribution に裏付けられていることを検証する。
      runner は `TypeCaseMode` について
      `type_elaboration.external_dependency.ast_payload_extraction` ではなく
      `type_elaboration.checker.checker.type.external.mode_expansion_payload`
      に到達する。real imported mode-definition/module-summary expansion
      payload はまだ存在しないためである。この task は imported module AST
      extraction、`ModeExpansion` payload、arity checking、positive mode
      elaboration、CoreIr、ControlFlowIr、VC、proof payload を捏造してはならず、
      imported structure、imported attribute、argument、bracket、qualified
      attribute、imported evidence は既存 gap に残す。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、51、55、78、79。参照: Step 5 source-derived semantic
      bridge、mizar-test task 10、spec 03 type expressions、spec 07 modes、
      spec 11 symbol management、spec 12 modules and namespaces。

83. **Source-derived imported structure reserve provenance bridge を追加する。** [x]
    - task-78 の imported-structure reserve boundary を、active
      `type_elaboration` runner が documented `parser.type_fixtures`
      import-summary 由来の `R` structure symbol を checker `TypeHeadInput` として
      渡せるところまでだけ昇格する。
    - Acceptance: checker reserve bridge は `R` が `SymbolEnv` で可視であり、
      `SymbolKind::Structure` を持ち、`parser.type_fixtures` の `ImportedSource`
      contribution に裏付けられていることを検証する。runner は `R` について
      `type_elaboration.external_dependency.ast_payload_extraction` ではなく
      `type_elaboration.checker.checker.declaration.deferred.evidence_query` に到達する。
      imported module AST extraction と base-shape / constructor-witness evidence は
      まだ存在しないためである。この task は imported module AST extraction、
      base-shape / constructor-witness evidence、positive structure elaboration、
      CoreIr、ControlFlowIr、VC、proof payload を捏造してはならず、later task-97
      `TypeCaseStruct` slice 外の generic imported structure、imported attribute、
      argument、bracket、qualified attribute、imported evidence は既存 gap に残す。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、52、76、78、82。参照: Step 5 source-derived semantic
      bridge、mizar-test task 10、spec 03 type expressions、spec 05 structures、
      spec 11 symbol management、spec 12 modules and namespaces。

84. **Source-derived imported attribute reserve provenance bridge を追加する。** [x]
    - task-80 の imported-attribute reserve boundary を、active
      `type_elaboration` runner が documented `parser.type_fixtures`
      import-summary 由来の `TypeCaseAttr` attribute symbol を builtin `set` 上の
      checker `AttributeInput` として渡せるところまでだけ昇格する。
    - Acceptance: checker reserve bridge は `TypeCaseAttr` が `SymbolEnv` で可視で
      あり、`SymbolKind::Attribute` を持ち、`parser.type_fixtures` の
      `ImportedSource` contribution に裏付けられていることを検証する。runner は
      `TypeCaseAttr set` について
      `type_elaboration.external_dependency.ast_payload_extraction` ではなく
      `type_elaboration.checker.checker.declaration.deferred.evidence_query` に到達する。
      imported module AST extraction と attributed-type existential/evidence payload は
      まだ存在しないためである。この task は imported module AST extraction、
      attributed-type evidence、positive attributed type elaboration、CoreIr、
      ControlFlowIr、VC、proof payload を捏造してはならず、`empty` のような generic
      imported attribute、structure-qualified owner provenance、argument、bracket、
      qualified attribute、imported evidence は既存 gap に残す。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、50、67、80、83。参照: Step 5 source-derived semantic
      bridge、mizar-test task 10、spec 03 type expressions、spec 06 attributes、
      spec 11 symbol management、spec 12 modules and namespaces。

85. **Source-derived imported non-empty attribute reserve provenance bridge を追加する。** [x]
    - 既存 task-80 の imported-attribute reserve boundary を、active
      `type_elaboration` runner が `non empty set` について documented
      `parser.type_fixtures` import-summary 由来の `empty` attribute symbol を
      builtin `set` 上の negative checker `AttributeInput` として渡せるところまでだけ
      昇格する。
    - Acceptance: checker reserve bridge は `empty` が `SymbolEnv` で可視であり、
      `SymbolKind::Attribute` を持ち、`parser.type_fixtures` の `ImportedSource`
      contribution に裏付けられ、negative polarity で builtin `set` に付いていることを
      検証する。既存 `fail_type_elaboration_attributed_reserve_gap_001` fixture は
      `type_elaboration.external_dependency.ast_payload_extraction` ではなく
      `type_elaboration.checker.checker.declaration.deferred.evidence_query` に到達する。
      imported module AST extraction と attributed-type existential/evidence payload は
      まだ存在しないためである。この task は imported module AST extraction、
      attributed-type evidence、positive `empty set` elaboration、non-`set` head 上の
      imported `empty`、CoreIr、ControlFlowIr、VC、proof payload を捏造してはならず、
      positive `empty set` と `non empty object` の active boundary sidecar、
      attribute argument、qualified owner provenance、broader imported attribute は既存
      gap に残す。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、50、80、84。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 03 type expressions、spec 06 attributes、
      spec 11 symbol management、spec 12 modules and namespaces。

86. **Source-derived theorem formula extraction-gap boundary を追加する。** [x]
    - `theorem FormulaPayloadBoundary: thesis;` のような formula-only theorem
      source について、専用の active `type_elaboration` boundary を追加する。
    - Acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned theorem/formula payload extraction、local proof context、
      recorded fact、theorem acceptance、CoreIr、ControlFlowIr、VC、proof payload、
      `formula_statement` runner がまだ存在しないためである。この task は formula
      payload、fact、proof skeleton、downstream semantic payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: task 48。参照: Step 5 source-derived semantic bridge、mizar-test
      task 10、spec 14 formulas、spec 16 theorems and proofs。

87. **Source-derived term formula extraction-gap boundary を追加する。** [x]
    - `theorem TermFormulaPayloadBoundary: 1 = 1;` のように source term を含む
      theorem formula について、専用の active `type_elaboration` boundary を追加する。
    - Acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned term/formula payload extraction、term inference、formula
      checking、recorded fact、theorem acceptance、CoreIr、ControlFlowIr、VC、proof
      payload、`formula_statement` runner がまだ存在しないためである。この task は
      term payload、formula payload、fact、proof skeleton、downstream semantic
      payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: task 86。参照: Step 5 source-derived semantic bridge、mizar-test
      task 10、spec 13 term expressions、spec 14 formulas、spec 16 theorems and
      proofs。

88. **Source-derived proof skeleton extraction-gap boundary を追加する。** [x]
    - `theorem ProofSkeletonPayloadBoundary: thesis proof thus thesis; end;` の
      ように proof block と conclusion statement を持つ theorem について、専用の
      active `type_elaboration` boundary を追加する。
    - Acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned proof skeleton payload extraction、local proof context、
      formula payload extraction、recorded fact、theorem acceptance、CoreIr、
      ControlFlowIr、VC、proof payload、`formula_statement` runner がまだ存在しないため
      である。この task は proof skeleton payload、formula payload、local fact、
      theorem acceptance、downstream semantic payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: task 87。参照: Step 5 source-derived semantic bridge、mizar-test
      task 10、spec 14 formulas、spec 15 statements、spec 16 theorems and proofs。

89. **Source-derived statement proof extraction-gap boundary を追加する。** [x]
    - labeled `A: thesis proof ... end;` と final
      `thus thesis proof ... end;` のような statement-level proof justification
      を含む theorem proof について、専用の active `type_elaboration` boundary
      を追加する。
    - Acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned statement proof payload extraction、nested proof skeleton
      payload、local proof context、formula payload extraction、label-reference
      semantic checking、recorded fact、theorem acceptance、CoreIr、ControlFlowIr、
      VC、proof payload、`formula_statement` runner が存在しないためである。この
      task は statement proof payload、proof skeleton payload、formula payload、
      local fact、theorem acceptance、downstream semantic payload を捏造しては
      ならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: task 88。参照: Step 5 source-derived semantic bridge、mizar-test
      task 10、spec 14 formulas、spec 15 statements、spec 16 theorems and proofs。

90. **Source-derived predicate/functor definition extraction-gap boundary を追加する。** [x]
    - predicate definition と functor definition を含む definition block について、
      専用の active `type_elaboration` boundary を追加する。
    - Acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned predicate/functor definition declaration payload extraction、
      definition-local context、definiens formula/term payload、overload payload、
      recorded fact、CoreIr、ControlFlowIr、VC、proof payload、
      `formula_statement` runner がまだ存在しないためである。この task は
      definition payload、formula/term body payload、overload payload、fact、
      downstream semantic payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: task 89。参照: Step 5 source-derived semantic bridge、mizar-test
      task 10、spec 09 predicate definitions、spec 10 functor definitions。

91. **Source-derived attribute definition extraction-gap boundary を追加する。** [x]
    - attribute definition を含む definition block について、専用の active
      `type_elaboration` boundary を追加する。
    - Acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned attribute definition declaration payload extraction、
      definition-local context、formula-definiens payload、attributed-type
      evidence、recorded fact、CoreIr、ControlFlowIr、VC、proof payload、
      `formula_statement` runner がまだ存在しないためである。この task は
      definition payload、formula body payload、evidence、fact、downstream
      semantic payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: task 90。参照: Step 5 source-derived semantic bridge、mizar-test
      task 10、spec 06 attribute definitions。

92. **Source-derived mode/structure definition extraction-gap boundary を追加する。** [x]
    - structure definition と mode definition を含む definition block について、
      専用の active `type_elaboration` boundary を追加する。
    - Acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned mode/structure definition declaration payload extraction、
      mode expansion、structure base-shape / constructor / selector evidence、
      definition-local context、recorded fact、CoreIr、ControlFlowIr、VC、
      proof payload、`formula_statement` runner がまだ存在しないためである。
      この task は definition payload、mode-expansion payload、structure
      evidence、fact、downstream semantic payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: task 91。参照: Step 5 source-derived semantic bridge、mizar-test
      task 10、spec 05 structures、spec 07 mode definitions。

93. **Source-derived proof-local declaration extraction-gap boundary を追加する。** [x]
    - `let`、`given`、`consider`、`set`、`reconsider` statement を含む theorem
      proof について、専用の active `type_elaboration` boundary を追加する。
    - Acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned proof-local declaration payload extraction、local proof
      context、formula/term payload、RHS term inference、reconsider coercion /
      obligation evidence、recorded fact、CoreIr、ControlFlowIr、VC、proof
      payload、`formula_statement` runner がまだ存在しないためである。この
      task は proof-local declaration payload、formula/term payload、local
      fact、theorem acceptance、downstream semantic payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: task 92。参照: Step 5 source-derived semantic bridge、mizar-test
      task 10、spec 15 statements、spec 16 theorems and proofs。

94. **Source-derived proof-local inline definition extraction-gap boundary を追加する。** [x]
    - proof-local `deffunc` と `defpred` statement を含む theorem proof
      について、専用の active `type_elaboration` boundary を追加する。
    - Acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned inline definition formal/body payload extraction、local
      abbreviation expansion、term/formula body payload、guard evidence、
      recorded fact、theorem acceptance、CoreIr、ControlFlowIr、VC、proof payload、
      `formula_statement` runner がまだ存在しないためである。この task は
      inline definition payload、local abbreviation expansion、term/formula
      body payload、fact、theorem acceptance、downstream semantic payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: task 93。参照: Step 5 source-derived semantic bridge、mizar-test
      task 10、spec 15 statements。

95. **Source-derived registration block extraction-gap boundary を追加する。** [x]
    - existential cluster と conditional cluster を含む top-level
      `registration` block について、専用の active `type_elaboration`
      boundary を追加する。
    - Acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned registration-item payload extraction、correctness-condition /
      proof-obligation payload、accepted activation / evidence status、cluster /
      reduction semantics、recorded fact、CoreIr、ControlFlowIr、VC、proof payload、
      `formula_statement` / `advanced_semantics` runner がまだ存在しないためである。
      この task は registration payload、activation status、cluster / reduction
      fact、Chapter 17 semantic coverage、downstream semantic payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: task 94。参照: Step 5 source-derived semantic bridge、mizar-test
      task 10、spec 17 clusters and registrations。

96. **Source-derived redefinition / notation extraction-gap boundary を追加する。** [x]
    - top-level と definition-local の synonym / antonym alias、および
      attribute、predicate、functor redefinition declaration について、専用の
      active `type_elaboration` boundary を追加する。
    - Acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned redefinition payload extraction、notation alias relation
      payload、target inference、coherence proof-obligation payload、overload
      candidate payload、recorded fact、CoreIr、ControlFlowIr、VC、proof payload、
      `formula_statement` / `advanced_semantics` runner がまだ存在しないためである。
      この task は alias semantics、redefinition payload、overload fact、Chapter 11
      alias semantic resolution、Chapter 19 overload / redefinition semantic
      coverage、downstream semantic payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: task 95。参照: Step 5 source-derived semantic bridge、mizar-test
      task 10、spec 11 symbol management、spec 19 overload resolution。

97. **Source-derived imported TypeCaseStruct reserve provenance bridge を追加する。** [x]
    - task-78 の imported-structure reserve boundary を、active
      `type_elaboration` runner が documented `parser.type_fixtures`
      import-summary 由来の `TypeCaseStruct` structure symbol を checker
      `TypeHeadInput` として渡せるところまでだけ昇格する。
    - Acceptance: checker reserve bridge は `TypeCaseStruct` が `SymbolEnv` で可視で
      あり、`SymbolKind::Structure` を持ち、`parser.type_fixtures` の
      `ImportedSource` contribution に裏付けられていることを検証する。runner は
      `TypeCaseStruct` について
      `type_elaboration.external_dependency.ast_payload_extraction` ではなく
      `type_elaboration.checker.checker.declaration.deferred.evidence_query` に到達する。
      imported module AST extraction と base-shape / constructor-witness evidence は
      まだ存在しないためである。この task は imported module AST extraction、
      base-shape / constructor-witness evidence、positive structure elaboration、
      CoreIr、ControlFlowIr、VC、proof payload を捏造してはならず、他の generic
      imported structure、imported attribute、argument、bracket、qualified attribute、
      imported evidence は既存 gap に残す。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、52、76、78、83。参照: Step 5 source-derived semantic
      bridge、mizar-test task 10、spec 03 type expressions、spec 05 structures、
      spec 11 symbol management、spec 12 modules and namespaces。

98. **Source-derived imported predicate/functor term-formula extraction-gap boundary を追加する。** [x]
    - `parser.type_fixtures` を import し、`divides` や `++` のような documented
      imported predicate/functor surface を使う theorem formula 専用の active
      `type_elaboration` boundary を追加する。
    - Acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned imported predicate/functor term/formula payload extraction、
      term inference、formula checking、recorded fact、theorem acceptance、CoreIr、
      ControlFlowIr、VC、proof payload、`formula_statement` runner がまだ利用できないためである。
      この task は imported predicate/functor semantic payload、term/formula
      payload、fact、theorem acceptance、downstream semantic payload を捏造してはならず、
      imported module AST extraction も主張してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: tasks 86、87。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 11 symbol management、spec 12 modules and namespaces、
      spec 13 term expressions、spec 14 formulas、spec 16 theorems and proofs。

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
