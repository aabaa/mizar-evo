# mizar-session TODO

> 正規言語: 英語。英語版が正典です: [../en/todo.md](../en/todo.md)。

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
`SourceId` は他の全モジュールが参照する共有プリミティブ。

## Ordered Task List

各タスクは単独で実装・テスト・コミットできる粒度。依存順に並んでおり、後続タスクは
先行タスクがマージ済みであることを前提とする。各タスクは `cargo test -p mizar-session`
を緑に保つこと（[Suggested Verification](#suggested-verification) 参照）。

### モジュール: ids (`src/ids.rs`)

1. **不透明な id プリミティブと id newtype。** [x]
   - `lib.rs` に `pub mod ids;` を追加し、公開 id 型を再エクスポートする。
   - 内部の `OpaqueId` プリミティブと、内容由来 id が使う `Hash` newtype を定義する。
   - `BuildSessionId`, `BuildRequestId`, `BuildSnapshotId(Hash)`, `SourceId`, `SourceMapId`, `SnapshotLeaseId` を、適切に `Debug`/`Clone`/`Copy`/`Eq`/`Hash` 付きで定義する。
   - `IdError` 列挙（不正形式 / ドメイン不一致 / 未知レジストリ / オーバーフロー / 永続化不可のシリアライズ）を追加する。
   - テスト: 等価性、copy/clone、id が不透明（意味的な順序を公開しない）こと。
   - 仕様: [ids.md](./ids.md) "Public API", "Identifier Scope"。

2. **内容由来 id のエンコード。** [x]
   - `BuildSnapshotId` の正準的な小文字 16 進シリアライズ/デシリアライズをドメインセパレータ付きで実装し、不正形式/ドメイン不一致は `IdError` で拒否する。
   - 内部ハッシュヘルパー（ドメインセパレータ + スキーマ/ツールチェイン識別子のフック + 非順序コレクションのソート要件）を用意する。snapshot モジュールがこれに入力する（実際のスナップショットのハッシュ化はタスク 10）。
   - アロケータ発行の id を公開スキーマへシリアライズすることを拒否する。
   - テスト: 16 進エンコード/デコードのラウンドトリップ、ドメインセパレータ変更で id が変わる、公開スキーマのシリアライズが永続化不可 id を拒否する。
   - 仕様: [ids.md](./ids.md) "Serialization", "Content-Derived Id Construction"。

3. **セッション id アロケータ。** [x]
   - `SessionIdAllocator` trait と、session/request/source/source-map/lease の各 id 向けの具体的なインメモリアロケータ（単調増加カウンタまたはアリーナインデックス）を定義する。
   - テスト: 1 つのレジストリ内で id が一意であること、アロケータのオーバーフローが `IdError` になること。
   - 仕様: [ids.md](./ids.md) "Allocator-Issued Id Construction"。

### モジュール: source_map (`src/source_map.rs`)

4. **`SourceRange` と `LineMap` へ `SourceId` を統合。** [x]
   - `SourceRange` に `source_id: SourceId` を、`LineMap` に `source_id` + `text_hash: Hash` を追加する。
   - バイトオフセットの意味は維持し、`with_source(source_id, text)` コンストラクタを追加して既存の `new` 経路を維持/調整する。
   - 変換前に、範囲/オフセットが期待するソースに属することを検証する。
   - `SourceMapError` を spec の全変種へ向けて拡張する（各機能の実装時に変種を追加）: unknown source id、range outside source text、UTF-8 境界でないオフセット、行/列オーバーフロー（タスク 5）、前処理済みテキスト外の字句範囲（タスク 7）、欠落したローディングマップセグメント（タスク 6）、欠落した前処理セグメント（タスク 7）、起点の無い生成スパン（タスク 8）。
   - クレート横断の影響: `mizar-lsp::range_mapper` の呼び出し箇所とテストを `SourceId` を渡すよう更新する。
   - テスト: 既存の行/列テストを更新、クロスソースの範囲は拒否される、unknown source id は拒否される。
   - 依存: 1。仕様: [source_map.md](./source_map.md) "Line Map", "Source Range"。
   - 注: これは lexer に対して加法的（lexer は自前の `SourceSpan` を維持、ブリッジは `mizar-lsp` に残る）。span 橋渡しの方針は確認するが、lexer 変更を待たずに進めてよい。

5. **行/列のオーバーフロー方針。** [x]
   - `LineColumn` の値は `u32` のままにし、`SourceMapError::LineColumnOverflow` を追加する。
   - `usize` からの飽和/巻き戻し/縮小ではなく、オーバーフローを報告する。
   - テスト: 表現不能な行/列はオーバーフローを報告、通常のマルチバイト変換は引き続き 1 始まりの Unicode スカラー列を返す。
   - 依存: 4。仕様: [source_map.md](./source_map.md) "Public API"（`LineColumn` の注記）。

6. **ローディングマップ。** [x]
   - `TextRange`（読み込み/字句テキストへのバイト範囲。source-id でスコープされる `SourceRange` とは区別する）を導入する。
   - `LoadingMap`, `LoadingOrigin`, `LoadingMapSegment`（`Original` / `RemovedLeadingBom` / `NormalizedNewline`）を追加する。
   - 読み込みテキスト → 元テキストの対応付けを実装する。オフセットを変える変換が無い場合は恒等とする。
   - テスト: 先頭 BOM で読み込み `0` → 元バイト `3`、CRLF→LF セグメント、正規化セグメントをまたぐ複合対応付け。
   - 依存: 4。仕様: [source_map.md](./source_map.md) "Loading Map", "Loaded-to-Original Mapping"。

7. **前処理マップとアンカー。** [x]
   - `PreprocessMap`, `PreprocessSegment`（`Original` / `RemovedComment` / `SyntheticWhitespace`）と `SourceAnchor` を追加する。
   - 字句 → ソースの対応付けを実装し、長さ 0 の境界では隣接する複合アンカーを返す。
   - テスト: 除去コメントが保持範囲に対応付く、除去コメントをまたぐ字句範囲が複合対応付けになる、合成空白が主たるユーザー範囲にならない。
   - 依存: 6。仕様: [source_map.md](./source_map.md) "Preprocess Map", "Lexical-to-Source Mapping"。

8. **`SourceMapService` と生成スパン。** [x]
   - `MappedSourceRange`（主たる `SourceRange` + 二次アンカー群 + loaded-to-original の `original_input` バイト範囲）を、読み込み/字句の対応付けの複合返り値型として定義する。
   - `SourceMapService` trait（`line_column`, `original_range_for_loaded`, `source_range_for_lexical`, `attach_generated_span`, `validate_range`）と、保持されたマップ上の具体実装を定義する。
   - 理由必須の生成スパン起点（`GeneratedSpanOrigin`）を追加する。
   - テスト: 代表入力に対する各 trait メソッド、複合対応付けは主アンカー + 二次アンカーを返す、起点の無い生成スパンは拒否される。
   - 依存: 5, 7。仕様: [source_map.md](./source_map.md) "Public API", "Generated Spans"。

### モジュール: snapshot (`src/snapshot.rs`)

9. **ソースバージョンのレコード。** [x]
   - `pub mod snapshot;` を追加する。`SourceVersion` と `SourceOrigin`（`Disk` / `OpenBuffer{version}` / `Generated{generator}`）を定義する。
   - `SnapshotError` を spec の変種付きで定義する（後続タスクが必要とする時に追加）: 不正/非正規化のソースパス、duplicate module path、欠落した依存アーティファクト、未対応の lockfile/ツールチェインメタデータ、古いオープンバッファバージョン、未知のスナップショット id、リースリリースの不一致。
   - 正準ソートキー（package id、module path、normalized path、source hash）を用意する。
   - テスト: 挿入順に依らず正準キーで決定的に順序付く。
   - 依存: 1, 4。仕様: [snapshot.md](./snapshot.md) "Source Version"。

10. **ビルドスナップショットの同一性。** [x]
    - `BuildSnapshot` と `SnapshotInput` を定義し、正準入力（ソート済みのソース/依存サマリー、lockfile ハッシュ、ツールチェイン、検証器構成ハッシュ）から内容由来の `BuildSnapshotId` を計算する。セッションローカル id/タイムスタンプは除外する。
    - タスク 9 からの申し送り: 正準キー（package id、module path、normalized path、source hash）で等価に比較される source-version エントリが、スナップショットハッシュを挿入順依存にしてはならない。そのような重複がハッシュ化に到達し得る場合は決定的にエンコードする。望ましくは、タスク 11 の検証で重複する source-version identity をハッシュ化前に拒否する。
    - テスト: 同一の正準入力 ⇒ 同一 id、ソース/依存/構成の変更 ⇒ 異なる id、セッションローカル id はハッシュに含まれない。
    - 依存: 2, 9。仕様: [snapshot.md](./snapshot.md) "Snapshot Identity"。

11. **スナップショットレジストリ、作成、鮮度。** [x]
    - `SnapshotRegistry` に `create_snapshot`、`get`、`is_current_for_request` を定義する。
    - タスク 11 の境界に従う: `create_snapshot` は source-loading 層から読み込み済みの `SourceVersion` レコードを受け取り、作成入力を検証し、id をハッシュし、スナップショットを挿入して、active-build `SnapshotLease` とともに返す。spec のシグネチャは `Result<(BuildSnapshot, SnapshotLease), SnapshotError>` に更新する。（これは「作成時に lease を返すか / 呼び出し側が acquire するか」の論点を spec 寄りに解消する: レジストリが active-build lease を返す。）
    - ここで、最小の `SnapshotLease` ハンドルと、それが持つ依存無しの `RetentionReason` 列挙を導入する（`retention` モジュールと共有。active-build lease は `RetentionReason::ActiveBuild` を使う）。完全なリース計上はタスク 12 で行い、retention（タスク 17）はこの `RetentionReason` を再定義せず再利用する。
    - タスク 9/10 からの申し送り: 正準キーが等しい重複 source-version identity はスナップショットハッシュ化前に拒否し、作成処理が挿入順に敏感な重複レコードを受理しないようにする。
    - テスト: 作成したスナップショットが取得可能で active-build lease を返す、古い id は鮮度チェックで拒否、古いスナップショットは current として報告されない、duplicate module path は拒否、パス正規化済みの重複ソース同一性は source-version canonical key で拒否、欠落した依存アーティファクト/コンテンツフィンガープリントは拒否、未対応の lockfile/ツールチェインメタデータは拒否、構造的に不正な open-buffer version は拒否。expected-vs-actual の真の open-buffer staleness は、リクエストメタデータを持つ source-loading タスクで検証する。
    - 依存: 3, 10。仕様: [snapshot.md](./snapshot.md) "Snapshot Creation", "Freshness Check", "Error Handling"。

12. **スナップショットリースの計上。** [x]
    - レジストリの `SnapshotLease` を `acquire_lease`/`release_lease` で完成させ、`RetentionReason`（タスク 11 由来）ごとにリース数を追跡する。回収ポリシーはまだ持たない（それは retention、タスク 17-18）。
    - テスト: acquire/release で数が増減する、タスク 11 の active-build lease の解放が計上される、未知のスナップショット id とリースリリースの不一致が `SnapshotError` になる。
    - 依存: 11。仕様: [snapshot.md](./snapshot.md) "Snapshot Lease"。

13. **スナップショット構築 API の堅牢化。** [x]
    - 直接の unchecked constructor（`BuildSnapshot::from_input`, `SnapshotInput::build_snapshot`, `SnapshotInput::build_snapshot_id`）を public のまま残すか、crate-private にするか、identity-only の unchecked helper として rename/document するかを決める。
    - registry snapshot の検証済み public 作成経路として `SnapshotRegistry::create_snapshot` を維持する。
    - identity テストや tooling のために直接構築を残す場合は、downstream crate が誤って creation validation を迂回しないよう unchecked semantics を明示する。
    - テスト: invalid `SnapshotInput` は検証済み API から published/registry snapshot を作れない。直接 unchecked construction は public に使えないか、identity-only として明示的に document/test されている。
    - 依存: 12。仕様: [snapshot.md](./snapshot.md) "Snapshot Creation", "Error Handling"。

### モジュール: source (`src/source.rs`)

14. **ロード済みソース型とローダー面。** [x]
    - `SourceInput`, `SourceOriginInput`（`path` / `uri,version,text` / 生成器 text+anchor を持つソース読み込み入力の variant）, `LoadedSource` と `SourceLoader` trait を定義する。`load` は `SourceInput` の前に対象の `BuildSnapshotId` を受け取り、スナップショットスコープの `SourceId` を発行できるようにする。`LoadedSource.origin` には snapshot の `SourceOrigin`（タスク 9）を再定義せず再利用する。`hash_text` と `normalize_path`（既存の `normalize_source_path` を再利用）を実装する。
    - `SourceLoadError` を spec の変種付きで定義する: パッケージルート外のソースパス、未対応のファイル拡張子、不正な UTF-8、読み取り不能なソースファイル、duplicate module path、古い LSP ドキュメントバージョン、パッケージソースに対応付けられないオープンバッファ URI、必須の生成器メタデータが無い生成ソース、`SessionIdAllocator` による source id 発行失敗。
    - テスト: `source_hash` は絶対パス/ドキュメントバージョンを含まない、別 origin の同一テキストはハッシュを共有する。
    - 依存: 1, 4, 6, 9。仕様: [source.md](./source.md) "Public API", "Loaded Source"。

15. **ディスクソースの読み込み。** [x]
    - ディスク読み込みを実装する: パス正規化 + パッケージルート強制、バイト読み込み、UTF-8 検証（非可逆 `U+FFFD` なし）、先頭 BOM 除去、CRLF→LF 正規化、`source_hash`、`LineMap`、`LoadingMap` の生成。
    - 先頭 UTF-8 BOM のみエンコーディング signature として扱い、非先頭の `U+FEFF` は読み込みテキストに残す。CRLF 対のみ LF に正規化し、単独の `\r` は保持する（プラットフォーム改行として扱わない）。
    - テスト: 不正な UTF-8 は line-map 前に拒否、未対応拡張子は拒否、先頭 BOM → ローディングマップ `0`↔`3`、非先頭 `U+FEFF` は読み込みテキストに保持、CRLF は正規化しつつ単独 `\r` は保持、ルート外パスの拒否。
    - 依存: 14。仕様: [source.md](./source.md) "Disk Source Loading"。

16. **オープンバッファと生成ソースの読み込み。** [x]
    - オープンバッファ読み込み（LSP ドキュメントバージョン検証、URI→パッケージパス、BOM 除去、CRLF 正規化、エディタオフセットへ戻すローディングマップ）と、生成ソース読み込み（生成器メタデータ + アンカー）を実装する。
    - テスト: オープンバッファは一致するバージョンでのみディスクを上書き、古いバージョンは拒否、オープンバッファのローディングマップは読み込みテキストオフセットを（LSP UTF-16 変換の前に）エディタ提供テキストのバイトオフセットへ戻す、対応付け不能なオープンバッファ URI は拒否、メタデータの無い生成ソースは拒否。
    - 依存: 15。仕様: [source.md](./source.md) "Open-Buffer Source Loading", "Generated Source Loading"。

### モジュール: retention (`src/retention.rs`)

17. **保持マネージャとリース。** [x]
    - `pub mod retention;` を追加する。`RetentionManager`, `RetainSnapshotInput`, `RetainGuard`, `RetainOwner` と、参照カウント付きの `retain_snapshot`/`release` を定義する。`RetentionReason`（タスク 11 で定義）は再定義せず再利用する。
    - snapshot registry の既存 `SnapshotLease` を `retain_existing_lease` で retention accounting へ接続し、`create_snapshot` が返す active-build lease が、重複した lease id を割り当てずに retention の回収可否をブロックできるようにする。
    - `RetentionError` を spec の変種付きで定義する: 未知のスナップショット id、未知/解放済みのリース id、リースとスナップショットの不一致、不正な owner/reason の組み合わせ、欠落スナップショットを current にしようとする、回収が不整合な保持状態でブロックされる。
    - 古いスナップショットの retain は、診断 / 説明 / LSP の古い表示 / IR 出力の理由では許可するが、スナップショットを current にしてはならない。
    - テスト: アクティブなリースが回収対象化を防ぐ、二重リリースはアンダーフローせず報告される、不正な owner/reason の組み合わせは拒否、古いスナップショットの retain は current にせず成功する。
    - 依存: 13。仕様: [retention.md](./retention.md) "Retain", "Release", "Error Handling"。

18. **current マークと回収。** [x]
    - `mark_current`/`unmark_current`、`record_retained_resources`、`collect`、`CollectionSummary` を追加し、回収ポリシー（リース無し・current マーク無し・保持マップ/説明/LSP/IR のリース無し）を実装する。
    - `CollectionSummary` は、スキャン/回収したスナップショット数、解放したソースとマップ、current マークでスキップしたスナップショット、ライブリースでスキップしたスナップショット、古い/不一致リースの診断を報告する。
    - テスト: current マークは他のリースが無くても回収を防ぐ、最後のリース解放で回収される、フェーズ出力リースは解放まで回収を阻む、欠落スナップショットを current にしようとすると `RetentionError` になる、`CollectionSummary` が current/リースでのスキップ数と古いリース診断を報告する、回収はアーティファクト/キャッシュを削除しない。
    - 依存: 17。仕様: [retention.md](./retention.md) "Collection", "Current Marks", "Collection Summary"。

### 横断フォローアップ前のモジュール全体メンテナンス

19. **実装リファクタリングパス。** [x]
    - 最初の実装パスが完了した状態で、`ids`、`source_map`、`snapshot`、`source`、`retention` をレビューする。
    - 明確なバグまたは仕様不一致が見つからない限り、公開 API と挙動は安定させる。
    - 大きな書き換えよりも、小さな局所抽出、共有テスト fixture、命名整理を優先する。
    - 仕様との対応関係を見えにくくしない範囲で、タスク実装中に生じた重複を取り除く。
    - テスト: 挙動保持のリファクタに必要な assertion だけ更新し、既存モジュールテストを緑に保つ。
    - 依存: 18。仕様: mizar-session の全モジュール仕様。

20. **ソースと仕様の対応関係監査。** [x]
    - `ids.md`、`source_map.md`、`snapshot.md`、`source.md`、`retention.md` の public API、error variant、タスク要件から、実装ソースとテストへの軽量な traceability を確認する。
    - 未実装、古い仕様記述、未規定の挙動、テスト不足があれば、広い変更を監査に混ぜず follow-up task として記録する。
    - まず英語の正典仕様を確認し、その後、日本語 companion が同じ API と挙動上の約束を持っているか確認する。
    - テスト: 監査で小さく安全なギャップが見つかった場合を除き、product test の追加は想定しない。編集があれば標準 verification command を実行する。
    - 依存: 19。仕様: mizar-session の全モジュール仕様と本 TODO。
    - 監査結果: タスク 1-19 の実装とユニットテスト coverage は概ね
      trace 可能。残る差分は follow-up task 25-28 として記録した。

## 横断的フォローアップタスク

21. **英日ドキュメント同期監査。** [x]
    - `doc/design/mizar-session/en/` の英語正典文書と、`doc/design/mizar-session/ja/` の日本語 companion をすべて比較する。
    - タスク 1-20 で導入された API 一覧、タスク状態、用語、リンクを同期する。
    - 日本語 companion を同じ変更で完全同期できない場合は、そのギャップを明示し、英語正典の該当 section へリンクする。
    - テスト: documentation-only。リポジトリに既定の formatting/link check コマンドがあれば実行する。
    - 依存: 20。仕様: リポジトリの documentation policy。
    - 監査結果: 英語正典文書と日本語 companion は、モジュールステータス、
      public API/error list、タスクステータス、タスク 1-20 で導入された用語、
      companion 内のローカルリンクについて同期済み。未同期の日本語 companion
      gap は残っていない。

22. **決定性プロパティテスト。** [x]
    - 同一の正準入力から、挿入順や scheduling-like な構築順に依らず同一の `BuildSnapshotId` が得られることを、crate-level の決定性テストで補強する。
    - 等価な保持済み line/loading/preprocess map に対する source-range 変換の決定性を確認する。
    - 実装詳細ではなく、決定的な public behavior に焦点を絞る。
    - テスト: property/regression test を追加し、`cargo test -p mizar-session` を実行する。
    - 依存: 20。仕様: [ids.md](./ids.md)、[snapshot.md](./snapshot.md)、[source_map.md](./source_map.md)。
    - 結果: `crates/mizar-session/tests/determinism.rs` に public integration
      coverage を追加し、registry で作成された `BuildSnapshotId` が
      source/dependency の挿入順、scheduling-like な source-id 割り当て順、
      等価な保持済み source-map の変換順に依存しないことを確認する。

23. **スナップショットリース割り当て mutex の堅牢化。** [x]
    - `SnapshotRegistry::acquire_lease` の lease id 割り当てを、`create_snapshot` と同様に registry mutex の外へ出すべきかを決める。
    - 変更する場合は、既存の lease count 挙動を維持し、allocator failure が registry state を変更しないようにする。
    - 変更しない場合は、その理由を [snapshot.md](./snapshot.md) または本 TODO に記録する。
    - 決定: `acquire_lease` は allocation 前に未知スナップショットを拒否し、既知スナップショットでは registry mutex の外で lease id を割り当て、duplicate-id defense はここでは追加せず mutex の下で lease を記録する。
    - テスト: allocator failure が registry state を変えないこと、繰り返し lease acquisition が一意で reason ごとに計上されること。
    - 依存: 20。仕様: [snapshot.md](./snapshot.md) "Snapshot Lease"。

24. **スナップショットリース duplicate-id 防御。** [x]
    - `SessionIdAllocator` は一意な lease id を発行する前提だが、`SnapshotRegistry` state 側にも defensive duplicate-lease-id check または debug assertion を追加するかを決める。
    - 実装する場合は、duplicate allocation を内部的な allocation/registry error として表し、lease count や current snapshot state を壊さない。
    - debug assertion のみにする場合は、allocator contract に対してそれで十分な理由を文書化する。
    - テスト: 挙動が debug assertion の外から観測可能な場合は、custom allocator による duplicate id scenario を追加する。
    - 判断: `SessionIdAllocator` は public trait であり、release build の registry でも custom allocator が live lease map を上書きしながら count を増やせてはならないため、`SnapshotRegistryState` に観測可能な duplicate live lease id check を追加する。Duplicate allocation は、snapshot record、current mark、live lease、lease count を変更する前に `SnapshotError::DuplicateLeaseIdAllocation` として報告する。
    - テスト: `acquire_lease` と `create_snapshot` で返された duplicate id が、lease count、live lease、snapshot record、current request state を変更しないこと。
    - 依存: 23。仕様: [snapshot.md](./snapshot.md) "Snapshot Lease"、[ids.md](./ids.md) "Allocator-Issued Id Construction"。

25. **public API block とソースマップ error surface の仕様同期。** [x]
    - 現在の public API block に載っていない実装済みの公開 helper / alias を、
      意図した public API として残すか、公開範囲を狭めるかを決める。少なくとも
      `Hash::{from_bytes, as_bytes}`、`LineMap::source`、
      `TextRange::{new, try_new, len, is_empty}`、`DocumentUri`、
      `LspDocumentVersion`、`NormalizedPath::as_str` を監査する。
    - その判断に合わせて、英語版と日本語版の public API block とエラー一覧を同期する。
      逆順範囲が public error のままなら、実装済みの
      `SourceMapError::ReversedRange` variant も含める。
    - API 判断で明示的に狭める場合を除き、既存の validation 挙動は安定させる。
    - テスト: 現状の surface を文書化するだけなら documentation-only。公開面を
      変更する場合は unit test または compile-fail coverage を調整する。
    - 依存: 20。仕様: [ids.md](./ids.md), [source.md](./source.md), [source_map.md](./source_map.md)。
    - 判断: 既存の helper と alias は public のまま維持し、意図した API として文書化する。
      validation 挙動は狭めない。`Hash` の byte helper は、単体の公開シリアライズ形式ではなく、
      低レベルの正準バイト accessor として残す。`LineMap::source`、`TextRange` helper、
      `DocumentUri`、`LspDocumentVersion`、`NormalizedPath::as_str` は public のまま残す。
      手動構築された逆順の `SourceRange` / `TextRange` に対する public error surface として、
      `SourceMapError::ReversedRange` を含める。
    - テスト方針: Rust の public surface と validation 挙動は維持したため documentation-only。

26. **source / snapshot のソース同一性 validation 境界。** [x]
    - 空または不正な `WorkspaceRoot`、`PackageId`、`ModulePath`、`Edition`、
      生成ソース metadata を、constructor、source loading、snapshot creation、
      上流の build-plan 層のどこで拒否するかを決める。
    - `SourceLoadError::DuplicateModulePath` を将来の source-loading aggregator が
      emit するのか、duplicate module path は
      `SnapshotRegistry::create_snapshot` の validation 責務だけなのかを明確にする。
    - source または snapshot validation を強化する場合は、不正なソース同一性を
      snapshot hashing の前に拒否し、決定的な hashing を保つ。
    - テスト: 選んだ境界に対して、空の package/module/edition 値と duplicate
      module path を中心に focused case を追加する。
    - 依存: 20。仕様: [source.md](./source.md), [snapshot.md](./snapshot.md)。
    - 判断: `WorkspaceRoot`、`PackageId`、`ModulePath`、`Edition`、
      `GeneratedSourceKind` の constructor は失敗しない string wrapper のまま維持する。
      上流の build planning は、正規化済み workspace root、package id、edition、
      canonical module discovery を所有する。source loading は、読み込み済み入力に対する
      source path、text、open-buffer freshness、generated-source metadata validation を所有する。
      `SnapshotRegistry::create_snapshot` は最後の pre-hash registry boundary として、
      空の workspace root、空または whitespace を含む package id、不正な module path
      component（予約語を含む）、空の edition、direct な `SourceVersion` 入力内の空の
      generated-source metadata、重複する source-version identity、重複する module path を、
      hash 化、lease 割り当て、registry 挿入より前に拒否する。空でない package-id spelling は、
      package-name specs が揃うまで上流の build-plan の責務に残す。
    - 判断: `SourceLoadError::DuplicateModulePath` は、build plan 全体を見る将来の
      source-loading aggregator のために予約しておく。単一の `SourceLoader::load`
      呼び出しはこれを emit せず、snapshot creation が必須の whole-snapshot duplicate
      module path check を保持する。
    - テスト方針: focused snapshot unit test で、空の workspace root、package id、
      module path、edition、direct generated-source metadata、既存の duplicate module
      path/source identity の pre-hash check を cover する。source test は
      source-id allocation 前の generated-source metadata rejection を維持し、
      reserved-word namespace component を拒否する。
    - Follow-up: `doc/spec/en/23.package_management_and_build_system.md`
      (`[a-z][a-z0-9-]*`) と `doc/spec/en/12.modules_and_namespaces.md` (`snake_case`)
      の English canonical package-name spelling conflict を解決し、その後 Japanese
      companion を同期する。`doc/spec/ja/23.package_management_and_build_system.md` の
      package-name table row の malformed text も合わせて直す。

27. **生成ソースの正規化方針。** [x]
    - 生成ソーステキストを UTF-8 検証後に byte-for-byte で保持するのか、先頭
      BOM 除去や CRLF→LF 変換のような source-loading 正規化を生成入力にも適用するのかを決めて文書化する。
    - 現在の実装は生成テキストをそのまま保持し、`LoadingMap` を emit しない。
      この方針を維持するなら、英語版と日本語版の `source.md` の生成ソース節で明示する。
    - 生成入力も disk/open-buffer と同様に正規化するなら、実装、source-map
      expectation、source-hash test を同時に更新する。
    - テスト: 選んだ挙動に対して、生成ソースの BOM/CRLF focused case を追加または更新する。
    - 依存: 20。仕様: [source.md](./source.md), [source_map.md](./source_map.md)。
    - 判断: 生成ソース読み込みは byte-for-byte 保持のままにする。生成入力はすでに
      valid UTF-8 の `Arc<str>` として API に入り、ローダーは受理したテキストを
      そのまま保持する。先頭 `U+FEFF` はエンコードシグネチャとして扱わず、
      CRLF ペアは LF に変換せず、単独の `\r` も変更しない。`source_hash` と
      `LineMap` は、その正確な生成テキストに対して計算する。生成読み込みは
      `LoadingMap` を emit しない。生成ソースの位置は
      `LoadedSource.generated_anchor`、`SourceAnchor::Generated`、
      `GeneratedSpanOrigin` を通して復元する。パッケージ記述テキストと同じ
      正規化が必要な生成器は、`SourceOriginInput::Generated` を構築する前に
      自分の出力を正規化しなければならない。
    - テスト方針: `source.rs` に、先頭 `U+FEFF` と CRLF を含む生成ソースを
      focused case として追加し、テキストが byte-for-byte で保持されること、
      正規化後の綴りではなく正確なテキストとして hash 化されること、
      `LoadingMap` を emit しないことを確認する。
    - Follow-up: この方針についてはなし。`LoadingOrigin::Generated` は、
      生成テキストに対して service レベルの loaded-to-original 変換を保持したい
      custom caller が恒等 map を作る場合に引き続き利用できるが、既定のローダーは
      それを作らない。

28. **予約済み / 診断専用 error variant の traceability。** [x]
    - 現時点で予約済み、custom-loader 専用、または diagnostic-summary 専用である
      public error variant を分類する。対象には `IdError::UnknownSnapshotRegistry`、
      `SnapshotError::InvalidSourcePath`、
      `SourceLoadError::UnsupportedSourceOrigin`、retention の inconsistent state
      surface を含める。
    - 各 variant について、public に観測可能な経路と focused test を追加するか、
      public な non-exhaustive enum に残す理由を reserved/internal として文書化する。
    - `retention.md` では、collection 時の stale/mismatched lease state は
      `CollectionSummary::lease_diagnostics` で報告され、
      `RetentionError::CollectionBlockedByInconsistentRetentionState` は
      retain/release/allocation 経路で使われることを明確にする。
    - テスト: 新たに観測可能にする経路に必要な case だけを追加する。variant を
      reserved/internal として維持する場合は documentation-only。
    - 依存: 20。仕様: [ids.md](./ids.md), [snapshot.md](./snapshot.md), [source.md](./source.md), [retention.md](./retention.md)。
    - 決定: 新しい public observable path は追加しない。`IdError::UnknownSnapshotRegistry` は
      registry-aware な custom allocator のために予約し、
      `InMemorySessionIdAllocator` は emit しない。
      `SnapshotError::InvalidSourcePath` は、public な `create_snapshot` がすでに正規化済みの
      `SourceVersion` record を消費するため、将来の snapshot construction または
      revalidation 経路向けの reserved/internal variant として残す。
      `SourceLoadError::UnsupportedSourceOrigin` は、`DiskSourceLoader` が現在のすべての
      `SourceOriginInput` variant を support するため custom-loader 専用とする。
      retention の collection 時の stale/mismatched lease state は
      `CollectionSummary::lease_diagnostics` を通る diagnostic-summary-only surface とし、
      `RetentionError::CollectionBlockedByInconsistentRetentionState` は
      retain/release/allocation の不整合に使う。allocation 側の既存テストはこの error surface を
      すでに cover しており、文書化した release 経路について focused な missing live-lease-count test を追加した。

29. **lint 強制の恒久化。** [x]
    - [Suggested Verification](#suggested-verification) の clippy/rustc lint gate を、
      各コントリビューターの手動実行に頼るのではなく、ツリー内で恒久的に強制する。
      ワークスペースには現在 `[workspace.lints]` テーブルも crate レベルの lint
      属性も無い。
    - ワークスペースの `[workspace.lints]` テーブル（rustc + clippy グループ）と、
      `crates/mizar-session/Cargo.toml` の `lints.workspace = true` を優先する。
      これにより素の `cargo build`/`cargo test` でも単体の clippy コマンドと同じ
      denial が出るようにする。代わりに crate ローカルの
      `#![deny(...)]`/`#![warn(...)]` 方針を選ぶ場合は、その理由を文書化する。
    - 基準とする severity を決める（少なくとも `warnings` と `clippy::all` を deny。
      `clippy::pedantic` の選択的有効化も検討）。意図的な `allow` 例外は、その
      `allow` のそばに理由を添えて記録する。
    - 既存の public API と挙動は変更しない。本タスクは lint 設定の追加と、
      クリーンな gate に到達するために必要な機械的修正のみを行う。
    - テスト: gate を有効にした状態で
      `cargo clippy -p mizar-session --all-targets -- -D warnings` が通り、
      `cargo test -p mizar-session` が緑のままであること。
    - 依存: 20。仕様: 本 TODO "Suggested Verification"。
    - 決定: rustc `warnings` と `clippy::all` を deny する workspace
      `[workspace.lints]` baseline を追加し、`crates/mizar-session/Cargo.toml`
      で `lints.workspace = true` によって opt-in する。
      `mizar-lexer`、`mizar-lsp`、`mizar-test` は opt-in せず、本タスクの
      enforcement 範囲を `mizar-session` 外へ広げない。今後の crate は manifest の
      opt-in を追加するだけで同じ policy を採用できる。
    - 決定: `clippy::pedantic` は baseline に含めない。
      `-W clippy::pedantic -D warnings` の試行では、`similar_names`、
      `# Errors`/`# Panics` の不足、`must_use_candidate` など API doc/style
      中心の広範な churn が発生し、本タスクの lint gate 範囲を超えるため。
      新しい `allow` 例外は不要だった。すでに接続済みの snapshot ID derivation
      helper に付いていた古い `allow(dead_code)` 属性は、代わりに削除した。

30. **肥大化したモジュールファイルの分割。** [x]
    - 最大級のソースファイル（`snapshot.rs`、`source_map.rs`、`source.rs` は
      テストを含めて各おおよそ 2.3k〜3.4k 行）を、`lib.rs` から再エクスポートされる
      public API 面やモジュール仕様の "Public API" block を変えずに、凝集した
      サブモジュールへ抽出して縮小する。
    - 大きな `#[cfg(test)]` ブロックを兄弟のテストモジュールや `tests/` 形式の
      ファイルへ移し、明確に独立した関心事（例: snapshot identity / lease accounting /
      registry）を同じ public module path 配下の子モジュールへ分離することを優先する。
    - `mod` の可視性と再エクスポートは安定させ、下流クレートと仕様の "Public API"
      block から見て変化が無いようにする。
    - テスト: 挙動保持。全モジュールテストと doctest を緑に保ち、標準の
      verification コマンドを再実行する。
    - 依存: 19, 20。仕様: mizar-session の全モジュール仕様。
    - 決定: `snapshot`、`source_map`、`source` の大きな `#[cfg(test)]`
      ブロックを、`src/snapshot/tests.rs`、`src/source_map/tests.rs`、
      `src/source/tests.rs` の private な兄弟テストモジュールへ分割した。
      public module path と `lib.rs` の再エクスポートは変えず、実装ファイルを
      production code 中心に保つ。

## Suggested Verification

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

テストが通ったら、このファイルでタスクにチェックを入れる（または "Completed" 節へ移す）。

## 注記

- `mizar-session` は識別子・座標の最下層クレート。下流クレートはそのハンドルを consume して source/snapshot 状態を一致させる。
- `mizar-lexer` は本クレートから疎結合に保つこと。lexerトークンの span 統合はフロントエンドの責務（[../../mizar-lexer/ja/todo.md](../../mizar-lexer/ja/todo.md) 参照）。
- source map と snapshot identity は内部コンパイラデータであり、外部公開スキーマではない。
