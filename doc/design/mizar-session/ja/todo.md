# mizar-session TODO

> 正規言語: 英語。英語版が正典です: [../en/todo.md](../en/todo.md)。

## ステータス凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

| モジュール | 仕様 | ソース | ステータス |
|---|---|---|---|
| ids | [ids.md](../en/ids.md) | `src/ids.rs` | [ ] |
| source_map | [source_map.md](../en/source_map.md) | `src/source_map.rs` | [~] |
| snapshot | [snapshot.md](../en/snapshot.md) | `src/snapshot.rs` | [ ] |
| source | [source.md](../en/source.md) | `src/source.rs` | [~] |
| retention | [retention.md](../en/retention.md) | `src/retention.rs` | [ ] |

本クレートは識別子・座標を提供する最下層（leaf）なので、内部依存に沿って
ボトムアップで構築する: `ids` → `source_map` → `snapshot` → `source` → `retention`。
`SourceId` は他の全モジュールが参照する共有プリミティブ。

## Ordered Task List

各タスクは単独で実装・テスト・コミットできる粒度。依存順に並んでおり、後続タスクは
先行タスクがマージ済みであることを前提とする。各タスクは `cargo test -p mizar-session`
を緑に保つこと（[Suggested Verification](#suggested-verification) 参照）。

### モジュール: ids (`src/ids.rs`)

1. **不透明な id プリミティブと id newtype。** [ ]
   - `lib.rs` に `pub mod ids;` を追加し、公開 id 型を再エクスポートする。
   - 内部の `OpaqueId` プリミティブと、内容由来 id が使う `Hash` newtype を定義する。
   - `BuildSessionId`, `BuildRequestId`, `BuildSnapshotId(Hash)`, `SourceId`, `SourceMapId`, `SnapshotLeaseId` を、適切に `Debug`/`Clone`/`Copy`/`Eq`/`Hash` 付きで定義する。
   - `IdError` 列挙（不正形式 / ドメイン不一致 / 未知レジストリ / オーバーフロー / 永続化不可のシリアライズ）を追加する。
   - テスト: 等価性、copy/clone、id が不透明（意味的な順序を公開しない）こと。
   - 仕様: [ids.md](../en/ids.md) "Public API", "Identifier Scope"。

2. **内容由来 id のエンコード。** [ ]
   - `BuildSnapshotId` の正準的な小文字 16 進シリアライズ/デシリアライズをドメインセパレータ付きで実装し、不正形式/ドメイン不一致は `IdError` で拒否する。
   - 内部ハッシュヘルパー（ドメインセパレータ + スキーマ/ツールチェイン識別子のフック + 非順序コレクションのソート要件）を用意する。snapshot モジュールがこれに入力する（実際のスナップショットのハッシュ化はタスク 10）。
   - アロケータ発行の id を公開スキーマへシリアライズすることを拒否する。
   - テスト: 16 進エンコード/デコードのラウンドトリップ、ドメインセパレータ変更で id が変わる、公開スキーマのシリアライズが永続化不可 id を拒否する。
   - 仕様: [ids.md](../en/ids.md) "Serialization", "Content-Derived Id Construction"。

3. **セッション id アロケータ。** [ ]
   - `SessionIdAllocator` trait と、session/request/source/source-map/lease の各 id 向けの具体的なインメモリアロケータ（単調増加カウンタまたはアリーナインデックス）を定義する。
   - テスト: 1 つのレジストリ内で id が一意であること、アロケータのオーバーフローが `IdError` になること。
   - 仕様: [ids.md](../en/ids.md) "Allocator-Issued Id Construction"。

### モジュール: source_map (`src/source_map.rs`)

4. **`SourceRange` と `LineMap` へ `SourceId` を統合。** [ ]
   - `SourceRange` に `source_id: SourceId` を、`LineMap` に `source_id` + `text_hash: Hash` を追加する。
   - バイトオフセットの意味は維持し、`with_source(source_id, text)` コンストラクタを追加して既存の `new` 経路を維持/調整する。
   - 変換前に、範囲/オフセットが期待するソースに属することを検証する。
   - クレート横断の影響: `mizar-lsp::range_mapper` の呼び出し箇所とテストを `SourceId` を渡すよう更新する。
   - テスト: 既存の行/列テストを更新、クロスソースの範囲は拒否される。
   - 依存: 1。仕様: [source_map.md](../en/source_map.md) "Line Map", "Source Range"。
   - 注: これは lexer に対して加法的（lexer は自前の `SourceSpan` を維持、ブリッジは `mizar-lsp` に残る）。span 橋渡しの方針は確認するが、lexer 変更を待たずに進めてよい。

5. **行/列のオーバーフロー方針。** [ ]
   - `LineColumn` の値は `u32` のままにし、`SourceMapError::LineColumnOverflow` を追加する。
   - `usize` からの飽和/巻き戻し/縮小ではなく、オーバーフローを報告する。
   - テスト: 表現不能な行/列はオーバーフローを報告、通常のマルチバイト変換は引き続き 1 始まりの Unicode スカラー列を返す。
   - 依存: 4。仕様: [source_map.md](../en/source_map.md) "Public API"（`LineColumn` の注記）。

6. **ローディングマップ。** [ ]
   - `LoadingMap`, `LoadingOrigin`, `LoadingMapSegment`（`Original` / `RemovedLeadingBom` / `NormalizedNewline`）を追加する。
   - 読み込みテキスト → 元テキストの対応付けを実装する。オフセットを変える変換が無い場合は恒等とする。
   - テスト: 先頭 BOM で読み込み `0` → 元バイト `3`、CRLF→LF セグメント、正規化セグメントをまたぐ複合対応付け。
   - 依存: 4。仕様: [source_map.md](../en/source_map.md) "Loading Map", "Loaded-to-Original Mapping"。

7. **前処理マップとアンカー。** [ ]
   - `PreprocessMap`, `PreprocessSegment`（`Original` / `RemovedComment` / `SyntheticWhitespace`）と `SourceAnchor` を追加する。
   - 字句 → ソースの対応付けを実装し、長さ 0 の境界では隣接する複合アンカーを返す。
   - テスト: 除去コメントが保持範囲に対応付く、除去コメントをまたぐ字句範囲が複合対応付けになる、合成空白が主たるユーザー範囲にならない。
   - 依存: 6。仕様: [source_map.md](../en/source_map.md) "Preprocess Map", "Lexical-to-Source Mapping"。

8. **`SourceMapService` と生成スパン。** [ ]
   - `SourceMapService` trait（`line_column`, `original_range_for_loaded`, `source_range_for_lexical`, `attach_generated_span`, `validate_range`）と、保持されたマップ上の具体実装を定義する。
   - 理由必須の生成スパン起点（`GeneratedSpanOrigin`）を追加する。
   - テスト: 代表入力に対する各 trait メソッド、起点の無い生成スパンは拒否される。
   - 依存: 5, 7。仕様: [source_map.md](../en/source_map.md) "Public API", "Generated Spans"。

### モジュール: snapshot (`src/snapshot.rs`)

9. **ソースバージョンのレコード。** [ ]
   - `pub mod snapshot;` を追加する。`SourceVersion` と `SourceOrigin`（`Disk` / `OpenBuffer{version}` / `Generated{generator}`）を定義する。
   - 正準ソートキー（package id、module path、normalized path、source hash）を用意する。
   - テスト: 挿入順に依らず正準キーで決定的に順序付く。
   - 依存: 1, 4。仕様: [snapshot.md](../en/snapshot.md) "Source Version"。

10. **ビルドスナップショットの同一性。** [ ]
    - `BuildSnapshot` と `SnapshotInput` を定義し、正準入力（ソート済みのソース/依存サマリー、lockfile ハッシュ、ツールチェイン、検証器構成ハッシュ）から内容由来の `BuildSnapshotId` を計算する。セッションローカル id/タイムスタンプは除外する。
    - テスト: 同一の正準入力 ⇒ 同一 id、ソース/依存/構成の変更 ⇒ 異なる id、セッションローカル id はハッシュに含まれない。
    - 依存: 2, 9。仕様: [snapshot.md](../en/snapshot.md) "Snapshot Identity"。

11. **スナップショットレジストリと鮮度。** [ ]
    - `SnapshotRegistry` に `create_snapshot`、`get`、`is_current_for_request` を定義する。
    - テスト: 作成したスナップショットが取得可能、古い id は鮮度チェックで拒否、古いスナップショットは current として報告されない。
    - 依存: 3, 10。仕様: [snapshot.md](../en/snapshot.md) "Snapshot Creation", "Freshness Check"。

12. **スナップショットリース（基本）。** [ ]
    - レジストリに `SnapshotLease` と `acquire_lease`/`release_lease` を追加し、リース数を追跡する。回収ポリシーはまだ持たない。
    - テスト: acquire/release で数が増減する、リリースの不一致が `SnapshotError` になる。
    - 依存: 11。仕様: [snapshot.md](../en/snapshot.md) "Snapshot Lease"。

### モジュール: source (`src/source.rs`)

13. **ロード済みソース型とローダー面。** [ ]
    - `SourceInput`, `SourceOriginInput`, `SourceOrigin`, `LoadedSource` と `SourceLoader` trait を定義し、`hash_text` と `normalize_path`（既存の `normalize_source_path` を再利用）を実装する。
    - テスト: `source_hash` は絶対パス/ドキュメントバージョンを含まない、別 origin の同一テキストはハッシュを共有する。
    - 依存: 1, 4, 6。仕様: [source.md](../en/source.md) "Public API", "Loaded Source"。

14. **ディスクソースの読み込み。** [ ]
    - ディスク読み込みを実装する: パス正規化 + パッケージルート強制、バイト読み込み、UTF-8 検証（非可逆 `U+FFFD` なし）、先頭 BOM 除去、CRLF→LF 正規化、`source_hash`、`LineMap`、`LoadingMap` の生成。
    - テスト: 不正な UTF-8 は line-map 前に拒否、先頭 BOM → ローディングマップ `0`↔`3`、CRLF の扱い、ルート外パスの拒否。
    - 依存: 13。仕様: [source.md](../en/source.md) "Disk Source Loading"。

15. **オープンバッファと生成ソースの読み込み。** [ ]
    - オープンバッファ読み込み（LSP ドキュメントバージョン検証、URI→パッケージパス、BOM 除去、CRLF 正規化、エディタオフセットへ戻すローディングマップ）と、生成ソース読み込み（生成器メタデータ + アンカー）を実装する。
    - テスト: オープンバッファは一致するバージョンでのみディスクを上書き、古いバージョンは拒否、メタデータの無い生成ソースは拒否。
    - 依存: 14。仕様: [source.md](../en/source.md) "Open-Buffer Source Loading", "Generated Source Loading"。

### モジュール: retention (`src/retention.rs`)

16. **保持マネージャとリース。** [ ]
    - `pub mod retention;` を追加する。`RetentionManager`, `RetainSnapshotInput`, `RetainGuard`, `RetainOwner`, `RetentionReason` と、参照カウント付きの `retain_snapshot`/`release` を定義する。
    - テスト: アクティブなリースが回収対象化を防ぐ、二重リリースはアンダーフローせず報告される。
    - 依存: 12。仕様: [retention.md](../en/retention.md) "Retain", "Release"。

17. **current マークと回収。** [ ]
    - `mark_current`/`unmark_current`、`collect`、`CollectionSummary` を追加し、回収ポリシー（リース無し・current マーク無し・保持マップ/説明無し・IR フェーズ出力リース解放済み）を実装する。
    - テスト: current マークは他のリースが無くても回収を防ぐ、最後のリース解放で回収される、フェーズ出力リースは解放まで回収を阻む、回収はアーティファクト/キャッシュを削除しない。
    - 依存: 16。仕様: [retention.md](../en/retention.md) "Collection", "Current Marks"。

## 横断的フォローアップ

- [ ] 実装中に API が変わった場合は `ja/` のモジュール仕様を同期する。
- [ ] クレート全体の決定性プロパティテスト: 同一の正準入力 ⇒ 同一の `BuildSnapshotId` と同一のソース範囲変換が、スケジューリング順に依らず得られる。

## Suggested Verification

各タスクの後に、以下を実行する:

```text
cargo test -p mizar-session
cargo test -p mizar-test
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
