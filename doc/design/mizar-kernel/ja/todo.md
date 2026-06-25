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
探索、暗黙の coercion 挿入、フォールバック推論の禁止。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| clause | `clause.md`（task 2） | `src/clause.rs` | [x] |
| certificate_parser | `certificate_parser.md`（task 4） | `src/certificate_parser.rs` | [x] |
| rejection | `rejection.md`（task 6） | `src/rejection.rs` | [x] |
| resolution_trace | `resolution_trace.md`（task 8） | `src/resolution_trace.rs` | [x] |
| substitution_checker | `substitution_checker.md`（task 10） | `src/substitution_checker.rs` | [x] |
| checker | `checker.md`（task 13） | `src/checker.rs` | [x] |

`mizar-kernel` はパイプライン phase 14 を実装する。入力は証明書と kernel
コンテキスト、出力は信頼された証明状態である。verifier 全体の信頼された
中核（Small Kernel 原則）であり、証拠の検証のみを行う — 証明書の構文解析と
構造検証、MiniSAT 互換の resolution trace 検査、置換/alpha/自由変数の検査、
cluster trace の再生、import 済み事実の検査。証明書は証拠であって受理では
ない。信頼されるのはこの crate の肯定的結果のみであり、その上のポリシー
射影は `mizar-proof` に属し、ここには属さない。

依存順序: `clause` → `certificate_parser` / `rejection` →
`resolution_trace` / `substitution_checker` → `checker`（統括、import 済み
事実、cluster 再生）。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は `mizar-session` と `mizar-core`（core の論理式表現と、独立に
再検査する binder 契約）にのみ依存する — 依存集合は意図的に最小であり、
追加には記録された正当化が必要である。`mizar-atp` と `mizar-proof` が
この crate に依存するのであって、逆は決してない。アーキテクチャ:
[15.kernel_certificate_format.md](../../architecture/ja/15.kernel_certificate_format.md)、
[16.substitution_and_binding.md](../../architecture/ja/16.substitution_and_binding.md)、
[17.cluster_trace_format.md](../../architecture/ja/17.cluster_trace_format.md)、
[08.reasoning_boundary.md](../../architecture/ja/08.reasoning_boundary.md)。
統合: [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)。

## 解決済みおよび保留中の決定

- **証明書スキーマの所有権: task 4 で解決済み。** アーキテクチャ 15 が
  証明書フォーマットを定義し、`mizar-kernel` が normalized certificate schema
  type、schema-version table、section tag、byte grammar、parser-owned failure
  location を所有する。将来の `mizar-atp` などの evidence producer はこの schema
  を構築してよいが、kernel は evidence producer に依存しない。producer /
  consumer integration は、それらの crate が存在するまで `external_dependency_gap`
  のままである。
- **trusted ベースラインの crate ポリシー: task 1 で解決済み。**
  trusted kernel source は unsafe code を forbid し、workspace lint denial を
  使い、production dependency を `mizar-session` と `mizar-core` に限定して
  dev/build/target dependency escape hatch を持たず、crate-root trust statement
  を必須にし、paired module spec が存在するまで public semantic surface を
  禁止し、downstream ATP/proof/cache/artifact coupling を guard する。
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

22. **module 境界リファクタリング gate。** [~]
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
- バックエンドが成功を報告していても、証明書検証の失敗は証明エラーで
  ある。外部で認証された証拠は `mizar-proof` のポリシーであり、kernel の
  結果では決してない。
- 依存集合を最小かつ監査済みに保つ。健全性に関わるコードは共有された
  巧妙さより重複を選ぶ（substitution checker は再利用ではなく再検査する）。
- この crate の近傍では fail/健全性テストが pass テストより優先される。
