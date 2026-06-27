# mizar-atp TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様は、それを引用する実装タスクより前に、専用の仕様タスクが
（英語と日本語を同じ変更で）追加する。完了済みの仕様と source-deferred の
module は表で示す。この crate はアーキテクチャ 09、10、15、19 と internal 04
を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| problem | `problem.md`（task 2） | `src/problem.rs` | [x] |
| translator | `translator.md`（task 4） | `src/translator.rs` | [x] declaration、symbol-map、axiom、conjecture translation source complete |
| property_encoding | `property_encoding.md`（task 7） | `src/property_encoding.rs` | [x] axiom-form property source 完了。native declaration は deferred |
| tptp_encoder | `tptp_encoder.md`（task 9） | `src/tptp_encoder.rs` | [x] deterministic FOF source 完了。typed/native/backend route は deferred |
| smtlib_encoder | `smtlib_encoder.md`（task 11） | `src/smtlib_encoder.rs` | [x] deterministic uninterpreted SMT-LIB source 完了。theory/sorted/native/backend route は deferred |
| backend | `backend.md`（task 13） | `src/backend.rs` | [x] generic runner と mock classification 完了。real adapter / extraction は deferred |
| portfolio | `portfolio.md`（task 17） | `src/portfolio.rs` | [x] task-18 no-early-stop source 完了。proof policy、real extraction、kernel check、witness/cache/artifact handoff は deferred |

`mizar-atp` はパイプライン phase 13 を実装する。入力は ATP 対象の
`VcStatus::NeedsAtp` `VcIr` 義務、出力はバックエンド中立の `AtpProblem`、
具体的な prover プロトコルの出力、外部バックエンドの実行、そして
formula/substitution evidence candidate である。この crate が生産するものはすべて
untrusted な証拠である: `Proved` の主張は `mizar-kernel` が
formula/substitution evidence を検査して初めて信頼され、勝者/ポリシーの選択は
`mizar-proof` に属する。決定性規則は Mizar 側のすべて（premise 順、
エンコーディング、problem id）に適用され、バックエンドの非決定性は
メタデータとして記録され、黙って吸収されることはない。

依存順序: `problem` データ → `translator` / `property_encoding` →
プロトコルエンコーダ → `backend` runner → `portfolio`。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は `mizar-session`、`mizar-core`（core 論理式）、`mizar-vc`
（`NeedsAtp` 状態の `VcIr`）、`mizar-kernel`（kernel post-closeout correction
後の formula/substitution evidence schema type）に依存する。バックエンドの
バイナリは `PATH` または明示的設定で構成される外部プロセスであり、crate の
テストはモックバックエンドを使う。アーキテクチャ:
[09.atp_interface_protocol.md](../../architecture/ja/09.atp_interface_protocol.md)、
[10.atp_backend_integration.md](../../architecture/ja/10.atp_backend_integration.md)。
統合: [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)。

## Postponement gate

`mizar-atp` の自律開発は、`mizar-kernel` が formula/substitution evidence schema を
記録・実装し、`mizar-vc` が対応 handoff contract を記録するまで deferred とする。
新しい ATP 作業は trusted output として MiniSAT-compatible resolution trace を
target にしてはならない。ATP backend は内部では任意の proof-search method を使って
よいが、この crate の trusted handoff は kernel の SAT-backed checking 向けの
formula、substitution、provenance、target binding を含む candidate evidence package
である。instantiated formula と SAT problem は `mizar-kernel` が導出するものであり、
trusted ATP payload として生成しない。

現在の gate 状態: `mizar-kernel` task 23-28 と `mizar-vc` task 24-25 により、
generic runner path については満たされている。task 1-14 は spec-backed な
problem、translation、encoding、generic backend-runner slice だけを構築してよい。
task 15 は first real backend adapter と evidence extractor を、paired extraction spec と
guarded supported backend route が存在するまで `external_dependency_gap` / `deferred` として
記録する。proof policy、witness publication、cache promotion はそれぞれ専用 crate/task まで
deferred のままである。`mizar-proof` は workspace crate ではないため、policy と
witness-publication integration は `external_dependency_gap` であり、ここで placeholder を
追加する理由にはならない。

## 解決済みおよび保留中の決定

- **最初のバックエンドと evidence route: task-15 gate 後も deferred。**
  kernel formula/substitution evidence schema と VC handoff は利用可能になったが、
  concrete backend output を kernel が parse できる formula/substitution candidate
  bytes または ref に変換する paired `evidence.md` extractor spec と source module が
  この crate にはまだない。task-15 verification environment では architecture-10 supported
  backend binary も利用できなかった。この extraction route が仕様化されるまで、
  real adapter、backend-output parser、placeholder candidate schema を追加しない。
- **evidence schema ownership: `mizar-kernel` task 23-25 に従う。** この crate は
  kernel 所有のスキーマ型に対して candidate evidence を構築する。kernel がこの
  crate に依存することは決してない。
- **外部認証された証拠: ここでは範囲外。** ラベル付けはこの crate が
  生産するが、受理ポリシーは `mizar-proof` が所有する（アーキテクチャ 10
  の制約。同 task 4）。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-atp` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### problem 層

1. **crate の足場と lint 方針のガード。** [x]
   - `mizar-session`、`mizar-core`、`mizar-vc`、`mizar-kernel` に依存する
     workspace メンバー `mizar-atp` を追加し、`mizar-frontend` のガードに
     倣った `tests/lint_policy.rs` を追加する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: `mizar-vc` task 24、`mizar-kernel` task 23-25。仕様:
     アーキテクチャ 09 と post-closeout evidence correction。
   - 状態: scaffold のみの task として完了。crate plan は、task 前に crate が
     存在しない状態を `source_drift` として分類し、paired spec が存在するまで
     semantic module 実装を deferred に保ち、未存在の `mizar-proof` 連携と
     first-backend route を `external_dependency_gap` / `deferred` として記録する。

2. **仕様: `problem.md`。** [x]
   - `AtpProblem` のデータ形状仕様を執筆する（英語と日本語、コードなし）:
     logic profile、宣言、公理、conjecture、型コンテキスト、エンコード
     済みプロパティ、シンボルマップ、`AtpProvenance`、`expected_result` の
     極性。
   - 依存: 1。仕様: アーキテクチャ 09「Backend-Neutral Problem Layer」、
     [01.ir_layers.md](../../architecture/ja/01.ir_layers.md)、architecture 15、
     architecture 19、internal 04。
   - 状態: docs-only task として完了。`problem.md` は backend-neutral problem
     boundary、deterministic identity、provenance requirement、`Unsat` polarity
     contract、trusted material として禁止されるものを定義する。Rust data shape は
     task 2 closeout 時点では task 3 に deferred され、task 3 で実装済みである。

3. **`problem` データ形状の実装。** [x]
   - task 2 に従って `AtpProblem` と来歴テーブルを実装し、決定的 debug
     レンダリングを加える。
   - テスト: 構築のラウンドトリップ。すべての論理式が来歴で追跡可能。
     レンダリングの安定性。
   - 依存: 2。仕様: `problem.md`。
   - 状態: 完了。module は validated problem construction、deterministic
     identity/rendering、provenance と symbol-map check、missing required input の
     fail-closed、unsupported profile-feature classification、固定された `Unsat`
     expected-result contract を実装する。backend runner、kernel checking、proof
     policy、witness、cache、trusted backend proof material は導入しない。

### 翻訳

4. **仕様: `translator.md`。** [x]
   - `VcIr`→`AtpProblem` 翻訳の仕様を執筆する（英語と日本語、コード
     なし）: premise の具体化、決定的な premise 順、soft type 事実の保存
     （sort エンコーディングは VC の正当化に必要な事実を消してはなら
     ない）、validity 検査の極性。
   - 依存: 2。仕様: アーキテクチャ 09「Encoding Strategy」「Validity
     Checking Polarity」。
   - 状態: docs-only task として完了。`translator.md` は deterministic な `VcIr` /
     kernel-handoff から `AtpProblem` への translation boundary、target-binding check、
     premise materialization limit、structured projection input、duplicate-premise
     rejection、proof-hint non-pruning、soft-type preservation、declaration /
     symbol-map responsibility、`Unsat` polarity、trusted/backend material として禁止される
     ものを定義する。declaration / symbol-map translator source は task 5 で実装済みであり、
     axiom/conjecture problem construction は task 6 で実装済みである。

5. **宣言とシンボルマップの翻訳。** [x]
   - `VcIr` / handoff input 由来の structured declaration / soft-type projection を、
     診断に十分なだけ逆引き可能なシンボルマップとともに `AtpDeclaration` へ翻訳する。
   - テスト: non-`NeedsAtp` VC と stale handoff を reject する。missing/malformed の
     structured declaration / soft-type projection で fail closed する。shuffled equivalent
     input で deterministic declaration / symbol-map を生成する。duplicate/missing/kind/arity
     mismatch declaration で fail closed する。explicit profile choice を profile の黙った
     切り替えなしに保持する。translator API/debug rendering に prohibited
     backend/kernel/SAT/proof-acceptance material が入らないことを確認する。
   - 依存: 3、4。仕様: `translator.md`。
   - 状態: 完了。`src/translator.rs` は task-5 partial translation を公開する。
     `NeedsAtp` status と target handoff を検査し、structured declaration / soft-type
     projection を消費し、deterministic declaration、symbol-map row、type guard、provenance、
     diagnostic、target binding を導出し、final `AtpProblem` を構築せずに type-guard
     signature を検証する。

6. **公理と conjecture の翻訳。** [x]
   - 引用された premise を決定的な順序で公理に具体化し、goal を
     conjecture としてエンコードし、来歴と `expected_result` を付ける。
   - テスト: non-`NeedsAtp` VC と mismatched target handoff を reject する。
     missing/malformed の structured formula projection で fail closed する。
     unsupported formula/profile feature または alpha-repair / substitution-invention
     requirement は unsupported/open outcome として報告する。duplicate premise ref/source
     identity を reject する。proof hint と `Only` / `Exclude` restriction が premise を
     add/drop/prune しないことを確認する。required proof status、statement fingerprint、
     formula context が missing の imported fact で fail closed する。premise-order determinism、
     provenance completeness、soft-type preservation、固定の
     `ExpectedBackendResult::Unsat` polarity、prohibited backend/kernel/SAT/proof-acceptance
     material がないことを確認する。
   - 依存: 5。仕様: `translator.md`。
   - 状態: 完了。`src/translator.rs` は task-6 の `AtpTranslationInput`、
     VC formula ref と imported fact 用の structured `AtpFormulaProjection` target、
     `translate_problem` を公開する。translator は immutable な sorted `vc.premises`
     list を axiom に materialize し、VC goal を conjecture として materialize し、
     `ExpectedBackendResult::Unsat` を記録し、final-goal handoff polarity が
     `AssertFalseForRefutation` であることを検査し、final-goal projection binding に
     `goal:1` を要求し、duplicate premise ref、duplicate resolved formula/source identity、
     repeated imported source tuple を reject し、projection fingerprint / provenance payload を
     対応する VC kernel handoff と照合する。この照合では handoff formula byte を parse しない。
     local-context、cited、generated、imported fact の materialization は coverage 済みである。
     checker-owned と type-predicate premise materialization は、VC handoff が対応する explicit
     source class/projection を公開するまで fail-closed のままであり、`mizar-atp` は placeholder
     source class を発明しない。

7. **仕様: `property_encoding.md`。** [x]
   - プロパティエンコーディングの仕様を執筆する（英語と日本語、コード
     なし）: 定義のプロパティ（commutativity など）を公理として、または
     バックエンドネイティブのプロパティとしてエンコードする方法と、
     各戦略の適用条件。
   - 依存: 4。仕様: アーキテクチャ 09「Property Encoding」。
   - 状態: 完了。`property_encoding.md` は supported property family、axiom-form
     encoding、generated-binder declaration、native declaration gate、deterministic
     identity、provenance requirement、connectedness disjunction handling、
     fail-closed/deferred class、task-8 test expectation を仕様化する。この spec-only task
     では Rust source を追加しない。

8. **プロパティエンコーディング。** [x]
   - エンコーディング決定を `EncodedProperty` に記録しつつプロパティ
     エンコーディング規則を実装する。Task 8 は axiom-form property だけを emit し、
     native declaration は concrete encoder spec が exact semantics を定義するまで deferred
     のままにする。
   - テスト: プロパティごとのフィクスチャ、generated-binder declaration/provenance
     coverage、connectedness disjunction coverage、deterministic ordering、
     native-declaration deferred/fail-closed coverage。
   - 依存: 6、7。仕様: `property_encoding.md`。
   - 状態: 完了。`src/property_encoding.rs` は structured explicit property projection を受け取り、
     target declaration / symbol-map row と profile capability を検証し、deterministic な binder
     declaration、symbol-map row、provenance を生成し、`EncodedProperty::axiom` row だけを emit
     する。duplicate と unsupported/deferred family は fail closed し、native declaration は
     deferred のままにする。

### プロトコルエンコーダ

9. **仕様: `tptp_encoder.md`。** [x]
   - TPTP 出力の仕様を執筆する（英語と日本語、コードなし）: 方言の
     カバレッジ、名前マングリング、決定的出力規則。
   - paired `tptp_encoder.md` docs で完了。task-10 source は deterministic FOF emission に
     限る。TFF-like typed output、CNF、include file、native property declaration、
     backend pragma、backend execution、evidence extraction は deferred のままである。
   - 依存: 2。仕様: アーキテクチャ 09「Supported Formats」。

10. **TPTP エンコーダ。** [x]
    - `AtpProblem` から TPTP テキストを決定的に出力する。
    - テスト: golden ファイルのフィクスチャ。実行をまたぐ byte-identical output。
      exact separator、parenthesization、label、final newline。profile gate。
      native-property rejection。free-variable、duplicate-binder、shadowing rejection。
      raw-name injection と mangling-collision rejection。provenance side metadata。
      lint/API boundary guard。
    - Status: complete。`src/tptp_encoder.rs` は validated `AtpProblem` から
      deterministic FOF text だけを emit し、symbol と formula-label の side metadata を
      記録し、unsupported profile、sorted binder、native declaration、scope failure、
      malformed private formula case、raw-name injection、name/label collision を reject し、
      diagnostic を semantic text に入れない。backend runner、kernel/SAT checking、proof
      acceptance、witness/cache integration、TFF/native shortcut、legacy certificate、
      resolution-trace acceptance は追加しない。
    - 依存: 6、9。仕様: `tptp_encoder.md`。

11. **仕様: `smtlib_encoder.md`。** [x]
    - SMT-LIB 出力の仕様を執筆する（英語と日本語、コードなし）: sort
      エンコーディング、logic の選択、決定的出力規則。
    - paired `smtlib_encoder.md` docs により完了。task-12 source は、1 つの fixed
      `mizar_universe` sort と explicit guard predicate / type-guard assertion を使う
      deterministic uninterpreted SMT-LIB emission に限定する。arithmetic theory、array、
      datatype、bit-vector、sorted function/predicate signature、`BackendSorts`、
      `SortsAndGuards`、native property declaration、solver option、proof/unsat-core
      command、backend execution、evidence extraction は deferred のままである。
    - 依存: 2。仕様: アーキテクチャ 09「Supported Formats」。

12. **SMT-LIB エンコーダ。** [x]
    - `AtpProblem` から SMT-LIB テキストを決定的に出力する。
    - テスト: golden ファイルのフィクスチャ。`QF_UF` / `UF` logic selection。
      exact formula rendering。premise plus negated conjecture polarity。fixed
      `mizar_universe` sort と explicit guard/type-guard preservation。unsupported sort
      strategy、sorted binder、equality、quantifier、sort-dependent use の profile gate。
      未使用の sort declaration は無視され output に含まれないこと。native-property
      rejection。scope/arity/source failure。raw-name injection と SMT-LIB symbol collision
      rejection。provenance side metadata。proof/unsat-core/backend-material trust がないこと。
    - Status: complete。`src/smtlib_encoder.rs` は validated `AtpProblem` から
      deterministic uninterpreted SMT-LIB text だけを emit し、symbol と assertion-label の
      side metadata を記録し、`Unsat` contract の下で premise/type guard/property と negated
      conjecture を emit し、unsupported profile、sorted binder、native declaration、
      scope failure、malformed private formula case、raw-name injection、name/label collision を
      reject し、diagnostic と proof/unsat-core material を semantic text に入れない。backend
      runner、kernel/SAT checking、proof acceptance、witness/cache integration、
      theory/sorted/native shortcut、legacy certificate、resolution-trace acceptance は追加しない。
    - 依存: 6、11。仕様: `smtlib_encoder.md`。

### バックエンド実行

13. **仕様: `backend.md`。** [x]
    - バックエンドの仕様を執筆する（英語と日本語、コードなし）:
      バックエンド trait、プロセスモデル（spawn、リソース制限、終了）、
      設定とバージョン記録、クラッシュ処理、そして「`Proved` は
      `expected_result` の一致と証拠の存在を要する」規則を含む結果分類。
    - Status: paired `backend.md` docs により完了。task-14 source は generic
      child-process runner、mock backend fixture、deterministic run metadata、
      resource / timeout / cancellation / crash handling、invariant-preserving mock
      classification に限定する。real backend adapter、backend-specific output parsing、
      candidate evidence extraction、portfolio execution、proof policy、witness/cache
      publication、kernel checking は deferred のままである。
    - 依存: 2。仕様: アーキテクチャ 10「Process Model」「Result
      Classification」、[internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)
      「Backend Runner」。

14. **バックエンド runner。** [x]
    - リソース制限、タイムアウト、キャンセル、graceful なクラッシュ処理を
      備えたプロセス実行を実装する。テスト用モックバックエンドを用意する。
    - テスト: stdin と private problem-file mode。shell interpretation を使わない
      direct executable / argument spawning。deterministic command fingerprint。
      version-probe success/failure metadata。timeout、cancellation、kill-grace、
      crash、non-zero exit、missing executable、spawn-permission fixture。
      stdin と private-file mode の両方で byte-exact input delivery を行い、rewriting、
      normalization、appended proof command、unsat-core request、shell interpretation、
      inferred polarity change がないこと。stdin delivery は path を backend に露出せず
      fd 0 に接続した private spool を通じて行い、verifier 側 writer-thread deadlock を
      作らないこと。process id、temp path、timestamp、raw completion order、
      machine-local absolute executable / working-directory path を fingerprint から除外し、
      allowlist された environment variable を sort 済みで記録すること。stdout/stderr hash と
      truncation diagnostic。private temporary cleanup。
      retained limit 後も drain を続け、stream hash が complete observed stream を cover すること。
      exclusive / private-path semantics による private temporary creation と cleanup。
      timeout/cancellation/crash 後に child process が残らないこと。resource-limit record と
      unsupported-limit diagnostic。unsupported required limit が `Error` になること。polarity
      mismatch、formula/substitution evidence payload/ref 不在、candidate metadata mismatch、
      unsupported required limit、incomplete stream、または timeout/cancellation/crash/parsing
      corruption 後の otherwise matching evidence での `Proved` rejection。observed result が
      `ExpectedBackendResult::Unsat` と一致し、supported payload/ref と candidate metadata が一致する場合だけ
      mock `Proved` になること。kernel/SAT checking、proof policy、witness/cache publication、
      backend proof-method trust、resolution-trace trust、unsat-core trust、
      SMT proof-object trust、trusted backend `used_axioms` がないこと。
    - Status: 完了。`src/backend.rs` は generic direct-spawn child-process runner、
      deterministic input / command / stream metadata hashing、stdin と private problem-file
      mode、version probe、timeout / cancellation / crash / missing-executable handling、
      drain-safe stdout/stderr capture、unsupported required-limit の fail-closed behavior、
      private temp cleanup、mock observation classification を実装する。`Proved` は
      candidate-evidence-only のままであり、matching `Unsat`、supported
      formula/substitution payload/ref、matching target / input / label / symbol / provenance
      metadata を要求する。real backend adapter、backend-specific parser、real output からの
      formula/substitution candidate extraction、portfolio、proof policy、witness/cache
      publication、kernel checking は deferred のままである。
    - 依存: 13。仕様: `backend.md`。

15. **最初の具体バックエンド統合。** [x] deferred / external_dependency_gap
    - 最初のバックエンドの決定を解決し、実バックエンド 1 つをエンド
      ツーエンドで統合する: problem 出力、実行、出力を kernel スキーマに
      対する formula/substitution evidence candidate として extract する。
    - テスト: バックエンド存在ガード付きの統合テスト。candidate evidence が
      `mizar-kernel` の構造検証を通り、kernel checking まで untrusted のままである。
    - 依存: 10 または 12（選んだバックエンドに応じて）、14、
      `mizar-kernel` task 25-28。仕様: `backend.md`。
    - Gate result: deferred。`mizar-kernel` task 25-28 と `mizar-vc` handoff は
      存在するが、real backend output を kernel-owned formula/substitution candidate
      payload に変換する paired evidence-extraction spec / source module が
      `mizar-atp` に存在しない。supported architecture-10 backend executable
      (`vampire`, `eprover`, `cvc5`, `z3`) は task-15 environment で見つからず、
      backend-available integration fixture は skip され、adapter を検証できない。
      real adapter、backend-specific parser、fake candidate schema、kernel call、
      proof-policy hook、witness/cache output、trusted backend proof material は追加しない。
      English/Japanese `evidence.md` spec（または同等の backend-specific extraction spec）が
      candidate payload/ref contract を定義し、supported backend route を guarded integration
      test で利用できるようになった後にだけ、この task を再開する。

16. **結果分類と極性検証。** [x] deferred / external_dependency_gap
    - バックエンドの結果を分類する（proved、反例、タイムアウト、
      unknown、エラー）。観測結果が `expected_result` に一致し candidate
      formula/substitution evidence が存在するときのみ `Proved` を発行する。
      反例は診断のみに供給する。
    - テスト: 結果ごとの分類フィクスチャ。極性不一致ケースが proved に
      分類されない。
    - 依存: 15 に加え task-15 reopen condition、すなわち paired evidence-extraction
      spec と guarded supported backend route が存在すること。real-output classification は
      その後に実装する。仕様: `backend.md`（分類の節）。
    - Gate result: deferred。task 14 は process-status と mock candidate classification
      invariant を既に cover している。real-output parsing と polarity fixture は task-15 の
      extraction route を必要とするが、その route は paired evidence-extraction spec / source
      module と guarded supported backend がないため blocked のままである。backend-specific
      parser、fake observed output schema、adapter-specific classification table、kernel call、
      trusted backend proof material は追加しない。extraction route と supported backend fixture が
      存在した後に task 15 と共に再開する。

### portfolio

17. **仕様: `portfolio.md`。** [x]
    - portfolio の仕様を執筆する（英語と日本語、コードなし）: VC ごとの
      portfolio タスク、候補証拠の収集、early stop、リソース予算、そして
      「勝者選択は `mizar-proof` のポリシーであり、完了順が結果を決める
      ことは決してない」という境界。early stop は、保留中の候補が選択済み
      class を覆せないと proof policy が報告した後に限って許可する。
    - 依存: 13。仕様: アーキテクチャ 10「Portfolio Execution」、
      [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)
      「ATP Portfolio Service」。
    - 状態: docs-only task として完了。`portfolio.md` は policy-neutral な portfolio
      planning、candidate collection、deterministic candidate ordering、resource budget、
      cancellation / early-stop constraint、kernel / proof-policy handoff boundary を定義する。
      Rust source、proof policy evaluator、kernel call、witness/cache publication、real backend
      evidence extractor、fake real-output schema、trusted backend proof material は追加しない。

18. **portfolio 実行。** [x]
    - 決定的な候補順と協調的キャンセルを備えた portfolio 構築と候補収集を
      実装する。
    - テスト: 完了順をシャッフルしても同一の候補集合と順序。cancellation は
      partial candidate を残さず、early-stop oracle を fabricate しない。
    - 依存: 14、16、17。仕様: `portfolio.md`。
    - 状態: task-17 no-early-stop boundary 内で完了。`src/portfolio.rs` は prebuilt
      `BackendRunInput` value から deterministic plan を構築し、same-problem membership を
      検証し、terminal `BackendRunResult` value を policy-neutral な `PortfolioEvidenceSet` に
      収集し、completion order から独立して formula/substitution candidate を順序付け、partial
      candidate を出さずに cancellation する。kernel call、proof policy evaluation、
      witness/cache/artifact state、early-stop policy finality、real backend extractor、trusted
      backend proof material は実装しない。

19. **ATP 実行メタデータの記録。** [x]
    - artifact と再現性記録のために、シード、タイムアウト設定、
      バックエンドの識別/バージョン、リソース使用を read-only backend
      run-metadata projection として記録する。
    - stream/resource usage と diagnostic を含めるが、runtime observation は
      trusted acceptance material と downstream candidate hash の外に保つ。
    - テスト: メタデータ完全性のフィクスチャ。メタデータが意味論ハッシュ
      から除外される。
    - 依存: 16。仕様: アーキテクチャ 00「Incrementality and
      Reproducibility」、`backend.md`。
    - 境界: artifact writing、proof policy、kernel checking、witness/cache
      publication、real backend extraction、trusted backend proof material は追加しない。
    - 状態: backend-runner metadata boundary 内で完了。`BackendRunMetadata` は
      `BackendRunResult` から seed、timeout setting、backend identity / version record、
      command fingerprint、stream/resource usage、elapsed time、diagnostic を project し、
      command identity、candidate evidence、kernel check、proof policy、artifact/cache/witness
      publication は変更しない。

### 強化と横断フォローアップ

20. **コーパスとモックバックエンド統合スイート。** [x]
    - モックバックエンドで駆動する stage `advanced_semantics` のコーパス
      ケースを `spec_trace.toml` 項目付きで追加する。
    - `mizar-test` にはまだ active `advanced_semantics` runner / tag gate がないため、
      metadata-only の `tests/property` fixture を使う。crate-local integration test は
      それらの fixture を読み、既存の mock backend runner、mock observed-result classification、
      portfolio collection API に流してよい。
    - formula/substitution candidate handoff、counterexample recording、unknown/open result を
      cover する。ただし kernel checking、proof policy、real-output extraction、
      witness/cache/artifact publication、placeholder evidence schema は追加しない。
    - 依存: 18。仕様: [staged_model.md](../../mizar-test/ja/staged_model.md);
      `portfolio.md`。
    - 状態: metadata-only corpus と crate-local mock backend boundary 内で完了。
      `tests/property/atp_mock_backend_integration_001.*` は inert な
      `advanced_semantics` corpus anchor を記録し、`tests/mock_backend_corpus.rs` は
      formula/substitution、counterexample、unknown/open case を既存の mock backend
      classification と portfolio collection に通す。active `.miz` advanced-semantics
      execution、real-output extraction、kernel checking、proof policy、witness/cache/artifact
      publication、placeholder evidence schema は deferred/external のまま残る。

21. **決定性スイート。** [x]
    - 同一の `VcIr` 入力がモックバックエンドの下で同一の problem、
      エンコーディング、候補順を生むことのプロパティ的検証。
    - 依存: 18。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。
    - 状態: mock-backend candidate-production boundary 内で完了。
      `tests/determinism_suite.rs` は同一の public `VcIr` fixture を構築し、VC kernel
      handoff を再構築し、TPTP FOF と SMT-LIB uninterpreted の両 profile を translate し、
      byte-identical な encoder text と side metadata を確認し、planned-run order と
      backend completion order を反転しても portfolio candidate handoff が deterministic
      であることを検証する。real backend extraction、kernel checking、proof policy、
      artifact witness、proof-cache promotion、active `.miz` advanced-semantics execution は
      deferred/external のまま残る。

22. **公開 enum の前方互換性ポリシー。** [x]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用し、所有
      モジュール仕様に決定を記録する。
    - 依存: 18。仕様: 全モジュール仕様。
    - Status: public API compatibility boundary 内で完了。`problem`、
      `translator`、`property_encoding`、`tptp_encoder`、`smtlib_encoder`、
      `backend`、`portfolio` のすべての public enum は downstream 向け
      `#[non_exhaustive]` であり、所有する英語/日本語 spec は source の exact
      inventory を記録し、`lint_policy.rs` は source attribute と EN/JA inventory を
      検査する。behavior-sensitive match は意図的 fallback を document していない
      限り explicit/fail-closed のままである。trusted acceptance、backend、
      kernel、witness、proof-policy、cache behavior は追加しない。

23. **ソース/仕様対応監査。** [x]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - 依存: 22。仕様: 全モジュール仕様と本 TODO。
    - Status: audit-only task として完了。paired `source_spec_audit.md` document は
      現在の public module、public top-level item、public entry function、
      cross-module evidence、ATP-AUDIT follow-up register を inventory し、
      `lint_policy.rs` は audit を現在の source と EN/JA gap id/class に照合する。
      source behavior、public API、backend route、kernel call、proof policy、witness、
      cache、placeholder downstream integration は追加しない。

24. **二言語ドキュメント同期監査。** [x]
    - `doc/design/mizar-atp/en/` の各英語正本と日本語版を比較し、内容を
      同期する。
    - 依存: 23。仕様: リポジトリのドキュメント方針。
    - Status: audit-only task として完了。paired `bilingual_sync_audit.md`
      document は現在の EN/JA design doc pair をすべて inventory し、bilingual drift
      または `repo_metadata_conflict` がないことを記録し、分類済み
      external/deferred gap を保持する。`lint_policy.rs` は EN/JA Markdown filename の
      完全一致と必要な sync-audit marker を検査する。source behavior、public API、
      backend route、kernel call、proof policy、witness、cache、placeholder downstream
      integration は追加しない。

25. **portfolio 完了順独立性 gate。** [x]
    - adversarial な完了順を持つ mock backend で portfolio-specific regression
      gate を追加する。candidate collection が早期終了できるのは、保留中の
      candidate が選択済み class を覆せないと `mizar-proof` policy が報告する
      場合だけである。生の完了時刻を proof identity にしてはならない。
    - テスト: release policy の下では、後から返った kernel-verifiable candidate
      が先に返った externally attested result より優先される。tie は
      deterministic backend priority / evidence strength / problem hash key
      で解く。cancel または kill された敗者 backend が部分的な accepted state を
      残さない。
    - 依存: 18、21、`mizar-proof` task 7、9、12、13。仕様:
      [10.atp_backend_integration.md](../../architecture/ja/10.atp_backend_integration.md),
      [14.parallel_verification_and_scheduling.md](../../architecture/ja/14.parallel_verification_and_scheduling.md),
      [22.incremental_verification_contract.md](../../architecture/ja/22.incremental_verification_contract.md)。
    - Status: deferred/external_dependency_gap completion。Task 25 は
      task 18、21、23、24 の後に release-policy completion-order gate を
      再評価した。`mizar-proof` は workspace crate ではなく、proof-policy
      task 7、9、12、13 はここで利用できないため、後から返った
      kernel-verifiable candidate と先に返った externally attested result の
      winner test、tie policy、early-stop finality oracle は、proof-policy
      境界を越えずに `mizar-atp` 内へ実装できない。既存の task-18/task-21
      coverage は、shuffled completion order 下の crate-local no-early-stop
      deterministic candidate handoff を既に guard している。この task は
      external_dependency_gap を記録し、paired docs を更新し、mock
      proof-policy oracle、placeholder `mizar-proof` adapter、accepted proof state、
      kernel call、witness/cache output、trusted backend proof material を追加しない
      ことを lint-policy guard で固定する。

26. **architecture-22 フォローアップ監査。** [x]
    - task 25 の portfolio ordering と early-stop 契約について、ソース/仕様
      対応監査と二言語ドキュメント同期監査を再実行する。残る policy-boundary
      または completion-order gap をフォローアップタスクとして記録する。
    - 依存: 25。仕様: 全モジュール仕様、本 TODO、リポジトリの
      ドキュメント方針。
    - Status: audit-only follow-up として完了。source/spec audit と bilingual
      sync audit は、backend completion order と runtime duration を semantic
      proof identity にしてはならないという Architecture 22 の規則に、task-25
      completion-order deferral を明示的に結び付けた。新しい source/spec drift、
      bilingual drift、repo metadata conflict、追加 follow-up gap は見つからなかった。
      `ATP-AUDIT-G005` は、`mizar-proof` が存在し release policy finality、winner
      selection、tie-breaking、candidate displacement を所有するまで、単一の
      policy-boundary / completion-order follow-up として残る。

27. **module 境界リファクタリング gate。** [ ]
    - crate を下流 consumer 向けに完了扱いにする前に、source layout を監査し、
      oversized file、混在した責務、module table と module spec 境界に沿って
      分割すべき private helper を洗い出す。review bottleneck になった実装
      ファイルは、公開 API、診断、決定的 rendering、artifact-facing schema、
      consumer-visible behavior を変えずに private module へ分割する。
    - 分割後は必要に応じて本 module table / source path を更新し、移動した
      API について source/spec 対応監査と二言語ドキュメント同期監査の範囲を
      再実行する。挙動 cleanup や API 公開を移動と混ぜない。それらは独立した
      spec task を要求する。
    - 依存: 26。仕様: 本 TODO、
      [internal 07](../../internal/ja/07.crate_module_layout.md)、全モジュール仕様。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-atp
cargo clippy -p mizar-atp --all-targets -- -D warnings
```

VC や kernel の境界に触れるタスクでは追加で実行する:

```text
cargo test -p mizar-vc
cargo test -p mizar-kernel
```

portfolio ordering と early-stop のタスクでは追加で実行する:

```text
cargo test -p mizar-proof
```

テストが通ったらここでタスクにチェックを付ける。

## 備考

- ここで生産されるものはすべて untrusted な証拠である。信頼された状態は
  kernel evidence checking 後にのみ存在し、受理ポリシーは `mizar-proof` にある。
- エンコーディングは可逆である必要はないが、バックエンドに見えるすべての
  論理式は `AtpProvenance` で追跡可能でなければならない。backend-reported
  used axiom は trusted `used_axioms` ではない。downstream witness material に
  つながってよいのは kernel-checked formula/provenance evidence だけである。
- バックエンドの非決定性は記録される（シード、バージョン、時間）。黙って
  吸収されることはない。Mizar 側の翻訳とエンコーディングはビット安定で
  ある。
- ATP が利用不能でも前段の phase を壊してはならない。この crate は
  パイプラインの他所のエラーではなく、`open` の VC 状態へ退化する。
- Backend proof method と log は diagnostic または extraction input のみである。
  `AtpProvenance`、kernel evidence、trusted handoff material、resolution trace
  certificate ではない。
