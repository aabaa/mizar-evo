# mizar-cache TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様はまだ存在しない。各仕様は、それを引用する実装タスクより前に、
専用の仕様タスクが（英語と日本語を同じ変更で）執筆する。モジュール名は
[internal 07](../../internal/ja/07.crate_module_layout.md) の最小分割
（`cache_key`、`dependency_fingerprint`、`proof_reuse`、`cluster_db`）に、
internal 02 のレコード/blob ストアを加えたものに従う。この crate は
アーキテクチャ 11、17、18 と internal 02、06 を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| cache_key | `cache_key.md`（task 2） | `src/cache_key.rs` | [ ] |
| dependency_fingerprint | `dependency_fingerprint.md`（task 4） | `src/dependency_fingerprint.rs` | [ ] |
| cache_store | `cache_store.md`（task 7） | `src/cache_store.rs` | [ ] |
| proof_reuse | `proof_reuse.md`（task 10） | `src/proof_reuse.rs` | [ ] |
| cluster_db | `cluster_db.md`（task 12） | `src/cluster_db.rs` | [ ] |

`mizar-cache` は内部ビルドキャッシュを所有する: 正準 `CacheKey` の構築、
依存スライスと fingerprint およびその再ビルドトリガー、`cache_dir` 配下の
キャッシュレコード/blob ストア、witness ハッシュに結び付いた proof 再利用
の検証、import スコープのビューを持つ `cluster-db/` 索引。キャッシュは
最適化であり、決して権威ではない: レコードはいつでも削除でき、ソース
レベルの意味論を変えない。ヒットはクリーンビルドと同じ検証規則を満たさ
なければならず、キャッシュレコードが外部認証された証拠を kernel 検証済み
状態へ昇格させることは決してない。

依存順序: `cache_key` → `dependency_fingerprint` → `cache_store` →
`proof_reuse` / `cluster_db`。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は `mizar-session` と `mizar-artifact`（正準ハッシュ規則、
同 task 16 の interface/implementation ハッシュ入力、witness 参照）に
依存する。消費者は seam を通じて統合する: `mizar-build` スケジューラ
（同 task 18）、`mizar-ir` の cache adapter（同 task 10）。アーキテクチャ:
[11.artifact_and_incremental_build.md](../../architecture/ja/11.artifact_and_incremental_build.md)、
[18.dependency_fingerprint.md](../../architecture/ja/18.dependency_fingerprint.md)、
[17.cluster_trace_format.md](../../architecture/ja/17.cluster_trace_format.md)。
internal: [02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)、
[06](../../internal/ja/06.ir_storage_and_snapshot_handles.md)。

## 解決済みおよび保留中の決定

- **キー/ストアの分離: internal 02 により解決済み。** `CacheKeyBuilder`
  はキーと依存サマリーを生産するだけである。再利用を決めるのは互換性
  チェックと proof witness 検証であり、どちらも信頼を与えない。
- **スライス粒度: 未解決。task 5 で解決する。** アーキテクチャ 18 は
  theorem、definition、cluster、notation、mode、attribute のスライスを
  挙げる。実際に計算する初期スライス粒度（再ビルドトリガーが保守的で
  ある限り粗くてよい）を決め、`dependency_fingerprint.md` に記録する。
- **レコードエンコーディング: 未解決。task 7 で解決する。** キャッシュ
  レコードのバイナリエンコーディングと `cache_schema_version` の進化
  規則を決める。キャッシュレコードは内部用であり、artifact と違って
  生 IR エンコーディングを含んでよい。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-cache` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### キーと fingerprint

1. **crate の足場と lint 方針のガード。** [ ]
   - `mizar-session` と `mizar-artifact` に依存する workspace メンバー
     `mizar-cache` を追加し、`mizar-frontend` のガードに倣った
     `tests/lint_policy.rs` を追加する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: `mizar-artifact` task 3。仕様: internal 02。

2. **仕様: `cache_key.md`。** [ ]
   - キャッシュキーの仕様を執筆する（英語と日本語、コードなし）:
     `CacheKey` のフィールド（phase、work unit、ソース識別、入力/依存
     ハッシュ、依存スライス、設定ハッシュ、schema version、ポリシー
     fingerprint）、ベクタの正準順序、ドメイン分離された最終ハッシュ。
   - 依存: 1。仕様: [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)
     「Cache Key」。

3. **キャッシュキービルダー。** [ ]
   - `CacheKeyBuilder` を、識別・ハッシュ・schema version・ポリシーから
     の純粋な射影として実装する — 可変なタスク状態は読まない。
   - テスト: キーの決定性。フィールド順への非依存（正準ソート）。
     どの入力の変化もキーを変える。
   - 依存: 2。仕様: `cache_key.md`。

4. **仕様: `dependency_fingerprint.md`。** [ ]
   - fingerprint の仕様を執筆する（英語と日本語、コードなし）:
     fingerprint の対象、依存スライス、安定入力（と除外される非決定的
     入力）、再ビルドトリガー、アーキテクチャ 18 の API 互換性 diff。
     スライス粒度の決定を含む。
   - 依存: 2。仕様:
     [18.dependency_fingerprint.md](../../architecture/ja/18.dependency_fingerprint.md)。

5. **依存スライスと fingerprint の計算。** [ ]
   - interface/implementation ハッシュ入力（`mizar-artifact` task 16）と
     VC ごとの依存スライス（`mizar-vc` task 14）の上で、決定した粒度の
     fingerprint を計算する。
   - テスト: 非インターフェース編集に対する fingerprint の安定性。
     スライスの変化が正確に依存先の fingerprint だけを変える。
   - 依存: 3、4、`mizar-artifact` task 16。仕様:
     `dependency_fingerprint.md`。

6. **再ビルドトリガーの評価。** [ ]
   - トリガー規則を実装する: どの fingerprint 変化がどのキャッシュ済み
     phase を無効化するか。スライスが粗い場合は保守的に。
   - テスト: 変更種別（ソース、import、registration、ポリシー、ツール
     チェーン）ごとのトリガーフィクスチャ。保守モードでの偽陰性なし。
   - 依存: 5。仕様: `dependency_fingerprint.md`（トリガーの節）。

### ストア

7. **仕様: `cache_store.md`。** [ ]
   - ストアの仕様を執筆する（英語と日本語、コードなし）:
     `.mizar-cache/` のレイアウト（phase とキーごとのレコード、
     content-addressed blob）、`CacheRecordHeader`、互換性チェック、
     レコードエンコーディングの決定、削除可能性の保証。
   - 依存: 2。仕様: [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)
     「Cache Store」「Cache Record」。

8. **レコードストア。** [ ]
   - ヘッダーの互換性チェック（cache schema version、ツールチェーン、
     出力ハッシュ）を備えたレコードの書き込み/読み込み/検証を実装する。
   - テスト: ラウンドトリップ。非互換ヘッダーはエラーではなくミスに
     なる。破損レコードの検出。
   - 依存: 3、7。仕様: `cache_store.md`。

9. **blob ストア。** [ ]
   - ハッシュ検証つき読み込みと安全な並行書き込みを備えた、大きな出力の
     content-addressed blob ストレージを実装する。
   - テスト: blob のラウンドトリップ。ハッシュ不一致の検出。並行 writer
     の収束。
   - 依存: 8。仕様: `cache_store.md`。

### proof 再利用と cluster db

10. **仕様: `proof_reuse.md`。** [ ]
    - proof 再利用の仕様を執筆する（英語と日本語、コードなし）: 再利用は
      witness ハッシュ、依存 artifact ハッシュ、ポリシー fingerprint、
      schema version の一致を要する。決定的な built-in discharge
      レコード。レコードが証拠クラスを決して昇格させない規則。
    - 依存: 7。仕様: [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)、
      [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)
      「Proof Witness and Artifact Flow」。

11. **proof 再利用の検証。** [ ]
    - `ProofReuseEvidence` 上の再利用検証を実装する。失敗は再計算へ
      退化し、受理へは決して退化しない。
    - テスト: 一致する証拠の再利用。各構成要素（witness ハッシュ、
      ポリシー、schema）の不一致が再利用をブロックする。外部証拠が
      キャッシュ経由で kernel 検証済みになることは決してない。
    - 依存: 8、10。仕様: `proof_reuse.md`。

12. **仕様: `cluster_db.md`。** [ ]
    - cluster-db の仕様を執筆する（英語と日本語、コードなし）: 索引
      レイアウト、寄与ごとの origin メタデータ、import スコープの
      ビュー、可視 origin 変化による無効化、未受理の registration 証明が
      importer 可視の索引に決して入らない規則。
    - 依存: 7。仕様: [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)
      「Cluster and Registration Cache Update」、
      [17.cluster_trace_format.md](../../architecture/ja/17.cluster_trace_format.md)。

13. **cluster-db の書き込みと origin 追跡。** [ ]
    - origin メタデータ付きの寄与書き込み、古い origin の除去、origin
      ごとの索引再構築を実装する。
    - テスト: 改名/削除が古い origin を掃除する。再構築は影響を受けた
      origin のみに触れる。未受理の寄与の拒否。
    - 依存: 12、`mizar-checker` task 16。仕様: `cluster_db.md`。

14. **import スコープのビューと無効化。** [ ]
    - 可視 origin 集合でキー付けされた import スコープのビューと、
      ビュー集合変化による無効化を実装する。
    - テスト: 無関係な変更をまたぐビューの再利用。可視 origin の変化が
      正確に影響を受けたビューだけを無効化する。
    - 依存: 13。仕様: `cluster_db.md`。

### 統合とフォローアップ

15. **スケジューラと IR adapter の統合。** [ ]
    - ストアを `mizar-build` のキャッシュ seam（同 task 18）と
      `mizar-ir` の cache adapter（同 task 10）に接続する。キャッシュ
      ヒットは外部から見て同一の結果で作業をスキップする。
    - テスト: ヒット/ミスのエンドツーエンドフィクスチャ。ヒット結果が
      クリーンビルドの結果とバイト同一。
    - 依存: 8、`mizar-build` task 18、`mizar-ir` task 10。仕様:
      [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)
      「Cache Lookup Before Task Execution」。

16. **決定性と削除可能性のスイート。** [ ]
    - プロパティ的検証: 同一入力が同一のキーとレコードを生む。任意の
      キャッシュ部分集合を削除してもビルド結果は変わらず、変わるのは
      ビルド時間だけである。
    - 依存: 15。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

17. **公開 enum の前方互換性ポリシー。** [ ]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用する。
    - 依存: 14。仕様: 全モジュール仕様。

18. **ソース/仕様対応監査。** [ ]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - 依存: 17。仕様: 全モジュール仕様と本 TODO。

19. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-cache/en/` の各英語正本と日本語版を比較し、内容を
      同期する。
    - 依存: 18。仕様: リポジトリのドキュメント方針。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-cache
cargo clippy -p mizar-cache --all-targets -- -D warnings
```

統合タスクでは追加で実行する:

```text
cargo test -p mizar-artifact
cargo test -p mizar-build
cargo test -p mizar-ir
```

テストが通ったらここでタスクにチェックを付ける。

## 備考

- キャッシュは最適化であり、決して権威ではない: ヒットはクリーンビルドと
  同じ検証規則を満たし、レコードの削除は意味論を決して変えない。
- キャッシュレコードが外部認証された証拠を kernel 検証済み状態へ昇格
  させることは決してない。proof 再利用は受理済み witness ハッシュに
  結び付く。
- キーは純粋な射影である。信頼の決定はキー構築ではなく、互換性チェックと
  proof 再利用検証にある。
- キャッシュレコードは内部用であり、生 IR エンコーディングを含んでよい。
  公開 artifact は含んではならない（その境界は
  `mizar-artifact`/`mizar-ir` のもの）。
