# mizar-diagnostics TODO

> 正本は英語です。英語版: [../en/todo.md](../en/todo.md)。

## 状態の凡例

- [ ] 未着手
- [~] 進行中
- [x] 完了

## モジュール実装

モジュール仕様はまだ存在しない。各仕様は、それを引用する実装タスクより前に、
専用の仕様タスクが（英語と日本語を同じ変更で）執筆する。モジュール名は
[internal 07](../../internal/ja/07.crate_module_layout.md) の最小分割
（`failure_record`、`aggregator`）に、アーキテクチャ 12 と internal 03 の
registry/render/fix/explain モジュールを加えたものに従う。この crate は
アーキテクチャ 12、19 と internal 03 を精緻化する。

| モジュール | 仕様 | ソース | 状態 |
|---|---|---|---|
| registry | `registry.md`（task 2） | `src/registry.rs` | [x] |
| failure_record | `failure_record.md`（task 4） | `src/failure_record.rs` | [x] |
| sink | `sink.md`（task 6） | `src/sink.rs` | [x] |
| aggregator | `aggregator.md`（task 8） | `src/aggregator.rs` | [x] |
| render | `render.md`（task 10） | `src/render.rs` | [x] |
| fix | `fix.md`（task 12） | `src/fix.rs` | [x] |
| explain | `explain.md`（task 14） | `src/explain.rs` | [ ] |

`mizar-diagnostics` はすべての phase が共有する正準 diagnostic レコードを
所有する: 安定した diagnostic コードのレジストリ、構造化された failure
レコード（アーキテクチャ 19）、生産者向け sink API、決定的順序を持つ
ビルドレベルの集約、CLI レンダリング、構造化された fix 提案、遅延
explanation ハンドル。ツールは `DiagnosticCode` をキーにし、メッセージ
テキストには決して依存しない。メッセージはバージョン間で改善してよいが、
コードは再利用してはならない。

依存順序: `registry` → `failure_record` → `sink` → `aggregator` →
`render` / `fix` / `explain`。

以下の各タスクは意図的に小さくしてある — 1 つのモジュール仕様、または
1 モジュールの 1 挙動スライス — 。これにより、crate の残りを抱え込まずに
1 タスクを単独で実装・テスト・コミットまで自律的に完遂できる。

## crate の前提条件

この crate は `mizar-session`（ソース範囲と snapshot id）にのみ依存する。
最初の resolver 採用地点は `mizar-resolve` task 13 で deferred された。次の
trigger は最初の user-facing resolver diagnostic integration である。`mizar-lsp` の
LSP ブリッジは `mizar-diagnostics` の records と indexes の target consumer だが、
real LSP adoption は consumer seam が準備されるまで `external_dependency_gap` のままである。
アーキテクチャ:
[12.diagnostics_and_lsp.md](../../architecture/ja/12.diagnostics_and_lsp.md)、
[19.failure_semantics.md](../../architecture/ja/19.failure_semantics.md)。
internal: [03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)。
仕様: [22.error_handling_and_diagnostics.md](../../../spec/ja/22.error_handling_and_diagnostics.md)。

## 解決済みおよび保留中の決定

- **採用時期: `mizar-resolve` task 13 で deferred。** この crate は目標配置に
  残るが、R-013 は resolver failure を crate-local/internal record として保持し、
  R-015 は name diagnostic を crate-local/internal に保った。resolver diagnostic code
  ownership がまだ `spec_gap` であるためである。後続の user-facing resolver diagnostic
  integration の前に再検討する。
- **既存の crate ごとの診断の移行: 未解決。task 16 で解決する。**
  `mizar-lexer`/`mizar-frontend`/`mizar-parser` の診断はこの crate より
  古い。共有レコードへ移行する（その順序も）か、変換アダプターの背後で
  ローカル型を維持するかを決め、決定とそのトリガーをここととトップ
  レベルに記録する。
- **コード空間の割り当て: 初期 spec-22 registry については task 2 で解決済み。**
  `registry.md` は現在の数値 range、canonical `PhaseFamily` vocabulary、
  descriptor default、retirement finality を固定する。normative range を持たない
  architecture surface は `external_dependency_gap` または `spec_gap` のままとし、
  placeholder allocation を与えてはならない。

## 順序付きタスク一覧

各タスクの後で `cargo test -p mizar-diagnostics` を成功状態に保つこと
（[推奨検証](#推奨検証)を参照）。

### レコードとレジストリ

1. **crate の足場と lint 方針のガード。** [x]
   - `mizar-session` に依存する workspace メンバー `mizar-diagnostics` を
     追加し、`mizar-frontend` のガードに倣った `tests/lint_policy.rs` を
     追加する。
   - テスト: lint 方針ガードが通る。workspace がビルドできる。
   - 依存: なし。仕様: アーキテクチャ 12。
   - task 1 で完了: workspace member、crate manifest、空の boundary root、
     lint-policy guard を追加した。registry、record、sink、adapter、driver、LSP、
     artifact behavior はまだ追加していない。この guard は workspace membership、
     package metadata、`mizar-session` のみの dependency、workspace lint opt-in、
     共有 rustc/clippy lint baseline、documented allow exceptions、空の初期 public API、
     `mizar-diagnostics` への未準備な workspace reverse dependency の禁止を覆う。
     verification は `cargo test -p mizar-diagnostics`、
     `cargo clippy -p mizar-diagnostics --all-targets -- -D warnings`、
     `cargo fmt --check` が通った。

2. **仕様: `registry.md`。** [x]
   - レジストリの仕様を執筆する（英語と日本語、コードなし）: 恒久的な
     `DiagnosticCode` の割り当て、phase ファミリーごとのコード空間、
     retirement 規則、互換性検証、参照メタデータ（意味論名、既定の
     重大度、ドキュメント URL）。
   - 依存: 1。仕様: [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)
     「Diagnostic Registry」、
     [22.error_handling_and_diagnostics.md](../../../spec/ja/22.error_handling_and_diagnostics.md)。
   - task 2 で完了: `registry.md` は stable code identity、初期 spec-22
     phase-family range、canonical `PhaseFamily` 名、descriptor default、allocation
     と final retirement の規則、compatibility validation、spec 22.7 に基づく
     initial allocation、deferred code-space gap、lookup behavior を定義する。
     message text、localized text、rendering、ordering、LSP mapping、proof status、
     driver orchestration、artifact mutation は registry authority の外に置くことを
     明記した。verification は `git diff --check` と `git diff --cached --check` が
     通った。

3. **レジストリの実装。** [x]
   - 互換性検証（コードは別の意味で決して再利用されない）と、割り当て
     済みコードを固定するレジストリ整合性テストを備えたコードレジストリを
     実装する。
   - テスト: 割り当て/retirement のフィクスチャ。再利用の試みの失敗。
     参照メタデータのラウンドトリップ。
   - 依存: 2。仕様: `registry.md`。
   - task 3 で完了: `src/registry.rs` は `DiagnosticCode`、canonical
     `PhaseFamily` と severity metadata、spec-22 の built-in descriptor、code /
     semantic name / alias による validated `DiagnosticRegistry` lookup、descriptor
     consistency validation、baseline compatibility validation を提供する。tests は
     initial allocation list、metadata round-trip、malformed/deferred range rejection、
     retirement finality、code-reuse rejection、semantic rename alias requirement、
     alias-domain determinism を固定する。verification は
     `cargo test -p mizar-diagnostics`、
     `cargo clippy -p mizar-diagnostics --all-targets -- -D warnings`、
     `cargo fmt --check` が通った。

4. **仕様: `failure_record.md`。** [x]
   - レコードの仕様を執筆する（英語と日本語、コードなし）:
     `DiagnosticDraft` と `DiagnosticRecord` の形、アーキテクチャ 19 に
     従う安定した failure カテゴリ、primary/secondary スパン、構造化
     詳細、機械可読ペイロードの規則。
   - 依存: 2。仕様: [19.failure_semantics.md](../../architecture/ja/19.failure_semantics.md)、
     [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)。
   - task 4 で完了: `failure_record.md` は 2 段階の draft/record lifecycle、
     stable code を持つ共有 field、`SourceRange` に基づく primary/secondary span、
     snapshot freshness state、安定した failure category、structured-detail payload
     rules、note payload、後続の fix/explain task のための attachment slot、
     deterministic debug rendering requirements、proof、driver、LSP、cache、artifact
     authority を record model の外に保つ boundary rules を定義する。verification は
     `git diff --check` と `git diff --cached --check` が通った。

5. **レコードとドラフトの実装。** [x]
   - スパン・詳細テーブルと決定的 debug レンダリングを備えたドラフトと
     レコードを実装する。
   - テスト: レコードのラウンドトリップ。スパンは必ず `SourceId` を参照
     する。レンダリングの安定性。
   - 依存: 3、4。仕様: `failure_record.md`。
   - task 5 で完了: `src/failure_record.rs` は validated `DiagnosticDraft` と
     immutable `DiagnosticRecord`、source snapshot/freshness state、snapshot-scoped
     handle、stable failure category、zero-width intent 付き span validation、
     deterministic key grammar と value ordering を持つ structured detail map、note
     payload、structured fix storage と explanation attachment slot、
     registry descriptor projection、
     deterministic debug snapshot を提供する。tests は structural draft-to-record
     round-trip、`SourceId` に基づく span invariant、detail-key validation と sorted
     details、byte-stable debug output、stale/current freshness validation、current
     record での retired-code rejection、related-handle snapshot boundary を覆う。
     verification は `cargo test -p mizar-diagnostics`、
     `cargo clippy -p mizar-diagnostics --all-targets -- -D warnings`、
     `cargo fmt --check` が通った。

### 生産と集約

6. **仕様: `sink.md`。** [x]
   - 生産者 API の仕様を執筆する（英語と日本語、コードなし）:
     `DiagnosticSink`、phase 側のドラフト発行規則、生産者がしてはなら
     ないこと（CLI 整形なし、LSP 形なし）。
   - 依存: 4。仕様: [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)
     「Diagnostic Producer API」。
   - task 6 で完了: `sink.md` は producer scope、`DiagnosticSink` /
     `DiagnosticBatch` behavior、sink-level phase/snapshot validation、immutable draft
     preservation、local production order、deterministic debug data、producer emission
     rules、sink error、formatting、LSP protocol shape、proof/phase status、driver
     orchestration、artifact mutation、consumer migration を sink の外に保つ boundary を
     定義する。verification は `git diff --check` と `git diff --cached --check` が
     通った。

7. **sink の実装。** [x]
   - 集約に備えた phase ごとのドラフト収集を持つ sink を実装する。
   - テスト: 模擬 phase をまたぐ sink のフィクスチャ。ドラフトが無変更で
     保存される。
   - 依存: 5、6。仕様: `sink.md`。
   - task 7 で完了: `src/sink.rs` は `DiagnosticProducerScope`、
     `DiagnosticSink`、immutable `DiagnosticBatch`、`DiagnosticSinkError` を提供する。
     sink は scope と phase/source snapshot が一致する validated draft を受け取り、
     sealed または mismatched emit を previously collected draft を mutate せず reject
     し、draft を local order で保存し、byte-stable batch debug snapshot を公開する。
     tests は local-order preservation、non-mutating failed emits、sealed behavior、
     consumed-batch preservation、empty/non-empty debug snapshot、crate boundary guard を
     覆う。verification は `cargo test -p mizar-diagnostics`、
     `cargo clippy -p mizar-diagnostics --all-targets -- -D warnings`、
     `cargo fmt --check` が通った。

8. **仕様: `aggregator.md`。** [x]
   - 集約の仕様を執筆する（英語と日本語、コードなし）: 正規化、識別の
     割り当て、重複排除、正準ソート順、`BuildDiagnosticIndex`、古い
     snapshot の規則（古い snapshot 由来の診断を現在のものとして公開
     しない）。
   - 依存: 4、6、7。仕様: [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)
     「Diagnostic Aggregator」、アーキテクチャ 19。
   - task 8 で完了: `aggregator.md` は sealed batch からの current-snapshot aggregation、
     immutable `BuildDiagnosticIndex` semantics、deterministic source key、strict な
     obsolete-snapshot filtering、stable structured deduplication identity、production
     order に依存しない sorting/handle assignment、debug snapshot、boundary rules を定義
     する。phase status join、workspace path normalization、driver/LSP/artifact/resolver
     adoption、legacy diagnostic migration は task 9 の scope 外または placeholder API では
     なく `external_dependency_gap` として記録した。

9. **集約の実装。** [x]
   - 生産順に依存しない決定的順序を持つ不変の `BuildDiagnosticIndex` への
     集約を実装する。
   - テスト: 入力をシャッフルしても同一の索引。重複排除のフィクスチャ。
     snapshot-scoped id determinism。古い snapshot の拒否。code/phase/primary-span
     が同じでも structured details、fix edits、explanation refs が異なる record を
     merge しない negative dedup cases。
   - 依存: 7、8。仕様: `aggregator.md`。
   - task 9 で完了: `src/aggregator.rs` は `DiagnosticAggregationInput`、
     deterministic `DiagnosticSourceKey`、obsolete draft accounting、immutable
     `BuildDiagnosticIndex`、`DiagnosticAggregationError` を提供する。aggregation は
     sealed batch を consume し、publication snapshot ではない draft を current record
     から除外し、message text ではなく stable structured identity で deduplicate し、
     representative を deterministic に選び、canonical sort 後に dense snapshot-local
     handle を assign し、by-source/by-id lookup と byte-stable debug snapshot を公開する。
     tests は shuffled input determinism、message-independent deduplication、details/fixes/
     explanations の negative dedup case、obsolete snapshot withholding、snapshot-scoped id
     lookup、debug output を覆う。verification は `cargo test -p mizar-diagnostics`、
     `cargo clippy -p mizar-diagnostics --all-targets -- -D warnings`、
     `cargo fmt --check` が通った。

### 表示

10. **仕様: `render.md`。** [x]
    - CLI レンダリングの仕様を執筆する（英語と日本語、コードなし）:
      メッセージレイアウト、スパン抜粋、重大度のスタイル、レンダリングは
      コードメタデータをキーにするという規則。
    - 依存: 8。仕様: [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)、
      アーキテクチャ 12。
    - task 10 で完了: `render.md` は CLI rendering を `DiagnosticRecord` と
      caller-supplied source context からの deterministic projection として定義する。
      header layout、span/source-block layout、Unicode-scalar column、missing-source
      fallback、note/help projection、実装 task 前の bounded fix/explanation reference、
      plain/styled output option、そして code identity、aggregation、source loading、LSP
      conversion、proof/phase status、driver orchestration、artifact mutation を rendering
      authority の外に保つ boundary rules を定義した。

11. **CLI レンダリング。** [x]
    - レコードと行マップから決定的な CLI レンダリングを実装する。
    - テスト: golden ファイルのレンダリングフィクスチャ。バイト同一の
      出力。workspace-relative paths、primary/secondary spans、multiline spans、
      Unicode-scalar column counts、notes、fix/help projections の coverage。
    - 依存: 9、10。仕様: `render.md`。
    - task 11 で完了: `src/render.rs` は `DiagnosticSourceContext`、
      `RenderOptions`、`RenderStyle`、`DiagnosticRenderInput`、`render_diagnostics` を
      提供する。rendering は input order を保存し、code/severity/semantic header を emit
      し、caller-supplied path/source key/line-column data を読み、primary/secondary source
      block と note span を render し、note、structured fix payload、explanation ref を
      bounded text として project し、byte-stable plain output と ANSI header styling を
      support し、source context が欠ける場合も deterministic fallback を使う。tests は byte-stable plain
      rendering、secondary/note span、fix/help と explanation projection、missing-source
      fallback、multi-diagnostic separator、input ordering、ANSI header styling を覆う。
      verification は `cargo test -p mizar-diagnostics`、
      `cargo clippy -p mizar-diagnostics --all-targets -- -D warnings`、
      `cargo fmt --check` が通った。

12. **仕様: `fix.md`。** [x]
    - fix 提案の仕様を執筆する（英語と日本語、コードなし）: 構造化された
      編集提案、適用可能性レベル、安全規則（提案は決して自動適用され
      ない）。
    - 依存: 4。仕様: アーキテクチャ 12「Fix Suggestion」。
    - task 12 で完了: `fix.md` は structured fix suggestion を diagnostic record に
      attached される bounded advisory payload として定義する。stable suggestion identity、
      edit payload shape、applicability/safety class、source-range edit validation、
      expected-text/snapshot/hash precondition、attachment/deduplication rule、debug
      snapshot、そして automatic application、LSP code-action conversion、current-buffer
      validation、command execution、artifact mutation、driver orchestration、
      proof/kernel acceptance を fix authority の外に保つ boundary を定義した。

13. **fix 提案。** [x]
    - レコードに付く構造化 fix ペイロードを実装する。
    - テスト: fix のラウンドトリップ。編集が有効な範囲を参照する。
    - 依存: 5、12。仕様: `fix.md`。
    - task 13 で完了: `src/fix.rs` は stable `FixSuggestionId`、optional
      producer key、structured edit payload、applicability/safety metadata、
      command ref、snapshot/hash precondition、deterministic edit ordering、
      overlap/range validation、byte-stable debug snapshot を提供する。
      `DiagnosticDraft` と `DiagnosticRecord` は normalized structured fix を保持し、
      aggregation dedup key は title/message text を除外しながら canonical fix payload を
      含める。CLI rendering は bounded help text を projection するだけで、edit の
      application や LSP code action は作らない。

14. **仕様: `explain.md`。** [x]
    - explanation の仕様を執筆する（英語と日本語、コードなし）: 遅延
      explanation ハンドル、上限付きプレビュー、「大きな trace は
      artifact または専用ファイルに留まる」という規則。
    - 依存: 4。仕様: [internal 03](../../internal/ja/03.diagnostics_model_and_lsp_bridge.md)
      「Explanation Store」。
    - task 14 で完了: `explain.md` は lazy explanation handle を kind、subject、source
      reference、snapshot/artifact/hash precondition、optional bounded preview、lazy resolution
      status、canonical explanation identity、deterministic debug snapshot を持つ compact structured
      reference として定義する。trace generation、proof/kernel/trusted status、artifact/cache
      mutation、LSP request shaping、driver orchestration、source loading、protocol conversion は
      explain authority の外に保つ。

15. **explanation ストア。** [ ]
    - 遅延解決と上限付きプレビューを備えた explanation ストアを実装する。
    - テスト: ハンドル解決のフィクスチャ。プレビュー上限の強制。裏付け
      データ欠落時の明確な劣化。
    - 依存: 13、14。仕様: `explain.md`。

### 採用とフォローアップ

16. **消費者の採用と移行の決定。** [ ]
    - 最初の消費者（`mizar-resolve`）および既存 lexer/frontend/parser 診断に
      real consumer adoption seam が存在するかを判断する。resolver/LSP/driver
      または frontend-family の adoption seam が未準備なら、ここおよびトップ
      レベルに `external_dependency_gap` または `deferred` の disposition を
      記録する。
    - real seam が存在する場合のみ、その consumer を sink と集約に接続し、
      対応するエンドツーエンドテストを追加する。real seam が存在しない場合、
      この task は documentation-only とする。
    - placeholder adapter、stub API、fake resolver adoption、provisional
      conversion layer、またはこの crate からの `mizar-driver` / `mizar-lsp`
      dependency を追加しない。
    - テスト: adoption が deferred の場合は documentation verification。そうで
      ない場合は、real adopted consumer ごとの end-to-end flow、owning
      consumer が選んだ real conversion adapter の round-trip、user-facing
      language diagnostics を migrate する場合の consumer corpus または `.miz`
      coverage。
    - 依存: 9、`mizar-resolve` task 15。仕様: `aggregator.md`。

17. **決定性スイート。** [ ]
    - 同一入力が同一のレコード、索引、レンダリング出力、explanation
      プレビューを生むことのプロパティ的検証。
    - 依存: 11、15。仕様: [20.test_strategy.md](../../architecture/ja/20.test_strategy.md)。

18. **公開 enum の前方互換性ポリシー。** [ ]
    - 各公開 enum に `mizar-frontend` task 25 の手続きを適用する。重大度と
      カテゴリの enum はさらにアーキテクチャ 19 の互換性ポリシーに従う。
    - 依存: 16。仕様: 全モジュール仕様。

19. **ソース/仕様対応監査。** [ ]
    - モジュール仕様の全公開 API と約束された挙動を実装とテストへ
      トレースし、ギャップをフォローアップタスクとして記録する。
    - 依存: 18。仕様: 全モジュール仕様と本 TODO。

20. **二言語ドキュメント同期監査。** [ ]
    - `doc/design/mizar-diagnostics/en/` の各英語正本と日本語版を比較し、
      内容を同期する。
    - 依存: 19。仕様: リポジトリのドキュメント方針。

21. **module 境界リファクタリング gate。** [ ]
    - crate を下流 consumer 向けに完了扱いにする前に、source layout を監査し、
      oversized file、混在した責務、module table と module spec 境界に沿って
      分割すべき private helper を洗い出す。review bottleneck になった実装
      ファイルは、公開 API、診断、決定的 rendering、artifact-facing schema、
      consumer-visible behavior を変えずに private module へ分割する。
    - 分割後は必要に応じて本 module table / source path を更新し、移動した
      API について source/spec 対応監査と二言語ドキュメント同期監査の範囲を
      再実行する。挙動 cleanup や API 公開を移動と混ぜない。それらは独立した
      spec task を要求する。
    - 依存: 20。仕様: 本 TODO、
      [internal 07](../../internal/ja/07.crate_module_layout.md)、全モジュール仕様。

## 推奨検証

各タスクの後で実行する:

```text
cargo test -p mizar-diagnostics
cargo clippy -p mizar-diagnostics --all-targets -- -D warnings
```

採用/移行のタスクでは消費側も実行する:

```text
cargo test -p mizar-resolve
cargo test -p mizar-frontend
cargo test -p mizar-parser
cargo test -p mizar-lexer
cargo test -p mizar-lsp
cargo test -p mizar-build
```

実行するのは、その task が触れた implemented seam を持つ consumer command のみとする。
名前が挙がっている consumer crate または `mizar-driver` integration seam が存在しない
場合は、placeholder を追加せず `external_dependency_gap` として記録する。

テストが通ったらここでタスクにチェックを付ける。

## 備考

- ツールは `DiagnosticCode` をキーにする。メッセージはバージョン間で
  改善してよいが、コードは恒久的で、別の意味での再利用は決してない。
- 集約の出力は不変かつ決定的順序であり、生産順や並列性が透けて見える
  ことはない。
- open バッファ（LSP オーバーレイ）診断はレコードの形を再利用するが、
  CLI 出力や `VerifiedArtifact` には決して公開されない。ブリッジの
  ロジックは `mizar-lsp` にある。
- 大きな trace が診断にインラインで置かれることは決してない — コンパクトな
  参照と上限付きプレビューのみ。
