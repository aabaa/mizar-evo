# mizar-cache TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様は専用の仕様タスクが（英語と日本語を同じ変更で）、それを
引用する実装タスクより前に執筆する。モジュール名は
[internal 07](../../internal/ja/07.crate_module_layout.md) の最小分割
（`cache_key`、`dependency_fingerprint`、`proof_reuse`、`cluster_db`）に、
internal 02 のレコード/blob ストアを加えたものに従う。この crate は
アーキテクチャ 11、17、18 と internal 02、06 を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| cache_key | `cache_key.md`（task 2） | `src/cache_key.rs` | [x] |
| dependency_fingerprint | `dependency_fingerprint.md`（task 4） | `src/dependency_fingerprint.rs` | [x] |
| cache_store | `cache_store.md`（task 7） | `src/cache_store.rs` | [x] |
| proof_reuse | `proof_reuse.md`（task 10） | `src/proof_reuse.rs` | [x] |
| cluster_db | `cluster_db.md`（task 12） | `src/cluster_db.rs` | [x] |

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

この crate は `mizar-session`、`mizar-artifact`、`mizar-vc`、そして task 11 以降は
status projection が export する proof-reuse metadata を消費するために `mizar-proof` に
依存する。`mizar-vc` は公開された VC ごとの dependency-slice fingerprint を供給する。
`mizar-artifact` は正準ハッシュ規則、同 task 16 の interface/implementation ハッシュ入力、
witness 参照を提供する。消費者は seam を通じて統合する: `mizar-build` スケジューラ
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
- **スライス粒度: task 4 で解決済み。** `dependency_fingerprint.md` は
  theorem、definition、cluster、notation、mode、attribute を semantic target
  taxonomy として維持する。一方で、より細かい producer slice が landing するまで、
  task 5 は conservative な published-summary と per-VC slice 粒度から開始してよい。
- **レコードエンコーディング: task 7 で解決済み。** `cache_store.md` は
  canonical UTF-8 JSON header と inline または blob-backed payload bytes を持つ
  binary record envelope を使う。cache record は内部用だが、`mizar-ir` adapter と
  raw-IR integration は external dependency gap のままであり、owner task が landing
  する前に placeholder IR storage API を追加しない。
- **cluster-db visibility: task 12 で解決済み。** `cluster_db.md` は
  `origins/` を invalidation の source of truth とし、spec 23.7.7 の aggregate
  index を accepted origin record だけから導出し、import スコープ view を削除可能な
  cache data として materialize する。unaccepted、recovered、externally attested
  material は cache record 経由で importer-visible にならない。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-cache` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### キーと fingerprint

1. **crate の足場と lint 方針のガード。** [x]
   - `mizar-session` と `mizar-artifact` に依存する workspace メンバー
     `mizar-cache` を追加し、`mizar-frontend` のガードに倣った
     `tests/lint_policy.rs` を追加する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: `mizar-artifact` task 3。仕様: internal 02。

2. **仕様: `cache_key.md`。** [x]
   - キャッシュキーの仕様を執筆する（英語と日本語、コードなし）:
     `CacheKey` のフィールド（phase、work unit、ソース識別、入力/依存
     ハッシュ、依存スライス、設定ハッシュ、schema version、ポリシー
     fingerprint）、ベクタの正準順序、ドメイン分離された最終ハッシュ。
     verifier policy、toolchain/schema 互換性、canonical VC fingerprint、
     local-context fingerprint、dependency-slice fingerprint に関する
     architecture-22 の検証入力を含める。
   - 依存: 1。仕様: [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)
     「Cache Key」。

3. **キャッシュキービルダー。** [x]
   - `CacheKeyBuilder` を、識別・ハッシュ・schema version・ポリシーから
     の純粋な射影として実装する — 可変なタスク状態は読まない。
   - テスト: キーの決定性。フィールド順への非依存（正準ソート）。
     どの入力の変化もキーを変える。
   - 依存: 2。仕様: `cache_key.md`。

4. **仕様: `dependency_fingerprint.md`。** [x]
   - fingerprint の仕様を執筆する（英語と日本語、コードなし）:
     fingerprint の対象、依存スライス、安定入力（と除外される非決定的
     入力）、再ビルドトリガー、アーキテクチャ 18 の API 互換性 diff。
     スライス粒度の決定を含む。完全な dependency-footprint 要件と、
     footprint を信頼できない場合に使う保守的な `uncacheable` marker を
     仕様化する。
   - 依存: 2。仕様:
     [18.dependency_fingerprint.md](../../architecture/ja/18.dependency_fingerprint.md)。

5. **依存スライスと fingerprint の計算。** [x]
   - interface/implementation ハッシュ入力（`mizar-artifact` task 16）と
     VC ごとの依存スライス（`mizar-vc` task 14）の上で、決定した粒度の
     fingerprint を計算する。
   - テスト: 非インターフェース編集に対する fingerprint の安定性。
     スライスの変化が正確に依存先の fingerprint だけを変えること。
     deterministic output、canonical ordering と duplicate-conflict rejection、
     comment/formatting/diagnostic/runtime/order/temporary/local path/
     snapshot-local id の安定除外、missing/unknown/uncacheable miss behavior、
     proof-reuse validation data が untrusted のままであることも cover する。
   - 依存: 3、4、`mizar-artifact` task 16、`mizar-vc` task 14。仕様:
     `dependency_fingerprint.md`。

6. **再ビルドトリガーの評価。** [x]
   - トリガー規則を実装する: どの fingerprint 変化がどのキャッシュ済み
     phase を無効化するか。スライスが粗い場合は保守的に。
   - テスト: 変更種別（ソース、import、registration、cluster/reduction、
     policy、toolchain、schema、proof-body、diagnostic-only、incomplete footprint、
     unknown schema/toolchain、uncacheable marker、missing proof-reuse validation）ごとの
     トリガーフィクスチャ。保守モードでの偽陰性なし。
   - 依存: 5。仕様: `dependency_fingerprint.md`（トリガーの節）。

### ストア

7. **仕様: `cache_store.md`。** [x]
   - ストアの仕様を執筆する（英語と日本語、コードなし）:
     `.mizar-cache/` のレイアウト（phase とキーごとのレコード、
     content-addressed blob）、`CacheRecordHeader`、互換性チェック、
     レコードエンコーディングの決定、`uncacheable` record の扱い、
     incomplete-footprint 時の miss、unknown-compatibility 時の miss、
     削除可能性の保証。
   - 依存: 2。仕様: [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)
     「Cache Store」「Cache Record」。

8. **レコードストア。** [x]
   - ヘッダーの互換性チェック（cache schema version、ツールチェーン、
     出力ハッシュ）を備えたレコードの書き込み/読み込み/検証を実装する。
   - テスト: ラウンドトリップ。非互換ヘッダーはエラーではなくミスに
     なる。破損レコードの検出。
   - 依存: 3、7。仕様: `cache_store.md`。

9. **blob ストア。** [x]
   - ハッシュ検証つき読み込みと安全な並行書き込みを備えた、大きな出力の
     content-addressed blob ストレージを実装する。
   - テスト: blob のラウンドトリップ。ハッシュ不一致の検出。並行 writer
     の収束。
   - 依存: 8。仕様: `cache_store.md`。

### proof 再利用と cluster db

10. **仕様: `proof_reuse.md`。** [x]
    - proof 再利用の仕様を執筆する（英語と日本語、コードなし）: 再利用は
      `ObligationAnchor`、canonical VC fingerprint、local-context fingerprint、
      dependency-slice fingerprint、選択された proof witness hash または
      deterministic discharge hash、依存 artifact ハッシュ、ポリシー
      fingerprint、schema version の一致を要する。決定的な built-in
      discharge レコード。レコードが証拠クラスを決して昇格させない規則。
    - 依存: 7。仕様: [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)、
      [internal 04](../../internal/ja/04.atp_portfolio_and_kernel_check_integration.md)
      「Proof Witness and Artifact Flow」。

11. **proof 再利用の検証。** [x]
    - `mizar-proof` の `StatusReuseMetadata` から導いた proof-reuse metadata snapshot
      上の再利用検証を実装する。失敗は再計算へ退化し、受理へは決して退化しない。
    - テスト: 一致する `KernelVerified` と `DischargedBuiltin` evidence の再利用。
      必要な各構成要素（`ObligationAnchor`、obligation fingerprint、canonical VC
      fingerprint、local-context fingerprint、dependency-slice fingerprint、選択された
      witness hash、deterministic discharge hash、selected proof class、
      proof-evidence identity、selected evidence hash、selected candidate provenance hash、
      selection reason、tie-break key hash、export される場合の trusted used-axioms reference hash、
      proof-reuse validation hash、policy、schema、dependency artifact）の不一致が再利用を
      ブロックすること（欠落も含む）。incomplete footprint、
      unsupported schema、unknown toolchain、uncacheable marker が miss になること。
      non-trusted class と externally attested evidence が cache 経由で kernel-verified
      または trusted `used_axioms` にならず、trusted `used_axioms` を合成する record が
      reject されること、trusted used-axioms reference hash を合成する record も reject されること。
      upstream proof-reuse completeness が尊重されること。local validation が record
      arrival/write order と cache hit/miss timing に依存しないこと。lint/source-surface guard により
      scheduler、`mizar-ir`、artifact publication-token、witness publication shortcut を追加しないことを
      確認すること。cross-crate producer wiring と full clean/incremental equivalence は task 20 の
      gate に残す。
    - 依存: 8、10。仕様: `proof_reuse.md`。

12. **仕様: `cluster_db.md`。** [x]
    - cluster-db の仕様を執筆する（英語と日本語、コードなし）: 索引
      レイアウト、寄与ごとの origin メタデータ、import スコープの
      ビュー、可視 origin 変化による無効化、未受理の registration 証明が
      importer 可視の索引に決して入らない規則。
    - task 12 で完了: `cluster_db.md` は logical `.mizar-cache/cluster-db/`
      layout、origin metadata、accepted-only visibility rule、aggregate index
      ordering、import-scoped view key、visible-origin invalidation、ResolutionTrace
      boundary、task 13/14 planned test を、source implementation なしで定義する。
    - 依存: 7。仕様: [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)
      「Cluster and Registration Cache Update」、
      [17.cluster_trace_format.md](../../architecture/ja/17.cluster_trace_format.md)。

13. **cluster-db の書き込みと origin 追跡。** [x]
    - origin メタデータ付きの寄与書き込み、古い origin の除去、origin
      ごとの索引再構築を実装する。
    - テスト: accepted contribution insertion。accepted だが private/local-only、
      rejected、pending、recovered、uncacheable、externally attested な contribution が
      visible index から reject されること。incomplete origin metadata、incomplete
      footprint、missing dependency-interface hash、missing trace replay hash、proof-backed
      accepted contribution の missing proof witness/discharge identity が reject されること。
      unknown schema/toolchain compatibility が reject されること。
      改名/削除が古い origin を掃除する。再構築は影響を受けた origin のみに触れる。
      deterministic same-index ordering。conflict する重複 origin と cross-module
      origin-key collision の rejection。
    - 実装: task 13 は producer-owned `ClusterContributionRecord` snapshot 上の
      in-memory cache-side data layer を追加する。raw source の parse、proof/checker
      authority の主張、scheduler hook、`mizar-ir` API、import-scoped view materialization
      は行わない。
    - task 13 で完了: `src/cluster_db.rs` は `ClusterDbIndex::apply_module_update`、
      fail-closed origin validation、stale-origin cleanup、aggregate index snapshot、
      rebuild report、downstream/proof-authority stub に対する lint guard を公開する。
    - 依存: 12、`mizar-checker` task 16。仕様: `cluster_db.md`。

14. **import スコープのビューと無効化。** [x]
    - 可視 origin 集合でキー付けされた import スコープのビューと、
      ビュー集合変化による無効化を実装する。
    - テスト: 無関係な変更をまたぐビューの再利用。可視 origin の変化が
      正確に影響を受けたビューだけを無効化すること。欠落 origin、
      unsupported schema、missing producer schema metadata、不明な
      policy/schema/toolchain/traversal compatibility、mismatched verifier policy、
      mismatched producer compatibility metadata の request validation が miss になること。
      view ordering が record arrival/write order と visible-origin request order に
      依存しないこと。hidden cluster/reduction step を推論しないこと。
    - 実装: task 14 は in-memory の `ImportScopedViewRequest`、
      `ImportScopedViewKey`、`ImportScopedView`、`ClusterDbViewMiss`、
      `ClusterDbIndex::import_scoped_view` を追加する。view は request metadata を
      canonicalize / validate し、visible origin を accepted origin record と照合し、
      aggregate row を visible origin set で filter する。view hit は cache
      optimization のままである。
    - task 14 で deferred / scope 外: durable な `views/` file、scheduler
      integration、`mizar-ir` adapter integration、artifact publication-token
      linkage、proof status projection、trace construction は scope 外に残し、
      stub しない。
    - 依存: 13。仕様: `cluster_db.md`。

### 統合とフォローアップ

15. **スケジューラと IR adapter の integration readiness。** [x]
    - ストアを `mizar-build` のキャッシュ seam（同 task 18）と
      `mizar-ir` の cache adapter（同 task 10）に接続する。キャッシュ
      ヒットは外部から見て同一の結果で作業をスキップする。
    - テスト: ヒット/ミスのエンドツーエンドフィクスチャ。ヒット結果が
      クリーンビルドの結果とバイト同一。task 15 自体は、それらの seam が
      ready でないため documentation/review のみである。
    - 依存: 8、`mizar-build` task 18、`mizar-ir` task 10。仕様:
      [internal 02](../../internal/ja/02.artifact_store_cache_key_and_manifest.md)
      「Cache Lookup Before Task Execution」。
    - task 15 で完了: [integration_readiness.md](./integration_readiness.md) は、
      まだ open の `mizar-build` cache-aware scheduler seam、存在しない
      `mizar-ir` cache adapter、artifact committed-publication token linkage を
      `external_dependency_gap` として記録する。placeholder scheduler、`mizar-ir`、
      artifact-publication-token API は追加しない。上記の実装と end-to-end test は
      owner が landing するまで deferred のままにする。

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

20. **増分検証の fail-closed cache 契約。** [ ]
    - architecture 22 の cache contract を `cache_key`、
      `dependency_fingerprint`、`cache_store`、`proof_reuse` 全体で実装・
      テストする。再利用される output は完全な dependency slice / footprint
      を持たなければならない。`uncacheable` marker は常に miss を強制する。
      不明な schema / toolchain 互換性は miss を強制する。proof reuse は
      policy、`ObligationAnchor`、canonical VC fingerprint、local-context
      fingerprint、dependency-slice fingerprint、proof witness または
      deterministic discharge hash を検証する。
    - テスト: 欠落または不一致の各 field が独立に reuse を止める。任意の
      cache 部分集合を削除しても変わるのは性能だけである。cache hit/miss
      timing が diagnostics、artifact order、proof acceptance を変えない。
      reused record が外部認証された証拠を昇格させない。
    - 依存: 5、6、11、15、16、`mizar-vc` task 20、`mizar-proof` task 17。
      仕様:
      [22.incremental_verification_contract.md](../../architecture/ja/22.incremental_verification_contract.md),
      [11.artifact_and_incremental_build.md](../../architecture/ja/11.artifact_and_incremental_build.md),
      [18.dependency_fingerprint.md](../../architecture/ja/18.dependency_fingerprint.md)。

21. **architecture-22 フォローアップ監査。** [ ]
    - task 20 の cache-key、dependency-footprint、store、proof-reuse 契約に
      ついて、ソース/仕様対応監査と二言語ドキュメント同期監査を再実行する。
      残る fail-closed または trust-boundary gap をフォローアップタスクとして
      記録する。
    - 依存: 20。仕様: 全モジュール仕様、本 TODO、リポジトリの
      ドキュメント方針。

22. **module 境界リファクタリング gate。** [ ]
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

23. **crate exit report と quality review。** [ ]
    - task 1-22 が完了、または external dependency gap として明示的に deferred
      された後で、paired crate exit report を作成する。task commit、hard-gate
      status、review result、verification、deferred item、有効な 90/100 以上の
      read-only quality score を記録する。
    - 依存: 22。仕様:
      [autonomous_crate_development.md](../../autonomous_crate_development.md)。
    - Status: closeout のみ。この task には review が要求する修正以外の
      new feature implementation を混ぜない。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-cache
cargo clippy -p mizar-cache --all-targets -- -D warnings
cargo fmt --check
```

統合タスクでは追加で実行する:

```text
cargo test -p mizar-artifact
cargo test -p mizar-build
cargo test -p mizar-ir
```

architecture-22 の proof-reuse 契約では、reuse identity と metadata の
producer も追加で実行する:

```text
cargo test -p mizar-vc
cargo test -p mizar-proof
```

Rust source 変更では、未実行理由を明示しない限り、finalize 前に
AGENTS.md の broad verification も適用する:

```text
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

task に応じた verification と必要な broad verification が通ったら、ここで
task にチェックを付ける。

## 備考

- キャッシュは最適化であり、決して権威ではない: ヒットはクリーンビルドと
  同じ検証規則を満たし、レコードの削除は意味論を決して変えない。
- キャッシュレコードが外部認証された証拠を kernel 検証済み状態へ昇格
  させることは決してない。proof 再利用は受理済み witness ハッシュに
  結び付く。
- キーは純粋な射影である。信頼の決定はキー構築ではなく、互換性チェックと
  proof 再利用検証にある。
- キャッシュレコードは内部用であり、opaque payload bytes を保存してよい。
  raw-IR encoding と adapter integration は `mizar-ir` の external dependency gap の
  ままであり、この crate は placeholder IR storage API を invent してはならない。
