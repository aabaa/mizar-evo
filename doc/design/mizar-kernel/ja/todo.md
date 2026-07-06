# mizar-kernel TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様は、それを引用する実装タスクより前に、専用の仕様タスクが
（英語と日本語を同じ変更で）執筆する。モジュール名は
[internal 07](../../internal/ja/07.crate_module_layout.md) の最小分割に従う。
この crate はアーキテクチャ 15、16、19 と internal 04 を精緻化する。
すべてのモジュール仕様は kernel の禁止事項を再掲しなければならない:
証明探索、ヒューリスティック選択、オーバーロード解決、cluster 探索、ATP
探索、暗黙の coercion 挿入、フォールバック推論の禁止。closeout 後の
SAT-backed correction ではこれを精緻化する: trusted SAT checking が許可されるのは、
呼び出し側が与えた formula、substitution、provenance、target/goal binding から
kernel が導出した SAT problem に対してだけであり、formula や substitution の選択は
引き続き禁止される。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| clause | `clause.md`（task 2） | `src/clause.rs` | [x] |
| certificate_parser | `certificate_parser.md`（task 4） | `src/certificate_parser.rs` | [x] |
| formula_evidence | `formula_evidence.md`（task 23） | `src/formula_evidence.rs` | [x] |
| rejection | `rejection.md`（task 6） | `src/rejection.rs` | [x] |
| resolution_trace | `resolution_trace.md`（task 8） | `src/resolution_trace.rs` | [x] |
| sat_encoding | `sat_encoding.md`（task 23） | `src/sat_encoding.rs` | [x] |
| sat_checker | `sat_checker.md`（task 23） | `src/sat_checker.rs` | [x] |
| substitution_checker | `substitution_checker.md`（task 10） | `src/substitution_checker.rs` | [x] |
| checker | `checker.md`（task 13） | `src/checker.rs` | [x] |

`mizar-kernel` はパイプライン phase 14 を実装する。入力は kernel evidence と
immutable kernel context、出力は信頼された証明状態である。verifier 全体の信頼された
中核（Small Kernel 原則）であり、証拠の検証のみを行う。closeout 後の target
は formula/substitution evidence in、trusted proof status out である:
エビデンスを parse / validate し、provenance を検査し、substitution を検証して
適用し、instantiated formula を導出し、それを決定的に SAT へ encode し、
trusted in-process Rust SAT checker が refutation を報告する場合だけ受理する。
task-22 実装は
MiniSAT 互換 resolution-trace checker のままであり、task 23-29 が acceptance
path を置き換えるまで `source_drift` / `design_drift` と分類する。証明書や
evidence record は証拠であって受理ではない。信頼されるのはこの crate の
肯定的結果のみであり、その上のポリシー射影は `mizar-proof` に属し、ここには
属さない。

依存順序: `clause` → `certificate_parser` / `rejection` →
`resolution_trace` / `substitution_checker` → legacy `checker`。closeout 後の
correction order は `formula_evidence` → `substitution_checker` →
`sat_encoding` / `sat_checker` → `checker`（統括、import 済み事実、
SAT-backed acceptance、明示的エビデンスがある場合の cluster 再生）である。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は `mizar-session` と `mizar-core`（core の論理式表現と、独立に
再検査する binder 契約）に依存する。Task 24 は、予定される唯一の production
dependency 追加として `batsat = { version = "=0.6.0", default-features = false }`
を選択する。Task 27 は audit 済み `sat_checker` wrapper の背後に限ってこれを
追加してよい。それは deterministic、in-process、resource-bounded で、
backend process execution を含まないものでなければならない。その他の依存追加には
記録された正当化が必要である。`mizar-atp` と `mizar-proof` がこの crate に
依存するのであって、逆は決してない。アーキテクチャ:
[15.kernel_certificate_format.md](../../architecture/ja/15.kernel_certificate_format.md)、
[16.substitution_and_binding.md](../../architecture/ja/16.substitution_and_binding.md)、
[17.cluster_trace_format.md](../../architecture/ja/17.cluster_trace_format.md)、
[08.reasoning_boundary.md](../../architecture/ja/08.reasoning_boundary.md)。
統合: [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)。

## 解決済みおよび保留中の決定

- **formula/substitution evidence correction: 未解決。task 23-29 が所有する。**
  アーキテクチャ 15 は ATP 行き VC について、resolution-trace certificate
  acceptance path を supersede した。kernel evidence は formula、substitution、
  provenance、target/goal binding を保持し、kernel が instantiated formula と
  deterministic SAT problem を導出する。backend の proof method は trusted
  evidence ではない。現在の source は post-closeout correction task が着地するまで
  `source_drift` と分類する。
- **trusted SAT dependency: task 24 で解決済み。source integration は task 27
  待ち。** kernel は task 27 が wrapper を統合した後、direct
  `batsat = { version = "=0.6.0", default-features = false }` を small kernel の
  一部として信頼してよい。Audit は version pinning、determinism、limit、unsafe
  usage、no-process/no-network constraint、却下した候補、lockfile expectation を
  記録する。kernel は外部 SAT/ATP process を呼び出したり ATP search を実装したり
  してはならない。
- **legacy certificate schema ownership: task 4 で解決済み。** アーキテクチャ
  15 は以前 normalized certificate format を定義し、`mizar-kernel` がそれらの
  legacy certificate schema type、schema-version table、section tag、byte grammar、
  parser-owned failure location を migration gate が retire または isolate するまで所有する。
  将来の `mizar-atp` などの evidence producer は formula/substitution evidence schema を
  構築しなければならず、kernel は evidence producer に依存しない。producer /
  consumer integration は、それらの crate が存在するまで `external_dependency_gap`
  のままである。この決定は resolution-trace acceptance path については legacy
  であり、通常 proof acceptance については task-23 formula/substitution evidence schema に
  supersede される。
- **trusted ベースラインの crate ポリシー: task 1 と task 24 で解決済み。
  source guard update は task 27 待ち。**
  trusted kernel source は unsafe code を forbid し、workspace lint denial を
  使い、production dependency を `mizar-session` と `mizar-core` に限定して
  dev/build/target dependency escape hatch を持たず、crate-root trust statement
  を必須にし、paired module spec が存在するまで public semantic surface を
  禁止し、downstream ATP/proof/cache/artifact coupling を guard する。task 24 は
  `mizar-session` と `mizar-core` に加えて audit 済み direct SAT checker
  dependency をちょうど 1 つだけ許す形へ policy を改訂する。task 27 は manifest
  を編集するときに、この exact allow-list を `tests/lint_policy.rs` に符号化しなければならない。
- **discharge 証拠の検証範囲: 未解決。`mizar-proof` task 6 が所有する。**
  `mizar-vc` の pre-ATP discharge 証拠を kernel が再生するか、ポリシー
  レベルの built-in 証拠として受理するか。再生が選ばれた場合、再生
  checker はフォローアップタスクとしてここに着地する。トップレベルで
  追跡する。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-kernel` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### clause と証明書の基盤

1. **crate の足場と trusted ベースライン lint 方針。** [x]
   - `mizar-session` と `mizar-core` にのみ依存する workspace メンバー
     `mizar-kernel` を追加する。trusted ベースラインの決定を解決し、
     `tests/lint_policy.rs`（deny ベースライン＋trusted コードの追加分）に
     符号化する。
   - テスト: lint 方針ガードが通る。依存集合が宣言どおりである。
   - 依存: `mizar-core` task 5。仕様: internal 07「Kernel and Proof」。

2. **仕様: `clause.md`。** [x]
   - アーキテクチャ 15「Clause Representation」に従って clause 表現の
     仕様を執筆する（英語と日本語、コードなし）: リテラル、正準順序、
     構造的 well-formedness、trust 文。
   - 依存: 1。仕様: アーキテクチャ 15。

3. **clause 表現の実装。** [x]
   - 構造検証と決定的レンダリングを備えた clause を実装する。
   - テスト: well-formed/不正のフィクスチャ。正準順序。レンダリングの
     安定性。
   - 依存: 2。仕様: `clause.md`。

4. **仕様: `certificate_parser.md`。** [x]
   - 証明書の仕様を執筆する（英語と日本語、コードなし）: アーキテクチャ
     15 のトップレベルスキーマ、format タグ、バックエンドメタデータ、
     構造検証規則、スキーマ所有権の決定。
   - 依存: 2。仕様: アーキテクチャ 15「Certificate Top Level」「Trust
     Scope」。

5. **証明書の構文解析と構造検証の実装。** [x]
   - 証明書をスキーマ型へ構文解析し、構造検証のみを行う — 構文解析が
     意味論的信頼を与えることはない。
   - テスト: ラウンドトリップ。不正な証明書は位置付きで拒否。未知の
     format タグは拒否。
   - 依存: 4。仕様: `certificate_parser.md`。

6. **仕様: `rejection.md`。** [x]
   - 拒否意味論の仕様を執筆する（英語と日本語、コードなし）:
     アーキテクチャ 15「Kernel Rejection Semantics」とアーキテクチャ 19 に
     従う安定した拒否カテゴリと構造化された理由。
   - 依存: 1。仕様: アーキテクチャ 15、
     [19.failure_semantics.md](../../architecture/ja/19.failure_semantics.md)。

7. **拒否レコードの実装。** [x]
   - 後続のすべての checker が使う拒否カテゴリ/理由を実装する。バック
     エンドが成功を報告していても、拒否は証明エラーである。
   - テスト: カテゴリの安定性。理由が証明書内の位置を保持する。
   - 依存: 5、6。仕様: `rejection.md`。

### checker 群

8. **仕様: `resolution_trace.md`。** [x]
   - resolution trace 検査の仕様を執筆する（英語と日本語、コードなし）:
     MiniSAT 互換の trace ステップ、clause resolution の検証、
     アーキテクチャ 15「Resolution Trace」に従う線形再生上限。
   - 依存: 4。仕様: アーキテクチャ 15。

9. **resolution trace checker の実装。** [x]
   - clause resolution trace をステップごとに検査し、従わないステップを
     拒否する。
   - テスト: 有効な trace の受理。各 1 ステップ変異の拒否。trace サイズに
     線形な再生コスト。
   - 依存: 7、8。仕様: `resolution_trace.md`。

10. **仕様: `substitution_checker.md`。** [x]
    - 置換検査の仕様を執筆する（英語と日本語、コードなし）: アーキテクチャ
      15「Substitution Rule」とアーキテクチャ 16 に従う置換検証、alpha
      変換検査、自由変数条件。`mizar-core` の binder ライブラリの論理を
      再利用するのではなく、独立に再検査する。
    - 依存: 4。仕様: アーキテクチャ 15、16。

11. **置換検査の実装。** [x]
    - 置換の適用を証明書が主張する結果に対して検証する。
    - テスト: 有効な置換の受理。捕獲違反の拒否。結果不一致の拒否。
    - 依存: 7、10。仕様: `substitution_checker.md`。

12. **alpha 変換、新鮮性、自由変数検査の実装。** [x]
    - alpha 同値の主張、deterministic freshness witness、自由変数の側条件を検査する。
    - テスト: 同値のフィクスチャ。freshness counter mismatch と自由変数条件違反の拒否。
    - 依存: 11。仕様: `substitution_checker.md`。

### 統括と受理

13. **仕様: `checker.md`。** [x]
    - kernel check サービスの仕様を執筆する（英語と日本語、コードなし）:
      `KernelCheckInput`/`KernelCheckResult`、サブ checker 上の検査
      パイプライン、アーキテクチャ 15 に従う import 済み事実の検査、
      アーキテクチャ 17 に従う cluster trace の再生、受理条件 — kernel の
      禁止事項を再掲する。
    - 依存: 6、8、10。仕様: アーキテクチャ 15「Imported Facts」、
      [17.cluster_trace_format.md](../../architecture/ja/17.cluster_trace_format.md)、
      [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)
      「Kernel Check Service」。

14. **import 済み事実検査の実装。** [x]
    - 証明書が使う事実が宣言された import 済み事実と正確に一致することを
      検証する（content-addressed 参照、黙った追加なし）。
    - テスト: 未宣言事実の使用の拒否。ハッシュ不一致の拒否。
    - 依存: 13。仕様: `checker.md`（import 済み事実の節）。

15. **cluster trace 再生の実装。** [x]
    - `ResolutionTrace` の cluster/reduction ステップを線形時間で再生し、
      主張された事実を再導出できない trace を拒否する。
    - テスト: 有効な trace の再生。前件/導出事実の変異の拒否。再生コスト
      上限の強制。
    - 依存: 13、14。仕様: `checker.md`（cluster 再生の節）、
      アーキテクチャ 17。Upstream `mizar-checker` trace production は ready payload
      contract が存在しない限り `external_dependency_gap` のままである。

16. **kernel check サービスと決定的バッチ順序。** [x]
    - サービス API を実装する: 証明書 1 つに対して信頼された結果 1 つ。
      target VC fingerprint と、同一 target では caller input order による
      in-crate batch checking。
    - テスト: サービスのラウンドトリップ。caller input order のシャッフルと
      equal-target ties の下でも batch order が決定的。
    - 依存: 9、12、14、15。仕様: `checker.md`、
      [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)。

### 強化と横断フォローアップ

17. **健全性 fail テストコーパス。** [x]
    - 変異ベースの健全性スイートを構築する: すべての checker に対して、
      体系的に変異させた証明書/trace が拒否されなければならない
      （テスト戦略と [fail_soundness.md](../../mizar-test/ja/fail_soundness.md)
      に従う fail 重視）。
    - 依存: 16。仕様: [fail_soundness.md](../../mizar-test/ja/fail_soundness.md)、
      [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

18. **決定性と再生コストのスイート。** [x]
    - 同一入力が同一の結果と拒否理由を生み、再生が文書化されたコスト上限
      内に収まることのプロパティ的検証。
    - 依存: 16。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

19. **公開 enum の前方互換性ポリシー。** [x]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用する。拒否
      カテゴリはさらにアーキテクチャ 19 の互換性ポリシーに従う。
    - 依存: 16。仕様: [public_enum_policy.md](./public_enum_policy.md) と、
      その inventory が参照する module specs。

20. **ソース/仕様対応と禁止事項の監査。** [x]
    - 全公開 API と約束された挙動を実装とテストへトレースする。すべての
      モジュール仕様が kernel の禁止事項と trust 文を再掲していることを
      検証する。
    - 依存: 19。仕様: 全モジュール仕様と本 TODO。

21. **二言語ドキュメント同期監査。** [x]
    - `doc/design/mizar-kernel/en/` の各英語正本と日本語版を比較し、内容を
      同期する。
    - 依存: 20。仕様: リポジトリのドキュメント方針。

22. **module 境界リファクタリング gate。** [x]
    - crate を下流 consumer 向けに完了扱いにする前に、source layout を監査し、
      oversized file、混在した責務、module table と module spec 境界に沿って
      分割すべき private helper を洗い出す。review bottleneck になった実装
      ファイルは、公開 API、診断、決定的 rendering、artifact-facing schema、
      consumer-visible behavior を変えずに private module へ分割する。
    - 分割後は必要に応じて本 module table / source path を更新し、移動した
      API について source/spec 対応監査と二言語ドキュメント同期監査の範囲を
      再実行する。挙動 cleanup や API 公開を移動と混ぜない。それらは独立した
      spec task を要求する。
    - 依存: 21。仕様: 本 TODO、
      [internal 07](../../internal/ja/07.crate_module_layout.md)、全モジュール仕様。

### Closeout 後の SAT-backed evidence correction

23. **仕様: kernel evidence format correction。** [x]
    - paired module spec を更新し、resolution-trace certificate を
      formula/substitution kernel evidence で supersede する。legacy
      resolution-trace acceptance path を `design_drift` / `source_drift` として
      分類し、外部 producer gap を記録し、与えられた evidence に対する SAT
      checking は許可されるが proof search は引き続き禁止であることを再掲する。
      Task 23 は paired `formula_evidence.md`、`sat_encoding.md`、
      `sat_checker.md` を追加し、proof 関連 language spec text を同期し、
      checker/rejection/resolution-trace docs を更新し、legacy certificate/trace input は
      normal-policy unsupported かつ migration/audit-only であることを記録する。
    - テスト: docs-only verification。
    - 依存: 22。仕様:
      [15.kernel_certificate_format.md](../../architecture/ja/15.kernel_certificate_format.md),
      [08.reasoning_boundary.md](../../architecture/ja/08.reasoning_boundary.md),
      [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)。

24. **仕様と監査: trusted SAT checker dependency。** [x]
    - kernel 内で信頼する pure-Rust MiniSAT-compatible SAT checker を選び、正当化する。
      version pinning、determinism requirement、resource limit、unsafe-code audit、
      no-process/no-network constraint、task-1 baseline からの lint/dependency policy
      revision、`sat_checker.md` が期待する wrapper API を記録する。
      結果: task 24 は `batsat = { version = "=0.6.0", default-features = false }`
      を選択し、expected transitive lockfile resolution を `bit-vec 0.5.1` とする。
      統合は task 27 に deferred。
    - テスト: docs-only verification。候補 crate が選ばれた後は dependency
      metadata audit も実行する。
    - 依存: 23。仕様: アーキテクチャ 15 "Post-Closeout Correction"。

25. **Formula/substitution evidence schema and parser。** [x]
    - formula ref または formula、substitution record、provenance binding、
      target/goal binding、stable hash の kernel-owned evidence schema を実装する。
      legacy certificate parsing は、新しい acceptance path の外側であることが
      明確な場合に限り互換性のために残してよい。
    - テスト: structural round-trip。不正 evidence の拒否。provenance gap は
      fail-closed。deterministic rendering / hashing。
    - 依存: 23、24。仕様: task 23 の `formula_evidence.md`。

26. **Formula instantiation and deterministic SAT encoding。** [x]
    - substitution side condition を検証し、evidence formula から instantiated
      formula を導出し、結果の formula set と negated/target goal を deterministic
      SAT problem へ encode する。instantiated formula と SAT clause は
      kernel-derived check artifact であり、trusted input ではない。encoding は
      premise を選んだり、substitution を発明したり、backend-method proof trace を
      trusted input へ隠したりしてはならない。
    - テスト: valid instantiation は安定して encode される。capture と
      provenance mutation は拒否される。同値な caller order は同一 SAT bytes を
      生成する。
    - 依存: 25。仕様: `formula_evidence.md`, `sat_encoding.md`,
      [16.substitution_and_binding.md](../../architecture/ja/16.substitution_and_binding.md)。

27. **Trusted SAT checker wrapper。** [x]
    - audit 済み Rust SAT checker を小さな deterministic wrapper の背後に統合する。
      kernel が構築した SAT problem が unsatisfiable かどうかを判定するために
      必要な操作だけを公開し、limit を強制し、solver error を安定した kernel
      rejection へ変換する。
    - テスト: satisfiable kernel-derived SAT problem は non-acceptance wrapper
      evidence を返す。unsatisfiable problem は UNSAT wrapper evidence を返す。
      limit、unsupported clause、solver error は決定的に拒否される。
      dependency / lockfile guard は exact `batsat` / `bit-vec` resolution を強制し、
      alternate SAT/process dependency を拒否する。wrapper test は deterministic
      `batsat` heuristic options が pin され caller に expose されないことを証明する。
    - 依存: 24、26。仕様: task 23 の `sat_checker.md`。

28. **SAT-backed kernel check service。** [x]
    - trusted acceptance path を置き換え、`checker` が、検証済み
      formula/substitution evidence から kernel が導出した SAT problem を trusted SAT
      checker が refute した場合だけ受理するようにする。imported-fact、
      provenance、cluster-trace、used-axiom extraction は fail-closed のまま保つ。
    - テスト: end-to-end の accepted/rejected evidence fixture。mutated
      substitution、missing premise、satisfiable goal、context mismatch は拒否。
      batch ordering は deterministic のまま。
    - 依存: 25、26、27。仕様: `checker.md`,
      [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)。

29. **Migration audit and quality re-review。** [x]
    - legacy resolution-trace public surface を retire、gate、または明示的に legacy
      と印付けし、downstream crate が target acceptance path と誤認しないように
      する。source/spec、bilingual、prohibition、dependency、quality audit を再実行し、
      `mizar-vc`、`mizar-atp`、`mizar-proof`、`mizar-cache`、`mizar-artifact` の
      残る `external_dependency_gap` を記録する。
    - テスト: full `mizar-kernel` verification、実用上可能な workspace verification、
      docs diff check、new closeout claim の前に 90/100 以上の quality review score。
    - 依存: 28。仕様: 本 TODO と autonomous crate exit criteria。

### 健全性監査フォローアップ(2026-07-03)

[soundness_argument.md](./soundness_argument.md) は追加実装に先立って信頼
受理境界を監査し、所見 F1-F9 と `tests/certificates/fail/` 配下の 23 件の
reject-first corpus を記録した。High 2 件は同一変更(`f75af877`)で
architecture レベルで修正済みであり、以下のタスクが実装とスキーマの作業を
引き受ける。全所見はタスクまたは記録済みの処置に対応する:

| 所見 | 処置 |
|---|---|
| F1(goal polarity) | architecture 15 修正済み。kernel 実装は task 30。producer-side polarity declaration/rejection は [mizar-vc task 27](../../mizar-vc/en/todo.md) で解決済み |
| F2(非 import ソース束縛) | architecture 15 で fail-closed に修正済み。スキーマと検証は task 31(mizar-vc の context-identity payload と対) |
| F3(solver step budget) | `sat_checker.md` で設計上 deferral(batsat 0.6.0 に安定 budget API なし)。再訪トリガーを task 32 として記録 |
| F4(KernelEvidence field drift) | `f75af877` で解決。追加タスクなし |
| F5(fingerprint 衝突耐性) | `f75af877` で architecture 15 に制約を追加。追加タスクなし — 将来の fingerprint 登録はこの制約を満たすこと |
| F6(imported-statement projection) | task 33(mizar-vc と対) |
| F7(mizar-test soundness 語彙) | [mizar-test](../../mizar-test/en/todo.md) の監査フォローアップが所有 |
| F8(corpus ディレクトリ命名) | [mizar-test](../../mizar-test/en/todo.md) の監査フォローアップが所有 |
| F9(レガシー tautology marker) | task 34 |

30. **check service における goal-polarity 束縛(F1、不変条件 B4)。** [ ]
    - architecture 15「Goal Polarity Is Bound By The Target Obligation」を
      実装する: task-28 の check service は呼び出し側の immutable kernel
      context から check kind を読み取り、宣言された goal polarity が一致
      しない evidence を `context_mismatch` として拒否する。証明義務は
      refutation polarity を要求し、`AssertTrueForConsistency` は明示的に
      consistency kind の検査でのみ受理可能。
    - 受け入れ条件: `fail_certificate_sat_goal_polarity_mismatch_001` が
      (先行する構造的理由ではなく)polarity 理由で拒否される。両 polarity
      × 両 check kind の Rust regression を持つ。`soundness_argument.md` の
      不変条件 B4 が実装済みと記録される。
    - 検証: `cargo test -p mizar-kernel`、`cargo test -p mizar-test`、
      `cargo clippy -p mizar-kernel --all-targets -- -D warnings`。
    - 依存: 28。仕様: architecture 15(監査後本文)、`checker.md`、
      `sat_encoding.md`; soundness_argument.md F1/B4。

31. **非 import ソース束縛の context-identity 検証(F2、P クラス不変条件)。** [ ]
    - local-hypothesis / cited-premise / generated-VC-fact ソース束縛の検証
      データを仕様化・実装する: `FormulaEvidenceContext`(または immutable
      kernel context)を拡張して canonical な `mizar-vc` kernel-evidence
      handoff hash を運び、非 import 束縛それぞれがその hash に対して検証
      可能であることを受理の前提にする。payload を欠く evidence への現行の
      fail-closed `missing_provenance` 挙動は維持する。`formula_evidence.md`
      と architecture 15 を同一変更で更新する。producer 側 payload は対と
      なる mizar-vc タスク。
    - 受け入れ条件:
      `fail_certificate_symbols_unverifiable_local_hypothesis_001` が
      provenance 理由で拒否される。mizar-vc の payload 契約が存在すれば、
      有効な context-identity payload を持つ pass fixture が受理される。
      goal を hypothesis とラベルする ATP 側 mutation が拒否される。
    - 検証: `cargo test -p mizar-kernel`、`cargo test -p mizar-test`、
      `cargo clippy -p mizar-kernel --all-targets -- -D warnings`。
    - 依存: 30; 対: mizar-vc context-identity payload タスク。仕様:
      architecture 15「Context Identity Covers Non-Imported Source
      Bindings」、`formula_evidence.md`; soundness_argument.md F2、edge
      case 5。

32. **solver step-budget deferral の再訪(F3)。** [ ]
    - 可用性のみのギャップ: `batsat` 0.6.0 が決定的な conflict/propagation
      budget を公開しないため、solve 中に checker `timeout` が発火できない。
      固定した依存が変わる際に再評価する: budget API を採用する(task-24
      手続きで監査した新バージョンまたは代替)か、size-limit 根拠とともに
      deferral を再記録する。割り込み可能性のために決定性を弱めたり
      process 実行を追加したりしない。
    - 受け入れ条件: 依存バージョンを引用した決定が `sat_checker.md`
      (英日)に記録される。budget が入る場合、solve 途中の budget 枯渇を
      決定的にカバーする resource-rejection テストを持つ。
    - 検証: `cargo test -p mizar-kernel`; dependency/lockfile guard。
    - 依存: `batsat` バージョン変更がトリガー(task-24 監査手続き)。仕様:
      `sat_checker.md`; soundness_argument.md F3。

33. **imported-statement projection の仕様化(F6)。** [ ]
    - arch-18 の imported statement fingerprint(リッチな式)から evidence
      checker が比較する propositional formula-tree fingerprint への
      projection を仕様化し、現実的な imported fact を引用可能にする。
      これが入るまで fingerprint 等値規則が import 引用を fail-closed に
      保つ(健全)。kernel 側: `formula_evidence.md` + architecture 15 の
      projection 検証規則。producer 側は対となる mizar-vc/mizar-atp の
      スキーマ作業。
    - 受け入れ条件: projection が決定的かつ F5 制約に従い衝突耐性を持つ。
      projection された imported statement を引用する pass fixture を持つ。
      mutation fixture(誤った projection、stale fingerprint)が拒否される。
      `soundness_argument.md` の F クラス不変条件行を更新する。
    - 検証: `cargo test -p mizar-kernel`、`cargo test -p mizar-test`。
    - 依存: 31; 対: mizar-vc の dependency-slice/import projection タスク。
      仕様: architecture 15、18; soundness_argument.md F6。

34. **レガシー tautology marker の意味論(F9、low)。** [ ]
    - レガシー resolution-trace の tautology marker を確定または廃止する:
      現在の意味は profile 依存で希薄にしか仕様化されていない。推奨:
      migration/audit-only のレガシー path とともに廃止する。audit profile
      向けに残す場合は正確な受理効果を `resolution_trace.md`(英日)に
      仕様化する。
    - 受け入れ条件: marker がすべての policy の下で到達不能であることが
      文書化されるか、意味論が仕様節と mutation-rejection テストを持つ。
      誤ラベルは premise 弱化のみに留まる(受理強化にならない)。
    - 検証: `cargo test -p mizar-kernel`。
    - 依存: 29。仕様: `resolution_trace.md`; soundness_argument.md F9、
      L クラス不変条件。

35. **reduct-view エンコードに対する soundness argument の再訪。** [ ]
    - テンプレートエンコーディング監査
      ([template_encoding_audit.md](../../mizar-core/en/template_encoding_audit.md))
      は spec 05/13 で flattened structure widening を reduct-view 項
      (`view_{D→B}`)に置き換えた。mizar-core の view lowering が入り次第、
      attribute 述語が subject ごとに atomic であるという
      `soundness_argument.md` の前提を再訪する: structure widening に言及
      する certificate の形が変わるため、F クラスの式不変条件と attribute
      atom に触れる corpus seed を view 項に対して再点検する。
    - 受け入れ条件: `soundness_argument.md`(英日)が再監査結果を記録する。
      不変条件が変わる場合、corpus sidecar note を同一変更で更新する
      (本文書の constraint 節に従う)。
    - 検証: `cargo test -p mizar-kernel`、`cargo test -p mizar-test`。
    - 依存: 外部 — mizar-core の reduct/view lowering タスク; その後 31。
      仕様: spec 05 §5.8.3、13 §13.8.7; template_encoding_audit.md F1/F3。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-kernel
cargo clippy -p mizar-kernel --all-targets -- -D warnings
```

core の binder 契約や cluster 再生に触れるタスクでは追加で実行する:

```text
cargo test -p mizar-core
cargo test -p mizar-checker
```

テストが通ったらここでタスクにチェックを付ける。

## 備考

- kernel は証拠の検証のみを行う。証明探索、ヒューリスティック選択、
  オーバーロード解決、cluster 探索、ATP 探索、暗黙の coercion 挿入、
  フォールバック推論を決して行ってはならない。
- すでに与えられた formula/substitution evidence に対する trusted SAT
  checking は evidence check であり、proof search ではない。kernel は依然として
  premise を選んだり、substitution を発明したり、ATP backend を呼び出したり、
  fallback inference を行ったりしてはならない。
- バックエンドが成功を報告していても、kernel evidence validation の失敗は
  証明エラーである。外部で認証された証拠は `mizar-proof` のポリシーであり、
  kernel の結果では決してない。
- 依存集合を最小かつ監査済みに保つ。健全性に関わるコードは共有された
  巧妙さより重複を選ぶ（substitution checker は再利用ではなく再検査する）。
- この crate の近傍では fail/健全性テストが pass テストより優先される。
