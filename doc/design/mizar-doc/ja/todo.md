# mizar-doc TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様はまだ存在しない。各仕様は、それを引用する実装タスクより前に、
専用の仕様タスクが（英語と日本語を同じ変更で）執筆する。
[internal 07](../../internal/ja/07.crate_module_layout.md) により、この
crate はドキュメントのレンダリングと抽出の**両方**を所有する
（以前の phase-16 draft は別 crate `mizar-extract` を挙げていたが、現在の
architecture/internal docs は `mizar-doc` module 名を使う）。この crate はアーキテクチャ 13 と
internal 05 を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| artifact_reader | `artifact_reader.md`（task 2） | `src/artifact_reader.rs` | [ ] |
| doc_build | `doc_build.md`（task 4） | `src/doc_build.rs` | [ ] |
| comments | `comments.md`（task 6） | `src/comments.rs` | [ ] |
| links | `links.md`（task 9） | `src/links.rs` | [ ] |
| math | `math.md`（task 11） | `src/math.rs` | [ ] |
| render | `render.md`（task 13） | `src/render.rs` | [ ] |
| extract_select | `extract_select.md`（task 16） | `src/extract_select.rs` | [ ] |
| runtime_ir | `runtime_ir.md`（task 18） | `src/runtime_ir.rs` | [ ] |
| extract_backend | `extract_backend.md`（task 21） | `src/extract_backend.rs` | [ ] |
| publisher | `publisher.md`（task 23） | `src/publisher.rs` | [ ] |

`mizar-doc` はパイプライン phase 16 を実装する。入力は検証済み artifact と
ドキュメントコメント、出力はレンダリングされたドキュメント、検索索引、
抽出された実行コードである。phase 16 は消費 phase である: 公開された
artifact を読み、意味論解析を再実行せず、証明の有効性に影響せず、その
出力は自由に削除・再生成できる。ドキュメントは表示エラーに対して
緩やかに劣化してよいが、抽出は対照的に、未対応の実行時構文を拒否
しなければならない。

依存順序: `artifact_reader` → `doc_build` → `comments` → `links` /
`math` → `render`（ドキュメントの strand）、次に `extract_select` →
`runtime_ir` → `extract_backend`（抽出の strand）→ `publisher`。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は `mizar-session` と `mizar-artifact`（スキーマとリーダー）に
依存する。実際のエンドツーエンド入力は phase 15 の emission
（`mizar-artifact` task 17）を必要とする。それまではフィクスチャ
artifact で開発を進める。アーキテクチャ:
[13.documentation_and_extraction.md](../../architecture/ja/13.documentation_and_extraction.md)。
internal: [05](../../internal/ja/05.documentation_extraction.md)。

## 解決済みおよび保留中の決定

- **抽出はこの crate に属する: internal 07 により解決済み。**
  以前の phase-16 draft は `mizar-extract` を挙げていたが、internal 07 が
  レンダリングと抽出をここに統合し、現在の architecture/internal docs は
  `mizar-doc` module 名を使う。抽出が大きく育った場合は、その時点で
  分割判断を提起しトップレベルに登録する。
- **ドキュメントコメントの供給源: 未解決。task 6 で解決する。**
  ドキュメントコメントが phase 16 に届く方法を決める: emission 時に
  artifact へ射影する（既定候補。消費 phase の規則に忠実）か、
  `mizar-frontend` 経由で `PreprocessedSource` から再読込するか。この
  決定はこの crate が frontend 依存を持つかどうかを定める。
- **最初の抽出ターゲット: 未解決。task 21 で解決する。** まず 1 つの
  ターゲットバックエンド、その後一般化（アーキテクチャ 13 の採用方針）。
  最初のターゲット言語を選び、決定を `extract_backend.md` に記録する。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-doc` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### 入力

1. **crate の足場と lint 方針のガード。** [ ]
   - `mizar-session` と `mizar-artifact` に依存する workspace メンバー
     `mizar-doc` を追加し、`mizar-frontend` のガードに倣った
     `tests/lint_policy.rs` を追加する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: `mizar-artifact` task 11。仕様: アーキテクチャ 13。

2. **仕様: `artifact_reader.md`。** [ ]
   - artifact リーダーの仕様を執筆する（英語と日本語、コードなし）:
     `VerifiedArtifact` と manifest の検証つき読み込み、スキーマ互換性の
     扱い、ドキュメント入力の射影（internal 05「Artifact Documentation
     Reader」）。
   - 依存: 1。仕様: [internal 05](../../internal/ja/05.documentation_extraction.md)。

3. **artifact リーダー。** [ ]
   - schema-version チェックつきの検証読み込みを実装する。テストは
     フィクスチャ artifact を使う。
   - テスト: 有効/無効 artifact のフィクスチャ。非互換バージョンが診断
     付きで明確に失敗する。
   - 依存: 2。仕様: `artifact_reader.md`。

4. **仕様: `doc_build.md`。** [ ]
   - ドキュメントビルド計画の仕様を執筆する（英語と日本語、コード
     なし）: phase 16 リクエスト、パッケージをまたぐビルド計画、
     ドキュメント索引、決定性規則、消費 phase の規則。
   - 依存: 2。仕様: アーキテクチャ 13「Documentation Build Plan」、
     [internal 05](../../internal/ja/05.documentation_extraction.md)。

5. **ドキュメントビルド計画と索引。** [ ]
   - リーダー出力上にビルド計画とドキュメント索引を実装する。
   - テスト: 複数パッケージ入力上の計画フィクスチャ。決定的な索引順。
   - 依存: 3、4。仕様: `doc_build.md`。

### ドキュメントの strand

6. **仕様: `comments.md`。** [ ]
   - ドキュメントコメントの仕様を執筆する（英語と日本語、コードなし）:
     attachment の対象、Markdown サブセット、診断ポリシー（緩やかな
     劣化）、ドキュメントコメント供給源の決定。
   - 依存: 4。仕様: アーキテクチャ 13「Step 2」、
     [24.documentation_generation.md](../../../spec/ja/24.documentation_generation.md)。

7. **ドキュメントコメントの attachment。** [ ]
   - 決定された供給源に従い、ドキュメントコメントを文書化対象の item に
     取り付ける。
   - テスト: item 種別ごとの attachment フィクスチャ。取り付け先のない
     コメントは診断され、黙って捨てられない。
   - 依存: 5、6。仕様: `comments.md`。

8. **Markdown の構文解析。** [ ]
   - 文書化された Markdown サブセットを、緩やかな劣化と表示診断つきで
     構文解析する。
   - テスト: サブセットのフィクスチャ。不正な Markdown がビルドを中断
     せずに劣化する。
   - 依存: 7。仕様: `comments.md`。

9. **仕様: `links.md`。** [ ]
   - 相互参照の仕様を執筆する（英語と日本語、コードなし）: リンク構文、
     export 済みシンボル識別（抽出と共有）に対する解決、未解決リンクの
     ポリシー。
   - 依存: 4。仕様: アーキテクチャ 13「Step 3」「Documentation and
     Extraction Share Symbol Identity」。

10. **相互参照の解決。** [ ]
    - artifact のシンボル識別に対してドキュメントリンクを解決し、
      決定的な未解決リンク診断を出す。
    - テスト: パッケージ内・パッケージ間リンクのフィクスチャ。未解決
      リンクが位置付きで診断される。
    - 依存: 8、9。仕様: `links.md`。

11. **仕様: `math.md`。** [ ]
    - 数式レンダリングの仕様を執筆する（英語と日本語、コードなし）:
      LaTeX サブセット、式レンダリング戦略、決定性規則。
    - 依存: 4。仕様: アーキテクチャ 13「Step 3」。

12. **数式レンダリング。** [ ]
    - 緩やかな劣化を備えた決定的な LaTeX/数式レンダリングを実装する。
    - テスト: 数式のフィクスチャ。実行をまたぐ同一出力。不正な数式が
      診断付きで劣化する。
    - 依存: 8、11。仕様: `math.md`。

13. **仕様: `render.md`。** [ ]
    - レンダリングの仕様を執筆する（英語と日本語、コードなし）: 静的
      HTML サイトのレイアウト、検索索引フォーマット、決定的出力規則。
    - 依存: 4。仕様: アーキテクチャ 13「Step 4」、
      [24.documentation_generation.md](../../../spec/ja/24.documentation_generation.md)。

14. **HTML レンダリング。** [ ]
    - ドキュメント索引から静的サイトを決定的に出力する。
    - テスト: golden ファイルのフィクスチャ。実行をまたぐバイト同一の
      出力。
    - 依存: 10、12、13。仕様: `render.md`。

15. **検索索引の出力。** [ ]
    - サイトと並んで検索索引を正準順で出力する。
    - テスト: 索引のフィクスチャ。決定的な順序。項目がシンボル識別へ
      追跡できる。
    - 依存: 14。仕様: `render.md`。

### 抽出の strand

16. **仕様: `extract_select.md`。** [ ]
    - 選択の仕様を執筆する（英語と日本語、コードなし）: 検証済み実行
      可能サブセット、選択規則、未対応の実行時構文の拒否（ここに緩やかな
      劣化はない）。
    - 依存: 2。仕様: アーキテクチャ 13「Step 1（抽出）」「Extraction
      Uses Verified Executable Subset」。

17. **抽出対象 item の選択。** [ ]
    - artifact メタデータ上の選択を実装する。未対応の構文は安定した
      診断付きで拒否される。
    - テスト: 選択のフィクスチャ。未対応構文ごとの拒否ケース。
    - 依存: 3、16。仕様: `extract_select.md`。

18. **仕様: `runtime_ir.md`。** [ ]
    - `RuntimeIr` の仕様を執筆する（英語と日本語、コードなし）: ターゲット
      中立の実行時表現、ghost/証明限定の消去規則、ソース item と
      artifact ハッシュへの追跡可能性。
    - 依存: 16。仕様: アーキテクチャ 13「Step 2（抽出）」、
      [internal 05](../../internal/ja/05.documentation_extraction.md)。

19. **`RuntimeIr` の構築。** [ ]
    - 選択された item から `RuntimeIr` を構築する。
    - テスト: 構築のフィクスチャ。すべてのノードが artifact ハッシュへ
      追跡できる。
    - 依存: 17、18。仕様: `runtime_ir.md`。

20. **ghost と証明限定の消去。** [ ]
    - ghost 状態と証明限定の注釈を消去する。抽出されたコードはどちらも
      含んではならない。
    - テスト: 消去のフィクスチャ。ghost の漏出はビルドを失敗させる。
    - 依存: 19。仕様: `runtime_ir.md`（消去の節）。

21. **仕様: `extract_backend.md`。** [ ]
    - ターゲットバックエンドの仕様を執筆する（英語と日本語、コード
      なし）: バックエンドインターフェース、`RuntimeIr` とターゲット
      オプションに対する決定性、最初のターゲットの決定。
    - 依存: 18。仕様: アーキテクチャ 13「Step 3（抽出）」。

22. **最初のターゲットバックエンド。** [ ]
    - 選ばれた最初のターゲット言語へのコード出力を実装する。
    - テスト: golden ファイルのフィクスチャ。決定的な出力。出力コードが
      ソース item へ追跡できる。
    - 依存: 20、21。仕様: `extract_backend.md`。

### 公開とフォローアップ

23. **仕様: `publisher.md`、出力公開、phase 16 マニフェスト。** [ ]
    - publisher の仕様を執筆し、ドキュメント、検索索引、抽出コード、
      抽出マニフェストの原子的な公開を実装する。出力は証明の有効性に
      影響せず削除・再生成できる。
    - テスト: 中断された公開が混在状態を残さない。再生成がバイト同一で
      ある。
    - 依存: 15、22。仕様: [internal 05](../../internal/ja/05.documentation_extraction.md)
      「Output Publisher」、アーキテクチャ 13「Step 4（抽出）」。

24. **決定性スイート。** [ ]
    - 同一の artifact がバイト同一のサイト、索引、抽出コードを生むことの
      プロパティ的検証。
    - 依存: 23。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

25. **公開 enum の前方互換性ポリシー。** [ ]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用する。
    - 依存: 23。仕様: 全モジュール仕様。

26. **ソース/仕様対応監査。** [ ]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - 依存: 25。仕様: 全モジュール仕様と本 TODO。

27. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-doc/en/` の各英語正本と日本語版を比較し、内容を
      同期する。
    - 依存: 26。仕様: リポジトリのドキュメント方針。

28. **module 境界リファクタリング gate。** [ ]
    - crate を下流 consumer 向けに完了扱いにする前に、source layout を監査し、
      oversized file、混在した責務、module table と module spec 境界に沿って
      分割すべき private helper を洗い出す。review bottleneck になった実装
      ファイルは、公開 API、診断、決定的 rendering、artifact-facing schema、
      consumer-visible behavior を変えずに private module へ分割する。
    - 分割後は必要に応じて本 module table / source path を更新し、移動した
      API について source/spec 対応監査と二言語ドキュメント同期監査の範囲を
      再実行する。挙動 cleanup や API 公開を移動と混ぜない。それらは独立した
      spec task を要求する。
    - 依存: 27。仕様: 本 TODO、
      [internal 07](../../internal/ja/07.crate_module_layout.md)、全モジュール仕様。

29. **仕様 coverage closure 監査。** [ ]
    - module spec と実装 strand が揃った後、`mizar-doc` の挙動を spec
      chapters 20、21、24 と [spec_coverage_audit.md](../../spec_coverage_audit.md)
      に照らして比較する。ドキュメントコメント、相互参照、数式
      レンダリング、HTML 出力、`@latex`、`@eval` の表示境界、抽出可能
      algorithm の選択、`RuntimeIr`、ghost/証明専用の消去、ターゲット
      backend、生成出力 manifest が、module spec / test で cover されて
      いるか、または owner link 付きで明示的に deferred と分類されている
      ことを確認する。
    - 依存: 28。仕様:
      [20.algorithm_and_verification.md](../../../spec/ja/20.algorithm_and_verification.md),
      [21.source_code_annotation_and_atp.md](../../../spec/ja/21.source_code_annotation_and_atp.md),
      [24.documentation_generation.md](../../../spec/ja/24.documentation_generation.md),
      本 TODO、および全 `mizar-doc` module spec。

### 監査レビュー注記(2026-07-06)

2026 年 7 月の意味論・kernel 健全性・テンプレートエンコーディング監査
([mizar-checker semantic_spec_audit.md](../../mizar-checker/en/semantic_spec_audit.md)、
[mizar-kernel soundness_argument.md](../../mizar-kernel/en/soundness_argument.md)、
[mizar-core template_encoding_audit.md](../../mizar-core/en/template_encoding_audit.md))
を本 crate のスコープに対してレビューした。crate 所有のタスクは生じない:
本 crate は意味論・証明の authority を持たず、全監査所見は各 TODO と
トップレベル roadmap に記録された checker/kernel/vc/core/atp/proof/cache/
artifact/test のフォローアップタスクが所有する。本 crate の統合順序は不変
であり、対となるタスクが到着したら owning producer を通じて訂正済み契約を
消費すること。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-doc
cargo clippy -p mizar-doc --all-targets -- -D warnings
```

artifact スキーマに触れるタスクでは追加で実行する:

```text
cargo test -p mizar-artifact
```

テストが通ったらここでタスクにチェックを付ける。

## 備考

- phase 16 は消費 phase である: 意味論解析を再実行せず、証明の有効性の
  公開ゲートには決してならない。
- ドキュメントは表示エラーに対して緩やかに劣化する。抽出は代わりに
  未対応構文を拒否する。
- ghost 状態と証明限定の注釈は抽出された実行コードに決して現れては
  ならない。抽出コードはソース item と artifact ハッシュへ追跡可能で
  なければならない。
- 出力はいつでも削除・再生成できる。下流の何ものもそれを権威として
  扱ってはならない。
