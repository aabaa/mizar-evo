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

production crate は `mizar-session` と `mizar-resolve` に依存し、
`mizar-syntax` へは推移的にだけ到達する。Task 258B1 は parser-shaped
corruption fixture 用に direct な test-only `mizar-syntax` dev-dependency を
追加するが、production source は syntax-free のままである。第 1 波は
`mizar-resolve` task 14 と 20（名前解決、`SymbolEnv` 骨格）を必要とし、
以後の波は `mizar-resolve` task 21 のシグネチャ増分と、対応する
`mizar-parser` の定義文法タスク（23-31）とともに成長する。アーキテクチャ:
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
| corpus seeds | task 49 が必要な runner、parser support、declaration-symbol support、source-to-checker payload extraction 到着時に、監査 fixture 16 件、task-35 constructor-property seed、task-36 duplicate-coverage seed、task-37 ordinary/template-derived equivalent-root seed、task-38 functorial-`for` guard seed、task-39 property-overlap coherence seed、task-44 omitted-`reconsider` / ambiguous-redefinition-target seed を活性化する。task-37 same-return signature-conflict seed は Resolver Task 31 が `declaration_symbol` で単独活性化し、task 49 はその member を exact 24-fixture set と reconciliation/deduplication するだけである |

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
      exact scopeは
      [payload_family_decomposition.md](./payload_family_decomposition.md)の
      24-fixture reconciliation setである。same-return memberはresolver Task 31が
      `declaration_symbol`でsole activationし、Task 49は残り23件をactivateした後に
      24件全体をreconcile/deduplicateする。same-signature/different-return fixtureは
      set外で既にactiveであり、再activationせずunchanged controlとして保つ。task-29 の
      deferred corpus record を監査由来の requirement id を指す(または
      置き換えられる)よう改訂する。
    - 受け入れ条件: `mizar-test` plan が fixture を active と表示し plan
      error が 0 件。deferred record が二重計上されない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: 完了済みtasks 35-44、parser Tasks 47-48、resolver Task 31、完了済み
      checker Task 247、checker Tasks 248-264/269-279（blocked-reserved Task
      274のexternal accepted-status gateとexternal scheme/theorem-role Gate S1を
      含む）、mizar-test Task-10 increments `MT10-FS`/`MT10-AS`。Tasks 266-268
      だけでは不十分。参照:
      [payload_family_decomposition.md](./payload_family_decomposition.md)、
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
    - Acceptance: task 84 / task 85 / task 116 より前は、active type-elaboration runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告した。
      task 84 が documented `TypeCaseAttr` 部分を、task 85 が negative
      `empty`/builtin-`set` 部分を、task 116 が positive
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
      attributed-type evidence、non-`set` head 上の
      imported `empty`、CoreIr、ControlFlowIr、VC、proof payload を捏造してはならず、
      task 116 が positive `empty set` sidecar を supersede する。この task は
      `non empty object` の active boundary sidecar、attribute argument、qualified
      owner provenance、broader imported attribute を既存 gap に残す。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、50、80、84。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 03 type expressions、spec 06 attributes、
      spec 11 symbol management、spec 12 modules and namespaces。

116. **Source-derived imported positive empty attribute reserve provenance bridge を追加する。** [x]
    - 既存 task-80 の positive imported-attribute reserve boundary を、active
      `type_elaboration` runner が `empty set` について documented
      `parser.type_fixtures` import-summary 由来の `empty` attribute symbol を
      builtin `set` 上の positive checker `AttributeInput` として渡せるところまでだけ
      昇格する。
    - Acceptance: checker reserve bridge は `empty` が `SymbolEnv` で可視であり、
      `SymbolKind::Attribute` を持ち、`parser.type_fixtures` の `ImportedSource`
      contribution に裏付けられ、positive polarity で builtin `set` に付いていることを
      検証する。既存
      `fail_type_elaboration_imported_empty_positive_gap_001` fixture は
      `type_elaboration.external_dependency.ast_payload_extraction` ではなく
      `type_elaboration.checker.checker.declaration.deferred.evidence_query` に到達する。
      imported module AST extraction と attributed-type existential/evidence payload は
      まだ存在しないためである。この task は imported module AST extraction、
      attributed-type evidence、positive attributed-type acceptance、non-`set` head 上の
      imported `empty`、CoreIr、ControlFlowIr、VC、proof payload を捏造してはならず、
      task 171 は後で exact `non empty object` sidecar だけを supersede する。
      attribute argument、qualified owner provenance、broader imported attribute
      は既存 gap に残す。
    - 検証: `cargo test -p mizar-test`、`cargo test -p mizar-checker`。
    - 依存: tasks 48、50、80、84、85。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 03 type expressions、spec 06 attributes、
      spec 11 symbol management、spec 12 modules and namespaces。

171. **Source-derived imported negative empty object reserve provenance bridge を追加する。** [x]
    - `import parser.type_fixtures; reserve x for non empty object;` を含む既存
      `fail_type_elaboration_imported_empty_object_gap_001` source だけを昇格する。
    - Acceptance: parser / resolver は real imported `empty`
      `SymbolKind::Attribute` と `ImportedSource` provenance を供給し、source
      extractor は negative polarity と builtin `object` を保持する。checker-owned
      reserve bridge はその exact provenance/polarity/head combination を受理して
      negative `AttributeInput` 1 個を declaration checking へ渡し、active case は
      `type_elaboration.checker.checker.declaration.deferred.evidence_query` で fail
      closed する。positive `empty object`、symbol head 上の imported attribute、
      attribute admissibility/evidence、accepted attributed type、imported module AST
      extraction、CoreIr、ControlFlowIr、VC、proof payload は deferred のままとする。
    - 検証: `cargo test -p mizar-checker`、`cargo test -p mizar-test`、final
      workspace verification。
    - 依存: tasks 48、50、80、84、85、116。参照: Step 5 source-derived semantic
      bridge、mizar-test task 10、spec 03 type expressions、spec 06 attributes、
      spec 11 symbol management、spec 12 modules and namespaces。

86. **Source-derived theorem formula extraction-gap boundary を追加する。** [x]
    - `theorem FormulaPayloadBoundary: thesis;` のような formula-only theorem
      source について、専用の active `type_elaboration` boundary を追加する。
    - Historical acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned theorem/formula payload extraction、local proof context、
      recorded fact、theorem acceptance、CoreIr、ControlFlowIr、VC、proof payload、
      `formula_statement` runner がまだ存在しないためである。task 115 はこの exact
      formula-only theorem source だけを supersede し、source-derived `thesis`
      formula constant site/range を checker の recovery `FormulaInput` として渡す。
      historical task は broader formula payload extraction、theorem acceptance、
      fact、proof skeleton、downstream semantic payload と読んではならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: task 48。参照: Step 5 source-derived semantic bridge、mizar-test
      task 10、spec 14 formulas、spec 16 theorems and proofs。

115. **Exact source-derived formula statement recovery checker bridge を追加する。** [x]
    - exact unrecovered source
      `theorem FormulaPayloadBoundary: thesis;` についてのみ task 86 を supersede
      する。
    - Acceptance: parser と resolver は source を実行し、active runner は module が
      1 つの theorem item だけを含み、その theorem が direct token text `thesis`
      を持つ `FormulaConstant(Thesis)` だけを包む 1 つの `FormulaExpression`
      child を持つことを検証する。そのうえで、その source site/range を checker
      recovery `FormulaInput` として渡す。task 117 はこの recovery marker を
      real `FormulaKind::Thesis` payload に進め、missing formula payload の
      fail-closed diagnostic を維持する。
    - formula constant semantics、child-formula graph payload、theorem
      acceptance、fact、proof skeleton/context/statement payload、
      `formula_statement`、CoreIr、ControlFlowIr、VC、proof payload を捏造してはならない。
      proof block や追加 item を含む non-exact shape は
      `type_elaboration.external_dependency.ast_payload_extraction` に残す。
    - 検証: `cargo test -p mizar-test`。
    - 依存: tasks 86、112。参照: Step 5 source-derived semantic bridge、mizar-test
      task 10、spec 14 formulas、spec 16 theorems and proofs。

117. **Source-derived formula constant kind checker bridge を追加する。** [x]
    - exact unrecovered `theorem FormulaPayloadBoundary: thesis;` source について
      task 115 を supersede し、source-derived formula constant を generic
      unsupported recovery kind ではなく `FormulaKind::Thesis` として渡す。
    - exact unrecovered
      `FormulaConnectiveQuantifierPayloadBoundary: contradiction implies for x
      being set holds not contradiction` theorem source についてのみ task 112 を
      拡張し、2 つの real `contradiction` constant site/range を existing
      implication、quantifier、negation shell payload とともに
      `FormulaKind::Contradiction` として渡す。
    - Acceptance: parser と resolver は source を実行し、active runner は exact
      supported AST shape を検証し、source-derived checker formula constant
      payload を `TermFormulaChecker` に渡し、connective case では既存の
      quantifier payload diagnostic とともに
      `type_elaboration.checker.checker.formula.external.formula_payload` で
      fail closed する。
    - formula constant semantic truth value、child-formula graph link、
      quantifier binder/context payload、formula checking、fact、theorem
      acceptance、proof skeleton/context/statement payload、`formula_statement`、
      CoreIr、ControlFlowIr、VC、proof payload を捏造してはならない。non-exact
      shape は `type_elaboration.external_dependency.ast_payload_extraction` に残す。
    - 検証: `cargo test -p mizar-checker`, `cargo test -p mizar-test`。
    - 依存: tasks 86、99、112、115。参照: Step 5 source-derived semantic
      bridge、mizar-test task 10、spec 14 formulas、spec 16 theorems and proofs。

118. **Builtin binary theorem exact-token guard を厳密化する。** [x]
    - task 106/107/108 が共有する builtin-binary numeral theorem bridge の
      source-derived producer guard を修正する。active runner は theorem item の
      direct token slice が exact `theorem <label> : ;` である場合だけ equality、
      inequality、membership config を選び、追加 theorem token の中に label が含まれる
      だけでは選ばない。
    - Acceptance: 既存の exact active `.miz` sidecar と checker term/formula
      handoff payload は変更しない。status-prefixed または extra-token を持つ
      builtin-binary theorem shape は
      `type_elaboration.external_dependency.ast_payload_extraction` に残る。
    - label、operator、literal、accepted theorem surface を広げてはならず、
      numeric type payload、formula checking、fact、theorem acceptance、
      `formula_statement`、CoreIr、ControlFlowIr、VC、proof payload を捏造しては
      ならない。新しい active sidecar や spec coverage credit は追加しないため、
      `doc/design/spec_coverage_audit.md` は変更しない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: tasks 106、107、108。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 13 term expressions、spec 14 formulas、spec 16
      theorems and proofs。

119. **Exact source-derived reserved-variable equality checker bridge を追加する。** [x]
    - exact unrecovered source
      `reserve x for set; theorem ReservedVariableEqualityPayloadBoundary: x = x;`
      だけを昇格する。
    - Acceptance: parser と resolver は source を実行する。runner は real reserve
      declaration handoff を再利用し、2 つの identifier term を checker-owned
      `BindingEnv::lookup` で解決し、記述された builtin `set` reserve type を
      distinct result/expected-type role site に投影して、2 つの variable
      `TermInput` と 1 つの equality `FormulaInput` を `TermFormulaChecker` に渡す。
      2 term は `Inferred`、formula は `Checked` となり、active
      type-elaboration pass case は diagnostic/fact を持たない。
    - `Checked` は type/well-formedness に限定する。implicit universal-closure
      node、equality fact/truth、theorem acceptance、proof skeleton、
      `formula_statement`、CoreIr、ControlFlowIr、VC、proof payload を捏造しては
      ならない。non-exact source は payload-extraction gap に残す。
    - 検証: `cargo test -p mizar-test`、最終 workspace verification。
    - 依存: tasks 20、48、106、118。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 04 reserved variables、spec 13 term expressions、
      spec 14 formulas、spec 16 theorems and proofs。

120. **Exact source-derived reserved-variable membership checker bridge を追加する。** [x]
    - exact unrecovered source
      `reserve x for set; theorem ReservedVariableMembershipPayloadBoundary: x in x;`
      だけを昇格する。
    - Acceptance: task 119 の real reserve handoff と independent source-order
      `BindingEnv` lookup を再利用し、2 つの known builtin-`set` variable result
      payload、右 operand の single expected-`set` payload、membership
      `FormulaInput` を `TermFormulaChecker` に渡す。2 つの `Inferred` term、1 つの
      no-fact `Checked` membership、exact 3 role owner、empty
      candidate/deferred/diagnostic output、task-specific invalid-payload key、real
      frontend/resolver active-sidecar payload test を要求する。
    - `Checked` は type/well-formedness だけである。membership truth/fact、
      implicit closure、theorem acceptance、`formula_statement`、proof、CoreIr、
      ControlFlowIr、VC payload を捏造しない。non-exact source は extraction gap
      に残す。
    - 検証: `cargo test -p mizar-test`、最終 workspace verification。
    - 依存: tasks 108、119。参照: Step 5、mizar-test task 10、spec 04、13、14、16。

121. **Exact source-derived reserved-variable inequality checker bridge を追加する。** [x]
    - `reserve x for set; theorem ReservedVariableInequalityPayloadBoundary: x <> x;`
      だけを昇格する。
    - shared real lookup/type producer を 2 linked result role、2 expected role、
      2 `Inferred` term、1 fact-free pre-desugaring `Checked` inequality に再利用し、
      task-specific invalid key、near-miss matrix、real frontend/resolver payload
      test で guard する。
    - inequality desugaring/truth/fact、implicit closure、theorem acceptance、proof、
      CoreIr、ControlFlowIr、VC を主張しない。
    - 依存: tasks 107、119、120。mizar-test と full workspace で検証する。

122. **Checker の reflexive type-assertion admissibility と exact reserved-variable source bridge を追加する。** [x]
    - `TermFormulaChecker` を修正し、type assertion は one ready subject と one
      asserted type を要求し、normalized identity だけを現時点で supported な
      reflexive reachability として受理する。known non-identical type は
      `checker.formula.external.type_assertion_reachability_payload` で defer する。
    - task 119 の real reserve lookup/result producer と task 109 の formula-side
      asserted-type AST producer を結合し、
      `reserve x for set; theorem ReservedVariableTypeAssertionPayloadBoundary: x is set;`
      だけを昇格する。normalization 前の両 input と independent source anchor を
      保持・検証する。
    - 1 `Inferred` variable と 1 fact-free `Checked` type assertion を要求し、general
      reachability/widening/`qua`、attribute、truth/fact、implicit closure、theorem
      acceptance、`formula_statement`、proof、CoreIr、ControlFlowIr、VC は deferred
      のままにする。
    - 依存: tasks 109、119。mizar-checker、mizar-test、full workspace で検証する。

123. **Exact source-derived distinct reserved-variable equality checker bridge を追加する。** [x]
    - `reserve x, y for set; theorem DistinctReservedVariableEqualityPayloadBoundary: x = y;`
      だけを対象とする spec-derived active pass fixture を追加する。
    - 受入条件: parser と resolver が source を実行し、runner は real
      multi-reserve declaration handoff を再利用する。記述された 1 個の
      `set` type を指す source type range を共有しつつ、checker の異なる
      binding identity 2 個を保存し、source-order の `BindingEnv::lookup`
      site で `x` と `y` を独立に解決する。operand ごとの result と
      expected-type role input は、candidate、deferred reason、diagnostic を
      持たない 2 個の `Inferred` variable term と、fact を持たない 1 個の
      `Checked` equality に到達しなければならない。
    - production invariant validation、near-miss matrix、real
      frontend/resolver active-sidecar payload test を追加する。既存
      expectation を rebaseline してはならない。新規 pass expectation は
      spec 4.3、13.1.1、14.5.2 と theorem declaration contract から導く。
    - この task が credit するのは exact distinct-binding
      type/well-formedness だけである。implicit universal-closure または
      quantifier-order node、equality truth/fact、theorem acceptance、
      `formula_statement`、proof、CoreIr、ControlFlowIr、VC payload を捏造して
      はならない。non-exact multi-binding source は extraction gap のままにする。
    - `doc/design/spec_coverage_audit.md` の Chapter 4、13、14、16 row を更新する。
      `crates/mizar-checker/src/` が変わらない限り checker source-layout
      inventory の更新は不要である。
    - 依存: tasks 20、119。mizar-test と full workspace を検証する。

124. **Exact source-derived multiple-reserve-declaration equality checker bridge を追加する。** [x]
    - `reserve x for set; reserve y for set; theorem MultipleReserveDeclarationEqualityPayloadBoundary: x = y;`
      だけを対象とする spec-derived active pass fixture を追加する。
    - 受入条件: parser と resolver が 2 reserve declaration と theorem を実行し、
      runner は real two-binding handoff を再利用して `BindingId(0)` / `BindingId(1)`
      を保持し、両 source binding 後の use ordinal を導出し、operand ごとの
      pre-normalization result/expected `TypeExpressionInput` に distinct written
      type range を保持する。
    - semantic builtin `set` は、source が deterministic canonical representative
      である 1 `NormalizedTypeId` に intern されてよい。normalization 前の両 original
      input を検証し、両 source range の付与だけを目的に duplicate normalized node
      を要求・捏造してはならない。
    - 2 `Inferred` variable term と 1 fact-free `Checked` equality、production
      invariant validation、task-specific invalid key、near-miss matrix、real
      frontend/resolver sidecar test を要求する。既存 expectation を rebaseline しない。
    - exact multiple-declaration type/well-formedness だけを credit する。implicit
      closure/order、equality truth/fact、theorem acceptance、`formula_statement`、
      proof、CoreIr、ControlFlowIr、VC を materialize しない。
    - spec coverage audit の Chapter 4、13、14、16 を更新する。
      `crates/mizar-checker/src/` が変わらない限り checker source-layout update は不要。
    - 依存: tasks 20、119、123。mizar-test と full workspace を検証する。

125. **Exact source-derived heterogeneous-reserve membership checker bridge を追加する。** [x]
    - `reserve x for object; reserve y for set; theorem HeterogeneousReserveMembershipPayloadBoundary: x in y;`
      だけを対象とする spec-derived active pass fixture を追加する。
    - 受入条件: parser と resolver が 2 reserve declaration と theorem を実行し、
      runner は real mixed-builtin two-binding handoff を再利用して `BindingId(0)` /
      `BindingId(1)` と source-derived lookup ordinal を保持し、両 distinct written
      type range を左 `object` result input、右 `set` result input、右
      expected-`set` input に保持する。
    - exactly two normalized semantic identity を要求する。右 result/expected input
      は `set` identity を共有し、左 `object` identity は distinct のままでなければ
      ならない。各 normalized source representative は original written input から
      導出し、2 type を collapse したり duplicate semantic node を捏造したりしない。
    - 2 `Inferred` variable term、1 fact-free `Checked` membership、production
      invariant validation、task-specific invalid key、exact near-miss matrix、real
      frontend/resolver sidecar test を要求する。既存 expectation を rebaseline しない。
    - exact heterogeneous membership type/well-formedness だけを credit する。
      membership truth/fact、object/set coercion evidence、implicit closure/order、
      theorem acceptance、`formula_statement`、proof、CoreIr、ControlFlowIr、VC を
      materialize しない。
    - spec coverage audit の Chapter 3、4、13、14、16 を更新する。
      `crates/mizar-checker/src/` が変わらない限り checker source-layout update は不要。
    - 依存: tasks 20、120、124。mizar-test と full workspace を検証する。

126. **Exact direct-local-mode reserved-variable equality checker bridge を追加する。** [x]
    - bare builtin `set` RHS を持つ unique、unrecovered、source-preceding、
      no-argument local mode definition 1 個、その mode の `reserve x`、
      `theorem LocalModeReservedVariableEqualityPayloadBoundary: x = x;` だけを
      含む spec-derived active pass fixture を追加する。
    - 受入条件: task-55 real AST-derived `ModeExpansion` を再利用し、4 個すべての
      pre-normalization result/expected input に local-mode symbol/range を保持し、
      extracted expansion で `TermFormulaChecker` を構築する。両 source-order lookup
      は `BindingId(0)` を解決し、checker は全 role を 1 builtin-`set` identity に
      normalize する。
    - 2 `Inferred` variable、1 fact-free `Checked` equality、production invariant
      validation、task-specific invalid key、withheld mode-definition family 全体の near
      miss、real frontend/resolver sidecar test を要求する。既存 expectation を
      rebaseline しない。
    - exact mode-backed identifier equality type/well-formedness slice だけを credit
      する。mode definition declaration checking、accepted mode status/inhabitation
      evidence、broader/chained mode、closure/order、truth/fact、theorem acceptance、
      `formula_statement`、proof、CoreIr、ControlFlowIr、VC を credit しない。
    - spec coverage audit の Chapter 4、7、13、14、16 を更新する。
      `crates/mizar-checker/src/` が変わらない限り source-layout update は不要。
    - 依存: tasks 55、119。mizar-test と full workspace を検証する。

127. **Exact one-edge local-mode-chain reserved-variable equality checker bridge を追加する。** [x]
    - separate、unique、unrecovered、source-preceding、no-argument mode definition
      `BaseModeFormula -> set` と `ChainModeFormula -> BaseModeFormula` を exact に
      2 個、`reserve x for ChainModeFormula` 1 個、theorem
      `ChainedLocalModeReservedVariableEqualityPayloadBoundary: x = x;` を持つ
      spec-derived active pass fixture を追加する。
    - 受入条件: task-56 real AST-derived `ModeExpansion` entry 2 個を再利用し、4 raw
      result/expected input すべてに outer `ChainModeFormula` symbol/range を保持し、
      全 role を terminal `set` RHS を canonical source とする 1 builtin-`set`
      identity へ recursively normalize する。両 source-order lookup は
      `BindingId(0)` を解決する。
    - 2 `Inferred` variable、1 fact-free `Checked` equality、production invariant
      validation、task-specific invalid key、exact withheld-family near-miss matrix、
      real frontend/resolver sidecar を要求する。既存 expectation を rebaseline しない。
    - exact one-edge-chain identifier equality type/well-formedness slice だけを
      credit する。mode declaration checking/acceptance、inhabitation evidence、
      object terminal、longer-chain formula、closure/order、truth/fact、theorem
      acceptance、`formula_statement`、proof、CoreIr、ControlFlowIr、VC は deferred
      のままにする。
    - spec coverage audit の Chapter 4、7、13、14、16 を更新する。
      `crates/mizar-checker/src/` が変わらない限り source-layout update は不要。
    - 依存: tasks 56、126。mizar-test と full workspace を検証する。

128. **Exact direct local-object-mode reserved-variable equality checker bridge を追加する。** [x]
    - unique、unrecovered、source-preceding、no-argument definition
      `mode LocalObjectModeDef: LocalObjectMode is object;` 1 個、
      `reserve x for LocalObjectMode` 1 個、theorem
      `LocalObjectModeReservedVariableEqualityPayloadBoundary: x = x;` を持つ
      spec-derived active pass fixture を追加する。
    - 受入条件: task-55 real AST-derived object `ModeExpansion` を再利用し、4 raw
      result/expected input すべてに `LocalObjectMode` symbol/range を保持し、全 role
      を real RHS を canonical source とする 1 builtin-`object` identity へ normalize
      する。両 use は `BindingId(0)` を解決する。
    - 2 `Inferred` variable、1 fact-free `Checked` equality、production validation、
      task invalid key、withheld-family near miss、real frontend/resolver sidecar を
      要求する。既存 expectation を rebaseline しない。
    - exact direct object-mode equality type/well-formedness slice だけを credit する。
      mode declaration checking/acceptance、inhabitation evidence、closure/order、
      truth/fact、theorem acceptance、`formula_statement`、proof、CoreIr、
      ControlFlowIr、VC は deferred のままにする。
    - spec coverage audit の Chapter 3、4、7、13、14、16 を更新する。checker source
      が変わらない限り source-layout update は不要。
    - 依存: tasks 55、126。mizar-test と full workspace を検証する。

129. **Exact one-edge local-object-mode-chain reserved-variable equality checker bridge を追加する。** [x]
    - task 56 の exact `BaseObjectMode -> object` と
      `ChainObjectMode -> BaseObjectMode` definition block、
      `reserve z for ChainObjectMode` 1 個、theorem
      `ChainedLocalObjectModeReservedVariableEqualityPayloadBoundary: z = z;`
      を再利用する spec-derived active pass fixture を追加する。
    - acceptance: 4 raw result/expected input すべてに `ChainObjectMode`
      symbol/range を保持し、両 real expansion を消費して全 role を terminal RHS
      が canonical source の 1 builtin-`object` identity に normalize し、両 use を
      `BindingId(0)` に解決する。
    - 2 `Inferred` variable、1 fact-free `Checked` equality、production
      validation、invalid-link corruption、withheld-family near miss、real
      frontend/resolver sidecar を要求する。既存 expectation は rebaseline しない。
    - この exact one-edge object-terminal equality type/well-formedness slice
      だけを credit する。mode declaration acceptance/inhabitation、closure/order、
      truth/fact、theorem acceptance、`formula_statement`、proof、CoreIr、
      ControlFlowIr、VC は deferred のままにする。
    - spec coverage audit の Chapter 3、4、7、13、14、16 を更新する。checker
      source が変わらない限り source-layout 更新は不要である。
    - 依存: tasks 56、127、128。mizar-test と full workspace を検証する。

130. **Exact direct-local-mode reserved-variable inequality checker bridge を追加する。** [x]
    - exact bare-set `LocalModeInequality`、reserve 1 個、`x <> x` theorem を持つ
      spec-derived active pass source を追加する。
    - 4 raw mode-headed result/expected input を保持し、real expansion 1 本を
      消費し、RHS 起点の builtin-set identity 1 個、両 `BindingId(0)` use、
      fact-free pre-desugaring `Checked` inequality を要求する。exact/near-miss/
      corruption と real-sidecar guard を必須とする。
    - declaration acceptance/inhabitation、desugaring、closure/order、truth/fact、
      theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 4、7、13、14、16 を更新する。依存: tasks 55、121。

131. **Exact direct-local-object-mode reserved-variable inequality checker bridge を追加する。** [x]
    - exact bare-object `LocalObjectModeInequality`、reserve 1 個、`x <> x`
      theorem を持つ spec-derived active pass source を追加する。
    - 4 raw object-mode-headed result/expected input を保持し、real expansion
      1 本を消費し、RHS 起点の builtin-object identity 1 個、両
      `BindingId(0)` use、fact-free pre-desugaring `Checked` inequality を要求する。
      exact/near-miss/corruption と real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、desugaring、closure/order、
      truth/fact、theorem acceptance、proof/Core/ControlFlow/VC は deferred の
      ままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 55、121、128、130。

132. **Exact one-edge local-mode-chain reserved-variable inequality checker bridge を追加する。** [x]
    - exact `ChainModeInequality -> BaseModeInequality -> set`、outer reserve
      1 個、`x <> x` theorem を持つ spec-derived active pass source を追加する。
    - 4 raw outer-mode result/expected input を保持し、real expansion 2 本を
      消費し、terminal RHS 起点の builtin-set identity 1 個、両
      `BindingId(0)` use、fact-free pre-desugaring `Checked` inequality を要求する。
      exact/near-miss/link-corruption と real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、object/direct/longer shape、
      desugaring、closure/order、truth/fact、theorem acceptance、
      proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 4、7、13、14、16 を更新する。依存: tasks 56、121、127、130。

133. **Exact one-edge local-object-mode-chain reserved-variable inequality checker bridge を追加する。** [x]
    - spec-derived `ChainObjectModeInequality -> BaseObjectModeInequality -> object`、outer reserve 1 個、`z <> z` theorem source だけを追加する。
    - 4 raw outer-mode input と `BindingId(0)` を保持し、real expansion 2 本を消費し、terminal-RHS builtin-object identity 1 個、2 `Inferred` term、1 fact-free pre-desugaring `Checked` inequality を要求する。exact、link-corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - declaration acceptance/inhabitation、desugaring、closure/order、truth/fact、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 121、129、131。

134. **Exact two-edge local-mode-chain reserved-variable equality checker bridge を追加する。** [x]
    - spec-derived `OuterTwoEdgeModeEquality -> MiddleTwoEdgeModeEquality -> BaseTwoEdgeModeEquality -> set`、outer reserve 1 個、`z = z` theorem source だけを追加する。
    - 4 raw outer-mode input と `BindingId(0)` を保持し、real expansion 3 本を消費し、terminal-RHS builtin-set identity 1 個、2 `Inferred` term、1 fact-free `Checked` equality を要求する。exact、link-corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - declaration acceptance/inhabitation、implicit closure/order、truth/fact、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 4、7、13、14、16 を更新する。依存: tasks 72、127。

135. **Exact two-edge local-object-mode-chain reserved-variable equality checker bridge を追加する。** [x]
    - spec-derived `OuterTwoEdgeObjectModeEquality -> MiddleTwoEdgeObjectModeEquality -> BaseTwoEdgeObjectModeEquality -> object`、outer reserve 1 個、`z = z` theorem source だけを追加する。
    - 4 raw outer-mode input と `BindingId(0)` を保持し、real expansion 3 本を消費し、terminal-RHS builtin-object identity 1 個、2 `Inferred` term、1 fact-free `Checked` equality を要求する。exact、link-corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - declaration acceptance/inhabitation、implicit closure/order、truth/fact、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 72、134。

136. **Exact two-edge local-mode-chain reserved-variable inequality checker bridge を追加する。** [x]
    - spec-derived `OuterTwoEdgeModeInequality -> MiddleTwoEdgeModeInequality -> BaseTwoEdgeModeInequality -> set`、outer reserve 1 個、`z <> z` theorem source だけを追加する。
    - 4 raw outer-mode input と `BindingId(0)` を保持し、real expansion 3 本を消費し、terminal-RHS builtin-set identity 1 個、2 `Inferred` term、1 fact-free pre-desugaring `Checked` inequality を要求する。exact、link-corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、inequality desugaring、implicit closure/order、truth/fact、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 4、7、13、14、16 を更新する。依存: tasks 72、132。

137. **Exact two-edge local-object-mode-chain reserved-variable inequality checker bridge を追加する。** [x]
    - spec-derived `OuterTwoEdgeObjectModeInequality -> MiddleTwoEdgeObjectModeInequality -> BaseTwoEdgeObjectModeInequality -> object`、outer reserve 1 個、`z <> z` theorem source だけを追加する。
    - 4 raw outer-mode input と `BindingId(0)` を保持し、real expansion 3 本を消費し、terminal-RHS builtin-object identity 1 個、2 `Inferred` term、1 fact-free pre-desugaring `Checked` inequality を要求する。exact、link-corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - declaration acceptance/inhabitation、inequality desugaring、implicit closure/order、truth/fact、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 72、133。

138. **Exact direct-local-mode reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
    - spec-derived `LocalModeTypeAssertion -> set`、その mode の reserve 1 個、`x is set` theorem source だけを追加する。
    - raw local-mode subject と独立した formula-side builtin-set asserted-type input、`BindingId(0)` を保持し、real expansion 1 本を消費して terminal-RHS builtin-set identity 1 個、1 `Inferred` term、1 fact-free `Checked` type assertion を要求する。exact、expansion-corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、formula-side local-mode asserted head、general reachability/widening/`qua`、truth/fact、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 55、122。

139. **Exact direct-local-mode left reserved-variable membership checker bridge を追加する。** [x]
    - spec-derived `LocalModeMembership -> set`、その mode の `x` と explicit `set` の `y` から成る ordered reserve、`x in y` theorem source だけを追加する。
    - raw local-mode left result と独立した right result/expected-set input、`BindingId(0/1)` を保持し、real expansion 1 本を消費して terminal-RHS builtin-set identity 1 個、2 `Inferred` term、right expected constraint だけを持つ 1 fact-free `Checked` membership を要求する。exact、expansion/right-expected corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、membership truth/fact、implicit closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 4、7、13、14、16 を更新する。依存: tasks 55、120、125。

140. **Exact direct-local-object-mode left reserved-variable membership checker bridge を追加する。** [x]
    - spec-derived `LocalObjectModeMembership -> object`、その mode の `x` と explicit `set` の `y` から成る ordered reserve、`x in y` theorem source だけを追加する。
    - raw local-object-mode left result と独立した right result/expected-set input、`BindingId(0/1)` を保持し、real expansion 1 本を消費して distinct terminal-RHS builtin-object identity と explicit-reserve builtin-set identity、2 `Inferred` term、right expected constraint だけを持つ 1 fact-free `Checked` membership を要求する。exact、expansion/right-expected corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、membership truth/fact、object/set coercion、implicit closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 55、125、139。

141. **Exact one-edge local-mode-chain left reserved-variable membership checker bridge を追加する。** [x]
    - spec-derived `ChainModeMembership -> BaseModeMembership -> set`、outer mode の `x` と explicit `set` の `y` から成る ordered reserve、`x in y` theorem source だけを追加する。
    - raw outer-mode left result と独立した right result/expected-set input、`BindingId(0/1)` を保持し、real expansion 2 本を消費して terminal-RHS builtin-set identity 1 個、2 `Inferred` term、right expected constraint だけを持つ 1 fact-free `Checked` membership を要求する。exact、独立した chain-link/right-expected corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、membership truth/fact、implicit closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 4、7、13、14、16 を更新する。依存: tasks 56、125、139。

142. **Exact one-edge local-object-mode-chain left reserved-variable membership checker bridge を追加する。** [x]
    - spec-derived `ChainObjectModeMembership -> BaseObjectModeMembership -> object`、outer mode の `x` と explicit `set` の `y` から成る ordered reserve、`x in y` theorem source だけを追加する。
    - raw outer-mode left result と独立した right result/expected-set input、`BindingId(0/1)` を保持し、real expansion 2 本を消費して distinct terminal-RHS builtin-object / explicit-reserve builtin-set identity、2 `Inferred` term、right expected constraint だけを持つ 1 fact-free `Checked` membership を要求する。exact、独立した chain-link/right-expected corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、membership truth/fact、object/set coercion、implicit closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 56、125、140、141。

143. **Exact two-edge local-mode-chain left reserved-variable membership checker bridge を追加する。** [x]
    - spec-derived `OuterTwoEdgeModeMembership -> MiddleTwoEdgeModeMembership -> BaseTwoEdgeModeMembership -> set`、outer mode の `x` と explicit `set` の `y` から成る ordered reserve、`x in y` theorem source だけを追加する。
    - raw outer-mode left result と独立した right result/expected-set input、`BindingId(0/1)` を保持し、real expansion 3 本を消費して terminal-RHS builtin-set identity 1 個、2 `Inferred` term、right expected constraint だけを持つ 1 fact-free `Checked` membership を要求する。exact、独立した 3 link/right-expected corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、membership truth/fact、implicit closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 4、7、13、14、16 を更新する。依存: tasks 72、125、139、141。

144. **Exact two-edge local-object-mode-chain left reserved-variable membership checker bridge を追加する。** [x]
    - spec-derived `OuterTwoEdgeObjectModeMembership -> MiddleTwoEdgeObjectModeMembership -> BaseTwoEdgeObjectModeMembership -> object`、outer mode の `x` と explicit `set` の `y` から成る ordered reserve、`x in y` theorem source だけを追加する。
    - raw outer-mode left result と独立した right result/expected-set input、`BindingId(0/1)` を保持し、real expansion 3 本を消費して distinct terminal-RHS builtin-object / explicit-reserve builtin-set identity、2 `Inferred` term、right expected constraint だけを持つ 1 fact-free `Checked` membership を要求する。exact、独立した 3 link/right-expected corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、membership truth/fact、object/set coercion、implicit closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 72、125、140、142、143。

145. **Exact direct local-object-mode reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
    - spec-derived `LocalObjectModeTypeAssertion -> object`、その mode の reserve `x` 1 個、`x is object` theorem source だけを追加する。
    - raw local-mode subject result と独立した formula-anchored builtin-object asserted type を保持し、`BindingId(0)` と source-order use ordinal 1 を要求し、real expansion 1 本を消費して両 input を terminal-RHS builtin-object identity 1 個へ normalize してから 1 `Inferred` term と 1 fact-free `Checked` type assertion を要求する。exact、definition/expansion corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、formula-side local-mode asserted head、general reachability/widening/`qua`、object/set coercion、truth/fact、implicit closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 55、122、138。

146. **Exact one-edge local-mode-chain reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
    - spec-derived `ChainModeTypeAssertion -> BaseModeTypeAssertion -> set`、outer mode の reserve `x` 1 個、`x is set` theorem source だけを追加する。
    - raw outer-mode subject result と独立した formula-anchored builtin-set asserted type を保持し、`BindingId(0)` と source-order use ordinal 1 を要求し、real expansion 2 本を消費して両 input を terminal-RHS builtin-set identity 1 個へ再帰的に normalize してから 1 `Inferred` term と 1 fact-free `Checked` type assertion を要求する。exact、独立した definition/expansion corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、formula-side local-mode asserted head、general reachability/widening/`qua`、truth/fact、implicit closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 56、122、138。

147. **Exact one-edge local-object-mode-chain reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
    - spec-derived `ChainObjectModeTypeAssertion -> BaseObjectModeTypeAssertion -> object`、outer mode の reserve `x` 1 個、`x is object` theorem source だけを追加する。
    - raw outer-mode subject result と独立した formula-anchored builtin-object asserted type を保持し、`BindingId(0)` と source-order use ordinal 1 を要求し、real expansion 2 本を消費して両 input を terminal-RHS builtin-object identity 1 個へ再帰的に normalize してから 1 `Inferred` term と 1 fact-free `Checked` type assertion を要求する。exact、独立した definition/expansion corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、formula-side local-mode asserted head、general reachability/widening/`qua`、object/set coercion、truth/fact、implicit closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 56、122、145、146。

148. **Exact two-edge local-mode-chain reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
    - spec-derived `OuterTwoEdgeModeTypeAssertion -> MiddleTwoEdgeModeTypeAssertion -> BaseTwoEdgeModeTypeAssertion -> set`、outer mode の reserve `x` 1 個、`x is set` theorem source だけを追加する。
    - raw outer-mode subject result と独立した formula-anchored builtin-set asserted type を保持し、`BindingId(0)` と source-order use ordinal 1 を要求し、real expansion 3 本を消費して両 input を terminal-RHS builtin-set identity 1 個へ再帰的に normalize してから 1 `Inferred` term と 1 fact-free `Checked` type assertion を要求する。exact、独立した definition/expansion corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、formula-side local-mode asserted head、general reachability/widening/`qua`、truth/fact、implicit closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 72、122、146、147。

149. **Exact two-edge local-object-mode-chain reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
    - spec-derived `OuterTwoEdgeObjectModeTypeAssertion -> MiddleTwoEdgeObjectModeTypeAssertion -> BaseTwoEdgeObjectModeTypeAssertion -> object`、outer mode の reserve `x` 1 個、`x is object` theorem source だけを追加する。
    - raw outer-mode subject result と独立した formula-anchored builtin-object asserted type を保持し、`BindingId(0)` と source-order use ordinal 1 を要求し、real expansion 3 本を消費して両 input を terminal-RHS builtin-object identity 1 個へ再帰的に normalize してから 1 `Inferred` term と 1 fact-free `Checked` type assertion を要求する。exact、独立した definition/expansion corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、formula-side local-mode asserted head、general reachability/widening/`qua`、object/set coercion、truth/fact、implicit closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 72、122、145、147、148。

150. **Exact three-edge local-mode-chain reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
    - spec-derived `OuterThreeEdgeModeTypeAssertion -> MiddleThreeEdgeModeTypeAssertion -> InnerThreeEdgeModeTypeAssertion -> BaseThreeEdgeModeTypeAssertion -> set`、outer mode の reserve `x` 1 個、`x is set` theorem source だけを追加する。
    - raw outer-mode subject result と独立した formula-anchored builtin-set asserted type を保持し、`BindingId(0)` と source-order use ordinal 1 を要求し、real expansion 4 本を消費して両 input を terminal-RHS builtin-set identity 1 個へ再帰的に normalize してから 1 `Inferred` term と 1 fact-free `Checked` type assertion を要求する。exact、独立した definition/radix/expansion corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、formula-side local-mode asserted head、general reachability/widening/`qua`、truth/fact、implicit closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 73、122、148、149。

151. **Exact three-edge local-object-mode-chain reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
    - spec-derived `OuterThreeEdgeObjectModeTypeAssertion -> MiddleThreeEdgeObjectModeTypeAssertion -> InnerThreeEdgeObjectModeTypeAssertion -> BaseThreeEdgeObjectModeTypeAssertion -> object`、outer mode の reserve `x` 1 個、`x is object` theorem source だけを追加する。
    - raw outer-mode subject result と独立した formula-anchored builtin-object asserted type を保持し、`BindingId(0)` と source-order use ordinal 1 を要求し、real expansion 4 本を消費して両 input を terminal-RHS builtin-object identity 1 個へ再帰的に normalize してから 1 `Inferred` term と 1 fact-free `Checked` type assertion を要求する。exact、独立した definition/radix/expansion corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、formula-side local-mode asserted head、general reachability/widening/`qua`、object/set coercion、truth/fact、implicit closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 73、122、149、150。

152. **Exact four-edge local-mode-chain reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
    - spec-derived `TooDeepFourEdgeModeTypeAssertion -> OuterFourEdgeModeTypeAssertion -> MiddleFourEdgeModeTypeAssertion -> InnerFourEdgeModeTypeAssertion -> BaseFourEdgeModeTypeAssertion -> set`、outermost mode の reserve `x` 1 個、`x is set` theorem source だけを追加する。
    - raw outermost-mode subject result と独立した formula-anchored builtin-set asserted type を保持し、`BindingId(0)` と source-order use ordinal 1 を要求し、real expansion 5 本を消費して両 input を terminal-RHS builtin-set identity 1 個へ再帰的に normalize してから 1 `Inferred` term と 1 fact-free `Checked` type assertion を要求する。exact、独立した definition/radix/expansion corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、formula-side local-mode asserted head、general reachability/widening/`qua`、truth/fact、implicit closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 74、122、150、151。

153. **Exact four-edge local-object-mode-chain reserved-variable normalized-reflexive type assertion checker bridge を追加する。** [x]
    - spec-derived `TooDeepFourEdgeObjectModeTypeAssertion -> OuterFourEdgeObjectModeTypeAssertion -> MiddleFourEdgeObjectModeTypeAssertion -> InnerFourEdgeObjectModeTypeAssertion -> BaseFourEdgeObjectModeTypeAssertion -> object`、outermost mode の reserve `x` 1 個、`x is object` theorem source だけを追加する。
    - raw outermost-mode subject result と独立した formula-anchored builtin-object asserted type を保持し、`BindingId(0)` と source-order use ordinal 1 を要求し、real expansion 5 本を消費して terminal-RHS builtin-object identity 1 個、1 `Inferred` term、1 fact-free `Checked` type assertion を要求する。exact、definition/radix/expansion corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、formula-side local-mode asserted head、general reachability/widening/`qua`、object/set coercion、truth/fact、closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 3、4、7、13、14、16 を更新する。依存: tasks 74、122、151、152。

154. **Exact three-edge local-mode-chain reserved-variable equality checker bridge を追加する。** [x]
    - spec-derived `OuterThreeEdgeModeEquality -> MiddleThreeEdgeModeEquality -> InnerThreeEdgeModeEquality -> BaseThreeEdgeModeEquality -> set`、outer mode の reserve `z` 1 個、`z = z` theorem source だけを追加する。
    - raw outer-mode result/expected input 4 個を保持し、両 operand を source-order ordinal 1、2 で独立に `BindingId(0)` へ解決し、real expansion 4 本を消費して全 role を terminal-RHS builtin-set identity 1 個へ normalize してから 2 `Inferred` variable と 1 fact/deferred-free `Checked` equality を要求する。exact definition/radix/expansion corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、equality truth/fact、closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままにする。
    - Chapter 4、7、13、14、16 を更新する。依存: tasks 73、134。

155. **Exact three-edge local-object-mode-chain reserved-variable equality checker bridge を追加する。** [x]
    - spec-derived `OuterThreeEdgeObjectModeEquality -> MiddleThreeEdgeObjectModeEquality -> InnerThreeEdgeObjectModeEquality -> BaseThreeEdgeObjectModeEquality -> object`、outer mode の reserve `z` 1 個、`z = z` theorem source だけを追加する。
    - raw outer-mode result/expected input 4 個を保持し、両 operand を source-order ordinal 1、2 で独立に `BindingId(0)` へ解決し、real expansion 4 本を消費して全 role を terminal-RHS builtin-object identity 1 個へ normalize してから 2 `Inferred` variable と 1 fact/deferred-free `Checked` equality を要求する。exact definition/radix/expansion corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、object/set coercion、equality truth/fact、closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままとする。
    - Chapter 4、7、13、14、16 を更新する。依存: tasks 73、135。

156. **Exact three-edge local-mode-chain reserved-variable inequality checker bridge を追加する。** [x]
    - spec-derived `OuterThreeEdgeModeInequality -> MiddleThreeEdgeModeInequality -> InnerThreeEdgeModeInequality -> BaseThreeEdgeModeInequality -> set`、outer mode の reserve `z` 1 個、`z <> z` theorem source だけを追加する。
    - raw outer-mode result/expected input 4 個を保持し、両 operand を source-order ordinal 1、2 で独立に `BindingId(0)` へ解決し、real expansion 4 本を消費して全 role を terminal-RHS builtin-set identity 1 個へ normalize してから 2 `Inferred` variable と 1 fact/deferred-free pre-desugaring `Checked` inequality を要求する。exact definition/radix/expansion corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、inequality desugaring、truth/fact、closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままとする。
    - Chapter 4、7、13、14、16 を更新する。依存: tasks 73、136。

157. **Exact three-edge local-object-mode-chain reserved-variable inequality checker bridge を追加する。** [x]
    - spec-derived `OuterThreeEdgeObjectModeInequality -> MiddleThreeEdgeObjectModeInequality -> InnerThreeEdgeObjectModeInequality -> BaseThreeEdgeObjectModeInequality -> object`、outer mode の reserve `z` 1 個、`z <> z` theorem source だけを追加する。
    - raw outer-mode result/expected input 4 個を保持し、両 operand を source-order ordinal 1、2 で独立に `BindingId(0)` へ解決し、real expansion 4 本を消費して全 role を terminal-RHS builtin-object identity 1 個へ normalize してから 2 `Inferred` variable と 1 fact/deferred-free pre-desugaring `Checked` inequality を要求する。exact definition/radix/expansion corruption、withheld-family near-miss、real-sidecar guard を必須とする。
    - mode declaration acceptance/inhabitation、object/set coercion、inequality desugaring、truth/fact、closure/order、theorem acceptance、proof/Core/ControlFlow/VC は deferred のままとする。
    - Chapter 4、7、13、14、16 を更新する。依存: tasks 73、137。

158. **Exact three-edge local-mode-chain left reserved-variable membership checker bridge を追加する。** [x]
    - spec-derived `OuterThreeEdgeModeMembership -> MiddleThreeEdgeModeMembership -> InnerThreeEdgeModeMembership -> BaseThreeEdgeModeMembership -> set`、outer mode の `x` と explicit `set` の `y` から成る ordered reserve、`x in y` theorem source だけを追加する。
    - raw outer-mode left result と独立した explicit-set right result/sole expected input を保持し、source-order ordinal 2/3 で `BindingId(0/1)` へ解決し、real expansion 4 本を消費して全 3 role を terminal-RHS builtin-set identity 1 個へ normalize してから 2 `Inferred` variable と 1 fact/deferred-free `Checked` membership を要求する。exactly one right-owned constraint と no left expected type を必須とする。
    - 全 non-exact definition/radix/expansion、reserve、formula、terminal、chain depth、recovery、context、parameter、argument、cycle、extra-item shape を reject する。declaration acceptance/inhabitation、membership truth/fact、closure/order、theorem/proof/Core/VC、object-terminal behavior、broader shape は deferred のままにする。
    - spec-derived active `.miz`、expectation、trace row、unit/near-miss/corruption tests、real frontend/resolver sidecar、metadata、bilingual docs、coverage audit を追加する。Chapter 4、7、13、14、16 を更新する。依存: tasks 73、143。

159. **Exact distinct-binding shared-reserve membership checker bridge を追加する。** [x]
    - `reserve x, y for set; theorem DistinctReservedVariableMembershipPayloadBoundary: x in y;` だけを追加する。
    - ordinal 2/3 の `BindingId(0/1)` と left-result/right-result/right-expected role に written set range 1 個を保持し、left expected は持たず、normalized builtin-set identity 1 個、2 `Inferred` variable、1 fact/deferred-free `Checked` membership、right-owned constraint 1 個を要求する。
    - non-exact reserve/formula と matched-output corruption を拒否する。truth/fact、closure/order、theorem/proof/Core/ControlFlow/VC、separate declaration、broader shape は deferred のままとする。
    - fixture/expectation/trace、unit/near-miss/corruption/real-sidecar test、metadata、bilingual docs、audit を更新した。active runner は 110 件である。Chapter 4、13、14、16。Deps: tasks 120、123、125。

160. **Exact distinct-binding shared-reserve inequality checker bridge を追加する。** [x]
    - `reserve x, y for set; theorem DistinctReservedVariableInequalityPayloadBoundary: x <> y;` だけを追加する。
    - ordinal 2/3 の `BindingId(0/1)` と result/expected role pair 2 組に written set range 1 個を保持し、normalized builtin-set identity 1 個、2 `Inferred` variable、2 ordered operand-owned constraint を持つ 1 fact/deferred-free pre-desugaring `Checked` inequality を要求する。
    - non-exact reserve/formula shape、route collision、matched-output corruption を拒否する。desugaring/truth/fact、closure/order、theorem/proof/Core/ControlFlow/VC、separate declaration、broader shape は deferred のままとする。
    - fixture/expectation/trace contract に production route、unit/near-miss/corruption/real-sidecar test、metadata、bilingual docs、audit を追加した。active runner は 111 件である。Chapter 4、13、14、16。Deps: tasks 121、123。

161. **Exact multiple-reserve-declaration inequality checker bridge を追加する。** [x]
    - `reserve x for set; reserve y for set; theorem MultipleReserveDeclarationInequalityPayloadBoundary: x <> y;` だけを追加する。
    - ordinal 2/3 の `BindingId(0/1)` と result/expected pair 2 組に distinct written range を保持しつつ earlier x range に canonical anchor された builtin-set identity 1 個へ intern し、2 `Inferred` variable、2 ordered constraint を持つ 1 fact/deferred-free pre-desugaring `Checked` inequality を要求する。
    - non-exact declaration order/shape、formula、route collision、matched-output corruption を拒否する。desugaring/truth/fact、closure/order、theorem/proof/Core/ControlFlow/VC、shared range、broader shape は deferred のままとする。
    - source/trace contract に production route、unit/near-miss/corruption/real-sidecar test、metadata、bilingual docs、audit を追加した。active runner は 112 件である。Chapter 4、13、14、16。Deps: tasks 124、160。

162. **Exact multiple-reserve-declaration membership checker bridge を追加する。** [x]
    - `reserve x for set; reserve y for set; theorem MultipleReserveDeclarationMembershipPayloadBoundary: x in y;` だけを追加する。
    - ordinal 2/3 の `BindingId(0/1)` に distinct written range を保持し、first range は left result、second range は right result と sole right expected input に保持する。left expected input は持たず、earlier x range に canonical anchor された builtin-set identity 1 個、2 `Inferred` variable、exactly one right-owned constraint を持つ 1 fact/deferred-free `Checked` membership を要求する。
    - non-exact declaration order/shape、formula、route collision、matched-output corruption を拒否する。membership truth/fact、closure/order、theorem/proof/Core/ControlFlow/VC、shared range、broader shape は deferred のままとする。
    - fixture/expectation/trace contract に production route、unit/near-miss/corruption/real-sidecar test、metadata、bilingual docs、audit を追加した。active runner は 113 件である。Chapter 4、13、14、16。Deps: tasks 120、124、159。

87. **Source-derived term formula extraction-gap boundary を追加する。** [x]
    - `theorem TermFormulaPayloadBoundary: 1 = 1;` のように source term を含む
      theorem formula について、専用の active `type_elaboration` boundary を追加する。
    - Acceptance: parser と resolver は source を実行し、その後 active runner は
      当初 `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      task 106 はこの exact builtin equality slice を supersede し、real checker
      term/formula payload を抽出したうえで missing numeric type payload と partial
      formula checking で fail closed する。
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
    - Historical boundary: `parser.type_fixtures` を import し、`divides` や
      `++` のような documented imported predicate/functor surface を使う theorem
      formula 専用の active `type_elaboration` boundary を追加した。
    - task 110 は exact
      `ImportedPredicateFunctorPayloadBoundary: 1 divides (1 ++ 2)` source を
      supersede し、real checker numeral、imported functor-application、
      predicate-application payload を渡してから missing numeric/signature
      payload と partial formula checking で fail closed する。task 98 は
      parser / resolver executable な extraction-gap boundary として historical
      に残り、imported module AST extraction、semantic predicate/functor
      signature、term inference、formula checking、recorded fact、theorem
      acceptance、CoreIr、ControlFlowIr、VC、proof payload、`formula_statement`
      runner support として読んではならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: tasks 86、87。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 11 symbol management、spec 12 modules and namespaces、
      spec 13 term expressions、spec 14 formulas、spec 16 theorems and proofs。

99. **Source-derived formula connective/quantifier extraction-gap boundary を追加する。** [x]
    - implication、universal quantification、negation など Chapter 14 の
      connective / quantifier surface を使う theorem formula 専用の active
      `type_elaboration` boundary を追加する。
    - Acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned formula payload extraction、quantifier binder/context payload、
      formula checking、recorded fact、theorem acceptance、CoreIr、ControlFlowIr、
      VC、proof payload、`formula_statement` runner がまだ利用できないためである。
      この task は formula payload、quantifier binder/context payload、fact、
      theorem acceptance、downstream semantic payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: tasks 86、87、98。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 14 formulas、spec 16 theorems and proofs。

112. **Exact source-derived formula connective/quantifier shell checker bridge を追加する。** [x]
    - exact unrecovered
      `FormulaConnectiveQuantifierPayloadBoundary: contradiction implies for x being set holds not contradiction`
      theorem source だけについて task 99 を supersede する。
    - Acceptance: parser と resolver は source を実行し、runner は implication、
      universal quantification、negation shell の real source site/range を抽出し、
      それらの checker `FormulaInput` を `TermFormulaChecker` に渡して、
      missing formula/quantifier payload で fail closed する。task 117 は後続で
      この source 内の 2 つの exact contradiction constant だけを real formula
      constant kind payload に進める。この bridge は child-formula link、
      binder/context payload、formula fact/checking、theorem acceptance、
      `formula_statement`、CoreIr、ControlFlowIr、VC、proof payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: tasks 86、99、106、110、111。参照: Step 5 source-derived semantic
      bridge、mizar-test task 10、spec 14 formulas、spec 16 theorems and proofs。

100. **Source-derived builtin membership formula extraction-gap boundary を追加する。** [x]
    - Chapter 14 の builtin membership predicate と Chapter 13 の numeral term
      operand を使う theorem formula 専用の active `type_elaboration` boundary を
      追加する。
    - Acceptance: parser と resolver は source を実行する。task 108 はこの exact
      sidecar を supersede し、real checker term/formula payload を渡して missing
      numeric type payload と partial formula checking を報告する。この task は
      numeric type payload、membership operand expected-type construction/checking、
      fact、theorem acceptance、`formula_statement`、CoreIr、ControlFlowIr、VC、
      proof payload、downstream semantic payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: tasks 86、87、98。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 13 term expressions、spec 14 formulas、spec 16
      theorems and proofs。

101. **Source-derived builtin inequality formula extraction-gap boundary を追加する。** [x]
    - Chapter 14 の builtin inequality predicate と Chapter 13 の numeral term
      operand を使う theorem formula 専用の active `type_elaboration` boundary を
      追加する。
    - Acceptance: parser と resolver は source を実行する。task 107 はこの exact
      sidecar を supersede し、real checker term/formula payload を渡して missing
      numeric type payload と partial formula checking を報告する。この task は
      numeric type payload、inequality desugaring / equality semantic checking、
      fact、theorem acceptance、`formula_statement`、CoreIr、ControlFlowIr、VC、
      proof payload、downstream semantic payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: tasks 86、87、98、100。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 13 term expressions、spec 14 formulas、spec 16
      theorems and proofs。

102. **Source-derived builtin type assertion formula extraction-gap boundary を追加する。** [x]
    - Chapter 14 の builtin type-assertion form と Chapter 13 の numeral term を
      使う theorem formula 専用の active `type_elaboration` boundary を追加する。
    - task 109 は exact builtin `set` theorem source だけを source-derived checker
      `TermInput`、`FormulaInput`、asserted `TypeExpressionInput` payload に
      supersede し、missing numeric type payload と partial formula checking で
      fail closed する。より広い asserted type payload extraction、
      type-assertion semantic checking、recorded fact、theorem acceptance、CoreIr、
      ControlFlowIr、VC、proof payload、`formula_statement` runner はまだ利用
      できない。これらの task は type-assertion fact、theorem acceptance、
      downstream semantic payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: tasks 86、87、98、100、101。参照: Step 5 source-derived semantic
      bridge、mizar-test task 10、spec 13 term expressions、spec 14 formulas、
      spec 16 theorems and proofs。

103. **Source-derived imported attribute assertion formula extraction-gap boundary を追加する。** [x]
    - `parser.type_fixtures` を import し、documented `empty` attribute を
      Chapter 14 の attribute-assertion form と Chapter 13 の numeral subject で
      使う theorem formula 専用の active `type_elaboration` boundary を追加する。
    - Acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned term/formula payload extraction、imported attribute assertion
      attribute-chain/provenance payload extraction、term inference、attribute
      admissibility/semantic checking、formula checking、recorded fact、theorem
      acceptance、CoreIr、ControlFlowIr、VC、proof payload、`formula_statement`
      runner がまだ利用できないためである。この task は term/formula payload、
      imported attribute assertion payload、imported module AST extraction、
      theorem acceptance、downstream semantic payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: tasks 86、87、98、100、101、102。参照: Step 5 source-derived semantic
      bridge、mizar-test task 10、spec 06 attributes、spec 11 symbol management、
      spec 12 modules and namespaces、spec 13 term expressions、spec 14 formulas、
      spec 16 theorems and proofs。

104. **Source-derived attribute-level non-empty imported attribute assertion formula extraction-gap boundary を追加する。** [x]
    - `parser.type_fixtures` を import し、documented `empty` attribute を
      Chapter 14 の attribute-assertion form と Chapter 13 の numeral subject で
      attribute-level `non empty` assertion として使う theorem formula 専用の
      active `type_elaboration` boundary を追加する。
    - Acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned term/formula payload extraction、imported attribute-level
      non-empty assertion attribute-chain/provenance payload extraction、term
      inference、negated attribute admissibility/semantic checking、formula
      checking、recorded fact、theorem acceptance、CoreIr、ControlFlowIr、VC、
      proof payload、`formula_statement` runner がまだ利用できないためである。
      この task は term/formula payload、imported attribute-level non-empty
      assertion payload、imported module AST extraction、theorem acceptance、
      downstream semantic payload を捏造してはならない。
      task 114 は exact
      `ImportedNonEmptyAttributeAssertionPayloadBoundary: 1 is non empty`
      source だけを real checker term/formula handoff で supersede する。
    - 検証: `cargo test -p mizar-test`。
    - 依存: tasks 86、87、98、100、101、102、103。参照: Step 5 source-derived
      semantic bridge、mizar-test task 10、spec 06 attributes、spec 11 symbol
      management、spec 12 modules and namespaces、spec 13 term expressions、
      spec 14 formulas、spec 16 theorems and proofs。

114. **Exact source-derived attribute-level non-empty imported attribute assertion theorem checker bridge を追加する。** [x]
    - exact active source
      `import parser.type_fixtures; theorem ImportedNonEmptyAttributeAssertionPayloadBoundary: 1 is non empty;`
      だけで task 104 を supersede する。
    - Acceptance: parser と resolver は source を実行する。active runner は
      direct `non` surface と imported `empty` provenance を検証し、1 つの
      source-derived numeral `TermInput` と 1 つの attribute-assertion
      `FormulaInput` を抽出し、`TermFormulaChecker` は missing numeric type
      payload、missing formula / attribute semantic payload、partial formula
      checking を報告する。この task は imported module AST extraction、negated
      attribute-chain semantic payload、theorem formula 向け checker
      `AttributeInput` payload、negated attribute admissibility/semantic
      checking、formula checking、theorem acceptance、`formula_statement`、
      CoreIr、ControlFlowIr、VC、proof payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`。
    - 依存: tasks 86、87、98、100、101、102、103、104。参照: Step 5
      source-derived semantic bridge、mizar-test task 10、spec 06 attributes、
      spec 11 symbol management、spec 12 modules and namespaces、spec 13 term
      expressions、spec 14 formulas、spec 16 theorems and proofs。

105. **Source-derived set-enumeration formula extraction-gap boundary を追加する。** [x]
    - Chapter 13 の set-enumeration term operand と Chapter 14 の builtin
      equality を使う theorem formula 専用の active `type_elaboration`
      boundary を追加する。
    - Historical acceptance: parser と resolver は source を実行し、その後 active runner は
      `type_elaboration.external_dependency.ast_payload_extraction` を報告する。
      checker-owned set-enumeration term payload extraction、term/formula
      payload extraction、term inference、equality/formula checking、recorded
      fact、theorem acceptance、CoreIr、ControlFlowIr、VC、proof payload、
      `formula_statement` runner がまだ利用できないためである。この task は
      set-enumeration payload、term/formula payload、theorem acceptance、
      downstream semantic payload を捏造してはならない。
      task 111 は exact `{1, 2} = {1, 2}` source だけを real checker
      term/formula handoff で supersede する。
    - 検証: `cargo test -p mizar-test`。
    - 依存: tasks 86、87、98、100、101、102、103、104。参照: Step 5
      source-derived semantic bridge、mizar-test task 10、spec 13 term
      expressions、spec 14 formulas、spec 16 theorems and proofs。

111. **Exact source-derived set-enumeration theorem checker bridge を追加する。** [x]
    - exact active source
      `theorem SetEnumerationPayloadBoundary: {1, 2} = {1, 2};` だけで
      task 105 を supersede する。
    - Acceptance: parser と resolver は source を実行する。active runner は AST
      から 4 つの source-derived numeral item term、2 つの set-enumeration
      `TermInput`、1 つの builtin equality `FormulaInput` を抽出する。
      `TermFormulaChecker` はその後 missing numeric type payload、missing
      set-enumeration result-type payload、partial formula checking を
      報告する。この task は result type、equality fact/checking、
      theorem acceptance、`formula_statement`、CoreIr、ControlFlowIr、VC、proof
      payload を捏造してはならない。
    - 検証: `cargo test -p mizar-test`; final workspace verification。
    - 依存: tasks 105、106、107、108、109、110。参照: Step 5
      source-derived semantic bridge、mizar-test task 10、spec 13 term
      expressions、spec 14 formulas、spec 16 theorems and proofs。

106. **Source-derived builtin equality theorem term/formula checker bridge を追加する。** [x]
    - unrecovered `TheoremItem -> FormulaExpression -> BuiltinPredicateApplication("=")`
      source shape で、structural Chapter 13 `NumeralTerm` operand が 2 つだけの
      場合に限って昇格する。
    - Acceptance: active runner は real module-shell checker binding context を作り、
      2 つの source-derived `TermInput` と 1 つの equality `FormulaInput` を
      `TermFormulaChecker` に渡し、
      `type_elaboration.checker.checker.term.external.numeric_type_payload` と
      `type_elaboration.checker.checker.formula.term.partial` で fail closed する。
      numeric type payload、equality fact/checking、theorem acceptance、
      `formula_statement` runner、downstream semantic payload は捏造してはならず、
      membership、inequality、type assertion、imported、set-enumeration、
      connective/quantifier、proof theorem surface を昇格してはならない。
    - 検証: `cargo test -p mizar-test --test metadata`。
    - 依存: tasks 86、87。参照: Step 5 source-derived semantic bridge、mizar-test
      task 10、spec 13 term expressions、spec 14 formulas、spec 16 theorems and
      proofs。

108. **Source-derived builtin membership theorem term/formula checker bridge を追加する。** [x]
    - label `BuiltinMembershipPayloadBoundary` を持ち、structural Chapter 13
      `NumeralTerm` operand が `1` と `1` の 2 つだけである unrecovered
      `TheoremItem -> FormulaExpression -> BuiltinPredicateApplication("in")`
      source shape だけを昇格する。
    - Acceptance: active runner は real module-shell checker binding context を作り、
      2 つの source-derived `TermInput` と 1 つの membership `FormulaInput` を
      `TermFormulaChecker` に渡し、
      `type_elaboration.checker.checker.term.external.numeric_type_payload` と
      `type_elaboration.checker.checker.formula.term.partial` で fail closed する。
      numeric type payload、membership operand expected type、membership fact、
      theorem acceptance、`formula_statement` runner、downstream semantic payload は
      捏造してはならず、equality、inequality、type assertion、imported、
      set-enumeration、connective/quantifier、proof theorem surface を昇格しては
      ならない。
    - 検証: `cargo test -p mizar-test --test metadata`。
    - 依存: tasks 86、87、98、100。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 13 term expressions、spec 14 formulas、spec 16
      theorems and proofs。

110. **Source-derived imported predicate/functor theorem checker bridge を追加する。** [x]
    - `parser.type_fixtures` を import し、
      `theorem ImportedPredicateFunctorPayloadBoundary: 1 divides (1 ++ 2);`
      を使う exact source だけを昇格する。
    - Acceptance: active runner は imported `divides` と `++` の resolver
      provenance を検証し、source-derived numeral term、imported
      functor-application term、predicate-application formula を
      `TermFormulaChecker` に渡し、missing numeric/signature payload と partial
      formula checking で fail closed する。imported module AST extraction、
      semantic predicate/functor signature、term inference、formula checking、
      fact、theorem acceptance、`formula_statement`、downstream semantic payload は
      捏造してはならない。
    - 検証: `cargo test -p mizar-test --test metadata`。
    - 依存: tasks 86、87、98。参照: Step 5 source-derived semantic bridge、
      mizar-test task 10、spec 11 symbol management、spec 12 modules and
      namespaces、spec 13 term expressions、spec 14 formulas、spec 16 theorems
      and proofs。

163. **Exact three-edge local-object-mode membership checker bridge を追加する。** [x]
    - crate plan と test-first fixture に記録した spec-derived object-terminal
      definition chain 4 本、ordered outer-mode/set reserve、exact `x in y`
      theorem label だけを昇格する。
    - Acceptance: real expansion 4 本、raw left / explicit-set right provenance、
      ordinal 2/3 の `BindingId(0/1)`、normalized identity 2 個、no left expected
      type、2 inferred term、exactly one right-owned constraint を持つ 1 fact-free
      checked membership を保持する。exact/corruption/near-miss/real frontend-
      resolver test を追加し、coercion、truth、closure、theorem、proof、CoreIr、
      ControlFlowIr、VC を捏造しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 73、144、151、155、157。参照: Step 5、mizar-test task 10、
      specs 3、4、7、13、14.5.3、16。

164. **Exact four-edge local-mode membership checker bridge を追加する。** [x]
    - crate plan と test-first fixture に記録した spec-derived set-terminal
      definition chain 5 本、ordered outermost-mode/set reserve、exact `x in y`
      theorem label だけを昇格する。
    - Acceptance: real expansion 5 本、raw left / explicit-set right provenance、
      ordinal 2/3 の `BindingId(0/1)`、terminal-set-RHS identity 1 個、no left
      expected type、2 inferred term、exactly one right-owned constraint を持つ
      1 fact-free checked membership を保持する。exact/corruption/near-miss/
      real frontend-resolver test を追加し、truth、closure、theorem、proof、
      CoreIr、ControlFlowIr、VC を捏造しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、152、158。参照: Step 5、mizar-test task 10、specs 4、
      7、13、14.5.3、16。

165. **Exact four-edge local-object-mode membership checker bridge を追加する。** [x]
    - crate plan と test-first fixture に記録した spec-derived object-terminal
      definition chain 5 本、ordered outermost-mode/set reserve、exact `x in y`
      theorem label だけを昇格する。
    - Acceptance: real expansion 5 本、raw left / explicit-set right provenance、
      ordinal 2/3 の `BindingId(0/1)`、distinct terminal-object-RHS / explicit-set
      normalized identity、no left expected type、2 inferred term、exactly one
      right-owned constraint を持つ 1 fact-free checked membership を保持する。
      exact/corruption/near-miss/real frontend-resolver test を追加し、object/set
      coercion、truth、closure、theorem、proof、CoreIr、ControlFlowIr、VC を捏造
      しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、153、163。参照: Step 5、mizar-test task 10、specs 3、4、
      7、13、14.5.3、16。

166. **Exact four-edge local-mode equality checker bridge を追加する。** [x]
    - crate plan と test-first fixture に記録した spec-derived set-terminal
      definition chain 5 本、outermost mode reserve 1 個、exact `z = z` theorem
      label だけを昇格する。
    - Acceptance: real expansion 5 本、raw result/expected input 4 個、ordinal
      1/2 の `BindingId(0)`、terminal-set-RHS normalized identity 1 個、2
      inferred term、1 fact/deferred-free checked equality を保持する。exact/
      corruption/near-miss/real frontend-resolver test を追加し、declaration
      acceptance、truth、closure、theorem、proof、CoreIr、ControlFlowIr、VC を
      捏造しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、152、154。参照: Step 5、mizar-test task 10、specs 4、
      7、13、14.5.2、16。

167. **Exact four-edge local-object-mode equality checker bridge を追加する。** [x]
    - crate plan と test-first fixture に記録した spec-derived object-terminal
      definition chain 5 本、outermost mode reserve 1 個、exact `z = z` theorem
      label だけを昇格する。
    - Acceptance: real expansion 5 本、raw result/expected input 4 個、ordinal
      1/2 の `BindingId(0)`、terminal-object-RHS normalized identity 1 個、2
      inferred term、1 fact/deferred-free checked equality、ordered operand-
      owned expected constraint 2 個を保持する。exact/corruption/near-miss/
      real frontend-resolver test を追加し、declaration acceptance、object/set
      coercion、truth、closure、theorem、proof、CoreIr、ControlFlowIr、VC を
      捏造しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、153、155。参照: Step 5、mizar-test task 10、specs 3、4、
      7、13、14.5.2、16。

168. **Exact four-edge local-mode inequality checker bridge を追加する。** [x]
    - crate plan と test-first fixture に記録した spec-derived set-terminal
      definition chain 5 本、outermost mode reserve 1 個、exact `z <> z`
      theorem label だけを昇格する。
    - Acceptance: real expansion 5 本、raw result/expected input 4 個、ordinal
      1/2 の `BindingId(0)`、terminal-set-RHS normalized identity 1 個、2
      inferred term、1 fact/deferred-free pre-desugaring checked inequality、
      ordered operand-owned expected constraint 2 個を保持する。exact/
      corruption/near-miss/real sidecar test を追加し、desugaring、truth、
      declaration acceptance、closure、theorem、proof、CoreIr、ControlFlowIr、
      VC を捏造しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、152、156。参照: Step 5、mizar-test task 10、specs 4、7、
      13、14.5.2、16。

169. **Exact four-edge local-object-mode inequality checker bridge を追加する。** [x]
    - spec-derived object-terminal definition chain 5 本、outermost mode
      reserve 1 個、exact `z <> z` theorem label だけを昇格する。
    - Acceptance: real expansion 5 本、raw result/expected input 4 個、ordinal
      1/2 の `BindingId(0)`、terminal-object-RHS identity 1 個、2 inferred
      term、1 fact/deferred-free pre-desugaring checked inequality、ordered
      operand-owned expected constraint 2 個を object/set coercion なしで保持
      する。exact/corruption/near-miss/real-sidecar test を追加し、desugaring、
      truth、declaration acceptance、closure、theorem、proof、CoreIr、
      ControlFlowIr、VC を捏造しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、153、157。参照: Step 5、mizar-test task 10、specs 3、4、
      7、13、14.5.2、16。

172. **Exact local-mode long-chain equality checker bridge を追加する。** [x]
    - task 74 が既に実行する spec-derived set-terminal definition chain 7 本、
      `ChainMode6` reserve 1 個、test-first fixture に記録した exact `z = z`
      theorem label だけを昇格する。
    - Acceptance: real AST-derived expansion 7 本、raw `ChainMode6` result/
      expected input 4 個、ordinal 1/2 の `BindingId(0)`、terminal `BaseMode`
      RHS の builtin-set identity 1 個、2 inferred term、1 fact/deferred-free
      checked equality、ordered operand-owned expected constraint 2 個を保持
      する。exact/corruption/near-miss/real frontend-resolver test を追加し、
      declaration acceptance、truth、closure、theorem、proof、general
      unbounded semantics、CoreIr、ControlFlowIr、VC を捏造しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、166。参照: Step 5、mizar-test task 10、specs 4、7、13、
      14.5.2、16。

173. **Exact local-mode long-chain inequality checker bridge を追加する。** [x]
    - task 74 の set-terminal definition chain 7 本、`ChainMode6` reserve 1 個、
      test-first fixture の exact `z <> z` theorem label だけを task 168 の
      real inequality consumer へ渡す。
    - Acceptance: real expansion 7 本、raw `ChainMode6` result/expected input
      4 個、ordinal 1/2 の `BindingId(0)`、terminal `BaseMode` RHS の builtin-
      set identity 1 個、2 inferred term、1 fact/deferred-free pre-desugaring
      checked inequality、ordered operand-owned expected constraint 2 個を保持
      する。full exact/near-miss/corruption/real-sidecar guard を必須とし、
      desugaring、truth、acceptance、closure、theorem/proof/CoreIr/
      ControlFlowIr/VC、general unbounded semantics を捏造しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、168。参照: Step 5、mizar-test task 10、specs 4、7、13、
      14.5.2、16。

174. **Exact local-mode long-chain membership checker bridge を追加する。** [x]
    - task 74 の set-terminal definition chain 7 本、ordered reserve `x` for
      `ChainMode6` と `y` for explicit `set`、test-first fixture の exact
      `x in y` theorem label だけを task 164 の real membership consumer へ渡す。
    - Acceptance: real expansion 7 本、raw `ChainMode6` left と独立した explicit-
      set right result/sole right expected input、ordinal 2/3 の
      `BindingId(0/1)`、terminal `BaseMode` RHS builtin-set identity 1 個、left
      expected input なし、2 inferred term、1 fact/deferred-free checked
      membership、right-owned constraint 1 個を保持する。full exact/near-miss/
      corruption/real-sidecar guard を必須とし、truth/fact、acceptance、closure、
      theorem/proof/CoreIr/ControlFlowIr/VC、general unbounded semantics を捏造
      しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、164。参照: Step 5、mizar-test task 10、specs 4、7、13、
      14.5.3、16。

175. **Exact local-mode long-chain type assertion checker bridge を追加する。** [x]
    - task 74 の set-terminal definition chain 7 本、`ChainMode6` reserve 1 個、
      test-first fixture の exact `x is set` theorem label だけを task 152 の
      real normalized-reflexive type-assertion consumer へ渡す。
    - Acceptance: real expansion 7 本、raw `ChainMode6` subject と独立した
      formula-side builtin-set asserted input、ordinal 1 の `BindingId(0)`、
      terminal `BaseMode` RHS builtin-set identity 1 個、1 inferred term、general
      reachability を用いない 1 fact/deferred-free checked type assertion を保持
      する。full exact/near-miss/corruption/real-sidecar guard を必須とし、
      widening/`qua`、truth/fact、acceptance、closure、theorem/proof/CoreIr/
      ControlFlowIr/VC、general unbounded semantics を捏造しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、152。参照: Step 5、mizar-test task 10、specs 3、4、7、13、
      14.2.3、16。

176. **Exact local-object-mode long-chain equality checker bridge を追加する。** [x]
    - task 74 の AST-bounded object-terminal definition chain 7 本、
      `ChainObjectMode6` reserve 1 個、test-first fixture の exact `z = z`
      theorem label だけを task 167 の real object-normalizing equality
      consumer へ渡す。
    - Acceptance: real expansion 7 本、raw `ChainObjectMode6` result/expected
      input 4 個、ordinal 1/2 の `BindingId(0)`、terminal `BaseObjectMode` RHS
      builtin-object identity 1 個、2 inferred term、1 fact/deferred-free
      checked equality、ordered operand-owned constraint 2 個を object/set
      coercion なしで保持する。full exact/near-miss/corruption/real-sidecar
      guard を必須とし、truth/fact、acceptance、closure、theorem/proof/CoreIr/
      ControlFlowIr/VC、general unbounded semantics を捏造しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、167。参照: Step 5、mizar-test task 10、specs 3、4、7、13、
      14.5.2、16。

177. **Exact local-object-mode long-chain inequality checker bridge を追加する。** [x]
    - task 74 の AST-bounded object-terminal definition chain 7 本、
      `ChainObjectMode6` reserve 1 個、test-first fixture の exact `z <> z`
      theorem label だけを task 169 の real object-normalizing pre-desugaring
      inequality consumer へ渡す。
    - Acceptance: real expansion 7 本、raw `ChainObjectMode6` result/expected
      input 4 個、ordinal 1/2 の `BindingId(0)`、terminal `BaseObjectMode` RHS
      builtin-object identity 1 個、2 inferred term、1 fact/deferred-free pre-
      desugaring checked inequality、ordered operand-owned constraint 2 個を
      object/set coercion なしで保持する。full exact/near-miss/corruption/real-
      sidecar guard を必須とし、inequality desugaring、truth/fact、acceptance、
      closure、theorem/proof/CoreIr/ControlFlowIr/VC、general unbounded
      semantics を捏造しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、169。参照: Step 5、mizar-test task 10、specs 3、4、7、13、
      14.5.2、16。

178. **Exact local-object-mode long-chain membership checker bridge を追加する。** [x]
    - task 74 の AST-bounded object-terminal definition chain 7 本、ordered
      reserve `x` for `ChainObjectMode6` と `y` for explicit `set`、test-first
      fixture の exact `x in y` theorem label だけを task 165 の real object-
      left/set-right membership consumer へ渡す。
    - Acceptance: real expansion 7 本、raw `ChainObjectMode6` left result と
      独立した explicit-set right result/sole right expected input、ordinal 2/3
      の `BindingId(0/1)`、distinct terminal `BaseObjectMode` RHS builtin-object
      identity と explicit-set identity、left expected input なし、2 inferred
      term、1 fact/deferred-free checked membership、right-owned constraint 1 個
      を object/set coercion なしで保持する。full exact/near-miss/corruption/
      real-sidecar guard を必須とし、truth/fact、acceptance、closure、theorem/
      proof/CoreIr/ControlFlowIr/VC、general unbounded semantics を捏造しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、165。参照: Step 5、mizar-test task 10、specs 3、4、7、13、
      14.5.3、16。

179. **Exact local-object-mode long-chain type assertion checker bridge を追加する。** [x]
    - task 74 の AST-bounded object-terminal definition chain 7 本、
      `ChainObjectMode6` reserve 1 個、test-first fixture の exact `x is object`
      theorem label だけを task 153 の real object-normalizing type-assertion
      consumer へ渡す。
    - Acceptance: real expansion 7 本、raw `ChainObjectMode6` subject result と
      独立した formula-side builtin-object asserted input、ordinal 1 の
      `BindingId(0)`、terminal `BaseObjectMode` RHS builtin-object identity 1 個、
      1 inferred term、general reachability と object/set coercion を用いない
      1 fact/deferred-free normalized-reflexive checked type assertion を保持する。
      full exact/near-miss/corruption/real-sidecar guard を必須とし、widening/
      `qua`、truth/fact、acceptance、closure、theorem/proof/CoreIr/ControlFlowIr/
      VC、general unbounded semantics を捏造しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、153、175。参照: Step 5、mizar-test task 10、specs 3、4、7、
      13、14.2.3、16。

180. **Exact source-derived contradiction formula-constant checker bridge を追加する。** [x]
    - `theorem SourceDerivedContradictionConstantBoundary: contradiction;`
      だけを新規 standalone exact leaf extractor から既存
      `FormulaKind::Contradiction` consumer へ渡す。
    - Acceptance: real leaf site/range と module-root context を保持し、term、
      asserted type、expected constraint、candidate、fact、deferred reason、
      diagnostic を持たない 1 checked formula を記録する。exact/near-miss/
      corruption/real-sidecar guard を必須とし、task 112/117 composite と thesis
      behavior を変更しない。truth/fact publication、theorem acceptance、proof-
      goal closure、implicit closure/child graph、`formula_statement`、proof、
      CoreIr、ControlFlowIr、VC を主張しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 112、115、117。参照: Step 5、mizar-test task 10、specs 14、16。

181. **Exact imported attributed-reserve routing を repair する。** [x]
    - task 180 後に見つかった `source_undocumented_behavior` を repair する。
      generic reserve extractor は現在、documented task-84/85/116/171 source
      shape 5 件を超える imported fixture attribute を受理し得る。
    - Acceptance: reserve binding が `parser.type_fixtures` imported attribute
      を持つ場合、unrelated top-level item なし、single-binding exact source
      （positive `TypeCaseAttr set`、positive `empty set`、negative `empty set`、
      negative `empty object`）のいずれか、または既に trace 済みの ordered
      mixed source `reserve x for set; reserve y for non empty set;` を要求する。
      各 attributed binding は argument-free attribute 1 個を持つ。既存 `.miz`
      expectation 5 件は変更しない。duplicate/mixed attribute、wrong polarity/
      head、exact mixed source 外の multiple binding/item、extra definition は
      source extraction gap に残す。repair 前に source-
      shaped unit regression を追加し、positive `empty object` または evidence/
      acceptance semantics を昇格しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 84、85、116、171。参照: Step 5、mizar-test task 10、specs
      3、6、11、12、17。

182. **Exact formula-side local-mode asserted-head checker bridge を追加する。** [x]
    - 既存 type-assertion matrix が cover しない最小 spec-derived source role として、
      `mode LocalModeAssertedHeadDef: LocalModeAssertedHead is set;` を含む
      exact definition block、その mode の reserve 1 個、builtin head ではなく同じ
      local mode を assert する exact theorem を追加する。
    - Acceptance: `LocalModeAssertedHeadPayloadBoundary: x is
      LocalModeAssertedHead;` について、distinct source site/range と同じ resolved
      local-mode symbol を持つ raw reserve-subject type と独立 formula-side asserted
      `TypeExpressionInput` を保持する。real AST-derived expansion 1 個を消費し、
      ordinal 1 を `BindingId(0)` に解決し、known type entry 3 個を terminal-
      definition-RHS builtin-set identity 1 個へ intern し、general reachability なしで
      1 inferred variable と 1 fact/deferred-free normalized-reflexive checked type
      assertion を記録する。exact/near-miss/corruption/real-sidecar guard を必須とする。
      builtin、other-mode、attributed、argument-bearing、object-terminal、chained、
      recovered、broader source shape は deferred のままにし、declaration acceptance/
      inhabitation、widening/`qua`、truth/fact、theorem acceptance、proof/CoreIr/
      ControlFlowIr/VC、general semantics を主張しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 55、122、138。参照: Step 5、mizar-test task 10、specs 3、4、
      7、13、14、16。

183. **Object-terminal formula-side local-mode asserted-head checker bridge を追加する。** [x]
    - task 182 の direct object-terminal sibling を閉じる。exact definition block
      に `mode LocalObjectModeAssertedHeadDef: LocalObjectModeAssertedHead is
      object;`、その mode の reserve 1 個、`LocalObjectModeAssertedHeadPayloadBoundary:
      x is LocalObjectModeAssertedHead;` だけを含める。
    - Acceptance: distinct site/range と同じ resolved mode symbol を持つ独立した
      raw reserve-subject/formula-side asserted input を保持する。real AST-derived
      object-terminal expansion 1 個を消費し、ordinal 1 を `BindingId(0)` に解決し、
      known type entry 3 個を definition-RHS-anchored builtin-object identity 1 個へ
      intern し、general reachability と object/set coercion を用いず、1 inferred
      variable と 1 fact/deferred-free normalized-reflexive checked type assertion
      を記録する。exact/near-miss/corruption/real-sidecar guard を必須とする。
      builtin、other-mode、attributed、argument-bearing、chained、recovered、
      broader source shape は deferred のままにする。exact direct set-terminal
      sibling は task 182 の credit を維持し、task 183 は新しい set-terminal
      credit を追加しない。declaration acceptance/inhabitation、truth/fact、
      theorem acceptance、proof/CoreIr/
      ControlFlowIr/VC、general semantics を主張しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 55、145、182。参照: Step 5、mizar-test task 10、specs 3、4、
      7、13、14、16。

184. **One-edge formula-side local-mode asserted-head checker bridge を追加する。** [x]
    - `mode BaseModeAssertedHeadDef: BaseModeAssertedHead is set;` と `mode
      ChainModeAssertedHeadDef: ChainModeAssertedHead is BaseModeAssertedHead;`
      を含む ordered definition block 2 個、`ChainModeAssertedHead` の reserve 1 個、
      `ChainedLocalModeAssertedHeadPayloadBoundary: x is
      ChainModeAssertedHead;` から成る set-terminal same-outer-mode one-edge
      chain だけを閉じる。
    - Acceptance: distinct site/range と同じ resolved outer mode symbol を持つ
      独立した raw reserve-subject/formula-side asserted input を保持する。real
      AST-derived expansion 2 個を消費し、ordinal 1 を `BindingId(0)` に解決し、
      known type entry 3 個を terminal base-definition-RHS builtin-set identity
      1 個へ intern し、general reachability、widening、`qua` を用いず、1 inferred
      variable と 1 fact/deferred-free normalized-reflexive checked type assertion
      を記録する。exact/near-miss/corruption/real-sidecar guard を必須とする。
      direct、object-terminal、deeper、attributed、argument-bearing、imported、
      recovered、他 asserted-head shape は task 外とし、declaration acceptance/
      inhabitation、truth/fact、theorem acceptance、closure/order、proof/CoreIr/
      ControlFlowIr/VC、general chain semantics を主張しない。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 56、146、182。参照: Step 5、mizar-test task 10、specs 3、4、
      7、13、14、16。

185. **One-edge object-terminal formula-side local-mode asserted-head checker bridge を追加する。** [x]
    - `mode BaseObjectModeAssertedHeadDef: BaseObjectModeAssertedHead is object;`
      と `mode ChainObjectModeAssertedHeadDef: ChainObjectModeAssertedHead is
      BaseObjectModeAssertedHead;` を含む ordered definition block 2 個、
      `ChainObjectModeAssertedHead` の reserve 1 個、
      `ChainedLocalObjectModeAssertedHeadPayloadBoundary: x is
      ChainObjectModeAssertedHead;` から成る object-terminal same-outer-mode
      one-edge chain だけを閉じる。
    - Acceptance: distinct site/range と同じ resolved outer mode symbol を持つ
      独立した raw reserve-subject/formula-side asserted input を保持する。real
      AST-derived expansion 2 個を消費し、ordinal 1 を `BindingId(0)` に解決し、
      known type entry 3 個を terminal base-definition-RHS builtin-object identity
      1 個へ intern し、general reachability、widening、`qua`、object/set coercion
      を用いず、1 inferred variable と 1 fact/deferred-free normalized-reflexive
      checked type assertion を記録する。exact/near-miss/corruption/real-sidecar
      guard と shared trace backlink 5 個 + dedicated row 1 個を必須とし、active
      runner を 132 から 133 に増やす。set-terminal、direct、deeper、attributed、
      argument-bearing、imported、recovered、他 asserted-head shape は task 外とし、
      declaration/attribute acceptance、broader term/formula/child-graph semantics、
      truth/fact、theorem acceptance、closure/order、proof/CoreIr/ControlFlowIr/VC、
      general chain semantics を主張しない。module layout 更新は不要である。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 56、147、183、184。参照: Step 5、mizar-test task 10、specs 3、
      4、7、13、14、16。

186. **Two-edge formula-side local-mode asserted-head checker bridge を追加する。** [x]
    - `BaseTwoEdgeModeAssertedHead` is `set`、`MiddleTwoEdgeModeAssertedHead` is
      `BaseTwoEdgeModeAssertedHead`、`OuterTwoEdgeModeAssertedHead` is
      `MiddleTwoEdgeModeAssertedHead` の ordered definition 3 個だけを閉じる。
      outer mode に `x` を reserve し、`TwoEdgeLocalModeAssertedHeadPayloadBoundary:
      x is OuterTwoEdgeModeAssertedHead;` だけを check する。
    - Acceptance: 同じ outer symbol 向けの distinct reserve/asserted site/range、
      real expansion 3 個、ordinal 1 の `BindingId(0)`、terminal base RHS builtin-
      set identity 1 個へ normalize する known entry 3 個、reachability、widening、
      `qua` を用いない 1 `Inferred` term と 1 fact/deferred-free normalized-
      reflexive `Checked` assertion を保持する。imported/ambiguous provenance を
      含む exact/near-miss/corruption/real-sidecar guard、shared 5 + dedicated 1
      trace row を必須とし active runner を 133 から 134 に増やす。object/deeper/
      imported semantics、declaration/attribute、broader term/formula/child graph、
      truth/fact、proof/Core/ControlFlow/VC、general chain semantics は deferred の
      ままとする。module layout 更新は不要である。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 72、148、184、185。参照: Step 5、mizar-test task 10、specs 3、
      4、7、13、14、16。

187. **Two-edge object-terminal formula-side local-mode asserted-head checker bridge を追加する。** [x]
    - `mode BaseTwoEdgeObjectModeAssertedHeadDef:
      BaseTwoEdgeObjectModeAssertedHead is object;`、`mode
      MiddleTwoEdgeObjectModeAssertedHeadDef: MiddleTwoEdgeObjectModeAssertedHead
      is BaseTwoEdgeObjectModeAssertedHead;`、`mode
      OuterTwoEdgeObjectModeAssertedHeadDef: OuterTwoEdgeObjectModeAssertedHead is
      MiddleTwoEdgeObjectModeAssertedHead;` の ordered definition 3 個だけを
      閉じる。outer mode に `x` を reserve し、
      `TwoEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
      OuterTwoEdgeObjectModeAssertedHead;` だけを check する。
    - Acceptance: 同じ local outer symbol 向けの distinct raw subject/asserted
      site/range、real expansion 3 個、ordinal 1 の `BindingId(0)`、terminal base-
      definition-RHS builtin-object identity 1 個へ normalize する known entry
      3 個、expected constraint、reachability、widening、`qua`、object/set coercion
      を持たない 1 `Inferred` term と 1 fact/deferred-free normalized-reflexive
      `Checked` assertion を保持する。wrong label、attributed/argument-bearing
      formula-side asserted head、imported Base/Middle/Outer、imported/ambiguous
      asserted provenance、collapsed provenance、`BuiltinSet` output
      corruption を含む exact/near-miss/corruption/real-sidecar guard を備える。
      shared 5 + dedicated 1 trace row が active runner 135 を保護する。positive
      imported semantics、declaration/attribute、broader term/
      formula/child graph、truth/fact、implicit closure/order、theorem acceptance、
      proof/Core/ControlFlow/VC、general chain semantics は deferred のままとする。
      Step 5 は active、Steps 6/7 は
      deferred のまま。module layout 更新は不要である。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 72、149、185、186。参照: Step 5、mizar-test task 10、specs 3、
      4、7、13、14、16。

188. **Exact builtin-object reserved-variable equality checker bridge を追加する。** [x]
    - real builtin-object reserve handoff と reserved-variable equality checker
      consumer を再利用し、`reserve x for object; theorem
      ReservedObjectVariableEqualityPayloadBoundary: x = x;` だけを閉じる。
    - Acceptance: source-order use を ordinal 1/2 で `BindingId(0)` に解決し、
      written `object` range 1 個上の distinct operand result/expected role site
      4 個、canonical builtin-object identity 1 個、`Inferred` variable 2 個、
      ordered expected constraint 2 個、fact/deferred-free `Checked` equality
      1 個を保持する。exact/near-miss、matched-output、canonical-source、
      `BuiltinSet` corruption、route-order、real frontend/resolver-sidecar guard
      を必須とする。shared trace backlink 5 個 + dedicated 1 個を持つ spec-derived
      active pass fixture 1 件を追加し、既存 expectation を変更せず active runner
      135 を 136 に増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`。`spec_gap` はない。
      object/set coercion、general/non-reflexive object equality、truth/fact、
      closure/order、theorem acceptance、proof/Core/ControlFlow/VC、broader source
      shape は deferred のままとする。Step 5 は active、Steps 6/7 は deferred
      のまま。module layout 更新は不要である。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 48、119、125、128。参照: Step 5、mizar-test task 10、specs 3、4、
      13、14、16。

189. **Exact builtin-object reserved-variable type-assertion checker bridge を追加する。** [x]
    - real builtin-object reserve handoff と normalized-reflexive type-assertion
      checker consumer を再利用し、`reserve x for object; theorem
      ReservedObjectVariableTypeAssertionPayloadBoundary: x is object;` だけを
      閉じる。
    - Acceptance: source-order subject を ordinal 1 で `BindingId(0)` に解決し、
      distinct reserve-subject result/formula-side asserted site/range、written
      reserve type を anchor とする canonical builtin-object identity 1 個、
      `Inferred` variable 1 個、known type entry 3 個、expected constraint 0 個、
      fact/deferred-free `Checked` assertion 1 個を保持する。exact/near-miss、
      matched-output、canonical-source、`BuiltinSet` corruption、route-order、
      real frontend/resolver-sidecar guard を必須とする。shared trace backlink
      5 個 + dedicated 1 個を持つ spec-derived active pass fixture 1 件を追加し、
      既存 expectation を変更せず active runner 136 を 137 に増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`。`spec_gap` はない。
      reachability/widening/`qua`、object/set coercion、truth/fact、closure/order、
      theorem acceptance、proof/Core/ControlFlow/VC、broader source shape は
      deferred のままとする。Step 5 は active、Steps 6/7 は deferred のまま。
      module layout 更新は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 48、122、125、145、188。参照: Step 5、mizar-test task 10、
      specs 3、4、13、14、16。

190. **Exact builtin-object reserved-variable inequality checker bridge を追加する。** [x]
    - real builtin-object reserve handoff と pre-desugaring inequality checker
      consumer を再利用し、`reserve x for object; theorem
      ReservedObjectVariableInequalityPayloadBoundary: x <> x;` だけを閉じる。
    - Acceptance: source-order ordinal 1/2 の use を `BindingId(0)` へ解決し、
      written `object` range 1 個上の distinct operand result/expected role site
      4 個を保持し、canonical builtin-object identity 1 個へ intern する。
      `Inferred` variable 2 個、known type entry 6 個、ordered expected
      constraint 2 個、fact/candidate/diagnostic/deferred-free `Checked`
      inequality 1 個を要求する。exact/near-miss、matched-output、canonical-
      source、`BuiltinSet` corruption、route-order、real frontend/resolver-
      sidecar guard を必須とする。spec-derived active pass fixture 1 件と
      shared 5 + dedicated 1 trace backlink を追加し、既存 expectation を変更せず
      active runner を 137 から 138 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`。`spec_gap` はない。
      inequality desugaring/equality truth、object/set coercion、fact、closure/
      order、theorem acceptance、proof/Core/ControlFlow/VC、broader source shape
      は deferred のままにする。Step 5 は active、Steps 6/7 は deferred のまま。
      checker source または module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 48、121、125、128、188。参照: Step 5、mizar-test task 10、
      specs 3、4、13、14、16。

191. **Exact distinct-binding shared-builtin-object equality checker bridge を追加する。** [x]
    - real one-item/two-binding shared-range reserve producer と real builtin-
      object equality consumer を合成し、`reserve x, y for object; theorem
      DistinctReservedObjectVariableEqualityPayloadBoundary: x = y;` だけを
      閉じる。
    - Acceptance: source-order ordinal 2/3 の use を `BindingId(0/1)` へ
      解決し、両 binding と distinct operand result/expected role site 4 個に
      shared written `object` range 1 個を保持する。その reserve range を
      anchor とする canonical builtin-object identity 1 個、`Inferred`
      variable 2 個、known type entry 6 個、operand-owned ordered expected
      constraint 2 個、fact/candidate/diagnostic/deferred-free `Checked`
      equality 1 個を要求する。exact/near-miss、matched-output、canonical-
      source、`BuiltinSet` corruption、route-order、real frontend/resolver-
      sidecar guard を必須とする。test-first active pass fixture 1 件と
      shared 5 + dedicated 1 trace backlink を追加し、既存 expectation を
      変更せず active runner を 138 から 139 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`。`spec_gap` は
      ない。equality truth、object/set coercion、fact、implicit closure/order、
      declaration/theorem acceptance、proof/Core/ControlFlow/VC、broader
      distinct-object source shape は deferred のままとする。Step 5 は
      active、Steps 6/7 は deferred のまま。checker source または module-
      layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 48、123、125、188。参照: Step 5、mizar-test task 10、
      specs 3、4、13、14、16。

192. **Exact distinct-binding shared-builtin-object inequality checker bridge を追加する。** [x]
    - real one-item/two-binding shared-range builtin-object producer と real
      pre-desugaring inequality consumer を合成し、`reserve x, y for object;
      theorem DistinctReservedObjectVariableInequalityPayloadBoundary: x <>
      y;` だけを閉じる。
    - 受入条件: source-order use の ordinal 2/3 を `BindingId(0/1)` に解決し、
      両 binding と distinct operand result/expected role site 4 個に shared
      written `object` range 1 個を保持し、その reserve range を anchor とする
      canonical builtin-object identity 1 個、`Inferred` variable 2 個、known
      type entry 6 個、operand-owned ordered expected constraint 2 個、fact/
      candidate/diagnostic/deferred-free `Checked` inequality 1 個を要求する。
      exact/near-miss、matched-output、canonical-source、`BuiltinSet`
      corruption、route-order、real frontend/resolver-sidecar guard を必須とする。
      test-first active pass fixture 1 件と shared 5 + dedicated 1 trace backlink
      を追加し、既存 expectation を変更せず active runner を 139 から 140 へ
      増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`。`spec_gap` は
      ない。inequality desugaring/equality truth、object/set coercion、fact、
      implicit closure/order、declaration/theorem acceptance、proof/Core/
      ControlFlow/VC、broader distinct-object source shape は deferred のままと
      する。Step 5 は active、Steps 6/7 は deferred のまま。checker source と
      module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 48、121、123、160、190、191。参照: Step 5、mizar-test
      task 10、specs 3、4、13、14、16。

193. **Exact multiple-reserve-declaration builtin-object equality checker bridge を追加する。** [x]
    - Task 124 の real two-item/two-binding/distinct-written-range reserve
      producer と real builtin-object equality consumer を合成し、`reserve x
      for object; reserve y for object; theorem
      MultipleObjectReserveDeclarationEqualityPayloadBoundary: x = y;` だけを
      閉じる。
    - 受入条件: source-order use の ordinal 2/3 を `BindingId(0/1)` に解決し、
      distinct written `object` range 2 個を distinct operand result/expected
      role site 4 個に保持する。先行する `x` reserve range を anchor とする
      canonical builtin-object identity 1 個、`Inferred` variable 2 個、known
      type entry 6 個、operand-owned ordered expected constraint 2 個、fact/
      candidate/diagnostic/deferred-free `Checked` equality 1 個を要求する。
      exact structural/provenance near miss、matched-output と canonical-source
      corruption probe、`BuiltinSet` corruption、route-order isolation、real
      frontend/resolver sidecar を必須とする。test-first active pass fixture
      1 件と shared 5 + dedicated 1 trace backlink を追加し、既存 expectation
      を変更せず active runner を 140 から 141 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`。`spec_gap` は
      ない。equality truth、object/set coercion、fact、implicit closure/order、
      declaration/theorem acceptance、proof/Core/ControlFlow/VC、shared-range
      と broader multiple-reserve source shape は deferred のままとする。
      Step 5 は active、Steps 6/7 は deferred のまま。checker source と
      module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 48、119、124、188、191。参照: Step 5、mizar-test task 10、
      specs 3、4、13、14、16。

194. **Exact multiple-reserve-declaration builtin-object inequality checker bridge を追加する。** [x]
    - Task 193 の ordered two-item/two-binding/distinct-object-range producer と
      real pre-desugaring builtin-object inequality consumer を合成し、`reserve
      x for object; reserve y for object; theorem
      MultipleObjectReserveDeclarationInequalityPayloadBoundary: x <> y;` だけを
      閉じる。
    - 受入条件: ordinal 2/3 を `BindingId(0/1)` に解決し、source-ordered
      distinct written `object` range 2 個を distinct raw result/expected role
      site 4 個に保持する。先行する `x` range を anchor とする builtin-object
      identity 1 個、`Inferred` variable 2 個、known type entry 6 個、operand-
      owned ordered constraint 2 個、fact/candidate/diagnostic/deferred-free
      pre-desugaring `Checked` inequality 1 個を要求する。exact structural/
      provenance near miss、binding/ordinal/range/role/head/raw-source/
      canonical-source/expected-input/module corruption、route isolation、
      positive immutable-output check、real frontend/resolver sidecar を必須と
      する。test-first active pass fixture 1 件と shared 5 + dedicated 1 backlink
      を追加し、既存 expectation を変更せず active runner を 141 から 142、
      cases を 356 から 357、requirements を 320 から 321 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`。`spec_gap` は
      ない。inequality desugaring/equality truth、object/set coercion、fact、
      closure/order、declaration/theorem acceptance、proof/Core/ControlFlow/VC、
      shared-range shape、broader reserve shape は deferred のままとする。
      Step 5 は active、Steps 6/7 は deferred のまま。checker source と module-
      layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 48、121、124、161、190、192、193。参照: Step 5、mizar-test
      task 10、specs 3、4、13、14、16。

195. **Exact three-edge formula-side local-mode asserted-head checker bridge を追加する。** [x]
    - ordered local mode definition 4 個 `BaseThreeEdgeModeAssertedHead -> set`、
      `InnerThreeEdgeModeAssertedHead -> BaseThreeEdgeModeAssertedHead`、
      `MiddleThreeEdgeModeAssertedHead -> InnerThreeEdgeModeAssertedHead`、
      `OuterThreeEdgeModeAssertedHead -> MiddleThreeEdgeModeAssertedHead`、
      `reserve x for OuterThreeEdgeModeAssertedHead` 1 個、theorem
      `ThreeEdgeLocalModeAssertedHeadPayloadBoundary: x is
      OuterThreeEdgeModeAssertedHead;` だけを、Task 73 の four-expansion producer
      と Task 186 の same-outer formula-side asserted-head consumer の合成で閉じる。
    - 受入条件: 同じ resolved outer mode symbol の distinct raw reserve-subject
      と formula-side asserted site/range を保持し、real AST-derived expansion
      4 個だけを消費し、ordinal 1 を `BindingId(0)` に解決する。known type entry
      3 個を base definition RHS を anchor とする canonical builtin-set identity
      1 個へ normalize し、expected constraint と general reachability なしで
      `Inferred` variable 1 個と fact/candidate/diagnostic/deferred-free normalized-
      reflexive `Checked` `TypeAssertion` 1 個を記録する。exact structural/
      provenance near miss、独立した expansion/binding/ordinal/head/spelling/
      range/canonical-source corruption、route isolation、positive immutable-
      output check、real frontend/resolver sidecar を必須とする。test-first active
      pass fixture 1 件と shared 5 + dedicated 1 backlink を追加し、既存
      expectation を変更せず active runner を 142 から 143、cases を 357 から
      358、requirements を 321 から 322 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`。`spec_gap` はない。
      object-terminal/deeper/imported/attributed/argument-bearing/other asserted-
      head shape、reachability/widening/`qua`、mode declaration acceptance/
      inhabitation、assertion truth/fact、implicit closure/order、theorem
      acceptance、broader term/formula/child-graph semantics、proof/Core/
      ControlFlow/VC、general chain semantics は deferred のままとする。Step 5
      は active、Steps 6/7 は deferred のまま。checker source と module-layout
      change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 73、150、182、186。参照: Step 5、mizar-test task 10、specs
      3、4、7、13、14、16。

196. **Exact three-edge object-terminal formula-side local-mode asserted-head checker bridge を追加する。** [x]
    - ordered local mode definition 4 個
      `BaseThreeEdgeObjectModeAssertedHead -> object`、
      `InnerThreeEdgeObjectModeAssertedHead -> BaseThreeEdgeObjectModeAssertedHead`、
      `MiddleThreeEdgeObjectModeAssertedHead -> InnerThreeEdgeObjectModeAssertedHead`、
      `OuterThreeEdgeObjectModeAssertedHead -> MiddleThreeEdgeObjectModeAssertedHead`、
      `reserve x for OuterThreeEdgeObjectModeAssertedHead` 1 個、theorem
      `ThreeEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
      OuterThreeEdgeObjectModeAssertedHead;` だけを、Task 73/151 の real four-
      expansion object-terminal producer と Task 187 の same-outer formula-
      side asserted-head consumer の合成で閉じる。
    - 受入条件: 同じ resolved outer mode symbol の distinct raw reserve-subject
      と formula-side asserted site/range を保持し、real AST-derived expansion
      4 個だけを消費し、ordinal 1 を `BindingId(0)` に解決する。known type
      entry 3 個を base definition RHS を anchor とする canonical builtin-object
      identity 1 個へ normalize し、expected constraint、general reachability、
      object/set coercion なしで `Inferred` variable 1 個と fact/candidate/
      diagnostic/deferred-free normalized-reflexive `Checked` `TypeAssertion`
      1 個を記録する。exact structural/provenance near miss、独立した expansion/
      binding/ordinal/head/spelling/range/canonical-source corruption、route
      isolation、positive immutable-output check、real frontend/resolver sidecar
      を必須とする。test-first active pass fixture 1 件と shared 5 + dedicated 1
      backlink を追加し、既存 expectation を変更せず active runner を 143 から
      144、cases を 358 から 359、requirements を 322 から 323 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`。`spec_gap` はない。
      deeper/imported/attributed/argument-bearing/other asserted-head shape、
      reachability/widening/`qua`、mode declaration acceptance/inhabitation、
      assertion truth/fact、implicit closure/order、theorem acceptance、broader
      term/formula/child-graph semantics、proof/Core/ControlFlow/VC、general
      chain semantics は deferred のままとする。Step 5 は active、Steps 6/7
      は deferred のまま。checker source と module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 73、151、187、195。参照: Step 5、mizar-test task 10、
      specs 3、4、7、13、14、16。

197. **Exact four-edge formula-side local-mode asserted-head checker bridge を追加する。** [x]
    - ordered local mode definition 5 個
      `BaseFourEdgeModeAssertedHead -> set`、
      `InnerFourEdgeModeAssertedHead -> BaseFourEdgeModeAssertedHead`、
      `MiddleFourEdgeModeAssertedHead -> InnerFourEdgeModeAssertedHead`、
      `OuterFourEdgeModeAssertedHead -> MiddleFourEdgeModeAssertedHead`、
      `TooDeepFourEdgeModeAssertedHead -> OuterFourEdgeModeAssertedHead`、
      `reserve x for TooDeepFourEdgeModeAssertedHead` 1 個、theorem
      `FourEdgeLocalModeAssertedHeadPayloadBoundary: x is
      TooDeepFourEdgeModeAssertedHead;` だけを、Task 74/152 の real five-
      expansion set-terminal producer と Task 186/195 の same-outer formula-
      side asserted-head consumer の合成で閉じる。
    - 受入条件: 同じ resolved outermost mode symbol の distinct raw reserve-
      subject と formula-side asserted site/range を保持し、real AST-derived
      expansion 5 個だけを消費し、ordinal 1 を `BindingId(0)` に解決する。
      known type entry 3 個を base definition RHS を anchor とする canonical
      builtin-set identity 1 個へ normalize し、expected constraint と general
      reachability なしで `Inferred` variable 1 個と fact/candidate/diagnostic/
      deferred-free normalized-reflexive `Checked` `TypeAssertion` 1 個を記録
      する。unrelated local/imported/ambiguous asserted head を含む exact
      structural/provenance near miss、独立した expansion/binding/ordinal/head/
      spelling/range/canonical-source corruption、route isolation、positive
      immutable-output check、real frontend/resolver sidecar を必須とする。
      test-first active pass fixture 1 件と shared 5 + dedicated 1 backlink を
      追加し、既存 expectation を変更せず active runner を 144 から 145、
      cases を 359 から 360、requirements を 323 から 324 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`。`spec_gap` はない。
      object-terminal/other-depth/imported/attributed/argument-bearing/other
      asserted-head shape、reachability/widening/`qua`、mode declaration
      acceptance/inhabitation、assertion truth/fact、implicit closure/order、
      theorem acceptance、broader term/formula/child-graph semantics、proof/
      Core/ControlFlow/VC、general chain semantics は deferred のままとする。
      Step 5 は active、Steps 6/7 は deferred のまま。checker source または
      module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、152、186、195。参照: Step 5、mizar-test task 10、
      specs 3、4、7、13、14、16。

198. **Exact four-edge object-terminal formula-side local-mode asserted-head checker bridge を追加する。** [x]
    - ordered local mode definition 5 個
      `BaseFourEdgeObjectModeAssertedHead -> object`、
      `InnerFourEdgeObjectModeAssertedHead -> BaseFourEdgeObjectModeAssertedHead`、
      `MiddleFourEdgeObjectModeAssertedHead -> InnerFourEdgeObjectModeAssertedHead`、
      `OuterFourEdgeObjectModeAssertedHead -> MiddleFourEdgeObjectModeAssertedHead`、
      `TooDeepFourEdgeObjectModeAssertedHead ->
      OuterFourEdgeObjectModeAssertedHead`、`reserve x for
      TooDeepFourEdgeObjectModeAssertedHead` 1 個、theorem
      `FourEdgeLocalObjectModeAssertedHeadPayloadBoundary: x is
      TooDeepFourEdgeObjectModeAssertedHead;` だけを、Task 74/153 の real
      five-expansion object-terminal producer と Task 187/196 の same-
      outermost formula-side asserted-head consumer の合成として閉じる。
    - 受入条件: 同じ resolved outermost mode symbol に対する distinct raw
      reserve-subject/formula-side asserted site/range を保持し、real AST-derived
      expansion 5 個だけを消費し、ordinal 1 を `BindingId(0)` に解決し、known
      type entry 3 個を base definition RHS に anchor した canonical builtin-
      object identity 1 個へ normalize し、expected constraint、general
      reachability、object/set coercion なしで `Inferred` variable 1 個と fact/
      candidate/diagnostic/deferred-free normalized-reflexive `Checked`
      `TypeAssertion` 1 個を記録する。full reorder、connected deeper、unrelated
      local/imported/ambiguous asserted head を含む exact structural/provenance
      near miss、独立した expansion/binding/ordinal/head/spelling/range/
      canonical-source corruption、route isolation、positive immutable-output
      check、real frontend/resolver sidecar を要求する。test-first active pass
      fixture 1 件、shared backlink 5 件、dedicated backlink 1 件を追加し、既存
      expectation を変更せず active runner を 145 から 146、cases を 360 から
      361、requirements を 324 から 325 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap`
      なし。set-terminal/other-depth/imported/attributed/argument-bearing/other
      asserted-head shape、reachability/widening/`qua`、mode declaration
      acceptance/inhabitation、assertion truth/fact、implicit closure/order、
      theorem acceptance、broader term/formula/child-graph semantics、proof/
      Core/ControlFlow/VC、general chain semantics は deferred のままとする。
      Step 5 は active、Steps 6/7 は deferred のままとする。checker source
      または module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、153、187、196、197。参照: Step 5、mizar-test task 10、
      specs 3、4、7、13、14、16。

199. **Exact seven-expansion set-terminal formula-side local-mode asserted-head checker bridge を追加する。** [x]
    - 既存の ordered bare local mode definition 7 個 `ChainMode6 ->
      ChainMode5 -> ChainMode4 -> ChainMode3 -> ChainMode2 -> ChainMode1 ->
      BaseMode -> set`、`reserve x for ChainMode6` 1 個、theorem
      `LongLocalModeAssertedHeadPayloadBoundary: x is ChainMode6;` だけを、Task
      74/175 の real seven-expansion set-terminal producer と Task 186/195/197
      の same-symbol formula-side asserted-head consumer の合成として閉じる。
    - 受入条件: 同じ resolved `ChainMode6` symbol に対する distinct raw
      reserve-subject/formula-side asserted site/range を保持し、real AST-derived
      expansion 7 個だけを消費し、ordinal 1 を `BindingId(0)` に解決し、known
      type entry 3 個を `BaseModeDef` RHS anchor の canonical builtin-set
      identity 1 個へ normalize し、expected constraint と general reachability
      なしで `Inferred` variable 1 個と fact/candidate/diagnostic/deferred-free
      normalized-reflexive `Checked` `TypeAssertion` 1 個を記録する。full
      reverse、真に接続した eighth edge、exact structural/provenance near miss、
      独立した seven-expansion/binding/ordinal/head/spelling/site/range/
      canonical-source corruption、route isolation、immutable-output check、
      real frontend/resolver sidecar を要求する。test-first active pass fixture
      1 件、shared backlink 5 件、dedicated backlink 1 件を追加し、既存
      expectation を変更せず active runner を 146 から 147、cases を 361 から
      362、requirements を 325 から 326 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap`
      なし。object-terminal/other-depth/imported/attributed/argument-bearing/
      other asserted-head shape、reachability/widening/`qua`、mode declaration
      acceptance/inhabitation、assertion truth/fact、implicit closure/order、
      theorem acceptance、broader term/formula/child-graph semantics、proof/
      Core/ControlFlow/VC、general unbounded chain semantics は deferred のまま
      とする。Step 5 は active、Steps 6/7 は deferred のままとする。checker
      source または module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、175、186、195、197。参照: Step 5、mizar-test task 10、
      specs 3、4、7、13、14、16。

200. **Exact seven-expansion object-terminal formula-side local-mode asserted-head checker bridge を追加する。** [x]
    - 既存の ordered bare local object-mode definition 7 個 `ChainObjectMode6 ->
      ChainObjectMode5 -> ChainObjectMode4 -> ChainObjectMode3 ->
      ChainObjectMode2 -> ChainObjectMode1 -> BaseObjectMode -> object`、
      `reserve x for ChainObjectMode6` 1 個、theorem
      `LongLocalObjectModeAssertedHeadPayloadBoundary: x is ChainObjectMode6;`
      だけを、Task 74/179 の real seven-expansion object-terminal producer と
      Tasks 187/196/198 の same-symbol formula-side asserted-head consumer、Task
      199 の depth-matched set-terminal sibling の合成として閉じる。
    - 受入条件: 同じ resolved `ChainObjectMode6` symbol に対する distinct raw
      reserve-subject/formula-side asserted site/range を保持し、real AST-derived
      expansion 7 個だけを消費し、ordinal 1 を `BindingId(0)` に解決し、known
      type entry 3 個を `BaseObjectModeDef` RHS anchor の canonical builtin-object
      identity 1 個へ normalize し、expected constraint、general reachability、
      object/set coercion なしで `Inferred` variable 1 個と fact/candidate/
      diagnostic/deferred-free normalized-reflexive `Checked` `TypeAssertion` 1
      個を記録する。full reverse、真に接続した eighth edge、exact structural/
      provenance near miss、独立した seven-expansion/binding/ordinal/head/
      spelling/site/range/canonical-source corruption、route isolation、
      immutable-output check、real frontend/resolver sidecar を要求する。test-
      first active pass fixture 1 件、shared backlink 5 件、dedicated backlink 1
      件を追加し、既存 expectation を変更せず active runner を 147 から 148、
      cases を 362 から 363、requirements を 326 から 327 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap`
      なし。set-terminal/other-depth/imported/attributed/argument-bearing/other
      asserted-head shape、reachability/widening/`qua`、mode declaration
      acceptance/inhabitation、assertion truth/fact、implicit closure/order、
      theorem acceptance、broader term/formula/child-graph semantics、proof/
      Core/ControlFlow/VC、general unbounded chain semantics は deferred のまま
      とする。Step 5 は active、Steps 6/7 は deferred のままとする。checker
      source または module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、179、187、196、198、199。参照: Step 5、mizar-test task
      10、specs 3、4、7、13、14、16。

201. **Exact one-edge formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
    - `mode BaseModeRadixAssertedHeadDef: BaseModeRadixAssertedHead is set;`、
      `mode OuterModeRadixAssertedHeadDef: OuterModeRadixAssertedHead is
      BaseModeRadixAssertedHead;`、outer mode reserve 1 個、
      `ChainedLocalModeRadixAssertedHeadPayloadBoundary: x is
      BaseModeRadixAssertedHead;` だけを、Tasks 56/146 の real two-expansion
      normalized type handoff と Task 184 の formula-side local-mode asserted-
      head consumer の合成として閉じる。
    - 受入条件: 既存の builtin asserted-type route は不変、既存の same-mode
      asserted-head route は same-mode のまま保ち、この exact immediate-radix
      edge だけを受理する explicit asserted-head relation を追加する。
      distinct raw subject/asserted site/range/resolved symbol を保持し、real
      expansion 2 個だけを消費し、ordinal 1 を `BindingId(0)` に解決し、known
      entry 3 個を Base definition RHS anchor の builtin-set identity 1 個へ
      normalize し、expected constraint 0 個、`Inferred` variable 1 個、fact/
      candidate/diagnostic/deferred-free normalized-reflexive `Checked`
      `TypeAssertion` 1 個を記録する。exact structural/provenance near miss、
      独立した expansion/binding/ordinal/head/spelling/site/range/immediate-
      edge/canonical corruption、Task 146/184 route isolation、immutable-output
      check、real frontend/resolver sidecar を要求する。test-first active pass
      fixture 1 件、shared backlink 5 件、dedicated backlink 1 件を追加し、既存
      expectation を変更せず active runner を 148 から 149、cases を 363 から
      364、requirements を 327 から 328 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap`
      なし。object-terminal/deeper/unrelated/imported/attributed/argument-
      bearing asserted head、general reachability/widening/`qua`、mode
      declaration acceptance/inhabitation、assertion truth/fact、implicit
      closure/order、theorem acceptance、broader term/formula/child-graph
      semantics、proof/Core/ControlFlow/VC、general chain semantics は deferred
      のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。
      checker source または module-layout change はない見込みである。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 56、146、184。参照: Step 5、mizar-test task 10、specs 3、4、
      7、13、14、16。

202. **Exact one-edge object-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
    - `mode BaseObjectModeRadixAssertedHeadDef:
      BaseObjectModeRadixAssertedHead is object;`、`mode
      OuterObjectModeRadixAssertedHeadDef: OuterObjectModeRadixAssertedHead is
      BaseObjectModeRadixAssertedHead;`、outer mode reserve 1 個、
      `ChainedLocalObjectModeRadixAssertedHeadPayloadBoundary: x is
      BaseObjectModeRadixAssertedHead;` だけを、Tasks 56/147 の real two-
      expansion object-normalization handoff、Task 185 の object formula-side
      local asserted-head consumer、Task 201 の immediate-radix relation の
      合成として閉じる。
    - 受入条件: 既存の builtin、same-mode、set-terminal immediate-radix route
      をすべて保ち、この exact object-terminal config だけを受理する。
      distinct raw Outer/Base site/range/resolved symbol を保持し、real
      expansion 2 個だけを消費し、ordinal 1 を `BindingId(0)` に解決し、known
      entry 3 個を Base definition RHS anchor の builtin-object identity 1 個へ
      normalize し、expected constraint 0 個、`Inferred` variable 1 個、fact/
      candidate/diagnostic/deferred-free normalized-reflexive `Checked`
      `TypeAssertion` 1 個を object/set coercion なしで記録する。exact
      structural/provenance near miss、独立した expansion/binding/ordinal/head/
      spelling/site/range/immediate-edge/`BuiltinSet`/canonical corruption、
      Tasks 147/185/201 route isolation、immutable-output check、real frontend/
      resolver sidecar を要求する。test-first active pass fixture 1 件、shared
      backlink 5 件、dedicated backlink 1 件を追加し、既存 expectation を変更
      せず active runner を 149 から 150、cases を 364 から 365、requirements
      を 328 から 329 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap`
      なし。Task 201 以外の追加 set-terminal shape、deeper/unrelated/imported/
      attributed/argument-bearing asserted head、general reachability/widening/
      `qua`、mode declaration
      acceptance/inhabitation、assertion truth/fact、implicit closure/order、
      theorem acceptance、broader term/formula/child-graph semantics、proof/
      Core/ControlFlow/VC、general chain semantics は deferred のままとする。
      Step 5 は active、Steps 6/7 は deferred のままとする。checker source
      または module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 56、147、185、201。参照: Step 5、mizar-test task 10、specs
      3、4、7、13、14、16。

203. **Exact two-edge set-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
    - `mode BaseTwoEdgeModeRadixAssertedHeadDef:
      BaseTwoEdgeModeRadixAssertedHead is set;`、`mode
      MiddleTwoEdgeModeRadixAssertedHeadDef: MiddleTwoEdgeModeRadixAssertedHead
      is BaseTwoEdgeModeRadixAssertedHead;`、`mode
      OuterTwoEdgeModeRadixAssertedHeadDef: OuterTwoEdgeModeRadixAssertedHead is
      MiddleTwoEdgeModeRadixAssertedHead;`、`reserve x for
      OuterTwoEdgeModeRadixAssertedHead`、
      `TwoEdgeLocalModeRadixAssertedHeadPayloadBoundary: x is
      MiddleTwoEdgeModeRadixAssertedHead` だけを、
      Task 72 の real three-expansion set-terminal producer、Task 186 の
      depth-matched formula consumer、Task 201 の unchanged closed immediate-
      radix relation の合成として閉じる。
    - 受入条件: 既存の builtin、same-mode、one-edge immediate-radix、object-
      terminal、same-Outer route をすべて保ち、この exact Outer-to-Middle
      config だけを受理する。distinct raw Outer/Middle site/range/resolved
      symbol を保持し、asserted symbol が outer expansion の immediate radix
      head と一致することを要求し、real expansion 3 個だけを消費し、ordinal
      1 を `BindingId(0)` に解決し、known entry 3 個を Base-definition-RHS
      builtin-set identity 1 個へ normalize し、expected constraint 0 個、
      `Inferred` variable 1 個、fact/candidate/diagnostic/deferred-free
      normalized-reflexive `Checked` `TypeAssertion` 1 個を記録する。exact
      structural/provenance near miss、独立した expansion/binding/ordinal/
      head/spelling/site/range/immediate-edge/`BuiltinObject`/canonical
      corruption、Tasks 72/186/201/202 route isolation、immutable-output
      check、real frontend/resolver sidecar を要求する。test-first active pass
      fixture 1 件、shared backlink 5 件、dedicated backlink 1 件を追加し、
      既存 expectation を変更せず active runner を 150 から 151、cases を
      365 から 366、requirements を 329 から 330 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap`
      なし。2 link をまたぐ Base assertion、object-terminal sibling、other
      depth、imported/attributed/argument-bearing asserted head、general
      reachability/widening/`qua`、mode declaration acceptance/inhabitation、
      assertion truth/fact、implicit closure/order、theorem acceptance、broader
      term/formula/child-graph semantics、proof/Core/ControlFlow/VC、general
      chain semantics は deferred のままとする。Step 5 は active、Steps 6/7
      は deferred のままとする。checker source または module-layout change
      は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 72、186、201、202。参照: Step 5、mizar-test task 10、specs
      3、4、7、13、14、16。

204. **Exact two-edge object-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
    - `mode BaseTwoEdgeObjectModeRadixAssertedHeadDef: BaseTwoEdgeObjectModeRadixAssertedHead is object;`、`mode MiddleTwoEdgeObjectModeRadixAssertedHeadDef: MiddleTwoEdgeObjectModeRadixAssertedHead is BaseTwoEdgeObjectModeRadixAssertedHead;`、`mode OuterTwoEdgeObjectModeRadixAssertedHeadDef: OuterTwoEdgeObjectModeRadixAssertedHead is MiddleTwoEdgeObjectModeRadixAssertedHead;`、`reserve x for OuterTwoEdgeObjectModeRadixAssertedHead`、`TwoEdgeLocalObjectModeRadixAssertedHeadPayloadBoundary: x is MiddleTwoEdgeObjectModeRadixAssertedHead` だけを、Task 72 の real object-terminal three-expansion producer、Task 187 の formula consumer、Tasks 202/203 の変更しない closed immediate-radix relation を合成して閉じる。
    - distinct raw Outer subject/Middle asserted symbol/site/range を保持し、asserted symbol が outer expansion の real immediate radix であることを要求し、ordinal 1 を `BindingId(0)` に解決し、expansion 3 個だけを消費し、known entry 3 個を Base-definition-RHS builtin-object identity 1 個へ normalize し、expected constraint 0 個、`Inferred` variable 1 個、fact/candidate/diagnostic/deferred-free normalized-reflexive `Checked` `TypeAssertion` 1 個を object/set coercion なしで記録する。Task 72 は producer integration を所有する。Tasks 189/145/147/149/187/202 および set-terminal Tasks 148/186/203 との bidirectional route isolation、全 nonidentity definition order、exact structural/provenance near miss、独立した expansion/binding/ordinal/head/spelling/site/range/immediate-edge/`BuiltinSet`/canonical corruption、immutable output、real frontend/resolver sidecar を要求する。test-first active fixture 1 件、shared backlink 5 件、dedicated backlink 1 件を追加し、既存 expectation を変更せず active runner を 151 から 152、cases を 366 から 367、requirements を 330 から 331 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。2 link をまたぐ Base assertion、other depth、imported/attributed/argument-bearing asserted head、general reachability/widening/`qua`、mode declaration acceptance/inhabitation、assertion truth/fact、closure/order、theorem acceptance、broader term/formula/child-graph semantics、proof/Core/ControlFlow/VC、general chain semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。checker source または module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 72、187、202、203。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

205. **Exact three-edge set-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
    - ordered definition 4 個 `BaseThreeEdgeModeRadixAssertedHead -> set`、`InnerThreeEdgeModeRadixAssertedHead -> BaseThreeEdgeModeRadixAssertedHead`、`MiddleThreeEdgeModeRadixAssertedHead -> InnerThreeEdgeModeRadixAssertedHead`、`OuterThreeEdgeModeRadixAssertedHead -> MiddleThreeEdgeModeRadixAssertedHead`、`reserve x for OuterThreeEdgeModeRadixAssertedHead`、`ThreeEdgeLocalModeRadixAssertedHeadPayloadBoundary: x is MiddleThreeEdgeModeRadixAssertedHead` だけを、Task 73 の real four-expansion producer、Task 195 の formula consumer、Tasks 201/203/204 の変更しない closed immediate-radix relation を合成して閉じる。
    - distinct raw Outer subject/Middle asserted symbol/site/range を保持し、asserted symbol が outer expansion の real immediate radix であることを要求し、ordinal 1 を `BindingId(0)` に解決し、expansion 4 個だけを消費し、known entry 3 個を Base-definition-RHS builtin-set identity 1 個へ normalize し、expected constraint 0 個、`Inferred` variable 1 個、fact/candidate/diagnostic/deferred-free normalized-reflexive `Checked` `TypeAssertion` 1 個を記録する。Task 73 は producer integration を所有する。set Tasks 122/138/146/148/150/195/201/203 および object Tasks 189/145/147/149/151/196/202/204 との bidirectional route isolation、全 nonidentity definition order、exact structural/provenance near miss、独立した expansion/binding/ordinal/head/spelling/site/range/immediate-edge/`BuiltinObject`/canonical corruption、immutable output、real frontend/resolver sidecar を要求する。test-first active fixture 1 件、shared backlink 5 件、dedicated backlink 1 件を追加し、既存 expectation を変更せず active runner を 152 から 153、cases を 367 から 368、requirements を 331 から 332 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。multi-hop Inner/Base assertion、matching object sibling、other depth、imported/attributed/argument-bearing asserted head、general reachability/widening/`qua`、mode declaration acceptance/inhabitation、assertion truth/fact、closure/order、theorem acceptance、broader term/formula/child-graph semantics、proof/Core/ControlFlow/VC、general chain semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。checker source または module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 73、195、201、203、204。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

206. **Exact three-edge object-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
    - ordered definition 4 個 `BaseThreeEdgeObjectModeRadixAssertedHead -> object`、`InnerThreeEdgeObjectModeRadixAssertedHead -> BaseThreeEdgeObjectModeRadixAssertedHead`、`MiddleThreeEdgeObjectModeRadixAssertedHead -> InnerThreeEdgeObjectModeRadixAssertedHead`、`OuterThreeEdgeObjectModeRadixAssertedHead -> MiddleThreeEdgeObjectModeRadixAssertedHead`、`reserve x for OuterThreeEdgeObjectModeRadixAssertedHead`、`ThreeEdgeLocalObjectModeRadixAssertedHeadPayloadBoundary: x is MiddleThreeEdgeObjectModeRadixAssertedHead` だけを、Task 73 の real four-expansion object producer、Task 196 の formula consumer、Tasks 201/204/205 の変更しない closed immediate-radix relation を合成して閉じる。
    - distinct raw Outer subject/Middle asserted symbol/site/range を保持し、asserted symbol が outer expansion の real immediate radix であることを要求し、ordinal 1 を `BindingId(0)` に解決し、expansion 4 個だけを消費し、known entry 3 個を Base-definition-RHS builtin-object identity 1 個へ normalize し、expected constraint 0 個、`Inferred` variable 1 個、fact/candidate/diagnostic/deferred-free normalized-reflexive `Checked` `TypeAssertion` 1 個を object/set coercion なしで記録する。Task 73 は producer integration を所有する。set Tasks 122/138/146/148/150/195/201/203/205 および object Tasks 189/145/147/149/151/196/202/204 との bidirectional route isolation、全 23 nonidentity definition order、exact structural/provenance near miss、各 definition の missing/duplicate/label/spelling/radix probe、独立した expansion/binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinSet`/canonical corruption、immutable output、real frontend/resolver sidecar を要求する。test-first active fixture 1 件、shared backlink 5 件、dedicated backlink 1 件を追加し、既存 expectation を変更せず active runner を 153 から 154、cases を 368 から 369、requirements を 332 から 333 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。multi-hop Inner/Base assertion、other depth、imported/attributed/argument-bearing asserted head、general reachability/widening/`qua`、mode declaration acceptance/inhabitation、assertion truth/fact、closure/order、theorem acceptance、broader term/formula/child-graph semantics、proof/Core/ControlFlow/VC、general chain semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。checker source または module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 73、196、201、204、205。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

207. **Exact four-edge set-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
    - ordered definition 5 個 `BaseFourEdgeModeRadixAssertedHead -> set`、`InnerFourEdgeModeRadixAssertedHead -> BaseFourEdgeModeRadixAssertedHead`、`MiddleFourEdgeModeRadixAssertedHead -> InnerFourEdgeModeRadixAssertedHead`、`OuterFourEdgeModeRadixAssertedHead -> MiddleFourEdgeModeRadixAssertedHead`、`TooDeepFourEdgeModeRadixAssertedHead -> OuterFourEdgeModeRadixAssertedHead`、`reserve x for TooDeepFourEdgeModeRadixAssertedHead`、`FourEdgeLocalModeRadixAssertedHeadPayloadBoundary: x is OuterFourEdgeModeRadixAssertedHead` だけを、Task 74 の real five-expansion set producer、Task 197 の formula consumer、Tasks 201/203/205/206 の変更しない closed immediate-radix relation を合成して閉じる。
    - distinct raw TooDeep subject/Outer asserted symbol/site/range を保持し、asserted symbol が binding expansion の real immediate radix であることを要求し、ordinal 1 を `BindingId(0)` に解決し、expansion 5 個だけを消費し、known entry 3 個を Base-definition-RHS builtin-set identity 1 個へ normalize し、expected constraint 0 個、`Inferred` variable 1 個、fact/candidate/diagnostic/deferred-free normalized-reflexive `Checked` `TypeAssertion` 1 個を記録する。Task 74 は producer integration を所有する。set Tasks 122/138/146/148/150/152/197/201/203/205 および object Tasks 189/145/147/149/151/153/198/202/204/206 との bidirectional route isolation、全 119 nonidentity definition order、exact structural/provenance near miss、各 definition の missing/duplicate/label/spelling/radix probe、独立した expansion/binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinObject`/canonical corruption、immutable output、real frontend/resolver sidecar を要求する。test-first active fixture 1 件、shared backlink 5 件、dedicated backlink 1 件を追加し、既存 expectation を変更せず active runner を 154 から 155、cases を 369 から 370、requirements を 333 から 334 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。multi-hop Middle/Inner/Base assertion、matching object sibling、other depth、imported/attributed/argument-bearing asserted head、general reachability/widening/`qua`、mode declaration acceptance/inhabitation、assertion truth/fact、closure/order、theorem acceptance、broader term/formula/child-graph semantics、proof/Core/ControlFlow/VC、general chain semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。checker source または module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、197、201、203、205、206。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

208. **Exact four-edge object-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
    - ordered definition 5 個 `BaseFourEdgeObjectModeRadixAssertedHead -> object`、`InnerFourEdgeObjectModeRadixAssertedHead -> BaseFourEdgeObjectModeRadixAssertedHead`、`MiddleFourEdgeObjectModeRadixAssertedHead -> InnerFourEdgeObjectModeRadixAssertedHead`、`OuterFourEdgeObjectModeRadixAssertedHead -> MiddleFourEdgeObjectModeRadixAssertedHead`、`TooDeepFourEdgeObjectModeRadixAssertedHead -> OuterFourEdgeObjectModeRadixAssertedHead`、`reserve x for TooDeepFourEdgeObjectModeRadixAssertedHead`、`FourEdgeLocalObjectModeRadixAssertedHeadPayloadBoundary: x is OuterFourEdgeObjectModeRadixAssertedHead` だけを、Tasks 74/153 の real five-expansion object producer、Task 198 の formula consumer、Tasks 202/204/206/207 の変更しない closed immediate-radix relation を合成して閉じる。
    - distinct raw TooDeep subject/Outer asserted symbol/site/range を保持し、asserted symbol が binding expansion の real immediate radix であることを要求し、ordinal 1 を `BindingId(0)` に解決し、expansion 5 個だけを消費し、known entry 3 個を Base-definition-RHS builtin-object identity 1 個へ normalize し、expected constraint 0 個、`Inferred` variable 1 個、fact/candidate/diagnostic/deferred-free normalized-reflexive `Checked` `TypeAssertion` 1 個を object/set coercion なしで記録する。Task 74 は producer integration を所有する。set Tasks 122/138/146/148/150/152/197/201/203/205/207 および object Tasks 189/145/147/149/151/153/198/202/204/206 との bidirectional route isolation、全 119 nonidentity definition order、exact structural/provenance near miss、各 definition の missing/duplicate/label/spelling/radix/recovery/context/parameter/argument/attribute probe、non-exact reserve binding/type と extra reserve、non-exact formula label/subject/negation/status/recovery と extra item、asserted same-TooDeep、multi-hop Middle/Inner/Base、builtin `object`/`set`、local Other、argument-bearing/attributed head、connected sixth-edge rejection、unrelated-import positive control、全 mode symbol 5 個それぞれの imported/ambiguous substitution、全 expansion 5 個それぞれの removal、独立した expansion-payload/binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinSet`/canonical corruption、immutable output、real frontend/resolver sidecar を要求する。test-first active fixture 1 件、shared backlink 5 件、dedicated backlink 1 件を追加し、既存 expectation を変更せず active runner を 155 から 156、cases を 370 から 371、requirements を 334 から 335 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。multi-hop Middle/Inner/Base assertion、other depth、imported/attributed/argument-bearing asserted head、general reachability/widening/`qua`、mode declaration acceptance/inhabitation、assertion truth/fact、closure/order、theorem acceptance、broader term/formula/child-graph semantics、proof/Core/ControlFlow/VC、general chain semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。checker source または module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、153、198、202、204、206、207。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

209. **Exact seven-expansion set-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
    - ordered `BaseMode -> set`、`ChainMode1 -> BaseMode` から `ChainMode6 -> ChainMode5`、`reserve x for ChainMode6`、`LongLocalModeRadixAssertedHeadPayloadBoundary: x is ChainMode5` だけを、Task 74 の real seven-expansion producer、Task 199 の formula consumer、変更しない closed immediate-radix relation を合成して閉じる。Task 175 は既存 builtin asserted-type sibling/owner route と reusable output guard を提供し、expansion producer ではない。
    - distinct ChainMode6-subject/ChainMode5-asserted symbol/site/range を保持し、real immediate edge を要求し、ordinal 1 を `BindingId(0)` に解決し、expansion 7 個を消費し、known entry 3 個を BaseModeDef-RHS builtin-set identity 1 個へ normalize し、constraint/fact/candidate/diagnostic/deferred 0 個の inferred variable 1 個と checked assertion 1 個を出力する。全 5,039 nonidentity order、各 definition の missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed probe、non-exact reserve binding/type と extra reserve、formula label/subject/negation/status/recovery/extra-item probe、same `ChainMode6`、multi-hop `ChainMode4` から `BaseMode`、builtin `set`/`object`、local-other、argument-bearing、attributed asserted head、connected eighth edge、unrelated-import positive、全7 symbol の imported/ambiguous substitution、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinObject`/canonical corruption、immutable output、real sidecar、Task 209 実装前の既存 type-assertion owner route 34 件すべてとの bidirectional isolation を要求する。test-first fixture 1 件と shared 5 + dedicated 1 backlink を追加し、既存 expectation を変更せず active runner を 156 から 157、cases を 371 から 372、requirements を 335 から 336 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。multi-hop ChainMode4 から BaseMode、object sibling、imported positive expansion、attributed/argument-bearing head、general reachability/widening/`qua`、declaration/theorem acceptance、truth/fact、proof/Core/ControlFlow/VC、broader semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred。checker source/module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、175、199、201、203、205、207、208。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

210. **Exact seven-expansion object-terminal formula-side immediate-radix local-mode asserted-head checker bridge を追加する。** [x]
    - ordered `BaseObjectMode -> object`、`ChainObjectMode1 -> BaseObjectMode` から `ChainObjectMode6 -> ChainObjectMode5`、`reserve x for ChainObjectMode6`、`LongLocalObjectModeRadixAssertedHeadPayloadBoundary: x is ChainObjectMode5` だけを、Task 74 の real seven-expansion object producer、Task 200 の formula consumer、変更しない closed immediate-radix relation を合成して閉じる。Task 179 は builtin-object asserted-type sibling と reusable output guard、Task 209 は set-terminal immediate-radix sibling を提供する。
    - distinct ChainObjectMode6-subject/ChainObjectMode5-asserted symbol/site/range を保持し、real immediate edge を要求し、ordinal 1 を `BindingId(0)` に解決し、expansion 7 個を消費し、known entry 3 個を BaseObjectModeDef-RHS builtin-object identity 1 個へ normalize し、object/set coercion なしで constraint/fact/candidate/diagnostic/deferred 0 個の inferred variable 1 個と checked assertion 1 個を出力する。全 5,039 nonidentity order、各 definition の missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed probe、non-exact reserve binding/type、ChainObjectMode5/local-other/multi-binding reserve、extra reserve、formula label/subject/negation/status/recovery/extra-item probe、same `ChainObjectMode6`、multi-hop `ChainObjectMode4` から `BaseObjectMode`、builtin `object`/`set`、local-other、argument-bearing、attributed asserted head、connected eighth edge、unrelated-import positive、全7 symbol の imported/ambiguous substitution、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/immediate-edge/internal-link/`BuiltinSet`/canonical corruption、immutable output、real sidecar、既存 type-assertion owner route 35 件すべてとの bidirectional isolation を要求する。test-first fixture 1 件と shared 5 + dedicated 1 backlink を追加し、既存 expectation を変更せず active runner を 157 から 158、cases を 372 から 373、requirements を 336 から 337 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。multi-hop ChainObjectMode4 から BaseObjectMode、imported positive expansion、attributed/argument-bearing head、general reachability/widening/`qua`、object/set coercion、declaration/theorem acceptance、truth/fact、proof/Core/ControlFlow/VC、broader semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred。checker source/module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、179、200、202、204、206、208、209。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

211. **Exact two-edge set-terminal formula-side two-hop local-mode asserted-head checker bridge を追加する。** [x]
    - `BaseTwoHopModeAssertedHead -> set`、`MiddleTwoHopModeAssertedHead -> BaseTwoHopModeAssertedHead`、`OuterTwoHopModeAssertedHead -> MiddleTwoHopModeAssertedHead`、`reserve x for OuterTwoHopModeAssertedHead`、`TwoEdgeLocalModeTwoHopAssertedHeadPayloadBoundary: x is BaseTwoHopModeAssertedHead` だけを閉じる。Task 72 の real AST-derived expansion 3 個と既存 reserved-variable type-assertion producer/consumer を、新しい独立した closed two-link relation で合成する。
    - relation は pairwise-distinct resolved symbol 3 個を持つ real bare Outer-to-Middle edge と Middle-to-Base edge を明示的に検証し、exact Base-to-builtin-set terminal も要求する。generic terminal traversal だけを relation evidence にしない。distinct raw Outer-subject/Base-asserted symbol/site/range、ordinal 1 / `BindingId(0)`、expansion 3 個、Base-definition-RHS builtin-set identity 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。全5 nonidentity definition order、各 definition の missing/duplicate/label/spelling/radix/recovery/context/parameter/argument/attribute case、non-exact reserve/formula/extra-item case、same-Outer/immediate-Middle/builtin/local-other/object/deeper/argument-bearing/attributed asserted head、unrelated-import positive と全3 symbol の imported/ambiguous substitution、全 expansion、internal edge 2 本、terminal、binding、ordinal、subject/asserted head/spelling/site/range、canonical source の removal/独立 corruption、immutable output、real frontend/resolver sidecar、既存 type-assertion owner route 36 件すべてとの bidirectional isolation を要求する。test-first active fixture 1 件と shared 5 + dedicated 1 backlink を追加し、既存 expectation を変更せず active runner を 158 から 159、cases を 373 から 374、requirements を 337 から 338 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。object-terminal sibling、他 distance、imported-positive/attributed/argument-bearing asserted head、general reachability/widening/`qua`、mode declaration acceptance/inhabitation、assertion truth/fact、closure/order、theorem acceptance、broader term/formula/child-graph semantics、proof/Core/ControlFlow/VC、general chain semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred。checker source/module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 72、148、186、203。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

212. **Exact two-edge object-terminal formula-side two-hop local-mode asserted-head checker bridge を追加する。** [x]
    - `BaseTwoHopObjectModeAssertedHead -> object`、`MiddleTwoHopObjectModeAssertedHead -> BaseTwoHopObjectModeAssertedHead`、`OuterTwoHopObjectModeAssertedHead -> MiddleTwoHopObjectModeAssertedHead`、`reserve x for OuterTwoHopObjectModeAssertedHead`、`TwoEdgeLocalObjectModeTwoHopAssertedHeadPayloadBoundary: x is BaseTwoHopObjectModeAssertedHead` だけを閉じる。Task 72 の real object-terminal AST-derived expansion 3 個と既存 reserved-variable type-assertion producer/consumer を、Task 211 の closed two-link relation で合成する。
    - relation は pairwise-distinct resolved symbol 3 個を持つ real bare Outer-to-Middle edge と Middle-to-Base edge を明示的に検証し、exact Base-to-builtin-object terminal も要求する。generic terminal traversal だけを relation evidence にしない。distinct raw Outer-subject/Base-asserted symbol/site/range、ordinal 1 / `BindingId(0)`、expansion 3 個、Base-definition-RHS builtin-object identity 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を object/set coercion なしで保持する。全5 nonidentity definition order、各 definition の missing/duplicate/label/spelling/radix/recovery/context/parameter/argument/attribute case、non-exact reserve/formula/extra-item case、same-Outer/immediate-Middle/builtin-object/builtin-set/local-other/deeper/argument-bearing/attributed asserted head、unrelated-import positive と全3 symbol の imported/ambiguous substitution、全 expansion、internal edge 2 本、terminal、binding、ordinal、subject/asserted head/spelling/site/range、`BuiltinSet`、canonical source の removal/独立 corruption、immutable output、real frontend/resolver sidecar、既存 type-assertion owner route 37 件すべてとの bidirectional isolation を要求する。test-first active fixture 1 件と shared 5 + dedicated 1 backlink を追加し、既存 expectation を変更せず active runner を 159 から 160、cases を 374 から 375、requirements を 338 から 339 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。他 distance、imported-positive/attributed/argument-bearing asserted head、general reachability/widening/`qua`、object/set coercion、mode declaration acceptance/inhabitation、assertion truth/fact、closure/order、theorem acceptance、broader term/formula/child-graph semantics、proof/Core/ControlFlow/VC、general chain semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred。checker source/module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 72、149、187、204、211。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

213. **Exact three-edge set-terminal formula-side two-hop local-mode asserted-head checker bridge を追加する。** [x]
    - ordered definition 4 個 `BaseThreeEdgeModeTwoHopAssertedHead -> set`、`InnerThreeEdgeModeTwoHopAssertedHead -> BaseThreeEdgeModeTwoHopAssertedHead`、`MiddleThreeEdgeModeTwoHopAssertedHead -> InnerThreeEdgeModeTwoHopAssertedHead`、`OuterThreeEdgeModeTwoHopAssertedHead -> MiddleThreeEdgeModeTwoHopAssertedHead`、Outer reserve 1 個、`ThreeEdgeLocalModeTwoHopAssertedHeadPayloadBoundary: x is InnerThreeEdgeModeTwoHopAssertedHead` だけを閉じる。Task 73 の real AST-derived expansion 4 個、Task 195 の formula/checker consumer、Task 211 の closed two-link relation を合成する。
    - relation の pairwise-distinct な explicit Outer-to-Middle/Middle-to-Inner link を保持し、残る Inner-to-Base-to-set tail は terminal normalization だけで検証して relation evidence にはしない。distinct raw Outer-subject/Inner-asserted symbol/site/range、ordinal 1 / `BindingId(0)`、expansion 4 個、Base-definition-RHS builtin-set identity 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。全23 nonidentity definition order、全 definition の missing/duplicate/label/spelling/radix/recovery/context/parameter/argument/attribute variant、non-exact reserve/formula/extra-item と same/immediate/full-distance/builtin/object/local-other/deeper asserted head、unrelated-import positive と全4 symbol の imported/ambiguous substitution、全 expansion、relation link 2 本、tail link、terminal、binding、ordinal、raw subject/asserted head/spelling/site/range、canonical source の removal/独立 corruption、immutable output、real frontend/resolver sidecar、Tasks 211/212 focused regression、既存 type-assertion owner route 38 件すべてとの bidirectional isolation を要求する。test-first active fixture 1 件と shared 5 + dedicated 1 backlink を追加し、既存 expectation を変更せず active runner 160 から 161、cases 375 から 376、requirements 339 から 340、type-elaboration coverage 207/195 から 208/196 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。object sibling、Base/full-distance assertion、deeper/imported-positive/attributed/argument-bearing asserted head、general reachability/widening/`qua`、mode declaration acceptance/inhabitation、assertion truth/fact、closure/order、theorem acceptance、broader term/formula/child-graph semantics、proof/Core/ControlFlow/VC、general chain semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。checker source/module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 73、195、205、211、212。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

214. **Exact three-edge object-terminal formula-side two-hop local-mode asserted-head checker bridge を追加する。** [x]
    - ordered definition 4 個 `BaseThreeEdgeObjectModeTwoHopAssertedHead -> object`、`InnerThreeEdgeObjectModeTwoHopAssertedHead -> BaseThreeEdgeObjectModeTwoHopAssertedHead`、`MiddleThreeEdgeObjectModeTwoHopAssertedHead -> InnerThreeEdgeObjectModeTwoHopAssertedHead`、`OuterThreeEdgeObjectModeTwoHopAssertedHead -> MiddleThreeEdgeObjectModeTwoHopAssertedHead`、Outer reserve 1 個、`ThreeEdgeLocalObjectModeTwoHopAssertedHeadPayloadBoundary: x is InnerThreeEdgeObjectModeTwoHopAssertedHead` だけを閉じる。Task 73 の real AST-derived object expansion 4 個、Task 196 の formula/checker consumer、Tasks 211-213 が使う変更しない closed two-link relation を合成する。
    - relation の pairwise-distinct な explicit Outer-to-Middle/Middle-to-Inner link を保持し、残る Inner-to-Base-to-object tail は terminal normalization だけで検証して relation evidence にはしない。distinct raw Outer-subject/Inner-asserted symbol/site/range、ordinal 1 / `BindingId(0)`、expansion 4 個、Base-definition-RHS builtin-object identity 1 個へ normalize する known entry 3 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。全23 nonidentity definition order、全 definition の missing/duplicate/label/spelling/radix/recovery/context/parameter/argument/attribute variant、non-exact reserve/formula/extra-item と same/immediate/full-distance/builtin-object/builtin-set/local-other/deeper asserted head、unrelated-import positive と全4 symbol の imported/ambiguous substitution、全 expansion、relation link 2 本、tail link、terminal、binding、ordinal、raw subject/asserted head/spelling/site/range、`BuiltinSet`、canonical source の removal/独立 corruption、immutable output、real frontend/resolver sidecar、Tasks 211/212/213 focused regression、既存 type-assertion owner route 39 件すべてとの bidirectional isolation を要求する。test-first active fixture 1 件と shared 5 + dedicated 1 backlink を追加し、既存 expectation を変更せず active runner 161 から 162、cases 376 から 377、requirements 340 から 341、type-elaboration coverage 208/196 から 209/197、pass/fail 192/184 から 193/184 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。Base/full-distance assertion、deeper/imported-positive/attributed/argument-bearing asserted head、general reachability/widening/`qua`、mode declaration acceptance/inhabitation、assertion truth/fact、closure/order、theorem acceptance、broader term/formula/child-graph semantics、proof/Core/ControlFlow/VC、object/set coercion、general chain semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。checker source/module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 73、196、206、211、212、213。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

215. **Exact four-edge set-terminal formula-side two-hop local-mode asserted-head checker bridge を追加する。** [x]
    - ordered definition 5 個 `BaseFourEdgeModeTwoHopAssertedHead -> set`、`InnerFourEdgeModeTwoHopAssertedHead -> BaseFourEdgeModeTwoHopAssertedHead`、`MiddleFourEdgeModeTwoHopAssertedHead -> InnerFourEdgeModeTwoHopAssertedHead`、`OuterFourEdgeModeTwoHopAssertedHead -> MiddleFourEdgeModeTwoHopAssertedHead`、`TooDeepFourEdgeModeTwoHopAssertedHead -> OuterFourEdgeModeTwoHopAssertedHead`、TooDeep reserve 1 個、`FourEdgeLocalModeTwoHopAssertedHeadPayloadBoundary: x is MiddleFourEdgeModeTwoHopAssertedHead` だけを閉じる。Task 74 の real AST-derived set expansion 5 個、Task 197 の formula/checker consumer、Tasks 211-214 が使う byte-for-byte 変更しない `BindingTwoHopRadix` relation を合成する。
    - relation の pairwise-distinct な explicit TooDeep-to-Outer/Outer-to-Middle link を保持し、残る Middle-to-Inner-to-Base-to-set tail は terminal normalization だけで検証して relation evidence にはしない。distinct raw TooDeep-subject/Middle-asserted symbol/site/range、ordinal 1 / `BindingId(0)`、expansion 5 個、Base-definition-RHS builtin-set identity 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。全119 nonidentity definition order、全 definition の missing/duplicate/label/spelling/radix/recovery/context/parameter/argument/attribute variant、non-exact reserve/formula/extra-item と same/immediate/three-hop/full-distance/builtin/local-other/deeper asserted head、unrelated-import positive と全5 symbol の imported/ambiguous substitution、全 expansion、relation link 2 本、全 terminal-tail link、terminal、binding、ordinal、raw subject/asserted head/spelling/site/range、`BuiltinObject`、canonical source の removal/独立 corruption、immutable output、real frontend/resolver sidecar、Tasks 211/212/213/214 focused regression、既存 type-assertion owner route 40 件すべてとの bidirectional isolation を要求する。test-first active fixture 1 件と shared 5 + dedicated 1 backlink を追加し、既存 expectation を変更せず active runner 162 から 163、cases 377 から 378、requirements 341 から 342、type-elaboration coverage 209/197 から 210/198、pass/fail 193/184 から 194/184 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。object sibling、three-hop Inner/full-distance Base assertion、deeper/imported-positive/attributed/argument-bearing asserted head、general reachability/widening/`qua`、mode declaration acceptance/inhabitation、assertion truth/fact、closure/order、theorem acceptance、broader term/formula/child-graph semantics、proof/Core/ControlFlow/VC、general chain semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。checker source/module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、197、207、211、212、213、214。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

216. **Exact four-edge object-terminal formula-side two-hop local-mode asserted-head checker bridge を追加する。** [x]
    - ordered definition 5 個 `BaseFourEdgeObjectModeTwoHopAssertedHead -> object`、`InnerFourEdgeObjectModeTwoHopAssertedHead -> BaseFourEdgeObjectModeTwoHopAssertedHead`、`MiddleFourEdgeObjectModeTwoHopAssertedHead -> InnerFourEdgeObjectModeTwoHopAssertedHead`、`OuterFourEdgeObjectModeTwoHopAssertedHead -> MiddleFourEdgeObjectModeTwoHopAssertedHead`、`TooDeepFourEdgeObjectModeTwoHopAssertedHead -> OuterFourEdgeObjectModeTwoHopAssertedHead`、TooDeep reserve 1 個、`FourEdgeLocalObjectModeTwoHopAssertedHeadPayloadBoundary: x is MiddleFourEdgeObjectModeTwoHopAssertedHead` だけを閉じる。Task 74 の real AST-derived object expansion 5 個、Task 198 の formula/checker consumer、Tasks 211-215 が使う byte-for-byte 変更しない `BindingTwoHopRadix` relation を合成する。
    - relation の pairwise-distinct な explicit TooDeep-to-Outer/Outer-to-Middle link を保持し、残る Middle-to-Inner-to-Base-to-object tail は terminal normalization だけで検証して relation evidence にはしない。distinct raw TooDeep-subject/Middle-asserted symbol/site/range、ordinal 1 / `BindingId(0)`、expansion 5 個、Base-definition-RHS builtin-object identity 1 個へ normalize する known entry 3 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。全119 nonidentity definition order、全 definition の missing/duplicate/label/spelling/radix/recovery/context/parameter/argument/attribute variant、non-exact reserve/formula/extra-item と same/immediate/three-hop/full-distance/builtin/local-other/deeper asserted head、unrelated-import positive と全5 symbol の imported/ambiguous substitution、全 expansion、relation link 2 本、全 terminal-tail link、terminal、binding、ordinal、raw subject/asserted head/spelling/site/range、`BuiltinSet`、canonical source の removal/独立 corruption、immutable output、real frontend/resolver sidecar、Tasks 211-215 focused regression、既存 type-assertion owner route 41 件すべてとの bidirectional isolation を要求する。test-first active fixture 1 件と shared 5 + dedicated 1 backlink を追加し、既存 expectation を変更せず active runner 163 から 164、cases 378 から 379、requirements 342 から 343、type-elaboration coverage 210/198 から 211/199、pass/fail 194/184 から 195/184 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。three-hop Inner/full-distance Base assertion、deeper/imported-positive/attributed/argument-bearing asserted head、general reachability/widening/`qua`、mode declaration acceptance/inhabitation、assertion truth/fact、closure/order、theorem acceptance、broader term/formula/child-graph semantics、proof/Core/ControlFlow/VC、object/set coercion、general chain semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。checker source/module-layout change は不要であった。
    - 検証: `cargo test -p mizar-test` と workspace Rust verification。
    - 依存: tasks 74、198、208、211、212、213、214、215。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

217. **Exact three-edge set-terminal formula-side three-hop local-mode asserted-head checker bridge を追加する。** [x]
    - ordered definition 4 個 `BaseThreeEdgeModeThreeHopAssertedHead -> set`、`InnerThreeEdgeModeThreeHopAssertedHead -> BaseThreeEdgeModeThreeHopAssertedHead`、`MiddleThreeEdgeModeThreeHopAssertedHead -> InnerThreeEdgeModeThreeHopAssertedHead`、`OuterThreeEdgeModeThreeHopAssertedHead -> MiddleThreeEdgeModeThreeHopAssertedHead`、Outer reserve 1 個、`ThreeEdgeLocalModeThreeHopAssertedHeadPayloadBoundary: x is BaseThreeEdgeModeThreeHopAssertedHead` だけを閉じる。Task 73 の real AST-derived set expansion 4 個と Task 195 の formula/checker consumer を合成する。pairwise-distinct な Outer-to-Middle、Middle-to-Inner、Inner-to-Base link を直接検証する closed `BindingThreeHopRadix` relation を追加する。Base-to-set は terminal-normalization evidence のみで、generic relation evidence にはしない。`BindingTwoHopRadix` と既存 expectation は変更しない。
    - distinct raw Outer-subject/Base-asserted symbol/site/range、ordinal 1 / `BindingId(0)`、real expansion 4 個、Base-definition-RHS builtin-set identity 1 個へ normalize する known entry 3 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。全23 nonidentity definition order、全 definition の missing/duplicate/label/spelling/radix/recovery/context/parameter/argument/attribute variant、non-exact reserve/formula/extra-item と same/immediate/two-hop/builtin/local-other/deeper asserted head、unrelated-import positive と全4 symbol の imported/ambiguous substitution、全 expansion、relation link 3 本、terminal、binding、ordinal、raw subject/asserted head/spelling/site/range、`BuiltinObject`、canonical source の removal/独立 corruption、immutable output、real frontend/resolver sidecar、Tasks 211-216 focused regression、既存 type-assertion owner route 42 件すべてとの bidirectional isolation を要求する。test-first active fixture 1 件と shared 5 + dedicated 1 backlink を追加し、既存 expectation を変更せず active runner 164 から 165、cases 379 から 380、requirements 343 から 344、type-elaboration coverage 211/199 から 212/200、pass/fail 195/184 から 196/184 へ増やす。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。object sibling、four-edge/long-chain three-hop/full-distance assertion、imported-positive/attributed/argument-bearing head、generic reachability/widening/`qua`、mode declaration acceptance/inhabitation、assertion truth/fact、closure/order、theorem acceptance、broader term/formula/child-graph semantics、proof/CoreIr/ControlFlowIr/VC、object/set coercion、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。checker source/module-layout change は不要であった。
    - 検証: focused Task 217 と Tasks 211-216 regression、`cargo test -p mizar-test`、`cargo test -p mizar-checker`、workspace Rust verification。
    - 依存: tasks 73、195、211-216。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

218. **Exact three-edge object-terminal formula-side three-hop local-mode asserted-head checker bridge を追加する。** [x]
    - ordered definition 4 個 `BaseThreeEdgeObjectModeThreeHopAssertedHead -> object`、`InnerThreeEdgeObjectModeThreeHopAssertedHead -> BaseThreeEdgeObjectModeThreeHopAssertedHead`、`MiddleThreeEdgeObjectModeThreeHopAssertedHead -> InnerThreeEdgeObjectModeThreeHopAssertedHead`、`OuterThreeEdgeObjectModeThreeHopAssertedHead -> MiddleThreeEdgeObjectModeThreeHopAssertedHead`、Outer reserve 1 個、`ThreeEdgeLocalObjectModeThreeHopAssertedHeadPayloadBoundary: x is BaseThreeEdgeObjectModeThreeHopAssertedHead` だけを閉じた。Task 73 の real AST-derived object expansion 4 個、Task 196 の formula/checker consumer、Task 217 の byte-for-byte 変更しない `BindingThreeHopRadix` relation を合成する。pairwise-distinct な Outer-to-Middle、Middle-to-Inner、Inner-to-Base link を直接検証し、Base-to-object は terminal-normalization evidence のみで generic relation evidence にはしない。既存 expectation は変更しなかった。
    - distinct raw Outer-subject/Base-asserted symbol/site/range、ordinal 1 / `BindingId(0)`、real expansion 4 個、Base-definition-RHS builtin-object identity 1 個へ normalize する known entry 3 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。全23 nonidentity definition order、全 definition の missing/duplicate/label/spelling/radix/recovery/context/parameter/argument/attribute variant、non-exact reserve/formula/extra-item と same/immediate/two-hop/builtin/local-other/deeper asserted head、unrelated-import positive と全4 symbol の imported/ambiguous substitution、全 expansion、relation link 3 本、terminal、binding、ordinal、raw subject/asserted head/spelling/site/range、`BuiltinSet`、canonical source の removal/独立 corruption、immutable output、real frontend/resolver sidecar、Tasks 211-217 focused regression、既存 type-assertion owner route 43 件すべてとの bidirectional isolation を網羅した。test-first active fixture 1 件と shared 5 + dedicated 1 backlink により、既存 expectation を変更せず active runner 165 から 166、cases 380 から 381、requirements 344 から 345、type-elaboration coverage 212/200 から 213/201、pass/fail 196/184 から 197/184 へ増えた。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。four-edge/long-chain three-hop/full-distance assertion、imported-positive/attributed/argument-bearing head、generic reachability/widening/`qua`、mode declaration acceptance/inhabitation、assertion truth/fact、closure/order、theorem acceptance、broader term/formula/child-graph semantics、proof/CoreIr/ControlFlowIr/VC、object/set coercion、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。checker source/module-layout change は不要であった。
    - 検証: focused Task 218 と Tasks 211-217 regression、`cargo test -p mizar-test`、`cargo test -p mizar-checker`、workspace Rust verification。
    - 依存: tasks 73、196、211-217。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

219. [x] **Exact four-edge set-terminal three-hop asserted head を bridge する。**
    - ordered definition 5 個 `BaseFourEdgeModeThreeHopAssertedHead -> set`、`InnerFourEdgeModeThreeHopAssertedHead -> BaseFourEdgeModeThreeHopAssertedHead`、`MiddleFourEdgeModeThreeHopAssertedHead -> InnerFourEdgeModeThreeHopAssertedHead`、`OuterFourEdgeModeThreeHopAssertedHead -> MiddleFourEdgeModeThreeHopAssertedHead`、`TooDeepFourEdgeModeThreeHopAssertedHead -> OuterFourEdgeModeThreeHopAssertedHead`、TooDeep reserve 1 個、`FourEdgeLocalModeThreeHopAssertedHeadPayloadBoundary: x is InnerFourEdgeModeThreeHopAssertedHead` だけを閉じた。Task 74 の real AST-derived set expansion 5 個、Task 197 の formula/checker consumer、Task 217 由来の byte-for-byte 変更しない `BindingThreeHopRadix` relation を合成する。
    - TooDeep-to-Outer、Outer-to-Middle、Middle-to-Inner link を直接検証し、Inner-to-Base-to-set tail は terminal normalization evidence のみに保った。distinct raw TooDeep/Inner symbol/site/range provenance、ordinal 1 / `BindingId(0)`、real expansion 5 個、Base-definition-RHS builtin-set identity 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。全119 nonidentity order、unconnected unsupported deeper asserted head と actual connected sixth-definition/sixth-edge asserted head の独立 guard を含む finite definition/reserve/formula/provenance/corruption matrix、immutable output、real sidecar、Task 207 と Tasks 211-218 focused regression、先行 type-assertion owner route 44 件との bidirectional isolation を網羅した。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。object sibling、Base full-distance assertion、imported-positive/attributed/argument-bearing head、generic reachability/widening/`qua`、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。checker source/module-layout change は不要であった。
    - 検証: focused Tasks 207、219、211-218 regression、`cargo test -p mizar-test`、`cargo test -p mizar-checker`、workspace Rust verification。
    - 依存: tasks 74、197、207、211-218。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

220. [x] **Exact four-edge object-terminal three-hop asserted head を bridge する。**
    - ordered definition 5 個 `BaseFourEdgeObjectModeThreeHopAssertedHead -> object`、`InnerFourEdgeObjectModeThreeHopAssertedHead -> BaseFourEdgeObjectModeThreeHopAssertedHead`、`MiddleFourEdgeObjectModeThreeHopAssertedHead -> InnerFourEdgeObjectModeThreeHopAssertedHead`、`OuterFourEdgeObjectModeThreeHopAssertedHead -> MiddleFourEdgeObjectModeThreeHopAssertedHead`、`TooDeepFourEdgeObjectModeThreeHopAssertedHead -> OuterFourEdgeObjectModeThreeHopAssertedHead`、TooDeep reserve 1 個、`FourEdgeLocalObjectModeThreeHopAssertedHeadPayloadBoundary: x is InnerFourEdgeObjectModeThreeHopAssertedHead` だけを閉じた。Task 74 の real AST-derived object expansion 5 個、Task 198 の formula/checker consumer、Task 208 の immediate-edge sibling guard、Task 217 由来の byte-for-byte 変更しない `BindingThreeHopRadix` relation を合成する。
    - pairwise-distinct な TooDeep-to-Outer、Outer-to-Middle、Middle-to-Inner link を直接検証し、Inner-to-Base-to-object tail は terminal normalization evidence のみに保った。distinct raw TooDeep/Inner symbol/site/range provenance、ordinal 1 / `BindingId(0)`、real expansion 5 個、Base-definition-RHS builtin-object identity 1 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。全119 nonidentity order、unconnected unsupported deeper asserted head と actual connected sixth-definition/sixth-edge asserted head の独立 guard を含む finite definition/reserve/formula/provenance/corruption matrix、immutable output、real sidecar、Tasks 208 と 211-219 focused regression、先行 type-assertion owner route 45 件との bidirectional isolation を網羅した。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。Base full-distance assertion、imported-positive/attributed/argument-bearing head、generic reachability/widening/`qua`、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、object/set coercion、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。checker source/module-layout change は不要であった。
    - active fixture 1 件と shared 5 + dedicated 1 backlink により、既存 expectation を変更せず active runner 167 から 168、cases 382 から 383、requirements 346 から 347、type-elaboration coverage 214/202 から 215/203、pass/fail 198/184 から 199/184 へ増えた。
    - focused Task 220、Tasks 208 と 211-219 regression、`cargo test -p mizar-test`、`cargo test -p mizar-checker`、`cargo fmt --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo test`、`git diff --check` は成功した。
    - 依存: tasks 74、198、208、211-219。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

221. [x] **Exact four-edge set-terminal full-distance four-hop asserted head を bridge する。**
    - ordered definition 5 個 `BaseFourEdgeModeFourHopAssertedHead -> set`、`InnerFourEdgeModeFourHopAssertedHead -> BaseFourEdgeModeFourHopAssertedHead`、`MiddleFourEdgeModeFourHopAssertedHead -> InnerFourEdgeModeFourHopAssertedHead`、`OuterFourEdgeModeFourHopAssertedHead -> MiddleFourEdgeModeFourHopAssertedHead`、`TooDeepFourEdgeModeFourHopAssertedHead -> OuterFourEdgeModeFourHopAssertedHead`、TooDeep reserve 1 個、`FourEdgeLocalModeFourHopAssertedHeadPayloadBoundary: x is BaseFourEdgeModeFourHopAssertedHead` だけを追加する。Task 74 の real AST-derived set expansion 5 個と Task 197 の formula/checker consumer を合成する。
    - closed `BindingFourHopRadix` で pairwise-distinct な TooDeep-to-Outer、Outer-to-Middle、Middle-to-Inner、Inner-to-Base link を直接検証し、Base-to-set は terminal-normalization evidence のみに保つ。distinct raw TooDeep/Base symbol/site/range provenance、ordinal 1 / `BindingId(0)`、real expansion 5 個、Base-definition-RHS builtin-set identity 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。
    - 全119 nonidentity definition order、各 definition の missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variant、exact reserve/formula/head restriction、全5 symbol の imported/ambiguous rejection と unrelated-import positive、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/four-link/terminal/`BuiltinObject`/canonical corruption、unconnected-deeper と connected sixth-definition/fifth-link の独立 guard、immutable output、real sidecar、Task 207 と Tasks 211-220 focused regression、先行 type-assertion owner route 46 件との bidirectional isolation を検証する。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。object sibling、longer chain、imported-positive definition、attributed/argument-bearing head、generic reachability/widening/`qua`、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。
    - active fixture と backlink 6 件は、既存 expectation を変更せず active runner 169、384 cases、348 requirements、type-elaboration coverage 216/204、pass/fail 200/184 を計上する。relevant crate verification は成功し、checker source/module-layout change は不要であった。
    - 依存: tasks 74、197、207、211-220。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

222. [x] **Exact four-edge object-terminal full-distance four-hop asserted head を bridge する。**
    - ordered definition 5 個 `BaseFourEdgeObjectModeFourHopAssertedHead -> object`、`InnerFourEdgeObjectModeFourHopAssertedHead -> BaseFourEdgeObjectModeFourHopAssertedHead`、`MiddleFourEdgeObjectModeFourHopAssertedHead -> InnerFourEdgeObjectModeFourHopAssertedHead`、`OuterFourEdgeObjectModeFourHopAssertedHead -> MiddleFourEdgeObjectModeFourHopAssertedHead`、`TooDeepFourEdgeObjectModeFourHopAssertedHead -> OuterFourEdgeObjectModeFourHopAssertedHead`、TooDeep reserve 1 個、`FourEdgeLocalObjectModeFourHopAssertedHeadPayloadBoundary: x is BaseFourEdgeObjectModeFourHopAssertedHead` だけを追加した。route は Task 74 の real AST-derived object expansion 5 個と Task 198 の formula/checker consumer を合成し、Task 221 の closed `BindingFourHopRadix` を byte-for-byte 変更せず再利用する。
    - pairwise-distinct な TooDeep-to-Outer、Outer-to-Middle、Middle-to-Inner、Inner-to-Base link を直接検証し、Base-to-object は terminal-normalization evidence のみに保つ。distinct raw TooDeep/Base symbol/site/range provenance、ordinal 1 / `BindingId(0)`、real expansion 5 個、Base-definition-RHS builtin-object identity 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を object/set coercion なしで保持する。
    - 全119 nonidentity definition order、各 definition の missing/duplicate/label/spelling/radix/recovery/contextual/parameterized/argument-bearing/attributed variant、exact reserve/formula/head restriction、全5 symbol の imported/ambiguous rejection と unrelated-import positive、全 expansion removal、独立 binding/ordinal/head/spelling/site/range/4 link/terminal/`BuiltinSet`/canonical corruption、unconnected-deeper と connected sixth-definition/fifth-link の独立 guard、immutable output、real sidecar、Task 208 と Tasks 211-221 focused regression、先行 type-assertion owner route 47 件との bidirectional isolation を test する。
    - 分類は `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。longer chain、imported-positive definition、attributed/argument-bearing head、generic reachability/widening/`qua`、object/set coercion、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、downstream payload は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。
    - active fixture と backlink 6 件は、既存 expectation を変更せず active runner 170、385 cases、349 requirements、type-elaboration coverage 217/205、pass/fail 201/184 を計上する。relevant-crate と workspace verification は成功した。checker source/module-layout change は不要であった。
    - 依存: Tasks 74、198、208、211-221。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

223. [x] **Exact transparent single-parenthesized reserved-variable equality を bridge する。**
    - `reserve x for set;` と `ParenthesizedReservedVariableEqualityPayloadBoundary: (x) = x;` だけを追加する。real parser `ParenthesizedTerm` と Task 119 の reserve extraction、`BindingEnv` lookup、builtin-set projection、equality consumer を合成し、direct/direct Task 119 route は変更しない。
    - direct `(` / `)` token、nested `TermExpression` 1 個、identifier `TermReference` 1 個を持つ unrecovered left wrapper 1 個だけを検証し、direct right `x` を要求する。source payload metadata で distinct wrapper/inner/right site/range を保持し、inner/right reference を ordinal 1/2 で `BindingId(0)` へ解決し、inner reference の real reserve-derived type/value を再利用して wrapper を透明に lower する。parenthesis 独自 type/axiom/fact/FOL node または fabricated child payload は emit しない。
    - exact frontend/resolver sidecar、direct/right/both/nested/empty/non-identifier/recovered/malformed wrapper rejection、non-exact reserve/theorem/label/operator/operand rejection、独立 wrapper/inner/right site/range、binding、ordinal、spelling、head、result/expected-type corruption、matched output、immutable output、先行 reserved-variable binary-formula owner route 52 件との bidirectional isolation を test する。
    - 分類は `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。arbitrary nesting/operand、general precedence、formula parenthesization、closure/order materialization、equality truth/fact、theorem acceptance、proof/CoreIr/ControlFlowIr/VC、child graph、broader term/formula semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。
    - active fixture と backlink 5 件は、既存 expectation を変更せず active runner 171、386 cases、350 requirements、type-elaboration coverage 218/206、pass/fail 202/184 を計上する。focused、relevant-crate、workspace verification は成功した。checker source/module-layout change は不要であった。
    - 依存: Tasks 9、119。参照: Step 5、mizar-test task 10、specs 4、13、14、16。

224. [x] **exact seven-expansion set-terminal two-hop asserted head を bridge する。**
    - ordered bare definition 7 個 `BaseMode -> set` から `ChainMode6 -> ChainMode5`、`ChainMode6` reserve 1 個、`LongLocalModeTwoHopAssertedHeadPayloadBoundary: x is ChainMode4` だけを追加する。Task 74 の real expansion producer、Task 199 の formula/checker consumer、Task 211 の byte-for-byte unchanged `BindingTwoHopRadix` を合成し、Task 209 は immediate-edge sibling regression のみに使う。
    - pairwise-distinct `ChainMode6 -> ChainMode5 -> ChainMode4` link を直接検証する。`ChainMode4 -> ChainMode3 -> ChainMode2 -> ChainMode1 -> BaseMode -> set` は cycle-safe terminal-normalization evidence のみに使う。distinct subject/asserted provenance、ordinal 1 / `BindingId(0)`、real expansion 7 個、BaseModeDef-RHS `BuiltinSet` 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。
    - 全5,039 nonidentity order、finite definition/reserve/formula/head/provenance/removal/corruption variant、relation link 2 本、全 tail link、terminal/canonical anchor/binding/ordinal/site/range、connected eighth-edge rejection、unrelated-import positive、immutable output、real sidecar、focused relation sibling、先行 type-assertion owner 48 件との bidirectional isolation を test する。
    - 分類は `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。object-terminal、imported-positive definition、attributed/argument-bearing head、generic reachability/widening/`qua`、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、broader semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。
    - test-first fixture と backlink 6 件は既存 expectation を変更せず active runner 172、387 cases、351 requirements、type-elaboration 219/207、pass/fail 203/184 を計上する。focused、relevant-crate、workspace verification は成功した。checker source/module-layout change は不要であった。
    - Dependencies: Tasks 74, 199, 209, 211。References: Step 5、mizar-test task 10、specs 3, 4, 7, 13, 14, 16。

225. [x] **exact seven-expansion object-terminal two-hop asserted head を bridge する。**
    - ordered bare definition 7 個 `BaseObjectMode -> object` から `ChainObjectMode6 -> ChainObjectMode5`、`ChainObjectMode6` reserve 1 個、`LongLocalObjectModeTwoHopAssertedHeadPayloadBoundary: x is ChainObjectMode4` だけを追加する。Task 74 の real object expansion producer、Task 200 の formula/checker consumer、Task 211 の byte-for-byte unchanged `BindingTwoHopRadix` を合成し、Task 210 は immediate-edge sibling、Task 224 は set-terminal two-hop sibling として使う。
    - pairwise-distinct `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4` link を直接検証する。`ChainObjectMode4 -> ChainObjectMode3 -> ChainObjectMode2 -> ChainObjectMode1 -> BaseObjectMode -> object` は cycle-safe terminal-normalization evidence のみに使う。distinct subject/asserted provenance、ordinal 1 / `BindingId(0)`、real expansion 7 個、BaseObjectModeDef-RHS `BuiltinObject` 1 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。
    - 全5,039 nonidentity order、finite definition/reserve/formula/head/provenance/removal/corruption variant、relation link 2 本、全 tail link、terminal/canonical anchor/binding/ordinal/site/range、connected eighth-edge rejection、unrelated-import positive、immutable output、real sidecar、focused sibling、先行 type-assertion owner 49 件との bidirectional isolation を test する。
    - 分類は `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。Task 224 を超える set-terminal variant、imported-positive definition、attributed/argument-bearing head、generic reachability/widening/`qua`、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、broader semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。
    - test-first fixture と backlink 6 件は既存 expectation を変更せず active runner 173、388 cases、352 requirements、type-elaboration 220/208、pass/fail 204/184 を計上する。focused、relevant-crate、workspace verification は成功した。checker source/module-layout change は不要であった。
    - Dependencies: Tasks 74, 200, 210, 211, 224。References: Step 5、mizar-test task 10、specs 3, 4, 7, 13, 14, 16。

226. [x] **exact seven-expansion set-terminal three-hop asserted head を bridge する。**
    - ordered bare definition 7 個 `BaseMode -> set` から `ChainMode6 -> ChainMode5`、`ChainMode6` reserve 1 個、`LongLocalModeThreeHopAssertedHeadPayloadBoundary: x is ChainMode3` だけを追加する。Task 74 の real expansion producer、Task 199 の formula/checker consumer、byte-for-byte unchanged `BindingThreeHopRadix` を合成し、Tasks 209/224 と 217/219 は shorter-distance/three-hop sibling として使う。
    - pairwise-distinct `ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3` link を直接検証する。`ChainMode3 -> ChainMode2 -> ChainMode1 -> BaseMode -> set` は cycle-safe terminal-normalization evidence のみに使う。distinct subject/asserted provenance、ordinal 1 / `BindingId(0)`、real expansion 7 個、BaseModeDef-RHS `BuiltinSet` 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。
    - 全5,039 nonidentity order、finite definition/reserve/formula/head/provenance/removal/corruption variant、relation link 3 本、全 tail link、terminal/canonical anchor/binding/ordinal/site/range、connected eighth-edge rejection、unrelated-import positive、immutable output、real sidecar、focused sibling、先行 type-assertion owner 50 件との bidirectional isolation を test する。
    - 分類は `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。object-terminal、imported-positive definition、attributed/argument-bearing head、generic reachability/widening/`qua`、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、broader semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。
    - test-first fixture と backlink 6 件は既存 expectation を変更せず active runner 174、389 cases、353 requirements、type-elaboration 221/209、pass/fail 205/184 を計上する。focused、relevant-crate、workspace verification は成功した。checker source/module-layout change は不要であった。
    - Dependencies: Tasks 74, 199, 209, 217, 219, 224。References: Step 5、mizar-test task 10、specs 3, 4, 7, 13, 14, 16。

227. [x] **exact seven-expansion object-terminal three-hop asserted head を bridge する。**
    - ordered bare definition 7 個 `BaseObjectMode -> object` から `ChainObjectMode6 -> ChainObjectMode5`、`ChainObjectMode6` reserve 1 個、`LongLocalObjectModeThreeHopAssertedHeadPayloadBoundary: x is ChainObjectMode3` だけを追加する。Task 74 の real object expansion producer、Task 200 の formula/checker consumer、byte-for-byte unchanged `BindingThreeHopRadix` を合成し、Tasks 210/225、217/220、226 は shorter-distance/terminal sibling として使う。
    - pairwise-distinct `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4 -> ChainObjectMode3` link を直接検証する。`ChainObjectMode3 -> ChainObjectMode2 -> ChainObjectMode1 -> BaseObjectMode -> object` は cycle-safe terminal-normalization evidence のみに使う。distinct subject/asserted provenance、ordinal 1 / `BindingId(0)`、real expansion 7 個、BaseObjectModeDef-RHS `BuiltinObject` 1 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。
    - 全5,039 nonidentity order、finite definition/reserve/formula/head/provenance/removal/corruption variant、relation link 3 本、全 tail link、terminal/canonical anchor/binding/ordinal/site/range、connected eighth-edge rejection、unrelated-import positive、immutable output、real sidecar、focused sibling、先行 type-assertion owner 51 件との bidirectional isolation を test する。
    - 分類は `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。imported-positive definition、attributed/argument-bearing head、generic reachability/widening/`qua`、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、broader semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。
    - test-first fixture と backlink 6 件は既存 expectation を変更せず active runner 175、390 cases、354 requirements、type-elaboration 222/210、pass/fail 206/184 を計上する。focused、relevant-crate、workspace verification は成功した。checker source/module-layout change は不要であった。
    - Dependencies: Tasks 74, 200, 210, 217, 220, 225, 226。References: Step 5、mizar-test task 10、specs 3, 4, 7, 13, 14, 16。

228. [x] **exact seven-expansion set-terminal four-hop asserted head を bridge する。**
    - ordered bare definition 7 個 `BaseMode -> set` から `ChainMode6 -> ChainMode5`、`ChainMode6` reserve 1 個、`LongLocalModeFourHopAssertedHeadPayloadBoundary: x is ChainMode2` だけを追加した。route は Task 74 の real expansion producer、Task 199 の formula/checker consumer、byte-for-byte unchanged `BindingFourHopRadix` を合成し、Tasks 221/222、224/226、227 は relation/shorter-distance/terminal sibling のままとする。
    - pairwise-distinct `ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3 -> ChainMode2` link を直接検証する。`ChainMode2 -> ChainMode1 -> BaseMode -> set` は cycle-safe terminal-normalization evidence のみに使い、distinct subject/asserted provenance、ordinal 1 / `BindingId(0)`、real expansion 7 個、BaseModeDef-RHS `BuiltinSet` 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。
    - 全5,039 nonidentity order、finite definition/reserve/formula/head/provenance/removal/corruption variant、relation link 4 本、全 tail link、terminal/canonical anchor/binding/ordinal/site/range、connected fifth-hop rejection、unrelated-import positive、immutable output、real sidecar、focused sibling、先行 type-assertion owner 52 件との bidirectional isolation を test する。
    - 分類は `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。object-terminal、imported-positive definition、attributed/argument-bearing head、generic reachability/widening/`qua`、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、broader semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。
    - test-first fixture と backlink 6 件は既存 expectation を変更せず active runner 176、391 cases、355 requirements、type-elaboration 223/211、pass/fail 207/184 を計上する。focused、relevant-crate、workspace verification は成功した。checker source/module-layout change は不要であった。
    - Dependencies: Tasks 74, 199, 221, 224, 226, 227。References: Step 5、mizar-test task 10、specs 3, 4, 7, 13, 14, 16。

229. [x] **exact seven-expansion object-terminal four-hop asserted head を bridge する。**
    - ordered bare definition 7 個 `BaseObjectMode -> object` から `ChainObjectMode6 -> ChainObjectMode5`、`ChainObjectMode6` reserve 1 個、`LongLocalObjectModeFourHopAssertedHeadPayloadBoundary: x is ChainObjectMode2` だけを追加した。route は Task 74 の real object expansion producer、Task 200 の formula/checker consumer、byte-for-byte unchanged `BindingFourHopRadix` を合成し、Tasks 221/222、225/227、228 は relation/shorter-distance/terminal sibling のままとする。
    - pairwise-distinct `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4 -> ChainObjectMode3 -> ChainObjectMode2` link を直接検証する。`ChainObjectMode2 -> ChainObjectMode1 -> BaseObjectMode -> object` は cycle-safe terminal-normalization evidence のみに使い、distinct subject/asserted provenance、ordinal 1 / `BindingId(0)`、real expansion 7 個、BaseObjectModeDef-RHS `BuiltinObject` 1 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。
    - 全5,039 nonidentity order、finite definition/reserve/formula/head/provenance/removal/corruption variant、relation link 4 本、全 tail link、terminal/canonical anchor/binding/ordinal/site/range、connected fifth-hop rejection、unrelated-import positive、immutable output、real sidecar、focused sibling、先行 type-assertion owner 53 件との bidirectional isolation を test する。
    - 分類は `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。imported-positive definition、attributed/argument-bearing head、generic reachability/widening/`qua`、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、broader semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。
    - test-first fixture と backlink 6 件は既存 expectation を変更せず active runner 177、392 cases、356 requirements、type-elaboration 224/212、pass/fail 208/184 を計上する。focused、relevant-crate、workspace verification は成功した。checker source/module-layout change は不要であった。
    - Dependencies: Tasks 74, 200, 221, 222, 225, 227, 228。References: Step 5、mizar-test task 10、specs 3, 4, 7, 13, 14, 16。

230. [x] **exact seven-expansion set-terminal five-hop asserted head を bridge する。**
    - ordered bare definition 7 個 `BaseMode -> set` から `ChainMode6 -> ChainMode5`、`ChainMode6` reserve 1 個、`LongLocalModeFiveHopAssertedHeadPayloadBoundary: x is ChainMode1` だけを追加した。Task 74 の real expansion producer、Task 199 の real formula/checker consumer、新規 closed `BindingFiveHopRadix` を合成し、Tasks 224/226/228 と Task 229 は shorter-distance/terminal guard とする。
    - pairwise-distinct `ChainMode6 -> ChainMode5 -> ChainMode4 -> ChainMode3 -> ChainMode2 -> ChainMode1` link を直接検証する。`ChainMode1 -> BaseMode -> set` は cycle-safe terminal-normalization evidence のみに使い、distinct subject/asserted provenance、ordinal 1 / `BindingId(0)`、real expansion 7 個、BaseModeDef-RHS `BuiltinSet` 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。
    - 全5,039 nonidentity order、finite definition/reserve/formula/head/provenance/removal/corruption matrix、relation link 5 本と全 tail/terminal/canonical/binding/source property、connected sixth-hop rejection、unrelated-import positive、immutable output、real sidecar、focused sibling、先行 type-assertion owner 54 件との bidirectional isolation を test する。
    - 分類は `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。object-terminal five-hop、imported-positive definition、attributed/argument-bearing head、generic reachability/widening/`qua`、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、broader semantics は deferred のままとする。Step 5 は active、Steps 6/7 は deferred のままとする。
    - test-first fixture と backlink 6 件は既存 expectation を変更せず active runner 178、393 cases、357 requirements、type-elaboration 225/213、pass/fail 209/184 を計上する。focused、relevant-crate、workspace verification は成功した。checker source/module-layout change は不要であった。
    - Dependencies: Tasks 74, 199, 221, 224, 226, 228, 229。References: Step 5、mizar-test task 10、specs 3, 4, 7, 13, 14, 16。

231. [x] **exact seven-expansion object-terminal five-hop asserted head を bridge する。**
    - completed route は ordered bare `BaseObjectMode -> object` から `ChainObjectMode6 -> ChainObjectMode5` までの definition 7 個、`ChainObjectMode6` reserve 1 個、`LongLocalObjectModeFiveHopAssertedHeadPayloadBoundary: x is ChainObjectMode1` だけを追加した。Task 74 の real object expansion producer、Task 200 の real formula/checker consumer、Task 230 の byte-for-byte unchanged closed `BindingFiveHopRadix` を合成し、Tasks 229/230 を shorter-distance/terminal guard とする。
    - active route は pairwise-distinct `ChainObjectMode6 -> ChainObjectMode5 -> ChainObjectMode4 -> ChainObjectMode3 -> ChainObjectMode2 -> ChainObjectMode1` link を直接検証する。`ChainObjectMode1 -> BaseObjectMode -> object` は cycle-safe terminal-normalization evidence のみに使い、distinct subject/asserted provenance、ordinal 1 / `BindingId(0)`、real expansion 7 個、BaseObjectModeDef-RHS `BuiltinObject` 1 個、inferred variable 1 個、object/set coercion なしの constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。
    - completed finite test contract は全5,039 nonidentity order、finite definition/reserve/formula/head/provenance/removal/corruption matrix、link 5 本と全 tail/terminal/canonical/binding/source property、connected sixth-hop rejection、unrelated-import positive、immutable output、real sidecar、focused sibling、先行 type-assertion owner 55 件との bidirectional isolation を検証する。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。imported-positive definition、attributed/argument-bearing head、generic reachability/widening/`qua`、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、broader semantics は deferred。Step 5 は active、Steps 6/7 は deferred のままとする。
    - fixture と backlink 6 件は active runner 179、394 cases、358 requirements、type-elaboration 226/214、pass/fail 210/184 を既存 expectation の変更なしで構成する。focused、relevant-crate、workspace verification は成功した。checker source/module-layout change は不要であった。
    - 依存: Tasks 74、200、229、230。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

233. [x] **exact parenthesized builtin-object reserved-variable equality を bridge する。**
    - `reserve x for object; theorem ParenthesizedReservedObjectVariableEqualityPayloadBoundary: (x) = x;` だけを追加し、Task 223 の real `ParenthesizedTerm` producer と Task 188 の real object reserve/BindingEnv/equality consumer を合成した。
    - private parenthesized extraction/output assertion path だけを exact static config で parameterize した。Task 223 は object reserve を、Task 188 は parenthesized operand を引き続き reject し、新 public route は両者の exact intersection だけを所有する。
    - 独立 wrapper/inner/right site/range、ordinal 1/2 の `BindingId(0)` lookup、canonical `BuiltinObject` 1 個、inferred variable 2 個、ordered expected constraint 2 個、object/set coercion と独立 wrapper type/value のない checked equality 1 個を保持する。
    - finite structural/provenance/lookup/type/matched-output corruption matrix、immutable output、real frontend/resolver sidecar、先行 binary-formula owner 53 件との bidirectional isolation を test する。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。arbitrary parenthesis/operand/precedence、formula grouping、closure/order、truth/fact、acceptance、proof/CoreIr/ControlFlowIr/VC、child graph、broader semantics は deferred。Step 5 は active、Steps 6/7 は deferred のままとする。
    - test-first fixture と backlink 6 件は既存 expectation を変更せず active runner 180、395 cases、359 requirements、type-elaboration 227/215、pass/fail 211/184 を計上する。checker source/module-layout change は不要であった。
    - 依存: Tasks 9、119、188、223。参照: Step 5、mizar-test task 10、specs 3、4、13、14、16。

234. [x] **Exact seven-expansion set-terminal full-distance six-hop asserted head を bridge する。**
    - ordered bare definition 7 個 `BaseMode -> set` から `ChainMode6 -> ChainMode5`、`ChainMode6` reserve 1 個、`LongLocalModeSixHopAssertedHeadPayloadBoundary: x is BaseMode` だけを追加し、Task 74 の real expansion producer と Task 199 の real formula/checker consumer を合成する。
    - `BaseMode` までの pairwise-distinct link 6 本すべてを直接検証する closed `BindingSixHopRadix` relation を追加し、`BaseMode -> set` は terminal normalization のみに使う。distinct subject/asserted provenance、ordinal 1 / `BindingId(0)`、real expansion 7 個、BaseModeDef-RHS `BuiltinSet` 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を保持する。
    - 全5,039 nonidentity order、finite definition/reserve/formula/head/provenance/removal/corruption matrix、immutable output、real sidecar、focused sibling、先行 type-assertion owner 56 件との bidirectional isolation を検証する。
    - 分類は `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。object-terminal six-hop、imported-positive/attributed/argument-bearing head、generic reachability/widening/`qua`、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、child graph、broader semantics は deferred。Step 5 は active、Steps 6/7 は deferred。
    - test-first fixture と backlink 6 件は既存 expectation を変更せず active runner 181、396 cases、360 requirements、type-elaboration 228/216、pass/fail 212/184 を計上する。checker source/module-layout change は不要であった。
    - 依存: Tasks 74、199、230、231。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

236. [x] **Exact seven-expansion object-terminal full-distance six-hop asserted head を bridge する。**
    - ordered bare definition 7 個 `BaseObjectMode -> object` から `ChainObjectMode6 -> ChainObjectMode5`、`ChainObjectMode6` reserve 1 個、`LongLocalObjectModeSixHopAssertedHeadPayloadBoundary: x is BaseObjectMode` だけを追加する。Task 74 の real object expansion producer と Task 200 の real formula/checker consumer を合成する。
    - closed `BindingSixHopRadix` を byte-for-byte unchanged で再利用し、`BaseObjectMode` までの pairwise-distinct link 6 本すべてを直接検証し、`BaseObjectMode -> object` は terminal normalization のみに使う。distinct subject/asserted provenance、ordinal 1 / `BindingId(0)`、real expansion 7 個、BaseObjectModeDef-RHS `BuiltinObject` 1 個、inferred variable 1 個、constraint/fact/candidate/diagnostic/deferred 0 個の checked assertion 1 個を object/set coercion なしで保持する。
    - 全5,039 nonidentity order、finite definition/reserve/formula/head/provenance/removal/corruption matrix、immutable output、real sidecar、focused sibling、先行 type-assertion owner 57 件との bidirectional isolation を検証する。
    - 分類: `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。imported-positive/attributed/argument-bearing head、generic reachability/widening/`qua`、acceptance、truth/fact、proof/CoreIr/ControlFlowIr/VC、child graph、broader semantics は deferred。Step 5 は active、Steps 6/7 は deferred のままとする。
    - test-first fixture と backlink 6 件は既存 expectation を変更せず active runner 182、397 cases、361 requirements、type-elaboration 229/217、pass/fail 213/184 を計上する。checker source/module-layout change は不要であった。
    - 依存: Tasks 74、200、231、234。参照: Step 5、mizar-test task 10、specs 3、4、7、13、14、16。

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

## Task 241 Active Addendum

- [x] Task 223 の exact real single-left `ParenthesizedTerm` producer と Task
  121 の real builtin-set inequality consumer を `(x) <> x` だけに合成する。
  独立した wrapper/inner/right provenance、ordinal 1/2 の `BindingId(0)`
  lookup、canonical `BuiltinSet` 1 個、inferred term 2 個、ordered expected
  constraint 2 個、fact/candidate/diagnostic/deferred 0 個の checked inequality
  1 個を保持する。private shared helper のみ binary-formula generic とし、
  closed equality/object wrapper を維持する。
- [x] exact/near-miss/corruption/immutable-output/active-sidecar/focused
  equality regression と先行 owner 54 件との bidirectional isolation で route
  を保護する。active runner/count は 183、398/362、type-elaboration 230/218、
  pass/fail 214/184。
- 分類は `test_gap`、narrow `source_drift`、`design_drift`、`spec_gap` なし。
  parenthesized membership、imported/other parenthesized variant、desugaring/
  truth、acceptance、proof/CoreIr/ControlFlowIr/VC、downstream payload は Task
  241 の credit 外。Step 5 は active、Steps 6/7 は deferred。checker
  source/API/module-layout update は不要。

## Task 242 Active Addendum

- [x] Task 233 の exact real builtin-object single-left `ParenthesizedTerm`
  producer と Task 190 の real object inequality consumer を `(x) <> x` だけに
  合成する。独立した wrapper/inner/right provenance、ordinal 1/2 の
  `BindingId(0)` lookup、canonical `BuiltinObject` 1 個、inferred term 2 個、
  type entry 6 個、ordered expected constraint 2 個、fact/candidate/diagnostic/
  deferred 0 個の checked inequality 1 個を object/set coercion と独立 wrapper
  semantic node なしで保持する。
- [x] unique key/config と closed wrapper/route だけを Task 233 直後へ追加し、
  先行 Task 188 route、shared private binary-formula helper、Tasks 190/223/233/
  241 ownership を変更しない。先行 owner 55 件との bidirectional isolation、
  exact/near-miss/provenance/corruption、immutable output、focused regression、
  real sidecar で保護する。
- [x] runner 184、399/363、type-elaboration 231/219、pass/fail 215/184 を同期
  する。parenthesized membership と active imported provenance は Task 242
  credit 外。未成立 imported expansion/evidence/signature payload と proof/
  CoreIr/ControlFlowIr/VC は deferred。Step 5 は active、Steps 6/7 は deferred。
  checker source/API/module-layout update は不要。

## Task 243 Active Addendum

- [x] Task 223 の exact real builtin-set single-left `ParenthesizedTerm`
  producer と Task 120 の real membership consumer を `(x) in x` だけに合成
  する。独立 wrapper/inner/right provenance、ordinal 1/2 の `BindingId(0)`
  lookup、canonical `BuiltinSet` 1 個、inferred term 2 個、type entry 5 個、
  left expected input 0 個、変更しない direct-right producer の唯一の
  expected-set constraint、独立 wrapper semantic node のない checked
  membership 1 個を保持する。
- [x] unique key/config と closed wrapper/route だけを Task 241 直後へ追加し、
  先行 Task 188 route と Tasks 120/223/233/241/242 ownership を維持する。
  先行 owner 56 件との bidirectional isolation、unexpected-left/wrong-right/
  missing-right expected input を含む exact/near-miss/provenance/corruption、
  immutable output、focused regression、real sidecar で保護する。
- [x] runner 185、400/364、type-elaboration 232/220、pass/fail 216/184 を同期
  する。extraction gap の解除はこの exact source だけ。object-left/set-right
  parenthesized membership と active imported provenance は Task 243 credit
  外。未成立 imported expansion/evidence/signature payload と proof/CoreIr/
  ControlFlowIr/VC は deferred。Step 5 は active、Steps 6/7 は deferred。
  checker source/API/module-layout update は不要。

## Task 244 Active Addendum

- [x] Chapters 03/04/13/14/16 と既存 Task 125 direct membership intent から、
  exact two-reserve source `reserve x for object; reserve y for set; theorem
  ParenthesizedHeterogeneousReserveMembershipPayloadBoundary: (x) in y;` を追加。
- [x] private parenthesized binary extractor を finite config に限定して一般化。
  exact reserve count、ordered spelling/type arrays、operand binding indices、
  shared/distinct written type-range policy を検査し、従来5 config は不変。
- [x] ordinal 2/3、`BindingId(0/1)`、別々の written object/set identity、
  inferred term 2件、type entry 5件、left expected なし、right expected-set
  constraint 1件、fact/candidate/diagnostic/deferred/coercion/wrapper semantics
  なしの checked membership を保持。
- [x] exact/near-miss/corruption/provenance、immutable output、既存 binary owner
  57件、Tasks 120/125/223/233/241/242/243、real imported-mode-gap diagnostic
  fixture、real frontend/resolver sidecar を guard。
- [x] shared backlink 5件 + dedicated trace 1件と count を同期: active 186、
  cases/requirements 401/365、type 233/221、pass/fail 217/184。
- [x] その他 parenthesized shape と imported-positive provenance は Task 244
  credit 外。未成立 imported expansion/evidence/signature payload と proof/
  CoreIr/ControlFlowIr/VC は deferred。Step 5 は active、Steps 6/7 は deferred。
  checker source/API/module-layout update は不要。

## Task 245 Active Addendum

- [x] Chapters 04/13/14/16 から exact `reserve x for set; theorem
  RightParenthesizedReservedVariableMembershipPayloadBoundary: x in (x);` を追加。
- [x] 従来6 config を explicit `Left` とし、Task-245-only key/config/role と
  private `Right` side を追加。ordinal 1/2 の双方 `BindingId(0)`、canonical
  `BuiltinSet` 1件、inferred term 2件、type entry 5件、right-inner-owned sole
  expected-set constraint を保持。
- [x] side/config/range/expected corruption、Task-243 cross-route、immutable/
  module boundary、既存 owner 58件の双方向、Left route 6件、real sidecar を
  guard。
- [x] runner 187、plan 402/366、type 234/222、pass/fail 218/184、shared 4 +
  dedicated 1 backlink を同期。その他 shape は credit 外、未成立 imported/
  proof/downstream payload は deferred。Step 5 active、Steps 6/7 deferred。
  checker source/API/module-layout update は不要。

## Task 246 Active Addendum

- [x] exact 3-definition set-terminal chain、`reserve z for
  OuterTwoEdgeModeEquality`、`(z) = z` だけを Task-246-only key/config/role と
  transparent Left wrapper で追加。
- [x] mode-definition node は nonempty-mode parenthesized config だけ許可し、
  旧 empty-mode route、expansion 3件、raw Outer input 4件、ordinal 1/2 の
  `BindingId(0)`、entry 6件、constraint 2件、wrapper output なしの clean
  equality を保持。
- [x] 全5 definition order、finite corruption、Tasks 134/223 cross-rejection、
  immutable/module、既存 owner 59件の双方向、real sidecar を guard。
- [x] runner 188、plan 403/367、type 235/223、pass/fail 219/184、shared 5 +
  dedicated 1 を同期。Step 5 active、Steps 6/7 deferred、broader/downstream
  semantics は credit 外。

## Tasks 266-268 Final Checker Handoff Queue

- [x] **Task 266: exact Task-180 statement-semantic projection。** checker-owned
  `ResolvedTypedAst`へsyntax-free final projectionを追加し、resolver theorem owner
  1件を既存checked `FormulaKind::Contradiction` 1件へlinkする。owner/formula
  identity、state、source range、provenanceを保存し、missing/duplicate/reordered/
  recovered/mismatched rowをrejectする。`mizar-test`はAST extractionとexact active-
  runner assertionを所有する。既存source/expectationは不変で再利用する。truth/
  fact publication、theorem acceptance、proof/terminal-goal/Core/VC payload、broader
  formula、runner-stage promotionは禁止する。依存: mizar-test Task 265、checker
  Task 180。仕様: 14/16。
- [x] **Task 267: omitted-justification proof-handoff contract。** docs-onlyで、
  written justificationのないordinary theoremに対するchecker-owned pending-auto-
  proof status、proof skeleton、explicit terminal-goal payload、source/provenance
  link、malformed/missing behavior、exact core mappingを定義する。code実装、coreの
  raw syntax推論、proof search、omissionとacceptanceの同一視は禁止する。依存:
  Task 266。仕様: 15/16、architecture 06。
  完了: explicit `Unmodified`/`Omitted` intentはdistinct
  `PendingAutomaticProof` 1件、direct terminal node 1件、`proof/0` terminal row
  1件を生成し、future exact core mappingはatomic/non-acceptingである。
- [x] **Task 268: accepted Task-267 producerを実装する。** exact Task-180 final
  handoffだけを拡張し、missing/duplicate/reordered/corrupt/owner-formula-proof mismatch
  をfail closedにする。3 proof tableのdeterministic nonempty debug renderingと
  empty renderingのbyte stabilityもcoverする。theorem acceptance、discharge、
  Core/VC generation、broader
  proof form、expectation change、Steps 6/7は禁止する。依存: Task 267。
  independently validated Public/Exported resolver factを
  `CheckedStatementOwner`に保持し、proof-intent rowとcross-checkし、authenticated
  owner/row corruptionを独立にtestする。
  完了: exact all-or-none producer、authenticated-owner validation、private
  output postvalidation、corruption matrix、deterministic nonempty rendering、
  captured byte-identical empty renderingを実装した。次はCore Task 31で、
  acceptance、Core/VC、broader proof、Steps 6/7はdeferredのままである。

## Task 247 STEP 5 Payload-Family Decomposition

- [x] 残るAST-wide declaration、attribute、term、formula、proof-skeleton、
  registration/activation/trace、overload、Task-49 payload familyをcanonical spec、
  既存`.miz`、trace、expectation、checker APIに対してinventoryする。prepared
  mizar-test Task-10 consumerとexplicit forbidden scopeを持つbounded producer task
  を作る。docs/traceability-onlyであり、source、fixture、expectation、trace status、
  coverage creditを変更しない。Parser Tasks 47-48とresolver Task 31は独立
  prerequisiteのまま。Task 49は自身の全prerequisite成立までdependency-gated。
  Task 266へのdependencyはない。そのinventoryは再利用できるが、contradiction
  sliceをgeneral扱いしない。
  accepted producer-task graphはcore Task 32のinput authorityとなる。
  完了: [payload_family_decomposition.md](./payload_family_decomposition.md)は
  全remaining familyをchecker Tasks 248-264/269-279、mizar-test Task-10
  increments `MT10-FS`/`MT10-AS`、既存Task 49、またはexplicit external gateへ
  割り当てた。exact 24-fixture reconciliation setを固定し、resolver Task 31が
  same-return memberを`declaration_symbol`でsole activation、Task 49が他23件を
  activateして24件全体をreconcile/deduplicateする。既にactiveなdifferent-return
  controlも維持する。Task 274とexternal Gate S1はcanonical
  upstream owner未命名のためblocked。trace status/tests/
  coverage credit/source/fixture/expectation/Steps 6/7は不変。Core Task 32は
  accepted graphを自身のdocs-only decompositionへ利用できる。

## Tasks 248-264/269-279 STEP 5 ソースペイロードproducer queue

完全なauthority slice、dependency、consumer、gate、negative scope、exit
criteriaは[payload_family_decomposition.md](./payload_family_decomposition.md)
をcanonicalとする。以下の各unchecked rowは将来の1 nonempty logical taskかつ
1 commitである。

- [x] **Task 248:** source item/declaration site/binding-context producer。
  syntax-free producer、immutable `TypedAst`/`ResolvedTypedAst` handoff、exact
  reserve/definition-parameter shadowing consumer、recovery boundary、corruption
  matrix、bounded trace rowは完了した。term-use selection、type result、RHS/
  formula/proof semantics、accepted factへのcreditは与えない。
- [x] **Task 249:** type head/application/argument producer。public
  syntax-free flat producer、input-only legacy binding-environment seam、
  immutable `TypedAst`/clone-only `ResolvedTypedAst` handoff、exact broad
  10/13/6 consumer、Task-248 2/2/0 co-consumer、corruption/determinism matrix、
  runner-only readiness detail、bounded trace row、paired docs、count/hash
  verificationはcomplete。normalization、evidence、term/`qua` selection、
  accepted fact/declaration/proof、downstream IRはpublishしない。
- [x] **Task 250:** attribute-chain/qualification/provenance producer。public
  syntax-free five-table producer、immutable `TypedAst`/clone-only
  `ResolvedTypedAst` handoff、exact real consumer 4件とaggregate
  4/4/0・4/4/1/1/1 oracle、written polarity/qualifier/group/actual preservation、
  synthetic prefix extractor、corruption/determinism matrix、bounded
  trace/expectation progression、paired documentationはcomplete。semantic
  attribute instance、arity/admissibility、term result、evidence request/result、
  accepted fact/declaration/proof、downstream IRはpublishしない。
- [x] **Task 251:** evidence-query request/dependency-fact-reference producer。
  paired crate-plan contractを実装し、exact Task-249 broad route +
  Task-84/85がdense syntax-free request/response-reference tableを通してmissing request
  10件（mode-expansion 5 / structure-inhabitation 3 / attributed 2）をemitする。
  `TypedAst`がownし、
  `ResolvedTypedAst`がclone-preserveする。production-path testはsupplied inputを
  accepted evidenceと扱わずrequested/missing/rejected/suppliedを区別する。
  checker testはexact association/corruption/cardinality/atomicityをcoverし、
  evidence resultを捏造しない。
- [x] **Task 252:** primary term producer。paired crate-plan contractをpublic
  3-table syntax-free contract、exact 3-route 7/4/2 real consumer oracle、
  transparent-parenthesis rule、numeric-request-only boundary、synthetic
  constant/`it` dependency coverage、final ownership、test、bounded covered
  trace rowまで実装した。real `it` ownerと
  local-constant binding productionはTasks 260/264/269に残す。post-freeze
  contract correctionは各reference ordinalを先行して完了したbinding rowから
  deriveし、reachableな`Ambiguous` rejectionのためduplicate-priority binding
  groupを保持し、このproducerで`Resolver`がstructurally unreachableと記録する。
- [x] **Task 253:** functor/inline-functor application producer。public
  5-table `source_application` contractをexact imported caseと同じdefinition
  blockの後続definiens consumer、aggregate application/wrapper/candidate/
  argument/request 2/1/2/3/4 oracle、Task-252 term/reference/numeric-request
  3/1/2 slice、Task-248 `DefinitionParameter` shadow-handoff再利用、
  transparent cross-family origin ownership、individual candidate-reference
  boundary、synthetic schema、corruption matrix、final ownershipまで実装した。
  inline identity/formal/capture/substitutionはTask 270、direct template
  transportはTask 277、ordinary/template candidate collection/viability/
  winnerはTask 278に残り、Task 253はTask-252 primary termを複製しない。
- [x] **Task 254:** structure constructor/selector/update term producer。
  public seven-table syntax-free handoff、Task-248 context reuse、
  Task-252/253/254 ownership/fingerprint matrix、exact
  5/0/3/9/2/10/26 + 8/0/8 consumer、arena-key class 5個、bounded
  fixture/trace rowとreciprocal backlink、corruption/determinism/final
  ownership coverage、measured 413/377・243/231 oracleはcompleteである。
  structure/member/view semanticsはTask 263 ownershipに残す。
- [x] **Task 255:** set/comprehension/choice/`qua` term producer。public
  6-table syntax-free `source_set_term` transaction、exact local-definition
  consumer、4/0/1/3/4/7 + Task-252 4/0/4 oracle、one-way
  Task-252/253/254/255 child ownership、conditional fingerprint、one-shot
  final handoff、bounded fixture/trace row、frozen producer/extractor/
  corruption/install-order matrixはcompleteである。comprehension binder
  identity/captureはTask 257、condition付きcomprehension formula ownershipは
  Tasks 256-257、semantic result/sethood/nonemptiness/widening decisionはdeferred
  のままとする。
- [x] **Task 256:** atomic formula producer。public 8-table syntax-free
  transaction、private exact 8-route consumer、exact
  `8/0/1/1/1/2/13/11` aggregate、Task-252 `16/0/16`、Task-253
  `1/1/1/2/2`、Task-255 `2/0/0/0/4/2`、conditional
  Task-253/254/255 fingerprint、unresolved input request 11件、final immutable
  handoff、reciprocal trace increment、review済みreal/synthetic/exclusion/
  corruption/install matrixは完了した。既存semantic routeと全outcome/detail
  fieldは不変である。
- [ ] **Task 257:** composite/quantified formula、binder、predicate-chain、
  conditioned-comprehension umbrella。
  - [x] **Task 257A:** exact implication/universal/negation/contradiction treeと
    explicit unused binder 1件。public seven-table transaction、`2/1/4`
    binding extension、private exact consumer、final ownership、reciprocal
    trace row、review済みreal/synthetic/corruption/isolation matrixは完了した。
    preflight-corrected real rangeをcanonical `.miz`/semantic detail vector
    不変のまま保持する。
  - [ ] **Task 257B:** broader connective/quantifier shape、implicit binder、
    bound use/capture。
    - [x] **Task 257B1:** explicit universal-to-atomic compositionとbinderが
      selectするbound use 2件。exact 79-byte pass consumer、第2 exact Task-257
      composite profile、Task-252/256 dependency、`1/2` formula-composition
      transaction、final ownership、bounded trace rowはsemantic truth/theorem
      acceptanceなしでcomplete。
    - [x] **Task 257B2:** Task 257B1後のexact conjunction/disjunction/`iff`/
      repetition/executable formula grouping transport。connective
      truth/theorem acceptanceは含めない。
    - [x] **Task 257B3:** Tasks 257B1/B2後のexact existential/restricted/
      nested quantification、implicit reserved-binder shadowing、scoped use
      6件。semantic truth/closure/capture result/theorem creditなしでfrozen
      source-to-final-handoff transportを実装済み。
  - [ ] **Task 257C:** separately frozen Task-256/255 extension後の
    predicate-chain/conditioned-comprehension composition。
    - [x] **Task 257C1:** Task 256をpredicate-chain segment、polarity token、
      shared-boundary transportで拡張する。
      - [x] syntax-free 9-table contract、exact consumer、test、trace
        projection、ownership、semantic deferralをfreeze。
      - [x] fresh preflight後、別logical task/commitでfrozen contractを実装。
    - [x] Task 255のcondition-bearing comprehension transportを別の
      documentation/implementation pairとしてfreeze/実装する。
    - [ ] predicate-chain/conditioned-comprehension formula compositionは
      後続の別Task-257C sliceだけで追加する。
      - [x] **Task 257C2 prerequisite:** exact independent
        condition-to-atomic-formula associationをsemanticsなしでfreeze。
      - [x] **Task 257C2 implementation:** separate Task-256C1とfresh
        preflight後、frozen condition-formula associationだけをimplement。
      - [x] **Task 257C3 prerequisite:** Task 257C2後、predicate-chain
        conjunction/segment-negation compositionをseparately freeze。
      - [x] **Task 257C3 implementation:** fresh post-documentation
        preflight後にfrozen predicate-chain compositionだけを実装。
- [ ] **Task 258:** general theorem-owner/statement-semantic/assumption/
  visibility-scoped input-fact producer。accepted theorem factをpublishしない。
  - [x] **Task 258A:** exact reserved-variable equality theorem owner、
    statement shell、implicit reserved-type-guard input、unverified
    proposition candidate。
    - [x] exact 81-byte future `MT10-FS` source、resolver owner/label
      provenance、Task-252/256 lower profile、syntax-free `1/1/1/1/1`
      transaction、typed/resolved ownership、empty-semantic boundary、owned
      BindingEnv/fingerprint、production/named test-only seamでのTask-248
      exclusion、tests、trace non-activation、exit criteriaをfreeze。
    - [x] dedicated documentation commitとfresh parser/resolver/lower-API/
      count/hash preflight後にfrozen Task-258A transportだけを実装。
  - [ ] **Task 258B:** explicit assumption/conclusion/witness、local
    label/citation input、composite theorem root、nested statement context、
    broader visibility。Tasks 269-272はproof-local binding/closure/
    reconsider intent/proof skeleton/justification semanticsを保持する。
    - [x] **Task 258B1 prerequisite:** exact 139-byte nested
      equality-statement source、theorem owner 1件、statement/context/guard/
      candidate各4件、proof binding context 3件、local proof-step label 1件、
      resolved citation 1件、replayable resolver projection/reference/result、
      sole keyed node 68を持つtwo-pass 77-node/root-76 resolver AST、
      Task-252/256 dependency、typed/resolved ownership、test-only syntax
      dev-dependency、empty-semantic boundary、tests、non-activationをfreeze。
    - [x] **Task 258B1 implementation:** dedicated documentation commitと
      fresh parser/resolver/lower-API/count/hash preflight後にfrozen nested
      conclusion/local-label transportだけを実装した。checker 4本とrunner 5本で
      bounded `source_drift`/`test_gap`を閉じ、semantic/corpus activation
      gateはすべてdeferredのままとした。
    - [x] **Task 258B2 prerequisite:** exact 113-byte unlabeled single-
      assumption source、55-node/root-54 parser shape、theorem/assumption/
      conclusion `1/3/3/3/3` profile、Task-48 `2/1/0`、Task-252
      `6/6/0`、Task-256 `3/0/0/0/0/0/0/6/6`、base-only typed/final
      ownership、empty-semantic boundary、tests、non-activationをfreeze。
    - [x] **Task 258B2 implementation:** dedicated documentation commitと
      fresh parser/resolver/lower-API/count/hash preflight後にfrozen
      single-assumption transportだけをimplementした。checker 4本/runner
      5本がbounded `source_drift`/`test_gap`をcloseし、semantic/corpus
      activationは追加していない。
    - [x] **Task 258B3 prerequisite:** exact 104-byte unnamed witness
      source、49-node/root-48 parser identity、theorem-only resolver
      provenance、Task-48 `2/1/0`、Task-252 `5/5/0`、Task-256
      `2/0/0/0/0/0/0/4/4`、formula-only base `1/2/2/2/2`、one-row
      witness companion、paired typed/final ownership、tests、
      non-activationをfreeze。
    - [x] **Task 258B3 implementation:** documentation commit/fresh
      preflight後にfrozen paired witness transportだけをimplementした。
      checker 4本/runner 5本がbounded `source_drift`/`test_gap`をcloseし、
      semantics/corpus activationは追加していない。
    - [ ] **Tasks 258B3N/M:** B3後/B4前にnamed-witness transportと
      multiple/other witness-term transportをseparately freezeする。
      abbreviation/substitution/type-obligation/goal semanticsをinferしない。
    - [ ] **Tasks 258B4-B5:** composite theorem rootとbroader
      imported/outer/inner visibility profileをseparately freezeし、Tasks
      269-272 semanticsを吸収しない。
- [x] **Task 259:** predicate-definition/initial-obligation intake producer。
- [x] **Task 260:** functor-definition/initial-obligation intake producer。
- [x] **Task 261:** attribute-definition producer。
- [x] **Task 262:** mode-definition producer。
- [x] **Task 263:** structure/inheritance/constructor-definition producer。
- [x] **Task 264:** property-implementation producer。parser Task 48に依存。
- [ ] **Task 269:** proof-local declaration/binding producer。
- [ ] **Task 270:** inline-definition closure/capture/substitution-request producer。
- [ ] **Task 271:** `reconsider` intent/coercion/evidence-request producer。
  parser Task 47に依存。
- [ ] **Task 272:** non-Task-180 proof-skeleton/justification producer。
- [ ] **Task 273:** registration-item/correctness/initial-obligation intake producer。
- [ ] **Task 274 (blocked-reserved):** accepted verifier/artifact-status import/
  activation adapter。canonical authorityがupstream owner/schema/authentication
  rule/testsを命名するまで実行不能。
- [ ] **Task 275:** source-derived cluster-closure trace producer。
- [ ] **Task 276:** source-derived reduction/normalization trace producer。
- [ ] **Task 277:** direct template role/actual/guard producer。missing
  scheme/theorem roleはexecutable task外のexternal Gate S1で、Task 49はS1にも
  gateされる。
- [ ] **Task 278:** ordinary/template overload input-to-selection producer。
- [ ] **Task 279:** redefinition/notation target/coherence/refinement producer。
  dependency cycleなしにTask 278 ordinary-root resultをconsumeする。

各taskはfamilyを適用可能な`TypedAst`/`ResolvedTypedAst` tableまでtransactionally
projectし、実`mizar-test` Task-10 caseがconsumeする。未消費DTO、placeholder
runner、docs-only implementation commitはproducer taskを満たさない。

## Task 257B2 frozen-contract addendum

- [x] fixed/repeated conjunction/disjunction、`iff`、executable grouping 6件を
  含むexact 166-byte explicit-universal sourceをfreezeする。
- [x] parser range/token、第3 `8/6/1/1/1/7/9` composite、Task-252
  `16/0/16`、Task-256 `8/0/0/0/0/0/16/16`、composition `8/0`をfreeze。
- [x] new composite kind、same-family/atomic-edge role、real wrapper validation、
  profile partition、install/final ownership、corruption/isolation test、
  trace impact、semantic deferralをfreeze。
- [x] prerequisiteをdocumentation-onlyとし415/381、247/235、225/190、
  active 101/5/194/1、library 306/338 tests、production 29 paths /
  31,374 linesを不変にする。
- [x] fresh parser/resolver/baseline preflight後、別logical taskでこのfrozen
  Task 257B2だけを実装する。
- [x] exact `16/0/16`、`8/0/0/0/0/0/16/16`、`8/6/1/1/1/7/9`、
  `8/0`、fail-closed tests、final ownership、corpus `416/382`、semantic
  deferralをverifyする。
- [x] production/test intentを変える前に次のdependency-ordered Task 257B3を
  freezeする。

## Task 257B3 frozen-contract addendum

- [x] exact 138-byte reserve/restricted-universal/existential/
  nested-implicit-universal sourceとfinal-LF SHA-256をfreeze。
- [x] Task-48 one-reserve base、明示的Task-248 exclusion、empty captureの
  4 contexts/bindings、reserve-default provenance、inner-`r` shadowをfreeze。
- [x] exact Task-252 `6/6/0`、Task-256
  `3/0/0/0/0/0/6/6`、Task-257B3 `3/0/1/3/3/2/6`、
  composition `3/6` profile、row order/association、`body_edge`
  compatibilityをfreeze。
- [x] parser/resolver preflight fact、complete corruption/isolation/install
  test、sidecar/trace projection 1件、audit impact、semantic deferralをfreeze。
- [x] prerequisiteをdocumentation-onlyでcorpus `416/382`、type `248/236`、
  pass/fail `226/190`、active `101/5/195/1`、library `312/343`、
  29 paths / 32,064 linesに保つ。
- [x] fresh preflight後、別logical task/commitでこのfrozen B3 sliceだけを
  実装する。
- [x] exact `4/4/0`、`6/6/0`、`3/0/0/0/0/0/6/6`、
  `3/0/1/3/3/2/6`、`3/6` profile、fail-closed install/final ownership、
  corpus `417/383`、全semantic deferralをverifyする。

## Task 257C1 frozen-contract addendum

- [x] exact 107-byte imported-predicate chain/final-LF hash、parser/resolver
  range、同一imported symbolを指す2 head、exact private selector/subtree
  exclusionをfreeze。
- [x] Task-252 `3/0/3`と拡張Task-256
  `1/0/2/2/2/0/0/3/2`（segment row 2件、`does not` token provenance、
  shared `PredicateChainBoundary` edge 1件）をfreeze。
- [x] public segment schema、legacy empty-segment compatibility、validation/
  debug/final ownership、complete corruption/isolation tests、exact
  sidecar/trace projection 1件、semantic deferralをfreeze。
- [x] prerequisiteをdocumentation-onlyでcorpus `417/383`、type `249/237`、
  pass/fail `227/190`、active `101/5/196/1`、libraries `319/349`、
  29 paths / 32,809 linesに保つ。
- [x] fresh preflight後、このfrozen C1 sliceだけを実装し、実測を
  corpus `418/384`、type `250/238`、pass/fail `228/190`、active type
  `197`とする。
- [x] library `322/353`、exact source/near-miss/corruption matrix、
  shared-boundary ownership、covered trace row、全semantic deferralをverify。
- [x] 別Task-255 condition-bearing-comprehension transport prerequisiteを
  次にfreezeする。

## Checker Task 255C1 frozen-contract ledger

- [x] valid 191-byte source/hash、exact parser range、imported `++`
  provenance、loaded-source/final-LF guardをfreeze。
- [x] seven-table public API/debug contract、colon/direct condition-wrapper
  arena anchor、Task-252
  `4/0/4`、Task-253 `1/0/1/2/2`、Task-255
  `1/0/1/1/1/1/2`をfreeze。
- [x] condition-subtree lower-family exclusion、reusable private Task-253
  seam、16 compatibility literal、atomic install/clone、testsをfreeze。
- [x] missing contractを`design_drift`、implementationをbounded
  `source_drift`/`test_gap`、origin driftをreport-only
  `repo_metadata_conflict`にclassifyし、blocking `spec_gap`なし。
- [x] production、fixture、sidecar、trace metadata/count、executable
  coverage、count、hashをTask-257C1 baselineで保持。
- [x] fresh parser/resolver/APIとcount/hash preflight後、Task 255C1をseparate
  logical taskとしてimplement。exact routeはfrozen
  `4/0/4 -> 1/0/1/2/2 -> 1/0/1/1/1/1/2` chain、recursive condition
  boundary、fixture/sidecar/trace row各1件、`419/385`、`251/239`、
  `228/191`、active `101/5/198/1`、checker/runner tests `326/357`を
  semantic promotionなしでpublishする。

## Checker Task 257C2 frozen-contract ledger

- [x] unchanged 191-byte source/hash、direct condition-wrapperからinner
  equalityへのrelation、exact range、built-in equality identity、imported
  `++` provenance exclusionをfreeze。
- [x] Task-252 `4/0/4`、Task-253 `1/0/1/2/2`、Task-255
  `1/0/1/1/1/1/2`、Task-256 `1/0/0/0/0/0/0/2/2`、condition-formula
  association 1件をfreeze。
- [x] Task-257B APIを変更しないdedicated immutable handoff/producer/table、
  dependency fingerprint 4件、deterministic debug、typed/resolved ownership、
  validation、rollback、compatibility boundaryをfreeze。
- [x] existing fixture/sidecar reuse、future trace row 1件、exact test
  matrix、unchanged 419-case/pass-fail/active count、projected plan
  `419/386`/type `252/240`、semantic deferral、audit impactをfreeze。
- [x] missing contractを`design_drift`、implementationをbounded
  `source_drift`/`test_gap`、committed Task-256 condition-container
  rejectionをseparate authority-backed `source_drift`、stale Task-255C1
  umbrella checkboxをresolved `design_drift`、origin driftをreport-only
  `repo_metadata_conflict`にclassify。
- [x] separate Task-256C1 condition-container compatibility prerequisiteを
  freeze/reviewし、documentation-only commitを作る。
- [x] Task-256C1を両install order/strict arbitrary-overlap rejection込みで
  own commitにimplement/verify。
- [ ] Task-256C1後、fresh parser/resolver/API、both-install-order、
  count/test-list/production/CLI-hash preflightを行い、Task 257C2だけを
  separate logical task/commitでimplement。

## Checker Task 256C1 frozen-contract ledger

- [x] Chapter-13/14 authority、unchanged 191-byte source/hash、exact
  `139..184` condition container/`177..182` equality、checker-local/future
  Task-257C2 consumerをfreeze。
- [x] sole admitted relationをfreeze。Task-255 comprehension condition 0が
  distinct Task-256 equality 0をdirect containし、equal range/spelling、
  normal recovery、equality/owner-term context、exact profileを持ちcondition
  contextをfabricateしない。
- [x] public API、全table/ID/error/debug byte、
  `set_term_fingerprint() == None`、disjoint/formula-contains-set case、
  strict arbitrary/substituted/copied/stale/wrong-context/non-direct overlap
  rejectionを保持。
- [x] `TypedAst`両installation order、atomic rollback/replay、checker tests
  exact 3件、pair前は個別validでpair時だけexact
  `SetTermDependencyMismatch`となるnear miss、projected libraries
  `329/357`、runner/trace変更ゼロ、全semantic deferralをfreeze。
- [x] frozen pre-implementation two-order rejectionをauthority-backed
  `source_drift`、missing contractをresolved `design_drift`、testsを
  `test_gap`、origin divergenceを
  report-only `repo_metadata_conflict`と分類。blocking `spec_gap`なし。
- [x] prerequisiteをdocumentation-only plan `419/385`、type `251/239`、
  pass/fail `228/191`、active `101/5/198/1`、libraries `326/357`、
  production/test-list/CLI hash不変に保つ。
- [x] 本documentation commit/fresh preflight後にTask 256C1だけをimplementし、
  review/verify後にseparate commit。
- [x] Task 256C1の両installation order/fresh inventory後だけTask 257C2
  implementationへ戻る。

## Checker Task 256C1 implementation ledger

- [x] `source_atomic_formula.rs`のID-independent authenticated
  condition-container private predicateだけでbounded `source_drift`をcloseし、
  public row/edge/fingerprint/error/debug fieldや`TypedAst` production editを
  追加しない。
- [x] overlapping `Comprehension`がownするequal-range/equal-spellingのnormal
  Task-255 conditionについて、owner contextと一致し、condition siteのdistinct
  direct arena childであるnormal `Equality`だけをauthenticateする。
- [x] exact `4/0/4`、`1/0/1/2/2`、`1/0/1/1/1/1/2`、
  `1/0/0/0/0/0/0/2/2` profile、validation-only optional set context、
  `set_term_fingerprint() == None`、両install order、rollback/replay、
  corruption/preservationをexactly 3 checker testsで固定する。
- [x] checker/mizar-test library `329/357`を測定し、plan `419/385`、type
  `251/239`、pass/fail `228/191`、active `101/5/198/1`、
  warnings/errors `23/0`、runner production、trace、fixture、sidecar、
  expectation、CLI outputを不変に保つ。
- [x] 全semantic deferral/coverage creditを保持する。Task-256C1 exit時点で
  Task 257C2は本implementation commitとfresh inventory後に次の
  dependency-ready logical taskとなった。

## Checker Task 257C2 implementation ledger

- [x] dedicated condition-to-atomic handoff/dense table/ID、exact dependency
  fingerprint 4件、deterministic debug、dedicated typed/resolved
  error/ownershipをTask-257B API不変でpublish。
- [x] exact Task-255C1 selector、imported Task-253 seam、same-arena Task-256
  equality builderをreuseし、全lower ID/site/existing definition-intake detailを
  保持。
- [x] exact publication、corruption、両lower install order、reciprocal
  A/B/C2 exclusion、near miss、isolation、rollback/replay、final cloneを
  checker tests 3件/runner tests 4件でpass。
- [x] reciprocal sidecar reference/noteとcovered trace row 1件だけを追加し、
  191-byte `.miz`、outcome、phase、detail、diagnostic payloadを不変に保持。
- [x] plan `419/386`、type `252/240`、pass/fail `228/191`、active
  `101/5/198/1`、warnings/errors `23/0`、libraries `332/361`、runner
  manifest 29 paths / 34,064 linesを測定。
- [x] 全frozen semantic deferralを保持。

## Checker Task 257C3 frozen-contract ledger

- [x] existing 107-byte Task-257C1 source/hash、exact range、imported
  `divides` provenance、Task-252 `3/0/3`、Task-256
  `1/0/2/2/2/0/0/3/2`を保持。
- [x] `formula=0, left_segment=0, right_segment=1, boundary=1`
  conjunction row 1件と`formula=0, segment=1` negation row 1件をfreeze。
- [x] primary/atomic fingerprint、shared-boundary/negative-token
  reauthentication、deterministic debug/accessor、fail-closed errorをfreeze。
- [x] typed/resolved one-shot ownership、clone preservation、全orderでの
  reciprocal A/B/C2/C3 exclusionをfreeze。
- [x] existing sidecarへfuture reciprocal reference/note 1件とcovered trace
  row 1件だけを許可し、fixture/semantic changeを追加しない。
- [x] baseline `419/386`、type `252/240`、pass/fail `228/191`、active
  `101/5/198/1`、warnings/errors `23/0`、libraries `332/361`、runner
  production 29 paths / 34,064 linesを保持。
- [x] signature/applicability、overload selection、conjunction/negation
  truth、fact/result、theorem acceptance、proof、IR/VC、broader chain、
  Steps 6/7をdefer。
- [x] 本documentation commitとfresh parser/resolver/lower-API/count/hash
  preflight後だけTask 257C3を実装。

## Checker Task 257C3 implementation ledger

- [x] exact Task-252/256 authentication後にfrozen `1/1` compositionだけを
  publishし、全lower row/semantic deferralを不変に保持。
- [x] optional handoffをinstall/revalidate/debug-project/clone-preserveし、
  A/B/C2/C3全6方向をmutation-sensitiveにcover。
- [x] unchanged 107-byte fixtureをlower C1 consumerより先にrouteし、
  typed/resolved semantic tableをemptyに保持。
- [x] exactly checker 3 tests / runner 4 tests、covered trace row 1件、
  existing sidecar ordered reciprocal reference/noteだけを追加。
- [x] plan/type `419/387` / `253/241`、libraries `335/365`、runner
  29 paths / 34,290 linesを測定し、全frozen CLI outcomeを保持。

## Checker Task 258B3 frozen-contract ledger

- [x] final-LF 104-byte source/hash、49-node/root-48 parser range、
  public/exported theorem provenance `[2,1]`、resolver companionなしをfreeze。
- [x] Task-48 `2/1/0`、Task-252 `5/5/0`、Task-256
  `2/0/0/0/0/0/0/4/4`、formula-only base `1/2/2/2/2`、atomic edgeからの
  exact term-2 exclusionをfreeze。
- [x] public witness ID/input/row/table/handoff/producer/error、
  fingerprint 2件、deterministic debug、direct binding context、unnamed
  primary target 1件をfreeze。
- [x] dense base IDs 0/1 + global ordinals 0/2、witness
  source/within-take ordinals 1/0、paired unique `[0,1,2]` validationをfreeze。
- [x] typed/final pair-only ownership、全profile/source-family exclusion、
  rollback/replay、checker tests 4本、runner tests 5本、empty semanticsを
  freeze。
- [x] 全fixture、sidecar、expectation、trace row/status/count、active route、
  source、test list、hashをlibraries `346/379`、runner 30 paths /
  36,479 linesで不変に保持。
- [x] closed contractを`design_drift`、future codeをbounded
  `source_drift`、testsを`test_gap`と分類し、blocking protocol
  disagreementなし。
- [x] 本dedicated documentation commitとfresh parser/resolver/lower-API/
  count/hash preflight後にTask 258B3だけをimplementした。librariesは
  `350/384`、changed hash/lineはimplementation resultで再測定済み。
- [x] fresh-inventoryし、Task 258B3N named-primary witness transportだけを
  freeze。exact 107-byte/51-node source、`1 witness / 1 name` table、B3
  compatibility、no binding/semantics、checker/runner tests 4/5本、
  unchanged baselineを確定。
- [x] dedicated documentation commitとfresh parser/resolver/lower/count/hash
  preflight後、Task 258B3Nだけをimplement。exact dense witness-name
  transport、checker tests 4本、runner tests 5本がpassし、libraryは
  `354/389`。semantic/corpus activationはない。
- [x] broad Task 258B3Mをexact reserved-variable B3M1と
  non-reserved-variable/other-term B3M2へ分解。
- [x] Task 258B3M1だけをfreeze: exact 113-byte/56-node mixed
  two-witness source、Task-252 `6/6/0`、base/witness/name
  `1/2/2/2/2` + `2/1`、shared source ordinal 1、dense ordinals 0/1、
  no new API/semantics、tests 4/5本、unchanged baseline。
- [x] documentation commitとfresh parser/resolver/lower/count/hash
  preflight後にfrozen Task 258B3M1だけをimplement。
- [x] Task 258B3M2をexact unnamed-numeral B3M2Aとremaining other-term
  B3M2BへdecomposeしてからTask 258B4を選ぶ。
- [x] Task 258B3M2Aだけをfreeze: final-LF 107-byte/49-node source、
  Task-252 `5/4/1`、base/witness/name `1/2/2/2/2` + `1/0`、numeric
  request ownership、new API/semanticsなし、tests 4/5本、unchanged
  baselines。
- [x] documentation commitとfresh parser/resolver/lower/count/hash
  preflight後にfrozen Task 258B3M2Aだけをimplement。
- [x] Task 258B3M2Bをexact parenthesized B3M2B1とremaining
  authority-valid B3M2B2へdecompose。
- [x] B3M2B1 documentation prerequisiteをfreezeし、implementationと
  B3M2B2をseparate taskに維持。
- [x] frozen B3M2B1をdocs commit/fresh preflight後にimplement。
- [x] Task 258B3M2B2をexact nested-parenthesized B3M2B2Aとremaining
  authority-valid B3M2B2Bへdecompose。
- [x] Task 258B3M2B2Aだけをfreezeし、implementationをseparateに維持。
- [x] documentation commitとfresh parser/resolver/lower/count/hash
  preflight後にfrozen Task 258B3M2B2Aをimplement。
- [ ] B3M2B2B remaining witness-term shapesをfreeze/implementしてから
  Task 258B4を選ぶ。

## Checker Task 258B3N 実装ledger

- [x] B3 v1 debug bytesを維持し、exact B3N name rowと
  dependency/aggregate/witness/name validation precedenceを追加。
- [x] full 51-node arena、resolver/lower provenance、forward/reverse link、
  subtree exclusion、replay、rollbackをauthenticate。
- [x] binding、semantic、proof、goal、fixture、expectation、sidecar、trace
  status/count、active-corpus ownershipを変更しない。
- [x] `354/389` tests、checker modules `12114/4644/7200/3156`、runner
  production 30 paths / 37,555 linesを再測定。

## Checker Task 258B3M1 documentation ledger

- [x] canonical authority、exact consumer bytes/hash、complete 56-node
  parser identity、theorem-only resolver provenance、subtree exclusionを
  freeze。
- [x] Task-48/252/256/base profileとexact two-witness/one-name syntax
  tableを、`BindingId`/semantic effectなしでfreeze。
- [x] public-API no-op、debug compatibility、fail-closed validation
  precedence、paired typed/final ownership、exact tests 4/5本をfreeze。
- [x] canonical spec、既存`.miz`、fixture、expectation、sidecar、trace
  status/count、active route、source/tests、list/count/hashを維持。
- [x] EN/JA plan/auditのfollow-up ownershipを更新し、B3M1 implementation
  後はB3M2がnextでB4前であることを記録。

## Checker Task 258B3M1 implementation ledger

- [x] exact 113-byte/56-node mixed reserved-variable profileだけをpublishし、
  Task-252 `6/6/0`、Task-256 `2/0/0/0/0/0/0/4/4`、base
  `1/2/2/2/2`、witness/name `2/1`を維持。
- [x] B3/B3N v1 bytesを維持し、complete shared arenaに対する
  dependency/fingerprint、aggregate、witness 0、witness 1、name precedenceを
  validate。
- [x] typed/final ownershipでpaired handoffをatomically installし、binding、
  semantic、proof、goal、corpus、trace、public API ownershipを不変に維持。
- [x] checker exactly 4本 / runner exactly 5本のcompound testsを追加し、
  library `358/394`、checker modules `14045/4659/7201/3156`、runner
  production 30 paths / 38,103 linesを再測定。

## Checker Task 258B3M2A documentation ledger

- [x] canonical authority、exact 107-byte source/hash、complete
  49-node/root-48 unrecovered arena、theorem resolver provenance、lexer
  prerequisite後frontend diagnostics 0をfreeze。
- [x] Task-48 `2/1/0`、Task-252 `5/4/1`、Task-256
  `2/0/0/0/0/0/0/4/4`、base `1/2/2/2/2`、witness/name `1/0`、
  numeral request ownership、source partition `[0,1,2]`、subtree
  exclusionsをfreeze。
- [x] public-API no-op、prior debug compatibility、validation precedence、
  paired typed/final ownership、exact 4/5 compound tests、replay/rollback、
  empty semanticsをfreeze。
- [x] canonical spec、existing `.miz`、fixture、expectation、sidecar、trace
  status/count、active route、source/test、list、count、hashを維持し、
  B3M2B-before-B4を保持。

## Checker Task 258B3M2A implementation ledger

- [x] private exact B3M2A dependency/base/witness profileだけを追加し、
  public APIとprior debug grammarを維持。
- [x] paired `1 witness / 0 names` handoffだけをtyped/final ownershipで
  install/revalidateし、bindingと全semantic/proof/goal tablesを維持。
- [x] identity、precedence、全dependency/node/byte/subtree mutation、
  cross-family order、replay/rollback、empty semanticsをcoverするchecker
  exactly 4本 / runner exactly 5本のcompound testsを追加。
- [x] libraries `362/399`、checker modules
  `15746/4660/7202/3156`、runner sizes `4185/691/2505/8611`、production
  30 paths / 38,571 linesを実測し、B3M2B-before-B4を保持。

## Checker Task 258B3M2B1 frozen-contract ledger

- [x] B3M2Bをparenthesized B3M2B1 / remaining authority-valid B3M2B2へ
  splitし、future `it`をvalid `means` contextへ限定。
- [x] final-LF 113-byte/hash、53-node/root-52、theorem/resolver provenance、
  diagnostics 0をfreeze。
- [x] Task-48 `2/1/0`、five roots / Task-252 `6/5/0`、term 2/child 3、
  refs `0/1/2/3/4 -> 0/1/3/4/5`、scopesをfreeze。
- [x] Task-256 `[0,1]` / `[4,5]`、`2/3` exclusion、base
  `1/2/2/2/2`、input refs `[0,1]` / `[3,4]`、witness/name `1/0`、
  source partition `[0,1,2]`。
- [x] no public API/binding/semantics、tests 4/5、unchanged `362/399`、
  coverage deferred/empty、B3M2B2-before-B4をfreeze。
- [x] dedicated docs commit/fresh preflight後にB3M2B1だけをimplementし、
  `366/404`を実測してB3M2B2を保持。

## Checker Task 258B3M2B1 implementation ledger

- [x] private exact B3M2B1 dependency/base/witness profileだけを追加し、
  public API/prior debug grammarを維持。
- [x] five roots / six primaries、wrapper/child edge、dense references、
  subtree exclusions、complete 53-node arenaをrevalidate。
- [x] paired `1 witness / 0 names`だけをatomic installし、bindingと全
  semantic/proof/goal tablesを維持。
- [x] exactly checker 4 / runner 5 testsでidentity、precedence、mutation、
  Tasks 253–255/family order、replay、cloneをcover。
- [x] libraries `366/404`、checker `17569/4661/7203/3156`、runner
  `4676/695/2508/9902`、production 30 paths / 39,069 linesを実測し、
  B3M2B2をB4前に保持。

## Checker Task 258B3M2B2A frozen-contract ledger

- [x] broad B3M2B2をexact `take ((x));` B3M2B2Aとremaining
  application/structure/selector/update/set/choice/other B3M2B2Bへ分割。
- [x] final-LF 121-byte/hash identity、diagnostics 0、57 nodes/root 56、
  exact theorem-only resolver provenanceをfreeze。
- [x] Task-48 `2/1/0`、five roots / Task-252 `7/5/0`、
  outer/inner/variable chain `2 -> 3 -> 4`、refs `0/1/4/5/6`をfreeze。
- [x] Task-256 `[0,1]` / `[5,6]`、complete witness subtree `2/3/4`
  exclusion、base `1/2/2/2/2`、witness/name `1/0`をfreeze。
- [x] public API/binding/semantics/active/trace changeなし、future checker
  4 / runner 5 tests、unchanged `366/404` baseline、
  B3M2B2B-before-B4をfreeze。

## Checker Task 258B3M2B2A implementation ledger

- [x] private exact nested-parentheses dependency/base/witness profileだけを
  追加し、public API/debug grammarを維持。
- [x] 全57 nodes、five roots/seven primaries、chain `2 -> 3 -> 4`、
  five refs、Task-256 subtree exclusionをrevalidate。
- [x] paired base + `1 witness / 0 names`だけをatomic publishし、binding
  とsemantic/proof/goal tablesを維持。
- [x] exactly checker 4 / runner 5 compound testsを追加し、identity、
  corruption、family、replay、clone casesを全pass。
- [x] libraries `370/409`、checker `19571/4662/7204/3156`、runner
  `5188/699/2513/11234`、production 30 paths / 39,590 linesを実測し、
  B3M2B2BをB4前に保持。

## Checker Task 258B3M2B2B1P frozen lower-prerequisite ledger

- [x] broad B3M2B2Bをdependency-firstでprivate Task-253 proof-context seam
  B1P、exact application-witness B1A、後続Task-253/254/255/other slicesへ
  decompose。
- [x] motivating final-LF 143-byte/hash source、diagnostic 0、63-node/root-62
  identity、projected proof-context-1 Task-48 `2/1/0`、Task-252 `6/4/2`、
  Task-253 `1/0/1/2/2`をfreeze。
- [x] legacy context-0 helper/outputを維持し、checker API/statement consumer
  changeなしでprivate explicit-context Task-253 reuse entry pointだけを追加
  する契約をfreeze。
- [x] context/provenance/form fail-close、replay、legacy byte compatibilityの
  runner compound testsちょうど2件をfreeze。
- [x] canonical artifacts、active routes、fixtures、expectations、sidecars、
  trace status/count、coverage credit、tests、counts、hashesを維持し、
  B1A前にB1P implementationを保持。

## Checker Task 258B3M2B2B1P implementation ledger

- [x] private explicit-context Task-253 reuse siblingだけを追加し、legacy
  helperをcontext-0 delegationで維持。
- [x] exact extractor/public producerをproof context 1でreuseし、Task-253
  `1/0/1/2/2`だけをpublish。
- [x] identity、lower profiles、context/provenance/range/form corruption、
  stale replay、fixed legacy debug bytes、empty statement/semantic/proof/
  goal outputをcoverするcompound testsちょうど2件をpass。
- [x] libraries `370/411`、runner Task-253 sizes
  `1782/701/2514/2799`、production 30 paths / 39,857 linesを実測。
- [x] B3M2B2B1Aのexact application-witness consumerを実装する前にfresh
  inventory/frozen contractを作成。

## Checker Task 258B3M2B2B1A frozen-contract ledger

- [x] Chapter 13/15/16 authority、final-LF 143-byte/hash source、diagnostics
  0、63 nodes/root 62、theorem owner、imported `++` provenanceをfreeze。
- [x] Task-48 `2/1/0`、Task-252 `6/4/2`、Task-253 `1/0/1/2/2`、
  Task-256 equality-only exclusion、base `1/2/2/2/2`、unnamed
  `Application(0)` witness 1/names 0をfreeze。
- [x] node `48 -> 47 -> 46` containmentをwrapper/primary duplicateなしで
  freezeし、witness handoffだけをTask-253 consumerとする。
- [x] `Application` target、optional fingerprint、legacy-compatible
  builder/debug、application-aware producer、atomic typed/final installを
  freeze。
- [x] checker 4/runner 5 tests、unchanged `370/411`、30 paths/39,857
  lines、no active/canonical/fixture/expectation/sidecar/trace/semantic
  change、coverage `deferred`/`tests = []`をfreeze。
- [x] dedicated docs commit/fresh preflight後、B3M2B2B1Aだけを実装し、
  libraries `374/416`を実測。

## Checker Task 258B3M2B2B1A implementation ledger

- [x] `Application(0)` witness ownership、B1Aだけのoptional application
  fingerprint、application-aware builderを追加し、legacy
  application-free bytesを維持。
- [x] exact real `parser.type_fixtures::++` symbol、local/FQN lookup、
  contribution/path/export provenance、Task-252 arguments、Task-253
  application、Task-256 exclusion、全63 nodesをauthenticate。
- [x] application/statement/witness handoffsをatomic installし、stale、
  orphan、hybrid、reverse、repeat、semantic coexistenceをpartial publish
  なしでrejectし、final cloneでもrevalidate。
- [x] checker/runner compound tests `4/5`をpassし、143 loaded-source
  bytes全mutation、reparsed near miss、dependency/provenance corruption、
  validation precedence、family order、rollback/replay、empty semantic/
  proof/goal ownershipをcover。
- [x] libraries `374/416`、checker modules
  `21664/4742/7224/3156`、runner statement sizes
  `5618/706/2520/11945`、production 30 paths / 40,298 linesを実測し、
  canonical artifacts、active routes、fixtures、sidecars、expectations、
  trace status/count、coverage `deferred` / `tests = []`を維持。

## Checker Task 258B3M2B2B1B1P frozen-prerequisite ledger

- [x] 全Task-254/255 witness shapesより前に、exact final-LF
  158-byte/67-node `take (1 ++ 2);` sourceをselect。
- [x] Task-48 `2/1/0`、Task-252 `6/4/2`、wrapped Task-253
  `1/1/1/2/2`をproof context 1、exact imported `++` provenanceと共に
  freeze。
- [x] application node 48を囲むwrapper node 50をTask-253 containment
  onlyとしてfreezeし、later witness targetを`Application(0)`のまま維持。
- [x] private wrapper-aware reuse sibling 1件、legacy unwrapped byte
  compatibility、future runner compound testsちょうど2件をfreeze。
- [x] exhaustive 158-byte/67-node selector isolation、全successful lower
  row fields、dormant-route exclusion、empty upper tablesをfreeze。
- [x] selector/Task-252/Task-253/typed-install failure precedence、atomic
  rollback/replay、separate legacy context-0/context-1 hashesをfreeze。
- [x] production/tests、canonical artifacts、active routes、fixtures、
  expectations、sidecars、trace status/count、public APIs、semantic
  ownership、libraries `374/416`、全measured hashesをpreserve。
- [x] このdocumentation prerequisiteだけをcommitし、fresh-inventory後、
  B1B1 statement consumerをfreezeする前にB1B1Pだけをimplement。

## Checker Task 258B3M2B2B1B1P implementation ledger

- [x] runner-private wrapped Task-253 reuse seamだけを追加し、checker
  source、public APIs、active routes、B1B1 consumerを不変に維持。
- [x] exact imported `++` provenanceをauthenticateし、same-source resolver
  substitution 5 classesをlower-table publication前にreject。
- [x] 全158 bytes、67-node/root mutations、eight-entry diagnostic/node
  matrix、precedence、rollback/replay、legacy hashes、exact rows、empty
  upper tablesをcoverするcompound testsちょうど2件をpass。
- [x] checker/runner tests `374/418`、runner sizes
  `2652/708/2523/3727`、production 30 paths / 41,173 linesを実測。
- [x] canonical artifacts、fixtures、expectations、sidecars、trace
  status/count、executable coverage credit、semantic/proof/goal ownersを
  preserve。
- [x] implementation commit後、B1B1 statement-consumer documentationを
  separate logical taskとしてfresh-inventory。

## Checker Task 258B3M2B2B1B1 frozen-contract ledger

- [x] final-LF 158-byte/67-node `take (1 ++ 2);` sourceとcanonical
  Chapter 13/15/16 + parser-fixture authorityをfresh-inventory。
- [x] exact local theorem owner/contribution/label bundleとimported
  `parser.type_fixtures::++#12` provenanceをfreeze。
- [x] Task-48 `2/1/0`、Task-252 `6/4/2`、wrapped Task-253
  `1/1/1/2/2`、equality-only Task-256、base `1/2/2/2/2`、one unnamed
  `Application(0)` witness/no namesをfreeze。
- [x] take 53 -> witness 52 -> unowned 51 -> wrapper 50 -> unowned 49
  -> application 48をfreeze。wrapper 0はTask-253 containmentでwitness
  targetではない。
- [x] existing public B1A schema/atomic installerをreuseし、explicit
  crate-private B1B1 profile 1件をrequire、B1A byte/API compatibilityを維持。
- [x] exact validation precedence、atomic rollback/replay/final
  revalidation、named checker tests 4件/runner tests 5件、全bytes/nodes、
  resolver substitutions、near-miss matrix、family/active isolation、empty
  semantic outputsをfreeze。
- [x] production/tests、canonical specs、`.miz`、fixtures、expectations、
  sidecars、trace status/count、public/active routes、semantic owners、
  libraries `374/418`、全measured hashesをpreserve。
- [x] このdocumentation prerequisiteだけをcommitし、fresh-inventory後、
  B1B1だけをimplement。

## Checker Task 258B3M2B2B1B1 implementation ledger

- [x] exact private B1B1 statement/witness profileをimplementし、B1A
  byte/API compatibilityをpreserve。
- [x] frozen checker tests 4件/runner tests 5件をpass。inventoriesは
  `378/423`。
- [x] `source_drift`、`test_gap`、completion `design_drift`をcloseし、
  test-sufficiency/implementation reviewsはfindingsなし。
- [x] spec、`.miz`、fixture、expectation、sidecar、trace、active、
  public-API artifactsを不変にし、semantic/proof/goal/type-substitution
  deferralsを維持。
- [x] commit前のfinal read-only quality reviewは全hard gate PASS、
  valid score `98/100`。

## Checker Task 258B3M2B2B2P frozen-prerequisite ledger

- [x] final-LF 172-byte/76-node source、SHA-256、exact theorem/proof/
  take/witness/constructor/root/member/numeral/conclusion/formula nodes、
  containmentをfreeze。
- [x] Task-48 `2/1/0`、Task-252 `6/4/2`、exact Task-254
  `1/0/1/2/0/2/6` rows、imported `TypeCaseStruct#5` provenanceをfreeze。
- [x] owned-kind mapをconstructor 59とassignment members 20/24だけにfreeze。
  root 52はunowned、Task-252 private extraction roots 54/57とpublishされる
  `source.term.numeral` sites 53/56を区別し、54/57はarena-unownedとする。
- [x] 全bytes/nodes、provenance/substitution/precedence、replay、legacy
  output、empty upper families用future runner tests 2件、checker testなし。
- [x] §5.7 selector semanticsをB2B、witness ownershipをB2A、update/
  `FieldUpdate`をB2Cへexcludeし、semantic/proof/goal/type deferralsと
  existing Task-254 creditをpreserve。
- [x] `378/423`、全counts/hashes、public/active/fixture/expectation/
  sidecar/trace artifactsをpreserveし、selected gapを`design_drift`に分類。
- [x] no-findings docs review、hard gates、quality 90/100以上、dedicated docs
  commit後、fresh-inventoryしてB2Pだけをimplement。

## Checker Task 258B3M2B2B2P implementation ledger

- [x] exact production-private owned-kind selectorと
  existing-context/shared-Task-252 Task-254 reuse seamをimplement。
- [x] frozen runner tests 2件をpass。inventoriesは`378/425`。
- [x] `source_drift`、`test_gap`、completion `design_drift`をclose。
- [x] checker/public/active/fixture/expectation/sidecar/trace/semantic
  boundariesをpreserveし、Task-258 statement/witness rowはpublishしない。
- [x] runner sizes/manifests/test-list hashesを再測定し、B2Aを次に維持。
- [x] final read-only quality reviewはfindingsなし、全hard gate PASS、
  valid score `98/100`。
- [x] commit/fresh inventory後、B2Aをseparate documentation taskとして
  freeze。

## Checker Task 258B3M2B2B2A frozen-contract ledger

- [x] Task `258B3M2B2B2A`をhistorical `258B3M2B2A`と区別し、exact
  172-byte/76-node constructor-witness sourceをfreeze。
- [x] Task-48/252/254/256 lower rows、Task-258 base `1/2/2/2/2`、
  witness/name `1/0`、ownership/subtree exclusions、両resolver rootsをfreeze。
- [x] additive structure target/fingerprint/builder/atomic installer APIsを
  freezeし、全legacy/application debug/installer pathsをpreserve。
- [x] checker tests 4件/runner tests 5件、validation precedence、
  rollback/replay/final clone、family/active isolation、empty semanticsをfreeze。
- [x] canonical/test/fixture/expectation/sidecar/trace/active artifacts、
  formula trace `deferred` / `tests = []`、全counts/hashesをpreserve。
- [x] no-findings specification reviewと全documentation hard gatesを
  valid final quality `98/100`で完了。
- [x] dedicated docs commit後、fresh-inventoryしてB2Aだけをimplement。

## Checker Task 258B3M2B2B2A implementation ledger

- [x] exact frozen profileのstructure target/fingerprint/builderとatomic
  typed/final installationをimplement。
- [x] existing B2P seam経由のbounded runner consumerをimplementし、exact
  checker 4/runner 5 testsをpass。
- [x] bounded B2A `source_drift`/`test_gap`をcloseし、B2B/B2C/semantic
  deferralsをpreserve。
- [x] tests `382/430`、module/manifests、test-list hashesをrecordし、trace
  `deferred` / `tests = []`をcreditなしで維持。
- [x] no-findings test-sufficiency/implementation/source-doc consistency
  reviewsをcomplete。
- [x] focused checker/runner `4/4`/`5/5`、`cargo fmt --all --check`、
  all-target/all-feature Clippy warnings denied、libraries `382/430`/
  lint policies `15/14`を含む`cargo test -q`をPASS。
- [x] five CLIs exit 0、warnings 23/errors 0、counts/hashes不変をverifyし、
  manifests/test lists/forbidden artifactsはunchanged、`stash@{0}`は
  untouched。
- [x] 全9 hard gates PASS、valid score `98/100`でfinal read-only quality
  reviewをcomplete。
- [x] logical taskを`7613d50d`でcommitし、clean metadata/stash
  invariantsをverifyしてnext dependencyをfresh-inventory。

## Checker Task 258B3M2B2B2BP frozen-contract ledger

- [x] B2B前のprivate Task-254 selector reuse prerequisiteとしてB2BPを
  selectし、171-byte/79-node exact source/parser identityをfreeze。
- [x] Task-48 `2/1/0`、Task-252 `6/4/2`、Task-254
  `2/0/1/3/0/3/9`とprovenance/ownership/edges/requests/malformed/
  subtree exclusionsをfreeze。
- [x] runner-private selector site/owned-kind/context-handoff siblingsだけを
  freezeし、checker APIs、Task-256/258、active routes、diagnostics、
  statement installation、semanticsをexclude。
- [x] runner tests exact 2件/checker tests 0件とcorruption/precedence/
  replay/constructor compatibility/excluded shapes/empty upper tablesを
  freeze。
- [x] specs、`.miz`、fixtures、expectations、sidecars、trace
  `deferred`/`tests = []`、Task-254 credit、baselines/hashesをpreserve。
- [x] no-findings specification/source-documentation reviewsと全docs
  verificationをcomplete。
- [x] externally created docs commit `6f84d4eb`をreport-only
  `repo_metadata_conflict`としてrecordし、amend/revert/fetch/push/stash
  actionを行わない。
- [x] Task `258B3M2B2B2BPC1`をB2BPをimported constructor/root provenance
  だけに限定しlocal theorem owner/label provenanceをB2Bへdeferする
  docs-only correctionとしてfreezeし、3 reviewsをfindingsなしまでrepeat。
- [x] BPC1 final read-only quality reviewをfindingsなし、全9 hard gates、
  valid score `98/100`でPASS。
- [x] correctionだけをcommit、clean/stash invariants verify後にB2BP
  implementationだけをfresh-inventory。
- [x] separate B2BP implementation commit後にB2B frozen consumer docsへ
  戻る。

## Checker Task 258B3M2B2B2BP implementation ledger

- [x] frozen 4 runner filesでexact production-private selector site、
  owned-kind map、proof-context handoffを実装。
- [x] malformed diagnostic code/rangeとstale-fingerprint clean replayを
  含むexact runner tests 2件をPASS。
- [x] bounded `source_drift`/`test_gap`をcloseし、test-sufficiency/
  implementation reviewsはfindingsなし。
- [x] checker/public/active/spec/fixture/expectation/sidecar/trace/semantic
  boundaryをpreserveし、Task-256/258 rowをpublishしない。
- [x] libraries `382/432`、current module sizes、production/test-list
  hashes、不変のCLI counts/hashes 5件を記録。
- [x] source/documentation consistencyをfindingsなしで完了し、全9 hard
  gatesとvalid `98/100`でfinal quality reviewをPASS。
- [x] implementation commit 1件を作成後、B2B documentationを
  fresh-inventory。

## Checker Task 258B3M2B2B2B frozen-contract ledger

- [x] exact 171-byte selector-witness source、parser nodes、malformed
  recovery、resolver provenance、lower Task-48/252/254/256 rows、
  ownership/subtree exclusionsをfreeze。
- [x] Task-258 base `1/2/2/2/2`、witness `1/0`、target
  `Structure(0)`、selector-base `Structure(1)`、exact B2A/B2B sibling
  boundaryをfreeze。
- [x] exact checker/runner consumers、implementation file scopes、checker
  tests 4件、runner tests 5件、validation precedence、family isolation、
  semantic/B2C deferralsをfreeze。
- [x] baseline library/projection counts、module sizes、production/test-list
  hashes、CLI counts/hashes、unchanged coverage-credit impactを記録。
- [x] specification、test-sufficiency、implementation-boundary、
  source/documentation reviewsをfindingsなしで完了し、final quality全9
  hard gatesをvalid `98/100`でPASS。
- [x] このdocumentation prerequisiteだけを`4d2fb2b6`としてcommitし、
  clean worktree、ahead count、untouched stashをverify。
- [x] commit後にTask 258B3M2B2B2B implementationをfresh-inventory。

## Checker Task 258B3M2B2B2B implementation ledger

- [x] exact frozen B2B profileをauthorized 8 filesでimplementし、unnamed
  witness 1件をselector `Structure(0)`へtarget。
- [x] Task-48/252/254/256とTask-258 base rows、selector base
  `Structure(1)`、Task-256 ownership `51/70`、unowned containers
  `52/71`をpreserve。
- [x] B2A/B2Bをexact fail-closed typed/final siblingsとしてenumerateし、
  generic structure admission、hybrid、stale fingerprint、partial/
  repeated bundleをatomically reject。
- [x] frozen checker 4/runner 5 testsを追加。checker 4 testsとfocused
  runner matrixはPASS。
- [x] bounded `source_drift`、`test_gap`、`design_drift`をcloseし、
  public、semantic/proof/goal、corpus active-route、fixture、expectation、
  sidecar、trace-credit boundariesをpreserve。
- [x] libraries `386/437`、checker sizes
  `29941/4830/7244/5036`、23-path / 124,016-line production manifest、
  checker production/test-list hashesを記録。
- [x] specification/dependency、test-sufficiency、implementation reviewsを
  findingsなしで完了。
- [x] source/documentation consistency reviewをfindingsなしで完了。
- [x] broad final verificationを実行し、全count/hash gatesを確認。
- [x] final read-only quality reviewで全hard gatesをPASSし、valid
  `90/100`以上を取得。
- [x] implementationをlogical commit `8311502c`としてcommitし、clean
  worktree、ahead-three origin metadata、untouched stashをverifyしてから
  B2Cより先のB2CP prerequisiteをfresh-inventory。

## Checker Task 258B3M2B2B2CP frozen-prerequisite ledger

- [x] later B2C statement consumerより先にmissing private Task-254 update
  reuse seamをB2CPとしてestablish。
- [x] exact final-LF 181-byte/hash、86-node/root-85 source、180-byte
  missing-value recovery profileをfreeze。
- [x] Task-48 `2/1/0`、Task-252 `7/4/3`、Task-254
  `2/0/1/3/1/4/9`、imported `TypeCaseStruct#5` provenance、ownership、
  subtree exclusionsをfreeze。
- [x] runner implementation files 4件、runner tests exactly 2件、checker
  tests 0件、no-public/no-active/no-semantic surfaceをfreeze。
- [x]
  `task258b3m2b2b2cp_structure_update_proof_context_reuse_is_exact`と
  `task258b3m2b2b2cp_structure_update_corruption_replay_and_prior_sibling_compatibility_fail_closed`
  をfreezeし、exact B2P constructor/B2BP selector compatibilityをcover。
- [x] canonical artifacts、fixtures、expectations、sidecars、trace
  status/count/credit、libraries `386/437`、all baseline hashesをpreserve。
- [x] repeated specification/dependency、test-sufficiency、
  implementation-boundary、source/documentation reviewsをfindingsなしで
  completeし、全documentation verification/hard gatesをPASS。
- [x] concurrent docs commit `817bb92b`をreport-only
  `repo_metadata_conflict`としてrecordし、restored `spec_gap` labelが
  hard gates 1/9とrecorded `98/100`をinvalidateしたことを記録。
- [x] CPC1 repeated no-findings reviewsをcompleteし、全9 hard gatesを
  PASS、valid final quality `98/100`を取得。unrelated incomplete source
  diffでblockされるlive broad rerunを明示的にjustify。
- [x] docs-only correction `258B3M2B2B2CPC1`を`ee267d9c`として
  separate commit。
- [x] fresh-inventory後private dormant B2CP seamだけをimplementし、
  frozen tests exactly 2件をPASS、`design_drift`、`source_drift`、
  `test_gap`をclose。
- [x] final test-sufficiency/implementation re-reviewsをfindingsなしで
  complete。
- [x] `cargo fmt --check`、workspace Clippy warnings-denied、`cargo test`、
  focused B2CP `2/2`、全count/hash gatesをPASS。
- [x] specification/corpus/trace-creditを変更せずcompletion metricsと
  narrative-only audit impactを同期。
- [x] final source/documentation reviewをfindingsなしでcomplete。
- [x] independent final qualityをfindingsなし、全9 hard gates PASS、
  valid `98/100`でcomplete。
- [x] staged-diff auditをPASSし、dedicated B2CP implementation commit
  `b146f0f72dceac2233c9d679b7820e264974b227`を作成。clean worktree、
  ahead-six branch、unchanged stashをverify。
- [x] B2CP commit後B2Cをfresh-inventory。

## Checker Task 258B3M2B2B2C frozen-contract ledger

- [x] exact 181-byte/hash、zero-diagnostic 86-node/root-85 sourceと
  180-byte malformed missing-value profileをfreeze。
- [x] Task-48 `2/1/0`、Task-252 `7/4/3`、Task-254
  `2/0/1/3/1/4/9`、Task-256 `2/0/0/0/0/0/0/4/4`、Task-258 base
  `1/2/2/2/2`、witness `1/0`をfreeze。
- [x] local theorem/label、imported `TypeCaseStruct#5` provenance、
  exact ownership、unowned containers、directed cross-family graphを
  freeze。
- [x] unchanged public structure-witness APIsとprivate B2CP seamのreuse、
  implementation files exactly 8件、checker tests 4件、runner tests
  5件をfreeze。
- [x] documentation-only scope、baseline `386/439`、implementation
  projection `390/444`、全production/test-list/CLI hashes/counts、
  narrative-only `deferred`, `tests = []` trace statusをpreserve。
- [x] missing contract/stale statusを`design_drift`、future
  implementationをbounded `source_drift`、9 testsを`test_gap`とclassify。
  `spec_gap`、boundary、expectation、semantic issueはない。
- [x] specification/dependency reviewをfindingsなしでcomplete。
- [x] test-sufficiency reviewをfindingsなしでcomplete。
- [x] implementation-boundary reviewをfindingsなしでcomplete。
- [x] source/documentation consistency reviewをfindingsなしでcomplete。
- [x] documentation verificationとrequired count/hash gatesを実行。
- [x] 全hard gates PASSかつvalid score `98/100`のfinal read-only quality
  reviewをcomplete。
- [x] B2C documentation prerequisiteを
  `d6076cc757ce675d1b46a720b4f00805923d3c70`としてseparate commit。
- [x] fresh-inventoryしてfrozen B2C contractだけをimplement。

## Checker Task 258B3M2B2B2C implementation ledger

- [x] implementationをfrozen checker 3/runner 5 filesに限定し、unrelated
  formatter/pre-existing semantic churnを残さない。
- [x] existing public structure-witness APIsとunchanged private B2CP update
  seamをreuseし、public schema/active corpus caseを追加しない。
- [x] update `Structure(0)`をtargetとするunnamed witness 1件だけをpublishし、
  complete Task252/254/256/base ownership graphをpreserve。
- [x] checker 4/runner 5 tests exactlyを追加しbounded
  `source_drift`/`test_gap`をclose。
- [x] focused checker `4/4`、runner `5/5`、checker library `390`、
  runner library `444`+policy suitesをPASS。
- [x] final test-sufficiency/implementation reviewsをfindingsなしでcomplete。
- [x] final sizes、production manifests、test-list hashesをpaired completion
  docsへrecord。
- [x] canonical spec、`.miz`、fixtures、expectations、sidecars、trace
  status/tests、coverage credit、public API、semanticsをunchangedに保つ。
- [x] broad workspace fmt/Clippy/test verificationをcompleteし、focused
  `4/4`/`5/5`、sibling `12/12`/`21/21` suitesもPASS。
- [x] final source/docs consistency re-reviewを**NO FINDINGS**でcomplete。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  valid `98/100`でcomplete。
- [x] cached diffをauditしdedicated B2C implementation commit
  `e8373c683448e524cb98edde83fdf8de83a125cd`を作成。
- [x] clean ahead 8/behind 0 post-commit state、unchanged stashをverifyし、
  次のdependency-authorized task B3Pをfresh inventory。

## Checker Task 258B3M2B2B3P frozen-contract ledger

- [x] B2Cをcommit `e8373c683448e524cb98edde83fdf8de83a125cd`、
  clean post-commit invariants、untouched stashでclose。
- [x] exact 117-byte/hash、zero-diagnostic 57-node/root-56 source、complete
  significant kind/range/containment mapをfreeze。
- [x] local resolver provenance、Task48 `2/1/0`、Task252 `6/4/2`、
  Task255 `1/0/0/0/0/2/1`、empty Tasks253/254/256/258をfreeze。
- [x] Task252 owner `30/32/36/38/44/46`、Task255 owner 40、unowned
  term/statement/proof/theorem containersをfreeze。
- [x] private runner implementation files exactly 4件、compound tests 2件、
  explicit proof-context reuse、byte-identical context-0 legacy behaviorをfreeze。
- [x] 2 testsを117 bytes/final-LF variants全件、57 node fields/root、
  complete resolver/Task48/252/255 fields、exact owner partitions、
  precedence/stale replay/atomic clone、empty adjacent/semantic outputs、
  Task111 literal legacy debug hashes 3件までexhaustiveにfreeze。
- [x] checker source/tests/APIとupper B3A witness edgeをseparateに保つ。
- [x] semantic result、sethood/element unification、existential/proof/goal/
  theorem behavior、adjacent term forms、imported behavior、Tasks
  253/254/256/258、B4/B5をexclude。
- [x] docs-only scope、baseline `390/444`、projection `390/446`、全
  production/test-list/CLI counts/hashes、deliberate trace no-opをpreserve。
- [x] missing contractを`design_drift`、future private seamを
  `source_drift`、2 testsを`test_gap`にclassifyし、他disagreementなし。
- [x] specification reviewを**NO FINDINGS**でcomplete。
- [x] documentation review/repeatを**NO FINDINGS**でcomplete。
- [x] test-sufficiency reviewを**NO FINDINGS**でcomplete。
- [x] implementation-boundary reviewを**NO FINDINGS**でcomplete。
- [x] source/documentation consistency reviewを**NO FINDINGS**でcomplete。
- [x] source/hash、lint `15/14`、libraries `390/444`、production/test-list、
  5 CLI hashes、exact 26-doc scope、diff-check、trace no-op verificationをPASS。
- [x] final qualityを**NO FINDINGS**、全9 hard gates PASS、valid
  `98/100`（`20/20/15/14/10/10/5/4`）でcomplete。
- [x] task-only docs diffをauditしB3P prerequisiteを
  `285a1f11c310bb313c4c6b4feae914eb11f74754`としてcommit。
- [x] clean post-commit invariants、unchanged stashをverifyしB3P
  implementationをfresh inventory。

## Checker Task 258B3M2B2B3P implementation-closure ledger

- [x] prerequisite commit
  `285a1f11c310bb313c4c6b4feae914eb11f74754`をrecord。
- [x] exact 4 existing runner filesに`pub(super)` explicit-context sibling/
  context-0 delegateをimplementし、3 literal Task-111 hashesをpreserve。
- [x] exact 2 testsで117 bytes/LF、57 nodes、resolver `63`、binding `39`、
  Task-252/255、fingerprint-only absence、stale precedence、immediate
  replay、clones/isolationをcover。
- [x] B3P `source_drift`/`test_gap`をcloseし、checker/public/active、
  canonical/fixture/expectation/sidecar/traceをunchangedに維持。
- [x] test-sufficiency reviewを**NO FINDINGS**でcomplete。
- [x] implementation reviewを**NO FINDINGS**でcomplete。
- [x] focused `2/2`、runner library `446/446`、format、package Clippy
  `-D warnings`、diff checkをPASS。
- [x] current counts/hashesとtrace no-opをrecord。
- [x] source/documentation consistencyとdocumentation/boundaryのrepeat
  reviewを**NO FINDINGS**でcomplete。
- [x] lint-policy `15/14`、metadata `137`、focused `2/2`、runner
  library `446/446`、format、workspace-wide warnings-denied Clippy/tests、
  5 CLI/count/hash、current manifest/test-list hash、exact 30-file scope、
  diff-check gatesをPASS。
- [x] final read-only quality reviewを**NO FINDINGS**、全9 hard gates
  PASS、valid `98/100`（`20/20/15/14/10/10/5/4`）でcomplete。
- [x] dedicated B3P implementation commit
  `abbfedfc2cdbaa97d8294893859da8cd350ad9a8`をaudit/stage/create。
- [x] clean post-commit HEAD、ahead-10 origin metadata、untouched stash
  `f65cf4a13752ec380710814a9ac6392ccb9d75d4`をverifyしupper B3Aを
  fresh inventory。

## Checker Task 258B3M2B2B3A frozen-contract ledger

- [x] B3P commit `abbfedfc2cdbaa97d8294893859da8cd350ad9a8`、
  clean/ahead-10/untouched-stashをcloseしB3Aへownership transfer。
- [x] Chapters4/13/15/16 authority、parser/failure artifacts、B3P evidence、
  Tasks48/252/255/256/258 patternsをfreeze。
- [x] 117 bytes/57 nodes、resolver label/owner provenance、lower tables、
  witness1/names0、partition/graph、source-only intentをfreeze。
- [x] additive API/debug、exact7 implementation files、checker4+runner5
  tests、precedence、semantic deferralsをfreeze。
- [x] `design_drift`/`source_drift`/`test_gap`、blocking disagreementなし、
  trace `deferred`, `tests = []`、Task-111/255 credit unchanged、
  current/projected counts/hashesをrecord。
- [x] specification reviewを**NO FINDINGS**でcomplete。
- [x] documentation review/repeat、test-sufficiency、
  implementation/API boundary repeatを**NO FINDINGS**でcomplete。
- [x] source/count/hash、lint/library、5 CLI、exact32 scope、diff、
  trace no-op verificationをPASS。
- [x] source/docs consistencyとdocumentation/boundary reviewsを
  **NO FINDINGS**でcomplete。
- [x] final qualityを**NO FINDINGS**、全9 hard gates PASS、valid
  `98/100`（`20/20/15/14/10/10/5/4`）でcomplete。
- [x] dedicated B3A documentation-only commit
  `f4ff45964d97b31b6c328381120ba8ede080a2b1`をcreate。
- [x] clean postcommit ahead-11/behind-0 metadata、unchanged stash
  `f65cf4a13752ec380710814a9ac6392ccb9d75d4`、fresh B3A implementation
  inventoryをverify。

## Checker Task 258B3M2B2B3A implementation ledger

- [x] prerequisite commit
  `f4ff45964d97b31b6c328381120ba8ede080a2b1`とclean
  ahead-11/behind-0、unchanged-stash、fresh-inventory gatesをclose。
- [x] exact checker3+runner4 filesだけをimplementし、両
  `source_set_term.rs` ownerと全authority artifactをpreserve。
- [x] exact set-witness API、set-only fingerprint tuple、atomic typed
  installation、final revalidation/clone、checker4+runner5 testsを追加し、
  semantic deferralsとtrace no-creditを保持。
- [x] specification/test-sufficiency/implementation reviewsを
  **NO FINDINGS**でcomplete。
- [x] focused/package tests、fmt、targeted Clippy、5 CLI、final
  count/hash manifests、diff checksをPASS。
- [x] 2回目のsource/documentation consistency repeatを
  **NO FINDINGS**でcomplete。
- [x] final documentation/boundary rereadを**NO FINDINGS**でcomplete。
- [x] parent final verificationをPASS：focused checker `4` + runner `5`、
  checker package `394` + lint-policy `15`、mizar-test package `451` +
  layout `3` + lint-policy `14` + metadata `137` + public-enum `2` +
  snapshot `21`、fmt、workspace Clippy/tests、5 CLI counts/hashes、
  production manifests/test lists、diff check、exact `39`-file scope。
- [x] independent final read-only quality reviewを**NO FINDINGS**、全9
  hard gates PASS、score capなし、valid `98/100`
  （`20/20/15/14/10/10/5/4`）、記載済みresidual deferrals unchangedで
  complete。
- [x] dedicated implementation commit
  `a147bad88f1963c504f796051ba0b855eca71d07`をcreate。
- [x] clean ahead-12/behind-0 implementation postcommit invariantsと
  unchanged stashをverify。
- [x] fresh next-task inventoryをcompleteし、B3B empty enumerationを
  select。

## Checker Task 258B3M2B2B3B frozen-contract ledger

- [x] B3Aを`a147bad88f1963c504f796051ba0b855eca71d07`でcloseし、clean
  ahead-12/behind-0 stateとunchanged stashをverify。
- [x] empty-enumeration対choiceのorderingをnonblockingな
  task-decomposition `design_drift`としてresolveし、dependency-minimal
  zero-edge enumerationをselectして、その他すべてのset/choice/B3 siblingは
  post-implementation fresh inventoryまで保持。
- [x] final-LF 118-byte/hash identity、diagnostics 0、全50 normal
  nodes/root 49、exact local theorem/label provenanceをfreeze。
- [x] Task-48 `2/1/0`、Task-252 `4/4/0`、empty Tasks 253/254、
  Task-255 `1/0/0/0/0/0/1`、Task-256
  `2/0/0/0/0/0/0/4/4`、Task-258 base `1/2/2/2/2`、witness
  `1/0`をfreeze。
- [x] ownership、zero-child Task-255 boundary、sole
  witness-to-SetTerm edge、resolver provenance、complete subtree
  exclusionをfreeze。
- [x] B3A SetTerm APIとexact seven future implementation filesをreuseし、
  Task-255 source changesと全authority artifactsをforbid。
- [x] checker4+runner5 tests、baseline `394/451`、projection
  `398/456`、current production/test/CLI hashes、trace `deferred`、
  `tests = []` no-opをfreeze。
- [x] inactive template fixtureのexisting semantic intent/creditをpreserveし、
  source、expectation、trace rowを変更しない。
- [x] specification/documentation reviewを**NO FINDINGS**でcomplete。
- [x] test-sufficiency/implementation-boundary reviewsをno findingsで
  complete。
- [x] source/documentation consistency reviewを**NO FINDINGS**でcomplete。
- [x] exact source/count/hash/scope/diff/trace-no-op verificationをPASS。
- [x] final quality reviewを**NO FINDINGS**、全9 hard gates PASS、
  score capなし、valid `98/100`
  （`20/20/15/14/10/10/5/4`）でcomplete。
- [x] dedicated B3B documentation-only commit
  `080e6824d843655986079f5d5fc41abe06b0fbd6`をcreate。
- [x] clean ahead-13/behind-0 post-commit state、unchanged stash
  `f65cf4a13752ec380710814a9ac6392ccb9d75d4`をverifyし、B3B
  implementationをfresh inventory。

## Task 258B3M2B2B3B implementation completion

- [x] prerequisite commit/post-commit/fresh-inventory gatesをclose。
- [x] exact seven-file private transportとfrozen checker 4 / runner 5
  testsを実装し、public API/error/debug/dependency/active routeをpreserve。
- [x] initial test-sufficiency findings 3件をexisting tests内でremediate。
- [x] additional B3B-specific currently mutable Task-48/252/255
  mutation/replay gapをexact `32/55/23` matricesでremediate。
- [x] independent full implementation repeatを**NO FINDINGS**でcomplete。
- [x] post-auth injectionとstage-prefix/non-generic-guard assertionsを追加し、
  全test-sufficiency repeatsを**NO FINDINGS**でcomplete。
- [x] final implementation repeatを**NO FINDINGS**でcomplete。
- [x] bounded follow-up前にfocused checker `4/4`、runner `5/5`、
  format/diff、workspace Clippy `-D warnings`、workspace `cargo test -q`
  をPASS。
- [x] follow-up後のaffected runner counts/content/test hashesをremeasure。
- [x] focused tests、libraries `398/456`、format/diff、workspace Clippy
  `-D warnings`、`cargo test -q`、5 CLI verificationをrerun。
- [x] medium `design_drift` wording fixes 2件後のsource/documentation
  consistency repeatを**NO FINDINGS**でcomplete。
- [x] independent final documentation/boundary reviewを
  **NO FINDINGS**でcomplete。
- [x] independent final read-only quality reviewを**NO FINDINGS**、全9
  hard gates PASS、score capなし、valid `98/100`
  （`20/20/15/14/10/10/5/4`）でcomplete。
- [x] exact `39` task filesをstageし、cached diffをinspect。
- [x] dedicated implementation commit
  `dbbf5f6a2b0bd58d8434fb4687f7bfad398ca4bc`をcreate。
- [x] clean ahead-14/behind-0 post-commit invariants/stash unchangedをverify。
- [x] fresh inventoryでB3C choice witnessをselect。

## Checker Task 258B3M2B2B3C frozen-contract ledger

- [x] B3Bを
  `dbbf5f6a2b0bd58d8434fb4687f7bfad398ca4bc`、clean
  ahead-14/behind-0、stash unchangedでclose。
- [x] choiceをcomprehension/`qua`よりdependency-minimalなprepared Task-255
  siblingとしてselectし、missing contract/route/testsを
  `design_drift`/`source_drift`/`test_gap`にclassify。
- [x] final-LF `110`-byte/hash source、diagnostic 0、全`52` nodes/root `51`、
  resolver owner/label provenance、exact sitesをfreeze。
- [x] Task-48 `2/1/0`、Task-252 `4/4/0`、Tasks 253/254 empty、
  Task-255 `1/0/0/1/0/0/2`、Task-256 `2/0/0/0/0/0/0/4/4`、
  Task-258 base `1/2/2/2/2`、witness `1/0`をfreeze。
- [x] exact ownership/unowned partition、complete graph、Task-255 child edge
  0、choice/witness subtree exclusionをfreeze。
- [x] B3A/B3B SetTerm APIsとfuture source consumers exact 7をreuseし、
  両`source_set_term.rs`と全authority artifactsをforbid。
- [x] exact checker 4 + runner 5 namesとbyte/LF、`52 x 4`+root、
  resolver、`32/55/39/72/62/21`、zero-edge、family-order、
  replay/rollback/clone matricesをfreeze。
- [x] parser/Task-255 fixture、expectation、active intent、trace creditを
  preserveし、formula-statement traceは`deferred`, `tests = []`。
- [x] initial medium ownership/subtree `design_drift`とexact-matrix
  `test_gap`をremediateし、repeat specification review **NO FINDINGS**。
- [x] final documentation consistency/boundary review **NO FINDINGS**。
- [x] exact source/count/hash/scope/diff/trace-no-op verification、crate tests、
  format、workspace Clippy/tests、5 CLIsをPASS。
- [x] independent final read-only quality reviewを**NO FINDINGS**、全9
  hard gates PASS、score capなし、valid `98/100`
  （`20/20/15/14/10/10/5/4`）でcomplete。
- [x] synchronized documentation scopeだけをstageしcached diffをinspect。
- [x] dedicated B3C documentation-only commit
  `ea48ffc4fa586ac6d0813cd23a6b1d9b571087b2`を作成。
- [x] clean ahead-15/behind-0 post-commit/stash invariantsとfresh B3C
  implementation inventoryを
  verify。

## Checker Task 258B3M2B2B3C implementation ledger

- [x] prerequisite
  `ea48ffc4fa586ac6d0813cd23a6b1d9b571087b2`をclean
  ahead-15/behind-0、stash fingerprint `f65cf4a...` unchangedでclose。
- [x] lower-stage prerequisite不要をconfirmし、frozen checker 3 + runner 4
  source consumersだけをimplement。
- [x] 両`source_set_term.rs`、public API/error/debug、dependency、authority
  artifacts、semantics、全existing active-corpus route/outcomeをpreserveし、
  frozen private dormant exact selector branchだけを追加。
- [x] 110-byte、52-node/root-51 choice witnessと
  `32/55/39/72/62/21`をexact checker 4 + runner 5 testsで実装。
- [x] resolver replay、exact upper stage prefix、generic-guard rejectionで
  medium `test_gap` 2件をremediate。
- [x] 両enumeration siblingをpreserveしB3A-hard-coded B3C
  `source_drift`/`test_gap`をremediate。
- [x] repeat test-sufficiency/implementation reviewsを**NO FINDINGS**で
  complete。
- [x] focused checker `4/4`、runner `5/5`、runner library `461`、formatを
  PASS。
- [x] final checker/runner count、size、production/test-list hashes、
  unchanged 5 CLI hashes、deliberate trace no-opをrecord。
- [x] workspace Clippy/testsとfinal count/hash rerunをcomplete。
- [x] final source/docs consistencyとindependent quality reviewをcomplete。
- [x] exact 39 synchronized task filesをstageしcached diffをinspect。
- [x] implementation commit
  `7988a50934656ff90b31e06b883225f86196103b`をcreate。
- [x] clean ahead-1/behind-0 post-commit stateとunchanged stashをverifyし、
  external origin movementを`repo_metadata_conflict`としてreportのみ。
- [x] fresh inventoryでB3D qua witnessをselect。

## Checker Task 258B3M2B2B3D frozen-contract ledger

- [x] B3Cを
  `7988a50934656ff90b31e06b883225f86196103b`、clean worktree、
  current originに対するahead-1/behind-0、unchanged stashでclose。
- [x] comprehension-versus-`qua` task-decomposition
  `design_drift`をresolveし、strictly smallerなqua profileをselect。
- [x] final-LF `109`-byte/hash、24-token、54-node/root-53 source、
  exact sites、local resolver owner/label provenanceをfreeze。
- [x] Task-48 `2/1/0`、Task-252 `5/4/1`、Tasks 253/254 empty、
  Task-255 `1/0/0/1/0/1/2`、Task-256
  `2/0/0/0/0/0/0/4/4`、Task-258 `1/2/2/2/2`、witness `1/0`を
  freeze。
- [x] exact ownership/unowned partition、complete graph、
  `QuaBase -> Primary(2)`、witness-to-SetTerm edgeをfreeze。
- [x] existing SetTerm APIsとfuture source consumers exact 7をreuseし、
  両`source_set_term.rs` ownersと全authority artifactsをforbid。
- [x] exact checker 4/runner 5 test namesとexhaustive byte/LF、
  `54 x 4`+root、resolver、`32/70/44/72/62/21`、
  family-order、replay/rollback/clone matricesをfreeze。
- [x] parser/Task-255 fixtures、expectations、sidecars、trace
  status/count/tests、active behavior、semantic deferralsをpreserve。
- [x] repeated specification、test-sufficiency、implementation-boundary、
  source/documentation consistency reviewsをno findingsでcomplete。
- [x] documentation-only count/hash/scope/no-op verificationをPASS。
- [x] 全hard gatesとvalid score `>=90/100`でfinal read-only quality reviewを
  complete。
- [x] synchronized 32-document scopeだけをstageしprerequisite commit
  `43af562c2cb84e72658cee059abbe7543ee73fe7`をcreate。
- [x] clean ahead-2/behind-0 post-commit stateとunchanged stashをverify。
- [x] B3D implementationをfresh inventoryし、lower-stage prerequisite
  不要をconfirm。

## Checker Task 258B3M2B2B3D implementation ledger

- [x] frozen checker 3 + runner 4 Rust consumersだけをimplementし、両
  `source_set_term.rs` ownersをpreserve。
- [x] exact checker 4 + runner 5 testsと
  `32/70/44/72/62/21` field matricesをimplement。
- [x] exact source/resolver/owner/graph、all-family isolation、
  replay/rollback/clone、empty semanticsをcover。
- [x] test-sufficiency reviewを**NO FINDINGS**でcomplete。
- [x] focused `4/4 + 5/5`、checker/runner packages
  `406+15` / `466+3/14/137/2/21`、format、full ClippyをPASS。
- [x] final module sizes、production/test hashes、unchanged 5 CLI hashes、
  authority/trace/active/semantic no-opをrecord。
- [x] independent implementation reviewを**NO FINDINGS**でcomplete。
- [x] stale implementation-review stateのMedium `design_drift`、
  24-order wordingのLow、EN qua-edge table wordingのLowをfix。
- [x] final source/documentation consistency、bilingual、documentation/
  boundary repeatsを**NO FINDINGS**でcomplete。
- [x] checker/runner packages、format、full Clippy、full workspace tests、
  5 CLI/count/hash final rerunsをPASS。
- [x] independent final read-only quality reviewを**NO FINDINGS**、全9
  hard gates PASS、score capなし、valid `100/100`
  （`20/20/15/15/10/10/5/5`）でcomplete。
- [x] metadata CLI warnings/errors `23/0`とlarge repeated-test diff review
  volumeをnonblocking residualとしてrecord。
- [ ] exact synchronized implementation scopeだけをstageしcached diffを
  inspect。
- [ ] implementation commit 1件をcreate。
- [ ] clean post-commit/stash invariantsをverifyし、next dependency-minimal
  taskをfresh inventory。

## Checker Task 258B3M2B2B3E documentation ledger

- [x] B3D implementation commit
  `08a7d1e3d8c4b3b439325a16e1e139df4a1c18ed`、clean
  `origin/main...HEAD = 0/3` closure、stash
  `f65cf4a13752ec380710814a9ac6392ccb9d75d4` unchangedをrecord。
- [x] condition-free comprehension witnessのexact final-LF 139-byte/hash、
  28-token、60-node/root-59 sourceをfreeze。
- [x] resolver theorem/label/proof-context provenanceとwitness
  owner/source ordinalをfreeze。
- [x] lower profiles `2/1/0`、`5/4/1`、Tasks 253/254 empty、
  `1/0/1/1/0/1/2`、`2/0/0/0/0/0/0/4/4`、Task-258
  `1/2/2/2/2`、witness/names `1/0`をfreeze。
- [x] owner partition Task-252 `{32,34,38,47,49}`、Task-255
  `{16,40,41,43}`、Task-256 `{36,51}`、Task-258 `{54,56}`、B3E
  `{45,46}`をfreezeし、generator segment `42`をunownedに保持。
- [x] `ComprehensionMapper -> Primary(2)`、ordered
  `GeneratorSethood`/`ResultType`、`Witness(0) -> SetTerm(0)`をfreeze。
- [x] exact matrices `32/70/53/72/62/21`、five-family `120` orders、
  checker 4 + runner 5 future test namesをfreeze。
- [x] future implementationをexact seven consumersへ限定し、両
  `source_set_term.rs`、public/authority/trace/active/semantic ownersを
  unchangedに保持。
- [x] generator binding/capture、sethood/result typing、goal/proof/
  theorem semantics、B4/B5をdeferし、lower prerequisite不要をrecord。
- [x] `tests/coverage/spec_trace.toml` no-opと
  `spec_coverage_audit.md` narrative-only ownership updateをrecord。
- [x] specification/documentation reviewを実行し、findingがあれば修正後に
  repeat。
- [x] test-sufficiency、implementation-boundary、source/documentation、
  bilingual consistency reviewsを実行。
- [x] relevant verificationとforbidden-path/no-op checksを実行。
- [x] independent final quality reviewを全hard gatesとvalid score
  `>=90/100`でcomplete。
- [x] exact synchronized documentation scopeだけをstageしcached diffを
  inspect。
- [x] B3E documentation commit
  `8075000bf79be3fdea6b22f366fb6d9e59781fe7`をcreate。
- [x] clean post-commit/stash invariantsをverifyし、B3E implementationを
  fresh inventory。

## Checker Task 258B3M2B2B3E implementation ledger

- [x] exact checker 3＋runner 4 consumersだけを実装し、両
  `source_set_term.rs`を保持。
- [x] checker 4/runner 5 testsと`32/70/53/72/62/21`を追加。
- [x] post-auth negativesへsame-provenance successful coherent Task-255
  handoffを使い、repeated failure/clean replayを検査。
- [x] test-sufficiency/implementation re-reviewを**NO FINDINGS**で完了。
- [x] focused `4/4 + 5/5`、libraries `410/471`をPASS。
- [x] final module/production/test-list hashesとauthority/trace/active/
  semantic/public-API no-opを記録。
- [x] 3件の`design_drift`修正後、source/docs、bilingual、boundary
  consistency re-reviewを**NO FINDINGS**で完了。
- [x] independent final qualityを**NO FINDINGS**、全9 gates PASS、
  capなし、valid `100/100`（`20/20/15/15/10/10/5/5`）で完了。
- [x] focused/package、fmt、full workspace Clippy、root tests、5 CLI、
  count/hash/scope/forbidden/stash gatesをPASS。
- [x] exact implementation scopeをstageしcached diffをinspect。
- [x] B3E implementation commit
  `e4479691db3b0a8785bb16e94d386bd71a394274`をcreate。
- [x] clean ahead-5/behind-0 post-commit state、unchanged stash
  fingerprint `f65cf4a13752ec...`をverifyし、Task 258B4Aをfresh
  inventory。

## Checker Task 258B4A documentation prerequisite

- [x] B4をB4A explicit-universal、B4B connective/grouping、
  B4C restricted/existential/nested rootsへdecomposeし、B5 visibilityを
  retain。
- [x] canonical formula/theorem authority、parser/resolver fixtures、
  Tasks 252/256/257 public APIをauditし、lower-stage prerequisiteなしを
  confirm。
- [x] active 79-byte fixture reuseを`test_expectation_drift`と分類し、
  distinct private 80-byte/double-LF source/hashをfreeze。
- [x] 26 nodes/root 25、resolver contribution 0/origin `[2,0]`、
  lower `2/2/0`、`1/0/0/0/0/0/2/2`、`1/0/1/1/1/0/2`、
  `1/2`、`2/1/4`、upper `1/1/1/0/1`をfreeze。
- [x] `Composite(0)`、zero input facts、optional lower fingerprints、
  dedicated producer、paired typed installer、final revalidationをfreeze。
- [x] exact eight future source consumers（checker 3/runner 5）、single
  crate-private Task-257B1 helper visibility seam、checker 4/runner 5
  tests、near-miss matrices、cross-family edges、semantic deferrals、
  narrative-only audit/trace no-opをfreeze。
- [x] repeated specification/documentation reviewを**NO FINDINGS**でcomplete。
- [x] documentation-only scope、forbidden-artifact、count/hash、CLI、
  crate/workspace、stash verificationをPASS。
- [x] independent final qualityを全9 hard gatesとvalid score
  `>=90/100`でcomplete。
- [x] synchronized documentation scopeだけをstageしcached diffをinspect。
- [x] dedicated B4A documentation prerequisite commit
  `9da1ac13e811c78359d8d64e740832b2a30dae24`をcreate。
- [x] clean ahead-6/behind-0 post-commit state、unchanged stash
  fingerprintをverifyし、B4A implementationをfresh inventory。

## Checker Task 258B4A implementation ledger

- [x] frozen checker 3/runner 5 consumersだけをimplement。
- [x] exact checker 4/runner 5 testsとfrozen lower/upper mutation、
  coherent-near-miss、cross-family、replay、clone matricesを追加。
- [x] lower Task-257 `UnassignedStatement` ownershipを保持し、Surface root
  25とrootless lower typed arenaを区別。
- [x] separate test-sufficiency/implementation reviewsを
  **NO FINDINGS**でcomplete。
- [x] focused checker `4/4` / runner `5/5`をPASSし、libraries
  `414/476`、production `23/139828` / `30/55109`をmeasure。
- [x] specifications、existing corpus/expectation/sidecar/trace artifacts、
  active behavior、semantic tables、public runner schemasをpreserve。
- [x] source/documentation、bilingual、boundary consistency reviewを
  **NO FINDINGS**でcomplete。
- [x] focused/package、formatting、full Clippy、workspace、5 CLI、
  count/hash/scope/forbidden/stash verificationをPASS。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  capなし、valid `100/100`でcomplete。
- [x] exact B4A implementation scopeだけをstage/inspect。
- [x] dedicated B4A implementation commit
  `662adbde71e665ab37504ac476e94c935c493535`をcreate。
- [x] clean ahead-7/behind-0 post-commit state、unchanged stash
  fingerprintをverifyし、B4Bをfresh inventory。

## Checker Task 258B4B documentation prerequisite

- [x] Task-257B2 connective/grouping rootだけをselectし、B4C/B5をretain。
- [x] active 166-byte upper-route reuseを`test_expectation_drift`と分類し、
  private 167-byte/double-LF source、hash、124 Surface nodes/root 123、
  resolver contribution 0/origin `[2,0]`をfreeze。
- [x] lower `16/0/16`、`8/0/0/0/0/0/0/16/16`、
  `8/6/1/1/1/7/9`、`8/0`、binding `2/1/4`、rootless 124-node arena、
  exact 42/1/81 ownership partitionをfreeze。
- [x] upper `1/1/1/0/1`、両`Composite(0)` edges、zero input facts、
  statement spelling/context、resolver provenance、subtree exclusionsを
  freeze。
- [x] B4A public API/debug grammarをreuseし、future consumers 7件とchecker
  4/runner 5 testsだけをfreezeし、全lower owner、
  source-formula-composition helper、corpus、sidecar、trace editをforbid。
- [x] cross-family/profile isolation、error precedence、semantic
  deferrals、baseline/projection、narrative-only audit impact、exit
  criteriaをfreeze。
- [x] repeated specification/documentation reviewを**NO FINDINGS**で
  complete。
- [x] documentation-only scope、forbidden-artifact、count/hash、CLI、
  crate/workspace、stash verificationをPASS。
- [x] independent final qualityを全9 hard gates、valid score
  `>=90/100`でcomplete。
- [x] synchronized Task-258B4B documentationだけをstageしcached diffを
  inspect。
- [x] dedicated B4B documentation prerequisite commit
  `b8a7b8257a682f7c88de943ceaa35b67c0585bc4`をcreate。
- [x] clean ahead-8/behind-0 post-commit state、unchanged stash
  fingerprintをverifyし、B4B implementationを
  fresh inventory。

## Checker Task 258B4B implementation ledger

- [x] frozen checker 3/runner 4、合計7 consumersだけをimplement。
- [x] private 167-byte source、raw label-free resolverからenriched
  `1/1/1/1/0`へのhandoff、Task-257B2 lower transactionをauthenticate。
- [x] rootless 124-node arena、exact `42/1/81` ownership、upper
  `1/1/1/0/1`、statement/candidateの両`Composite(0)`をpreserve。
- [x] B1/B4AとB2/B4Bだけをpairし、B4B private telemetry
  `0/0/[]`、B4A `1/1/[1,1]`をfail-closedに保持。
- [x] active 166-byte sourceをlower-onlyに保持し、public API、
  semantics、corpus、expectation、sidecar、traceを変更しない。
- [x] exact checker `4/4` / runner `5/5` focused testsをPASS。
- [x] separate test-sufficiency/implementation reviewsを
  **NO FINDINGS**でcomplete。
- [x] final source/documentation、bilingual、boundary consistency reviewを
  repeatし、
  **NO FINDINGS**でclose。
- [x] focused/package、`cargo fmt --check`、full Clippy、workspace、
  5 CLI、全count/hash/scope/forbidden/stash verificationをcomplete。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  capなし、valid `100/100`
  （`20/20/15/15/10/10/5/5`）でcomplete。
- [x] exact Task-258B4B implementation scopeだけをstageしcached diffを
  inspectし、dedicated implementation commit
  `752c17ae7d552d5268d1028612b8174e480b6f3e`をcreate。
- [x] report-only external origin movement後のclean ahead-1/behind-0
  post-commit state、unchanged stash fingerprintをverifyし、Task 258B4Cを
  fresh inventory。

## Checker Task 258B4C documentation prerequisite

- [x] Task-257B3 restricted-universal、existential、nested、
  implicit-reserve rootsだけをselectし、B5をretain。
- [x] active 138-byte upper-route reuseを`test_expectation_drift`と分類し、
  private 139-byte/double-LF source、hash
  `36e5a68a92451590644951838a9af8926212bd78f88d1f90563f12b650b161c1`
  をfreeze。
- [x] Surface 66/root 65、theorem 62 `19..137`、label 6 `27..65`、
  outer composite 60 `67..136`、raw resolver `1/0/1/1/0`、
  origin `[2,1]`、contribution 0 anchor `0..18`、enriched
  `1/1/1/1/0`をfreeze。
- [x] lower binding `4/4/0`、primary `6/6/0`、atomic
  `3/0/0/0/0/0/0/6/6`、composite `3/0/1/3/3/2/6`、composition
  `3/6`、lower-owned sites 24件、upper theorem site 1件、unowned
  sites 41件をfreeze。
- [x] upper `1/1/1/0/1`、context visibility `[0]`、input facts 0、
  両`Composite(0)` links、telemetry `2/2/[2,2,4,4,4,4]`をfreeze。
- [x] one-LF-only Task-257B3 selectorをbounded `source_drift`と分類し、
  B4C upper implementation前のseparate lower-stage prerequisiteを必須化。
- [x] prerequisiteをrunner `type_elaboration/source_formula.rs`と
  `runner/tests/type_elaboration/source_formula_composition.rs`だけにbound
  し、exact 138/139-byte routesをadmit、zero/three LFをreject、production
  `source_formula_composition.rs`をunchangedにする。
- [x] later upper scopeをB4Bと同じconsumers 7件、exact B1/B4A、
  B2/B4B、B3/B4C pairing、unchanged public API/debug/error、
  authority/trace no-op、subtree exclusion、semantic deferrals、audit
  narrative-only effect、tests、baseline impact、exit criteriaにfreeze。
- [x] repeated specification/documentation reviewを**NO FINDINGS**で
  complete。
- [x] documentation-only scope、forbidden-artifact、count/hash、CLI、
  crate/workspace、diff、stash verificationをPASS。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  capなし、valid `100/100`（`20/20/15/15/10/10/5/5`）でcomplete。
- [x] synchronized Task-258B4C documentationだけをstageしcached diffを
  inspectしてdedicated documentation prerequisite commit
  `3c723316ae632a867d29e8f4fc36348be30df202`をcreate。
- [x] clean post-commit/stash invariantsをverifyし、mandatory lower-stage
  prerequisiteをfresh inventory。

## Checker Task 258B4C lower-stage prerequisite ledger

- [x] authority、exact 138/139-byte routes、selector ownership、tests、
  counts、hashes、clean documentation commitをfresh inventory。
- [x] prerequisite specification reviewを**NO FINDINGS**でclose。
- [x] frozen runner selector/composition test ownerだけを変更し、
  production `source_formula_composition.rs`を変更しない。
- [x] test sufficiency、implementation、source/document consistencyを
  separateに**NO FINDINGS**までreview。
- [x] focused/package/workspace、formatting、Clippy、CLI、count/hash、
  scope、forbidden-artifact、audit-no-op、stash gatesをPASS。
- [x] independent final quality `>=90/100`、two-file staging/cached diff、
  dedicated prerequisite commit
  `42356f38ed0e679d7b878caf0e647c6aa8148d82`をcomplete。
- [x] clean post-commit/stash invariantsをverifyし、B4C upper
  implementationをfresh inventory。

## Checker Task 258B4C implementation ledger

- [x] frozen checker 3 / runner 4 source/test filesだけを変更。
- [x] exact source/Surface/raw/enriched resolver provenance、lower profiles、
  rootless `24/1/41`、upper `1/1/1/0/1`、両 `Composite(0)` link、`[0]`、
  no input fact、telemetry `2/2/[2,2,4,4,4,4]`をauthenticate。
- [x] exact checker `4/4` / runner `5/5`をPASSし、independent
  test-sufficiency/implementation reviewを**NO FINDINGS**でclose。
- [x] libraries `422/488`、production `23/141952` と `30/56872`、
  owner sizes、production/test-list hashesをmeasure。
- [x] public API、active/corpus authority、expectation、sidecar、
  trace/coverage state、semantic output、lower-owner boundaryをpreserve。
- [x] Medium `design_drift` 1件をcorrect後、final
  source/documentation、bilingual、boundary reviewを**NO FINDINGS**で
  complete。
- [x] broad workspace/fmt/Clippy/CLI/count/hash/scope/stash gatesをPASSし、
  全frozen count/hashをreproduce。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  capなし、valid `100/100`（`20/20/15/15/10/10/5/5`）でcomplete。
- [x] Task-258B4Cだけをstage/inspectしimplementation commit
  `50ab1ebc747e912fff1f0cf111832e3c2c81ba01`をcreate。
- [x] clean post-commit state、unchanged protected stash、next
  dependency-ready taskのfresh inventoryをverify。

## Checker Task 258B5A frozen-contract documentation prerequisite

- [x] stale B4C ledger/unsafe B5 aggregationを`design_drift`、missing
  imported/confinement active routeをbounded `test_gap`、absent B5A
  implementationをimmediate-next-task-owned bounded `source_drift`にclassify。
- [x] exact 185-byte source/hash、93-node/root-92 frontend、resolver、
  Binding/252/256/258 base/reference rows、20/73 ownershipをfreeze。
- [x] proof label scope `[0]`、descendant citation scope `[0,1]`、
  local-only provenance、exact seven consumers、checker 4 tests、runner
  5 testsをfreeze。
- [x] later B5B imported-public/B5C negative-confinement workをsplitし、
  semantic/public API/corpus/expectation/sidecar/trace boundaryをpreserve。
- [x] independent specification/documentation、test-sufficiency、
  source/documentation boundary、bilingual reviewsを**NO FINDINGS**で
  complete。
- [x] checker/runner/workspace tests、fmt、Clippy、five CLI、exact
  32-document scope、全counts/hashes、authority no-op、repository/stash
  invariantsをreproduce。
- [x] repeated independent final qualityを**NO FINDINGS**、全9 hard gates
  PASS、capなし、valid `100/100`
  （`20/20/15/15/10/10/5/5`）でcomplete。
- [x] synchronized B5A documentationだけをstageしてprerequisite commit
  `59021f764f146d669f84877042f0512882c9c5ff`をcreateし、post-commit
  invariants後にimplementationをfresh inventory。

## Checker Task 258B5A implementation ledger

- [x] frozen three checker/four runner source/test filesだけを変更。
- [x] exact source/Surface/raw/enriched resolver provenance、Binding/
  Task-252/Task-256 lower profile、Task-258 base/reference row、`20/73`
  ownership、label `[0]`、citation `[0,1]`、resolver node 82をauthenticate。
- [x] full resolver-node-kind identityとatomic rollback/replayを含め、
  unchanged B1またはexact B5A paired stateだけをadmit。
- [x] B5B/B5C ownership、public API、active/corpus authority、expectation、
  sidecar、trace status/count/backlink、coverage credit、diagnostic、全
  semantic outputをpreserve。
- [x] frozen focused checker `4/4`、runner `5/5`、preserved B1 runner
  `6/6` testsをrun。
- [x] separate test-sufficiency/implementation reviewを**NO FINDINGS**で
  complete。
- [x] final source-documentation consistency reviewを**NO FINDINGS**で
  complete。
- [x] checker `426/426`、runner `493/493`、full workspace test、formatting、
  exact Clippy、five CLI、count/hash、diff gateをPASS。
- [x] final scope/forbidden-artifact、repository-state、stash gateをcomplete。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  capなし、valid `100/100`（`20/20/15/15/10/10/5/5`）でcomplete。
- [x] Task-258B5Aだけをstage/inspectしimplementation commit
  `4a79116c1a6f71155e4f366950fee8335b4dc8f1`を作り、post-commit
  invariantをverifyしてnext taskをfresh inventory。

## Checker Task 258B5B frozen-contract documentation prerequisite

- [x] stale B5A post-commit ledgerをcorrectし、missing frozen B5B/API
  ownershipを`design_drift`、missing opt-in imported-label populationを
  lower-owned `source_drift`、missing active B5B corpusをbounded
  `test_gap`とclassify。
- [x] exact 146-byte source/hash、57-node/root-56 frontend/resolver
  identities、raw `1/0/1/1/0`、opt-in `8/1/1/3/1`、lower profile、
  upper `1/2/2/2/2 + 0/1`、`8/49` ownershipをfreeze。
- [x] separate two-file lower prerequisite、one opt-in public/exported
  theorem label、exact two lower tests、unchanged default augmentation
  routeをfreeze。
- [x] `SourceStatementCitationTarget::{Local, Imported}`、
  `SimpleImported`、local-label row 0、imported projection/debug branch、
  exact seven upper consumers、four checker/five runner testsをfreeze。
- [x] B5C/semantic deferralと全specification、fixture、expectation、
  sidecar、trace status/count/backlink/credit、active outcome、public
  runner-schema boundaryをpreserve。
- [x] independent specification/documentation、test-contract、
  source/documentation boundary、bilingual reviewを**NO FINDINGS**でcomplete。
- [x] focused/crate/workspace test、formatting、Clippy、five CLI、exact
  count/hash/scope、authority no-op、repository-state、stash gateをreproduce。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  capなし、valid `100/100`（`20/20/15/15/10/10/5/5`）でcomplete。
- [x] synchronized B5B documentationだけをstageしprerequisite commit
  `141dc44a757555e8d4837756515e1577f672348b`を作り、post-commit
  invariantをverifyしてmandatory lower-stage prerequisiteをfresh inventory。

## Checker Task 258B5B lower-stage prerequisite

- [x] docs commit後、
  `crates/mizar-test/src/runner/import_fixtures.rs`とexisting statement
  test leafだけを変更。
- [x] normal augmentation/existing callerを変更せずcrate-private opt-in
  `Ref` label augmentationを追加し、exact two frozen lower testsと全protocol
  gatesをseparate commit
  `46dd9db56ced2fcc57799420de9d5fed06f284f5`でPASS。

## Checker Task 258B5B upper implementation

- [x] lower commitとfresh inventory後、frozen three checker/four runner
  consumersだけを変更。
- [x] exact imported target/API、profile、resolver replay、mutation matrix、
  four checker/five runner testsをimplementし、全review/hard gatesを
  pre-commit状態でPASS。
- [x] classified `design_drift`/adjacent-profile `source_drift`をrepair後、
  separate test-sufficiency、implementation、final source/documentation
  consistency reviewを**NO FINDINGS**でcomplete。
- [x] focused/crate/workspace、formatting、exact Clippy、five-CLI、
  count/hash、forbidden-artifact、repository-state、stash gateをPASS。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  capなし、valid `100/100`（`20/20/15/15/10/10/5/5`）でcomplete。
- [x] Task-258B5B upperだけをstage/inspectしdedicated implementation
  commit `f27d2c9169b08078f00b75c4a57f94e30fa28f59`を作り、clean
  post-commit invariantをverifyしてnext dependency-ready taskをfresh
  inventory。

## Checker Task 258B5C frozen-contract documentation prerequisite

- [x] absent production proof-label source walkをmedium `source_drift`と
  potential `boundary_violation`、placeholder/stale statusを
  `design_drift`、absent active fixtureを`test_gap`、unspecified public
  resolver diagnostic codeをlow deferred `spec_gap`とclassify。
- [x] exact 173/197-byte source/hash、normal 61/root-60・71/root-70
  Surface identity、label/citation range、proof scope `[0]`、`[0,0]`、
  `[0,1]`、source-statement ordinalをfreeze。
- [x] known-absent structural Surface-to-resolved providerをcollector後の
  conditionでなくauthority十分なresolver prerequisiteとしてrecord。
- [x] R-032A `SurfaceResolvedArena` lowering/accessor、same-index structural
  invariant、fail-closed validation/overflow error、dedicated
  `resolved_ast.rs` / `resolved_ast/tests.rs` ownership、sole
  `tests/lint_policy.rs` R-026 `SurfaceResolvedArenaError` owning-spec entry、
  testsをfreeze。
- [x] preflight two-Rust-file omissionをsemantic `spec_gap`なしのHigh
  `design_drift`と分類し、separate同期docs-only correction、fresh inventory、
  exact three-Rust-file R-032A implementationを要求。
- [x] R-032B exact `Result`-returning collector API、narrow source
  inclusion/exclusion、generic theorem-root scope、completion visibility
  ordinal 3、exact origin path、exact `labels.rs` / `labels/tests.rs`
  ownership、sole `tests/lint_policy.rs` R-026
  `ProofLabelSourceCollectionError` / `labels.md` owning-spec decision、
  positive/negative/provenance/cross-theorem testsをfreeze。
- [x] same-`'a` ast/resolved storage、validation-only module、owned
  namespace/contribution、`Self` return、`SurfaceNodeId` state/key/overflow
  error、module-global one-based ordinal、`ConclusionStatement` chain、
  canonical `proof-step-v1` identityをfreeze。
- [x] S-026 documentation、S-026 implementation、R-032A lint-policy
  documentation correction、R-032A implementation、R-032B lint-policy
  documentation correction、R-032B implementation、active
  declaration-symbol fixture/runner/traceというeffective seven-task
  dependency orderをfreeze。
- [x] future two fixture/sidecar contract、detail key
  `declaration_symbol.label.proof_scope_confinement`、empty public
  diagnostic codes、two trace ids、future count impactをfreeze。
- [x] source-byte+normal-AST runner selection、exact shared resolver/
  contribution-0 authentication、separate private input/confinement detail、
  expectation-copy guard、measured 48-file docs scopeをfreeze。
- [x] checker profile/DTO/reference/citation row、binding context、keyed
  semantic label-resolution result、typed/final installation、全cross-family
  edge、全semantic outputをexclude。
- [x] independent specification、test-contract、boundary、
  source/documentation、bilingual reviewを**NO FINDINGS**でcomplete。
- [x] unchanged focused/crate/workspace、formatting、Clippy、five-CLI、
  count/hash、forbidden-artifact、repository-state、stash gateをreproduce。
- [x] independent final qualityを全9 hard gates PASSかつvalid
  `90/100`以上でcomplete。
- [x] synchronized B5C design documentationだけをstageしてone
  prerequisite commitを作り、post-commit invariant後に
  `mizar-resolve` R-032A prerequisiteをfresh inventory。
- [x] R-032B exact
  `Root -> CompilationUnit -> ItemList -> direct TheoremItem -> direct
  ProofBlock` upper chain、Root/CompilationUnit exact-one normal child、
  direct-normal theorem scan、excluded formのno ordinal/no descent、
  positive-edge/missing/additional/wrong/direct-relocation/`VisibleItem`/
  mixed-list testをfreeze。
- [x] env/module、derived namespace、exact one id-0 LocalSource record/
  source id、全projection provenance fieldとcomplete independent
  input-only mutation matrixのrunner authenticationをfreeze。
- [x] source-bytes-plus-normal-AST selection、expectation non-selection、
  empty public code、exact 48-file scopeを維持。

## B5C R-032A preflight overlay

- [x] separate mizar-syntax S-026 frozen-documentation commitを完了。
- [x] exact dense accessor と passing review/verification gate を持つ
  separate S-026 implementation を完了。
- [x] dedicated commit 後に R-032A を fresh inventoryし、mandatory R-026
  enum-decision owner omissionをHigh `design_drift`として特定。
- [x] separate R-032A lint-policy docs correctionをfindings-free review、
  full verification、valid `100/100` final qualityまで完了。
- [x] post-commit invariant checkとfresh inventoryをR-032A
  implementation前に完了。
- [x] checker consumer / B5C artifactを変更せずexact resolver R-032A
  implementationを完了。次のlower prerequisiteはR-032B。
- [x] lower prerequisite 中も全 frozen checker consumer、B5C fixture/
  expectation/trace、public diagnostic、semantic deferral を維持。

## B5C R-032B lint-policy preflight overlay

- [x] R-032A dedicated commit後にR-032Bをfresh inventoryし、mandatory
  R-026 enum-decision owner omissionをsemantic `spec_gap`または
  `test_gap`ではないHigh `design_drift`として特定。
- [x] current docs-only correctionをexactly 31 design files、すなわち
  eight paired resolver families、four paired checker families、three paired
  `mizar-test` families、global design TODOにfreeze。
- [x] separate R-032B lint-policy documentation correctionを
  findings-free review、full verification、dedicated commit、post-commit
  fresh inventoryまで完了してからR-032B implementationへ進む。
- [x] correction後に限りR-032Bをexactly
  `crates/mizar-resolve/src/labels.rs`、
  `crates/mizar-resolve/src/labels/tests.rs`、
  `crates/mizar-resolve/tests/lint_policy.rs`でimplementし、最後のfileには
  R-026 `ProofLabelSourceCollectionError` / `labels.md` owning-spec
  decisionだけを追加。
- [x] initial/fresh test gapと全implementation findingをfixし、final fresh
  test-sufficiency、implementation、source/documentation rereviewを
  **NO FINDINGS**で完了し、全focused/full/count/hash/scope verification gateを
  PASSする。このlower taskではchecker consumerもB5C artifactも追加しない。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gates PASS、
  score capなし、valid `100/100`
  （`20/20/15/15/10/10/5/5`）で完了する。
- [x] task-only restaging/cached-diff reviewとdedicated R-032B commit
  `b3a7e79a6b60db2974e911c69bb56ff5f4609064`を完了し、post-commit
  invariant/fresh inventoryをverifyする。
- [x] 全frozen checker consumer、B5C fixture/expectation/trace、public
  diagnostic code、semantics、`spec_coverage_audit.md`を維持し、mapping、
  owner、deferral、coverage creditを変更しない。

## Checker Task 258B5C active implementation

- [x] private declaration-symbol consumerでexact spec-derived fail case 2件と
  covered trace row 2件を追加し、public codeをemptyに維持。
- [x] omitted metadata count consumerのfour `5 -> 7` assertionを修正し、
  `test_expectation_drift`/scope `design_drift`に分類。
- [x] 全checker source/API/semantic resultをno-opに保ち、R-G007内の
  confinement requirement 2件だけをclose。
- [x] findings-free test/implementation/source-documentation reviewと
  focused/crate/workspace/count/hash verification gateを完了。
- [x] final qualityを**NO FINDINGS**、全9 hard gates PASS、score capなし、
  valid `100/100`（`20/20/15/15/10/10/5/5`）で完了。
- [ ] task-only commit、post-commit invariant、next-task fresh inventoryを
  完了。

## Checker Task 259 Frozen-Contract Documentation Prerequisite

- [x] active B5Cを
  `33ac57e96f048dc40559565f54369cac854409a7`でcloseし、clean post-commit
  invariantとprotected-stash identityを確認し、fresh canonical/API
  inventoryでTask 259を選択する。
- [x] exact 165-byte/final-LF pass sourceとSHA-256、71-row frontend
  identity、three-shell/two-projection resolver profile、exact range、
  source-order/same-block predicate/property associationを凍結する。
- [x] exact `1/2/1/1/1` cardinality、Task-248/249/252/256 fingerprint、
  Tasks-253--255/257/258 family absent、immutable typed/final installation
  を持つ5 syntax-free dense tableを凍結する。
- [x] empty assumptionsとopaque deterministic goal/provenanceを持つexactly
  one pending `PredicatePropertyCorrectness` obligationを凍結し、guardを
  separately preserveしてそのFOL property-VC compositionをdeferする。
- [x] property proof subtreeをTask 259ではconsumeせずTask 272用に保持する。
  Task 258へ移したりcomputation justificationをaccepted/dischargedとして
  扱わない。
- [x] 現行Task-248 exact-profile rejectionをbounded `source_drift`/
  dependency `design_drift`に分類する。private `BindingEnv`
  reconstructionを禁止し、Task 259 implementation前に別Task-248
  profile-extension documentation/implementation commitを必須とする。
- [x] mixed predicate-plus-functor Task-260 gapを不変にしたまま、future
  pass sidecar/trace intentとtest/corruption/installation/determinism
  matrixを凍結する。
- [x] findings-free documentation、test-contract、implementation-boundary、
  source/documentation reviewを繰り返し、full docs/count/hash verification、
  全9 hard gate PASSのindependent final quality `100/100`をcompleteする。
- [x] task-only commit
  `d5294b8f4be46a420bbdfa2fc4062384be983ce0`とpost-commit fresh
  inventoryをcompleteする。
- [x] implementation前にseparate Task-248 two-parameter profile-extension
  documentation prerequisiteをfresh-inventoryする。

## Checker Task 248 Two-Parameter Profile-Extension Documentation Prerequisite

- [x] implemented Profile Aをone shadow link、recovered-empty branch、active
  route、public error、debug grammarを含めexactにpreserveする。
- [x] normal-only Profile B、すなわちone top-level definition shell、no
  reserve、ordered direct `x`/`y` parameter/bare `set` range、scope `[0]`、
  dense bindings `0/1`、no shadowをfreezeする。
- [x] syntax-free checker validation、real shell provenance、private
  shared-`TypedArena` extractor、exact `1/2/2/2/2/2/0` table、完全な
  guard/predicate/property/justification subtree exclusionをfreezeする。
- [x] `design_drift`、bounded `source_drift`、`test_gap`をclassifyし、
  Task-259-private binding reconstructionを禁止して全semantic deferralを
  preserveする。
- [x] later exact five Rust files/four runner tests、projected runner
  `504 -> 508`、unchanged corpus/CLI/trace metadata、fresh-hash requirementを
  freezeする。
- [x] findings-free documentation/test-boundary/source-consistency reviewと
  full docs-only focused/crate/workspace/count/hash verificationをcompleteする。
- [x] independent final qualityを**NO FINDINGS**、全9 hard gate PASS、
  score capなし、有効な `100/100`
  (`20/20/15/15/10/10/5/5`)でcompleteする。
- [x] exact-scope staging、one dedicated documentation commit
  `f9b47375acc18acebf56a69f5d8a7edec539c2be`、
  clean/stash-invariant post-commit inventoryをcompleteする。
- [x] separate Task-248 extensionをone logical task/commit
  `ca54135f36c9fecfc02c2b8120ec4e63e8c6ca36`でfresh-inventory/
  implementし、Task 259 implementationへ戻る。

## Checker Task 259 Frozen-Contract Correction Prerequisite

- [x] pre-implementation findingをmissing public-enum/lint consumer policy、
  implicit immutable output/debug ABI、stale Task-248 closure stateという
  nonblocking `design_drift`にclassifyする。
- [x] exact five output row field/getter、five immutable table API、four
  complete dependency fingerprint、handoff/producer surface、complete
  line-family debug grammarをsynchronized EN/JAでfreezeする。
- [x] `lib.rs`、typed/final、obligation serializer、lint policy、runner
  facade/new route/new test leaf、metadata count assertion、fixture/sidecar/
  trace row各1件、derived audit recordをexact future consumerとしてfreezeする。
- [x] Task-248 commit `f9b47375` / `ca54135f`、Profile-B readiness、runner
  `508`、current production/test-list hash、Task-259 implementationがnext
  dependency-ready taskであることを記録する。
- [x] specification/documentation reviewをfindingsなしまでrepeatし、
  production source、fixture、sidecar、expectation、trace status/count、
  Cargo metadataを変更せずdocs-only verificationをcompleteする。
- [x] correction documentだけをstageし、dedicated commit
  `e202dd70bf4e97ddb53c1275b49e667b6a77f7a0`を作成する。そのclean
  post-commit state、`main`が`origin/main`より1 commit ahead、protected stash
  `f65cf4a13752ec380710814a9ac6392ccb9d75d4`不変をverifyしてからTask-259
  implementationをfresh-inventoryする。

## Checker Task 259 active implementation

- [x] exact public predicate-definition module、five dense `1/2/1/1/1`
  table、baseline-preserving projection、pending
  `PredicatePropertyCorrectness` obligation 1件をimplementする。
- [x] typed/final ownerでhandoffとcomplete obligation tableをatomicにinstallし、
  second final input/getter、fact、proof、VC、acceptance、Task-260 ownershipを
  追加しない。
- [x] private Task-248 -> 249 -> 252 -> 256 -> 259 runner routeを通じてexact
  165-byte pass artifact、same-stem sidecar、covered trace backlink各1件だけを
  activateする。
- [x] checker unit-test 5件をprivate non-integration test
  supportへ移し、lint policyを弱めずpublic resolver test APIも追加せず、
  production checker sourceをsyntax-freeに保つ。
- [x] independently discovered source-statement active-count assertion 2件だけを
  bounded `test_expectation_drift` plus `design_drift`としてclassify/correctし、
  empty-selection isolation assertion 2件をpreserveする。
- [x] focused checker `5/5`、focused runner `4/4`、checker `435`、runner
  `512`、resolver `144`、syntax `59`、metadata `137`、formatting、
  checker/runner Clippy、`422/390`、`229/193`、`101/7/199/1`、type
  `254/242`、warnings/errors `23/0`をreproduceする。
- [x] final independent test-sufficiency、implementation、
  source/documentation-consistency reviewを全件no findingsで完了し、全9 hard
  gateをPASS、score capなしのfinal quality score `100/100`を得る。
- [x] Task-259 fileだけをstageしimplementation commit
  `b61be7e567b92d31b3544b86e5c7a68537625743`を作成、clean repository/
  protected-stash invariantをverifyしてTask 260をnextとしてfresh inventoryする。

## Checker Task 260 Frozen-Contract Documentation Prerequisite

- [x] clean post-Task-259 inventoryからTask 260をselectし、origin reference差を
  report-only `repo_metadata_conflict`にclassifyする。
- [x] exact 262-byte source/hash、108-row AST、three-shell/two-functor resolver、
  parameter 2、guard 1、return type 2、equals/means、existence/uniqueness associationをfreeze。
- [x] five dense `2/2/1/2/2` table、Task-248/249/252 requiredおよび
  Task-253--256 optional fingerprint、Task-259 sibling isolationをfreeze。
- [x] Pending `FunctorExistence`/`FunctorUniqueness`、empty assumptions、opaque
  goal/provenance、proof/discharge/acceptance/fact/VC deferralをfreeze。
- [x] Typed/Resolved atomic ownership、runner consumer、新規pass pair/trace intent、
  test 9件、write scope、count projection、exclusionをfreeze。
- [x] review-only specification auditを**NO FINDINGS**までrepeatする。
- [x] docs-only verificationをcompleteし、all nine gates PASS、score capなし、
  final read-only quality `100/100`を得る。
- [x] exact stagingとdedicated docs commit
  `b587038f12f84a77720f6441a000ddb84c7b996f`をcompleteし、fresh
  implementation preflightを実行する。

## Checker Task 249R definition-return documentation prerequisite

- [x] impossible Task-260 `4/4/0` binding-linked profileをnonblocking
  `design_drift` plus lower `source_drift`にclassifyし、fabricated bindingを
  `boundary_violation`として禁止する。
- [x] Chapter 10 §§10.1/10.5 authority、Task 260 sole consumer、exact additive
  ABI/debug、`2/4/0/2` oracle、typed/final ownership、test 4件、exclusion、
  semantic deferral、audit impact、exit criteriaをfreezeする。
- [x] review-only specification auditを**NO FINDINGS**までrepeatし、executable/
  count/hash invariantとall nine hard gateを含むdocs-only verificationを行う。
- [x] synchronized Task-249R docsだけを
  `b292b8002f9656c4ab2a6c3b606743b1bda7d551`としてcommitし、separate
  implementationをfresh-inventoryする。

## Checker Task 249R active implementation

- [x] independent definition-return table/producerだけを`source_type.rs`へ
  実装し、Task-249 application cardinalityとlegacy empty-table byteを保つ。
- [x] frozen checker test exactly 4件を追加し、initial review gapをそのmatrix
  内で閉じ、test-sufficiency/full implementation reviewを**NO FINDINGS**まで
  repeatする。
- [x] checker `439`、runner/resolver/syntax `512/144/59`、metadata `137`、
  unchanged CLI 5本のcount/hash、checker production `24/148143`、fresh
  checker test-list/content hashを再現する。
- [x] focused/module/crate test、lint policy、formatting、full
  warnings-denied Clippy、unconstrained full workspace test、diff checkをPASS。
- [x] wording-only Medium/Low `design_drift`各1件をcorrectし、source/docs
  consistency reviewを**NO FINDINGS**までrepeatする。
- [x] final read-only qualityを**NO FINDINGS**、全9 hard gate PASS、score cap
  なし、`100/100` (`20/20/15/15/10/10/5/5`)でcompleteする。
- [x] exact staging、dedicated implementation commit
  `c233bfdff8317a1f4ffdd5750e62a29ee6e69b2f`、clean/stash post-commit
  inventory、Task 260への自動復帰をcompleteする。

## Checker Task 260 active implementation

- [x] syntax-free five-table `2/2/1/2/2` producer、exact lower fingerprint、
  resolver provenance、Pending functor obligation 2件、one-shot Typed/final
  ownershipをTask-259 behavior unchangedで実装する。
- [x] private exact-source runner、new pass sidecar 1組、reciprocal covered trace
  row 1件、six active-count `199 -> 200` consumerをexisting artifact rebaseline
  なしで実装する。
- [x] optional application/structure/set targetをvalidation-only/semantic
  deferredのまま保ち、goal composition、proof/discharge、acceptance、fact/axiom、
  overload semantics、IR、VCをpublishしない。
- [x] frozen checker 5/runner 4 testsをexpandし、repeated test-sufficiency reviewと
  full implementation reviewを**NO FINDINGS**で終了する。
- [x] focused `5/5` / `4/4`、library `444/516/144/59`、checker lint `15/15`、
  metadata `137`、CLI `423/391`、`230/193`、`101/7/200/1`、type
  `255/243`、warnings/errors `23/0`を再現する。
- [x] checker `25/150547`、runner `32/64711` production、library test-list/
  CLI hashをfresh measureする。
- [x] source/docs consistencyを**NO FINDINGS**で完了し、fmt、warnings-deny
  Clippy、workspace test、metadata、CLI 5本、count/hash reproduction、
  whitespaceを含むfull workspace verificationをPASSする。
- [x] final hard gate 9件をすべて**NO FINDINGS**、quality `100/100`、score
  capなしでPASSする。
- [x] exact staging、commit
  `c83e424a485a24dd0f00ddea687903a235d85850`、clean/stash post-commit
  invariant、fresh Task 261 selectionを完了する。

## Checker Task 261 frozen-contract documentation prerequisite

- [x] clean post-Task-260 inventoryからTask 261をselectし、origin差をreport-only
  `repo_metadata_conflict`、missing contractを`design_drift`、implementationを
  `source_drift`、consumerを`test_gap`と分類する。
- [x] Chapter 6/16 authority、exact 116-byte/final-LF source、SHA-256
  `ffd4954aad628d7946aaf7afb1b472a6bdfca7bce5ba0cf09f5b284c9dda07bf`、
  45-row/root-44 Surface oracle、resolver profileをfreezeする。
- [x] lower Tasks 248/249/252/256を`1/2/2/2/2/2/0`、`2/2/0`、`2/2/0`、
  `1/0/0/0/0/0/0/2/2`にfreezeし、lower editなし、他family absentとする。
- [x] syntax-free definition/parameter/subject/definiens `1/2/1/1`、resolver
  provenance、fingerprint、unchanged obligation、one-shot Typed/final、Task-259/
  260 isolation、debug/failure orderをfreezeする。
- [x] spec-derived pass pair、future sole covered trace、checker 5/runner 4 tests、
  projected count、write scope、audit impact、semantic deferral、exitをfreezeする。
- [x] docs prerequisiteで`doc/spec`、existing `.miz`/expectation/sidecar、trace
  count/status、production/test/CLI/hashをpreserveする。
- [x] spec reviewを**NO FINDINGS**までrepeatし、docs gate、prerequisite
  `209c32fc2ec547ceedd32f1052345ae2fc5b0451`、post-commit invariantを
  PASSしてTask 261 implementationへreturnした。

## Checker Task 261 implementation

- [x] exact four-table producer、one-shot Typed/final ownership、strict
  obligation preservation、Task-259/260 isolationを追加する。
- [x] sole pass pair/covered trace row、shell-41/valid-but-stale context-site
  認証を持つprivate exact runnerを追加する。
- [x] checker `5/5`、runner `4/4`、test-sufficiency/full implementation
  re-reviewを**NO FINDINGS**で閉じる。
- [x] `449/520/144/59`、metadata `137`、`424/392`、`231/193`、
  `101/7/201/1`、type `256/244`、warnings/errors `23/0`、全hashを再現する。
- [x] source/docs consistencyを**NO FINDINGS**で完了し、exact count/hash
  reproductionを含むfull shared verification matrixをPASSする。
- [x] final hard gate 9件を**NO FINDINGS**、score capなし、quality
  `100/100`でPASSする。
- [x] exact stage/commit/post-commitを
  `b1782bfc06388410229f07ee193a5febe0bf525e`として完了し、fresh Task 262を
  selectする。

## Checker Task 262 frozen-contract documentation prerequisite

- [x] Chapter 7/16 authority、mode parser/resolver fixture、mixed gap/sidecar/
  trace、public Tasks 248--261、全baseline、clean HEAD/origin delta、protected
  stashをfresh-inventoryする。
- [x] missing contractを`design_drift`、producerを`source_drift`、consumerを
  `test_gap`、origin差をreport-only `repo_metadata_conflict`と分類し、blocking
  `spec_gap`がないことを確認後、binding-linked Task-249 RHS mismatchをmandatory
  lower `source_drift`、fabricated third bindingを`boundary_violation`と分類する。
- [x] exact 141-byte source/hash、全54 AST row、two-shell resolver provenance、
  parameter/context/application/RHS association、post-Task-249M lower profile
  Task 248 `1/2/2/2/2/2/0`、Task-249 base `2/3/0`、standalone mode-RHS row
  1個をfreezeする。
- [x] dense table 6個を`1/2/1/1/1/1`、fingerprint 2個、unresolved RHS-
  inhabitation request、pending existing-kind `Sethood` row 1個、exact
  Typed/final ownership、Tasks-259--261 isolation、debug/failure orderをfreezeする。
- [x] assume/equals/means/return/formula payload不在、ParamGuard/proof/
  discharge/acceptance/fact/IR/VC deferral、subtree exclusion、test、projected
  count、audit impact、write scope、exit criteriaをfreezeする。
- [x] docs prerequisiteでproduction、existing fixture/sidecar/expectation、
  trace count/status、CLI、list、manifest、hashを保つ。
- [x] specification reviewを**NO FINDINGS**まで反復し、全docs hard gateを
  score capなし`100/100`でPASSし、本upper frozen-contract prerequisiteだけを
  `8c3fa20acef42477d38a66ddddec42dacced0863`としてcommitする。
- [x] Task 249M exact standalone mode-RHS ABI、`2/3/0/0/1` profile、validation/
  debug/test/exclusion/scopeをfresh-inventory/freezeする。
- [x] Task-249M findings-free review/unchanged docs gate/exact staging/dedicated
  docs commit/clean post-commit inventoryを完了する。
- [x] frozen Task 249M API/test 4件をimplementする。review/verification/
  exact staging/separate commitをTask 262前に完了する。
- [x] Task 262だけへ戻り、exact six-table producer、Pending `Sethood` suffix、
  Typed/final transaction、private runner、pass pair、trace、audit、focused test、
  projected countをimplementした。
- [x] repeated final reviewを**NO FINDINGS**で完了し、全9 hard gateをscore cap
  なしのquality `100/100`で通過してexact Task-262 commit-readyとする。その後、
  semantic spilloverなしにfresh-inventory Task 263+へ進む。

## Checker Task 263 preflight lower prerequisite

- [x] fresh Task-262 post-commit inventory後、Chapter-5 structure/inheritance/
  constructor-definition intakeを選択。
- [x] false cross-structure selector duplicateをlower resolver
  `source_drift`、paired `design_drift`/`test_gap`と分類。
- [x] separate resolver Task 263Rをfreezeし、そのdocs prerequisiteではchecker
  source、corpus、trace status/count、structure semanticsを変更しない。
- [x] Task-263R documentationをseparate commit
  `34692ee222d5465750f061da82fe878566a1557c`で完了し、lower implementation前に
  fresh inventoryを実行。
- [x] frozen resolver two-file correctionだけを実装し、test-sufficiency/
  implementation reviewを**NO FINDINGS**で完了。
- [x] consistency/full/final gateを**NO FINDINGS**、全9 hard gate PASS、score
  capなし、valid `100/100`で完了。
- [x] dedicated implementation commit/clean fresh inventory後、Task 263
  freezeへ戻る。
  commitは`997457dd3189030aa3b137b568ce82fed456fe1e`。clean fresh inventory、
  `origin/main...HEAD = 0/7`、protected stash不変を確認済み。

## Checker Task 249S standalone structure-member type prerequisite

- [x] Task-263R後にTask 263をfresh inventoryし、missing member-type ownerを
  `source_drift`、contractを`design_drift`、testを`test_gap`、fabricated/reused
  ownerを`boundary_violation`に分類する。
- [x] Chapter-5 authority、exact 320-byte source/hash、member/type/head row 4件、
  standalone `0/4/0/0/0/4` profileをfreezeする。
- [x] additive public ABI、error 5件/precedence、debug、Typed/final ownership、
  sibling isolation、tests/counts/exclusions/audit impact/two-commit exitをEN/JA
  syncする。
- [x] specification reviewを**NO FINDINGS**まで反復し、docs-only format、
  Clippy、full workspace test、5 CLI、count/hash、scope、whitespace verificationを
  PASSする。
- [x] final read-only qualityを**NO FINDINGS**、全9 hard gates PASS、score cap
  なし、valid `100/100`で完了する。
- [x] 同期文書だけをstage/commitし、clean post-commit inventoryを確認する。
  documentation commitは
  `274917ab21cf436411d7b7d308bd676f4b444a67`。clean inventoryで
  `origin/main...HEAD = 0/8`とprotected stash不変を確認済み。
- [x] fresh inventory後exact `0/4/0/0/0/4` handoffとchecker test 4件を
  実装し、test-sufficiency/implementation reviewを**NO FINDINGS**でcloseする。
- [x] source/documentation/final quality gate、exact stage、dedicated
  implementation commit、clean fresh inventoryを完了し、Task 263へ戻る。
  implementation commitは
  `93d64c33eb4234793f7e6f9d95516a366464dd9b`、9 gatesはuncapped
  `100/100`、fresh inventoryは`origin/main...HEAD = 0/9`、protected stash不変。

## Checker Task 263 structure-definition intake

- [x] canonical plan/TODO/audits、Chapter 5とbounded Chapters 13/16/19、exact
  parser/resolver source、active mixed gap、Task-249S lower/public API、counts/
  CLI/hashをfresh inventoryする。
- [x] Task-263R/249S committedとexact source dependency-ready、frontend/resolver
  `75/10/8/8/0`、source type `0/4/0/0/0/4`を確認する。
- [x] exact source/hash、parameter/context absence、`2/4/1/2/0` rows、constructor/
  selector、root/path/view coverage、resolver provenance、lower fingerprint、
  identical-type zero coherenceをfreezeする。
- [x] arbitrary baseline obligations unchanged、new kind/goal/guardなし、Typed/final
  ownership、Task-259--262 isolation、private runner、subtree exclusions、tests、
  count/hash、deferrals、scope、exitをEN/JA同期する。
- [x] review-only specificationとsource/documentation auditを**NO FINDINGS**
  まで反復し、docs hard gatesをuncapped `100/100`でpassする。parent-ownedの
  staging/commit用exact docs-only targetを保つ。
- [x] docs commit `1fe0b156f312628f0997261ef6a8c8de251a15c8`後fresh inventoryし、
  frozen Task-263 producer/ownership/private
  runner/pass pair/trace/auditsだけを実装する。
- [x] test/implementation/source-doc/final reviewsを**NO FINDINGS**で完了し、
  全9 gatesをscore capなしの`100/100`でPASSし、全verificationもPASSする。
  Task-263 implementation commit `f11a517e91433b461447522eff06cd85e6187063`とclean
  fresh inventoryを完了し、その後Task 264+へ自動継続する。

## Checker Task 264 lower-prerequisite sequence

- [x] Task 264をfresh inventoryし、parser-represented property-implementation shell欠落を
  lower `source_drift`、contract欠落を`design_drift`、regression欠落を`test_gap`、
  identity fabricationを`boundary_violation`と分類し、blocking `spec_gap`なしを確認。
- [x] resolver Task 264Rをdocs-first/context-only shell intakeとして、exact
  file 4/test 2 scopeとchecker/corpus/trace無影響をfreeze。
- [x] Task 264R docs commit
  `b1ed8ea19f8845d8c54f795a7375d4add4af237d`、fresh inventory、exact lower
  implementation commit `db8c39e31678d6b8a1f0900a5368c3b95c7162b5`を完了。
  clean fresh inventory/protected-stash invarianceを確認済み。
- [x] separate checker Task 248P binding-context admissionをfresh inventoryし、property
  semantics/runner creditなしのclosed Profile Cとしてfreeze。
- [x] Task 248P docs reviewと全9 hard gateをfindings-free、score capなし`100/100`で
  完了し、exact 32-document staging targetを保存。
- [x] exact docs commit
  `1e3fa789ce335b900fca4ac6ef5ad56b40cb5f24`、fresh inventory、findings-free
  test/implementation review付きseparate one-file/two-test implementationを完了。
- [x] source/documentation/final quality reviewを**NO FINDINGS**、全9 gateをscore cap
  なし`100/100`、全required verification/count/hash gateをPASS。
- [x] Task 264 exact property payload contractをfreezeし、Task 259分離と
  authority-limited semantic deferralを保存。
- [x] Task264 exact means/equals sources/hashes、85/56 AST、resolver provenance、
  parameter/context/declared return、lower owners/fingerprints、means-only `it`、
  no `assume`、five-table ABI、obligations、Typed/Resolved、tests/counts/deferrals/
  exitをEN/JA同期freezeする。
- [x] Combined parameter/member source-type gapをseparate lower `source_drift`と
  classifyし、Task249PIをmandatory next prerequisiteとしてfreezeする。
- [x] Specification/boundary/source-doc reviewをNO FINDINGSまでrepeatし、docs-only
  all nine gatesをscore capなし`100/100`でcompleteする。
- [x] Exact 32-document targetを
  `4c3f74b053d31cae45b8af3fc478498b4a112768`としてcommitし、fresh Task249PI
  selectionをcompleteする。
- [x] Task249PI docs/implementation後Task264へ自動復帰し、frozen checker/runner
  transportだけを実装した。
- [x] Task264 review、全9 gate、exact count/hash verificationを完了し、
  parent-owned commit用task-only targetを保存する。
- [x] commit後clean fresh inventoryを確認し、次のdependency-ready taskへ
  自動継続する。

## Checker Task 249PI property-type composition prerequisite

- [x] committed Task264 docs、clean worktree、origin divergence、protected stash、exact
  sources/AST、lower API、dependency readinessをfresh inventoryした。
- [x] missing combined source-type handoffをlower `source_drift`/`design_drift`/
  canonical-derived `test_gap`、blocking `spec_gap`なしと分類した。
- [x] exact `1/3/0/0/0/2` profiles/sites/ranges、authenticated structure head、
  additive API/errors/precedence、debug/Typed/final ownership、exclusion、test 4件、count/
  exitをfreezeした。
- [x] spec/boundary reviewを**NO FINDINGS**まで繰返し、docs hard gate 9件をvalid
  uncapped `100/100`でPASSする。
- [x] exact synchronized Task249PI design recordを
  `7e194bb3d7dd01454958b8d319b8c48cf478896a`としてcommitした。
- [x] fresh inventory後`source_type.rs`だけへimplementationし、test sufficiency/
  implementation reviewは**NO FINDINGS**、measured module inventory同期後full
  verificationはPASSした。
- [x] source/doc reviewは**NO FINDINGS**、final qualityはhard gate 9件PASS、score
  capなし`100/100`。
- [x] reviewed checker-only Task249PI implementationを
  `73a34f94c7d46d7c0698b09a43ab3e1f00bb07a7`としてcommitし、Task249PIへ
  property semantics/runner/corpus/adjacent lower workを混ぜずTask264へ戻った。

## Checker Task 269A named-witness binding slice

- [x] committed Task 264、clean worktree、report-only origin divergence、
  protected stash不変、canonical authority、広いgap fixture/trace、Tasks
  248--258 public APIをfresh inventoryし、Task 269Aだけを選択する。
- [x] exact 107-byte Task-258B3N source、51-node arena、resolver
  `LocalTermBinding`、`2/1/0 -> 2/2/0` binding transition、witness/name/RHS
  link、5 fingerprints、debug grammar、Typed/final ownership、private
  dormant consumer、exclusion、exact 8 tests、count/hash impact、deferral、
  exit criteriaをEN/JAへfreezeする。
- [x] review-only specification/source-doc auditを**NO FINDINGS**まで反復し、
  docs-only全9 hard gatesをscore capなし90/100以上でpassする。
- [x] Task-269A documentation prerequisiteだけを
  `1360a9c0517eacbc67bbf2351db57e81eef03bfc`としてstage/commitし、
  production、fixture、sidecar、expectation、trace、metadata、CLIが不変で
  あることを確認してfrozen implementationをfresh inventoryした。
- [x] frozen producer、one-shot Typed/final owner、private dormant consumer、
  checker 4 + runner 4 testsだけを実装する。
- [x] test/implementation/source-doc/final quality reviewを**NO FINDINGS**まで
  反復し、全verification/count/hash gate後implementationを
  `f548ceb9f1acbeca72919809f2a1db84da213982`としてcommitした。

## Checker Task 269B mixed-witness binding increment

- [x] Task-258B3M1 lower/public API、canonical authority、Task-269A commit、
  clean worktree、origin/stashをfresh inventoryし、B3M1だけを選択する。
- [x] exact 113-byte/56-node source、`2/1/0 -> 2/2/0`、declaration 1件と
  witness/name/RHS `0/0/2`、immutable unnamed witness1、fingerprint 5件、API
  no-op、tests、exclusion、impact、exit criteriaをfreezeする。
- [x] synchronized EN/JA documentation prerequisiteだけをreview/commitする。
  repeated specification reviewはNO FINDINGS、docs-only hard gate 9件はscore
  capなし`100/100`ですべてPASSした。
- [x] fresh preflight後frozen B3M1 incrementだけを実装し、existing 8 compound
  tests内でunnamed row non-bindingの直接assertionとall-field/cross-profile
  fail-closed coverageを追加した。
- [x] 全review/hard gate/verification後Task 269Bを
  `afd54a37ce4022929bdaf60be519ac4adbdd9b8e`としてcommitした。post-commitは
  clean、origin差はreport-only、protected stashは不変。

## Checker Task 269CP isolated proof-`let` lower prerequisite

- [x] canonical authority、broad read-only gap artifacts、parser/resolver shape、
  Task-269A/B APIs、metadataをfresh inventoryし、later-use/captureまたはchecker
  let bindingより前に269CPだけをselectする。
- [x] exact 100-byte source/hash、51-node/root-50 Surface、resolver theorem
  provenance、let/segment/name/bare-set sites、scope/ordinal/local row、fingerprint、
  exclusion、tests 4件、credit 0、exitをEN/JAでfreezeする。
- [x] specification reviewを**NO FINDINGS**までrepeatし、docs-only gate 9件を
  PASSしてdocumentation prerequisiteだけをcommitする。
- [x] fresh preflight後runner-private lower projectionだけをimplementする。
  checker/public API、BindingEnv、Typed/final owner、corpus、traceは不変。
- [x] independent reviewを**NO FINDINGS**まで完了し、verificationとfinal gate
  9件をscore capなしの`100/100`でPASSする。exact task-only commitをauthorizeし、
  その後missing type siteを維持してsource-type admissionを分離したbinding-only
  Task 269Cをfresh inventoryする。

## Checker Task 269C binding-only proof-`let` transaction

- [x] canonical Chapters 4/15/16、broad proof-local gap、Task-269CP lower、reserve
  bridge、BindingEnv、Typed/final APIをfresh inventoryしbinding-only 269Cだけをselect。
- [x] exact syntax-free input/output、base `1/1/0`、final `2/2/0`、missing typeの
  `LetBinding` 1行、lookup/debug/fingerprint、7-file scope、tests 8件、exclusion、
  count/hash、audit impact、exitをEN/JAでfreeze。
- [x] specification reviewを**NO FINDINGS**までrepeatし、docs-only hard gate 9件を
  score capなしの`100/100`でPASS。
- [x] documentation prerequisiteだけを
  `e3bc93c36577e7e250efab8cfc11d9b9695c3953`としてstage/commitし、fresh
  implementation preflight。
- [x] fresh preflight後exact 7-file binding transactionだけをimplementし、source-type
  admission/use-captureを分離。
- [x] independent test/implementation/source-doc/final review、verification、final gate
  9件、task-only commit `399dc44b2a4400f9eeb1b651d1ddd0bbc7a09f6a`、fresh inventoryを完了。

## Checker Task 269CT proof-`let` source-type prerequisite

- [x] Task-269CP/C profile、canonical Chapters 4/8/15/16、Task-249 API、ownership、
  corpus/trace、count/hash、origin/protected stashをfresh inventoryし、later use/captureより
  先に269CTだけをselect。
- [x] two-binding type overlay、source-type `2/2/0/0/0/0`、3-node arena、API/error/
  debug/fingerprint、Typed/final owner、dormant runner、7-file scope、test 8件、zero-credit
  audit impact、exclusion、projected count、exitをEN/JAでfreeze。
- [x] specification reviewを**NO FINDINGS**まで反復し、frozen contractとverification
  ledgerを同期。
- [x] independent source/docs/final quality reviewを**NO FINDINGS**、hard gate 9件
  PASS、score capなし`100/100`で完了。
- [x] documentation prerequisiteだけを
  `b1c91b1b42391ca205b709b47444f3f2e748a799`としてstage/commit。
- [x] fresh preflight後frozen seven Rust filesだけへ269CTを実装し、test-sufficiency/
  implementation reviewを**NO FINDINGS**までrepeat。
- [x] source/docs/final qualityを**NO FINDINGS**、hard gate 9件capなし`100/100`で
  完了し、full verificationをPASS。
- [x] exact implementation commit
  `c60361977f6c4d832cf4217b85bd9b458c902848`後、fresh inventoryを継続。

## Checker Task 269GP proof-`given` lower prerequisite

- [x] Task-269CT commit/clean inventory、canonical authority、exact parser/resolver
  measurement、report-only origin差、protected stash、dependency readinessを確認し、
  Task270より先に269GPだけをselect。
- [x] exact source/Surface/shell/resolver/output/debug fingerprint、subtree exclusion、
  binding-shaped field禁止、4 files/tests、zero credit、count/hash、semantic deferral、
  exitをEN/JAへfreezeし、269G/269GTをblockするChapter-4/16 scope矛盾を記録。
- [x] specification reviewを**NO FINDINGS**までrepeatし、docs-only verificationと
  frozen count/hashをPASS。
- [x] source/docs/final-quality reviewを**NO FINDINGS**で完了し、docs-only hard gate
  9件をscore capなし`100/100`でPASS。
- [x] synchronized design 40 filesだけをexact stageし、docs commit
  `97a75fd9bf6a791055f236b3e3b4bb07b8d3d7c3`を作成。
- [x] fresh preflight後freeze済みrunner 4 filesだけへ269GPを実装し、finding修正後の
  test-sufficiency/implementation再reviewを**NO FINDINGS**まで完了。
- [x] source/docs/final-quality reviewを**NO FINDINGS**、full verification、hard gate
  9件をscore capなし`100/100`で完了。
- [x] exact staging/implementation commitを完了し、fresh inventoryはcleanで、269Gを
  selectせずhuman-owned scope矛盾をreport。

## Checker Task 269GS canonical `given` scope reconciliation

- [x] clean HEAD/origin/stash、paired Chapter 4/15/16、existing parser/diagnostic
  fixture、trace row、269GP lower output、checker consumer不在をfresh inventory。
- [x] human-approved ruleをfreeze: 各`given`変数はdeclarationの`such that` condition内を
  bindし、後続statementでは最内のenclosing proof/reasoning block末尾までscope内、
  shadowされない限りnested childへ継承し、parent/sibling blockへescapeしない。
- [x] label scopeを不変とし、condition/fact、existential/Skolem、goal、proof、discharge、
  acceptance、IR、VC semanticsをdefer。
- [x] exact Markdown 46-file write scope、parser/broad-gap source/sidecar path/hash、
  byte-identical trace hash、library/production/list/CLI/count baseline、audit credit 0、
  Task269G `test_gap`/`source_drift` ownershipをfreeze。
- [x] specification reviewを**NO FINDINGS**までrepeatし、docs-only verification/count/
  hash/hard-gateをuncapped `100/100`で全PASS。
- [x] synchronized 269GS documentationだけをstage/commitし、binding-only Task269Gを
  fresh inventoryから自動選択。

## Checker Task 269G proof-`given` binding consumer

- [x] Task269GS authority、immutable 269GP lower、reserve base、binding/Typed/Resolved
  API、tests/count/hashをfresh inventory。
- [x] exact `GivenWitness` ABI、`1/1/0 -> 2/2/0`、lookup/inheritance/shadow/
  restore matrix、error/debug、Typed/final、Task269GT boundary、8-file scope、focused test
  8件、audit/baseline/exitをfreeze。
- [x] specification reviewを**NO FINDINGS**までrepeatし、docs-only gateとuncapped
  `>=90/100`をPASS。
- [x] synchronized existing Markdown 40件だけを
  `1672486e7c7923e56d9019404bc9c75ffa119f96`としてcommitし、fresh preflight後frozen
  Task269Gだけをimplement。
- [x] test/implementation/source-doc/final reviewは**NO FINDINGS**、全verification/hard gateは
  capなし`100/100`でPASS。exact implementation commit
  `4f65bc4d50ab950c6976a4b3f3cb4bc0948b27c1`後cleanでTask269GTをfresh inventory。

## Checker Task 269GT proof-`given` source-type consumer

- [x] canonical Chapters 4/8/15/16、immutable 269GP lower、committed 269G binding、
  269CT type model、public API、focused test、baseline、origin差、stashをfresh inventoryし、
  269GTだけをselect。
- [x] exact Given-type composite/error ABI、`Missing -> Source(84..87)` overlay、
  `2/2/0/0/0/0` payload、3-node arena、fingerprint、Typed/final/private runner ownership、
  7-file/8-test scope、zero-credit audit、semantic exclusion、exitをfreeze。
- [x] specification reviewを**NO FINDINGS**までrepeatし、docs-only verification/count/hash/
  hard gateをuncapped `>=90/100`でPASS。
- [x] synchronized Markdown 40件だけを
  `35bc97b92ce075226105e8fcd4c1e43c8621995c`としてcommitし、fresh preflight後frozen
  Task269GTだけを実装。
- [x] test/implementation/source-doc/final review **NO FINDINGS**、full verification/hard gate、
  exact implementation+synchronized record commit後later-use/captureまたはTask270をfresh inventory。

### Task 269GT implementation handoff

- [x] exact 7-file Given-type transactionとchecker 4件/runner 4件を実装。
- [x] test-sufficiency/implementation reviewを **NO FINDINGS** まで反復し、focused/full library `498/560`をpass。
- [x] source/docs/independent final-quality reviewは **NO FINDINGS**、full workspace/count/hash
  とhard gate 9件はcapなし`100/100`でPASS。exact staging、single implementation commitを
  完了し、dependency-ready successorをfresh inventoryする。

## Checker Task 269GUP proof-`given` use-profile binding prerequisite

- [x] clean Task269GT、Chapter 4/15/16、immutable 269GP/G/GT API、exact parser/resolver、
  baseline、origin差、stashをfresh inventoryし、269GUPT/269GU/capture/270より先に128-byte
  sibling binding profileだけをselect。
- [x] missing profileを`test_gap`、binding handoff不在を`source_drift`、stale status/ownerを
  `design_drift`、`source_type.rs`でのbinding reconstructionまたはresolver use-ID追加を禁止
  `boundary_violation`と分類。blocking `spec_gap`なし。
- [x] exact 128-byte/54-node profile、unique lower output、new-source identity、
  `1/1/0 -> 2/2/0`、lookup matrix、public ABI/error/debug、exact 6-file/8-test scope、
  zero credit、exclusion、42-file docs stage、exitをfreeze。
- [x] specification reviewは**NO FINDINGS**、docs-only verification/hard gateは完了し、
  42 synchronized Markdownはdocumentation prerequisite
  `ae03ae0772fe98532dbd68164c8a1fc4f4172e7e`としてcommit済み。
- [x] Task 269GUPと全exact reviewは**NO FINDINGS**で完了し、hard gate 9件は
  capなし`100/100`でPASS。exact stagingとsingle implementation commit後に
  Task 269GUPTをfresh inventoryする。source type、term/use、Typed/final、captureはabsent。
### Task 269GUP binding profile 実装状況

凍結済みの6ファイル transactionとchecker/runner各4件の正確なtestを実装した。libraryは`502/564`、checker/runner productionは`30/172531`と`37/74826`で、path hashは不変、content hashは`e0342952a01a0b379cf7b06ad243cd40a1656e940480196323cf43fbe7d8f7c5` / `8fe7c8c0b7e855e5113f3830873e133f42c8048a3272055e2fddd5ebd9cbb1bc`である。

閉じるのはdormant private lexical-binding evidenceだけで、active corpus、trace、type、term/use、condition/fact、goal/proof、obligation、diagnostic、CLIのcreditは0のままである。次はTask 269GUPTであり、Task 269GU、capture、Task 270は引き続きdeferする。

## Checker Task 269GUPT source-type prerequisite

- [x] clean GUP commit、canonical Chapters 3/4/8/15/16、exact lower/GUP handoff、old GT pattern、baseline、origin/stashをfresh inventory。
- [x] absent composite/testを`source_drift`/`test_gap`、stale statusを`design_drift`、binding reconstruction/semantic publicationを`boundary_violation`、origin `0/7`をreport-only `repo_metadata_conflict`と分類。blocking `spec_gap`なし。
- [x] by-value dependency、binding 1 `Source(84..87)`、`2/2/0/0/0/0`、distinct arena、public ABI/error/debug、Typed/Resolved、runner、7-file/8-test、40 docs、zero credit、baseline/deferral/exitをfreeze。
- [x] specおよびsource/docs review **NO FINDINGS**、docs-only hard gate `9/9`、capなし`100/100`。
- [x] exact 40 Markdownをstageしdocs commitする。
- [x] GUPTだけのfresh preflight/implementationは完了し、test/implementation/
  source-docs reviewは**NO FINDINGS**、hard gate `9/9`、capなし`100/100`。
- [x] exact stageとseparate implementation commit
  `c529245138b6d40be65c590ba701fef4f4ea0881`を完了し、clean fresh inventoryは
  269GUを選択。capture/270はdefer。

## Checker Task 269GU later-use term/reference prerequisite

- [x] committed GUPT HEAD、canonical Chapters 3/4/8/13/15/16、exact source、
  GUP/GUPT API、baseline、origin/stashをfresh inventoryしGUだけを選択。
- [x] absent composite/test=`source_drift`/`test_gap`、stale status=
  `design_drift`、generic admission/semantics=`boundary_violation`、origin
  `0/9`=report-only `repo_metadata_conflict`と分類。blocking `spec_gap`なし。
- [x] exact GUPT dependency、`2/2/0` payload、profile-scoped Given admission、
  6-node arena、public ABI/error/debug、boxed Typed/final、private runner、
  7 files/8 tests、42 docs、zero credit、baseline/deferral/exitを凍結。
- [x] spec review **NO FINDINGS**、docs-only gates、exact 42-file prerequisite
  commit `5f61e125eddeaf2a6defeb2419436a2f37396421`を完了。
- [x] fresh preflight後GUだけを実装し、test-sufficiency/implementation reviewは
  **NO FINDINGS**、exact count/hashを記録しexcluded artifactを保持。
- [x] source/docs/final-quality reviewは**NO FINDINGS**、全verificationを含む
  hard gate 9件はcapなし`100/100`でPASS。
- [x] exact stage/separate implementation commit
  `998dc104957d47e2707f4a8292d2002f1c5beb2d`完了、clean fresh inventoryは
  Task269GCPを選択。Task270はseparate。

## Checker Task 269GCP Given-condition lower prerequisite

- [x] committed GU HEAD、origin/stash、canonical condition authority、fixture/
  trace、lower API、baselineをfresh inventory。
- [x] condition occurrenceをdescendant/capture/exportより先に選択し、missing
  profile/testを`source_drift`/`test_gap`、stale recordを`design_drift`、origin
  `0/11`をreport-only `repo_metadata_conflict`に分類。
- [x] exact 134-byte/54-node source/hash、shell/resolver、private lower/debug、
  mutation、4 files/tests、zero credit、exclusion、successor/exitをEN/JAでfreeze。
- [x] spec reviewを**NO FINDINGS**まで反復し、docs-only gate、exact Markdown
  stage、documentation prerequisite commit
  `db907a789dc01ba65ed8fdcc001e568e4f03cf49`を完了。
- [x] fresh-preflight後frozen 4 files/4 testsへGCPだけを実装し、canonical
  artifact/public route/semantic ownerを不変に維持。
- [x] source-doc/final-quality review **NO FINDINGS**、full verification、全9
  hard gatesをscore capなし`100/100`で完了。
- [x] exact stagingとdedicated implementation commit
  `59eb7de68d83901375883a2a6249796afc6a0de3`後Task269GCをfresh inventory。

## Checker Task 269GC Given-condition binding consumer

- [x] clean GCP commit、canonical scope、exact GCP lower、G/GUP boundary、
  baseline、origin、stashをfresh inventoryしGCだけをselect。
- [x] missing producer/test=`source_drift`/`test_gap`、contract=
  `design_drift`、forbidden semantic/higher-owner=`boundary_violation`、origin
  `0/13`=report-only `repo_metadata_conflict`、blocking `spec_gap`なし。
- [x] distinct ABI、GCP dependency、`1/1/0 -> 2/2/0`、block matrix、Typed/
  Resolved/runner、7 files/8 tests、zero credit、42 docs、deferral/exitをfreeze。
- [x] spec review **NO FINDINGS**、docs gate `100/100`、exact Markdown commit
  `dd053c86dab322508a15823de1c4afd268c2d35a`を完了。
- [x] fresh preflight後frozen 7-file/8-test GCだけをimplementし、test-sufficiency/
  implementation reviewは**NO FINDINGS**。
- [x] source-doc consistency/final-quality review、workspace-wide final gate、
  exact implementation commit `8181ae8fc8af0c7028254ad30147b417fbf84611`後
  Task269GCTをautomatic inventory。

## Checker Task 269GCT Given-condition source-type consumer

- [x] clean GC、canonical type authority、GC/GCP、GT/GUPT pattern、baseline、
  origin/stashをfresh inventoryしGCTだけをselect。
- [x] missing family/test=`source_drift`/`test_gap`、contract/stale status=
  `design_drift`、forbidden semantics/artifact/runner owner=`boundary_violation`、
  origin `0/15`=`repo_metadata_conflict`、blocking `spec_gap`なし。
- [x] by-value composite、`2/2/0` overlay、2 type rows、3-node arena、fingerprint、
  Typed/Resolved/private runner、7 files/8 tests、zero credit、deferral/exitを
  EN/JAでfreeze。
- [x] spec reviewを**NO FINDINGS**までrepeatしmeasured docs-only verification
  gateを全PASS。final qualityも**NO FINDINGS**、全9 gates PASS、capなしの
  `100/100`。exact stage/commitはremaining。
- [ ] fresh preflight、GCTだけ実装、全review/verification/9 gates、separate
  commit後Task269GCUをautomatic inventory。
