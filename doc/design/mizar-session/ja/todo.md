# mizar-session TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## ステータス凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

| モジュール | 仕様 | ソース | ステータス |
|---|---|---|---|
| ids | [ids.md](./ids.md) | `src/ids.rs` | [x] |
| source_map | [source_map.md](./source_map.md) | `src/source_map.rs` | [x] |
| snapshot | [snapshot.md](./snapshot.md) | `src/snapshot.rs` | [x] |
| source | [source.md](./source.md) | `src/source.rs` | [x] |
| retention | [retention.md](./retention.md) | `src/retention.rs` | [x] |

本クレートは識別子・座標を提供する最下層（leaf）なので、内部依存に沿って
ボトムアップで構築する: `ids` → `source_map` → `snapshot` → `source` → `retention`。
`SourceId` は他の全モジュールが参照する共有プリミティブである。

## 順序付きタスク一覧

各タスクは単独で実装・テスト・コミットできる粒度になっている。依存順に並んでおり、後続タスクは
先行タスクがマージ済みであることを前提とする。各タスクは `cargo test -p mizar-session`
を成功状態に保つこと（[推奨検証](#推奨検証)を参照）。

### モジュール: ids (`src/ids.rs`)

1. **不透明な id プリミティブと id newtype。** [x]
   - `lib.rs` に `pub mod ids;` を追加し、公開 id 型を再エクスポートする。
   - 内部の `OpaqueId` プリミティブと、内容由来 id が使う `Hash` newtype を定義する。
   - `BuildSessionId`, `BuildRequestId`, `BuildSnapshotId(Hash)`, `SourceId`, `SourceMapId`, `SnapshotLeaseId` を、適切に `Debug`/`Clone`/`Copy`/`Eq`/`Hash` 付きで定義する。
   - `IdError` 列挙（不正形式 / ドメイン不一致 / 未知レジストリ / オーバーフロー / 永続化不可のシリアライズ）を追加する。
   - テスト: 等価性、copy/clone、id が不透明（意味的な順序を公開しない）こと。
   - 仕様: [ids.md](./ids.md)「Public API」「Identifier Scope」。

2. **内容由来 id のエンコード。** [x]
   - `BuildSnapshotId` の正準的な小文字 16 進シリアライズ/デシリアライズをドメインセパレータ付きで実装し、不正形式/ドメイン不一致は `IdError` で拒否する。
   - 内部ハッシュヘルパー（ドメインセパレータ + スキーマ/ツールチェイン識別子のフック + 非順序コレクションのソート要件）を用意する。snapshot モジュールがこれに入力する（実際のスナップショットのハッシュ化はタスク 10）。
   - アロケータ発行の id を公開スキーマへシリアライズすることを拒否する。
   - テスト: 16 進エンコード/デコードのラウンドトリップ、ドメインセパレータ変更で id が変わる、公開スキーマのシリアライズが永続化不可 id を拒否する。
   - 仕様: [ids.md](./ids.md)「Serialization」「Content-Derived Id Construction」。

3. **セッション id アロケータ。** [x]
   - `SessionIdAllocator` トレイトと、session/request/source/source-map/lease の各 id 向けの具体的なインメモリアロケータ（単調増加カウンタまたはアリーナインデックス）を定義する。
   - テスト: 1 つのレジストリ内で id が一意であること、アロケータのオーバーフローが `IdError` になること。
   - 仕様: [ids.md](./ids.md)「Allocator-Issued Id Construction」。

### モジュール: source_map (`src/source_map.rs`)

4. **`SourceRange` と `LineMap` へ `SourceId` を統合。** [x]
   - `SourceRange` に `source_id: SourceId` を、`LineMap` に `source_id` + `text_hash: Hash` を追加する。
   - バイトオフセットの意味は維持し、`with_source(source_id, text)` コンストラクタを追加して既存の `new` 経路を維持/調整する。
   - 変換前に、範囲/オフセットが期待するソースに属することを検証する。
   - `SourceMapError` を仕様の全変種へ向けて拡張する（各機能の実装時に変種を追加）: unknown source id、range outside source text、UTF-8 境界でないオフセット、行/列オーバーフロー（タスク 5）、前処理済みテキスト外の字句範囲（タスク 7）、欠落したローディングマップセグメント（タスク 6）、欠落した前処理セグメント（タスク 7）、起点の無い生成スパン（タスク 8）。
   - クレート横断の影響: `mizar-lsp::range_mapper` の呼び出し箇所とテストを `SourceId` を渡すよう更新する。
   - テスト: 既存の行/列テストを更新、クロスソースの範囲は拒否される、unknown source id は拒否される。
   - 依存: 1。仕様: [source_map.md](./source_map.md)「Line Map」「Source Range」。
   - 注: これは字句解析器に対して加法的（字句解析器は自前の `SourceSpan` を維持、橋渡しは `mizar-lsp` に残る）。スパン橋渡しの方針は確認するが、字句解析器の変更を待たずに進めてよい。

5. **行/列のオーバーフロー方針。** [x]
   - `LineColumn` の値は `u32` のままにし、`SourceMapError::LineColumnOverflow` を追加する。
   - `usize` からの飽和/巻き戻し/縮小ではなく、オーバーフローを報告する。
   - テスト: 表現不能な行/列はオーバーフローを報告、通常のマルチバイト変換は引き続き 1 始まりの Unicode スカラー列を返す。
   - 依存: 4。仕様: [source_map.md](./source_map.md)「Public API」（`LineColumn` の注記）。

6. **ローディングマップ。** [x]
   - `TextRange`（読み込み/字句テキストへのバイト範囲。source-id でスコープされる `SourceRange` とは区別する）を導入する。
   - `LoadingMap`, `LoadingOrigin`, `LoadingMapSegment`（`Original` / `RemovedLeadingBom` / `NormalizedNewline`）を追加する。
   - 読み込みテキスト → 元テキストの対応付けを実装する。オフセットを変える変換が無い場合は恒等とする。
   - テスト: 先頭 BOM で読み込み `0` → 元バイト `3`、CRLF→LF セグメント、正規化セグメントをまたぐ複合対応付け。
   - 依存: 4。仕様: [source_map.md](./source_map.md)「Loading Map」「Loaded-to-Original Mapping」。

7. **前処理マップとアンカー。** [x]
   - `PreprocessMap`, `PreprocessSegment`（`Original` / `RemovedComment` / `SyntheticWhitespace`）と `SourceAnchor` を追加する。
   - 字句 → ソースの対応付けを実装し、長さ 0 の境界では隣接する複合アンカーを返す。
   - テスト: 除去コメントが保持範囲に対応付く、除去コメントをまたぐ字句範囲が複合対応付けになる、合成空白が主たるユーザー範囲にならない。
   - 依存: 6。仕様: [source_map.md](./source_map.md)「Preprocess Map」「Lexical-to-Source Mapping」。

8. **`SourceMapService` と生成スパン。** [x]
   - `MappedSourceRange`（主たる `SourceRange` + 二次アンカー群 + loaded-to-original の `original_input` バイト範囲）を、読み込み/字句の対応付けの複合返り値型として定義する。
   - `SourceMapService` トレイト（`line_column`, `original_range_for_loaded`, `source_range_for_lexical`, `attach_generated_span`, `validate_range`）と、保持されたマップ上の具体実装を定義する。
   - 理由必須の生成スパン起点（`GeneratedSpanOrigin`）を追加する。
   - テスト: 代表入力に対する各トレイトメソッド、複合対応付けは主アンカー + 二次アンカーを返す、起点の無い生成スパンは拒否される。
   - 依存: 5, 7。仕様: [source_map.md](./source_map.md)「Public API」「Generated Spans」。

### モジュール: snapshot (`src/snapshot.rs`)

9. **ソースバージョンのレコード。** [x]
   - `pub mod snapshot;` を追加する。`SourceVersion` と `SourceOrigin`（`Disk` / `OpenBuffer{version}` / `Generated{generator}`）を定義する。
   - `SnapshotError` を仕様の変種付きで定義する（後続タスクが必要とする時に追加）: 不正/非正規化のソースパス、重複するモジュールパス、欠落した依存アーティファクト、未対応の lockfile/ツールチェインメタデータ、古いオープンバッファバージョン、未知のスナップショット id、リース解放の不一致。
   - 正準ソートキー（package id、module path、normalized path、source hash）を用意する。
   - テスト: 挿入順に依らず正準キーで決定的に順序付く。
   - 依存: 1, 4。仕様: [snapshot.md](./snapshot.md)「Source Version」。

10. **ビルドスナップショットの同一性。** [x]
    - `BuildSnapshot` と `SnapshotInput` を定義し、正準入力（ソート済みのソース/依存サマリー、lockfile ハッシュ、ツールチェイン、検証器構成ハッシュ）から内容由来の `BuildSnapshotId` を計算する。セッションローカル id/タイムスタンプは除外する。
    - タスク 9 からの申し送り: 正準キー（package id、module path、normalized path、source hash）で等価に比較されるソースバージョンのエントリが、スナップショットハッシュを挿入順依存にしてはならない。そのような重複がハッシュ化に到達し得る場合は決定的にエンコードする。望ましくは、タスク 11 の検証で重複するソースバージョンの同一性をハッシュ化前に拒否する。
    - テスト: 同一の正準入力 ⇒ 同一 id、ソース/依存/構成の変更 ⇒ 異なる id、セッションローカル id はハッシュに含まれない。
    - 依存: 2, 9。仕様: [snapshot.md](./snapshot.md)「Snapshot Identity」。

11. **スナップショットレジストリ、作成、鮮度。** [x]
    - `SnapshotRegistry` に `create_snapshot`、`get`、`is_current_for_request` を定義する。
    - タスク 11 の境界に従う: `create_snapshot` はソース読み込み層から読み込み済みの `SourceVersion` レコードを受け取り、作成入力を検証し、id をハッシュし、スナップショットを挿入して、実行中ビルドの `SnapshotLease` とともに返す。仕様のシグネチャは `Result<(BuildSnapshot, SnapshotLease), SnapshotError>` に更新する。（これは「作成時にリースを返すか / 呼び出し側が acquire するか」の論点を仕様寄りに解消する: レジストリが実行中ビルドのリースを返す。）
    - ここで、最小の `SnapshotLease` ハンドルと、それが持つ依存無しの `RetentionReason` 列挙を導入する（`retention` モジュールと共有。実行中ビルドのリースは `RetentionReason::ActiveBuild` を使う）。完全なリース計上はタスク 12 で行い、retention（タスク 17）はこの `RetentionReason` を再定義せず再利用する。
    - タスク 9/10 からの申し送り: 正準キーが等しい重複ソースバージョンの同一性はスナップショットハッシュ化前に拒否し、作成処理が挿入順に敏感な重複レコードを受理しないようにする。
    - テスト: 作成したスナップショットが取得可能で実行中ビルドのリースを返す、古い id は鮮度チェックで拒否、古いスナップショットは現行として報告されない、重複するモジュールパスは拒否、パス正規化済みの重複ソース同一性はソースバージョン正準キーで拒否、欠落した依存アーティファクト/コンテンツフィンガープリントは拒否、未対応の lockfile/ツールチェインメタデータは拒否、構造的に不正なオープンバッファバージョンは拒否。期待値と実際値を比べる真のオープンバッファ失効は、リクエストメタデータを持つソース読み込みタスクで検証する。
    - 依存: 3, 10。仕様: [snapshot.md](./snapshot.md)「Snapshot Creation」「Freshness Check」「Error Handling」。

12. **スナップショットリースの計上。** [x]
    - レジストリの `SnapshotLease` を `acquire_lease`/`release_lease` で完成させ、`RetentionReason`（タスク 11 由来）ごとにリース数を追跡する。回収ポリシーはまだ持たない（それは retention、タスク 17-18）。
    - テスト: acquire/release で数が増減する、タスク 11 の実行中ビルドのリースの解放が計上される、未知のスナップショット id とリース解放の不一致が `SnapshotError` になる。
    - 依存: 11。仕様: [snapshot.md](./snapshot.md)「Snapshot Lease」。

13. **スナップショット構築 API の堅牢化。** [x]
    - 直接の未検証コンストラクタ（`BuildSnapshot::from_input`, `SnapshotInput::build_snapshot`, `SnapshotInput::build_snapshot_id`）を公開のまま残すか、crate 内に閉じるか、同一性専用の未検証ヘルパーとして改名/文書化するかを決める。
    - レジストリスナップショットの検証済み公開作成経路として `SnapshotRegistry::create_snapshot` を維持する。
    - 同一性テストやツールのために直接構築を残す場合は、下流クレートが誤って作成時検証を迂回しないよう、未検証であることの意味を明示する。
    - テスト: 不正な `SnapshotInput` は検証済み API から公開/レジストリのスナップショットを作れない。直接の未検証構築は公開には使えないか、同一性専用として明示的に文書化/テストされている。
    - 依存: 12。仕様: [snapshot.md](./snapshot.md)「Snapshot Creation」「Error Handling」。

### モジュール: source (`src/source.rs`)

14. **読み込み済みソース型とローダー面。** [x]
    - `SourceInput`, `SourceOriginInput`（`path` / `uri,version,text` / 生成器の text+anchor を持つソース読み込み入力の変種）, `LoadedSource` と `SourceLoader` トレイトを定義する。`load` は `SourceInput` の前に対象の `BuildSnapshotId` を受け取り、スナップショットスコープの `SourceId` を発行できるようにする。`LoadedSource.origin` には snapshot の `SourceOrigin`（タスク 9）を再定義せず再利用する。`hash_text` と `normalize_path`（既存の `normalize_source_path` を再利用）を実装する。
    - `SourceLoadError` を仕様の変種付きで定義する: パッケージルート外のソースパス、未対応のファイル拡張子、不正な UTF-8、読み取り不能なソースファイル、重複するモジュールパス、古い LSP ドキュメントバージョン、パッケージソースに対応付けられないオープンバッファ URI、必須の生成器メタデータが無い生成ソース、`SessionIdAllocator` による source id 発行失敗。
    - テスト: `source_hash` は絶対パス/ドキュメントバージョンを含まない、別の由来の同一テキストはハッシュを共有する。
    - 依存: 1, 4, 6, 9。仕様: [source.md](./source.md)「Public API」「Loaded Source」。

15. **ディスクソースの読み込み。** [x]
    - ディスク読み込みを実装する: パス正規化 + パッケージルート強制、バイト読み込み、UTF-8 検証（非可逆 `U+FFFD` なし）、先頭 BOM 除去、CRLF→LF 正規化、`source_hash`、`LineMap`、`LoadingMap` の生成。
    - 先頭 UTF-8 BOM のみエンコーディングシグネチャとして扱い、非先頭の `U+FEFF` は読み込みテキストに残す。CRLF 対のみ LF に正規化し、単独の `\r` は保持する（プラットフォーム改行として扱わない）。
    - テスト: 不正な UTF-8 は行マップ前に拒否、未対応拡張子は拒否、先頭 BOM → ローディングマップ `0`↔`3`、非先頭 `U+FEFF` は読み込みテキストに保持、CRLF は正規化しつつ単独 `\r` は保持、ルート外パスの拒否。
    - 依存: 14。仕様: [source.md](./source.md)「Disk Source Loading」。

16. **オープンバッファと生成ソースの読み込み。** [x]
    - オープンバッファ読み込み（LSP ドキュメントバージョン検証、URI→パッケージパス、BOM 除去、CRLF 正規化、エディタオフセットへ戻すローディングマップ）と、生成ソース読み込み（生成器メタデータ + アンカー）を実装する。
    - テスト: オープンバッファは一致するバージョンでのみディスクを上書き、古いバージョンは拒否、オープンバッファのローディングマップは読み込みテキストオフセットを（LSP UTF-16 変換の前に）エディタ提供テキストのバイトオフセットへ戻す、対応付け不能なオープンバッファ URI は拒否、メタデータの無い生成ソースは拒否。
    - 依存: 15。仕様: [source.md](./source.md)「Open-Buffer Source Loading」「Generated Source Loading」。

### モジュール: retention (`src/retention.rs`)

17. **保持マネージャとリース。** [x]
    - `pub mod retention;` を追加する。`RetentionManager`, `RetainSnapshotInput`, `RetainGuard`, `RetainOwner` と、参照カウント付きの `retain_snapshot`/`release` を定義する。`RetentionReason`（タスク 11 で定義）は再定義せず再利用する。
    - スナップショットレジストリの既存 `SnapshotLease` を `retain_existing_lease` で保持の会計へ接続し、`create_snapshot` が返す実行中ビルドのリースが、重複したリース id を割り当てずに保持の回収可否をブロックできるようにする。
    - `RetentionError` を仕様の変種付きで定義する: 未知のスナップショット id、未知/解放済みのリース id、リースとスナップショットの不一致、不正な所有者/理由の組み合わせ、欠落スナップショットを現行にしようとする、回収が不整合な保持状態でブロックされる。
    - 古いスナップショットの保持は、診断 / 説明 / LSP の古い表示 / IR 出力の理由では許可するが、スナップショットを現行にしてはならない。
    - テスト: アクティブなリースが回収対象化を防ぐ、二重解放はアンダーフローせず報告される、不正な所有者/理由の組み合わせは拒否、古いスナップショットの保持は現行にせず成功する。
    - 依存: 13。仕様: [retention.md](./retention.md)「Retain」「Release」「Error Handling」。

18. **現行マークと回収。** [x]
    - `mark_current`/`unmark_current`、`record_retained_resources`、`collect`、`CollectionSummary` を追加し、回収ポリシー（リース無し・現行マーク無し・保持マップ/説明/LSP/IR のリース無し）を実装する。
    - `CollectionSummary` は、スキャン/回収したスナップショット数、解放したソースとマップ、現行マークでスキップしたスナップショット、生存中リースでスキップしたスナップショット、失効/不一致リースの診断を報告する。
    - テスト: 現行マークは他のリースが無くても回収を防ぐ、最後のリース解放で回収される、フェーズ出力リースは解放まで回収を阻む、欠落スナップショットを現行にしようとすると `RetentionError` になる、`CollectionSummary` が現行/リースでのスキップ数と失効リース診断を報告する、回収はアーティファクト/キャッシュを削除しない。
    - 依存: 17。仕様: [retention.md](./retention.md)「Collection」「Current Marks」「Collection Summary」。

### 横断フォローアップ前のモジュール全体メンテナンス

19. **実装リファクタリングパス。** [x]
    - 最初の実装パスが完了した状態で、`ids`、`source_map`、`snapshot`、`source`、`retention` をレビューする。
    - 明確なバグまたは仕様不一致が見つからない限り、公開 API と挙動は安定させる。
    - 大きな書き換えよりも、小さな局所抽出、共有テストフィクスチャ、命名整理を優先する。
    - 仕様との対応関係を見えにくくしない範囲で、タスク実装中に生じた重複を取り除く。
    - テスト: 挙動保持のリファクタに必要な assertion だけ更新し、既存モジュールテストを成功状態に保つ。
    - 依存: 18。仕様: mizar-session の全モジュール仕様。

20. **ソースと仕様の対応関係監査。** [x]
    - `ids.md`、`source_map.md`、`snapshot.md`、`source.md`、`retention.md` の公開 API、エラー変種、タスク要件から、実装ソースとテストへの軽量なトレーサビリティを確認する。
    - 未実装、古い仕様記述、未規定の挙動、テスト不足があれば、広い変更を監査に混ぜずフォローアップタスクとして記録する。
    - まず英語の正典仕様を確認し、その後、日本語版が同じ API と挙動上の約束を持っているか確認する。
    - テスト: 監査で小さく安全なギャップが見つかった場合を除き、プロダクトテストの追加は想定しない。編集があれば標準の検証コマンドを実行する。
    - 依存: 19。仕様: mizar-session の全モジュール仕様と本 TODO。
    - 監査結果: タスク 1-19 の実装とユニットテストの網羅は概ね
      追跡可能。残る差分はフォローアップタスク 25-28 として記録した。

## 横断的フォローアップタスク

21. **英日ドキュメント同期監査。** [x]
    - `doc/design/mizar-session/en/` の英語正典文書と、`doc/design/mizar-session/ja/` の日本語版をすべて比較する。
    - タスク 1-20 で導入された API 一覧、タスク状態、用語、リンクを同期する。
    - 日本語版を同じ変更で完全同期できない場合は、そのギャップを明示し、英語正典の該当節へリンクする。
    - テスト: ドキュメントのみ。リポジトリに既定の整形/リンクチェックコマンドがあれば実行する。
    - 依存: 20。仕様: リポジトリのドキュメント方針。
    - 監査結果: 英語正典文書と日本語版は、モジュールステータス、
      公開 API/エラー一覧、タスクステータス、タスク 1-20 で導入された用語、
      日本語版内のローカルリンクについて同期済み。未同期の日本語版の
      ギャップは残っていない。

22. **決定性プロパティテスト。** [x]
    - 同一の正準入力から、挿入順やスケジューリングに似た構築順に依らず同一の `BuildSnapshotId` が得られることを、crate レベルの決定性テストで補強する。
    - 等価な保持済みの line/loading/preprocess map に対するソース範囲変換の決定性を確認する。
    - 実装詳細ではなく、決定的な公開挙動に焦点を絞る。
    - テスト: プロパティ/回帰テストを追加し、`cargo test -p mizar-session` を実行する。
    - 依存: 20。仕様: [ids.md](./ids.md)、[snapshot.md](./snapshot.md)、[source_map.md](./source_map.md)。
    - 結果: `crates/mizar-session/tests/determinism.rs` に公開された統合
      網羅を追加し、レジストリで作成された `BuildSnapshotId` が
      ソース/依存の挿入順、スケジューリングに似た source-id 割り当て順、
      等価な保持済みソースマップの変換順に依存しないことを確認する。

23. **スナップショットリース割り当て mutex の堅牢化。** [x]
    - `SnapshotRegistry::acquire_lease` のリース id 割り当てを、`create_snapshot` と同様にレジストリの mutex の外へ出すべきかを決める。
    - 変更する場合は、既存のリース数の挙動を維持し、アロケータの失敗がレジストリ状態を変更しないようにする。
    - 変更しない場合は、その理由を [snapshot.md](./snapshot.md) または本 TODO に記録する。
    - 決定: `acquire_lease` は割り当て前に未知スナップショットを拒否し、既知スナップショットではレジストリの mutex の外でリース id を割り当て、重複 id 防御はここでは追加せず mutex の下でリースを記録する。
    - テスト: アロケータの失敗がレジストリ状態を変えないこと、繰り返しのリース取得が一意で理由ごとに計上されること。
    - 依存: 20。仕様: [snapshot.md](./snapshot.md)「Snapshot Lease」。

24. **スナップショットリースの重複 id 防御。** [x]
    - `SessionIdAllocator` は一意なリース id を発行する前提だが、`SnapshotRegistry` の状態側にも防御的な重複リース id チェックまたはデバッグ表明を追加するかを決める。
    - 実装する場合は、重複割り当てを内部的な割り当て/レジストリのエラーとして表し、リース数や現行スナップショット状態を壊さない。
    - デバッグ表明のみにする場合は、アロケータ契約に対してそれで十分な理由を文書化する。
    - テスト: 挙動がデバッグ表明の外から観測可能な場合は、カスタムアロケータによる重複 id シナリオを追加する。
    - 判断: `SessionIdAllocator` は公開トレイトであり、リリースビルドのレジストリでもカスタムアロケータが生存中リースのマップを上書きしながらカウントを増やせてはならないため、`SnapshotRegistryState` に観測可能な重複生存中リース id チェックを追加する。重複割り当ては、スナップショットレコード、現行マーク、生存中リース、リース数を変更する前に `SnapshotError::DuplicateLeaseIdAllocation` として報告する。
    - テスト: `acquire_lease` と `create_snapshot` で返された重複 id が、リース数、生存中リース、スナップショットレコード、現行リクエスト状態を変更しないこと。
    - 依存: 23。仕様: [snapshot.md](./snapshot.md)「Snapshot Lease」、[ids.md](./ids.md)「Allocator-Issued Id Construction」。

25. **公開 API ブロックとソースマップのエラー経路の仕様同期。** [x]
    - 現在の公開 API ブロックに載っていない実装済みの公開ヘルパー / エイリアスを、
      意図した公開 API として残すか、公開範囲を狭めるかを決める。少なくとも
      `Hash::{from_bytes, as_bytes}`、`LineMap::source`、
      `TextRange::{new, try_new, len, is_empty}`、`DocumentUri`、
      `LspDocumentVersion`、`NormalizedPath::as_str` を監査する。
    - その判断に合わせて、英語版と日本語版の公開 API ブロックとエラー一覧を同期する。
      逆順範囲が公開エラーのままなら、実装済みの
      `SourceMapError::ReversedRange` 変種も含める。
    - API 判断で明示的に狭める場合を除き、既存の検証挙動は安定させる。
    - テスト: 現状の経路を文書化するだけならドキュメントのみ。公開面を
      変更する場合はユニットテストまたはコンパイル失敗の網羅を調整する。
    - 依存: 20。仕様: [ids.md](./ids.md), [source.md](./source.md), [source_map.md](./source_map.md)。
    - 判断: 既存のヘルパーとエイリアスは公開のまま維持し、意図した API として文書化する。
      検証挙動は狭めない。`Hash` のバイトヘルパーは、単体の公開シリアライズ形式ではなく、
      低レベルの正準バイトアクセサとして残す。`LineMap::source`、`TextRange` のヘルパー、
      `DocumentUri`、`LspDocumentVersion`、`NormalizedPath::as_str` は公開のまま残す。
      手動構築された逆順の `SourceRange` / `TextRange` に対する公開エラー経路として、
      `SourceMapError::ReversedRange` を含める。
    - テスト方針: Rust の公開面と検証挙動は維持したためドキュメントのみ。

26. **source / snapshot のソース同一性の検証境界。** [x]
    - 空または不正な `WorkspaceRoot`、`PackageId`、`ModulePath`、`Edition`、
      生成ソースのメタデータを、コンストラクタ、ソース読み込み、スナップショット作成、
      上流のビルドプラン層のどこで拒否するかを決める。
    - `SourceLoadError::DuplicateModulePath` を将来のソース読み込みアグリゲータが
      送出するのか、重複するモジュールパスは
      `SnapshotRegistry::create_snapshot` の検証責務だけなのかを明確にする。
    - source または snapshot の検証を強化する場合は、不正なソース同一性を
      スナップショットのハッシュ化の前に拒否し、決定的なハッシュ化を保つ。
    - テスト: 選んだ境界に対して、空の package/module/edition 値と重複する
      モジュールパスを中心に焦点を絞ったケースを追加する。
    - 依存: 20。仕様: [source.md](./source.md), [snapshot.md](./snapshot.md)。
    - 判断: `WorkspaceRoot`、`PackageId`、`ModulePath`、`Edition`、
      `GeneratedSourceKind` のコンストラクタは失敗しない文字列ラッパーのまま維持する。
      上流のビルド計画は、正規化済みのワークスペースルート、package id、edition、
      正準なモジュール探索を所有する。ソース読み込みは、読み込み済み入力に対する
      ソースパス、テキスト、オープンバッファの鮮度、生成ソースメタデータの検証を所有する。
      `SnapshotRegistry::create_snapshot` は最後のハッシュ化前のレジストリ境界として、
      空の workspace root、空または空白を含む package id、不正な module path
      要素（予約語を含む）、空の edition、直接渡された `SourceVersion` 入力内の空の
      生成ソースメタデータ、重複するソースバージョンの同一性、重複するモジュールパスを、
      ハッシュ化、リース割り当て、レジストリ挿入より前に拒否する。空でない package-id の綴りは、
      パッケージ名仕様が揃うまで上流のビルドプランの責務に残す。
    - 判断: `SourceLoadError::DuplicateModulePath` は、ビルドプラン全体を見る将来の
      ソース読み込みアグリゲータのために予約しておく。単一の `SourceLoader::load`
      呼び出しはこれを送出せず、スナップショット作成が必須のスナップショット全体での重複
      モジュールパスチェックを保持する。
    - テスト方針: 焦点を絞ったスナップショットのユニットテストで、空の workspace root、package id、
      module path、edition、直接渡された生成ソースメタデータ、既存の重複モジュール
      パス/ソース同一性のハッシュ化前チェックを網羅する。source テストは
      source-id 割り当て前の生成ソースメタデータ拒否を維持し、
      予約語の名前空間要素を拒否する。
    - 解決済みフォローアップ: `doc/spec/en/23.package_management_and_build_system.md`
      (`[a-z][a-z0-9-]*`) と `doc/spec/en/12.modules_and_namespaces.md` (`snake_case`)
      の英語正本のパッケージ名綴りの矛盾は、小文字の `snake_case`
      (`[a-z][a-z0-9]*(?:_[a-z0-9]+)*`) を採用し、ハイフン正規化を行わない方針で
      解決した。日本語 companion も同期済みであり、
      `doc/spec/ja/23.package_management_and_build_system.md` の
      パッケージ名テーブル行の不正なテキストも修正済み。

27. **生成ソースの正規化方針。** [x]
    - 生成ソーステキストを UTF-8 検証後に 1 バイトもたがえず保持するのか、先頭
      BOM 除去や CRLF→LF 変換のようなソース読み込みの正規化を生成入力にも適用するのかを決めて文書化する。
    - 現在の実装は生成テキストをそのまま保持し、`LoadingMap` を出力しない。
      この方針を維持するなら、英語版と日本語版の `source.md` の生成ソース節で明示する。
    - 生成入力も disk/open-buffer と同様に正規化するなら、実装、ソースマップの
      期待、source-hash テストを同時に更新する。
    - テスト: 選んだ挙動に対して、生成ソースの BOM/CRLF に焦点を絞ったケースを追加または更新する。
    - 依存: 20。仕様: [source.md](./source.md), [source_map.md](./source_map.md)。
    - 判断: 生成ソース読み込みは 1 バイトもたがえず保持するままにする。生成入力はすでに
      有効な UTF-8 の `Arc<str>` として API に入り、ローダーは受理したテキストを
      そのまま保持する。先頭 `U+FEFF` はエンコードシグネチャとして扱わず、
      CRLF ペアは LF に変換せず、単独の `\r` も変更しない。`source_hash` と
      `LineMap` は、その正確な生成テキストに対して計算する。生成読み込みは
      `LoadingMap` を出力しない。生成ソースの位置は
      `LoadedSource.generated_anchor`、`SourceAnchor::Generated`、
      `GeneratedSpanOrigin` を通して復元する。パッケージ記述テキストと同じ
      正規化が必要な生成器は、`SourceOriginInput::Generated` を構築する前に
      自分の出力を正規化しなければならない。
    - テスト方針: `source.rs` に、先頭 `U+FEFF` と CRLF を含む生成ソースを
      焦点を絞ったケースとして追加し、テキストが 1 バイトもたがえず保持されること、
      正規化後の綴りではなく正確なテキストとしてハッシュ化されること、
      `LoadingMap` を出力しないことを確認する。
    - フォローアップ: この方針についてはなし。`LoadingOrigin::Generated` は、
      生成テキストに対してサービスレベルの読み込み済みから元入力への変換を保持したい
      カスタム呼び出し側が恒等マップを作る場合に引き続き利用できるが、既定のローダーは
      それを作らない。

28. **予約済み / 診断専用エラー変種のトレーサビリティ。** [x]
    - 現時点で予約済み、カスタムローダー専用、または診断サマリ専用である
      公開エラー変種を分類する。対象には `IdError::UnknownSnapshotRegistry`、
      `SnapshotError::InvalidSourcePath`、
      `SourceLoadError::UnsupportedSourceOrigin`、retention の不整合状態の
      経路を含める。
    - 各変種について、公開に観測可能な経路と焦点を絞ったテストを追加するか、
      公開の非網羅 enum に残す理由を予約/内部として文書化する。
    - `retention.md` では、回収時の失効/不一致のリース状態は
      `CollectionSummary::lease_diagnostics` で報告され、
      `RetentionError::CollectionBlockedByInconsistentRetentionState` は
      保持/解放/割り当ての経路で使われることを明確にする。
    - テスト: 新たに観測可能にする経路に必要なケースだけを追加する。変種を
      予約/内部として維持する場合はドキュメントのみ。
    - 依存: 20。仕様: [ids.md](./ids.md), [snapshot.md](./snapshot.md), [source.md](./source.md), [retention.md](./retention.md)。
    - 決定: 新しい公開の観測経路は追加しない。`IdError::UnknownSnapshotRegistry` は
      レジストリ対応のカスタムアロケータのために予約し、
      `InMemorySessionIdAllocator` は送出しない。
      `SnapshotError::InvalidSourcePath` は、公開された `create_snapshot` がすでに正規化済みの
      `SourceVersion` レコードを消費するため、将来のスナップショット構築または
      再検証経路向けの予約/内部変種として残す。
      `SourceLoadError::UnsupportedSourceOrigin` は、`DiskSourceLoader` が現在のすべての
      `SourceOriginInput` 変種をサポートするためカスタムローダー専用とする。
      retention の回収時の失効/不一致のリース状態は
      `CollectionSummary::lease_diagnostics` を通る診断サマリ専用の経路とし、
      `RetentionError::CollectionBlockedByInconsistentRetentionState` は
      保持/解放/割り当ての不整合に使う。割り当て側の既存テストはこのエラー経路を
      すでに網羅しており、文書化した解放経路について焦点を絞った欠落生存中リース計数テストを追加した。

29. **lint 強制の恒久化。** [x]
    - [推奨検証](#推奨検証) の clippy/rustc lint ゲートを、
      各コントリビューターの手動実行に頼るのではなく、ツリー内で恒久的に強制する。
      ワークスペースには現在 `[workspace.lints]` テーブルも crate レベルの lint
      属性も無い。
    - ワークスペースの `[workspace.lints]` テーブル（rustc + clippy グループ）と、
      `crates/mizar-session/Cargo.toml` の `lints.workspace = true` を優先する。
      これにより素の `cargo build`/`cargo test` でも単体の clippy コマンドと同じ
      拒否が出るようにする。代わりに crate ローカルの
      `#![deny(...)]`/`#![warn(...)]` 方針を選ぶ場合は、その理由を文書化する。
    - 基準とする重大度を決める（少なくとも `warnings` と `clippy::all` を deny。
      `clippy::pedantic` の選択的有効化も検討）。意図的な `allow` 例外は、その
      `allow` のそばに理由を添えて記録する。
    - 既存の公開 API と挙動は変更しない。本タスクは lint 設定の追加と、
      クリーンなゲートに到達するために必要な機械的修正のみを行う。
    - テスト: ゲートを有効にした状態で
      `cargo clippy -p mizar-session --all-targets -- -D warnings` が通り、
      `cargo test -p mizar-session` が成功したままであること。
    - 依存: 20。仕様: 本 TODO「Suggested Verification」。
    - 決定: rustc `warnings` と `clippy::all` を deny するワークスペース
      `[workspace.lints]` のベースラインを追加し、`crates/mizar-session/Cargo.toml`
      で `lints.workspace = true` によってオプトインする。
      `mizar-lexer`、`mizar-lsp`、`mizar-test` はオプトインせず、本タスクの
      強制範囲を `mizar-session` 外へ広げない。今後の crate はマニフェストの
      オプトインを追加するだけで同じ方針を採用できる。
    - 決定: `clippy::pedantic` はベースラインに含めない。
      `-W clippy::pedantic -D warnings` の試行では、`similar_names`、
      `# Errors`/`# Panics` の不足、`must_use_candidate` など API doc/スタイル
      中心の広範な手直しが発生し、本タスクの lint ゲート範囲を超えるため。
      新しい `allow` 例外は不要だった。すでに接続済みの snapshot ID 導出
      ヘルパーに付いていた古い `allow(dead_code)` 属性は、代わりに削除した。

30. **肥大化したモジュールファイルの分割。** [x]
    - 最大級のソースファイル（`snapshot.rs`、`source_map.rs`、`source.rs` は
      テストを含めて各おおよそ 2.3k〜3.4k 行）を、`lib.rs` から再エクスポートされる
      公開 API 面やモジュール仕様の「Public API」ブロックを変えずに、凝集した
      サブモジュールへ抽出して縮小する。
    - 大きな `#[cfg(test)]` ブロックを兄弟のテストモジュールや `tests/` 形式の
      ファイルへ移し、明確に独立した関心事（例: スナップショット同一性 / リース会計 /
      レジストリ）を同じ公開モジュールパス配下の子モジュールへ分離することを優先する。
    - `mod` の可視性と再エクスポートは安定させ、下流クレートと仕様の「Public API」
      ブロックから見て変化が無いようにする。
    - テスト: 挙動保持。全モジュールテストと doctest を成功状態に保ち、標準の
      検証コマンドを再実行する。
    - 依存: 19, 20。仕様: mizar-session の全モジュール仕様。
    - 決定: `snapshot`、`source_map`、`source` の大きな `#[cfg(test)]`
      ブロックを、`src/snapshot/tests.rs`、`src/source_map/tests.rs`、
      `src/source/tests.rs` の非公開な兄弟テストモジュールへ分割した。
      公開モジュールパスと `lib.rs` の再エクスポートは変えず、実装ファイルを
      プロダクションコード中心に保つ。

31. **オープンバッファ読み込みのエラー特定性。** [x]
    - 問題: `DiskSourceLoader::normalize_open_buffer_uri` が
      `normalize_source_path` の全失敗を
      `SourceLoadError::UnmappedOpenBufferUri` に丸めるため（`src/source.rs`）、
      パッケージ位置には解決できるが後段のパスチェック（未対応拡張子、
      非正準のエイリアス/綴り、不正な名前空間要素）で失敗した
      オープンバッファ URI が、ディスク読み込みが使う具体的なカテゴリではなく
      「対応付け不能な URI」として報告される。
    - オープンバッファ読み込みが「URI がそもそもどのパッケージソースにも
      対応付けられない」（`file://` でないスキーム、デコード不能なパーセント
      エンコーディング、パッケージルート外）ケースと、ディスク読み込みが
      `SourceLoadError::from_source_path_error` を通じてすでに具体的に報告している
      パスエラー（`UnsupportedFileExtension`、`InvalidSourcePath`）を区別すべきかを
      決める。
    - 再分類する場合は既存のディスク用マッピングを再利用し、ディスクと
      オープンバッファの由来で一貫したエラーカテゴリを共有する。
      `UnmappedOpenBufferUri` は、パッケージ相対パスになり得ない URI のみに残す。
    - 既存の検証挙動（同じ入力を引き続き拒否する）は維持し、報告される
      エラー変種のみ変える。
    - その判断に合わせて、英語版と日本語版の `source.md`「Open-Buffer Source
      Loading」「Error Handling」節を同期する。
    - テスト: パッケージ `src/` 配下で `.miz` 以外の拡張子を持つオープンバッファ
      URI は未対応拡張子のカテゴリで報告される。`file://` でない、または
      デコード不能な URI は引き続き `UnmappedOpenBufferUri` で報告される。
    - 依存: 20。仕様: [source.md](./source.md)「Open-Buffer Source Loading」,
      「Error Handling」。
    - 判断: オープンバッファ URI の失敗を、URI からパッケージへの対応付け失敗と、
      対応付け後のパス検証失敗に分ける。`UnmappedOpenBufferUri` は、
      `file://` でないスキーム、デコード不能なパーセントエンコーディング、パッケージルート外の
      file URI など、パッケージ相対パスになれない URI に限定する。URI が
      パッケージ相対パスへ対応付けられた後は、`DiskSourceLoader` がディスク読み込み用の
      `SourceLoadError::from_source_path_error` 分類を再利用し、未対応拡張子や不正な
      ソースパスについて disk/open-buffer が同じカテゴリを報告する。

32. **`src/` ルート欠落パスエラーの忠実度。** [x]
    - 問題: `SourceLoadError::from_source_path_error` が
      `SourcePathError::MissingSourceRoot` を
      `SourceLoadError::SourcePathOutsidePackageRoot` に写像するため
      （`src/source.rs`）、パッケージルート内だが必須の `src/` ソースツリー配下に
      無いパスが「パッケージルート外」として報告され、実態と食い違う。
    - `src/` ルート欠落ケースを区別して表面化するかを決める: 専用の
      `SourceLoadError` 変種（例: `SourcePathOutsideSourceRoot`）を追加するか、
      `SourcePathError::MissingSourceRoot` を `SourceLoadError::InvalidSourcePath`
      経由で運び、パッケージルート境界の変種を流用せずに実際の条件を
      メッセージへ反映する。
    - これは公開の非網羅 `SourceLoadError` 列挙と、現在 `src/` 要件を
      「source path outside package root」に束ねている `source.md` のエラー一覧に
      影響する。判断に合わせて一覧と文言を更新する。
    - 既存の拒否挙動は維持し、エラーの同一性とメッセージのみ変える。
    - その判断に合わせて、英語版と日本語版の `source.md`「Disk Source Loading」
      （ステップ 2）「Error Handling」節を同期する。
    - テスト: パッケージルート内だが `src/` 外のパスが、本当にパッケージルート外の
      パスとは区別して、ソースルート欠落の条件として報告される。
    - 依存: 20。仕様: [source.md](./source.md)「Disk Source Loading」,
      「Error Handling」。
    - 判断: package へ対応付けられるが source-path normalization に失敗するパスの
      公開 carrier として、`SourceLoadError::InvalidSourcePath` を維持する。
      `SourcePathError::MissingSourceRoot` はその変種を通じて観測可能なままにし、
      追加の公開 `SourceLoadError` 変種を増やさずに「package root 内だが `src/`
      外」と実際の package-root escape を区別できるようにする。
    - テスト方針: disk と mapped-open-buffer の focused tests が package root 内
      だが `src/` 外のパスをカバーし、package-root escape tests は引き続き
      `SourcePathOutsidePackageRoot` / `UnmappedOpenBufferUri` をカバーする。
    - 結果: `source.md` と英語正本が refined error boundary を文書化した。
      `crates/mizar-session/src/source/tests.rs` には
      `SourcePathError::MissingSourceRoot` を `SourceLoadError::InvalidSourcePath`
      経由で保持する regression coverage がある。

## 推奨検証

各タスクの後に、以下を実行する:

```text
cargo test -p mizar-session
cargo test -p mizar-test
cargo clippy -p mizar-session --all-targets -- -D warnings
```

タスク 4 は `LineMap` / `SourceRange` の面を変えるので、以下も実行する:

```text
cargo test -p mizar-lsp
```

テストが通ったら、このファイルでタスクにチェックを入れる（または「Completed」節へ移す）。

## 注記

- `mizar-session` は識別子・座標の最下層クレート。下流クレートはそのハンドルを利用して source/snapshot 状態を一致させる。
- `mizar-lexer` は本クレートから疎結合に保つこと。字句解析器トークンのスパン統合はフロントエンドの責務（[../../mizar-lexer/ja/todo.md](../../mizar-lexer/ja/todo.md) 参照）。
- ソースマップとスナップショット同一性は内部コンパイラデータであり、外部公開スキーマではない。
